//! Minimal owned projections from compiler-verified, synthetic declarations.
use lintra::*;
use serde_json::{Value, json};

fn sample() -> Value {
    let fields = ["PK", "SOURCE", "TARGET", "ID", "VALUE", "FLAG", "UNUSED"]
        .into_iter()
        .enumerate()
        .map(|(i, id)| {
            json!({
                "id":id,"name":id,"physical":{"type":"alphanumeric","start":2*i+1,"length":2,
                "alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}},
                "missing_value":{"kind":"never_missing"},
                "presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[]
            })
        })
        .collect::<Vec<_>>();
    let record = |id| {
        json!({"id":id,"name":id,"record_length":14,"fields":fields,
        "record_key":{"fields":["PK"],"uniqueness_scope":"trace","error_code":"E_DUPLICATE_RECORD_KEY"},
        "identity":[{"target":"technical","field":"ID","technical_name":"synthetic-id"}],
        "joins":[],"rules":[],"named_conditions":{},"derived_values":{},
        "repeated_groups":[],"documented_non_executable_rules":[]})
    };
    let mut raw = json!({"schema_format_version":1,"flow_id":"SYNTHETIC","schema_version":"v1",
        "regulatory_version":"synthetic","source":{},"encoding":{"mode":"runtime_required","position_unit":"byte"},
        "required_capabilities":["local.v1","joins.v1","joins.sets.v1","joins.consensus.v1"],
        "parameters":[],"reference_data":[],"persistence_policy":null,"traces":[record("S"),record("T"),record("U")]});
    raw["traces"][0]["joins"] = json!([join("S_T", "T")]);
    raw
}
fn join(id: &str, target: &str) -> Value {
    let missing = json!({"policy":"unknown","diagnostic":{"code":"SYNTHETIC_MISSING","message_template":"Synthetic missing relation"}});
    json!({"id":id,"target_trace":target,"source_fields":["SOURCE"],"target_fields":["TARGET"],
        "cardinality":"zero_or_many","scope":"current_submission","namespace_parameters":[],
        "on_missing_record":missing,"on_missing_trace":missing})
}
fn reduction(condition: Value) -> Value {
    json!({"kind":"consensus","fields":["VALUE"],"include_if":condition,"on_conflict":"unknown",
        "diagnostic":{"code":"SYNTHETIC_CONFLICT","message_template":"Synthetic conflicting tuples"}})
}
fn compile(value: Value) -> Result<CompiledFlowSchema, SchemaCompilationError> {
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw)
}
fn trace<'a>(flow: &'a CompiledFlowSchema, id: &str) -> &'a CompiledRecordSchema {
    flow.trace(&TraceId(id.into())).unwrap()
}
fn binding(id: &str, side: JoinProjectionSide) -> JoinProjectionKey {
    JoinProjectionKey {
        join_id: JoinId(id.into()),
        side,
    }
}
fn available<'a>(record: &'a ProjectedRecord, id: &str, side: JoinProjectionSide) -> &'a JoinKey {
    match &record.join_keys[&binding(id, side)] {
        ProjectedJoinValue::Available(key) => key,
        other => panic!("expected available synthetic key, got {other:?}"),
    }
}

#[test]
fn compiler_selects_only_pk_identity_both_join_sides_and_consensus_dependencies() {
    let mut raw = sample();
    raw["traces"][0]["joins"][0]["reduction"] =
        reduction(json!({"eq":{"ref":{"field":"FLAG"},"value":"OK"}}));
    let flow = compile(raw).unwrap();
    let source = trace(&flow, "S").projection_plan();
    let target = trace(&flow, "T").projection_plan();
    assert!(source.required());
    assert_eq!(source.field_indexes, [0, 1, 3]);
    assert_eq!(target.field_indexes, [0, 2, 3, 4, 5]);
    assert_eq!(trace(&flow, "U").projection_plan().field_indexes, [0, 3]);
    assert_eq!(source.join_indexes, [0]);
    assert_eq!(target.join_indexes, [0]);
    assert!(trace(&flow, "U").projection_plan().join_indexes.is_empty());
    assert_eq!(source.join_plans[0].field_indexes, [1]);
    assert_eq!(target.join_plans[0].field_indexes, [2]);
    assert_eq!(
        source.join_plans[0].key,
        binding("S_T", JoinProjectionSide::Source)
    );
    assert_eq!(
        target.join_plans[0].key,
        binding("S_T", JoinProjectionSide::Target)
    );
    // LocalOnly can retain the cross plan without executing a simulated pass.
    assert!(RecordValidator::new(&flow, &TraceId("S".into())).is_ok());
}

