//! Stable protected identities. This module never chooses regulatory fields,
//! proves schema compatibility, writes storage, or discovers/generates a key.
use crate::{
    Fingerprint, FlowId, JoinKey, NamespaceValue, PersistenceNamespace, PersistenceSchemaScope,
    PersistenceTypeError, PersistentRecordIdentity, Sha256Digest, StoredIdentity, TraceId,
};
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{error::Error, fmt, sync::Arc};
use zeroize::Zeroizing;

type HmacSha256 = Hmac<Sha256>;
const POLICY: &str = "lintra.hmac-sha256.v1";
const DOMAIN: &[u8] = b"LINTRA-PERSISTENCE-PROTECTION-v1";
const KEY_SCHEME: &str = "lintra.hmac-sha256.key.v1";
const CONFIRM_SCHEME: &str = "lintra.hmac-sha256.confirm.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProtectionError {
    MissingKey,
    KeyUnavailable,
    ShortKey,
    InvalidContext,
    InvalidAudit,
    KeyMismatch,
    LengthOverflow,
    Metadata(PersistenceTypeError),
}
impl fmt::Display for ProtectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::MissingKey => "persistence protection key is not configured",
            Self::KeyUnavailable => "persistence protection key is unavailable",
            Self::ShortKey => "persistence protection key must contain at least 32 bytes",
            Self::InvalidContext => "invalid persistence protection metadata",
            Self::InvalidAudit => "invalid persistence protection audit",
            Self::KeyMismatch => "persistence protection key differs from historical context",
            Self::LengthOverflow => "persistence protection component is too large",
            Self::Metadata(_) => "invalid protected identity metadata",
        })
    }
}
impl Error for ProtectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::Metadata(error) = self {
            Some(error)
        } else {
            None
        }
    }
}
/// Owned key buffer; no default, serialization, raw accessor or implicit copy.
/// Caller provisioning must supply high-entropy material, not a password/CF.
/// ```compile_fail
/// fn leak(key: &lintra::PersistenceKey) { let _ = serde_json::to_string(key); }
/// ```
pub struct PersistenceKey(Zeroizing<Vec<u8>>);
impl PersistenceKey {
    pub fn new(bytes: Vec<u8>) -> Result<Self, ProtectionError> {
        let guarded = Zeroizing::new(bytes);
        if guarded.len() < 32 {
            return Err(ProtectionError::ShortKey);
        }
        Ok(Self(guarded))
    }
}
impl fmt::Debug for PersistenceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PersistenceKey([redacted])")
    }
}
/// Injection point for an explicitly configured secret manager. No filesystem,
/// environment fallback or production provider is selected by the core.
pub trait PersistenceKeyProvider: Send + Sync {
    fn resolve(&self, key_id: &str) -> Result<Arc<PersistenceKey>, ProtectionError>;
}

/// Exact technical scope metadata, not a compiled compatibility proof. The
/// future historical preflight must bind it to CompiledPersistenceProfile.
#[derive(Clone)]
pub struct IdentityProtectionScope {
    namespace: PersistenceNamespace,
    flow: FlowId,
    schema: PersistenceSchemaScope,
    trace: TraceId,
    key_kind: String,
}
fn label(value: &str) -> Result<(), ProtectionError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        Err(ProtectionError::InvalidContext)
    } else {
        Ok(())
    }
}
impl IdentityProtectionScope {
    pub fn new(
        namespace: PersistenceNamespace,
        flow: FlowId,
        schema: PersistenceSchemaScope,
        trace: TraceId,
        key_kind: String,
    ) -> Result<Self, ProtectionError> {
        label(&flow.0)?;
        label(&trace.0)?;
        label(&key_kind)?;
        match &schema {
            PersistenceSchemaScope::SchemaVersion { version } => label(&version.0)?,
            PersistenceSchemaScope::CompatibilityGroup { id } => label(id)?,
        }
        Ok(Self {
            namespace,
            flow,
            schema,
            trace,
            key_kind,
        })
    }
}
impl fmt::Debug for IdentityProtectionScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("IdentityProtectionScope { .. }")
    }
}
/// Public provenance, never key material. Deserialization is not attestation:
/// preflight must verify it with the configured key before comparing history.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtectionAudit {
    policy_id: String,
    key_confirmation: Fingerprint,
}
impl ProtectionAudit {
    pub fn policy_id(&self) -> &str {
        &self.policy_id
    }
    pub fn key_id(&self) -> &str {
        self.key_confirmation.key_id()
    }
}
impl fmt::Debug for ProtectionAudit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProtectionAudit { .. }")
    }
}

