//! Historical repository ports, separate from the ephemeral two-pass ContextStore.
//!
//! These contracts store protected identities and audit facts, not original
//! records. They do not approve a submission or apply a clinical history policy.
use crate::{
    Fingerprint, PersistenceNamespace, PersistentRecordIdentity, SnapshotRevision, SourceSubmission,
};
use std::{collections::HashSet, error::Error, fmt, sync::Arc};

mod memory;
pub use memory::{InMemoryPersistenceImage, InMemoryPersistenceRepository};

/// Technical storage/structural failures. Messages deliberately omit tenant,
/// submission identifiers, fingerprints and record contents.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PersistenceError {
    NamespaceMismatch,
    DuplicateIdentity,
    DuplicateSubmission,
    InvalidRevision,
    RevisionOverflow,
    GenerationConflict {
        expected: Option<SnapshotRevision>,
        actual: Option<SnapshotRevision>,
    },
    NothingStaged,
    StorageUnavailable,
}
impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NamespaceMismatch => "persistence namespace mismatch",
            Self::DuplicateIdentity => "duplicate identity in staged snapshot",
            Self::DuplicateSubmission => "submission identifier already committed",
            Self::InvalidRevision => "snapshot revision must follow the expected generation",
            Self::RevisionOverflow => "snapshot revision exhausted",
            Self::GenerationConflict { .. } => "persistence generation conflict",
            Self::NothingStaged => "no snapshot staged",
            Self::StorageUnavailable => "persistence storage unavailable",
        })
    }
}
impl Error for PersistenceError {}

/// A protected key and its protected record fingerprint. No raw/projection
/// payload is accepted by this minimal port; protection is qualified by PERSIST-04.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredIdentity {
    identity: PersistentRecordIdentity,
    record_fingerprint: Fingerprint,
}
impl StoredIdentity {
    pub fn new(identity: PersistentRecordIdentity, record_fingerprint: Fingerprint) -> Self {
        Self {
            identity,
            record_fingerprint,
        }
    }
    pub fn identity(&self) -> &PersistentRecordIdentity {
        &self.identity
    }
    pub fn record_fingerprint(&self) -> &Fingerprint {
        &self.record_fingerprint
    }
}

/// An immutable supplied snapshot, not an algorithm that computes a cumulative
/// state. The future history service owns reconciliation and approval policy.
/// Record order is retained; index iteration never determines report ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalSnapshot {
    revision: SnapshotRevision,
    source: SourceSubmission,
    identities: Arc<[StoredIdentity]>,
}
impl HistoricalSnapshot {
    pub fn new(
        revision: SnapshotRevision,
        source: SourceSubmission,
        identities: Vec<StoredIdentity>,
    ) -> Result<Self, PersistenceError> {
        let mut keys = HashSet::with_capacity(identities.len());
        for record in &identities {
            if record.identity().namespace() != source.namespace() {
                return Err(PersistenceError::NamespaceMismatch);
            }
            if !keys.insert(record.identity()) {
                return Err(PersistenceError::DuplicateIdentity);
            }
        }
        Ok(Self {
            revision,
            source,
            identities: identities.into(),
        })
    }
    pub fn revision(&self) -> SnapshotRevision {
        self.revision
    }
    pub fn namespace(&self) -> &PersistenceNamespace {
        self.source.namespace()
    }
    pub fn source(&self) -> &SourceSubmission {
        &self.source
    }
    pub fn identities(&self) -> &[StoredIdentity] {
        &self.identities
    }
}

/// Successful atomic publication identifies the exact new generation. A receipt
/// says storage committed; it is not proof of regulatory validity or consent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitReceipt {
    revision: SnapshotRevision,
}
impl CommitReceipt {
    pub fn revision(&self) -> SnapshotRevision {
        self.revision
    }
}

/// Generic application port. No engine ContextStore dependency and no filesystem
/// discovery. All lookups are exact, namespace-scoped and read only.
pub trait PersistentContextRepository: Send + Sync {
    /// Latest committed occurrence, including identities no longer present in
    /// the current snapshot. Omission does not erase history or imply revocation.
    fn lookup_identity(
        &self,
        namespace: &PersistenceNamespace,
        identity: &PersistentRecordIdentity,
    ) -> Result<Option<StoredIdentity>, PersistenceError>;
    fn load_current_snapshot(
        &self,
        namespace: &PersistenceNamespace,
    ) -> Result<Option<HistoricalSnapshot>, PersistenceError>;
    fn load_snapshot(
        &self,
        namespace: &PersistenceNamespace,
        revision: SnapshotRevision,
    ) -> Result<Option<HistoricalSnapshot>, PersistenceError>;
    fn load_submission(
        &self,
        namespace: &PersistenceNamespace,
        submission_id: &str,
    ) -> Result<Option<SourceSubmission>, PersistenceError>;
    /// None is an empty namespace; no sentinel snapshot/revision is fabricated.
    /// expected_generation=None means exactly no previously committed snapshot.
    fn begin_transaction(
        &self,
        namespace: &PersistenceNamespace,
        expected_generation: Option<SnapshotRevision>,
    ) -> Result<Box<dyn PersistenceTransaction>, PersistenceError>;
}

/// Staged data remains invisible until consuming commit. Consuming abort or
/// dropping a transaction discards it; no partially staged state is published.
pub trait PersistenceTransaction: Send {
    /// Replaces staging; failure clears any previous candidate. It never changes
    /// committed storage, and a later successful explicit stage may retry.
    fn stage_snapshot(&mut self, snapshot: &HistoricalSnapshot) -> Result<(), PersistenceError>;
    fn commit(self: Box<Self>) -> Result<CommitReceipt, PersistenceError>;
    fn abort(self: Box<Self>) -> Result<(), PersistenceError>;
}
