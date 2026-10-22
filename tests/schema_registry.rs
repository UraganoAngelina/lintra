use lintra::{
    EncodingSpec, FlowId, InMemorySchemaRepository, RawFlowSchema, RawResolvedEncoding,
    SchemaCompilationError, SchemaCompiler, SchemaIssueCode, SchemaKey, SchemaLoadError,
    SchemaRegistry, SchemaRepository, SchemaResolutionError, SchemaVersion,
};
use serde_json::json;
use std::{
    error::Error,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

fn key(version: &str) -> SchemaKey {
    SchemaKey {
        flow_id: FlowId("EXAMPLE-REGISTRY".into()),
        version: SchemaVersion(version.into()),
    }
}
fn raw(version: &str) -> RawFlowSchema {
    serde_json::from_value(json!({
        "schema_format_version":1,"flow_id":"EXAMPLE-REGISTRY","schema_version":version,
        "regulatory_version":"synthetic-only", "source":{"note":"Synthetic configuration"},
        "encoding":{"mode":"runtime_required","position_unit":"byte"},
        "required_capabilities":["local.v1"],"parameters":[],"reference_data":[],"persistence_policy":null,
        "traces":[{"id":"T","name":"Synthetic trace","record_length":3,
            "fields":[{"id":"ID","name":"Synthetic identifier","physical":{
                "type":"alphanumeric","start":1,"length":3,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}},
                "missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]}],
            "record_key":{"fields":["ID"],"uniqueness_scope":"trace","error_code":"E_DUPLICATE_RECORD_KEY"},
            "joins":[],"identity":[],"named_conditions":{},"derived_values":{},"repeated_groups":[],"rules":[],"documented_non_executable_rules":[]
        }]
    })).unwrap()
}
fn memory() -> Arc<InMemorySchemaRepository> {
    let mut repository = InMemorySchemaRepository::new();
    repository.insert(raw("v1")).unwrap();
    repository.insert(raw("v2")).unwrap();
    Arc::new(repository)
}

#[test]
fn resolves_exact_versions_and_returns_distinct_shared_compiled_values() {
    let repository = memory();
    let registry = SchemaRegistry::new(
        repository.clone(),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    );
    let first = registry.resolve(&key("v1")).unwrap();
    let again = registry.resolve(&key("v1")).unwrap();
    let second = registry.resolve(&key("v2")).unwrap();
    assert_eq!(first.key(), &key("v1"));
    assert_eq!(second.key(), &key("v2"));
    assert!(!Arc::ptr_eq(&first, &again));
    assert_eq!(first.encoding(), &EncodingSpec::Utf8);
    assert_eq!(
        first.traces()[0].fields()[0].range().slice(b"ABC"),
        Some(&b"ABC"[..])
    );
    assert_eq!(first.traces()[0].record_key().field_indexes, [0]);
    assert!(
        repository
            .load_raw(&key("v1"))
            .unwrap()
            .encoding
            .resolved
            .is_none()
    );
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SchemaRegistry>();
}

#[test]
fn missing_version_is_a_load_failure_without_latest_or_fallback() {
    let registry = SchemaRegistry::new(memory(), SchemaCompiler, RawResolvedEncoding::Utf8);
    let error = registry.resolve(&key("v3")).unwrap_err();
    assert!(
        matches!(&error,SchemaResolutionError::Load(SchemaLoadError::NotFound {key:missing}) if missing==&key("v3"))
    );
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<SchemaLoadError>()
            .is_some()
    );
    assert!(error.to_string().starts_with("schema loading failed:"));
}

#[test]
fn compiler_failure_preserves_aggregated_issues_and_technical_source() {
    let mut invalid = raw("v1");
    invalid.schema_format_version = 3;
    invalid.traces[0].fields[0].name.clear();
    let mut repository = InMemorySchemaRepository::new();
    repository.insert(invalid).unwrap();
    let registry = SchemaRegistry::new(
        Arc::new(repository),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    );
    let error = registry.resolve(&key("v1")).unwrap_err();
    let SchemaResolutionError::Compilation(failure) = &error else {
        panic!("expected compiler failure")
    };
    assert!(
        failure
            .issues
            .iter()
            .any(|i| i.code == SchemaIssueCode::UnsupportedSchemaFormat)
    );
    assert!(
        failure
            .issues
            .iter()
            .any(|i| i.code == SchemaIssueCode::InvalidSchemaValue)
    );
    assert_eq!(
        error
            .source()
            .unwrap()
            .downcast_ref::<SchemaCompilationError>()
            .unwrap(),
        failure
    );
}

#[test]
fn caller_encoding_is_explicit_and_unsupported_encoding_does_not_fall_back() {
    let registry = SchemaRegistry::new(memory(), SchemaCompiler, RawResolvedEncoding::Windows1252);
    let SchemaResolutionError::Compilation(error) = registry.resolve(&key("v1")).unwrap_err()
    else {
        panic!("expected compiler failure")
    };
    assert_eq!(error.issues.len(), 1);
    assert_eq!(error.issues[0].code, SchemaIssueCode::UnsupportedEncoding);
    // An incidental in-memory repository binding does not override run configuration.
    let mut source = raw("v1");
    source.encoding.resolved = Some(RawResolvedEncoding::Windows1252);
    let mut repository = InMemorySchemaRepository::new();
    repository.insert(source).unwrap();
    let registry = SchemaRegistry::new(
        Arc::new(repository),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    );
    assert_eq!(
        registry.resolve(&key("v1")).unwrap().encoding(),
        &EncodingSpec::Utf8
    );
}

struct ChangingRepository {
    value: Mutex<RawFlowSchema>,
    loads: AtomicUsize,
}
impl SchemaRepository for ChangingRepository {
    fn load_raw(&self, _: &SchemaKey) -> Result<RawFlowSchema, SchemaLoadError> {
        self.loads.fetch_add(1, Ordering::SeqCst);
        Ok(self.value.lock().unwrap().clone())
    }
}
#[test]
fn each_resolution_reloads_and_recompiles_including_changes_and_later_failures() {
    let repository = Arc::new(ChangingRepository {
        value: Mutex::new(raw("v1")),
        loads: AtomicUsize::new(0),
    });
    let registry = SchemaRegistry::new(
        repository.clone(),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    );
    let first = registry.resolve(&key("v1")).unwrap();
    repository.value.lock().unwrap().traces[0].name = "Updated synthetic trace".into();
    let second = registry.resolve(&key("v1")).unwrap();
    assert_eq!(first.traces()[0].name(), "Synthetic trace");
    assert_eq!(second.traces()[0].name(), "Updated synthetic trace");
    repository.value.lock().unwrap().schema_format_version = 3;
    assert!(matches!(
        registry.resolve(&key("v1")),
        Err(SchemaResolutionError::Compilation(_))
    ));
    assert_eq!(repository.loads.load(Ordering::SeqCst), 3);
}

#[test]
fn custom_repository_cannot_substitute_another_schema_key() {
    let repository = Arc::new(ChangingRepository {
        value: Mutex::new(raw("v2")),
        loads: AtomicUsize::new(0),
    });
    let registry = SchemaRegistry::new(repository, SchemaCompiler, RawResolvedEncoding::Utf8);
    let error = registry.resolve(&key("v1")).unwrap_err();
    assert!(
        matches!(error,SchemaResolutionError::Load(SchemaLoadError::KeyMismatch {requested,found}) if requested==key("v1") && found==key("v2"))
    );
}
