//! Synthetic audit-component fixtures, not claims of a historical validation
//! run. Public fixed test keys and protected canonical tuples only.
use chrono::{TimeZone, Utc};
use lintra::*;
use serde_json::{Value, json};
use std::{
    io,
    sync::{Arc, Mutex},
};

struct Provider(u8);
impl PersistenceKeyProvider for Provider {
    fn resolve(&self, id: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        if id != "synthetic-key" {
            return Err(ProtectionError::KeyUnavailable);
        }
        Ok(Arc::new(PersistenceKey::new(vec![self.0; 64])?))
    }
}
fn protector() -> IdentityProtector {
    IdentityProtector::preflight(Some("synthetic-key"), &Provider(0xaa), None).unwrap()
}
fn model(flow: &str) -> Value {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    value["flow_id"] = json!(flow);
    value["traces"][0]["fields"][0]["physical"]["length"] = json!(3);
    let mut field = value["traces"][0]["fields"][0].clone();
    field["id"] = json!("F2");
    field["physical"]["start"] = json!(4);
    value["traces"][0]["fields"]
        .as_array_mut()
        .unwrap()
        .push(field);
    let mut second = value["traces"][0].clone();
    second["id"] = json!("T2");
    value["traces"].as_array_mut().unwrap().push(second);
    value["parameters"] =
        json!([{"id":"unit","type":"string","allowed_values":["UNIT-A","UNIT-B"]}]);
    value["required_capabilities"] = json!(["local.v1", "persistence.audit.v1"]);
    value["persistence_policy"] = json!({
        "namespace_parameters":["unit"],"version_compatibility":{"kind":"same_schema_version"},
        "run_uniqueness":[],"cross_run_audit":[{"trace":"T2","key":"record_key"},{"trace":"T1","key":"record_key"}],
        "history_identity_invariants":[],"snapshot":null,"temporal_rules":[],
        "storage_protection":"external_secure_configuration_required"
    });
    value
}
fn raw(value: Value) -> RawFlowSchema {
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    raw
}
fn profile(flow: &str) -> CompiledPersistenceProfile {
    SchemaCompiler
        .compile_persistence_profile(&raw(model(flow)), &[])
        .unwrap()
}
fn namespace(unit: &str) -> PersistenceNamespace {
    PersistenceNamespace::new(vec![("unit".into(), NamespaceValue::String(unit.into()))]).unwrap()
}
fn scope(p: &CompiledPersistenceProfile) -> PersistenceSchemaScope {
    match &p.policy().unwrap().version_compatibility {
        VersionCompatibility::SameSchemaVersion => PersistenceSchemaScope::SchemaVersion {
            version: p.schema_key().version.clone(),
        },
        VersionCompatibility::DeclaredGroup { id, .. } => {
            PersistenceSchemaScope::CompatibilityGroup { id: id.to_string() }
        }
    }
}
fn row(
    p: &CompiledPersistenceProfile,
    protector: &IdentityProtector,
    unit: &str,
    trace: &str,
    key: &str,
    body: &str,
) -> StoredIdentity {
    row_defined(
        p,
        protector,
        unit,
        trace,
        key,
        body,
        &Sha256Digest([17; 32]),
    )
}
fn row_defined(
    p: &CompiledPersistenceProfile,
    protector: &IdentityProtector,
    unit: &str,
    trace: &str,
    key: &str,
    body: &str,
    definition: &Sha256Digest,
) -> StoredIdentity {
    let mut v = model(&p.schema_key().flow_id.0);
    v["persistence_policy"] = Value::Null;
    v["required_capabilities"] = json!(["local.v1"]);
    let compiled = SchemaCompiler.compile(raw(v)).unwrap();
    let bytes = format!("{key:3}{body:3}");
    let view = RecordView::new(bytes.as_bytes());
    let schema = &compiled.traces()[0];
    let pk = build_join_key(&view, &schema.record_key().field_indexes, schema).unwrap();
    let projection = build_join_key(&view, &[1], schema).unwrap();
    let scope = IdentityProtectionScope::new(
        namespace(unit),
        p.schema_key().flow_id.clone(),
        scope(p),
        TraceId(trace.into()),
        "record_key".into(),
    )
    .unwrap();
    protector
        .protect_projection(&scope, &pk, definition, &projection)
        .unwrap()
}
fn parts(
    p: &CompiledPersistenceProfile,
    unit: &str,
    id: &str,
    quarter: u32,
    rows: &[StoredIdentity],
    absent: &[&str],
) -> SourceSubmissionParts {
    SourceSubmissionParts {
        namespace: namespace(unit),
        submission_id: id.into(),
        schema_key: p.schema_key().clone(),
        schema_hash: p.schema_hash().clone(),
        period: Some(format!("2026-Q{quarter}")),
        created_at: Utc.with_ymd_and_hms(2026, quarter * 3, 1, 0, 0, 0).unwrap(),
        validator_version: "synthetic-audit-fixture".into(),
        validation_mode: ValidationMode::Full,
        validation_status: RunStatus::CompletedValid,
        reference_data_versions: vec![],
        traces: p
            .trace_ids()
            .iter()
            .map(|trace| {
                let missing = absent.contains(&trace.0.as_str());
                SubmissionTrace {
                    trace_id: trace.clone(),
                    presence: if missing {
                        TracePresence::Absent
                    } else {
                        TracePresence::Present {
                            record_count: rows
                                .iter()
                                .filter(|r| r.identity().trace_id() == trace)
                                .count() as u64,
                        }
                    },
                    input_sha256: if missing {
                        None
                    } else {
                        Some(Sha256Digest([quarter as u8; 32]))
                    },
                }
            })
            .collect(),
        fingerprint: Fingerprint::new(
            "synthetic-preparation".into(),
            "synthetic-key".into(),
            vec![1],
        )
        .unwrap(),
        state: SessionState::ApprovedImmutable,
    }
}
fn snapshot(
    p: &CompiledPersistenceProfile,
    key: &IdentityProtector,
    unit: &str,
    position: (u64, u32),
    id: &str,
    rows: Vec<StoredIdentity>,
    absent: &[&str],
) -> HistoricalSnapshot {
    let source = SourceSubmission::new(parts(p, unit, id, position.1, &rows, absent)).unwrap();
    HistoricalSnapshot::new(
        SnapshotRevision::new(position.0),
        key.protect_submission(&source).unwrap(),
        rows,
    )
    .unwrap()
}
fn publish(repo: &dyn PersistentContextRepository, snapshot: &HistoricalSnapshot) {
    let mut tx = repo
        .begin_transaction(
            snapshot.namespace(),
            repo.current_generation(snapshot.namespace()).unwrap(),
        )
        .unwrap();
    tx.stage_snapshot(snapshot).unwrap();
    tx.commit().unwrap();
}
fn collect(
    p: &CompiledPersistenceProfile,
    key: &IdentityProtector,
    repo: &dyn PersistentContextRepository,
    candidate: &HistoricalSnapshot,
) -> (HistoricalAuditSummary, Vec<HistoricalDiff>) {
    let service = HistoricalAuditService::new(p, key, repo);
    let token = CancellationToken::default();
    let prepared = service.prepare(candidate, &token).unwrap();
    let mut rows = Vec::new();
    let summary = prepared
        .write_diffs(&token, |row| {
            rows.push(row.clone());
            Ok(())
        })
        .unwrap();
    (summary, rows)
}
#[test]
fn multiple_quarters_classify_every_change_without_publishing_the_preview() {
    let p = profile("FLOW-A");
    let key = protector();
    let repo = InMemoryPersistenceRepository::default();
    let a = row(&p, &key, "UNIT-A", "T1", "001", "AAA");
    let b = row(&p, &key, "UNIT-A", "T1", "002", "BBB");
    let c = row(&p, &key, "UNIT-A", "T1", "003", "CCC");
    let d = row(&p, &key, "UNIT-A", "T1", "004", "DDD");
    let q1 = snapshot(
        &p,
        &key,
        "UNIT-A",
        (1, 1),
        "q1",
        vec![a.clone(), b.clone(), c],
        &[],
    );
    let first = collect(&p, &key, &repo, &q1).0;
    assert_eq!(first.counts.inserted, 3);
    assert_eq!(first.metadata.namespace_generation, None);
    publish(&repo, &q1);
    let changed = row(&p, &key, "UNIT-A", "T1", "003", "NEW");
    let q2 = snapshot(
        &p,
        &key,
        "UNIT-A",
        (2, 2),
        "q2",
        vec![d.clone(), changed.clone(), b.clone()],
        &[],
    );
    let (report, rows) = collect(&p, &key, &repo, &q2);
    assert_eq!(
        report.counts,
        HistoricalAuditCounts {
            inserted: 1,
            updated: 1,
            disappeared: 1,
            reactivated: 0,
            unchanged: 1
        }
    );
    assert_eq!(
        repo.current_generation(&namespace("UNIT-A")).unwrap(),
        Some(SnapshotRevision::new(1))
    );
    assert!(
        repo.load_submission(&namespace("UNIT-A"), "q2")
            .unwrap()
            .is_none()
    );
    assert!(
        rows.iter()
            .all(|r| r.base_revision() == Some(SnapshotRevision::new(1)))
    );
    publish(&repo, &q2);
    let q3 = snapshot(
        &p,
        &key,
        "UNIT-A",
        (3, 3),
        "q3",
        vec![a, b, changed, d],
        &[],
    );
    assert_eq!(
        collect(&p, &key, &repo, &q3).0.counts,
        HistoricalAuditCounts {
            reactivated: 1,
            unchanged: 3,
            ..Default::default()
        }
    );
}
#[test]
fn absent_files_preserve_per_trace_baselines_across_other_flows() {
    let p = profile("FLOW-A");
    let other = profile("FLOW-B");
    let key = protector();
    let repo = InMemoryPersistenceRepository::default();
    let a = row(&p, &key, "UNIT-A", "T1", "001", "AAA");
    let b = row(&p, &key, "UNIT-A", "T2", "001", "BBB");
    publish(
        &repo,
        &snapshot(
            &p,
            &key,
            "UNIT-A",
            (1, 1),
            "q1",
            vec![a.clone(), b.clone()],
            &[],
        ),
    );
    let q2 = snapshot(
        &p,
        &key,
        "UNIT-A",
        (2, 2),
        "q2",
        vec![a.clone(), b.clone()],
        &["T1"],
    );
    let report = collect(&p, &key, &repo, &q2).0;
    assert_eq!(report.counts.disappeared, 0);
    assert_eq!(report.counts.unchanged, 1);
    publish(&repo, &q2);
    let foreign = snapshot(
        &other,
        &key,
        "UNIT-A",
        (3, 2),
        "foreign-flow",
        vec![row(&other, &key, "UNIT-A", "T1", "001", "ZZZ")],
        &[],
    );
    assert_eq!(collect(&other, &key, &repo, &foreign).0.counts.inserted, 1);
    publish(&repo, &foreign);
    let q3 = snapshot(&p, &key, "UNIT-A", (4, 3), "q3", vec![a, b], &[]);
    let report = collect(&p, &key, &repo, &q3).0;
    assert_eq!(
        report.counts,
        HistoricalAuditCounts {
            unchanged: 2,
            ..Default::default()
        }
    );
    assert_eq!(
        report.metadata.namespace_generation,
        Some(SnapshotRevision::new(3))
    );
    assert_eq!(
        report
            .metadata
            .traces
            .iter()
            .map(|t| t.revision.unwrap().value())
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
}
#[test]
fn present_empty_disappears_and_later_reactivation_survives_real_sqlite_restart() {
    let p = profile("FLOW-A");
    let key = protector();
    let parent =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/history-audit-tests");
    std::fs::create_dir_all(&parent).unwrap();
    let temporary = tempfile::tempdir_in(parent).unwrap();
    let root = temporary.path().join("archive");
    let repo =
        SQLitePersistenceAdapter::create(&root, Some("synthetic-key"), &Provider(0xaa)).unwrap();
    let row = row(&p, &key, "UNIT-A", "T1", "001", "AAA");
    publish(
        &repo,
        &snapshot(&p, &key, "UNIT-A", (1, 1), "q1", vec![row.clone()], &[]),
    );
    let empty = snapshot(&p, &key, "UNIT-A", (2, 2), "q2", vec![], &[]);
    assert_eq!(collect(&p, &key, &repo, &empty).0.counts.disappeared, 1);
    publish(&repo, &empty);
    drop(repo);
    let repo =
        SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider(0xaa)).unwrap();
    let occurrence = repo
        .lookup_identity_occurrence(&namespace("UNIT-A"), row.identity())
        .unwrap()
        .unwrap();
    assert_eq!(occurrence.revision(), SnapshotRevision::new(1));
    assert_eq!(occurrence.source().submission_id(), "q1");
    let q3 = snapshot(&p, &key, "UNIT-A", (3, 3), "q3", vec![row], &[]);
    assert_eq!(collect(&p, &key, &repo, &q3).0.counts.reactivated, 1);
    assert_eq!(
        repo.current_generation(&namespace("UNIT-A")).unwrap(),
        Some(SnapshotRevision::new(2))
    );
}
#[test]
fn order_is_deterministic_for_reversed_rows_and_policy_trace_order() {
    let p = profile("FLOW-A");
    let key = protector();
    let repo = InMemoryPersistenceRepository::default();
    let rows = vec![
        row(&p, &key, "UNIT-A", "T2", "003", "CCC"),
        row(&p, &key, "UNIT-A", "T1", "002", "BBB"),
        row(&p, &key, "UNIT-A", "T1", "001", "AAA"),
    ];
    let first = snapshot(&p, &key, "UNIT-A", (1, 1), "candidate", rows.clone(), &[]);
    let second = snapshot(
        &p,
        &key,
        "UNIT-A",
        (1, 1),
        "candidate",
        rows.into_iter().rev().collect(),
        &[],
    );
    let a = collect(&p, &key, &repo, &first);
    let b = collect(&p, &key, &repo, &second);
    assert_eq!(
        serde_json::to_vec(&a.1).unwrap(),
        serde_json::to_vec(&b.1).unwrap()
    );
    assert_eq!(
        a.1.iter()
            .map(|r| r.identity().trace_id().0.as_str())
            .collect::<Vec<_>>(),
        vec!["T1", "T1", "T2"]
    );
    assert_eq!(a.0.metadata.format_version, 1);
    assert_eq!(
        serde_json::to_value(a.0.metadata).unwrap()["coverage"],
        "local_repository_only"
    );
}
#[test]
fn namespace_isolation_and_wrong_key_never_look_like_new_local_history() {
    let p = profile("FLOW-A");
    let key = protector();
    let repo = InMemoryPersistenceRepository::default();
    let a = row(&p, &key, "UNIT-A", "T1", "001", "AAA");
    let b = row(&p, &key, "UNIT-B", "T1", "001", "AAA");
    assert_ne!(a.identity(), b.identity());
    publish(
        &repo,
        &snapshot(&p, &key, "UNIT-A", (1, 1), "q1", vec![a], &[]),
    );
    let candidate = snapshot(&p, &key, "UNIT-B", (1, 1), "q1-b", vec![b], &[]);
    assert_eq!(collect(&p, &key, &repo, &candidate).0.counts.inserted, 1);
    let wrong = IdentityProtector::preflight(Some("synthetic-key"), &Provider(0xbb), None).unwrap();
    assert!(matches!(
        HistoricalAuditService::new(&p, &wrong, &repo)
            .prepare(&candidate, &CancellationToken::default()),
        Err(HistoricalAuditError::Protection(_))
    ));
}
#[test]
fn incompatible_version_and_source_hash_fail_explicitly() {
    let p = profile("FLOW-A");
    let key = protector();
    let repo = InMemoryPersistenceRepository::default();
    let mut v = model("FLOW-A");
    v["schema_version"] = json!("v2");
    let old = SchemaCompiler
        .compile_persistence_profile(&raw(v), &[])
        .unwrap();
    publish(
        &repo,
        &snapshot(&old, &key, "UNIT-A", (1, 1), "old", vec![], &[]),
    );
    let candidate = snapshot(&p, &key, "UNIT-A", (2, 2), "new", vec![], &[]);
    assert!(matches!(
        HistoricalAuditService::new(&p, &key, &repo)
            .prepare(&candidate, &CancellationToken::default()),
        Err(HistoricalAuditError::IncompatibleSchema)
    ));
    let mut parts = parts(&p, "UNIT-B", "wrong-hash", 1, &[], &[]);
    parts.schema_hash = Sha256Digest([9; 32]);
    let bad = HistoricalSnapshot::new(
        SnapshotRevision::new(1),
        key.protect_submission(&SourceSubmission::new(parts).unwrap())
            .unwrap(),
        vec![],
    )
    .unwrap();
    assert!(matches!(
        HistoricalAuditService::new(&p, &key, &repo).prepare(&bad, &CancellationToken::default()),
        Err(HistoricalAuditError::IncompatibleSchema)
    ));
}
#[test]
fn declared_group_uses_compiler_verified_member_hashes() {
    let key = protector();
    let mut v2 = model("FLOW-A");
    v2["schema_version"] = json!("v2");
    let old = SchemaCompiler
        .compile_persistence_profile(&raw(v2.clone()), &[])
        .unwrap();
    let mut v1 = model("FLOW-A");
    v1["persistence_policy"]["version_compatibility"] =
        json!({"kind":"declared_group","id":"fixture-group","schema_versions":["v1","v2"]});
    let p = SchemaCompiler
        .compile_persistence_profile(&raw(v1), &[raw(v2)])
        .unwrap();
    let repo = InMemoryPersistenceRepository::default();
    let record = row(&p, &key, "UNIT-A", "T1", "001", "AAA");
    publish(
        &repo,
        &snapshot(
            &old,
            &key,
            "UNIT-A",
            (1, 1),
            "old",
            vec![record.clone()],
            &[],
        ),
    );
    let candidate = snapshot(&p, &key, "UNIT-A", (2, 2), "new", vec![record], &[]);
    assert_eq!(collect(&p, &key, &repo, &candidate).0.counts.unchanged, 1);
}
#[test]
fn unattested_rows_and_changed_projection_definition_are_rejected() {
    let p = profile("FLOW-A");
    let key = protector();
    let repo = InMemoryPersistenceRepository::default();
    let record = row(&p, &key, "UNIT-A", "T1", "001", "AAA");
    let unattested = StoredIdentity::new(
        record.identity().clone(),
        record.record_fingerprint().clone(),
    );
    let bad = snapshot(&p, &key, "UNIT-A", (1, 1), "bad", vec![unattested], &[]);
    assert!(matches!(
        HistoricalAuditService::new(&p, &key, &repo).prepare(&bad, &CancellationToken::default()),
        Err(HistoricalAuditError::Protection(_))
    ));
    publish(
        &repo,
        &snapshot(&p, &key, "UNIT-A", (1, 1), "base", vec![record], &[]),
    );
    let changed = row_defined(
        &p,
        &key,
        "UNIT-A",
        "T1",
        "001",
        "AAA",
        &Sha256Digest([18; 32]),
    );
    let candidate = snapshot(&p, &key, "UNIT-A", (2, 2), "new", vec![changed], &[]);
    let service = HistoricalAuditService::new(&p, &key, &repo);
    let token = CancellationToken::default();
    assert!(matches!(
        service
            .prepare(&candidate, &token)
            .unwrap()
            .write_diffs(&token, |_| Ok(())),
        Err(HistoricalAuditError::IncompatibleProjection)
    ));
}
#[test]
fn cancellation_sink_failure_and_generation_changes_never_return_complete_summary() {
    let p = profile("FLOW-A");
    let key = protector();
    let repo = InMemoryPersistenceRepository::default();
    let candidate = snapshot(
        &p,
        &key,
        "UNIT-A",
        (1, 1),
        "candidate",
        vec![
            row(&p, &key, "UNIT-A", "T1", "001", "AAA"),
            row(&p, &key, "UNIT-A", "T1", "002", "BBB"),
        ],
        &[],
    );
    let service = HistoricalAuditService::new(&p, &key, &repo);
    let token = CancellationToken::default();
    let prepared = service.prepare(&candidate, &token).unwrap();
    assert!(matches!(
        prepared.write_diffs(&token, |_| Err(ReportError::Write(io::Error::other(
            "synthetic sink failure"
        )))),
        Err(HistoricalAuditError::Report(_))
    ));
    let cancelled = CancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        service.prepare(&candidate, &cancelled),
        Err(HistoricalAuditError::Cancelled)
    ));
    assert!(matches!(
        prepared.write_diffs(&token, |_| {
            token.cancel();
            Ok(())
        }),
        Err(HistoricalAuditError::Cancelled)
    ));
    let token = CancellationToken::default();
    assert!(matches!(
        prepared.write_diffs(&token, |_| {
            publish(&repo, &candidate);
            Ok(())
        }),
        Err(HistoricalAuditError::Persistence(
            PersistenceError::GenerationConflict { .. }
        ))
    ));
}

