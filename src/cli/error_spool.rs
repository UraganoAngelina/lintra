//! One canonical error stream: requested atomic JSONL or secure temporary file.
use lintra::{AtomicReportFile, PartialOutputPolicy, ReportCompletion, ReportError};
use std::{
    error::Error,
    fs::File,
    io::{self, Write},
    path::Path,
};

pub(super) enum ErrorSpool {
    Report(AtomicReportFile),
    Temporary(tempfile::NamedTempFile),
}
impl ErrorSpool {
    pub fn create(destination: Option<&Path>) -> Result<Self, Box<dyn Error>> {
        Ok(match destination {
            Some(path) => Self::Report(AtomicReportFile::create_replacing(
                path,
                PartialOutputPolicy::Delete,
            )?),
            None => Self::Temporary(tempfile::NamedTempFile::new()?),
        })
    }
    pub fn writer(&mut self) -> &mut dyn Write {
        match self {
            Self::Report(report) => report.writer(),
            Self::Temporary(file) => file.as_file_mut(),
        }
    }
    /// Reopen rather than clone: the reader starts at zero and does not share
    /// the write cursor. NamedTempFile::reopen also checks the file identity.
    pub fn reader(&mut self) -> Result<File, Box<dyn Error>> {
        Ok(match self {
            Self::Report(report) => report.open_reader()?,
            Self::Temporary(file) => {
                file.as_file_mut().flush()?;
                file.reopen()?
            }
        })
    }
    pub fn abort(self) -> Result<(), ReportError> {
        match self {
            Self::Report(report) => report.abort().map(|_| ()),
            Self::Temporary(file) => file.close().map_err(cleanup),
        }
    }
    /// The temporary is closed/deleted before any final XLSX publication. A
    /// failed cleanup remains technical, not a successful export with a leak.
    pub fn complete(self, completion: ReportCompletion) -> Result<(), ReportError> {
        match self {
            Self::Report(report) => report.complete(completion).map(|_| ()),
            Self::Temporary(file) => file.close().map_err(cleanup),
        }
    }
}
fn cleanup(source: io::Error) -> ReportError {
    ReportError::OutputCleanup {
        original: None,
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn temporary_spool_reopens_from_zero_and_explicitly_removes_its_file() {
        for abort in [false, true] {
            let mut spool = ErrorSpool::create(None).unwrap();
            let ErrorSpool::Temporary(file) = &spool else {
                panic!("expected temporary spool")
            };
            let path = file.path().to_owned();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(
                    file.as_file().metadata().unwrap().permissions().mode() & 0o777,
                    0o600
                );
            }
            spool.writer().write_all(b"synthetic\n").unwrap();
            for _ in 0..2 {
                let mut text = String::new();
                spool.reader().unwrap().read_to_string(&mut text).unwrap();
                assert_eq!(text, "synthetic\n");
            }
            if abort {
                spool.abort().unwrap();
            } else {
                spool
                    .complete(ReportCompletion {
                        run_status: lintra::RunStatus::CompletedValid,
                        truncated: false,
                    })
                    .unwrap();
            }
            assert!(!path.exists());
        }
    }
}
