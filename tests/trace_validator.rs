//! Trace loop tests execute the real reader, local stages and input auditing.
use lintra::*;
use std::{
    collections::BTreeMap,
    fs,
    io::{self, BufRead, BufReader, Cursor, Read},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

fn schema() -> CompiledFlowSchema {
    let mut raw: RawFlowSchema =
        serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw).unwrap()
}
fn builder(s: &CompiledFlowSchema) -> ValidationSummaryBuilder {
    ValidationSummaryBuilder::new(
        s.key().clone(),
        ScopeSummary::Trace {
            trace_id: s.traces()[0].id().clone(),
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

/// Mixed CRLF/LF and EOF framing exercise actual counts and final notifications.
#[test]
fn reader_streams_records_without_double_counting() {
    let s = schema();
    let mut b = builder(&s);
    let mut sink = VecSink::default();
    let mut events = Vec::new();
    let audits = InputAuditSession::new();
    let bytes = b"ABCDEF\r\nx\nUVWXYZ";
    let reader = audits
        .wrap(
            s.traces()[0].id().clone(),
            "synthetic.txt".into(),
            bytes.len() as u64,
            Cursor::new(bytes),
        )
        .unwrap();
    let result = validate_reader(
        BufReader::new(reader),
        &s,
        s.traces()[0].id(),
        NewlinePolicy::AutoLfOrCrlf,
        &mut sink,
        &mut b,
        &mut TraceControl {
            is_cancelled: &|| false,
            on_event: &mut |event| events.push(event),
        },
    )
    .unwrap();
    assert_eq!(result, TraceCompletion::Completed { count: 3 });
    assert_eq!(sink.errors().len(), 1);
    assert_eq!(sink.errors()[0].line_number, 2);
    let summary = b.finish(audits.finish(metadata()).unwrap()).unwrap();
    assert_eq!(
        (
            summary.records_processed,
            summary.valid_records,
            summary.invalid_records
        ),
        (3, 2, 1)
    );
    assert!(matches!(
        summary.inputs[0].completion,
        InputAuditCompletion::Complete { .. }
    ));
    assert!(matches!(
        events.last(),
        Some(TraceEvent::Completed { count: 3, .. })
    ));
}

/// Any read after a known cancellation would panic, so the test proves that the
/// checkpoint precedes even BOM preflight and does not just construct an outcome.
struct Unreadable;
impl Read for Unreadable {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        panic!("unexpected read")
    }
}
impl BufRead for Unreadable {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        panic!("unexpected fill")
    }
    fn consume(&mut self, _: usize) {
        panic!("unexpected consume")
    }
}
#[test]
fn cancellation_before_input_does_not_read_any_bytes() {
    let s = schema();
    let mut b = builder(&s);
    let mut sink = VecSink::default();
    let token = CancellationToken::default();
    token.cancel();
    let result = validate_reader(
        Unreadable,
        &s,
        s.traces()[0].id(),
        NewlinePolicy::AutoLfOrCrlf,
        &mut sink,
        &mut b,
        &mut TraceControl {
            is_cancelled: &|| token.is_cancelled(),
            on_event: &mut |_| {},
        },
    )
    .unwrap();
    assert_eq!(result, TraceCompletion::Cancelled { count: 0 });
    assert_eq!(
        b.finish(metadata()).unwrap().run_status,
        RunStatus::Cancelled
    );
    assert!(sink.errors().is_empty());
}

/// Cancellation occurs inside an actual emission. The acquired record finishes
/// classification, and the next record is not validated/read to finish the hash.
struct CancelSink {
    token: CancellationToken,
    errors: VecSink,
}
impl ValidationSink for CancelSink {
    fn on_error(&mut self, error: &ValidationError) -> Result<(), ReportError> {
        self.errors.on_error(error)?;
        self.token.cancel();
        Ok(())
    }
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        panic!("loop must not publish summary")
    }
}
#[test]
fn cancellation_between_records_preserves_partial_audit_and_counts() {
    let s = schema();
    let mut b = builder(&s);
    let token = CancellationToken::default();
    let mut sink = CancelSink {
        token: token.clone(),
        errors: VecSink::default(),
    };
    let audits = InputAuditSession::new();
    let bytes = b"x\nABCDEF\n";
    let reader = audits
        .wrap(
            s.traces()[0].id().clone(),
            "synthetic.txt".into(),
            bytes.len() as u64,
            Cursor::new(bytes),
        )
        .unwrap();
    let result = validate_reader(
        BufReader::new(reader),
        &s,
        s.traces()[0].id(),
        NewlinePolicy::AutoLfOrCrlf,
        &mut sink,
        &mut b,
        &mut TraceControl {
            is_cancelled: &|| token.is_cancelled(),
            on_event: &mut |_| {},
        },
    )
    .unwrap();
    assert_eq!(result, TraceCompletion::Cancelled { count: 1 });
    assert_eq!(sink.errors.errors().len(), 1);
    let summary = b.finish(audits.finish(metadata()).unwrap()).unwrap();
    assert_eq!((summary.records_processed, summary.invalid_records), (1, 1));
    assert_eq!(summary.run_status, RunStatus::Cancelled);
    assert_eq!(
        summary.inputs[0].completion,
        InputAuditCompletion::Incomplete
    );
    assert_eq!(summary.inputs[0].bytes_read, bytes.len() as u64); // prefetched, but no EOF observed
}

