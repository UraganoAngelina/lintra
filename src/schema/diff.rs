//! Offline review of typed schema revisions; no validation or migration policy.
use crate::{CanonicalSchemaError, RawFlowSchema, Sha256Digest, schema_fingerprint};
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaDiffSide {
    Before,
    After,
}

/// Ambiguous IDs cannot be paired safely. Other lint failures do not prevent
/// review of a draft, and are deliberately left to the existing schema linter.
#[derive(Debug)]
pub enum SchemaDiffError {
    DifferentFlows {
        before: String,
        after: String,
    },
    DuplicateTraceId {
        side: SchemaDiffSide,
        id: String,
    },
    DuplicateFieldId {
        side: SchemaDiffSide,
        trace_id: String,
        id: String,
    },
    Canonical(CanonicalSchemaError),
    Serialization(serde_json::Error),
}
impl fmt::Display for SchemaDiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DifferentFlows { .. } => f.write_str("schema diff requires the same flow ID"),
            Self::DuplicateTraceId { .. } => f.write_str("schema diff has ambiguous trace IDs"),
            Self::DuplicateFieldId { .. } => f.write_str("schema diff has ambiguous field IDs"),
            Self::Canonical(_) => f.write_str("schema diff fingerprint failed"),
            Self::Serialization(_) => f.write_str("schema diff serialization failed"),
        }
    }
}
impl Error for SchemaDiffError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Canonical(cause) => Some(cause),
            Self::Serialization(cause) => Some(cause),
            _ => None,
        }
    }
}

/// Canonical hash is of the normative model, not source-file bytes or run options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SchemaDiffRevision {
    pub flow_id: String,
    pub schema_version: String,
    pub regulatory_version: String,
    pub schema_hash: Sha256Digest,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SchemaChangeKind {
    Added {
        after: Value,
    },
    Removed {
        before: Value,
    },
    Modified {
        before: Value,
        after: Value,
    },
    /// Full ID order for review, emitted only if shared IDs change relative order.
    OrderChanged {
        before_ids: Vec<String>,
        after_ids: Vec<String>,
    },
}
/// JSON pointers refer to the serialized raw models. Added has only after_path,
/// removed only before_path. Both pointers are present for modification/order.
/// IDs are scoped to their trace, without assuming numeric or dotted syntax.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SchemaChange {
    pub trace_id: Option<String>,
    pub field_id: Option<String>,
    pub before_path: Option<String>,
    pub after_path: Option<String>,
    pub change: SchemaChangeKind,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SchemaDiff {
    pub before: SchemaDiffRevision,
    pub after: SchemaDiffRevision,
    pub changes: Vec<SchemaChange>,
}

