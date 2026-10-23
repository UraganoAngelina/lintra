//! Byte-exact independent expectations, synthetic diagnostics only.
use lintra::*;
use std::{
    borrow::Cow,
    io::{self, Write},
};

const HEADER: &str = "code,flow,schema_version,trace,line,record_id,fiscal_code,field_id,field_name,start,end,actual,expected,message\r\n";
fn diagnostic() -> ValidationError {
    ValidationError {
        code: ValidationCode::InvalidDomain,
        flow_id: FlowId("SYNTHETIC".into()),
        schema_version: SchemaVersion("v1".into()),
        trace_id: TraceId("T".into()),
        line_number: 7,
        record_identity: RecordIdentity {
            sioc_record_id: Some("00123".into()),
            fiscal_code: Some("SYNTHETIC_ONLY".into()),
            ..Default::default()
        },
        field: Some(FieldDiagnostic {
            id: FieldId("F".into()),
            name: "Campo à".into(),
            start_one_based: 2,
            end_one_based_inclusive: 6,
        }),
        actual: Some(DiagnosticValue {
            display: "00,\"é\"\r\nX".into(),
            raw_hex: None,
        }),
        expected: Some("=00123".into()),
        context: vec![],
        message: "errore\nriga\rfine".into(),
    }
}

#[test]
fn exact_rfc4180_bytes_quote_commas_quotes_and_both_newline_bytes() {
    let mut sink = CsvSink::new(Vec::new()).unwrap();
    sink.on_error(&diagnostic()).unwrap();
    let bytes = sink.finish().unwrap();
    let expected = concat!(
        "code,flow,schema_version,trace,line,record_id,fiscal_code,field_id,field_name,start,end,actual,expected,message\r\n",
        "E_INVALID_DOMAIN,SYNTHETIC,v1,T,7,00123,SYNTHETIC_ONLY,F,Campo à,2,6,\"00,\"\"é\"\"\r\nX\",=00123,\"errore\nriga\rfine\"\r\n"
    );
    assert_eq!(bytes, expected.as_bytes());
    assert!(!bytes.starts_with(b"\xef\xbb\xbf"));
}

#[test]
fn empty_run_contains_header_once_and_record_errors_have_empty_optional_cells() {
    assert_eq!(
        CsvSink::new(Vec::new()).unwrap().finish().unwrap(),
        HEADER.as_bytes()
    );
    let mut error = diagnostic();
    error.code = ValidationCode::RecordLength;
    error.line_number = u64::MAX;
    error.record_identity = RecordIdentity::default();
    error.field = None;
    error.actual = None;
    error.expected = None;
    error.message = "corto".into();
    let mut sink = CsvSink::new(Vec::new()).unwrap();
    sink.on_error(&error).unwrap();
    assert_eq!(
        sink.finish().unwrap(),
        format!("{HEADER}E_RECORD_LENGTH,SYNTHETIC,v1,T,18446744073709551615,,,,,,,,,corto\r\n")
            .as_bytes()
    );
}

#[test]
fn repeated_rows_preserve_order_and_all_quotes_are_doubled() {
    let mut sink = CsvSink::new(Vec::new()).unwrap();
    let mut error = diagnostic();
    error.message = "\"\"".into();
    sink.on_error(&error).unwrap();
    error.message = " 00123 ".into();
    sink.on_error(&error).unwrap();
    let bytes = sink.finish().unwrap();
    assert!(bytes.ends_with(b", 00123 \r\n"));
    assert_eq!(
        bytes
            .windows(8)
            .filter(|w| *w == b"\"\"\"\"\"\"\r\n")
            .count(),
        1
    );
    assert_eq!(
        bytes
            .windows(HEADER.len())
            .filter(|w| *w == HEADER.as_bytes())
            .count(),
        1
    );
}

#[test]
fn shared_columns_use_borrowed_text_and_never_numeric_coercion() {
    let error = diagnostic();
    assert_eq!(ERROR_COLUMNS.len(), 14);
    assert_eq!(
        ERROR_COLUMNS
            .iter()
            .map(|c| c.header())
            .collect::<Vec<_>>()
            .join(",")
            + "\r\n",
        HEADER
    );
    assert!(matches!(
        ErrorColumn::RecordId.value(&error).unwrap(),
        Cow::Borrowed("00123")
    ));
    assert_eq!(ErrorColumn::Line.value(&error).unwrap(), "7");
    assert_eq!(ErrorColumn::Start.value(&error).unwrap(), "2");
    assert_eq!(ErrorColumn::End.value(&error).unwrap(), "6");
    assert_eq!(ErrorColumn::Actual.value(&error).unwrap(), "00,\"é\"\r\nX");
}

