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
    UnreportedReferenceEvidence,
    UnreportedFullEvidence,
    InconsistentReferenceAudit,
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
            Self::UnreportedFullEvidence => {
                f.write_str("Full execution requires an explicit evidence summary contract")
            }
            Self::InconsistentReferenceAudit => {
                f.write_str("reference audit is inconsistent with the run")
            }
            Self::UnreportedReferenceEvidence => {
                f.write_str("reference completeness requires an explicit summary contract")
            }
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
        if let Some(audit) = &self.reference_validation
            && (!audit.valid(interrupted || self.skipped_rules > 0)
                || audit.verified_versions() != self.reference_data_versions
                || audit
                    .controls
                    .iter()
                    .any(|control| match &self.scope_summary {
                        ScopeSummary::Trace { trace_id } => trace_id != &control.trace_id,
                        ScopeSummary::Flow { trace_ids } => !trace_ids.contains(&control.trace_id),
                    }))
        {
            return Err(SummaryBuildError::InconsistentReferenceAudit);
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
    unreported_reference_evidence: bool,
    unreported_full_evidence: bool,
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
            unreported_reference_evidence: false,
            unreported_full_evidence: false,
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
    /// Batch known omitted physical record/rule evaluations without an O(n)
    /// loop when cancellation leaves a large projected context unevaluated.
    pub(crate) fn rules_skipped(&mut self, count: u64) -> Result<(), SummaryBuildError> {
        if self.failed {
            return Err(SummaryBuildError::PreviousFailure);
        }
        match self.skipped.checked_add(count) {
            Some(next) => {
                self.skipped = next;
                Ok(())
            }
            None => {
                self.failed = true;
                Err(SummaryBuildError::CounterOverflow(
                    SummaryCounter::SkippedRules,
                ))
            }
        }
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
    pub(crate) fn mark_unreported_reference_evidence(&mut self) {
        self.unreported_reference_evidence = true;
    }
    pub(crate) fn begin_full(
        &mut self,
        key: &SchemaKey,
        traces: &[crate::TraceId],
    ) -> Result<(), SummaryBuildError> {
        if self.failed
            || self.unreported_full_evidence
            || self.processed != 0
            || self.errors != 0
            || self.valid != 0
            || self.invalid != 0
            || self.skipped != 0
            || self.cancelled
            || self.truncated
        {
            return Err(SummaryBuildError::PreviousFailure);
        }
        if &self.key != key
            || !matches!(&self.scope, ScopeSummary::Flow { trace_ids } if trace_ids == traces)
        {
            return Err(SummaryBuildError::InconsistentRecordCounts);
        }
        self.unreported_full_evidence = true;
        Ok(())
    }
    pub(crate) fn invalidate_previously_valid_record(&mut self) -> Result<(), SummaryBuildError> {
        if self.failed {
            return Err(SummaryBuildError::PreviousFailure);
        }
        let Some(valid) = self.valid.checked_sub(1) else {
            self.failed = true;
            return Err(SummaryBuildError::InconsistentRecordCounts);
        };
        let Some(invalid) = self.invalid.checked_add(1) else {
            self.failed = true;
            return Err(SummaryBuildError::CounterOverflow(SummaryCounter::Invalid));
        };
        self.valid = valid;
        self.invalid = invalid;
        Ok(())
    }
    pub fn counters(&self) -> ValidationCounters {
        ValidationCounters {
            records_processed: self.processed,
            valid_records: self.valid,
            invalid_records: self.invalid,
            total_errors: self.errors,
            skipped_evaluations: self.skipped,
        }
    }
    /// Checked Result refines DET §58 to report counter/audit failures explicitly.
    pub fn finish(self, metadata: RunMetadata) -> Result<ValidationSummary, SummaryBuildError> {
        self.finish_inner(metadata, None)
    }
    pub fn finish_with_reference_data(
        self,
        metadata: RunMetadata,
        mut audit: super::ReferenceValidationSummary,
    ) -> Result<ValidationSummary, SummaryBuildError> {
        if self.unreported_reference_evidence != audit.has_non_definitive_evaluations() {
            return Err(SummaryBuildError::InconsistentReferenceAudit);
        }
        audit.completeness = if self.cancelled
            || self.truncated
            || self.skipped > 0
            || audit.has_non_definitive_evaluations()
        {
            super::ValidationCompleteness::Partial
        } else {
            super::ValidationCompleteness::Complete
        };
        self.finish_inner(metadata, Some(audit))
    }
    fn finish_inner(
        self,
        metadata: RunMetadata,
        reference_validation: Option<super::ReferenceValidationSummary>,
    ) -> Result<ValidationSummary, SummaryBuildError> {
        if self.failed {
            return Err(SummaryBuildError::PreviousFailure);
        }
        if self.unreported_full_evidence {
            return Err(SummaryBuildError::UnreportedFullEvidence);
        }
        if self.unreported_reference_evidence && reference_validation.is_none() {
            return Err(SummaryBuildError::UnreportedReferenceEvidence);
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
            reference_validation,
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

/// Snapshot of the single authoritative builder; no independent Full counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationCounters {
    pub records_processed: u64,
    pub valid_records: u64,
    pub invalid_records: u64,
    pub total_errors: u64,
    pub skipped_evaluations: u64,
}
