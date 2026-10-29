//! Post-analysis JSONL to XLSX export. No ValidationSink or engine loop wiring.
use crate::{ERROR_COLUMNS, ReportError, ValidationError, ValidationSummary};
use rust_xlsxwriter::{DocProperties, Workbook, Worksheet, XlsxError};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    error::Error,
    fmt,
    io::{self, BufRead, Write},
    path::PathBuf,
    sync::Arc,
};

pub const XLSX_SHEET_ROWS: u32 = 1_048_576;
const WIDTHS: [f64; 14] = [
    28., 16., 18., 16., 14., 24., 22., 20., 30., 12., 12., 38., 38., 70.,
];
#[derive(Debug)]
pub enum XlsxReportError {
    Summary(ReportError),
    InvalidRowLimit,
    CancelledRun,
    Read(io::Error),
    Write(io::Error),
    JsonLine {
        line: u64,
        source: serde_json::Error,
    },
    Diagnostic {
        line: u64,
        source: crate::InvalidDiagnostic,
    },
    InvalidCode {
        line: u64,
        source: crate::InvalidValidationCode,
    },
    SummaryMismatch,
    CounterOverflow,
    Backend(XlsxError),
}
impl fmt::Display for XlsxReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Summary(_) => "XLSX summary is invalid",
            Self::InvalidRowLimit => {
                "XLSX sheet row limit must include a header and at least one data row"
            }
            Self::CancelledRun => "cancelled validation cannot produce a final XLSX report",
            Self::Read(_) => "cannot read canonical JSONL report",
            Self::Write(_) => "cannot flush XLSX output",
            Self::JsonLine { .. } => "canonical JSONL contains a malformed diagnostic line",
            Self::Diagnostic { .. } | Self::InvalidCode { .. } => {
                "canonical JSONL contains an invalid diagnostic"
            }
            Self::SummaryMismatch => "canonical JSONL differs from the supplied summary",
            Self::CounterOverflow => "XLSX report counter overflow",
            Self::Backend(_) => "XLSX workbook generation failed",
        })
    }
}
impl Error for XlsxReportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Summary(source) => Some(source),
            Self::Read(source) | Self::Write(source) => Some(source),
            Self::JsonLine { source, .. } => Some(source),
            Self::Diagnostic { source, .. } => Some(source),
            Self::InvalidCode { source, .. } => Some(source),
            Self::Backend(source) => Some(source),
            _ => None,
        }
    }
}
impl From<XlsxError> for XlsxReportError {
    fn from(source: XlsxError) -> Self {
        Self::Backend(source)
    }
}

