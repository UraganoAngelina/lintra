//! Configuration adapter: acquisition, explicit selection and bounded byte probe.
//! No record validation, regulatory rule, historical policy or fake progress.
pub use lintra::AnalysisBundle;
use lintra::*;
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub use lintra::{ConfigurationError, InputPreview};

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
            .entries()
            .iter()
            .any(|e| e.key == key && e.traces.contains(&trace))
        {
            return Err(ConfigurationError::InvalidSelection);
        }
        let registry = Arc::new(SchemaRegistry::new(
            bundle.repository(),
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
        let captured_references = Arc::new(Mutex::new(None));
        let service = ValidationService::new(registry).with_reference_data(
            Arc::new(CapturingReferences {
                inner: bundle.references(),
                captured: captured_references.clone(),
            }),
            DatasetPreparationPolicy::ContinueWithAvailableEvidence,
        );
        Ok(PreparedAnalysis {
            request,
            service: Arc::new(service),
            audit: bundle.audit().clone(),
            schema_hash: resolved.schema_hash().clone(),
            schema: resolved.schema().clone(),
            preview,
            reports: self.reports,
            captured_references,
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
    /// Read-only schema resolved by the same registry used by this request.
    pub schema: Arc<CompiledFlowSchema>,
    pub preview: InputPreview,
    pub reports: ReportSelection,
    pub captured_references: Arc<Mutex<Option<Arc<PreparedReferenceEvidence>>>>,
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

/// Captures precisely the immutable evidence supplied to ValidationService.
/// Clone shares lookup Arc handles, never rereads a possibly changed dataset.
struct CapturingReferences {
    inner: Arc<dyn ReferenceDataPreparer>,
    captured: Arc<Mutex<Option<Arc<PreparedReferenceEvidence>>>>,
}
impl ReferenceDataPreparer for CapturingReferences {
    fn prepare(
        &self,
        dependencies: &[ReferenceDependency],
        policy: DatasetPreparationPolicy,
    ) -> Result<PreparedReferenceEvidence, ReferencePreparationError> {
        let prepared = self.inner.prepare(dependencies, policy)?;
        let shared = Arc::new(prepared.clone());
        *self
            .captured
            .lock()
            .map_err(|_| ReferencePreparationError::InconsistentEvidence)? = Some(shared);
        Ok(prepared)
    }
}
