//! Lower the already-normalized authoring policy without discarding conditions.
use super::{
    compiled::{BasePresence, CompiledPresencePolicy},
    expression::compile_local_expr,
    linter::SchemaIssueCode,
    raw::{RawPresence, RawPresenceBase, RawRecordSchema},
};

/// RequiredUnless is authored as required_if Not(E), per ratified CNF-10.
/// No additional raw spelling, default or mutable runtime flag is introduced.
pub(crate) fn compile_presence(
    policy: &RawPresence,
    record: &RawRecordSchema,
) -> Result<CompiledPresencePolicy, (SchemaIssueCode, String)> {
    Ok(CompiledPresencePolicy {
        base: match policy.base {
            RawPresenceBase::Required => BasePresence::Required,
            RawPresenceBase::Optional => BasePresence::Optional,
        },
        required_if: policy
            .required_if
            .iter()
            .map(|expr| compile_local_expr(expr, record))
            .collect::<Result<_, _>>()?,
        forbidden_if: policy
            .forbidden_if
            .iter()
            .map(|expr| compile_local_expr(expr, record))
            .collect::<Result<_, _>>()?,
    })
}
