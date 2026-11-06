//! CAD09 calendar-year age rules through the real two-pass service. All rows
//! are synthetic; fields outside the versioned component are explicit filler.
use lintra::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, sync::Arc};
const COMPONENT: &str =
    include_str!("../schemas/cad/components/2024-06.calendar-age.1/component.json");
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
fn model(scope: &str) -> Value {
    let c = component();
    let mut v: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    v["flow_id"] = json!("CAD-AGE-COMPONENT-TEST");
    v["schema_version"] = c["component_version"].clone();
    v["regulatory_version"] = c["source"]["regulatory_version"].clone();
    v["schema_format_version"] = c["schema_format_version"].clone();
    v["source"] = json!({"scope":"age component only; not full CAD compliance","rule_group":scope});
    v["required_capabilities"] = c["required_capabilities"].clone();
    v["parameters"] = c["parameters"].clone();
    v["persistence_policy"] = c["persistence_policy"].clone();
    let template = v["traces"][0].clone();
    v["traces"] = json!([]);
    for id in ["A", "B1"] {
        let d = &c["traces"][id];
        let mut t = template.clone();
        t["id"] = json!(id);
        t["record_length"] = d["record_length"].clone();
        t["record_key"] = d["record_key"].clone();
        t["identity"] = json!([]);
        let active = scope == "all" || scope == id;
        t["joins"] = if active {
            d["joins"].clone()
        } else {
            json!([])
        };
        t["derived_values"] = if active {
            d["derived_values"].clone()
        } else {
            json!({})
        };
        t["rules"] = if active {
            d["rules"].clone()
        } else {
            json!([])
        };
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
fn set(r: &mut [u8], start: usize, value: &str) {
    r[start - 1..start - 1 + value.len()].copy_from_slice(value.as_bytes());
}
fn a(birth: &str, guardian: u8, unit: &str) -> Vec<u8> {
    let mut r = vec![b' '; 218];
    set(&mut r, 1, unit);
    set(&mut r, 149, birth);
    set(&mut r, 206, "000000000001");
    r[217] = guardian;
    r
}
fn b(start: &str, mobility: u8, function: u8, sequence: u64) -> Vec<u8> {
    let mut r = vec![b' '; 198];
    set(&mut r, 1, "000000000001");
    set(&mut r, 16, start);
    r[61] = mobility;
    r[62] = function;
    set(&mut r, 187, &format!("{sequence:012}"));
    r
}
struct Fixture {
    dir: tempfile::TempDir,
}
impl Fixture {
    fn new() -> Self {
        Self {
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
        mode: ValidationMode,
    ) -> Result<(ValidationRunOutcome, VecSink), ValidationRunError> {
        let mut inputs = vec![];
        for (index, rows) in [arows, brows].into_iter().enumerate() {
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
                max_errors: None,
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
            &CancellationToken::default(),
        )?;
        write_run_summary(Vec::new(), &out.summary).unwrap();
        Ok((out, sink))
    }
}
fn required(sink: &VecSink) -> Vec<String> {
    sink.errors()
        .iter()
        .filter(|e| e.code == ValidationCode::ConditionalRequired)
        .map(|e| {
            e.field
                .as_ref()
                .expect("field-present errors must name their target")
                .id
                .0
                .clone()
        })
        .collect()
}
fn jsonl(sink: &VecSink) -> Vec<u8> {
    let mut out = JsonlSink::new(Vec::new());
    for e in sink.errors() {
        out.on_error(e).unwrap();
    }
    out.finish().unwrap()
}

#[test]
fn actual_calendar_thresholds_two_three_five_six_do_not_use_the_birthday() {
    let f = Fixture::new();
    for (age, fields) in [
        (2, vec![]),
        (3, vec!["2.17"]),
        (5, vec!["2.17"]),
        (6, vec!["2.17", "2.18"]),
    ] {
        let ar = [a(&format!("3112{}", 2026 - age), b'1', "501")];
        let br = [b("01012026", b'0', b'0', 1)];
        let (out, sink) = f
            .run(model("all"), Some(&ar), Some(&br), ValidationMode::Full)
            .unwrap();
        assert_eq!(required(&sink), fields);
        assert_eq!(out.status == RunStatus::CompletedValid, age == 2);
        assert_eq!(out.summary.completeness(), ValidationCompleteness::Complete);
        // Before the December birthday, completed-years would be one smaller.
        if age >= 3 {
            assert!(
                sink.errors()
                    .iter()
                    .all(|e| e.code == ValidationCode::ConditionalRequired)
            );
        }
    }
}
#[test]
fn parental_responsibility_is_required_through_seventeen_and_optional_after() {
    let f = Fixture::new();
    for age in [17, 18] {
        let ar = [a(&format!("3112{}", 2026 - age), b'0', "501")];
        let br = [b("01012026", b'1', b'1', 1)];
        let (out, sink) = f
            .run(model("all"), Some(&ar), Some(&br), ValidationMode::Full)
            .unwrap();
        assert_eq!(
            required(&sink),
            if age == 17 { vec!["1.24"] } else { vec![] }
        );
        assert_eq!(out.status == RunStatus::CompletedValid, age == 18);
    }
    let ar = [a("31122008", b'1', "501")];
    let br = [b("01012026", b'1', b'1', 1)];
    assert_eq!(
        f.run(model("all"), Some(&ar), Some(&br), ValidationMode::Full)
            .unwrap()
            .0
            .status,
        RunStatus::CompletedValid
    );
}
#[test]
fn declared_age_sources_really_enter_both_projection_plans_and_the_cross_pass() {
    let flow = compile(model("all")).unwrap();
    assert_eq!(flow.cross_record_plan().rules.len(), 3);
    for (trace, id) in [("A", "1.11"), ("B1", "2.4")] {
        let t = flow.trace(&TraceId(trace.into())).unwrap();
        let index = t.field_index(&FieldId(id.into())).unwrap();
        assert!(t.projection_plan().field_indexes.contains(&index));
        let projected: Vec<_> = t
            .projection_plan()
            .field_indexes
            .iter()
            .map(|&i| t.fields()[i].id().0.as_str())
            .collect();
        assert_eq!(
            projected,
            if trace == "A" {
                vec!["1.1", "1.11", "1.23", "1.24"]
            } else {
                vec!["2.1", "2.4", "2.17", "2.18", "2.36"]
            }
        );
    }
    let f = Fixture::new();
    let ar = [a("31122020", b'1', "501")];
    let br = [b("01012026", b'1', b'1', 1)];
    let (out, sink) = f
        .run(model("all"), Some(&ar), Some(&br), ValidationMode::Full)
        .unwrap();
    assert!(sink.errors().is_empty());
    assert_eq!(out.status, RunStatus::CompletedValid);
    assert_eq!(out.summary.full_validation.unwrap().controls.len(), 3);
}
#[test]
fn invalid_or_missing_dates_do_not_turn_into_an_age_or_a_second_presence_error() {
    let f = Fixture::new();
    for (birth, start) in [
        ("31022020", "01012026"),
        ("        ", "01012026"),
        ("31122020", "31022026"),
        ("31122020", "        "),
    ] {
        let ar = [a(birth, b'0', "501")];
        let br = [b(start, b'0', b'0', 1)];
        let (out, sink) = f
            .run(model("all"), Some(&ar), Some(&br), ValidationMode::Full)
            .unwrap();
        assert_eq!(out.status, RunStatus::CompletedInvalid);
        assert!(required(&sink).is_empty());
        assert!(
            out.summary
                .full_validation
                .unwrap()
                .controls
                .iter()
                .any(|c| c.unknown > 0)
        );
    }
}
#[test]
fn consensus_never_chooses_one_of_two_inconsistent_b1_start_dates() {
    let f = Fixture::new();
    let ar = [a("01012010", b'0', "501")];
    for starts in [["01012026", "01012026"], ["01012026", "02012026"]] {
        let br = [b(starts[0], b'1', b'1', 1), b(starts[1], b'1', b'1', 2)];
        let (out, sink) = f
            .run(model("all"), Some(&ar), Some(&br), ValidationMode::Full)
            .unwrap();
        if starts[0] == starts[1] {
            assert_eq!(required(&sink), vec!["1.24"]);
            assert_eq!(out.summary.completeness(), ValidationCompleteness::Complete);
        } else {
            assert!(required(&sink).is_empty());
            assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
            let audit = out.summary.full_validation.unwrap();
            let guardian = audit
                .controls
                .iter()
                .find(|c| c.rule_id == "CAD-AGE-GUARDIAN")
                .unwrap();
            assert_eq!(guardian.unknown, 1);
            assert!(guardian.causes.iter().any(
                |cause| cause.diagnostic_code.as_deref() == Some("W_CAD_AGE_CONTEXT_CONFLICT")
            ));
        }
    }
}
#[test]
fn unavailable_target_is_unknown_but_available_empty_parent_uses_the_declared_policy() {
    let f = Fixture::new();
    let br = [b("01012026", b'0', b'0', 1)];
    let (out, sink) = f
        .run(model("B1"), None, Some(&br), ValidationMode::Full)
        .unwrap();
    assert_eq!(out.status, RunStatus::CompletedValid);
    assert!(sink.errors().is_empty());
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
    let (out, sink) = f
        .run(model("B1"), Some(&[]), Some(&br), ValidationMode::Full)
        .unwrap();
    assert_eq!(out.status, RunStatus::CompletedInvalid);
    assert!(required(&sink).is_empty());
    assert_eq!(sink.errors().len(), 2);
    assert!(
        sink.errors()
            .iter()
            .all(|e| e.code.as_symbol().unwrap() == "E_RELATED_RECORD_MISSING")
    );
    let ar = [a("01012010", b'0', "501")];
    let (out, sink) = f
        .run(model("A"), Some(&ar), None, ValidationMode::Full)
        .unwrap();
    assert!(sink.errors().is_empty());
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
}
#[test]
fn wrong_namespace_or_duplicate_parent_cannot_supply_a_reliable_birth_date() {
    let f = Fixture::new();
    let br = [b("01012026", b'0', b'0', 1)];
    for ar in [
        vec![a("01012010", b'0', "502")],
        vec![a("01012010", b'0', "501"), a("01012010", b'0', "501")],
    ] {
        let (out, sink) = f
            .run(model("B1"), Some(&ar), Some(&br), ValidationMode::Full)
            .unwrap();
        assert_eq!(out.status, RunStatus::CompletedInvalid);
        assert!(required(&sink).is_empty());
        assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
        assert!(
            sink.errors()
                .iter()
                .all(|e| e.code.as_symbol().unwrap() != "E_RELATED_RECORD_MISSING")
        );
    }
}
#[test]
fn local_only_performs_local_checks_and_explicitly_skips_cross_derived_obligations() {
    let f = Fixture::new();
    let br = [b("01012026", b'0', b'0', 1)];
    let (out, sink) = f
        .run(model("B1"), None, Some(&br), ValidationMode::LocalOnly)
        .unwrap();
    assert_eq!(out.status, RunStatus::CompletedValid);
    assert!(sink.errors().is_empty());
    assert!(out.summary.skipped_rules >= 2);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
}
#[test]
fn derived_unknown_provenance_preserves_false_and_true_boolean_dominance() {
    let f = Fixture::new();
    let br = [b("01012026", b'1', b'0', 1)];
    for (op, constant) in [("and", false), ("or", true)] {
        let mut v = model("B1");
        v["traces"][1]["rules"].as_array_mut().unwrap().truncate(1);
        let when = v["traces"][1]["rules"][0]["when"].clone();
        v["traces"][1]["rules"][0]["when"] = json!({op:[{"const":constant},when]});
        let (out, sink) = f
            .run(v, Some(&[]), Some(&br), ValidationMode::Full)
            .unwrap();
        assert_eq!(out.status, RunStatus::CompletedValid);
        assert!(sink.errors().is_empty());
        assert_eq!(out.summary.completeness(), ValidationCompleteness::Complete);
    }
}
#[test]
fn unsupported_local_presence_set_predicate_and_non_date_bindings_fail_at_compile_time() {
    let base = model("all");
    let mut missing_join = base.clone();
    missing_join["traces"][1]["derived_values"]["calendar_age"]["birth_date_ref"]["join"] =
        json!("not-declared");
    assert!(compile(missing_join).is_err());
    let mut unscoped_set = base.clone();
    unscoped_set["traces"][0]["joins"][0]
        .as_object_mut()
        .unwrap()
        .remove("reduction");
    assert!(
        compile(unscoped_set)
            .unwrap_err()
            .issues
            .iter()
            .any(|e| e.code == SchemaIssueCode::UnsupportedConstruct)
    );
    let mut completed_years = base.clone();
    completed_years["traces"][1]["derived_values"]["calendar_age"]["kind"] =
        json!("age_in_completed_years");
    completed_years["traces"][1]["derived_values"]["calendar_age"]["leap_day_anniversary"] =
        json!("mar01");
    assert!(
        compile(completed_years)
            .unwrap_err()
            .issues
            .iter()
            .any(|e| e.code == SchemaIssueCode::UnsupportedConstruct)
    );
    let mut v = base.clone();
    v["traces"][1]["derived_values"]["calendar_age"]["birth_date_ref"]["field"] = json!("1.24");
    assert!(
        compile(v)
            .unwrap_err()
            .issues
            .iter()
            .any(|e| e.code == SchemaIssueCode::ConstraintTypeMismatch)
    );
    let mut v = base.clone();
    let field = v["traces"][1]["fields"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|f| f["id"] == "2.17")
        .unwrap();
    field["presence"]["required_if"] =
        json!([{"derived_compare":{"derived_value":"calendar_age","operator":">=","value":3}}]);
    assert!(
        compile(v)
            .unwrap_err()
            .issues
            .iter()
            .any(|e| e.code == SchemaIssueCode::UnsupportedConstruct)
    );
    let mut v = base;
    v["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!("joins.sets.v1"));
    v["traces"][0]["rules"][0]["assertions"] = json!([{"kind":"set_exists","join":"age_start","predicate":{"derived_compare":{"derived_value":"calendar_age","operator":">=","value":3}}}]);
    assert!(
        compile(v)
            .unwrap_err()
            .issues
            .iter()
            .any(|e| e.code == SchemaIssueCode::UnsupportedConstruct)
    );
}
#[test]
fn renamed_ids_and_repeated_jsonl_prove_schema_driven_deterministic_evaluation() {
    fn rename(v: &mut Value) {
        match v {
            Value::String(s) => {
                let replacement = match s.as_str() {
                    "A" => Some("PARENT"),
                    "B1" => Some("CHILD"),
                    "1.1" => Some("UNIT"),
                    "1.11" => Some("DOB"),
                    "1.23" => Some("P-EVENT"),
                    "1.24" => Some("GUARD"),
                    "2.1" => Some("C-EVENT"),
                    "2.4" => Some("START"),
                    "2.17" => Some("MOBILITY"),
                    "2.18" => Some("FUNCTION"),
                    "2.36" => Some("SEQUENCE"),
                    _ => None,
                };
                if let Some(r) = replacement {
                    *s = r.into();
                }
            }
            Value::Array(a) => {
                for v in a {
                    rename(v);
                }
            }
            Value::Object(m) => {
                for v in m.values_mut() {
                    rename(v);
                }
            }
            _ => {}
        }
    }
    let f = Fixture::new();
    let ar = [a("31122020", b'1', "501")];
    let br = [b("01012026", b'0', b'0', 1)];
    let mut v = model("all");
    rename(&mut v);
    let (out, one) = f
        .run(v.clone(), Some(&ar), Some(&br), ValidationMode::Full)
        .unwrap();
    assert_eq!(out.status, RunStatus::CompletedInvalid);
    assert_eq!(required(&one), vec!["MOBILITY", "FUNCTION"]);
    let (_, two) = f
        .run(v, Some(&ar), Some(&br), ValidationMode::Full)
        .unwrap();
    assert_eq!(jsonl(&one), jsonl(&two));
    assert!(!jsonl(&one).contains(&b'\r'));
}

#[test]
fn generic_format_one_calendar_join_does_not_require_a_persistence_profile() {
    let f = Fixture::new();
    let mut v = model("B1");
    v["flow_id"] = json!("SYNTHETIC-CROSS-DATES");
    v["schema_format_version"] = json!(1);
    v["source"] = json!({"scope":"generic calendar join only; no CAD namespace compliance"});
    v["parameters"] = json!([]);
    v["persistence_policy"] = Value::Null;
    v["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c != "persistence.namespace_binding.v1");
    v["traces"][1]["joins"][0]["namespace_parameters"] = json!([]);
    let ar = [a("31122020", b'1', "501")];
    let br = [b("01012026", b'0', b'0', 1)];
    let (out, sink) = f
        .run(v, Some(&ar), Some(&br), ValidationMode::Full)
        .unwrap();
    assert_eq!(out.status, RunStatus::CompletedInvalid);
    assert_eq!(required(&sink), vec!["2.17", "2.18"]);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Complete);
}
