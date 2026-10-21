use super::{
    InputAuditCompletion, InvalidInputAudit, InvalidValidationCode, RunMetadata, RunStatus,
    SchemaKey, ScopeSummary, ValidationCode, ValidationSummary,
};
use std::{collections::BTreeMap, error::Error, fmt, time::Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryCounter {
    Processed,
    Valid,
    Invalid,
    Errors,
    ErrorsByCode,
    SkippedRules,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SummaryBuildError {
    CounterOverflow(SummaryCounter),
    PreviousFailure,
    InconsistentRecordCounts,
    InconsistentErrorCounts,
    InconsistentStatus,
    InvalidCode(InvalidValidationCode),
    InvalidInputAudit(InvalidInputAudit),
    IncompleteInput,
    MissingInputAudit,
    InputOutsideScope,
    EmptyMetadata,
}
impl fmt::Display for SummaryBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CounterOverflow(counter) => write!(f, "summary counter overflow: {counter:?}"),
            Self::PreviousFailure => f.write_str("summary builder cannot resume after failure"),
            Self::InconsistentRecordCounts => f.write_str("summary record counts are inconsistent"),
            Self::InconsistentErrorCounts => f.write_str("summary error counts are inconsistent"),
            Self::InconsistentStatus => {
                f.write_str("summary status disagrees with validation counts")
            }
            Self::InvalidCode(_) => f.write_str("summary has an invalid validation code"),
            Self::InvalidInputAudit(_) => f.write_str("summary has invalid input audit metadata"),
            Self::IncompleteInput => {
                f.write_str("normally completed run has an incomplete input audit")
            }
            Self::MissingInputAudit => f.write_str("processed records have no input audit"),
            Self::InputOutsideScope => {
                f.write_str("summary input audit is outside the selected scope")
            }
            Self::EmptyMetadata => f.write_str("summary metadata is empty"),
        }
    }
}
impl Error for SummaryBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidCode(e) => Some(e),
            Self::InvalidInputAudit(e) => Some(e),
            _ => None,
        }
    }
}

impl ValidationSummary {
    pub fn validate(&self) -> Result<(), SummaryBuildError> {
        if self.schema_key.flow_id.0.is_empty()
            || self.schema_key.version.0.is_empty()
            || self.validator_version.is_empty()
            || self.schema_hash.is_empty()
        {
            return Err(SummaryBuildError::EmptyMetadata);
        }
        let classified = self.valid_records.checked_add(self.invalid_records).ok_or(
            SummaryBuildError::CounterOverflow(SummaryCounter::Processed),
        )?;
        if self.records_processed > 0 && self.inputs.is_empty() {
            return Err(SummaryBuildError::MissingInputAudit);
        }
        let interrupted = self.run_status == RunStatus::Cancelled || self.truncated;
        if classified > self.records_processed
            || (!interrupted && classified != self.records_processed)
        {
            return Err(SummaryBuildError::InconsistentRecordCounts);
        }
        let mut total = 0u64;
        for (code, &count) in &self.errors_by_code {
            code.as_symbol().map_err(SummaryBuildError::InvalidCode)?;
            total = total
                .checked_add(count)
                .ok_or(SummaryBuildError::CounterOverflow(SummaryCounter::Errors))?;
        }
        if total != self.total_errors {
            return Err(SummaryBuildError::InconsistentErrorCounts);
        }
        let invalid = self.total_errors > 0 || self.invalid_records > 0;
        if (self.run_status == RunStatus::CompletedValid && invalid)
            || (self.run_status == RunStatus::CompletedInvalid && !invalid)
        {
            return Err(SummaryBuildError::InconsistentStatus);
        }
        for audit in &self.inputs {
            audit
                .validate()
                .map_err(SummaryBuildError::InvalidInputAudit)?;
            let in_scope = match &self.scope_summary {
                ScopeSummary::Trace { trace_id } => trace_id == &audit.trace_id,
                ScopeSummary::Flow { trace_ids } => trace_ids.contains(&audit.trace_id),
            };
            if !in_scope {
                return Err(SummaryBuildError::InputOutsideScope);
            }
            if !interrupted && !matches!(audit.completion, InputAuditCompletion::Complete { .. }) {
                return Err(SummaryBuildError::IncompleteInput);
            }
        }
        Ok(())
    }
}

