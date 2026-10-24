//! Real compiled/projected synthetic records; no persistent history or rules.
use lintra::*;
use serde_json::{Value, json};

fn schema() -> CompiledFlowSchema {
    let mut raw: Value =
        serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
    let mut record = raw["traces"][0].clone();
    record["fields"] = json!(
        ["PK", "SOURCE", "TARGET"]
            .into_iter()
            .enumerate()
            .map(|(i, id)| {
                let mut field = record["fields"][0].clone();
                field["id"] = json!(id);
                field["physical"]["start"] = json!(2 * i + 1);
                field["physical"]["length"] = json!(2);
                field
            })
            .collect::<Vec<_>>()
    );
    record["record_key"]["fields"] = json!(["PK"]);
    let traces = ["S", "T", "U"].map(|id| {
        let mut trace = record.clone();
        trace["id"] = json!(id);
        trace
    });
    raw["traces"] = json!(traces);
    raw["required_capabilities"] = json!(["local.v1", "joins.v1", "joins.sets.v1"]);
    let relation = |id, target, source_field, target_field| {
        let missing = json!({"policy":"unknown","diagnostic":{"code":"SYNTHETIC_MISSING","message_template":"Synthetic missing context"}});
        json!({"id":id,"target_trace":target,"source_fields":[source_field],"target_fields":[target_field],
            "cardinality":"zero_or_many","scope":"current_submission","namespace_parameters":[],
            "on_missing_record":missing,"on_missing_trace":missing})
    };
    raw["traces"][0]["joins"] = json!([
        relation("S_T", "T", "SOURCE", "TARGET"),
        relation("SELF", "S", "SOURCE", "TARGET"),
        relation("OTHER", "T", "PK", "PK")
    ]);
    raw["traces"][1]["fields"][2]["missing_value"] = json!({"kind":"spaces"});
    let mut raw: RawFlowSchema = serde_json::from_value(raw).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw).unwrap()
}
fn projected(schema: &CompiledFlowSchema, trace: &str, bytes: &[u8], line: u64) -> ProjectedRecord {
    project_record(
        &RecordView::new(bytes),
        schema.trace(&TraceId(trace.into())).unwrap(),
        RecordIdentity::default(),
        line,
    )
    .unwrap()
}
fn key(schema: &CompiledFlowSchema, bytes: &[u8], index: usize) -> JoinKey {
    build_join_key(
        &RecordView::new(bytes),
        &[index],
        schema.trace(&TraceId("S".into())).unwrap(),
    )
    .unwrap()
}
fn lines(set: &RelatedRecordSet<'_>) -> Vec<u64> {
    set.iter().map(|record| record.line_number).collect()
}

