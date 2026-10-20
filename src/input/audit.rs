//! One authoritative audit state per session, frozen before metadata is returned.
use crate::domain::{
    InputAudit, InputAuditCompletion, InvalidInputAudit, RunMetadata, Sha256Digest, TraceId,
};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fmt,
    fs::File,
    io::{self, Read},
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum InputAuditError {
    Io(Arc<io::Error>),
    InvalidAudit(InvalidInputAudit),
    NonUtf8Basename,
    MissingBasename,
    NotRegularFile,
    SizeMismatch { expected: u64, actual: u64 },
    ByteCountOverflow,
    InvalidReadCount,
    MetadataAlreadyContainsInputs,
    ClosedSession,
    PoisonedSession,
}
impl fmt::Display for InputAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(_) => f.write_str("input audit source I/O failed"),
            Self::InvalidAudit(_) => f.write_str("invalid input audit metadata"),
            Self::NonUtf8Basename => f.write_str("input audit basename is not UTF-8"),
            Self::MissingBasename => f.write_str("input audit source has no basename"),
            Self::NotRegularFile => f.write_str("input audit source is not a regular file"),
            Self::SizeMismatch { expected, actual } => write!(
                f,
                "input audit size mismatch at EOF: expected {expected}, read {actual}"
            ),
            Self::ByteCountOverflow => f.write_str("input audit byte count exceeds u64::MAX"),
            Self::InvalidReadCount => {
                f.write_str("input source returned a count larger than the supplied buffer")
            }
            Self::MetadataAlreadyContainsInputs => {
                f.write_str("run metadata already contains independent input audits")
            }
            Self::ClosedSession => f.write_str("input audit session is finalized"),
            Self::PoisonedSession => f.write_str("input audit session lock is poisoned"),
        }
    }
}
impl Error for InputAuditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(e) => Some(e.as_ref()),
            Self::InvalidAudit(e) => Some(e),
            _ => None,
        }
    }
}
struct Slot {
    audit: InputAudit,
    hasher: Sha256,
    eof: bool,
}
#[derive(Default)]
struct State {
    slots: Vec<Slot>,
    failure: Option<InputAuditError>,
    closed: bool,
}
#[derive(Default)]
pub struct InputAuditSession {
    state: Arc<Mutex<State>>,
}
fn lock(state: &Mutex<State>) -> Result<MutexGuard<'_, State>, InputAuditError> {
    state.lock().map_err(|_| InputAuditError::PoisonedSession)
}
fn active(state: &State) -> Result<(), InputAuditError> {
    if state.closed {
        return Err(InputAuditError::ClosedSession);
    }
    if let Some(error) = &state.failure {
        return Err(error.clone());
    }
    Ok(())
}
impl InputAuditSession {
    pub fn new() -> Self {
        Self::default()
    }
    /// Register only a source actually selected by the caller, in selection order.
    pub fn wrap<R: Read>(
        &self,
        trace_id: TraceId,
        file_name: String,
        file_size_bytes: u64,
        reader: R,
    ) -> Result<AuditedReader<R>, InputAuditError> {
        let mut state = lock(&self.state)?;
        active(&state)?;
        let audit = InputAudit {
            trace_id,
            file_name,
            file_size_bytes,
            bytes_read: 0,
            completion: InputAuditCompletion::Incomplete,
        };
        if let Err(error) = audit.validate() {
            let failure = InputAuditError::InvalidAudit(error);
            state.failure = Some(failure.clone());
            return Err(failure);
        }
        let index = state.slots.len();
        state.slots.push(Slot {
            audit,
            hasher: Sha256::new(),
            eof: false,
        });
        Ok(AuditedReader {
            reader,
            state: Arc::clone(&self.state),
            index,
        })
    }
    /// Size is obtained from the opened handle, never a separate lookup of its path.
    pub fn open(
        &self,
        trace_id: TraceId,
        path: &Path,
    ) -> Result<AuditedReader<File>, InputAuditError> {
        let result = (|| {
            let name = path
                .file_name()
                .ok_or(InputAuditError::MissingBasename)?
                .to_str()
                .ok_or(InputAuditError::NonUtf8Basename)?
                .to_owned();
            let file = File::open(path).map_err(|e| InputAuditError::Io(Arc::new(e)))?;
            let metadata = file
                .metadata()
                .map_err(|e| InputAuditError::Io(Arc::new(e)))?;
            if !metadata.is_file() {
                return Err(InputAuditError::NotRegularFile);
            }
            self.wrap(trace_id, name, metadata.len(), file)
        })();
        if let Err(error) = &result
            && let Ok(mut state) = lock(&self.state)
            && !state.closed
            && state.failure.is_none()
        {
            state.failure = Some(error.clone());
        }
        result
    }
    /// Owned diagnostic snapshot without mutation access to authoritative state.
    pub fn snapshot(&self) -> Result<Vec<InputAudit>, InputAuditError> {
        let state = lock(&self.state)?;
        active(&state)?;
        Ok(state.slots.iter().map(|slot| slot.audit.clone()).collect())
    }
    /// Freezes all readers without reading or draining any source.
    /// Existing metadata must have no independently assembled audits.
    pub fn finish(self, mut metadata: RunMetadata) -> Result<RunMetadata, InputAuditError> {
        let mut state = lock(&self.state)?;
        active(&state)?;
        state.closed = true;
        if !metadata.inputs.is_empty() {
            return Err(InputAuditError::MetadataAlreadyContainsInputs);
        }
        metadata.inputs = state.slots.drain(..).map(|slot| slot.audit).collect();
        Ok(metadata)
    }
}

