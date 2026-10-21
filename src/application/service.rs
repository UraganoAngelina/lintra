//! Request preflight only. ValidationService execution is delivered in T-P7-04.
use super::request::*;
use crate::{
    CompiledFlowSchema, CompiledRecordSchema, InputSource, SchemaKey, TraceId,
    engine::preflight::{InputPlanError, PreparedTrace, prepare_trace},
};
use std::{collections::HashSet, error::Error, fmt, path::Path};

/// Application composes the engine's typed errors, never the reverse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidRequestError {
    SchemaKeyMismatch {
        requested: SchemaKey,
        loaded: SchemaKey,
    },
    UnsupportedMode(ValidationMode),
    UnsupportedFlowScope,
    UnsupportedMaxErrors {
        limit: u64,
    },
    EmptyFlow,
    DuplicateTrace(TraceId),
    Input(InputPlanError),
}
impl fmt::Display for InvalidRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SchemaKeyMismatch { .. } => "request and compiled schema keys differ",
            Self::UnsupportedMode(_) => "Full validation is not available until P13",
            Self::UnsupportedFlowScope => "Flow scope is not available until P13",
            Self::UnsupportedMaxErrors { .. } => "max_errors must be None in the current engine",
            Self::EmptyFlow => "flow input must contain at least one trace",
            Self::DuplicateTrace(_) => "trace input is duplicated",
            Self::Input(_) => "input preflight failed",
        })
    }
}
impl Error for InvalidRequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Input(error) => Some(error),
            _ => None,
        }
    }
}

/// Only a supported Trace/LocalOnly request can create this plan. Borrowed
/// checked inputs and copied options cannot change while the plan is in use.
#[derive(Debug)]
pub struct PreflightPlan<'schema, 'input> {
    input: PreparedTrace<'schema, 'input>,
    options: ValidationOptions,
}
impl<'schema, 'input> PreflightPlan<'schema, 'input> {
    pub fn trace(&self) -> &'schema CompiledRecordSchema {
        self.input.trace()
    }
    pub fn path(&self) -> &'input Path {
        self.input.path()
    }
    pub fn options(&self) -> ValidationOptions {
        self.options
    }
}

/// DET §86 orchestration stays in application (DET §5); engine prepares neutral
/// input references. Error precedence: key, mode, options, scope/input checks.
/// No file read, counter, dataset load or normative check occurs here.
pub fn preflight<'schema, 'input>(
    request: &'input ValidationRequest,
    schema: &'schema CompiledFlowSchema,
) -> Result<PreflightPlan<'schema, 'input>, InvalidRequestError> {
    if &request.schema_key != schema.key() {
        return Err(InvalidRequestError::SchemaKeyMismatch {
            requested: request.schema_key.clone(),
            loaded: schema.key().clone(),
        });
    }
    if request.mode == ValidationMode::Full {
        return Err(InvalidRequestError::UnsupportedMode(request.mode));
    }
    if let Some(limit) = request.options.max_errors {
        return Err(InvalidRequestError::UnsupportedMaxErrors { limit });
    }
    match &request.scope {
        ValidationScope::Trace(input) => {
            let InputSource::Path(path) = &input.source;
            Ok(PreflightPlan {
                input: prepare_trace(schema, &input.trace_id, path)
                    .map_err(InvalidRequestError::Input)?,
                options: request.options,
            })
        }
        ValidationScope::Flow(flow) => {
            if flow.traces.is_empty() {
                return Err(InvalidRequestError::EmptyFlow);
            }
            let mut seen = HashSet::new();
            for input in &flow.traces {
                if !seen.insert(&input.trace_id) {
                    return Err(InvalidRequestError::DuplicateTrace(input.trace_id.clone()));
                }
                let InputSource::Path(path) = &input.source;
                prepare_trace(schema, &input.trace_id, path).map_err(InvalidRequestError::Input)?;
            }
            // This is a capability boundary, not a normative incompatibility.
            // P7 is explicitly Trace/LocalOnly; never silently pick one input.
            Err(InvalidRequestError::UnsupportedFlowScope)
        }
    }
}
