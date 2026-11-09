//! Fact payload/atomicity qualification, not a simulated clinical service.
use super::*;
use crate::persistence::history::*;
use serde_json::{Value, json};

pub(super) fn candidate(
    repo: &SQLitePersistenceAdapter,
    revision: u64,
    id: &str,
) -> HistoricalSnapshot {
    let p = repo.protector();
    let record = row(p, "A", 1, "RAW");
    let mut value = serde_json::to_value(source("A", id)).unwrap();
    value["state"] = serde_json::to_value(SessionState::ApprovedImmutable).unwrap();
    value["validation_mode"] = serde_json::to_value(ValidationMode::Full).unwrap();
    value["period"] = json!("synthetic-2026-Q1");
    value["traces"][0]["presence"] =
        serde_json::to_value(TracePresence::Present { record_count: 1 }).unwrap();
    value["traces"][0]["input_sha256"] = serde_json::to_value(Sha256Digest([9; 32])).unwrap();
    let source = p
        .protect_submission(&serde_json::from_value::<SourceSubmission>(value).unwrap())
        .unwrap();
    let entity_scope = IdentityProtectionScope::new(
        namespace("A"),
        FlowId("SYNTHETIC".into()),
        PersistenceSchemaScope::SchemaVersion {
            version: SchemaVersion("v1".into()),
        },
        TraceId("T1".into()),
        "history.entity.v1".into(),
    )
    .unwrap();
    let key = crate::context::join_key::build_join_key_from_components(
        [b"PUBLIC-EVENT-NEVER-PERSISTED".as_slice()]
            .into_iter()
            .map(Ok),
    )
    .unwrap();
    let entity = p
        .protect_key(&entity_scope, &key)
        .unwrap()
        .key_fingerprint()
        .clone();
    let payload = HistoryBatchPayload {
        format: 1,
        schema_key: source.schema_key().clone(),
        schema_hash: source.schema_hash().clone(),
        namespace: namespace("A"),
        target_trace: TraceId("T1".into()),
        plan_digest: Sha256Digest([7; 32]),
        source_fingerprint: source.fingerprint().clone(),
        revision: SnapshotRevision::new(revision),
        base_generation: revision
            .checked_sub(1)
            .filter(|v| *v != 0)
            .map(SnapshotRevision::new),
        period: HistoryPeriod {
            year: 2026,
            quarter: 1,
        },
        source_period: source.period().unwrap().to_owned(),
        facts: vec![HistoryFactRow {
            entity: Some(entity),
            witness_ordinal: 0,
            witness_key: record.identity().key_fingerprint().clone(),
            witness_record: record.record_fingerprint().clone(),
            assessment: HistoryAssessment::Positive,
        }],
        origins: vec![],
    };
    let batch = HistoryFactBatch::create(payload, &source, p).unwrap();
    HistoricalSnapshot::new(SnapshotRevision::new(revision), source, vec![record])
        .unwrap()
        .with_history_facts(vec![batch], p)
        .unwrap()
}
#[test]
fn history_batch_roundtrip_reopen_and_private_attestation() {
    let dir = parent();
    let root = dir.path().join("archive");
    let repo = create(&root);
    assert_eq!(repo.settings().unwrap().format_version, 3);
    let value = candidate(&repo, 1, "fact-one");
    publish(&repo, &value).unwrap();
    drop(repo);
    let repo = open(&root);
    let read = repo
        .load_snapshot(&namespace("A"), SnapshotRevision::new(1))
        .unwrap()
        .unwrap();
    assert_eq!(value, read);
    assert_eq!(read.history_facts().len(), 1);
    assert_eq!(read.history_facts()[0].fact_count(), 1);
    let facts = repo
        .load_history_facts(&namespace("A"), SnapshotRevision::new(1))
        .unwrap()
        .unwrap();
    assert!(facts.inventory_supported());
    assert_eq!(facts.source(), read.source());
    assert_eq!(facts.batches(), read.history_facts());
    assert!(matches!(
        repo.resolve_commit(&value).unwrap(),
        CommitResolution::Published(_)
    ));
    let bytes = fs::read(root.join("lintra.db")).unwrap();
    assert!(
        !bytes
            .windows(b"PUBLIC-EVENT-NEVER-PERSISTED".len())
            .any(|v| v == b"PUBLIC-EVENT-NEVER-PERSISTED")
    );
}
#[test]
fn history_payload_tampering_cannot_reuse_a_valid_identity_token() {
    for change in 0..5 {
        let dir = parent();
        let root = dir.path().join("archive");
        let repo = create(&root);
        let value = candidate(&repo, 1, "fact-one");
        publish(&repo, &value).unwrap();
        let connection = repo.lock().unwrap();
        let bytes: Vec<u8> = connection
            .query_row("SELECT payload FROM history_batches", [], |r| r.get(0))
            .unwrap();
        let mut payload: Value = serde_json::from_slice(&bytes).unwrap();
        match change {
            0 => payload["facts"][0]["assessment"] = json!({"kind":"non_witness"}),
            1 => payload["period"]["quarter"] = json!(2),
            2 => payload["facts"] = json!([]),
            3 => {
                payload["origins"] = json!([{"entity":payload["facts"][0]["entity"].clone(),"from":{"year":2026,"quarter":1},"evidence_sha256":Sha256Digest([8;32]),"evidence_reference":"fake","responsible":"fake","recorded_at":"2026-01-01T00:00:00Z"}])
            }
            4 => payload["format"] = json!(99),
            _ => unreachable!(),
        }
        connection
            .execute(
                "UPDATE history_batches SET payload=?1",
                [serde_json::to_vec(&payload).unwrap()],
            )
            .unwrap();
        drop(connection);
        assert!(matches!(
            repo.load_current_snapshot(&namespace("A")),
            Err(PersistenceError::CorruptStorage)
        ));
    }
}
#[test]
fn history_facts_share_transaction_cas_and_ambiguous_commit_resolution() {
    let dir = parent();
    let root = dir.path().join("archive");
    let repo = create(&root);
    let first = candidate(&repo, 1, "fact-one");
    *repo.inner.fault.lock().unwrap() = Some(TestFault::Before);
    assert!(publish(&repo, &first).is_err());
    assert_eq!(repo.current_generation(&namespace("A")).unwrap(), None);
    assert_eq!(
        repo.lock()
            .unwrap()
            .query_row("SELECT count(*) FROM history_batches", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    *repo.inner.fault.lock().unwrap() = Some(TestFault::AfterLost);
    assert!(matches!(
        publish(&repo, &first),
        Err(PersistenceError::CommitOutcomeUnknown { .. })
    ));
    assert!(matches!(
        repo.resolve_commit(&first).unwrap(),
        CommitResolution::Published(_)
    ));
    let second = candidate(&repo, 2, "fact-two");
    let mut a = repo
        .begin_transaction(&namespace("A"), Some(SnapshotRevision::new(1)))
        .unwrap();
    let mut b = repo
        .begin_transaction(&namespace("A"), Some(SnapshotRevision::new(1)))
        .unwrap();
    a.stage_snapshot(&second).unwrap();
    b.stage_snapshot(&second).unwrap();
    a.commit().unwrap();
    assert!(matches!(
        b.commit(),
        Err(PersistenceError::GenerationConflict { .. })
    ));
    assert_eq!(
        repo.lock()
            .unwrap()
            .query_row("SELECT count(*) FROM history_batches", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        2
    );
}
#[test]
fn legacy_two_is_not_implicitly_upgraded_or_given_fact_coverage() {
    let dir = parent();
    let root = dir.path().join("archive");
    let repo = create(&root);
    let value = candidate(&repo, 1, "fact-one");
    repo.lock()
        .unwrap()
        .execute("DROP TABLE history_batches", [])
        .unwrap();
    repo.lock()
        .unwrap()
        .pragma_update(None, "user_version", 2)
        .unwrap();
    let mut tx = repo.begin_transaction(&namespace("A"), None).unwrap();
    assert!(matches!(
        tx.stage_snapshot(&value),
        Err(PersistenceError::StorageUpgradeRequired {
            found: 2,
            required: 3
        })
    ));
    drop(tx);
    repo.upgrade_storage_v1_to_v2().unwrap();
    assert_eq!(repo.settings().unwrap().format_version, 2);
    assert_eq!(repo.current_generation(&namespace("A")).unwrap(), None);
}

#[test]
fn deleting_batches_records_or_manifest_is_corruption_not_negative_evidence() {
    for mutation in [
        "DELETE FROM history_batches",
        "DELETE FROM snapshot_manifests",
        // Adjusting the unauthenticated SQL count cannot repair the inventory.
        "DELETE FROM latest; DELETE FROM records; UPDATE snapshots SET record_count=0",
    ] {
        let dir = parent();
        let repo = create(&dir.path().join("archive"));
        let value = candidate(&repo, 1, "fact-one");
        publish(&repo, &value).unwrap();
        repo.lock().unwrap().execute_batch(mutation).unwrap();
        assert!(
            matches!(
                repo.load_current_snapshot(&namespace("A")),
                Err(PersistenceError::CorruptStorage)
            ),
            "{mutation}"
        );
        assert!(!matches!(
            repo.resolve_commit(&value),
            Ok(CommitResolution::Published(_))
        ));
    }
}

#[test]
fn authenticated_fact_must_reference_a_physical_record_in_its_own_snapshot() {
    let dir = parent();
    let repo = create(&dir.path().join("archive"));
    let snapshot = candidate(&repo, 1, "fact-one");
    for mutation in 0..3 {
        let mut payload = snapshot.history_facts()[0].payload.clone();
        let other = row(repo.protector(), "A", 2, "NEW");
        if mutation == 0 {
            payload.facts[0].witness_key = other.identity().key_fingerprint().clone();
        } else if mutation == 1 {
            payload.facts[0].witness_record = other.record_fingerprint().clone();
        } else {
            payload.facts[0].witness_ordinal = u64::MAX;
        }
        let batch = HistoryFactBatch::create(payload, snapshot.source(), repo.protector()).unwrap();
        let bare = HistoricalSnapshot::new(
            snapshot.revision(),
            snapshot.source().clone(),
            snapshot.identities().to_vec(),
        )
        .unwrap();
        assert!(matches!(
            bare.with_history_facts(vec![batch], repo.protector()),
            Err(PersistenceError::CorruptStorage)
        ));
    }
}

#[test]
fn facts_only_read_has_indexed_access_and_distinguishes_missing_legacy_and_empty() {
    let dir = parent();
    let repo = create(&dir.path().join("archive"));
    assert!(
        repo.load_history_facts(&namespace("A"), SnapshotRevision::new(1))
            .unwrap()
            .is_none()
    );
    let value = snapshot(&repo, "A", 1, "empty", vec![]);
    publish(&repo, &value).unwrap();
    let read = repo
        .load_history_facts(&namespace("A"), SnapshotRevision::new(1))
        .unwrap()
        .unwrap();
    assert!(read.inventory_supported());
    assert!(read.batches().is_empty()); // inventory support does not prove coverage
    let connection = repo.lock().unwrap();
    for query in [
        "EXPLAIN QUERY PLAN SELECT source,proof FROM snapshots WHERE ns=?1 AND revision=?2",
        "EXPLAIN QUERY PLAN SELECT payload,seal,key_proof,record_proof FROM snapshot_manifests WHERE ns=?1 AND revision=?2",
        "EXPLAIN QUERY PLAN SELECT definition,payload,seal,key_proof,record_proof FROM history_batches WHERE ns=?1 AND revision=?2 ORDER BY definition",
    ] {
        let plan: String = connection
            .query_row(
                query,
                params![codec::encode(&namespace("A")).unwrap(), 1],
                |row| row.get(3),
            )
            .unwrap();
        assert!(
            plan.contains("SEARCH") && plan.contains("PRIMARY KEY"),
            "{plan}"
        );
    }
    connection.pragma_update(None, "user_version", 2).unwrap();
    drop(connection);
    let legacy = repo
        .load_history_facts(&namespace("A"), SnapshotRevision::new(1))
        .unwrap()
        .unwrap();
    assert!(!legacy.inventory_supported());
    assert!(legacy.batches().is_empty());
    assert_eq!(legacy.source(), value.source());
}

#[test]
fn facts_only_inventory_detects_deleted_or_modified_root_and_batch() {
    for mutation in [
        "DELETE FROM history_batches",
        "DELETE FROM snapshot_manifests",
        "UPDATE snapshot_manifests SET payload=x'7b7d'",
        "UPDATE snapshot_manifests SET record_proof=zeroblob(32)",
        "UPDATE history_batches SET definition=zeroblob(32)",
    ] {
        let dir = parent();
        let repo = create(&dir.path().join("archive"));
        publish(&repo, &candidate(&repo, 1, "fact-one")).unwrap();
        repo.lock().unwrap().execute_batch(mutation).unwrap();
        assert!(
            matches!(
                repo.load_history_facts(&namespace("A"), SnapshotRevision::new(1)),
                Err(PersistenceError::CorruptStorage)
            ),
            "{mutation}"
        );
    }
}

#[test]
fn batch_order_is_canonical_and_duplicate_definitions_are_rejected() {
    let dir = parent();
    let repo = create(&dir.path().join("archive"));
    let value = candidate(&repo, 1, "fact-one");
    let one = value.history_facts()[0].clone();
    let mut payload = one.payload.clone();
    payload.plan_digest = Sha256Digest([6; 32]);
    let two = HistoryFactBatch::create(payload, value.source(), repo.protector()).unwrap();
    assert!(matches!(
        value
            .clone()
            .with_history_facts(vec![one.clone(), one.clone()], repo.protector()),
        Err(PersistenceError::CorruptStorage)
    ));
    let normal = value
        .clone()
        .with_history_facts(vec![two.clone(), one.clone()], repo.protector())
        .unwrap();
    let reversed = value
        .with_history_facts(vec![one, two], repo.protector())
        .unwrap();
    assert_eq!(normal, reversed);
    publish(&repo, &normal).unwrap();
    assert_eq!(
        repo.load_current_snapshot(&namespace("A"))
            .unwrap()
            .unwrap(),
        normal
    );
}

#[test]
fn manifest_cannot_be_replayed_between_revisions() {
    let dir = parent();
    let repo = create(&dir.path().join("archive"));
    let first = candidate(&repo, 1, "first");
    let second = candidate(&repo, 2, "second");
    publish(&repo, &first).unwrap();
    publish(&repo, &second).unwrap();
    repo.lock().unwrap().execute_batch("UPDATE snapshot_manifests SET (seal,key_proof,record_proof)=(SELECT seal,key_proof,record_proof FROM snapshot_manifests WHERE revision=1) WHERE revision=2").unwrap();
    assert!(matches!(
        repo.load_current_snapshot(&namespace("A")),
        Err(PersistenceError::CorruptStorage)
    ));
    assert_eq!(
        repo.load_snapshot(&namespace("A"), SnapshotRevision::new(1))
            .unwrap()
            .unwrap(),
        first
    );
}

#[test]
fn logical_fact_v1_v2_are_explicit_and_v1_bytes_remain_unchanged() {
    let dir = parent();
    let repo = create(&dir.path().join("archive"));
    let value = candidate(&repo, 1, "codec-versions");
    let original = value.history_facts()[0].payload.clone();
    let encoded = serde_json::to_value(&original).unwrap();
    assert_eq!(
        encoded["facts"][0]["entity"],
        serde_json::to_value(original.facts[0].entity.as_ref().unwrap()).unwrap()
    );
    let decoded: HistoryBatchPayload = serde_json::from_value(encoded.clone()).unwrap();
    assert!(serde_json::to_value(decoded).unwrap() == encoded);
    let mut nullable = original.clone();
    nullable.facts[0].entity = None;
    nullable.facts[0].assessment = HistoryAssessment::Indeterminate {
        reason: HistoryIndeterminate::UnqualifiedCorrelation,
    };
    assert!(HistoryFactBatch::create(nullable.clone(), value.source(), repo.protector()).is_err());
    nullable.format = 2;
    let batch =
        HistoryFactBatch::create(nullable.clone(), value.source(), repo.protector()).unwrap();
    let snapshot = value
        .clone()
        .with_history_facts(vec![batch], repo.protector())
        .unwrap();
    publish(&repo, &snapshot).unwrap();
    assert_eq!(
        repo.load_snapshot(&namespace("A"), SnapshotRevision::new(1))
            .unwrap()
            .unwrap(),
        snapshot
    );
    nullable.format = 3;
    assert!(HistoryFactBatch::create(nullable, value.source(), repo.protector()).is_err());
}
