//! Post-analysis exports own the original report while a bounded worker runs.
//! The export engine (preflight, destination policy, canonical writers and
//! per-file receipt) is the shared library one
//! (`lintra::export_captured_report*`);
//! this adapter only binds it to the Slint `AnalysisReport` and its worker.
use crate::{analysis::ReportSelection, worker::AnalysisReport};
use lintra::{AuthorizedReportDestinations, CancellationToken, ExportSource, ValidationSummary};
pub use lintra::{ExportError, ExportFailure, ExportFormat, ExportReceipt, summary_path};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
};

/// Format preselected from the reports requested with the analysis.
pub fn preferred_format(reports: ReportSelection) -> ExportFormat {
    if reports.xlsx {
        ExportFormat::Xlsx
    } else if reports.csv {
        ExportFormat::Csv
    } else {
        ExportFormat::Jsonl
    }
}

/// Run `export` against the captured Slint report: its spool, its private
/// directory and its single analysed input are the protected sources.
fn with_source<T>(report: &AnalysisReport, export: impl FnOnce(&ExportSource<'_>) -> T) -> T {
    let jsonl = report.jsonl_path();
    export(&ExportSource {
        jsonl: &jsonl,
        private_root: report.directory(),
        protected_inputs: std::slice::from_ref(report.protected_input()),
        audit: report.audit(),
    })
}

pub fn export_report(
    report: &AnalysisReport,
    summary: &ValidationSummary,
    format: ExportFormat,
    path: &Path,
    overwrite: bool,
    cancellation: &CancellationToken,
) -> Result<ExportReceipt, ExportFailure> {
    export_report_with_authorization(report, summary, format, path, overwrite, cancellation, None)
}

/// The dialog can acknowledge an inspection returned by a previous attempt.
/// Local exports use the ordinary entry point with no extra privacy checkbox.
/// Both routes run the exact same OS policy and canonical writers in Rust.
pub fn export_report_with_authorization(
    report: &AnalysisReport,
    summary: &ValidationSummary,
    format: ExportFormat,
    path: &Path,
    overwrite: bool,
    cancellation: &CancellationToken,
    authorization: Option<AuthorizedReportDestinations>,
) -> Result<ExportReceipt, ExportFailure> {
    with_source(report, |source| {
        lintra::export_captured_report_with_authorization(
            source,
            summary,
            format,
            path,
            overwrite,
            cancellation,
            authorization,
        )
    })
}

/// One terminal slot, owned join handle and cooperative token. The report Arc
/// keeps the JSONL/temp directory alive across a new validation or UI selection.
pub struct ExportJob {
    receiver: Receiver<Result<ExportReceipt, ExportFailure>>,
    token: CancellationToken,
    thread: Option<JoinHandle<()>>,
}
impl ExportJob {
    pub fn start(
        report: Arc<AnalysisReport>,
        summary: ValidationSummary,
        format: ExportFormat,
        path: PathBuf,
        overwrite: bool,
    ) -> Result<Self, ExportError> {
        Self::start_with_authorization(report, summary, format, path, overwrite, None)
    }
    pub fn start_with_authorization(
        report: Arc<AnalysisReport>,
        summary: ValidationSummary,
        format: ExportFormat,
        path: PathBuf,
        overwrite: bool,
        authorization: Option<AuthorizedReportDestinations>,
    ) -> Result<Self, ExportError> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let token = CancellationToken::default();
        let cancellation = token.clone();
        let thread = thread::Builder::new()
            .name("lintra-export".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    export_report_with_authorization(
                        &report,
                        &summary,
                        format,
                        &path,
                        overwrite,
                        &cancellation,
                        authorization,
                    )
                }))
                .unwrap_or_else(|_| Err(ExportError::WorkerUnavailable.into()));
                let _ = sender.try_send(result);
            })
            .map_err(ExportError::Spawn)?;
        Ok(Self {
            receiver,
            token,
            thread: Some(thread),
        })
    }
    pub fn cancel(&self) {
        self.token.cancel();
    }
    pub fn try_result(&mut self) -> Option<Result<ExportReceipt, ExportFailure>> {
        match self.receiver.try_recv() {
            Ok(result) => {
                if let Some(thread) = self.thread.take() {
                    let _ = thread.join();
                }
                Some(result)
            }
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err(ExportError::WorkerUnavailable.into()))
            }
        }
    }
}
impl Drop for ExportJob {
    fn drop(&mut self) {
        self.token.cancel();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
