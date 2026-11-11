//! Actual CLI plus OS destinations; all input and retained outputs synthetic.
#![cfg(target_os = "linux")]
use std::{fs, path::Path, process::Command};

fn command(root: &Path, errors: &Path, summary: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lintra"));
    command
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
        .arg("--schema-dir")
        .arg(root.join("schemas"))
        .arg("--reference-data-dir")
        .arg(root.join("reference-data"))
        .arg("--errors")
        .arg(errors)
        .arg("--summary")
        .arg(summary);
    command
}
fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("schemas/SYNTHETIC/v1")).unwrap();
    fs::create_dir(dir.path().join("reference-data")).unwrap();
    fs::write(
        dir.path().join("schemas/SYNTHETIC/v1/flow.json"),
        include_bytes!("fixtures/application/schema.json"),
    )
    .unwrap();
    fs::write(dir.path().join("input.txt"), b"001Z  \n").unwrap();
    dir
}
#[test]
fn ordinary_local_cli_needs_no_confirmation_and_remains_normatively_invalid() {
    let root = fixture();
    let errors = root.path().join("synthetic.jsonl");
    let summary = root.path().join("summary.json");
    let result = command(root.path(), &errors, &summary).output().unwrap();
    assert_eq!(result.status.code(), Some(2), "{:?}", result);
    assert!(!String::from_utf8_lossy(&result.stderr).contains("export_destination_confirmed"));
    let persisted: serde_json::Value = serde_json::from_slice(&fs::read(summary).unwrap()).unwrap();
    assert_eq!(persisted["result"], "INVALID");
    assert_eq!(persisted["total_errors"], 1);
    assert!(!fs::read(errors).unwrap().is_empty());
}
#[test]
fn undetermined_destination_requires_warning_and_confirmation_cannot_override_io() {
    let root = fixture();
    let errors = root.path().join("synthetic.jsonl");
    fs::write(&errors, b"previous synthetic report").unwrap();
    // Real opaque kernel filesystem: the path can be inspected but it cannot
    // host a new report. Consent must not turn that I/O failure into success.
    let summary = Path::new("/proc/lintra-synthetic-summary.json");
    let denied = command(root.path(), &errors, summary).output().unwrap();
    assert_eq!(denied.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&denied.stderr).contains("--allow-shared-export"));
    assert_eq!(fs::read(&errors).unwrap(), b"previous synthetic report");
    assert!(!root.path().join("synthetic.jsonl.part").exists());
    let confirmed = command(root.path(), &errors, summary)
        .arg("--allow-shared-export")
        .output()
        .unwrap();
    assert_eq!(confirmed.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&confirmed.stderr).contains("\"undetermined_files\":1"));
    assert_eq!(fs::read(&errors).unwrap(), b"previous synthetic report");
    assert!(!root.path().join("synthetic.jsonl.part").exists());
    assert!(!summary.exists());
}
