//! Generic approval coordination. The profile selects session parameters; the
//! repository only publishes immutable protected values. This module performs no
//! clinical validation, cumulative reconciliation or SQL. PERSIST08/09 and CAD
//! integration must supply those results before using this boundary.
use crate::{
    CancellationToken, CommitReceipt, CommitResolution, CompiledPersistenceProfile,
    HistoricalSnapshot, IdentityProtector, MissingTraceProceedConsent, NamespaceValue,
    PersistenceError, PersistenceNamespace, PersistenceTypeError, PersistentContextRepository,
    ReportError, RunParameterAudit, RunStatus, ScopeSummary, SessionState, SnapshotRevision,
    SourceSubmission, SourceSubmissionParts, StoredIdentity, TracePresence, ValidationMode,
    ValidationRunOutcome, schema::compiled::ParameterDefinition,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, error::Error, fmt};

const PERIOD_PREFIX: &str = "lintra-session-v1:";

/// Normative rejection is distinct from a storage/report failure. No PHI appears
/// in Display or Debug; the approved-session symbol comes from ADR001, not a
/// flow-name branch or an inferred clinical rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRejection {
    ApprovedSessionImmutable,
    ValidationNotSuccessful,
    MissingProceedConsent,
}
impl SessionRejection {
    pub fn code(self) -> Option<&'static str> {
        match self {
            Self::ApprovedSessionImmutable => Some("E_APPROVED_SESSION_IMMUTABLE"),
            _ => None,
        }
    }
}
#[derive(Debug)]
#[non_exhaustive]
pub enum SessionApprovalError {
    Rejected(SessionRejection),
    InvalidBinding,
    InvalidValidationEvidence,
    UnsupportedSessionPolicy,
    UnboundApprovedHistory,
    SessionPolicyChanged,
    Cancelled,
    InvalidState,
    Metadata(PersistenceTypeError),
    Persistence(PersistenceError),
    Report(ReportError),
}
impl fmt::Display for SessionApprovalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Rejected(SessionRejection::ApprovedSessionImmutable) => "E_APPROVED_SESSION_IMMUTABLE: the session is already approved",
            Self::Rejected(SessionRejection::ValidationNotSuccessful) => "approval requires a successful validation",
            Self::Rejected(SessionRejection::MissingProceedConsent) => "explicit PROCEED is required for every absent trace",
            Self::InvalidBinding => "session parameters or source metadata disagree with the compiled profile",
            Self::InvalidValidationEvidence => "validation evidence does not match the prepared submission",
            Self::UnsupportedSessionPolicy => "the compiled profile has no supported session approval policy",
            Self::UnboundApprovedHistory => "approved history has no verified session binding; explicit reconciliation is required",
            Self::SessionPolicyChanged => "session parameter definitions differ from approved history",
            Self::Cancelled => "session approval was cancelled",
            Self::InvalidState => "session action is not valid in its current state",
            Self::Metadata(_) => "invalid session metadata",
            Self::Persistence(_) => "session persistence operation failed",
            Self::Report(_) => "session report failed; no approval was committed",
        })
    }
}
impl Error for SessionApprovalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Metadata(e) => Some(e),
            Self::Persistence(e) => Some(e),
            Self::Report(e) => Some(e),
            _ => None,
        }
    }
}
impl From<PersistenceError> for SessionApprovalError {
    fn from(e: PersistenceError) -> Self {
        Self::Persistence(e)
    }
}
impl From<PersistenceTypeError> for SessionApprovalError {
    fn from(e: PersistenceTypeError) -> Self {
        Self::Metadata(e)
    }
}

