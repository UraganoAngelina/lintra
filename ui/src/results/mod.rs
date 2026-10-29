//! Disk-backed diagnostic browsing. An index stores byte offsets only; the
//! virtualized model keeps a fixed-size cache, never a global Vec of errors.
pub mod controller;
mod diagnostic;

use lintra::{ScopeSummary, ValidationError, ValidationSummary};
use slint::{Model, ModelNotify, ModelTracker};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, VecDeque},
    error::Error,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader, Seek, SeekFrom},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Debug)]
pub enum ResultsError {
    Io(io::Error),
    Malformed,
    ScopeMismatch,
    CountMismatch,
    TooManyRows,
    InvalidLineFilter,
    Cancelled,
    WorkerUnavailable,
}
impl fmt::Display for ResultsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Io(_) => "Impossibile leggere il report. Ripeti l’analisi.",
            Self::Malformed => {
                "Il report contiene una diagnostica non leggibile. Ripeti l’analisi."
            }
            Self::ScopeMismatch => "Il report non corrisponde all’ambito dell’analisi.",
            Self::CountMismatch => "Il report non corrisponde ai conteggi dell’analisi.",
            Self::TooManyRows => "Il report supera la capacità della tabella desktop.",
            Self::InvalidLineFilter => "Riga: inserisci un numero intero maggiore di zero.",
            Self::Cancelled => "Lettura del report annullata.",
            Self::WorkerUnavailable => "Impossibile preparare la tabella. Riprova.",
        })
    }
}
impl Error for ResultsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}
impl From<io::Error> for ResultsError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

/// Exact, case-sensitive AND filters. Empty text means no restriction. Input
/// values are never trimmed/coerced into patient identifiers or schema values.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct ResultsFilter {
    pub code: String,
    pub field: String,
    pub record: String,
    pub fiscal_code: String,
    pub line: Option<u64>,
}
impl ResultsFilter {
    pub fn parse_line(text: &str) -> Result<Option<u64>, ResultsError> {
        if text.is_empty() {
            return Ok(None);
        }
        if !text.bytes().all(|v| v.is_ascii_digit()) {
            return Err(ResultsError::InvalidLineFilter);
        }
        text.parse::<u64>()
            .ok()
            .filter(|v| *v > 0)
            .map(Some)
            .ok_or(ResultsError::InvalidLineFilter)
    }
    fn matches(&self, error: &ValidationError) -> bool {
        (self.code.is_empty() || error.code.as_symbol().is_ok_and(|v| v == self.code))
            && (self.field.is_empty() || error.field.as_ref().is_some_and(|v| v.id.0 == self.field))
            && (self.record.is_empty()
                || error.record_identity.sioc_record_id.as_deref() == Some(&self.record))
            && (self.fiscal_code.is_empty()
                || error.record_identity.fiscal_code.as_deref() == Some(&self.fiscal_code))
            && self.line.is_none_or(|v| v == error.line_number)
    }
}

fn decode(bytes: &[u8]) -> Result<ValidationError, ResultsError> {
    serde_json::from_slice::<diagnostic::Diagnostic>(bytes)
        .map_err(|_| ResultsError::Malformed)?
        .into_error()
}
fn in_scope(error: &ValidationError, summary: &ValidationSummary) -> bool {
    error.flow_id == summary.schema_key.flow_id
        && error.schema_version == summary.schema_key.version
        && match &summary.scope_summary {
            ScopeSummary::Trace { trace_id } => *trace_id == error.trace_id,
            ScopeSummary::Flow { trace_ids } => trace_ids.contains(&error.trace_id),
        }
}

