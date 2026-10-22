//! Pure three-valued expression evaluation over resolved byte values.
use std::ops::Not;

/// Unavailability is diagnostic information, never a false value or a pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownReason {
    FieldUnavailable,
    RelatedRecordMissing,
    TraceInputUnavailable,
    AmbiguousJoin,
    /// A directly supplied plan has no executor; never a simulated decision.
    UnsupportedPlan,
    /// Present bytes cannot be interpreted as the explicitly requested type.
    InvalidFieldValue,
}

/// A determinate truth value or an explicitly unavailable decision.
/// No implicit conversion to bool is provided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruthValue {
    True,
    False,
    Unknown(UnknownReason),
}
impl TruthValue {
    /// False determines AND even when the other operand is unavailable.
    /// Otherwise preserve Unknown; two Unknowns retain the left operand's
    /// reason, following declared expression order deterministically.
    pub const fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::False, _) | (_, Self::False) => Self::False,
            (Self::Unknown(reason), _) | (_, Self::Unknown(reason)) => Self::Unknown(reason),
            (Self::True, Self::True) => Self::True,
        }
    }

    /// True determines OR even when the other operand is unavailable.
    /// The Unknown tie policy is the same stable left-first choice as AND.
    pub const fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::True, _) | (_, Self::True) => Self::True,
            (Self::Unknown(reason), _) | (_, Self::Unknown(reason)) => Self::Unknown(reason),
            (Self::False, Self::False) => Self::False,
        }
    }
}
impl Not for TruthValue {
    type Output = Self;

    /// Negation changes determinate truth only, retaining unavailability.
    fn not(self) -> Self::Output {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown(reason) => Self::Unknown(reason),
        }
    }
}

use crate::schema::compiled::{CompiledExpr, ResolvedFieldRef};
use crate::{CompiledRecordSchema, FieldSlice, RecordView, is_missing, logical_value};

/// A missing value is distinct from a present empty logical byte slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedFieldValue<'a> {
    Present(&'a [u8]),
    Missing,
    Unavailable(UnknownReason),
}

/// Pure, borrowed field access. Future projected/related contexts can implement
/// this port without coupling expression evaluation to storage or application.
pub trait FieldValueResolver {
    fn resolve_field(&self, reference: &ResolvedFieldRef) -> ResolvedFieldValue<'_>;
    /// A resolver without physical access must report it explicitly; logical
    /// bytes are never substituted for a requested raw target.
    fn resolve_raw_field(&self, _reference: &ResolvedFieldRef) -> ResolvedFieldValue<'_> {
        ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan)
    }
}

/// The evaluator depends on the resolver abstraction, not on an I/O provider.
pub struct EvaluationContext<'a> {
    pub fields: &'a dyn FieldValueResolver,
}

/// Reads a real local record through its checked compiled field ranges.
pub struct LocalEvaluationContext<'a> {
    pub record: &'a RecordView<'a>,
    pub schema: &'a CompiledRecordSchema,
}
impl<'a> LocalEvaluationContext<'a> {
    /// Invalid record width or index is unavailable, never unchecked slicing.
    /// This raw accessor does not infer missingness or modify source bytes.
    pub fn resolve_local(&self, field_index: usize) -> Option<&'a [u8]> {
        if self.record.raw().len() != self.schema.record_length() {
            return None;
        }
        let field = self.schema.fields().get(field_index)?;
        self.record.field(field).map(|value| value.raw)
    }
}
impl LocalEvaluationContext<'_> {
    fn resolve_value(
        &self,
        reference: &ResolvedFieldRef,
        raw_target: bool,
    ) -> ResolvedFieldValue<'_> {
        let ResolvedFieldRef::Local {
            field_index,
            field_id,
        } = reference
        else {
            return ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan);
        };
        let Some(field) = self
            .schema
            .fields()
            .get(*field_index)
            .filter(|field| field.id() == field_id)
        else {
            return ResolvedFieldValue::Unavailable(UnknownReason::FieldUnavailable);
        };
        let Some(raw) = self.resolve_local(*field_index) else {
            return ResolvedFieldValue::Unavailable(UnknownReason::FieldUnavailable);
        };
        if is_missing(field.missing_value_policy(), raw) {
            return ResolvedFieldValue::Missing;
        }
        if raw_target {
            return ResolvedFieldValue::Present(raw);
        }
        match logical_value(&field.alignment(), FieldSlice { raw }) {
            Ok(value) => ResolvedFieldValue::Present(value.raw),
            Err(_) => ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan),
        }
    }
}
impl FieldValueResolver for LocalEvaluationContext<'_> {
    fn resolve_field(&self, reference: &ResolvedFieldRef) -> ResolvedFieldValue<'_> {
        self.resolve_value(reference, false)
    }
    fn resolve_raw_field(&self, reference: &ResolvedFieldRef) -> ResolvedFieldValue<'_> {
        self.resolve_value(reference, true)
    }
}

