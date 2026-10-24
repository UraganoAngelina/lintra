//! Immutable payloads shared by trace order and target-key indexes, one run.
use crate::{
    ContextError, ContextStore, JoinId, JoinKey, JoinProjectionSide, ProjectedJoinValue,
    ProjectedRecord, RelatedRecordSet, TraceId,
};
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    sync::Arc,
};

type JoinIndex = HashMap<JoinId, HashMap<JoinKey, Vec<Arc<ProjectedRecord>>>>;

#[derive(Default)]
struct TraceContext {
    records: BTreeMap<u64, Arc<ProjectedRecord>>,
    targets: JoinIndex,
}

#[derive(Default)]
pub struct InMemoryContextStore {
    traces: HashMap<TraceId, TraceContext>,
}
impl InMemoryContextStore {
    pub fn new() -> Self {
        Self::default()
    }
}
impl fmt::Debug for InMemoryContextStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemoryContextStore")
            .field("trace_count", &self.traces.len())
            .field(
                "record_count",
                &self
                    .traces
                    .values()
                    .map(|trace| trace.records.len())
                    .sum::<usize>(),
            )
            .finish_non_exhaustive()
    }
}
impl ContextStore for InMemoryContextStore {
    fn insert(&mut self, record: ProjectedRecord) -> Result<(), ContextError> {
        // Every recoverable rejection precedes mutation of either index.
        if record.trace_id.0.is_empty() {
            return Err(ContextError::InvalidTraceId);
        }
        if record.line_number == 0 {
            return Err(ContextError::InvalidLineNumber);
        }
        if record
            .join_keys
            .keys()
            .any(|binding| binding.join_id.0.is_empty())
        {
            return Err(ContextError::InvalidJoinId);
        }
        if self
            .traces
            .get(&record.trace_id)
            .is_some_and(|trace| trace.records.contains_key(&record.line_number))
        {
            return Err(ContextError::DuplicatePosition {
                trace_id: record.trace_id,
                line_number: record.line_number,
            });
        }
        let record = Arc::new(record);
        let trace = self.traces.entry(record.trace_id.clone()).or_default();
        for (binding, value) in &record.join_keys {
            if binding.side == JoinProjectionSide::Target
                && let ProjectedJoinValue::Available(key) = value
            {
                let matches = trace
                    .targets
                    .entry(binding.join_id.clone())
                    .or_default()
                    .entry(key.clone())
                    .or_default();
                // Normal pass-1 order appends; explicit out-of-order inserts
                // still produce the same physical-line order on every host.
                let position =
                    matches.partition_point(|prior| prior.line_number < record.line_number);
                matches.insert(position, Arc::clone(&record));
            }
        }
        trace.records.insert(record.line_number, record);
        Ok(())
    }

    fn find(
        &self,
        trace_id: &TraceId,
        join_id: &JoinId,
        key: &JoinKey,
    ) -> Result<RelatedRecordSet<'_>, ContextError> {
        if trace_id.0.is_empty() {
            return Err(ContextError::InvalidTraceId);
        }
        if join_id.0.is_empty() {
            return Err(ContextError::InvalidJoinId);
        }
        let records = self
            .traces
            .get(trace_id)
            .and_then(|trace| trace.targets.get(join_id))
            .and_then(|index| index.get(key))
            .map_or(&[][..], Vec::as_slice);
        Ok(RelatedRecordSet::new(records))
    }

    fn iter_trace<'a>(
        &'a self,
        trace_id: &TraceId,
    ) -> Box<dyn Iterator<Item = &'a ProjectedRecord> + 'a> {
        match self.traces.get(trace_id) {
            Some(trace) => Box::new(trace.records.values().map(Arc::as_ref)),
            None => Box::new(std::iter::empty()),
        }
    }
}
