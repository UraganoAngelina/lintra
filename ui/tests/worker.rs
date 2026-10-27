//! Real ValidationService and synthetic bytes on an owned dedicated thread.
#![cfg(feature = "embedded-bundle")]
use lintra::*;
use lintra_ui::{analysis::*, worker::*};
use std::{
    fs,
    sync::{Arc, Mutex},
    thread,
    time::SystemTime,
};

fn prepare(bytes: &[u8]) -> (tempfile::TempDir, PreparedAnalysis) {
    let root = tempfile::tempdir().unwrap();
    let input = root.path().join("input.txt");
    fs::write(&input, bytes).unwrap();
    let selection = AnalysisSelection {
        flow: Some("SYNTHETIC".into()),
        version: Some("v1".into()),
        trace: Some("T1".into()),
        encoding: Some(RawResolvedEncoding::Utf8),
        file: Some(input),
        reports: ReportSelection::default(),
    };
    let prepared = selection
        .prepare(&AnalysisBundle::builtin().unwrap())
        .unwrap();
    (root, prepared)
}
#[test]
fn completed_runs_stream_real_jsonl_and_keep_normative_invalidity_distinct() {
    for (bytes, status, errors) in [
        (b"001A  \r\n".as_slice(), RunStatus::CompletedValid, 0),
        (b"001Z  \n".as_slice(), RunStatus::CompletedInvalid, 1),
    ] {
        let (root, prepared) = prepare(bytes);
        let completed =
            WorkerHandle::start(prepared, CancellationToken::default(), Arc::new(|| {}))
                .unwrap()
                .wait()
                .unwrap();
        assert_eq!(completed.requested_reports, ReportSelection::default());
        assert_eq!(completed.outcome.status, status);
        assert_eq!(completed.outcome.summary.total_errors, errors);
        assert_eq!(completed.outcome.summary.records_processed, 1);
        let report = completed.report.unwrap();
        let path = report.jsonl_path();
        let raw = fs::read(&path).unwrap();
        assert_eq!(
            raw.split(|b| *b == b'\n')
                .filter(|line| !line.is_empty())
                .count(),
            errors as usize
        );
        if errors == 1 {
            let diagnostic: serde_json::Value = serde_json::from_slice(&raw).unwrap();
            assert!(diagnostic["code"].as_str().unwrap().starts_with("E_"));
            assert!(raw.ends_with(b"\n"));
            assert!(!raw.contains(&b'\r'));
        }
        assert!(!path.with_extension("jsonl.part").exists());
        assert_eq!(report.audit().source(), "embedded");
        assert_eq!(fs::read(root.path().join("input.txt")).unwrap(), bytes);
        drop(report);
        assert!(
            !path.exists(),
            "spool lives only while owned by its consumer"
        );
    }
}
#[test]
fn cancellation_has_a_real_cancelled_outcome_and_no_published_report() {
    let (_root, prepared) = prepare(b"001A  \n");
    let token = CancellationToken::default();
    token.cancel();
    let completed = WorkerHandle::start(prepared, token, Arc::new(|| {}))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(completed.outcome.status, RunStatus::Cancelled);
    assert!(completed.report.is_none());
}
#[test]
fn input_failures_after_configuration_remain_technical_and_do_not_become_cancelled() {
    for bom in [false, true] {
        let (root, prepared) = prepare(b"001A  \n");
        let input = root.path().join("input.txt");
        if bom {
            fs::write(&input, b"\xef\xbb\xbf001A  \n").unwrap();
        } else {
            fs::remove_file(&input).unwrap();
        }
        let result = WorkerHandle::start(prepared, CancellationToken::default(), Arc::new(|| {}))
            .unwrap()
            .wait();
        assert!(matches!(result, Err(WorkerError::Validation(_))));
    }
}
struct RecordingClock(Arc<Mutex<Option<thread::ThreadId>>>);
impl Clock for RecordingClock {
    fn now(&self) -> SystemTime {
        *self.0.lock().unwrap() = Some(thread::current().id());
        SystemTime::UNIX_EPOCH
    }
}
#[test]
fn actual_service_runs_on_the_worker_without_a_ui_progress_consumer() {
    let (_root, mut prepared) = prepare(b"001A  \n");
    let observed = Arc::new(Mutex::new(None));
    let bundle = EmbeddedBundle::builtin().unwrap();
    let registry = Arc::new(SchemaRegistry::new(
        Arc::new(EmbeddedSchemaRepository::new(bundle.clone()).unwrap()),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    ));
    prepared.service = Arc::new(
        ValidationService::with_clock(registry, Arc::new(RecordingClock(observed.clone())))
            .with_reference_data(
                Arc::new(EmbeddedReferenceDataPreparer::new(bundle)),
                DatasetPreparationPolicy::ContinueWithAvailableEvidence,
            ),
    );
    let caller = thread::current().id();
    // No progress receiver is drained. A one-slot backlog cannot deadlock the
    // service or prevent delivery of the separate terminal message.
    let completed = WorkerHandle::start(prepared, CancellationToken::default(), Arc::new(|| {}))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(completed.outcome.status, RunStatus::CompletedValid);
    assert_ne!(observed.lock().unwrap().unwrap(), caller);
}
