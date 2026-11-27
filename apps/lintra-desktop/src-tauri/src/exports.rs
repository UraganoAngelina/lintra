//! Report provenance (About) and native export of the published report
//! (T-MIG-P4-10).
//!
//! Contract: `docs/migration/ipc-v1/EXPORT.md` (IPC v1, ratified 14 Nov 2026)
//! and `docs/EXPORT_DESTINATIONS.md` (D04).
//!
//! The renderer never sends or receives a path, a file name, a confirmation
//! flag or report content:
//! - `report_about` returns the provenance of the published report and its
//!   opaque `report_ref`;
//! - `export_prepare` runs the native save dialog and the native
//!   confirmations, then binds an opaque `export_ref` to the captured report,
//!   the format, the chosen destinations and their classification;
//! - `export_execute` consumes that plan and starts one owned export job;
//! - `export_state` and `export_cancel` observe and cancel it.
//!
//! The export itself is the shared library engine
//! (`lintra::export_captured_report_with_authorization`): structural preflight
//! and `verify_unchanged` before staging and again before publication,
//! canonical JSONL/CSV/XLSX writers, `AtomicReportFile` per file and an
//! explicit receipt for partial publication. Every input of the run, a whole
//! Full bundle included, and the private spool are protected destinations.
use crate::{
    configuration::fields,
    ipc::{self, ErrorCode},
    jobs::{Report, RunRef},
    results::{identifier, nonce, preview},
};
use lintra::{
    AuthorizedReportDestinations, CancellationToken, ExportFailure, ExportFormat, ExportReceipt,
    ExportSource, ReportDestinationAssessment, ReportDestinationReceipt, ReportError, RunStatus,
    ScopeSummary, ValidationMode,
};
use serde_json::{Map, Value, json};
use std::{
    path::PathBuf,
    sync::Arc,
    thread::{self, JoinHandle},
};

/// Native capabilities the export needs, supplied by the window owner (the
/// GUI thread). Tests supply synthetic answers. Each call blocks only that
/// thread, as the input picker already does.
pub(crate) trait ExportDialogs {
    /// Native save dialog; `None` when the operator cancels it.
    fn save(&self, extension: &str) -> Result<Option<PathBuf>, ErrorCode>;
    /// Native confirmation; `true` only on an explicit confirmation.
    fn confirm(&self, title: &str, message: &str) -> Result<bool, ErrorCode>;
}

/// Exact purpose-specific DTOs, decoded before any state is touched.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ExportRequest {
    About {
        run: RunRef,
    },
    Prepare {
        report_ref: String,
        format: ExportFormat,
    },
    Execute {
        export_ref: String,
    },
    State {
        export_ref: String,
    },
    Cancel {
        export_ref: String,
    },
}
impl ExportRequest {
    pub(crate) fn decode(operation: &str, dto: &Map<String, Value>) -> Result<Self, ErrorCode> {
        ipc::version(dto)?;
        let reference = |key: &str| identifier(&dto[key]);
        match operation {
            "report_about" => {
                fields(dto, &["run"])?;
                Ok(Self::About {
                    run: RunRef::decode(&dto["run"])?,
                })
            }
            "export_prepare" => {
                fields(dto, &["report_ref", "format"])?;
                Ok(Self::Prepare {
                    report_ref: reference("report_ref")?,
                    format: match dto["format"].as_str() {
                        Some("jsonl") => ExportFormat::Jsonl,
                        Some("csv") => ExportFormat::Csv,
                        Some("xlsx") => ExportFormat::Xlsx,
                        _ => return Err(ErrorCode::InvalidMessage),
                    },
                })
            }
            "export_execute" | "export_state" | "export_cancel" => {
                fields(dto, &["export_ref"])?;
                let export_ref = reference("export_ref")?;
                Ok(match operation {
                    "export_execute" => Self::Execute { export_ref },
                    "export_state" => Self::State { export_ref },
                    _ => Self::Cancel { export_ref },
                })
            }
            _ => Err(ErrorCode::UnknownOperation),
        }
    }
}

fn format_name(format: ExportFormat) -> &'static str {
    format.extension()
}

/// A report is exportable only when its run completed and was not truncated;
/// a normative Partial (Unknown checks) is not a reason to refuse it.
fn exportable(report: &Report) -> bool {
    let summary = report.summary();
    matches!(
        summary.run_status,
        RunStatus::CompletedValid | RunStatus::CompletedInvalid
    ) && !summary.truncated
}

fn counts(receipt: ReportDestinationReceipt) -> Value {
    json!({"schemaVersion":1,"local_files":receipt.local_files.to_string(),
        "remote_files":receipt.remote_files.to_string(),
        "undetermined_files":receipt.undetermined_files.to_string(),
        "warning_acknowledged":receipt.warning_acknowledged})
}

