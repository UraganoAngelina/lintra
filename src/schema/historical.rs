//! Checked audit execution artifacts. The ordinary compiler remains unchanged;
//! the application must explicitly select the historical validation pipeline.
use crate::{
    CompiledFlowSchema, CompiledPersistenceProfile, FlowId, RawFlowSchema, SchemaCompilationError,
    SchemaCompiler, SchemaIssue, SchemaIssueCode, SchemaIssueSeverity, Sha256Digest, TraceId,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

/// Compiler-owned description of a physical-byte projection. It contains schema
/// metadata only; neither record values nor a cryptographic key are stored here.
#[derive(Debug, Clone, Serialize)]
pub struct RecordByteProjectionDefinition {
    trace_id: TraceId,
    definition_hash: Sha256Digest,
    descriptor: serde_json::Value,
}
impl RecordByteProjectionDefinition {
    pub fn trace_id(&self) -> &TraceId {
        &self.trace_id
    }
    pub fn definition_hash(&self) -> &Sha256Digest {
        &self.definition_hash
    }
    pub fn descriptor(&self) -> &serde_json::Value {
        &self.descriptor
    }
    pub fn canonical_descriptor_bytes(&self) -> Result<Vec<u8>, crate::CanonicalSchemaError> {
        super::canonical::canonical_value_bytes(self.descriptor.clone())
    }
}

/// A checked execution/profile pairing from ONE raw snapshot. No public
/// constructor or deserializer can forge the proof or replace one component.
/// The flow schema is consumed only by the historical application wiring.
pub struct CompiledHistoricalFlowSchema {
    schema: CompiledFlowSchema,
    profile: CompiledPersistenceProfile,
    definitions: Vec<RecordByteProjectionDefinition>,
}
impl CompiledHistoricalFlowSchema {
    pub fn profile(&self) -> &CompiledPersistenceProfile {
        &self.profile
    }
    pub fn record_projections(&self) -> &[RecordByteProjectionDefinition] {
        &self.definitions
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        CompiledFlowSchema,
        CompiledPersistenceProfile,
        Vec<RecordByteProjectionDefinition>,
    ) {
        (self.schema, self.profile, self.definitions)
    }
}
fn unsupported(raw: &RawFlowSchema, message: &str) -> SchemaCompilationError {
    SchemaCompilationError {
        issues: vec![SchemaIssue {
            severity: SchemaIssueSeverity::Error,
            code: SchemaIssueCode::UnsupportedConstruct,
            flow_id: Some(FlowId(raw.flow_id.clone())),
            trace_id: None,
            field_id: None,
            message: message.into(),
            source_note: None,
        }],
    }
}
impl SchemaCompiler {
    /// Enable only the implemented audit capability. Snapshot reconciliation,
    /// history invariants and temporal policies remain explicitly unsupported.
    /// No policy/capability is removed or rewritten to obtain an executable AST.
    pub fn compile_historical_audit(
        &self,
        raw: RawFlowSchema,
        other_versions: &[RawFlowSchema],
    ) -> Result<CompiledHistoricalFlowSchema, SchemaCompilationError> {
        let profile = self.compile_persistence_profile(&raw, other_versions)?;
        let policy = profile.policy().cloned().ok_or_else(|| {
            unsupported(
                &raw,
                "historical audit requires an explicit persistence policy",
            )
        })?;
        if policy.cross_run_audit.is_empty()
            || policy.snapshot.is_some()
            || !policy.history_identity_invariants.is_empty()
            || !policy.temporal_rules.is_empty()
        {
            return Err(unsupported(
                &raw,
                "this historical service supports audit-only profiles without snapshot or temporal policies",
            ));
        }
        let schema = self.compile_scoped(raw.clone(), Some(policy.clone()))?;
        let encoding = match schema.encoding() {
            crate::EncodingSpec::Utf8 => "utf-8",
            crate::EncodingSpec::Windows1252 => "windows-1252",
            crate::EncodingSpec::Iso8859_1 => "iso-8859-1",
            crate::EncodingSpec::Other(_) => {
                return Err(unsupported(
                    &raw,
                    "record-byte projection requires a supported explicit encoding",
                ));
            }
        };
        let mut definitions = Vec::with_capacity(policy.cross_run_audit.len());
        for key in &policy.cross_run_audit {
            let trace = raw
                .traces
                .get(key.trace_index)
                .ok_or_else(|| unsupported(&raw, "audit trace binding is inconsistent"))?;
            // Authoring vector order is not a record-byte meaning. Sort the
            // layout descriptors by field ID, retaining exact physical/missing
            // declarations. Offsets/encoding changes produce different contexts.
            let mut fields: Vec<_> = trace
                .fields
                .iter()
                .map(|f| (&f.id, &f.physical, &f.missing_value))
                .collect();
            fields.sort_by(|a, b| a.0.cmp(b.0));
            let descriptor = serde_json::json!({"kind":"lintra.record-bytes-projection.v1",
                "schema_format_version":raw.schema_format_version,"encoding":encoding,
                "trace_id":trace.id,"record_length":trace.record_length,"fields":fields});
            let bytes = super::canonical::canonical_value_bytes(descriptor.clone())
                .map_err(|_| unsupported(&raw, "cannot encode record projection definition"))?;
            definitions.push(RecordByteProjectionDefinition {
                trace_id: TraceId(trace.id.clone()),
                definition_hash: Sha256Digest(Sha256::digest(bytes).into()),
                descriptor,
            });
        }
        definitions.sort_by(|a, b| a.trace_id.0.cmp(&b.trace_id.0));
        Ok(CompiledHistoricalFlowSchema {
            schema,
            profile,
            definitions,
        })
    }
}
