//! Stable validation vocabulary, ARCH §26 and DET §51. Mapping revision 1.

use std::{error::Error, fmt, str::FromStr, sync::Arc};

use super::{ByteRange, FieldId, FlowId, RecordIdentity, SchemaVersion, TraceId};

/// An owned diagnostic representation, independent of decoding or input I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticValue {
    pub display: String,
    pub raw_hex: Option<String>,
}

/// Analyst-facing positions are one-based and inclusive, never runtime offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDiagnostic {
    pub id: FieldId,
    pub name: Arc<str>,
    pub start_one_based: usize,
    pub end_one_based_inclusive: usize,
}

impl FieldDiagnostic {
    /// The single runtime-to-diagnostic position conversion boundary.
    pub fn from_range(id: FieldId, name: Arc<str>, range: ByteRange) -> Self {
        // A checked, nonempty ByteRange guarantees start < end <= usize::MAX.
        Self {
            id,
            name,
            start_one_based: range.start() + 1,
            end_one_based_inclusive: range.end_exclusive(),
        }
    }

    pub fn validate(&self) -> Result<(), InvalidDiagnostic> {
        if self.id.0.is_empty() {
            return Err(InvalidDiagnostic::EmptyFieldId);
        }
        if self.name.is_empty() {
            return Err(InvalidDiagnostic::EmptyFieldName);
        }
        if self.start_one_based == 0 || self.end_one_based_inclusive < self.start_one_based {
            return Err(InvalidDiagnostic::InvalidFieldPositions);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticContextItem {
    pub trace_id: TraceId,
    pub field_id: FieldId,
    pub value: DiagnosticValue,
}

/// A normative violation. Technical execution failures use separate error types.
/// Public payloads follow DET §52; call `validate` before emitting manual values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub code: ValidationCode,
    pub flow_id: FlowId,
    pub schema_version: SchemaVersion,
    pub trace_id: TraceId,
    pub line_number: u64,
    pub record_identity: RecordIdentity,
    pub field: Option<FieldDiagnostic>,
    pub actual: Option<DiagnosticValue>,
    pub expected: Option<String>,
    pub context: Vec<DiagnosticContextItem>,
    pub message: String,
}

/// Malformed diagnostic metadata, not a normative ValidationCode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidDiagnostic {
    InvalidCode(InvalidValidationCode),
    EmptyFlowId,
    EmptySchemaVersion,
    EmptyTraceId,
    ZeroLineNumber,
    EmptyFieldId,
    EmptyFieldName,
    InvalidFieldPositions,
    EmptyContextTraceId,
    EmptyContextFieldId,
}
impl fmt::Display for InvalidDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidCode(_) => "invalid diagnostic validation code",
            Self::EmptyFlowId => "diagnostic flow ID is empty",
            Self::EmptySchemaVersion => "diagnostic schema version is empty",
            Self::EmptyTraceId => "diagnostic trace ID is empty",
            Self::ZeroLineNumber => "diagnostic line number must be one-based",
            Self::EmptyFieldId => "diagnostic field ID is empty",
            Self::EmptyFieldName => "diagnostic field name is empty",
            Self::InvalidFieldPositions => {
                "diagnostic field positions must be one-based and inclusive"
            }
            Self::EmptyContextTraceId => "diagnostic context trace ID is empty",
            Self::EmptyContextFieldId => "diagnostic context field ID is empty",
        })
    }
}
impl Error for InvalidDiagnostic {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidCode(source) => Some(source),
            _ => None,
        }
    }
}
impl ValidationError {
    /// Checks DET §164 metadata without guessing rule scope from a code or text.
    /// A field-level producer supplies `Some(FieldDiagnostic)` as a complete unit.
    pub fn validate(&self) -> Result<(), InvalidDiagnostic> {
        self.code
            .as_symbol()
            .map_err(InvalidDiagnostic::InvalidCode)?;
        if self.flow_id.0.is_empty() {
            return Err(InvalidDiagnostic::EmptyFlowId);
        }
        if self.schema_version.0.is_empty() {
            return Err(InvalidDiagnostic::EmptySchemaVersion);
        }
        if self.trace_id.0.is_empty() {
            return Err(InvalidDiagnostic::EmptyTraceId);
        }
        if self.line_number == 0 {
            return Err(InvalidDiagnostic::ZeroLineNumber);
        }
        if let Some(field) = &self.field {
            field.validate()?;
        }
        for item in &self.context {
            if item.trace_id.0.is_empty() {
                return Err(InvalidDiagnostic::EmptyContextTraceId);
            }
            if item.field_id.0.is_empty() {
                return Err(InvalidDiagnostic::EmptyContextFieldId);
            }
        }
        Ok(())
    }
}

/// The symbol is malformed or aliases a reserved core code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidValidationCode;

impl fmt::Display for InvalidValidationCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid or reserved validation code symbol")
    }
}

impl Error for InvalidValidationCode {}

fn is_regulatory_symbol(s: &str) -> bool {
    let Some(body) = s.strip_prefix("E_") else {
        return false;
    };
    body.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
        && body
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
        && s != "E_SCHEMA_ERROR"
}

