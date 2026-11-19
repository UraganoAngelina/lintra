//! Post-analysis export of a captured canonical report, shared by every
//! native adapter (Slint and Tauri desktop). Formats are written by the
//! existing writers; no rules, CSV quoting or workbook layouts are
//! implemented here. Each file is published by AtomicReportFile, and the
//! private spool and every input of the run are protected destinations.
//!
//! Extracted from the Slint adapter (T-MIG-P4-10), in the same way as the
//! shared JSONL index (P4-07): behaviour is unchanged, except that the
//! protected inputs are now a list, so a Full bundle protects all its inputs.
use crate::*;
use std::{
    error::Error,
    fmt,
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

/// The captured report an export reads, and every path it must never write.
///
/// `jsonl` is the committed canonical spool; `private_root` is its private
/// directory, protected as a whole (including future sidecars) and also the
/// XLSX writer's temporary directory. `protected_inputs` are all the input
/// files of the run (one for a trace run, all traces of a Full bundle).
pub struct ExportSource<'a> {
    pub jsonl: &'a Path,
    pub private_root: &'a Path,
    pub protected_inputs: &'a [PathBuf],
    pub audit: &'a BundleAudit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Xlsx,
    Jsonl,
    Csv,
}
impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Xlsx => "xlsx",
            Self::Jsonl => "jsonl",
            Self::Csv => "csv",
        }
    }
}
#[derive(Debug)]
pub enum ExportError {
    InvalidPath,
    InvalidExtension,
    ProtectedSource,
    NotRegularFile,
    OverwriteRequired,
    Destination(ReportDestinationError),
    Cancelled,
    IncompleteRun,
    Io(io::Error),
    Source(io::Error),
    Spawn(io::Error),
    Report(ReportError),
    Replay(CanonicalReplayError),
    Workbook(XlsxReportError),
    WorkerUnavailable,
}
impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if matches!(
            self,
            Self::Report(ReportError::PublishedDurabilityUnconfirmed { .. })
        ) {
            return f.write_str("Il file è visibile, ma il sistema non ha confermato la scrittura su disco. Controlla la destinazione prima di riprovare.");
        }
        if nested_io_kind(self) == Some(io::ErrorKind::StorageFull) {
            return f.write_str("Spazio insufficiente per il report o per i file temporanei. Libera spazio e riprova.");
        }
        f.write_str(match self {
            Self::InvalidPath=>"Inserisci il percorso completo del file in una cartella esistente.",
            Self::InvalidExtension=>"L’estensione del file deve corrispondere al formato scelto.",
            Self::ProtectedSource=>"Scegli un altro file: questa destinazione coincide con il file analizzato o con il report sorgente.",
            Self::NotRegularFile=>"La destinazione è una cartella o un collegamento. Scegli un file regolare.",
            Self::OverwriteRequired=>"Il report o il relativo riepilogo esiste già. Conferma la sostituzione oppure cambia nome.",
            Self::Destination(ReportDestinationError::ConfirmationRequired(assessment)) if assessment.counts().undetermined_files > 0 =>"Non è possibile determinare se la destinazione è locale o condivisa. Verifica gli accessi alla cartella e conferma il salvataggio.",
            Self::Destination(ReportDestinationError::ConfirmationRequired(_)) =>"La destinazione è remota o condivisa. Altre persone potrebbero accedere ai file. Verifica gli accessi e conferma il salvataggio.",
            Self::Destination(ReportDestinationError::SelectionChanged) =>"La destinazione è cambiata. Verifica il percorso e ripeti il salvataggio con una nuova conferma se richiesta.",
            Self::Destination(ReportDestinationError::Io(_)) =>"Impossibile verificare la cartella di destinazione. Controlla che sia accessibile.",
            Self::Cancelled=>"Salvataggio annullato prima della pubblicazione dei file.",
            Self::IncompleteRun=>"Il run è annullato o troncato: non è disponibile un report completo da esportare.",
            Self::Io(e) if e.kind()==io::ErrorKind::NotFound=>"La cartella di destinazione non esiste. Scegli una cartella esistente.",
            Self::Io(e) if e.kind()==io::ErrorKind::PermissionDenied=>"Non hai i permessi per scrivere nella cartella scelta. Scegli una cartella scrivibile.",
            Self::Io(_)=>"Impossibile accedere ai file del report. Verifica la destinazione e lo spazio disponibile.",
            Self::Report(ReportError::Output{source,..}) if source.kind()==io::ErrorKind::AlreadyExists=>"Esiste già un file temporaneo di salvataggio. Verifica che non sia in corso un’altra esportazione o scegli un altro nome.",
            Self::Source(_)=>"Il report originale non è più leggibile. Ripeti l’analisi prima di esportare.",
            Self::Spawn(_)=>"Impossibile avviare il salvataggio. Attendi la fine delle altre operazioni e riprova.",
            Self::Report(ReportError::Output{source,..}) if source.kind()==io::ErrorKind::PermissionDenied=>"Non hai i permessi per creare o sostituire il report nella cartella scelta.",
            Self::Report(_)=>"Impossibile completare il file. Verifica permessi e spazio nella cartella scelta.",
            Self::Replay(CanonicalReplayError::Report(_))=>"Scrittura del report non riuscita. Verifica permessi e spazio nella destinazione.",
            Self::Replay(_)=>"Il report sorgente è incompleto, danneggiato o non corrisponde al riepilogo. Ripeti l’analisi.",
            Self::Workbook(_)=>"Impossibile generare l’XLSX dal report. Verifica il report sorgente e lo spazio disponibile.",
            Self::WorkerUnavailable=>"Il salvataggio si è interrotto senza un esito completo. Verifica i file nella destinazione prima di riprovare.",
        })
    }
}
impl Error for ExportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(e) | Self::Source(e) | Self::Spawn(e) => Some(e),
            Self::Destination(e) => Some(e),
            Self::Report(e) => Some(e),
            Self::Replay(e) => Some(e),
            Self::Workbook(e) => Some(e),
            _ => None,
        }
    }
}
impl From<ReportDestinationError> for ExportError {
    fn from(error: ReportDestinationError) -> Self {
        match error {
            ReportDestinationError::Io(error) => Self::Io(error),
            other => Self::Destination(other),
        }
    }
}
/// Inspect typed causes, never guess from OS error strings or expose filenames.
fn nested_io_kind(error: &(dyn Error + 'static)) -> Option<io::ErrorKind> {
    let mut current = Some(error);
    while let Some(error) = current {
        if let Some(io) = error.downcast_ref::<io::Error>() {
            return Some(io.kind());
        }
        current = error.source();
    }
    None
}
/// Partial publication is explicit: two renames are not a filesystem transaction.
/// Display reports it, while technical causes/paths are not logged automatically.
pub struct ExportFailure {
    pub cause: ExportError,
    pub published: Vec<PathBuf>,
    pub cleanup: Vec<ReportError>,
    pub cancelled: bool,
}
impl fmt::Debug for ExportFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExportFailure")
            .field("cause", &self.cause.to_string())
            .field("published_count", &self.published.len())
            .field("cleanup_count", &self.cleanup.len())
            .finish()
    }
}
impl fmt::Display for ExportFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.cancelled {
            write!(
                f,
                "Salvataggio annullato prima della pubblicazione dei file."
            )?;
        } else {
            write!(f, "{}", self.cause)?;
        }
        if self.published.len() == 1 {
            write!(
                f,
                " Un file è stato salvato, ma l’esportazione non è completa. Controlla la destinazione."
            )?;
        } else if self.published.len() > 1 {
            write!(
                f,
                " {} file sono stati salvati, ma l’esportazione non è completa. Controlla la destinazione.",
                self.published.len()
            )?;
        }
        if !self.cleanup.is_empty() {
            write!(
                f,
                " Non è stato possibile eliminare tutti i file temporanei creati da questo salvataggio."
            )?;
        }
        Ok(())
    }
}
impl Error for ExportFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}
impl From<ExportError> for ExportFailure {
    fn from(cause: ExportError) -> Self {
        let cancelled = matches!(&cause, ExportError::Cancelled);
        Self {
            cause,
            published: vec![],
            cleanup: vec![],
            cancelled,
        }
    }
}
pub struct ExportReceipt {
    pub files: Vec<PathBuf>,
    pub destination: ReportDestinationReceipt,
}
impl fmt::Debug for ExportReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExportReceipt")
            .field("file_count", &self.files.len())
            .field("destination", &self.destination)
            .finish()
    }
}
pub fn summary_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".summary.json");
    PathBuf::from(name)
}
fn preflight(
    source: &ExportSource<'_>,
    path: &Path,
    format: ExportFormat,
    overwrite: bool,
) -> Result<Vec<PathBuf>, ExportError> {
    if !path.is_absolute() || path.file_name().is_none() {
        return Err(ExportError::InvalidPath);
    }
    if path.extension().and_then(|e| e.to_str()) != Some(format.extension()) {
        return Err(ExportError::InvalidExtension);
    }
    // Compare like representations on Windows too (extended-length prefixes).
    // Protect the entire private report root, including future sidecars.
    let protected_root = fs::canonicalize(source.private_root).map_err(ExportError::Io)?;
    // Each input is compared both as given and canonicalized, so neither a
    // relative/aliased spelling nor a missing file weakens the protection.
    let inputs: Vec<PathBuf> = source
        .protected_inputs
        .iter()
        .flat_map(|input| [Some(input.clone()), fs::canonicalize(input).ok()])
        .flatten()
        .collect();
    let paths = vec![path.to_owned(), summary_path(path)];
    for destination in &paths {
        let parent = destination.parent().ok_or(ExportError::InvalidPath)?;
        let parent = fs::canonicalize(parent).map_err(ExportError::Io)?;
        let normalized = parent.join(destination.file_name().ok_or(ExportError::InvalidPath)?);
        let canonical = match fs::symlink_metadata(destination) {
            Ok(metadata) => {
                if !metadata.is_file() || metadata.file_type().is_symlink() {
                    return Err(ExportError::NotRegularFile);
                }
                fs::canonicalize(destination).map_err(ExportError::Io)?
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => normalized,
            Err(e) => return Err(ExportError::Io(e)),
        };
        if inputs.contains(&canonical) || canonical.starts_with(&protected_root) {
            return Err(ExportError::ProtectedSource);
        }
        if destination.exists() && !overwrite {
            return Err(ExportError::OverwriteRequired);
        }
    }
    Ok(paths)
}
/// Cancellation uses a non-Interrupted error: read_until/write_all retry
/// Interrupted automatically and would otherwise spin forever after cancel.
struct CancelReader<R> {
    inner: R,
    token: CancellationToken,
}
impl<R: Read> Read for CancelReader<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if self.token.is_cancelled() {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "export cancelled",
            ));
        }
        self.inner.read(bytes)
    }
}
impl<R: BufRead> BufRead for CancelReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.token.is_cancelled() {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "export cancelled",
            ));
        }
        self.inner.fill_buf()
    }
    fn consume(&mut self, bytes: usize) {
        self.inner.consume(bytes);
    }
}
struct CancelWriter<'a> {
    inner: &'a mut (dyn Write + Send),
    token: &'a CancellationToken,
}
impl Write for CancelWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.token.is_cancelled() {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "export cancelled",
            ));
        }
        self.inner.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.token.is_cancelled() {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "export cancelled",
            ));
        }
        self.inner.flush()
    }
}
fn abort_all(mut failure: ExportFailure, guards: Vec<AtomicReportFile>) -> ExportFailure {
    for guard in guards {
        if let Err(cleanup) = guard.abort() {
            failure.cleanup.push(cleanup);
        }
    }
    failure
}
/// A successful result names both files. Before publication both remain staged
/// under the selected destination's OS permissions, distinct from private spool.
/// After the final cancellation checkpoint we finish publication; late cancel
/// cannot undo a file already renamed. External concurrent path replacement is
/// outside the existing AtomicReportFile contract; no stronger claim is added.
/// The two destinations an export would publish (report, then summary
/// sidecar), after the same structural preflight the export itself repeats:
/// absolute path, matching extension, existing parent, regular files only and
/// no protected source. Existing files are allowed here: whether to replace
/// them is the caller's explicit, separate overwrite choice.
pub fn export_destinations(
    source: &ExportSource<'_>,
    path: &Path,
    format: ExportFormat,
) -> Result<Vec<PathBuf>, ExportError> {
    preflight(source, path, format, true)
}

