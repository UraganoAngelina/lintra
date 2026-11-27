//! T-MIG-P6-09: numeric verification and golden report of the desktop
//! diagnostic path (G-MIG-6, QG-4). Every committed golden scenario the
//! desktop can run (LocalOnly profiles and the Full SIOC referential bundle)
//! goes through the production worker, the results/detail IPC operations and
//! the export bridge; what the operator sees and saves is compared with the
//! committed oracle, not with another run of the same code:
//!
//! - the published spool and the JSONL export are the committed
//!   `expected.errors.jsonl`, byte for byte, in order;
//! - the summary is `expected.summary.json`, except wall clock and elapsed
//!   time; where the oracle is the single-reader `golden-test` adapter also
//!   `validator_version`, and its file-bytes `schema_hash` is replaced by the
//!   canonical `schema_fingerprint` the service uses; the export sidecar is
//!   the same summary plus the report's bundle provenance;
//! - overview counts, unfiltered paging, every exact filter and the detail
//!   of each row are counted from the oracle lines, independently of Rust;
//! - T-MIG-P6-10: the XLSX export is the canonical workbook, and its cells,
//!   read back from the ZIP, are the oracle lines and the summary;
//!   a Full cross-trace detail (guide, position, context of both traces) is
//!   the spool diagnostic and the captured schema.
use super::tests::{Dialogs, about, canonical, execute, prepare, published};
use crate::{
    configuration::PreparedIntent,
    jobs::{Jobs, RunRef},
    results::{DetailRequest, ResultsRequest},
};
use lintra::*;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

fn golden() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tests/golden")
}

/// Intent over committed golden inputs, read in place (the export bridge
/// protects them as inputs). Same options as the golden producers.
fn intent(
    schema: &Path,
    scope: ValidationScope,
    mode: ValidationMode,
    parameters: BTreeMap<String, JoinParameterValue>,
) -> (Arc<PreparedIntent>, Arc<dyn ReferenceDataPreparer>) {
    let raw: RawFlowSchema = serde_json::from_slice(&fs::read(schema).unwrap()).unwrap();
    let traces = match &scope {
        ValidationScope::Trace(_) => 1,
        ValidationScope::Flow(flow) => flow.traces.len(),
    };
    let bundle = AnalysisBundle::builtin().unwrap();
    let intent = PreparedIntent {
        trace_refs: (0..traces).map(|i| format!("config_1_trace_{i}")).collect(),
        request: ValidationRequest {
            schema_key: SchemaKey {
                flow_id: FlowId(raw.flow_id.clone()),
                version: SchemaVersion(raw.schema_version.clone()),
            },
            scope,
            mode,
            options: ValidationOptions {
                record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                newline_policy: NewlinePolicy::AutoLfOrCrlf,
                max_errors: None,
            },
        },
        parameters,
        raw: Arc::new(raw),
        audit: bundle.audit().clone(),
    };
    (Arc::new(intent), bundle.references())
}

/// The golden scenarios of one producer root, in name order.
fn scenarios(root: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    found.sort();
    assert!(!found.is_empty(), "empty golden root {}", root.display());
    found
}

fn results(jobs: &mut Jobs, operation: &str, payload: Value) -> Value {
    let request = ResultsRequest::decode(operation, payload.as_object().unwrap()).unwrap();
    let bytes = jobs
        .results(request, "golden")
        .unwrap_or_else(|error| panic!("{operation} {payload}: {error:?}"));
    serde_json::from_slice::<Value>(&bytes).unwrap()["payload"].clone()
}
fn detail(jobs: &mut Jobs, operation: &str, payload: Value) -> Value {
    let request = DetailRequest::decode(operation, payload.as_object().unwrap()).unwrap();
    let bytes = jobs.detail(request, "golden").unwrap();
    serde_json::from_slice::<Value>(&bytes).unwrap()["payload"].clone()
}

