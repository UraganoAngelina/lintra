//! Actual application runs: synthetic schema, real input files, actual audit and
//! serializers. No mocked outcomes, artificial Complete digests or normative data.
use lintra::*;
use std::{
    collections::BTreeMap,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
static NEXT: AtomicU64 = AtomicU64::new(0);

/// Each fixture owns its unique directory and only synthetic source bytes.
struct Fixture {
    dir: PathBuf,
    input: PathBuf,
}
impl Fixture {
    fn new(bytes: &[u8]) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "lintra service à {} {stamp} {}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        let input = dir.join("synthetic.txt");
        fs::write(&input, bytes).unwrap();
        Self { dir, input }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).unwrap();
    }
}
fn raw() -> RawFlowSchema {
    serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap()
}
fn service(raw: RawFlowSchema) -> ValidationService {
    let mut repo = InMemorySchemaRepository::new();
    repo.insert(raw).unwrap();
    ValidationService::new(Arc::new(SchemaRegistry::new(
        Arc::new(repo),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    )))
}
fn request(path: &Path) -> ValidationRequest {
    let raw = raw();
    ValidationRequest {
        schema_key: SchemaKey {
            flow_id: FlowId(raw.flow_id),
            version: SchemaVersion(raw.schema_version),
        },
        scope: ValidationScope::Trace(TraceValidationInput {
            trace_id: TraceId(raw.traces[0].id.clone()),
            source: InputSource::Path(path.into()),
        }),
        mode: ValidationMode::LocalOnly,
        options: ValidationOptions {
            record_length_policy: RecordLengthPolicy::StrictSkipRecord,
            newline_policy: NewlinePolicy::AutoLfOrCrlf,
            max_errors: None,
        },
    }
}
/// Capture genuine streaming callbacks and serialize the actual received
/// summary using its authoritative timestamp, not a separately supplied clock.
#[derive(Default)]
struct Capture {
    sink: VecSink,
    json: Vec<u8>,
}
impl ValidationSink for Capture {
    fn on_error(&mut self, error: &ValidationError) -> Result<(), ReportError> {
        self.sink.on_error(error)
    }
    fn on_summary(&mut self, summary: &ValidationSummary) -> Result<(), ReportError> {
        self.json = write_run_summary(Vec::new(), summary)?;
        self.sink.on_summary(summary)
    }
}
#[test]
fn successful_run_reports_local_mode_real_hash_timestamp_and_complete_audit() {
    let file = Fixture::new(b"ABCDEF\r\nUVWXYZ\r\n");
    let raw = raw();
    let hash = schema_fingerprint(&raw).unwrap().to_string();
    let mut capture = Capture::default();
    let mut events = Vec::new();
    let before = SystemTime::now();
    let result = service(raw)
        .validate(
            request(&file.input),
            &mut capture,
            &mut |event| events.push(event),
            &CancellationToken::default(),
        )
        .unwrap();
    assert_eq!(result.status, RunStatus::CompletedValid);
    assert_eq!(result.summary.validation_mode, ValidationMode::LocalOnly);
    assert_eq!(result.summary.schema_hash, hash);
    assert!(result.summary.started_at >= before);
    assert_eq!(
        (
            result.summary.records_processed,
            result.summary.valid_records,
            result.summary.invalid_records
        ),
        (2, 2, 0)
    );
    assert_eq!(
        capture.sink.summaries(),
        std::slice::from_ref(&result.summary)
    );
    assert!(capture.sink.errors().is_empty());
    assert!(matches!(
        result.summary.inputs[0].completion,
        InputAuditCompletion::Complete { .. }
    ));
    assert_eq!(result.summary.inputs[0].file_name, "synthetic.txt");
    let json: serde_json::Value = serde_json::from_slice(&capture.json).unwrap();
    assert_eq!(json["validation_mode"], "local_only");
    assert_eq!(json["result"], "VALID");
    assert_eq!(
        capture.json,
        write_summary(Vec::new(), &result.summary, result.summary.started_at).unwrap()
    );
    assert_eq!(events.first(), Some(&ProgressEvent::RunStarted));
    assert_eq!(events.last(), Some(&ProgressEvent::RunCompleted));
    assert!(!events.contains(&ProgressEvent::PhaseStarted(
        ValidationPhase::CrossRecordValidation
    )));
}
#[test]
fn normative_invalidity_is_ok_completed_invalid_with_streamed_diagnostics() {
    let file = Fixture::new(b"x\nABCDEF\n");
    let mut capture = Capture::default();
    let result = service(raw())
        .validate(
            request(&file.input),
            &mut capture,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    assert_eq!(result.status, RunStatus::CompletedInvalid);
    assert_eq!(result.summary.total_errors, 1);
    assert_eq!(capture.sink.errors()[0].code, ValidationCode::RecordLength);
    assert_eq!(result.summary.invalid_records, 1);
    assert_eq!(result.summary.valid_records, 1);
}
#[test]
fn precancelled_run_reads_no_input_and_serializes_an_incomplete_cancelled_audit() {
    let file = Fixture::new(b"ABCDEF\n");
    let token = CancellationToken::default();
    token.cancel();
    let mut capture = Capture::default();
    let result = service(raw())
        .validate(request(&file.input), &mut capture, &mut |_| {}, &token)
        .unwrap();
    assert_eq!(result.status, RunStatus::Cancelled);
    assert_eq!(result.summary.records_processed, 0);
    assert_eq!(result.summary.inputs[0].bytes_read, 0);
    assert_eq!(
        result.summary.inputs[0].completion,
        InputAuditCompletion::Incomplete
    );
    let json: serde_json::Value = serde_json::from_slice(&capture.json).unwrap();
    assert_eq!(json["result"], "CANCELLED");
    assert_eq!(json["validation_mode"], "local_only");
    assert!(json["inputs"][0]["completion"].get("sha256").is_none());
}
/// Cancelling in a real sink callback leaves a whole classified first record.
struct CancellingSink {
    capture: Capture,
    token: CancellationToken,
}
impl ValidationSink for CancellingSink {
    fn on_error(&mut self, e: &ValidationError) -> Result<(), ReportError> {
        self.capture.on_error(e)?;
        self.token.cancel();
        Ok(())
    }
    fn on_summary(&mut self, s: &ValidationSummary) -> Result<(), ReportError> {
        self.capture.on_summary(s)
    }
}
#[test]
fn cancellation_mid_run_does_not_validate_the_next_record() {
    let file = Fixture::new(b"x\nABCDEF\n");
    let token = CancellationToken::default();
    let mut sink = CancellingSink {
        capture: Capture::default(),
        token: token.clone(),
    };
    let result = service(raw())
        .validate(request(&file.input), &mut sink, &mut |_| {}, &token)
        .unwrap();
    assert_eq!(result.status, RunStatus::Cancelled);
    assert_eq!(result.summary.records_processed, 1);
    assert_eq!(result.summary.invalid_records, 1);
    assert_eq!(result.summary.total_errors, 1);
    assert_eq!(
        result.summary.inputs[0].completion,
        InputAuditCompletion::Incomplete
    );
}
#[test]
fn cancellation_after_eof_keeps_complete_audit_and_cancelled_status() {
    let file = Fixture::new(b"ABCDEF\n");
    let token = CancellationToken::default();
    let mut capture = Capture::default();
    let mut observer = |event| {
        if event == ProgressEvent::PhaseStarted(ValidationPhase::FinalizingReport) {
            token.cancel();
        }
    };
    let result = service(raw())
        .validate(request(&file.input), &mut capture, &mut observer, &token)
        .unwrap();
    assert_eq!(result.status, RunStatus::Cancelled);
    assert_eq!(result.summary.valid_records, 1);
    assert!(matches!(
        result.summary.inputs[0].completion,
        InputAuditCompletion::Complete { .. }
    ));
    assert_eq!(result.summary.inputs[0].bytes_read, 7);
}
#[test]
fn cancellation_after_run_completed_does_not_rewrite_a_finished_outcome() {
    let file = Fixture::new(b"ABCDEF\n");
    let token = CancellationToken::default();
    let mut capture = Capture::default();
    let result = service(raw())
        .validate(
            request(&file.input),
            &mut capture,
            &mut |event| {
                if event == ProgressEvent::RunCompleted {
                    token.cancel();
                }
            },
            &token,
        )
        .unwrap();
    assert!(token.is_cancelled());
    assert_eq!(result.status, RunStatus::CompletedValid);
}
#[test]
fn schema_and_input_failures_never_publish_a_summary() {
    let file = Fixture::new(b"ABCDEF\n");
    let valid = service(raw());
    let mut capture = Capture::default();
    let mut req = request(&file.input);
    req.schema_key.version = SchemaVersion("absent".into());
    assert!(matches!(
        valid.validate(
            req,
            &mut capture,
            &mut |_| {},
            &CancellationToken::default()
        ),
        Err(ValidationRunError::SchemaLoad(
            SchemaLoadError::NotFound { .. }
        ))
    ));
    let mut bad = raw();
    bad.traces[0].record_length = 0;
    assert!(matches!(
        service(bad).validate(
            request(&file.input),
            &mut capture,
            &mut |_| {},
            &CancellationToken::default()
        ),
        Err(ValidationRunError::SchemaCompilation(_))
    ));
    let error = valid
        .validate(
            request(&file.input.with_extension("missing")),
            &mut capture,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap_err();
    assert!(error.source().is_some());
    assert!(matches!(
        error,
        ValidationRunError::Input(InputError::Audit(InputAuditError::Io(_)))
    ));
    assert!(
        capture.sink.errors().is_empty()
            && capture.sink.summaries().is_empty()
            && capture.json.is_empty()
    );
}
#[test]
fn full_and_unsupported_options_fail_before_input_opening() {
    let valid = service(raw());
    let mut req = request(Path::new("absent synthetic input.txt"));
    let mut capture = Capture::default();
    req.mode = ValidationMode::Full;
    assert!(matches!(
        valid.validate(
            req.clone(),
            &mut capture,
            &mut |_| {},
            &CancellationToken::default()
        ),
        Err(ValidationRunError::InvalidRequest(
            InvalidRequestError::UnsupportedMode(ValidationMode::Full)
        ))
    ));
    req.mode = ValidationMode::LocalOnly;
    req.options.max_errors = Some(1);
    assert!(matches!(
        valid.validate(
            req,
            &mut capture,
            &mut |_| {},
            &CancellationToken::default()
        ),
        Err(ValidationRunError::InvalidRequest(
            InvalidRequestError::UnsupportedMaxErrors { .. }
        ))
    ));
    assert!(capture.sink.summaries().is_empty());
}
/// Fail actual writes at either stage, preserving the OS error payload.
struct FailingSink {
    fail_error: bool,
    summary_calls: usize,
}
impl ValidationSink for FailingSink {
    fn on_error(&mut self, _: &ValidationError) -> Result<(), ReportError> {
        if self.fail_error {
            Err(ReportError::Write(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "synthetic",
            )))
        } else {
            Ok(())
        }
    }
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        self.summary_calls += 1;
        Err(ReportError::Write(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "synthetic",
        )))
    }
}
#[test]
fn sink_errors_are_report_errors_and_never_completed_outcomes() {
    for numeric in [false, true] {
        let file = Fixture::new(if numeric { b"ABCDEF\n" } else { b"x\n" });
        let mut value = serde_json::to_value(raw()).unwrap();
        if numeric {
            value["traces"][0]["fields"][0]["physical"]["type"] = serde_json::json!("numeric");
        }
        let valid = service(serde_json::from_value(value).unwrap());
        for fail_error in [true, false] {
            let mut sink = FailingSink {
                fail_error,
                summary_calls: 0,
            };
            let mut events = Vec::new();
            let error = valid
                .validate(
                    request(&file.input),
                    &mut sink,
                    &mut |e| events.push(e),
                    &CancellationToken::default(),
                )
                .unwrap_err();
            assert!(
                matches!(error, ValidationRunError::Report(ReportError::Write(source))
                if source.kind() == io::ErrorKind::PermissionDenied)
            );
            assert_eq!(sink.summary_calls, usize::from(!fail_error));
            assert!(!events.contains(&ProgressEvent::RunCompleted));
        }
    }
}
#[test]
fn mode_wire_values_roundtrip_and_metadata_are_authoritative() {
    for (mode, json) in [
        (ValidationMode::LocalOnly, "\"local_only\""),
        (ValidationMode::Full, "\"full\""),
    ] {
        assert_eq!(serde_json::to_string(&mode).unwrap(), json);
        assert_eq!(serde_json::from_str::<ValidationMode>(json).unwrap(), mode);
    }
    assert!(serde_json::from_str::<ValidationMode>("\"unknown\"").is_err());
    let key = SchemaKey {
        flow_id: FlowId("synthetic".into()),
        version: SchemaVersion("v1".into()),
    };
    let summary = ValidationSummaryBuilder::new(key, ScopeSummary::Flow { trace_ids: vec![] })
        .finish(RunMetadata {
            validation_mode: ValidationMode::LocalOnly,
            validator_version: "synthetic".into(),
            schema_hash: "synthetic".into(),
            reference_data_versions: BTreeMap::new(),
            started_at: UNIX_EPOCH,
            inputs: vec![],
        })
        .unwrap();
    assert_eq!(summary.validation_mode, ValidationMode::LocalOnly);
    assert_eq!(summary.started_at, UNIX_EPOCH);
    let bytes = write_run_summary(Vec::new(), &summary).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["validation_mode"], "local_only");
    assert_eq!(json["started_at"], "1970-01-01T00:00:00Z");
}

