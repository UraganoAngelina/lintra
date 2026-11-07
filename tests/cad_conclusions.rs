//! Synthetic CAD07 component executed through the actual two-pass service.
use lintra::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, sync::Arc};
const COMPONENT: &str =
    include_str!("../schemas/cad/components/2024-06.conclusions.1/component.json");
const CODE: &str = "E_CAD_ACCESS_AFTER_CONCLUSION";
fn component() -> Value {
    serde_json::from_str(COMPONENT).unwrap()
}
fn filler(start: usize, length: usize) -> Value {
    json!({"id":format!("SYNTHETIC-{start}"),"name":"Unvalidated synthetic filler",
        "physical":{"type":"alphanumeric","start":start,"length":length,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}},
        "missing_value":{"kind":"spaces"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[]})
}
/// Component test schema: selected normative fields retain their coordinates;
/// filler never claims clinical validation of omitted fields.
fn model() -> Value {
    let c = component();
    let mut v: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    v["flow_id"] = json!("CAD-CONCLUSION-COMPONENT-TEST");
    v["schema_version"] = c["component_version"].clone();
    v["regulatory_version"] = c["source"]["regulatory_version"].clone();
    v["schema_format_version"] = c["schema_format_version"].clone();
    v["source"] = json!({"scope":"conclusion component only; not full CAD compliance"});
    v["required_capabilities"] = c["required_capabilities"].clone();
    v["parameters"] = c["parameters"].clone();
    v["persistence_policy"] = c["persistence_policy"].clone();
    let template = v["traces"][0].clone();
    v["traces"] = json!([]);
    for id in ["A", "B1", "C"] {
        let d = &c["traces"][id];
        let mut t = template.clone();
        t["id"] = json!(id);
        t["record_length"] = d["record_length"].clone();
        t["record_key"] = d["record_key"].clone();
        t["identity"] = json!([]);
        t["named_conditions"] = d.get("named_conditions").cloned().unwrap_or(json!({}));
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
fn conclusion(date: &str, code: &str, seq: u64) -> Vec<u8> {
    let mut r = vec![b' '; 198];
    set(&mut r, 1, "000000000001");
    r[14] = if code == "13" { b'2' } else { b'1' };
    set(&mut r, 25, date);
    set(&mut r, 33, code);
    set(&mut r, 187, &format!("{seq:012}"));
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
        .find(|c| c.rule_id == "temporal.after_date_restriction.0")
        .unwrap()
}

#[test]
fn every_valid_conclusion_reason_forbids_only_strictly_later_home_visits() {
    for code in [
        "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "80", "82",
        "88", "97", "98", "99",
    ] {
        let rows = [
            access("14012026", b'1', 1),
            access("15012026", b'1', 2),
            access("16012026", b'1', 3),
        ];
        let (out, s) = run(Some(&[conclusion("15012026", code, 1)]), &rows);
        assert_eq!(errors(&s), 1, "code {code}");
        assert_eq!(control(&out).violations, 1);
        assert_eq!(control(&out).satisfied, 2);
        assert_eq!(
            s.errors().len(),
            1,
            "unexpected local errors for {code}: {:?}",
            s.errors()
        );
    }
}
#[test]
fn four_death_codes_restrict_five_sites_but_preserve_condolence_and_same_day() {
    for code in ["03", "04", "07", "82"] {
        let rows: Vec<_> = b"123456"
            .iter()
            .map(|site| access("16012026", *site, u64::from(*site - b'0')))
            .collect();
        let (_, s) = run(Some(&[conclusion("15012026", code, 1)]), &rows);
        assert_eq!(errors(&s), 5);
        assert_eq!(
            s.errors().iter().map(|e| e.line_number).collect::<Vec<_>>(),
            [1, 2, 4, 5, 6]
        );
        let rows: Vec<_> = b"123456"
            .iter()
            .map(|site| access("15012026", *site, u64::from(*site - b'0')))
            .collect();
        let (_, s) = run(Some(&[conclusion("15012026", code, 1)]), &rows);
        assert_eq!(errors(&s), 0);
    }
    let rows: Vec<_> = b"123456"
        .iter()
        .map(|site| access("16012026", *site, u64::from(*site - b'0')))
        .collect();
    let (_, s) = run(Some(&[conclusion("15012026", "01", 1)]), &rows);
    assert_eq!(errors(&s), 1);
}
#[test]
fn open_event_is_clear_but_absent_empty_or_unmatched_evidence_is_unknown() {
    let rows = [access("16012026", b'1', 1)];
    let (out, s) = run(Some(&[conclusion("        ", "00", 1)]), &rows);
    assert_eq!(errors(&s), 0);
    assert_eq!(control(&out).satisfied, 1);
    for evidence in [None, Some(vec![])] {
        let (out, s) = run(evidence.as_deref(), &rows);
        assert_eq!(errors(&s), 0);
        assert_eq!(control(&out).unknown, 1);
    }
    let mut other = conclusion("15012026", "03", 1);
    set(&mut other, 1, "000000000002");
    let (out, s) = run(Some(&[other]), &rows);
    assert_eq!(errors(&s), 0);
    assert_eq!(control(&out).unknown, 1);
}
#[test]
fn identical_conclusions_are_qualified_and_conflicts_are_never_filtered_away() {
    let rows = [access("16012026", b'1', 1), access("16012026", b'2', 2)];
    let (out, s) = run(
        Some(&[
            conclusion("        ", "00", 1),
            conclusion("15012026", "03", 2),
            conclusion("15012026", "03", 3),
        ]),
        &rows,
    );
    assert_eq!(errors(&s), 2);
    assert_eq!(control(&out).unknown, 0);
    for (date, code) in [("14012026", "03"), ("15012026", "01")] {
        let (out, s) = run(
            Some(&[conclusion("15012026", "03", 1), conclusion(date, code, 2)]),
            &rows,
        );
        assert_eq!(errors(&s), 0);
        let full = out.summary.full_validation.unwrap();
        for c in full
            .controls
            .iter()
            .filter(|c| c.rule_id.starts_with("temporal."))
        {
            assert!(c.unknown > 0);
            assert!(c.causes.iter().any(|cause| cause.diagnostic_code.as_deref()
                == Some("W_CAD_CONCLUSION_CONTEXT_CONFLICT")));
        }
    }
}
#[test]
fn invalid_dates_and_partial_target_never_become_an_open_or_qualified_event() {
    for date in ["32132026", "00000000"] {
        let (out, s) = run(
            Some(&[conclusion(date, "03", 1)]),
            &[access("16012026", b'1', 1)],
        );
        assert_eq!(errors(&s), 0);
        assert_eq!(control(&out).unknown, 1);
    }
    let (out, s) = run(
        Some(&[b"short".to_vec(), conclusion("15012026", "03", 2)]),
        &[access("16012026", b'1', 1)],
    );
    assert_eq!(errors(&s), 0);
    assert_eq!(control(&out).unknown, 1);
    let (out, s) = run(
        Some(&[conclusion("15012026", "03", 1)]),
        &[access("32132026", b'1', 1)],
    );
    assert_eq!(errors(&s), 0);
    assert_eq!(control(&out).unknown, 1);
}
#[test]
fn calendar_ordering_handles_year_changes_and_leap_days() {
    for (end, before, same, after) in [
        ("31122025", "30122025", "31122025", "01012026"),
        ("29022024", "28022024", "29022024", "01032024"),
    ] {
        let (_, s) = run(
            Some(&[conclusion(end, "01", 1)]),
            &[
                access(before, b'1', 1),
                access(same, b'1', 2),
                access(after, b'1', 3),
            ],
        );
        assert_eq!(errors(&s), 1);
        assert_eq!(s.errors()[0].line_number, 3);
    }
}

#[test]
fn compiler_checks_temporal_types_capabilities_sets_and_consensus_closure() {
    compile(model()).unwrap();
    for mutation in 0..9 {
        let mut v = model();
        match mutation {
            0 => v["persistence_policy"]["temporal_rules"][0]["date_field"] = json!("4.7"),
            1 => {
                v["persistence_policy"]["temporal_rules"][0]["conclusion_date_field"] = json!("2.7")
            }
            2 => {
                v["traces"][2]["joins"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("reduction");
            }
            3 => v["traces"][2]["joins"][0]["reduction"]["fields"] = json!(["2.6"]),
            4 => v["persistence_policy"]["temporal_rules"][0]["forbidden_values"] = json!([]),
            5 => {
                v["persistence_policy"]["temporal_rules"][0]["forbidden_values"] = json!(["1", "1"])
            }
            6 => {
                v["persistence_policy"]["temporal_rules"][0]["on_missing_trace"]["diagnostic"]["code"] =
                    json!("different")
            }
            7 => v["persistence_policy"]["temporal_rules"][0]["owner_trace"] = json!("missing"),
            _ => {
                v["required_capabilities"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|c| c != "temporal.after_date.v1");
            }
        }
        assert!(compile(v).is_err(), "mutation {mutation}");
    }
}
#[test]
fn qualified_target_predicate_supports_local_derived_values_and_rejects_hidden_external_reads() {
    let mut v = model();
    v["flow_id"] = json!("GENERIC-AFTER-DERIVED");
    v["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!("derived.substring.v1"));
    v["traces"][1]["derived_values"] = json!({"code_copy":{"kind":"substring","source_ref":{"field":"2.7"},"target":"logical","start":1,"length":2}});
    v["persistence_policy"]["temporal_rules"][1]["target_when"] =
        json!({"derived_compare":{"derived_value":"code_copy","operator":"==","value":"03"}});
    let (_, s) = Fixture::new()
        .run(
            v,
            Some(&[a()]),
            Some(&[conclusion("15012026", "03", 1)]),
            Some(&[access("16012026", b'2', 1)]),
            ValidationMode::Full,
        )
        .unwrap();
    assert_eq!(errors(&s), 1);
    let mut v = model();
    v["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!("derived.calendar_year_delta.v1"));
    let mut join = v["traces"][2]["joins"][0].clone();
    join["id"] = json!("B1_SELF");
    join["target_trace"] = json!("B1");
    join["source_fields"] = json!(["2.1"]);
    join["target_fields"] = json!(["2.1"]);
    join["cardinality"] = json!("zero_or_one");
    join.as_object_mut().unwrap().remove("reduction");
    v["traces"][1]["joins"] = json!([join]);
    v["traces"][1]["derived_values"] = json!({"external_delta":{"kind":"calendar_year_delta","birth_date_ref":{"trace":"B1","field":"2.6","join":"B1_SELF"},"reference_date_ref":{"field":"2.6"},"date_format":"DDMMYYYY"}});
    v["persistence_policy"]["temporal_rules"][1]["target_when"] =
        json!({"derived_compare":{"derived_value":"external_delta","operator":"==","value":0}});
    assert!(compile(v).is_err());
}
#[test]
fn scalar_join_refuses_multiple_targets_instead_of_selecting_a_conclusion() {
    let mut v = model();
    v["traces"][2]["joins"][0]["cardinality"] = json!("zero_or_one");
    v["traces"][2]["joins"][0]
        .as_object_mut()
        .unwrap()
        .remove("reduction");
    let (_, s) = Fixture::new()
        .run(
            v.clone(),
            Some(&[a()]),
            Some(&[conclusion("15012026", "03", 1)]),
            Some(&[access("16012026", b'2', 1)]),
            ValidationMode::Full,
        )
        .unwrap();
    assert_eq!(errors(&s), 1);
    let (out, s) = Fixture::new()
        .run(
            v,
            Some(&[a()]),
            Some(&[
                conclusion("15012026", "03", 1),
                conclusion("15012026", "03", 2),
            ]),
            Some(&[access("16012026", b'1', 1)]),
            ValidationMode::Full,
        )
        .unwrap();
    assert_eq!(errors(&s), 0);
    assert_eq!(control(&out).unknown, 1);
}
#[test]
fn deterministic_diagnostics_budget_and_precancelled_run_preserve_outcomes() {
    let b = [conclusion("15012026", "03", 1)];
    let rows = [access("16012026", b'1', 1), access("16012026", b'2', 2)];
    let encode = |s: &VecSink| {
        let mut sink = JsonlSink::new(Vec::new());
        for e in s.errors() {
            sink.on_error(e).unwrap();
        }
        sink.finish().unwrap()
    };
    let (_, first) = run(Some(&b), &rows);
    let (_, second) = run(Some(&b), &rows);
    assert_eq!(encode(&first), encode(&second));
    assert_eq!(
        first
            .errors()
            .iter()
            .map(|e| e.line_number)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let mut f = Fixture::new();
    f.max_errors = Some(1);
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
fn generic_renaming_preserves_restricted_field_diagnostics_without_cad_branches() {
    let mut v = model();
    v["flow_id"] = json!("GENERIC-AFTER-COMPONENT");
    let mut text = serde_json::to_string(&v).unwrap();
    for (old, new) in [
        ("A", "People"),
        ("B1", "Conclusions"),
        ("C", "Points"),
        ("1.1", "organization"),
        ("1.23", "person_event"),
        ("2.1", "conclusion_event"),
        ("2.3", "kind"),
        ("2.6", "closed_on"),
        ("2.7", "close_reason"),
        ("2.36", "closure_id"),
        ("4.1", "point_event"),
        ("4.2", "point_date"),
        ("4.4", "type"),
        ("4.5", "actor"),
        ("4.7", "location"),
        ("4.11", "point_id"),
        ("C_TO_B1_CONCLUSION", "POINT_CONCLUSION"),
        (CODE, "E_GENERIC_AFTER_CONCLUSION"),
    ] {
        text = text.replace(&format!("\"{old}\""), &format!("\"{new}\""));
    }
    let v: Value = serde_json::from_str(&text).unwrap();
    let (_, s) = Fixture::new()
        .run(
            v,
            Some(&[a()]),
            Some(&[conclusion("15012026", "03", 1)]),
            Some(&[access("16012026", b'2', 1)]),
            ValidationMode::Full,
        )
        .unwrap();
    assert_eq!(s.errors().len(), 1);
    let e = &s.errors()[0];
    assert_eq!(e.code, "E_GENERIC_AFTER_CONCLUSION".parse().unwrap());
    assert_eq!(e.trace_id, TraceId("Points".into()));
    assert_eq!(e.line_number, 1);
    let field = e.field.as_ref().unwrap();
    assert_eq!(field.id, FieldId("location".into()));
}
