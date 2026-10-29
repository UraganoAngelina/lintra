//! Frozen storage1 source/proof created by product3712ed6, not by the new codec.
use super::*;
use serde_json::{Value, json};
struct LegacyProvider;
impl PersistenceKeyProvider for LegacyProvider {
    fn resolve(&self, id: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        if id != "legacy-fixture-key" {
            return Err(ProtectionError::KeyUnavailable);
        }
        Ok(Arc::new(PersistenceKey::new(vec![0xaa; 64])?))
    }
}
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../../tests/fixtures/persistence/legacy-storage-v1.json"
    ))
    .unwrap()
}
fn hex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn legacy_namespace() -> PersistenceNamespace {
    let source: Value = serde_json::from_str(fixture()["source_utf8"].as_str().unwrap()).unwrap();
    serde_json::from_value(source["namespace"].clone()).unwrap()
}
fn legacy_open(root: &Path) -> SQLitePersistenceAdapter {
    SQLitePersistenceAdapter::open(root, Some("legacy-fixture-key"), &LegacyProvider).unwrap()
}
fn seed_legacy(root: &Path) {
    let raw = fixture();
    let guarded = security::root(root, true).unwrap();
    security::check_files(root, true).unwrap();
    let _database = security::database(root, true).unwrap();
    let mut c = Connection::open(guarded.path.join("lintra.db")).unwrap();
    configure(&c, true).unwrap();
    let source = raw["source_utf8"].as_str().unwrap();
    let body: Value = serde_json::from_str(source).unwrap();
    let ns = serde_json::to_vec(&body["namespace"]).unwrap();
    let tx = c
        .transaction_with_behavior(TransactionBehavior::Exclusive)
        .unwrap();
    tx.execute_batch(SCHEMA).unwrap();
    tx.pragma_update(None, "application_id", APPLICATION)
        .unwrap();
    tx.pragma_update(None, "user_version", 1).unwrap();
    tx.execute(
        "INSERT INTO storage_meta VALUES(1,?1)",
        [raw["protection_audit_utf8"].as_str().unwrap().as_bytes()],
    )
    .unwrap();
    tx.execute("INSERT INTO namespaces VALUES(?1,1)", [&ns])
        .unwrap();
    tx.execute(
        "INSERT INTO snapshots VALUES(?1,1,?2,?3,?4,0)",
        params![
            ns,
            body["submission_id"].as_str().unwrap(),
            source.as_bytes(),
            hex(raw["source_proof_hex"].as_str().unwrap())
        ],
    )
    .unwrap();
    tx.commit().unwrap();
}
fn next(repo: &SQLitePersistenceAdapter) -> HistoricalSnapshot {
    let mut p: SourceSubmissionParts = serde_json::from_slice(include_bytes!(
        "../../../../tests/fixtures/persistence/source-submission.json"
    ))
    .unwrap();
    p.submission_id = "synthetic-metadata-2".into();
    p.state = SessionState::Validated;
    p.period = Some("2026-Q2".into());
    p.created_at = Utc.with_ymd_and_hms(2026, 4, 2, 3, 4, 5).unwrap();
    p.schema_hash = Sha256Digest([7; 32]);
    p.traces.push(SubmissionTrace {
        trace_id: TraceId("T3".into()),
        presence: TracePresence::Present { record_count: 2 },
        input_sha256: Some(Sha256Digest([21; 32])),
    });
    let consent = MissingTraceProceedConsent::new(
        vec![TraceId("T2".into())],
        Utc.with_ymd_and_hms(2026, 4, 2, 4, 0, 0).unwrap(),
    )
    .unwrap();
    let source = SourceSubmission::new_with_missing_trace_consent(p, consent).unwrap();
    let source = repo.protector().protect_submission(&source).unwrap();
    HistoricalSnapshot::new(SnapshotRevision::new(2), source, vec![]).unwrap()
}
fn legacy_blobs(repo: &SQLitePersistenceAdapter) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let c = repo.lock().unwrap();
    let (a, b) = c
        .query_row(
            "SELECT source,proof FROM snapshots WHERE revision=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    let audit = c
        .query_row("SELECT audit FROM storage_meta", [], |r| r.get(0))
        .unwrap();
    (a, b, audit)
}
#[test]
fn legacy_signed_metadata_stays_readable_and_upgrade_is_explicit_atomic_and_idempotent() {
    let parent = parent();
    let root = parent.path().join("archive");
    seed_legacy(&root);
    let repo = legacy_open(&root);
    assert_eq!(repo.settings().unwrap().format_version, 1);
    let ns = legacy_namespace();
    let old = repo.load_current_snapshot(&ns).unwrap().unwrap();
    assert!(old.source().missing_trace_consent().is_none());
    assert_eq!(old.source().state(), SessionState::ApprovedImmutable);
    assert_eq!(
        codec::encode(old.source()).unwrap(),
        fixture()["source_utf8"].as_str().unwrap().as_bytes()
    );
    let blobs = legacy_blobs(&repo);
    let candidate = next(&repo);
    assert_eq!(
        publish(&repo, &candidate).unwrap_err(),
        PersistenceError::StorageUpgradeRequired {
            found: 1,
            required: 2
        }
    );
    assert_eq!(repo.settings().unwrap().format_version, 1);
    assert_eq!(repo.load_current_snapshot(&ns).unwrap(), Some(old.clone()));
    // A real second SQLite connection holds the writer reservation. Upgrade must
    // fail without altering the format or any historical byte.
    let mut blocker = Connection::open(root.join("lintra.db")).unwrap();
    let held = blocker
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    assert_eq!(
        repo.upgrade_storage_v1_to_v2().unwrap_err(),
        PersistenceError::StorageBusy
    );
    assert_eq!(repo.settings().unwrap().format_version, 1);
    drop(held);
    drop(blocker);
    repo.upgrade_storage_v1_to_v2().unwrap();
    repo.upgrade_storage_v1_to_v2().unwrap();
    assert_eq!(repo.settings().unwrap().format_version, 2);
    assert_eq!(legacy_blobs(&repo), blobs);
    assert_eq!(
        repo.load_snapshot(&ns, SnapshotRevision::new(1)).unwrap(),
        Some(old.clone())
    );
    publish(&repo, &candidate).unwrap();
    drop(repo);
    let repo = legacy_open(&root);
    assert_eq!(repo.load_current_snapshot(&ns).unwrap(), Some(candidate));
    assert_eq!(
        repo.load_snapshot(&ns, SnapshotRevision::new(1)).unwrap(),
        Some(old)
    );
    drop(repo);
    let status = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "persistence::sqlite::tests::metadata::metadata_restart_worker",
            "--ignored",
            "--nocapture",
        ])
        .env("LINTRA_METADATA_TEST_ROOT", &root)
        .status()
        .unwrap();
    assert!(status.success());
}
#[test]
#[ignore = "synthetic subprocess helper, invoked by metadata parent"]
fn metadata_restart_worker() {
    let root = PathBuf::from(std::env::var_os("LINTRA_METADATA_TEST_ROOT").unwrap());
    let repo = legacy_open(&root);
    if let Ok(mode) = std::env::var("LINTRA_METADATA_UPGRADE_CRASH") {
        *repo.inner.fault.lock().unwrap() = Some(if mode == "during" {
            TestFault::DuringExit
        } else {
            TestFault::AfterExit
        });
        repo.upgrade_storage_v1_to_v2().unwrap();
        panic!("crash hook did not execute");
    }
    let current = repo
        .load_current_snapshot(&legacy_namespace())
        .unwrap()
        .unwrap();
    let source = current.source();
    assert_eq!(current.revision(), SnapshotRevision::new(2));
    assert_eq!(source.period(), Some("2026-Q2"));
    assert_eq!(source.schema_hash(), &Sha256Digest([7; 32]));
    assert_eq!(source.state(), SessionState::Validated);
    assert_eq!(source.reference_data_versions().len(), 2);
    assert_eq!(
        *source.created_at(),
        Utc.with_ymd_and_hms(2026, 4, 2, 3, 4, 5).unwrap()
    );
    let traces = source.traces();
    assert_eq!(
        traces[0].presence,
        TracePresence::Present { record_count: 0 }
    );
    assert_eq!(traces[1].presence, TracePresence::Absent);
    assert_eq!(
        traces[2].presence,
        TracePresence::Present { record_count: 2 }
    );
    assert!(traces[0].input_sha256.is_some());
    assert!(traces[1].input_sha256.is_none());
    assert_eq!(traces[2].input_sha256, Some(Sha256Digest([21; 32])));
    assert_eq!(
        source.missing_trace_consent().unwrap().traces(),
        &[TraceId("T2".into())]
    );
    assert!(
        repo.load_snapshot(&legacy_namespace(), SnapshotRevision::new(1))
            .unwrap()
            .unwrap()
            .source()
            .missing_trace_consent()
            .is_none()
    );
}
#[test]
fn consent_is_authenticated_and_tampering_never_turns_into_valid_metadata() {
    let parent = parent();
    let root = parent.path().join("archive");
    seed_legacy(&root);
    let repo = legacy_open(&root);
    repo.upgrade_storage_v1_to_v2().unwrap();
    let candidate = next(&repo);
    publish(&repo, &candidate).unwrap();
    let mut source = serde_json::to_value(candidate.source()).unwrap();
    source["missing_trace_consent"]["recorded_at"] = json!("2026-04-02T05:00:00.000000000Z");
    repo.lock()
        .unwrap()
        .execute(
            "UPDATE snapshots SET source=?1 WHERE revision=2",
            [serde_json::to_vec(&source).unwrap()],
        )
        .unwrap();
    assert_eq!(
        repo.load_current_snapshot(&legacy_namespace()).unwrap_err(),
        PersistenceError::CorruptStorage
    );
}
#[test]
fn upgrade_preserves_all_identity_payloads_indices_and_generations() {
    let parent = parent();
    let root = parent.path().join("archive");
    let repo = create(&root);
    let row = row(repo.protector(), "A", 5, "A");
    let old = snapshot(&repo, "A", 1, "legacy-compatible", vec![row.clone()]);
    publish(&repo, &old).unwrap();
    repo.lock()
        .unwrap()
        .pragma_update(None, "user_version", 1)
        .unwrap();
    let before: (Vec<u8>, Vec<u8>, Vec<u8>) = repo
        .lock()
        .unwrap()
        .query_row(
            "SELECT payload,key_proof,record_proof FROM records",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    repo.upgrade_storage_v1_to_v2().unwrap();
    let after: (Vec<u8>, Vec<u8>, Vec<u8>) = repo
        .lock()
        .unwrap()
        .query_row(
            "SELECT payload,key_proof,record_proof FROM records",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(before, after);
    assert_eq!(
        repo.load_current_snapshot(&namespace("A")).unwrap(),
        Some(old)
    );
    assert_eq!(
        repo.lookup_identity(&namespace("A"), row.identity())
            .unwrap(),
        Some(row)
    );
}

#[test]
fn upgrade_failure_before_commit_rolls_back_and_lost_ack_is_not_false_rollback() {
    for fault in [
        TestFault::Before,
        TestFault::RejectCommit,
        TestFault::AfterLost,
    ] {
        let parent = parent();
        let root = parent.path().join("archive");
        seed_legacy(&root);
        let repo = legacy_open(&root);
        let old = legacy_blobs(&repo);
        let published = matches!(fault, TestFault::AfterLost);
        let before = matches!(fault, TestFault::Before);
        *repo.inner.fault.lock().unwrap() = Some(fault);
        assert_eq!(
            repo.upgrade_storage_v1_to_v2().unwrap_err(),
            if before {
                PersistenceError::StorageUnavailable
            } else {
                PersistenceError::StorageUpgradeOutcomeUnknown
            }
        );
        repo.lock()
            .unwrap()
            .commit_hook(None::<fn() -> bool>)
            .unwrap();
        drop(repo);
        let repo = legacy_open(&root);
        assert_eq!(
            repo.settings().unwrap().format_version,
            if published { 2 } else { 1 }
        );
        assert_eq!(legacy_blobs(&repo), old);
    }
}
#[test]
fn upgrade_process_interruption_recovers_the_old_or_committed_format_with_all_metadata() {
    for mode in ["during", "after"] {
        let parent = parent();
        let root = parent.path().join("archive");
        seed_legacy(&root);
        let repo = legacy_open(&root);
        let before = legacy_blobs(&repo);
        drop(repo);
        let status = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "persistence::sqlite::tests::metadata::metadata_restart_worker",
                "--ignored",
                "--nocapture",
            ])
            .env("LINTRA_METADATA_TEST_ROOT", &root)
            .env("LINTRA_METADATA_UPGRADE_CRASH", mode)
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(if mode == "during" { 86 } else { 87 }));
        let repo = legacy_open(&root);
        assert_eq!(
            repo.settings().unwrap().format_version,
            if mode == "during" { 1 } else { 2 }
        );
        assert_eq!(legacy_blobs(&repo), before);
        assert!(
            repo.load_current_snapshot(&legacy_namespace())
                .unwrap()
                .is_some()
        );
    }
}
