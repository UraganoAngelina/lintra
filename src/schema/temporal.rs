//! Lower only the ratified open-interval plan. No clinical field IDs, storage
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
    let capability = raw
        .required_capabilities
        .contains(&RawFlowSchemaRequiredCapabilitiesItem::TemporalIntervalsV1);
    if capability
        != policy
            .temporal_rules
            .iter()
            .any(|p| matches!(p, RawTemporal::IntervalExclusion { .. }))
    {
        return Err((
            Code::MissingCapability,
            "interval declarations and temporal.intervals.v1 must agree".into(),
        ));
    }
    if policy.temporal_rules.is_empty() {
        return Ok(vec![]);
    }
    let joins = joins::compile_joins(raw)?;
    policy
        .temporal_rules
        .iter()
        .map(|plan| {
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
