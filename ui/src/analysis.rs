//! Configuration adapter: acquisition, explicit selection and bounded byte probe.
//! No record validation, regulatory rule, historical policy or fake progress.
use lintra::*;
use std::{
    collections::BTreeSet,
    error::Error,
    fmt, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::Arc,
};
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
#[derive(Debug)]
#[non_exhaustive]
pub enum ConfigurationError {
    Missing(&'static str),
    InvalidSelection,
    Input(io::Error),
    Bom(InputError),
    Schema(SchemaResolutionError),
}
impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(field) => write!(f, "Seleziona {field}."),
            Self::InvalidSelection => {
                f.write_str("La selezione non appartiene al catalogo corrente.")
            }
            Self::Input(_) => f.write_str("Il file non è accessibile o non è un file regolare."),
            Self::Bom(InputError::BomRejected(_)) => {
                f.write_str("BOM non supportato: il file non è stato modificato.")
            }
            Self::Bom(_) => {
                f.write_str("Lettura del file non riuscita durante il controllo preliminare.")
            }
            Self::Schema(error) => write!(f, "Schema non eseguibile: {error}"),
        }
    }
}
impl Error for ConfigurationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Input(e) => Some(e),
            Self::Bom(e) => Some(e),
            Self::Schema(e) => Some(e),
            _ => None,
        }
    }
}
/// Only counters escape this probe. Raw patient bytes are neither retained,
/// decoded, normalized, logged nor treated as proof of normative validity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputPreview {
    pub file_size: u64,
    pub sampled_bytes: usize,
    pub lf: u64,
    pub crlf: u64,
    pub lone_cr: u64,
    pub reached_eof: bool,
}
impl InputPreview {
    pub fn inspect(path: &Path) -> Result<Self, ConfigurationError> {
        let initial = fs::metadata(path).map_err(ConfigurationError::Input)?;
        if !initial.is_file() {
            return Err(ConfigurationError::Input(io::Error::new(
                io::ErrorKind::InvalidInput,
                "input must be a regular file",
            )));
        }
        let file = fs::File::open(path).map_err(ConfigurationError::Input)?;
        let metadata = file.metadata().map_err(ConfigurationError::Input)?;
        if !metadata.is_file() {
            return Err(ConfigurationError::Input(io::Error::new(
                io::ErrorKind::InvalidInput,
                "input must be a regular file",
            )));
        }
        let mut reader = preflight_bom(file, BomPolicy::Reject).map_err(ConfigurationError::Bom)?;
        let mut result = Self {
            file_size: metadata.len(),
            sampled_bytes: 0,
            lf: 0,
            crlf: 0,
            lone_cr: 0,
            reached_eof: false,
        };
        let mut previous_cr = false;
        let mut buffer = [0u8; 4096];
        while result.sampled_bytes < 65536 {
            let limit = buffer.len().min(65536 - result.sampled_bytes);
            let count = match reader.read(&mut buffer[..limit]) {
                Ok(n) => n,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(ConfigurationError::Input(e)),
            };
            if count == 0 {
                result.reached_eof = true;
                break;
            }
            for &byte in &buffer[..count] {
                if byte == b'\n' {
                    if previous_cr {
                        result.crlf += 1;
                    } else {
                        result.lf += 1;
                    }
                } else if previous_cr {
                    result.lone_cr += 1;
                }
                previous_cr = byte == b'\r';
            }
            result.sampled_bytes += count;
        }
        if previous_cr && result.reached_eof {
            result.lone_cr += 1;
        }
        Ok(result)
    }
    pub fn label(&self) -> String {
        let endings = match (self.lf > 0, self.crlf > 0) {
            (true, true) => "LF e CRLF",
            (true, false) => "LF",
            (false, true) => "CRLF",
            _ => "nessun LF nel campione",
        };
        let cr = if self.lone_cr > 0 {
            " · CR isolati rilevati"
        } else {
            ""
        };
        format!(
            "Campione {} byte: {endings} · nessun BOM{cr}",
            self.sampled_bytes
        )
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportSelection {
    pub jsonl: bool,
    pub xlsx: bool,
    pub csv: bool,
}
impl Default for ReportSelection {
    fn default() -> Self {
        Self {
            jsonl: true,
            xlsx: true,
            csv: false,
        }
    }
}
#[derive(Debug, Default)]
pub struct AnalysisSelection {
    pub flow: Option<String>,
    pub version: Option<String>,
    pub trace: Option<String>,
    pub encoding: Option<RawResolvedEncoding>,
    pub file: Option<PathBuf>,
    pub reports: ReportSelection,
}
impl AnalysisSelection {
    /// Changing an ancestor invalidates dependent IDs; no single/latest version
    /// or trace is chosen silently, even if only one item exists in the catalog.
    pub fn select_flow(&mut self, flow: Option<String>) {
        self.flow = flow;
        self.version = None;
        self.trace = None;
    }
    pub fn select_version(&mut self, version: Option<String>) {
        self.version = version;
        self.trace = None;
    }
    pub fn key(&self) -> Result<SchemaKey, ConfigurationError> {
        Ok(SchemaKey {
            flow_id: FlowId(
                self.flow
                    .clone()
                    .ok_or(ConfigurationError::Missing("il flusso"))?,
            ),
            version: SchemaVersion(self.version.clone().ok_or(ConfigurationError::Missing(
                "la versione esplicita dello schema",
            ))?),
        })
    }
    pub fn prepare(&self, bundle: &AnalysisBundle) -> Result<PreparedAnalysis, ConfigurationError> {
        let key = self.key()?;
        let trace = TraceId(
            self.trace
                .clone()
                .ok_or(ConfigurationError::Missing("il tracciato"))?,
        );
        let encoding = self
            .encoding
            .clone()
            .ok_or(ConfigurationError::Missing("l’encoding"))?;
        if !bundle
            .entries
            .iter()
            .any(|e| e.key == key && e.traces.contains(&trace))
        {
            return Err(ConfigurationError::InvalidSelection);
        }
        let registry = Arc::new(SchemaRegistry::new(
            bundle.repository.clone(),
            SchemaCompiler,
            encoding,
        ));
        // Compiler/repository remain authoritative; UI never translates regional
        // notes or removes unsupported capabilities to enable an analysis.
        let resolved = registry
            .resolve_with_hash(&key)
            .map_err(ConfigurationError::Schema)?;
        let path = self
            .file
            .as_ref()
            .filter(|p| !p.as_os_str().is_empty())
            .ok_or(ConfigurationError::Missing("il file di input"))?;
        let preview = InputPreview::inspect(path)?;
        let request = ValidationRequest {
            schema_key: key,
            scope: ValidationScope::Trace(TraceValidationInput {
                trace_id: trace,
                source: InputSource::Path(path.clone()),
            }),
            mode: ValidationMode::LocalOnly,
            options: ValidationOptions {
                record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                newline_policy: NewlinePolicy::AutoLfOrCrlf,
                max_errors: None,
            },
        };
        let service = ValidationService::new(registry).with_reference_data(
            bundle.references.clone(),
            DatasetPreparationPolicy::ContinueWithAvailableEvidence,
        );
        Ok(PreparedAnalysis {
            request,
            service: Arc::new(service),
            audit: bundle.audit.clone(),
            schema_hash: resolved.schema_hash().clone(),
            preview,
            reports: self.reports,
        })
    }
}
/// Prepared intent only. The dedicated worker introduced by T-P16-04 executes
/// ValidationService; preparing a request never starts record validation.
#[derive(Clone)]
pub struct PreparedAnalysis {
    pub request: ValidationRequest,
    pub service: Arc<ValidationService>,
    pub audit: BundleAudit,
    pub schema_hash: Sha256Digest,
    pub preview: InputPreview,
    pub reports: ReportSelection,
}

#[derive(Debug, Clone)]
pub struct BrowserEntry {
    pub label: String,
    pub path: PathBuf,
    pub directory: bool,
    pub selectable: bool,
}
/// Slint's integrated browser uses no GTK/portal/helper executable. Directory
/// metadata only; files are not opened until the operator selects one.
pub fn browse_directory(path: &Path) -> io::Result<Vec<BrowserEntry>> {
    if path.as_os_str().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "explicit browser directory required",
        ));
    }
    let mut result = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata();
        let label = entry.file_name().to_str().map(str::to_owned);
        let selectable = label.is_some()
            && path.to_str().is_some()
            && metadata.as_ref().is_ok_and(|m| m.is_file() || m.is_dir());
        let directory = metadata.is_ok_and(|m| m.is_dir());
        result.push(BrowserEntry {
            label: label.unwrap_or_else(|| "Nome non UTF-8 (non supportato)".into()),
            path,
            directory,
            selectable,
        });
    }
    result.sort_by(|a, b| {
        b.directory
            .cmp(&a.directory)
            .then_with(|| a.label.cmp(&b.label))
    });
    Ok(result)
}
