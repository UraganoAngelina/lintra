//! Public application path on explicitly synthetic layouts. These tests qualify
//! generic history execution; they do not label a component as the CAD83 profile.
use lintra::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
const KEY: &str = "public-history-fixture";
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
        UNIX_EPOCH
    }
}
struct Repo(Value);
impl SchemaRepository for Repo {
    fn load_raw(&self, _: &SchemaKey) -> Result<RawFlowSchema, SchemaLoadError> {
        Ok(raw(self.0.clone()))
    }
}
fn raw(v: Value) -> RawFlowSchema {
    let mut r: RawFlowSchema = serde_json::from_value(v).unwrap();
    r.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    r
}
fn key() -> SchemaKey {
    SchemaKey {
        flow_id: FlowId("HISTORY-SYNTHETIC".into()),
        version: SchemaVersion("v1".into()),
    }
}
fn model() -> Value {
    let mut v: Value =
        serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
    v["schema_format_version"] = json!(3);
    v["flow_id"] = json!("HISTORY-SYNTHETIC");
    v["required_capabilities"] = json!([
        "local.v1",
        "expressions.v1",
        "joins.v1",
        "joins.sets.v1",
        "persistence.audit.v1",
        "persistence.snapshot.v1",
        "history.existence.v1"
    ]);
    let component: Value = serde_json::from_str(include_str!(
        "../schemas/cad/components/2024-06.namespace.1/component.json"
    ))
    .unwrap();
    v["parameters"] = component["parameters"].clone();
    v["persistence_policy"] = component["persistence_policy"].clone();
    v["persistence_policy"]["namespace_bindings"] = Value::Null;
    v["persistence_policy"]
        .as_object_mut()
        .unwrap()
        .remove("namespace_bindings");
    v["persistence_policy"]["cross_run_audit"] =
        json!([{"trace":"O","key":"record_key"},{"trace":"T","key":"record_key"}]);
    v["persistence_policy"]["run_uniqueness"] = json!([]);
    let mut owner = v["traces"][0].clone();
    owner["id"] = json!("O");
    owner["record_length"] = json!(3);
    owner["fields"][0]["id"] = json!("event");
    owner["fields"][0]["physical"]["length"] = json!(3);
    owner["fields"][0]["presence"]["base"] = json!("required");
    owner["fields"][0]["constraints"] = json!([{"kind":"regex","match":"required","target":"logical","pattern":"\\A[A-Z][0-9]{2}\\z"}]);
    owner["record_key"]["fields"] = json!(["event"]);
    let mut target = owner.clone();
    target["id"] = json!("T");
    target["record_length"] = json!(7);
    let base = owner["fields"][0].clone();
    let fields = target["fields"].as_array_mut().unwrap();
    for (id, start, len, kind, allowed) in [
        ("seq", 4, 2, "numeric", vec![]),
        ("kind", 6, 1, "numeric", vec!["1", "2"]),
        ("other", 7, 1, "alphanumeric", vec!["A"]),
    ] {
        let mut f = base.clone();
        f["id"] = json!(id);
        f["physical"]["start"] = json!(start);
        f["physical"]["length"] = json!(len);
        f["physical"]["type"] = json!(kind);
        f["constraints"] = if allowed.is_empty() {
            json!([])
        } else {
            json!([{"kind":"allowed_values","target":"logical","values":allowed}])
        };
        fields.push(f);
    }
    target["record_key"]["fields"] = json!(["event", "seq"]);
    owner["joins"] = json!([{"id":"initial","target_trace":"T","source_fields":["event"],"target_fields":["event"],"cardinality":"zero_or_many","scope":"history_including_current","namespace_parameters":["sending_ulss"],
        "on_missing_record":{"policy":"violation","error_code":"E_SYNTHETIC_RELATED_MISSING","diagnostic":{"code":"E_SYNTHETIC_RELATED_MISSING","message_template":"Missing related record."}},
        "on_missing_trace":{"policy":"unknown","diagnostic":{"code":"W_SYNTHETIC_HISTORY_INPUT","message_template":"History input unavailable."}}}]);
    owner["rules"] = json!([{"id":"initial-required","when":{"const":true},"assertions":[{"kind":"set_exists","join":"initial","predicate":{"eq":{"ref":{"field":"kind"},"value":"1"}}}],"error_code":"E_SYNTHETIC_INITIAL_HISTORY","message_template":"Initial evaluation required."}]);
    v["traces"] = json!([owner, target]);
    v
}
struct Fixture {
    dir: tempfile::TempDir,
    service: ValidationService,
    registry: Arc<SchemaRegistry>,
}
impl Fixture {
    fn new() -> Self {
        Self::with_model(model())
    }
    fn with_model(v: Value) -> Self {
        let registry = Arc::new(SchemaRegistry::new(
            Arc::new(Repo(v)),
            SchemaCompiler,
            RawResolvedEncoding::Utf8,
        ));
        Self {
            dir: tempfile::tempdir().unwrap(),
            service: ValidationService::with_clock(registry.clone(), Arc::new(FixedClock)),
            registry,
        }
    }
    fn request(
        &self,
        id: &str,
        unit: &str,
        q: i64,
        owner: &str,
        target: Option<&str>,
    ) -> HistoricalSnapshotValidationRequest {
        let mut traces = vec![];
        for (trace, data) in [("O", Some(owner)), ("T", target)] {
            if let Some(data) = data {
                let path = self.dir.path().join(format!("{trace}.dat"));
                fs::write(&path, data).unwrap();
                traces.push(TraceValidationInput {
                    trace_id: TraceId(trace.into()),
                    source: InputSource::Path(path),
                });
            }
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
                    ("quarter".into(), NamespaceValue::Integer(q)),
                ]),
                submission_id: id.into(),
            },
            proceed_consent: None,
            origins: vec![],
        }
    }
    fn origin(&self) -> HistoricalOriginAttestation {
        let correlation = vec![b"E01".to_vec()];
        let document = b"Public synthetic document attesting origin in 2026 Q1";
        HistoricalOriginAttestation::from_document(
            HistoricalOriginClaim {
                join_id: JoinId("initial".into()),
                correlation,
                year: 2026,
                quarter: 1,
                reference: "public-synthetic-origin".into(),
                responsible: "synthetic-reviewer".into(),
                recorded_at: "2026-01-01T00:00:00Z".into(),
            },
            document,
            Sha256Digest(Sha256::digest(document).into()),
        )
        .unwrap()
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
fn execute<'r>(
    fixture: &Fixture,
    repo: &'r SQLitePersistenceAdapter,
    request: HistoricalSnapshotValidationRequest,
) -> (HistoricalValidationOutcome<'r>, VecSink) {
    let mut sink = VecSink::default();
    let protector = IdentityProtector::preflight(Some(KEY), &Keys, None).unwrap();
    let result = HistoricalValidationService::new(&fixture.service, repo, &protector)
        .validate_snapshot(
            request,
            &mut sink,
            &mut Audit,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    (result, sink)
}
fn approve(outcome: HistoricalValidationOutcome<'_>) {
    let HistoricalValidationOutcome::Preview(mut preview) = outcome else {
        panic!("expected real preview")
    };
    let review = preview.review();
    preview
        .approve_reviewed(&review, &CancellationToken::default(), |_, _, _| Ok(()))
        .unwrap();
}
fn archive() -> (tempfile::TempDir, SQLitePersistenceAdapter) {
    let dir = tempfile::tempdir_in("target").unwrap();
    let repo =
        SQLitePersistenceAdapter::create(&dir.path().join("archive"), Some(KEY), &Keys).unwrap();
    (dir, repo)
}

#[test]
fn real_snapshot_pass1_pass2_commit_reopen_absent_keeps_witness() {
    let f = Fixture::new();
    let (dir, repo) = archive();
    let request = f.request("T1", "501", 1, "E01\n", Some("E01011A\n"));
    approve(execute(&f, &repo, request).0);
    drop(repo);
    let repo =
        SQLitePersistenceAdapter::open(&dir.path().join("archive"), Some(KEY), &Keys).unwrap();
    let mut request = f.request("T2", "501", 2, "E01\n", None);
    request.proceed_consent = Some(
        MissingTraceProceedConsent::new(
            vec![TraceId("T".into())],
            chrono::DateTime::parse_from_rfc3339("2026-04-01T00:00:00Z")
                .unwrap()
                .to_utc(),
        )
        .unwrap(),
    );
    let (result, sink) = execute(&f, &repo, request);
    assert!(sink.errors().is_empty());
    approve(result);
    let ns = PersistenceNamespace::new(vec![(
        "sending_ulss".into(),
        NamespaceValue::String("501".into()),
    )])
    .unwrap();
    assert_eq!(
        repo.current_generation(&ns).unwrap(),
        Some(SnapshotRevision::new(2))
    );
    let facts = repo
        .load_history_facts(&ns, SnapshotRevision::new(1))
        .unwrap()
        .unwrap();
    assert_eq!(facts.batches()[0].fact_count(), 1);
}
#[test]
fn correction_invalidates_old_initial_and_c1_proves_false() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    let mut request = f.request("T1", "501", 1, "E01\n", Some("E01011A\n"));
    request.origins.push(f.origin());
    let (result, _) = execute(&f, &repo, request);
    if let HistoricalValidationOutcome::Preview(p) = &result {
        assert_eq!(p.history_origins().len(), 1);
    } else {
        panic!("preview missing")
    };
    approve(result);
    let (result, sink) = execute(
        &f,
        &repo,
        f.request("T2", "501", 2, "E01\n", Some("E01012A\n")),
    );
    let HistoricalValidationOutcome::ValidationOnly(outcome) = result else {
        panic!("corrected initial must not survive")
    };
    assert_eq!(outcome.status, RunStatus::CompletedInvalid);
    assert!(
        sink.errors()
            .iter()
            .any(|e| e.code.as_symbol().unwrap() == "E_SYNTHETIC_INITIAL_HISTORY")
    );
}
#[test]
fn no_origin_or_other_namespace_remains_unknown_not_false() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    approve(
        execute(
            &f,
            &repo,
            f.request("T1", "501", 1, "E01\n", Some("E01011A\n")),
        )
        .0,
    );
    for (unit, q) in [("501", 2), ("502", 1)] {
        let (result, sink) = execute(
            &f,
            &repo,
            f.request("T2", unit, q, "E01\n", Some("E01012A\n")),
        );
        let HistoricalValidationOutcome::ValidationOnly(outcome) = result else {
            panic!("unknown cannot approve")
        };
        assert_eq!(outcome.status, RunStatus::CompletedValid);
        assert_eq!(
            outcome.summary.completeness(),
            ValidationCompleteness::Partial
        );
        assert!(sink.errors().is_empty());
    }
}
#[test]
fn independent_local_violation_preserves_positive_but_invalid_operand_does_not() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    for (row, positive) in [("E01011X\n", true), ("E01019A\n", false)] {
        let (result, _) = execute(&f, &repo, f.request("T1", "501", 1, "E01\n", Some(row)));
        let HistoricalValidationOutcome::ValidationOnly(outcome) = result else {
            panic!("invalid run cannot approve")
        };
        assert_eq!(outcome.status, RunStatus::CompletedInvalid);
        let control = &outcome.summary.full_validation.as_ref().unwrap().controls[0];
        assert_eq!(control.satisfied, u64::from(positive));
        assert_eq!(control.unknown, u64::from(!positive));
    }
}

