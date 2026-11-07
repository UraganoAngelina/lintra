//! Lower only ratified interval and after-date plans. No clinical field IDs, storage
//! lookups, schema edits or fallback executors belong to this compiler module.
use super::*;
use crate::schema::{expression::compile_local_expr, reference_binding::ReferenceBindings};

type Issue = (Code, String);
fn invalid(message: &str) -> Issue {
    (Code::InvalidSchemaValue, message.into())
}

/// Ordinary validation may execute read-only current-input temporal checks.
/// Active audit, snapshot or identity obligations still require their producer;
/// this predicate never removes those obligations to make a profile runnable.
pub(super) fn read_only(raw: &RawFlowSchema) -> bool {
    raw.persistence_policy.as_ref().is_some_and(|p| {
        !p.temporal_rules.is_empty()
            && p.cross_run_audit.is_empty()
            && p.history_identity_invariants.is_empty()
            && p.snapshot.is_none()
    })
}

fn date(trace: &RawRecordSchema, id: &str) -> Result<usize, Issue> {
    let (index, field) = trace
        .fields
        .iter()
        .enumerate()
        .find(|(_, f)| f.id == id)
        .ok_or((
            Code::MissingFieldReference,
            "temporal date field is not declared".into(),
        ))?;
    if !matches!(field.physical, RawPhysical::Date { .. }) {
        return Err((
            Code::ConstraintTypeMismatch,
            "temporal fields require physical dates".into(),
        ));
    }
    Ok(index)
}

pub(super) fn compile(raw: &RawFlowSchema) -> Result<Vec<TemporalPlan>, Issue> {
    let Some(policy) = &raw.persistence_policy else {
        return Ok(vec![]);
    };
    for (capability, declared) in [
        (
            RawFlowSchemaRequiredCapabilitiesItem::TemporalIntervalsV1,
            policy
                .temporal_rules
                .iter()
                .any(|p| matches!(p, RawTemporal::IntervalExclusion { .. })),
        ),
        (
            RawFlowSchemaRequiredCapabilitiesItem::TemporalAfterDateV1,
            policy
                .temporal_rules
                .iter()
                .any(|p| matches!(p, RawTemporal::AfterDateRestriction { .. })),
        ),
    ] {
        if raw.required_capabilities.contains(&capability) != declared {
            return Err((
                Code::MissingCapability,
                "temporal declarations and capabilities must agree".into(),
            ));
        }
    }
    if policy.temporal_rules.is_empty() {
        return Ok(vec![]);
    }
    let joins = joins::compile_joins(raw)?;
    policy
        .temporal_rules
        .iter()
        .map(|plan| {
            if matches!(plan, RawTemporal::AfterDateRestriction { .. }) {
                return after_date(raw, &joins, plan);
            }
            let RawTemporal::IntervalExclusion {
                owner_trace,
                when,
                point_field,
                interval_join,
                start_field,
                end_field,
                on_missing_trace,
                error_code,
                ..
            } = plan
            else {
                return Err((
                    Code::UnsupportedConstruct,
                    "temporal executor is not implemented".into(),
                ));
            };
            let owner = raw
                .traces
                .iter()
                .position(|t| t.id == *owner_trace)
                .ok_or_else(|| invalid("temporal owner trace is not declared"))?;
            let bindings = ReferenceBindings::cross(raw, owner, &joins)?;
            let (join_index, join) = bindings.join(interval_join)?;
            // A set-valued interval search must inspect every target. Consensus
            // cannot replace intervals with a single representative record.
            if join.cardinality != JoinCardinality::ZeroOrMany || join.consensus.is_some() {
                return Err(invalid(
                    "interval exclusion requires a zero_or_many join without consensus",
                ));
            }
            let source = &raw.traces[owner];
            let target = &raw.traces[join.target_trace];
            let raw_join = source
                .joins
                .iter()
                .find(|j| j.id == *interval_join)
                .ok_or_else(|| invalid("temporal join is not declared"))?;
            if raw_join.on_missing_trace != *on_missing_trace {
                return Err(invalid(
                    "temporal and join missing-trace policies must agree",
                ));
            }
            // No related record is a proven empty set, unlike a missing target file.
            // Its join-level missing-record policy must not imply an obligation to
            // suspend every event. The temporal executor does not use that policy.
            let on_missing_trace = joins::policy(on_missing_trace)?;
            if !matches!(on_missing_trace, RelatedPolicy::Unknown { .. }) {
                return Err((
                    Code::UnsupportedConstruct,
                    "interval missing input requires explicit Unknown".into(),
                ));
            }
            let when = compile_local_expr(when, source)?;
            // A local derived name can hide an external calendar operand. Reuse
            // the exhaustive locality guard, including indirectly read fields.
            let derived = crate::schema::derived::compile_scoped(&bindings)?;
            if !crate::schema::derived::expression_is_local(&when, &derived)? {
                return Err((
                    Code::UnsupportedConstruct,
                    "temporal when must remain local through derived dependencies".into(),
                ));
            }
            Ok(TemporalPlan::IntervalExclusion(IntervalExclusionPlan {
                owner_trace: owner,
                when,
                point_field: date(source, point_field)?,
                interval_join: join_index,
                start_field: date(target, start_field)?,
                end_field: date(target, end_field)?,
                on_missing_trace,
                error_code: error_code.parse().map_err(|_| {
                    (
                        Code::InvalidDiagnosticCode,
                        "invalid interval diagnostic code".into(),
                    )
                })?,
            }))
        })
        .collect()
}