/// Source size is captured from an opened handle before TraceStarted. A real
/// truncation then triggers the audited reader's technical EOF-size mismatch,
/// without producing a normative error or a definitive summary.
#[test]
fn source_size_change_is_a_technical_input_failure_without_summary() {
    let file = Fixture::new(b"ABCDEF\n");
    let mut capture = Capture::default();
    let mut progress = |event| {
        if matches!(event, ProgressEvent::TraceStarted { .. }) {
            fs::write(&file.input, b"").unwrap();
        }
    };
    let error = service(raw())
        .validate(
            request(&file.input),
            &mut capture,
            &mut progress,
            &CancellationToken::default(),
        )
        .unwrap_err();
    assert!(matches!(error, ValidationRunError::Input(_)));
    assert!(error.source().is_some());
    assert!(capture.sink.summaries().is_empty() && capture.sink.errors().is_empty());
}

/// The fixed clock is an actual injected dependency, not a timestamp rewritten
/// after the run. Its call counter proves the service samples a single source.
struct FixedClock(std::sync::atomic::AtomicU64);
impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        self.0.fetch_add(1, Ordering::Relaxed);
        UNIX_EPOCH + std::time::Duration::from_secs(123)
    }
}
#[test]
fn injected_clock_drives_the_actual_summary_and_utc_wire_timestamp() {
    let file = Fixture::new(b"ABCDEF\n");
    let mut repo = InMemorySchemaRepository::new();
    repo.insert(raw()).unwrap();
    let registry = Arc::new(SchemaRegistry::new(
        Arc::new(repo),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    ));
    let clock = Arc::new(FixedClock(AtomicU64::new(0)));
    let service = ValidationService::with_clock(registry, clock.clone());
    let mut sink = Capture::default();
    let result = service
        .validate(
            request(&file.input),
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    assert_eq!(
        result.summary.started_at,
        UNIX_EPOCH + std::time::Duration::from_secs(123)
    );
    assert_eq!(clock.0.load(Ordering::Relaxed), 1);
    let value: serde_json::Value = serde_json::from_slice(&sink.json).unwrap();
    assert_eq!(value["started_at"], "1970-01-01T00:02:03Z");
}
