use lintra::{
    CompiledFlowSchema, DiagnosticContextItem, DiagnosticValue, EncodingSpec, ErrorFactory,
    FieldId, FieldViolation, FlowId, RawFlowSchema, RawResolvedEncoding, RecordIdentity,
    SchemaCompiler, SchemaVersion, TraceId, ValidationCode,
};
use serde_json::json;

fn schema() -> CompiledFlowSchema {
    let mut raw: RawFlowSchema = serde_json::from_value(json!({
        "schema_format_version":1,"flow_id":"SYNTHETIC","schema_version":"v1","regulatory_version":"synthetic",
        "source":{},"encoding":{"mode":"runtime_required","position_unit":"byte"},"required_capabilities":["local.v1"],
        "parameters":[],"reference_data":[],"persistence_policy":null,
        "traces":[{"id":"T1","name":"Synthetic","record_length":6,
            "fields":([("F0",1,"Filler"),("F1",4,"Campo sintetico")].map(|(id,start,name)|json!({"id":id,"name":name,
                "physical":{"type":"alphanumeric","start":start,"length":3,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}},
                "missing_value":{"kind":"never_missing"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[]}))),
            "record_key":{"fields":["F0"],"uniqueness_scope":"trace","error_code":"E_DUPLICATE_RECORD_KEY"},
            "joins":[],"identity":[],"named_conditions":{},"derived_values":{},"repeated_groups":[],"rules":[],"documented_non_executable_rules":[]}]
    })).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw).unwrap()
}
fn factory(encoding: &EncodingSpec) -> ErrorFactory<'_> {
    // Static metadata only; no environmental defaults or source-file paths.
    static FLOW: std::sync::LazyLock<FlowId> =
        std::sync::LazyLock::new(|| FlowId("SYNTHETIC".into()));
    static VERSION: std::sync::LazyLock<SchemaVersion> =
        std::sync::LazyLock::new(|| SchemaVersion("v1".into()));
    static TRACE: std::sync::LazyLock<TraceId> = std::sync::LazyLock::new(|| TraceId("T1".into()));
    ErrorFactory {
        flow_id: &FLOW,
        schema_version: &VERSION,
        trace_id: &TRACE,
        encoding,
    }
}
fn violation(code: ValidationCode) -> FieldViolation {
    FieldViolation {
        code,
        expected: Some("atteso sintetico".into()),
        reason: "Dettaglio sintetico.".into(),
        diagnostic_context: vec![],
    }
}

#[test]
fn every_code_message_matches_the_versioned_byte_snapshot() {
    use ValidationCode::*;
    let codes = [
        RecordLength,
        RequiredField,
        ConditionalRequired,
        ForbiddenField,
        Alignment,
        InvalidType,
        InvalidDomain,
        OutOfRange,
        InvalidDate,
        CrossField,
        ReferenceData,
        CrossRecord,
        RuleConflict,
        RelatedRecordMissing,
        DuplicateRecordKey,
        RegulatoryRule("E_SYNTHETIC_RULE".into()),
    ];
    let schema = schema();
    let field = &schema.traces()[0].fields()[1];
    let mut snapshot = String::new();
    for code in codes {
        let error = factory(&EncodingSpec::Utf8).field_error(
            2,
            &RecordIdentity::default(),
            field,
            b"ABC",
            violation(code),
        );
        assert_eq!(error.validate(), Ok(()));
        snapshot.push_str(&format!(
            "{}\t{}\n",
            error.code.as_symbol().unwrap(),
            error.message
        ));
    }
    assert_eq!(
        snapshot.as_bytes(),
        include_bytes!("fixtures/error_factory_messages.txt")
    );
}

#[test]
fn field_errors_keep_structured_metadata_identity_context_and_lossless_diagnostics() {
    let schema = schema();
    let mut identity = RecordIdentity::default();
    identity
        .technical_ids
        .insert("test_id".into(), "SYNTHETIC-ID".into());
    let context = vec![DiagnosticContextItem {
        trace_id: TraceId("T2".into()),
        field_id: FieldId("TRIGGER".into()),
        value: DiagnosticValue {
            display: "YES".into(),
            raw_hex: None,
        },
    }];
    let mut failure = violation(ValidationCode::ConditionalRequired);
    failure.diagnostic_context = context.clone();
    let error = factory(&EncodingSpec::Utf8).field_error(
        9,
        &identity,
        &schema.traces()[0].fields()[1],
        b"\xff\0 ",
        failure,
    );
    assert_eq!(error.validate(), Ok(()));
    assert_eq!(
        (
            error.flow_id.0.as_str(),
            error.schema_version.0.as_str(),
            error.trace_id.0.as_str(),
            error.line_number
        ),
        ("SYNTHETIC", "v1", "T1", 9)
    );
    assert_eq!(error.record_identity, identity);
    assert_eq!(error.context, context);
    assert_eq!(error.expected.as_deref(), Some("atteso sintetico"));
    let field = error.field.unwrap();
    assert_eq!(
        (
            field.id.0.as_str(),
            field.name.as_ref(),
            field.start_one_based,
            field.end_one_based_inclusive
        ),
        ("F1", "Campo sintetico", 4, 6)
    );
    let actual = error.actual.unwrap();
    assert_eq!(actual.display, "\u{fffd}\\0 ");
    assert_eq!(actual.raw_hex.as_deref(), Some("FF 00 20"));
}

#[test]
fn rendering_uses_explicit_encoding_and_empty_reasons_need_no_message_parsing() {
    let schema = schema();
    let mut failure = violation(ValidationCode::InvalidDomain);
    failure.reason.clear();
    failure.expected = None;
    let error = factory(&EncodingSpec::Windows1252).field_error(
        1,
        &RecordIdentity::default(),
        &schema.traces()[0].fields()[1],
        b"\x80  ",
        failure,
    );
    assert_eq!(error.actual.unwrap().display, "\u{20ac}  ");
    assert_eq!(
        error.message,
        "Campo F1 (Campo sintetico): valore fuori dominio."
    );
    assert_eq!(error.expected, None);
    assert_eq!(error.code, ValidationCode::InvalidDomain);
}

#[test]
fn record_length_errors_keep_machine_data_without_inventing_field_or_identity() {
    let factory = factory(&EncodingSpec::Utf8);
    for actual in [0, 5, 7, usize::MAX] {
        let error = factory.record_length_error(3, None, 6, actual);
        assert_eq!(error.validate(), Ok(()));
        assert_eq!(error.code, ValidationCode::RecordLength);
        assert_eq!(error.field, None);
        assert_eq!(error.record_identity, RecordIdentity::default());
        assert_eq!(error.expected.as_deref(), Some("6 byte"));
        assert_eq!(error.actual.unwrap().display, actual.to_string());
        assert!(error.context.is_empty());
        assert_eq!(
            error.message,
            format!("Lunghezza record non valida: attesi 6 byte, letti {actual}.")
        );
    }
    let identity = RecordIdentity {
        episode_id: Some("SYNTHETIC-EPISODE".into()),
        ..RecordIdentity::default()
    };
    assert_eq!(
        factory
            .record_length_error(4, Some(&identity), 6, 5)
            .record_identity,
        identity
    );
}
