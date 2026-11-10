//! Operational soft budgets are independent of normative validation and hard
//! safety bounds. A prepared warning owns an exact request and file fingerprints;
//! accepting it cannot authorize a different request or replaced input.
use crate::{
    CancellationToken, InputSource, ValidationRequest, ValidationScope, ValidationService,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fmt,
    fs::File,
    io::{self, Read},
    num::NonZeroU64,
    path::{Path, PathBuf},
};

/// No product default is invented. Operators configure measured budgets, with
/// a versioned source describing the measurement that motivated their values.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceBudget {
    pub version: u32,
    pub measurement_source: String,
    pub input_bytes: Option<NonZeroU64>,
    pub reference_bytes: Option<NonZeroU64>,
}

/// The caller supplies the same selected bundle as the real run. Embedded data
/// is immutable; filesystem references are fingerprinted individually.
pub enum ResourceReferenceSource<'a> {
    Filesystem(&'a Path),
    #[cfg(feature = "embedded-bundle")]
    Embedded(&'a crate::EmbeddedBundle),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ResourceBudgetAudit {
    pub input_bytes: u64,
    pub reference_bytes: u64,
    pub missing_reference_files: u64,
    pub input_budget_exceeded: bool,
    pub reference_budget_exceeded: bool,
    pub proceeded_over_budget: bool,
}
impl ResourceBudgetAudit {
    pub fn requires_confirmation(&self) -> bool {
        self.input_budget_exceeded || self.reference_budget_exceeded
    }
}

#[derive(Debug)]
pub enum ResourcePreflightError {
    InvalidConfiguration,
    Schema(crate::ValidationRunError),
    Io(io::Error),
    NotRegularFile,
    ReferenceBoundary,
    SizeOverflow,
    ChangedRequest,
    ChangedFile,
    Cancelled,
    ConfirmationRequired,
}
impl fmt::Display for ResourcePreflightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidConfiguration => {
                "resource budget requires version 1 and an explicit measurement source"
            }
            Self::Schema(_) => "resource preflight schema resolution failed",
            Self::Io(_) => "resource preflight file operation failed",
            Self::NotRegularFile => "resource preflight requires regular files",
            Self::ReferenceBoundary => {
                "resource preflight reference path violates the selected bundle boundary"
            }
            Self::SizeOverflow => "resource byte count exceeds the supported integer range",
            Self::ChangedRequest => "resource confirmation belongs to a different request",
            Self::ChangedFile => {
                "resource preflight files changed; repeat preflight and confirmation"
            }
            Self::Cancelled => "resource preflight cancelled",
            Self::ConfirmationRequired => {
                "soft resource budget exceeded; explicitly choose Proceed anyway or cancel"
            }
        })
    }
}
impl Error for ResourcePreflightError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Schema(e) => Some(e),
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}
impl From<io::Error> for ResourcePreflightError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Fingerprint {
    bytes: u64,
    signature: Vec<u64>,
    sha256: [u8; 32],
}
#[derive(Debug)]
struct CapturedFile {
    path: PathBuf,
    // A reference missing at preflight must still be missing at confirmation.
    // This never means a verified empty dataset or a valid normative result.
    fingerprint: Option<Fingerprint>,
    reference: bool,
    reference_root: Option<PathBuf>,
}

/// Private fields prevent a renderer from forging totals or acceptance. Keep
/// this object in Rust and expose only an opaque handle in a future IPC adapter.
#[derive(Debug)]
pub struct PreparedResourcePreflight {
    request: ValidationRequest,
    schema_hash: crate::Sha256Digest,
    working_directory: PathBuf,
    reference_roots: Vec<(PathBuf, PathBuf)>,
    files: Vec<CapturedFile>,
    audit: ResourceBudgetAudit,
}
#[derive(Debug)]
pub struct AcceptedResourcePreflight(PreparedResourcePreflight);

