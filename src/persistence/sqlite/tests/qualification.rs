//! Transaction visibility and real cross-OS file exchange. Every fixture uses
//! public synthetic key material; none of these helpers exists in release code.
use super::*;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};

/// A failed assertion must not leave a paused child running. Even after an
/// explicit kill/wait, the cached exit status prevents another kill request.
struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
fn wait_for_marker(child: &mut OwnedChild, marker: &Path) {
    let start = Instant::now();
    while !marker.exists() {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "writer exited before its barrier"
        );
        assert!(
            start.elapsed() < Duration::from_secs(25),
            "writer barrier deadline"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
/// Blocking is a technical outcome, never an empty/partially committed result.
/// Staging itself owns no write lock, so only the SQL phase permits StorageBusy.
fn committed_or_busy<T: PartialEq + std::fmt::Debug>(
    actual: Result<T, PersistenceError>,
    expected: T,
    sql_phase: bool,
) {
    match actual {
        Ok(actual) => assert_eq!(actual, expected),
        Err(PersistenceError::StorageBusy) if sql_phase => {}
        Err(error) => panic!("unexpected read failure: {error:?}"),
    }
}

#[test]
fn readers_never_observe_staged_or_uncommitted_rows_and_killed_writer_recovers() {
    for mode in ["stage-pause", "write-pause"] {
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
        let reader = open(&root); // genuinely independent SQLite connection
        let pending = row(repo.protector(), "A", 2, "B");
        drop(repo);
        let mut writer = OwnedChild(child(&root, mode, Some(dir.path())));
        wait_for_marker(&mut writer, &dir.path().join(format!("{mode}.ready")));
        let sql_phase = mode == "write-pause";
        committed_or_busy(
            reader.current_generation(&namespace("A")),
            Some(SnapshotRevision::new(1)),
            sql_phase,
        );
        committed_or_busy(
            reader.load_current_snapshot(&namespace("A")),
            Some(base.clone()),
            sql_phase,
        );
        committed_or_busy(
            reader.load_snapshot(&namespace("A"), SnapshotRevision::new(2)),
            None,
            sql_phase,
        );
        committed_or_busy(
            reader.load_submission(&namespace("A"), mode),
            None,
            sql_phase,
        );
        committed_or_busy(
            reader.lookup_identity(&namespace("A"), pending.identity()),
            None,
            sql_phase,
        );
        // Child::kill is abrupt on both supported OSes: no Rust Drop/rollback
        // runs in the writer. Parent retains ownership and waits for termination.
        writer.0.kill().unwrap();
        assert!(!writer.0.wait().unwrap().success());
        drop(reader);
        let recovered = open(&root);
        assert_eq!(
            recovered.current_generation(&namespace("A")).unwrap(),
            Some(SnapshotRevision::new(1))
        );
        assert_eq!(
            recovered.load_current_snapshot(&namespace("A")).unwrap(),
            Some(base)
        );
        assert_eq!(
            recovered
                .load_snapshot(&namespace("A"), SnapshotRevision::new(2))
                .unwrap(),
            None
        );
        assert_eq!(
            recovered.load_submission(&namespace("A"), mode).unwrap(),
            None
        );
        assert_eq!(
            recovered
                .lookup_identity(&namespace("A"), pending.identity())
                .unwrap(),
            None
        );
        // Recovery must leave a usable archive. Publish the exact candidate
        // that was interrupted, then check the complete atomic result.
        let candidate = snapshot(&recovered, "A", 2, mode, vec![pending]);
        publish(&recovered, &candidate).unwrap();
        assert_eq!(
            recovered.load_current_snapshot(&namespace("A")).unwrap(),
            Some(candidate.clone())
        );
        assert_eq!(
            recovered.resolve_commit(&candidate).unwrap(),
            CommitResolution::Published(CommitReceipt {
                revision: SnapshotRevision::new(2)
            })
        );
    }
}

const DB_LIMIT: u64 = 4 * 1024 * 1024;
const MANIFEST_LIMIT: u64 = 16 * 1024;
const FIXTURE_KIND: &str = "lintra.synthetic-sqlite-transfer.v1";
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct TransferManifest {
    kind: String,
    producer_os: String,
    db_sha256: String,
}
fn digest(bytes: &[u8]) -> String {
    Sha256Digest(Sha256::digest(bytes).into()).to_string()
}
fn storage_io<T>(value: std::io::Result<T>) -> Result<T, PersistenceError> {
    value.map_err(|_| PersistenceError::StorageUnavailable)
}
fn bounded_file(path: &Path, limit: u64) -> Result<Vec<u8>, PersistenceError> {
    let mut bytes = Vec::new();
    storage_io(
        storage_io(fs::File::open(path))?
            .take(limit + 1)
            .read_to_end(&mut bytes),
    )?;
    if bytes.len() as u64 > limit {
        return Err(PersistenceError::CorruptStorage);
    }
    Ok(bytes)
}
fn expected_snapshots(repo: &SQLitePersistenceAdapter) -> Vec<HistoricalSnapshot> {
    // One physical snapshot exercises both logical readers and format2's new
    // nullable correlation across the actual Linux/MSVC database exchange.
    let facts = super::history::candidate(repo, 3, "history-facts");
    let legacy = facts.history_facts()[0].clone();
    let mut payload = legacy.payload.clone();
    payload.format = 2;
    payload.plan_digest = Sha256Digest([6; 32]);
    payload.facts[0].entity = None;
    payload.facts[0].assessment = crate::persistence::history::HistoryAssessment::Indeterminate {
        reason: crate::persistence::history::HistoryIndeterminate::UnqualifiedCorrelation,
    };
    let current = crate::persistence::history::HistoryFactBatch::create(
        payload,
        facts.source(),
        repo.protector(),
    )
    .unwrap();
    let facts = facts
        .with_history_facts(vec![legacy, current], repo.protector())
        .unwrap();
    vec![
        snapshot(
            repo,
            "A",
            1,
            "one",
            vec![
                row(repo.protector(), "A", 1, "A"),
                row(repo.protector(), "A", 2, "B"),
            ],
        ),
        snapshot(
            repo,
            "A",
            2,
            "two",
            vec![row(repo.protector(), "A", 1, "C")],
        ),
        snapshot(
            repo,
            "B",
            1,
            "one",
            vec![row(repo.protector(), "B", 1, "A")],
        ),
        facts,
    ]
}
/// A closed native database, not a serialized Rust snapshot standing in for it.
/// Only a new test output directory is accepted; old files are never replaced.
fn export_fixture(output: &Path) {
    let temp = parent();
    let root = temp.path().join("archive");
    let repo = create(&root);
    for value in expected_snapshots(&repo) {
        publish(&repo, &value).unwrap();
    }
    drop(repo);
    let bytes = bounded_file(&root.join("lintra.db"), DB_LIMIT).unwrap();
    assert!(!bytes.windows(64).any(|w| w == [0xaa; 64]));
    assert!(
        !bytes
            .windows(b"SYNTHETIC-RAW-SUBMISSION-MARKER".len())
            .any(|w| w == b"SYNTHETIC-RAW-SUBMISSION-MARKER")
    );
    for raw in [b"00000001A  ", b"00000001C  ", b"00000002B  "] {
        assert!(!bytes.windows(raw.len()).any(|w| w == raw));
    }
    fs::create_dir(output).unwrap();
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join("fixture.db"))
        .unwrap()
        .write_all(&bytes)
        .unwrap();
    let manifest = TransferManifest {
        kind: FIXTURE_KIND.into(),
        producer_os: std::env::consts::OS.into(),
        db_sha256: digest(&bytes),
    };
    fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
/// Test transfer only: download permissions are not accepted as archive ACLs.
/// Create a NEW private container through the real backend, close it, then copy
/// the bounded synthetic fixture into that container and reopen/verify it.
/// This deliberately exports no product restore API or migration behavior.
fn import_fixture(
    input: &Path,
    root: &Path,
) -> Result<(SQLitePersistenceAdapter, TransferManifest), PersistenceError> {
    let manifest: TransferManifest =
        serde_json::from_slice(&bounded_file(&input.join("manifest.json"), MANIFEST_LIMIT)?)
            .map_err(|_| PersistenceError::CorruptStorage)?;
    let bytes = bounded_file(&input.join("fixture.db"), DB_LIMIT)?;
    if manifest.kind != FIXTURE_KIND
        || !matches!(manifest.producer_os.as_str(), "linux" | "windows")
        || manifest.db_sha256 != digest(&bytes)
    {
        return Err(PersistenceError::CorruptStorage);
    }
    let private = SQLitePersistenceAdapter::create(root, Some("synthetic-key"), &Provider(0xaa))?;
    drop(private);
    let mut file = storage_io(
        fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(root.join("lintra.db")),
    )?;
    storage_io(file.write_all(&bytes))?;
    storage_io(file.sync_all())?;
    drop(file);
    let repo = SQLitePersistenceAdapter::open(root, Some("synthetic-key"), &Provider(0xaa))?;
    Ok((repo, manifest))
}
fn verify_fixture(repo: &SQLitePersistenceAdapter) {
    let settings = repo.settings().unwrap();
    assert_eq!(settings.format_version, FORMAT as u32);
    assert_eq!(settings.journal_mode, "delete");
    assert_eq!(settings.synchronous, 3);
    repo.protector()
        .verify_audit(&repo.load_protection_audit().unwrap())
        .unwrap();
    for expected in expected_snapshots(repo) {
        let actual = repo
            .load_snapshot(expected.namespace(), expected.revision())
            .unwrap()
            .unwrap();
        assert_eq!(actual, expected);
        let facts = repo
            .load_history_facts(expected.namespace(), expected.revision())
            .unwrap()
            .unwrap();
        assert!(facts.inventory_supported());
        assert_eq!(facts.batches(), expected.history_facts());
        assert_eq!(facts.source(), expected.source());
        repo.protector().verify_submission(actual.source()).unwrap();
        for record in actual.identities() {
            repo.protector().verify_stored_identity(record).unwrap();
        }
        assert_eq!(
            repo.resolve_commit(&expected).unwrap(),
            CommitResolution::Published(CommitReceipt {
                revision: expected.revision()
            })
        );
    }
    assert_eq!(
        repo.current_generation(&namespace("A")).unwrap(),
        Some(SnapshotRevision::new(3))
    );
    assert_eq!(
        repo.current_generation(&namespace("B")).unwrap(),
        Some(SnapshotRevision::new(1))
    );
    let retained = row(repo.protector(), "A", 2, "B");
    let occurrence = repo
        .lookup_identity_occurrence(&namespace("A"), retained.identity())
        .unwrap()
        .unwrap();
    assert_eq!(occurrence.revision(), SnapshotRevision::new(1));
    assert_eq!(occurrence.source().submission_id(), "one");
    assert_eq!(occurrence.record(), &retained);
    assert_eq!(
        repo.load_submission_page(&namespace("A"), None)
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn closed_native_fixture_roundtrips_with_provenance_retained_keys_and_key_confirmation() {
    let temp = parent();
    let output = temp.path().join("transfer");
    export_fixture(&output);
    let root = temp.path().join("imported");
    let (repo, _) = import_fixture(&output, &root).unwrap();
    verify_fixture(&repo);
    drop(repo);
    let before = fs::read(root.join("lintra.db")).unwrap();
    assert!(matches!(
        SQLitePersistenceAdapter::open(&root, Some("synthetic-key"), &Provider(0xbb)),
        Err(PersistenceError::Protection(ProtectionError::KeyMismatch))
    ));
    assert_eq!(fs::read(root.join("lintra.db")).unwrap(), before);
}

#[test]
fn malformed_or_oversized_transfer_manifest_never_creates_an_archive() {
    let temp = parent();
    let output = temp.path().join("transfer");
    export_fixture(&output);
    let root = temp.path().join("imported");
    // Unknown fields are rejected as well as invalid JSON. A transport envelope
    // cannot silently acquire extra semantics or cause unbounded allocation.
    for invalid in [
        b"{".to_vec(),
        br#"{"kind":"lintra.synthetic-sqlite-transfer.v1","producer_os":"linux","db_sha256":"unused","extra":true}"#.to_vec(),
        vec![b' '; MANIFEST_LIMIT as usize + 1],
    ] {
        fs::write(output.join("manifest.json"), invalid).unwrap();
        assert!(matches!(
            import_fixture(&output, &root),
            Err(PersistenceError::CorruptStorage)
        ));
        assert!(!root.exists());
    }
}
#[test]
fn corrupt_transfer_hash_or_database_fails_without_reinitialization() {
    let temp = parent();
    let output = temp.path().join("transfer");
    export_fixture(&output);
    let mut bytes = fs::read(output.join("fixture.db")).unwrap();
    bytes[..16].fill(0);
    fs::write(output.join("fixture.db"), &bytes).unwrap();
    let root = temp.path().join("imported");
    assert!(matches!(
        import_fixture(&output, &root),
        Err(PersistenceError::CorruptStorage)
    ));
    assert!(!root.exists());
    // Even with a matching transport hash, malformed SQLite is not a new store.
    let manifest = TransferManifest {
        kind: FIXTURE_KIND.into(),
        producer_os: std::env::consts::OS.into(),
        db_sha256: digest(&bytes),
    };
    fs::write(
        output.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        import_fixture(&output, &root),
        Err(PersistenceError::CorruptStorage)
    ));
    assert_eq!(fs::read(root.join("lintra.db")).unwrap(), bytes);
}

#[test]
#[ignore = "CI exports a public synthetic database for the opposite native runner"]
fn export_native_snapshot_fixture() {
    let output = PathBuf::from(
        std::env::var_os("LINTRA_SQLITE_EXPORT_DIR").expect("explicit synthetic export directory"),
    );
    export_fixture(&output);
}
#[test]
#[ignore = "CI reopens the actual synthetic database produced by the opposite OS"]
fn import_opposite_native_snapshot_fixture() {
    let input = PathBuf::from(
        std::env::var_os("LINTRA_SQLITE_IMPORT_DIR").expect("explicit synthetic import directory"),
    );
    let expected_os =
        std::env::var("LINTRA_SQLITE_EXPECTED_ORIGIN").expect("explicit native producer OS");
    assert_ne!(expected_os, std::env::consts::OS);
    let temp = parent();
    let (repo, manifest) = import_fixture(&input, &temp.path().join("imported")).unwrap();
    assert_eq!(manifest.producer_os, expected_os);
    verify_fixture(&repo);
    let result = serde_json::json!({"fixture_kind":FIXTURE_KIND,"producer_os":manifest.producer_os,"consumer_os":std::env::consts::OS,"source_db_sha256":manifest.db_sha256,"snapshots_verified":4,"namespaces_verified":2,"retained_occurrence_verified":true,"protection_verified":true,"history_batch_verified":true,"snapshot_manifests_verified":true,"logical_fact_formats_verified":[1,2],"nullable_format2_verified":true});
    let output = PathBuf::from(
        std::env::var_os("LINTRA_SQLITE_IMPORT_RESULT").expect("explicit result file"),
    );
    fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}
