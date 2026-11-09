//! Checked persistence declarations, separate from an executable flow schema.
//! No repository, historical query, mutation or new runtime capability is enabled.
use super::{SchemaCompilationError, SchemaCompiler, joins};
use crate::schema::{
    compiled::{
        AlignmentRule, HistoricalIdentityInvariant, MissingValuePolicy, ParameterDefinition,
        PersistencePolicy, PolicyKey, SnapshotPlan, VersionCompatibility,
    },
    linter::{SchemaIssue, SchemaIssueCode as Code, SchemaIssueSeverity, lint_raw_schema},
    raw::*,
};
use crate::{
    ByteRange, FieldId, FlowId, SchemaKey, SchemaVersion, Sha256Digest, ValidationCode,
    schema_fingerprint,
};
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
    namespace_bindings: Vec<crate::NamespaceBindingPlan>,
    identity_projections: BTreeMap<usize, IdentityProjection>,
}

/// Physical key extraction metadata only, never a partially validated field.
/// All offsets, policies and identifiers are bound once by the profile compiler.
#[derive(Debug, Clone)]
pub(crate) struct CanonicalKeyField {
    pub(crate) id: FieldId,
    pub(crate) range: ByteRange,
    pub(crate) alignment: AlignmentRule,
    pub(crate) missing: MissingValuePolicy,
}

#[derive(Debug, Clone)]
pub(crate) struct InvariantProjection {
    pub(crate) progressive: Vec<CanonicalKeyField>,
    pub(crate) event: Vec<CanonicalKeyField>,
    pub(crate) on_reassignment_error: ValidationCode,
}

