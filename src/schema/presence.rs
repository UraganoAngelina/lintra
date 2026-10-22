//! Lower the already-normalized authoring policy without discarding conditions.
use super::{
    compiled::{BasePresence, CompiledPresencePolicy},
    expression::compile_local_expr,
    linter::SchemaIssueCode,
    raw::{RawPresence, RawPresenceBase, RawRecordSchema},
};

/// RequiredUnless is authored as required_if Not(E), per ratified CNF-10.
/// No additional raw spelling, default or mutable runtime flag is introduced.
pub(crate) fn compile_presence(
    policy: &RawPresence,
    record: &RawRecordSchema,
) -> Result<CompiledPresencePolicy, (SchemaIssueCode, String)> {
    let compiled = CompiledPresencePolicy {
        base: match policy.base {
            RawPresenceBase::Required => BasePresence::Required,
            RawPresenceBase::Optional => BasePresence::Optional,
        },
        required_if: policy
            .required_if
            .iter()
            .map(|expr| compile_local_expr(expr, record))
            .collect::<Result<_, _>>()?,
        forbidden_if: policy
            .forbidden_if
            .iter()
            .map(|expr| compile_local_expr(expr, record))
            .collect::<Result<_, _>>()?,
    };
    let required = compiled.base == BasePresence::Required
        || compiled
            .required_if
            .iter()
            .any(|expr| static_truth(expr) == Some(true));
    let forbidden = compiled
        .forbidden_if
        .iter()
        .any(|expr| static_truth(expr) == Some(true));
    if required && forbidden {
        return Err((
            SchemaIssueCode::PresenceConflict,
            "presence is unconditionally both required and forbidden".into(),
        ));
    }
    Ok(compiled)
}

/// Conservative constant interpretation only, never evaluation of record data.
/// Unknown fields remain dynamic; decisive constants in AND/OR may still prove
/// a result. Explicit stacks also cover long expanded named-condition chains.
fn static_truth(expression: &super::compiled::CompiledExpr) -> Option<bool> {
    use super::compiled::CompiledExpr;
    enum Step<'a> {
        Expr(&'a CompiledExpr),
        Group { and: bool, count: usize },
        Not,
    }
    let mut work = vec![Step::Expr(expression)];
    let mut values = Vec::new();
    while let Some(step) = work.pop() {
        match step {
            Step::Expr(expr) => match expr {
                CompiledExpr::Const(value) => values.push(Some(*value)),
                CompiledExpr::Not(child) => {
                    work.push(Step::Not);
                    work.push(Step::Expr(child));
                }
                CompiledExpr::And(children) | CompiledExpr::Or(children) => {
                    if children.is_empty() {
                        values.push(None);
                        continue;
                    }
                    work.push(Step::Group {
                        and: matches!(expr, CompiledExpr::And(_)),
                        count: children.len(),
                    });
                    work.extend(children.iter().rev().map(Step::Expr));
                }
                _ => values.push(None),
            },
            Step::Not => {
                let value = values.pop()?;
                values.push(value.map(|value| !value));
            }
            Step::Group { and, count } => {
                let start = values.len().checked_sub(count)?;
                // None means dynamic, not a failed computation: keep scanning
                // because a later False/True may decide AND/OR respectively.
                let mut result = Some(and);
                for right in values.drain(start..) {
                    result = match (and, result, right) {
                        (true, Some(false), _) | (true, _, Some(false)) => Some(false),
                        (false, Some(true), _) | (false, _, Some(true)) => Some(true),
                        (_, Some(left), Some(right)) => {
                            Some(if and { left && right } else { left || right })
                        }
                        _ => None,
                    };
                }
                values.push(result);
            }
        }
    }
    if values.len() != 1 {
        return None;
    }
    values.pop().flatten()
}
