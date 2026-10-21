use lintra::{CancellationToken, ProgressEvent, ProgressObserver, TraceId, ValidationPhase};
use std::{sync::mpsc, thread};

/// Clones observe the same monotonic flag, independent defaults do not.
#[test]
fn cancellation_is_shared_idempotent_and_independent_of_other_tokens() {
    let token = CancellationToken::default();
    let cloned = token.clone();
    let other = CancellationToken::default();
    assert!(!token.is_cancelled() && !cloned.is_cancelled());
    cloned.cancel();
    token.cancel();
    assert!(token.is_cancelled() && cloned.is_cancelled());
    assert!(!other.is_cancelled());
}
/// Channel controls scheduling; join proves the worker completed its cancel.
/// No sleep, busy waiting, timing assumption or background thread is retained.
#[test]
fn cloned_token_can_be_cancelled_from_another_thread() {
    let token = CancellationToken::default();
    let worker_token = token.clone();
    let (tx, rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        rx.recv().unwrap();
        worker_token.cancel();
    });
    assert!(!token.is_cancelled());
    tx.send(()).unwrap();
    worker.join().unwrap();
    assert!(token.is_cancelled());
}
/// The real trait forwards every event unchanged in order; an observer may
/// batch a million-record notification without fabricating per-record events.
#[test]
fn observer_receives_owned_events_and_can_batch_counts() {
    let trace = TraceId("synthetic".into());
    let events = vec![
        ProgressEvent::RunStarted,
        ProgressEvent::PhaseStarted(ValidationPhase::Preflight),
        ProgressEvent::PhaseStarted(ValidationPhase::LocalValidation),
        ProgressEvent::TraceStarted {
            trace_id: trace.clone(),
        },
        ProgressEvent::RecordsProcessed {
            trace_id: trace.clone(),
            count: 1_000_000,
        },
        ProgressEvent::TraceCompleted {
            trace_id: trace,
            count: 1_000_000,
        },
        ProgressEvent::PhaseStarted(ValidationPhase::FinalizingReport),
        ProgressEvent::RunCompleted,
    ];
    let mut received = Vec::new();
    let mut callback = |event| received.push(event);
    let observer: &mut dyn ProgressObserver = &mut callback;
    for event in &events {
        observer.on_event(event.clone());
    }
    assert_eq!(received, events);
}
