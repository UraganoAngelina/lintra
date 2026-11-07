//! Synthetic CAD06 component executed through the actual two-pass service.
use lintra::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, sync::Arc};
const COMPONENT: &str =
    include_str!("../schemas/cad/components/2024-06.suspensions.1/component.json");
const CODE: &str = "E_CAD_HOME_ACCESS_DURING_SUSPENSION";
fn component() -> Value {
    serde_json::from_str(COMPONENT).unwrap()
}
fn filler(start: usize, length: usize) -> Value {
    json!({"id":format!("SYNTHETIC-{start}"),"name":"Unvalidated synthetic filler",
        "physical":{"type":"alphanumeric","start":start,"length":length,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}},
        "missing_value":{"kind":"spaces"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[]})
}
/// Individual rule groups are explicitly scoped test schemas, useful for a
/// missing target that would itself own other rules in the combined component.
/// This never removes an obligation while claiming complete CAD validation.
fn model() -> Value {
    let c = component();
    let mut v: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    v["flow_id"] = json!("CAD-SUSPENSION-COMPONENT-TEST");
    v["schema_version"] = c["component_version"].clone();
    v["regulatory_version"] = c["source"]["regulatory_version"].clone();
    v["schema_format_version"] = c["schema_format_version"].clone();
    v["source"] = json!({"scope":"suspension component only; not full CAD compliance"});
    v["required_capabilities"] = c["required_capabilities"].clone();
    v["parameters"] = c["parameters"].clone();
    v["persistence_policy"] = c["persistence_policy"].clone();
    let template = v["traces"][0].clone();
    v["traces"] = json!([]);
    for id in ["A", "B2", "C"] {
        let d = &c["traces"][id];
        let mut t = template.clone();
        t["id"] = json!(id);
        t["record_length"] = d["record_length"].clone();
        t["record_key"] = d["record_key"].clone();
        t["identity"] = json!([]);
        t["joins"] = d["joins"].clone();
        t["derived_values"] = d["derived_values"].clone();
        t["rules"] = d["rules"].clone();
        let mut fields = vec![];
        let mut next = 1;
        for field in d["fields"].as_array().unwrap() {
            let start = field["physical"]["start"].as_u64().unwrap() as usize;
            assert!(start >= next);
            if start > next {
                fields.push(filler(next, start - next));
            }
            next = start + field["physical"]["length"].as_u64().unwrap() as usize;
            fields.push(field.clone());
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
fn compile(v: Value) -> Result<CompiledFlowSchema, SchemaCompilationError> {
    let mut r: RawFlowSchema = serde_json::from_value(v).unwrap();
    r.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(r)
}
fn set(r: &mut [u8], start: usize, v: &str) {
    r[start - 1..start - 1 + v.len()].copy_from_slice(v.as_bytes());
}
fn a() -> Vec<u8> {
    let mut r = vec![b' '; 218];
    set(&mut r, 1, "501");
    set(&mut r, 206, "000000000001");
    r
}
fn suspension(start: &str, end: &str, reason: u8, event: u64, seq: u64) -> Vec<u8> {
    let mut r = vec![b' '; 41];
    set(&mut r, 1, &format!("{event:012}"));
    set(&mut r, 13, start);
    set(&mut r, 21, end);
    r[28] = reason;
    set(&mut r, 30, &format!("{seq:012}"));
    r
}
fn access(date: &str, site: u8, seq: u64) -> Vec<u8> {
    let mut r = vec![b' '; 45];
    set(&mut r, 1, "000000000001");
    set(&mut r, 13, date);
    r[21] = b'1';
    set(&mut r, 23, "01");
    r[25] = site;
    set(&mut r, 34, &format!("{seq:012}"));
    r
}
struct Fixture {
    dir: tempfile::TempDir,
    max_errors: Option<u64>,
    cancellation: CancellationToken,
}
impl Fixture {
    fn new() -> Self {
        Self {
            max_errors: None,
            cancellation: CancellationToken::default(),
            dir: tempfile::tempdir_in(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target"),
            )
            .unwrap(),
        }
    }
    fn run(
        &self,
        v: Value,
        arows: Option<&[Vec<u8>]>,
        brows: Option<&[Vec<u8>]>,
        crows: Option<&[Vec<u8>]>,
        mode: ValidationMode,
    ) -> Result<(ValidationRunOutcome, VecSink), ValidationRunError> {
        let mut inputs = vec![];
        for (index, rows) in [arows, brows, crows].into_iter().enumerate() {
            if let Some(rows) = rows {
                let path = self.dir.path().join(format!("trace-{index}.dat"));
                let mut bytes = vec![];
                for row in rows {
                    bytes.extend_from_slice(row);
                    bytes.push(b'\n');
                }
                fs::write(&path, bytes).unwrap();
                inputs.push(TraceValidationInput {
                    trace_id: TraceId(v["traces"][index]["id"].as_str().unwrap().into()),
                    source: InputSource::Path(path),
                });
            }
        }
        let key = SchemaKey {
            flow_id: FlowId(v["flow_id"].as_str().unwrap().into()),
            version: SchemaVersion(v["schema_version"].as_str().unwrap().into()),
        };
        let has_unit = v["parameters"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["id"] == "sending_ulss");
        let mut repo = InMemorySchemaRepository::new();
        repo.insert(serde_json::from_value(v).unwrap()).unwrap();
        let svc = ValidationService::new(Arc::new(SchemaRegistry::new(
            Arc::new(repo),
            SchemaCompiler,
            RawResolvedEncoding::Utf8,
        )));
        let scope = if mode == ValidationMode::Full {
            ValidationScope::Flow(FlowValidationInput { traces: inputs })
        } else {
            ValidationScope::Trace(inputs.pop().unwrap())
        };
        let request = ValidationRequest {
            schema_key: key,
            scope,
            mode,
            options: ValidationOptions {
                record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                newline_policy: NewlinePolicy::AutoLfOrCrlf,
                max_errors: self.max_errors,
            },
        };
        let params: BTreeMap<_, _> = if has_unit {
            [(
                "sending_ulss".into(),
                JoinParameterValue::Text("501".into()),
            )]
            .into()
        } else {
            BTreeMap::new()
        };
        let mut sink = VecSink::default();
        let out = svc.validate_with_typed_parameters(
            request,
            params,
            &mut sink,
            &mut |_| {},
            &self.cancellation,
        )?;
        write_run_summary(Vec::new(), &out.summary).unwrap();
        Ok((out, sink))
    }
}
fn run(b: Option<&[Vec<u8>]>, c: &[Vec<u8>]) -> (ValidationRunOutcome, VecSink) {
    Fixture::new()
        .run(model(), Some(&[a()]), b, Some(c), ValidationMode::Full)
        .unwrap()
}
fn errors(s: &VecSink) -> usize {
    s.errors()
        .iter()
        .filter(|e| e.code == CODE.parse().unwrap())
        .count()
}
fn control(out: &ValidationRunOutcome) -> &CrossControlSummary {
    out.summary
        .full_validation
        .as_ref()
        .unwrap()
        .controls
        .iter()
        .find(|c| c.rule_id == "temporal.interval_exclusion.0")
        .unwrap()
}
#[test]
fn all_six_reasons_exclude_only_strict_interior_home_access() {
    for reason in *b"123479" {
        let b = [suspension("10012026", "20012026", reason, 1, 1)];
        let c: Vec<_> = [
            "09012026", "10012026", "11012026", "19012026", "20012026", "21012026",
        ]
        .iter()
        .enumerate()
        .map(|(i, d)| access(d, b'1', i as u64 + 1))
        .collect();
        let (out, s) = run(Some(&b), &c);
        assert_eq!(errors(&s), 2, "reason {reason}");
        assert_eq!(
            (
                control(&out).evaluations,
                control(&out).violations,
                control(&out).satisfied
            ),
            (6, 2, 4)
        );
        for site in *b"23456" {
            let (_, s) = run(Some(&b), &[access("15012026", site, 1)]);
            assert_eq!(errors(&s), 0);
        }
    }
}
#[test]
fn open_end_overlaps_and_other_events_use_entire_related_set() {
    let b = [
        suspension("01012026", "05012026", b'1', 1, 1),
        suspension("10012026", "20012026", b'9', 1, 2),
        suspension("15012026", "        ", b'7', 1, 3),
        suspension("01012026", "31122026", b'2', 2, 4),
    ];
    let c: Vec<_> = ["09012026", "10012026", "15012026", "20012026", "31122026"]
        .iter()
        .enumerate()
        .map(|(i, d)| access(d, b'1', i as u64 + 1))
        .collect();
    let (out, s) = run(Some(&b), &c);
    assert_eq!(errors(&s), 3);
    assert_eq!(control(&out).satisfied, 2);
    let (_, s) = run(
        Some(&[suspension("10012026", "        ", b'1', 1, 1)]),
        &[access("10012026", b'1', 1)],
    );
    assert_eq!(errors(&s), 0);
}
#[test]
fn absent_input_is_unknown_but_present_empty_or_unmatched_is_proven_clear() {
    let (out, s) = run(None, &[access("15012026", b'1', 1)]);
    assert_eq!(errors(&s), 0);
    assert_eq!(control(&out).unknown, 1);
    assert!(control(&out).causes.iter().any(|c| matches!(
        c.result,
        CrossCauseKind::Unknown(CrossUncertaintyReason::TraceInputUnavailable)
    )));
    for b in [vec![], vec![suspension("01012026", "31122026", b'1', 2, 1)]] {
        let (out, s) = run(Some(&b), &[access("15012026", b'1', 1)]);
        assert_eq!(errors(&s), 0);
        assert_eq!(control(&out).satisfied, 1);
    }
}
#[test]
fn invalid_dates_are_unknown_and_valid_counterexample_dominates_uncertainty() {
    for (start, end) in [
        ("32132026", "20012026"),
        ("10012026", "32132026"),
        ("20012026", "10012026"),
    ] {
        let (out, s) = run(
            Some(&[suspension(start, end, b'1', 1, 1)]),
            &[access("15012026", b'1', 1)],
        );
        assert_eq!(errors(&s), 0);
        assert_eq!(control(&out).unknown, 1);
        let (out, s) = run(
            Some(&[
                suspension(start, end, b'1', 1, 1),
                suspension("10012026", "20012026", b'2', 1, 2),
            ]),
            &[access("15012026", b'1', 1)],
        );
        assert_eq!(errors(&s), 1);
        assert_eq!(control(&out).violations, 1);
    }
    let (out, s) = run(
        Some(&[suspension("10012026", "20012026", b'1', 1, 1)]),
        &[access("32132026", b'1', 1)],
    );
    assert_eq!(errors(&s), 0);
    assert_eq!(control(&out).unknown, 1);
}
#[test]
fn compiler_rejects_invalid_temporal_references_types_cardinality_and_policy() {
    compile(model()).unwrap();
    for mutation in 0..7 {
        let mut v = model();
        match mutation {
            0 => v["persistence_policy"]["temporal_rules"][0]["point_field"] = json!("missing"),
            1 => v["persistence_policy"]["temporal_rules"][0]["point_field"] = json!("4.1"),
            2 => v["traces"][2]["joins"][0]["cardinality"] = json!("zero_or_one"),
            3 => v["persistence_policy"]["temporal_rules"][0]["interval_join"] = json!("missing"),
            4 => v["persistence_policy"]["temporal_rules"][0]["owner_trace"] = json!("missing"),
            5 => {
                v["persistence_policy"]["temporal_rules"][0]["on_missing_trace"]["diagnostic"]["code"] =
                    json!("different")
            }
            _ => {
                v["required_capabilities"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|c| c != "temporal.intervals.v1");
            }
        };
        assert!(compile(v).is_err(), "mutation {mutation}");
    }
}

#[test]
fn partial_target_cannot_prove_clear_but_a_valid_witness_still_proves_violation() {
    let short = b"unprojectable".to_vec();
    let (out, s) = run(
        Some(std::slice::from_ref(&short)),
        &[access("15012026", b'1', 1)],
    );
    assert_eq!(errors(&s), 0);
    assert_eq!(control(&out).unknown, 1);
    assert!(control(&out).causes.iter().any(|c| matches!(
        c.result,
        CrossCauseKind::Unknown(CrossUncertaintyReason::IncompleteRelatedContext)
    )));
    let (out, s) = run(
        Some(&[short, suspension("10012026", "20012026", b'1', 1, 2)]),
        &[access("15012026", b'1', 1)],
    );
    assert_eq!(errors(&s), 1);
    assert_eq!(control(&out).violations, 1);
    let (out, s) = run(None, &[access("15012026", b'6', 1)]);
    assert_eq!(errors(&s), 0);
    assert_eq!(control(&out).satisfied, 1);
}
#[test]
fn original_source_order_and_deterministic_jsonl_are_preserved() {
    let b = [suspension("10012026", "20012026", b'1', 1, 1)];
    let rows = [
        access("15012026", b'1', 1),
        access("20012026", b'1', 2),
        access("16012026", b'1', 3),
    ];
    let encode = |sink: &VecSink| {
        let mut jsonl = JsonlSink::new(Vec::new());
        for e in sink.errors() {
            jsonl.on_error(e).unwrap();
        }
        jsonl.finish().unwrap()
    };
    let (_, first) = run(Some(&b), &rows);
    let (_, second) = run(Some(&b), &rows);
    assert_eq!(
        first
            .errors()
            .iter()
            .map(|e| e.line_number)
            .collect::<Vec<_>>(),
        [1, 3]
    );
    assert!(
        first
            .errors()
            .iter()
            .all(|e| e.trace_id == TraceId("C".into()) && e.actual.is_none() && e.field.is_none())
    );
    assert_eq!(encode(&first), encode(&second));
}
#[test]
fn generic_renaming_uses_declarations_instead_of_cad_identifiers() {
    let mut text = serde_json::to_string(&model()).unwrap();
    for (old, new) in [
        ("A", "People"),
        ("B2", "Intervals"),
        ("C", "Points"),
        ("1.1", "organization"),
        ("1.23", "person_event"),
        ("3.1", "interval_event"),
        ("3.2", "begin"),
        ("3.3", "end"),
        ("3.4", "reason"),
        ("3.5", "interval_id"),
        ("4.1", "point_event"),
        ("4.2", "point_date"),
        ("4.4", "type"),
        ("4.5", "actor"),
        ("4.7", "location"),
        ("4.11", "point_id"),
        ("C_TO_B2_INTERVALS", "POINT_INTERVALS"),
        (CODE, "E_GENERIC_INTERVAL_EXCLUDED"),
    ] {
        text = text.replace(&format!("\"{old}\""), &format!("\"{new}\""));
    }
    let v: Value = serde_json::from_str(&text).unwrap();
    let (_, s) = Fixture::new()
        .run(
            v,
            Some(&[a()]),
            Some(&[suspension("10012026", "20012026", b'1', 1, 1)]),
            Some(&[access("15012026", b'1', 1)]),
            ValidationMode::Full,
        )
        .unwrap();
    assert_eq!(s.errors().len(), 1);
    assert_eq!(
        s.errors()[0].code,
        "E_GENERIC_INTERVAL_EXCLUDED".parse().unwrap()
    );
    assert_eq!(s.errors()[0].trace_id, TraceId("Points".into()));
}
#[test]
fn error_limit_and_precancelled_run_never_claim_complete_execution() {
    let mut f = Fixture::new();
    f.max_errors = Some(1);
    let rows = [access("15012026", b'1', 1), access("16012026", b'1', 2)];
    let b = [suspension("10012026", "20012026", b'1', 1, 1)];
    let (out, s) = f
        .run(
            model(),
            Some(&[a()]),
            Some(&b),
            Some(&rows),
            ValidationMode::Full,
        )
        .unwrap();
    assert_eq!(errors(&s), 1);
    assert!(out.summary.truncated);
    f.max_errors = None;
    f.cancellation.cancel();
    let (out, s) = f
        .run(
            model(),
            Some(&[a()]),
            Some(&b),
            Some(&rows),
            ValidationMode::Full,
        )
        .unwrap();
    assert_eq!(out.summary.run_status, RunStatus::Cancelled);
    assert_eq!(errors(&s), 0);
}
#[test]
fn active_historical_obligations_cannot_be_bypassed_by_interval_support() {
    let mut v = model();
    v["persistence_policy"]["snapshot"] = serde_json::from_str::<Value>(include_str!(
        "../schemas/cad/components/2024-06.namespace.1/component.json"
    ))
    .unwrap()["persistence_policy"]["snapshot"]
        .clone();
    assert!(compile(v).is_err());
    let mut v = model();
    v["persistence_policy"]["cross_run_audit"] = json!([{"trace":"C","key":"record_key"}]);
    assert!(compile(v).is_err());
}

#[test]
fn temporal_when_cannot_hide_an_external_derived_dependency() {
    let mut v = model();
    v["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!("derived.calendar_year_delta.v1"));
    let mut join = v["traces"][2]["joins"][0].clone();
    join["id"] = json!("C_SELF");
    join["target_trace"] = json!("C");
    join["target_fields"] = json!(["4.1"]);
    join["cardinality"] = json!("zero_or_one");
    v["traces"][2]["joins"].as_array_mut().unwrap().push(join);
    v["traces"][2]["derived_values"] = json!({"external_delta":{"kind":"calendar_year_delta","birth_date_ref":{"trace":"C","field":"4.2","join":"C_SELF"},"reference_date_ref":{"field":"4.2"},"date_format":"DDMMYYYY"}});
    v["persistence_policy"]["temporal_rules"][0]["when"] =
        json!({"derived_compare":{"derived_value":"external_delta","operator":"==","value":0}});
    assert!(
        compile(v).is_err(),
        "when must remain local through derived dependencies"
    );
}
