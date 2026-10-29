//! End-to-end DSL lowering, physical-order independence and provable conflicts.
use lintra::*;
use serde_json::{Value, json};
fn value() -> Value {
    serde_json::from_slice(include_bytes!("golden/dsl/v1/schema.json")).unwrap()
}
fn compile(value: Value) -> Result<CompiledFlowSchema, SchemaCompilationError> {
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw)
}
fn errors(
    schema: &CompiledFlowSchema,
    bytes: &[u8],
) -> Result<(bool, Vec<ValidationError>), RecordValidationError> {
    let trace = &schema.traces()[0];
    let mut sink = VecSink::default();
    let mut summary = ValidationSummaryBuilder::new(
        schema.key().clone(),
        ScopeSummary::Trace {
            trace_id: trace.id().clone(),
        },
    );
    let result = RecordValidator::new(schema, trace.id())?.validate_local(
        RawRecord {
            line_number: 1,
            bytes,
        },
        &mut sink,
        &mut summary,
    )?;
    // This direct record test does not publish a run summary or invent audits.
    Ok((result.valid, sink.errors().to_vec()))
}
#[test]
fn every_raw_operator_is_compiled_and_executed_with_three_valued_semantics() {
    use TruthValue::{False as F, True as T, Unknown as U};
    let unknown = U(UnknownReason::FieldUnavailable);
    let equal = json!({"eq":{"ref":{"field":"ORIGIN"},"value":"05"}});
    for (expression, present, missing) in [
        (equal.clone(), T, unknown),
        (
            json!({"ne":{"ref":{"field":"ORIGIN"},"value":"05"}}),
            F,
            unknown,
        ),
        (
            json!({"in":{"ref":{"field":"ORIGIN"},"values":["01","05"]}}),
            T,
            unknown,
        ),
        (
            json!({"not_in":{"ref":{"field":"ORIGIN"},"values":["01","02"]}}),
            T,
            unknown,
        ),
        (json!({"is_present":{"ref":{"field":"ORIGIN"}}}), T, F),
        (json!({"is_missing":{"ref":{"field":"ORIGIN"}}}), F, T),
        (json!({"and":[{"const":false},equal.clone()]}), F, F),
        (json!({"or":[{"const":true},equal.clone()]}), T, T),
        (json!({"not":equal}), F, unknown),
        (json!({"const":true}), T, T),
    ] {
        let mut value = value();
        value["traces"][0]["rules"][0]["when"] = expression;
        let schema = compile(value).unwrap();
        for (bytes, expected) in [(b"A  05".as_slice(), present), (b"A    ", missing)] {
            let view = RecordView::new(bytes);
            let local = LocalEvaluationContext {
                record: &view,
                schema: &schema.traces()[0],
            };
            assert_eq!(
                ExpressionEvaluator.evaluate(
                    &schema.traces()[0].local_rules()[0].when,
                    &EvaluationContext { fields: &local }
                ),
                expected
            );
        }
    }
}
#[test]
fn equivalent_physical_field_orders_preserve_rule_decisions_messages_and_context() {
    let first = compile(value()).unwrap();
    let mut reversed = value();
    let fields = reversed["traces"][0]["fields"].as_array_mut().unwrap();
    fields.reverse();
    fields[0]["physical"]["start"] = json!(1);
    fields[1]["physical"]["start"] = json!(3);
    let second = compile(reversed).unwrap();
    for (left, right, valid) in [
        (b"   05".as_slice(), b"05   ".as_slice(), false),
        (b"X  05", b"05X  ", false),
        (b"A  05", b"05A  ", true),
        (b"   01", b"01   ", true),
    ] {
        let (left_valid, left) = errors(&first, left).unwrap();
        let (right_valid, right) = errors(&second, right).unwrap();
        assert_eq!(left_valid, valid);
        assert_eq!(right_valid, valid);
        assert_eq!(left.len(), right.len());
        for (left, right) in left.iter().zip(&right) {
            assert_eq!(left.code, right.code);
            assert_eq!(left.actual, right.actual);
            assert_eq!(left.context, right.context);
            assert_eq!(left.message, right.message);
            assert_eq!(left.field.is_some(), right.field.is_some());
            if left.field.is_some() {
                let guide = |schema: &CompiledFlowSchema, error: &ValidationError| {
                    let field = schema
                        .trace(&error.trace_id)
                        .unwrap()
                        .field(&error.field.as_ref().unwrap().id)
                        .unwrap();
                    lintra::field_format_guide(field, schema.encoding())
                };
                let left_guide = guide(&first, left);
                let right_guide = guide(&second, right);
                assert!(left.expected.as_deref().unwrap().contains(&left_guide));
                assert!(right.expected.as_deref().unwrap().contains(&right_guide));
                assert_eq!(
                    left.expected
                        .as_deref()
                        .unwrap()
                        .replacen(&left_guide, "[physical guide]", 1),
                    right.expected.as_deref().unwrap().replacen(
                        &right_guide,
                        "[physical guide]",
                        1
                    )
                );
            } else {
                // Record-level assertions have no physical field guide.
                assert_eq!(left.expected, right.expected);
            }
            if let (Some(left), Some(right)) = (&left.field, &right.field) {
                assert_eq!(left.id, right.id);
                assert_eq!(left.start_one_based, 1);
                assert_eq!(right.start_one_based, 3);
            }
        }
    }
    assert!(matches!(
        errors(&first, b"A    "),
        Err(RecordValidationError::UndeterminedRule {
            reason: UnknownReason::FieldUnavailable,
            ..
        })
    ));
    assert!(matches!(
        errors(&second, b"  A  "),
        Err(RecordValidationError::UndeterminedRule {
            reason: UnknownReason::FieldUnavailable,
            ..
        })
    ));
}
#[test]
fn unconditional_presence_conflicts_are_schema_errors_with_field_context() {
    let dynamic = json!({"is_present":{"ref":{"field":"ORIGIN"}}});
    for (base, required, forbidden) in [
        ("required", json!([]), json!([{"const":true}])),
        ("optional", json!([{"const":true}]), json!([{"const":true}])),
        (
            "optional",
            json!([{"or":[dynamic.clone(),{"const":true}]}]),
            json!([{"not":{"const":false}}]),
        ),
    ] {
        let mut value = value();
        value["traces"][0]["fields"][0]["presence"] =
            json!({"base":base,"required_if":required,"forbidden_if":forbidden});
        let error = compile(value).unwrap_err();
        assert!(
            error
                .issues
                .iter()
                .any(|issue| issue.code == SchemaIssueCode::PresenceConflict
                    && issue.trace_id.as_ref().is_some_and(|id| id.0 == "T1")
                    && issue.field_id.as_ref().is_some_and(|id| id.0 == "DETAIL"))
        );
        assert!(
            !error
                .issues
                .iter()
                .any(|issue| issue.code == SchemaIssueCode::InternalCompilerInvariant)
        );
    }
    let mut value = value();
    value["traces"][0]["named_conditions"] = json!({"ALWAYS":{"not":{"const":false}}});
    value["traces"][0]["fields"][0]["presence"] =
        json!({"base":"required","required_if":[],"forbidden_if":[{"condition":"ALWAYS"}]});
    assert!(
        compile(value)
            .unwrap_err()
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::PresenceConflict)
    );
}
#[test]
fn dynamic_conflicts_remain_runtime_errors_and_decisive_false_is_not_a_conflict() {
    let condition = json!({"eq":{"ref":{"field":"ORIGIN"},"value":"05"}});
    let mut value = value();
    value["traces"][0]["fields"][0]["presence"] =
        json!({"base":"optional","required_if":[condition.clone()],"forbidden_if":[condition]});
    let schema = compile(value).unwrap();
    let (_, errors) = errors(&schema, b"A  05").unwrap();
    assert_eq!(errors[0].code, ValidationCode::RuleConflict);
    let mut value = self::value();
    value["traces"][0]["fields"][0]["presence"] = json!({"base":"required","required_if":[],"forbidden_if":[{"and":[{"is_present":{"ref":{"field":"ORIGIN"}}},{"const":false}]}]});
    compile(value).unwrap();
}
