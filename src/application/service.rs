//! LocalOnly orchestration and request preflight with typed technical failures.
use super::request::*;
use crate::{
    CompiledFlowSchema, CompiledRecordSchema, InputSource, SchemaKey, TraceId,
    engine::preflight::{InputPlanError, PreparedTrace, prepare_trace},
};
use std::{collections::HashSet, error::Error, fmt, path::Path};

/// Application composes the engine's typed errors, never the reverse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidRequestError {
    SchemaKeyMismatch {
        requested: SchemaKey,
        loaded: SchemaKey,
    },
    UnsupportedMode(ValidationMode),
    UnsupportedFlowScope,
    UnsupportedMaxErrors {
        limit: u64,
    },
    EmptyFlow,
    DuplicateTrace(TraceId),
    Input(InputPlanError),
}
impl fmt::Display for InvalidRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SchemaKeyMismatch { .. } => "request and compiled schema keys differ",
            Self::UnsupportedMode(_) => "Full validation is not available until P13",
            Self::UnsupportedFlowScope => "Flow scope is not available until P13",
            Self::UnsupportedMaxErrors { .. } => "max_errors must be None in the current engine",
            Self::EmptyFlow => "flow input must contain at least one trace",
            Self::DuplicateTrace(_) => "trace input is duplicated",
            Self::Input(_) => "input preflight failed",
        })
    }
}
impl Error for InvalidRequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Input(error) => Some(error),
            _ => None,
        }
    }
}

/// Only a supported Trace/LocalOnly request can create this plan. Borrowed
/// checked inputs and copied options cannot change while the plan is in use.
#[derive(Debug)]
pub struct PreflightPlan<'schema, 'input> {
    input: PreparedTrace<'schema, 'input>,
    options: ValidationOptions,
}
impl<'schema, 'input> PreflightPlan<'schema, 'input> {
    pub fn trace(&self) -> &'schema CompiledRecordSchema {
        self.input.trace()
    }
    pub fn path(&self) -> &'input Path {
        self.input.path()
    }
    pub fn options(&self) -> ValidationOptions {
        self.options
    }
}

/// DET §86 orchestration stays in application (DET §5); engine prepares neutral
/// input references. Error precedence: key, mode, options, scope/input checks.
/// No file read, counter, dataset load or normative check occurs here.
pub fn preflight<'schema, 'input>(
    request: &'input ValidationRequest,
    schema: &'schema CompiledFlowSchema,
) -> Result<PreflightPlan<'schema, 'input>, InvalidRequestError> {
    if &request.schema_key != schema.key() {
        return Err(InvalidRequestError::SchemaKeyMismatch {
            requested: request.schema_key.clone(),
            loaded: schema.key().clone(),
        });
    }
    if request.mode == ValidationMode::Full {
        return Err(InvalidRequestError::UnsupportedMode(request.mode));
    }
    if let Some(limit) = request.options.max_errors {
        return Err(InvalidRequestError::UnsupportedMaxErrors { limit });
    }
    match &request.scope {
        ValidationScope::Trace(input) => {
            let InputSource::Path(path) = &input.source;
            Ok(PreflightPlan {
                input: prepare_trace(schema, &input.trace_id, path)
                    .map_err(InvalidRequestError::Input)?,
                options: request.options,
            })
        }
        ValidationScope::Flow(flow) => {
            if flow.traces.is_empty() {
                return Err(InvalidRequestError::EmptyFlow);
            }
            let mut seen = HashSet::new();
            for input in &flow.traces {
                if !seen.insert(&input.trace_id) {
                    return Err(InvalidRequestError::DuplicateTrace(input.trace_id.clone()));
                }
                let InputSource::Path(path) = &input.source;
                prepare_trace(schema, &input.trace_id, path).map_err(InvalidRequestError::Input)?;
            }
            // This is a capability boundary, not a normative incompatibility.
            // P7 is explicitly Trace/LocalOnly; never silently pick one input.
            Err(InvalidRequestError::UnsupportedFlowScope)
        }
    }
}

