//! CAD08 executor qualification through private scoped compilation. This does
//! not construct a public historical approval or enable ordinary CAD Full.
use super::*;
use crate::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs};
const COMPONENT: &str =
    include_str!("../../schemas/cad/components/2024-06.inactivity.1/component.json");
const CODE: &str = "E_CAD_MISSING_ADMINISTRATIVE_CLOSURE";
fn filler(start: usize, length: usize) -> Value {
    json!({"id":format!("SYNTHETIC-{start}"),"name":"Unvalidated synthetic filler",
        "physical":{"type":"alphanumeric","start":start,"length":length,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}},
        "missing_value":{"kind":"spaces"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[]})
}
fn model() -> Value {
    let c: Value = serde_json::from_str(COMPONENT).unwrap();
    let mut v: Value = serde_json::from_slice(include_bytes!(
        "../../tests/golden/synthetic/v1/schema.json"
    ))
    .unwrap();
    v["schema_format_version"] = c["schema_format_version"].clone();
    v["flow_id"] = json!("CAD-INACTIVITY-COMPONENT-TEST");
    v["schema_version"] = c["component_version"].clone();
    v["required_capabilities"] = c["required_capabilities"].clone();
    v["parameters"] = c["parameters"].clone();
    v["persistence_policy"] = c["persistence_policy"].clone();
    let template = v["traces"][0].clone();
    v["traces"] = json!([]);
    for id in ["A", "B1", "B2", "C", "D"] {
        let d = &c["traces"][id];
        let mut t = template.clone();
        t["id"] = json!(id);
        for key in [
            "record_length",
            "record_key",
            "joins",
            "rules",
            "named_conditions",
            "derived_values",
        ] {
            t[key] = d[key].clone();
        }
        let mut fields = vec![];
        let mut next = 1;
        for f in d["fields"].as_array().unwrap() {
            let start = f["physical"]["start"].as_u64().unwrap() as usize;
            assert!(start >= next);
            if start > next {
                fields.push(filler(next, start - next));
            }
            next = start + f["physical"]["length"].as_u64().unwrap() as usize;
            fields.push(f.clone());
        }
        let length = d["record_length"].as_u64().unwrap() as usize;
        if next <= length {
            fields.push(filler(next, length + 1 - next));
        }
        t["fields"] = json!(fields);
        v["traces"].as_array_mut().unwrap().push(t);
    }
    v
}
fn raw(v: Value) -> RawFlowSchema {
    let mut r: RawFlowSchema = serde_json::from_value(v).unwrap();
    r.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    r
}
fn compile(v: Value) -> Result<CompiledFlowSchema, SchemaCompilationError> {
    let raw = raw(v);
    let profile = SchemaCompiler.compile_persistence_profile(&raw, &[])?;
    SchemaCompiler.compile_scoped(raw, profile.policy().cloned())
}
fn set(row: &mut [u8], start: usize, value: &str) {
    row[start - 1..start - 1 + value.len()].copy_from_slice(value.as_bytes());
}
fn a() -> Vec<u8> {
    let mut row = vec![b' '; 218];
    set(&mut row, 1, "501");
    set(&mut row, 206, "000000000001");
    row
}
fn evaluation(start: &str, end: &str, code: &str, seq: u64) -> Vec<u8> {
    let mut row = vec![b' '; 198];
    set(&mut row, 1, "000000000001");
    row[14] = b'1';
    set(&mut row, 16, start);
    set(&mut row, 25, end);
    set(&mut row, 33, if code.is_empty() { "00" } else { code });
    set(&mut row, 187, &format!("{seq:012}"));
    row
}
fn pause(start: &str, end: &str, seq: u64) -> Vec<u8> {
    let mut row = vec![b' '; 41];
    set(&mut row, 1, "000000000001");
    set(&mut row, 13, start);
    set(&mut row, 21, end);
    row[28] = b'1';
    set(&mut row, 30, &format!("{seq:012}"));
    row
}
fn access(date: &str, seq: u64) -> Vec<u8> {
    let mut row = vec![b' '; 45];
    set(&mut row, 1, "000000000001");
    set(&mut row, 13, date);
    row[21] = b'1';
    set(&mut row, 23, "01");
    row[25] = b'1';
    set(&mut row, 34, &format!("{seq:012}"));
    row
}
fn provision(date: &str, seq: u64) -> Vec<u8> {
    let mut row = vec![b' '; 39];
    set(&mut row, 1, "000000000001");
    set(&mut row, 13, date);
    row[20] = b'1';
    set(&mut row, 22, "01");
    set(&mut row, 26, "01");
    set(&mut row, 28, &format!("{seq:012}"));
    row
}
#[derive(Default)]
struct Observed(Vec<String>);
impl CrossRecordEvaluationObserver for Observed {
    fn on_evaluation(
        &mut self,
        _: &TraceId,
        _: u64,
        rule: &str,
        outcome: &CrossRuleOutcome<'_>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if rule.starts_with("temporal.inactivity_with_pauses.") {
            self.0.push(
                match outcome {
                    CrossRuleOutcome::Satisfied => "satisfied",
                    CrossRuleOutcome::Violated { .. } => "violation",
                    CrossRuleOutcome::Unknown { .. } => "unknown",
                    CrossRuleOutcome::NotApplicable { .. } => "not_applicable",
                }
                .into(),
            );
        }
        Ok(())
    }
}
struct Outcome {
    sink: VecSink,
    observed: Observed,
    artifacts: crate::engine::flow_execution::FlowRunArtifacts,
}
fn parameters(year: u32, quarter: u32) -> BTreeMap<String, JoinParameterValue> {
    [
        (
            "sending_ulss".into(),
            JoinParameterValue::Text("501".into()),
        ),
        (
            "year".into(),
            JoinParameterValue::Integer(year.to_string().parse().unwrap()),
        ),
        (
            "quarter".into(),
            JoinParameterValue::Integer(quarter.to_string().parse().unwrap()),
        ),
    ]
    .into()
}
fn execute(
    schema: &CompiledFlowSchema,
    rows: &[Option<Vec<Vec<u8>>>],
    parameters: &BTreeMap<String, JoinParameterValue>,
    cancelled: bool,
) -> Result<Outcome, FlowValidationError> {
    execute_budgeted(schema, rows, parameters, cancelled, None)
}
fn execute_budgeted(
    schema: &CompiledFlowSchema,
    rows: &[Option<Vec<Vec<u8>>>],
    parameters: &BTreeMap<String, JoinParameterValue>,
    cancelled: bool,
    budget: Option<std::num::NonZeroU64>,
) -> Result<Outcome, FlowValidationError> {
    let dir = tempfile::tempdir().unwrap();
    let mut paths = vec![];
    for (i, rows) in rows.iter().enumerate() {
        if let Some(rows) = rows {
            let path = dir.path().join(format!("trace-{i}.dat"));
            let mut data = vec![];
            for row in rows {
                data.extend(row);
                data.push(b'\n');
            }
            fs::write(&path, data).unwrap();
            paths.push((i, path));
        }
    }
    let inputs = paths
        .iter()
        .map(|(i, path)| FlowTraceInput {
            trace_id: schema.traces()[*i].id(),
            path,
        })
        .collect::<Vec<_>>();
    let mut sink = VecSink::default();
    let audits = InputAuditSession::new();
    let mut summary = ValidationSummaryBuilder::new(
        schema.key().clone(),
        ScopeSummary::Flow {
            trace_ids: inputs.iter().map(|t| t.trace_id.clone()).collect(),
        },
    )
    .with_error_limit(budget);
    let mut observed = Observed::default();
    let artifacts = FlowValidator::new(schema, NewlinePolicy::AutoLfOrCrlf)
        .unwrap()
        .validate_with_record_observer(
            &inputs,
            parameters,
            FlowExecutionPorts {
                sink: &mut sink,
                summary: &mut summary,
                audits: &audits,
                control: &mut FlowControl {
                    is_cancelled: &|| cancelled,
                    on_event: &mut |_| {},
                },
                reference_observers: None,
                cross_observer: Some(&mut observed),
            },
            None,
        )?;
    Ok(Outcome {
        sink,
        observed,
        artifacts,
    })
}
fn rows(start: &str) -> Vec<Option<Vec<Vec<u8>>>> {
    vec![
        Some(vec![a()]),
        Some(vec![evaluation(start, "", "", 1)]),
        Some(vec![]),
        Some(vec![]),
        Some(vec![]),
    ]
}
fn run(rows: &[Option<Vec<Vec<u8>>>], year: u32, quarter: u32) -> Outcome {
    execute(
        &compile(model()).unwrap(),
        rows,
        &parameters(year, quarter),
        false,
    )
    .unwrap()
}
fn expect(rows: &[Option<Vec<Vec<u8>>>], year: u32, quarter: u32, expected: &str) -> Outcome {
    let out = run(rows, year, quarter);
    assert_eq!(out.observed.0, [expected]);
    let count = out
        .sink
        .errors()
        .iter()
        .filter(|e| e.code == CODE.parse().unwrap())
        .count();
    assert_eq!(count, usize::from(expected == "violation"));
    // No accidental local error can masquerade as a temporal success.
    assert_eq!(out.sink.errors().len(), count, "{:?}", out.sink.errors());
    out
}
#[test]
fn inactivity_profile_compiles_period_but_public_executable_entry_remains_closed() {
    let r = raw(model());
    let profile = SchemaCompiler.compile_persistence_profile(&r, &[]).unwrap();
    let TemporalPlan::InactivityWithPauses(plan) = &profile.policy().unwrap().temporal_rules[0]
    else {
        panic!()
    };
    assert_eq!((plan.year_parameter, plan.quarter_parameter), (1, 2));
    assert!(SchemaCompiler.compile(r.clone()).is_err());
    assert!(SchemaCompiler.compile_historical_audit(r, &[]).is_err());
    assert!(compile(model()).is_ok());
}
#[test]
fn inactivity_no_activity_uses_event_start_at_the_session_quarter_end() {
    expect(&rows("31122025"), 2026, 1, "violation");
    expect(&rows("01012026"), 2026, 1, "satisfied");
    expect(&rows("31012026"), 2026, 2, "violation");
    expect(&rows("01072026"), 2026, 2, "satisfied");
}
#[test]
fn inactivity_serviced_access_uses_four_fields_and_not_independent_progressives() {
    let mut r = rows("01102025");
    r[3] = Some(vec![access("01032026", 7)]);
    r[4] = Some(vec![provision("01032026", 999)]);
    expect(&r, 2026, 1, "satisfied");
    // A C alone is not activity, even with a late timestamp.
    r[4] = Some(vec![]);
    expect(&r, 2026, 1, "violation");
    let mut d = provision("01032026", 999);
    set(&mut d, 22, "02");
    r[4] = Some(vec![d]);
    expect(&r, 2026, 1, "violation");
    r[4] = Some(vec![provision("02032026", 999)]);
    expect(&r, 2026, 1, "violation");
}
#[test]
fn inactivity_latest_matched_access_wins_independently_of_input_order() {
    let mut r = rows("01102025");
    r[3] = Some(vec![
        access("01012026", 7),
        access("01022026", 8),
        access("01032026", 9),
    ]);
    r[4] = Some(vec![provision("01022026", 111), provision("01012026", 222)]);
    let first = expect(&r, 2026, 1, "satisfied");
    r[3].as_mut().unwrap().reverse();
    r[4].as_mut().unwrap().reverse();
    let second = expect(&r, 2026, 1, "satisfied");
    assert_eq!(first.observed.0, second.observed.0);
    // Latest unmatched March record does not reset inactivity at June end.
    expect(&r, 2026, 2, "violation");
}
#[test]
fn inactivity_missing_input_is_unknown_while_present_zero_means_no_activity() {
    let r = rows("01102025");
    expect(&r, 2026, 1, "violation");
    for index in 1..5 {
        let mut absent = r.clone();
        absent[index] = None;
        expect(&absent, 2026, 1, "unknown");
    }
    let mut empty = r;
    empty[1] = Some(vec![]);
    expect(&empty, 2026, 1, "unknown");
}
#[test]
fn inactivity_pauses_union_without_reset_and_open_end_covers_the_horizon() {
    let mut r = rows("31122025");
    r[2] = Some(vec![pause("01012026", "04012026", 1)]);
    expect(&r, 2026, 1, "satisfied");
    r[2] = Some(vec![pause("01042026", "", 1)]);
    expect(&r, 2026, 1, "violation");
    r[2] = Some(vec![pause("01012026", "", 1)]);
    expect(&r, 2026, 2, "satisfied");
    r[2] = Some(vec![
        pause("01012026", "05012026", 1),
        pause("02012026", "04012026", 2),
    ]);
    expect(&r, 2026, 1, "satisfied");
}
#[test]
fn inactivity_legitimate_conclusions_do_not_require_a_second_closure() {
    for code in ["01", "03", "12", "82"] {
        let mut r = rows("01102025");
        r[1] = Some(vec![evaluation("01102025", "01012026", code, 1)]);
        expect(&r, 2026, 1, "satisfied");
    }
    let mut r = rows("01102025");
    r[1] = Some(vec![evaluation("01102025", "01042026", "12", 1)]);
    expect(&r, 2026, 1, "violation");
}
#[test]
fn inactivity_equal_evaluations_consensus_and_conflicting_start_or_closure_unknown() {
    let mut r = rows("01102025");
    r[1] = Some(vec![
        evaluation("01102025", "", "", 1),
        evaluation("01102025", "", "", 2),
    ]);
    expect(&r, 2026, 1, "violation");
    r[1] = Some(vec![
        evaluation("01102025", "", "", 1),
        evaluation("02102025", "", "", 2),
    ]);
    expect(&r, 2026, 1, "unknown");
    r[1] = Some(vec![
        evaluation("01102025", "", "", 1),
        evaluation("01102025", "01012026", "12", 2),
    ]);
    expect(&r, 2026, 1, "unknown");
}
#[test]
fn inactivity_preflight_rejects_missing_period_even_with_empty_inputs_and_cancelled() {
    let s = compile(model()).unwrap();
    for id in ["year", "quarter"] {
        let mut p = parameters(2026, 1);
        p.remove(id);
        assert!(matches!(
            execute(&s, &[], &p, true),
            Err(FlowValidationError::Inputs(
                RelatedContextError::InvalidParameter
            ))
        ));
    }
    for (year, quarter) in [(0, 1), (2026, 0), (2026, 5), (10000, 1)] {
        assert!(execute(&s, &[], &parameters(year, quarter), true).is_err());
    }
}
#[test]
fn inactivity_cancellation_never_emits_a_false_temporal_success() {
    let out = execute(
        &compile(model()).unwrap(),
        &rows("01102025"),
        &parameters(2026, 1),
        true,
    )
    .unwrap();
    assert_eq!(
        out.artifacts.completion,
        crate::domain::ExecutionStop::Cancelled
    );
    assert!(out.observed.0.is_empty());
    assert!(out.sink.errors().is_empty());
}
#[test]
fn inactivity_compiler_rejects_ambiguous_contracts_instead_of_selecting_first() {
    for change in 0..9 {
        let mut v = model();
        let p = &mut v["persistence_policy"];
        match change {
            0 => p["snapshot"] = Value::Null,
            1 => p["temporal_rules"][0]["threshold"]["calendar_months"] = json!(0),
            2 => p["temporal_rules"][0]["provision_fields"] = json!(["5.1"]),
            3 => v["traces"][0]["joins"][0]["reduction"]["fields"] = json!(["2.6", "2.7"]),
            4 => {
                v["traces"][0]["joins"][0]["reduction"]["include_if"] =
                    json!({"is_present":{"ref":{"field":"2.6"}}})
            }
            5 => v["traces"][0]["joins"][1]["cardinality"] = json!("zero_or_one"),
            6 => p["temporal_rules"][0]["access_fields"] = json!(["4.1", "4.1", "4.4", "4.5"]),
            7 => p["temporal_rules"][0]["access_date_field"] = json!("4.11"),
            8 => v["traces"][0]["joins"][0]["cardinality"] = json!("zero_or_one"),
            _ => unreachable!(),
        }
        assert!(compile(v).is_err(), "case {change}");
    }
}

