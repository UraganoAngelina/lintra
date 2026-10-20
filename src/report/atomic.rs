//! Per-file publication: the destination changes only after successful writing.
//! This layer owns filesystem lifecycle, while JSONL/summary encoders own format.
use super::sink::ReportError;
use crate::domain::RunStatus;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

/// Cleanup defaults to deletion. Retention requires explicit diagnostic intent;
/// retained files keep their `.part` suffix and are never definitive reports.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PartialOutputPolicy {
    #[default]
    Delete,
    RetainForDiagnostics,
}

/// The producer returns this only after its own processing has succeeded.
/// Normative invalidity is a completed analysis; cancellation/truncation is not.
#[derive(Clone, Copy, Debug)]
pub struct ReportCompletion {
    pub run_status: RunStatus,
    pub truncated: bool,
}

/// Controlled interruption is an ordinary outcome, not a technical error.
/// Paths are local filesystem control data, never serialized into report JSON.
#[derive(Debug, PartialEq, Eq)]
pub enum AtomicOutputOutcome {
    Published,
    Unpublished { retained_part: Option<PathBuf> },
}

/// Identifies the failed filesystem operation without exposing paths in Display.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputOperation {
    Create,
    Flush,
    Sync,
    Publish,
}

/// Write one report through a same-directory `.part`, then close and rename it.
///
/// An existing destination is deliberately replaced only on successful publish.
/// An existing `.part` is never truncated or removed: exclusive creation fails.
/// The callback must propagate all producer/input/sink errors; publication is
/// possible only when it returns Ok with a completed, non-truncated run.
/// This convenience function is for report producers. Applications with their
/// own typed input errors can use AtomicReportFile and abort explicitly instead
/// of translating an input failure into a report failure.
///
/// Each call is atomic for one file; separate reports are not a transaction.
/// No parent directory is created. The directory must be caller-controlled:
/// concurrent external replacement of paths is outside this contract.
///
/// The callback finalizes the JSONL encoder before requesting publication, so
/// both serialization and flush failures propagate through `?` into cleanup:
/// ```no_run
/// use lintra::{write_atomic_report, AtomicOutputOutcome, JsonlSink,
///     PartialOutputPolicy, ReportCompletion, ReportError, RunStatus};
/// fn empty_report(path: &std::path::Path) -> Result<AtomicOutputOutcome, ReportError> {
///     write_atomic_report(path, PartialOutputPolicy::Delete, |writer| {
///         JsonlSink::new(writer).finish()?;
///         Ok(ReportCompletion { run_status: RunStatus::CompletedValid, truncated: false })
///     })
/// }
/// ```
pub fn write_atomic_report<F>(
    destination: &Path,
    policy: PartialOutputPolicy,
    produce: F,
) -> Result<AtomicOutputOutcome, ReportError>
where
    F: FnOnce(&mut dyn Write) -> Result<ReportCompletion, ReportError>,
{
    let mut pending = AtomicReportFile::create_replacing(destination, policy)?;
    // Borrowing the writer prevents the producer from publishing or taking the
    // file. Report errors use explicit cleanup so cleanup failures are visible.
    match produce(pending.writer()) {
        Ok(completion) => pending.complete(completion),
        Err(error) => pending.discard(Some(error)),
    }
}

/// RAII guard for callers with their own typed upstream errors. Only this
/// object owns cleanup of the file it created. Call abort on upstream failure;
/// preserve that original error in the application, without relabeling it.
/// Option<File> permits explicit close before Windows rename/removal, and the
/// ownership bit distinguishes a pending file from an already published one.
pub struct AtomicReportFile {
    file: Option<File>,
    destination: PathBuf,
    part: PathBuf,
    policy: PartialOutputPolicy,
    owns_part: bool,
}

