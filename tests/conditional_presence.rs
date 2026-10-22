//! Conditional policies compiled and executed against actual local record bytes.
use lintra::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, time::UNIX_EPOCH};
fn schema(base: &str, required: Value, forbidden: Value) -> CompiledFlowSchema {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("fixtures/application/schema.json")).unwrap();
    value["required_capabilities"] = json!(["local.v1", "expressions.v1"]);
    value["traces"][0]["fields"][0]["presence"] =
        json!({"base":base,"required_if":required,"forbidden_if":forbidden});
    value["traces"][0]["fields"][1]["presence"]["base"] = json!("optional");
    for field in value["traces"][0]["fields"].as_array_mut().unwrap() {
        field["constraints"] = json!([]);
    }
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw).unwrap()
}
fn controller() -> Value {
    json!({"eq":{"ref":{"field":"F2"},"value":"A"}})
}
fn builder(schema: &CompiledFlowSchema) -> ValidationSummaryBuilder {
    ValidationSummaryBuilder::new(
        schema.key().clone(),
        ScopeSummary::Trace {
            trace_id: schema.traces()[0].id().clone(),
        },
    )
}
fn metadata() -> RunMetadata {
    RunMetadata {
        validation_mode: ValidationMode::LocalOnly,
        validator_version: "synthetic".into(),
        schema_hash: "synthetic".into(),
        reference_data_versions: BTreeMap::new(),
        started_at: UNIX_EPOCH,
        inputs: vec![],
    }
}
fn field(
    schema: &CompiledFlowSchema,
    bytes: &[u8],
) -> (FieldPipelineResult, Vec<ValidationCode>, ValidationSummary) {
    let trace = &schema.traces()[0];
    let record = RecordView::new(bytes);
    let local = LocalEvaluationContext {
        record: &record,
        schema: trace,
    };
    let ctx = EvaluationContext { fields: &local };
    let mut sink = VecSink::default();
    let mut summary = builder(schema);
    let factory = ErrorFactory {
        flow_id: &schema.key().flow_id,
        schema_version: &schema.key().version,
        trace_id: trace.id(),
        encoding: schema.encoding(),
    };
    let result = FieldPipeline::new(factory, &mut sink, &mut summary)
        .run_with_context(
            &trace.fields()[0],
            record.field(&trace.fields()[0]).unwrap(),
            1,
            &RecordIdentity::default(),
            &ctx,
        )
        .unwrap();
    (
        result,
        sink.errors().iter().map(|e| e.code.clone()).collect(),
        summary.finish(metadata()).unwrap(),
    )
}
#[test]
fn presence_state_matrix_covers_base_and_every_required_forbidden_truth_combination() {
    use PresenceState::*;
    let unknown = Unknown(UnknownReason::FieldUnavailable);
    let optional = [
        [Optional, Forbidden, unknown],
        [Required, Conflict, unknown],
        [unknown, unknown, unknown],
    ];
    let conditions = [
        json!({"const":false}),
        json!({"is_missing":{"ref":{"field":"F2"}}}),
        controller(),
    ];
    for base in ["optional", "required"] {
        for (row, required) in conditions.iter().enumerate() {
            for (column, forbidden) in conditions.iter().enumerate() {
                let schema = schema(base, json!([required]), json!([forbidden]));
                let record = RecordView::new(b"001   ");
                let local = LocalEvaluationContext {
                    record: &record,
                    schema: &schema.traces()[0],
                };
                let actual = resolve_presence(
                    schema.traces()[0].fields()[0].presence_policy(),
                    &EvaluationContext { fields: &local },
                );
                let expected = if base == "optional" {
                    optional[row][column]
                } else {
                    [Required, Conflict, unknown][column]
                };
                assert_eq!(actual, expected);
            }
        }
    }
}
#[test]
fn required_forbidden_and_conflict_stop_before_datatype_and_domains() {
    for (base, required, forbidden, input, expected) in [
        (
            "optional",
            json!([controller()]),
            json!([]),
            b"   A  ".as_slice(),
            ValidationCode::ConditionalRequired,
        ),
        (
            "required",
            json!([]),
            json!([]),
            b"   A  ",
            ValidationCode::RequiredField,
        ),
        (
            "optional",
            json!([]),
            json!([controller()]),
            b"ABCA  ",
            ValidationCode::ForbiddenField,
        ),
        (
            "optional",
            json!([controller()]),
            json!([controller()]),
            b"ABCA  ",
            ValidationCode::RuleConflict,
        ),
    ] {
        let schema = schema(base, required, forbidden);
        let (outcome, errors, summary) = field(&schema, input);
        assert_eq!(outcome, FieldPipelineResult::Invalid);
        assert_eq!(errors, [expected]);
        assert_eq!(summary.total_errors, 1);
    }
    let schema = schema("optional", json!([]), json!([controller()]));
    assert_eq!(
        field(&schema, b"   A  ").0,
        FieldPipelineResult::MissingOptional
    );
}
#[test]
fn unknown_stops_field_and_counts_one_skip_without_fabricating_violation() {
    let schema = schema("optional", json!([controller()]), json!([]));
    let (outcome, errors, summary) = field(&schema, b"ABC   ");
    assert_eq!(outcome, FieldPipelineResult::SkippedDueToUnknownPresence);
    assert!(errors.is_empty());
    assert_eq!(summary.skipped_rules, 1);
    assert_eq!(summary.total_errors, 0);
    // This field-only fixture has no classified records and makes no run-validity assertion.
}
#[test]
fn required_unless_is_explicit_not_and_conditions_are_retained() {
    let schema = schema("optional", json!([{"not":controller()}]), json!([]));
    assert_eq!(
        schema.traces()[0].fields()[0]
            .presence_policy()
            .required_if
            .len(),
        1
    );
    assert_eq!(
        field(&schema, b"   A  ").0,
        FieldPipelineResult::MissingOptional
    );
    assert_eq!(
        field(&schema, b"   B  ").1,
        [ValidationCode::ConditionalRequired]
    );
    assert_eq!(field(&schema, b"001B  ").0, FieldPipelineResult::Valid);
}
#[test]
fn required_condition_reads_later_field_without_mutating_it() {
    let schema = schema("optional", json!([controller()]), json!([]));
    for (bytes, valid, errors) in [
        (b"001A  ".as_slice(), true, 0),
        (b"   A  ", false, 1),
        (b"   B  ", true, 0),
    ] {
        let original = bytes.to_vec();
        let mut sink = VecSink::default();
        let mut summary = builder(&schema);
        let mut validator = RecordValidator::new(&schema, schema.traces()[0].id()).unwrap();
        let result = validator
            .validate_local(
                RawRecord {
                    line_number: 1,
                    bytes,
                },
                &mut sink,
                &mut summary,
            )
            .unwrap();
        assert_eq!(result.valid, valid);
        assert_eq!(sink.errors().len(), errors);
        assert_eq!(bytes, original);
        let audits = InputAuditSession::new();
        let mut reader = audits
            .wrap(
                schema.traces()[0].id().clone(),
                "synthetic.txt".into(),
                bytes.len().try_into().unwrap(),
                std::io::Cursor::new(bytes),
            )
            .unwrap();
        std::io::copy(&mut reader, &mut std::io::sink()).unwrap();
        drop(reader);
        let summary = summary.finish(audits.finish(metadata()).unwrap()).unwrap();
        assert_eq!(summary.records_processed, 1);
    }
}
#[test]
fn undecidable_record_returns_typed_terminal_error_instead_of_valid_or_invalid() {
    let schema = schema("optional", json!([controller()]), json!([]));
    let mut sink = VecSink::default();
    let mut summary = builder(&schema);
    let mut validator = RecordValidator::new(&schema, schema.traces()[0].id()).unwrap();
    let error = validator
        .validate_local(
            RawRecord {
                line_number: 1,
                bytes: b"001   ",
            },
            &mut sink,
            &mut summary,
        )
        .unwrap_err();
    assert!(
        matches!(error,RecordValidationError::UndeterminedPresence{field_id,reason:UnknownReason::FieldUnavailable} if field_id.0=="F1")
    );
    assert!(sink.errors().is_empty());
    assert!(matches!(
        validator.validate_local(
            RawRecord {
                line_number: 2,
                bytes: b"001A  "
            },
            &mut sink,
            &mut summary
        ),
        Err(RecordValidationError::PreviousFailure)
    ));
}

