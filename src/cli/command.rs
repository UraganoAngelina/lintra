//! Wire public application/report APIs. No normative rule belongs in this adapter.
use super::{
    args::ValidateArgs,
    error_spool::ErrorSpool,
    paths::{self, BundleSource},
};
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
fn abort(errors: Option<ErrorSpool>, outputs: Vec<AtomicReportFile>, primary: Failure) -> Failure {
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
    for destination in args
        .errors
        .iter()
        .chain(std::iter::once(&args.summary))
        .chain(args.xlsx.iter())
    {
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
    let BundleSource::External {
        schema_root,
        reference_data_root,
    } = paths::resolve_bundle_source(&args.dirs, &paths::SystemEnv, &exe)?;
    check_outputs(&args, &schema_root, &reference_data_root)?;
    let service = build_local_service_with_reference_data(
        schema_root,
        reference_data_root,
        args.encoding,
        args.reference_policy,
    )?;
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
        let outcome = service.validate(args.request, &mut sink, &mut progress, cancellation)?;
        sink.finish()?;
        log_result?;
        log.flush()?;
        write_run_summary(summary.writer(), &outcome.summary)?;
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
/// or canonicalization failures remain technical (1); compiler issues return 2.
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
        Ok(_) => {
            writeln!(
                output,
                "{}",
                serde_json::json!({"result": "VALID", "scope": "compiler_supported_subset"})
            )?;
            0
        }
        Err(SchemaResolutionError::Compilation(error)) => {
            for issue in &error.issues {
                // JSON escaping prevents metadata/newlines from creating fake
                // diagnostics. Serialization changes presentation, never severity.
                writeln!(
                    output,
                    "{}",
                    serde_json::json!({
                        "severity": match issue.severity { SchemaIssueSeverity::Error => "error", SchemaIssueSeverity::Warning => "warning" },
                        "code": format!("{:?}", issue.code),
                        "flow_id": issue.flow_id.as_ref().map(|id| &id.0),
                        "trace_id": issue.trace_id.as_ref().map(|id| &id.0),
                        "field_id": issue.field_id.as_ref().map(|id| &id.0),
                        "message": issue.message,
                        "source_note": issue.source_note,
                    })
                )?;
            }
            writeln!(
                output,
                "{}",
                serde_json::json!({"result": "INVALID", "issues": error.issues.len()})
            )?;
            2
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
        for persistent in [false, true] {
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
            let args = ValidateArgs {
                reference_policy: DatasetPreparationPolicy::ContinueWithAvailableEvidence,
                dirs: paths::CliDirs {
                    schema_dir: Some(root.join("schemas")),
                    reference_data_dir: Some(root.join("reference-data")),
                },
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
