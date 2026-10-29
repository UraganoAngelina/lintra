//! A deterministic transaction fake. Reopening shares its committed image;
//! this simulates a repository restart, NOT process/disk durability or recovery.
use super::*;
use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

#[derive(Default)]
struct NamespaceState {
    snapshots: Vec<HistoricalSnapshot>,
    latest: HashMap<PersistentRecordIdentity, StoredIdentity>,
    submissions: HashMap<String, SourceSubmission>,
}
impl NamespaceState {
    fn revision(&self) -> Option<SnapshotRevision> {
        self.snapshots.last().map(HistoricalSnapshot::revision)
    }
}
#[derive(Default)]
struct CommittedImage {
    namespaces: HashMap<PersistenceNamespace, NamespaceState>,
}
/// Explicit in-process backing for tests. Uncommitted staging is never part of
/// this image. Clone/reopen preserves committed facts without claiming a WAL.
#[derive(Clone, Default)]
pub struct InMemoryPersistenceImage(Arc<Mutex<CommittedImage>>);
impl InMemoryPersistenceImage {
    fn lock(&self) -> Result<MutexGuard<'_, CommittedImage>, PersistenceError> {
        // A poisoned lock is a technical failure. Never recover a possibly
        // partially mutated image by silently ignoring the poison.
        self.0
            .lock()
            .map_err(|_| PersistenceError::StorageUnavailable)
    }
}
#[derive(Clone, Default)]
pub struct InMemoryPersistenceRepository {
    image: InMemoryPersistenceImage,
}
impl InMemoryPersistenceRepository {
    pub fn open(image: InMemoryPersistenceImage) -> Self {
        Self { image }
    }
    pub fn image(&self) -> InMemoryPersistenceImage {
        self.image.clone()
    }
}
fn generation(
    image: &CommittedImage,
    namespace: &PersistenceNamespace,
) -> Option<SnapshotRevision> {
    image
        .namespaces
        .get(namespace)
        .and_then(NamespaceState::revision)
}
fn check_generation(
    expected: Option<SnapshotRevision>,
    actual: Option<SnapshotRevision>,
) -> Result<(), PersistenceError> {
    if expected != actual {
        // The public error retains None rather than inventing revision zero.
        return Err(PersistenceError::GenerationConflict { expected, actual });
    }
    Ok(())
}
impl PersistentContextRepository for InMemoryPersistenceRepository {
    fn lookup_identity(
        &self,
        namespace: &PersistenceNamespace,
        identity: &PersistentRecordIdentity,
    ) -> Result<Option<StoredIdentity>, PersistenceError> {
        if identity.namespace() != namespace {
            return Err(PersistenceError::NamespaceMismatch);
        }
        Ok(self
            .image
            .lock()?
            .namespaces
            .get(namespace)
            .and_then(|s| s.latest.get(identity))
            .cloned())
    }
    fn load_current_snapshot(
        &self,
        namespace: &PersistenceNamespace,
    ) -> Result<Option<HistoricalSnapshot>, PersistenceError> {
        Ok(self
            .image
            .lock()?
            .namespaces
            .get(namespace)
            .and_then(|s| s.snapshots.last())
            .cloned())
    }
    fn load_snapshot(
        &self,
        namespace: &PersistenceNamespace,
        revision: SnapshotRevision,
    ) -> Result<Option<HistoricalSnapshot>, PersistenceError> {
        Ok(self
            .image
            .lock()?
            .namespaces
            .get(namespace)
            .and_then(|s| {
                s.snapshots
                    .iter()
                    .find(|snapshot| snapshot.revision() == revision)
            })
            .cloned())
    }
    fn load_submission(
        &self,
        namespace: &PersistenceNamespace,
        submission_id: &str,
    ) -> Result<Option<SourceSubmission>, PersistenceError> {
        Ok(self
            .image
            .lock()?
            .namespaces
            .get(namespace)
            .and_then(|s| s.submissions.get(submission_id))
            .cloned())
    }
    fn current_generation(
        &self,
        ns: &PersistenceNamespace,
    ) -> Result<Option<SnapshotRevision>, PersistenceError> {
        Ok(generation(&*self.image.lock()?, ns))
    }
    fn load_submission_page(
        &self,
        ns: &PersistenceNamespace,
        after: Option<SnapshotRevision>,
    ) -> Result<Vec<(SnapshotRevision, SourceSubmission)>, PersistenceError> {
        Ok(self
            .image
            .lock()?
            .namespaces
            .get(ns)
            .map(|state| {
                state
                    .snapshots
                    .iter()
                    .filter(|s| after.is_none_or(|r| s.revision() > r))
                    .take(128)
                    .map(|s| (s.revision(), s.source().clone()))
                    .collect()
            })
            .unwrap_or_default())
    }
    fn begin_transaction(
        &self,
        namespace: &PersistenceNamespace,
        expected_generation: Option<SnapshotRevision>,
    ) -> Result<Box<dyn PersistenceTransaction>, PersistenceError> {
        check_generation(
            expected_generation,
            generation(&*self.image.lock()?, namespace),
        )?;
        Ok(Box::new(MemoryTransaction {
            image: self.image.clone(),
            namespace: namespace.clone(),
            expected: expected_generation,
            staged: None,
        }))
    }
}
struct MemoryTransaction {
    image: InMemoryPersistenceImage,
    namespace: PersistenceNamespace,
    expected: Option<SnapshotRevision>,
    staged: Option<HistoricalSnapshot>,
}
impl PersistenceTransaction for MemoryTransaction {
    fn stage_snapshot(&mut self, snapshot: &HistoricalSnapshot) -> Result<(), PersistenceError> {
        // A failed replacement must not leave an older candidate committable.
        // The caller can explicitly stage again, or consume abort/drop.
        self.staged = None;
        if snapshot.namespace() != &self.namespace {
            return Err(PersistenceError::NamespaceMismatch);
        }
        let next = match self.expected {
            Some(base) => base
                .next()
                .map_err(|_| PersistenceError::RevisionOverflow)?,
            None => SnapshotRevision::new(1),
        };
        if snapshot.revision() != next {
            return Err(PersistenceError::InvalidRevision);
        }
        // A clone shares immutable record metadata. Staging never acquires or
        // mutates committed storage, including when it replaces earlier staging.
        self.staged = Some(snapshot.clone());
        Ok(())
    }
    fn commit(self: Box<Self>) -> Result<CommitReceipt, PersistenceError> {
        let snapshot = self.staged.ok_or(PersistenceError::NothingStaged)?;
        let mut image = self.image.lock()?;
        check_generation(self.expected, generation(&image, &self.namespace))?;
        if image.namespaces.get(&self.namespace).is_some_and(|s| {
            s.submissions
                .contains_key(snapshot.source().submission_id())
        }) {
            return Err(PersistenceError::DuplicateSubmission);
        }
        // Build the complete replacement privately. Readers hold the same lock;
        // the single final insertion publishes snapshot, index and audit together.
        // Allocation panic leaves the old image untouched and poisons the mutex.
        let old = image.namespaces.get(&self.namespace);
        let mut replacement = NamespaceState {
            snapshots: old.map(|s| s.snapshots.clone()).unwrap_or_default(),
            latest: old.map(|s| s.latest.clone()).unwrap_or_default(),
            submissions: old.map(|s| s.submissions.clone()).unwrap_or_default(),
        };
        for record in snapshot.identities() {
            replacement
                .latest
                .insert(record.identity().clone(), record.clone());
        }
        replacement.submissions.insert(
            snapshot.source().submission_id().to_owned(),
            snapshot.source().clone(),
        );
        let revision = snapshot.revision();
        replacement.snapshots.push(snapshot);
        image.namespaces.insert(self.namespace, replacement);
        Ok(CommitReceipt { revision })
    }
    fn abort(self: Box<Self>) -> Result<(), PersistenceError> {
        // Consuming self drops private staging even if storage is unavailable.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn revision_exhaustion_and_poisoned_storage_are_technical_failures() {
        let source: SourceSubmission = serde_json::from_slice(include_bytes!(
            "../../tests/fixtures/persistence/source-submission.json"
        ))
        .unwrap();
        let namespace = source.namespace().clone();
        let candidate = HistoricalSnapshot::new(SnapshotRevision::new(1), source, vec![]).unwrap();
        // Seed the boundary in the private transaction fake, without exposing a
        // production API that could forge a committed generation or receipt.
        let mut transaction = MemoryTransaction {
            image: InMemoryPersistenceImage::default(),
            namespace: namespace.clone(),
            expected: Some(SnapshotRevision::new(u64::MAX)),
            staged: None,
        };
        assert_eq!(
            transaction.stage_snapshot(&candidate).unwrap_err(),
            PersistenceError::RevisionOverflow
        );
        assert!(transaction.staged.is_none());
        let image = InMemoryPersistenceImage::default();
        let fault = image.clone();
        assert!(
            std::thread::spawn(move || {
                let _guard = fault.0.lock().unwrap();
                panic!("synthetic storage fault");
            })
            .join()
            .is_err()
        );
        let repo = InMemoryPersistenceRepository::open(image.clone());
        assert_eq!(
            repo.load_current_snapshot(&namespace).unwrap_err(),
            PersistenceError::StorageUnavailable
        );
        // Aborting private staging needs no access to the failed storage.
        Box::new(MemoryTransaction {
            image,
            namespace,
            expected: None,
            staged: Some(candidate),
        })
        .abort()
        .unwrap();
    }
}
