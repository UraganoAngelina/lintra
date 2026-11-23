//! Integration tests of the Full lifecycle through the native job manager
//! (T-MIG-P4-12): progress, cancel and cleanup of a real two-trace bundle.
//!
//! Every test drives the production `Jobs` worker thread through the same IPC
//! requests the renderer uses. A test-only probe, called by the worker after
//! each progress event has been recorded, can hold the worker at an exact
//! point (a trace of PASS1, PASS2, finalization), so cancellation and state
//! are checked deterministically rather than by timing. Run spools are
//! created under a private root to prove that no directory survives a run
//! that did not publish, a superseded report or the application closing.
use super::{tests::full_fixture, *};
use std::{
    collections::BTreeSet,
    fs,
    path::Path,
    sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender},
    time::{Duration, Instant},
};

const INSTANCE: &str = "dddddddddddddddddddddddddddddddd";
const TIMEOUT: Duration = Duration::from_secs(60);
/// Contract of the progress DTO (PROGRESS.md): exactly these keys.
const PROGRESS_KEYS: [&str; 7] = [
    "schemaVersion",
    "run",
    "sequence",
    "phase",
    "trace_ref",
    "trace_records",
    "cross_records",
];

/// `count` records per trace, all joined S→T with equal DATA: a valid bundle.
/// DATA carries the marker `Q7`, which must never appear in an IPC frame.
fn bundle(count: usize) -> (Vec<u8>, Vec<u8>) {
    const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    assert!(count <= DIGITS.len() * DIGITS.len());
    let mut source = Vec::new();
    let mut target = Vec::new();
    for i in 0..count {
        let key = [DIGITS[i / DIGITS.len()], DIGITS[i % DIGITS.len()]];
        for file in [&mut source, &mut target] {
            file.extend_from_slice(&key);
            file.extend_from_slice(&key);
            file.extend_from_slice(b"Q7\n");
        }
    }
    (source, target)
}

/// Worker rendezvous: every event is reported; events matching `hold` keep
/// the worker waiting until the test releases it. Dropping the release side
/// unblocks the worker for good (it then runs to its end).
struct Gate {
    events: Receiver<ProgressEvent>,
    release: Option<SyncSender<()>>,
}
impl Gate {
    fn install(
        jobs: &mut Jobs,
        hold: impl Fn(&ProgressEvent) -> bool + Send + Sync + 'static,
    ) -> Self {
        let (events, observed) = mpsc::channel();
        let (release, released) = mpsc::sync_channel::<()>(0);
        let (events, released) = (Mutex::new(events), Mutex::new(released));
        jobs.probe = Some(Arc::new(move |event: &ProgressEvent| {
            if let Ok(events) = events.lock() {
                let _ = events.send(event.clone());
            }
            if hold(event)
                && let Ok(released) = released.lock()
            {
                // A disconnected release side means: run freely to the end.
                let _ = released.recv();
            }
        }));
        Self {
            events: observed,
            release: Some(release),
        }
    }
    /// Next reported event; the worker is held if it matches the predicate.
    fn next(&self) -> ProgressEvent {
        self.events
            .recv_timeout(TIMEOUT)
            .expect("worker reported no further progress event")
    }
    fn until(&self, wanted: impl Fn(&ProgressEvent) -> bool) -> ProgressEvent {
        loop {
            let event = self.next();
            if wanted(&event) {
                return event;
            }
        }
    }
    fn release(&self) {
        self.release
            .as_ref()
            .unwrap()
            .send(())
            .expect("held worker is waiting");
    }
    fn open(&mut self) {
        self.release = None;
    }
}

