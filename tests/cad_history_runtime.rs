//! Full 83-field CAD profile through filesystem schema/catalogs and the public
//! snapshot service. Every value below is a public synthetic fixture, no PHI.
use lintra::*;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
const VERSION: &str = "2024-06.runtime-candidate.1";
const KEY: &str = "public-cad83-fixture";
const EVENT: &str = "000000000001";
struct Keys;
impl PersistenceKeyProvider for Keys {
    fn resolve(&self, id: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        if id != KEY {
            return Err(ProtectionError::MissingKey);
        }
        Ok(Arc::new(PersistenceKey::new(vec![0xaa; 32])?))
    }
}
struct FixedClock;
impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_794_225_600)
    }
}
fn profile_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schemas/operational/CAD")
        .join(VERSION)
}
fn declaration() -> Value {
    serde_json::from_str(include_str!(
        "../schemas/operational/CAD/2024-06.runtime-candidate.1/flow.json"
    ))
    .unwrap()
}
fn key() -> SchemaKey {
    SchemaKey {
        flow_id: FlowId("CAD".into()),
        version: SchemaVersion(VERSION.into()),
    }
}
fn namespace(unit: &str) -> PersistenceNamespace {
    PersistenceNamespace::new(vec![(
        "sending_ulss".into(),
        NamespaceValue::String(unit.into()),
    )])
    .unwrap()
}
fn row(trace: &str, values: &[(&str, &str)]) -> Vec<u8> {
    let schema = declaration();
    let t = schema["traces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == trace)
        .unwrap();
    let mut bytes = vec![b' '; t["record_length"].as_u64().unwrap() as usize];
    for field in t["fields"].as_array().unwrap() {
        let start = field["physical"]["start"].as_u64().unwrap() as usize - 1;
        let width = field["physical"]["length"].as_u64().unwrap() as usize;
        if field["missing_value"]["kind"] == "exact" {
            let missing = field["missing_value"]["raw"].as_str().unwrap();
            assert_eq!(missing.len(), width);
            bytes[start..start + width].copy_from_slice(missing.as_bytes());
        }
        if let Some((_, value)) = values.iter().find(|(id, _)| field["id"] == *id) {
            assert!(value.len() <= width, "{}", field["id"]);
            bytes[start..start + width].fill(b' ');
            bytes[start..start + value.len()].copy_from_slice(value.as_bytes());
        }
    }
    for (id, _) in values {
        assert!(
            t["fields"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f["id"] == *id),
            "unknown source field {id}"
        );
    }
    bytes
}
fn demographic(unit: &str) -> Vec<u8> {
    row(
        "A",
        &[
            ("1.1", unit),
            ("1.2", "PUBLIC-FIXTURE-00001"),
            ("1.3", "1"),
            ("1.10", "1"),
            ("1.11", "01011980"),
            ("1.12", "999999"),
            ("1.13", "100"),
            ("1.14", "999999"),
            ("1.15", "050501"),
            ("1.19", "1"),
            ("1.20", "01"),
            ("1.21", "1"),
            ("1.22", "1"),
            ("1.23", EVENT),
        ],
    )
}
fn evaluation(kind: &str, date: &str, progressive: u64) -> Vec<u8> {
    let sequence = format!("{progressive:012}");
    let bits = format!("{} {}", "2".repeat(59), "2".repeat(10));
    row(
        "B1",
        &[
            ("2.1", EVENT),
            ("2.2", "01"),
            ("2.3", "1"),
            ("2.4", "01012026"),
            ("2.8", kind),
            ("2.9", date),
            ("2.11", "000"),
            ("2.12", "000"),
            ("2.13", "1"),
            ("2.14", "1"),
            ("2.17", "1"),
            ("2.18", "1"),
            ("2.20", "1"),
            ("2.21", "1"),
            ("2.22", &bits),
            ("2.23", "1"),
            ("2.36", &sequence),
        ],
    )
}
fn access(date: &str) -> Vec<u8> {
    row(
        "C",
        &[
            ("4.1", EVENT),
            ("4.2", date),
            ("4.3", "1"),
            ("4.4", "1"),
            ("4.5", "03"),
            ("4.7", "1"),
            ("4.8", "2"),
            ("4.11", "000000000001"),
        ],
    )
}
fn provision(date: &str) -> Vec<u8> {
    row(
        "D",
        &[
            ("5.1", EVENT),
            ("5.2", date),
            ("5.3", "1"),
            ("5.4", "03"),
            ("5.5", "01"),
            ("5.6", "01"),
            ("5.7", "000000000099"),
        ],
    )
}
fn rows(unit: &str, quarter: u8) -> BTreeMap<String, Vec<Vec<u8>>> {
    let date = match quarter {
        1 => "01032026",
        2 => "01062026",
        _ => "01092026",
    };
    let mut evaluations = vec![evaluation("1", "01012026", 1)];
    if quarter > 1 {
        evaluations.push(evaluation("2", date, 2));
    }
    BTreeMap::from([
        ("A".into(), vec![demographic(unit)]),
        ("B1".into(), evaluations),
        ("B2".into(), vec![]),
        ("C".into(), vec![access(date)]),
        ("D".into(), vec![provision(date)]),
    ])
}
struct Fixture {
    dir: tempfile::TempDir,
    service: ValidationService,
}
impl Fixture {
    fn new() -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas/operational");
        let registry = Arc::new(SchemaRegistry::new(
            Arc::new(FileSystemSchemaRepository::new(root).unwrap()),
            SchemaCompiler,
            RawResolvedEncoding::Utf8,
        ));
        let service = ValidationService::with_clock(registry, Arc::new(FixedClock))
            .with_reference_data(
                Arc::new(FileSystemReferenceDataPreparer::new(profile_root()).unwrap()),
                DatasetPreparationPolicy::Strict,
            );
        Self {
            dir: tempfile::tempdir_in("target").unwrap(),
            service,
        }
    }
    fn request(
        &self,
        id: &str,
        unit: &str,
        quarter: u8,
        rows: &BTreeMap<String, Vec<Vec<u8>>>,
    ) -> HistoricalSnapshotValidationRequest {
        let mut traces = Vec::new();
        for (trace, records) in rows.iter().rev() {
            let path = self.dir.path().join(format!("{trace}.dat"));
            let mut bytes = Vec::new();
            for record in records {
                bytes.extend(record);
                bytes.push(b'\n');
            }
            fs::write(&path, bytes).unwrap();
            traces.push(TraceValidationInput {
                trace_id: TraceId(trace.clone()),
                source: InputSource::Path(path),
            });
        }
        HistoricalSnapshotValidationRequest {
            validation: HistoricalValidationRequest {
                validation: ValidationRequest {
                    schema_key: key(),
                    scope: ValidationScope::Flow(FlowValidationInput { traces }),
                    mode: ValidationMode::Full,
                    options: ValidationOptions {
                        record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                        newline_policy: NewlinePolicy::AutoLfOrCrlf,
                        max_errors: None,
                    },
                },
                parameters: BTreeMap::from([
                    ("sending_ulss".into(), NamespaceValue::String(unit.into())),
                    ("year".into(), NamespaceValue::Integer(2026)),
                    (
                        "quarter".into(),
                        NamespaceValue::Integer(i64::from(quarter)),
                    ),
                ]),
                submission_id: id.into(),
            },
            proceed_consent: None,
            origins: Vec::new(),
        }
    }
}
#[derive(Default)]
struct Audit;
impl HistoricalAuditSink for Audit {
    fn on_diff(&mut self, _: &HistoricalDiff) -> Result<(), ReportError> {
        Ok(())
    }
    fn on_summary(&mut self, _: &HistoricalAuditSummary) -> Result<(), ReportError> {
        Ok(())
    }
}
fn execute<'a>(
    fixture: &Fixture,
    repo: &'a SQLitePersistenceAdapter,
    request: HistoricalSnapshotValidationRequest,
) -> Result<(HistoricalValidationOutcome<'a>, VecSink), HistoricalValidationError> {
    let protector = IdentityProtector::preflight(Some(KEY), &Keys, None).unwrap();
    let mut sink = VecSink::default();
    let result = HistoricalValidationService::new(&fixture.service, repo, &protector)
        .validate_snapshot(
            request,
            &mut sink,
            &mut Audit,
            &mut |_| {},
            &CancellationToken::default(),
        )?;
    Ok((result, sink))
}
fn archive() -> (tempfile::TempDir, SQLitePersistenceAdapter) {
    let dir = tempfile::tempdir_in("target").unwrap();
    let repo =
        SQLitePersistenceAdapter::create(&dir.path().join("archive"), Some(KEY), &Keys).unwrap();
    (dir, repo)
}
fn approve(outcome: HistoricalValidationOutcome<'_>) {
    let HistoricalValidationOutcome::Preview(mut preview) = outcome else {
        panic!("full CAD must produce a real preview")
    };
    let review = preview.review();
    preview
        .approve_reviewed(&review, &CancellationToken::default(), |_, _, _| Ok(()))
        .unwrap();
}
fn summary<'a>(outcome: &'a HistoricalValidationOutcome<'_>) -> &'a ValidationSummary {
    match outcome {
        HistoricalValidationOutcome::Preview(p) => &p.outcome().summary,
        HistoricalValidationOutcome::ValidationOnly(o) => &o.summary,
    }
}
fn set(trace: &str, record: &mut [u8], id: &str, value: &str) {
    let v = declaration();
    let t = v["traces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == trace)
        .unwrap();
    let f = t["fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] == id)
        .unwrap();
    let start = f["physical"]["start"].as_u64().unwrap() as usize - 1;
    let width = f["physical"]["length"].as_u64().unwrap() as usize;
    assert!(value.len() <= width);
    record[start..start + width].fill(b' ');
    record[start..start + value.len()].copy_from_slice(value.as_bytes());
}
fn has_code(sink: &VecSink, code: &str) -> bool {
    sink.errors()
        .iter()
        .any(|e| e.code.as_symbol().unwrap() == code)
}
fn origin() -> HistoricalOriginAttestation {
    let document=b"Public synthetic evidence: namespace501 entity000000000001 begins in2026Q1. No patient data.";
    HistoricalOriginAttestation::from_document(
        HistoricalOriginClaim {
            join_id: JoinId("B1_INITIAL_HISTORY".into()),
            correlation: vec![EVENT.as_bytes().to_vec()],
            year: 2026,
            quarter: 1,
            reference: "cad83-public-origin-document".into(),
            responsible: "public-synthetic-curator".into(),
            recorded_at: "2026-01-01T00:00:00Z".into(),
        },
        document,
        Sha256Digest(Sha256::digest(document).into()),
    )
    .unwrap()
}

#[test]
fn complete_cad83_t1_t2_t3_with_real_catalogs_reopened_sqlite_and_independent_progressives() {
    let f = Fixture::new();
    let (dir, mut repo) = archive();
    let expected: Value = serde_json::from_str(include_str!(
        "golden/cross-trace/cad-history/2024-06.runtime-candidate.1/expected-timeline.json"
    ))
    .unwrap();
    let mut timeline = Vec::new();
    for quarter in 1..=3 {
        let records = rows("501", quarter);
        let (outcome, sink) = execute(
            &f,
            &repo,
            f.request(&format!("T{quarter}"), "501", quarter, &records),
        )
        .unwrap();
        let summary = match &outcome {
            HistoricalValidationOutcome::Preview(p) => &p.outcome().summary,
            HistoricalValidationOutcome::ValidationOnly(o) => &o.summary,
        };
        assert!(sink.errors().is_empty(), "q{quarter}: {:?}", sink.errors());
        assert_eq!(summary.run_status, RunStatus::CompletedValid);
        assert_eq!(
            summary.completeness(),
            ValidationCompleteness::Complete,
            "{:?}",
            summary.full_validation
        );
        assert_eq!(summary.inputs.len(), 5);
        let trace_counts = summary
            .full_validation
            .as_ref()
            .unwrap()
            .traces
            .iter()
            .map(|t| (t.trace_id.0.clone(), t.records_processed))
            .collect::<BTreeMap<_, _>>();
        timeline.push(serde_json::json!({"quarter":quarter,"status":summary.run_status,"completeness":summary.completeness(),"records":trace_counts,"historical_facts":if quarter == 1 {1} else {2}}));
        assert_eq!(
            summary
                .reference_validation
                .as_ref()
                .unwrap()
                .datasets
                .len(),
            2
        );
        approve(outcome);
        drop(repo);
        repo =
            SQLitePersistenceAdapter::open(&dir.path().join("archive"), Some(KEY), &Keys).unwrap();
        let snapshot = repo
            .load_snapshot(&namespace("501"), SnapshotRevision::new(u64::from(quarter)))
            .unwrap()
            .unwrap();
        assert_eq!(snapshot.source().state(), SessionState::ApprovedImmutable);
        assert_eq!(snapshot.history_facts().len(), 1);
        assert_eq!(
            snapshot.history_facts()[0].fact_count(),
            if quarter == 1 { 1 } else { 2 }
        );
    }
    assert_eq!(serde_json::json!(timeline), expected);
}

#[test]
fn cad83_historical_violation_jsonl_has_a_fixed_normative_oracle_and_stable_bytes() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    let mut records = rows("501", 1);
    records.insert("B1".into(), vec![evaluation("2", "01032026", 2)]);
    let mut outputs = Vec::new();
    for _ in 0..2 {
        let mut request = f.request("negative-golden", "501", 1, &records);
        request.origins.push(origin());
        let protector = IdentityProtector::preflight(Some(KEY), &Keys, None).unwrap();
        let mut sink = JsonlSink::new(Vec::new());
        let outcome = HistoricalValidationService::new(&f.service, &repo, &protector)
            .validate_snapshot(
                request,
                &mut sink,
                &mut Audit,
                &mut |_| {},
                &CancellationToken::default(),
            )
            .unwrap();
        assert_eq!(summary(&outcome).run_status, RunStatus::CompletedInvalid);
        assert!(matches!(
            outcome,
            HistoricalValidationOutcome::ValidationOnly(_)
        ));
        outputs.push(sink.finish().unwrap());
    }
    assert_eq!(outputs[0], outputs[1]);
    let lines = std::str::from_utf8(&outputs[0])
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(lines[0]["code"], "E_CAD_INITIAL_EVALUATION_REQUIRED");
    assert_eq!(lines[0]["trace"], "B1");
    assert_eq!(lines[0]["line"], 1);
    assert_eq!(
        lines[0]["message"],
        "A reevaluation requires a valid initial evaluation in approved history or current input."
    );
    assert_eq!(repo.current_generation(&namespace("501")).unwrap(), None);
}

#[test]
fn missing_b2_is_partial_with_exact_consent_and_cannot_publish_facts() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    let mut records = rows("501", 1);
    records.remove("B2");
    let mut request = f.request("missing-B2", "501", 1, &records);
    request.proceed_consent = Some(
        MissingTraceProceedConsent::new(vec![TraceId("B2".into())], FixedClock.now().into())
            .unwrap(),
    );
    let (outcome, _) = execute(&f, &repo, request).unwrap();
    assert_eq!(summary(&outcome).run_status, RunStatus::CompletedValid);
    assert_eq!(
        summary(&outcome).completeness(),
        ValidationCompleteness::Partial
    );
    assert_eq!(summary(&outcome).inputs.len(), 4);
    assert!(matches!(
        outcome,
        HistoricalValidationOutcome::ValidationOnly(_)
    ));
    assert_eq!(repo.current_generation(&namespace("501")).unwrap(), None);
}

