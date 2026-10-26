//! CNF-AUDIT-01 acceptance: fingerprints are produced by real reads, never forged Complete states.
use lintra::{
    BomPolicy, DatasetId, DatasetVersion, FlowId, InputAuditCompletion, InputAuditError,
    InputAuditSession, InputError, NewlinePolicy, RecordReader, RunMetadata, RunStatus, SchemaKey,
    SchemaVersion, ScopeSummary, SummaryBuildError, TraceId, ValidationCode, ValidationSummary,
    ValidationSummaryBuilder, preflight_bom, write_summary,
};
use std::{
    cell::Cell,
    collections::BTreeMap,
    error::Error,
    fs::{self, OpenOptions},
    io::{self, BufRead, BufReader, Cursor, Read, Write},
    path::PathBuf,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const ORIGINAL: &[u8] = b"ABC  \r\n";
const RAW_BOM: &[u8] = b"\xef\xbb\xbfABC  \r\nDEF  \r\n";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct TempDir(PathBuf);
impl TempDir {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        loop {
            let path = std::env::temp_dir().join(format!(
                "lintra audit à {} {stamp} {}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("cannot create synthetic directory: {e}"),
            }
        }
    }
    fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        file.write_all(bytes).unwrap();
        path
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn metadata() -> RunMetadata {
    RunMetadata {
        validation_mode: lintra::ValidationMode::LocalOnly,
        validator_version: "synthetic-build".into(),
        schema_hash: "1".repeat(64),
        reference_data_versions: BTreeMap::from([
            (DatasetId("Z".into()), DatasetVersion("v2".into())),
            (DatasetId("A".into()), DatasetVersion("v1".into())),
        ]),
        started_at: UNIX_EPOCH,
        inputs: vec![],
    }
}
fn builder() -> ValidationSummaryBuilder {
    ValidationSummaryBuilder::new(
        SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        ScopeSummary::Trace {
            trace_id: TraceId("T1".into()),
        },
    )
}
fn read_file_metadata() -> RunMetadata {
    let dir = TempDir::new();
    let path = dir.file("synthetic-T1.txt", ORIGINAL);
    let session = InputAuditSession::new();
    let mut reader = session.open(TraceId("T1".into()), &path).unwrap();
    reader.read_to_end(&mut Vec::new()).unwrap();
    session.finish(metadata()).unwrap()
}
fn json(summary: &ValidationSummary, at: SystemTime) -> serde_json::Value {
    serde_json::from_slice(&write_summary(Vec::new(), summary, at).unwrap()).unwrap()
}

#[test]
fn whole_real_file_matches_approved_complete_fixture_and_independent_sha256() {
    let meta = read_file_metadata();
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("../docs/proposals/input-audit/complete.json")).unwrap();
    let mut b = builder();
    b.record_processed().unwrap();
    b.record_valid().unwrap();
    let at = meta.started_at;
    let mut summary = b.finish(meta).unwrap();
    assert_eq!(summary.run_status, RunStatus::CompletedValid);
    assert_eq!(json(&summary, at)["inputs"][0], expected);
    summary.elapsed = Duration::from_millis(12); // fixed only for serialization regression, not the input audit
    assert_eq!(
        write_summary(Vec::new(), &summary, at).unwrap(),
        include_bytes!("fixtures/summary.complete.json")
    );
}

#[test]
fn bom_spaces_and_crlf_are_hashed_before_preflight_and_record_framing() {
    let dir = TempDir::new();
    let path = dir.file("raw-source.txt", RAW_BOM);
    let session = InputAuditSession::new();
    let audited = session.open(TraceId("T1".into()), &path).unwrap();
    let checked = preflight_bom(audited, BomPolicy::StripKnownBom).unwrap();
    let mut reader = RecordReader::new(
        BufReader::with_capacity(2, checked),
        NewlinePolicy::AutoLfOrCrlf,
    );
    let mut buffer = Vec::new();
    assert_eq!(
        reader.read_next(&mut buffer).unwrap().unwrap().bytes,
        b"ABC  "
    );
    assert_eq!(
        reader.read_next(&mut buffer).unwrap().unwrap().bytes,
        b"DEF  "
    );
    assert!(reader.read_next(&mut buffer).unwrap().is_none());
    let meta = session.finish(metadata()).unwrap();
    let InputAuditCompletion::Complete { sha256 } = &meta.inputs[0].completion else {
        panic!("missing full digest")
    };
    assert_eq!(
        sha256.to_string(),
        include_str!("fixtures/audit-raw.sha256").trim_end()
    );
    assert_eq!(meta.inputs[0].bytes_read, RAW_BOM.len() as u64);
}

struct Counted<R> {
    inner: R,
    calls: Rc<Cell<usize>>,
}
impl<R: Read> Read for Counted<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        self.calls.set(self.calls.get() + 1);
        self.inner.read(out)
    }
}
#[test]
fn cancellation_before_eof_matches_approved_incomplete_fixture_without_an_extra_read() {
    let calls = Rc::new(Cell::new(0));
    let session = InputAuditSession::new();
    let mut reader = session
        .wrap(
            TraceId("T1".into()),
            "synthetic-T1.txt".into(),
            7,
            Counted {
                inner: Cursor::new(ORIGINAL),
                calls: calls.clone(),
            },
        )
        .unwrap();
    reader.read_exact(&mut [0; 3]).unwrap();
    let before = calls.get();
    let mut b = builder();
    b.cancel();
    let meta = session.finish(metadata()).unwrap();
    assert_eq!(calls.get(), before);
    let mut summary = b.finish(meta).unwrap();
    assert_eq!(summary.run_status, RunStatus::Cancelled);
    summary.elapsed = Duration::from_millis(12);
    let output = json(&summary, UNIX_EPOCH);
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../docs/proposals/input-audit/cancelled-before-eof.json"
    ))
    .unwrap();
    assert_eq!(output["inputs"][0], expected);
    assert!(output["inputs"][0]["completion"].get("sha256").is_none());
    assert_eq!(
        write_summary(Vec::new(), &summary, UNIX_EPOCH).unwrap(),
        include_bytes!("fixtures/summary.cancelled.json")
    );
    assert_eq!(calls.get(), before);
    assert!(reader.read(&mut [0]).is_err());
    assert_eq!(calls.get(), before);
}

