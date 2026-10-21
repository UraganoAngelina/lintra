//! Whole local-stage paths with synthetic compiled fields and real sinks.
use lintra::{
    CompiledFlowSchema, ErrorFactory, FieldPipeline, FieldPipelineError,
    FieldPipelineResult as ResultKind, FieldSlice, RawFlowSchema, RawResolvedEncoding,
    RecordIdentity, ReportError, RunMetadata, SchemaCompiler, ScopeSummary, ValidationCode,
    ValidationError, ValidationSink, ValidationSummary, ValidationSummaryBuilder, VecSink,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, io, time::UNIX_EPOCH};

/// Two independent four-byte fields test both the pipeline and lack of mutable
/// cross-field flags. Every policy/domain is explicit in the synthetic JSON.
fn schema(base: &str, kind: &str, constraints: Value) -> CompiledFlowSchema {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    value["traces"][0]["record_length"] = json!(8);
    let mut field = value["traces"][0]["fields"][0].clone();
    field["physical"]["length"] = json!(4);
    field["physical"]["type"] = json!(kind);
    field["physical"]["alignment"] =
        json!({"kind":"right_pad","fill_byte":32,"content_policy":"strip_trailing_fill"});
    field["presence"]["base"] = json!(base);
    field["missing_value"] = json!({"kind":"spaces"});
    field["constraints"] = constraints;
    let mut second = field.clone();
    second["id"] = json!("F2");
    second["name"] = json!("Second synthetic field");
    second["physical"]["start"] = json!(5);
    value["traces"][0]["fields"] = json!([field, second]);
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw).unwrap()
}

/// Factory and counters use the same compiled flow/trace metadata as the caller.
fn factory(schema: &CompiledFlowSchema) -> ErrorFactory<'_> {
    ErrorFactory {
        flow_id: &schema.key().flow_id,
        schema_version: &schema.key().version,
        trace_id: schema.traces()[0].id(),
        encoding: schema.encoding(),
    }
}

/// Only error counters are exercised; record counters belong to T-P6-08.
fn builder(schema: &CompiledFlowSchema) -> ValidationSummaryBuilder {
    ValidationSummaryBuilder::new(
        schema.key().clone(),
        ScopeSummary::Trace {
            trace_id: schema.traces()[0].id().clone(),
        },
    )
}

/// A zero-record counter fixture needs no invented input audit. Fixed metadata
/// is sufficient to inspect actual emitted-error counts after a field-only test.
fn metadata() -> RunMetadata {
    RunMetadata {
        validation_mode: lintra::ValidationMode::LocalOnly,
        validator_version: "synthetic".into(),
        schema_hash: "synthetic".into(),
        reference_data_versions: BTreeMap::new(),
        started_at: UNIX_EPOCH,
        inputs: vec![],
    }
}

/// Execute one real field pipeline and expose only small test-owned results.
fn observe(
    schema: &CompiledFlowSchema,
    raw: &[u8],
) -> (ResultKind, Vec<ValidationError>, ValidationSummary) {
    let mut sink = VecSink::default();
    let mut summary = builder(schema);
    let result = FieldPipeline::new(factory(schema), &mut sink, &mut summary)
        .run(
            &schema.traces()[0].fields()[0],
            FieldSlice { raw },
            3,
            &RecordIdentity::default(),
        )
        .unwrap();
    (
        result,
        sink.errors().to_vec(),
        summary.finish(metadata()).unwrap(),
    )
}

/// Required blanks stop before extraction, numeric type and domains. Exactly
/// one presence error is emitted and counted, with original physical bytes.
#[test]
fn missing_required_has_no_datatype_or_domain_cascade() {
    let schema = schema(
        "required",
        "numeric",
        json!([{"kind":"allowed_values","target":"raw","values":["0012"]}]),
    );
    let (result, errors, summary) = observe(&schema, b"    ");
    assert_eq!(result, ResultKind::Invalid);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, ValidationCode::RequiredField);
    assert_eq!(errors[0].actual.as_ref().unwrap().display, "    ");
    assert_eq!(summary.total_errors, 1);
    assert_eq!(summary.errors_by_code[&ValidationCode::RequiredField], 1);
}

/// MissingOptional is successful early termination, not Present and not a
/// numeric/domain error. No error counter or report line is added.
#[test]
fn missing_optional_stops_successfully_without_further_validation() {
    let schema = schema(
        "optional",
        "numeric",
        json!([{"kind":"allowed_values","target":"raw","values":["0012"]}]),
    );
    let (result, errors, summary) = observe(&schema, b"    ");
    assert_eq!(result, ResultKind::MissingOptional);
    assert!(errors.is_empty());
    assert_eq!(summary.total_errors, 0);
}

/// A present but nonnumeric field stops at datatype, even though it also
/// violates the domain: the later failure must not create redundant output.
#[test]
fn invalid_datatype_stops_before_semantic_constraints() {
    let schema = schema(
        "required",
        "numeric",
        json!([{"kind":"allowed_values","target":"raw","values":["0012"]}]),
    );
    let (result, errors, summary) = observe(&schema, b"AB  ");
    assert_eq!(result, ResultKind::Invalid);
    assert_eq!(
        errors.iter().map(|e| &e.code).collect::<Vec<_>>(),
        [&ValidationCode::InvalidType]
    );
    assert_eq!(summary.total_errors, 1);
}

