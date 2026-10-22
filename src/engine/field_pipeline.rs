//! Ordered local field stages. No normative flags are mutated on other fields.
use super::error_factory::ErrorFactory;
use crate::{
    domain::{FieldId, RecordIdentity, SummaryBuildError, ValidationSummaryBuilder},
    input::record::FieldSlice,
    report::sink::{ReportError, ValidationSink},
    schema::compiled::{CompiledConstraint, CompiledFieldSpec, ValueTarget},
    validators::{
        FieldViolation,
        alignment::{LogicalValueError, logical_value},
        datatype::{DatatypeError, DatatypeValidationResult, validate_datatype},
        date::validate_date,
        domain::validate_allowed_values,
        presence::{PresenceValidationResult, validate_presence},
        range::validate_numeric_range,
        regex::validate_regex,
    },
};
use std::{error::Error, fmt};

/// Controlled field outcome (DET §41), never a technical failure or an input
/// to mutable required/forbidden flags. Unknown presence is not a valid field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldPipelineResult {
    Valid,
    MissingOptional,
    Invalid,
    SkippedDueToUnknownPresence,
}

/// Every payload is technical: normative violations are streamed then mapped
/// to Invalid. Unsupported future plans fail closed, not as domain errors.
#[derive(Debug)]
pub enum FieldPipelineError {
    Logical(LogicalValueError),
    Datatype(DatatypeError),
    UnsupportedConstraint {
        field_id: FieldId,
        constraint_index: usize,
    },
    Report(ReportError),
    Summary(SummaryBuildError),
    PreviousFailure,
}
impl fmt::Display for FieldPipelineError {
    /// Fixed text avoids putting record values/paths into technical messages.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Logical(_) => "field logical extraction failed",
            Self::Datatype(_) => "field datatype plan failed",
            Self::UnsupportedConstraint { .. } => "field constraint plan is not supported",
            Self::Report(_) => "field diagnostic emission failed",
            Self::Summary(_) => "field error counter update failed",
            Self::PreviousFailure => "field pipeline cannot resume after a technical failure",
        })
    }
}
impl Error for FieldPipelineError {
    /// Preserve original payloads for application mapping; unsupported plan
    /// location and terminal state have no nested exception to expose.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Logical(source) => Some(source),
            Self::Datatype(source) => Some(source),
            Self::Report(source) => Some(source),
            Self::Summary(source) => Some(source),
            Self::UnsupportedConstraint { .. } | Self::PreviousFailure => None,
        }
    }
}

/// Borrow run-level reporting dependencies, never record bytes. The failed bit
/// describes only terminal technical state, not a normative presence flag.
/// Record counts belong to RecordValidator, not to each field invocation.
pub struct FieldPipeline<'schema, 'output> {
    factory: ErrorFactory<'schema>,
    sink: &'output mut dyn ValidationSink,
    summary: &'output mut ValidationSummaryBuilder,
    failed: bool,
}
impl<'schema, 'output> FieldPipeline<'schema, 'output> {
    /// Own the lightweight factory and borrow sink/counters for ordered output.
    pub fn new(
        factory: ErrorFactory<'schema>,
        sink: &'output mut dyn ValidationSink,
        summary: &'output mut ValidationSummaryBuilder,
    ) -> Self {
        Self {
            factory,
            sink,
            summary,
            failed: false,
        }
    }

