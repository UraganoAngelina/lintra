//! Notifications describe actual execution; observer implementations own throttling.
use crate::TraceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationPhase {
    Preflight,
    LocalValidation,
    CrossRecordValidation,
    FinalizingReport,
}
/// Counts are cumulative processed records for the named trace. No obligatory
/// event per record or implicit claim that any phase/trace has run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressEvent {
    RunStarted,
    PhaseStarted(ValidationPhase),
    TraceStarted { trace_id: TraceId },
    RecordsProcessed { trace_id: TraceId, count: u64 },
    TraceCompleted { trace_id: TraceId, count: u64 },
    CrossRecordsProcessed { count: u64 },
    RunCompleted,
}
/// Synchronous observer contract. Thread affinity, batching and UI scheduling
/// belong to the adapter; the engine does not require an async/GUI runtime.
pub trait ProgressObserver {
    fn on_event(&mut self, event: ProgressEvent);
}
/// Closures can collect, forward or throttle events without an extra wrapper
/// type. FnMut permits observer state while preserving sequential delivery.
impl<F: FnMut(ProgressEvent)> ProgressObserver for F {
    fn on_event(&mut self, event: ProgressEvent) {
        self(event);
    }
}
