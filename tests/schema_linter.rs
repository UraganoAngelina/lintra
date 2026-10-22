use lintra::{RawFlowSchema, SchemaIssueCode as Code, SchemaIssueSeverity, lint_raw_schema};
use serde_json::{Value, json};

fn sample() -> Value {
    let mut value: Value = serde_json::from_str(include_str!(
        "../docs/schema-authoring/examples/schemas/local-flow.json"
    ))
    .unwrap();
    let traces = value["traces"].as_array_mut().unwrap();
    traces.truncate(1);
    let trace = &mut traces[0];
    trace["id"] = json!("Z");
    trace["record_length"] = json!(8);
    let fields = trace["fields"].as_array_mut().unwrap();
    fields.truncate(2);
    for (index, field) in fields.iter_mut().enumerate() {
        field["id"] = json!(if index == 0 { "alpha" } else { "beta" });
        field["physical"] = json!({
            "type": "alphanumeric", "start": 1 + index * 4, "length": 4,
            "alignment": { "kind": "exact" },
            "lexical": { "kind": "encoded_text" }
        });
    }
    value
}

fn lint(value: Value) -> Vec<lintra::SchemaIssue> {
    lint_raw_schema(&serde_json::from_value::<RawFlowSchema>(value).unwrap())
}

fn codes(value: Value) -> Vec<Code> {
    lint(value).into_iter().map(|i| i.code).collect()
}

#[test]
fn contiguous_layout_and_explicit_filler_are_valid_without_mutating_raw() {
    let mut value = sample();
    let mut filler = value["traces"][0]["fields"][1].clone();
    filler["id"] = json!("explicit filler");
    filler["physical"]["start"] = json!(9);
    value["traces"][0]["fields"]
        .as_array_mut()
        .unwrap()
        .push(filler);
    value["traces"][0]["record_length"] = json!(12);
    let raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    let before = raw.clone();
    assert!(lint_raw_schema(&raw).is_empty());
    assert_eq!(raw, before);
}

#[test]
fn invalid_identifiers_are_reported_without_trimming_or_case_folding() {
    for path in ["flow", "version", "trace", "field"] {
        for bad in ["", " x", "x\t"] {
            let mut value = sample();
            match path {
                "flow" => value["flow_id"] = json!(bad),
                "version" => value["schema_version"] = json!(bad),
                "trace" => value["traces"][0]["id"] = json!(bad),
                _ => value["traces"][0]["fields"][0]["id"] = json!(bad),
            }
            assert_eq!(codes(value), [Code::InvalidIdentifier], "{path}/{bad:?}");
        }
    }
    let mut value = sample();
    value["flow_id"] = json!("Été");
    value["traces"][0]["fields"][1]["id"] = json!("Alpha");
    assert!(lint(value).is_empty()); // distinct from alpha; Unicode is preserved
}

#[test]
fn unsupported_format_and_empty_flow_are_errors() {
    let mut value = sample();
    value["schema_format_version"] = json!(3);
    value["traces"] = json!([]);
    assert_eq!(
        codes(value),
        [Code::UnsupportedSchemaFormat, Code::EmptySchema]
    );
}

#[test]
fn duplicate_ids_are_local_to_their_scope() {
    let mut value = sample();
    value["traces"][0]["fields"][1]["id"] = json!("alpha");
    assert_eq!(codes(value), [Code::DuplicateFieldId]);
    let mut value = sample();
    let second = value["traces"][0].clone();
    value["traces"].as_array_mut().unwrap().push(second);
    assert_eq!(codes(value.clone()), [Code::DuplicateTraceId]);
    value["traces"][1]["id"] = json!("A");
    assert!(lint(value).is_empty());
}

#[test]
fn invalid_start_and_zero_width_are_aggregated() {
    let mut value = sample();
    value["traces"][0]["fields"][0]["physical"]["start"] = json!(0);
    value["traces"][0]["fields"][0]["physical"]["length"] = json!(0);
    assert_eq!(
        codes(value),
        [Code::InvalidPosition, Code::ZeroLength, Code::Gap]
    );
}

