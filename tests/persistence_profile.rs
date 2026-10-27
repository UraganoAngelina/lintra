//! Declaration compilation only. No backend, patient data or historical outcome.
use lintra::*;
use serde_json::{Value, json};

fn model() -> Value {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    value["parameters"] =
        json!([{"id":"organization","type":"string","allowed_values":["SYNTHETIC-UNIT"]}]);
    value["required_capabilities"] = json!(["local.v1", "persistence.audit.v1"]);
    value["persistence_policy"] = json!({
        "namespace_parameters":["organization"],
        "version_compatibility":{"kind":"same_schema_version"},
        "run_uniqueness":[{"trace":"T1","key":"record_key"}],
        "cross_run_audit":[{"trace":"T1","key":"record_key"}],
        "history_identity_invariants":[],"snapshot":null,"temporal_rules":[],
        "storage_protection":"external_secure_configuration_required"
    });
    value
}
fn raw(value: Value) -> RawFlowSchema {
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    raw
}
fn code(value: Value, expected: SchemaIssueCode) {
    let error = SchemaCompiler
        .compile_persistence_profile(&raw(value), &[])
        .unwrap_err();
    assert!(error.issues.iter().any(|i| i.code == expected), "{error:?}");
}

#[test]
fn compile_explicit_namespace_pk_order_and_audit_without_enabling_execution() {
    let raw = raw(model());
    let profile = SchemaCompiler
        .compile_persistence_profile(&raw, &[])
        .unwrap();
    assert_eq!(
        profile.schema_key(),
        &SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into())
        }
    );
    assert_eq!(profile.schema_hash(), &schema_fingerprint(&raw).unwrap());
    assert_eq!(profile.parameters().len(), 1);
    let policy = profile.policy().unwrap();
    assert_eq!(policy.namespace_parameters, [0]);
    assert_eq!(policy.run_uniqueness[0].trace_index, 0);
    assert_eq!(policy.cross_run_audit[0].trace_index, 0);
    assert_eq!(profile.record_key_fields(0), Some([0].as_slice()));
    assert!(matches!(
        policy.version_compatibility,
        VersionCompatibility::SameSchemaVersion
    ));
    assert_eq!(profile.compatible_schemas().len(), 1);
    assert!(policy.snapshot.is_none() && policy.temporal_rules.is_empty());
    // An inspection artifact cannot weaken the existing executable-schema gate.
    let rejected = SchemaCompiler.compile(raw).unwrap_err();
    assert!(
        rejected
            .issues
            .iter()
            .any(|i| i.code == SchemaIssueCode::UnsupportedCapability
                || i.code == SchemaIssueCode::UnsupportedConstruct)
    );
}

#[test]
fn disabled_policy_does_not_activate_persistence_or_remove_existing_pk() {
    let mut value = model();
    value["persistence_policy"] = Value::Null;
    value["required_capabilities"] = json!(["local.v1"]);
    let raw = raw(value);
    let profile = SchemaCompiler
        .compile_persistence_profile(&raw, &[])
        .unwrap();
    assert!(profile.policy().is_none());
    assert!(profile.compatible_schemas().is_empty());
    assert!(profile.record_key_fields(0).is_none());
    assert!(
        !SchemaCompiler.compile(raw).unwrap().traces()[0]
            .record_key()
            .field_indexes
            .is_empty()
    );
}

#[test]
fn reject_unknown_format_missing_namespace_unknown_refs_and_duplicates() {
    let mut value = model();
    value["schema_format_version"] = json!(999);
    code(value, SchemaIssueCode::UnsupportedSchemaFormat);
    let mut value = model();
    value["persistence_policy"]["namespace_parameters"] = json!([]);
    code(value, SchemaIssueCode::MissingParameterReference);
    let mut value = model();
    value["persistence_policy"]["namespace_parameters"] = json!(["undeclared"]);
    code(value, SchemaIssueCode::MissingParameterReference);
    let mut value = model();
    value["persistence_policy"]["namespace_parameters"] = json!(["organization", "organization"]);
    code(value, SchemaIssueCode::InvalidSchemaValue);
    let mut value = model();
    value["persistence_policy"]["run_uniqueness"][0]["trace"] = json!("absent");
    code(value, SchemaIssueCode::MissingTraceReference);
    let mut value = model();
    value["traces"][0]["record_key"]["fields"] = json!(["absent"]);
    code(value, SchemaIssueCode::MissingFieldReference);
    let mut value = model();
    value["traces"][0]["record_key"]["fields"] = json!([]);
    code(value, SchemaIssueCode::InvalidRecordKey);
    let mut value = model();
    value["traces"][0]["record_key"]["fields"] = json!(["F1", "F1"]);
    code(value, SchemaIssueCode::InvalidRecordKey);
    let mut value = model();
    value["persistence_policy"]["cross_run_audit"] =
        json!([{"trace":"T1","key":"record_key"},{"trace":"T1","key":"record_key"}]);
    code(value, SchemaIssueCode::InvalidSchemaValue);
    let mut value = model();
    value["required_capabilities"] = json!(["local.v1"]);
    code(value, SchemaIssueCode::MissingCapability);
}

