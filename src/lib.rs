//! Generic fixed-width validation engine.
//!
//! The library can be linked by an external consumer:
//!
//! ```
//! use lintra as _;
//! ```
//!
//! Engine internals are private (DETAILED_DESIGN §94):
//!
//! ```compile_fail,E0603
//! use lintra::engine;
//! ```
//!
//! Core persistence internals are private as well:
//!
//! ```compile_fail,E0603
//! use lintra::persistence;
//! ```
//!
//! Domain identifiers cannot be substituted for one another:
//!
//! ```compile_fail,E0308
//! use lintra::{FlowId, TraceId};
//! let flow: FlowId = TraceId("EXAMPLE".to_owned());
//! ```
//!
//! Byte ranges use zero-based byte offsets, without decoding or copying:
//!
//! ```
//! use lintra::ByteRange;
//! let range = ByteRange::new(1, 2).unwrap();
//! assert_eq!(range.slice(b"ABCD"), Some(&b"BC"[..]));
//! ```

//!
//! Compiled schemas cannot be forged from JSON, defaulted or constructed directly:
//!
//! ```compile_fail,E0277
//! use lintra::CompiledFlowSchema;
//! let schema: CompiledFlowSchema = serde_json::from_str("{}").unwrap();
//! ```
//!
//! ```compile_fail,E0451
//! use lintra::CompiledFlowSchema;
//! fn forge(schema: CompiledFlowSchema) -> CompiledFlowSchema {
//!     CompiledFlowSchema { ..schema }
//! }
//! ```
//!
//! ```compile_fail,E0599
//! use lintra::CompiledFlowSchema;
//! let schema = CompiledFlowSchema::default();
//! ```
//!
//! ```compile_fail,E0451
//! use lintra::CompiledRecordSchema;
//! fn forge(schema: CompiledRecordSchema) -> CompiledRecordSchema {
//!     CompiledRecordSchema { ..schema }
//! }
//! ```
//!
//! ```compile_fail,E0451
//! use lintra::CompiledFieldSpec;
//! fn forge(field: CompiledFieldSpec) -> CompiledFieldSpec {
//!     CompiledFieldSpec { ..field }
//! }
//! ```

//! Decimal metadata is read-only outside compilation:
//!
//! ```compile_fail,E0624
//! use lintra::{DecimalSpec, SignPolicy};
//! let spec = DecimalSpec::new(7, 2, b'.', SignPolicy::Unsigned);
//! ```

mod application;
mod context;
mod domain;
mod engine;
mod input;
mod persistence;
mod reference_data;
mod report;
mod schema;
mod validators;

// Explicit domain primitives; module internals remain private.
pub use domain::{
    ByteRange, ByteRangeError, DatasetId, DatasetVersion, FieldId, FlowId, JoinId, SchemaVersion,
    TraceId,
};

pub use domain::{
    DiagnosticContextItem, FieldDiagnostic, InvalidDiagnostic, InvalidValidationCode,
    ValidationCode, ValidationError,
};

pub use domain::{
    RecordIdentity, RunMetadata, RunStatus, SchemaKey, ScopeSummary, ValidationRunOutcome,
    ValidationSummary,
};

pub use schema::raw::{
    RawEncodingSpec, RawFieldSpec, RawFlowSchema, RawIdentitySpec, RawJoinSpec, RawRecordSchema,
    RawReferenceDataDependency, RawResolvedEncoding, RawRule,
};

pub use schema::compiled::{
    AlignmentRule, BasePresence, Capability, CompiledFieldSpec, CompiledFlowSchema,
    CompiledRecordSchema, ContentBoundaryPolicy, DecimalSpec, EncodingSpec, FieldType, LexicalPlan,
    MissingValuePolicy, SignPolicy,
};

pub use schema::repository::{
    FileSystemSchemaRepository, InMemorySchemaRepository, SchemaKeyComponent, SchemaKeyError,
    SchemaLoadError, SchemaRepository,
};

pub use schema::linter::{SchemaIssue, SchemaIssueCode, SchemaIssueSeverity, lint_raw_schema};

pub use schema::{SchemaCompilationError, SchemaCompiler};

