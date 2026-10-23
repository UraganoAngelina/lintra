//! Public compiler -> stored plans -> actual local record/rule/report paths.
use lintra::*;
use serde_json::{Value, json};

fn value(rules: Value) -> Value {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    value["required_capabilities"] = json!([
        "local.v1",
        "expressions.v1",
        "derived.substring.v1",
        "derived.age_in_completed_years.v1",
        "derived.calendar_year_delta.v1"
    ]);
    let trace = &mut value["traces"][0];
    let base = trace["fields"][0].clone();
    let mut fields = vec![];
    for (id, start, length, kind) in [
        ("DETAIL", 1, 4, "alphanumeric"),
        ("BIRTH", 5, 8, "date"),
        ("REF", 13, 8, "date"),
        ("AGE", 21, 3, "numeric"),
        ("CODE", 24, 5, "alphanumeric"),
    ] {
        let mut field = base.clone();
        field["id"] = json!(id);
        field["name"] = json!(format!("Synthetic {id}"));
        field["physical"] = if kind == "date" {
            json!({"type":kind,"start":start,"length":length,"format":"DDMMYYYY","alignment":{"kind":"exact"}})
        } else {
            json!({"type":kind,"start":start,"length":length,"alignment":if id=="CODE" {json!({"kind":"left_pad","fill_byte":32,"content_policy":"strip_leading_fill"})} else {json!({"kind":"exact"})},"lexical":{"kind":"encoded_text"}})
        };
        field["missing_value"] = json!({"kind":"spaces"});
        fields.push(field);
    }
    trace["fields"] = json!(fields);
    trace["record_length"] = json!(28);
    trace["record_key"]["fields"] = json!(["DETAIL"]);
    trace["derived_values"] = json!({
        "AGE":{"kind":"age_in_completed_years","birth_date_ref":{"field":"BIRTH"},"reference_date_ref":{"field":"REF"},"date_format":"DDMMYYYY","leap_day_anniversary":"mar01"},
        "PREFIX":{"kind":"substring","source_ref":{"field":"CODE"},"target":"logical","start":1,"length":2},
        "YEAR":{"kind":"calendar_year_delta","birth_date_ref":{"field":"BIRTH"},"reference_date_ref":{"field":"REF"},"date_format":"DDMMYYYY"}
    });
    trace["rules"] = rules;
    value
}
fn compile(value: Value) -> Result<CompiledFlowSchema, SchemaCompilationError> {
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw)
}
fn rule(when: Value, assertions: Value) -> Value {
    json!({"id":"R-DERIVED","when":when,"assertions":assertions,"error_code":"E_CROSS_FIELD","message_template":"Regola {rule_id}: {condition}"})
}
fn compare(id: &str, operator: &str, value: Value) -> Value {
    json!({"kind":"derived_compare","derived_value":id,"operator":operator,"value":value})
}
fn bytes(detail: &str, birth: &str, reference: &str, age: &str) -> Vec<u8> {
    format!("{detail}{birth}{reference}{age}  AB ").into_bytes()
}
fn evaluate(schema: &CompiledFlowSchema, bytes: &[u8]) -> RuleValidationResult {
    let view = RecordView::new(bytes);
    let local = LocalEvaluationContext {
        record: &view,
        schema: &schema.traces()[0],
    };
    validate_local_rule(
        &schema.traces()[0].local_rules()[0],
        &EvaluationContext { fields: &local },
    )
}
fn observe(
    schema: &CompiledFlowSchema,
    bytes: &[u8],
) -> Result<(LocalRecordValidationArtifacts, Vec<ValidationError>), RecordValidationError> {
    let mut sink = VecSink::default();
    let trace = &schema.traces()[0];
    let mut summary = ValidationSummaryBuilder::new(
        schema.key().clone(),
        ScopeSummary::Trace {
            trace_id: trace.id().clone(),
        },
    );
    let result = RecordValidator::new(schema, trace.id())?.validate_local(
        RawRecord {
            line_number: 7,
            bytes,
        },
        &mut sink,
        &mut summary,
    )?;
    Ok((result, sink.errors().to_vec()))
}

