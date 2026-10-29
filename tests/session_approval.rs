//! Synthetic component acceptance. Outcomes below are explicit summary fixtures,
//! not claims that the still-gated historical ValidationService has executed.
use chrono::{TimeZone, Utc};
use lintra::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, io, sync::Arc, time::Duration};
struct Provider;
impl PersistenceKeyProvider for Provider {
    fn resolve(&self, id: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        if id != "synthetic-key" {
            return Err(ProtectionError::MissingKey);
        }
        Ok(Arc::new(PersistenceKey::new(vec![0xaa; 64])?))
    }
}
fn protector() -> IdentityProtector {
    IdentityProtector::preflight(Some("synthetic-key"), &Provider, None).unwrap()
}
fn model() -> Value {
    let mut v: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    let mut second = v["traces"][0].clone();
    second["id"] = json!("T2");
    v["traces"].as_array_mut().unwrap().push(second);
    v["parameters"] = json!([
        {"id":"unit","type":"string","allowed_values":["UNIT-A","UNIT-B"]},
        {"id":"year","type":"integer","min":2020,"max":2100},
        {"id":"quarter","type":"integer","min":1,"max":4}
    ]);
    v["required_capabilities"] = json!(["local.v1", "persistence.snapshot.v1"]);
    v["persistence_policy"] = json!({
        "namespace_parameters":["unit"],"version_compatibility":{"kind":"same_schema_version"},
        "run_uniqueness":[],"cross_run_audit":[],"history_identity_invariants":[],"temporal_rules":[],
        "storage_protection":"external_secure_configuration_required",
        "snapshot":{"scope_parameters":["unit"],"period":{"year_parameter":"year","quarter_parameter":"quarter"},
            "approved_immutable":true,"require_explicit_approval":true,"commit":"atomic_idempotent",
            "missing_trace_policy":{"consent":"proceed_or_abort","warning_code":"W_SYNTHETIC_MISSING",
                "on_absent_trace":"preserve","present_trace_is_complete":true}}
    });
    v
}
fn raw(v: Value) -> RawFlowSchema {
    let mut raw: RawFlowSchema = serde_json::from_value(v).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    raw
}
fn profile() -> CompiledPersistenceProfile {
    SchemaCompiler
        .compile_persistence_profile(&raw(model()), &[])
        .unwrap()
}
fn parameters(unit: &str, q: i64) -> BTreeMap<String, NamespaceValue> {
    [
        ("unit".into(), NamespaceValue::String(unit.into())),
        ("year".into(), NamespaceValue::Integer(2026)),
        ("quarter".into(), NamespaceValue::Integer(q)),
    ]
    .into()
}
fn parts(
    p: &CompiledPersistenceProfile,
    unit: &str,
    id: &str,
    absent: bool,
) -> SourceSubmissionParts {
    SourceSubmissionParts {
        namespace: PersistenceNamespace::new(vec![(
            "unit".into(),
            NamespaceValue::String(unit.into()),
        )])
        .unwrap(),
        submission_id: id.into(),
        schema_key: p.schema_key().clone(),
        schema_hash: p.schema_hash().clone(),
        period: None,
        created_at: Utc.with_ymd_and_hms(2026, 1, 2, 3, 0, 0).unwrap(),
        validator_version: "synthetic-test".into(),
        validation_mode: ValidationMode::Full,
        validation_status: RunStatus::CompletedValid,
        reference_data_versions: vec![],
        traces: p
            .trace_ids()
            .iter()
            .map(|id| SubmissionTrace {
                trace_id: id.clone(),
                presence: if absent && id.0 == "T2" {
                    TracePresence::Absent
                } else {
                    TracePresence::Present { record_count: 0 }
                },
                input_sha256: if absent && id.0 == "T2" {
                    None
                } else {
                    Some(Sha256Digest([0; 32]))
                },
            })
            .collect(),
        fingerprint: Fingerprint::new(
            "synthetic-preparation".into(),
            "synthetic-key".into(),
            vec![1, 2, 3],
        )
        .unwrap(),
        state: SessionState::Draft,
    }
}
fn outcome(
    parts: &SourceSubmissionParts,
    parameters: &BTreeMap<String, NamespaceValue>,
) -> ValidationRunOutcome {
    let present: Vec<_> = parts
        .traces
        .iter()
        .filter(|t| matches!(t.presence, TracePresence::Present { .. }))
        .collect();
    let summary = ValidationSummary {
        started_at: parts.created_at.into(),
        validation_mode: ValidationMode::Full,
        run_status: RunStatus::CompletedValid,
        schema_key: parts.schema_key.clone(),
        scope_summary: ScopeSummary::Flow {
            trace_ids: present.iter().map(|t| t.trace_id.clone()).collect(),
        },
        records_processed: 0,
        valid_records: 0,
        invalid_records: 0,
        total_errors: 0,
        errors_by_code: BTreeMap::new(),
        skipped_rules: 0,
        truncated: false,
        validator_version: parts.validator_version.clone(),
        schema_hash: parts.schema_hash.to_string(),
        reference_data_versions: BTreeMap::new(),
        reference_validation: None,
        full_validation: Some(FullValidationSummary {
            completeness: ValidationCompleteness::Complete,
            parameters: parameters
                .iter()
                .map(|(id, v)| {
                    (
                        id.clone(),
                        match v {
                            NamespaceValue::String(v) => RunParameterAudit::String(v.clone()),
                            NamespaceValue::Integer(v) => RunParameterAudit::Integer(v.to_string()),
                        },
                    )
                })
                .collect(),
            traces: parts
                .traces
                .iter()
                .map(|t| FlowTraceEvidence {
                    trace_id: t.trace_id.clone(),
                    coverage: if t.presence == TracePresence::Absent {
                        ContextCoverage::Unavailable
                    } else {
                        ContextCoverage::Complete
                    },
                    records_processed: 0,
                    unprojectable_records: 0,
                    cross_rules_declared: 0,
                    record_keys_checked: 0,
                    duplicate_record_keys: 0,
                    inconclusive_record_keys: 0,
                })
                .collect(),
            controls: vec![],
        }),
        elapsed: Duration::ZERO,
        inputs: present
            .iter()
            .map(|t| InputAudit {
                trace_id: t.trace_id.clone(),
                file_name: "synthetic.dat".into(),
                file_size_bytes: 0,
                bytes_read: 0,
                completion: InputAuditCompletion::Complete {
                    sha256: t.input_sha256.clone().unwrap(),
                },
            })
            .collect(),
    };
    summary.validate().unwrap();
    ValidationRunOutcome {
        status: summary.run_status,
        summary,
    }
}
fn validated(unit: &str, id: &str, quarter: i64) -> ValidatedApproval {
    let p = profile();
    let params = parameters(unit, quarter);
    let parts = parts(&p, unit, id, false);
    let result = outcome(&parts, &params);
    let draft = ApprovalDraft::new(&p, params, parts, None).unwrap();
    assert_eq!(draft.state(), SessionState::Draft);
    draft.validated(&result, vec![], &protector()).unwrap()
}
fn approve(v: &ValidatedApproval, repo: &dyn PersistentContextRepository) -> CommitReceipt {
    v.preview(repo)
        .unwrap()
        .approve_commit(&CancellationToken::default(), |_| Ok(()))
        .unwrap()
}
#[test]
fn explicit_policy_lowering_rejects_invalid_variants_and_keeps_runtime_gate() {
    let p = profile();
    let plan = p.policy().unwrap().snapshot.as_ref().unwrap();
    assert_eq!(plan.scope_parameters, [0]);
    assert_eq!(plan.year_parameter, 1);
    assert_eq!(plan.quarter_parameter, 2);
    assert!(SchemaCompiler.compile(raw(model())).is_err());
    for pointer in [
        "/persistence_policy/snapshot/approved_immutable",
        "/persistence_policy/snapshot/require_explicit_approval",
        "/persistence_policy/snapshot/missing_trace_policy/present_trace_is_complete",
    ] {
        let mut v = model();
        *v.pointer_mut(pointer).unwrap() = json!(false);
        assert!(
            SchemaCompiler
                .compile_persistence_profile(&raw(v), &[])
                .is_err()
        );
    }
    for change in [
        json!({"year_parameter":"unit","quarter_parameter":"quarter"}),
        json!({"year_parameter":"year","quarter_parameter":"year"}),
        json!({"year_parameter":"absent","quarter_parameter":"quarter"}),
    ] {
        let mut v = model();
        v["persistence_policy"]["snapshot"]["period"] = change;
        assert!(
            SchemaCompiler
                .compile_persistence_profile(&raw(v), &[])
                .is_err()
        );
    }
    let mut v = model();
    v["required_capabilities"] = json!(["local.v1"]);
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&raw(v), &[])
            .is_err()
    );
}
#[test]
fn draft_binding_validates_every_parameter_inventory_and_provenance() {
    let p = profile();
    for params in [
        parameters("UNIT-A", 0),
        parameters("UNKNOWN", 1),
        {
            let mut v = parameters("UNIT-A", 1);
            v.insert("unused".into(), NamespaceValue::Integer(1));
            v
        },
        {
            let mut v = parameters("UNIT-A", 1);
            v.remove("year");
            v
        },
    ] {
        assert!(ApprovalDraft::new(&p, params, parts(&p, "UNIT-A", "run", false), None).is_err());
    }
    for mutate in 0..5 {
        let mut s = parts(&p, "UNIT-A", "run", false);
        match mutate {
            0 => s.state = SessionState::ApprovedImmutable,
            1 => s.period = Some("free form period".into()),
            2 => s.schema_hash = Sha256Digest([1; 32]),
            3 => s.traces.clear(),
            _ => s.schema_key.version = SchemaVersion("wrong".into()),
        };
        assert!(ApprovalDraft::new(&p, parameters("UNIT-A", 1), s, None).is_err());
    }
}
#[test]
fn outcome_failures_mismatch_cancel_and_missing_consent_never_become_validated() {
    let p = profile();
    let params = parameters("UNIT-A", 1);
    for mutation in 0..7 {
        let s = parts(&p, "UNIT-A", "run", false);
        let mut result = outcome(&s, &params);
        match mutation {
            0 => result.status = RunStatus::Cancelled,
            1 => result.summary.truncated = true,
            2 => result.summary.schema_hash = "wrong".into(),
            3 => result.summary.validation_mode = ValidationMode::LocalOnly,
            4 => result
                .summary
                .full_validation
                .as_mut()
                .unwrap()
                .parameters
                .clear(),
            5 => result.summary.inputs[0].completion = InputAuditCompletion::Incomplete,
            _ => {
                result.summary.inputs[0].completion = InputAuditCompletion::Complete {
                    sha256: Sha256Digest([1; 32]),
                }
            }
        };
        assert!(
            ApprovalDraft::new(&p, params.clone(), s, None)
                .unwrap()
                .validated(&result, vec![], &protector())
                .is_err()
        );
    }
    let s = parts(&p, "UNIT-A", "absent", true);
    let result = outcome(&s, &params);
    assert!(matches!(
        ApprovalDraft::new(&p, params.clone(), s.clone(), None)
            .unwrap()
            .validated(&result, vec![], &protector()),
        Err(SessionApprovalError::Rejected(
            SessionRejection::MissingProceedConsent
        ))
    ));
    let consent =
        MissingTraceProceedConsent::new(vec![TraceId("T2".into())], s.created_at).unwrap();
    let draft = ApprovalDraft::new(&p, params, s, None).unwrap();
    assert_eq!(
        draft.missing_trace_warning_code(),
        Some("W_SYNTHETIC_MISSING")
    );
    assert_eq!(
        draft.absent_traces().collect::<Vec<_>>(),
        vec![&TraceId("T2".into())]
    );
    let v = draft
        .with_proceed_consent(consent)
        .unwrap()
        .validated(&result, vec![], &protector())
        .unwrap();
    assert_eq!(v.missing_trace_warning_code(), Some("W_SYNTHETIC_MISSING"));
}
#[test]
fn preview_abort_drop_report_error_and_cooperative_cancel_publish_nothing() {
    let repo = InMemoryPersistenceRepository::default();
    let v = validated("UNIT-A", "run", 1);
    let ns = v.source().namespace();
    {
        let p = v.preview(&repo).unwrap();
        assert_eq!(p.state(), Some(SessionState::Validated));
        assert!(repo.current_generation(ns).unwrap().is_none());
    }
    let mut p = v.preview(&repo).unwrap();
    p.cancel().unwrap();
    p.cancel().unwrap();
    assert!(matches!(
        p.approve_commit(&CancellationToken::default(), |_| panic!(
            "must not publish report"
        )),
        Err(SessionApprovalError::Cancelled)
    ));
    let mut p = v.preview(&repo).unwrap();
    assert!(matches!(
        p.approve_commit(&CancellationToken::default(), |_| Err(ReportError::Write(
            io::Error::other("synthetic IO")
        ))),
        Err(SessionApprovalError::Report(_))
    ));
    assert_eq!(p.state(), Some(SessionState::Validated));
    let token = CancellationToken::default();
    assert!(matches!(
        p.approve_commit(&token, |_| {
            token.cancel();
            Ok(())
        }),
        Err(SessionApprovalError::Cancelled)
    ));
    assert!(repo.current_generation(ns).unwrap().is_none());
    assert!(repo.load_submission_page(ns, None).unwrap().is_empty());
}
#[test]
fn immutable_old_session_exact_retry_and_namespace_isolation() {
    let repo = InMemoryPersistenceRepository::default();
    let v = validated("UNIT-A", "run-1", 1);
    let mut p = v.preview(&repo).unwrap();
    let receipt = p
        .approve_commit(&CancellationToken::default(), |_| Ok(()))
        .unwrap();
    assert_eq!(receipt.revision().value(), 1);
    assert_eq!(p.state(), Some(SessionState::ApprovedImmutable));
    assert!(p.cancel().is_err());
    assert_eq!(
        p.approve_commit(&CancellationToken::default(), |_| panic!(
            "idempotent retry must not redo report"
        ))
        .unwrap(),
        receipt
    );
    approve(&validated("UNIT-A", "run-2", 2), &repo);
    assert!(matches!(
        validated("UNIT-A", "retroactive", 1).preview(&repo),
        Err(SessionApprovalError::Rejected(
            SessionRejection::ApprovedSessionImmutable
        ))
    ));
    let mut retry = v.preview(&repo).unwrap();
    assert_eq!(
        retry
            .approve_commit(&CancellationToken::default(), |_| panic!(
                "restart retry is read only"
            ))
            .unwrap(),
        receipt
    );
    assert_eq!(
        repo.current_generation(v.source().namespace())
            .unwrap()
            .unwrap()
            .value(),
        2
    );
    assert_eq!(
        approve(&validated("UNIT-B", "independent", 1), &repo)
            .revision()
            .value(),
        1
    );
}
#[test]
fn stale_preview_and_double_transaction_do_not_publish_second_approval() {
    let repo = InMemoryPersistenceRepository::default();
    let a = validated("UNIT-A", "a", 1);
    let b = validated("UNIT-A", "b", 1);
    let mut pa = a.preview(&repo).unwrap();
    let mut pb = b.preview(&repo).unwrap();
    pa.approve_commit(&CancellationToken::default(), |_| Ok(()))
        .unwrap();
    assert!(matches!(
        pb.approve_commit(&CancellationToken::default(), |_| Ok(())),
        Err(SessionApprovalError::Persistence(
            PersistenceError::GenerationConflict { .. }
        ))
    ));
    assert_eq!(
        repo.current_generation(a.source().namespace())
            .unwrap()
            .unwrap()
            .value(),
        1
    );
}
#[test]
fn sqlite_restart_metadata_cursor_and_legacy_history_fail_closed() {
    let temp =
        tempfile::tempdir_in(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target"))
            .unwrap();
    let root = temp.path().join("archive");
    let v = validated("UNIT-A", "run-1", 1);
    let repo = SQLitePersistenceAdapter::create(&root, Some("synthetic-key"), &Provider).unwrap();
    approve(&v, &repo);
    let settings = repo.settings().unwrap();
    assert_eq!(settings.journal_mode, "delete");
    assert_eq!(settings.synchronous, 3);
    drop(repo);
    let repo = SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider).unwrap();
    assert_eq!(
        repo.load_submission_page(v.source().namespace(), None)
            .unwrap()
            .len(),
        1
    );
    assert!(
        repo.load_submission_page(v.source().namespace(), Some(SnapshotRevision::new(1)))
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        validated("UNIT-A", "rewrite", 1).preview(&repo),
        Err(SessionApprovalError::Rejected(
            SessionRejection::ApprovedSessionImmutable
        ))
    ));
    assert_eq!(approve(&v, &repo).revision().value(), 1);
    let p = profile();
    let mut legacy = parts(&p, "UNIT-B", "legacy", false);
    legacy.state = SessionState::ApprovedImmutable;
    legacy.period = Some("opaque-old-period".into());
    let source = repo
        .protector()
        .protect_submission(&SourceSubmission::new(legacy).unwrap())
        .unwrap();
    let ns = source.namespace().clone();
    let mut tx = repo.begin_transaction(&ns, None).unwrap();
    tx.stage_snapshot(&HistoricalSnapshot::new(SnapshotRevision::new(1), source, vec![]).unwrap())
        .unwrap();
    tx.commit().unwrap();
    assert!(matches!(
        validated("UNIT-B", "new", 2).preview(&repo),
        Err(SessionApprovalError::UnboundApprovedHistory)
    ));
}

