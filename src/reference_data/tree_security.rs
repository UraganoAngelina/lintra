//! Read-only deployment checks for reference integrity. Reference values are
//! public dictionaries, so read access need not be private. Modification rights
//! must remain with this account or OS administrators. No ACL is repaired here.
use std::{fs::Metadata, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceTreeViolation {
    UntrustedOwner,
    SharedMutation,
    SecurityInspection,
    UnsupportedAcl,
}

#[cfg(unix)]
pub(super) fn check(_path: &Path, meta: &Metadata) -> Result<(), ReferenceTreeViolation> {
    use std::os::unix::fs::MetadataExt;
    // SAFETY: geteuid reads the effective process identity and has no arguments.
    let uid = unsafe { libc::geteuid() };
    if meta.uid() != uid && meta.uid() != 0 {
        return Err(ReferenceTreeViolation::UntrustedOwner);
    }
    // A group is not automatically trusted merely because this user belongs to
    // it. Operators can select a controlled tree; the application never chmods
    // an author's files or invents a corporate group authorization.
    if meta.mode() & 0o022 != 0 {
        return Err(ReferenceTreeViolation::SharedMutation);
    }
    Ok(())
}

#[cfg(windows)]
pub(super) fn check(path: &Path, _meta: &Metadata) -> Result<(), ReferenceTreeViolation> {
    use ReferenceTreeViolation as V;
    use std::{
        os::windows::{
            ffi::OsStrExt,
            io::{AsRawHandle, FromRawHandle, OwnedHandle},
        },
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::*,
        Security::{Authorization::*, *},
        Storage::FileSystem::*,
        System::Threading::*,
    };

    struct Local(*mut core::ffi::c_void);
    impl Drop for Local {
        fn drop(&mut self) {
            // SAFETY: only LocalAlloc-owned GetNamedSecurityInfo results enter here.
            if !self.0.is_null() {
                unsafe {
                    LocalFree(self.0);
                }
            }
        }
    }
    let mut name = path.as_os_str().encode_wide().collect::<Vec<_>>();
    if name.contains(&0) {
        return Err(V::SecurityInspection);
    }
    name.push(0);
    let (mut owner, mut dacl, mut descriptor) = (ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
    // SAFETY: terminated path and writable outputs; descriptor owns owner/ACL.
    if unsafe {
        GetNamedSecurityInfoW(
            name.as_ptr(),
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
        return Err(V::SecurityInspection);
    }
    let _descriptor = Local(descriptor);
    if owner.is_null() || dacl.is_null() {
        return Err(V::SecurityInspection);
    }
    let mut token = ptr::null_mut();
    // SAFETY: current-process pseudo handle and writable output handle storage.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(V::SecurityInspection);
    }
    // SAFETY: OpenProcessToken produced an owned live handle.
    let token = unsafe { OwnedHandle::from_raw_handle(token) };
    let mut needed = 0;
    // SAFETY: null/zero queries the required token buffer size.
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
        return Err(V::SecurityInspection);
    }
    let mut buffer = vec![0usize; (needed as usize).div_ceil(size_of::<usize>())];
    // SAFETY: aligned, sufficiently sized output and a live token handle.
    if unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            buffer.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    } == 0
    {
        return Err(V::SecurityInspection);
    }
    // SAFETY: successful OS call initialized TOKEN_USER and its borrowed SID.
    let user = unsafe { (*(buffer.as_ptr() as *const TOKEN_USER)).User.Sid };
    let trusted = |sid: PSID| {
        // SAFETY: SIDs borrow live OS-validated descriptor/token buffers.
        unsafe {
            EqualSid(sid, user) != 0
                || IsWellKnownSid(sid, WinLocalSystemSid) != 0
                || IsWellKnownSid(sid, WinBuiltinAdministratorsSid) != 0
        }
    };
    if !trusted(owner) {
        return Err(V::UntrustedOwner);
    }
    // These bits can mutate contents, names or the access boundary. Generic
    // rights are included, while broad read/execute rights remain allowed.
    let mutation = FILE_WRITE_DATA
        | FILE_APPEND_DATA
        | FILE_WRITE_EA
        | FILE_WRITE_ATTRIBUTES
        | FILE_DELETE_CHILD
        | DELETE
        | WRITE_DAC
        | WRITE_OWNER
        | GENERIC_WRITE
        | GENERIC_ALL;
    // SAFETY: dacl belongs to the live descriptor returned by Windows.
    for index in 0..u32::from(unsafe { (*dacl).AceCount }) {
        let mut entry = ptr::null_mut();
        // SAFETY: validated ACL and writable ACE output storage.
        if unsafe { GetAce(dacl, index, &mut entry) } == 0 {
            return Err(V::SecurityInspection);
        }
        // SAFETY: every returned ACE starts with a header.
        let header = unsafe { &*(entry as *const ACE_HEADER) };
        // Deny entries never grant mutation; inherit-only entries do not apply
        // to this object. Each used child is inspected independently below.
        if header.AceType == 1 || u32::from(header.AceFlags) & INHERIT_ONLY_ACE != 0 {
            continue;
        }
        if header.AceType != 0 || usize::from(header.AceSize) < size_of::<ACCESS_ALLOWED_ACE>() {
            return Err(V::UnsupportedAcl);
        }
        // SAFETY: ordinary allow ACE and minimum size checked; SID is its tail.
        let ace = unsafe { &*(entry as *const ACCESS_ALLOWED_ACE) };
        let sid = ptr::addr_of!(ace.SidStart) as PSID;
        if ace.Mask & mutation != 0 && !trusted(sid) {
            return Err(V::SharedMutation);
        }
    }
    Ok(())
}

