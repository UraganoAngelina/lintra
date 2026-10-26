//! Strict v1 filesystem loading plus explicit, immutable preparation evidence.
//!
//! Confinement checks reject unsafe paths/symlinks in a stable configuration.
//! They are NOT atomic handle-relative resolution against hostile concurrent
//! filesystem changes (TOCTOU). Deployers must control root/tree mutations.
use super::{
    dataset::{ByteLookupSet, DatasetKey, LookupSet},
    evidence::*,
    membership_format,
    provider::{ReferenceDataError, ReferenceDataProvider, validate_key},
};
use crate::{DatasetId, DatasetVersion, Sha256Digest, schema::compiled::ReferenceDependency};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceBindingIssue {
    InvalidIdentity,
    DuplicateIdentity,
    UnsafeRelativePath,
    UnsupportedMetadata,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemBoundaryViolation {
    Symlink,
    OutsideRoot,
}
#[derive(Debug)]
#[non_exhaustive]
pub enum FilesystemReferenceError {
    EmptyRoot,
    RootIo(io::Error),
    InvalidBinding {
        key: DatasetKey,
        issue: ReferenceBindingIssue,
    },
    Boundary {
        key: DatasetKey,
        violation: FilesystemBoundaryViolation,
    },
    StrictPreparation {
        evidence: Box<DatasetEvidence>,
    },
    InvalidPreparedState,
}
impl fmt::Display for FilesystemReferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyRoot => "an explicit nonempty reference root is required",
            Self::RootIo(_) => "cannot resolve the reference root configuration",
            Self::InvalidBinding { .. } => "reference dependency binding is invalid or unsupported",
            Self::Boundary { .. } => "reference filesystem boundary check failed",
            Self::StrictPreparation { .. } => {
                "reference dataset does not meet strict preparation requirements"
            }
            Self::InvalidPreparedState => {
                "reference preparation produced inconsistent internal state"
            }
        })
    }
}
impl Error for FilesystemReferenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RootIo(source) => Some(source),
            Self::StrictPreparation { evidence } => evidence
                .anomalies()
                .iter()
                .find_map(DatasetAnomaly::source)
                .map(|source| source as &dyn Error),
            _ => None,
        }
    }
}

pub struct FileSystemReferenceDataProvider {
    root: PathBuf,
    dependencies: Vec<ReferenceDependency>,
    indices: HashMap<DatasetKey, usize>,
}
impl FileSystemReferenceDataProvider {
    /// Configuration validation performs no dataset reads. Relative native roots
    /// are bound to the current directory once, never reinterpreted at lookup.
    pub fn new(
        root: PathBuf,
        dependencies: Vec<ReferenceDependency>,
    ) -> Result<Self, FilesystemReferenceError> {
        if root.as_os_str().is_empty() {
            return Err(FilesystemReferenceError::EmptyRoot);
        }
        let indices = validate_bindings(&dependencies)?;
        let root = if root.is_absolute() {
            root
        } else {
            std::env::current_dir()
                .map_err(FilesystemReferenceError::RootIo)?
                .join(root)
        };
        Ok(Self {
            root,
            dependencies,
            indices,
        })
    }
    pub fn prepare(
        &self,
        policy: DatasetPreparationPolicy,
    ) -> Result<PreparedReferenceEvidence, FilesystemReferenceError> {
        self.prepare_selected(&self.dependencies, policy)
    }
    fn prepare_selected(
        &self,
        dependencies: &[ReferenceDependency],
        policy: DatasetPreparationPolicy,
    ) -> Result<PreparedReferenceEvidence, FilesystemReferenceError> {
        let mut prepared = PreparedReferenceEvidence {
            sets: HashMap::new(),
            evidence: Vec::new(),
            indices: HashMap::new(),
            policy,
        };
        if dependencies.is_empty() {
            return Ok(prepared);
        }
        let root = fs::canonicalize(&self.root).map_err(Arc::new);
        for dependency in dependencies {
            let (evidence, set) = load_one(&root, dependency, policy)?;
            append_prepared(&mut prepared, evidence, set)?;
        }
        Ok(prepared)
    }
}

