//! LocalOnly and Full orchestration and request preflight with typed technical failures.
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
    MissingRequiredTrace(TraceId),
    DuplicateTrace(TraceId),
    Input(InputPlanError),
}
impl fmt::Display for InvalidRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SchemaKeyMismatch { .. } => "request and compiled schema keys differ",
            Self::UnsupportedMode(_) => "Full validation requires Flow scope",
            Self::UnsupportedFlowScope => "Flow scope is not available until P13",
            Self::UnsupportedMaxErrors { .. } => "max_errors must be None in the current engine",
            Self::MissingRequiredTrace(_) => "Full bundle lacks a required trace",
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

/// Legacy trace-only preflight plan. Full uses preflight_flow_inputs. Borrowed
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
/// This compatibility entry point prepares Trace/LocalOnly only; Full service
/// uses the bundle preparation port. No input is opened by either preflight.
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

/// Technical run failures with concrete module payloads. ReferenceData retains
/// real preparation causes; no normative ValidationError is wrapped.
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
    ReferenceData(crate::ReferencePreparationError),
    Flow(crate::FlowValidationError),
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
            Self::Flow(_) => "run Full execution failed",
            Self::ReferenceData(_) => "run reference preparation failed",
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
            Self::ReferenceData(e) => e,
            Self::Flow(e) => e,
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
            e @ (Trace::PreviousFailure | Trace::Collection(_)) => Self::Trace(e),
        }
    }
}

impl From<crate::FlowValidationError> for ValidationRunError {
    fn from(error: crate::FlowValidationError) -> Self {
        use crate::FlowValidationError as Flow;
        match error {
            Flow::Trace(error) => error.into(),
            Flow::Summary(error) => Self::Summary(error),
            Flow::Cross(crate::CrossRecordValidationError::Report(error)) => Self::Report(error),
            Flow::MissingTrace(trace) => {
                Self::InvalidRequest(InvalidRequestError::MissingRequiredTrace(trace))
            }
            Flow::DuplicateTrace(trace) => {
                Self::InvalidRequest(InvalidRequestError::DuplicateTrace(trace))
            }
            Flow::UnknownTrace(trace) => Self::InvalidRequest(InvalidRequestError::Input(
                InputPlanError::TraceNotFound(trace),
            )),
            Flow::EmptyPath(trace_id) => {
                Self::InvalidRequest(InvalidRequestError::Input(InputPlanError::EmptyPath {
                    trace_id,
                }))
            }
            error => Self::Flow(error),
        }
    }
}

