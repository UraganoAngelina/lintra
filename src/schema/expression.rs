//! Local expression lowering only; execution belongs to the engine (P10-03).
use super::{
    compiled::{ComparisonOperator, CompiledExpr, ResolvedFieldRef, ValueType},
    linter::SchemaIssueCode as Code,
    raw::{RawExpr, RawOperator, RawPhysical, RawRecordSchema, RawRef},
};
use crate::FieldId;
use std::collections::HashSet;

type Issue = (Code, String);

/// Checked reference binding over an already compiled flow. Public flow
/// compilation still gates cross-rule AST until the pass-2 integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldReferenceError {
    SourceTraceMissing,
    TargetTraceMissing,
    FieldMissing,
    JoinMissing,
    JoinSourceMismatch,
    JoinTargetMismatch,
    ScalarRequiresSingleCardinality,
    FieldOutsideReduction,
    /// Retained for source compatibility with the initial scalar-only binder.
    UnsupportedReduction,
}
impl std::fmt::Display for FieldReferenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("field reference cannot be resolved against the compiled flow")
    }
}
impl std::error::Error for FieldReferenceError {}

pub fn resolve_field_reference(
    reference: &RawRef,
    source_trace: &crate::TraceId,
    flow: &crate::CompiledFlowSchema,
) -> Result<ResolvedFieldRef, FieldReferenceError> {
    let source_index = flow
        .traces()
        .iter()
        .position(|trace| trace.id() == source_trace)
        .ok_or(FieldReferenceError::SourceTraceMissing)?;
    let source = &flow.traces()[source_index];
    match reference {
        RawRef::Local(reference) => {
            let field_id = crate::FieldId(reference.field.clone());
            let field_index = source
                .field_index(&field_id)
                .ok_or(FieldReferenceError::FieldMissing)?;
            Ok(ResolvedFieldRef::Local {
                field_index,
                field_id,
            })
        }
        RawRef::External(reference) => {
            let (join_index, join) = flow
                .cross_record_plan()
                .joins
                .iter()
                .enumerate()
                .find(|(_, join)| join.id.0 == reference.join)
                .ok_or(FieldReferenceError::JoinMissing)?;
            if join.source_trace != source_index {
                return Err(FieldReferenceError::JoinSourceMismatch);
            }
            let trace_index = flow
                .traces()
                .iter()
                .position(|trace| trace.id().0 == reference.trace)
                .ok_or(FieldReferenceError::TargetTraceMissing)?;
            if join.target_trace != trace_index {
                return Err(FieldReferenceError::JoinTargetMismatch);
            }
            let field_id = crate::FieldId(reference.field.clone());
            let field_index = flow.traces()[trace_index]
                .field_index(&field_id)
                .ok_or(FieldReferenceError::FieldMissing)?;
            if let Some(consensus) = &join.consensus {
                if !consensus.fields.contains(&field_index) {
                    return Err(FieldReferenceError::FieldOutsideReduction);
                }
            } else if !matches!(
                join.cardinality,
                crate::JoinCardinality::ZeroOrOne | crate::JoinCardinality::ExactlyOne
            ) {
                return Err(FieldReferenceError::ScalarRequiresSingleCardinality);
            }
            Ok(ResolvedFieldRef::External {
                trace_index,
                field_index,
                field_id,
                join_index,
                join_id: join.id.clone(),
            })
        }
    }
}

