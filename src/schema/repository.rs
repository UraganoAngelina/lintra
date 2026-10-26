//! Explicit-key raw schema loading. No lint, compilation, dataset loading or fallback.

use super::raw::RawFlowSchema;
use crate::domain::{FlowId, SchemaKey, SchemaVersion};
use serde::de::{Error as _, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

/// A repository locates and deserializes a raw schema; it never makes it executable.
#[path = "catalog.rs"]
mod catalog;
pub use catalog::{SchemaCatalog, SchemaCatalogEntry, SchemaCatalogError};

/// Loading remains independent of catalog discovery.
pub trait SchemaRepository: Send + Sync {
    fn load_raw(&self, key: &SchemaKey) -> Result<RawFlowSchema, SchemaLoadError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaKeyComponent {
    FlowId,
    Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaKeyError {
    Empty,
    NonPortablePathComponent,
}

/// Technical loading failures, separate from record validation failures.
#[derive(Debug)]
#[non_exhaustive]
pub enum SchemaLoadError {
    Bundle(crate::BundleError),
    EmptyRoot,
    InvalidKey {
        component: SchemaKeyComponent,
        reason: SchemaKeyError,
    },
    NotFound {
        key: SchemaKey,
    },
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    KeyMismatch {
        requested: SchemaKey,
        found: SchemaKey,
    },
    DuplicateKey {
        key: SchemaKey,
    },
}

impl fmt::Display for SchemaLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bundle(error) => write!(f, "schema bundle acquisition failed: {error}"),
            Self::EmptyRoot => f.write_str("an explicit nonempty schema root is required"),
            Self::InvalidKey { component, reason } => {
                write!(f, "invalid schema selector {component:?}: {reason:?}")
            }
            Self::NotFound { key } => write!(
                f,
                "schema {:?} version {:?} was not found",
                key.flow_id.0, key.version.0
            ),
            Self::Io { path, .. } => write!(f, "cannot read schema file {}", path.display()),
            Self::Json { path, .. } => write!(f, "invalid schema JSON in {}", path.display()),
            Self::KeyMismatch { .. } => {
                f.write_str("loaded schema identity does not match the requested flow and version")
            }
            Self::DuplicateKey { .. } => {
                f.write_str("schema flow and version are already registered")
            }
        }
    }
}

