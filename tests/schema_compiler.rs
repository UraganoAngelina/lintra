use lintra::{
    AlignmentRule, BasePresence, ContentBoundaryPolicy, EncodingSpec, FieldId, FieldType,
    LexicalPlan, MissingValuePolicy, RawFlowSchema, RawResolvedEncoding, SchemaCompilationError,
    SchemaCompiler, SchemaIssueCode as Code, SchemaIssueSeverity, TraceId, ValidationCode,
    lint_raw_schema,
};
use serde_json::{Value, json};

fn sample() -> Value {
    json!({
        "schema_format_version":1, "flow_id":"EXAMPLE-MINIMAL", "schema_version":"v1",
        "regulatory_version":"synthetic-only", "source":{"note":"Synthetic configuration", "nested":{"eval":"metadata only"}},
        "encoding":{"mode":"runtime_required", "position_unit":"byte"},
        "required_capabilities":["local.v1"], "parameters":[], "reference_data":[], "persistence_policy":null,
        "traces":[{
            "id":"T", "name":"Synthetic trace", "record_length":8,
            "fields":[
                {"id":"Z", "name":"Synthetic Z", "physical":{
                    "type":"alphanumeric", "start":1, "length":3,
                    "alignment":{"kind":"right_pad", "fill_byte":32,"content_policy":"strip_trailing_fill"},
                    "lexical":{"kind":"encoded_text"}},
                 "missing_value":{"kind":"spaces"}, "presence":{"base":"required","required_if":[],"forbidden_if":[]},
                 "constraints":[], "regulatory_metadata":{"note":"No executable rule", "arbitrary":[1,"two"]}},
                {"id":"A", "name":"Synthetic A", "physical":{
                    "type":"numeric", "start":4, "length":2,
                    "alignment":{"kind":"left_pad", "fill_byte":48,"content_policy":"preserve_width"},
                    "lexical":{"kind":"byte_set", "allowed_bytes":[48,49,50,51,52,53,54,55,56,57], "lowercase_bytes":[]}},
                 "missing_value":{"kind":"exact","raw":"00"}, "presence":{"base":"optional","required_if":[],"forbidden_if":[]}, "constraints":[]},
                {"id":"N", "name":"Synthetic N", "physical":{
                    "type":"alphanumeric", "start":6, "length":3,
                    "alignment":{"kind":"exact"}, "lexical":{"kind":"encoded_text"}},
                 "missing_value":{"kind":"any_of","raw_values":["   ","000"]},
                 "presence":{"base":"required","required_if":[],"forbidden_if":[]}, "constraints":[]}
            ],
            "record_key":{"fields":["A","Z"],"uniqueness_scope":"trace", "error_code":"E_DUPLICATE_RECORD_KEY"},
            "joins":[], "identity":[], "named_conditions":{}, "derived_values":{}, "repeated_groups":[], "rules":[],
            "documented_non_executable_rules":[{"id":"DEFERRED", "fields":["N"], "policy":"not_enforced_in_this_version", "reason":"Synthetic deferred rule", "decision_reference":"EXAMPLE-DECISION"}]
        }]
    })
}
fn bound(value: Value) -> RawFlowSchema {
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    raw
}
fn error(value: Value) -> SchemaCompilationError {
    SchemaCompiler.compile(bound(value)).unwrap_err()
}
fn has(error: &SchemaCompilationError, code: Code) -> bool {
    error.issues.iter().any(|issue| issue.code == code)
}

