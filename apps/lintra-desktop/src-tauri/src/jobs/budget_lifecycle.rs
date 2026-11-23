//! D09 soft-budget gate through the native job manager (T-MIG-P5-05).
//!
//! Every test drives the production `Jobs` worker with the same IPC requests
//! the renderer uses: the warning, the one-time confirmation handle, Annulla,
//! Procedi comunque, cancel while warned, hard failures and concurrency.
use super::{tests::fixture, *};
use std::time::{Duration, Instant};

const INSTANCE: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const TIMEOUT: Duration = Duration::from_secs(30);

fn budget(input: u64) -> ResourceBudget {
    ResourceBudget::parse(
        format!(r#"{{"version":1,"measurement_source":"misura sintetica","input_bytes":{input},"reference_bytes":null}}"#)
            .as_bytes(),
    )
    .unwrap()
}
fn preflight(input: u64) -> Option<Preflight> {
    Some(Preflight {
        budget: budget(input),
        bundle: Arc::new(AnalysisBundle::builtin().unwrap()),
    })
}
fn payload(bytes: Result<Vec<u8>, ErrorCode>) -> Value {
    serde_json::from_slice::<Value>(&bytes.unwrap()).unwrap()["payload"].clone()
}
fn run(jobs: &Jobs) -> RunRef {
    jobs.current.as_ref().unwrap().run.clone()
}
fn gate(jobs: &mut Jobs) -> Value {
    let run = run(jobs);
    payload(jobs.dispatch(JobRequest::Budget { run }, "budget"))
}
fn state(jobs: &mut Jobs) -> Value {
    payload(jobs.dispatch(JobRequest::State { run: None }, "state"))
}
/// Poll until `done` holds for the budget DTO (the worker publishes it).
fn wait_gate(jobs: &mut Jobs, done: impl Fn(&Value) -> bool) -> Value {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let value = gate(jobs);
        if done(&value) {
            return value;
        }
        assert!(Instant::now() < deadline, "gate stuck at {value}");
        thread::sleep(Duration::from_millis(5));
    }
}
fn terminal(jobs: &mut Jobs) -> Value {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let value = state(jobs);
        if value["state"] == "terminal" {
            return value;
        }
        assert!(Instant::now() < deadline, "worker did not terminate");
        thread::sleep(Duration::from_millis(5));
    }
}
fn start(jobs: &mut Jobs, preflight: Option<Preflight>) -> tempfile::TempDir {
    let (directory, intent, refs) = fixture();
    jobs.start_budgeted(INSTANCE, "start", "plan_1".into(), intent, refs, preflight)
        .unwrap();
    directory
}
fn warned(jobs: &mut Jobs) -> (Value, String) {
    let value = wait_gate(jobs, |v| v["state"] == "confirmation_required");
    let handle = value["confirmation_ref"].as_str().unwrap().to_owned();
    (value, handle)
}
fn decide(jobs: &mut Jobs, handle: &str, proceed: bool) -> Result<Vec<u8>, ErrorCode> {
    let run = run(jobs);
    jobs.dispatch(
        JobRequest::Decide {
            run,
            handle: handle.into(),
            proceed,
        },
        "decide",
    )
}

#[test]
fn without_configuration_the_run_starts_unchanged_and_says_so() {
    let mut jobs = Jobs::default();
    let _directory = start(&mut jobs, None);
    let end = terminal(&mut jobs);
    assert_eq!(end["outcome"]["kind"], "validation");
    assert_eq!(end["outcome"]["report_available"], true);
    let value = gate(&mut jobs);
    assert_eq!(value["state"], "unconfigured");
    assert_eq!(value["audit"], Value::Null);
    assert_eq!(value["confirmation_ref"], Value::Null);
    // No decision is pending: a decision is refused, never a bypass.
    assert_eq!(decide(&mut jobs, "h", true), Err(ErrorCode::StaleContext));
}

#[test]
fn within_budget_proceeds_without_a_question_and_reports_real_counters() {
    let mut jobs = Jobs::default();
    let _directory = start(&mut jobs, preflight(1_000));
    let end = terminal(&mut jobs);
    assert_eq!(end["outcome"]["report_available"], true);
    let value = gate(&mut jobs);
    assert_eq!(value["state"], "within_budget");
    // fixture input "001A  \nX\n" is 9 bytes; embedded schema needs no reference.
    assert_eq!(value["audit"]["input_bytes"], "9");
    assert_eq!(value["audit"]["input_budget"], "1000");
    assert_eq!(value["audit"]["input_budget_exceeded"], false);
    assert_eq!(value["confirmation_ref"], Value::Null);
}

