//! Presence resolution stays in engine; schema/validators never import engine.
use super::expression_evaluator::{
    EvaluationContext, ExpressionEvaluator, TruthValue, UnknownReason,
};
use crate::{
    FieldSlice,
    schema::compiled::{BasePresence, CompiledFieldSpec, CompiledPresencePolicy},
    validators::{
        FieldViolation,
        presence::{DecidedPresence, PresenceValidationResult, validate_decided_presence},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceState {
    Required,
    Optional,
    Forbidden,
    Conflict,
    Unknown(UnknownReason),
}

/// Conditions within each list are ORed in declared order. The base required
/// obligation persists; a forbidden obligation cannot silently override it.
pub fn resolve_presence(
    policy: &CompiledPresencePolicy,
    ctx: &EvaluationContext<'_>,
) -> PresenceState {
    let required = policy.required_if.iter().fold(
        if policy.base == BasePresence::Required {
            TruthValue::True
        } else {
            TruthValue::False
        },
        |value, expr| value.or(ExpressionEvaluator.evaluate(expr, ctx)),
    );
    let forbidden = policy
        .forbidden_if
        .iter()
        .fold(TruthValue::False, |value, expr| {
            value.or(ExpressionEvaluator.evaluate(expr, ctx))
        });
    match (required, forbidden) {
        (TruthValue::True, TruthValue::True) => PresenceState::Conflict,
        (TruthValue::True, TruthValue::False) => PresenceState::Required,
        (TruthValue::False, TruthValue::True) => PresenceState::Forbidden,
        (TruthValue::False, TruthValue::False) => PresenceState::Optional,
        (TruthValue::Unknown(reason), _) | (_, TruthValue::Unknown(reason)) => {
            PresenceState::Unknown(reason)
        }
    }
}

/// Pure bridge: evaluation is owned by engine, byte-presence checks by validators.
/// Unknown has no decided policy and therefore never reaches semantic checks.
pub(crate) fn validate_presence_state(
    state: PresenceState,
    field: &CompiledFieldSpec,
    value: FieldSlice<'_>,
) -> Result<PresenceValidationResult, FieldViolation> {
    let decided = match state {
        PresenceState::Required => DecidedPresence::Required,
        PresenceState::Optional => DecidedPresence::Optional,
        PresenceState::Forbidden => DecidedPresence::Forbidden,
        PresenceState::Conflict => DecidedPresence::Conflict,
        PresenceState::Unknown(_) => return Ok(PresenceValidationResult::Unknown),
    };
    validate_decided_presence(field, value, decided)
}
