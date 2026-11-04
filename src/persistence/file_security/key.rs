//! Read the specifically configured file through verified handles. All buffers
//! holding bytes are zeroizing, including partial reads and failing branches.
use super::*;
use crate::FileKeyError as E;
use std::io::Read;
use zeroize::Zeroizing;

struct Opened {
    file: File,
    parents: Vec<(PathBuf, File)>,
    path: PathBuf,
}

/// Callback is a private test seam, not a runtime fault injector. Replacement
/// tests run a real rename between opening and reading the protected file.
pub(in crate::persistence) fn read(
    source: &Path,
    after_open: impl FnOnce(),
) -> Result<Zeroizing<Vec<u8>>, E> {
    let path = std::path::absolute(source).map_err(|_| E::InvalidSource)?;
    if path.file_name().is_none()
        || path
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return Err(E::InvalidSource);
    }
    let mut opened = open(&path)?;
    validate(&opened)?;
    let before = signature(&opened.file)?;
    after_open();
    validate(&opened)?;
    if signature(&opened.file)? != before {
        return Err(E::ChangedSource);
    }
    // Fixed initialized allocation: unlike read_to_end, this cannot reallocate
    // a partially filled secret buffer and leave an old allocation unwiped.
    let mut bytes = Zeroizing::new(vec![0u8; 33]);
    let mut length = 0;
    while length < bytes.len() {
        match opened.file.read(&mut bytes[length..]) {
            Ok(0) => break,
            Ok(count) => length += count,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return Err(E::Unavailable),
        }
    }
    validate(&opened)?;
    if signature(&opened.file)? != before {
        return Err(E::ChangedSource);
    }
    if length != 32 {
        return Err(E::InvalidLength);
    }
    // The unused33rd initialized byte is zero. No secret is discarded unwiped.
    bytes.truncate(32);
    Ok(bytes)
}

fn validate(opened: &Opened) -> Result<(), E> {
    for (path, handle) in &opened.parents {
        let actual = handle.metadata().map_err(|_| E::Unavailable)?;
        let named = fs::symlink_metadata(path).map_err(|_| E::ChangedSource)?;
        if !actual.is_dir()
            || !named.is_dir()
            || reparse(&actual)
            || reparse(&named)
            || actual.file_type().is_symlink()
            || named.file_type().is_symlink()
            || !same_named(handle, path)?
        {
            return Err(E::ChangedSource);
        }
        check_ancestor(&actual).map_err(|_| E::UnsafeSource)?;
    }
    let actual = opened.file.metadata().map_err(|_| E::Unavailable)?;
    let named = fs::symlink_metadata(&opened.path).map_err(|_| E::ChangedSource)?;
    if !actual.is_file() || reparse(&actual) || actual.file_type().is_symlink() {
        return Err(E::UnsafeSource);
    }
    if !named.is_file()
        || reparse(&named)
        || named.file_type().is_symlink()
        || !same_named(&opened.file, &opened.path)?
    {
        return Err(E::ChangedSource);
    }
    #[cfg(unix)]
    check_private(&opened.path, &actual, false).map_err(|_| E::UnsafeSource)?;
    #[cfg(windows)]
    super::windows::check_open_key(&opened.file).map_err(|_| E::UnsafeSource)?;
    check_volume(&opened.file, &opened.path)?;
    Ok(())
}

