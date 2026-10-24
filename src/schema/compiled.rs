//! Immutable compiled schema models. Raw data cannot construct these types.
//!
//! Only the compiler child module (reexported by schema) constructs production
//! values. Engine and adapters cannot access fields or constructors.

#[path = "compiler.rs"]
pub(crate) mod compiler;

pub(crate) mod plans;
pub use plans::*;

use crate::domain::{ByteRange, FieldId, SchemaKey, TraceId};
use std::{collections::HashMap, sync::Arc};

macro_rules! borrowed_readers {
    ($($field:ident: $ty:ty),* $(,)?) => {$(
        pub fn $field(&self) -> &$ty { &self.$field }
    )*};
}

/// A field resolved and checked by the compiler. No setters or deserialization.
#[derive(Debug, Clone)]
pub struct CompiledFieldSpec {
    id: FieldId,
    name: Arc<str>,
    range: ByteRange,
    field_type: FieldType,
    missing_value_policy: MissingValuePolicy,
    presence_policy: CompiledPresencePolicy,
    alignment: AlignmentRule,
    lexical: Option<LexicalPlan>,
    constraints: Vec<CompiledConstraint>,
    regulatory_metadata: Option<RegulatoryMetadata>,
}

impl CompiledFieldSpec {
    borrowed_readers! {
        id: FieldId, name: str, missing_value_policy: MissingValuePolicy,
        presence_policy: CompiledPresencePolicy, lexical: Option<LexicalPlan>,
        constraints: [CompiledConstraint], regulatory_metadata: Option<RegulatoryMetadata>,
    }
    pub fn range(&self) -> ByteRange {
        self.range
    }
    pub fn field_type(&self) -> FieldType {
        self.field_type
    }
    pub fn alignment(&self) -> AlignmentRule {
        self.alignment
    }
}

/// Ordered physical fields with an index derived exclusively from that vector.
#[derive(Debug, Clone)]
pub struct CompiledRecordSchema {
    id: TraceId,
    name: Arc<str>,
    record_length: usize,
    fields: Vec<CompiledFieldSpec>,
    field_index: HashMap<FieldId, usize>,
    local_rules: Vec<CompiledRule>,
    derived_values: Vec<DerivedValuePlan>,
    identity_plan: IdentityExtractionPlan,
    projection_plan: ContextProjectionPlan,
    record_key: RecordKeyPlan,
    repeated_groups: Vec<RepeatedGroupPlan>,
    documented_non_executable_rules: Vec<DocumentedNonExecutableRule>,
}

// Private construction input; a sibling engine/adapter cannot instantiate it.
struct RecordParts {
    id: TraceId,
    name: Arc<str>,
    record_length: usize,
    fields: Vec<CompiledFieldSpec>,
    local_rules: Vec<CompiledRule>,
    derived_values: Vec<DerivedValuePlan>,
    identity_plan: IdentityExtractionPlan,
    projection_plan: ContextProjectionPlan,
    record_key: RecordKeyPlan,
    repeated_groups: Vec<RepeatedGroupPlan>,
    documented_non_executable_rules: Vec<DocumentedNonExecutableRule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ModelInvariantError {
    EmptyRecord,
    DuplicateFieldId(FieldId),
    FieldOutsideRecord(FieldId),
    PhysicalOrderMismatch(FieldId),
    RecordLengthMismatch,
    InvalidFieldIndex(usize),
    InvalidProjectionPlan,
    EmptyRecordKey,
    EmptyFlow,
    DuplicateTraceId(TraceId),
}

impl CompiledRecordSchema {
    borrowed_readers! {
        id: TraceId, name: str, fields: [CompiledFieldSpec],
        local_rules: [CompiledRule], derived_values: [DerivedValuePlan],
        identity_plan: IdentityExtractionPlan, projection_plan: ContextProjectionPlan,
        record_key: RecordKeyPlan, repeated_groups: [RepeatedGroupPlan],
        documented_non_executable_rules: [DocumentedNonExecutableRule],
    }
    pub fn record_length(&self) -> usize {
        self.record_length
    }
    pub fn field_index(&self, id: &FieldId) -> Option<usize> {
        self.field_index.get(id).copied()
    }
    pub fn field(&self, id: &FieldId) -> Option<&CompiledFieldSpec> {
        self.field_index(id)
            .and_then(|index| self.fields.get(index))
    }

