use super::*;
use serde_json::{Value, json};

const LOCAL: &str = include_str!("../../docs/schema-authoring/examples/schemas/local-flow.json");
const HISTORY: &str =
    include_str!("../../docs/schema-authoring/examples/schemas/history-flow.json");
const COMPATIBLE: &str =
    include_str!("../../docs/schema-authoring/examples/schemas/compatible-history-flow.json");
const SIOC: &str =
    include_str!("../../schemas/sioc/2026-04-20.structural-minimal.1/flow_schema.json");

#[test]
fn approved_examples_and_minimal_profile_roundtrip_without_loss() {
    for source in [LOCAL, HISTORY, COMPATIBLE, SIOC] {
        let raw: RawFlowSchema = serde_json::from_str(source).unwrap();
        assert_eq!(
            serde_json::to_value(raw).unwrap(),
            serde_json::from_str::<Value>(source).unwrap()
        );
    }
}

#[test]
fn rejects_unknown_operational_fields_at_each_object_boundary() {
    fn visit(v: &Value, pointer: String, out: &mut Vec<String>) {
        match v {
            Value::Object(map) => {
                // Metadata open; named_conditions/derived_values are maps of typed nodes.
                out.push(pointer.clone());
                for (k, child) in map {
                    if matches!(k.as_str(), "source" | "regulatory_metadata") {
                        continue;
                    }
                    let path = format!("{}/{}", pointer, k.replace('~', "~0").replace('/', "~1"));
                    if matches!(k.as_str(), "named_conditions" | "derived_values") {
                        for (key, entry) in child.as_object().unwrap() {
                            visit(
                                entry,
                                format!("{path}/{}", key.replace('~', "~0").replace('/', "~1")),
                                out,
                            );
                        }
                    } else {
                        visit(child, path, out);
                    }
                }
            }
            Value::Array(values) => {
                for (i, child) in values.iter().enumerate() {
                    visit(child, format!("{pointer}/{i}"), out);
                }
            }
            _ => (),
        }
    }
    for source in [LOCAL, HISTORY, COMPATIBLE] {
        let base: Value = serde_json::from_str(source).unwrap();
        let mut pointers = Vec::new();
        visit(&base, String::new(), &mut pointers);
        for pointer in &pointers {
            let mut invalid = base.clone();
            invalid
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unknown_operational_property".into(), json!(true));
            assert!(
                serde_json::from_value::<RawFlowSchema>(invalid).is_err(),
                "{pointer}"
            );
        }
        assert!(pointers.len() > 100);
    }
}

#[test]
fn required_nullable_fields_are_explicit_and_optional_fields_cannot_be_null() {
    let base: Value = serde_json::from_str(LOCAL).unwrap();
    let mut missing = base.clone();
    missing
        .as_object_mut()
        .unwrap()
        .remove("persistence_policy");
    assert!(serde_json::from_value::<RawFlowSchema>(missing).is_err());
    let mut missing = serde_json::from_str::<Value>(HISTORY).unwrap();
    missing["persistence_policy"]
        .as_object_mut()
        .unwrap()
        .remove("snapshot");
    assert!(serde_json::from_value::<RawFlowSchema>(missing).is_err());
    let mut invalid = base;
    invalid["traces"][0]["fields"][0]["regulatory_metadata"] = Value::Null;
    assert!(serde_json::from_value::<RawFlowSchema>(invalid).is_err());
}

#[test]
fn keeps_metadata_open_and_positions_one_based_without_interpreting_them() {
    let mut source: Value = serde_json::from_str(LOCAL).unwrap();
    source["source"]["future_metadata"] =
        json!({"operator": "metadata only", "values": [null, 1.25, true]});
    source["traces"][0]["fields"][0]["regulatory_metadata"] =
        json!({"source_note": "synthetic", "anything": {"x": false}});
    let raw: RawFlowSchema = serde_json::from_value(source.clone()).unwrap();
    let output = serde_json::to_value(raw).unwrap();
    assert_eq!(output, source);
    assert_eq!(output["traces"][0]["fields"][0]["physical"]["start"], 1);
}

#[test]
fn rejects_duplicate_named_nodes_and_preserves_large_integer_literals() {
    let duplicate = LOCAL.replace("\"named_conditions\": {", "\"named_conditions\": {\"duplicate\": {\"const\": true}, \"duplicate\": {\"const\": false},");
    assert_ne!(duplicate, LOCAL);
    assert!(serde_json::from_str::<RawFlowSchema>(&duplicate).is_err());
    let huge = "1234567890123456789012345678901234567890";
    let value: RawInteger = serde_json::from_str(huge).unwrap();
    assert_eq!(serde_json::to_string(&value).unwrap(), huge);
    for invalid in ["1.25", "1e3", "true", "null", "\"12\""] {
        assert!(
            serde_json::from_str::<RawInteger>(invalid).is_err(),
            "{invalid}"
        );
    }
}
