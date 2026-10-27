//! Synthetic protected tokens only; no patient rows, crypto or real storage.
use chrono::{TimeZone, Utc};
use lintra::*;
use std::sync::{Arc, Barrier};

fn ns(unit: &str) -> PersistenceNamespace {
    PersistenceNamespace::new(vec![(
        "organization".into(),
        NamespaceValue::String(unit.into()),
    )])
    .unwrap()
}
fn token(value: u8) -> Fingerprint {
    Fingerprint::new("synthetic-opaque".into(), "test-key".into(), vec![value]).unwrap()
}
fn record(unit: &str, key: u8, body: u8) -> StoredIdentity {
    StoredIdentity::new(
        PersistentRecordIdentity::new(
            ns(unit),
            FlowId("SYNTHETIC".into()),
            PersistenceSchemaScope::SchemaVersion {
                version: SchemaVersion("v1".into()),
            },
            TraceId("T1".into()),
            "record_key".into(),
            token(key),
        )
        .unwrap(),
        token(body),
    )
}
fn source(unit: &str, id: &str) -> SourceSubmission {
    SourceSubmission::new(SourceSubmissionParts {
        namespace: ns(unit),
        submission_id: id.into(),
        schema_key: SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        schema_hash: Sha256Digest([0; 32]),
        period: None,
        created_at: Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap(),
        validator_version: "test".into(),
        validation_mode: ValidationMode::LocalOnly,
        validation_status: RunStatus::CompletedValid,
        reference_data_versions: vec![],
        traces: vec![SubmissionTrace {
            trace_id: TraceId("T1".into()),
            presence: TracePresence::Absent,
            input_sha256: None,
        }],
        fingerprint: token(42),
        state: SessionState::Validated,
    })
    .unwrap()
}
fn snapshot(
    unit: &str,
    revision: u64,
    id: &str,
    records: Vec<StoredIdentity>,
) -> HistoricalSnapshot {
    HistoricalSnapshot::new(SnapshotRevision::new(revision), source(unit, id), records).unwrap()
}
fn publish(
    repo: &dyn PersistentContextRepository,
    expected: Option<u64>,
    value: &HistoricalSnapshot,
) -> CommitReceipt {
    let mut transaction = repo
        .begin_transaction(value.namespace(), expected.map(SnapshotRevision::new))
        .unwrap();
    transaction.stage_snapshot(value).unwrap();
    transaction.commit().unwrap()
}
#[test]
fn staging_is_invisible_and_commit_publishes_snapshot_index_and_metadata_together() {
    let repo = InMemoryPersistenceRepository::default();
    let namespace = ns("A");
    let row = record("A", 1, 10);
    let expected = snapshot("A", 1, "run-1", vec![row.clone()]);
    let mut transaction = repo.begin_transaction(&namespace, None).unwrap();
    transaction.stage_snapshot(&expected).unwrap();
    assert!(repo.load_current_snapshot(&namespace).unwrap().is_none());
    assert!(
        repo.lookup_identity(&namespace, row.identity())
            .unwrap()
            .is_none()
    );
    assert!(repo.load_submission(&namespace, "run-1").unwrap().is_none());
    assert_eq!(
        transaction.commit().unwrap().revision(),
        SnapshotRevision::new(1)
    );
    assert_eq!(
        repo.load_current_snapshot(&namespace).unwrap().unwrap(),
        expected
    );
    assert_eq!(
        repo.lookup_identity(&namespace, row.identity()).unwrap(),
        Some(row)
    );
    assert_eq!(
        repo.load_submission(&namespace, "run-1").unwrap(),
        Some(source("A", "run-1"))
    );
}
#[test]
fn abort_drop_and_empty_commit_preserve_committed_image() {
    let repo = InMemoryPersistenceRepository::default();
    let namespace = ns("A");
    let base = snapshot("A", 1, "run-1", vec![record("A", 1, 10)]);
    publish(&repo, None, &base);
    for abort in [true, false] {
        let mut transaction = repo
            .begin_transaction(&namespace, Some(SnapshotRevision::new(1)))
            .unwrap();
        transaction
            .stage_snapshot(&snapshot("A", 2, "discarded", vec![record("A", 2, 20)]))
            .unwrap();
        if abort {
            transaction.abort().unwrap();
        } else {
            drop(transaction);
        }
        assert_eq!(
            repo.load_current_snapshot(&namespace).unwrap(),
            Some(base.clone())
        );
        assert!(
            repo.load_submission(&namespace, "discarded")
                .unwrap()
                .is_none()
        );
        assert!(
            repo.lookup_identity(&namespace, record("A", 2, 20).identity())
                .unwrap()
                .is_none()
        );
    }
    assert_eq!(
        repo.begin_transaction(&namespace, Some(SnapshotRevision::new(1)))
            .unwrap()
            .commit()
            .unwrap_err(),
        PersistenceError::NothingStaged
    );
}
#[test]
fn reopened_fake_recovers_all_committed_revisions_and_never_staging() {
    let image = InMemoryPersistenceImage::default();
    let first = InMemoryPersistenceRepository::open(image.clone());
    let base = snapshot("A", 1, "one", vec![record("A", 1, 10)]);
    publish(&first, None, &base);
    let mut pending = first
        .begin_transaction(&ns("A"), Some(SnapshotRevision::new(1)))
        .unwrap();
    pending
        .stage_snapshot(&snapshot("A", 2, "not-committed", vec![record("A", 2, 20)]))
        .unwrap();
    drop(first);
    let reopened = InMemoryPersistenceRepository::open(image);
    assert_eq!(
        reopened.load_current_snapshot(&ns("A")).unwrap(),
        Some(base.clone())
    );
    assert!(
        reopened
            .load_submission(&ns("A"), "not-committed")
            .unwrap()
            .is_none()
    );
    pending.abort().unwrap();
    let next = snapshot("A", 2, "two", vec![record("A", 1, 11)]);
    publish(&reopened, Some(1), &next);
    assert_eq!(
        reopened
            .load_snapshot(&ns("A"), SnapshotRevision::new(1))
            .unwrap(),
        Some(base)
    );
    assert_eq!(
        reopened
            .lookup_identity(&ns("A"), record("A", 1, 0).identity())
            .unwrap(),
        Some(record("A", 1, 11))
    );
    assert_eq!(
        reopened.load_current_snapshot(&ns("A")).unwrap(),
        Some(next)
    );
}
#[test]
fn exact_scope_isolation_and_omission_do_not_erase_observed_history() {
    let repo = InMemoryPersistenceRepository::default();
    for unit in ["A", "B"] {
        publish(
            &repo,
            None,
            &snapshot(unit, 1, "same-id", vec![record(unit, 1, 10)]),
        );
    }
    publish(&repo, Some(1), &snapshot("A", 2, "next", vec![]));
    assert!(
        repo.load_current_snapshot(&ns("A"))
            .unwrap()
            .unwrap()
            .identities()
            .is_empty()
    );
    assert_eq!(
        repo.lookup_identity(&ns("A"), record("A", 1, 0).identity())
            .unwrap(),
        Some(record("A", 1, 10))
    );
    assert_eq!(
        repo.lookup_identity(&ns("B"), record("A", 1, 0).identity())
            .unwrap_err(),
        PersistenceError::NamespaceMismatch
    );
    assert_eq!(
        repo.load_current_snapshot(&ns("B"))
            .unwrap()
            .unwrap()
            .revision(),
        SnapshotRevision::new(1)
    );
    assert!(
        repo.lookup_identity(&ns("C"), record("C", 1, 0).identity())
            .unwrap()
            .is_none()
    );
}
#[test]
fn wrong_generation_stale_commit_and_collision_leave_old_state_intact() {
    let repo = InMemoryPersistenceRepository::default();
    let namespace = ns("A");
    assert!(matches!(
        repo.begin_transaction(&namespace, Some(SnapshotRevision::new(0))),
        Err(PersistenceError::GenerationConflict { actual: None, .. })
    ));
    let mut stale = repo.begin_transaction(&namespace, None).unwrap();
    stale
        .stage_snapshot(&snapshot("A", 1, "loser", vec![record("A", 2, 20)]))
        .unwrap();
    let winner = snapshot("A", 1, "winner", vec![record("A", 1, 10)]);
    publish(&repo, None, &winner);
    assert!(matches!(
        stale.commit(),
        Err(PersistenceError::GenerationConflict {
            actual: Some(_),
            ..
        })
    ));
    let mut collision = repo
        .begin_transaction(&namespace, Some(SnapshotRevision::new(1)))
        .unwrap();
    collision
        .stage_snapshot(&snapshot("A", 2, "winner", vec![record("A", 1, 99)]))
        .unwrap();
    assert_eq!(
        collision.commit().unwrap_err(),
        PersistenceError::DuplicateSubmission
    );
    assert_eq!(
        repo.load_current_snapshot(&namespace).unwrap(),
        Some(winner)
    );
    assert!(repo.load_submission(&namespace, "loser").unwrap().is_none());
}
#[test]
fn structural_errors_and_failed_staging_leave_no_committable_candidate() {
    let repo = InMemoryPersistenceRepository::default();
    let row = record("A", 1, 10);
    assert_eq!(
        HistoricalSnapshot::new(
            SnapshotRevision::new(1),
            source("B", "one"),
            vec![row.clone()]
        )
        .unwrap_err(),
        PersistenceError::NamespaceMismatch
    );
    assert_eq!(
        HistoricalSnapshot::new(
            SnapshotRevision::new(1),
            source("A", "one"),
            vec![row.clone(), row.clone()]
        )
        .unwrap_err(),
        PersistenceError::DuplicateIdentity
    );
    let valid = snapshot("A", 1, "one", vec![row]);
    for (rejected, error) in [
        (
            snapshot("B", 1, "two", vec![]),
            PersistenceError::NamespaceMismatch,
        ),
        (
            snapshot("A", 0, "bad", vec![]),
            PersistenceError::InvalidRevision,
        ),
    ] {
        let mut transaction = repo.begin_transaction(&ns("A"), None).unwrap();
        transaction.stage_snapshot(&valid).unwrap();
        assert_eq!(transaction.stage_snapshot(&rejected).unwrap_err(), error);
        assert_eq!(
            transaction.commit().unwrap_err(),
            PersistenceError::NothingStaged
        );
        assert!(repo.load_current_snapshot(&ns("A")).unwrap().is_none());
    }
    publish(&repo, None, &valid);
    assert_eq!(repo.load_current_snapshot(&ns("A")).unwrap(), Some(valid));
    // Diagnostics contain no actual tenant, record token or submission ID.
    assert_eq!(
        format!("{}", PersistenceError::DuplicateSubmission),
        "submission identifier already committed"
    );
}
#[test]
fn concurrent_transactions_have_exactly_one_cas_winner() {
    let repo = Arc::new(InMemoryPersistenceRepository::default());
    let barrier = Arc::new(Barrier::new(2));
    let handles: Vec<_> = (1..=2)
        .map(|number| {
            let repo = repo.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let mut transaction = repo.begin_transaction(&ns("A"), None).unwrap();
                transaction
                    .stage_snapshot(&snapshot(
                        "A",
                        1,
                        &format!("run-{number}"),
                        vec![record("A", number, number)],
                    ))
                    .unwrap();
                barrier.wait();
                transaction.commit()
            })
        })
        .collect();
    let results: Vec<_> = handles
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|r| matches!(r, Err(PersistenceError::GenerationConflict { .. })))
            .count(),
        1
    );
    let current = repo.load_current_snapshot(&ns("A")).unwrap().unwrap();
    assert_eq!(current.revision(), SnapshotRevision::new(1));
    assert_eq!(current.identities().len(), 1);
    assert_eq!(
        repo.load_submission(&ns("A"), current.source().submission_id())
            .unwrap(),
        Some(current.source().clone())
    );
}
