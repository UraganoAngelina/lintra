//! Independent reader checks for the public XLSX content contract (ARCH 39C.3).
use calamine::{Data, Reader, Xlsx};
use lintra::*;
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, io::Cursor, sync::Arc};

// Literal oracle: do not reuse the writer's column descriptors or projection.
const HEADERS: [&str; 14] = [
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

struct Run {
    _root: tempfile::TempDir,
    jsonl: Vec<u8>,
    summary: ValidationSummary,
}
impl Run {
    /// A real compiler/service run with small, exclusively synthetic input.
    /// None = independent rules; Some(false) = missing reference;
    /// Some(true) = interpretable reference whose expected hash is incorrect.
    fn new(input: &[u8], reference: Option<bool>) -> Self {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("sintetico à.txt");
        fs::write(&path, input).unwrap();
        let mut raw: Value =
            serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
        let mut id = raw["traces"][0]["fields"][0].clone();
        id["id"] = json!("ID");
        id["name"] = json!("Identità sintetica à<&");
        id["physical"]["length"] = json!(5);
        id["constraints"] =
            json!([{"kind":"allowed_values","target":"logical","values":["ABCDE"]}]);
        let mut code = id.clone();
        code["id"] = json!("CODE");
        code["name"] = json!("Codice sintetico");
        code["physical"]["start"] = json!(6);
        code["physical"]["length"] = json!(2);
        code["constraints"] = json!([{"kind":"allowed_values","target":"logical","values":["00"]}]);
        if let Some(readable) = reference {
            raw["reference_data"] = json!([{
                "id":"A", "dataset_id":"synthetic-codes", "version":"1",
                "path":"codes.json", "sha256":"0".repeat(64), "metadata_attributes":[]
            }]);
            code["constraints"] = json!([{
                "kind":"reference_data", "datasets":["A"],
                "fallback_values":[], "metadata_bindings":[]
            }]);
            if readable {
                fs::write(root.path().join("codes.json"),
                    br#"{"dataset_format_version":1,"dataset_id":"synthetic-codes","version":"1","keys_hex":["3035"]}"#).unwrap();
            }
        }
        raw["traces"][0]["record_length"] = json!(7);
        raw["traces"][0]["fields"] = json!([id, code]);
        raw["traces"][0]["record_key"]["fields"] = json!(["ID"]);
        raw["traces"][0]["identity"] = json!([{"target":"sioc_record_id","field":"ID"}]);
        let mut repository = InMemorySchemaRepository::new();
        repository
            .insert(serde_json::from_value(raw).unwrap())
            .unwrap();
        let mut service = ValidationService::new(Arc::new(SchemaRegistry::new(
            Arc::new(repository),
            SchemaCompiler,
            RawResolvedEncoding::Utf8,
        )));
        if reference.is_some() {
            service = service.with_reference_data(
                Arc::new(FileSystemReferenceDataPreparer::new(root.path().to_owned()).unwrap()),
                DatasetPreparationPolicy::ContinueWithAvailableEvidence,
            );
        }
        let request = ValidationRequest {
            schema_key: SchemaKey {
                flow_id: FlowId("SYNTHETIC".into()),
                version: SchemaVersion("v1".into()),
            },
            scope: ValidationScope::Trace(TraceValidationInput {
                trace_id: TraceId("T1".into()),
                source: InputSource::Path(path),
            }),
            mode: ValidationMode::LocalOnly,
            options: ValidationOptions {
                record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                newline_policy: NewlinePolicy::AutoLfOrCrlf,
                max_errors: None,
            },
        };
        let mut sink = JsonlSink::new(Vec::new());
        let outcome = service
            .validate(
                request,
                &mut sink,
                &mut |_| {},
                &CancellationToken::default(),
            )
            .unwrap();
        Self {
            _root: root,
            jsonl: sink.finish().unwrap(),
            summary: outcome.summary,
        }
    }
    fn workbook(&self, row_limit: u32) -> Xlsx<Cursor<Vec<u8>>> {
        let bytes = XlsxReportWriter::with_sheet_row_limit(row_limit)
            .unwrap()
            .write(Cursor::new(&self.jsonl), &self.summary, Vec::new())
            .unwrap();
        Xlsx::new(Cursor::new(bytes)).unwrap()
    }
}

/// Empty values may be omitted by Excel; every populated cell must be String,
/// so converting a Float/Date/Formula to text cannot conceal coercion here.
fn text(cell: &Data) -> &str {
    match cell {
        Data::String(value) => value,
        Data::Empty => "",
        other => panic!("expected textual XLSX cell, got {other:?}"),
    }
}
fn json_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}
fn error_rows(jsonl: &[u8]) -> Vec<Vec<String>> {
    jsonl
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            let value: Value = serde_json::from_slice(line).unwrap();
            [
                "/code",
                "/flow",
                "/schema_version",
                "/trace",
                "/line",
                "/record_identity/sioc_record_id",
                "/record_identity/fiscal_code",
                "/field/id",
                "/field/name",
                "/field/start",
                "/field/end",
                "/actual/display",
                "/expected",
                "/message",
            ]
            .map(|path| json_text(value.pointer(path).unwrap_or(&Value::Null)))
            .to_vec()
        })
        .collect()
}
fn read_errors(book: &mut Xlsx<Cursor<Vec<u8>>>) -> Vec<Vec<String>> {
    let mut all = Vec::new();
    for name in book
        .sheet_names()
        .into_iter()
        .filter(|name| name.starts_with("Errori"))
    {
        let range = book.worksheet_range(&name).unwrap();
        assert_eq!(range.width(), HEADERS.len());
        let mut rows = range.rows();
        assert_eq!(
            rows.next().unwrap().iter().map(text).collect::<Vec<_>>(),
            HEADERS
        );
        all.extend(rows.map(|row| row.iter().map(|cell| text(cell).to_owned()).collect()));
        assert!(book.worksheet_formula(&name).unwrap().is_empty());
    }
    all
}
/// Invert each exported JSON Pointer against the canonical document, rather
/// than deriving expected paths with the writer's flattening routine.
fn assert_summary(book: &mut Xlsx<Cursor<Vec<u8>>>, summary: &ValidationSummary) {
    fn leaves(value: &Value) -> usize {
        match value {
            Value::Object(map) if !map.is_empty() => map.values().map(leaves).sum(),
            Value::Array(array) if !array.is_empty() => array.iter().map(leaves).sum(),
            _ => 1,
        }
    }
    let canonical: Value =
        serde_json::from_slice(&write_run_summary(Vec::new(), summary).unwrap()).unwrap();
    let range = book.worksheet_range("Riepilogo").unwrap();
    assert_eq!(range.width(), 2);
    let mut rows = range.rows();
    assert_eq!(
        rows.next().unwrap().iter().map(text).collect::<Vec<_>>(),
        ["property", "value"]
    );
    let mut paths = BTreeSet::new();
    for row in rows {
        let path = text(&row[0]);
        assert!(
            paths.insert(path.to_owned()),
            "duplicate summary property {path}"
        );
        let expected = canonical
            .pointer(path)
            .unwrap_or_else(|| panic!("unknown property {path}"));
        let expected_text = match expected {
            Value::String(value) => value.clone(),
            other => other.to_string(),
        };
        assert_eq!(text(&row[1]), expected_text, "summary property {path}");
    }
    assert_eq!(
        paths.len(),
        leaves(&canonical),
        "missing canonical summary properties"
    );
    assert!(book.worksheet_formula("Riepilogo").unwrap().is_empty());
}

