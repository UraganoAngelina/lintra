//! Synthetic monthly submissions through the real Rust service and repositories.
//! No inferred SIOC rules: the generic fixture explicitly declares every mode.
use lintra::*;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs, io,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

struct Key;
impl PersistenceKeyProvider for Key {
    fn resolve(&self, _: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        Ok(Arc::new(PersistenceKey::new(vec![0xab; 32])?))
    }
}
fn protector() -> IdentityProtector {
    IdentityProtector::preflight(Some("synthetic"), &Key, None).unwrap()
}
struct Fixed;
impl Clock for Fixed {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH
    }
}
struct Source(Value);
impl SchemaRepository for Source {
    fn load_raw(&self, _: &SchemaKey) -> Result<RawFlowSchema, SchemaLoadError> {
        Ok(serde_json::from_value(self.0.clone()).unwrap())
    }
}
const IDS: [&str; 5] = ["1", "2", "3", "4.1", "4.2"];
fn model() -> Value {
    let mut v: Value =
        serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
    v["schema_format_version"] = json!(3);
    v["schema_version"] = json!("monthly.1");
    v["required_capabilities"] = json!([
        "local.v1",
        "persistence.audit.v1",
        "persistence.cross_run.v1"
    ]);
    v["parameters"] = json!([
        {"id":"org","type":"integer","min":1,"max":100},
        {"id":"year","type":"integer","min":1900,"max":9999},
        {"id":"month","type":"integer","min":1,"max":12}
    ]);
    let mut trace = v["traces"][0].clone();
    trace["fields"][0]["physical"]["length"] = json!(4);
    let mut data = trace["fields"][0].clone();
    data["id"] = json!("DATA");
    data["physical"]["start"] = json!(5);
    data["physical"]["length"] = json!(2);
    trace["fields"].as_array_mut().unwrap().push(data);
    v["traces"] = Value::Array(
        IDS.iter()
            .map(|id| {
                let mut t = trace.clone();
                t["id"] = json!(id);
                t
            })
            .collect(),
    );
    let keys: Vec<_> = IDS
        .iter()
        .map(|id| json!({"trace":id,"key":"record_key"}))
        .collect();
    let modes: Vec<_> = IDS.iter().map(|id| json!({"trace":id,
        "mode":if ["1","2","3"].contains(id) { "cumulative_identical" } else { "reject_previously_seen" },
        "duplicate_in_second_pass":*id=="4.1", "reuse_error":"E_HISTORICAL_KEY_REUSED", "missing_error":"E_MISSING_PREVIOUS_RECORD"})).collect();
    v["persistence_policy"] = json!({"namespace_parameters":["org"],"version_compatibility":{"kind":"same_schema_version"},
        "run_uniqueness":keys,"cross_run_audit":keys,"history_identity_invariants":[],"snapshot":null,"temporal_rules":[],
        "storage_protection":"external_secure_configuration_required", "cross_run_validation":{
            "year_parameter":"year","month_parameter":"month","content_identity":"fixed_width_record_bytes_without_terminator.v1","traces":modes}});
    v
}
struct Fixture {
    temp: tempfile::TempDir,
    service: ValidationService,
    model: Value,
}
impl Fixture {
    fn new() -> Self {
        Self::with_model(model())
    }
    fn with_model(model: Value) -> Self {
        let registry = Arc::new(SchemaRegistry::new(
            Arc::new(Source(model.clone())),
            SchemaCompiler,
            RawResolvedEncoding::Utf8,
        ));
        let result = Self {
            temp: tempfile::tempdir().unwrap(),
            service: ValidationService::with_clock(registry, Arc::new(Fixed)),
            model,
        };
        for id in IDS {
            result.put(id, b"");
        }
        result
    }
    fn put(&self, trace: &str, bytes: &[u8]) {
        fs::write(self.temp.path().join(format!("{trace}.dat")), bytes).unwrap();
    }
    fn cumulative(&self, rows: &[u16]) {
        let text = rows
            .iter()
            .map(|v| format!("{v:04}AA\n"))
            .collect::<String>();
        for id in ["1", "2", "3"] {
            self.put(id, text.as_bytes());
        }
    }
    fn request(&self, id: &str, month: i64, org: i64) -> HistoricalValidationRequest {
        HistoricalValidationRequest {
            validation: ValidationRequest {
                schema_key: SchemaKey {
                    flow_id: FlowId(self.model["flow_id"].as_str().unwrap().into()),
                    version: SchemaVersion(self.model["schema_version"].as_str().unwrap().into()),
                },
                scope: ValidationScope::Flow(FlowValidationInput {
                    traces: IDS
                        .iter()
                        .map(|id| TraceValidationInput {
                            trace_id: TraceId((*id).into()),
                            source: InputSource::Path(self.temp.path().join(format!("{id}.dat"))),
                        })
                        .collect(),
                }),
                mode: ValidationMode::Full,
                options: ValidationOptions {
                    record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                    newline_policy: NewlinePolicy::AutoLfOrCrlf,
                    max_errors: None,
                },
            },
            parameters: BTreeMap::from([
                ("org".into(), NamespaceValue::Integer(org)),
                ("year".into(), NamespaceValue::Integer(2026)),
                ("month".into(), NamespaceValue::Integer(month)),
            ]),
            submission_id: id.into(),
        }
    }
    fn run<'a>(
        &self,
        repo: &'a dyn PersistentContextRepository,
        request: HistoricalValidationRequest,
    ) -> (HistoricalValidationOutcome<'a>, VecSink) {
        self.run_controlled(repo, request, &CancellationToken::default(), &mut |_| {})
    }
    fn run_controlled<'a>(
        &self,
        repo: &'a dyn PersistentContextRepository,
        request: HistoricalValidationRequest,
        cancellation: &CancellationToken,
        progress: &mut dyn ProgressObserver,
    ) -> (HistoricalValidationOutcome<'a>, VecSink) {
        let mut sink = VecSink::default();
        let result = HistoricalValidationService::new(&self.service, repo, &protector())
            .validate(request, &mut sink, &mut Audit, progress, cancellation)
            .unwrap();
        (result, sink)
    }
}
struct Audit;
impl HistoricalAuditSink for Audit {
    fn on_diff(&mut self, _: &HistoricalDiff) -> Result<(), ReportError> {
        Ok(())
    }
    fn on_summary(&mut self, _: &HistoricalAuditSummary) -> Result<(), ReportError> {
        Ok(())
    }
}
fn raw_repo() -> InMemoryPersistenceRepository {
    InMemoryPersistenceRepository::with_protection_audit(protector().audit().unwrap())
}
fn initialize(f: &Fixture, repo: &dyn PersistentContextRepository, org: i64) {
    // This is a real reviewed/approved empty submission, not an injected
    // repository witness or an audit-only promotion. Restore current inputs.
    let saved: Vec<_> = IDS
        .iter()
        .map(|id| fs::read(f.temp.path().join(format!("{id}.dat"))).unwrap())
        .collect();
    for id in IDS {
        f.put(id, b"");
    }
    let request = if f.model["flow_id"] == "SIOC" {
        sioc_request(f, "approved-empty-initial-baseline", 1)
    } else {
        f.request("approved-empty-initial-baseline", 1, org)
    };
    let mut initial = preview(f.run(repo, request).0);
    assert!(
        initial
            .outcome()
            .summary
            .full_validation
            .as_ref()
            .unwrap()
            .cross_run_validation
            .as_ref()
            .unwrap()
            .initial_baseline
    );
    commit(&mut initial);
    for (id, bytes) in IDS.iter().zip(saved) {
        f.put(id, &bytes);
    }
}
fn initialized_repo(f: &Fixture) -> InMemoryPersistenceRepository {
    let repo = raw_repo();
    initialize(f, &repo, 1);
    repo
}
fn preview(result: HistoricalValidationOutcome<'_>) -> Box<HistoricalValidationPreview<'_>> {
    match result {
        HistoricalValidationOutcome::Preview(v) => v,
        HistoricalValidationOutcome::ValidationOnly(v) => {
            panic!("unexpected non-preview {:?}", v.summary)
        }
    }
}
fn summary<'a>(result: &'a HistoricalValidationOutcome<'_>) -> &'a ValidationSummary {
    match result {
        HistoricalValidationOutcome::Preview(v) => &v.outcome().summary,
        HistoricalValidationOutcome::ValidationOnly(v) => &v.summary,
    }
}
fn stats(summary: &ValidationSummary, id: &str) -> CrossRunTraceSummary {
    summary
        .full_validation
        .as_ref()
        .unwrap()
        .cross_run_validation
        .as_ref()
        .unwrap()
        .traces
        .iter()
        .find(|v| v.trace_id.0 == id)
        .unwrap()
        .clone()
}
fn commit(p: &mut HistoricalValidationPreview<'_>) -> CommitReceipt {
    p.approve_commit(&CancellationToken::default(), |_, _, _| Ok(()))
        .unwrap()
}
fn namespace(org: i64) -> PersistenceNamespace {
    PersistenceNamespace::new(vec![("org".into(), NamespaceValue::Integer(org))]).unwrap()
}

#[test]
fn january_february_march_carry_forward_is_not_a_new_acquisition_and_replay_is_idempotent() {
    let f = Fixture::new();
    let repo = initialized_repo(&f);
    let cases = [
        ("jan", 1, vec![1001, 1002], 0, 2),
        ("feb", 2, vec![1001, 1002, 1003], 2, 1),
        ("mar", 3, vec![1001, 1002, 1003, 1004], 3, 1),
    ];
    for (id, month, rows, carry, new) in cases {
        f.cumulative(&rows);
        let (result, sink) = f.run(&repo, f.request(id, month, 1));
        assert!(sink.errors().is_empty());
        assert_eq!(
            summary(&result).completeness(),
            ValidationCompleteness::Complete
        );
        for trace in ["1", "2", "3"] {
            let s = stats(summary(&result), trace);
            assert_eq!((s.carry_forward_identical, s.new_records), (carry, new));
        }
        commit(&mut preview(result));
    }
    f.cumulative(&[1001, 1002]); // exact replay after subsequent monthly approvals
    let (result, _) = f.run(&repo, f.request("jan", 1, 1));
    assert_eq!(stats(summary(&result), "1").new_records, 0);
    let receipt = commit(&mut preview(result));
    assert_eq!(receipt.revision(), SnapshotRevision::new(2));
    assert_eq!(
        repo.current_generation(&namespace(1)).unwrap(),
        Some(SnapshotRevision::new(4))
    );
}

#[test]
fn changed_content_is_rejected_in_second_pass_and_omissions_do_not_erase_history() {
    let f = Fixture::new();
    let repo = initialized_repo(&f);
    f.cumulative(&[1001, 1002]);
    commit(&mut preview(f.run(&repo, f.request("jan", 1, 1)).0));
    f.put("1", b"1001BB\n1002AA\n");
    let (result, sink) = f.run(&repo, f.request("changed", 2, 1));
    assert_eq!(summary(&result).run_status, RunStatus::CompletedInvalid);
    assert_eq!(summary(&result).invalid_records, 1);
    assert_eq!(
        stats(summary(&result), "1").historical_key_content_mismatch,
        1
    );
    assert_eq!(sink.errors()[0].line_number, 1);
    f.cumulative(&[1001]);
    let (result, _) = f.run(&repo, f.request("omitted", 2, 1));
    assert!(matches!(
        result,
        HistoricalValidationOutcome::ValidationOnly(_)
    ));
    assert_eq!(
        summary(&result).completeness(),
        ValidationCompleteness::Partial
    );
    assert_eq!(stats(summary(&result), "1").missing_previous_records, 1);
    assert_eq!(
        repo.current_generation(&namespace(1)).unwrap(),
        Some(SnapshotRevision::new(2))
    );
    f.cumulative(&[1001, 1002, 1003]);
    assert_eq!(
        stats(summary(&f.run(&repo, f.request("complete", 2, 1)).0), "1").new_records,
        1
    );
}

#[test]
fn strict_reuse_including_identical_and_updated_udo_is_rejected_but_other_trace_and_namespace_are_isolated()
 {
    let f = Fixture::new();
    let repo = initialized_repo(&f);
    f.cumulative(&[1001]);
    f.put("4.1", b"1001AA\n");
    commit(&mut preview(f.run(&repo, f.request("jan", 1, 1)).0));
    for payload in [b"1001AA\n", b"1001BB\n"] {
        f.put("4.1", payload);
        let (result, sink) = f.run(&repo, f.request("reuse", 2, 1));
        assert_eq!(summary(&result).run_status, RunStatus::CompletedInvalid);
        assert_eq!(stats(summary(&result), "4.1").reused_records, 1);
        assert_eq!(sink.errors().len(), 1);
    }
    f.put("4.1", b"");
    f.put("4.2", b"1001AA\n");
    assert_eq!(
        stats(
            summary(&f.run(&repo, f.request("other-trace", 2, 1)).0),
            "4.2"
        )
        .new_records,
        1
    );
    initialize(&f, &repo, 2);
    f.put("4.1", b"1001AA\n");
    assert_eq!(
        stats(
            summary(&f.run(&repo, f.request("other-org", 1, 2)).0),
            "4.1"
        )
        .new_records,
        1
    );
}

#[test]
fn duplicates_use_second_pass_budget_and_preserve_cancellation() {
    let f = Fixture::new();
    let repo = initialized_repo(&f);
    f.cumulative(&[1001]);
    f.put("4.1", b"9001AA\n9001AA\n9001AA\n");
    let mut request = f.request("duplicates", 1, 1);
    request.validation.options.max_errors = Some(1);
    let (result, sink) = f.run(&repo, request);
    assert_eq!(sink.errors().len(), 1);
    assert_eq!(sink.errors()[0].line_number, 2);
    assert_eq!(summary(&result).run_status, RunStatus::CompletedInvalid);
    assert!(summary(&result).truncated);
    assert_eq!(summary(&result).records_processed, 6);
    let cancel = CancellationToken::default();
    let (result, sink) =
        f.run_controlled(&repo, f.request("cancelled", 1, 1), &cancel, &mut |event| {
            if matches!(
                event,
                ProgressEvent::PhaseStarted(ValidationPhase::CrossRecordValidation)
            ) {
                cancel.cancel();
            }
        });
    assert_eq!(summary(&result).run_status, RunStatus::Cancelled);
    assert!(sink.errors().is_empty());
    assert_eq!(
        repo.current_generation(&namespace(1)).unwrap(),
        Some(SnapshotRevision::new(1))
    );
}

#[test]
fn missing_baseline_and_backward_period_are_unknown_not_new() {
    let f = Fixture::new();
    let repo = raw_repo();
    f.cumulative(&[1001]);
    let (result, sink) = f.run(&repo, f.request("missing", 2, 1));
    assert!(sink.errors().is_empty());
    assert_eq!(
        summary(&result).completeness(),
        ValidationCompleteness::Partial
    );
    assert_eq!(stats(summary(&result), "1").new_records, 0);
    initialize(&f, &repo, 1);
    commit(&mut preview(f.run(&repo, f.request("jan", 1, 1)).0));
    commit(&mut preview(f.run(&repo, f.request("mar", 3, 1)).0));
    let (result, _) = f.run(&repo, f.request("backward", 2, 1));
    assert!(stats(summary(&result), "1").indeterminate);
}

#[test]
fn report_failure_and_cas_conflict_leave_the_approved_baseline_unchanged() {
    let f = Fixture::new();
    let repo = initialized_repo(&f);
    f.cumulative(&[1001]);
    commit(&mut preview(f.run(&repo, f.request("jan", 1, 1)).0));
    f.cumulative(&[1001, 1002]);
    let mut failed = preview(f.run(&repo, f.request("failed", 2, 1)).0);
    assert!(
        failed
            .approve_commit(&CancellationToken::default(), |_, _, _| Err(
                ReportError::Write(io::Error::other("synthetic fault"))
            ))
            .is_err()
    );
    assert_eq!(
        repo.current_generation(&namespace(1)).unwrap(),
        Some(SnapshotRevision::new(2))
    );
    let mut a = preview(f.run(&repo, f.request("a", 2, 1)).0);
    let mut b = preview(f.run(&repo, f.request("b", 2, 1)).0);
    commit(&mut a);
    assert!(
        b.approve_commit(&CancellationToken::default(), |_, _, _| Ok(()))
            .is_err()
    );
    assert_eq!(
        repo.current_generation(&namespace(1)).unwrap(),
        Some(SnapshotRevision::new(3))
    );
}

#[test]
fn sqlite_reopen_uses_the_same_baseline_and_carry_does_not_multiply_acquisitions() {
    let parent =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/cross-run-sqlite-tests");
    fs::create_dir_all(&parent).unwrap();
    let dir = tempfile::tempdir_in(parent).unwrap();
    let archive = dir.path().join("private");
    let f = Fixture::new();
    f.cumulative(&[1001, 1002]);
    {
        let repo = SQLitePersistenceAdapter::create(&archive, Some("synthetic"), &Key).unwrap();
        initialize(&f, &repo, 1);
        commit(&mut preview(f.run(&repo, f.request("jan", 1, 1)).0));
    }
    let repo = SQLitePersistenceAdapter::open(&archive, Some("synthetic"), &Key).unwrap();
    f.cumulative(&[1001, 1002, 1003]);
    let (result, _) = f.run(&repo, f.request("feb", 2, 1));
    assert_eq!(stats(summary(&result), "1").new_records, 1);
    commit(&mut preview(result));
    let (result, _) = f.run(&repo, f.request("feb", 2, 1));
    assert_eq!(stats(summary(&result), "1").new_records, 0);
    commit(&mut preview(result));
    assert_eq!(
        repo.current_generation(&namespace(1)).unwrap(),
        Some(SnapshotRevision::new(3))
    );
}

fn sioc() -> Fixture {
    let f = Fixture::with_model(
        serde_json::from_str(include_str!(
            "../schemas/sioc/2026-04-20.cross-run.1/flow_schema.json"
        ))
        .unwrap(),
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/cross-trace/sioc-referential/v1/valid");
    for id in IDS {
        f.put(id, &fs::read(root.join(format!("trace-{id}.dat"))).unwrap());
    }
    f
}
fn sioc_request(f: &Fixture, id: &str, month: i64) -> HistoricalValidationRequest {
    let mut request = f.request(id, month, 1);
    request.parameters = BTreeMap::from([
        ("organisation_namespace".into(), NamespaceValue::Integer(1)),
        ("reporting_year".into(), NamespaceValue::Integer(2026)),
        ("reporting_month".into(), NamespaceValue::Integer(month)),
    ]);
    request
}

#[test]
fn actual_sioc_profile_preserves_all_four_udo_key_fields_and_rejects_new_submission_reuse() {
    let f = sioc();
    let repo = initialized_repo(&f);
    commit(&mut preview(f.run(&repo, sioc_request(&f, "jan", 1)).0));
    let (result, sink) = f.run(&repo, sioc_request(&f, "feb", 2));
    assert_eq!(stats(summary(&result), "4.1").reused_records, 2);
    for id in ["1", "2", "3"] {
        assert_eq!(stats(summary(&result), id).carry_forward_identical, 1);
    }
    assert!(
        sink.errors()
            .iter()
            .all(|e| e.trace_id.0 == "4.1" || e.trace_id.0 == "4.2")
    );
    let mut bytes = fs::read(f.temp.path().join("4.1.dat")).unwrap();
    // The same first three components with a different 4.19.1 is a new PK.
    bytes[388..394].copy_from_slice(b"000099");
    bytes[394 + 1 + 388..394 + 1 + 394].copy_from_slice(b"000098");
    f.put("4.1", &bytes);
    let (result, sink) = f.run(&repo, sioc_request(&f, "changed-key", 2));
    assert_eq!(stats(summary(&result), "4.1").new_records, 2);
    assert!(sink.errors().iter().all(|e| e.trace_id.0 != "4.1"));
}

#[test]
fn actual_sioc_udo_duplicate_is_rejected_only_after_pass_one() {
    use std::{cell::Cell, rc::Rc};
    struct PhaseSink {
        phase: Rc<Cell<bool>>,
        errors: Vec<ValidationError>,
    }
    impl ValidationSink for PhaseSink {
        fn on_error(&mut self, error: &ValidationError) -> Result<(), ReportError> {
            assert!(
                self.phase.get(),
                "UDO duplicate must be emitted in pass two"
            );
            self.errors.push(error.clone());
            Ok(())
        }
        fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
            Ok(())
        }
    }
    let f = sioc();
    let repo = initialized_repo(&f);
    let mut bytes = fs::read(f.temp.path().join("4.1.dat")).unwrap();
    bytes.extend_from_within(..395);
    f.put("4.1", &bytes);
    let phase = Rc::new(Cell::new(false));
    let shared = phase.clone();
    let mut sink = PhaseSink {
        phase,
        errors: Vec::new(),
    };
    let result = HistoricalValidationService::new(&f.service, &repo, &protector())
        .validate(
            sioc_request(&f, "duplicate", 1),
            &mut sink,
            &mut Audit,
            &mut |event| {
                if event == ProgressEvent::PhaseStarted(ValidationPhase::CrossRecordValidation) {
                    shared.set(true);
                }
            },
            &CancellationToken::default(),
        )
        .unwrap();
    assert_eq!(summary(&result).invalid_records, 1);
    assert_eq!(sink.errors.len(), 1);
    assert_eq!(sink.errors[0].trace_id.0, "4.1");
    assert_eq!(sink.errors[0].line_number, 3);
    assert!(matches!(
        result,
        HistoricalValidationOutcome::ValidationOnly(_)
    ));
}

#[test]
fn content_identity_ignores_only_line_terminators_and_schema_capability_is_explicit() {
    let f = Fixture::new();
    let repo = initialized_repo(&f);
    f.cumulative(&[1001]);
    commit(&mut preview(f.run(&repo, f.request("jan", 1, 1)).0));
    for id in ["1", "2", "3"] {
        f.put(id, b"1001AA\r\n");
    }
    let (result, sink) = f.run(&repo, f.request("crlf", 2, 1));
    assert!(sink.errors().is_empty());
    assert_eq!(stats(summary(&result), "1").carry_forward_identical, 1);
    let mut raw: RawFlowSchema = serde_json::from_value(model()).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    assert!(SchemaCompiler.compile(raw.clone()).is_err());
    assert!(
        SchemaCompiler
            .compile_historical_audit(raw.clone(), &[])
            .is_err()
    );
    assert!(
        SchemaCompiler
            .compile_historical_cross_run(raw.clone(), &[])
            .is_ok()
    );
    raw.schema_format_version = 2;
    assert!(
        SchemaCompiler
            .compile_historical_cross_run(raw, &[])
            .is_err()
    );
}

#[test]
fn unapproved_preview_is_never_a_baseline_and_legacy_snapshots_remain_unchanged() {
    let f = Fixture::new();
    let repo = initialized_repo(&f);
    f.cumulative(&[1001]);
    let pending = preview(f.run(&repo, f.request("unapproved", 1, 1)).0);
    let (result, _) = f.run(&repo, f.request("feb", 2, 1));
    assert_eq!(stats(summary(&result), "1").new_records, 1);
    drop(pending);
    let repo = raw_repo();
    let mut legacy = model();
    legacy["schema_format_version"] = json!(1);
    legacy["schema_version"] = json!("legacy.1");
    legacy["persistence_policy"]
        .as_object_mut()
        .unwrap()
        .remove("cross_run_validation");
    legacy["required_capabilities"] = json!(["local.v1", "persistence.audit.v1"]);
    let old = Fixture::with_model(legacy);
    old.cumulative(&[1001]);
    commit(&mut preview(old.run(&repo, old.request("old", 1, 1)).0));
    let before = repo
        .load_snapshot(&namespace(1), SnapshotRevision::new(1))
        .unwrap()
        .unwrap();
    let (result, sink) = f.run(&repo, f.request("new", 2, 1));
    assert!(sink.errors().is_empty());
    assert!(stats(summary(&result), "1").indeterminate);
    assert_eq!(stats(summary(&result), "1").new_records, 0);
    assert_eq!(
        repo.load_snapshot(&namespace(1), SnapshotRevision::new(1))
            .unwrap()
            .unwrap(),
        before
    );
}

#[test]
fn year_boundary_changes_required_coverage_without_resetting_identity_history() {
    let f = Fixture::new();
    let repo = initialized_repo(&f);
    f.cumulative(&[1001]);
    commit(&mut preview(f.run(&repo, f.request("year2026", 1, 1)).0));
    let mut next = f.request("year2027", 1, 1);
    next.parameters
        .insert("year".into(), NamespaceValue::Integer(2027));
    let (result, _) = f.run(&repo, next);
    assert_eq!(stats(summary(&result), "1").carry_forward_identical, 1);
    assert_eq!(stats(summary(&result), "1").new_records, 0);
    commit(&mut preview(result));
    f.cumulative(&[1002]);
    let mut next = f.request("year2027-feb", 2, 1);
    next.parameters
        .insert("year".into(), NamespaceValue::Integer(2027));
    let (result, _) = f.run(&repo, next);
    assert_eq!(stats(summary(&result), "1").missing_previous_records, 1);
    assert_eq!(
        summary(&result).completeness(),
        ValidationCompleteness::Partial
    );
}

#[test]
fn canonical_normative_jsonl_matches_the_independent_golden_on_every_os() {
    let f = Fixture::new();
    let repo = initialized_repo(&f);
    f.cumulative(&[1001]);
    f.put("4.1", b"9001AA\n");
    commit(&mut preview(f.run(&repo, f.request("jan", 1, 1)).0));
    f.put("1", b"1001BB\n");
    let (_, sink) = f.run(&repo, f.request("feb", 2, 1));
    let mut jsonl = JsonlSink::new(Vec::new());
    for error in sink.errors() {
        jsonl.on_error(error).unwrap();
    }
    assert_eq!(
        jsonl.finish().unwrap(),
        include_bytes!("golden/sioc-cross-run/v1/expected.errors.jsonl")
    );
}

#[test]
fn cli_uses_the_real_normative_service_for_carry_forward_and_rejection() {
    use std::process::Command;
    let parent =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/cross-run-cli-tests");
    fs::create_dir_all(&parent).unwrap();
    let temp = tempfile::tempdir_in(parent).unwrap();
    let bundle = temp.path().join("private bundle");
    // The qualified backend provisions an owned private parent on each OS.
    drop(SQLitePersistenceAdapter::create(&bundle, Some("synthetic"), &Key).unwrap());
    let key = bundle.join("hmac.key");
    fs::write(&key, [0xab; 32]).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).unwrap();
    }
    let schemas = bundle.join("schemas");
    let schema = schemas.join("SYNTHETIC/monthly.1");
    fs::create_dir_all(&schema).unwrap();
    fs::write(
        schema.join("flow.json"),
        serde_json::to_vec(&model()).unwrap(),
    )
    .unwrap();
    let references = bundle.join("reference-data");
    fs::create_dir(&references).unwrap();
    let archive = temp.path().join("archive");
    let f = Fixture::new();
    f.cumulative(&[1001, 1002]);
    {
        let repo = SQLitePersistenceAdapter::create(&archive, Some("synthetic"), &Key).unwrap();
        initialize(&f, &repo, 1);
        commit(&mut preview(f.run(&repo, f.request("jan", 1, 1)).0));
    }
    f.cumulative(&[1001, 1002, 1003]);
    for (id, expected_code) in [("feb", 0), ("modified", 2)] {
        if id == "modified" {
            f.put("1", b"1001BB\n1002AA\n1003AA\n");
        }
        let output = temp.path().join(id);
        fs::create_dir(&output).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_lintra"));
        command.args([
            "history",
            "preview",
            "--flow",
            "SYNTHETIC",
            "--schema-version",
            "monthly.1",
            "--encoding",
            "utf8",
            "--parameter",
            "org=1",
            "--parameter",
            "year=2026",
            "--parameter",
            "month=2",
            "--key-id",
            "synthetic",
            "--submission-id",
            id,
        ]);
        for (flag, path) in [
            ("--schema-dir", schemas.clone()),
            ("--reference-data-dir", references.clone()),
            ("--key-file", key.clone()),
            ("--archive", archive.clone()),
            ("--errors", output.join("errors.jsonl")),
            ("--summary", output.join("summary.json")),
            ("--audit", output.join("audit.jsonl")),
            ("--audit-summary", output.join("audit-summary.json")),
        ] {
            command.arg(flag).arg(path);
        }
        for trace in IDS {
            let mut value = std::ffi::OsString::from(format!("{trace}="));
            value.push(f.temp.path().join(format!("{trace}.dat")));
            command.arg("--trace-input").arg(value);
        }
        let result = command.output().unwrap();
        assert_eq!(
            result.status.code(),
            Some(expected_code),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let summary: Value =
            serde_json::from_slice(&fs::read(output.join("summary.json")).unwrap()).unwrap();
        let traces = summary["full_validation"]["cross_run_validation"]["traces"]
            .as_array()
            .unwrap();
        let first = traces.iter().find(|t| t["trace_id"] == "1").unwrap();
        if id == "feb" {
            assert_eq!(first["carry_forward_identical"], 2);
            assert_eq!(first["new_records"], 1);
        } else {
            assert_eq!(first["historical_key_content_mismatch"], 1);
        }
    }
    let repo = SQLitePersistenceAdapter::open(&archive, Some("synthetic"), &Key).unwrap();
    assert_eq!(
        repo.current_generation(&namespace(1)).unwrap(),
        Some(SnapshotRevision::new(2))
    );
}