/// A query over the published report, built off the main thread, then read
/// page by page with `limit` rows: (total_matches, total_diagnostics, rows).
fn query(jobs: &mut Jobs, run: &RunRef, filters: Value, limit: u64) -> (u64, u64, Vec<Value>) {
    let mut filters = filters;
    filters["schemaVersion"] = 1.into();
    let opened = results(
        jobs,
        "results_query",
        json!({"schemaVersion":1,"run":run.dto(),"filters":filters}),
    );
    let query_ref = opened["query_ref"].clone();
    let deadline = Instant::now() + Duration::from_secs(30);
    let ready = loop {
        let state = results(
            jobs,
            "results_query_state",
            json!({"schemaVersion":1,"query_ref":query_ref}),
        );
        if state["state"] != "building" {
            break state;
        }
        assert!(Instant::now() < deadline, "query did not settle");
        thread::sleep(Duration::from_millis(2));
    };
    assert_eq!(ready["state"], "ready", "{ready}");
    let number = |value: &Value| value.as_str().unwrap().parse::<u64>().unwrap();
    let (mut rows, mut offset) = (Vec::new(), "0".to_owned());
    loop {
        let page = results(
            jobs,
            "results_page",
            json!({"schemaVersion":1,"query_ref":query_ref,"offset":offset,"limit":limit}),
        );
        let chunk = page["rows"].as_array().unwrap();
        assert!(chunk.len() as u64 <= limit);
        for row in chunk {
            let mut row = row.clone();
            row["query_ref"] = query_ref.clone();
            rows.push(row);
        }
        if page["has_more"] == false {
            assert_eq!(number(&page["total_matches"]), rows.len() as u64);
            return (
                number(&ready["total_matches"]),
                number(&ready["total_diagnostics"]),
                rows,
            );
        }
        offset = page["next_offset"].as_str().unwrap().to_owned();
    }
}