/// Logical domain selection does not change diagnostics: the reported actual
/// field must retain padding, line and one-based positions from the raw source.
#[test]
fn logical_domain_failure_reports_original_raw_value() {
    let schema = schema(
        "required",
        "alphanumeric",
        json!([{"kind":"allowed_values","target":"logical","values":["CD"]}]),
    );
    let (_, errors, _) = observe(&schema, b"AB  ");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, ValidationCode::InvalidDomain);
    assert_eq!(errors[0].actual.as_ref().unwrap().display, "AB  ");
    assert_eq!(errors[0].line_number, 3);
    let field = errors[0].field.as_ref().unwrap();
    assert_eq!(
        (field.start_one_based, field.end_one_based_inclusive),
        (1, 4)
    );
}

/// The first primary semantic failure stops the stage, per MIP §7. A field
/// failing two independent domains emits one error, in declared schema order.
#[test]
fn first_semantic_failure_stops_remaining_constraints() {
    let schema = schema(
        "required",
        "numeric",
        json!([
            {"kind":"allowed_values","target":"raw","values":["0012"]},
            {"kind":"allowed_values","target":"raw","values":["0014"]}
        ]),
    );
    let (_, errors, summary) = observe(&schema, b"0013");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, ValidationCode::InvalidDomain);
    assert_eq!(summary.total_errors, 1);
}

/// Both target modes have a successful path. No extra field diagnostic is
/// invented for the remaining leading whitespace explicitly allowed in text.
#[test]
fn successful_raw_and_logical_fields_have_no_errors() {
    for (kind, target, value, raw) in [
        ("numeric", "raw", "0012", b"0012".as_slice()),
        ("alphanumeric", "logical", " AB", b" AB "),
    ] {
        let schema = schema(
            "required",
            kind,
            json!([{"kind":"allowed_values","target":target,"values":[value]}]),
        );
        let (result, errors, summary) = observe(&schema, raw);
        assert_eq!(result, ResultKind::Valid);
        assert!(errors.is_empty());
        assert_eq!(summary.total_errors, 0);
    }
}

/// Normative Invalid is not terminal technical state. The next field retains
/// its own static policy and can succeed; no required flag is toggled anywhere.
#[test]
fn invalid_field_does_not_change_or_stop_the_next_field() {
    let schema = schema("required", "numeric", json!([]));
    let mut sink = VecSink::default();
    let mut summary = builder(&schema);
    let mut pipeline = FieldPipeline::new(factory(&schema), &mut sink, &mut summary);
    let fields = schema.traces()[0].fields();
    assert_eq!(
        pipeline
            .run(
                &fields[0],
                FieldSlice { raw: b"    " },
                1,
                &RecordIdentity::default()
            )
            .unwrap(),
        ResultKind::Invalid
    );
    assert_eq!(
        pipeline
            .run(
                &fields[1],
                FieldSlice { raw: b"0012" },
                1,
                &RecordIdentity::default()
            )
            .unwrap(),
        ResultKind::Valid
    );
    assert_eq!(sink.errors().len(), 1);
    assert_eq!(sink.errors()[0].field.as_ref().unwrap().id.0, "F1");
}

/// Synthetic sink counts attempts and fails before accepting a diagnostic.
/// This makes terminal/report-count behavior observable without OS privileges.
struct FailingSink {
    calls: usize,
}
impl ValidationSink for FailingSink {
    fn on_error(&mut self, _: &ValidationError) -> Result<(), ReportError> {
        self.calls += 1;
        Err(ReportError::Write(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "synthetic sink failure",
        )))
    }
    /// Summary is outside this field-only test and no summary is published.
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        Ok(())
    }
}

/// Preserve the original sink cause and forbid subsequent processing. A
/// rejected diagnostic must not be counted as successfully emitted.
#[test]
fn report_failure_is_typed_terminal_and_not_counted() {
    let schema = schema("required", "numeric", json!([]));
    let field = &schema.traces()[0].fields()[0];
    let mut sink = FailingSink { calls: 0 };
    let mut summary = builder(&schema);
    let mut pipeline = FieldPipeline::new(factory(&schema), &mut sink, &mut summary);
    let error = pipeline
        .run(
            field,
            FieldSlice { raw: b"    " },
            1,
            &RecordIdentity::default(),
        )
        .unwrap_err();
    assert!(
        matches!(error, FieldPipelineError::Report(ReportError::Write(source)) if source.kind() == io::ErrorKind::PermissionDenied)
    );
    assert!(matches!(
        pipeline.run(
            field,
            FieldSlice { raw: b"0012" },
            2,
            &RecordIdentity::default()
        ),
        Err(FieldPipelineError::PreviousFailure)
    ));
    assert_eq!(sink.calls, 1);
    assert_eq!(summary.finish(metadata()).unwrap().total_errors, 0);
}

/// Poison counters through their public checked API to inject an actual
/// technical dependency failure. No valid summary may be produced afterward.
#[test]
fn summary_failure_is_preserved_and_pipeline_cannot_resume() {
    let schema = schema("required", "numeric", json!([]));
    let field = &schema.traces()[0].fields()[0];
    let mut sink = VecSink::default();
    let mut summary = builder(&schema);
    assert!(
        summary
            .error_emitted(&ValidationCode::RegulatoryRule("bad".into()))
            .is_err()
    );
    let mut pipeline = FieldPipeline::new(factory(&schema), &mut sink, &mut summary);
    assert!(matches!(
        pipeline.run(
            field,
            FieldSlice { raw: b"    " },
            1,
            &RecordIdentity::default()
        ),
        Err(FieldPipelineError::Summary(_))
    ));
    assert!(matches!(
        pipeline.run(
            field,
            FieldSlice { raw: b"0012" },
            2,
            &RecordIdentity::default()
        ),
        Err(FieldPipelineError::PreviousFailure)
    ));
    assert_eq!(sink.errors().len(), 1);
    assert!(summary.finish(metadata()).is_err());
}
