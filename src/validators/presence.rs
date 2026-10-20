use super::FieldViolation;
use crate::{
    domain::ValidationCode,
    input::record::FieldSlice,
    schema::compiled::{BasePresence, CompiledFieldSpec, MissingValuePolicy},
};

/// Presence is separate from lexical/semantic validity. MissingOptional stops
/// the field pipeline successfully; Unknown cannot be treated as Present.
/// The latter is the explicit boundary for conditions not evaluated in P6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceValidationResult {
    Present,
    MissingOptional,
    Unknown,
}

/// Validate unconditional Required/Optional against unmodified source bytes.
///
/// DET §42's evaluation context is omitted for this base-only stage (DET §147):
/// no expression evaluation exists here. Nonempty condition plans return
/// Unknown, never silently falling back to the base policy. Full conditional
/// presence/forbidden evaluation remains P10's responsibility.
///
/// Field width must already have been checked by the record pipeline. Neither
/// presence nor is_missing trims, decodes, validates padding or checks datatype.
pub fn validate_presence(
    field: &CompiledFieldSpec,
    value: FieldSlice<'_>,
) -> Result<PresenceValidationResult, FieldViolation> {
    let policy = field.presence_policy();
    // Even a seemingly constant condition belongs to the future evaluator.
    // Refusing to decide prevents a false success or missing-required error.
    if !policy.required_if.is_empty() || !policy.forbidden_if.is_empty() {
        return Ok(PresenceValidationResult::Unknown);
    }
    if !is_missing(field.missing_value_policy(), value.as_bytes()) {
        return Ok(PresenceValidationResult::Present);
    }
    // Only the missing+Required branch allocates a lightweight diagnostic.
    // ErrorFactory later adds record identity, positions and the stable message.
    match policy.base {
        BasePresence::Required => Err(FieldViolation {
            code: ValidationCode::RequiredField,
            expected: Some("valore presente".into()),
            reason: String::new(),
            diagnostic_context: vec![],
        }),
        BasePresence::Optional => Ok(PresenceValidationResult::MissingOptional),
    }
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