pub fn export_captured_report(
    source: &ExportSource<'_>,
    summary: &ValidationSummary,
    format: ExportFormat,
    path: &Path,
    overwrite: bool,
    cancellation: &CancellationToken,
) -> Result<ExportReceipt, ExportFailure> {
    export_captured_report_with_authorization(
        source,
        summary,
        format,
        path,
        overwrite,
        cancellation,
        None,
    )
}

/// The dialog can acknowledge an inspection returned by a previous attempt.
/// Local exports use the ordinary entry point with no extra privacy checkbox.
/// Both routes run the exact same OS policy and canonical writers in Rust.
pub fn export_captured_report_with_authorization(
    source: &ExportSource<'_>,
    summary: &ValidationSummary,
    format: ExportFormat,
    path: &Path,
    overwrite: bool,
    cancellation: &CancellationToken,
    authorization: Option<AuthorizedReportDestinations>,
) -> Result<ExportReceipt, ExportFailure> {
    if summary.run_status == RunStatus::Cancelled || summary.truncated {
        return Err(ExportError::IncompleteRun.into());
    }
    if cancellation.is_cancelled() {
        return Err(ExportError::Cancelled.into());
    }
    let paths = preflight(source, path, format, overwrite).map_err(ExportFailure::from)?;
    let permission = match authorization {
        Some(permission) => permission,
        None => ReportDestinationAssessment::inspect(&paths)
            .and_then(ReportDestinationAssessment::allow_local)
            .map_err(ExportError::from)
            .map_err(ExportFailure::from)?,
    };
    permission
        .verify_unchanged(&paths)
        .map_err(ExportError::from)
        .map_err(ExportFailure::from)?;
    if cancellation.is_cancelled() {
        return Err(ExportError::Cancelled.into());
    }
    let mut guards = Vec::new();
    for destination in &paths {
        match AtomicReportFile::create_replacing(destination, PartialOutputPolicy::Delete) {
            Ok(guard) => guards.push(guard),
            Err(e) => return Err(abort_all(ExportError::Report(e).into(), guards)),
        }
    }
    let write = (|| -> Result<(), ExportError> {
        let file = File::open(source.jsonl).map_err(ExportError::Source)?;
        let file = CancelReader {
            inner: BufReader::new(file),
            token: cancellation.clone(),
        };
        let writer = CancelWriter {
            inner: guards[0].writer_send(),
            token: cancellation,
        };
        match format {
            ExportFormat::Xlsx => {
                XlsxReportWriter::new()
                    .with_temp_dir(source.private_root.to_owned())
                    .write(file, summary, writer)
                    .map_err(ExportError::Workbook)?;
            }
            ExportFormat::Jsonl => {
                let mut sink = JsonlSink::new(BufWriter::new(writer));
                replay_canonical_report(file, summary, &mut sink, cancellation)
                    .map_err(ExportError::Replay)?;
                sink.finish().map_err(ExportError::Report)?;
            }
            ExportFormat::Csv => {
                let mut sink = CsvSink::new(BufWriter::new(writer)).map_err(ExportError::Report)?;
                replay_canonical_report(file, summary, &mut sink, cancellation)
                    .map_err(ExportError::Replay)?;
                sink.finish().map_err(ExportError::Report)?;
            }
        }
        write_run_summary_with_bundle(guards[1].writer(), summary, source.audit)
            .map(|_| ())
            .map_err(ExportError::Report)
    })();
    if let Err(error) = write {
        let mut failure: ExportFailure = error.into();
        failure.cancelled = cancellation.is_cancelled();
        return Err(abort_all(failure, guards));
    }
    if cancellation.is_cancelled() {
        return Err(abort_all(ExportError::Cancelled.into(), guards));
    }
    // Repeat the ordinary overwrite preflight immediately before publication;
    // hostile concurrent mutation is explicitly outside the core's contract.
    if let Err(error) = preflight(source, path, format, overwrite) {
        return Err(abort_all(error.into(), guards));
    }
    if let Err(error) = permission.verify_unchanged(&paths) {
        return Err(abort_all(ExportError::from(error).into(), guards));
    }
    let mut published = Vec::new();
    for destination in paths {
        let guard = guards.remove(0);
        match guard.complete(ReportCompletion {
            run_status: summary.run_status,
            truncated: false,
        }) {
            Ok(AtomicOutputOutcome::Published) => published.push(destination),
            Ok(AtomicOutputOutcome::Unpublished { .. }) => {
                return Err(abort_all(
                    ExportFailure {
                        cause: ExportError::IncompleteRun,
                        published,
                        cleanup: vec![],
                        cancelled: false,
                    },
                    guards,
                ));
            }
            Err(e) => {
                // Directory sync can fail after a successful rename. Preserve
                // the actual published-file receipt instead of claiming that
                // this destination was rolled back or never written.
                if e.output_was_published() {
                    published.push(destination);
                }
                return Err(abort_all(
                    ExportFailure {
                        cause: ExportError::Report(e),
                        published,
                        cleanup: vec![],
                        cancelled: false,
                    },
                    guards,
                ));
            }
        }
    }
    Ok(ExportReceipt {
        files: published,
        destination: permission.receipt(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Every input of a bundle is a protected destination, whatever its
    /// spelling; an unrelated file in the same folder is accepted.
    #[test]
    fn every_bundle_input_and_the_private_spool_are_protected_destinations() {
        let root = tempfile::tempdir().unwrap();
        let private = root.path().join("private");
        fs::create_dir(&private).unwrap();
        let inputs: Vec<PathBuf> = ["S.jsonl", "T.jsonl"]
            .iter()
            .map(|name| root.path().join(name))
            .collect();
        for input in &inputs {
            fs::write(input, b"input").unwrap();
        }
        let audit = BundleAudit::external(None);
        let jsonl = private.join("errors.jsonl");
        let source = ExportSource {
            jsonl: &jsonl,
            private_root: &private,
            protected_inputs: &inputs,
            audit: &audit,
        };
        for destination in [
            inputs[0].clone(),
            inputs[1].clone(),
            root.path().join("private/../T.jsonl"),
            private.join("copy.jsonl"),
        ] {
            assert!(
                matches!(
                    preflight(&source, &destination, ExportFormat::Jsonl, true),
                    Err(ExportError::ProtectedSource)
                ),
                "{destination:?}"
            );
        }
        let other = root.path().join("report.jsonl");
        assert_eq!(
            preflight(&source, &other, ExportFormat::Jsonl, false).unwrap(),
            [other.clone(), summary_path(&other)]
        );
    }
    /// Returning Interrupted here would cause standard read_until/write_all to
    /// retry forever. Check the kind before exercising those convenience loops.
    #[test]
    fn cancelled_stream_adapters_return_a_terminal_error_without_retry_spinning() {
        let token = CancellationToken::default();
        token.cancel();
        let mut bytes = Vec::new();
        let mut writer = CancelWriter {
            inner: &mut bytes,
            token: &token,
        };
        assert_eq!(
            writer.write(b"x").unwrap_err().kind(),
            io::ErrorKind::ConnectionAborted
        );
        assert!(writer.write_all(b"x").is_err());
        assert!(bytes.is_empty());
        let mut reader = CancelReader {
            inner: io::Cursor::new(b"row\n"),
            token,
        };
        assert_eq!(
            reader.fill_buf().unwrap_err().kind(),
            io::ErrorKind::ConnectionAborted
        );
        assert!(reader.read_until(b'\n', &mut Vec::new()).is_err());
    }
}
