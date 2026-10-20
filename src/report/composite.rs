use super::sink::{ReportError, ValidationSink};
use crate::domain::{ValidationError, ValidationSummary};

/// Ordered fan-out; stops at the first failing sink and remains terminal.
/// Earlier children may already have written: fan-out does not imply rollback.
pub struct CompositeSink<'a> {
    sinks: Vec<Box<dyn ValidationSink + 'a>>,
    failed: bool,
}
impl<'a> CompositeSink<'a> {
    pub fn new(sinks: Vec<Box<dyn ValidationSink + 'a>>) -> Self {
        Self {
            sinks,
            failed: false,
        }
    }
}
impl ValidationSink for CompositeSink<'_> {
    fn on_error(&mut self, error: &ValidationError) -> Result<(), ReportError> {
        if self.failed {
            return Err(ReportError::PreviousFailure);
        }
        if let Err(source) = error.validate() {
            self.failed = true;
            return Err(ReportError::InvalidDiagnostic(source));
        }
        for sink in &mut self.sinks {
            if let Err(error) = sink.on_error(error) {
                self.failed = true;
                return Err(error);
            }
        }
        Ok(())
    }
    fn on_summary(&mut self, summary: &ValidationSummary) -> Result<(), ReportError> {
        if self.failed {
            return Err(ReportError::PreviousFailure);
        }
        for sink in &mut self.sinks {
            if let Err(error) = sink.on_summary(summary) {
                self.failed = true;
                return Err(error);
            }
        }
        Ok(())
    }
}
