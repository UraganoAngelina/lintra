//! Streaming producer and pinned, namespace-scoped lifecycle evidence. SQL and
//! cryptography remain application dependencies, never engine responsibilities.
use crate::engine::{
    flow_execution::FlowRecordObserver, projected_context::HistoricalSetContext,
    record_validator::OperandValidity,
};
use crate::persistence::history::*;
use crate::*;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, HashMap},
    error::Error,
    fmt,
};
type Technical = Box<dyn Error + Send + Sync>;

/// Caller supplies a concrete origin claim and the document it actually read.
/// This public input is not a verified fact or an approval capability.
pub struct HistoricalOriginClaim {
    pub join_id: JoinId,
    pub correlation: Vec<Vec<u8>>,
    pub year: u16,
    pub quarter: u8,
    pub reference: String,
    pub responsible: String,
    pub recorded_at: String,
}
#[derive(Clone)]
pub struct HistoricalOriginAttestation {
    claim: OriginClaim,
    digest: Sha256Digest,
}
#[derive(Clone)]
struct OriginClaim {
    join_id: JoinId,
    correlation: JoinKey,
    correlation_arity: usize,
    period: HistoryPeriod,
    reference: String,
    responsible: String,
    recorded_at: String,
}
impl fmt::Debug for HistoricalOriginAttestation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HistoricalOriginAttestation { .. }")
    }
}
impl HistoricalOriginAttestation {
    /// Explicit C1 claim; verifies document identity, not external clinical
    /// truth. The resulting origin is visible only after snapshot approval.
    pub fn from_document(
        claim: HistoricalOriginClaim,
        document: &[u8],
        expected: Sha256Digest,
    ) -> Result<Self, PersistenceError> {
        let period = HistoryPeriod {
            year: claim.year,
            quarter: claim.quarter,
        };
        period.validate()?;
        let arity = claim.correlation.len();
        let correlation = crate::context::join_key::build_join_key_from_components(
            claim.correlation.iter().map(|v| Ok(v.as_slice())),
        )
        .map_err(|_| PersistenceError::CorruptStorage)?;
        if Sha256Digest(Sha256::digest(document).into()) != expected
            || document.is_empty()
            || claim.reference.trim().is_empty()
            || claim.responsible.trim().is_empty()
            || claim.reference.chars().any(char::is_control)
            || claim.responsible.chars().any(char::is_control)
            || chrono::DateTime::parse_from_rfc3339(&claim.recorded_at).is_err()
        {
            return Err(PersistenceError::CorruptStorage);
        }
        Ok(Self {
            digest: expected,
            claim: OriginClaim {
                join_id: claim.join_id,
                correlation,
                correlation_arity: arity,
                period,
                reference: claim.reference,
                responsible: claim.responsible,
                recorded_at: claim.recorded_at,
            },
        })
    }
}

#[derive(Default)]
struct Lifecycle {
    live: HashMap<Fingerprint, bool>,
    origins: HashMap<Fingerprint, HistoryOrigin>,
    complete: BTreeSet<u32>,
    gaps: HashMap<Fingerprint, BTreeSet<u32>>,
    global_gaps: BTreeSet<u32>,
    unperioded_gap: bool,
}
impl Lifecycle {
    fn apply(
        &mut self,
        period: HistoryPeriod,
        present: bool,
        complete: bool,
        rows: &[HistoryFactRow],
        origins: &[HistoryOrigin],
    ) -> Result<(), PersistenceError> {
        if present {
            self.live.clear();
            for row in rows {
                if let Some(entity) = &row.entity {
                    let positive = self.live.entry(entity.clone()).or_default();
                    *positive |= row.assessment == HistoryAssessment::Positive;
                }
            }
        }
        let n = period.ordinal();
        if complete {
            self.complete.insert(n);
        } else {
            self.global_gaps.insert(n);
        }
        for row in rows {
            if matches!(row.assessment, HistoryAssessment::Indeterminate { .. }) {
                if let Some(entity) = &row.entity {
                    self.gaps.entry(entity.clone()).or_default().insert(n);
                } else {
                    self.global_gaps.insert(n);
                }
            }
        }
        for origin in origins {
            if let Some(previous) = self.origins.get(&origin.entity) {
                if previous != origin {
                    return Err(PersistenceError::CorruptStorage);
                }
            } else {
                self.origins.insert(origin.entity.clone(), origin.clone());
            }
        }
        Ok(())
    }
    fn exists(&self, entity: &Fingerprint, period: HistoryPeriod) -> TruthValue {
        // L1: only this qualified live inventory contributes. A superseded or
        // omitted initial never reappears from an immutable audit snapshot.
        if self.live.get(entity).copied().unwrap_or(false) {
            return TruthValue::True;
        }
        let unknown = TruthValue::Unknown(UnknownReason::IncompleteRelatedContext);
        let Some(origin) = self.origins.get(entity) else {
            return unknown;
        };
        if self.unperioded_gap || origin.from.ordinal() > period.ordinal() {
            return unknown;
        }
        for n in origin.from.ordinal()..=period.ordinal() {
            if !self.complete.contains(&n)
                || self.global_gaps.contains(&n)
                || self.gaps.get(entity).is_some_and(|gaps| gaps.contains(&n))
            {
                return unknown;
            }
        }
        TruthValue::False
    }
}