#[cfg(unix)]
fn open(path: &Path) -> Result<Opened, E> {
    use std::os::{
        fd::{AsRawFd, FromRawFd},
        unix::ffi::OsStrExt,
    };
    let mut parents = vec![(
        PathBuf::from("/"),
        File::open("/").map_err(|_| E::Unavailable)?,
    )];
    let mut prefix = PathBuf::from("/");
    let parent_path = path.parent().ok_or(E::InvalidSource)?;
    for component in parent_path.components() {
        let Component::Normal(name) = component else {
            continue;
        };
        let name = std::ffi::CString::new(name.as_bytes()).map_err(|_| E::InvalidSource)?;
        let parent = &parents.last().ok_or(E::InvalidSource)?.1;
        // SAFETY: parent is an owned live directory fd, name is terminated, no
        // creation or variadic mode argument. O_NOFOLLOW applies to EACH edge.
        let fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err(E::UnsafeSource);
        }
        // SAFETY: successful openat transfers ownership of a new descriptor.
        let file = unsafe { File::from_raw_fd(fd) };
        check_ancestor(&file.metadata().map_err(|_| E::Unavailable)?)
            .map_err(|_| E::UnsafeSource)?;
        prefix.push(component.as_os_str());
        parents.push((prefix.clone(), file));
    }
    let name = std::ffi::CString::new(path.file_name().ok_or(E::InvalidSource)?.as_bytes())
        .map_err(|_| E::InvalidSource)?;
    let parent = &parents.last().ok_or(E::InvalidSource)?.1;
    // SAFETY: same checked fd/name conditions. O_NONBLOCK prevents a FIFO/device
    // from hanging before the regular-file check; no content is read yet.
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(E::Unavailable);
    }
    // SAFETY: successful openat returned a new descriptor owned by this value.
    let file = unsafe { File::from_raw_fd(fd) };
    Ok(Opened {
        file,
        parents,
        path: path.to_owned(),
    })
}
#[cfg(windows)]
fn open(path: &Path) -> Result<Opened, E> {
    use std::os::windows::fs::OpenOptionsExt;
    use std::path::Prefix;
    use windows_sys::Win32::Storage::FileSystem::*;
    if !matches!(path.components().next(), Some(Component::Prefix(p))
        if matches!(p.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
    {
        return Err(E::InvalidSource);
    }
    let mut parents = Vec::new();
    let mut prefix = PathBuf::new();
    for component in path.parent().ok_or(E::InvalidSource)?.components() {
        prefix.push(component.as_os_str());
        if matches!(component, Component::Prefix(_)) {
            continue;
        }
        let handle = open_directory(&prefix).map_err(|_| E::UnsafeSource)?;
        let meta = handle.metadata().map_err(|_| E::Unavailable)?;
        if !meta.is_dir() || reparse(&meta) {
            return Err(E::UnsafeSource);
        }
        parents.push((prefix.clone(), handle));
    }
    // Deny both write and delete sharing while loading. Reparse points are
    // opened themselves, not followed; validation inspects this exact handle.
    let file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|_| E::Unavailable)?;
    Ok(Opened {
        file,
        parents,
        path: path.to_owned(),
    })
}

#[cfg(unix)]
fn signature(file: &File) -> Result<Vec<u64>, E> {
    use std::os::unix::fs::MetadataExt;
    let m = file.metadata().map_err(|_| E::Unavailable)?;
    Ok(vec![
        m.dev(),
        m.ino(),
        m.size(),
        m.mode().into(),
        m.uid().into(),
        m.nlink(),
        m.mtime() as u64,
        m.mtime_nsec() as u64,
        m.ctime() as u64,
        m.ctime_nsec() as u64,
    ])
}
#[cfg(unix)]
fn same_named(file: &File, path: &Path) -> Result<bool, E> {
    use std::os::unix::fs::MetadataExt;
    let a = file.metadata().map_err(|_| E::Unavailable)?;
    let b = fs::symlink_metadata(path).map_err(|_| E::ChangedSource)?;
    Ok(a.dev() == b.dev() && a.ino() == b.ino())
}
#[cfg(windows)]
fn info(
    file: &File,
) -> Result<windows_sys::Win32::Storage::FileSystem::BY_HANDLE_FILE_INFORMATION, E> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::*;
    let mut value = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
    // SAFETY: owned live handle and correctly sized output storage.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), value.as_mut_ptr()) } == 0 {
        return Err(E::UnsafeSource);
    }
    // SAFETY: successful OS call initialized the whole output structure.
    Ok(unsafe { value.assume_init() })
}
#[cfg(windows)]
fn signature(file: &File) -> Result<Vec<u64>, E> {
    let i = info(file)?;
    if i.nNumberOfLinks != 1 {
        return Err(E::UnsafeSource);
    }
    Ok(vec![
        i.dwVolumeSerialNumber.into(),
        i.nFileIndexHigh.into(),
        i.nFileIndexLow.into(),
        i.nFileSizeHigh.into(),
        i.nFileSizeLow.into(),
        i.ftLastWriteTime.dwHighDateTime.into(),
        i.ftLastWriteTime.dwLowDateTime.into(),
    ])
}
#[cfg(windows)]
fn same_named(file: &File, path: &Path) -> Result<bool, E> {
    // Parent handles deny deletion; the key handle denies both write/deletion.
    // Reopening for metadata also verifies that the configured name still names
    // the held object, rather than accepting pathname-only security evidence.
    let other = open_directory(path).map_err(|_| E::ChangedSource)?;
    let a = info(file)?;
    let b = info(&other)?;
    Ok(a.dwVolumeSerialNumber == b.dwVolumeSerialNumber
        && a.nFileIndexHigh == b.nFileIndexHigh
        && a.nFileIndexLow == b.nFileIndexLow)
}
#[cfg(target_os = "linux")]
fn check_volume(file: &File, _: &Path) -> Result<(), E> {
    use std::os::fd::AsRawFd;
    let mut data = std::mem::MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: live descriptor and correctly sized writable statfs storage.
    if unsafe { libc::fstatfs(file.as_raw_fd(), data.as_mut_ptr()) } != 0 {
        return Err(E::UnsafeSource);
    }
    // SAFETY: successful fstatfs initialized the complete structure.
    let kind = unsafe { data.assume_init() }.f_type as u64;
    if !matches!(kind, 0xef53 | 0x9123683e) {
        return Err(E::UnsafeSource);
    }
    Ok(())
}
#[cfg(windows)]
fn check_volume(_: &File, path: &Path) -> Result<(), E> {
    check_filesystem(path).map_err(|_| E::UnsafeSource)
}
#[cfg(all(unix, not(target_os = "linux")))]
fn check_volume(_: &File, _: &Path) -> Result<(), E> {
    Err(E::UnsafeSource)
}
