//! Checked persistence declarations, separate from an executable flow schema.
//! No repository, historical query, mutation or new runtime capability is enabled.
use super::{SchemaCompilationError, SchemaCompiler, joins};
use crate::schema::{
    compiled::{
        ParameterDefinition, PersistencePolicy, PolicyKey, SnapshotPlan, VersionCompatibility,
    },
    linter::{SchemaIssue, SchemaIssueCode as Code, SchemaIssueSeverity, lint_raw_schema},
    raw::*,
};
use crate::{FlowId, SchemaKey, SchemaVersion, Sha256Digest, ValidationCode, schema_fingerprint};
use std::collections::{BTreeMap, HashSet};

/// Compiler-owned profile bindings. This artifact contains no executable flow
/// and cannot be passed to ValidationService as proof of historical validation.
/// All namespace indexes denote mandatory parameters; values are bound by the
/// future persistence preflight, never defaulted by this declaration compiler.
/// A profile cannot be deserialized from untrusted data as a compiled proof.
/// ```compile_fail
/// let _: lintra::CompiledPersistenceProfile = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct CompiledPersistenceProfile {
    key: SchemaKey,
    schema_hash: Sha256Digest,
    parameters: Vec<ParameterDefinition>,
    policy: Option<PersistencePolicy>,
    record_keys: BTreeMap<usize, Vec<usize>>,
    compatible_schemas: Vec<(SchemaVersion, Sha256Digest)>,
    trace_ids: Vec<crate::TraceId>,
}
impl CompiledPersistenceProfile {
    /// Complete trace inventory, including traces absent from a particular run.
    pub fn trace_ids(&self) -> &[crate::TraceId] {
        &self.trace_ids
    }
    pub fn schema_key(&self) -> &SchemaKey {
        &self.key
    }
    pub fn schema_hash(&self) -> &Sha256Digest {
        &self.schema_hash
    }
    pub fn parameters(&self) -> &[ParameterDefinition] {
        &self.parameters
    }
    pub fn policy(&self) -> Option<&PersistencePolicy> {
        self.policy.as_ref()
    }
    /// Checked current-schema PK field indexes, preserving author declaration order.
    pub fn record_key_fields(&self, trace_index: usize) -> Option<&[usize]> {
        self.record_keys.get(&trace_index).map(Vec::as_slice)
    }
    /// Actual source fingerprints used to establish declared-group compatibility.
    /// Labels alone never certify equivalence between two different PK layouts.
    pub fn compatible_schemas(&self) -> &[(SchemaVersion, Sha256Digest)] {
        &self.compatible_schemas
    }
}

