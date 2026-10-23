//! Local rule lowering inside compiler: refs/literals/templates are frozen first.
use super::*;
use crate::schema::{
    expression::{compile_local_expr, operator_plan, resolve_local_ref},
    rule_template::{self, Part, Token},
};

type Issue = (Code, String);
impl Checks<'_> {
    pub(super) fn rules(&mut self, record: &RawRecordSchema) {
        let mut ids = HashSet::new();
        for rule in &record.rules {
            if rule.id.is_empty() || rule.id.trim() != rule.id || !ids.insert(&rule.id) {
                self.error(
                    Code::InvalidSchemaValue,
                    Some(&record.id),
                    None,
                    "rules require unique nonempty unpadded IDs",
                );
            }
            // Reuse exactly the field constraint checks and regex cache. A rule
            // target does not get an independent, weaker validation dialect.
            for assertion in &rule.assertions {
                if let RawAssertion::Constraint { field, constraint } = assertion
                    && let Ok(ResolvedFieldRef::Local { field_index, .. }) =
                        resolve_local_ref(field, record)
                {
                    let mut spec = record.fields[field_index].clone();
                    spec.constraints = vec![constraint.clone()];
                    let width = match spec.physical {
                        RawPhysical::Numeric { length, .. }
                        | RawPhysical::Alphanumeric { length, .. }
                        | RawPhysical::Date { length, .. }
                        | RawPhysical::Decimal { length, .. } => length,
                    };
                    self.constraints(&spec, width, Some(&record.id));
                }
            }
            if let Err((code, message)) = compile_rule(rule, record, &self.regexes) {
                self.error(code, Some(&record.id), None, message);
            }
        }
    }
}
pub(super) fn compile_rules(
    record: &RawRecordSchema,
    regexes: &RegexPlans,
) -> Result<Vec<CompiledRule>, Issue> {
    record
        .rules
        .iter()
        .map(|rule| compile_rule(rule, record, regexes))
        .collect()
}
fn compile_rule(
    rule: &RawRule,
    record: &RawRecordSchema,
    regexes: &RegexPlans,
) -> Result<CompiledRule, Issue> {
    if rule.assertions.is_empty() {
        return Err((
            Code::InvalidSchemaValue,
            "rule assertions must be nonempty".into(),
        ));
    }
    let when = compile_local_expr(&rule.when, record)?;
    let assertions = rule
        .assertions
        .iter()
        .map(|assertion| compile_assertion(assertion, record, regexes))
        .collect::<Result<Vec<_>, _>>()?;
    let parts = rule_template::parse(&rule.message_template)
        .map_err(|message| (Code::InvalidSchemaValue, message.into()))?;
    if parts
        .iter()
        .any(|part| matches!(part, Part::Token(Token::FieldId | Token::FieldName)))
        && assertions
            .iter()
            .any(|assertion| assertion.primary_ref().is_none())
    {
        return Err((
            Code::InvalidSchemaValue,
            "field placeholders require a primary field in every assertion".into(),
        ));
    }
    Ok(CompiledRule {
        id: rule.id.clone().into(),
        when,
        assertions,
        error_code: rule.error_code.parse().map_err(|_| {
            (
                Code::InvalidDiagnosticCode,
                "invalid rule diagnostic symbol".into(),
            )
        })?,
        message_template: rule.message_template.clone().into(),
    })
}
fn compile_assertion(
    raw: &RawAssertion,
    record: &RawRecordSchema,
    regexes: &RegexPlans,
) -> Result<CompiledAssertion, Issue> {
    Ok(match raw {
        RawAssertion::FieldPresent { field } => {
            CompiledAssertion::FieldPresent(resolve_local_ref(field, record)?)
        }
        RawAssertion::ConditionTrue { condition } => {
            CompiledAssertion::ConditionTrue(compile_local_expr(condition, record)?)
        }
        RawAssertion::Constraint { field, constraint } => CompiledAssertion::Constraint {
            field: resolve_local_ref(field, record)?,
            constraint: compile_constraint(constraint, regexes)
                .map_err(|code| (code, "rule constraint is not supported or valid".into()))?,
        },
        RawAssertion::EqualsConcat { field, parts } => {
            if parts.is_empty() {
                return Err((
                    Code::InvalidSchemaValue,
                    "concat requires nonempty parts".into(),
                ));
            }
            CompiledAssertion::EqualsConcat {
                field: resolve_local_ref(field, record)?,
                parts: parts
                    .iter()
                    .map(|part| resolve_local_ref(part, record))
                    .collect::<Result<_, _>>()?,
            }
        }
        RawAssertion::FieldCompare {
            left,
            operator,
            right,
            value_type,
        } => {
            let value_type = match value_type {
                RawMetadataBindingValueType::Bytes => ValueType::Bytes,
                RawMetadataBindingValueType::Integer => ValueType::Integer,
                RawMetadataBindingValueType::Date => ValueType::Date,
            };
            CompiledAssertion::FieldCompare {
                left: operand(left, value_type, record)?,
                operator: operator_plan(operator),
                right: operand(right, value_type, record)?,
                value_type,
            }
        }
        RawAssertion::DerivedCompare {
            derived_value,
            operator,
            value,
        } => {
            let (derived_index, kind) = crate::schema::derived::binding(record, derived_value)?;
            CompiledAssertion::DerivedCompare {
                derived_index,
                operator: operator_plan(operator),
                value: crate::schema::derived::literal(value, kind)?,
            }
        }
        RawAssertion::SubstringEqualsDerivedYear {
            field,
            start,
            length,
            source_ref,
            source_format,
            year_digits,
        } => {
            let field = resolve_local_ref(field, record)?;
            let range = source_range(*start, *length)
                .map_err(|code| (code, "invalid year assertion substring range".into()))?;
            let year_digits = match year_digits {
                2 => YearDigits::Two,
                4 => YearDigits::Four,
                _ => {
                    return Err((
                        Code::InvalidTypeParameters,
                        "year_digits must be 2 or 4".into(),
                    ));
                }
            };
            let ResolvedFieldRef::Local { field_index, .. } = &field else {
                return Err((
                    Code::UnsupportedConstruct,
                    "year assertion requires a local target".into(),
                ));
            };
            let width = match record.fields[*field_index].physical {
                RawPhysical::Numeric { length, .. }
                | RawPhysical::Alphanumeric { length, .. }
                | RawPhysical::Date { length, .. }
                | RawPhysical::Decimal { length, .. } => length,
            };
            let digits = match year_digits {
                YearDigits::Two => 2,
                YearDigits::Four => 4,
            };
            if range.len() != digits
                || range.end_exclusive()
                    > usize::try_from(width).map_err(|_| {
                        (Code::RangeOverflow, "year assertion width overflow".into())
                    })?
            {
                return Err((
                    Code::InvalidTypeParameters,
                    "year assertion range must fit target and year_digits".into(),
                ));
            }
            CompiledAssertion::SubstringEqualsDerivedYear {
                field,
                range,
                source: crate::schema::derived::date_ref(source_ref, record)?,
                format: crate::schema::derived::date_format_plan(source_format),
                year_digits,
            }
        }
        _ => {
            return Err((
                Code::UnsupportedConstruct,
                "rule assertion requires an executor not delivered in the local subset".into(),
            ));
        }
    })
}
fn operand(
    raw: &RawOperand,
    kind: ValueType,
    record: &RawRecordSchema,
) -> Result<CompiledOperand, Issue> {
    let mismatch = || {
        (
            Code::ConstraintTypeMismatch,
            "rule operand is incompatible with its declared value_type".into(),
        )
    };
    Ok(match raw {
        RawOperand::Ref(reference) => {
            let field = resolve_local_ref(reference, record)?;
            let ResolvedFieldRef::Local { field_index, .. } = &field else {
                return Err(mismatch());
            };
            let compatible = match kind {
                ValueType::Bytes => true,
                ValueType::Integer => matches!(
                    record.fields[*field_index].physical,
                    RawPhysical::Numeric { .. }
                ),
                ValueType::Date => matches!(
                    record.fields[*field_index].physical,
                    RawPhysical::Date { .. }
                ),
            };
            if !compatible {
                return Err(mismatch());
            }
            CompiledOperand::Field(field)
        }
        RawOperand::Literal(value) => CompiledOperand::Literal(match (kind, value) {
            (ValueType::Bytes, RawValue::Text(value)) => {
                CompiledValue::Bytes(value.as_bytes().to_vec())
            }
            (ValueType::Integer, RawValue::Integer(value)) => {
                CompiledValue::Integer(value.0.to_string().parse().map_err(|_| mismatch())?)
            }
            (ValueType::Date, RawValue::Text(value)) => CompiledValue::Date(
                DateLiteral::from_ddmmyyyy(value.as_bytes()).ok_or_else(|| {
                    (
                        Code::InvalidSchemaValue,
                        "invalid DDMMYYYY rule literal".into(),
                    )
                })?,
            ),
            _ => return Err(mismatch()),
        }),
    })
}
