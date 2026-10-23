//! Private fixtures exercise production presence/record paths while the public
//! compiler remains closed until derived rule diagnostics are delivered.
use super::*;
use crate::*;
use serde_json::{Value, json};

fn schema() -> CompiledFlowSchema {
    let mut value: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/golden/synthetic/v1/schema.json"
    ))
    .unwrap();
    let base = value["traces"][0]["fields"][0].clone();
    let mut fields = vec![];
    for (id, start, length, kind) in [
        ("DETAIL", 1, 3, "alphanumeric"),
        ("BIRTH", 4, 8, "date"),
        ("REF", 12, 8, "date"),
        ("AGE", 20, 3, "numeric"),
    ] {
        let mut field = base.clone();
        field["id"] = json!(id);
        field["name"] = json!(id);
        field["physical"] = if kind == "date" {
            json!({"type":kind,"start":start,"length":length,"format":"DDMMYYYY","alignment":{"kind":"exact"}})
        } else {
            json!({"type":kind,"start":start,"length":length,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}})
        };
        field["missing_value"] = json!({"kind":"spaces"});
        fields.push(field);
    }
    value["traces"][0]["record_length"] = json!(22);
    value["traces"][0]["fields"] = json!(fields);
    value["traces"][0]["record_key"]["fields"] = json!(["DETAIL"]);
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let mut schema = SchemaCompiler.compile(raw).unwrap();
    schema.traces[0].derived_values = vec![
        DerivedValuePlan::AgeInCompletedYears {
            birth: reference(1),
            reference: reference(2),
            format: DateFormat::DdMmYyyy,
            leap_day: LeapDayAnniversary::Mar01,
        },
        DerivedValuePlan::CalendarYearDelta {
            birth: reference(1),
            reference: reference(2),
            format: DateFormat::DdMmYyyy,
        },
        DerivedValuePlan::Substring {
            source: reference(3),
            target: ValueTarget::Raw,
            range: ByteRange::new(0, 3).unwrap(),
        },
    ];
    schema
}
fn reference(index: usize) -> ResolvedFieldRef {
    ResolvedFieldRef::Local {
        field_index: index,
        field_id: FieldId(["DETAIL", "BIRTH", "REF", "AGE"][index].into()),
    }
}
fn adult() -> CompiledExpr {
    CompiledExpr::DerivedCompare {
        derived_index: 0,
        operator: ComparisonOperator::Ge,
        value: CompiledValue::Integer("18".parse().unwrap()),
    }
}
fn bytes(detail: &str, birth: &str, reference: &str, age: &str) -> Vec<u8> {
    format!("{detail}{birth}{reference}{age}").into_bytes()
}
fn state(schema: &CompiledFlowSchema, bytes: &[u8]) -> PresenceState {
    let view = RecordView::new(bytes);
    let local = LocalEvaluationContext {
        record: &view,
        schema: &schema.traces[0],
    };
    resolve_presence(
        &schema.traces[0].fields[0].presence_policy,
        &EvaluationContext { fields: &local },
    )
}

#[test]
fn real_derived_context_preserves_every_presence_state_and_base_obligation() {
    let mut schema = schema();
    let policy = &mut schema.traces[0].fields[0].presence_policy;
    policy.required_if = vec![adult()];
    assert_eq!(
        state(&schema, &bytes("ABC", "01012000", "01012021", "021")),
        PresenceState::Required
    );
    assert_eq!(
        state(&schema, &bytes("ABC", "01012010", "01012021", "011")),
        PresenceState::Optional
    );
    assert_eq!(
        state(&schema, &bytes("ABC", "        ", "01012021", "021")),
        PresenceState::Unknown(UnknownReason::FieldUnavailable)
    );
    assert_eq!(
        state(&schema, &bytes("ABC", "31022000", "01012021", "021")),
        PresenceState::Unknown(UnknownReason::InvalidFieldValue)
    );
    schema.traces[0].fields[0].presence_policy.forbidden_if = vec![adult()];
    assert_eq!(
        state(&schema, &bytes("ABC", "01012000", "01012021", "021")),
        PresenceState::Conflict
    );
    schema.traces[0].fields[0]
        .presence_policy
        .required_if
        .clear();
    assert_eq!(
        state(&schema, &bytes("ABC", "01012000", "01012021", "021")),
        PresenceState::Forbidden
    );
    schema.traces[0].fields[0].presence_policy.base = BasePresence::Required;
    assert_eq!(
        state(&schema, &bytes("ABC", "01012000", "01012021", "021")),
        PresenceState::Conflict
    );
}

