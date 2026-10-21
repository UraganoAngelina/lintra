//! Bounded prefix preflight, with no seek, decoding, or implicit encoding selection.
use super::error::{BomKind, InputError};
use std::io::{self, Read};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BomPolicy {
    #[default]
    Reject,
    StripKnownBom,
}

/// Replays every non-BOM byte consumed during preflight before the remaining input.
#[derive(Debug)]
pub struct BomCheckedReader<R> {
    inner: R,
    prefix: [u8; 4],
    length: usize,
    offset: usize,
    detected: Option<BomKind>,
}
impl<R> BomCheckedReader<R> {
    pub fn detected_bom(&self) -> Option<BomKind> {
        self.detected
    }
}
impl<R: Read> Read for BomCheckedReader<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }
        if self.offset < self.length {
            let count = out.len().min(self.length - self.offset);
            out[..count].copy_from_slice(&self.prefix[self.offset..self.offset + count]);
            self.offset += count;
            return Ok(count);
        }
        self.inner.read(out)
    }
}

pub fn preflight_bom<R: Read>(
    mut inner: R,
    policy: BomPolicy,
) -> Result<BomCheckedReader<R>, InputError> {
    let mut prefix = [0; 4];
    let mut length = 0;
    while length < prefix.len() {
        match inner.read(&mut prefix[length..]) {
            Ok(0) => break,
            Ok(count) => length += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(InputError::Read(error)),
        }
    }
    // Longest signatures first: FF FE 00 00 must not be mistaken for UTF-16LE.
    let detected = [
        BomKind::Utf32Le,
        BomKind::Utf32Be,
        BomKind::Utf8,
        BomKind::Utf16Le,
        BomKind::Utf16Be,
    ]
    .into_iter()
    .find(|kind| prefix[..length].starts_with(kind.bytes()));
    if let (BomPolicy::Reject, Some(kind)) = (policy, detected) {
        return Err(InputError::BomRejected(kind));
    }
    let offset = detected.map_or(0, |kind| kind.bytes().len());
    Ok(BomCheckedReader {
        inner,
        prefix,
        length,
        offset,
        detected,
    })
}

/// DET §27 path input: the path is used as supplied, never silently decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputSource {
    Path(std::path::PathBuf),
}