/// Immutable index built in one streaming pass. The File is already open, so
/// arbitrary paths cannot be reopened lazily after report ownership changes.
pub struct ReportIndex {
    reader: BufReader<File>,
    offsets: Vec<u64>,
    total: u64,
    // File/reader drops before the last directory owner, even if a completed
    // index is abandoned in the worker channel before any model is installed.
    owner: Option<Arc<crate::worker::AnalysisReport>>,
}
impl ReportIndex {
    /// Called on the results worker. Malformed, truncated or out-of-scope rows
    /// fail the entire candidate index: they are never silently skipped.
    pub fn scan(
        file: File,
        summary: &ValidationSummary,
        filter: &ResultsFilter,
        cancelled: &AtomicBool,
    ) -> Result<Self, ResultsError> {
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(0))?;
        let mut buffer = Vec::new();
        let mut offsets = Vec::new();
        let mut offset = 0u64;
        let mut total = 0u64;
        let mut by_code = BTreeMap::new();
        loop {
            if cancelled.load(Ordering::Acquire) {
                return Err(ResultsError::Cancelled);
            }
            buffer.clear();
            let bytes = reader.read_until(b'\n', &mut buffer)?;
            if bytes == 0 {
                break;
            }
            // Only committed canonical LF output is accepted; a truncated last
            // JSON object must not look like a complete report after corruption.
            if buffer.last() != Some(&b'\n') || buffer.contains(&b'\r') {
                return Err(ResultsError::Malformed);
            }
            let error = decode(&buffer)?;
            if !in_scope(&error, summary) {
                return Err(ResultsError::ScopeMismatch);
            }
            total = total.checked_add(1).ok_or(ResultsError::TooManyRows)?;
            *by_code.entry(error.code.clone()).or_insert(0u64) += 1;
            if filter.matches(&error) {
                if offsets.len() == i32::MAX as usize {
                    return Err(ResultsError::TooManyRows);
                }
                offsets.push(offset);
            }
            offset = offset
                .checked_add(u64::try_from(bytes).map_err(|_| ResultsError::TooManyRows)?)
                .ok_or(ResultsError::TooManyRows)?;
        }
        if total != summary.total_errors || by_code != summary.errors_by_code {
            return Err(ResultsError::CountMismatch);
        }
        Ok(Self {
            reader,
            offsets,
            total,
            owner: None,
        })
    }
    fn with_owner(mut self, report: Arc<crate::worker::AnalysisReport>) -> Self {
        self.owner = Some(report);
        self
    }
    pub fn row_count(&self) -> usize {
        self.offsets.len()
    }
    pub fn total(&self) -> u64 {
        self.total
    }
    /// Bytes of offset payload, excluding allocator/reader and the single JSON
    /// decoding buffer. This is an index-size measure, not process peak RSS.
    pub fn offset_bytes(&self) -> usize {
        self.offsets.len() * size_of::<u64>()
    }
}

const CACHE_ROWS: usize = 128;
struct Rows {
    reader: BufReader<File>,
    cache: BTreeMap<usize, crate::ErrorRow>,
    fifo: VecDeque<usize>,
    buffer: Vec<u8>,
    decoded: u64,
    highest: usize,
}
/// Slint requests visible rows; fixed FIFO cache also serves repeated layout
/// requests. Selected detail is copied independently by the controller, so
/// scrolling cannot replace it or force retention of all earlier diagnostics.
pub struct JsonlModel {
    offsets: Vec<u64>,
    rows: RefCell<Rows>,
    notify: ModelNotify,
    failed: Cell<bool>,
    // File/cache drop BEFORE the TempDir owner, including on Windows.
    _owner: Option<Arc<crate::worker::AnalysisReport>>,
}
impl JsonlModel {
    pub fn new(index: ReportIndex) -> Self {
        Self {
            offsets: index.offsets,
            rows: RefCell::new(Rows {
                reader: index.reader,
                cache: BTreeMap::new(),
                fifo: VecDeque::new(),
                buffer: Vec::new(),
                decoded: 0,
                highest: 0,
            }),
            notify: ModelNotify::default(),
            failed: Cell::new(false),
            _owner: index.owner,
        }
    }
    pub fn retain_report(&mut self, report: Arc<crate::worker::AnalysisReport>) {
        self._owner = Some(report);
    }
    pub fn failed(&self) -> bool {
        self.failed.get()
    }
    pub fn cache_len(&self) -> usize {
        self.rows.borrow().cache.len()
    }
    pub fn decoded_rows(&self) -> u64 {
        self.rows.borrow().decoded
    }
    pub fn highest_requested_row(&self) -> usize {
        self.rows.borrow().highest
    }
    pub fn offset_at(&self, row: usize) -> Option<u64> {
        self.offsets.get(row).copied()
    }
    pub fn position_of(&self, offset: u64) -> Option<usize> {
        self.offsets.binary_search(&offset).ok()
    }
    pub fn read_row(&self, row: usize) -> Result<Option<crate::ErrorRow>, ResultsError> {
        let Some(offset) = self.offset_at(row) else {
            return Ok(None);
        };
        let mut rows = self.rows.borrow_mut();
        rows.highest = rows.highest.max(row);
        if let Some(value) = rows.cache.get(&row) {
            return Ok(Some(value.clone()));
        }
        let Rows { reader, buffer, .. } = &mut *rows;
        reader.seek(SeekFrom::Start(offset))?;
        buffer.clear();
        if reader.read_until(b'\n', buffer)? == 0 || buffer.last() != Some(&b'\n') {
            return Err(ResultsError::Malformed);
        }
        let value = project(decode(buffer)?)?;
        rows.decoded += 1;
        if rows.cache.len() == CACHE_ROWS
            && let Some(old) = rows.fifo.pop_front()
        {
            rows.cache.remove(&old);
        }
        rows.cache.insert(row, value.clone());
        rows.fifo.push_back(row);
        Ok(Some(value))
    }
}
impl Model for JsonlModel {
    type Data = crate::ErrorRow;
    fn row_count(&self) -> usize {
        self.offsets.len()
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