#[test]
fn record_accepts_real_derived_plans_and_required_missing_is_invalid() {
    let mut schema = schema();
    schema.traces[0].fields[0].presence_policy.required_if = vec![adult()];
    for (detail, birth, valid) in [
        ("ABC", "01012000", true),
        ("   ", "01012000", false),
        ("   ", "01012010", true),
    ] {
        let bytes = bytes(detail, birth, "01012021", "021");
        let before = bytes.clone();
        assert_eq!(run_private_record(&schema, &bytes).unwrap().valid, valid);
        assert_eq!(bytes, before);
    }
}

#[test]
fn derived_unknown_stops_record_and_validator_remains_terminal() {
    let mut schema = schema();
    schema.traces[0].fields[0].presence_policy.required_if = vec![adult()];
    let mut validator = RecordValidator::new(&schema, schema.traces[0].id()).unwrap();
    let mut sink = VecSink::default();
    let mut summary = ValidationSummaryBuilder::new(
        schema.key().clone(),
        ScopeSummary::Trace {
            trace_id: schema.traces[0].id().clone(),
        },
    );
    let input = bytes("ABC", "31022000", "01012021", "021");
    assert!(
        matches!(validator.validate_local(RawRecord { line_number: 1, bytes: &input }, &mut sink, &mut summary), Err(RecordValidationError::UndeterminedPresence { field_id, reason: UnknownReason::InvalidFieldValue }) if field_id.0 == "DETAIL")
    );
    assert!(sink.errors().is_empty());
    let input = bytes("ABC", "01012000", "01012021", "021");
    assert!(matches!(
        validator.validate_local(
            RawRecord {
                line_number: 2,
                bytes: &input
            },
            &mut sink,
            &mut summary
        ),
        Err(RecordValidationError::PreviousFailure)
    ));
}

#[test]
fn field_pipeline_reports_primary_presence_and_counts_unknown_once() {
    for (required, forbidden, detail, expected) in [
        (
            true,
            false,
            "   ",
            Some(ValidationCode::ConditionalRequired),
        ),
        (false, true, "ABC", Some(ValidationCode::ForbiddenField)),
        (true, true, "   ", Some(ValidationCode::RuleConflict)),
        (true, true, "ABC", Some(ValidationCode::RuleConflict)),
        (true, false, "ABC", None),
    ] {
        for invalid_source in [false, true] {
            let mut schema = schema();
            let policy = &mut schema.traces[0].fields[0].presence_policy;
            if required {
                policy.required_if.push(adult());
            }
            if forbidden {
                policy.forbidden_if.push(adult());
            }
            let input = bytes(
                detail,
                if invalid_source {
                    "31022000"
                } else {
                    "01012000"
                },
                "01012021",
                "021",
            );
            let view = RecordView::new(&input);
            let trace = &schema.traces[0];
            let local = LocalEvaluationContext {
                record: &view,
                schema: trace,
            };
            let mut sink = VecSink::default();
            let mut summary = ValidationSummaryBuilder::new(
                schema.key().clone(),
                ScopeSummary::Trace {
                    trace_id: trace.id().clone(),
                },
            );
            let outcome = FieldPipeline::new(
                ErrorFactory {
                    flow_id: &schema.key().flow_id,
                    schema_version: &schema.key().version,
                    trace_id: trace.id(),
                    encoding: schema.encoding(),
                },
                &mut sink,
                &mut summary,
            )
            .run_with_context(
                &trace.fields[0],
                view.field(&trace.fields[0]).unwrap(),
                1,
                &RecordIdentity::default(),
                &EvaluationContext { fields: &local },
            )
            .unwrap();
            assert_eq!(
                sink.errors()
                    .iter()
                    .map(|e| e.code.clone())
                    .collect::<Vec<_>>(),
                if invalid_source {
                    vec![]
                } else {
                    expected.clone().into_iter().collect()
                }
            );
            assert_eq!(
                outcome,
                if invalid_source {
                    FieldPipelineResult::SkippedDueToUnknownPresence
                } else if expected.is_some() {
                    FieldPipelineResult::Invalid
                } else {
                    FieldPipelineResult::Valid
                }
            );
            let totals = summary
                .finish(RunMetadata {
                    validation_mode: ValidationMode::LocalOnly,
                    validator_version: "test".into(),
                    schema_hash: "test".into(),
                    reference_data_versions: Default::default(),
                    started_at: std::time::UNIX_EPOCH,
                    inputs: vec![],
                })
                .unwrap();
            assert_eq!(totals.skipped_rules, u64::from(invalid_source));
        }
    }
}

