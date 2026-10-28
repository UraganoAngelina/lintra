//! Persistence metadata only. No backend, crypto implementation, raw record,
//! historical policy execution or proof of approval is introduced here.
use super::{
    DatasetId, DatasetVersion, FlowId, RunStatus, SchemaKey, SchemaVersion, Sha256Digest, TraceId,
    ValidationMode,
};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Private provenance capability, not part of a metadata value's Eq/Hash or
/// public wire representation. Authentication MUST use the protector's MAC
/// verification, never this deliberately value-neutral equality implementation.
#[derive(Clone, Default)]
pub(crate) struct ProtectionEvidence(pub(crate) Option<[u8; 32]>);
impl PartialEq for ProtectionEvidence {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl Eq for ProtectionEvidence {}
impl std::hash::Hash for ProtectionEvidence {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}
impl fmt::Debug for ProtectionEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProtectionEvidence([redacted])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PersistenceTypeError {
    EmptyNamespace,
    DuplicateNamespaceParameter,
    InvalidLabel,
    EmptyFingerprint,
    InvalidFingerprintEncoding,
    InconsistentTraceAudit,
    DuplicateTrace,
    DuplicateReferenceVersion,
    RevisionOverflow,
    InconsistentDiff,
}
impl fmt::Display for PersistenceTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never include caller values: namespace/keys may be confidential.
        f.write_str(match self {
            Self::EmptyNamespace => "persistence namespace is empty",
            Self::DuplicateNamespaceParameter => "namespace parameter is duplicated",
            Self::InvalidLabel => "persistence metadata label is empty or contains controls",
            Self::EmptyFingerprint => "fingerprint bytes are empty",
            Self::InvalidFingerprintEncoding => "fingerprint encoding is invalid",
            Self::InconsistentTraceAudit => "trace presence disagrees with input hash availability",
            Self::DuplicateTrace => "submission trace is duplicated",
            Self::DuplicateReferenceVersion => "submission dataset version is duplicated",
            Self::RevisionOverflow => "snapshot revision overflows",
            Self::InconsistentDiff => "historical diff payload contradicts its declared change",
        })
    }
}
impl Error for PersistenceTypeError {}
fn label(value: &str) -> Result<(), PersistenceTypeError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        Err(PersistenceTypeError::InvalidLabel)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum NamespaceValue {
    String(String),
    Integer(i64),
}
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NamespaceEntry {
    parameter: String,
    value: NamespaceValue,
}