impl Error for SchemaLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Bundle(error) => Some(error),
            Self::Io { source, .. } => Some(source),
            Self::Json { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn validate_selection(key: &SchemaKey) -> Result<(), SchemaLoadError> {
    for (component, value) in [
        (SchemaKeyComponent::FlowId, &key.flow_id.0),
        (SchemaKeyComponent::Version, &key.version.0),
    ] {
        if value.is_empty() {
            return Err(SchemaLoadError::InvalidKey {
                component,
                reason: SchemaKeyError::Empty,
            });
        }
    }
    Ok(())
}

// These are path-component checks, not a normative identifier alphabet.
// Unicode is retained; neither case nor version spelling is normalized.
pub(crate) fn portable_component(value: &str) -> bool {
    if matches!(value, "." | "..")
        || value.ends_with(['.', ' '])
        || value.chars().any(|c| {
            c.is_control() || matches!(c, '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*')
        })
    {
        return false;
    }
    let base = value
        .split('.')
        .next()
        .unwrap_or(value)
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    if matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL") {
        return false;
    }
    if base
        .strip_prefix("COM")
        .or_else(|| base.strip_prefix("LPT"))
        .is_some_and(|number| {
            matches!(
                number,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    {
        return false;
    }
    true
}

/// Uses only root/flow_id/version/flow.json (ARCH §12), without directory scans.
#[derive(Debug, Clone)]
pub struct FileSystemSchemaRepository {
    root: PathBuf,
}

impl FileSystemSchemaRepository {
    pub fn new(root: PathBuf) -> Result<Self, SchemaLoadError> {
        if root.as_os_str().is_empty() {
            return Err(SchemaLoadError::EmptyRoot);
        }
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn schema_path(&self, key: &SchemaKey) -> Result<PathBuf, SchemaLoadError> {
        validate_selection(key)?;
        for (component, value) in [
            (SchemaKeyComponent::FlowId, &key.flow_id.0),
            (SchemaKeyComponent::Version, &key.version.0),
        ] {
            if !portable_component(value) {
                return Err(SchemaLoadError::InvalidKey {
                    component,
                    reason: SchemaKeyError::NonPortablePathComponent,
                });
            }
        }
        Ok(self
            .root
            .join(&key.flow_id.0)
            .join(&key.version.0)
            .join("flow.json"))
    }
}

impl SchemaRepository for FileSystemSchemaRepository {
    fn load_raw(&self, key: &SchemaKey) -> Result<RawFlowSchema, SchemaLoadError> {
        let path = self.schema_path(key)?;
        let bytes = fs::read(&path).map_err(|source| {
            if source.kind() == io::ErrorKind::NotFound {
                SchemaLoadError::NotFound { key: key.clone() }
            } else {
                SchemaLoadError::Io {
                    path: path.clone(),
                    source,
                }
            }
        })?;
        let raw = parse_raw_schema(&bytes, &path)?;
        let found = schema_key(&raw);
        if found != *key {
            return Err(SchemaLoadError::KeyMismatch {
                requested: key.clone(),
                found,
            });
        }
        Ok(raw)
    }
}

/// Mutable setup followed by shared reads; every load returns an independent raw value.
#[derive(Debug, Default)]
pub struct InMemorySchemaRepository {
    schemas: HashMap<SchemaKey, RawFlowSchema>,
}

impl InMemorySchemaRepository {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rejects duplicate keys; replacement is never an implicit schema update.
    pub fn insert(&mut self, raw: RawFlowSchema) -> Result<(), SchemaLoadError> {
        let key = schema_key(&raw);
        validate_selection(&key)?;
        match self.schemas.entry(key) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                Err(SchemaLoadError::DuplicateKey {
                    key: entry.key().clone(),
                })
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(raw);
                Ok(())
            }
        }
    }
}

impl SchemaRepository for InMemorySchemaRepository {
    fn load_raw(&self, key: &SchemaKey) -> Result<RawFlowSchema, SchemaLoadError> {
        validate_selection(key)?;
        self.schemas
            .get(key)
            .cloned()
            .ok_or_else(|| SchemaLoadError::NotFound { key: key.clone() })
    }
}

fn schema_key(raw: &RawFlowSchema) -> SchemaKey {
    SchemaKey {
        flow_id: FlowId(raw.flow_id.clone()),
        version: SchemaVersion(raw.schema_version.clone()),
    }
}

/// Tooling file input with exactly the repository's duplicate-key/typed parsing.
/// This acquires a raw draft only; no selection, lint or compilation is implied.
pub fn read_raw_schema_file(path: &Path) -> Result<RawFlowSchema, SchemaLoadError> {
    let bytes = fs::read(path).map_err(|source| SchemaLoadError::Io {
        path: path.to_owned(),
        source,
    })?;
    parse_raw_schema(&bytes, path)
}

pub(crate) fn parse_raw_schema(
    bytes: &[u8],
    path: &Path,
) -> Result<RawFlowSchema, SchemaLoadError> {
    let parse_error = |source| SchemaLoadError::Json {
        path: path.to_owned(),
        source,
    };
    validate_unique_json(bytes).map_err(parse_error)?;
    serde_json::from_slice(bytes).map_err(parse_error)
}

pub(crate) fn validate_unique_json(bytes: &[u8]) -> Result<(), serde_json::Error> {
    let mut parser = serde_json::Deserializer::from_slice(bytes);
    UniqueJson::deserialize(&mut parser)?;
    parser.end()
}

// Syntax-only pass to reject duplicates even inside open metadata. It does not
// materialize a Value tree or round numbers through floats; typed parsing follows.
struct UniqueJson;
impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D: Deserializer<'de>>(parser: D) -> Result<Self, D::Error> {
        struct Structure;
        impl<'de> Visitor<'de> for Structure {
            type Value = UniqueJson;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("JSON with unique object keys")
            }
            fn visit_bool<E: serde::de::Error>(self, _: bool) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                while seq.next_element::<UniqueJson>()?.is_some() {}
                Ok(UniqueJson)
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut keys = HashSet::new();
                while let Some(key) = map.next_key::<String>()? {
                    if !keys.insert(key) {
                        return Err(A::Error::custom("duplicate JSON key"));
                    }
                    map.next_value::<UniqueJson>()?;
                }
                Ok(UniqueJson)
            }
        }
        parser.deserialize_any(Structure)
    }
}