#[test]
fn exact_size_and_empty_buffer_do_not_substitute_for_observed_eof() {
    let dir = TempDir::new();
    let path = dir.file("synthetic-T1.txt", ORIGINAL);
    let session = InputAuditSession::new();
    let mut reader = session.open(TraceId("T1".into()), &path).unwrap();
    reader.read_exact(&mut [0; 7]).unwrap();
    assert_eq!(reader.read(&mut []).unwrap(), 0);
    let snapshot = session.snapshot().unwrap();
    assert_eq!(snapshot[0].bytes_read, snapshot[0].file_size_bytes);
    assert_eq!(snapshot[0].completion, InputAuditCompletion::Incomplete);
    assert_eq!(reader.read(&mut [0]).unwrap(), 0);
    assert!(matches!(
        session.snapshot().unwrap()[0].completion,
        InputAuditCompletion::Complete { .. }
    ));
}

#[test]
fn cancelled_and_truncated_runs_can_have_either_actual_audit_completion() {
    for complete in [false, true] {
        for cancelled in [false, true] {
            let session = InputAuditSession::new();
            let mut reader = session
                .wrap(
                    TraceId("T1".into()),
                    "synthetic-T1.txt".into(),
                    7,
                    Cursor::new(ORIGINAL),
                )
                .unwrap();
            if complete {
                reader.read_to_end(&mut Vec::new()).unwrap();
            } else {
                reader.read_exact(&mut [0; 3]).unwrap();
            }
            let mut b = builder();
            b.record_processed().unwrap();
            b.record_invalid().unwrap();
            b.error_emitted(&ValidationCode::RequiredField).unwrap();
            if cancelled {
                b.cancel();
            } else {
                b.truncate();
            }
            let summary = b.finish(session.finish(metadata()).unwrap()).unwrap();
            assert_eq!(
                summary.run_status,
                if cancelled {
                    RunStatus::Cancelled
                } else {
                    RunStatus::CompletedInvalid
                }
            );
            assert_eq!(summary.truncated, !cancelled);
            assert_eq!(
                matches!(
                    summary.inputs[0].completion,
                    InputAuditCompletion::Complete { .. }
                ),
                complete
            );
        }
    }
}