/// Canonical binding stored in the already opaque period metadata. The explicit
/// prefix/version prevents silently interpreting a legacy free-form period.
/// Namespace and flow are also covered by the existing source HMAC. Schema
/// version is deliberately not part of the session identity: updating software
/// or schema must not unlock an already approved session.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionBinding {
    flow: crate::FlowId,
    scope: PersistenceNamespace,
    year: (String, i64),
    quarter: (String, i64),
}
impl SessionBinding {
    fn same_definition(&self, other: &Self) -> bool {
        self.year.0 == other.year.0
            && self.quarter.0 == other.quarter.0
            && self
                .scope
                .parameters()
                .map(|(id, v)| (id, std::mem::discriminant(v)))
                .eq(other
                    .scope
                    .parameters()
                    .map(|(id, v)| (id, std::mem::discriminant(v))))
    }
    fn encode(&self) -> Result<String, SessionApprovalError> {
        Ok(format!(
            "{PERIOD_PREFIX}{}",
            serde_json::to_string(self).map_err(|_| SessionApprovalError::InvalidBinding)?
        ))
    }
    fn decode(value: Option<&str>) -> Result<Self, SessionApprovalError> {
        let encoded = value
            .and_then(|v| v.strip_prefix(PERIOD_PREFIX))
            .ok_or(SessionApprovalError::UnboundApprovedHistory)?;
        let binding: Self = serde_json::from_str(encoded)
            .map_err(|_| SessionApprovalError::UnboundApprovedHistory)?;
        if binding.encode()?.strip_prefix(PERIOD_PREFIX) != Some(encoded) {
            return Err(SessionApprovalError::UnboundApprovedHistory);
        }
        Ok(binding)
    }
}
fn parameter_id(definition: &ParameterDefinition) -> &str {
    match definition {
        ParameterDefinition::String { id, .. } | ParameterDefinition::Integer { id, .. } => id,
    }
}
fn bind(
    profile: &CompiledPersistenceProfile,
    values: &BTreeMap<String, NamespaceValue>,
) -> Result<(PersistenceNamespace, SessionBinding), SessionApprovalError> {
    let policy = profile
        .policy()
        .ok_or(SessionApprovalError::UnsupportedSessionPolicy)?;
    let plan = policy
        .snapshot
        .as_ref()
        .ok_or(SessionApprovalError::UnsupportedSessionPolicy)?;
    // Validate every supplied parameter, including otherwise unused ones. No
    // defaults, numeric coercion or silently ignored unknown bindings.
    for (id, value) in values {
        let valid = profile.parameters().iter().any(|p| match (p, value) {
            (
                ParameterDefinition::String {
                    id: declared,
                    allowed_values,
                },
                NamespaceValue::String(v),
            ) => declared.as_ref() == id && allowed_values.iter().any(|a| a.as_ref() == v),
            (
                ParameterDefinition::Integer {
                    id: declared,
                    min,
                    max,
                },
                NamespaceValue::Integer(v),
            ) => {
                declared.as_ref() == id
                    && v.to_string()
                        .parse::<crate::schema::compiled::IntegerLiteral>()
                        .ok()
                        .is_some_and(|v| min <= &v && &v <= max)
            }
            _ => false,
        });
        if !valid {
            return Err(SessionApprovalError::InvalidBinding);
        }
    }
    let get = |index: usize| -> Result<(String, NamespaceValue), SessionApprovalError> {
        let id = profile
            .parameters()
            .get(index)
            .map(parameter_id)
            .ok_or(SessionApprovalError::InvalidBinding)?;
        Ok((
            id.into(),
            values
                .get(id)
                .ok_or(SessionApprovalError::InvalidBinding)?
                .clone(),
        ))
    };
    let namespace = PersistenceNamespace::new(
        policy
            .namespace_parameters
            .iter()
            .map(|&i| get(i))
            .collect::<Result<_, _>>()?,
    )?;
    let scope = PersistenceNamespace::new(
        plan.scope_parameters
            .iter()
            .map(|&i| get(i))
            .collect::<Result<_, _>>()?,
    )?;
    let integer = |index| -> Result<(String, i64), SessionApprovalError> {
        match get(index)? {
            (id, NamespaceValue::Integer(v)) => Ok((id, v)),
            _ => Err(SessionApprovalError::InvalidBinding),
        }
    };
    Ok((
        namespace,
        SessionBinding {
            flow: profile.schema_key().flow_id.clone(),
            scope,
            year: integer(plan.year_parameter)?,
            quarter: integer(plan.quarter_parameter)?,
        },
    ))
}

