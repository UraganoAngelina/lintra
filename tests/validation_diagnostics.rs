use lintra::{
    ByteRange, DiagnosticContextItem, DiagnosticValue, FieldDiagnostic, FieldId, FlowId,
    InvalidDiagnostic, InvalidValidationCode, RecordIdentity, SchemaVersion, TraceId,
    ValidationCode, ValidationError,
};
use std::error::Error;

fn example() -> ValidationError {
    ValidationError {
        code: ValidationCode::ConditionalRequired,
        flow_id: FlowId("SYNTHETIC".into()),
        schema_version: SchemaVersion("v1".into()),
        trace_id: TraceId("T1".into()),
        line_number: 7,
        record_identity: RecordIdentity::default(),
        field: Some(FieldDiagnostic::from_range(
            FieldId("F1".into()),
            "Synthetic field".into(),
            ByteRange::new(9, 3).unwrap(),
        )),
        actual: Some(DiagnosticValue {
            display: "   ".into(),
            raw_hex: None,
        }),
        expected: Some("value required by synthetic rule".into()),
        context: vec![DiagnosticContextItem {
            trace_id: TraceId("T2".into()),
            field_id: FieldId("F2".into()),
            value: DiagnosticValue {
                display: "X\\0".into(),
                raw_hex: Some("58 00".into()),
            },
        }],
        message: "Synthetic condition requires F1".into(),
    }
}

#[test]
fn field_metadata_converts_byte_offsets_once_including_machine_boundary() {
    for (start, len, first, last) in [
        (0, 1, 1, 1),
        (9, 3, 10, 12),
        (1, 2, 2, 3),
        (usize::MAX - 1, 1, usize::MAX, usize::MAX),
    ] {
        let field = FieldDiagnostic::from_range(
            FieldId("F".into()),
            "Name".into(),
            ByteRange::new(start, len).unwrap(),
        );
        assert_eq!(
            (field.start_one_based, field.end_one_based_inclusive),
            (first, last)
        );
        assert_eq!(field.validate(), Ok(()));
    }
    // Length is measured in bytes; the two-byte UTF-8 character occupies 2..=3.
    let bytes = b"A\xc3\xa8Z";
    let range = ByteRange::new(1, 2).unwrap();
    assert_eq!(range.slice(bytes), Some(&bytes[1..3]));
    let field = FieldDiagnostic::from_range(FieldId("F".into()), "Name".into(), range);
    assert_eq!(
        (field.start_one_based, field.end_one_based_inclusive),
        (2, 3)
    );
}

#[test]
fn field_and_record_diagnostics_keep_identity_optional_and_payloads_owned() {
    let error = example();
    assert_eq!(error.validate(), Ok(()));
    assert_eq!(error.line_number, 7);
    assert_eq!(error.field.as_ref().unwrap().id.0, "F1");
    assert_eq!(error.context[0].trace_id.0, "T2");
    assert_eq!(error.context[0].value.raw_hex.as_deref(), Some("58 00"));
    assert_eq!(error.actual.as_ref().unwrap().display, "   ");
    assert_eq!(error.record_identity, RecordIdentity::default());
    let mut record = error.clone();
    record.code = ValidationCode::RecordLength;
    record.field = None;
    record.actual = None;
    record.expected = None;
    record.context.clear();
    assert_eq!(record.validate(), Ok(()));
    assert!(error.field.is_some());
}

#[test]
fn mandatory_record_metadata_cannot_pass_validation_when_missing_or_zero() {
    let mut error = example();
    error.flow_id.0.clear();
    assert_eq!(error.validate(), Err(InvalidDiagnostic::EmptyFlowId));
    let mut error = example();
    error.schema_version.0.clear();
    assert_eq!(error.validate(), Err(InvalidDiagnostic::EmptySchemaVersion));
    let mut error = example();
    error.trace_id.0.clear();
    assert_eq!(error.validate(), Err(InvalidDiagnostic::EmptyTraceId));
    let mut error = example();
    error.line_number = 0;
    assert_eq!(error.validate(), Err(InvalidDiagnostic::ZeroLineNumber));
    error.line_number = u64::MAX;
    assert_eq!(error.validate(), Ok(()));
}

#[test]
fn manually_constructed_field_metadata_must_be_complete_and_ordered() {
    let mut error = example();
    error.field.as_mut().unwrap().id.0.clear();
    assert_eq!(error.validate(), Err(InvalidDiagnostic::EmptyFieldId));
    let mut error = example();
    error.field.as_mut().unwrap().name = "".into();
    assert_eq!(error.validate(), Err(InvalidDiagnostic::EmptyFieldName));
    for (start, end) in [(0, 0), (0, 1), (2, 1)] {
        let mut error = example();
        let field = error.field.as_mut().unwrap();
        field.start_one_based = start;
        field.end_one_based_inclusive = end;
        assert_eq!(
            error.validate(),
            Err(InvalidDiagnostic::InvalidFieldPositions)
        );
    }
}

#[test]
fn invalid_context_identifiers_and_codes_are_technical_metadata_failures() {
    let mut error = example();
    error.context[0].trace_id.0.clear();
    assert_eq!(
        error.validate(),
        Err(InvalidDiagnostic::EmptyContextTraceId)
    );
    let mut error = example();
    error.context[0].field_id.0.clear();
    assert_eq!(
        error.validate(),
        Err(InvalidDiagnostic::EmptyContextFieldId)
    );
    let mut error = example();
    error.code = ValidationCode::RegulatoryRule("E_SCHEMA_ERROR".into());
    let invalid = error.validate().unwrap_err();
    assert_eq!(
        invalid,
        InvalidDiagnostic::InvalidCode(InvalidValidationCode)
    );
    assert!(
        invalid
            .source()
            .unwrap()
            .downcast_ref::<InvalidValidationCode>()
            .is_some()
    );
    error.code = "E_SYNTHETIC_RULE".parse().unwrap();
    assert_eq!(error.validate(), Ok(()));
}
