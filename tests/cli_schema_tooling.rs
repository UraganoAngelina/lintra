//! Executable tests: JSON success is inventory completion, never Strict eligibility.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    process::{Command, Output},
};
struct Fixture {
    root: tempfile::TempDir,
    raw: Value,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("schemas/SYNTHETIC/v1")).unwrap();
        fs::create_dir(root.path().join("data")).unwrap();
        Self {
            root,
            raw: serde_json::from_str(include_str!("fixtures/application/schema.json")).unwrap(),
        }
    }
    fn save(&self) {
        fs::write(
            self.root.path().join("schemas/SYNTHETIC/v1/flow.json"),
            serde_json::to_vec(&self.raw).unwrap(),
        )
        .unwrap();
    }
    fn dependency(&mut self, id: &str, bytes: Option<&[u8]>, expected: Option<&str>) {
        if let Some(bytes) = bytes {
            fs::write(
                self.root.path().join("data").join(format!("{id}.json")),
                bytes,
            )
            .unwrap();
        }
        let digest = expected.map(str::to_owned).unwrap_or_else(|| {
            Sha256::digest(bytes.unwrap_or(b"missing"))
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        });
        self.raw["reference_data"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":format!("binding-{id}"),"dataset_id":id,
            "version":"v1","path":format!("{id}.json"),"sha256":digest,"metadata_attributes":[]}));
    }
    fn command(&self, reference: bool) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_lintra"));
        cmd.args(if reference {
            ["reference-data", "list"]
        } else {
            ["schema", "list"]
        })
        .arg("--schema-dir")
        .arg(self.root.path().join("schemas"));
        if reference {
            cmd.arg("--reference-data-dir")
                .arg(self.root.path().join("data"))
                .args(["--flow", "SYNTHETIC", "--schema-version", "v1"]);
        }
        cmd
    }
    fn list(&self) -> Output {
        self.save();
        self.command(true).output().unwrap()
    }
}
fn dataset(id: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({"dataset_format_version":1,"dataset_id":id,"version":"v1","keys_hex":["534543524554"]})).unwrap()
}
fn success(out: &Output) -> Value {
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}
fn failure(out: &Output) {
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty(), "error must precede stdout");
}
#[test]
fn inventory_lists_all_trust_states_and_typed_causes_without_keys() {
    let mut f = Fixture::new();
    f.dependency("verified", Some(&dataset("verified")), None);
    f.dependency("missing", None, None);
    f.dependency(
        "degraded",
        Some(&dataset("degraded")),
        Some(&"0".repeat(64)),
    );
    f.dependency("malformed", Some(b"{bad json"), None);
    f.dependency("mismatch", Some(&dataset("different-id")), None);
    let out = f.list();
    let doc = success(&out);
    let rows = doc["datasets"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .map(|r| r["dataset_id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["degraded", "malformed", "mismatch", "missing", "verified"]
    );
    assert_eq!(
        rows.iter()
            .map(|r| r["availability"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "degraded",
            "unavailable",
            "unavailable",
            "unavailable",
            "verified"
        ]
    );
    assert_ne!(rows[0]["expected_sha256"], rows[0]["actual_sha256"]);
    assert_eq!(rows[0]["diagnostics"][0]["kind"], "integrity_mismatch");
    assert_eq!(rows[1]["diagnostics"][0]["kind"], "format");
    assert_eq!(rows[2]["diagnostics"][0]["kind"], "identity_mismatch");
    assert_eq!(rows[3]["diagnostics"][0]["kind"], "io");
    assert!(rows[3]["actual_sha256"].is_null());
    assert_eq!(rows[4]["expected_sha256"], rows[4]["actual_sha256"]);
    assert_eq!(doc["policy"], "continue_with_available_evidence");
    assert!(!String::from_utf8_lossy(&out.stdout).contains("534543524554"));
    assert!(!String::from_utf8_lossy(&out.stdout).contains("keys_hex"));
    assert_eq!(out.stdout, f.list().stdout);
    // Strict remains a separate operation: the very same bindings fail Strict.
    let raw: lintra::RawFlowSchema = serde_json::from_value(f.raw.clone()).unwrap();
    let deps = lintra::compile_reference_dependencies(&raw.reference_data).unwrap();
    let provider =
        lintra::FileSystemReferenceDataProvider::new(f.root.path().join("data"), deps).unwrap();
    assert!(
        provider
            .prepare(lintra::DatasetPreparationPolicy::Strict)
            .is_err()
    );
}
#[test]
fn no_dependencies_and_unsupported_global_capability_do_not_fabricate_datasets() {
    let mut f = Fixture::new();
    f.raw["required_capabilities"] = json!(["persistence.audit.v1"]);
    assert_eq!(success(&f.list())["datasets"], json!([]));
}
#[test]
fn invalid_roots_are_fatal_even_without_dependencies() {
    let f = Fixture::new();
    f.save();
    fs::remove_dir(f.root.path().join("data")).unwrap();
    failure(&f.command(true).output().unwrap());
    fs::remove_dir_all(f.root.path().join("schemas")).unwrap();
    failure(&f.command(false).output().unwrap());
}
#[test]
fn version_mismatch_and_corrupt_content_with_matching_hash_stay_unavailable() {
    let mut f = Fixture::new();
    let mut value: Value = serde_json::from_slice(&dataset("wrong-version")).unwrap();
    value["version"] = json!("v2");
    f.dependency(
        "wrong-version",
        Some(&serde_json::to_vec(&value).unwrap()),
        None,
    );
    f.dependency("corrupt", Some(b"not json"), None);
    let doc = success(&f.list());
    for row in doc["datasets"].as_array().unwrap() {
        assert_eq!(row["availability"], "unavailable");
        assert_eq!(row["expected_sha256"], row["actual_sha256"]);
    }
}
#[test]
fn duplicate_invalid_and_unsupported_bindings_are_fatal() {
    for edit in 0..4 {
        let mut f = Fixture::new();
        f.dependency("codes", Some(&dataset("codes")), None);
        match edit {
            0 => {
                let d = f.raw["reference_data"][0].clone();
                f.raw["reference_data"].as_array_mut().unwrap().push(d);
            }
            1 => f.raw["reference_data"][0]["path"] = json!("../escape.json"),
            2 => f.raw["reference_data"][0]["sha256"] = json!("invalid"),
            _ => f.raw["reference_data"][0]["metadata_attributes"] = json!(["clinical_attribute"]),
        }
        failure(&f.list());
    }
}
#[test]
fn schema_listing_is_json_acquisition_and_bad_candidate_prevents_any_stdout() {
    let mut f = Fixture::new();
    f.raw["required_capabilities"] = json!(["persistence.audit.v1"]);
    f.save();
    let doc = success(&f.command(false).output().unwrap());
    assert_eq!(
        doc,
        json!({"schemas":[{"flow_id":"SYNTHETIC","schema_version":"v1","traces":["T1"]}]})
    );
    fs::create_dir_all(f.root.path().join("schemas/Z/v1")).unwrap();
    fs::write(f.root.path().join("schemas/Z/v1/flow.json"), b"malformed").unwrap();
    failure(&f.command(false).output().unwrap());
}
#[cfg(unix)]
#[test]
fn reference_symlink_escape_is_fatal_instead_of_unavailable() {
    let mut f = Fixture::new();
    f.dependency("codes", None, None);
    let outside = tempfile::NamedTempFile::new().unwrap();
    std::os::unix::fs::symlink(outside.path(), f.root.path().join("data/codes.json")).unwrap();
    failure(&f.list());
}
#[test]
fn tooling_requires_explicit_selectors_and_import_inputs() {
    for args in [
        vec!["schema", "list"],
        vec!["reference-data", "list"],
        vec!["schema", "import", "--input", "x.xlsx"],
    ] {
        failure(
            &Command::new(env!("CARGO_BIN_EXE_lintra"))
                .args(args)
                .output()
                .unwrap(),
        );
    }
}
#[cfg(unix)]
#[test]
fn permission_denied_reference_root_is_fatal_when_enforced() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    f.save();
    let root = f.root.path().join("data");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o0)).unwrap();
    let denied = fs::read_dir(&root).is_err();
    let out = f.command(true).output().unwrap();
    fs::set_permissions(root, fs::Permissions::from_mode(0o700)).unwrap();
    if denied {
        failure(&out);
    } else {
        eprintln!("process bypasses directory permission bits");
    }
}
#[cfg(unix)]
#[test]
fn native_non_utf8_root_paths_are_preserved() {
    use std::os::unix::ffi::OsStringExt;
    let root = tempfile::tempdir().unwrap();
    let path = root
        .path()
        .join(std::ffi::OsString::from_vec(vec![b'x', 255]));
    fs::create_dir(&path).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_lintra"))
        .args(["schema", "list", "--schema-dir"])
        .arg(path)
        .output()
        .unwrap();
    assert_eq!(success(&out), json!({"schemas":[]}));
}
