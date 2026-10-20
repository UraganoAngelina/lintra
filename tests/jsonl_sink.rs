use lintra::{
    DiagnosticContextItem, DiagnosticValue, FieldDiagnostic, FieldId, FlowId, JsonlSink,
    RecordIdentity, ReportError, RunStatus, SchemaKey, SchemaVersion, ScopeSummary, TraceId,
    ValidationCode, ValidationError, ValidationSink, ValidationSummary,
};
use std::{
    collections::BTreeMap,
    error::Error,
    io::{self, Write},
    time::Duration,
};

fn field_error() -> ValidationError {
    let mut technical_ids = BTreeMap::new();
    technical_ids.insert("Z".into(), "SYNTHETIC-Z".into());
    technical_ids.insert("A".into(), "SYNTHETIC-A".into());
    ValidationError {
        code: ValidationCode::RegulatoryRule("E_SYNTHETIC_RULE".into()),
        flow_id: FlowId("SYNTHETIC".into()),
        schema_version: SchemaVersion("v1".into()),
        trace_id: TraceId("T1".into()),
        line_number: 7,
        record_identity: RecordIdentity {
            technical_ids,
            ..RecordIdentity::default()
        },
        field: Some(FieldDiagnostic {
            id: FieldId("F1".into()),
            name: "Campo à".into(),
            start_one_based: 1,
            end_one_based_inclusive: 3,
        }),
        actual: Some(DiagnosticValue {
            display: "\\0é".into(),
            raw_hex: Some("00 C3 A9".into()),
        }),
        expected: Some("testo".into()),
        context: vec![DiagnosticContextItem {
            trace_id: TraceId("T2".into()),
            field_id: FieldId("F2".into()),
            value: DiagnosticValue {
                display: "01".into(),
                raw_hex: None,
            },
        }],
        message: "Errore sintetico \"à\".".into(),
    }
}
fn record_error() -> ValidationError {
    ValidationError {
        code: ValidationCode::RecordLength,
        field: None,
        actual: None,
        expected: None,
        context: vec![],
        record_identity: RecordIdentity::default(),
        line_number: 8,
        message: "Record corto\nriga.".into(),
        ..field_error()
    }
}
fn summary() -> ValidationSummary {
    ValidationSummary {
        run_status: RunStatus::CompletedInvalid,
        schema_key: SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        scope_summary: ScopeSummary::Trace {
            trace_id: TraceId("T1".into()),
        },
        records_processed: 2,
        valid_records: 0,
        invalid_records: 2,
        total_errors: 2,
        errors_by_code: BTreeMap::from([
            (ValidationCode::RecordLength, 1),
            (ValidationCode::RegulatoryRule("E_SYNTHETIC_RULE".into()), 1),
        ]),
        skipped_rules: 0,
        truncated: false,
        validator_version: "synthetic-build".into(),
        schema_hash: "synthetic-hash".into(),
        reference_data_versions: BTreeMap::new(),
        elapsed: Duration::from_millis(1),
        inputs: Vec::new(),
    }
}

#[test]
fn jsonl_matches_versioned_utf8_bytes_without_bom_crlf_or_summary_line() {
    let mut sink = JsonlSink::new(Vec::new());
    sink.on_error(&field_error()).unwrap();
    sink.on_error(&record_error()).unwrap();
    let before_summary = sink.get_ref().clone();
    sink.on_summary(&summary()).unwrap();
    assert_eq!(sink.get_ref(), &before_summary);
    let output = sink.finish().unwrap();
    assert_eq!(output, include_bytes!("fixtures/diagnostics.errors.jsonl"));
    assert!(!output.starts_with(b"\xef\xbb\xbf"));
    assert!(!output.contains(&b'\r'));
    assert_eq!(output.iter().filter(|&&byte| byte == b'\n').count(), 2);
    for line in output
        .split(|&byte| byte == b'\n')
        .filter(|line| !line.is_empty())
    {
        let value: serde_json::Value = serde_json::from_slice(line).unwrap();
        assert!(value["code"].as_str().unwrap().starts_with("E_"));
        assert!(value.get("run_status").is_none());
    }
}

