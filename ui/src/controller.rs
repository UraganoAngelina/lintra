//! UI-thread configuration state; validation execution belongs to the worker.
use crate::{BrowserItem, MainWindow, analysis::*};
use lintra::*;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};
fn strings(values: Vec<String>) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(
        values
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>(),
    ))
}

pub struct ConfigurationController {
    window: slint::Weak<MainWindow>,
    bundle: Option<AnalysisBundle>,
    selection: AnalysisSelection,
    file_confirmed: bool,
    prepared: Option<PreparedAnalysis>,
    directory: PathBuf,
    entries: Vec<BrowserEntry>,
}
impl ConfigurationController {
    pub fn selection(&self) -> &AnalysisSelection {
        &self.selection
    }
    pub fn prepared(&self) -> Option<&PreparedAnalysis> {
        self.prepared.as_ref()
    }
    pub fn take_prepared(&mut self) -> Option<PreparedAnalysis> {
        if let Some(ui) = self.window.upgrade() {
            ui.set_analysis_ready(false);
        }
        self.prepared.take()
    }
    fn refresh(&mut self) {
        self.prepared = None;
        let Some(ui) = self.window.upgrade() else {
            return;
        };
        ui.set_analysis_ready(false);
        ui.set_schema_label(
            self.selection
                .key()
                .map(|key| {
                    format!(
                        "Schema {} / {} · controlli locali",
                        key.flow_id.0, key.version.0
                    )
                })
                .unwrap_or_else(|_| "Nessuno schema selezionato".into())
                .into(),
        );
        let Some(bundle) = &self.bundle else { return };
        if !self.file_confirmed {
            ui.set_configuration_error(false);
            ui.set_status_note(
                if self.selection.file.is_some() {
                    "Premi Invio nel percorso per verificare il file."
                } else {
                    "Scegli il file e la configurazione dello schema."
                }
                .into(),
            );
            return;
        }
        match self.selection.prepare(bundle) {
            Ok(prepared) => {
                ui.set_preview_note(prepared.preview.label().into());
                ui.set_status_note(
                    "Configurazione verificata. L’analisi non è stata avviata.".into(),
                );
                ui.set_configuration_error(false);
                ui.set_analysis_ready(true);
                self.prepared = Some(prepared);
            }
            Err(error) => {
                ui.set_configuration_error(!matches!(error, ConfigurationError::Missing(_)));
                ui.set_status_note(error.to_string().into());
            }
        }
    }
    pub fn select_flow(&mut self, index: i32) {
        let flow = self
            .bundle
            .as_ref()
            .and_then(|b| b.flows().get(index as usize).cloned());
        self.selection.select_flow(flow);
        if let Some(ui) = self.window.upgrade() {
            ui.set_flow_index(if self.selection.flow.is_some() {
                index
            } else {
                -1
            });
            ui.set_version_index(-1);
            ui.set_trace_index(-1);
            ui.set_versions(strings(
                self.selection
                    .flow
                    .as_deref()
                    .map(|f| {
                        self.bundle
                            .as_ref()
                            .map(|b| b.versions(f))
                            .unwrap_or_default()
                    })
                    .unwrap_or_default(),
            ));
            ui.set_traces(strings(vec![]));
        }
        self.refresh();
    }
    pub fn select_version(&mut self, index: i32) {
        let version = self
            .bundle
            .as_ref()
            .zip(self.selection.flow.as_ref())
            .and_then(|(b, f)| b.versions(f).get(index as usize).cloned());
        self.selection.select_version(version);
        if let Some(ui) = self.window.upgrade() {
            ui.set_version_index(if self.selection.version.is_some() {
                index
            } else {
                -1
            });
            ui.set_trace_index(-1);
            ui.set_traces(strings(
                self.selection
                    .key()
                    .ok()
                    .and_then(|key| self.bundle.as_ref().map(|b| b.traces(&key)))
                    .unwrap_or_default(),
            ));
        }
        self.refresh();
    }
    pub fn select_trace(&mut self, index: i32) {
        self.selection.trace = self.selection.key().ok().and_then(|key| {
            self.bundle
                .as_ref()
                .and_then(|b| b.traces(&key).get(index as usize).cloned())
        });
        if let Some(ui) = self.window.upgrade() {
            ui.set_trace_index(if self.selection.trace.is_some() {
                index
            } else {
                -1
            });
        }
        self.refresh();
    }
    pub fn select_encoding(&mut self, index: i32) {
        self.selection.encoding = (index == 0).then_some(RawResolvedEncoding::Utf8);
        if let Some(ui) = self.window.upgrade() {
            ui.set_encoding_index(if index == 0 { 0 } else { -1 });
        }
        self.refresh();
    }
    pub fn edit_file(&mut self, text: String) {
        if let Some(ui) = self.window.upgrade() {
            // Editing invalidates the old probe even before Enter confirms a new
            // path. A previous file's endings must never describe this input.
            ui.set_file_path(text.clone().into());
            ui.set_preview_note("Premi Invio per verificare il percorso.".into());
        }
        self.selection.file = (!text.is_empty()).then(|| PathBuf::from(text));
        self.file_confirmed = false;
        self.refresh();
    }
    pub fn confirm_file(&mut self, text: String) {
        self.edit_file(text.clone());
        let Some(ui) = self.window.upgrade() else {
            return;
        };
        ui.set_file_path(text.into());
        let Some(path) = &self.selection.file else {
            return;
        };
        match InputPreview::inspect(path) {
            Ok(preview) => {
                self.file_confirmed = true;
                ui.set_preview_note(preview.label().into());
                self.refresh();
            }
            Err(error) => {
                ui.set_configuration_error(true);
                ui.set_preview_note(error.to_string().into());
                ui.set_status_note(error.to_string().into());
            }
        }
    }
    pub fn select_reports(&mut self) {
        if let Some(ui) = self.window.upgrade() {
            self.selection.reports = ReportSelection {
                jsonl: ui.get_report_jsonl(),
                xlsx: ui.get_report_xlsx(),
                csv: ui.get_report_csv(),
            };
            if let Some(prepared) = &mut self.prepared {
                prepared.reports = self.selection.reports;
            }
        }
    }
    pub fn open_browser(&mut self, path: &Path) {
        let Some(ui) = self.window.upgrade() else {
            return;
        };
        ui.set_browser_open(true);
        ui.set_browser_index(-1);
        self.entries.clear();
        ui.set_browser_entries(ModelRc::new(VecModel::<BrowserItem>::default()));
        let Some(text) = path.to_str() else {
            ui.set_browser_error("Percorso non UTF-8 non supportato dal selettore.".into());
            return;
        };
        ui.set_browser_directory(text.into());
        match browse_directory(path) {
            Ok(entries) => {
                self.directory = path.to_owned();
                self.entries = entries;
                ui.set_browser_entries(ModelRc::new(VecModel::from(
                    self.entries
                        .iter()
                        .map(|e| BrowserItem {
                            label: e.label.clone().into(),
                            directory: e.directory,
                            selectable: e.selectable,
                        })
                        .collect::<Vec<_>>(),
                )));
                ui.set_browser_error(
                    if self.entries.is_empty() {
                        "La cartella è vuota."
                    } else {
                        ""
                    }
                    .into(),
                );
            }
            Err(_) => ui.set_browser_error(
                "Cartella non accessibile. Inserisci un percorso valido.".into(),
            ),
        }
    }
    pub fn activate_browser_entry(&mut self, index: i32) {
        let Some(entry) = self
            .entries
            .get(index as usize)
            .filter(|e| e.selectable)
            .cloned()
        else {
            return;
        };
        if entry.directory {
            self.open_browser(&entry.path);
            return;
        }
        if let Some(text) = entry.path.to_str() {
            self.confirm_file(text.to_owned());
        }
        if let Some(ui) = self.window.upgrade() {
            ui.set_browser_open(false);
        }
    }
    pub fn parent_directory(&mut self) {
        if let Some(parent) = self.directory.parent().map(Path::to_path_buf) {
            self.open_browser(&parent);
        }
    }
    pub fn cancel_browser(&mut self) {
        if let Some(ui) = self.window.upgrade() {
            ui.set_browser_open(false);
        }
    }
}