/// A private preparation only. Replacing/dropping a draft has no repository
/// effect. SourceSubmission's public metadata states are never approval tokens.
pub struct ApprovalDraft {
    profile: CompiledPersistenceProfile,
    parameters: BTreeMap<String, NamespaceValue>,
    parts: SourceSubmissionParts,
    consent: Option<MissingTraceProceedConsent>,
    binding: SessionBinding,
}
impl fmt::Debug for ApprovalDraft {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApprovalDraft { .. }")
    }
}
impl ApprovalDraft {
    /// For a NEW draft, None period is bound canonically from the compiled
    /// parameters. A supplied period must already be that exact binding. This
    /// never rewrites a stored source, legacy period or signed payload.
    pub fn new(
        profile: &CompiledPersistenceProfile,
        parameters: BTreeMap<String, NamespaceValue>,
        mut parts: SourceSubmissionParts,
        consent: Option<MissingTraceProceedConsent>,
    ) -> Result<Self, SessionApprovalError> {
        let (namespace, binding) = bind(profile, &parameters)?;
        let period = binding.encode()?;
        if parts.state != SessionState::Draft
            || parts.namespace != namespace
            || &parts.schema_key != profile.schema_key()
            || &parts.schema_hash != profile.schema_hash()
            || parts.period.as_ref().is_some_and(|p| p != &period)
        {
            return Err(SessionApprovalError::InvalidBinding);
        }
        parts.period = Some(period);
        // Use the existing domain validation for sorting, labels, presence/hash
        // coherence and duplicate traces. Preserve the public Parts wire/API.
        let source = source(parts.clone(), consent.clone())?;
        let mut traces: Vec<_> = profile.trace_ids().iter().collect();
        traces.sort_by(|a, b| a.0.cmp(&b.0));
        if !source.traces().iter().map(|t| &t.trace_id).eq(traces) {
            return Err(SessionApprovalError::InvalidBinding);
        }
        Ok(Self {
            profile: profile.clone(),
            parameters,
            parts,
            consent,
            binding,
        })
    }
    pub fn state(&self) -> SessionState {
        SessionState::Draft
    }
    /// The caller can present the schema warning before asking for PROCEED.
    /// No history is queried and no consent is inferred by this accessor.
    pub fn missing_trace_warning_code(&self) -> Option<&str> {
        self.parts
            .traces
            .iter()
            .any(|t| t.presence == TracePresence::Absent)
            .then(|| {
                self.profile
                    .policy()
                    .and_then(|p| p.snapshot.as_ref())
                    .map(|p| p.missing_trace_warning_code.as_ref())
            })
            .flatten()
    }
    pub fn absent_traces(&self) -> impl Iterator<Item = &crate::TraceId> {
        self.parts
            .traces
            .iter()
            .filter(|t| t.presence == TracePresence::Absent)
            .map(|t| &t.trace_id)
    }
    /// Record the explicit decision on this private draft. Domain validation
    /// rejects acknowledgements of present/unknown traces. Coverage of every
    /// omission is checked when crossing into Validated.
    pub fn with_proceed_consent(
        mut self,
        consent: MissingTraceProceedConsent,
    ) -> Result<Self, SessionApprovalError> {
        SourceSubmission::new_with_missing_trace_consent(self.parts.clone(), consent.clone())?;
        self.consent = Some(consent);
        Ok(self)
    }
    /// Bind the supplied validation outcome, not a success bool. This boundary
    /// checks the existing public summary contract; it is not a cryptographic
    /// attestation of Full execution. Callers must use the actual validation
    /// service (integration tasks remain separate).
    pub fn validated(
        mut self,
        outcome: &ValidationRunOutcome,
        identities: Vec<StoredIdentity>,
        protector: &IdentityProtector,
    ) -> Result<ValidatedApproval, SessionApprovalError> {
        verify_outcome(&self, outcome)?;
        let absent: Vec<_> = self
            .parts
            .traces
            .iter()
            .filter(|t| t.presence == TracePresence::Absent)
            .map(|t| t.trace_id.clone())
            .collect();
        if !absent.is_empty()
            && !self
                .consent
                .as_ref()
                .is_some_and(|c| absent.iter().all(|id| c.traces().contains(id)))
        {
            return Err(SessionApprovalError::Rejected(
                SessionRejection::MissingProceedConsent,
            ));
        }
        self.parts.state = SessionState::Validated;
        let validated = protector
            .protect_submission(&source(self.parts.clone(), self.consent.clone())?)
            .map_err(PersistenceError::Protection)?;
        self.parts.state = SessionState::ApprovedImmutable;
        let approved = protector
            .protect_submission(&source(self.parts, self.consent)?)
            .map_err(PersistenceError::Protection)?;
        // Protect/verify every identity BEFORE the adapter sees a candidate.
        for identity in &identities {
            let id = identity.identity();
            let trace_index = self
                .profile
                .trace_ids()
                .iter()
                .position(|t| t == id.trace_id());
            let scope_matches = match (
                id.schema_scope(),
                &self
                    .profile
                    .policy()
                    .ok_or(SessionApprovalError::UnsupportedSessionPolicy)?
                    .version_compatibility,
            ) {
                (
                    crate::PersistenceSchemaScope::SchemaVersion { version },
                    crate::VersionCompatibility::SameSchemaVersion,
                ) => version == &self.profile.schema_key().version,
                (
                    crate::PersistenceSchemaScope::CompatibilityGroup { id },
                    crate::VersionCompatibility::DeclaredGroup { id: declared, .. },
                ) => id == declared.as_ref(),
                _ => false,
            };
            if id.flow_id() != &self.profile.schema_key().flow_id
                || !scope_matches
                || id.key_kind() != "record_key"
                || !trace_index.is_some_and(|index| self.profile.record_key_fields(index).is_some())
            {
                return Err(SessionApprovalError::InvalidBinding);
            }
            protector
                .verify_stored_identity(identity)
                .map_err(PersistenceError::Protection)?;
        }
        let snapshot = HistoricalSnapshot::new(SnapshotRevision::new(1), validated, identities)?;
        Ok(ValidatedApproval {
            snapshot,
            approved,
            binding: self.binding,
            warning: self
                .profile
                .policy()
                .and_then(|p| p.snapshot.as_ref())
                .map(|p| p.missing_trace_warning_code.to_string()),
        })
    }
    pub fn cancel(self) { /* consumes private preparation only */
    }
}
fn source(
    parts: SourceSubmissionParts,
    consent: Option<MissingTraceProceedConsent>,
) -> Result<SourceSubmission, PersistenceTypeError> {
    match consent {
        Some(c) => SourceSubmission::new_with_missing_trace_consent(parts, c),
        None => SourceSubmission::new(parts),
    }
}
/// Convert from the already validated chrono value with checked SystemTime
/// arithmetic. From<SystemTime> for chrono can panic for representable Unix
/// instants outside its calendar range; an external summary must never trigger
/// that panic. Before-epoch fractions use floor seconds plus positive nanos.
fn same_validation_start(
    created: &chrono::DateTime<chrono::Utc>,
    actual: std::time::SystemTime,
) -> bool {
    use std::time::{Duration, UNIX_EPOCH};
    let seconds = created.timestamp();
    let nanos = created.timestamp_subsec_nanos();
    // SystemTime clocks do not represent chrono's leap-second nanosecond form.
    if nanos >= 1_000_000_000 {
        return false;
    }
    let base = if seconds >= 0 {
        UNIX_EPOCH.checked_add(Duration::from_secs(seconds as u64))
    } else {
        UNIX_EPOCH.checked_sub(Duration::from_secs(seconds.unsigned_abs()))
    };
    base.and_then(|base| base.checked_add(Duration::from_nanos(u64::from(nanos)))) == Some(actual)
}
fn verify_outcome(
    draft: &ApprovalDraft,
    outcome: &ValidationRunOutcome,
) -> Result<(), SessionApprovalError> {
    let s = &outcome.summary;
    if outcome.status == RunStatus::Cancelled || s.run_status == RunStatus::Cancelled {
        return Err(SessionApprovalError::Cancelled);
    }
    if outcome.status != RunStatus::CompletedValid
        || s.run_status != outcome.status
        || s.truncated
        || s.total_errors != 0
        || s.invalid_records != 0
    {
        return Err(SessionApprovalError::Rejected(
            SessionRejection::ValidationNotSuccessful,
        ));
    }
    if s.validate().is_err()
        || s.validation_mode != ValidationMode::Full
        || draft.parts.validation_mode != s.validation_mode
        || draft.parts.validation_status != s.run_status
        || draft.parts.schema_key != s.schema_key
        || s.schema_hash != draft.parts.schema_hash.to_string()
        || s.validator_version != draft.parts.validator_version
        || !same_validation_start(&draft.parts.created_at, s.started_at)
        || draft
            .parts
            .reference_data_versions
            .iter()
            .cloned()
            .collect::<BTreeMap<_, _>>()
            != s.reference_data_versions
    {
        return Err(SessionApprovalError::InvalidValidationEvidence);
    }
    let ScopeSummary::Flow { trace_ids } = &s.scope_summary else {
        return Err(SessionApprovalError::InvalidValidationEvidence);
    };
    let Some(full) = &s.full_validation else {
        return Err(SessionApprovalError::InvalidValidationEvidence);
    };
    let parameters: BTreeMap<_, _> = draft
        .parameters
        .iter()
        .map(|(id, v)| {
            (
                id.clone(),
                match v {
                    NamespaceValue::String(v) => RunParameterAudit::String(v.clone()),
                    NamespaceValue::Integer(v) => RunParameterAudit::Integer(v.to_string()),
                },
            )
        })
        .collect();
    if trace_ids.is_empty()
        || full.traces.len() != draft.parts.traces.len()
        || full.parameters != parameters
        || s.inputs.len()
            != draft
                .parts
                .traces
                .iter()
                .filter(|t| matches!(t.presence, TracePresence::Present { .. }))
                .count()
    {
        return Err(SessionApprovalError::InvalidValidationEvidence);
    }
    for trace in &draft.parts.traces {
        let input = s.inputs.iter().find(|i| i.trace_id == trace.trace_id);
        let evidence = full.traces.iter().find(|t| t.trace_id == trace.trace_id);
        let valid = match trace.presence {
            TracePresence::Absent => input.is_none() && !trace_ids.contains(&trace.trace_id) && evidence.is_none_or(|e| e.records_processed == 0),
            TracePresence::Present { record_count } => trace_ids.contains(&trace.trace_id)
                && input.is_some_and(|i| matches!(&i.completion, crate::InputAuditCompletion::Complete { sha256 } if Some(sha256) == trace.input_sha256.as_ref()))
                && evidence.is_some_and(|e| e.records_processed == record_count && e.coverage == crate::ContextCoverage::Complete),
        };
        if !valid {
            return Err(SessionApprovalError::InvalidValidationEvidence);
        }
    }
    Ok(())
}

