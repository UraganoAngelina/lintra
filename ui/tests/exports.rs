//! Synthetic real ValidationService -> owned report -> core writers. No export
//! fixture pretends to be validation output; the worker produces every source.
#![cfg(feature = "embedded-bundle")]
use lintra::*;
use lintra_ui::{analysis::*, export::*, worker::*};
use std::{
    fs,
    io::{BufReader, Cursor},
    path::Path,
    sync::Arc,
};
fn analysis(bytes: &[u8]) -> (tempfile::TempDir, CompletedAnalysis) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.jsonl");
    fs::write(&path, bytes).unwrap();
    let prepared = AnalysisSelection {
        flow: Some("SYNTHETIC".into()),
        version: Some("v1".into()),
        trace: Some("T1".into()),
        encoding: Some(RawResolvedEncoding::Utf8),
        file: Some(path),
        reports: ReportSelection::default(),
    }
    .prepare(&AnalysisBundle::builtin().unwrap())
    .unwrap();
    let result = WorkerHandle::start(prepared, CancellationToken::default(), Arc::new(|| {}))
        .unwrap()
        .wait()
        .unwrap();
    (dir, result)
}
fn no_parts(dir: &Path) {
    assert!(
        !fs::read_dir(dir).unwrap().any(|e| e
            .unwrap()
            .path()
            .extension()
            .is_some_and(|s| s == "part"))
    );
}

#[test]
fn late_durability_failure_presents_the_actual_published_file_count_without_paths() {
    let failure = ExportFailure {
        cause: ExportError::Report(ReportError::PublishedDurabilityUnconfirmed {
            source: std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "synthetic directory sync",
            ),
        }),
        published: vec![
            "SYNTHETIC_PRIVATE_PATH.jsonl".into(),
            "SYNTHETIC_PRIVATE_PATH.summary.json".into(),
        ],
        cleanup: vec![],
        cancelled: false,
    };
    let text = failure.to_string();
    assert!(text.contains("2 file sono stati salvati"));
    assert!(text.contains("non ha confermato la scrittura su disco"));
    assert!(!text.contains("SYNTHETIC_PRIVATE_PATH"));
    assert!(!text.contains("Un file è stato salvato"));
}