/// Central checked counters; consumes the authoritative frozen metadata at finish.
pub struct ValidationSummaryBuilder {
    key: SchemaKey,
    scope: ScopeSummary,
    started: Instant,
    processed: u64,
    valid: u64,
    invalid: u64,
    errors: u64,
    skipped: u64,
    by_code: BTreeMap<ValidationCode, u64>,
    cancelled: bool,
    truncated: bool,
    failed: bool,
}
impl ValidationSummaryBuilder {
    pub fn new(key: SchemaKey, scope: ScopeSummary) -> Self {
        Self {
            key,
            scope,
            started: Instant::now(),
            processed: 0,
            valid: 0,
            invalid: 0,
            errors: 0,
            skipped: 0,
            by_code: BTreeMap::new(),
            cancelled: false,
            truncated: false,
            failed: false,
        }
    }
    fn increment(&mut self, counter: SummaryCounter) -> Result<(), SummaryBuildError> {
        if self.failed {
            return Err(SummaryBuildError::PreviousFailure);
        }
        let value = match counter {
            SummaryCounter::Processed => &mut self.processed,
            SummaryCounter::Valid => &mut self.valid,
            SummaryCounter::Invalid => &mut self.invalid,
            SummaryCounter::SkippedRules => &mut self.skipped,
            _ => unreachable!("only independent counters use increment"),
        };
        match value.checked_add(1) {
            Some(next) => {
                *value = next;
                Ok(())
            }
            None => {
                self.failed = true;
                Err(SummaryBuildError::CounterOverflow(counter))
            }
        }
    }
    pub fn record_processed(&mut self) -> Result<(), SummaryBuildError> {
        self.increment(SummaryCounter::Processed)
    }
    pub fn record_valid(&mut self) -> Result<(), SummaryBuildError> {
        self.increment(SummaryCounter::Valid)
    }
    pub fn record_invalid(&mut self) -> Result<(), SummaryBuildError> {
        self.increment(SummaryCounter::Invalid)
    }
    pub fn rule_skipped(&mut self) -> Result<(), SummaryBuildError> {
        self.increment(SummaryCounter::SkippedRules)
    }
    pub fn error_emitted(&mut self, code: &ValidationCode) -> Result<(), SummaryBuildError> {
        if self.failed {
            return Err(SummaryBuildError::PreviousFailure);
        }
        if let Err(e) = code.as_symbol() {
            self.failed = true;
            return Err(SummaryBuildError::InvalidCode(e));
        }
        let total = self.errors.checked_add(1);
        let count = self.by_code.get(code).copied().unwrap_or(0).checked_add(1);
        let (Some(total), Some(count)) = (total, count) else {
            self.failed = true;
            return Err(SummaryBuildError::CounterOverflow(if total.is_none() {
                SummaryCounter::Errors
            } else {
                SummaryCounter::ErrorsByCode
            }));
        };
        self.errors = total;
        self.by_code.insert(code.clone(), count);
        Ok(())
    }
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }
    pub fn truncate(&mut self) {
        self.truncated = true;
    }
    /// Checked Result refines DET §58 to report counter/audit failures explicitly.
    pub fn finish(self, metadata: RunMetadata) -> Result<ValidationSummary, SummaryBuildError> {
        if self.failed {
            return Err(SummaryBuildError::PreviousFailure);
        }
        let run_status = if self.cancelled {
            RunStatus::Cancelled
        } else if self.errors > 0 || self.invalid > 0 {
            RunStatus::CompletedInvalid
        } else {
            RunStatus::CompletedValid
        };
        let summary = ValidationSummary {
            started_at: metadata.started_at,
            validation_mode: metadata.validation_mode,
            run_status,
            schema_key: self.key,
            scope_summary: self.scope,
            records_processed: self.processed,
            valid_records: self.valid,
            invalid_records: self.invalid,
            total_errors: self.errors,
            errors_by_code: self.by_code,
            skipped_rules: self.skipped,
            truncated: self.truncated,
            validator_version: metadata.validator_version,
            schema_hash: metadata.schema_hash,
            reference_data_versions: metadata.reference_data_versions,
            elapsed: self.started.elapsed(),
            inputs: metadata.inputs,
        };
        summary.validate()?;
        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_counter_overflows_are_detected_and_terminal() {
        for kind in [
            SummaryCounter::Processed,
            SummaryCounter::Valid,
            SummaryCounter::Invalid,
            SummaryCounter::SkippedRules,
        ] {
            let mut b = ValidationSummaryBuilder::new(
                SchemaKey {
                    flow_id: super::super::FlowId("T".into()),
                    version: super::super::SchemaVersion("v1".into()),
                },
                ScopeSummary::Flow { trace_ids: vec![] },
            );
            b.processed = u64::MAX;
            b.valid = u64::MAX;
            b.invalid = u64::MAX;
            b.skipped = u64::MAX;
            assert_eq!(
                b.increment(kind),
                Err(SummaryBuildError::CounterOverflow(kind))
            );
            assert_eq!(
                b.record_processed(),
                Err(SummaryBuildError::PreviousFailure)
            );
        }
        for kind in [SummaryCounter::Errors, SummaryCounter::ErrorsByCode] {
            let mut b = ValidationSummaryBuilder::new(
                SchemaKey {
                    flow_id: super::super::FlowId("T".into()),
                    version: super::super::SchemaVersion("v1".into()),
                },
                ScopeSummary::Flow { trace_ids: vec![] },
            );
            if kind == SummaryCounter::Errors {
                b.errors = u64::MAX;
            } else {
                b.by_code.insert(ValidationCode::RequiredField, u64::MAX);
            }
            assert_eq!(
                b.error_emitted(&ValidationCode::RequiredField),
                Err(SummaryBuildError::CounterOverflow(kind))
            );
            assert_eq!(
                b.error_emitted(&ValidationCode::RequiredField),
                Err(SummaryBuildError::PreviousFailure)
            );
        }
    }
}
