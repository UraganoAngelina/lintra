//! Pure local assertion execution; technical orchestration/reporting stays outside.
use super::expression_evaluator::{
    EvaluationContext, ExpressionEvaluator, ResolvedFieldValue, TruthValue, UnknownReason,
};
use crate::{
    FieldSlice, FieldViolation,
    schema::compiled::*,
    validators::{
        date::validate_date, domain::validate_allowed_values, range::validate_numeric_range,
        regex::validate_regex,
    },
};
use std::{borrow::Cow, cmp::Ordering};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleValidationResult {
    Satisfied,
    Violated(FieldViolation),
    Unknown(UnknownReason),
}
/// Private execution metadata enriches reports without changing DET §47's result.
pub(crate) struct LocalRuleArtifacts {
    pub result: RuleValidationResult,
    pub failed_assertion: Option<usize>,
}
pub fn validate_local_rule(
    rule: &CompiledRule,
    ctx: &EvaluationContext<'_>,
) -> RuleValidationResult {
    evaluate_local_rule(rule, ctx).result
}
pub(crate) fn evaluate_local_rule(
    rule: &CompiledRule,
    ctx: &EvaluationContext<'_>,
) -> LocalRuleArtifacts {
    if !local_rule_supported(rule, None) {
        return artifacts(RuleValidationResult::Unknown(
            UnknownReason::UnsupportedPlan,
        ));
    }
    let result = match ExpressionEvaluator.evaluate(&rule.when, ctx) {
        TruthValue::False => RuleValidationResult::Satisfied,
        TruthValue::Unknown(reason) => RuleValidationResult::Unknown(reason),
        TruthValue::True => {
            if rule.assertions.is_empty() {
                return artifacts(RuleValidationResult::Unknown(
                    UnknownReason::UnsupportedPlan,
                ));
            }
            let mut unknown = None;
            for (index, assertion) in rule.assertions.iter().enumerate() {
                match validate_assertion(assertion, ctx) {
                    Ok(()) => {}
                    Err(AssertionFailure::Unknown(reason)) => {
                        unknown.get_or_insert(reason);
                    }
                    Err(AssertionFailure::Violated(mut violation)) => {
                        violation.code = rule.error_code.clone();
                        return LocalRuleArtifacts {
                            result: RuleValidationResult::Violated(violation),
                            failed_assertion: Some(index),
                        };
                    }
                }
            }
            unknown.map_or(
                RuleValidationResult::Satisfied,
                RuleValidationResult::Unknown,
            )
        }
    };
    artifacts(result)
}
fn artifacts(result: RuleValidationResult) -> LocalRuleArtifacts {
    LocalRuleArtifacts {
        result,
        failed_assertion: None,
    }
}
enum AssertionFailure {
    Violated(FieldViolation),
    Unknown(UnknownReason),
}
impl From<UnknownReason> for AssertionFailure {
    fn from(reason: UnknownReason) -> Self {
        Self::Unknown(reason)
    }
}
fn mismatch(expected: &str) -> AssertionFailure {
    AssertionFailure::Violated(FieldViolation {
        code: crate::ValidationCode::CrossField,
        expected: Some(expected.into()),
        reason: String::new(),
        diagnostic_context: vec![],
    })
}
fn present(value: ResolvedFieldValue<'_>) -> Result<&[u8], UnknownReason> {
    match value {
        ResolvedFieldValue::Present(bytes) => Ok(bytes),
        ResolvedFieldValue::Missing => Err(UnknownReason::FieldUnavailable),
        ResolvedFieldValue::Unavailable(reason) => Err(reason),
    }
}
fn validate_assertion(
    assertion: &CompiledAssertion,
    ctx: &EvaluationContext<'_>,
) -> Result<(), AssertionFailure> {
    match assertion {
        CompiledAssertion::FieldPresent(field) => match ctx.fields.resolve_field(field) {
            ResolvedFieldValue::Present(_) => Ok(()),
            ResolvedFieldValue::Missing => Err(mismatch("valore presente")),
            ResolvedFieldValue::Unavailable(reason) => Err(reason.into()),
        },
        CompiledAssertion::ConditionTrue(condition) => {
            match ExpressionEvaluator.evaluate(condition, ctx) {
                TruthValue::True => Ok(()),
                TruthValue::False => Err(mismatch("condizione della regola soddisfatta")),
                TruthValue::Unknown(reason) => Err(reason.into()),
            }
        }
        CompiledAssertion::EqualsConcat { field, parts } => {
            if parts.is_empty() {
                return Err(UnknownReason::UnsupportedPlan.into());
            }
            let expected = present(ctx.fields.resolve_field(field))?;
            let mut offset = 0usize;
            let mut matched = true;
            for part in parts {
                let bytes = present(ctx.fields.resolve_field(part))?;
                let Some(end) = offset.checked_add(bytes.len()) else {
                    return Err(UnknownReason::UnsupportedPlan.into());
                };
                matched &= expected.get(offset..end) == Some(bytes);
                offset = end;
            }
            if matched && offset == expected.len() {
                Ok(())
            } else {
                Err(mismatch("concatenazione ordinata dei valori dichiarati"))
            }
        }
        CompiledAssertion::FieldCompare {
            left,
            operator,
            right,
            value_type,
        } => {
            let left = typed_value(left, *value_type, ctx)?;
            let right = typed_value(right, *value_type, ctx)?;
            let order = match (left, right) {
                (TypedValue::Bytes(left), TypedValue::Bytes(right)) => left.cmp(right),
                (TypedValue::Integer(left), TypedValue::Integer(right)) => left.cmp(&right),
                (TypedValue::Date(left), TypedValue::Date(right)) => left.cmp(&right),
                _ => return Err(UnknownReason::UnsupportedPlan.into()),
            };
            let valid = match operator {
                ComparisonOperator::Eq => order == Ordering::Equal,
                ComparisonOperator::Ne => order != Ordering::Equal,
                ComparisonOperator::Lt => order == Ordering::Less,
                ComparisonOperator::Le => order != Ordering::Greater,
                ComparisonOperator::Gt => order == Ordering::Greater,
                ComparisonOperator::Ge => order != Ordering::Less,
            };
            if valid {
                Ok(())
            } else {
                Err(mismatch("confronto tipizzato tra gli operandi dichiarati"))
            }
        }
        CompiledAssertion::Constraint { field, constraint } => {
            let target = match constraint {
                CompiledConstraint::AllowedValues { target, .. }
                | CompiledConstraint::Regex { target, .. } => *target,
                _ => ValueTarget::Logical,
            };
            let bytes = present(match target {
                ValueTarget::Logical => ctx.fields.resolve_field(field),
                ValueTarget::Raw => ctx.fields.resolve_raw_field(field),
            })?;
            let value = FieldSlice { raw: bytes };
            let result = match constraint {
                CompiledConstraint::AllowedValues { values, .. } => {
                    validate_allowed_values(values, value)
                }
                CompiledConstraint::Regex { regex, mode, .. } => {
                    validate_regex(regex, *mode, value)
                }
                CompiledConstraint::Range { min, max } => {
                    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
                        return Err(UnknownReason::InvalidFieldValue.into());
                    }
                    validate_numeric_range(min, max, value)
                }
                CompiledConstraint::Date { format } => validate_date(*format, value),
                _ => return Err(UnknownReason::UnsupportedPlan.into()),
            };
            result.map_err(AssertionFailure::Violated)
        }
        CompiledAssertion::DerivedCompare {
            derived_index,
            operator,
            value,
        } => {
            match super::derived_evaluator::compare_derived(
                ctx.fields.resolve_derived(*derived_index),
                *operator,
                value,
            ) {
                TruthValue::True => Ok(()),
                TruthValue::False => Err(mismatch(
                    "confronto del valore derivato con il limite dichiarato",
                )),
                TruthValue::Unknown(reason) => Err(reason.into()),
            }
        }
        CompiledAssertion::SubstringEqualsDerivedYear {
            field,
            range,
            source,
            format,
            year_digits,
        } => {
            let bytes = present(ctx.fields.resolve_field(field))?;
            let part = range.slice(bytes).ok_or(UnknownReason::FieldUnavailable)?;
            let date = present(ctx.fields.resolve_field(source))?;
            let date = match format {
                DateFormat::DdMmYyyy => DateLiteral::from_ddmmyyyy(date),
            }
            .ok_or(UnknownReason::InvalidFieldValue)?;
            let year = format!("{:04}", date.year);
            let expected = match year_digits {
                YearDigits::Two => &year.as_bytes()[2..],
                YearDigits::Four => year.as_bytes(),
            };
            if part == expected {
                Ok(())
            } else {
                Err(mismatch(
                    "sottostringa uguale alle cifre dichiarate dell'anno sorgente",
                ))
            }
        }
        _ => Err(UnknownReason::UnsupportedPlan.into()),
    }
}
enum TypedValue<'a> {
    Bytes(&'a [u8]),
    Integer(Cow<'a, IntegerLiteral>),
    Date(DateLiteral),
}
fn typed_value<'a>(
    operand: &'a CompiledOperand,
    kind: ValueType,
    ctx: &'a EvaluationContext<'_>,
) -> Result<TypedValue<'a>, UnknownReason> {
    match operand {
        CompiledOperand::Literal(value) => match (kind, value) {
            (ValueType::Bytes, CompiledValue::Bytes(bytes)) => Ok(TypedValue::Bytes(bytes)),
            (ValueType::Integer, CompiledValue::Integer(value)) => {
                Ok(TypedValue::Integer(Cow::Borrowed(value)))
            }
            (ValueType::Date, CompiledValue::Date(value)) => Ok(TypedValue::Date(*value)),
            _ => Err(UnknownReason::UnsupportedPlan),
        },
        CompiledOperand::Field(field) => {
            let bytes = present(ctx.fields.resolve_field(field))?;
            match kind {
                ValueType::Bytes => Ok(TypedValue::Bytes(bytes)),
                ValueType::Integer => {
                    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
                        return Err(UnknownReason::InvalidFieldValue);
                    }
                    let text =
                        std::str::from_utf8(bytes).map_err(|_| UnknownReason::InvalidFieldValue)?;
                    Ok(TypedValue::Integer(Cow::Owned(
                        text.parse().map_err(|_| UnknownReason::InvalidFieldValue)?,
                    )))
                }
                ValueType::Date => DateLiteral::from_ddmmyyyy(bytes)
                    .map(TypedValue::Date)
                    .ok_or(UnknownReason::InvalidFieldValue),
            }
        }
    }
}

