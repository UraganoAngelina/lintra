//! End-to-end pass 2 from real compiler/projection/store, entirely synthetic.
use lintra::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, error::Error, io};

fn policy(name: &str) -> Value {
    let mut value = json!({"policy":name,"diagnostic":{"code":"SYNTHETIC_RELATED","message_template":"Synthetic relation for {rule_id}"}});
    if name == "violation" {
        value["error_code"] = json!("E_RELATED_RECORD_MISSING");
    }
    value
}
fn external(target: &str, field: &str, join: &str) -> Value {
    json!({"trace":target,"field":field,"join":join})
}
fn comparison() -> Value {
    json!({"kind":"field_compare","left":{"ref":{"field":"DATA"}},"operator":"==",
        "right":{"ref":external("T","DATA","S_T")},"value_type":"bytes"})
}
fn rule(id: &str, assertion: Value) -> Value {
    json!({"id":id,"when":{"const":true},"assertions":[assertion],"error_code":"E_CROSS_FIELD","message_template":"Synthetic {rule_id}: {condition}"})
}
fn sample(missing: &str, assertion: Value) -> Value {
    let mut raw: Value =
        serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
    let mut record = raw["traces"][0].clone();
    record["record_length"] = json!(8);
    record["fields"] = json!(
        ["PK", "KEY", "DATA", "SPARE"]
            .into_iter()
            .enumerate()
            .map(|(i, id)| {
                let mut field = record["fields"][0].clone();
                field["id"] = json!(id);
                field["physical"]["start"] = json!(i * 2 + 1);
                field["physical"]["length"] = json!(2);
                if id == "DATA" || id == "SPARE" {
                    field["missing_value"] = json!({"kind":"spaces"});
                }
                field
            })
            .collect::<Vec<_>>()
    );
    record["record_key"]["fields"] = json!(["PK"]);
    raw["traces"] = json!(["S", "T", "U"].map(|id| {
        let mut record = record.clone();
        record["id"] = json!(id);
        record
    }));
    raw["required_capabilities"] = json!(["local.v1", "expressions.v1", "joins.v1"]);
    raw["traces"][0]["joins"] = json!([{"id":"S_T","target_trace":"T","source_fields":["KEY","PK"],"target_fields":["KEY","PK"],
        "cardinality":"zero_or_one","scope":"current_submission","namespace_parameters":[],
        "on_missing_record":policy(missing),"on_missing_trace":policy("unknown")}]);
    raw["traces"][0]["rules"] = json!([rule("CROSS", assertion)]);
    raw
}
fn compile(raw: Value) -> Result<CompiledFlowSchema, SchemaCompilationError> {
    let mut raw: RawFlowSchema = serde_json::from_value(raw).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw)
}
fn project(flow: &CompiledFlowSchema, trace: &str, bytes: &[u8], line: u64) -> ProjectedRecord {
    project_record(
        &RecordView::new(bytes),
        flow.trace(&TraceId(trace.into())).unwrap(),
        RecordIdentity::default(),
        line,
    )
    .unwrap()
}
#[derive(Debug)]
struct Event {
    trace: String,
    line: u64,
    rule: String,
    kind: &'static str,
    reason: Option<UnknownReason>,
    join: Option<String>,
    diagnostic: Option<String>,
}
#[derive(Default)]
struct Events {
    values: Vec<Event>,
    fail: bool,
}
impl CrossRecordEvaluationObserver for Events {
    fn on_evaluation(
        &mut self,
        trace: &TraceId,
        line: u64,
        rule: &str,
        outcome: &CrossRuleOutcome<'_>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.fail {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "synthetic observer",
            )));
        }
        let (kind, reason, join, diagnostic) = match outcome {
            CrossRuleOutcome::Satisfied => ("satisfied", None, None, None),
            CrossRuleOutcome::Violated { join_id, .. } => ("violated", None, *join_id, None),
            CrossRuleOutcome::NotApplicable {
                join_id,
                diagnostic,
            } => (
                "not_applicable",
                None,
                Some(*join_id),
                Some(diagnostic.code.to_string()),
            ),
            CrossRuleOutcome::Unknown {
                reason,
                join_id,
                diagnostic,
            } => (
                "unknown",
                Some(*reason),
                *join_id,
                diagnostic.map(|d| d.code.to_string()),
            ),
        };
        self.values.push(Event {
            trace: trace.0.clone(),
            line,
            rule: rule.into(),
            kind,
            reason,
            join: join.map(|id| id.0.clone()),
            diagnostic,
        });
        Ok(())
    }
}
fn run(
    flow: &CompiledFlowSchema,
    store: &dyn ContextStore,
    target_available: bool,
) -> (CrossRecordPassSummary, VecSink, Events) {
    let traces = if target_available {
        vec![
            TraceId("S".into()),
            TraceId("T".into()),
            TraceId("U".into()),
        ]
    } else {
        vec![TraceId("S".into())]
    };
    let mut sink = VecSink::default();
    let mut observer = Events::default();
    let totals = validate_cross_record_rules(
        flow,
        store,
        CrossRecordInputs {
            available_traces: &traces,
            parameters: &BTreeMap::new(),
        },
        &mut sink,
        &mut observer,
    )
    .unwrap();
    (totals, sink, observer)
}