#[test]
fn real_errors_preserve_leading_zeros_text_types_unicode_and_jsonl_order() {
    let run = Run::new(b"0012305\n0000505\n", None);
    assert_eq!(run.summary.total_errors, 4);
    let mut book = run.workbook(XLSX_SHEET_ROWS);
    assert_eq!(book.sheet_names(), ["Riepilogo", "Errori"]);
    let rows = read_errors(&mut book);
    assert_eq!(rows, error_rows(&run.jsonl));
    assert_eq!(
        rows.iter()
            .map(|r| (&*r[4], &*r[5], &*r[7], &*r[11]))
            .collect::<Vec<_>>(),
        [
            ("1", "00123", "ID", "00123"),
            ("1", "00123", "CODE", "05"),
            ("2", "00005", "ID", "00005"),
            ("2", "00005", "CODE", "05")
        ]
    );
    assert_eq!(rows[0][8], "Identità sintetica à<&");
    assert_summary(&mut book, &run.summary);
}

#[test]
fn rollover_preserves_global_order_and_exact_boundary_without_empty_extra_sheets() {
    let run = Run::new(b"0012305\n0000505\n", None);
    for (limit, expected) in [
        (
            2,
            vec![
                "Riepilogo",
                "Errori",
                "Errori (2)",
                "Errori (3)",
                "Errori (4)",
            ],
        ),
        (3, vec!["Riepilogo", "Errori", "Errori (2)"]),
        (4, vec!["Riepilogo", "Errori", "Errori (2)"]),
        (5, vec!["Riepilogo", "Errori"]),
    ] {
        let mut book = run.workbook(limit);
        assert_eq!(book.sheet_names(), expected);
        let names = book.sheet_names();
        for name in &names[1..] {
            let range = book.worksheet_range(name).unwrap();
            assert!((2..=limit as usize).contains(&range.height()));
        }
        assert_eq!(read_errors(&mut book), error_rows(&run.jsonl));
        assert_summary(&mut book, &run.summary);
    }
}

