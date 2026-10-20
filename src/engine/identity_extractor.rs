//! Schema-directed diagnostic identity, never a normative or persistent key.
use crate::{
    domain::RecordIdentity,
    input::{encoding::render_diagnostic_value, record::RecordView},
    schema::compiled::{CompiledRecordSchema, EncodingSpec, IdentityTarget},
    validators::presence::is_missing,
};
use std::{error::Error, fmt};

/// Violated extraction preconditions are technical, not field violations.
/// Counts/indexes provide context without copying source bytes or paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityExtractionError {
    RecordLength { expected: usize, actual: usize },
    InvalidFieldIndex { field_index: usize },
    FieldOutOfBounds { field_index: usize },
}
impl fmt::Display for IdentityExtractionError {
    /// Fixed text keeps identity contents outside technical messages.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::RecordLength { .. } => "identity extraction requires a full-width record",
            Self::InvalidFieldIndex { .. } => "identity field index is outside compiled fields",
            Self::FieldOutOfBounds { .. } => "identity field range is outside record bytes",
        })
    }
}
impl Error for IdentityExtractionError {}

/// Stateless extraction; all field locations come from the compiled plan.
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentityExtractor;
impl IdentityExtractor {
    /// Encoding is explicit (DET §147 refinement of §37). Checked access makes
    /// misuse of the full-length precondition fail without slicing/panicking.
    /// Missing values follow the declared raw policy; present values preserve
    /// their physical padding and use the shared diagnostic display renderer.
    /// No logical conversion, validation, case folding or key normalization.
    pub fn extract(
        &self,
        record: &RecordView<'_>,
        schema: &CompiledRecordSchema,
        encoding: &EncodingSpec,
    ) -> Result<RecordIdentity, IdentityExtractionError> {
        if record.raw().len() != schema.record_length() {
            return Err(IdentityExtractionError::RecordLength {
                expected: schema.record_length(),
                actual: record.raw().len(),
            });
        }
        let mut identity = RecordIdentity::default();
        for binding in &schema.identity_plan().bindings {
            let field = schema.fields().get(binding.field_index).ok_or(
                IdentityExtractionError::InvalidFieldIndex {
                    field_index: binding.field_index,
                },
            )?;
            let value = record
                .field(field)
                .ok_or(IdentityExtractionError::FieldOutOfBounds {
                    field_index: binding.field_index,
                })?;
            if is_missing(field.missing_value_policy(), value.as_bytes()) {
                continue;
            }
            // Display is deliberately diagnostic and may escape invalid bytes.
            // It must never serve as a PK, uniqueness key or persistence ID.
            let display = render_diagnostic_value(value.as_bytes(), encoding)
                .display
                .into();
            match &binding.target {
                IdentityTarget::SiocRecordId => identity.sioc_record_id = Some(display),
                IdentityTarget::FiscalCode => identity.fiscal_code = Some(display),
                IdentityTarget::EpisodeId => identity.episode_id = Some(display),
                IdentityTarget::Technical(name) => {
                    identity.technical_ids.insert(name.clone(), display);
                }
            }
        }
        Ok(identity)
    }
}
