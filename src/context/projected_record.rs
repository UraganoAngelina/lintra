//! Owned, minimal byte projections. No validation rule or store is run here.
use crate::{
    CompiledRecordSchema, FieldId, JoinKey, JoinKeyError, JoinProjectionKey, RecordIdentity,
    RecordView, TraceId, build_join_key,
};
use std::{
    collections::{HashMap, TryReserveError},
    error::Error,
    fmt,
};

#[derive(Debug, Clone)]
pub enum ProjectedJoinValue {
    Available(JoinKey),
    /// Missing under the declared policy: do not index an invented empty key.
    MissingComponent(FieldId),
}

#[derive(Clone)]
pub struct ProjectedRecord {
    pub trace_id: TraceId,
    pub line_number: u64,
    pub identity: RecordIdentity,
    pub join_keys: HashMap<JoinProjectionKey, ProjectedJoinValue>,
    /// Exact physical bytes of selected fields; logical keys are separate.
    pub values: HashMap<FieldId, Vec<u8>>,
}
impl fmt::Debug for ProjectedRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProjectedRecord")
            .field("trace_id", &self.trace_id)
            .field("line_number", &self.line_number)
            .field("join_count", &self.join_keys.len())
            .field("field_count", &self.values.len())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub enum ProjectionError {
    InvalidLineNumber,
    RecordLengthMismatch {
        expected: usize,
        actual: usize,
    },
    InvalidFieldIndex(usize),
    FieldUnavailable(FieldId),
    JoinKey {
        binding: JoinProjectionKey,
        source: JoinKeyError,
    },
    Allocation(TryReserveError),
}
impl fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidLineNumber => "projection requires a one-based line number",
            Self::RecordLengthMismatch { .. } => "projection requires the declared record length",
            Self::InvalidFieldIndex(_) => "projection field index is invalid",
            Self::FieldUnavailable(_) => "projection field bytes are unavailable",
            Self::JoinKey { .. } => "projection join key construction failed",
            Self::Allocation(_) => "projection allocation failed",
        })
    }
}
impl Error for ProjectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::JoinKey { source, .. } => Some(source),
            Self::Allocation(source) => Some(source),
            _ => None,
        }
    }
}

/// Copy only compiled dependencies, with a real source line supplied by the
/// caller (RecordView intentionally contains no position metadata). This does
/// not certify local validity or decide missing-related-record policies.
pub fn project_record(
    record: &RecordView<'_>,
    schema: &CompiledRecordSchema,
    identity: RecordIdentity,
    line_number: u64,
) -> Result<ProjectedRecord, ProjectionError> {
    if line_number == 0 {
        return Err(ProjectionError::InvalidLineNumber);
    }
    if record.raw().len() != schema.record_length() {
        return Err(ProjectionError::RecordLengthMismatch {
            expected: schema.record_length(),
            actual: record.raw().len(),
        });
    }
    let plan = schema.projection_plan();
    let mut values = HashMap::new();
    values
        .try_reserve(plan.field_indexes.len())
        .map_err(ProjectionError::Allocation)?;
    for &index in &plan.field_indexes {
        let field = schema
            .fields()
            .get(index)
            .ok_or(ProjectionError::InvalidFieldIndex(index))?;
        let raw = record
            .field(field)
            .ok_or_else(|| ProjectionError::FieldUnavailable(field.id().clone()))?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(raw.len())
            .map_err(ProjectionError::Allocation)?;
        bytes.extend_from_slice(raw.as_bytes());
        values.insert(field.id().clone(), bytes);
    }
    let mut join_keys = HashMap::new();
    join_keys
        .try_reserve(plan.join_plans.len())
        .map_err(ProjectionError::Allocation)?;
    for binding in &plan.join_plans {
        let value = match build_join_key(record, &binding.field_indexes, schema) {
            Ok(key) => ProjectedJoinValue::Available(key),
            Err(JoinKeyError::MissingComponent(field)) => {
                ProjectedJoinValue::MissingComponent(field)
            }
            Err(source) => {
                return Err(ProjectionError::JoinKey {
                    binding: binding.key.clone(),
                    source,
                });
            }
        };
        join_keys.insert(binding.key.clone(), value);
    }
    Ok(ProjectedRecord {
        trace_id: schema.id().clone(),
        line_number,
        identity,
        join_keys,
        values,
    })
}
