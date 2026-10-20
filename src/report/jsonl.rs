use super::sink::{ReportError, ValidationSink};
use crate::domain::{
    DiagnosticContextItem, DiagnosticValue, FieldDiagnostic, InvalidDiagnostic, RecordIdentity,
    ValidationError, ValidationSummary,
};
use serde::{
    Serialize, Serializer,
    ser::{SerializeMap, SerializeSeq},
};
use std::{collections::BTreeMap, io::Write, sync::Arc};

/// Error-only UTF-8 JSON Lines. Failed/partial output must not be published.
/// File publication and rollback belong to the separate atomic-output layer.
pub struct JsonlSink<W> {
    writer: W,
    failed: bool,
}
impl<W: Write> JsonlSink<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            failed: false,
        }
    }
    pub fn get_ref(&self) -> &W {
        &self.writer
    }

    /// Checks buffered write failures before returning the underlying writer.
    pub fn finish(mut self) -> Result<W, ReportError> {
        self.flush()?;
        Ok(self.writer)
    }
    fn flush(&mut self) -> Result<(), ReportError> {
        if self.failed {
            return Err(ReportError::PreviousFailure);
        }
        if let Err(source) = self.writer.flush() {
            self.failed = true;
            return Err(ReportError::Write(source));
        }
        Ok(())
    }
}
impl<W: Write> ValidationSink for JsonlSink<W> {
    fn on_error(&mut self, error: &ValidationError) -> Result<(), ReportError> {
        if self.failed {
            return Err(ReportError::PreviousFailure);
        }
        if let Err(source) = error.validate() {
            self.failed = true;
            return Err(ReportError::InvalidDiagnostic(source));
        }
        let code = error
            .code
            .as_symbol()
            .map_err(|e| ReportError::InvalidDiagnostic(InvalidDiagnostic::InvalidCode(e)))?;
        let line = ErrorLine {
            code,
            flow: &error.flow_id.0,
            schema_version: &error.schema_version.0,
            trace: &error.trace_id.0,
            line: error.line_number,
            record_identity: IdentityLine::from(&error.record_identity),
            field: error.field.as_ref().map(FieldLine::from),
            actual: error.actual.as_ref().map(ValueLine::from),
            expected: error.expected.as_deref(),
            context: ContextLines(&error.context),
            message: &error.message,
        };
        if let Err(source) = serde_json::to_writer(&mut self.writer, &line) {
            self.failed = true;
            return Err(ReportError::Serialize(source));
        }
        if let Err(source) = self.writer.write_all(b"\n") {
            self.failed = true;
            return Err(ReportError::Write(source));
        }
        Ok(())
    }
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        // Summary is a separate document. Only flush this error stream.
        self.flush()
    }
}

// Wire DTOs keep report names/order independent of domain representation.
#[derive(Serialize)]
struct ErrorLine<'a> {
    code: &'a str,
    flow: &'a str,
    schema_version: &'a str,
    trace: &'a str,
    line: u64,
    record_identity: IdentityLine<'a>,
    field: Option<FieldLine<'a>>,
    actual: Option<ValueLine<'a>>,
    expected: Option<&'a str>,
    context: ContextLines<'a>,
    message: &'a str,
}
#[derive(Serialize)]
struct FieldLine<'a> {
    id: &'a str,
    name: &'a str,
    start: usize,
    end: usize,
}
impl<'a> From<&'a FieldDiagnostic> for FieldLine<'a> {
    fn from(value: &'a FieldDiagnostic) -> Self {
        Self {
            id: &value.id.0,
            name: &value.name,
            start: value.start_one_based,
            end: value.end_one_based_inclusive,
        }
    }
}
#[derive(Serialize)]
struct ValueLine<'a> {
    display: &'a str,
    raw_hex: Option<&'a str>,
}
impl<'a> From<&'a DiagnosticValue> for ValueLine<'a> {
    fn from(value: &'a DiagnosticValue) -> Self {
        Self {
            display: &value.display,
            raw_hex: value.raw_hex.as_deref(),
        }
    }
}
#[derive(Serialize)]
struct IdentityLine<'a> {
    sioc_record_id: Option<&'a str>,
    fiscal_code: Option<&'a str>,
    episode_id: Option<&'a str>,
    technical_ids: TechnicalIds<'a>,
}
impl<'a> From<&'a RecordIdentity> for IdentityLine<'a> {
    fn from(value: &'a RecordIdentity) -> Self {
        Self {
            sioc_record_id: value.sioc_record_id.as_deref(),
            fiscal_code: value.fiscal_code.as_deref(),
            episode_id: value.episode_id.as_deref(),
            technical_ids: TechnicalIds(&value.technical_ids),
        }
    }
}
struct TechnicalIds<'a>(&'a BTreeMap<Arc<str>, Arc<str>>);
impl Serialize for TechnicalIds<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in self.0 {
            map.serialize_entry(key.as_ref(), value.as_ref())?;
        }
        map.end()
    }
}
#[derive(Serialize)]
struct ContextLine<'a> {
    trace: &'a str,
    field_id: &'a str,
    value: ValueLine<'a>,
}
struct ContextLines<'a>(&'a [DiagnosticContextItem]);
impl Serialize for ContextLines<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for item in self.0 {
            seq.serialize_element(&ContextLine {
                trace: &item.trace_id.0,
                field_id: &item.field_id.0,
                value: ValueLine::from(&item.value),
            })?;
        }
        seq.end()
    }
}