#[test]
fn compiler_binds_semantics_and_rejects_unsupported_historical_operations() {
    let baseline = model();
    let compiled = SchemaCompiler
        .compile_historical_snapshot(raw(baseline.clone()), &[])
        .unwrap();
    assert_eq!(compiled.history_existence().len(), 1);
    let plan = &compiled.history_existence()[0];
    assert_eq!(plan.equality_bytes(), b"1");
    assert_eq!(plan.operand_index(), 2);
    assert_eq!(plan.source_dependencies(), &[0]);
    let mut changed = baseline.clone();
    changed["traces"][0]["rules"][0]["assertions"][0]["predicate"]["eq"]["value"] = json!("2");
    let other = SchemaCompiler
        .compile_historical_snapshot(raw(changed), &[])
        .unwrap();
    assert_ne!(
        plan.definition_hash(),
        other.history_existence()[0].definition_hash()
    );
    for mutation in 0..4 {
        let mut v = baseline.clone();
        match mutation {
            0 => {
                let p = v["traces"][0]["rules"][0]["assertions"][0]["predicate"]["eq"].take();
                v["traces"][0]["rules"][0]["assertions"][0]["predicate"] = json!({"ne":p});
            }
            1 => v["traces"][0]["joins"][0]["cardinality"] = json!("zero_or_one"),
            2 => v["traces"][0]["joins"][0]["namespace_parameters"] = json!([]),
            3 => v["traces"][0]["rules"] = json!([]),
            _ => unreachable!(),
        }
        assert!(
            SchemaCompiler
                .compile_historical_snapshot(raw(v), &[])
                .is_err(),
            "mutation {mutation}"
        );
    }
    assert!(SchemaCompiler.compile(raw(baseline)).is_err());
    let fixture = Fixture::new();
    assert!(fixture.registry.resolve_historical_audit(&key()).is_err());
}