/// A ready plan: everything the job needs, owned by Rust. The report `Arc`
/// keeps the captured spool alive even if a newer run is published.
struct Plan {
    export_ref: String,
    report_ref: String,
    run: RunRef,
    report: Arc<Report>,
    format: ExportFormat,
    path: PathBuf,
    destinations: Vec<PathBuf>,
    overwrite: bool,
    authorization: AuthorizedReportDestinations,
}

type Outcome = Result<ExportReceipt, ExportFailure>;

/// The one export job of this instance. Dropping it cancels and joins.
struct Job {
    export_ref: String,
    report_ref: String,
    run: RunRef,
    format: ExportFormat,
    destinations: Vec<PathBuf>,
    receipt: ReportDestinationReceipt,
    token: CancellationToken,
    thread: Option<JoinHandle<Outcome>>,
    result: Option<Value>,
}
impl Drop for Job {
    fn drop(&mut self) {
        self.token.cancel();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
impl Job {
    /// Join a finished worker without blocking and fix its receipt once.
    fn poll(&mut self) {
        if !self.thread.as_ref().is_some_and(JoinHandle::is_finished) {
            return;
        }
        let outcome = self
            .thread
            .take()
            .map(JoinHandle::join)
            .and_then(Result::ok);
        self.result = Some(self.receipt_dto(outcome));
    }
    /// ExportResult DTO. Success needs both files published, durability
    /// confirmed and no cancel or cleanup failure. A failure keeps the exact
    /// published roles, including a rename followed by a failed sync; a lost
    /// worker never claims that nothing was published.
    fn receipt_dto(&self, outcome: Option<Outcome>) -> Value {
        let role = |path: &PathBuf| {
            if Some(path) == self.destinations.first() {
                "report"
            } else {
                "summary"
            }
        };
        let (state, published, cleanup_failed, durable) = match &outcome {
            Some(Ok(receipt)) => (
                "succeeded",
                receipt.files.iter().map(role).collect::<Vec<_>>(),
                false,
                true,
            ),
            Some(Err(failure)) => (
                if failure.cancelled {
                    "cancelled"
                } else {
                    "failed"
                },
                failure.published.iter().map(role).collect(),
                !failure.cleanup.is_empty(),
                !failure.published.is_empty()
                    && !matches!(
                        &failure.cause,
                        lintra::ExportError::Report(
                            ReportError::PublishedDurabilityUnconfirmed { .. }
                        )
                    ),
            ),
            None => ("failed", Vec::new(), false, false),
        };
        let succeeded = state == "succeeded" && published.len() == 2;
        let state = if state == "succeeded" && !succeeded {
            "failed"
        } else {
            state
        };
        let destination = match &outcome {
            Some(Ok(receipt)) => receipt.destination,
            _ => self.receipt,
        };
        json!({"schemaVersion":1,"export_ref":self.export_ref,"report_ref":self.report_ref,
        "run":self.run.dto(),"format":format_name(self.format),"state":state,
        "published_roles":published,"cleanup_failed":cleanup_failed,
        "durability_confirmed":durable,"destination":counts(destination),
        "error":if state == "failed" {
            json!({"schemaVersion":1,"category":"technical","code":"IPC_TECHNICAL_FAILURE"})
        } else {
            Value::Null
        }})
    }
}

/// Export state of the instance, owned by `Jobs` next to the last report.
#[derive(Default)]
pub(crate) struct Exports {
    // Dropped (cancelled and joined) before anything else of this struct.
    job: Option<Job>,
    plan: Option<Plan>,
    // Opaque handle of the published report, minted by `report_about`.
    report: Option<(String, RunRef, Arc<Report>)>,
}
impl Exports {
    pub(crate) fn cancel_on_close(&self) {
        if let Some(job) = &self.job {
            job.token.cancel();
        }
    }

    pub(crate) fn dispatch(
        &mut self,
        request: ExportRequest,
        published: Option<&(RunRef, Arc<Report>)>,
        dialogs: &dyn ExportDialogs,
        id: &str,
    ) -> Result<Vec<u8>, ErrorCode> {
        if let Some(job) = &mut self.job {
            job.poll();
        }
        match request {
            ExportRequest::About { run } => self.about(run, published, id),
            ExportRequest::Prepare { report_ref, format } => {
                self.prepare(&report_ref, format, published, dialogs, id)
            }
            ExportRequest::Execute { export_ref } => self.execute(&export_ref, id),
            ExportRequest::State { export_ref } => self.state(&export_ref, id),
            ExportRequest::Cancel { export_ref } => self.cancel(&export_ref, id),
        }
    }

    /// Provenance of the published report: executable and validator versions,
    /// schema identity and hash, bundle provenance, mode, scope and outcome.
    /// Counts and bounded previews only; no path, input or report content.
    fn about(
        &mut self,
        run: RunRef,
        published: Option<&(RunRef, Arc<Report>)>,
        id: &str,
    ) -> Result<Vec<u8>, ErrorCode> {
        let (current, report) = published.ok_or(ErrorCode::InvalidState)?;
        if run != *current {
            return Err(ErrorCode::StaleContext);
        }
        let report_ref = match &self.report {
            Some((reference, run, held)) if run == current && Arc::ptr_eq(held, report) => {
                reference.clone()
            }
            previous => {
                let reference = nonce("p")?;
                if previous.as_ref().is_some_and(|(old, ..)| *old == reference) {
                    return Err(ErrorCode::TechnicalFailure);
                }
                reference
            }
        };
        let summary = report.summary();
        let audit = report.audit();
        let traces = match &summary.scope_summary {
            ScopeSummary::Trace { .. } => 1,
            ScopeSummary::Flow { trace_ids } => trace_ids.len(),
        };
        let datasets = summary
            .reference_validation
            .as_ref()
            .map_or(0, |audit| audit.datasets.len());
        let bytes = ipc::response(
            id,
            json!({"schemaVersion":1,"report_ref":report_ref,"run":current.dto(),
                "app_version":env!("CARGO_PKG_VERSION"),
                "validator_version":preview(&summary.validator_version),
                "flow_id":preview(&summary.schema_key.flow_id.0),
                "schema_version":preview(&summary.schema_key.version.0),
                "schema_hash":preview(&summary.schema_hash),
                "mode":match summary.validation_mode {
                    ValidationMode::LocalOnly => "local_only",
                    ValidationMode::Full => "full",
                },
                "status":summary.run_status,"completeness":summary.completeness(),
                "truncated":summary.truncated,"exportable":exportable(report),
                "trace_count":traces.to_string(),"reference_dataset_count":datasets.to_string(),
                "bundle":{"schemaVersion":1,"bundle_source":audit.source(),
                    "bundle_version":audit.bundle_version(),
                    "bundle_manifest_status":audit.manifest_status()}}),
        )?;
        if self
            .report
            .as_ref()
            .is_none_or(|(reference, ..)| *reference != report_ref)
        {
            // A new published report invalidates the old handle and plan.
            self.plan = None;
            self.report = Some((report_ref, current.clone(), report.clone()));
        }
        Ok(bytes)
    }

    /// Native selection and confirmations, then an immutable ready plan.
    fn prepare(
        &mut self,
        report_ref: &str,
        format: ExportFormat,
        published: Option<&(RunRef, Arc<Report>)>,
        dialogs: &dyn ExportDialogs,
        id: &str,
    ) -> Result<Vec<u8>, ErrorCode> {
        if self.job.as_ref().is_some_and(|job| job.result.is_none()) {
            return Err(ErrorCode::Busy);
        }
        let (reference, run, report) = self
            .report
            .as_ref()
            .filter(|(reference, ..)| reference == report_ref)
            .cloned()
            .ok_or(ErrorCode::StaleContext)?;
        // The handle names the report still published: a newer run made it stale.
        if published.is_none_or(|(current, held)| *current != run || !Arc::ptr_eq(held, &report)) {
            return Err(ErrorCode::StaleContext);
        }
        if !exportable(&report) {
            return Err(ErrorCode::InvalidState);
        }
        // A new selection replaces any previous ready plan.
        self.plan = None;
        let cancelled = |reason: &str| {
            ipc::response(
                id,
                json!({"schemaVersion":1,"state":"cancelled","reason":reason}),
            )
        };
        let Some(path) = dialogs.save(format.extension())? else {
            return cancelled("native_selection_cancelled");
        };
        let spool = report.spool();
        let source = ExportSource {
            jsonl: &spool,
            private_root: report.private_root(),
            protected_inputs: report.inputs(),
            audit: report.audit(),
        };
        // Destination structure and protection are checked before asking
        // anything: a protected or invalid choice is refused, never confirmed.
        // The answer says so without naming the path or the reason detail.
        let Ok(destinations) = lintra::export_destinations(&source, &path, format) else {
            return ipc::response(
                id,
                json!({"schemaVersion":1,"state":"rejected","reason":"destination_not_allowed"}),
            );
        };
        let assessment = ReportDestinationAssessment::inspect(&destinations)
            .map_err(|_| ErrorCode::TechnicalFailure)?;
        let shared = assessment.requires_confirmation();
        let overwrite = destinations.iter().any(|path| path.exists());
        let names = destinations
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join("\n");
        if shared {
            let undetermined = assessment.counts().undetermined_files > 0;
            let message = format!(
                "{}\n\nFile che verranno salvati:\n{names}",
                if undetermined {
                    "Non è possibile determinare se la destinazione è locale o condivisa. Verifica gli accessi alla cartella prima di confermare."
                } else {
                    "La destinazione è remota o condivisa: altre persone potrebbero accedere ai file. Verifica gli accessi prima di confermare."
                }
            );
            if !dialogs.confirm("Destinazione condivisa", &message)? {
                return cancelled("native_confirmation_declined");
            }
        }
        if overwrite
            && !dialogs.confirm(
                "Sostituire i file esistenti?",
                &format!(
                    "Il report o il relativo riepilogo esiste già e verrà sostituito:\n{names}"
                ),
            )?
        {
            return cancelled("native_confirmation_declined");
        }
        let authorization = if shared {
            assessment.acknowledge_warning()
        } else {
            assessment
                .allow_local()
                .map_err(|_| ErrorCode::TechnicalFailure)?
        };
        let export_ref = nonce("e")?;
        if self
            .job
            .as_ref()
            .is_some_and(|job| job.export_ref == export_ref)
        {
            return Err(ErrorCode::TechnicalFailure);
        }
        let bytes = ipc::response(
            id,
            json!({"schemaVersion":1,"export_ref":export_ref,"report_ref":reference,
                "run":run.dto(),"format":format_name(format),"state":"ready",
                "destination":counts(authorization.receipt())}),
        )?;
        self.plan = Some(Plan {
            export_ref,
            report_ref: reference,
            run,
            report,
            format,
            path,
            destinations,
            overwrite,
            authorization,
        });
        Ok(bytes)
    }

    /// Consume the plan before any I/O (no replay) and start the owned job.
    fn execute(&mut self, export_ref: &str, id: &str) -> Result<Vec<u8>, ErrorCode> {
        if self.job.as_ref().is_some_and(|job| job.result.is_none()) {
            return Err(ErrorCode::Busy);
        }
        if self
            .plan
            .as_ref()
            .is_none_or(|plan| plan.export_ref != export_ref)
        {
            return Err(ErrorCode::StaleContext);
        }
        let bytes = ipc::response(
            id,
            json!({"schemaVersion":1,"export_ref":export_ref,"state":"writing"}),
        )?;
        let plan = self.plan.take().ok_or(ErrorCode::StaleContext)?;
        let token = CancellationToken::default();
        let cancellation = token.clone();
        let receipt = plan.authorization.receipt();
        let (report, format, path, overwrite, authorization) = (
            plan.report.clone(),
            plan.format,
            plan.path.clone(),
            plan.overwrite,
            plan.authorization,
        );
        let thread = thread::Builder::new()
            .name("lintra-desktop-export".into())
            .spawn(move || {
                let spool = report.spool();
                let source = ExportSource {
                    jsonl: &spool,
                    private_root: report.private_root(),
                    protected_inputs: report.inputs(),
                    audit: report.audit(),
                };
                lintra::export_captured_report_with_authorization(
                    &source,
                    report.summary(),
                    format,
                    &path,
                    overwrite,
                    &cancellation,
                    Some(authorization),
                )
            })
            .map_err(|_| ErrorCode::TechnicalFailure)?;
        self.job = Some(Job {
            export_ref: plan.export_ref,
            report_ref: plan.report_ref,
            run: plan.run,
            format: plan.format,
            destinations: plan.destinations,
            receipt,
            token,
            thread: Some(thread),
            result: None,
        });
        Ok(bytes)
    }

    fn job(&self, export_ref: &str) -> Result<&Job, ErrorCode> {
        self.job
            .as_ref()
            .filter(|job| job.export_ref == export_ref)
            .ok_or(ErrorCode::StaleContext)
    }

    /// `writing` until the worker is joined, then the fixed receipt.
    fn state(&self, export_ref: &str, id: &str) -> Result<Vec<u8>, ErrorCode> {
        let job = self.job(export_ref)?;
        match &job.result {
            Some(receipt) => ipc::response(id, receipt.clone()),
            None => ipc::response(
                id,
                json!({"schemaVersion":1,"export_ref":export_ref,"state":"writing"}),
            ),
        }
    }

    /// Cooperative cancel; files already renamed cannot be rolled back, and
    /// the receipt then says exactly which were published.
    fn cancel(&self, export_ref: &str, id: &str) -> Result<Vec<u8>, ErrorCode> {
        let job = self.job(export_ref)?;
        let disposition = if job.result.is_some() {
            "already_terminal"
        } else if job.token.is_cancelled() {
            "already_requested"
        } else {
            job.token.cancel();
            "requested"
        };
        ipc::response(
            id,
            json!({"schemaVersion":1,"export_ref":export_ref,"disposition":disposition}),
        )
    }
}

#[cfg(test)]
mod golden_parity;
#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::{
        JobRequest, Jobs,
        tests::{fixture, full_fixture},
    };
    use lintra::*;
    use std::{
        cell::RefCell,
        collections::VecDeque,
        fs,
        path::Path,
        time::{Duration, Instant},
    };

