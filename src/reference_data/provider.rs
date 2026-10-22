//! Loading is a preparation operation, distinct from normative membership failure.
use super::dataset::{DatasetKey, LookupSet};
use std::{error::Error, fmt, sync::Arc};

pub trait ReferenceDataProvider: Send + Sync {
    fn load(&self, key: &DatasetKey) -> Result<Arc<dyn LookupSet>, ReferenceDataError>;
}

/// Technical preparation errors only; a patient's unknown code is a normal
/// contains=false result, not one of these errors. No future adapter type is
/// required to construct an error or replaced by a generic string.
#[derive(Debug)]
#[non_exhaustive]
pub enum ReferenceDataError {
    InvalidKey {
        key: DatasetKey,
    },
    Unavailable {
        key: DatasetKey,
    },
    KeyMismatch {
        requested: DatasetKey,
        actual: DatasetKey,
    },
    Provider {
        key: DatasetKey,
        source: Box<dyn Error + Send + Sync>,
    },
}
impl fmt::Display for ReferenceDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidKey { .. } => "reference dataset identity is invalid",
            Self::Unavailable { .. } => "required reference dataset is unavailable",
            Self::KeyMismatch { .. } => {
                "loaded reference dataset identity differs from the requested key"
            }
            Self::Provider { .. } => "reference dataset preparation failed",
        })
    }
}
impl Error for ReferenceDataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Provider { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}
