//! Calendar validation of the explicit format-1 DDMMYYYY byte representation.
use super::FieldViolation;
use crate::{
    domain::ValidationCode,
    input::record::FieldSlice,
    schema::compiled::{DateFormat, DateLiteral},
};

/// No decoding, trimming, timezone or host clock. Syntax checks precede every
/// fixed byte index; chrono checks the Gregorian calendar without panicking.
pub(crate) fn validate_date(
    format: DateFormat,
    value: FieldSlice<'_>,
) -> Result<(), FieldViolation> {
    let bytes = value.as_bytes();
    let valid = match format {
        DateFormat::DdMmYyyy => DateLiteral::from_ddmmyyyy(bytes).is_some(),
    };
    if valid {
        Ok(())
    } else {
        Err(FieldViolation {
            code: ValidationCode::InvalidDate,
            expected: Some("data DDMMYYYY valida, anno da 0001 a 9999".into()),
            reason: String::new(),
            diagnostic_context: vec![],
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_format_and_calendar_boundaries() {
        for (value, valid) in [
            ("01010001", true),
            ("31129999", true),
            ("29022000", true),
            ("29022024", true),
            ("29021900", false),
            ("29022023", false),
            ("31042026", false),
            ("00012026", false),
            ("01132026", false),
            ("01010000", false),
            ("20261231", false),
            ("01/01/26", false),
            ("0101202", false),
            ("010120260", false),
            ("٠١٠١٢٠٢٦", false),
            (" 1012026", false),
        ] {
            assert_eq!(
                validate_date(
                    DateFormat::DdMmYyyy,
                    FieldSlice {
                        raw: value.as_bytes()
                    }
                )
                .is_ok(),
                valid,
                "{value}"
            );
        }
    }
}
