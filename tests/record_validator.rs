//! Real LocalOnly integration: compiled synthetic schema, audited source reads,
//! record framing, local stages and streaming diagnostics. No P7 service.
use lintra::{
    CompiledFlowSchema, FieldPipelineError, InputAuditSession, JsonlSink, NewlinePolicy,
    RawFlowSchema, RawRecord, RawResolvedEncoding, RecordReader, RecordValidationError,
    RecordValidator, ReportError, RunMetadata, RunStatus, SchemaCompiler, SchemaIssueCode,
    ScopeSummary, ValidationCode, ValidationError, ValidationSink, ValidationSummary,
    ValidationSummaryBuilder, VecSink,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    error::Error,
    io::{self, BufReader, Cursor, Read},
    time::{Duration, UNIX_EPOCH},
};

/// Two fields expose independent errors and explicit raw/logical policies.
/// The compiler's PK/identity projection remains nonempty throughout these tests.
fn raw_schema() -> RawFlowSchema {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    let trace = &mut value["traces"][0];
    let mut field = trace["fields"][0].clone();
    field["physical"]["length"] = json!(3);
    field["physical"]["type"] = json!("numeric");
    field["physical"]["alignment"] =
        json!({"kind":"right_pad","fill_byte":32,"content_policy":"strip_trailing_fill"});
    field["presence"]["base"] = json!("required");
    field["missing_value"] = json!({"kind":"spaces"});
    field["constraints"] = json!([{"kind":"allowed_values","target":"raw","values":["001","002"]}]);
    let mut second = field.clone();
    second["id"] = json!("F2");
    second["physical"]["start"] = json!(4);
    second["physical"]["type"] = json!("alphanumeric");
    second["constraints"] =
        json!([{"kind":"allowed_values","target":"logical","values":["ABC","XYZ"]}]);
    trace["fields"] = json!([field, second]);
    trace["identity"] = json!([{"target":"technical","technical_name":"synthetic","field":"F2"}]);
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    raw
}
fn schema() -> CompiledFlowSchema {
    SchemaCompiler.compile(raw_schema()).unwrap()
}

/// Common scoped counters. This helper does not count a record on behalf of
/// RecordValidator: the production record stage must do that exactly once.
fn builder(schema: &CompiledFlowSchema) -> ValidationSummaryBuilder {
    ValidationSummaryBuilder::new(
        schema.key().clone(),
        ScopeSummary::Trace {
            trace_id: schema.traces()[0].id().clone(),
        },
    )
}
fn metadata() -> RunMetadata {
    RunMetadata {
        validation_mode: lintra::ValidationMode::LocalOnly,
        validator_version: "synthetic".into(),
        schema_hash: "synthetic".into(),
        reference_data_versions: BTreeMap::new(),
        started_at: UNIX_EPOCH,
        inputs: vec![],
    }
}

/// Read all original bytes to real EOF for diagnostic counter tests too. No
/// manually forged Complete audit and no production summary after an Err.
fn audited_metadata(schema: &CompiledFlowSchema, bytes: &[u8]) -> RunMetadata {
    let audits = InputAuditSession::new();
    let mut reader = audits
        .wrap(
            schema.traces()[0].id().clone(),
            "synthetic.txt".into(),
            bytes.len().try_into().unwrap(),
            Cursor::new(bytes),
        )
        .unwrap();
    reader.read_to_end(&mut Vec::new()).unwrap();
    audits.finish(metadata()).unwrap()
}

/// A valid record returns actual extracted identity, leaves the projection
/// unchanged and updates exactly one processed/valid count, without errors.
#[test]
fn valid_local_record_extracts_identity_with_nonempty_projection() {
    let schema = schema();
    let plan = schema.traces()[0].projection_plan();
    assert_eq!(plan.field_indexes, [0, 1]);
    let mut validator = RecordValidator::new(&schema, schema.traces()[0].id()).unwrap();
    let mut sink = VecSink::default();
    let mut summary = builder(&schema);
    let artifacts = validator
        .validate_local(
            RawRecord {
                line_number: 1,
                bytes: b"001ABC",
            },
            &mut sink,
            &mut summary,
        )
        .unwrap();
    assert!(artifacts.valid);
    assert_eq!(
        artifacts.identity.unwrap().technical_ids["synthetic"].as_ref(),
        "ABC"
    );
    assert_eq!(schema.traces()[0].projection_plan().field_indexes, [0, 1]);
    assert!(sink.errors().is_empty());
    let summary = summary
        .finish(audited_metadata(&schema, b"001ABC"))
        .unwrap();
    assert_eq!(
        (
            summary.records_processed,
            summary.valid_records,
            summary.invalid_records,
            summary.total_errors
        ),
        (1, 1, 0, 0)
    );
}

