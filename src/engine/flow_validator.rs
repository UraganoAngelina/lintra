//! Pass-2 execution over owned projections. Input orchestration belongs to the
//! following FlowValidator task; this pass never reopens a source file.
use super::{
    cross_diagnostics::CrossDiagnosticInput,
    error_factory::ErrorFactory,
    expression_evaluator::EvaluatedTruth,
    projected_context::{ProjectedEvaluationContext, ProjectedFieldContext, validate_cross_inputs},
    rule_diagnostics::{RuleDiagnosticError, assertion_refs},
    rule_validator::{AssertionFailure, validate_assertion},
};
use crate::{
    ContextStore, CrossRecordInputs, EvaluationContext, ExpressionEvaluator, FieldValueResolver,
    FieldViolation, JoinId, ProjectedRecord, RelatedContextError, RelatedResolution, ReportError,
    ResolvedFieldValue, TraceId, TruthValue, UnknownReason, ValidationError, ValidationSink,
    schema::compiled::*,
};
use std::{error::Error, fmt};

pub enum CrossRuleOutcome<'a> {
    Satisfied,
    Violated {
        error: Box<ValidationError>,
        join_id: Option<&'a JoinId>,
    },
    NotApplicable {
        join_id: &'a JoinId,
        diagnostic: &'a RelatedDiagnostic,
    },
    Unknown {
        reason: UnknownReason,
        join_id: Option<&'a JoinId>,
        diagnostic: Option<&'a RelatedDiagnostic>,
    },
}
pub trait CrossRecordEvaluationObserver {
    /// One event per source-record/rule evaluation; implementations can aggregate
    /// by rule/cause without retaining errors or issuing one warning per record.
    fn on_evaluation(
        &mut self,
        trace_id: &TraceId,
        line_number: u64,
        rule_id: &str,
        outcome: &CrossRuleOutcome<'_>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CrossRecordPassSummary {
    pub evaluations: u64,
    pub satisfied: u64,
    pub violations: u64,
    pub not_applicable: u64,
    pub unknown: u64,
}
impl CrossRecordPassSummary {
    fn after(self, outcome: &CrossRuleOutcome<'_>) -> Result<Self, CrossRecordValidationError> {
        let mut next = self;
        next.evaluations = next
            .evaluations
            .checked_add(1)
            .ok_or(CrossRecordValidationError::CounterOverflow)?;
        let count = match outcome {
            CrossRuleOutcome::Satisfied => &mut next.satisfied,
            CrossRuleOutcome::Violated { .. } => &mut next.violations,
            CrossRuleOutcome::NotApplicable { .. } => &mut next.not_applicable,
            CrossRuleOutcome::Unknown { .. } => &mut next.unknown,
        };
        *count = count
            .checked_add(1)
            .ok_or(CrossRecordValidationError::CounterOverflow)?;
        Ok(next)
    }
}
#[derive(Debug)]
pub enum CrossRecordValidationError {
    InvalidPlan,
    Context(RelatedContextError),
    Diagnostic(RuleDiagnosticError),
    Reference(super::reference_execution::ReferenceExecutionError),
    Report(ReportError),
    Observer(Box<dyn Error + Send + Sync>),
    CounterOverflow,
}
impl fmt::Display for CrossRecordValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidPlan => "cross-record execution plan is inconsistent or unsupported",
            Self::Context(_) => "cross-record context preparation failed",
            Self::Diagnostic(_) => "cross-record diagnostic construction failed",
            Self::Reference(_) => "cross-record reference evaluation failed",
            Self::Report(_) => "cross-record report failed",
            Self::Observer(_) => "cross-record observation failed",
            Self::CounterOverflow => "cross-record counter overflow",
        })
    }
}
impl Error for CrossRecordValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Context(source) => Some(source),
            Self::Diagnostic(source) => Some(source),
            Self::Reference(source) => Some(source),
            Self::Report(source) => Some(source),
            Self::Observer(source) => Some(source.as_ref()),
            _ => None,
        }
    }
}

