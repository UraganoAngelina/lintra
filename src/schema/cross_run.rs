//! Compile generic cross-run bindings before any input or history is read.
//! The capability never enables itself on an old profile or ordinary execution.
use super::{compiled::*, raw::*};
use crate::{SchemaIssueCode as Code, ValidationCode};
use std::collections::HashSet;

pub(super) fn compile(
    raw: &RawFlowSchema,
    parameters: &[ParameterDefinition],
    keys: &[PolicyKey],
) -> Result<Option<CrossRunValidationPlan>, (Code, String)> {
    let fail = || {
        (
            Code::InvalidSchemaValue,
            "invalid cross-run policy bindings".into(),
        )
    };
    let declared = raw
        .required_capabilities
        .contains(&RawFlowSchemaRequiredCapabilitiesItem::PersistenceCrossRunV1);
    let Some(policy) = raw
        .persistence_policy
        .as_ref()
        .and_then(|p| p.cross_run_validation.as_ref())
    else {
        return if declared { Err(fail()) } else { Ok(None) };
    };
    let persistence = raw.persistence_policy.as_ref().ok_or_else(fail)?;
    if !declared
        || raw.schema_format_version != 3
        || policy.traces.is_empty()
        || persistence.snapshot.is_some()
        || !persistence.history_identity_invariants.is_empty()
        || !persistence.temporal_rules.is_empty()
        || !matches!(
            persistence.version_compatibility,
            RawPersistenceVersionCompatibility::SameSchemaVersion {}
        )
    {
        return Err(fail());
    }
    let integer = |id: &str, low: i64, high: i64| {
        parameters
            .iter()
            .position(|p| {
                matches!(p,
            ParameterDefinition::Integer {id: name, min, max}
            if name.as_ref() == id && min >= &low.to_string().parse().unwrap()
                && max <= &high.to_string().parse().unwrap())
            })
            .ok_or_else(fail)
    };
    let year_parameter = integer(&policy.year_parameter, 1, 9999)?;
    let month_parameter = integer(&policy.month_parameter, 1, 12)?;
    if year_parameter == month_parameter
        || persistence
            .namespace_parameters
            .contains(&policy.year_parameter)
        || persistence
            .namespace_parameters
            .contains(&policy.month_parameter)
    {
        return Err(fail());
    }
    let mut seen = HashSet::new();
    let traces = policy
        .traces
        .iter()
        .map(|trace| {
            let trace_index = raw
                .traces
                .iter()
                .position(|t| t.id == trace.trace)
                .ok_or_else(fail)?;
            if !seen.insert(trace_index)
                || !keys.iter().any(|k| k.trace_index == trace_index)
                || !persistence
                    .run_uniqueness
                    .iter()
                    .any(|k| k.trace == trace.trace && matches!(k.key, RawPolicyKeyKey::RecordKey))
            {
                return Err(fail());
            }
            for symbol in [&trace.reuse_error, &trace.missing_error] {
                if symbol.is_empty() {
                    return Err(fail());
                }
                symbol.parse::<ValidationCode>().map_err(|_| fail())?;
            }
            Ok(CrossRunTracePlan {
                trace_index,
                mode: trace.mode,
                duplicate_in_second_pass: trace.duplicate_in_second_pass,
                reuse_error: trace.reuse_error.parse().map_err(|_| fail())?,
                missing_error: trace.missing_error.parse().map_err(|_| fail())?,
            })
        })
        .collect::<Result<Vec<_>, (Code, String)>>()?;
    Ok(Some(CrossRunValidationPlan {
        year_parameter,
        month_parameter,
        traces,
    }))
}