#[test]
fn inactivity_overlap_union_and_pause_extension_crossing_the_original_deadline() {
    let mut r = rows("30112025");
    r[2] = Some(vec![
        pause("01012026", "21012026", 1),
        pause("11012026", "31012026", 2),
    ]);
    // 29 union days, not 38: February 28 + 29 = March 29, due at Q1 end.
    expect(&r, 2026, 1, "violation");
    r = rows("01122025");
    r[2] = Some(vec![
        pause("25022026", "10032026", 1),
        pause("10032026", "01042026", 2),
    ]);
    // The first pause extends March 1 beyond the start of the second pause.
    expect(&r, 2026, 1, "satisfied");
    r = rows("31122025");
    r[2] = Some(vec![pause("01032026", "01042026", 1)]);
    // A pause extends the original threshold; it does not reset to 1 April +3m.
    expect(&r, 2026, 2, "violation");
}
#[test]
fn inactivity_invalid_partial_evidence_cannot_prove_a_clear_result() {
    for index in 1..5 {
        let mut r = rows("01102025");
        r[index] = Some(vec![match index {
            1 => evaluation("32132025", "", "", 1),
            2 => pause("32132026", "", 1),
            3 => access("32132026", 1),
            4 => provision("32132026", 1),
            _ => unreachable!(),
        }]);
        let out = run(&r, 2026, 1);
        assert_eq!(
            out.observed.0,
            ["unknown"],
            "trace {index}: {:?}",
            out.sink.errors()
        );
        assert!(!out.sink.errors().is_empty());
        assert!(
            !out.sink
                .errors()
                .iter()
                .any(|e| e.code == CODE.parse().unwrap())
        );
    }
    let mut r = rows("01102025");
    r[2] = Some(vec![pause("10012026", "09012026", 1)]);
    expect(&r, 2026, 1, "unknown");
}
#[test]
fn inactivity_parameter_and_trace_names_have_no_built_in_meaning() {
    // Rename schema labels through JSON token values; numeric field IDs are
    // unchanged here because they are already compiler-resolved indexes.
    let mut text = serde_json::to_string(&model()).unwrap();
    for (from, to) in [
        ("sending_ulss", "organization"),
        ("year", "report_year"),
        ("quarter", "report_quarter"),
        ("A", "ROOT"),
        ("B1", "EVAL"),
        ("B2", "PAUSE"),
        ("C", "ACTIVITY"),
        ("D", "PROVISION"),
    ] {
        text = text.replace(&format!("\"{from}\""), &format!("\"{to}\""));
    }
    let schema = compile(serde_json::from_str(&text).unwrap()).unwrap();
    let p = parameters(2026, 1)
        .into_iter()
        .map(|(name, value)| {
            (
                match name.as_str() {
                    "sending_ulss" => "organization",
                    "year" => "report_year",
                    "quarter" => "report_quarter",
                    _ => unreachable!(),
                }
                .into(),
                value,
            )
        })
        .collect();
    let out = execute(&schema, &rows("01102025"), &p, false).unwrap();
    assert_eq!(out.observed.0, ["violation"]);
    assert_eq!(out.sink.errors().len(), 1);
}