pub use schema::compatibility::{EngineCapabilities, SchemaFormatVersion};

pub use schema::registry::{SchemaRegistry, SchemaResolutionError};

pub use schema::compiled::{CompiledConstraint, ValueTarget};
pub use validators::alignment::{LogicalValueError, logical_value};
pub use validators::datatype::{DatatypeError, DatatypeValidationResult, validate_datatype};
pub use validators::domain::validate_allowed_values;
pub use validators::presence::{PresenceValidationResult, is_missing, validate_presence};

pub use engine::error_factory::ErrorFactory;
pub use engine::field_pipeline::{FieldPipeline, FieldPipelineError, FieldPipelineResult};
pub use engine::identity_extractor::{IdentityExtractionError, IdentityExtractor};
pub use validators::FieldViolation;

pub use report::{
    composite::CompositeSink,
    sink::{ReportError, ValidationSink, VecSink},
};

pub use report::atomic::{
    AtomicOutputOutcome, AtomicReportFile, OutputOperation, PartialOutputPolicy, ReportCompletion,
    write_atomic_report,
};
pub use report::columns::{ERROR_COLUMNS, ErrorColumn};
pub use report::csv::CsvSink;
pub use report::jsonl::JsonlSink;

pub use domain::{
    InputAudit, InputAuditCompletion, InvalidInputAudit, Sha256Digest, SummaryBuildError,
    SummaryCounter, ValidationSummaryBuilder,
};
pub use input::audit::{AuditedReader, InputAuditError, InputAuditSession};
pub use report::summary::write_summary;

pub use input::{
    error::{BomKind, InputError},
    reader::{NewlinePolicy, RecordReader},
    record::{FieldSlice, RawRecord, RecordView},
    source::{BomCheckedReader, BomPolicy, preflight_bom},
};

pub use input::encoding::{DiagnosticValue, render_diagnostic_value};

pub use engine::record_validator::{
    LocalRecordValidationArtifacts, RecordValidationError, RecordValidator, UnsupportedLocalFeature,
};

pub use application::{
    cancellation::CancellationToken,
    progress::{ProgressEvent, ProgressObserver, ValidationPhase},
};
pub use application::{
    request::{
        FlowValidationInput, RecordLengthPolicy, TraceValidationInput, ValidationMode,
        ValidationOptions, ValidationRequest, ValidationScope,
    },
    service::{InvalidRequestError, PreflightPlan, preflight},
};
pub use engine::preflight::InputPlanError;
pub use engine::trace_validator::{
    TraceCompletion, TraceControl, TraceEvent, TraceValidationError, TraceValidator,
    validate_reader,
};
pub use input::source::InputSource;
pub use schema::canonical::{CanonicalSchemaError, canonical_schema_bytes, schema_fingerprint};
pub use schema::diff::{
    SchemaChange, SchemaChangeKind, SchemaDiff, SchemaDiffError, SchemaDiffRevision,
    SchemaDiffSide, diff_schemas,
};
pub use schema::registry::ResolvedSchema;

pub use application::clock::{Clock, SystemClock};
pub use application::service::{
    ValidationRunError, ValidationService, build_default_local_service,
};
pub use report::summary::write_run_summary;

// Text summary adapter for CLI/UI consumers; report internals stay private.
pub use report::human::HumanReadableSink;

pub use engine::expression_evaluator::{TruthValue, UnknownReason};

pub use engine::expression_evaluator::{
    EvaluationContext, ExpressionEvaluator, FieldValueResolver, LocalEvaluationContext,
    ResolvedFieldValue,
};

pub use schema::compiled::{CompiledExpr, ResolvedFieldRef};

pub use engine::presence::{PresenceState, resolve_presence};

pub use engine::rule_validator::{RuleValidationResult, validate_local_rule};
pub use schema::compiled::{
    ComparisonOperator, CompiledAssertion, CompiledOperand, CompiledRule, CompiledValue,
    DateLiteral, ValueType,
};

pub use engine::rule_diagnostics::RuleDiagnosticError;

