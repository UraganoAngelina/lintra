//! UI-owned selection and bounded background index job. No diagnostic collection
//! or full JSONL scan is performed by an event-loop callback.
use super::{JsonlModel, ReportIndex, ResultsError, ResultsFilter};
use crate::{MainWindow, worker::AnalysisReport, worker_controller::WorkerController};
use lintra::ValidationSummary;
use slint::{ComponentHandle, Model, ModelRc, Timer, TimerMode};
use std::{
    cell::RefCell,
    fs::File,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

struct IndexJob {
    receiver: Receiver<Result<ReportIndex, ResultsError>>,
    cancelled: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}
impl IndexJob {
    fn start(
        report: Arc<AnalysisReport>,
        summary: ValidationSummary,
        filter: ResultsFilter,
    ) -> Result<Self, ResultsError> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        let token = cancelled.clone();
        let handle = thread::Builder::new()
            .name("lintra-results-index".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let file = File::open(report.jsonl_path())?;
                    ReportIndex::scan(file, &summary, &filter, &token)
                        .map(|index| index.with_owner(report.clone()))
                }))
                .unwrap_or(Err(ResultsError::WorkerUnavailable));
                // Terminal channel has a single producer and exactly one message.
                // Never block on a closed UI or accidentally create an error backlog.
                let _ = sender.try_send(result);
            })
            .map_err(|_| ResultsError::WorkerUnavailable)?;
        Ok(Self {
            receiver,
            cancelled,
            handle: Some(handle),
        })
    }
}
impl Drop for IndexJob {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

pub struct ResultsController {
    window: slint::Weak<MainWindow>,
    worker: Rc<RefCell<WorkerController>>,
    model: Option<Rc<JsonlModel>>,
    report: Option<Arc<AnalysisReport>>,
    summary: Option<ValidationSummary>,
    selected_offset: Option<u64>,
    job: Option<IndexJob>,
    timer: Timer,
}
impl ResultsController {
    pub fn model(&self) -> Option<&Rc<JsonlModel>> {
        self.model.as_ref()
    }
    fn error(&self, error: &ResultsError) {
        if let Some(ui) = self.window.upgrade() {
            ui.set_results_loading(false);
            ui.set_results_error(true);
            ui.set_results_note(error.to_string().into());
        }
    }
    pub fn open(&mut self) {
        let Some(ui) = self.window.upgrade() else {
            return;
        };
        if ui.get_busy() || !ui.get_results_available() {
            return;
        }
        let worker = self.worker.borrow();
        let Some(completed) = worker.last_completed() else {
            return;
        };
        let Some(report) = &completed.report else {
            return;
        };
        let report = report.clone();
        let summary = completed.outcome.summary.clone();
        drop(worker);
        ui.set_show_results(true);
        // Reopening the same immutable analysis preserves its filters, viewport
        // selection and cache; a new Arc identifies a genuinely new result.
        if self
            .report
            .as_ref()
            .is_some_and(|old| Arc::ptr_eq(old, &report))
        {
            return;
        }
        self.job.take();
        self.model = None;
        self.selected_offset = None;
        ui.set_result_rows(ModelRc::default());
        ui.set_selected_result(-1);
        ui.set_result_detail(Default::default());
        ui.set_filter_code("".into());
        ui.set_filter_field("".into());
        ui.set_filter_record("".into());
        ui.set_filter_fiscal("".into());
        ui.set_filter_line("".into());
        ui.set_results_heading(
            match completed_status(&summary) {
                CompletedStatus::Invalid => "Analisi con violazioni",
                CompletedStatus::Partial => "Analisi con verifiche parziali",
                CompletedStatus::Valid => "Analisi completata",
            }
            .into(),
        );
        ui.set_results_invalid(summary.run_status == lintra::RunStatus::CompletedInvalid);
        let completeness = if summary.completeness() == lintra::ValidationCompleteness::Partial {
            "parziali"
        } else {
            "complete nell’ambito richiesto"
        };
        let mode = match summary.validation_mode {
            lintra::ValidationMode::LocalOnly => "Controlli locali",
            lintra::ValidationMode::Full => "Controlli completi",
        };
        ui.set_results_summary(format!("{mode}. Record: {}   Non validi: {}   Errori: {}   Tempo: {:.2} s   Verifiche: {completeness}",
            summary.records_processed, summary.invalid_records, summary.total_errors,
            summary.elapsed.as_secs_f64()).into());
        let scope = match &summary.scope_summary {
            lintra::ScopeSummary::Trace { trace_id } => format!("Tracciato {}", trace_id.0),
            lintra::ScopeSummary::Flow { trace_ids } => format!(
                "Tracciati {}",
                trace_ids
                    .iter()
                    .map(|id| id.0.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        };
        let source = if report.audit().source() == "embedded" {
            "Dati incorporati"
        } else {
            "Dati esterni"
        };
        ui.set_results_scope(
            format!(
                "{} / {} / {scope} / {mode} / {source} / schema {}",
                summary.schema_key.flow_id.0, summary.schema_key.version.0, summary.schema_hash
            )
            .into(),
        );
        self.report = Some(report);
        self.summary = Some(summary);
        self.apply();
    }
    pub fn apply(&mut self) {
        if self.job.is_some() {
            return;
        }
        let Some(ui) = self.window.upgrade() else {
            return;
        };
        let line = match ResultsFilter::parse_line(&ui.get_filter_line()) {
            Ok(line) => line,
            Err(error) => {
                self.error(&error);
                return;
            }
        };
        let (Some(report), Some(summary)) = (&self.report, &self.summary) else {
            return;
        };
        let filter = ResultsFilter {
            code: ui.get_filter_code().to_string(),
            field: ui.get_filter_field().to_string(),
            record: ui.get_filter_record().to_string(),
            fiscal_code: ui.get_filter_fiscal().to_string(),
            line,
        };
        match IndexJob::start(report.clone(), summary.clone(), filter) {
            Ok(job) => {
                self.job = Some(job);
                ui.set_results_loading(true);
                ui.set_results_error(false);
                ui.set_results_note("Lettura del report e applicazione dei filtri…".into());
            }
            Err(error) => self.error(&error),
        }
    }
    pub fn select(&mut self, index: i32) {
        if index < 0 {
            return;
        }
        let Some(model) = &self.model else {
            return;
        };
        let index = index as usize;
        match model.read_row(index) {
            Ok(Some(value)) => {
                self.selected_offset = model.offset_at(index);
                if let Some(ui) = self.window.upgrade() {
                    ui.set_selected_result(index as i32);
                    ui.set_result_detail(value);
                }
            }
            Ok(None) => {}
            Err(error) => self.error(&error),
        }
    }
    pub fn poll(&mut self) {
        if self.job.is_none() && self.model.as_ref().is_some_and(|v| v.failed()) {
            self.error(&ResultsError::Malformed);
        }
        let result = match self.job.as_ref().map(|job| job.receiver.try_recv()) {
            Some(Ok(value)) => value,
            Some(Err(mpsc::TryRecvError::Disconnected)) => Err(ResultsError::WorkerUnavailable),
            _ => return,
        };
        self.job.take();
        match result {
            Err(error) => {
                // A failed candidate must not leave old rows looking as though
                // they satisfied the newly entered filter or a corrupt report.
                self.model = None;
                self.selected_offset = None;
                if let Some(ui) = self.window.upgrade() {
                    ui.set_result_rows(ModelRc::default());
                    ui.set_selected_result(-1);
                    ui.set_result_detail(Default::default());
                }
                self.error(&error);
            }
            Ok(index) => {
                let total = index.total();
                let mut model = JsonlModel::new(index);
                if let Some(report) = &self.report {
                    model.retain_report(report.clone());
                }
                let model = Rc::new(model);
                let selection = self
                    .selected_offset
                    .and_then(|offset| model.position_of(offset));
                let count = model.row_count();
                if let Some(ui) = self.window.upgrade() {
                    ui.set_result_rows(ModelRc::from(model.clone()));
                    ui.set_selected_result(-1);
                    ui.set_result_detail(Default::default());
                    ui.set_results_loading(false);
                    ui.set_results_error(false);
                    ui.set_results_note(if total == 0 {
                        "Nessuna violazione nell’ambito richiesto. Le verifiche eseguite sono indicate nel riepilogo.".into()
                    } else if count == 0 {
                        "Nessuna diagnostica corrisponde ai filtri. Modifica i criteri e applicali di nuovo.".into()
                    } else {
                        format!("{count} diagnostiche di {total}. Ordine originale del JSONL; filtri esatti combinati.").into()
                    });
                }
                self.model = Some(model);
                self.selected_offset = None;
                if let Some(row) = selection {
                    self.select(row as i32);
                }
            }
        }
    }
    pub fn shutdown(&mut self) {
        self.timer.stop();
        self.job.take();
        // Close readers before releasing the last report owner on Windows.
        self.model = None;
        if let Some(ui) = self.window.upgrade() {
            ui.set_result_rows(ModelRc::default());
        }
        self.report = None;
    }
}
enum CompletedStatus {
    Invalid,
    Partial,
    Valid,
}
fn completed_status(summary: &ValidationSummary) -> CompletedStatus {
    if summary.run_status == lintra::RunStatus::CompletedInvalid {
        CompletedStatus::Invalid
    } else if summary.completeness() == lintra::ValidationCompleteness::Partial {
        CompletedStatus::Partial
    } else {
        CompletedStatus::Valid
    }
}

pub fn install_results(
    ui: &MainWindow,
    worker: Rc<RefCell<WorkerController>>,
) -> Rc<RefCell<ResultsController>> {
    let state = Rc::new(RefCell::new(ResultsController {
        window: ui.as_weak(),
        worker,
        model: None,
        report: None,
        summary: None,
        selected_offset: None,
        job: None,
        timer: Timer::default(),
    }));
    // Timer holds a Weak, preventing a controller/timer ownership cycle. Work is
    // bounded polling of one result slot; scans never run on the UI thread.
    let weak = Rc::downgrade(&state);
    state
        .borrow()
        .timer
        .start(TimerMode::Repeated, Duration::from_millis(50), move || {
            if let Some(state) = weak.upgrade() {
                state.borrow_mut().poll();
            }
        });
    let c = state.clone();
    ui.on_open_results(move || c.borrow_mut().open());
    let c = state.clone();
    ui.on_apply_results_filter(move || c.borrow_mut().apply());
    let c = state.clone();
    ui.on_select_result(move |index| c.borrow_mut().select(index));
    state
}