#[test]
fn compiler_projects_cross_operands_and_pass_reports_real_source_and_target_context() {
    let flow = compile(sample("unknown", comparison())).unwrap();
    assert!(flow.traces()[0].local_rules().is_empty());
    assert_eq!(flow.cross_record_plan().rules.len(), 1);
    for trace in &flow.traces()[..2] {
        assert_eq!(trace.projection_plan().field_indexes, [0, 1, 2]);
    }
    for (target, expected_errors) in [(b"00AA01ZZ".as_slice(), 0), (b"00AA02YY".as_slice(), 1)] {
        let mut store = InMemoryContextStore::new();
        store.insert(project(&flow, "S", b"00AA01ZZ", 3)).unwrap();
        store.insert(project(&flow, "T", target, 7)).unwrap();
        let (counts, sink, events) = run(&flow, &store, true);
        assert_eq!(counts.evaluations, 1);
        assert_eq!(counts.violations, expected_errors);
        assert_eq!(counts.unknown, 0);
        assert_eq!(events.values[0].trace, "S");
        assert_eq!(events.values[0].line, 3);
        if expected_errors == 1 {
            let error = &sink.errors()[0];
            error.validate().unwrap();
            assert_eq!(error.code, ValidationCode::CrossField);
            assert_eq!(error.trace_id, TraceId("S".into()));
            assert_eq!(error.line_number, 3);
            let field = error.field.as_ref().unwrap();
            assert_eq!(field.id, FieldId("DATA".into()));
            assert_eq!(
                (field.start_one_based, field.end_one_based_inclusive),
                (5, 6)
            );
            assert_eq!(error.actual.as_ref().unwrap().display, "01");
            assert_eq!(
                error
                    .context
                    .iter()
                    .map(|item| (&*item.trace_id.0, &*item.field_id.0, &*item.value.display))
                    .collect::<Vec<_>>(),
                [("S", "DATA", "01"), ("T", "DATA", "02")]
            );
            assert!(error.message.contains("CROSS"));
            assert!(!error.message.contains("field_index"));
        } else {
            assert_eq!(counts.satisfied, 1);
            assert!(sink.errors().is_empty());
        }
    }
}

