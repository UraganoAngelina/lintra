//! Synthetic identity bindings exercise the actual compiler and source slices.
use lintra::{
    CompiledFlowSchema, EncodingSpec, IdentityExtractionError, IdentityExtractor, RawFlowSchema,
    RawResolvedEncoding, RecordIdentity, RecordView, SchemaCompiler, SchemaIssueCode,
};
use serde_json::{Value, json};

/// Three fields with distinct missing policies ensure that absent identity is
/// decided by configuration, not by a global blank/zero/field-name convention.
fn raw(bindings: Value) -> RawFlowSchema {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    value["traces"][0]["record_length"] = json!(9);
    let base = value["traces"][0]["fields"][0].clone();
    let fields: Vec<Value> = [
        ("X", 1, json!({"kind":"spaces"})),
        ("Y", 4, json!({"kind":"exact","raw":"000"})),
        ("Z", 7, json!({"kind":"never_missing"})),
    ]
    .into_iter()
    .map(|(id, start, missing)| {
        let mut field = base.clone();
        field["id"] = json!(id);
        field["physical"]["start"] = json!(start);
        field["physical"]["length"] = json!(3);
        field["missing_value"] = missing;
        field
    })
    .collect();
    value["traces"][0]["fields"] = json!(fields);
    value["traces"][0]["record_key"]["fields"] = json!(["X"]);
    value["traces"][0]["identity"] = bindings;
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    raw
}

/// Build through public compilation, so bindings cannot be injected at runtime.
fn schema(bindings: Value) -> CompiledFlowSchema {
    SchemaCompiler.compile(raw(bindings)).unwrap()
}

/// Feed original source bytes with the compiled flow's explicit encoding.
fn extract(schema: &CompiledFlowSchema, bytes: &[u8]) -> RecordIdentity {
    IdentityExtractor
        .extract(
            &RecordView::new(bytes),
            &schema.traces()[0],
            schema.encoding(),
        )
        .unwrap()
}

/// Every supported destination is populated without a hardcoded source offset.
/// Technical keys remain BTreeMap-ordered, independently of declaration order.
#[test]
fn complete_identity_follows_configured_indexes_and_keeps_raw_padding() {
    let schema = schema(json!([
        {"target":"episode_id","field":"Z"},
        {"target":"fiscal_code","field":"X"},
        {"target":"sioc_record_id","field":"Y"},
        {"target":"technical","technical_name":"zeta","field":"Z"},
        {"target":"technical","technical_name":"alpha","field":"X"}
    ]));
    let id = extract(&schema, b" A 007END");
    assert_eq!(id.fiscal_code.as_deref(), Some(" A "));
    assert_eq!(id.sioc_record_id.as_deref(), Some("007"));
    assert_eq!(id.episode_id.as_deref(), Some("END"));
    assert_eq!(
        id.technical_ids
            .keys()
            .map(AsRef::as_ref)
            .collect::<Vec<&str>>(),
        ["alpha", "zeta"]
    );
    assert_eq!(
        schema.traces()[0]
            .identity_plan()
            .bindings
            .iter()
            .map(|b| b.field_index)
            .collect::<Vec<_>>(),
        [2, 0, 1, 2, 0]
    );
}

/// An empty plan does not infer identity from field names or record key fields.
#[test]
fn absent_plan_returns_empty_identity() {
    assert_eq!(
        extract(&schema(json!([])), b"ABC123XYZ"),
        RecordIdentity::default()
    );
}

/// Missing policies operate on raw bytes before any stripping or validation.
/// NeverMissing means three blanks are still an available diagnostic value.
#[test]
fn partial_identity_respects_each_explicit_missing_policy() {
    let schema = schema(json!([
        {"target":"fiscal_code","field":"X"},
        {"target":"episode_id","field":"Y"},
        {"target":"technical","technical_name":"available","field":"Z"}
    ]));
    let id = extract(&schema, b"   000   ");
    assert!(id.fiscal_code.is_none() && id.episode_id.is_none());
    assert_eq!(id.technical_ids["available"].as_ref(), "   ");
    let id = extract(&schema, b"000   XYZ");
    assert_eq!(id.fiscal_code.as_deref(), Some("000"));
    assert_eq!(id.episode_id.as_deref(), Some("   "));
}

/// Wrong width is rejected before extracting even when the plan is empty.
#[test]
fn short_and_long_records_fail_the_length_precondition() {
    for bindings in [json!([]), json!([{"target":"fiscal_code","field":"X"}])] {
        let schema = schema(bindings);
        for bytes in [b"short".as_slice(), b"longer-than-nine"] {
            assert!(
                matches!(IdentityExtractor.extract(&RecordView::new(bytes), &schema.traces()[0], schema.encoding()),
                Err(IdentityExtractionError::RecordLength { expected:9, actual }) if actual == bytes.len())
            );
        }
    }
}

/// Invalid content is still diagnostic identity: no datatype/domain validation
/// or implicit fallback decoding is performed by this extraction stage.
#[test]
fn rendering_uses_explicit_encoding_without_normative_validation() {
    let schema = schema(json!([{"target":"fiscal_code","field":"X"}]));
    let bytes = b"\xe9\\\n123XYZ";
    let view = RecordView::new(bytes);
    for (encoding, expected) in [
        (EncodingSpec::Iso8859_1, "é\\\\\\n"),
        (EncodingSpec::Utf8, "�\\\\\\n"),
        (EncodingSpec::Other("unknown".into()), "\\xe9\\\\\\n"),
    ] {
        let id = IdentityExtractor
            .extract(&view, &schema.traces()[0], &encoding)
            .unwrap();
        assert_eq!(id.fiscal_code.as_deref(), Some(expected));
    }
    assert_eq!(view.raw(), bytes);
}

/// Permuting bindings changes destinations without changing the source layout.
#[test]
fn source_field_binding_controls_identity_not_target_name() {
    let schema = schema(json!([
        {"target":"fiscal_code","field":"Z"},
        {"target":"episode_id","field":"X"}
    ]));
    let id = extract(&schema, b"ABC123XYZ");
    assert_eq!(id.fiscal_code.as_deref(), Some("XYZ"));
    assert_eq!(id.episode_id.as_deref(), Some("ABC"));
}

/// Public compiler rejects missing refs, malformed technical targets and
/// colliding destinations. No last-writer-wins identity can escape compilation.
#[test]
fn compiler_rejects_invalid_or_colliding_identity_bindings() {
    for (bindings, expected) in [
        (
            json!([{"target":"fiscal_code","field":"absent"}]),
            SchemaIssueCode::MissingFieldReference,
        ),
        (
            json!([{"target":"technical","field":"X"}]),
            SchemaIssueCode::InvalidSchemaValue,
        ),
        (
            json!([{"target":"technical","field":"X","technical_name":""}]),
            SchemaIssueCode::InvalidSchemaValue,
        ),
        (
            json!([{"target":"episode_id","field":"X","technical_name":"bad"}]),
            SchemaIssueCode::InvalidSchemaValue,
        ),
        (
            json!([{"target":"fiscal_code","field":"X"},{"target":"fiscal_code","field":"Y"}]),
            SchemaIssueCode::InvalidSchemaValue,
        ),
        (
            json!([{"target":"technical","field":"X","technical_name":"same"},{"target":"technical","field":"Y","technical_name":"same"}]),
            SchemaIssueCode::InvalidSchemaValue,
        ),
    ] {
        let error = SchemaCompiler.compile(raw(bindings)).unwrap_err();
        assert!(error.issues.iter().any(|issue| issue.code == expected));
    }
}
