//! Rust-owned filtered queries and bounded result pages (T-MIG-P4-08).
//!
//! Contract: `docs/migration/ipc-v1/PAGING.md` (IPC v1, ratified 14 Nov 2026).
//! The renderer never receives the index, byte offsets, the JSONL file or a
//! complete diagnostic: it opens an immutable query, polls its state and then
//! reads pages of at most 128 rows that carry only opaque references and
//! bounded metadata previews.
//!
//! Threading: Tauri runs this synchronous command on the main thread, so a
//! filtered O(n) scan never runs here. It runs on one owned worker; the result
//! is published only when a later call joins a finished worker and the
//! candidate is still the current intention. The unfiltered query reuses the
//! index already verified at report publication (P4-07) and is ready at once.
//!
//! The selected diagnostic, its bounded texts, context and captured domains
//! (T-MIG-P4-09) live in the `detail` submodule and are pinned to the
//! current query: a new query or report drops the selection.
mod detail;
pub(crate) use detail::DetailRequest;

use crate::{
    configuration::fields,
    ipc::{self, ErrorCode},
    jobs::{Report, RunRef},
};
use lintra::{CancellationToken, JsonlReportIndex, ReportFilter, ReportIndexError, TraceId};
use serde_json::{Map, Value, json};
use std::{
    collections::VecDeque,
    sync::Arc,
    thread::{self, JoinHandle},
};

/// Maximum rows per page; equals the existing Slint/library row cache bound.
pub(crate) const MAX_PAGE_ROWS: u64 = 128;
/// Pages whose diagnostic references stay registered for the current query.
const MAX_RETAINED_PAGES: usize = 2;
/// Maximum UTF-8 bytes of a metadata preview, cut on a character boundary.
const PREVIEW_BYTES: usize = 256;
const FILTER_KEYS: [&str; 7] = [
    "schemaVersion",
    "code",
    "field",
    "record",
    "fiscal_code",
    "line",
    "trace_ref",
];

/// Wire identifier grammar of IPC v1 (`[A-Za-z0-9_-]{1,64}`).
pub(crate) fn identifier(value: &Value) -> Result<String, ErrorCode> {
    value
        .as_str()
        .filter(|text| {
            (1..=64).contains(&text.len())
                && text
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
        })
        .map(str::to_owned)
        .ok_or(ErrorCode::InvalidMessage)
}

/// Canonical decimal u64 counter: no sign, no leading zero, no overflow.
fn canonical_u64(value: &Value) -> Result<u64, ErrorCode> {
    let text = value.as_str().ok_or(ErrorCode::InvalidMessage)?;
    if text.is_empty()
        || text.len() > 20
        || !text.bytes().all(|byte| byte.is_ascii_digit())
        || (text.len() > 1 && text.starts_with('0'))
    {
        return Err(ErrorCode::InvalidMessage);
    }
    text.parse().map_err(|_| ErrorCode::InvalidMessage)
}

/// Filter DTO schemaVersion 1. Text filters are exact and never trimmed; an
/// absent or empty value means no restriction. `line` reuses the Slint
/// grammar (positive u64, leading zeros accepted). `trace_ref` is resolved
/// later against the captured scope of the report.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FilterRequest {
    code: String,
    field: String,
    record: String,
    fiscal_code: String,
    line: Option<u64>,
    trace_ref: Option<String>,
}
impl FilterRequest {
    fn decode(value: &Value) -> Result<Self, ErrorCode> {
        let dto = value.as_object().ok_or(ErrorCode::InvalidMessage)?;
        ipc::version(dto)?;
        if !dto.keys().all(|key| FILTER_KEYS.contains(&key.as_str())) {
            return Err(ErrorCode::InvalidMessage);
        }
        let text = |key: &str| match dto.get(key) {
            None => Ok(String::new()),
            Some(Value::String(text)) => Ok(text.clone()),
            Some(_) => Err(ErrorCode::InvalidMessage),
        };
        let line = match dto.get("line") {
            None | Some(Value::Null) => None,
            Some(Value::String(text)) => {
                ReportFilter::parse_line(text).map_err(|_| ErrorCode::InvalidMessage)?
            }
            Some(_) => return Err(ErrorCode::InvalidMessage),
        };
        let trace_ref = match dto.get("trace_ref") {
            None | Some(Value::Null) => None,
            Some(value) => Some(identifier(value)?),
        };
        Ok(Self {
            code: text("code")?,
            field: text("field")?,
            record: text("record")?,
            fiscal_code: text("fiscal_code")?,
            line,
            trace_ref,
        })
    }
    /// Resolve the opaque trace handle; an unknown handle is stale context,
    /// never a silently ignored restriction.
    fn resolve(self, traces: &[TraceId]) -> Result<ReportFilter, ErrorCode> {
        let trace = match self.trace_ref {
            None => None,
            Some(handle) => Some(
                handle
                    .strip_prefix("trace_")
                    .filter(|digits| digits == &"0" || !digits.starts_with('0'))
                    .and_then(|digits| digits.parse::<usize>().ok())
                    .and_then(|position| traces.get(position))
                    .cloned()
                    .ok_or(ErrorCode::StaleContext)?,
            ),
        };
        Ok(ReportFilter {
            code: self.code,
            field: self.field,
            record: self.record,
            fiscal_code: self.fiscal_code,
            line: self.line,
            trace,
        })
    }
}

/// Exact purpose-specific DTOs, decoded before any state or I/O is touched.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ResultsRequest {
    Query {
        run: RunRef,
        filter: FilterRequest,
    },
    State {
        query_ref: String,
    },
    Page {
        query_ref: String,
        offset: u64,
        limit: u64,
    },
    /// Report overview for the results charts (T-MIG-P6-01).
    Overview {
        run: RunRef,
    },
}
impl ResultsRequest {
    pub(crate) fn decode(operation: &str, dto: &Map<String, Value>) -> Result<Self, ErrorCode> {
        ipc::version(dto)?;
        match operation {
            "results_query" => {
                fields(dto, &["run", "filters"])?;
                Ok(Self::Query {
                    run: RunRef::decode(&dto["run"])?,
                    filter: FilterRequest::decode(&dto["filters"])?,
                })
            }
            "results_query_state" => {
                fields(dto, &["query_ref"])?;
                Ok(Self::State {
                    query_ref: identifier(&dto["query_ref"])?,
                })
            }
            "results_page" => {
                fields(dto, &["query_ref", "offset", "limit"])?;
                Ok(Self::Page {
                    query_ref: identifier(&dto["query_ref"])?,
                    offset: canonical_u64(&dto["offset"])?,
                    // JSON integer only: strings, booleans and 0/129+ are rejected.
                    limit: dto["limit"]
                        .as_u64()
                        .filter(|limit| (1..=MAX_PAGE_ROWS).contains(limit))
                        .ok_or(ErrorCode::InvalidMessage)?,
                })
            }
            "results_overview" => {
                fields(dto, &["run"])?;
                Ok(Self::Overview {
                    run: RunRef::decode(&dto["run"])?,
                })
            }
            _ => Err(ErrorCode::UnknownOperation),
        }
    }
}

/// OS-random opaque reference: not an offset, HMAC key or capability derived
/// from a predictable name. Collisions are checked by the caller.
pub(crate) fn nonce(prefix: &str) -> Result<String, ErrorCode> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| ErrorCode::TechnicalFailure)?;
    Ok(bytes.iter().fold(format!("{prefix}_"), |text, byte| {
        text + &format!("{byte:02x}")
    }))
}