/// Stateless execution over immutable plans/values. No I/O or record mutation.
pub struct ExpressionEvaluator;
enum EvalStep<'a> {
    Expr(&'a CompiledExpr),
    Group { and: bool, count: usize },
    Not,
}
impl ExpressionEvaluator {
    /// Evaluate in declared order with explicit stacks, avoiding recursive calls
    /// on nested input trees. Every unsupported/inconsistent direct plan is
    /// Unknown(UnsupportedPlan); the schema compiler already rejects such plans.
    pub fn evaluate(&self, expression: &CompiledExpr, ctx: &EvaluationContext<'_>) -> TruthValue {
        let mut work = vec![EvalStep::Expr(expression)];
        let mut values = Vec::new();
        while let Some(step) = work.pop() {
            match step {
                EvalStep::Expr(expression) => match expression {
                    CompiledExpr::Const(value) => values.push(truth(*value)),
                    CompiledExpr::Eq(reference, expected)
                    | CompiledExpr::Ne(reference, expected) => {
                        let value = match ctx.fields.resolve_field(reference) {
                            ResolvedFieldValue::Present(actual) => truth(
                                (actual == expected) == matches!(expression, CompiledExpr::Eq(..)),
                            ),
                            ResolvedFieldValue::Missing => {
                                TruthValue::Unknown(UnknownReason::FieldUnavailable)
                            }
                            ResolvedFieldValue::Unavailable(reason) => TruthValue::Unknown(reason),
                        };
                        values.push(value);
                    }
                    CompiledExpr::In(reference, domain)
                    | CompiledExpr::NotIn(reference, domain) => {
                        let value = if domain.is_empty() {
                            unsupported()
                        } else {
                            match ctx.fields.resolve_field(reference) {
                                ResolvedFieldValue::Present(actual) => truth(
                                    domain.iter().any(|value| value.as_slice() == actual)
                                        == matches!(expression, CompiledExpr::In(..)),
                                ),
                                ResolvedFieldValue::Missing => {
                                    TruthValue::Unknown(UnknownReason::FieldUnavailable)
                                }
                                ResolvedFieldValue::Unavailable(reason) => {
                                    TruthValue::Unknown(reason)
                                }
                            }
                        };
                        values.push(value);
                    }
                    CompiledExpr::IsPresent(reference) | CompiledExpr::IsMissing(reference) => {
                        let value = match ctx.fields.resolve_field(reference) {
                            ResolvedFieldValue::Present(_) => {
                                truth(matches!(expression, CompiledExpr::IsPresent(_)))
                            }
                            ResolvedFieldValue::Missing => {
                                truth(matches!(expression, CompiledExpr::IsMissing(_)))
                            }
                            ResolvedFieldValue::Unavailable(reason) => TruthValue::Unknown(reason),
                        };
                        values.push(value);
                    }
                    CompiledExpr::And(children) | CompiledExpr::Or(children) => {
                        if children.is_empty() {
                            values.push(unsupported());
                            continue;
                        }
                        work.push(EvalStep::Group {
                            and: matches!(expression, CompiledExpr::And(_)),
                            count: children.len(),
                        });
                        work.extend(children.iter().rev().map(EvalStep::Expr));
                    }
                    CompiledExpr::Not(child) => {
                        work.push(EvalStep::Not);
                        work.push(EvalStep::Expr(child));
                    }
                    CompiledExpr::DerivedEq { .. } | CompiledExpr::DerivedCompare { .. } => {
                        values.push(unsupported())
                    }
                },
                EvalStep::Group { and, count } => {
                    let Some(start) = values.len().checked_sub(count) else {
                        return unsupported();
                    };
                    let value = values.drain(start..).fold(
                        if and {
                            TruthValue::True
                        } else {
                            TruthValue::False
                        },
                        |left, right| if and { left.and(right) } else { left.or(right) },
                    );
                    values.push(value);
                }
                EvalStep::Not => {
                    let Some(value) = values.pop() else {
                        return unsupported();
                    };
                    values.push(!value);
                }
            }
        }
        if values.len() != 1 {
            return unsupported();
        }
        values.pop().unwrap_or_else(unsupported)
    }
}
fn truth(value: bool) -> TruthValue {
    if value {
        TruthValue::True
    } else {
        TruthValue::False
    }
}
fn unsupported() -> TruthValue {
    TruthValue::Unknown(UnknownReason::UnsupportedPlan)
}

/// Runtime boundary for directly assembled private plans as well as future
/// compiler extensions. Derived/external expressions need their actual executor.
pub(crate) fn local_expression_supported(
    expression: &crate::schema::compiled::CompiledExpr,
) -> bool {
    use crate::schema::compiled::{CompiledExpr, ResolvedFieldRef};
    let mut work = vec![expression];
    while let Some(expr) = work.pop() {
        match expr {
            CompiledExpr::Const(_) => {}
            CompiledExpr::In(_, domain) | CompiledExpr::NotIn(_, domain) if domain.is_empty() => {
                return false;
            }
            CompiledExpr::Eq(reference, _)
            | CompiledExpr::Ne(reference, _)
            | CompiledExpr::In(reference, _)
            | CompiledExpr::NotIn(reference, _)
            | CompiledExpr::IsMissing(reference)
            | CompiledExpr::IsPresent(reference) => {
                if !matches!(reference, ResolvedFieldRef::Local { .. }) {
                    return false;
                }
            }
            CompiledExpr::And(children) | CompiledExpr::Or(children) => {
                if children.is_empty() {
                    return false;
                }
                work.extend(children);
            }
            CompiledExpr::Not(child) => work.push(child),
            CompiledExpr::DerivedEq { .. } | CompiledExpr::DerivedCompare { .. } => return false,
        }
    }
    true
}
