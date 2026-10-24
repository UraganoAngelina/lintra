mod byte_range;
mod errors;
mod ids;

pub use byte_range::{ByteRange, ByteRangeError};
pub use ids::{DatasetId, DatasetVersion, FieldId, FlowId, JoinId, SchemaVersion, TraceId};

pub use errors::{
    DiagnosticContextItem, DiagnosticValue, FieldDiagnostic, InvalidDiagnostic,
    InvalidValidationCode, ValidationCode, ValidationError,
};

mod identity;
mod input_audit;
mod summary;
mod summary_builder;
pub use summary_builder::{
    SummaryBuildError, SummaryCounter, ValidationCounters, ValidationSummaryBuilder,
};

pub use input_audit::{InputAudit, InputAuditCompletion, InvalidInputAudit, Sha256Digest};

pub use identity::RecordIdentity;
pub use summary::{
    RunMetadata, RunStatus, SchemaKey, ScopeSummary, ValidationMode, ValidationRunOutcome,
    ValidationSummary,
};

pub(crate) mod reference;
pub use reference::*;