// Shared configuration/evidence logic for real filesystem and embedded sources.
pub(super) fn validate_bindings(
    dependencies: &[ReferenceDependency],
) -> Result<HashMap<DatasetKey, usize>, FilesystemReferenceError> {
    let mut indices = HashMap::new();
    for (index, dependency) in dependencies.iter().enumerate() {
        let issue = if validate_key(&dependency.key).is_err() {
            Some(ReferenceBindingIssue::InvalidIdentity)
        } else if !dependency.metadata_attributes.is_empty() {
            Some(ReferenceBindingIssue::UnsupportedMetadata)
        } else if !portable_relative(&dependency.bundle_path) {
            Some(ReferenceBindingIssue::UnsafeRelativePath)
        } else if indices.insert(dependency.key.clone(), index).is_some() {
            Some(ReferenceBindingIssue::DuplicateIdentity)
        } else {
            None
        };
        if let Some(issue) = issue {
            return Err(FilesystemReferenceError::InvalidBinding {
                key: dependency.key.clone(),
                issue,
            });
        }
    }
    Ok(indices)
}
pub(super) fn append_prepared(
    prepared: &mut PreparedReferenceEvidence,
    evidence: DatasetEvidence,
    set: Option<Arc<dyn LookupSet>>,
) -> Result<(), FilesystemReferenceError> {
    if prepared.policy == DatasetPreparationPolicy::Strict
        && evidence.availability != DatasetAvailability::Verified
    {
        return Err(FilesystemReferenceError::StrictPreparation {
            evidence: Box::new(evidence),
        });
    }
    if let Some(set) = set {
        prepared.sets.insert(evidence.key.clone(), set);
    }
    prepared
        .indices
        .insert(evidence.key.clone(), prepared.evidence.len());
    prepared.evidence.push(evidence);
    Ok(())
}
impl ReferenceDataProvider for FileSystemReferenceDataProvider {
    /// Legacy API is deliberately strict: it cannot express degraded trust.
    fn load(&self, key: &DatasetKey) -> Result<Arc<dyn LookupSet>, ReferenceDataError> {
        validate_key(key)?;
        let Some(dependency) = self
            .indices
            .get(key)
            .and_then(|index| self.dependencies.get(*index))
        else {
            return Err(ReferenceDataError::Unavailable { key: key.clone() });
        };
        let load = || {
            let prepared = self.prepare_selected(
                std::slice::from_ref(dependency),
                DatasetPreparationPolicy::Strict,
            )?;
            prepared
                .sets
                .get(key)
                .cloned()
                .ok_or(FilesystemReferenceError::InvalidPreparedState)
        };
        load().map_err(|source| ReferenceDataError::Provider {
            key: key.clone(),
            source: Box::new(source),
        })
    }
}

