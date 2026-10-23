//! Current-submission join declarations. Full execution remains a later task.
use super::*;
use crate::JoinId;

pub(super) fn compile_joins(raw: &RawFlowSchema) -> Result<Vec<CompiledJoinSpec>, (Code, String)> {
    let mut ids = HashSet::new();
    let mut joins = vec![];
    for (source_index, source) in raw.traces.iter().enumerate() {
        for join in &source.joins {
            let invalid = |message: &str| (Code::InvalidSchemaValue, message.to_owned());
            if join.id.is_empty() || join.id.trim() != join.id || !ids.insert(&join.id) {
                return Err(invalid(
                    "join IDs must be nonempty, unpadded and flow-unique",
                ));
            }
            if !matches!(join.scope, RawJoinSpecScope::CurrentSubmission) {
                return Err((
                    Code::UnsupportedConstruct,
                    "join history requires the persistent execution contract".into(),
                ));
            }
            if join
                .namespace_parameters
                .iter()
                .collect::<HashSet<_>>()
                .len()
                != join.namespace_parameters.len()
            {
                return Err(invalid("join namespace parameters must be unique"));
            }
            let namespace_parameters = join
                .namespace_parameters
                .iter()
                .map(|id| {
                    raw.parameters
                        .iter()
                        .position(|parameter| match parameter {
                            RawParameter::String {
                                id: parameter_id, ..
                            }
                            | RawParameter::Integer {
                                id: parameter_id, ..
                            } => parameter_id == id,
                        })
                        .ok_or((
                            Code::MissingParameterReference,
                            "join namespace parameter is not declared".into(),
                        ))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let target_index = raw
                .traces
                .iter()
                .position(|target| target.id == join.target_trace)
                .ok_or((
                    Code::MissingTraceReference,
                    "join target trace does not exist".into(),
                ))?;
            let target = &raw.traces[target_index];
            if join.source_fields.is_empty()
                || join.source_fields.len() != join.target_fields.len()
                || join.source_fields.iter().collect::<HashSet<_>>().len()
                    != join.source_fields.len()
                || join.target_fields.iter().collect::<HashSet<_>>().len()
                    != join.target_fields.len()
            {
                return Err(invalid(
                    "join fields must be nonempty ordered unique lists of equal arity",
                ));
            }
            let source_fields = join
                .source_fields
                .iter()
                .map(|id| index(source, id))
                .collect::<Result<Vec<_>, _>>()?;
            let target_fields = join
                .target_fields
                .iter()
                .map(|id| index(target, id))
                .collect::<Result<Vec<_>, _>>()?;
            let cardinality = match join.cardinality {
                RawJoinSpecCardinality::ZeroOrOne => JoinCardinality::ZeroOrOne,
                RawJoinSpecCardinality::ExactlyOne => JoinCardinality::ExactlyOne,
                RawJoinSpecCardinality::OneOrMany => JoinCardinality::OneOrMany,
                RawJoinSpecCardinality::ZeroOrMany => JoinCardinality::ZeroOrMany,
            };
            let on_missing_record = policy(&join.on_missing_record)?;
            let on_missing_trace = policy(&join.on_missing_trace)?;
            if matches!(
                cardinality,
                JoinCardinality::ExactlyOne | JoinCardinality::OneOrMany
            ) && (matches!(on_missing_record, RelatedPolicy::NotApplicable { .. })
                || matches!(on_missing_trace, RelatedPolicy::NotApplicable { .. }))
            {
                return Err(invalid(
                    "minimum-one join cannot make a missing required relation not applicable",
                ));
            }
            if !raw
                .required_capabilities
                .contains(&RawFlowSchemaRequiredCapabilitiesItem::JoinsV1)
            {
                return Err((
                    Code::MissingCapability,
                    "join declaration requires joins.v1".into(),
                ));
            }
            if matches!(
                cardinality,
                JoinCardinality::OneOrMany | JoinCardinality::ZeroOrMany
            ) && !raw
                .required_capabilities
                .contains(&RawFlowSchemaRequiredCapabilitiesItem::JoinsSetsV1)
            {
                return Err((
                    Code::MissingCapability,
                    "set cardinality requires joins.sets.v1".into(),
                ));
            }
            let consensus = join
                .reduction
                .as_ref()
                .map(|reduction| {
                    if !raw
                        .required_capabilities
                        .contains(&RawFlowSchemaRequiredCapabilitiesItem::JoinsConsensusV1)
                    {
                        return Err((
                            Code::MissingCapability,
                            "consensus requires joins.consensus.v1".into(),
                        ));
                    }
                    if reduction.fields.is_empty()
                        || reduction.fields.iter().collect::<HashSet<_>>().len()
                            != reduction.fields.len()
                    {
                        return Err(invalid("consensus fields must be nonempty and unique"));
                    }
                    Ok(ConsensusPlan {
                        fields: reduction
                            .fields
                            .iter()
                            .map(|id| index(target, id))
                            .collect::<Result<_, _>>()?,
                        include_if: crate::schema::expression::compile_local_expr(
                            &reduction.include_if,
                            target,
                        )?,
                        conflict_diagnostic: diagnostic(&reduction.diagnostic)?,
                    })
                })
                .transpose()?;
            joins.push(CompiledJoinSpec {
                id: JoinId(join.id.clone()),
                source_trace: source_index,
                target_trace: target_index,
                source_fields,
                target_fields,
                cardinality,
                scope: JoinScope::CurrentSubmission,
                namespace_parameters,
                on_missing_record,
                on_missing_trace,
                consensus,
            });
        }
    }
    Ok(joins)
}
fn policy(raw: &RawRelatedPolicy) -> Result<RelatedPolicy, (Code, String)> {
    let diagnostic = diagnostic(&raw.diagnostic)?;
    Ok(match (&raw.policy, &raw.error_code) {
        (RawRelatedPolicyPolicy::Violation, Some(code)) => RelatedPolicy::Violation {
            diagnostic,
            error_code: code.parse().map_err(|_| {
                (
                    Code::InvalidDiagnosticCode,
                    "invalid related violation code".into(),
                )
            })?,
        },
        (RawRelatedPolicyPolicy::NotApplicable, None) => {
            RelatedPolicy::NotApplicable { diagnostic }
        }
        (RawRelatedPolicyPolicy::Unknown, None) => RelatedPolicy::Unknown { diagnostic },
        _ => {
            return Err((
                Code::InvalidSchemaValue,
                "only violation requires an error_code; other policies forbid it".into(),
            ));
        }
    })
}

fn diagnostic(raw: &RawDiagnostic) -> Result<RelatedDiagnostic, (Code, String)> {
    if raw.code.is_empty() || raw.code.trim() != raw.code || raw.message_template.is_empty() {
        return Err((
            Code::InvalidSchemaValue,
            "related diagnostic requires explicit code and message".into(),
        ));
    }
    crate::schema::rule_template::parse(&raw.message_template)
        .map_err(|message| (Code::InvalidSchemaValue, message.into()))?;
    Ok(RelatedDiagnostic {
        code: raw.code.as_str().into(),
        message_template: raw.message_template.as_str().into(),
    })
}

pub(super) fn compile_parameters(
    raw: &[RawParameter],
) -> Result<Vec<ParameterDefinition>, (Code, String)> {
    let mut ids = HashSet::new();
    raw.iter()
        .map(|parameter| {
            let id = match parameter {
                RawParameter::String { id, .. } | RawParameter::Integer { id, .. } => id,
            };
            if id.is_empty() || id.trim() != id || !ids.insert(id) {
                return Err((
                    Code::InvalidSchemaValue,
                    "parameter IDs must be nonempty, unpadded and unique".into(),
                ));
            }
            Ok(match parameter {
                RawParameter::String { id, allowed_values } => {
                    if allowed_values.iter().collect::<HashSet<_>>().len() != allowed_values.len() {
                        return Err((
                            Code::InvalidSchemaValue,
                            "parameter allowed_values must be unique".into(),
                        ));
                    }
                    ParameterDefinition::String {
                        id: id.as_str().into(),
                        allowed_values: allowed_values
                            .iter()
                            .map(|value| value.as_str().into())
                            .collect(),
                    }
                }
                RawParameter::Integer { id, min, max } => {
                    let invalid = || {
                        (
                            Code::InvalidSchemaValue,
                            "integer parameter requires min <= max".into(),
                        )
                    };
                    let min = min
                        .0
                        .to_string()
                        .parse::<IntegerLiteral>()
                        .map_err(|_| invalid())?;
                    let max = max
                        .0
                        .to_string()
                        .parse::<IntegerLiteral>()
                        .map_err(|_| invalid())?;
                    if min > max {
                        return Err(invalid());
                    }
                    ParameterDefinition::Integer {
                        id: id.as_str().into(),
                        min,
                        max,
                    }
                }
            })
        })
        .collect()
}