/// A selected row's text component, reassembled through bounded reads.
fn text(jobs: &mut Jobs, selection: &Value, kind: &str) -> Option<String> {
    let component = selection["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["kind"] == kind)?
        .clone();
    let (mut text, mut offset) = (String::new(), "0".to_owned());
    loop {
        let chunk = detail(
            jobs,
            "results_text",
            json!({"schemaVersion":1,"selection_ref":selection["selection_ref"],
                "component_ref":component["component_ref"],"offset":offset,"limit":64}),
        );
        text.push_str(chunk["text"].as_str().unwrap());
        if chunk["has_more"] == false {
            return Some(text);
        }
        offset = chunk["next_offset"].as_str().unwrap().to_owned();
    }
}

/// A row preview is the exact text, or (`complete: false`) its exact
/// prefix; the whole text is read through the detail.
fn assert_preview(preview: &Value, expected: &Value, label: &str) {
    let (text, expected) = (
        preview["text"].as_str().unwrap(),
        expected.as_str().unwrap(),
    );
    if preview["complete"] == true {
        assert_eq!(text, expected, "{label}");
    } else {
        assert!(
            text.len() < expected.len() && expected.starts_with(text),
            "{label}: {text}"
        );
    }
}

/// Wall clock and elapsed time are runtime metadata; `golden-test` is the
/// version label of the single-reader golden adapter, not of a release.
fn normalized(summary: &[u8]) -> Value {
    let mut value: Value = serde_json::from_slice(summary).unwrap();
    let object = value.as_object_mut().unwrap();
    object.remove("started_at");
    object.remove("elapsed");
    if object["validator_version"] == "golden-test" {
        object.remove("validator_version");
    }
    value
}

/// One golden scenario through the whole desktop path. `traces` are the
/// trace ids in request order (trace_ref `trace_<i>`).
fn verify(
    directory: &Path,
    intent: Arc<PreparedIntent>,
    refs: Arc<dyn ReferenceDataPreparer>,
    traces: &[&str],
) {
    let label = directory.display().to_string();
    let expected_bytes = fs::read(directory.join("expected.errors.jsonl")).unwrap();
    let oracle: Vec<Value> = expected_bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).unwrap())
        .collect();
    let mut expected_summary =
        normalized(&fs::read(directory.join("expected.summary.json")).unwrap());
    if !expected_summary
        .as_object()
        .unwrap()
        .contains_key("validator_version")
    {
        // The golden-test adapter hashes the schema file bytes (README T-P5-07);
        // the service and the desktop use the canonical fingerprint.
        expected_summary["schema_hash"] =
            schema_fingerprint(&intent.raw).unwrap().to_string().into();
    }
    let (mut jobs, run) = published(intent, refs);
    let report = jobs.published().unwrap().1;

    // QG-4: the published spool is the committed golden, byte for byte.
    assert_eq!(
        fs::read(report.spool()).unwrap(),
        expected_bytes,
        "{label} spool"
    );
    let mut summary = normalized(&write_run_summary(Vec::new(), report.summary()).unwrap());
    if !expected_summary
        .as_object()
        .unwrap()
        .contains_key("validator_version")
    {
        summary.as_object_mut().unwrap().remove("validator_version");
    }
    assert_eq!(summary, expected_summary, "{label} summary");

    // Overview: every count the charts show, from the oracle.
    let overview = results(
        &mut jobs,
        "results_overview",
        json!({"schemaVersion":1,"run":run.dto()}),
    );
    for key in [
        "records_processed",
        "valid_records",
        "invalid_records",
        "total_errors",
    ] {
        assert_eq!(
            overview[key],
            expected_summary[key].to_string(),
            "{label} {key}"
        );
    }
    let by_code: Vec<(String, String)> = overview["by_code"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["code"]["text"].as_str().unwrap().to_owned(),
                c["count"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    let expected_by_code: Vec<(String, String)> = expected_summary["errors_by_code"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(code, count)| (code.clone(), count.to_string()))
        .collect();
    assert_eq!(by_code, expected_by_code, "{label} by_code");
    let by_trace: Vec<String> = overview["by_trace"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| {
            format!(
                "{}={}",
                t["trace"]["text"].as_str().unwrap(),
                t["count"].as_str().unwrap()
            )
        })
        .collect();
    let expected_by_trace: Vec<String> = traces
        .iter()
        .map(|id| {
            format!(
                "{id}={}",
                oracle.iter().filter(|row| row["trace"] == *id).count()
            )
        })
        .collect();
    assert_eq!(by_trace, expected_by_trace, "{label} by_trace");

    // Unfiltered paging (two rows per page, so paging is exercised) keeps
    // the canonical order: line, trace, code and message of each oracle row.
    let (total, diagnostics, rows) = query(&mut jobs, &run, json!({}), 2);
    assert_eq!(
        (total, diagnostics),
        (oracle.len() as u64, oracle.len() as u64),
        "{label}"
    );
    let trace_ref = |id: &Value| format!("trace_{}", traces.iter().position(|t| *t == id).unwrap());
    for (row, expected) in rows.iter().zip(&oracle) {
        assert_eq!(row["line"], expected["line"].to_string(), "{label}");
        assert_eq!(row["trace_ref"], trace_ref(&expected["trace"]), "{label}");
        assert_eq!(row["code"]["text"], expected["code"], "{label}");
        assert_preview(&row["message"], &expected["message"], &label);
    }

    // Detail of each row: the complete code, message and expectation of the
    // oracle diagnostic, not the bounded row preview. Selections belong to the
    // current query (a new query makes them stale), so they are read from
    // one unfiltered page before any filter.
    let (_, _, rows) = query(&mut jobs, &run, json!({}), 128);
    assert_eq!(rows.len(), oracle.len(), "{label}");
    for (row, expected) in rows.iter().zip(&oracle) {
        let selection = detail(
            &mut jobs,
            "results_select",
            json!({"schemaVersion":1,"query_ref":row["query_ref"],"diagnostic_ref":row["diagnostic_ref"]}),
        );
        assert_eq!(selection["line"], expected["line"].to_string(), "{label}");
        assert_eq!(
            text(&mut jobs, &selection, "code").as_deref(),
            expected["code"].as_str(),
            "{label}"
        );
        assert_eq!(
            text(&mut jobs, &selection, "message").as_deref(),
            expected["message"].as_str(),
            "{label}"
        );
        assert_eq!(
            text(&mut jobs, &selection, "expected").as_deref(),
            expected["expected"].as_str(),
            "{label}"
        );
    }

    // Exact filters: each value present in the oracle selects exactly the
    // oracle rows carrying it, in order.
    // (filter DTO, oracle key, value): the oracle side of each filter.
    let key = |row: &Value, key: &str| -> Value {
        if key == "field" {
            row["field"]["id"].clone()
        } else {
            row[key].clone()
        }
    };
    let mut filters: Vec<(Value, &str, Value)> = Vec::new();
    for row in &oracle {
        filters.push((json!({"code":row["code"]}), "code", row["code"].clone()));
        filters.push((
            json!({"line":row["line"].to_string()}),
            "line",
            row["line"].clone(),
        ));
        filters.push((
            json!({"trace_ref":trace_ref(&row["trace"])}),
            "trace",
            row["trace"].clone(),
        ));
        if row["field"]["id"].is_string() {
            filters.push((
                json!({"field":row["field"]["id"]}),
                "field",
                row["field"]["id"].clone(),
            ));
        }
    }
    for (filter, name, value) in &filters {
        let expected: Vec<&Value> = oracle
            .iter()
            .filter(|row| key(row, name) == *value)
            .collect();
        let (total, diagnostics, rows) = query(&mut jobs, &run, filter.clone(), 128);
        assert_eq!(
            (total, diagnostics),
            (expected.len() as u64, oracle.len() as u64),
            "{label} {filter}"
        );
        for (row, expected) in rows.iter().zip(expected) {
            assert_eq!(
                row["line"],
                expected["line"].to_string(),
                "{label} {filter}"
            );
            assert_eq!(row["code"]["text"], expected["code"], "{label} {filter}");
        }
    }

    // Export: JSONL is the golden itself; CSV is the canonical writer over
    // the same report; the sidecar is the golden summary plus bundle
    // provenance. XLSX follows, read back cell by cell.
    let report_ref = about(&mut jobs, &run)["report_ref"].clone();
    let out = tempfile::tempdir().unwrap();
    for format in [ExportFormat::Jsonl, ExportFormat::Csv] {
        let path = out.path().join(format!("golden.{}", format.extension()));
        let dialogs = Dialogs::saving(Some(&path));
        let ready = prepare(&mut jobs, &report_ref, format.extension(), &dialogs, &[]).unwrap();
        assert_eq!(ready["state"], "ready", "{label} {ready}");
        let receipt = execute(&mut jobs, &ready["export_ref"], &[]);
        assert_eq!(receipt["state"], "succeeded", "{label} {receipt}");
        let written = fs::read(&path).unwrap();
        let sidecar = fs::read(summary_path(&path)).unwrap();
        let (body, summary_bytes) = canonical(&report, format);
        if format == ExportFormat::Jsonl {
            assert_eq!(written, expected_bytes, "{label} exported JSONL");
        }
        assert_eq!(
            (written, &sidecar),
            (body, &summary_bytes),
            "{label} {format:?}"
        );
        let mut exported = normalized(&sidecar);
        // Bundle provenance is flattened into the sidecar (bundle_* keys).
        let provenance = serde_json::to_value(report.audit()).unwrap();
        for (key, value) in provenance.as_object().unwrap() {
            assert_eq!(
                exported.as_object_mut().unwrap().remove(key).as_ref(),
                Some(value),
                "{label} {key}"
            );
        }
        if !expected_summary
            .as_object()
            .unwrap()
            .contains_key("validator_version")
        {
            exported
                .as_object_mut()
                .unwrap()
                .remove("validator_version");
        }
        assert_eq!(exported, expected_summary, "{label} sidecar");
    }
    xlsx(&mut jobs, &report, &report_ref, out.path(), &oracle, &label);
}

/// XLSX export of the published report. Its bytes are the canonical
/// workbook writer's (the creation time is the summary's `started_at`, so the
/// workbook is deterministic); its cells, read back from the ZIP, are the
/// oracle: «Errori» holds one row per golden diagnostic in order, column by
/// column as `ERROR_COLUMNS` defines them, and «Riepilogo» the report summary
/// flattened to JSON Pointer paths.
fn xlsx(
    jobs: &mut Jobs,
    report: &crate::jobs::Report,
    report_ref: &Value,
    out: &Path,
    oracle: &[Value],
    label: &str,
) {
    let path = out.join("golden.xlsx");
    let dialogs = Dialogs::saving(Some(&path));
    let ready = prepare(jobs, report_ref, "xlsx", &dialogs, &[]).unwrap();
    assert_eq!(ready["state"], "ready", "{label} {ready}");
    let receipt = execute(jobs, &ready["export_ref"], &[]);
    assert_eq!(receipt["state"], "succeeded", "{label} {receipt}");
    let written = fs::read(&path).unwrap();
    let spool = std::io::BufReader::new(fs::File::open(report.spool()).unwrap());
    let direct = XlsxReportWriter::new()
        .write(spool, report.summary(), Vec::new())
        .unwrap();
    assert!(
        written == direct,
        "{label}: XLSX differs from the canonical writer"
    );
    assert_eq!(
        fs::read(summary_path(&path)).unwrap(),
        canonical(report, ExportFormat::Jsonl).1,
        "{label} XLSX sidecar"
    );

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(written)).unwrap();
    let workbook = entry(&mut archive, "xl/workbook.xml");
    let names: Vec<&str> = workbook
        .split("<sheet ")
        .skip(1)
        .map(|sheet| attribute(sheet, "name"))
        .collect();
    assert_eq!(names, ["Riepilogo", "Errori"], "{label}");
    let has_shared = archive.file_names().any(|name| {
        name.as_deref()
            .is_ok_and(|name| name == "xl/sharedStrings.xml")
    });
    let shared: Vec<String> = if has_shared {
        let xml = entry(&mut archive, "xl/sharedStrings.xml");
        xml.split("<si>").skip(1).map(texts).collect()
    } else {
        Vec::new()
    };

    let errors = cells(&entry(&mut archive, "xl/worksheets/sheet2.xml"), &shared);
    let header = [
        "code",
        "flow",
        "schema_version",
        "trace",
        "line",
        "record_id",
        "fiscal_code",
        "field_id",
        "field_name",
        "start",
        "end",
        "actual",
        "expected",
        "message",
    ];
    assert_eq!(errors[0], header, "{label} XLSX header");
    let text = |value: &Value| {
        value.as_str().map_or_else(
            || {
                if value.is_null() {
                    String::new()
                } else {
                    value.to_string()
                }
            },
            str::to_owned,
        )
    };
    let expected: Vec<Vec<String>> = oracle
        .iter()
        .map(|row| {
            let field = &row["field"];
            [
                &row["code"],
                &row["flow"],
                &row["schema_version"],
                &row["trace"],
                &row["line"],
                &row["record_identity"]["sioc_record_id"],
                &row["record_identity"]["fiscal_code"],
                &field["id"],
                &field["name"],
                &field["start"],
                &field["end"],
                &row["actual"]["display"],
                &row["expected"],
                &row["message"],
            ]
            .into_iter()
            .map(text)
            .collect()
        })
        .collect();
    assert_eq!(errors[1..], expected[..], "{label} XLSX diagnostics");

    let summary: Value =
        serde_json::from_slice(&write_run_summary(Vec::new(), report.summary()).unwrap()).unwrap();
    let mut flattened = vec![vec!["property".to_owned(), "value".to_owned()]];
    flatten("", &summary, &mut flattened);
    assert_eq!(
        cells(&entry(&mut archive, "xl/worksheets/sheet1.xml"), &shared),
        flattened,
        "{label} XLSX Riepilogo"
    );
}