/// Both field failures are emitted in physical order. Identity is diagnostic
/// even if its source value fails a domain; Err never wraps those violations.
#[test]
fn normative_failures_return_invalid_and_continue_to_other_fields() {
    let schema = schema();
    let mut validator = RecordValidator::new(&schema, schema.traces()[0].id()).unwrap();
    let mut sink = VecSink::default();
    let mut summary = builder(&schema);
    let outcome = validator
        .validate_local(
            RawRecord {
                line_number: 2,
                bytes: b"xxxyyy",
            },
            &mut sink,
            &mut summary,
        )
        .unwrap();
    assert!(!outcome.valid);
    assert_eq!(
        outcome.identity.unwrap().technical_ids["synthetic"].as_ref(),
        "yyy"
    );
    assert_eq!(
        sink.errors()
            .iter()
            .map(|e| (&e.code, e.field.as_ref().unwrap().id.0.as_str()))
            .collect::<Vec<_>>(),
        [
            (&ValidationCode::InvalidType, "F1"),
            (&ValidationCode::InvalidDomain, "F2")
        ]
    );
    let summary = summary
        .finish(audited_metadata(&schema, b"xxxyyy"))
        .unwrap();
    assert_eq!(
        (
            summary.records_processed,
            summary.invalid_records,
            summary.total_errors
        ),
        (1, 1, 2)
    );
    assert_eq!(summary.run_status, RunStatus::CompletedInvalid);
}

/// Bytes that would also violate type/presence are never sliced when their
/// width is wrong, including empty input. No partial identity is extracted.
#[test]
fn malformed_records_emit_only_length_and_never_identity_or_fields() {
    let schema = schema();
    for bytes in [b"".as_slice(), b"   ", b"xxxBADextra"] {
        let mut validator = RecordValidator::new(&schema, schema.traces()[0].id()).unwrap();
        let mut sink = VecSink::default();
        let mut summary = builder(&schema);
        let outcome = validator
            .validate_local(
                RawRecord {
                    line_number: 1,
                    bytes,
                },
                &mut sink,
                &mut summary,
            )
            .unwrap();
        assert!(!outcome.valid && outcome.identity.is_none());
        let errors = sink.errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationCode::RecordLength);
        assert!(errors[0].field.is_none());
        assert_eq!(errors[0].record_identity, lintra::RecordIdentity::default());
        let summary = summary.finish(audited_metadata(&schema, bytes)).unwrap();
        assert_eq!(
            (
                summary.records_processed,
                summary.valid_records,
                summary.invalid_records,
                summary.total_errors
            ),
            (1, 0, 1, 1)
        );
    }
}

/// Test-only reader loop uses real framing/audit and local RecordValidator,
/// without implementing the future TraceValidator/ValidationService.
fn stream(bytes: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<bool>) {
    let schema = schema();
    let mut validator = RecordValidator::new(&schema, schema.traces()[0].id()).unwrap();
    let audits = InputAuditSession::new();
    let reader = audits
        .wrap(
            schema.traces()[0].id().clone(),
            "synthetic.txt".into(),
            bytes.len().try_into().unwrap(),
            Cursor::new(bytes),
        )
        .unwrap();
    let mut reader = RecordReader::new(BufReader::new(reader), NewlinePolicy::AutoLfOrCrlf);
    let mut sink = JsonlSink::new(Vec::new());
    let mut summary = builder(&schema);
    let mut buffer = Vec::new();
    let mut results = Vec::new();
    while let Some(record) = reader.read_next(&mut buffer).unwrap() {
        results.push(
            validator
                .validate_local(record, &mut sink, &mut summary)
                .unwrap()
                .valid,
        );
    }
    let mut summary = summary.finish(audits.finish(metadata()).unwrap()).unwrap();
    summary.elapsed = Duration::ZERO; // fix only test timing, before encoding
    (
        sink.finish().unwrap(),
        lintra::write_summary(Vec::new(), &summary, UNIX_EPOCH).unwrap(),
        results,
    )
}

