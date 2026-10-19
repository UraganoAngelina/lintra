//! Checked zero-based, end-exclusive byte ranges. No normative position conversion.

use std::error::Error;
use std::fmt;

/// A nonempty byte range whose exclusive end fits in `usize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    start: usize,
    len: usize,
}

/// Failure to construct a valid byte range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteRangeError {
    ZeroLength,
    EndOverflow,
}

impl fmt::Display for ByteRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ZeroLength => "byte range length must be greater than zero",
            Self::EndOverflow => "byte range end exceeds usize::MAX",
        })
    }
}

impl Error for ByteRangeError {}

impl ByteRange {
    /// Rejects zero length and overflow before constructing the range.
    pub fn new(start: usize, len: usize) -> Result<Self, ByteRangeError> {
        if len == 0 {
            return Err(ByteRangeError::ZeroLength);
        }
        start.checked_add(len).ok_or(ByteRangeError::EndOverflow)?;
        Ok(Self { start, len })
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Always false: construction rejects empty ranges.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn end_exclusive(&self) -> usize {
        // Private fields and the checked constructor guarantee this sum fits.
        self.start + self.len
    }

    /// Borrows the original bytes, or returns `None` when outside the record.
    pub fn slice<'a>(&self, bytes: &'a [u8]) -> Option<&'a [u8]> {
        bytes.get(self.start..self.end_exclusive())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slices_at_byte_offsets_without_copying_or_decoding() {
        let bytes = [b'A', 0xc3, 0xa8, b' '];
        let range = ByteRange::new(1, 2).unwrap();
        assert_eq!(
            (range.start(), range.len(), range.end_exclusive()),
            (1, 2, 3)
        );
        assert!(!range.is_empty());
        let slice = range.slice(&bytes).unwrap();
        assert_eq!(slice, &[0xc3, 0xa8]);
        assert_eq!(slice.as_ptr(), bytes[1..].as_ptr());
        // A single byte can cut a UTF-8 sequence: this primitive is byte-oriented.
        assert_eq!(
            ByteRange::new(2, 1).unwrap().slice(&bytes),
            Some(&bytes[2..3])
        );
    }

    #[test]
    fn rejects_zero_length_and_end_overflow() {
        assert_eq!(ByteRange::new(0, 0), Err(ByteRangeError::ZeroLength));
        assert_eq!(
            ByteRange::new(usize::MAX, 0),
            Err(ByteRangeError::ZeroLength)
        );
        assert_eq!(
            ByteRange::new(usize::MAX, 1),
            Err(ByteRangeError::EndOverflow)
        );
        assert_eq!(
            ByteRange::new(1, usize::MAX),
            Err(ByteRangeError::EndOverflow)
        );
    }

    #[test]
    fn exact_end_is_valid_and_outside_record_is_none() {
        let bytes = b"ABC ";
        assert_eq!(ByteRange::new(0, 4).unwrap().slice(bytes), Some(&bytes[..]));
        assert_eq!(
            ByteRange::new(3, 1).unwrap().slice(bytes),
            Some(&bytes[3..])
        );
        for (start, len) in [(0, 5), (4, 1), (8, 1), (usize::MAX - 1, 1), (0, usize::MAX)] {
            let range = ByteRange::new(start, len).unwrap();
            assert_eq!(range.slice(bytes), None);
            assert_eq!(range.slice(&[]), None);
        }
        assert_eq!(
            ByteRange::new(usize::MAX - 1, 1).unwrap().end_exclusive(),
            usize::MAX
        );
    }

    #[test]
    fn boundary_grid_agrees_with_checked_slice_bounds() {
        let bytes = [0u8; 8];
        let edges = [0, 1, 2, 7, 8, 9, usize::MAX - 1, usize::MAX];
        for record_len in 0..=bytes.len() {
            let record = &bytes[..record_len];
            for start in edges {
                for len in edges {
                    match ByteRange::new(start, len) {
                        Ok(range) => {
                            let end = start.checked_add(len).unwrap();
                            assert_eq!(range.slice(record), record.get(start..end));
                            assert_eq!(range.slice(record).is_some(), end <= record_len);
                        }
                        Err(error) => assert!(
                            (len == 0 && error == ByteRangeError::ZeroLength)
                                || (len > 0
                                    && start.checked_add(len).is_none()
                                    && error == ByteRangeError::EndOverflow)
                        ),
                    }
                }
            }
        }
    }
}
