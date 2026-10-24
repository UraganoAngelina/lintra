//! Pure projected/related field access. Construct once before expression
//! evaluation so storage errors cannot be converted to normative absence.
use crate::{
    CompiledFlowSchema, CompiledJoinSpec, CompiledRecordSchema, ContextError, ContextStore,
    DerivedValue, DerivedValueEvaluator, EvaluationContext, FieldSlice, FieldValueResolver,
    IntegerLiteral, JoinCardinality, JoinProjectionKey, JoinProjectionSide, ParameterDefinition,
    ProjectedJoinValue, ProjectedRecord, RelatedRecordSet, ResolvedFieldRef, ResolvedFieldValue,
    TraceId, UnknownReason, is_missing, logical_value,
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    error::Error,
    fmt,
};

#[derive(Clone, PartialEq, Eq)]
pub enum JoinParameterValue {
    Text(String),
    Integer(IntegerLiteral),
}
impl fmt::Debug for JoinParameterValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Text(_) => "JoinParameterValue::Text(..)",
            Self::Integer(_) => "JoinParameterValue::Integer(..)",
        })
    }
}

/// Explicit run inputs, never inferred from the records present in the store.
pub struct CrossRecordInputs<'a> {
    pub available_traces: &'a [TraceId],
    pub parameters: &'a BTreeMap<String, JoinParameterValue>,
}
pub struct CoveredCrossRecordInputs<'a> {
    pub inputs: CrossRecordInputs<'a>,
    pub coverage: &'a HashMap<TraceId, crate::TraceContextCoverage>,
}

#[derive(Debug)]
pub enum RelatedContextError {
    SourceTraceMissing,
    InvalidSourcePosition,
    InvalidTraceInput,
    DuplicateTraceInput,
    SourceInputUnavailable,
    InvalidParameter,
    MissingNamespaceParameter,
    InvalidJoinBinding,
    InconsistentCoverage,
    UnsupportedReduction,
    Store(ContextError),
}
impl fmt::Display for RelatedContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SourceTraceMissing => "projected source trace is not declared",
            Self::InvalidSourcePosition => "projected source position is invalid",
            Self::InvalidTraceInput => "available trace input is not declared",
            Self::DuplicateTraceInput => "available trace inputs must be unique",
            Self::SourceInputUnavailable => "projected source input must be available",
            Self::InvalidParameter => "runtime join parameter is unknown or invalid",
            Self::MissingNamespaceParameter => "required join namespace parameter is missing",
            Self::InvalidJoinBinding => "projected source join binding is inconsistent",
            Self::InconsistentCoverage => {
                "context coverage disagrees with explicit input availability"
            }
            Self::UnsupportedReduction => "reduction is unsupported by the selected context",
            Self::Store(_) => "related context lookup failed",
        })
    }
}
impl Error for RelatedContextError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Store(source) => Some(source),
            _ => None,
        }
    }
}

pub enum RelatedResolution<'a> {
    Records(RelatedRecordSet<'a>),
    Unavailable(UnknownReason),
}
/// The join reference retains both missing policies and diagnostics. Future
/// cross-rule outcomes can preserve NotApplicable without calling it True.
pub struct RelatedLookup<'a> {
    pub join: &'a CompiledJoinSpec,
    pub resolution: RelatedResolution<'a>,
}

