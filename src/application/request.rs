//! Requests describe intent; preflight checks support before any input is opened.
use crate::{InputSource, NewlinePolicy, SchemaKey, TraceId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceValidationInput {
    pub trace_id: TraceId,
    pub source: InputSource,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowValidationInput {
    pub traces: Vec<TraceValidationInput>,
}
/// Scope is independent of mode. Flow execution remains a P13 capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationScope {
    Trace(TraceValidationInput),
    Flow(FlowValidationInput),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    LocalOnly,
    Full,
}
/// The sole supported length policy stops malformed records before slicing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordLengthPolicy {
    StrictSkipRecord,
}
/// Explicit options; no implicit newline/BOM policy or operational limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationOptions {
    pub record_length_policy: RecordLengthPolicy,
    pub newline_policy: NewlinePolicy,
    pub max_errors: Option<u64>,
}
/// DET §25's key/scope/mode/options structure is preserved verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationRequest {
    pub schema_key: SchemaKey,
    pub scope: ValidationScope,
    pub mode: ValidationMode,
    pub options: ValidationOptions,
}