/// Fault adapter delegates genuine storage facts and injects one concurrent
/// publication during occurrence lookup; it is never a performance substitute.
struct RacingRepository {
    inner: InMemoryPersistenceRepository,
    pending: Mutex<Option<HistoricalSnapshot>>,
    unsupported: bool,
}
impl PersistentContextRepository for RacingRepository {
    fn lookup_identity_occurrence(
        &self,
        n: &PersistenceNamespace,
        i: &PersistentRecordIdentity,
    ) -> Result<Option<HistoricalIdentityOccurrence>, PersistenceError> {
        if self.unsupported {
            return Err(PersistenceError::StorageUnavailable);
        }
        let result = self.inner.lookup_identity_occurrence(n, i)?;
        if let Some(snapshot) = self.pending.lock().unwrap().take() {
            publish(&self.inner, &snapshot);
        }
        Ok(result)
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
    fn load_snapshot(
        &self,
        n: &PersistenceNamespace,
        r: SnapshotRevision,
    ) -> Result<Option<HistoricalSnapshot>, PersistenceError> {
        self.inner.load_snapshot(n, r)
    }
    fn load_submission(
        &self,
        n: &PersistenceNamespace,
        s: &str,
    ) -> Result<Option<SourceSubmission>, PersistenceError> {
        self.inner.load_submission(n, s)
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
        self.inner.load_submission_page(n, a)
    }
    fn begin_transaction(
        &self,
        n: &PersistenceNamespace,
        r: Option<SnapshotRevision>,
    ) -> Result<Box<dyn PersistenceTransaction>, PersistenceError> {
        self.inner.begin_transaction(n, r)
    }
}
#[test]
fn concurrent_occurrence_lookup_and_unavailable_provenance_are_not_insertions() {
    let p = profile("FLOW-A");
    let key = protector();
    let candidate = snapshot(
        &p,
        &key,
        "UNIT-A",
        (1, 1),
        "candidate",
        vec![row(&p, &key, "UNIT-A", "T1", "001", "AAA")],
        &[],
    );
    for unavailable in [false, true] {
        let repo = RacingRepository {
            inner: InMemoryPersistenceRepository::default(),
            pending: Mutex::new(Some(candidate.clone())),
            unsupported: unavailable,
        };
        let service = HistoricalAuditService::new(&p, &key, &repo);
        let token = CancellationToken::default();
        let prepared = service.prepare(&candidate, &token).unwrap();
        let mut emitted = 0;
        let result = prepared.write_diffs(&token, |_| {
            emitted += 1;
            Ok(())
        });
        assert_eq!(emitted, 0);
        if unavailable {
            assert!(matches!(
                result,
                Err(HistoricalAuditError::Persistence(
                    PersistenceError::StorageUnavailable
                ))
            ));
        } else {
            assert!(matches!(
                result,
                Err(HistoricalAuditError::Persistence(
                    PersistenceError::GenerationConflict { .. }
                ))
            ));
        }
    }
}

#[test]
fn metadata_cursor_covers_more_than_one_page_without_loading_all_snapshots() {
    let p = profile("FLOW-A");
    let key = protector();
    let repo = InMemoryPersistenceRepository::default();
    // Empty component snapshots isolate metadata paging, not storage performance.
    for revision in 1..=130 {
        publish(
            &repo,
            &snapshot(
                &p,
                &key,
                "UNIT-A",
                (revision, 1),
                &format!("synthetic-{revision}"),
                vec![],
                &[],
            ),
        );
    }
    let candidate = snapshot(&p, &key, "UNIT-A", (131, 2), "candidate", vec![], &[]);
    let (report, rows) = collect(&p, &key, &repo, &candidate);
    assert!(rows.is_empty());
    assert_eq!(
        report.metadata.namespace_generation,
        Some(SnapshotRevision::new(130))
    );
    assert!(
        report
            .metadata
            .traces
            .iter()
            .all(|t| t.revision == Some(SnapshotRevision::new(130)))
    );
    assert_eq!(
        repo.current_generation(&namespace("UNIT-A")).unwrap(),
        Some(SnapshotRevision::new(130))
    );
}

#[test]
fn undeclared_audit_invalid_namespace_and_unknown_trace_are_explicit_errors() {
    let p = profile("FLOW-A");
    let key = protector();
    let repo = InMemoryPersistenceRepository::default();
    let token = CancellationToken::default();
    let candidate = snapshot(&p, &key, "UNIT-A", (1, 1), "candidate", vec![], &[]);
    let mut model = model("FLOW-A");
    model["persistence_policy"]["cross_run_audit"] = json!([]);
    let disabled = SchemaCompiler
        .compile_persistence_profile(&raw(model), &[])
        .unwrap();
    assert!(matches!(
        HistoricalAuditService::new(&disabled, &key, &repo).prepare(&candidate, &token),
        Err(HistoricalAuditError::AuditNotDeclared)
    ));
    let wrong_namespace = snapshot(
        &p,
        &key,
        "SYNTHETIC-UNDECLARED",
        (1, 1),
        "bad-namespace",
        vec![],
        &[],
    );
    let service = HistoricalAuditService::new(&p, &key, &repo);
    let result = service.prepare(&wrong_namespace, &token);
    assert!(matches!(result, Err(HistoricalAuditError::InvalidBinding)));
    let record = row(&p, &key, "UNIT-A", "T3", "001", "AAA");
    let bad = snapshot(
        &p,
        &key,
        "UNIT-A",
        (1, 1),
        "unknown-trace",
        vec![record],
        &["T1"],
    );
    assert!(matches!(
        HistoricalAuditService::new(&p, &key, &repo).prepare(&bad, &token),
        Err(HistoricalAuditError::InvalidBinding)
    ));
    let message = HistoricalAuditError::InvalidBinding.to_string();
    assert!(!message.contains("SYNTHETIC-UNDECLARED"));
}

#[test]
fn retained_absent_trace_rows_are_valid_history_without_inventing_presence() {
    let p = profile("FLOW-A");
    let key = protector();
    let parent =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/history-audit-tests");
    std::fs::create_dir_all(&parent).unwrap();
    let temporary = tempfile::tempdir_in(parent).unwrap();
    let root = temporary.path().join("archive");
    let repo =
        SQLitePersistenceAdapter::create(&root, Some("synthetic-key"), &Provider(0xaa)).unwrap();
    let record = row(&p, &key, "UNIT-A", "T1", "001", "AAA");
    publish(
        &repo,
        &snapshot(&p, &key, "UNIT-A", (1, 1), "q1", vec![record.clone()], &[]),
    );
    publish(
        &repo,
        &snapshot(&p, &key, "UNIT-A", (2, 2), "q2", vec![], &[]),
    );
    let retained = snapshot(
        &p,
        &key,
        "UNIT-A",
        (3, 3),
        "q3-retained",
        vec![record.clone()],
        &["T1"],
    );
    assert_eq!(
        collect(&p, &key, &repo, &retained).0.counts,
        HistoricalAuditCounts::default()
    );
    publish(&repo, &retained);
    drop(repo);
    let repo =
        SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider(0xaa)).unwrap();
    let occurrence = repo
        .lookup_identity_occurrence(&namespace("UNIT-A"), record.identity())
        .unwrap()
        .unwrap();
    assert_eq!(occurrence.revision(), SnapshotRevision::new(3));
    assert!(
        occurrence
            .source()
            .traces()
            .iter()
            .any(|t| t.trace_id.0 == "T1" && t.presence == TracePresence::Absent)
    );
    let candidate = snapshot(&p, &key, "UNIT-A", (4, 4), "q4", vec![record], &[]);
    let report = collect(&p, &key, &repo, &candidate).0;
    assert_eq!(report.counts.reactivated, 1);
    assert_eq!(
        report.metadata.traces[0].revision,
        Some(SnapshotRevision::new(2))
    );
    assert_eq!(
        repo.current_generation(&namespace("UNIT-A")).unwrap(),
        Some(SnapshotRevision::new(3))
    );
}

#[test]
fn retained_key_without_a_present_baseline_is_not_reported_as_new_or_reactivated() {
    let p = profile("FLOW-A");
    let key = protector();
    let repo = InMemoryPersistenceRepository::default();
    let record = row(&p, &key, "UNIT-A", "T1", "001", "AAA");
    publish(
        &repo,
        &snapshot(
            &p,
            &key,
            "UNIT-A",
            (1, 1),
            "only-retained",
            vec![record.clone()],
            &["T1"],
        ),
    );
    let candidate = snapshot(&p, &key, "UNIT-A", (2, 2), "candidate", vec![record], &[]);
    let service = HistoricalAuditService::new(&p, &key, &repo);
    let token = CancellationToken::default();
    let prepared = service.prepare(&candidate, &token).unwrap();
    let mut emitted = 0;
    assert!(matches!(
        prepared.write_diffs(&token, |_| {
            emitted += 1;
            Ok(())
        }),
        Err(HistoricalAuditError::NoComparableBaseline)
    ));
    assert_eq!(emitted, 0);
}