pub struct IdentityProtector {
    key_id: String,
    key: Arc<PersistenceKey>,
}
impl fmt::Debug for IdentityProtector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("IdentityProtector { .. }")
    }
}
fn component(mac: &mut HmacSha256, value: &[u8]) -> Result<(), ProtectionError> {
    let length = u64::try_from(value.len()).map_err(|_| ProtectionError::LengthOverflow)?;
    mac.update(&length.to_be_bytes());
    mac.update(value);
    Ok(())
}
impl IdentityProtector {
    /// expected=None is for a NEW context only. An existing store must supply
    /// its audit at application preflight; a missing audit is never a fallback.
    pub fn preflight(
        key_id: Option<&str>,
        provider: &dyn PersistenceKeyProvider,
        expected: Option<&ProtectionAudit>,
    ) -> Result<Self, ProtectionError> {
        let id = key_id
            .filter(|id| !id.trim().is_empty())
            .ok_or(ProtectionError::MissingKey)?;
        label(id)?;
        let key = provider.resolve(id)?;
        let value = Self {
            key_id: id.to_owned(),
            key,
        };
        if let Some(audit) = expected {
            value.verify_audit(audit)?;
        }
        Ok(value)
    }
    fn mac(&self, purpose: &[u8]) -> Result<HmacSha256, ProtectionError> {
        let mut mac =
            HmacSha256::new_from_slice(&self.key.0).map_err(|_| ProtectionError::ShortKey)?;
        component(&mut mac, DOMAIN)?;
        component(&mut mac, purpose)?;
        component(&mut mac, self.key_id.as_bytes())?;
        Ok(mac)
    }
    pub fn audit(&self) -> Result<ProtectionAudit, ProtectionError> {
        let mac = self.mac(b"key-confirmation")?;
        let tag = Fingerprint::new(
            CONFIRM_SCHEME.into(),
            self.key_id.clone(),
            mac.finalize().into_bytes().to_vec(),
        )
        .map_err(ProtectionError::Metadata)?;
        Ok(ProtectionAudit {
            policy_id: POLICY.into(),
            key_confirmation: tag,
        })
    }
    pub fn verify_audit(&self, audit: &ProtectionAudit) -> Result<(), ProtectionError> {
        if audit.policy_id != POLICY
            || audit.key_confirmation.protection_scheme() != CONFIRM_SCHEME
            || audit.key_confirmation.as_bytes().len() != 32
        {
            return Err(ProtectionError::InvalidAudit);
        }
        if audit.key_id() != self.key_id {
            return Err(ProtectionError::KeyMismatch);
        }
        // HMAC verification is constant-time; normal Fingerprint Eq is for
        // indexed identity metadata, never for authenticating this confirmation.
        self.mac(b"key-confirmation")?
            .verify_slice(audit.key_confirmation.as_bytes())
            .map_err(|_| ProtectionError::KeyMismatch)
    }
    fn scoped_mac(
        &self,
        scope: &IdentityProtectionScope,
        purpose: &[u8],
    ) -> Result<HmacSha256, ProtectionError> {
        let mut mac = self.mac(purpose)?;
        let parameters = scope.namespace.parameters();
        let count = u64::try_from(parameters.len()).map_err(|_| ProtectionError::LengthOverflow)?;
        mac.update(&count.to_be_bytes());
        for (name, value) in parameters {
            component(&mut mac, name.as_bytes())?;
            match value {
                NamespaceValue::String(text) => {
                    mac.update(b"S");
                    component(&mut mac, text.as_bytes())?;
                }
                NamespaceValue::Integer(number) => {
                    mac.update(b"I");
                    component(&mut mac, &number.to_be_bytes())?;
                }
            }
        }
        component(&mut mac, scope.flow.0.as_bytes())?;
        match &scope.schema {
            PersistenceSchemaScope::SchemaVersion { version } => {
                mac.update(b"V");
                component(&mut mac, version.0.as_bytes())?;
            }
            PersistenceSchemaScope::CompatibilityGroup { id } => {
                mac.update(b"G");
                component(&mut mac, id.as_bytes())?;
            }
        }
        component(&mut mac, scope.trace.0.as_bytes())?;
        component(&mut mac, scope.key_kind.as_bytes())?;
        Ok(mac)
    }
    pub fn protect_key(
        &self,
        scope: &IdentityProtectionScope,
        pk: &JoinKey,
    ) -> Result<PersistentRecordIdentity, ProtectionError> {
        let mut mac = self.scoped_mac(scope, b"key")?;
        component(&mut mac, pk.as_bytes())?;
        let fingerprint = Fingerprint::new(
            KEY_SCHEME.into(),
            self.key_id.clone(),
            mac.finalize().into_bytes().to_vec(),
        )
        .map_err(ProtectionError::Metadata)?;
        PersistentRecordIdentity::new(
            scope.namespace.clone(),
            scope.flow.clone(),
            scope.schema.clone(),
            scope.trace.clone(),
            scope.key_kind.clone(),
            fingerprint,
        )
        .map_err(ProtectionError::Metadata)
    }
    /// The caller supplies the canonical selected projection and an explicit
    /// definition digest. Neither field selection nor cross-version equivalence
    /// is inferred here. The record token binds BOTH PK and projection definition.
    pub fn protect_projection(
        &self,
        scope: &IdentityProtectionScope,
        pk: &JoinKey,
        definition: &Sha256Digest,
        projection: &JoinKey,
    ) -> Result<StoredIdentity, ProtectionError> {
        let identity = self.protect_key(scope, pk)?;
        let mut mac = self.scoped_mac(scope, b"record")?;
        component(&mut mac, pk.as_bytes())?;
        component(&mut mac, &definition.0)?;
        component(&mut mac, projection.as_bytes())?;
        let scheme = format!("lintra.hmac-sha256.record.v1:{definition}");
        let fingerprint = Fingerprint::new(
            scheme,
            self.key_id.clone(),
            mac.finalize().into_bytes().to_vec(),
        )
        .map_err(ProtectionError::Metadata)?;
        Ok(StoredIdentity::new(identity, fingerprint))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rfc4231_case6_sha256_matches_the_independent_standard_vector() {
        // Public RFC test material, never a configured application key.
        let mut mac = HmacSha256::new_from_slice(&[0xaa; 131]).unwrap();
        mac.update(b"Test Using Larger Than Block-Size Key - Hash Key First");
        let bytes = mac.finalize().into_bytes();
        let expected = "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54";
        assert_eq!(Sha256Digest(bytes.into()).to_string(), expected);
    }
}