#[test]
fn leading_and_internal_gaps_are_not_implicit_padding() {
    for index in [0, 1] {
        let mut value = sample();
        value["traces"][0]["fields"][index]["physical"]["start"] =
            json!(if index == 0 { 2 } else { 6 });
        value["traces"][0]["fields"][index]["physical"]["length"] = json!(3);
        assert_eq!(codes(value), [Code::Gap]);
    }
}

#[test]
fn overlap_uses_the_whole_covered_union_including_nested_ranges() {
    let mut value = sample();
    let mut third = value["traces"][0]["fields"][1].clone();
    third["id"] = json!("gamma");
    third["physical"]["start"] = json!(6);
    third["physical"]["length"] = json!(2);
    value["traces"][0]["fields"][0]["physical"]["length"] = json!(10);
    value["traces"][0]["fields"][1]["physical"]["start"] = json!(3);
    value["traces"][0]["fields"][1]["physical"]["length"] = json!(2);
    value["traces"][0]["fields"]
        .as_array_mut()
        .unwrap()
        .push(third);
    value["traces"][0]["record_length"] = json!(10);
    assert_eq!(codes(value), [Code::Overlap, Code::Overlap]);
}

#[test]
fn physical_order_is_checked_without_inventing_a_gap_from_declaration_order() {
    let mut value = sample();
    value["traces"][0]["fields"]
        .as_array_mut()
        .unwrap()
        .reverse();
    assert_eq!(codes(value), [Code::InvalidFieldOrder]);
}

#[test]
fn record_length_must_match_nonempty_coverage_exactly() {
    for length in [0, 7, 9] {
        let mut value = sample();
        value["traces"][0]["record_length"] = json!(length);
        assert_eq!(codes(value), [Code::RecordLengthMismatch]);
    }
    let mut value = sample();
    value["traces"][0]["fields"] = json!([]);
    assert_eq!(codes(value), [Code::RecordLengthMismatch]);
}

#[test]
fn physical_type_parameters_are_checked() {
    let mut value = sample();
    value["traces"][0]["fields"][0]["physical"] = json!({
        "type": "date", "start": 1, "length": 4, "format": "DDMMYYYY", "alignment": {"kind": "exact"}
    });
    assert_eq!(codes(value), [Code::InvalidTypeParameters]);
    for lexical in [
        json!({"kind":"byte_set", "allowed_bytes":[], "lowercase_bytes":[]}),
        json!({"kind":"byte_set", "allowed_bytes":[65,65], "lowercase_bytes":[]}),
        json!({"kind":"byte_set", "allowed_bytes":[65], "lowercase_bytes":[65,65]}),
        json!({"kind":"byte_set", "allowed_bytes":[65], "lowercase_bytes":[97]}),
    ] {
        let mut value = sample();
        value["traces"][0]["fields"][0]["physical"]["lexical"] = lexical;
        assert_eq!(codes(value), [Code::InvalidTypeParameters]);
    }
    let mut value = sample();
    value["traces"][0]["fields"][0]["physical"]["lexical"] =
        json!({"kind":"byte_set", "allowed_bytes":[65,97], "lowercase_bytes":[97]});
    assert!(lint(value).is_empty());
}

#[test]
fn unknown_type_aliases_and_date_formats_are_rejected_before_lint() {
    for kind in ["arbitrary", "AN", "N", "alphabetic", "decimal"] {
        let mut value = sample();
        value["traces"][0]["fields"][0]["physical"]["type"] = json!(kind);
        assert!(serde_json::from_value::<RawFlowSchema>(value).is_err());
    }
    let mut value = sample();
    value["traces"][0]["fields"][0]["physical"] = json!({
        "type":"date", "start":1, "length":8, "format":"YYYYMMDD", "alignment":{"kind":"exact"}
    });
    assert!(serde_json::from_value::<RawFlowSchema>(value).is_err());
}

