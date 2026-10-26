//! DEC-18 acquisition catalog. Listing never compiles or certifies a schema.
use super::*;

pub trait SchemaCatalog: Send + Sync {
    fn list_schemas(&self) -> Result<Vec<SchemaCatalogEntry>, SchemaCatalogError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaCatalogEntry {
    pub key: SchemaKey,
    /// Declaration order, not lexical trace order.
    pub traces: Vec<crate::TraceId>,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum SchemaCatalogError {
    Io { path: PathBuf, source: io::Error },
    InvalidComponent { path: PathBuf },
    Symlink { path: PathBuf },
    Load(SchemaLoadError),
    InvalidTraces { key: SchemaKey },
}
impl fmt::Display for SchemaCatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, .. } => {
                write!(f, "cannot inspect schema catalog path {}", path.display())
            }
            Self::InvalidComponent { path } => {
                write!(f, "invalid schema catalog component {}", path.display())
            }
            Self::Symlink { path } => write!(
                f,
                "schema catalog symlink is not allowed: {}",
                path.display()
            ),
            Self::Load(error) => write!(f, "schema catalog acquisition failed: {error}"),
            Self::InvalidTraces { .. } => {
                f.write_str("schema catalog requires nonempty unique trace IDs")
            }
        }
    }
}
impl Error for SchemaCatalogError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Load(error) => Some(error),
            _ => None,
        }
    }
}
impl From<SchemaLoadError> for SchemaCatalogError {
    fn from(error: SchemaLoadError) -> Self {
        Self::Load(error)
    }
}

fn entry(raw: &RawFlowSchema) -> Result<SchemaCatalogEntry, SchemaCatalogError> {
    let key = schema_key(raw);
    let mut seen = HashSet::new();
    if raw
        .traces
        .iter()
        .any(|trace| trace.id.is_empty() || !seen.insert(&trace.id))
    {
        return Err(SchemaCatalogError::InvalidTraces { key });
    }
    Ok(SchemaCatalogEntry {
        key,
        traces: raw
            .traces
            .iter()
            .map(|trace| crate::TraceId(trace.id.clone()))
            .collect(),
    })
}

fn io_error(path: &Path, source: io::Error) -> SchemaCatalogError {
    SchemaCatalogError::Io {
        path: path.to_owned(),
        source,
    }
}

/// Sort names before metadata/identity checks; valid names use exact UTF-8 bytes.
/// Lossy names are used only for ordering invalid native names before rejection.
fn children(path: &Path) -> Result<Vec<PathBuf>, SchemaCatalogError> {
    let mut paths = fs::read_dir(path)
        .map_err(|source| io_error(path, source))?
        .map(|result| {
            result
                .map(|e| e.path())
                .map_err(|source| io_error(path, source))
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort_by(|a, b| {
        a.file_name()
            .unwrap()
            .to_string_lossy()
            .cmp(&b.file_name().unwrap().to_string_lossy())
            .then_with(|| a.cmp(b))
    });
    Ok(paths)
}
fn metadata(path: &Path) -> Result<fs::Metadata, SchemaCatalogError> {
    let m = fs::symlink_metadata(path).map_err(|source| io_error(path, source))?;
    if m.file_type().is_symlink() {
        return Err(SchemaCatalogError::Symlink {
            path: path.to_owned(),
        });
    }
    Ok(m)
}
fn component(path: &Path) -> Result<String, SchemaCatalogError> {
    path.file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty() && portable_component(s))
        .map(str::to_owned)
        .ok_or_else(|| SchemaCatalogError::InvalidComponent {
            path: path.to_owned(),
        })
}

impl SchemaCatalog for FileSystemSchemaRepository {
    /// Bounded scan of a stable tree, not protection against concurrent hostile
    /// mutation. Explicit root may resolve through a symlink; enumerated levels
    /// and candidate files must not be symlinks.
    fn list_schemas(&self) -> Result<Vec<SchemaCatalogEntry>, SchemaCatalogError> {
        let root = fs::canonicalize(&self.root).map_err(|source| io_error(&self.root, source))?;
        let repository = Self::new(root.clone())?;
        let mut entries = Vec::new();
        for flow in children(&root)? {
            if !metadata(&flow)?.is_dir() {
                continue;
            }
            let flow_id = component(&flow)?;
            for version in children(&flow)? {
                if !metadata(&version)?.is_dir() {
                    continue;
                }
                let version_id = component(&version)?;
                let candidate = version.join("flow.json");
                match fs::symlink_metadata(&candidate) {
                    Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                    Err(error) => return Err(io_error(&candidate, error)),
                    Ok(_) => {
                        if !metadata(&candidate)?.is_file() {
                            return Err(io_error(
                                &candidate,
                                io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    "schema candidate must be a regular file",
                                ),
                            ));
                        }
                    }
                }
                let key = SchemaKey {
                    flow_id: FlowId(flow_id.clone()),
                    version: SchemaVersion(version_id),
                };
                entries.push(entry(&repository.load_raw(&key)?)?);
            }
        }
        Ok(entries)
    }
}
impl SchemaCatalog for InMemorySchemaRepository {
    fn list_schemas(&self) -> Result<Vec<SchemaCatalogEntry>, SchemaCatalogError> {
        let mut raw = self.schemas.values().collect::<Vec<_>>();
        raw.sort_by(|a, b| (&a.flow_id, &a.schema_version).cmp(&(&b.flow_id, &b.schema_version)));
        raw.into_iter().map(entry).collect()
    }
}
