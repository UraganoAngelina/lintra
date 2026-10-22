//! Real binary processes with synthetic data, native paths and isolated bundles.
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new(input: &[u8]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "lintra validate à {} {} {}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("schemas/SYNTHETIC/v1")).unwrap();
        fs::create_dir(root.join("reference-data")).unwrap();
        fs::create_dir(root.join("unrelated cwd")).unwrap();
        fs::write(
            root.join("schemas/SYNTHETIC/v1/flow.json"),
            include_bytes!("fixtures/application/schema.json"),
        )
        .unwrap();
        fs::write(root.join("input data.txt"), input).unwrap();
        Self { root }
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lintra"));
        command
            .current_dir(self.root.join("unrelated cwd"))
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
            .arg(self.root.join("input data.txt"))
            .arg("--schema-dir")
            .arg(self.root.join("schemas"))
            .arg("--reference-data-dir")
            .arg(self.root.join("reference-data"));
        command
    }
    fn run(&self) -> Output {
        self.with_outputs("errors.jsonl", "summary.json")
    }
    fn with_outputs(&self, errors: &str, summary: &str) -> Output {
        self.command()
            .arg("--output")
            .arg(self.root.join(errors))
            .arg("--summary")
            .arg(self.root.join(summary))
            .output()
            .unwrap()
    }
    fn assert_no_parts(&self) {
        assert!(!self.root.join("errors.jsonl.part").exists());
        assert!(!self.root.join("summary.json.part").exists());
    }
    fn old_reports(&self) {
        fs::write(self.root.join("errors.jsonl"), b"old errors").unwrap();
        fs::write(self.root.join("summary.json"), b"old summary").unwrap();
    }
    fn assert_old(&self) {
        assert_eq!(
            fs::read(self.root.join("errors.jsonl")).unwrap(),
            b"old errors"
        );
        assert_eq!(
            fs::read(self.root.join("summary.json")).unwrap(),
            b"old summary"
        );
        self.assert_no_parts();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).unwrap()
}
#[test]
fn valid_process_publishes_empty_jsonl_and_audited_local_summary() {
    let file = Fixture::new(b"001A  \r\n");
    file.old_reports();
    let out = file.run();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(text(&out.stdout).starts_with("Result: VALID\nMode: LocalOnly"));
    assert!(text(&out.stdout).contains("No validation errors found in the selected mode."));
    assert!(fs::read(file.root.join("errors.jsonl")).unwrap().is_empty());
    let bytes = fs::read(file.root.join("summary.json")).unwrap();
    assert!(!bytes.starts_with(&[239, 187, 191]));
    assert!(!bytes.contains(&b'\r'));
    let summary: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(summary["result"], "VALID");
    assert_eq!(summary["validation_mode"], "local_only");
    assert_eq!(summary["records_processed"], 1);
    assert_eq!(summary["inputs"][0]["bytes_read"], 8);
    assert_eq!(summary["inputs"][0]["completion"]["kind"], "complete");
    assert_eq!(summary["inputs"][0]["file_name"], "input data.txt");
    for line in text(&out.stderr).lines() {
        let event: serde_json::Value = serde_json::from_str(line).unwrap();
        assert!(event["event"].is_string());
        assert!(!line.contains("001A"));
    }
    assert!(text(&out.stderr).contains("run_completed"));
    file.assert_no_parts();
}
#[test]
fn normative_invalidity_returns_two_with_real_streamed_diagnostics() {
    let file = Fixture::new(b"002A  \n\n");
    let out = file.run();
    assert_eq!(out.status.code(), Some(2), "{}", text(&out.stderr));
    assert!(text(&out.stdout).starts_with("Result: INVALID\n"));
    let bytes = fs::read(file.root.join("errors.jsonl")).unwrap();
    let errors: Vec<serde_json::Value> = text(&bytes)
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0]["code"], "E_INVALID_DOMAIN");
    assert_eq!(errors[1]["code"], "E_RECORD_LENGTH");
    assert!(!bytes.contains(&b'\r'));
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(file.root.join("summary.json")).unwrap()).unwrap();
    assert_eq!(summary["records_processed"], 2);
    assert_eq!(summary["invalid_records"], 2);
    file.assert_no_parts();
}
#[test]
fn bom_and_schema_failures_preserve_existing_reports_and_clean_parts() {
    for schema_error in [false, true] {
        let file = Fixture::new(b"\xef\xbb\xbf001A  \n");
        file.old_reports();
        if schema_error {
            fs::write(file.root.join("schemas/SYNTHETIC/v1/flow.json"), b"{}").unwrap();
        }
        let out = file.run();
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
        assert!(text(&out.stderr).contains("command failed:"));
        file.assert_old();
    }
}
#[test]
fn missing_input_and_missing_selected_schema_are_technical_without_fallback() {
    for missing_input in [false, true] {
        let file = Fixture::new(b"001A  \n");
        file.old_reports();
        if missing_input {
            fs::remove_file(file.root.join("input data.txt")).unwrap();
        } else {
            fs::remove_file(file.root.join("schemas/SYNTHETIC/v1/flow.json")).unwrap();
        }
        assert_eq!(file.run().status.code(), Some(1));
        file.assert_old();
    }
}
#[test]
fn output_collisions_cannot_replace_input_bundle_or_each_other() {
    let file = Fixture::new(b"001A  \n");
    for (errors, summary) in [
        ("input data.txt", "summary.json"),
        ("errors.jsonl", "errors.jsonl"),
        ("errors.jsonl", "errors.jsonl.part"),
        ("schemas/SYNTHETIC/v1/flow.json", "summary.json"),
        ("reference-data/new.json", "summary.json"),
    ] {
        assert_eq!(file.with_outputs(errors, summary).status.code(), Some(1));
        assert_eq!(
            fs::read(file.root.join("input data.txt")).unwrap(),
            b"001A  \n"
        );
        assert_eq!(
            fs::read(file.root.join("schemas/SYNTHETIC/v1/flow.json")).unwrap(),
            include_bytes!("fixtures/application/schema.json")
        );
        file.assert_no_parts();
    }
}
#[test]
fn second_output_creation_failure_aborts_first_and_preserves_foreign_part() {
    let file = Fixture::new(b"001A  \n");
    file.old_reports();
    fs::write(
        file.root.join("summary.json.part"),
        b"foreign pending output",
    )
    .unwrap();
    assert_eq!(file.run().status.code(), Some(1));
    assert!(!file.root.join("errors.jsonl.part").exists());
    assert_eq!(
        fs::read(file.root.join("summary.json.part")).unwrap(),
        b"foreign pending output"
    );
    assert_eq!(
        fs::read(file.root.join("errors.jsonl")).unwrap(),
        b"old errors"
    );
}
#[test]
fn version_is_a_real_process_success_with_version_and_provenance() {
    let out = Command::new(env!("CARGO_BIN_EXE_lintra"))
        .arg("--version")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        text(&out.stdout),
        concat!(
            "lintra ",
            env!("CARGO_PKG_VERSION"),
            " (commit ",
            env!("LINTRA_BUILD_COMMIT"),
            ")\n"
        )
    );
    assert!(out.stderr.is_empty());
}