/// Real Read failure starts after the first complete source line, not by
/// manually constructing InputError. BufRead delegates to the same boundary.
struct Faulty(Cursor<&'static [u8]>);
impl Read for Faulty {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.0.position() == self.0.get_ref().len() as u64 {
            Err(io::Error::other("synthetic I/O failure"))
        } else {
            self.0.read(out)
        }
    }
}
impl BufRead for Faulty {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.0.position() == self.0.get_ref().len() as u64 {
            Err(io::Error::other("synthetic I/O failure"))
        } else {
            self.0.fill_buf()
        }
    }
    fn consume(&mut self, n: usize) {
        self.0.consume(n);
    }
}
#[test]
fn read_failure_keeps_line_and_is_terminal() {
    let s = schema();
    let mut b = builder(&s);
    let mut sink = VecSink::default();
    let mut validator =
        TraceValidator::new(&s, s.traces()[0].id(), NewlinePolicy::AutoLfOrCrlf).unwrap();
    let mut callback = |_| {};
    let mut control = TraceControl {
        is_cancelled: &|| false,
        on_event: &mut callback,
    };
    let error = validator
        .validate_reader(
            Faulty(Cursor::new(b"ABCDEF\n")),
            &mut sink,
            &mut b,
            &mut control,
        )
        .unwrap_err();
    assert!(matches!(
        error,
        TraceValidationError::Input(InputError::RecordRead {
            last_completed_line: 1,
            ..
        })
    ));
    assert!(sink.errors().is_empty());
    assert!(matches!(
        validator.validate_reader(Unreadable, &mut sink, &mut b, &mut control),
        Err(TraceValidationError::PreviousFailure)
    ));
}
#[test]
fn bom_is_technical_and_never_a_record_length_violation() {
    let s = schema();
    let mut b = builder(&s);
    let mut sink = VecSink::default();
    let error = validate_reader(
        Cursor::new(b"\xef\xbb\xbfABCDEF\n"),
        &s,
        s.traces()[0].id(),
        NewlinePolicy::AutoLfOrCrlf,
        &mut sink,
        &mut b,
        &mut TraceControl {
            is_cancelled: &|| false,
            on_event: &mut |_| {},
        },
    )
    .unwrap_err();
    assert!(matches!(
        error,
        TraceValidationError::Input(InputError::BomRejected(BomKind::Utf8))
    ));
    assert!(sink.errors().is_empty());
    assert_eq!(b.finish(metadata()).unwrap().records_processed, 0);
}

/// Generated input retains no file; sink retains only a count. Batch events
/// stay bounded by checkpoints rather than allocating one event per record.
struct CountSink(u64);
impl ValidationSink for CountSink {
    fn on_error(&mut self, _: &ValidationError) -> Result<(), ReportError> {
        self.0 += 1;
        Ok(())
    }
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        panic!("no summary publication")
    }
}
#[test]
fn large_generated_stream_uses_counting_sink_and_batched_progress() {
    let s = schema();
    let mut b = builder(&s);
    let mut sink = CountSink(0);
    let mut events = 0;
    let result = validate_reader(
        BufReader::new(io::repeat(b'\n').take(100_000)),
        &s,
        s.traces()[0].id(),
        NewlinePolicy::AutoLfOrCrlf,
        &mut sink,
        &mut b,
        &mut TraceControl {
            is_cancelled: &|| false,
            on_event: &mut |_| events += 1,
        },
    )
    .unwrap();
    assert_eq!(result, TraceCompletion::Completed { count: 100_000 });
    assert_eq!(sink.0, 100_000);
    assert!(events < 110);
}