#[test]
fn all_derived_types_compose_under_not_and_or_without_changing_sources() {
    let mut schema = schema();
    let same_age = CompiledExpr::DerivedEq {
        field: reference(3),
        derived_index: 0,
    };
    let substring = CompiledExpr::DerivedCompare {
        derived_index: 2,
        operator: ComparisonOperator::Eq,
        value: CompiledValue::Bytes(b"021".to_vec()),
    };
    let year = CompiledExpr::DerivedCompare {
        derived_index: 1,
        operator: ComparisonOperator::Eq,
        value: CompiledValue::Integer("21".parse().unwrap()),
    };
    schema.traces[0].fields[0].presence_policy.required_if =
        vec![CompiledExpr::And(vec![same_age, substring, year])];
    let input = bytes("ABC", "01012000", "01012021", "021");
    assert_eq!(state(&schema, &input), PresenceState::Required);
    assert!(run_private_record(&schema, &input).unwrap().valid);
    schema.traces[0].fields[0].presence_policy.required_if =
        vec![CompiledExpr::Not(Box::new(adult()))];
    assert_eq!(state(&schema, &input), PresenceState::Optional);
    schema.traces[0].fields[0].presence_policy.required_if =
        vec![CompiledExpr::Or(vec![adult(), CompiledExpr::Const(true)])];
    assert_eq!(
        state(&schema, &bytes("ABC", "        ", "01012021", "021")),
        PresenceState::Required
    );
}

#[test]
fn invalid_metadata_is_rejected_even_when_a_constant_dominates_the_expression() {
    for bad in 0..6 {
        let mut schema = schema();
        let expr = match bad {
            0 => CompiledExpr::DerivedCompare {
                derived_index: 90,
                operator: ComparisonOperator::Eq,
                value: CompiledValue::Integer("18".parse().unwrap()),
            },
            1 => CompiledExpr::DerivedCompare {
                derived_index: 0,
                operator: ComparisonOperator::Eq,
                value: CompiledValue::Bytes(b"18".to_vec()),
            },
            2 => CompiledExpr::DerivedEq {
                field: reference(0),
                derived_index: 0,
            },
            3 => CompiledExpr::DerivedEq {
                field: ResolvedFieldRef::Local {
                    field_index: 3,
                    field_id: FieldId("WRONG".into()),
                },
                derived_index: 0,
            },
            4 => {
                schema.traces[0].derived_values[0] = DerivedValuePlan::CalendarYearDelta {
                    birth: reference(0),
                    reference: reference(2),
                    format: DateFormat::DdMmYyyy,
                };
                adult()
            }
            _ => {
                schema.traces[0].derived_values[2] = DerivedValuePlan::Substring {
                    source: reference(3),
                    target: ValueTarget::Logical,
                    range: ByteRange::new(0, 4).unwrap(),
                };
                adult()
            }
        };
        schema.traces[0].fields[0].presence_policy.required_if =
            vec![CompiledExpr::Or(vec![CompiledExpr::Const(true), expr])];
        assert!(matches!(
            RecordValidator::new(&schema, schema.traces[0].id()),
            Err(RecordValidationError::UnsupportedLocalPlan { .. })
        ));
    }
}
