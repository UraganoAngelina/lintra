//! Declaration registration, not a normative CAD fixture or historical execution.
//! The synthetic CAD-labelled snapshot tests the same generic registry as SIOC.
use lintra::*;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

fn model() -> Value {
    let mut v: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    v["flow_id"] = json!("CAD");
    v["schema_version"] = json!("synthetic-registration-v1");
    v["source"] = json!({"scope":"synthetic-registry-test-only", "normative_profile":false});
    v["parameters"] = json!([
        {"id":"unit","type":"string","allowed_values":["SYNTHETIC-UNIT"]},
        {"id":"year","type":"integer","min":2020,"max":2100},
        {"id":"quarter","type":"integer","min":1,"max":4}
    ]);
    v["required_capabilities"] = json!(["local.v1", "persistence.snapshot.v1"]);
    v["persistence_policy"] = json!({
        "namespace_parameters":["unit"],"version_compatibility":{"kind":"same_schema_version"},
        "run_uniqueness":[],"cross_run_audit":[],"history_identity_invariants":[],"temporal_rules":[],
        "storage_protection":"external_secure_configuration_required",
        "snapshot":{"scope_parameters":["unit"],"period":{"year_parameter":"year","quarter_parameter":"quarter"},
            "approved_immutable":true,"require_explicit_approval":true,"commit":"atomic_idempotent",
            "missing_trace_policy":{"consent":"proceed_or_abort","warning_code":"W_SYNTHETIC_MISSING",
                "on_absent_trace":"preserve","present_trace_is_complete":true}}
    });
    v
}
fn raw(v: Value) -> RawFlowSchema {
    serde_json::from_value(v).unwrap()
}
fn key(r: &RawFlowSchema) -> SchemaKey {
    SchemaKey {
        flow_id: FlowId(r.flow_id.clone()),
        version: SchemaVersion(r.schema_version.clone()),
    }
}
fn registry(sources: Vec<RawFlowSchema>) -> SchemaRegistry {
    let mut repository = InMemorySchemaRepository::new();
    for r in sources {
        repository.insert(r).unwrap();
    }
    SchemaRegistry::new(
        Arc::new(repository),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    )
}

#[test]
fn snapshot_declarations_resolve_by_exact_key_without_becoming_executable() {
    let r = raw(model());
    let k = key(&r);
    let expected_hash = schema_fingerprint(&r).unwrap();
    let registry = registry(vec![r]);
    let plan = registry.resolve_persistence_profile(&k).unwrap();
    assert_eq!(plan.schema_key(), &k);
    assert_eq!(plan.schema_hash(), &expected_hash);
    let snapshot = plan.policy().unwrap().snapshot.as_ref().unwrap();
    assert_eq!(snapshot.scope_parameters, [0]);
    assert_eq!(snapshot.year_parameter, 1);
    assert_eq!(snapshot.quarter_parameter, 2);
    // A declaration artifact never substitutes for an executable schema.
    assert!(registry.resolve(&k).is_err());
    assert!(registry.resolve_historical_audit(&k).is_err());
}

#[test]
fn disabled_policy_stays_disabled_and_sioc_remains_audit_only() {
    let mut v = model();
    v["persistence_policy"] = Value::Null;
    v["required_capabilities"] = json!(["local.v1"]);
    let disabled = raw(v);
    let k = key(&disabled);
    assert!(
        registry(vec![disabled])
            .resolve_persistence_profile(&k)
            .unwrap()
            .policy()
            .is_none()
    );

    let sioc: RawFlowSchema = serde_json::from_slice(include_bytes!(
        "../schemas/sioc/2026-04-20.structural-minimal.2/flow_schema.json"
    ))
    .unwrap();
    let k = key(&sioc);
    let registry = registry(vec![sioc]);
    let plan = registry.resolve_persistence_profile(&k).unwrap();
    let policy = plan.policy().unwrap();
    assert_eq!(policy.cross_run_audit.len(), 5);
    assert!(policy.snapshot.is_none());
    assert!(policy.history_identity_invariants.is_empty());
    assert!(policy.temporal_rules.is_empty());
    assert!(
        registry
            .resolve_persistence_profile(&SchemaKey {
                flow_id: k.flow_id,
                version: SchemaVersion("missing-version".into())
            })
            .is_err()
    );
}

struct RecordingRepository {
    selected: RawFlowSchema,
    member: Option<RawFlowSchema>,
    reads: Arc<Mutex<Vec<SchemaKey>>>,
}
impl SchemaRepository for RecordingRepository {
    fn load_raw(&self, requested: &SchemaKey) -> Result<RawFlowSchema, SchemaLoadError> {
        self.reads.lock().unwrap().push(requested.clone());
        if requested.version.0 == self.selected.schema_version {
            Ok(self.selected.clone())
        } else {
            self.member
                .clone()
                .ok_or_else(|| SchemaLoadError::NotFound {
                    key: requested.clone(),
                })
        }
    }
}
fn recording(
    selected: RawFlowSchema,
    member: Option<RawFlowSchema>,
) -> (SchemaRegistry, Arc<Mutex<Vec<SchemaKey>>>) {
    let reads = Arc::new(Mutex::new(vec![]));
    (
        SchemaRegistry::new(
            Arc::new(RecordingRepository {
                selected,
                member,
                reads: reads.clone(),
            }),
            SchemaCompiler,
            RawResolvedEncoding::Utf8,
        ),
        reads,
    )
}
fn group() -> Value {
    let mut v = model();
    v["persistence_policy"]["version_compatibility"] = json!({
        "kind":"declared_group","id":"synthetic-registration",
        "schema_versions":["synthetic-registration-v1","synthetic-registration-v2"]
    });
    v
}

