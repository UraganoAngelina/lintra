//! Isolated Full prototype. Only disjoint trace state is sent between workers.
//! Bounded channels preserve the real sink callback and schema/line order;
//! projections move once, and the existing sequential pass two is untouched.
use super::*;
use std::{
    cell::Cell,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError, SyncSender},
    },
    thread,
    time::{Duration, Instant},
};
thread_local! {
    static WORKERS: Cell<usize> = const { Cell::new(0) };
    static TIMING: Cell<(f64, f64)> = const { Cell::new((0.0, 0.0)) };
    static CROSS_SECONDS: Cell<f64> = const { Cell::new(0.0) };
    static CAPTURE_CONTEXT: Cell<bool> = const { Cell::new(false) };
    static CONTEXT: std::cell::RefCell<Option<Value>> = const { std::cell::RefCell::new(None) };
}
pub(super) fn end_pass_one(seconds: f64) {
    let (_, aggregation) = TIMING.get();
    TIMING.set((seconds, aggregation));
}
pub(super) fn end_cross(seconds: f64) {
    CROSS_SECONDS.set(seconds);
}
/// Untimed conformance hook only; benchmark mode leaves it disabled.
pub(super) fn capture(state: &FlowState, schema: &CompiledFlowSchema) {
    if !CAPTURE_CONTEXT.get() {
        return;
    }
    let records = schema.traces().iter().map(|trace| {
        let rows=state.store.iter_trace(trace.id()).map(|record| {
            let values=record.values.iter().map(|(k,v)|(k.0.as_str(),v)).collect::<BTreeMap<_,_>>();
            let joins=record.join_keys.iter().map(|(k,v)|(format!("{k:?}"),format!("{v:?}"))).collect::<BTreeMap<_,_>>();
            json!({"trace":record.trace_id.0,"line":record.line_number,"identity":format!("{:?}",record.identity),"values":values,"joins":joins})
        }).collect::<Vec<_>>();
        json!({"trace":trace.id().0,"records":rows})
    }).collect::<Vec<_>>();
    CONTEXT.with_borrow_mut(|v| *v = Some(json!(records)));
}
pub(super) fn enabled(
    validator: &FlowValidator<'_>,
    ports: &FlowExecutionPorts<'_, '_>,
    historical: bool,
) -> bool {
    WORKERS.get() > 0
        && !historical
        && ports.summary.error_budget().is_none()
        && validator.schema.required_reference_data().is_empty()
        && !(ports.control.is_cancelled)()
}
struct Completed {
    state: FlowState,
    summary: ValidationSummaryBuilder,
    audits: InputAuditSession,
}
enum Message {
    Error(Box<ValidationError>),
    Event(TraceEvent),
    Done(Box<Result<Completed, FlowValidationError>>),
}
struct ChannelSink<'a>(&'a SyncSender<Message>);
impl ValidationSink for ChannelSink<'_> {
    fn on_error(&mut self, e: &ValidationError) -> Result<(), ReportError> {
        self.0
            .send(Message::Error(Box::new(e.clone())))
            .map_err(|_| ReportError::Write(std::io::Error::other("experiment receiver closed")))
    }
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        Err(ReportError::PreviousFailure)
    }
}
fn trace(
    validator: &FlowValidator<'_>,
    index: usize,
    path: &Path,
    tx: &SyncSender<Message>,
    stop: &AtomicBool,
) -> Result<Completed, FlowValidationError> {
    let schema = validator.schema;
    let trace = &schema.traces()[index];
    let mut state = FlowState::new(schema);
    let mut summary = ValidationSummaryBuilder::new(
        schema.key().clone(),
        ScopeSummary::Trace {
            trace_id: trace.id().clone(),
        },
    );
    let audits = InputAuditSession::new();
    let mut no_history = None;
    let mut collector = ProjectionCollector {
        flow: schema,
        trace_index: index,
        state: &mut state,
        keys: HashMap::new(),
        record_observer: &mut no_history,
    };
    let mut sink = ChannelSink(tx);
    let cancelled = || stop.load(Ordering::Relaxed);
    let mut event = |e| {
        let _ = tx.send(Message::Event(e));
    };
    let mut control = TraceControl {
        is_cancelled: &cancelled,
        on_event: &mut event,
    };
    let mut reader = TraceValidator::new(schema, trace.id(), validator.newline)
        .map_err(FlowValidationError::Trace)?;
    let result = reader
        .validate_full_path(
            path,
            TracePassOnePorts {
                sink: &mut sink,
                summary: &mut summary,
                control: &mut control,
                audits: &audits,
                collector: &mut collector,
            },
        )
        .map_err(FlowValidationError::Trace)?;
    drop(reader);
    if result.completion != ExecutionStop::Completed {
        return Err(FlowValidationError::InvalidPlan);
    }
    state.traces[index].coverage = if state.traces[index].unprojectable_records == 0 {
        TraceContextCoverage::Complete
    } else {
        TraceContextCoverage::Partial
    };
    Ok(Completed {
        state,
        summary,
        audits,
    })
}
pub(super) fn pass_one(
    validator: &FlowValidator<'_>,
    prepared: &[(usize, &Path)],
    ports: &mut FlowExecutionPorts<'_, '_>,
) -> Result<FlowState, FlowValidationError> {
    let started = Instant::now();
    let stop = AtomicBool::new(false);
    let workers = WORKERS.get().min(prepared.len()).max(1);
    let mut state = FlowState::new(validator.schema);
    let mut aggregation = 0.0;
    // One fixed-capacity FIFO per declared trace. Later traces backpressure
    // while the caller consumes earlier traces; never a global error Vec.
    let (senders, receivers): (Vec<_>, Vec<_>) =
        prepared.iter().map(|_| mpsc::sync_channel(64)).unzip();
    let result = thread::scope(|scope| {
        let mut handles = Vec::new();
        for worker in 0..workers {
            let senders = &senders;
            let stop = &stop;
            handles.push(scope.spawn(move || {
                for position in (worker..prepared.len()).step_by(workers) {
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    let (index, path) = prepared[position];
                    let output = trace(validator, index, path, &senders[position], stop);
                    if senders[position]
                        .send(Message::Done(Box::new(output)))
                        .is_err()
                    {
                        break;
                    }
                }
            }));
        }
        let mut result = Ok(());
        'traces: for (position, rx) in receivers.iter().enumerate() {
            loop {
                if (ports.control.is_cancelled)() {
                    // Deliberate research limitation: dynamic cancellation
                    // aborts technically. Never publish a false complete context.
                    result = Err(FlowValidationError::InvalidPlan);
                    break 'traces;
                }
                match rx.recv_timeout(Duration::from_millis(2)) {
                    Ok(Message::Error(error)) => {
                        if let Err(error) = ports.sink.on_error(&error) {
                            result = Err(FlowValidationError::Trace(TraceValidationError::Record(
                                RecordValidationError::Report(error),
                            )));
                            break 'traces;
                        }
                    }
                    Ok(Message::Event(event)) => (ports.control.on_event)(FlowEvent::Trace(event)),
                    Ok(Message::Done(output)) => {
                        let completed = match *output {
                            Ok(v) => v,
                            Err(e) => {
                                result = Err(e);
                                break 'traces;
                            }
                        };
                        let now = Instant::now();
                        let index = prepared[position].0;
                        let mut local = completed.state;
                        state.traces[index] = local.traces.remove(index);
                        state.record_keys[index] = local.record_keys.remove(index);
                        state.invalid.extend(local.invalid);
                        // Payload ownership moves; secondary indexes are rebuilt
                        // once centrally and their cost is inside this interval.
                        for projection in local.store.dec17_take() {
                            if state.store.insert(projection).is_err() {
                                result = Err(FlowValidationError::InvalidPlan);
                                break 'traces;
                            }
                        }
                        if let Err(e) = ports.summary.dec17_merge(completed.summary) {
                            result = Err(FlowValidationError::Summary(e));
                            break 'traces;
                        }
                        if ports.audits.dec17_adopt(completed.audits).is_err() {
                            result = Err(FlowValidationError::InvalidPlan);
                            break 'traces;
                        }
                        aggregation += now.elapsed().as_secs_f64();
                        break;
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        if handles.iter().all(|handle| handle.is_finished()) {
                            result = Err(FlowValidationError::InvalidPlan);
                            break 'traces;
                        }
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        result = Err(FlowValidationError::InvalidPlan);
                        break 'traces;
                    }
                }
            }
        }
        stop.store(true, Ordering::Relaxed);
        // Unblock every bounded send before joining, including on sink/I/O
        // failure or panic. No detached worker survives this foreground scope.
        drop(receivers);
        for handle in handles {
            if handle.join().is_err() {
                result = Err(FlowValidationError::InvalidPlan);
            }
        }
        result
    });
    TIMING.set((started.elapsed().as_secs_f64(), aggregation));
    result.map(|()| state)
}

