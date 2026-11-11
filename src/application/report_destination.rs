//! D04: classify only the selected report destinations, then bind consent to
//! that exact set. Locality describes the OS filesystem, not who can read its
//! files or whether another application synchronizes them. Unknown is never
//! silently treated as local. This module neither writes nor prohibits shares.
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportDestinationLocation {
    Local,
    Remote,
    Undetermined,
}

/// Receipt contains only counts and the operator's choice, never path data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportDestinationReceipt {
    pub local_files: usize,
    pub remote_files: usize,
    pub undetermined_files: usize,
    pub warning_acknowledged: bool,
}

#[derive(Clone, PartialEq, Eq)]
struct SelectedDestination {
    resolved: PathBuf,
    location: ReportDestinationLocation,
}

/// The application keeps this Rust object while presenting the warning. It
/// must also retain its selected report/run; this object grants no access to
/// input data, archive approval or validation services.
#[derive(Clone)]
pub struct ReportDestinationAssessment {
    destinations: Vec<SelectedDestination>,
}
impl fmt::Debug for ReportDestinationAssessment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReportDestinationAssessment")
            .field("receipt", &self.receipt(false))
            .finish()
    }
}

#[derive(Debug)]
pub enum ReportDestinationError {
    Io(io::Error),
    ConfirmationRequired(ReportDestinationAssessment),
    SelectionChanged,
}
impl fmt::Display for ReportDestinationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Io(_) => "cannot inspect report destination",
            Self::ConfirmationRequired(_) => {
                "remote or undetermined report destination requires explicit operator confirmation"
            }
            Self::SelectionChanged => {
                "report destination changed; inspect it and confirm again before exporting"
            }
        })
    }
}
impl std::error::Error for ReportDestinationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}
impl From<io::Error> for ReportDestinationError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Only an inspection can construct this authorization. It must be rechecked
/// before creating staged export files and again before publishing them. This
/// is an ordinary path-change check, not a hostile-concurrency/TOCTOU guarantee.
pub struct AuthorizedReportDestinations {
    assessment: ReportDestinationAssessment,
    receipt: ReportDestinationReceipt,
}
impl fmt::Debug for AuthorizedReportDestinations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthorizedReportDestinations")
            .field("receipt", &self.receipt)
            .finish()
    }
}

