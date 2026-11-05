//! T-CAD-10: the actual versioned T70 policy in an explicitly scoped synthetic
//! record harness. B1 offsets/length are real; other fields are not validated.
use lintra::*;
use serde_json::{Value, json};

const COMPONENT: &str = include_str!("../schemas/cad/components/2024-06.t70.1/component.json");
fn component() -> Value {
    serde_json::from_str(COMPONENT).unwrap()
}

fn filler(id: &str, start: usize, length: usize) -> Value {
    json!({"id":id,"name":"Synthetic bytes outside the T70 component",
        "physical":{"start":start,"length":length,"type":"alphanumeric",
            "alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}},
        "missing_value":{"kind":"spaces"},
        "presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[]})
}
fn schema() -> CompiledFlowSchema {
    let component = component();
    let mut v: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    v["flow_id"] = json!("CAD-T70-COMPONENT-TEST");
    v["schema_version"] = component["component_version"].clone();
    v["regulatory_version"] = json!("2024-06");
    v["source"] = json!({"scope":"T70-only synthetic harness; not full CAD compliance"});
    v["required_capabilities"] = component["required_capabilities"].clone();
    let trace = &mut v["traces"][0];
    trace["id"] = json!("B1");
    trace["record_length"] = component["source"]["record_length"].clone();
    trace["fields"] = json!([
        filler("SYNTHETIC-PREFIX", 1, 14),
        component["fields"][0],
        filler("SYNTHETIC-BETWEEN", 16, 51),
        component["fields"][1],
        filler("SYNTHETIC-SUFFIX", 137, 62)
    ]);
    // A synthetic harness needs a PK to compile. This is not the normative B1 PK.
    trace["record_key"]["fields"] = json!(["2.3"]);
    trace["identity"] = json!([]);
    trace["rules"] = component["rules"].clone();
    let mut raw: RawFlowSchema = serde_json::from_value(v).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw).unwrap()
}
fn row(kind: u8) -> Vec<u8> {
    let mut row = vec![b' '; 198];
    row[14] = kind;
    row[66..125].fill(b'1');
    row[125] = if kind == b'2' { b'1' } else { b' ' };
    row[126..136].fill(b'2');
    row
}
fn observe(schema: &CompiledFlowSchema, bytes: &[u8]) -> (bool, Vec<ValidationError>) {
    let trace = &schema.traces()[0];
    let mut summary = ValidationSummaryBuilder::new(
        schema.key().clone(),
        ScopeSummary::Trace {
            trace_id: trace.id().clone(),
        },
    );
    let mut sink = VecSink::default();
    let result = RecordValidator::new(schema, trace.id())
        .unwrap()
        .validate_local(
            RawRecord {
                line_number: 7,
                bytes,
            },
            &mut sink,
            &mut summary,
        )
        .unwrap();
    // Field/record behavior only: no invented input audit or run summary.
    (result.valid, sink.errors().to_vec())
}
fn jsonl(errors: &[ValidationError]) -> Vec<u8> {
    let mut sink = JsonlSink::new(Vec::new());
    for error in errors {
        sink.on_error(error).unwrap();
    }
    sink.finish().unwrap()
}

#[test]
fn actual_source_coordinates_exact_raw_width_and_both_valid_contexts() {
    let component = component();
    assert_eq!(component["source"]["record_length"], 198);
    assert_eq!(component["fields"][0]["physical"]["start"], 15);
    assert_eq!(component["fields"][1]["physical"]["start"], 67);
    assert_eq!(component["fields"][1]["physical"]["length"], 70);
    assert_eq!(
        component["fields"][1]["physical"]["alignment"]["kind"],
        "exact"
    );
    assert_eq!(component["source"]["full_normative_compliance"], false);
    let schema = schema();
    for kind in *b"12" {
        let mut bytes = row(kind);
        for index in (66..125).step_by(2) {
            bytes[index] = b'2';
        }
        assert_eq!(observe(&schema, &bytes), (true, vec![]));
        if kind == b'2' {
            bytes[125] = b'2';
            assert!(observe(&schema, &bytes).0);
        }
        // Neighboring bytes belong to the synthetic harness, not this field.
        bytes[65] = b'A';
        bytes[136] = b'Z';
        assert!(observe(&schema, &bytes).0);
    }
}

#[test]
fn all_256_byte_values_at_each_of_70_positions_match_the_source_domains() {
    let schema = schema();
    for kind in *b"12" {
        for position in 1..=70 {
            let mut bytes = row(kind);
            for candidate in 0..=u8::MAX {
                bytes[65 + position] = candidate;
                let expected = match position {
                    1..=59 => candidate == b'1' || candidate == b'2',
                    60 if kind == b'1' => candidate == b' ',
                    60 => candidate == b'1' || candidate == b'2',
                    _ => candidate == b'2',
                };
                let (valid, errors) = observe(&schema, &bytes);
                assert_eq!(
                    valid, expected,
                    "kind={} position={position} byte={candidate}",
                    kind as char
                );
                if !valid {
                    assert!(!errors.is_empty());
                    assert!(
                        errors
                            .iter()
                            .all(|error| error.field.as_ref().is_some_and(|f| f.id.0 == "2.22"))
                    );
                }
            }
        }
    }
}

#[test]
fn conditional_position_60_has_real_coordinates_context_and_deterministic_jsonl() {
    let schema = schema();
    for (kind, wrong) in [(b'1', b'1'), (b'1', b'2'), (b'2', b' ')] {
        let mut bytes = row(kind);
        bytes[125] = wrong;
        let (valid, errors) = observe(&schema, &bytes);
        assert!(!valid);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationCode::CrossField);
        let field = errors[0].field.as_ref().unwrap();
        assert_eq!(field.id.0, "2.22");
        assert_eq!(
            (field.start_one_based, field.end_one_based_inclusive),
            (67, 136)
        );
        assert_eq!(errors[0].trace_id.0, "B1");
        assert_eq!(errors[0].line_number, 7);
        assert!(errors[0].message.contains("posizione 60"));
        assert_eq!(jsonl(&errors), jsonl(&observe(&schema, &bytes).1));
        assert!(!jsonl(&errors).contains(&b'\r'));
    }
}

#[test]
fn zero_fill_space_fill_and_a_blank_tail_never_receive_numeric_or_text_fallback() {
    let schema = schema();
    for kind in *b"12" {
        for fill in *b"0 " {
            let mut bytes = row(kind);
            bytes[66..136].fill(fill);
            assert!(!observe(&schema, &bytes).0);
        }
        let mut bytes = row(kind);
        bytes[135] = b' ';
        let (valid, errors) = observe(&schema, &bytes);
        assert!(!valid);
        assert_eq!(errors[0].code, ValidationCode::InvalidDomain);
    }
}

#[test]
fn invalid_control_values_cannot_make_the_record_valid() {
    let schema = schema();
    for control in *b"03 ?" {
        let mut bytes = row(b'1');
        bytes[14] = control;
        let (valid, errors) = observe(&schema, &bytes);
        assert!(!valid);
        assert!(
            errors
                .iter()
                .any(|e| e.field.as_ref().is_some_and(|f| f.id.0 == "2.3"))
        );
    }
}