#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;
    use std::{fs, os::windows::ffi::OsStrExt, ptr};
    use windows_sys::Win32::{
        Foundation::*,
        Security::{Authorization::*, *},
    };

    /// Only this test's synthetic owned file has its DACL changed. The helper
    /// never edits parents, the checkout, a real reference tree or storage ACLs.
    fn apply(path: &Path, text: &str) {
        let mut name: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
        let sddl: Vec<u16> = text.encode_utf16().chain([0]).collect();
        let mut descriptor = ptr::null_mut();
        // SAFETY: terminated input and writable descriptor output.
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
        let (mut present, mut defaulted, mut dacl) = (0, 0, ptr::null_mut());
        // SAFETY: live OS-produced descriptor and writable scalar/ACL outputs.
        assert_ne!(
            unsafe {
                GetSecurityDescriptorDacl(descriptor, &mut present, &mut dacl, &mut defaulted)
            },
            0
        );
        assert_ne!(present, 0);
        // SAFETY: live path/ACL, only the owned fixture DACL is updated; owner unchanged.
        let result = unsafe {
            SetNamedSecurityInfoW(
                name.as_mut_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                dacl,
                ptr::null_mut(),
            )
        };
        // SAFETY: LocalAlloc-owned descriptor, released after its ACL was used.
        unsafe {
            LocalFree(descriptor);
        }
        assert_eq!(result, ERROR_SUCCESS);
    }

    #[test]
    fn broad_read_is_allowed_but_broad_modification_is_rejected_without_repair() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("synthetic-reference.json");
        fs::write(&path, b"synthetic only").unwrap();
        let writable = "D:(A;;FA;;;AU)(A;;FA;;;SY)(A;;FA;;;BA)";
        apply(&path, writable);
        assert_eq!(
            check(&path, &fs::metadata(&path).unwrap()),
            Err(ReferenceTreeViolation::SharedMutation)
        );
        // Repeating inspection must not silently repair the original ACL.
        assert_eq!(
            check(&path, &fs::metadata(&path).unwrap()),
            Err(ReferenceTreeViolation::SharedMutation)
        );
        apply(&path, "D:(A;;FR;;;AU)(A;;FA;;;SY)(A;;FA;;;BA)");
        check(&path, &fs::metadata(&path).unwrap()).unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"synthetic only");
        // Return the disposable synthetic file to deletable permissions before
        // TempDir cleanup; no externally owned object is ever changed.
        apply(&path, writable);
    }
}