#[test]
fn invalid_when_dependency_does_not_mask_an_independent_rule_on_the_same_join() {
    let mut v = model();
    let mut gate = v["traces"][1]["fields"][3].clone();
    gate["id"] = json!("gate");
    gate["physical"]["start"] = json!(4);
    v["traces"][0]["record_length"] = json!(4);
    v["traces"][0]["fields"].as_array_mut().unwrap().push(gate);
    let mut conditional = v["traces"][0]["rules"][0].clone();
    conditional["id"] = json!("conditional-initial");
    conditional["when"] = json!({"eq":{"ref":{"field":"gate"},"value":"A"}});
    v["traces"][0]["rules"]
        .as_array_mut()
        .unwrap()
        .push(conditional);
    let f = Fixture::with_model(v);
    let (_dir, repo) = archive();
    let (outcome, _) = execute(
        &f,
        &repo,
        f.request("dependency", "501", 1, "E01X\n", Some("E01011A\n")),
    );
    let HistoricalValidationOutcome::ValidationOnly(outcome) = outcome else {
        panic!("invalid gate cannot approve")
    };
    let controls = &outcome.summary.full_validation.unwrap().controls;
    assert_eq!(
        controls
            .iter()
            .find(|c| c.rule_id == "initial-required")
            .unwrap()
            .satisfied,
        1
    );
    assert_eq!(
        controls
            .iter()
            .find(|c| c.rule_id == "conditional-initial")
            .unwrap()
            .unknown,
        1
    );
}

