//! Local expression lowering only; execution belongs to the engine (P10-03).
use super::{
    compiled::{CompiledExpr, ResolvedFieldRef},
    linter::SchemaIssueCode as Code,
    raw::{RawExpr, RawRecordSchema, RawRef},
};
use crate::FieldId;
use std::collections::HashSet;

type Issue = (Code, String);

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
    let mut work = vec![Step::Expr(expression)];
    let mut values = Vec::new();
    let mut active_names = HashSet::new();
    while let Some(step) = work.pop() {
        match step {
            Step::Expr(expression) => match expression {
                RawExpr::Eq(node) => values.push(CompiledExpr::Eq(
                    resolve(&node.r#ref, record)?,
                    node.value.as_bytes().to_vec(),
                )),
                RawExpr::Ne(node) => values.push(CompiledExpr::Ne(
                    resolve(&node.r#ref, record)?,
                    node.value.as_bytes().to_vec(),
                )),
                RawExpr::In(node) => values.push(CompiledExpr::In(
                    resolve(&node.r#ref, record)?,
                    literals(&node.values)?,
                )),
                RawExpr::NotIn(node) => values.push(CompiledExpr::NotIn(
                    resolve(&node.r#ref, record)?,
                    literals(&node.values)?,
                )),
                RawExpr::IsPresent(node) => {
                    values.push(CompiledExpr::IsPresent(resolve(&node.r#ref, record)?))
                }
                RawExpr::IsMissing(node) => {
                    values.push(CompiledExpr::IsMissing(resolve(&node.r#ref, record)?))
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
                RawExpr::DerivedEq(_) | RawExpr::DerivedCompare(_) => {
                    return Err((
                        Code::UnsupportedConstruct,
                        "derived expressions require a compiled derived-value plan".into(),
                    ));
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
fn resolve(reference: &RawRef, record: &RawRecordSchema) -> Result<ResolvedFieldRef, Issue> {
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
