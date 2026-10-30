//! Read-only, schema-scoped historical comparison. Publication, clinical state
//! transitions and normative cross-run errors are deliberately outside this API.
use crate::schema::compiled::{IntegerLiteral, ParameterDefinition};
use crate::{
    CancellationToken, CompiledPersistenceProfile, HistoricalChange, HistoricalDiff,
    HistoricalSnapshot, IdentityProtector, NamespaceValue, PersistenceError, PersistenceNamespace,
    PersistenceSchemaScope, PersistenceTypeError, PersistentContextRepository, ProtectionError,
    ReportError, SchemaKey, Sha256Digest, SnapshotRevision, SourceSubmission, StoredIdentity,
    TraceId, VersionCompatibility,
};
use serde::Serialize;
use std::{cmp::Ordering, collections::BTreeMap, error::Error, fmt};

/// Technical failures, never a clinical rejection or an inferred SIOC rule.
/// Display omits caller identifiers, protected tokens and record contents.
#[derive(Debug)]
#[non_exhaustive]
pub enum HistoricalAuditError {
    AuditNotDeclared,
    InvalidBinding,
    IncompatibleSchema,
    IncompatibleProjection,
    InconsistentHistory,
    NoComparableBaseline,
    Cancelled,
    CountOverflow,
    Persistence(PersistenceError),
    Protection(ProtectionError),
    Diff(PersistenceTypeError),
    Report(ReportError),
}
impl fmt::Display for HistoricalAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AuditNotDeclared => "the compiled profile declares no cross-run audit",
            Self::InvalidBinding => "historical audit namespace, trace or identity binding is invalid",
            Self::IncompatibleSchema => "historical schema version/hash is not qualified by the compiled profile",
            Self::IncompatibleProjection => "historical fingerprints use incompatible projection definitions or protection contexts",
            Self::InconsistentHistory => "historical provenance, metadata or snapshot contents disagree",
            Self::NoComparableBaseline => "a historical identity is stored, but no present-file baseline is available for comparison",
            Self::Cancelled => "historical audit was cancelled",
            Self::CountOverflow => "historical audit count overflow",
            Self::Persistence(_) => "historical audit storage read failed",
            Self::Protection(_) => "historical audit identity or source protection verification failed",
            Self::Diff(_) => "historical audit diff is structurally inconsistent",
            Self::Report(_) => "historical audit output failed; no complete comparison is available",
        })
    }
}
impl Error for HistoricalAuditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Persistence(e) => Some(e),
            Self::Protection(e) => Some(e),
            Self::Diff(e) => Some(e),
            Self::Report(e) => Some(e),
            _ => None,
        }
    }
}
impl From<PersistenceError> for HistoricalAuditError {
    fn from(e: PersistenceError) -> Self {
        Self::Persistence(e)
    }
}
impl From<ProtectionError> for HistoricalAuditError {
    fn from(e: ProtectionError) -> Self {
        Self::Protection(e)
    }
}