#[test]
fn declared_missing_policies_are_disjoint_and_unknown_does_not_emit_validation_errors() {
    for (name, kind) in [
        ("violation", "violated"),
        ("not_applicable", "not_applicable"),
        ("unknown", "unknown"),
    ] {
        let flow = compile(sample(name, comparison())).unwrap();
        let mut store = InMemoryContextStore::new();
        store.insert(project(&flow, "S", b"00AA01ZZ", 1)).unwrap();
        let (counts, sink, events) = run(&flow, &store, true);
        assert_eq!(events.values[0].kind, kind);
        assert_eq!(events.values[0].join.as_deref(), Some("S_T"));
        assert_eq!(counts.satisfied, 0);
        assert_eq!(counts.evaluations, 1);
        assert_eq!(counts.violations, u64::from(name == "violation"));
        assert_eq!(counts.not_applicable, u64::from(name == "not_applicable"));
        assert_eq!(counts.unknown, u64::from(name == "unknown"));
        if name == "violation" {
            assert_eq!(sink.errors()[0].code, ValidationCode::RelatedRecordMissing);
            assert!(sink.errors()[0].field.is_none());
        } else {
            assert!(sink.errors().is_empty());
            assert_eq!(
                events.values[0].diagnostic.as_deref(),
                Some("SYNTHETIC_RELATED")
            );
        }
        let (counts, sink, events) = run(&flow, &store, false);
        assert_eq!(counts.unknown, 1);
        assert!(sink.errors().is_empty());
        assert_eq!(
            events.values[0].reason,
            Some(UnknownReason::TraceInputUnavailable)
        );
    }
}

#[test]
fn nested_boolean_cause_uses_the_determining_join_not_an_earlier_dominated_branch() {
    let mut raw = sample(
        "violation",
        json!({"kind":"field_present","field":{"field":"DATA"}}),
    );
    let mut other = raw["traces"][0]["joins"][0].clone();
    other["id"] = json!("S_U");
    other["target_trace"] = json!("U");
    other["on_missing_record"] = policy("not_applicable");
    raw["traces"][0]["joins"]
        .as_array_mut()
        .unwrap()
        .push(other);
    let missing_a = json!({"is_present":{"ref":external("T","DATA","S_T")}});
    let missing_b = json!({"is_present":{"ref":external("U","DATA","S_U")}});
    raw["traces"][0]["rules"][0]["when"] =
        json!({"or":[{"and":[{"const":false},missing_a]},missing_b]});
    let flow = compile(raw).unwrap();
    let mut store = InMemoryContextStore::new();
    store.insert(project(&flow, "S", b"00AA01ZZ", 1)).unwrap();
    let (counts, sink, events) = run(&flow, &store, true);
    assert_eq!(counts.not_applicable, 1);
    assert_eq!(events.values[0].join.as_deref(), Some("S_U"));
    assert!(sink.errors().is_empty());
    for when in [
        json!({"and":[{"const":false},{"is_present":{"ref":external("T","DATA","S_T")}}]}),
        json!({"or":[{"const":true},{"is_present":{"ref":external("T","DATA","S_T")}}]}),
    ] {
        let mut raw = sample(
            "violation",
            json!({"kind":"field_present","field":{"field":"DATA"}}),
        );
        raw["traces"][0]["rules"][0]["when"] = when;
        let flow = compile(raw).unwrap();
        let mut store = InMemoryContextStore::new();
        store.insert(project(&flow, "S", b"00AA01ZZ", 1)).unwrap();
        let (counts, sink, _) = run(&flow, &store, true);
        assert_eq!(counts.satisfied, 1);
        assert!(sink.errors().is_empty());
    }
}

#[test]
fn active_not_applicable_scope_is_not_hidden_by_an_independent_false_assertion() {
    for reverse in [false, true] {
        let mut raw = sample("not_applicable", comparison());
        let mut assertions = vec![
            json!({"kind":"condition_true","condition":{"const":false}}),
            comparison(),
        ];
        if reverse {
            assertions.reverse();
        }
        raw["traces"][0]["rules"][0]["assertions"] = json!(assertions);
        let flow = compile(raw).unwrap();
        let mut store = InMemoryContextStore::new();
        store.insert(project(&flow, "S", b"00AA01ZZ", 1)).unwrap();
        let (counts, sink, _) = run(&flow, &store, true);
        assert_eq!(counts.not_applicable, 1);
        assert_eq!(counts.violations, 0);
        assert!(sink.errors().is_empty());
    }
}