#[test]
fn compiler_retains_every_declared_supported_capability_and_real_derived_plan() {
    let schema = compile(value(json!([]))).unwrap();
    assert_eq!(
        schema.required_capabilities(),
        &[
            Capability::LocalV1,
            Capability::ExpressionsV1,
            Capability::DerivedSubstringV1,
            Capability::DerivedAgeInCompletedYearsV1,
            Capability::DerivedCalendarYearDeltaV1
        ]
    );
    assert_eq!(schema.traces()[0].derived_values().len(), 3);
    assert!(
        observe(&schema, &bytes("ABCD", "01012000", "01012021", "021"))
            .unwrap()
            .0
            .valid
    );
}

#[test]
fn derived_guard_reports_actual_primary_and_all_source_bytes_once_in_order() {
    let when = json!({"and":[
        {"derived_eq":{"ref":{"field":"AGE"},"derived_value":"AGE"}},
        {"derived_compare":{"derived_value":"PREFIX","operator":"==","value":"AB"}},
        {"derived_compare":{"derived_value":"YEAR","operator":">=","value":21}}
    ]});
    let mut r = rule(
        when,
        json!([{"kind":"field_present","field":{"field":"DETAIL"}}]),
    );
    r["message_template"] = json!("{field_id} ({field_name}) richiesto: {condition}. {rule_id}");
    let schema = compile(value(json!([r]))).unwrap();
    let input = bytes("    ", "01012000", "01012021", "021");
    let before = input.clone();
    let (result, errors) = observe(&schema, &input).unwrap();
    assert!(!result.valid);
    assert_eq!(input, before);
    assert_eq!(errors.len(), 1);
    let error = &errors[0];
    assert_eq!(error.code, ValidationCode::CrossField);
    assert_eq!(error.field.as_ref().unwrap().id.0, "DETAIL");
    assert_eq!(error.actual.as_ref().unwrap().display, "    ");
    assert_eq!(
        error
            .context
            .iter()
            .map(|item| (item.field_id.0.as_str(), item.value.display.as_str()))
            .collect::<Vec<_>>(),
        [
            ("AGE", "021"),
            ("BIRTH", "01012000"),
            ("REF", "01012021"),
            ("CODE", "  AB "),
            ("DETAIL", "    ")
        ]
    );
    assert_eq!(
        error.message,
        "DETAIL (Synthetic DETAIL) richiesto: (AGE = anni compiuti(BIRTH, REF; anniversario 29/02: 01/03) AND sottostringa logical di CODE [start 1, length 2] = \"AB\" AND differenza anni(BIRTH, REF) >= 21). R-DERIVED"
    );
    assert!(!error.message.contains("derived_index"));
    let mut sink = JsonlSink::new(Vec::new());
    sink.on_error(error).unwrap();
    let wire: Value = serde_json::from_slice(&sink.finish().unwrap()).unwrap();
    assert_eq!(wire["field"]["start"], 1);
    assert_eq!(wire["field"]["end"], 4);
    assert_eq!(wire["context"][3]["value"]["display"], "  AB ");
}

#[test]
fn derived_assertions_cover_all_operators_exact_integer_and_byte_results() {
    let input = bytes("ABCD", "01012000", "01012021", "021");
    for (operator, literal, satisfied) in [
        ("==", 21, true),
        ("!=", 21, false),
        ("<", 22, true),
        ("<=", 20, false),
        (">", 20, true),
        (">=", 22, false),
    ] {
        let schema = compile(value(json!([rule(
            json!({"const":true}),
            json!([compare("AGE", operator, json!(literal))])
        )])))
        .unwrap();
        let result = evaluate(&schema, &input);
        assert_eq!(matches!(result, RuleValidationResult::Satisfied), satisfied);
        let (record, errors) = observe(&schema, &input).unwrap();
        assert_eq!(record.valid, satisfied);
        if !satisfied {
            assert_eq!(errors.len(), 1);
            assert!(errors[0].field.is_none());
            assert!(errors[0].actual.is_none());
            assert_eq!(
                errors[0]
                    .context
                    .iter()
                    .map(|item| item.field_id.0.as_str())
                    .collect::<Vec<_>>(),
                ["BIRTH", "REF"]
            );
        }
    }
    let schema = compile(value(json!([rule(
        json!({"const":true}),
        json!([
            compare("PREFIX", "==", json!("AB")),
            compare("YEAR", "==", json!(21))
        ])
    )])))
    .unwrap();
    assert_eq!(evaluate(&schema, &input), RuleValidationResult::Satisfied);
    let schema = compile(value(json!([rule(
        json!({"const":true}),
        json!([
            compare("AGE", "==", json!(-5)),
            compare("YEAR", "==", json!(-4))
        ])
    )])))
    .unwrap();
    assert_eq!(
        evaluate(&schema, &bytes("ABCD", "31012025", "30012021", "000")),
        RuleValidationResult::Satisfied
    );
}

