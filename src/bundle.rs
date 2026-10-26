//! Bundle identity and exact file hashes. No schema compilation or record rules.
use crate::schema::repository::validate_unique_json;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, error::Error, fmt};

#[derive(Debug)]
#[non_exhaustive]
pub enum BundleError {
    Json(serde_json::Error),
    Io(std::io::Error),
    InvalidManifest(&'static str),
    UnsafePath(String),
    MissingFile(String),
    Integrity(String),
}
impl fmt::Display for BundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(_) => f.write_str("cannot acquire bundle metadata or payload"),
            Self::Json(_) => f.write_str("invalid bundle manifest JSON"),
            Self::InvalidManifest(reason) => write!(f, "invalid bundle manifest: {reason}"),
            Self::UnsafePath(path) => write!(f, "unsafe bundle path: {path}"),
            Self::MissingFile(path) => write!(f, "file absent from selected bundle: {path}"),
            Self::Integrity(path) => write!(f, "bundle file SHA-256 mismatch: {path}"),
        }
    }
}
impl Error for BundleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Json(e) => Some(e),
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleManifest {
    bundle_version: String,
    schema_format_version: u32,
    files: BTreeMap<String, String>,
}
impl BundleManifest {
    pub fn parse(bytes: &[u8]) -> Result<Self, BundleError> {
        validate_unique_json(bytes).map_err(BundleError::Json)?;
        let value: Self = serde_json::from_slice(bytes).map_err(BundleError::Json)?;
        if value.bundle_version.is_empty() || value.files.is_empty() {
            return Err(BundleError::InvalidManifest(
                "version and payload inventory required",
            ));
        }
        if value.schema_format_version != 1 {
            return Err(BundleError::InvalidManifest("unsupported schema format"));
        }
        for (path, hash) in &value.files {
            if !crate::reference_data::filesystem::portable_relative(path)
                || !(path.starts_with("schemas/") || path.starts_with("reference-data/"))
            {
                return Err(BundleError::UnsafePath(path.clone()));
            }
            if hash.len() != 64
                || !hash
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(BundleError::InvalidManifest("expected lowercase SHA-256"));
            }
        }
        Ok(value)
    }
    pub fn bundle_version(&self) -> &str {
        &self.bundle_version
    }
    pub fn schema_format_version(&self) -> u32 {
        self.schema_format_version
    }
    pub fn files(&self) -> &BTreeMap<String, String> {
        &self.files
    }
    pub fn verify_bytes(&self, path: &str, bytes: &[u8]) -> Result<(), BundleError> {
        let expected = self
            .files
            .get(path)
            .ok_or_else(|| BundleError::MissingFile(path.into()))?;
        let actual = Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        if &actual != expected {
            return Err(BundleError::Integrity(path.into()));
        }
        Ok(())
    }
}

#[cfg(feature = "embedded-bundle")]
#[derive(Debug)]
pub struct EmbeddedFile {
    pub path: &'static str,
    pub bytes: &'static [u8],
}
#[cfg(feature = "embedded-bundle")]
#[derive(Debug, Clone)]
pub struct EmbeddedBundle {
    manifest: std::sync::Arc<BundleManifest>,
    files: std::sync::Arc<BTreeMap<&'static str, &'static [u8]>>,
}
#[cfg(feature = "embedded-bundle")]
impl EmbeddedBundle {
    /// Hash every supplied payload, reject duplicates/missing/unlisted assets.
    /// Bytes remain borrowed from the executable; no temporary extraction.
    pub fn from_static(
        manifest: &[u8],
        files: &'static [EmbeddedFile],
    ) -> Result<Self, BundleError> {
        let manifest = BundleManifest::parse(manifest)?;
        let mut inventory = BTreeMap::new();
        for file in files {
            if inventory.insert(file.path, file.bytes).is_some() {
                return Err(BundleError::InvalidManifest("duplicate embedded path"));
            }
            manifest.verify_bytes(file.path, file.bytes)?;
        }
        for path in manifest.files.keys() {
            if !inventory.contains_key(path.as_str()) {
                return Err(BundleError::MissingFile(path.clone()));
            }
        }
        Ok(Self {
            manifest: std::sync::Arc::new(manifest),
            files: std::sync::Arc::new(inventory),
        })
    }
    pub fn builtin() -> Result<Self, BundleError> {
        static FILES: &[EmbeddedFile] = &[
            EmbeddedFile {
                path: "schemas/SYNTHETIC/v1/flow.json",
                bytes: include_bytes!("../bundles/embedded/schemas/SYNTHETIC/v1/flow.json"),
            },
            EmbeddedFile {
                path: "reference-data/example-codes.json",
                bytes: include_bytes!("../bundles/embedded/reference-data/example-codes.json"),
            },
        ];
        Self::from_static(include_bytes!("../bundles/embedded/MANIFEST.json"), FILES)
    }
    pub fn manifest(&self) -> &BundleManifest {
        &self.manifest
    }
    pub fn file(&self, path: &str) -> Option<&'static [u8]> {
        self.files.get(path).copied()
    }
}

