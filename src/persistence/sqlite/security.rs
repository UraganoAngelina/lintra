//! Local storage boundaries. Creation is explicit and private; opening never
//! repairs permissions or follows a path into a different archive silently.
use crate::PersistenceError;
use std::{
    fs::{self, File, OpenOptions},
    path::{Component, Path, PathBuf},
};

pub(super) struct RootGuard {
    pub path: PathBuf,
    _directory: File,
}
fn bad() -> PersistenceError {
    PersistenceError::UnsafeStorageLocation
}
pub(super) fn root(path: &Path, create: bool) -> Result<RootGuard, PersistenceError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
        || path.file_name().is_none()
    {
        return Err(bad());
    }
    let mut prefix = PathBuf::new();
    for component in path.components() {
        prefix.push(component.as_os_str());
        if matches!(component, Component::Prefix(_) | Component::RootDir) {
            continue;
        }
        match fs::symlink_metadata(&prefix) {
            Ok(meta) => {
                if meta.file_type().is_symlink() || reparse(&meta) {
                    return Err(bad());
                }
                check_ancestor(&meta)?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && create && prefix == path => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(PersistenceError::StorageUninitialized);
            }
            Err(_) => return Err(PersistenceError::StorageUnavailable),
        }
    }
    if create && !path.exists() {
        create_directory(path)?;
    }
    if !path.is_dir() {
        return Err(bad());
    }
    let directory = open_directory(path)?;
    check_private(path, &directory.metadata().map_err(|_| bad())?, true)?;
    check_filesystem(path)?;
    Ok(RootGuard {
        path: path.to_owned(),
        _directory: directory,
    })
}
pub(super) fn check_files(root: &Path, create: bool) -> Result<(), PersistenceError> {
    let entries = fs::read_dir(root).map_err(|_| PersistenceError::StorageUnavailable)?;
    for entry in entries {
        let entry = entry.map_err(|_| PersistenceError::StorageUnavailable)?;
        if create {
            return Err(PersistenceError::StorageAlreadyExists);
        }
        let name = entry.file_name();
        if name != "lintra.db" && name != "lintra.db-journal" {
            return Err(bad());
        }
        let meta = fs::symlink_metadata(entry.path()).map_err(|_| bad())?;
        if !meta.is_file() || meta.file_type().is_symlink() || reparse(&meta) {
            return Err(bad());
        }
        check_private(&entry.path(), &meta, false)?;
    }
    Ok(())
}
pub(super) fn database(root: &Path, create: bool) -> Result<File, PersistenceError> {
    let path = root.join("lintra.db");
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    if create {
        options.create_new(true);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::*;
        options
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options.open(&path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => PersistenceError::StorageUninitialized,
        std::io::ErrorKind::AlreadyExists => PersistenceError::StorageAlreadyExists,
        _ => PersistenceError::StorageUnavailable,
    })?;
    let meta = file.metadata().map_err(|_| bad())?;
    if !meta.is_file() || reparse(&meta) {
        return Err(bad());
    }
    check_private(&path, &meta, false)?;
    if !create && meta.len() == 0 {
        return Err(PersistenceError::StorageUninitialized);
    }
    Ok(file)
}
#[cfg(unix)]
fn reparse(_: &fs::Metadata) -> bool {
    false
}
#[cfg(windows)]
fn reparse(m: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    m.file_attributes() & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT != 0
}
#[cfg(unix)]
fn check_ancestor(m: &fs::Metadata) -> Result<(), PersistenceError> {
    use std::os::unix::fs::PermissionsExt;
    // Shared writable parents require sticky protection (e.g. /tmp); otherwise
    // another unprivileged user could rename a supposedly private child directory.
    if m.permissions().mode() & 0o022 != 0 && m.permissions().mode() & 0o1000 == 0 {
        return Err(bad());
    }
    Ok(())
}
#[cfg(windows)]
fn check_ancestor(_: &fs::Metadata) -> Result<(), PersistenceError> {
    Ok(())
}
#[cfg(unix)]
fn create_directory(p: &Path) -> Result<(), PersistenceError> {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new()
        .mode(0o700)
        .create(p)
        .map_err(|_| bad())
}
#[cfg(unix)]
fn open_directory(p: &Path) -> Result<File, PersistenceError> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
        .open(p)
        .map_err(|_| bad())
}
#[cfg(unix)]
fn check_private(_: &Path, m: &fs::Metadata, dir: bool) -> Result<(), PersistenceError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    // SAFETY: geteuid has no arguments or pointer preconditions.
    let uid = unsafe { libc::geteuid() };
    if m.uid() != uid
        || m.permissions().mode() & 0o7777 != if dir { 0o700 } else { 0o600 }
        || (!dir && m.nlink() != 1)
    {
        return Err(bad());
    }
    Ok(())
}
#[cfg(target_os = "linux")]
fn check_filesystem(p: &Path) -> Result<(), PersistenceError> {
    use std::os::unix::ffi::OsStrExt;
    let name = std::ffi::CString::new(p.as_os_str().as_bytes()).map_err(|_| bad())?;
    let mut info = std::mem::MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: a NUL-terminated path and correctly sized writable statfs storage.
    if unsafe { libc::statfs(name.as_ptr(), info.as_mut_ptr()) } != 0 {
        return Err(bad());
    }
    // SAFETY: successful statfs initialized the complete structure.
    let kind = unsafe { info.assume_init() }.f_type as u64;
    if kind != 0xef53 {
        return Err(PersistenceError::UnsupportedStorageFilesystem);
    }
    Ok(())
}
#[cfg(all(unix, not(target_os = "linux")))]
fn check_filesystem(_: &Path) -> Result<(), PersistenceError> {
    Err(PersistenceError::UnsupportedStorageFilesystem)
}

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows::{check_filesystem, check_private, create_directory, open_directory};
