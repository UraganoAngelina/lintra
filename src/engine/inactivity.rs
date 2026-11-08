//! Current-run inactivity evaluation. Every identifier/field/code is a compiled
//! index/value; this module has no storage or clinical trace-name knowledge.
use super::{
    projected_context::{ProjectedEvaluationContext, ProjectedFieldContext},
    temporal_context::QualifiedTarget,
};
use crate::schema::compiled::InactivityWithPausesPlan;
use crate::*;
use std::collections::HashSet;

type Verdict = (TruthValue, Option<usize>);
fn reference(schema: &CompiledRecordSchema, index: usize) -> ResolvedFieldRef {
    ResolvedFieldRef::Local {
        field_index: index,
        field_id: schema.fields()[index].id().clone(),
    }
}
fn bytes(
    fields: &dyn FieldValueResolver,
    schema: &CompiledRecordSchema,
    index: usize,
) -> Result<Option<Vec<u8>>, UnknownReason> {
    match fields.resolve_field(&reference(schema, index)) {
        ResolvedFieldValue::Present(value) => Ok(Some(value.to_vec())),
        ResolvedFieldValue::Missing => Ok(None),
        ResolvedFieldValue::Unavailable(reason) => Err(reason),
    }
}
fn date(
    fields: &dyn FieldValueResolver,
    schema: &CompiledRecordSchema,
    index: usize,
) -> Result<Option<DateLiteral>, UnknownReason> {
    bytes(fields, schema, index)?
        .map(|value| DateLiteral::from_ddmmyyyy(&value).ok_or(UnknownReason::InvalidFieldValue))
        .transpose()
}
fn tuple(
    fields: &dyn FieldValueResolver,
    schema: &CompiledRecordSchema,
    indexes: &[usize],
) -> Result<Vec<Vec<u8>>, UnknownReason> {
    indexes
        .iter()
        .map(|&index| {
            let value = bytes(fields, schema, index)?.ok_or(UnknownReason::FieldUnavailable)?;
            // A malformed date cannot prove a missing service match merely because
            // its raw bytes differ. Pass-1 diagnostics and usable projections are
            // independent: explicitly qualify dates used as correlation evidence.
            if matches!(schema.fields()[index].field_type(), FieldType::Date(_))
                && DateLiteral::from_ddmmyyyy(&value).is_none()
            {
                return Err(UnknownReason::InvalidFieldValue);
            }
            Ok(value)
        })
        .collect()
}