#[test]
fn incompatible_schema_or_predicate_never_uses_old_positives_as_proof() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    approve(
        execute(
            &f,
            &repo,
            f.request("T1", "501", 1, "E01\n", Some("E01011A\n")),
        )
        .0,
    );
    let mut changed = model();
    changed["traces"][0]["rules"][0]["assertions"][0]["predicate"]["eq"]["value"] = json!("2");
    let other = Fixture::with_model(changed);
    let mut request = other.request("T2", "501", 2, "E01\n", None);
    request.proceed_consent = Some(
        MissingTraceProceedConsent::new(vec![TraceId("T".into())], FixedClock.now().into())
            .unwrap(),
    );
    let (outcome, sink) = execute(&other, &repo, request);
    let HistoricalValidationOutcome::ValidationOnly(outcome) = outcome else {
        panic!("incompatible history must not approve")
    };
    assert!(sink.errors().is_empty());
    assert_eq!(
        outcome.summary.completeness(),
        ValidationCompleteness::Partial
    );
    assert_eq!(
        outcome.summary.full_validation.unwrap().controls[0].unknown,
        1
    );
}

#[test]
fn approved_identity_inventory_without_predicate_facts_is_an_explicit_gap() {
    let f = Fixture::new();
    let (_original_dir, original) = archive();
    approve(
        execute(
            &f,
            &original,
            f.request("T1", "501", 1, "E01\n", Some("E01011A\n")),
        )
        .0,
    );
    let ns = PersistenceNamespace::new(vec![(
        "sending_ulss".into(),
        NamespaceValue::String("501".into()),
    )])
    .unwrap();
    let stored = original
        .load_snapshot(&ns, SnapshotRevision::new(1))
        .unwrap()
        .unwrap();
    // Preserve authentic approved source/tokens, deliberately publish via the
    // older constructor with no fact inventory. PK presence proves no Eq fact.
    let legacy = HistoricalSnapshot::new(
        stored.revision(),
        stored.source().clone(),
        stored.identities().to_vec(),
    )
    .unwrap();
    let (_dir, repo) = archive();
    let mut tx = repo.begin_transaction(&ns, None).unwrap();
    tx.stage_snapshot(&legacy).unwrap();
    tx.commit().unwrap();
    let mut request = f.request("T2", "501", 2, "E01\n", None);
    request.proceed_consent = Some(
        MissingTraceProceedConsent::new(vec![TraceId("T".into())], FixedClock.now().into())
            .unwrap(),
    );
    let (outcome, sink) = execute(&f, &repo, request);
    let HistoricalValidationOutcome::ValidationOnly(outcome) = outcome else {
        panic!("legacy tokens are not facts")
    };
    assert!(sink.errors().is_empty());
    assert_eq!(
        outcome.summary.completeness(),
        ValidationCompleteness::Partial
    );
    assert_eq!(
        outcome.summary.full_validation.unwrap().controls[0].unknown,
        1
    );
    assert_eq!(
        repo.current_generation(&ns).unwrap(),
        Some(SnapshotRevision::new(1))
    );
}

