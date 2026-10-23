//! Real binary, isolated Unicode bundle/temp paths, actual post-analysis export.
use std::{
    fs,
    io::{Cursor, Read},
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
            "lintra CLI XLSX à {} {} {}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("schemas/SYNTHETIC/v1")).unwrap();
        fs::create_dir(root.join("reference-data")).unwrap();
        fs::create_dir(root.join("temporary files")).unwrap();
        fs::create_dir(root.join("cwd")).unwrap();
        fs::write(
            root.join("schemas/SYNTHETIC/v1/flow.json"),
            include_bytes!("fixtures/application/schema.json"),
        )
        .unwrap();
        fs::write(root.join("input.txt"), input).unwrap();
        Self { root }
    }
    fn command(&self) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_lintra"));
        c.current_dir(self.root.join("cwd"))
            .env_remove("LINTRA_SCHEMA_DIR")
            .env_remove("LINTRA_REFERENCE_DATA_DIR")
            .env("TMPDIR", self.root.join("temporary files"))
            .env("TMP", self.root.join("temporary files"))
            .env("TEMP", self.root.join("temporary files"))
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
            .arg(self.root.join("input.txt"))
            .arg("--schema-dir")
            .arg(self.root.join("schemas"))
            .arg("--reference-data-dir")
            .arg(self.root.join("reference-data"));
        c
    }
    fn run(&self, persistent: bool) -> Output {
        let mut c = self.command();
        c.arg("--xlsx")
            .arg(self.root.join("report.xlsx"))
            .arg("--summary")
            .arg(self.root.join("summary.json"));
        if persistent {
            c.arg("--errors").arg(self.root.join("errors.jsonl"));
        }
        c.output().unwrap()
    }
    fn old(&self) {
        for (name, bytes) in [
            ("errors.jsonl", b"old errors".as_slice()),
            ("summary.json", b"old summary"),
            ("report.xlsx", b"old workbook"),
        ] {
            fs::write(self.root.join(name), bytes).unwrap();
        }
    }
    fn assert_old(&self) {
        for (name, bytes) in [
            ("errors.jsonl", b"old errors".as_slice()),
            ("summary.json", b"old summary"),
            ("report.xlsx", b"old workbook"),
        ] {
            assert_eq!(fs::read(self.root.join(name)).unwrap(), bytes);
        }
    }
    fn clean(&self) {
        for name in ["errors.jsonl.part", "summary.json.part", "report.xlsx.part"] {
            assert!(!self.root.join(name).exists(), "leaked {name}");
        }
        assert_eq!(
            fs::read_dir(self.root.join("temporary files"))
                .unwrap()
                .count(),
            0
        );
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn xml(file: &Fixture, name: &str) -> String {
    let bytes = fs::read(file.root.join("report.xlsx")).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut text = String::new();
    archive
        .by_name(name)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    text
}
#[test]
fn xlsx_only_uses_a_temporary_jsonl_for_valid_and_invalid_runs() {
    for (input, code, count) in [(b"001A  \n".as_slice(), 0, 0), (b"999A  \nshort\n", 2, 2)] {
        let f = Fixture::new(input);
        let out = f.run(false);
        assert_eq!(
            out.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!f.root.join("errors.jsonl").exists());
        let workbook = xml(&f, "xl/workbook.xml");
        assert!(workbook.contains("name=\"Errori\""));
        assert!(workbook.contains("name=\"Riepilogo\""));
        let errors = xml(&f, "xl/worksheets/sheet2.xml");
        assert_eq!(errors.matches("<row ").count(), count + 1);
        let summary: serde_json::Value =
            serde_json::from_slice(&fs::read(f.root.join("summary.json")).unwrap()).unwrap();
        assert_eq!(summary["total_errors"], count);
        f.clean();
    }
}
#[test]
fn explicit_jsonl_and_xlsx_share_the_same_diagnostics_in_order() {
    let f = Fixture::new(b"999A  \nshort\n");
    let out = f.run(true);
    assert_eq!(out.status.code(), Some(2));
    let source = fs::read_to_string(f.root.join("errors.jsonl")).unwrap();
    let lines = source
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);
    let errors = xml(&f, "xl/worksheets/sheet2.xml");
    let first = errors.find("<t>E_INVALID_DOMAIN</t>").unwrap();
    let second = errors.find("<t>E_RECORD_LENGTH</t>").unwrap();
    assert!(first < second);
    assert_eq!(errors.matches("<row ").count(), 3);
    f.clean();
}
#[test]
fn stale_xlsx_part_aborts_only_owned_outputs_and_preserves_previous_reports() {
    for persistent in [false, true] {
        let f = Fixture::new(b"001A  \n");
        f.old();
        fs::write(f.root.join("report.xlsx.part"), b"foreign part").unwrap();
        let out = f.run(persistent);
        assert_eq!(out.status.code(), Some(1));
        f.assert_old();
        assert_eq!(
            fs::read(f.root.join("report.xlsx.part")).unwrap(),
            b"foreign part"
        );
        assert!(!f.root.join("summary.json.part").exists());
        assert!(!f.root.join("errors.jsonl.part").exists());
        assert_eq!(
            fs::read_dir(f.root.join("temporary files"))
                .unwrap()
                .count(),
            0
        );
    }
}
#[test]
fn xlsx_collisions_with_input_bundle_or_other_output_parts_are_rejected() {
    for destination in [
        "input.txt",
        "schemas/SYNTHETIC/v1/flow.json",
        "reference-data/new.xlsx",
        "errors.jsonl",
        "errors.jsonl.part",
        "summary.json",
        "summary.json.part",
    ] {
        let f = Fixture::new(b"001A  \n");
        f.old();
        let before = fs::read(f.root.join("schemas/SYNTHETIC/v1/flow.json")).unwrap();
        let out = f
            .command()
            .arg("--errors")
            .arg(f.root.join("errors.jsonl"))
            .arg("--summary")
            .arg(f.root.join("summary.json"))
            .arg("--xlsx")
            .arg(f.root.join(destination))
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "{destination}");
        f.assert_old();
        assert_eq!(fs::read(f.root.join("input.txt")).unwrap(), b"001A  \n");
        assert_eq!(
            fs::read(f.root.join("schemas/SYNTHETIC/v1/flow.json")).unwrap(),
            before
        );
        f.clean();
    }
}
#[test]
fn export_failure_preserves_all_old_reports_and_removes_temporary_jsonl() {
    for persistent in [false, true] {
        let f = Fixture::new(b"999A  \n");
        f.old();
        let path = f.root.join("schemas/SYNTHETIC/v1/flow.json");
        let mut schema: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        schema["traces"][0]["fields"][0]["name"] = serde_json::json!("X".repeat(32768));
        fs::write(path, serde_json::to_vec(&schema).unwrap()).unwrap();
        let out = f.run(persistent);
        assert_eq!(out.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&out.stderr).contains("XLSX workbook generation failed"));
        f.assert_old();
        f.clean();
    }
}
#[test]
fn validation_failure_never_publishes_xlsx_or_leaves_spool_files() {
    for persistent in [false, true] {
        let f = Fixture::new(b"\xef\xbb\xbf001A  \n");
        f.old();
        let out = f.run(persistent);
        assert_eq!(out.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&out.stderr).contains("run_started"));
        f.assert_old();
        f.clean();
    }
}
#[test]
fn summary_publication_failure_cannot_leave_a_new_xlsx() {
    let f = Fixture::new(b"001A  \n");
    fs::create_dir(f.root.join("summary.json")).unwrap();
    fs::write(f.root.join("report.xlsx"), b"old workbook").unwrap();
    let out = f.run(false);
    assert_eq!(out.status.code(), Some(1));
    assert!(f.root.join("summary.json").is_dir());
    assert_eq!(
        fs::read(f.root.join("report.xlsx")).unwrap(),
        b"old workbook"
    );
    f.clean();
}
#[test]
fn completed_partial_reference_run_exports_its_explicit_completeness() {
    let f = Fixture::new(b"001A  \n");
    let path = f.root.join("schemas/SYNTHETIC/v1/flow.json");
    let mut schema: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    schema["reference_data"] = serde_json::json!([{"id":"A","dataset_id":"synthetic-codes","version":"1","path":"absent.json","sha256":"0".repeat(64),"metadata_attributes":[]}]);
    schema["traces"][0]["fields"][0]["constraints"].as_array_mut().unwrap().push(serde_json::json!({"kind":"reference_data","datasets":["A"],"fallback_values":[],"metadata_bindings":[]}));
    fs::write(path, serde_json::to_vec(&schema).unwrap()).unwrap();
    let out = f.run(false);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("Completeness: Partial"));
    let summary = xml(&f, "xl/worksheets/sheet1.xml");
    assert!(summary.contains("<t>/reference_validation/completeness</t>"));
    assert!(summary.contains("<t>partial</t>"));
    assert!(summary.contains("<t>unavailable</t>"));
    f.clean();
}
#[test]
fn no_error_format_is_a_parser_error_and_xlsx_does_not_replace_summary_requirement() {
    let f = Fixture::new(b"001A  \n");
    let out = f
        .command()
        .arg("--summary")
        .arg(f.root.join("summary.json"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let out = f
        .command()
        .arg("--xlsx")
        .arg(f.root.join("report.xlsx"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    f.clean();
}