/// Faults exercise the coordinator, not SQLite durability (already qualified by
/// PERSIST05/06). The backing fake is used only for deterministic state assertions.
struct FaultRepo {
    real: InMemoryPersistenceRepository,
    mode: std::sync::atomic::AtomicU8,
}
struct FaultTransaction {
    inner: Box<dyn PersistenceTransaction>,
    mode: u8,
}
impl PersistenceTransaction for FaultTransaction {
    fn stage_snapshot(&mut self, s: &HistoricalSnapshot) -> Result<(), PersistenceError> {
        if self.mode == 4 {
            return Err(PersistenceError::StorageUnavailable);
        }
        self.inner.stage_snapshot(s)
    }
    fn commit(self: Box<Self>) -> Result<CommitReceipt, PersistenceError> {
        let Self { inner, mode } = *self;
        if mode == 1 {
            return Err(PersistenceError::StorageUnavailable);
        }
        if mode == 2 {
            return Err(PersistenceError::CommitOutcomeUnknown {
                revision: SnapshotRevision::new(1),
            });
        }
        let receipt = inner.commit()?;
        if mode == 3 {
            return Err(PersistenceError::CommitOutcomeUnknown {
                revision: receipt.revision(),
            });
        }
        Ok(receipt)
    }
    fn abort(self: Box<Self>) -> Result<(), PersistenceError> {
        self.inner.abort()
    }
}
impl PersistentContextRepository for FaultRepo {
    fn lookup_identity(
        &self,
        ns: &PersistenceNamespace,
        id: &PersistentRecordIdentity,
    ) -> Result<Option<StoredIdentity>, PersistenceError> {
        self.real.lookup_identity(ns, id)
    }
    fn load_current_snapshot(
        &self,
        ns: &PersistenceNamespace,
    ) -> Result<Option<HistoricalSnapshot>, PersistenceError> {
        self.real.load_current_snapshot(ns)
    }
    fn load_snapshot(
        &self,
        ns: &PersistenceNamespace,
        r: SnapshotRevision,
    ) -> Result<Option<HistoricalSnapshot>, PersistenceError> {
        self.real.load_snapshot(ns, r)
    }
    fn load_submission(
        &self,
        ns: &PersistenceNamespace,
        id: &str,
    ) -> Result<Option<SourceSubmission>, PersistenceError> {
        self.real.load_submission(ns, id)
    }
    fn current_generation(
        &self,
        ns: &PersistenceNamespace,
    ) -> Result<Option<SnapshotRevision>, PersistenceError> {
        self.real.current_generation(ns)
    }
    fn load_submission_page(
        &self,
        ns: &PersistenceNamespace,
        after: Option<SnapshotRevision>,
    ) -> Result<Vec<(SnapshotRevision, SourceSubmission)>, PersistenceError> {
        self.real.load_submission_page(ns, after)
    }
    fn begin_transaction(
        &self,
        ns: &PersistenceNamespace,
        base: Option<SnapshotRevision>,
    ) -> Result<Box<dyn PersistenceTransaction>, PersistenceError> {
        Ok(Box::new(FaultTransaction {
            inner: self.real.begin_transaction(ns, base)?,
            mode: self.mode.load(std::sync::atomic::Ordering::Relaxed),
        }))
    }
}
#[test]
fn storage_failure_and_ambiguous_commit_require_explicit_read_only_resolution() {
    for mode in 1..=4 {
        let repo = FaultRepo {
            real: InMemoryPersistenceRepository::default(),
            mode: std::sync::atomic::AtomicU8::new(mode),
        };
        let v = validated("UNIT-A", "attempt", 1);
        let mut p = v.preview(&repo).unwrap();
        assert!(
            p.approve_commit(&CancellationToken::default(), |_| Ok(()))
                .is_err()
        );
        assert_eq!(
            repo.current_generation(v.source().namespace())
                .unwrap()
                .is_some(),
            mode == 3
        );
        if mode == 2 || mode == 3 {
            assert_eq!(p.state(), None);
            assert!(p.cancel().is_err());
            assert!(matches!(
                p.approve_commit(&CancellationToken::default(), |_| panic!(
                    "uncertain must not retry report/write"
                )),
                Err(SessionApprovalError::Persistence(
                    PersistenceError::CommitOutcomeUnknown { .. }
                ))
            ));
            let resolved = p.resolve_outcome().unwrap();
            if mode == 3 {
                assert!(matches!(resolved, CommitResolution::Published(_)));
                assert_eq!(p.state(), Some(SessionState::ApprovedImmutable));
                assert_eq!(
                    p.approve_commit(&CancellationToken::default(), |_| panic!(
                        "already committed"
                    ))
                    .unwrap()
                    .revision()
                    .value(),
                    1
                );
            } else {
                assert_eq!(resolved, CommitResolution::NotPublished);
                assert_eq!(p.state(), Some(SessionState::Validated));
                repo.mode.store(0, std::sync::atomic::Ordering::Relaxed);
                p.approve_commit(&CancellationToken::default(), |_| Ok(()))
                    .unwrap();
            }
        } else {
            assert_eq!(p.state(), Some(SessionState::Validated));
        }
    }
}
#[test]
fn sqlite_cursor_crosses_page_boundaries_without_loading_record_payloads() {
    let temp =
        tempfile::tempdir_in(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target"))
            .unwrap();
    let root = temp.path().join("archive");
    let repo = SQLitePersistenceAdapter::create(&root, Some("synthetic-key"), &Provider).unwrap();
    let p = profile();
    let ns = parts(&p, "UNIT-A", "x", false).namespace;
    // Unrelated flow metadata exercises a namespace shared by independent flows.
    for i in 1..=130 {
        let mut source = parts(&p, "UNIT-A", &format!("other-{i}"), false);
        source.schema_key.flow_id = FlowId("OTHER-SYNTHETIC".into());
        source.state = SessionState::ApprovedImmutable;
        let source = repo
            .protector()
            .protect_submission(&SourceSubmission::new(source).unwrap())
            .unwrap();
        let base = repo.current_generation(&ns).unwrap();
        let mut tx = repo.begin_transaction(&ns, base).unwrap();
        tx.stage_snapshot(
            &HistoricalSnapshot::new(SnapshotRevision::new(i), source, vec![]).unwrap(),
        )
        .unwrap();
        tx.commit().unwrap();
    }
    let first = repo.load_submission_page(&ns, None).unwrap();
    assert_eq!(first.len(), 128);
    assert_eq!(first[127].0.value(), 128);
    let last = repo.load_submission_page(&ns, Some(first[127].0)).unwrap();
    assert_eq!(last.len(), 2);
    assert_eq!(last[0].0.value(), 129);
    let v = validated("UNIT-A", "after-page-1", 1);
    assert_eq!(approve(&v, &repo).revision().value(), 131);
    let other = SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider).unwrap();
    assert!(matches!(
        validated("UNIT-A", "retroactive", 1).preview(&other),
        Err(SessionApprovalError::Rejected(
            SessionRejection::ApprovedSessionImmutable
        ))
    ));
    let later = validated("UNIT-A", "later", 2);
    let mut a = later.preview(&repo).unwrap();
    let mut b = validated("UNIT-A", "contending", 2)
        .preview(&other)
        .unwrap();
    a.approve_commit(&CancellationToken::default(), |_| Ok(()))
        .unwrap();
    assert!(matches!(
        b.approve_commit(&CancellationToken::default(), |_| Ok(())),
        Err(SessionApprovalError::Persistence(
            PersistenceError::GenerationConflict { .. }
        ))
    ));
    // This is the actual bounded adapter query, using the pre-existing PK.
    let db = rusqlite::Connection::open_with_flags(
        root.join("lintra.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let plan:String=db.query_row("EXPLAIN QUERY PLAN SELECT revision,submission,source,proof FROM snapshots WHERE ns=?1 AND revision>?2 ORDER BY revision LIMIT 128",rusqlite::params![serde_json::to_vec(&ns).unwrap(),0],|r| r.get(3)).unwrap();
    assert!(
        plan.contains("SEARCH") && plan.contains("PRIMARY KEY"),
        "{plan}"
    );
    assert!(!plan.contains("SCAN"), "{plan}");
}

#[test]
fn schema_version_change_does_not_unlock_a_session_and_cancelled_preview_is_read_only() {
    let repo = InMemoryPersistenceRepository::default();
    let old = validated("UNIT-A", "old", 1);
    approve(&old, &repo);
    let mut v = model();
    v["schema_version"] = json!("v2");
    let p = SchemaCompiler
        .compile_persistence_profile(&raw(v), &[])
        .unwrap();
    let params = parameters("UNIT-A", 1);
    let s = parts(&p, "UNIT-A", "new-version", false);
    let result = outcome(&s, &params);
    let newer = ApprovalDraft::new(&p, params, s, None)
        .unwrap()
        .validated(&result, vec![], &protector())
        .unwrap();
    assert!(matches!(
        newer.preview(&repo),
        Err(SessionApprovalError::Rejected(
            SessionRejection::ApprovedSessionImmutable
        ))
    ));
    let token = CancellationToken::default();
    token.cancel();
    assert!(matches!(
        newer.preview_with_cancellation(&repo, &token),
        Err(SessionApprovalError::Cancelled)
    ));
    let retry = old.preview(&repo).unwrap();
    assert_eq!(retry.snapshot().revision().value(), 1);
    assert_eq!(
        retry.snapshot().source().state(),
        SessionState::ApprovedImmutable
    );
    assert_eq!(retry.expected_generation(), None);
    assert_eq!(
        repo.current_generation(old.source().namespace())
            .unwrap()
            .unwrap()
            .value(),
        1
    );
}

#[test]
fn corrupt_metadata_indexes_cannot_masquerade_as_an_empty_or_different_session() {
    for broken_head in [false, true] {
        let temp =
            tempfile::tempdir_in(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target"))
                .unwrap();
        let root = temp.path().join("archive");
        let repo =
            SQLitePersistenceAdapter::create(&root, Some("synthetic-key"), &Provider).unwrap();
        let v = validated("UNIT-A", "original", 1);
        approve(&v, &repo);
        let db = rusqlite::Connection::open(root.join("lintra.db")).unwrap();
        if broken_head {
            db.execute("UPDATE namespaces SET generation=99", [])
                .unwrap();
        } else {
            db.execute(
                "UPDATE snapshots SET submission='synthetic-tampered-index'",
                [],
            )
            .unwrap();
        }
        assert!(matches!(
            validated("UNIT-A", "new", 2).preview(&repo),
            Err(SessionApprovalError::Persistence(
                PersistenceError::CorruptStorage
            ))
        ));
    }
}

#[test]
fn changing_the_period_of_an_approved_submission_is_a_normative_rejection() {
    let repo = InMemoryPersistenceRepository::default();
    let original = validated("UNIT-A", "stable-id", 1);
    approve(&original, &repo);
    assert!(matches!(
        validated("UNIT-A", "stable-id", 2).preview(&repo),
        Err(SessionApprovalError::Rejected(
            SessionRejection::ApprovedSessionImmutable
        ))
    ));
    let mut v = model();
    v["flow_id"] = json!("OTHER-SYNTHETIC");
    let p = SchemaCompiler
        .compile_persistence_profile(&raw(v), &[])
        .unwrap();
    let params = parameters("UNIT-A", 1);
    let s = parts(&p, "UNIT-A", "stable-id", false);
    let result = outcome(&s, &params);
    let other = ApprovalDraft::new(&p, params, s, None)
        .unwrap()
        .validated(&result, vec![], &protector())
        .unwrap();
    assert!(matches!(
        other.preview(&repo),
        Err(SessionApprovalError::Persistence(
            PersistenceError::DuplicateSubmission
        ))
    ));
}

#[test]
fn nonempty_candidate_requires_declared_verified_identities_before_sqlite_sees_it() {
    use sha2::{Digest, Sha256};
    let mut v = model();
    v["persistence_policy"]["run_uniqueness"] = json!([{"trace":"T1","key":"record_key"}]);
    let source_raw = raw(v.clone());
    let p = SchemaCompiler
        .compile_persistence_profile(&source_raw, &[])
        .unwrap();
    // Key extraction uses the very same field declarations; stripping only the
    // non-executable persistence policy is confined to this component fixture,
    // never a production validation capability bypass.
    v["persistence_policy"] = Value::Null;
    v["required_capabilities"] = json!(["local.v1"]);
    let local = SchemaCompiler.compile(raw(v)).unwrap();
    let trace = local.trace(&TraceId("T1".into())).unwrap();
    let record = b"AB9xyz";
    let key = build_join_key(
        &RecordView::new(record),
        &trace.record_key().field_indexes,
        trace,
    )
    .unwrap();
    let params = parameters("UNIT-A", 1);
    let mut s = parts(&p, "UNIT-A", "nonempty", false);
    let first = s.traces.iter_mut().find(|t| t.trace_id.0 == "T1").unwrap();
    first.presence = TracePresence::Present { record_count: 1 };
    first.input_sha256 = Some(Sha256Digest(Sha256::digest(b"AB9xyz\n").into()));
    let mut result = outcome(&parts(&p, "UNIT-A", "nonempty", false), &params);
    result.summary.records_processed = 1;
    result.summary.valid_records = 1;
    let evidence = result
        .summary
        .full_validation
        .as_mut()
        .unwrap()
        .traces
        .iter_mut()
        .find(|t| t.trace_id.0 == "T1")
        .unwrap();
    evidence.records_processed = 1;
    evidence.record_keys_checked = 1;
    let input = result
        .summary
        .inputs
        .iter_mut()
        .find(|t| t.trace_id.0 == "T1")
        .unwrap();
    input.file_size_bytes = 7;
    input.bytes_read = 7;
    input.completion = InputAuditCompletion::Complete {
        sha256: s.traces[0].input_sha256.clone().unwrap(),
    };
    result.summary.validate().unwrap();
    let protection = protector();
    let scope = |flow: &str, trace: &str, kind: &str| {
        IdentityProtectionScope::new(
            s.namespace.clone(),
            FlowId(flow.into()),
            PersistenceSchemaScope::SchemaVersion {
                version: p.schema_key().version.clone(),
            },
            TraceId(trace.into()),
            kind.into(),
        )
        .unwrap()
    };
    for invalid in [
        scope("OTHER", "T1", "record_key"),
        scope("SYNTHETIC", "T2", "record_key"),
        scope("SYNTHETIC", "T1", "undeclared_key_kind"),
    ] {
        let row = protection
            .protect_projection(&invalid, &key, p.schema_hash(), &key)
            .unwrap();
        assert!(matches!(
            ApprovalDraft::new(&p, params.clone(), s.clone(), None)
                .unwrap()
                .validated(&result, vec![row], &protection),
            Err(SessionApprovalError::InvalidBinding)
        ));
    }
    let row = protection
        .protect_projection(
            &scope("SYNTHETIC", "T1", "record_key"),
            &key,
            p.schema_hash(),
            &key,
        )
        .unwrap();
    let unattested = StoredIdentity::new(row.identity().clone(), row.record_fingerprint().clone());
    assert!(matches!(
        ApprovalDraft::new(&p, params.clone(), s.clone(), None)
            .unwrap()
            .validated(&result, vec![unattested], &protection),
        Err(SessionApprovalError::Persistence(
            PersistenceError::Protection(_)
        ))
    ));
    let v = ApprovalDraft::new(&p, params, s, None)
        .unwrap()
        .validated(&result, vec![row.clone()], &protection)
        .unwrap();
    let temp =
        tempfile::tempdir_in(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target"))
            .unwrap();
    let root = temp.path().join("archive");
    let repo = SQLitePersistenceAdapter::create(&root, Some("synthetic-key"), &Provider).unwrap();
    approve(&v, &repo);
    drop(repo);
    let repo = SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider).unwrap();
    assert_eq!(
        repo.lookup_identity(v.source().namespace(), row.identity())
            .unwrap(),
        Some(row)
    );
    assert_eq!(
        repo.load_current_snapshot(v.source().namespace())
            .unwrap()
            .unwrap()
            .identities()
            .len(),
        1
    );
    assert!(
        !std::fs::read(root.join("lintra.db"))
            .unwrap()
            .windows(record.len())
            .any(|w| w == record)
    );
}

#[test]
fn foreign_clock_values_outside_chrono_range_are_rejected_without_panicking() {
    let p = profile();
    let params = parameters("UNIT-A", 1);
    let s = parts(&p, "UNIT-A", "clock", false);
    let mut result = outcome(&s, &params);
    // Unix can represent instants much farther than chrono; on platforms whose
    // SystemTime rejects this instant, that invalid value cannot enter the API.
    if let Some(extreme) =
        std::time::UNIX_EPOCH.checked_add(Duration::from_secs(9_000_000_000_000_000))
    {
        result.summary.started_at = extreme;
        assert!(matches!(
            ApprovalDraft::new(&p, params, s, None).unwrap().validated(
                &result,
                vec![],
                &protector()
            ),
            Err(SessionApprovalError::InvalidValidationEvidence)
        ));
    }
}

#[test]
fn fractional_pre_epoch_clock_is_compared_exactly_without_calendar_conversion() {
    let p = profile();
    let params = parameters("UNIT-A", 1);
    let mut s = parts(&p, "UNIT-A", "pre-epoch", false);
    s.created_at = Utc.timestamp_opt(-1, 500_000_000).unwrap();
    let mut result = outcome(&s, &params);
    assert!(
        ApprovalDraft::new(&p, params.clone(), s.clone(), None)
            .unwrap()
            .validated(&result, vec![], &protector())
            .is_ok()
    );
    result.summary.started_at = std::time::UNIX_EPOCH
        .checked_sub(Duration::from_millis(499))
        .unwrap();
    assert!(matches!(
        ApprovalDraft::new(&p, params, s, None)
            .unwrap()
            .validated(&result, vec![], &protector()),
        Err(SessionApprovalError::InvalidValidationEvidence)
    ));
}
