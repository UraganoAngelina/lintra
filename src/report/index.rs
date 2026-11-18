//! Shared disk-backed canonical JSONL index for native adapters. The streaming
//! scan retains only byte offsets; requested diagnostics use a 128-row FIFO.
//! Offset memory is O(matches), not a constant-memory claim for the full index.
use crate::{ScopeSummary, TraceId, ValidationError, ValidationSummary};
use std::{
    collections::{BTreeMap, VecDeque},
    error::Error,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader, Seek, SeekFrom},
};

const CACHE_ROWS: usize = 128;

/// Technical browsing failures. Display never includes canonical row contents;
/// WorkerUnavailable is retained for native adapters that prepare an index off-thread.
#[derive(Debug)]
pub enum ReportIndexError {
    Io(io::Error),
    Malformed,
    ScopeMismatch,
    CountMismatch,
    TooManyRows,
    InvalidLineFilter,
    Cancelled,
    WorkerUnavailable,
}
impl fmt::Display for ReportIndexError {
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
impl Error for ReportIndexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}
impl From<io::Error> for ReportIndexError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

/// Exact, case-sensitive AND filters. Empty text means no restriction. Input
/// values are never trimmed/coerced into patient identifiers or schema values.
/// `trace` is the optional trace restriction of the IPC v1 Filter DTO
/// (`trace_ref`, resolved by the native adapter against the captured scope);
/// `None` keeps the five Slint filters unchanged.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct ReportFilter {
    pub code: String,
    pub field: String,
    pub record: String,
    pub fiscal_code: String,
    pub line: Option<u64>,
    pub trace: Option<TraceId>,
}
impl ReportFilter {
    pub fn parse_line(text: &str) -> Result<Option<u64>, ReportIndexError> {
        if text.is_empty() {
            return Ok(None);
        }
        if !text.bytes().all(|v| v.is_ascii_digit()) {
            return Err(ReportIndexError::InvalidLineFilter);
        }
        text.parse::<u64>()
            .ok()
            .filter(|v| *v > 0)
            .map(Some)
            .ok_or(ReportIndexError::InvalidLineFilter)
    }
    /// True when the decoded diagnostic satisfies every active restriction.
    /// Public so adapters can re-verify a lazily read row against the
    /// immutable query that produced its offset.
    pub fn matches(&self, error: &ValidationError) -> bool {
        (self.code.is_empty() || error.code.as_symbol().is_ok_and(|v| v == self.code))
            && (self.field.is_empty() || error.field.as_ref().is_some_and(|v| v.id.0 == self.field))
            && (self.record.is_empty()
                || error.record_identity.sioc_record_id.as_deref() == Some(&self.record))
            && (self.fiscal_code.is_empty()
                || error.record_identity.fiscal_code.as_deref() == Some(&self.fiscal_code))
            && self.line.is_none_or(|v| v == error.line_number)
            && self.trace.as_ref().is_none_or(|v| *v == error.trace_id)
    }
}

fn decode(bytes: &[u8]) -> Result<ValidationError, ReportIndexError> {
    super::xlsx::read_canonical_diagnostic(bytes, 0).map_err(|_| ReportIndexError::Malformed)
}
fn in_scope(error: &ValidationError, summary: &ValidationSummary) -> bool {
    error.flow_id == summary.schema_key.flow_id
        && error.schema_version == summary.schema_key.version
        && match &summary.scope_summary {
            ScopeSummary::Trace { trace_id } => *trace_id == error.trace_id,
            ScopeSummary::Flow { trace_ids } => trace_ids.contains(&error.trace_id),
        }
}

