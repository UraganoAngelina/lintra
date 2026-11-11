//! Explicit native CI case: real loopback SMB/drive mapping, no fake provider.
#![cfg(windows)]
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

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

#[test]
#[ignore = "Requires the real loopback SMB fixture; CI explicitly runs this case"]
fn real_shared_cli_requires_scoped_consent_and_preserves_normative_invalidity() {
    let local = tempfile::tempdir().unwrap();
    let root = local.path();
    fs::create_dir_all(root.join("schemas/SYNTHETIC/v1")).unwrap();
    fs::create_dir(root.join("reference-data")).unwrap();
    fs::write(
        root.join("schemas/SYNTHETIC/v1/flow.json"),
        include_bytes!("fixtures/application/schema.json"),
    )
    .unwrap();
    fs::write(root.join("input.txt"), b"001Z  \n").unwrap();
    for variable in ["LINTRA_TEST_EXPORT_UNC", "LINTRA_TEST_EXPORT_MAPPED"] {
        let endpoint =
            PathBuf::from(std::env::var_os(variable).expect("native SMB fixture is required"));
        let outputs = tempfile::tempdir_in(endpoint).unwrap();
        let errors = outputs.path().join("synthetic.jsonl");
        let summary = outputs.path().join("synthetic.summary.json");
        let denied = command(root, &errors, &summary).output().unwrap();
        assert_eq!(denied.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&denied.stderr).contains("--allow-shared-export"));
        assert_eq!(fs::read_dir(outputs.path()).unwrap().count(), 0);
        let accepted = command(root, &errors, &summary)
            .arg("--allow-shared-export")
            .output()
            .unwrap();
        assert_eq!(accepted.status.code(), Some(2), "{:?}", accepted);
        let receipt = String::from_utf8_lossy(&accepted.stderr);
        assert!(receipt.contains("export_destination_confirmed"));
        assert!(receipt.contains("\"remote_files\":2"));
        assert!(receipt.contains("\"warning_acknowledged\":true"));
        let persisted: serde_json::Value =
            serde_json::from_slice(&fs::read(&summary).unwrap()).unwrap();
        assert_eq!(persisted["result"], "INVALID");
        assert_eq!(persisted["total_errors"], 1);
        assert!(!fs::read(&errors).unwrap().is_empty());
        assert_eq!(fs::read_dir(outputs.path()).unwrap().count(), 2);
        // Confirmation is not permission to overwrite the input or the bundle.
        let unsafe_request = command(root, &root.join("input.txt"), &summary)
            .arg("--allow-shared-export")
            .output()
            .unwrap();
        assert_eq!(unsafe_request.status.code(), Some(1));
        assert_eq!(fs::read(root.join("input.txt")).unwrap(), b"001Z  \n");
    }
}