/// A real failed stdout write must abort both pending report files. /dev/full is
/// a Linux device, so native Windows coverage uses the portable sink tests.
#[cfg(target_os = "linux")]
#[test]
fn human_output_failure_aborts_reports_before_publication() {
    let file = Fixture::new(b"001A  \n");
    file.old_reports();
    let out = file
        .command()
        .arg("--errors")
        .arg(file.root.join("errors.jsonl"))
        .arg("--summary")
        .arg(file.root.join("summary.json"))
        .stdout(
            fs::OpenOptions::new()
                .write(true)
                .open("/dev/full")
                .unwrap(),
        )
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    file.assert_old();
}

#[cfg(unix)]
#[test]
fn symlink_alias_of_input_is_rejected_without_modifying_source() {
    let file = Fixture::new(b"001A  \n");
    std::os::unix::fs::symlink(
        file.root.join("input data.txt"),
        file.root.join("alias.jsonl"),
    )
    .unwrap();
    assert_eq!(
        file.with_outputs("alias.jsonl", "summary.json")
            .status
            .code(),
        Some(1)
    );
    assert_eq!(
        fs::read(file.root.join("input data.txt")).unwrap(),
        b"001A  \n"
    );
    assert!(!file.root.join("alias.jsonl.part").exists());
}

