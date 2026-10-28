//! Only synthetic records and public, fixed test key material. Subprocesses run
//! this test executable, so crash/restart exercises the real SQLite adapter.
use super::*;
use crate::*;
use chrono::{TimeZone, Utc};
use std::{
    fs,
    path::PathBuf,
    process::{Child, Command},
    time::{Duration, Instant},
};

pub(super) enum TestFault {
    Before,
    RejectCommit,
    DuringExit,
    AfterLost,
    AfterExit,
}
impl SQLitePersistenceAdapter {
    pub(super) fn configure_commit_fault(&self, c: &mut Connection) {
        let mut fault = self.inner.fault.lock().unwrap();
        match fault.as_ref() {
            Some(TestFault::RejectCommit) => {
                c.commit_hook(Some(|| true)).unwrap();
                fault.take();
            }
            // Exit in SQLite's hook without running Rust destructors.
            Some(TestFault::DuringExit) => {
                c.commit_hook(Some(|| std::process::exit(86))).unwrap();
            }
            _ => {}
        }
    }
    pub(super) fn before_commit(
        &self,
        tx: &rusqlite::Transaction<'_>,
    ) -> Result<(), PersistenceError> {
        let mut fault = self.inner.fault.lock().unwrap();
        if matches!(*fault, Some(TestFault::Before)) {
            fault.take();
            return Err(PersistenceError::StorageUnavailable);
        }
        if matches!(*fault, Some(TestFault::DuringExit)) {
            // Spill real uncommitted pages under the rollback journal before
            // entering COMMIT. Recovery must repair disk, not just clear RAM.
            tx.cache_flush().map_err(sql)?;
        }
        Ok(())
    }
    pub(super) fn after_commit(&self, revision: SnapshotRevision) -> Result<(), PersistenceError> {
        match self.inner.fault.lock().unwrap().take() {
            Some(TestFault::AfterLost) => Err(PersistenceError::CommitOutcomeUnknown { revision }),
            Some(TestFault::AfterExit) => std::process::exit(87),
            _ => Ok(()),
        }
    }
}
struct Provider(u8);
impl PersistenceKeyProvider for Provider {
    fn resolve(&self, id: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        if id != "synthetic-key" {
            return Err(ProtectionError::KeyUnavailable);
        }
        Ok(Arc::new(PersistenceKey::new(vec![self.0; 64])?))
    }
}
fn namespace(unit: &str) -> PersistenceNamespace {
    PersistenceNamespace::new(vec![("unit".into(), NamespaceValue::String(unit.into()))]).unwrap()
}
fn schema() -> CompiledFlowSchema {
    // Extend only the synthetic fixture's key width; never a CAD/SIOC schema.
    let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../bundles/embedded/schemas/SYNTHETIC/v1/flow.json"
    ))
    .unwrap();

    raw["traces"][0]["record_length"] = serde_json::json!(11);
    raw["traces"][0]["fields"][0]["physical"]["length"] = serde_json::json!(8);
    raw["traces"][0]["fields"][1]["physical"]["start"] = serde_json::json!(9);
    SchemaCompiler
        .compile({
            let mut raw: RawFlowSchema = serde_json::from_value(raw).unwrap();
            raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
            raw
        })
        .unwrap()
}
fn row(p: &IdentityProtector, unit: &str, key: usize, body: &str) -> StoredIdentity {
    make_row(p, &schema(), unit, key, body)
}
fn make_row(
    p: &IdentityProtector,
    schema: &CompiledFlowSchema,
    unit: &str,
    key: usize,
    body: &str,
) -> StoredIdentity {
    let bytes = format!("{key:08}{body:3}").into_bytes();
    let view = RecordView::new(&bytes);
    let trace = &schema.traces()[0];
    let pk = build_join_key(&view, &trace.record_key().field_indexes, trace).unwrap();
    let projection = build_join_key(&view, &[1], trace).unwrap();
    let scope = IdentityProtectionScope::new(
        namespace(unit),
        FlowId("SYNTHETIC".into()),
        PersistenceSchemaScope::SchemaVersion {
            version: SchemaVersion("v1".into()),
        },
        TraceId("T1".into()),
        "record_key".into(),
    )
    .unwrap();
    p.protect_projection(&scope, &pk, &Sha256Digest([17; 32]), &projection)
        .unwrap()
}
fn source(unit: &str, id: &str) -> SourceSubmission {
    SourceSubmission::new(SourceSubmissionParts {
        namespace: namespace(unit),
        submission_id: id.into(),
        schema_key: SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        schema_hash: Sha256Digest([0; 32]),
        period: None,
        created_at: Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap(),
        validator_version: "synthetic-test".into(),
        validation_mode: ValidationMode::LocalOnly,
        validation_status: RunStatus::CompletedValid,
        reference_data_versions: vec![],
        traces: vec![SubmissionTrace {
            trace_id: TraceId("T1".into()),
            presence: TracePresence::Absent,
            input_sha256: None,
        }],
        fingerprint: Fingerprint::new(
            "synthetic-unprotected".into(),
            "fixture".into(),
            b"SYNTHETIC-RAW-SUBMISSION-MARKER".to_vec(),
        )
        .unwrap(),
        state: SessionState::Validated,
    })
    .unwrap()
}
fn snapshot(
    repo: &SQLitePersistenceAdapter,
    unit: &str,
    rev: u64,
    id: &str,
    rows: Vec<StoredIdentity>,
) -> HistoricalSnapshot {
    HistoricalSnapshot::new(
        SnapshotRevision::new(rev),
        repo.protector()
            .protect_submission(&source(unit, id))
            .unwrap(),
        rows,
    )
    .unwrap()
}
fn publish(
    repo: &SQLitePersistenceAdapter,
    snapshot: &HistoricalSnapshot,
) -> Result<CommitReceipt, PersistenceError> {
    let expected = if snapshot.revision().value() == 1 {
        None
    } else {
        Some(SnapshotRevision::new(snapshot.revision().value() - 1))
    };
    let mut tx = repo.begin_transaction(snapshot.namespace(), expected)?;
    tx.stage_snapshot(snapshot)?;
    tx.commit()
}
fn parent() -> tempfile::TempDir {
    // Use the workspace filesystem, not an ephemeral tmpfs which the durable
    // backend intentionally refuses. CI uses ext4 (Linux) or local NTFS (MSVC).
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/sqlite-tests");
    fs::create_dir_all(&base).unwrap();
    tempfile::tempdir_in(base).unwrap()
}
fn create(path: &Path) -> SQLitePersistenceAdapter {
    SQLitePersistenceAdapter::create(path, Some("synthetic-key"), &Provider(0xaa)).unwrap()
}
fn open(path: &Path) -> SQLitePersistenceAdapter {
    SQLitePersistenceAdapter::open(path, Some("synthetic-key"), &Provider(0xaa)).unwrap()
}
#[test]
fn roundtrip_restart_immutable_snapshots_omission_namespace_and_settings() {
    let dir = parent();
    let root = dir.path().join("archive");
    let repo = create(&root);
    let p = repo.protector();
    let a = row(p, "A", 1, "A");
    let b = row(p, "A", 2, "B");
    let first = snapshot(&repo, "A", 1, "one", vec![a.clone(), b.clone()]);
    let settings = repo.settings().unwrap();
    assert_eq!(settings.sqlite_version, "3.54.1");
    assert_eq!(settings.journal_mode, "delete");
    assert_eq!(settings.synchronous, 3);
    assert_eq!(settings.cache_kib, 2048);
    assert_eq!(settings.mmap_bytes, 0);
    let mut tx = repo.begin_transaction(&namespace("A"), None).unwrap();
    tx.stage_snapshot(&first).unwrap();
    assert!(
        repo.load_current_snapshot(&namespace("A"))
            .unwrap()
            .is_none()
    );
    tx.commit().unwrap();
    let changed = row(repo.protector(), "A", 1, "C");
    let second = snapshot(&repo, "A", 2, "two", vec![changed.clone()]);
    publish(&repo, &second).unwrap();
    let other = snapshot(
        &repo,
        "B",
        1,
        "one",
        vec![row(repo.protector(), "B", 1, "A")],
    );
    publish(&repo, &other).unwrap();
    assert_ne!(
        first.identities()[0].identity().key_fingerprint(),
        other.identities()[0].identity().key_fingerprint()
    );
    drop(repo);
    let repo = open(&root);
    assert_eq!(
        repo.load_snapshot(&namespace("A"), SnapshotRevision::new(1))
            .unwrap(),
        Some(first)
    );
    assert_eq!(
        repo.load_current_snapshot(&namespace("A")).unwrap(),
        Some(second.clone())
    );
    assert_eq!(
        repo.lookup_identity(&namespace("A"), a.identity()).unwrap(),
        Some(changed)
    );
    assert_eq!(
        repo.lookup_identity(&namespace("A"), b.identity()).unwrap(),
        Some(b)
    );
    assert_eq!(
        repo.resolve_commit(&second).unwrap(),
        CommitResolution::Published(CommitReceipt {
            revision: SnapshotRevision::new(2)
        })
    );
    assert_eq!(
        repo.lookup_identity(&namespace("B"), a.identity())
            .unwrap_err(),
        PersistenceError::NamespaceMismatch
    );
    let duplicate = snapshot(&repo, "A", 3, "two", vec![]);
    assert_eq!(
        publish(&repo, &duplicate).unwrap_err(),
        PersistenceError::DuplicateSubmission
    );
}
#[test]
fn abort_drop_stale_generation_and_failure_before_commit_do_not_publish() {
    let dir = parent();
    let repo = create(&dir.path().join("archive"));
    let ns = namespace("A");
    let candidate = snapshot(
        &repo,
        "A",
        1,
        "candidate",
        vec![row(repo.protector(), "A", 7, "A")],
    );
    for abort in [true, false] {
        let mut tx = repo.begin_transaction(&ns, None).unwrap();
        tx.stage_snapshot(&candidate).unwrap();
        if abort {
            tx.abort().unwrap();
        } else {
            drop(tx);
        }
        assert!(repo.load_current_snapshot(&ns).unwrap().is_none());
    }
    *repo.inner.fault.lock().unwrap() = Some(TestFault::Before);
    assert_eq!(
        publish(&repo, &candidate).unwrap_err(),
        PersistenceError::StorageUnavailable
    );
    assert_eq!(
        repo.resolve_commit(&candidate).unwrap(),
        CommitResolution::NotPublished
    );
    let mut stale = repo.begin_transaction(&ns, None).unwrap();
    stale.stage_snapshot(&candidate).unwrap();
    publish(&repo, &candidate).unwrap();
    assert!(matches!(
        stale.commit(),
        Err(PersistenceError::GenerationConflict { .. })
    ));
    assert!(matches!(
        repo.begin_transaction(&ns, None),
        Err(PersistenceError::GenerationConflict { .. })
    ));
}
#[test]
fn ambiguous_commit_is_resolved_by_exact_immutable_candidate() {
    for fault in [TestFault::RejectCommit, TestFault::AfterLost] {
        let dir = parent();
        let repo = create(&dir.path().join("archive"));
        let candidate = snapshot(&repo, "A", 1, "candidate", vec![]);
        let rejected = matches!(fault, TestFault::RejectCommit);
        *repo.inner.fault.lock().unwrap() = Some(fault);
        assert!(matches!(
            publish(&repo, &candidate),
            Err(PersistenceError::CommitOutcomeUnknown { .. })
        ));
        // Remove the injected hook before subsequent read/transaction work.
        repo.lock()
            .unwrap()
            .commit_hook(None::<fn() -> bool>)
            .unwrap();
        assert_eq!(
            repo.resolve_commit(&candidate).unwrap(),
            if rejected {
                CommitResolution::NotPublished
            } else {
                CommitResolution::Published(CommitReceipt {
                    revision: SnapshotRevision::new(1),
                })
            }
        );
    }
}
#[test]
fn protection_cannot_be_bypassed_by_public_dto_or_same_label_wrong_key() {
    let dir = parent();
    let root = dir.path().join("archive");
    let repo = create(&root);
    let protected = row(repo.protector(), "A", 1, "A");
    let dto: PersistentRecordIdentity =
        serde_json::from_slice(&codec::encode(protected.identity()).unwrap()).unwrap();
    assert_eq!(&dto, protected.identity());
    assert!(matches!(
        repo.lookup_identity(&namespace("A"), &dto),
        Err(PersistenceError::Protection(ProtectionError::Unattested))
    ));
    let other = IdentityProtector::preflight(Some("synthetic-key"), &Provider(0xbb), None).unwrap();
    let wrong = row(&other, "A", 1, "A");
    assert!(matches!(
        repo.lookup_identity(&namespace("A"), wrong.identity()),
        Err(PersistenceError::Protection(
            ProtectionError::InvalidAttestation
        ))
    ));
    let fake = StoredIdentity::new(
        protected.identity().clone(),
        protected.record_fingerprint().clone(),
    );
    let bad = snapshot(&repo, "A", 1, "bad", vec![fake]);
    assert!(matches!(
        publish(&repo, &bad),
        Err(PersistenceError::Protection(ProtectionError::Unattested))
    ));
    let bad =
        HistoricalSnapshot::new(SnapshotRevision::new(1), source("A", "raw"), vec![]).unwrap();
    assert!(matches!(
        publish(&repo, &bad),
        Err(PersistenceError::Protection(_))
    ));
    let good = snapshot(&repo, "A", 1, "good", vec![protected]);
    publish(&repo, &good).unwrap();
    assert_eq!(
        repo.protector().protect_submission(good.source()).unwrap(),
        *good.source()
    );
    drop(repo);
    assert!(matches!(
        SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider(0xbb)),
        Err(PersistenceError::Protection(ProtectionError::KeyMismatch))
    ));
    assert!(matches!(
        SQLitePersistenceAdapter::open(&root, None, &Provider(0xaa)),
        Err(PersistenceError::Protection(ProtectionError::MissingKey))
    ));
    let bytes = fs::read(root.join("lintra.db")).unwrap();
    for forbidden in [
        b"00000001A  ".as_slice(),
        b"SYNTHETIC-RAW-SUBMISSION-MARKER".as_slice(),
        &[0xaa; 64],
    ] {
        assert!(!bytes.windows(forbidden.len()).any(|v| v == forbidden));
    }
}
#[test]
fn unknown_unsupported_corrupt_and_uninitialized_are_distinct_without_reinitialization() {
    let dir = parent();
    let root = dir.path().join("archive");
    assert!(matches!(
        SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider(0xaa)),
        Err(PersistenceError::StorageUninitialized)
    ));
    let repo = create(&root);
    drop(repo);
    let file = root.join("lintra.db");
    let original = fs::read(&file).unwrap();
    assert!(matches!(
        SQLitePersistenceAdapter::create(&root, Some("synthetic-key"), &Provider(0xaa)),
        Err(PersistenceError::StorageAlreadyExists)
    ));
    for (offset, value, expected) in [
        (
            60,
            2,
            PersistenceError::UnsupportedStorageVersion { found: 2 },
        ),
        (68, 7, PersistenceError::UnrecognizedStorage),
    ] {
        let mut bytes = original.clone();
        bytes[offset..offset + 4].copy_from_slice(&(value as u32).to_be_bytes());
        fs::write(&file, &bytes).unwrap();
        assert_eq!(
            SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider(0xaa))
                .unwrap_err(),
            expected
        );
        assert_eq!(fs::read(&file).unwrap(), bytes);
    }
    fs::write(&file, b"damaged archive").unwrap();
    assert_eq!(
        SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider(0xaa)).unwrap_err(),
        PersistenceError::CorruptStorage
    );
    fs::write(&file, []).unwrap();
    assert_eq!(
        SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider(0xaa)).unwrap_err(),
        PersistenceError::StorageUninitialized
    );
    fs::write(&file, &original).unwrap();
    let repo = open(&root);
    repo.lock()
        .unwrap()
        .execute("UPDATE storage_meta SET audit=x'00'", [])
        .unwrap();
    drop(repo);
    assert_eq!(
        SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider(0xaa)).unwrap_err(),
        PersistenceError::CorruptStorage
    );
}
#[test]
fn tampered_record_or_source_is_rejected_instead_of_returning_unverified_domain_values() {
    for table in ["records", "snapshots"] {
        let dir = parent();
        let repo = create(&dir.path().join("archive"));
        let row = row(repo.protector(), "A", 1, "A");
        publish(&repo, &snapshot(&repo, "A", 1, "one", vec![row.clone()])).unwrap();
        if table == "records" {
            repo.lock()
                .unwrap()
                .execute("UPDATE records SET record_proof=zeroblob(32)", [])
                .unwrap();
            assert_eq!(
                repo.lookup_identity(&namespace("A"), row.identity())
                    .unwrap_err(),
                PersistenceError::CorruptStorage
            );
        } else {
            repo.lock()
                .unwrap()
                .execute("UPDATE snapshots SET proof=zeroblob(32)", [])
                .unwrap();
            assert_eq!(
                repo.load_submission(&namespace("A"), "one").unwrap_err(),
                PersistenceError::CorruptStorage
            );
        }
    }
}
#[test]
fn unsafe_paths_permissions_and_foreign_sidecars_fail_closed() {
    let dir = parent();
    assert!(matches!(
        SQLitePersistenceAdapter::create(
            Path::new("relative/archive"),
            Some("synthetic-key"),
            &Provider(0xaa)
        ),
        Err(PersistenceError::UnsafeStorageLocation)
    ));
    let root = dir.path().join("archive");
    let repo = create(&root);
    drop(repo);
    assert!(matches!(
        SQLitePersistenceAdapter::open(
            &root.join("../archive"),
            Some("synthetic-key"),
            &Provider(0xaa)
        ),
        Err(PersistenceError::UnsafeStorageLocation)
    ));
    fs::write(root.join("lintra.db-wal"), b"foreign").unwrap();
    assert!(matches!(
        SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider(0xaa)),
        Err(PersistenceError::UnsafeStorageLocation)
    ));
    fs::remove_file(root.join("lintra.db-wal")).unwrap();
    #[cfg(windows)]
    {
        // Native MSVC runner already grants symlink creation for the existing
        // reference-filesystem suite; this exercises the SQLite boundary itself.
        let alias = dir.path().join("alias");
        std::os::windows::fs::symlink_dir(&root, &alias).unwrap();
        assert!(matches!(
            SQLitePersistenceAdapter::open(&alias, Some("synthetic-key"), &Provider(0xaa)),
            Err(PersistenceError::UnsafeStorageLocation)
        ));
        let backup = dir.path().join("original-db");
        fs::rename(root.join("lintra.db"), &backup).unwrap();
        std::os::windows::fs::symlink_file(&backup, root.join("lintra.db")).unwrap();
        assert!(matches!(
            SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider(0xaa)),
            Err(PersistenceError::UnsafeStorageLocation)
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{PermissionsExt, symlink};
        assert_eq!(
            fs::metadata(&root).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(root.join("lintra.db"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        let link = dir.path().join("alias");
        symlink(&root, &link).unwrap();
        assert!(matches!(
            SQLitePersistenceAdapter::open(&link, Some("synthetic-key"), &Provider(0xaa)),
            Err(PersistenceError::UnsafeStorageLocation)
        ));
        fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(matches!(
            SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider(0xaa)),
            Err(PersistenceError::UnsafeStorageLocation)
        ));
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let backup = dir.path().join("original-db");
        fs::rename(root.join("lintra.db"), &backup).unwrap();
        symlink(&backup, root.join("lintra.db")).unwrap();
        assert!(matches!(
            SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider(0xaa)),
            Err(PersistenceError::UnsafeStorageLocation)
        ));
        fs::remove_file(root.join("lintra.db")).unwrap();
        fs::hard_link(&backup, root.join("lintra.db")).unwrap();
        assert!(matches!(
            SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider(0xaa)),
            Err(PersistenceError::UnsafeStorageLocation)
        ));
    }
}
fn child(root: &Path, mode: &str, barrier: Option<&Path>) -> Child {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "persistence::sqlite::tests::process_worker",
            "--ignored",
            "--nocapture",
        ])
        .env("LINTRA_SQLITE_TEST_ROOT", root)
        .env("LINTRA_SQLITE_TEST_MODE", mode);
    if let Some(barrier) = barrier {
        command.env("LINTRA_SQLITE_TEST_BARRIER", barrier);
    }
    command.spawn().unwrap()
}
#[test]
#[ignore = "subprocess helper invoked only with a synthetic archive by parent tests"]
fn process_worker() {
    let root = PathBuf::from(std::env::var_os("LINTRA_SQLITE_TEST_ROOT").unwrap());
    let mode = std::env::var("LINTRA_SQLITE_TEST_MODE").unwrap();
    let repo = open(&root);
    if mode == "restart" {
        let current = repo
            .load_current_snapshot(&namespace("A"))
            .unwrap()
            .unwrap();
        assert_eq!(current.revision(), SnapshotRevision::new(1));
        assert!(
            repo.lookup_identity(
                &namespace("A"),
                row(repo.protector(), "A", 1, "A").identity()
            )
            .unwrap()
            .is_some()
        );
        return;
    }
    let candidate = snapshot(
        &repo,
        "A",
        2,
        &mode,
        vec![row(repo.protector(), "A", 2, "B")],
    );
    let mut tx = repo
        .begin_transaction(&namespace("A"), Some(SnapshotRevision::new(1)))
        .unwrap();
    tx.stage_snapshot(&candidate).unwrap();
    match mode.as_str() {
        "during" => *repo.inner.fault.lock().unwrap() = Some(TestFault::DuringExit),
        "after" => *repo.inner.fault.lock().unwrap() = Some(TestFault::AfterExit),
        _ => {
            let barrier = PathBuf::from(std::env::var_os("LINTRA_SQLITE_TEST_BARRIER").unwrap());
            fs::write(barrier.join(format!("{mode}.ready")), []).unwrap();
            let start = Instant::now();
            while !barrier.join("go").exists() {
                assert!(start.elapsed() < Duration::from_secs(20));
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
    match tx.commit() {
        Ok(_) => std::process::exit(0),
        Err(PersistenceError::StorageBusy | PersistenceError::GenerationConflict { .. }) => {
            std::process::exit(3)
        }
        Err(e) => panic!("unexpected contender error: {e}"),
    }
}
#[test]
fn process_restart_and_abrupt_exit_inside_or_after_commit_recover_coherently() {
    for mode in ["during", "after"] {
        let dir = parent();
        let root = dir.path().join("archive");
        let repo = create(&root);
        let base = snapshot(
            &repo,
            "A",
            1,
            "one",
            vec![row(repo.protector(), "A", 1, "A")],
        );
        publish(&repo, &base).unwrap();
        drop(repo);
        assert!(child(&root, "restart", None).wait().unwrap().success());
        assert_eq!(
            child(&root, mode, None).wait().unwrap().code(),
            Some(if mode == "during" { 86 } else { 87 })
        );
        if mode == "during" {
            assert!(fs::metadata(root.join("lintra.db-journal")).unwrap().len() > 0);
        }
        let repo = open(&root);
        let current = repo
            .load_current_snapshot(&namespace("A"))
            .unwrap()
            .unwrap();
        assert_eq!(
            current.revision().value(),
            if mode == "during" { 1 } else { 2 }
        );
        assert_eq!(
            repo.load_snapshot(&namespace("A"), SnapshotRevision::new(1))
                .unwrap(),
            Some(base)
        );
        assert_eq!(
            repo.load_submission(&namespace("A"), mode)
                .unwrap()
                .is_some(),
            mode == "after"
        );
        assert_eq!(
            repo.lookup_identity(
                &namespace("A"),
                row(repo.protector(), "A", 2, "B").identity()
            )
            .unwrap()
            .is_some(),
            mode == "after"
        );
    }
}
#[test]
fn two_process_contenders_publish_exactly_one_generation() {
    let dir = parent();
    let root = dir.path().join("archive");
    let repo = create(&root);
    publish(&repo, &snapshot(&repo, "A", 1, "one", vec![])).unwrap();
    drop(repo);
    let mut a = child(&root, "a", Some(dir.path()));
    let mut b = child(&root, "b", Some(dir.path()));
    let start = Instant::now();
    while !dir.path().join("a.ready").exists() || !dir.path().join("b.ready").exists() {
        if start.elapsed() > Duration::from_secs(25) {
            let _ = a.kill();
            let _ = b.kill();
            panic!("contenders did not stage within deadline");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    fs::write(dir.path().join("go"), []).unwrap();
    let mut codes = [a.wait().unwrap().code(), b.wait().unwrap().code()];
    codes.sort();
    assert_eq!(codes, [Some(0), Some(3)]);
    let repo = open(&root);
    assert_eq!(
        repo.load_current_snapshot(&namespace("A"))
            .unwrap()
            .unwrap()
            .revision(),
        SnapshotRevision::new(2)
    );
    let winners = ["a", "b"]
        .iter()
        .filter(|id| repo.load_submission(&namespace("A"), id).unwrap().is_some())
        .count();
    assert_eq!(winners, 1);
}
#[test]
fn query_plans_are_indexed_and_external_writer_contention_is_explicit() {
    let dir = parent();
    let root = dir.path().join("archive");
    let repo = create(&root);
    let record = row(repo.protector(), "A", 1, "A");
    publish(&repo, &snapshot(&repo, "A", 1, "one", vec![record.clone()])).unwrap();
    let ns = codec::encode(&namespace("A")).unwrap();
    let id = codec::encode(record.identity()).unwrap();
    let c = repo.lock().unwrap();
    let plans:Vec<String>=c.prepare("EXPLAIN QUERY PLAN SELECT r.payload FROM latest l JOIN records r ON r.ns=l.ns AND r.revision=l.revision AND r.ordinal=l.ordinal WHERE l.ns=?1 AND l.identity=?2").unwrap().query_map(params![ns,id],|r|r.get(3)).unwrap().collect::<Result<_,_>>().unwrap();
    assert!(
        plans
            .iter()
            .all(|p| p.contains("SEARCH") && !p.contains("SCAN")),
        "{plans:?}"
    );
    drop(c);
    let mut other = Connection::open(root.join("lintra.db")).unwrap();
    let tx = other
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    let next = snapshot(&repo, "A", 2, "busy", vec![]);
    assert_eq!(
        publish(&repo, &next).unwrap_err(),
        PersistenceError::StorageBusy
    );
    drop(tx);
    publish(&repo, &next).unwrap();
}

#[test]
fn an_unfinished_transaction_view_cannot_be_mistaken_for_publication() {
    let dir = parent();
    let root = dir.path().join("archive");
    let repo = create(&root);
    repo.lock()
        .unwrap()
        .execute_batch("BEGIN IMMEDIATE")
        .unwrap();
    assert_eq!(
        repo.load_current_snapshot(&namespace("A")).unwrap_err(),
        PersistenceError::StorageUnavailable
    );
    drop(repo);
    assert!(
        open(&root)
            .load_current_snapshot(&namespace("A"))
            .unwrap()
            .is_none()
    );
}
