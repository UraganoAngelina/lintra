//! Independent text snapshots for synthetic summary fixtures, no normative data.
use lintra::*;
use std::{
    collections::BTreeMap,
    io::{self, Cursor, Write},
    time::{Duration, UNIX_EPOCH},
};

/// Build consistent report-only counters. Audit completion comes from actual
/// EOF reads over synthetic bytes, never from constructing a fake digest.
fn summary(invalid: bool, cancelled: bool, truncated: bool) -> ValidationSummary {
    let mut builder = ValidationSummaryBuilder::new(
        SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        ScopeSummary::Trace {
            trace_id: TraceId("T1".into()),
        },
    );
    builder.record_processed().unwrap();
    if invalid {
        builder.record_invalid().unwrap();
        // Reverse insertion intentionally: the rendered symbols must be sorted.
        builder.error_emitted(&ValidationCode::InvalidType).unwrap();
        builder
            .error_emitted(&ValidationCode::InvalidDomain)
            .unwrap();
    } else {
        builder.record_valid().unwrap();
    }
    if cancelled {
        builder.cancel();
    }
    if truncated {
        builder.truncate();
    }
    let audits = InputAuditSession::new();
    let mut source = audits
        .wrap(
            TraceId("T1".into()),
            "input.txt".into(),
            7,
            Cursor::new(b"ABCDEF\n"),
        )
        .unwrap();
    io::copy(&mut source, &mut io::sink()).unwrap();
    drop(source);
    let metadata = audits
        .finish(RunMetadata {
            validation_mode: ValidationMode::LocalOnly,
            validator_version: "synthetic".into(),
            schema_hash: "synthetic".into(),
            reference_data_versions: BTreeMap::new(),
            started_at: UNIX_EPOCH,
            inputs: vec![],
        })
        .unwrap();
    let mut summary = builder.finish(metadata).unwrap();
    // Report-only deterministic elapsed fixture, before any serialization.
    summary.elapsed = Duration::from_millis(125);
    summary
}
/// Exercise the public sink, retaining exact output bytes for snapshots.
fn render(summary: &ValidationSummary) -> Vec<u8> {
    let mut sink = HumanReadableSink::new(Vec::new());
    sink.on_summary(summary).unwrap();
    sink.finish().unwrap()
}
const COMMON: &str = "Mode: LocalOnly (local checks)\nFlow: SYNTHETIC\nTrace: T1\nSchema version: v1\nInput: \"input.txt\" (trace T1, 7 bytes)\nRecords processed: 1\n";
#[test]
fn valid_snapshot_explicitly_limits_conformity_to_the_selected_mode() {
    let expected = format!(
        "Result: VALID\n{COMMON}Valid records: 1\nInvalid records: 0\nErrors: 0\nSkipped rules: 0\nTruncated: false\nElapsed: 0.125000000s\n\nNo validation errors found in the selected mode.\n"
    );
    assert_eq!(render(&summary(false, false, false)), expected.as_bytes());
}
#[test]
fn invalid_snapshot_orders_symbolic_counts_without_legacy_numbers() {
    let expected = format!(
        "Result: INVALID\n{COMMON}Valid records: 0\nInvalid records: 1\nErrors: 2\nSkipped rules: 0\nTruncated: false\nElapsed: 0.125000000s\n\nE_INVALID_DOMAIN: 1\nE_INVALID_TYPE: 1\n"
    );
    assert_eq!(render(&summary(true, false, false)), expected.as_bytes());
}
#[test]
fn cancelled_and_truncated_results_never_claim_error_free_completion() {
    for (cancelled, truncated, result, message) in [
        (
            true,
            false,
            "CANCELLED",
            "Validation cancelled; results are incomplete.",
        ),
        (
            false,
            true,
            "VALID",
            "Validation truncated; results are incomplete.",
        ),
    ] {
        let expected = format!(
            "Result: {result}\n{COMMON}Valid records: 1\nInvalid records: 0\nErrors: 0\nSkipped rules: 0\nTruncated: {truncated}\nElapsed: 0.125000000s\n\n{message}\n"
        );
        assert_eq!(
            render(&summary(false, cancelled, truncated)),
            expected.as_bytes()
        );
    }
}
#[test]
fn scope_order_and_basename_escaping_are_preserved() {
    let mut summary = summary(false, false, false);
    summary.scope_summary = ScopeSummary::Flow {
        trace_ids: vec![TraceId("T2".into()), TraceId("T1".into())],
    };
    summary.inputs[0].file_name = "synthetic\nname.txt".into();
    let bytes = render(&summary);
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("Traces: T2 T1\n"));
    assert!(text.contains("Input: \"synthetic\\nname.txt\""));
    assert!(!text.contains("synthetic\nname.txt"));
}
#[test]
fn invalid_summary_is_rejected_before_output_and_sink_stays_failed() {
    let mut summary = summary(false, false, false);
    summary.total_errors = 1;
    let mut sink = HumanReadableSink::new(Vec::new());
    assert!(matches!(
        sink.on_summary(&summary),
        Err(ReportError::InvalidSummary(_))
    ));
    assert!(sink.get_ref().is_empty());
    assert!(matches!(
        sink.on_summary(&summary),
        Err(ReportError::PreviousFailure)
    ));
    assert!(matches!(sink.finish(), Err(ReportError::PreviousFailure)));
}
/// Fail real writes or final flush separately, preserving the I/O payload.
struct FailingWriter {
    fail_flush: bool,
}
impl Write for FailingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.fail_flush {
            Ok(bytes.len())
        } else {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "synthetic"))
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "synthetic"))
    }
}
#[test]
fn write_and_flush_errors_are_typed_and_terminal() {
    for fail_flush in [false, true] {
        let mut sink = HumanReadableSink::new(FailingWriter { fail_flush });
        assert!(
            matches!(sink.on_summary(&summary(false,false,false)),Err(ReportError::Write(ref error)) if error.kind()==io::ErrorKind::BrokenPipe)
        );
        assert!(matches!(
            sink.on_summary(&summary(false, false, false)),
            Err(ReportError::PreviousFailure)
        ));
    }
}

/// The text sink is summary-only even when used independently of JSONL: a real
/// diagnostic must not write or retain record data through this adapter.
#[test]
fn individual_diagnostics_are_acknowledged_without_text_output() {
    let flow = FlowId("SYNTHETIC".into());
    let version = SchemaVersion("v1".into());
    let trace = TraceId("T1".into());
    let schema: RawFlowSchema =
        serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
    let mut repository = InMemorySchemaRepository::new();
    repository.insert(schema).unwrap();
    let registry = SchemaRegistry::new(
        std::sync::Arc::new(repository),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    );
    let compiled = registry
        .resolve(&SchemaKey {
            flow_id: flow.clone(),
            version: version.clone(),
        })
        .unwrap();
    let diagnostic = ErrorFactory {
        flow_id: &flow,
        schema_version: &version,
        trace_id: &trace,
        encoding: compiled.encoding(),
    }
    .record_length_error(1, None, 6, 1);
    let mut sink = HumanReadableSink::new(Vec::new());
    sink.on_error(&diagnostic).unwrap();
    assert!(sink.get_ref().is_empty());
    let mut inconsistent = summary(false, false, false);
    inconsistent.total_errors = 1;
    assert!(sink.on_summary(&inconsistent).is_err());
    assert!(matches!(
        sink.on_error(&diagnostic),
        Err(ReportError::PreviousFailure)
    ));
}
