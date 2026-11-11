//! CLI adapter for the real historical service. Preview, report publication,
//! human review and repository approval remain separate observable operations.
use super::{
    args::{ArchiveArgs, HistoricalArgs},
    command,
    error_spool::ErrorSpool,
    paths,
};
use lintra::*;
use serde_json::json;
use std::{
    error::Error,
    fs,
    io::{self, BufReader, Write},
    path::PathBuf,
};

type Failure = Box<dyn Error>;
fn invalid() -> Failure {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "invalid historical request or review",
    )
    .into()
}
fn archive_path(args: &ArchiveArgs) -> Result<PathBuf, Failure> {
    let archive = std::path::absolute(&args.archive)?;
    let resolved = command::resolved(&archive)?;
    let key = fs::canonicalize(&args.key_file)?;
    if key.starts_with(&resolved) {
        return Err(invalid());
    }
    Ok(archive)
}

/// Resolve metadata only BEFORE composing repositories/preparers. In particular
/// an explicit or implicit bundle manifest must never parse the secret file into
/// an ordinary JSON buffer. Key bytes belong exclusively to the guarded loader.
fn preparation_paths(
    args: &HistoricalArgs,
    source: &paths::BundleSource,
    archive: &std::path::Path,
) -> Result<PathBuf, Failure> {
    let key = fs::canonicalize(&args.archive.key_file)?;
    let root = fs::canonicalize(archive)?;
    let manifest = match source {
        paths::BundleSource::External {
            schema_root,
            reference_data_root,
        } => {
            for data in [schema_root, reference_data_root] {
                if key.starts_with(fs::canonicalize(data)?) {
                    return Err(invalid());
                }
            }
            external_bundle_manifest_path(
                args.validation.bundle_manifest.as_deref(),
                schema_root,
                reference_data_root,
            )?
        }
        #[cfg(feature = "embedded-bundle")]
        paths::BundleSource::Embedded => args.validation.bundle_manifest.clone(),
    };
    if let Some(manifest) = manifest
        && fs::canonicalize(manifest)? == key
    {
        return Err(invalid());
    }
    let ValidationScope::Flow(inputs) = &args.validation.request.scope else {
        return Err(invalid());
    };
    for input in &inputs.traces {
        let InputSource::Path(path) = &input.source;
        let path = fs::canonicalize(path)?;
        if path == key || path.starts_with(&root) {
            return Err(invalid());
        }
    }
    Ok(root)
}

/// Init is explicit and uses the operator-supplied key. Open failures elsewhere
/// NEVER call this method as a recovery mechanism.
pub(super) fn initialize(args: ArchiveArgs) -> Result<(), Failure> {
    let provider = FilePersistenceKeyProvider::load(&args.key_id, &args.key_file)?;
    let archive = archive_path(&args)?;
    let _repository = SQLitePersistenceAdapter::create(&archive, Some(&args.key_id), &provider)?;
    json_line(
        &mut io::stdout().lock(),
        &json!({"event":"local_archive_created","format_version":1}),
    )?;
    Ok(())
}

/// Manual recovery verification uses the SAME key-confirmation boundary as a
/// real run, including an empty archive. No create, migration or commit fallback.
pub(super) fn check(args: ArchiveArgs) -> Result<(), Failure> {
    let provider = FilePersistenceKeyProvider::load(&args.key_id, &args.key_file)?;
    let archive = archive_path(&args)?;
    let repository = SQLitePersistenceAdapter::open(&archive, Some(&args.key_id), &provider)?;
    json_line(
        &mut io::stdout().lock(),
        &json!({"event":"local_archive_key_verified",
        "storage_format":repository.settings()?.format_version,"new_commit":false}),
    )?;
    Ok(())
}

struct AuditSink<'a> {
    diffs: &'a mut dyn Write,
    summary: &'a mut dyn Write,
    completed: bool,
}
impl HistoricalAuditSink for AuditSink<'_> {
    fn on_diff(&mut self, value: &HistoricalDiff) -> Result<(), ReportError> {
        json_line(self.diffs, value)
    }
    fn on_summary(&mut self, value: &HistoricalAuditSummary) -> Result<(), ReportError> {
        json_line(
            self.summary,
            &json!({"format_version":1,"phase":"historical_preview","audit":value}),
        )?;
        self.completed = true;
        Ok(())
    }
}
fn json_line(writer: &mut dyn Write, value: &impl serde::Serialize) -> Result<(), ReportError> {
    serde_json::to_writer(&mut *writer, value).map_err(ReportError::Serialize)?;
    writer.write_all(b"\n").map_err(ReportError::Write)?;
    writer.flush().map_err(ReportError::Write)
}