pub(super) fn evaluate(
    flow: &CompiledFlowSchema,
    context: &ProjectedEvaluationContext<'_>,
    plan: &InactivityWithPausesPlan,
) -> Result<Verdict, crate::CrossRecordValidationError> {
    let unknown = |reason, join| Ok((TruthValue::Unknown(reason), Some(join)));
    // Fail closed on unavailable/partial sets before drawing absence conclusions.
    // A supplied empty set remains complete evidence of no activity/pauses.
    for index in [
        plan.evaluations_join,
        plan.suspensions_join,
        plan.accesses_join,
        plan.provisions_join,
    ] {
        let lookup = context
            .related(index)
            .ok_or(CrossRecordValidationError::InvalidPlan)?;
        if let RelatedResolution::Unavailable(reason) = lookup.resolution {
            return unknown(reason, index);
        }
        if context.lookup_coverage(index) != Some(TraceContextCoverage::Complete) {
            return unknown(UnknownReason::IncompleteRelatedContext, index);
        }
    }
    let evaluation = context
        .related(plan.evaluations_join)
        .ok_or(CrossRecordValidationError::InvalidPlan)?;
    let target = &flow.traces()[evaluation.join.target_trace];
    let qualified = QualifiedTarget {
        context,
        schema: target,
        join: evaluation.join,
        join_index: plan.evaluations_join,
    };
    let start = match date(&qualified, target, plan.start_field) {
        Ok(Some(value)) => value,
        Ok(None) => return unknown(UnknownReason::FieldUnavailable, plan.evaluations_join),
        Err(reason) => return unknown(reason, plan.evaluations_join),
    };
    let end = match date(&qualified, target, plan.conclusion_date_field) {
        Ok(value) => value,
        Err(reason) => return unknown(reason, plan.evaluations_join),
    };
    let code = match bytes(&qualified, target, plan.conclusion_code_field) {
        Ok(value) => value,
        Err(reason) => return unknown(reason, plan.evaluations_join),
    };
    let horizon = context
        .session_horizon()
        .ok_or(CrossRecordValidationError::InvalidPlan)?;
    match (end, code) {
        (Some(end), Some(_)) if start <= end && end <= horizon => {
            // No second invented closure for an already legitimately closed
            // event (administrative closure itself also discharges the duty).
            return Ok((TruthValue::True, None));
        }
        (Some(end), Some(_)) if end < start => {
            return unknown(UnknownReason::InvalidFieldValue, plan.evaluations_join);
        }
        (None, None) | (Some(_), Some(_)) => {}
        _ => return unknown(UnknownReason::FieldUnavailable, plan.evaluations_join),
    }
    let sets = |index| -> Result<_, CrossRecordValidationError> {
        let lookup = context
            .related(index)
            .ok_or(CrossRecordValidationError::InvalidPlan)?;
        let RelatedResolution::Records(records) = &lookup.resolution else {
            return Err(CrossRecordValidationError::InvalidPlan);
        };
        Ok((&flow.traces()[lookup.join.target_trace], records))
    };
    // Only correlation tuples are retained, never source rows or progressives.
    let (schema, records) = sets(plan.provisions_join)?;
    let mut serviced = HashSet::with_capacity(records.len());
    for record in records.iter() {
        match tuple(
            &ProjectedFieldContext { schema, record },
            schema,
            &plan.provision_fields,
        ) {
            Ok(value) => {
                serviced.insert(value);
            }
            Err(reason) => return unknown(reason, plan.provisions_join),
        }
    }
    let (schema, records) = sets(plan.accesses_join)?;
    let mut last = None;
    for record in records.iter() {
        let fields = ProjectedFieldContext { schema, record };
        let key = match tuple(&fields, schema, &plan.access_fields) {
            Ok(key) => key,
            Err(reason) => return unknown(reason, plan.accesses_join),
        };
        let current = match date(&fields, schema, plan.access_date_field) {
            Ok(Some(value)) => value,
            Ok(None) => return unknown(UnknownReason::FieldUnavailable, plan.accesses_join),
            Err(reason) => return unknown(reason, plan.accesses_join),
        };
        if current <= horizon && serviced.contains(&key) {
            last = Some(last.map_or(current, |previous: DateLiteral| previous.max(current)));
        }
    }
    let (schema, records) = sets(plan.suspensions_join)?;
    let mut pauses = Vec::with_capacity(records.len());
    for record in records.iter() {
        let fields = ProjectedFieldContext { schema, record };
        let start = match date(&fields, schema, plan.suspension_start_field) {
            Ok(Some(value)) => value,
            Ok(None) => return unknown(UnknownReason::FieldUnavailable, plan.suspensions_join),
            Err(reason) => return unknown(reason, plan.suspensions_join),
        };
        let end = match date(&fields, schema, plan.suspension_end_field) {
            Ok(value) => value,
            Err(reason) => return unknown(reason, plan.suspensions_join),
        };
        pauses.push((start, end));
    }
    match super::inactivity_calendar::due(
        last.unwrap_or(start),
        plan.calendar_months,
        &pauses,
        horizon,
    ) {
        Ok(true) => Ok((TruthValue::False, None)),
        Ok(false) => Ok((TruthValue::True, None)),
        Err(reason) => unknown(reason, plan.suspensions_join),
    }
}
