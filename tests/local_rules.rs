//! Local rules: synthetic equivalent of ARCH 2.29/2.2 and typed assertion paths.
use lintra::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, time::UNIX_EPOCH};
fn value(rules: Value) -> Value {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("fixtures/application/schema.json")).unwrap();
    value["required_capabilities"] = json!(["local.v1", "expressions.v1"]);
    let trace = &mut value["traces"][0];
    trace["record_length"] = json!(5);
    trace["record_key"]["fields"] = json!(["DETAIL"]);
    let fields = trace["fields"].as_array_mut().unwrap();
    fields[0]["id"] = json!("DETAIL");
    fields[0]["name"] = json!("Synthetic detail");
    fields[0]["physical"] = json!({"type":"alphanumeric","start":1,"length":3,"alignment":{"kind":"right_pad","fill_byte":32,"content_policy":"strip_trailing_fill"},"lexical":{"kind":"encoded_text"}});
    fields[1]["id"] = json!("ORIGIN");
    fields[1]["physical"] = json!({"type":"numeric","start":4,"length":2,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}});
    for field in fields {
        field["presence"]["base"] = json!("optional");
        field["constraints"] = json!([]);
    }
    trace["identity"] = json!([]);
    trace["rules"] = rules;
    value
}
fn compile(value: Value) -> Result<CompiledFlowSchema, SchemaCompilationError> {
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw)
}
fn rule(when: Value, assertions: Value) -> Value {
    json!({"id":"R-PRESENCE","when":when,"assertions":assertions,"error_code":"E_CROSS_FIELD","message_template":"Regola {rule_id}, campo {field_id} ({field_name}), condizione {condition}."})
}
fn presence_rule() -> Value {
    let mut rule = rule(
        json!({"not_in":{"ref":{"field":"ORIGIN"},"values":["01","02"]}}),
        json!([{"kind":"field_present","field":{"field":"DETAIL"}}]),
    );
    rule["error_code"] = json!("E_CONDITIONAL_REQUIRED");
    rule["message_template"] = json!(
        "Il campo {field_id} ({field_name}) è obbligatorio perché {condition}. Regola {rule_id}."
    );
    rule
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
) -> Result<
    (
        LocalRecordValidationArtifacts,
        Vec<ValidationError>,
        ValidationSummary,
    ),
    RecordValidationError,