#[test]
fn minimal_schema_freezes_exact_ranges_types_presence_keys_and_metadata() {
    let value = sample();
    let raw = bound(value.clone());
    let compiled = SchemaCompiler.compile(raw.clone()).unwrap();
    assert_eq!(serde_json::to_value(&raw).unwrap(), value);
    assert_eq!(compiled.key().flow_id.0, "EXAMPLE-MINIMAL");
    assert_eq!(compiled.key().version.0, "v1");
    assert_eq!(compiled.regulatory_version(), "synthetic-only");
    assert_eq!(compiled.schema_format_version(), 1);
    assert_eq!(compiled.encoding(), &EncodingSpec::Utf8);
    assert_eq!(
        serde_json::to_value(compiled.source()).unwrap(),
        value["source"]
    );
    let trace = compiled.trace(&TraceId("T".into())).unwrap();
    assert_eq!(trace.record_length(), 8);
    assert_eq!(
        trace
            .fields()
            .iter()
            .map(|f| f.id().0.as_str())
            .collect::<Vec<_>>(),
        ["Z", "A", "N"]
    );
    for (field, (start, length, end)) in
        trace.fields().iter().zip([(0, 3, 3), (3, 2, 5), (5, 3, 8)])
    {
        assert_eq!(
            (
                field.range().start(),
                field.range().len(),
                field.range().end_exclusive()
            ),
            (start, length, end)
        );
    }
    let a = trace.field(&FieldId("A".into())).unwrap();
    assert_eq!(a.field_type(), FieldType::Numeric);
    assert_eq!(a.range().slice(b"AB 01XYZ"), Some(&b"01"[..]));
    assert_eq!(a.presence_policy().base, BasePresence::Optional);
    assert_eq!(
        a.alignment(),
        AlignmentRule::LeftPad {
            fill: b'0',
            content_policy: ContentBoundaryPolicy::PreserveWidth
        }
    );
    assert_eq!(
        a.missing_value_policy(),
        &MissingValuePolicy::Exact(b"00".to_vec())
    );
    assert!(
        matches!(a.lexical(), Some(LexicalPlan::ByteSet(class)) if class.allowed[48] && class.allowed[57] && !class.allowed[65] && !class.lowercase[48])
    );
    let z = &trace.fields()[0];
    assert_eq!(z.field_type(), FieldType::Alphanumeric);
    assert_eq!(z.presence_policy().base, BasePresence::Required);
    assert_eq!(
        z.alignment(),
        AlignmentRule::RightPad {
            fill: b' ',
            content_policy: ContentBoundaryPolicy::StripTrailingFill
        }
    );
    assert_eq!(z.missing_value_policy(), &MissingValuePolicy::Spaces);
    assert_eq!(
        serde_json::to_value(z.regulatory_metadata()).unwrap(),
        value["traces"][0]["fields"][0]["regulatory_metadata"]
    );
    assert_eq!(trace.fields()[2].alignment(), AlignmentRule::Exact);
    assert_eq!(
        trace.fields()[2].missing_value_policy(),
        &MissingValuePolicy::AnyOf(vec![b"   ".to_vec(), b"000".to_vec()])
    );
    assert_eq!(trace.record_key().field_indexes, [1, 0]);
    assert_eq!(
        trace.record_key().error_code,
        ValidationCode::DuplicateRecordKey
    );
    assert_eq!(trace.projection_plan().field_indexes, [0, 1]);
    let deferred = &trace.documented_non_executable_rules()[0];
    assert_eq!(deferred.field_indexes, [2]);
    assert_eq!(deferred.id.as_ref(), "DEFERRED");
    assert_eq!(deferred.reason.as_ref(), "Synthetic deferred rule");
    assert_eq!(deferred.decision_reference.as_ref(), "EXAMPLE-DECISION");
    assert!(trace.local_rules().is_empty() && trace.identity_plan().bindings.is_empty());
    assert!(compiled.cross_record_plan().joins.is_empty());
    assert!(compiled.persistence_policy().is_none());
    let second = SchemaCompiler.compile(raw).unwrap();
    assert_eq!(second.traces()[0].fields()[1].range(), a.range());
}

#[test]
fn binding_is_explicit_rust_only_and_serialization_does_not_inject_it() {
    let value = sample();
    let raw: RawFlowSchema = serde_json::from_value(value.clone()).unwrap();
    assert!(raw.encoding.resolved.is_none());
    assert!(has(
        &SchemaCompiler.compile(raw).unwrap_err(),
        Code::MissingEncodingBinding
    ));
    assert_eq!(serde_json::to_value(bound(value.clone())).unwrap(), value);
    let mut injected = value;
    injected["encoding"]["resolved"] = json!("Utf8");
    assert!(serde_json::from_value::<RawFlowSchema>(injected).is_err());
    let serialized = serde_json::to_string(&bound(sample())).unwrap();
    let unbound: RawFlowSchema = serde_json::from_str(&serialized).unwrap();
    assert!(unbound.encoding.resolved.is_none());
    for encoding in [
        RawResolvedEncoding::Windows1252,
        RawResolvedEncoding::Iso8859_1,
        RawResolvedEncoding::Other("synthetic-codec".into()),
    ] {
        let mut raw = bound(sample());
        raw.encoding.resolved = Some(encoding);
        assert!(has(
            &SchemaCompiler.compile(raw).unwrap_err(),
            Code::UnsupportedEncoding
        ));
    }
}