#[test]
fn empty_file_requires_a_real_eof_and_has_the_known_empty_sha256() {
    let dir = TempDir::new();
    let path = dir.file("empty.txt", b"");
    let session = InputAuditSession::new();
    let mut reader = session.open(TraceId("T1".into()), &path).unwrap();
    assert_eq!(reader.read(&mut []).unwrap(), 0);
    assert_eq!(
        session.snapshot().unwrap()[0].completion,
        InputAuditCompletion::Incomplete
    );
    assert_eq!(reader.read(&mut [0]).unwrap(), 0);
    let meta = session.finish(metadata()).unwrap();
    let InputAuditCompletion::Complete { sha256 } = &meta.inputs[0].completion else {
        panic!("missing empty-file hash")
    };
    assert_eq!(
        sha256.to_string(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn initial_size_disagreement_at_eof_is_a_terminal_technical_error() {
    for bytes in [b"AB".as_slice(), b"ABCD"] {
        let dir = TempDir::new();
        let path = dir.file("changing.txt", b"ABC");
        let session = InputAuditSession::new();
        let mut reader = session.open(TraceId("T1".into()), &path).unwrap();
        fs::write(&path, bytes).unwrap(); // deliberate synthetic concurrent change after metadata capture
        let error = reader.read_to_end(&mut Vec::new()).unwrap_err();
        assert!(
            matches!(error.get_ref().unwrap().downcast_ref::<InputAuditError>(),Some(InputAuditError::SizeMismatch{expected:3,actual}) if *actual==bytes.len()as u64)
        );
        assert!(matches!(
            session.finish(metadata()),
            Err(InputAuditError::SizeMismatch { .. })
        ));
    }
}

#[test]
fn selection_order_and_one_authority_are_preserved_without_missing_trace_invention() {
    let dir = TempDir::new();
    let z = dir.file("z.txt", b"Z");
    let a = dir.file("a.txt", b"A");
    let session = InputAuditSession::new();
    let mut zr = session.open(TraceId("Z".into()), &z).unwrap();
    let mut ar = session.open(TraceId("A".into()), &a).unwrap();
    ar.read_to_end(&mut Vec::new()).unwrap();
    zr.read_to_end(&mut Vec::new()).unwrap();
    let snapshots = session.snapshot().unwrap();
    let meta = session.finish(metadata()).unwrap();
    assert_eq!(meta.inputs, snapshots);
    let allocation = meta.inputs.as_ptr();
    let b = ValidationSummaryBuilder::new(
        SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        ScopeSummary::Flow {
            trace_ids: vec![
                TraceId("Z".into()),
                TraceId("A".into()),
                TraceId("NOT-PROVIDED".into()),
            ],
        },
    );
    let summary = b.finish(meta).unwrap();
    assert_eq!(summary.inputs.as_ptr(), allocation); // metadata vector moved, not rebuilt independently
    assert_eq!(summary.inputs, snapshots);
    assert_eq!(
        summary
            .inputs
            .iter()
            .map(|a| a.trace_id.0.as_str())
            .collect::<Vec<_>>(),
        ["Z", "A"]
    );
    assert!(zr.read(&mut [0]).is_err());
    assert_eq!(summary.inputs, snapshots);
    let output = write_summary(Vec::new(), &summary, UNIX_EPOCH).unwrap();
    assert!(
        !String::from_utf8(output)
            .unwrap()
            .contains(dir.0.to_str().unwrap())
    );
}

#[test]
fn buffered_read_ahead_counts_source_bytes_not_validated_bytes() {
    let session = InputAuditSession::new();
    let reader = session
        .wrap(
            TraceId("T1".into()),
            "raw.txt".into(),
            7,
            Cursor::new(ORIGINAL),
        )
        .unwrap();
    let mut buffered = BufReader::with_capacity(32, reader);
    assert_eq!(buffered.fill_buf().unwrap(), ORIGINAL);
    buffered.consume(1);
    let audit = session.snapshot().unwrap();
    assert_eq!(audit[0].bytes_read, 7);
    assert_eq!(audit[0].completion, InputAuditCompletion::Incomplete);
}

struct Failing(Cursor<&'static [u8]>);
impl Read for Failing {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let n = self.0.read(out)?;
        if n == 0 {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "synthetic source failure",
            ))
        } else {
            Ok(n)
        }
    }
}
#[test]
fn io_failure_remains_input_error_and_prevents_any_definitive_summary() {
    let session = InputAuditSession::new();
    let reader = session
        .wrap(
            TraceId("T1".into()),
            "failure.txt".into(),
            7,
            Failing(Cursor::new(ORIGINAL)),
        )
        .unwrap();
    let mut records = RecordReader::new(
        BufReader::with_capacity(2, reader),
        NewlinePolicy::AutoLfOrCrlf,
    );
    let mut buffer = Vec::new();
    let mut b = builder();
    let mut output = Vec::new();
    let result = (|| -> Result<(), InputError> {
        while records.read_next(&mut buffer)?.is_some() {
            b.record_processed().unwrap();
            b.record_valid().unwrap();
        }
        let meta = session.finish(metadata()).map_err(InputError::Audit)?;
        let at = meta.started_at;
        let summary = b.finish(meta).unwrap();
        write_summary(&mut output, &summary, at).unwrap();
        Ok(())
    })();
    let error = result.unwrap_err();
    assert!(matches!(error, InputError::RecordRead { .. }));
    let source = error.source().unwrap().downcast_ref::<io::Error>().unwrap();
    assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
    let audit_error = source
        .get_ref()
        .unwrap()
        .downcast_ref::<InputAuditError>()
        .unwrap();
    assert_eq!(
        audit_error
            .source()
            .unwrap()
            .downcast_ref::<io::Error>()
            .unwrap()
            .kind(),
        io::ErrorKind::PermissionDenied
    );
    assert!(output.is_empty());
    // Even a caller that ignores the read error cannot obtain frozen metadata.
    let guarded = InputAuditSession::new();
    let mut failing = guarded
        .wrap(
            TraceId("T1".into()),
            "failed.txt".into(),
            7,
            Failing(Cursor::new(ORIGINAL)),
        )
        .unwrap();
    assert!(failing.read_to_end(&mut Vec::new()).is_err());
    assert!(matches!(
        guarded.finish(metadata()),
        Err(InputAuditError::Io(_))
    ));
}

#[test]
fn ordinary_completion_refuses_incomplete_audit_and_invalid_counters() {
    let session = InputAuditSession::new();
    let mut reader = session
        .wrap(
            TraceId("T1".into()),
            "raw.txt".into(),
            7,
            Cursor::new(ORIGINAL),
        )
        .unwrap();
    reader.read_exact(&mut [0; 7]).unwrap();
    assert_eq!(
        builder().finish(session.finish(metadata()).unwrap()),
        Err(SummaryBuildError::IncompleteInput)
    );
    let mut b = builder();
    b.record_processed().unwrap();
    assert_eq!(
        b.finish(read_file_metadata()),
        Err(SummaryBuildError::InconsistentRecordCounts)
    );
    let mut b = builder();
    b.record_processed().unwrap();
    b.record_invalid().unwrap();
    b.error_emitted(&ValidationCode::InvalidType).unwrap();
    b.error_emitted(&ValidationCode::RequiredField).unwrap();
    b.rule_skipped().unwrap();
    let summary = b.finish(read_file_metadata()).unwrap();
    assert_eq!(summary.run_status, RunStatus::CompletedInvalid);
    assert_eq!(summary.total_errors, 2);
    assert_eq!(summary.skipped_rules, 1);
    assert_eq!(json(&summary, UNIX_EPOCH)["result"], "INVALID");
    let mut no_audit = builder();
    no_audit.record_processed().unwrap();
    no_audit.record_valid().unwrap();
    assert_eq!(
        no_audit.finish(metadata()),
        Err(SummaryBuildError::MissingInputAudit)
    );
}

#[test]
fn metadata_cannot_inject_an_independent_mutable_audit_list() {
    let independent = read_file_metadata();
    let session = InputAuditSession::new();
    assert!(matches!(
        session.finish(independent),
        Err(InputAuditError::MetadataAlreadyContainsInputs)
    ));
}

#[test]
fn basename_privacy_open_failure_and_non_utf8_are_explicit_technical_boundaries() {
    for name in [
        "/private/source.txt",
        "C:\\private\\source.txt",
        "../source.txt",
        ".",
        "",
    ] {
        let session = InputAuditSession::new();
        assert!(
            session
                .wrap(TraceId("T1".into()), name.into(), 0, Cursor::new(b""))
                .is_err()
        );
        assert!(session.finish(metadata()).is_err());
    }
    let dir = TempDir::new();
    let session = InputAuditSession::new();
    assert!(matches!(
        session.open(TraceId("T1".into()), &dir.0.join("missing.txt")),
        Err(InputAuditError::Io(_))
    ));
    assert!(session.finish(metadata()).is_err());
    #[cfg(unix)]
    let name = {
        use std::os::unix::ffi::OsStringExt;
        std::ffi::OsString::from_vec(vec![b'x', 0xff])
    };
    #[cfg(windows)]
    let name = {
        use std::os::windows::ffi::OsStringExt;
        std::ffi::OsString::from_wide(&[b'x' as u16, 0xd800])
    };
    #[cfg(any(unix, windows))]
    {
        let session = InputAuditSession::new();
        assert!(matches!(
            session.open(TraceId("T1".into()), &dir.0.join(name)),
            Err(InputAuditError::NonUtf8Basename)
        ));
        assert!(matches!(
            session.finish(metadata()),
            Err(InputAuditError::NonUtf8Basename)
        ));
    }
}

#[test]
fn utc_timestamp_and_actual_summary_file_are_deterministic_and_keep_errors_separate() {
    let mut b = builder();
    b.record_processed().unwrap();
    b.record_valid().unwrap();
    let mut summary = b.finish(read_file_metadata()).unwrap();
    summary.elapsed = Duration::from_millis(12);
    let dir = TempDir::new();
    let path = dir.0.join("errors.summary.json");
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    drop(write_summary(file, &summary, UNIX_EPOCH).unwrap());
    assert_eq!(
        fs::read(path).unwrap(),
        include_bytes!("fixtures/summary.complete.json")
    );
    assert!(!dir.0.join("errors.jsonl").exists());
    let before_epoch = UNIX_EPOCH.checked_sub(Duration::from_millis(500)).unwrap();
    assert_eq!(
        json(&summary, before_epoch)["started_at"],
        "1969-12-31T23:59:59.500Z"
    );
    let year_10000 = UNIX_EPOCH
        .checked_add(Duration::from_secs(253_402_300_800))
        .unwrap();
    let mut output = Vec::new();
    assert!(write_summary(&mut output, &summary, year_10000).is_err());
    assert!(output.is_empty());
}

#[test]
fn interrupted_read_is_not_a_failed_audit_and_successful_bytes_are_counted_once() {
    struct InterruptedOnce {
        interrupted: bool,
        inner: Cursor<&'static [u8]>,
    }
    impl Read for InterruptedOnce {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::ErrorKind::Interrupted.into());
            }
            self.inner.read(out)
        }
    }
    let session = InputAuditSession::new();
    let mut reader = session
        .wrap(
            TraceId("T1".into()),
            "synthetic-T1.txt".into(),
            7,
            InterruptedOnce {
                interrupted: false,
                inner: Cursor::new(ORIGINAL),
            },
        )
        .unwrap();
    assert_eq!(
        reader.read(&mut [0]).unwrap_err().kind(),
        io::ErrorKind::Interrupted
    );
    assert_eq!(session.snapshot().unwrap()[0].bytes_read, 0);
    reader.read_to_end(&mut Vec::new()).unwrap();
    let meta = session.finish(metadata()).unwrap();
    assert_eq!(meta.inputs[0], read_file_metadata().inputs[0]);
}

