//! Explicit byte domains compiled from synthetic format-1 schemas.
use lintra::{
    CompiledConstraint, CompiledFlowSchema, ErrorFactory, FieldSlice, RawFlowSchema,
    RawResolvedEncoding, RecordIdentity, SchemaCompiler, SchemaIssueCode, ValidationCode,
    ValueTarget, logical_value, validate_allowed_values,
};
use serde_json::{Value, json};

/// Build a four-byte raw field from the existing synthetic shape. Domains and
/// target are explicit; strings are never padded by this helper or the compiler.
fn raw(alignment: Value, target: &str, values: &[&str]) -> RawFlowSchema {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    value["traces"][0]["record_length"] = json!(4);
    let field = &mut value["traces"][0]["fields"][0];
    field["physical"]["length"] = json!(4);
    field["physical"]["alignment"] = alignment;
    field["constraints"] = json!([{"kind":"allowed_values","target":target,"values":values}]);
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    raw
}

/// Read the actual lowered domain/target, refusing a dropped constraint or a
/// different plan. Returning a borrow also avoids copying domains in tests.
fn plan(schema: &CompiledFlowSchema) -> (&[Vec<u8>], ValueTarget) {
    let CompiledConstraint::AllowedValues { values, target } =
        &schema.traces()[0].fields()[0].constraints()[0]
    else {
        panic!("compiled byte-domain plan required");
    };
    (values, *target)
}

/// Physical padding, zeros and case belong to the value: same apparent code
/// with different bytes must not match. The field's explicit target is Raw.
#[test]
fn raw_membership_never_trims_pads_or_parses() {
    let schema = SchemaCompiler
        .compile(raw(json!({"kind":"exact"}), "raw", &["01  ", "AB  "]))
        .unwrap();
    let (values, target) = plan(&schema);
    assert_eq!(target, ValueTarget::Raw);
    for bytes in [b"01  ".as_slice(), b"AB  "] {
        validate_allowed_values(values, FieldSlice { raw: bytes }).unwrap();
    }
    for bytes in [b"01".as_slice(), b" 01 ", b"1   ", b"ab  ", b"01\0 "] {
        let error = validate_allowed_values(values, FieldSlice { raw: bytes }).unwrap_err();
        assert_eq!(error.code, ValidationCode::InvalidDomain);
        assert!(error.reason.is_empty());
        assert!(error.diagnostic_context.is_empty());
    }
}

/// Logical matching happens only when the caller selects and extracts that
/// explicit target. The membership helper never performs extraction itself.
#[test]
fn logical_target_uses_only_explicit_extraction() {
    let schema = SchemaCompiler
        .compile(raw(
            json!({"kind":"right_pad","fill_byte":32,"content_policy":"strip_trailing_fill"}),
            "logical",
            &["01"],
        ))
        .unwrap();
    let (values, target) = plan(&schema);
    assert_eq!(target, ValueTarget::Logical);
    let source = FieldSlice { raw: b"01  " };
    assert!(validate_allowed_values(values, source).is_err());
    let logical = logical_value(&schema.traces()[0].fields()[0].alignment(), source).unwrap();
    validate_allowed_values(values, logical).unwrap();
    assert_eq!(source.raw, b"01  ");
}

/// PreserveWidth keeps significant zeros and therefore demands a full-width
/// logical literal, not a shortened value padded behind the author's back.
#[test]
fn preserve_width_keeps_zero_codes_and_requires_full_width_literals() {
    let alignment = json!({"kind":"left_pad","fill_byte":48,"content_policy":"preserve_width"});
    let schema = SchemaCompiler
        .compile(raw(alignment.clone(), "logical", &["0010"]))
        .unwrap();
    validate_allowed_values(plan(&schema).0, FieldSlice { raw: b"0010" }).unwrap();
    assert!(
        SchemaCompiler
            .compile(raw(alignment, "logical", &["10"]))
            .unwrap_err()
            .issues
            .iter()
            .any(|i| i.code == SchemaIssueCode::InvalidTypeParameters)
    );
}

/// Literal width is encoded bytes: é plus two spaces is four UTF-8 bytes;
/// adding a third space is five bytes despite only four Unicode characters.
#[test]
fn literal_width_is_bytes_not_unicode_characters() {
    let schema = SchemaCompiler
        .compile(raw(json!({"kind":"exact"}), "raw", &["é  "]))
        .unwrap();
    validate_allowed_values(
        plan(&schema).0,
        FieldSlice {
            raw: "é  ".as_bytes(),
        },
    )
    .unwrap();
    let error = SchemaCompiler
        .compile(raw(json!({"kind":"exact"}), "raw", &["é   "]))
        .unwrap_err();
    assert!(
        error
            .issues
            .iter()
            .any(|i| i.code == SchemaIssueCode::InvalidTypeParameters
                && i.trace_id.as_ref().is_some_and(|t| t.0 == "T1")
                && i.field_id.as_ref().is_some_and(|f| f.0 == "F1"))
    );
}