#[test]
fn projected_bytes_are_owned_minimal_and_keep_physical_bytes_separate_from_logical_keys() {
    let mut raw = sample();
    raw["traces"][0]["fields"][1]["physical"]["alignment"] =
        json!({"kind":"right_pad","fill_byte":32,"content_policy":"strip_trailing_fill"});
    let flow = compile(raw).unwrap();
    let schema = trace(&flow, "S");
    let identity = RecordIdentity {
        sioc_record_id: Some("SYNTHETIC-ID".into()),
        ..Default::default()
    };
    let owned = {
        let mut buffer = b"00A ZZ05PPQQXX".to_vec();
        let result =
            project_record(&RecordView::new(&buffer), schema, identity.clone(), 7).unwrap();
        buffer.fill(0);
        result
    };
    assert_eq!(owned.trace_id, TraceId("S".into()));
    assert_eq!(owned.line_number, 7);
    assert_eq!(owned.identity, identity);
    assert_eq!(owned.values.len(), 3);
    assert_eq!(owned.values[&FieldId("PK".into())], b"00");
    assert_eq!(owned.values[&FieldId("SOURCE".into())], b"A ");
    assert_eq!(owned.values[&FieldId("ID".into())], b"05");
    assert_eq!(owned.values.values().map(Vec::len).sum::<usize>(), 6);
    assert!(!owned.values.contains_key(&FieldId("UNUSED".into())));
    assert_eq!(
        available(&owned, "S_T", JoinProjectionSide::Source).as_bytes(),
        &[0, 0, 0, 0, 0, 0, 0, 1, b'A']
    );
    let debug = format!("{owned:?}");
    assert!(!debug.contains("SYNTHETIC-ID"));
    assert!(!debug.contains("A "));
}

#[test]
fn self_join_keeps_source_and_target_keys_distinct_and_composite_order_exact() {
    let mut raw = sample();
    let mut own = join("SELF", "S");
    own["source_fields"] = json!(["SOURCE", "PK"]);
    own["target_fields"] = json!(["PK", "TARGET"]);
    raw["traces"][0]["joins"] = json!([own, join("S_T", "T")]);
    let flow = compile(raw).unwrap();
    let schema = trace(&flow, "S");
    assert_eq!(schema.projection_plan().field_indexes, [0, 1, 2, 3]);
    assert_eq!(schema.projection_plan().join_indexes, [0, 1]);
    assert_eq!(schema.projection_plan().join_plans.len(), 3);
    let view = RecordView::new(b"00ABCD05PPQQXX");
    let owned = project_record(&view, schema, RecordIdentity::default(), 1).unwrap();
    assert_eq!(owned.join_keys.len(), 3);
    let source = available(&owned, "SELF", JoinProjectionSide::Source);
    let target = available(&owned, "SELF", JoinProjectionSide::Target);
    assert_eq!(source, &build_join_key(&view, &[1, 0], schema).unwrap());
    assert_eq!(target, &build_join_key(&view, &[0, 2], schema).unwrap());
    assert_ne!(source, target);
}

#[test]
fn missing_key_is_typed_while_independent_keys_values_and_binary_bytes_survive() {
    let mut raw = sample();
    raw["traces"][0]["fields"][1]["missing_value"] = json!({"kind":"spaces"});
    raw["traces"][0]["joins"] = json!([join("SELF", "S")]);
    let flow = compile(raw).unwrap();
    let owned = project_record(
        &RecordView::new(b"00  \0\xff05PPQQXX"),
        trace(&flow, "S"),
        RecordIdentity::default(),
        1,
    )
    .unwrap();
    assert!(
        matches!(&owned.join_keys[&binding("SELF", JoinProjectionSide::Source)],
        ProjectedJoinValue::MissingComponent(id) if id.0 == "SOURCE")
    );
    assert_eq!(owned.values[&FieldId("TARGET".into())], b"\0\xff");
    assert!(
        available(&owned, "SELF", JoinProjectionSide::Target)
            .as_bytes()
            .ends_with(b"\0\xff")
    );
    assert_eq!(owned.values[&FieldId("SOURCE".into())], b"  ");
}

