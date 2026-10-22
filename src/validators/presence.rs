use super::FieldViolation;
use crate::{
    domain::ValidationCode,
    input::record::FieldSlice,
    schema::compiled::{BasePresence, CompiledFieldSpec, MissingValuePolicy},
};

/// Presence is separate from lexical/semantic validity. MissingOptional stops
/// the field pipeline successfully; Unknown cannot be treated as Present.
/// Conditional decisions require the context-aware engine pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceValidationResult {
    Present,
    MissingOptional,
    Unknown,
}

/// Validate unconditional Required/Optional against unmodified source bytes.
///
/// DET §42's evaluation context is omitted for this base-only stage (DET §147):
/// expression evaluation belongs to engine. Without that context, nonempty
/// condition plans return Unknown, never silently falling back to the base
/// policy. FieldPipeline::run_with_context executes conditional presence.
///
/// Field width must already have been checked by the record pipeline. Neither
/// presence nor is_missing trims, decodes, validates padding or checks datatype.
pub fn validate_presence(
    field: &CompiledFieldSpec,
    value: FieldSlice<'_>,
) -> Result<PresenceValidationResult, FieldViolation> {
    let policy = field.presence_policy();
    // Even a constant condition needs the context-aware executor path.
    // Refusing to decide prevents a false success or missing-required error.
    if !policy.required_if.is_empty() || !policy.forbidden_if.is_empty() {
        return Ok(PresenceValidationResult::Unknown);
    }
    let state = match policy.base {
        BasePresence::Required => DecidedPresence::Required,
        BasePresence::Optional => DecidedPresence::Optional,
    };
    validate_decided_presence(field, value, state)
}

/// An immutable decision passed down from engine, with no evaluator dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecidedPresence {
    Required,
    Optional,
    Forbidden,
    Conflict,
}

pub(crate) fn validate_decided_presence(
    field: &CompiledFieldSpec,
    value: FieldSlice<'_>,
    state: DecidedPresence,
) -> Result<PresenceValidationResult, FieldViolation> {
    let missing = is_missing(field.missing_value_policy(), value.as_bytes());
    let failure = match state {
        DecidedPresence::Conflict => Some((
            ValidationCode::RuleConflict,
            "regole di presenza non in conflitto",
        )),
        DecidedPresence::Forbidden if !missing => {
            Some((ValidationCode::ForbiddenField, "valore assente"))
        }
        DecidedPresence::Required if missing => Some((
            if field.presence_policy().base == BasePresence::Required {
                ValidationCode::RequiredField
            } else {
                ValidationCode::ConditionalRequired
            },
            "valore presente",
        )),
        _ => None,
    };
    if let Some((code, expected)) = failure {
        return Err(FieldViolation {
            code,
            expected: Some(expected.into()),
            reason: String::new(),
            diagnostic_context: vec![],
        });
    }
    Ok(if missing {
        PresenceValidationResult::MissingOptional
    } else {
        PresenceValidationResult::Present
    })
}

/// Tests the unmodified bytes against the schema's explicit missing policy.
/// Field width is checked by the record pipeline, not inferred here.
pub fn is_missing(policy: &MissingValuePolicy, raw: &[u8]) -> bool {
    match policy {
        MissingValuePolicy::Spaces => raw.iter().all(|&byte| byte == b' '),
        MissingValuePolicy::Exact(value) => raw == value,
        MissingValuePolicy::AnyOf(values) => values.iter().any(|value| raw == value),
        MissingValuePolicy::NeverMissing => false,
    }
}