/// Read faults cannot be converted into an empty, apparently unique baseline.
struct FaultRepository<'a> {
    inner: &'a InMemoryPersistenceRepository,
    mode: u8,
}
impl PersistentContextRepository for FaultRepository<'_> {
    fn load_protection_audit(&self) -> Result<ProtectionAudit, PersistenceError> {
        self.inner.load_protection_audit()
    }
    fn current_generation(
        &self,
        n: &PersistenceNamespace,
    ) -> Result<Option<SnapshotRevision>, PersistenceError> {
        self.inner.current_generation(n)
    }
    fn load_submission_page(
        &self,
        n: &PersistenceNamespace,
        a: Option<SnapshotRevision>,
    ) -> Result<Vec<(SnapshotRevision, SourceSubmission)>, PersistenceError> {
        if self.mode == 0 {
            Ok(Vec::new())
        } else {
            self.inner.load_submission_page(n, a)
        }
    }
    fn load_snapshot(
        &self,
        n: &PersistenceNamespace,
        r: SnapshotRevision,
    ) -> Result<Option<HistoricalSnapshot>, PersistenceError> {
        if self.mode == 1 {
            Ok(None)
        } else {
            self.inner.load_snapshot(n, r)
        }
    }
    fn lookup_identity_occurrence(
        &self,
        n: &PersistenceNamespace,
        i: &PersistentRecordIdentity,
    ) -> Result<Option<HistoricalIdentityOccurrence>, PersistenceError> {
        if self.mode == 2 {
            Err(PersistenceError::StorageUnavailable)
        } else {
            self.inner.lookup_identity_occurrence(n, i)
        }
    }
    fn lookup_identity(
        &self,
        n: &PersistenceNamespace,
        i: &PersistentRecordIdentity,
    ) -> Result<Option<StoredIdentity>, PersistenceError> {
        self.inner.lookup_identity(n, i)
    }
    fn load_current_snapshot(
        &self,
        n: &PersistenceNamespace,
    ) -> Result<Option<HistoricalSnapshot>, PersistenceError> {
        self.inner.load_current_snapshot(n)
    }
    fn load_submission(
        &self,
        n: &PersistenceNamespace,
        id: &str,
    ) -> Result<Option<SourceSubmission>, PersistenceError> {
        self.inner.load_submission(n, id)
    }
    fn begin_transaction(
        &self,
        n: &PersistenceNamespace,
        g: Option<SnapshotRevision>,
    ) -> Result<Box<dyn PersistenceTransaction>, PersistenceError> {
        self.inner.begin_transaction(n, g)
    }
}

