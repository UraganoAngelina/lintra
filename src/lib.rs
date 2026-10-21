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
    AlignmentRule, BasePresence, CompiledFieldSpec, CompiledFlowSchema, CompiledRecordSchema,
    ContentBoundaryPolicy, EncodingSpec, FieldType, LexicalPlan, MissingValuePolicy,
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
pub use input::source::InputSource;