/// PK and secondary projections refer to the same physical record definition.
/// This cannot be deserialized or supplied as proof of clinical validation.
#[derive(Debug, Clone)]
pub(crate) struct IdentityProjection {
    pub(crate) record_length: usize,
    pub(crate) record_key: Vec<CanonicalKeyField>,
    pub(crate) invariant: Option<InvariantProjection>,
}
impl CompiledPersistenceProfile {
    pub fn namespace_bindings(&self) -> &[crate::NamespaceBindingPlan] {
        &self.namespace_bindings
    }
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
    pub(crate) fn identity_projections(&self) -> &BTreeMap<usize, IdentityProjection> {
        &self.identity_projections
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

/// An invariant selects two nonempty exact tuples, independently of the PK.
/// The first implementation supports the approved immutable binding contract:
/// one invariant per trace, same-version scope, same-event reactivation allowed.
/// Other declarations fail before input instead of gaining invented semantics.
fn invariants(
    raw: &RawFlowSchema,
    policy: &RawPersistence,
    record_keys: &mut BTreeMap<usize, Vec<usize>>,
) -> Result<Vec<HistoricalIdentityInvariant>, Failure> {
    if policy.history_identity_invariants.is_empty() {
        return Ok(vec![]);
    }
    if !matches!(
        policy.version_compatibility,
        RawPersistenceVersionCompatibility::SameSchemaVersion {}
    ) {
        return Err((Code::UnsupportedConstruct, "historical identity invariants require same_schema_version until their projection compatibility is qualified".into()));
    }
    let mut traces = HashSet::new();
    policy
        .history_identity_invariants
        .iter()
        .map(|entry| {
            if !entry.reactivate_same_event {
                return Err((
                    Code::UnsupportedConstruct,
                    "this invariant implementation requires explicit same-event reactivation"
                        .into(),
                ));
            }
            let trace_index = raw
                .traces
                .iter()
                .position(|trace| trace.id == entry.trace)
                .ok_or((
                    Code::MissingTraceReference,
                    "identity invariant references an undeclared trace".into(),
                ))?;
            if !traces.insert(trace_index) {
                return Err((
                    Code::UnsupportedConstruct,
                    "multiple identity invariants on one trace are not supported".into(),
                ));
            }
            let trace = &raw.traces[trace_index];
            let resolve = |ids: &[String]| -> Result<Vec<usize>, Failure> {
                if ids.is_empty() {
                    return Err(invalid("identity invariant tuples must not be empty"));
                }
                let mut seen = HashSet::new();
                ids.iter()
                    .map(|id| {
                        if !label(id) || !seen.insert(id) {
                            return Err(invalid(
                                "identity invariant fields must be unique and unpadded",
                            ));
                        }
                        trace
                            .fields
                            .iter()
                            .position(|field| &field.id == id)
                            .ok_or((
                                Code::MissingFieldReference,
                                "identity invariant references an undeclared field".into(),
                            ))
                    })
                    .collect()
            };
            let progressive_fields = resolve(&entry.progressive_fields)?;
            let event_fields = resolve(&entry.event_fields)?;
            let on_reassignment_error = entry
                .on_reassignment_error
                .parse::<ValidationCode>()
                .map_err(|_| {
                    (
                        Code::InvalidDiagnosticCode,
                        "identity invariant requires a valid declared error code".into(),
                    )
                })?;
            record_keys.insert(trace_index, key_fields(trace)?);
            Ok(HistoricalIdentityInvariant {
                trace_index,
                progressive_fields,
                event_fields,
                on_reassignment_error,
                reactivate_same_event: true,
            })
        })
        .collect()
}

fn canonical_fields(
    trace: &RawRecordSchema,
    indexes: &[usize],
) -> Result<Vec<CanonicalKeyField>, Failure> {
    indexes
        .iter()
        .map(|&index| {
            let field = trace.fields.get(index).ok_or((
                Code::InternalCompilerInvariant,
                "identity projection index is inconsistent".into(),
            ))?;
            let (start, length, alignment) = match &field.physical {
                RawPhysical::Numeric {
                    start,
                    length,
                    alignment,
                    ..
                }
                | RawPhysical::Alphanumeric {
                    start,
                    length,
                    alignment,
                    ..
                } => (*start, *length, super::alignment(alignment)),
                RawPhysical::Date { start, length, .. }
                | RawPhysical::Decimal { start, length, .. } => {
                    (*start, *length, AlignmentRule::Exact)
                }
            };
            let missing = match &field.missing_value {
                RawMissing::Spaces {} => MissingValuePolicy::Spaces,
                RawMissing::NeverMissing {} => MissingValuePolicy::NeverMissing,
                RawMissing::Exact { raw } => MissingValuePolicy::Exact(raw.as_bytes().to_vec()),
                RawMissing::AnyOf { raw_values } => MissingValuePolicy::AnyOf(
                    raw_values
                        .iter()
                        .map(|value| value.as_bytes().to_vec())
                        .collect(),
                ),
            };
            Ok(CanonicalKeyField {
                id: FieldId(field.id.clone()),
                range: super::source_range(start, length)
                    .map_err(|code| (code, "identity projection range is invalid".into()))?,
                alignment,
                missing,
            })
        })
        .collect()
}

fn identity_projections(
    raw: &RawFlowSchema,
    keys: &BTreeMap<usize, Vec<usize>>,
    invariants: &[HistoricalIdentityInvariant],
) -> Result<BTreeMap<usize, IdentityProjection>, Failure> {
    if invariants.is_empty()
        && raw
            .persistence_policy
            .as_ref()
            .is_none_or(|policy| policy.snapshot.is_none())
    {
        return Ok(BTreeMap::new());
    }
    keys.iter()
        .map(|(&index, fields)| {
            let trace = &raw.traces[index];
            let invariant = invariants
                .iter()
                .find(|plan| plan.trace_index == index)
                .map(|plan| {
                    Ok::<_, Failure>(InvariantProjection {
                        progressive: canonical_fields(trace, &plan.progressive_fields)?,
                        event: canonical_fields(trace, &plan.event_fields)?,
                        on_reassignment_error: plan.on_reassignment_error.clone(),
                    })
                })
                .transpose()?;
            Ok((
                index,
                IdentityProjection {
                    record_length: usize::try_from(trace.record_length).map_err(|_| {
                        (
                            Code::RangeOverflow,
                            "identity record length overflows".into(),
                        )
                    })?,
                    record_key: canonical_fields(trace, fields)?,
                    invariant,
                },
            ))
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

fn profile_bindings(raw: &RawFlowSchema) -> Result<Vec<crate::NamespaceBindingPlan>, Failure> {
    crate::schema::namespace_binding::compile(raw)
}
fn binding_value_signature(field: &RawFieldSpec) -> (RawPhysical, RawMissing) {
    let mut physical = field.physical.clone();
    match &mut physical {
        RawPhysical::Numeric { start, .. }
        | RawPhysical::Alphanumeric { start, .. }
        | RawPhysical::Date { start, .. }
        | RawPhysical::Decimal { start, .. } => *start = 1,
    }
    (physical, field.missing_value.clone())
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
                if member_policy.namespace_bindings != policy.namespace_bindings
                    || member_policy.namespace_parameters != policy.namespace_parameters
                {
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
                // Binding compatibility includes the exact logical extraction
                // semantics even if the bound field is not a selected PK.
                for binding in &profile_bindings(raw)? {
                    let current = &raw.traces[binding.trace_index];
                    let foreign = member
                        .traces
                        .iter()
                        .find(|t| t.id == current.id)
                        .ok_or(invalid("compatibility member lacks a binding trace"))?;
                    let field = &current.fields[binding.field_index];
                    let other = foreign
                        .fields
                        .iter()
                        .find(|f| f.id == field.id)
                        .ok_or(invalid("compatibility member lacks a binding field"))?;
                    if binding_value_signature(field) != binding_value_signature(other) {
                        return Err(invalid(
                            "compatibility group changes namespace binding value semantics",
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
    /// Snapshot and identity declarations support the approval coordinator.
    /// Temporal metadata may include the snapshot period needed by inactivity.
    /// Executable public entry points independently reject combined historical
    /// obligations until their producer is integrated; this profile is no proof
    /// of a completed validation or an approved snapshot.
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
            let namespace_bindings = crate::schema::namespace_binding::compile(raw)?;
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
                namespace_bindings,
                record_keys: BTreeMap::new(),
                identity_projections: BTreeMap::new(),
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
            // This entry point returns metadata, never an executable flow or an
            // approval. Inactivity needs the existing snapshot.period binding.
            // Ordinary/historical-audit executable entry points retain their
            // independent fail-closed checks on unintegrated obligations.
            let temporal_rules = super::temporal::compile(raw)?;
            if (!policy.cross_run_audit.is_empty()
                || !policy.history_identity_invariants.is_empty())
                && !raw
                    .required_capabilities
                    .contains(&RawFlowSchemaRequiredCapabilitiesItem::PersistenceAuditV1)
            {
                return Err((
                    Code::MissingCapability,
                    "historical comparison declarations require persistence.audit.v1".into(),
                ));
            }
            let namespace_parameters = namespace(raw, &policy.namespace_parameters)?;
            let run_uniqueness = keys(raw, &policy.run_uniqueness, &mut profile.record_keys)?;
            let cross_run_audit = keys(raw, &policy.cross_run_audit, &mut profile.record_keys)?;
            let history_identity_invariants = invariants(raw, policy, &mut profile.record_keys)?;
            profile.identity_projections =
                identity_projections(raw, &profile.record_keys, &history_identity_invariants)?;
            let (version_compatibility, fingerprints) =
                compatibility(raw, policy, other_versions, &profile.record_keys)?;
            profile.compatible_schemas = fingerprints;
            profile.policy = Some(PersistencePolicy {
                namespace_parameters,
                version_compatibility,
                run_uniqueness,
                cross_run_audit,
                history_identity_invariants,
                snapshot: compile_snapshot(raw, policy)?,
                temporal_rules,
            });
            Ok(profile)
        };
        lowered().map_err(|error| issue(raw, error))
    }
}
