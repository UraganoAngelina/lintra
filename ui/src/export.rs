//! Post-analysis exports own the original report while a bounded worker runs.
//! Formats are written by the core. No rules, CSV quoting or workbook layouts
//! are implemented here; each file is published by AtomicReportFile.
use crate::{analysis::ReportSelection, worker::AnalysisReport};
use lintra::*;
use std::{
    error::Error,
    fmt,
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
};

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
    pub fn preferred(reports: ReportSelection) -> Self {
        if reports.xlsx {
            Self::Xlsx
        } else if reports.csv {
            Self::Csv
        } else {
            Self::Jsonl
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
        if nested_io_kind(self) == Some(io::ErrorKind::StorageFull) {
            return f.write_str("Spazio insufficiente per il report o per i file temporanei. Libera spazio e riprova.");
        }
        f.write_str(match self {
            Self::InvalidPath=>"Inserisci il percorso completo del file in una cartella esistente.",
            Self::InvalidExtension=>"L’estensione del file deve corrispondere al formato scelto.",
            Self::ProtectedSource=>"Scegli un altro file: questa destinazione coincide con il file analizzato o con il report sorgente.",
            Self::NotRegularFile=>"La destinazione è una cartella o un collegamento. Scegli un file regolare.",
            Self::OverwriteRequired=>"Il report o il relativo riepilogo esiste già. Conferma la sostituzione oppure cambia nome.",
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
            Self::Report(e) => Some(e),
            Self::Replay(e) => Some(e),
            Self::Workbook(e) => Some(e),
            _ => None,
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
        if !self.published.is_empty() {
            write!(
                f,
                " Un file è stato salvato, ma l’esportazione non è completa. Controlla la destinazione."
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
}
impl fmt::Debug for ExportReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExportReceipt")
            .field("file_count", &self.files.len())
            .finish()
    }
}
pub fn summary_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".summary.json");
    PathBuf::from(name)
}
fn preflight(
    report: &AnalysisReport,
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
        if canonical == report.input_path()
            || canonical == report.jsonl_path()
            || canonical.starts_with(report.directory())
        {
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
/// A successful result names both files. Before publication both remain private.
/// After the final cancellation checkpoint we finish publication; late cancel
/// cannot undo a file already renamed. External concurrent path replacement is
/// outside the existing AtomicReportFile contract; no stronger claim is added.
pub fn export_report(
    report: &AnalysisReport,
    summary: &ValidationSummary,
    format: ExportFormat,
    path: &Path,
    overwrite: bool,
    cancellation: &CancellationToken,
) -> Result<ExportReceipt, ExportFailure> {
    if summary.run_status == RunStatus::Cancelled || summary.truncated {
        return Err(ExportError::IncompleteRun.into());
    }
    if cancellation.is_cancelled() {
        return Err(ExportError::Cancelled.into());
    }
    let paths = preflight(report, path, format, overwrite).map_err(ExportFailure::from)?;
    let mut guards = Vec::new();
    for destination in &paths {
        match AtomicReportFile::create_replacing(destination, PartialOutputPolicy::Delete) {
            Ok(guard) => guards.push(guard),
            Err(e) => return Err(abort_all(ExportError::Report(e).into(), guards)),
        }
    }
    let write = (|| -> Result<(), ExportError> {
        let file = File::open(report.jsonl_path()).map_err(ExportError::Source)?;
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
                    .with_temp_dir(report.directory().to_owned())
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
        write_run_summary_with_bundle(guards[1].writer(), summary, report.audit())
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
    if let Err(error) = preflight(report, path, format, overwrite) {
        return Err(abort_all(error.into(), guards));
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
    Ok(ExportReceipt { files: published })
}
/// One terminal slot, owned join handle and cooperative token. The report Arc
/// keeps the JSONL/temp directory alive across a new validation or UI selection.
pub struct ExportJob {
    receiver: Receiver<Result<ExportReceipt, ExportFailure>>,
    token: CancellationToken,
    thread: Option<JoinHandle<()>>,
}
impl ExportJob {
    pub fn start(
        report: Arc<AnalysisReport>,
        summary: ValidationSummary,
        format: ExportFormat,
        path: PathBuf,
        overwrite: bool,
    ) -> Result<Self, ExportError> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let token = CancellationToken::default();
        let cancellation = token.clone();
        let thread = thread::Builder::new()
            .name("lintra-export".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    export_report(&report, &summary, format, &path, overwrite, &cancellation)
                }))
                .unwrap_or_else(|_| Err(ExportError::WorkerUnavailable.into()));
                let _ = sender.try_send(result);
            })
            .map_err(ExportError::Spawn)?;
        Ok(Self {
            receiver,
            token,
            thread: Some(thread),
        })
    }
    pub fn cancel(&self) {
        self.token.cancel();
    }
    pub fn try_result(&mut self) -> Option<Result<ExportReceipt, ExportFailure>> {
        match self.receiver.try_recv() {
            Ok(result) => {
                if let Some(thread) = self.thread.take() {
                    let _ = thread.join();
                }
                Some(result)
            }
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err(ExportError::WorkerUnavailable.into()))
            }
        }
    }
}
impl Drop for ExportJob {
    fn drop(&mut self) {
        self.token.cancel();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