/// These observations concern only the explicitly supplied local repository.
/// An empty local history is never proof of absence across an organization.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoricalAuditCoverage {
    LocalRepositoryOnly,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceAuditBaseline {
    pub trace_id: TraceId,
    pub candidate_presence: crate::TracePresence,
    /// Latest PRESENT submission for this trace. Absent files do not replace it.
    pub revision: Option<SnapshotRevision>,
    pub source: Option<SourceSubmission>,
}
/// Version1 is an audit report contract, independent of database/schema versions.
/// Sources retain their hashes, reference versions and session provenance; no raw
/// identity or clinical projection is added to these metadata.
#[derive(Debug, Clone, Serialize)]
pub struct HistoricalAuditMetadata {
    pub format_version: u32,
    pub coverage: HistoricalAuditCoverage,
    pub namespace_generation: Option<SnapshotRevision>,
    pub schema_key: SchemaKey,
    pub schema_hash: Sha256Digest,
    pub schema_scope: PersistenceSchemaScope,
    pub candidate: SourceSubmission,
    pub traces: Vec<TraceAuditBaseline>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct HistoricalAuditCounts {
    pub inserted: u64,
    pub updated: u64,
    pub disappeared: u64,
    pub reactivated: u64,
    pub unchanged: u64,
}
impl HistoricalAuditCounts {
    fn add(&mut self, change: HistoricalChange) -> Result<(), HistoricalAuditError> {
        let count = match change {
            HistoricalChange::Inserted => &mut self.inserted,
            HistoricalChange::Updated => &mut self.updated,
            HistoricalChange::Removed => &mut self.disappeared,
            HistoricalChange::Reactivated => &mut self.reactivated,
            HistoricalChange::Unchanged => &mut self.unchanged,
        };
        *count = count
            .checked_add(1)
            .ok_or(HistoricalAuditError::CountOverflow)?;
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct HistoricalAuditSummary {
    pub metadata: HistoricalAuditMetadata,
    pub counts: HistoricalAuditCounts,
}

/// Explicit application dependencies; the validation engine never queries SQL.
pub struct HistoricalAuditService<'a> {
    profile: &'a CompiledPersistenceProfile,
    protector: &'a IdentityProtector,
    repository: &'a dyn PersistentContextRepository,
}
impl<'a> HistoricalAuditService<'a> {
    pub fn new(
        profile: &'a CompiledPersistenceProfile,
        protector: &'a IdentityProtector,
        repository: &'a dyn PersistentContextRepository,
    ) -> Self {
        Self {
            profile,
            protector,
            repository,
        }
    }

    /// Resolve bounded metadata pages first, then materialize only snapshots
    /// supplying a selected PRESENT baseline. Never load all historical keys.
    /// The owned load_snapshot port still has cost proportional to each selected
    /// snapshot; an eventual cursor port is needed to eliminate that cost.
    pub fn prepare<'b>(
        &'b self,
        candidate: &'b HistoricalSnapshot,
        cancellation: &CancellationToken,
    ) -> Result<PreparedHistoricalAudit<'b>, HistoricalAuditError> {
        checkpoint(cancellation)?;
        let policy = self
            .profile
            .policy()
            .filter(|p| !p.cross_run_audit.is_empty())
            .ok_or(HistoricalAuditError::AuditNotDeclared)?;
        check_namespace(self.profile, candidate.namespace())?;
        if candidate.source().schema_key() != self.profile.schema_key()
            || candidate.source().schema_hash() != self.profile.schema_hash()
        {
            return Err(HistoricalAuditError::IncompatibleSchema);
        }
        let mut traces = Vec::new();
        for key in &policy.cross_run_audit {
            let trace = self
                .profile
                .trace_ids()
                .get(key.trace_index)
                .ok_or(HistoricalAuditError::InvalidBinding)?;
            let presence = candidate
                .source()
                .traces()
                .iter()
                .find(|t| &t.trace_id == trace)
                .ok_or(HistoricalAuditError::InvalidBinding)?
                .presence;
            traces.push(TraceAuditBaseline {
                trace_id: trace.clone(),
                candidate_presence: presence,
                revision: None,
                source: None,
            });
        }
        traces.sort_by(|a, b| a.trace_id.0.cmp(&b.trace_id.0));
        let scope = match &policy.version_compatibility {
            VersionCompatibility::SameSchemaVersion => PersistenceSchemaScope::SchemaVersion {
                version: self.profile.schema_key().version.clone(),
            },
            VersionCompatibility::DeclaredGroup { id, .. } => {
                PersistenceSchemaScope::CompatibilityGroup { id: id.to_string() }
            }
        };
        let generation = self.repository.current_generation(candidate.namespace())?;
        let mut result = PreparedHistoricalAudit {
            profile: self.profile,
            protector: self.protector,
            repository: self.repository,
            candidate,
            baselines: BTreeMap::new(),
            metadata: HistoricalAuditMetadata {
                format_version: 1,
                coverage: HistoricalAuditCoverage::LocalRepositoryOnly,
                namespace_generation: generation,
                schema_key: self.profile.schema_key().clone(),
                schema_hash: self.profile.schema_hash().clone(),
                schema_scope: scope,
                candidate: candidate.source().clone(),
                traces,
            },
        };
        result.check_snapshot(candidate, cancellation)?;
        // Each cursor page is bounded by the port. Revision continuity and a
        // generation guard prevent an incomplete metadata view looking empty.
        let mut after: Option<SnapshotRevision> = None;
        loop {
            checkpoint(cancellation)?;
            result.check_generation()?;
            let page = result
                .repository
                .load_submission_page(candidate.namespace(), after)?;
            if page.len() > 128 {
                return Err(HistoricalAuditError::InconsistentHistory);
            }
            if page.is_empty() {
                break;
            }
            for (revision, source) in page {
                checkpoint(cancellation)?;
                let expected = after
                    .map(|r| r.next())
                    .transpose()
                    .map_err(HistoricalAuditError::Diff)?
                    .unwrap_or(SnapshotRevision::new(1));
                if revision != expected
                    || Some(revision) > generation
                    || source.namespace() != candidate.namespace()
                {
                    result.check_generation()?;
                    return Err(HistoricalAuditError::InconsistentHistory);
                }
                result.protector.verify_submission(&source)?;
                if source.schema_key().flow_id == self.profile.schema_key().flow_id {
                    for trace in &mut result.metadata.traces {
                        if source.traces().iter().any(|t| {
                            t.trace_id == trace.trace_id
                                && matches!(t.presence, crate::TracePresence::Present { .. })
                        }) {
                            trace.revision = Some(revision);
                            trace.source = Some(source.clone());
                        }
                    }
                }
                after = Some(revision);
            }
        }
        result.check_generation()?;
        if after != generation {
            return Err(HistoricalAuditError::InconsistentHistory);
        }
        for trace in &result.metadata.traces {
            // Missing current files are explicitly not compared. Their old
            // identities are not reported as disappeared or overwritten.
            if matches!(trace.candidate_presence, crate::TracePresence::Absent) {
                continue;
            }
            if let (Some(revision), Some(source)) = (trace.revision, trace.source.as_ref()) {
                result.check_source(source)?;
                if let std::collections::btree_map::Entry::Vacant(entry) =
                    result.baselines.entry(revision.value())
                {
                    let snapshot = result
                        .repository
                        .load_snapshot(candidate.namespace(), revision)?
                        .ok_or(HistoricalAuditError::InconsistentHistory)?;
                    if snapshot.revision() != revision || snapshot.source() != source {
                        return Err(HistoricalAuditError::InconsistentHistory);
                    }
                    entry.insert(snapshot);
                }
            }
        }
        for snapshot in result.baselines.values() {
            result.check_snapshot(snapshot, cancellation)?;
        }
        result.check_generation()?;
        Ok(result)
    }
}