/// Repeated complete runs give identical JSONL/summary bytes; counters reflect
/// valid, invalid and malformed records without double counting fields.
#[test]
fn audited_stream_is_deterministic_and_counts_mixed_records() {
    let input = b"001ABC\r\nxxxyyy\r\nx\r\n001ABCx\r\n   ABC\r\n002   \r\n";
    let first = stream(input);
    assert_eq!(first, stream(input));
    assert_eq!(first.2, [true, false, false, false, false, false]);
    let summary: Value = serde_json::from_slice(&first.1).unwrap();
    assert_eq!(summary["records_processed"], 6);
    assert_eq!(summary["valid_records"], 1);
    assert_eq!(summary["invalid_records"], 5);
    assert_eq!(summary["total_errors"], 6);
    let lines: Vec<Value> = std::str::from_utf8(&first.0)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        lines
            .iter()
            .map(|v| (v["line"].as_u64().unwrap(), v["code"].as_str().unwrap()))
            .collect::<Vec<_>>(),
        [
            (2, "E_INVALID_TYPE"),
            (2, "E_INVALID_DOMAIN"),
            (3, "E_RECORD_LENGTH"),
            (4, "E_RECORD_LENGTH"),
            (5, "E_REQUIRED_FIELD"),
            (6, "E_REQUIRED_FIELD")
        ]
    );
}

/// Optional absence must count as locally valid, preserving field-stage rules.
#[test]
fn missing_optional_field_does_not_make_record_invalid() {
    let mut raw = raw_schema();
    // Mutate only the synthetic input JSON, not private compiled invariants.
    let mut value = serde_json::to_value(&raw).unwrap();
    value["traces"][0]["fields"][1]["presence"]["base"] = json!("optional");
    raw = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let schema = SchemaCompiler.compile(raw).unwrap();
    let mut summary = builder(&schema);
    let mut sink = VecSink::default();
    let outcome = RecordValidator::new(&schema, schema.traces()[0].id())
        .unwrap()
        .validate_local(
            RawRecord {
                line_number: 1,
                bytes: b"001   ",
            },
            &mut sink,
            &mut summary,
        )
        .unwrap();
    assert!(outcome.valid);
    assert!(outcome.identity.unwrap().technical_ids.is_empty());
    assert!(sink.errors().is_empty());
}

/// Count attempted emissions; reject the write before storing any diagnostic.
struct FailingSink {
    attempts: usize,
}
impl ValidationSink for FailingSink {
    fn on_error(&mut self, _: &ValidationError) -> Result<(), ReportError> {
        self.attempts += 1;
        Err(ReportError::Write(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "synthetic",
        )))
    }
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        panic!("record validator must not publish summary")
    }
}

/// Both length and field emissions preserve real sink payloads and terminal
/// behavior. Partial counters are inspected only as cancelled diagnostic data.
#[test]
fn sink_failure_is_typed_terminal_and_never_counted_as_emitted() {
    let schema = schema();
    for bytes in [b"short".as_slice(), b"xxxABC"] {
        let mut validator = RecordValidator::new(&schema, schema.traces()[0].id()).unwrap();
        let mut summary = builder(&schema);
        let mut sink = FailingSink { attempts: 0 };
        let error = validator
            .validate_local(
                RawRecord {
                    line_number: 1,
                    bytes,
                },
                &mut sink,
                &mut summary,
            )
            .unwrap_err();
        assert!(error.source().is_some());
        let source = match error {
            RecordValidationError::Report(ReportError::Write(source)) if bytes.len() != 6 => source,
            RecordValidationError::Pipeline(FieldPipelineError::Report(ReportError::Write(
                source,
            ))) if bytes.len() == 6 => source,
            other => panic!("unexpected error: {other:?}"),
        };
        assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
        assert!(matches!(
            validator.validate_local(
                RawRecord {
                    line_number: 2,
                    bytes: b"001ABC"
                },
                &mut sink,
                &mut summary
            ),
            Err(RecordValidationError::PreviousFailure)
        ));
        assert_eq!(sink.attempts, 1);
        summary.cancel();
        let partial = summary.finish(audited_metadata(&schema, bytes)).unwrap();
        assert_eq!(
            (
                partial.records_processed,
                partial.valid_records,
                partial.invalid_records,
                partial.total_errors
            ),
            (1, 0, 0, 0)
        );
    }
}