#[test]
fn local_capability_is_required_and_other_capabilities_are_never_silently_accepted() {
    let mut value = sample();
    value["required_capabilities"] = json!([]);
    assert!(has(&error(value), Code::MissingCapability));
    for capability in [
        "expressions.v1",
        "joins.v1",
        "persistence.audit.v1",
        "temporal.intervals.v1",
    ] {
        let mut value = sample();
        value["required_capabilities"] = json!(["local.v1", capability]);
        assert!(has(&error(value), Code::UnsupportedCapability));
    }
    let mut value = sample();
    value["required_capabilities"] = json!(["local.v1", "local.v1"]);
    assert!(has(&error(value), Code::InvalidSchemaValue));
}

#[test]
fn lint_errors_and_unsupported_constructs_are_aggregated_with_context() {
    let mut value = sample();
    value["traces"][0]["fields"][0]["physical"]["start"] = json!(0);
    value["traces"][0]["fields"][1]["presence"]["required_if"] = json!([{"const":true}]);
    let mut second = value["traces"][0].clone();
    second["id"] = json!("A");
    value["traces"][0]["id"] = json!("Z");
    value["traces"].as_array_mut().unwrap().push(second);
    let raw = bound(value);
    let lint = lint_raw_schema(&raw);
    let result = SchemaCompiler.compile(raw.clone()).unwrap_err();
    assert!(lint.iter().all(|issue| result.issues.contains(issue)));
    assert!(has(&result, Code::UnsupportedConstruct));
    assert!(
        result
            .issues
            .iter()
            .all(|i| i.severity == SchemaIssueSeverity::Error)
    );
    let trace_ids: Vec<_> = result
        .issues
        .iter()
        .filter_map(|i| i.trace_id.as_ref().map(|id| id.0.as_str()))
        .collect();
    assert_eq!(trace_ids, ["Z", "Z", "Z", "A", "A", "A"]);
    assert!(!has(&result, Code::InternalCompilerInvariant));
    for _ in 0..8 {
        assert_eq!(SchemaCompiler.compile(raw.clone()).unwrap_err(), result);
    }
    let cause: &dyn std::error::Error = &result;
    assert!(cause.to_string().contains("issue(s)"));
    assert!(cause.source().is_none());
}

#[test]
fn every_excluded_ast_collection_is_a_blocking_boundary() {
    let local: Value = serde_json::from_str(include_str!(
        "../docs/schema-authoring/examples/schemas/local-flow.json"
    ))
    .unwrap();
    let history: Value = serde_json::from_str(include_str!(
        "../docs/schema-authoring/examples/schemas/history-flow.json"
    ))
    .unwrap();
    for name in [
        "joins",
        "named_conditions",
        "derived_values",
        "repeated_groups",
        "rules",
    ] {
        let payload = [&local, &history]
            .into_iter()
            .flat_map(|v| v["traces"].as_array().unwrap())
            .map(|t| &t[name])
            .find(|v| {
                v.as_array().is_some_and(|a| !a.is_empty())
                    || v.as_object().is_some_and(|m| !m.is_empty())
            })
            .unwrap();
        let mut value = sample();
        value["traces"][0][name] = payload.clone();
        let result = error(value);
        assert!(
            result
                .issues
                .iter()
                .any(|i| i.code == Code::UnsupportedConstruct && i.message.starts_with(name)),
            "{name}"
        );
    }
    for name in ["parameters", "reference_data", "persistence_policy"] {
        let mut value = sample();
        value[name] = history[name].clone();
        assert!(!value[name].is_null());
        assert!(
            error(value)
                .issues
                .iter()
                .any(|i| i.code == Code::UnsupportedConstruct && i.message.starts_with(name)),
            "{name}"
        );
    }
    let constraint = local["traces"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|t| t["fields"].as_array().unwrap())
        .find(|f| !f["constraints"].as_array().unwrap().is_empty())
        .unwrap()["constraints"]
        .clone();
    let mut value = sample();
    value["traces"][0]["fields"][0]["constraints"] = constraint;
    assert!(has(&error(value), Code::UnsupportedConstruct));
    let mut value = sample();
    value["traces"][0]["fields"][0]["physical"] = json!({"type":"date","start":1,"length":8,"format":"DDMMYYYY","alignment":{"kind":"exact"}});
    // Date is implemented now; expanding only one field still breaks layout.
    let result = error(value);
    assert!(has(&result, Code::Overlap) || has(&result, Code::RecordLengthMismatch));
    let mut value = sample();
    value["traces"][0]["fields"][0]["presence"]["forbidden_if"] = json!([{"const":true}]);
    assert!(has(&error(value), Code::UnsupportedConstruct));
}