/// Validated preparation. Repeated preview is read-only; it does not open a
/// transaction. This is a candidate preview, not PERSIST08's cumulative diff.
pub struct ValidatedApproval {
    snapshot: HistoricalSnapshot,
    approved: SourceSubmission,
    binding: SessionBinding,
    warning: Option<String>,
}
impl fmt::Debug for ValidatedApproval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ValidatedApproval { .. }")
    }
}
impl ValidatedApproval {
    pub fn state(&self) -> SessionState {
        SessionState::Validated
    }
    pub fn source(&self) -> &SourceSubmission {
        self.snapshot.source()
    }
    pub fn missing_trace_warning_code(&self) -> Option<&str> {
        self.source()
            .traces()
            .iter()
            .any(|t| t.presence == TracePresence::Absent)
            .then_some(self.warning.as_deref())
            .flatten()
    }
    pub fn preview<'repo>(
        &self,
        repository: &'repo dyn PersistentContextRepository,
    ) -> Result<ApprovalPreview<'repo>, SessionApprovalError> {
        self.preview_with_cancellation(repository, &CancellationToken::default())
    }
    /// A fixed generation bounds the scan even while another process writes.
    /// Cancellation is checked between bounded metadata pages, not by loading
    /// all identities or interrupting a transaction halfway through publication.
    pub fn preview_with_cancellation<'repo>(
        &self,
        repository: &'repo dyn PersistentContextRepository,
        cancellation: &CancellationToken,
    ) -> Result<ApprovalPreview<'repo>, SessionApprovalError> {
        if cancellation.is_cancelled() {
            return Err(SessionApprovalError::Cancelled);
        }
        let base = repository.current_generation(self.snapshot.namespace())?;
        let candidate = self
            .snapshot
            .with_source(SnapshotRevision::new(1), self.approved.clone())?;
        if let Some(receipt) =
            find_approval(repository, &candidate, &self.binding, base, cancellation)?
        {
            let revision = receipt.revision();
            let candidate = candidate.with_source(revision, self.approved.clone())?;
            return Ok(ApprovalPreview {
                repository,
                snapshot: candidate.clone(),
                candidate,
                base: revision
                    .value()
                    .checked_sub(1)
                    .filter(|v| *v > 0)
                    .map(SnapshotRevision::new),
                phase: ApprovalPhase::Approved(receipt),
            });
        }
        let actual = repository.current_generation(candidate.namespace())?;
        if actual != base {
            return Err(PersistenceError::GenerationConflict {
                expected: base,
                actual,
            }
            .into());
        }
        let revision = base
            .map(|r| r.next())
            .transpose()
            .map_err(|_| PersistenceError::RevisionOverflow)?
            .unwrap_or(SnapshotRevision::new(1));
        Ok(ApprovalPreview {
            repository,
            snapshot: self
                .snapshot
                .with_source(revision, self.snapshot.source().clone())?,
            candidate: candidate.with_source(revision, self.approved.clone())?,
            base,
            phase: ApprovalPhase::Validated,
        })
    }
    pub fn cancel(self) { /* no repository has been mutated */
    }
}
fn find_approval(
    repository: &dyn PersistentContextRepository,
    candidate: &HistoricalSnapshot,
    binding: &SessionBinding,
    upper: Option<SnapshotRevision>,
    cancellation: &CancellationToken,
) -> Result<Option<CommitReceipt>, SessionApprovalError> {
    let Some(upper) = upper else {
        return Ok(None);
    };
    let mut after = None;
    loop {
        if cancellation.is_cancelled() {
            return Err(SessionApprovalError::Cancelled);
        }
        let page = repository.load_submission_page(candidate.namespace(), after)?;
        if page.len() > 128 {
            return Err(PersistenceError::CorruptStorage.into());
        }
        if page.is_empty() {
            return Ok(None);
        }
        for (revision, source) in page {
            if after.is_some_and(|previous| revision <= previous)
                || source.namespace() != candidate.namespace()
            {
                return Err(PersistenceError::CorruptStorage.into());
            }
            if revision > upper {
                return Ok(None);
            }
            after = Some(revision);
            if source.submission_id() == candidate.source().submission_id() {
                if source != *candidate.source() {
                    // Reusing an approved submission to change its period or
                    // contents is an immutable-session rejection, not an I/O
                    // failure. A collision with another flow/unapproved source
                    // remains the repository's technical duplicate-ID error.
                    if source.state() == SessionState::ApprovedImmutable
                        && source.schema_key().flow_id == binding.flow
                    {
                        let previous = SessionBinding::decode(source.period())?;
                        if previous.flow != binding.flow {
                            return Err(SessionApprovalError::UnboundApprovedHistory);
                        }
                        return Err(SessionApprovalError::Rejected(
                            SessionRejection::ApprovedSessionImmutable,
                        ));
                    }
                    return Err(PersistenceError::DuplicateSubmission.into());
                }
                let original = candidate.with_source(revision, candidate.source().clone())?;
                return match repository.resolve_commit(&original)? {
                    CommitResolution::Published(receipt) => Ok(Some(receipt)),
                    _ => Err(SessionApprovalError::Rejected(
                        SessionRejection::ApprovedSessionImmutable,
                    )),
                };
            }
            if source.state() != SessionState::ApprovedImmutable
                || source.schema_key().flow_id != binding.flow
            {
                continue;
            }
            let previous = SessionBinding::decode(source.period())?;
            if previous.flow != source.schema_key().flow_id {
                return Err(SessionApprovalError::UnboundApprovedHistory);
            }
            if !previous.same_definition(binding) {
                return Err(SessionApprovalError::SessionPolicyChanged);
            }
            if previous == *binding {
                return Err(SessionApprovalError::Rejected(
                    SessionRejection::ApprovedSessionImmutable,
                ));
            }
        }
    }
}