> {
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
    let audits = InputAuditSession::new();
    let mut reader = audits
        .wrap(
            trace.id().clone(),
            "synthetic.txt".into(),
            bytes.len().try_into().unwrap(),
            std::io::Cursor::new(bytes),
        )
        .unwrap();
    std::io::copy(&mut reader, &mut std::io::sink()).unwrap();
    drop(reader);
    let metadata = audits
        .finish(RunMetadata {
            validation_mode: ValidationMode::LocalOnly,
            validator_version: "synthetic".into(),
            schema_hash: "synthetic".into(),
            reference_data_versions: BTreeMap::new(),
            started_at: UNIX_EPOCH,
            inputs: vec![],
        })
        .unwrap();
    Ok((
        result,
        sink.errors().to_vec(),
        summary.finish(metadata).unwrap(),
    ))
}
#[test]
fn source_equivalent_rule_reports_target_trigger_raw_context_and_controlled_message() {
    let schema = compile(value(json!([presence_rule()]))).unwrap();
    for bytes in [b"   01".as_slice(), b"   02", b"A  05"] {
        assert_eq!(evaluate(&schema, bytes), RuleValidationResult::Satisfied);
    }
    assert!(matches!(
        evaluate(&schema, b"   05"),
        RuleValidationResult::Violated(_)
    ));
    let (result, errors, summary) = observe(&schema, b"   05").unwrap();
    assert!(!result.valid);
    assert_eq!(summary.invalid_records, 1);
    assert_eq!(errors.len(), 1);
    let error = &errors[0];
    assert_eq!(error.code, ValidationCode::ConditionalRequired);
    assert_eq!(error.field.as_ref().unwrap().id.0, "DETAIL");
    assert_eq!(error.actual.as_ref().unwrap().display, "   ");
    assert_eq!(error.line_number, 7);
    assert_eq!(
        error
            .context
            .iter()
            .map(|item| (item.field_id.0.as_str(), item.value.display.as_str()))
            .collect::<Vec<_>>(),
        [("ORIGIN", "05"), ("DETAIL", "   ")]
    );
    assert_eq!(
        error.message,
        "Il campo DETAIL (Synthetic detail) è obbligatorio perché ORIGIN NOT IN [\"01\", \"02\"]. Regola R-PRESENCE."
    );
}
#[test]
fn rule_order_and_actual_failed_assertion_are_preserved() {
    let mut first = presence_rule();
    first["id"] = json!("Z");
    first["assertions"] = json!([{"kind":"field_present","field":{"field":"ORIGIN"}},{"kind":"field_present","field":{"field":"DETAIL"}}]);
    let mut second = presence_rule();
    second["id"] = json!("A");
    let schema = compile(value(json!([first, second]))).unwrap();
    let (_, errors, summary) = observe(&schema, b"   05").unwrap();
    assert_eq!(errors.len(), 2);
    assert_eq!(summary.records_processed, 1);
    assert_eq!(summary.invalid_records, 1);
    assert!(errors[0].message.ends_with("Regola Z."));
    assert!(errors[1].message.ends_with("Regola A."));
    assert!(
        errors
            .iter()
            .all(|error| error.field.as_ref().unwrap().id.0 == "DETAIL")
    );
}
#[test]
fn unknown_guard_and_assertion_remain_unknown_and_abort_record_explicitly() {
    let schema = compile(value(json!([presence_rule()]))).unwrap();
    assert_eq!(
        evaluate(&schema, b"A    "),
        RuleValidationResult::Unknown(UnknownReason::FieldUnavailable)
    );
    assert!(matches!(
        observe(&schema, b"A    "),
        Err(RecordValidationError::UndeterminedRule {
            reason: UnknownReason::FieldUnavailable,
            ..
        })
    ));
    let mut guarded = presence_rule();
    guarded["when"] = json!({"const":false});
    let schema = compile(value(json!([guarded]))).unwrap();
    assert_eq!(evaluate(&schema, b"     "), RuleValidationResult::Satisfied);
}
#[test]
fn regex_assertions_respect_raw_and_logical_target_and_reuse_schema_compilation() {
    for (target, valid) in [("raw", false), ("logical", true)] {
        let rule = rule(
            json!({"const":true}),
            json!([{"kind":"constraint","field":{"field":"DETAIL"},"constraint":{"kind":"regex","pattern":"^A$","match":"required","target":target}}]),
        );
        let schema = compile(value(json!([rule]))).unwrap();
        assert_eq!(
            matches!(evaluate(&schema, b"A  05"), RuleValidationResult::Satisfied),
            valid
        );
    }
    let rule = rule(
        json!({"const":true}),
        json!([{"kind":"constraint","field":{"field":"DETAIL"},"constraint":{"kind":"regex","pattern":"[","match":"required","target":"logical"}}]),
    );
    let error = compile(value(json!([rule]))).unwrap_err();
    assert!(
        error
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::InvalidRegex)
    );
    assert!(
        !error
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::InternalCompilerInvariant)
    );
}
#[test]
fn byte_comparisons_cover_six_operators_and_integer_comparisons_are_exact() {
    for (operator, expected) in [
        ("==", [false, true, false]),
        ("!=", [true, false, true]),
        ("<", [true, false, false]),
        ("<=", [true, true, false]),
        (">", [false, false, true]),
        (">=", [false, true, true]),
    ] {
        let rule = rule(
            json!({"const":true}),
            json!([{"kind":"field_compare","left":{"ref":{"field":"DETAIL"}},"operator":operator,"right":{"literal":"B"},"value_type":"bytes"}]),
        );
        let schema = compile(value(json!([rule]))).unwrap();
        for (bytes, valid) in [b"A  05", b"B  05", b"C  05"].iter().zip(expected) {
            assert_eq!(
                matches!(evaluate(&schema, *bytes), RuleValidationResult::Satisfied),
                valid
            );
        }
    }
    let assertion:Value=serde_json::from_str(r#"{"kind":"field_compare","left":{"ref":{"field":"ORIGIN"}},"operator":"<","right":{"literal":340282366920938463463374607431768211456},"value_type":"integer"}"#).unwrap();
    let schema = compile(value(json!([rule(
        json!({"const":true}),
        json!([assertion])
    )])))
    .unwrap();
    assert_eq!(evaluate(&schema, b"A  05"), RuleValidationResult::Satisfied);
    assert_eq!(
        evaluate(&schema, b"A  XX"),
        RuleValidationResult::Unknown(UnknownReason::InvalidFieldValue)
    );
}
#[test]
fn date_comparison_uses_calendar_order_and_invalid_values_are_not_false() {
    let rule = rule(
        json!({"const":true}),
        json!([{"kind":"field_compare","left":{"ref":{"field":"DETAIL"}},"operator":"<","right":{"literal":"01032026"},"value_type":"date"}]),
    );
    let mut value = value(json!([rule]));
    value["traces"][0]["record_length"] = json!(10);
    value["traces"][0]["fields"][0]["physical"] = json!({"type":"date","start":1,"length":8,"alignment":{"kind":"exact"},"format":"DDMMYYYY"});
    value["traces"][0]["fields"][1]["physical"]["start"] = json!(9);
    let schema = compile(value).unwrap();
    assert_eq!(
        evaluate(&schema, b"2802202605"),
        RuleValidationResult::Satisfied
    );
    assert!(matches!(
        evaluate(&schema, b"0203202605"),
        RuleValidationResult::Violated(_)
    ));
    assert_eq!(
        evaluate(&schema, b"3102202605"),
        RuleValidationResult::Unknown(UnknownReason::InvalidFieldValue)
    );
}
#[test]
fn concat_and_condition_true_have_real_executors() {
    let rule = rule(
        json!({"const":true}),
        json!([{"kind":"equals_concat","field":{"field":"DETAIL"},"parts":[{"field":"ORIGIN"},{"field":"ORIGIN"}]}]),
    );
    let mut value = value(json!([rule]));
    value["traces"][0]["record_length"] = json!(6);
    value["traces"][0]["fields"][0]["physical"]["length"] = json!(4);
    value["traces"][0]["fields"][1]["physical"]["start"] = json!(5);
    let schema = compile(value).unwrap();
    assert_eq!(
        evaluate(&schema, b"010101"),
        RuleValidationResult::Satisfied
    );
    assert!(matches!(
        evaluate(&schema, b"010201"),
        RuleValidationResult::Violated(_)
    ));
    let mut rule = presence_rule();
    rule["when"] = json!({"const":true});
    rule["message_template"] = json!("Condizione {rule_id}");
    rule["assertions"] = json!([{"kind":"condition_true","condition":{"eq":{"ref":{"field":"ORIGIN"},"value":"05"}}}]);
    let schema = compile(self::value(json!([rule]))).unwrap();
    assert_eq!(evaluate(&schema, b"A  05"), RuleValidationResult::Satisfied);
    let (_, errors, _) = observe(&schema, b"A  01").unwrap();
    assert!(errors[0].field.is_none());
    assert_eq!(errors[0].context[0].field_id.0, "ORIGIN");
}
#[test]
fn malformed_templates_types_references_and_future_assertions_fail_closed() {
    for template in ["{unknown}", "{rule_id", "stray }", ""] {
        let mut rule = presence_rule();
        rule["message_template"] = json!(template);
        assert!(
            compile(value(json!([rule])))
                .unwrap_err()
                .issues
                .iter()
                .any(|issue| issue.code == SchemaIssueCode::InvalidSchemaValue)
        );
    }
    for assertion in [
        json!({"kind":"field_present","field":{"field":"ABSENT"}}),
        json!({"kind":"derived_compare","derived_value":"future","operator":"==","value":1}),
        json!({"kind":"field_compare","left":{"ref":{"field":"DETAIL"}},"operator":"==","right":{"literal":1},"value_type":"integer"}),
    ] {
        assert!(
            compile(value(json!([rule(
                json!({"const":false}),
                json!([assertion])
            )])))
            .is_err()
        );
    }
}

#[test]
fn range_domain_assertions_and_prior_field_errors_follow_real_stage_order() {
    for (constraint, target, valid, invalid) in [
        (
            json!({"kind":"numeric_range","min":1,"max":4}),
            "ORIGIN",
            b"A  04".as_slice(),
            b"A  05".as_slice(),
        ),
        (
            json!({"kind":"allowed_values","target":"raw","values":["A  "]}),
            "DETAIL",
            b"A  05",
            b"B  05",
        ),
    ] {
        let schema = compile(value(json!([rule(
            json!({"const":true}),
            json!([{"kind":"constraint","field":{"field":target},"constraint":constraint}])
        )])))
        .unwrap();
        assert_eq!(evaluate(&schema, valid), RuleValidationResult::Satisfied);
        assert!(matches!(
            evaluate(&schema, invalid),
            RuleValidationResult::Violated(_)
        ));
    }
    let schema = compile(value(json!([presence_rule()]))).unwrap();
    let (_, errors, _) = observe(&schema, b"   XX").unwrap();
    assert_eq!(
        errors
            .iter()
            .map(|error| error.code.clone())
            .collect::<Vec<_>>(),
        [
            ValidationCode::InvalidType,
            ValidationCode::ConditionalRequired
        ]
    );
}
#[test]
fn known_assertion_failure_is_not_hidden_by_an_earlier_unknown() {
    let mut rule = rule(
        json!({"const":true}),
        json!([
            {"kind":"condition_true","condition":{"eq":{"ref":{"field":"ORIGIN"},"value":"05"}}},
            {"kind":"field_present","field":{"field":"DETAIL"}}
        ]),
    );
    rule["message_template"] = json!("Regola {rule_id}");
    let schema = compile(value(json!([rule]))).unwrap();
    assert!(matches!(
        evaluate(&schema, b"     "),
        RuleValidationResult::Violated(_)
    ));
    let (_, errors, _) = observe(&schema, b"     ").unwrap();
    assert_eq!(errors[0].field.as_ref().unwrap().id.0, "DETAIL");
}
#[test]
fn interpolation_is_single_pass_and_rule_metadata_errors_are_compile_time() {
    let mut input = value(json!([presence_rule()]));
    input["traces"][0]["fields"][0]["name"] = json!("Detail {rule_id}");
    let schema = compile(input).unwrap();
    let (_, errors, _) = observe(&schema, b"   05").unwrap();
    assert!(errors[0].message.contains("(Detail {rule_id})"));
    assert!(
        compile(value(json!([presence_rule(), presence_rule()])))
            .unwrap_err()
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::InvalidSchemaValue)
    );
    let mut rule = presence_rule();
    rule["error_code"] = json!("E001");
    assert!(
        compile(value(json!([rule])))
            .unwrap_err()
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::InvalidDiagnosticCode)
    );
}

