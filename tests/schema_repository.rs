//! Repository I/O with synthetic schemas only; no compiler or normative validation.
use lintra::{
    FileSystemSchemaRepository, FlowId, InMemorySchemaRepository, RawFlowSchema, SchemaKey,
    SchemaKeyComponent, SchemaKeyError, SchemaLoadError, SchemaRepository, SchemaVersion,
};
use serde_json::{Value, json};
use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const SOURCE: &str = include_str!("../docs/schema-authoring/examples/schemas/local-flow.json");
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TempDirectory(PathBuf);
impl TempDirectory {
    fn new() -> io::Result<Self> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        loop {
            let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "lintra schema repository à {} {now} {sequence}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
    }
}
impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn key(version: &str) -> SchemaKey {
    SchemaKey {
        flow_id: FlowId("EXAMPLE-LOCAL".into()),
        version: SchemaVersion(version.into()),
    }
}
fn raw(key: &SchemaKey) -> RawFlowSchema {
    let mut raw: RawFlowSchema = serde_json::from_str(SOURCE).unwrap();
    raw.flow_id.clone_from(&key.flow_id.0);
    raw.schema_version.clone_from(&key.version.0);
    raw
}
// Independent fixture layout from ARCH §12, not the implementation's path helper.
fn write_fixture(
    root: &Path,
    key: &SchemaKey,
    filename: &str,
    bytes: &[u8],
) -> io::Result<PathBuf> {
    let directory = root.join(&key.flow_id.0).join(&key.version.0);
    fs::create_dir_all(&directory)?;
    let path = directory.join(filename);
    fs::write(&path, bytes)?;
    Ok(path)
}

#[test]
fn filesystem_loads_exact_version_under_path_with_spaces_and_unicode() {
    let directory = TempDirectory::new().unwrap();
    let key = key("v1");
    let expected = raw(&key);
    let bytes = serde_json::to_vec(&expected).unwrap();
    let path = write_fixture(&directory.0, &key, "flow.json", &bytes).unwrap();
    let repository = FileSystemSchemaRepository::new(directory.0.clone()).unwrap();
    assert_eq!(repository.root(), directory.0);
    assert_eq!(repository.schema_path(&key).unwrap(), path);
    let port: &dyn SchemaRepository = &repository;
    let loaded = port.load_raw(&key).unwrap();
    assert_eq!(
        serde_json::to_value(loaded).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
}

#[test]
fn missing_or_empty_version_never_selects_a_newer_version_or_alternate_filename() {
    let directory = TempDirectory::new().unwrap();
    let repository = FileSystemSchemaRepository::new(directory.0.clone()).unwrap();
    write_fixture(
        &directory.0,
        &key("v2"),
        "flow.json",
        &serde_json::to_vec(&raw(&key("v2"))).unwrap(),
    )
    .unwrap();
    write_fixture(
        &directory.0,
        &key("v1"),
        "flow_schema.json",
        &serde_json::to_vec(&raw(&key("v1"))).unwrap(),
    )
    .unwrap();
    assert!(
        matches!(repository.load_raw(&key("v1")), Err(SchemaLoadError::NotFound { key: missing }) if missing == key("v1"))
    );
    assert!(matches!(
        repository.load_raw(&key("")),
        Err(SchemaLoadError::InvalidKey {
            component: SchemaKeyComponent::Version,
            reason: SchemaKeyError::Empty
        })
    ));
    assert_eq!(
        repository.load_raw(&key("v2")).unwrap().schema_version,
        "v2"
    );
}

#[test]
fn memory_is_exact_keyed_rejects_duplicates_and_returns_independent_values() {
    let mut repository = InMemorySchemaRepository::new();
    repository.insert(raw(&key("v1"))).unwrap();
    repository.insert(raw(&key("v2"))).unwrap();
    assert!(matches!(
        repository.insert(raw(&key("v1"))),
        Err(SchemaLoadError::DuplicateKey { .. })
    ));
    assert_eq!(
        repository.load_raw(&key("v1")).unwrap().schema_version,
        "v1"
    );
    let mut loaded = repository.load_raw(&key("v1")).unwrap();
    loaded.traces[0].name = "modified copy".into();
    assert_ne!(
        repository.load_raw(&key("v1")).unwrap().traces[0].name,
        loaded.traces[0].name
    );
    let mut different_case = key("v1");
    different_case.flow_id.0.make_ascii_lowercase();
    assert!(matches!(
        repository.load_raw(&different_case),
        Err(SchemaLoadError::NotFound { .. })
    ));
    assert!(matches!(
        repository.load_raw(&key("v3")),
        Err(SchemaLoadError::NotFound { .. })
    ));
    assert!(matches!(
        repository.load_raw(&key("")),
        Err(SchemaLoadError::InvalidKey { .. })
    ));
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<FileSystemSchemaRepository>();
    assert_send_sync::<InMemorySchemaRepository>();
}

#[test]
fn loaded_file_must_identify_the_requested_flow_and_version_exactly() {
    let directory = TempDirectory::new().unwrap();
    let repository = FileSystemSchemaRepository::new(directory.0.clone()).unwrap();
    for found in [
        key("v2"),
        SchemaKey {
            flow_id: FlowId("example-local".into()),
            version: SchemaVersion("v1".into()),
        },
    ] {
        write_fixture(
            &directory.0,
            &key("v1"),
            "flow.json",
            &serde_json::to_vec(&raw(&found)).unwrap(),
        )
        .unwrap();
        assert!(
            matches!(repository.load_raw(&key("v1")), Err(SchemaLoadError::KeyMismatch { requested, found: actual }) if requested == key("v1") && actual == found)
        );
    }
}

#[test]
fn malformed_json_unknown_properties_and_nested_duplicates_are_loading_errors() {
    let directory = TempDirectory::new().unwrap();
    let repository = FileSystemSchemaRepository::new(directory.0.clone()).unwrap();
    let mut value = serde_json::to_value(raw(&key("v1"))).unwrap();
    value["unexpected_property"] = json!(true);
    let unknown = serde_json::to_vec(&value).unwrap();
    let mut value = serde_json::to_value(raw(&key("v1"))).unwrap();
    value["source"]["synthetic"] = json!({"nested": [{"marker": 0}]});
    let valid = serde_json::to_string(&value).unwrap();
    let duplicate = valid.replace("\"marker\":0", "\"marker\":0,\"marker\":1");
    assert_ne!(duplicate, valid);
    for bytes in [
        b"{".as_slice(),
        b"{}",
        &[0xff, 0x00],
        &unknown,
        duplicate.as_bytes(),
        format!("{valid} trailing").as_bytes(),
    ] {
        write_fixture(&directory.0, &key("v1"), "flow.json", bytes).unwrap();
        let error = repository.load_raw(&key("v1")).unwrap_err();
        assert!(matches!(error, SchemaLoadError::Json { .. }));
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<serde_json::Error>()
                .is_some()
        );
    }
}

