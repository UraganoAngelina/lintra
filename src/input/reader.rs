use super::{error::InputError, record::RawRecord};
use std::io::BufRead;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewlinePolicy {
    AutoLfOrCrlf,
    LfOnly,
    CrlfOnly,
}

/// Streams records into the caller's reusable buffer; retains no borrowed record.
/// A transport failure is terminal, as read_until may already have consumed bytes.
#[derive(Debug)]
pub struct RecordReader<R> {
    inner: R,
    line_number: u64,
    newline_policy: NewlinePolicy,
    failed: bool,
}
impl<R: BufRead> RecordReader<R> {
    pub fn new(inner: R, newline_policy: NewlinePolicy) -> Self {
        Self {
            inner,
            line_number: 0,
            newline_policy,
            failed: false,
        }
    }
    pub fn line_number(&self) -> u64 {
        self.line_number
    }

    pub fn read_next<'a>(
        &mut self,
        buffer: &'a mut Vec<u8>,
    ) -> Result<Option<RawRecord<'a>>, InputError> {
        buffer.clear();
        if self.failed {
            return Err(InputError::PreviousFailure);
        }
        loop {
            let count = match self.inner.read_until(b'\n', buffer) {
                Ok(count) => count,
                Err(source) => {
                    self.failed = true;
                    return Err(InputError::RecordRead {
                        last_completed_line: self.line_number,
                        source,
                    });
                }
            };
            if count == 0 {
                if buffer.is_empty() {
                    return Ok(None);
                }
                break;
            }
            match self.newline_policy {
                NewlinePolicy::AutoLfOrCrlf => {
                    if buffer.last() == Some(&b'\n') {
                        buffer.pop();
                        if buffer.last() == Some(&b'\r') {
                            buffer.pop();
                        }
                    }
                    break;
                }
                NewlinePolicy::LfOnly => {
                    if buffer.last() == Some(&b'\n') {
                        buffer.pop();
                    }
                    break;
                }
                NewlinePolicy::CrlfOnly => {
                    if buffer.ends_with(b"\r\n") {
                        buffer.truncate(buffer.len() - 2);
                        break;
                    }
                    // Bare LF is data under this framing policy, not a CRLF delimiter.
                    if buffer.last() != Some(&b'\n') {
                        break;
                    }
                }
            }
        }
        let Some(line_number) = self.line_number.checked_add(1) else {
            self.failed = true;
            return Err(InputError::LineNumberOverflow);
        };
        self.line_number = line_number;
        Ok(Some(RawRecord {
            line_number,
            bytes: buffer.as_slice(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn counter_overflow_is_checked_but_clean_eof_never_increments_it() {
        let mut reader = RecordReader::new(Cursor::new(b"A\n"), NewlinePolicy::AutoLfOrCrlf);
        reader.line_number = u64::MAX;
        let mut buffer = Vec::new();
        assert!(matches!(
            reader.read_next(&mut buffer),
            Err(InputError::LineNumberOverflow)
        ));
        assert_eq!(reader.line_number(), u64::MAX);
        assert!(matches!(
            reader.read_next(&mut buffer),
            Err(InputError::PreviousFailure)
        ));
        let mut eof = RecordReader::new(Cursor::new(b""), NewlinePolicy::AutoLfOrCrlf);
        eof.line_number = u64::MAX;
        assert_eq!(eof.read_next(&mut buffer).unwrap(), None);
        assert_eq!(eof.line_number(), u64::MAX);
    }
}
