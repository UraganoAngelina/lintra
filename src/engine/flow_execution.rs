//! Two-pass, single-run execution. Historical storage and application request
//! handling remain outside this module. All input I/O goes through TraceValidator.
use super::{
    flow_validator::{CrossPassControl, run_cross_record_rules},
    trace_validator::{FullRecordCollector, TracePassOnePorts},
};
use crate::*;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    error::Error,
    fmt,
    path::Path,
};

type TechnicalError = Box<dyn Error + Send + Sync>;

/// Borrowed pass-one evidence for an explicit application producer. No storage,
/// cryptography or regulatory decision is owned by this neutral engine hook.
pub(crate) trait FlowRecordObserver {
    fn on_record(
        &mut self,
        trace: &CompiledRecordSchema,
        record: &RawRecord<'_>,
        key: &JoinKey,
    ) -> Result<(), TechnicalError>;
}

pub struct FlowTraceInput<'a> {
    pub trace_id: &'a TraceId,
    pub path: &'a Path,
}
#[derive(Debug)]
pub enum FlowValidationError {
    UnknownTrace(TraceId),
    DuplicateTrace(TraceId),
    MissingTrace(TraceId),
    EmptyPath(TraceId),
    ReferencesNotConfigured,
    ReferenceObserver(TechnicalError),
    Trace(TraceValidationError),
    Cross(CrossRecordValidationError),
    Summary(SummaryBuildError),
    Inputs(RelatedContextError),
    InvalidPlan,
}
impl fmt::Display for FlowValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnknownTrace(_) => "bundle contains an unknown trace",
            Self::DuplicateTrace(_) => "bundle contains a duplicate trace",
            Self::MissingTrace(_) => "Full bundle lacks a required trace",
            Self::EmptyPath(_) => "bundle contains an empty path",
            Self::ReferencesNotConfigured => {
                "Full execution requires prepared references and trace observers"
            }
            Self::ReferenceObserver(_) => "Full reference observer failed",
            Self::Trace(_) => "Full trace execution failed",
            Self::Cross(_) => "Full cross-record execution failed",
            Self::Summary(_) => "Full summary update failed",
            Self::Inputs(_) => "Full runtime input binding failed",
            Self::InvalidPlan => "Full plan is inconsistent",
        })
    }
}
impl Error for FlowValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReferenceObserver(e) => Some(e.as_ref()),
            Self::Trace(e) => Some(e),
            Self::Cross(e) => Some(e),
            Self::Summary(e) => Some(e),
            Self::Inputs(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowEvent {
    Trace(TraceEvent),
    CrossStarted,
    CrossRecordsProcessed { count: u64 },
    Completed,
    Cancelled,
}
pub struct FlowControl<'a> {
    pub is_cancelled: &'a dyn Fn() -> bool,
    pub on_event: &'a mut dyn FnMut(FlowEvent),
}
/// Actual per-trace aggregation ports: provenance is never assigned using a
/// single trace collector for the entire bundle.
pub trait FlowReferenceObservers {
    fn for_trace(
        &mut self,
        trace: &TraceId,
    ) -> Result<&mut dyn ReferenceEvaluationObserver, TechnicalError>;
}
pub struct FlowExecutionPorts<'a, 'control> {
    pub sink: &'a mut dyn ValidationSink,
    pub summary: &'a mut ValidationSummaryBuilder,
    pub audits: &'a InputAuditSession,
    pub control: &'a mut FlowControl<'control>,
    pub reference_observers: Option<&'a mut dyn FlowReferenceObservers>,
    pub cross_observer: Option<&'a mut dyn CrossRecordEvaluationObserver>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowCompletion {
    Completed,
    Cancelled,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NonDefinitiveCrossResult {
    Unknown(UnknownReason),
    NotApplicable,
}
#[derive(Debug, Clone)]
pub struct CrossEvidenceCause {
    pub result: NonDefinitiveCrossResult,
    pub join_id: Option<JoinId>,
    pub diagnostic: Option<RelatedDiagnostic>,
    pub evaluations: u64,
}
#[derive(Debug, Clone)]
pub struct CrossControlEvidence {
    pub trace_id: TraceId,
    pub rule_id: String,
    pub counts: CrossRecordPassSummary,
    /// Physical source records whose width prevents projection, not rule IDs.
    pub skipped_evaluations: u64,
    pub non_definitive: Vec<CrossEvidenceCause>,
}
#[derive(Debug, Clone)]
pub struct RecordKeyEvidence {
    pub trace_id: TraceId,
    pub checked: u64,
    pub duplicates: u64,
    pub inconclusive: u64,
}
#[derive(Debug)]
pub struct FlowValidationArtifacts {
    pub completion: FlowCompletion,
    pub counters: ValidationCounters,
    pub traces: Vec<TraceCoverageEvidence>,
    pub record_keys: Vec<RecordKeyEvidence>,
    pub cross: Vec<CrossControlEvidence>,
    pub cross_totals: CrossRecordPassSummary,
}

pub struct FlowValidator<'a> {
    schema: &'a CompiledFlowSchema,
    references: Option<&'a PreparedReferenceEvidence>,
    newline: NewlinePolicy,
}
impl<'a> FlowValidator<'a> {
    pub fn new(
        schema: &'a CompiledFlowSchema,
        newline: NewlinePolicy,
    ) -> Result<Self, FlowValidationError> {
        if !schema.required_reference_data().is_empty() {
            return Err(FlowValidationError::ReferencesNotConfigured);
        }
        Ok(Self {
            schema,
            references: None,
            newline,
        })
    }
    pub fn with_reference_data(
        schema: &'a CompiledFlowSchema,
        references: &'a PreparedReferenceEvidence,
        newline: NewlinePolicy,
    ) -> Result<Self, FlowValidationError> {
        if references.datasets().len() != schema.required_reference_data().len() {
            return Err(FlowValidationError::ReferencesNotConfigured);
        }
        // Reuse the real record preflight, including expected digest binding.
        for trace in schema.traces() {
            ReferenceRecordValidator::new(schema, trace.id(), references)
                .map_err(|e| FlowValidationError::Trace(TraceValidationError::Record(e)))?;
        }
        Ok(Self {
            schema,
            references: Some(references),
            newline,
        })
    }
    pub fn validate(
        &self,
        inputs: &[FlowTraceInput<'_>],
        parameters: &BTreeMap<String, JoinParameterValue>,
        ports: FlowExecutionPorts<'_, '_>,
    ) -> Result<FlowValidationArtifacts, FlowValidationError> {
        self.validate_with_record_observer(inputs, parameters, ports, None)
    }
    pub(crate) fn validate_with_record_observer(
        &self,
        inputs: &[FlowTraceInput<'_>],
        parameters: &BTreeMap<String, JoinParameterValue>,
        mut ports: FlowExecutionPorts<'_, '_>,
        mut record_observer: Option<&mut dyn FlowRecordObserver>,
    ) -> Result<FlowValidationArtifacts, FlowValidationError> {
        let prepared = prepare_flow(self.schema, inputs, parameters)?;
        let available = prepared
            .iter()
            .map(|(index, _)| self.schema.traces()[*index].id().clone())
            .collect::<Vec<_>>();
        if !self.schema.required_reference_data().is_empty() {
            let observers = ports
                .reference_observers
                .as_deref_mut()
                .ok_or(FlowValidationError::ReferencesNotConfigured)?;
            for trace in &available {
                observers
                    .for_trace(trace)
                    .map_err(FlowValidationError::ReferenceObserver)?;
            }
        }
        ports
            .summary
            .begin_full(
                self.schema.key(),
                &available,
                self.schema
                    .traces()
                    .iter()
                    .map(|trace| trace.id().clone())
                    .collect(),
                self.schema
                    .cross_record_plan()
                    .rules
                    .iter()
                    .map(|plan| {
                        (
                            self.schema.traces()[plan.source_trace].id().clone(),
                            plan.rule.id.to_string(),
                        )
                    })
                    .collect(),
            )
            .map_err(FlowValidationError::Summary)?;
        let mut state = FlowState::new(self.schema);
        for (index, path) in prepared {
            if (ports.control.is_cancelled)() {
                return state.finish(FlowCompletion::Cancelled, ports);
            }
            let trace = &self.schema.traces()[index];
            let mut collector = ProjectionCollector {
                flow: self.schema,
                trace_index: index,
                state: &mut state,
                keys: HashMap::new(),
                record_observer: &mut record_observer,
            };
            let mut on_event = |event| (ports.control.on_event)(FlowEvent::Trace(event));
            let mut control = TraceControl {
                is_cancelled: ports.control.is_cancelled,
                on_event: &mut on_event,
            };
            let mut validator = match self.references {
                Some(references) => {
                    // A reference snapshot can be supplied for a flow with no
                    // dependencies without inventing an observer.
                    if self.schema.required_reference_data().is_empty() {
                        TraceValidator::new(self.schema, trace.id(), self.newline)
                    } else {
                        let observer = ports
                            .reference_observers
                            .as_deref_mut()
                            .ok_or(FlowValidationError::ReferencesNotConfigured)?
                            .for_trace(trace.id())
                            .map_err(FlowValidationError::ReferenceObserver)?;
                        TraceValidator::with_reference_data(
                            self.schema,
                            trace.id(),
                            self.newline,
                            references,
                            observer,
                        )
                    }
                }
                None => TraceValidator::new(self.schema, trace.id(), self.newline),
            }
            .map_err(FlowValidationError::Trace)?;
            let completion = validator
                .validate_full_path(
                    path,
                    TracePassOnePorts {
                        sink: ports.sink,
                        summary: ports.summary,
                        control: &mut control,
                        audits: ports.audits,
                        collector: &mut collector,
                    },
                )
                .map_err(FlowValidationError::Trace)?;
            drop(validator);
            if matches!(completion, TraceCompletion::Cancelled { .. }) {
                return state.finish(FlowCompletion::Cancelled, ports);
            }
            let evidence = &mut state.traces[index];
            evidence.coverage = if evidence.unprojectable_records == 0 {
                TraceContextCoverage::Complete
            } else {
                TraceContextCoverage::Partial
            };
        }
        if (ports.control.is_cancelled)() {
            return state.finish(FlowCompletion::Cancelled, ports);
        }
        (ports.control.on_event)(FlowEvent::CrossStarted);
        let coverage = state
            .traces
            .iter()
            .map(|trace| (trace.trace_id.clone(), trace.coverage))
            .collect::<HashMap<_, _>>();
        let mut observer = FullCrossObserver {
            invalid: &mut state.invalid,
            summary: ports.summary,
            external: &mut ports.cross_observer,
            evidence: &mut state.cross,
        };
        let mut on_record =
            |count| (ports.control.on_event)(FlowEvent::CrossRecordsProcessed { count });
        let (totals, cancelled) = run_cross_record_rules(
            self.schema,
            &state.store,
            CrossRecordInputs {
                available_traces: &available,
                parameters,
            },
            ports.sink,
            &mut observer,
            CrossPassControl {
                coverage: Some(&coverage),
                is_cancelled: ports.control.is_cancelled,
                on_record: &mut on_record,
            },
        )
        .map_err(FlowValidationError::Cross)?;
        state.totals = totals;
        state.finish(
            if cancelled {
                FlowCompletion::Cancelled
            } else {
                FlowCompletion::Completed
            },
            ports,
        )
    }
}

struct FlowState {
    store: InMemoryContextStore,
    invalid: HashSet<(TraceId, u64)>,
    traces: Vec<TraceCoverageEvidence>,
    record_keys: Vec<RecordKeyEvidence>,
    cross: Vec<CrossControlEvidence>,
    totals: CrossRecordPassSummary,
}
impl FlowState {
    fn new(flow: &CompiledFlowSchema) -> Self {
        Self {
            store: InMemoryContextStore::new(),
            invalid: HashSet::new(),
            traces: flow
                .traces()
                .iter()
                .map(|trace| TraceCoverageEvidence {
                    trace_id: trace.id().clone(),
                    coverage: TraceContextCoverage::Unavailable,
                    records_processed: 0,
                    unprojectable_records: 0,
                })
                .collect(),
            record_keys: flow
                .traces()
                .iter()
                .map(|trace| RecordKeyEvidence {
                    trace_id: trace.id().clone(),
                    checked: 0,
                    duplicates: 0,
                    inconclusive: 0,
                })
                .collect(),
            cross: flow
                .cross_record_plan()
                .rules
                .iter()
                .map(|plan| CrossControlEvidence {
                    trace_id: flow.traces()[plan.source_trace].id().clone(),
                    rule_id: plan.rule.id.to_string(),
                    counts: CrossRecordPassSummary::default(),
                    skipped_evaluations: 0,
                    non_definitive: Vec::new(),
                })
                .collect(),
            totals: CrossRecordPassSummary::default(),
        }
    }
    fn finish(
        mut self,
        completion: FlowCompletion,
        ports: FlowExecutionPorts<'_, '_>,
    ) -> Result<FlowValidationArtifacts, FlowValidationError> {
        if completion == FlowCompletion::Cancelled {
            ports.summary.cancel();
        }
        // Account known source records not evaluated after cooperative cancellation.
        for control in &mut self.cross {
            let source = self
                .traces
                .iter()
                .find(|trace| trace.trace_id == control.trace_id)
                .ok_or(FlowValidationError::InvalidPlan)?;
            control.skipped_evaluations = source
                .records_processed
                .checked_sub(control.counts.evaluations)
                .ok_or(FlowValidationError::InvalidPlan)?;
            ports
                .summary
                .rules_skipped(control.skipped_evaluations)
                .map_err(FlowValidationError::Summary)?;
        }
        (ports.control.on_event)(if completion == FlowCompletion::Cancelled {
            FlowEvent::Cancelled
        } else {
            FlowEvent::Completed
        });
        Ok(FlowValidationArtifacts {
            completion,
            counters: ports.summary.counters(),
            traces: self.traces,
            record_keys: self.record_keys,
            cross: self.cross,
            cross_totals: self.totals,
        })
    }
}

struct ProjectionCollector<'a, 'schema, 'observer> {
    flow: &'schema CompiledFlowSchema,
    trace_index: usize,
    state: &'a mut FlowState,
    keys: HashMap<JoinKey, u64>,
    record_observer: &'a mut Option<&'observer mut dyn FlowRecordObserver>,
}
impl FullRecordCollector for ProjectionCollector<'_, '_, '_> {
    fn collect(
        &mut self,
        record: RawRecord<'_>,
        local: LocalRecordValidationArtifacts,
        sink: &mut dyn ValidationSink,
        summary: &mut ValidationSummaryBuilder,
    ) -> Result<(), TechnicalError> {
        let trace = &self.flow.traces()[self.trace_index];
        let evidence = &mut self.state.traces[self.trace_index];
        evidence.records_processed = record.line_number;
        evidence.coverage = TraceContextCoverage::Partial; // Complete only after observed EOF.
        if !local.valid {
            self.state
                .invalid
                .insert((trace.id().clone(), record.line_number));
        }
        if record.bytes.len() != trace.record_length() {
            evidence.unprojectable_records += 1;
            self.state.record_keys[self.trace_index].inconclusive += 1;
            return Ok(());
        }
        let view = RecordView::new(record.bytes);
        let identity = local.identity.unwrap_or_default();
        let key_evidence = &mut self.state.record_keys[self.trace_index];
        match build_join_key(&view, &trace.record_key().field_indexes, trace) {
            Ok(key) => {
                key_evidence.checked += 1;
                // The exact PK already extracted for uniqueness is borrowed;
                // the producer must finish before this row buffer is reused.
                if let Some(observer) = self.record_observer.as_deref_mut() {
                    observer.on_record(trace, &record, &key)?;
                }
                if let Some(first) = self.keys.get(&key) {
                    key_evidence.duplicates += 1;
                    let factory = super::error_factory::ErrorFactory {
                        flow_id: &self.flow.key().flow_id,
                        schema_version: &self.flow.key().version,
                        trace_id: trace.id(),
                        encoding: self.flow.encoding(),
                    };
                    let error = factory.duplicate_record_key_error(
                        record.line_number,
                        &identity,
                        &trace.record_key().error_code,
                        *first,
                    );
                    sink.on_error(&error)?;
                    summary.error_emitted(&error.code)?;
                    if self
                        .state
                        .invalid
                        .insert((trace.id().clone(), record.line_number))
                    {
                        summary.invalidate_previously_valid_record()?;
                    }
                } else {
                    self.keys.insert(key, record.line_number);
                }
            }
            Err(JoinKeyError::MissingComponent(_)) => {
                key_evidence.inconclusive += 1;
            }
            Err(error) => return Err(Box::new(error)),
        }
        // Invalid full-width records still contribute independent byte evidence.
        if trace.projection_plan().required() {
            let projected = project_record(&view, trace, identity, record.line_number)?;
            self.state.store.insert(projected)?;
        }
        Ok(())
    }
}
struct FullCrossObserver<'a, 'observer> {
    invalid: &'a mut HashSet<(TraceId, u64)>,
    summary: &'a mut ValidationSummaryBuilder,
    external: &'a mut Option<&'observer mut dyn CrossRecordEvaluationObserver>,
    evidence: &'a mut [CrossControlEvidence],
}
impl CrossRecordEvaluationObserver for FullCrossObserver<'_, '_> {
    fn on_evaluation(
        &mut self,
        trace: &TraceId,
        line: u64,
        rule: &str,
        outcome: &CrossRuleOutcome<'_>,
    ) -> Result<(), TechnicalError> {
        let entry = self
            .evidence
            .iter_mut()
            .find(|entry| &entry.trace_id == trace && entry.rule_id == rule)
            .ok_or(FlowValidationError::InvalidPlan)?;
        entry.counts = entry.counts.after(outcome)?;
        if let CrossRuleOutcome::Violated { error, .. } = outcome {
            self.summary.error_emitted(&error.code)?;
            if self.invalid.insert((trace.clone(), line)) {
                self.summary.invalidate_previously_valid_record()?;
            }
        }
        let cause = match outcome {
            CrossRuleOutcome::Unknown {
                reason,
                join_id,
                diagnostic,
            } => Some((
                NonDefinitiveCrossResult::Unknown(*reason),
                *join_id,
                *diagnostic,
            )),
            CrossRuleOutcome::NotApplicable {
                join_id,
                diagnostic,
            } => Some((
                NonDefinitiveCrossResult::NotApplicable,
                Some(*join_id),
                Some(*diagnostic),
            )),
            _ => None,
        };
        if let Some((result, join, diagnostic)) = cause {
            if let Some(cause) = entry.non_definitive.iter_mut().find(|cause| {
                cause.result == result
                    && cause.join_id.as_ref() == join
                    && cause.diagnostic.as_ref().map(|d| d.code.as_ref())
                        == diagnostic.map(|d| d.code.as_ref())
            }) {
                cause.evaluations = cause
                    .evaluations
                    .checked_add(1)
                    .ok_or(CrossRecordValidationError::CounterOverflow)?;
            } else {
                entry.non_definitive.push(CrossEvidenceCause {
                    result,
                    join_id: join.cloned(),
                    diagnostic: diagnostic.cloned(),
                    evaluations: 1,
                });
            }
        }
        match &mut self.external {
            Some(observer) => observer.on_evaluation(trace, line, rule, outcome),
            None => Ok(()),
        }
    }
}

