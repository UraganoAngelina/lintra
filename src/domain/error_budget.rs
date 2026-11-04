//! Crate-private operational control, never a serialized status or cancellation.
use std::{
    num::NonZeroU64,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Shared only between the checked summary counter and execution checkpoints.
/// Atomic storage preserves the builder's Send/Sync capabilities; the actual
/// counters still have one mutable owner, not competing error producers.
#[derive(Clone)]
pub(crate) struct ErrorBudget(Arc<State>);
struct State {
    maximum: NonZeroU64,
    reached: AtomicBool,
}
impl ErrorBudget {
    pub(crate) fn new(maximum: NonZeroU64) -> Self {
        Self(Arc::new(State {
            maximum,
            reached: AtomicBool::new(false),
        }))
    }
    /// Called only after sink acknowledgement and successful checked counters.
    pub(crate) fn observe(&self, emitted: u64) {
        if emitted >= self.0.maximum.get() {
            self.0.reached.store(true, Ordering::Release);
        }
    }
    pub(crate) fn reached(&self) -> bool {
        self.0.reached.load(Ordering::Acquire)
    }
}

/// Internal completion of the current execution, distinct from RunStatus and
/// from the legacy public engine completions, which remain unlimited APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExecutionStop {
    Completed,
    Cancelled,
    ErrorLimit,
}
