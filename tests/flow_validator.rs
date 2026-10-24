//! Synthetic filesystem integration: the real compiler, audited reader,
//! local validation, projection/index and pass 2 all participate.
use lintra::*;
use serde_json::{Value, json};
use std::{
    cell::Cell, collections::BTreeMap, error::Error, fs, io, path::PathBuf, time::UNIX_EPOCH,
};

fn raw() -> Value {
    let mut raw: Value =
        serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
    let mut trace = raw["traces"][0].clone();
    trace["record_length"] = json!(6);
    let field = trace["fields"][0].clone();
    trace["fields"] = json!(
        ["PK", "KEY", "DATA"]
            .into_iter()
            .enumerate()
            .map(|(i, id)| {
                let mut field = field.clone();
                field["id"] = json!(id);
                field["physical"]["start"] = json!(i * 2 + 1);
                field["physical"]["length"] = json!(2);
                field
            })
            .collect::<Vec<_>>()
    );
    trace["record_key"]["fields"] = json!(["PK"]);
    raw["traces"] = json!(["S", "T"].map(|id| {
        let mut trace = trace.clone();
        trace["id"] = json!(id);
        trace
    }));
    raw["required_capabilities"] = json!(["local.v1", "expressions.v1", "joins.v1"]);
    let policy = json!({"policy":"violation","error_code":"E_RELATED_RECORD_MISSING","diagnostic":{"code":"SYNTHETIC_MISSING","message_template":"Missing related record for {rule_id}"}});
    raw["traces"][0]["joins"] = json!([{"id":"S_T","target_trace":"T","source_fields":["KEY"],"target_fields":["KEY"],"cardinality":"zero_or_one","scope":"current_submission","namespace_parameters":[],"on_missing_record":policy,"on_missing_trace":policy}]);
    raw["traces"][0]["rules"] = json!([{"id":"COMPARE","when":{"const":true},"assertions":[{"kind":"field_compare","left":{"ref":{"field":"DATA"}},"operator":"==","right":{"ref":{"trace":"T","field":"DATA","join":"S_T"}},"value_type":"bytes"}],"error_code":"E_CROSS_FIELD","message_template":"Synthetic {rule_id}: {condition}"}]);
    raw
}
fn compile(value: Value) -> CompiledFlowSchema {
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw).unwrap()
}
fn builder(s: &CompiledFlowSchema) -> ValidationSummaryBuilder {
    ValidationSummaryBuilder::new(
        s.key().clone(),
        ScopeSummary::Flow {
            trace_ids: s.traces().iter().map(|t| t.id().clone()).collect(),
        },
    )
}
fn metadata() -> RunMetadata {
    RunMetadata {
        validation_mode: ValidationMode::Full,
        validator_version: "synthetic".into(),
        schema_hash: "synthetic".into(),
        reference_data_versions: BTreeMap::new(),
        started_at: UNIX_EPOCH,
        inputs: vec![],
    }
}
#[derive(Default)]
struct CrossEvents {
    values: Vec<(String, u64, String, &'static str)>,
    fail: bool,
}
impl CrossRecordEvaluationObserver for CrossEvents {
    fn on_evaluation(
        &mut self,
        trace: &TraceId,
        line: u64,
        rule: &str,
        outcome: &CrossRuleOutcome<'_>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.fail {
            return Err(Box::new(io::Error::other("synthetic observer failure")));
        }
        let kind = match outcome {
            CrossRuleOutcome::Satisfied => "satisfied",
            CrossRuleOutcome::Violated { .. } => "violation",
            CrossRuleOutcome::Unknown { .. } => "unknown",
            CrossRuleOutcome::NotApplicable { .. } => "not_applicable",
        };
        self.values.push((trace.0.clone(), line, rule.into(), kind));
        Ok(())
    }
}
struct Bundle {
    _temp: tempfile::TempDir,
    source: PathBuf,
    target: PathBuf,
}
impl Bundle {
    fn new(source: &[u8], target: &[u8]) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let s = temp.path().join("sorgente é.txt");
        let t = temp.path().join("target.txt");
        fs::write(&s, source).unwrap();
        fs::write(&t, target).unwrap();
        Self {
            _temp: temp,
            source: s,
            target: t,
        }
    }
    fn inputs<'a>(&'a self, s: &'a CompiledFlowSchema) -> [FlowTraceInput<'a>; 2] {
        [
            FlowTraceInput {
                trace_id: s.traces()[1].id(),
                path: &self.target,
            },
            FlowTraceInput {
                trace_id: s.traces()[0].id(),
                path: &self.source,
            },
        ]
    }
}
struct ResultRun {
    artifacts: FlowValidationArtifacts,
    sink: VecSink,
    audits: InputAuditSession,
    summary: ValidationSummaryBuilder,
    events: Vec<FlowEvent>,
    cross: CrossEvents,
}
fn run(s: &CompiledFlowSchema, files: &Bundle) -> ResultRun {
    let mut sink = VecSink::default();
    let audits = InputAuditSession::new();
    let mut summary = builder(s);
    let mut events = vec![];
    let mut cross = CrossEvents::default();
    let artifacts = FlowValidator::new(s, NewlinePolicy::AutoLfOrCrlf)
        .unwrap()
        .validate(
            &files.inputs(s),
            &BTreeMap::new(),
            FlowExecutionPorts {
                sink: &mut sink,
                summary: &mut summary,
                audits: &audits,
                control: &mut FlowControl {
                    is_cancelled: &|| false,
                    on_event: &mut |event| events.push(event),
                },
                reference_observers: None,
                cross_observer: &mut cross,
            },
        )
        .unwrap();
    ResultRun {
        artifacts,
        sink,
        audits,
        summary,
        events,
        cross,
    }
}
#[test]
fn real_two_pass_order_audit_and_no_local_only_skips() {
    use sha2::{Digest, Sha256};
    let s = compile(raw());
    let files = Bundle::new(b"01AAOK\r\n02BBOK\n", b"11AAOK\n12BBOK");
    let out = run(&s, &files);
    assert_eq!(out.artifacts.completion, FlowCompletion::Completed);
    assert_eq!(
        out.artifacts.counters,
        ValidationCounters {
            records_processed: 4,
            valid_records: 4,
            invalid_records: 0,
            total_errors: 0,
            skipped_evaluations: 0
        }
    );
    assert_eq!(out.artifacts.cross_totals.satisfied, 2);
    assert!(out.sink.errors().is_empty());
    let order = out
        .events
        .iter()
        .filter_map(|e| {
            if let FlowEvent::Trace(TraceEvent::Started { trace_id }) = e {
                Some(trace_id.0.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(order, ["S", "T"]);
    assert_eq!(
        out.cross.values.iter().map(|e| e.1).collect::<Vec<_>>(),
        [1, 2]
    );
    let metadata = out.audits.finish(metadata()).unwrap();
    assert_eq!(metadata.inputs.len(), 2);
    assert_eq!(metadata.inputs[0].file_name, "sorgente é.txt");
    for (audit, path) in metadata.inputs.iter().zip([&files.source, &files.target]) {
        assert_eq!(audit.bytes_read, fs::metadata(path).unwrap().len());
        assert!(
            matches!(&audit.completion,InputAuditCompletion::Complete {sha256} if sha256.0.as_slice()==Sha256::digest(fs::read(path).unwrap()).as_slice())
        );
    }
    // T13-07 must persist the evidence envelope. Legacy finish cannot discard it.
    assert!(matches!(
        out.summary.finish(metadata),
        Err(SummaryBuildError::UnreportedFullEvidence)
    ));
}
#[test]
fn bundle_errors_and_namespace_fail_before_opening_any_file() {
    let mut value = raw();
    value["parameters"] = json!([{"id":"YEAR","type":"integer","min":2020,"max":2030}]);
    value["traces"][0]["joins"][0]["namespace_parameters"] = json!(["YEAR"]);
    let s = compile(value);
    let files = Bundle::new(b"01AAOK", b"11AAOK");
    let unknown = TraceId("UNKNOWN".into());
    let cases = vec![
        vec![FlowTraceInput {
            trace_id: &unknown,
            path: &files.source,
        }],
        vec![
            FlowTraceInput {
                trace_id: s.traces()[0].id(),
                path: &files.source,
            },
            FlowTraceInput {
                trace_id: s.traces()[0].id(),
                path: &files.target,
            },
        ],
        vec![FlowTraceInput {
            trace_id: s.traces()[0].id(),
            path: &files.source,
        }],
        vec![FlowTraceInput {
            trace_id: s.traces()[0].id(),
            path: std::path::Path::new(""),
        }],
        files.inputs(&s).into_iter().collect(),
    ];
    for (index, inputs) in cases.into_iter().enumerate() {
        let audits = InputAuditSession::new();
        let mut sink = VecSink::default();
        let mut summary = builder(&s);
        let mut cross = CrossEvents::default();
        let mut events = vec![];
        let err = FlowValidator::new(&s, NewlinePolicy::AutoLfOrCrlf)
            .unwrap()
            .validate(
                &inputs,
                &BTreeMap::new(),
                FlowExecutionPorts {
                    sink: &mut sink,
                    summary: &mut summary,
                    audits: &audits,
                    control: &mut FlowControl {
                        is_cancelled: &|| false,
                        on_event: &mut |e| events.push(e),
                    },
                    reference_observers: None,
                    cross_observer: &mut cross,
                },
            )
            .unwrap_err();
        assert!(match index {
            0 => matches!(err, FlowValidationError::UnknownTrace(_)),
            1 => matches!(err, FlowValidationError::DuplicateTrace(_)),
            2 => matches!(err, FlowValidationError::MissingTrace(_)),
            3 => matches!(err, FlowValidationError::EmptyPath(_)),
            _ => matches!(
                err,
                FlowValidationError::Inputs(RelatedContextError::MissingNamespaceParameter)
            ),
        });
        assert!(audits.finish(metadata()).unwrap().inputs.is_empty());
        assert!(events.is_empty());
        assert_eq!(summary.counters().records_processed, 0);
    }
}
#[test]
fn multiple_cross_failures_reclassify_a_source_once() {
    let mut value = raw();
    let mut second = value["traces"][0]["rules"][0].clone();
    second["id"] = json!("SECOND");
    value["traces"][0]["rules"]
        .as_array_mut()
        .unwrap()
        .push(second);
    let s = compile(value);
    let out = run(&s, &Bundle::new(b"01AANO\n02BBNO", b"11AAOK\n12BBOK"));
    assert_eq!(
        (
            out.artifacts.counters.records_processed,
            out.artifacts.counters.valid_records,
            out.artifacts.counters.invalid_records,
            out.artifacts.counters.total_errors
        ),
        (4, 2, 2, 4)
    );
    assert_eq!(out.artifacts.cross_totals.violations, 4);
    assert_eq!(
        out.sink
            .errors()
            .iter()
            .map(|e| e.line_number)
            .collect::<Vec<_>>(),
        [1, 1, 2, 2]
    );
}
#[test]
fn record_keys_are_trace_local_and_do_not_deduplicate_join_parents() {
    let s = compile(raw());
    let out = run(&s, &Bundle::new(b"01AAOK\n01AAOK", b"01AAOK\n01AAOK"));
    assert_eq!(
        out.artifacts
            .record_keys
            .iter()
            .map(|e| e.duplicates)
            .collect::<Vec<_>>(),
        [1, 1]
    );
    assert_eq!(out.artifacts.counters.invalid_records, 2);
    assert_eq!(out.artifacts.counters.total_errors, 2);
    assert_eq!(out.artifacts.cross_totals.unknown, 2); // two parents remain ambiguous
    assert!(
        out.artifacts.cross[0]
            .non_definitive
            .iter()
            .any(|c| c.result == NonDefinitiveCrossResult::Unknown(UnknownReason::AmbiguousJoin))
    );
}
#[test]
fn bad_target_width_prevents_false_missing_but_does_not_hide_local_errors() {
    let s = compile(raw());
    let out = run(&s, &Bundle::new(b"01AAOK", b"bad\n11BBOK"));
    assert_eq!(out.sink.errors().len(), 1);
    assert_eq!(out.sink.errors()[0].code, ValidationCode::RecordLength);
    assert_eq!(out.artifacts.cross_totals.unknown, 1);
    assert_eq!(
        out.artifacts.cross[0].non_definitive[0].result,
        NonDefinitiveCrossResult::Unknown(UnknownReason::IncompleteRelatedContext)
    );
    assert_eq!(
        out.artifacts.traces[1].coverage,
        TraceContextCoverage::Partial
    );
    assert_eq!(out.artifacts.traces[1].unprojectable_records, 1);
    assert!(
        out.audits
            .finish(metadata())
            .unwrap()
            .inputs
            .iter()
            .all(|a| matches!(a.completion, InputAuditCompletion::Complete { .. }))
    );
    let out = run(&s, &Bundle::new(b"bad\n01AAOK", b"11AAOK"));
    assert_eq!(out.artifacts.cross[0].skipped_evaluations, 1);
    assert_eq!(out.artifacts.cross_totals.satisfied, 1);
}
#[test]
fn existence_positive_survives_partial_index_negative_is_unknown() {
    let mut value = raw();
    value["traces"][0]["rules"][0]["assertions"] = json!([{"kind":"record_exists","join":"S_T"}]);
    let s = compile(value);
    let out = run(&s, &Bundle::new(b"01AAOK\n02BBOK", b"11AAOK\nbad"));
    assert_eq!(
        (
            out.artifacts.cross_totals.satisfied,
            out.artifacts.cross_totals.unknown
        ),
        (1, 1)
    );
    assert_eq!(out.sink.errors().len(), 1);
    // All targets checked and absent: this time the missing policy is definitive.
    let out = run(&s, &Bundle::new(b"01AAOK", b""));
    assert_eq!(out.artifacts.cross_totals.violations, 1);
    assert_eq!(
        out.sink.errors()[0].code,
        ValidationCode::RelatedRecordMissing
    );
}
#[test]
fn cancellation_at_each_phase_preserves_observed_audit_and_atomic_record() {
    let s = compile(raw());
    let files = Bundle::new(b"01AAOK\n02BBOK", b"11AAOK\n12BBOK");
    for stage in [0, 1, 2, 3, 4] {
        let cancel = Cell::new(stage == 0);
        let audits = InputAuditSession::new();
        let mut summary = builder(&s);
        let mut sink = VecSink::default();
        let mut cross = CrossEvents::default();
        let mut events = vec![];
        let out = FlowValidator::new(&s, NewlinePolicy::AutoLfOrCrlf)
            .unwrap()
            .validate(
                &files.inputs(&s),
                &BTreeMap::new(),
                FlowExecutionPorts {
                    sink: &mut sink,
                    summary: &mut summary,
                    audits: &audits,
                    control: &mut FlowControl {
                        is_cancelled: &|| cancel.get(),
                        on_event: &mut |event| {
                            if matches!(
                                (&event, stage),
                                (FlowEvent::Trace(TraceEvent::Started { .. }), 1)
                                    | (FlowEvent::Trace(TraceEvent::Completed { .. }), 2)
                                    | (FlowEvent::CrossStarted, 3)
                                    | (FlowEvent::CrossRecordsProcessed { count: 1 }, 4)
                            ) {
                                cancel.set(true);
                            }
                            events.push(event);
                        },
                    },
                    reference_observers: None,
                    cross_observer: &mut cross,
                },
            )
            .unwrap();
        assert_eq!(out.completion, FlowCompletion::Cancelled);
        assert!(matches!(events.last(), Some(FlowEvent::Cancelled)));
        let audits = audits.finish(metadata()).unwrap();
        match stage {
            0 => assert!(audits.inputs.is_empty()),
            1 => {
                assert_eq!(out.counters.records_processed, 0);
                assert!(matches!(
                    audits.inputs[0].completion,
                    InputAuditCompletion::Incomplete
                ));
            }
            2 => {
                assert_eq!(audits.inputs.len(), 1);
                assert_eq!(out.counters.records_processed, 2);
                assert_eq!(out.cross_totals.evaluations, 0);
            }
            3 => {
                assert_eq!(audits.inputs.len(), 2);
                assert_eq!(out.cross_totals.evaluations, 0);
            }
            _ => {
                assert_eq!(out.cross_totals.evaluations, 1);
                assert_eq!(out.cross[0].skipped_evaluations, 1);
            }
        }
    }
}
#[test]
fn observer_failure_is_technical_and_cannot_finalize_a_full_summary() {
    let s = compile(raw());
    let files = Bundle::new(b"01AANO", b"11AAOK");
    let audits = InputAuditSession::new();
    let mut summary = builder(&s);
    let mut sink = VecSink::default();
    let mut cross = CrossEvents {
        fail: true,
        ..Default::default()
    };
    let err = FlowValidator::new(&s, NewlinePolicy::AutoLfOrCrlf)
        .unwrap()
        .validate(
            &files.inputs(&s),
            &BTreeMap::new(),
            FlowExecutionPorts {
                sink: &mut sink,
                summary: &mut summary,
                audits: &audits,
                control: &mut FlowControl {
                    is_cancelled: &|| false,
                    on_event: &mut |_| {},
                },
                reference_observers: None,
                cross_observer: &mut cross,
            },
        )
        .unwrap_err();
    assert!(matches!(
        err,
        FlowValidationError::Cross(CrossRecordValidationError::Observer(_))
    ));
    assert!(err.source().unwrap().source().is_some());
    assert_eq!(sink.errors().len(), 1);
    assert_eq!(summary.counters().total_errors, 1);
    assert!(matches!(
        summary.finish(audits.finish(metadata()).unwrap()),
        Err(SummaryBuildError::UnreportedFullEvidence)
    ));
}

#[test]
fn cancellation_during_pass_one_does_not_drain_for_an_eof_digest() {
    let s = compile(raw());
    let mut bytes = Vec::new();
    let alphabet = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    for index in 0..1100 {
        bytes.extend([alphabet[index / 36], alphabet[index % 36]]);
        bytes.extend_from_slice(b"AAOK\n");
    }
    let files = Bundle::new(&bytes, b"11AAOK");
    let cancel = Cell::new(false);
    let audits = InputAuditSession::new();
    let mut summary = builder(&s);
    let mut sink = VecSink::default();
    let mut cross = CrossEvents::default();
    let out = FlowValidator::new(&s, NewlinePolicy::AutoLfOrCrlf)
        .unwrap()
        .validate(
            &files.inputs(&s),
            &BTreeMap::new(),
            FlowExecutionPorts {
                sink: &mut sink,
                summary: &mut summary,
                audits: &audits,
                control: &mut FlowControl {
                    is_cancelled: &|| cancel.get(),
                    on_event: &mut |event| {
                        if matches!(
                            event,
                            FlowEvent::Trace(TraceEvent::RecordsProcessed { count: 1024, .. })
                        ) {
                            cancel.set(true);
                        }
                    },
                },
                reference_observers: None,
                cross_observer: &mut cross,
            },
        )
        .unwrap();
    assert_eq!(out.counters.records_processed, 1024);
    assert_eq!(out.counters.valid_records, 1024);
    assert_eq!(out.cross[0].skipped_evaluations, 1024);
    assert_eq!(out.traces[0].coverage, TraceContextCoverage::Partial);
    let inputs = audits.finish(metadata()).unwrap().inputs;
    assert_eq!(inputs.len(), 1);
    assert!(matches!(
        inputs[0].completion,
        InputAuditCompletion::Incomplete
    ));
    assert!(cross.values.is_empty());
}

#[test]
fn already_invalid_local_record_is_not_reclassified_twice() {
    let mut value = raw();
    value["traces"][0]["fields"][2]["constraints"] =
        json!([{"kind":"allowed_values","target":"raw","values":["OK"]}]);
    let s = compile(value);
    let out = run(&s, &Bundle::new(b"01AANO", b"11AAOK"));
    assert_eq!(
        (
            out.artifacts.counters.records_processed,
            out.artifacts.counters.valid_records,
            out.artifacts.counters.invalid_records,
            out.artifacts.counters.total_errors
        ),
        (2, 1, 1, 2)
    );
    assert_eq!(out.artifacts.cross_totals.violations, 1);
    assert_eq!(out.sink.errors()[0].code, ValidationCode::InvalidDomain);
    assert_eq!(out.sink.errors()[1].code, ValidationCode::CrossField);
}

#[test]
fn set_predicate_negative_on_partial_context_is_not_a_violation() {
    let mut value = raw();
    value["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!("joins.sets.v1"));
    value["traces"][0]["joins"][0]["cardinality"] = json!("zero_or_many");
    value["traces"][0]["rules"][0]["assertions"] = json!([{"kind":"set_exists","join":"S_T","predicate":{"eq":{"ref":{"field":"DATA"},"value":"OK"}}}]);
    let s = compile(value);
    let out = run(&s, &Bundle::new(b"01AAOK", b"11AANO\nbad"));
    assert_eq!(out.artifacts.cross_totals.unknown, 1);
    assert_eq!(out.artifacts.cross_totals.violations, 0);
    assert_eq!(out.sink.errors().len(), 1);
    let out = run(&s, &Bundle::new(b"01AAOK", b"11AANO\n12AAOK\nbad"));
    assert_eq!(out.artifacts.cross_totals.satisfied, 1);
}

#[derive(Default)]
struct MembershipEvents {
    verified: u64,
    provisional: u64,
    inconclusive: u64,
}
impl ReferenceEvaluationObserver for MembershipEvents {
    fn on_evaluation(
        &mut self,
        _control: ReferenceControl<'_>,
        evaluation: ReferenceConstraintEvaluation<'_>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match evaluation {
            ReferenceConstraintEvaluation::Membership(MembershipEvaluation::VerifiedPresent(_)) => {
                self.verified += 1
            }
            ReferenceConstraintEvaluation::Membership(
                MembershipEvaluation::ProvisionalPresent(_),
            ) => self.provisional += 1,
            ReferenceConstraintEvaluation::Membership(MembershipEvaluation::Inconclusive) => {
                self.inconclusive += 1
            }
            _ => {}
        }
        Ok(())
    }
}
#[derive(Default)]
struct ReferenceObservers {
    source: MembershipEvents,
    target: MembershipEvents,
}
impl FlowReferenceObservers for ReferenceObservers {
    fn for_trace(
        &mut self,
        trace: &TraceId,
    ) -> Result<&mut dyn ReferenceEvaluationObserver, Box<dyn Error + Send + Sync>> {
        match trace.0.as_str() {
            "S" => Ok(&mut self.source),
            "T" => Ok(&mut self.target),
            _ => Err(Box::new(io::Error::other("unexpected synthetic trace"))),
        }
    }
}
#[test]
fn full_pass_one_uses_real_reference_snapshot_and_separate_trace_observers() {
    use sha2::{Digest, Sha256};
    let dataset=b"{\"dataset_format_version\":1,\"dataset_id\":\"synthetic\",\"version\":\"1\",\"keys_hex\":[\"4f4b\"]}";
    for state in [
        DatasetAvailability::Verified,
        DatasetAvailability::Degraded,
        DatasetAvailability::Unavailable,
    ] {
        let mut value = raw();
        let files = Bundle::new(b"01AAOK", b"11AAOK");
        let digest = if state == DatasetAvailability::Degraded {
            "0".repeat(64)
        } else {
            Sha256::digest(dataset)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        };
        value["reference_data"] = json!([{"id":"MEMBERS","dataset_id":"synthetic","version":"1","path":"members.json","sha256":digest,"metadata_attributes":[]}]);
        for trace in value["traces"].as_array_mut().unwrap() {
            trace["fields"][2]["constraints"] = json!([{"kind":"reference_data","datasets":["MEMBERS"],"fallback_values":[],"metadata_bindings":[]}]);
        }
        let s = compile(value);
        if state != DatasetAvailability::Unavailable {
            fs::write(files._temp.path().join("members.json"), dataset).unwrap();
        }
        let snapshot = FileSystemReferenceDataPreparer::new(files._temp.path().to_path_buf())
            .unwrap()
            .prepare(
                s.required_reference_data(),
                DatasetPreparationPolicy::ContinueWithAvailableEvidence,
            )
            .unwrap();
        assert_eq!(snapshot.datasets()[0].availability(), state);
        assert!(matches!(
            FlowValidator::new(&s, NewlinePolicy::AutoLfOrCrlf),
            Err(FlowValidationError::ReferencesNotConfigured)
        ));
        let validator =
            FlowValidator::with_reference_data(&s, &snapshot, NewlinePolicy::AutoLfOrCrlf).unwrap();
        let mut observers = ReferenceObservers::default();
        let audits = InputAuditSession::new();
        let mut summary = builder(&s);
        let mut sink = VecSink::default();
        let mut cross = CrossEvents::default();
        let out = validator
            .validate(
                &files.inputs(&s),
                &BTreeMap::new(),
                FlowExecutionPorts {
                    sink: &mut sink,
                    summary: &mut summary,
                    audits: &audits,
                    control: &mut FlowControl {
                        is_cancelled: &|| false,
                        on_event: &mut |_| {},
                    },
                    reference_observers: Some(&mut observers),
                    cross_observer: &mut cross,
                },
            )
            .unwrap();
        assert_eq!(out.cross_totals.satisfied, 1);
        assert_eq!(out.counters.total_errors, 0);
        assert!(sink.errors().is_empty());
        for observer in [&observers.source, &observers.target] {
            assert_eq!(
                (
                    observer.verified,
                    observer.provisional,
                    observer.inconclusive
                ),
                match state {
                    DatasetAvailability::Verified => (1, 0, 0),
                    DatasetAvailability::Degraded => (0, 1, 0),
                    DatasetAvailability::Unavailable => (0, 0, 1),
                }
            );
        }
    }
}

#[test]
fn explicit_coverage_must_match_availability_before_lookup() {
    use std::collections::HashMap;
    let s = compile(raw());
    let source = project_record(
        &RecordView::new(b"01AAOK"),
        &s.traces()[0],
        RecordIdentity::default(),
        1,
    )
    .unwrap();
    let store = InMemoryContextStore::new();
    let available = s
        .traces()
        .iter()
        .map(|t| t.id().clone())
        .collect::<Vec<_>>();
    let parameters = BTreeMap::new();
    for coverage in [
        HashMap::new(),
        HashMap::from([
            (available[0].clone(), TraceContextCoverage::Complete),
            (available[1].clone(), TraceContextCoverage::Unavailable),
        ]),
    ] {
        assert!(matches!(
            ProjectedEvaluationContext::with_coverage(
                &s,
                &source,
                &store,
                CoveredCrossRecordInputs {
                    inputs: CrossRecordInputs {
                        available_traces: &available,
                        parameters: &parameters
                    },
                    coverage: &coverage
                }
            ),
            Err(RelatedContextError::InconsistentCoverage)
        ));
    }
}

struct FailedSink;
impl ValidationSink for FailedSink {
    fn on_error(&mut self, _: &ValidationError) -> Result<(), ReportError> {
        Err(ReportError::Write(io::Error::other(
            "synthetic Full sink failure",
        )))
    }
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        panic!("Full core does not publish summaries")
    }
}
#[test]
fn sink_failure_before_acknowledgement_does_not_increase_cross_counts() {
    let s = compile(raw());
    let files = Bundle::new(b"01AANO", b"11AAOK");
    let audits = InputAuditSession::new();
    let mut summary = builder(&s);
    let mut cross = CrossEvents::default();
    let err = FlowValidator::new(&s, NewlinePolicy::AutoLfOrCrlf)
        .unwrap()
        .validate(
            &files.inputs(&s),
            &BTreeMap::new(),
            FlowExecutionPorts {
                sink: &mut FailedSink,
                summary: &mut summary,
                audits: &audits,
                control: &mut FlowControl {
                    is_cancelled: &|| false,
                    on_event: &mut |_| {},
                },
                reference_observers: None,
                cross_observer: &mut cross,
            },
        )
        .unwrap_err();
    assert!(matches!(
        err,
        FlowValidationError::Cross(CrossRecordValidationError::Report(ReportError::Write(_)))
    ));
    assert_eq!(summary.counters().total_errors, 0);
    assert_eq!(summary.counters().invalid_records, 0);
    assert!(cross.values.is_empty());
}
