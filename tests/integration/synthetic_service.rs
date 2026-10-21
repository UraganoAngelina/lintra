//! Synthetic format-1 application runs; these fixtures express no regional norm.
use lintra::*;
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
static NEXT: AtomicU64 = AtomicU64::new(0);

/// Own the real source and repository tree for one test without global state.
struct Fixture {
    root: PathBuf,
    input: PathBuf,
}
impl Fixture {
    /// Use the same filesystem adapter/layout as a consumer; all bytes are synthetic.
    fn new(bytes: &[u8]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "lintra e2e {} {} {}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let dir = root.join("SYNTHETIC/v1");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("flow.json"),
            include_bytes!("../fixtures/application/schema.json"),
        )
        .unwrap();
        let input = root.join("input.txt");
        fs::write(&input, bytes).unwrap();
        Self { root, input }
    }
    /// The request goes through service preflight, never through a test-run engine loop.
    fn request(&self) -> ValidationRequest {
        ValidationRequest {
            schema_key: SchemaKey {
                flow_id: FlowId("SYNTHETIC".into()),
                version: SchemaVersion("v1".into()),
            },
            scope: ValidationScope::Trace(TraceValidationInput {
                trace_id: TraceId("T1".into()),
                source: InputSource::Path(self.input.clone()),
            }),
            mode: ValidationMode::LocalOnly,
            options: ValidationOptions {
                record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                newline_policy: NewlinePolicy::AutoLfOrCrlf,
                max_errors: None,
            },
        }
    }
    /// Exercise the real repository, registry and application, injecting only time.
    fn service(&self) -> ValidationService {
        let repository = FileSystemSchemaRepository::new(self.root.clone()).unwrap();
        let registry = SchemaRegistry::new(
            Arc::new(repository),
            SchemaCompiler,
            RawResolvedEncoding::Utf8,
        );
        ValidationService::with_clock(Arc::new(registry), Arc::new(FixedClock))
    }
}
impl Drop for Fixture {
    /// Handles close before cleanup, allowing the same lifecycle on Windows.
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
/// Fix only the supplied clock, without rewriting any resulting report/model.
struct FixedClock;
impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH
    }
}

/// Stream through production encoders and preserve their exact bytes.
struct Reports {
    errors: JsonlSink<Vec<u8>>,
    summary: Vec<u8>,
}
impl Reports {
    /// Small fixture reports can be held in test memory; production remains streaming.
    fn new() -> Self {
        Self {
            errors: JsonlSink::new(Vec::new()),
            summary: Vec::new(),
        }
    }
}
impl ValidationSink for Reports {
    /// Delegation exercises real validation and serialization of each diagnostic.
    fn on_error(&mut self, error: &ValidationError) -> Result<(), ReportError> {
        self.errors.on_error(error)
    }
    /// Serialize the authoritative summary only when the service actually emits it.
    fn on_summary(&mut self, summary: &ValidationSummary) -> Result<(), ReportError> {
        self.errors.on_summary(summary)?;
        self.summary = write_run_summary(Vec::new(), summary)?;
        Ok(())
    }
}

