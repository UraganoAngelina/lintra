use crate::schema::compiled::MissingValuePolicy;

/// Tests the unmodified bytes against the schema's explicit missing policy.
/// Field width is checked by the record pipeline, not inferred here.
pub fn is_missing(policy: &MissingValuePolicy, raw: &[u8]) -> bool {
    match policy {
        MissingValuePolicy::Spaces => raw.iter().all(|&byte| byte == b' '),
        MissingValuePolicy::Exact(value) => raw == value,
        MissingValuePolicy::AnyOf(values) => values.iter().any(|value| raw == value),
        MissingValuePolicy::NeverMissing => false,
    }
}