impl PreparedResourcePreflight {
    pub fn audit(&self) -> ResourceBudgetAudit {
        self.audit
    }
    /// Consumes this exact prepared warning. The decision cannot be reused with
    /// another preflight; verification still runs even after explicit override.
    pub fn accept(
        mut self,
        proceed_anyway: bool,
    ) -> Result<AcceptedResourcePreflight, ResourcePreflightError> {
        if self.audit.requires_confirmation() && !proceed_anyway {
            return Err(ResourcePreflightError::ConfirmationRequired);
        }
        self.audit.proceeded_over_budget = self.audit.requires_confirmation() && proceed_anyway;
        Ok(AcceptedResourcePreflight(self))
    }
}
impl AcceptedResourcePreflight {
    pub fn audit(&self) -> ResourceBudgetAudit {
        self.0.audit
    }
    /// Recheck immediately before the adapter calls validation. The ordinary
    /// validator still reads to EOF and computes its own complete input audit.
    /// This check is not hostile handle-relative filesystem confinement: the
    /// selected local tree must remain controlled during the run (D10).
    pub fn verify_unchanged(
        &self,
        service: &ValidationService,
        request: &ValidationRequest,
        cancellation: &CancellationToken,
    ) -> Result<(), ResourcePreflightError> {
        if cancellation.is_cancelled() {
            return Err(ResourcePreflightError::Cancelled);
        }
        if request != &self.0.request || std::env::current_dir()? != self.0.working_directory {
            return Err(ResourcePreflightError::ChangedRequest);
        }
        let resolved = service
            .schema_registry
            .resolve_with_hash(&request.schema_key)
            .map_err(|e| ResourcePreflightError::Schema(e.into()))?;
        if resolved.schema_hash() != &self.0.schema_hash {
            return Err(ResourcePreflightError::ChangedRequest);
        }
        for (named, resolved) in &self.0.reference_roots {
            if std::fs::canonicalize(named)? != *resolved {
                return Err(ResourcePreflightError::ChangedFile);
            }
        }
        for file in &self.0.files {
            if let Some(root) = &file.reference_root {
                let relative = file
                    .path
                    .strip_prefix(root)
                    .map_err(|_| ResourcePreflightError::ReferenceBoundary)?;
                let relative = relative
                    .to_str()
                    .ok_or(ResourcePreflightError::ReferenceBoundary)?
                    .replace('\\', "/");
                checked_reference(root, &relative)?;
            }
            if fingerprint(&file.path, file.reference, cancellation)? != file.fingerprint {
                return Err(ResourcePreflightError::ChangedFile);
            }
        }
        Ok(())
    }
}

impl ValidationService {
    /// Resolve the real compiled schema to select reference bindings; never
    /// scan unrelated directories or infer references from names in the UI.
    pub fn resource_preflight(
        &self,
        request: ValidationRequest,
        references: ResourceReferenceSource<'_>,
        budget: &ResourceBudget,
        cancellation: &CancellationToken,
    ) -> Result<PreparedResourcePreflight, ResourcePreflightError> {
        if budget.version != 1 || budget.measurement_source.trim().is_empty() {
            return Err(ResourcePreflightError::InvalidConfiguration);
        }
        if cancellation.is_cancelled() {
            return Err(ResourcePreflightError::Cancelled);
        }
        let resolved = self
            .schema_registry
            .resolve_with_hash(&request.schema_key)
            .map_err(|e| ResourcePreflightError::Schema(e.into()))?;
        crate::reference_data::filesystem::validate_bindings(
            resolved.schema().required_reference_data(),
        )
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let inputs = match &request.scope {
            ValidationScope::Trace(input) => std::slice::from_ref(input),
            ValidationScope::Flow(flow) => flow.traces.as_slice(),
        };
        let mut files = Vec::new();
        let mut reference_roots = Vec::new();
        let mut audit = ResourceBudgetAudit {
            input_bytes: 0,
            reference_bytes: 0,
            missing_reference_files: 0,
            input_budget_exceeded: false,
            reference_budget_exceeded: false,
            proceeded_over_budget: false,
        };
        for input in inputs {
            let InputSource::Path(path) = &input.source;
            let path = absolute(path)?;
            let value = fingerprint(&path, false, cancellation)?;
            add_bytes(
                &mut audit.input_bytes,
                value.as_ref().expect("input cannot be missing").bytes,
            )?;
            files.push(CapturedFile {
                path,
                fingerprint: value,
                reference: false,
                reference_root: None,
            });
        }
        for binding in resolved.schema().required_reference_data() {
            match &references {
                ResourceReferenceSource::Filesystem(root) => {
                    let named_root = absolute(root)?;
                    let root = std::fs::canonicalize(root)?;
                    if !reference_roots.iter().any(|(path, _)| path == &named_root) {
                        reference_roots.push((named_root, root.clone()));
                    }
                    checked_reference(&root, &binding.bundle_path)?;
                    let path = root.join(binding.bundle_path.as_ref());
                    let value = fingerprint(&path, true, cancellation)?;
                    if let Some(value) = &value {
                        add_bytes(&mut audit.reference_bytes, value.bytes)?;
                    } else {
                        add_bytes(&mut audit.missing_reference_files, 1)?;
                    }
                    files.push(CapturedFile {
                        path,
                        fingerprint: value,
                        reference: true,
                        reference_root: Some(root),
                    });
                }
                #[cfg(feature = "embedded-bundle")]
                ResourceReferenceSource::Embedded(bundle) => {
                    if let Some(bytes) =
                        bundle.file(&format!("reference-data/{}", binding.bundle_path))
                    {
                        add_bytes(
                            &mut audit.reference_bytes,
                            u64::try_from(bytes.len())
                                .map_err(|_| ResourcePreflightError::SizeOverflow)?,
                        )?;
                    } else {
                        add_bytes(&mut audit.missing_reference_files, 1)?;
                    }
                }
            }
        }
        audit.input_budget_exceeded = budget
            .input_bytes
            .is_some_and(|n| audit.input_bytes > n.get());
        audit.reference_budget_exceeded = budget
            .reference_bytes
            .is_some_and(|n| audit.reference_bytes > n.get());
        Ok(PreparedResourcePreflight {
            request,
            schema_hash: resolved.schema_hash().clone(),
            working_directory: std::env::current_dir()?,
            reference_roots,
            files,
            audit,
        })
    }
}