#[test]
fn io_failure_preserves_its_cause_and_empty_root_has_no_cwd_default() {
    let directory = TempDirectory::new().unwrap();
    let repository = FileSystemSchemaRepository::new(directory.0.clone()).unwrap();
    let path = directory
        .0
        .join("EXAMPLE-LOCAL")
        .join("v1")
        .join("flow.json");
    fs::create_dir_all(path).unwrap();
    let error = repository.load_raw(&key("v1")).unwrap_err();
    assert!(matches!(error, SchemaLoadError::Io { .. }));
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<io::Error>()
            .is_some()
    );
    assert!(matches!(
        FileSystemSchemaRepository::new(PathBuf::new()),
        Err(SchemaLoadError::EmptyRoot)
    ));
}

#[test]
fn loading_preserves_semantic_issues_and_large_integer_literals_for_the_compiler() {
    let directory = TempDirectory::new().unwrap();
    let repository = FileSystemSchemaRepository::new(directory.0.clone()).unwrap();
    let mut value = serde_json::to_value(raw(&key("v1"))).unwrap();
    value["traces"][0]["fields"][0]["physical"]["start"] = json!(0);
    value["traces"][0]["named_conditions"]["unresolved"] =
        json!({"is_present": {"ref": {"field": "MISSING-FIELD"}}});
    let huge = "123456789012345678901234567890123456789012345678901234567890";
    value["source"]["large_integer"] = serde_json::from_str::<Value>(huge).unwrap();
    write_fixture(
        &directory.0,
        &key("v1"),
        "flow.json",
        &serde_json::to_vec(&value).unwrap(),
    )
    .unwrap();
    let loaded = repository.load_raw(&key("v1")).unwrap();
    assert_eq!(serde_json::to_value(&loaded).unwrap(), value);
    assert_eq!(loaded.source["large_integer"].to_string(), huge);
    let mut memory = InMemorySchemaRepository::new();
    memory.insert(loaded).unwrap();
    assert_eq!(
        serde_json::to_value(memory.load_raw(&key("v1")).unwrap()).unwrap(),
        value
    );
}

