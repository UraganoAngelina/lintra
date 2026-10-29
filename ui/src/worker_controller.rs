//! Slint thread bridge. Worker notifications only queue an event-loop callback;
//! Rc controllers and component properties are accessed exclusively by the UI.
use crate::{MainWindow, controller::ConfigurationController, worker::*};
use lintra::{
    CancellationToken, RunStatus, ValidationCompleteness, ValidationPhase, ValidationRunOutcome,
};
use slint::ComponentHandle;
use std::{cell::RefCell, rc::Rc, sync::Arc};

pub struct WorkerController {
    window: slint::Weak<MainWindow>,
    configuration: Rc<RefCell<ConfigurationController>>,
    active: Option<WorkerHandle>,
    cancel_requested: bool,
    last_outcome: Option<ValidationRunOutcome>,
    last_completed: Option<CompletedAnalysis>,
    last_error: Option<WorkerError>,
}
impl WorkerController {
    /// Typed technical cause of the latest failed run, separate from diagnostics.
    pub fn last_error(&self) -> Option<&WorkerError> {
        self.last_error.as_ref()
    }
    /// Last controlled outcome, including Cancelled. A technical failure clears
    /// this slot; it never reuses an old successful status as the current result.
    pub fn last_outcome(&self) -> Option<&ValidationRunOutcome> {
        self.last_outcome.as_ref()
    }
    /// Last complete report remains owned across failed/cancelled retries. This
    /// is explicitly the LAST completed analysis, not proof the latest run passed.
    pub fn last_completed(&self) -> Option<&CompletedAnalysis> {
        self.last_completed.as_ref()
    }
    pub fn start(&mut self) {
        if self.active.is_some() {
            return;
        }
        let Some(ui) = self.window.upgrade() else {
            return;
        };
        let Some(prepared) = self.configuration.borrow().prepared().cloned() else {
            ui.set_configuration_error(true);
            ui.set_status_note("Verifica file, schema ed encoding prima di avviare.".into());
            return;
        };
        ui.set_preview_note(format!("Controllo iniziale: {}", prepared.preview.label()).into());
        let window = self.window.clone();
        let notify = Arc::new(move || {
            // Weak is Send/Sync; upgrading and all Rc/model access happen only
            // after Slint dispatches this closure on its native event loop.
            let _ = window.upgrade_in_event_loop(|ui| ui.invoke_worker_poll());
        });
        match WorkerHandle::start(prepared, CancellationToken::default(), notify) {
            Ok(worker) => {
                self.active = Some(worker);
                self.cancel_requested = false;
                self.last_outcome = None;
                self.last_error = None;
                ui.set_validation_invalid(false);
                ui.set_execution_partial(false);
                ui.set_busy(true);
                ui.set_results_available(false);
                ui.set_analysis_ready(false);
                ui.set_configuration_error(false);
                ui.set_execution_label("Analisi avviata".into());
                ui.set_execution_detail("In attesa del primo aggiornamento.".into());
                ui.set_status_note("Analisi in corso.".into());
            }
            Err(error) => self.show_error(&ui, error),
        }
    }
    pub fn cancel(&mut self) {
        if let Some(worker) = &self.active {
            worker.cancel();
            self.cancel_requested = true;
            if let Some(ui) = self.window.upgrade() {
                ui.set_execution_label("Annullamento richiesto".into());
                ui.set_status_note("Attendi l’esito dell’analisi.".into());
            }
        }
    }
    fn show_error(&mut self, ui: &MainWindow, error: WorkerError) {
        self.last_outcome = None;
        ui.set_results_available(false);
        ui.set_validation_invalid(false);
        ui.set_execution_partial(false);
        ui.set_busy(false);
        ui.set_analysis_ready(self.configuration.borrow().prepared().is_some());
        ui.set_configuration_error(true);
        ui.set_execution_label("Errore tecnico".into());
        ui.set_preview_note("Controllo del file da ripetere: premi Invio nel percorso.".into());
        ui.set_execution_detail("Controlla il file e la configurazione, poi riprova.".into());
        // Display is intentionally redacted; typed causes remain in WorkerError,
        // rather than being mixed into regulatory diagnostics or a fake outcome.
        ui.set_status_note(error.to_string().into());
        self.last_error = Some(error);
    }
    pub fn poll(&mut self) {
        let Some(ui) = self.window.upgrade() else {
            return;
        };
        if let Some(progress) = self.active.as_ref().and_then(WorkerHandle::try_progress) {
            if !self.cancel_requested {
                let label = match progress.phase {
                    None => "Analisi avviata",
                    Some(ValidationPhase::Preflight) => "Controllo preliminare",
                    Some(ValidationPhase::LocalValidation) => "Validazione locale",
                    Some(ValidationPhase::CrossRecordValidation) => "Validazione tra record",
                    Some(ValidationPhase::FinalizingReport) => "Finalizzazione report",
                };
                ui.set_execution_label(label.into());
            }
            let count = if progress.phase == Some(ValidationPhase::CrossRecordValidation) {
                format!(
                    "Record elaborati nella verifica tra record: {}",
                    progress.cross_records
                )
            } else if let Some(trace) = progress.trace {
                format!(
                    "Record nel tracciato {}: {}",
                    trace.0, progress.trace_records
                )
            } else {
                "In attesa dei contatori del tracciato.".into()
            };
            ui.set_execution_detail(count.into());
        }
        let Some(result) = self.active.as_mut().and_then(WorkerHandle::try_result) else {
            return;
        };
        // Terminal is delivered once validation and atomic publication are over.
        // Joining an exited thread cannot wait on a UI consumer: both channels
        // use try_send and the notification merely posts an event-loop action.
        self.active.take();
        match result {
            Err(error) => self.show_error(&ui, error),
            Ok(completed) => {
                let outcome = &completed.outcome;
                ui.set_validation_invalid(outcome.status == RunStatus::CompletedInvalid);
                ui.set_execution_partial(
                    outcome.summary.completeness() == ValidationCompleteness::Partial,
                );
                ui.set_busy(false);
                ui.set_analysis_ready(self.configuration.borrow().prepared().is_some());
                ui.set_configuration_error(false);
                let (label, note) = match outcome.status {
                    RunStatus::Cancelled => (
                        "Analisi annullata",
                        "Nessun report completo pubblicato per questa analisi.",
                    ),
                    _ if outcome.summary.truncated => (
                        "Analisi parziale",
                        "Raggiunto il limite operativo; nessun report completo pubblicato.",
                    ),
                    RunStatus::CompletedInvalid => (
                        "Analisi completata con violazioni",
                        "Le violazioni riguardano l’ambito richiesto; consulta il riepilogo.",
                    ),
                    RunStatus::CompletedValid
                        if outcome.summary.completeness() == ValidationCompleteness::Partial =>
                    {
                        (
                            "Analisi completata, verifiche parziali",
                            "Alcuni controlli non hanno un esito conclusivo.",
                        )
                    }
                    RunStatus::CompletedValid => (
                        "Analisi completata",
                        "Nessuna violazione nell’ambito richiesto.",
                    ),
                };
                ui.set_execution_label(label.into());
                ui.set_status_note(note.into());
                ui.set_execution_detail(
                    format!(
                        "Record elaborati: {} · errori: {} · tempo: {:.2} s · completezza: {}",
                        outcome.summary.records_processed,
                        outcome.summary.total_errors,
                        outcome.summary.elapsed.as_secs_f64(),
                        if outcome.summary.completeness() == ValidationCompleteness::Partial {
                            "parziale"
                        } else {
                            "completa nell’ambito richiesto"
                        }
                    )
                    .into(),
                );
                self.last_outcome = Some(outcome.clone());
                ui.set_results_available(completed.report.is_some());
                if completed.report.is_some() {
                    self.last_completed = Some(completed);
                }
            }
        }
    }
    /// Called after window.run returns, not while processing UI events. Cancel
    /// and join ensure owned temp files are cleaned before the process exits.
    pub fn shutdown(&mut self) {
        self.active.take();
    }
}

pub fn install_worker(
    window: &MainWindow,
    configuration: Rc<RefCell<ConfigurationController>>,
) -> Rc<RefCell<WorkerController>> {
    window.set_worker_available(true);
    let state = Rc::new(RefCell::new(WorkerController {
        window: window.as_weak(),
        configuration,
        active: None,
        cancel_requested: false,
        last_outcome: None,
        last_completed: None,
        last_error: None,
    }));
    let c = state.clone();
    window.on_start_analysis(move || c.borrow_mut().start());
    let c = state.clone();
    window.on_cancel_analysis(move || c.borrow_mut().cancel());
    let c = state.clone();
    window.on_worker_poll(move || c.borrow_mut().poll());
    state
}