enum Decision<'a> {
    Passed,
    Failure {
        violation: FieldViolation,
        template: &'a str,
        join: Option<usize>,
    },
    NotApplicable {
        join_id: &'a JoinId,
        diagnostic: &'a RelatedDiagnostic,
    },
    Unknown {
        reason: UnknownReason,
        join_id: Option<&'a JoinId>,
        diagnostic: Option<&'a RelatedDiagnostic>,
    },
}
fn indeterminate<'a>(
    flow: &'a CompiledFlowSchema,
    value: EvaluatedTruth,
) -> Result<Decision<'a>, CrossRecordValidationError> {
    let TruthValue::Unknown(reason) = value.truth else {
        return Err(CrossRecordValidationError::InvalidPlan);
    };
    if reason == UnknownReason::UnsupportedPlan {
        return Err(CrossRecordValidationError::InvalidPlan);
    }
    let join = value
        .external_cause
        .map(|index| {
            flow.cross_record_plan()
                .joins
                .get(index)
                .ok_or(CrossRecordValidationError::InvalidPlan)
        })
        .transpose()?;
    let missing_policy = join.and_then(|join| match reason {
        UnknownReason::RelatedRecordMissing => Some(&join.on_missing_record),
        UnknownReason::TraceInputUnavailable => Some(&join.on_missing_trace),
        _ => None,
    });
    match (missing_policy, join) {
        (
            Some(RelatedPolicy::Violation {
                diagnostic,
                error_code,
            }),
            Some(_),
        ) => Ok(Decision::Failure {
            violation: FieldViolation {
                code: error_code.clone(),
                expected: Some("record correlato richiesto".into()),
                reason: String::new(),
                diagnostic_context: vec![],
            },
            template: &diagnostic.message_template,
            join: value.external_cause,
        }),
        (Some(RelatedPolicy::NotApplicable { diagnostic }), Some(join)) => {
            Ok(Decision::NotApplicable {
                join_id: &join.id,
                diagnostic,
            })
        }
        (Some(RelatedPolicy::Unknown { diagnostic }), Some(join)) => Ok(Decision::Unknown {
            reason,
            join_id: Some(&join.id),
            diagnostic: Some(diagnostic),
        }),
        _ => Ok(Decision::Unknown {
            reason,
            join_id: join.map(|join| &join.id),
            diagnostic: join
                .filter(|join| {
                    reason == UnknownReason::AmbiguousJoin
                        && matches!(
                            join.cardinality,
                            JoinCardinality::OneOrMany | JoinCardinality::ZeroOrMany
                        )
                })
                .and_then(|join| join.consensus.as_ref())
                .map(|consensus| &consensus.conflict_diagnostic),
        }),
    }
}
fn cause_for_assertion(
    assertion: &CompiledAssertion,
    schema: &CompiledRecordSchema,
    context: &ProjectedEvaluationContext<'_>,
    reason: UnknownReason,
) -> Result<Option<usize>, CrossRecordValidationError> {
    if !matches!(
        reason,
        UnknownReason::RelatedRecordMissing
            | UnknownReason::TraceInputUnavailable
            | UnknownReason::AmbiguousJoin
    ) {
        return Ok(None);
    }
    let mut references = Vec::new();
    assertion_refs(assertion, schema, &mut references)
        .map_err(CrossRecordValidationError::Diagnostic)?;
    for reference in references {
        if let ResolvedFieldRef::External { join_index, .. } = reference
            && (match assertion {
                CompiledAssertion::Constraint {
                    constraint:
                        CompiledConstraint::AllowedValues {
                            target: ValueTarget::Raw,
                            ..
                        }
                        | CompiledConstraint::Regex {
                            target: ValueTarget::Raw,
                            ..
                        },
                    ..
                } => context.resolve_raw_field(reference),
                _ => context.resolve_field(reference),
            }) == ResolvedFieldValue::Unavailable(reason)
        {
            return Ok(Some(*join_index));
        }
    }
    Ok(None)
}
fn existence<'a>(
    flow: &'a CompiledFlowSchema,
    context: &ProjectedEvaluationContext<'_>,
    join_index: usize,
    predicate: Option<&CompiledExpr>,
    rule: &'a CompiledRule,
) -> Result<Decision<'a>, CrossRecordValidationError> {
    let lookup = context
        .related(join_index)
        .ok_or(CrossRecordValidationError::InvalidPlan)?;
    let records = match &lookup.resolution {
        RelatedResolution::Unavailable(reason) => {
            return indeterminate(
                flow,
                EvaluatedTruth {
                    truth: TruthValue::Unknown(*reason),
                    external_cause: Some(join_index),
                },
            );
        }
        RelatedResolution::Records(records) => records,
    };
    if records.is_empty() {
        return indeterminate(
            flow,
            EvaluatedTruth {
                truth: TruthValue::Unknown(UnknownReason::RelatedRecordMissing),
                external_cause: Some(join_index),
            },
        );
    }
    if matches!(
        lookup.join.cardinality,
        JoinCardinality::ZeroOrOne | JoinCardinality::ExactlyOne
    ) && records.len() > 1
    {
        return indeterminate(
            flow,
            EvaluatedTruth {
                truth: TruthValue::Unknown(UnknownReason::AmbiguousJoin),
                external_cause: Some(join_index),
            },
        );
    }
    let Some(predicate) = predicate else {
        return Ok(Decision::Passed);
    };
    let target = flow
        .traces()
        .get(lookup.join.target_trace)
        .ok_or(CrossRecordValidationError::InvalidPlan)?;
    let mut unknown = None;
    for record in records.iter() {
        let projected = ProjectedFieldContext {
            schema: target,
            record,
        };
        match ExpressionEvaluator.evaluate(predicate, &EvaluationContext { fields: &projected }) {
            TruthValue::True => return Ok(Decision::Passed),
            TruthValue::False => {}
            TruthValue::Unknown(UnknownReason::UnsupportedPlan) => {
                return Err(CrossRecordValidationError::InvalidPlan);
            }
            TruthValue::Unknown(reason) => {
                unknown.get_or_insert(reason);
            }
        }
    }
    if let Some(reason) = unknown {
        return Ok(Decision::Unknown {
            reason,
            join_id: Some(&flow.cross_record_plan().joins[join_index].id),
            diagnostic: None,
        });
    }
    Ok(Decision::Failure {
        violation: FieldViolation {
            code: rule.error_code.clone(),
            expected: Some("almeno un record correlato che soddisfi il predicato".into()),
            reason: String::new(),
            diagnostic_context: vec![],
        },
        template: &rule.message_template,
        join: None,
    })
}
fn assertion<'a>(
    flow: &'a CompiledFlowSchema,
    rule: &'a CompiledRule,
    index: usize,
    source_schema: &CompiledRecordSchema,
    context: &ProjectedEvaluationContext<'_>,
) -> Result<Decision<'a>, CrossRecordValidationError> {
    let current = rule
        .assertions
        .get(index)
        .ok_or(CrossRecordValidationError::InvalidPlan)?;
    let fields = EvaluationContext { fields: context };
    match current {
        CompiledAssertion::RecordExists { join_index } => {
            existence(flow, context, *join_index, None, rule)
        }
        CompiledAssertion::SetExists {
            join_index,
            predicate,
        } => existence(flow, context, *join_index, Some(predicate), rule),
        CompiledAssertion::ConditionTrue(condition) => {
            let result = ExpressionEvaluator.evaluate_with_cause(condition, &fields);
            match result.truth {
                TruthValue::True => Ok(Decision::Passed),
                TruthValue::False => Ok(Decision::Failure {
                    violation: FieldViolation {
                        code: rule.error_code.clone(),
                        expected: Some("condizione della regola soddisfatta".into()),
                        reason: String::new(),
                        diagnostic_context: vec![],
                    },
                    template: &rule.message_template,
                    join: None,
                }),
                TruthValue::Unknown(_) => indeterminate(flow, result),
            }
        }
        _ => match validate_assertion(current, &fields, None, rule, index) {
            Ok(()) => Ok(Decision::Passed),
            Err(AssertionFailure::Violated(mut violation)) => {
                violation.code = rule.error_code.clone();
                Ok(Decision::Failure {
                    violation,
                    template: &rule.message_template,
                    join: None,
                })
            }
            Err(AssertionFailure::Unknown(reason)) => indeterminate(
                flow,
                EvaluatedTruth {
                    truth: TruthValue::Unknown(reason),
                    external_cause: cause_for_assertion(current, source_schema, context, reason)?,
                },
            ),
            Err(AssertionFailure::Reference(error)) => {
                Err(CrossRecordValidationError::Reference(error))
            }
        },
    }
}
fn evaluate_rule<'a>(
    flow: &'a CompiledFlowSchema,
    source: &ProjectedRecord,
    rule: &'a CompiledRule,
    context: &ProjectedEvaluationContext<'_>,
) -> Result<CrossRuleOutcome<'a>, CrossRecordValidationError> {
    let schema = flow
        .trace(&source.trace_id)
        .ok_or(CrossRecordValidationError::InvalidPlan)?;
    let when =
        ExpressionEvaluator.evaluate_with_cause(&rule.when, &EvaluationContext { fields: context });
    if when.truth == TruthValue::False {
        return Ok(CrossRuleOutcome::Satisfied);
    }
    let mut failure = None;
    let mut unknown = None;
    let first = match when.truth {
        TruthValue::True => None,
        TruthValue::Unknown(_) => Some(indeterminate(flow, when)?),
        TruthValue::False => None,
    };
    let count = if first.is_some() {
        1
    } else {
        rule.assertions.len()
    };
    let mut first = first;
    for index in 0..count {
        let from_when = first.is_some();
        let decision = match first.take() {
            Some(decision) => decision,
            None => assertion(flow, rule, index, schema, context)?,
        };
        match decision {
            Decision::Passed => {}
            Decision::NotApplicable {
                join_id,
                diagnostic,
            } => {
                return Ok(CrossRuleOutcome::NotApplicable {
                    join_id,
                    diagnostic,
                });
            }
            Decision::Unknown {
                reason,
                join_id,
                diagnostic,
            } => {
                unknown.get_or_insert(CrossRuleOutcome::Unknown {
                    reason,
                    join_id,
                    diagnostic,
                });
            }
            Decision::Failure {
                violation,
                template,
                join,
            } => {
                failure.get_or_insert((
                    if from_when { None } else { Some(index) },
                    violation,
                    template,
                    join,
                ));
            }
        }
    }
    if let Some((index, violation, template, join)) = failure {
        let factory = ErrorFactory {
            flow_id: &flow.key().flow_id,
            schema_version: &flow.key().version,
            trace_id: schema.id(),
            encoding: flow.encoding(),
        };
        let error = factory
            .cross_rule_error(
                CrossDiagnosticInput {
                    flow,
                    source,
                    rule,
                    assertion_index: index,
                    context,
                    missing_join: join,
                    template,
                },
                violation,
            )
            .map_err(CrossRecordValidationError::Diagnostic)?;
        return Ok(CrossRuleOutcome::Violated {
            error: Box::new(error),
            join_id: join.map(|index| &flow.cross_record_plan().joins[index].id),
        });
    }
    Ok(unknown.unwrap_or(CrossRuleOutcome::Satisfied))
}

