//! Summary-only text adapter. Record-level details remain in the JSONL sink.
use super::sink::{ReportError, ValidationSink};
use crate::{RunStatus, ScopeSummary, ValidationError, ValidationMode, ValidationSummary};
use std::{collections::BTreeMap, io::Write};

/// Owns only a writer and terminal-failure flag, never records or diagnostics.
/// Use CompositeSink with JsonlSink when both detailed and human output are needed.
pub struct HumanReadableSink<W> {
    writer: W,
    failed: bool,
}
impl<W: Write> HumanReadableSink<W> {
    /// The caller selects stdout/stderr/file; no global stream or path is assumed.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            failed: false,
        }
    }
    /// Inspect test-owned bytes without enabling mutation or bypassing failures.
    pub fn get_ref(&self) -> &W {
        &self.writer
    }
    /// Observe buffered writer failures before handing ownership back to a caller.
    pub fn finish(mut self) -> Result<W, ReportError> {
        if self.failed {
            return Err(ReportError::PreviousFailure);
        }
        self.writer.flush().map_err(ReportError::Write)?;
        Ok(self.writer)
    }
}
impl<W: Write> ValidationSink for HumanReadableSink<W> {
    /// Deliberately summary-only: acknowledge streamed diagnostics without storing
    /// or printing record values. Counts come from the authoritative final summary.
    fn on_error(&mut self, _: &ValidationError) -> Result<(), ReportError> {
        if self.failed {
            Err(ReportError::PreviousFailure)
        } else {
            Ok(())
        }
    }
    /// Any failure is terminal; retrying cannot append a misleading second report.
    fn on_summary(&mut self, summary: &ValidationSummary) -> Result<(), ReportError> {
        if self.failed {
            return Err(ReportError::PreviousFailure);
        }
        let result = render(&mut self.writer, summary);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}
/// Validate all metadata/codes before the first byte. Code sorting is bounded by
/// distinct code count, not the number of errors; formatting is locale-independent.
fn render(writer: &mut dyn Write, summary: &ValidationSummary) -> Result<(), ReportError> {
    summary.validate().map_err(ReportError::InvalidSummary)?;
    let codes = summary
        .errors_by_code
        .iter()
        .map(|(code, count)| {
            code.as_symbol()
                .map(|symbol| (symbol, *count))
                .map_err(|error| {
                    ReportError::InvalidSummary(crate::SummaryBuildError::InvalidCode(error))
                })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let result = match summary.run_status {
        RunStatus::CompletedValid => "VALID",
        RunStatus::CompletedInvalid => "INVALID",
        RunStatus::Cancelled => "CANCELLED",
    };
    let mode = match summary.validation_mode {
        ValidationMode::LocalOnly => "LocalOnly (local checks)",
        ValidationMode::Full => "Full",
    };
    // The closure uses io::Result so every formatting/flush failure retains its
    // original OS cause when converted once at this adapter boundary.
    let write = || -> std::io::Result<()> {
        writeln!(writer, "Result: {result}")?;
        writeln!(writer, "Mode: {mode}")?;
        if summary.reference_validation.is_some() || summary.full_validation.is_some() {
            writeln!(writer, "Completeness: {:?}", summary.completeness())?;
            if summary.completeness() == crate::ValidationCompleteness::Partial {
                writeln!(
                    writer,
                    "Partial validation: absence of violations does not establish full conformance."
                )?;
            }
            if let Some(audit) = &summary.reference_validation {
                for control in &audit.controls {
                    for (kind, count) in control.warnings() {
                        writeln!(
                            writer,
                            "Reference warning: {kind:?}; field {}; evaluations {count}",
                            control.field_id.0
                        )?;
                    }
                }
            }
        }
        if let Some(audit) = &summary.full_validation {
            for control in &audit.controls {
                if control.unknown > 0 || control.not_applicable > 0 || control.skipped > 0 {
                    writeln!(
                        writer,
                        "Cross control: {}; trace {}; inconclusive {}; not applicable {}; skipped {}",
                        control.rule_id,
                        control.trace_id.0,
                        control.unknown,
                        control.not_applicable,
                        control.skipped
                    )?;
                }
            }
        }
        writeln!(writer, "Flow: {}", summary.schema_key.flow_id.0)?;
        match &summary.scope_summary {
            ScopeSummary::Trace { trace_id } => writeln!(writer, "Trace: {}", trace_id.0)?,
            ScopeSummary::Flow { trace_ids } => {
                write!(writer, "Traces:")?;
                for id in trace_ids {
                    write!(writer, " {}", id.0)?;
                }
                writeln!(writer)?;
            }
        }
        writeln!(writer, "Schema version: {}", summary.schema_key.version.0)?;
        if summary.inputs.is_empty() {
            writeln!(writer, "Input: (none supplied)")?;
        }
        for input in &summary.inputs {
            // Debug string formatting quotes/escapes control characters while
            // retaining only the already-validated basename, never an absolute path.
            writeln!(
                writer,
                "Input: {:?} (trace {}, {} bytes)",
                input.file_name, input.trace_id.0, input.file_size_bytes
            )?;
        }
        writeln!(writer, "Records processed: {}", summary.records_processed)?;
        writeln!(writer, "Valid records: {}", summary.valid_records)?;
        writeln!(writer, "Invalid records: {}", summary.invalid_records)?;
        writeln!(writer, "Errors: {}", summary.total_errors)?;
        writeln!(writer, "Skipped rules: {}", summary.skipped_rules)?;
        writeln!(writer, "Truncated: {}", summary.truncated)?;
        writeln!(
            writer,
            "Elapsed: {}.{:09}s",
            summary.elapsed.as_secs(),
            summary.elapsed.subsec_nanos()
        )?;
        writeln!(writer)?;
        if summary.run_status == RunStatus::Cancelled {
            writeln!(writer, "Validation cancelled; results are incomplete.")?;
        } else if summary.truncated {
            writeln!(writer, "Validation truncated; results are incomplete.")?;
        } else if summary.total_errors == 0 {
            writeln!(writer, "No validation errors found in the selected mode.")?;
        }
        for (code, count) in codes {
            writeln!(writer, "{code}: {count}")?;
        }
        writer.flush()
    };
    write().map_err(ReportError::Write)
}
