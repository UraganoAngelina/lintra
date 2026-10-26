//! Whole-bundle resolution, DET A.1 levels 1–4. No cwd/home discovery.
use std::{
    error::Error,
    ffi::OsString,
    fmt, fs, io,
    path::{Path, PathBuf},
};

/// Pair selected by the caller. One supplied directory never enables fallback.
#[derive(Debug, Default)]
pub struct CliDirs {
    pub schema_dir: Option<PathBuf>,
    pub reference_data_dir: Option<PathBuf>,
}
/// External directories are selected together; embedded is the final level only.
#[derive(Debug, PartialEq, Eq)]
pub enum BundleSource {
    #[cfg(feature = "embedded-bundle")]
    Embedded,
    External {
        schema_root: PathBuf,
        reference_data_root: PathBuf,
    },
}
/// Native environment values preserve paths without lossy UTF-8 conversions.
pub trait EnvProvider {
    fn var_os(&self, key: &str) -> Option<OsString>;
}
/// The only production environment reader; tests inject an independent map.
pub struct SystemEnv;
impl EnvProvider for SystemEnv {
    fn var_os(&self, key: &str) -> Option<OsString> {
        std::env::var_os(key)
    }
}
/// The winning level is retained for actionable incomplete-pair diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleLevel {
    CommandLine,
    Environment,
    Executable,
}
/// Identify the absent component without guessing the other root or its source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryKind {
    Schemas,
    ReferenceData,
}
/// Typed filesystem/configuration errors; none is a normative ValidationError.
#[derive(Debug)]
pub enum BundleResolveError {
    Incomplete {
        level: BundleLevel,
        missing: DirectoryKind,
    },
    EmptyPath {
        kind: DirectoryKind,
    },
    NotDirectory {
        kind: DirectoryKind,
        path: PathBuf,
    },
    Inspect {
        kind: DirectoryKind,
        path: PathBuf,
        source: io::Error,
    },
    InvalidExecutablePath,
    #[cfg(not(feature = "embedded-bundle"))]
    NoExternalBundle,
}
impl fmt::Display for BundleResolveError {
    /// These paths are adapter diagnostics, never embedded in validation reports.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Incomplete { level, missing } => write!(
                f,
                "incomplete external bundle at {level:?}: missing {missing:?}"
            ),
            Self::EmptyPath { kind } => write!(f, "empty {kind:?} directory"),
            Self::NotDirectory { kind, path } => {
                write!(f, "{kind:?} is not a directory: {}", path.display())
            }
            Self::Inspect { kind, path, .. } => {
                write!(f, "cannot inspect {kind:?} directory: {}", path.display())
            }
            Self::InvalidExecutablePath => {
                f.write_str("executable path needs an explicit parent directory")
            }
            #[cfg(not(feature = "embedded-bundle"))]
            Self::NoExternalBundle => f.write_str(
                "no external bundle found; embedded bundle is not available in this build",
            ),
        }
    }
}
impl Error for BundleResolveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Inspect { source, .. } => Some(source),
            _ => None,
        }
    }
}
/// Reject empty roots before metadata (Path::new("") must not mean cwd).
/// Explicit metadata failures propagate; a missing selection never means fallback.
fn directory(path: PathBuf, kind: DirectoryKind) -> Result<PathBuf, BundleResolveError> {
    if path.as_os_str().is_empty() {
        return Err(BundleResolveError::EmptyPath { kind });
    }
    let metadata = fs::metadata(&path).map_err(|source| BundleResolveError::Inspect {
        kind,
        path: path.clone(),
        source,
    })?;
    if !metadata.is_dir() {
        return Err(BundleResolveError::NotDirectory { kind, path });
    }
    Ok(path)
}
/// A level with neither component is absent; a partial pair fails immediately.
/// Both roots must be actual directories before returning the external bundle.
fn selected_pair(
    schema: Option<PathBuf>,
    reference: Option<PathBuf>,
    level: BundleLevel,
) -> Result<Option<BundleSource>, BundleResolveError> {
    match (schema, reference) {
        (None, None) => Ok(None),
        (Some(_), None) => Err(BundleResolveError::Incomplete {
            level,
            missing: DirectoryKind::ReferenceData,
        }),
        (None, Some(_)) => Err(BundleResolveError::Incomplete {
            level,
            missing: DirectoryKind::Schemas,
        }),
        (Some(schema), Some(reference)) => Ok(Some(BundleSource::External {
            schema_root: directory(schema, DirectoryKind::Schemas)?,
            reference_data_root: directory(reference, DirectoryKind::ReferenceData)?,
        })),
    }
}
/// Probe only executable-adjacent defaults. Only NotFound means absent; access
/// errors and wrong types remain explicit, so corruption never enables fallback.
fn adjacent(path: PathBuf, kind: DirectoryKind) -> Result<Option<PathBuf>, BundleResolveError> {
    match fs::symlink_metadata(&path) {
        // A dangling link is a present but broken external selection. Never
        // treat its missing target as permission to use the embedded snapshot.
        Ok(metadata) if metadata.file_type().is_symlink() => directory(path, kind).map(Some),
        Ok(metadata) if metadata.is_dir() => Ok(Some(path)),
        Ok(_) => Err(BundleResolveError::NotDirectory { kind, path }),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(BundleResolveError::Inspect { kind, path, source }),
    }
}
/// Resolve complete pairs by priority, without mixing levels or snapshots.
/// Lookup of selected schema/version remains the repository's responsibility;
/// this function neither opens datasets nor claims a provider is available.
pub fn resolve_bundle_source(
    cli: &CliDirs,
    env: &dyn EnvProvider,
    exe_path: &Path,
) -> Result<BundleSource, BundleResolveError> {
    if let Some(source) = selected_pair(
        cli.schema_dir.clone(),
        cli.reference_data_dir.clone(),
        BundleLevel::CommandLine,
    )? {
        return Ok(source);
    }
    if let Some(source) = selected_pair(
        env.var_os("LINTRA_SCHEMA_DIR").map(PathBuf::from),
        env.var_os("LINTRA_REFERENCE_DATA_DIR").map(PathBuf::from),
        BundleLevel::Environment,
    )? {
        return Ok(source);
    }
    let parent = exe_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or(BundleResolveError::InvalidExecutablePath)?;
    let schema = adjacent(parent.join("schemas"), DirectoryKind::Schemas)?;
    let reference = adjacent(parent.join("reference-data"), DirectoryKind::ReferenceData)?;
    if let Some(source) = selected_pair(schema, reference, BundleLevel::Executable)? {
        return Ok(source);
    }
    #[cfg(feature = "embedded-bundle")]
    {
        Ok(BundleSource::Embedded)
    }
    #[cfg(not(feature = "embedded-bundle"))]
    {
        Err(BundleResolveError::NoExternalBundle)
    }
}
