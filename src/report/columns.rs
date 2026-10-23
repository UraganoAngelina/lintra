//! Shared error projection for CSV and the post-analysis XLSX writer (DET A.2).
use crate::ValidationError;
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorColumn {
    Code,
    Flow,
    SchemaVersion,
    Trace,
    Line,
    RecordId,
    FiscalCode,
    FieldId,
    FieldName,
    Start,
    End,
    Actual,
    Expected,
    Message,
}

/// The sole ordered list of the fourteen error columns specified by DET A.2.
pub const ERROR_COLUMNS: [ErrorColumn; 14] = [
    ErrorColumn::Code,
    ErrorColumn::Flow,
    ErrorColumn::SchemaVersion,
    ErrorColumn::Trace,
    ErrorColumn::Line,
    ErrorColumn::RecordId,
    ErrorColumn::FiscalCode,
    ErrorColumn::FieldId,
    ErrorColumn::FieldName,
    ErrorColumn::Start,
    ErrorColumn::End,
    ErrorColumn::Actual,
    ErrorColumn::Expected,
    ErrorColumn::Message,
];

impl ErrorColumn {
    pub const fn header(self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Flow => "flow",
            Self::SchemaVersion => "schema_version",
            Self::Trace => "trace",
            Self::Line => "line",
            Self::RecordId => "record_id",
            Self::FiscalCode => "fiscal_code",
            Self::FieldId => "field_id",
            Self::FieldName => "field_name",
            Self::Start => "start",
            Self::End => "end",
            Self::Actual => "actual",
            Self::Expected => "expected",
            Self::Message => "message",
        }
    }

    /// Caller validates the diagnostic before output. No value is trimmed,
    /// padded, converted to a number or interpreted as a spreadsheet formula.
    pub fn value(self, error: &ValidationError) -> Result<Cow<'_, str>, crate::InvalidDiagnostic> {
        let field = error.field.as_ref();
        let text = match self {
            Self::Code => error
                .code
                .as_symbol()
                .map_err(crate::InvalidDiagnostic::InvalidCode)?,
            Self::Flow => &error.flow_id.0,
            Self::SchemaVersion => &error.schema_version.0,
            Self::Trace => &error.trace_id.0,
            Self::Line => return Ok(Cow::Owned(error.line_number.to_string())),
            Self::RecordId => error
                .record_identity
                .sioc_record_id
                .as_deref()
                .unwrap_or(""),
            Self::FiscalCode => error.record_identity.fiscal_code.as_deref().unwrap_or(""),
            Self::FieldId => field.map_or("", |f| f.id.0.as_str()),
            Self::FieldName => field.map_or("", |f| f.name.as_ref()),
            Self::Start => {
                return Ok(field.map_or(Cow::Borrowed(""), |f| {
                    Cow::Owned(f.start_one_based.to_string())
                }));
            }
            Self::End => {
                return Ok(field.map_or(Cow::Borrowed(""), |f| {
                    Cow::Owned(f.end_one_based_inclusive.to_string())
                }));
            }
            Self::Actual => error.actual.as_ref().map_or("", |v| v.display.as_str()),
            Self::Expected => error.expected.as_deref().unwrap_or(""),
            Self::Message => &error.message,
        };
        Ok(Cow::Borrowed(text))
    }
}