/// Verify all nodes before execution, including branches whose predicate would
/// otherwise be false. A future executor cannot be silently hidden by a guard.
pub(crate) fn local_rule_supported(
    rule: &CompiledRule,
    schema: Option<&CompiledRecordSchema>,
) -> bool {
    use super::expression_evaluator::{
        local_expression_supported, local_expression_supported_in_schema,
    };
    use crate::schema::rule_template::{self, Part, Token};
    let expression = |expr: &CompiledExpr| {
        schema.map_or_else(
            || local_expression_supported(expr),
            |schema| local_expression_supported_in_schema(expr, schema),
        )
    };
    if rule.id.is_empty()
        || rule.id.trim() != rule.id.as_ref()
        || rule.assertions.is_empty()
        || !expression(&rule.when)
        || rule.error_code.as_symbol().is_err()
    {
        return false;
    }
    let Ok(parts) = rule_template::parse(&rule.message_template) else {
        return false;
    };
    if parts
        .iter()
        .any(|part| matches!(part, Part::Token(Token::FieldId | Token::FieldName)))
        && rule
            .assertions
            .iter()
            .any(|assertion| assertion.primary_ref().is_none())
    {
        return false;
    }
    let local = |field: &ResolvedFieldRef| match field {
        ResolvedFieldRef::Local {
            field_index,
            field_id,
        } => schema.is_none_or(|schema| {
            schema
                .fields()
                .get(*field_index)
                .is_some_and(|spec| spec.id() == field_id)
        }),
        _ => false,
    };
    rule.assertions.iter().all(|assertion| match assertion {
        CompiledAssertion::FieldPresent(field) => local(field),
        CompiledAssertion::ConditionTrue(expr) => expression(expr),
        CompiledAssertion::DerivedCompare {
            derived_index,
            value,
            ..
        } => {
            super::expression_evaluator::derived_comparison_supported(*derived_index, value, schema)
        }
        CompiledAssertion::SubstringEqualsDerivedYear {
            field,
            range,
            source,
            format,
            year_digits,
        } => {
            let digits = match year_digits {
                YearDigits::Two => 2,
                YearDigits::Four => 4,
            };
            local(field)
                && local(source)
                && range.len() == digits
                && schema.is_none_or(|schema| {
                    let (
                        ResolvedFieldRef::Local { field_index, .. },
                        ResolvedFieldRef::Local {
                            field_index: source_index,
                            ..
                        },
                    ) = (field, source)
                    else {
                        return false;
                    };
                    range.end_exclusive() <= schema.fields()[*field_index].range().len()
                        && schema.fields()[*source_index].field_type() == FieldType::Date(*format)
                })
        }
        CompiledAssertion::EqualsConcat { field, parts } => {
            local(field) && !parts.is_empty() && parts.iter().all(local)
        }
        CompiledAssertion::Constraint { field, constraint } => {
            local(field)
                && match constraint {
                    CompiledConstraint::AllowedValues { values, .. } => !values.is_empty(),
                    CompiledConstraint::Range { min, max } => min <= max,
                    CompiledConstraint::Regex { .. } | CompiledConstraint::Date { .. } => true,
                    _ => false,
                }
        }
        CompiledAssertion::FieldCompare {
            left,
            right,
            value_type,
            ..
        } => [left, right].iter().all(|operand| match operand {
            CompiledOperand::Field(field) => local(field),
            CompiledOperand::Literal(value) => match (value_type, value) {
                (ValueType::Bytes, CompiledValue::Bytes(_))
                | (ValueType::Integer, CompiledValue::Integer(_)) => true,
                (ValueType::Date, CompiledValue::Date(value)) => {
                    (1..=9999).contains(&value.year)
                        && chrono::NaiveDate::from_ymd_opt(
                            i32::from(value.year),
                            u32::from(value.month),
                            u32::from(value.day),
                        )
                        .is_some()
                }
                _ => false,
            },
        }),
        _ => false,
    })
}
