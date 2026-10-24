//! Per-run context port. Empty membership alone says nothing about input
//! availability; the caller applies schema cardinality and missing policies.
use crate::{JoinId, JoinKey, ProjectedRecord, TraceId};
use std::{error::Error, fmt, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextError {
    InvalidTraceId,
    InvalidJoinId,
    InvalidLineNumber,
    DuplicatePosition { trace_id: TraceId, line_number: u64 },
}
impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidTraceId => "context trace ID must not be empty",
            Self::InvalidJoinId => "context join ID must not be empty",
            Self::InvalidLineNumber => "context record needs a one-based line number",
            Self::DuplicatePosition { .. } => "context already contains this trace and line",
        })
    }
}
impl Error for ContextError {}

/// Borrowed view over immutable shared records. No payload copy or temporary
/// match vector is needed by find; all matches remain visible to the evaluator.
pub struct RelatedRecordSet<'a> {
    records: &'a [Arc<ProjectedRecord>],
}
impl<'a> RelatedRecordSet<'a> {
    pub fn new(records: &'a [Arc<ProjectedRecord>]) -> Self {
        Self { records }
    }
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
    pub fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = &'a ProjectedRecord> + ExactSizeIterator + '_ {
        self.records.iter().map(Arc::as_ref)
    }
}
impl fmt::Debug for RelatedRecordSet<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RelatedRecordSet")
            .field("len", &self.len())
            .finish_non_exhaustive()
    }
}

/// A new store belongs to one validation run in its preflighted namespace.
/// No persistence, cross-run uniqueness or normative cardinality is implied.
pub trait ContextStore {
    fn insert(&mut self, record: ProjectedRecord) -> Result<(), ContextError>;
    /// Search TARGET bindings, returning every match in ascending source line.
    fn find(
        &self,
        trace_id: &TraceId,
        join_id: &JoinId,
        key: &JoinKey,
    ) -> Result<RelatedRecordSet<'_>, ContextError>;
    fn iter_trace<'a>(
        &'a self,
        trace_id: &TraceId,
    ) -> Box<dyn Iterator<Item = &'a ProjectedRecord> + 'a>;
}
