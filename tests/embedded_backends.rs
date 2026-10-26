//! Explicitly synthetic byte payloads; no clinical/regional rules or real inputs.
#![cfg(feature = "embedded-bundle")]
use lintra::*;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, sync::Arc};

fn dependency(bundle: &EmbeddedBundle) -> ReferenceDependency {
    ReferenceDependency {
        key: DatasetKey {
            id: DatasetId("SYNTHETIC_CODES".into()),
            version: DatasetVersion("v1".into()),
        },
        bundle_path: "example-codes.json".into(),
        metadata_attributes: vec![],
        sha256: Sha256::digest(bundle.file("reference-data/example-codes.json").unwrap()).into(),
    }
}
fn schema_key() -> SchemaKey {
    SchemaKey {
        flow_id: FlowId("SYNTHETIC".into()),
        version: SchemaVersion("v1".into()),
    }
}

#[test]
fn builtin_manifest_repository_catalog_and_registry_are_real() {
    let bundle = EmbeddedBundle::builtin().unwrap();
    assert_eq!(bundle.manifest().bundle_version(), "synthetic-embedded-v1");
    let embedded = Arc::new(EmbeddedSchemaRepository::new(bundle.clone()).unwrap());
    let entries = embedded.list_schemas().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].key, schema_key());
    assert_eq!(entries[0].traces, vec![TraceId("T1".into())]);
    let mut missing = schema_key();
    missing.version = SchemaVersion("absent".into());
    assert!(matches!(
        embedded.load_raw(&missing),
        Err(SchemaLoadError::NotFound { .. })
    ));
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("SYNTHETIC/v1")).unwrap();
    fs::write(
        root.path().join("SYNTHETIC/v1/flow.json"),
        bundle.file("schemas/SYNTHETIC/v1/flow.json").unwrap(),
    )
    .unwrap();
    let external = Arc::new(FileSystemSchemaRepository::new(root.path().to_owned()).unwrap());
    let embedded = SchemaRegistry::new(embedded, SchemaCompiler, RawResolvedEncoding::Utf8);
    let external = SchemaRegistry::new(external, SchemaCompiler, RawResolvedEncoding::Utf8);
    assert_eq!(
        embedded
            .resolve_with_hash(&schema_key())
            .unwrap()
            .schema_hash(),
        external
            .resolve_with_hash(&schema_key())
            .unwrap()
            .schema_hash()
    );
}

#[test]
fn verified_degraded_unavailable_and_strict_match_filesystem_evidence() {
    let bundle = EmbeddedBundle::builtin().unwrap();
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("example-codes.json"),
        bundle.file("reference-data/example-codes.json").unwrap(),
    )
    .unwrap();
    for mode in 0..4 {
        let mut dep = dependency(&bundle);
        match mode {
            1 => dep.sha256[0] ^= 1,
            2 => dep.bundle_path = "missing.json".into(),
            3 => dep.key.id = DatasetId("WRONG_ID".into()),
            _ => {}
        }
        let embedded =
            EmbeddedReferenceDataProvider::new(bundle.clone(), vec![dep.clone()]).unwrap();
        let external =
            FileSystemReferenceDataProvider::new(root.path().to_owned(), vec![dep.clone()])
                .unwrap();
        for policy in [
            DatasetPreparationPolicy::Strict,
            DatasetPreparationPolicy::ContinueWithAvailableEvidence,
        ] {
            let left = embedded.prepare(policy);
            let right = external.prepare(policy);
            assert_eq!(left.is_ok(), right.is_ok());
            if let (Ok(left), Ok(right)) = (left, right) {
                let left = left.observe(&dep.key, b"001").unwrap();
                let right = right.observe(&dep.key, b"001").unwrap();
                assert_eq!(left.availability(), right.availability());
                assert_eq!(left.observed_present(), right.observed_present());
                assert_eq!(
                    left.evidence().expected_sha256(),
                    right.evidence().expected_sha256()
                );
                assert_eq!(
                    left.evidence().actual_sha256(),
                    right.evidence().actual_sha256()
                );
                assert_eq!(
                    left.evidence().declared_key(),
                    right.evidence().declared_key()
                );
                assert_eq!(
                    left.evidence().anomalies().len(),
                    right.evidence().anomalies().len()
                );
            }
        }
        assert_eq!(embedded.load(&dep.key).is_ok(), mode == 0);
    }
}

