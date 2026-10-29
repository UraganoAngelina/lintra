//! Prepared immutable lookup handles; the byte-key policy belongs to the schema.
use super::provider::{ReferenceDataError, ReferenceDataProvider, validate_key};
pub use crate::schema::compiled::DatasetKey;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Versioned membership over exact bytes. Implementations must not silently
/// trim, fold case or strip zeros; ordinary lookup requires no file opening.
pub trait LookupSet: Send + Sync {
    fn contains(&self, value: &[u8]) -> bool;
    fn key(&self) -> &DatasetKey;
    /// Optional read-only diagnostic enumeration. Third-party providers retain
    /// an explicit unsupported state instead of pretending their domain is empty.
    fn page_after(&self, _after: Option<&[u8]>, _cancelled: &AtomicBool) -> Option<LookupPage> {
        None
    }
}
/// Lexicographically ordered exact dataset bytes; no normalization or labels.
/// At most64 values, plus a continuation, are copied from the existing lookup.
pub struct LookupPage {
    pub values: Vec<Vec<u8>>,
    pub next: Option<Vec<u8>>,
}

/// Shared immutable storage, not a trust decision. Only preparation exposes it.
pub(super) struct ByteLookupSet {
    key: DatasetKey,
    values: HashSet<Vec<u8>>,
}
impl ByteLookupSet {
    pub(super) fn new(key: DatasetKey, values: HashSet<Vec<u8>>) -> Self {
        Self { key, values }
    }
}
impl LookupSet for ByteLookupSet {
    fn contains(&self, value: &[u8]) -> bool {
        self.values.contains(value)
    }
    fn key(&self) -> &DatasetKey {
        &self.key
    }
    fn page_after(&self, after: Option<&[u8]>, cancelled: &AtomicBool) -> Option<LookupPage> {
        // Bounded ordered selection: O(N log64) per explicit browser page,
        // O(64) scratch references, no second permanent dataset index/copy.
        // UI callers execute this on a worker, never on the event-loop thread.
        let mut selected = BTreeSet::new();
        for value in &self.values {
            if cancelled.load(Ordering::Relaxed) {
                return None;
            }
            if after.is_some_and(|cursor| value.as_slice() <= cursor) {
                continue;
            }
            selected.insert(value);
            if selected.len() > 65 {
                selected.pop_last();
            }
        }
        let more = selected.len() > 64;
        if more {
            selected.pop_last();
        }
        let values: Vec<Vec<u8>> = selected.into_iter().cloned().collect();
        let next = if more { values.last().cloned() } else { None };
        Some(LookupPage { values, next })
    }
}

/// Built only by successful preparation. It retains lookup sets, not a provider,
/// so field validation cannot accidentally reopen files through this context.
pub struct PreparedReferenceData {
    sets: HashMap<DatasetKey, Arc<dyn LookupSet>>,
    keys: Vec<DatasetKey>,
}
impl PreparedReferenceData {
    pub fn get(&self, key: &DatasetKey) -> Option<&dyn LookupSet> {
        self.sets.get(key).map(Arc::as_ref)
    }
    /// First-declaration order, independent of randomized map iteration.
    pub fn keys(&self) -> &[DatasetKey] {
        &self.keys
    }
    pub fn len(&self) -> usize {
        self.sets.len()
    }
    pub fn is_empty(&self) -> bool {
        self.sets.is_empty()
    }
}

/// All dependencies are prepared or no context is returned. Exact repeated keys
/// share one load; distinct versions remain distinct without a latest fallback.
/// Service wiring that invokes this before input parsing belongs to T-P11-06.
pub fn prepare_reference_data(
    dependencies: &[DatasetKey],
    provider: &dyn ReferenceDataProvider,
) -> Result<PreparedReferenceData, ReferenceDataError> {
    // Reject malformed identities before the first provider side effect; this
    // validates spelling and never normalizes dataset IDs or membership bytes.
    for key in dependencies {
        validate_key(key)?;
    }
    let mut sets = HashMap::with_capacity(dependencies.len());
    let mut keys = Vec::new();
    for key in dependencies {
        if sets.contains_key(key) {
            continue;
        }
        let set = provider.load(key)?;
        if set.key() != key {
            return Err(ReferenceDataError::KeyMismatch {
                requested: key.clone(),
                actual: set.key().clone(),
            });
        }
        sets.insert(key.clone(), set);
        keys.push(key.clone());
    }
    Ok(PreparedReferenceData { sets, keys })
}

#[cfg(test)]
mod diagnostic_tests {
    use super::*;
    #[test]
    fn bounded_dataset_pages_cover_exact_keys_and_cancel_without_fake_empty() {
        let values: HashSet<_> = (0..300)
            .rev()
            .map(|i| format!("{i:04}").into_bytes())
            .collect();
        let set = ByteLookupSet::new(
            DatasetKey {
                id: crate::DatasetId("synthetic".into()),
                version: crate::DatasetVersion("v1".into()),
            },
            values,
        );
        let cancelled = AtomicBool::new(false);
        let mut cursor = None;
        let mut all = Vec::new();
        loop {
            let page = set.page_after(cursor.as_deref(), &cancelled).unwrap();
            assert!(page.values.len() <= 64);
            all.extend(page.values);
            cursor = page.next;
            if cursor.is_none() {
                break;
            }
        }
        assert_eq!(
            all,
            (0..300)
                .map(|i| format!("{i:04}").into_bytes())
                .collect::<Vec<_>>()
        );
        assert!(set.contains(b"0001"));
        assert!(!set.contains(b"1"));
        cancelled.store(true, Ordering::Relaxed);
        assert!(set.page_after(None, &cancelled).is_none());
    }
}
