//! Synthetic tuple/metadata tests, no normative trace or Full execution implied.
use lintra::*;
use serde_json::{Value, json};
use std::collections::HashSet;

fn sample() -> Value {
    let field = |id, start, length| json!({"id":id,"name":id,"physical":{"type":"alphanumeric","start":start,"length":length,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]});
    let record = |id| json!({"id":id,"name":id,"record_length":4,"fields":[field("A",1,2),field("B",3,2)],"record_key":{"fields":["A"],"uniqueness_scope":"trace","error_code":"E_DUPLICATE_RECORD_KEY"},"joins":[],"identity":[],"named_conditions":{},"derived_values":{},"repeated_groups":[],"rules":[],"documented_non_executable_rules":[]});
    json!({"schema_format_version":1,"flow_id":"SYNTHETIC","schema_version":"v1","regulatory_version":"test","source":{},"encoding":{"mode":"runtime_required","position_unit":"byte"},"required_capabilities":["local.v1","joins.v1","joins.sets.v1"],"parameters":[],"reference_data":[],"persistence_policy":null,"traces":[record("S"),record("T")]})
}
fn compile(value: Value) -> Result<CompiledFlowSchema, SchemaCompilationError> {
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw)
}
fn relation(cardinality: &str, policy: &str) -> Value {
    let mut missing = json!({"policy":policy,"diagnostic":{"code":"SYNTHETIC_CONTEXT","message_template":"Synthetic related context"}});
    if policy == "violation" {
        missing["error_code"] = json!("E_RELATED_RECORD_MISSING");
    }
    json!({"id":"S_T","target_trace":"T","source_fields":["A","B"],"target_fields":["B","A"],"cardinality":cardinality,"scope":"current_submission","namespace_parameters":[],"on_missing_record":missing,"on_missing_trace":{"policy":"unknown","diagnostic":{"code":"SYNTHETIC_UNAVAILABLE","message_template":"Synthetic unavailable trace"}}})
}
#[test]
fn keys_are_byte_exact_owned_and_use_host_independent_length_prefixes() {
    let schema = compile(sample()).unwrap();
    let record = schema.trace(&TraceId("S".into())).unwrap();
    let key = build_join_key(&RecordView::new(b"00\0\xff"), &[0, 1], record).unwrap();
    assert_eq!(
        key.as_bytes(),
        &[
            0, 0, 0, 0, 0, 0, 0, 2, b'0', b'0', 0, 0, 0, 0, 0, 0, 0, 2, 0, 255
        ]
    );
    let mut original = vec![b'A', b'B', b'C', b'D'];
    let owned = build_join_key(&RecordView::new(&original), &[0, 1], record).unwrap();
    original.fill(0);
    assert!(owned.as_bytes().ends_with(b"CD"));
    assert!(!format!("{owned:?}").contains("AB"));
}
#[test]
fn compound_keys_do_not_collide_when_concatenated_values_would() {
    let mut left = sample();
    left["traces"][0]["fields"][0]["physical"]["length"] = json!(1);
    left["traces"][0]["fields"][1]["physical"]["start"] = json!(2);
    left["traces"][0]["fields"][1]["physical"]["length"] = json!(3);
    let mut right = sample();
    right["traces"][0]["fields"][0]["physical"]["length"] = json!(3);
    right["traces"][0]["fields"][1]["physical"]["start"] = json!(4);
    right["traces"][0]["fields"][1]["physical"]["length"] = json!(1);
    let left = compile(left).unwrap();
    let right = compile(right).unwrap();
    let bytes = RecordView::new(b"ABCD");
    let a = build_join_key(&bytes, &[0, 1], left.trace(&TraceId("S".into())).unwrap()).unwrap();
    let b = build_join_key(&bytes, &[0, 1], right.trace(&TraceId("S".into())).unwrap()).unwrap();
    assert_ne!(a, b);
    assert_eq!(HashSet::from([a.clone(), b.clone(), a]).len(), 2);
    let c = build_join_key(&bytes, &[1, 0], right.trace(&TraceId("S".into())).unwrap()).unwrap();
    assert_ne!(b, c);
}
#[test]
fn only_declared_alignment_changes_key_components_and_never_normalizes_zeros_or_case() {
    let mut value = sample();
    value["traces"][0]["fields"][0]["physical"]["alignment"] =
        json!({"kind":"right_pad","fill_byte":32,"content_policy":"strip_trailing_fill"});
    let schema = compile(value).unwrap();
    let record = schema.trace(&TraceId("S".into())).unwrap();
    let stripped = build_join_key(&RecordView::new(b"A Bc"), &[0, 1], record).unwrap();
    assert_eq!(&stripped.as_bytes()[..9], &[0, 0, 0, 0, 0, 0, 0, 1, b'A']);
    let zero = build_join_key(&RecordView::new(b"00Bc"), &[0, 1], record).unwrap();
    assert_eq!(&zero.as_bytes()[8..10], b"00");
    assert!(zero.as_bytes().ends_with(b"Bc"));
    let empty = build_join_key(&RecordView::new(b"  Bc"), &[0, 1], record).unwrap();
    assert_eq!(&empty.as_bytes()[..8], &[0; 8]);
}
#[test]
fn unavailable_missing_and_invalid_plans_never_become_empty_join_keys() {
    let mut value = sample();
    value["traces"][0]["fields"][0]["missing_value"] = json!({"kind":"spaces"});
    let schema = compile(value).unwrap();
    let record = schema.trace(&TraceId("S".into())).unwrap();
    assert!(matches!(
        build_join_key(&RecordView::new(b"ABCD"), &[], record),
        Err(JoinKeyError::EmptyFields)
    ));
    assert!(matches!(
        build_join_key(&RecordView::new(b"ABCD"), &[99], record),
        Err(JoinKeyError::InvalidFieldIndex(99))
    ));
    assert!(matches!(
        build_join_key(&RecordView::new(b"A"), &[0], record),
        Err(JoinKeyError::FieldUnavailable(_))
    ));
    assert!(matches!(
        build_join_key(&RecordView::new(b"  CD"), &[0, 1], record),
        Err(JoinKeyError::MissingComponent(_))
    ));
}
#[test]
fn compiler_preserves_all_cardinalities_policies_and_ordered_field_pairs() {
    for (name, expected, missing) in [
        ("zero_or_one", JoinCardinality::ZeroOrOne, "not_applicable"),
        ("exactly_one", JoinCardinality::ExactlyOne, "violation"),
        ("one_or_many", JoinCardinality::OneOrMany, "violation"),
        ("zero_or_many", JoinCardinality::ZeroOrMany, "unknown"),
    ] {
        let mut value = sample();
        value["traces"][0]["joins"] = json!([relation(name, missing)]);
        let compiled = compile(value).unwrap();
        let plan = compiled.cross_record_plan();
        assert_eq!(plan.required_traces, [0, 1]);
        let join = &plan.joins[0];
        assert_eq!(join.cardinality, expected);
        assert_eq!(join.source_trace, 0);
        assert_eq!(join.target_trace, 1);
        assert_eq!(join.source_fields, [0, 1]);
        assert_eq!(join.target_fields, [1, 0]);
        assert_eq!(join.scope, JoinScope::CurrentSubmission);
        assert!(matches!(
            join.on_missing_trace,
            RelatedPolicy::Unknown { .. }
        ));
        match missing {
            "not_applicable" => assert!(matches!(
                join.on_missing_record,
                RelatedPolicy::NotApplicable { .. }
            )),
            "violation" => assert!(matches!(
                join.on_missing_record,
                RelatedPolicy::Violation { .. }
            )),
            _ => assert!(matches!(
                join.on_missing_record,
                RelatedPolicy::Unknown { .. }
            )),
        }
        // Plans are retained without claiming that local validation executes Full.
        assert!(RecordValidator::new(&compiled, &TraceId("S".into())).is_ok());
    }
}
#[test]
fn malformed_join_references_policies_and_capabilities_are_schema_errors() {
    let mut base = sample();
    base["traces"][0]["joins"] = json!([relation("zero_or_one", "not_applicable")]);
    for (mut value, code) in [
        (
            {
                let mut v = base.clone();
                v["traces"][0]["joins"][0]["target_trace"] = json!("ABSENT");
                v
            },
            SchemaIssueCode::MissingTraceReference,
        ),
        (
            {
                let mut v = base.clone();
                v["traces"][0]["joins"][0]["source_fields"] = json!(["A"]);
                v
            },
            SchemaIssueCode::InvalidSchemaValue,
        ),
        (
            {
                let mut v = base.clone();
                v["traces"][0]["joins"][0]["source_fields"] = json!(["A", "ABSENT"]);
                v
            },
            SchemaIssueCode::MissingFieldReference,
        ),
        (
            {
                let mut v = base.clone();
                v["traces"][0]["joins"][0]["cardinality"] = json!("exactly_one");
                v
            },
            SchemaIssueCode::InvalidSchemaValue,
        ),
        (
            {
                let mut v = base.clone();
                v["traces"][0]["joins"][0]["on_missing_record"]["error_code"] =
                    json!("E_CROSS_FIELD");
                v
            },
            SchemaIssueCode::InvalidSchemaValue,
        ),
        (
            {
                let mut v = base.clone();
                v["required_capabilities"] = json!(["local.v1"]);
                v
            },
            SchemaIssueCode::MissingCapability,
        ),
        (
            {
                let mut v = base.clone();
                v["traces"][0]["joins"][0]["namespace_parameters"] = json!(["unit"]);
                v
            },
            SchemaIssueCode::MissingParameterReference,
        ),
        (
            {
                let mut v = base.clone();
                v["traces"][0]["joins"][0]["scope"] = json!("history_including_current");
                v
            },
            SchemaIssueCode::UnsupportedConstruct,
        ),
    ] {
        assert!(
            compile(value.take())
                .unwrap_err()
                .issues
                .iter()
                .any(|issue| issue.code == code)
        );
    }
    for property in ["cardinality", "on_missing_record", "on_missing_trace"] {
        let mut value = base.clone();
        value["traces"][0]["joins"][0]
            .as_object_mut()
            .unwrap()
            .remove(property);
        assert!(serde_json::from_value::<RawFlowSchema>(value).is_err());
    }
}

