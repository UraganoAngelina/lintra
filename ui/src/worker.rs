//! Validation executes only on an owned worker thread. Two single-slot channels
//! carry bounded progress and one reliable terminal result; no error backlog.
use crate::analysis::PreparedAnalysis;
use lintra::*;
use std::{
    error::Error,
    fmt,
    io::{self, BufWriter},
    path::Path,
    sync::{
        Arc,
        mpsc::{self, Receiver, SyncSender, TryRecvError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Debug)]
pub enum WorkerError {
    Spawn(io::Error),
    TemporaryOutput(io::Error),
    Validation(ValidationRunError),
    Report(ReportError),
    Cleanup {
        original: Box<WorkerError>,
        cleanup: ReportError,
    },
    Panicked,
    Disconnected,
}
impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Spawn(_) => "Impossibile avviare l’analisi.",
            Self::TemporaryOutput(_) => "Impossibile preparare il report dell’analisi.",
            Self::Validation(_) => "Errore tecnico: l’analisi non è stata completata.",
            Self::Report(_) => "Impossibile completare il report dell’analisi.",
            Self::Cleanup { .. } => {
                "Errore tecnico con pulizia del report incompleto non riuscita."
            }
            Self::Panicked => "L’analisi si è interrotta senza un esito completo.",
            Self::Disconnected => "Impossibile ricevere l’esito dell’analisi.",
        })
    }
}
impl Error for WorkerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn(e) | Self::TemporaryOutput(e) => Some(e),
            Self::Validation(e) => Some(e),
            Self::Report(e) => Some(e),
            Self::Cleanup { original, .. } => Some(original.as_ref()),
            Self::Panicked | Self::Disconnected => None,
        }
    }
}
/// Latest observed counters for the CURRENT trace. A total or percentage is not
/// inferred from a bounded prefix, file size, or a different trace's count.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkerProgress {
    pub phase: Option<ValidationPhase>,
    pub trace: Option<TraceId>,
    pub trace_records: u64,
    pub cross_records: u64,
}
/// Owns a private temporary directory until the consumer finishes using JSONL.
/// An internal spool exists even when JSONL export was not selected by the user.
pub struct AnalysisReport {
    directory: tempfile::TempDir,
    audit: BundleAudit,
    input_path: std::path::PathBuf,
}
impl AnalysisReport {
    /// Captured from the validated run, never from the currently edited form.
    pub fn input_path(&self) -> &Path {
        &self.input_path
    }
    pub fn jsonl_path(&self) -> std::path::PathBuf {
        self.directory.path().join("errors.jsonl")
    }
    pub fn audit(&self) -> &BundleAudit {
        &self.audit
    }
    pub fn directory(&self) -> &Path {
        self.directory.path()
    }
}
pub struct CompletedAnalysis {
    /// Requested export formats are retained with this run. Export writers are
    /// connected in T-P16-06; this field never claims files were exported.
    pub requested_reports: crate::analysis::ReportSelection,
    pub outcome: ValidationRunOutcome,
    /// None for Cancelled/truncated; no partial file is a complete report.
    pub report: Option<Arc<AnalysisReport>>,
}
pub type WorkerResult = Result<CompletedAnalysis, WorkerError>;
type Notify = Arc<dyn Fn() + Send + Sync>;
struct ChannelObserver {
    sender: SyncSender<WorkerProgress>,
    notify: Notify,
    latest: WorkerProgress,
    last_sent: Option<Instant>,
}
impl ChannelObserver {
    fn emit(&mut self, boundary: bool, now: Instant) {
        if !boundary
            && self
                .last_sent
                .is_some_and(|last| now.duration_since(last) < Duration::from_millis(50))
        {
            return;
        }
        // Never block validation on the UI. A full slot already has a wakeup;
        // terminal summary reconciles exact counters after coalescing.
        if self.sender.try_send(self.latest.clone()).is_ok() {
            self.last_sent = Some(now);
            (self.notify)();
        }
    }
}
impl ProgressObserver for ChannelObserver {
    fn on_event(&mut self, event: ProgressEvent) {
        let boundary = match event {
            ProgressEvent::RunStarted => {
                self.latest = WorkerProgress::default();
                true
            }
            ProgressEvent::PhaseStarted(phase) => {
                self.latest.phase = Some(phase);
                true
            }
            ProgressEvent::TraceStarted { trace_id } => {
                self.latest.trace = Some(trace_id);
                self.latest.trace_records = 0;
                true
            }
            ProgressEvent::RecordsProcessed { trace_id, count } => {
                self.latest.trace = Some(trace_id);
                self.latest.trace_records = count;
                false
            }
            ProgressEvent::TraceCompleted { trace_id, count } => {
                self.latest.trace = Some(trace_id);
                self.latest.trace_records = count;
                true
            }
            ProgressEvent::CrossRecordsProcessed { count } => {
                self.latest.cross_records = count;
                false
            }
            ProgressEvent::RunCompleted => true,
        };
        self.emit(boundary, Instant::now());
    }
}

