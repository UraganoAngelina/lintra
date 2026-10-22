//! Contract-level preparation with explicit real byte sets and test-only I/O faults.
use lintra::*;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    sync::{Arc, Mutex},
};
fn key(id: &str, version: &str) -> DatasetKey {
    DatasetKey {
        id: DatasetId(id.into()),
        version: DatasetVersion(version.into()),
    }
}
struct Set {
    key: DatasetKey,
    values: HashSet<Vec<u8>>,
}
impl LookupSet for Set {
    fn contains(&self, value: &[u8]) -> bool {
        self.values.contains(value)
    }
    fn key(&self) -> &DatasetKey {
        &self.key
    }
}
fn set(key: DatasetKey, values: &[&[u8]]) -> Arc<dyn LookupSet> {
    Arc::new(Set {
        key,
        values: values.iter().map(|value| value.to_vec()).collect(),
    })
}
// This recording test implementation is not the production in-memory provider
// (T-P11-02) and never substitutes for a runtime dependency in ValidationService.
struct Provider {
    sets: HashMap<DatasetKey, Arc<dyn LookupSet>>,
    calls: Mutex<Vec<DatasetKey>>,
}
impl ReferenceDataProvider for Provider {
    fn load(&self, key: &DatasetKey) -> Result<Arc<dyn LookupSet>, ReferenceDataError> {
        self.calls.lock().unwrap().push(key.clone());
        self.sets
            .get(key)
            .cloned()
            .ok_or_else(|| ReferenceDataError::Unavailable { key: key.clone() })
    }
}
fn provider(sets: Vec<(DatasetKey, Arc<dyn LookupSet>)>) -> Provider {
    Provider {
        sets: sets.into_iter().collect(),
        calls: Mutex::new(vec![]),
    }
}
#[test]
fn preparation_loads_every_unique_dependency_once_and_lookup_never_calls_provider() {
    let first = key("codes", "v1");
    let second = key("codes", "v2");
    let other = key("other", "v1");
    let provider = provider(vec![
        (first.clone(), set(first.clone(), &[b"01"])),
        (second.clone(), set(second.clone(), &[b"02"])),
        (other.clone(), set(other.clone(), &[b"X"])),
    ]);
    let prepared = prepare_reference_data(
        &[second.clone(), first.clone(), second.clone(), other.clone()],
        &provider,
    )
    .unwrap();
    assert_eq!(
        prepared.keys(),
        [second.clone(), first.clone(), other.clone()]
    );
    assert_eq!(prepared.len(), 3);
    assert!(!prepared.is_empty());
    assert_eq!(
        *provider.calls.lock().unwrap(),
        [second.clone(), first.clone(), other]
    );
    for _ in 0..1000 {
        assert!(prepared.get(&first).unwrap().contains(b"01"));
        assert!(!prepared.get(&second).unwrap().contains(b"01"));
    }
    assert_eq!(provider.calls.lock().unwrap().len(), 3);
    assert!(prepared.get(&key("codes", "v3")).is_none());
}
#[test]
fn lookup_preserves_zeros_spaces_case_and_arbitrary_bytes() {
    let id = key("synthetic", "version-A");
    let provider = provider(vec![(
        id.clone(),
        set(id.clone(), &[b"001 ", b"Ab", &[0, 255]]),
    )]);
    let prepared = prepare_reference_data(std::slice::from_ref(&id), &provider).unwrap();
    let lookup = prepared.get(&id).unwrap();
    for value in [b"001 ".as_slice(), b"Ab", &[0, 255]] {
        assert!(lookup.contains(value));
    }
    for value in [b"001".as_slice(), b"1", b"ab", b"001  ", &[0, 254]] {
        assert!(!lookup.contains(value));
    }
    assert_eq!(lookup.key(), &id);
}
#[test]
fn missing_dependency_has_no_fallback_and_stops_preparation() {
    let first = key("first", "1");
    let missing = key("missing", "2");
    let after = key("after", "1");
    let provider = provider(vec![(first.clone(), set(first.clone(), &[b"X"]))]);
    let error = match prepare_reference_data(&[first.clone(), missing.clone(), after], &provider) {
        Ok(_) => panic!("missing must fail"),
        Err(error) => error,
    };
    assert!(matches!(error,ReferenceDataError::Unavailable{key} if key==missing));
    assert_eq!(*provider.calls.lock().unwrap(), [first, missing]);
}
#[test]
fn wrong_dataset_or_version_is_rejected_even_if_its_codes_would_match() {
    let requested = key("codes", "1");
    for actual in [key("other", "1"), key("codes", "2")] {
        let provider = provider(vec![(requested.clone(), set(actual.clone(), &[b"01"]))]);
        let error = match prepare_reference_data(std::slice::from_ref(&requested), &provider) {
            Ok(_) => panic!("wrong key must fail"),
            Err(error) => error,
        };
        assert!(
            matches!(error,ReferenceDataError::KeyMismatch{requested:found,actual:loaded} if found==requested&&loaded==actual)
        );
    }
}
#[test]
fn invalid_keys_are_rejected_before_any_provider_calls_and_empty_plan_is_explicit() {
    for invalid in [
        key("", "1"),
        key("codes", ""),
        key(" codes", "1"),
        key("codes", "1 "),
    ] {
        let provider = provider(vec![]);
        assert!(matches!(
            prepare_reference_data(&[key("valid", "1"), invalid], &provider),
            Err(ReferenceDataError::InvalidKey { .. })
        ));
        assert!(provider.calls.lock().unwrap().is_empty());
    }
    let provider = provider(vec![]);
    let empty = prepare_reference_data(&[], &provider).unwrap();
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
    assert!(empty.keys().is_empty());
    assert!(provider.calls.lock().unwrap().is_empty());
}
#[test]
fn provider_failure_keeps_the_actual_error_cause() {
    struct Broken;
    impl ReferenceDataProvider for Broken {
        fn load(&self, key: &DatasetKey) -> Result<Arc<dyn LookupSet>, ReferenceDataError> {
            Err(ReferenceDataError::Provider {
                key: key.clone(),
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "synthetic failure",
                )),
            })
        }
    }
    let error = match prepare_reference_data(&[key("codes", "1")], &Broken) {
        Ok(_) => panic!("load must fail"),
        Err(error) => error,
    };
    assert_eq!(
        error
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::PermissionDenied
    );
}