#[test]
fn unknown_reasons_survive_guard_and_assertion_and_false_guard_does_not_invent_a_violation() {
    for (birth, reason) in [
        ("        ", UnknownReason::FieldUnavailable),
        ("31022000", UnknownReason::InvalidFieldValue),
    ] {
        let input = bytes("ABCD", birth, "01012021", "021");
        for when in [
            json!({"const":true}),
            json!({"derived_compare":{"derived_value":"AGE","operator":">","value":0}}),
        ] {
            let schema = compile(value(json!([rule(
                when,
                json!([compare("AGE", ">", json!(0))])
            )])))
            .unwrap();
            assert_eq!(
                evaluate(&schema, &input),
                RuleValidationResult::Unknown(reason)
            );
            assert!(
                matches!(observe(&schema, &input), Err(RecordValidationError::UndeterminedRule { reason: found, .. }) if found == reason)
            );
        }
        let schema = compile(value(json!([rule(
            json!({"const":false}),
            json!([compare("AGE", ">", json!(0))])
        )])))
        .unwrap();
        assert_eq!(evaluate(&schema, &input), RuleValidationResult::Satisfied);
    }
}

#[test]
fn condition_true_derived_eq_uses_the_real_plan_and_failure_context() {
    let expr = json!({"derived_eq":{"ref":{"field":"AGE"},"derived_value":"AGE"}});
    let schema = compile(value(json!([rule(
        json!({"const":true}),
        json!([{"kind":"condition_true","condition":expr}])
    )])))
    .unwrap();
    assert!(
        observe(&schema, &bytes("ABCD", "01012000", "01012021", "021"))
            .unwrap()
            .0
            .valid
    );
    let (_, errors) = observe(&schema, &bytes("ABCD", "01012000", "01012021", "020")).unwrap();
    assert_eq!(
        errors[0]
            .context
            .iter()
            .map(|item| item.field_id.0.as_str())
            .collect::<Vec<_>>(),
        ["AGE", "BIRTH", "REF"]
    );
    assert!(errors[0].field.is_none());
}

#[test]
fn public_presence_path_uses_compiled_derived_plan_and_preserves_conflict() {
    let mut raw = value(json!([]));
    let condition = json!({"derived_compare":{"derived_value":"AGE","operator":">=","value":18}});
    raw["traces"][0]["fields"][0]["presence"]["required_if"] = json!([condition]);
    let schema = compile(raw.clone()).unwrap();
    let (_, errors) = observe(&schema, &bytes("    ", "01012000", "01012021", "021")).unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, ValidationCode::ConditionalRequired);
    assert!(
        observe(&schema, &bytes("    ", "01012010", "01012021", "011"))
            .unwrap()
            .0
            .valid
    );
    raw["traces"][0]["fields"][0]["presence"]["forbidden_if"] = json!([condition]);
    let schema = compile(raw).unwrap();
    let (_, errors) = observe(&schema, &bytes("ABCD", "01012000", "01012021", "021")).unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, ValidationCode::RuleConflict);
}