/// Only configuration is retained. Row memory is one JSONL diagnostic plus the
/// backend's current row, not the complete input/report. Sheet metadata grows
/// with the number of continuation sheets, as required by the XLSX container.
pub struct XlsxReportWriter {
    row_limit: u32,
    temp_dir: Option<PathBuf>,
}
impl Default for XlsxReportWriter {
    fn default() -> Self {
        Self::new()
    }
}
impl XlsxReportWriter {
    pub fn new() -> Self {
        Self {
            row_limit: XLSX_SHEET_ROWS,
            temp_dir: None,
        }
    }
    /// Useful for bounded rollover tests; never permits exceeding Excel's limit.
    pub fn with_sheet_row_limit(limit: u32) -> Result<Self, XlsxReportError> {
        if !(2..=XLSX_SHEET_ROWS).contains(&limit) {
            return Err(XlsxReportError::InvalidRowLimit);
        }
        Ok(Self {
            row_limit: limit,
            temp_dir: None,
        })
    }
    pub fn with_temp_dir(mut self, directory: PathBuf) -> Self {
        self.temp_dir = Some(directory);
        self
    }
    /// Read source strictly in order; publish no bytes until parsing/count/scope
    /// checks finish. The caller owns atomic file publication and rollback.
    pub fn write<R: BufRead, W: Write + Send>(
        self,
        mut source: R,
        summary: &ValidationSummary,
        mut output: W,
    ) -> Result<W, XlsxReportError> {
        let summary_bytes =
            crate::write_run_summary(Vec::new(), summary).map_err(XlsxReportError::Summary)?;
        if summary.run_status == crate::RunStatus::Cancelled {
            return Err(XlsxReportError::CancelledRun);
        }
        // Serializer already checked timestamp/audit; retain all current/future
        // summary fields, including reference completeness and provenance.
        let document: serde_json::Value = serde_json::from_slice(&summary_bytes)
            .map_err(|source| XlsxReportError::Summary(ReportError::Serialize(source)))?;
        let mut workbook = Workbook::new();
        if let Some(directory) = self.temp_dir {
            workbook.set_tempdir(directory)?;
        }
        if let Some(timestamp) = document["started_at"].as_str() {
            workbook.set_properties(
                &DocProperties::new().set_creation_datetime(AuditTimestamp(timestamp)),
            );
        }
        summary_sheet(&mut workbook, &document)?;
        error_sheet(&mut workbook, 1)?;
        let mut sheet = 1usize;
        let mut row = 1u32;
        let mut count = 0u64;
        let mut by_code = BTreeMap::new();
        let mut buffer = Vec::new();
        loop {
            buffer.clear();
            if source
                .read_until(b'\n', &mut buffer)
                .map_err(XlsxReportError::Read)?
                == 0
            {
                break;
            }
            let line = count
                .checked_add(1)
                .ok_or(XlsxReportError::CounterOverflow)?;
            // serde_json rejects BOM, blank lines, trailing garbage, duplicate
            // known properties and invalid UTF-8; no permissive repair/skip.
            let error = read_canonical_diagnostic(&buffer, line)?;
            let in_scope = match &summary.scope_summary {
                crate::ScopeSummary::Trace { trace_id } => trace_id == &error.trace_id,
                crate::ScopeSummary::Flow { trace_ids } => trace_ids.contains(&error.trace_id),
            };
            if error.flow_id != summary.schema_key.flow_id
                || error.schema_version != summary.schema_key.version
                || !in_scope
            {
                return Err(XlsxReportError::SummaryMismatch);
            }
            if row == self.row_limit {
                finish_error_sheet(workbook.worksheet_from_index(sheet)?, row)?;
                sheet = sheet
                    .checked_add(1)
                    .ok_or(XlsxReportError::CounterOverflow)?;
                error_sheet(&mut workbook, sheet)?;
                row = 1;
            }
            let worksheet = workbook.worksheet_from_index(sheet)?;
            for (column, descriptor) in ERROR_COLUMNS.iter().enumerate() {
                let value = descriptor
                    .value(&error)
                    .map_err(|source| XlsxReportError::Diagnostic { line, source })?;
                worksheet.write_string(row, column as u16, value.as_ref())?;
            }
            row += 1;
            count = line;
            let n = by_code.entry(error.code).or_insert(0u64);
            *n = n.checked_add(1).ok_or(XlsxReportError::CounterOverflow)?;
        }
        let expected: BTreeMap<_, _> = summary
            .errors_by_code
            .iter()
            .filter(|(_, n)| **n > 0)
            .map(|(code, n)| (code.clone(), *n))
            .collect();
        if count != summary.total_errors || by_code != expected {
            return Err(XlsxReportError::SummaryMismatch);
        }
        finish_error_sheet(workbook.worksheet_from_index(sheet)?, row)?;
        workbook.save_to_writer(&mut output)?;
        output.flush().map_err(XlsxReportError::Write)?;
        Ok(output)
    }
}
/// The canonical serializer already verified UTC/RFC3339. This adapter avoids
/// enabling optional backend clock/date features or using the host wall clock.
struct AuditTimestamp<'a>(&'a str);
impl rust_xlsxwriter::IntoCustomDateTime for AuditTimestamp<'_> {
    fn utc_datetime(self) -> String {
        self.0.to_owned()
    }
}
fn error_sheet(workbook: &mut Workbook, number: usize) -> Result<(), XlsxReportError> {
    let sheet = workbook.add_worksheet_with_constant_memory()?;
    sheet.set_name(if number == 1 {
        "Errori".into()
    } else {
        format!("Errori ({number})")
    })?;
    sheet.set_freeze_panes(1, 0)?;
    for (column, descriptor) in ERROR_COLUMNS.iter().enumerate() {
        sheet.set_column_width(column as u16, WIDTHS[column])?;
        sheet.write_string(0, column as u16, descriptor.header())?;
    }
    Ok(())
}
fn finish_error_sheet(sheet: &mut Worksheet, next_row: u32) -> Result<(), XlsxReportError> {
    sheet.autofilter(0, 0, next_row - 1, 13)?;
    Ok(())
}
fn summary_sheet(
    workbook: &mut Workbook,
    document: &serde_json::Value,
) -> Result<(), XlsxReportError> {
    let sheet = workbook.add_worksheet_with_constant_memory()?;
    sheet.set_name("Riepilogo")?.set_freeze_panes(1, 0)?;
    sheet.set_column_width(0, 70.)?.set_column_width(1, 70.)?;
    sheet
        .write_string(0, 0, "property")?
        .write_string(0, 1, "value")?;
    let mut row = 1u32;
    summary_values(sheet, &mut row, "", document)?;
    Ok(())
}
/// Flatten the canonical summary into deterministic JSON Pointer paths. Arrays
/// retain order; empty arrays/objects remain explicit, never silently omitted.
fn summary_values(
    sheet: &mut Worksheet,
    row: &mut u32,
    path: &str,
    value: &serde_json::Value,
) -> Result<(), XlsxReportError> {
    match value {
        serde_json::Value::Object(map) if !map.is_empty() => {
            for (key, value) in map {
                let key = key.replace('~', "~0").replace('/', "~1");
                summary_values(sheet, row, &format!("{path}/{key}"), value)?;
            }
        }
        serde_json::Value::Array(array) if !array.is_empty() => {
            for (index, value) in array.iter().enumerate() {
                summary_values(sheet, row, &format!("{path}/{index}"), value)?;
            }
        }
        value => {
            let text = match value {
                serde_json::Value::String(text) => text.clone(),
                _ => value.to_string(),
            };
            sheet
                .write_string(*row, 0, path)?
                .write_string(*row, 1, &text)?;
            *row = row.checked_add(1).ok_or(XlsxReportError::CounterOverflow)?;
        }
    }
    Ok(())
}

