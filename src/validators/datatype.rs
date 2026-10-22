//! Base datatype validation of an extracted logical value, not a normative domain test.
use super::FieldViolation;
use crate::{
    domain::ValidationCode,
    input::{encoding::is_valid_encoded_text, record::FieldSlice},
    schema::compiled::{CompiledFieldSpec, EncodingSpec, FieldType, LexicalPlan},
};
use std::{error::Error, fmt};

/// Invalid data is a controlled result, ready for ErrorFactory enrichment.
/// It must not be propagated as a technical ValidationRunError by the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatatypeValidationResult {
    Valid,
    Invalid(FieldViolation),
}

/// Only unsupported/inconsistent plans are technical failures. Alphabetic and
/// Decimal remain absent from format 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatatypeError {
    UnsupportedType(FieldType),
    UnsupportedEncoding(EncodingSpec),
    MissingLexicalPlan,
}
impl fmt::Display for DatatypeError {
    /// Fixed descriptions keep actual field bytes out of technical messages.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnsupportedType(_) => {
                "datatype is outside this lexical validator's supported subset"
            }
            Self::UnsupportedEncoding(_) => "encoding is not supported by the lexical validator",
            Self::MissingLexicalPlan => "compiled field has no lexical plan",
        })
    }
}
// Metadata inconsistency is the cause itself, with no nested I/O exception.
impl Error for DatatypeError {}

/// Validate Numeric/Alphanumeric/Date/Decimal against the explicit encoding and field plan.
/// DET §147 refinement of §45: lexical class and encoding are indispensable
/// metadata, and outer Err contains only technical errors, never violations.
/// Call only after presence and logical extraction; this function never trims
/// padding, normalizes case, parses a number, checks a domain or mutates input.
pub fn validate_datatype(
    field: &CompiledFieldSpec,
    logical: FieldSlice<'_>,
    encoding: &EncodingSpec,
) -> Result<DatatypeValidationResult, DatatypeError> {
    let kind = field.field_type();
    if let FieldType::Decimal(spec) = kind {
        // Exact alignment preserves every physical byte, including separator
        // position. The plan comes only from a checked explicit Decimal node.
        if is_valid_encoded_text(logical.as_bytes(), encoding).is_none() {
            return Err(DatatypeError::UnsupportedEncoding(encoding.clone()));
        }
        return Ok(match super::decimal::validate_decimal(&spec, logical) {
            Ok(()) => DatatypeValidationResult::Valid,
            Err(violation) => DatatypeValidationResult::Invalid(violation),
        });
    }
    if let FieldType::Date(format) = kind {
        // Dates have an explicit ASCII representation instead of an authoring
        // lexical class. Reject unknown encoding plans without assuming a profile;
        // the date validator itself then checks every byte and calendar component.
        if is_valid_encoded_text(logical.as_bytes(), encoding).is_none() {
            return Err(DatatypeError::UnsupportedEncoding(encoding.clone()));
        }
        return Ok(match super::date::validate_date(format, logical) {
            Ok(()) => DatatypeValidationResult::Valid,
            Err(violation) => DatatypeValidationResult::Invalid(violation),
        });
    }
    let lexical = field
        .lexical()
        .as_ref()
        .ok_or(DatatypeError::MissingLexicalPlan)?;
    let bytes = logical.as_bytes();
    let encoded = is_valid_encoded_text(bytes, encoding)
        .ok_or_else(|| DatatypeError::UnsupportedEncoding(encoding.clone()))?;
    if !encoded {
        return Ok(invalid("testo valido per il profilo di encoding esplicito"));
    }
    // ASCII digits only, with at least one byte. No integer-size bound and no
    // accidental acceptance of signs, separators or Unicode numeric symbols.
    if kind == FieldType::Numeric && (bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit)) {
        return Ok(invalid("almeno una cifra ASCII da 0 a 9"));
    }
    // ByteSet is an additional explicit lexical restriction, even on Numeric.
    // Lowercase membership alone does not prohibit a letter: no_lowercase is a
    // separate constraint, and case folding is never inferred from AN.
    if let LexicalPlan::ByteSet(class) = lexical
        && !bytes.iter().all(|&byte| class.allowed[usize::from(byte)])
    {
        return Ok(invalid("byte ammessi dalla classe lessicale dello schema"));
    }
    Ok(DatatypeValidationResult::Valid)
}

/// Construct only the lightweight primary violation; record identity, raw
/// rendering, range and message remain ErrorFactory's responsibility.
fn invalid(expected: &str) -> DatatypeValidationResult {
    DatatypeValidationResult::Invalid(FieldViolation {
        code: ValidationCode::InvalidType,
        expected: Some(expected.into()),
        reason: String::new(),
        diagnostic_context: vec![],
    })
}