pub use reference_data::{
    dataset::{DatasetKey, LookupSet, PreparedReferenceData, prepare_reference_data},
    evidence::{
        DatasetAnomaly, DatasetAvailability, DatasetEvidence, DatasetFileTypeIssue,
        DatasetIoOperation, DatasetObservation, DatasetPreparationPolicy, MembershipFormatIssue,
        MembershipProperty, PreparedReferenceEvidence,
    },
    filesystem::{
        FileSystemReferenceDataProvider, FilesystemBoundaryViolation, FilesystemReferenceError,
        ReferenceBindingIssue,
    },
    in_memory::InMemoryReferenceDataProvider,
    normalization::LookupNormalization,
    provider::{ReferenceDataError, ReferenceDataProvider},
};

pub use reference_data::membership::{
    MembershipEvaluation, MembershipPlanError, evaluate_membership,
};
pub use schema::compiled::ReferenceDependency;
pub use validators::reference_data::{
    ReferenceConstraintEvaluation, ReferenceEvaluationError, validate_reference_constraint,
};

pub use engine::derived_evaluator::{DerivedValue, DerivedValueEvaluator};
pub use schema::compiled::{DateFormat, DerivedValuePlan, LeapDayAnniversary};
pub use schema::compiled::{IntegerLiteral, InvalidIntegerLiteral};

pub use engine::record_validator::{ReferenceRecordValidationArtifacts, ReferenceRecordValidator};
pub use engine::reference_execution::{
    ReferenceConclusion, ReferenceControl, ReferenceEvaluationObserver, ReferenceExecutionError,
};

pub use context::join_key::{JoinKey, JoinKeyError, build_join_key};
pub use schema::compiled::{
    CompiledJoinSpec, JoinCardinality, JoinScope, RelatedDiagnostic, RelatedPolicy,
};
pub use schema::compiled::{ConsensusPlan, ParameterDefinition};

pub use domain::{
    ReferenceControlSummary, ReferenceDatasetAudit, ReferenceDatasetDiagnostic,
    ReferenceEvaluationCounts, ReferenceIoKind, ReferenceSourceUsage, ReferenceValidationSummary,
    ReferenceWarningKind, ValidationCompleteness,
};

pub use reference_data::preparer::{
    FileSystemReferenceDataPreparer, ReferenceDataPreparer, ReferencePreparationError,
};

pub use application::service::build_local_service_with_reference_data;

pub use report::xlsx::{XLSX_SHEET_ROWS, XlsxReportError, XlsxReportWriter};

pub use context::projected_record::{
    ProjectedJoinValue, ProjectedRecord, ProjectionError, project_record,
};
pub use schema::compiled::{
    ContextProjectionPlan, JoinProjectionKey, JoinProjectionPlan, JoinProjectionSide,
};

pub use context::coverage::{TraceContextCoverage, TraceCoverageEvidence};
pub use context::in_memory::InMemoryContextStore;
pub use context::store::{ContextError, ContextStore, RelatedRecordSet};

pub use engine::projected_context::{
    CoveredCrossRecordInputs, CrossRecordInputs, JoinParameterValue, ProjectedEvaluationContext,
    RelatedContextError, RelatedLookup, RelatedResolution,
};

pub use engine::flow_validator::{
    CrossRecordEvaluationObserver, CrossRecordPassSummary, CrossRecordValidationError,
    CrossRuleOutcome, validate_cross_record_rules,
};
pub use schema::expression::{FieldReferenceError, resolve_field_reference};
pub use schema::raw::{RawRef, RawRefExternal, RawRefLocal};

pub use domain::ValidationCounters;

pub use engine::flow_execution::{
    CrossControlEvidence, CrossEvidenceCause, FlowCompletion, FlowControl, FlowEvent,
    FlowExecutionPorts, FlowReferenceObservers, FlowTraceInput, FlowValidationArtifacts,
    FlowValidationError, FlowValidator, NonDefinitiveCrossResult, RecordKeyEvidence,
};

pub use domain::{
    ContextCoverage, CrossCauseKind, CrossControlSummary, CrossEvaluationCause,
    CrossUncertaintyReason, FlowTraceEvidence, FullValidationSummary, RunParameterAudit,
};

pub use engine::flow_execution::preflight_flow_inputs;
