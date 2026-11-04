//! Synthetic storage privacy/closed-copy qualification; no production restore API.
use lintra::*;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
#[path = "support/persistence_profiles.rs"]
mod profiles;
use profiles::*;

fn has_sentinel(bytes: &[u8]) -> bool {
    [
        RAW_MARKER.as_slice(),
        CHANGED_MARKER.as_slice(),
        SOURCE_MARKER,
        b"7700000077000000",
        &[0xab; 32],
    ]
    .iter()
    .any(|needle| {
        let hex = needle
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        bytes.windows(needle.len()).any(|w| w == *needle)
            || bytes
                .windows(hex.len())
                .any(|w| w.eq_ignore_ascii_case(hex.as_bytes()))
    })
}
fn scan(root: &Path) {
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        assert!(path.is_file(), "unexpected non-file in synthetic storage");
        assert!(
            !has_sentinel(&fs::read(path).unwrap()),
            "synthetic raw sentinel reached storage"
        );
    }
}

#[test]
fn both_shapes_keep_raw_values_out_of_storage_audit_and_diagnostics() {
    for shape in [Shape::Sioc, Shape::Cad] {
        let dir =
            tempfile::tempdir_in(Path::new(env!("CARGO_MANIFEST_DIR")).join("target")).unwrap();
        let root = dir.path().join("archive");
        let repo = SQLitePersistenceAdapter::create(&root, Some(KEY_ID), &Provider(0xab)).unwrap();
        let fixture = Fixture::new(shape, 20);
        let one = fixture.snapshot(repo.protector(), "synthetic-a", 1, false);
        publish(&repo, &one).unwrap();
        scan(&root);
        let two = fixture.snapshot(repo.protector(), "synthetic-a", 2, true);
        assert_eq!(
            one.identities()[0].identity(),
            two.identities()[0].identity()
        );
        assert_ne!(
            one.identities()[0].record_fingerprint(),
            two.identities()[0].record_fingerprint()
        );
        publish(&repo, &two).unwrap();
        scan(&root);
        let audit = serde_json::to_vec(&serde_json::json!({
            "source":two.source(),"protection":repo.load_protection_audit().unwrap(),
            "local_repository_only":true
        }))
        .unwrap();
        assert!(!has_sentinel(&audit));
        let diagnostics = format!(
            "{one:?} {two:?} {:?} {}",
            PersistenceError::Protection(ProtectionError::KeyMismatch),
            ProtectionError::KeyMismatch
        );
        assert!(!has_sentinel(diagnostics.as_bytes()));
        assert_eq!(
            repo.load_snapshot(&namespace("synthetic-a"), SnapshotRevision::new(1))
                .unwrap(),
            Some(one)
        );
        assert_eq!(
            repo.load_current_snapshot(&namespace("synthetic-a"))
                .unwrap(),
            Some(two)
        );
        assert_eq!(
            repo.current_generation(&namespace("synthetic-b")).unwrap(),
            None
        );
        drop(repo);
        scan(&root);
    }
    // A real positive control prevents a vacuous scanner that accepts everything.
    assert!(has_sentinel(RAW_MARKER));
    assert!(has_sentinel(SOURCE_MARKER));
    let encoded = SOURCE_MARKER
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    assert!(has_sentinel(encoded.as_bytes()));
    assert!(!has_sentinel(b"synthetic-count-only-report"));
}

#[test]
fn closed_copy_restores_exact_history_and_wrong_key_or_damage_never_become_empty() {
    let dir = tempfile::tempdir_in(Path::new(env!("CARGO_MANIFEST_DIR")).join("target")).unwrap();
    let root = dir.path().join("original");
    let repo = SQLitePersistenceAdapter::create(&root, Some(KEY_ID), &Provider(0xab)).unwrap();
    let fixture = Fixture::new(Shape::Cad, 10);
    let one = fixture.snapshot(repo.protector(), "synthetic-a", 1, false);
    let two = fixture.snapshot(repo.protector(), "synthetic-a", 2, true);
    publish(&repo, &one).unwrap();
    publish(&repo, &two).unwrap();
    let expected_audit = repo.load_protection_audit().unwrap();
    drop(repo); // A closed, quiescent DB; never copy live database/journal files.
    let backup = fs::read(root.join("lintra.db")).unwrap();
    let hash = Sha256::digest(&backup);
    assert!(!has_sentinel(&backup));
    let mut damaged = backup.clone();
    damaged[0] ^= 1;
    assert_ne!(Sha256::digest(&damaged), hash); // Reject before restore, not repair.

    let restored = dir.path().join("isolated-restore");
    // Only establishes the adapter's native private-root ACL in a disposable test
    // sandbox. Explicitly replace its closed synthetic DB with the qualified copy;
    // this is not a product command, overwrite of active storage, or key fallback.
    drop(SQLitePersistenceAdapter::create(&restored, Some(KEY_ID), &Provider(0xab)).unwrap());
    fs::write(restored.join("lintra.db"), &backup).unwrap();
    assert_eq!(
        Sha256::digest(fs::read(restored.join("lintra.db")).unwrap()),
        hash
    );
    assert!(matches!(
        SQLitePersistenceAdapter::open(&restored, Some(KEY_ID), &Provider(0xcd)),
        Err(PersistenceError::Protection(ProtectionError::KeyMismatch))
    ));
    let opened = SQLitePersistenceAdapter::open(&restored, Some(KEY_ID), &Provider(0xab)).unwrap();
    assert_eq!(
        serde_json::to_value(opened.load_protection_audit().unwrap()).unwrap(),
        serde_json::to_value(expected_audit).unwrap()
    );
    assert_eq!(
        opened
            .load_snapshot(&namespace("synthetic-a"), SnapshotRevision::new(1))
            .unwrap(),
        Some(one)
    );
    assert_eq!(
        opened
            .load_current_snapshot(&namespace("synthetic-a"))
            .unwrap(),
        Some(two)
    );
    drop(opened);
    assert_eq!(fs::read(root.join("lintra.db")).unwrap(), backup);
    fs::write(restored.join("lintra.db"), damaged).unwrap();
    assert!(SQLitePersistenceAdapter::open(&restored, Some(KEY_ID), &Provider(0xab)).is_err());
    assert_eq!(fs::read(root.join("lintra.db")).unwrap(), backup);
}
