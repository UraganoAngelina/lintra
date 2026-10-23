//! Streaming LocalOnly trace execution; no application dependency or Full claim.
use crate::{
    BomPolicy, CompiledFlowSchema, InputAuditSession, InputError, NewlinePolicy,
    RecordValidationError, RecordValidator, TraceId, ValidationSink, ValidationSummaryBuilder,
    input::reader::RecordReader, preflight_bom,
};
use std::{
    error::Error,
    fmt,
    io::{BufRead, BufReader},
    path::Path,
};

/// Controlled completion is separate from technical errors and global RunStatus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceCompletion {
    Completed { count: u64 },
    Cancelled { count: u64 },
}
/// Neutral engine notifications; application maps them to ProgressEvent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceEvent {
    Started { trace_id: TraceId },
    RecordsProcessed { trace_id: TraceId, count: u64 },
    Completed { trace_id: TraceId, count: u64 },
}
/// Borrow adapters rather than importing application cancellation/progress types.
/// Callbacks are synchronous; no thread, timer or observer state is owned here.
pub struct TraceControl<'a> {
    pub is_cancelled: &'a dyn Fn() -> bool,
    pub on_event: &'a mut dyn FnMut(TraceEvent),
}
#[derive(Debug)]
pub enum TraceValidationError {
    Input(InputError),
    Record(RecordValidationError),
    PreviousFailure,
}
impl fmt::Display for TraceValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Input(_) => "trace input failed",
            Self::Record(_) => "trace record processing failed",
            Self::PreviousFailure => "trace validator cannot resume after technical failure",
        })
    }
}
impl Error for TraceValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Input(e) => Some(e),
            Self::Record(e) => Some(e),
            Self::PreviousFailure => None,
        }
    }
}

