//! Explicit Full + audit pipeline. Ordinary CLI/UI constructors never activate
//! it. Execution, source metadata, PKs and projections share one schema/run.
use super::{full::FullServicePorts, session::bind_audit_namespace};
use crate::engine::flow_execution::FlowRecordObserver;
use crate::*;
use std::{collections::BTreeMap, error::Error, fmt};

/// The organization parameters are typed and explicit; there is no inferred
/// period or organization, and the caller supplies a non-sensitive archive ID.
pub struct HistoricalValidationRequest {
    pub validation: ValidationRequest,
    pub parameters: BTreeMap<String, NamespaceValue>,
    pub submission_id: String,
}

/// A sink may contain partial output on error. Its owner must use atomic report
/// publication, exactly as with ValidationSink. No Vec of diffs is required.
pub trait HistoricalAuditSink {
    fn on_diff(&mut self, diff: &HistoricalDiff) -> Result<(), ReportError>;
    fn on_summary(&mut self, summary: &HistoricalAuditSummary) -> Result<(), ReportError>;
}

#[derive(Debug)]
#[non_exhaustive]
pub enum HistoricalValidationError {
    InvalidRequest,
    Cancelled,
    Validation(ValidationRunError),
    Resolution(SchemaResolutionError),
    Persistence(PersistenceError),
    Protection(ProtectionError),
    Approval(SessionApprovalError),
    Audit(HistoricalAuditError),
    Report(ReportError),
}
impl fmt::Display for HistoricalValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidRequest => {
                "historical audit requires Full flow scope and a valid submission ID"
            }
            Self::Cancelled => "historical validation was cancelled before preview completed",
            Self::Validation(_) => "historical input validation failed technically",
            Self::Resolution(_) => "historical schema resolution failed",
            Self::Persistence(_) => "historical repository preflight failed",
            Self::Protection(_) => "historical protection context is incompatible",
            Self::Approval(_) => "historical archive preparation failed",
            Self::Audit(_) => "historical comparison failed",
            Self::Report(_) => "historical report failed; no approval is available",
        })
    }
}
impl Error for HistoricalValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Validation(e) => Some(e),
            Self::Resolution(e) => Some(e),
            Self::Persistence(e) => Some(e),
            Self::Protection(e) => Some(e),
            Self::Approval(e) => Some(e),
            Self::Audit(e) => Some(e),
            Self::Report(e) => Some(e),
            _ => None,
        }
    }
}

/// Invalid, cancelled or inconclusive runs return their real validation outcome
/// with no approval capability. Only a successful audit can construct Preview.
pub enum HistoricalValidationOutcome<'repo> {
    ValidationOnly(Box<ValidationRunOutcome>),
    Preview(Box<HistoricalValidationPreview<'repo>>),
}

/// Holds the exact reviewed candidate and repository. Writing reports or
/// constructing this object does not mutate storage; approval is a separate call.
pub struct HistoricalValidationPreview<'repo> {
    outcome: ValidationRunOutcome,
    audit: HistoricalAuditSummary,
    approval: ApprovalPreview<'repo>,
}
impl HistoricalValidationPreview<'_> {
    pub fn outcome(&self) -> &ValidationRunOutcome {
        &self.outcome
    }
    pub fn audit(&self) -> &HistoricalAuditSummary {
        &self.audit
    }
    pub fn snapshot(&self) -> &HistoricalSnapshot {
        self.approval.snapshot()
    }
    pub fn state(&self) -> Option<SessionState> {
        self.approval.state()
    }
    pub fn cancel(&mut self) -> Result<(), SessionApprovalError> {
        self.approval.cancel()
    }
    /// The adapter must finish publishing ALL required run/audit reports in this
    /// callback. Failure prevents the transaction; a later CAS/storage failure
    /// remains explicit even if report publication already succeeded.
    pub fn approve_commit(
        &mut self,
        cancellation: &CancellationToken,
        publish_reports: impl FnOnce(
            &ValidationRunOutcome,
            &HistoricalAuditSummary,
            &HistoricalSnapshot,
        ) -> Result<(), ReportError>,
    ) -> Result<CommitReceipt, SessionApprovalError> {
        self.approval.approve_commit(cancellation, |snapshot| {
            publish_reports(&self.outcome, &self.audit, snapshot)
        })
    }
    pub fn resolve_outcome(&mut self) -> Result<CommitResolution, SessionApprovalError> {
        self.approval.resolve_outcome()
    }
}

