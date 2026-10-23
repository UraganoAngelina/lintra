//! Production export tests: actual JSONL/summary from a real synthetic run.
use lintra::*;
use std::{
    fs,
    io::{self, BufRead, BufReader, BufWriter, Cursor, Read, Write},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lintra xlsx writer à {} {} {}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self { root }
    }
    /// No Vec of records/errors: fixture generation, validation and JSONL stay
    /// streaming even in the explicit large memory probe.
    fn run(&self, count: u64) -> ValidationSummary {
        let input = self.root.join("synthetic.txt");
        let mut file = BufWriter::new(fs::File::create(&input).unwrap());
        for _ in 0..count {
            file.write_all(b"x\n").unwrap();
        }
        file.flush().unwrap();
        drop(file);
        let raw: RawFlowSchema =
            serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
        let mut repo = InMemorySchemaRepository::new();
        repo.insert(raw).unwrap();
        let service = ValidationService::new(Arc::new(SchemaRegistry::new(
            Arc::new(repo),
            SchemaCompiler,
            RawResolvedEncoding::Utf8,
        )));
        let request = ValidationRequest {
            schema_key: SchemaKey {
                flow_id: FlowId("SYNTHETIC".into()),
                version: SchemaVersion("v1".into()),
            },
            scope: ValidationScope::Trace(TraceValidationInput {
                trace_id: TraceId("T1".into()),
                source: InputSource::Path(input),
            }),
            mode: ValidationMode::LocalOnly,
            options: ValidationOptions {
                record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                newline_policy: NewlinePolicy::AutoLfOrCrlf,
                max_errors: None,
            },
        };
        let mut sink = JsonlSink::new(BufWriter::new(
            fs::File::create(self.root.join("errors.jsonl")).unwrap(),
        ));
        let outcome = service
            .validate(
                request,
                &mut sink,
                &mut |_| {},
                &CancellationToken::default(),
            )
            .unwrap();
        sink.finish().unwrap();
        outcome.summary
    }
    fn input(&self) -> BufReader<fs::File> {
        BufReader::new(fs::File::open(self.root.join("errors.jsonl")).unwrap())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn xml(bytes: &[u8], name: &str) -> String {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut s = String::new();
    archive
        .by_name(name)
        .unwrap()
        .read_to_string(&mut s)
        .unwrap();
    s
}
fn sheet(bytes: &[u8], index: usize) -> String {
    xml(bytes, &format!("xl/worksheets/sheet{index}.xml"))
}
fn values(xml: &str) -> Vec<String> {
    regex::Regex::new(r"<t(?: [^>]*)?>(.*?)</t>")
        .unwrap()
        .captures_iter(xml)
        .map(|c| {
            c[1].replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&quot;", "\"")
                .replace("&amp;", "&")
        })
        .collect()
}
#[test]
fn real_jsonl_export_preserves_order_text_zeros_formulas_and_explicit_layout() {
    let f = Fixture::new();
    let summary = f.run(3);
    let lines = f.input().lines().map(Result::unwrap).collect::<Vec<_>>();
    let mut source = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let mut value: serde_json::Value = serde_json::from_str(line).unwrap();
        value["record_identity"]["sioc_record_id"] = serde_json::json!(format!("{i:05}"));
        value["record_identity"]["fiscal_code"] = serde_json::json!("SYNTHETIC_ONLY");
        value["actual"] = serde_json::json!({"display":"05","raw_hex":null});
        value["expected"] = serde_json::json!("=1+1");
        value["message"] = serde_json::json!("à<&");
        serde_json::to_writer(&mut source, &value).unwrap();
        source.push(b'\n');
    }
    let bytes = XlsxReportWriter::new()
        .write(Cursor::new(source), &summary, Vec::new())
        .unwrap();
    let errors = sheet(&bytes, 2);
    assert!(!errors.contains("<f>"));
    assert!(!errors.contains("t=\"n\""));
    assert!(!errors.contains("t=\"s\""));
    assert!(errors.contains("state=\"frozen\""));
    assert!(errors.contains("autoFilter ref=\"A1:N4\""));
    assert!(errors.contains("customWidth=\"1\""));
    let cells = values(&errors);
    let ids = cells
        .iter()
        .filter(|v| v.starts_with("0000"))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(ids, ["00000", "00001", "00002"]);
    assert_eq!(cells.iter().filter(|v| *v == "05").count(), 3);
    assert_eq!(cells.iter().filter(|v| *v == "=1+1").count(), 3);
    assert_eq!(cells.iter().filter(|v| *v == "à<&").count(), 3);
    for column in ERROR_COLUMNS {
        assert!(cells.contains(&column.header().to_owned()));
    }
    let summary_xml = sheet(&bytes, 1);
    let summary_values = values(&summary_xml);
    assert!(
        summary_values
            .windows(2)
            .any(|v| v == ["/total_errors", "3"])
    );
    assert!(
        summary_values
            .windows(2)
            .any(|v| v == ["/validation_mode", "local_only"])
    );
    let props = xml(&bytes, "docProps/core.xml");
    let json: serde_json::Value =
        serde_json::from_slice(&write_run_summary(Vec::new(), &summary).unwrap()).unwrap();
    assert!(props.contains(json["started_at"].as_str().unwrap()));
}
#[test]
fn rollover_and_empty_report_keep_exact_headers_and_never_add_empty_continuation() {
    for count in [0, 2, 3, 4] {
        let f = Fixture::new();
        let summary = f.run(count);
        let bytes = XlsxReportWriter::with_sheet_row_limit(3)
            .unwrap()
            .write(f.input(), &summary, Vec::new())
            .unwrap();
        let metadata = xml(&bytes, "xl/workbook.xml");
        let expected = if count > 2 { 2 } else { 1 };
        assert_eq!(metadata.matches("name=\"Errori").count(), expected);
        for index in 0..expected {
            let errors = sheet(&bytes, index + 2);
            assert_eq!(
                values(&errors)
                    .iter()
                    .filter(|v| v.as_str() == "code")
                    .count(),
                1
            );
            assert!(errors.contains("autoFilter ref="));
        }
        if count > 2 {
            assert!(metadata.contains("name=\"Errori (2)\""));
        }
        let total = (0..expected)
            .map(|i| sheet(&bytes, i + 2).matches("<row ").count() - 1)
            .sum::<usize>();
        assert_eq!(total, count as usize);
    }
    for limit in [0, 1, XLSX_SHEET_ROWS + 1] {
        assert!(matches!(
            XlsxReportWriter::with_sheet_row_limit(limit),
            Err(XlsxReportError::InvalidRowLimit)
        ));
    }
    assert!(XlsxReportWriter::with_sheet_row_limit(XLSX_SHEET_ROWS).is_ok());
}
#[test]
fn malformed_missing_or_mixed_jsonl_is_rejected_before_output_bytes() {
    let f = Fixture::new();
    let summary = f.run(1);
    let original = fs::read(f.root.join("errors.jsonl")).unwrap();
    let mut wrong: serde_json::Value = serde_json::from_slice(&original).unwrap();
    wrong["trace"] = serde_json::json!("OTHER");
    let mut duplicate = original.clone();
    duplicate.extend_from_slice(&original);
    let mut bad_code: serde_json::Value = serde_json::from_slice(&original).unwrap();
    bad_code["code"] = serde_json::json!("NOT_A_CODE");
    let mut bad_line: serde_json::Value = serde_json::from_slice(&original).unwrap();
    bad_line["line"] = serde_json::json!(0);
    for source in [
        vec![],
        b"\n".to_vec(),
        b"{broken\n".to_vec(),
        [b"\xef\xbb\xbf".as_slice(), original.as_slice()].concat(),
        duplicate,
        serde_json::to_vec(&wrong).unwrap(),
        serde_json::to_vec(&bad_code).unwrap(),
        serde_json::to_vec(&bad_line).unwrap(),
    ] {
        let mut output = Vec::new();
        assert!(
            XlsxReportWriter::new()
                .write(Cursor::new(source), &summary, &mut output)
                .is_err()
        );
        assert!(output.is_empty());
    }
    let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
    value["message"] = serde_json::json!("X".repeat(32768));
    assert!(matches!(
        XlsxReportWriter::new().write(
            Cursor::new(serde_json::to_vec(&value).unwrap()),
            &summary,
            Vec::new()
        ),
        Err(XlsxReportError::Backend(
            rust_xlsxwriter::XlsxError::MaxStringLengthExceeded
        ))
    ));
}
#[test]
fn io_failures_and_cancellation_remain_technical_and_keep_their_causes() {
    let f = Fixture::new();
    let mut summary = f.run(1);
    struct BrokenRead;
    impl Read for BrokenRead {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "synthetic reader",
            ))
        }
    }
    let result = XlsxReportWriter::new().write(BufReader::new(BrokenRead), &summary, Vec::new());
    assert!(
        matches!(result,Err(XlsxReportError::Read(source))if source.kind()==io::ErrorKind::PermissionDenied)
    );
    struct BrokenWrite;
    impl Write for BrokenWrite {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::StorageFull,
                "synthetic output",
            ))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    assert!(matches!(
        XlsxReportWriter::new().write(f.input(), &summary, BrokenWrite),
        Err(XlsxReportError::Backend(_))
    ));
    assert!(matches!(
        XlsxReportWriter::new()
            .with_temp_dir(f.root.join("absent"))
            .write(f.input(), &summary, Vec::new()),
        Err(XlsxReportError::Backend(_))
    ));
    summary.run_status = RunStatus::Cancelled;
    assert!(matches!(
        XlsxReportWriter::new().write(f.input(), &summary, Vec::new()),
        Err(XlsxReportError::CancelledRun)
    ));
}
#[test]
#[ignore = "explicit memory probe in a separate foreground process; LINTRA_XLSX_PROBE_ROWS controls size"]
fn xlsx_memory_probe() {
    let count = std::env::var("LINTRA_XLSX_PROBE_ROWS")
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let f = Fixture::new();
    let summary = f.run(count);
    let output = f.root.join("report.xlsx");
    let file = XlsxReportWriter::new()
        .write(f.input(), &summary, fs::File::create(&output).unwrap())
        .unwrap();
    drop(file);
    println!(
        "rows={count}, jsonl_bytes={}, xlsx_bytes={}",
        fs::metadata(f.root.join("errors.jsonl")).unwrap().len(),
        fs::metadata(&output).unwrap().len()
    );
    #[cfg(target_os = "linux")]
    {
        let status = fs::read_to_string("/proc/self/status").unwrap();
        println!(
            "{}",
            status
                .lines()
                .find(|line| line.starts_with("VmHWM:"))
                .unwrap()
        );
    }
}

