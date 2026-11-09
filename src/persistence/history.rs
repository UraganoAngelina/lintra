//! Minimal historical predicate assessments. Integrity uses the EXISTING
//! selected-projection v1 protocol; no new MAC purpose, algorithm or key.
use super::*;
use crate::TraceId;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt};

pub(crate) const HISTORY_FACT_FORMAT: u32 = 2;

/// Minimal authenticated inventory. Roots commit to ordered protected tokens
/// and sorted fact batches. The record root lets a facts-only reader verify the
/// inventory seal without loading clinical-record identities from other tables.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SnapshotManifest {
    format: u32,
    revision: SnapshotRevision,
    records_root: crate::Sha256Digest,
    batches_root: crate::Sha256Digest,
}
fn root(
    values: impl Iterator<Item = Result<serde_json::Value, PersistenceError>>,
) -> Result<crate::Sha256Digest, PersistenceError> {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    for value in values {
        let bytes = crate::schema::canonical::canonical_value_bytes(value?)
            .map_err(|_| PersistenceError::CorruptStorage)?;
        let length = u64::try_from(bytes.len()).map_err(|_| PersistenceError::CorruptStorage)?;
        hash.update(length.to_be_bytes());
        hash.update(bytes);
    }
    Ok(crate::Sha256Digest(hash.finalize().into()))
}
impl SnapshotManifest {
    pub(crate) fn create(snapshot: &HistoricalSnapshot) -> Result<Self, PersistenceError> {
        Ok(Self {
            format: 1,
            revision: snapshot.revision(),
            records_root: root(snapshot.identities().iter().map(|record| {
                serde_json::to_value((record.identity(), record.record_fingerprint()))
                    .map_err(|_| PersistenceError::CorruptStorage)
            }))?,
            batches_root: Self::batch_root(snapshot.history_facts())?,
        })
    }
    fn batch_root(batches: &[HistoryFactBatch]) -> Result<crate::Sha256Digest, PersistenceError> {
        root(batches.iter().map(|batch| {
            serde_json::to_value(&batch.payload).map_err(|_| PersistenceError::CorruptStorage)
        }))
    }
    pub(crate) fn verify_batches(
        &self,
        revision: SnapshotRevision,
        batches: &[HistoryFactBatch],
    ) -> Result<(), PersistenceError> {
        if self.format != 1
            || self.revision != revision
            || self.batches_root != Self::batch_root(batches)?
        {
            return Err(PersistenceError::CorruptStorage);
        }
        Ok(())
    }
    /// Use the existing selected-projection protocol, binding the whole source,
    /// revision, format, and both roots. An identity token alone is insufficient.
    pub(crate) fn seal(
        &self,
        source: &SourceSubmission,
        protector: &IdentityProtector,
    ) -> Result<StoredIdentity, PersistenceError> {
        protector
            .verify_submission(source)
            .map_err(PersistenceError::Protection)?;
        let scope = IdentityProtectionScope::new(
            source.namespace().clone(),
            source.schema_key().flow_id.clone(),
            crate::PersistenceSchemaScope::SchemaVersion {
                version: source.schema_key().version.clone(),
            },
            TraceId("history.snapshot.manifest.v1".into()),
            "history.snapshot.manifest.v1".into(),
        )
        .map_err(PersistenceError::Protection)?;
        let bytes = crate::schema::canonical::canonical_value_bytes(
            serde_json::json!({"source": source, "manifest": self}),
        )
        .map_err(|_| PersistenceError::CorruptStorage)?;
        let revision = self.revision.value().to_be_bytes();
        let key = crate::context::join_key::build_join_key_from_components(
            [source.fingerprint().as_bytes(), revision.as_slice()]
                .into_iter()
                .map(Ok),
        )
        .map_err(|_| PersistenceError::CorruptStorage)?;
        let projection = crate::context::join_key::build_join_key_from_components(
            [bytes.as_slice()].into_iter().map(Ok),
        )
        .map_err(|_| PersistenceError::CorruptStorage)?;
        protector
            .protect_projection(&scope, &key, source.schema_hash(), &projection)
            .map_err(PersistenceError::Protection)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HistoryPeriod {
    pub(crate) year: u16,
    pub(crate) quarter: u8,
}
impl HistoryPeriod {
    pub(crate) fn validate(self) -> Result<(), PersistenceError> {
        if !(1..=9999).contains(&self.year) || !(1..=4).contains(&self.quarter) {
            return Err(PersistenceError::CorruptStorage);
        }
        Ok(())
    }
    pub(crate) fn ordinal(self) -> u32 {
        u32::from(self.year) * 4 + u32::from(self.quarter) - 1
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoryIndeterminate {
    MissingOperand,
    InvalidOperand,
    UnavailableOperand,
    UnqualifiedCorrelation,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields, rename_all = "snake_case")]
pub(crate) enum HistoryAssessment {
    Positive,
    NonWitness,
    Indeterminate { reason: HistoryIndeterminate },
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HistoryFactRow {
    pub(crate) entity: Option<Fingerprint>,
    /// Index into the snapshot's protected-token vector, never a clinical line.
    pub(crate) witness_ordinal: u64,
    pub(crate) witness_key: Fingerprint,
    pub(crate) witness_record: Fingerprint,
    pub(crate) assessment: HistoryAssessment,
}
/// An explicit origin claim, admitted by the application only with verified
/// source evidence and operator responsibility. First DB revision is no claim.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HistoryOrigin {
    pub(crate) entity: Fingerprint,
    pub(crate) from: HistoryPeriod,
    pub(crate) evidence_sha256: crate::Sha256Digest,
    pub(crate) evidence_reference: String,
    pub(crate) responsible: String,
    pub(crate) recorded_at: String,
}
/// Payload DTO is NOT a verified batch. It has no public constructor/export as
/// a capability. Decode and projection verification must precede every use.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HistoryBatchPayload {
    pub(crate) format: u32,
    pub(crate) schema_key: crate::SchemaKey,
    pub(crate) schema_hash: crate::Sha256Digest,
    pub(crate) namespace: PersistenceNamespace,
    pub(crate) target_trace: TraceId,
    pub(crate) plan_digest: crate::Sha256Digest,
    pub(crate) source_fingerprint: Fingerprint,
    pub(crate) revision: SnapshotRevision,
    pub(crate) base_generation: Option<SnapshotRevision>,
    pub(crate) period: HistoryPeriod,
    pub(crate) source_period: String,
    pub(crate) facts: Vec<HistoryFactRow>,
    pub(crate) origins: Vec<HistoryOrigin>,
}
fn canonical(payload: &HistoryBatchPayload) -> Result<Vec<u8>, PersistenceError> {
    crate::schema::canonical::canonical_value_bytes(
        serde_json::to_value(payload).map_err(|_| PersistenceError::CorruptStorage)?,
    )
    .map_err(|_| PersistenceError::CorruptStorage)
}
fn key(payload: &HistoryBatchPayload) -> Result<crate::JoinKey, PersistenceError> {
    let revision = payload.revision.value().to_be_bytes();
    crate::context::join_key::build_join_key_from_components(
        [
            payload.source_fingerprint.as_bytes(),
            payload.plan_digest.0.as_slice(),
            revision.as_slice(),
        ]
        .into_iter()
        .map(Ok),
    )
    .map_err(|_| PersistenceError::CorruptStorage)
}
fn scope(payload: &HistoryBatchPayload) -> Result<IdentityProtectionScope, PersistenceError> {
    IdentityProtectionScope::new(
        payload.namespace.clone(),
        payload.schema_key.flow_id.clone(),
        crate::PersistenceSchemaScope::SchemaVersion {
            version: payload.schema_key.version.clone(),
        },
        payload.target_trace.clone(),
        "history.existence.batch.v1".into(),
    )
    .map_err(PersistenceError::Protection)
}
fn projection(payload: &HistoryBatchPayload) -> Result<crate::JoinKey, PersistenceError> {
    let bytes = canonical(payload)?;
    crate::context::join_key::build_join_key_from_components([bytes.as_slice()].into_iter().map(Ok))
        .map_err(|_| PersistenceError::CorruptStorage)
}
/// Compiler/application-owned authenticated set. Public DTO deserialization
/// cannot fabricate this object. Its seal commits to every row, origin and count
/// through the canonical payload, not just to its protected correlation key.
#[derive(Clone, PartialEq, Eq)]
pub struct HistoryFactBatch {
    pub(crate) payload: HistoryBatchPayload,
    pub(crate) seal: StoredIdentity,
}
impl fmt::Debug for HistoryFactBatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HistoryFactBatch")
            .field("format", &self.payload.format)
            .field("facts", &self.payload.facts.len())
            .finish_non_exhaustive()
    }
}
impl HistoryFactBatch {
    /// Bind every assessment to the actual physical record token in the same
    /// publication, not an unrelated but correctly MAC-protected fingerprint.
    /// Secondary progressive/event tokens are deliberately excluded.
    pub(crate) fn verify_records(
        &self,
        records: &[StoredIdentity],
    ) -> Result<(), PersistenceError> {
        let physical_count = records
            .iter()
            .filter(|record| {
                record.identity().trace_id() == &self.payload.target_trace
                    && record.identity().key_kind() == "record_key"
            })
            .count();
        if physical_count != self.payload.facts.len()
            || self.payload.facts.iter().any(|row| {
                let record = usize::try_from(row.witness_ordinal)
                    .ok()
                    .and_then(|index| records.get(index));
                !record.is_some_and(|record| {
                    record.identity().trace_id() == &self.payload.target_trace
                        && record.identity().key_kind() == "record_key"
                        && record.identity().key_fingerprint() == &row.witness_key
                        && record.record_fingerprint() == &row.witness_record
                })
            })
        {
            return Err(PersistenceError::CorruptStorage);
        }
        Ok(())
    }
    pub fn plan_digest(&self) -> &crate::Sha256Digest {
        &self.payload.plan_digest
    }
    pub fn fact_count(&self) -> usize {
        self.payload.facts.len()
    }
    pub(crate) fn create(
        payload: HistoryBatchPayload,
        source: &SourceSubmission,
        protector: &IdentityProtector,
    ) -> Result<Self, PersistenceError> {
        validate(&payload, source)?;
        protector
            .verify_submission(source)
            .map_err(PersistenceError::Protection)?;
        let seal = protector
            .protect_projection(
                &scope(&payload)?,
                &key(&payload)?,
                &payload.plan_digest,
                &projection(&payload)?,
            )
            .map_err(PersistenceError::Protection)?;
        Ok(Self { payload, seal })
    }
    pub(crate) fn verify(
        &self,
        source: &SourceSubmission,
        protector: &IdentityProtector,
    ) -> Result<(), PersistenceError> {
        protector
            .verify_stored_identity(&self.seal)
            .map_err(PersistenceError::Protection)?;
        // Reconstruct the complete canonical selected projection; verifying only
        // the private identity token does not authenticate serialized metadata.
        let expected = Self::create(self.payload.clone(), source, protector)?.seal;
        if expected.identity() != self.seal.identity()
            || expected.record_fingerprint() != self.seal.record_fingerprint()
        {
            return Err(PersistenceError::CorruptStorage);
        }
        Ok(())
    }
}
fn fingerprint(value: &Fingerprint) -> bool {
    value.as_bytes().len() == 32 && !value.key_id().is_empty()
}
fn validate(
    payload: &HistoryBatchPayload,
    source: &SourceSubmission,
) -> Result<(), PersistenceError> {
    payload.period.validate()?;
    if !matches!(payload.format, 1 | HISTORY_FACT_FORMAT)
        || payload.revision.value() == 0
        || payload
            .base_generation
            .is_some_and(|base| base.value().checked_add(1) != Some(payload.revision.value()))
        || (payload.base_generation.is_none() && payload.revision.value() != 1)
        || source.state() != crate::SessionState::ApprovedImmutable
        || source.validation_mode() != crate::ValidationMode::Full
        || source.validation_status() != crate::RunStatus::CompletedValid
        || source.namespace() != &payload.namespace
        || source.schema_key() != &payload.schema_key
        || source.schema_hash() != &payload.schema_hash
        || source.fingerprint() != &payload.source_fingerprint
        || source.period() != Some(payload.source_period.as_str())
        || payload.source_period.trim().is_empty()
    {
        return Err(PersistenceError::CorruptStorage);
    }
    let trace = source
        .traces()
        .iter()
        .find(|trace| trace.trace_id == payload.target_trace)
        .ok_or(PersistenceError::CorruptStorage)?;
    let count = match trace.presence {
        crate::TracePresence::Present { record_count } => record_count,
        crate::TracePresence::Absent => 0,
    };
    if u64::try_from(payload.facts.len()).ok() != Some(count) {
        return Err(PersistenceError::CorruptStorage);
    }
    let mut witnesses = HashSet::with_capacity(payload.facts.len());
    for row in &payload.facts {
        if payload.format == 1 && row.entity.is_none() {
            return Err(PersistenceError::CorruptStorage);
        }
        if !row.entity.as_ref().map_or(
            matches!(
                row.assessment,
                HistoryAssessment::Indeterminate {
                    reason: HistoryIndeterminate::UnqualifiedCorrelation
                }
            ),
            |entity| fingerprint(entity) && entity.key_id() == source.fingerprint().key_id(),
        ) || !fingerprint(&row.witness_key)
            || !fingerprint(&row.witness_record)
            || row.witness_key.key_id() != source.fingerprint().key_id()
            || row.witness_record.key_id() != source.fingerprint().key_id()
            || !witnesses.insert(row.witness_key.as_bytes())
        {
            return Err(PersistenceError::CorruptStorage);
        }
    }
    let mut origins = HashSet::with_capacity(payload.origins.len());
    for origin in &payload.origins {
        origin.from.validate()?;
        if !fingerprint(&origin.entity)
            || origin.entity.key_id() != source.fingerprint().key_id()
            || origin.from.ordinal() > payload.period.ordinal()
            || origin.evidence_reference.trim().is_empty()
            || origin.responsible.trim().is_empty()
            || origin.evidence_reference.chars().any(char::is_control)
            || origin.responsible.chars().any(char::is_control)
            || chrono::DateTime::parse_from_rfc3339(&origin.recorded_at).is_err()
            || !origins.insert(origin.entity.as_bytes())
        {
            return Err(PersistenceError::CorruptStorage);
        }
    }
    Ok(())
}