static NEXT: AtomicU64 = AtomicU64::new(0);
/// Owned synthetic temporary file, removed only after readers have closed.
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "lintra trace à {} {stamp} {}.txt",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&path, b"ABCDEF\r\nUVWXYZ\r\n").unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_file(&self.0).unwrap();
    }
}
#[test]
fn path_execution_produces_real_complete_audit_with_basename_only() {
    let file = Fixture::new();
    let s = schema();
    let mut b = builder(&s);
    let mut sink = VecSink::default();
    let audits = InputAuditSession::new();
    let result = TraceValidator::new(&s, s.traces()[0].id(), NewlinePolicy::AutoLfOrCrlf)
        .unwrap()
        .validate_path(
            &file.0,
            &mut sink,
            &mut b,
            &mut TraceControl {
                is_cancelled: &|| false,
                on_event: &mut |_| {},
            },
            &audits,
        )
        .unwrap();
    assert_eq!(result, TraceCompletion::Completed { count: 2 });
    let summary = b.finish(audits.finish(metadata()).unwrap()).unwrap();
    assert_eq!(
        summary.inputs[0].file_name,
        file.0.file_name().unwrap().to_str().unwrap()
    );
    assert!(matches!(
        summary.inputs[0].completion,
        InputAuditCompletion::Complete { .. }
    ));
    assert_eq!(summary.inputs[0].bytes_read, 16);
}

/// Preserve a genuine sink write failure through the record/trace layers;
/// the loop must become terminal and never publish a summary.
struct FailedSink;
impl ValidationSink for FailedSink {
    fn on_error(&mut self, _: &ValidationError) -> Result<(), ReportError> {
        Err(ReportError::Write(io::Error::other("synthetic sink")))
    }
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        panic!("no summary after failure")
    }
}
#[test]
fn sink_failure_keeps_payload_and_stops_trace() {
    let s = schema();
    let mut b = builder(&s);
    let mut sink = FailedSink;
    let mut validator =
        TraceValidator::new(&s, s.traces()[0].id(), NewlinePolicy::AutoLfOrCrlf).unwrap();
    let mut callback = |_| {};
    let mut control = TraceControl {
        is_cancelled: &|| false,
        on_event: &mut callback,
    };
    assert!(matches!(
        validator.validate_reader(Cursor::new(b"x\nABCDEF\n"), &mut sink, &mut b, &mut control),
        Err(TraceValidationError::Record(RecordValidationError::Report(
            ReportError::Write(_)
        )))
    ));
    assert!(matches!(
        validator.validate_reader(Unreadable, &mut sink, &mut b, &mut control),
        Err(TraceValidationError::PreviousFailure)
    ));
}

/// A real path opening error is technical and retains the authoritative audit
/// failure. The validator must not continue with a replacement source.
#[test]
fn path_open_failure_is_typed_and_terminal() {
    let file = Fixture::new();
    let s = schema();
    let mut b = builder(&s);
    let mut sink = VecSink::default();
    let mut validator =
        TraceValidator::new(&s, s.traces()[0].id(), NewlinePolicy::AutoLfOrCrlf).unwrap();
    let mut callback = |_| {};
    let mut control = TraceControl {
        is_cancelled: &|| false,
        on_event: &mut callback,
    };
    assert!(matches!(
        validator.validate_path(
            &file.0.with_extension("missing"),
            &mut sink,
            &mut b,
            &mut control,
            &InputAuditSession::new()
        ),
        Err(TraceValidationError::Input(InputError::Audit(
            InputAuditError::Io(_)
        )))
    ));
    assert!(matches!(
        validator.validate_reader(Unreadable, &mut sink, &mut b, &mut control),
        Err(TraceValidationError::PreviousFailure)
    ));
}
