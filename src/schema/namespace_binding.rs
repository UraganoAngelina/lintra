//! Checked format-3 bindings independent of any open persistence repository.
use crate::schema::{compiled::NamespaceBindingPlan, linter::SchemaIssueCode as Code, raw::*};
use std::collections::HashSet;

pub(crate) fn compile(raw: &RawFlowSchema) -> Result<Vec<NamespaceBindingPlan>, (Code, String)> {
    let fail = |message: &str| (Code::InvalidSchemaValue, message.to_owned());
    let capability = raw
        .required_capabilities
        .contains(&RawFlowSchemaRequiredCapabilitiesItem::PersistenceNamespaceBindingV1);
    let entries = raw
        .persistence_policy
        .as_ref()
        .and_then(|p| p.namespace_bindings.as_ref());
    if (capability || entries.is_some()) && raw.schema_format_version != 3 {
        return Err(fail("namespace bindings require format 3"));
    }
    if entries.is_some_and(Vec::is_empty) || capability != entries.is_some_and(|v| !v.is_empty()) {
        return Err(fail(
            "namespace bindings require a nonempty declaration and matching capability",
        ));
    }
    let Some(entries) = entries else {
        return Ok(vec![]);
    };
    let policy = raw
        .persistence_policy
        .as_ref()
        .ok_or_else(|| fail("missing namespace policy"))?;
    let mut seen = HashSet::new();
    entries
        .iter()
        .map(|entry| {
            let trace_index = raw
                .traces
                .iter()
                .position(|t| t.id == entry.trace)
                .ok_or_else(|| fail("namespace binding references an unknown trace"))?;
            let field_index = raw.traces[trace_index]
                .fields
                .iter()
                .position(|f| f.id == entry.field)
                .ok_or_else(|| fail("namespace binding references an unknown field"))?;
            let parameter_index = raw
                .parameters
                .iter()
                .position(|p| {
                    matches!(p,
            RawParameter::String { id, .. } if id == &entry.parameter)
                })
                .ok_or_else(|| fail("namespace binding requires a declared string parameter"))?;
            if !policy.namespace_parameters.contains(&entry.parameter)
                || !seen.insert((trace_index, field_index))
            {
                return Err(fail(
                    "namespace binding must target a namespace parameter and a unique field",
                ));
            }
            // Numeric lexical fields may represent exact textual identifiers. Dates
            // and monetary Decimal carry other semantics and cannot be rebound as text.
            if !matches!(
                raw.traces[trace_index].fields[field_index].physical,
                RawPhysical::Numeric { .. } | RawPhysical::Alphanumeric { .. }
            ) {
                return Err(fail(
                    "namespace binding field is incompatible with exact text comparison",
                ));
            }
            let error_code = entry
                .error_code
                .parse()
                .map_err(|_| fail("invalid namespace diagnostic code"))?;
            if entry.message_template.is_empty()
                || entry.message_template.trim() != entry.message_template
                || entry.message_template.contains(['{', '}'])
            {
                return Err(fail(
                    "namespace diagnostic requires a nonempty literal message template",
                ));
            }
            Ok(NamespaceBindingPlan {
                trace_index,
                field_index,
                parameter_index,
                error_code,
                message_template: entry.message_template.as_str().into(),
            })
        })
        .collect()
}

/// Ordinary validation may execute binding-only policy declarations. Historical
/// audit/snapshot/temporal obligations retain their explicit compiler gates.
pub(crate) fn binding_only(raw: &RawFlowSchema) -> bool {
    raw.persistence_policy.as_ref().is_some_and(|p| {
        p.namespace_bindings.as_ref().is_some_and(|v| !v.is_empty())
            && p.cross_run_audit.is_empty()
            && p.history_identity_invariants.is_empty()
            && p.snapshot.is_none()
            && p.temporal_rules.is_empty()
    })
}
