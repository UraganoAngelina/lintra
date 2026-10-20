//! Exact membership only: raw/logical target selection belongs to the caller.
use super::FieldViolation;
use crate::{domain::ValidationCode, input::record::FieldSlice};

/// Compare every byte, including padding, case, leading zeros and encoding.
/// The compiler validates a nonempty domain and literal widths once; runtime
/// neither pads literals nor parses/normalizes data to obtain a match.
pub fn validate_allowed_values(
    values: &[Vec<u8>],
    value: FieldSlice<'_>,
) -> Result<(), FieldViolation> {
    if values
        .iter()
        .any(|allowed| allowed.as_slice() == value.as_bytes())
    {
        return Ok(());
    }
    // ErrorFactory enriches this single primary failure; do not copy a large
    // domain into every diagnostic or construct record metadata in a validator.
    Err(FieldViolation {
        code: ValidationCode::InvalidDomain,
        expected: Some("uno dei valori del dominio dichiarato".into()),
        reason: String::new(),
        diagnostic_context: vec![],
    })
}
