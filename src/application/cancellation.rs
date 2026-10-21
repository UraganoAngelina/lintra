//! Cooperative cancellation: one shared flag, no runtime or worker ownership.
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Default allocates a fresh false flag; clones share only that flag. Cancelling
/// one token never changes an independently created token. There is no reset.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}
impl CancellationToken {
    /// Idempotent state change. Relaxed is sufficient for this standalone flag:
    /// it does not publish other memory or promise synchronization of payloads.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
    /// Call at safe checkpoints between records/traces, never interrupt a
    /// critical structure halfway through construction. No polling thread here.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}