#[test]
fn multiple_issues_have_stable_trace_field_and_stage_order() {
    let mut value = sample();
    value["schema_format_version"] = json!(0);
    value["traces"][0]["fields"][0]["id"] = json!("beta");
    value["traces"][0]["fields"][0]["physical"]["start"] = json!(0);
    value["traces"][0]["fields"][0]["physical"]["length"] = json!(0);
    let mut second = value["traces"][0].clone();
    second["id"] = json!("A");
    value["traces"].as_array_mut().unwrap().push(second);
    let raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    let expected = lint_raw_schema(&raw);
    assert_eq!(
        expected.iter().map(|i| i.code).collect::<Vec<_>>(),
        [
            Code::UnsupportedSchemaFormat,
            Code::InvalidPosition,
            Code::ZeroLength,
            Code::DuplicateFieldId,
            Code::Gap,
            Code::InvalidPosition,
            Code::ZeroLength,
            Code::DuplicateFieldId,
            Code::Gap,
        ]
    );
    assert_eq!(expected[1].trace_id.as_ref().unwrap().0, "Z");
    assert_eq!(expected[5].trace_id.as_ref().unwrap().0, "A");
    assert!(
        expected
            .iter()
            .all(|i| i.severity == SchemaIssueSeverity::Error && i.source_note.is_none())
    );
    assert!(
        expected[1..]
            .iter()
            .all(|i| i.field_id.as_ref().unwrap().0 == "beta")
    );
    for _ in 0..16 {
        assert_eq!(lint_raw_schema(&raw), expected);
    }
}

#[test]
fn maximal_raw_coordinates_are_checked_without_u32_wraparound() {
    let mut value = sample();
    value["traces"][0]["fields"]
        .as_array_mut()
        .unwrap()
        .truncate(1);
    value["traces"][0]["fields"][0]["physical"]["start"] = json!(u32::MAX);
    value["traces"][0]["fields"][0]["physical"]["length"] = json!(u32::MAX);
    value["traces"][0]["record_length"] = json!(u32::MAX);
    let result = codes(value);
    assert_eq!(result.contains(&Code::RangeOverflow), usize::BITS == 32);
    assert!(result.contains(&Code::Gap));
    assert!(result.contains(&Code::RecordLengthMismatch));
}

#[test]
fn approved_examples_and_p0_sioc_profile_pass_structural_lint() {
    for text in [
        include_str!("../docs/schema-authoring/examples/schemas/local-flow.json"),
        include_str!("../docs/schema-authoring/examples/schemas/history-flow.json"),
        include_str!("../docs/schema-authoring/examples/schemas/compatible-history-flow.json"),
        include_str!("../schemas/sioc/2026-04-20.structural-minimal.1/flow_schema.json"),
    ] {
        let raw: RawFlowSchema = serde_json::from_str(text).unwrap();
        assert_eq!(lint_raw_schema(&raw), [], "{}", raw.flow_id);
    }
    // Standalone P0 trace files are configuration projections, not RawFlowSchema.
    let flow: Value = serde_json::from_str(include_str!(
        "../schemas/sioc/2026-04-20.structural-minimal.1/flow_schema.json"
    ))
    .unwrap();
    for (index, text) in [
        include_str!("../schemas/sioc/2026-04-20.structural-minimal.1/trace-1.json"),
        include_str!("../schemas/sioc/2026-04-20.structural-minimal.1/trace-2.json"),
        include_str!("../schemas/sioc/2026-04-20.structural-minimal.1/trace-3.json"),
        include_str!("../schemas/sioc/2026-04-20.structural-minimal.1/trace-4.1.json"),
        include_str!("../schemas/sioc/2026-04-20.structural-minimal.1/trace-4.2.json"),
    ]
    .into_iter()
    .enumerate()
    {
        let trace: Value = serde_json::from_str(text).unwrap();
        assert_eq!(trace, flow["traces"][index]);
        let mut wrapper = flow.clone();
        wrapper["traces"] = json!([trace]);
        assert!(lint(wrapper).is_empty());
    }
}
