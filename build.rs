//! Build provenance only: no build-host path is used by the executable.
use std::process::Command;

/// Git-free source archives explicitly report unavailable provenance. Release
/// builders may supply a full hexadecimal commit without requiring Git at runtime.
pub fn main() {
    println!("cargo:rerun-if-env-changed=LINTRA_BUILD_COMMIT");
    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .output()
            .ok()
            .and_then(|out| {
                out.status
                    .success()
                    .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
            })
    };
    // Follow Git's own paths so linked worktrees and packed refs also rebuild.
    for name in ["HEAD", "index", "packed-refs"] {
        if let Some(path) = git(&["rev-parse", "--git-path", name]) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    if let Some(reference) = git(&["symbolic-ref", "-q", "HEAD"])
        && let Some(path) = git(&["rev-parse", "--git-path", &reference])
    {
        println!("cargo:rerun-if-changed={path}");
    }
    let commit = std::env::var("LINTRA_BUILD_COMMIT")
        .ok()
        .or_else(|| git(&["rev-parse", "HEAD"]))
        .filter(|value| value.len() == 40 && value.bytes().all(|b| b.is_ascii_hexdigit()))
        .unwrap_or_else(|| "unavailable".into());
    println!("cargo:rustc-env=LINTRA_BUILD_COMMIT={commit}");
}
