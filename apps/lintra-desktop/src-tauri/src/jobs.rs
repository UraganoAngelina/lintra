//! One owned native worker; its JoinHandle is the reliable terminal slot.
use crate::{
    configuration::{PreparedIntent, fields, handle},
    ipc::{self, ErrorCode},
};
use lintra::*;
use serde_json::{Map, Value, json};
use std::{
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
    fn decode(value: &Value) -> Result<Self, ErrorCode> {
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
    fn dto(&self) -> Value {
        json!({"schemaVersion":1,"instance_id":self.instance,"run_id":self.number.to_string()})
    }
}

pub(crate) enum JobRequest {
    Start { plan: String, mode: ValidationMode },
    State { run: Option<RunRef> },
    Acknowledge { run: RunRef },
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
}
struct Report {
    // Ownership keeps the private canonical JSONL alive, never a wire path.
    _directory: tempfile::TempDir,
}
enum Completion {
    Validation {
        outcome: ValidationRunOutcome,
        report: Option<Arc<Report>>,
    },
    NotStarted,
}
type WorkerResult = Result<Completion, ErrorCode>;

fn execute(
    intent: &PreparedIntent,
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
        AtomicOutputOutcome::Published => Some(Arc::new(Report {
            _directory: directory,
        })),
        AtomicOutputOutcome::Unpublished { .. } => {
            directory.close().map_err(|_| ErrorCode::TechnicalFailure)?;
            None
        }
    };
    Ok(Completion::Validation { outcome, report })
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
        let state = self
            .shared
            .lock()
            .map_err(|_| ErrorCode::TechnicalFailure)?;
        Ok(
            json!({"schemaVersion":1,"run":self.run.dto(),"plan_ref":self.plan,"mode":self.mode,
            "state":if self.result.is_some() {"terminal"} else {state.stage},"cancel_requested":self.token.is_cancelled(),"outcome":self.outcome()}),
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
    last_report: Option<Arc<Report>>,
}
impl Jobs {
    pub(crate) fn cancel_on_close(&self) {
        if let Some(job) = &self.current {
            job.token.cancel();
        }
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
                    if let Ok(mut state) = state.lock() {
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
                    execute(&intent, references, &mut observer, &cancellation)
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
            JobRequest::Acknowledge { run } => Some(run),
            _ => return Err(ErrorCode::InvalidState),
        };
        if let Some(run) = requested
            && self.current.as_ref().is_none_or(|job| &job.run != run)
        {
            return Err(ErrorCode::StaleContext);
        }
        let Some(job) = &mut self.current else {
            return ipc::response(
                id,
                json!({"schemaVersion":1,"state":"idle","run":null,"outcome":null}),
            );
        };
        job.poll();
        if let Some(Ok(Completion::Validation {
            report: Some(report),
            ..
        })) = &job.result
        {
            self.last_report = Some(report.clone());
        }
        match request {
            JobRequest::State { .. } => ipc::response(id, job.snapshot()?),
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