fn spool() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}
fn spools(root: &Path) -> BTreeSet<String> {
    fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect()
}
/// Payload of one IPC answer, checking the frame stays a small control DTO.
fn frame(bytes: Vec<u8>, frames: &mut Vec<String>) -> Value {
    assert!(
        bytes.len() <= 1024,
        "control frame of {} bytes",
        bytes.len()
    );
    let text = String::from_utf8(bytes).unwrap();
    assert!(!text.contains("Q7"), "input content crossed IPC: {text}");
    let value: Value = serde_json::from_str(&text).unwrap();
    frames.push(text);
    value["payload"].clone()
}
fn state(jobs: &mut Jobs, frames: &mut Vec<String>) -> Value {
    frame(
        jobs.dispatch(JobRequest::State { run: None }, "state")
            .unwrap(),
        frames,
    )
}
fn progress(jobs: &mut Jobs, run: &RunRef, frames: &mut Vec<String>) -> Value {
    let value = frame(
        jobs.dispatch(JobRequest::Progress { run: run.clone() }, "progress")
            .unwrap(),
        frames,
    );
    let keys: Vec<_> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let mut expected = PROGRESS_KEYS.to_vec();
    expected.sort_unstable();
    let mut actual = keys.clone();
    actual.sort_unstable();
    assert_eq!(actual, expected, "{value}");
    value
}
fn cancel(jobs: &mut Jobs, run: &RunRef, frames: &mut Vec<String>) -> Value {
    frame(
        jobs.dispatch(JobRequest::Cancel { run: run.clone() }, "cancel")
            .unwrap(),
        frames,
    )["disposition"]
        .clone()
}
fn terminal(jobs: &mut Jobs, frames: &mut Vec<String>) -> Value {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let value = state(jobs, frames);
        if value["state"] == "terminal" {
            return value;
        }
        assert!(Instant::now() < deadline, "worker did not terminate");
        thread::sleep(Duration::from_millis(1));
    }
}
fn start(
    jobs: &mut Jobs,
    intent: Arc<PreparedIntent>,
    refs: Arc<dyn ReferenceDataPreparer>,
    id: &str,
) -> RunRef {
    let bytes = jobs
        .start(INSTANCE, id, "plan_1".into(), intent, refs)
        .unwrap();
    RunRef::decode(&serde_json::from_slice::<Value>(&bytes).unwrap()["payload"]).unwrap()
}
fn acknowledge(jobs: &mut Jobs, run: &RunRef) {
    jobs.dispatch(JobRequest::Acknowledge { run: run.clone() }, "ack")
        .unwrap();
}
/// The owned worker of the current job has been joined: no thread survives.
fn joined(jobs: &Jobs) -> bool {
    jobs.current
        .as_ref()
        .is_some_and(|job| job.thread.is_none())
}
fn u64_of(value: &Value) -> u64 {
    value.as_str().unwrap().parse().unwrap()
}

