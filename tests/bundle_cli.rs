//! Real CLI composition with isolated native paths and explicitly synthetic data.
use lintra::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn install(root: &Path) {
    fs::create_dir_all(root.join("schemas/SYNTHETIC/v1")).unwrap();
    fs::create_dir_all(root.join("reference-data")).unwrap();
    fs::write(
        root.join("schemas/SYNTHETIC/v1/flow.json"),
        include_bytes!("../bundles/embedded/schemas/SYNTHETIC/v1/flow.json"),
    )
    .unwrap();
    fs::write(
        root.join("reference-data/example-codes.json"),
        include_bytes!("../bundles/embedded/reference-data/example-codes.json"),
    )
    .unwrap();
}
fn manifest(root: &Path) {
    fs::write(
        root.join("MANIFEST.json"),
        include_bytes!("../bundles/embedded/MANIFEST.json"),
    )
    .unwrap();
}
fn command(root: &Path, external: bool) -> Command {
    fs::write(root.join("input.txt"), b"XXXA  \n").unwrap();
    let mut c = Command::new(env!("CARGO_BIN_EXE_lintra"));
    c.current_dir(root)
        .env_remove("LINTRA_SCHEMA_DIR")
        .env_remove("LINTRA_REFERENCE_DATA_DIR")
        .args([
            "validate",
            "--flow",
            "SYNTHETIC",
            "--schema-version",
            "v1",
            "--trace",
            "T1",
            "--encoding",
            "utf8",
        ])
        .arg("--input")
        .arg(root.join("input.txt"))
        .arg("--errors")
        .arg(root.join("errors.jsonl"))
        .arg("--summary")
        .arg(root.join("summary.json"));
    if external {
        c.arg("--schema-dir")
            .arg(root.join("schemas"))
            .arg("--reference-data-dir")
            .arg(root.join("reference-data"));
    }
    c
}
fn assert_code(output: Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn summary(root: &Path) -> Value {
    serde_json::from_slice(&fs::read(root.join("summary.json")).unwrap()).unwrap()
}
#[test]
fn legacy_external_audit_explicitly_lacks_version_and_does_not_discover_cwd_manifest() {
    let t = tempfile::tempdir().unwrap();
    let r = t.path();
    install(r);
    assert_code(command(r, true).output().unwrap(), 2);
    let audit = summary(r);
    assert_eq!(audit["bundle_source"], "external");
    assert!(audit.get("bundle_version").unwrap().is_null());
    assert_eq!(audit["bundle_manifest_status"], "absent");
    fs::rename(r.join("schemas"), r.join("other-schemas")).unwrap();
    manifest(r);
    // Rename the pair: cwd manifest must not be discovered for nonstandard names.
    let mut c = command(r, false);
    c.arg("--schema-dir")
        .arg(r.join("other-schemas"))
        .arg("--reference-data-dir")
        .arg(r.join("reference-data"));
    assert_code(c.output().unwrap(), 2);
    assert_eq!(summary(r)["bundle_manifest_status"], "absent");
}
#[test]
fn implicit_manifest_and_explicit_manifest_with_separate_parents_work() {
    let t = tempfile::tempdir().unwrap();
    let r = t.path();
    install(r);
    manifest(r);
    assert_code(command(r, true).output().unwrap(), 2);
    assert_eq!(summary(r)["bundle_version"], "synthetic-embedded-v1");
    assert_eq!(summary(r)["bundle_manifest_status"], "present");
    fs::create_dir(r.join("separate")).unwrap();
    fs::rename(r.join("reference-data"), r.join("separate/reference-data")).unwrap();
    let mut c = command(r, false);
    c.arg("--schema-dir")
        .arg(r.join("schemas"))
        .arg("--reference-data-dir")
        .arg(r.join("separate/reference-data"))
        .arg("--bundle-manifest")
        .arg(r.join("MANIFEST.json"));
    assert_code(c.output().unwrap(), 2);
    assert_eq!(summary(r)["bundle_manifest_status"], "present");
}
#[test]
fn invalid_explicit_or_present_manifest_and_schema_hash_fail_preserving_reports() {
    for mode in 0..6 {
        let t = tempfile::tempdir().unwrap();
        let r = t.path();
        install(r);
        manifest(r);
        fs::write(r.join("summary.json"), b"old summary").unwrap();
        fs::write(r.join("errors.jsonl"), b"old errors").unwrap();
        let mut c = command(r, true);
        match mode {
            0 => {
                c.arg("--bundle-manifest").arg(r.join("absent.json"));
            }
            1 => fs::write(r.join("MANIFEST.json"), b"{").unwrap(),
            2 => fs::write(
                r.join("MANIFEST.json"),
                br#"{"bundle_version":"a","bundle_version":"b"}"#,
            )
            .unwrap(),
            3 => {
                let mut m: Value =
                    serde_json::from_slice(include_bytes!("../bundles/embedded/MANIFEST.json"))
                        .unwrap();
                m["files"]["schemas/../escape"] = "0".repeat(64).into();
                fs::write(r.join("MANIFEST.json"), serde_json::to_vec(&m).unwrap()).unwrap();
            }
            4 => {
                let path = r.join("schemas/SYNTHETIC/v1/flow.json");
                let mut bytes = fs::read(&path).unwrap();
                bytes.push(b' ');
                fs::write(path, bytes).unwrap();
            }
            5 => {
                fs::remove_file(r.join("MANIFEST.json")).unwrap();
                fs::create_dir(r.join("MANIFEST.json")).unwrap();
            }
            _ => unreachable!(),
        }
        assert_code(c.output().unwrap(), 1);
        assert_eq!(fs::read(r.join("summary.json")).unwrap(), b"old summary");
        assert_eq!(fs::read(r.join("errors.jsonl")).unwrap(), b"old errors");
        assert!(!r.join("summary.json.part").exists());
        assert!(!r.join("errors.jsonl.part").exists());
    }
}
#[test]
fn external_missing_selected_schema_does_not_fall_back() {
    let t = tempfile::tempdir().unwrap();
    let r = t.path();
    install(r);
    fs::remove_file(r.join("schemas/SYNTHETIC/v1/flow.json")).unwrap();
    assert_code(command(r, true).output().unwrap(), 1);
    assert!(!r.join("summary.json").exists());
}
#[test]
fn explicit_manifest_is_protected_against_output_collision() {
    let t = tempfile::tempdir().unwrap();
    let r = t.path();
    install(r);
    manifest(r);
    // Move the manifest onto the ordinary output name, outside either data root.
    fs::rename(r.join("MANIFEST.json"), r.join("summary.json")).unwrap();
    let before = fs::read(r.join("summary.json")).unwrap();
    assert_code(
        command(r, true)
            .arg("--bundle-manifest")
            .arg(r.join("summary.json"))
            .output()
            .unwrap(),
        1,
    );
    assert_eq!(fs::read(r.join("summary.json")).unwrap(), before);
}
#[cfg(feature = "embedded-bundle")]
#[test]
fn real_embedded_cli_matches_external_errors_and_schema_hash() {
    let t = tempfile::tempdir().unwrap();
    let r = t.path();
    install(r);
    manifest(r);
    assert_code(command(r, true).output().unwrap(), 2);
    let external = summary(r);
    let errors = fs::read(r.join("errors.jsonl")).unwrap();
    assert_code(command(r, false).output().unwrap(), 2);
    let embedded = summary(r);
    assert_eq!(embedded["schema_hash"], external["schema_hash"]);
    assert_eq!(fs::read(r.join("errors.jsonl")).unwrap(), errors);
    assert_eq!(embedded["bundle_source"], "embedded");
    assert_eq!(external["bundle_source"], "external");
    assert_eq!(embedded["bundle_version"], external["bundle_version"]);
    assert_eq!(embedded["bundle_manifest_status"], "present");
}
#[cfg(feature = "embedded-bundle")]
#[test]
fn explicit_manifest_without_external_roots_is_not_ignored() {
    let t = tempfile::tempdir().unwrap();
    let r = t.path();
    install(r);
    manifest(r);
    assert_code(
        command(r, false)
            .arg("--bundle-manifest")
            .arg(r.join("MANIFEST.json"))
            .output()
            .unwrap(),
        1,
    );
}
#[cfg(unix)]
#[test]
fn manifest_symlink_and_schema_escape_are_rejected() {
    use std::os::unix::fs::symlink;
    let t = tempfile::tempdir().unwrap();
    let r = t.path();
    install(r);
    manifest(r);
    fs::rename(r.join("MANIFEST.json"), r.join("original.json")).unwrap();
    symlink(r.join("original.json"), r.join("MANIFEST.json")).unwrap();
    assert_code(command(r, true).output().unwrap(), 1);
    fs::remove_file(r.join("MANIFEST.json")).unwrap();
    manifest(r);
    fs::rename(
        r.join("schemas/SYNTHETIC/v1/flow.json"),
        r.join("escaped.json"),
    )
    .unwrap();
    symlink(
        r.join("escaped.json"),
        r.join("schemas/SYNTHETIC/v1/flow.json"),
    )
    .unwrap();
    assert_code(command(r, true).output().unwrap(), 1);
}

#[test]
fn manifest_reference_guard_keeps_actual_byte_anomalies_in_provider_policy() {
    let t = tempfile::tempdir().unwrap();
    let r = t.path();
    install(r);
    manifest(r);
    let manifest = BundleManifest::parse(&fs::read(r.join("MANIFEST.json")).unwrap()).unwrap();
    let bytes = fs::read(r.join("reference-data/example-codes.json")).unwrap();
    let mut dep = ReferenceDependency {
        key: DatasetKey {
            id: DatasetId("SYNTHETIC_CODES".into()),
            version: DatasetVersion("v1".into()),
        },
        sha256: Sha256::digest(&bytes).into(),
        bundle_path: "example-codes.json".into(),
        metadata_attributes: vec![],
    };
    let preparer = ManifestReferenceDataPreparer::new(r.join("reference-data"), manifest).unwrap();
    for (actual, availability) in [
        (Some(bytes.clone()), DatasetAvailability::Verified),
        (
            Some([bytes.clone(), b" ".to_vec()].concat()),
            DatasetAvailability::Degraded,
        ),
        (Some(b"{".to_vec()), DatasetAvailability::Unavailable),
        (None, DatasetAvailability::Unavailable),
    ] {
        let path = r.join("reference-data/example-codes.json");
        if let Some(bytes) = actual {
            fs::write(&path, bytes).unwrap();
        } else {
            fs::remove_file(&path).unwrap();
        }
        let result = preparer
            .prepare(
                &[dep.clone()],
                DatasetPreparationPolicy::ContinueWithAvailableEvidence,
            )
            .unwrap();
        assert_eq!(result.datasets()[0].availability(), availability);
        assert_eq!(
            preparer
                .prepare(&[dep.clone()], DatasetPreparationPolicy::Strict)
                .is_ok(),
            availability == DatasetAvailability::Verified
        );
    }
    dep.sha256[0] ^= 1;
    assert!(
        preparer
            .prepare(
                &[dep],
                DatasetPreparationPolicy::ContinueWithAvailableEvidence
            )
            .is_err()
    );
    // Manifest presence does not imply successful byte acquisition or membership.
    assert_eq!(serde_json::to_value(BundleAudit::external(Some(&BundleManifest::parse(&serde_json::to_vec(&json!({"bundle_version":"x","schema_format_version":1,"files":{"reference-data/absent.json":"0".repeat(64)}})).unwrap()).unwrap()))).unwrap()["bundle_manifest_status"],"present");
}

#[test]
fn real_cli_manifest_preserves_partial_reference_evidence_and_strict() {
    for mode in 0..5 {
        let t = tempfile::tempdir().unwrap();
        let r = t.path();
        install(r);
        let dataset = fs::read(r.join("reference-data/example-codes.json")).unwrap();
        let other=br#"{"dataset_format_version":1,"dataset_id":"OTHER","version":"v1","keys_hex":["303031"]}"#;
        fs::write(r.join("reference-data/other.json"), other).unwrap();
        let hex = |bytes: &[u8]| {
            Sha256::digest(bytes)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        };
        let mut raw: Value = serde_json::from_slice(include_bytes!(
            "../bundles/embedded/schemas/SYNTHETIC/v1/flow.json"
        ))
        .unwrap();
        raw["reference_data"] = json!([
            {"id":"A","dataset_id":"SYNTHETIC_CODES","version":"v1","path":"example-codes.json","sha256":hex(&dataset),"metadata_attributes":[]},
            {"id":"B","dataset_id":"OTHER","version":"v1","path":"other.json","sha256":hex(other),"metadata_attributes":[]}
        ]);
        let bytes = serde_json::to_vec(&raw).unwrap();
        fs::write(r.join("schemas/SYNTHETIC/v1/flow.json"), &bytes).unwrap();
        let mut m = json!({"bundle_version":"synthetic-reference-v1","schema_format_version":1,"files":{
            "schemas/SYNTHETIC/v1/flow.json":hex(&bytes),"reference-data/example-codes.json":hex(&dataset),"reference-data/other.json":hex(other)}});
        match mode {
            1 => fs::write(
                r.join("reference-data/example-codes.json"),
                [dataset.clone(), b" ".to_vec()].concat(),
            )
            .unwrap(),
            2 => fs::remove_file(r.join("reference-data/example-codes.json")).unwrap(),
            3 => fs::write(r.join("reference-data/example-codes.json"), b"{").unwrap(),
            4 => m["files"]["reference-data/example-codes.json"] = "0".repeat(64).into(),
            _ => {}
        }
        fs::write(r.join("MANIFEST.json"), serde_json::to_vec(&m).unwrap()).unwrap();
        let out = command(r, true).output().unwrap();
        if mode == 4 {
            assert_code(out, 1);
            assert!(!r.join("summary.json").exists());
            continue;
        }
        assert_code(out, 2); // Deliberately invalid numeric input, independent of reference trust.
        let s = summary(r);
        assert_eq!(s["bundle_manifest_status"], "present");
        let datasets = s["reference_validation"]["datasets"].as_array().unwrap();
        assert_eq!(
            datasets[0]["availability"],
            match mode {
                0 => "verified",
                1 => "degraded",
                _ => "unavailable",
            }
        );
        assert_eq!(datasets[1]["availability"], "verified");
        assert_code(
            command(r, true)
                .args(["--reference-policy", "strict"])
                .output()
                .unwrap(),
            if mode == 0 { 2 } else { 1 },
        );
    }
}

#[cfg(feature = "embedded-bundle")]
#[test]
fn embedded_missing_version_is_an_error_without_any_guessed_latest() {
    let t = tempfile::tempdir().unwrap();
    let r = t.path();
    let mut c = Command::new(env!("CARGO_BIN_EXE_lintra"));
    fs::write(r.join("input"), b"001A  \n").unwrap();
    c.env_remove("LINTRA_SCHEMA_DIR")
        .env_remove("LINTRA_REFERENCE_DATA_DIR")
        .args([
            "validate",
            "--flow",
            "SYNTHETIC",
            "--schema-version",
            "absent",
            "--trace",
            "T1",
            "--encoding",
            "utf8",
        ])
        .arg("--input")
        .arg(r.join("input"))
        .arg("--errors")
        .arg(r.join("errors.jsonl"))
        .arg("--summary")
        .arg(r.join("summary.json"));
    assert_code(c.output().unwrap(), 1);
    assert!(!r.join("summary.json").exists());
}