/// Wait for an actual service event before signalling a long-running real CLI.
/// The retained stderr reader prevents a closed pipe from masking cancellation.
#[cfg(target_os = "linux")]
#[test]
fn sigint_returns_130_and_preserves_old_reports() {
    use std::{
        io::{BufRead, BufReader, Read, Write},
        process::Stdio,
    };
    let file = Fixture::new(b"");
    let mut input = fs::File::create(file.root.join("input data.txt")).unwrap();
    let block = b"001A  \n".repeat(8192);
    for _ in 0..128 {
        input.write_all(&block).unwrap();
    }
    drop(input);
    file.old_reports();
    let mut child = file
        .command()
        .arg("--errors")
        .arg(file.root.join("errors.jsonl"))
        .arg("--summary")
        .arg(file.root.join("summary.json"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stderr = BufReader::new(child.stderr.take().unwrap());
    let mut first = String::new();
    stderr.read_line(&mut first).unwrap();
    assert_eq!(first, "{\"event\":\"run_started\"}\n");
    let signal = Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .unwrap();
    if !signal.success() {
        let _ = child.kill();
        let _ = child.wait();
        panic!("failed to send SIGINT to CLI process");
    }
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(130));
    assert!(text(&out.stdout).starts_with("Result: CANCELLED\n"));
    assert!(text(&out.stdout).contains("results are incomplete"));
    assert!(!text(&out.stdout).contains("No validation errors found"));
    let mut events = String::new();
    stderr.read_to_string(&mut events).unwrap();
    assert!(events.contains("run_completed"));
    file.assert_old();
}

/// No command is a syntax error, while explicit help/version remain successful.
#[test]
fn process_without_options_is_technical_and_help_is_successful() {
    for args in [vec![], vec!["--help"], vec!["validate", "--help"]] {
        let out = Command::new(env!("CARGO_BIN_EXE_lintra"))
            .args(&args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(if args.is_empty() { 1 } else { 0 }));
        if args.is_empty() {
            assert!(!out.stderr.is_empty());
        } else {
            assert!(!out.stdout.is_empty());
        }
    }
}

/// A request builder with no root flags lets production resolution exercise the
/// actual child environment and executable location, independent of test cwd.
fn request(file: &Fixture, executable: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut cmd = Command::new(executable);
    cmd.current_dir(file.root.join("unrelated cwd"))
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
        .arg(file.root.join("input data.txt"))
        .arg("--errors")
        .arg(file.root.join("errors.jsonl"))
        .arg("--summary")
        .arg(file.root.join("summary.json"));
    cmd
}
#[test]
fn complete_cli_pair_overrides_broken_environment_in_a_real_process() {
    let file = Fixture::new(b"001A  \n");
    let out = file
        .command()
        .env("LINTRA_SCHEMA_DIR", file.root.join("missing schemas"))
        .env(
            "LINTRA_REFERENCE_DATA_DIR",
            file.root.join("missing reference"),
        )
        .arg("--errors")
        .arg(file.root.join("errors.jsonl"))
        .arg("--summary")
        .arg(file.root.join("summary.json"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    file.assert_no_parts();
}
#[test]
fn actual_environment_pair_resolves_and_partial_pair_fails_without_fallback() {
    let file = Fixture::new(b"001A  \n");
    let out = request(&file, env!("CARGO_BIN_EXE_lintra"))
        .env("LINTRA_SCHEMA_DIR", file.root.join("schemas"))
        .env(
            "LINTRA_REFERENCE_DATA_DIR",
            file.root.join("reference-data"),
        )
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    file.old_reports();
    let partial = request(&file, env!("CARGO_BIN_EXE_lintra"))
        .env("LINTRA_SCHEMA_DIR", file.root.join("schemas"))
        .output()
        .unwrap();
    assert_eq!(partial.status.code(), Some(1));
    assert!(text(&partial.stderr).contains("incomplete external bundle"));
    file.assert_old();
}
#[test]
fn copied_executable_resolves_adjacent_bundle_but_never_falls_back_from_selected_env() {
    let file = Fixture::new(b"001A  \n");
    let portable = file.root.join("portable binary");
    fs::create_dir(&portable).unwrap();
    let executable = portable.join(format!("lintra{}", std::env::consts::EXE_SUFFIX));
    fs::copy(env!("CARGO_BIN_EXE_lintra"), &executable).unwrap();
    fs::rename(file.root.join("schemas"), portable.join("schemas")).unwrap();
    fs::rename(
        file.root.join("reference-data"),
        portable.join("reference-data"),
    )
    .unwrap();
    let out = request(&file, &executable).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    file.old_reports();
    fs::create_dir(file.root.join("empty schemas")).unwrap();
    fs::create_dir(file.root.join("empty reference")).unwrap();
    let selected = request(&file, &executable)
        .env("LINTRA_SCHEMA_DIR", file.root.join("empty schemas"))
        .env(
            "LINTRA_REFERENCE_DATA_DIR",
            file.root.join("empty reference"),
        )
        .output()
        .unwrap();
    assert_eq!(selected.status.code(), Some(1));
    assert!(selected.stdout.is_empty());
    file.assert_old();
}

/// A compiler-supported range must also pass the record capability preflight
/// and reach the normative pipeline when invoked through the executable.
#[test]
fn numeric_range_runs_through_service_and_returns_normative_invalidity() {
    let file = Fixture::new(b"002A  \n");
    let path = file.root.join("schemas/SYNTHETIC/v1/flow.json");
    let mut schema: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    schema["traces"][0]["fields"][0]["constraints"] =
        serde_json::json!([{"kind":"numeric_range","min":1,"max":1}]);
    fs::write(path, serde_json::to_vec(&schema).unwrap()).unwrap();
    let out = file.run();
    assert_eq!(out.status.code(), Some(2), "{}", text(&out.stderr));
    let bytes = fs::read(file.root.join("errors.jsonl")).unwrap();
    let errors: Vec<serde_json::Value> = text(&bytes)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], "E_OUT_OF_RANGE");
    file.assert_no_parts();
}
