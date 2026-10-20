//! Lexical oracles from the approved format-1 authoring contract.
use lintra::{
    AlignmentRule, CompiledFlowSchema, ContentBoundaryPolicy, DatatypeError,
    DatatypeValidationResult as Outcome, EncodingSpec, ErrorFactory, FieldSlice, FieldViolation,
    RawFlowSchema, RawResolvedEncoding, RecordIdentity, SchemaCompiler, ValidationCode,
    logical_value, validate_datatype,
};
use serde_json::{Value, json};

/// Reuse only the committed synthetic raw shape, changing an in-memory copy.
/// A wide field permits logical samples of different lengths without testing
/// record width here. The real compiler remains the only compiled constructor.
fn schema(kind: &str, lexical: Value) -> CompiledFlowSchema {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    value["traces"][0]["record_length"] = json!(256);
    let physical = &mut value["traces"][0]["fields"][0]["physical"];
    physical["type"] = json!(kind);
    physical["length"] = json!(256);
    physical["lexical"] = lexical;
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw).unwrap()
}

/// Unwrap the controlled invalid outcome, refusing to confuse a technical
/// configuration error or a successful value with a normative failure.
fn violation(result: Result<Outcome, DatatypeError>) -> FieldViolation {
    let Outcome::Invalid(violation) = result.unwrap() else {
        panic!("synthetic value must be lexically invalid");
    };
    assert_eq!(violation.code, ValidationCode::InvalidType);
    assert!(violation.reason.is_empty());
    assert!(violation.diagnostic_context.is_empty());
    violation
}

/// Numeric uses nonempty ASCII digits, with no parsing overflow or hidden
/// allowance for signs, separators, Unicode digits, whitespace or empty values.
#[test]
fn numeric_is_nonempty_ascii_digits_without_integer_conversion() {
    let schema = schema("numeric", json!({"kind":"encoded_text"}));
    let field = &schema.traces()[0].fields()[0];
    for raw in [b"0".as_slice(), b"0001", b"1000", &[b'9'; 200]] {
        assert_eq!(
            validate_datatype(field, FieldSlice { raw }, &EncodingSpec::Utf8),
            Ok(Outcome::Valid)
        );
    }
    for raw in [
        b"".as_slice(),
        b"+1",
        b"-1",
        b"1.0",
        b"1,0",
        b" 12",
        b"12 ",
        b"\0",
        "١".as_bytes(),
        "１".as_bytes(),
    ] {
        violation(validate_datatype(
            field,
            FieldSlice { raw },
            &EncodingSpec::Utf8,
        ));
    }
}

/// AN means encoded text, not A-Z/0-9. Leading spaces, punctuation, accents,
/// emoji and empty logical text are legal unless another explicit rule forbids.
#[test]
fn alphanumeric_does_not_invent_a_character_domain() {
    let schema = schema("alphanumeric", json!({"kind":"encoded_text"}));
    let field = &schema.traces()[0].fields()[0];
    for raw in ["", " ROSSI", "D'ANGELO", "é", "🙂", "a-z / 12"] {
        assert_eq!(
            validate_datatype(
                field,
                FieldSlice {
                    raw: raw.as_bytes()
                },
                &EncodingSpec::Utf8
            ),
            Ok(Outcome::Valid)
        );
    }
}

/// UTF-8 validity is checked without replacement decoding: truncated,
/// overlong, surrogate and lone continuation encodings are invalid data.
#[test]
fn malformed_utf8_is_invalid_without_lossy_replacement() {
    let schema = schema("alphanumeric", json!({"kind":"encoded_text"}));
    let field = &schema.traces()[0].fields()[0];
    for raw in [
        b"\xff".as_slice(),
        b"\xc3",
        b"\xc0\xaf",
        b"\xed\xa0\x80",
        b"\x80",
    ] {
        let before = raw.to_vec();
        violation(validate_datatype(
            field,
            FieldSlice { raw },
            &EncodingSpec::Utf8,
        ));
        assert_eq!(raw, before);
    }
}

/// ByteSet applies exactly its explicit byte class. Lowercase metadata does
/// not automatically activate the separate no_lowercase semantic constraint.
#[test]
fn explicit_byte_class_is_checked_without_case_folding() {
    let schema = schema(
        "alphanumeric",
        json!({"kind":"byte_set","allowed_bytes":[65,97],"lowercase_bytes":[97]}),
    );
    let field = &schema.traces()[0].fields()[0];
    assert_eq!(
        validate_datatype(field, FieldSlice { raw: b"aA" }, &EncodingSpec::Utf8),
        Ok(Outcome::Valid)
    );
    violation(validate_datatype(
        field,
        FieldSlice { raw: b"B" },
        &EncodingSpec::Utf8,
    ));
}

/// An allowed byte does not excuse an invalid encoded sequence; UTF-8 still
/// has to be valid even when the individual raw byte belongs to the class.
#[test]
fn byte_class_never_bypasses_encoding_validation() {
    let schema = schema(
        "alphanumeric",
        json!({"kind":"byte_set","allowed_bytes":[233],"lowercase_bytes":[]}),
    );
    violation(validate_datatype(
        &schema.traces()[0].fields()[0],
        FieldSlice { raw: &[233] },
        &EncodingSpec::Utf8,
    ));
}