#[test]
fn incomplete_metadata_missing_snapshot_and_failed_identity_lookup_are_technical_failures() {
    let f = Fixture::new();
    let repo = initialized_repo(&f);
    f.cumulative(&[1001]);
    commit(&mut preview(f.run(&repo, f.request("jan", 1, 1)).0));
    f.cumulative(&[1001, 1002]);
    for mode in 0..3 {
        let faulty = FaultRepository { inner: &repo, mode };
        let mut sink = VecSink::default();
        let result = HistoricalValidationService::new(&f.service, &faulty, &protector()).validate(
            f.request("fault", 2, 1),
            &mut sink,
            &mut Audit,
            &mut |_| {},
            &CancellationToken::default(),
        );
        assert!(result.is_err());
        assert!(sink.summaries().is_empty());
        assert_eq!(
            repo.current_generation(&namespace(1)).unwrap(),
            Some(SnapshotRevision::new(2))
        );
    }
}

#[test]
fn invalid_policy_bindings_fail_closed_and_sioc_source_keys_are_preserved() {
    for mode in 0..6 {
        let mut value = model();
        match mode {
            0 => value["persistence_policy"]["cross_run_validation"]["traces"] = json!([]),
            1 => {
                value["persistence_policy"]["cross_run_validation"]["year_parameter"] =
                    json!("missing")
            }
            2 => value["persistence_policy"]["namespace_parameters"] = json!(["org", "year"]),
            3 => value["required_capabilities"] = json!(["local.v1", "persistence.audit.v1"]),
            4 => {
                value["persistence_policy"]["cross_run_validation"]["traces"][0]["trace"] =
                    json!("missing")
            }
            _ => {
                value["persistence_policy"]["cross_run_validation"]["traces"][0]["reuse_error"] =
                    json!("not-an-error-code")
            }
        }
        let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
        raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
        assert!(
            SchemaCompiler
                .compile_historical_cross_run(raw, &[])
                .is_err()
        );
    }
    let source: Value = serde_json::from_str(include_str!(
        "../schemas/sioc/2026-04-20.structural-minimal.2/flow_schema.json"
    ))
    .unwrap();
    let target: Value = serde_json::from_str(include_str!(
        "../schemas/sioc/2026-04-20.cross-run.1/flow_schema.json"
    ))
    .unwrap();
    for (old, new) in source["traces"]
        .as_array()
        .unwrap()
        .iter()
        .zip(target["traces"].as_array().unwrap())
    {
        assert_eq!(old["id"], new["id"]);
        assert_eq!(old["record_key"], new["record_key"]);
    }
}

