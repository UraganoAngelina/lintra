//! Schema/input checks independent of application requests or execution modes.
use crate::{CompiledFlowSchema, CompiledRecordSchema, TraceId};
use std::{error::Error, fmt, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputPlanError {
    TraceNotFound(TraceId),
    EmptyPath { trace_id: TraceId },
}
impl fmt::Display for InputPlanError {
    /// Technical text contains no file path or record data.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TraceNotFound(_) => "requested trace does not exist",
            Self::EmptyPath { .. } => "input path must be nonempty",
        })
    }
}
impl Error for InputPlanError {}

/// Checked references; constructors remain private to the preparing function.
#[derive(Debug)]
pub struct PreparedTrace<'schema, 'input> {
    trace: &'schema CompiledRecordSchema,
    path: &'input Path,
}
impl<'schema, 'input> PreparedTrace<'schema, 'input> {
    pub fn trace(&self) -> &'schema CompiledRecordSchema {
        self.trace
    }
    pub fn path(&self) -> &'input Path {
        self.path
    }
}

/// Borrow exactly the declared path and trace; do not canonicalize, trim,
/// resolve against a new directory or open the file. I/O errors belong to input.
pub fn prepare_trace<'schema, 'input>(
    schema: &'schema CompiledFlowSchema,
    trace_id: &TraceId,
    path: &'input Path,
) -> Result<PreparedTrace<'schema, 'input>, InputPlanError> {
    let trace = schema
        .trace(trace_id)
        .ok_or_else(|| InputPlanError::TraceNotFound(trace_id.clone()))?;
    if path.as_os_str().is_empty() {
        return Err(InputPlanError::EmptyPath {
            trace_id: trace_id.clone(),
        });
    }
    Ok(PreparedTrace { trace, path })
}