#[test]
fn summary_write_and_flush_failures_are_technical_report_errors() {
    struct BrokenWriter {
        fail_flush: bool,
    }
    impl Write for BrokenWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.fail_flush {
                Ok(bytes.len())
            } else {
                Err(io::ErrorKind::PermissionDenied.into())
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
    }
    let summary = builder().finish(read_file_metadata()).unwrap();
    for fail_flush in [false, true] {
        let error = write_summary(BrokenWriter { fail_flush }, &summary, UNIX_EPOCH)
            .err()
            .unwrap();
        if fail_flush {
            assert!(matches!(error, lintra::ReportError::Write(_)));
            assert_eq!(
                error
                    .source()
                    .unwrap()
                    .downcast_ref::<io::Error>()
                    .unwrap()
                    .kind(),
                io::ErrorKind::BrokenPipe
            );
        } else {
            let source = error
                .source()
                .unwrap()
                .downcast_ref::<serde_json::Error>()
                .unwrap();
            assert_eq!(
                source.io_error_kind(),
                Some(io::ErrorKind::PermissionDenied)
            );
        }
    }
}

#[test]
fn bundle_writer_is_additive_and_propagates_io_errors() {
    let summary = builder().finish(read_file_metadata()).unwrap();
    let bundle = lintra::BundleAudit::external(None);
    let bytes = lintra::write_run_summary_with_bundle(Vec::new(), &summary, &bundle).unwrap();
    let mut new: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(new["bundle_source"], "external");
    assert!(new["bundle_version"].is_null());
    assert_eq!(new["bundle_manifest_status"], "absent");
    for key in ["bundle_source", "bundle_version", "bundle_manifest_status"] {
        new.as_object_mut().unwrap().remove(key);
    }
    let old = lintra::write_run_summary(Vec::new(), &summary).unwrap();
    assert_eq!(
        new,
        serde_json::from_slice::<serde_json::Value>(&old).unwrap()
    );
    struct BrokenWriter;
    impl Write for BrokenWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
    }
    assert!(lintra::write_run_summary_with_bundle(BrokenWriter, &summary, &bundle).is_err());
}