/// Numeric still obeys a narrower declared class, rather than replacing it
/// with the broader built-in digit set. Domain checks remain separate.
#[test]
fn numeric_also_obeys_the_explicit_byte_class() {
    let schema = schema(
        "numeric",
        json!({"kind":"byte_set","allowed_bytes":[49,50],"lowercase_bytes":[]}),
    );
    let field = &schema.traces()[0].fields()[0];
    assert_eq!(
        validate_datatype(field, FieldSlice { raw: b"12" }, &EncodingSpec::Utf8),
        Ok(Outcome::Valid)
    );
    violation(validate_datatype(
        field,
        FieldSlice { raw: b"0" },
        &EncodingSpec::Utf8,
    ));
    violation(validate_datatype(
        field,
        FieldSlice { raw: b"" },
        &EncodingSpec::Utf8,
    ));
}

/// CP1252 has five undefined bytes per the existing Unicode mapping; ISO-8859-1
/// maps every byte. These test explicit validator profiles, not new compiler
/// binding support (the current incremental compiler still accepts only UTF-8).
#[test]
fn single_byte_profiles_use_the_declared_mapping() {
    let schema = schema("alphanumeric", json!({"kind":"encoded_text"}));
    let field = &schema.traces()[0].fields()[0];
    for byte in 0..=255u8 {
        let raw = [byte];
        let result =
            validate_datatype(field, FieldSlice { raw: &raw }, &EncodingSpec::Windows1252).unwrap();
        if [0x81, 0x8d, 0x8f, 0x90, 0x9d].contains(&byte) {
            assert!(matches!(result, Outcome::Invalid(_)));
        } else {
            assert_eq!(result, Outcome::Valid);
        }
        assert_eq!(
            validate_datatype(field, FieldSlice { raw: &raw }, &EncodingSpec::Iso8859_1),
            Ok(Outcome::Valid)
        );
    }
}

/// A profile unknown to the engine is a technical error even on malformed
/// input. It must not be turned into an InvalidType violation of the record.
#[test]
fn unsupported_encoding_preserves_a_technical_boundary() {
    let schema = schema("alphanumeric", json!({"kind":"encoded_text"}));
    let profile = EncodingSpec::Other("synthetic-unknown".into());
    assert_eq!(
        validate_datatype(
            &schema.traces()[0].fields()[0],
            FieldSlice { raw: b"\xff" },
            &profile
        ),
        Err(DatatypeError::UnsupportedEncoding(profile))
    );
}

/// Presence/extraction/datatype stay ordered: all-zero stripping yields empty
/// Numeric (invalid), whereas PreserveWidth keeps real zero digits (valid).
#[test]
fn logical_extraction_is_explicit_and_never_invents_zero() {
    let schema = schema("numeric", json!({"kind":"encoded_text"}));
    let field = &schema.traces()[0].fields()[0];
    let rule = AlignmentRule::LeftPad {
        fill: b'0',
        content_policy: ContentBoundaryPolicy::StripLeadingFill,
    };
    let logical = logical_value(&rule, FieldSlice { raw: b"001000" }).unwrap();
    assert_eq!(logical.raw, b"1000");
    assert_eq!(
        validate_datatype(field, logical, &EncodingSpec::Utf8),
        Ok(Outcome::Valid)
    );
    violation(validate_datatype(
        field,
        logical_value(&rule, FieldSlice { raw: b"000000" }).unwrap(),
        &EncodingSpec::Utf8,
    ));
    let preserve = AlignmentRule::LeftPad {
        fill: b'0',
        content_policy: ContentBoundaryPolicy::PreserveWidth,
    };
    assert_eq!(
        validate_datatype(
            field,
            logical_value(&preserve, FieldSlice { raw: b"000000" }).unwrap(),
            &EncodingSpec::Utf8
        ),
        Ok(Outcome::Valid)
    );
    // Numeric validation does not trim spaces automatically.
    violation(validate_datatype(
        field,
        FieldSlice { raw: b" 12 " },
        &EncodingSpec::Utf8,
    ));
}

/// Reuse the actual diagnostic enrichment: controlled Invalid carries only
/// the field violation; full identity/positions/rendering belong to the factory.
#[test]
fn invalid_type_integrates_with_error_factory() {
    let schema = schema("numeric", json!({"kind":"encoded_text"}));
    let trace = &schema.traces()[0];
    let field = &trace.fields()[0];
    let failure = violation(validate_datatype(
        field,
        FieldSlice { raw: b"+12" },
        schema.encoding(),
    ));
    let error = ErrorFactory {
        flow_id: &schema.key().flow_id,
        schema_version: &schema.key().version,
        trace_id: trace.id(),
        encoding: schema.encoding(),
    }
    .field_error(3, &RecordIdentity::default(), field, b"+12", failure);
    assert_eq!(error.validate(), Ok(()));
    assert_eq!(error.code, ValidationCode::InvalidType);
    assert_eq!(error.actual.unwrap().display, "+12");
    assert_eq!(error.line_number, 3);
}

/// Alphabetic is deliberately not part of approved format 1. No repertoire
/// is invented or silently replaced with Alphanumeric to satisfy a legacy task.
#[test]
fn alphabetic_is_still_rejected_by_the_format_parser() {
    let mut raw: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    raw["traces"][0]["fields"][0]["physical"]["type"] = json!("alphabetic");
    assert!(serde_json::from_value::<RawFlowSchema>(raw).is_err());
}