macro_rules! validation_codes {
    ($($variant:ident => $symbol:literal),+ $(,)?) => {
        /// Validation failures, distinct from schema/I/O execution errors.
        ///
        /// Regulatory codes must be registered by the compiled schema. Parsing
        /// a symbol here verifies syntax, not its normative registration.
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum ValidationCode {
            $($variant,)+
            RegulatoryRule(Arc<str>),
        }

        impl ValidationCode {
            /// Resolves the stable public symbol; rejects invalid regulatory payloads.
            pub fn as_symbol(&self) -> Result<&str, InvalidValidationCode> {
                match self {
                    $(Self::$variant => Ok($symbol),)+
                    Self::RegulatoryRule(s) => {
                        if !is_regulatory_symbol(s) || matches!(s.as_ref(), $($symbol)|+) {
                            Err(InvalidValidationCode)
                        } else {
                            Ok(s)
                        }
                    }
                }
            }
        }

        impl FromStr for ValidationCode {
            type Err = InvalidValidationCode;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($symbol => Ok(Self::$variant),)+
                    _ if is_regulatory_symbol(s) => Ok(Self::RegulatoryRule(Arc::from(s))),
                    _ => Err(InvalidValidationCode),
                }
            }
        }
    };
}

// Single executable mapping. Order follows DET §51, not lexical string order.
validation_codes! {
    RecordLength => "E_RECORD_LENGTH",
    RequiredField => "E_REQUIRED_FIELD",
    ConditionalRequired => "E_CONDITIONAL_REQUIRED",
    ForbiddenField => "E_FORBIDDEN_FIELD",
    Alignment => "E_ALIGNMENT",
    InvalidType => "E_INVALID_TYPE",
    InvalidDomain => "E_INVALID_DOMAIN",
    OutOfRange => "E_OUT_OF_RANGE",
    InvalidDate => "E_INVALID_DATE",
    CrossField => "E_CROSS_FIELD",
    ReferenceData => "E_REFERENCE_DATA",
    CrossRecord => "E_CROSS_RECORD",
    RuleConflict => "E_RULE_CONFLICT",
    RelatedRecordMissing => "E_RELATED_RECORD_MISSING",
    DuplicateRecordKey => "E_DUPLICATE_RECORD_KEY",
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashSet};

    // Expectations transcribed from ARCH §26 / DET §51; not generated by the macro.
    fn expected() -> [(ValidationCode, &'static str); 15] {
        use ValidationCode::*;
        [
            (RecordLength, "E_RECORD_LENGTH"),
            (RequiredField, "E_REQUIRED_FIELD"),
            (ConditionalRequired, "E_CONDITIONAL_REQUIRED"),
            (ForbiddenField, "E_FORBIDDEN_FIELD"),
            (Alignment, "E_ALIGNMENT"),
            (InvalidType, "E_INVALID_TYPE"),
            (InvalidDomain, "E_INVALID_DOMAIN"),
            (OutOfRange, "E_OUT_OF_RANGE"),
            (InvalidDate, "E_INVALID_DATE"),
            (CrossField, "E_CROSS_FIELD"),
            (ReferenceData, "E_REFERENCE_DATA"),
            (CrossRecord, "E_CROSS_RECORD"),
            (RuleConflict, "E_RULE_CONFLICT"),
            (RelatedRecordMissing, "E_RELATED_RECORD_MISSING"),
            (DuplicateRecordKey, "E_DUPLICATE_RECORD_KEY"),
        ]
    }

    #[test]
    fn core_mapping_matches_source_and_roundtrips_without_collisions() {
        let mut symbols = HashSet::new();
        for (code, symbol) in expected() {
            assert_eq!(code.as_symbol(), Ok(symbol));
            assert_eq!(symbol.parse(), Ok(code));
            assert!(symbols.insert(symbol));
        }
    }

    #[test]
    fn map_order_is_stable_independent_of_insertion_order() {
        let mut map = BTreeMap::new();
        for (code, _) in expected().into_iter().rev() {
            map.insert(code, 1);
        }
        assert_eq!(
            map.into_keys().collect::<Vec<_>>(),
            expected().map(|(code, _)| code)
        );
        assert!(
            ValidationCode::DuplicateRecordKey
                < ValidationCode::RegulatoryRule("E_EXAMPLE_A".into())
        );
        assert!(
            ValidationCode::RegulatoryRule("E_EXAMPLE_A".into())
                < ValidationCode::RegulatoryRule("E_EXAMPLE_B".into())
        );
    }

    #[test]
    fn regulatory_codes_roundtrip_but_do_not_claim_registration() {
        let code = ValidationCode::RegulatoryRule("E_EXAMPLE_RULE_1".into());
        assert_eq!(code.as_symbol(), Ok("E_EXAMPLE_RULE_1"));
        assert_eq!("E_EXAMPLE_RULE_1".parse(), Ok(code));
        for (_, symbol) in expected() {
            assert_eq!(
                ValidationCode::RegulatoryRule(symbol.into()).as_symbol(),
                Err(InvalidValidationCode)
            );
        }
    }

    #[test]
    fn rejects_legacy_malformed_and_schema_execution_symbols() {
        for symbol in [
            "",
            "E001",
            "E001_REQUIRED",
            "E_",
            "E_1",
            "E_bad",
            "E_A-B",
            " E_REQUIRED_FIELD",
            "E_X\n",
            "E_À",
            "E_SCHEMA_ERROR",
        ] {
            assert_eq!(symbol.parse::<ValidationCode>(), Err(InvalidValidationCode));
            assert_eq!(
                ValidationCode::RegulatoryRule(symbol.into()).as_symbol(),
                Err(InvalidValidationCode)
            );
        }
    }
}
