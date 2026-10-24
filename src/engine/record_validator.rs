//! Explicit LocalOnly record validation, ratified by CNF-RECORD-01.
use super::{
    error_factory::ErrorFactory,
    expression_evaluator::{
        EvaluationContext, LocalEvaluationContext, UnknownReason,
        local_expression_supported_in_schema,
    },
    field_pipeline::{FieldPipeline, FieldPipelineError, FieldPipelineResult},
    identity_extractor::{IdentityExtractionError, IdentityExtractor},
    presence::{PresenceState, resolve_presence},
    rule_diagnostics::{RuleDiagnosticError, RuleDiagnosticInput},
    rule_validator::{RuleValidationResult, evaluate_local_rule},
};
use crate::{
    domain::{FieldId, RecordIdentity, SummaryBuildError, TraceId, ValidationSummaryBuilder},
    input::record::{RawRecord, RecordView},
    report::sink::{ReportError, ValidationSink},
    schema::compiled::{CompiledConstraint, CompiledFlowSchema, CompiledRecordSchema},
};
use std::{error::Error, fmt};

/// Only locally executed checks are represented. `valid` is not a statement
/// about Full validation, cross-record rules, datasets or persistent history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalRecordValidationArtifacts {
    pub identity: Option<RecordIdentity>,
    pub valid: bool,
}

/// Features required inside the local scope that have no executor yet.
/// Projection is deliberately absent: it belongs to Full, not to LocalOnly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedLocalFeature {
    Rules,
    DerivedValues,
    RepeatedGroups,
    ConditionalPresence,
    Constraint,
}

/// Technical failures only. Normative ValidationError values go to the sink,
/// never inside this enum. Application will compose this payload in P7.
#[derive(Debug)]
pub enum RecordValidationError {
    TraceNotFound(TraceId),
    UnsupportedLocalPlan {
        feature: UnsupportedLocalFeature,
        field_id: Option<FieldId>,
    },
    InvalidLineNumber,
    /// Local presence cannot be determined from available values. Abort the run
    /// rather than publish either a normative violation or a false valid report.
    UndeterminedPresence {
        field_id: FieldId,
        reason: UnknownReason,
    },
    FieldOutOfBounds {
        field_id: FieldId,
    },
    Identity(IdentityExtractionError),
    Pipeline(FieldPipelineError),
    Report(ReportError),
    Summary(SummaryBuildError),
    UndeterminedRule {
        rule_id: std::sync::Arc<str>,
        reason: UnknownReason,
    },
    RuleDiagnostic(RuleDiagnosticError),
    Reference(super::reference_execution::ReferenceExecutionError),
    PreviousFailure,
}
impl fmt::Display for RecordValidationError {
    /// Messages omit record bytes and patient identity; typed payloads retain
    /// schema context for the caller without inventing generic string errors.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Reference(_) => "reference record validation failed",
            Self::TraceNotFound(_) => "local validation trace does not exist",
            Self::UnsupportedLocalPlan { .. } => "required local plan is not supported",
            Self::UndeterminedPresence { .. } => {
                "local presence cannot be determined from available values"
            }
            Self::InvalidLineNumber => "record line number must be one-based",
            Self::FieldOutOfBounds { .. } => "record field range is outside source bytes",
            Self::Identity(_) => "record identity extraction failed",
            Self::Pipeline(_) => "record field pipeline failed",
            Self::Report(_) => "record diagnostic emission failed",
            Self::Summary(_) => "record counter update failed",
            Self::UndeterminedRule { .. } => {
                "local rule cannot be determined from available values"
            }
            Self::RuleDiagnostic(_) => "local rule diagnostic construction failed",
            Self::PreviousFailure => "record validator cannot resume after a technical failure",
        })
    }
}
impl Error for RecordValidationError {
    /// Original technical causes survive wrapping and can be mapped without
    /// parsing human text. Context-only variants have no nested source.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Reference(error) => Some(error),
            Self::RuleDiagnostic(error) => Some(error),
            Self::Identity(error) => Some(error),
            Self::Pipeline(error) => Some(error),
            Self::Report(error) => Some(error),
            Self::Summary(error) => Some(error),
            _ => None,
        }
    }
}