    fn from_parts(parts: RecordParts) -> Result<Self, ModelInvariantError> {
        if parts.record_length == 0 || parts.fields.is_empty() {
            return Err(ModelInvariantError::EmptyRecord);
        }
        let mut field_index = HashMap::with_capacity(parts.fields.len());
        let mut expected_start = 0;
        for (index, field) in parts.fields.iter().enumerate() {
            if field.range.end_exclusive() > parts.record_length {
                return Err(ModelInvariantError::FieldOutsideRecord(field.id.clone()));
            }
            if field.range.start() != expected_start {
                return Err(ModelInvariantError::PhysicalOrderMismatch(field.id.clone()));
            }
            expected_start = field.range.end_exclusive();
            if field_index.insert(field.id.clone(), index).is_some() {
                return Err(ModelInvariantError::DuplicateFieldId(field.id.clone()));
            }
        }
        if expected_start != parts.record_length {
            return Err(ModelInvariantError::RecordLengthMismatch);
        }
        if parts.record_key.field_indexes.is_empty() {
            return Err(ModelInvariantError::EmptyRecordKey);
        }
        for index in parts
            .record_key
            .field_indexes
            .iter()
            .chain(parts.projection_plan.field_indexes.iter())
            .chain(parts.identity_plan.bindings.iter().map(|b| &b.field_index))
        {
            if *index >= parts.fields.len() {
                return Err(ModelInvariantError::InvalidFieldIndex(*index));
            }
        }
        if parts
            .projection_plan
            .field_indexes
            .windows(2)
            .any(|w| w[0] >= w[1])
            || parts
                .projection_plan
                .join_indexes
                .windows(2)
                .any(|w| w[0] >= w[1])
            || parts.projection_plan.join_indexes.iter().any(|index| {
                !parts
                    .projection_plan
                    .join_plans
                    .iter()
                    .any(|plan| plan.join_index == *index)
            })
            || parts.identity_plan.bindings.iter().any(|binding| {
                !parts
                    .projection_plan
                    .field_indexes
                    .contains(&binding.field_index)
            })
            || parts
                .record_key
                .field_indexes
                .iter()
                .any(|index| !parts.projection_plan.field_indexes.contains(index))
        {
            return Err(ModelInvariantError::InvalidProjectionPlan);
        }
        for (index, plan) in parts.projection_plan.join_plans.iter().enumerate() {
            if plan.field_indexes.is_empty()
                || plan
                    .field_indexes
                    .iter()
                    .any(|field| !parts.projection_plan.field_indexes.contains(field))
                || !parts
                    .projection_plan
                    .join_indexes
                    .contains(&plan.join_index)
                || parts.projection_plan.join_plans[..index]
                    .iter()
                    .any(|prior| prior.key == plan.key)
            {
                return Err(ModelInvariantError::InvalidProjectionPlan);
            }
        }
        Ok(Self {
            id: parts.id,
            name: parts.name,
            record_length: parts.record_length,
            fields: parts.fields,
            field_index,
            local_rules: parts.local_rules,
            derived_values: parts.derived_values,
            identity_plan: parts.identity_plan,
            projection_plan: parts.projection_plan,
            record_key: parts.record_key,
            repeated_groups: parts.repeated_groups,
            documented_non_executable_rules: parts.documented_non_executable_rules,
        })
    }
}

/// A whole, immutable flow. Declaration order controls iteration, never hash order.
#[derive(Debug, Clone)]
pub struct CompiledFlowSchema {
    key: SchemaKey,
    schema_format_version: u32,
    regulatory_version: Arc<str>,
    source: std::collections::BTreeMap<String, serde_json::Value>,
    encoding: EncodingSpec,
    traces: Vec<CompiledRecordSchema>,
    required_reference_data: Vec<ReferenceDependency>,
    cross_record_plan: CrossRecordValidationPlan,
    parameters: Vec<ParameterDefinition>,
    required_capabilities: Vec<Capability>,
    persistence_policy: Option<PersistencePolicy>,
}

struct FlowParts {
    key: SchemaKey,
    schema_format_version: u32,
    regulatory_version: Arc<str>,
    source: std::collections::BTreeMap<String, serde_json::Value>,
    encoding: EncodingSpec,
    traces: Vec<CompiledRecordSchema>,
    required_reference_data: Vec<ReferenceDependency>,
    cross_record_plan: CrossRecordValidationPlan,
    parameters: Vec<ParameterDefinition>,
    required_capabilities: Vec<Capability>,
    persistence_policy: Option<PersistencePolicy>,
}

impl CompiledFlowSchema {
    borrowed_readers! {
        key: SchemaKey, regulatory_version: str,
        source: std::collections::BTreeMap<String, serde_json::Value>, encoding: EncodingSpec,
        traces: [CompiledRecordSchema], required_reference_data: [ReferenceDependency],
        cross_record_plan: CrossRecordValidationPlan, parameters: [ParameterDefinition],
        required_capabilities: [Capability], persistence_policy: Option<PersistencePolicy>,
    }
    pub fn schema_format_version(&self) -> u32 {
        self.schema_format_version
    }
    pub fn trace(&self, id: &TraceId) -> Option<&CompiledRecordSchema> {
        self.traces.iter().find(|trace| trace.id == *id)
    }