impl AtomicReportFile {
    /// Append the suffix to the native filename (no lossy UTF-8 conversion),
    /// keeping source and destination on the same filesystem. create_new also
    /// rejects an existing symlink, so a stale part cannot be followed/truncated.
    pub fn create_replacing(
        destination: &Path,
        policy: PartialOutputPolicy,
    ) -> Result<Self, ReportError> {
        let mut name = destination
            .file_name()
            .ok_or_else(|| ReportError::Output {
                operation: OutputOperation::Create,
                source: io::Error::new(io::ErrorKind::InvalidInput, "output needs a filename"),
            })?
            .to_os_string();
        name.push(".part");
        let part = destination.with_file_name(name);
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&part)
            .map_err(|source| ReportError::Output {
                operation: OutputOperation::Create,
                source,
            })?;
        Ok(Self {
            file: Some(file),
            destination: destination.to_owned(),
            part,
            policy,
            owns_part: true,
        })
    }

    /// Borrow the output without releasing ownership. JSONL/summary writers can
    /// use this streaming interface; errors must be propagated before complete.
    pub fn writer(&mut self) -> &mut dyn Write {
        self.file.as_mut().expect("owned open output")
    }

    /// Consume the guard to prevent any writes after publication. Only complete,
    /// non-truncated analyses can replace the destination; cancellation is a
    /// controlled unpublished outcome. No reads or extra validation take place.
    pub fn complete(
        mut self,
        completion: ReportCompletion,
    ) -> Result<AtomicOutputOutcome, ReportError> {
        // CompletedInvalid still has a complete report. Cancelled or truncated
        // runs must never replace the definitive destination, even with valid JSON.
        if completion.run_status == RunStatus::Cancelled || completion.truncated {
            return self.discard(None);
        }
        // File::flush has no user-space buffering, but checking it and sync_all
        // detects filesystem writeback errors before we expose the new filename.
        let file = self.file.as_mut().expect("owned open output");
        if let Err(source) = file.flush() {
            return self.discard(Some(ReportError::Output {
                operation: OutputOperation::Flush,
                source,
            }));
        }
        if let Err(source) = file.sync_all() {
            return self.discard(Some(ReportError::Output {
                operation: OutputOperation::Sync,
                source,
            }));
        }
        // Close our handle before rename: Windows sharing restrictions can reject
        // renaming an open file. Never delete the old destination as a workaround.
        self.file.take();
        if let Err(source) = fs::rename(&self.part, &self.destination) {
            return self.discard(Some(ReportError::Output {
                operation: OutputOperation::Publish,
                source,
            }));
        }
        // Ownership ends only after rename succeeded. Drop must not unlink a path
        // that could subsequently be reused by another writer.
        self.owns_part = false;
        Ok(AtomicOutputOutcome::Published)
    }

    /// Explicitly abort for an upstream technical failure. This closes the file
    /// and reports cleanup errors without coercing the upstream error type.
    pub fn abort(mut self) -> Result<AtomicOutputOutcome, ReportError> {
        self.discard(None)
    }

    /// Close first, then retain or remove only our pending file. A cleanup
    /// failure preserves both its own I/O cause and the initial technical error;
    /// it cannot silently turn an unsuccessful cleanup into a successful run.
    fn discard(
        &mut self,
        original: Option<ReportError>,
    ) -> Result<AtomicOutputOutcome, ReportError> {
        self.file.take();
        let retained_part = if self.policy == PartialOutputPolicy::RetainForDiagnostics {
            Some(self.part.clone())
        } else {
            if let Err(source) = fs::remove_file(&self.part) {
                return Err(ReportError::OutputCleanup {
                    original: original.map(Box::new),
                    source,
                });
            }
            None
        };
        self.owns_part = false;
        match original {
            Some(error) => Err(error),
            None => Ok(AtomicOutputOutcome::Unpublished { retained_part }),
        }
    }
}

impl Drop for AtomicReportFile {
    /// Fallback for panic/unwinding or interrupted control flow. Drop cannot
    /// return an error, so ordinary errors always go through discard instead.
    /// Abrupt process termination cannot run Drop and may leave a stale part.
    fn drop(&mut self) {
        self.file.take();
        if self.owns_part && self.policy == PartialOutputPolicy::Delete {
            let _ = fs::remove_file(&self.part);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Closing the private handle and externally deleting the part injects a
    /// real cleanup failure deterministically on both OSes, without permissions
    /// assumptions (tests can run as root). This is not a public API capability.
    fn fail_cleanup(original: Option<ReportError>) -> ReportError {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "lintra cleanup {} {stamp} {}",
            std::process::id(),
            original.is_some()
        ));
        fs::create_dir(&directory).unwrap();
        let mut pending = AtomicReportFile::create_replacing(
            &directory.join("report"),
            PartialOutputPolicy::Delete,
        )
        .unwrap();
        pending.file.take();
        fs::remove_file(&pending.part).unwrap();
        let error = pending.discard(original).unwrap_err();
        drop(pending);
        fs::remove_dir(directory).unwrap();
        error
    }

    /// The initial serialization failure must remain inspectable even when
    /// removal independently fails; the cleanup cause is also preserved.
    #[test]
    fn cleanup_failure_keeps_original_error_and_os_cause() {
        let error = fail_cleanup(Some(ReportError::InvalidTimestamp));
        assert!(matches!(
            error,
            ReportError::OutputCleanup { original: Some(original), source }
            if matches!(*original, ReportError::InvalidTimestamp)
                && source.kind() == io::ErrorKind::NotFound
        ));
    }

    /// A cancellation has no technical cause initially, but failed cleanup is
    /// still an error, never an incorrectly successful Unpublished outcome.
    #[test]
    fn cleanup_failure_during_cancellation_is_not_silent_success() {
        assert!(matches!(
            fail_cleanup(None),
            ReportError::OutputCleanup { original: None, source }
            if source.kind() == io::ErrorKind::NotFound
        ));
    }
}
