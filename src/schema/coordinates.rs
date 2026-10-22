//! Single one-based to zero-based conversion for source field and substring ranges.
use super::linter::SchemaIssueCode as Code;
use crate::domain::ByteRange;
pub(crate) fn source_range(start: u32, length: u32) -> Result<ByteRange, Code> {
    let start = start.checked_sub(1).ok_or(Code::InvalidPosition)?;
    let start = usize::try_from(start).map_err(|_| Code::RangeOverflow)?;
    let length = usize::try_from(length).map_err(|_| Code::RangeOverflow)?;
    ByteRange::new(start, length).map_err(|error| match error {
        crate::domain::ByteRangeError::ZeroLength => Code::ZeroLength,
        crate::domain::ByteRangeError::EndOverflow => Code::RangeOverflow,
    })
}