#[test]
fn approved_annulment_t2_requires_another_initial_t3_and_allows_reactivation() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    let mut t1 = f.request("T1", "501", 1, "E01\n", Some("E01011A\n"));
    t1.origins.push(f.origin());
    approve(execute(&f, &repo, t1).0);
    approve(execute(&f, &repo, f.request("T2", "501", 2, "", Some(""))).0);
    let (result, sink) = execute(
        &f,
        &repo,
        f.request("T3-invalid", "501", 3, "E01\n", Some("E01022A\n")),
    );
    let HistoricalValidationOutcome::ValidationOnly(o) = result else {
        panic!("annulled witness survived")
    };
    assert_eq!(o.status, RunStatus::CompletedInvalid);
    assert!(
        sink.errors()
            .iter()
            .any(|e| e.code.as_symbol().unwrap() == "E_SYNTHETIC_INITIAL_HISTORY")
    );
    approve(
        execute(
            &f,
            &repo,
            f.request("T3", "501", 3, "E01\n", Some("E01011A\nE01022A\n")),
        )
        .0,
    );
}
#[test]
fn another_valid_initial_survives_and_missing_period_never_proves_false() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    let mut t1 = f.request("T1", "501", 1, "E01\n", Some("E01011A\nE01021A\n"));
    t1.origins.push(f.origin());
    approve(execute(&f, &repo, t1).0);
    let (result, _) = execute(
        &f,
        &repo,
        f.request("T3-gap", "501", 3, "E01\n", Some("E01022A\n")),
    );
    let HistoricalValidationOutcome::ValidationOnly(o) = result else {
        panic!("gap cannot approve")
    };
    assert_eq!(o.status, RunStatus::CompletedValid);
    assert_eq!(o.summary.completeness(), ValidationCompleteness::Partial);
    approve(
        execute(
            &f,
            &repo,
            f.request("T2", "501", 2, "E01\n", Some("E01021A\n")),
        )
        .0,
    );
}
#[test]
fn historical_identity_does_not_reset_at_year_boundary() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    approve(
        execute(
            &f,
            &repo,
            f.request("T4", "501", 4, "E01\n", Some("E01011A\n")),
        )
        .0,
    );
    let mut next = f.request("next-year", "501", 1, "E01\n", None);
    next.validation
        .parameters
        .insert("year".into(), NamespaceValue::Integer(2027));
    next.proceed_consent = Some(
        MissingTraceProceedConsent::new(
            vec![TraceId("T".into())],
            chrono::DateTime::parse_from_rfc3339("2027-01-01T00:00:00Z")
                .unwrap()
                .to_utc(),
        )
        .unwrap(),
    );
    approve(execute(&f, &repo, next).0);
}
#[test]
fn missing_trace_needs_consent_before_history_or_input_access() {
    let f = Fixture::new();
    let request = f.request("T1", "501", 1, "E01\n", None);
    fs::remove_file(f.dir.path().join("O.dat")).unwrap();
    let repository = InMemoryPersistenceRepository::default(); // no key audit: any history read would fail
    let protector = IdentityProtector::preflight(Some(KEY), &Keys, None).unwrap();
    let result = HistoricalValidationService::new(&f.service, &repository, &protector)
        .validate_snapshot(
            request,
            &mut VecSink::default(),
            &mut Audit,
            &mut |_| {},
            &CancellationToken::default(),
        );
    assert!(matches!(
        result,
        Err(HistoricalValidationError::Approval(
            SessionApprovalError::Rejected(SessionRejection::MissingProceedConsent)
        ))
    ));
}
#[test]
fn cancellation_and_error_budget_never_publish_facts() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    let protector = IdentityProtector::preflight(Some(KEY), &Keys, None).unwrap();
    let token = CancellationToken::default();
    let mut sink = VecSink::default();
    let result = HistoricalValidationService::new(&f.service, &repo, &protector)
        .validate_snapshot(
            f.request("cancel", "501", 1, "E01\n", Some("E01011A\n")),
            &mut sink,
            &mut Audit,
            &mut |e| {
                if matches!(e, ProgressEvent::TraceStarted { .. }) {
                    token.cancel();
                }
            },
            &token,
        )
        .unwrap();
    let HistoricalValidationOutcome::ValidationOnly(o) = result else {
        panic!("cancelled preview")
    };
    assert_eq!(o.status, RunStatus::Cancelled);
    let mut request = f.request("budget", "501", 1, "bad\n", Some("E01011A\n"));
    request.validation.validation.options.max_errors = Some(1);
    let (result, sink) = execute(&f, &repo, request);
    let HistoricalValidationOutcome::ValidationOnly(o) = result else {
        panic!("truncated preview")
    };
    assert!(o.summary.truncated);
    assert_eq!(sink.errors().len(), 1);
    let ns = PersistenceNamespace::new(vec![(
        "sending_ulss".into(),
        NamespaceValue::String("501".into()),
    )])
    .unwrap();
    assert_eq!(repo.current_generation(&ns).unwrap(), None);
}
#[test]
fn pass_two_and_approval_do_not_reopen_the_input_paths() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    let protector = IdentityProtector::preflight(Some(KEY), &Keys, None).unwrap();
    let result = HistoricalValidationService::new(&f.service, &repo, &protector)
        .validate_snapshot(
            f.request("T1", "501", 1, "E01\n", Some("E01011A\n")),
            &mut VecSink::default(),
            &mut Audit,
            &mut |e| {
                if e == ProgressEvent::PhaseStarted(ValidationPhase::CrossRecordValidation) {
                    for id in ["O", "T"] {
                        fs::remove_file(f.dir.path().join(format!("{id}.dat"))).unwrap();
                    }
                }
            },
            &CancellationToken::default(),
        )
        .unwrap();
    assert!(!f.dir.path().join("O.dat").exists());
    approve(result);
}
#[test]
fn report_abort_cas_and_repeat_approval_preserve_the_exact_unit() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    let (result, _) = execute(
        &f,
        &repo,
        f.request("abort", "501", 1, "E01\n", Some("E01011A\n")),
    );
    let HistoricalValidationOutcome::Preview(mut abort) = result else {
        panic!("preview")
    };
    assert!(matches!(
        abort.approve_commit(&CancellationToken::default(), |_, _, _| Err(
            ReportError::Write(std::io::Error::other("synthetic report failure"))
        )),
        Err(SessionApprovalError::Report(_))
    ));
    abort.cancel().unwrap();
    assert!(matches!(
        abort.approve_commit(&CancellationToken::default(), |_, _, _| Ok(())),
        Err(SessionApprovalError::Cancelled)
    ));
    let (first, _) = execute(
        &f,
        &repo,
        f.request("one", "501", 1, "E01\n", Some("E01011A\n")),
    );
    let (second, _) = execute(
        &f,
        &repo,
        f.request("two", "501", 1, "E01\n", Some("E01011A\n")),
    );
    let HistoricalValidationOutcome::Preview(mut first) = first else {
        panic!("preview")
    };
    let HistoricalValidationOutcome::Preview(mut second) = second else {
        panic!("preview")
    };
    let receipt = first
        .approve_commit(&CancellationToken::default(), |_, _, _| Ok(()))
        .unwrap();
    assert_eq!(
        first
            .approve_commit(&CancellationToken::default(), |_, _, _| panic!(
                "replay must not republish reports"
            ))
            .unwrap(),
        receipt
    );
    assert!(matches!(
        second.approve_commit(&CancellationToken::default(), |_, _, _| Ok(())),
        Err(SessionApprovalError::Persistence(
            PersistenceError::GenerationConflict { .. }
        ))
    ));
    let ns = PersistenceNamespace::new(vec![(
        "sending_ulss".into(),
        NamespaceValue::String("501".into()),
    )])
    .unwrap();
    assert_eq!(
        repo.current_generation(&ns).unwrap(),
        Some(SnapshotRevision::new(1))
    );
    assert!(
        repo.load_history_facts(&ns, SnapshotRevision::new(2))
            .unwrap()
            .is_none()
    );
}
#[test]
fn replay_reconstructs_original_origins_after_later_approved_attestations() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    approve(
        execute(
            &f,
            &repo,
            f.request("T1", "501", 1, "E01\n", Some("E01011A\n")),
        )
        .0,
    );
    let mut later = f.request("T2", "501", 2, "E01\n", Some("E01011A\n"));
    later.origins.push(f.origin());
    approve(execute(&f, &repo, later).0);
    // T1 originally had no origin. Reconstructing its candidate with T2's C1
    // claim would change its authenticated batch and wrongly reject replay.
    let (replayed, _) = execute(
        &f,
        &repo,
        f.request("T1", "501", 1, "E01\n", Some("E01011A\n")),
    );
    approve(replayed);
    let ns = PersistenceNamespace::new(vec![(
        "sending_ulss".into(),
        NamespaceValue::String("501".into()),
    )])
    .unwrap();
    assert_eq!(
        repo.current_generation(&ns).unwrap(),
        Some(SnapshotRevision::new(2))
    );
}

