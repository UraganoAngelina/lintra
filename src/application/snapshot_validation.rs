//! Explicit executable snapshot workflow. The ordinary audit/CLI paths retain
//! their original scope; this path uses the same input/run and approval ports.
use super::*;
use crate::application::history_existence::HistoryCollectorSetup;
use crate::application::history_existence::{HistoricalOriginAttestation, HistoryCollector};
use sha2::{Digest, Sha256};

pub struct HistoricalSnapshotValidationRequest {
    pub validation: HistoricalValidationRequest,
    pub proceed_consent: Option<MissingTraceProceedConsent>,
    pub origins: Vec<HistoricalOriginAttestation>,
}
/// Origin information to present in the actual approval review. Entity and row
/// values remain protected; the user sees the source/responsibility they attest.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HistoricalOriginReview {
    pub predicate: Sha256Digest,
    pub entity: Fingerprint,
    pub year: u16,
    pub quarter: u8,
    pub evidence_sha256: Sha256Digest,
    pub reference: String,
    pub responsible: String,
    pub recorded_at: String,
}

impl<'repo> HistoricalValidationService<'_, 'repo> {
    pub fn validate_snapshot(
        &self,
        request: HistoricalSnapshotValidationRequest,
        sink: &mut dyn ValidationSink,
        audit_sink: &mut dyn HistoricalAuditSink,
        progress: &mut dyn ProgressObserver,
        cancellation: &CancellationToken,
    ) -> Result<HistoricalValidationOutcome<'repo>, HistoricalValidationError> {
        use HistoricalValidationError as E;
        checkpoint(cancellation)?;
        let HistoricalSnapshotValidationRequest {
            validation: request,
            proceed_consent,
            origins,
        } = request;
        if request.validation.mode != ValidationMode::Full
            || !matches!(request.validation.scope, ValidationScope::Flow(_))
            || request.submission_id.trim().is_empty()
            || request.submission_id.trim() != request.submission_id
        {
            return Err(E::InvalidRequest);
        }
        let resolved = self
            .validation
            .schema_registry
            .resolve_historical_snapshot(&request.validation.schema_key)
            .map_err(E::Resolution)?;
        let profile = resolved.profile();
        if profile.policy().is_none_or(|p| {
            p.cross_run_audit.is_empty() && p.history_identity_invariants.is_empty()
        }) {
            return Err(E::InvalidRequest);
        }
        let schema = resolved.resolved().schema();
        let (namespace, period) = crate::application::session::bind_snapshot_context(
            profile,
            &request.parameters,
            &request.submission_id,
        )
        .map_err(E::Approval)?;
        let ValidationScope::Flow(input) = &request.validation.scope else {
            return Err(E::InvalidRequest);
        };
        let mut absent = profile
            .trace_ids()
            .iter()
            .filter(|id| !input.traces.iter().any(|t| &t.trace_id == *id))
            .cloned()
            .collect::<Vec<_>>();
        absent.sort_by(|a, b| a.0.cmp(&b.0));
        match (&proceed_consent, absent.is_empty()) {
            (None, false) => {
                return Err(E::Approval(SessionApprovalError::Rejected(
                    SessionRejection::MissingProceedConsent,
                )));
            }
            (Some(consent), _) if consent.traces() != absent => {
                return Err(E::Approval(SessionApprovalError::InvalidBinding));
            }
            _ => {}
        }
        let mut collector = HistoryCollector::new(
            HistoryCollectorSetup {
                schema,
                profile,
                namespace: namespace.clone(),
                period,
                repository: self.repository,
                protector: self.protector,
                cancellation,
            },
            origins,
            &request.submission_id,
        )?;
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
        // Unknown/skipped rules never acquire an approval capability, even with
        // PROCEED. Missing trace consent cannot manufacture historical coverage.
        if outcome.status != RunStatus::CompletedValid
            || outcome
                .summary
                .full_validation
                .as_ref()
                .is_none_or(|f| f.has_non_definitive_evaluations())
            || outcome
                .summary
                .reference_validation
                .as_ref()
                .is_some_and(|r| r.completeness != ValidationCompleteness::Complete)
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
            .map(|id| {
                let Some(input) = summary.inputs.iter().find(|i| &i.trace_id == id) else {
                    return Ok(SubmissionTrace {
                        trace_id: id.clone(),
                        presence: TracePresence::Absent,
                        input_sha256: None,
                    });
                };
                let InputAuditCompletion::Complete { sha256 } = &input.completion else {
                    return Err(E::InvalidRequest);
                };
                let trace = full
                    .traces
                    .iter()
                    .find(|t| &t.trace_id == id)
                    .ok_or(E::InvalidRequest)?;
                Ok(SubmissionTrace {
                    trace_id: id.clone(),
                    presence: TracePresence::Present {
                        record_count: trace.records_processed,
                    },
                    input_sha256: Some(sha256.clone()),
                })
            })
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
        let draft = ApprovalDraft::new(profile, request.parameters, parts, proceed_consent)
            .map_err(E::Approval)?;
        let projections = std::mem::take(&mut collector.projections);
        let qualify = |snapshot: &HistoricalSnapshot| collector.attach(snapshot);
        let approval = if profile
            .policy()
            .is_some_and(|p| !p.history_identity_invariants.is_empty())
        {
            draft
                .validated_with_identity_records(&outcome, projections, self.protector)
                .map_err(E::Approval)?
                .preview_qualified(self.repository, cancellation, &qualify)
                .map_err(E::Approval)?
        } else {
            let identities = projections.into_iter().flat_map(|p| p.tokens).collect();
            draft
                .validated(&outcome, identities, self.protector)
                .map_err(E::Approval)?
                .preview_qualified(self.repository, cancellation, &qualify)
                .map_err(E::Approval)?
        };
        let service = if profile
            .policy()
            .is_some_and(|p| p.cross_run_audit.is_empty())
        {
            HistoricalAuditService::for_identity_invariants(
                profile,
                self.protector,
                self.repository,
            )
        } else {
            HistoricalAuditService::new(profile, self.protector, self.repository)
        };
        let prepared = service
            .prepare(approval.snapshot(), cancellation)
            .map_err(E::Audit)?;
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
        let manifest = crate::persistence::history::SnapshotManifest::create(approval.candidate())?;
        let canonical = crate::schema::canonical::canonical_value_bytes(
            serde_json::to_value(manifest).map_err(|_| PersistenceError::CorruptStorage)?,
        )
        .map_err(|_| PersistenceError::CorruptStorage)?;
        let history_commitment = Some(Sha256Digest(Sha256::digest(canonical).into()));
        let origins = approval
            .candidate()
            .history_facts()
            .iter()
            .flat_map(|batch| {
                batch
                    .payload
                    .origins
                    .iter()
                    .map(|o| HistoricalOriginReview {
                        predicate: batch.plan_digest().clone(),
                        entity: o.entity.clone(),
                        year: o.from.year,
                        quarter: o.from.quarter,
                        evidence_sha256: o.evidence_sha256.clone(),
                        reference: o.evidence_reference.clone(),
                        responsible: o.responsible.clone(),
                        recorded_at: o.recorded_at.clone(),
                    })
            })
            .collect();
        checkpoint(cancellation)?;
        sink.on_summary(&outcome.summary).map_err(E::Report)?;
        progress.on_event(ProgressEvent::RunCompleted);
        Ok(HistoricalValidationOutcome::Preview(Box::new(
            HistoricalValidationPreview {
                outcome,
                audit,
                approval,
                history_commitment,
                origins,
            },
        )))
    }
}
