//! Streaming RFC 4180 output. Publication belongs to the atomic report layer.
use super::{
    columns::ERROR_COLUMNS,
    sink::{ReportError, ValidationSink},
};
use crate::{ValidationError, ValidationSummary};
use std::io::{self, Write};

pub struct CsvSink<W> {
    writer: W,
    failed: bool,
}

impl<W: Write> CsvSink<W> {
    /// Write the header immediately, including for runs without violations.
    /// A header failure returns Err, never a usable partially initialized sink.
    pub fn new(mut writer: W) -> Result<Self, ReportError> {
        for (index, column) in ERROR_COLUMNS.iter().enumerate() {
            if index != 0 {
                writer.write_all(b",").map_err(ReportError::Write)?;
            }
            write_cell(&mut writer, column.header()).map_err(ReportError::Write)?;
        }
        writer.write_all(b"\r\n").map_err(ReportError::Write)?;
        Ok(Self {
            writer,
            failed: false,
        })
    }

    pub fn get_ref(&self) -> &W {
        &self.writer
    }

    pub fn finish(mut self) -> Result<W, ReportError> {
        self.flush()?;
        Ok(self.writer)
    }

    fn flush(&mut self) -> Result<(), ReportError> {
        if self.failed {
            return Err(ReportError::PreviousFailure);
        }
        self.writer.flush().map_err(|source| {
            self.failed = true;
            ReportError::Write(source)
        })
    }

    fn row(&mut self, error: &ValidationError) -> Result<(), ReportError> {
        error.validate().map_err(ReportError::InvalidDiagnostic)?;
        for (index, column) in ERROR_COLUMNS.iter().enumerate() {
            let value = column
                .value(error)
                .map_err(ReportError::InvalidDiagnostic)?;
            if index != 0 {
                self.writer.write_all(b",").map_err(ReportError::Write)?;
            }
            write_cell(&mut self.writer, &value).map_err(ReportError::Write)?;
        }
        self.writer.write_all(b"\r\n").map_err(ReportError::Write)
    }
}

impl<W: Write> ValidationSink for CsvSink<W> {
    fn on_error(&mut self, error: &ValidationError) -> Result<(), ReportError> {
        if self.failed {
            return Err(ReportError::PreviousFailure);
        }
        let result = self.row(error);
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        // CSV contains errors only; summary is a separate artifact.
        self.flush()
    }
}

/// Quote cells containing CSV delimiters, quotes or either newline byte. Double
/// embedded quotes without copying/normalizing the original UTF-8 payload.
fn write_cell(writer: &mut impl Write, value: &str) -> io::Result<()> {
    let bytes = value.as_bytes();
    if !bytes
        .iter()
        .any(|b| matches!(b, b',' | b'"' | b'\r' | b'\n'))
    {
        return writer.write_all(bytes);
    }
    writer.write_all(b"\"")?;
    let mut start = 0;
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'"' {
            writer.write_all(&bytes[start..index])?;
            writer.write_all(b"\"\"")?;
            start = index + 1;
        }
    }
    writer.write_all(&bytes[start..])?;
    writer.write_all(b"\"")
}