#[test]
fn each_udo_key_component_discriminates_records_and_cumulative_intra_file_duplicates_still_fail() {
    for (offset, replacement) in [
        (0, b"000099".as_slice()),
        (144, b"999".as_slice()),
        (147, b"998".as_slice()),
        (388, b"000097".as_slice()),
    ] {
        let f = sioc();
        let repo = initialized_repo(&f);
        let mut bytes = fs::read(f.temp.path().join("4.1.dat")).unwrap();
        let mut row = bytes[..395].to_vec();
        row[offset..offset + replacement.len()].copy_from_slice(replacement);
        bytes.extend(row);
        f.put("4.1", &bytes);
        let (_, sink) = f.run(&repo, sioc_request(&f, "distinct-key", 1));
        assert!(
            !sink.errors().iter().any(|e| e.trace_id.0 == "4.1"
                && e.code.as_symbol().unwrap() == "E_DUPLICATE_RECORD_KEY")
        );
    }
    let f = Fixture::new();
    let repo = initialized_repo(&f);
    f.cumulative(&[1001]);
    commit(&mut preview(f.run(&repo, f.request("jan", 1, 1)).0));
    f.put("1", b"1001AA\n1001AA\n");
    let (result, sink) = f.run(&repo, f.request("duplicate-carry", 2, 1));
    assert_eq!(summary(&result).run_status, RunStatus::CompletedInvalid);
    assert_eq!(sink.errors().len(), 1);
    assert_eq!(
        sink.errors()[0].code.as_symbol().unwrap(),
        "E_DUPLICATE_RECORD_KEY"
    );
}