#[test]
fn zero_errors_still_yield_readable_header_and_complete_summary() {
    let run = Run::new(b"ABCDE00\n", None);
    assert_eq!(run.summary.total_errors, 0);
    assert_eq!(run.summary.completeness(), ValidationCompleteness::Complete);
    let mut book = run.workbook(2);
    assert_eq!(book.sheet_names(), ["Riepilogo", "Errori"]);
    assert!(read_errors(&mut book).is_empty());
    assert_eq!(book.worksheet_range("Errori").unwrap().height(), 1);
    assert_summary(&mut book, &run.summary);
}

#[test]
fn partial_summary_retains_actual_missing_and_degraded_reference_audit() {
    for readable in [false, true] {
        let run = Run::new(b"ABCDE05\nABCDE06\n", Some(readable));
        assert_eq!(run.summary.total_errors, 0);
        assert_eq!(run.summary.completeness(), ValidationCompleteness::Partial);
        let audit = run.summary.reference_validation.as_ref().unwrap();
        let counts = &audit.controls[0].evaluations;
        assert_eq!(counts.provisional_present, u64::from(readable));
        assert_eq!(counts.inconclusive, if readable { 1 } else { 2 });
        let mut book = run.workbook(XLSX_SHEET_ROWS);
        assert!(read_errors(&mut book).is_empty());
        assert_summary(&mut book, &run.summary);
    }
}

#[test]
fn formula_looking_diagnostics_remain_literal_strings_in_an_independent_reader() {
    let mut run = Run::new(b"0012305\n", None);
    // Synthetic report text only; keep the real run's identity, error codes,
    // row count and summary, changing no normative schema or engine rule.
    let mut source = Vec::new();
    for line in run
        .jsonl
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
    {
        let mut value: Value = serde_json::from_slice(line).unwrap();
        value["expected"] = json!("=1+1");
        value["message"] = json!("+05\tà<&\nseconda riga");
        serde_json::to_writer(&mut source, &value).unwrap();
        source.push(b'\n');
    }
    run.jsonl = source;
    let mut book = run.workbook(XLSX_SHEET_ROWS);
    let rows = read_errors(&mut book);
    assert_eq!(rows, error_rows(&run.jsonl));
    assert!(
        rows.iter()
            .all(|row| row[12] == "=1+1" && row[13] == "+05\tà<&\nseconda riga")
    );
}