#[test]
fn selectors_cannot_be_paths_and_unicode_spelling_is_preserved() {
    let repository = FileSystemSchemaRepository::new(PathBuf::from("explicit root")).unwrap();
    for invalid in [
        "",
        ".",
        "..",
        "../other",
        "..\\other",
        "/absolute",
        "C:relative",
        "C:\\absolute",
        "\\\\server\\share",
        "bad/child",
        "bad\\child",
        "bad\0",
        "bad\n",
        "bad.",
        "bad ",
        "bad?",
        "bad*",
        "bad|",
        "bad\"",
        "<bad>",
        "CON",
        "NUL.txt",
        "COM1",
        "COM³.txt",
        "LPT9",
        "con .txt",
    ] {
        for invalid_key in [
            SchemaKey {
                flow_id: FlowId(invalid.into()),
                ..key("v1")
            },
            key(invalid),
        ] {
            assert!(
                matches!(
                    repository.schema_path(&invalid_key),
                    Err(SchemaLoadError::InvalidKey { .. })
                ),
                "{invalid:?}"
            );
        }
    }
    let directory = TempDirectory::new().unwrap();
    let repository = FileSystemSchemaRepository::new(directory.0.clone()).unwrap();
    let unicode_key = SchemaKey {
        flow_id: FlowId("ÉTÉ".into()),
        version: SchemaVersion("révision 1".into()),
    };
    write_fixture(
        &directory.0,
        &unicode_key,
        "flow.json",
        &serde_json::to_vec(&raw(&unicode_key)).unwrap(),
    )
    .unwrap();
    assert_eq!(repository.load_raw(&unicode_key).unwrap().flow_id, "ÉTÉ");
}

#[test]
fn sioc_draft_loads_when_explicitly_staged_in_the_registry_layout() {
    // This is configuration data, not a patient record or a normative validation run.
    let bytes = include_bytes!("../schemas/sioc/2026-04-20.structural-minimal.1/flow_schema.json");
    let expected: RawFlowSchema = serde_json::from_slice(bytes).unwrap();
    let selected = SchemaKey {
        flow_id: FlowId(expected.flow_id.clone()),
        version: SchemaVersion(expected.schema_version.clone()),
    };
    let directory = TempDirectory::new().unwrap();
    write_fixture(&directory.0, &selected, "flow.json", bytes).unwrap();
    let repository = FileSystemSchemaRepository::new(directory.0.clone()).unwrap();
    let loaded = repository.load_raw(&selected).unwrap();
    assert_eq!(loaded.traces.len(), 5);
    assert_eq!(
        loaded
            .traces
            .iter()
            .map(|trace| trace.fields.len())
            .sum::<usize>(),
        135
    );
    assert_eq!(
        serde_json::to_value(loaded).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
}

#[test]
fn schema_format_is_required_and_never_defaulted_or_coerced_during_loading() {
    let directory = TempDirectory::new().unwrap();
    let repository = FileSystemSchemaRepository::new(directory.0.clone()).unwrap();
    let selection = key("v1");
    for invalid in [
        None,
        Some(json!(null)),
        Some(json!("1")),
        Some(json!("1.0.0")),
        Some(json!(-1)),
        Some(json!(1.5)),
        Some(json!(4294967296_u64)),
        Some(json!(true)),
    ] {
        let mut value = serde_json::to_value(raw(&selection)).unwrap();
        match invalid {
            Some(version) => {
                value["schema_format_version"] = version;
            }
            None => {
                value
                    .as_object_mut()
                    .unwrap()
                    .remove("schema_format_version");
            }
        }
        write_fixture(
            &directory.0,
            &selection,
            "flow.json",
            &serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        let error = repository.load_raw(&selection).unwrap_err();
        assert!(matches!(error,SchemaLoadError::Json {ref source,..} if source.is_data()));
    }
    // Repository remains a deserializer; numeric compatibility belongs to lint/compiler.
    for version in [0, 1, 2, u32::MAX] {
        let mut value = raw(&selection);
        value.schema_format_version = version;
        write_fixture(
            &directory.0,
            &selection,
            "flow.json",
            &serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        assert_eq!(
            repository
                .load_raw(&selection)
                .unwrap()
                .schema_format_version,
            version
        );
    }
}
