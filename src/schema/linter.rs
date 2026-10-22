//! Structural lint only. An empty result is not a compiled or executable schema.

use super::{
    compatibility::{EngineCapabilities, SchemaFormatVersion},
    raw::{RawFlowSchema, RawLexical, RawPhysical, RawRecordSchema},
};
use crate::domain::{FieldId, FlowId, TraceId};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaIssueSeverity {
    Error,
    Warning,
}

/// Codes implemented by structural lint. Later compiler stages may extend this set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaIssueCode {
    InvalidIdentifier,
    UnsupportedSchemaFormat,
    EmptySchema,
    DuplicateTraceId,
    DuplicateFieldId,
    InvalidPosition,
    ZeroLength,
    RangeOverflow,
    InvalidFieldOrder,
    Gap,
    Overlap,
    RecordLengthMismatch,
    InvalidTypeParameters,
    MissingEncodingBinding,
    UnsupportedEncoding,
    MissingCapability,
    UnsupportedCapability,
    UnsupportedConstruct,
    InvalidSchemaValue,
    InvalidRegex,
    ConstraintTypeMismatch,
    SelfReferentialRule,
    PresenceConflict,
    InvalidRecordKey,
    MissingFieldReference,
    MissingDerivedValueReference,
    InvalidDiagnosticCode,
    InternalCompilerInvariant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaIssue {
    pub severity: SchemaIssueSeverity,
    pub code: SchemaIssueCode,
    pub flow_id: Option<FlowId>,
    pub trace_id: Option<TraceId>,
    pub field_id: Option<FieldId>,
    pub message: String,
    pub source_note: Option<String>,
}

struct Diagnostics<'a> {
    raw: &'a RawFlowSchema,
    issues: Vec<SchemaIssue>,
}

impl Diagnostics<'_> {
    fn error(
        &mut self,
        code: SchemaIssueCode,
        trace: Option<&str>,
        field: Option<&str>,
        message: impl Into<String>,
    ) {
        self.issues.push(SchemaIssue {
            severity: SchemaIssueSeverity::Error,
            code,
            flow_id: Some(FlowId(self.raw.flow_id.clone())),
            trace_id: trace.map(|s| TraceId(s.to_owned())),
            field_id: field.map(|s| FieldId(s.to_owned())),
            message: message.into(),
            // Open metadata is not a standardized source-note contract.
            source_note: None,
        });
    }

    fn identifier(&mut self, id: &str, trace: Option<&str>, field: Option<&str>) {
        if id.is_empty() || id.trim() != id {
            self.error(
                SchemaIssueCode::InvalidIdentifier,
                trace,
                field,
                "identifier must be nonempty and have no surrounding whitespace",
            );
        }
    }

    fn record(&mut self, record: &RawRecordSchema) {
        let trace = Some(record.id.as_str());
        let mut ids = HashSet::new();
        // Each field owns its diagnostics; emission below follows declaration order,
        // even though coverage is computed in source-position order.
        let mut field_issues = vec![Vec::new(); record.fields.len()];
        let mut intervals = Vec::new();
        let mut previous_start = None;
        for (index, field) in record.fields.iter().enumerate() {
            let issues = &mut field_issues[index];
            if field.id.is_empty() || field.id.trim() != field.id {
                issues.push((
                    SchemaIssueCode::InvalidIdentifier,
                    "field identifier must be nonempty and have no surrounding whitespace"
                        .to_owned(),
                ));
            }
            if !ids.insert(field.id.as_str()) {
                issues.push((
                    SchemaIssueCode::DuplicateFieldId,
                    "duplicate field identifier in this trace".to_owned(),
                ));
            }
            let (start, length) = match &field.physical {
                RawPhysical::Numeric {
                    start,
                    length,
                    lexical,
                    ..
                }
                | RawPhysical::Alphanumeric {
                    start,
                    length,
                    lexical,
                    ..
                } => {
                    if let RawLexical::ByteSet {
                        allowed_bytes,
                        lowercase_bytes,
                    } = lexical
                    {
                        let allowed: HashSet<_> = allowed_bytes.iter().collect();
                        let lower: HashSet<_> = lowercase_bytes.iter().collect();
                        if allowed.is_empty()
                            || allowed.len() != allowed_bytes.len()
                            || lower.len() != lowercase_bytes.len()
                            || !lower.is_subset(&allowed)
                        {
                            issues.push((SchemaIssueCode::InvalidTypeParameters, "byte sets must be unique, allowed_bytes nonempty, and lowercase_bytes a subset".to_owned()));
                        }
                    }
                    (*start, *length)
                }
                RawPhysical::Decimal {
                    start,
                    length,
                    scale,
                    ..
                } => {
                    if self.raw.schema_format_version != 2 {
                        issues.push((
                            SchemaIssueCode::UnsupportedConstruct,
                            "Decimal requires schema_format_version 2".to_owned(),
                        ));
                    }
                    if *length < 4 || *scale != 2 {
                        issues.push((
                            SchemaIssueCode::InvalidTypeParameters,
                            "monetary Decimal requires width >= 4 and explicit scale 2".to_owned(),
                        ));
                    }
                    (*start, *length)
                }
                RawPhysical::Date { start, length, .. } => {
                    if *length != 8 {
                        issues.push((
                            SchemaIssueCode::InvalidTypeParameters,
                            "DDMMYYYY requires exactly eight bytes".to_owned(),
                        ));
                    }
                    (*start, *length)
                }
            };
            if start == 0 {
                issues.push((
                    SchemaIssueCode::InvalidPosition,
                    "source start must be greater than zero".to_owned(),
                ));
            }
            if length == 0 {
                issues.push((
                    SchemaIssueCode::ZeroLength,
                    "field length must be greater than zero".to_owned(),
                ));
            }
            if previous_start.is_some_and(|previous| start < previous) {
                issues.push((
                    SchemaIssueCode::InvalidFieldOrder,
                    "fields must be declared in physical source order".to_owned(),
                ));
            }
            previous_start = Some(start);
            if start > 0 && length > 0 {
                let runtime_end = usize::try_from(start)
                    .ok()
                    .zip(usize::try_from(length).ok())
                    .and_then(|(start, length)| checked_source_end(start, length).ok());
                if runtime_end.is_none() {
                    issues.push((
                        SchemaIssueCode::RangeOverflow,
                        "field range cannot be represented by usize".to_owned(),
                    ));
                }
                // Source coordinates stay one-based and inclusive. No ByteRange is
                // constructed here; the compiler owns the runtime conversion.
                intervals.push((
                    u64::from(start),
                    u64::from(start) + u64::from(length) - 1,
                    index,
                ));
            }
        }
        intervals.sort_by_key(|&(start, _, index)| (start, index));
        let mut covered_end = 0;
        for (start, end, index) in intervals {
            if start > covered_end + 1 {
                field_issues[index].push((
                    SchemaIssueCode::Gap,
                    format!(
                        "undeclared bytes {}..={} before field",
                        covered_end + 1,
                        start - 1
                    ),
                ));
            }
            if start <= covered_end {
                field_issues[index].push((
                    SchemaIssueCode::Overlap,
                    "field overlaps an earlier source range".to_owned(),
                ));
            }
            covered_end = covered_end.max(end);
        }
        for (field, issues) in record.fields.iter().zip(field_issues) {
            for (code, message) in issues {
                self.error(code, trace, Some(&field.id), message);
            }
        }
        if record.record_length == 0
            || record.fields.is_empty()
            || covered_end != u64::from(record.record_length)
        {
            self.error(SchemaIssueCode::RecordLengthMismatch, trace, None,
                format!("record_length {} differs from nonempty physical coverage ending at {covered_end}", record.record_length));
        }
    }
}

