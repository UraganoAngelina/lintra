//! Explicit schema-bound secondary identities. No raw record survives projection;
//! a projection is not evidence that clinical validation has been performed.
use crate::schema::compiled::compiler::CanonicalKeyField;
use crate::{
    ApprovalPreview, CancellationToken, CompiledPersistenceProfile, IdentityProtectionScope,
    IdentityProtector, JoinKey, JoinKeyError, PersistenceError, PersistenceNamespace,
    PersistenceSchemaScope, PersistentContextRepository, ProtectionError, RawRecord,
    RecordIdentity, SessionApprovalError, SessionState, Sha256Digest, StoredIdentity, TraceId,
    ValidatedApproval, ValidationError, VersionCompatibility,
};
use std::{error::Error, fmt};

pub(crate) const PROGRESSIVE_RECORD: &str = "history_identity.progressive_record.v1";
pub(crate) const EVENT_BINDING: &str = "history_identity.event_binding.v1";

#[derive(Debug)]
#[non_exhaustive]
pub enum HistoricalProjectionError {
    NotDeclared,
    InvalidRecord,
    Key(JoinKeyError),
    Protection(ProtectionError),
}
impl fmt::Display for HistoricalProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotDeclared => "no supported historical identity projection is declared",
            Self::InvalidRecord => {
                "historical projection requires a full-width record and a positive source line"
            }
            Self::Key(_) => "historical key extraction failed",
            Self::Protection(_) => "historical identity protection failed",
        })
    }
}
impl Error for HistoricalProjectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Key(error) => Some(error),
            Self::Protection(error) => Some(error),
            _ => None,
        }
    }
}

/// An inseparable compiler-bound projection of one source row. There is no
/// deserializer, public constructor from tokens, raw byte accessor or setter.
/// Diagnostic origin is retained in memory, never added to storage payloads.
/// ```compile_fail
/// let _: lintra::ProjectedHistoricalRecord = serde_json::from_str("{}").unwrap();
/// ```
pub struct ProjectedHistoricalRecord {
    pub(crate) profile_hash: Sha256Digest,
    pub(crate) trace_index: usize,
    pub(crate) line_number: u64,
    pub(crate) tokens: Vec<StoredIdentity>,
}
impl fmt::Debug for ProjectedHistoricalRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProjectedHistoricalRecord { .. }")
    }
}
impl ProjectedHistoricalRecord {
    /// The physical PK, progressive identity and event binding are extracted
    /// from the SAME borrowed row using compiler-resolved ranges and policies.
    /// Different key kinds separate their existing v1 HMAC contexts; neither
    /// the cryptographic scheme nor the generic storage wire changes.
    pub fn project(
        profile: &CompiledPersistenceProfile,
        namespace: &PersistenceNamespace,
        trace: &TraceId,
        record: RawRecord<'_>,
        protector: &IdentityProtector,
    ) -> Result<Self, HistoricalProjectionError> {
        use HistoricalProjectionError as E;
        let trace_index = profile
            .trace_ids()
            .iter()
            .position(|id| id == trace)
            .ok_or(E::NotDeclared)?;
        let plan = profile
            .identity_projections()
            .get(&trace_index)
            .ok_or(E::NotDeclared)?;
        if record.line_number == 0 || record.bytes.len() != plan.record_length {
            return Err(E::InvalidRecord);
        }
        let policy = profile.policy().ok_or(E::NotDeclared)?;
        let VersionCompatibility::SameSchemaVersion = &policy.version_compatibility else {
            return Err(E::NotDeclared);
        };
        let scope = |kind: &str| {
            IdentityProtectionScope::new(
                namespace.clone(),
                profile.schema_key().flow_id.clone(),
                PersistenceSchemaScope::SchemaVersion {
                    version: profile.schema_key().version.clone(),
                },
                trace.clone(),
                kind.into(),
            )
            .map_err(E::Protection)
        };
        let definition = profile.schema_hash();
        let pk = key(record, &plan.record_key).map_err(E::Key)?;
        let mut tokens = vec![
            protector
                .protect_record_bytes(&scope("record_key")?, &pk, definition, record.bytes)
                .map_err(E::Protection)?,
        ];
        if let Some(invariant) = &plan.invariant {
            let progressive = key(record, &invariant.progressive).map_err(E::Key)?;
            let event = key(record, &invariant.event).map_err(E::Key)?;
            tokens.push(
                protector
                    .protect_record_bytes(
                        &scope(PROGRESSIVE_RECORD)?,
                        &progressive,
                        definition,
                        record.bytes,
                    )
                    .map_err(E::Protection)?,
            );
            tokens.push(
                protector
                    .protect_projection(&scope(EVENT_BINDING)?, &progressive, definition, &event)
                    .map_err(E::Protection)?,
            );
        }
        Ok(Self {
            profile_hash: definition.clone(),
            trace_index,
            line_number: record.line_number,
            tokens,
        })
    }
    pub fn line_number(&self) -> u64 {
        self.line_number
    }
}