/// Borrows one compiled trace and its flow's diagnostic metadata. No reader,
/// dataset, projection, application type or owned record is stored here.
/// A technical failure is terminal for this instance; normative Invalid is not.
pub struct RecordValidator<'schema> {
    schema: &'schema CompiledRecordSchema,
    factory: ErrorFactory<'schema>,
    failed: bool,
    skipped_cross_rules: usize,
}
impl<'schema> RecordValidator<'schema> {
    /// Resolve a trace through its actual compiled flow, preventing inconsistent
    /// flow/trace/encoding metadata. Reject unsupported local plans once, before
    /// any record or summary mutation. Nonempty projection plans are permitted.
    pub fn new(
        flow: &'schema CompiledFlowSchema,
        trace_id: &TraceId,
    ) -> Result<Self, RecordValidationError> {
        Self::build(flow, trace_id, false)
    }
    fn build(
        flow: &'schema CompiledFlowSchema,
        trace_id: &TraceId,
        references: bool,
    ) -> Result<Self, RecordValidationError> {
        if !references && !flow.required_reference_data().is_empty() {
            return Err(RecordValidationError::UnsupportedLocalPlan {
                feature: UnsupportedLocalFeature::Constraint,
                field_id: None,
            });
        }
        let schema = flow
            .trace(trace_id)
            .ok_or_else(|| RecordValidationError::TraceNotFound(trace_id.clone()))?;
        check_local_plan(schema, references)?;
        Ok(Self {
            schema,
            skipped_cross_rules: flow
                .cross_record_plan()
                .rules
                .iter()
                .filter(|rule| {
                    flow.traces()
                        .get(rule.source_trace)
                        .is_some_and(|source| source.id() == trace_id)
                })
                .count(),
            factory: ErrorFactory {
                flow_id: &flow.key().flow_id,
                schema_version: &flow.key().version,
                trace_id: schema.id(),
                encoding: flow.encoding(),
            },
            failed: false,
        })
    }

