//! Deterministic schema-model JSON for audit; not a schema validator or RFC 8785.
use crate::{RawFlowSchema, Sha256Digest};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, error::Error, fmt};

/// Original serialization cause is retained, without record/path diagnostics.
#[derive(Debug)]
pub struct CanonicalSchemaError(serde_json::Error);
impl fmt::Display for CanonicalSchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("canonical schema serialization failed")
    }
}
impl Error for CanonicalSchemaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

/// Sort object keys recursively, preserving arrays, exact numbers and strings.
/// Explicit BTreeMap ordering stays deterministic even if JSON map features
/// change. Array/field/rule order remains semantically significant.
fn ordered(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<_, _> = map
                .into_iter()
                .map(|(key, value)| (key, ordered(value)))
                .collect();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(values) => Value::Array(values.into_iter().map(ordered).collect()),
        scalar => scalar,
    }
}
/// Serialize the typed normative model as compact UTF-8, no BOM/final newline.
/// This normalizes object order/whitespace, not numeric spelling, Unicode or
/// logical field values. Runtime encoding binding is excluded by Raw's existing
/// serialization contract, like other run options; no source JSON is modified.
pub fn canonical_schema_bytes(raw: &RawFlowSchema) -> Result<Vec<u8>, CanonicalSchemaError> {
    let value = serde_json::to_value(raw).map_err(CanonicalSchemaError)?;
    canonical_value_bytes(value)
}
/// Reuse the same deterministic ordering for compiler-owned metadata such as
/// fingerprint definitions. This does not change the normative schema codec.
pub(crate) fn canonical_value_bytes(value: Value) -> Result<Vec<u8>, CanonicalSchemaError> {
    serde_json::to_vec(&ordered(value)).map_err(CanonicalSchemaError)
}
/// Hash exactly the canonical model bytes using the already approved SHA-256
/// dependency. Independent of file formatting; distinct from input-file hashes.
pub fn schema_fingerprint(raw: &RawFlowSchema) -> Result<Sha256Digest, CanonicalSchemaError> {
    Ok(Sha256Digest(
        Sha256::digest(canonical_schema_bytes(raw)?).into(),
    ))
}