/// Requires an ordinary OS account: a write/search-only directory permits
/// saving and rename, while opening it for directory fsync fails after publish.
/// CI explicitly runs this ignored case; running as root is not a valid proof.
#[cfg(target_os = "linux")]
#[test]
#[ignore = "Requires an unprivileged Linux account; native CI explicitly runs it"]
fn directory_sync_failure_after_rename_reports_the_published_file() {
    use std::os::unix::fs::PermissionsExt;
    struct Restore(std::path::PathBuf);
    impl Drop for Restore {
        fn drop(&mut self) {
            let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(0o700));
        }
    }
    let (_source_root, completed) = analysis(b"001Z  \n");
    let report = completed.report.unwrap();
    let parent = Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/export-durability-tests");
    fs::create_dir_all(&parent).unwrap();
    let output_root = tempfile::tempdir_in(parent).unwrap();
    let directory = output_root.path().join("write-search-only");
    fs::create_dir(&directory).unwrap();
    let _restore = Restore(directory.clone());
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o300)).unwrap();
    assert_eq!(
        fs::File::open(&directory).unwrap_err().kind(),
        std::io::ErrorKind::PermissionDenied,
        "this real permission-fault qualification requires an unprivileged account"
    );
    let path = directory.join("saved.jsonl");
    let failure = export_report(
        &report,
        &completed.outcome.summary,
        ExportFormat::Jsonl,
        &path,
        false,
        &CancellationToken::default(),
    )
    .unwrap_err();
    assert!(
        matches!(&failure.cause, ExportError::Report(ReportError::PublishedDurabilityUnconfirmed { source })
        if source.kind() == std::io::ErrorKind::PermissionDenied)
    );
    assert_eq!(failure.published, vec![path.clone()]);
    assert!(failure.cleanup.is_empty());
    assert!(!failure.cancelled);
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(
        fs::read(&path).unwrap(),
        fs::read(report.jsonl_path()).unwrap()
    );
    assert!(!summary_path(&path).exists());
    assert_eq!(
        completed.outcome.summary.run_status,
        RunStatus::CompletedInvalid
    );
    no_parts(&directory);
}
#[test]
fn formats_match_the_core_cli_writer_and_keep_summary_bundle_provenance() {
    for bytes in [b"001A  \n".as_slice(), b"001Z  \n".as_slice()] {
        let (root, completed) = analysis(bytes);
        let report = completed.report.unwrap();
        let summary = &completed.outcome.summary;
        let original = fs::read(report.jsonl_path()).unwrap();
        for format in [ExportFormat::Xlsx, ExportFormat::Jsonl, ExportFormat::Csv] {
            let path = root.path().join(format!("output.{}", format.extension()));
            let receipt = export_report(
                &report,
                summary,
                format,
                &path,
                false,
                &CancellationToken::default(),
            )
            .unwrap();
            assert_eq!(receipt.files, vec![path.clone(), summary_path(&path)]);
            let actual = fs::read(&path).unwrap();
            match format {
                ExportFormat::Xlsx => {
                    let expected = XlsxReportWriter::new()
                        .write(Cursor::new(&original), summary, Vec::new())
                        .unwrap();
                    assert_eq!(
                        actual, expected,
                        "same canonical source/summary as the CLI writer"
                    );
                }
                ExportFormat::Jsonl => assert_eq!(actual, original),
                ExportFormat::Csv => {
                    let mut sink = CsvSink::new(Vec::new()).unwrap();
                    replay_canonical_report(
                        Cursor::new(&original),
                        summary,
                        &mut sink,
                        &CancellationToken::default(),
                    )
                    .unwrap();
                    assert_eq!(actual, sink.finish().unwrap());
                }
            }
            let expected =
                write_run_summary_with_bundle(Vec::new(), summary, report.audit()).unwrap();
            assert_eq!(fs::read(summary_path(&path)).unwrap(), expected);
            no_parts(root.path());
        }
        assert_eq!(fs::read(root.path().join("input.jsonl")).unwrap(), bytes);
    }
}
#[test]
fn overwrite_source_protection_bad_paths_cancel_and_spool_damage_never_fake_success() {
    let (root, completed) = analysis(b"001Z  \n");
    let report = completed.report.unwrap();
    let summary = completed.outcome.summary;
    let path = root.path().join("saved.jsonl");
    fs::write(&path, b"existing report").unwrap();
    assert!(matches!(
        export_report(
            &report,
            &summary,
            ExportFormat::Jsonl,
            &path,
            false,
            &CancellationToken::default()
        )
        .unwrap_err()
        .cause,
        ExportError::OverwriteRequired
    ));
    assert_eq!(fs::read(&path).unwrap(), b"existing report");
    export_report(
        &report,
        &summary,
        ExportFormat::Jsonl,
        &path,
        true,
        &CancellationToken::default(),
    )
    .unwrap();
    assert!(matches!(
        export_report(
            &report,
            &summary,
            ExportFormat::Jsonl,
            &root.path().join("input.jsonl"),
            true,
            &CancellationToken::default()
        )
        .unwrap_err()
        .cause,
        ExportError::ProtectedSource
    ));
    assert!(matches!(
        export_report(
            &report,
            &summary,
            ExportFormat::Jsonl,
            &report.jsonl_path(),
            true,
            &CancellationToken::default()
        )
        .unwrap_err()
        .cause,
        ExportError::ProtectedSource
    ));
    assert!(matches!(
        export_report(
            &report,
            &summary,
            ExportFormat::Csv,
            Path::new("relative.csv"),
            false,
            &CancellationToken::default()
        )
        .unwrap_err()
        .cause,
        ExportError::InvalidPath
    ));
    let missing = root.path().join("missing/out.csv");
    assert!(matches!(
        export_report(
            &report,
            &summary,
            ExportFormat::Csv,
            &missing,
            false,
            &CancellationToken::default()
        )
        .unwrap_err()
        .cause,
        ExportError::Io(_)
    ));
    let cancel = CancellationToken::default();
    cancel.cancel();
    let cancelled = root.path().join("cancel.csv");
    assert!(
        export_report(
            &report,
            &summary,
            ExportFormat::Csv,
            &cancelled,
            false,
            &cancel
        )
        .unwrap_err()
        .cancelled
    );
    assert!(!cancelled.exists());
    fs::write(report.jsonl_path(), b"{broken\n").unwrap();
    let damaged = root.path().join("damaged.csv");
    assert!(matches!(
        export_report(
            &report,
            &summary,
            ExportFormat::Csv,
            &damaged,
            false,
            &CancellationToken::default()
        )
        .unwrap_err()
        .cause,
        ExportError::Replay(_)
    ));
    assert!(!damaged.exists());
    assert!(!summary_path(&damaged).exists());
    no_parts(root.path());
}
#[test]
fn replay_rejects_count_scope_and_framing_mismatch_before_publication() {
    let (_root, completed) = analysis(b"001Z  \n");
    let report = completed.report.unwrap();
    let original = fs::read(report.jsonl_path()).unwrap();
    let summary = completed.outcome.summary;
    for bytes in [
        Vec::new(),
        [original.clone(), original.clone()].concat(),
        {
            let mut v: serde_json::Value = serde_json::from_slice(&original).unwrap();
            v["trace"] = serde_json::json!("other");
            let mut out = serde_json::to_vec(&v).unwrap();
            out.push(b'\n');
            out
        },
        original.strip_suffix(b"\n").unwrap().to_vec(),
    ] {
        let mut sink = JsonlSink::new(Vec::new());
        assert!(
            replay_canonical_report(
                Cursor::new(bytes),
                &summary,
                &mut sink,
                &CancellationToken::default()
            )
            .is_err()
        );
    }
    let mut sink = JsonlSink::new(Vec::new());
    let file = fs::File::open(report.jsonl_path()).unwrap();
    replay_canonical_report(
        BufReader::new(file),
        &summary,
        &mut sink,
        &CancellationToken::default(),
    )
    .unwrap();
    assert_eq!(sink.finish().unwrap(), original);
}

#[test]
fn foreign_partial_file_is_preserved_and_our_staged_output_is_aborted() {
    let (root, completed) = analysis(b"001Z  \n");
    let report = completed.report.unwrap();
    let path = root.path().join("blocked.csv");
    let foreign = root.path().join("blocked.csv.summary.json.part");
    fs::write(&foreign, b"foreign pending writer").unwrap();
    let failure = export_report(
        &report,
        &completed.outcome.summary,
        ExportFormat::Csv,
        &path,
        false,
        &CancellationToken::default(),
    )
    .unwrap_err();
    assert!(matches!(failure.cause, ExportError::Report(_)));
    assert!(failure.published.is_empty());
    assert!(failure.cleanup.is_empty());
    assert_eq!(fs::read(&foreign).unwrap(), b"foreign pending writer");
    assert!(!path.exists());
    assert!(!root.path().join("blocked.csv.part").exists());
}
