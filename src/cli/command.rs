//! Wire public application/report APIs. No normative rule belongs in this adapter.
use super::{args::ValidateArgs, error_spool::ErrorSpool, paths};
use lintra::*;
use std::{
    error::Error,
    fmt, fs,
    io::{self, BufReader, Write},
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
pub(super) fn abort(
    errors: Option<ErrorSpool>,
    outputs: Vec<AtomicReportFile>,
    primary: Failure,
) -> Failure {
    let mut cleanup: Vec<_> = outputs
        .into_iter()
        .filter_map(|output| output.abort().err())
        .collect();
    if let Some(errors) = errors
        && let Err(error) = errors.abort()
    {
        cleanup.push(error);
    }
    Box::new(FailedRun { primary, cleanup })
}
fn invalid(message: &'static str) -> Failure {
    io::Error::new(io::ErrorKind::InvalidInput, message).into()
}
/// Controlled cancellation remains exit130 unless cleanup itself fails. All
/// guards are explicitly aborted and every cleanup error is retained.
pub(super) fn discard(errors: ErrorSpool, outputs: Vec<AtomicReportFile>) -> Result<(), Failure> {
    let mut cleanup: Vec<_> = outputs
        .into_iter()
        .filter_map(|report| report.abort().err())
        .collect();
    if let Err(error) = errors.abort() {
        cleanup.push(error);
    }
    if cleanup.is_empty() {
        Ok(())
    } else {
        Err(Box::new(FailedRun {
            primary: io::Error::other("cancelled report cleanup failed").into(),
            cleanup,
        }))
    }
}
/// Resolve an existing target (including symlinks), or its existing parent for
/// a new filename. Missing parents remain technical errors; none are created.
pub(super) fn resolved(path: &Path) -> io::Result<PathBuf> {
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
pub(super) fn check_outputs(
    args: &ValidateArgs,
    roots: &[PathBuf],
    manifest: Option<&Path>,
    extra_destinations: &[PathBuf],
    protected_inputs: &[PathBuf],
) -> Result<(), Failure> {
    let inputs = match &args.request.scope {
        ValidationScope::Trace(input) => vec![input],
        ValidationScope::Flow(flow) => flow.traces.iter().collect(),
    };
    let mut inputs = inputs
        .into_iter()
        .map(|input| {
            let InputSource::Path(path) = &input.source;
            fs::canonicalize(path)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for input in protected_inputs {
        inputs.push(fs::canonicalize(input)?);
    }
    if let Some(manifest) = manifest {
        inputs.push(fs::canonicalize(manifest)?);
    }
    let roots = roots
        .iter()
        .map(fs::canonicalize)
        .collect::<Result<Vec<_>, _>>()?;
    let mut seen = Vec::new();
    for destination in args
        .errors
        .iter()
        .chain(std::iter::once(&args.summary))
        .chain(args.xlsx.iter())
        .chain(extra_destinations.iter())
    {
        let mut part = destination.as_os_str().to_os_string();
        part.push(".part");
        for path in [destination.clone(), PathBuf::from(part)] {
            let path = resolved(&path)?;
            if inputs.contains(&path)
                || roots.iter().any(|root| path.starts_with(root))
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
    let cancellation = super::signal::install()?;
    validate_with_token(args, &cancellation)
}
/// Keep signal registration outside execution so tests can exercise the exact
/// filesystem/service path with a controlled cooperative token on either OS.
fn validate_with_token(
    args: ValidateArgs,
    cancellation: &CancellationToken,
) -> Result<RunStatus, Failure> {
    let exe = std::env::current_exe()?;
    let source = paths::resolve_bundle_source(&args.dirs, &paths::SystemEnv, &exe)?;
    let selected = super::bundle::compose(&args, source)?;
    let protected_configuration = args.resource_budget.iter().cloned().collect::<Vec<_>>();
    check_outputs(
        &args,
        &selected.roots,
        selected.manifest_path.as_deref(),
        &[],
        &protected_configuration,
    )?;
    let resources = match super::resource_budget::prepare(&args, &selected, cancellation) {
        Err(error)
            if matches!(
                error.downcast_ref::<ResourcePreflightError>(),
                Some(ResourcePreflightError::Cancelled)
            ) =>
        {
            return Ok(RunStatus::Cancelled);
        }
        other => other?,
    };
    let service = selected.service;
    let mut errors = ErrorSpool::create(args.errors.as_deref())?;
    let mut summary =
        match AtomicReportFile::create_replacing(&args.summary, PartialOutputPolicy::Delete) {
            Ok(output) => output,
            Err(error) => return Err(abort(Some(errors), vec![], error.into())),
        };
    let mut xlsx = match &args.xlsx {
        Some(path) => match AtomicReportFile::create_replacing(path, PartialOutputPolicy::Delete) {
            Ok(output) => Some(output),
            Err(error) => return Err(abort(Some(errors), vec![summary], error.into())),
        },
        None => None,
    };
    let result = (|| -> Result<ValidationRunOutcome, Failure> {
        if let Some(resources) = &resources {
            resources.verify_unchanged(&service, &args.request, cancellation)?;
        }
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
        let outcome = service.validate_with_parameters(
            args.request,
            args.parameters,
            &mut sink,
            &mut progress,
            cancellation,
        )?;
        sink.finish()?;
        log_result?;
        log.flush()?;
        write_run_summary_with_bundle(summary.writer(), &outcome.summary, &selected.audit)?;
        // Human output uses its own stream. A broken pipe is a technical error
        // and must be observed while both report guards can still be aborted.
        let mut human = HumanReadableSink::new(io::stdout().lock());
        human.on_summary(&outcome.summary)?;
        drop(human.finish()?);
        if outcome.status != RunStatus::Cancelled
            && !outcome.summary.truncated
            && let Some(workbook) = xlsx.as_mut()
        {
            let reader = BufReader::new(errors.reader()?);
            XlsxReportWriter::new().write(reader, &outcome.summary, workbook.writer_send())?;
        }
        Ok(outcome)
    })();
    let outcome = match result {
        Ok(outcome) => outcome,
        Err(error) => {
            let mut reports = vec![summary];
            reports.extend(xlsx);
            if matches!(
                error.downcast_ref::<ResourcePreflightError>(),
                Some(ResourcePreflightError::Cancelled)
            ) {
                discard(errors, reports)?;
                return Ok(RunStatus::Cancelled);
            }
            return Err(abort(Some(errors), reports, error));
        }
    };
    let completion = ReportCompletion {
        run_status: outcome.status,
        truncated: outcome.summary.truncated,
    };
    if let Err(error) = errors.complete(completion) {
        let mut reports = vec![summary];
        reports.extend(xlsx);
        return Err(abort(None, reports, error.into()));
    }
    if let Err(error) = summary.complete(completion) {
        return Err(abort(None, xlsx.into_iter().collect(), error.into()));
    }
    // Last fallible action: Err/Cancelled cannot leave a new definitive XLSX.
    if let Some(workbook) = xlsx {
        workbook.complete(completion)?;
    }
    Ok(outcome.status)
}

/// Lint follows the same repository/registry/compiler path as validate. Loading
/// or compilation failures are tooling errors (1), never record violations (2).
/// The issue sequence is retained verbatim; the CLI does not infer new rules.
pub fn schema_lint(args: super::args::LintArgs) -> Result<u8, Failure> {
    use std::sync::Arc;
    let registry = SchemaRegistry::new(
        Arc::new(FileSystemSchemaRepository::new(args.schema_root)?),
        SchemaCompiler,
        args.encoding,
    );
    let stdout = io::stdout();
    let mut output = stdout.lock();
    let code = match registry.resolve_with_hash(&args.key) {
        Ok(resolved) => {
            for issue in resolved.warnings() {
                writeln!(output, "{}", super::tooling::issue_json(issue))?;
            }
            writeln!(
                output,
                "{}",
                serde_json::json!({"result": "VALID", "scope": "compiler_supported_subset"})
            )?;
            0
        }
        Err(SchemaResolutionError::Compilation(error)) => {
            for issue in &error.issues {
                writeln!(output, "{}", super::tooling::issue_json(issue))?;
            }
            writeln!(
                output,
                "{}",
                serde_json::json!({"result": "INVALID", "issues": error.issues.len()})
            )?;
            1
        }
        Err(error) => return Err(error.into()),
    };
    output.flush()?;
    Ok(code)
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    /// Exercise the same adapter with an already-cancelled token. No OS signal
    /// is needed on Windows to verify status 130 and atomic output preservation.
    #[test]
    fn controlled_cancellation_never_publishes_complete_reports() {
        for (persistent, budget_enabled) in [(false, false), (true, false), (true, true)] {
            let root = std::env::temp_dir().join(format!(
                "lintra cancelled {} {}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(root.join("schemas/SYNTHETIC/v1")).unwrap();
            fs::create_dir(root.join("reference-data")).unwrap();
            fs::write(
                root.join("schemas/SYNTHETIC/v1/flow.json"),
                include_bytes!("../../tests/fixtures/application/schema.json"),
            )
            .unwrap();
            fs::write(root.join("input.txt"), b"001A  \n").unwrap();
            fs::write(root.join("errors.jsonl"), b"old errors").unwrap();
            fs::write(root.join("summary.json"), b"old summary").unwrap();
            fs::write(root.join("report.xlsx"), b"old workbook").unwrap();
            fs::write(root.join("budget.json"), br#"{"version":1,"measurement_source":"synthetic cancellation probe","input_bytes":1,"reference_bytes":null}"#).unwrap();
            let args = ValidateArgs {
                resource_budget: budget_enabled.then(|| root.join("budget.json")),
                proceed_over_budget: budget_enabled,
                bundle_manifest: None,
                reference_policy: DatasetPreparationPolicy::ContinueWithAvailableEvidence,
                dirs: paths::CliDirs {
                    schema_dir: Some(root.join("schemas")),
                    reference_data_dir: Some(root.join("reference-data")),
                },
                parameters: std::collections::BTreeMap::new(),
                request: ValidationRequest {
                    schema_key: SchemaKey {
                        flow_id: FlowId("SYNTHETIC".into()),
                        version: SchemaVersion("v1".into()),
                    },
                    scope: ValidationScope::Trace(TraceValidationInput {
                        trace_id: TraceId("T1".into()),
                        source: InputSource::Path(root.join("input.txt")),
                    }),
                    mode: ValidationMode::LocalOnly,
                    options: ValidationOptions {
                        record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                        newline_policy: NewlinePolicy::AutoLfOrCrlf,
                        max_errors: None,
                    },
                },
                encoding: RawResolvedEncoding::Utf8,
                errors: persistent.then(|| root.join("errors.jsonl")),
                xlsx: Some(root.join("report.xlsx")),
                summary: root.join("summary.json"),
            };
            let cancellation = CancellationToken::default();
            cancellation.cancel();
            let status = validate_with_token(args, &cancellation).unwrap();
            assert_eq!(super::super::exit_code::from_status(status), 130);
            assert_eq!(fs::read(root.join("errors.jsonl")).unwrap(), b"old errors");
            assert_eq!(fs::read(root.join("summary.json")).unwrap(), b"old summary");
            assert!(!root.join("errors.jsonl.part").exists());
            assert!(!root.join("summary.json.part").exists());
            assert_eq!(fs::read(root.join("report.xlsx")).unwrap(), b"old workbook");
            assert!(!root.join("report.xlsx.part").exists());
            fs::remove_dir_all(root).unwrap();
        }
    }
}
