//! Only the three ratified, read-only operations. No host date or function names.
use super::expression_evaluator::{
    EvaluationContext, FieldValueResolver, ResolvedFieldValue, TruthValue, UnknownReason,
};
use crate::schema::compiled::*;
use std::cmp::Ordering;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DerivedValue<'a> {
    Bytes(&'a [u8]),
    Integer(IntegerLiteral),
    Unknown(UnknownReason),
}
pub struct DerivedValueEvaluator;
impl DerivedValueEvaluator {
    /// Bytes borrow the actual source record; integers own only their bounded
    /// exact magnitude. Missing/invalid inputs never become empty/zero dates.
    pub fn evaluate<'a>(
        &self,
        plan: &DerivedValuePlan,
        ctx: &EvaluationContext<'a>,
    ) -> DerivedValue<'a> {
        let fields: &'a dyn FieldValueResolver = ctx.fields;
        let result = match plan {
            DerivedValuePlan::Substring {
                source,
                target,
                range,
            } => source_bytes(fields, source, *target)
                .and_then(|value| range.slice(value).ok_or(UnknownReason::FieldUnavailable))
                .map(DerivedValue::Bytes),
            DerivedValuePlan::CalendarYearDelta {
                birth,
                reference,
                format,
            } => dates(fields, birth, reference, *format)
                .and_then(|(birth, reference)| {
                    i32::from(reference.year)
                        .checked_sub(i32::from(birth.year))
                        .ok_or(UnknownReason::UnsupportedPlan)
                })
                .and_then(integer),
            DerivedValuePlan::AgeInCompletedYears {
                birth,
                reference,
                format,
                leap_day,
            } => dates(fields, birth, reference, *format)
                .and_then(|(birth, reference)| {
                    // Keep signed results: only schema constraints decide
                    // whether a reference preceding birth is admissible.
                    let anniversary = if birth.month == 2
                        && birth.day == 29
                        && chrono::NaiveDate::from_ymd_opt(i32::from(reference.year), 2, 29)
                            .is_none()
                    {
                        match leap_day {
                            LeapDayAnniversary::Feb28 => (2, 28),
                            LeapDayAnniversary::Mar01 => (3, 1),
                        }
                    } else {
                        (birth.month, birth.day)
                    };
                    let before = (reference.month, reference.day) < anniversary;
                    i32::from(reference.year)
                        .checked_sub(i32::from(birth.year))
                        .and_then(|years| years.checked_sub(if before { 1 } else { 0 }))
                        .ok_or(UnknownReason::UnsupportedPlan)
                })
                .and_then(integer),
        };
        result.unwrap_or_else(DerivedValue::Unknown)
    }
}
fn source_bytes<'a>(
    fields: &'a dyn FieldValueResolver,
    reference: &ResolvedFieldRef,
    target: ValueTarget,
) -> Result<&'a [u8], UnknownReason> {
    match match target {
        ValueTarget::Raw => fields.resolve_raw_field(reference),
        ValueTarget::Logical => fields.resolve_field(reference),
    } {
        ResolvedFieldValue::Present(value) => Ok(value),
        ResolvedFieldValue::Missing => Err(UnknownReason::FieldUnavailable),
        ResolvedFieldValue::Unavailable(reason) => Err(reason),
    }
}
fn dates(
    fields: &dyn FieldValueResolver,
    birth: &ResolvedFieldRef,
    reference: &ResolvedFieldRef,
    format: DateFormat,
) -> Result<(DateLiteral, DateLiteral), UnknownReason> {
    let parse = |value: &[u8]| match format {
        DateFormat::DdMmYyyy => {
            DateLiteral::from_ddmmyyyy(value).ok_or(UnknownReason::InvalidFieldValue)
        }
    };
    Ok((
        parse(source_bytes(fields, birth, ValueTarget::Logical)?)?,
        parse(source_bytes(fields, reference, ValueTarget::Logical)?)?,
    ))
}
fn integer<'a>(value: i32) -> Result<DerivedValue<'a>, UnknownReason> {
    value
        .to_string()
        .parse()
        .map(DerivedValue::Integer)
        .map_err(|_| UnknownReason::UnsupportedPlan)
}
pub(crate) fn compare_derived(
    value: DerivedValue<'_>,
    operator: ComparisonOperator,
    literal: &CompiledValue,
) -> TruthValue {
    let order = match (value, literal) {
        (DerivedValue::Unknown(reason), _) => return TruthValue::Unknown(reason),
        (DerivedValue::Bytes(left), CompiledValue::Bytes(right)) => left.cmp(right),
        (DerivedValue::Integer(left), CompiledValue::Integer(right)) => left.cmp(right),
        _ => return TruthValue::Unknown(UnknownReason::UnsupportedPlan),
    };
    ordered_truth(order, operator)
}
pub(crate) fn ordered_truth(order: Ordering, operator: ComparisonOperator) -> TruthValue {
    let valid = match operator {
        ComparisonOperator::Eq => order == Ordering::Equal,
        ComparisonOperator::Ne => order != Ordering::Equal,
        ComparisonOperator::Lt => order == Ordering::Less,
        ComparisonOperator::Le => order != Ordering::Greater,
        ComparisonOperator::Gt => order == Ordering::Greater,
        ComparisonOperator::Ge => order != Ordering::Less,
    };
    if valid {
        TruthValue::True
    } else {
        TruthValue::False
    }
}