    /// Run one already sliced, full-width field. Input metadata (line/identity)
    /// is supplied by the record stage. Stop on the first primary violation;
    /// a normative Invalid does not stop other fields, a technical Err does.
    pub fn run(
        &mut self,
        field: &CompiledFieldSpec,
        raw: FieldSlice<'_>,
        line_number: u64,
        identity: &RecordIdentity,
    ) -> Result<FieldPipelineResult, FieldPipelineError> {
        if self.failed {
            return Err(FieldPipelineError::PreviousFailure);
        }
        let presence = validate_presence(field, raw);
        let result = self.after_presence(presence, field, raw, line_number, identity);
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    /// All presence violations use the same stop path, including future
    /// precomputed conditional/forbidden violations. This does not evaluate P10
    /// expressions: current runtime presence remains the base-only validator.
    fn after_presence(
        &mut self,
        presence: Result<PresenceValidationResult, FieldViolation>,
        field: &CompiledFieldSpec,
        raw: FieldSlice<'_>,
        line_number: u64,
        identity: &RecordIdentity,
    ) -> Result<FieldPipelineResult, FieldPipelineError> {
        match presence {
            Err(violation) => return self.emit(field, raw, line_number, identity, violation),
            Ok(PresenceValidationResult::MissingOptional) => {
                return Ok(FieldPipelineResult::MissingOptional);
            }
            Ok(PresenceValidationResult::Unknown) => {
                return Ok(FieldPipelineResult::SkippedDueToUnknownPresence);
            }
            Ok(PresenceValidationResult::Present) => {}
        }
        let logical =
            logical_value(&field.alignment(), raw).map_err(FieldPipelineError::Logical)?;
        match validate_datatype(field, logical, self.factory.encoding)
            .map_err(FieldPipelineError::Datatype)?
        {
            DatatypeValidationResult::Invalid(violation) => {
                return self.emit(field, raw, line_number, identity, violation);
            }
            DatatypeValidationResult::Valid => {}
        }
        for (constraint_index, constraint) in field.constraints().iter().enumerate() {
            match constraint {
                CompiledConstraint::AllowedValues { values, target } => {
                    // Target choice is explicit; diagnostics still receive the
                    // original physical bytes, even on a logical-domain error.
                    let value = match target {
                        ValueTarget::Raw => raw,
                        ValueTarget::Logical => logical,
                    };
                    if let Err(violation) = validate_allowed_values(values, value) {
                        return self.emit(field, raw, line_number, identity, violation);
                    }
                }
                CompiledConstraint::Regex {
                    regex,
                    mode,
                    target,
                } => {
                    let value = match target {
                        ValueTarget::Raw => raw,
                        ValueTarget::Logical => logical,
                    };
                    if let Err(violation) = validate_regex(regex, *mode, value) {
                        return self.emit(field, raw, line_number, identity, violation);
                    }
                }
                CompiledConstraint::Date { format } => {
                    if let Err(violation) = validate_date(*format, logical) {
                        return self.emit(field, raw, line_number, identity, violation);
                    }
                }
                CompiledConstraint::Range { min, max } => {
                    if let Err(violation) = validate_numeric_range(min, max, logical) {
                        return self.emit(field, raw, line_number, identity, violation);
                    }
                }
                _ => {
                    return Err(FieldPipelineError::UnsupportedConstraint {
                        field_id: field.id().clone(),
                        constraint_index,
                    });
                }
            }
        }
        Ok(FieldPipelineResult::Valid)
    }

    /// Stream first, then count only successfully emitted diagnostics. Either
    /// failure is terminal, with its original typed cause. No global error Vec.
    fn emit(
        &mut self,
        field: &CompiledFieldSpec,
        raw: FieldSlice<'_>,
        line_number: u64,
        identity: &RecordIdentity,
        violation: FieldViolation,
    ) -> Result<FieldPipelineResult, FieldPipelineError> {
        let error =
            self.factory
                .field_error(line_number, identity, field, raw.as_bytes(), violation);
        self.sink
            .on_error(&error)
            .map_err(FieldPipelineError::Report)?;
        self.summary
            .error_emitted(&error.code)
            .map_err(FieldPipelineError::Summary)?;
        Ok(FieldPipelineResult::Invalid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RawFlowSchema, RawResolvedEncoding, SchemaCompiler, ScopeSummary, ValidationCode, VecSink,
    };

    /// Feed precomputed presence failures to the common stop path. This tests
    /// forbidden/conditional propagation without evaluating any P10 expression
    /// or enabling unsupported normative ASTs in the production compiler.
    #[test]
    fn every_presence_violation_uses_the_same_stop_path() {
        let mut raw: RawFlowSchema =
            serde_json::from_str(include_str!("../../tests/golden/synthetic/v1/schema.json"))
                .unwrap();
        raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
        let schema = SchemaCompiler.compile(raw).unwrap();
        let trace = &schema.traces()[0];
        let field = &trace.fields()[0];
        let mut sink = VecSink::default();
        let mut summary = ValidationSummaryBuilder::new(
            schema.key().clone(),
            ScopeSummary::Trace {
                trace_id: trace.id().clone(),
            },
        );
        let factory = ErrorFactory {
            flow_id: &schema.key().flow_id,
            schema_version: &schema.key().version,
            trace_id: trace.id(),
            encoding: schema.encoding(),
        };
        let mut pipeline = FieldPipeline::new(factory, &mut sink, &mut summary);
        for code in [
            ValidationCode::RequiredField,
            ValidationCode::ConditionalRequired,
            ValidationCode::ForbiddenField,
        ] {
            let failure = FieldViolation {
                code,
                expected: None,
                reason: String::new(),
                diagnostic_context: vec![],
            };
            assert_eq!(
                pipeline
                    .after_presence(
                        Err(failure),
                        field,
                        FieldSlice { raw: b"ABCDEF" },
                        1,
                        &RecordIdentity::default()
                    )
                    .unwrap(),
                FieldPipelineResult::Invalid
            );
        }
        assert_eq!(
            sink.errors()
                .iter()
                .map(|error| &error.code)
                .collect::<Vec<_>>(),
            [
                &ValidationCode::RequiredField,
                &ValidationCode::ConditionalRequired,
                &ValidationCode::ForbiddenField
            ]
        );
    }
}
