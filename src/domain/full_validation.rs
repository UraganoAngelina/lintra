//! Persistent Full-run evidence, independent of engine execution types.
use super::{ScopeSummary, TraceId, ValidationCompleteness};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextCoverage {
    Complete,
    Partial,
    Unavailable,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FlowTraceEvidence {
    pub trace_id: TraceId,
    pub coverage: ContextCoverage,
    pub records_processed: u64,
    pub unprojectable_records: u64,
    /// Distinct compiled rules, unlike per-record evaluation counters.
    pub cross_rules_declared: u64,
    pub record_keys_checked: u64,
    pub duplicate_record_keys: u64,
    pub inconclusive_record_keys: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CrossUncertaintyReason {
    FieldUnavailable,
    RelatedRecordMissing,
    TraceInputUnavailable,
    AmbiguousJoin,
    IncompleteRelatedContext,
    InvalidFieldValue,
    ReferenceProvisional,
    ReferenceInconclusive,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "reason", rename_all = "snake_case")]
pub enum CrossCauseKind {
    Unknown(CrossUncertaintyReason),
    NotApplicable,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CrossEvaluationCause {
    pub result: CrossCauseKind,
    pub join_id: Option<String>,
    pub diagnostic_code: Option<String>,
    pub message_template: Option<String>,
    pub evaluations: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CrossControlSummary {
    pub trace_id: TraceId,
    pub rule_id: String,
    pub evaluations: u64,
    pub satisfied: u64,
    pub violations: u64,
    pub not_applicable: u64,
    pub unknown: u64,
    pub skipped: u64,
    pub causes: Vec<CrossEvaluationCause>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum RunParameterAudit {
    String(String),
    Integer(String),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FullValidationSummary {
    pub completeness: ValidationCompleteness,
    pub parameters: BTreeMap<String, RunParameterAudit>,
    pub traces: Vec<FlowTraceEvidence>,
    pub controls: Vec<CrossControlSummary>,
}
impl FullValidationSummary {
    pub fn has_non_definitive_evaluations(&self) -> bool {
        self.controls
            .iter()
            .any(|control| control.unknown > 0 || control.skipped > 0)
    }
    pub(crate) fn partial(&self, scope: &ScopeSummary) -> bool {
        self.has_non_definitive_evaluations() || self.traces.iter().any(|trace| {
            matches!(scope,ScopeSummary::Flow {trace_ids} if trace_ids.contains(&trace.trace_id))
                && (trace.coverage != ContextCoverage::Complete
                    || trace.inconclusive_record_keys > 0)
        })
    }
    pub(crate) fn valid(&self, summary: &super::ValidationSummary) -> bool {
        let ScopeSummary::Flow { trace_ids } = &summary.scope_summary else {
            return false;
        };
        let interrupted = summary.run_status == super::RunStatus::Cancelled || summary.truncated;
        if summary
            .inputs
            .iter()
            .map(|input| &input.trace_id)
            .collect::<HashSet<_>>()
            .len()
            != summary.inputs.len()
        {
            return false;
        }
        let mut trace_set = HashSet::new();
        let mut records = 0u64;
        let mut normative = 0u64;
        for trace in &self.traces {
            let input = summary
                .inputs
                .iter()
                .find(|input| input.trace_id == trace.trace_id);
            if (trace.records_processed > 0 && input.is_none())
                || (trace.coverage == ContextCoverage::Complete
                    && !input.is_some_and(|input| {
                        matches!(
                            input.completion,
                            super::InputAuditCompletion::Complete { .. }
                        )
                    }))
            {
                return false;
            }

            if trace.trace_id.0.is_empty()
                || !trace_set.insert(&trace.trace_id)
                || trace.unprojectable_records > trace.records_processed
                || trace.duplicate_record_keys > trace.record_keys_checked
                || trace
                    .record_keys_checked
                    .checked_add(trace.inconclusive_record_keys)
                    != Some(trace.records_processed)
                || trace.inconclusive_record_keys < trace.unprojectable_records
            {
                return false;
            }
            if !trace_ids.contains(&trace.trace_id)
                && (trace.records_processed != 0 || trace.coverage != ContextCoverage::Unavailable)
            {
                return false;
            }
            if trace.coverage == ContextCoverage::Unavailable && trace.records_processed != 0 {
                return false;
            }
            if trace.coverage == ContextCoverage::Complete && trace.unprojectable_records != 0 {
                return false;
            }
            if !interrupted
                && trace_ids.contains(&trace.trace_id)
                && (trace.coverage == ContextCoverage::Unavailable
                    || (trace.coverage == ContextCoverage::Partial
                        && trace.unprojectable_records == 0))
            {
                return false;
            }
            let Some(next) = records.checked_add(trace.records_processed) else {
                return false;
            };
            records = next;
            let Some(next) = normative.checked_add(trace.duplicate_record_keys) else {
                return false;
            };
            normative = next;
        }
        for trace in &self.traces {
            if u64::try_from(
                self.controls
                    .iter()
                    .filter(|control| control.trace_id == trace.trace_id)
                    .count(),
            )
            .ok()
                != Some(trace.cross_rules_declared)
            {
                return false;
            }
        }
        if records != summary.records_processed
            || trace_ids.iter().any(|trace| !trace_set.contains(trace))
            || trace_ids.iter().collect::<HashSet<_>>().len() != trace_ids.len()
        {
            return false;
        }
        let mut controls = HashSet::new();
        let mut skipped = 0u64;
        for control in &self.controls {
            let Some(trace) = self
                .traces
                .iter()
                .find(|trace| trace.trace_id == control.trace_id)
            else {
                return false;
            };
            if !trace_ids.contains(&control.trace_id)
                || control.rule_id.is_empty()
                || !controls.insert((&control.trace_id, &control.rule_id))
                || control.evaluations.checked_add(control.skipped) != Some(trace.records_processed)
                || [
                    control.satisfied,
                    control.violations,
                    control.not_applicable,
                    control.unknown,
                ]
                .into_iter()
                .try_fold(0u64, |sum, count| sum.checked_add(count))
                    != Some(control.evaluations)
            {
                return false;
            }
            let Some(next) = skipped.checked_add(control.skipped) else {
                return false;
            };
            skipped = next;
            let Some(next) = normative.checked_add(control.violations) else {
                return false;
            };
            normative = next;
            let mut unknown = 0u64;
            let mut na = 0u64;
            for (index, cause) in control.causes.iter().enumerate() {
                if cause.evaluations == 0
                    || cause.join_id.as_ref().is_some_and(|id| id.is_empty())
                    || cause
                        .diagnostic_code
                        .as_ref()
                        .is_some_and(|code| code.is_empty())
                    || cause
                        .message_template
                        .as_ref()
                        .is_some_and(|text| text.is_empty())
                    || cause.diagnostic_code.is_some() != cause.message_template.is_some()
                    || control.causes[..index].iter().any(|old| {
                        old.result == cause.result
                            && old.join_id == cause.join_id
                            && old.diagnostic_code == cause.diagnostic_code
                    })
                {
                    return false;
                }
                let count = match cause.result {
                    CrossCauseKind::Unknown(_) => &mut unknown,
                    CrossCauseKind::NotApplicable => {
                        if cause.join_id.is_none() || cause.diagnostic_code.is_none() {
                            return false;
                        }
                        &mut na
                    }
                };
                let Some(next) = count.checked_add(cause.evaluations) else {
                    return false;
                };
                *count = next;
            }
            if unknown != control.unknown || na != control.not_applicable {
                return false;
            }
        }
        let partial = interrupted
            || self.partial(&summary.scope_summary)
            || summary
                .reference_validation
                .as_ref()
                .is_some_and(|audit| audit.has_non_definitive_evaluations());
        if skipped != summary.skipped_rules
            || normative > summary.total_errors
            || self.completeness
                != if partial {
                    ValidationCompleteness::Partial
                } else {
                    ValidationCompleteness::Complete
                }
        {
            return false;
        }
        self.parameters.iter().all(|(id, value)| {
            !id.is_empty()
                && match value {
                    RunParameterAudit::String(_) => true,
                    RunParameterAudit::Integer(value) => {
                        let digits = value.strip_prefix('-').unwrap_or(value);
                        !digits.is_empty()
                            && digits.bytes().all(|b| b.is_ascii_digit())
                            && (digits == "0" || !digits.starts_with('0'))
                            && value != "-0"
                    }
                }
        })
    }
}
