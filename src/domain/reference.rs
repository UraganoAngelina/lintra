//! Portable reference audit: no provider, filesystem, schema or engine ownership.
use super::{DatasetId, DatasetVersion, FieldId, Sha256Digest, TraceId};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetAvailability {
    Verified,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetPreparationPolicy {
    Strict,
    ContinueWithAvailableEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetIoOperation {
    ResolveRoot,
    InspectPath,
    ResolvePath,
    ReadFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetFileTypeIssue {
    NonDirectoryComponent,
    NonRegularFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipProperty {
    FormatVersion,
    DatasetId,
    Version,
    KeysHex,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipFormatIssue {
    Bom,
    InvalidJson,
    DuplicateProperty(String),
    UnknownProperty(String),
    MissingProperty(MembershipProperty),
    InvalidProperty(MembershipProperty),
    UnsupportedVersion(u32),
    InvalidHex { entry: usize },
    DuplicateEntry { entry: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct DatasetKey {
    pub id: DatasetId,
    pub version: DatasetVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationCompleteness {
    Complete,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceIoKind {
    NotFound,
    PermissionDenied,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum ReferenceDatasetDiagnostic {
    FileType(DatasetFileTypeIssue),
    Io {
        operation: DatasetIoOperation,
        error: ReferenceIoKind,
    },
    IntegrityMismatch,
    InvalidIdentity,
    IdentityMismatch,
    Format(MembershipFormatIssue),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceDatasetAudit {
    pub requested: DatasetKey,
    pub declared: Option<DatasetKey>,
    pub relative_path: String,
    pub expected_sha256: Sha256Digest,
    pub actual_sha256: Option<Sha256Digest>,
    pub availability: DatasetAvailability,
    pub diagnostics: Vec<ReferenceDatasetDiagnostic>,
}

/// Disjoint counts of attempted reference predicates, not records or rules.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ReferenceEvaluationCounts {
    pub definitive_satisfied: u64,
    pub definitive_violated: u64,
    pub provisional_present: u64,
    pub inconclusive: u64,
}
impl ReferenceEvaluationCounts {
    pub fn total(&self) -> Option<u64> {
        self.definitive_satisfied
            .checked_add(self.definitive_violated)?
            .checked_add(self.provisional_present)?
            .checked_add(self.inconclusive)
    }
    pub fn non_definitive(&self) -> bool {
        self.provisional_present > 0 || self.inconclusive > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceSourceUsage {
    pub dataset_index: usize,
    pub consulted: u64,
    pub selected_matches: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceWarningKind {
    ProvisionalEvidence,
    SearchNotExhaustive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceControlSummary {
    pub trace_id: TraceId,
    pub field_id: FieldId,
    pub rule_id: Option<String>,
    pub index: usize,
    /// Declaration-ordered indices into the dataset audit table.
    pub sources: Vec<ReferenceSourceUsage>,
    pub evaluations: ReferenceEvaluationCounts,
}
impl ReferenceControlSummary {
    /// At most two aggregated warning categories per compiled predicate.
    pub fn warnings(&self) -> impl Iterator<Item = (ReferenceWarningKind, u64)> {
        [
            (
                ReferenceWarningKind::ProvisionalEvidence,
                self.evaluations.provisional_present,
            ),
            (
                ReferenceWarningKind::SearchNotExhaustive,
                self.evaluations.inconclusive,
            ),
        ]
        .into_iter()
        .filter(|(_, count)| *count > 0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceValidationSummary {
    pub policy: DatasetPreparationPolicy,
    pub completeness: ValidationCompleteness,
    pub datasets: Vec<ReferenceDatasetAudit>,
    pub controls: Vec<ReferenceControlSummary>,
}
impl ReferenceValidationSummary {
    pub fn has_non_definitive_evaluations(&self) -> bool {
        self.controls
            .iter()
            .any(|control| control.evaluations.non_definitive())
    }
    /// Legacy map can represent only one verified version per dataset ID.
    pub fn verified_versions(&self) -> BTreeMap<DatasetId, DatasetVersion> {
        let mut versions: BTreeMap<DatasetId, Option<DatasetVersion>> = BTreeMap::new();
        for dataset in &self.datasets {
            if dataset.availability != DatasetAvailability::Verified {
                continue;
            }
            versions
                .entry(dataset.requested.id.clone())
                .and_modify(|version| {
                    if version.as_ref() != Some(&dataset.requested.version) {
                        *version = None;
                    }
                })
                .or_insert_with(|| Some(dataset.requested.version.clone()));
        }
        versions
            .into_iter()
            .filter_map(|(id, version)| version.map(|version| (id, version)))
            .collect()
    }
    pub(crate) fn valid(&self, partial: bool) -> bool {
        if self.completeness
            != if partial || self.has_non_definitive_evaluations() {
                ValidationCompleteness::Partial
            } else {
                ValidationCompleteness::Complete
            }
        {
            return false;
        }
        let mut keys = std::collections::HashSet::new();
        for dataset in &self.datasets {
            if !keys.insert(&dataset.requested)
                || dataset.requested.id.0.is_empty()
                || dataset.requested.version.0.is_empty()
                || dataset.relative_path.is_empty()
            {
                return false;
            }
            let trustworthy = dataset.actual_sha256.as_ref() == Some(&dataset.expected_sha256)
                && dataset.declared.as_ref() == Some(&dataset.requested)
                && dataset.diagnostics.is_empty();
            match dataset.availability {
                DatasetAvailability::Verified if !trustworthy => return false,
                DatasetAvailability::Degraded
                    if dataset.actual_sha256.is_none()
                        || dataset.declared.as_ref() != Some(&dataset.requested)
                        || dataset.actual_sha256.as_ref() == Some(&dataset.expected_sha256)
                        || dataset.diagnostics
                            != [ReferenceDatasetDiagnostic::IntegrityMismatch] =>
                {
                    return false;
                }
                DatasetAvailability::Unavailable if dataset.diagnostics.is_empty() => return false,
                _ => {}
            }
            if self.policy == DatasetPreparationPolicy::Strict
                && dataset.availability != DatasetAvailability::Verified
            {
                return false;
            }
        }
        let mut controls = std::collections::HashSet::new();
        for control in &self.controls {
            if control.field_id.0.is_empty()
                || control.trace_id.0.is_empty()
                || !controls.insert((
                    &control.trace_id,
                    &control.field_id,
                    &control.rule_id,
                    control.index,
                ))
                || control.evaluations.total().is_none()
                || control.sources.is_empty()
            {
                return false;
            }
            let total = control.evaluations.total().unwrap_or(0);
            let all_verified = control.sources.iter().all(|source| {
                self.datasets
                    .get(source.dataset_index)
                    .is_some_and(|dataset| dataset.availability == DatasetAvailability::Verified)
            });
            let has_degraded = control.sources.iter().any(|source| {
                self.datasets
                    .get(source.dataset_index)
                    .is_some_and(|dataset| dataset.availability == DatasetAvailability::Degraded)
            });
            if (!all_verified && control.evaluations.definitive_violated > 0)
                || (all_verified && control.evaluations.inconclusive > 0)
                || (!has_degraded && control.evaluations.provisional_present > 0)
            {
                return false;
            }

            let mut sources = std::collections::HashSet::new();
            for source in &control.sources {
                let Some(dataset) = self.datasets.get(source.dataset_index) else {
                    return false;
                };
                if !sources.insert(source.dataset_index)
                    || source.selected_matches > source.consulted
                    || source.consulted > total
                    || (dataset.availability == DatasetAvailability::Unavailable
                        && source.consulted > 0)
                {
                    return false;
                }
            }
        }
        true
    }
}
