use crate::domain::{InvalidDiagnostic, ValidationError, ValidationSummary};
use std::{error::Error, fmt, io};

/// Technical report failure, never a normative violation.
#[derive(Debug)]
#[non_exhaustive]
pub enum ReportError {
    Write(io::Error),
    InvalidDiagnostic(InvalidDiagnostic),
    PreviousFailure,
}
impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Write(_) => "cannot write validation report",
            Self::InvalidDiagnostic(_) => "cannot emit malformed validation diagnostic",
            Self::PreviousFailure => "report sink cannot resume after a previous failure",
        })
    }
}
impl Error for ReportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Write(source) => Some(source),
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