/// Left zero stripping, right blank stripping, UTF-8 byte widths, required fields
/// and allowed values must work together through application orchestration.
#[test]
fn valid_local_file_reaches_real_jsonl_and_summary_without_trimming_data() {
    let file = Fixture::new("001A  \r\n001É \r\n001A  ".as_bytes());
    let mut reports = Reports::new();
    let outcome = file
        .service()
        .validate(
            file.request(),
            &mut reports,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    assert_eq!(outcome.status, RunStatus::CompletedValid);
    assert_eq!(
        (
            outcome.summary.records_processed,
            outcome.summary.valid_records
        ),
        (3, 3)
    );
    assert!(reports.errors.finish().unwrap().is_empty());
    let summary: serde_json::Value = serde_json::from_slice(&reports.summary).unwrap();
    assert_eq!(summary["validation_mode"], "local_only");
    assert_eq!(summary["result"], "VALID");
    assert_eq!(summary["started_at"], "1970-01-01T00:00:00Z");
    assert_eq!(summary["inputs"][0]["bytes_read"], 22);
    assert_eq!(summary["inputs"][0]["completion"]["kind"], "complete");
    assert_eq!(summary["inputs"][0]["file_name"], "input.txt");
}

/// Repeat the actual run: JSONL is byte-identical without sorting or masking.
/// Only malformed record lengths bypass field checks; all other stages execute.
#[test]
fn invalid_local_file_preserves_field_stage_order_and_exact_diagnostic_bytes() {
    let file = Fixture::new(b"001A  \r\n0x1A  \r\n002A  \r\n001B  \r\n      \r\nxx\r\n001A   \r\n");
    let mut previous = None;
    for _ in 0..2 {
        let mut reports = Reports::new();
        let result = file
            .service()
            .validate(
                file.request(),
                &mut reports,
                &mut |_| {},
                &CancellationToken::default(),
            )
            .unwrap();
        assert_eq!(result.status, RunStatus::CompletedInvalid);
        assert_eq!(
            (
                result.summary.records_processed,
                result.summary.valid_records,
                result.summary.invalid_records,
                result.summary.total_errors
            ),
            (7, 1, 6, 7)
        );
        let bytes = reports.errors.finish().unwrap();
        assert!(!bytes.contains(&b'\r') && !bytes.starts_with(&[0xef, 0xbb, 0xbf]));
        let lines: Vec<serde_json::Value> = bytes
            .split(|b| *b == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice(line).unwrap())
            .collect();
        let observed: Vec<_> = lines
            .iter()
            .map(|line| {
                (
                    line["line"].as_u64().unwrap(),
                    line["code"].as_str().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            observed,
            [
                (2, "E_INVALID_TYPE"),
                (3, "E_INVALID_DOMAIN"),
                (4, "E_INVALID_DOMAIN"),
                (5, "E_REQUIRED_FIELD"),
                (5, "E_REQUIRED_FIELD"),
                (6, "E_RECORD_LENGTH"),
                (7, "E_RECORD_LENGTH")
            ]
        );
        // Two required violations on one record must keep physical field order.
        assert_eq!(lines[3]["field"]["id"], "F1");
        assert_eq!(lines[4]["field"]["id"], "F2");
        let summary: serde_json::Value = serde_json::from_slice(&reports.summary).unwrap();
        assert_eq!(summary["result"], "INVALID");
        assert_eq!(summary["errors_by_code"]["E_RECORD_LENGTH"], 2);
        if let Some(previous) = &previous {
            assert_eq!(&bytes, previous);
        }
        previous = Some(bytes);
    }
}

/// A real invalid schema on disk is a technical compilation failure, not an
/// invalid input record. No callbacks may publish a definitive report.
#[test]
fn schema_gap_fails_before_reading_input_or_emitting_reports() {
    let file = Fixture::new(b"001A  \n");
    let path = file.root.join("SYNTHETIC/v1/flow.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["traces"][0]["fields"][1]["physical"]["start"] = serde_json::json!(5);
    fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    fs::remove_file(&file.input).unwrap();
    let mut reports = Reports::new();
    let mut events = Vec::new();
    let result = file.service().validate(
        file.request(),
        &mut reports,
        &mut |event| events.push(event),
        &CancellationToken::default(),
    );
    assert!(matches!(
        result,
        Err(ValidationRunError::SchemaCompilation(_))
    ));
    assert!(reports.summary.is_empty() && reports.errors.finish().unwrap().is_empty());
    assert!(events.is_empty());
}

/// Discard serialized diagnostics while retaining only counters, demonstrating
/// an application consumer that never accumulates a global error collection.
struct CountingReports {
    jsonl: JsonlSink<std::io::Sink>,
    errors: u64,
    summaries: u64,
}
impl CountingReports {
    fn new() -> Self {
        Self {
            jsonl: JsonlSink::new(std::io::sink()),
            errors: 0,
            summaries: 0,
        }
    }
}
impl ValidationSink for CountingReports {
    /// Count only after the actual JSONL writer accepts the diagnostic.
    fn on_error(&mut self, error: &ValidationError) -> Result<(), ReportError> {
        self.jsonl.on_error(error)?;
        self.errors += 1;
        Ok(())
    }
    /// Count only a real summary callback, allowing failure tests to forbid it.
    fn on_summary(&mut self, summary: &ValidationSummary) -> Result<(), ReportError> {
        self.jsonl.on_summary(summary)?;
        self.summaries += 1;
        Ok(())
    }
}

/// Write the large source incrementally; neither fixture nor sink retains its
/// contents. Mutation at the first batch checkpoint (1024 records) injects a real EOF-size
/// audit failure after processing has started, with OS buffering left intact.
#[test]
fn large_stream_and_mid_run_input_failure_preserve_application_boundaries() {
    use std::io::Write;
    for truncate in [false, true] {
        let file = Fixture::new(b"");
        let mut writer = std::io::BufWriter::new(fs::File::create(&file.input).unwrap());
        for _ in 0..10_000 {
            writer.write_all(b"x\n").unwrap();
        }
        writer.flush().unwrap();
        drop(writer);
        let mut reports = CountingReports::new();
        let mut progress_events = 0;
        let mut observer = |event| {
            progress_events += 1;
            if truncate && matches!(event, ProgressEvent::RecordsProcessed { count: 1024, .. }) {
                fs::write(&file.input, b"").unwrap();
            }
        };
        let result = file.service().validate(
            file.request(),
            &mut reports,
            &mut observer,
            &CancellationToken::default(),
        );
        if truncate {
            let Err(ValidationRunError::Input(InputError::RecordRead {
                last_completed_line,
                source,
            })) = result
            else {
                panic!("expected a typed record-read failure, got {result:?}");
            };
            assert!(last_completed_line >= 1024);
            assert!(matches!(
                source
                    .get_ref()
                    .and_then(|cause| cause.downcast_ref::<InputAuditError>()),
                Some(InputAuditError::SizeMismatch { .. })
            ));
            assert!(reports.errors >= 1024 && reports.errors < 10_000);
            assert_eq!(reports.summaries, 0);
        } else {
            let result = result.unwrap();
            assert_eq!(result.status, RunStatus::CompletedInvalid);
            assert_eq!(result.summary.records_processed, 10_000);
            assert_eq!(reports.errors, 10_000);
            assert_eq!(reports.summaries, 1);
            assert!(progress_events < 30);
        }
    }
}