#[test]
fn a_new_backdated_snapshot_cannot_read_future_facts_or_replace_live_state() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    approve(
        execute(
            &f,
            &repo,
            f.request("T2", "501", 2, "E01\n", Some("E01011A\n")),
        )
        .0,
    );
    let mut request = f.request("new-T1", "501", 1, "E01\n", Some("E01011A\n"));
    if let ValidationScope::Flow(flow) = &mut request.validation.validation.scope {
        for input in &mut flow.traces {
            input.source = InputSource::Path(f.dir.path().join("must-not-open"));
        }
    }
    let protector = IdentityProtector::preflight(Some(KEY), &Keys, None).unwrap();
    assert!(matches!(
        HistoricalValidationService::new(&f.service, &repo, &protector).validate_snapshot(
            request,
            &mut VecSink::default(),
            &mut Audit,
            &mut |_| {},
            &CancellationToken::default()
        ),
        Err(HistoricalValidationError::InvalidRequest)
    ));
}
#[test]
fn persisted_fact_tampering_is_technical_even_with_a_current_positive() {
    for mutation in [
        "DELETE FROM history_batches",
        "UPDATE history_batches SET record_proof=zeroblob(32)",
        "DELETE FROM snapshot_manifests",
    ] {
        let f = Fixture::new();
        let (dir, repo) = archive();
        approve(
            execute(
                &f,
                &repo,
                f.request("T1", "501", 1, "E01\n", Some("E01011A\n")),
            )
            .0,
        );
        let connection = rusqlite::Connection::open(dir.path().join("archive/lintra.db")).unwrap();
        connection.execute_batch(mutation).unwrap();
        drop(connection);
        let protector = IdentityProtector::preflight(Some(KEY), &Keys, None).unwrap();
        let result = HistoricalValidationService::new(&f.service, &repo, &protector)
            .validate_snapshot(
                f.request("T2", "501", 2, "E01\n", Some("E01011A\n")),
                &mut VecSink::default(),
                &mut Audit,
                &mut |_| {},
                &CancellationToken::default(),
            );
        assert!(
            matches!(
                result,
                Err(HistoricalValidationError::Persistence(
                    PersistenceError::CorruptStorage
                ))
            ),
            "{mutation}"
        );
    }
}
#[test]
fn facts_and_documents_never_persist_raw_event_rows_or_key_material() {
    let f = Fixture::new();
    let (dir, repo) = archive();
    let mut request = f.request("T1", "501", 1, "E01\n", Some("E01011A\n"));
    request.origins.push(f.origin());
    approve(execute(&f, &repo, request).0);
    drop(repo);
    let bytes = fs::read(dir.path().join("archive/lintra.db")).unwrap();
    for marker in [b"E01".as_slice(), b"E01011A".as_slice(), &[0xaa; 32]] {
        assert!(!bytes.windows(marker.len()).any(|w| w == marker));
    }
}

