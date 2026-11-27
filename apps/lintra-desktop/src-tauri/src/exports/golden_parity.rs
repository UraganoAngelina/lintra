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
//!   of each row are counted from the oracle lines, independently of Rust.
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
    // provenance. XLSX is covered by the bridge test (ZIP signature).
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