/// Progress of a real Full bundle, sampled by the renderer's own request at
/// every event: one latest sample, per-trace identity that restarts from zero,
/// PASS2 without a trace, a monotone sequence, and no terminal before the
/// worker is joined even once the service reported `RunCompleted`.
#[test]
fn full_progress_is_one_bounded_sample_per_trace_and_pass() {
    let (source, target) = bundle(2_000);
    let (_inputs, intent, refs) = full_fixture(&source, &target);
    let root = spool();
    let mut jobs = Jobs {
        spool_root: Some(root.path().to_owned()),
        ..Jobs::default()
    };
    let mut gate = Gate::install(&mut jobs, |_| true);
    let mut frames = Vec::new();
    let run = start(&mut jobs, intent, refs, "start_progress");
    let (mut sequence, mut trace_records, mut cross_records) = (0u64, 0u64, 0u64);
    let mut traces_seen = Vec::new();
    let mut events = 0usize;
    loop {
        let event = gate.next();
        events += 1;
        let sample = progress(&mut jobs, &run, &mut frames);
        let snapshot = state(&mut jobs, &mut frames);
        // The worker is held inside the service: never terminal, no outcome.
        assert_ne!(snapshot["state"], "terminal", "{event:?}");
        assert_eq!(snapshot["outcome"], Value::Null);
        let current = u64_of(&sample["sequence"]);
        assert!(current >= sequence, "sequence regressed at {event:?}");
        sequence = current;
        match &event {
            ProgressEvent::TraceStarted { trace_id } => {
                traces_seen.push(trace_id.0.clone());
                assert_eq!(
                    sample["trace_ref"],
                    format!("config_1_trace_{}", traces_seen.len() - 1)
                );
                assert_eq!(sample["trace_records"], "0");
                trace_records = 0;
            }
            ProgressEvent::RecordsProcessed { .. } | ProgressEvent::TraceCompleted { .. } => {
                assert_eq!(sample["phase"], "local_validation");
                let count = u64_of(&sample["trace_records"]);
                assert!(count >= trace_records && count <= 2_000);
                trace_records = count;
            }
            ProgressEvent::PhaseStarted(ValidationPhase::CrossRecordValidation) => {
                assert_eq!(sample["trace_ref"], Value::Null);
                assert_eq!(sample["trace_records"], "0");
            }
            ProgressEvent::CrossRecordsProcessed { .. } => {
                assert_eq!(sample["phase"], "cross_record_validation");
                assert_eq!(sample["trace_ref"], Value::Null);
                let count = u64_of(&sample["cross_records"]);
                assert!(count >= cross_records);
                cross_records = count;
            }
            ProgressEvent::RunCompleted => {
                // Service completion is not the adapter terminal (PROGRESS.md).
                assert_eq!(snapshot["state"], "finalizing");
                assert_eq!(cancel(&mut jobs, &run, &mut frames), "too_late");
                gate.release();
                break;
            }
            _ => {}
        }
        gate.release();
    }
    gate.open();
    assert_eq!(traces_seen, ["S", "T"]);
    assert!(events > 4, "only {events} progress events");
    let done = terminal(&mut jobs, &mut frames);
    assert_eq!(done["outcome"]["status"], "completed_valid", "{done}");
    assert_eq!(done["outcome"]["completeness"], "complete");
    assert_eq!(done["outcome"]["report_available"], true);
    assert!(joined(&jobs));
    // A terminal run keeps its last sample: no late event reopens it.
    let last = progress(&mut jobs, &run, &mut frames);
    assert_eq!(progress(&mut jobs, &run, &mut frames), last);
    assert_eq!(spools(root.path()).len(), 1, "exactly the published report");
    // Frames are driven by requests, not by records: 2 × 2 000 records, and
    // every frame stayed a small control DTO without input content.
    assert!(frames.iter().all(|f| f.len() <= 1024));
}

/// Cancellation requested through IPC while the worker is held in each part
/// of a Full bundle: the request is idempotent, the run stays non-terminal
/// until the worker is joined, the outcome is Cancelled/Partial without a
/// report, no spool survives, and a new run needs the acknowledgement.
#[test]
fn full_cancel_through_ipc_in_each_pass_publishes_nothing_and_leaves_nothing() {
    type Hold = fn(&ProgressEvent) -> bool;
    let holds: [(&str, Hold); 4] = [
        (
            "trace S",
            |e| matches!(e, ProgressEvent::TraceStarted { trace_id } if trace_id.0 == "S"),
        ),
        (
            "trace T",
            |e| matches!(e, ProgressEvent::TraceStarted { trace_id } if trace_id.0 == "T"),
        ),
        ("pass two", |e| {
            *e == ProgressEvent::PhaseStarted(ValidationPhase::CrossRecordValidation)
        }),
        ("pass two records", |e| {
            matches!(e, ProgressEvent::CrossRecordsProcessed { .. })
        }),
    ];
    let (source, target) = bundle(500);
    for (name, hold) in holds {
        let (_inputs, intent, refs) = full_fixture(&source, &target);
        let root = spool();
        let mut jobs = Jobs {
            spool_root: Some(root.path().to_owned()),
            ..Jobs::default()
        };
        let mut gate = Gate::install(&mut jobs, hold);
        let mut frames = Vec::new();
        let run = start(&mut jobs, intent.clone(), refs.clone(), "start_cancel");
        gate.until(hold);
        assert_eq!(spools(root.path()).len(), 1, "{name}: run spool exists");
        assert_eq!(cancel(&mut jobs, &run, &mut frames), "requested", "{name}");
        assert_eq!(cancel(&mut jobs, &run, &mut frames), "already_requested");
        let held = state(&mut jobs, &mut frames);
        assert_eq!(held["state"], "cancelling", "{name}");
        assert_eq!(held["cancel_requested"], true);
        assert_eq!(held["outcome"], Value::Null);
        gate.open();
        let done = terminal(&mut jobs, &mut frames);
        assert_eq!(done["outcome"]["kind"], "validation", "{name}: {done}");
        assert_eq!(done["outcome"]["status"], "cancelled", "{name}");
        assert_eq!(done["outcome"]["completeness"], "partial");
        assert_eq!(done["outcome"]["report_available"], false);
        assert_eq!(cancel(&mut jobs, &run, &mut frames), "already_terminal");
        assert!(joined(&jobs), "{name}: worker joined");
        assert!(jobs.last_report.is_none(), "{name}: no false report");
        assert!(spools(root.path()).is_empty(), "{name}: spool removed");
        // Terminal is sticky until acknowledged; only then a new run starts.
        assert_eq!(
            jobs.start(
                INSTANCE,
                "start_next",
                "plan_1".into(),
                intent.clone(),
                refs.clone()
            ),
            Err(ErrorCode::InvalidState)
        );
        acknowledge(&mut jobs, &run);
        jobs.probe = None;
        let next = start(&mut jobs, intent, refs, "start_next");
        let done = terminal(&mut jobs, &mut frames);
        assert_eq!(done["run"], next.dto());
        assert_eq!(done["outcome"]["status"], "completed_valid", "{name}");
        assert_eq!(spools(root.path()).len(), 1);
    }
}

