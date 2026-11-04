//! Synthetic storage shapes, not regional schemas or executable CAD policies.
//! Shared by the benchmark and privacy qualification so the measured rows use
//! the same compiled extraction/protection path as the assertions.
use chrono::{TimeZone, Utc};
use lintra::*;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub const KEY_ID: &str = "synthetic-profile-key";
pub const RAW_MARKER: &[u8; 16] = b"SYNTHETIC-RAW-XX";
pub const CHANGED_MARKER: &[u8; 16] = b"SYNTHETIC-RAW-YY";
pub const SOURCE_MARKER: &[u8] = b"SYNTHETIC-RAW-SOURCE-FINGERPRINT";

pub struct Provider(pub u8);
impl PersistenceKeyProvider for Provider {
    fn resolve(&self, id: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        if id != KEY_ID {
            return Err(ProtectionError::KeyUnavailable);
        }
        // Deliberately public test material, never an operational key.
        Ok(Arc::new(PersistenceKey::new(vec![self.0; 32])?))
    }
}

#[derive(Clone, Copy)]
pub enum Shape {
    Sioc,
    Cad,
}
impl Shape {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sioc => "sioc-shaped-synthetic",
            Self::Cad => "cad-shaped-synthetic",
        }
    }
    pub fn arities(self) -> [usize; 5] {
        // ADR-PERSIST-001 §3: only component counts are borrowed. Field widths,
        // identifiers and payloads here are arbitrary; no regional rows loaded.
        match self {
            Self::Sioc => [3, 3, 4, 4, 2],
            Self::Cad => [2, 2, 2, 5, 5],
        }
    }
}

pub fn namespace(unit: &str) -> PersistenceNamespace {
    PersistenceNamespace::new(vec![("unit".into(), NamespaceValue::String(unit.into()))]).unwrap()
}

pub struct Fixture {
    pub schema: CompiledFlowSchema,
    pub schema_hash: Sha256Digest,
    pub count: usize,
}
impl Fixture {
    pub fn new(shape: Shape, count: usize) -> Self {
        assert!((5..=300_000).contains(&count));
        let field = |id: String, start: usize, length: usize| {
            json!({
                "id":id,"name":"Synthetic field",
                "physical":{"type":"alphanumeric","start":start,"length":length,
                    "alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}},
                "missing_value":{"kind":"never_missing"},
                "presence":{"base":"required","required_if":[],"forbidden_if":[]},
                "constraints":[]
            })
        };
        let traces: Vec<_> = shape
            .arities()
            .into_iter()
            .enumerate()
            .map(|(i, arity)| {
                let ids: Vec<_> = (0..arity).map(|k| format!("K{k}")).collect();
                let mut fields: Vec<_> = ids
                    .iter()
                    .enumerate()
                    .map(|(k, id)| field(id.clone(), k * 8 + 1, 8))
                    .collect();
                fields.push(field("PAYLOAD".into(), arity * 8 + 1, 16));
                json!({"id":format!("T{i}"),"name":"Synthetic trace",
                "record_length":arity*8+16,"fields":fields,
                "record_key":{"fields":ids,"uniqueness_scope":"trace",
                    "error_code":"E_DUPLICATE_RECORD_KEY"},
                "joins":[],"identity":[],"named_conditions":{},"derived_values":{},
                "repeated_groups":[],"rules":[],"documented_non_executable_rules":[]})
            })
            .collect();
        let value = json!({"schema_format_version":1,"flow_id":"SYNTHETIC_STORAGE",
            "schema_version":shape.label(),"regulatory_version":"synthetic-only",
            "source":{},"encoding":{"mode":"runtime_required","position_unit":"byte"},
            "required_capabilities":["local.v1"],"parameters":[],"reference_data":[],
            "persistence_policy":null,"traces":traces});
        let schema_hash = Sha256Digest(Sha256::digest(serde_json::to_vec(&value).unwrap()).into());
        let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
        raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
        Self {
            schema: SchemaCompiler.compile(raw).unwrap(),
            schema_hash,
            count,
        }
    }

    /// The physical row is transient; only attested HMAC identities/fingerprints
    /// escape. Two versions retain identical PK and change only the payload.
    pub fn row(
        &self,
        p: &IdentityProtector,
        unit: &str,
        index: usize,
        changed: bool,
    ) -> StoredIdentity {
        let trace = &self.schema.traces()[index % 5];
        let mut bytes = format!("{:08}", 77_000_000 + index)
            .repeat(trace.record_key().field_indexes.len())
            .into_bytes();
        bytes.extend_from_slice(if changed { CHANGED_MARKER } else { RAW_MARKER });
        let view = RecordView::new(&bytes);
        let key = build_join_key(&view, &trace.record_key().field_indexes, trace).unwrap();
        let projection = build_join_key(&view, &[trace.fields().len() - 1], trace).unwrap();
        let scope = IdentityProtectionScope::new(
            namespace(unit),
            self.schema.key().flow_id.clone(),
            PersistenceSchemaScope::SchemaVersion {
                version: self.schema.key().version.clone(),
            },
            trace.id().clone(),
            "record_key".into(),
        )
        .unwrap();
        p.protect_projection(&scope, &key, &self.schema_hash, &projection)
            .unwrap()
    }

    pub fn snapshot(
        &self,
        p: &IdentityProtector,
        unit: &str,
        revision: u64,
        changed: bool,
    ) -> HistoricalSnapshot {
        let source = SourceSubmission::new(SourceSubmissionParts {
            namespace: namespace(unit),
            submission_id: format!("synthetic-session-{revision}"),
            schema_key: self.schema.key().clone(),
            schema_hash: self.schema_hash.clone(),
            period: None,
            created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            validator_version: "synthetic-qualification".into(),
            validation_mode: ValidationMode::LocalOnly,
            validation_status: RunStatus::CompletedValid,
            reference_data_versions: vec![],
            traces: self
                .schema
                .traces()
                .iter()
                .enumerate()
                .map(|(i, t)| SubmissionTrace {
                    trace_id: t.id().clone(),
                    presence: TracePresence::Present {
                        record_count: ((self.count + 4 - i) / 5) as u64,
                    },
                    input_sha256: Some(Sha256Digest([17; 32])),
                })
                .collect(),
            fingerprint: Fingerprint::new(
                "synthetic-raw".into(),
                "fixture".into(),
                SOURCE_MARKER.to_vec(),
            )
            .unwrap(),
            state: SessionState::Validated,
        })
        .unwrap();
        let rows = (0..self.count)
            .map(|i| self.row(p, unit, i, changed))
            .collect();
        HistoricalSnapshot::new(
            SnapshotRevision::new(revision),
            p.protect_submission(&source).unwrap(),
            rows,
        )
        .unwrap()
    }
}

/// Exercise the generic transaction port; this fixture helper is not the UI
/// approval service and deliberately makes no claim of validating policy.
pub fn publish(
    repo: &dyn PersistentContextRepository,
    snapshot: &HistoricalSnapshot,
) -> Result<CommitReceipt, PersistenceError> {
    let expected = if snapshot.revision().value() == 1 {
        None
    } else {
        Some(SnapshotRevision::new(snapshot.revision().value() - 1))
    };
    let mut tx = repo.begin_transaction(snapshot.namespace(), expected)?;
    tx.stage_snapshot(snapshot)?;
    tx.commit()
}
