//! Evidence-aware preparation port; the legacy bool provider remains strict.
use crate::{
    DatasetPreparationPolicy, FileSystemReferenceDataProvider, FilesystemReferenceError,
    PreparedReferenceEvidence, ReferenceDependency,
};
use std::{error::Error, fmt, path::PathBuf};

#[derive(Debug)]
pub enum ReferencePreparationError {
    NotConfigured,
    InconsistentEvidence,
    Provider(Box<dyn Error + Send + Sync>),
}
impl fmt::Display for ReferencePreparationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotConfigured => "required reference preparation is not configured",
            Self::InconsistentEvidence => {
                "prepared reference evidence differs from the requested bindings"
            }
            Self::Provider(_) => "reference preparation failed",
        })
    }
}
impl Error for ReferencePreparationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Provider(source) => Some(source.as_ref()),
            _ => None,
        }
    }
}
pub trait ReferenceDataPreparer: Send + Sync {
    fn prepare(
        &self,
        dependencies: &[ReferenceDependency],
        policy: DatasetPreparationPolicy,
    ) -> Result<PreparedReferenceEvidence, ReferencePreparationError>;
}
/// Binds the native root once. Dataset bindings are supplied by each compiled run.
pub struct FileSystemReferenceDataPreparer {
    root: PathBuf,
}
impl FileSystemReferenceDataPreparer {
    pub fn new(root: PathBuf) -> Result<Self, FilesystemReferenceError> {
        if root.as_os_str().is_empty() {
            return Err(FilesystemReferenceError::EmptyRoot);
        }
        let root = if root.is_absolute() {
            root
        } else {
            std::env::current_dir()
                .map_err(FilesystemReferenceError::RootIo)?
                .join(root)
        };
        Ok(Self { root })
    }
}
impl ReferenceDataPreparer for FileSystemReferenceDataPreparer {
    fn prepare(
        &self,
        dependencies: &[ReferenceDependency],
        policy: DatasetPreparationPolicy,
    ) -> Result<PreparedReferenceEvidence, ReferencePreparationError> {
        let prepare = || {
            FileSystemReferenceDataProvider::new(self.root.clone(), dependencies.to_vec())?
                .prepare(policy)
        };
        prepare().map_err(|source| ReferencePreparationError::Provider(Box::new(source)))
    }
}
