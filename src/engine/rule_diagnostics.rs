//! Rule report enrichment from real execution metadata and physical record bytes.
use super::error_factory::ErrorFactory;
use crate::{
    DiagnosticContextItem, FieldDiagnostic, FieldViolation, RecordIdentity, RecordView,
    ValidationError, render_diagnostic_value,
    schema::{
        compiled::*,
        rule_template::{self, Part, Token},
    },
};
use std::{collections::BTreeSet, error::Error, fmt};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleDiagnosticError {
    InvalidPlan,
    InvalidTemplate,
}
impl fmt::Display for RuleDiagnosticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidPlan => "local rule diagnostic plan is inconsistent",
            Self::InvalidTemplate => "local rule message template is invalid",
        })
    }
}
impl Error for RuleDiagnosticError {}

pub(crate) struct RuleDiagnosticInput<'a> {
    pub rule: &'a CompiledRule,
    pub assertion_index: usize,
    pub record: &'a RecordView<'a>,
    pub schema: &'a CompiledRecordSchema,
}
impl ErrorFactory<'_> {
    pub(crate) fn local_rule_error(
        &self,
        line: u64,
        identity: &RecordIdentity,
        input: RuleDiagnosticInput<'_>,
        violation: FieldViolation,
    ) -> Result<ValidationError, RuleDiagnosticError> {
        let rule = input.rule;
        let assertion = rule
            .assertions
            .get(input.assertion_index)
            .ok_or(RuleDiagnosticError::InvalidPlan)?;
        let primary = assertion
            .primary_ref()
            .map(|reference| field(reference, input.schema))
            .transpose()?;
        let mut references = Vec::new();
        expression_refs(&rule.when, input.schema, &mut references)?;
        assertion_refs(assertion, input.schema, &mut references)?;
        let mut seen = BTreeSet::new();
        let mut context = Vec::new();
        for reference in references {
            let spec = field(reference, input.schema)?;
            if seen.insert(spec.id().0.as_str()) {
                let raw = input
                    .record
                    .field(spec)
                    .ok_or(RuleDiagnosticError::InvalidPlan)?;
                context.push(DiagnosticContextItem {
                    trace_id: self.trace_id.clone(),
                    field_id: spec.id().clone(),
                    value: render_diagnostic_value(raw.raw, self.encoding),
                });
            }
        }
        let condition = condition_text(&rule.when, input.schema)?;
        let mut message = String::new();
        for part in rule_template::parse(&rule.message_template)
            .map_err(|_| RuleDiagnosticError::InvalidTemplate)?
        {
            match part {
                Part::Text(text) => message.push_str(text),
                Part::Token(token) => message.push_str(match token {
                    Token::RuleId => &rule.id,
                    Token::Condition => &condition,
                    Token::FieldId => &primary.ok_or(RuleDiagnosticError::InvalidTemplate)?.id().0,
                    Token::FieldName => primary.ok_or(RuleDiagnosticError::InvalidTemplate)?.name(),
                }),
            }
        }
        let actual = primary
            .map(|spec| {
                input
                    .record
                    .field(spec)
                    .ok_or(RuleDiagnosticError::InvalidPlan)
                    .map(|value| render_diagnostic_value(value.raw, self.encoding))
            })
            .transpose()?;
        Ok(ValidationError {
            code: violation.code,
            flow_id: self.flow_id.clone(),
            schema_version: self.schema_version.clone(),
            trace_id: self.trace_id.clone(),
            line_number: line,
            record_identity: identity.clone(),
            field: primary.map(|spec| {
                FieldDiagnostic::from_range(spec.id().clone(), spec.name().into(), spec.range())
            }),
            actual,
            expected: violation.expected,
            context,
            message,
        })
    }
}
fn field<'a>(
    reference: &ResolvedFieldRef,
    schema: &'a CompiledRecordSchema,
) -> Result<&'a CompiledFieldSpec, RuleDiagnosticError> {
    let ResolvedFieldRef::Local {
        field_index,
        field_id,
    } = reference
    else {
        return Err(RuleDiagnosticError::InvalidPlan);
    };
    schema
        .fields()
        .get(*field_index)
        .filter(|field| field.id() == field_id)
        .ok_or(RuleDiagnosticError::InvalidPlan)
}
pub(crate) fn expression_refs<'a>(
    expression: &'a CompiledExpr,
    schema: &'a CompiledRecordSchema,
    refs: &mut Vec<&'a ResolvedFieldRef>,
) -> Result<(), RuleDiagnosticError> {
    let mut work = vec![expression];
    while let Some(expr) = work.pop() {
        match expr {
            CompiledExpr::Eq(field, _)
            | CompiledExpr::Ne(field, _)
            | CompiledExpr::In(field, _)
            | CompiledExpr::NotIn(field, _)
            | CompiledExpr::IsPresent(field)
            | CompiledExpr::IsMissing(field) => refs.push(field),
            CompiledExpr::And(children) | CompiledExpr::Or(children) => {
                work.extend(children.iter().rev())
            }
            CompiledExpr::Not(child) => work.push(child),
            CompiledExpr::DerivedEq {
                field,
                derived_index,
            } => {
                refs.push(field);
                derived_refs(*derived_index, schema, refs)?;
            }
            CompiledExpr::DerivedCompare { derived_index, .. } => {
                derived_refs(*derived_index, schema, refs)?
            }
            CompiledExpr::Const(_) => {}
        }
    }
    Ok(())
}
fn derived_refs<'a>(
    index: usize,
    schema: &'a CompiledRecordSchema,
    refs: &mut Vec<&'a ResolvedFieldRef>,
) -> Result<(), RuleDiagnosticError> {
    match schema
        .derived_values()
        .get(index)
        .ok_or(RuleDiagnosticError::InvalidPlan)?
    {
        DerivedValuePlan::Substring { source, .. } => refs.push(source),
        DerivedValuePlan::AgeInCompletedYears {
            birth, reference, ..
        }
        | DerivedValuePlan::CalendarYearDelta {
            birth, reference, ..
        } => {
            refs.push(birth);
            refs.push(reference);
        }
    }
    Ok(())
}
pub(crate) fn assertion_refs<'a>(
    assertion: &'a CompiledAssertion,
    schema: &'a CompiledRecordSchema,
    refs: &mut Vec<&'a ResolvedFieldRef>,
) -> Result<(), RuleDiagnosticError> {
    match assertion {
        CompiledAssertion::FieldPresent(field) | CompiledAssertion::Constraint { field, .. } => {
            refs.push(field)
        }
        CompiledAssertion::EqualsConcat { field, parts } => {
            refs.push(field);
            refs.extend(parts);
        }
        CompiledAssertion::ConditionTrue(expr) => expression_refs(expr, schema, refs)?,
        CompiledAssertion::DerivedCompare { derived_index, .. } => {
            derived_refs(*derived_index, schema, refs)?
        }
        CompiledAssertion::SubstringEqualsDerivedYear { field, source, .. } => {
            refs.push(field);
            refs.push(source);
        }
        CompiledAssertion::FieldCompare { left, right, .. } => {
            for operand in [left, right] {
                if let CompiledOperand::Field(field) = operand {
                    refs.push(field);
                }
            }
        }
        _ => return Err(RuleDiagnosticError::InvalidPlan),
    }
    Ok(())
}
fn label(reference: &ResolvedFieldRef) -> &str {
    match reference {
        ResolvedFieldRef::Local { field_id, .. } | ResolvedFieldRef::External { field_id, .. } => {
            &field_id.0
        }
    }
}
fn quote(bytes: &[u8]) -> Result<String, RuleDiagnosticError> {
    serde_json::to_string(std::str::from_utf8(bytes).map_err(|_| RuleDiagnosticError::InvalidPlan)?)
        .map_err(|_| RuleDiagnosticError::InvalidPlan)
}
/// Descriptive text only: controlled operators and quoted literal bytes, no
/// runtime field_index, Debug AST output, script or recursive interpolation.
pub(crate) fn condition_text(
    expression: &CompiledExpr,
    schema: &CompiledRecordSchema,
) -> Result<String, RuleDiagnosticError> {
    enum Step<'a> {
        Expr(&'a CompiledExpr),
        Text(&'static str),
    }
    let mut work = vec![Step::Expr(expression)];
    let mut output = String::new();
    while let Some(step) = work.pop() {
        match step {
            Step::Text(text) => output.push_str(text),
            Step::Expr(expr) => match expr {
                CompiledExpr::Const(value) => {
                    output.push_str(if *value { "vero" } else { "falso" })
                }
                CompiledExpr::Eq(reference, value) | CompiledExpr::Ne(reference, value) => output
                    .push_str(&format!(
                        "{} {} {}",
                        label(reference),
                        if matches!(expr, CompiledExpr::Eq(..)) {
                            "="
                        } else {
                            "!="
                        },
                        quote(value)?
                    )),
                CompiledExpr::In(reference, values) | CompiledExpr::NotIn(reference, values) => {
                    let values = values
                        .iter()
                        .map(|value| quote(value))
                        .collect::<Result<Vec<_>, _>>()?;
                    output.push_str(&format!(
                        "{} {} [{}]",
                        label(reference),
                        if matches!(expr, CompiledExpr::In(..)) {
                            "IN"
                        } else {
                            "NOT IN"
                        },
                        values.join(", ")
                    ));
                }
                CompiledExpr::IsPresent(reference) => {
                    output.push_str(&format!("{} presente", label(reference)))
                }
                CompiledExpr::IsMissing(reference) => {
                    output.push_str(&format!("{} mancante", label(reference)))
                }
                CompiledExpr::Not(child) => {
                    output.push_str("NOT (");
                    work.push(Step::Text(")"));
                    work.push(Step::Expr(child));
                }
                CompiledExpr::And(children) | CompiledExpr::Or(children) => {
                    output.push('(');
                    work.push(Step::Text(")"));
                    for (index, child) in children.iter().enumerate().rev() {
                        work.push(Step::Expr(child));
                        if index > 0 {
                            work.push(Step::Text(if matches!(expr, CompiledExpr::And(_)) {
                                " AND "
                            } else {
                                " OR "
                            }));
                        }
                    }
                }
                CompiledExpr::DerivedEq {
                    field,
                    derived_index,
                } => {
                    output.push_str(&format!(
                        "{} = {}",
                        label(field),
                        derived_text(*derived_index, schema)?
                    ));
                }
                CompiledExpr::DerivedCompare {
                    derived_index,
                    operator,
                    value,
                } => {
                    output.push_str(&format!(
                        "{} {} {}",
                        derived_text(*derived_index, schema)?,
                        operator_text(*operator),
                        literal_text(value)?
                    ));
                }
            },
        }
    }
    Ok(output)
}

fn derived_text(
    index: usize,
    schema: &CompiledRecordSchema,
) -> Result<String, RuleDiagnosticError> {
    Ok(
        match schema
            .derived_values()
            .get(index)
            .ok_or(RuleDiagnosticError::InvalidPlan)?
        {
            DerivedValuePlan::Substring {
                source,
                target,
                range,
            } => format!(
                "sottostringa {} di {} [start {}, length {}]",
                match target {
                    ValueTarget::Raw => "raw",
                    ValueTarget::Logical => "logical",
                },
                label(source),
                range
                    .start()
                    .checked_add(1)
                    .ok_or(RuleDiagnosticError::InvalidPlan)?,
                range.len()
            ),
            DerivedValuePlan::CalendarYearDelta {
                birth, reference, ..
            } => format!("differenza anni({}, {})", label(birth), label(reference)),
            DerivedValuePlan::AgeInCompletedYears {
                birth,
                reference,
                leap_day,
                ..
            } => format!(
                "anni compiuti({}, {}; anniversario 29/02: {})",
                label(birth),
                label(reference),
                match leap_day {
                    LeapDayAnniversary::Feb28 => "28/02",
                    LeapDayAnniversary::Mar01 => "01/03",
                }
            ),
        },
    )
}
fn operator_text(operator: ComparisonOperator) -> &'static str {
    match operator {
        ComparisonOperator::Eq => "=",
        ComparisonOperator::Ne => "!=",
        ComparisonOperator::Lt => "<",
        ComparisonOperator::Le => "<=",
        ComparisonOperator::Gt => ">",
        ComparisonOperator::Ge => ">=",
    }
}
fn literal_text(value: &CompiledValue) -> Result<String, RuleDiagnosticError> {
    match value {
        CompiledValue::Bytes(bytes) => quote(bytes),
        CompiledValue::Integer(value) => Ok(format!(
            "{}{}",
            if value.is_negative() { "-" } else { "" },
            std::str::from_utf8(value.magnitude()).map_err(|_| RuleDiagnosticError::InvalidPlan)?
        )),
        CompiledValue::Date(_) => Err(RuleDiagnosticError::InvalidPlan),
    }
}
