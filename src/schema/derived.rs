//! Ratified closed derived plans: declarations read fields, never execute code.
use super::{
    compiled::*, coordinates::source_range, expression::resolve_local_ref,
    linter::SchemaIssueCode as Code, raw::*,
};
type Issue = (Code, String);
#[cfg(test)]
pub(crate) fn compile_derived_values(
    record: &RawRecordSchema,
) -> Result<Vec<DerivedValuePlan>, Issue> {
    compile_scoped(&super::reference_binding::ReferenceBindings::local(record))
}

/// Calendar-year differences may use one checked scalar/consensus join. Other
/// derived operations retain their existing local-only boundary. The binder
/// checks trace/field/join ownership before a plan can reach either pass.
pub(crate) fn compile_scoped(
    bindings: &super::reference_binding::ReferenceBindings<'_>,
) -> Result<Vec<DerivedValuePlan>, Issue> {
    bindings
        .record
        .derived_values
        .iter()
        .map(|(id, raw)| {
            if id.is_empty() || id.trim() != id {
                return Err((
                    Code::InvalidSchemaValue,
                    "derived IDs must be nonempty and unpadded".into(),
                ));
            }
            compile_plan(raw, bindings)
        })
        .collect()
}

/// Local presence and set predicates cannot read a cross-pass derived value.
/// Walk every node, even behind a constant guard, so metadata cannot silently
/// turn an unsupported obligation into a local success.
pub(crate) fn expression_is_local(
    expression: &CompiledExpr,
    plans: &[DerivedValuePlan],
) -> Result<bool, Issue> {
    let local = |reference: &ResolvedFieldRef| matches!(reference, ResolvedFieldRef::Local { .. });
    let mut pending = vec![expression];
    while let Some(node) = pending.pop() {
        let index = match node {
            CompiledExpr::Const(_) => continue,
            CompiledExpr::And(nodes) | CompiledExpr::Or(nodes) => {
                pending.extend(nodes);
                continue;
            }
            CompiledExpr::Not(node) => {
                pending.push(node);
                continue;
            }
            CompiledExpr::Eq(reference, _)
            | CompiledExpr::Ne(reference, _)
            | CompiledExpr::In(reference, _)
            | CompiledExpr::NotIn(reference, _)
            | CompiledExpr::IsPresent(reference)
            | CompiledExpr::IsMissing(reference) => {
                if !local(reference) {
                    return Ok(false);
                }
                continue;
            }
            CompiledExpr::DerivedEq {
                field,
                derived_index,
            } => {
                if !local(field) {
                    return Ok(false);
                }
                *derived_index
            }
            CompiledExpr::DerivedCompare { derived_index, .. } => *derived_index,
        };
        let plan = plans.get(index).ok_or((
            Code::InternalCompilerInvariant,
            "derived expression index is inconsistent".into(),
        ))?;
        let valid = match plan {
            DerivedValuePlan::Substring { source, .. } => local(source),
            DerivedValuePlan::AgeInCompletedYears {
                birth, reference, ..
            }
            | DerivedValuePlan::CalendarYearDelta {
                birth, reference, ..
            } => local(birth) && local(reference),
        };
        if !valid {
            return Ok(false);
        }
    }
    Ok(true)
}
fn compile_plan(
    raw: &RawDerived,
    bindings: &super::reference_binding::ReferenceBindings<'_>,
) -> Result<DerivedValuePlan, Issue> {
    let record = bindings.record;
    let scoped_date = |reference: &RawRef| {
        let resolved = bindings.resolve(reference)?;
        if !matches!(
            bindings.field(&resolved)?.physical,
            RawPhysical::Date { .. }
        ) {
            return Err((
                Code::ConstraintTypeMismatch,
                "derived calendar operations require date fields".into(),
            ));
        }
        Ok(resolved)
    };
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
            birth: scoped_date(birth_date_ref)?,
            reference: scoped_date(reference_date_ref)?,
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