#[test]
fn integer_field_values_beyond_machine_limits_are_compared_without_loss() {
    let assertion:Value=serde_json::from_str(r#"{"kind":"field_compare","left":{"ref":{"field":"ORIGIN"}},"operator":"==","right":{"literal":340282366920938463463374607431768211456},"value_type":"integer"}"#).unwrap();
    let mut value = value(json!([rule(json!({"const":true}), json!([assertion]))]));
    value["traces"][0]["record_length"] = json!(42);
    value["traces"][0]["fields"][1]["physical"]["length"] = json!(39);
    let schema = compile(value).unwrap();
    assert_eq!(
        evaluate(&schema, b"A  340282366920938463463374607431768211456"),
        RuleValidationResult::Satisfied
    );
    assert!(matches!(
        evaluate(&schema, b"A  340282366920938463463374607431768211457"),
        RuleValidationResult::Violated(_)
    ));
}

#[test]
fn directly_constructed_future_or_invalid_plans_are_not_hidden_by_false_guards() {
    let schema = compile(value(json!([presence_rule()]))).unwrap();
    let view = RecordView::new(b"A  05");
    let local = LocalEvaluationContext {
        record: &view,
        schema: &schema.traces()[0],
    };
    let ctx = EvaluationContext { fields: &local };
    let mut plan = schema.traces()[0].local_rules()[0].clone();
    plan.when = CompiledExpr::Const(false);
    plan.assertions = vec![CompiledAssertion::FieldCompare {
        left: CompiledOperand::Literal(CompiledValue::Date(DateLiteral {
            year: 0,
            month: 1,
            day: 1,
        })),
        right: CompiledOperand::Literal(CompiledValue::Date(DateLiteral {
            year: 2026,
            month: 1,
            day: 1,
        })),
        operator: ComparisonOperator::Lt,
        value_type: ValueType::Date,
    }];
    plan.message_template = "Regola {rule_id}".into();
    assert_eq!(
        validate_local_rule(&plan, &ctx),
        RuleValidationResult::Unknown(UnknownReason::UnsupportedPlan)
    );
    plan.assertions = vec![CompiledAssertion::RecordExists { join_index: 0 }];
    assert_eq!(
        validate_local_rule(&plan, &ctx),
        RuleValidationResult::Unknown(UnknownReason::UnsupportedPlan)
    );
}
