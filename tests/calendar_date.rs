//! The approved DDMMYYYY schema drives calendar checks and pipeline stop rules.
use lintra::*;
use serde_json::{Value, json};
fn raw_schema(constraints: Value) -> RawFlowSchema {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("fixtures/application/schema.json")).unwrap();
    value["traces"][0]["record_length"] = 11.into();
    value["traces"][0]["fields"][0]["physical"] = json!({"type":"date","start":1,"length":8,"format":"DDMMYYYY","alignment":{"kind":"exact"}});
    value["traces"][0]["fields"][0]["constraints"] = constraints;
    value["traces"][0]["fields"][1]["physical"]["start"] = 9.into();
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
fn calendar_table_checks_centuries_leap_years_and_explicit_year_bounds() {
    let schema = SchemaCompiler.compile(raw_schema(json!([]))).unwrap();
    assert!(schema.traces()[0].fields()[0].lexical().is_none());
    for (bytes, valid) in [
        (b"29022000", true),
        (b"29021900", false),
        (b"29022024", true),
        (b"29022023", false),
        (b"31042026", false),
        (b"01132026", false),
        (b"00012026", false),
        (b"01010001", true),
        (b"31129999", true),
        (b"01010000", false),
        (b"20261231", false),
        (b"01/01/26", false),
    ] {
        let (result, errors) = check(&schema, bytes);
        assert_eq!(result == FieldPipelineResult::Valid, valid, "{bytes:?}");
        if !valid {
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].code, ValidationCode::InvalidDate);
        }
    }
}
#[test]
fn missing_presence_and_bad_dates_stop_before_domain_constraints() {
    let schema = SchemaCompiler
        .compile(raw_schema(
            json!([{"kind":"allowed_values","target":"logical","values":["01012026"]}]),
        ))
        .unwrap();
    for (bytes, expected) in [
        (b"        ", ValidationCode::RequiredField),
        (b"31022026", ValidationCode::InvalidDate),
        (b"02012026", ValidationCode::InvalidDomain),
    ] {
        let (_, errors) = check(&schema, bytes);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, expected);
    }
}
#[test]
fn optional_missing_date_is_not_a_calendar_violation() {
    let mut raw = raw_schema(json!([]));
    let mut value = serde_json::to_value(&raw).unwrap();
    value["traces"][0]["fields"][0]["presence"]["base"] = "optional".into();
    raw = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let schema = SchemaCompiler.compile(raw).unwrap();
    let (result, errors) = check(&schema, b"        ");
    assert_eq!(result, FieldPipelineResult::MissingOptional);
    assert!(errors.is_empty());
}
#[test]
fn explicit_calendar_constraint_is_supported_but_cannot_be_disabled() {
    let schema = SchemaCompiler
        .compile(raw_schema(
            json!([{"kind":"date","format":"DDMMYYYY","calendar_valid":true}]),
        ))
        .unwrap();
    assert!(matches!(
        schema.traces()[0].fields()[0].constraints()[0],
        CompiledConstraint::Date { .. }
    ));
    assert_eq!(check(&schema, b"29022000").0, FieldPipelineResult::Valid);
    assert_eq!(
        check(&schema, b"29021900").1[0].code,
        ValidationCode::InvalidDate
    );
    let error = SchemaCompiler
        .compile(raw_schema(
            json!([{"kind":"date","format":"DDMMYYYY","calendar_valid":false}]),
        ))
        .unwrap_err();
    assert!(
        error
            .issues
            .iter()
            .any(|i| i.code == SchemaIssueCode::InvalidSchemaValue)
    );
}
#[test]
fn invalid_date_width_and_constraint_type_are_compiler_errors() {
    let mut value = serde_json::to_value(raw_schema(json!([]))).unwrap();
    value["traces"][0]["fields"][0]["physical"]["length"] = 7.into();
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let error = SchemaCompiler.compile(raw).unwrap_err();
    assert!(
        error
            .issues
            .iter()
            .any(|i| i.code == SchemaIssueCode::InvalidTypeParameters)
    );
    let mut value: Value =
        serde_json::from_slice(include_bytes!("fixtures/application/schema.json")).unwrap();
    value["traces"][0]["fields"][0]["constraints"] =
        json!([{"kind":"date","format":"DDMMYYYY","calendar_valid":true}]);
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let error = SchemaCompiler.compile(raw).unwrap_err();
    assert!(
        error
            .issues
            .iter()
            .any(|i| i.code == SchemaIssueCode::ConstraintTypeMismatch)
    );
}