#[test]
fn namespace_and_consensus_definitions_are_preserved_without_runtime_defaults() {
    let mut value = sample();
    value["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!("joins.consensus.v1"));
    value["parameters"] = json!([{"type":"string","id":"unit","allowed_values":["001","002"]},{"type":"integer","id":"year","min":1900,"max":9999}]);
    let mut join = relation("zero_or_many", "unknown");
    join["namespace_parameters"] = json!(["year", "unit"]);
    join["reduction"] = json!({"kind":"consensus","fields":["B","A"],"include_if":{"const":true},"on_conflict":"unknown","diagnostic":{"code":"SYNTHETIC_CONFLICT","message_template":"Conflicting synthetic tuples"}});
    value["traces"][0]["joins"] = json!([join]);
    let schema = compile(value.clone()).unwrap();
    let plan = &schema.cross_record_plan().joins[0];
    assert_eq!(plan.namespace_parameters, [1, 0]);
    assert_eq!(schema.parameters().len(), 2);
    match &schema.parameters()[0] {
        ParameterDefinition::String { id, allowed_values } => {
            assert_eq!(id.as_ref(), "unit");
            assert_eq!(
                allowed_values
                    .iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<_>>(),
                ["001", "002"]
            );
        }
        _ => panic!("string definition lost"),
    }
    let consensus = plan.consensus.as_ref().unwrap();
    assert_eq!(consensus.fields, [1, 0]);
    assert_eq!(
        consensus.conflict_diagnostic.code.as_ref(),
        "SYNTHETIC_CONFLICT"
    );
    value["parameters"][1]["min"] = json!(10000);
    assert!(
        compile(value)
            .unwrap_err()
            .issues
            .iter()
            .any(|i| i.code == SchemaIssueCode::InvalidSchemaValue)
    );
}
