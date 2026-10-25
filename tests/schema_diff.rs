use lintra::*;
use serde_json::{Value, json};

fn raw() -> RawFlowSchema {
    serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap()
}
fn value(raw: &RawFlowSchema) -> Value {
    serde_json::to_value(raw).unwrap()
}
fn same(raw: &RawFlowSchema) -> RawFlowSchema {
    serde_json::from_value(value(raw)).unwrap()
}
fn field(raw: &RawFlowSchema, id: &str) -> RawFieldSpec {
    let mut field = raw.traces[0].fields[0].clone();
    field.id = id.to_owned();
    field
}
fn trace(raw: &RawFlowSchema, id: &str) -> RawRecordSchema {
    let mut trace = raw.traces[0].clone();
    trace.id = id.to_owned();
    trace
}
fn assert_pointers(report: &SchemaDiff, before: &RawFlowSchema, after: &RawFlowSchema) {
    let before = value(before);
    let after = value(after);
    for item in &report.changes {
        match &item.change {
            SchemaChangeKind::Added { after: expected } => {
                assert!(item.before_path.is_none());
                assert_eq!(
                    after.pointer(item.after_path.as_deref().unwrap()),
                    Some(expected)
                );
            }
            SchemaChangeKind::Removed { before: expected } => {
                assert!(item.after_path.is_none());
                assert_eq!(
                    before.pointer(item.before_path.as_deref().unwrap()),
                    Some(expected)
                );
            }
            SchemaChangeKind::Modified {
                before: a,
                after: b,
            } => {
                assert_eq!(
                    before.pointer(item.before_path.as_deref().unwrap()),
                    Some(a)
                );
                assert_eq!(after.pointer(item.after_path.as_deref().unwrap()), Some(b));
            }
            SchemaChangeKind::OrderChanged {
                before_ids,
                after_ids,
            } => {
                for (doc, path, ids) in [
                    (&before, &item.before_path, before_ids),
                    (&after, &item.after_path, after_ids),
                ] {
                    let actual: Vec<_> = doc
                        .pointer(path.as_deref().unwrap())
                        .unwrap()
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v["id"].as_str().unwrap())
                        .collect();
                    assert_eq!(actual, ids.iter().map(String::as_str).collect::<Vec<_>>());
                }
            }
        }
    }
}
#[test]
fn identical_normative_models_and_different_runtime_bindings_have_no_diff() {
    let a = raw();
    let mut b = same(&a);
    b.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let report = diff_schemas(&a, &b).unwrap();
    assert!(report.changes.is_empty());
    assert_eq!(report.before.schema_hash, schema_fingerprint(&a).unwrap());
    assert_eq!(report.before, report.after);
    assert_eq!(
        serde_json::to_vec(&report).unwrap(),
        serde_json::to_vec(&diff_schemas(&a, &b).unwrap()).unwrap()
    );
}
#[test]
fn fields_are_added_removed_and_modified_by_id_without_false_rename_or_reorder() {
    let mut a = raw();
    a.traces[0].fields = vec![field(&a, "A"), field(&a, "B"), field(&a, "C")];
    let mut b = same(&a);
    b.traces[0].fields.remove(0);
    b.traces[0].fields[0].name = "Description updated".into();
    b.traces[0].fields.push(field(&a, "D"));
    let report = diff_schemas(&a, &b).unwrap();
    assert_eq!(report.changes.len(), 3);
    let ids: Vec<_> = report
        .changes
        .iter()
        .map(|c| c.field_id.as_deref().unwrap())
        .collect();
    assert_eq!(ids, ["A", "B", "D"]);
    assert!(matches!(
        report.changes[0].change,
        SchemaChangeKind::Removed { .. }
    ));
    assert!(matches!(
        report.changes[1].change,
        SchemaChangeKind::Modified { .. }
    ));
    assert!(matches!(
        report.changes[2].change,
        SchemaChangeKind::Added { .. }
    ));
    assert_eq!(
        report.changes[1].before_path.as_deref(),
        Some("/traces/0/fields/1/name")
    );
    assert_eq!(
        report.changes[1].after_path.as_deref(),
        Some("/traces/0/fields/0/name")
    );
    assert_pointers(&report, &a, &b);
}
#[test]
fn trace_and_field_order_are_explicit_and_common_entities_remain_correctly_paired() {
    let mut a = raw();
    a.traces = vec![trace(&a, "T1"), trace(&a, "T2")];
    a.traces[0].fields = vec![field(&a, "A"), field(&a, "B")];
    let mut b = same(&a);
    b.traces.swap(0, 1);
    b.traces[1].fields.swap(0, 1);
    let mut physical = serde_json::to_value(&b.traces[1].fields[0].physical).unwrap();
    physical["length"] = json!(physical["length"].as_u64().unwrap() + 1);
    b.traces[1].fields[0].physical = serde_json::from_value(physical).unwrap();
    let report = diff_schemas(&a, &b).unwrap();
    assert_eq!(report.changes.len(), 3);
    assert!(matches!(
        report.changes[0].change,
        SchemaChangeKind::OrderChanged { .. }
    ));
    assert!(matches!(
        report.changes[1].change,
        SchemaChangeKind::OrderChanged { .. }
    ));
    let changed = &report.changes[2];
    assert_eq!(changed.trace_id.as_deref(), Some("T1"));
    assert_eq!(changed.field_id.as_deref(), Some("B"));
    assert_eq!(
        changed.before_path.as_deref(),
        Some("/traces/0/fields/1/physical/length")
    );
    assert_eq!(
        changed.after_path.as_deref(),
        Some("/traces/1/fields/0/physical/length")
    );
    assert_pointers(&report, &a, &b);
}
#[test]
fn trace_add_remove_and_reused_field_ids_in_different_traces_are_unambiguous() {
    let mut a = raw();
    a.traces = vec![trace(&a, "A"), trace(&a, "B")];
    let mut b = same(&a);
    b.traces[0].id = "C".into();
    let report = diff_schemas(&a, &b).unwrap();
    assert_eq!(report.changes.len(), 2);
    assert_eq!(report.changes[0].trace_id.as_deref(), Some("A"));
    assert_eq!(report.changes[1].trace_id.as_deref(), Some("C"));
    assert!(report.changes.iter().all(|c| c.field_id.is_none()));
    assert_pointers(&report, &a, &b);
}
#[test]
fn metadata_rules_and_exact_values_are_not_lost_or_normalized() {
    let a = raw();
    let mut b = same(&a);
    let large: Value =
        serde_json::from_str("12345678901234567890123456789012345678901234567890").unwrap();
    b.source
        .insert("a/b~c".into(), json!({"value": [" 001é ", large]}));
    b.traces[0].fields[0].regulatory_metadata = Some(std::collections::BTreeMap::from([(
        "notes_original".into(),
        json!("Nota\r\n  originale"),
    )]));
    b.traces[0].record_length += 1;
    b.schema_version = "revision-two".into();
    b.regulatory_version = "regional-two".into();
    let report = diff_schemas(&a, &b).unwrap();
    assert_eq!(report.changes.len(), 5);
    assert_ne!(report.before.schema_hash, report.after.schema_hash);
    assert!(
        report
            .changes
            .iter()
            .any(|c| c.after_path.as_deref() == Some("/source/a~1b~0c"))
    );
    assert!(
        serde_json::to_string(&report)
            .unwrap()
            .contains("12345678901234567890123456789012345678901234567890")
    );
    assert_pointers(&report, &a, &b);
    let reverse = diff_schemas(&b, &a).unwrap();
    assert_pointers(&reverse, &b, &a);
    assert_eq!(reverse.changes.len(), report.changes.len());
}
#[test]
fn ordinary_arrays_keep_positional_semantics_instead_of_sorting_away_changes() {
    let mut a = raw();
    a.source.insert("sequence".into(), json!(["01", "02"]));
    let mut b = same(&a);
    b.source
        .insert("sequence".into(), json!(["02", "01", "03"]));
    let report = diff_schemas(&a, &b).unwrap();
    assert_eq!(report.changes.len(), 3);
    assert!(matches!(
        report.changes[0].change,
        SchemaChangeKind::Modified { .. }
    ));
    assert!(matches!(
        report.changes[1].change,
        SchemaChangeKind::Modified { .. }
    ));
    assert!(matches!(
        report.changes[2].change,
        SchemaChangeKind::Added { .. }
    ));
    assert_pointers(&report, &a, &b);
}
#[test]
fn ambiguous_ids_and_different_flows_are_typed_errors_but_draft_geometry_is_reviewable() {
    let a = raw();
    let mut b = same(&a);
    b.flow_id = "other-flow".into();
    assert!(matches!(
        diff_schemas(&a, &b),
        Err(SchemaDiffError::DifferentFlows { .. })
    ));
    b = same(&a);
    b.traces.push(b.traces[0].clone());
    assert!(matches!(
        diff_schemas(&a, &b),
        Err(SchemaDiffError::DuplicateTraceId {
            side: SchemaDiffSide::After,
            ..
        })
    ));
    assert!(matches!(
        diff_schemas(&b, &a),
        Err(SchemaDiffError::DuplicateTraceId {
            side: SchemaDiffSide::Before,
            ..
        })
    ));
    b = same(&a);
    let duplicate = b.traces[0].fields[0].clone();
    b.traces[0].fields.push(duplicate);
    assert!(matches!(
        diff_schemas(&a, &b),
        Err(SchemaDiffError::DuplicateFieldId {
            side: SchemaDiffSide::After,
            ..
        })
    ));
    assert!(matches!(
        diff_schemas(&b, &a),
        Err(SchemaDiffError::DuplicateFieldId {
            side: SchemaDiffSide::Before,
            ..
        })
    ));
    b = same(&a);
    b.traces[0].record_length = 0;
    assert!(!diff_schemas(&a, &b).unwrap().changes.is_empty());
    b.traces.clear();
    assert_pointers(&diff_schemas(&a, &b).unwrap(), &a, &b);
}
#[test]
fn object_order_is_irrelevant_and_report_order_is_lexical_and_deterministic() {
    let a = raw();
    let mut b = same(&a);
    b.source.insert("z".into(), json!(1));
    b.source.insert("a".into(), json!(2));
    let report = diff_schemas(&a, &b).unwrap();
    let paths: Vec<_> = report
        .changes
        .iter()
        .map(|c| c.after_path.as_deref().unwrap())
        .collect();
    assert_eq!(paths, ["/source/a", "/source/z"]);
    let mut doc = value(&b);
    doc["source"] = serde_json::from_str("{\"z\":1,\"a\":2}").unwrap();
    // Include the remaining metadata unchanged, merely rebuild in reverse order.
    let mut map = value(&b)["source"].as_object().unwrap().clone();
    map.remove("a");
    map.remove("z");
    for (key, v) in map {
        doc["source"][key] = v;
    }
    let shuffled: RawFlowSchema = serde_json::from_value(doc).unwrap();
    assert_eq!(report, diff_schemas(&a, &shuffled).unwrap());
}
#[test]
fn approved_sioc_revision_reports_real_rules_and_policy_changes_without_execution() {
    let a: RawFlowSchema = serde_json::from_str(include_str!(
        "../schemas/sioc/2026-04-20.structural-minimal.1/flow_schema.json"
    ))
    .unwrap();
    let b: RawFlowSchema = serde_json::from_str(include_str!(
        "../schemas/sioc/2026-04-20.structural-minimal.2/flow_schema.json"
    ))
    .unwrap();
    let report = diff_schemas(&a, &b).unwrap();
    assert_pointers(&report, &a, &b);
    let added: Vec<_> = report
        .changes
        .iter()
        .filter_map(|c| match &c.change {
            SchemaChangeKind::Added { after } => after.get("id").and_then(Value::as_str),
            _ => None,
        })
        .collect();
    assert_eq!(
        added,
        [
            "REL-trace2_to_trace1",
            "REL-trace3_to_trace2",
            "REL-trace3_to_trace1",
            "REL-trace42_to_trace41"
        ]
    );
    assert!(!report.changes.iter().any(|c| c.field_id.is_some()));
    assert!(report.changes.iter().any(|c| matches!(&c.change, SchemaChangeKind::Modified { after: Value::String(code), .. } if code=="violation")));
    assert_eq!(report.after.schema_hash, schema_fingerprint(&b).unwrap());
}
