//! Disk-backed diagnostic browsing using the shared library index. Slint only
//! projects requested rows; the library owns offsets and the bounded cache.
pub mod controller;

use lintra::{JsonlReportIndex, ValidationError, ValidationSummary};
pub use lintra::{ReportFilter as ResultsFilter, ReportIndexError as ResultsError};
use slint::{Model, ModelNotify, ModelTracker};
use std::{
    cell::{Cell, RefCell},
    fs::File,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Slint retains its report owner alongside the shared index, including while
/// an indexed candidate is still in the worker channel. File drops before owner.
pub struct ReportIndex {
    inner: JsonlReportIndex,
    owner: Option<Arc<crate::worker::AnalysisReport>>,
}
impl ReportIndex {
    pub fn scan(
        file: File,
        summary: &ValidationSummary,
        filter: &ResultsFilter,
        cancelled: &AtomicBool,
    ) -> Result<Self, ResultsError> {
        Ok(Self {
            inner: JsonlReportIndex::scan(file, summary, filter, || {
                cancelled.load(Ordering::Acquire)
            })?,
            owner: None,
        })
    }
    fn with_owner(mut self, report: Arc<crate::worker::AnalysisReport>) -> Self {
        self.owner = Some(report);
        self
    }
    pub fn row_count(&self) -> usize {
        self.inner.row_count()
    }
    pub fn total(&self) -> u64 {
        self.inner.total()
    }
    pub fn offset_bytes(&self) -> usize {
        self.inner.offset_bytes()
    }
}

/// Slint requests visible rows. The shared index handles repeated layout reads
/// with its bounded diagnostic cache; selected detail is held by the controller.
pub struct JsonlModel {
    index: RefCell<ReportIndex>,
    notify: ModelNotify,
    failed: Cell<bool>,
}
impl JsonlModel {
    pub fn new(index: ReportIndex) -> Self {
        Self {
            index: RefCell::new(index),
            notify: ModelNotify::default(),
            failed: Cell::new(false),
        }
    }
    pub fn retain_report(&mut self, report: Arc<crate::worker::AnalysisReport>) {
        self.index.get_mut().owner = Some(report);
    }
    pub fn failed(&self) -> bool {
        self.failed.get()
    }
    pub fn cache_len(&self) -> usize {
        self.index.borrow().inner.cache_len()
    }
    pub fn decoded_rows(&self) -> u64 {
        self.index.borrow().inner.decoded_rows()
    }
    pub fn highest_requested_row(&self) -> usize {
        self.index.borrow().inner.highest_requested_row()
    }
    pub fn offset_at(&self, row: usize) -> Option<u64> {
        self.index.borrow().inner.offset_at(row)
    }
    pub fn position_of(&self, offset: u64) -> Option<usize> {
        self.index.borrow().inner.position_of(offset)
    }
    pub fn read_row(&self, row: usize) -> Result<Option<crate::ErrorRow>, ResultsError> {
        self.index
            .borrow_mut()
            .inner
            .read_row(row)?
            .map(project)
            .transpose()
    }
}
impl Model for JsonlModel {
    type Data = crate::ErrorRow;
    fn row_count(&self) -> usize {
        self.index.borrow().row_count()
    }
    fn row_data(&self, row: usize) -> Option<Self::Data> {
        match self.read_row(row) {
            Ok(value) => value,
            Err(_) => {
                self.failed.set(true);
                Some(crate::ErrorRow {
                    message: "Errore di lettura del report".into(),
                    ..Default::default()
                })
            }
        }
    }
    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }
}
fn diagnostic_value(value: &lintra::DiagnosticValue) -> String {
    match &value.raw_hex {
        Some(raw) => format!("{} (hex: {raw})", value.display.replace(' ', "␠")),
        None => value.display.replace(' ', "␠"),
    }
}
fn project(error: ValidationError) -> Result<crate::ErrorRow, ResultsError> {
    let mut context: Vec<String> = error
        .context
        .iter()
        .map(|v| {
            format!(
                "{}.{} = {}",
                v.trace_id.0,
                v.field_id.0,
                diagnostic_value(&v.value)
            )
        })
        .collect();
    if let Some(value) = error.record_identity.fiscal_code {
        context.push(format!("Codice fiscale: {value}"));
    }
    if let Some(value) = error.record_identity.episode_id {
        context.push(format!("Episodio: {value}"));
    }
    context.extend(
        error
            .record_identity
            .technical_ids
            .iter()
            .map(|(k, v)| format!("{k}: {v}")),
    );
    Ok(crate::ErrorRow {
        trace: error.trace_id.0.into(),
        line: error.line_number.to_string().into(),
        record: error
            .record_identity
            .sioc_record_id
            .as_deref()
            .unwrap_or("—")
            .into(),
        field: error
            .field
            .as_ref()
            .map(|v| v.id.0.as_str())
            .unwrap_or("—")
            .into(),
        field_name: error
            .field
            .as_ref()
            .map(|v| v.name.as_ref())
            .unwrap_or("Record")
            .into(),
        code: error
            .code
            .as_symbol()
            .map_err(|_| ResultsError::Malformed)?
            .into(),
        position: error
            .field
            .as_ref()
            .map(|v| format!("{}–{}", v.start_one_based, v.end_one_based_inclusive))
            .unwrap_or_else(|| "—".into())
            .into(),
        actual: error
            .actual
            .as_ref()
            .map(diagnostic_value)
            .unwrap_or_else(|| "Non disponibile".into())
            .into(),
        expected: error
            .expected
            .unwrap_or_else(|| "Non specificato".into())
            .into(),
        context: if context.is_empty() {
            "Nessun contesto aggiuntivo".into()
        } else {
            context.join("\n").into()
        },
        message: error.message.into(),
    })
}