/// Both predicates are local to their respective rows/qualified tuple. The
/// consensus must cover every target read, including indirect derived inputs.
fn after_date(
    raw: &RawFlowSchema,
    joins: &[CompiledJoinSpec],
    plan: &RawTemporal,
) -> Result<TemporalPlan, Issue> {
    let RawTemporal::AfterDateRestriction {
        owner_trace,
        when,
        date_field,
        conclusion_join,
        conclusion_date_field,
        target_when,
        restricted_field,
        forbidden_values,
        on_missing_trace,
        error_code,
        ..
    } = plan
    else {
        return Err(invalid("after-date plan expected"));
    };
    let owner = raw
        .traces
        .iter()
        .position(|t| t.id == *owner_trace)
        .ok_or_else(|| invalid("temporal owner trace is not declared"))?;
    let source = &raw.traces[owner];
    let bindings = ReferenceBindings::cross(raw, owner, joins)?;
    let (join_index, join) = bindings.join(conclusion_join)?;
    if !matches!(
        join.cardinality,
        JoinCardinality::ZeroOrOne | JoinCardinality::ExactlyOne
    ) && join.consensus.is_none()
    {
        return Err(invalid("after-date set join requires explicit consensus"));
    }
    let target = &raw.traces[join.target_trace];
    let raw_join = source
        .joins
        .iter()
        .find(|j| j.id == *conclusion_join)
        .ok_or_else(|| invalid("temporal join is not declared"))?;
    if raw_join.on_missing_trace != *on_missing_trace {
        return Err(invalid(
            "temporal and join missing-trace policies must agree",
        ));
    }
    let policy = joins::policy(on_missing_trace)?;
    if !matches!(policy, RelatedPolicy::Unknown { .. }) {
        return Err((
            Code::UnsupportedConstruct,
            "after-date missing input requires explicit Unknown".into(),
        ));
    }
    let when = compile_local_expr(when, source)?;
    let source_derived = crate::schema::derived::compile_scoped(&bindings)?;
    let target_when = compile_local_expr(target_when, target)?;
    let target_derived =
        crate::schema::derived::compile_scoped(&bindings.target(join.target_trace)?)?;
    for (condition, derived) in [(&when, &source_derived), (&target_when, &target_derived)] {
        if !crate::schema::derived::expression_is_local(condition, derived)? {
            return Err((
                Code::UnsupportedConstruct,
                "temporal predicate must remain local through derived dependencies".into(),
            ));
        }
    }
    let conclusion_date_field = date(target, conclusion_date_field)?;
    if let Some(consensus) = &join.consensus {
        if !crate::schema::derived::expression_is_local(&consensus.include_if, &target_derived)? {
            return Err((
                Code::UnsupportedConstruct,
                "consensus predicate must remain local".into(),
            ));
        }
        let mut required = crate::schema::rule_dependencies::RuleDependencies::local_fields(
            &target_when,
            &target_derived,
        )?;
        required.insert(conclusion_date_field);
        if !required
            .iter()
            .all(|field| consensus.fields.contains(field))
        {
            return Err(invalid(
                "conclusion consensus must contain every target predicate/date dependency",
            ));
        }
    }
    if forbidden_values.is_empty()
        || forbidden_values.iter().collect::<HashSet<_>>().len() != forbidden_values.len()
    {
        return Err(invalid("forbidden values must be a nonempty unique list"));
    }
    Ok(TemporalPlan::AfterDateRestriction(
        AfterDateRestrictionPlan {
            owner_trace: owner,
            when,
            date_field: date(source, date_field)?,
            conclusion_join: join_index,
            conclusion_date_field,
            target_when,
            restricted_field: index(source, restricted_field)?,
            forbidden_values: forbidden_values
                .iter()
                .map(|v| v.as_bytes().to_vec())
                .collect(),
            on_missing_trace: policy,
            error_code: error_code.parse().map_err(|_| {
                (
                    Code::InvalidDiagnosticCode,
                    "invalid after-date diagnostic code".into(),
                )
            })?,
        },
    ))
}