/// All retained clinical-row material is reduced to protected tokens and minimal
/// assessments. Source coordinates are ephemeral, for the current engine only.
pub(crate) struct HistoryCollector<'a> {
    schema: &'a CompiledFlowSchema,
    profile: &'a CompiledPersistenceProfile,
    namespace: PersistenceNamespace,
    period: HistoryPeriod,
    repository: &'a dyn PersistentContextRepository,
    protector: &'a IdentityProtector,
    cancellation: &'a CancellationToken,
    generation: Option<SnapshotRevision>,
    views: Vec<Lifecycle>,
    current: Vec<Vec<HistoryFactRow>>,
    sources: Vec<(usize, TraceId, u64, Option<Fingerprint>)>,
    source_validity: HashMap<(TraceId, u64), Box<[OperandValidity]>>,
    results: HashMap<(usize, TraceId, u64), TruthValue>,
    pub(crate) projections: Vec<ProjectedHistoricalRecord>,
    token_count: u64,
    finished: bool,
}
pub(crate) struct HistoryCollectorSetup<'a> {
    pub(crate) schema: &'a CompiledFlowSchema,
    pub(crate) profile: &'a CompiledPersistenceProfile,
    pub(crate) namespace: PersistenceNamespace,
    pub(crate) period: HistoryPeriod,
    pub(crate) repository: &'a dyn PersistentContextRepository,
    pub(crate) protector: &'a IdentityProtector,
    pub(crate) cancellation: &'a CancellationToken,
}
impl<'a> HistoryCollector<'a> {
    pub(crate) fn new(
        setup: HistoryCollectorSetup<'a>,
        origins: Vec<HistoricalOriginAttestation>,
        replay_submission: &str,
    ) -> Result<Self, HistoricalValidationError> {
        let HistoryCollectorSetup {
            schema,
            profile,
            namespace,
            period,
            repository,
            protector,
            cancellation,
        } = setup;
        protector
            .verify_audit(&repository.load_protection_audit()?)
            .map_err(HistoricalValidationError::Protection)?;
        let generation = repository.current_generation(&namespace)?;
        let mut result = Self {
            schema,
            profile,
            namespace,
            period,
            repository,
            protector,
            cancellation,
            generation,
            views: (0..schema.history_existence().len())
                .map(|_| Lifecycle::default())
                .collect(),
            current: vec![Vec::new(); schema.history_existence().len()],
            sources: Vec::new(),
            source_validity: HashMap::new(),
            results: HashMap::new(),
            projections: Vec::new(),
            token_count: 0,
            finished: false,
        };
        result.load_history(replay_submission)?;
        for attestation in origins {
            let index = schema
                .history_existence()
                .iter()
                .position(|p| {
                    schema.cross_record_plan().joins[p.join_index()].id == attestation.claim.join_id
                })
                .ok_or(HistoricalValidationError::InvalidRequest)?;
            if attestation.claim.correlation_arity
                != schema.cross_record_plan().joins[schema.history_existence()[index].join_index()]
                    .source_fields
                    .len()
            {
                return Err(HistoricalValidationError::InvalidRequest);
            }
            if attestation.claim.period.ordinal() > period.ordinal() {
                return Err(HistoricalValidationError::InvalidRequest);
            }
            let entity = result.entity(index, &attestation.claim.correlation)?;
            let origin = HistoryOrigin {
                entity,
                from: attestation.claim.period,
                evidence_sha256: attestation.digest,
                evidence_reference: attestation.claim.reference,
                responsible: attestation.claim.responsible,
                recorded_at: attestation.claim.recorded_at,
            };
            if let Some(previous) = result.views[index]
                .origins
                .insert(origin.entity.clone(), origin.clone())
                && previous != origin
            {
                return Err(HistoricalValidationError::InvalidRequest);
            }
        }
        Ok(result)
    }
    fn checkpoint(&self) -> Result<(), PersistenceError> {
        let actual = self.repository.current_generation(&self.namespace)?;
        if actual != self.generation {
            return Err(PersistenceError::GenerationConflict {
                expected: self.generation,
                actual,
            });
        }
        Ok(())
    }
    fn entity(
        &self,
        index: usize,
        key: &JoinKey,
    ) -> Result<Fingerprint, HistoricalValidationError> {
        let plan = &self.schema.history_existence()[index];
        let join = &self.schema.cross_record_plan().joins[plan.join_index()];
        let scope = IdentityProtectionScope::new(
            self.namespace.clone(),
            self.profile.schema_key().flow_id.clone(),
            PersistenceSchemaScope::SchemaVersion {
                version: self.profile.schema_key().version.clone(),
            },
            self.schema.traces()[join.target_trace].id().clone(),
            format!("history.entity.v1:{}", plan.definition_hash()),
        )
        .map_err(HistoricalValidationError::Protection)?;
        Ok(self
            .protector
            .protect_key(&scope, key)
            .map_err(HistoricalValidationError::Protection)?
            .key_fingerprint()
            .clone())
    }
    fn load_history(&mut self, replay_submission: &str) -> Result<(), HistoricalValidationError> {
        let mut after = None;
        // Reconstruct a replay from the same predecessor inventory as its
        // original execution. Later approved origins/facts must not silently
        // enter that immutable candidate. Still verify every later inventory
        // and the pinned generation: replay never bypasses integrity or CAS.
        let mut replay_boundary = false;
        let mut later_period = false;
        loop {
            if self.cancellation.is_cancelled() {
                return Err(HistoricalValidationError::Cancelled);
            }
            self.checkpoint()?;
            let page = self
                .repository
                .load_submission_page(&self.namespace, after)?;
            if page.len() > 128 {
                return Err(PersistenceError::CorruptStorage.into());
            }
            if page.is_empty() {
                break;
            }
            for (revision, source) in page {
                self.checkpoint()?;
                let expected = after
                    .map(|r: SnapshotRevision| r.next())
                    .transpose()
                    .map_err(|_| PersistenceError::RevisionOverflow)?
                    .unwrap_or(SnapshotRevision::new(1));
                if revision != expected
                    || Some(revision) > self.generation
                    || source.namespace() != &self.namespace
                {
                    return Err(PersistenceError::CorruptStorage.into());
                }
                self.protector
                    .verify_submission(&source)
                    .map_err(HistoricalValidationError::Protection)?;
                after = Some(revision);
                if source.state() != SessionState::ApprovedImmutable
                    || source.schema_key().flow_id != self.profile.schema_key().flow_id
                {
                    continue;
                }
                let period = super::session::historical_period(&source)?;
                later_period |= period.is_some_and(|p| p.ordinal() > self.period.ordinal());
                let inventory = self
                    .repository
                    .load_history_facts(&self.namespace, revision)?
                    .ok_or(PersistenceError::CorruptStorage)?;
                if inventory.source() != &source || inventory.revision() != revision {
                    return Err(PersistenceError::CorruptStorage.into());
                }
                replay_boundary |= source.submission_id() == replay_submission;
                if replay_boundary {
                    continue;
                }
                for (index, plan) in self.schema.history_existence().iter().enumerate() {
                    let target = self.schema.traces()
                        [self.schema.cross_record_plan().joins[plan.join_index()].target_trace]
                        .id();
                    let present = source
                        .traces()
                        .iter()
                        .find(|t| &t.trace_id == target)
                        .is_some_and(|t| matches!(t.presence, TracePresence::Present { .. }));
                    let batch = inventory
                        .batches()
                        .iter()
                        .find(|b| b.plan_digest() == plan.definition_hash());
                    let compatible = source.schema_key() == self.profile.schema_key()
                        && source.schema_hash() == self.profile.schema_hash();
                    if let Some(period) = period {
                        let qualified =
                            batch.filter(|_| compatible && inventory.inventory_supported());
                        if let Some(batch) = qualified
                            && (batch.payload.period != period
                                || &batch.payload.target_trace != target)
                        {
                            return Err(PersistenceError::CorruptStorage.into());
                        }
                        self.views[index].apply(
                            period,
                            present,
                            present && qualified.is_some(),
                            qualified.map(|b| b.payload.facts.as_slice()).unwrap_or(&[]),
                            qualified
                                .map(|b| b.payload.origins.as_slice())
                                .unwrap_or(&[]),
                        )?;
                    } else {
                        self.views[index].unperioded_gap = true;
                        if present {
                            self.views[index].live.clear();
                        }
                    }
                }
            }
        }
        self.checkpoint()?;
        if after != self.generation {
            return Err(PersistenceError::CorruptStorage.into());
        }
        // Quarterly snapshot contracts require chronological new approvals.
        // Refuse a new backdated run before input evaluation, rather than use
        // future facts or replace the active inventory with an earlier period.
        // An exact immutable replay instead reconstructs its original boundary.
        if later_period && !replay_boundary {
            return Err(HistoricalValidationError::InvalidRequest);
        }
        Ok(())
    }
    pub(crate) fn attach(
        &self,
        snapshot: &HistoricalSnapshot,
    ) -> Result<HistoricalSnapshot, SessionApprovalError> {
        self.checkpoint()?;
        if !self.finished || snapshot.namespace() != &self.namespace {
            return Err(SessionApprovalError::InvalidValidationEvidence);
        }
        let mut batches = Vec::new();
        for (index, plan) in self.schema.history_existence().iter().enumerate() {
            let source = snapshot.source();
            let mut origins = self.views[index]
                .origins
                .values()
                .cloned()
                .collect::<Vec<_>>();
            origins.sort_by(|a, b| a.entity.as_bytes().cmp(b.entity.as_bytes()));
            batches.push(HistoryFactBatch::create(
                HistoryBatchPayload {
                    format: HISTORY_FACT_FORMAT,
                    schema_key: self.profile.schema_key().clone(),
                    schema_hash: self.profile.schema_hash().clone(),
                    namespace: self.namespace.clone(),
                    target_trace: self.schema.traces()
                        [self.schema.cross_record_plan().joins[plan.join_index()].target_trace]
                        .id()
                        .clone(),
                    plan_digest: plan.definition_hash().clone(),
                    source_fingerprint: source.fingerprint().clone(),
                    revision: snapshot.revision(),
                    base_generation: snapshot
                        .revision()
                        .value()
                        .checked_sub(1)
                        .filter(|v| *v != 0)
                        .map(SnapshotRevision::new),
                    period: self.period,
                    source_period: source
                        .period()
                        .ok_or(SessionApprovalError::InvalidBinding)?
                        .into(),
                    facts: self.current[index].clone(),
                    origins,
                },
                source,
                self.protector,
            )?);
        }
        snapshot
            .clone()
            .with_history_facts(batches, self.protector)
            .map_err(Into::into)
    }
}
impl HistoricalSetContext for HistoryCollector<'_> {
    fn operand(&self, source: &TraceId, line: u64, field: usize) -> Option<OperandValidity> {
        self.schema.trace(source)?.fields().get(field)?;
        Some(
            self.source_validity
                .get(&(source.clone(), line))
                .and_then(|states| states.get(field))
                .copied()
                .unwrap_or(OperandValidity::Unavailable),
        )
    }
    fn exists(&self, join: usize, trace: &TraceId, line: u64) -> Option<TruthValue> {
        self.results
            .get(&(join, trace.clone(), line))
            .copied()
            .or_else(|| {
                self.schema
                    .history_existence()
                    .iter()
                    .find(|p| p.join_index() == join)
                    .filter(|p| {
                        self.schema.traces()
                            [self.schema.cross_record_plan().joins[p.join_index()].source_trace]
                            .id()
                            == trace
                    })
                    .map(|_| TruthValue::Unknown(UnknownReason::InvalidFieldValue))
            })
    }
}
fn correlation(
    trace: &CompiledRecordSchema,
    record: &RawRecord<'_>,
    fields: &[usize],
    validity: &[OperandValidity],
) -> Result<Option<JoinKey>, Technical> {
    if !fields
        .iter()
        .all(|i| validity.get(*i) == Some(&OperandValidity::Valid))
    {
        return Ok(None);
    }
    Ok(Some(crate::build_join_key(
        &crate::RecordView::new(record.bytes),
        fields,
        trace,
    )?))
}
impl FlowRecordObserver for HistoryCollector<'_> {
    fn historical_context(&self) -> Option<&dyn HistoricalSetContext> {
        Some(self)
    }
    fn on_record(
        &mut self,
        trace: &CompiledRecordSchema,
        record: &RawRecord<'_>,
        _key: &JoinKey,
        operands: &[OperandValidity],
    ) -> Result<(), Technical> {
        // The view is immutable during PASS1. Pin/check around preparation and
        // completion, then CAS at approval; no repository query per input row.
        self.source_validity
            .insert((trace.id().clone(), record.line_number), operands.into());
        let projection = ProjectedHistoricalRecord::project(
            self.profile,
            &self.namespace,
            trace.id(),
            *record,
            self.protector,
        )?;
        let physical = &projection.tokens[0];
        for (index, plan) in self.schema.history_existence().iter().enumerate() {
            let join = &self.schema.cross_record_plan().joins[plan.join_index()];
            if self.schema.traces()[join.source_trace].id() == trace.id() {
                let key = correlation(trace, record, &join.source_fields, operands)?;
                let entity = key
                    .as_ref()
                    .map(|key| self.entity(index, key))
                    .transpose()?;
                self.sources.push((
                    plan.join_index(),
                    trace.id().clone(),
                    record.line_number,
                    entity,
                ));
            }
            if self.schema.traces()[join.target_trace].id() == trace.id() {
                let key = correlation(trace, record, &join.target_fields, operands)?;
                let entity = key
                    .as_ref()
                    .map(|key| self.entity(index, key))
                    .transpose()?;
                let pk_valid = trace
                    .record_key()
                    .field_indexes
                    .iter()
                    .all(|i| operands.get(*i) == Some(&OperandValidity::Valid));
                let assessment = if entity.is_none() || !pk_valid {
                    HistoryAssessment::Indeterminate {
                        reason: HistoryIndeterminate::UnqualifiedCorrelation,
                    }
                } else {
                    match operands.get(plan.operand_index()) {
                        Some(OperandValidity::Valid) => {
                            let field = &trace.fields()[plan.operand_index()];
                            let raw = field
                                .range()
                                .slice(record.bytes)
                                .ok_or(PersistenceError::CorruptStorage)?;
                            let logical =
                                crate::logical_value(&field.alignment(), FieldSlice { raw })?;
                            if logical.as_bytes() == plan.equality_bytes() {
                                HistoryAssessment::Positive
                            } else {
                                HistoryAssessment::NonWitness
                            }
                        }
                        Some(OperandValidity::Missing) => HistoryAssessment::Indeterminate {
                            reason: HistoryIndeterminate::MissingOperand,
                        },
                        Some(OperandValidity::Invalid) => HistoryAssessment::Indeterminate {
                            reason: HistoryIndeterminate::InvalidOperand,
                        },
                        _ => HistoryAssessment::Indeterminate {
                            reason: HistoryIndeterminate::UnavailableOperand,
                        },
                    }
                };
                self.current[index].push(HistoryFactRow {
                    entity,
                    witness_ordinal: self.token_count,
                    witness_key: physical.identity().key_fingerprint().clone(),
                    witness_record: physical.record_fingerprint().clone(),
                    assessment,
                });
            }
        }
        self.token_count = self
            .token_count
            .checked_add(u64::try_from(projection.tokens.len())?)
            .ok_or(PersistenceError::RevisionOverflow)?;
        self.projections.push(projection);
        Ok(())
    }
    fn finish_pass_one(&mut self, coverage: &[TraceCoverageEvidence]) -> Result<(), Technical> {
        self.checkpoint()?;
        for (index, plan) in self.schema.history_existence().iter().enumerate() {
            let target = self.schema.traces()
                [self.schema.cross_record_plan().joins[plan.join_index()].target_trace]
                .id();
            let status = coverage
                .iter()
                .find(|c| &c.trace_id == target)
                .ok_or(PersistenceError::CorruptStorage)?;
            let present = status.coverage != TraceContextCoverage::Unavailable;
            self.views[index].apply(
                self.period,
                present,
                status.coverage == TraceContextCoverage::Complete
                    && u64::try_from(self.current[index].len()).ok()
                        == Some(status.records_processed),
                &self.current[index],
                &[],
            )?;
        }
        for (join, trace, line, entity) in &self.sources {
            let index = self
                .schema
                .history_existence()
                .iter()
                .position(|p| p.join_index() == *join)
                .ok_or(PersistenceError::CorruptStorage)?;
            let truth = entity
                .as_ref()
                .map(|entity| self.views[index].exists(entity, self.period))
                .unwrap_or(TruthValue::Unknown(UnknownReason::InvalidFieldValue));
            self.results.insert((*join, trace.clone(), *line), truth);
        }
        self.checkpoint()?;
        self.finished = true;
        Ok(())
    }
}