#[test]
fn line_and_record_length_failures_are_technical_without_placeholder_projections() {
    let flow = compile(sample()).unwrap();
    let schema = trace(&flow, "S");
    assert!(matches!(
        project_record(
            &RecordView::new(b"00ABCD05PPQQXX"),
            schema,
            RecordIdentity::default(),
            0
        ),
        Err(ProjectionError::InvalidLineNumber)
    ));
    for bytes in [b"short".as_slice(), b"00ABCD05PPQQXXextra".as_slice()] {
        assert!(
            matches!(project_record(&RecordView::new(bytes), schema, RecordIdentity::default(), 1),
            Err(ProjectionError::RecordLengthMismatch { expected:14, actual }) if actual == bytes.len())
        );
    }
}

#[test]
fn consensus_predicate_visits_every_boolean_field_operator_and_derived_dependency() {
    let mut raw = sample();
    raw["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!("derived.substring.v1"));
    raw["traces"][1]["derived_values"] = json!({"PART":{"kind":"substring","source_ref":{"field":"UNUSED"},"target":"raw","start":1,"length":1}});
    let r = json!({"field":"FLAG"});
    let condition = json!({"and":[
        {"eq":{"ref":r,"value":"OK"}}, {"ne":{"ref":r,"value":"NO"}},
        {"in":{"ref":r,"values":["OK"]}}, {"not_in":{"ref":r,"values":["NO"]}},
        {"is_present":{"ref":r}}, {"not":{"is_missing":{"ref":r}}},
        {"or":[{"const":false},{"derived_eq":{"ref":{"field":"VALUE"},"derived_value":"PART"}}]},
        {"derived_compare":{"derived_value":"PART","operator":"==","value":"A"}}
    ]});
    raw["traces"][0]["joins"][0]["reduction"] = reduction(condition);
    let flow = compile(raw).unwrap();
    assert_eq!(
        trace(&flow, "T").projection_plan().field_indexes,
        [0, 2, 3, 4, 5, 6]
    );
    assert_eq!(trace(&flow, "S").projection_plan().field_indexes, [0, 1, 3]);
}

#[test]
fn calendar_derived_consensus_predicate_projects_both_source_dates() {
    for (kind, capability) in [
        (
            "age_in_completed_years",
            "derived.age_in_completed_years.v1",
        ),
        ("calendar_year_delta", "derived.calendar_year_delta.v1"),
    ] {
        let mut raw = sample();
        raw["required_capabilities"]
            .as_array_mut()
            .unwrap()
            .push(json!(capability));
        let fields = raw["traces"][1]["fields"].as_array_mut().unwrap();
        for (index, id) in ["BIRTH", "REFERENCE"].into_iter().enumerate() {
            let mut field = fields[0].clone();
            field["id"] = json!(id);
            field["physical"] = json!({"type":"date","start":15+index*8,"length":8,"format":"DDMMYYYY","alignment":{"kind":"exact"}});
            fields.push(field);
        }
        raw["traces"][1]["record_length"] = json!(30);
        let mut derived = json!({"kind":kind,"birth_date_ref":{"field":"BIRTH"},"reference_date_ref":{"field":"REFERENCE"},"date_format":"DDMMYYYY"});
        if kind == "age_in_completed_years" {
            derived["leap_day_anniversary"] = json!("feb28");
        }
        raw["traces"][1]["derived_values"] = json!({"YEARS":derived});
        raw["traces"][0]["joins"][0]["reduction"] = reduction(
            json!({"derived_compare":{"derived_value":"YEARS","operator":">=","value":18}}),
        );
        let flow = compile(raw).unwrap();
        assert_eq!(
            trace(&flow, "T").projection_plan().field_indexes,
            [0, 2, 3, 4, 7, 8]
        );
    }
}

#[test]
fn unsupported_cross_rule_is_rejected_instead_of_silently_underprojected() {
    let mut raw = sample();
    raw["traces"][0]["rules"] = json!([{"id":"EXTERNAL","when":{"const":true},
        "assertions":[{"kind":"field_present","field":{"trace":"T","field":"VALUE","join":"S_T"}}],
        "error_code":"E_CROSS_FIELD","message_template":"Synthetic external requirement"}]);
    let error = compile(raw).unwrap_err();
    assert!(
        error
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::UnsupportedConstruct)
    );
}
