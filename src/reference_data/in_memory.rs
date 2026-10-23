//! Explicit synthetic datasets for tests and callers providing their own bytes.
//! No built-in normative entries, normalization or implicit latest version.
use super::{
    dataset::{ByteLookupSet, DatasetKey, LookupSet},
    provider::{ReferenceDataError, ReferenceDataProvider, validate_key},
};
use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

#[derive(Default)]
pub struct InMemoryReferenceDataProvider {
    sets: HashMap<DatasetKey, Arc<dyn LookupSet>>,
}
impl InMemoryReferenceDataProvider {
    /// An empty registry is not a loaded dataset: absent versions still fail.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one immutable version. Reject duplicate identities before
    /// consuming entries so an existing snapshot can never be overwritten.
    pub fn insert(
        &mut self,
        key: DatasetKey,
        values: impl IntoIterator<Item = Vec<u8>>,
    ) -> Result<(), ReferenceDataError> {
        validate_key(&key)?;
        match self.sets.entry(key.clone()) {
            Entry::Occupied(_) => Err(ReferenceDataError::DuplicateKey { key }),
            Entry::Vacant(entry) => {
                entry.insert(Arc::new(ByteLookupSet::new(
                    key,
                    values.into_iter().collect(),
                )));
                Ok(())
            }
        }
    }
}
impl ReferenceDataProvider for InMemoryReferenceDataProvider {
    fn load(&self, key: &DatasetKey) -> Result<Arc<dyn LookupSet>, ReferenceDataError> {
        validate_key(key)?;
        self.sets
            .get(key)
            .cloned()
            .ok_or_else(|| ReferenceDataError::Unavailable { key: key.clone() })
    }
}