/// All dependencies are explicit. The repository is verified against the key
/// before input access, including when this namespace has never been written.
pub struct HistoricalValidationService<'service, 'repo> {
    validation: &'service ValidationService,
    repository: &'repo dyn PersistentContextRepository,
    protector: &'service IdentityProtector,
}
impl<'service, 'repo> HistoricalValidationService<'service, 'repo> {
    pub fn new(
        validation: &'service ValidationService,
        repository: &'repo dyn PersistentContextRepository,
        protector: &'service IdentityProtector,
    ) -> Self {
        Self {
            validation,
            repository,
            protector,
        }
    }
    pub fn validate(
        &self,
        request: HistoricalValidationRequest,
        sink: &mut dyn ValidationSink,
        audit_sink: &mut dyn HistoricalAuditSink,
        progress: &mut dyn ProgressObserver,
        cancellation: &CancellationToken,
    ) -> Result<HistoricalValidationOutcome<'repo>, HistoricalValidationError> {
        use HistoricalValidationError as E;
        checkpoint(cancellation)?;
        if request.validation.mode != ValidationMode::Full
            || !matches!(request.validation.scope, ValidationScope::Flow(_))
            || request.submission_id.is_empty()
            || request.submission_id.trim() != request.submission_id
        {
            return Err(E::InvalidRequest);
        }
        let resolved = self
            .validation
            .schema_registry
            .resolve_historical_audit(&request.validation.schema_key)
            .map_err(E::Resolution)?;
        let profile = resolved.profile();
        let namespace = bind_audit_namespace(profile, &request.parameters, &request.submission_id)
            .map_err(E::Approval)?;
        let audit = self
            .repository
            .load_protection_audit()
            .map_err(E::Persistence)?;
        self.protector.verify_audit(&audit).map_err(E::Protection)?;
        let mut collector = ProtectedCollector::new(&resolved, namespace.clone(), self.protector)?;
        let parameters = request
            .parameters
            .iter()
            .map(|(id, value)| {
                (
                    id.clone(),
                    match value {
                        NamespaceValue::String(v) => v.clone(),
                        NamespaceValue::Integer(v) => v.to_string(),
                    },
                )
            })
            .collect();
        // Suppress only terminal callbacks. Errors and progress still stream.
        // Their success cannot imply that the later historical audit succeeded.
        let mut deferred = DeferredSink(sink);
        let mut deferred_progress = |event| {
            if event != ProgressEvent::RunCompleted {
                progress.on_event(event);
            }
        };
        let outcome = self
            .validation
            .validate_resolved_with_parameters(
                request.validation,
                parameters,
                FullServicePorts {
                    sink: &mut deferred,
                    progress: &mut deferred_progress,
                    cancellation,
                    started_at: self.validation.clock.now(),
                    resolved: resolved.resolved(),
                    record_observer: Some(&mut collector),
                },
            )
            .map_err(E::Validation)?;
        if outcome.status != RunStatus::CompletedValid
            || outcome.summary.completeness() != ValidationCompleteness::Complete
        {
            sink.on_summary(&outcome.summary).map_err(E::Report)?;
            progress.on_event(ProgressEvent::RunCompleted);
            return Ok(HistoricalValidationOutcome::ValidationOnly(Box::new(
                outcome,
            )));
        }
        checkpoint(cancellation)?;
        let summary = &outcome.summary;
        let full = summary.full_validation.as_ref().ok_or(E::InvalidRequest)?;
        let traces = profile
            .trace_ids()
            .iter()
            .map(
                |id| match summary.inputs.iter().find(|input| &input.trace_id == id) {
                    None => Ok(SubmissionTrace {
                        trace_id: id.clone(),
                        presence: TracePresence::Absent,
                        input_sha256: None,
                    }),
                    Some(input) => {
                        let InputAuditCompletion::Complete { sha256 } = &input.completion else {
                            return Err(E::InvalidRequest);
                        };
                        let trace = full
                            .traces
                            .iter()
                            .find(|trace| &trace.trace_id == id)
                            .ok_or(E::InvalidRequest)?;
                        Ok(SubmissionTrace {
                            trace_id: id.clone(),
                            presence: TracePresence::Present {
                                record_count: trace.records_processed,
                            },
                            input_sha256: Some(sha256.clone()),
                        })
                    }
                },
            )
            .collect::<Result<Vec<_>, E>>()?;
        let parts = SourceSubmissionParts {
            namespace,
            submission_id: request.submission_id,
            schema_key: profile.schema_key().clone(),
            schema_hash: profile.schema_hash().clone(),
            period: None,
            created_at: source_time(summary.started_at)
                .ok_or(E::Report(ReportError::InvalidTimestamp))?,
            validator_version: summary.validator_version.clone(),
            validation_mode: summary.validation_mode,
            validation_status: summary.run_status,
            reference_data_versions: summary
                .reference_data_versions
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            traces,
            fingerprint: self
                .protector
                .draft_fingerprint(profile.schema_hash())
                .map_err(E::Protection)?,
            state: SessionState::Draft,
        };
        let validated = ApprovalDraft::new(profile, request.parameters, parts, None)
            .map_err(E::Approval)?
            .validated(&outcome, collector.identities, self.protector)
            .map_err(E::Approval)?;
        let approval = validated
            .preview_with_cancellation(self.repository, cancellation)
            .map_err(E::Approval)?;
        let service = HistoricalAuditService::new(profile, self.protector, self.repository);
        let prepared = service
            .prepare(approval.snapshot(), cancellation)
            .map_err(E::Audit)?;
        // Idempotent replay can reference an already committed revision. A new
        // candidate must share the audit's exact base, never a newer generation.
        if approval.state() == Some(SessionState::Validated)
            && prepared.metadata().namespace_generation != approval.expected_generation()
        {
            return Err(E::Persistence(PersistenceError::GenerationConflict {
                expected: approval.expected_generation(),
                actual: prepared.metadata().namespace_generation,
            }));
        }
        let audit = prepared
            .write_diffs(cancellation, |diff| audit_sink.on_diff(diff))
            .map_err(E::Audit)?;
        audit_sink.on_summary(&audit).map_err(E::Report)?;
        checkpoint(cancellation)?;
        let actual = self
            .repository
            .current_generation(approval.snapshot().namespace())
            .map_err(E::Persistence)?;
        if actual != audit.metadata.namespace_generation {
            return Err(E::Persistence(PersistenceError::GenerationConflict {
                expected: audit.metadata.namespace_generation,
                actual,
            }));
        }
        sink.on_summary(&outcome.summary).map_err(E::Report)?;
        checkpoint(cancellation)?;
        progress.on_event(ProgressEvent::RunCompleted);
        Ok(HistoricalValidationOutcome::Preview(Box::new(
            HistoricalValidationPreview {
                outcome,
                audit,
                approval,
            },
        )))
    }
}
fn checkpoint(cancellation: &CancellationToken) -> Result<(), HistoricalValidationError> {
    if cancellation.is_cancelled() {
        Err(HistoricalValidationError::Cancelled)
    } else {
        Ok(())
    }
}
/// An injected clock can represent instants beyond chrono's calendar. Convert
/// with checked floor-seconds arithmetic, preserving pre-epoch nanoseconds and
/// returning a typed report failure instead of DateTime::from's panic.
fn source_time(time: std::time::SystemTime) -> Option<chrono::DateTime<chrono::Utc>> {
    let (seconds, nanos) = match time.duration_since(std::time::UNIX_EPOCH) {
        Ok(value) => (i128::from(value.as_secs()), value.subsec_nanos()),
        Err(error) => {
            let value = error.duration();
            if value.subsec_nanos() == 0 {
                (-i128::from(value.as_secs()), 0)
            } else {
                (
                    -i128::from(value.as_secs()) - 1,
                    1_000_000_000 - value.subsec_nanos(),
                )
            }
        }
    };
    chrono::DateTime::from_timestamp(i64::try_from(seconds).ok()?, nanos)
}
struct DeferredSink<'a>(&'a mut dyn ValidationSink);
impl ValidationSink for DeferredSink<'_> {
    fn on_error(&mut self, error: &ValidationError) -> Result<(), ReportError> {
        self.0.on_error(error)
    }
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        Ok(())
    }
}

