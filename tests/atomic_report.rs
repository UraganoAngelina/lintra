//! Real filesystem checks for DET §56/117/119. All data is synthetic.
use lintra::{
    AtomicOutputOutcome, AtomicReportFile, FlowId, JsonlSink, OutputOperation, PartialOutputPolicy,
    RecordIdentity, ReportCompletion, ReportError, RunStatus, SchemaKey, SchemaVersion,
    ScopeSummary, TraceId, ValidationCode, ValidationError, ValidationSink, ValidationSummary,
    write_atomic_report, write_summary,
};
use std::{
    collections::BTreeMap,
    error::Error,
    fs, io,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, UNIX_EPOCH},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// Each test owns an exclusive directory, including spaces/non-ASCII characters.
/// PID plus an atomic sequence isolate concurrent tests; create_dir arbitrates
/// collisions with leftovers rather than trusting the generated name alone.
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        loop {
            let path = std::env::temp_dir().join(format!(
                "lintra output à {} {}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("synthetic directory creation failed: {error}"),
            }
        }
    }
    /// Native path construction preserves spaces and Unicode on both OSes.
    fn output(&self) -> PathBuf {
        self.0.join("report à.jsonl")
    }
    /// Match the documented suffix, independently of implementation internals.
    fn part(&self) -> PathBuf {
        self.0.join("report à.jsonl.part")
    }
}
impl Drop for Fixture {
    /// Test resources are removed after the output guard has closed its file.
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

/// A complete producer outcome permits publication irrespective of validity.
fn complete(status: RunStatus) -> ReportCompletion {
    ReportCompletion {
        run_status: status,
        truncated: false,
    }
}

/// A genuine well-formed record diagnostic exercises the existing JSONL sink.
fn diagnostic() -> ValidationError {
    ValidationError {
        code: ValidationCode::RecordLength,
        flow_id: FlowId("SYNTHETIC".into()),
        schema_version: SchemaVersion("v1".into()),
        trace_id: TraceId("T1".into()),
        line_number: 1,
        record_identity: RecordIdentity::default(),
        field: None,
        actual: None,
        expected: None,
        context: vec![],
        message: "Synthetic record length error".into(),
    }
}

/// Empty valid scope requires no invented input audits; fixed times/counters
/// make the summary bytes deterministic for direct-versus-atomic comparison.
fn summary() -> ValidationSummary {
    ValidationSummary {
        reference_validation: None,
        full_validation: None,
        started_at: std::time::UNIX_EPOCH,
        validation_mode: lintra::ValidationMode::LocalOnly,
        run_status: RunStatus::CompletedValid,
        schema_key: SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        scope_summary: ScopeSummary::Trace {
            trace_id: TraceId("T1".into()),
        },
        records_processed: 0,
        valid_records: 0,
        invalid_records: 0,
        total_errors: 0,
        errors_by_code: BTreeMap::new(),
        skipped_rules: 0,
        truncated: false,
        validator_version: "synthetic-build".into(),
        schema_hash: "1".repeat(64),
        reference_data_versions: BTreeMap::new(),
        elapsed: Duration::ZERO,
        inputs: vec![],
    }
}

/// The callback sees only the part file; the definitive name appears after
/// success, and its bytes equal those already checked by the JSONL encoder.
#[test]
fn jsonl_is_invisible_until_successful_publication() {
    let fixture = Fixture::new();
    let mut expected = JsonlSink::new(Vec::new());
    expected.on_error(&diagnostic()).unwrap();
    let expected = expected.finish().unwrap();
    let outcome = write_atomic_report(&fixture.output(), PartialOutputPolicy::Delete, |writer| {
        let mut sink = JsonlSink::new(writer);
        sink.on_error(&diagnostic())?;
        sink.finish()?;
        assert!(!fixture.output().exists());
        assert_eq!(fs::read(fixture.part()).unwrap(), expected);
        Ok(complete(RunStatus::CompletedInvalid))
    })
    .unwrap();
    assert_eq!(outcome, AtomicOutputOutcome::Published);
    assert_eq!(fs::read(fixture.output()).unwrap(), expected);
    assert!(!fixture.part().exists());
}

/// The old destination remains readable while writing and is replaced via
/// rename rather than pre-deletion; the native Windows job runs this same test.
#[test]
fn existing_destination_is_replaced_only_at_success() {
    let fixture = Fixture::new();
    fs::write(fixture.output(), b"old complete report").unwrap();
    write_atomic_report(&fixture.output(), PartialOutputPolicy::Delete, |writer| {
        writer
            .write_all(b"new complete report")
            .map_err(ReportError::Write)?;
        assert_eq!(fs::read(fixture.output()).unwrap(), b"old complete report");
        Ok(complete(RunStatus::CompletedValid))
    })
    .unwrap();
    assert_eq!(fs::read(fixture.output()).unwrap(), b"new complete report");
    assert!(!fixture.part().exists());
}

/// Cancellation and truncation are unpublished outcomes, even if bytes were
/// written successfully. The loop also covers empty and preexisting targets.
#[test]
fn cancelled_and_truncated_runs_never_replace_a_definitive_report() {
    for preexisting in [false, true] {
        for (status, truncated) in [
            (RunStatus::Cancelled, false),
            (RunStatus::CompletedInvalid, true),
            (RunStatus::CompletedValid, true),
        ] {
            let fixture = Fixture::new();
            if preexisting {
                fs::write(fixture.output(), b"old").unwrap();
            }
            let outcome = write_atomic_report(
                &fixture.output(),
                PartialOutputPolicy::default(),
                |writer| {
                    writer.write_all(b"partial").map_err(ReportError::Write)?;
                    Ok(ReportCompletion {
                        run_status: status,
                        truncated,
                    })
                },
            )
            .unwrap();
            assert_eq!(
                outcome,
                AtomicOutputOutcome::Unpublished {
                    retained_part: None
                }
            );
            assert!(!fixture.part().exists());
            assert_eq!(fixture.output().exists(), preexisting);
            if preexisting {
                assert_eq!(fs::read(fixture.output()).unwrap(), b"old");
            }
        }
    }
}

/// An explicit diagnostic policy keeps partial bytes under their part name;
/// the same policy applies to controlled interruption and technical failure.
#[test]
fn diagnostic_retention_is_explicit_and_never_publishes() {
    for technical in [false, true] {
        let fixture = Fixture::new();
        let result = write_atomic_report(
            &fixture.output(),
            PartialOutputPolicy::RetainForDiagnostics,
            |writer| {
                writer.write_all(b"partial").map_err(ReportError::Write)?;
                if technical {
                    Err(ReportError::InvalidTimestamp)
                } else {
                    Ok(complete(RunStatus::Cancelled))
                }
            },
        );
        if technical {
            assert!(matches!(result, Err(ReportError::InvalidTimestamp)));
        } else {
            assert_eq!(
                result.unwrap(),
                AtomicOutputOutcome::Unpublished {
                    retained_part: Some(fixture.part()),
                }
            );
        }
        assert_eq!(fs::read(fixture.part()).unwrap(), b"partial");
        assert!(!fixture.output().exists());
    }
}

/// Real JSONL validation rejects a malformed diagnostic after a first valid
/// line. Its typed error survives cleanup; no prefix becomes a final report.
#[test]
fn sink_error_discards_partial_bytes_and_preserves_previous_report() {
    let fixture = Fixture::new();
    fs::write(fixture.output(), b"old").unwrap();
    let error = write_atomic_report(&fixture.output(), PartialOutputPolicy::Delete, |writer| {
        let mut sink = JsonlSink::new(writer);
        sink.on_error(&diagnostic())?;
        let mut invalid = diagnostic();
        invalid.line_number = 0;
        sink.on_error(&invalid)?;
        Ok(complete(RunStatus::CompletedInvalid))
    })
    .unwrap_err();
    assert!(matches!(error, ReportError::InvalidDiagnostic(_)));
    assert!(error.source().is_some());
    assert_eq!(fs::read(fixture.output()).unwrap(), b"old");
    assert!(!fixture.part().exists());
}

/// A stale part is someone else's data. Exclusive creation must fail before
/// the producer runs and must leave both existing files unchanged.
#[test]
fn preexisting_part_is_neither_truncated_nor_cleaned_up() {
    let fixture = Fixture::new();
    fs::write(fixture.part(), b"other writer").unwrap();
    fs::write(fixture.output(), b"old").unwrap();
    let error = write_atomic_report(&fixture.output(), PartialOutputPolicy::Delete, |_| {
        panic!("producer must not run after exclusive creation failed")
    })
    .unwrap_err();
    assert!(matches!(error, ReportError::Output {
        operation: OutputOperation::Create, ref source
    } if source.kind() == io::ErrorKind::AlreadyExists));
    assert_eq!(fs::read(fixture.part()).unwrap(), b"other writer");
    assert_eq!(fs::read(fixture.output()).unwrap(), b"old");
}

/// A directory cannot be replaced by a file rename. This produces a real OS
/// error on Linux/Windows and verifies that failed publication cleans its part.
#[test]
fn rename_failure_is_technical_and_preserves_destination_directory() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.output()).unwrap();
    fs::write(fixture.output().join("marker"), b"preserved").unwrap();
    let error = write_atomic_report(&fixture.output(), PartialOutputPolicy::Delete, |writer| {
        writer.write_all(b"complete").map_err(ReportError::Write)?;
        Ok(complete(RunStatus::CompletedValid))
    })
    .unwrap_err();
    assert!(matches!(
        error,
        ReportError::Output {
            operation: OutputOperation::Publish,
            ..
        }
    ));
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<io::Error>()
            .is_some()
    );
    assert!(!fixture.part().exists());
    assert_eq!(
        fs::read(fixture.output().join("marker")).unwrap(),
        b"preserved"
    );
}