/// A weak window handle prevents callback cycles. The controller owns only
/// metadata/intent; no hidden validation thread or record collection is started.
pub fn install_configuration(
    window: &MainWindow,
    bundle: Result<AnalysisBundle, Box<dyn std::error::Error + Send + Sync>>,
) -> Rc<RefCell<ConfigurationController>> {
    let bundle = match bundle {
        Ok(bundle) => {
            window.set_flows(strings(bundle.flows()));
            window.set_origin_label(bundle.origin_label().into());
            window.set_bundle_ready(!bundle.entries().is_empty());
            if bundle.entries().is_empty() {
                window.set_status_note("Nessuno schema disponibile nel bundle selezionato.".into());
            }
            Some(bundle)
        }
        Err(error) => {
            window.set_origin_label("Dati non disponibili".into());
            window.set_status_note(
                format!("Impossibile acquisire i dati configurati: {error}").into(),
            );
            window.set_configuration_error(true);
            None
        }
    };
    let state = Rc::new(RefCell::new(ConfigurationController {
        window: window.as_weak(),
        bundle,
        selection: AnalysisSelection::default(),
        file_confirmed: false,
        prepared: None,
        directory: PathBuf::new(),
        entries: vec![],
    }));
    let c = state.clone();
    window.on_choose_flow(move |i| c.borrow_mut().select_flow(i));
    let c = state.clone();
    window.on_choose_version(move |i| c.borrow_mut().select_version(i));
    let c = state.clone();
    window.on_choose_trace(move |i| c.borrow_mut().select_trace(i));
    let c = state.clone();
    window.on_choose_encoding(move |i| c.borrow_mut().select_encoding(i));
    let c = state.clone();
    window.on_file_edited(move |s| c.borrow_mut().edit_file(s.into()));
    let c = state.clone();
    window.on_file_confirmed(move |s| c.borrow_mut().confirm_file(s.into()));
    let c = state.clone();
    window.on_reports_changed(move || c.borrow_mut().select_reports());
    let c = state.clone();
    window.on_browse(move || {
        let home =
            std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from);
        let selected = c
            .borrow()
            .selection
            .file
            .as_ref()
            .and_then(|p| p.parent())
            .filter(|p| !p.as_os_str().is_empty())
            .map(Path::to_path_buf);
        if let Some(path) = selected.or(home) {
            c.borrow_mut().open_browser(&path);
        }
    });
    let c = state.clone();
    window.on_browser_directory_chosen(move |s| c.borrow_mut().open_browser(Path::new(s.as_str())));
    let c = state.clone();
    window.on_browser_activate(move |i| c.borrow_mut().activate_browser_entry(i));
    let c = state.clone();
    window.on_browser_parent(move || c.borrow_mut().parent_directory());
    let c = state.clone();
    window.on_browser_cancel(move || c.borrow_mut().cancel_browser());
    state
}