#[test]
fn public_draft_api_cannot_approve_a_normative_profile_without_its_real_producer() {
    let f = Fixture::new();
    let repo = initialized_repo(&f);
    f.cumulative(&[1001]);
    let p = preview(f.run(&repo, f.request("jan", 1, 1)).0);
    let mut parts: SourceSubmissionParts =
        serde_json::from_value(serde_json::to_value(p.snapshot().source()).unwrap()).unwrap();
    parts.state = SessionState::Draft;
    parts.period = None;
    let mut raw: RawFlowSchema = serde_json::from_value(model()).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let profile = SchemaCompiler
        .compile_persistence_profile(&raw, &[])
        .unwrap();
    let draft =
        ApprovalDraft::new(&profile, f.request("jan", 1, 1).parameters, parts, None).unwrap();
    assert!(matches!(
        draft.validated(
            p.outcome(),
            p.snapshot().identities().to_vec(),
            &protector()
        ),
        Err(SessionApprovalError::UnsupportedSessionPolicy)
    ));
    assert_eq!(
        repo.current_generation(&namespace(1)).unwrap(),
        Some(SnapshotRevision::new(1))
    );
}

#[test]
fn an_empty_database_does_not_prove_new_keys_even_in_january() {
    let f = Fixture::new();
    let repo = raw_repo();
    f.cumulative(&[1001]);
    f.put("4.1", b"9001AA\n");
    let (result, _) = f.run(&repo, f.request("unqualified-january", 1, 1));
    assert!(matches!(
        result,
        HistoricalValidationOutcome::ValidationOnly(_)
    ));
    for id in ["1", "2", "3", "4.1"] {
        let row = stats(summary(&result), id);
        assert_eq!(row.new_records, 0);
        assert_eq!(
            row.unknown_reason.as_deref(),
            Some("missing_initial_approved_baseline")
        );
    }
    assert_eq!(repo.current_generation(&namespace(1)).unwrap(), None);
    initialize(&f, &repo, 1);
    let (result, _) = f.run(&repo, f.request("qualified-january", 1, 1));
    assert_eq!(stats(summary(&result), "1").new_records, 1);
    assert_eq!(stats(summary(&result), "4.1").new_records, 1);
    assert!(matches!(result, HistoricalValidationOutcome::Preview(_)));
}

