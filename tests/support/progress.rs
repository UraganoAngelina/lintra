//! Real observer implementations for application integration tests.
use lintra::{ProgressEvent, ProgressObserver};

/// Discards notifications without affecting validation or cancellation.
pub struct NoopProgressObserver;
impl ProgressObserver for NoopProgressObserver {
    /// No state is needed when the caller deliberately does not observe progress.
    fn on_event(&mut self, _: ProgressEvent) {}
}

/// Owns notifications in callback order, allowing exact lifecycle assertions.
#[derive(Default)]
pub struct CollectingProgressObserver {
    pub events: Vec<ProgressEvent>,
}
impl ProgressObserver for CollectingProgressObserver {
    /// Append rather than sort/deduplicate so the actual service order survives.
    fn on_event(&mut self, event: ProgressEvent) {
        self.events.push(event);
    }
}