/// Retains protected tokens only. Raw bytes live solely in the engine's borrowed
/// row buffer, and neither a second input scan nor a second schema load occurs.
struct ProtectedCollector<'a> {
    definitions: BTreeMap<String, (IdentityProtectionScope, Sha256Digest)>,
    protector: &'a IdentityProtector,
    identities: Vec<StoredIdentity>,
}
impl<'a> ProtectedCollector<'a> {
    fn new(
        resolved: &ResolvedHistoricalSchema,
        namespace: PersistenceNamespace,
        protector: &'a IdentityProtector,
    ) -> Result<Self, HistoricalValidationError> {
        let profile = resolved.profile();
        let schema_scope = match &profile
            .policy()
            .ok_or(HistoricalValidationError::InvalidRequest)?
            .version_compatibility
        {
            VersionCompatibility::SameSchemaVersion => PersistenceSchemaScope::SchemaVersion {
                version: profile.schema_key().version.clone(),
            },
            VersionCompatibility::DeclaredGroup { id, .. } => {
                PersistenceSchemaScope::CompatibilityGroup { id: id.to_string() }
            }
        };
        let definitions = resolved
            .record_projections()
            .iter()
            .map(|definition| {
                let scope = IdentityProtectionScope::new(
                    namespace.clone(),
                    profile.schema_key().flow_id.clone(),
                    schema_scope.clone(),
                    definition.trace_id().clone(),
                    "record_key".into(),
                )
                .map_err(HistoricalValidationError::Protection)?;
                Ok((
                    definition.trace_id().0.clone(),
                    (scope, definition.definition_hash().clone()),
                ))
            })
            .collect::<Result<_, HistoricalValidationError>>()?;
        Ok(Self {
            definitions,
            protector,
            identities: Vec::new(),
        })
    }
}
impl FlowRecordObserver for ProtectedCollector<'_> {
    fn on_record(
        &mut self,
        trace: &CompiledRecordSchema,
        record: &RawRecord<'_>,
        key: &JoinKey,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some((scope, definition)) = self.definitions.get(&trace.id().0) {
            self.identities.push(self.protector.protect_record_bytes(
                scope,
                key,
                definition,
                record.bytes,
            )?);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::source_time;
    use std::time::{Duration, UNIX_EPOCH};
    #[test]
    fn source_clock_handles_pre_epoch_fraction_and_rejects_out_of_calendar_range() {
        // 700ns is representable on both Unix and Windows' 100ns clock grid.
        let value = source_time(UNIX_EPOCH - Duration::new(1, 700)).unwrap();
        assert_eq!(
            (value.timestamp(), value.timestamp_subsec_nanos()),
            (-2, 999_999_300)
        );
        // checked_add is platform-specific; when the host can represent this
        // instant it must fail cleanly at the narrower chrono boundary.
        if let Some(time) = UNIX_EPOCH.checked_add(Duration::from_secs(10_000_000_000_000)) {
            assert!(source_time(time).is_none());
        }
    }
}