/// Own every guard until the analysis has completed. Publication is atomic per
/// file, exactly as in the ordinary CLI; it is not a report/DB transaction.
struct Reports {
    export_paths: Vec<PathBuf>,
    export_permission: AuthorizedReportDestinations,
    errors: ErrorSpool,
    summary: AtomicReportFile,
    audit: AtomicReportFile,
    audit_summary: AtomicReportFile,
    xlsx: Option<AtomicReportFile>,
}
impl Reports {
    fn create(args: &HistoricalArgs) -> Result<Self, Failure> {
        let (export_paths, export_permission) = command::authorize_report_outputs(
            &args.validation,
            &[args.audit.clone(), args.audit_summary.clone()],
        )?;
        let errors = ErrorSpool::create(args.validation.errors.as_deref())?;
        let mut reports = Vec::new();
        for path in [&args.validation.summary, &args.audit, &args.audit_summary]
            .into_iter()
            .chain(args.validation.xlsx.iter())
        {
            match AtomicReportFile::create_replacing(path, PartialOutputPolicy::Delete) {
                Ok(report) => reports.push(report),
                Err(error) => return Err(command::abort(Some(errors), reports, error.into())),
            }
        }
        let mut reports = reports.into_iter();
        Ok(Self {
            export_paths,
            export_permission,
            errors,
            summary: reports
                .next()
                .expect("three mandatory report guards were constructed"),
            audit: reports
                .next()
                .expect("three mandatory report guards were constructed"),
            audit_summary: reports
                .next()
                .expect("three mandatory report guards were constructed"),
            xlsx: reports.next(),
        })
    }
    fn parts(self) -> (ErrorSpool, Vec<AtomicReportFile>) {
        let mut reports = vec![self.summary, self.audit, self.audit_summary];
        reports.extend(self.xlsx);
        (self.errors, reports)
    }
    fn fail(self, error: Failure) -> Failure {
        let (errors, reports) = self.parts();
        command::abort(Some(errors), reports, error)
    }
    fn discard(self) -> Result<RunStatus, Failure> {
        let (errors, reports) = self.parts();
        command::discard(errors, reports)?;
        Ok(RunStatus::Cancelled)
    }
    fn publish(self, completion: ReportCompletion) -> Result<(), Failure> {
        // This check remains inside the genuine approval publication callback:
        // an invalidated destination prevents the transaction, never bypasses CAS.
        if completion.run_status != RunStatus::Cancelled
            && !completion.truncated
            && let Err(error) = self.export_permission.verify_unchanged(&self.export_paths)
        {
            return Err(self.fail(error.into()));
        }
        let (errors, reports) = self.parts();
        if let Err(error) = errors.complete(completion) {
            return Err(command::abort(None, reports, error.into()));
        }
        let mut reports = reports.into_iter();
        while let Some(report) = reports.next() {
            if let Err(error) = report.complete(completion) {
                return Err(command::abort(None, reports.collect(), error.into()));
            }
        }
        Ok(())
    }
}

pub(super) fn validate(args: HistoricalArgs) -> Result<RunStatus, Failure> {
    let cancellation = super::signal::install()?;
    execute(
        args,
        &cancellation,
        &mut io::stdout().lock(),
        |review, cancellation| {
            writeln!(
                io::stderr().lock(),
                "Per approvare digitare esattamente: {}\nOppure ABORT.",
                review.confirmation()
            )?;
            super::review_input::read(cancellation)
        },
    )
}

