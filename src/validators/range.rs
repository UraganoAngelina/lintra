//! Inclusive integer bounds over an already validated Numeric logical value.
use super::FieldViolation;
use crate::{domain::ValidationCode, input::record::FieldSlice, schema::compiled::IntegerLiteral};
use std::cmp::Ordering;

/// Numeric datatype has already established nonempty ASCII digits. Borrow only
/// the significant magnitude: leading zeros affect representation, never value.
/// Length then lexicographic comparison is exact even beyond u128::MAX.
pub(crate) fn validate_numeric_range(
    min: &IntegerLiteral,
    max: &IntegerLiteral,
    logical: FieldSlice<'_>,
) -> Result<(), FieldViolation> {
    let bytes = logical.as_bytes();
    let first = bytes
        .iter()
        .position(|&digit| digit != b'0')
        .unwrap_or(bytes.len().saturating_sub(1));
    let magnitude = &bytes[first..];
    let compare = |bound: &IntegerLiteral| {
        if bound.is_negative() {
            Ordering::Greater // Numeric record values have no sign by contract.
        } else {
            magnitude
                .len()
                .cmp(&bound.magnitude().len())
                .then_with(|| magnitude.cmp(bound.magnitude()))
        }
    };
    if compare(min) != Ordering::Less && compare(max) != Ordering::Greater {
        Ok(())
    } else {
        Err(FieldViolation {
            code: ValidationCode::OutOfRange,
            expected: Some(format!("un intero da {min} a {max}, estremi inclusi")),
            reason: String::new(),
            diagnostic_context: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_magnitudes_cover_zero_negative_bounds_and_unbounded_width() {
        let cases = [
            ("0", "0", "000", true),
            ("-2", "-1", "0", false),
            ("-2", "0", "000", true),
            ("10", "20", "00010", true),
            ("10", "20", "20", true),
            ("10", "20", "9", false),
            ("10", "20", "21", false),
            (
                "340282366920938463463374607431768211456",
                "340282366920938463463374607431768211457",
                "340282366920938463463374607431768211456",
                true,
            ),
            (
                "340282366920938463463374607431768211456",
                "340282366920938463463374607431768211457",
                "340282366920938463463374607431768211458",
                false,
            ),
        ];
        for (min, max, value, expected) in cases {
            let result = validate_numeric_range(
                &min.parse().unwrap(),
                &max.parse().unwrap(),
                FieldSlice {
                    raw: value.as_bytes(),
                },
            );
            assert_eq!(result.is_ok(), expected, "{min}..={max}: {value}");
        }
    }
}