/// Metadata preview DTO: at most 256 UTF-8 bytes on a character boundary with
/// an explicit completeness flag, so a prefix is never mistaken for the value.
pub(crate) fn preview(text: &str) -> Value {
    bounded_preview(text, PREVIEW_BYTES)
}

/// Row texts (message, expected, value found; T-MIG-P6-01, author decision
/// of 24 Nov 2026): shorter previews, so a page of 128 rows with every text
/// at its worst-case JSON escaping (6 bytes per control byte) still fits the
/// 1 MiB frame instead of failing the whole page.
const ROW_TEXT_BYTES: usize = 128;

fn bounded_preview(text: &str, limit: usize) -> Value {
    let mut end = text.len().min(limit);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    json!({"schemaVersion":1,"text":&text[..end],"complete":end == text.len()})
}

fn status(run: &RunRef, query_ref: &str, state: &str, totals: Option<(u64, u64)>) -> Value {
    json!({"schemaVersion":1,"run":run.dto(),"query_ref":query_ref,"state":state,
        "total_matches":totals.map(|(matches, _)| matches.to_string()),
        "total_diagnostics":totals.map(|(_, total)| total.to_string())})
}

/// A filtered index being built off the main thread.
struct Candidate {
    thread: Option<JoinHandle<Result<JsonlReportIndex, ReportIndexError>>>,
    token: CancellationToken,
    query_ref: String,
    run: RunRef,
    filter: ReportFilter,
    report: Arc<Report>,
}
impl Drop for Candidate {
    /// A superseded, reset or shut-down candidate is cancelled and joined: the
    /// scan observes the token between rows, so the join is short and the
    /// worker's file handle closes before the report can release its spool.
    fn drop(&mut self) {
        self.token.cancel();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Unfiltered queries share the publication index; filtered ones own theirs.
enum Source {
    Shared,
    // Boxed: the index grew with per-trace counts (T-MIG-P6-01) and the
    // shared variant carries nothing (clippy::large_enum_variant).
    Owned(Box<JsonlReportIndex>),
}

/// A published immutable query. `source` is declared (and dropped) before
/// `report`, so an owned reader never outlives the spool directory; the
/// selection, with any pending domain read, is dropped first of all.
struct Query {
    selection: Option<detail::Selection>,
    source: Source,
    report: Arc<Report>,
    run: RunRef,
    query_ref: String,
    filter: ReportFilter,
    generation: u64,
    // Diagnostic references of the retained pages, oldest first, each with
    // the query row it names. Selection (P4-09) resolves a reference only
    // through this registry: an evicted page's references are stale.
    pages: VecDeque<Vec<(String, usize)>>,
}
impl Query {
    fn read<T>(
        &mut self,
        read: impl FnOnce(&mut JsonlReportIndex) -> Result<T, ErrorCode>,
    ) -> Result<T, ErrorCode> {
        match &mut self.source {
            Source::Owned(index) => read(index),
            Source::Shared => self.report.with_index(read),
        }
    }
    fn totals(&mut self) -> Result<(u64, u64), ErrorCode> {
        self.read(|index| {
            Ok((
                u64::try_from(index.row_count()).map_err(|_| ErrorCode::TechnicalFailure)?,
                index.total(),
            ))
        })
    }
    fn holds(&self, token: &str) -> bool {
        self.pages
            .iter()
            .any(|page| page.iter().any(|(t, _)| t == token))
    }
}

/// Query state of the last published report, owned by `Jobs`.
#[derive(Default)]
pub(crate) struct Results {
    candidate: Option<Candidate>,
    current: Option<Query>,
    failed: Option<(String, RunRef)>,
}
impl Results {
    /// Forget every query; a building candidate is cancelled and joined.
    pub(crate) fn reset(&mut self) {
        self.candidate = None;
        self.current = None;
        self.failed = None;
    }
    pub(crate) fn cancel_on_close(&self) {
        if let Some(candidate) = &self.candidate {
            candidate.token.cancel();
        }
        if let Some(selection) = self.current.as_ref().and_then(|q| q.selection.as_ref()) {
            selection.cancel();
        }
    }
    /// Publish a finished candidate without blocking on a running worker. A
    /// failed scan (I/O, malformed row, scope or count mismatch) leaves the
    /// previous query untouched and is reported as `failed` for its own ref.
    fn poll(&mut self) {
        if !self
            .candidate
            .as_ref()
            .and_then(|candidate| candidate.thread.as_ref())
            .is_some_and(JoinHandle::is_finished)
        {
            return;
        }
        let Some(mut candidate) = self.candidate.take() else {
            return;
        };
        match candidate.thread.take().map(JoinHandle::join) {
            Some(Ok(Ok(index))) => {
                self.current = Some(Query {
                    source: Source::Owned(Box::new(index)),
                    report: candidate.report.clone(),
                    run: candidate.run.clone(),
                    query_ref: std::mem::take(&mut candidate.query_ref),
                    filter: std::mem::take(&mut candidate.filter),
                    generation: 0,
                    pages: VecDeque::new(),
                    selection: None,
                });
                self.failed = None;
            }
            _ => self.failed = Some((candidate.query_ref.clone(), candidate.run.clone())),
        }
    }
    fn known(&self, query_ref: &str) -> bool {
        self.current
            .as_ref()
            .is_some_and(|q| q.query_ref == query_ref)
            || self
                .candidate
                .as_ref()
                .is_some_and(|c| c.query_ref == query_ref)
            || self.failed.as_ref().is_some_and(|(r, _)| r == query_ref)
    }

    pub(crate) fn dispatch(
        &mut self,
        request: ResultsRequest,
        report: Option<&(RunRef, Arc<Report>)>,
        id: &str,
    ) -> Result<Vec<u8>, ErrorCode> {
        self.poll();
        match request {
            ResultsRequest::Query { run, filter } => self.query(run, filter, report, id),
            ResultsRequest::State { query_ref } => self.state(&query_ref, id),
            ResultsRequest::Page {
                query_ref,
                offset,
                limit,
            } => self.page(&query_ref, offset, limit, id),
            ResultsRequest::Overview { run } => overview(&run, report, id),
        }
    }

    /// Detail operations over the current query (P4-09). `results_select`
    /// needs the query to be the current one; the other operations need the
    /// pinned selection, which a newer query or report has already dropped.
    pub(crate) fn detail(
        &mut self,
        request: DetailRequest,
        id: &str,
    ) -> Result<Vec<u8>, ErrorCode> {
        self.poll();
        let query = self.current.as_mut();
        if let DetailRequest::Select {
            query_ref,
            diagnostic_ref,
        } = &request
        {
            let pending = self
                .candidate
                .as_ref()
                .is_some_and(|c| &c.query_ref == query_ref);
            return match query.filter(|q| &q.query_ref == query_ref) {
                Some(query) => detail::select(query, diagnostic_ref, id),
                None if pending => Err(ErrorCode::InvalidState),
                None => Err(ErrorCode::StaleContext),
            };
        }
        let wanted = request.selection_ref().ok_or(ErrorCode::TechnicalFailure)?;
        let query = query.ok_or(ErrorCode::StaleContext)?;
        let report = query.report.clone();
        let selection = query
            .selection
            .as_mut()
            .filter(|s| s.selection_ref == wanted)
            .ok_or(ErrorCode::StaleContext)?;
        match request {
            DetailRequest::Select { .. } => Err(ErrorCode::TechnicalFailure),
            DetailRequest::Text {
                component_ref,
                offset,
                limit,
                ..
            } => detail::text(selection, &component_ref, offset, limit, id),
            DetailRequest::Context { offset, limit, .. } => {
                detail::context(selection, &report, offset, limit, id)
            }
            DetailRequest::Sources { offset, limit, .. } => {
                detail::sources_page(selection, &report, offset, limit, id)
            }
            DetailRequest::Domain {
                source_ref,
                cursor_ref,
                ..
            } => detail::domain(selection, &report, &source_ref, cursor_ref.as_deref(), id),
            DetailRequest::DomainState { read_ref, .. } => {
                detail::domain_state(selection, &report, &read_ref, id)
            }
        }
    }

    fn query(
        &mut self,
        run: RunRef,
        filter: FilterRequest,
        report: Option<&(RunRef, Arc<Report>)>,
        id: &str,
    ) -> Result<Vec<u8>, ErrorCode> {
        let (published, report) = report.ok_or(ErrorCode::InvalidState)?;
        if run != *published {
            return Err(ErrorCode::StaleContext);
        }
        let filter = filter.resolve(report.traces())?;
        let query_ref = nonce("q")?;
        if self.known(&query_ref) {
            return Err(ErrorCode::TechnicalFailure);
        }
        if filter == ReportFilter::default() {
            // Reuse the verified publication index: no rescan, ready at once.
            let totals = report.with_index(|index| {
                Ok((
                    u64::try_from(index.row_count()).map_err(|_| ErrorCode::TechnicalFailure)?,
                    index.total(),
                ))
            })?;
            let bytes = ipc::response(id, status(&run, &query_ref, "ready", Some(totals)))?;
            self.candidate = None;
            self.failed = None;
            self.current = Some(Query {
                source: Source::Shared,
                report: report.clone(),
                run,
                query_ref,
                filter,
                generation: 0,
                pages: VecDeque::new(),
                selection: None,
            });
            return Ok(bytes);
        }
        let bytes = ipc::response(id, status(&run, &query_ref, "building", None))?;
        let token = CancellationToken::default();
        let (worker_report, worker_filter, cancelled) =
            (report.clone(), filter.clone(), token.clone());
        let thread = thread::Builder::new()
            .name("lintra-desktop-results".into())
            .spawn(move || {
                let file = worker_report.open()?;
                JsonlReportIndex::scan(file, worker_report.summary(), &worker_filter, || {
                    cancelled.is_cancelled()
                })
            })
            .map_err(|_| ErrorCode::TechnicalFailure)?;
        // Replacing the slot cancels and joins a superseded candidate.
        self.candidate = Some(Candidate {
            thread: Some(thread),
            token,
            query_ref,
            run,
            filter,
            report: report.clone(),
        });
        self.failed = None;
        Ok(bytes)
    }

    fn state(&mut self, query_ref: &str, id: &str) -> Result<Vec<u8>, ErrorCode> {
        if let Some(query) = self.current.as_mut().filter(|q| q.query_ref == query_ref) {
            let totals = query.totals()?;
            return ipc::response(id, status(&query.run, query_ref, "ready", Some(totals)));
        }
        if let Some(candidate) = self.candidate.as_ref().filter(|c| c.query_ref == query_ref) {
            return ipc::response(id, status(&candidate.run, query_ref, "building", None));
        }
        if let Some((_, run)) = self.failed.as_ref().filter(|(r, _)| r == query_ref) {
            return ipc::response(id, status(run, query_ref, "failed", None));
        }
        Err(ErrorCode::StaleContext)
    }

    fn page(
        &mut self,
        query_ref: &str,
        offset: u64,
        limit: u64,
        id: &str,
    ) -> Result<Vec<u8>, ErrorCode> {
        let pending = self
            .candidate
            .as_ref()
            .is_some_and(|c| c.query_ref == query_ref);
        let Some(query) = self.current.as_mut().filter(|q| q.query_ref == query_ref) else {
            return Err(if pending {
                ErrorCode::InvalidState
            } else {
                ErrorCode::StaleContext
            });
        };
        let (total, diagnostics) = query.totals()?;
        // No clamp: an offset past the end is a client error, not an empty page.
        if offset > total {
            return Err(ErrorCode::InvalidMessage);
        }
        let end = offset.saturating_add(limit).min(total);
        let generation = query
            .generation
            .checked_add(1)
            .ok_or(ErrorCode::TechnicalFailure)?;
        let (filter, traces) = (query.filter.clone(), query.report.traces().to_vec());
        let mut tokens = Vec::new();
        let mut rows = Vec::new();
        for position in offset..end {
            let row = usize::try_from(position).map_err(|_| ErrorCode::TechnicalFailure)?;
            let error = query
                .read(|index| index.read_row(row).map_err(|_| ErrorCode::TechnicalFailure))?
                .ok_or(ErrorCode::TechnicalFailure)?;
            // The private spool is immutable; a mismatch means it changed.
            if !filter.matches(&error) {
                return Err(ErrorCode::TechnicalFailure);
            }
            let trace = traces
                .iter()
                .position(|trace| *trace == error.trace_id)
                .ok_or(ErrorCode::TechnicalFailure)?;
            let code = error
                .code
                .as_symbol()
                .map_err(|_| ErrorCode::TechnicalFailure)?;
            let token = nonce("d")?;
            if tokens.iter().any(|(t, _)| t == &token) || query.holds(&token) {
                return Err(ErrorCode::TechnicalFailure);
            }
            // Row DTO: references and previews. Since T-MIG-P6-01 (author
            // decision, PAGING.md amendment of 24 Nov 2026) the row also
            // carries the message, the expected text and the displayed value
            // found, as bounded previews; raw bytes, context and record
            // identity stay in Rust (P2-06).
            let text = |value: &str| bounded_preview(value, ROW_TEXT_BYTES);
            rows.push(json!({"schemaVersion":1,"diagnostic_ref":token,
                "ordinal":position.to_string(),"line":error.line_number.to_string(),
                "trace_ref":format!("trace_{trace}"),"code":preview(code),
                "field":error.field.as_ref().map(|field| preview(&field.name)),
                "message":text(&error.message),
                "expected":error.expected.as_deref().map(text),
                "actual":error.actual.as_ref().map(|actual| text(&actual.display))}));
            tokens.push((token, row));
        }
        // Encode the whole frame within 1 MiB before publishing any token or
        // generation: an oversized page changes no state and is never cut.
        let bytes = ipc::response(
            id,
            json!({"schemaVersion":1,"query_ref":query_ref,"run":query.run.dto(),
                "page_generation":generation.to_string(),"offset":offset.to_string(),
                "rows":rows,"next_offset":end.to_string(),"has_more":end < total,
                "total_matches":total.to_string(),"total_diagnostics":diagnostics.to_string()}),
        )?;
        query.generation = generation;
        query.pages.push_back(tokens);
        while query.pages.len() > MAX_RETAINED_PAGES {
            query.pages.pop_front();
        }
        Ok(bytes)
    }
}

/// Overview of the published report for the results charts (T-MIG-P6-01):
/// record counts and errors per code from the verified summary, errors per
/// trace from the publication index (same verified pass). Every scope trace
/// is listed, in scope order, with its own handle and name, so the renderer
/// never decodes a `trace_ref`. Nothing is computed from rows in the renderer.
fn overview(
    run: &RunRef,
    report: Option<&(RunRef, Arc<Report>)>,
    id: &str,
) -> Result<Vec<u8>, ErrorCode> {
    let (published, report) = report.ok_or(ErrorCode::InvalidState)?;
    if run != published {
        return Err(ErrorCode::StaleContext);
    }
    let summary = report.summary();
    let by_trace = report.with_index(|index| Ok(index.errors_by_trace().clone()))?;
    let traces = report.traces();
    // The scan already rejects out-of-scope rows; a mismatch is a defect.
    if by_trace.keys().any(|trace| !traces.contains(trace))
        || by_trace.values().sum::<u64>() != summary.total_errors
    {
        return Err(ErrorCode::TechnicalFailure);
    }
    let codes = summary
        .errors_by_code
        .iter()
        .map(|(code, count)| {
            let symbol = code.as_symbol().map_err(|_| ErrorCode::TechnicalFailure)?;
            Ok(json!({"schemaVersion":1,"code":preview(symbol),"count":count.to_string()}))
        })
        .collect::<Result<Vec<_>, ErrorCode>>()?;
    let per_trace: Vec<Value> = traces
        .iter()
        .enumerate()
        .map(|(position, trace)| {
            json!({"schemaVersion":1,"trace_ref":format!("trace_{position}"),
                "trace":preview(&trace.0),
                "count":by_trace.get(trace).copied().unwrap_or(0).to_string()})
        })
        .collect();
    ipc::response(
        id,
        json!({"schemaVersion":1,"run":run.dto(),
            "records_processed":summary.records_processed.to_string(),
            "valid_records":summary.valid_records.to_string(),
            "invalid_records":summary.invalid_records.to_string(),
            "total_errors":summary.total_errors.to_string(),
            "by_code":codes,"by_trace":per_trace}),
    )
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::jobs::{
        JobRequest, Jobs,
        tests::{captured, fixture},
    };
    use lintra::*;
    use std::{
        collections::{BTreeMap, HashSet},
        fs,
        io::{BufWriter, Write},
        path::PathBuf,
        sync::mpsc,
        time::{Duration, Instant, UNIX_EPOCH},
    };

    const FIXTURE: &[u8] = include_bytes!("../../../../tests/fixtures/diagnostics.errors.jsonl");
    const INSTANCE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const PAGE_KEYS: [&str; 10] = [
        "schemaVersion",
        "query_ref",
        "run",
        "page_generation",
        "offset",
        "rows",
        "next_offset",
        "has_more",
        "total_matches",
        "total_diagnostics",
    ];
    const ROW_KEYS: [&str; 10] = [
        "schemaVersion",
        "diagnostic_ref",
        "ordinal",
        "line",
        "trace_ref",
        "code",
        "field",
        "message",
        "expected",
        "actual",
    ];

    /// The two canonical synthetic diagnostics: [field-level, record-level].
    pub(super) fn base() -> Vec<Value> {
        FIXTURE
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice(line).unwrap())
            .collect()
    }
    fn summary(total: u64, by_code: BTreeMap<String, u64>, traces: &[&str]) -> ValidationSummary {
        let mut ids: Vec<TraceId> = traces.iter().map(|t| TraceId((*t).into())).collect();
        ValidationSummary {
            started_at: UNIX_EPOCH,
            validation_mode: ValidationMode::LocalOnly,
            run_status: RunStatus::CompletedInvalid,
            schema_key: SchemaKey {
                flow_id: FlowId("SYNTHETIC".into()),
                version: SchemaVersion("v1".into()),
            },
            scope_summary: if ids.len() == 1 {
                ScopeSummary::Trace {
                    trace_id: ids.remove(0),
                }
            } else {
                ScopeSummary::Flow { trace_ids: ids }
            },
            records_processed: total,
            valid_records: 0,
            invalid_records: total,
            total_errors: total,
            errors_by_code: by_code
                .into_iter()
                .map(|(code, count)| (code.parse().unwrap(), count))
                .collect(),
            skipped_rules: 0,
            truncated: false,
            validator_version: "synthetic".into(),
            schema_hash: "synthetic".into(),
            reference_data_versions: BTreeMap::new(),
            reference_validation: None,
            full_validation: None,
            elapsed: Duration::ZERO,
            inputs: Vec::new(),
        }
    }
    /// Publish synthetic rows through the production `Report::publish` path.
    pub(super) fn report(rows: &[Value], traces: &[&str]) -> (Arc<Report>, PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("errors.jsonl");
        let mut by_code = BTreeMap::new();
        {
            let mut writer = BufWriter::new(fs::File::create(&path).unwrap());
            for row in rows {
                serde_json::to_writer(&mut writer, row).unwrap();
                writer.write_all(b"\n").unwrap();
                *by_code
                    .entry(row["code"].as_str().unwrap().to_owned())
                    .or_insert(0) += 1;
            }
            writer.flush().unwrap();
        }
        let (_input, intent, _) = fixture();
        let summary = summary(rows.len() as u64, by_code, traces);
        (
            Arc::new(
                Report::publish(directory, summary, intent.clone(), captured(&intent)).unwrap(),
            ),
            path,
        )
    }
    /// `count` rows alternating field/record diagnostics, lines 1..=count,
    /// traces cycling over `traces`.
    pub(super) fn rows(count: usize, traces: &[&str]) -> Vec<Value> {
        let base = base();
        (0..count)
            .map(|i| {
                let mut row = base[i % 2].clone();
                row["line"] = (i as u64 + 1).into();
                row["trace"] = traces[(i / 2) % traces.len()].into();
                row
            })
            .collect()
    }
    pub(super) fn run(number: u64) -> RunRef {
        RunRef::decode(
            &json!({"schemaVersion":1,"instance_id":INSTANCE,"run_id":number.to_string()}),
        )
        .unwrap()
    }
    fn decode(operation: &str, payload: Value) -> Result<ResultsRequest, ErrorCode> {
        ResultsRequest::decode(operation, payload.as_object().unwrap())
    }
    pub(super) fn call(
        results: &mut Results,
        operation: &str,
        payload: Value,
        published: Option<&(RunRef, Arc<Report>)>,
    ) -> Result<Value, ErrorCode> {
        let bytes = results.dispatch(decode(operation, payload)?, published, "test_1")?;
        assert!(bytes.len() <= ipc::MAX_BYTES);
        let frame: Value = serde_json::from_slice(&bytes).unwrap();
        Ok(frame["payload"].clone())
    }
    pub(super) fn query(
        results: &mut Results,
        published: &(RunRef, Arc<Report>),
        filters: Value,
    ) -> Result<Value, ErrorCode> {
        call(
            results,
            "results_query",
            json!({"schemaVersion":1,"run":published.0.dto(),"filters":filters}),
            Some(published),
        )
    }
    fn state(results: &mut Results, query_ref: &Value) -> Result<Value, ErrorCode> {
        call(
            results,
            "results_query_state",
            json!({"schemaVersion":1,"query_ref":query_ref}),
            None,
        )
    }
    pub(super) fn settle(results: &mut Results, query_ref: &Value) -> Value {
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            let status = state(results, query_ref).unwrap();
            if status["state"] != "building" {
                return status;
            }
            assert!(Instant::now() < deadline, "query worker did not finish");
            thread::sleep(Duration::from_millis(5));
        }
    }
    pub(super) fn page(
        results: &mut Results,
        query_ref: &Value,
        offset: &str,
        limit: u64,
    ) -> Result<Value, ErrorCode> {
        call(
            results,
            "results_page",
            json!({"schemaVersion":1,"query_ref":query_ref,"offset":offset,"limit":limit}),
            None,
        )
    }
    fn lines(page: &Value) -> Vec<u64> {
        page["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["line"].as_str().unwrap().parse().unwrap())
            .collect()
    }
    fn keys(value: &Value) -> Vec<&str> {
        let mut keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        keys
    }
    fn sorted(names: &[&'static str]) -> Vec<&'static str> {
        let mut names = names.to_vec();
        names.sort_unstable();
        names
    }
    /// A candidate whose worker waits for the test, making build races explicit.
    fn held(
        results: &mut Results,
        published: &(RunRef, Arc<Report>),
        filter: ReportFilter,
    ) -> (
        String,
        mpsc::SyncSender<Result<JsonlReportIndex, ReportIndexError>>,
    ) {
        let (sender, receiver) = mpsc::sync_channel(1);
        let query_ref = nonce("q").unwrap();
        let token = CancellationToken::default();
        let cancelled = token.clone();
        results.candidate = Some(Candidate {
            // Like the real scan, the held worker observes cancellation, so a
            // superseding query can join it while the test still holds the sender.
            thread: Some(thread::spawn(move || {
                loop {
                    if cancelled.is_cancelled() {
                        return Err(ReportIndexError::Cancelled);
                    }
                    match receiver.recv_timeout(Duration::from_millis(1)) {
                        Ok(result) => return result,
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            return Err(ReportIndexError::Cancelled);
                        }
                    }
                }
            })),
            token,
            query_ref: query_ref.clone(),
            run: published.0.clone(),
            filter,
            report: published.1.clone(),
        });
        (query_ref, sender)
    }
    fn wait_finished(results: &Results) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !results
            .candidate
            .as_ref()
            .and_then(|c| c.thread.as_ref())
            .is_some_and(JoinHandle::is_finished)
        {
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
    }

    #[test]
    fn dto_decoding_is_exact_and_rejects_noncanonical_values() {
        let published = run(1).dto();
        let filters = |filters: Value| {
            decode(
                "results_query",
                json!({"schemaVersion":1,"run":published,"filters":filters}),
            )
        };
        let Ok(ResultsRequest::Query { filter, .. }) = filters(json!({"schemaVersion":1,
            "code":" E ","field":"","record":"R","fiscal_code":"F","line":"007","trace_ref":"trace_1"}))
        else {
            panic!("full filter DTO is valid")
        };
        // Exact text: no trim/casefold; zeros accepted only by the line grammar.
        assert_eq!(
            filter,
            FilterRequest {
                code: " E ".into(),
                field: String::new(),
                record: "R".into(),
                fiscal_code: "F".into(),
                line: Some(7),
                trace_ref: Some("trace_1".into()),
            }
        );
        assert!(filters(json!({"schemaVersion":1})).is_ok());
        assert!(filters(json!({"schemaVersion":1,"line":null,"trace_ref":null,"line":""})).is_ok());
        for bad in [
            json!({"schemaVersion":1,"path":"SYNTHETIC"}),
            json!({"schemaVersion":1,"code":1}),
            json!({"schemaVersion":1,"code":null}),
            json!({"schemaVersion":1,"line":"0"}),
            json!({"schemaVersion":1,"line":"-1"}),
            json!({"schemaVersion":1,"line":" 1"}),
            json!({"schemaVersion":1,"line":"1.0"}),
            json!({"schemaVersion":1,"line":"18446744073709551616"}),
            json!({"schemaVersion":1,"line":7}),
            json!({"schemaVersion":1,"trace_ref":""}),
            json!({"schemaVersion":1,"trace_ref":"a b"}),
            json!({"schemaVersion":1,"trace_ref":"t".repeat(65)}),
            json!({"code":"E"}),
            json!([]),
        ] {
            assert_eq!(filters(bad), Err(ErrorCode::InvalidMessage));
        }
        assert_eq!(
            filters(json!({"schemaVersion":2})),
            Err(ErrorCode::UnsupportedVersion)
        );
        for bad in [
            json!({"schemaVersion":1,"run":published}),
            json!({"schemaVersion":1,"run":published,"filters":{"schemaVersion":1},"offset":"0"}),
            json!({"schemaVersion":1,"run":{"schemaVersion":1,"instance_id":INSTANCE,"run_id":"01"},"filters":{"schemaVersion":1}}),
        ] {
            assert_eq!(decode("results_query", bad), Err(ErrorCode::InvalidMessage));
        }
        assert_eq!(
            decode(
                "results_page",
                json!({"schemaVersion":1,"query_ref":"q_1","offset":"0","limit":128})
            ),
            Ok(ResultsRequest::Page {
                query_ref: "q_1".into(),
                offset: 0,
                limit: 128
            })
        );
        for (offset, limit) in [
            (json!("01"), json!(1)),
            (json!("-1"), json!(1)),
            (json!(""), json!(1)),
            (json!(0), json!(1)),
            (json!("18446744073709551616"), json!(1)),
            (json!("0"), json!(0)),
            (json!("0"), json!(129)),
            (json!("0"), json!("64")),
            (json!("0"), json!(true)),
            (json!("0"), Value::Null),
            (json!("0"), json!(-1)),
        ] {
            assert_eq!(
                decode(
                    "results_page",
                    json!({"schemaVersion":1,"query_ref":"q_1","offset":offset,"limit":limit})
                ),
                Err(ErrorCode::InvalidMessage)
            );
        }
        for bad in [json!(""), json!("q/1"), json!(1), json!("q".repeat(65))] {
            assert_eq!(
                decode(
                    "results_query_state",
                    json!({"schemaVersion":1,"query_ref":bad})
                ),
                Err(ErrorCode::InvalidMessage)
            );
        }
        assert_eq!(
            decode("results_all", json!({"schemaVersion":1})),
            Err(ErrorCode::UnknownOperation)
        );
    }

    #[test]
    fn unfiltered_query_reuses_publication_index_and_pages_are_bounded_and_ordered() {
        let (report, path) = report(&rows(300, &["T1"]), &["T1"]);
        let published = (run(1), report.clone());
        let mut results = Results::default();
        let opened = query(&mut results, &published, json!({"schemaVersion":1})).unwrap();
        // Ready at once from the verified publication index: no worker, no decode.
        assert_eq!(opened["state"], "ready");
        assert_eq!(opened["total_matches"], "300");
        assert_eq!(opened["total_diagnostics"], "300");
        assert!(results.candidate.is_none());
        // T-MIG-P6-02: the filter bar always sends its five fields; all empty
        // is the same unfiltered query, not a rescan.
        let blank =
            json!({"schemaVersion":1,"code":"","field":"","record":"","fiscal_code":"","line":""});
        let cleared = query(&mut results, &published, blank).unwrap();
        assert_eq!(cleared["state"], "ready");
        assert_eq!(cleared["total_matches"], "300");
        assert!(results.candidate.is_none());
        report
            .with_index(|index| {
                assert_eq!(index.decoded_rows(), 0);
                Ok(())
            })
            .unwrap();
        let opened = query(&mut results, &published, json!({"schemaVersion":1})).unwrap();
        let q = &opened["query_ref"];
        let mut seen = HashSet::new();
        let mut collected = Vec::new();
        for (offset, generation, count, next, more) in [
            ("0", "1", 128, "128", true),
            ("128", "2", 128, "256", true),
            ("256", "3", 44, "300", false),
        ] {
            let page = page(&mut results, q, offset, 128).unwrap();
            assert_eq!(keys(&page), sorted(&PAGE_KEYS));
            assert_eq!(page["page_generation"], generation);
            assert_eq!(page["offset"], offset);
            assert_eq!(page["next_offset"], next);
            assert_eq!(page["has_more"], more);
            assert_eq!(page["run"], published.0.dto());
            let rows = page["rows"].as_array().unwrap();
            assert_eq!(rows.len(), count);
            let start: u64 = offset.parse().unwrap();
            for (i, row) in rows.iter().enumerate() {
                assert_eq!(keys(row), sorted(&ROW_KEYS));
                assert_eq!(row["ordinal"], (start + i as u64).to_string());
                assert_eq!(row["trace_ref"], "trace_0");
                assert!(seen.insert(row["diagnostic_ref"].as_str().unwrap().to_owned()));
            }
            // References and previews only: since T-MIG-P6-01 message,
            // expected and the displayed value found travel as previews, but
            // never raw bytes, context, record identity or paths.
            let text = page.to_string();
            for forbidden in [
                "raw_hex",
                "00 C3 A9",
                "context",
                "record_identity",
                "technical_ids",
                "SYNTHETIC-A",
                "errors.jsonl",
                path.parent().unwrap().to_str().unwrap(),
            ] {
                assert!(!text.contains(forbidden), "{forbidden} leaked");
            }
            collected.extend(lines(&page));
        }
        // Canonical JSONL order, no duplicates and no gaps across pages.
        assert_eq!(collected, (1..=300).collect::<Vec<_>>());
        assert!(results.current.as_ref().unwrap().pages.len() <= MAX_RETAINED_PAGES);
        // Field-level and record-level rows keep their distinct previews.
        let first = page(&mut results, q, "0", 2).unwrap();
        assert_eq!(
            first["rows"][0]["code"],
            json!({"schemaVersion":1,"text":"E_SYNTHETIC_RULE","complete":true})
        );
        assert_eq!(
            first["rows"][0]["field"],
            json!({"schemaVersion":1,"text":"Campo à","complete":true})
        );
        assert_eq!(first["rows"][1]["field"], Value::Null);
        let tail = page(&mut results, q, "300", 64).unwrap();
        assert_eq!(tail["rows"], json!([]));
        assert_eq!(tail["has_more"], false);
        assert_eq!(tail["next_offset"], "300");
        assert_eq!(
            page(&mut results, q, "301", 1),
            Err(ErrorCode::InvalidMessage)
        );
        // Lazy reads stay inside the shared 128-row cache.
        report
            .with_index(|index| {
                assert!(index.cache_len() <= 128);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn filtered_queries_are_exact_and_combinable_and_built_off_the_main_thread() {
        let mut rows = rows(40, &["T1", "T2"]);
        rows[5]["record_identity"]["sioc_record_id"] = "SYNTH-RECORD".into();
        rows[5]["record_identity"]["fiscal_code"] = "SYNTH-FISCAL".into();
        let (report, _) = report(&rows, &["T1", "T2"]);
        let published = (run(1), report);
        let mut results = Results::default();
        let expect = |keep: &dyn Fn(&Value) -> bool| -> Vec<u64> {
            rows.iter()
                .filter(|row| keep(row))
                .map(|row| row["line"].as_u64().unwrap())
                .collect()
        };
        let cases: Vec<(Value, Vec<u64>)> = vec![
            (
                json!({"code":"E_SYNTHETIC_RULE"}),
                expect(&|r| r["code"] == "E_SYNTHETIC_RULE"),
            ),
            (json!({"code":"e_synthetic_rule"}), vec![]),
            (json!({"code":" E_SYNTHETIC_RULE"}), vec![]),
            (json!({"field":"F1"}), expect(&|r| r["field"]["id"] == "F1")),
            (json!({"record":"SYNTH-RECORD"}), vec![6]),
            (json!({"fiscal_code":"SYNTH-FISCAL"}), vec![6]),
            (json!({"line":"007"}), vec![7]),
            (
                json!({"trace_ref":"trace_1"}),
                expect(&|r| r["trace"] == "T2"),
            ),
            (
                json!({"code":"E_RECORD_LENGTH","trace_ref":"trace_0","line":"10"}),
                vec![10],
            ),
            (json!({"code":"E_RECORD_LENGTH","line":"9"}), vec![]),
        ];
        for (mut filters, expected) in cases {
            filters["schemaVersion"] = 1.into();
            let opened = query(&mut results, &published, filters).unwrap();
            assert_eq!(opened["state"], "building");
            assert_eq!(opened["total_matches"], Value::Null);
            assert!(results.candidate.is_some());
            let ready = settle(&mut results, &opened["query_ref"]);
            assert_eq!(ready["state"], "ready");
            assert_eq!(ready["total_matches"], expected.len().to_string());
            assert_eq!(ready["total_diagnostics"], "40");
            let page = page(&mut results, &opened["query_ref"], "0", 128).unwrap();
            assert_eq!(lines(&page), expected);
        }
        let only_t2 = query(
            &mut results,
            &published,
            json!({"schemaVersion":1,"trace_ref":"trace_1"}),
        )
        .unwrap();
        settle(&mut results, &only_t2["query_ref"]);
        let page = page(&mut results, &only_t2["query_ref"], "0", 128).unwrap();
        assert!(
            page["rows"]
                .as_array()
                .unwrap()
                .iter()
                .all(|r| r["trace_ref"] == "trace_1")
        );
        for unknown in ["trace_2", "trace_01", "t0"] {
            assert_eq!(
                query(
                    &mut results,
                    &published,
                    json!({"schemaVersion":1,"trace_ref":unknown})
                ),
                Err(ErrorCode::StaleContext)
            );
        }
    }

    #[test]
    fn candidates_publish_atomically_and_failures_never_replace_the_current_query() {
        let (report, path) = report(&rows(20, &["T1"]), &["T1"]);
        let published = (run(1), report.clone());
        let mut results = Results::default();
        let all = query(&mut results, &published, json!({"schemaVersion":1})).unwrap();
        let old = &all["query_ref"];
        // A building candidate is not paged and does not hide the old query.
        let filter = ReportFilter {
            code: "E_RECORD_LENGTH".into(),
            ..Default::default()
        };
        let (pending, sender) = held(&mut results, &published, filter.clone());
        assert_eq!(
            state(&mut results, &json!(pending)).unwrap()["state"],
            "building"
        );
        assert_eq!(
            page(&mut results, &json!(pending), "0", 1),
            Err(ErrorCode::InvalidState)
        );
        assert_eq!(
            page(&mut results, old, "0", 3).unwrap()["rows"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
        // Scan failure: explicit failed state for its own ref, old query intact.
        sender.send(Err(ReportIndexError::CountMismatch)).unwrap();
        wait_finished(&results);
        assert_eq!(
            state(&mut results, &json!(pending)).unwrap()["state"],
            "failed"
        );
        assert_eq!(state(&mut results, old).unwrap()["state"], "ready");
        assert_eq!(
            page(&mut results, old, "0", 1).unwrap()["total_matches"],
            "20"
        );
        assert_eq!(
            page(&mut results, &json!(pending), "0", 1),
            Err(ErrorCode::StaleContext)
        );
        // A successful candidate replaces the current query only when joined.
        let (next, sender) = held(&mut results, &published, filter.clone());
        let index =
            JsonlReportIndex::scan(report.open().unwrap(), report.summary(), &filter, || false)
                .unwrap();
        sender.send(Ok(index)).unwrap();
        wait_finished(&results);
        assert_eq!(
            state(&mut results, &json!(next)).unwrap()["total_matches"],
            "10"
        );
        assert_eq!(state(&mut results, old), Err(ErrorCode::StaleContext));
        assert_eq!(
            state(&mut results, &json!(pending)),
            Err(ErrorCode::StaleContext)
        );
        // Closing cancels a building candidate; replacing it joins its worker.
        let (closing, _sender) = held(&mut results, &published, filter);
        results.cancel_on_close();
        assert!(results.candidate.as_ref().unwrap().token.is_cancelled());
        let superseding = query(
            &mut results,
            &published,
            json!({"schemaVersion":1,"line":"2"}),
        )
        .unwrap();
        assert_eq!(
            state(&mut results, &json!(closing)),
            Err(ErrorCode::StaleContext)
        );
        assert_eq!(
            settle(&mut results, &superseding["query_ref"])["total_matches"],
            "1"
        );
        // Real I/O corruption after publication: the rescan fails explicitly.
        fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"{\"truncated\"")
            .unwrap();
        let broken = query(
            &mut results,
            &published,
            json!({"schemaVersion":1,"line":"3"}),
        )
        .unwrap();
        assert_eq!(
            settle(&mut results, &broken["query_ref"])["state"],
            "failed"
        );
        assert_eq!(
            page(&mut results, &superseding["query_ref"], "0", 1).unwrap()["rows"][0]["line"],
            "2"
        );
    }

    #[test]
    fn page_registry_is_bounded_and_rows_reject_a_changed_spool() {
        let (report, _) = report(&rows(10, &["T1"]), &["T1"]);
        let published = (run(1), report);
        let mut results = Results::default();
        let all = query(&mut results, &published, json!({"schemaVersion":1})).unwrap();
        let q = &all["query_ref"];
        for offset in ["0", "2", "4", "6"] {
            page(&mut results, q, offset, 2).unwrap();
        }
        let current = results.current.as_ref().unwrap();
        assert_eq!(current.pages.len(), MAX_RETAINED_PAGES);
        assert_eq!(current.generation, 4);
        // A row that no longer satisfies the immutable filter is a technical error.
        results.current.as_mut().unwrap().filter.code = "E_OTHER".into();
        assert_eq!(
            page(&mut results, q, "0", 1),
            Err(ErrorCode::TechnicalFailure)
        );
        assert_eq!(results.current.as_ref().unwrap().generation, 4);
    }

    #[test]
    fn queries_are_bound_to_the_published_run() {
        let (report, _) = report(&rows(4, &["T1"]), &["T1"]);
        let published = (run(1), report);
        let mut results = Results::default();
        let filters = json!({"schemaVersion":1});
        assert_eq!(
            call(
                &mut results,
                "results_query",
                json!({"schemaVersion":1,"run":run(1).dto(),"filters":filters}),
                None
            ),
            Err(ErrorCode::InvalidState)
        );
        assert_eq!(
            call(
                &mut results,
                "results_query",
                json!({"schemaVersion":1,"run":run(2).dto(),"filters":filters}),
                Some(&published)
            ),
            Err(ErrorCode::StaleContext)
        );
        let opened = query(&mut results, &published, filters).unwrap();
        results.reset();
        assert_eq!(
            page(&mut results, &opened["query_ref"], "0", 1),
            Err(ErrorCode::StaleContext)
        );
        assert_eq!(
            page(&mut results, &json!("q_unknown"), "0", 1),
            Err(ErrorCode::StaleContext)
        );
    }

    #[test]
    fn a_new_published_report_invalidates_queries_of_the_previous_run() {
        let (_directory, intent, refs) = fixture();
        let mut jobs = Jobs::default();
        let terminal = |jobs: &mut Jobs| -> Value {
            let deadline = Instant::now() + Duration::from_secs(30);
            loop {
                let bytes = jobs
                    .dispatch(JobRequest::State { run: None }, "state")
                    .unwrap();
                let frame: Value = serde_json::from_slice(&bytes).unwrap();
                if frame["payload"]["state"] == "terminal" {
                    return frame["payload"].clone();
                }
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(5));
            }
        };
        let results = |jobs: &mut Jobs, payload: Value, operation: &str| {
            let request = ResultsRequest::decode(operation, payload.as_object().unwrap())?;
            jobs.results(request, "results")
                .map(|bytes| serde_json::from_slice::<Value>(&bytes).unwrap()["payload"].clone())
        };
        assert_eq!(
            results(
                &mut jobs,
                json!({"schemaVersion":1,"run":run(1).dto(),"filters":{"schemaVersion":1}}),
                "results_query"
            ),
            Err(ErrorCode::InvalidState)
        );
        jobs.start(
            INSTANCE,
            "start_1",
            "plan_1".into(),
            intent.clone(),
            refs.clone(),
        )
        .unwrap();
        let first = terminal(&mut jobs)["run"].clone();
        let opened = results(
            &mut jobs,
            json!({"schemaVersion":1,"run":first,"filters":{"schemaVersion":1}}),
            "results_query",
        )
        .unwrap();
        assert_eq!(opened["total_matches"], "1");
        let q = opened["query_ref"].clone();
        let page = json!({"schemaVersion":1,"query_ref":q,"offset":"0","limit":64});
        assert_eq!(
            results(&mut jobs, page.clone(), "results_page").unwrap()["rows"][0]["code"]["text"],
            "E_RECORD_LENGTH"
        );
        jobs.dispatch(
            JobRequest::Acknowledge {
                run: RunRef::decode(&first).unwrap(),
            },
            "ack",
        )
        .unwrap();
        jobs.start(INSTANCE, "start_2", "plan_1".into(), intent, refs)
            .unwrap();
        let second = terminal(&mut jobs)["run"].clone();
        assert_ne!(first, second);
        assert_eq!(
            results(&mut jobs, page, "results_page"),
            Err(ErrorCode::StaleContext)
        );
        assert_eq!(
            results(
                &mut jobs,
                json!({"schemaVersion":1,"run":first,"filters":{"schemaVersion":1}}),
                "results_query"
            ),
            Err(ErrorCode::StaleContext)
        );
        assert!(
            results(
                &mut jobs,
                json!({"schemaVersion":1,"run":second,"filters":{"schemaVersion":1}}),
                "results_query"
            )
            .is_ok()
        );
    }

    #[test]
    fn previews_are_utf8_bounded_and_a_full_page_fits_the_message_limit() {
        assert_eq!(preview(&"x".repeat(256))["complete"], true);
        assert_eq!(preview(&"x".repeat(257))["text"], "x".repeat(256));
        assert_eq!(preview(&"x".repeat(257))["complete"], false);
        // 'à' is two bytes: the cut stays on a character boundary.
        let cut = preview(&format!("x{}", "à".repeat(200)));
        assert_eq!(cut["text"], format!("x{}", "à".repeat(127)));
        assert_eq!(cut["complete"], false);
        // Worst case for escaping: control characters expand to \u00XX.
        let base = base();
        let rows: Vec<Value> = (0..128)
            .map(|i| {
                let mut row = base[0].clone();
                row["line"] = (i as u64 + 1).into();
                row["field"]["name"] = "\u{1}".repeat(400).into();
                // T-MIG-P6-01: the row texts at their worst case as well.
                row["message"] = "\u{1}".repeat(400).into();
                row["expected"] = "\u{1}".repeat(400).into();
                row["actual"]["display"] = "\u{1}".repeat(400).into();
                row
            })
            .collect();
        let (report, _) = report(&rows, &["T1"]);
        let published = (run(1), report);
        let mut results = Results::default();
        let q = query(&mut results, &published, json!({"schemaVersion":1})).unwrap()["query_ref"]
            .clone();
        let bytes = results
            .dispatch(
                decode(
                    "results_page",
                    json!({"schemaVersion":1,"query_ref":q,"offset":"0","limit":128}),
                )
                .unwrap(),
                None,
                "a".repeat(64).as_str(),
            )
            .unwrap();
        assert!(
            bytes.len() <= ipc::MAX_BYTES,
            "page is {} bytes",
            bytes.len()
        );
        let frame: Value = serde_json::from_slice(&bytes).unwrap();
        for row in frame["payload"]["rows"].as_array().unwrap() {
            assert_eq!(row["field"]["text"].as_str().unwrap().len(), 256);
            assert_eq!(row["field"]["complete"], false);
            for text in ["message", "expected", "actual"] {
                assert_eq!(
                    row[text]["text"].as_str().unwrap().len(),
                    ROW_TEXT_BYTES,
                    "{text}"
                );
                assert_eq!(row[text]["complete"], false, "{text}");
            }
        }
    }

    /// T-MIG-P6-01: the row reason is the canonical diagnostic's message,
    /// expected text and displayed value found, each a bounded preview;
    /// absent ones are null, never an invented text.
    #[test]
    fn rows_carry_the_reason_as_bounded_previews() {
        let (report, _) = report(&base(), &["T1"]);
        let published = (run(1), report);
        let mut results = Results::default();
        let q = query(&mut results, &published, json!({"schemaVersion":1})).unwrap()["query_ref"]
            .clone();
        let first = page(&mut results, &q, "0", 2).unwrap();
        let field = &first["rows"][0];
        assert_eq!(
            field["message"],
            json!({"schemaVersion":1,"text":"Errore sintetico \"à\".","complete":true})
        );
        assert_eq!(
            field["expected"],
            json!({"schemaVersion":1,"text":"testo","complete":true})
        );
        assert_eq!(
            field["actual"],
            json!({"schemaVersion":1,"text":"\\0é","complete":true})
        );
        let record = &first["rows"][1];
        assert_eq!(record["message"]["text"], "Record corto\nriga.");
        assert_eq!(record["expected"], Value::Null);
        assert_eq!(record["actual"], Value::Null);
        // 'à' is two bytes: a long value is cut on a character boundary.
        let cut = bounded_preview(&"à".repeat(100), ROW_TEXT_BYTES);
        assert_eq!(cut["text"], "à".repeat(64));
        assert_eq!(cut["complete"], false);
    }

    /// T-MIG-P6-01: the overview reads the verified summary and the per-trace
    /// counts of the publication index, lists every scope trace in order
    /// (zero included) and is bound to the published run.
    #[test]
    fn overview_counts_come_from_the_verified_report_and_its_scope() {
        let all = rows(10, &["S", "T"]);
        let (report, _) = report(&all, &["S", "T", "U"]);
        let published = (run(1), report);
        let ask = |run_ref: RunRef, published: Option<&(RunRef, Arc<Report>)>| {
            Results::default().dispatch(
                decode(
                    "results_overview",
                    json!({"schemaVersion":1,"run":run_ref.dto()}),
                )
                .unwrap(),
                published,
                "a",
            )
        };
        let bytes = ask(run(1), Some(&published)).unwrap();
        let frame: Value = serde_json::from_slice(&bytes).unwrap();
        let overview = &frame["payload"];
        assert_eq!(overview["records_processed"], "10");
        assert_eq!(overview["valid_records"], "0");
        assert_eq!(overview["invalid_records"], "10");
        assert_eq!(overview["total_errors"], "10");
        assert_eq!(
            overview["by_code"],
            json!([
                {"schemaVersion":1,"code":{"schemaVersion":1,"text":"E_RECORD_LENGTH","complete":true},"count":"5"},
                {"schemaVersion":1,"code":{"schemaVersion":1,"text":"E_SYNTHETIC_RULE","complete":true},"count":"5"}
            ])
        );
        let traces: Vec<(String, String, String)> = overview["by_trace"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| {
                (
                    t["trace_ref"].as_str().unwrap().into(),
                    t["trace"]["text"].as_str().unwrap().into(),
                    t["count"].as_str().unwrap().into(),
                )
            })
            .collect();
        // rows() cycles traces every two rows: S gets 6, T 4, U none.
        assert_eq!(
            traces,
            vec![
                ("trace_0".into(), "S".into(), "6".into()),
                ("trace_1".into(), "T".into(), "4".into()),
                ("trace_2".into(), "U".into(), "0".into())
            ]
        );
        assert_eq!(ask(run(2), Some(&published)), Err(ErrorCode::StaleContext));
        assert_eq!(ask(run(1), None), Err(ErrorCode::InvalidState));
        assert_eq!(
            decode(
                "results_overview",
                json!({"schemaVersion":1,"run":run(1).dto(),"x":1})
            ),
            Err(ErrorCode::InvalidMessage)
        );
    }

    #[test]
    fn one_million_diagnostics_stay_in_rust_and_pages_stay_bounded() {
        // Declared synthetic browsing load: one canonical record-level row with
        // distinct line numbers, not a fabricated clinical result.
        const COUNT: u64 = 1_000_000;
        let template =
            String::from_utf8(FIXTURE.split(|b| *b == b'\n').nth(1).unwrap().to_vec()).unwrap();
        let (head, tail) = template.split_once("\"line\":8,").unwrap();
        let directory = tempfile::tempdir().unwrap();
        {
            let mut writer =
                BufWriter::new(fs::File::create(directory.path().join("errors.jsonl")).unwrap());
            for line in 1..=COUNT {
                writeln!(writer, "{head}\"line\":{line},{tail}").unwrap();
            }
            writer.flush().unwrap();
        }
        let (_input, intent, _) = fixture();
        let summary = summary(
            COUNT,
            BTreeMap::from([("E_RECORD_LENGTH".to_owned(), COUNT)]),
            &["T1"],
        );
        let report = Arc::new(
            Report::publish(directory, summary, intent.clone(), captured(&intent)).unwrap(),
        );
        let published = (run(1), report.clone());
        let mut results = Results::default();
        let all = query(&mut results, &published, json!({"schemaVersion":1})).unwrap();
        assert_eq!(all["total_matches"], "1000000");
        let last = page(&mut results, &all["query_ref"], "999936", 128).unwrap();
        assert_eq!(last["rows"].as_array().unwrap().len(), 64);
        assert_eq!(lines(&last).last(), Some(&COUNT));
        assert_eq!(last["has_more"], false);
        let first = page(&mut results, &all["query_ref"], "0", 128).unwrap();
        assert_eq!(first["rows"].as_array().unwrap().len(), 128);
        assert!(first.to_string().len() < ipc::MAX_BYTES / 4);
        report
            .with_index(|index| {
                assert_eq!(index.offset_bytes(), COUNT as usize * 8);
                assert!(index.cache_len() <= 128);
                assert_eq!(index.decoded_rows(), 192);
                Ok(())
            })
            .unwrap();
        let one = query(
            &mut results,
            &published,
            json!({"schemaVersion":1,"line":"500000"}),
        )
        .unwrap();
        assert_eq!(one["state"], "building");
        let ready = settle(&mut results, &one["query_ref"]);
        assert_eq!(ready["total_matches"], "1");
        assert_eq!(ready["total_diagnostics"], "1000000");
        assert_eq!(
            lines(&page(&mut results, &one["query_ref"], "0", 128).unwrap()),
            vec![500_000]
        );
    }
}
