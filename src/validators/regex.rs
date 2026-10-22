//! Byte matching with a program compiled before any record reaches the engine.
use super::FieldViolation;
use crate::{FieldSlice, ValidationCode, schema::compiled::RegexMatch};
use regex::bytes::Regex;

/// Required/forbidden invert only the predicate, never byte order or content.
/// Patterns select anchoring/classes explicitly; the engine adds no anchors,
/// Unicode normalization, trimming, decoding or runtime compilation.
pub(crate) fn validate_regex(
    regex: &Regex,
    mode: RegexMatch,
    value: FieldSlice<'_>,
) -> Result<(), FieldViolation> {
    let matched = regex.is_match(value.as_bytes());
    let valid = match mode {
        RegexMatch::Required => matched,
        RegexMatch::Forbidden => !matched,
    };
    if valid {
        Ok(())
    } else {
        Err(FieldViolation {
            code: ValidationCode::InvalidDomain,
            expected: Some(format!(
                "{} il pattern dichiarato {}",
                match mode {
                    RegexMatch::Required => "corrispondenza con",
                    RegexMatch::Forbidden => "nessuna corrispondenza con",
                },
                regex.as_str()
            )),
            reason: String::new(),
            diagnostic_context: vec![],
        })
    }
}
