//! Bounded post-analysis replay into existing core sinks. UI code neither
//! reimplements CSV/JSONL nor collects all diagnostics before exporting.
use crate::{
    CancellationToken, ReportError, RunStatus, ScopeSummary, ValidationSink, ValidationSummary,
    XlsxReportError,
};
use std::{
    collections::BTreeMap,
    error::Error,
    fmt,
    io::{self, BufRead},
};
#[derive(Debug)]
pub enum CanonicalReplayError {
    IncompleteRun,
    Cancelled,
    InvalidFraming,
    Read(io::Error),
    Decode(XlsxReportError),
    Report(ReportError),
    SummaryMismatch,
    CounterOverflow,
}
impl fmt::Display for CanonicalReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::IncompleteRun => "incomplete validation cannot produce a complete export",
            Self::Cancelled => "report export was cancelled",
            Self::InvalidFraming => {
                "canonical report requires one UTF-8 JSON diagnostic per LF-terminated line"
            }
            Self::Read(_) => "cannot read canonical report",
            Self::Decode(_) => "canonical report contains an invalid diagnostic",
            Self::Report(_) => "cannot write exported report",
            Self::SummaryMismatch => "canonical report does not match its validation summary",
            Self::CounterOverflow => "report diagnostic counter overflow",
        })
    }
}
impl Error for CanonicalReplayError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read(e) => Some(e),
            Self::Decode(e) => Some(e),
            Self::Report(e) => Some(e),
            _ => None,
        }
    }
}
/// The destination must remain private until this returns success. Counts,
/// histogram and scope are checked before on_summary, and errors stop replay;
/// no malformed row is skipped or silently converted into an empty export.
pub fn replay_canonical_report(
    mut reader: impl BufRead,
    summary: &ValidationSummary,
    sink: &mut dyn ValidationSink,
    cancellation: &CancellationToken,
) -> Result<(), CanonicalReplayError> {
    summary
        .validate()
        .map_err(|e| CanonicalReplayError::Report(ReportError::InvalidSummary(e)))?;
    if summary.run_status == RunStatus::Cancelled || summary.truncated {
        return Err(CanonicalReplayError::IncompleteRun);
    }
    let mut buffer = Vec::new();
    let mut count = 0u64;
    let mut histogram = BTreeMap::new();
    loop {
        if cancellation.is_cancelled() {
            return Err(CanonicalReplayError::Cancelled);
        }
        buffer.clear();
        if reader
            .read_until(b'\n', &mut buffer)
            .map_err(CanonicalReplayError::Read)?
            == 0
        {
            break;
        }
        if !buffer.ends_with(b"\n") || buffer.ends_with(b"\r\n") {
            return Err(CanonicalReplayError::InvalidFraming);
        }
        count = count
            .checked_add(1)
            .ok_or(CanonicalReplayError::CounterOverflow)?;
        let error = super::xlsx::read_canonical_diagnostic(&buffer, count)
            .map_err(CanonicalReplayError::Decode)?;
        let scope = match &summary.scope_summary {
            ScopeSummary::Trace { trace_id } => trace_id == &error.trace_id,
            ScopeSummary::Flow { trace_ids } => trace_ids.contains(&error.trace_id),
        };
        if error.flow_id != summary.schema_key.flow_id
            || error.schema_version != summary.schema_key.version
            || !scope
        {
            return Err(CanonicalReplayError::SummaryMismatch);
        }
        let entry = histogram.entry(error.code.clone()).or_insert(0u64);
        *entry = entry
            .checked_add(1)
            .ok_or(CanonicalReplayError::CounterOverflow)?;
        sink.on_error(&error)
            .map_err(CanonicalReplayError::Report)?;
    }
    if count != summary.total_errors || histogram != summary.errors_by_code {
        return Err(CanonicalReplayError::SummaryMismatch);
    }
    if cancellation.is_cancelled() {
        return Err(CanonicalReplayError::Cancelled);
    }
    sink.on_summary(summary)
        .map_err(CanonicalReplayError::Report)
}