#[test]
fn malformed_metadata_is_rejected_before_row_bytes_and_failure_is_terminal() {
    for error in [
        ValidationError {
            line_number: 0,
            ..diagnostic()
        },
        ValidationError {
            code: ValidationCode::RegulatoryRule("bad".into()),
            ..diagnostic()
        },
    ] {
        let mut sink = CsvSink::new(Vec::new()).unwrap();
        assert!(matches!(
            sink.on_error(&error),
            Err(ReportError::InvalidDiagnostic(_))
        ));
        assert_eq!(sink.get_ref(), HEADER.as_bytes());
        assert!(matches!(
            sink.on_error(&diagnostic()),
            Err(ReportError::PreviousFailure)
        ));
        assert!(matches!(sink.finish(), Err(ReportError::PreviousFailure)));
    }
}

struct FailingWriter {
    remaining: usize,
    flush_fails: bool,
    bytes: Vec<u8>,
}
impl Write for FailingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::Error::new(io::ErrorKind::StorageFull, "synthetic"));
        }
        let n = bytes.len().min(self.remaining);
        self.remaining -= n;
        self.bytes.extend_from_slice(&bytes[..n]);
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.flush_fails {
            Err(io::Error::new(
                io::ErrorKind::StorageFull,
                "synthetic flush",
            ))
        } else {
            Ok(())
        }
    }
}

#[test]
fn header_row_terminator_and_flush_failures_remain_technical() {
    let writer = |remaining, flush_fails| FailingWriter {
        remaining,
        flush_fails,
        bytes: vec![],
    };
    assert!(matches!(
        CsvSink::new(writer(0, false)),
        Err(ReportError::Write(_))
    ));
    let mut expected = CsvSink::new(Vec::new()).unwrap();
    expected.on_error(&diagnostic()).unwrap();
    let total = expected.finish().unwrap().len();
    for limit in [HEADER.len(), HEADER.len() + 10, total - 1] {
        let mut sink = CsvSink::new(writer(limit, false)).unwrap();
        assert!(
            matches!(sink.on_error(&diagnostic()), Err(ReportError::Write(ref source)) if source.kind() == io::ErrorKind::StorageFull)
        );
        let before = sink.get_ref().bytes.len();
        assert!(matches!(
            sink.on_error(&diagnostic()),
            Err(ReportError::PreviousFailure)
        ));
        assert_eq!(sink.get_ref().bytes.len(), before);
        assert!(matches!(sink.finish(), Err(ReportError::PreviousFailure)));
    }
    assert!(matches!(
        CsvSink::new(writer(usize::MAX, true)).unwrap().finish(),
        Err(ReportError::Write(_))
    ));
}

#[test]
fn streaming_many_errors_retains_no_history_in_the_sink() {
    let mut sink = CsvSink::new(io::sink()).unwrap();
    let mut error = diagnostic();
    for line in 1..=100_000 {
        error.line_number = line;
        sink.on_error(&error).unwrap();
    }
    sink.finish().unwrap();
}

#[test]
fn summary_only_flushes_and_a_failed_summary_flush_is_terminal() {
    let summary = ValidationSummaryBuilder::new(
        SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        ScopeSummary::Trace {
            trace_id: TraceId("T".into()),
        },
    )
    .finish(RunMetadata {
        validation_mode: ValidationMode::LocalOnly,
        started_at: std::time::UNIX_EPOCH,
        validator_version: "synthetic".into(),
        schema_hash: "synthetic".into(),
        reference_data_versions: Default::default(),
        inputs: vec![],
    })
    .unwrap();
    let mut sink = CsvSink::new(Vec::new()).unwrap();
    sink.on_summary(&summary).unwrap();
    assert_eq!(sink.finish().unwrap(), HEADER.as_bytes());
    let mut failing = CsvSink::new(FailingWriter {
        remaining: usize::MAX,
        flush_fails: true,
        bytes: vec![],
    })
    .unwrap();
    assert!(matches!(
        failing.on_summary(&summary),
        Err(ReportError::Write(_))
    ));
    assert!(matches!(
        failing.on_summary(&summary),
        Err(ReportError::PreviousFailure)
    ));
    assert!(matches!(
        failing.on_error(&diagnostic()),
        Err(ReportError::PreviousFailure)
    ));
}