    const INSTANCE: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    /// ExportResult DTO keys (EXPORT.md).
    const RECEIPT_KEYS: [&str; 11] = [
        "cleanup_failed",
        "destination",
        "durability_confirmed",
        "error",
        "export_ref",
        "format",
        "published_roles",
        "report_ref",
        "run",
        "state",
        "schemaVersion",
    ];

    /// Synthetic native dialogs: queued save answers and confirmation
    /// answers, with every confirmation title recorded.
    #[derive(Default)]
    pub(super) struct Dialogs {
        saves: RefCell<VecDeque<Option<PathBuf>>>,
        answers: RefCell<VecDeque<bool>>,
        asked: RefCell<Vec<String>>,
    }
    impl Dialogs {
        pub(super) fn saving(path: Option<&Path>) -> Self {
            let dialogs = Self::default();
            dialogs
                .saves
                .borrow_mut()
                .push_back(path.map(Path::to_owned));
            dialogs
        }
        fn answering(self, answers: &[bool]) -> Self {
            self.answers.borrow_mut().extend(answers);
            self
        }
    }
    impl ExportDialogs for Dialogs {
        fn save(&self, _: &str) -> Result<Option<PathBuf>, ErrorCode> {
            Ok(self
                .saves
                .borrow_mut()
                .pop_front()
                .expect("unexpected save dialog"))
        }
        fn confirm(&self, title: &str, _: &str) -> Result<bool, ErrorCode> {
            self.asked.borrow_mut().push(title.to_owned());
            Ok(self
                .answers
                .borrow_mut()
                .pop_front()
                .expect("unexpected confirmation"))
        }
    }