impl ReportDestinationAssessment {
    /// Resolve existing parents/targets without creating files or repairing
    /// ACLs. Relative CLI destinations retain their existing supported syntax.
    pub fn inspect(paths: &[PathBuf]) -> Result<Self, ReportDestinationError> {
        Self::inspect_with(paths, classify_parent)
    }
    fn inspect_with(
        paths: &[PathBuf],
        classify: impl Fn(&Path) -> ReportDestinationLocation,
    ) -> Result<Self, ReportDestinationError> {
        let destinations = paths
            .iter()
            .map(|path| {
                let resolved = resolved_destination(path)?;
                let parent = resolved.parent().ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "report destination needs a filename",
                    )
                })?;
                let location = classify(parent);
                // Keep an explicitly selected UNC/mapped endpoint remote even
                // if Windows canonicalizes it to another spelling.
                #[cfg(windows)]
                let location = {
                    let absolute = if path.is_absolute() {
                        path.clone()
                    } else {
                        std::env::current_dir()?.join(path)
                    };
                    if absolute.parent().map(classify_parent)
                        == Some(ReportDestinationLocation::Remote)
                    {
                        ReportDestinationLocation::Remote
                    } else {
                        location
                    }
                };
                Ok(SelectedDestination { location, resolved })
            })
            .collect::<io::Result<Vec<_>>>()?;
        Ok(Self { destinations })
    }
    pub fn requires_confirmation(&self) -> bool {
        self.destinations
            .iter()
            .any(|d| d.location != ReportDestinationLocation::Local)
    }
    fn receipt(&self, acknowledged: bool) -> ReportDestinationReceipt {
        let count = |kind| {
            self.destinations
                .iter()
                .filter(|d| d.location == kind)
                .count()
        };
        ReportDestinationReceipt {
            local_files: count(ReportDestinationLocation::Local),
            remote_files: count(ReportDestinationLocation::Remote),
            undetermined_files: count(ReportDestinationLocation::Undetermined),
            warning_acknowledged: acknowledged,
        }
    }
    pub fn counts(&self) -> ReportDestinationReceipt {
        self.receipt(false)
    }
    /// There is no new privacy confirmation for a recognized local filesystem.
    pub fn allow_local(self) -> Result<AuthorizedReportDestinations, ReportDestinationError> {
        if self.requires_confirmation() {
            return Err(ReportDestinationError::ConfirmationRequired(self));
        }
        Ok(self.authorized(false))
    }
    /// Call only after this inspection's warning was accepted by the operator,
    /// or an explicit CLI choice for this command. No persistent global bypass.
    pub fn acknowledge_warning(self) -> AuthorizedReportDestinations {
        let acknowledged = self.requires_confirmation();
        self.authorized(acknowledged)
    }
    fn authorized(self, warning_acknowledged: bool) -> AuthorizedReportDestinations {
        let receipt = self.receipt(warning_acknowledged);
        AuthorizedReportDestinations {
            assessment: self,
            receipt,
        }
    }
}
impl AuthorizedReportDestinations {
    pub fn receipt(&self) -> ReportDestinationReceipt {
        self.receipt
    }
    pub fn verify_unchanged(&self, paths: &[PathBuf]) -> Result<(), ReportDestinationError> {
        self.verify_with(paths, classify_parent)
    }
    fn verify_with(
        &self,
        paths: &[PathBuf],
        classify: impl Fn(&Path) -> ReportDestinationLocation,
    ) -> Result<(), ReportDestinationError> {
        let current = ReportDestinationAssessment::inspect_with(paths, classify)?;
        if current.destinations != self.assessment.destinations {
            return Err(ReportDestinationError::SelectionChanged);
        }
        Ok(())
    }
}

fn resolved_destination(path: &Path) -> io::Result<PathBuf> {
    let name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "report destination needs a filename",
        )
    })?;
    match fs::canonicalize(path) {
        Ok(resolved) => Ok(resolved),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            Ok(fs::canonicalize(parent)?.join(name))
        }
        Err(error) => Err(error),
    }
}

#[cfg(target_os = "linux")]
fn classify_parent(parent: &Path) -> ReportDestinationLocation {
    use std::os::unix::ffi::OsStrExt;
    let Ok(path) = std::ffi::CString::new(parent.as_os_str().as_bytes()) else {
        return ReportDestinationLocation::Undetermined;
    };
    let mut info = std::mem::MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: terminated path and correctly sized writable statfs storage.
    if unsafe { libc::statfs(path.as_ptr(), info.as_mut_ptr()) } != 0 {
        return ReportDestinationLocation::Undetermined;
    }
    // SAFETY: statfs returned success and initialized the complete structure.
    linux_filesystem_location(unsafe { info.assume_init() }.f_type as u64)
}
#[cfg(target_os = "linux")]
fn linux_filesystem_location(kind: u64) -> ReportDestinationLocation {
    use ReportDestinationLocation::*;
    // Linux UAPI linux/magic.h: classification, never a filesystem denylist.
    // FUSE/overlay/unknown backing storage stays undetermined and is allowed
    // after warning, rather than guessed local or denied outright.
    match kind {
        0x6969 | 0xff53_4d42 | 0xfe53_4d42 | 0x517b | 0x5346_414f | 0x00c3_6400 | 0x7375_7245
        | 0x564c | 0x0102_1997 => Remote,
        0xef53 | 0x9123_683e | 0x5846_5342 | 0xf2f5_2010 | 0x0102_1994 | 0x8584_58f6 | 0x4d44
        | 0x2011_bab0 | 0xca45_1a4e => Local,
        _ => Undetermined,
    }
}

