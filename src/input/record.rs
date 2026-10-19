//! A borrowed raw record, with no decoding or validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawRecord<'a> {
    pub line_number: u64,
    pub bytes: &'a [u8],
}
