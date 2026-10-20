//! Logical byte extraction only. Format 1 does not validate physical padding.
use crate::{
    input::record::FieldSlice,
    schema::compiled::{AlignmentRule, ContentBoundaryPolicy},
};
use std::{error::Error, fmt};

/// Technical inconsistency of a directly constructed plan, never invalid data.
/// The compiler already prevents opposite-direction stripping in trusted plans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalValueError {
    IncompatibleContentPolicy,
}
impl fmt::Display for LogicalValueError {
    /// A fixed message describes the bad plan without exposing input bytes.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("content policy is incompatible with padding direction")
    }
}
// No nested I/O/data error exists: the incompatible plan is the entire cause.
impl Error for LogicalValueError {}

/// Borrow the logical value without decoding, allocating or modifying raw data.
/// The output lifetime follows the input, not the rule: only bytes are borrowed.
/// All-fill becomes empty under stripping; preserve/exact keep every byte.
/// A leading blank in ` ROSSI ` remains part of the result under RightPad.
pub fn logical_value<'a>(
    rule: &AlignmentRule,
    value: FieldSlice<'a>,
) -> Result<FieldSlice<'a>, LogicalValueError> {
    use AlignmentRule::{Exact, LeftPad, RightPad};
    use ContentBoundaryPolicy::{PreserveWidth, StripLeadingFill, StripTrailingFill};
    let raw = value.as_bytes();
    let logical = match rule {
        Exact
        | RightPad {
            content_policy: PreserveWidth,
            ..
        }
        | LeftPad {
            content_policy: PreserveWidth,
            ..
        } => raw,
        RightPad {
            fill,
            content_policy: StripTrailingFill,
        } => {
            // The last non-fill byte belongs to the content. Its successor is
            // within 0..=len; no non-fill (including empty input) means prefix 0.
            let end = raw
                .iter()
                .rposition(|byte| byte != fill)
                .map_or(0, |index| index + 1);
            &raw[..end]
        }
        LeftPad {
            fill,
            content_policy: StripLeadingFill,
        } => {
            // Keep the first non-fill byte and everything after it. With only
            // fill, len selects a valid empty suffix rather than inventing 0.
            let start = raw
                .iter()
                .position(|byte| byte != fill)
                .unwrap_or(raw.len());
            &raw[start..]
        }
        RightPad {
            content_policy: StripLeadingFill,
            ..
        }
        | LeftPad {
            content_policy: StripTrailingFill,
            ..
        } => {
            return Err(LogicalValueError::IncompatibleContentPolicy);
        }
    };
    Ok(FieldSlice { raw: logical })
}