fn year_assertion(digits: u32) -> Value {
    json!({"kind":"substring_equals_derived_year","field":{"field":"DETAIL"},"start":1,"length":digits,"source_ref":{"field":"REF"},"source_format":"DDMMYYYY","year_digits":digits})
}
#[test]
fn year_assertion_validates_source_date_and_preserves_two_or_four_digits() {
    for (digits, good, bad) in [(2, "01xx", "00xx"), (4, "0001", "0000")] {
        let mut r = rule(json!({"const":true}), json!([year_assertion(digits)]));
        r["message_template"] = json!("{field_id} {field_name}: {condition}");
        let schema = compile(value(json!([r]))).unwrap();
        assert_eq!(
            evaluate(&schema, &bytes(good, "01010001", "01010001", "000")),
            RuleValidationResult::Satisfied
        );
        let (_, errors) = observe(&schema, &bytes(bad, "01010001", "01010001", "000")).unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field.as_ref().unwrap().id.0, "DETAIL");
        assert_eq!(
            errors[0]
                .context
                .iter()
                .map(|item| item.field_id.0.as_str())
                .collect::<Vec<_>>(),
            ["DETAIL", "REF"]
        );
        assert_eq!(
            evaluate(&schema, &bytes(good, "01010001", "31022021", "000")),
            RuleValidationResult::Unknown(UnknownReason::InvalidFieldValue)
        );
    }
}

#[test]
fn invalid_derivation_bindings_types_ranges_and_templates_fail_before_records() {
    let cases = [
        (
            compare("ABSENT", "==", json!(1)),
            SchemaIssueCode::MissingDerivedValueReference,
        ),
        (
            compare("AGE", "==", json!("21")),
            SchemaIssueCode::ConstraintTypeMismatch,
        ),
        (
            compare("PREFIX", "==", json!(21)),
            SchemaIssueCode::ConstraintTypeMismatch,
        ),
        (year_assertion(3), SchemaIssueCode::InvalidTypeParameters),
    ];
    for (assertion, expected) in cases {
        let error = compile(value(json!([rule(
            json!({"const":false}),
            json!([assertion])
        )])))
        .unwrap_err();
        assert!(
            error.issues.iter().any(|issue| issue.code == expected),
            "{error:?}"
        );
    }
    let mut r = rule(
        json!({"const":true}),
        json!([compare("AGE", ">", json!(0))]),
    );
    r["message_template"] = json!("{field_id}");
    assert!(
        compile(value(json!([r])))
            .unwrap_err()
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::InvalidSchemaValue)
    );
    let mut raw = value(json!([]));
    raw["traces"][0]["derived_values"]["PREFIX"]["length"] = json!(6);
    assert!(
        compile(raw)
            .unwrap_err()
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::InvalidTypeParameters)
    );
    let mut raw = value(json!([]));
    raw["traces"][0]["derived_values"]["AGE"]["birth_date_ref"] = json!({"field":"CODE"});
    assert!(
        compile(raw)
            .unwrap_err()
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::ConstraintTypeMismatch)
    );
}

