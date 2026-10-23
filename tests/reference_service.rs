//! Real service/provider/report path, synthetic versioned data only.
use lintra::*;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    input: PathBuf,
}
impl Fixture {
    fn new(bytes: &[u8]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "lintra reference service é {} {} {}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("data")).unwrap();
        let input = root.join("input.txt");
        fs::write(&input, bytes).unwrap();
        Self { root, input }
    }
    fn schema(&self, available: [bool; 3], degraded_a: bool, rule: bool) -> RawFlowSchema {
        let cases: Vec<Value> = serde_json::from_slice(include_bytes!(
            "fixtures/reference_membership/union_cases.json"
        ))
        .unwrap();
        let mut deps = vec![];
        for (index, id) in ["A", "B", "C"].into_iter().enumerate() {
            let case = &cases[index];
            if available[index] {
                fs::write(
                    self.root.join(format!("data/{id}.json")),
                    decode(case["bytes_hex"].as_str().unwrap()),
                )
                .unwrap();
            }
            deps.push(json!({"id":id,"dataset_id":format!("union-{id}"),"version":"1","path":format!("{id}.json"),"sha256":if index==0 && degraded_a{"0".repeat(64)}else{case["sha256"].as_str().unwrap().to_owned()},"metadata_attributes":[]}));
        }
        let reference = json!({"kind":"reference_data","datasets":["A","B","C"],"fallback_values":["ZZ"],"metadata_bindings":[]});
        let fields=["CODE","OTHER"].into_iter().enumerate().map(|(i,id)|json!({"id":id,"name":id,"physical":{"type":"alphanumeric","start":i*2+1,"length":2,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":if i==0 {if rule{vec![]}else{vec![reference.clone()]}}else{vec![json!({"kind":"allowed_values","target":"raw","values":["OK"]})]}})).collect::<Vec<_>>();
        serde_json::from_value(json!({"schema_format_version":1,"flow_id":"SYNTHETIC","schema_version":"v1","regulatory_version":"test","source":{},"encoding":{"mode":"runtime_required","position_unit":"byte"},"required_capabilities":["local.v1","expressions.v1"],"parameters":[],"reference_data":deps,"persistence_policy":null,"traces":[{"id":"T","name":"synthetic","record_length":4,"fields":fields,"record_key":{"fields":["CODE"],"uniqueness_scope":"trace","error_code":"E_DUPLICATE_RECORD_KEY"},"joins":[],"identity":[],"named_conditions":{},"derived_values":{},"repeated_groups":[],"rules":if rule{vec![json!({"id":"R-MEMBER","when":{"const":true},"assertions":[{"kind":"constraint","field":{"field":"CODE"},"constraint":reference}],"error_code":"E_REFERENCE_DATA","message_template":"reference membership"})]}else{vec![]},"documented_non_executable_rules":[]}]})).unwrap()
    }
    fn service(&self, raw: RawFlowSchema, policy: DatasetPreparationPolicy) -> ValidationService {
        service(raw).with_reference_data(
            Arc::new(FileSystemReferenceDataPreparer::new(self.root.join("data")).unwrap()),
            policy,
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn decode(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn service(raw: RawFlowSchema) -> ValidationService {
    let mut repo = InMemorySchemaRepository::new();
    repo.insert(raw).unwrap();
    ValidationService::new(Arc::new(SchemaRegistry::new(
        Arc::new(repo),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    )))
}
fn request(path: &Path) -> ValidationRequest {
    ValidationRequest {
        schema_key: SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        scope: ValidationScope::Trace(TraceValidationInput {
            trace_id: TraceId("T".into()),
            source: InputSource::Path(path.into()),
        }),
        mode: ValidationMode::LocalOnly,
        options: ValidationOptions {
            record_length_policy: RecordLengthPolicy::StrictSkipRecord,
            newline_policy: NewlinePolicy::AutoLfOrCrlf,
            max_errors: None,
        },
    }
}
fn run(service: &ValidationService, input: &Path) -> (ValidationRunOutcome, VecSink) {
    let mut sink = VecSink::default();
    let outcome = service
        .validate(
            request(input),
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    (outcome, sink)
}
const CONTINUE: DatasetPreparationPolicy = DatasetPreparationPolicy::ContinueWithAvailableEvidence;
#[test]
fn verified_union_fallback_real_violations_and_actual_audit_are_preserved() {
    let f = Fixture::new(b"AAOK\nQQOK\nZZOK\nAAXX\n");
    let raw = f.schema([true; 3], false, false);
    let (out, sink) = run(&f.service(raw, CONTINUE), &f.input);
    assert_eq!(out.status, RunStatus::CompletedInvalid);
    assert_eq!(out.summary.total_errors, 2);
    assert_eq!(sink.errors().len(), 2);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Complete);
    let audit = out.summary.reference_validation.as_ref().unwrap();
    let c = &audit.controls[0];
    assert_eq!(
        c.evaluations,
        ReferenceEvaluationCounts {
            definitive_satisfied: 3,
            definitive_violated: 1,
            ..Default::default()
        }
    );
    assert_eq!(
        c.sources
            .iter()
            .map(|s| (s.consulted, s.selected_matches))
            .collect::<Vec<_>>(),
        vec![(3, 2), (1, 0), (1, 0)]
    );
    assert_eq!(out.summary.reference_data_versions.len(), 3);
    assert!(c.warnings().next().is_none());
    let document: Value =
        serde_json::from_slice(&write_run_summary(Vec::new(), &out.summary).unwrap()).unwrap();
    assert_eq!(document["reference_validation"]["completeness"], "complete");
    for dataset in &audit.datasets {
        assert_eq!(
            dataset.actual_sha256.as_ref(),
            Some(&dataset.expected_sha256)
        );
        assert_eq!(dataset.declared.as_ref(), Some(&dataset.requested));
    }
}
#[test]
fn degraded_matches_are_provisional_negatives_inconclusive_and_independent_errors_remain() {
    let f = Fixture::new(b"AAOK\nBBOK\nQQXX\n");
    let raw = f.schema([true; 3], true, false);
    let (out, sink) = run(&f.service(raw, CONTINUE), &f.input);
    assert_eq!(out.summary.total_errors, 1);
    assert_eq!(sink.errors().len(), 1);
    assert_ne!(sink.errors()[0].code, ValidationCode::ReferenceData);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
    let audit = out.summary.reference_validation.as_ref().unwrap();
    let c = &audit.controls[0];
    assert_eq!(
        c.evaluations,
        ReferenceEvaluationCounts {
            definitive_satisfied: 1,
            provisional_present: 1,
            inconclusive: 1,
            ..Default::default()
        }
    );
    assert_eq!(
        c.warnings().collect::<Vec<_>>(),
        vec![
            (ReferenceWarningKind::ProvisionalEvidence, 1),
            (ReferenceWarningKind::SearchNotExhaustive, 1)
        ]
    );
    assert!(
        !out.summary
            .reference_data_versions
            .contains_key(&DatasetId("union-A".into()))
    );
    let a = &audit.datasets[0];
    assert_eq!(a.availability, DatasetAvailability::Degraded);
    assert_eq!(a.expected_sha256, Sha256Digest([0; 32]));
    assert_ne!(a.actual_sha256.as_ref(), Some(&a.expected_sha256));
    let json = write_run_summary(Vec::new(), &out.summary).unwrap();
    let doc: Value = serde_json::from_slice(&json).unwrap();
    assert_eq!(
        doc["reference_validation"]["datasets"][0]["availability"],
        "degraded"
    );
    assert_ne!(
        doc["reference_validation"]["datasets"][0]["actual_sha256"],
        doc["reference_validation"]["datasets"][0]["expected_sha256"]
    );
    let mut human = HumanReadableSink::new(Vec::new());
    human.on_summary(&out.summary).unwrap();
    let text = String::from_utf8(human.finish().unwrap()).unwrap();
    assert!(text.contains("Completeness: Partial"));
    assert!(text.contains("does not establish full conformance"));
}
#[test]
fn all_missing_continues_without_normative_errors_or_false_full_conformance() {
    let f = Fixture::new(b"QQOK\n");
    let raw = f.schema([false; 3], false, false);
    let (out, sink) = run(&f.service(raw, CONTINUE), &f.input);
    assert_eq!(out.status, RunStatus::CompletedValid);
    assert!(sink.errors().is_empty());
    assert_eq!(out.summary.total_errors, 0);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
    let a = out.summary.reference_validation.as_ref().unwrap();
    assert_eq!(a.controls[0].evaluations.inconclusive, 1);
    assert!(out.summary.reference_data_versions.is_empty());
    for dataset in &a.datasets {
        assert_eq!(dataset.availability, DatasetAvailability::Unavailable);
        assert!(dataset.actual_sha256.is_none());
        assert!(matches!(
            dataset.diagnostics.as_slice(),
            [ReferenceDatasetDiagnostic::Io {
                error: ReferenceIoKind::NotFound,
                ..
            }]
        ));
    }
}
#[test]
fn verified_positive_proves_union_even_when_other_datasets_are_missing() {
    let f = Fixture::new(b"AAOK\n");
    let raw = f.schema([true, false, false], false, false);
    let (out, _) = run(&f.service(raw, CONTINUE), &f.input);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Complete);
    let a = out.summary.reference_validation.unwrap();
    assert_eq!(a.controls[0].evaluations.definitive_satisfied, 1);
    assert_eq!(a.datasets[1].availability, DatasetAvailability::Unavailable);
    assert!(a.controls[0].warnings().next().is_none());
}
#[test]
fn strict_and_unconfigured_fail_before_input_open_or_progress_events() {
    for degraded in [false, true] {
        let f = Fixture::new(b"AAOK\n");
        let raw = f.schema([degraded; 3], degraded, false);
        let mut events = vec![];
        let error = f
            .service(raw.clone(), DatasetPreparationPolicy::Strict)
            .validate(
                request(&f.root.join("absent-input")),
                &mut VecSink::default(),
                &mut |e| events.push(e),
                &CancellationToken::default(),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            ValidationRunError::ReferenceData(ReferencePreparationError::Provider(_))
        ));
        assert!(events.is_empty());
        let error = service(raw)
            .validate(
                request(&f.root.join("absent-input")),
                &mut VecSink::default(),
                &mut |_| {},
                &CancellationToken::default(),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            ValidationRunError::ReferenceData(ReferencePreparationError::NotConfigured)
        ));
    }
}
#[test]
fn cancellation_remains_distinct_from_completed_partial_validation() {
    for limit in [0, 1024] {
        let f = Fixture::new(&b"QQOK\n".repeat(5000));
        let raw = f.schema([false; 3], false, false);
        let token = CancellationToken::default();
        if limit == 0 {
            token.cancel();
        }
        let out = f
            .service(raw, CONTINUE)
            .validate(
                request(&f.input),
                &mut VecSink::default(),
                &mut |event| {
                    if matches!(event,ProgressEvent::RecordsProcessed{count,..} if count==limit) {
                        token.cancel();
                    }
                },
                &token,
            )
            .unwrap();
        assert_eq!(out.status, RunStatus::Cancelled);
        assert_eq!(out.summary.records_processed, limit);
        assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
        assert_eq!(
            out.summary.inputs[0].completion,
            InputAuditCompletion::Incomplete
        );
        assert_eq!(
            out.summary.reference_validation.as_ref().unwrap().controls[0]
                .evaluations
                .inconclusive,
            limit
        );
    }
}
#[test]
fn many_warnings_are_aggregated_without_jsonl_errors_or_report_growth_per_record() {
    let mut lengths = vec![];
    for count in [1, 10000] {
        let f = Fixture::new(&b"QQOK\n".repeat(count));
        let raw = f.schema([false; 3], false, false);
        let mut sink = JsonlSink::new(Vec::new());
        let out = f
            .service(raw, CONTINUE)
            .validate(
                request(&f.input),
                &mut sink,
                &mut |_| {},
                &CancellationToken::default(),
            )
            .unwrap();
        assert!(sink.finish().unwrap().is_empty());
        let audit = out.summary.reference_validation.as_ref().unwrap();
        assert_eq!(audit.controls.len(), 1);
        assert_eq!(audit.controls[0].evaluations.inconclusive, count as u64);
        assert_eq!(audit.controls[0].warnings().count(), 1);
        lengths.push(serde_json::to_vec(audit).unwrap().len());
    }
    assert!(lengths[1] <= lengths[0] + 8);
}
#[test]
fn rule_reference_predicates_use_the_same_audit_and_unknown_propagation() {
    let f = Fixture::new(b"AAOK\nQQOK\n");
    let raw = f.schema([true; 3], true, true);
    let (out, sink) = run(&f.service(raw, CONTINUE), &f.input);
    assert!(sink.errors().is_empty());
    let audit = out.summary.reference_validation.unwrap();
    assert_eq!(audit.controls[0].rule_id.as_deref(), Some("R-MEMBER"));
    assert_eq!(audit.controls[0].evaluations.provisional_present, 1);
    assert_eq!(audit.controls[0].evaluations.inconclusive, 1);
    assert_eq!(audit.completeness, ValidationCompleteness::Partial);
}

#[test]
fn summary_validation_rejects_forged_trust_completeness_counts_and_legacy_versions() {
    let f = Fixture::new(b"AAOK\nQQOK\n");
    let raw = f.schema([true; 3], true, false);
    let (out, _) = run(&f.service(raw, CONTINUE), &f.input);
    let original = out.summary;
    for mutate in [
        |s: &mut ValidationSummary| {
            s.reference_validation.as_mut().unwrap().completeness =
                ValidationCompleteness::Complete;
        },
        |s: &mut ValidationSummary| {
            s.reference_validation.as_mut().unwrap().datasets[0].availability =
                DatasetAvailability::Verified;
        },
        |s: &mut ValidationSummary| {
            s.reference_data_versions
                .insert(DatasetId("union-A".into()), DatasetVersion("1".into()));
        },
        |s: &mut ValidationSummary| {
            s.reference_validation.as_mut().unwrap().controls[0].sources[0].consulted = u64::MAX;
        },
        |s: &mut ValidationSummary| {
            s.reference_validation.as_mut().unwrap().controls[0]
                .evaluations
                .definitive_violated = 1;
        },
        |s: &mut ValidationSummary| {
            s.reference_validation.as_mut().unwrap().controls[0]
                .evaluations
                .definitive_satisfied = u64::MAX;
        },
    ] {
        let mut changed = original.clone();
        mutate(&mut changed);
        assert_eq!(
            changed.validate(),
            Err(SummaryBuildError::InconsistentReferenceAudit)
        );
        assert!(matches!(
            write_run_summary(Vec::new(), &changed),
            Err(ReportError::InvalidSummary(_))
        ));
    }
}
#[test]
fn malformed_and_wrong_identity_are_unavailable_but_independent_validation_continues() {
    for bytes in [
        b"{broken".as_slice(),
        br#"{"dataset_format_version":1,"dataset_id":"wrong","version":"1","keys_hex":["4141"]}"#,
    ] {
        let f = Fixture::new(b"AAOK\nQQXX\n");
        let raw = f.schema([true, false, false], false, false);
        fs::write(f.root.join("data/A.json"), bytes).unwrap();
        let (out, sink) = run(&f.service(raw, CONTINUE), &f.input);
        assert_eq!(out.summary.total_errors, 1);
        assert_eq!(sink.errors().len(), 1);
        let audit = out.summary.reference_validation.unwrap();
        assert_eq!(
            audit.datasets[0].availability,
            DatasetAvailability::Unavailable
        );
        assert!(audit.datasets[0].actual_sha256.is_some());
        assert_eq!(audit.controls[0].evaluations.inconclusive, 2);
        assert!(
            audit.datasets[0]
                .diagnostics
                .contains(&ReferenceDatasetDiagnostic::IntegrityMismatch)
        );
    }
}
#[test]
fn inconsistent_preparation_policy_is_rejected_before_input_access() {
    struct WrongPolicy {
        root: PathBuf,
    }
    impl ReferenceDataPreparer for WrongPolicy {
        fn prepare(
            &self,
            deps: &[ReferenceDependency],
            _: DatasetPreparationPolicy,
        ) -> Result<PreparedReferenceEvidence, ReferencePreparationError> {
            FileSystemReferenceDataPreparer::new(self.root.clone())
                .unwrap()
                .prepare(deps, DatasetPreparationPolicy::Strict)
        }
    }
    let f = Fixture::new(b"AAOK\n");
    let raw = f.schema([true; 3], false, false);
    let svc = service(raw).with_reference_data(
        Arc::new(WrongPolicy {
            root: f.root.join("data"),
        }),
        CONTINUE,
    );
    let error = svc
        .validate(
            request(&f.root.join("missing-input")),
            &mut VecSink::default(),
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap_err();
    assert!(matches!(
        error,
        ValidationRunError::ReferenceData(ReferencePreparationError::InconsistentEvidence)
    ));
}
#[cfg(unix)]
#[test]
fn symlink_boundary_is_fatal_in_continue_and_strict_service_modes() {
    for policy in [CONTINUE, DatasetPreparationPolicy::Strict] {
        let f = Fixture::new(b"AAOK\n");
        let raw = f.schema([true; 3], false, false);
        let path = f.root.join("data/A.json");
        let outside = f.root.join("outside.json");
        fs::rename(&path, &outside).unwrap();
        std::os::unix::fs::symlink(&outside, &path).unwrap();
        let error = f
            .service(raw, policy)
            .validate(
                request(&f.root.join("missing-input")),
                &mut VecSink::default(),
                &mut |_| {},
                &CancellationToken::default(),
            )
            .unwrap_err();
        let ValidationRunError::ReferenceData(ReferencePreparationError::Provider(source)) = error
        else {
            panic!("unexpected error")
        };
        assert!(matches!(
            source.downcast_ref::<FilesystemReferenceError>(),
            Some(FilesystemReferenceError::Boundary { .. })
        ));
    }
}
#[test]
fn cli_wires_reference_root_and_publishes_partial_summary_with_no_error_jsonl() {
    use std::process::Command;
    let f = Fixture::new(b"QQOK\n");
    let schemas = f.root.join("schemas");
    fs::create_dir_all(schemas.join("SYNTHETIC/v1")).unwrap();
    // Add a synthetic membership constraint without rewriting the golden file.
    let mut json: Value =
        serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
    json["traces"][0]["fields"][0]["constraints"] = json!([{"kind":"reference_data","datasets":["A"],"fallback_values":[],"metadata_bindings":[]}]);
    json["reference_data"] = json!([{"id":"A","dataset_id":"union-A","version":"1","path":"A.json","sha256":"16693471e9fe33bdf8dfe07f1901d41803db09adc6495d9adbe98b48825c61a7","metadata_attributes":[]}]);
    fs::write(
        schemas.join("SYNTHETIC/v1/flow.json"),
        serde_json::to_vec(&json).unwrap(),
    )
    .unwrap();
    fs::write(&f.input, b"QQQQQQ\n").unwrap();
    let errors = f.root.join("errors.jsonl");
    let summary = f.root.join("summary.json");
    let args = |policy: &str| {
        Command::new(env!("CARGO_BIN_EXE_lintra"))
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
                "--reference-policy",
                policy,
            ])
            .arg("--input")
            .arg(&f.input)
            .arg("--schema-dir")
            .arg(&schemas)
            .arg("--reference-data-dir")
            .arg(f.root.join("data"))
            .arg("--errors")
            .arg(&errors)
            .arg("--summary")
            .arg(&summary)
            .output()
            .unwrap()
    };
    let result = args("continue");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(fs::read(&errors).unwrap().is_empty());
    let doc: Value = serde_json::from_slice(&fs::read(&summary).unwrap()).unwrap();
    assert_eq!(doc["reference_validation"]["completeness"], "partial");
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .contains("Completeness: Partial")
    );
    let old = fs::read(&summary).unwrap();
    let result = args("strict");
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(fs::read(&summary).unwrap(), old);
    assert!(!f.root.join("summary.json.part").exists());
    assert!(!f.root.join("errors.jsonl.part").exists());
}

#[test]
fn legacy_version_map_omits_ambiguous_ids_but_full_audit_keeps_both_verified_versions() {
    let f = Fixture::new(b"AAOK\n");
    let mut raw = f.schema([true, true, false], false, false);
    let case: Value = serde_json::from_slice(include_bytes!(
        "fixtures/reference_membership/service_version_2.json"
    ))
    .unwrap();
    fs::write(
        f.root.join("data/B.json"),
        decode(case["bytes_hex"].as_str().unwrap()),
    )
    .unwrap();
    raw.reference_data[1].dataset_id = "union-A".into();
    raw.reference_data[1].version = "2".into();
    raw.reference_data[1].sha256 = case["sha256"].as_str().unwrap().into();
    let (out, _) = run(&f.service(raw, CONTINUE), &f.input);
    assert!(out.summary.reference_data_versions.is_empty());
    let audit = out.summary.reference_validation.unwrap();
    assert_eq!(
        audit.datasets[0].availability,
        DatasetAvailability::Verified
    );
    assert_eq!(
        audit.datasets[1].availability,
        DatasetAvailability::Verified
    );
    assert_eq!(
        audit.datasets[0].requested.id,
        audit.datasets[1].requested.id
    );
    assert_ne!(
        audit.datasets[0].requested.version,
        audit.datasets[1].requested.version
    );
    assert_eq!(audit.completeness, ValidationCompleteness::Complete);
}
#[test]
fn explicit_summary_finish_cannot_erase_a_record_observed_non_definitive_marker() {
    struct Observer {
        count: u64,
    }
    impl ReferenceEvaluationObserver for Observer {
        fn on_evaluation(
            &mut self,
            _: ReferenceControl<'_>,
            _: ReferenceConstraintEvaluation<'_>,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.count += 1;
            Ok(())
        }
    }
    let f = Fixture::new(b"QQOK\n");
    let mut raw = f.schema([false; 3], false, false);
    let (out, _) = run(&f.service(raw.clone(), CONTINUE), &f.input);
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let flow = SchemaCompiler.compile(raw).unwrap();
    let prepared = FileSystemReferenceDataPreparer::new(f.root.join("data"))
        .unwrap()
        .prepare(flow.required_reference_data(), CONTINUE)
        .unwrap();
    let mut builder = ValidationSummaryBuilder::new(
        flow.key().clone(),
        ScopeSummary::Trace {
            trace_id: TraceId("T".into()),
        },
    );
    let mut observer = Observer { count: 0 };
    ReferenceRecordValidator::new(&flow, &TraceId("T".into()), &prepared)
        .unwrap()
        .validate_local(
            RawRecord {
                bytes: b"QQOK",
                line_number: 1,
            },
            &mut VecSink::default(),
            &mut builder,
            &mut observer,
        )
        .unwrap();
    assert_eq!(observer.count, 1);
    let mut audit = out.summary.reference_validation.unwrap();
    audit.controls[0].evaluations = ReferenceEvaluationCounts::default();
    let metadata = RunMetadata {
        validation_mode: ValidationMode::LocalOnly,
        validator_version: "synthetic".into(),
        schema_hash: "synthetic".into(),
        reference_data_versions: Default::default(),
        started_at: UNIX_EPOCH,
        inputs: vec![],
    };
    assert_eq!(
        builder.finish_with_reference_data(metadata, audit),
        Err(SummaryBuildError::InconsistentReferenceAudit)
    );
}