/// Explicit measurement, not a timing-sensitive pass/fail threshold. The caller
/// records RSS/IO externally and chooses the event count; fixture construction
/// is excluded from timings. Each sample includes the actual catalogs, both
/// passes, history, review, atomic approval and SQLite close/reopen.
#[test]
#[ignore = "explicit CAD83 application performance qualification"]
fn measure_full_cad83_history_application() {
    let count = std::env::var("LINTRA_CAD83_EVENT_COUNT")
        .unwrap_or_else(|_| "100".into())
        .parse::<usize>()
        .unwrap();
    assert!((1..=10_000).contains(&count));
    let f = Fixture::new();
    let (dir, mut repo) = archive();
    let schema = declaration();
    let mut milliseconds = Vec::new();
    for unit in ["501", "502"] {
        for quarter in 1..=3 {
            let base = rows(unit, quarter);
            let mut records = BTreeMap::new();
            for (trace, event_id, progressive_id) in [
                ("A", "1.23", None),
                ("B1", "2.1", Some("2.36")),
                ("B2", "3.1", Some("3.5")),
                ("C", "4.1", Some("4.11")),
                ("D", "5.1", Some("5.7")),
            ] {
                let declaration = schema["traces"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|t| t["id"] == trace)
                    .unwrap();
                let mut expanded = Vec::new();
                for event in 1..=count {
                    for (position, record) in base[trace].iter().enumerate() {
                        let mut record = record.clone();
                        for (id, number) in [
                            (Some(event_id), event),
                            (progressive_id, event * 10 + position),
                        ] {
                            if let Some(id) = id {
                                let field = declaration["fields"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .find(|v| v["id"] == id)
                                    .unwrap();
                                let start =
                                    field["physical"]["start"].as_u64().unwrap() as usize - 1;
                                let width = field["physical"]["length"].as_u64().unwrap() as usize;
                                let value = format!("{number:0width$}");
                                record[start..start + width].copy_from_slice(value.as_bytes());
                            }
                        }
                        expanded.push(record);
                    }
                }
                records.insert(trace.into(), expanded);
            }
            let request = f.request(
                &format!("measure-{unit}-Q{quarter}"),
                unit,
                quarter,
                &records,
            );
            let started = std::time::Instant::now();
            let (outcome, sink) = execute(&f, &repo, request).unwrap();
            assert!(sink.errors().is_empty(), "{:?}", sink.errors());
            assert_eq!(
                summary(&outcome).completeness(),
                ValidationCompleteness::Complete
            );
            approve(outcome);
            drop(repo);
            repo = SQLitePersistenceAdapter::open(&dir.path().join("archive"), Some(KEY), &Keys)
                .unwrap();
            milliseconds.push(started.elapsed().as_secs_f64() * 1000.0);
        }
    }
    let mut sorted = milliseconds.clone();
    sorted.sort_by(f64::total_cmp);
    let db_bytes = fs::metadata(dir.path().join("archive/lintra.db"))
        .unwrap()
        .len();
    println!(
        "CAD83_MEASUREMENT {}",
        serde_json::json!({"events_per_namespace":count,"namespaces":2,"approved_snapshots":6,"samples_ms":milliseconds,"empirical_p95_ms":sorted[((sorted.len() as f64*0.95).ceil() as usize)-1],"database_bytes":db_bytes,"profile":"CAD/2024-06.runtime-candidate.1"})
    );
}

#[test]
fn source_mapping_retains_all_83_notes_layouts_and_complete_normative_keys() {
    let v = declaration();
    let mut count = 0;
    for trace in v["traces"].as_array().unwrap() {
        let id = trace["id"].as_str().unwrap();
        let expected: Value = serde_json::from_slice(
            &fs::read(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("docs/normative/cad/{id}-fields.json")),
            )
            .unwrap(),
        )
        .unwrap();
        for field in trace["fields"].as_array().unwrap() {
            let source = expected["fields"]
                .as_array()
                .unwrap()
                .iter()
                .find(|f| f["id"] == field["id"])
                .unwrap();
            assert_eq!(field["name"], source["name"]);
            assert_eq!(field["regulatory_metadata"], source["regulatory_metadata"]);
            for property in ["start", "length", "type"] {
                assert_eq!(
                    field["physical"][property], source["physical"][property],
                    "{} {property}",
                    field["id"]
                );
            }
            count += 1;
        }
    }
    assert_eq!(count, 83);
    let c = v["traces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == "C")
        .unwrap();
    assert_eq!(
        c["record_key"]["fields"],
        serde_json::json!(["4.1", "4.2", "4.4", "4.5", "4.11"])
    );
    let d = v["traces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == "D")
        .unwrap();
    assert_eq!(
        d["record_key"]["fields"],
        serde_json::json!(["5.1", "5.2", "5.3", "5.4", "5.7"])
    );
    let join = d["joins"]
        .as_array()
        .unwrap()
        .iter()
        .find(|j| j["id"] == "D_TO_C_ACCESS")
        .unwrap();
    assert_eq!(
        join["source_fields"],
        serde_json::json!(["5.1", "5.2", "5.3", "5.4"])
    );
    assert_eq!(v["reference_data"].as_array().unwrap().len(), 2);
}
#[test]
fn cad_initial_annulled_in_t2_does_not_resurrect_for_t3_reevaluation() {
    let f = Fixture::new();
    let (dir, repo) = archive();
    let mut t1 = f.request("T1", "501", 1, &rows("501", 1));
    t1.origins.push(origin());
    approve(execute(&f, &repo, t1).unwrap().0);
    drop(repo);
    let repo =
        SQLitePersistenceAdapter::open(&dir.path().join("archive"), Some(KEY), &Keys).unwrap();
    let empty = BTreeMap::from([
        ("A".into(), vec![]),
        ("B1".into(), vec![]),
        ("B2".into(), vec![]),
        ("C".into(), vec![]),
        ("D".into(), vec![]),
    ]);
    approve(
        execute(&f, &repo, f.request("T2", "501", 2, &empty))
            .unwrap()
            .0,
    );
    let mut records = rows("501", 3);
    records.get_mut("B1").unwrap().remove(0);
    let (result, sink) = execute(&f, &repo, f.request("T3-invalid", "501", 3, &records)).unwrap();
    assert_eq!(summary(&result).run_status, RunStatus::CompletedInvalid);
    assert!(has_code(&sink, "E_CAD_INITIAL_EVALUATION_REQUIRED"));
    assert!(matches!(
        result,
        HistoricalValidationOutcome::ValidationOnly(_)
    ));
    assert_eq!(
        repo.current_generation(&namespace("501")).unwrap(),
        Some(SnapshotRevision::new(2))
    );
    approve(
        execute(&f, &repo, f.request("T3", "501", 3, &rows("501", 3)))
            .unwrap()
            .0,
    );
}
#[test]
fn cad_namespaces_with_the_same_event_are_independent_and_unknown_is_not_a_violation() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    approve(
        execute(&f, &repo, f.request("T1-501", "501", 1, &rows("501", 1)))
            .unwrap()
            .0,
    );
    let mut other = rows("502", 1);
    other.insert("B1".into(), vec![evaluation("2", "01032026", 2)]);
    let (result, sink) = execute(&f, &repo, f.request("T1-502-unknown", "502", 1, &other)).unwrap();
    assert_eq!(summary(&result).run_status, RunStatus::CompletedValid);
    assert_eq!(
        summary(&result).completeness(),
        ValidationCompleteness::Partial
    );
    assert!(!has_code(&sink, "E_CAD_INITIAL_EVALUATION_REQUIRED"));
    assert!(matches!(
        result,
        HistoricalValidationOutcome::ValidationOnly(_)
    ));
    approve(
        execute(&f, &repo, f.request("T1-502", "502", 1, &rows("502", 1)))
            .unwrap()
            .0,
    );
    for unit in ["501", "502"] {
        assert_eq!(
            repo.current_generation(&namespace(unit)).unwrap(),
            Some(SnapshotRevision::new(1))
        );
    }
}
#[test]
fn normative_dates_professional_15_and_invalid_dates_keep_distinct_outcomes() {
    let f = Fixture::new();
    for (start, invalid) in [("01012024", false), ("01012025", true), ("99012025", false)] {
        let (_dir, repo) = archive();
        let mut records = rows("501", 1);
        set("B1", &mut records.get_mut("B1").unwrap()[0], "2.4", start);
        set("B1", &mut records.get_mut("B1").unwrap()[0], "2.9", start);
        set("C", &mut records.get_mut("C").unwrap()[0], "4.5", "15");
        set("D", &mut records.get_mut("D").unwrap()[0], "5.4", "15");
        let (result, sink) = execute(&f, &repo, f.request("T1", "501", 1, &records)).unwrap();
        assert_eq!(
            has_code(&sink, "E_CAD_PROFESSIONAL_15_AFTER_2024"),
            invalid,
            "{start}: {:?}",
            sink.errors()
        );
        if start == "01012024" {
            assert_eq!(summary(&result).run_status, RunStatus::CompletedValid);
            assert!(sink.errors().is_empty());
        } else {
            assert_eq!(summary(&result).run_status, RunStatus::CompletedInvalid);
        }
    }
}
#[test]
fn all_suspension_reasons_apply_to_interior_home_access_and_boundaries_remain_allowed() {
    let f = Fixture::new();
    for (begin, end, violation) in [
        ("01022026", "01042026", true),
        ("01032026", "01042026", false),
        ("01022026", "01032026", false),
    ] {
        let (_dir, repo) = archive();
        let mut records = rows("501", 1);
        records.insert(
            "B2".into(),
            vec![row(
                "B2",
                &[
                    ("3.1", EVENT),
                    ("3.2", begin),
                    ("3.3", end),
                    ("3.4", "7"),
                    ("3.5", "000000000002"),
                ],
            )],
        );
        let (result, sink) = execute(&f, &repo, f.request("T1", "501", 1, &records)).unwrap();
        assert_eq!(
            has_code(&sink, "E_CAD_HOME_ACCESS_DURING_SUSPENSION"),
            violation,
            "{:?}",
            sink.errors()
        );
        assert_eq!(
            summary(&result).run_status,
            if violation {
                RunStatus::CompletedInvalid
            } else {
                RunStatus::CompletedValid
            }
        );
    }
}
#[test]
fn missing_d_with_proceed_is_partial_validation_only_and_not_a_fake_empty_file() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    let mut records = rows("501", 1);
    records.remove("D");
    let mut request = f.request("T1", "501", 1, &records);
    request.proceed_consent = Some(
        MissingTraceProceedConsent::new(
            vec![TraceId("D".into())],
            chrono::DateTime::parse_from_rfc3339("2026-03-01T00:00:00Z")
                .unwrap()
                .to_utc(),
        )
        .unwrap(),
    );
    let (result, sink) = execute(&f, &repo, request).unwrap();
    assert_eq!(summary(&result).run_status, RunStatus::CompletedValid);
    assert_eq!(
        summary(&result).completeness(),
        ValidationCompleteness::Partial
    );
    assert!(sink.errors().is_empty());
    assert!(matches!(
        result,
        HistoricalValidationOutcome::ValidationOnly(_)
    ));
    assert_eq!(repo.current_generation(&namespace("501")).unwrap(), None);
}