/// Technical run failures with concrete module payloads. ReferenceData/Context
/// variants remain P11/P13, as ratified; no normative ValidationError is wrapped.
#[derive(Debug)]
#[non_exhaustive]
pub enum ValidationRunError {
    SchemaLoad(crate::SchemaLoadError),
    SchemaCompilation(crate::SchemaCompilationError),
    SchemaCanonical(crate::CanonicalSchemaError),
    InvalidRequest(InvalidRequestError),
    Input(crate::InputError),
    Record(crate::RecordValidationError),
    Trace(crate::TraceValidationError),
    Summary(crate::SummaryBuildError),
    Report(crate::ReportError),
}
impl fmt::Display for ValidationRunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SchemaLoad(_) => "run schema loading failed",
            Self::SchemaCompilation(_) => "run schema compilation failed",
            Self::SchemaCanonical(_) => "run schema fingerprint failed",
            Self::InvalidRequest(_) => "run request is invalid or unsupported",
            Self::Input(_) => "run input failed",
            Self::Record(_) => "run record processing failed",
            Self::Trace(_) => "run trace processing failed",
            Self::Summary(_) => "run summary construction failed",
            Self::Report(_) => "run report failed",
        })
    }
}
impl Error for ValidationRunError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(match self {
            Self::SchemaLoad(e) => e,
            Self::SchemaCompilation(e) => e,
            Self::SchemaCanonical(e) => e,
            Self::InvalidRequest(e) => e,
            Self::Input(e) => e,
            Self::Record(e) => e,
            Self::Trace(e) => e,
            Self::Summary(e) => e,
            Self::Report(e) => e,
        })
    }
}
impl From<crate::SchemaResolutionError> for ValidationRunError {
    fn from(error: crate::SchemaResolutionError) -> Self {
        match error {
            crate::SchemaResolutionError::Load(e) => Self::SchemaLoad(e),
            crate::SchemaResolutionError::Compilation(e) => Self::SchemaCompilation(e),
            crate::SchemaResolutionError::Canonical(e) => Self::SchemaCanonical(e),
        }
    }
}
impl From<crate::TraceValidationError> for ValidationRunError {
    /// Expose canonical Input/Report/Summary categories without losing causes;
    /// remaining record/trace invariants keep their original typed payloads.
    fn from(error: crate::TraceValidationError) -> Self {
        use crate::{
            FieldPipelineError as Field, RecordValidationError as Record,
            TraceValidationError as Trace,
        };
        match error {
            Trace::Input(e) => Self::Input(e),
            Trace::Record(Record::Report(e) | Record::Pipeline(Field::Report(e))) => {
                Self::Report(e)
            }
            Trace::Record(Record::Summary(e) | Record::Pipeline(Field::Summary(e))) => {
                Self::Summary(e)
            }
            Trace::Record(e) => Self::Record(e),
            e @ Trace::PreviousFailure => Self::Trace(e),
        }
    }
}

/// LocalOnly application orchestrator. The real registry is injected; no fake
/// reference provider or context store stands in for the future P11/P13 ports.
pub struct ValidationService {
    schema_registry: std::sync::Arc<crate::SchemaRegistry>,
    clock: std::sync::Arc<dyn crate::Clock>,
}

/// Build the currently supported LocalOnly service from an explicit schema root.
///
/// The repository rejects an empty root; no working-directory/home discovery or
/// fallback is performed. Schema files are loaded lazily from
/// `schema_root/<flow>/<version>/flow.json` when `validate` resolves the request.
/// Loading/compilation failures, including unsupported encoding, are therefore
/// returned by `validate` as typed `ValidationRunError`s, not hidden here.
/// The encoding binding is passed unchanged to the registry. The existing `new`
/// constructor supplies SystemClock, keeping a single standard wiring path.
/// Full/reference-data/persistence wiring remains a future, separate factory.
pub fn build_default_local_service(
    schema_root: std::path::PathBuf,
    encoding: crate::RawResolvedEncoding,
) -> Result<ValidationService, crate::SchemaLoadError> {
    let repository = crate::FileSystemSchemaRepository::new(schema_root)?;
    let registry = crate::SchemaRegistry::new(
        std::sync::Arc::new(repository),
        crate::SchemaCompiler,
        encoding,
    );
    Ok(ValidationService::new(std::sync::Arc::new(registry)))
}