/// One canonical decoder shared by XLSX and post-analysis CSV/JSONL replay.
/// Domain checks and exact wire names remain in this one implementation.
pub(crate) fn read_canonical_diagnostic(
    bytes: &[u8],
    line: u64,
) -> Result<ValidationError, XlsxReportError> {
    let diagnostic: JsonDiagnostic = serde_json::from_slice(bytes)
        .map_err(|source| XlsxReportError::JsonLine { line, source })?;
    diagnostic.into_error(line)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JsonDiagnostic {
    code: String,
    flow: String,
    schema_version: String,
    trace: String,
    line: u64,
    record_identity: JsonIdentity,
    field: Option<JsonField>,
    actual: Option<JsonValue>,
    expected: Option<String>,
    context: Vec<JsonContext>,
    message: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JsonIdentity {
    sioc_record_id: Option<String>,
    fiscal_code: Option<String>,
    episode_id: Option<String>,
    technical_ids: BTreeMap<String, String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JsonField {
    id: String,
    name: String,
    start: usize,
    end: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JsonValue {
    display: String,
    raw_hex: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JsonContext {
    trace: String,
    field_id: String,
    value: JsonValue,
}
impl From<JsonValue> for crate::DiagnosticValue {
    fn from(value: JsonValue) -> Self {
        Self {
            display: value.display,
            raw_hex: value.raw_hex,
        }
    }
}
impl JsonDiagnostic {
    fn into_error(self, line: u64) -> Result<ValidationError, XlsxReportError> {
        let error = ValidationError {
            code: self
                .code
                .parse()
                .map_err(|source| XlsxReportError::InvalidCode { line, source })?,
            flow_id: crate::FlowId(self.flow),
            schema_version: crate::SchemaVersion(self.schema_version),
            trace_id: crate::TraceId(self.trace),
            line_number: self.line,
            record_identity: crate::RecordIdentity {
                sioc_record_id: self.record_identity.sioc_record_id.map(Arc::from),
                fiscal_code: self.record_identity.fiscal_code.map(Arc::from),
                episode_id: self.record_identity.episode_id.map(Arc::from),
                technical_ids: self
                    .record_identity
                    .technical_ids
                    .into_iter()
                    .map(|(key, value)| (Arc::from(key), Arc::from(value)))
                    .collect(),
            },
            field: self.field.map(|field| crate::FieldDiagnostic {
                id: crate::FieldId(field.id),
                name: Arc::from(field.name),
                start_one_based: field.start,
                end_one_based_inclusive: field.end,
            }),
            actual: self.actual.map(Into::into),
            expected: self.expected,
            context: self
                .context
                .into_iter()
                .map(|item| crate::DiagnosticContextItem {
                    trace_id: crate::TraceId(item.trace),
                    field_id: crate::FieldId(item.field_id),
                    value: item.value.into(),
                })
                .collect(),
            message: self.message,
        };
        error
            .validate()
            .map_err(|source| XlsxReportError::Diagnostic { line, source })?;
        Ok(error)
    }
}

#[cfg(test)]
mod tests {
    use crate::ERROR_COLUMNS;
    use rust_xlsxwriter::{Workbook, XlsxError};
    use std::{
        fs,
        io::{self, Cursor, Read, Write},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn workbook() -> Workbook {
        let mut workbook = Workbook::new();
        workbook
            .add_worksheet_with_constant_memory()
            .unwrap()
            .set_name("Riepilogo")
            .unwrap()
            .write_string(0, 0, "Synthetic backend probe")
            .unwrap();
        let sheet = workbook.add_worksheet_with_constant_memory().unwrap();
        sheet.set_name("Errori").unwrap();
        for (column, descriptor) in ERROR_COLUMNS.iter().enumerate() {
            sheet
                .write_string(0, column.try_into().unwrap(), descriptor.header())
                .unwrap();
        }
        // Explicit strings: neither leading zeros nor formula-looking values
        // may be reinterpreted by the writer chosen for the future report.
        sheet.write_string(1, 0, "00123").unwrap();
        sheet.write_string(1, 1, "=1+1").unwrap();
        sheet.write_string(1, 2, "à<&\"\n").unwrap();
        workbook
    }
    fn xml(bytes: &[u8], name: &str) -> String {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let mut text = String::new();
        archive
            .by_name(name)
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        text
    }

    #[test]
    fn constant_memory_workbook_preserves_text_and_shared_column_headers() {
        let bytes = workbook().save_to_buffer().unwrap();
        assert!(bytes.starts_with(b"PK\x03\x04"));
        let metadata = xml(&bytes, "xl/workbook.xml");
        assert!(metadata.contains("name=\"Riepilogo\""));
        assert!(metadata.contains("name=\"Errori\""));
        let sheet = xml(&bytes, "xl/worksheets/sheet2.xml");
        for column in ERROR_COLUMNS {
            assert!(sheet.contains(&format!("<t>{}</t>", column.header())));
        }
        assert!(sheet.contains("t=\"inlineStr\""));
        assert!(sheet.contains("<t>00123</t>"));
        assert!(sheet.contains("<t>=1+1</t>"));
        assert!(sheet.contains("à&lt;&amp;\""));
        assert!(!sheet.contains("<f>"));
        assert!(
            zip::ZipArchive::new(Cursor::new(&bytes))
                .unwrap()
                .by_name("xl/sharedStrings.xml")
                .is_err()
        );
    }

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "lintra xlsx à {} {}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn native_workbook_file_with_unicode_path_is_a_readable_zip() {
        let directory = Directory::new();
        let path = directory.0.join("report à.xlsx");
        workbook().save(&path).unwrap();
        let bytes = fs::read(path).unwrap();
        assert!(xml(&bytes, "xl/worksheets/sheet2.xml").contains("<t>00123</t>"));
    }

    #[test]
    fn sequential_constant_memory_rows_survive_flushes_without_shared_string_table() {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet_with_constant_memory().unwrap();
        for row in 0..10_000 {
            sheet.write_string(row, 0, format!("{row:05}")).unwrap();
        }
        let bytes = workbook.save_to_buffer().unwrap();
        let sheet = xml(&bytes, "xl/worksheets/sheet1.xml");
        assert_eq!(sheet.matches("<row ").count(), 10_000);
        assert!(sheet.contains("<t>00000</t>"));
        assert!(sheet.contains("<t>09999</t>"));
        assert!(!sheet.contains("t=\"s\""));
    }

    #[test]
    fn backend_errors_include_unusable_tempdir_and_output_failure() {
        let directory = Directory::new();
        assert!(matches!(
            Workbook::new().set_tempdir(directory.0.join("missing")),
            Err(XlsxError::TempFileError(_))
        ));
        struct FailingWriter;
        impl Write for FailingWriter {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(
                    io::ErrorKind::StorageFull,
                    "synthetic output",
                ))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let error = workbook().save_to_writer(FailingWriter).unwrap_err();
        let source = match error {
            XlsxError::IoError(source) | XlsxError::ZipError(zip::result::ZipError::Io(source)) => {
                source
            }
            other => panic!("unexpected technical cause: {other:?}"),
        };
        assert_eq!(source.kind(), io::ErrorKind::StorageFull);
    }
}