#[test]
fn binding_errors_are_rejected_before_embedded_preparation() {
    let bundle = EmbeddedBundle::builtin().unwrap();
    let dep = dependency(&bundle);
    assert!(
        EmbeddedReferenceDataProvider::new(bundle.clone(), vec![dep.clone(), dep.clone()]).is_err()
    );
    let mut bad = dep.clone();
    bad.bundle_path = "../example-codes.json".into();
    assert!(EmbeddedReferenceDataProvider::new(bundle.clone(), vec![bad]).is_err());
    let mut bad = dep;
    bad.metadata_attributes.push("unsupported".into());
    assert!(EmbeddedReferenceDataProvider::new(bundle, vec![bad]).is_err());
}

#[test]
fn malformed_bytes_are_unavailable_not_a_salvaged_empty_set() {
    static FILES: &[EmbeddedFile] = &[EmbeddedFile {
        path: "reference-data/bad.json",
        bytes: b"{",
    }];
    let digest = Sha256::digest(b"{");
    let hex = digest
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let manifest = serde_json::to_vec(
        &json!({"bundle_version":"synthetic-malformed-v1", "schema_format_version":1,
        "files":{"reference-data/bad.json":hex}}),
    )
    .unwrap();
    let bundle = EmbeddedBundle::from_static(&manifest, FILES).unwrap();
    let provider = EmbeddedReferenceDataProvider::new(
        bundle,
        vec![ReferenceDependency {
            key: DatasetKey {
                id: DatasetId("BAD".into()),
                version: DatasetVersion("v1".into()),
            },
            bundle_path: "bad.json".into(),
            sha256: digest.into(),
            metadata_attributes: vec![],
        }],
    )
    .unwrap();
    assert!(provider.prepare(DatasetPreparationPolicy::Strict).is_err());
    let prepared = provider
        .prepare(DatasetPreparationPolicy::ContinueWithAvailableEvidence)
        .unwrap();
    assert_eq!(
        prepared.datasets()[0].availability(),
        DatasetAvailability::Unavailable
    );
    assert!(
        prepared
            .observe(prepared.datasets()[0].key(), b"")
            .unwrap()
            .observed_present()
            .is_none()
    );
}

#[test]
fn manifest_duplicates_unsafe_paths_missing_and_modified_assets_fail() {
    let original = include_bytes!("../bundles/embedded/MANIFEST.json");
    assert!(
        BundleManifest::parse(
            br#"{"bundle_version":"a","bundle_version":"b","schema_format_version":1,"files":{}}"#
        )
        .is_err()
    );
    let mut manifest: serde_json::Value = serde_json::from_slice(original).unwrap();
    manifest["files"]["schemas/../escape.json"] = "0".repeat(64).into();
    assert!(BundleManifest::parse(&serde_json::to_vec(&manifest).unwrap()).is_err());
    assert!(EmbeddedBundle::from_static(original, &[]).is_err());
    static MODIFIED: &[EmbeddedFile] = &[EmbeddedFile {
        path: "schemas/SYNTHETIC/v1/flow.json",
        bytes: b"{}",
    }];
    assert!(matches!(
        EmbeddedBundle::from_static(original, MODIFIED),
        Err(BundleError::Integrity(_))
    ));
    let mut manifest: serde_json::Value = serde_json::from_slice(original).unwrap();
    manifest["schema_format_version"] = 99.into();
    assert!(BundleManifest::parse(&serde_json::to_vec(&manifest).unwrap()).is_err());
}
