//! Shared native-adapter bundle acquisition; listing never certifies a schema.
use crate::*;
use std::{collections::BTreeSet, error::Error, fs, io, path::Path, sync::Arc};

type Failure = Box<dyn Error + Send + Sync>;

pub struct AnalysisBundle {
    entries: Vec<SchemaCatalogEntry>,
    repository: Arc<dyn SchemaRepository>,
    references: Arc<dyn ReferenceDataPreparer>,
    audit: BundleAudit,
}
impl AnalysisBundle {
    pub fn resolve(
        dirs: &BundleDirectories,
        env: &dyn BundleEnvironment,
        exe: &Path,
    ) -> Result<Self, Failure> {
        Self::from_selection(resolve_bundle_source(dirs, env, exe)?)
    }
    pub fn from_selection(source: BundleSelection) -> Result<Self, Failure> {
        if let Some((schema, reference)) = source.external_roots() {
            let manifest = external_bundle_manifest_path(None, schema, reference)?
                .map(|path| {
                    fs::read(path)
                        .map_err(BundleError::Io)
                        .and_then(|bytes| BundleManifest::parse(&bytes))
                })
                .transpose()?;
            let filesystem = Arc::new(FileSystemSchemaRepository::new(schema.to_owned())?);
            let entries = filesystem.list_schemas()?;
            let audit = BundleAudit::external(manifest.as_ref());
            let (repository, references): (
                Arc<dyn SchemaRepository>,
                Arc<dyn ReferenceDataPreparer>,
            ) = match manifest {
                Some(manifest) => (
                    Arc::new(ManifestSchemaRepository::new(
                        schema.to_owned(),
                        manifest.clone(),
                    )?),
                    Arc::new(ManifestReferenceDataPreparer::new(
                        reference.to_owned(),
                        manifest,
                    )?),
                ),
                None => (
                    filesystem,
                    Arc::new(FileSystemReferenceDataPreparer::new(reference.to_owned())?),
                ),
            };
            // Listing is acquisition, not certification. Enforce selected bundle
            // integrity for every acquired schema before exposing its metadata.
            for entry in &entries {
                repository.load_raw(&entry.key)?;
            }
            return Ok(Self {
                entries,
                repository,
                references,
                audit,
            });
        }
        if source.is_embedded() {
            #[cfg(feature = "embedded-bundle")]
            return Self::builtin();
        }
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "selected data source is unavailable in this UI build",
        )
        .into())
    }
    #[cfg(feature = "embedded-bundle")]
    pub fn builtin() -> Result<Self, Failure> {
        let bundle = EmbeddedBundle::builtin()?;
        let audit = BundleAudit::embedded(&bundle);
        let repo = Arc::new(EmbeddedSchemaRepository::new(bundle.clone())?);
        let entries = repo.list_schemas()?;
        Ok(Self {
            entries,
            repository: repo,
            references: Arc::new(EmbeddedReferenceDataPreparer::new(bundle)),
            audit,
        })
    }
    pub fn repository(&self) -> Arc<dyn SchemaRepository> {
        self.repository.clone()
    }
    pub fn references(&self) -> Arc<dyn ReferenceDataPreparer> {
        self.references.clone()
    }
    pub fn entries(&self) -> &[SchemaCatalogEntry] {
        &self.entries
    }
    pub fn audit(&self) -> &BundleAudit {
        &self.audit
    }
    pub fn flows(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|e| e.key.flow_id.0.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    pub fn versions(&self, flow: &str) -> Vec<String> {
        self.entries
            .iter()
            .filter(|e| e.key.flow_id.0 == flow)
            .map(|e| e.key.version.0.clone())
            .collect()
    }
    pub fn traces(&self, key: &SchemaKey) -> Vec<String> {
        self.entries
            .iter()
            .find(|e| e.key == *key)
            .map(|e| e.traces.iter().map(|id| id.0.clone()).collect())
            .unwrap_or_default()
    }
    pub fn origin_label(&self) -> String {
        let source = if self.audit.source() == "embedded" {
            "incorporati"
        } else {
            "esterni"
        };
        format!(
            "Dati {source} · bundle {}",
            self.audit.bundle_version().unwrap_or("non dichiarato")
        )
    }
}