fn absolute(path: &Path) -> io::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_owned())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}
fn checked_reference(root: &Path, relative: &str) -> Result<(), ResourcePreflightError> {
    use crate::reference_data::filesystem::{PathFailure, checked_path};
    match checked_path(root, relative) {
        Ok(_) => Ok(()),
        Err(PathFailure::Io(_, e)) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(PathFailure::Io(_, e)) => Err(e.into()),
        Err(PathFailure::Boundary(_)) => Err(ResourcePreflightError::ReferenceBoundary),
        Err(PathFailure::FileType(_)) => Err(ResourcePreflightError::NotRegularFile),
        Err(PathFailure::Trust(violation)) => Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("reference tree permissions are not controlled: {violation:?}"),
        )
        .into()),
    }
}
fn add_bytes(total: &mut u64, bytes: u64) -> Result<(), ResourcePreflightError> {
    *total = total
        .checked_add(bytes)
        .ok_or(ResourcePreflightError::SizeOverflow)?;
    Ok(())
}
fn fingerprint(
    path: &Path,
    reference: bool,
    cancellation: &CancellationToken,
) -> Result<Option<Fingerprint>, ResourcePreflightError> {
    if cancellation.is_cancelled() {
        return Err(ResourcePreflightError::Cancelled);
    }
    // Reject named pipes/devices before open, which could otherwise block. The
    // opened handle is checked again below; the deployment tree is controlled.
    match std::fs::metadata(path) {
        Ok(metadata) if !metadata.is_file() => return Err(ResourcePreflightError::NotRegularFile),
        Err(e) if reference && e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
        _ => {}
    }
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) if reference && e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    if !file.metadata()?.is_file() {
        return Err(ResourcePreflightError::NotRegularFile);
    }
    let before = signature(&file)?;
    let mut bytes = 0;
    let mut digest = Sha256::new();
    // A bounded stack buffer avoids materializing any input/reference payload.
    let mut buffer = [0u8; 65536];
    loop {
        if cancellation.is_cancelled() {
            return Err(ResourcePreflightError::Cancelled);
        }
        let count = match file.read(&mut buffer) {
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            other => other?,
        };
        if count == 0 {
            break;
        }
        add_bytes(&mut bytes, count as u64)?;
        digest.update(&buffer[..count]);
    }
    if before != signature(&file)? || bytes != file.metadata()?.len() {
        return Err(ResourcePreflightError::ChangedFile);
    }
    // Reopening detects a replaced directory entry, even with identical bytes.
    if signature(&File::open(path)?)? != before {
        return Err(ResourcePreflightError::ChangedFile);
    }
    Ok(Some(Fingerprint {
        bytes,
        signature: before,
        sha256: digest.finalize().into(),
    }))
}

#[cfg(unix)]
fn signature(file: &File) -> io::Result<Vec<u64>> {
    use std::os::unix::fs::MetadataExt;
    let m = file.metadata()?;
    Ok(vec![
        m.dev(),
        m.ino(),
        m.len(),
        m.mtime() as u64,
        m.mtime_nsec() as u64,
        m.ctime() as u64,
        m.ctime_nsec() as u64,
    ])
}
#[cfg(windows)]
fn signature(file: &File) -> io::Result<Vec<u64>> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, GetFileInformationByHandle,
    };
    let mut info = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
    // SAFETY: live owned handle and correctly sized output; read only on success.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), info.as_mut_ptr()) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: the successful OS call initialized the whole output structure.
    let i = unsafe { info.assume_init() };
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hard_integer_overflow_cannot_be_overridden() {
        let mut total = u64::MAX;
        assert!(matches!(
            add_bytes(&mut total, 1),
            Err(ResourcePreflightError::SizeOverflow)
        ));
        assert_eq!(total, u64::MAX);
    }
}
