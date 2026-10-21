//! DET §97: normative invalidity and technical failure have distinct exit codes.
use lintra::RunStatus;
pub const VALID: u8 = from_status(RunStatus::CompletedValid);
pub const TECHNICAL: u8 = 1;

/// Exhaustive mapping: adding a controlled outcome requires reviewing the CLI.
pub const fn from_status(status: RunStatus) -> u8 {
    match status {
        RunStatus::CompletedValid => 0,
        RunStatus::CompletedInvalid => 2,
        RunStatus::Cancelled => 130,
    }
}
/// Help is successful; parser failures are technical, regardless of clap's own
/// default code 2 (reserved here for completed normative invalidity).
pub fn from_parse_error(error: &clap::Error) -> u8 {
    if error.use_stderr() { TECHNICAL } else { VALID }
}