fn key(record: RawRecord<'_>, fields: &[CanonicalKeyField]) -> Result<JoinKey, JoinKeyError> {
    crate::context::join_key::build_join_key_from_components(fields.iter().map(|field| {
        let raw = field
            .range
            .slice(record.bytes)
            .ok_or_else(|| JoinKeyError::FieldUnavailable(field.id.clone()))?;
        if crate::is_missing(&field.missing, raw) {
            return Err(JoinKeyError::MissingComponent(field.id.clone()));
        }
        crate::logical_value(&field.alignment, crate::FieldSlice { raw })
            .map(|value| value.as_bytes())
            .map_err(JoinKeyError::Logical)
    }))
}

pub(crate) struct InvariantAnchor {
    pub(crate) trace_index: usize,
    pub(crate) line_number: u64,
    pub(crate) event_index: usize,
}

/// No raw-token entry point can construct this capability. The verifier remains
/// borrowed, rather than copying key material into a draft or stored snapshot.
pub struct InvariantValidatedApproval<'protector> {
    base: ValidatedApproval,
    profile: CompiledPersistenceProfile,
    anchors: Vec<InvariantAnchor>,
    protector: &'protector IdentityProtector,
}
impl fmt::Debug for InvariantValidatedApproval<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InvariantValidatedApproval { .. }")
    }
}
impl<'protector> InvariantValidatedApproval<'protector> {
    pub(crate) fn new(
        base: ValidatedApproval,
        profile: CompiledPersistenceProfile,
        anchors: Vec<InvariantAnchor>,
        protector: &'protector IdentityProtector,
    ) -> Self {
        Self {
            base,
            profile,
            anchors,
            protector,
        }
    }
    pub fn source(&self) -> &crate::SourceSubmission {
        self.base.source()
    }
    pub fn preview<'repo>(
        &self,
        repository: &'repo dyn PersistentContextRepository,
    ) -> Result<ApprovalPreview<'repo>, SessionApprovalError> {
        self.preview_with_cancellation(repository, &CancellationToken::default())
    }
    /// Existing session immutability/idempotence runs first. A NEW candidate
    /// then checks every progressive/event token at the preview's exact base.
    /// CAS in the unchanged approval coordinator protects that base until commit.
    /// The first normative rejection is returned in source order, not collected
    /// into an unbounded error buffer. Its line comes from the real source row.
    pub fn preview_with_cancellation<'repo>(
        &self,
        repository: &'repo dyn PersistentContextRepository,
        cancellation: &CancellationToken,
    ) -> Result<ApprovalPreview<'repo>, SessionApprovalError> {
        checkpoint(cancellation)?;
        self.protector
            .verify_audit(&repository.load_protection_audit()?)
            .map_err(protection)?;
        let preview = self
            .base
            .preview_with_cancellation(repository, cancellation)?;
        if preview.state() == Some(SessionState::ApprovedImmutable) {
            return Ok(preview);
        }
        let snapshot = preview.snapshot();
        let expected = preview.expected_generation();
        // Qualify the selected PRESENT baselines before trusting an index miss.
        // A different schema version/hash or an incomplete set of secondary
        // identities must not silently reset a progressive's historical scope.
        // The existing audit preparation checks bounded metadata continuity,
        // protection, counts and source compatibility without emitting diffs.
        {
            let audit = crate::HistoricalAuditService::for_identity_invariants(
                &self.profile,
                self.protector,
                repository,
            );
            let prepared = audit
                .prepare(snapshot, cancellation)
                .map_err(SessionApprovalError::HistoricalContext)?;
            let actual = prepared.metadata().namespace_generation;
            if actual != expected {
                return Err(PersistenceError::GenerationConflict { expected, actual }.into());
            }
        }
        let check_generation = || -> Result<(), SessionApprovalError> {
            let actual = repository.current_generation(snapshot.namespace())?;
            if actual != expected {
                return Err(PersistenceError::GenerationConflict { expected, actual }.into());
            }
            Ok(())
        };
        for anchor in &self.anchors {
            checkpoint(cancellation)?;
            check_generation()?;
            let current = snapshot
                .identities()
                .get(anchor.event_index)
                .ok_or(SessionApprovalError::InvalidBinding)?;
            let occurrence =
                repository.lookup_identity_occurrence(snapshot.namespace(), current.identity())?;
            check_generation()?;
            let Some(occurrence) = occurrence else {
                continue;
            };
            let source = occurrence.source();
            let previous = occurrence.record();
            self.protector
                .verify_submission(source)
                .map_err(protection)?;
            self.protector
                .verify_stored_identity(previous)
                .map_err(protection)?;
            if source.state() != SessionState::ApprovedImmutable
                || source.schema_key() != self.profile.schema_key()
                || source.schema_hash() != self.profile.schema_hash()
                || previous.identity() != current.identity()
                || expected.is_none_or(|base| occurrence.revision() > base)
                || previous.record_fingerprint().protection_scheme()
                    != current.record_fingerprint().protection_scheme()
                || previous.record_fingerprint().key_id() != current.record_fingerprint().key_id()
            {
                return Err(SessionApprovalError::InvalidBinding);
            }
            if previous.record_fingerprint() != current.record_fingerprint() {
                return Err(reassignment_error(
                    &self.profile,
                    anchor.trace_index,
                    anchor.line_number,
                ));
            }
        }
        checkpoint(cancellation)?;
        check_generation()?;
        Ok(preview)
    }
    pub fn cancel(self) { /* No repository has been mutated. */
    }
}

