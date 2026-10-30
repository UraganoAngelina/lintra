//! Actual executable probes with an empty PATH and a fresh working directory.
#![cfg(feature = "embedded-bundle")]
use std::process::Command;
#[test]
fn executable_version_and_embedded_software_probe_need_no_development_tools() {
    let directory = tempfile::tempdir().unwrap();
    let binary = env!("CARGO_BIN_EXE_lintra-ui");
    let version = Command::new(binary)
        .arg("--version")
        .current_dir(directory.path())
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert!(
        String::from_utf8(version.stdout)
            .unwrap()
            .starts_with(concat!(
                "lintra-ui ",
                env!("CARGO_PKG_VERSION"),
                " (commit "
            ))
    );
    let probe = Command::new(binary)
        .arg("--packaging-check")
        .current_dir(directory.path())
        .env("PATH", "")
        .env_remove("DISPLAY")
        .output()
        .unwrap();
    assert!(
        probe.status.success(),
        "{}",
        String::from_utf8_lossy(&probe.stderr)
    );
    assert!(
        String::from_utf8(probe.stdout)
            .unwrap()
            .starts_with("ui_self_check=ok schemas=")
    );
}