#[test]
fn keys_and_deferred_references_are_resolved_without_defaults() {
    for fields in [json!([]), json!(["Z", "Z"])] {
        let mut value = sample();
        value["traces"][0]["record_key"]["fields"] = fields;
        assert!(has(&error(value), Code::InvalidRecordKey));
    }
    let mut value = sample();
    value["traces"][0]["record_key"]["fields"] = json!(["absent"]);
    let result = error(value);
    assert!(result.issues.iter().any(
        |i| i.code == Code::MissingFieldReference && i.field_id.as_ref().unwrap().0 == "absent"
    ));
    for code in ["E010", "E_SCHEMA_ERROR", "not-a-symbol"] {
        let mut value = sample();
        value["traces"][0]["record_key"]["error_code"] = json!(code);
        assert!(has(&error(value), Code::InvalidDiagnosticCode));
    }
    let mut value = sample();
    value["traces"][0]["documented_non_executable_rules"][0]["fields"] = json!(["absent"]);
    assert!(has(&error(value), Code::MissingFieldReference));
    for (name, bad) in [
        ("reason", json!("")),
        ("decision_reference", json!("")),
        ("id", json!(" x")),
        ("fields", json!(["Z", "Z"])),
    ] {
        let mut value = sample();
        value["traces"][0]["documented_non_executable_rules"][0][name] = bad;
        assert!(has(&error(value), Code::InvalidSchemaValue));
    }
}

#[test]
fn literal_widths_are_encoded_byte_widths_and_padding_is_not_inferred() {
    let mut value = sample();
    value["traces"][0]["fields"][0]["missing_value"] = json!({"kind":"exact","raw":"è "});
    let compiled = SchemaCompiler.compile(bound(value.clone())).unwrap();
    assert_eq!(
        compiled.traces()[0].fields()[0].missing_value_policy(),
        &MissingValuePolicy::Exact("è ".as_bytes().to_vec())
    );
    value["traces"][0]["fields"][0]["missing_value"] = json!({"kind":"exact","raw":"è"});
    assert!(has(&error(value), Code::InvalidTypeParameters));
    for values in [json!([]), json!(["000", "000"])] {
        let mut value = sample();
        value["traces"][0]["fields"][2]["missing_value"] =
            json!({"kind":"any_of","raw_values":values});
        assert!(has(&error(value), Code::InvalidSchemaValue));
    }
    let mut value = sample();
    value["traces"][0]["fields"][0]["physical"]["alignment"]["fill_byte"] = json!(195);
    assert!(has(&error(value), Code::InvalidTypeParameters));
    let mut value = sample();
    value["traces"][0]["fields"][0]["missing_value"] = json!({"kind":"never_missing"});
    assert_eq!(
        SchemaCompiler.compile(bound(value)).unwrap().traces()[0].fields()[0]
            .missing_value_policy(),
        &MissingValuePolicy::NeverMissing
    );
}

#[test]
fn names_and_regulatory_identity_are_checked_before_freezing() {
    for path in [
        "/regulatory_version",
        "/traces/0/name",
        "/traces/0/fields/0/name",
    ] {
        let mut value = sample();
        *value.pointer_mut(path).unwrap() = json!("");
        let result = error(value);
        assert!(has(&result, Code::InvalidIdentifier) || has(&result, Code::InvalidSchemaValue));
    }
}

