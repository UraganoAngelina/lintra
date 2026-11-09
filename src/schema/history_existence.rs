//! Checked historical set predicates. This module lowers metadata only: the
//! ordinary compiler and engine still require an explicit historical producer.
use super::*;
use crate::schema::rule_dependencies::RuleDependencies;
use sha2::{Digest, Sha256};
type Issue = (Code, String);
fn unsupported(message: &str) -> Issue {
    (Code::UnsupportedConstruct, message.into())
}

pub(super) fn compile(
    raw: &RawFlowSchema,
    traces: &[CompiledRecordSchema],
    joins: &[CompiledJoinSpec],
    rules: &[CompiledCrossRecordRule],
) -> Result<Vec<HistoryExistencePlan>, Issue> {
    let historical = joins
        .iter()
        .filter(|j| j.scope == JoinScope::HistoryIncludingCurrent)
        .count();
    let declared = raw
        .required_capabilities
        .contains(&RawFlowSchemaRequiredCapabilitiesItem::HistoryExistenceV1);
    if declared != (historical != 0) {
        return Err((
            Code::MissingCapability,
            "history capability and actual historical joins must agree".into(),
        ));
    }
    if historical == 0 {
        return Ok(Vec::new());
    }
    if raw.schema_format_version != 3 {
        return Err(unsupported(
            "historical existence requires explicit schema format3",
        ));
    }
    let policy = raw
        .persistence_policy
        .as_ref()
        .filter(|p| p.snapshot.is_some())
        .ok_or_else(|| unsupported("historical existence requires a checked snapshot policy"))?;
    let hash = crate::schema::canonical::schema_fingerprint(raw)
        .map_err(|_| unsupported("historical schema identity could not be encoded"))?;
    let tables = traces
        .iter()
        .map(|t| t.derived_values().to_vec())
        .collect::<Vec<_>>();
    let mut plans: BTreeMap<usize, HistoryExistencePlan> = BTreeMap::new();
    for cross in rules {
        for assertion in &cross.rule.assertions {
            let isolated = CompiledRule {
                when: CompiledExpr::Const(true),
                assertions: vec![assertion.clone()],
                ..cross.rule.clone()
            };
            let deps = RuleDependencies::collect(&isolated, cross.source_trace, joins, &tables)?;
            if !deps
                .joins
                .iter()
                .any(|i| joins[*i].scope == JoinScope::HistoryIncludingCurrent)
            {
                continue;
            }
            let CompiledAssertion::SetExists {
                join_index,
                predicate,
            } = assertion
            else {
                return Err(unsupported(
                    "historical joins support only set_exists, without scalar or temporal reduction",
                ));
            };
            let join = &joins[*join_index];
            if join.scope != JoinScope::HistoryIncludingCurrent {
                return Err(unsupported(
                    "historical dependencies cannot be hidden in a current predicate",
                ));
            }
            let CompiledExpr::Eq(ResolvedFieldRef::Local { field_index, .. }, equality) = predicate
            else {
                return Err(unsupported(
                    "historical set_exists supports exactly one local field Eq literal",
                ));
            };
            let raw_join = raw.traces[join.source_trace]
                .joins
                .iter()
                .find(|j| j.id == join.id.0)
                .ok_or_else(|| unsupported("historical join declaration is missing"))?;
            if join.cardinality != JoinCardinality::ZeroOrMany
                || join.consensus.is_some()
                || raw_join.namespace_parameters != policy.namespace_parameters
            {
                return Err(unsupported(
                    "historical existence requires a namespace-bound zero_or_many set without consensus",
                ));
            }
            let target = traces
                .get(join.target_trace)
                .ok_or_else(|| unsupported("historical target is missing"))?;
            let operand = target
                .fields()
                .get(*field_index)
                .ok_or_else(|| unsupported("historical operand is missing"))?;
            // Local fields and their entire derived closure are resolved now;
            // a related/derived operand may not masquerade as local evidence.
            let mut source_dependencies = RuleDependencies::local_fields(
                &cross.rule.when,
                traces[join.source_trace].derived_values(),
            )?;
            source_dependencies.extend(&join.source_fields);
            let descriptor = serde_json::json!({
                "kind":"history.existence.v1", "schema_hash":hash,
                "encoding":raw.encoding, "join":raw_join,
                "operand":raw.traces[join.target_trace].fields[*field_index],
                "operand_id":operand.id(), "eq_bytes":equality,
                "target_record_key":raw.traces[join.target_trace].record_key,
            });
            let canonical = crate::schema::canonical::canonical_value_bytes(descriptor)
                .map_err(|_| unsupported("historical predicate identity could not be encoded"))?;
            let definition = crate::Sha256Digest(Sha256::digest(canonical).into());
            if let Some(existing) = plans.get_mut(join_index) {
                if existing.definition != definition {
                    return Err(unsupported(
                        "one historical join must name one predicate definition; use distinct joins for distinct predicates",
                    ));
                }
                existing.source_dependencies.extend(source_dependencies);
                existing.source_dependencies.sort_unstable();
                existing.source_dependencies.dedup();
            } else {
                plans.insert(
                    *join_index,
                    HistoryExistencePlan {
                        join_index: *join_index,
                        operand: *field_index,
                        equality: equality.clone(),
                        definition,
                        source_dependencies: source_dependencies.into_iter().collect(),
                    },
                );
            }
        }
    }
    if plans.len() != historical {
        return Err(unsupported(
            "each historical join requires an executed supported set predicate",
        ));
    }
    Ok(plans.into_values().collect())
}