/// Provenance supplied by the adapter that actually selected the bundle.
/// Private fields keep source/version/manifest status consistent. An absent
/// external version is serialized as null, never inferred from a schema or path.
#[derive(Debug, Clone, Serialize)]
pub struct BundleAudit {
    bundle_source: &'static str,
    bundle_version: Option<String>,
    bundle_manifest_status: &'static str,
}
impl BundleAudit {
    pub fn external(manifest: Option<&BundleManifest>) -> Self {
        Self {
            bundle_source: "external",
            bundle_version: manifest.map(|m| m.bundle_version().to_owned()),
            bundle_manifest_status: if manifest.is_some() {
                "present"
            } else {
                "absent"
            },
        }
    }
    #[cfg(feature = "embedded-bundle")]
    pub fn embedded(bundle: &EmbeddedBundle) -> Self {
        Self {
            bundle_source: "embedded",
            bundle_version: Some(bundle.manifest().bundle_version().to_owned()),
            bundle_manifest_status: "present",
        }
    }
}

/// Optional manifest guard for an explicitly configured filesystem schema root.
/// Reads once: exactly the bytes checked against the manifest are deserialized.
/// Canonical containment rejects symlink escape, without claiming protection
/// against hostile concurrent replacement of filesystem objects.
pub struct ManifestSchemaRepository {
    repository: crate::FileSystemSchemaRepository,
    manifest: BundleManifest,
}
impl ManifestSchemaRepository {
    pub fn new(
        root: std::path::PathBuf,
        manifest: BundleManifest,
    ) -> Result<Self, crate::SchemaLoadError> {
        Ok(Self {
            repository: crate::FileSystemSchemaRepository::new(root)?,
            manifest,
        })
    }
}
impl crate::SchemaRepository for ManifestSchemaRepository {
    fn load_raw(
        &self,
        key: &crate::SchemaKey,
    ) -> Result<crate::RawFlowSchema, crate::SchemaLoadError> {
        use crate::SchemaLoadError;
        let path = self.repository.schema_path(key)?;
        let acquire = || -> Result<Vec<u8>, BundleError> {
            let root = std::fs::canonicalize(self.repository.root()).map_err(BundleError::Io)?;
            let actual = std::fs::canonicalize(&path).map_err(BundleError::Io)?;
            if !actual.starts_with(&root) {
                return Err(BundleError::UnsafePath(path.display().to_string()));
            }
            let bytes = std::fs::read(actual).map_err(BundleError::Io)?;
            self.manifest.verify_bytes(
                &format!("schemas/{}/{}/flow.json", key.flow_id.0, key.version.0),
                &bytes,
            )?;
            Ok(bytes)
        };
        let bytes = acquire().map_err(SchemaLoadError::Bundle)?;
        let raw = crate::schema::repository::parse_raw_schema(&bytes, &path)?;
        if raw.schema_format_version != self.manifest.schema_format_version() {
            return Err(SchemaLoadError::Bundle(BundleError::InvalidManifest(
                "schema format differs from manifest",
            )));
        }
        let found = crate::SchemaKey {
            flow_id: crate::FlowId(raw.flow_id.clone()),
            version: crate::SchemaVersion(raw.schema_version.clone()),
        };
        if found != *key {
            return Err(SchemaLoadError::KeyMismatch {
                requested: key.clone(),
                found,
            });
        }
        Ok(raw)
    }
}

/// Checks declaration metadata only, then delegates byte acquisition and trust
/// classification to the existing filesystem preparer. No global file scan,
/// digest override, dataset fallback or altered Strict/Continue behavior.
pub struct ManifestReferenceDataPreparer {
    preparer: crate::FileSystemReferenceDataPreparer,
    manifest: BundleManifest,
}
impl ManifestReferenceDataPreparer {
    pub fn new(
        root: std::path::PathBuf,
        manifest: BundleManifest,
    ) -> Result<Self, crate::FilesystemReferenceError> {
        Ok(Self {
            preparer: crate::FileSystemReferenceDataPreparer::new(root)?,
            manifest,
        })
    }
}
impl crate::ReferenceDataPreparer for ManifestReferenceDataPreparer {
    fn prepare(
        &self,
        dependencies: &[crate::ReferenceDependency],
        policy: crate::DatasetPreparationPolicy,
    ) -> Result<crate::PreparedReferenceEvidence, crate::ReferencePreparationError> {
        for dependency in dependencies {
            let path = format!("reference-data/{}", dependency.bundle_path);
            let digest = dependency
                .sha256
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            if self.manifest.files().get(&path) != Some(&digest) {
                return Err(crate::ReferencePreparationError::Provider(Box::new(
                    BundleError::InvalidManifest(
                        "reference declaration path/digest differs from manifest",
                    ),
                )));
            }
        }
        self.preparer.prepare(dependencies, policy)
    }
}