impl ValidationService {
    pub fn new(schema_registry: std::sync::Arc<crate::SchemaRegistry>) -> Self {
        Self::with_clock(schema_registry, std::sync::Arc::new(crate::SystemClock))
    }
    /// Explicit clock injection for deterministic runs/adapters (DET §166).
    /// The registry-only constructor remains the system-clock convenience.
    pub fn with_clock(
        schema_registry: std::sync::Arc<crate::SchemaRegistry>,
        clock: std::sync::Arc<dyn crate::Clock>,
    ) -> Self {
        Self {
            schema_registry,
            clock,
        }
    }
    /// Normative invalidity is Ok(CompletedInvalid); cooperative cancellation is
    /// Ok(Cancelled). On Err, no completed outcome is returned; the caller must
    /// abort any atomic output, including a failed summary write. The final summary checkpoint fixes run status:
    /// cancellation after RunCompleted cannot retroactively change that result.
    pub fn validate(
        &self,
        request: ValidationRequest,
        sink: &mut dyn crate::ValidationSink,
        progress: &mut dyn crate::ProgressObserver,
        cancellation: &crate::CancellationToken,
    ) -> Result<crate::ValidationRunOutcome, ValidationRunError> {
        use crate::{ProgressEvent as Event, ValidationPhase as Phase};
        let started_at = self.clock.now();
        let resolved = self
            .schema_registry
            .resolve_with_hash(&request.schema_key)?;
        let schema = resolved.schema();
        let plan = preflight(&request, schema).map_err(ValidationRunError::InvalidRequest)?;
        let mut summary = crate::ValidationSummaryBuilder::new(
            schema.key().clone(),
            crate::ScopeSummary::Trace {
                trace_id: plan.trace().id().clone(),
            },
        );
        // Unsupported reference constraints already fail compilation. There is
        // no dataset preparation/no-op provider to misrepresent as successful.
        let audits = crate::InputAuditSession::new();
        progress.on_event(Event::RunStarted);
        progress.on_event(Event::PhaseStarted(Phase::LocalValidation));
        {
            let mut notify = |event| {
                progress.on_event(match event {
                    crate::TraceEvent::Started { trace_id } => Event::TraceStarted { trace_id },
                    crate::TraceEvent::RecordsProcessed { trace_id, count } => {
                        Event::RecordsProcessed { trace_id, count }
                    }
                    crate::TraceEvent::Completed { trace_id, count } => {
                        Event::TraceCompleted { trace_id, count }
                    }
                })
            };
            let mut control = crate::TraceControl {
                is_cancelled: &|| cancellation.is_cancelled(),
                on_event: &mut notify,
            };
            crate::TraceValidator::new(schema, plan.trace().id(), plan.options().newline_policy)?
                .validate_path(plan.path(), sink, &mut summary, &mut control, &audits)?;
        }
        progress.on_event(Event::PhaseStarted(Phase::FinalizingReport));
        if cancellation.is_cancelled() {
            summary.cancel();
        }
        let metadata = audits
            .finish(crate::RunMetadata {
                validation_mode: request.mode,
                validator_version: env!("CARGO_PKG_VERSION").into(),
                schema_hash: resolved.schema_hash().to_string(),
                reference_data_versions: std::collections::BTreeMap::new(),
                started_at,
                inputs: vec![],
            })
            .map_err(|e| ValidationRunError::Input(crate::InputError::Audit(e)))?;
        let summary = summary
            .finish(metadata)
            .map_err(ValidationRunError::Summary)?;
        sink.on_summary(&summary)
            .map_err(ValidationRunError::Report)?;
        progress.on_event(Event::RunCompleted);
        Ok(crate::ValidationRunOutcome {
            status: summary.run_status,
            summary,
        })
    }
}
