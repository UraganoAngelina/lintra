//! Full-run adaptation: neutral requests/parameters -> engine, engine evidence
//! -> persistent domain DTOs. No input read or normative decision is duplicated.
use super::reference_audit::ReferenceAuditCollector;
use crate::*;
use std::{collections::BTreeMap, error::Error, io};

pub(crate) fn bind_parameters(
    schema: &CompiledFlowSchema,
    supplied: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, JoinParameterValue>, ValidationRunError> {
    supplied
        .iter()
        .map(|(id, value)| {
            let definition = schema
                .parameters()
                .iter()
                .find(|parameter| match parameter {
                    ParameterDefinition::String { id: declared, .. }
                    | ParameterDefinition::Integer { id: declared, .. } => declared.as_ref() == id,
                });
            let value = match definition {
                Some(ParameterDefinition::String { .. }) => JoinParameterValue::Text(value.clone()),
                Some(ParameterDefinition::Integer { .. }) => {
                    JoinParameterValue::Integer(value.parse().map_err(|_| {
                        ValidationRunError::Flow(FlowValidationError::Inputs(
                            RelatedContextError::InvalidParameter,
                        ))
                    })?)
                }
                None => {
                    return Err(ValidationRunError::Flow(FlowValidationError::Inputs(
                        RelatedContextError::InvalidParameter,
                    )));
                }
            };
            Ok((id.clone(), value))
        })
        .collect()
}
pub(crate) struct FlowReferences {
    collectors: Vec<(TraceId, ReferenceAuditCollector)>,
}
impl FlowReferences {
    pub(crate) fn new(
        schema: &CompiledFlowSchema,
        traces: &[TraceId],
        prepared: &PreparedReferenceEvidence,
        policy: DatasetPreparationPolicy,
    ) -> Result<Self, ReferencePreparationError> {
        let collectors = traces
            .iter()
            .map(|id| {
                let trace = schema
                    .trace(id)
                    .ok_or(ReferencePreparationError::InconsistentEvidence)?;
                Ok((
                    id.clone(),
                    ReferenceAuditCollector::new(schema, trace, prepared, policy)?,
                ))
            })
            .collect::<Result<_, ReferencePreparationError>>()?;
        Ok(Self { collectors })
    }
    pub(crate) fn into_audit(
        self,
    ) -> Result<ReferenceValidationSummary, ReferencePreparationError> {
        let mut collectors = self.collectors.into_iter();
        let mut audit = collectors
            .next()
            .ok_or(ReferencePreparationError::InconsistentEvidence)?
            .1
            .into_audit();
        for (_, collector) in collectors {
            let other = collector.into_audit();
            if other.policy != audit.policy || other.datasets != audit.datasets {
                return Err(ReferencePreparationError::InconsistentEvidence);
            }
            audit.controls.extend(other.controls);
        }
        Ok(audit)
    }
}
impl FlowReferenceObservers for FlowReferences {
    fn for_trace(
        &mut self,
        trace: &TraceId,
    ) -> Result<&mut dyn ReferenceEvaluationObserver, Box<dyn Error + Send + Sync>> {
        match self.collectors.iter_mut().find(|(id, _)| id == trace) {
            Some((_, collector)) => Ok(collector),
            None => Err(Box::new(io::Error::other(
                "reference observer trace not prepared",
            ))),
        }
    }
}
fn uncertainty(reason: UnknownReason) -> Result<CrossUncertaintyReason, ValidationRunError> {
    Ok(match reason {
        UnknownReason::FieldUnavailable => CrossUncertaintyReason::FieldUnavailable,
        UnknownReason::RelatedRecordMissing => CrossUncertaintyReason::RelatedRecordMissing,
        UnknownReason::TraceInputUnavailable => CrossUncertaintyReason::TraceInputUnavailable,
        UnknownReason::AmbiguousJoin => CrossUncertaintyReason::AmbiguousJoin,
        UnknownReason::IncompleteRelatedContext => CrossUncertaintyReason::IncompleteRelatedContext,
        UnknownReason::InvalidFieldValue => CrossUncertaintyReason::InvalidFieldValue,
        UnknownReason::ReferenceProvisional => CrossUncertaintyReason::ReferenceProvisional,
        UnknownReason::ReferenceInconclusive => CrossUncertaintyReason::ReferenceInconclusive,
        UnknownReason::UnsupportedPlan => {
            return Err(ValidationRunError::Flow(FlowValidationError::InvalidPlan));
        }
    })
}
pub(crate) fn into_summary(
    artifacts: FlowValidationArtifacts,
    parameters: &BTreeMap<String, JoinParameterValue>,
) -> Result<FullValidationSummary, ValidationRunError> {
    let traces = artifacts
        .traces
        .into_iter()
        .zip(artifacts.record_keys)
        .map(|(trace, keys)| {
            if trace.trace_id != keys.trace_id {
                return Err(ValidationRunError::Flow(FlowValidationError::InvalidPlan));
            }
            Ok(FlowTraceEvidence {
                cross_rules_declared: u64::try_from(
                    artifacts
                        .cross
                        .iter()
                        .filter(|control| control.trace_id == trace.trace_id)
                        .count(),
                )
                .map_err(|_| ValidationRunError::Flow(FlowValidationError::InvalidPlan))?,
                trace_id: trace.trace_id,
                coverage: match trace.coverage {
                    TraceContextCoverage::Complete => ContextCoverage::Complete,
                    TraceContextCoverage::Partial => ContextCoverage::Partial,
                    TraceContextCoverage::Unavailable => ContextCoverage::Unavailable,
                },
                records_processed: trace.records_processed,
                unprojectable_records: trace.unprojectable_records,
                record_keys_checked: keys.checked,
                duplicate_record_keys: keys.duplicates,
                inconclusive_record_keys: keys.inconclusive,
            })
        })
        .collect::<Result<_, ValidationRunError>>()?;
    let controls = artifacts
        .cross
        .into_iter()
        .map(|control| {
            let causes = control
                .non_definitive
                .into_iter()
                .map(|cause| {
                    Ok(CrossEvaluationCause {
                        result: match cause.result {
                            NonDefinitiveCrossResult::Unknown(reason) => {
                                CrossCauseKind::Unknown(uncertainty(reason)?)
                            }
                            NonDefinitiveCrossResult::NotApplicable => {
                                CrossCauseKind::NotApplicable
                            }
                        },
                        join_id: cause.join_id.map(|id| id.0),
                        diagnostic_code: cause.diagnostic.as_ref().map(|d| d.code.to_string()),
                        message_template: cause.diagnostic.map(|d| d.message_template.to_string()),
                        evaluations: cause.evaluations,
                    })
                })
                .collect::<Result<_, ValidationRunError>>()?;
            Ok(CrossControlSummary {
                trace_id: control.trace_id,
                rule_id: control.rule_id,
                evaluations: control.counts.evaluations,
                satisfied: control.counts.satisfied,
                violations: control.counts.violations,
                not_applicable: control.counts.not_applicable,
                unknown: control.counts.unknown,
                skipped: control.skipped_evaluations,
                causes,
            })
        })
        .collect::<Result<_, ValidationRunError>>()?;
    let parameters = parameters
        .iter()
        .map(|(id, value)| {
            (
                id.clone(),
                match value {
                    JoinParameterValue::Text(value) => RunParameterAudit::String(value.clone()),
                    JoinParameterValue::Integer(value) => {
                        let mut text = String::new();
                        if value.is_negative() {
                            text.push('-');
                        }
                        text.extend(value.magnitude().iter().map(|byte| char::from(*byte)));
                        RunParameterAudit::Integer(text)
                    }
                },
            )
        })
        .collect();
    Ok(FullValidationSummary {
        completeness: ValidationCompleteness::Complete,
        parameters,
        traces,
        controls,
    })
}

pub(crate) struct FullServicePorts<'a> {
    pub sink: &'a mut dyn ValidationSink,
    pub progress: &'a mut dyn ProgressObserver,
    pub cancellation: &'a CancellationToken,
    pub started_at: std::time::SystemTime,
    pub resolved: &'a ResolvedSchema,
    pub record_observer: Option<&'a mut dyn crate::engine::flow_execution::FlowRecordObserver>,
}