/// The existing summary encoder produces identical bytes through the guard.
/// Its own validation error triggers rollback before any definitive output.
#[test]
fn summary_encoder_integrates_without_format_changes() {
    let fixture = Fixture::new();
    let summary = summary();
    let expected = write_summary(Vec::new(), &summary, UNIX_EPOCH).unwrap();
    write_atomic_report(&fixture.output(), PartialOutputPolicy::Delete, |writer| {
        write_summary(writer, &summary, UNIX_EPOCH)?;
        Ok(complete(summary.run_status))
    })
    .unwrap();
    assert_eq!(fs::read(fixture.output()).unwrap(), expected);
    let mut invalid = summary;
    invalid.valid_records = 1;
    let error = write_atomic_report(&fixture.output(), PartialOutputPolicy::Delete, |writer| {
        write_summary(writer, &invalid, UNIX_EPOCH)?;
        Ok(complete(invalid.run_status))
    })
    .unwrap_err();
    assert!(matches!(error, ReportError::InvalidSummary(_)));
    assert_eq!(fs::read(fixture.output()).unwrap(), expected);
    assert!(!fixture.part().exists());
}

/// The guard can abort after an actual upstream read failure without converting
/// InputError into ReportError. Error classification remains caller-owned.
#[test]
fn upstream_read_error_can_abort_without_publishing() {
    let fixture = Fixture::new();
    let mut output =
        AtomicReportFile::create_replacing(&fixture.output(), PartialOutputPolicy::Delete).unwrap();
    output.writer().write_all(b"prefix").unwrap();
    let upstream = fs::read(fixture.0.join("absent-input")).unwrap_err();
    assert_eq!(upstream.kind(), io::ErrorKind::NotFound);
    assert_eq!(
        output.abort().unwrap(),
        AtomicOutputOutcome::Unpublished {
            retained_part: None
        }
    );
    assert!(!fixture.output().exists());
    assert!(!fixture.part().exists());
}

