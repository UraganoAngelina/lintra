//! Synthetic compiled regex plans exercised through the actual field pipeline.
use lintra::{
    CompiledFlowSchema, ErrorFactory, FieldPipeline, FieldPipelineResult, FieldSlice,
    RawFlowSchema, RawResolvedEncoding, RecordIdentity, SchemaCompiler, SchemaIssueCode,
    ScopeSummary, ValidationCode, ValidationSummaryBuilder, VecSink,
};
use serde_json::{Value, json};

// Keep the schema explicit: the only variable semantics are declared constraints,
// datatype and extraction. No helper compiles a separate runtime regex.
fn raw(constraints: Value, logical: bool, numeric: bool) -> RawFlowSchema {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    let field = &mut value["traces"][0]["fields"][0];
    field["constraints"] = constraints;
    if logical {
        field["physical"]["alignment"] =
            json!({"kind":"right_pad","fill_byte":32,"content_policy":"strip_trailing_fill"});
    }
    if numeric {
        field["physical"]["type"] = json!("numeric");
        field["physical"]["lexical"] = json!({"kind":"encoded_text"});
    }
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    raw
}
fn constraint(pattern: &str, mode: &str, target: &str) -> Value {
    json!({"kind":"regex","pattern":pattern,"match":mode,"target":target})
}
fn observe(
    schema: &CompiledFlowSchema,
    bytes: &[u8],
) -> (FieldPipelineResult, Vec<ValidationCode>) {
    let trace = &schema.traces()[0];
    let factory = ErrorFactory {
        flow_id: &schema.key().flow_id,
        schema_version: &schema.key().version,
        trace_id: trace.id(),
        encoding: schema.encoding(),
    };
    let mut sink = VecSink::default();
    let mut summary = ValidationSummaryBuilder::new(
        schema.key().clone(),
        ScopeSummary::Trace {
            trace_id: trace.id().clone(),
        },
    );
    let result = FieldPipeline::new(factory, &mut sink, &mut summary)
        .run(
            &trace.fields()[0],
            FieldSlice { raw: bytes },
            1,
            &RecordIdentity::default(),
        )
        .unwrap();
    (
        result,
        sink.errors()
            .iter()
            .map(|error| error.code.clone())
            .collect(),
    )
}

#[test]
fn required_and_forbidden_are_opposite_predicates_without_implicit_anchors() {
    for mode in ["required", "forbidden"] {
        let schema = SchemaCompiler
            .compile(raw(json!([constraint("AB", mode, "raw")]), false, false))
            .unwrap();
        for (bytes, matched) in [(b"xxAByy", true), (b"xxACyy", false)] {
            let valid = matched == (mode == "required");
            let (result, codes) = observe(&schema, bytes);
            assert_eq!(
                result,
                if valid {
                    FieldPipelineResult::Valid
                } else {
                    FieldPipelineResult::Invalid
                }
            );
            assert_eq!(
                codes,
                if valid {
                    vec![]
                } else {
                    vec![ValidationCode::InvalidDomain]
                }
            );
        }
    }
}
#[test]
fn target_selects_physical_padding_or_extracted_logical_bytes() {
    for (target, valid) in [("raw", false), ("logical", true)] {
        let schema = SchemaCompiler
            .compile(raw(
                json!([constraint("^AB$", "required", target)]),
                true,
                false,
            ))
            .unwrap();
        assert_eq!(observe(&schema, b"AB    ").1.is_empty(), valid);
    }
}
#[test]
fn malformed_and_unsupported_patterns_are_contextual_schema_errors() {
    for pattern in ["[", "(?=A)", r"(A)\1", ""] {
        let error = SchemaCompiler
            .compile(raw(
                json!([constraint(pattern, "required", "raw")]),
                false,
                false,
            ))
            .unwrap_err();
        assert!(
            error
                .issues
                .iter()
                .any(|issue| issue.code == SchemaIssueCode::InvalidRegex
                    && issue.trace_id.as_ref().is_some_and(|id| id.0 == "T1")
                    && issue.field_id.as_ref().is_some_and(|id| id.0 == "F1"))
        );
    }
}
#[test]
fn repeated_patterns_preserve_declaration_order_and_modes() {
    let schema = SchemaCompiler
        .compile(raw(
            json!([
                constraint("AB", "required", "raw"),
                constraint("AB", "forbidden", "raw")
            ]),
            false,
            false,
        ))
        .unwrap();
    // The shared compiled program still has two distinct declared predicates.
    assert_eq!(schema.traces()[0].fields()[0].constraints().len(), 2);
    assert_eq!(
        observe(&schema, b"xxAByy").1,
        [ValidationCode::InvalidDomain]
    );
    assert_eq!(
        observe(&schema, b"xxACyy").1,
        [ValidationCode::InvalidDomain]
    );
}
#[test]
fn datatype_error_stops_before_regex_and_valid_digits_reach_it() {
    let schema = SchemaCompiler
        .compile(raw(
            json!([constraint("^0+$", "required", "raw")]),
            false,
            true,
        ))
        .unwrap();
    assert_eq!(observe(&schema, b"ABCDEF").1, [ValidationCode::InvalidType]);
    assert_eq!(
        observe(&schema, b"123456").1,
        [ValidationCode::InvalidDomain]
    );
    assert!(observe(&schema, b"000000").1.is_empty());
}
#[test]
fn regex_byte_mode_can_express_invalid_utf8_without_decoding() {
    let schema = SchemaCompiler
        .compile(raw(
            json!([constraint(r"(?-u:^\xFF$)", "required", "raw")]),
            false,
            false,
        ))
        .unwrap();
    // Directly inspect the compiled byte program; the field datatype's encoding
    // checks remain authoritative and are intentionally not bypassed in engine.
    let lintra::CompiledConstraint::Regex { regex, .. } =
        &schema.traces()[0].fields()[0].constraints()[0]
    else {
        panic!("regex plan")
    };
    assert!(regex.is_match(&[255]));
    assert!(!regex.is_match("ÿ".as_bytes()));
}

#[test]
fn missing_values_stop_before_regex_for_both_presence_policies() {
    for (policy, expected) in [
        ("required", vec![ValidationCode::RequiredField]),
        ("optional", vec![]),
    ] {
        let mut value = serde_json::to_value(raw(
            json!([constraint("^AB$", "required", "logical")]),
            true,
            false,
        ))
        .unwrap();
        value["traces"][0]["fields"][0]["missing_value"] = json!({"kind":"spaces"});
        value["traces"][0]["fields"][0]["presence"]["base"] = json!(policy);
        let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
        raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
        let schema = SchemaCompiler.compile(raw).unwrap();
        assert_eq!(observe(&schema, b"      ").1, expected);
    }
}
