//! UI-thread state and one owned export job. A dialog captures the selected
//! completed run; editing filters/configuration cannot retarget an active save.
use crate::{
    MainWindow,
    export::{ExportError, ExportFailure, ExportFormat, ExportJob, ExportReceipt},
    results::controller::ResultsController,
    worker::AnalysisReport,
};
use lintra::ValidationSummary;
use slint::{ComponentHandle, Timer, TimerMode};
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Arc, time::Duration};
struct Selection {
    report: Arc<AnalysisReport>,
    summary: ValidationSummary,
    format: ExportFormat,
}
pub struct ExportController {
    window: slint::Weak<MainWindow>,
    results: Rc<RefCell<ResultsController>>,
    selection: Option<Selection>,
    job: Option<ExportJob>,
    timer: Timer,
    failure: Option<ExportFailure>,
    receipt: Option<ExportReceipt>,
}
impl ExportController {
    pub fn last_failure(&self) -> Option<&ExportFailure> {
        self.failure.as_ref()
    }
    pub fn last_receipt(&self) -> Option<&ExportReceipt> {
        self.receipt.as_ref()
    }
    pub fn open(&mut self) {
        let Some(ui) = self.window.upgrade() else {
            return;
        };
        if self.job.is_some() {
            ui.set_export_open(true);
            return;
        }
        if ui.get_busy() || !ui.get_results_available() {
            return;
        }
        let Some((report, summary, reports)) = self.results.borrow().displayed_analysis() else {
            return;
        };
        let format = ExportFormat::preferred(reports);
        let path = report
            .input_path()
            .parent()
            .map(|p| p.join(format!("lintra-report.{}", format.extension())))
            .and_then(|p| p.to_str().map(str::to_owned))
            .unwrap_or_default();
        ui.set_export_path(path.into());
        ui.set_export_format(match format {
            ExportFormat::Xlsx => 0,
            ExportFormat::Jsonl => 1,
            ExportFormat::Csv => 2,
        });
        let scope = match &summary.scope_summary {
            lintra::ScopeSummary::Trace { trace_id } => format!("tracciato {}", trace_id.0),
            lintra::ScopeSummary::Flow { .. } => "insieme di tracciati".into(),
        };
        ui.set_export_run(
            format!(
                "{} / {} · {} · {} · {} {}{}",
                summary.schema_key.flow_id.0,
                summary.schema_key.version.0,
                scope,
                if summary.validation_mode == lintra::ValidationMode::LocalOnly {
                    "controlli locali"
                } else {
                    "Full"
                },
                summary.total_errors,
                if summary.total_errors == 1 {
                    "diagnostica"
                } else {
                    "diagnostiche"
                },
                if summary.completeness() == lintra::ValidationCompleteness::Partial {
                    " · verifiche parziali"
                } else {
                    ""
                }
            )
            .into(),
        );
        ui.set_export_note("Verranno salvati il report e un riepilogo JSON con versioni e provenienza. I filtri della tabella non limitano l’esportazione.".into());
        ui.set_export_error(false);
        ui.set_export_overwrite(false);
        ui.set_export_confirmation_required(false);
        ui.set_show_info(false);
        ui.set_export_open(true);
        self.selection = Some(Selection {
            report: report.clone(),
            summary,
            format,
        });
        self.failure = None;
        self.receipt = None;
    }
    pub fn choose_format(&mut self, index: i32) {
        if self.job.is_some() {
            return;
        }
        let Some(ui) = self.window.upgrade() else {
            return;
        };
        let Some(selection) = &mut self.selection else {
            return;
        };
        let format = match index {
            0 => ExportFormat::Xlsx,
            1 => ExportFormat::Jsonl,
            2 => ExportFormat::Csv,
            _ => return,
        };
        selection.format = format;
        ui.set_export_format(index);
        let text = ui.get_export_path();
        if !text.is_empty() {
            let mut path = PathBuf::from(text.as_str());
            path.set_extension(format.extension());
            if let Some(text) = path.to_str() {
                ui.set_export_path(text.into());
            }
        }
        self.path_changed();
    }
    pub fn path_changed(&self) {
        if let Some(ui) = self.window.upgrade() {
            ui.set_export_overwrite(false);
            ui.set_export_confirmation_required(false);
            ui.set_export_error(false);
        }
    }
    pub fn save(&mut self) {
        if self.job.is_some() {
            return;
        }
        let Some(ui) = self.window.upgrade() else {
            return;
        };
        let Some(selection) = &self.selection else {
            return;
        };
        self.failure = None;
        self.receipt = None;
        match ExportJob::start(
            selection.report.clone(),
            selection.summary.clone(),
            selection.format,
            PathBuf::from(ui.get_export_path().as_str()),
            ui.get_export_overwrite(),
        ) {
            Ok(job) => {
                self.job = Some(job);
                let window = self.window.clone();
                self.timer
                    .start(TimerMode::Repeated, Duration::from_millis(40), move || {
                        if let Some(ui) = window.upgrade() {
                            ui.invoke_export_poll();
                        }
                    });
                ui.set_export_busy(true);
                ui.set_export_error(false);
                ui.set_export_note(
                    "Salvataggio in corso. Attendi la conferma prima di usare i file.".into(),
                );
            }
            Err(error) => self.show_failure(error.into()),
        }
    }
    fn show_failure(&mut self, failure: ExportFailure) {
        if let Some(ui) = self.window.upgrade() {
            ui.set_export_busy(false);
            ui.set_export_error(!failure.cancelled);
            ui.set_export_confirmation_required(matches!(
                failure.cause,
                ExportError::OverwriteRequired
            ));
            ui.set_export_note(failure.to_string().into());
        }
        self.failure = Some(failure);
    }
    pub fn poll(&mut self) {
        let Some(result) = self.job.as_mut().and_then(ExportJob::try_result) else {
            return;
        };
        self.job.take();
        self.timer.stop();
        match result {
            Err(failure) => self.show_failure(failure),
            Ok(receipt) => {
                if let Some(ui) = self.window.upgrade() {
                    ui.set_export_busy(false);
                    ui.set_export_error(false);
                    ui.set_export_confirmation_required(false);
                    ui.set_export_overwrite(false);
                    ui.set_export_note(
                        format!(
                            "Salvati:\n{}",
                            receipt
                                .files
                                .iter()
                                .map(|p| p.display().to_string())
                                .collect::<Vec<_>>()
                                .join("\n")
                        )
                        .into(),
                    );
                }
                self.receipt = Some(receipt);
            }
        }
    }
    pub fn close(&mut self) {
        if let Some(job) = &self.job {
            job.cancel();
            if let Some(ui) = self.window.upgrade() {
                ui.set_export_note(
                    "Annullamento richiesto. Attendi l’esito del salvataggio.".into(),
                );
            }
        } else {
            self.selection = None;
            if let Some(ui) = self.window.upgrade() {
                ui.set_export_open(false);
            }
        }
    }
    pub fn refresh_information(&self) {
        let Some(ui) = self.window.upgrade() else {
            return;
        };
        if ui.get_show_results()
            && let Some((report, summary, _)) = self.results.borrow().displayed_analysis()
        {
            ui.set_info_schema(
                format!(
                    "Schema dell’analisi visualizzata: {} / {}",
                    summary.schema_key.flow_id.0, summary.schema_key.version.0
                )
                .into(),
            );
            ui.set_info_bundle(
                format!(
                    "Bundle dell’analisi: {} · versione {}",
                    if report.audit().source() == "embedded" {
                        "incorporato"
                    } else {
                        "esterno"
                    },
                    report.audit().bundle_version().unwrap_or("non dichiarata")
                )
                .into(),
            );
            return;
        }
        ui.set_info_schema(format!("Schema selezionato: {}", ui.get_schema_label()).into());
        ui.set_info_bundle(ui.get_origin_label());
    }
    pub fn shutdown(&mut self) {
        self.timer.stop();
        if let Some(job) = self.job.take() {
            job.cancel();
            drop(job);
        }
        self.selection = None;
    }
}
pub fn install_exports(
    ui: &MainWindow,
    results: Rc<RefCell<ResultsController>>,
) -> Rc<RefCell<ExportController>> {
    let controller = Rc::new(RefCell::new(ExportController {
        window: ui.as_weak(),
        results,
        selection: None,
        job: None,
        timer: Timer::default(),
        failure: None,
        receipt: None,
    }));
    let weak = Rc::downgrade(&controller);
    ui.on_request_export(move || {
        if let Some(c) = weak.upgrade() {
            c.borrow_mut().open();
        }
    });
    let weak = Rc::downgrade(&controller);
    ui.on_choose_export_format(move |i| {
        if let Some(c) = weak.upgrade() {
            c.borrow_mut().choose_format(i);
        }
    });
    let weak = Rc::downgrade(&controller);
    ui.on_export_path_edited(move || {
        if let Some(c) = weak.upgrade() {
            c.borrow().path_changed();
        }
    });
    let weak = Rc::downgrade(&controller);
    ui.on_confirm_export(move || {
        if let Some(c) = weak.upgrade() {
            c.borrow_mut().save();
        }
    });
    let weak = Rc::downgrade(&controller);
    ui.on_close_export(move || {
        if let Some(c) = weak.upgrade() {
            c.borrow_mut().close();
        }
    });
    let weak = Rc::downgrade(&controller);
    ui.on_export_poll(move || {
        if let Some(c) = weak.upgrade() {
            c.borrow_mut().poll();
        }
    });
    let weak = Rc::downgrade(&controller);
    ui.on_refresh_information(move || {
        if let Some(c) = weak.upgrade() {
            c.borrow().refresh_information();
        }
    });
    controller
}
