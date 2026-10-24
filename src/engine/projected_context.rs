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
    collections::{BTreeMap, HashSet},
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
            Self::UnsupportedReduction => "consensus execution is not integrated in this context",
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
}
impl<'a> ProjectedEvaluationContext<'a> {
    pub fn new(
        flow: &'a CompiledFlowSchema,
        projected: &'a ProjectedRecord,
        store: &'a dyn ContextStore,
        inputs: CrossRecordInputs<'_>,
    ) -> Result<Self, RelatedContextError> {
        let source_index = flow
            .traces()
            .iter()
            .position(|trace| trace.id() == &projected.trace_id)
            .ok_or(RelatedContextError::SourceTraceMissing)?;
        if projected.line_number == 0 {
            return Err(RelatedContextError::InvalidSourcePosition);
        }
        let mut available = HashSet::new();
        for id in inputs.available_traces {
            if flow.trace(id).is_none() {
                return Err(RelatedContextError::InvalidTraceInput);
            }
            if !available.insert(id) {
                return Err(RelatedContextError::DuplicateTraceInput);
            }
        }
        if !available.contains(&projected.trace_id) {
            return Err(RelatedContextError::SourceInputUnavailable);
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
        let joins = &flow.cross_record_plan().joins;
        // Complete preflight before the first lookup, even if another join's
        // target is unavailable or its source key is missing.
        for join in joins
            .iter()
            .filter(|join| join.source_trace == source_index)
        {
            if join.consensus.is_some() {
                return Err(RelatedContextError::UnsupportedReduction);
            }
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
        Ok(Self {
            flow,
            projected,
            source_index,
            related,
        })
    }
    pub fn related(&self, join_index: usize) -> Option<&RelatedLookup<'a>> {
        self.related.get(&join_index)
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
                let Some(lookup) = self.related.get(join_index).filter(|lookup| {
                    lookup.join.id == *join_id && lookup.join.target_trace == *trace_index
                }) else {
                    return ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan);
                };
                if !matches!(
                    lookup.join.cardinality,
                    JoinCardinality::ZeroOrOne | JoinCardinality::ExactlyOne
                ) {
                    return ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan);
                }
                let records = match &lookup.resolution {
                    RelatedResolution::Unavailable(reason) => {
                        return ResolvedFieldValue::Unavailable(*reason);
                    }
                    RelatedResolution::Records(records) => records,
                };
                if records.is_empty() {
                    return ResolvedFieldValue::Unavailable(UnknownReason::RelatedRecordMissing);
                }
                if records.len() != 1 {
                    return ResolvedFieldValue::Unavailable(UnknownReason::AmbiguousJoin);
                }
                let Some(record) = records.iter().next() else {
                    return ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan);
                };
                let Some(schema) = self.flow.traces().get(*trace_index) else {
                    return ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan);
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
fn projected_value<'a>(
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