fn group() -> Value {
    let mut value = model();
    value["persistence_policy"]["version_compatibility"] =
        json!({"kind":"declared_group","id":"synthetic-compatible","schema_versions":["v1","v2"]});
    value
}
fn other() -> RawFlowSchema {
    let mut value = model();
    value["schema_version"] = json!("v2");
    raw(value)
}

#[test]
fn group_compatibility_requires_actual_member_schemas_and_retains_their_hashes() {
    let current = raw(group());
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&current, &[])
            .is_err()
    );
    let member = other();
    let profile = SchemaCompiler
        .compile_persistence_profile(&current, std::slice::from_ref(&member))
        .unwrap();
    assert_eq!(
        profile
            .compatible_schemas()
            .iter()
            .find(|(version, _)| version.0 == "v2")
            .unwrap()
            .1,
        schema_fingerprint(&member).unwrap()
    );
    assert!(
        matches!(&profile.policy().unwrap().version_compatibility, VersionCompatibility::DeclaredGroup { id, schema_versions } if id.as_ref() == "synthetic-compatible" && schema_versions.len() == 2)
    );
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&current, &[member.clone(), member])
            .is_err()
    );
    let mut foreign = other();
    foreign.flow_id = "OTHER-SYNTHETIC".into();
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&current, &[foreign])
            .is_err()
    );
    let mut unknown = other();
    unknown.schema_version = "v3".into();
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&current, &[unknown])
            .is_err()
    );
}

#[test]
fn group_rejects_changed_key_logical_semantics_and_namespace_type() {
    let current = raw(group());
    let mut value = model();
    value["schema_version"] = json!("v2");
    value["traces"][0]["fields"][0]["missing_value"] = json!({"kind":"spaces"});
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&current, &[raw(value)])
            .is_err()
    );
    let mut value = model();
    value["schema_version"] = json!("v2");
    value["traces"][0]["record_length"] = json!(5);
    value["traces"][0]["fields"][0]["physical"]["length"] = json!(5);
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&current, &[raw(value)])
            .is_err()
    );
    let mut value = model();
    value["schema_version"] = json!("v2");
    value["parameters"] = json!([{"id":"organization","type":"integer","min":1,"max":9}]);
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&current, &[raw(value)])
            .is_err()
    );
}

#[test]
fn group_cannot_alias_a_different_or_disabled_namespace() {
    let current = raw(group());
    let mut value = model();
    value["schema_version"] = json!("v2");
    value["parameters"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"other-organization","type":"string","allowed_values":[]}));
    value["persistence_policy"]["namespace_parameters"] = json!(["other-organization"]);
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&current, &[raw(value)])
            .is_err()
    );
    let mut value = model();
    value["schema_version"] = json!("v2");
    value["persistence_policy"] = Value::Null;
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&current, &[raw(value)])
            .is_err()
    );
}

#[test]
fn key_may_move_in_a_version_without_changing_its_named_logical_projection() {
    let mut current = group();
    let mut first = current["traces"][0]["fields"][0].clone();
    first["physical"]["length"] = json!(3);
    let mut second = first.clone();
    second["id"] = json!("F2");
    second["physical"]["start"] = json!(4);
    current["traces"][0]["fields"] = json!([first.clone(), second.clone()]);
    let mut member = current.clone();
    member["schema_version"] = json!("v2");
    first["physical"]["start"] = json!(4);
    second["physical"]["start"] = json!(1);
    member["traces"][0]["fields"] = json!([second, first]);
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&raw(current), &[raw(member)])
            .is_ok()
    );
}

#[test]
fn unsupported_history_snapshot_temporal_features_are_not_silently_dropped() {
    let history: RawFlowSchema = serde_json::from_slice(include_bytes!(
        "../docs/schema-authoring/examples/schemas/history-flow.json"
    ))
    .unwrap();
    let error = SchemaCompiler
        .compile_persistence_profile(&history, &[])
        .unwrap_err();
    assert!(
        error
            .issues
            .iter()
            .any(|i| i.code == SchemaIssueCode::UnsupportedConstruct)
    );
    let mut value = model();
    value["persistence_policy"]["version_compatibility"] =
        json!({"kind":"declared_group","id":"synthetic","schema_versions":["v1","v1"]});
    code(value, SchemaIssueCode::InvalidSchemaValue);
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&raw(model()), &[other()])
            .is_err()
    );
}
