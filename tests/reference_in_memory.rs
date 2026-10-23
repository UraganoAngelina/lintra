//! Real provider, synthetic byte keys; no normative catalogue inferred.
use lintra::*;
use std::sync::Arc;
fn key(id: &str, version: &str) -> DatasetKey {
    DatasetKey {
        id: DatasetId(id.into()),
        version: DatasetVersion(version.into()),
    }
}
#[test]
fn exact_membership_preserves_all_bytes_and_handles_duplicate_entries() {
    let mut provider = InMemoryReferenceDataProvider::new();
    let key = key("synthetic", "1");
    provider
        .insert(
            key.clone(),
            [
                b"001 ".to_vec(),
                b"001 ".to_vec(),
                b"Ab".to_vec(),
                vec![0, 255],
                vec![],
            ],
        )
        .unwrap();
    let lookup = provider.load(&key).unwrap();
    assert_eq!(lookup.key(), &key);
    for value in [b"001 ".as_slice(), b"Ab", &[0, 255], b""] {
        assert!(lookup.contains(value));
    }
    for value in [b"001".as_slice(), b"1", b"ab", b"001  ", &[0, 254]] {
        assert!(!lookup.contains(value));
    }
}
#[test]
fn versions_are_isolated_and_never_fall_back_to_another_dataset() {
    let mut provider = InMemoryReferenceDataProvider::new();
    let first = key("codes", "1");
    let second = key("codes", "2");
    provider.insert(first.clone(), [b"01".to_vec()]).unwrap();
    provider.insert(second.clone(), [b"02".to_vec()]).unwrap();
    assert!(provider.load(&first).unwrap().contains(b"01"));
    assert!(!provider.load(&first).unwrap().contains(b"02"));
    assert!(provider.load(&second).unwrap().contains(b"02"));
    for missing in [key("codes", "3"), key("Codes", "1"), key("other", "1")] {
        assert!(
            matches!(provider.load(&missing), Err(ReferenceDataError::Unavailable { key }) if key == missing)
        );
    }
}
#[test]
fn duplicate_registration_preserves_the_original_immutable_handle() {
    let mut provider = InMemoryReferenceDataProvider::new();
    let key = key("codes", "1");
    provider.insert(key.clone(), [b"01".to_vec()]).unwrap();
    let first = provider.load(&key).unwrap();
    let forbidden = std::iter::from_fn(|| -> Option<Vec<u8>> {
        panic!("duplicate entries must not be consumed")
    });
    assert!(
        matches!(provider.insert(key.clone(), forbidden), Err(ReferenceDataError::DuplicateKey { key: found }) if found == key)
    );
    let after = provider.load(&key).unwrap();
    assert!(Arc::ptr_eq(&first, &after));
    assert!(after.contains(b"01"));
}
#[test]
fn explicitly_empty_dataset_is_distinct_from_an_unavailable_version() {
    let mut provider = InMemoryReferenceDataProvider::default();
    let key = key("empty", "1");
    assert!(matches!(
        provider.load(&key),
        Err(ReferenceDataError::Unavailable { .. })
    ));
    provider.insert(key.clone(), []).unwrap();
    let lookup = provider.load(&key).unwrap();
    assert!(!lookup.contains(b""));
    assert!(!lookup.contains(b"01"));
    assert_eq!(lookup.key(), &key);
}
#[test]
fn direct_load_and_registration_validate_identity_before_consuming_entries() {
    let mut provider = InMemoryReferenceDataProvider::new();
    for invalid in [
        key("", "1"),
        key("codes", ""),
        key(" codes", "1"),
        key("codes", "1 "),
    ] {
        let entries = std::iter::from_fn(|| -> Option<Vec<u8>> {
            panic!("invalid registration consumed entries")
        });
        assert!(
            matches!(provider.insert(invalid.clone(), entries), Err(ReferenceDataError::InvalidKey { key }) if key == invalid)
        );
        assert!(
            matches!(provider.load(&invalid), Err(ReferenceDataError::InvalidKey { key }) if key == invalid)
        );
    }
    provider
        .insert(key("codes", "1"), [b"01".to_vec()])
        .unwrap();
    assert!(provider.load(&key("codes", "1")).unwrap().contains(b"01"));
}
#[test]
fn preparation_uses_real_provider_and_prepared_handles_outlive_the_registry() {
    let first = key("codes", "1");
    let second = key("codes", "2");
    let mut provider = InMemoryReferenceDataProvider::new();
    provider.insert(first.clone(), [b"01".to_vec()]).unwrap();
    provider.insert(second.clone(), [b"02".to_vec()]).unwrap();
    let a = provider.load(&first).unwrap();
    let b = provider.load(&first).unwrap();
    assert!(Arc::ptr_eq(&a, &b));
    let prepared =
        prepare_reference_data(&[second.clone(), first.clone(), second.clone()], &provider)
            .unwrap();
    assert_eq!(prepared.keys(), [second.clone(), first.clone()]);
    drop(provider);
    for _ in 0..1000 {
        assert!(prepared.get(&first).unwrap().contains(b"01"));
        assert!(!prepared.get(&first).unwrap().contains(b"02"));
        assert!(prepared.get(&second).unwrap().contains(b"02"));
    }
    assert!(a.contains(b"01"));
}