pub struct ProjectedEvaluationContext<'a> {
    flow: &'a CompiledFlowSchema,
    projected: &'a ProjectedRecord,
    source_index: usize,
    related: BTreeMap<usize, RelatedLookup<'a>>,
    scalar: BTreeMap<usize, ScalarResolution<'a>>,
    coverage: BTreeMap<usize, crate::TraceContextCoverage>,
}
enum ScalarResolution<'a> {
    Record(&'a ProjectedRecord),
    Consensus(ConsensusProjection),
    Unavailable(UnknownReason),
}
struct ConsensusProjection {
    fields: Vec<usize>,
    // None = declared Missing; Some(empty) = present empty logical value.
    logical: Vec<Option<Vec<u8>>>,
    raw: Vec<Option<Vec<u8>>>,
    raw_consistent: bool,
}
impl<'a> ProjectedEvaluationContext<'a> {
    pub fn new(
        flow: &'a CompiledFlowSchema,
        projected: &'a ProjectedRecord,
        store: &'a dyn ContextStore,
        inputs: CrossRecordInputs<'_>,
    ) -> Result<Self, RelatedContextError> {
        Self::prepare(flow, projected, store, inputs, None)
    }
    pub fn with_coverage(
        flow: &'a CompiledFlowSchema,
        projected: &'a ProjectedRecord,
        store: &'a dyn ContextStore,
        inputs: CoveredCrossRecordInputs<'_>,
    ) -> Result<Self, RelatedContextError> {
        Self::prepare(flow, projected, store, inputs.inputs, Some(inputs.coverage))
    }
    fn prepare(
        flow: &'a CompiledFlowSchema,
        projected: &'a ProjectedRecord,
        store: &'a dyn ContextStore,
        inputs: CrossRecordInputs<'_>,
        coverage: Option<&HashMap<TraceId, crate::TraceContextCoverage>>,
    ) -> Result<Self, RelatedContextError> {
        let source_index = flow
            .traces()
            .iter()
            .position(|trace| trace.id() == &projected.trace_id)
            .ok_or(RelatedContextError::SourceTraceMissing)?;
        if projected.line_number == 0 {
            return Err(RelatedContextError::InvalidSourcePosition);
        }
        let available = validate_cross_inputs(flow, &inputs)?;
        if let Some(coverage) = coverage {
            validate_trace_coverage(flow, &inputs, coverage)?;
        }
        if !available.contains(&projected.trace_id) {
            return Err(RelatedContextError::SourceInputUnavailable);
        }
        let joins = &flow.cross_record_plan().joins;
        // Complete preflight before the first lookup, even if another join's
        // target is unavailable or its source key is missing.
        for join in joins
            .iter()
            .filter(|join| join.source_trace == source_index)
        {
            for &index in &join.namespace_parameters {
                let definition = flow
                    .parameters()
                    .get(index)
                    .ok_or(RelatedContextError::InvalidParameter)?;
                let id = match definition {
                    ParameterDefinition::String { id, .. }
                    | ParameterDefinition::Integer { id, .. } => id,
                };
                if !inputs.parameters.contains_key(id.as_ref()) {
                    return Err(RelatedContextError::MissingNamespaceParameter);
                }
            }
            let binding = JoinProjectionKey {
                join_id: join.id.clone(),
                side: JoinProjectionSide::Source,
            };
            if !projected.join_keys.contains_key(&binding) {
                return Err(RelatedContextError::InvalidJoinBinding);
            }
        }
        let mut related = BTreeMap::new();
        for (index, join) in joins
            .iter()
            .enumerate()
            .filter(|(_, join)| join.source_trace == source_index)
        {
            let target = flow
                .traces()
                .get(join.target_trace)
                .ok_or(RelatedContextError::InvalidJoinBinding)?;
            let binding = JoinProjectionKey {
                join_id: join.id.clone(),
                side: JoinProjectionSide::Source,
            };
            let resolution = if !available.contains(target.id()) {
                RelatedResolution::Unavailable(UnknownReason::TraceInputUnavailable)
            } else {
                match projected
                    .join_keys
                    .get(&binding)
                    .ok_or(RelatedContextError::InvalidJoinBinding)?
                {
                    ProjectedJoinValue::MissingComponent(_) => {
                        RelatedResolution::Unavailable(UnknownReason::FieldUnavailable)
                    }
                    ProjectedJoinValue::Available(key) => RelatedResolution::Records(
                        store
                            .find(target.id(), &join.id, key)
                            .map_err(RelatedContextError::Store)?,
                    ),
                }
            };
            related.insert(index, RelatedLookup { join, resolution });
        }
        let mut scalar = BTreeMap::new();
        let mut lookup_coverage = BTreeMap::new();
        for (&index, lookup) in &related {
            let target = flow
                .traces()
                .get(lookup.join.target_trace)
                .ok_or(RelatedContextError::InvalidJoinBinding)?;
            let status = if !available.contains(target.id()) {
                crate::TraceContextCoverage::Unavailable
            } else {
                coverage
                    .and_then(|coverage| coverage.get(target.id()))
                    .copied()
                    .unwrap_or(crate::TraceContextCoverage::Complete)
            };
            scalar.insert(index, scalar_resolution(lookup, target, status));
            lookup_coverage.insert(index, status);
        }
        Ok(Self {
            flow,
            projected,
            source_index,
            related,
            scalar,
            coverage: lookup_coverage,
        })
    }
    pub fn related(&self, join_index: usize) -> Option<&RelatedLookup<'a>> {
        self.related.get(&join_index)
    }
    pub fn lookup_coverage(&self, join_index: usize) -> Option<crate::TraceContextCoverage> {
        self.coverage.get(&join_index).copied()
    }

    fn resolve_value(
        &self,
        reference: &ResolvedFieldRef,
        raw_target: bool,
    ) -> ResolvedFieldValue<'_> {
        let (schema, record, field_index, field_id) = match reference {
            ResolvedFieldRef::Local {
                field_index,
                field_id,
            } => (
                &self.flow.traces()[self.source_index],
                self.projected,
                *field_index,
                field_id,
            ),
            ResolvedFieldRef::External {
                trace_index,
                field_index,
                field_id,
                join_index,
                join_id,
            } => {
                if !self.related.get(join_index).is_some_and(|lookup| {
                    lookup.join.id == *join_id && lookup.join.target_trace == *trace_index
                }) {
                    return ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan);
                };
                let Some(schema) = self.flow.traces().get(*trace_index) else {
                    return ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan);
                };
                if !schema
                    .fields()
                    .get(*field_index)
                    .is_some_and(|field| field.id() == field_id)
                {
                    return ResolvedFieldValue::Unavailable(UnknownReason::FieldUnavailable);
                }
                let record = match self.scalar.get(join_index) {
                    Some(ScalarResolution::Unavailable(reason)) => {
                        return ResolvedFieldValue::Unavailable(*reason);
                    }
                    Some(ScalarResolution::Record(record)) => *record,
                    Some(ScalarResolution::Consensus(tuple)) => {
                        let Some(position) =
                            tuple.fields.iter().position(|index| index == field_index)
                        else {
                            return ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan);
                        };
                        if raw_target && !tuple.raw_consistent {
                            return ResolvedFieldValue::Unavailable(UnknownReason::AmbiguousJoin);
                        }
                        let values = if raw_target {
                            &tuple.raw
                        } else {
                            &tuple.logical
                        };
                        return match values.get(position) {
                            Some(Some(value)) => ResolvedFieldValue::Present(value),
                            Some(None) => ResolvedFieldValue::Missing,
                            None => ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan),
                        };
                    }
                    None => return ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan),
                };
                if &record.trace_id != schema.id() {
                    return ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan);
                }
                (schema, record, *field_index, field_id)
            }
        };
        projected_value(schema, record, field_index, field_id, raw_target)
    }
}
pub(crate) fn projected_value<'a>(
    schema: &CompiledRecordSchema,
    record: &'a ProjectedRecord,
    index: usize,
    id: &crate::FieldId,
    raw_target: bool,
) -> ResolvedFieldValue<'a> {
    let Some(field) = schema.fields().get(index).filter(|field| field.id() == id) else {
        return ResolvedFieldValue::Unavailable(UnknownReason::FieldUnavailable);
    };
    let Some(raw) = record
        .values
        .get(id)
        .filter(|raw| raw.len() == field.range().len())
    else {
        return ResolvedFieldValue::Unavailable(UnknownReason::FieldUnavailable);
    };
    if is_missing(field.missing_value_policy(), raw) {
        return ResolvedFieldValue::Missing;
    }
    if raw_target {
        return ResolvedFieldValue::Present(raw);
    }
    match logical_value(&field.alignment(), FieldSlice { raw }) {
        Ok(value) => ResolvedFieldValue::Present(value.raw),
        Err(_) => ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan),
    }
}