#[test]
fn record_exists_and_set_exists_use_complete_sets_and_preserve_unknown_predicates() {
    let raw = sample("violation", json!({"kind":"record_exists","join":"S_T"}));
    let flow = compile(raw).unwrap();
    for count in 0..=2 {
        let mut store = InMemoryContextStore::new();
        store.insert(project(&flow, "S", b"00AA01ZZ", 1)).unwrap();
        for line in 1..=count {
            store
                .insert(project(&flow, "T", b"00AA01ZZ", line))
                .unwrap();
        }
        let (counts, sink, _) = run(&flow, &store, true);
        assert_eq!(counts.violations, u64::from(count == 0));
        assert_eq!(counts.satisfied, u64::from(count == 1));
        assert_eq!(counts.unknown, u64::from(count == 2));
        if count == 0 {
            assert_eq!(sink.errors()[0].code, ValidationCode::RelatedRecordMissing);
        } else {
            assert!(sink.errors().is_empty());
        }
    }
    let assertion = json!({"kind":"set_exists","join":"S_T","predicate":{"eq":{"ref":{"field":"SPARE"},"value":"OK"}}});
    for (parents, expected) in [
        (
            vec![b"00AA01NO".as_slice(), b"00AA02OK".as_slice()],
            "satisfied",
        ),
        (
            vec![b"00AA01NO".as_slice(), b"00AA02XX".as_slice()],
            "violated",
        ),
        (
            vec![b"00AA01  ".as_slice(), b"00AA02XX".as_slice()],
            "unknown",
        ),
    ] {
        let mut raw = sample("unknown", assertion.clone());
        raw["required_capabilities"]
            .as_array_mut()
            .unwrap()
            .push(json!("joins.sets.v1"));
        raw["traces"][0]["joins"][0]["cardinality"] = json!("zero_or_many");
        let flow = compile(raw).unwrap();
        assert_eq!(flow.traces()[1].projection_plan().field_indexes, [0, 1, 3]);
        let mut store = InMemoryContextStore::new();
        store.insert(project(&flow, "S", b"00AA01ZZ", 1)).unwrap();
        for (i, parent) in parents.iter().enumerate() {
            store
                .insert(project(&flow, "T", parent, i as u64 + 1))
                .unwrap();
        }
        let (counts, sink, events) = run(&flow, &store, true);
        assert_eq!(events.values[0].kind, expected);
        assert_eq!(counts.violations, u64::from(expected == "violated"));
        assert_eq!(sink.errors().len(), usize::from(expected == "violated"));
    }
}