/// JSON Pointer rows of a summary, as the Riepilogo sheet documents them:
/// object keys in document order, arrays by index, scalars as text.
fn flatten(path: &str, value: &Value, rows: &mut Vec<Vec<String>>) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            for (key, value) in map {
                flatten(
                    &format!("{path}/{}", key.replace('~', "~0").replace('/', "~1")),
                    value,
                    rows,
                );
            }
        }
        Value::Array(items) if !items.is_empty() => {
            for (index, value) in items.iter().enumerate() {
                flatten(&format!("{path}/{index}"), value, rows);
            }
        }
        Value::String(text) => rows.push(vec![path.to_owned(), text.clone()]),
        other => rows.push(vec![path.to_owned(), other.to_string()]),
    }
}

fn entry(archive: &mut zip::ZipArchive<std::io::Cursor<Vec<u8>>>, name: &str) -> String {
    let mut text = String::new();
    std::io::Read::read_to_string(&mut archive.by_name(name).unwrap(), &mut text).unwrap();
    text
}
/// Value of attribute `name` of the element whose attributes `element`
/// starts with (text after the tag name), never one of a later element.
fn attribute<'a>(element: &'a str, name: &str) -> &'a str {
    let tag = &element[..element.find('>').unwrap()];
    let start = if tag.starts_with(&format!("{name}=\"")) {
        name.len() + 2
    } else {
        tag.find(&format!(" {name}=\"")).unwrap() + name.len() + 3
    };
    &tag[start..start + tag[start..].find('"').unwrap()]
}
fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}
/// Concatenated `<t>` runs of one string item or inline string.
fn texts(xml: &str) -> String {
    xml.split("<t")
        .skip(1)
        .filter_map(|part| part.split_once('>'))
        .filter(|(attributes, _)| !attributes.ends_with('/'))
        .map(|(_, rest)| unescape(&rest[..rest.find("</t>").unwrap()]))
        .collect()
}
/// Rows of one worksheet as text, placed by the cell reference column
/// (absent cells are empty strings), for inline and shared strings.
fn cells(xml: &str, shared: &[String]) -> Vec<Vec<String>> {
    let column = |reference: &str| {
        reference
            .bytes()
            .take_while(u8::is_ascii_uppercase)
            .fold(0usize, |n, b| n * 26 + usize::from(b - b'A' + 1))
            - 1
    };
    xml.split("<row ")
        .skip(1)
        .map(|row| {
            let row = &row[..row.find("</row>").unwrap_or(row.len())];
            let mut values: Vec<String> = Vec::new();
            for cell in row.split("<c ").skip(1) {
                let index = column(attribute(cell, "r"));
                let value = if cell.contains("t=\"s\"") {
                    let v = &cell[cell.find("<v>").unwrap() + 3..];
                    shared[v[..v.find("</v>").unwrap()].parse::<usize>().unwrap()].clone()
                } else if cell.contains("<is>") {
                    texts(&cell[..cell.find("</c>").unwrap()])
                } else {
                    String::new()
                };
                if values.len() <= index {
                    values.resize(index + 1, String::new());
                }
                values[index] = value;
            }
            values
        })
        .collect()
}