#[test]
fn summary_sheet_preserves_real_partial_reference_evidence() {
    let f = Fixture::new();
    let input = f.root.join("valid.txt");
    fs::write(&input, b"ABCDEF\n").unwrap();
    let mut json: serde_json::Value =
        serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
    json["reference_data"] = serde_json::json!([{"id":"A","dataset_id":"synthetic-codes","version":"1","path":"absent.json","sha256":"0".repeat(64),"metadata_attributes":[]}]);
    json["traces"][0]["fields"][0]["constraints"] = serde_json::json!([{"kind":"reference_data","datasets":["A"],"fallback_values":[],"metadata_bindings":[]}]);
    let mut repo = InMemorySchemaRepository::new();
    repo.insert(serde_json::from_value(json).unwrap()).unwrap();
    let service = ValidationService::new(Arc::new(SchemaRegistry::new(
        Arc::new(repo),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    )))
    .with_reference_data(
        Arc::new(FileSystemReferenceDataPreparer::new(f.root.clone()).unwrap()),
        DatasetPreparationPolicy::ContinueWithAvailableEvidence,
    );
    let request = ValidationRequest {
        schema_key: SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        scope: ValidationScope::Trace(TraceValidationInput {
            trace_id: TraceId("T1".into()),
            source: InputSource::Path(input),
        }),
        mode: ValidationMode::LocalOnly,
        options: ValidationOptions {
            record_length_policy: RecordLengthPolicy::StrictSkipRecord,
            newline_policy: NewlinePolicy::AutoLfOrCrlf,
            max_errors: None,
        },
    };
    let mut sink = JsonlSink::new(Vec::new());
    let out = service
        .validate(
            request,
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    let source = sink.finish().unwrap();
    assert!(source.is_empty());
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
    let bytes = XlsxReportWriter::new()
        .write(Cursor::new(source), &out.summary, Vec::new())
        .unwrap();
    let entries = values(&sheet(&bytes, 1));
    for pair in [
        ["/reference_validation/completeness", "partial"],
        [
            "/reference_validation/datasets/0/availability",
            "unavailable",
        ],
        [
            "/reference_validation/controls/0/evaluations/inconclusive",
            "1",
        ],
    ] {
        assert!(entries.windows(2).any(|value| value == pair));
    }
}
