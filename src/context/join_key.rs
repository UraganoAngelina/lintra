//! Collision-safe owned tuples of explicitly extracted logical byte values.
use crate::{
    CompiledRecordSchema, FieldId, LogicalValueError, RecordView, is_missing, logical_value,
};
use std::{collections::TryReserveError, error::Error, fmt};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct JoinKey(Vec<u8>);
impl JoinKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
impl fmt::Debug for JoinKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JoinKey")
            .field("encoded_length", &self.0.len())
            .finish_non_exhaustive()
    }
}
#[derive(Debug)]
pub enum JoinKeyError {
    EmptyFields,
    InvalidFieldIndex(usize),
    FieldUnavailable(FieldId),
    MissingComponent(FieldId),
    Logical(LogicalValueError),
    LengthOverflow,
    Allocation(TryReserveError),
}
impl fmt::Display for JoinKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyFields => "join key requires at least one field",
            Self::InvalidFieldIndex(_) => "join key field index is invalid",
            Self::FieldUnavailable(_) => "join key component is unavailable",
            Self::MissingComponent(_) => "join key component is missing under its declared policy",
            Self::Logical(_) => "join key logical extraction failed",
            Self::LengthOverflow => "join key encoded length overflows",
            Self::Allocation(_) => "join key allocation failed",
        })
    }
}
impl Error for JoinKeyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Logical(e) => Some(e),
            Self::Allocation(e) => Some(e),
            _ => None,
        }
    }
}
fn component<'record>(
    record: &RecordView<'record>,
    index: usize,
    schema: &CompiledRecordSchema,
) -> Result<&'record [u8], JoinKeyError> {
    let field = schema
        .fields()
        .get(index)
        .ok_or(JoinKeyError::InvalidFieldIndex(index))?;
    let value = record
        .field(field)
        .ok_or_else(|| JoinKeyError::FieldUnavailable(field.id().clone()))?;
    if is_missing(field.missing_value_policy(), value.as_bytes()) {
        return Err(JoinKeyError::MissingComponent(field.id().clone()));
    }
    logical_value(&field.alignment(), value)
        .map(|value| value.as_bytes())
        .map_err(JoinKeyError::Logical)
}

/// Eight-byte big-endian length then original bytes per component. Lengths are
/// byte counts, independent of host pointer width/endianness or text encoding.
/// Missing data remains a typed outcome, never an invented empty key component.
pub fn build_join_key(
    record: &RecordView<'_>,
    fields: &[usize],
    schema: &CompiledRecordSchema,
) -> Result<JoinKey, JoinKeyError> {
    if fields.is_empty() {
        return Err(JoinKeyError::EmptyFields);
    }
    let mut length = 0usize;
    for &index in fields {
        let value = component(record, index, schema)?;
        u64::try_from(value.len()).map_err(|_| JoinKeyError::LengthOverflow)?;
        length = length
            .checked_add(8)
            .and_then(|length| length.checked_add(value.len()))
            .ok_or(JoinKeyError::LengthOverflow)?;
    }
    let mut encoded = Vec::new();
    encoded
        .try_reserve_exact(length)
        .map_err(JoinKeyError::Allocation)?;
    for &index in fields {
        let value = component(record, index, schema)?;
        let length = u64::try_from(value.len()).map_err(|_| JoinKeyError::LengthOverflow)?;
        encoded.extend_from_slice(&length.to_be_bytes());
        encoded.extend_from_slice(value);
    }
    Ok(JoinKey(encoded))
}
