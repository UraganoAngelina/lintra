use lintra::*;
use serde_json::{Value, json};
use std::sync::Arc;

fn raw() -> RawFlowSchema {
    serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap()
}
/// Reverse every object key order in actual JSON text, preserving arrays and
/// exact scalar literals. Tests real parsing rather than an already sorted map.
fn reversed(value: &Value) -> String {
    match value {
        Value::Object(map) => format!(
            "{{{}}}",
            map.iter()
                .rev()
                .map(|(k, v)| format!("{}:{}", serde_json::to_string(k).unwrap(), reversed(v)))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Array(a) => format!("[{}]", a.iter().map(reversed).collect::<Vec<_>>().join(",")),
        other => serde_json::to_string(other).unwrap(),
    }
}
#[test]
fn canonical_bytes_and_sha_match_independent_python_oracle() {
    let raw = raw();
    assert_eq!(
        canonical_schema_bytes(&raw).unwrap(),
        include_bytes!("fixtures/canonical-schema.json")
    );
    assert_eq!(
        schema_fingerprint(&raw).unwrap().to_string(),
        include_str!("fixtures/canonical-schema.sha256").trim()
    );
    let decoded: RawFlowSchema =
        serde_json::from_slice(&canonical_schema_bytes(&raw).unwrap()).unwrap();
    assert_eq!(decoded, raw);
}
#[test]
fn shuffled_object_keys_and_whitespace_have_identical_fingerprints() {
    let original = raw();
    let value = serde_json::to_value(&original).unwrap();
    let shuffled: RawFlowSchema =
        serde_json::from_str(&format!("\n {} \n", reversed(&value))).unwrap();
    assert_eq!(
        schema_fingerprint(&original).unwrap(),
        schema_fingerprint(&shuffled).unwrap()
    );
    assert_eq!(
        canonical_schema_bytes(&original).unwrap(),
        canonical_schema_bytes(&shuffled).unwrap()
    );
}
#[test]
fn changes_and_array_order_are_significant_but_runtime_binding_is_separate() {
    let mut raw = raw();
    let before = schema_fingerprint(&raw).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    assert_eq!(before, schema_fingerprint(&raw).unwrap());
    raw.source.insert("ordered".into(), json!(["a", "b"]));
    let first = schema_fingerprint(&raw).unwrap();
    raw.source.insert("ordered".into(), json!(["b", "a"]));
    assert_ne!(first, schema_fingerprint(&raw).unwrap());
    assert_ne!(before, first);
}
#[test]
fn canonicalization_preserves_large_numbers_and_significant_text_bytes() {
    let mut raw = raw();
    let number = "12345678901234567890123456789012345678901234567890";
    raw.source
        .insert("large".into(), serde_json::from_str(number).unwrap());
    raw.source.insert("text".into(), json!(" 000é "));
    let bytes = canonical_schema_bytes(&raw).unwrap();
    let decoded: RawFlowSchema = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(decoded, raw);
    assert!(std::str::from_utf8(&bytes).unwrap().contains(number));
}
#[test]
fn registry_fingerprint_is_bound_to_the_actual_loaded_schema() {
    let raw = raw();
    let expected = schema_fingerprint(&raw).unwrap();
    let key = SchemaKey {
        flow_id: FlowId(raw.flow_id.clone()),
        version: SchemaVersion(raw.schema_version.clone()),
    };
    let mut repo = InMemorySchemaRepository::new();
    repo.insert(raw).unwrap();
    let registry = SchemaRegistry::new(Arc::new(repo), SchemaCompiler, RawResolvedEncoding::Utf8);
    let resolved = registry.resolve_with_hash(&key).unwrap();
    assert_eq!(resolved.schema_hash(), &expected);
    assert_eq!(resolved.schema().key(), &key);
    assert_eq!(registry.resolve(&key).unwrap().key(), &key);
}