/// Predicate/derived access to one target projection, without preparing or
/// accidentally following that target's unrelated outgoing joins.
pub(crate) struct ProjectedFieldContext<'a> {
    pub schema: &'a CompiledRecordSchema,
    pub record: &'a ProjectedRecord,
}
impl ProjectedFieldContext<'_> {
    fn value(&self, reference: &ResolvedFieldRef, raw: bool) -> ResolvedFieldValue<'_> {
        match reference {
            ResolvedFieldRef::Local {
                field_index,
                field_id,
            } => projected_value(self.schema, self.record, *field_index, field_id, raw),
            ResolvedFieldRef::External { .. } => {
                ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan)
            }
        }
    }
}
impl FieldValueResolver for ProjectedFieldContext<'_> {
    fn resolve_field(&self, reference: &ResolvedFieldRef) -> ResolvedFieldValue<'_> {
        self.value(reference, false)
    }
    fn resolve_raw_field(&self, reference: &ResolvedFieldRef) -> ResolvedFieldValue<'_> {
        self.value(reference, true)
    }
    fn resolve_derived(&self, index: usize) -> DerivedValue<'_> {
        match self.schema.derived_values().get(index) {
            Some(plan) => DerivedValueEvaluator.evaluate(plan, &EvaluationContext { fields: self }),
            None => DerivedValue::Unknown(UnknownReason::FieldUnavailable),
        }
    }
}
fn owned_value(value: ResolvedFieldValue<'_>) -> Result<Option<Vec<u8>>, UnknownReason> {
    match value {
        ResolvedFieldValue::Present(bytes) => Ok(Some(bytes.to_vec())),
        ResolvedFieldValue::Missing => Ok(None),
        ResolvedFieldValue::Unavailable(reason) => Err(reason),
    }
}
fn scalar_resolution<'a>(
    lookup: &RelatedLookup<'a>,
    schema: &CompiledRecordSchema,
    coverage: crate::TraceContextCoverage,
) -> ScalarResolution<'a> {
    let single = matches!(
        lookup.join.cardinality,
        JoinCardinality::ZeroOrOne | JoinCardinality::ExactlyOne
    );
    if !single && lookup.join.consensus.is_none() {
        return ScalarResolution::Unavailable(UnknownReason::UnsupportedPlan);
    }
    let records = match &lookup.resolution {
        RelatedResolution::Unavailable(reason) => return ScalarResolution::Unavailable(*reason),
        RelatedResolution::Records(records) => records,
    };
    if records.is_empty() {
        return ScalarResolution::Unavailable(
            if coverage == crate::TraceContextCoverage::Partial {
                UnknownReason::IncompleteRelatedContext
            } else {
                UnknownReason::RelatedRecordMissing
            },
        );
    }
    if single && records.len() > 1 {
        return ScalarResolution::Unavailable(UnknownReason::AmbiguousJoin);
    }
    if coverage == crate::TraceContextCoverage::Partial {
        return ScalarResolution::Unavailable(UnknownReason::IncompleteRelatedContext);
    }
    let Some(consensus) = &lookup.join.consensus else {
        if !single {
            return ScalarResolution::Unavailable(UnknownReason::UnsupportedPlan);
        }
        return match records.iter().next() {
            Some(record) => ScalarResolution::Record(record),
            None => ScalarResolution::Unavailable(UnknownReason::RelatedRecordMissing),
        };
    };
    let mut tuple: Option<ConsensusProjection> = None;
    let mut unknown = None;
    for record in records.iter() {
        if &record.trace_id != schema.id() {
            return ScalarResolution::Unavailable(UnknownReason::UnsupportedPlan);
        }
        let context = ProjectedFieldContext { schema, record };
        match crate::ExpressionEvaluator.evaluate(
            &consensus.include_if,
            &EvaluationContext { fields: &context },
        ) {
            crate::TruthValue::False => continue,
            crate::TruthValue::Unknown(reason) => {
                unknown.get_or_insert(reason);
                continue;
            }
            crate::TruthValue::True => {}
        }
        let mut logical = Vec::with_capacity(consensus.fields.len());
        let mut raw = Vec::with_capacity(consensus.fields.len());
        for &index in &consensus.fields {
            let Some(field) = schema.fields().get(index) else {
                return ScalarResolution::Unavailable(UnknownReason::UnsupportedPlan);
            };
            let logical_value =
                owned_value(projected_value(schema, record, index, field.id(), false));
            let raw_value = owned_value(projected_value(schema, record, index, field.id(), true));
            match (logical_value, raw_value) {
                (Ok(value), Ok(original)) => {
                    logical.push(value);
                    raw.push(original);
                }
                (Err(reason), _) | (_, Err(reason)) => {
                    return ScalarResolution::Unavailable(reason);
                }
            }
        }
        match &mut tuple {
            Some(previous) => {
                if previous.logical != logical {
                    return ScalarResolution::Unavailable(UnknownReason::AmbiguousJoin);
                }
                previous.raw_consistent &= previous.raw == raw;
            }
            None => {
                tuple = Some(ConsensusProjection {
                    fields: consensus.fields.clone(),
                    logical,
                    raw,
                    raw_consistent: true,
                })
            }
        }
    }
    if let Some(reason) = unknown {
        return ScalarResolution::Unavailable(reason);
    }
    match tuple {
        Some(tuple) => ScalarResolution::Consensus(tuple),
        None => ScalarResolution::Unavailable(UnknownReason::RelatedRecordMissing),
    }
}
impl FieldValueResolver for ProjectedEvaluationContext<'_> {
    fn resolve_field(&self, reference: &ResolvedFieldRef) -> ResolvedFieldValue<'_> {
        self.resolve_value(reference, false)
    }
    fn resolve_raw_field(&self, reference: &ResolvedFieldRef) -> ResolvedFieldValue<'_> {
        self.resolve_value(reference, true)
    }
    fn resolve_derived(&self, index: usize) -> DerivedValue<'_> {
        match self.flow.traces()[self.source_index]
            .derived_values()
            .get(index)
        {
            Some(plan) => DerivedValueEvaluator.evaluate(plan, &EvaluationContext { fields: self }),
            None => DerivedValue::Unknown(UnknownReason::FieldUnavailable),
        }
    }
}