/// Parameter names have canonical lexical order; their values retain exact
/// spelling/type. No trim, numeric coercion or schema-independent PK rule.
#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct PersistenceNamespace(Vec<NamespaceEntry>);
impl PersistenceNamespace {
    pub fn new(values: Vec<(String, NamespaceValue)>) -> Result<Self, PersistenceTypeError> {
        if values.is_empty() {
            return Err(PersistenceTypeError::EmptyNamespace);
        }
        let mut entries = Vec::with_capacity(values.len());
        for (parameter, value) in values {
            label(&parameter)?;
            if let NamespaceValue::String(text) = &value {
                label(text)?;
            }
            entries.push(NamespaceEntry { parameter, value });
        }
        entries.sort_by(|a, b| a.parameter.cmp(&b.parameter));
        if entries.windows(2).any(|w| w[0].parameter == w[1].parameter) {
            return Err(PersistenceTypeError::DuplicateNamespaceParameter);
        }
        Ok(Self(entries))
    }
    pub fn parameters(&self) -> impl ExactSizeIterator<Item = (&str, &NamespaceValue)> {
        self.0.iter().map(|e| (e.parameter.as_str(), &e.value))
    }
}
impl fmt::Debug for PersistenceNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PersistenceNamespace")
            .field("parameter_count", &self.0.len())
            .finish_non_exhaustive()
    }
}
impl<'de> Deserialize<'de> for PersistenceNamespace {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        let entries = Vec::<NamespaceEntry>::deserialize(decoder)?;
        Self::new(
            entries
                .into_iter()
                .map(|e| (e.parameter, e.value))
                .collect(),
        )
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PersistenceSchemaScope {
    SchemaVersion { version: SchemaVersion },
    CompatibilityGroup { id: String },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FingerprintWire {
    protection_scheme: String,
    key_id: String,
    value_hex: String,
}
/// An externally supplied opaque token with explicit protection/key provenance.
/// This data type does NOT compute or attest keyed protection. A future secure
/// preflight must qualify the producer/key before enabling actual storage.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "FingerprintWire")]
pub struct Fingerprint {
    protection_scheme: String,
    key_id: String,
    #[serde(rename = "value_hex", serialize_with = "serialize_hex")]
    bytes: Vec<u8>,
}
fn serialize_hex<S: serde::Serializer>(bytes: &[u8], encoder: S) -> Result<S::Ok, S::Error> {
    use fmt::Write;
    let mut text = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        write!(text, "{byte:02x}").map_err(serde::ser::Error::custom)?;
    }
    encoder.serialize_str(&text)
}
impl Fingerprint {
    pub fn new(
        protection_scheme: String,
        key_id: String,
        bytes: Vec<u8>,
    ) -> Result<Self, PersistenceTypeError> {
        label(&protection_scheme)?;
        label(&key_id)?;
        if bytes.is_empty() {
            return Err(PersistenceTypeError::EmptyFingerprint);
        }
        Ok(Self {
            protection_scheme,
            key_id,
            bytes,
        })
    }
    pub fn protection_scheme(&self) -> &str {
        &self.protection_scheme
    }
    pub fn key_id(&self) -> &str {
        &self.key_id
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn same_protection_context(&self, other: &Self) -> bool {
        self.protection_scheme == other.protection_scheme && self.key_id == other.key_id
    }
}
impl TryFrom<FingerprintWire> for Fingerprint {
    type Error = PersistenceTypeError;
    fn try_from(wire: FingerprintWire) -> Result<Self, Self::Error> {
        let text = wire.value_hex;
        if !text.len().is_multiple_of(2) || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(PersistenceTypeError::InvalidFingerprintEncoding);
        }
        let bytes = (0..text.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&text[i..i + 2], 16)
                    .map_err(|_| PersistenceTypeError::InvalidFingerprintEncoding)
            })
            .collect::<Result<_, _>>()?;
        Self::new(wire.protection_scheme, wire.key_id, bytes)
    }
}
impl fmt::Debug for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Fingerprint")
            .field("byte_count", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
pub type RecordFingerprint = Fingerprint;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityWire {
    namespace: PersistenceNamespace,
    flow_id: FlowId,
    schema_scope: PersistenceSchemaScope,
    trace_id: TraceId,
    key_kind: String,
    key_fingerprint: Fingerprint,
}
/// Equality includes tenant, flow, explicit version/group, trace, key kind and
/// token protection context. A matching digest alone cannot join two tenants.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "IdentityWire")]
pub struct PersistentRecordIdentity {
    namespace: PersistenceNamespace,
    flow_id: FlowId,
    schema_scope: PersistenceSchemaScope,
    trace_id: TraceId,
    key_kind: String,
    key_fingerprint: Fingerprint,
    #[serde(skip)]
    pub(crate) protection_evidence: ProtectionEvidence,
}
impl PersistentRecordIdentity {
    pub fn new(
        namespace: PersistenceNamespace,
        flow_id: FlowId,
        schema_scope: PersistenceSchemaScope,
        trace_id: TraceId,
        key_kind: String,
        key_fingerprint: Fingerprint,
    ) -> Result<Self, PersistenceTypeError> {
        label(&flow_id.0)?;
        label(&trace_id.0)?;
        label(&key_kind)?;
        match &schema_scope {
            PersistenceSchemaScope::SchemaVersion { version } => label(&version.0)?,
            PersistenceSchemaScope::CompatibilityGroup { id } => label(id)?,
        }
        Ok(Self {
            namespace,
            flow_id,
            schema_scope,
            trace_id,
            key_kind,
            key_fingerprint,
            protection_evidence: ProtectionEvidence::default(),
        })
    }
    pub fn namespace(&self) -> &PersistenceNamespace {
        &self.namespace
    }
    pub fn flow_id(&self) -> &FlowId {
        &self.flow_id
    }
    pub fn schema_scope(&self) -> &PersistenceSchemaScope {
        &self.schema_scope
    }
    pub fn trace_id(&self) -> &TraceId {
        &self.trace_id
    }
    pub fn key_kind(&self) -> &str {
        &self.key_kind
    }
    pub fn key_fingerprint(&self) -> &Fingerprint {
        &self.key_fingerprint
    }
}
impl TryFrom<IdentityWire> for PersistentRecordIdentity {
    type Error = PersistenceTypeError;
    fn try_from(w: IdentityWire) -> Result<Self, Self::Error> {
        Self::new(
            w.namespace,
            w.flow_id,
            w.schema_scope,
            w.trace_id,
            w.key_kind,
            w.key_fingerprint,
        )
    }
}
impl fmt::Debug for PersistentRecordIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PersistentRecordIdentity")
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Draft,
    Validated,
    ApprovedImmutable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TracePresence {
    Present { record_count: u64 },
    Absent,
}
// Serde's internally tagged UNIT variant ignores trailing fields even with
// deny_unknown_fields. Decode through an empty STRUCT variant instead so an
// absent trace carrying a record_count is rejected, never silently rewritten.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum PresenceWire {
    Present { record_count: u64 },
    Absent {},
}
impl<'de> Deserialize<'de> for TracePresence {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        Ok(match PresenceWire::deserialize(decoder)? {
            PresenceWire::Present { record_count } => Self::Present { record_count },
            PresenceWire::Absent {} => Self::Absent,
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SnapshotRevision(u64);
impl SnapshotRevision {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    pub fn value(self) -> u64 {
        self.0
    }
    pub fn next(self) -> Result<Self, PersistenceTypeError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(PersistenceTypeError::RevisionOverflow)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmissionTrace {
    pub trace_id: TraceId,
    pub presence: TracePresence,
    pub input_sha256: Option<Sha256Digest>,
}
/// Explicit caller metadata. SourceSubmission validates it and exposes no mutation
/// of a restored approved session. This is data, never a commit/approval token.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSubmissionParts {
    pub namespace: PersistenceNamespace,
    pub submission_id: String,
    pub schema_key: SchemaKey,
    pub schema_hash: Sha256Digest,
    pub period: Option<String>,
    #[serde(with = "utc")]
    pub created_at: DateTime<Utc>,
    pub validator_version: String,
    pub validation_mode: ValidationMode,
    pub validation_status: RunStatus,
    pub reference_data_versions: Vec<(DatasetId, DatasetVersion)>,
    pub traces: Vec<SubmissionTrace>,
    pub fingerprint: Fingerprint,
    pub state: SessionState,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SourceSubmissionParts")]
pub struct SourceSubmission {
    namespace: PersistenceNamespace,
    submission_id: String,
    schema_key: SchemaKey,
    schema_hash: Sha256Digest,
    period: Option<String>,
    #[serde(with = "utc")]
    created_at: DateTime<Utc>,
    validator_version: String,
    validation_mode: ValidationMode,
    validation_status: RunStatus,
    reference_data_versions: Vec<(DatasetId, DatasetVersion)>,
    traces: Vec<SubmissionTrace>,
    fingerprint: Fingerprint,
    state: SessionState,
    #[serde(skip)]
    pub(crate) protection_evidence: ProtectionEvidence,
}
impl SourceSubmission {
    pub fn new(mut p: SourceSubmissionParts) -> Result<Self, PersistenceTypeError> {
        label(&p.submission_id)?;
        label(&p.schema_key.flow_id.0)?;
        label(&p.schema_key.version.0)?;
        label(&p.validator_version)?;
        if let Some(period) = &p.period {
            label(period)?;
        }
        p.reference_data_versions.sort_by(|a, b| a.0.cmp(&b.0));
        for (id, version) in &p.reference_data_versions {
            label(&id.0)?;
            label(&version.0)?;
        }
        if p.reference_data_versions
            .windows(2)
            .any(|w| w[0].0 == w[1].0)
        {
            return Err(PersistenceTypeError::DuplicateReferenceVersion);
        }
        p.traces.sort_by(|a, b| a.trace_id.0.cmp(&b.trace_id.0));
        for trace in &p.traces {
            label(&trace.trace_id.0)?;
            if matches!(trace.presence, TracePresence::Present { .. })
                != trace.input_sha256.is_some()
            {
                return Err(PersistenceTypeError::InconsistentTraceAudit);
            }
        }
        if p.traces.windows(2).any(|w| w[0].trace_id == w[1].trace_id) {
            return Err(PersistenceTypeError::DuplicateTrace);
        }
        Ok(Self {
            namespace: p.namespace,
            submission_id: p.submission_id,
            schema_key: p.schema_key,
            schema_hash: p.schema_hash,
            period: p.period,
            created_at: p.created_at,
            validator_version: p.validator_version,
            validation_mode: p.validation_mode,
            validation_status: p.validation_status,
            reference_data_versions: p.reference_data_versions,
            traces: p.traces,
            fingerprint: p.fingerprint,
            state: p.state,
            protection_evidence: ProtectionEvidence::default(),
        })
    }
    /// Only the protection layer may replace the opaque preparation fingerprint
    /// with its keyed representation. An old provenance seal is invalidated.
    pub(crate) fn set_protected_fingerprint(&mut self, value: Fingerprint) {
        self.fingerprint = value;
        self.protection_evidence = ProtectionEvidence::default();
    }
    pub fn namespace(&self) -> &PersistenceNamespace {
        &self.namespace
    }
    pub fn submission_id(&self) -> &str {
        &self.submission_id
    }
    pub fn schema_key(&self) -> &SchemaKey {
        &self.schema_key
    }
    pub fn schema_hash(&self) -> &Sha256Digest {
        &self.schema_hash
    }
    pub fn period(&self) -> Option<&str> {
        self.period.as_deref()
    }
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
    pub fn validator_version(&self) -> &str {
        &self.validator_version
    }
    pub fn validation_mode(&self) -> ValidationMode {
        self.validation_mode
    }
    pub fn validation_status(&self) -> RunStatus {
        self.validation_status
    }
    pub fn reference_data_versions(&self) -> &[(DatasetId, DatasetVersion)] {
        &self.reference_data_versions
    }
    pub fn traces(&self) -> &[SubmissionTrace] {
        &self.traces
    }
    pub fn fingerprint(&self) -> &Fingerprint {
        &self.fingerprint
    }
    pub fn state(&self) -> SessionState {
        self.state
    }
}
impl TryFrom<SourceSubmissionParts> for SourceSubmission {
    type Error = PersistenceTypeError;
    fn try_from(parts: SourceSubmissionParts) -> Result<Self, Self::Error> {
        Self::new(parts)
    }
}
impl fmt::Debug for SourceSubmission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceSubmission")
            .field("trace_count", &self.traces.len())
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}
mod utc {
    use super::*;
    pub fn serialize<S: serde::Serializer>(
        time: &DateTime<Utc>,
        encoder: S,
    ) -> Result<S::Ok, S::Error> {
        encoder.serialize_str(&time.to_rfc3339_opts(SecondsFormat::Nanos, true))
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        decoder: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        let text = String::deserialize(decoder)?;
        DateTime::parse_from_rfc3339(&text)
            .map(|v| v.with_timezone(&Utc))
            .map_err(|_| serde::de::Error::custom("invalid submission UTC timestamp"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoricalChange {
    Inserted,
    Updated,
    Removed,
    Reactivated,
    Unchanged,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiffWire {
    identity: PersistentRecordIdentity,
    base_revision: Option<SnapshotRevision>,
    change: HistoricalChange,
    previous: Option<Fingerprint>,
    current: Option<Fingerprint>,
}
/// One streaming diff item. No global record/error vector and no inference of
/// removal from an absent trace. The future policy executor produces these data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "DiffWire")]
pub struct HistoricalDiff {
    identity: PersistentRecordIdentity,
    base_revision: Option<SnapshotRevision>,
    change: HistoricalChange,
    previous: Option<Fingerprint>,
    current: Option<Fingerprint>,
}
impl HistoricalDiff {
    pub fn new(
        identity: PersistentRecordIdentity,
        base_revision: Option<SnapshotRevision>,
        change: HistoricalChange,
        previous: Option<Fingerprint>,
        current: Option<Fingerprint>,
    ) -> Result<Self, PersistenceTypeError> {
        use HistoricalChange::*;
        let comparable = previous
            .as_ref()
            .zip(current.as_ref())
            .is_some_and(|(a, b)| a.same_protection_context(b));
        let valid = match change {
            Inserted => previous.is_none() && current.is_some(),
            Removed => base_revision.is_some() && previous.is_some() && current.is_none(),
            Updated => base_revision.is_some() && comparable && previous != current,
            Unchanged => base_revision.is_some() && comparable && previous == current,
            Reactivated => base_revision.is_some() && comparable,
        };
        if !valid {
            return Err(PersistenceTypeError::InconsistentDiff);
        }
        Ok(Self {
            identity,
            base_revision,
            change,
            previous,
            current,
        })
    }
    pub fn identity(&self) -> &PersistentRecordIdentity {
        &self.identity
    }
    pub fn base_revision(&self) -> Option<SnapshotRevision> {
        self.base_revision
    }
    pub fn change(&self) -> HistoricalChange {
        self.change
    }
    pub fn previous(&self) -> Option<&Fingerprint> {
        self.previous.as_ref()
    }
    pub fn current(&self) -> Option<&Fingerprint> {
        self.current.as_ref()
    }
}
impl TryFrom<DiffWire> for HistoricalDiff {
    type Error = PersistenceTypeError;
    fn try_from(w: DiffWire) -> Result<Self, Self::Error> {
        Self::new(w.identity, w.base_revision, w.change, w.previous, w.current)
    }
}
pub type HistoryDiff = HistoricalDiff;
