//! Injected wall clock (DET §166); elapsed duration still uses monotonic Instant.
use std::time::SystemTime;

/// One timestamp per run, provided independently of validation or persistence.
/// Send/Sync permits sharing a service across callers without global clock state.
pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;
}
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}
