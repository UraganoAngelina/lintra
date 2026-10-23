//! Bounded, declaration-ordered audit aggregation. No patient value is retained.
use crate::*;
use std::{collections::HashMap, error::Error, io};

pub(crate) struct ReferenceAuditCollector {
    audit: ReferenceValidationSummary,
    fields: HashMap<FieldId, Vec<Option<usize>>>,
    rules: HashMap<String, Vec<Option<usize>>>,
}
impl ReferenceAuditCollector {
    pub fn new(
        flow: &CompiledFlowSchema,
        trace: &CompiledRecordSchema,
        prepared: &PreparedReferenceEvidence,
        policy: DatasetPreparationPolicy,
    ) -> Result<Self, ReferencePreparationError> {
        let invalid = || ReferencePreparationError::InconsistentEvidence;
        if prepared.policy() != policy
            || prepared.datasets().len() != flow.required_reference_data().len()
        {
            return Err(invalid());
        }
        let mut datasets = Vec::new();
        for (dependency, evidence) in flow
            .required_reference_data()
            .iter()
            .zip(prepared.datasets())
        {
            if evidence.key() != &dependency.key
                || evidence.expected_sha256() != &Sha256Digest(dependency.sha256)
                || evidence.policy() != policy
            {
                return Err(invalid());
            }
            datasets.push(ReferenceDatasetAudit {
                requested: evidence.key().clone(),
                declared: evidence.declared_key().cloned(),
                relative_path: dependency.bundle_path.to_string(),
                expected_sha256: evidence.expected_sha256().clone(),
                actual_sha256: evidence.actual_sha256().cloned(),
                availability: evidence.availability(),
                diagnostics: evidence
                    .anomalies()
                    .iter()
                    .map(|anomaly| match anomaly {
                        DatasetAnomaly::FileType(issue) => {
                            ReferenceDatasetDiagnostic::FileType(*issue)
                        }
                        DatasetAnomaly::Io { operation, source } => {
                            ReferenceDatasetDiagnostic::Io {
                                operation: *operation,
                                error: match source.kind() {
                                    io::ErrorKind::NotFound => ReferenceIoKind::NotFound,
                                    io::ErrorKind::PermissionDenied => {
                                        ReferenceIoKind::PermissionDenied
                                    }
                                    _ => ReferenceIoKind::Other,
                                },
                            }
                        }
                        DatasetAnomaly::IntegrityMismatch => {
                            ReferenceDatasetDiagnostic::IntegrityMismatch
                        }
                        DatasetAnomaly::InvalidIdentity => {
                            ReferenceDatasetDiagnostic::InvalidIdentity
                        }
                        DatasetAnomaly::IdentityMismatch => {
                            ReferenceDatasetDiagnostic::IdentityMismatch
                        }
                        DatasetAnomaly::Format { issue, .. } => {
                            ReferenceDatasetDiagnostic::Format(issue.clone())
                        }
                    })
                    .collect(),
            });
        }
        let mut collector = Self {
            audit: ReferenceValidationSummary {
                policy,
                completeness: ValidationCompleteness::Complete,
                datasets,
                controls: vec![],
            },
            fields: HashMap::new(),
            rules: HashMap::new(),
        };
        for field in trace.fields() {
            let mut indices = vec![None; field.constraints().len()];
            for (index, constraint) in field.constraints().iter().enumerate() {
                indices[index] =
                    collector.add_control(trace, field.id(), None, index, constraint)?;
            }
            collector.fields.insert(field.id().clone(), indices);
        }
        for rule in trace.local_rules() {
            let mut indices = vec![None; rule.assertions.len()];
            for (index, assertion) in rule.assertions.iter().enumerate() {
                if let CompiledAssertion::Constraint {
                    field: ResolvedFieldRef::Local { field_id, .. },
                    constraint,
                } = assertion
                {
                    indices[index] = collector.add_control(
                        trace,
                        field_id,
                        Some(rule.id.to_string()),
                        index,
                        constraint,
                    )?;
                }
            }
            collector.rules.insert(rule.id.to_string(), indices);
        }
        // Validate source identity/integrity even before the first input is opened.
        if !collector.audit.valid(false) {
            return Err(invalid());
        }
        Ok(collector)
    }
    fn add_control(
        &mut self,
        trace: &CompiledRecordSchema,
        field_id: &FieldId,
        rule_id: Option<String>,
        index: usize,
        constraint: &CompiledConstraint,
    ) -> Result<Option<usize>, ReferencePreparationError> {
        let CompiledConstraint::ReferenceData {
            dataset_indexes,
            metadata_bindings,
            ..
        } = constraint
        else {
            return Ok(None);
        };
        if !metadata_bindings.is_empty()
            || dataset_indexes.is_empty()
            || dataset_indexes
                .iter()
                .any(|i| *i >= self.audit.datasets.len())
        {
            return Err(ReferencePreparationError::InconsistentEvidence);
        }
        let row = self.audit.controls.len();
        self.audit.controls.push(ReferenceControlSummary {
            trace_id: trace.id().clone(),
            field_id: field_id.clone(),
            rule_id,
            index,
            sources: dataset_indexes
                .iter()
                .map(|index| ReferenceSourceUsage {
                    dataset_index: *index,
                    consulted: 0,
                    selected_matches: 0,
                })
                .collect(),
            evaluations: ReferenceEvaluationCounts::default(),
        });
        Ok(Some(row))
    }
    pub fn into_audit(self) -> ReferenceValidationSummary {
        self.audit
    }
}
fn invalid() -> Box<dyn Error + Send + Sync> {
    Box::new(ReferencePreparationError::InconsistentEvidence)
}
fn increment(value: &mut u64) -> Result<(), Box<dyn Error + Send + Sync>> {
    *value = value.checked_add(1).ok_or_else(|| {
        Box::new(io::Error::other("reference audit counter overflow"))
            as Box<dyn Error + Send + Sync>
    })?;
    Ok(())
}
impl ReferenceEvaluationObserver for ReferenceAuditCollector {
    fn on_evaluation(
        &mut self,
        control: ReferenceControl<'_>,
        evaluation: ReferenceConstraintEvaluation<'_>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mapping = if let Some(id) = control.rule_id {
            self.rules.get(id)
        } else {
            self.fields.get(control.field_id)
        };
        let row = mapping
            .and_then(|map| map.get(control.index))
            .copied()
            .flatten()
            .ok_or_else(invalid)?;
        let summary = self.audit.controls.get_mut(row).ok_or_else(invalid)?;
        if &summary.field_id != control.field_id {
            return Err(invalid());
        }
        let counts = &mut summary.evaluations;
        let winner = match evaluation {
            ReferenceConstraintEvaluation::DeclaredFallback => {
                increment(&mut counts.definitive_satisfied)?;
                return Ok(());
            }
            ReferenceConstraintEvaluation::Membership(MembershipEvaluation::VerifiedPresent(
                evidence,
            )) => {
                increment(&mut counts.definitive_satisfied)?;
                Some((evidence.key(), true))
            }
            ReferenceConstraintEvaluation::Membership(MembershipEvaluation::VerifiedAbsent) => {
                increment(&mut counts.definitive_violated)?;
                None
            }
            ReferenceConstraintEvaluation::Membership(
                MembershipEvaluation::ProvisionalPresent(evidence),
            ) => {
                increment(&mut counts.provisional_present)?;
                Some((evidence.key(), false))
            }
            ReferenceConstraintEvaluation::Membership(MembershipEvaluation::Inconclusive) => {
                increment(&mut counts.inconclusive)?;
                None
            }
        };
        for source in &mut summary.sources {
            let dataset = self
                .audit
                .datasets
                .get(source.dataset_index)
                .ok_or_else(invalid)?;
            if dataset.availability != DatasetAvailability::Unavailable {
                increment(&mut source.consulted)?;
            }
            if winner.is_some_and(|(key, _)| key == &dataset.requested) {
                increment(&mut source.selected_matches)?;
                if winner.is_some_and(|(_, verified)| verified) {
                    break;
                }
            }
        }
        Ok(())
    }
}
