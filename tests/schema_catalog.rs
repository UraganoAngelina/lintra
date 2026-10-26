//! Catalog acquisition is deliberately weaker than compiler acceptance.
use lintra::*;
use std::{fs, path::Path};
fn raw(flow: &str, version: &str) -> RawFlowSchema {
    let mut raw: RawFlowSchema =
        serde_json::from_str(include_str!("fixtures/application/schema.json")).unwrap();
    raw.flow_id = flow.into();
    raw.schema_version = version.into();
    raw
}
fn install(root: &Path, raw: &RawFlowSchema) -> std::path::PathBuf {
    let directory = root.join(&raw.flow_id).join(&raw.schema_version);
    fs::create_dir_all(&directory).unwrap();
    let file = directory.join("flow.json");
    fs::write(&file, serde_json::to_vec(raw).unwrap()).unwrap();
    file
}
#[test]
fn empty_missing_and_non_directory_roots_are_distinct() {
    let root = tempfile::tempdir().unwrap();
    let repo = FileSystemSchemaRepository::new(root.path().into()).unwrap();
    assert!(repo.list_schemas().unwrap().is_empty());
    assert!(matches!(
        FileSystemSchemaRepository::new(root.path().join("missing"))
            .unwrap()
            .list_schemas(),
        Err(SchemaCatalogError::Io { .. })
    ));
    fs::write(root.path().join("file"), b"x").unwrap();
    assert!(
        FileSystemSchemaRepository::new(root.path().join("file"))
            .unwrap()
            .list_schemas()
            .is_err()
    );
}
#[test]
fn filesystem_and_memory_match_exact_utf8_order_and_preserve_trace_order() {
    let root = tempfile::tempdir().unwrap();
    let mut memory = InMemorySchemaRepository::new();
    for (flow, version) in [
        ("à", "v1"),
        ("A", "v2"),
        ("b", "v1"),
        ("A", "v10"),
        ("A", "v1"),
    ] {
        let mut raw = raw(flow, version);
        let mut second = raw.traces[0].clone();
        second.id = "A".into();
        raw.traces[0].id = "Z".into();
        raw.traces.push(second);
        install(root.path(), &raw);
        memory.insert(raw).unwrap();
    }
    let repo = FileSystemSchemaRepository::new(root.path().into()).unwrap();
    let entries = repo.list_schemas().unwrap();
    assert_eq!(entries, memory.list_schemas().unwrap());
    assert_eq!(
        entries
            .iter()
            .map(|e| (e.key.flow_id.0.as_str(), e.key.version.0.as_str()))
            .collect::<Vec<_>>(),
        [
            ("A", "v1"),
            ("A", "v10"),
            ("A", "v2"),
            ("b", "v1"),
            ("à", "v1")
        ]
    );
    assert!(
        entries
            .iter()
            .all(|e| e.traces == [TraceId("Z".into()), TraceId("A".into())])
    );
}
#[test]
fn foreign_files_and_authoring_schemas_are_not_installed_implicitly() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("README.txt"), b"not json").unwrap();
    fs::create_dir_all(root.path().join("flow/version/deep")).unwrap();
    fs::write(
        root.path().join("flow/version/flow_schema.json"),
        b"not json",
    )
    .unwrap();
    install(&root.path().join("flow/version/deep"), &raw("A", "v1"));
    assert!(
        FileSystemSchemaRepository::new(root.path().into())
            .unwrap()
            .list_schemas()
            .unwrap()
            .is_empty()
    );
}
#[test]
fn non_regular_candidate_fails_before_attempting_to_read_it() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("A/v1/flow.json")).unwrap();
    assert!(matches!(
        FileSystemSchemaRepository::new(root.path().into())
            .unwrap()
            .list_schemas(),
        Err(SchemaCatalogError::Io { .. })
    ));
}
#[test]
fn relevant_invalid_candidates_fail_in_sorted_order_without_omission() {
    let root = tempfile::tempdir().unwrap();
    let b = install(root.path(), &raw("B", "v1"));
    let a = install(root.path(), &raw("A", "v1"));
    fs::write(b, b"bad B").unwrap();
    fs::write(&a, b"bad A").unwrap();
    match FileSystemSchemaRepository::new(root.path().into())
        .unwrap()
        .list_schemas()
    {
        // Catalog diagnostics use its canonical root. Windows canonicalize adds
        // the extended-length prefix; compare the same physical candidate in
        // that representation while still requiring A to fail before B.
        Err(SchemaCatalogError::Load(SchemaLoadError::Json { path, .. })) => {
            assert_eq!(path, fs::canonicalize(a).unwrap());
        }
        other => panic!("unexpected {other:?}"),
    }
}
#[test]
fn identity_mismatch_and_duplicate_json_properties_are_rejected_by_both_paths() {
    let root = tempfile::tempdir().unwrap();
    let raw = raw("A", "v1");
    let path = install(root.path(), &raw);
    let repo = FileSystemSchemaRepository::new(root.path().into()).unwrap();
    let key = SchemaKey {
        flow_id: FlowId("A".into()),
        version: SchemaVersion("v1".into()),
    };
    fs::write(&path, serde_json::to_vec(&self::raw("B", "v1")).unwrap()).unwrap();
    assert!(matches!(
        repo.load_raw(&key),
        Err(SchemaLoadError::KeyMismatch { .. })
    ));
    assert!(matches!(
        repo.list_schemas(),
        Err(SchemaCatalogError::Load(
            SchemaLoadError::KeyMismatch { .. }
        ))
    ));
    let bytes = serde_json::to_string(&raw)
        .unwrap()
        .replacen('{', "{\"flow_id\":\"A\",", 1);
    fs::write(&path, bytes).unwrap();
    assert!(matches!(
        repo.load_raw(&key),
        Err(SchemaLoadError::Json { .. })
    ));
    assert!(matches!(
        repo.list_schemas(),
        Err(SchemaCatalogError::Load(SchemaLoadError::Json { .. }))
    ));
    assert!(read_raw_schema_file(&path).is_err());
}
#[test]
fn empty_and_duplicate_trace_ids_fail_in_both_catalogs() {
    for duplicate in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let mut raw = raw("A", "v1");
        if duplicate {
            raw.traces.push(raw.traces[0].clone());
        } else {
            raw.traces[0].id.clear();
        }
        install(root.path(), &raw);
        let mut memory = InMemorySchemaRepository::new();
        memory.insert(raw).unwrap();
        assert!(matches!(
            memory.list_schemas(),
            Err(SchemaCatalogError::InvalidTraces { .. })
        ));
        assert!(matches!(
            FileSystemSchemaRepository::new(root.path().into())
                .unwrap()
                .list_schemas(),
            Err(SchemaCatalogError::InvalidTraces { .. })
        ));
    }
}
#[test]
fn unsupported_capability_does_not_prevent_acquisition() {
    let root = tempfile::tempdir().unwrap();
    let mut value = serde_json::to_value(raw("A", "v1")).unwrap();
    value["required_capabilities"] = serde_json::json!(["persistence.audit.v1"]);
    let raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    install(root.path(), &raw);
    assert!(SchemaCompiler.compile(raw).is_err());
    assert_eq!(
        FileSystemSchemaRepository::new(root.path().into())
            .unwrap()
            .list_schemas()
            .unwrap()
            .len(),
        1
    );
}
#[cfg(unix)]
#[test]
fn symlinks_in_each_enumerated_level_and_candidate_are_fatal() {
    use std::os::unix::fs::symlink;
    for level in 0..3 {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let target = install(outside.path(), &raw("A", "v1"));
        match level {
            0 => symlink(outside.path().join("A"), root.path().join("A")).unwrap(),
            1 => {
                fs::create_dir(root.path().join("A")).unwrap();
                symlink(outside.path().join("A/v1"), root.path().join("A/v1")).unwrap();
            }
            _ => {
                fs::create_dir_all(root.path().join("A/v1")).unwrap();
                symlink(target, root.path().join("A/v1/flow.json")).unwrap();
            }
        }
        assert!(matches!(
            FileSystemSchemaRepository::new(root.path().into())
                .unwrap()
                .list_schemas(),
            Err(SchemaCatalogError::Symlink { .. })
        ));
    }
}
#[cfg(unix)]
#[test]
fn unreadable_root_reports_io_when_permissions_are_enforced() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o0)).unwrap();
    let denied = fs::read_dir(root.path()).is_err();
    let result = FileSystemSchemaRepository::new(root.path().into())
        .unwrap()
        .list_schemas();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    if denied {
        assert!(matches!(result, Err(SchemaCatalogError::Io { .. })));
    } else {
        eprintln!(
            "privileged process bypasses permission bits; permission-denied assertion not applicable"
        );
    }
}
#[test]
fn declaration_tooling_reuses_validation_without_compiling_the_schema() {
    let declaration = serde_json::json!({"id":"d","dataset_id":"codes","version":"v1",
        "path":"codes.json","sha256":"0".repeat(64),"metadata_attributes":[]});
    let raw: RawReferenceDataDependency = serde_json::from_value(declaration.clone()).unwrap();
    assert_eq!(
        compile_reference_dependencies(std::slice::from_ref(&raw))
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        compile_reference_dependencies(&[raw.clone(), raw])
            .unwrap_err()
            .code,
        SchemaIssueCode::InvalidSchemaValue
    );
    let mut metadata = declaration;
    metadata["metadata_attributes"] = serde_json::json!(["age"]);
    assert_eq!(
        compile_reference_dependencies(&[serde_json::from_value(metadata).unwrap()])
            .unwrap_err()
            .code,
        SchemaIssueCode::UnsupportedConstruct
    );
}
