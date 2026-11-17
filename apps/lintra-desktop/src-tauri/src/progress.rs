//! One latest sample, no notification queue or diagnostic data.
use crate::ipc::ErrorCode;
use lintra::{ProgressEvent, TraceId, ValidationPhase, ValidationScope};
use serde_json::{Value, json};

#[derive(Clone)]
pub(crate) struct Progress {
    sequence: u64,
    phase: Option<ValidationPhase>,
    trace: Option<usize>,
    trace_records: u64,
    cross_records: u64,
    traces: Vec<TraceId>,
    failed: bool,
    sealed: bool,
}
impl Progress {
    pub(crate) fn new(scope: &ValidationScope) -> Self {
        Self {
            sequence: 0,
            phase: None,
            trace: None,
            trace_records: 0,
            cross_records: 0,
            traces: match scope {
                ValidationScope::Trace(input) => vec![input.trace_id.clone()],
                ValidationScope::Flow(input) => {
                    input.traces.iter().map(|t| t.trace_id.clone()).collect()
                }
            },
            failed: false,
            sealed: false,
        }
    }
    /// Producer faults seal only progress; the service result stays authoritative.
    pub(crate) fn update(&mut self, event: &ProgressEvent) {
        if self.failed || self.sealed {
            return;
        }
        let valid = match event {
            ProgressEvent::RunStarted => self.sequence == 0,
            ProgressEvent::RunCompleted => {
                self.sealed = true;
                return;
            }
            ProgressEvent::PhaseStarted(phase) => {
                let rank = |p| match p {
                    ValidationPhase::Preflight => 0,
                    ValidationPhase::LocalValidation => 1,
                    ValidationPhase::CrossRecordValidation => 2,
                    ValidationPhase::FinalizingReport => 3,
                };
                if self.phase.is_some_and(|p| rank(p) > rank(*phase)) {
                    false
                } else {
                    self.phase = Some(*phase);
                    if *phase == ValidationPhase::CrossRecordValidation {
                        self.trace = None;
                        self.trace_records = 0;
                    }
                    true
                }
            }
            ProgressEvent::TraceStarted { trace_id } => {
                let trace = self.traces.iter().position(|id| id == trace_id);
                if self.phase != Some(ValidationPhase::LocalValidation) || trace.is_none() {
                    false
                } else {
                    self.trace = trace;
                    self.trace_records = 0;
                    true
                }
            }
            ProgressEvent::RecordsProcessed { trace_id, count }
            | ProgressEvent::TraceCompleted { trace_id, count } => {
                if self.phase != Some(ValidationPhase::LocalValidation)
                    || self.trace.is_none_or(|i| &self.traces[i] != trace_id)
                    || *count < self.trace_records
                {
                    false
                } else {
                    self.trace_records = *count;
                    true
                }
            }
            ProgressEvent::CrossRecordsProcessed { count } => {
                if self.phase != Some(ValidationPhase::CrossRecordValidation)
                    || *count < self.cross_records
                {
                    false
                } else {
                    self.cross_records = *count;
                    true
                }
            }
        };
        if !valid {
            self.failed = true;
        } else if let Some(next) = self.sequence.checked_add(1) {
            self.sequence = next;
        } else {
            self.failed = true;
        }
    }
    pub(crate) fn dto(&self, run: Value) -> Result<Value, ErrorCode> {
        if self.failed {
            return Err(ErrorCode::TechnicalFailure);
        }
        let phase = self.phase.map(|p| match p {
            ValidationPhase::Preflight => "preflight",
            ValidationPhase::LocalValidation => "local_validation",
            ValidationPhase::CrossRecordValidation => "cross_record_validation",
            ValidationPhase::FinalizingReport => "finalizing_report",
        });
        Ok(
            json!({"schemaVersion":1,"run":run,"sequence":self.sequence.to_string(),"phase":phase,
            "trace_ref":self.trace.map(|i|format!("trace_{i}")),"trace_records":self.trace_records.to_string(),"cross_records":self.cross_records.to_string()}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lintra::{InputSource, TraceValidationInput};
    fn sample() -> Progress {
        Progress::new(&ValidationScope::Trace(TraceValidationInput {
            trace_id: TraceId("T1".into()),
            source: InputSource::Path("SYNTHETIC".into()),
        }))
    }
    #[test]
    fn slow_consumer_keeps_one_latest_immutable_sample_and_exact_u64() {
        let mut p = sample();
        p.update(&ProgressEvent::RunStarted);
        p.update(&ProgressEvent::PhaseStarted(
            ValidationPhase::LocalValidation,
        ));
        p.update(&ProgressEvent::TraceStarted {
            trace_id: TraceId("T1".into()),
        });
        let before = p.dto(Value::Null).unwrap();
        for count in 1..=10_000 {
            p.update(&ProgressEvent::RecordsProcessed {
                trace_id: TraceId("T1".into()),
                count,
            });
        }
        assert_eq!(before["trace_records"], "0");
        assert_eq!(p.dto(Value::Null).unwrap()["trace_records"], "10000");
        p.update(&ProgressEvent::RecordsProcessed {
            trace_id: TraceId("T1".into()),
            count: u64::MAX,
        });
        assert_eq!(
            p.dto(Value::Null).unwrap()["trace_records"],
            u64::MAX.to_string()
        );
        p.update(&ProgressEvent::RunCompleted);
        let terminal = p.dto(Value::Null).unwrap();
        p.update(&ProgressEvent::PhaseStarted(ValidationPhase::Preflight));
        assert_eq!(p.dto(Value::Null).unwrap(), terminal);
        assert!(terminal.to_string().len() < 1024);
    }
    #[test]
    fn inconsistent_events_and_sequence_overflow_fail_only_progress() {
        for event in [
            ProgressEvent::CrossRecordsProcessed { count: 1 },
            ProgressEvent::TraceStarted {
                trace_id: TraceId("OTHER".into()),
            },
        ] {
            let mut p = sample();
            p.update(&event);
            assert_eq!(p.dto(Value::Null), Err(ErrorCode::TechnicalFailure));
        }
        let mut p = sample();
        p.sequence = u64::MAX;
        p.update(&ProgressEvent::PhaseStarted(
            ValidationPhase::LocalValidation,
        ));
        assert_eq!(p.dto(Value::Null), Err(ErrorCode::TechnicalFailure));
        let mut p = sample();
        p.update(&ProgressEvent::PhaseStarted(
            ValidationPhase::FinalizingReport,
        ));
        p.update(&ProgressEvent::PhaseStarted(
            ValidationPhase::LocalValidation,
        ));
        assert_eq!(p.dto(Value::Null), Err(ErrorCode::TechnicalFailure));
    }
}
