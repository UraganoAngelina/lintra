//! Provider evidence, not normative rule outcomes or a validation summary.
use super::dataset::{DatasetKey, LookupSet};
use super::normalization::LookupNormalization;
use crate::Sha256Digest;
use std::{collections::HashMap, error::Error, io, sync::Arc};

pub use crate::domain::{
    DatasetAvailability, DatasetFileTypeIssue, DatasetIoOperation, DatasetPreparationPolicy,
    MembershipFormatIssue, MembershipProperty,
};

#[derive(Debug, Clone)]
pub enum DatasetAnomaly {
    FileType(DatasetFileTypeIssue),
    Io {
        operation: DatasetIoOperation,
        source: Arc<io::Error>,
    },
    IntegrityMismatch,
    InvalidIdentity,
    IdentityMismatch,
    Format {
        issue: MembershipFormatIssue,
        source: Option<Arc<serde_json::Error>>,
    },
}
impl DatasetAnomaly {
    pub fn source(&self) -> Option<&(dyn Error + Send + Sync + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source.as_ref()),
            Self::Format {
                source: Some(source),
                ..
            } => Some(source.as_ref()),
            _ => None,
        }
    }
}

/// Only the provider constructs evidence. Expected and actual hashes are never
/// interchanged; None means no complete original byte stream was obtained.
#[derive(Debug, Clone)]
pub struct DatasetEvidence {
    pub(super) key: DatasetKey,
    pub(super) expected_sha256: Sha256Digest,
    pub(super) actual_sha256: Option<Sha256Digest>,
    pub(super) declared_key: Option<DatasetKey>,
    pub(super) availability: DatasetAvailability,
    pub(super) policy: DatasetPreparationPolicy,
    pub(super) anomalies: Vec<DatasetAnomaly>,
}
impl DatasetEvidence {
    pub fn key(&self) -> &DatasetKey {
        &self.key
    }
    pub fn expected_sha256(&self) -> &Sha256Digest {
        &self.expected_sha256
    }
    pub fn actual_sha256(&self) -> Option<&Sha256Digest> {
        self.actual_sha256.as_ref()
    }
    pub fn declared_key(&self) -> Option<&DatasetKey> {
        self.declared_key.as_ref()
    }
    pub fn availability(&self) -> DatasetAvailability {
        self.availability
    }
    pub fn policy(&self) -> DatasetPreparationPolicy {
        self.policy
    }
    pub fn anomalies(&self) -> &[DatasetAnomaly] {
        &self.anomalies
    }
}

/// A physical observation with its trust evidence, NOT a definitive normative
/// conclusion. In particular None is not false, and Degraded+Some(true) is not
/// verified conformance. Global membership/union decisions belong to T-P11-05.
#[derive(Debug)]
pub struct DatasetObservation<'a> {
    evidence: &'a DatasetEvidence,
    observed_present: Option<bool>,
}
impl DatasetObservation<'_> {
    pub fn evidence(&self) -> &DatasetEvidence {
        self.evidence
    }
    pub fn availability(&self) -> DatasetAvailability {
        self.evidence.availability
    }
    pub fn observed_present(&self) -> Option<bool> {
        self.observed_present
    }
}

/// Immutable snapshot. No provider/root/file handle survives into the lookup
/// loop; source diagnostics are retained once per dependency, not per record.
#[derive(Clone)]
pub struct PreparedReferenceEvidence {
    pub(super) sets: HashMap<DatasetKey, Arc<dyn LookupSet>>,
    pub(super) evidence: Vec<DatasetEvidence>,
    pub(super) indices: HashMap<DatasetKey, usize>,
    pub(super) policy: DatasetPreparationPolicy,
}
impl PreparedReferenceEvidence {
    /// Enumerate only evidence verified during the run. Missing, degraded or
    /// non-enumerable datasets are never displayed as an empty valid domain.
    pub fn domain_page(
        &self,
        key: &DatasetKey,
        after: Option<&[u8]>,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Option<super::dataset::LookupPage> {
        if self.evidence(key)?.availability() != DatasetAvailability::Verified {
            return None;
        }
        self.sets.get(key)?.page_after(after, cancelled)
    }
    pub fn policy(&self) -> DatasetPreparationPolicy {
        self.policy
    }
    /// Declaration order, including unavailable dependencies.
    pub fn datasets(&self) -> &[DatasetEvidence] {
        &self.evidence
    }
    pub fn evidence(&self, key: &DatasetKey) -> Option<&DatasetEvidence> {
        self.evidence.get(*self.indices.get(key)?)
    }
    /// Borrowed HashMap/HashSet queries only. No allocation or filesystem access.
    /// An unregistered dependency is None; registered Unavailable is Some with
    /// observed_present=None, preserving the distinction from an empty set.
    pub fn observe(&self, key: &DatasetKey, value: &[u8]) -> Option<DatasetObservation<'_>> {
        Some(DatasetObservation {
            evidence: self.evidence(key)?,
            observed_present: self.sets.get(key).map(|set| set.contains(value)),
        })
    }

    /// Explicit query-only normalization, retaining exactly the same evidence
    /// as observe. Unknown/unavailable dependencies never become empty sets.
    pub fn observe_with_normalization(
        &self,
        key: &DatasetKey,
        value: &[u8],
        normalization: LookupNormalization,
    ) -> Option<DatasetObservation<'_>> {
        let evidence = self.evidence(key)?;
        let observed_present = self.sets.get(key).map(|set| {
            let normalized = normalization.apply(value);
            set.contains(normalized.as_ref())
        });
        Some(DatasetObservation {
            evidence,
            observed_present,
        })
    }
}