/// Private immutable comparison state. Only selected baselines are owned; diff
/// items are emitted individually and never accumulated in a global vector.
pub struct PreparedHistoricalAudit<'a> {
    profile: &'a CompiledPersistenceProfile,
    protector: &'a IdentityProtector,
    repository: &'a dyn PersistentContextRepository,
    candidate: &'a HistoricalSnapshot,
    metadata: HistoricalAuditMetadata,
    baselines: BTreeMap<u64, HistoricalSnapshot>,
}
impl PreparedHistoricalAudit<'_> {
    pub fn metadata(&self) -> &HistoricalAuditMetadata {
        &self.metadata
    }
    fn check_generation(&self) -> Result<(), HistoricalAuditError> {
        let actual = self
            .repository
            .current_generation(self.candidate.namespace())?;
        if actual != self.metadata.namespace_generation {
            return Err(PersistenceError::GenerationConflict {
                expected: self.metadata.namespace_generation,
                actual,
            }
            .into());
        }
        Ok(())
    }
    fn check_source(&self, source: &SourceSubmission) -> Result<(), HistoricalAuditError> {
        self.protector.verify_submission(source)?;
        if source.namespace() != self.candidate.namespace() {
            return Err(HistoricalAuditError::InvalidBinding);
        }
        if source.schema_key().flow_id != self.profile.schema_key().flow_id
            || !self
                .profile
                .compatible_schemas()
                .iter()
                .any(|(version, hash)| {
                    version == &source.schema_key().version && hash == source.schema_hash()
                })
        {
            return Err(HistoricalAuditError::IncompatibleSchema);
        }
        Ok(())
    }
    fn check_snapshot(
        &self,
        snapshot: &HistoricalSnapshot,
        cancellation: &CancellationToken,
    ) -> Result<(), HistoricalAuditError> {
        self.check_source(snapshot.source())?;
        for row in snapshot.identities() {
            checkpoint(cancellation)?;
            self.protector.verify_stored_identity(row)?;
            let id = row.identity();
            let selected = self
                .metadata
                .traces
                .iter()
                .any(|t| &t.trace_id == id.trace_id());
            if id.namespace() != self.candidate.namespace()
                || id.flow_id() != &self.profile.schema_key().flow_id
                || id.schema_scope() != &self.metadata.schema_scope
                || !snapshot
                    .source()
                    .traces()
                    .iter()
                    .any(|t| &t.trace_id == id.trace_id())
                || (selected && id.key_kind() != "record_key")
            {
                return Err(HistoricalAuditError::InvalidBinding);
            }
        }
        Ok(())
    }

    /// Order is trace ID then protected-key bytes. Working memory consists of
    /// references to two selected trace slices, not diff rows or all history.
    /// A sink may have partial output on Err; only Ok returns a complete summary.
    /// The caller owns atomic publication/cleanup of any resulting report.
    pub fn write_diffs(
        &self,
        cancellation: &CancellationToken,
        mut emit: impl FnMut(&HistoricalDiff) -> Result<(), ReportError>,
    ) -> Result<HistoricalAuditSummary, HistoricalAuditError> {
        checkpoint(cancellation)?;
        self.check_generation()?;
        let mut counts = HistoricalAuditCounts::default();
        for trace in &self.metadata.traces {
            if matches!(trace.candidate_presence, crate::TracePresence::Absent) {
                continue;
            }
            let mut current: Vec<_> = self
                .candidate
                .identities()
                .iter()
                .filter(|r| r.identity().trace_id() == &trace.trace_id)
                .collect();
            let mut previous: Vec<_> = trace
                .revision
                .and_then(|r| self.baselines.get(&r.value()))
                .into_iter()
                .flat_map(|s| s.identities())
                .filter(|r| r.identity().trace_id() == &trace.trace_id)
                .collect();
            current.sort_unstable_by(|a, b| compare_identity(a, b));
            previous.sort_unstable_by(|a, b| compare_identity(a, b));
            let (mut i, mut j) = (0, 0);
            while i < previous.len() || j < current.len() {
                checkpoint(cancellation)?;
                self.check_generation()?;
                let ordering = match (previous.get(i), current.get(j)) {
                    (Some(a), Some(b)) => compare_identity(a, b),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => break,
                };
                let (identity, change, old, new) = match ordering {
                    Ordering::Less => {
                        let row = previous[i];
                        i += 1;
                        (
                            row.identity().clone(),
                            HistoricalChange::Removed,
                            Some(row.record_fingerprint().clone()),
                            None,
                        )
                    }
                    Ordering::Equal => {
                        let old = previous[i];
                        let new = current[j];
                        i += 1;
                        j += 1;
                        comparable(old, new)?;
                        let change = if old.record_fingerprint() == new.record_fingerprint() {
                            HistoricalChange::Unchanged
                        } else {
                            HistoricalChange::Updated
                        };
                        (
                            new.identity().clone(),
                            change,
                            Some(old.record_fingerprint().clone()),
                            Some(new.record_fingerprint().clone()),
                        )
                    }
                    Ordering::Greater => {
                        let row = current[j];
                        j += 1;
                        let occurrence = self.repository.lookup_identity_occurrence(
                            self.candidate.namespace(),
                            row.identity(),
                        )?;
                        self.check_generation()?;
                        if let Some(occurrence) = occurrence {
                            self.check_source(occurrence.source())?;
                            self.protector.verify_stored_identity(occurrence.record())?;
                            let source_presence = occurrence
                                .source()
                                .traces()
                                .iter()
                                .find(|t| t.trace_id == trace.trace_id)
                                .ok_or(HistoricalAuditError::InconsistentHistory)?
                                .presence;
                            if occurrence.record().identity() != row.identity()
                                || Some(occurrence.revision()) > self.metadata.namespace_generation
                            {
                                return Err(HistoricalAuditError::InconsistentHistory);
                            }
                            let Some(base) = trace.revision else {
                                // Known retained history is not a fresh insert.
                                // Without a present baseline, reactivation is
                                // unavailable rather than inferred from a gap.
                                return Err(
                                    if matches!(source_presence, crate::TracePresence::Absent) {
                                        HistoricalAuditError::NoComparableBaseline
                                    } else {
                                        HistoricalAuditError::InconsistentHistory
                                    },
                                );
                            };
                            if occurrence.revision() == base
                                || (occurrence.revision() > base
                                    && matches!(
                                        source_presence,
                                        crate::TracePresence::Present { .. }
                                    ))
                            {
                                return Err(HistoricalAuditError::InconsistentHistory);
                            }
                            // A later absent-file snapshot may carry a protected
                            // key; it does not replace the present-file baseline
                            // and membership alone declares no clinical state.
                            comparable(occurrence.record(), row)?;
                            (
                                row.identity().clone(),
                                HistoricalChange::Reactivated,
                                Some(occurrence.record().record_fingerprint().clone()),
                                Some(row.record_fingerprint().clone()),
                            )
                        } else {
                            (
                                row.identity().clone(),
                                HistoricalChange::Inserted,
                                None,
                                Some(row.record_fingerprint().clone()),
                            )
                        }
                    }
                };
                let diff = HistoricalDiff::new(identity, trace.revision, change, old, new)
                    .map_err(HistoricalAuditError::Diff)?;
                emit(&diff).map_err(HistoricalAuditError::Report)?;
                counts.add(change)?;
            }
        }
        checkpoint(cancellation)?;
        self.check_generation()?;
        Ok(HistoricalAuditSummary {
            metadata: self.metadata.clone(),
            counts,
        })
    }
}
fn checkpoint(token: &CancellationToken) -> Result<(), HistoricalAuditError> {
    if token.is_cancelled() {
        Err(HistoricalAuditError::Cancelled)
    } else {
        Ok(())
    }
}
fn comparable(a: &StoredIdentity, b: &StoredIdentity) -> Result<(), HistoricalAuditError> {
    if a.record_fingerprint()
        .same_protection_context(b.record_fingerprint())
    {
        Ok(())
    } else {
        Err(HistoricalAuditError::IncompatibleProjection)
    }
}
fn compare_identity(a: &StoredIdentity, b: &StoredIdentity) -> Ordering {
    let a = a.identity().key_fingerprint();
    let b = b.identity().key_fingerprint();
    (a.protection_scheme(), a.key_id(), a.as_bytes()).cmp(&(
        b.protection_scheme(),
        b.key_id(),
        b.as_bytes(),
    ))
}
fn check_namespace(
    profile: &CompiledPersistenceProfile,
    namespace: &PersistenceNamespace,
) -> Result<(), HistoricalAuditError> {
    let policy = profile
        .policy()
        .ok_or(HistoricalAuditError::AuditNotDeclared)?;
    if namespace.parameters().len() != policy.namespace_parameters.len() {
        return Err(HistoricalAuditError::InvalidBinding);
    }
    for (id, value) in namespace.parameters() {
        let valid = policy.namespace_parameters.iter().any(|&index| {
            match (profile.parameters().get(index), value) {
                (
                    Some(ParameterDefinition::String {
                        id: declared,
                        allowed_values,
                    }),
                    NamespaceValue::String(v),
                ) => declared.as_ref() == id && allowed_values.iter().any(|a| a.as_ref() == v),
                (
                    Some(ParameterDefinition::Integer {
                        id: declared,
                        min,
                        max,
                    }),
                    NamespaceValue::Integer(v),
                ) => {
                    declared.as_ref() == id
                        && v.to_string()
                            .parse::<IntegerLiteral>()
                            .ok()
                            .is_some_and(|v| min <= &v && &v <= max)
                }
                _ => false,
            }
        });
        if !valid {
            return Err(HistoricalAuditError::InvalidBinding);
        }
    }
    Ok(())
}