type Failure = (Code, String);
type CompatibleSources = Vec<(SchemaVersion, Sha256Digest)>;
fn invalid(message: &str) -> Failure {
    (Code::InvalidSchemaValue, message.into())
}
fn issue(raw: &RawFlowSchema, (code, message): Failure) -> SchemaCompilationError {
    SchemaCompilationError {
        issues: vec![SchemaIssue {
            severity: SchemaIssueSeverity::Error,
            code,
            flow_id: Some(FlowId(raw.flow_id.clone())),
            trace_id: None,
            field_id: None,
            message,
            source_note: None,
        }],
    }
}
fn label(value: &str) -> bool {
    !value.is_empty() && value.trim() == value
}
fn lint(raw: &RawFlowSchema) -> Result<(), SchemaCompilationError> {
    let issues = lint_raw_schema(raw);
    if issues
        .iter()
        .any(|i| i.severity == SchemaIssueSeverity::Error)
    {
        return Err(SchemaCompilationError { issues });
    }
    Ok(())
}
fn hash(raw: &RawFlowSchema) -> Result<Sha256Digest, Failure> {
    schema_fingerprint(raw).map_err(|_| {
        (
            Code::InternalCompilerInvariant,
            "cannot fingerprint persistence profile source".into(),
        )
    })
}
fn parameter_id(parameter: &RawParameter) -> &str {
    match parameter {
        RawParameter::String { id, .. } | RawParameter::Integer { id, .. } => id,
    }
}
fn namespace(raw: &RawFlowSchema, ids: &[String]) -> Result<Vec<usize>, Failure> {
    if ids.is_empty() {
        return Err((
            Code::MissingParameterReference,
            "persistence namespace requires at least one declared parameter".into(),
        ));
    }
    let mut seen = HashSet::new();
    ids.iter()
        .map(|id| {
            if !seen.insert(id) {
                return Err(invalid("persistence namespace parameters must be unique"));
            }
            raw.parameters
                .iter()
                .position(|p| parameter_id(p) == id)
                .ok_or((
                    Code::MissingParameterReference,
                    "persistence namespace parameter is not declared".into(),
                ))
        })
        .collect()
}
fn key_fields(trace: &RawRecordSchema) -> Result<Vec<usize>, Failure> {
    if trace.record_key.fields.is_empty() {
        return Err((
            Code::InvalidRecordKey,
            "persistence key must contain at least one field".into(),
        ));
    }
    trace
        .record_key
        .error_code
        .parse::<ValidationCode>()
        .map_err(|_| {
            (
                Code::InvalidDiagnosticCode,
                "invalid record-key error code".into(),
            )
        })?;
    let mut seen = HashSet::new();
    trace
        .record_key
        .fields
        .iter()
        .map(|id| {
            if !seen.insert(id) {
                return Err((
                    Code::InvalidRecordKey,
                    "persistence record key contains a duplicate field".into(),
                ));
            }
            trace.fields.iter().position(|f| &f.id == id).ok_or((
                Code::MissingFieldReference,
                "persistence record key references an undeclared field".into(),
            ))
        })
        .collect()
}
fn keys(
    raw: &RawFlowSchema,
    entries: &[RawPolicyKey],
    bindings: &mut BTreeMap<usize, Vec<usize>>,
) -> Result<Vec<PolicyKey>, Failure> {
    let mut seen = HashSet::new();
    entries
        .iter()
        .map(|entry| {
            if !seen.insert(&entry.trace) {
                return Err(invalid(
                    "persistence policy repeats a trace within one control list",
                ));
            }
            let index = raw.traces.iter().position(|t| t.id == entry.trace).ok_or((
                Code::MissingTraceReference,
                "persistence policy references an undeclared trace".into(),
            ))?;
            bindings.insert(index, key_fields(&raw.traces[index])?);
            Ok(PolicyKey { trace_index: index })
        })
        .collect()
}

/// Key extraction semantics, excluding byte offset: a field can move between
/// versions while its declared ID, order, width, type, missing and logical-value
/// policies remain the same. No rename, width change or implicit conversion is
/// used to manufacture cross-version equivalence.
fn signature(trace: &RawRecordSchema) -> Result<Vec<(String, RawPhysical, RawMissing)>, Failure> {
    key_fields(trace)?
        .into_iter()
        .map(|index| {
            let field = &trace.fields[index];
            let mut physical = field.physical.clone();
            match &mut physical {
                RawPhysical::Numeric { start, .. }
                | RawPhysical::Alphanumeric { start, .. }
                | RawPhysical::Date { start, .. }
                | RawPhysical::Decimal { start, .. } => *start = 1,
            }
            Ok((field.id.clone(), physical, field.missing_value.clone()))
        })
        .collect()
}

