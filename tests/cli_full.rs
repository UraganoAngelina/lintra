//! Real processes: Full uses application wiring and the existing atomic reports.
use serde_json::{Value, json};
use std::{
    fs,
    process::{Command, Output},
};
#[path = "support/full_fixture.rs"]
mod full_fixture;
#[path = "support/full_schema.rs"]
mod full_schema;
use full_fixture::Fixture;
fn command(fixture: &Fixture) -> Command {
    let mut command = fixture.command();
    command
        .arg("--trace-input")
        .arg(fixture.trace_argument("T", &fixture.target))
        .arg("--trace-input")
        .arg(fixture.trace_argument("S", &fixture.source));
    command
}
fn outputs(command: &mut Command, fixture: &Fixture) -> Output {
    command
        .arg("--errors")
        .arg(fixture._root.path().join("errors.jsonl"))
        .arg("--summary")
        .arg(fixture._root.path().join("summary.json"))
        .output()
        .unwrap()
}
fn summary(fixture: &Fixture) -> Value {
    serde_json::from_slice(&fs::read(fixture._root.path().join("summary.json")).unwrap()).unwrap()
}
fn no_parts(fixture: &Fixture) {
    for entry in fs::read_dir(fixture._root.path()).unwrap() {
        assert!(
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".part")
        );
    }
}
#[test]
fn validate_flow_real_process_reports_valid_and_invalid_cross_records() {
    for (source, code, errors) in [(b"01AAOK".as_slice(), 0, 0), (b"01AANO".as_slice(), 2, 1)] {
        let fixture = Fixture::new(full_schema::raw(), source, b"11AAOK");
        let output = outputs(&mut command(&fixture), &fixture);
        assert_eq!(
            output.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let document = summary(&fixture);
        assert_eq!(document["validation_mode"], "full");
        assert_eq!(document["scope"]["trace_ids"], json!(["S", "T"]));
        assert_eq!(document["total_errors"], errors);
        assert_eq!(document["invalid_records"], errors);
        assert_eq!(document["completeness"], "complete");
        assert_eq!(document["full_validation"]["controls"][0]["evaluations"], 1);
        assert!(String::from_utf8_lossy(&output.stdout).contains("Mode: Full"));
        let lines = fs::read_to_string(fixture._root.path().join("errors.jsonl")).unwrap();
        assert_eq!(lines.lines().count(), errors as usize);
        if errors == 1 {
            let error: Value = serde_json::from_str(lines.lines().next().unwrap()).unwrap();
            assert_eq!(error["trace"], "S");
            assert_eq!(error["line"], 1);
        }
        no_parts(&fixture);
    }
}
#[test]
fn full_unknown_publishes_partial_summary_and_xlsx_without_normative_errors() {
    use calamine::{Reader, open_workbook_auto};
    let mut raw = full_schema::raw();
    raw["traces"][0]["joins"][0]["on_missing_record"] = json!({"policy":"unknown","diagnostic":{"code":"SYNTHETIC_PENDING","message_template":"Missing dependency for {rule_id}"}});
    let fixture = Fixture::new(raw, b"01AAOK", b"");
    let xlsx = fixture._root.path().join("full.xlsx");
    let output = outputs(command(&fixture).arg("--xlsx").arg(&xlsx), &fixture);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document = summary(&fixture);
    assert_eq!(document["result"], "VALID");
    assert_eq!(document["completeness"], "partial");
    assert_eq!(document["total_errors"], 0);
    assert_eq!(document["full_validation"]["controls"][0]["unknown"], 1);
    assert_eq!(
        document["full_validation"]["controls"][0]["causes"][0]["join_id"],
        "S_T"
    );
    assert!(
        fs::read(fixture._root.path().join("errors.jsonl"))
            .unwrap()
            .is_empty()
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Completeness: Partial"));
    assert!(text.contains("does not establish full conformance"));
    let mut workbook = open_workbook_auto(&xlsx).unwrap();
    let cells = workbook.worksheet_range("Riepilogo").unwrap();
    let text = cells
        .rows()
        .map(|row| {
            row.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("/full_validation/controls/0/unknown"));
    assert!(text.contains("partial"));
    no_parts(&fixture);
}
#[test]
fn flow_bundle_preflight_errors_leave_no_definitive_reports() {
    for case in [0, 1, 2] {
        let fixture = Fixture::new(full_schema::raw(), b"01AAOK", b"11AAOK");
        let mut command = fixture.command();
        command
            .arg("--trace-input")
            .arg(fixture.trace_argument("S", &fixture.source));
        if case != 0 {
            command.arg("--trace-input").arg(
                fixture.trace_argument(if case == 1 { "S" } else { "UNKNOWN" }, &fixture.target),
            );
        }
        let output = outputs(&mut command, &fixture);
        assert_eq!(output.status.code(), Some(1));
        assert!(!fixture._root.path().join("errors.jsonl").exists());
        assert!(!fixture._root.path().join("summary.json").exists());
        assert_eq!(fs::read(&fixture.source).unwrap(), b"01AAOK");
        no_parts(&fixture);
    }
}
#[test]
fn every_bundle_input_is_protected_from_final_and_part_output_collisions() {
    for case in [0, 1, 2] {
        let fixture = Fixture::new(full_schema::raw(), b"01AAOK", b"11AAOK");
        let mut command = command(&fixture);
        let (errors, summary) = match case {
            0 => (
                fixture.source.clone(),
                fixture._root.path().join("summary.json"),
            ),
            1 => (
                fixture._root.path().join("errors.jsonl"),
                fixture.target.clone(),
            ),
            _ => {
                let part = fixture._root.path().join("protected.part");
                fs::copy(&fixture.target, &part).unwrap();
                // Rebuild the command so that target is exactly the summary .part.
                command = fixture.command();
                command
                    .arg("--trace-input")
                    .arg(fixture.trace_argument("S", &fixture.source))
                    .arg("--trace-input")
                    .arg(fixture.trace_argument("T", &part));
                (
                    fixture._root.path().join("errors.jsonl"),
                    fixture._root.path().join("protected"),
                )
            }
        };
        let output = command
            .arg("--errors")
            .arg(errors)
            .arg("--summary")
            .arg(summary)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(fs::read(&fixture.source).unwrap(), b"01AAOK");
        assert_eq!(fs::read(&fixture.target).unwrap(), b"11AAOK");
        if case == 2 {
            assert_eq!(
                fs::read(fixture._root.path().join("protected.part")).unwrap(),
                b"11AAOK"
            );
        }
    }
}
#[test]
fn explicit_cli_namespace_parameters_use_schema_types_and_reject_duplicates() {
    for case in [0, 1, 2] {
        let mut raw = full_schema::raw();
        raw["parameters"] = json!([{"id":"org","type":"string","allowed_values":["001"]},{"id":"year","type":"integer","min":2020,"max":2030}]);
        raw["traces"][0]["joins"][0]["namespace_parameters"] = json!(["org", "year"]);
        let fixture = Fixture::new(raw, b"01AAOK", b"11AAOK");
        let mut command = command(&fixture);
        if case != 0 {
            command.args(["--parameter", "org=001", "--parameter", "year=2025"]);
        }
        if case == 2 {
            command.args(["--parameter", "org=001"]);
        }
        let output = outputs(&mut command, &fixture);
        assert_eq!(
            output.status.code(),
            Some(if case == 1 { 0 } else { 1 }),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if case == 1 {
            let audit = summary(&fixture);
            assert_eq!(
                audit["full_validation"]["parameters"]["org"],
                json!({"type":"string","value":"001"})
            );
            assert_eq!(
                audit["full_validation"]["parameters"]["year"],
                json!({"type":"integer","value":"2025"})
            );
        } else {
            assert!(!fixture._root.path().join("summary.json").exists());
        }
        no_parts(&fixture);
    }
}
#[cfg(unix)]
#[test]
fn native_non_utf8_parent_path_and_equals_are_preserved() {
    use std::os::unix::ffi::OsStringExt;
    let fixture = Fixture::new(full_schema::raw(), b"01AAOK", b"11AAOK");
    let directory = fixture
        ._root
        .path()
        .join(std::ffi::OsString::from_vec(b"parent\xff=name".to_vec()));
    fs::create_dir(&directory).unwrap();
    let input = directory.join("input=original.txt");
    fs::copy(&fixture.source, &input).unwrap();
    let mut command = fixture.command();
    command
        .arg("--trace-input")
        .arg(fixture.trace_argument("S", &input))
        .arg("--trace-input")
        .arg(fixture.trace_argument("T", &fixture.target));
    let output = outputs(&mut command, &fixture);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        summary(&fixture)["inputs"][0]["file_name"],
        "input=original.txt"
    );
}