/// Place directly around the raw source, *before* BOM processing and buffering.
/// No Seek/get_mut/into_inner bypass is exposed. EOF freezes the fingerprint.
pub struct AuditedReader<R> {
    reader: R,
    state: Arc<Mutex<State>>,
    index: usize,
}
impl<R: Read> Read for AuditedReader<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let mut state = lock(&self.state).map_err(io::Error::other)?;
        active(&state).map_err(io::Error::other)?;
        if out.is_empty() || state.slots[self.index].eof {
            return Ok(0);
        }
        // Lock spans the read: finalization cannot freeze a snapshot mid-read.
        let count = match self.reader.read(out) {
            Ok(count) => count,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => return Err(e),
            Err(e) => {
                let kind = e.kind();
                let failure = InputAuditError::Io(Arc::new(e));
                state.failure = Some(failure.clone());
                return Err(io::Error::new(kind, failure));
            }
        };
        let slot = &mut state.slots[self.index];
        let failure = if count > out.len() {
            Some(InputAuditError::InvalidReadCount)
        } else if count == 0 {
            if slot.audit.bytes_read != slot.audit.file_size_bytes {
                Some(InputAuditError::SizeMismatch {
                    expected: slot.audit.file_size_bytes,
                    actual: slot.audit.bytes_read,
                })
            } else {
                let digest: [u8; 32] = slot.hasher.clone().finalize().into();
                slot.audit.completion = InputAuditCompletion::Complete {
                    sha256: Sha256Digest(digest),
                };
                slot.eof = true;
                None
            }
        } else if let Some(total) = u64::try_from(count)
            .ok()
            .and_then(|n| slot.audit.bytes_read.checked_add(n))
        {
            slot.hasher.update(&out[..count]);
            slot.audit.bytes_read = total;
            None
        } else {
            Some(InputAuditError::ByteCountOverflow)
        };
        if let Some(failure) = failure {
            state.failure = Some(failure.clone());
            return Err(io::Error::other(failure));
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn byte_counter_overflow_is_terminal_and_does_not_hash_an_unaccounted_chunk() {
        let session = InputAuditSession::new();
        let mut reader = session
            .wrap(
                TraceId("T".into()),
                "synthetic.txt".into(),
                u64::MAX,
                io::Cursor::new(b"X"),
            )
            .unwrap();
        session.state.lock().unwrap().slots[0].audit.bytes_read = u64::MAX;
        let error = reader.read(&mut [0]).unwrap_err();
        assert!(matches!(
            error.get_ref().unwrap().downcast_ref::<InputAuditError>(),
            Some(InputAuditError::ByteCountOverflow)
        ));
        assert!(matches!(
            session.snapshot(),
            Err(InputAuditError::ByteCountOverflow)
        ));
        let state = session.state.lock().unwrap();
        assert_eq!(state.slots[0].audit.bytes_read, u64::MAX);
        assert_eq!(
            state.slots[0].hasher.clone().finalize(),
            Sha256::new().finalize()
        );
    }
}
