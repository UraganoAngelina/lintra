//! Wire public application/report APIs. No normative rule belongs in this adapter.
use super::{
    args::ValidateArgs,
    paths::{self, BundleSource},
};
use lintra::*;
use std::{
    error::Error,
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

type Failure = Box<dyn Error>;

/// Retain the upstream failure as well as every explicit cleanup failure.
/// Drop remains a last resort, rather than hiding a failed abort operation.
#[derive(Debug)]
struct FailedRun {
    primary: Failure,
    cleanup: Vec<ReportError>,
}
impl fmt::Display for FailedRun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.primary)?;
        for error in &self.cleanup {
            write!(f, "; output cleanup failed: {error}")?;
        }
        Ok(())
    }
}
impl Error for FailedRun {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.primary.as_ref())
    }
}
/// Always attempt both aborts, even when the first fails. No old destination is
/// removed and the original error never becomes a successful validation outcome.
fn abort(outputs: Vec<AtomicReportFile>, primary: Failure) -> Failure {
    let cleanup = outputs
        .into_iter()
        .filter_map(|output| output.abort().err())
        .collect();
    Box::new(FailedRun { primary, cleanup })
}
fn invalid(message: &'static str) -> Failure {
    io::Error::new(io::ErrorKind::InvalidInput, message).into()
}
/// Resolve an existing target (including symlinks), or its existing parent for
/// a new filename. Missing parents remain technical errors; none are created.
fn resolved(path: &Path) -> io::Result<PathBuf> {
    match fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let name = path.file_name().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "output needs a filename")
            })?;
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            Ok(fs::canonicalize(parent)?.join(name))
        }
        Err(error) => Err(error),
    }
}
/// Protect input and the selected bundle from both final destinations and part
/// filenames. Cross-output part collisions would otherwise depend on write order.
fn check_outputs(args: &ValidateArgs, schema: &Path, reference: &Path) -> Result<(), Failure> {
    let ValidationScope::Trace(input) = &args.request.scope else {
        return Err(invalid("validate requires Trace scope"));
    };
    let InputSource::Path(input) = &input.source;
    let input = fs::canonicalize(input)?;
    let schema = fs::canonicalize(schema)?;
    let reference = fs::canonicalize(reference)?;
    let mut seen = Vec::new();
    for destination in [&args.errors, &args.summary] {
        let mut part = destination.as_os_str().to_os_string();
        part.push(".part");
        for path in [destination.clone(), PathBuf::from(part)] {
            let path = resolved(&path)?;
            if path == input
                || path.starts_with(&schema)
                || path.starts_with(&reference)
                || seen.contains(&path)
            {
                return Err(invalid(
                    "output paths must be distinct and outside the input and data bundle",
                ));
            }
            seen.push(path);
        }
    }
    Ok(())
}
/// The real service is composed with streaming JSONL and a separate summary.
/// Publication is atomic per file, as in T-P5-06; this is not a two-file transaction.
pub fn validate(args: ValidateArgs) -> Result<RunStatus, Failure> {
    let exe = std::env::current_exe()?;
    let BundleSource::External {
        schema_root,
        reference_data_root,
    } = paths::resolve_bundle_source(&args.dirs, &paths::SystemEnv, &exe)?;
    check_outputs(&args, &schema_root, &reference_data_root)?;
    let service = build_default_local_service(schema_root, args.encoding)?;
    let mut errors = AtomicReportFile::create_replacing(&args.errors, PartialOutputPolicy::Delete)?;
    let mut summary =
        match AtomicReportFile::create_replacing(&args.summary, PartialOutputPolicy::Delete) {
            Ok(output) => output,
            Err(error) => return Err(abort(vec![errors], error.into())),
        };
    let result = (|| -> Result<ValidationRunOutcome, Failure> {
        let mut sink = JsonlSink::new(errors.writer());
        let stderr = io::stderr();
        let mut log = stderr.lock();
        let mut log_result = Ok(());
        // These fixed structured events contain no record, path or identifier.
        // Observer I/O cannot return through its trait: latch it and check before
        // publishing. This preserves errors without adding a core logging dependency.
        let mut progress = |event| {
            let event = match event {
                ProgressEvent::RunStarted => "run_started",
                ProgressEvent::TraceStarted { .. } => "trace_started",
                ProgressEvent::TraceCompleted { .. } => "trace_completed",
                ProgressEvent::RunCompleted => "run_completed",
                _ => return,
            };
            if log_result.is_ok() {
                log_result = writeln!(log, "{{\"event\":\"{event}\"}}");
            }
        };
        let outcome = service.validate(
            args.request,
            &mut sink,
            &mut progress,
            &CancellationToken::default(),
        )?;
        sink.finish()?;
        log_result?;
        log.flush()?;
        write_run_summary(summary.writer(), &outcome.summary)?;
        // Human output uses its own stream. A broken pipe is a technical error
        // and must be observed while both report guards can still be aborted.
        let mut human = HumanReadableSink::new(io::stdout().lock());
        human.on_summary(&outcome.summary)?;
        drop(human.finish()?);
        Ok(outcome)
    })();
    let outcome = match result {
        Ok(outcome) => outcome,
        Err(error) => return Err(abort(vec![errors, summary], error)),
    };
    let completion = ReportCompletion {
        run_status: outcome.status,
        truncated: outcome.summary.truncated,
    };
    if let Err(error) = errors.complete(completion) {
        return Err(abort(vec![summary], error.into()));
    }
    summary.complete(completion)?;
    Ok(outcome.status)
}