/// Empty or duplicate domains are malformed schemas, not runtime violations.
/// Raw/Exact literals must have exactly the declared field width.
#[test]
fn malformed_domains_and_incompatible_widths_are_compile_errors() {
    for values in [vec![], vec!["AB  ", "AB  "]] {
        let error = SchemaCompiler
            .compile(raw(json!({"kind":"exact"}), "raw", &values))
            .unwrap_err();
        assert!(
            error
                .issues
                .iter()
                .any(|i| i.code == SchemaIssueCode::InvalidSchemaValue)
        );
    }
    for (target, values) in [
        ("raw", vec!["A"]),
        ("raw", vec!["ABCDE"]),
        ("logical", vec!["A"]),
    ] {
        let error = SchemaCompiler
            .compile(raw(json!({"kind":"exact"}), target, &values))
            .unwrap_err();
        assert!(
            error
                .issues
                .iter()
                .any(|i| i.code == SchemaIssueCode::InvalidTypeParameters)
        );
    }
}

/// A logical literal must already be reachable after the declared extraction.
/// Wrong stripping-edge fill or excessive width is a plan error; opposite-edge
/// and internal fill remain content, per the ratified T-P6-03 decision.
#[test]
fn logical_literals_are_canonical_without_implicit_rewriting() {
    let right = json!({"kind":"right_pad","fill_byte":32,"content_policy":"strip_trailing_fill"});
    for values in [vec!["AB "], vec!["ABCDE"]] {
        assert!(
            SchemaCompiler
                .compile(raw(right.clone(), "logical", &values))
                .unwrap_err()
                .issues
                .iter()
                .any(|i| i.code == SchemaIssueCode::InvalidTypeParameters)
        );
    }
    let schema = SchemaCompiler
        .compile(raw(right, "logical", &[" AB", "A B", ""]))
        .unwrap();
    assert_eq!(plan(&schema).0, [b" AB".to_vec(), b"A B".to_vec(), vec![]]);
    let left = json!({"kind":"left_pad","fill_byte":48,"content_policy":"strip_leading_fill"});
    assert!(
        SchemaCompiler
            .compile(raw(left.clone(), "logical", &["010"]))
            .is_err()
    );
    let schema = SchemaCompiler
        .compile(raw(left, "logical", &["10"]))
        .unwrap();
    assert_eq!(plan(&schema).0, [b"10".to_vec()]);
}

/// Supported domains must not accidentally open other AST nodes or erase them.
/// Adding a range to an otherwise valid schema keeps a blocking typed issue.
#[test]
fn unsupported_constraints_are_still_explicitly_rejected() {
    let mut raw = raw(json!({"kind":"exact"}), "raw", &["AB  "]);
    let mut value = serde_json::to_value(&raw).unwrap();
    value["traces"][0]["fields"][0]["constraints"]
        .as_array_mut()
        .unwrap()
        .push(json!({"kind":"numeric_range","min":1,"max":9}));
    raw = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    assert!(
        SchemaCompiler
            .compile(raw)
            .unwrap_err()
            .issues
            .iter()
            .any(|i| i.code == SchemaIssueCode::UnsupportedConstruct)
    );
}

/// Compiling may validate with a set but must preserve the declared vector
/// order and literal bytes, rather than sorting or normalizing Unicode.
#[test]
fn compiled_domain_keeps_declaration_order_and_bytes() {
    let schema = SchemaCompiler
        .compile(raw(
            json!({"kind":"exact"}),
            "raw",
            &["ZZ  ", "AA  ", "é  "],
        ))
        .unwrap();
    assert_eq!(
        plan(&schema).0,
        [
            b"ZZ  ".to_vec(),
            b"AA  ".to_vec(),
            "é  ".as_bytes().to_vec()
        ]
    );
}

/// Membership is meaningful on arbitrary bytes, not strings. NUL/non-UTF-8
/// values do not trigger decoding, trimming, replacement or case changes.
#[test]
fn byte_membership_is_not_text_normalization() {
    let values = vec![vec![0, 0xff, b' '], "é".as_bytes().to_vec()];
    validate_allowed_values(
        &values,
        FieldSlice {
            raw: &[0, 0xff, b' '],
        },
    )
    .unwrap();
    validate_allowed_values(
        &values,
        FieldSlice {
            raw: "é".as_bytes(),
        },
    )
    .unwrap();
    assert!(validate_allowed_values(&values, FieldSlice { raw: &[0, 0xff] }).is_err());
    assert!(
        validate_allowed_values(
            &values,
            FieldSlice {
                raw: "e\u{301}".as_bytes()
            }
        )
        .is_err()
    );
}

/// The primary violation feeds the existing ErrorFactory. It retains the
/// original physical raw diagnostic rather than copying a domain into reports.
#[test]
fn domain_violation_integrates_with_error_factory() {
    let schema = SchemaCompiler
        .compile(raw(json!({"kind":"exact"}), "raw", &["01  "]))
        .unwrap();
    let trace = &schema.traces()[0];
    let field = &trace.fields()[0];
    let failure =
        validate_allowed_values(plan(&schema).0, FieldSlice { raw: b"03  " }).unwrap_err();
    let error = ErrorFactory {
        flow_id: &schema.key().flow_id,
        schema_version: &schema.key().version,
        trace_id: trace.id(),
        encoding: schema.encoding(),
    }
    .field_error(2, &RecordIdentity::default(), field, b"03  ", failure);
    assert_eq!(error.validate(), Ok(()));
    assert_eq!(error.code, ValidationCode::InvalidDomain);
    assert_eq!(error.actual.unwrap().display, "03  ");
    assert_eq!(error.line_number, 2);
}
