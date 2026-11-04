//! Explicit file adapter. Only callers configure a source: no discovery,
//! generation, rotation, SQLite access or regulatory policy belongs here.
use super::{PersistenceKey, PersistenceKeyProvider, ProtectionError, file_security};
use std::{
    error::Error,
    fmt,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

/// Redacted failures deliberately retain neither OS error paths nor key bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FileKeyError {
    InvalidKeyId,
    InvalidSource,
    Unavailable,
    UnsafeSource,
    ChangedSource,
    InvalidLength,
}
impl fmt::Display for FileKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidKeyId => "invalid persistence key identifier",
            Self::InvalidSource => "invalid explicit persistence key source",
            Self::Unavailable => "persistence key file is unavailable",
            Self::UnsafeSource => "persistence key file access boundary is unsafe",
            Self::ChangedSource => "persistence key file changed during loading",
            Self::InvalidLength => "persistence key file must contain exactly 32 binary bytes",
        })
    }
}
impl Error for FileKeyError {}

/// Pure adapter convention for the private portable Slint bundle. The caller
/// passes the ACTUAL executable path, normally `std::env::current_exe()`, and
/// configures the non-secret Key ID separately. No cwd/home fallback or I/O.
pub fn portable_persistence_key_path(executable: &Path) -> Result<PathBuf, FileKeyError> {
    if !executable.is_absolute()
        || executable.file_name().is_none()
        || executable
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return Err(FileKeyError::InvalidSource);
    }
    Ok(executable
        .parent()
        .ok_or(FileKeyError::InvalidSource)?
        .join("hmac.key"))
}

/// An explicitly loaded immutable key. The file and directory handles close
/// before construction completes; only the existing zeroizing key stays alive.
/// Moving a bundle or changing the cwd cannot change a loaded provider's key.
pub struct FilePersistenceKeyProvider {
    key_id: String,
    key: Arc<PersistenceKey>,
}
impl fmt::Debug for FilePersistenceKeyProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FilePersistenceKeyProvider { [redacted] }")
    }
}
impl FilePersistenceKeyProvider {
    /// Relative paths are resolved ONCE as an explicitly supplied filename;
    /// this never searches alternative directories. Symlink/reparse ancestry,
    /// unsafe access, hard links, replacement and non-32-byte files are rejected.
    pub fn load(key_id: impl Into<String>, path: &Path) -> Result<Self, FileKeyError> {
        let key_id = key_id.into();
        if key_id.is_empty() || key_id.trim() != key_id || key_id.chars().any(char::is_control) {
            return Err(FileKeyError::InvalidKeyId);
        }
        let mut bytes = file_security::key::read(path, || {})?;
        // Transfer ownership, never copy the buffer into an unguarded spare.
        // PersistenceKey::new immediately wraps the moved Vec in Zeroizing.
        let key = PersistenceKey::new(std::mem::take(&mut *bytes))
            .map_err(|_| FileKeyError::InvalidLength)?;
        Ok(Self {
            key_id,
            key: Arc::new(key),
        })
    }
}
impl PersistenceKeyProvider for FilePersistenceKeyProvider {
    fn resolve(&self, key_id: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        if key_id != self.key_id {
            return Err(ProtectionError::KeyUnavailable);
        }
        Ok(Arc::clone(&self.key))
    }
}

#[cfg(test)]
mod tests;