#[test]
fn named_conditions_compile_into_the_actual_presence_plan() {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("fixtures/application/schema.json")).unwrap();
    value["required_capabilities"] = json!(["local.v1", "expressions.v1"]);
    value["traces"][0]["named_conditions"] = json!({"NEEDS_VALUE":controller()});
    value["traces"][0]["fields"][0]["presence"] =
        json!({"base":"optional","required_if":[{"condition":"NEEDS_VALUE"}],"forbidden_if":[]});
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let schema = SchemaCompiler.compile(raw).unwrap();
    assert!(matches!(
        schema.traces()[0].fields()[0].presence_policy().required_if[0],
        CompiledExpr::Eq(..)
    ));
    assert_eq!(
        field(&schema, b"   A  ").1,
        [ValidationCode::ConditionalRequired]
    );
}

#[test]
fn failed_pipeline_never_queries_another_context() {
    struct Failing;
    impl ValidationSink for Failing {
        fn on_error(&mut self, _: &ValidationError) -> Result<(), ReportError> {
            Err(ReportError::Write(std::io::Error::other(
                "synthetic failure",
            )))
        }
        fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
            Ok(())
        }
    }
    struct Unreachable;
    impl FieldValueResolver for Unreachable {
        fn resolve_field(&self, _: &ResolvedFieldRef) -> ResolvedFieldValue<'_> {
            panic!("context queried after terminal failure")
        }
    }
    let schema = schema("optional", json!([controller()]), json!([]));
    let trace = &schema.traces()[0];
    let record = RecordView::new(b"   A  ");
    let local = LocalEvaluationContext {
        record: &record,
        schema: trace,
    };
    let factory = ErrorFactory {
        flow_id: &schema.key().flow_id,
        schema_version: &schema.key().version,
        trace_id: trace.id(),
        encoding: schema.encoding(),
    };
    let mut sink = Failing;
    let mut summary = builder(&schema);
    let mut pipeline = FieldPipeline::new(factory, &mut sink, &mut summary);
    assert!(matches!(
        pipeline.run_with_context(
            &trace.fields()[0],
            FieldSlice { raw: b"   " },
            1,
            &RecordIdentity::default(),
            &EvaluationContext { fields: &local }
        ),
        Err(FieldPipelineError::Report(_))
    ));
    assert!(matches!(
        pipeline.run_with_context(
            &trace.fields()[0],
            FieldSlice { raw: b"   " },
            2,
            &RecordIdentity::default(),
            &EvaluationContext {
                fields: &Unreachable
            }
        ),
        Err(FieldPipelineError::PreviousFailure)
    ));
}
