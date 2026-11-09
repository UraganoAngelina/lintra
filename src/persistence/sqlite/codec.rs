//! Physical format1 only. Domain wire stays unchanged; private keyed evidence is
//! stored in separate BLOBs and authenticated before any value leaves the adapter.
use super::*;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordWire {
    identity: PersistentRecordIdentity,
    fingerprint: Fingerprint,
}
pub(super) fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, PersistenceError> {
    serde_json::to_vec(value).map_err(|_| PersistenceError::CorruptStorage)
}
pub(super) fn decode<T: serde::de::DeserializeOwned>(value: &[u8]) -> Result<T, PersistenceError> {
    serde_json::from_slice(value).map_err(|_| PersistenceError::CorruptStorage)
}
pub(super) fn tag(value: &[u8]) -> Result<[u8; 32], PersistenceError> {
    value
        .try_into()
        .map_err(|_| PersistenceError::CorruptStorage)
}
pub(super) fn record(value: &StoredIdentity) -> Result<Vec<u8>, PersistenceError> {
    encode(&RecordWire {
        identity: value.identity().clone(),
        fingerprint: value.record_fingerprint().clone(),
    })
}
pub(super) fn read_record(
    bytes: &[u8],
    key_proof: &[u8],
    record_proof: &[u8],
    protector: &IdentityProtector,
) -> Result<StoredIdentity, PersistenceError> {
    let mut wire: RecordWire = decode(bytes)?;
    wire.identity.protection_evidence.0 = Some(tag(key_proof)?);
    let mut result = StoredIdentity::new(wire.identity, wire.fingerprint);
    result.protection_evidence.0 = Some(tag(record_proof)?);
    protector
        .verify_stored_identity(&result)
        .map_err(|_| PersistenceError::CorruptStorage)?;
    Ok(result)
}
pub(super) fn read_source(
    bytes: &[u8],
    proof: &[u8],
    protector: &IdentityProtector,
) -> Result<SourceSubmission, PersistenceError> {
    let mut value: SourceSubmission = decode(bytes)?;
    value.protection_evidence.0 = Some(tag(proof)?);
    protector
        .verify_submission(&value)
        .map_err(|_| PersistenceError::CorruptStorage)?;
    Ok(value)
}

/// The existing RecordWire/private proofs authenticate the technical record
/// seal; reconstructing the selected projection additionally binds the ENTIRE
/// fact payload. A valid identity token alone is deliberately insufficient.
pub(super) fn read_history(
    payload: &[u8],
    seal: &[u8],
    key_proof: &[u8],
    record_proof: &[u8],
    source: &SourceSubmission,
    protector: &IdentityProtector,
) -> Result<crate::HistoryFactBatch, PersistenceError> {
    let value = crate::HistoryFactBatch {
        payload: decode(payload)?,
        seal: read_record(seal, key_proof, record_proof, protector)?,
    };
    value
        .verify(source, protector)
        .map_err(|_| PersistenceError::CorruptStorage)?;
    Ok(value)
}