#[test]
fn over_budget_waits_for_the_operator_then_proceed_runs_the_whole_input_once() {
    let mut jobs = Jobs::default();
    let _directory = start(&mut jobs, preflight(1));
    let (value, handle) = warned(&mut jobs);
    assert_eq!(value["audit"]["input_bytes"], "9");
    assert_eq!(value["audit"]["input_budget"], "1");
    assert_eq!(value["audit"]["input_budget_exceeded"], true);
    assert_eq!(value["audit"]["reference_budget"], Value::Null);
    assert_eq!(handle.len(), 32);
    // The run waits: still preparing, no report and no progress beyond preflight.
    thread::sleep(Duration::from_millis(100));
    let waiting = state(&mut jobs);
    assert_eq!(waiting["state"], "preparing");
    assert_eq!(waiting["outcome"], Value::Null);
    // A forged or another run's handle is stale; nothing changes.
    assert_eq!(
        decide(&mut jobs, "0123456789abcdef0123456789abcdef", true),
        Err(ErrorCode::StaleContext)
    );
    assert_eq!(gate(&mut jobs)["state"], "confirmation_required");
    let answer = payload(decide(&mut jobs, &handle, true));
    assert_eq!(answer["disposition"], "proceeding");
    // One-time: the same handle cannot decide again (duplicate click).
    assert_eq!(
        decide(&mut jobs, &handle, true),
        Err(ErrorCode::StaleContext)
    );
    assert_eq!(
        decide(&mut jobs, &handle, false),
        Err(ErrorCode::StaleContext)
    );
    let end = terminal(&mut jobs);
    assert_eq!(end["cancel_requested"], false);
    assert_eq!(end["outcome"]["kind"], "validation");
    assert_eq!(end["outcome"]["status"], "completed_invalid");
    assert_eq!(end["outcome"]["completeness"], "complete");
    assert_eq!(end["outcome"]["truncated"], false);
    let after = gate(&mut jobs);
    assert_eq!(after["state"], "proceeded");
    assert_eq!(after["confirmation_ref"], Value::Null);
    // Proceed anyway never truncates: the published report has every record.
    let (_, report) = jobs.published().unwrap();
    assert_eq!(report.summary().records_processed, 2);
}

#[test]
fn annulla_cancels_the_pending_run_before_any_output() {
    let mut jobs = Jobs::default();
    let _directory = start(&mut jobs, preflight(1));
    let (_, handle) = warned(&mut jobs);
    assert_eq!(
        payload(decide(&mut jobs, &handle, false))["disposition"],
        "cancelled"
    );
    let end = terminal(&mut jobs);
    assert_eq!(end["cancel_requested"], true);
    assert_eq!(
        end["outcome"],
        json!({"schemaVersion":1,"kind":"not_started","reason":"cancelled"})
    );
    assert_eq!(gate(&mut jobs)["state"], "declined");
    assert!(jobs.published().is_none(), "no report after Annulla");
    assert_eq!(
        decide(&mut jobs, &handle, true),
        Err(ErrorCode::StaleContext)
    );
}

#[test]
fn cancel_while_warned_declines_and_a_later_decision_is_stale() {
    let mut jobs = Jobs::default();
    let _directory = start(&mut jobs, preflight(1));
    let (_, handle) = warned(&mut jobs);
    let run = run(&jobs);
    let cancel = payload(jobs.dispatch(JobRequest::Cancel { run }, "cancel"));
    assert_eq!(cancel["disposition"], "requested");
    assert_eq!(
        decide(&mut jobs, &handle, true),
        Err(ErrorCode::StaleContext)
    );
    let end = terminal(&mut jobs);
    assert_eq!(end["outcome"]["kind"], "not_started");
    assert_eq!(gate(&mut jobs)["state"], "declined");
    assert!(jobs.published().is_none());
}

#[test]
fn a_second_start_while_warned_is_busy_and_does_not_replace_the_pending_run() {
    let mut jobs = Jobs::default();
    let _directory = start(&mut jobs, preflight(1));
    let (_, handle) = warned(&mut jobs);
    let (_other, intent, refs) = fixture();
    assert_eq!(
        jobs.start_budgeted(
            INSTANCE,
            "start_2",
            "plan_1".into(),
            intent,
            refs,
            preflight(1)
        ),
        Err(ErrorCode::Busy)
    );
    assert_eq!(
        payload(decide(&mut jobs, &handle, true))["disposition"],
        "proceeding"
    );
    terminal(&mut jobs);
}