#[test]
fn full_cad_candidate_compiles_83_source_fields_without_filler() {
    let v: Value = serde_json::from_str(include_str!(
        "../schemas/operational/CAD/2024-06.runtime-candidate.1/flow.json"
    ))
    .unwrap();
    assert_eq!(
        v["traces"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["fields"].as_array().unwrap().len())
            .sum::<usize>(),
        83
    );
    assert!(
        v["traces"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|t| t["fields"].as_array().unwrap())
            .all(|f| f["id"].as_str().unwrap().contains('.'))
    );
    let compiled = SchemaCompiler
        .compile_historical_snapshot(raw(v), &[])
        .unwrap();
    assert_eq!(compiled.profile().trace_ids().len(), 5);
    assert_eq!(compiled.history_existence().len(), 1);
}

#[test]
fn absent_source_in_explicit_snapshot_is_preserved_with_consent() {
    let f = Fixture::new();
    let (_dir, repo) = archive();
    let mut request = f.request("T1", "501", 1, "", Some("E01011A\n"));
    let ValidationScope::Flow(input) = &mut request.validation.validation.scope else {
        unreachable!()
    };
    input.traces.retain(|t| t.trace_id.0 != "O");
    request.proceed_consent = Some(
        MissingTraceProceedConsent::new(
            vec![TraceId("O".into())],
            chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
                .unwrap()
                .to_utc(),
        )
        .unwrap(),
    );
    let (result, _) = execute(&f, &repo, request);
    approve(result);
}
