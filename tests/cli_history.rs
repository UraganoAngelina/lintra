//! Actual executable, native argument paths, stdin pipes and exit codes. The
//! shared key and every record are public synthetic fixtures, never operational.
use lintra::*;
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};
struct Synthetic;
impl PersistenceKeyProvider for Synthetic {
    fn resolve(&self, _: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        Ok(Arc::new(PersistenceKey::new(vec![0xaa; 32])?))
    }
}
struct Fixture(tempfile::TempDir);
impl Fixture {
    fn path(&self, path: &str) -> PathBuf {
        self.0.path().join(path)
    }
    fn new() -> Self {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/cli-history-process");
        fs::create_dir_all(&base).unwrap();
        let value = Self(tempfile::tempdir_in(base).unwrap());
        drop(
            SQLitePersistenceAdapter::create(
                &value.path("private bundle é"),
                Some("bootstrap"),
                &Synthetic,
            )
            .unwrap(),
        );
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options
            .open(value.path("private bundle é/hmac.key"))
            .unwrap()
            .write_all(&[0xaa; 32])
            .unwrap();
        let mut schema: Value =
            serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
        schema["parameters"] = json!([{"id":"unit","type":"integer","min":1,"max":99}]);
        schema["required_capabilities"] = json!(["local.v1", "persistence.audit.v1"]);
        schema["persistence_policy"] = json!({"namespace_parameters":["unit"],
            "version_compatibility":{"kind":"same_schema_version"},
            "run_uniqueness":[{"trace":"T1","key":"record_key"}],
            "cross_run_audit":[{"trace":"T1","key":"record_key"}],
            "history_identity_invariants":[],"snapshot":null,"temporal_rules":[],
            "storage_protection":"external_secure_configuration_required"});
        fs::create_dir_all(value.path("schemas/SYNTHETIC/v1")).unwrap();
        fs::create_dir(value.path("reference-data")).unwrap();
        fs::write(
            value.path("schemas/SYNTHETIC/v1/flow.json"),
            serde_json::to_vec(&schema).unwrap(),
        )
        .unwrap();
        fs::write(value.path("input.dat"), b"001A  \n").unwrap();
        value
    }
    fn base(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lintra"));
        command
            .current_dir(self.0.path())
            .env_remove("LINTRA_SCHEMA_DIR")
            .env_remove("LINTRA_REFERENCE_DATA_DIR");
        command
    }
    fn key(&self, command: &mut Command) {
        command
            .arg("--key-id")
            .arg("synthetic-shared")
            .arg("--key-file")
            .arg(self.path("private bundle é/hmac.key"))
            .arg("--archive")
            .arg(self.path("archive"));
    }
    fn init(&self) -> Output {
        let mut command = self.base();
        command.args(["history", "init"]);
        self.key(&mut command);
        command.output().unwrap()
    }
    fn command(&self, name: &str, review: bool) -> Command {
        fs::create_dir_all(self.path(name)).unwrap();
        let mut command = self.base();
        command.args(["history", "preview"]);
        self.key(&mut command);
        let mut input = std::ffi::OsString::from("T1=");
        input.push(self.path("input.dat"));
        command
            .args([
                "--flow",
                "SYNTHETIC",
                "--schema-version",
                "v1",
                "--encoding",
                "utf8",
                "--parameter",
                "unit=1",
            ])
            .arg("--submission-id")
            .arg(name)
            .arg("--trace-input")
            .arg(input)
            .arg("--schema-dir")
            .arg(self.path("schemas"))
            .arg("--reference-data-dir")
            .arg(self.path("reference-data"));
        for (flag, filename) in [
            ("--errors", "errors.jsonl"),
            ("--summary", "summary.json"),
            ("--audit", "audit.jsonl"),
            ("--audit-summary", "audit-summary.json"),
        ] {
            command.arg(flag).arg(self.path(name).join(filename));
        }
        if review {
            command.arg("--review");
        }
        command
    }
    fn generation(&self) -> Option<SnapshotRevision> {
        let provider = FilePersistenceKeyProvider::load(
            "synthetic-shared",
            &self.path("private bundle é/hmac.key"),
        )
        .unwrap();
        let repo = SQLitePersistenceAdapter::open(
            &self.path("archive"),
            Some("synthetic-shared"),
            &provider,
        )
        .unwrap();
        repo.current_generation(
            &PersistenceNamespace::new(vec![("unit".into(), NamespaceValue::Integer(1))]).unwrap(),
        )
        .unwrap()
    }
}
struct Owned(Option<Child>);
impl Drop for Owned {
    fn drop(&mut self) {
        if let Some(child) = &mut self.0
            && !matches!(child.try_wait(), Ok(Some(_)))
        {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
/// Read the actual emitted candidate before sending approval. Reader and child
/// have deadlines/ownership; no blocked input thread survives a failing test.
fn review(command: &mut Command, mode: &str) -> Output {
    let mut child = Owned(Some(
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    ));
    let stdout = child.0.as_mut().unwrap().stdout.take().unwrap();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let reader = std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let result = reader.read_line(&mut line);
        let _ = tx.send(());
        (reader, line, result)
    });
    if rx.recv_timeout(Duration::from_secs(20)).is_err() {
        child.0.as_mut().unwrap().kill().unwrap();
        child.0.as_mut().unwrap().wait().unwrap();
        // Read failure after killing the child is subordinate to this deadline.
        let _ = reader.join().unwrap();
        panic!("CLI preview output deadline");
    }
    let (mut reader, first, result) = reader.join().unwrap();
    result.unwrap();
    let value: Value = serde_json::from_str(&first).expect("actual preview JSON before review");
    assert_eq!(value["event"], "historical_preview");
    let mut stdin = child.0.as_mut().unwrap().stdin.take();
    match mode {
        "approve" => writeln!(
            stdin.as_mut().unwrap(),
            "{}",
            value["confirmation"].as_str().unwrap()
        )
        .unwrap(),
        "abort" => writeln!(stdin.as_mut().unwrap(), "ABORT").unwrap(),
        "mismatch" => writeln!(stdin.as_mut().unwrap(), "APPROVE a different candidate").unwrap(),
        #[cfg(unix)]
        "signal" => {
            // SAFETY: pid names this owned child, SIGINT invokes its real CLI
            // handler. stdin stays open to prove cancellation, rather than EOF.
            assert_eq!(
                unsafe { libc::kill(child.0.as_ref().unwrap().id() as i32, libc::SIGINT) },
                0
            );
        }
        _ => {
            drop(stdin.take());
        }
    }
    let end = Instant::now() + Duration::from_secs(20);
    while child.0.as_mut().unwrap().try_wait().unwrap().is_none() {
        assert!(Instant::now() < end, "CLI review completion deadline");
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(stdin);
    let mut output = child.0.take().unwrap().wait_with_output().unwrap();
    let mut rest = String::new();
    reader.read_to_string(&mut rest).unwrap();
    output.stdout = (first + &rest).into_bytes();
    output
}

#[test]
fn actual_binary_preview_review_abort_and_exit_codes_preserve_local_history() {
    let fixture = Fixture::new();
    assert_eq!(fixture.init().status.code(), Some(0));
    assert_eq!(fixture.init().status.code(), Some(1));
    let mut check = fixture.base();
    check.args(["history", "check"]);
    fixture.key(&mut check);
    let checked = check.output().unwrap();
    assert_eq!(checked.status.code(), Some(0));
    assert!(
        String::from_utf8(checked.stdout)
            .unwrap()
            .contains("local_archive_key_verified")
    );
    assert_eq!(fixture.generation(), None);
    assert_eq!(
        fixture
            .command("preview", false)
            .output()
            .unwrap()
            .status
            .code(),
        Some(0)
    );
    assert_eq!(fixture.generation(), None);
    for (mode, code) in [("abort", 130), ("eof", 130), ("mismatch", 1)] {
        let output = review(&mut fixture.command(mode, true), mode);
        assert_eq!(
            output.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fixture.generation(), None);
    }
    #[cfg(unix)]
    {
        let output = review(&mut fixture.command("signal", true), "signal");
        assert_eq!(
            output.status.code(),
            Some(130),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fixture.generation(), None);
    }
    let output = review(&mut fixture.command("approved", true), "approve");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("historical_commit")
    );
    assert_eq!(fixture.generation(), Some(SnapshotRevision::new(1)));
    fs::write(fixture.path("input.dat"), b"short\n").unwrap();
    assert_eq!(
        fixture
            .command("invalid", true)
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
    assert_eq!(fixture.generation(), Some(SnapshotRevision::new(1)));
    assert_eq!(
        fixture
            .base()
            .args(["history", "preview", "--help"])
            .output()
            .unwrap()
            .status
            .code(),
        Some(0)
    );
}