fn execute(
    prepared: PreparedAnalysis,
    progress: &mut dyn ProgressObserver,
    cancellation: &CancellationToken,
) -> WorkerResult {
    let input_path = match &prepared.request.scope {
        ValidationScope::Trace(input) => match &input.source {
            InputSource::Path(path) => path.clone(),
        },
        ValidationScope::Flow(_) => {
            return Err(WorkerError::TemporaryOutput(io::Error::new(
                io::ErrorKind::InvalidInput,
                "desktop worker requires trace scope",
            )));
        }
    };
    // Do not preempt the service's typed input error if the source disappeared
    // after preflight. A successful run normally has a canonical path; retain
    // an absolute lexical path as the non-reading fallback for this snapshot.
    let input_path = std::fs::canonicalize(&input_path)
        .or_else(|_| std::path::absolute(&input_path))
        .unwrap_or(input_path);
    // tempfile supplies a new owned install-independent directory. No input
    // pathname or patient identifier is used in report names or technical logs.
    let directory = tempfile::Builder::new()
        .prefix("lintra-analysis-")
        .tempdir()
        .map_err(WorkerError::TemporaryOutput)?;
    let mut pending = AtomicReportFile::create_replacing(
        &directory.path().join("errors.jsonl"),
        PartialOutputPolicy::Delete,
    )
    .map_err(WorkerError::Report)?;
    let result = {
        let mut sink = JsonlSink::new(BufWriter::new(pending.writer()));
        match prepared
            .service
            .validate(prepared.request, &mut sink, progress, cancellation)
        {
            Ok(outcome) => sink
                .finish()
                .map(|writer| {
                    drop(writer);
                    outcome
                })
                .map_err(WorkerError::Report),
            Err(error) => Err(WorkerError::Validation(error)),
        }
    };
    let outcome = match result {
        Ok(outcome) => outcome,
        Err(original) => {
            return match pending.abort() {
                Ok(_) => Err(original),
                Err(cleanup) => Err(WorkerError::Cleanup {
                    original: Box::new(original),
                    cleanup,
                }),
            };
        }
    };
    let completion = ReportCompletion {
        run_status: outcome.status,
        truncated: outcome.summary.truncated,
    };
    let published = pending.complete(completion).map_err(WorkerError::Report)?;
    let report = match published {
        AtomicOutputOutcome::Published => Some(Arc::new(AnalysisReport {
            directory,
            audit: prepared.audit,
            input_path,
        })),
        AtomicOutputOutcome::Unpublished { .. } => None,
    };
    Ok(CompletedAnalysis {
        requested_reports: prepared.reports,
        outcome,
        report,
    })
}

/// Owned thread/channel lifecycle. Drop cancels then joins; the UI explicitly
/// joins after its event loop ends. No detached worker, poll thread or daemon.
pub struct WorkerHandle {
    progress: Receiver<WorkerProgress>,
    terminal: Receiver<WorkerResult>,
    cancellation: CancellationToken,
    thread: Option<JoinHandle<()>>,
    received: bool,
}
impl WorkerHandle {
    pub fn start(
        prepared: PreparedAnalysis,
        cancellation: CancellationToken,
        notify: Notify,
    ) -> Result<Self, WorkerError> {
        let (progress_sender, progress) = mpsc::sync_channel(1);
        let (terminal_sender, terminal) = mpsc::sync_channel(1);
        let token = cancellation.clone();
        let thread = thread::Builder::new()
            .name("lintra-validation".into())
            .spawn(move || {
                let mut observer = ChannelObserver {
                    sender: progress_sender,
                    notify: notify.clone(),
                    latest: WorkerProgress::default(),
                    last_sent: None,
                };
                // A panic cannot be mislabeled as completed/cancelled. Unwinding
                // drops atomic guards and the private temporary directory.
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    execute(prepared, &mut observer, &token)
                }))
                .unwrap_or(Err(WorkerError::Panicked));
                // Exactly one terminal message, separate from droppable progress.
                // try_send cannot wait for a stopped/closed UI event loop.
                let _ = terminal_sender.try_send(result);
                (notify)();
            })
            .map_err(WorkerError::Spawn)?;
        Ok(Self {
            progress,
            terminal,
            cancellation,
            thread: Some(thread),
            received: false,
        })
    }
    pub fn cancel(&self) {
        self.cancellation.cancel();
    }
    pub fn try_progress(&self) -> Option<WorkerProgress> {
        self.progress.try_recv().ok()
    }
    pub fn try_result(&mut self) -> Option<WorkerResult> {
        if self.received {
            return None;
        }
        match self.terminal.try_recv() {
            Ok(result) => {
                self.received = true;
                Some(result)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.received = true;
                Some(Err(WorkerError::Disconnected))
            }
        }
    }
    /// Blocking consumption is for tests/shutdown, never the running UI loop.
    pub fn wait(mut self) -> WorkerResult {
        self.received = true;
        self.terminal
            .recv()
            .unwrap_or(Err(WorkerError::Disconnected))
    }
}
impl Drop for WorkerHandle {
    fn drop(&mut self) {
        self.cancel();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn progress_is_single_slot_and_throttled_without_an_error_backlog() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let (sender, receiver) = mpsc::sync_channel(1);
        let notifications = Arc::new(AtomicUsize::new(0));
        let calls = notifications.clone();
        let mut observer = ChannelObserver {
            sender,
            notify: Arc::new(move || {
                calls.fetch_add(1, Ordering::Relaxed);
            }),
            latest: WorkerProgress::default(),
            last_sent: None,
        };
        let start = Instant::now();
        observer.emit(false, start);
        for count in 1..=100_000 {
            observer.latest.trace_records = count;
            observer.emit(false, start + Duration::from_millis(100));
        }
        assert_eq!(notifications.load(Ordering::Relaxed), 1);
        assert_eq!(receiver.try_recv().unwrap().trace_records, 0);
        assert!(receiver.try_recv().is_err());
        observer.emit(false, start + Duration::from_millis(10));
        assert!(receiver.try_recv().is_err());
        observer.emit(false, start + Duration::from_millis(100));
        assert_eq!(receiver.try_recv().unwrap().trace_records, 100_000);
        assert_eq!(notifications.load(Ordering::Relaxed), 2);
    }
}