#[test]
fn derived_rules_resolve_by_id_across_reversed_physical_layouts() {
    let when = json!({"and":[
        {"derived_eq":{"ref":{"field":"AGE"},"derived_value":"AGE"}},
        {"derived_compare":{"derived_value":"PREFIX","operator":"==","value":"AB"}},
        {"derived_compare":{"derived_value":"YEAR","operator":">=","value":18}}
    ]});
    let mut raw = value(json!([rule(
        json!({"condition":"GUARD"}),
        json!([{"kind":"field_present","field":{"field":"DETAIL"}}])
    )]));
    raw["traces"][0]["named_conditions"] = json!({"GUARD":when});
    let first = compile(raw.clone()).unwrap();
    let original_fields = raw["traces"][0]["fields"].as_array().unwrap().clone();
    let fields = raw["traces"][0]["fields"].as_array_mut().unwrap();
    fields.reverse();
    let mut start = 1;
    for field in fields.iter_mut() {
        field["physical"]["start"] = json!(start);
        start += field["physical"]["length"].as_u64().unwrap();
    }
    let reversed_ids = fields
        .iter()
        .map(|field| field["id"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let second = compile(raw).unwrap();
    for input in [
        bytes("    ", "01012000", "01012021", "021"),
        bytes("ABCD", "01012000", "01012021", "021"),
        bytes("    ", "01012010", "01012021", "011"),
        bytes("    ", "        ", "01012021", "021"),
    ] {
        let parts = original_fields
            .iter()
            .map(|field| {
                let start =
                    usize::try_from(field["physical"]["start"].as_u64().unwrap() - 1).unwrap();
                let length =
                    usize::try_from(field["physical"]["length"].as_u64().unwrap()).unwrap();
                (field["id"].as_str().unwrap(), &input[start..start + length])
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        let reordered = reversed_ids
            .iter()
            .flat_map(|id| parts[id.as_str()].iter().copied())
            .collect::<Vec<_>>();
        match (observe(&first, &input), observe(&second, &reordered)) {
            (Ok((left, a)), Ok((right, b))) => {
                assert_eq!(left, right);
                assert_eq!(a.len(), b.len());
                for (a, b) in a.iter().zip(&b) {
                    assert_eq!(a.code, b.code);
                    assert_eq!(a.context, b.context);
                    assert_eq!(a.actual, b.actual);
                    assert_eq!(a.expected, b.expected);
                    assert_eq!(a.message, b.message);
                    assert_eq!(a.field.as_ref().unwrap().start_one_based, 1);
                    assert_eq!(b.field.as_ref().unwrap().start_one_based, 25);
                }
            }
            (
                Err(RecordValidationError::UndeterminedRule { reason: a, .. }),
                Err(RecordValidationError::UndeterminedRule { reason: b, .. }),
            ) => assert_eq!(a, b),
            pair => panic!("layout changed the outcome: {pair:?}"),
        }
    }
}

#[test]
fn known_failed_assertion_dominates_an_earlier_unknown_derived_obligation() {
    let schema = compile(value(json!([rule(
        json!({"const":true}),
        json!([
            compare("AGE",">",json!(0)), {"kind":"field_present","field":{"field":"DETAIL"}}
        ])
    )])))
    .unwrap();
    let input = bytes("    ", "        ", "01012021", "021");
    assert!(matches!(
        evaluate(&schema, &input),
        RuleValidationResult::Violated(_)
    ));
    let (result, errors) = observe(&schema, &input).unwrap();
    assert!(!result.valid);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field.as_ref().unwrap().id.0, "DETAIL");
    assert_eq!(
        errors[0]
            .context
            .iter()
            .map(|item| item.field_id.0.as_str())
            .collect::<Vec<_>>(),
        ["DETAIL"]
    );
}

#[test]
fn static_conflicts_are_proved_without_guessing_the_truth_of_a_derived_condition() {
    let mut raw = value(json!([]));
    let condition = json!({"derived_compare":{"derived_value":"AGE","operator":">=","value":18}});
    raw["traces"][0]["fields"][0]["presence"]["required_if"] = json!([condition]);
    raw["traces"][0]["fields"][0]["presence"]["forbidden_if"] = json!([condition]);
    let schema = compile(raw.clone()).unwrap();
    assert_eq!(
        observe(&schema, &bytes("ABCD", "01012000", "01012021", "021"))
            .unwrap()
            .1[0]
            .code,
        ValidationCode::RuleConflict
    );
    assert!(
        observe(&schema, &bytes("    ", "01012010", "01012021", "011"))
            .unwrap()
            .0
            .valid
    );
    raw["traces"][0]["fields"][0]["presence"]["required_if"] = json!([{"const":true}]);
    raw["traces"][0]["fields"][0]["presence"]["forbidden_if"] = json!([{"const":true}]);
    assert!(
        compile(raw)
            .unwrap_err()
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::PresenceConflict)
    );
}

#[test]
fn derived_profile_roundtrips_and_unreachable_bad_definitions_are_rejected() {
    let profile: Value =
        serde_json::from_slice(include_bytes!("golden/derived/v1/schema.json")).unwrap();
    let raw: RawFlowSchema = serde_json::from_value(profile.clone()).unwrap();
    assert_eq!(serde_json::to_value(raw).unwrap(), profile);
    assert_eq!(
        compile(profile).unwrap().traces()[0].derived_values().len(),
        3
    );
    let mut raw = value(json!([]));
    raw["traces"][0]["named_conditions"] =
        json!({"A":{"condition":"B"},"B":{"and":[{"condition":"A"},{"const":true}]}});
    assert!(
        compile(raw)
            .unwrap_err()
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::SelfReferentialRule)
    );
    let mut raw = value(json!([]));
    raw["traces"][0]["named_conditions"] =
        json!({"UNUSED":{"derived_compare":{"derived_value":"ABSENT","operator":"==","value":1}}});
    assert!(
        compile(raw)
            .unwrap_err()
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::MissingDerivedValueReference)
    );
}
