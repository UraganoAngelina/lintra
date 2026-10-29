//! Extended source wire kept separate from the public Parts constructor so
//! existing caller struct literals remain source compatible. Missing consent
//! means NOT RECORDED, including for legacy ApprovedImmutable metadata.
use super::*;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceSubmissionWire {
    namespace: PersistenceNamespace,
    submission_id: String,
    schema_key: SchemaKey,
    schema_hash: Sha256Digest,
    period: Option<String>,
    #[serde(with = "super::utc")]
    created_at: DateTime<Utc>,
    validator_version: String,
    validation_mode: ValidationMode,
    validation_status: RunStatus,
    reference_data_versions: Vec<(DatasetId, DatasetVersion)>,
    traces: Vec<SubmissionTrace>,
    fingerprint: Fingerprint,
    state: SessionState,
    #[serde(default)]
    missing_trace_consent: Option<MissingTraceProceedConsent>,
}
impl TryFrom<SourceSubmissionWire> for SourceSubmission {
    type Error = PersistenceTypeError;
    fn try_from(w: SourceSubmissionWire) -> Result<Self, Self::Error> {
        let parts = SourceSubmissionParts {
            namespace: w.namespace,
            submission_id: w.submission_id,
            schema_key: w.schema_key,
            schema_hash: w.schema_hash,
            period: w.period,
            created_at: w.created_at,
            validator_version: w.validator_version,
            validation_mode: w.validation_mode,
            validation_status: w.validation_status,
            reference_data_versions: w.reference_data_versions,
            traces: w.traces,
            fingerprint: w.fingerprint,
            state: w.state,
        };
        match w.missing_trace_consent {
            Some(consent) => Self::new_with_missing_trace_consent(parts, consent),
            None => Self::new(parts),
        }
    }
}
