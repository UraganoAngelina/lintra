//! Ratified closed derived plans: declarations read fields, never execute code.
use super::{
    compiled::*, coordinates::source_range, expression::resolve_local_ref,
    linter::SchemaIssueCode as Code, raw::*,
};
type Issue = (Code, String);
pub(crate) fn compile_derived_values(
    record: &RawRecordSchema,
) -> Result<Vec<DerivedValuePlan>, Issue> {
    record
        .derived_values
        .iter()
        .map(|(id, raw)| {
            if id.is_empty() || id.trim() != id {
                return Err((
                    Code::InvalidSchemaValue,
                    "derived IDs must be nonempty and unpadded".into(),
                ));
            }
            compile_plan(raw, record)
        })
        .collect()
}
fn compile_plan(raw: &RawDerived, record: &RawRecordSchema) -> Result<DerivedValuePlan, Issue> {
    Ok(match raw {
        RawDerived::Substring {
            source_ref,
            target,
            start,
            length,
        } => {
            let source = resolve_local_ref(source_ref, record)?;
            let range = source_range(*start, *length)
                .map_err(|code| (code, "invalid derived substring range".into()))?;
            let ResolvedFieldRef::Local { field_index, .. } = &source else {
                return Err((
                    Code::UnsupportedConstruct,
                    "derived source requires a local field".into(),
                ));
            };
            let width = match record.fields[*field_index].physical {
                RawPhysical::Numeric { length, .. }
                | RawPhysical::Alphanumeric { length, .. }
                | RawPhysical::Date { length, .. }
                | RawPhysical::Decimal { length, .. } => length,
            };
            if range.end_exclusive()
                > usize::try_from(width).map_err(|_| {
                    (
                        Code::RangeOverflow,
                        "derived source width is not representable".into(),
                    )
                })?
            {
                return Err((
                    Code::InvalidTypeParameters,
                    "derived substring exceeds source field width".into(),
                ));
            }
            DerivedValuePlan::Substring {
                source,
                target: match target {
                    RawConstraintAllowedValuesTarget::Raw => ValueTarget::Raw,
                    RawConstraintAllowedValuesTarget::Logical => ValueTarget::Logical,
                },
                range,
            }
        }
        RawDerived::AgeInCompletedYears {
            birth_date_ref,
            reference_date_ref,
            date_format,
            leap_day_anniversary,
        } => DerivedValuePlan::AgeInCompletedYears {
            birth: date_ref(birth_date_ref, record)?,
            reference: date_ref(reference_date_ref, record)?,
            format: date_format_plan(date_format),
            leap_day: match leap_day_anniversary {
                RawDerivedAgeInCompletedYearsLeapDayAnniversary::Feb28 => LeapDayAnniversary::Feb28,
                RawDerivedAgeInCompletedYearsLeapDayAnniversary::Mar01 => LeapDayAnniversary::Mar01,
            },
        },
        RawDerived::CalendarYearDelta {
            birth_date_ref,
            reference_date_ref,
            date_format,
        } => DerivedValuePlan::CalendarYearDelta {
            birth: date_ref(birth_date_ref, record)?,
            reference: date_ref(reference_date_ref, record)?,
            format: date_format_plan(date_format),
        },
    })
}
pub(crate) fn date_ref(raw: &RawRef, record: &RawRecordSchema) -> Result<ResolvedFieldRef, Issue> {
    let field = resolve_local_ref(raw, record)?;
    let ResolvedFieldRef::Local { field_index, .. } = &field else {
        return Err((
            Code::UnsupportedConstruct,
            "derived date source requires a local field".into(),
        ));
    };
    if !matches!(
        record.fields[*field_index].physical,
        RawPhysical::Date { .. }
    ) {
        return Err((
            Code::ConstraintTypeMismatch,
            "derived calendar operations require date fields".into(),
        ));
    }
    Ok(field)
}
pub(crate) fn date_format_plan(raw: &RawPhysicalDateFormat) -> DateFormat {
    match raw {
        RawPhysicalDateFormat::Ddmmyyyy => DateFormat::DdMmYyyy,
    }
}
/// The raw map is canonical and ordered: its index is exactly the plan vector's
/// index. References do not depend on declaration order or record validation.
pub(crate) fn binding(record: &RawRecordSchema, id: &str) -> Result<(usize, ValueType), Issue> {
    record
        .derived_values
        .iter()
        .enumerate()
        .find(|(_, (name, _))| name.as_str() == id)
        .map(|(index, (_, raw))| {
            (
                index,
                match raw {
                    RawDerived::Substring { .. } => ValueType::Bytes,
                    _ => ValueType::Integer,
                },
            )
        })
        .ok_or((
            Code::MissingDerivedValueReference,
            "derived value does not exist".into(),
        ))
}
pub(crate) fn literal(raw: &RawValue, kind: ValueType) -> Result<CompiledValue, Issue> {
    match (raw, kind) {
        (RawValue::Text(value), ValueType::Bytes) => {
            Ok(CompiledValue::Bytes(value.as_bytes().to_vec()))
        }
        (RawValue::Integer(value), ValueType::Integer) => Ok(CompiledValue::Integer(
            value.0.to_string().parse().map_err(|_| {
                (
                    Code::InvalidSchemaValue,
                    "invalid derived integer literal".into(),
                )
            })?,
        )),
        _ => Err((
            Code::ConstraintTypeMismatch,
            "derived literal is incompatible with its result type".into(),
        )),
    }
}
#[cfg(test)]
#[path = "derived_tests.rs"]
mod tests;
