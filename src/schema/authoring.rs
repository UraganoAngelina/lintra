//! Pure offline authoring helpers; no rule inference, I/O or runtime normalization.
use super::importer::XlsxImportProfile;
use crate::{RawFlowSchema, Sha256Digest};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportedPhysicalType {
    Numeric,
    Alphanumeric,
    Date,
    Decimal,
}
impl ImportedPhysicalType {
    fn tag(self) -> &'static str {
        match self {
            Self::Numeric => "numeric",
            Self::Alphanumeric => "alphanumeric",
            Self::Date => "date",
            Self::Decimal => "decimal",
        }
    }
    fn canonical(tag: &str) -> Option<Self> {
        match tag {
            "numeric" => Some(Self::Numeric),
            "alphanumeric" => Some(Self::Alphanumeric),
            "date" => Some(Self::Date),
            "decimal" => Some(Self::Decimal),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeAlias {
    pub alias: String,
    pub canonical: ImportedPhysicalType,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TypeNormalization {
    pub trace_id: String,
    pub field_id: String,
    pub before: String,
    pub after: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedImportProfile {
    pub profile: XlsxImportProfile,
    pub changes: Vec<TypeNormalization>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaAuthoringError {
    InvalidAlias(String),
    UnmappedAlias {
        trace_id: String,
        field_id: String,
        alias: String,
    },
    InvalidNoteReview(&'static str),
}
impl fmt::Display for SchemaAuthoringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAlias(_) => {
                f.write_str("type alias is empty, duplicated or redefines a canonical type")
            }
            Self::UnmappedAlias { .. } => {
                f.write_str("type alias requires an explicit reviewed mapping")
            }
            Self::InvalidNoteReview(reason) => write!(f, "imported note review: {reason}"),
        }
    }
}
impl Error for SchemaAuthoringError {}
/// Canonical tags are identity mappings. All other aliases are caller-supplied;
/// never trim/case-fold a token or infer a type from a field ID/name.
pub fn normalize_import_profile(
    profile: &XlsxImportProfile,
    aliases: &[TypeAlias],
) -> Result<NormalizedImportProfile, SchemaAuthoringError> {
    let mut table = BTreeMap::new();
    for entry in aliases {
        if entry.alias.is_empty()
            || table
                .insert(entry.alias.as_str(), entry.canonical)
                .is_some()
            || ImportedPhysicalType::canonical(&entry.alias).is_some_and(|t| t != entry.canonical)
        {
            return Err(SchemaAuthoringError::InvalidAlias(entry.alias.clone()));
        }
    }
    let mut result = NormalizedImportProfile {
        profile: profile.clone(),
        changes: Vec::new(),
    };
    for trace in &mut result.profile.traces {
        for field in &mut trace.fields {
            let before = &field.reviewed_physical_type;
            let canonical = ImportedPhysicalType::canonical(before)
                .or_else(|| table.get(before.as_str()).copied())
                .ok_or_else(|| SchemaAuthoringError::UnmappedAlias {
                    trace_id: trace.trace_id.clone(),
                    field_id: field.field_id.clone(),
                    alias: before.clone(),
                })?;
            if before != canonical.tag() {
                result.changes.push(TypeNormalization {
                    trace_id: trace.trace_id.clone(),
                    field_id: field.field_id.clone(),
                    before: before.clone(),
                    after: canonical.tag().into(),
                });
                field.reviewed_physical_type = canonical.tag().into();
            }
        }
    }
    Ok(result)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ImportedNoteDisposition {
    Metadata { reason: String },
    Mapped { rule_ids: Vec<String> },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportedNoteReview {
    pub note_sha256: String,
    pub disposition: ImportedNoteDisposition,
}
pub(crate) fn note_hash(note: &str) -> String {
    Sha256Digest(Sha256::digest(note.as_bytes()).into()).to_string()
}
pub(crate) fn valid_disposition(
    disposition: &ImportedNoteDisposition,
    rules: &[crate::RawRule],
) -> bool {
    match disposition {
        ImportedNoteDisposition::Metadata { reason } => !reason.trim().is_empty(),
        ImportedNoteDisposition::Mapped { rule_ids } => {
            let mut seen = std::collections::BTreeSet::new();
            !rule_ids.is_empty()
                && rule_ids
                    .iter()
                    .all(|id| seen.insert(id) && rules.iter().filter(|r| r.id == *id).count() == 1)
        }
    }
}
/// Records the caller's explicit classification, never performs that review.
/// The text digest prevents an old acknowledgement applying to changed notes.
pub fn review_imported_note(
    raw: &mut RawFlowSchema,
    trace_id: &str,
    field_id: &str,
    disposition: ImportedNoteDisposition,
) -> Result<(), SchemaAuthoringError> {
    let fail = SchemaAuthoringError::InvalidNoteReview;
    if raw.traces.iter().filter(|t| t.id == trace_id).count() != 1 {
        return Err(fail("trace is missing or ambiguous"));
    }
    let trace = raw
        .traces
        .iter_mut()
        .find(|t| t.id == trace_id)
        .expect("unique trace checked");
    if !valid_disposition(&disposition, &trace.rules) {
        return Err(fail(
            "classification needs a reason or existing unique rule references",
        ));
    }
    if trace.fields.iter().filter(|f| f.id == field_id).count() != 1 {
        return Err(fail("field is missing or ambiguous"));
    }
    let field = trace
        .fields
        .iter_mut()
        .find(|f| f.id == field_id)
        .expect("unique field checked");
    let metadata = field
        .regulatory_metadata
        .as_mut()
        .ok_or_else(|| fail("no imported note"))?;
    let note = metadata
        .get("xlsx_source")
        .and_then(|v| v.get("notes_original"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| fail("no imported note"))?;
    if metadata.get("source_note").and_then(|v| v.as_str()) != Some(note) {
        return Err(fail("source note disagrees with the imported original"));
    }
    let review = ImportedNoteReview {
        note_sha256: note_hash(note),
        disposition,
    };
    // This DTO contains only strings/vectors, so serialization is infallible.
    metadata.insert(
        "xlsx_note_review".into(),
        serde_json::to_value(review).expect("string-only review DTO"),
    );
    Ok(())
}
