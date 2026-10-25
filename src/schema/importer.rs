//! Offline XLSX authoring adapter. Never called by the validation engine.
use super::raw::RawPhysical;
use crate::{CanonicalSchemaError, RawFlowSchema, schema_fingerprint};
use calamine::{Data, Range, Reader, Xlsx, XlsxError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, error::Error, fmt, fs, io::Cursor, path::Path};

pub trait SchemaImporter {
    /// Returns a draft raw model, not a verified or executable schema.
    fn import(&self, source: &Path) -> Result<RawFlowSchema, ImportError>;
}
#[derive(Debug)]
pub enum ImportError {
    Io(std::io::Error),
    Workbook(XlsxError),
    Canonical(CanonicalSchemaError),
    InvalidProfile {
        reason: &'static str,
    },
    Cell {
        sheet: String,
        row: u32,
        column: u32,
        reason: &'static str,
    },
    UnexpectedRow {
        sheet: String,
        row: u32,
    },
    MacroWorkbook,
}
impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(_) => f.write_str("schema import input could not be read"),
            Self::Workbook(_) => f.write_str("schema import workbook could not be decoded"),
            Self::Canonical(_) => f.write_str("schema import blueprint fingerprint failed"),
            Self::InvalidProfile { reason } => write!(f, "schema import profile: {reason}"),
            Self::Cell {
                sheet,
                row,
                column,
                reason,
            } => write!(
                f,
                "schema import {sheet} row {row} column {column}: {reason}"
            ),
            Self::UnexpectedRow { sheet, row } => {
                write!(f, "schema import {sheet} row {row}: unbound source ID")
            }
            Self::MacroWorkbook => f.write_str("schema import does not accept VBA workbooks"),
        }
    }
}
impl Error for ImportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(cause) => Some(cause),
            Self::Workbook(cause) => Some(cause),
            Self::Canonical(cause) => Some(cause),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateFormulaPolicy {
    Reject,
    /// Use a numeric cached value, preserving formula text as unverified evidence.
    /// This never recalculates Excel formulas or guarantees cache freshness.
    UseCachedValues,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XlsxColumn {
    /// One-based Excel column number.
    pub number: u32,
    pub header: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XlsxColumns {
    pub field_id: XlsxColumn,
    pub key_marker: XlsxColumn,
    pub name: XlsxColumn,
    pub length: XlsxColumn,
    pub start: XlsxColumn,
    pub source_type: XlsxColumn,
    pub obligation: XlsxColumn,
    pub notes: XlsxColumn,
}
impl XlsxColumns {
    fn all(&self) -> [&XlsxColumn; 8] {
        [
            &self.field_id,
            &self.key_marker,
            &self.name,
            &self.length,
            &self.start,
            &self.source_type,
            &self.obligation,
            &self.notes,
        ]
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XlsxFieldRow {
    pub row: u32,
    pub field_id: String,
    /// Expected tokens are explicit reviewed bindings, not guessed type aliases.
    pub source_type: Option<String>,
    pub obligation: String,
    pub key_marker: Option<String>,
    /// Canonical type of the reviewed enrichment, guarded at construction.
    pub reviewed_physical_type: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XlsxTraceBlock {
    pub trace_id: String,
    pub heading_row: u32,
    pub heading: String,
    pub fields: Vec<XlsxFieldRow>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XlsxImportProfile {
    pub flow_id: String,
    pub blueprint_version: String,
    pub sheet: String,
    pub header_row: u32,
    pub columns: XlsxColumns,
    pub traces: Vec<XlsxTraceBlock>,
    pub coordinate_formulas: CoordinateFormulaPolicy,
    /// Preserve text cells of these sheets as source annotations, not DSL.
    pub annotation_sheets: Vec<String>,
}
/// The blueprint supplies reviewed semantics absent from the spreadsheet:
/// missing/alignment, conditional presence, rules, joins, identity, persistence.
/// Import replaces actual names/coordinates/notes only after source guards pass.
#[derive(Debug, Clone)]
pub struct XlsxSchemaImporter {
    blueprint: RawFlowSchema,
    profile: XlsxImportProfile,
}
fn invalid(reason: &'static str) -> ImportError {
    ImportError::InvalidProfile { reason }
}
fn physical_type(physical: &RawPhysical) -> &'static str {
    match physical {
        RawPhysical::Numeric { .. } => "numeric",
        RawPhysical::Alphanumeric { .. } => "alphanumeric",
        RawPhysical::Date { .. } => "date",
        RawPhysical::Decimal { .. } => "decimal",
    }
}
fn valid_row(row: u32) -> bool {
    (1..=1_048_576).contains(&row)
}
impl XlsxSchemaImporter {
    pub fn new(blueprint: RawFlowSchema, profile: XlsxImportProfile) -> Result<Self, ImportError> {
        if blueprint.flow_id != profile.flow_id
            || blueprint.schema_version != profile.blueprint_version
        {
            return Err(invalid("blueprint key does not match the profile"));
        }
        if profile.sheet.is_empty() || !valid_row(profile.header_row) || profile.traces.is_empty() {
            return Err(invalid("sheet, header and trace bindings are required"));
        }
        if blueprint.source.contains_key("xlsx_import")
            || blueprint.traces.len() != profile.traces.len()
        {
            return Err(invalid(
                "blueprint provenance collision or incomplete trace coverage",
            ));
        }
        let mut columns = BTreeSet::new();
        for column in profile.columns.all() {
            if !(1..=16_384).contains(&column.number)
                || column.header.is_empty()
                || !columns.insert(column.number)
            {
                return Err(invalid(
                    "columns must have distinct valid positions and exact headers",
                ));
            }
        }
        let mut rows = BTreeSet::from([profile.header_row]);
        let mut traces = BTreeSet::new();
        let mut previous = profile.header_row;
        for (trace, block) in blueprint.traces.iter().zip(&profile.traces) {
            if trace.id.is_empty()
                || trace.id != block.trace_id
                || !traces.insert(&trace.id)
                || trace.fields.len() != block.fields.len()
                || trace.fields.is_empty()
                || !valid_row(block.heading_row)
                || block.heading_row <= previous
                || block.heading.is_empty()
                || !rows.insert(block.heading_row)
            {
                return Err(invalid(
                    "trace bindings must cover the blueprint in worksheet order",
                ));
            }
            previous = block.heading_row;
            let mut ids = BTreeSet::new();
            for (field, binding) in trace.fields.iter().zip(&block.fields) {
                if field.id.is_empty()
                    || field.id != binding.field_id
                    || !ids.insert(&field.id)
                    || field
                        .regulatory_metadata
                        .as_ref()
                        .is_some_and(|m| m.contains_key("xlsx_source"))
                    || !valid_row(binding.row)
                    || binding.row <= previous
                    || !rows.insert(binding.row)
                    || binding.reviewed_physical_type != physical_type(&field.physical)
                    || binding.obligation.is_empty()
                    || binding.source_type.as_ref().is_some_and(String::is_empty)
                    || binding.key_marker.as_ref().is_some_and(String::is_empty)
                {
                    return Err(invalid(
                        "field bindings must be complete, unambiguous and explicitly typed",
                    ));
                }
                previous = binding.row;
            }
        }
        let mut sheets = BTreeSet::new();
        for sheet in &profile.annotation_sheets {
            if sheet.is_empty() || !sheets.insert(sheet) {
                return Err(invalid("annotation sheets must be explicit and unique"));
            }
        }
        Ok(Self { blueprint, profile })
    }
}
struct Sheet<'a> {
    name: &'a str,
    values: &'a Range<Data>,
    formulas: &'a Range<String>,
}
impl Sheet<'_> {
    fn error(&self, row: u32, column: u32, reason: &'static str) -> ImportError {
        ImportError::Cell {
            sheet: self.name.to_owned(),
            row,
            column,
            reason,
        }
    }
    fn formula(&self, row: u32, column: u32) -> Option<&str> {
        self.formulas
            .get_value((row - 1, column - 1))
            .map(String::as_str)
            .filter(|f| !f.is_empty())
    }
    fn text(&self, row: u32, column: u32) -> Result<Option<&str>, ImportError> {
        if self.formula(row, column).is_some() {
            return Err(self.error(row, column, "formula in a text cell"));
        }
        match self.values.get_value((row - 1, column - 1)) {
            Some(Data::String(value)) if !value.is_empty() => Ok(Some(value)),
            Some(Data::String(_) | Data::Empty) | None => Ok(None),
            _ => Err(self.error(
                row,
                column,
                "expected original text, not a numeric or converted value",
            )),
        }
    }
    fn required_text(&self, row: u32, column: u32) -> Result<&str, ImportError> {
        self.text(row, column)?
            .ok_or_else(|| self.error(row, column, "required text is missing"))
    }
    fn expect(&self, row: u32, column: u32, expected: Option<&str>) -> Result<(), ImportError> {
        if self.text(row, column)? != expected {
            return Err(self.error(row, column, "source does not match the reviewed binding"));
        }
        Ok(())
    }
    fn coordinate(
        &self,
        row: u32,
        column: u32,
        policy: CoordinateFormulaPolicy,
    ) -> Result<u32, ImportError> {
        if self.formula(row, column).is_some() && policy == CoordinateFormulaPolicy::Reject {
            return Err(self.error(row, column, "coordinate formula rejected by policy"));
        }
        match self.values.get_value((row - 1, column - 1)) {
            Some(Data::Int(n)) => {
                u32::try_from(*n).map_err(|_| self.error(row, column, "coordinate outside u32"))
            }
            Some(Data::Float(n))
                if n.is_finite() && n.fract() == 0.0 && *n >= 0.0 && *n <= u32::MAX as f64 =>
            {
                Ok(*n as u32)
            }
            _ => Err(self.error(
                row,
                column,
                "coordinate requires an integer value or numeric formula cache",
            )),
        }
    }
}
fn absolute(start: (u32, u32), row: usize, column: usize) -> Result<(u32, u32), ImportError> {
    let add = |base: u32, relative: usize| {
        u32::try_from(relative)
            .ok()
            .and_then(|n| base.checked_add(n))
            .and_then(|n| n.checked_add(1))
    };
    match (add(start.0, row), add(start.1, column)) {
        (Some(row), Some(column)) if valid_row(row) && column <= 16_384 => Ok((row, column)),
        _ => Err(invalid("worksheet coordinates exceed Excel bounds")),
    }
}
impl SchemaImporter for XlsxSchemaImporter {
    fn import(&self, source: &Path) -> Result<RawFlowSchema, ImportError> {
        // One snapshot binds the recorded digest to the exact bytes parsed.
        let bytes = fs::read(source).map_err(ImportError::Io)?;
        let mut workbook = Xlsx::new(Cursor::new(&bytes)).map_err(ImportError::Workbook)?;
        if workbook
            .vba_project()
            .map_err(ImportError::Workbook)?
            .is_some()
        {
            return Err(ImportError::MacroWorkbook);
        }
        let values = workbook
            .worksheet_range(&self.profile.sheet)
            .map_err(ImportError::Workbook)?;
        let formulas = workbook
            .worksheet_formula(&self.profile.sheet)
            .map_err(ImportError::Workbook)?;
        let sheet = Sheet {
            name: &self.profile.sheet,
            values: &values,
            formulas: &formulas,
        };
        let columns = &self.profile.columns;
        for column in columns.all() {
            sheet.expect(self.profile.header_row, column.number, Some(&column.header))?;
        }
        let mut id_rows = BTreeSet::from([self.profile.header_row]);
        let mut geometry_rows = id_rows.clone();
        for block in &self.profile.traces {
            sheet.expect(
                block.heading_row,
                columns.field_id.number,
                Some(&block.heading),
            )?;
            id_rows.insert(block.heading_row);
            for field in &block.fields {
                id_rows.insert(field.row);
                geometry_rows.insert(field.row);
            }
        }
        // Reject unbound IDs/geometry even outside the declared field blocks;
        // an appended or inserted field must not silently disappear from import.
        let check_row = |row: u32, column: u32| -> Result<(), ImportError> {
            if row > self.profile.header_row
                && ((column == columns.field_id.number && !id_rows.contains(&row))
                    || ([columns.start.number, columns.length.number].contains(&column)
                        && !geometry_rows.contains(&row)))
            {
                return Err(ImportError::UnexpectedRow {
                    sheet: self.profile.sheet.clone(),
                    row,
                });
            }
            Ok(())
        };
        for (r, c, value) in values.used_cells() {
            if matches!(value, Data::String(text) if text.is_empty()) {
                continue;
            }
            let (r, c) = absolute(values.start().unwrap_or((0, 0)), r, c)?;
            check_row(r, c)?;
        }
        for (r, c, _) in formulas.used_cells() {
            let (r, c) = absolute(formulas.start().unwrap_or((0, 0)), r, c)?;
            check_row(r, c)?;
        }
        let mut raw = self.blueprint.clone();
        for (trace, block) in raw.traces.iter_mut().zip(&self.profile.traces) {
            for (field, binding) in trace.fields.iter_mut().zip(&block.fields) {
                let row = binding.row;
                sheet.expect(row, columns.field_id.number, Some(&binding.field_id))?;
                sheet.expect(
                    row,
                    columns.source_type.number,
                    binding.source_type.as_deref(),
                )?;
                sheet.expect(row, columns.obligation.number, Some(&binding.obligation))?;
                sheet.expect(
                    row,
                    columns.key_marker.number,
                    binding.key_marker.as_deref(),
                )?;
                field.name = sheet.required_text(row, columns.name.number)?.to_owned();
                let start = sheet.coordinate(
                    row,
                    columns.start.number,
                    self.profile.coordinate_formulas,
                )?;
                let length = sheet.coordinate(
                    row,
                    columns.length.number,
                    self.profile.coordinate_formulas,
                )?;
                match &mut field.physical {
                    RawPhysical::Numeric {
                        start: a,
                        length: b,
                        ..
                    }
                    | RawPhysical::Alphanumeric {
                        start: a,
                        length: b,
                        ..
                    }
                    | RawPhysical::Date {
                        start: a,
                        length: b,
                        ..
                    }
                    | RawPhysical::Decimal {
                        start: a,
                        length: b,
                        ..
                    } => {
                        *a = start;
                        *b = length;
                    }
                }
                let notes = sheet.text(row, columns.notes.number)?.unwrap_or("");
                let metadata = field.regulatory_metadata.get_or_insert_default();
                metadata.insert("source_note".into(), json!(notes));
                metadata.insert(
                    "xlsx_source".into(),
                    json!({
                        "sheet": self.profile.sheet, "row": row,
                        "source_type": binding.source_type, "obligation": binding.obligation,
                        "key_marker": binding.key_marker, "notes_original": notes,
                        "start_formula": sheet.formula(row, columns.start.number),
                        "length_formula": sheet.formula(row, columns.length.number),
                        "formula_cache_verified": false
                    }),
                );
            }
        }
        let mut annotations = Vec::new();
        for name in &self.profile.annotation_sheets {
            let text = workbook
                .worksheet_range(name)
                .map_err(ImportError::Workbook)?;
            let expressions = workbook
                .worksheet_formula(name)
                .map_err(ImportError::Workbook)?;
            if expressions.used_cells().next().is_some() {
                return Err(invalid("annotation formulas are not supported"));
            }
            for (r, c, cell) in text.used_cells() {
                if let Data::String(value) = cell {
                    let (row, column) = absolute(text.start().unwrap_or((0, 0)), r, c)?;
                    annotations.push(json!({"sheet":name,"row":row,"column":column,"text":value}));
                }
            }
        }
        raw.source.insert("xlsx_import".into(), json!({
            "source_sha256": crate::Sha256Digest(Sha256::digest(&bytes).into()),
            "source_size_bytes": bytes.len(),
            "blueprint_sha256": schema_fingerprint(&self.blueprint).map_err(ImportError::Canonical)?,
            "profile": self.profile,
            "annotations": annotations,
            "status": "requires_review",
            "formula_cache_verified": false
        }));
        Ok(raw)
    }
}