enum ApprovalPhase {
    Validated,
    Approved(CommitReceipt),
    Cancelled,
    Uncertain,
}
/// Owns the exact preview/base/candidate. It is intentionally not Clone or
/// deserializable; a caller cannot swap the candidate or the repository after
/// reviewing it. The borrow keeps that repository alive through the operation.
/// ```compile_fail
/// fn cannot_switch_archive(mut preview: lintra::ApprovalPreview<'_>, other: &dyn lintra::PersistentContextRepository) {
///     preview.approve_commit(other, &lintra::CancellationToken::default(), |_| Ok(()));
/// }
/// ```
pub struct ApprovalPreview<'repo> {
    repository: &'repo dyn PersistentContextRepository,
    snapshot: HistoricalSnapshot,
    candidate: HistoricalSnapshot,
    base: Option<SnapshotRevision>,
    phase: ApprovalPhase,
}
impl fmt::Debug for ApprovalPreview<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApprovalPreview { .. }")
    }
}
impl ApprovalPreview<'_> {
    pub fn expected_generation(&self) -> Option<SnapshotRevision> {
        self.base
    }
    pub fn snapshot(&self) -> &HistoricalSnapshot {
        &self.snapshot
    }
    /// None denotes cancellation or an unresolved technical outcome, not Draft.
    pub fn state(&self) -> Option<SessionState> {
        match self.phase {
            ApprovalPhase::Validated => Some(SessionState::Validated),
            ApprovalPhase::Approved(_) => Some(SessionState::ApprovedImmutable),
            _ => None,
        }
    }
    pub fn cancel(&mut self) -> Result<(), SessionApprovalError> {
        match self.phase {
            ApprovalPhase::Validated | ApprovalPhase::Cancelled => {
                self.phase = ApprovalPhase::Cancelled;
                Ok(())
            }
            _ => Err(SessionApprovalError::InvalidState),
        }
    }
    /// The callback must finish the real required report publication. Failure
    /// returns before any repository transaction; report and database are not
    /// claimed to share one atomic transaction. Cancellation is cooperative up
    /// to the final pre-COMMIT checkpoint; publication cannot then be undone.
    pub fn approve_commit(
        &mut self,
        cancellation: &CancellationToken,
        publish_report: impl FnOnce(&HistoricalSnapshot) -> Result<(), ReportError>,
    ) -> Result<CommitReceipt, SessionApprovalError> {
        let repository = self.repository;
        match &self.phase {
            ApprovalPhase::Approved(receipt) => return Ok(receipt.clone()),
            ApprovalPhase::Cancelled => return Err(SessionApprovalError::Cancelled),
            ApprovalPhase::Uncertain => {
                return Err(PersistenceError::CommitOutcomeUnknown {
                    revision: self.candidate.revision(),
                }
                .into());
            }
            ApprovalPhase::Validated => {}
        }
        if cancellation.is_cancelled() {
            self.phase = ApprovalPhase::Cancelled;
            return Err(SessionApprovalError::Cancelled);
        }
        publish_report(&self.snapshot).map_err(SessionApprovalError::Report)?;
        if cancellation.is_cancelled() {
            self.phase = ApprovalPhase::Cancelled;
            return Err(SessionApprovalError::Cancelled);
        }
        let mut tx = repository.begin_transaction(self.candidate.namespace(), self.base)?;
        tx.stage_snapshot(&self.candidate)?;
        if cancellation.is_cancelled() {
            tx.abort()?;
            self.phase = ApprovalPhase::Cancelled;
            return Err(SessionApprovalError::Cancelled);
        }
        match tx.commit() {
            Ok(receipt) => {
                self.phase = ApprovalPhase::Approved(receipt.clone());
                Ok(receipt)
            }
            Err(e @ PersistenceError::CommitOutcomeUnknown { .. }) => {
                self.phase = ApprovalPhase::Uncertain;
                Err(e.into())
            }
            Err(e) => Err(e.into()),
        }
    }
    /// Explicit read-only reconciliation after a lost response. NotPublished
    /// permits a later explicit approval attempt; this method never retries a
    /// write, fabricates rollback or erases an already published approval.
    pub fn resolve_outcome(&mut self) -> Result<CommitResolution, SessionApprovalError> {
        if !matches!(self.phase, ApprovalPhase::Uncertain) {
            return Err(SessionApprovalError::InvalidState);
        }
        let resolution = self.repository.resolve_commit(&self.candidate)?;
        match &resolution {
            CommitResolution::Published(receipt) => {
                self.phase = ApprovalPhase::Approved(receipt.clone())
            }
            CommitResolution::NotPublished => self.phase = ApprovalPhase::Validated,
            CommitResolution::ConflictingSubmission => {}
        }
        Ok(resolution)
    }
}