pub(super) type Loaded = (DatasetEvidence, Option<Arc<dyn LookupSet>>);
pub(super) fn initial_evidence(
    dependency: &ReferenceDependency,
    policy: DatasetPreparationPolicy,
) -> DatasetEvidence {
    DatasetEvidence {
        key: dependency.key.clone(),
        expected_sha256: Sha256Digest(dependency.sha256),
        actual_sha256: None,
        declared_key: None,
        availability: DatasetAvailability::Unavailable,
        policy,
        anomalies: vec![],
    }
}
fn load_one(
    root: &Result<PathBuf, Arc<io::Error>>,
    dependency: &ReferenceDependency,
    policy: DatasetPreparationPolicy,
) -> Result<Loaded, FilesystemReferenceError> {
    let mut evidence = initial_evidence(dependency, policy);
    let root = match root {
        Ok(root) => root,
        Err(source) => {
            evidence.anomalies.push(DatasetAnomaly::Io {
                operation: DatasetIoOperation::ResolveRoot,
                source: source.clone(),
            });
            return Ok((evidence, None));
        }
    };
    let path = match checked_path(root, &dependency.bundle_path) {
        Ok(path) => path,
        Err(PathFailure::Io(operation, source)) => {
            evidence.anomalies.push(DatasetAnomaly::Io {
                operation,
                source: Arc::new(source),
            });
            return Ok((evidence, None));
        }
        Err(PathFailure::Boundary(violation)) => {
            return Err(FilesystemReferenceError::Boundary {
                key: dependency.key.clone(),
                violation,
            });
        }
        Err(PathFailure::FileType(reason)) => {
            evidence.anomalies.push(DatasetAnomaly::FileType(reason));
            return Ok((evidence, None));
        }
    };
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(source) => {
            evidence.anomalies.push(DatasetAnomaly::Io {
                operation: DatasetIoOperation::ReadFile,
                source: Arc::new(source),
            });
            return Ok((evidence, None));
        }
    };
    decode_bytes(evidence, dependency, &bytes)
}
pub(super) fn decode_bytes(
    mut evidence: DatasetEvidence,
    dependency: &ReferenceDependency,
    bytes: &[u8],
) -> Result<Loaded, FilesystemReferenceError> {
    let actual: [u8; 32] = Sha256::digest(bytes).into();
    evidence.actual_sha256 = Some(Sha256Digest(actual));
    if actual != dependency.sha256 {
        evidence.anomalies.push(DatasetAnomaly::IntegrityMismatch);
    }
    let format_issue = |issue| DatasetAnomaly::Format {
        issue,
        source: None,
    };
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        evidence
            .anomalies
            .push(format_issue(MembershipFormatIssue::Bom));
        return Ok((evidence, None));
    }
    let document = match membership_format::parse(bytes) {
        Ok(document) => document,
        Err(failure) => {
            evidence.anomalies.push(DatasetAnomaly::Format {
                issue: failure.issue,
                source: Some(failure.source),
            });
            return Ok((evidence, None));
        }
    };
    let declared = DatasetKey {
        id: DatasetId(document.id),
        version: DatasetVersion(document.version),
    };
    evidence.declared_key = Some(declared.clone());
    let mut invalid_header = false;
    if document.format != 1 {
        evidence
            .anomalies
            .push(format_issue(MembershipFormatIssue::UnsupportedVersion(
                document.format,
            )));
        invalid_header = true;
    }
    if validate_key(&declared).is_err() {
        evidence.anomalies.push(DatasetAnomaly::InvalidIdentity);
        invalid_header = true;
    } else if declared != dependency.key {
        evidence.anomalies.push(DatasetAnomaly::IdentityMismatch);
        invalid_header = true;
    }
    if invalid_header {
        return Ok((evidence, None));
    }
    let values = match membership_format::decode_keys(document.keys) {
        Ok(values) => values,
        Err(issue) => {
            evidence.anomalies.push(format_issue(issue));
            return Ok((evidence, None));
        }
    };
    evidence.availability = if evidence.anomalies.is_empty() {
        DatasetAvailability::Verified
    } else {
        DatasetAvailability::Degraded
    };
    Ok((
        evidence,
        Some(Arc::new(ByteLookupSet::new(dependency.key.clone(), values))),
    ))
}

enum PathFailure {
    Io(DatasetIoOperation, io::Error),
    Boundary(FilesystemBoundaryViolation),
    FileType(DatasetFileTypeIssue),
}
fn checked_path(root: &Path, relative: &str) -> Result<PathBuf, PathFailure> {
    let mut path = root.to_path_buf();
    let mut parts = relative.split('/').peekable();
    while let Some(part) = parts.next() {
        path.push(part);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|source| PathFailure::Io(DatasetIoOperation::InspectPath, source))?;
        if metadata.file_type().is_symlink() {
            return Err(PathFailure::Boundary(FilesystemBoundaryViolation::Symlink));
        }
        if parts.peek().is_some() && !metadata.is_dir() {
            return Err(PathFailure::FileType(
                DatasetFileTypeIssue::NonDirectoryComponent,
            ));
        }
        if parts.peek().is_none() && !metadata.is_file() {
            return Err(PathFailure::FileType(DatasetFileTypeIssue::NonRegularFile));
        }
    }
    let resolved = fs::canonicalize(path)
        .map_err(|source| PathFailure::Io(DatasetIoOperation::ResolvePath, source))?;
    if !resolved.starts_with(root) {
        return Err(PathFailure::Boundary(
            FilesystemBoundaryViolation::OutsideRoot,
        ));
    }
    Ok(resolved)
}
pub(crate) fn portable_relative(path: &str) -> bool {
    path.split('/').all(|part| {
        if part.is_empty()
            || matches!(part, "." | "..")
            || part.ends_with(['.', ' '])
            || part.chars().any(|c| {
                c.is_control() || matches!(c, '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*')
            })
        {
            return false;
        }
        let base = part
            .split('.')
            .next()
            .unwrap_or(part)
            .trim_end_matches(' ')
            .to_ascii_uppercase();
        if matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL") {
            return false;
        }
        !base
            .strip_prefix("COM")
            .or_else(|| base.strip_prefix("LPT"))
            .is_some_and(|number| {
                matches!(
                    number,
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                )
            })
    })
}