#[test]
fn trait_object_retains_zero_one_and_many_matches_in_physical_line_order() {
    let flow = schema();
    let mut store: Box<dyn ContextStore> = Box::new(InMemoryContextStore::new());
    let target = TraceId("T".into());
    let join = JoinId("S_T".into());
    let search = key(&flow, b"00aaBB", 2);
    assert!(store.find(&target, &join, &search).unwrap().is_empty());
    assert_eq!(store.iter_trace(&target).count(), 0);
    store.insert(projected(&flow, "T", b"00xxBB", 3)).unwrap();
    assert_eq!(lines(&store.find(&target, &join, &search).unwrap()), [3]);
    for line in [1, 2] {
        store
            .insert(projected(&flow, "T", b"00xxBB", line))
            .unwrap();
    }
    let matches = store.find(&target, &join, &search).unwrap();
    assert_eq!(matches.len(), 3);
    assert_eq!(lines(&matches), [1, 2, 3]);
    assert_eq!(
        matches
            .iter()
            .rev()
            .map(|r| r.line_number)
            .collect::<Vec<_>>(),
        [3, 2, 1]
    );
    assert_eq!(
        store
            .iter_trace(&target)
            .map(|r| r.line_number)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    // Equal PKs at distinct positions remain records, not a driver violation.
    assert!(
        matches
            .iter()
            .all(|r| r.values[&FieldId("PK".into())] == b"00")
    );
}

#[test]
fn indexes_isolate_trace_join_and_key_and_self_join_indexes_only_target_side() {
    let flow = schema();
    let mut store = InMemoryContextStore::new();
    store.insert(projected(&flow, "S", b"00aaBB", 1)).unwrap();
    store.insert(projected(&flow, "T", b"00aaBB", 1)).unwrap();
    store.insert(projected(&flow, "U", b"00aaBB", 1)).unwrap();
    let source = key(&flow, b"00aaBB", 1);
    let target = key(&flow, b"00aaBB", 2);
    let pk = key(&flow, b"00aaBB", 0);
    for (trace, join, search, count) in [
        ("S", "SELF", &target, 1),
        ("S", "SELF", &source, 0),
        ("S", "S_T", &source, 0),
        ("T", "S_T", &target, 1),
        ("T", "S_T", &source, 0),
        ("T", "OTHER", &pk, 1),
        ("T", "OTHER", &target, 0),
        ("U", "S_T", &target, 0),
        ("ABSENT", "S_T", &target, 0),
        ("T", "UNDECLARED", &target, 0),
    ] {
        assert_eq!(
            store
                .find(&TraceId(trace.into()), &JoinId(join.into()), search)
                .unwrap()
                .len(),
            count,
            "{trace}/{join}"
        );
    }
    assert_eq!(store.iter_trace(&TraceId("U".into())).count(), 1);
    assert_eq!(store.iter_trace(&TraceId("ABSENT".into())).count(), 0);
}

#[test]
fn missing_target_is_not_indexed_but_other_keys_and_the_record_are_retained() {
    let flow = schema();
    let mut store = InMemoryContextStore::new();
    store.insert(projected(&flow, "T", b"00aa  ", 1)).unwrap();
    let trace = TraceId("T".into());
    let spaces = key(&flow, b"00aa  ", 2);
    assert!(
        store
            .find(&trace, &JoinId("S_T".into()), &spaces)
            .unwrap()
            .is_empty()
    );
    let pk = key(&flow, b"00aa  ", 0);
    assert_eq!(
        store
            .find(&trace, &JoinId("OTHER".into()), &pk)
            .unwrap()
            .len(),
        1
    );
    let record = store.iter_trace(&trace).next().unwrap();
    assert!(
        matches!(&record.join_keys[&JoinProjectionKey { join_id:JoinId("S_T".into()),side:JoinProjectionSide::Target }],ProjectedJoinValue::MissingComponent(id) if id.0=="TARGET")
    );
}

#[test]
fn recoverable_insert_errors_leave_trace_order_and_indexes_unchanged() {
    let flow = schema();
    let mut store = InMemoryContextStore::new();
    store.insert(projected(&flow, "T", b"00aaBB", 1)).unwrap();
    let duplicate = projected(&flow, "T", b"99xxCC", 1);
    assert!(matches!(
        store.insert(duplicate),
        Err(ContextError::DuplicatePosition { line_number: 1, .. })
    ));
    let mut invalid = projected(&flow, "T", b"00aaBB", 2);
    invalid.line_number = 0;
    assert_eq!(store.insert(invalid), Err(ContextError::InvalidLineNumber));
    let mut invalid = projected(&flow, "T", b"00aaBB", 2);
    invalid.trace_id = TraceId(String::new());
    assert_eq!(store.insert(invalid), Err(ContextError::InvalidTraceId));
    let mut invalid = projected(&flow, "T", b"00aaBB", 2);
    invalid.join_keys.insert(
        JoinProjectionKey {
            join_id: JoinId(String::new()),
            side: JoinProjectionSide::Target,
        },
        ProjectedJoinValue::Available(key(&flow, b"00aaBB", 2)),
    );
    assert_eq!(store.insert(invalid), Err(ContextError::InvalidJoinId));
    let trace = TraceId("T".into());
    let join = JoinId("S_T".into());
    assert_eq!(
        store
            .iter_trace(&trace)
            .map(|r| r.line_number)
            .collect::<Vec<_>>(),
        [1]
    );
    assert_eq!(
        lines(
            &store
                .find(&trace, &join, &key(&flow, b"00aaBB", 2))
                .unwrap()
        ),
        [1]
    );
    assert!(
        store
            .find(&trace, &join, &key(&flow, b"99xxCC", 2))
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        store.find(&TraceId(String::new()), &join, &key(&flow, b"00aaBB", 2)),
        Err(ContextError::InvalidTraceId)
    ));
    assert!(matches!(
        store.find(&trace, &JoinId(String::new()), &key(&flow, b"00aaBB", 2)),
        Err(ContextError::InvalidJoinId)
    ));
}

#[test]
fn trace_and_multiple_indexes_share_one_immutable_payload_without_exposing_it_in_debug() {
    let flow = schema();
    let mut store = InMemoryContextStore::default();
    let mut record = projected(&flow, "T", b"00aa\0\xff", 1);
    record.identity.sioc_record_id = Some("SYNTHETIC-SECRET".into());
    store.insert(record).unwrap();
    let trace = TraceId("T".into());
    let matches = store
        .find(&trace, &JoinId("S_T".into()), &key(&flow, b"00aa\0\xff", 2))
        .unwrap();
    let other = store
        .find(
            &trace,
            &JoinId("OTHER".into()),
            &key(&flow, b"00aa\0\xff", 0),
        )
        .unwrap();
    let ordered = store.iter_trace(&trace).next().unwrap();
    assert!(std::ptr::eq(ordered, matches.iter().next().unwrap()));
    assert!(std::ptr::eq(ordered, other.iter().next().unwrap()));
    assert_eq!(ordered.values[&FieldId("TARGET".into())], b"\0\xff");
    assert!(!format!("{store:?} {matches:?}").contains("SYNTHETIC-SECRET"));
    let independent = InMemoryContextStore::new();
    assert_eq!(independent.iter_trace(&trace).count(), 0);
}