/// Application orchestrator with explicit schema/reference DI and per-run context.
pub struct ValidationService {
    schema_registry: std::sync::Arc<crate::SchemaRegistry>,
    clock: std::sync::Arc<dyn crate::Clock>,
    reference_data: Option<(
        std::sync::Arc<dyn crate::ReferenceDataPreparer>,
        crate::DatasetPreparationPolicy,
    )>,
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
/// Reference wiring uses build_local_service_with_reference_data; these historical
/// factory names remain compatible and now support Full Flow requests too.
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
            reference_data: None,
        }
    }
    /// Explicit preparation/policy DI; old constructors retain their behavior
    /// for flows without references and fail clearly when preparation is absent.
    pub fn with_reference_data(
        mut self,
        preparer: std::sync::Arc<dyn crate::ReferenceDataPreparer>,
        policy: crate::DatasetPreparationPolicy,
    ) -> Self {
        self.reference_data = Some((preparer, policy));
        self
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
        self.validate_with_parameters(
            request,
            std::collections::BTreeMap::new(),
            sink,
            progress,
            cancellation,
        )
    }
    pub fn validate_with_parameters(
        &self,
        request: ValidationRequest,
        parameters: std::collections::BTreeMap<String, String>,
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
        if request.mode == ValidationMode::Full {
            return self.validate_full(
                request,
                parameters,
                super::full::FullServicePorts {
                    sink,
                    progress,
                    cancellation,
                    started_at,
                    resolved: &resolved,
                },
            );
        }
        if !parameters.is_empty() {
            return Err(ValidationRunError::Flow(
                crate::FlowValidationError::Inputs(crate::RelatedContextError::InvalidParameter),
            ));
        }
        let plan = preflight(&request, schema).map_err(ValidationRunError::InvalidRequest)?;
        let mut summary = crate::ValidationSummaryBuilder::new(
            schema.key().clone(),
            crate::ScopeSummary::Trace {
                trace_id: plan.trace().id().clone(),
            },
        );
        let prepared = if schema.required_reference_data().is_empty() {
            None
        } else {
            let (preparer, policy) =
                self.reference_data
                    .as_ref()
                    .ok_or(ValidationRunError::ReferenceData(
                        crate::ReferencePreparationError::NotConfigured,
                    ))?;
            Some(
                preparer
                    .prepare(schema.required_reference_data(), *policy)
                    .map_err(ValidationRunError::ReferenceData)?,
            )
        };
        let mut reference_audit = match &prepared {
            Some(prepared) => Some(
                super::reference_audit::ReferenceAuditCollector::new(
                    schema,
                    plan.trace(),
                    prepared,
                    self.reference_data
                        .as_ref()
                        .ok_or(ValidationRunError::ReferenceData(
                            crate::ReferencePreparationError::NotConfigured,
                        ))?
                        .1,
                )
                .map_err(ValidationRunError::ReferenceData)?,
            ),
            None => None,
        };
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
            let mut validator = match (&prepared, &mut reference_audit) {
                (Some(prepared), Some(observer)) => crate::TraceValidator::with_reference_data(
                    schema,
                    plan.trace().id(),
                    plan.options().newline_policy,
                    prepared,
                    observer,
                )?,
                (None, None) => crate::TraceValidator::new(
                    schema,
                    plan.trace().id(),
                    plan.options().newline_policy,
                )?,
                _ => {
                    return Err(ValidationRunError::ReferenceData(
                        crate::ReferencePreparationError::InconsistentEvidence,
                    ));
                }
            };
            validator.validate_path(plan.path(), sink, &mut summary, &mut control, &audits)?;
        }
        progress.on_event(Event::PhaseStarted(Phase::FinalizingReport));
        if cancellation.is_cancelled() {
            summary.cancel();
        }
        let reference_audit = reference_audit.map(|collector| collector.into_audit());
        let reference_versions = reference_audit
            .as_ref()
            .map(|audit| audit.verified_versions())
            .unwrap_or_default();
        let metadata = audits
            .finish(crate::RunMetadata {
                validation_mode: request.mode,
                validator_version: env!("CARGO_PKG_VERSION").into(),
                schema_hash: resolved.schema_hash().to_string(),
                reference_data_versions: reference_versions,
                started_at,
                inputs: vec![],
            })
            .map_err(|e| ValidationRunError::Input(crate::InputError::Audit(e)))?;
        let summary = match reference_audit {
            Some(audit) => summary.finish_with_reference_data(metadata, audit),
            None => summary.finish(metadata),
        }
        .map_err(ValidationRunError::Summary)?;
        sink.on_summary(&summary)
            .map_err(ValidationRunError::Report)?;
        progress.on_event(Event::RunCompleted);
        Ok(crate::ValidationRunOutcome {
            status: summary.run_status,
            summary,
        })
    }
    fn validate_full(
        &self,
        request: ValidationRequest,
        supplied: std::collections::BTreeMap<String, String>,
        ports: super::full::FullServicePorts<'_>,
    ) -> Result<crate::ValidationRunOutcome, ValidationRunError> {
        use crate::*;
        let schema = ports.resolved.schema();
        if &request.schema_key != schema.key() {
            return Err(ValidationRunError::InvalidRequest(
                InvalidRequestError::SchemaKeyMismatch {
                    requested: request.schema_key,
                    loaded: schema.key().clone(),
                },
            ));
        }
        if let Some(limit) = request.options.max_errors {
            return Err(ValidationRunError::InvalidRequest(
                InvalidRequestError::UnsupportedMaxErrors { limit },
            ));
        }
        let ValidationScope::Flow(input) = &request.scope else {
            return Err(ValidationRunError::InvalidRequest(
                InvalidRequestError::UnsupportedMode(ValidationMode::Full),
            ));
        };
        if input.traces.is_empty() {
            return Err(ValidationRunError::InvalidRequest(
                InvalidRequestError::EmptyFlow,
            ));
        }
        let inputs = input
            .traces
            .iter()
            .map(|input| {
                let InputSource::Path(path) = &input.source;
                FlowTraceInput {
                    trace_id: &input.trace_id,
                    path,
                }
            })
            .collect::<Vec<_>>();
        let parameters = super::full::bind_parameters(schema, &supplied)?;
        let traces = preflight_flow_inputs(schema, &inputs, &parameters)?;
        let prepared = if schema.required_reference_data().is_empty() {
            None
        } else {
            let (preparer, policy) =
                self.reference_data
                    .as_ref()
                    .ok_or(ValidationRunError::ReferenceData(
                        ReferencePreparationError::NotConfigured,
                    ))?;
            Some(
                preparer
                    .prepare(schema.required_reference_data(), *policy)
                    .map_err(ValidationRunError::ReferenceData)?,
            )
        };
        let mut references = match &prepared {
            Some(prepared) => Some(
                super::full::FlowReferences::new(
                    schema,
                    &traces,
                    prepared,
                    self.reference_data
                        .as_ref()
                        .ok_or(ValidationRunError::ReferenceData(
                            ReferencePreparationError::NotConfigured,
                        ))?
                        .1,
                )
                .map_err(ValidationRunError::ReferenceData)?,
            ),
            None => None,
        };
        let mut summary = ValidationSummaryBuilder::new(
            schema.key().clone(),
            ScopeSummary::Flow { trace_ids: traces },
        );
        let audits = InputAuditSession::new();
        ports.progress.on_event(ProgressEvent::RunStarted);
        ports.progress.on_event(ProgressEvent::PhaseStarted(
            ValidationPhase::LocalValidation,
        ));
        let artifacts = {
            let mut notify = |event| match event {
                FlowEvent::Trace(event) => ports.progress.on_event(match event {
                    TraceEvent::Started { trace_id } => ProgressEvent::TraceStarted { trace_id },
                    TraceEvent::RecordsProcessed { trace_id, count } => {
                        ProgressEvent::RecordsProcessed { trace_id, count }
                    }
                    TraceEvent::Completed { trace_id, count } => {
                        ProgressEvent::TraceCompleted { trace_id, count }
                    }
                }),
                FlowEvent::CrossStarted => ports.progress.on_event(ProgressEvent::PhaseStarted(
                    ValidationPhase::CrossRecordValidation,
                )),
                FlowEvent::CrossRecordsProcessed { count } => ports
                    .progress
                    .on_event(ProgressEvent::CrossRecordsProcessed { count }),
                FlowEvent::Completed | FlowEvent::Cancelled => {}
            };
            let mut control = FlowControl {
                is_cancelled: &|| ports.cancellation.is_cancelled(),
                on_event: &mut notify,
            };
            let validator = match &prepared {
                Some(prepared) => FlowValidator::with_reference_data(
                    schema,
                    prepared,
                    request.options.newline_policy,
                )?,
                None => FlowValidator::new(schema, request.options.newline_policy)?,
            };
            validator.validate(
                &inputs,
                &parameters,
                FlowExecutionPorts {
                    sink: ports.sink,
                    summary: &mut summary,
                    audits: &audits,
                    control: &mut control,
                    reference_observers: references
                        .as_mut()
                        .map(|r| r as &mut dyn FlowReferenceObservers),
                    cross_observer: None,
                },
            )?
        };
        ports.progress.on_event(ProgressEvent::PhaseStarted(
            ValidationPhase::FinalizingReport,
        ));
        if ports.cancellation.is_cancelled() {
            summary.cancel();
        }
        let full = super::full::into_summary(artifacts, &parameters)?;
        let reference_audit = references
            .map(|r| r.into_audit())
            .transpose()
            .map_err(ValidationRunError::ReferenceData)?;
        let versions = reference_audit
            .as_ref()
            .map(|audit| audit.verified_versions())
            .unwrap_or_default();
        let metadata = audits
            .finish(RunMetadata {
                validation_mode: request.mode,
                validator_version: env!("CARGO_PKG_VERSION").into(),
                schema_hash: ports.resolved.schema_hash().to_string(),
                reference_data_versions: versions,
                started_at: ports.started_at,
                inputs: vec![],
            })
            .map_err(|e| ValidationRunError::Input(InputError::Audit(e)))?;
        let summary = summary
            .finish_full(metadata, full, reference_audit)
            .map_err(ValidationRunError::Summary)?;
        ports
            .sink
            .on_summary(&summary)
            .map_err(ValidationRunError::Report)?;
        ports.progress.on_event(ProgressEvent::RunCompleted);
        Ok(ValidationRunOutcome {
            status: summary.run_status,
            summary,
        })
    }
}

/// Explicit two-root LocalOnly filesystem wiring, with no fallback or discovery.
pub fn build_local_service_with_reference_data(
    schema_root: std::path::PathBuf,
    reference_root: std::path::PathBuf,
    encoding: crate::RawResolvedEncoding,
    policy: crate::DatasetPreparationPolicy,
) -> Result<ValidationService, ValidationRunError> {
    let service = build_default_local_service(schema_root, encoding)
        .map_err(ValidationRunError::SchemaLoad)?;
    let preparer =
        crate::FileSystemReferenceDataPreparer::new(reference_root).map_err(|source| {
            ValidationRunError::ReferenceData(crate::ReferencePreparationError::Provider(Box::new(
                source,
            )))
        })?;
    Ok(service.with_reference_data(std::sync::Arc::new(preparer), policy))
}
