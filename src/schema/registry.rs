//! Exact-key loading and compilation for each resolution, without a cache.
use super::{
    SchemaCompilationError, SchemaCompiler,
    compiled::CompiledFlowSchema,
    raw::RawResolvedEncoding,
    repository::{SchemaLoadError, SchemaRepository},
};
use crate::domain::{FlowId, SchemaKey, SchemaVersion};
use std::{error::Error, fmt, sync::Arc};

/// Module-local technical failures; application maps these payloads to its run error.
#[derive(Debug)]
#[non_exhaustive]
pub enum SchemaResolutionError {
    Load(SchemaLoadError),
    Compilation(SchemaCompilationError),
}
impl fmt::Display for SchemaResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(error) => write!(f, "schema loading failed: {error}"),
            Self::Compilation(error) => write!(f, "schema compilation failed: {error}"),
        }
    }
}
impl Error for SchemaResolutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(match self {
            Self::Load(error) => error,
            Self::Compilation(error) => error,
        })
    }
}
impl From<SchemaLoadError> for SchemaResolutionError {
    fn from(error: SchemaLoadError) -> Self {
        Self::Load(error)
    }
}
impl From<SchemaCompilationError> for SchemaResolutionError {
    fn from(error: SchemaCompilationError) -> Self {
        Self::Compilation(error)
    }
}

/// Resolves and compiles afresh, with an explicit encoding selected by the caller.
/// Format support does not bypass the compiler's AST capability checks.
pub struct SchemaRegistry {
    repository: Arc<dyn SchemaRepository>,
    compiler: SchemaCompiler,
    encoding: RawResolvedEncoding,
}
impl SchemaRegistry {
    /// Encoding is a runtime binding, never read from locale or inferred from data.
    /// This caller-supplied profile is authoritative for each resolution; repository
    /// values are raw configuration and do not select the run's encoding.
    pub fn new(
        repository: Arc<dyn SchemaRepository>,
        compiler: SchemaCompiler,
        encoding: RawResolvedEncoding,
    ) -> Self {
        Self {
            repository,
            compiler,
            encoding,
        }
    }

    pub fn resolve(
        &self,
        key: &SchemaKey,
    ) -> Result<Arc<CompiledFlowSchema>, SchemaResolutionError> {
        let mut raw = self.repository.load_raw(key)?;
        // Also enforce selection integrity for custom repository implementations.
        let found = SchemaKey {
            flow_id: FlowId(raw.flow_id.clone()),
            version: SchemaVersion(raw.schema_version.clone()),
        };
        if found != *key {
            return Err(SchemaLoadError::KeyMismatch {
                requested: key.clone(),
                found,
            }
            .into());
        }
        raw.encoding.resolved = Some(self.encoding.clone());
        Ok(Arc::new(self.compiler.compile(raw)?))
    }
}