/// Dropping a guard or unwinding a producer panic exercises RAII cleanup.
/// No success result exists on either path, so neither may publish a file.
#[test]
fn drop_and_panic_cleanup_default_parts() {
    let fixture = Fixture::new();
    {
        let mut output =
            AtomicReportFile::create_replacing(&fixture.output(), PartialOutputPolicy::Delete)
                .unwrap();
        output.writer().write_all(b"abandoned").unwrap();
    }
    assert!(!fixture.part().exists());
    let unwind = std::panic::catch_unwind(|| {
        let _ = write_atomic_report(&fixture.output(), PartialOutputPolicy::Delete, |writer| {
            writer.write_all(b"panic prefix").unwrap();
            panic!("synthetic producer panic")
        });
    });
    assert!(unwind.is_err());
    assert!(!fixture.part().exists());
    assert!(!fixture.output().exists());
}

/// Creation failure occurs before ownership: absent parents are not created,
/// and a root path without filename is rejected as a typed Create failure.
#[test]
fn invalid_destination_and_missing_parent_never_run_the_producer() {
    let fixture = Fixture::new();
    for destination in [
        fixture.0.join("absent/report"),
        fixture.0.ancestors().last().unwrap().to_owned(),
    ] {
        let error = write_atomic_report(&destination, PartialOutputPolicy::Delete, |_| {
            panic!("producer must not run after creation failure")
        })
        .unwrap_err();
        assert!(matches!(
            error,
            ReportError::Output {
                operation: OutputOperation::Create,
                ..
            }
        ));
    }
    assert!(!fixture.0.join("absent").exists());
}

/// An empty, valid JSONL stream is a complete zero-byte report, not a missing
/// output. The rename must succeed and remove its empty staging file as usual.
#[test]
fn empty_completed_report_is_published() {
    let fixture = Fixture::new();
    let outcome = write_atomic_report(&fixture.output(), PartialOutputPolicy::Delete, |writer| {
        JsonlSink::new(writer).finish()?;
        Ok(complete(RunStatus::CompletedValid))
    })
    .unwrap();
    assert_eq!(outcome, AtomicOutputOutcome::Published);
    assert!(fs::read(fixture.output()).unwrap().is_empty());
    assert!(!fixture.part().exists());
}

/// Windows can refuse replacement when another process holds the destination
/// without delete sharing. The implementation must report that OS error and
/// preserve the old file, rather than deleting it and retrying the rename.
#[cfg(windows)]
#[test]
fn windows_locked_destination_is_preserved_without_delete_fallback() {
    use std::os::windows::fs::OpenOptionsExt;
    let fixture = Fixture::new();
    fs::write(fixture.output(), b"old").unwrap();
    // FILE_SHARE_READ (0x1) permits readers but deliberately excludes deletion.
    let held = fs::OpenOptions::new()
        .read(true)
        .share_mode(0x1)
        .open(fixture.output())
        .unwrap();
    let result = write_atomic_report(&fixture.output(), PartialOutputPolicy::Delete, |writer| {
        writer.write_all(b"new").map_err(ReportError::Write)?;
        Ok(complete(RunStatus::CompletedValid))
    });
    assert!(matches!(
        result,
        Err(ReportError::Output {
            operation: OutputOperation::Publish,
            ..
        })
    ));
    assert_eq!(fs::read(fixture.output()).unwrap(), b"old");
    assert!(!fixture.part().exists());
    drop(held);
}