#[test]
fn declared_members_are_loaded_exactly_once_and_keep_their_fingerprints() {
    let current = raw(group());
    let k = key(&current);
    let mut v = model();
    v["schema_version"] = json!("synthetic-registration-v2");
    let member = raw(v);
    let member_key = key(&member);
    let member_hash = schema_fingerprint(&member).unwrap();
    let (registry, reads) = recording(current, Some(member));
    let profile = registry.resolve_persistence_profile(&k).unwrap();
    assert_eq!(*reads.lock().unwrap(), [k, member_key.clone()]);
    assert!(
        profile
            .compatible_schemas()
            .contains(&(member_key.version, member_hash))
    );
}

#[test]
fn custom_repository_cannot_substitute_selected_or_group_member_identity() {
    for wrong_flow in [true, false] {
        let current = raw(model());
        let requested = key(&current);
        let mut substituted = current;
        if wrong_flow {
            substituted.flow_id = "OTHER".into();
        } else {
            substituted.schema_version = "OTHER".into();
        }
        let (registry, _) = recording(substituted.clone(), Some(substituted));
        assert!(matches!(
            registry.resolve_persistence_profile(&requested),
            Err(SchemaResolutionError::Load(
                SchemaLoadError::KeyMismatch { .. }
            ))
        ));
    }
    let current = raw(group());
    let k = key(&current);
    let (registry, _) = recording(current, Some(raw(model())));
    assert!(matches!(
        registry.resolve_persistence_profile(&k),
        Err(SchemaResolutionError::Load(
            SchemaLoadError::KeyMismatch { .. }
        ))
    ));
}

#[test]
fn invalid_group_does_not_trigger_secondary_reads_and_missing_member_fails() {
    let mut v = group();
    v["schema_format_version"] = json!(999);
    let current = raw(v);
    let k = key(&current);
    let (registry, reads) = recording(current, None);
    assert!(matches!(
        registry.resolve_persistence_profile(&k),
        Err(SchemaResolutionError::Compilation(_))
    ));
    assert_eq!(reads.lock().unwrap().len(), 1);

    let current = raw(group());
    let k = key(&current);
    let (registry, _) = recording(current, None);
    assert!(matches!(
        registry.resolve_persistence_profile(&k),
        Err(SchemaResolutionError::Load(
            SchemaLoadError::NotFound { .. }
        ))
    ));
}

#[test]
fn unsupported_history_is_rejected_instead_of_losing_rules() {
    let r: RawFlowSchema = serde_json::from_slice(include_bytes!(
        "../docs/schema-authoring/examples/schemas/history-flow.json"
    ))
    .unwrap();
    let k = key(&r);
    match registry(vec![r]).resolve_persistence_profile(&k) {
        Err(SchemaResolutionError::Compilation(e)) => assert!(
            e.issues
                .iter()
                .any(|issue| issue.code == SchemaIssueCode::UnsupportedConstruct)
        ),
        _ => panic!("unsupported history must not resolve as a weakened profile"),
    }
}

#[test]
fn manifest_verified_filesystem_registration_rejects_changed_schema_bytes() {
    use sha2::{Digest, Sha256};
    use std::fs;
    let root = tempfile::tempdir().unwrap();
    let v = model();
    let r = raw(v.clone());
    let k = key(&r);
    let bytes = serde_json::to_vec(&v).unwrap();
    let path = format!("schemas/{}/{}/flow.json", k.flow_id.0, k.version.0);
    let full = root.path().join(&path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(&full, &bytes).unwrap();
    let manifest = BundleManifest::parse(
        &serde_json::to_vec(&json!({
            "bundle_version":"synthetic-cad-registration-only",
            "schema_format_version":1,"files":{path:Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect::<String>()}
        }))
        .unwrap(),
    )
    .unwrap();
    let repository = ManifestSchemaRepository::new(root.path().join("schemas"), manifest).unwrap();
    let registry = SchemaRegistry::new(
        Arc::new(repository),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    );
    let first = registry.resolve_persistence_profile(&k).unwrap();
    assert_eq!(first.schema_hash(), &schema_fingerprint(&r).unwrap());
    assert!(first.policy().unwrap().snapshot.is_some());
    // Even a parseable, semantically equivalent JSON whitespace change must fail
    // the manifest's byte identity before any compiled plan can be returned.
    fs::write(full, [&bytes[..], b"\n"].concat()).unwrap();
    assert!(matches!(
        registry.resolve_persistence_profile(&k),
        Err(SchemaResolutionError::Load(SchemaLoadError::Bundle(
            BundleError::Integrity(_)
        )))
    ));
}