#[test]
fn full_authoring_examples_and_sioc_are_explicitly_rejected_without_rewriting_them() {
    for text in [
        include_str!("../docs/schema-authoring/examples/schemas/local-flow.json"),
        include_str!("../docs/schema-authoring/examples/schemas/history-flow.json"),
        include_str!("../docs/schema-authoring/examples/schemas/compatible-history-flow.json"),
        include_str!("../schemas/sioc/2026-04-20.structural-minimal.1/flow_schema.json"),
    ] {
        let value: Value = serde_json::from_str(text).unwrap();
        let raw = bound(value.clone());
        assert_eq!(serde_json::to_value(&raw).unwrap(), value);
        let result = SchemaCompiler.compile(raw).unwrap_err();
        assert!(has(&result, Code::UnsupportedCapability));
        assert!(has(&result, Code::UnsupportedConstruct));
        assert!(!has(&result, Code::InternalCompilerInvariant));
    }
}

#[test]
fn remaining_padding_policies_and_lexical_classes_are_preserved() {
    for (kind, policy, expected) in [
        (
            "right_pad",
            "preserve_width",
            AlignmentRule::RightPad {
                fill: b' ',
                content_policy: ContentBoundaryPolicy::PreserveWidth,
            },
        ),
        (
            "left_pad",
            "strip_leading_fill",
            AlignmentRule::LeftPad {
                fill: b' ',
                content_policy: ContentBoundaryPolicy::StripLeadingFill,
            },
        ),
    ] {
        let mut value = sample();
        value["traces"][0]["fields"][0]["physical"]["alignment"] =
            json!({"kind":kind,"fill_byte":32,"content_policy":policy});
        value["traces"][0]["fields"][0]["physical"]["lexical"] =
            json!({"kind":"byte_set","allowed_bytes":[65,97],"lowercase_bytes":[97]});
        let compiled = SchemaCompiler.compile(bound(value)).unwrap();
        let field = &compiled.traces()[0].fields()[0];
        assert_eq!(field.alignment(), expected);
        assert!(
            matches!(field.lexical(),Some(LexicalPlan::ByteSet(class)) if class.allowed[65] && class.allowed[97] && class.lowercase[97] && !class.lowercase[65])
        );
    }
}

#[test]
fn successful_compilation_preserves_trace_order_and_local_key_scopes() {
    let mut value = sample();
    value["traces"][0]["id"] = json!("Z");
    let mut second = value["traces"][0].clone();
    second["id"] = json!("A");
    value["traces"].as_array_mut().unwrap().push(second);
    let compiled = SchemaCompiler.compile(bound(value)).unwrap();
    assert_eq!(
        compiled
            .traces()
            .iter()
            .map(|t| t.id().0.as_str())
            .collect::<Vec<_>>(),
        ["Z", "A"]
    );
    for trace in compiled.traces() {
        assert_eq!(trace.record_key().field_indexes, [1, 0]);
        assert_eq!(trace.fields()[1].id().0, "A");
    }
}

#[test]
fn format_range_is_enforced_by_the_compiler_without_a_fallback() {
    let compiled = SchemaCompiler.compile(bound(sample())).unwrap();
    assert_eq!(compiled.schema_format_version(), 1);
    for version in [0, 3, u32::MAX] {
        let mut value = sample();
        value["schema_format_version"] = json!(version);
        let failure = error(value);
        assert_eq!(failure.issues.len(), 1);
        assert_eq!(failure.issues[0].code, Code::UnsupportedSchemaFormat);
        let message = &failure.issues[0].message;
        assert!(message.contains(&format!("schema_format_version {version}")));
        assert!(message.contains(&format!("engine version {}", env!("CARGO_PKG_VERSION"))));
        assert!(message.contains("supported schema formats 1..=2"));
        assert!(message.contains("no compatible executable is declared"));
        assert!(message.contains(
            "minimum required executable version: not applicable to an undefined format"
        ));
        assert!(message.contains("defined format 1 minimum engine version: 0.1.0"));
        assert!(message.contains("development baseline; verified compiler b658e05"));
    }
}

#[test]
fn local_expression_model_errors_are_typed_while_execution_stays_blocked() {
    let mut value = sample();
    value["traces"][0]["fields"][0]["presence"]["required_if"] = json!([
        {"eq":{"ref":{"field":"ABSENT"},"value":"1"}}
    ]);
    let result = error(value);
    assert!(has(&result, Code::MissingFieldReference));
    assert!(has(&result, Code::UnsupportedConstruct));
    let mut value = sample();
    value["traces"][0]["named_conditions"] = json!({"loop":{"condition":"loop"}});
    let result = error(value);
    assert!(has(&result, Code::SelfReferentialRule));
    assert!(has(&result, Code::UnsupportedConstruct));
}