/// Injectable review I/O lets tests drive the exact adapter without global
/// stdin/signal mutation. The production reader polls cancellation on both OSes.
pub(super) fn execute(
    args: HistoricalArgs,
    cancellation: &CancellationToken,
    output: &mut dyn Write,
    read_review: impl FnOnce(&HistoricalReview, &CancellationToken) -> Result<Option<String>, Failure>,
) -> Result<RunStatus, Failure> {
    if cancellation.is_cancelled() {
        return Ok(RunStatus::Cancelled);
    }
    let provider = FilePersistenceKeyProvider::load(&args.archive.key_id, &args.archive.key_file)?;
    let archive = archive_path(&args.archive)?;
    let source = paths::resolve_bundle_source(
        &args.validation.dirs,
        &paths::SystemEnv,
        &std::env::current_exe()?,
    )?;
    let root = preparation_paths(&args, &source, &archive)?;
    let repository =
        SQLitePersistenceAdapter::open(&archive, Some(&args.archive.key_id), &provider)?;
    let mut selected = super::bundle::compose(&args.validation, source)?;
    selected.roots.push(root);
    command::check_outputs(
        &args.validation,
        &selected.roots,
        selected.manifest_path.as_deref(),
        &[args.audit.clone(), args.audit_summary.clone()],
        std::slice::from_ref(&args.archive.key_file),
    )?;
    if cancellation.is_cancelled() {
        return Ok(RunStatus::Cancelled);
    }
    let mut reports = Reports::create(&args)?;
    let service =
        HistoricalValidationService::new(&selected.service, &repository, repository.protector());
    let result = (|| -> Result<HistoricalValidationOutcome<'_>, HistoricalValidationError> {
        let mut sink = JsonlSink::new(reports.errors.writer());
        let mut audit = AuditSink {
            diffs: reports.audit.writer(),
            summary: reports.audit_summary.writer(),
            completed: false,
        };
        let outcome = service.validate_text(
            HistoricalTextValidationRequest {
                validation: args.validation.request,
                parameters: args.validation.parameters,
                submission_id: args.submission_id,
            },
            &mut sink,
            &mut audit,
            &mut |_| {},
            cancellation,
        )?;
        sink.finish().map_err(HistoricalValidationError::Report)?;
        if matches!(outcome, HistoricalValidationOutcome::Preview(_)) && !audit.completed {
            return Err(HistoricalValidationError::Report(
                ReportError::PreviousFailure,
            ));
        }
        Ok(outcome)
    })();
    let mut preview = match result {
        Err(HistoricalValidationError::Cancelled)
        | Err(HistoricalValidationError::Approval(SessionApprovalError::Cancelled)) => {
            return reports.discard();
        }
        Err(error) => return Err(reports.fail(error.into())),
        Ok(HistoricalValidationOutcome::ValidationOnly(outcome)) => {
            if outcome.status == RunStatus::Cancelled || cancellation.is_cancelled() {
                return reports.discard();
            }
            let result = (|| -> Result<(), Failure> {
                write_run_summary_with_bundle(
                    reports.summary.writer(),
                    &outcome.summary,
                    &selected.audit,
                )?;
                json_line(
                    reports.audit_summary.writer(),
                    &json!({"format_version":1,"historical_audit":"not_executed","reason":"no_complete_historical_preview"}),
                )?;
                write_xlsx(&mut reports, &outcome.summary)?;
                Ok(())
            })();
            if let Err(error) = result {
                return Err(reports.fail(error));
            }
            reports.publish(ReportCompletion {
                run_status: outcome.status,
                truncated: outcome.summary.truncated,
            })?;
            json_line(
                output,
                &json!({"event":"validation_only","run_status":outcome.status}),
            )?;
            if outcome.status == RunStatus::CompletedValid {
                // Normative validity is preserved in the report. Incomplete
                // evidence still cannot satisfy the requested historical
                // operation; exit1 never masquerades as a completed audit.
                return Err(io::Error::other(
                    "historical preview requires complete validation evidence",
                )
                .into());
            }
            return Ok(outcome.status);
        }
        Ok(HistoricalValidationOutcome::Preview(preview)) => preview,
    };
    if cancellation.is_cancelled() {
        return reports.discard();
    }
    let result = (|| -> Result<(), Failure> {
        write_run_summary_with_bundle(
            reports.summary.writer(),
            &preview.outcome().summary,
            &selected.audit,
        )?;
        write_xlsx(&mut reports, &preview.outcome().summary)?;
        Ok(())
    })();
    if let Err(error) = result {
        return Err(reports.fail(error));
    }
    if cancellation.is_cancelled() {
        return reports.discard();
    }
    let review = preview.review();
    let already_committed = preview.state() == Some(SessionState::ApprovedImmutable);
    if let Err(error) = json_line(
        output,
        &json!({"event":"historical_preview","coverage":"local_repository_only",
        "review":review,"confirmation":review.confirmation(),"audit":preview.audit(),
        "already_committed":already_committed,"new_commit":false,"reports_published":false}),
    ) {
        return Err(reports.fail(error.into()));
    }
    if !args.review || already_committed {
        reports.publish(ReportCompletion {
            run_status: RunStatus::CompletedValid,
            truncated: false,
        })?;
        json_line(
            output,
            &json!({"event":"preview_reports_published","new_commit":false}),
        )?;
        return Ok(RunStatus::CompletedValid);
    }
    let confirmation = match read_review(&review, cancellation) {
        Ok(value) => value,
        Err(error) => return Err(reports.fail(error)),
    };
    if cancellation.is_cancelled() || confirmation.as_deref().is_none_or(|value| value == "ABORT") {
        if preview.state() == Some(SessionState::Validated) {
            preview.cancel()?;
        }
        return reports.discard();
    }
    if confirmation.as_deref() != Some(review.confirmation().as_str()) {
        if preview.state() == Some(SessionState::Validated) {
            preview.cancel()?;
        }
        return Err(reports.fail(invalid()));
    }
    // Publishing happens INSIDE the real approval callback. A failed file or
    // cleanup is retained verbatim, and prevents beginning the transaction.
    let mut publication_error = None;
    let mut pending_reports = Some(reports);
    let result = preview.approve_reviewed(&review, cancellation, |_, _, _| {
        match pending_reports
            .take()
            .expect("FnOnce publishes owned guards once")
            .publish(ReportCompletion {
                run_status: RunStatus::CompletedValid,
                truncated: false,
            }) {
            Ok(()) => Ok(()),
            Err(error) => {
                publication_error = Some(error);
                Err(ReportError::PreviousFailure)
            }
        }
    });
    if let Some(reports) = pending_reports {
        return match result {
            Err(SessionApprovalError::Cancelled) => reports.discard(),
            Err(error) => Err(reports.fail(error.into())),
            Ok(_) => Err(reports.fail(invalid())),
        };
    }
    if let Some(error) = publication_error {
        return Err(error);
    }
    report_approval(result, || preview.resolve_outcome(), output)
}