// Explicit postorder work avoids call-stack growth for nested expressions and
// named-condition chains. LeaveName keeps cycle detection local to the path,
// allowing repeated uses of an acyclic definition in independent branches.
enum Step<'a> {
    Expr(&'a RawExpr),
    Group { and: bool, count: usize },
    Not,
    LeaveName(&'a str),
}

/// Caller supplies the same physical field order and explicit UTF-8 binding
/// checked by SchemaCompiler. No field bytes are read or evaluated here.
/// An invalid expression returns a typed issue, never a partial compiled tree.
pub(crate) fn compile_local_expr(
    expression: &RawExpr,
    record: &RawRecordSchema,
) -> Result<CompiledExpr, Issue> {
    compile_expr(
        expression,
        &super::reference_binding::ReferenceBindings::local(record),
    )
}

pub(crate) fn compile_expr(
    expression: &RawExpr,
    bindings: &super::reference_binding::ReferenceBindings<'_>,
) -> Result<CompiledExpr, Issue> {
    let record = bindings.record;
    let mut work = vec![Step::Expr(expression)];
    let mut values = Vec::new();
    let mut active_names = HashSet::new();
    while let Some(step) = work.pop() {
        match step {
            Step::Expr(expression) => match expression {
                RawExpr::Eq(node) => values.push(CompiledExpr::Eq(
                    bindings.resolve(&node.r#ref)?,
                    node.value.as_bytes().to_vec(),
                )),
                RawExpr::Ne(node) => values.push(CompiledExpr::Ne(
                    bindings.resolve(&node.r#ref)?,
                    node.value.as_bytes().to_vec(),
                )),
                RawExpr::In(node) => values.push(CompiledExpr::In(
                    bindings.resolve(&node.r#ref)?,
                    literals(&node.values)?,
                )),
                RawExpr::NotIn(node) => values.push(CompiledExpr::NotIn(
                    bindings.resolve(&node.r#ref)?,
                    literals(&node.values)?,
                )),
                RawExpr::IsPresent(node) => {
                    values.push(CompiledExpr::IsPresent(bindings.resolve(&node.r#ref)?))
                }
                RawExpr::IsMissing(node) => {
                    values.push(CompiledExpr::IsMissing(bindings.resolve(&node.r#ref)?))
                }
                RawExpr::Const(value) => values.push(CompiledExpr::Const(*value)),
                RawExpr::And(children) | RawExpr::Or(children) => {
                    if children.is_empty() {
                        return Err((
                            Code::InvalidSchemaValue,
                            "and/or require nonempty operands; use const for a truth literal"
                                .into(),
                        ));
                    }
                    work.push(Step::Group {
                        and: matches!(expression, RawExpr::And(_)),
                        count: children.len(),
                    });
                    work.extend(children.iter().rev().map(Step::Expr));
                }
                RawExpr::Not(child) => {
                    work.push(Step::Not);
                    work.push(Step::Expr(child));
                }
                RawExpr::Condition(name) => {
                    let definition = record.named_conditions.get(name).ok_or_else(|| {
                        (
                            Code::InvalidSchemaValue,
                            "named condition does not exist".into(),
                        )
                    })?;
                    if !active_names.insert(name.as_str()) {
                        return Err((
                            Code::SelfReferentialRule,
                            "named condition cycle is not executable".into(),
                        ));
                    }
                    work.push(Step::LeaveName(name));
                    work.push(Step::Expr(definition));
                }
                RawExpr::DerivedEq(node) => {
                    let field = bindings.resolve(&node.r#ref)?;
                    let (derived_index, kind) =
                        super::derived::binding(record, &node.derived_value)?;
                    if kind == ValueType::Integer
                        && !matches!(
                            bindings.field(&field)?.physical,
                            RawPhysical::Numeric { .. }
                        )
                    {
                        return Err((
                            Code::ConstraintTypeMismatch,
                            "integer derived_eq requires a numeric field".into(),
                        ));
                    }
                    values.push(CompiledExpr::DerivedEq {
                        field,
                        derived_index,
                    });
                }
                RawExpr::DerivedCompare(node) => {
                    let (derived_index, kind) =
                        super::derived::binding(record, &node.derived_value)?;
                    values.push(CompiledExpr::DerivedCompare {
                        derived_index,
                        operator: operator_plan(&node.operator),
                        value: super::derived::literal(&node.value, kind)?,
                    });
                }
            },
            Step::Group { and, count } => {
                let start = values.len().checked_sub(count).ok_or_else(invariant)?;
                let children = values.split_off(start);
                values.push(if and {
                    CompiledExpr::And(children)
                } else {
                    CompiledExpr::Or(children)
                });
            }
            Step::Not => {
                let child = values.pop().ok_or_else(invariant)?;
                values.push(CompiledExpr::Not(Box::new(child)));
            }
            Step::LeaveName(name) => {
                active_names.remove(name);
            }
        }
    }
    if values.len() != 1 {
        return Err(invariant());
    }
    values.pop().ok_or_else(invariant)
}

/// Logical literals remain exact UTF-8 bytes, with no inferred padding, trim,
/// numeric parsing or case folding. The authoring set is nonempty and unique.
fn literals(values: &[String]) -> Result<Vec<Vec<u8>>, Issue> {
    let unique: HashSet<_> = values.iter().collect();
    if values.is_empty() || unique.len() != values.len() {
        return Err((
            Code::InvalidSchemaValue,
            "in/not_in require nonempty unique literals".into(),
        ));
    }
    Ok(values
        .iter()
        .map(|value| value.as_bytes().to_vec())
        .collect())
}
pub(crate) fn resolve_local_ref(
    reference: &RawRef,
    record: &RawRecordSchema,
) -> Result<ResolvedFieldRef, Issue> {
    let RawRef::Local(reference) = reference else {
        return Err((
            Code::UnsupportedConstruct,
            "external expression references require resolved joins".into(),
        ));
    };
    let field_index = record
        .fields
        .iter()
        .position(|field| field.id == reference.field)
        .ok_or_else(|| {
            (
                Code::MissingFieldReference,
                "local expression field does not exist".into(),
            )
        })?;
    Ok(ResolvedFieldRef::Local {
        field_index,
        field_id: FieldId(reference.field.clone()),
    })
}
fn invariant() -> Issue {
    (
        Code::InternalCompilerInvariant,
        "expression lowering stack is inconsistent".into(),
    )
}

#[cfg(test)]
#[path = "expression_tests.rs"]
mod tests;

pub(crate) fn operator_plan(raw: &RawOperator) -> ComparisonOperator {
    match raw {
        RawOperator::Eq => ComparisonOperator::Eq,
        RawOperator::Ne => ComparisonOperator::Ne,
        RawOperator::Lt => ComparisonOperator::Lt,
        RawOperator::Le => ComparisonOperator::Le,
        RawOperator::Gt => ComparisonOperator::Gt,
        RawOperator::Ge => ComparisonOperator::Ge,
    }
}