fn consensus_sample() -> Value {
    let mut raw = sample("not_applicable", comparison());
    raw["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .extend([json!("joins.sets.v1"), json!("joins.consensus.v1")]);
    raw["traces"][0]["joins"][0]["cardinality"] = json!("zero_or_many");
    raw["traces"][0]["joins"][0]["reduction"] = json!({"kind":"consensus","fields":["DATA","SPARE"],"include_if":{"const":true},"on_conflict":"unknown","diagnostic":{"code":"SYNTHETIC_CONFLICT","message_template":"Synthetic tuple conflict"}});
    raw
}
#[test]
fn consensus_proves_the_whole_tuple_and_never_mixes_values_from_different_records() {
    for (parents, kind) in [
        (
            vec![b"00AA01ZZ".as_slice(), b"00AA01ZZ".as_slice()],
            "satisfied",
        ),
        (
            vec![b"00AA01ZZ".as_slice(), b"00AA01YY".as_slice()],
            "unknown",
        ),
        (
            vec![b"00AA01ZZ".as_slice(), b"00AA02ZZ".as_slice()],
            "unknown",
        ),
    ] {
        let flow = compile(consensus_sample()).unwrap();
        let mut store = InMemoryContextStore::new();
        store.insert(project(&flow, "S", b"00AA01ZZ", 1)).unwrap();
        for (i, parent) in parents.iter().enumerate() {
            store
                .insert(project(&flow, "T", parent, i as u64 + 1))
                .unwrap();
        }
        let (counts, sink, events) = run(&flow, &store, true);
        assert_eq!(events.values[0].kind, kind);
        assert!(sink.errors().is_empty());
        if kind == "unknown" {
            assert_eq!(counts.unknown, 1);
            assert_eq!(events.values[0].reason, Some(UnknownReason::AmbiguousJoin));
            assert_eq!(
                events.values[0].diagnostic.as_deref(),
                Some("SYNTHETIC_CONFLICT")
            );
        }
    }
}

#[test]
fn consensus_empty_selection_uses_policy_and_unknown_filter_is_not_false() {
    for (filter, kind) in [
        (json!({"const":false}), "not_applicable"),
        (
            json!({"eq":{"ref":{"field":"SPARE"},"value":"OK"}}),
            "unknown",
        ),
    ] {
        let mut raw = consensus_sample();
        raw["traces"][0]["joins"][0]["reduction"]["include_if"] = filter;
        let flow = compile(raw).unwrap();
        let mut store = InMemoryContextStore::new();
        store.insert(project(&flow, "S", b"00AA01ZZ", 1)).unwrap();
        store.insert(project(&flow, "T", b"00AA01  ", 1)).unwrap();
        store.insert(project(&flow, "T", b"00AA01OK", 2)).unwrap();
        let (_, sink, events) = run(&flow, &store, true);
        assert!(sink.errors().is_empty());
        assert_eq!(events.values[0].kind, kind);
        if kind == "unknown" {
            assert_eq!(
                events.values[0].reason,
                Some(UnknownReason::FieldUnavailable)
            );
        }
    }
}

#[test]
fn scalar_cardinality_violation_and_missing_target_value_are_not_not_applicable() {
    let flow = compile(sample("not_applicable", comparison())).unwrap();
    let mut store = InMemoryContextStore::new();
    store.insert(project(&flow, "S", b"00AA01ZZ", 1)).unwrap();
    store.insert(project(&flow, "T", b"00AA01ZZ", 1)).unwrap();
    store.insert(project(&flow, "T", b"00AA01ZZ", 2)).unwrap();
    let (counts, sink, events) = run(&flow, &store, true);
    assert_eq!(counts.unknown, 1);
    assert_eq!(counts.not_applicable, 0);
    assert!(sink.errors().is_empty());
    assert_eq!(events.values[0].reason, Some(UnknownReason::AmbiguousJoin));
    let mut store = InMemoryContextStore::new();
    store.insert(project(&flow, "S", b"00AA01ZZ", 1)).unwrap();
    store.insert(project(&flow, "T", b"00AA  ZZ", 1)).unwrap();
    let (counts, sink, events) = run(&flow, &store, true);
    assert_eq!(counts.unknown, 1);
    assert!(sink.errors().is_empty());
    assert_eq!(
        events.values[0].reason,
        Some(UnknownReason::FieldUnavailable)
    );
}

#[test]
fn compiler_rejects_underdeclared_sets_and_unbound_diagnostic_placeholders() {
    let raw = sample(
        "unknown",
        json!({"kind":"set_exists","join":"S_T","predicate":{"const":true}}),
    );
    assert!(
        compile(raw)
            .unwrap_err()
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::MissingCapability)
    );
    let mut raw = sample("violation", comparison());
    raw["traces"][0]["joins"][0]["on_missing_record"]["diagnostic"]["message_template"] =
        json!("Missing {field_id}");
    assert!(
        compile(raw)
            .unwrap_err()
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::UnsupportedConstruct)
    );
    let mut raw = consensus_sample();
    raw["traces"][0]["joins"][0]["reduction"]["fields"] = json!(["SPARE"]);
    assert!(
        compile(raw)
            .unwrap_err()
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::UnsupportedConstruct)
    );
}