/// LocalOnly: the four single-reader golden profiles, every scenario.
#[test]
fn local_only_goldens_match_through_worker_results_detail_and_export() {
    let mut count = 0;
    for profile in ["synthetic/v1", "semantic/v1", "dsl/v1", "derived/v1"] {
        let root = golden().join(profile);
        for directory in scenarios(&root) {
            let (intent, refs) = intent(
                &root.join("schema.json"),
                ValidationScope::Trace(TraceValidationInput {
                    trace_id: TraceId("T1".into()),
                    source: InputSource::Path(directory.join("input.txt")),
                }),
                ValidationMode::LocalOnly,
                BTreeMap::new(),
            );
            verify(&directory, intent, refs, &["T1"]);
            count += 1;
        }
    }
    assert_eq!(count, 12, "every LocalOnly golden scenario");
}

/// Full: the SIOC referential cross-trace goldens (five traces, typed
/// parameter `organisation_namespace = 1`, as in `tests/sioc_referential.rs`).
#[test]
fn full_sioc_referential_goldens_match_through_worker_results_detail_and_export() {
    const IDS: [&str; 5] = ["1", "2", "3", "4.1", "4.2"];
    let root = golden().join("cross-trace/sioc-referential/v1");
    let cases = scenarios(&root);
    assert_eq!(cases.len(), 3);
    for directory in cases {
        let (intent, refs) = intent(
            &root.join("schema.json"),
            ValidationScope::Flow(FlowValidationInput {
                traces: IDS
                    .iter()
                    .map(|id| TraceValidationInput {
                        trace_id: TraceId((*id).into()),
                        source: InputSource::Path(directory.join(format!("trace-{id}.dat"))),
                    })
                    .collect(),
            }),
            ValidationMode::Full,
            BTreeMap::from([(
                "organisation_namespace".to_owned(),
                JoinParameterValue::Integer("1".parse().unwrap()),
            )]),
        );
        verify(&directory, intent, refs, &IDS);
    }
}