/// A poisoned builder fails before any emission; the typed payload is retained.
#[test]
fn counter_failure_stops_the_record_without_normative_errors() {
    let schema = schema();
    let mut summary = builder(&schema);
    assert!(
        summary
            .error_emitted(&ValidationCode::RegulatoryRule("bad".into()))
            .is_err()
    );
    let mut sink = VecSink::default();
    let error = RecordValidator::new(&schema, schema.traces()[0].id())
        .unwrap()
        .validate_local(
            RawRecord {
                line_number: 1,
                bytes: b"001ABC",
            },
            &mut sink,
            &mut summary,
        )
        .unwrap_err();
    assert!(matches!(
        error,
        RecordValidationError::Summary(lintra::SummaryBuildError::PreviousFailure)
    ));
    assert!(sink.errors().is_empty());
    assert!(summary.finish(metadata()).is_err());
}

/// Bad trace/zero line metadata cannot become a valid local result.
#[test]
fn invalid_trace_and_zero_line_are_explicit_technical_errors() {
    let schema = schema();
    assert!(matches!(
        RecordValidator::new(&schema, &lintra::TraceId("absent".into())),
        Err(RecordValidationError::TraceNotFound(_))
    ));
    let mut summary = builder(&schema);
    let mut sink = VecSink::default();
    assert!(matches!(
        RecordValidator::new(&schema, schema.traces()[0].id())
            .unwrap()
            .validate_local(
                RawRecord {
                    line_number: 0,
                    bytes: b"001ABC"
                },
                &mut sink,
                &mut summary
            ),
        Err(RecordValidationError::InvalidLineNumber)
    ));
    assert!(sink.errors().is_empty());
    assert_eq!(summary.finish(metadata()).unwrap().records_processed, 0);
}

/// Unsupported primitives and missing dataset declarations remain distinct
/// compiler errors; neither is tolerated as degraded runtime data.
#[test]
fn compiler_rejects_unsupported_assertions_and_undeclared_reference_data() {
    for (name, payload) in [
        (
            "rules",
            json!([{"id":"FUTURE","when":{"const":true},"assertions":[{"kind":"record_exists","join":"J"}],"error_code":"E_CROSS_FIELD","message_template":"Future {rule_id}"}]),
        ),
        (
            "constraints",
            json!([{"kind":"reference_data","datasets":["absent"],"fallback_values":[],"metadata_bindings":[]}]),
        ),
    ] {
        let mut value = serde_json::to_value(raw_schema()).unwrap();
        if name == "rules" {
            assert!(!payload.as_array().unwrap().is_empty());
            value["traces"][0][name] = payload;
        } else {
            value["traces"][0]["fields"][0][name] = payload;
        }
        let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
        raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
        let error = SchemaCompiler.compile(raw).unwrap_err();
        assert!(error.issues.iter().any(|issue| issue.code
            == if name == "rules" {
                SchemaIssueCode::UnsupportedConstruct
            } else {
                SchemaIssueCode::MissingReferenceDatasetDeclaration
            }));
    }
}

/// An architectural dependency test scans only engine source; it catches direct
/// or crate-root-qualified application references, not merely one import style.
#[test]
fn engine_has_no_application_dependency() {
    for source in [
        include_str!("../src/engine/record_validator.rs"),
        include_str!("../src/engine/field_pipeline.rs"),
        include_str!("../src/engine/identity_extractor.rs"),
        include_str!("../src/engine/error_factory.rs"),
    ] {
        assert!(!source.contains("application::"));
    }
}
