//! Historical repository ports, separate from the ephemeral two-pass ContextStore.
//!
//! These contracts store protected identities and audit facts, not original
//! records. They do not approve a submission or apply a clinical history policy.
use crate::{
    Fingerprint, PersistenceNamespace, PersistentRecordIdentity, SnapshotRevision, SourceSubmission,
};
use std::{collections::HashSet, error::Error, fmt, sync::Arc};

mod memory;
mod protection;
pub use memory::{InMemoryPersistenceImage, InMemoryPersistenceRepository};
pub use protection::{
    IdentityProtectionScope, IdentityProtector, PersistenceKey, PersistenceKeyProvider,
    ProtectionAudit, ProtectionError,
};

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
    StorageUninitialized,
    StorageAlreadyExists,
    UnsupportedStorageVersion {
        found: i64,
    },
    StorageUpgradeRequired {
        found: i64,
        required: i64,
    },
    StorageUpgradeOutcomeUnknown,
    UnrecognizedStorage,
    CorruptStorage,
    UnsafeStorageLocation,
    UnsupportedStorageFilesystem,
    UnsupportedStorageConfiguration,
    StorageBusy,
    Protection(ProtectionError),
    CommitOutcomeUnknown {
        revision: SnapshotRevision,
    },
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
            Self::StorageUninitialized => "local persistence storage is not initialized",
            Self::StorageAlreadyExists => "local persistence storage already exists",
            Self::UnsupportedStorageVersion { .. } => "unsupported persistence storage version",
            Self::StorageUpgradeRequired { .. } => {
                "explicit storage upgrade is required for these metadata"
            }
            Self::StorageUpgradeOutcomeUnknown => {
                "storage upgrade outcome requires reopening and verification"
            }
            Self::UnrecognizedStorage => "unrecognized local persistence storage",
            Self::CorruptStorage => "local persistence storage is corrupt or inconsistent",
            Self::UnsafeStorageLocation => {
                "unsafe persistence root, permissions, or filesystem boundary"
            }
            Self::UnsupportedStorageFilesystem => "unsupported local persistence filesystem",
            Self::UnsupportedStorageConfiguration => {
                "required persistence durability configuration is unavailable"
            }
            Self::StorageBusy => "local persistence storage is busy",
            Self::Protection(_) => "persistence protection preflight or provenance failed",
            Self::CommitOutcomeUnknown { .. } => {
                "commit outcome requires verification; rollback is not confirmed"
            }
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
    pub(crate) protection_evidence: crate::domain::ProtectionEvidence,
}
impl StoredIdentity {
    pub fn new(identity: PersistentRecordIdentity, record_fingerprint: Fingerprint) -> Self {
        Self {
            identity,
            record_fingerprint,
            protection_evidence: crate::domain::ProtectionEvidence::default(),
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
    /// Application-only rebinding of the publication metadata/revision. The
    /// immutable protected identities remain shared; no O(n) record copy.
    pub(crate) fn with_source(
        &self,
        revision: SnapshotRevision,
        source: SourceSubmission,
    ) -> Result<Self, PersistenceError> {
        if source.namespace() != self.namespace() {
            return Err(PersistenceError::NamespaceMismatch);
        }
        Ok(Self {
            revision,
            source,
            identities: self.identities.clone(),
        })
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
    /// A lost commit response is not evidence of rollback. Resolve by immutable
    /// submission identity and exact snapshot value; never retry as a new run.
    /// The rare successful recovery may materialize one snapshot, not history.
    fn resolve_commit(
        &self,
        candidate: &HistoricalSnapshot,
    ) -> Result<CommitResolution, PersistenceError> {
        let Some(source) =
            self.load_submission(candidate.namespace(), candidate.source().submission_id())?
        else {
            return Ok(CommitResolution::NotPublished);
        };
        if source != *candidate.source() {
            return Ok(CommitResolution::ConflictingSubmission);
        }
        match self.load_snapshot(candidate.namespace(), candidate.revision())? {
            Some(snapshot) if snapshot == *candidate => {
                Ok(CommitResolution::Published(CommitReceipt {
                    revision: candidate.revision(),
                }))
            }
            _ => Ok(CommitResolution::ConflictingSubmission),
        }
    }
    /// Exact committed generation without materializing records. Implementations
    /// may override this compatibility default with a metadata-only query.
    fn current_generation(
        &self,
        namespace: &PersistenceNamespace,
    ) -> Result<Option<SnapshotRevision>, PersistenceError> {
        Ok(self.load_current_snapshot(namespace)?.map(|s| s.revision()))
    }
    /// At most 128 source metadata entries after an exclusive revision cursor,
    /// in ascending revision order. No identity payloads are loaded. A default
    /// fails closed so existing third-party ports never invent an empty history.
    fn load_submission_page(
        &self,
        _namespace: &PersistenceNamespace,
        _after: Option<SnapshotRevision>,
    ) -> Result<Vec<(SnapshotRevision, SourceSubmission)>, PersistenceError> {
        Err(PersistenceError::StorageUnavailable)
    }
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

/// Read-only resolution of an attempt with a known immutable submission and
/// candidate revision. It never reapplies staging or interprets CAD approval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitResolution {
    Published(CommitReceipt),
    NotPublished,
    ConflictingSubmission,
}

mod sqlite;
pub use sqlite::{SQLitePersistenceAdapter, SQLiteStorageSettings};