fn compatibility(
    raw: &RawFlowSchema,
    policy: &RawPersistence,
    other_versions: &[RawFlowSchema],
    selected_keys: &BTreeMap<usize, Vec<usize>>,
) -> Result<(VersionCompatibility, CompatibleSources), Failure> {
    let mut fingerprints = vec![(SchemaVersion(raw.schema_version.clone()), hash(raw)?)];
    match &policy.version_compatibility {
        RawPersistenceVersionCompatibility::SameSchemaVersion {} => {
            if !other_versions.is_empty() {
                return Err(invalid(
                    "same-schema persistence does not accept additional schema versions",
                ));
            }
            Ok((VersionCompatibility::SameSchemaVersion, fingerprints))
        }
        RawPersistenceVersionCompatibility::DeclaredGroup {
            id,
            schema_versions,
        } => {
            let mut versions = HashSet::new();
            if !label(id)
                || schema_versions.is_empty()
                || schema_versions
                    .iter()
                    .any(|v| !label(v) || !versions.insert(v.as_str()))
                || !versions.contains(raw.schema_version.as_str())
            {
                return Err(invalid(
                    "compatibility group requires an explicit ID and unique versions including the current schema",
                ));
            }
            let mut members = BTreeMap::new();
            for member in other_versions {
                if member.flow_id != raw.flow_id
                    || member.schema_version == raw.schema_version
                    || !versions.contains(member.schema_version.as_str())
                    || members
                        .insert(member.schema_version.as_str(), member)
                        .is_some()
                {
                    return Err(invalid(
                        "compatibility group members must be distinct, declared versions of the same flow",
                    ));
                }
            }
            for version in schema_versions {
                if version == &raw.schema_version {
                    continue;
                }
                let member = members.get(version.as_str()).ok_or(invalid(
                    "every declared compatibility version requires an explicit source schema",
                ))?;
                let member_policy = member.persistence_policy.as_ref().ok_or(invalid(
                    "compatibility member has no declared persistence namespace",
                ))?;
                if member_policy.namespace_parameters != policy.namespace_parameters {
                    return Err(invalid(
                        "compatibility group changes the declared namespace projection",
                    ));
                }
                let foreign_namespace = namespace(member, &policy.namespace_parameters)?;
                for (current, foreign) in namespace(raw, &policy.namespace_parameters)?
                    .into_iter()
                    .zip(foreign_namespace)
                {
                    if std::mem::discriminant(&raw.parameters[current])
                        != std::mem::discriminant(&member.parameters[foreign])
                    {
                        return Err(invalid(
                            "compatibility group changes a namespace parameter type",
                        ));
                    }
                }
                for &index in selected_keys.keys() {
                    let current = &raw.traces[index];
                    let foreign = member.traces.iter().find(|t| t.id == current.id).ok_or((
                        Code::MissingTraceReference,
                        "compatibility member lacks a selected trace".into(),
                    ))?;
                    if signature(current)? != signature(foreign)? {
                        return Err(invalid(
                            "compatibility group changes a selected record-key projection",
                        ));
                    }
                }
                fingerprints.push((SchemaVersion(version.clone()), hash(member)?));
            }
            Ok((
                VersionCompatibility::DeclaredGroup {
                    id: id.as_str().into(),
                    schema_versions: schema_versions.iter().cloned().map(SchemaVersion).collect(),
                },
                fingerprints,
            ))
        }
    }
}

/// Lower only the already specified immutable/explicit/atomic contract. Bool
/// flags are checked even for callers constructing RawSnapshot directly.
fn compile_snapshot(
    raw: &RawFlowSchema,
    policy: &RawPersistence,
) -> Result<Option<SnapshotPlan>, Failure> {
    let Some(snapshot) = &policy.snapshot else {
        return Ok(None);
    };
    if !snapshot.approved_immutable
        || !snapshot.require_explicit_approval
        || !snapshot.missing_trace_policy.present_trace_is_complete
    {
        return Err(invalid(
            "snapshot requires immutable explicit approval and complete present traces",
        ));
    }
    if !raw
        .required_capabilities
        .contains(&RawFlowSchemaRequiredCapabilitiesItem::PersistenceSnapshotV1)
    {
        return Err((
            Code::MissingCapability,
            "snapshot declarations require persistence.snapshot.v1".into(),
        ));
    }
    let scope_parameters = namespace(raw, &snapshot.scope_parameters)?;
    let find_integer = |id: &str| -> Result<usize, Failure> {
        raw.parameters
            .iter()
            .position(|p| parameter_id(p) == id && matches!(p, RawParameter::Integer { .. }))
            .ok_or((
                Code::MissingParameterReference,
                "snapshot period requires declared integer parameters".into(),
            ))
    };
    let year_parameter = find_integer(&snapshot.period.year_parameter)?;
    let quarter_parameter = find_integer(&snapshot.period.quarter_parameter)?;
    if year_parameter == quarter_parameter
        || scope_parameters.contains(&year_parameter)
        || scope_parameters.contains(&quarter_parameter)
    {
        return Err(invalid(
            "snapshot scope and period parameters must be distinct",
        ));
    }
    let warning = &snapshot.missing_trace_policy.warning_code;
    if !warning.starts_with("W_")
        || !warning
            .bytes()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_')
        || warning.len() <= 2
    {
        return Err((
            Code::InvalidDiagnosticCode,
            "invalid snapshot warning code".into(),
        ));
    }
    Ok(Some(SnapshotPlan {
        scope_parameters,
        year_parameter,
        quarter_parameter,
        missing_trace_warning_code: warning.as_str().into(),
    }))
}