#[test]
fn inactivity_local_only_context_never_requires_the_cross_record_period() {
    let schema = compile(model()).unwrap();
    assert!(VerifiedRunContext::new(&schema, BTreeMap::new(), ValidationMode::LocalOnly).is_ok());
    assert!(VerifiedRunContext::new(&schema, BTreeMap::new(), ValidationMode::Full).is_err());
}

#[test]
fn inactivity_all_quarter_boundaries_and_calendar_domain_extremes() {
    for (start, year, quarter, verdict) in [
        ("30062026", 2026, 3, "violation"),
        ("01072026", 2026, 3, "satisfied"),
        ("30092026", 2026, 4, "violation"),
        ("01102026", 2026, 4, "satisfied"),
        ("01010001", 1, 1, "satisfied"),
        ("01010001", 1, 2, "violation"),
        ("01109999", 9999, 4, "satisfied"),
    ] {
        expect(&rows(start), year, quarter, verdict);
    }
}

#[test]
fn inactivity_error_budget_stops_the_shared_second_pass_without_losing_determinism() {
    let schema = compile(model()).unwrap();
    let mut r = rows("01102025");
    let mut second = a();
    set(&mut second, 206, "000000000002");
    r[0].as_mut().unwrap().push(second);
    let mut second = evaluation("01102025", "", "", 1);
    set(&mut second, 1, "000000000002");
    r[1].as_mut().unwrap().push(second);
    let unlimited = execute(&schema, &r, &parameters(2026, 1), false).unwrap();
    assert_eq!(unlimited.observed.0, ["violation", "violation"]);
    let limited = execute_budgeted(
        &schema,
        &r,
        &parameters(2026, 1),
        false,
        std::num::NonZeroU64::new(1),
    )
    .unwrap();
    assert_eq!(
        limited.artifacts.completion,
        crate::domain::ExecutionStop::ErrorLimit
    );
    assert_eq!(limited.observed.0, ["violation"]);
    assert_eq!(limited.sink.errors().len(), 1);
    let repeated = execute(&schema, &r, &parameters(2026, 1), false).unwrap();
    let jsonl = |errors: &[ValidationError]| {
        let mut sink = JsonlSink::new(Vec::new());
        for error in errors {
            sink.on_error(error).unwrap();
        }
        sink.finish().unwrap()
    };
    assert_eq!(
        jsonl(unlimited.sink.errors()),
        jsonl(repeated.sink.errors())
    );
}