/// Acknowledgement mapping is independent of storage mutation. Only an actual
/// successful receipt emits historical_commit; uncertainty remains technical
/// even when a single read-only reconciliation finds the candidate published.
fn report_approval(
    result: Result<CommitReceipt, SessionApprovalError>,
    resolve: impl FnOnce() -> Result<CommitResolution, SessionApprovalError>,
    output: &mut dyn Write,
) -> Result<RunStatus, Failure> {
    match result {
        Ok(receipt) => {
            json_line(
                output,
                &json!({"event":"historical_commit","revision":receipt.revision()}),
            )?;
            Ok(RunStatus::CompletedValid)
        }
        Err(SessionApprovalError::Cancelled) => Ok(RunStatus::Cancelled),
        Err(
            error @ SessionApprovalError::Persistence(PersistenceError::CommitOutcomeUnknown {
                ..
            }),
        ) => {
            // One read-only reconciliation, no automatic write retry and no
            // claim of rollback. The command still reports the technical error.
            let resolution = match resolve() {
                Ok(CommitResolution::Published(_)) => "published",
                Ok(CommitResolution::NotPublished) => "not_published",
                Ok(CommitResolution::ConflictingSubmission) => "conflicting_submission",
                Err(_) => "unavailable",
            };
            json_line(
                output,
                &json!({"event":"commit_outcome_unknown","read_only_resolution":resolution}),
            )?;
            Err(error.into())
        }
        Err(error) => Err(error.into()),
    }
}

fn write_xlsx(reports: &mut Reports, summary: &ValidationSummary) -> Result<(), Failure> {
    if summary.run_status == RunStatus::Cancelled || summary.truncated {
        return Ok(());
    }
    if let Some(workbook) = reports.xlsx.as_mut() {
        XlsxReportWriter::new().write(
            BufReader::new(reports.errors.reader()?),
            summary,
            workbook.writer_send(),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