#[cfg(windows)]
fn windows_unc_path(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(component,
        std::path::Component::Prefix(prefix) if matches!(prefix.kind(),
            std::path::Prefix::UNC(..) | std::path::Prefix::VerbatimUNC(..)))
    })
}
#[cfg(windows)]
fn classify_parent(parent: &Path) -> ReportDestinationLocation {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::{
        Storage::FileSystem::{GetDriveTypeW, GetVolumePathNameW},
        System::WindowsProgramming as wp,
    };
    if windows_unc_path(parent) {
        return ReportDestinationLocation::Remote;
    }
    let mut path: Vec<_> = parent.as_os_str().encode_wide().collect();
    if path.contains(&0) {
        return ReportDestinationLocation::Undetermined;
    }
    path.push(0);
    let mut volume = vec![0u16; 32768];
    // SAFETY: terminated path and sized writable UTF16 volume buffer. This
    // resolves mounted volumes too; a drive letter is not proof of locality.
    if unsafe { GetVolumePathNameW(path.as_ptr(), volume.as_mut_ptr(), volume.len() as u32) } == 0 {
        return ReportDestinationLocation::Undetermined;
    }
    // SAFETY: the successful preceding API produced a terminated volume path.
    match unsafe { GetDriveTypeW(volume.as_ptr()) } {
        wp::DRIVE_REMOTE => ReportDestinationLocation::Remote,
        wp::DRIVE_FIXED | wp::DRIVE_REMOVABLE | wp::DRIVE_RAMDISK | wp::DRIVE_CDROM => {
            ReportDestinationLocation::Local
        }
        _ => ReportDestinationLocation::Undetermined,
    }
}
#[cfg(not(any(target_os = "linux", windows)))]
fn classify_parent(_: &Path) -> ReportDestinationLocation {
    ReportDestinationLocation::Undetermined
}

#[cfg(test)]
mod tests {
    use super::*;
    use ReportDestinationLocation::*;

    #[cfg(windows)]
    #[test]
    fn ordinary_and_extended_unc_are_recognized_without_contacting_a_server() {
        for path in [
            r"\\synthetic-server\share\folder",
            r"\\?\UNC\synthetic-server\share\folder",
        ] {
            assert!(windows_unc_path(Path::new(path)));
            assert_eq!(classify_parent(Path::new(path)), Remote);
        }
        assert!(!windows_unc_path(Path::new(r"C:\synthetic\folder")));
    }

    #[test]
    fn native_local_volume_needs_no_privacy_confirmation_and_creates_no_file() {
        let root = tempfile::tempdir().unwrap();
        let paths = vec![
            root.path().join("synthetic.jsonl"),
            root.path().join("summary.json"),
        ];
        let assessment = ReportDestinationAssessment::inspect(&paths).unwrap();
        assert!(!assessment.requires_confirmation());
        let permission = assessment.allow_local().unwrap();
        permission.verify_unchanged(&paths).unwrap();
        assert_eq!(
            permission.receipt(),
            ReportDestinationReceipt {
                local_files: 2,
                remote_files: 0,
                undetermined_files: 0,
                warning_acknowledged: false,
            }
        );
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }

    #[test]
    fn remote_and_undetermined_need_confirmation_then_are_allowed_without_denylist() {
        let root = tempfile::tempdir().unwrap();
        let paths = vec![root.path().join("synthetic.csv")];
        for kind in [Remote, Undetermined] {
            let assessment = ReportDestinationAssessment::inspect_with(&paths, |_| kind).unwrap();
            assert!(assessment.requires_confirmation());
            let error = assessment.allow_local().unwrap_err();
            let ReportDestinationError::ConfirmationRequired(inspection) = error else {
                panic!("expected explicit confirmation")
            };
            let permission = inspection.acknowledge_warning();
            permission.verify_with(&paths, |_| kind).unwrap();
            assert!(permission.receipt().warning_acknowledged);
            assert_eq!(
                permission.receipt().remote_files,
                usize::from(kind == Remote)
            );
            assert_eq!(
                permission.receipt().undetermined_files,
                usize::from(kind == Undetermined)
            );
            assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
        }
    }