#[test]
fn map_order_is_stable_and_same_field_id_in_different_context_traces_is_preserved() {
    let mut first = field_error();
    first.context.push(DiagnosticContextItem {
        trace_id: TraceId("T3".into()),
        ..first.context[0].clone()
    });
    let mut second = first.clone();
    second.record_identity.technical_ids = first
        .record_identity
        .technical_ids
        .iter()
        .rev()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let mut a = JsonlSink::new(Vec::new());
    let mut b = JsonlSink::new(Vec::new());
    a.on_error(&first).unwrap();
    b.on_error(&second).unwrap();
    assert_eq!(a.get_ref(), b.get_ref());
    let parsed: serde_json::Value = serde_json::from_slice(a.get_ref()).unwrap();
    assert_eq!(parsed["context"].as_array().unwrap().len(), 2);
    assert_eq!(parsed["context"][0]["trace"], "T2");
    assert_eq!(parsed["context"][1]["trace"], "T3");
    assert_eq!(parsed["actual"]["raw_hex"], "00 C3 A9");
    assert_eq!(parsed["field"]["start"], 1);
    assert_eq!(parsed["field"]["end"], 3);
}

struct FaultyWriter {
    bytes: Vec<u8>,
    remaining: usize,
    fail_flush: bool,
}
impl Write for FaultyWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "synthetic write failure",
            ));
        }
        // Also exercises short writes, including UTF-8 split across writes.
        let size = bytes.len().min(self.remaining).min(1);
        self.bytes.extend_from_slice(&bytes[..size]);
        self.remaining -= size;
        Ok(size)
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.fail_flush {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "synthetic flush failure",
            ))
        } else {
            Ok(())
        }
    }
}
#[test]
fn partial_json_and_newline_failures_keep_their_cause_and_are_terminal() {
    let mut reference = JsonlSink::new(Vec::new());
    reference.on_error(&field_error()).unwrap();
    let line = reference.finish().unwrap();
    for limit in [0, 4, line.len() - 1] {
        let mut sink = JsonlSink::new(FaultyWriter {
            bytes: vec![],
            remaining: limit,
            fail_flush: false,
        });
        let failure = sink.on_error(&field_error()).unwrap_err();
        if limit == line.len() - 1 {
            assert!(matches!(failure, ReportError::Write(_)));
            assert_eq!(
                failure
                    .source()
                    .unwrap()
                    .downcast_ref::<io::Error>()
                    .unwrap()
                    .kind(),
                io::ErrorKind::PermissionDenied
            );
        } else {
            let json = failure
                .source()
                .unwrap()
                .downcast_ref::<serde_json::Error>()
                .unwrap();
            assert_eq!(json.io_error_kind(), Some(io::ErrorKind::PermissionDenied));
        }
        assert_eq!(sink.get_ref().bytes, line[..limit]);
        assert!(matches!(
            sink.on_error(&record_error()),
            Err(ReportError::PreviousFailure)
        ));
        assert!(matches!(
            sink.on_summary(&summary()),
            Err(ReportError::PreviousFailure)
        ));
        assert!(matches!(sink.finish(), Err(ReportError::PreviousFailure)));
    }
}

#[test]
fn writes_are_immediate_and_flush_failures_never_claim_success() {
    let mut sink = JsonlSink::new(FaultyWriter {
        bytes: vec![],
        remaining: usize::MAX,
        fail_flush: false,
    });
    sink.on_error(&field_error()).unwrap();
    assert!(!sink.get_ref().bytes.is_empty());
    assert_eq!(
        sink.finish().unwrap().bytes,
        include_bytes!("fixtures/diagnostics.errors.jsonl")
            .split_inclusive(|&b| b == b'\n')
            .next()
            .unwrap()
    );
    for via_summary in [true, false] {
        let mut sink = JsonlSink::new(FaultyWriter {
            bytes: vec![],
            remaining: usize::MAX,
            fail_flush: true,
        });
        sink.on_error(&field_error()).unwrap();
        let failure = if via_summary {
            sink.on_summary(&summary()).unwrap_err()
        } else {
            sink.finish().err().unwrap()
        };
        assert_eq!(
            failure
                .source()
                .unwrap()
                .downcast_ref::<io::Error>()
                .unwrap()
                .kind(),
            io::ErrorKind::BrokenPipe
        );
    }
}

#[test]
fn malformed_diagnostic_writes_nothing_and_empty_stream_has_no_metadata_line() {
    let mut error = field_error();
    error.line_number = 0;
    let mut sink = JsonlSink::new(Vec::new());
    assert!(matches!(
        sink.on_error(&error),
        Err(ReportError::InvalidDiagnostic(_))
    ));
    assert!(sink.get_ref().is_empty());
    assert!(matches!(sink.finish(), Err(ReportError::PreviousFailure)));
    let mut empty = JsonlSink::new(Vec::new());
    empty.on_summary(&summary()).unwrap();
    assert!(empty.finish().unwrap().is_empty());
}
