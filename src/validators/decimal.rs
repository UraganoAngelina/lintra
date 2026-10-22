//! Representation checks from a verified Decimal plan; no currency/flow rules.
use super::FieldViolation;
use crate::{DecimalSpec, FieldSlice, SignPolicy, ValidationCode};

/// Preserve the full physical width. The separator's position follows only
/// from the plan's scale; every other byte must be an ASCII digit. This accepts
/// exact zero/leading zeros and rejects signs, grouping, whitespace and exponents.
/// No arithmetic conversion, allocation or input mutation occurs on success.
pub(crate) fn validate_decimal(
    spec: &DecimalSpec,
    value: FieldSlice<'_>,
) -> Result<(), FieldViolation> {
    let bytes = value.as_bytes();
    let separator_index = spec.total_length() - spec.scale() - 1;
    let valid = match spec.sign_policy() {
        SignPolicy::Unsigned => {
            bytes.len() == spec.total_length()
                && bytes.iter().enumerate().all(|(index, byte)| {
                    if index == separator_index {
                        *byte == spec.decimal_separator()
                    } else {
                        byte.is_ascii_digit()
                    }
                })
        }
    };
    if valid {
        Ok(())
    } else {
        Err(FieldViolation {
            code: ValidationCode::InvalidType,
            expected: Some(format!(
                "decimale senza segno di {} byte, separatore '{}' e {} cifre frazionarie",
                spec.total_length(),
                char::from(spec.decimal_separator()),
                spec.scale()
            )),
            reason: String::new(),
            diagnostic_context: vec![],
        })
    }
}