    #[test]
    fn confirmation_cannot_be_retargeted_to_another_file_format_or_location_kind() {
        let root = tempfile::tempdir().unwrap();
        let paths = vec![root.path().join("synthetic.jsonl")];
        let permission = ReportDestinationAssessment::inspect_with(&paths, |_| Remote)
            .unwrap()
            .acknowledge_warning();
        for changed in [
            root.path().join("other.jsonl"),
            root.path().join("synthetic.xlsx"),
        ] {
            assert!(matches!(
                permission.verify_with(&[changed], |_| Remote),
                Err(ReportDestinationError::SelectionChanged)
            ));
        }
        assert!(matches!(
            permission.verify_with(&paths, |_| Undetermined),
            Err(ReportDestinationError::SelectionChanged)
        ));
        permission.verify_with(&paths, |_| Remote).unwrap();
    }

    #[test]
    fn mixed_destinations_keep_counts_and_binding_without_exporting_path_data() {
        let root = tempfile::tempdir().unwrap();
        let paths = ["local", "remote", "unknown"].map(|name| {
            let dir = root.path().join(name);
            fs::create_dir(&dir).unwrap();
            dir.join("SYNTHETIC_PATH_SENTINEL.jsonl")
        });
        let probe = |p: &Path| match p.file_name().and_then(|s| s.to_str()) {
            Some("local") => Local,
            Some("remote") => Remote,
            _ => Undetermined,
        };
        let assessment = ReportDestinationAssessment::inspect_with(&paths, probe).unwrap();
        assert_eq!(
            assessment.counts(),
            ReportDestinationReceipt {
                local_files: 1,
                remote_files: 1,
                undetermined_files: 1,
                warning_acknowledged: false,
            }
        );
        assert!(!format!("{assessment:?}").contains("SYNTHETIC_PATH_SENTINEL"));
        let permission = assessment.acknowledge_warning();
        assert!(!format!("{permission:?}").contains("SYNTHETIC_PATH_SENTINEL"));
        permission.verify_with(&paths, probe).unwrap();
        assert!(matches!(
            permission.verify_with(&paths[..2], probe),
            Err(ReportDestinationError::SelectionChanged)
        ));
    }

    #[test]
    fn missing_parent_is_io_failure_not_an_implicitly_allowed_unknown() {
        let root = tempfile::tempdir().unwrap();
        let paths = [root.path().join("missing/report.jsonl")];
        assert!(matches!(ReportDestinationAssessment::inspect(&paths),
            Err(ReportDestinationError::Io(e)) if e.kind() == io::ErrorKind::NotFound));
        assert!(!root.path().join("missing").exists());
    }

    #[cfg(unix)]
    #[test]
    fn canonical_parent_binding_detects_changed_symlink_without_mutating_destinations() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let first = root.path().join("first");
        let second = root.path().join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        let link = root.path().join("selected");
        symlink(&first, &link).unwrap();
        let paths = [link.join("synthetic.jsonl")];
        let permission = ReportDestinationAssessment::inspect_with(&paths, |_| Remote)
            .unwrap()
            .acknowledge_warning();
        fs::remove_file(&link).unwrap();
        symlink(&second, &link).unwrap();
        assert!(matches!(
            permission.verify_with(&paths, |_| Remote),
            Err(ReportDestinationError::SelectionChanged)
        ));
        assert_eq!(fs::read_dir(first).unwrap().count(), 0);
        assert_eq!(fs::read_dir(second).unwrap().count(), 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn kernel_filesystem_types_distinguish_network_from_opaque_backing_storage() {
        for kind in [
            0x6969,
            0xff53_4d42,
            0xfe53_4d42,
            0x517b,
            0x5346_414f,
            0x00c3_6400,
            0x7375_7245,
            0x564c,
            0x0102_1997,
        ] {
            assert_eq!(linux_filesystem_location(kind), Remote);
        }
        for kind in [0xef53, 0x9123_683e, 0x5846_5342, 0x0102_1994] {
            assert_eq!(linux_filesystem_location(kind), Local);
        }
        for kind in [0x6573_5546, 0x794c_7630, 0xdead_beef] {
            assert_eq!(linux_filesystem_location(kind), Undetermined);
        }
    }
}
