//! Prepared immutable lookup handles; the byte-key policy belongs to the schema.
use super::provider::{ReferenceDataError, ReferenceDataProvider, validate_key};
pub use crate::schema::compiled::DatasetKey;
use std::{collections::HashMap, sync::Arc};

/// Versioned membership over exact bytes. Implementations must not silently
/// trim, fold case or strip zeros; ordinary lookup requires no file opening.
pub trait LookupSet: Send + Sync {
    fn contains(&self, value: &[u8]) -> bool;
    fn key(&self) -> &DatasetKey;
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