#[test]
fn hard_failures_are_technical_and_no_decision_can_bypass_them() {
    let mut jobs = Jobs::default();
    let directory = start(&mut jobs, None);
    terminal(&mut jobs);
    // Acknowledge so a new run may start, then make the input unreadable as
    // a regular file: the preflight fails before any warning or output.
    let run = run(&jobs);
    jobs.dispatch(JobRequest::Acknowledge { run }, "ack")
        .unwrap();
    let (_inner, intent, refs) = fixture();
    let ValidationScope::Trace(input) = &intent.request.scope else {
        unreachable!()
    };
    let InputSource::Path(path) = &input.source;
    std::fs::remove_file(path).unwrap();
    std::fs::create_dir(path).unwrap();
    jobs.start_budgeted(
        INSTANCE,
        "start_2",
        "plan_1".into(),
        intent,
        refs,
        preflight(1),
    )
    .unwrap();
    let end = terminal(&mut jobs);
    assert_eq!(end["outcome"]["kind"], "technical_failure");
    let value = gate(&mut jobs);
    assert_eq!(value["state"], "not_completed");
    assert_eq!(value["confirmation_ref"], Value::Null);
    drop(directory);
}

#[test]
fn closing_the_app_while_warned_joins_the_worker() {
    let mut jobs = Jobs::default();
    let _directory = start(&mut jobs, preflight(1));
    warned(&mut jobs);
    jobs.cancel_on_close();
    // Drop joins the worker; it must leave the wait on the token.
    let deadline = Instant::now() + TIMEOUT;
    drop(jobs);
    assert!(Instant::now() < deadline);
}

#[test]
fn ipc_payloads_are_exact() {
    let run = json!({"schemaVersion":1,"instance_id":INSTANCE,"run_id":"1"});
    let decode =
        |op: &str, payload: Value| JobRequest::decode(op, payload.as_object().unwrap()).map(|_| ());
    assert_eq!(
        decode("resource_budget_state", json!({"schemaVersion":1})),
        Ok(())
    );
    assert_eq!(
        decode("resource_budget_state", json!({"schemaVersion":1,"x":1})),
        Err(ErrorCode::InvalidMessage)
    );
    assert_eq!(
        decode("validation_budget", json!({"schemaVersion":1,"run":run})),
        Ok(())
    );
    assert_eq!(
        decode("validation_budget", json!({"schemaVersion":1})),
        Err(ErrorCode::InvalidMessage)
    );
    let ok = json!({"schemaVersion":1,"run":run,"confirmation_ref":"abc","decision":"proceed"});
    assert_eq!(decode("validation_budget_decide", ok.clone()), Ok(()));
    for (key, value) in [
        ("decision", json!("yes")),
        ("decision", json!(true)),
        ("confirmation_ref", json!("")),
        ("confirmation_ref", json!("a/b")),
        ("confirmation_ref", json!(1)),
    ] {
        let mut bad = ok.clone();
        bad[key] = value;
        assert_eq!(
            decode("validation_budget_decide", bad),
            Err(ErrorCode::InvalidMessage),
            "{key}"
        );
    }
    let mut extra = ok.clone();
    extra["proceed_anyway"] = json!(true);
    assert_eq!(
        decode("validation_budget_decide", extra),
        Err(ErrorCode::InvalidMessage)
    );
    assert!(
        decode(
            "validation_budget_decide",
            json!({"schemaVersion":2,"run":run,"confirmation_ref":"a","decision":"cancel"})
        )
        .is_err()
    );
}

/// P505-01: a configured run is observable before its worker is scheduled;
/// the very first read must already say "checking" (or later), never
/// "unconfigured", which a consumer treats as final. Repeated to cover the
/// interleavings where the worker has not run yet.
#[test]
fn a_configured_run_is_never_reported_unconfigured_before_its_worker_runs() {
    for _ in 0..50 {
        let mut jobs = Jobs::default();
        let _directory = start(&mut jobs, preflight(1));
        let first = gate(&mut jobs)["state"].clone();
        assert!(
            first == "checking" || first == "confirmation_required",
            "first read of a configured gate: {first}"
        );
        let (_, handle) = warned(&mut jobs);
        decide(&mut jobs, &handle, false).unwrap();
        terminal(&mut jobs);
    }
    // Without configuration the first read is unconfigured and stays so.
    let mut jobs = Jobs::default();
    let _directory = start(&mut jobs, None);
    assert_eq!(gate(&mut jobs)["state"], "unconfigured");
    terminal(&mut jobs);
    assert_eq!(gate(&mut jobs)["state"], "unconfigured");
}
