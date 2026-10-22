//! Check the real CLI delegates schema decisions to the production compiler.
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lintra lint à {} {} {}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("schemas/SYNTHETIC/v1")).unwrap();
        fs::create_dir(root.join("cwd")).unwrap();
        fs::write(
            root.join("schemas/SYNTHETIC/v1/flow.json"),
            include_bytes!("fixtures/application/schema.json"),
        )
        .unwrap();
        Self(root)
    }
    fn command(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_lintra"));
        cmd.current_dir(self.0.join("cwd"))
            .args([
                "schema",
                "lint",
                "--flow",
                "SYNTHETIC",
                "--schema-version",
                "v1",
                "--encoding",
                "utf8",
            ])
            .arg("--schema-dir")
            .arg(self.0.join("schemas"));
        cmd
    }
    fn run(&self) -> Output {
        self.command().output().unwrap()
    }
    fn mutate(&self, edit: impl FnOnce(&mut serde_json::Value)) {
        let path = self.0.join("schemas/SYNTHETIC/v1/flow.json");
        let mut raw = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        edit(&mut raw);
        fs::write(path, serde_json::to_vec(&raw).unwrap()).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn documents(output: &Output) -> Vec<serde_json::Value> {
    assert!(!output.stdout.contains(&b'\r'));
    std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
#[test]
fn valid_selected_schema_is_checked_without_input_or_reference_provider() {
    let file = Fixture::new();
    let out = file.run();
    assert_eq!(out.status.code(), Some(0), "{:?}", out.stderr);
    assert_eq!(
        documents(&out),
        [serde_json::json!({"result":"VALID","scope":"compiler_supported_subset"})]
    );
    assert!(out.stderr.is_empty());
}
#[test]
fn structural_and_semantic_errors_are_schema_invalidity_with_context() {
    for structural in [false, true] {
        let file = Fixture::new();
        file.mutate(|raw| {
            if structural {
                raw["traces"][0]["fields"][0]["physical"]["start"] = 0.into();
            } else {
                raw["traces"][0]["fields"][0]["constraints"][0]["values"] = serde_json::json!([]);
            }
        });
        let out = file.run();
        assert_eq!(out.status.code(), Some(2), "{:?}", out.stderr);
        let docs = documents(&out);
        let expected = if structural {
            "InvalidPosition"
        } else {
            "InvalidSchemaValue"
        };
        assert!(docs.iter().any(|doc| doc["code"] == expected
            && doc["severity"] == "error"
            && doc["field_id"] == "F1"));
        assert_eq!(docs.last().unwrap()["result"], "INVALID");
        assert!(out.stderr.is_empty());
        // Running a second process preserves compiler issue order and all bytes.
        assert_eq!(out.stdout, file.run().stdout);
    }
}
#[test]
fn unsupported_construct_is_not_silently_skipped_by_lint() {
    let file = Fixture::new();
    file.mutate(|raw| {
        raw["traces"][0]["fields"][0]["constraints"] =
            serde_json::json!([{"kind":"regex","pattern":"^[0-9]+$","match":"required","target":"logical"}]);
    });
    let out = file.run();
    assert_eq!(out.status.code(), Some(2), "{:?}", out.stderr);
    assert!(
        documents(&out)
            .iter()
            .any(|doc| doc["code"] == "UnsupportedConstruct")
    );
}
#[test]
fn load_errors_json_duplicates_and_key_mismatch_are_technical() {
    for case in 0..4 {
        let file = Fixture::new();
        let path = file.0.join("schemas/SYNTHETIC/v1/flow.json");
        match case {
            0 => fs::remove_file(&path).unwrap(),
            1 => fs::write(&path, b"{").unwrap(),
            2 => {
                let raw = fs::read_to_string(&path).unwrap();
                fs::write(
                    &path,
                    raw.replacen(
                        "\"source\": {}",
                        "\"source\": {\"duplicate\": 1, \"duplicate\": 2}",
                        1,
                    ),
                )
                .unwrap();
            }
            _ => file.mutate(|raw| {
                raw["flow_id"] = "OTHER".into();
            }),
        }
        let out = file.run();
        assert_eq!(out.status.code(), Some(1), "case {case}: {:?}", out.stdout);
        assert!(out.stdout.is_empty());
        assert!(!out.stderr.is_empty());
    }
}
#[test]
fn syntax_and_help_remain_distinct_from_schema_invalidity() {
    let file = Fixture::new();
    let missing = Command::new(env!("CARGO_BIN_EXE_lintra"))
        .args(["schema", "lint"])
        .output()
        .unwrap();
    assert_eq!(missing.status.code(), Some(1));
    let help = Command::new(env!("CARGO_BIN_EXE_lintra"))
        .args(["schema", "lint", "--help"])
        .output()
        .unwrap();
    assert_eq!(help.status.code(), Some(0));
    assert!(
        std::str::from_utf8(&help.stdout)
            .unwrap()
            .contains("--schema-dir")
    );
    assert_eq!(
        file.command()
            .args(["--input", "not accepted"])
            .output()
            .unwrap()
            .status
            .code(),
        Some(1)
    );
}