fn check_ids(raw: &RawFlowSchema, side: SchemaDiffSide) -> Result<(), SchemaDiffError> {
    let mut traces = BTreeSet::new();
    for trace in &raw.traces {
        if !traces.insert(&trace.id) {
            return Err(SchemaDiffError::DuplicateTraceId {
                side,
                id: trace.id.clone(),
            });
        }
        let mut fields = BTreeSet::new();
        for field in &trace.fields {
            if !fields.insert(&field.id) {
                return Err(SchemaDiffError::DuplicateFieldId {
                    side,
                    trace_id: trace.id.clone(),
                    id: field.id.clone(),
                });
            }
        }
    }
    Ok(())
}
fn revision(raw: &RawFlowSchema) -> Result<SchemaDiffRevision, SchemaDiffError> {
    Ok(SchemaDiffRevision {
        flow_id: raw.flow_id.clone(),
        schema_version: raw.schema_version.clone(),
        regulatory_version: raw.regulatory_version.clone(),
        schema_hash: schema_fingerprint(raw).map_err(SchemaDiffError::Canonical)?,
    })
}
/// Escape each JSON-pointer segment, including slash/tilde in metadata names.
fn child(path: &str, segment: &str) -> String {
    format!("{path}/{}", segment.replace('~', "~0").replace('/', "~1"))
}
#[derive(Clone, Copy)]
enum Node {
    Flow,
    Trace,
    Field,
    Other,
}
#[derive(Clone, Copy, Default)]
struct Context<'a> {
    trace: Option<&'a str>,
    field: Option<&'a str>,
}
fn emit(
    out: &mut Vec<SchemaChange>,
    context: Context<'_>,
    before: Option<&str>,
    after: Option<&str>,
    change: SchemaChangeKind,
) {
    out.push(SchemaChange {
        trace_id: context.trace.map(str::to_owned),
        field_id: context.field.map(str::to_owned),
        before_path: before.map(str::to_owned),
        after_path: after.map(str::to_owned),
        change,
    });
}
/// These arrays come only from RawFlowSchema.traces or RawRecordSchema.fields:
/// typed serialization guarantees a string id, and check_ids rejects duplicates.
fn by_id(values: &[Value]) -> BTreeMap<&str, (usize, &Value)> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            (
                value["id"]
                    .as_str()
                    .expect("typed schema entity has a string ID"),
                (index, value),
            )
        })
        .collect()
}
fn identified(
    before: &[Value],
    after: &[Value],
    paths: (&str, &str),
    node: Node,
    context: Context<'_>,
    out: &mut Vec<SchemaChange>,
) {
    let left = by_id(before);
    let right = by_id(after);
    let ids = |values: &[Value]| {
        values
            .iter()
            .map(|v| v["id"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    let before_ids = ids(before);
    let after_ids = ids(after);
    let left_shared = before_ids
        .iter()
        .filter(|id| right.contains_key(id.as_str()))
        .collect::<Vec<_>>();
    let right_shared = after_ids
        .iter()
        .filter(|id| left.contains_key(id.as_str()))
        .collect::<Vec<_>>();
    if left_shared != right_shared {
        emit(
            out,
            context,
            Some(paths.0),
            Some(paths.1),
            SchemaChangeKind::OrderChanged {
                before_ids,
                after_ids,
            },
        );
    }
    // Lexical ID traversal makes pairing/order independent of map implementation.
    for id in left
        .keys()
        .chain(right.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        let context = match node {
            Node::Trace => Context {
                trace: Some(id),
                field: None,
            },
            Node::Field => Context {
                field: Some(id),
                ..context
            },
            _ => unreachable!("only trace/field collections have identity pairing"),
        };
        let old = left
            .get(id)
            .map(|(i, value)| (child(paths.0, &i.to_string()), *value));
        let new = right
            .get(id)
            .map(|(i, value)| (child(paths.1, &i.to_string()), *value));
        compare(
            old.as_ref().map(|(p, v)| (p.as_str(), *v)),
            new.as_ref().map(|(p, v)| (p.as_str(), *v)),
            node,
            context,
            out,
        );
    }
}
fn compare(
    before: Option<(&str, &Value)>,
    after: Option<(&str, &Value)>,
    node: Node,
    context: Context<'_>,
    out: &mut Vec<SchemaChange>,
) {
    match (before, after) {
        (Some((path, value)), None) => emit(
            out,
            context,
            Some(path),
            None,
            SchemaChangeKind::Removed {
                before: value.clone(),
            },
        ),
        (None, Some((path, value))) => emit(
            out,
            context,
            None,
            Some(path),
            SchemaChangeKind::Added {
                after: value.clone(),
            },
        ),
        (Some((old_path, old)), Some((new_path, new))) if old != new => match (old, new) {
            (Value::Object(left), Value::Object(right)) => {
                for key in left.keys().chain(right.keys()).collect::<BTreeSet<_>>() {
                    let a = child(old_path, key);
                    let b = child(new_path, key);
                    match (node, key.as_str(), left.get(key), right.get(key)) {
                        (Node::Flow, "traces", Some(Value::Array(l)), Some(Value::Array(r))) => {
                            identified(l, r, (&a, &b), Node::Trace, context, out)
                        }
                        (Node::Trace, "fields", Some(Value::Array(l)), Some(Value::Array(r))) => {
                            identified(l, r, (&a, &b), Node::Field, context, out)
                        }
                        _ => compare(
                            left.get(key).map(|v| (a.as_str(), v)),
                            right.get(key).map(|v| (b.as_str(), v)),
                            Node::Other,
                            context,
                            out,
                        ),
                    }
                }
            }
            (Value::Array(left), Value::Array(right)) => {
                // All other arrays retain positional semantics: rules, joins,
                // constraints, operands and domain values are never sorted away.
                for i in 0..left.len().max(right.len()) {
                    let a = child(old_path, &i.to_string());
                    let b = child(new_path, &i.to_string());
                    compare(
                        left.get(i).map(|v| (a.as_str(), v)),
                        right.get(i).map(|v| (b.as_str(), v)),
                        Node::Other,
                        context,
                        out,
                    );
                }
            }
            _ => emit(
                out,
                context,
                Some(old_path),
                Some(new_path),
                SchemaChangeKind::Modified {
                    before: old.clone(),
                    after: new.clone(),
                },
            ),
        },
        _ => {} // Equal values (including empty containers) have no changes.
    }
}
/// Compare two revisions of one flow without I/O or compilation. Values and
/// array order are exact; runtime encoding bindings follow the existing canonical
/// exclusion. Trace/field IDs are paired only within their declared scope.
/// The result is descriptive: it makes no claim about regulatory compatibility.
pub fn diff_schemas(
    before: &RawFlowSchema,
    after: &RawFlowSchema,
) -> Result<SchemaDiff, SchemaDiffError> {
    if before.flow_id != after.flow_id {
        return Err(SchemaDiffError::DifferentFlows {
            before: before.flow_id.clone(),
            after: after.flow_id.clone(),
        });
    }
    check_ids(before, SchemaDiffSide::Before)?;
    check_ids(after, SchemaDiffSide::After)?;
    let old = serde_json::to_value(before).map_err(SchemaDiffError::Serialization)?;
    let new = serde_json::to_value(after).map_err(SchemaDiffError::Serialization)?;
    let mut report = SchemaDiff {
        before: revision(before)?,
        after: revision(after)?,
        changes: Vec::new(),
    };
    compare(
        Some(("", &old)),
        Some(("", &new)),
        Node::Flow,
        Context::default(),
        &mut report.changes,
    );
    Ok(report)
}
