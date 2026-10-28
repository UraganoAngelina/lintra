//! The canonical JSONL wire shape. This adapter decodes diagnostic data only;
//! every regulatory decision remains in the core's ValidationService.
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Diagnostic {
    pub code: String,
    pub flow: String,
    pub schema_version: String,
    pub trace: String,
    pub line: u64,
    pub record_identity: Identity,
    pub field: Option<Field>,
    pub actual: Option<Value>,
    pub expected: Option<String>,
    pub context: Vec<Context>,
    pub message: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Identity {
    pub sioc_record_id: Option<String>,
    pub fiscal_code: Option<String>,
    pub episode_id: Option<String>,
    pub technical_ids: BTreeMap<String, String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Field {
    pub id: String,
    pub name: String,
    pub start: usize,
    pub end: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Value {
    pub display: String,
    pub raw_hex: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Context {
    pub trace: String,
    pub field_id: String,
    pub value: Value,
}
impl Diagnostic {
    /// Reuse domain validation for wire invariants, never reproduce datatype or
    /// schema rules here. The returned error's Display is deliberately redacted.
    pub fn into_error(self) -> Result<lintra::ValidationError, super::ResultsError> {
        let value = lintra::ValidationError {
            code: self
                .code
                .parse()
                .map_err(|_| super::ResultsError::Malformed)?,
            flow_id: lintra::FlowId(self.flow),
            schema_version: lintra::SchemaVersion(self.schema_version),
            trace_id: lintra::TraceId(self.trace),
            line_number: self.line,
            record_identity: lintra::RecordIdentity {
                sioc_record_id: self.record_identity.sioc_record_id.map(Into::into),
                fiscal_code: self.record_identity.fiscal_code.map(Into::into),
                episode_id: self.record_identity.episode_id.map(Into::into),
                technical_ids: self
                    .record_identity
                    .technical_ids
                    .into_iter()
                    .map(|(key, value)| (key.into(), value.into()))
                    .collect(),
            },
            field: self.field.map(|v| lintra::FieldDiagnostic {
                id: lintra::FieldId(v.id),
                name: v.name.into(),
                start_one_based: v.start,
                end_one_based_inclusive: v.end,
            }),
            actual: self.actual.map(Into::into),
            expected: self.expected,
            context: self
                .context
                .into_iter()
                .map(|v| lintra::DiagnosticContextItem {
                    trace_id: lintra::TraceId(v.trace),
                    field_id: lintra::FieldId(v.field_id),
                    value: v.value.into(),
                })
                .collect(),
            message: self.message,
        };
        value
            .validate()
            .map_err(|_| super::ResultsError::Malformed)?;
        Ok(value)
    }
}
impl From<Value> for lintra::DiagnosticValue {
    fn from(v: Value) -> Self {
        Self {
            display: v.display,
            raw_hex: v.raw_hex,
        }
    }
}
