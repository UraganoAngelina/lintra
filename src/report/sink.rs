use super::atomic::OutputOperation;
use crate::domain::{InvalidDiagnostic, SummaryBuildError, ValidationError, ValidationSummary};
use std::{error::Error, fmt, io};

/// Technical report failure, never a normative violation.
#[derive(Debug)]
#[non_exhaustive]
pub enum ReportError {
    /// Filesystem publication failure, distinct from JSON/diagnostic errors.
    Output {
        operation: OutputOperation,
        source: io::Error,
    },
    /// Cleanup can fail independently: retain the initial failure as well as
    /// the cleanup cause instead of hiding either one behind a generic message.
    OutputCleanup {
        original: Option<Box<ReportError>>,
        source: io::Error,
    },
    InvalidSummary(SummaryBuildError),
    InvalidTimestamp,
    Write(io::Error),
    Serialize(serde_json::Error),
    InvalidDiagnostic(InvalidDiagnostic),
    PreviousFailure,
}
impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Output { .. } => "cannot prepare or publish validation report",
            Self::OutputCleanup { .. } => "cannot remove incomplete validation report",
            Self::InvalidSummary(_) => "cannot write inconsistent validation summary",
            Self::InvalidTimestamp => "summary timestamp cannot be represented as UTC RFC3339",
            Self::Write(_) => "cannot write validation report",
            Self::Serialize(_) => "cannot serialize validation report",
            Self::InvalidDiagnostic(_) => "cannot emit malformed validation diagnostic",
            Self::PreviousFailure => "report sink cannot resume after a previous failure",
        })
    }
}
impl Error for ReportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Output { source, .. } | Self::OutputCleanup { source, .. } => Some(source),
            Self::InvalidSummary(source) => Some(source),
            Self::InvalidTimestamp => None,
            Self::Write(source) => Some(source),
            Self::Serialize(source) => Some(source),
            Self::InvalidDiagnostic(source) => Some(source),
            Self::PreviousFailure => None,
        }
    }
}

/// Streaming callbacks. An error is terminal: the producer must stop the run.
pub trait ValidationSink {
    fn on_error(&mut self, error: &ValidationError) -> Result<(), ReportError>;
    fn on_summary(&mut self, summary: &ValidationSummary) -> Result<(), ReportError>;
}

/// In-memory collection for tests and small cases only, never a streaming default.
#[derive(Debug, Default)]
pub struct VecSink {
    errors: Vec<ValidationError>,
    summaries: Vec<ValidationSummary>,
    failed: bool,
}
impl VecSink {
    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }
    pub fn summaries(&self) -> &[ValidationSummary] {
        &self.summaries
    }
}
impl ValidationSink for VecSink {
    fn on_error(&mut self, error: &ValidationError) -> Result<(), ReportError> {
        if self.failed {
            return Err(ReportError::PreviousFailure);
        }
        if let Err(source) = error.validate() {
            self.failed = true;
            return Err(ReportError::InvalidDiagnostic(source));
        }
        self.errors.push(error.clone());
        Ok(())
    }
    fn on_summary(&mut self, summary: &ValidationSummary) -> Result<(), ReportError> {
        if self.failed {
            return Err(ReportError::PreviousFailure);
        }
        self.summaries.push(summary.clone());
        Ok(())
    }
}