// Numeric source endpoint equals the zero-based exclusive endpoint, without
// converting or storing a runtime range. Checked on the actual target usize.
fn checked_source_end(start: usize, length: usize) -> Result<usize, SchemaIssueCode> {
    length
        .checked_sub(1)
        .and_then(|tail| start.checked_add(tail))
        .ok_or(SchemaIssueCode::RangeOverflow)
}

/// Checks format identity, physical layout and parameters of the format-1/2 types.
///
/// Order: flow issues, then traces in declaration order; trace identity before
/// fields, fields in declaration order, record-length issue last. No hash-map
/// iteration determines diagnostics. References, constraint compatibility,
/// encoding binding and supported runtime capabilities require compiler stages.
pub fn lint_raw_schema(raw: &RawFlowSchema) -> Vec<SchemaIssue> {
    let mut diagnostics = Diagnostics {
        raw,
        issues: Vec::new(),
    };
    diagnostics.identifier(&raw.flow_id, None, None);
    diagnostics.identifier(&raw.schema_version, None, None);
    if let Some(message) = EngineCapabilities::current()
        .incompatibility_message(SchemaFormatVersion(raw.schema_format_version))
    {
        diagnostics.error(
            SchemaIssueCode::UnsupportedSchemaFormat,
            None,
            None,
            message,
        );
    }
    if raw.traces.is_empty() {
        diagnostics.error(
            SchemaIssueCode::EmptySchema,
            None,
            None,
            "flow must contain at least one trace",
        );
    }
    let mut traces = HashSet::new();
    for record in &raw.traces {
        diagnostics.identifier(&record.id, Some(&record.id), None);
        if !traces.insert(record.id.as_str()) {
            diagnostics.error(
                SchemaIssueCode::DuplicateTraceId,
                Some(&record.id),
                None,
                "duplicate trace identifier in this flow",
            );
        }
        diagnostics.record(record);
    }
    diagnostics.issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_source_endpoint_handles_machine_boundaries() {
        assert_eq!(checked_source_end(1, 1), Ok(1));
        assert_eq!(checked_source_end(usize::MAX, 1), Ok(usize::MAX));
        assert_eq!(
            checked_source_end(usize::MAX, 2),
            Err(SchemaIssueCode::RangeOverflow)
        );
        assert_eq!(
            checked_source_end(1, 0),
            Err(SchemaIssueCode::RangeOverflow)
        );
        // Raw u32 components cannot overflow a 64-bit usize; they can on 32-bit.
        let end = usize::try_from(u32::MAX)
            .ok()
            .and_then(|n| checked_source_end(n, n).ok());
        assert_eq!(end.is_none(), usize::BITS == 32);
    }
}