#[path = "../../tests/support/full_schema.rs"]
mod test_schema;
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
    sync::Arc,
    time::UNIX_EPOCH,
};
struct FixedClock;
impl Clock for FixedClock {
    fn now(&self) -> std::time::SystemTime {
        UNIX_EPOCH
    }
}
struct Fixture {
    root: tempfile::TempDir,
    raw: RawFlowSchema,
    paths: Vec<PathBuf>,
}
impl Fixture {
    fn new(count: usize, skewed: bool, heavy: bool) -> Self {
        Self::with_projection(count, skewed, heavy, false)
    }
    fn with_projection(count: usize, skewed: bool, heavy: bool, projection_heavy: bool) -> Self {
        let root = tempfile::tempdir_in("target").unwrap();
        let private = root.path().join("private");
        crate::persistence::dec17_private_root(&private).unwrap();
        let mut raw = test_schema::raw();
        let mut fields = raw["traces"][0]["fields"].as_array().unwrap().clone();
        for (i, (start, width)) in [(1, 8), (9, 8), (17, 2)].into_iter().enumerate() {
            fields[i]["physical"]["start"] = json!(start);
            fields[i]["physical"]["length"] = json!(width);
        }
        for i in 0..8 {
            let mut field = fields[2].clone();
            field["id"] = json!(format!("PAY{i}"));
            field["physical"]["start"] = json!(19 + i * 16);
            field["physical"]["length"] = json!(16);
            field["constraints"] = json!([{"kind":"regex","pattern":r"\A[A-Z0-9]{16}\z","match":"required","target":"raw"}]);
            fields.push(field);
        }
        let mut traces = vec![];
        for pair in 0..2 {
            for kind in 0..2 {
                let mut trace = raw["traces"][kind].clone();
                trace["id"] = json!(format!("{}{pair}", if kind == 0 { "S" } else { "T" }));
                trace["fields"] = json!(fields);
                trace["record_length"] = json!(146);
                trace["identity"] = json!([]);
                if kind == 0 {
                    trace["joins"][0]["id"] = json!(format!("CROSS{pair}"));
                    trace["joins"][0]["target_trace"] = json!(format!("T{pair}"));
                    trace["rules"][0]["assertions"][0]["right"]["ref"]["join"] =
                        json!(format!("CROSS{pair}"));
                    trace["rules"][0]["assertions"][0]["right"]["ref"]["trace"] =
                        json!(format!("T{pair}"));
                    let mut join = trace["joins"][0].clone();
                    join["id"] = json!(format!("SELF{pair}"));
                    join["target_trace"] = trace["id"].clone();
                    join["target_fields"] = json!(["PK"]);
                    trace["joins"].as_array_mut().unwrap().push(join);
                    let mut rule = trace["rules"][0].clone();
                    rule["id"] = json!("SELF_COMPARE");
                    rule["assertions"][0]["right"]["ref"]["trace"] = trace["id"].clone();
                    rule["assertions"][0]["right"]["ref"]["join"] = json!(format!("SELF{pair}"));
                    trace["rules"].as_array_mut().unwrap().push(rule);
                    if projection_heavy {
                        for i in 0..8 {
                            let mut assertion = trace["rules"][0]["assertions"][0].clone();
                            assertion["left"]["ref"]["field"] = json!(format!("PAY{i}"));
                            assertion["right"]["ref"]["field"] = json!(format!("PAY{i}"));
                            trace["rules"][0]["assertions"]
                                .as_array_mut()
                                .unwrap()
                                .push(assertion);
                        }
                    }
                }
                traces.push(trace);
            }
        }
        raw["traces"] = json!(traces);
        let weights = if skewed { [80, 10, 5, 5] } else { [25; 4] };
        let mut paths = vec![];
        for (index, weight) in weights.into_iter().enumerate() {
            let path = private.join(format!("synthetic-{index}.txt"));
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let file = options.open(&path).unwrap();
            crate::persistence::dec17_private_file(&path, &file).unwrap();
            let mut writer = BufWriter::new(file);
            for line in 1..=count * weight / 100 {
                let bad = line % if heavy { 4 } else { 1000 } == 0;
                // Source keys normally reference the previous physical row, so
                // SELF really depends on another record, not the current one.
                // Every seventeenth has zero, target duplicates exercise many.
                let pk = if index % 2 == 1 && line % 11 == 0 {
                    line - 1
                } else {
                    line
                };
                let key = if index % 2 == 0 && line % 17 == 0 {
                    99_999_999
                } else if index % 2 == 0 && line > 1 {
                    line - 1
                } else {
                    pk
                };
                write!(writer, "{pk:08}{key:08}{}", if bad { "NO" } else { "OK" }).unwrap();
                for _ in 0..8 {
                    writer
                        .write_all(if bad && heavy {
                            b"!!!!!!!!!!!!!!!!"
                        } else {
                            b"ABCDEF0123456789"
                        })
                        .unwrap();
                }
                writer.write_all(b"\n").unwrap();
            }
            writer.flush().unwrap();
            paths.push(path);
        }
        Self {
            root,
            raw: serde_json::from_value(raw).unwrap(),
            paths,
        }
    }
    fn service(&self) -> ValidationService {
        let mut repository = InMemorySchemaRepository::new();
        repository.insert(self.raw.clone()).unwrap();
        ValidationService::with_clock(
            Arc::new(SchemaRegistry::new(
                Arc::new(repository),
                SchemaCompiler,
                RawResolvedEncoding::Utf8,
            )),
            Arc::new(FixedClock),
        )
    }
    fn request(&self) -> ValidationRequest {
        ValidationRequest {
            schema_key: SchemaKey {
                flow_id: FlowId(self.raw.flow_id.clone()),
                version: SchemaVersion(self.raw.schema_version.clone()),
            },
            scope: ValidationScope::Flow(FlowValidationInput {
                traces: self
                    .raw
                    .traces
                    .iter()
                    .zip(&self.paths)
                    .map(|(trace, path)| TraceValidationInput {
                        trace_id: TraceId(trace.id.clone()),
                        source: InputSource::Path(path.clone()),
                    })
                    .collect(),
            }),
            mode: ValidationMode::Full,
            options: ValidationOptions {
                record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                newline_policy: NewlinePolicy::AutoLfOrCrlf,
                max_errors: None,
            },
        }
    }
}
struct Capture {
    jsonl: JsonlSink<BufWriter<fs::File>>,
    summary: Vec<u8>,
}
impl ValidationSink for Capture {
    fn on_error(&mut self, error: &ValidationError) -> Result<(), ReportError> {
        self.jsonl.on_error(error)
    }
    fn on_summary(&mut self, summary: &ValidationSummary) -> Result<(), ReportError> {
        self.summary = write_run_summary(Vec::new(), summary)?;
        self.jsonl.on_summary(summary)
    }
}
fn compare(reference: &Value, actual: &Value, path: &str, differences: &mut Vec<Value>) {
    if reference == actual {
        return;
    }
    if path == "/elapsed" {
        differences.push(json!({"path":path,"reference":reference,"actual":actual,"reason":"documented monotonic duration; original summary bytes retained"}));
        return;
    }
    match (reference, actual) {
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
            for (k, v) in a {
                compare(v, &b[k], &format!("{path}/{k}"), differences);
            }
        }
        _ => panic!("Full deterministic difference at {path}: {reference} != {actual}"),
    }
}
#[cfg(target_os = "linux")]
fn io_counts() -> Value {
    let counters = fs::read_to_string("/proc/self/io")
        .unwrap()
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            Some((key.to_owned(), value.trim().parse::<u64>().unwrap()))
        })
        .collect::<BTreeMap<_, _>>();
    json!(counters)
}
#[cfg(not(target_os = "linux"))]
fn io_counts() -> Value {
    Value::Null
}
struct StopSampler<'a>(&'a AtomicBool);
impl Drop for StopSampler<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}
fn measured<T>(work: impl FnOnce() -> T) -> (T, Value) {
    let before = io_counts();
    let starting_rss = super::selective_parallelism::rss();
    let heap = super::selective_parallelism::heap_start();
    let stop = AtomicBool::new(false);
    let started = Instant::now();
    let (result, rss) = thread::scope(|scope| {
        let sampler = scope.spawn(|| {
            let mut peak = starting_rss;
            while !stop.load(Ordering::Relaxed) {
                peak = peak.max(super::selective_parallelism::rss());
                thread::sleep(Duration::from_millis(2));
            }
            peak.max(super::selective_parallelism::rss())
        });
        let guard = StopSampler(&stop);
        let result = work();
        drop(guard);
        (result, sampler.join().unwrap())
    });
    let total = started.elapsed().as_secs_f64();
    let heap = super::selective_parallelism::heap_end(heap);
    let after = io_counts();
    (
        result,
        json!({"total_seconds":total,"starting_rss_bytes":starting_rss,"peak_rss_sampled_bytes":rss,"additional_peak_requested_heap_bytes":heap,"rss_sampling_ms":2,"io_before":before,"io_after":after,"worker_spool_bytes":0}),
    )
}
fn run(fixture: &Fixture, workers: usize) -> (Value, tempfile::NamedTempFile) {
    let file = tempfile::NamedTempFile::new_in(fixture.root.path().join("private")).unwrap();
    crate::persistence::dec17_private_file(file.path(), file.as_file()).unwrap();
    let mut sink = Capture {
        jsonl: JsonlSink::new(BufWriter::new(file.reopen().unwrap())),
        summary: vec![],
    };
    WORKERS.set(workers);
    TIMING.set((0.0, 0.0));
    CROSS_SECONDS.set(0.0);
    let mut progress = sha2::Sha256::default();
    let ((outcome, summary_bytes), memory) = measured(|| {
        let outcome = fixture
            .service()
            .validate(
                fixture.request(),
                &mut sink,
                &mut |event| progress.update(format!("{event:?}")),
                &CancellationToken::default(),
            )
            .unwrap();
        sink.jsonl.finish().unwrap();
        (outcome, sink.summary)
    });
    let total = memory["total_seconds"].as_f64().unwrap();
    WORKERS.set(0);
    let (pass_one, aggregation) = TIMING.get();
    let mut reader = BufReader::new(file.reopen().unwrap());
    let mut hash = sha2::Sha256::default();
    let mut buffer = [0; 65536];
    use sha2::Digest;
    loop {
        let n = reader.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    let progress_hash = progress
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let jsonl_hash = hash
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    (
        json!({"workers":workers,"total_seconds":total,"pass_one_including_aggregation_seconds":pass_one,"aggregation_seconds":aggregation,"pass_two_seconds":CROSS_SECONDS.get(),"summary_original_utf8":String::from_utf8(summary_bytes.clone()).unwrap(),"summary":serde_json::from_slice::<Value>(&summary_bytes).unwrap(),"jsonl_sha256":jsonl_hash,"jsonl_bytes":file.as_file().metadata().unwrap().len(),"records_processed":outcome.summary.records_processed,"memory_and_io":memory,"progress_sha256":progress_hash}),
        file,
    )
}
#[test]
fn full_equivalence() {
    let fixture = Fixture::new(400, false, true);
    CAPTURE_CONTEXT.set(true);
    let (baseline, original) = run(&fixture, 0);
    let context = CONTEXT.with_borrow(|v| v.clone());
    assert!(baseline["summary"]["total_errors"].as_u64().unwrap() > 0);
    let bytes = fs::read(original.path()).unwrap();
    assert!(
        std::str::from_utf8(&bytes)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .any(|line| line["trace"] == "S0"
                && line["line"] == 5
                && line["message"].as_str().unwrap().contains("SELF_COMPARE")),
        "line5 must depend on line4's different DATA"
    );
    let controls = baseline["summary"]["full_validation"]["controls"]
        .as_array()
        .unwrap();
    assert!(
        controls
            .iter()
            .any(|v| v["satisfied"].as_u64().unwrap() > 0)
    );
    assert!(
        controls
            .iter()
            .any(|v| v["violations"].as_u64().unwrap() > 0)
    );
    assert!(controls.iter().any(|v| v["unknown"].as_u64().unwrap() > 0));
    for workers in [1, 2, 4] {
        let (actual, file) = run(&fixture, workers);
        assert_eq!(context, CONTEXT.with_borrow(|v| v.clone()));
        let mut differences = vec![];
        compare(
            &baseline["summary"],
            &actual["summary"],
            "",
            &mut differences,
        );
        assert_eq!(baseline["jsonl_sha256"], actual["jsonl_sha256"]);
        assert_eq!(
            fs::read(original.path()).unwrap(),
            fs::read(file.path()).unwrap()
        );
        file.close().unwrap();
    }
    CAPTURE_CONTEXT.set(false);
    original.close().unwrap();
    fixture.root.close().unwrap();
}

#[test]
fn limited_and_precancelled_runs_use_original_path() {
    let fixture = Fixture::new(400, false, true);
    for limit in [Some(1), Some(7), Some(100), None] {
        let mut results = vec![];
        for workers in [0, 4] {
            WORKERS.set(workers);
            TIMING.set((0.0, 0.0));
            let token = CancellationToken::default();
            if limit.is_none() {
                token.cancel();
            }
            let mut request = fixture.request();
            request.options.max_errors = limit;
            let mut sink = VecSink::default();
            let outcome = fixture
                .service()
                .validate(request, &mut sink, &mut |_| {}, &token)
                .unwrap();
            assert_eq!(TIMING.get(), (0.0, 0.0));
            results.push((
                serde_json::from_slice::<Value>(
                    &write_run_summary(Vec::new(), &outcome.summary).unwrap(),
                )
                .unwrap(),
                sink,
            ));
            WORKERS.set(0);
        }
        assert_eq!(results[0].1.errors(), results[1].1.errors());
        compare(&results[0].0, &results[1].0, "", &mut vec![]);
    }
    fixture.root.close().unwrap();
}
struct BrokenSink;
impl ValidationSink for BrokenSink {
    fn on_error(&mut self, _: &ValidationError) -> Result<(), ReportError> {
        Err(ReportError::Write(std::io::Error::other(
            "synthetic sink fault",
        )))
    }
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        panic!("fault cannot publish summary")
    }
}
#[test]
fn technical_faults_join_backpressured_workers_without_summary() {
    let fixture = Fixture::new(8000, false, true);
    WORKERS.set(4);
    let error = fixture
        .service()
        .validate(
            fixture.request(),
            &mut BrokenSink,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap_err();
    assert!(
        matches!(error, ValidationRunError::Report(ReportError::Write(_))),
        "{error:?}"
    );
    fs::remove_file(&fixture.paths[0]).unwrap();
    let mut sink = VecSink::default();
    let error = fixture
        .service()
        .validate(
            fixture.request(),
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap_err();
    assert!(matches!(error, ValidationRunError::Input(_)));
    assert!(sink.summaries().is_empty());
    WORKERS.set(0);
    fixture.root.close().unwrap();
}
#[test]
fn dynamic_cancel_records_failed_observable_contract_and_no_false_completion() {
    let fixture = Fixture::new(8000, false, true);
    let token = CancellationToken::default();
    let mut sink = VecSink::default();
    WORKERS.set(4);
    let result = fixture.service().validate(
        fixture.request(),
        &mut sink,
        &mut |event| {
            if matches!(event, ProgressEvent::TraceStarted { .. }) {
                token.cancel();
            }
        },
        &token,
    );
    // This is a failed adoption condition, explicitly recorded in the report:
    // the prototype aborts rather than publishing speculative worker counters.
    assert!(result.is_err());
    assert!(token.is_cancelled());
    assert!(sink.summaries().is_empty());
    WORKERS.set(0);
    fixture.root.close().unwrap();
}

fn files_equal(a: &tempfile::NamedTempFile, b: &tempfile::NamedTempFile) -> bool {
    let mut left = BufReader::new(a.reopen().unwrap());
    let mut right = BufReader::new(b.reopen().unwrap());
    let mut x = [0; 65536];
    let mut y = [0; 65536];
    loop {
        let mut n = 0;
        while n < x.len() {
            let k = left.read(&mut x[n..]).unwrap();
            if k == 0 {
                break;
            }
            n += k;
        }
        if right.read_exact(&mut y[..n]).is_err() || x[..n] != y[..n] {
            return false;
        }
        if n == 0 {
            return right.read(&mut y[..1]).unwrap() == 0;
        }
    }
}
#[test]
#[ignore = "opt-in foreground DEC-17 A2/A3; never product feature"]
fn full_benchmark() {
    let output =
        std::env::var_os("LINTRA_DEC17_FULL_OUTPUT").expect("exclusive result path required");
    if std::env::var("LINTRA_DEC17_HEAP_MODE").as_deref() == Ok("off") {
        super::selective_parallelism::disable_heap_counter();
    }
    let smoke = std::env::var_os("LINTRA_DEC17_SMOKE").is_some();
    let workloads = if smoke {
        vec![(400, false, false, false), (400, false, true, true)]
    } else {
        vec![
            (10_000, false, false, false),
            (100_000, false, false, false),
            (500_000, false, false, false),
            (100_000, true, false, false),
            (500_000, true, false, false),
            (100_000, false, true, false),
            (500_000, false, true, false),
            (100_000, false, false, true),
        ]
    };
    let mut measurements = vec![];
    for (records, skewed, heavy, projection_heavy) in workloads {
        let fixture = Fixture::with_projection(records, skewed, heavy, projection_heavy);
        let (baseline, reference) = run(&fixture, 0);
        for repetition in 0..if smoke { 1 } else { 3 } {
            let mut order = vec![0, 1, 2, 4];
            if repetition == 1 {
                order.reverse();
            } else if repetition == 2 {
                order.rotate_left(2);
            }
            for workers in order {
                let (mut result, file) = run(&fixture, workers);
                let mut differences = vec![];
                compare(
                    &baseline["summary"],
                    &result["summary"],
                    "",
                    &mut differences,
                );
                assert!(files_equal(&reference, &file));
                assert_eq!(baseline["jsonl_sha256"], result["jsonl_sha256"]);
                assert_eq!(baseline["progress_sha256"], result["progress_sha256"]);
                result["summary_elapsed_differences"] = json!(differences);
                result["reference_summary_original_utf8"] =
                    baseline["summary_original_utf8"].clone();
                result["records"] = json!(records);
                result["input_bytes"] = json!(
                    fixture
                        .paths
                        .iter()
                        .map(|p| fs::metadata(p).unwrap().len())
                        .sum::<u64>()
                );
                result["skewed"] = json!(skewed);
                result["error_heavy"] = json!(heavy);
                result["projection_heavy"] = json!(projection_heavy);
                result["repetition"] = json!(repetition);
                result["byte_equal"] = json!(true);
                println!(
                    "FULL n={records} skew={skewed} heavy={heavy} rep={repetition} workers={workers} seconds={}",
                    result["total_seconds"]
                );
                measurements.push(result);
                file.close().unwrap();
            }
        }
        reference.close().unwrap();
        fixture.root.close().unwrap();
    }
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .unwrap();
    serde_json::to_writer_pretty(file,&json!({"task":"T-P17-05","activity":"A2/A3","source_commit":env!("LINTRA_BUILD_COMMIT"),"synthetic":true,"available_parallelism":thread::available_parallelism().unwrap().get(),"measurements":measurements,"dynamic_cancellation_equivalent":false,"adoption_authorized":false,"compiler_excluded":true,"schema_compilation_included":true,"normal_api_changed":false})).unwrap();
}

#[test]
fn dense_projection_and_incomplete_context_remain_equivalent() {
    for case in 0..3 {
        let fixture = Fixture::with_projection(400, false, false, true);
        if case == 1 {
            fs::write(&fixture.paths[1], b"").unwrap();
        }
        if case == 2 {
            fs::write(&fixture.paths[1], b"bad\n").unwrap();
        }
        CAPTURE_CONTEXT.set(true);
        let (baseline, original) = run(&fixture, 0);
        let context = CONTEXT.with_borrow(|v| v.clone());
        for workers in [1, 2, 4] {
            let (actual, file) = run(&fixture, workers);
            compare(&baseline["summary"], &actual["summary"], "", &mut vec![]);
            assert_eq!(context, CONTEXT.with_borrow(|v| v.clone()));
            assert!(files_equal(&original, &file));
            file.close().unwrap();
        }
        CAPTURE_CONTEXT.set(false);
        original.close().unwrap();
        fixture.root.close().unwrap();
    }
}
