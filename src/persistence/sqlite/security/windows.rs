//! Windows ACL boundary using the qualified OS APIs, not shell commands. New
//! directories have a protected inheritable DACL for this user and LocalSystem.
use super::*;
use std::{
    os::windows::{
        ffi::OsStrExt,
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    ptr,
};
use windows_sys::Win32::{
    Foundation::*,
    Security::{Authorization::*, *},
    Storage::FileSystem::*,
    System::{Threading::*, WindowsProgramming::DRIVE_FIXED},
};
fn wide(p: &Path) -> Result<Vec<u16>, PersistenceError> {
    let mut v: Vec<u16> = p.as_os_str().encode_wide().collect();
    if v.contains(&0) {
        return Err(bad());
    }
    v.push(0);
    Ok(v)
}
struct Local(*mut core::ffi::c_void);
impl Drop for Local {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: only LocalAlloc-owned results from security APIs enter this guard.
            unsafe {
                LocalFree(self.0);
            }
        }
    }
}
struct User {
    data: Vec<usize>,
}
impl User {
    fn sid(&self) -> PSID {
        // SAFETY: successful GetTokenInformation initialized aligned TOKEN_USER data;
        // its SID refers into data, which remains alive for the whole User value.
        unsafe { (*(self.data.as_ptr() as *const TOKEN_USER)).User.Sid }
    }
    fn current() -> Result<Self, PersistenceError> {
        let mut token = ptr::null_mut();
        // SAFETY: output handle storage is writable; the pseudo process is valid.
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
            return Err(bad());
        }
        // SAFETY: OpenProcessToken returned an owned, valid kernel handle.
        let token = unsafe { OwnedHandle::from_raw_handle(token) };
        let mut needed = 0;
        // SAFETY: null buffer/zero size requests the required byte count.
        unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                ptr::null_mut(),
                0,
                &mut needed,
            );
        }
        if needed == 0 {
            return Err(bad());
        }
        let mut data = vec![0usize; (needed as usize).div_ceil(size_of::<usize>())];
        // SAFETY: data is aligned and has at least needed writable bytes; token lives.
        if unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                data.as_mut_ptr().cast(),
                needed,
                &mut needed,
            )
        } == 0
        {
            return Err(bad());
        }
        Ok(Self { data })
    }
    fn text(&self) -> Result<String, PersistenceError> {
        let mut value = ptr::null_mut();
        // SAFETY: the SID lives in self; output is owned LocalAlloc memory.
        if unsafe { ConvertSidToStringSidW(self.sid(), &mut value) } == 0 {
            return Err(bad());
        }
        let _owned = Local(value.cast());
        let mut len = 0;
        // SAFETY: the OS returned a NUL-terminated UTF16 SID. Its specified textual
        // representation is far shorter than this defensive bound; no SID is logged.
        unsafe {
            while len < 256 && *value.add(len) != 0 {
                len += 1;
            }
        }
        if len == 256 {
            return Err(bad());
        }
        // SAFETY: the preceding scan established a valid initialized range.
        String::from_utf16(unsafe { std::slice::from_raw_parts(value, len) }).map_err(|_| bad())
    }
}
pub(super) fn create_directory(path: &Path) -> Result<(), PersistenceError> {
    let user = User::current()?;
    let sid = user.text()?;
    let sddl = format!("O:{sid}D:P(A;OICI;FA;;;{sid})(A;OICI;FA;;;SY)");
    let mut sddl: Vec<u16> = sddl.encode_utf16().collect();
    sddl.push(0);
    let mut descriptor = ptr::null_mut();
    // SAFETY: terminated SDDL input and writable output pointer, standard revision1.
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor,
            ptr::null_mut(),
        )
    } == 0
    {
        return Err(bad());
    }
    let _owned = Local(descriptor);
    let attrs = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor,
        bInheritHandle: 0,
    };
    let path = wide(path)?;
    // SAFETY: path/descriptor/attributes remain valid through the synchronous call.
    if unsafe { CreateDirectoryW(path.as_ptr(), &attrs) } == 0 {
        return Err(bad());
    }
    Ok(())
}
pub(super) fn open_directory(path: &Path) -> Result<File, PersistenceError> {
    // No FILE_SHARE_DELETE: keep the verified root pinned while SQLite uses its
    // pathname for journal creation. Reparse points are opened, never traversed.
    OpenOptions::new()
        .read(true)
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|_| bad())
}
pub(super) fn check_private(
    path: &Path,
    _meta: &fs::Metadata,
    dir: bool,
) -> Result<(), PersistenceError> {
    let user = User::current()?;
    let path = wide(path)?;
    let (mut owner, mut dacl, mut descriptor) = (ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
    // SAFETY: all outputs are writable and path is terminated. Returned owner/ACL
    // pointers borrow descriptor, kept alive until after the entire validation.
    if unsafe {
        GetNamedSecurityInfoW(
            path.as_ptr(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner,
            ptr::null_mut(),
            &mut dacl,
            ptr::null_mut(),
            &mut descriptor,
        )
    } != ERROR_SUCCESS
    {
        return Err(bad());
    }
    let _owned = Local(descriptor);
    // SAFETY: both owner and user SID are valid while their owners remain alive.
    if owner.is_null() || dacl.is_null() || unsafe { EqualSid(owner, user.sid()) } == 0 {
        return Err(bad());
    }
    let (mut control, mut revision) = (0, 0);
    // SAFETY: descriptor is OS-validated; scalar output storage is initialized.
    if unsafe { GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) } == 0
        || (dir && control & SE_DACL_PROTECTED == 0)
    {
        return Err(bad());
    }
    let (mut self_access, mut system_access) = (false, false);
    // SAFETY: dacl is a validated ACL inside descriptor; GetAce validates indices.
    let count = unsafe { (*dacl).AceCount };
    for index in 0..u32::from(count) {
        let mut entry = ptr::null_mut();
        // SAFETY: the OS ACL lives and entry output is writable.
        if unsafe { GetAce(dacl, index, &mut entry) } == 0 {
            return Err(bad());
        }
        // SAFETY: every valid ACE begins with ACE_HEADER. Accept only ordinary allow
        // ACEs before interpreting its larger ACCESS_ALLOWED_ACE representation.
        let header = unsafe { &*(entry as *const ACE_HEADER) };
        if header.AceType != 0 || usize::from(header.AceSize) < size_of::<ACCESS_ALLOWED_ACE>() {
            return Err(bad());
        }
        // SAFETY: ACE type and size checked; SID is variable-tail starting at SidStart.
        let ace = unsafe { &*(entry as *const ACCESS_ALLOWED_ACE) };
        let sid = ptr::addr_of!(ace.SidStart) as PSID;
        // SAFETY: SID pointers are valid inside the checked ACE/user buffers.
        let own = unsafe { EqualSid(sid, user.sid()) } != 0;
        // SAFETY: the SID is supplied by the validated security descriptor.
        let system = unsafe { IsWellKnownSid(sid, WinLocalSystemSid) } != 0;
        if (!own && !system) || ace.Mask & FILE_ALL_ACCESS != FILE_ALL_ACCESS {
            return Err(bad());
        }
        if dir
            && (u32::from(header.AceFlags) & (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE))
                != (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE)
        {
            return Err(bad());
        }
        self_access |= own;
        system_access |= system;
    }
    if !self_access || !system_access {
        return Err(bad());
    }
    Ok(())
}
pub(super) fn check_filesystem(path: &Path) -> Result<(), PersistenceError> {
    let path = wide(path)?;
    let mut volume = vec![0u16; 32768];
    // SAFETY: terminated path and sufficiently sized writable UTF16 volume buffer.
    if unsafe { GetVolumePathNameW(path.as_ptr(), volume.as_mut_ptr(), volume.len() as u32) } == 0 {
        return Err(bad());
    }
    // SAFETY: preceding API produced a terminated volume path.
    if unsafe { GetDriveTypeW(volume.as_ptr()) } != DRIVE_FIXED {
        return Err(PersistenceError::UnsupportedStorageFilesystem);
    }
    let mut filesystem = [0u16; 64];
    // SAFETY: unused outputs may be null; filesystem is sized writable storage.
    if unsafe {
        GetVolumeInformationW(
            volume.as_ptr(),
            ptr::null_mut(),
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            filesystem.as_mut_ptr(),
            filesystem.len() as u32,
        )
    } == 0
    {
        return Err(bad());
    }
    let end = filesystem.iter().position(|v| *v == 0).ok_or_else(bad)?;
    if String::from_utf16_lossy(&filesystem[..end]) != "NTFS" {
        return Err(PersistenceError::UnsupportedStorageFilesystem);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inherited_private_acl_is_required_and_world_access_is_rejected() {
        let base = std::env::current_dir()
            .unwrap()
            .join("target/sqlite-acl-tests");
        fs::create_dir_all(&base).unwrap();
        let parent = tempfile::tempdir_in(base).unwrap();
        let root = parent.path().join("archive");
        create_directory(&root).unwrap();
        let file = open_directory(&root).unwrap();
        check_private(&root, &file.metadata().unwrap(), true).unwrap();
        drop(file);
        let mut sddl: Vec<u16> = "D:P(A;OICI;FA;;;WD)".encode_utf16().collect();
        sddl.push(0);
        let mut descriptor = ptr::null_mut();
        // SAFETY: terminated fixed synthetic SDDL, writable output descriptor.
        assert_ne!(
            unsafe {
                ConvertStringSecurityDescriptorToSecurityDescriptorW(
                    sddl.as_ptr(),
                    1,
                    &mut descriptor,
                    ptr::null_mut(),
                )
            },
            0
        );
        let _owned = Local(descriptor);
        let (mut present, mut dacl, mut defaulted) = (0, ptr::null_mut(), 0);
        // SAFETY: descriptor is valid and all scalar/ACL outputs are writable.
        assert_ne!(
            unsafe {
                GetSecurityDescriptorDacl(descriptor, &mut present, &mut dacl, &mut defaulted)
            },
            0
        );
        let root_wide = wide(&root).unwrap();
        // SAFETY: the ACL borrows the still-live descriptor and applies only to
        // this test's owned directory; owner is preserved, no external ACL touched.
        assert_eq!(
            unsafe {
                SetNamedSecurityInfoW(
                    root_wide.as_ptr(),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    dacl,
                    ptr::null_mut(),
                )
            },
            ERROR_SUCCESS
        );
        assert_eq!(
            check_private(&root, &fs::metadata(&root).unwrap(), true).unwrap_err(),
            PersistenceError::UnsafeStorageLocation
        );
    }
}
