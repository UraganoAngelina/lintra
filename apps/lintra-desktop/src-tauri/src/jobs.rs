//! One owned native worker; its JoinHandle is the reliable terminal slot.
use crate::{
    configuration::{PreparedIntent, fields, handle},
    ipc::{self, ErrorCode},
    progress::Progress,
};
use lintra::*;
use serde_json::{Map, Value, json};
use std::{
    fs::File,
    io::BufWriter,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RunRef {
    instance: String,
    number: u64,
}
impl RunRef {
    pub(crate) fn decode(value: &Value) -> Result<Self, ErrorCode> {
        let dto = value.as_object().ok_or(ErrorCode::InvalidMessage)?;
        ipc::version(dto)?;
        fields(dto, &["instance_id", "run_id"])?;
        let instance = dto["instance_id"]
            .as_str()
            .filter(|s| {
                s.len() == 32
                    && s.bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
            .ok_or(ErrorCode::InvalidMessage)?;
        let number = dto["run_id"]
            .as_str()
            .filter(|s| {
                !s.is_empty() && !s.starts_with('0') && s.bytes().all(|b| b.is_ascii_digit())
            })
            .ok_or(ErrorCode::InvalidMessage)?
            .parse()
            .map_err(|_| ErrorCode::InvalidMessage)?;
        Ok(Self {
            instance: instance.into(),
            number,
        })
    }
    pub(crate) fn dto(&self) -> Value {
        json!({"schemaVersion":1,"instance_id":self.instance,"run_id":self.number.to_string()})
    }
}

pub(crate) enum JobRequest {
    Start { plan: String, mode: ValidationMode },
    State { run: Option<RunRef> },
    Acknowledge { run: RunRef },
    Cancel { run: RunRef },
    Progress { run: RunRef },
}
impl JobRequest {
    pub(crate) fn decode(operation: &str, dto: &Map<String, Value>) -> Result<Self, ErrorCode> {
        ipc::version(dto)?;
        match operation {
            "validation_start" => {
                fields(dto, &["plan_ref", "mode"])?;
                let mode = match dto["mode"].as_str() {
                    Some("local_only") => ValidationMode::LocalOnly,
                    Some("full") => ValidationMode::Full,
                    _ => return Err(ErrorCode::InvalidMessage),
                };
                Ok(Self::Start {
                    plan: handle(dto, "plan_ref")?,
                    mode,
                })
            }
            "validation_state" => {
                fields(dto, &["run"])?;
                Ok(Self::State {
                    run: if dto["run"].is_null() {
                        None
                    } else {
                        Some(RunRef::decode(&dto["run"])?)
                    },
                })
            }
            "validation_cancel" | "validation_progress" => {
                fields(dto, &["run"])?;
                let run = RunRef::decode(&dto["run"])?;
                Ok(if operation == "validation_cancel" {
                    Self::Cancel { run }
                } else {
                    Self::Progress { run }
                })
            }
            "validation_acknowledge" => {
                fields(dto, &["run"])?;
                Ok(Self::Acknowledge {
                    run: RunRef::decode(&dto["run"])?,
                })
            }
            _ => Err(ErrorCode::UnknownOperation),
        }
    }
}

struct WorkerState {
    stage: &'static str,
    sealed: bool,
    progress: Progress,
}
/// A published private report. Field order is drop order: the shared reader
/// must close before the TempDir is deleted (required on Windows).
pub(crate) struct Report {
    // Unfiltered library index/cache built once at publication (P4-07). The
    // Mutex only grants interior mutability for the row cache through Arc:
    // every access happens on the single serialized IPC gate (P4-08 paging).
    index: Mutex<JsonlReportIndex>,
    // Ownership keeps the private canonical JSONL alive, never a wire path.
    directory: tempfile::TempDir,
    // Captured summary: scope, totals and histogram that filtered rescans
    // must verify again before a new query can be published.
    summary: ValidationSummary,
    // Captured schema, paths, parameters and provenance belong to this report.
    _intent: Arc<PreparedIntent>,
}
impl Report {
    const FILE: &'static str = "errors.jsonl";

    /// Verify and index the committed spool. Any malformed, out-of-scope or
    /// miscounted row is a technical failure; no partial report is exposed.
    pub(crate) fn publish(
        directory: tempfile::TempDir,
        summary: ValidationSummary,
        intent: Arc<PreparedIntent>,
    ) -> Result<Self, ErrorCode> {
        let file = File::open(directory.path().join(Self::FILE))
            .map_err(|_| ErrorCode::TechnicalFailure)?;
        let index = JsonlReportIndex::scan(file, &summary, &ReportFilter::default(), || false)
            .map_err(|_| ErrorCode::TechnicalFailure)?;
        Ok(Self {
            index: Mutex::new(index),
            directory,
            summary,
            _intent: intent,
        })
    }
    /// Open an independent reader on the owned spool for a filtered rescan.
    /// The path is Rust-private and never derived from renderer input.
    pub(crate) fn open(&self) -> std::io::Result<File> {
        File::open(self.directory.path().join(Self::FILE))
    }
    pub(crate) fn summary(&self) -> &ValidationSummary {
        &self.summary
    }
    /// Captured scope traces in canonical order; position `i` is `trace_{i}`,
    /// the same opaque handle scheme already used by native progress.
    pub(crate) fn traces(&self) -> &[TraceId] {
        match &self.summary.scope_summary {
            ScopeSummary::Trace { trace_id } => std::slice::from_ref(trace_id),
            ScopeSummary::Flow { trace_ids } => trace_ids,
        }
    }
    /// Run `read` against the shared unfiltered index. A poisoned lock means a
    /// previous reader panicked mid-read: report a technical failure.
    pub(crate) fn with_index<T>(
        &self,
        read: impl FnOnce(&mut JsonlReportIndex) -> Result<T, ErrorCode>,
    ) -> Result<T, ErrorCode> {
        read(&mut *self.index.lock().map_err(|_| ErrorCode::TechnicalFailure)?)
    }
}
enum Completion {
    Validation {
        outcome: Box<ValidationRunOutcome>,
        report: Option<Arc<Report>>,
    },
    NotStarted,
}
type WorkerResult = Result<Completion, ErrorCode>;

fn execute(
    intent: Arc<PreparedIntent>,
    references: Arc<dyn ReferenceDataPreparer>,
    observer: &mut dyn ProgressObserver,
    token: &CancellationToken,
) -> WorkerResult {
    let mut repository = InMemorySchemaRepository::new();
    repository
        .insert((*intent.raw).clone())
        .map_err(|_| ErrorCode::TechnicalFailure)?;
    let registry = Arc::new(SchemaRegistry::new(
        Arc::new(repository),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    ));
    let expected = registry
        .resolve_with_hash(&intent.request.schema_key)
        .map_err(|_| ErrorCode::TechnicalFailure)?;
    let service = ValidationService::new(registry).with_reference_data(
        references,
        DatasetPreparationPolicy::ContinueWithAvailableEvidence,
    );
    let directory = tempfile::Builder::new()
        .prefix("lintra-desktop-run-")
        .tempdir()
        .map_err(|_| ErrorCode::TechnicalFailure)?;
    let mut pending = AtomicReportFile::create_replacing(
        &directory.path().join("errors.jsonl"),
        PartialOutputPolicy::Delete,
    )
    .map_err(|_| ErrorCode::TechnicalFailure)?;
    let result = (|| {
        let mut sink = JsonlSink::new(BufWriter::new(pending.writer()));
        let outcome = service
            .validate_with_typed_parameters(
                intent.request.clone(),
                intent.parameters.clone(),
                &mut sink,
                observer,
                token,
            )
            .map_err(|_| ErrorCode::TechnicalFailure)?;
        drop(sink.finish().map_err(|_| ErrorCode::TechnicalFailure)?);
        if outcome.summary.schema_hash != expected.schema_hash().to_string()
            || outcome.summary.schema_key != intent.request.schema_key
        {
            return Err(ErrorCode::TechnicalFailure);
        }
        Ok(outcome)
    })();
    let outcome = match result {
        Ok(outcome) => outcome,
        Err(code) => {
            pending.abort().map_err(|_| ErrorCode::TechnicalFailure)?;
            directory.close().map_err(|_| ErrorCode::TechnicalFailure)?;
            return Err(code);
        }
    };
    let published = pending
        .complete(ReportCompletion {
            run_status: outcome.status,
            truncated: outcome.summary.truncated,
        })
        .map_err(|_| ErrorCode::TechnicalFailure)?;
    let report = match published {
        AtomicOutputOutcome::Published => {
            // The final service checkpoint fixes the outcome: late cancellation
            // must not discard a published report during index preparation.
            Some(Arc::new(Report::publish(
                directory,
                outcome.summary.clone(),
                intent,
            )?))
        }
        AtomicOutputOutcome::Unpublished { .. } => {
            directory.close().map_err(|_| ErrorCode::TechnicalFailure)?;
            None
        }
    };
    Ok(Completion::Validation {
        outcome: Box::new(outcome),
        report,
    })
}

struct Job {
    run: RunRef,
    request_id: String,
    plan: String,
    mode: ValidationMode,
    shared: Arc<Mutex<WorkerState>>,
    token: CancellationToken,
    thread: Option<JoinHandle<WorkerResult>>,
    result: Option<WorkerResult>,
    acknowledged: bool,
}
impl Job {
    fn poll(&mut self) {
        if self.thread.as_ref().is_some_and(JoinHandle::is_finished) {
            // is_finished avoids waiting on input/report I/O in the live UI loop.
            self.result = Some(
                self.thread
                    .take()
                    .expect("finished owned thread")
                    .join()
                    .unwrap_or(Err(ErrorCode::TechnicalFailure)),
            );
        }
    }
    fn outcome(&self) -> Value {
        match self.result.as_ref() {
            None => Value::Null,
            Some(Err(_)) => {
                json!({"schemaVersion":1,"kind":"technical_failure","error":{"schemaVersion":1,"category":"technical","code":"IPC_TECHNICAL_FAILURE"}})
            }
            Some(Ok(Completion::NotStarted)) => {
                json!({"schemaVersion":1,"kind":"not_started","reason":"cancelled"})
            }
            Some(Ok(Completion::Validation { outcome, report })) => {
                json!({"schemaVersion":1,"kind":"validation","status":outcome.status,"completeness":outcome.summary.completeness(),"truncated":outcome.summary.truncated,"report_available":report.is_some()})
            }
        }
    }
    fn snapshot(&self) -> Result<Value, ErrorCode> {
        // A broken progress lock must never hide an already joined terminal.
        let stage = if self.result.is_some() {
            "terminal"
        } else {
            self.shared
                .lock()
                .map_err(|_| ErrorCode::TechnicalFailure)?
                .stage
        };
        Ok(
            json!({"schemaVersion":1,"run":self.run.dto(),"plan_ref":self.plan,"mode":self.mode,
            "state":stage,"cancel_requested":self.token.is_cancelled(),"outcome":self.outcome()}),
        )
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        self.token.cancel();
        if let Some(thread) = self.thread.take() {
            // Only replacement after terminal or shutdown reaches this blocking join.
            let _ = thread.join();
        }
    }
}

#[derive(Default)]
pub(crate) struct Jobs {
    number: u64,
    current: Option<Job>,
    // Queries and their worker are dropped (cancelled and joined) before the
    // report they borrow through Arc.
    results: crate::results::Results,
    last_report: Option<(RunRef, Arc<Report>)>,
}
impl Jobs {
    pub(crate) fn cancel_on_close(&self) {
        if let Some(job) = &self.current {
            job.token.cancel();
        }
        self.results.cancel_on_close();
    }
    /// Join a finished worker without blocking and retain its published report.
    /// A different published run invalidates every query of the previous one.
    fn refresh(&mut self) {
        let Some(job) = &mut self.current else {
            return;
        };
        job.poll();
        if let Some(Ok(Completion::Validation {
            report: Some(report),
            ..
        })) = &job.result
            && self
                .last_report
                .as_ref()
                .is_none_or(|(run, _)| run != &job.run)
        {
            self.results.reset();
            self.last_report = Some((job.run.clone(), report.clone()));
        }
    }
    /// Filtered queries and pages over the last published report (P4-08).
    pub(crate) fn results(
        &mut self,
        request: crate::results::ResultsRequest,
        id: &str,
    ) -> Result<Vec<u8>, ErrorCode> {
        self.refresh();
        self.results
            .dispatch(request, self.last_report.as_ref(), id)
    }
    pub(crate) fn start(
        &mut self,
        instance: &str,
        request_id: &str,
        plan: String,
        intent: Arc<PreparedIntent>,
        references: Arc<dyn ReferenceDataPreparer>,
    ) -> Result<Vec<u8>, ErrorCode> {
        if let Some(job) = &mut self.current {
            job.poll();
            if job.result.is_none() {
                return Err(ErrorCode::Busy);
            }
            if !job.acknowledged || job.request_id == request_id {
                return Err(ErrorCode::InvalidState);
            }
        }
        let number = self
            .number
            .checked_add(1)
            .ok_or(ErrorCode::TechnicalFailure)?;
        let run = RunRef {
            instance: instance.into(),
            number,
        };
        let bytes = ipc::response(request_id, run.dto())?;
        let shared = Arc::new(Mutex::new(WorkerState {
            stage: "preparing",
            sealed: false,
            progress: Progress::new(&intent.request.scope),
        }));
        let state = shared.clone();
        let token = CancellationToken::default();
        let cancellation = token.clone();
        let mode = intent.request.mode;
        let thread = thread::Builder::new()
            .name("lintra-desktop-validation".into())
            .spawn(move || {
                {
                    let mut state = state.lock().map_err(|_| ErrorCode::TechnicalFailure)?;
                    if cancellation.is_cancelled() {
                        return Ok(Completion::NotStarted);
                    }
                    state.stage = "running";
                }
                let mut observer = |event| {
                    if let Ok(mut state) = state.lock()
                        && !state.sealed
                    {
                        state.progress.update(&event);
                        if matches!(
                            event,
                            ProgressEvent::PhaseStarted(ValidationPhase::FinalizingReport)
                                | ProgressEvent::RunCompleted
                        ) {
                            state.stage = "finalizing";
                        }
                        if event == ProgressEvent::RunCompleted {
                            state.sealed = true;
                        }
                    }
                };
                // Unwinding drops pending report guards; a panic is never normative invalidity.
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    execute(intent, references, &mut observer, &cancellation)
                }))
                .unwrap_or(Err(ErrorCode::TechnicalFailure))
            })
            .map_err(|_| ErrorCode::TechnicalFailure)?;
        self.number = number;
        self.current = Some(Job {
            run,
            request_id: request_id.into(),
            plan,
            mode,
            shared,
            token,
            thread: Some(thread),
            result: None,
            acknowledged: false,
        });
        Ok(bytes)
    }
    pub(crate) fn dispatch(&mut self, request: JobRequest, id: &str) -> Result<Vec<u8>, ErrorCode> {
        let requested = match &request {
            JobRequest::State { run } => run.as_ref(),
            JobRequest::Acknowledge { run }
            | JobRequest::Cancel { run }
            | JobRequest::Progress { run } => Some(run),
            _ => return Err(ErrorCode::InvalidState),
        };
        if requested.is_some_and(|run| self.current.as_ref().is_none_or(|job| &job.run != run)) {
            return Err(ErrorCode::StaleContext);
        }
        self.refresh();
        let Some(job) = &mut self.current else {
            return ipc::response(
                id,
                json!({"schemaVersion":1,"state":"idle","run":null,"outcome":null}),
            );
        };
        match request {
            JobRequest::State { .. } => ipc::response(id, job.snapshot()?),
            JobRequest::Progress { .. } => {
                let sample = job
                    .shared
                    .lock()
                    .map_err(|_| ErrorCode::TechnicalFailure)?
                    .progress
                    .clone();
                ipc::response(id, sample.dto(job.run.dto())?)
            }
            JobRequest::Cancel { .. } => {
                let disposition = if job.result.is_some() {
                    "already_terminal"
                } else {
                    let mut state = job.shared.lock().map_err(|_| ErrorCode::TechnicalFailure)?;
                    if state.sealed {
                        "too_late"
                    } else if job.token.is_cancelled() {
                        "already_requested"
                    } else {
                        job.token.cancel();
                        state.stage = "cancelling";
                        "requested"
                    }
                };
                ipc::response(
                    id,
                    json!({"schemaVersion":1,"run":job.run.dto(),"disposition":disposition}),
                )
            }
            JobRequest::Acknowledge { .. } => {
                if job.result.is_none() {
                    return Err(ErrorCode::InvalidState);
                }
                let bytes = ipc::response(
                    id,
                    json!({"schemaVersion":1,"run":job.run.dto(),"acknowledged":true}),
                )?;
                job.acknowledged = true;
                Ok(bytes)
            }
            _ => Err(ErrorCode::InvalidState),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::{
        fs,
        sync::mpsc,
        time::{Duration, Instant},
    };
    const INSTANCE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    pub(crate) fn fixture() -> (
        tempfile::TempDir,
        Arc<PreparedIntent>,
        Arc<dyn ReferenceDataPreparer>,
    ) {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("SYNTHETIC.txt");
        fs::write(&path, b"001A  \nX\n").unwrap();
        let bundle = AnalysisBundle::builtin().unwrap();
        let raw: RawFlowSchema = serde_json::from_str(include_str!(
            "../../../../tests/golden/synthetic/v1/schema.json"
        ))
        .unwrap();
        let intent = Arc::new(PreparedIntent {
            request: ValidationRequest {
                schema_key: SchemaKey {
                    flow_id: FlowId(raw.flow_id.clone()),
                    version: SchemaVersion(raw.schema_version.clone()),
                },
                scope: ValidationScope::Trace(TraceValidationInput {
                    trace_id: TraceId("T1".into()),
                    source: InputSource::Path(path),
                }),
                mode: ValidationMode::LocalOnly,
                options: ValidationOptions {
                    record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                    newline_policy: NewlinePolicy::AutoLfOrCrlf,
                    max_errors: None,
                },
            },
            parameters: Default::default(),
            raw: Arc::new(raw),
            audit: bundle.audit().clone(),
        });
        (directory, intent, bundle.references())
    }

    mod full_schema {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../tests/support/full_schema.rs"
        ));
    }
    /// Captured Full intent over the shared synthetic two-trace schema
    /// (`S` joins `T` on KEY and compares DATA), inputs in schema order.
    fn full_fixture(
        source: &[u8],
        target: &[u8],
    ) -> (
        tempfile::TempDir,
        Arc<PreparedIntent>,
        Arc<dyn ReferenceDataPreparer>,
    ) {
        let directory = tempfile::tempdir().unwrap();
        let raw: RawFlowSchema = serde_json::from_value(full_schema::raw()).unwrap();
        let traces = [("S", source), ("T", target)]
            .into_iter()
            .map(|(id, bytes)| {
                let path = directory.path().join(format!("{id}=synthetic.txt"));
                fs::write(&path, bytes).unwrap();
                TraceValidationInput {
                    trace_id: TraceId(id.into()),
                    source: InputSource::Path(path),
                }
            })
            .collect();
        let bundle = AnalysisBundle::builtin().unwrap();
        let intent = Arc::new(PreparedIntent {
            request: ValidationRequest {
                schema_key: SchemaKey {
                    flow_id: FlowId(raw.flow_id.clone()),
                    version: SchemaVersion(raw.schema_version.clone()),
                },
                scope: ValidationScope::Flow(FlowValidationInput { traces }),
                mode: ValidationMode::Full,
                options: ValidationOptions {
                    record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                    newline_policy: NewlinePolicy::AutoLfOrCrlf,
                    max_errors: None,
                },
            },
            parameters: Default::default(),
            raw: Arc::new(raw),
            audit: bundle.audit().clone(),
        });
        (directory, intent, bundle.references())
    }
    /// Direct call to the public service with the same captured request,
    /// parameters and reference preparer used by the native worker.
    fn direct(
        intent: &PreparedIntent,
        refs: Arc<dyn ReferenceDataPreparer>,
    ) -> (ValidationRunOutcome, Vec<u8>) {
        let mut repo = InMemorySchemaRepository::new();
        repo.insert((*intent.raw).clone()).unwrap();
        let service = ValidationService::new(Arc::new(SchemaRegistry::new(
            Arc::new(repo),
            SchemaCompiler,
            RawResolvedEncoding::Utf8,
        )))
        .with_reference_data(
            refs,
            DatasetPreparationPolicy::ContinueWithAvailableEvidence,
        );
        let mut sink = JsonlSink::new(Vec::new());
        let outcome = service
            .validate_with_typed_parameters(
                intent.request.clone(),
                intent.parameters.clone(),
                &mut sink,
                &mut |_| {},
                &CancellationToken::default(),
            )
            .unwrap();
        (outcome, sink.finish().unwrap())
    }
    /// Wall clock and elapsed are runtime metadata, not diagnostics.
    fn normalize(bytes: &[u8]) -> Vec<Value> {
        String::from_utf8_lossy(bytes)
            .lines()
            .map(|line| {
                let mut row: Value = serde_json::from_str(line).unwrap();
                if let Some(obj) = row.as_object_mut() {
                    obj.remove("started_at");
                    obj.remove("elapsed");
                }
                row
            })
            .collect()
    }
    fn value(bytes: Vec<u8>) -> Value {
        serde_json::from_slice(&bytes).unwrap()
    }
    fn current(jobs: &mut Jobs) -> Value {
        value(
            jobs.dispatch(JobRequest::State { run: None }, "state")
                .unwrap(),
        )["payload"]
            .clone()
    }
    fn wait(jobs: &mut Jobs) -> Value {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let state = current(jobs);
            if state["state"] == "terminal" {
                return state;
            }
            assert!(Instant::now() < deadline, "worker did not terminate");
            thread::yield_now();
        }
    }
    fn run(jobs: &Jobs) -> RunRef {
        jobs.current.as_ref().unwrap().run.clone()
    }
    fn start(
        jobs: &mut Jobs,
        intent: Arc<PreparedIntent>,
        refs: Arc<dyn ReferenceDataPreparer>,
        id: &str,
    ) {
        jobs.start(INSTANCE, id, "plan_1".into(), intent, refs)
            .unwrap();
    }

    #[test]
    fn actual_local_worker_retains_terminal_report_identity_and_matches_core_jsonl() {
        let (_directory, intent, refs) = fixture();
        let mut jobs = Jobs::default();
        start(&mut jobs, intent.clone(), refs.clone(), "start_1");
        let terminal = wait(&mut jobs);
        assert_eq!(terminal["outcome"]["status"], "completed_invalid");
        assert_eq!(terminal["outcome"]["report_available"], true);
        let first = run(&jobs);
        let (identity, report) = jobs.last_report.as_ref().unwrap();
        assert_eq!(*identity, first);
        {
            let index = report.index.lock().unwrap();
            assert_eq!(index.row_count(), 1);
            assert_eq!(index.total(), 1);
            assert_eq!(index.cache_len(), 0);
            assert_eq!(index.decoded_rows(), 0);
        }
        let old_path = report.directory.path().to_owned();
        let bytes = fs::read(old_path.join("errors.jsonl")).unwrap();
        assert!(!terminal.to_string().contains(old_path.to_str().unwrap()));
        // Compare canonical output with a direct call to the same public service.
        let mut repo = InMemorySchemaRepository::new();
        repo.insert((*intent.raw).clone()).unwrap();
        let registry = Arc::new(SchemaRegistry::new(
            Arc::new(repo),
            SchemaCompiler,
            RawResolvedEncoding::Utf8,
        ));
        let service = ValidationService::new(registry).with_reference_data(
            refs.clone(),
            DatasetPreparationPolicy::ContinueWithAvailableEvidence,
        );
        let mut sink = JsonlSink::new(Vec::new());
        service
            .validate_with_typed_parameters(
                intent.request.clone(),
                intent.parameters.clone(),
                &mut sink,
                &mut |_| {},
                &CancellationToken::default(),
            )
            .unwrap();
        let direct = sink.finish().unwrap();
        // Wall clock and elapsed are intentionally runtime metadata, not diagnostics.
        fn normalize(bytes: &[u8]) -> Vec<Value> {
            String::from_utf8_lossy(bytes)
                .lines()
                .map(|line| {
                    let mut row: Value = serde_json::from_str(line).unwrap();
                    if let Some(obj) = row.as_object_mut() {
                        obj.remove("started_at");
                        obj.remove("elapsed");
                    }
                    row
                })
                .collect()
        }
        assert_eq!(normalize(&bytes), normalize(&direct));
        assert_eq!(current(&mut jobs), terminal);
        assert_eq!(
            jobs.start(
                INSTANCE,
                "start_2",
                "plan_1".into(),
                intent.clone(),
                refs.clone()
            ),
            Err(ErrorCode::InvalidState)
        );
        jobs.dispatch(JobRequest::Acknowledge { run: first.clone() }, "ack")
            .unwrap();
        jobs.dispatch(JobRequest::Acknowledge { run: first.clone() }, "ack_again")
            .unwrap();
        assert_eq!(
            jobs.start(
                INSTANCE,
                "start_1",
                "plan_1".into(),
                intent.clone(),
                refs.clone()
            ),
            Err(ErrorCode::InvalidState)
        );
        // Failed new input keeps the previous report under its original run identity.
        let InputSource::Path(path) = &match &intent.request.scope {
            ValidationScope::Trace(t) => t,
            _ => unreachable!(),
        }
        .source;
        fs::remove_file(path).unwrap();
        start(&mut jobs, intent, refs, "start_2");
        assert_eq!(run(&jobs).number, 2);
        assert_eq!(
            jobs.dispatch(
                JobRequest::State {
                    run: Some(first.clone())
                },
                "stale"
            ),
            Err(ErrorCode::StaleContext)
        );
        assert_eq!(wait(&mut jobs)["outcome"]["kind"], "technical_failure");
        assert_eq!(jobs.last_report.as_ref().unwrap().0, first);
        assert!(old_path.exists());
        drop(jobs);
        assert!(!old_path.exists());
    }

    #[test]
    fn cancellation_after_final_checkpoint_preserves_completed_report() {
        let (_directory, intent, refs) = fixture();
        let token = CancellationToken::default();
        let mut observer = |event| {
            // RunCompleted follows the final checkpoint and precedes index preparation.
            if event == ProgressEvent::RunCompleted {
                token.cancel();
            }
        };
        let completion = execute(intent, refs, &mut observer, &token).unwrap();
        assert!(token.is_cancelled());
        let Completion::Validation { outcome, report } = completion else {
            panic!("service was entered")
        };
        assert_eq!(outcome.status, RunStatus::CompletedInvalid);
        assert_eq!(
            outcome.summary.completeness(),
            ValidationCompleteness::Complete
        );
        let mut report = report.unwrap();
        let report = Arc::get_mut(&mut report).unwrap();
        assert!(report.directory.path().join("errors.jsonl").exists());
        let index = report.index.get_mut().unwrap();
        assert_eq!(index.row_count(), 1);
        assert!(index.read_row(0).unwrap().is_some());
    }

    #[test]
    fn native_report_owns_shared_lazy_index_for_valid_and_invalid_output() {
        for valid in [true, false] {
            let (_input_directory, intent, refs) = fixture();
            if valid {
                let ValidationScope::Trace(input) = &intent.request.scope else {
                    unreachable!()
                };
                let InputSource::Path(path) = &input.source;
                fs::write(path, b"001A  \n").unwrap();
            }
            let completion =
                execute(intent, refs, &mut |_| {}, &CancellationToken::default()).unwrap();
            let Completion::Validation { outcome, report } = completion else {
                panic!("service was entered")
            };
            assert_eq!(
                outcome.status,
                if valid {
                    RunStatus::CompletedValid
                } else {
                    RunStatus::CompletedInvalid
                }
            );
            let mut report = report.unwrap();
            let path = report.directory.path().to_owned();
            let index = Arc::get_mut(&mut report).unwrap().index.get_mut().unwrap();
            assert_eq!(index.row_count(), usize::from(!valid));
            assert_eq!(index.offset_bytes(), usize::from(!valid) * 8);
            assert_eq!(index.decoded_rows(), 0);
            assert_eq!(index.cache_len(), 0);
            if valid {
                assert_eq!(index.read_row(0).unwrap(), None);
            } else {
                let error = index.read_row(0).unwrap().unwrap();
                assert_eq!(error.code.as_symbol().unwrap(), "E_RECORD_LENGTH");
                assert_eq!(error.line_number, 2);
                assert_eq!(index.read_row(0).unwrap().unwrap(), error);
                assert_eq!(index.decoded_rows(), 1);
                assert_eq!(index.cache_len(), 1);
            }
            let retained = report.clone();
            drop(report);
            assert!(
                path.exists(),
                "a retained report still owns reader and spool"
            );
            drop(retained);
            assert!(
                !path.exists(),
                "dropping the last reader releases the spool"
            );
        }
    }

    // A real owned thread holds finalization open so control races are deterministic.
    fn held_job(sealed: bool) -> (Jobs, mpsc::SyncSender<WorkerResult>) {
        let (tx, rx) = mpsc::sync_channel(0);
        let (_directory, intent, _) = fixture();
        let shared = Arc::new(Mutex::new(WorkerState {
            stage: "finalizing",
            sealed,
            progress: Progress::new(&intent.request.scope),
        }));
        let job = Job {
            run: RunRef {
                instance: INSTANCE.into(),
                number: 1,
            },
            request_id: "start".into(),
            plan: "plan_1".into(),
            mode: ValidationMode::LocalOnly,
            shared,
            token: CancellationToken::default(),
            thread: Some(thread::spawn(move || rx.recv().unwrap())),
            result: None,
            acknowledged: false,
        };
        (
            Jobs {
                number: 1,
                current: Some(job),
                results: Default::default(),
                last_report: None,
            },
            tx,
        )
    }
    #[test]
    fn cancel_is_idempotent_and_service_completion_is_not_terminal() {
        let (mut jobs, release) = held_job(false);
        let run = run(&jobs);
        let (_directory, intent, refs) = fixture();
        assert_eq!(
            jobs.start(INSTANCE, "second", "plan_1".into(), intent, refs),
            Err(ErrorCode::Busy)
        );
        assert_eq!(jobs.number, 1);
        for expected in ["requested", "already_requested"] {
            assert_eq!(
                value(
                    jobs.dispatch(JobRequest::Cancel { run: run.clone() }, "cancel")
                        .unwrap()
                )["payload"]["disposition"],
                expected
            );
        }
        assert_eq!(current(&mut jobs)["state"], "cancelling");
        assert_eq!(
            jobs.dispatch(JobRequest::Acknowledge { run: run.clone() }, "ack"),
            Err(ErrorCode::InvalidState)
        );
        jobs.current.as_ref().unwrap().shared.lock().unwrap().sealed = true;
        assert_eq!(
            value(
                jobs.dispatch(JobRequest::Cancel { run: run.clone() }, "late")
                    .unwrap()
            )["payload"]["disposition"],
            "too_late"
        );
        release.send(Err(ErrorCode::TechnicalFailure)).unwrap();
        assert_eq!(wait(&mut jobs)["outcome"]["kind"], "technical_failure");
        assert_eq!(
            value(jobs.dispatch(JobRequest::Cancel { run }, "done").unwrap())["payload"]["disposition"],
            "already_terminal"
        );
    }
    #[test]
    fn failed_progress_does_not_replace_reliable_terminal_and_drop_joins_owned_worker() {
        let (mut jobs, release) = held_job(true);
        let run = run(&jobs);
        jobs.current
            .as_ref()
            .unwrap()
            .shared
            .lock()
            .unwrap()
            .progress
            .update(&ProgressEvent::CrossRecordsProcessed { count: 1 });
        assert_eq!(
            jobs.dispatch(JobRequest::Progress { run }, "progress"),
            Err(ErrorCode::TechnicalFailure)
        );
        assert_eq!(current(&mut jobs)["state"], "finalizing");
        let token = jobs.current.as_ref().unwrap().token.clone();
        let (dropped_tx, dropped_rx) = mpsc::channel();
        let dropper = thread::spawn(move || {
            drop(jobs);
            dropped_tx.send(()).unwrap();
        });
        let deadline = Instant::now() + Duration::from_secs(5);
        while !token.is_cancelled() {
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        assert!(dropped_rx.try_recv().is_err());
        release.send(Err(ErrorCode::TechnicalFailure)).unwrap();
        dropped_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        dropper.join().unwrap();
    }
    #[test]
    fn terminal_survives_poisoned_progress_and_cancelled_service_never_publishes() {
        let (mut jobs, release) = held_job(true);
        let shared = jobs.current.as_ref().unwrap().shared.clone();
        let _ = std::panic::catch_unwind(move || {
            let _guard = shared.lock().unwrap();
            panic!("synthetic progress panic");
        });
        release.send(Err(ErrorCode::TechnicalFailure)).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !jobs
            .current
            .as_ref()
            .unwrap()
            .thread
            .as_ref()
            .unwrap()
            .is_finished()
        {
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        assert_eq!(wait(&mut jobs)["outcome"]["kind"], "technical_failure");
        let run = run(&jobs);
        assert_eq!(
            value(jobs.dispatch(JobRequest::Cancel { run }, "cancel").unwrap())["payload"]["disposition"],
            "already_terminal"
        );
        let (_directory, intent, refs) = fixture();
        let token = CancellationToken::default();
        token.cancel();
        let completion = execute(intent, refs, &mut |_| {}, &token).unwrap();
        match completion {
            Completion::Validation { outcome, report } => {
                assert_eq!(outcome.status, RunStatus::Cancelled);
                assert_eq!(
                    outcome.summary.completeness(),
                    ValidationCompleteness::Partial
                );
                assert!(report.is_none());
            }
            Completion::NotStarted => panic!("service was entered"),
        }
    }
    #[test]
    fn truncated_service_output_is_partial_and_has_no_published_report() {
        let (_directory, mut intent, refs) = fixture();
        Arc::get_mut(&mut intent)
            .unwrap()
            .request
            .options
            .max_errors = Some(1);
        let completion = execute(intent, refs, &mut |_| {}, &CancellationToken::default()).unwrap();
        match completion {
            Completion::Validation { outcome, report } => {
                assert_eq!(outcome.status, RunStatus::CompletedInvalid);
                assert!(outcome.summary.truncated);
                assert_eq!(
                    outcome.summary.completeness(),
                    ValidationCompleteness::Partial
                );
                assert!(report.is_none());
            }
            Completion::NotStarted => panic!("service was entered"),
        }
    }
    #[test]
    fn strict_run_dtos_reject_paths_versions_and_noncanonical_or_overflow_ids() {
        let valid = json!({"schemaVersion":1,"instance_id":INSTANCE,"run_id":"1"});
        for operation in [
            "validation_progress",
            "validation_cancel",
            "validation_acknowledge",
        ] {
            assert!(
                JobRequest::decode(
                    operation,
                    json!({"schemaVersion":1,"run":valid}).as_object().unwrap()
                )
                .is_ok()
            );
            for run in [
                json!({"schemaVersion":2,"instance_id":INSTANCE,"run_id":"1"}),
                json!({"schemaVersion":1,"instance_id":INSTANCE,"run_id":"01"}),
                json!({"schemaVersion":1,"instance_id":INSTANCE,"run_id":"18446744073709551616"}),
                json!({"schemaVersion":1,"instance_id":INSTANCE,"run_id":"1","path":"SYNTHETIC"}),
            ] {
                assert!(
                    JobRequest::decode(
                        operation,
                        json!({"schemaVersion":1,"run":run}).as_object().unwrap()
                    )
                    .is_err()
                );
            }
        }
        let (_directory, intent, refs) = fixture();
        let mut jobs = Jobs {
            number: u64::MAX,
            ..Default::default()
        };
        assert_eq!(
            jobs.start(INSTANCE, "start", "plan_1".into(), intent, refs),
            Err(ErrorCode::TechnicalFailure)
        );
        assert!(jobs.current.is_none());
    }

    /// P4-11 acceptance: one owned Full worker per bundle produces exactly the
    /// outcome, summary and canonical JSONL of a direct call to the public
    /// service, across valid, cross-invalid, missing-related and Unknown/Partial
    /// synthetic bundles. Only wall clock/elapsed runtime metadata may differ.
    #[test]
    fn full_worker_matches_direct_service_outcome_summary_and_canonical_jsonl() {
        for (source, target, status, completeness, code) in [
            (
                &b"01AAOK\n"[..],
                &b"11AAOK\n"[..],
                RunStatus::CompletedValid,
                ValidationCompleteness::Complete,
                None,
            ),
            (
                b"01AANO\n02BBOK\n",
                b"11AAOK\n12BBOK\n",
                RunStatus::CompletedInvalid,
                ValidationCompleteness::Complete,
                Some("E_CROSS_FIELD"),
            ),
            (
                b"01CCOK\n",
                b"11AAOK\n",
                RunStatus::CompletedInvalid,
                ValidationCompleteness::Complete,
                Some("E_RELATED_RECORD_MISSING"),
            ),
            (
                b"01AAOK\n",
                b"bad\n11BBOK\n",
                RunStatus::CompletedInvalid,
                ValidationCompleteness::Partial,
                Some("E_RECORD_LENGTH"),
            ),
        ] {
            let (directory, intent, refs) = full_fixture(source, target);
            let mut jobs = Jobs::default();
            start(&mut jobs, intent.clone(), refs.clone(), "start_full");
            let terminal = wait(&mut jobs);
            assert_eq!(terminal["mode"], "full");
            assert_eq!(terminal["outcome"]["report_available"], true);
            // Per-trace/pass progress stays a valid single sample of one run.
            let run = run(&jobs);
            let progress = value(
                jobs.dispatch(JobRequest::Progress { run: run.clone() }, "progress")
                    .unwrap(),
            )["payload"]
                .clone();
            assert_eq!(progress["run"], run.dto());
            assert!(
                ["cross_record_validation", "finalizing_report"]
                    .contains(&progress["phase"].as_str().unwrap()),
                "{progress}"
            );
            let job = jobs.current.as_ref().unwrap();
            let Some(Ok(Completion::Validation {
                outcome,
                report: Some(report),
            })) = &job.result
            else {
                panic!("Full run must publish its report")
            };
            let (expected, expected_jsonl) = direct(&intent, refs);
            assert_eq!(outcome.status, status);
            assert_eq!(outcome.summary.completeness(), completeness);
            assert_eq!(outcome.summary.validation_mode, ValidationMode::Full);
            assert!(matches!(
                &outcome.summary.scope_summary,
                ScopeSummary::Flow { trace_ids }
                    if trace_ids.iter().map(|t| t.0.as_str()).collect::<Vec<_>>() == ["S", "T"]
            ));
            let mut actual = outcome.summary.clone();
            actual.started_at = expected.summary.started_at;
            actual.elapsed = expected.summary.elapsed;
            assert_eq!(outcome.status, expected.status);
            assert_eq!(actual, expected.summary);
            let bytes = fs::read(report.directory.path().join("errors.jsonl")).unwrap();
            assert_eq!(normalize(&bytes), normalize(&expected_jsonl));
            // The report is shared (Arc) since P4-08: read the unfiltered index
            // through its Mutex accessor, without consuming any row.
            let (total, rows, decoded) = report
                .with_index(|index| Ok((index.total(), index.row_count(), index.decoded_rows())))
                .unwrap();
            assert_eq!(total, outcome.summary.total_errors);
            assert_eq!(rows as u64, outcome.summary.total_errors);
            assert_eq!(decoded, 0, "no eager decoding");
            assert!(
                !terminal
                    .to_string()
                    .contains(directory.path().to_str().unwrap())
            );
            // Canonical service order is preserved: no adapter merge or sort.
            let codes = normalize(&bytes)
                .iter()
                .filter_map(|row| row.get("code").and_then(Value::as_str).map(str::to_owned))
                .collect::<Vec<_>>();
            match code {
                None => assert!(codes.is_empty(), "{codes:?}"),
                Some(code) => assert!(codes.iter().any(|c| c == code), "{codes:?}"),
            }
        }
    }

    /// Cooperative cancellation inside PASS1 and PASS2 of a Full bundle stays
    /// a Cancelled/Partial outcome without a published report; a missing
    /// bundle input is technical, never completed_valid/invalid.
    #[test]
    fn full_cancel_in_each_pass_and_input_failure_never_publish_or_complete() {
        for phase in [
            ValidationPhase::LocalValidation,
            ValidationPhase::CrossRecordValidation,
        ] {
            let (_directory, intent, refs) = full_fixture(b"01AANO\n", b"11AAOK\n");
            let token = CancellationToken::default();
            let mut observer = |event| {
                if event == ProgressEvent::PhaseStarted(phase) {
                    token.cancel();
                }
            };
            match execute(intent, refs, &mut observer, &token).unwrap() {
                Completion::Validation { outcome, report } => {
                    assert_eq!(outcome.status, RunStatus::Cancelled, "{phase:?}");
                    assert_eq!(
                        outcome.summary.completeness(),
                        ValidationCompleteness::Partial
                    );
                    assert!(report.is_none(), "{phase:?}");
                }
                Completion::NotStarted => panic!("service was entered"),
            }
        }
        let (_directory, intent, refs) = full_fixture(b"01AAOK\n", b"11AAOK\n");
        let ValidationScope::Flow(flow) = &intent.request.scope else {
            unreachable!()
        };
        let InputSource::Path(path) = &flow.traces[1].source;
        fs::remove_file(path).unwrap();
        let mut jobs = Jobs::default();
        start(&mut jobs, intent, refs, "start_missing");
        let terminal = wait(&mut jobs);
        assert_eq!(terminal["outcome"]["kind"], "technical_failure");
        assert!(jobs.last_report.is_none());
    }

    /// The operational error limit truncates a Full bundle: Partial, never
    /// exportable, and no report is published.
    #[test]
    fn truncated_full_bundle_is_partial_without_published_report() {
        let (_directory, mut intent, refs) = full_fixture(b"01AANO\n02BBNO\n", b"11AAOK\n12BBOK\n");
        Arc::get_mut(&mut intent)
            .unwrap()
            .request
            .options
            .max_errors = Some(1);
        match execute(intent, refs, &mut |_| {}, &CancellationToken::default()).unwrap() {
            Completion::Validation { outcome, report } => {
                assert!(outcome.summary.truncated);
                assert_eq!(
                    outcome.summary.completeness(),
                    ValidationCompleteness::Partial
                );
                assert!(report.is_none());
            }
            Completion::NotStarted => panic!("service was entered"),
        }
    }
}