pub(crate) fn validate_cross_inputs<'a>(
    flow: &CompiledFlowSchema,
    inputs: &CrossRecordInputs<'a>,
) -> Result<HashSet<&'a TraceId>, RelatedContextError> {
    let mut available = HashSet::new();
    for id in inputs.available_traces {
        if flow.trace(id).is_none() {
            return Err(RelatedContextError::InvalidTraceInput);
        }
        if !available.insert(id) {
            return Err(RelatedContextError::DuplicateTraceInput);
        }
    }
    // Validate all supplied bindings, including otherwise unused values:
    // no ignored unknown parameters, no default from an allowed domain.
    for (id, value) in inputs.parameters {
        let valid = flow
            .parameters()
            .iter()
            .any(|definition| match (definition, value) {
                (
                    ParameterDefinition::String {
                        id: declared,
                        allowed_values,
                    },
                    JoinParameterValue::Text(value),
                ) => {
                    declared.as_ref() == id
                        && allowed_values
                            .iter()
                            .any(|allowed| allowed.as_ref() == value)
                }
                (
                    ParameterDefinition::Integer {
                        id: declared,
                        min,
                        max,
                    },
                    JoinParameterValue::Integer(value),
                ) => declared.as_ref() == id && min <= value && value <= max,
                _ => false,
            });
        if !valid {
            return Err(RelatedContextError::InvalidParameter);
        }
    }
    Ok(available)
}

/// Explicit maps are exhaustive: omission cannot silently claim Complete.
pub(crate) fn validate_trace_coverage(
    flow: &CompiledFlowSchema,
    inputs: &CrossRecordInputs<'_>,
    coverage: &HashMap<TraceId, crate::TraceContextCoverage>,
) -> Result<(), RelatedContextError> {
    if coverage.len() != flow.traces().len()
        || flow
            .traces()
            .iter()
            .any(|trace| match coverage.get(trace.id()) {
                Some(
                    crate::TraceContextCoverage::Complete | crate::TraceContextCoverage::Partial,
                ) => !inputs.available_traces.contains(trace.id()),
                Some(crate::TraceContextCoverage::Unavailable) => {
                    inputs.available_traces.contains(trace.id())
                }
                None => true,
            })
    {
        return Err(RelatedContextError::InconsistentCoverage);
    }
    Ok(())
}
