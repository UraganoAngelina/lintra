//! Approved monthly history, independent of flow names and SQL adapters.
//! Only compiler-bound policies enter this producer. It holds protected tokens,
//! never clinical rows; byte identity is supplied by the existing HMAC producer.
use super::session::{CrossRunBinding, cross_run_binding};
use crate::{
    schema::{compiled::CrossRunTracePlan, raw::RawCrossRunMode},
    *,
};
use std::collections::{BTreeMap, HashMap, HashSet};

pub(super) struct CrossRunEvaluator<'a> {
    profile: CompiledPersistenceProfile,
    repository: &'a dyn PersistentContextRepository,
    protector: &'a IdentityProtector,
    namespace: PersistenceNamespace,
    generation: Option<SnapshotRevision>,
    traces: BTreeMap<String, TraceState>,
    errors: Vec<ValidationError>,
    rejected: HashSet<(TraceId, u64)>,
    replay: bool,
}
struct TraceState {
    plan: CrossRunTracePlan,
    baseline: HashMap<PersistentRecordIdentity, StoredIdentity>,
    seen: HashSet<PersistentRecordIdentity>,
    summary: CrossRunTraceSummary,
}
impl<'a> CrossRunEvaluator<'a> {
    pub(super) fn prepare(
        profile: &CompiledPersistenceProfile,
        binding: CrossRunBinding,
        repository: &'a dyn PersistentContextRepository,
        protector: &'a IdentityProtector,
        cancellation: &CancellationToken,
    ) -> Result<Self, HistoricalValidationError> {
        let plan = profile
            .policy()
            .and_then(|p| p.cross_run_validation.as_ref())
            .ok_or(HistoricalValidationError::InvalidRequest)?;
        let namespace = binding.scope.clone();
        let generation = repository.current_generation(&namespace)?;
        let mut result = Self {
            profile: profile.clone(),
            repository,
            protector,
            namespace,
            generation,
            traces: BTreeMap::new(),
            errors: Vec::new(),
            rejected: HashSet::new(),
            replay: false,
        };
        for trace in &plan.traces {
            let id = &profile.trace_ids()[trace.trace_index];
            result.traces.insert(
                id.0.clone(),
                TraceState {
                    plan: trace.clone(),
                    baseline: HashMap::new(),
                    seen: HashSet::new(),
                    summary: CrossRunTraceSummary {
                        trace_id: id.clone(),
                        carry_forward_identical: 0,
                        new_records: 0,
                        historical_key_content_mismatch: 0,
                        reused_records: 0,
                        missing_previous_records: 0,
                        indeterminate: false,
                        unknown_reason: None,
                    },
                },
            );
        }
        // Metadata is bounded by the existing 128-source cursor. Select one
        // latest PRESENT baseline per trace/year, never union cumulative files.
        let mut selected = BTreeMap::<String, (SnapshotRevision, SourceSubmission)>::new();
        let mut replay_source = None;
        let mut after: Option<SnapshotRevision> = None;
        let mut saw_flow = false;
        let mut future = false;
        loop {
            check_cancel(cancellation)?;
            result.check_generation()?;
            let page = repository.load_submission_page(&result.namespace, after)?;
            if page.len() > 128 {
                return Err(PersistenceError::CorruptStorage.into());
            }
            if page.is_empty() {
                break;
            }
            for (revision, source) in page {
                check_cancel(cancellation)?;
                let expected = after
                    .map(|r| r.next())
                    .transpose()
                    .map_err(|_| PersistenceError::RevisionOverflow)?
                    .unwrap_or(SnapshotRevision::new(1));
                if revision != expected
                    || Some(revision) > generation
                    || source.namespace() != &result.namespace
                {
                    return Err(PersistenceError::CorruptStorage.into());
                }
                after = Some(revision);
                protector
                    .verify_submission(&source)
                    .map_err(HistoricalValidationError::Protection)?;
                if source.schema_key().flow_id != profile.schema_key().flow_id {
                    continue;
                }
                saw_flow = true;
                if source.state() != SessionState::ApprovedImmutable {
                    result.unknown_all("unapproved_history");
                    continue;
                }
                let Some(previous) = cross_run_binding(&source)? else {
                    result.unknown_all("unqualified_legacy_period");
                    continue;
                };
                if source.schema_key() != profile.schema_key()
                    || source.schema_hash() != profile.schema_hash()
                    || previous.year.0 != binding.year.0
                    || previous.month.0 != binding.month.0
                {
                    result.unknown_all("incompatible_history_definition");
                    continue;
                }
                if source.submission_id() == binding.submission_id {
                    if previous != binding {
                        return Err(HistoricalValidationError::Approval(
                            SessionApprovalError::Rejected(
                                SessionRejection::ApprovedSessionImmutable,
                            ),
                        ));
                    }
                    replay_source = Some((revision, source.clone()));
                }
                if (previous.year.1, previous.month.1) > (binding.year.1, binding.month.1) {
                    // Exact replay is resolved after this scan; ordinary new
                    // submissions must not validate against a future baseline.
                    future = true;
                    continue;
                }
                if previous.year.1 == binding.year.1 {
                    for trace in source.traces() {
                        if result.traces.contains_key(&trace.trace_id.0)
                            && matches!(trace.presence, TracePresence::Present { .. })
                        {
                            selected.insert(trace.trace_id.0.clone(), (revision, source.clone()));
                        }
                    }
                }
            }
        }
        if after != generation {
            return Err(PersistenceError::CorruptStorage.into());
        }
        result.check_generation()?;
        if let Some((revision, source)) = replay_source {
            result.replay = true;
            selected.clear();
            for trace in source.traces() {
                if result.traces.contains_key(&trace.trace_id.0)
                    && matches!(trace.presence, TracePresence::Present { .. })
                {
                    selected.insert(trace.trace_id.0.clone(), (revision, source.clone()));
                }
            }
        } else if future {
            result.unknown_all("submission_precedes_approved_history");
        }
        let mut loaded = BTreeMap::new();
        for (id, (revision, source)) in selected {
            check_cancel(cancellation)?;
            if !loaded.contains_key(&revision.value()) {
                let snapshot = repository
                    .load_snapshot(&result.namespace, revision)?
                    .ok_or(PersistenceError::CorruptStorage)?;
                if snapshot.revision() != revision || snapshot.source() != &source {
                    return Err(PersistenceError::CorruptStorage.into());
                }
                result.verify_snapshot(&snapshot, cancellation)?;
                loaded.insert(revision.value(), snapshot);
            }
            let snapshot = &loaded[&revision.value()];
            let trace = result
                .traces
                .get_mut(&id)
                .ok_or(HistoricalValidationError::InvalidRequest)?;
            for row in snapshot
                .identities()
                .iter()
                .filter(|r| r.identity().trace_id().0 == id)
            {
                trace.baseline.insert(row.identity().clone(), row.clone());
            }
        }
        if !result.replay && binding.month.1 != 1 {
            for trace in result.traces.values_mut() {
                if trace.plan.mode == RawCrossRunMode::CumulativeIdentical
                    && trace.baseline.is_empty()
                {
                    // A qualified PRESENT empty baseline is valid, unlike an
                    // absent baseline. Find its actual metadata, not key count.
                    let present = loaded.values().any(|s| {
                        s.source().traces().iter().any(|t| {
                            t.trace_id == trace.summary.trace_id
                                && matches!(t.presence, TracePresence::Present { .. })
                        })
                    });
                    if !present {
                        unknown(
                            trace,
                            if saw_flow {
                                "missing_period_baseline"
                            } else {
                                "missing_initial_baseline"
                            },
                        );
                    }
                }
            }
        }
        result.check_generation()?;
        Ok(result)
    }
    fn unknown_all(&mut self, reason: &str) {
        for trace in self.traces.values_mut() {
            unknown(trace, reason);
        }
    }
    fn verify_snapshot(
        &self,
        snapshot: &HistoricalSnapshot,
        cancellation: &CancellationToken,
    ) -> Result<(), HistoricalValidationError> {
        self.protector
            .verify_submission(snapshot.source())
            .map_err(HistoricalValidationError::Protection)?;
        let mut counts = HashMap::<&TraceId, u64>::new();
        for row in snapshot.identities() {
            check_cancel(cancellation)?;
            self.protector
                .verify_stored_identity(row)
                .map_err(HistoricalValidationError::Protection)?;
            let id = row.identity();
            if id.namespace() != &self.namespace
                || id.flow_id() != &self.profile.schema_key().flow_id
                || id.key_kind() != "record_key"
                || id.schema_scope()
                    != &(PersistenceSchemaScope::SchemaVersion {
                        version: self.profile.schema_key().version.clone(),
                    })
            {
                return Err(PersistenceError::CorruptStorage.into());
            }
            *counts.entry(id.trace_id()).or_default() += 1;
        }
        for trace in snapshot.source().traces() {
            let expected = match trace.presence {
                TracePresence::Present { record_count } => record_count,
                TracePresence::Absent => 0,
            };
            if counts.remove(&trace.trace_id).unwrap_or_default() != expected {
                return Err(PersistenceError::CorruptStorage.into());
            }
        }
        if !counts.is_empty() {
            return Err(PersistenceError::CorruptStorage.into());
        }
        Ok(())
    }
    pub(super) fn observe(
        &mut self,
        row: &StoredIdentity,
        line: u64,
    ) -> Result<(), HistoricalValidationError> {
        let id = row.identity();
        let Some(trace) = self.traces.get_mut(&id.trace_id().0) else {
            return Ok(());
        };
        if !trace.seen.insert(id.clone()) {
            return Ok(());
        } // engine owns intra-run diagnostics
        if trace.summary.indeterminate {
            return Ok(());
        }
        let previous = if self.replay {
            trace.baseline.get(id).cloned()
        } else if trace.plan.mode == RawCrossRunMode::CumulativeIdentical
            && trace.baseline.contains_key(id)
        {
            trace.baseline.get(id).cloned()
        } else {
            // Calendar coverage is year-scoped, identity is never reset. An
            // exact earlier occurrence (including another year) is not a new
            // acquisition merely because it is outside this year's baseline.
            match self
                .repository
                .lookup_identity_occurrence(&self.namespace, id)?
            {
                None => None,
                Some(occurrence) => {
                    if Some(occurrence.revision()) > self.generation {
                        self.check_generation()?;
                        return Err(PersistenceError::CorruptStorage.into());
                    }
                    self.protector
                        .verify_submission(occurrence.source())
                        .map_err(HistoricalValidationError::Protection)?;
                    self.protector
                        .verify_stored_identity(occurrence.record())
                        .map_err(HistoricalValidationError::Protection)?;
                    if occurrence.record().identity() != id
                        || occurrence.source().namespace() != &self.namespace
                        || occurrence.source().schema_key() != self.profile.schema_key()
                        || occurrence.source().schema_hash() != self.profile.schema_hash()
                    {
                        return Err(PersistenceError::CorruptStorage.into());
                    }
                    if occurrence.source().state() != SessionState::ApprovedImmutable
                        || cross_run_binding(occurrence.source())?.is_none()
                    {
                        unknown(trace, "unqualified_identity_origin");
                        return Ok(());
                    }
                    Some(occurrence.record().clone())
                }
            }
        };
        let mut violated = false;
        if let Some(previous) = previous {
            if previous.record_fingerprint().protection_scheme()
                != row.record_fingerprint().protection_scheme()
                || previous.record_fingerprint().key_id() != row.record_fingerprint().key_id()
            {
                unknown(trace, "incompatible_content_fingerprint");
                return Ok(());
            }
            if !self.replay && trace.plan.mode == RawCrossRunMode::RejectPreviouslySeen {
                trace.summary.reused_records += 1;
                violated = true;
            } else if previous.record_fingerprint() == row.record_fingerprint() {
                trace.summary.carry_forward_identical += 1;
            } else {
                trace.summary.historical_key_content_mismatch += 1;
                violated = true;
            }
        } else if self.replay {
            trace.summary.historical_key_content_mismatch += 1;
            violated = true;
        } else {
            trace.summary.new_records += 1;
        }
        if violated {
            self.rejected.insert((id.trace_id().clone(), line));
            self.errors.push(ValidationError {
                code: trace.plan.reuse_error.clone(),
                flow_id: self.profile.schema_key().flow_id.clone(),
                schema_version: self.profile.schema_key().version.clone(),
                trace_id: id.trace_id().clone(),
                line_number: line,
                record_identity: RecordIdentity::default(),
                field: None,
                actual: None,
                expected: None,
                context: Vec::new(),
                message:
                    if trace.plan.mode == RawCrossRunMode::RejectPreviouslySeen && !self.replay {
                        "Primary key already belongs to a previous approved submission"
                    } else {
                        "Historical primary key has different record content"
                    }
                    .into(),
            });
        }
        Ok(())
    }
    pub(super) fn finish(
        &mut self,
        coverage: &[TraceCoverageEvidence],
    ) -> Result<(), HistoricalValidationError> {
        self.check_generation()?;
        for trace in self.traces.values_mut() {
            let complete = coverage.iter().any(|t| {
                t.trace_id == trace.summary.trace_id && t.coverage == TraceContextCoverage::Complete
            });
            let absent = coverage.iter().any(|t| {
                t.trace_id == trace.summary.trace_id
                    && t.coverage == TraceContextCoverage::Unavailable
            });
            if !complete && !(absent && trace.plan.mode == RawCrossRunMode::RejectPreviouslySeen) {
                unknown(trace, "incomplete_current_trace");
            }
            if trace.plan.mode == RawCrossRunMode::CumulativeIdentical
                && complete
                && !trace.summary.indeterminate
            {
                trace.summary.missing_previous_records = trace
                    .baseline
                    .keys()
                    .filter(|id| !trace.seen.contains(*id))
                    .count() as u64;
            }
        }
        Ok(())
    }
    pub(super) fn eligible(&self, trace: &TraceId, line: u64) -> bool {
        !self.rejected.contains(&(trace.clone(), line))
    }
    pub(super) fn take_errors(&mut self) -> Vec<ValidationError> {
        std::mem::take(&mut self.errors)
    }
    pub(super) fn summary(&self) -> CrossRunValidationSummary {
        CrossRunValidationSummary {
            traces: self.traces.values().map(|v| v.summary.clone()).collect(),
            unreported_errors: 0,
        }
    }
    pub(super) fn check_generation(&self) -> Result<(), HistoricalValidationError> {
        let actual = self.repository.current_generation(&self.namespace)?;
        if actual != self.generation {
            return Err(PersistenceError::GenerationConflict {
                expected: self.generation,
                actual,
            }
            .into());
        }
        Ok(())
    }
}
fn unknown(trace: &mut TraceState, reason: &str) {
    trace.summary.indeterminate = true;
    if trace.summary.unknown_reason.is_none() {
        trace.summary.unknown_reason = Some(reason.into());
    }
}
fn check_cancel(cancel: &CancellationToken) -> Result<(), HistoricalValidationError> {
    if cancel.is_cancelled() {
        Err(HistoricalValidationError::Cancelled)
    } else {
        Ok(())
    }
}