#[test]
fn cancellation_during_pass_one_keeps_deferred_udo_errors_unreported_and_terminal() {
    let f = Fixture::new();
    let repo = initialized_repo(&f);
    f.cumulative(&[1001]);
    f.put("4.1", b"9001AA\n9001AA\n");
    let cancel = CancellationToken::default();
    let (result, sink) = f.run_controlled(
        &repo,
        f.request("cancel-first-pass", 1, 1),
        &cancel,
        &mut |event| {
            if matches!(event, ProgressEvent::TraceCompleted { trace_id, .. } if trace_id.0 == "4.1") {
                cancel.cancel();
            }
        },
    );
    assert_eq!(summary(&result).run_status, RunStatus::Cancelled);
    assert!(sink.errors().is_empty());
    assert_eq!(
        summary(&result)
            .full_validation
            .as_ref()
            .unwrap()
            .cross_run_validation
            .as_ref()
            .unwrap()
            .unreported_errors,
        1
    );
    assert_eq!(
        repo.current_generation(&namespace(1)).unwrap(),
        Some(SnapshotRevision::new(1))
    );
}

#[test]
fn real_sioc_content_mismatch_is_a_normative_rejection_with_honest_skipped_join_counts() {
    let f = sioc();
    let repo = initialized_repo(&f);
    commit(&mut preview(f.run(&repo, sioc_request(&f, "jan", 1)).0));
    let mut row = fs::read(f.temp.path().join("1.dat")).unwrap();
    // An opaque span is part of the explicitly qualified content identity.
    // Its change must not be mistaken for an unchanged cumulative record.
    row[20] = if row[20] == b'X' { b'Y' } else { b'X' };
    f.put("1", &row);
    let (result, sink) = f.run(&repo, sioc_request(&f, "changed", 2));
    assert_eq!(summary(&result).run_status, RunStatus::CompletedInvalid);
    assert!(sink.errors().iter().any(|e| e.trace_id.0 == "1"
        && e.code.as_symbol().unwrap() == "E_HISTORICAL_KEY_CONTENT_MISMATCH"));
    assert_eq!(
        stats(summary(&result), "1").historical_key_content_mismatch,
        1
    );
    assert!(summary(&result).skipped_rules > 0);
    assert!(matches!(
        result,
        HistoricalValidationOutcome::ValidationOnly(_)
    ));
}