    /// Run the production worker to a published report.
    pub(super) fn published(
        intent: Arc<crate::configuration::PreparedIntent>,
        refs: Arc<dyn ReferenceDataPreparer>,
    ) -> (Jobs, RunRef) {
        let mut jobs = Jobs::default();
        jobs.start(INSTANCE, "start_1", "plan_1".into(), intent, refs)
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let bytes = jobs
                .dispatch(JobRequest::State { run: None }, "state")
                .unwrap();
            let state: Value = serde_json::from_slice::<Value>(&bytes).unwrap()["payload"].clone();
            if state["state"] == "terminal" {
                assert_eq!(state["outcome"]["report_available"], true, "{state}");
                return (jobs, RunRef::decode(&state["run"]).unwrap());
            }
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
    }
    /// One export operation; every frame stays a small control DTO with no
    /// path of the private report, the inputs or the chosen destination.
    fn call(
        jobs: &mut Jobs,
        operation: &str,
        payload: Value,
        dialogs: &dyn ExportDialogs,
        forbidden: &[&Path],
    ) -> Result<Value, ErrorCode> {
        let request = ExportRequest::decode(operation, payload.as_object().unwrap())?;
        let bytes = jobs.export(request, dialogs, "test_1")?;
        assert!(bytes.len() <= 4096, "{} bytes", bytes.len());
        let text = String::from_utf8(bytes).unwrap();
        for path in forbidden {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            assert!(!text.contains(&name), "{name} crossed IPC: {text}");
        }
        Ok(serde_json::from_str::<Value>(&text).unwrap()["payload"].clone())
    }
    fn none() -> Dialogs {
        Dialogs::default()
    }
    pub(super) fn about(jobs: &mut Jobs, run: &RunRef) -> Value {
        call(
            jobs,
            "report_about",
            json!({"schemaVersion":1,"run":run.dto()}),
            &none(),
            &[],
        )
        .unwrap()
    }
    pub(super) fn prepare(
        jobs: &mut Jobs,
        report_ref: &Value,
        format: &str,
        dialogs: &Dialogs,
        forbidden: &[&Path],
    ) -> Result<Value, ErrorCode> {
        call(
            jobs,
            "export_prepare",
            json!({"schemaVersion":1,"report_ref":report_ref,"format":format}),
            dialogs,
            forbidden,
        )
    }
    /// Execute a ready plan and poll its owned job to the receipt.
    pub(super) fn execute(jobs: &mut Jobs, export_ref: &Value, forbidden: &[&Path]) -> Value {
        let started = call(
            jobs,
            "export_execute",
            json!({"schemaVersion":1,"export_ref":export_ref}),
            &none(),
            forbidden,
        )
        .unwrap();
        assert_eq!(started["state"], "writing");
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            let state = call(
                jobs,
                "export_state",
                json!({"schemaVersion":1,"export_ref":export_ref}),
                &none(),
                forbidden,
            )
            .unwrap();
            if state["state"] != "writing" {
                let mut keys: Vec<_> = state.as_object().unwrap().keys().cloned().collect();
                keys.sort();
                let mut expected: Vec<String> =
                    RECEIPT_KEYS.iter().map(|k| (*k).to_owned()).collect();
                expected.sort();
                assert_eq!(keys, expected);
                return state;
            }
            assert!(Instant::now() < deadline, "export job did not finish");
            thread::sleep(Duration::from_millis(1));
        }
    }
    /// Bytes the canonical writers produce directly for this report.
    pub(super) fn canonical(report: &Report, format: ExportFormat) -> (Vec<u8>, Vec<u8>) {
        let file = std::io::BufReader::new(fs::File::open(report.spool()).unwrap());
        let token = CancellationToken::default();
        let body = match format {
            ExportFormat::Jsonl => {
                let mut sink = JsonlSink::new(Vec::new());
                replay_canonical_report(file, report.summary(), &mut sink, &token).unwrap();
                sink.finish().unwrap()
            }
            ExportFormat::Csv => {
                let mut sink = CsvSink::new(Vec::new()).unwrap();
                replay_canonical_report(file, report.summary(), &mut sink, &token).unwrap();
                sink.finish().unwrap()
            }
            ExportFormat::Xlsx => unreachable!("XLSX is compared through its cells"),
        };
        let summary =
            write_run_summary_with_bundle(Vec::new(), report.summary(), report.audit()).unwrap();
        (body, summary)
    }

    /// LocalOnly and Full reports produced by the real worker are exported
    /// through the bridge in all three formats: exactly the canonical writer
    /// output plus the summary with bundle provenance, both files published,
    /// local destinations without any extra confirmation.
    #[test]
    fn local_only_and_full_reports_export_canonical_files_through_the_bridge() {
        let (_local_inputs, local, local_refs) = fixture();
        let (_full_inputs, full, full_refs) =
            full_fixture(b"01AANO\n02BBOK\n", b"11AAOK\n12BBOK\n");
        for (mode, intent, refs, traces) in [
            ("local_only", local, local_refs, "1"),
            ("full", full, full_refs, "2"),
        ] {
            let (mut jobs, run) = published(intent, refs);
            let report = jobs.published().unwrap().1;
            let provenance = about(&mut jobs, &run);
            assert_eq!(provenance["mode"], mode);
            assert_eq!(provenance["trace_count"], traces);
            assert_eq!(provenance["exportable"], true);
            assert_eq!(provenance["app_version"], env!("CARGO_PKG_VERSION"));
            assert_eq!(
                provenance["schema_hash"]["text"],
                report.summary().schema_hash
            );
            assert_eq!(
                provenance["bundle"]["bundle_source"],
                report.audit().source()
            );
            assert_eq!(provenance["run"], run.dto());
            // Stable while the same report stays published.
            assert_eq!(
                about(&mut jobs, &run)["report_ref"],
                provenance["report_ref"]
            );
            let report_ref = provenance["report_ref"].clone();
            let out = tempfile::tempdir().unwrap();
            let inputs = report.inputs();
            for format in [ExportFormat::Jsonl, ExportFormat::Csv, ExportFormat::Xlsx] {
                let path = out.path().join(format!("{mode}.{}", format.extension()));
                let dialogs = Dialogs::saving(Some(&path));
                let mut forbidden: Vec<&Path> = inputs.iter().map(ProtectedInput::path).collect();
                forbidden.push(&path);
                forbidden.push(report.private_root());
                let ready = prepare(
                    &mut jobs,
                    &report_ref,
                    format.extension(),
                    &dialogs,
                    &forbidden,
                )
                .unwrap();
                assert!(
                    dialogs.asked.borrow().is_empty(),
                    "no confirmation for local files"
                );
                assert_eq!(ready["state"], "ready");
                assert_eq!(ready["destination"]["local_files"], "2");
                assert_eq!(ready["destination"]["warning_acknowledged"], false);
                let receipt = execute(&mut jobs, &ready["export_ref"], &forbidden);
                assert_eq!(receipt["state"], "succeeded", "{receipt}");
                assert_eq!(receipt["published_roles"], json!(["report", "summary"]));
                assert_eq!(receipt["durability_confirmed"], true);
                assert_eq!(receipt["cleanup_failed"], false);
                assert_eq!(receipt["error"], Value::Null);
                assert_eq!(receipt["report_ref"], report_ref);
                let written = fs::read(&path).unwrap();
                let summary = fs::read(summary_path(&path)).unwrap();
                if format == ExportFormat::Xlsx {
                    assert!(written.starts_with(b"PK"), "XLSX is a zip workbook");
                    assert_eq!(summary, canonical(&report, ExportFormat::Jsonl).1);
                } else {
                    assert_eq!(
                        (written, summary),
                        canonical(&report, format),
                        "{mode} {format:?}"
                    );
                }
            }
        }
    }

    /// Every input of a Full bundle and the private spool are refused as
    /// destinations before any confirmation; overwrite is a separate native
    /// confirmation, also for the summary sidecar alone; cancel and decline
    /// create no plan.
    #[test]
    fn full_inputs_are_protected_and_overwrite_is_a_separate_native_choice() {
        let (inputs_dir, intent, refs) = full_fixture(b"01AAOK\n", b"11AAOK\n");
        // The same bundle with inputs whose extension is an export format.
        let mut intent = Arc::try_unwrap(intent).unwrap_or_else(|_| unreachable!());
        let ValidationScope::Flow(flow) = &mut intent.request.scope else {
            unreachable!()
        };
        for trace in &mut flow.traces {
            let InputSource::Path(path) = &mut trace.source;
            let renamed = path.with_extension("csv");
            fs::rename(&*path, &renamed).unwrap();
            *path = renamed;
        }
        let protected: Vec<PathBuf> = flow
            .traces
            .iter()
            .map(|t| match &t.source {
                InputSource::Path(path) => path.clone(),
            })
            .collect();
        let (mut jobs, run) = published(Arc::new(intent), refs);
        let report = jobs.published().unwrap().1;
        let report_ref = about(&mut jobs, &run)["report_ref"].clone();
        let spool_copy = report.private_root().join("copy.csv");
        for target in protected.iter().chain([&spool_copy]) {
            let dialogs = Dialogs::saving(Some(target));
            let answer = prepare(&mut jobs, &report_ref, "csv", &dialogs, &[]).unwrap();
            assert_eq!(
                answer,
                json!({"schemaVersion":1,"state":"rejected",
                "reason":"destination_not_allowed"})
            );
            assert!(dialogs.asked.borrow().is_empty());
        }
        // The protected inputs were never touched.
        assert_eq!(fs::read(&protected[0]).unwrap(), b"01AAOK\n");
        assert_eq!(fs::read(&protected[1]).unwrap(), b"11AAOK\n");
        let selection =
            prepare(&mut jobs, &report_ref, "csv", &Dialogs::saving(None), &[]).unwrap();
        assert_eq!(selection["reason"], "native_selection_cancelled");

        let out = inputs_dir.path().join("out");
        fs::create_dir(&out).unwrap();
        let path = out.join("report.csv");
        // Only the sidecar exists: still an overwrite, confirmed separately.
        fs::write(summary_path(&path), b"old summary").unwrap();
        let declined = Dialogs::saving(Some(&path)).answering(&[false]);
        let answer = prepare(&mut jobs, &report_ref, "csv", &declined, &[]).unwrap();
        assert_eq!(answer["reason"], "native_confirmation_declined");
        assert_eq!(*declined.asked.borrow(), ["Sostituire i file esistenti?"]);
        assert_eq!(fs::read(summary_path(&path)).unwrap(), b"old summary");
        let accepted = Dialogs::saving(Some(&path)).answering(&[true]);
        let ready = prepare(&mut jobs, &report_ref, "csv", &accepted, &[]).unwrap();
        assert_eq!(ready["state"], "ready");
        let receipt = execute(&mut jobs, &ready["export_ref"], &[]);
        assert_eq!(receipt["state"], "succeeded");
        assert_eq!(
            fs::read(summary_path(&path)).unwrap(),
            canonical(&report, ExportFormat::Csv).1
        );
    }

    /// Codex P410-01 through the bridge, on a real Full run: input T was read
    /// through an alias that becomes unreachable, input S is named like the
    /// summary sidecar of a candidate report. Neither output file may land on
    /// an input in prepare, and execute refuses once the protection is no
    /// longer verifiable; input bytes never change.
    #[cfg(unix)]
    #[test]
    fn inputs_behind_unreachable_aliases_are_never_overwritten() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let (inputs_dir, intent, refs) = full_fixture(b"01AAOK\n", b"11AAOK\n");
        let root = inputs_dir.path();
        let (real, gated) = (root.join("real"), root.join("gated"));
        fs::create_dir(&real).unwrap();
        fs::create_dir(&gated).unwrap();
        symlink(&real, gated.join("alias")).unwrap();
        let mut intent = Arc::try_unwrap(intent).unwrap_or_else(|_| unreachable!());
        let ValidationScope::Flow(flow) = &mut intent.request.scope else {
            unreachable!()
        };
        // S: real/out.csv.summary.json; T: read as gated/alias/T.csv.
        for (trace, name) in flow
            .traces
            .iter_mut()
            .zip(["out.csv.summary.json", "T.csv"])
        {
            let InputSource::Path(path) = &mut trace.source;
            fs::rename(&*path, real.join(name)).unwrap();
            *path = if name == "T.csv" {
                gated.join("alias").join(name)
            } else {
                real.join(name)
            };
        }
        let (mut jobs, run) = published(Arc::new(intent), refs);
        let report_ref = about(&mut jobs, &run)["report_ref"].clone();
        let set_gate =
            |mode| fs::set_permissions(&gated, fs::Permissions::from_mode(mode)).unwrap();
        let unchanged = || {
            assert_eq!(
                fs::read(real.join("out.csv.summary.json")).unwrap(),
                b"01AAOK\n"
            );
            assert_eq!(fs::read(real.join("T.csv")).unwrap(), b"11AAOK\n");
        };
        let rejected =
            json!({"schemaVersion":1,"state":"rejected","reason":"destination_not_allowed"});

        set_gate(0o000);
        assert!(
            fs::canonicalize(gated.join("alias/T.csv")).is_err(),
            "run as non-root"
        );
        for target in [real.join("T.csv"), real.join("out.csv")] {
            // T itself, and a report whose summary sidecar is input S.
            let dialogs = Dialogs::saving(Some(&target)).answering(&[true, true]);
            let answer = prepare(&mut jobs, &report_ref, "csv", &dialogs, &[]).unwrap();
            assert_eq!(answer, rejected, "{target:?}");
            assert!(
                dialogs.asked.borrow().is_empty(),
                "refused before any confirmation"
            );
        }
        set_gate(0o755);
        unchanged();

        // Prepared while verifiable; the alias becomes unreachable before execute.
        let unrelated = real.join("report.csv");
        let ready = prepare(
            &mut jobs,
            &report_ref,
            "csv",
            &Dialogs::saving(Some(&unrelated)),
            &[],
        )
        .unwrap();
        assert_eq!(ready["state"], "ready");
        set_gate(0o000);
        let receipt = execute(&mut jobs, &ready["export_ref"], &[]);
        set_gate(0o755);
        assert_eq!(receipt["state"], "failed", "{receipt}");
        assert_eq!(receipt["published_roles"], json!([]));
        assert!(!unrelated.exists() && !summary_path(&unrelated).exists());
        unchanged();
    }

    /// Handles are bound to the published report and plans are consumed:
    /// no export without a report, a replayed or replaced plan is stale, and
    /// a newer published run makes the old report handle stale.
    #[test]
    fn export_handles_are_bound_to_the_published_report_and_consumed_once() {
        let mut empty = Jobs::default();
        let run = RunRef::decode(&json!({"schemaVersion":1,"instance_id":INSTANCE,"run_id":"1"}))
            .unwrap();
        assert_eq!(
            call(
                &mut empty,
                "report_about",
                json!({"schemaVersion":1,"run":run.dto()}),
                &none(),
                &[]
            ),
            Err(ErrorCode::InvalidState)
        );
        let (_inputs, intent, refs) = fixture();
        let (mut jobs, run) = published(intent.clone(), refs.clone());
        assert_eq!(
            prepare(&mut jobs, &json!("p_unknown"), "jsonl", &none(), &[]),
            Err(ErrorCode::StaleContext)
        );
        let report_ref = about(&mut jobs, &run)["report_ref"].clone();
        let out = tempfile::tempdir().unwrap();
        let path = out.path().join("report.jsonl");
        let first = prepare(
            &mut jobs,
            &report_ref,
            "jsonl",
            &Dialogs::saving(Some(&path)),
            &[],
        )
        .unwrap();
        // A new selection replaces the previous plan.
        let second = prepare(
            &mut jobs,
            &report_ref,
            "jsonl",
            &Dialogs::saving(Some(&path)),
            &[],
        )
        .unwrap();
        let execute_ref = |jobs: &mut Jobs, export_ref: &Value| {
            call(
                jobs,
                "export_execute",
                json!({"schemaVersion":1,"export_ref":export_ref}),
                &none(),
                &[],
            )
        };
        assert_eq!(
            execute_ref(&mut jobs, &first["export_ref"]),
            Err(ErrorCode::StaleContext)
        );
        let receipt = execute(&mut jobs, &second["export_ref"], &[]);
        assert_eq!(receipt["state"], "succeeded");
        // Consumed: no replay of the authorization.
        assert_eq!(
            execute_ref(&mut jobs, &second["export_ref"]),
            Err(ErrorCode::StaleContext)
        );
        assert_eq!(
            call(
                &mut jobs,
                "export_cancel",
                json!({"schemaVersion":1,"export_ref":second["export_ref"]}),
                &none(),
                &[]
            )
            .unwrap()["disposition"],
            "already_terminal"
        );
        // A newer published run: the old report handle is stale.
        jobs.dispatch(JobRequest::Acknowledge { run: run.clone() }, "ack")
            .unwrap();
        jobs.start(INSTANCE, "start_2", "plan_1".into(), intent, refs)
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        while jobs.published().unwrap().0 == run {
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        assert_eq!(
            prepare(
                &mut jobs,
                &report_ref,
                "jsonl",
                &Dialogs::saving(Some(&path)),
                &[]
            ),
            Err(ErrorCode::StaleContext)
        );
        assert_eq!(
            call(
                &mut jobs,
                "report_about",
                json!({"schemaVersion":1,"run":run.dto()}),
                &none(),
                &[]
            ),
            Err(ErrorCode::StaleContext)
        );
        for (operation, payload) in [
            (
                "export_prepare",
                json!({"schemaVersion":1,"report_ref":"p_a","format":"pdf"}),
            ),
            (
                "export_prepare",
                json!({"schemaVersion":1,"report_ref":"p_a","format":"csv","path":"/tmp/x.csv"}),
            ),
            (
                "export_prepare",
                json!({"schemaVersion":1,"report_ref":"p_a","format":"csv","overwrite":true}),
            ),
            (
                "export_execute",
                json!({"schemaVersion":1,"export_ref":"e/1"}),
            ),
            ("export_state", json!({"schemaVersion":1})),
        ] {
            assert_eq!(
                ExportRequest::decode(operation, payload.as_object().unwrap()),
                Err(ErrorCode::InvalidMessage),
                "{payload}"
            );
        }
    }
}