pub fn validate_cross_record_rules(
    flow: &CompiledFlowSchema,
    store: &dyn ContextStore,
    inputs: CrossRecordInputs<'_>,
    sink: &mut dyn ValidationSink,
    observer: &mut dyn CrossRecordEvaluationObserver,
) -> Result<CrossRecordPassSummary, CrossRecordValidationError> {
    validate_cross_inputs(flow, &inputs).map_err(CrossRecordValidationError::Context)?;
    let mut totals = CrossRecordPassSummary::default();
    // A missing source input cannot be mistaken for an available empty file.
    for plan in &flow.cross_record_plan().rules {
        let source = flow
            .traces()
            .get(plan.source_trace)
            .ok_or(CrossRecordValidationError::InvalidPlan)?;
        if !inputs.available_traces.contains(source.id()) {
            return Err(CrossRecordValidationError::Context(
                RelatedContextError::SourceInputUnavailable,
            ));
        }
        for join in flow
            .cross_record_plan()
            .joins
            .iter()
            .filter(|join| join.source_trace == plan.source_trace)
        {
            for &index in &join.namespace_parameters {
                let parameter = flow
                    .parameters()
                    .get(index)
                    .ok_or(CrossRecordValidationError::InvalidPlan)?;
                let id = match parameter {
                    ParameterDefinition::String { id, .. }
                    | ParameterDefinition::Integer { id, .. } => id,
                };
                if !inputs.parameters.contains_key(id.as_ref()) {
                    return Err(CrossRecordValidationError::Context(
                        RelatedContextError::MissingNamespaceParameter,
                    ));
                }
            }
        }
    }
    for (trace_index, trace) in flow.traces().iter().enumerate() {
        let rules = flow
            .cross_record_plan()
            .rules
            .iter()
            .filter(|plan| plan.source_trace == trace_index)
            .collect::<Vec<_>>();
        if rules.is_empty() {
            continue;
        }
        for record in store.iter_trace(trace.id()) {
            if &record.trace_id != trace.id() {
                return Err(CrossRecordValidationError::InvalidPlan);
            }
            let context = ProjectedEvaluationContext::new(
                flow,
                record,
                store,
                CrossRecordInputs {
                    available_traces: inputs.available_traces,
                    parameters: inputs.parameters,
                },
            )
            .map_err(CrossRecordValidationError::Context)?;
            for plan in &rules {
                let outcome = evaluate_rule(flow, record, &plan.rule, &context)?;
                let next = totals.after(&outcome)?;
                if let CrossRuleOutcome::Violated { error, .. } = &outcome {
                    sink.on_error(error)
                        .map_err(CrossRecordValidationError::Report)?;
                }
                observer
                    .on_evaluation(trace.id(), record.line_number, &plan.rule.id, &outcome)
                    .map_err(CrossRecordValidationError::Observer)?;
                totals = next;
            }
        }
    }
    Ok(totals)
}
