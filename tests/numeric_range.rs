//! Inclusive schema-defined ranges through compiler and the real field pipeline.
use lintra::*;
use serde_json::{Value, json};
fn raw_schema(width: u32, kind: &str, constraints: Value) -> RawFlowSchema {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("fixtures/application/schema.json")).unwrap();
    value["traces"][0]["record_length"] = (width + 3).into();
    let field = &mut value["traces"][0]["fields"][0];
    field["physical"]["type"] = kind.into();
    field["physical"]["length"] = width.into();
    field["physical"]["alignment"] = json!({"kind":"exact"});
    field["constraints"] = constraints;
    value["traces"][0]["fields"][1]["physical"]["start"] = (width + 1).into();
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    raw
}
fn check(schema: &CompiledFlowSchema, bytes: &[u8]) -> (FieldPipelineResult, Vec<ValidationError>) {
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
    (result, sink.errors().to_vec())
}
#[test]
fn inclusive_endpoints_and_leading_zeros_use_integer_value() {
    let schema = SchemaCompiler
        .compile(raw_schema(
            3,
            "numeric",
            json!([{"kind":"numeric_range","min":10,"max":20}]),
        ))
        .unwrap();
    for (bytes, valid) in [
        (b"009", false),
        (b"010", true),
        (b"015", true),
        (b"020", true),
        (b"021", false),
    ] {
        let (result, errors) = check(&schema, bytes);
        assert_eq!(result == FieldPipelineResult::Valid, valid);
        assert_eq!(errors.len(), usize::from(!valid));
        if !valid {
            assert_eq!(errors[0].code, ValidationCode::OutOfRange);
        }
    }
}
#[test]
fn datatype_and_presence_failures_stop_before_range() {
    let schema = SchemaCompiler
        .compile(raw_schema(
            3,
            "numeric",
            json!([{"kind":"numeric_range","min":10,"max":20}]),
        ))
        .unwrap();
    for (bytes, code) in [
        (b"A12", ValidationCode::InvalidType),
        (b"   ", ValidationCode::RequiredField),
    ] {
        let (_, errors) = check(&schema, bytes);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, code);
    }
}
#[test]
fn constraints_keep_declared_order_and_primary_error_stop() {
    let range = json!({"kind":"numeric_range","min":10,"max":20});
    let domain = json!({"kind":"allowed_values","target":"logical","values":["015"]});
    for (constraints, expected) in [
        (json!([range, domain]), ValidationCode::OutOfRange),
        (json!([domain, range]), ValidationCode::InvalidDomain),
    ] {
        let schema = SchemaCompiler
            .compile(raw_schema(3, "numeric", constraints))
            .unwrap();
        let (_, errors) = check(&schema, b"009");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, expected);
    }
}
#[test]
fn bounds_beyond_machine_integer_width_are_not_rounded_or_rejected() {
    let constraints: Value = serde_json::from_str(r#"[{"kind":"numeric_range","min":340282366920938463463374607431768211456,"max":340282366920938463463374607431768211457}]"#).unwrap();
    let schema = SchemaCompiler
        .compile(raw_schema(39, "numeric", constraints))
        .unwrap();
    assert_eq!(
        check(&schema, b"340282366920938463463374607431768211456").0,
        FieldPipelineResult::Valid
    );
    assert_eq!(
        check(&schema, b"340282366920938463463374607431768211458").1[0].code,
        ValidationCode::OutOfRange
    );
}
#[test]
fn compiler_rejects_reversed_bounds_and_incompatible_fields() {
    let reversed = SchemaCompiler
        .compile(raw_schema(
            3,
            "numeric",
            json!([{"kind":"numeric_range","min":20,"max":10}]),
        ))
        .unwrap_err();
    assert!(
        reversed
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::InvalidSchemaValue
                && issue.field_id == Some(FieldId("F1".into())))
    );
    let wrong_type = SchemaCompiler
        .compile(raw_schema(
            3,
            "alphanumeric",
            json!([{"kind":"numeric_range","min":0,"max":5}]),
        ))
        .unwrap_err();
    assert!(
        wrong_type
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::ConstraintTypeMismatch)
    );
}
#[test]
fn zero_and_signed_bounds_do_not_add_signed_numeric_record_syntax() {
    for (min, max, valid) in [(-1, 0, true), (-2, -1, false), (0, 0, true)] {
        let schema = SchemaCompiler
            .compile(raw_schema(
                3,
                "numeric",
                json!([{"kind":"numeric_range","min":min,"max":max}]),
            ))
            .unwrap();
        assert_eq!(
            check(&schema, b"000").0 == FieldPipelineResult::Valid,
            valid
        );
        assert_eq!(
            check(&schema, b"-01").1[0].code,
            ValidationCode::InvalidType
        );
    }
}