#[test]
fn output_is_ordered_by_source_line_then_compiled_rule_and_local_rules_stay_local() {
    let mut raw = sample("violation", comparison());
    raw["traces"][0]["rules"] = json!([
        rule("R1", comparison()),
        rule(
            "LOCAL",
            json!({"kind":"field_present","field":{"field":"PK"}})
        ),
        rule("R2", comparison())
    ]);
    let flow = compile(raw).unwrap();
    assert_eq!(flow.traces()[0].local_rules().len(), 1);
    assert_eq!(flow.cross_record_plan().rules.len(), 2);
    let mut store = InMemoryContextStore::new();
    for line in [3, 1, 2] {
        store
            .insert(project(&flow, "S", b"00AA01ZZ", line))
            .unwrap();
    }
    let (counts, sink, events) = run(&flow, &store, true);
    assert_eq!(counts.violations, 6);
    assert_eq!(
        events
            .values
            .iter()
            .map(|e| (e.line, &*e.rule))
            .collect::<Vec<_>>(),
        [
            (1, "R1"),
            (1, "R2"),
            (2, "R1"),
            (2, "R2"),
            (3, "R1"),
            (3, "R2")
        ]
    );
    assert_eq!(
        sink.errors()
            .iter()
            .map(|e| e.line_number)
            .collect::<Vec<_>>(),
        [1, 1, 2, 2, 3, 3]
    );
    assert!(RecordValidator::new(&flow, &TraceId("S".into())).is_ok());
}

#[test]
fn report_and_observer_failures_are_technical_and_missing_source_is_not_an_empty_run() {
    struct BrokenWriter;
    impl io::Write for BrokenWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::StorageFull,
                "synthetic report",
            ))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let flow = compile(sample("violation", comparison())).unwrap();
    let mut store = InMemoryContextStore::new();
    store.insert(project(&flow, "S", b"00AA01ZZ", 1)).unwrap();
    let traces = [TraceId("S".into()), TraceId("T".into())];
    let parameters = BTreeMap::new();
    let mut events = Events::default();
    let error = validate_cross_record_rules(
        &flow,
        &store,
        CrossRecordInputs {
            available_traces: &traces,
            parameters: &parameters,
        },
        &mut JsonlSink::new(BrokenWriter),
        &mut events,
    )
    .unwrap_err();
    assert!(matches!(error, CrossRecordValidationError::Report(_)));
    assert!(error.source().is_some());
    assert!(events.values.is_empty());
    let mut events = Events {
        values: vec![],
        fail: true,
    };
    let mut sink = VecSink::default();
    let error = validate_cross_record_rules(
        &flow,
        &store,
        CrossRecordInputs {
            available_traces: &traces,
            parameters: &parameters,
        },
        &mut sink,
        &mut events,
    )
    .unwrap_err();
    assert!(matches!(error, CrossRecordValidationError::Observer(_)));
    assert!(error.source().unwrap().is::<io::Error>());
    assert_eq!(sink.errors().len(), 1);
    let error = validate_cross_record_rules(
        &flow,
        &InMemoryContextStore::new(),
        CrossRecordInputs {
            available_traces: &[],
            parameters: &parameters,
        },
        &mut VecSink::default(),
        &mut Events::default(),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        CrossRecordValidationError::Context(RelatedContextError::SourceInputUnavailable)
    ));
}

#[test]
fn self_join_pass_preserves_multiple_candidates_instead_of_selecting_the_current_or_first_row() {
    let mut raw = sample("not_applicable", comparison());
    raw["traces"][0]["joins"][0]["target_trace"] = json!("S");
    raw["traces"][0]["rules"][0]["assertions"][0]["right"]["ref"]["trace"] = json!("S");
    let flow = compile(raw).unwrap();
    let mut store = InMemoryContextStore::new();
    store.insert(project(&flow, "S", b"00AA01ZZ", 1)).unwrap();
    let (counts, sink, _) = run(&flow, &store, true);
    assert_eq!(counts.satisfied, 1);
    assert!(sink.errors().is_empty());
    store.insert(project(&flow, "S", b"00AA02ZZ", 2)).unwrap();
    let (counts, sink, events) = run(&flow, &store, true);
    assert_eq!(counts.unknown, 2);
    assert!(sink.errors().is_empty());
    assert!(
        events
            .values
            .iter()
            .all(|event| event.reason == Some(UnknownReason::AmbiguousJoin))
    );
}