impl SchemaCompiler {
    /// Compile namespace, intra-run key and cross-run audit declarations using
    /// actual PKs and explicit sources for other group versions. The existing
    /// runtime `compile` gate continues to reject active persistence until its
    /// execution/storage prerequisites are implemented. No profile is runnable.
    /// Snapshot declarations support the approval coordinator. Historical identity
    /// and temporal execution remain explicitly rejected; no Full capability added.
    pub fn compile_persistence_profile(
        &self,
        raw: &RawFlowSchema,
        other_versions: &[RawFlowSchema],
    ) -> Result<CompiledPersistenceProfile, SchemaCompilationError> {
        lint(raw)?;
        for member in other_versions {
            lint(member)?;
        }
        let lowered = || -> Result<CompiledPersistenceProfile, Failure> {
            let parameters = joins::compile_parameters(&raw.parameters)?;
            for member in other_versions {
                joins::compile_parameters(&member.parameters)?;
            }
            let mut profile = CompiledPersistenceProfile {
                key: SchemaKey {
                    flow_id: FlowId(raw.flow_id.clone()),
                    version: SchemaVersion(raw.schema_version.clone()),
                },
                schema_hash: hash(raw)?,
                parameters,
                policy: None,
                record_keys: BTreeMap::new(),
                compatible_schemas: vec![],
                trace_ids: raw
                    .traces
                    .iter()
                    .map(|t| crate::TraceId(t.id.clone()))
                    .collect(),
            };
            let Some(policy) = &raw.persistence_policy else {
                if !other_versions.is_empty() {
                    return Err(invalid(
                        "disabled persistence has no compatibility group to resolve",
                    ));
                }
                return Ok(profile);
            };
            if !policy.history_identity_invariants.is_empty() || !policy.temporal_rules.is_empty() {
                return Err((Code::UnsupportedConstruct, "historical identity and temporal persistence plans require their later implementation tasks".into()));
            }
            if !policy.cross_run_audit.is_empty()
                && !raw
                    .required_capabilities
                    .contains(&RawFlowSchemaRequiredCapabilitiesItem::PersistenceAuditV1)
            {
                return Err((
                    Code::MissingCapability,
                    "cross-run audit declarations require persistence.audit.v1".into(),
                ));
            }
            let namespace_parameters = namespace(raw, &policy.namespace_parameters)?;
            let run_uniqueness = keys(raw, &policy.run_uniqueness, &mut profile.record_keys)?;
            let cross_run_audit = keys(raw, &policy.cross_run_audit, &mut profile.record_keys)?;
            let (version_compatibility, fingerprints) =
                compatibility(raw, policy, other_versions, &profile.record_keys)?;
            profile.compatible_schemas = fingerprints;
            profile.policy = Some(PersistencePolicy {
                namespace_parameters,
                version_compatibility,
                run_uniqueness,
                cross_run_audit,
                history_identity_invariants: vec![],
                snapshot: compile_snapshot(raw, policy)?,
                temporal_rules: vec![],
            });
            Ok(profile)
        };
        lowered().map_err(|error| issue(raw, error))
    }
}