fn checkpoint(cancellation: &CancellationToken) -> Result<(), SessionApprovalError> {
    if cancellation.is_cancelled() {
        Err(SessionApprovalError::Cancelled)
    } else {
        Ok(())
    }
}
fn protection(error: ProtectionError) -> SessionApprovalError {
    SessionApprovalError::Persistence(PersistenceError::Protection(error))
}

/// Check the candidate's own functional binding before snapshot uniqueness.
/// Keys borrow existing protected identities; no input bytes or identity clones
/// are retained. Authentication precedes comparison on a collision, so an
/// unverified token cannot be reported as a normative reassignment. This map is
/// released before the snapshot constructs its independent uniqueness set.
pub(crate) fn check_current_bindings(
    profile: &CompiledPersistenceProfile,
    identities: &[StoredIdentity],
    anchors: &[InvariantAnchor],
    protector: &IdentityProtector,
) -> Result<(), SessionApprovalError> {
    let mut bindings = std::collections::HashMap::new();
    for anchor in anchors {
        let current = identities
            .get(anchor.event_index)
            .ok_or(SessionApprovalError::InvalidBinding)?;
        if let Some(&previous) = bindings.get(&current.identity()) {
            let previous: &StoredIdentity = identities
                .get(previous)
                .ok_or(SessionApprovalError::InvalidBinding)?;
            for value in [previous, current] {
                protector
                    .verify_stored_identity(value)
                    .map_err(protection)?;
            }
            if previous.record_fingerprint() != current.record_fingerprint() {
                return Err(reassignment_error(
                    profile,
                    anchor.trace_index,
                    anchor.line_number,
                ));
            }
            return Err(PersistenceError::DuplicateIdentity.into());
        }
        bindings.insert(current.identity(), anchor.event_index);
    }
    Ok(())
}
/// Shared record-level rejection for current and historical bindings. Only
/// schema metadata and the supplied positive source line enter the diagnostic.
pub(crate) fn reassignment_error(
    profile: &CompiledPersistenceProfile,
    trace_index: usize,
    line_number: u64,
) -> SessionApprovalError {
    let Some(plan) = profile
        .identity_projections()
        .get(&trace_index)
        .and_then(|plan| plan.invariant.as_ref())
    else {
        return SessionApprovalError::InvalidBinding;
    };
    let Some(trace) = profile.trace_ids().get(trace_index) else {
        return SessionApprovalError::InvalidBinding;
    };
    if line_number == 0 {
        return SessionApprovalError::InvalidBinding;
    }
    SessionApprovalError::InvariantRejected(Box::new(ValidationError {
        code: plan.on_reassignment_error.clone(),
        flow_id: profile.schema_key().flow_id.clone(),
        schema_version: profile.schema_key().version.clone(),
        trace_id: trace.clone(),
        line_number,
        record_identity: RecordIdentity::default(),
        field: None,
        actual: None,
        expected: Some(
            "The declared progressive tuple must retain its original event tuple.".into(),
        ),
        context: vec![],
        message: "Persistent identity is already associated with a different event.".into(),
    }))
}