    /// StrictSkipRecord: malformed width emits only E_RECORD_LENGTH and returns
    /// no identity. On a full-width record all fields are checked in schema
    /// order; Invalid never suppresses independent later fields. The caller
    /// must abort its report on Err and must not publish a definitive summary.
    /// This API intentionally cannot request or claim Full validation.
    pub fn validate_local(
        &mut self,
        record: RawRecord<'_>,
        sink: &mut dyn ValidationSink,
        summary: &mut ValidationSummaryBuilder,
    ) -> Result<LocalRecordValidationArtifacts, RecordValidationError> {
        if self.failed {
            return Err(RecordValidationError::PreviousFailure);
        }
        let result = self.validate_record(record, sink, summary, None, RecordPhase::LocalOnly);
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    pub(crate) fn validate_full_pass_one(
        &mut self,
        record: RawRecord<'_>,
        sink: &mut dyn ValidationSink,
        summary: &mut ValidationSummaryBuilder,
    ) -> Result<LocalRecordValidationArtifacts, RecordValidationError> {
        if self.failed {
            return Err(RecordValidationError::PreviousFailure);
        }
        let result = self.validate_record(record, sink, summary, None, RecordPhase::FullPassOne);
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    /// Keep the technical terminal-state transition in the public wrapper so
    /// every early `?` below has the same failure behavior.
    fn validate_record(
        &self,
        record: RawRecord<'_>,
        sink: &mut dyn ValidationSink,
        summary: &mut ValidationSummaryBuilder,
        mut references: Option<&mut super::reference_execution::ReferenceExecution<'_>>,
        phase: RecordPhase,
    ) -> Result<LocalRecordValidationArtifacts, RecordValidationError> {
        if record.line_number == 0 {
            return Err(RecordValidationError::InvalidLineNumber);
        }
        summary
            .record_processed()
            .map_err(RecordValidationError::Summary)?;
        // Only LocalOnly omits cross rules permanently. Full pass one defers
        // them; the flow collector accounts actual skips after pass two.
        for _ in 0..if phase == RecordPhase::LocalOnly {
            self.skipped_cross_rules
        } else {
            0
        } {
            summary
                .rule_skipped()
                .map_err(RecordValidationError::Summary)?;
        }
        if record.bytes.len() != self.schema.record_length() {
            let error = self.factory.record_length_error(
                record.line_number,
                None,
                self.schema.record_length(),
                record.bytes.len(),
            );
            // Count only a successfully emitted error. If either dependency
            // fails the caller receives Err, never a fabricated valid result.
            sink.on_error(&error)
                .map_err(RecordValidationError::Report)?;
            summary
                .error_emitted(&error.code)
                .map_err(RecordValidationError::Summary)?;
            summary
                .record_invalid()
                .map_err(RecordValidationError::Summary)?;
            return Ok(LocalRecordValidationArtifacts {
                identity: None,
                valid: false,
            });
        }
        let view = RecordView::new(record.bytes);
        let identity = IdentityExtractor
            .extract(&view, self.schema, self.factory.encoding)
            .map_err(RecordValidationError::Identity)?;
        let factory = ErrorFactory {
            flow_id: self.factory.flow_id,
            schema_version: self.factory.schema_version,
            trace_id: self.factory.trace_id,
            encoding: self.factory.encoding,
        };
        let local = LocalEvaluationContext {
            record: &view,
            schema: self.schema,
        };
        let ctx = EvaluationContext { fields: &local };
        let mut pipeline = FieldPipeline::new(factory, sink, summary);
        let mut valid = true;
        for field in self.schema.fields() {
            let value =
                view.field(field)
                    .ok_or_else(|| RecordValidationError::FieldOutOfBounds {
                        field_id: field.id().clone(),
                    })?;
            let state = resolve_presence(field.presence_policy(), &ctx);
            let result = if let Some(execution) = references.as_deref_mut() {
                pipeline.run_with_reference_presence(
                    field,
                    value,
                    record.line_number,
                    &identity,
                    state,
                    execution,
                )
            } else {
                pipeline.run_with_presence(field, value, record.line_number, &identity, state)
            }
            .map_err(RecordValidationError::Pipeline)?;
            if let PresenceState::Unknown(reason) = state {
                return Err(RecordValidationError::UndeterminedPresence {
                    field_id: field.id().clone(),
                    reason,
                });
            }
            match result {
                FieldPipelineResult::Valid | FieldPipelineResult::MissingOptional => {}
                FieldPipelineResult::Invalid => valid = false,
                FieldPipelineResult::SkippedDueToUnknownPresence => {
                    // Defensive even after check_local_plan: Unknown cannot be
                    // promoted to a local pass or a normative violation.
                    return Err(RecordValidationError::UnsupportedLocalPlan {
                        feature: UnsupportedLocalFeature::ConditionalPresence,
                        field_id: Some(field.id().clone()),
                    });
                }
            }
        }
        for rule in self.schema.local_rules() {
            let artifacts = if references.is_some() {
                super::rule_validator::evaluate_rule_with_references(
                    rule,
                    &ctx,
                    references.as_deref_mut(),
                )
                .map_err(RecordValidationError::Reference)?
            } else {
                evaluate_local_rule(rule, &ctx)
            };
            match artifacts.result {
                RuleValidationResult::Satisfied => {}
                RuleValidationResult::Unknown(
                    UnknownReason::ReferenceProvisional | UnknownReason::ReferenceInconclusive,
                ) if references.is_some() => {}
                RuleValidationResult::Unknown(reason) => {
                    summary
                        .rule_skipped()
                        .map_err(RecordValidationError::Summary)?;
                    return Err(RecordValidationError::UndeterminedRule {
                        rule_id: rule.id.clone(),
                        reason,
                    });
                }
                RuleValidationResult::Violated(violation) => {
                    let assertion_index =
                        artifacts
                            .failed_assertion
                            .ok_or(RecordValidationError::RuleDiagnostic(
                                RuleDiagnosticError::InvalidPlan,
                            ))?;
                    let error = self
                        .factory
                        .local_rule_error(
                            record.line_number,
                            &identity,
                            RuleDiagnosticInput {
                                rule,
                                assertion_index,
                                record: &view,
                                schema: self.schema,
                            },
                            violation,
                        )
                        .map_err(RecordValidationError::RuleDiagnostic)?;
                    sink.on_error(&error)
                        .map_err(RecordValidationError::Report)?;
                    summary
                        .error_emitted(&error.code)
                        .map_err(RecordValidationError::Summary)?;
                    valid = false;
                }
            }
        }
        if references.as_ref().is_some_and(|execution| {
            execution.conclusion != super::reference_execution::ReferenceConclusion::Definitive
        }) {
            // Legacy finish must fail rather than omit non-definitive evidence;
            // finish_with_reference_data requires matching aggregated counts.
            summary.mark_unreported_reference_evidence();
        }
        if valid {
            summary.record_valid()
        } else {
            summary.record_invalid()
        }
        .map_err(RecordValidationError::Summary)?;
        Ok(LocalRecordValidationArtifacts {
            identity: Some(identity),
            valid,
        })
    }
}

/// Public compilation already rejects these features. Defend the runtime
/// boundary as it evolves: a future compiled local obligation cannot disappear
/// merely because the current executor does not know how to evaluate it.
fn check_local_plan(
    schema: &CompiledRecordSchema,
    references: bool,
) -> Result<(), RecordValidationError> {
    for (present, feature) in [
        (
            schema.local_rules().iter().any(|rule| {
                !super::rule_validator::local_rule_supported_with_references(
                    rule,
                    Some(schema),
                    references,
                )
            }),
            UnsupportedLocalFeature::Rules,
        ),
        (
            schema
                .derived_values()
                .iter()
                .any(|plan| super::derived_evaluator::local_derived_type(plan, schema).is_none()),
            UnsupportedLocalFeature::DerivedValues,
        ),
        (
            !schema.repeated_groups().is_empty(),
            UnsupportedLocalFeature::RepeatedGroups,
        ),
    ] {
        if present {
            return Err(RecordValidationError::UnsupportedLocalPlan {
                feature,
                field_id: None,
            });
        }
    }
    for field in schema.fields() {
        let policy = field.presence_policy();
        let feature = if policy
            .required_if
            .iter()
            .chain(&policy.forbidden_if)
            .any(|expr| !local_expression_supported_in_schema(expr, schema))
        {
            Some(UnsupportedLocalFeature::ConditionalPresence)
        } else if field.constraints().iter().any(|constraint| {
            if references && matches!(constraint, CompiledConstraint::ReferenceData { metadata_bindings, dataset_indexes, .. } if metadata_bindings.is_empty() && !dataset_indexes.is_empty()) { return false; }
            !matches!(
                constraint,
                CompiledConstraint::AllowedValues { .. }
                    | CompiledConstraint::Range { .. }
                    | CompiledConstraint::Date { .. }
                    | CompiledConstraint::Regex { .. }
            )
        }) {
            Some(UnsupportedLocalFeature::Constraint)
        } else {
            None
        };
        if let Some(feature) = feature {
            return Err(RecordValidationError::UnsupportedLocalPlan {
                feature,
                field_id: Some(field.id().clone()),
            });
        }
    }
    Ok(())
}

/// Explicit reference-aware record artifacts; absence of normative violations
/// does not make Provisional/Inconclusive evaluations definitive.
#[derive(Debug)]
pub struct ReferenceRecordValidationArtifacts {
    pub identity: Option<RecordIdentity>,
    pub normative_violations_detected: bool,
    pub reference_conclusion: super::reference_execution::ReferenceConclusion,
}

pub struct ReferenceRecordValidator<'schema, 'references> {
    inner: RecordValidator<'schema>,
    dependencies: &'schema [crate::ReferenceDependency],
    prepared: &'references crate::PreparedReferenceEvidence,
}
impl<'schema, 'references> ReferenceRecordValidator<'schema, 'references> {
    pub fn new(
        flow: &'schema CompiledFlowSchema,
        trace_id: &TraceId,
        prepared: &'references crate::PreparedReferenceEvidence,
    ) -> Result<Self, RecordValidationError> {
        // A snapshot with a different expected digest must never validate this
        // schema's version merely because its DatasetKey has the same spelling.
        for dependency in flow.required_reference_data() {
            if !prepared.evidence(&dependency.key).is_some_and(|evidence| {
                evidence.expected_sha256() == &crate::Sha256Digest(dependency.sha256)
            }) {
                return Err(RecordValidationError::Reference(
                    super::reference_execution::ReferenceExecutionError::InconsistentPreparation,
                ));
            }
        }
        Ok(Self {
            inner: RecordValidator::build(flow, trace_id, true)?,
            dependencies: flow.required_reference_data(),
            prepared,
        })
    }
    pub fn validate_local(
        &mut self,
        record: RawRecord<'_>,
        sink: &mut dyn ValidationSink,
        summary: &mut ValidationSummaryBuilder,
        observer: &mut dyn super::reference_execution::ReferenceEvaluationObserver,
    ) -> Result<ReferenceRecordValidationArtifacts, RecordValidationError> {
        self.validate_phase(record, sink, summary, observer, RecordPhase::LocalOnly)
    }
    pub(crate) fn validate_phase(
        &mut self,
        record: RawRecord<'_>,
        sink: &mut dyn ValidationSink,
        summary: &mut ValidationSummaryBuilder,
        observer: &mut dyn super::reference_execution::ReferenceEvaluationObserver,
        phase: RecordPhase,
    ) -> Result<ReferenceRecordValidationArtifacts, RecordValidationError> {
        if self.inner.failed {
            return Err(RecordValidationError::PreviousFailure);
        }
        let mut execution = super::reference_execution::ReferenceExecution {
            prepared: self.prepared,
            dependencies: self.dependencies,
            observer,
            conclusion: super::reference_execution::ReferenceConclusion::Definitive,
        };
        let result = self
            .inner
            .validate_record(record, sink, summary, Some(&mut execution), phase);
        match result {
            Ok(local) => Ok(ReferenceRecordValidationArtifacts {
                identity: local.identity,
                normative_violations_detected: !local.valid,
                reference_conclusion: execution.conclusion,
            }),
            Err(error) => {
                self.inner.failed = true;
                Err(error)
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecordPhase {
    LocalOnly,
    FullPassOne,
}