fn prepare_flow<'b>(
    schema: &CompiledFlowSchema,
    inputs: &'b [FlowTraceInput<'b>],
    parameters: &BTreeMap<String, JoinParameterValue>,
) -> Result<Vec<(usize, &'b Path)>, FlowValidationError> {
    let mut paths = HashMap::new();
    for input in inputs {
        if schema.trace(input.trace_id).is_none() {
            return Err(FlowValidationError::UnknownTrace(input.trace_id.clone()));
        }
        if paths.insert(input.trace_id, input.path).is_some() {
            return Err(FlowValidationError::DuplicateTrace(input.trace_id.clone()));
        }
        if input.path.as_os_str().is_empty() {
            return Err(FlowValidationError::EmptyPath(input.trace_id.clone()));
        }
    }
    for &index in &schema.cross_record_plan().required_traces {
        let trace = schema
            .traces()
            .get(index)
            .ok_or(FlowValidationError::InvalidPlan)?;
        if !paths.contains_key(trace.id()) {
            return Err(FlowValidationError::MissingTrace(trace.id().clone()));
        }
    }
    let available = inputs
        .iter()
        .map(|input| input.trace_id.clone())
        .collect::<Vec<_>>();
    super::projected_context::validate_cross_inputs(
        schema,
        &CrossRecordInputs {
            available_traces: &available,
            parameters,
        },
    )
    .map_err(FlowValidationError::Inputs)?;
    // Required parameters must be checked even for empty files or before cancellation.
    for join in &schema.cross_record_plan().joins {
        for &index in &join.namespace_parameters {
            let definition = schema
                .parameters()
                .get(index)
                .ok_or(FlowValidationError::InvalidPlan)?;
            let id = match definition {
                ParameterDefinition::String { id, .. }
                | ParameterDefinition::Integer { id, .. } => id,
            };
            if !parameters.contains_key(id.as_ref()) {
                return Err(FlowValidationError::Inputs(
                    RelatedContextError::MissingNamespaceParameter,
                ));
            }
        }
    }
    Ok(schema
        .traces()
        .iter()
        .enumerate()
        .filter_map(|(index, trace)| paths.get(trace.id()).map(|path| (index, *path)))
        .collect())
}

/// Same preparation used by execution, exposed for application preflight before
/// reference loading. Returned IDs are schema ordered; no input is opened.
pub fn preflight_flow_inputs(
    schema: &CompiledFlowSchema,
    inputs: &[FlowTraceInput<'_>],
    parameters: &BTreeMap<String, JoinParameterValue>,
) -> Result<Vec<TraceId>, FlowValidationError> {
    Ok(prepare_flow(schema, inputs, parameters)?
        .into_iter()
        .map(|(index, _)| schema.traces()[index].id().clone())
        .collect())
}