/// Full detail parity on the shared two-trace bundle (`S` joins `T` on KEY
/// and compares DATA): a cross-trace field diagnostic, a missing related
/// record and a length error. Every detail component is the
/// spool diagnostic's own text; the field guide and the byte position come
/// from the field of the diagnostic's trace in the captured schema; context
/// entries keep their trace, also the other trace's value. The renderer
/// receives all of this from Rust and joins nothing itself.
#[test]
fn full_cross_trace_detail_matches_the_spool_and_the_captured_schema() {
    let (_inputs, intent, refs) =
        crate::jobs::tests::full_fixture(b"01AANO\n02CCOK\nbad\n", b"11AAOK\n12BBOK\n");
    let (mut jobs, run) = published(intent, refs);
    let report = jobs.published().unwrap().1;
    let oracle: Vec<Value> = fs::read_to_string(report.spool())
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let codes: Vec<(&str, &str)> = oracle
        .iter()
        .map(|row| {
            (
                row["trace"].as_str().unwrap(),
                row["code"].as_str().unwrap(),
            )
        })
        .collect();
    // PASS1, then PASS2, in the service's order (a length error in T would
    // leave T incomplete and skip PASS2; cross-trace handles of several
    // traces are covered by the SIOC goldens).
    assert_eq!(
        codes,
        [
            ("S", "E_RECORD_LENGTH"),
            ("S", "E_CROSS_FIELD"),
            ("S", "E_RELATED_RECORD_MISSING")
        ]
    );
    let schema = report.schema();
    let (_, _, rows) = query(&mut jobs, &run, json!({}), 128);
    assert_eq!(rows.len(), oracle.len());
    for (row, expected) in rows.iter().zip(&oracle) {
        let trace = expected["trace"].as_str().unwrap();
        assert_eq!(
            row["trace_ref"],
            if trace == "S" { "trace_0" } else { "trace_1" }
        );
        let selection = detail(
            &mut jobs,
            "results_select",
            json!({"schemaVersion":1,"query_ref":row["query_ref"],"diagnostic_ref":row["diagnostic_ref"]}),
        );
        assert_eq!(selection["trace_ref"], row["trace_ref"]);
        for kind in ["code", "message", "expected"] {
            assert_eq!(
                text(&mut jobs, &selection, kind).as_deref(),
                expected[kind].as_str(),
                "{kind}"
            );
        }
        match expected["field"]["id"].as_str() {
            Some(id) => {
                let spec = schema
                    .trace(&TraceId(trace.into()))
                    .unwrap()
                    .field(&FieldId(id.into()))
                    .unwrap();
                assert_eq!(
                    text(&mut jobs, &selection, "guide").unwrap(),
                    field_format_guide(spec, schema.encoding())
                );
                assert_eq!(
                    text(&mut jobs, &selection, "field_name").as_deref(),
                    expected["field"]["name"].as_str()
                );
                assert_eq!(
                    selection["field_position"],
                    json!({"schemaVersion":1,"start":expected["field"]["start"].to_string(),
                        "end":expected["field"]["end"].to_string(),"units":"one_based_inclusive_bytes"})
                );
                assert_eq!(
                    text(&mut jobs, &selection, "actual").as_deref(),
                    expected["actual"]["display"].as_str()
                );
            }
            None => {
                assert_eq!(selection["field_position"], Value::Null);
                assert!(text(&mut jobs, &selection, "guide").is_none());
            }
        }
        // Every context entry, with its own trace: the two-byte fields are a
        // proper subset of the six-byte record, so their values are exposed.
        let context = detail(
            &mut jobs,
            "results_context",
            json!({"schemaVersion":1,"selection_ref":selection["selection_ref"],"offset":"0","limit":64}),
        );
        let entries: Vec<(String, String, String, String)> = context["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| {
                (
                    e["trace"]["text"].as_str().unwrap().into(),
                    e["field"]["text"].as_str().unwrap().into(),
                    e["value"]["text"].as_str().unwrap().into(),
                    e["visibility"].as_str().unwrap().into(),
                )
            })
            .collect();
        let wanted: Vec<(String, String, String, String)> = expected["context"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| {
                (
                    c["trace"].as_str().unwrap().into(),
                    c["field_id"].as_str().unwrap().into(),
                    c["value"]["display"].as_str().unwrap().into(),
                    "available".into(),
                )
            })
            .collect();
        assert_eq!(entries, wanted, "{expected}");
    }
    // The cross-trace comparison names T's value in S's diagnostic.
    assert!(
        oracle[1]["context"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["trace"] == "T")
    );
    // The trace filter selects each trace's own rows.
    for (handle, trace) in [("trace_0", "S"), ("trace_1", "T")] {
        let (total, _, rows) = query(&mut jobs, &run, json!({"trace_ref":handle}), 128);
        let lines: Vec<String> = oracle
            .iter()
            .filter(|row| row["trace"] == trace)
            .map(|row| row["line"].to_string())
            .collect();
        assert_eq!(total as usize, lines.len());
        assert_eq!(
            rows.iter()
                .map(|r| r["line"].as_str().unwrap().to_owned())
                .collect::<Vec<_>>(),
            lines
        );
    }
}