/// Reports and spools are released when superseded, on application close
/// with a run in flight, and when a bundle input disappears mid-run (a
/// technical failure, never a completed outcome or a report).
#[test]
fn superseded_closed_and_failed_full_runs_release_every_spool() {
    let (source, target) = bundle(300);
    let (inputs, intent, refs) = full_fixture(&source, &target);
    let root = spool();
    let mut jobs = Jobs {
        spool_root: Some(root.path().to_owned()),
        ..Jobs::default()
    };
    let mut frames = Vec::new();
    let first = start(&mut jobs, intent.clone(), refs.clone(), "start_1");
    terminal(&mut jobs, &mut frames);
    let published = spools(root.path());
    assert_eq!(published.len(), 1);
    acknowledge(&mut jobs, &first);
    let second = start(&mut jobs, intent.clone(), refs.clone(), "start_2");
    terminal(&mut jobs, &mut frames);
    // The newer report replaces the old one, whose spool is deleted.
    let current = spools(root.path());
    assert_eq!(current.len(), 1);
    assert!(current.is_disjoint(&published));
    assert_eq!(jobs.last_report.as_ref().unwrap().0, second);
    let stale = crate::results::ResultsRequest::decode(
        "results_query",
        json!({"schemaVersion":1,"run":first.dto(),"filters":{"schemaVersion":1}})
            .as_object()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(jobs.results(stale, "query"), Err(ErrorCode::StaleContext));
    acknowledge(&mut jobs, &second);

    // An input removed before PASS1 opens it is a technical failure.
    let ValidationScope::Flow(flow) = &intent.request.scope else {
        unreachable!()
    };
    let InputSource::Path(target_path) = &flow.traces[1].source;
    let target_path = target_path.clone();
    let hold = |e: &ProgressEvent| matches!(e, ProgressEvent::TraceStarted { trace_id } if trace_id.0 == "S");
    let mut gate = Gate::install(&mut jobs, hold);
    let failed = start(&mut jobs, intent.clone(), refs.clone(), "start_3");
    gate.until(hold);
    fs::remove_file(&target_path).unwrap();
    gate.open();
    let done = terminal(&mut jobs, &mut frames);
    assert_eq!(done["outcome"]["kind"], "technical_failure", "{done}");
    assert!(joined(&jobs));
    // The previous report stays the consultable one, with its own identity.
    assert_eq!(jobs.last_report.as_ref().unwrap().0, second);
    assert_eq!(spools(root.path()), current, "failed run left no spool");
    acknowledge(&mut jobs, &failed);
    fs::write(&target_path, &target).unwrap();

    // Closing with a run in flight: cancel, join on drop, delete everything.
    let hold = |e: &ProgressEvent| matches!(e, ProgressEvent::TraceStarted { trace_id } if trace_id.0 == "T");
    let mut gate = Gate::install(&mut jobs, hold);
    start(&mut jobs, intent, refs, "start_4");
    gate.until(hold);
    assert_eq!(spools(root.path()).len(), 2);
    jobs.cancel_on_close();
    gate.open();
    drop(jobs);
    // The worker ended with the manager: its probe (and channel) are gone.
    loop {
        match gate.events.recv_timeout(TIMEOUT) {
            Ok(_) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => panic!("zombie worker after close"),
        }
    }
    assert!(spools(root.path()).is_empty(), "close released every spool");
    drop(inputs);
}

/// T-MIG-P5-06: an operator cancel of a newer run publishes no report and
/// never replaces the previous one, which keeps its own identity: the
/// previous run stays queryable, the cancelled one is stale for results.
#[test]
fn cancel_of_a_newer_run_keeps_the_previous_report_and_publishes_nothing() {
    let (source, target) = bundle(300);
    let (_inputs, intent, refs) = full_fixture(&source, &target);
    let root = spool();
    let mut jobs = Jobs {
        spool_root: Some(root.path().to_owned()),
        ..Jobs::default()
    };
    let mut frames = Vec::new();
    let first = start(&mut jobs, intent.clone(), refs.clone(), "start_1");
    let done = terminal(&mut jobs, &mut frames);
    assert_eq!(done["outcome"]["report_available"], true, "{done}");
    let published = spools(root.path());
    acknowledge(&mut jobs, &first);

    // Hold the newer run inside PASS1, then cancel it as the operator does.
    let hold = |e: &ProgressEvent| matches!(e, ProgressEvent::TraceStarted { trace_id } if trace_id.0 == "T");
    let mut gate = Gate::install(&mut jobs, hold);
    let second = start(&mut jobs, intent, refs, "start_2");
    gate.until(hold);
    let answer = jobs
        .dispatch(
            JobRequest::Cancel {
                run: second.clone(),
            },
            "cancel",
        )
        .unwrap();
    let answer: Value = serde_json::from_slice(&answer).unwrap();
    assert_eq!(answer["payload"]["disposition"], "requested");
    gate.open();
    let end = terminal(&mut jobs, &mut frames);
    assert_eq!(end["cancel_requested"], true);
    assert_eq!(end["outcome"]["status"], "cancelled", "{end}");
    assert_eq!(end["outcome"]["completeness"], "partial");
    assert_eq!(
        end["outcome"]["report_available"], false,
        "no new report after cancel"
    );
    assert!(joined(&jobs));
    // The previous report is still the consultable one, with its identity.
    assert_eq!(jobs.last_report.as_ref().unwrap().0, first);
    assert_eq!(
        spools(root.path()),
        published,
        "cancelled run left no spool"
    );
    let query = |run: &RunRef| {
        crate::results::ResultsRequest::decode(
            "results_query",
            json!({"schemaVersion":1,"run":run.dto(),"filters":{"schemaVersion":1}})
                .as_object()
                .unwrap(),
        )
        .unwrap()
    };
    assert!(
        jobs.results(query(&first), "q1").is_ok(),
        "previous report queryable"
    );
    assert_eq!(
        jobs.results(query(&second), "q2"),
        Err(ErrorCode::StaleContext)
    );
    // A late cancel after the terminal changes nothing.
    let late = jobs
        .dispatch(
            JobRequest::Cancel {
                run: second.clone(),
            },
            "cancel_2",
        )
        .unwrap();
    let late: Value = serde_json::from_slice(&late).unwrap();
    assert_eq!(late["payload"]["disposition"], "already_terminal");
    assert_eq!(jobs.last_report.as_ref().unwrap().0, first);
}