/// One compiled trace, one reusable record buffer per invocation. Errors are
/// streamed; RecordValidator alone owns processed/valid/invalid record counts.
pub struct TraceValidator<'schema> {
    record: TraceRecordExecution<'schema>,
    trace_id: TraceId,
    newline: NewlinePolicy,
    failed: bool,
}
enum TraceRecordExecution<'a> {
    Local(RecordValidator<'a>),
    Reference {
        validator: crate::ReferenceRecordValidator<'a, 'a>,
        observer: &'a mut dyn crate::ReferenceEvaluationObserver,
    },
}
impl<'schema> TraceValidator<'schema> {
    pub fn new(
        flow: &'schema CompiledFlowSchema,
        trace: &TraceId,
        newline: NewlinePolicy,
    ) -> Result<Self, TraceValidationError> {
        Ok(Self {
            record: TraceRecordExecution::Local(
                RecordValidator::new(flow, trace).map_err(TraceValidationError::Record)?,
            ),
            trace_id: trace.clone(),
            newline,
            failed: false,
        })
    }
    /// Same framing/audit/cancellation path; evidence is immutable and prepared
    /// before opening the input. Observer aggregates once per compiled control.
    pub fn with_reference_data(
        flow: &'schema CompiledFlowSchema,
        trace: &TraceId,
        newline: NewlinePolicy,
        prepared: &'schema crate::PreparedReferenceEvidence,
        observer: &'schema mut dyn crate::ReferenceEvaluationObserver,
    ) -> Result<Self, TraceValidationError> {
        Ok(Self {
            record: TraceRecordExecution::Reference {
                validator: crate::ReferenceRecordValidator::new(flow, trace, prepared)
                    .map_err(TraceValidationError::Record)?,
                observer,
            },
            trace_id: trace.clone(),
            newline,
            failed: false,
        })
    }
    /// Caller may supply an audited reader. BOM Reject is the existing input
    /// baseline; hashing, if enabled, stays below this preflight and framing.
    pub fn validate_reader<R: BufRead>(
        &mut self,
        reader: R,
        sink: &mut dyn ValidationSink,
        summary: &mut ValidationSummaryBuilder,
        control: &mut TraceControl<'_>,
    ) -> Result<TraceCompletion, TraceValidationError> {
        if self.failed {
            return Err(TraceValidationError::PreviousFailure);
        }
        let result = self.run(reader, sink, summary, control);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    /// Open once through the authoritative audit session, then use exactly the
    /// same streaming path. Opening registers size/name; cancellation never
    /// reads extra source bytes just to complete a digest.
    pub fn validate_path(
        &mut self,
        path: &Path,
        sink: &mut dyn ValidationSink,
        summary: &mut ValidationSummaryBuilder,
        control: &mut TraceControl<'_>,
        audits: &InputAuditSession,
    ) -> Result<TraceCompletion, TraceValidationError> {
        if self.failed {
            return Err(TraceValidationError::PreviousFailure);
        }
        let source = match audits.open(self.trace_id.clone(), path) {
            Ok(source) => source,
            Err(error) => {
                self.failed = true;
                return Err(TraceValidationError::Input(InputError::Audit(error)));
            }
        };
        self.validate_reader(BufReader::new(source), sink, summary, control)
    }
    /// Checkpoints precede every source preflight/record read. A record already
    /// acquired is processed atomically with respect to cooperative cancellation.
    fn run<R: BufRead>(
        &mut self,
        reader: R,
        sink: &mut dyn ValidationSink,
        summary: &mut ValidationSummaryBuilder,
        control: &mut TraceControl<'_>,
    ) -> Result<TraceCompletion, TraceValidationError> {
        (control.on_event)(TraceEvent::Started {
            trace_id: self.trace_id.clone(),
        });
        if (control.is_cancelled)() {
            summary.cancel();
            return Ok(TraceCompletion::Cancelled { count: 0 });
        }
        let source =
            preflight_bom(reader, BomPolicy::Reject).map_err(TraceValidationError::Input)?;
        let mut reader = RecordReader::new(BufReader::new(source), self.newline);
        let mut buffer = Vec::new();
        let mut count = 0;
        let mut notified = 0;
        loop {
            if (control.is_cancelled)() {
                summary.cancel();
                self.notify_count(control, count, &mut notified);
                return Ok(TraceCompletion::Cancelled { count });
            }
            let Some(record) = reader
                .read_next(&mut buffer)
                .map_err(TraceValidationError::Input)?
            else {
                self.notify_count(control, count, &mut notified);
                (control.on_event)(TraceEvent::Completed {
                    trace_id: self.trace_id.clone(),
                    count,
                });
                return Ok(TraceCompletion::Completed { count });
            };
            match &mut self.record {
                TraceRecordExecution::Local(validator) => {
                    validator.validate_local(record, sink, summary).map(|_| ())
                }
                TraceRecordExecution::Reference {
                    validator,
                    observer,
                } => validator
                    .validate_local(record, sink, summary, *observer)
                    .map(|_| ()),
            }
            .map_err(TraceValidationError::Record)?;
            // RecordReader already checks line-counter overflow; no unchecked
            // second processed counter or summary increment is needed here.
            count = record.line_number;
            if count.is_multiple_of(1024) {
                self.notify_count(control, count, &mut notified);
            }
        }
    }
    /// Batch at 1024 and at a terminal checkpoint. Never send a fake count for
    /// a record whose validation failed technically, or duplicate final counts.
    fn notify_count(&self, control: &mut TraceControl<'_>, count: u64, notified: &mut u64) {
        if count != *notified {
            (control.on_event)(TraceEvent::RecordsProcessed {
                trace_id: self.trace_id.clone(),
                count,
            });
            *notified = count;
        }
    }
}
/// Lower-level testable entry point, sharing implementation with validate_path.
pub fn validate_reader<R: BufRead>(
    reader: R,
    flow: &CompiledFlowSchema,
    trace: &TraceId,
    newline: NewlinePolicy,
    sink: &mut dyn ValidationSink,
    summary: &mut ValidationSummaryBuilder,
    control: &mut TraceControl<'_>,
) -> Result<TraceCompletion, TraceValidationError> {
    TraceValidator::new(flow, trace, newline)?.validate_reader(reader, sink, summary, control)
}