/// Immutable offsets built in one streaming pass, with a mutable row cache.
/// The File is already open, so
/// arbitrary paths cannot be reopened lazily after report ownership changes.
pub struct JsonlReportIndex {
    reader: BufReader<File>,
    offsets: Vec<u64>,
    total: u64,
    buffer: Vec<u8>,
    cache: BTreeMap<usize, ValidationError>,
    fifo: VecDeque<usize>,
    decoded: u64,
    highest: usize,
}
impl JsonlReportIndex {
    /// Called on the results worker. Malformed, truncated or out-of-scope rows
    /// fail the entire candidate index: they are never silently skipped.
    /// The callback adapts the caller's cancellation token, checked between
    /// rows and at EOF. No candidate is returned after an observed cancellation.
    pub fn scan(
        file: File,
        summary: &ValidationSummary,
        filter: &ReportFilter,
        mut is_cancelled: impl FnMut() -> bool,
    ) -> Result<Self, ReportIndexError> {
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(0))?;
        let mut buffer = Vec::new();
        let mut offsets = Vec::new();
        let mut offset = 0u64;
        let mut total = 0u64;
        let mut by_code = BTreeMap::new();
        loop {
            if is_cancelled() {
                return Err(ReportIndexError::Cancelled);
            }
            buffer.clear();
            let bytes = reader.read_until(b'\n', &mut buffer)?;
            if bytes == 0 {
                break;
            }
            // Only committed canonical LF output is accepted; a truncated last
            // JSON object must not look like a complete report after corruption.
            if buffer.last() != Some(&b'\n') || buffer.contains(&b'\r') {
                return Err(ReportIndexError::Malformed);
            }
            let error = decode(&buffer)?;
            if !in_scope(&error, summary) {
                return Err(ReportIndexError::ScopeMismatch);
            }
            total = total.checked_add(1).ok_or(ReportIndexError::TooManyRows)?;
            *by_code.entry(error.code.clone()).or_insert(0u64) += 1;
            if filter.matches(&error) {
                if offsets.len() == i32::MAX as usize {
                    return Err(ReportIndexError::TooManyRows);
                }
                offsets.push(offset);
            }
            offset = offset
                .checked_add(u64::try_from(bytes).map_err(|_| ReportIndexError::TooManyRows)?)
                .ok_or(ReportIndexError::TooManyRows)?;
        }
        if total != summary.total_errors || by_code != summary.errors_by_code {
            return Err(ReportIndexError::CountMismatch);
        }
        Ok(Self {
            reader,
            offsets,
            total,
            buffer,
            cache: BTreeMap::new(),
            fifo: VecDeque::new(),
            decoded: 0,
            highest: 0,
        })
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

    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }
    /// Counts cache misses after indexing; the streaming validation pass does
    /// not populate the cache or create presentation rows.
    pub fn decoded_rows(&self) -> u64 {
        self.decoded
    }
    pub fn highest_requested_row(&self) -> usize {
        self.highest
    }
    pub fn offset_at(&self, row: usize) -> Option<u64> {
        self.offsets.get(row).copied()
    }
    pub fn position_of(&self, offset: u64) -> Option<usize> {
        self.offsets.binary_search(&offset).ok()
    }
    /// Decode only the requested row. A FIFO of at most CACHE_ROWS diagnostics
    /// serves repeated reads; neither offsets nor diagnostics leave Rust here.
    /// The caller retains the private spool owner until this reader is dropped.
    pub fn read_row(&mut self, row: usize) -> Result<Option<ValidationError>, ReportIndexError> {
        let Some(offset) = self.offset_at(row) else {
            return Ok(None);
        };
        self.highest = self.highest.max(row);
        if let Some(value) = self.cache.get(&row) {
            return Ok(Some(value.clone()));
        }
        self.reader.seek(SeekFrom::Start(offset))?;
        self.buffer.clear();
        if self.reader.read_until(b'\n', &mut self.buffer)? == 0
            || self.buffer.last() != Some(&b'\n')
            || self.buffer.contains(&b'\r')
        {
            return Err(ReportIndexError::Malformed);
        }
        let value = decode(&self.buffer)?;
        self.decoded += 1;
        if self.cache.len() == CACHE_ROWS
            && let Some(old) = self.fifo.pop_front()
        {
            self.cache.remove(&old);
        }
        self.cache.insert(row, value.clone());
        self.fifo.push_back(row);
        Ok(Some(value))
    }
}