    fn from_parts(parts: FlowParts) -> Result<Self, ModelInvariantError> {
        if parts.traces.is_empty() {
            return Err(ModelInvariantError::EmptyFlow);
        }
        for (index, trace) in parts.traces.iter().enumerate() {
            if parts.traces[..index]
                .iter()
                .any(|prior| prior.id == trace.id)
            {
                return Err(ModelInvariantError::DuplicateTraceId(trace.id.clone()));
            }
            for binding in &trace.projection_plan.join_plans {
                let join = parts
                    .cross_record_plan
                    .joins
                    .get(binding.join_index)
                    .ok_or(ModelInvariantError::InvalidProjectionPlan)?;
                let (expected_trace, expected_fields) = match binding.key.side {
                    JoinProjectionSide::Source => (join.source_trace, &join.source_fields),
                    JoinProjectionSide::Target => (join.target_trace, &join.target_fields),
                };
                if expected_trace != index
                    || binding.key.join_id != join.id
                    || binding.field_indexes != *expected_fields
                {
                    return Err(ModelInvariantError::InvalidProjectionPlan);
                }
            }
            for (join_index, join) in parts.cross_record_plan.joins.iter().enumerate() {
                for (expected_trace, side) in [
                    (join.source_trace, JoinProjectionSide::Source),
                    (join.target_trace, JoinProjectionSide::Target),
                ] {
                    if expected_trace == index
                        && !trace.projection_plan.join_plans.iter().any(|binding| {
                            binding.join_index == join_index && binding.key.side == side
                        })
                    {
                        return Err(ModelInvariantError::InvalidProjectionPlan);
                    }
                }
            }
        }
        Ok(Self {
            key: parts.key,
            schema_format_version: parts.schema_format_version,
            regulatory_version: parts.regulatory_version,
            source: parts.source,
            encoding: parts.encoding,
            traces: parts.traces,
            required_reference_data: parts.required_reference_data,
            cross_record_plan: parts.cross_record_plan,
            parameters: parts.parameters,
            required_capabilities: parts.required_capabilities,
            persistence_policy: parts.persistence_policy,
        })
    }
}

#[cfg(test)]
#[path = "compiled/tests.rs"]
mod tests;