#[test]
fn empty_available_source_still_requires_valid_explicit_run_bindings() {
    let mut raw = sample("unknown", comparison());
    raw["parameters"] = json!([{"type":"string","id":"org","allowed_values":["001"]}]);
    raw["traces"][0]["joins"][0]["namespace_parameters"] = json!(["org"]);
    let flow = compile(raw).unwrap();
    let store = InMemoryContextStore::new();
    let traces = [TraceId("S".into()), TraceId("T".into())];
    let mut sink = VecSink::default();
    let mut observer = Events::default();
    let empty = BTreeMap::new();
    let error = validate_cross_record_rules(
        &flow,
        &store,
        CrossRecordInputs {
            available_traces: &traces,
            parameters: &empty,
        },
        &mut sink,
        &mut observer,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        CrossRecordValidationError::Context(RelatedContextError::MissingNamespaceParameter)
    ));
    let valid = BTreeMap::from([("org".into(), JoinParameterValue::Text("001".into()))]);
    let counts = validate_cross_record_rules(
        &flow,
        &store,
        CrossRecordInputs {
            available_traces: &traces,
            parameters: &valid,
        },
        &mut sink,
        &mut observer,
    )
    .unwrap();
    assert_eq!(counts.evaluations, 0);
    let duplicate = [TraceId("S".into()), TraceId("S".into())];
    assert!(matches!(
        validate_cross_record_rules(
            &flow,
            &store,
            CrossRecordInputs {
                available_traces: &duplicate,
                parameters: &valid
            },
            &mut sink,
            &mut observer
        ),
        Err(CrossRecordValidationError::Context(
            RelatedContextError::DuplicateTraceInput
        ))
    ));
    let wrong = BTreeMap::from([("org".into(), JoinParameterValue::Text("1".into()))]);
    assert!(matches!(
        validate_cross_record_rules(
            &flow,
            &store,
            CrossRecordInputs {
                available_traces: &traces,
                parameters: &wrong
            },
            &mut sink,
            &mut observer
        ),
        Err(CrossRecordValidationError::Context(
            RelatedContextError::InvalidParameter
        ))
    ));
    assert!(observer.values.is_empty());
    assert!(sink.errors().is_empty());
}

#[test]
fn actual_local_only_service_reports_omitted_cross_evaluations_without_false_complete() {
    use std::{fs, sync::Arc};
    let root = tempfile::tempdir().unwrap();
    let input = root.path().join("synthetic.txt");
    fs::write(&input, b"00AA01ZZ\n01AA02YY\n").unwrap();
    let raw: RawFlowSchema = serde_json::from_value(sample("violation", comparison())).unwrap();
    let mut repository = InMemorySchemaRepository::new();
    repository.insert(raw).unwrap();
    let service = ValidationService::new(Arc::new(SchemaRegistry::new(
        Arc::new(repository),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    )));
    let request = ValidationRequest {
        schema_key: SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        scope: ValidationScope::Trace(TraceValidationInput {
            trace_id: TraceId("S".into()),
            source: InputSource::Path(input),
        }),
        mode: ValidationMode::LocalOnly,
        options: ValidationOptions {
            record_length_policy: RecordLengthPolicy::StrictSkipRecord,
            newline_policy: NewlinePolicy::AutoLfOrCrlf,
            max_errors: None,
        },
    };
    let mut sink = JsonlSink::new(Vec::new());
    let result = service
        .validate(
            request,
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    assert!(sink.finish().unwrap().is_empty());
    assert_eq!(result.summary.skipped_rules, 2);
    assert_eq!(result.summary.total_errors, 0);
    assert_eq!(result.summary.valid_records, 2);
    assert_eq!(result.summary.run_status, RunStatus::CompletedValid);
    assert_eq!(
        result.summary.completeness(),
        ValidationCompleteness::Partial
    );
    let summary: Value =
        serde_json::from_slice(&write_run_summary(Vec::new(), &result.summary).unwrap()).unwrap();
    assert_eq!(summary["skipped_rules"], 2);
    assert_eq!(summary["validation_mode"], "local_only");
}
