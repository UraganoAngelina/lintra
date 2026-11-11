//! Explicit Windows CI fixture: real SMB destinations and real Slint callbacks.
#![cfg(all(windows, feature = "embedded-bundle"))]
use lintra::*;
use lintra_ui::{
    MainWindow, analysis::AnalysisBundle, controller::install_configuration, export::*,
    export_controller::install_exports, results::controller::install_results,
    worker_controller::install_worker,
};
use slint::platform::{
    Platform, WindowAdapter,
    software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
};
use std::{
    fs,
    io::Cursor,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

struct Headless {
    window: Rc<MinimalSoftwareWindow>,
    started: Instant,
}
impl Platform for Headless {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> Duration {
        self.started.elapsed()
    }
}
fn wait(mut pending: impl FnMut() -> bool, mut drive: impl FnMut()) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while pending() {
        assert!(Instant::now() < deadline, "actual shared export timed out");
        drive();
        slint::platform::update_timers_and_animations();
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
#[ignore = "Requires the real loopback SMB fixture; CI explicitly runs this case"]
fn actual_smb_and_mapped_export_require_confirmation_and_preserve_canonical_bytes() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Headless {
        window,
        started: Instant::now(),
    }))
    .unwrap();
    let ui = MainWindow::new().unwrap();
    let config = install_configuration(&ui, AnalysisBundle::builtin());
    let worker = install_worker(&ui, config);
    let results = install_results(&ui, worker.clone());
    let exports = install_exports(&ui, results.clone());
    let input = tempfile::tempdir().unwrap();
    let source = input.path().join("synthetic.txt");
    fs::write(&source, b"001Z  \n").unwrap();
    ui.invoke_choose_flow(0);
    ui.invoke_choose_version(0);
    ui.invoke_choose_trace(0);
    ui.invoke_choose_encoding(0);
    ui.invoke_file_confirmed(source.to_str().unwrap().into());
    ui.invoke_start_analysis();
    wait(|| ui.get_busy(), || ui.invoke_worker_poll());
    assert!(ui.get_results_available());
    assert!(ui.get_validation_invalid());
    ui.invoke_open_results();
    wait(|| ui.get_results_loading(), || {});
    let (report, summary) = {
        let state = worker.borrow();
        let result = state.last_completed().unwrap();
        (
            result.report.as_ref().unwrap().clone(),
            result.outcome.summary.clone(),
        )
    };
    assert_eq!(summary.run_status, RunStatus::CompletedInvalid);
    let original = fs::read(report.jsonl_path()).unwrap();
    for variable in ["LINTRA_TEST_EXPORT_UNC", "LINTRA_TEST_EXPORT_MAPPED"] {
        let endpoint =
            PathBuf::from(std::env::var_os(variable).expect("native SMB fixture required"));
        let root = tempfile::tempdir_in(endpoint).unwrap();
        for format in [ExportFormat::Jsonl, ExportFormat::Csv, ExportFormat::Xlsx] {
            let path = root
                .path()
                .join(format!("synthetic.{}", format.extension()));
            let failure = export_report(
                &report,
                &summary,
                format,
                &path,
                false,
                &CancellationToken::default(),
            )
            .unwrap_err();
            let assessment = match failure.cause {
                ExportError::Destination(ReportDestinationError::ConfirmationRequired(
                    assessment,
                )) => assessment,
                other => panic!("expected real remote warning: {other:?}"),
            };
            assert_eq!(assessment.counts().remote_files, 2);
            assert!(!path.exists());
            assert!(!summary_path(&path).exists());
            let receipt = export_report_with_authorization(
                &report,
                &summary,
                format,
                &path,
                false,
                &CancellationToken::default(),
                Some(assessment.acknowledge_warning()),
            )
            .unwrap();
            assert!(receipt.destination.warning_acknowledged);
            assert_eq!(receipt.destination.remote_files, 2);
            assert_eq!(receipt.files, [path.clone(), summary_path(&path)]);
            let expected = match format {
                ExportFormat::Jsonl => original.clone(),
                ExportFormat::Csv => {
                    let mut sink = CsvSink::new(Vec::new()).unwrap();
                    replay_canonical_report(
                        Cursor::new(&original),
                        &summary,
                        &mut sink,
                        &CancellationToken::default(),
                    )
                    .unwrap();
                    sink.finish().unwrap()
                }
                ExportFormat::Xlsx => XlsxReportWriter::new()
                    .write(Cursor::new(&original), &summary, Vec::new())
                    .unwrap(),
            };
            assert_eq!(fs::read(&path).unwrap(), expected);
            let expected_summary =
                write_run_summary_with_bundle(Vec::new(), &summary, report.audit()).unwrap();
            assert_eq!(fs::read(summary_path(&path)).unwrap(), expected_summary);
            assert!(matches!(
                export_report(
                    &report,
                    &summary,
                    format,
                    &path,
                    false,
                    &CancellationToken::default()
                )
                .unwrap_err()
                .cause,
                ExportError::OverwriteRequired
            ));
        }
        let path = root.path().join("retarget.csv");
        let paths = [path.clone(), summary_path(&path)];
        let permission = ReportDestinationAssessment::inspect(&paths)
            .unwrap()
            .acknowledge_warning();
        let other = root.path().join("another.csv");
        assert!(matches!(
            export_report_with_authorization(
                &report,
                &summary,
                ExportFormat::Csv,
                &other,
                false,
                &CancellationToken::default(),
                Some(permission)
            )
            .unwrap_err()
            .cause,
            ExportError::Destination(ReportDestinationError::SelectionChanged)
        ));
        assert!(!other.exists());
        let cancellation = CancellationToken::default();
        cancellation.cancel();
        assert!(
            export_report(
                &report,
                &summary,
                ExportFormat::Csv,
                &path,
                false,
                &cancellation
            )
            .unwrap_err()
            .cancelled
        );
        assert!(!path.exists());
        assert!(
            !fs::read_dir(root.path()).unwrap().any(|p| p
                .unwrap()
                .path()
                .extension()
                .is_some_and(|s| s == "part"))
        );

        // Exercise the actual controller's warning -> operator checkbox -> job,
        // not a frontend reconstruction or a bypass flag on the history CLI.
        ui.invoke_request_export();
        ui.invoke_choose_export_format(2);
        let dialog_path = root.path().join("dialog.csv");
        ui.set_export_path(dialog_path.to_str().unwrap().into());
        ui.invoke_export_path_edited();
        ui.invoke_confirm_export();
        wait(|| ui.get_export_busy(), || {});
        assert!(ui.get_export_destination_confirmation_required());
        assert!(!ui.get_export_destination_confirmed());
        assert!(!ui.get_export_error());
        assert!(ui.get_export_note().contains("remota o condivisa"));
        assert!(!dialog_path.exists());
        ui.set_export_destination_confirmed(true);
        ui.invoke_confirm_export();
        wait(|| ui.get_export_busy(), || {});
        assert!(!ui.get_export_error());
        assert!(
            exports
                .borrow()
                .last_receipt()
                .unwrap()
                .destination
                .warning_acknowledged
        );
        assert!(dialog_path.exists());
        ui.invoke_confirm_export();
        wait(|| ui.get_export_busy(), || {});
        assert!(ui.get_export_confirmation_required());
        ui.set_export_overwrite(true);
        ui.invoke_confirm_export();
        wait(|| ui.get_export_busy(), || {});
        assert!(ui.get_export_destination_confirmation_required());
        assert!(
            ui.get_export_confirmation_required(),
            "both independent choices stay visible"
        );
        ui.set_export_destination_confirmed(true);
        ui.invoke_confirm_export();
        wait(|| ui.get_export_busy(), || {});
        assert!(!ui.get_export_error());
        assert!(
            ui.get_validation_invalid(),
            "export consent cannot change validation outcome"
        );
        ui.invoke_close_export();
        ui.invoke_request_export();
        ui.invoke_choose_export_format(2);
        ui.set_export_path(root.path().join("next.csv").to_str().unwrap().into());
        ui.invoke_export_path_edited();
        ui.invoke_confirm_export();
        wait(|| ui.get_export_busy(), || {});
        assert!(
            ui.get_export_destination_confirmation_required(),
            "no remembered global consent"
        );
        ui.set_export_destination_confirmed(true);
        ui.invoke_choose_export_format(1);
        assert!(
            !ui.get_export_destination_confirmed(),
            "format changes invalidate the choice"
        );
        ui.invoke_close_export();
    }
    exports.borrow_mut().shutdown();
    results.borrow_mut().shutdown();
    worker.borrow_mut().shutdown();
}
