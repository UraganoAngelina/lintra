//! CNF-DIAGNOSTIC-ORDER-01: assert the existing phase-major stream, never sort it.
use lintra::*;
use serde_json::{Value, json};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
#[path = "support/full_fixture.rs"]
mod full_fixture;
#[path = "support/full_schema.rs"]
mod full_schema;
use full_fixture::Fixture;

struct FixedClock;
impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH
    }
}
fn model() -> Value {
    let mut raw = full_schema::raw();
    raw["traces"][0]["id"] = json!("S0");
    raw["traces"][1]["id"] = json!("T1");
    raw["traces"][0]["joins"][0]["target_trace"] = json!("T1");
    raw["traces"][0]["rules"][0]["assertions"][0]["right"]["ref"]["trace"] = json!("T1");
    let first = raw["traces"][0]["rules"][0].clone();
    let mut earlier_field = first;
    earlier_field["id"] = json!("COMPARE_PK_AFTER_DATA");
    earlier_field["assertions"][0]["left"]["ref"]["field"] = json!("PK");
    earlier_field["assertions"][0]["right"]["ref"]["field"] = json!("PK");
    let mut missing_join = raw["traces"][0]["joins"][0].clone();
    missing_join["id"] = json!("EVENT_MISSING");
    missing_join["source_fields"] = json!(["DATA"]);
    raw["traces"][0]["joins"]
        .as_array_mut()
        .unwrap()
        .push(missing_join);
    raw["traces"][0]["rules"].as_array_mut().unwrap().extend([
        earlier_field,
        json!({"id":"EVENT_WITHOUT_FIELD", "when":{"const":true},
          "assertions":[{"kind":"record_exists","join":"EVENT_MISSING"}],
          "error_code":"E_RELATED_RECORD_MISSING","message_template":"Synthetic missing event"}),
    ]);
    raw["traces"][1]["fields"][2]["constraints"] = json!([
        {"kind":"allowed_values","values":["OK"],"target":"raw"},
        {"kind":"regex","pattern":"\\AOK\\z","match":"required","target":"raw"}
    ]);
    raw["traces"][1]["rules"] = json!([{
        "id":"LOCAL_PK_AFTER_FIELDS", "when":{"const":true},
        "assertions":[{"kind":"constraint","field":{"field":"PK"},
            "constraint":{"kind":"allowed_values","values":["ZZ"],"target":"raw"}}],
        "error_code":"E_CROSS_FIELD","message_template":"Synthetic local PK assertion"
    }]);
    raw
}
fn request(fixture: &Fixture, budget: Option<u64>) -> ValidationRequest {
    let mut request = fixture.request();
    let ValidationScope::Flow(ref mut flow) = request.scope else {
        panic!()
    };
    // Intentionally reverse schema order: request enumeration is not the order.
    flow.traces[0].trace_id = TraceId("T1".into());
    flow.traces[1].trace_id = TraceId("S0".into());
    request.options.max_errors = budget;
    request
}
fn service(fixture: &Fixture) -> ValidationService {
    let repository = FileSystemSchemaRepository::new(fixture.schema_root.clone()).unwrap();
    let registry = SchemaRegistry::new(
        Arc::new(repository),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    );
    ValidationService::with_clock(Arc::new(registry), Arc::new(FixedClock)).with_reference_data(
        Arc::new(FileSystemReferenceDataPreparer::new(fixture.reference_root.clone()).unwrap()),
        DatasetPreparationPolicy::ContinueWithAvailableEvidence,
    )
}
fn fixture() -> Fixture {
    Fixture::new(model(), b"01AAOK\n02AAOK", b"11AANO\n11BBNO")
}
fn run(fixture: &Fixture, budget: Option<u64>) -> (ValidationRunOutcome, VecSink) {
    let mut sink = VecSink::default();
    let outcome = service(fixture)
        .validate(
            request(fixture, budget),
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    outcome.summary.validate().unwrap();
    (outcome, sink)
}
fn signature(errors: &[ValidationError]) -> Vec<(String, u64, Option<String>, String)> {
    errors
        .iter()
        .map(|e| {
            (
                e.trace_id.0.clone(),
                e.line_number,
                e.field.as_ref().map(|f| f.id.0.clone()),
                e.code.as_symbol().unwrap().to_string(),
            )
        })
        .collect()
}
fn jsonl(errors: &[ValidationError]) -> Vec<u8> {
    let mut sink = JsonlSink::new(Vec::new());
    for error in errors {
        sink.on_error(error).unwrap();
    }
    sink.finish().unwrap()
}
#[test]
fn phase_major_field_pipeline_local_rules_pk_and_cross_declaration_order() {
    let fixture = fixture();
    let (out, sink) = run(&fixture, None);
    let mut expected = Vec::new();
    for line in [1, 2] {
        expected.push((
            "T1".into(),
            line,
            Some("DATA".into()),
            "E_INVALID_DOMAIN".into(),
        ));
        expected.push(("T1".into(), line, Some("PK".into()), "E_CROSS_FIELD".into()));
        if line == 2 {
            expected.push(("T1".into(), line, None, "E_DUPLICATE_RECORD_KEY".into()));
        }
    }
    for line in [1, 2] {
        expected.push((
            "S0".into(),
            line,
            Some("DATA".into()),
            "E_CROSS_FIELD".into(),
        ));
        expected.push(("S0".into(), line, Some("PK".into()), "E_CROSS_FIELD".into()));
        expected.push(("S0".into(), line, None, "E_RELATED_RECORD_MISSING".into()));
    }
    // Independent expected sequence, not merely two runs of the same algorithm.
    assert_eq!(signature(sink.errors()), expected);
    assert_eq!(out.summary.total_errors, 11);
    assert_eq!(out.status, RunStatus::CompletedInvalid);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Complete);
    let (again, second) = run(&fixture, None);
    assert_eq!(jsonl(sink.errors()), jsonl(second.errors()));
    // Elapsed is real monotonic telemetry, not a deterministic timestamp.
    // Compare the actual deterministic counters/evidence without rewriting
    // either summary or normalizing the error stream.
    assert_eq!(
        (
            out.summary.records_processed,
            out.summary.valid_records,
            out.summary.invalid_records,
            out.summary.total_errors,
            out.summary.skipped_rules,
            out.summary.truncated
        ),
        (
            again.summary.records_processed,
            again.summary.valid_records,
            again.summary.invalid_records,
            again.summary.total_errors,
            again.summary.skipped_rules,
            again.summary.truncated
        )
    );
    assert_eq!(out.summary.full_validation, again.summary.full_validation);
    assert_eq!(out.summary.inputs, again.summary.inputs);
    assert_eq!(out.summary.errors_by_code, again.summary.errors_by_code);
    assert_eq!(out.summary.started_at, again.summary.started_at);
    assert_eq!(out.summary.schema_hash, again.summary.schema_hash);
}
#[test]
fn budgets_preserve_exact_prefix_in_either_pass_without_balancing_errors() {
    let fixture = fixture();
    let (_, full) = run(&fixture, None);
    for limit in [2, 6] {
        let (limited, sink) = run(&fixture, Some(limit));
        assert_eq!(
            signature(sink.errors()),
            signature(&full.errors()[..limit as usize])
        );
        assert_eq!(
            jsonl(sink.errors()),
            jsonl(&full.errors()[..limit as usize])
        );
        assert_eq!(limited.summary.total_errors, limit);
        assert!(limited.summary.truncated);
        assert_eq!(
            limited.summary.completeness(),
            ValidationCompleteness::Partial
        );
        let controls = &limited.summary.full_validation.as_ref().unwrap().controls;
        assert_eq!(
            controls.iter().map(|c| c.evaluations).sum::<u64>(),
            u64::from(limit == 6)
        );
    }
}
#[test]
fn trace_order_comes_from_schema_not_trace_name_or_request_order() {
    for reversed in [false, true] {
        let mut raw = model();
        for trace in raw["traces"].as_array_mut().unwrap() {
            trace["rules"] = json!([]);
            trace["joins"] = json!([]);
            trace["fields"][2]["constraints"] =
                json!([{"kind":"allowed_values","target":"raw","values":["OK"]}]);
        }
        if reversed {
            raw["traces"].as_array_mut().unwrap().reverse();
        }
        let fixture = Fixture::new(raw, b"01AANO", b"11AANO");
        let (_, sink) = run(&fixture, None);
        let expected = if reversed {
            vec!["T1", "S0"]
        } else {
            vec!["S0", "T1"]
        };
        assert_eq!(
            sink.errors()
                .iter()
                .map(|e| e.trace_id.0.as_str())
                .collect::<Vec<_>>(),
            expected
        );
    }
}
#[test]
fn cancellation_keeps_existing_record_checkpoints_not_budget_semantics() {
    let fixture = fixture();
    let (_, full) = run(&fixture, None);
    for before_cross in [true, false] {
        let token = CancellationToken::default();
        let mut sink = CancellingSink {
            sink: VecSink::default(),
            token: token.clone(),
            cancel_on_cross: !before_cross,
        };
        let out = service(&fixture)
            .validate(
                request(&fixture, None),
                &mut sink,
                &mut |event| {
                    if before_cross
                        && event
                            == ProgressEvent::PhaseStarted(ValidationPhase::CrossRecordValidation)
                    {
                        token.cancel();
                    }
                },
                &token,
            )
            .unwrap();
        let count = if before_cross { 5 } else { 8 };
        assert_eq!(
            signature(sink.sink.errors()),
            signature(&full.errors()[..count])
        );
        assert_eq!(out.status, RunStatus::Cancelled);
        assert!(!out.summary.truncated);
        assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
        // Cancellation inside the first cross callback completes its current
        // record's three rules; it is observed before the next source record.
        assert_eq!(
            out.summary
                .full_validation
                .as_ref()
                .unwrap()
                .controls
                .iter()
                .map(|c| c.evaluations)
                .sum::<u64>(),
            if before_cross { 0 } else { 3 }
        );
    }
}
struct CancellingSink {
    sink: VecSink,
    token: CancellationToken,
    cancel_on_cross: bool,
}
impl ValidationSink for CancellingSink {
    fn on_error(&mut self, error: &ValidationError) -> Result<(), ReportError> {
        self.sink.on_error(error)?;
        if self.cancel_on_cross && error.trace_id.0 == "S0" {
            self.token.cancel();
        }
        Ok(())
    }
    fn on_summary(&mut self, summary: &ValidationSummary) -> Result<(), ReportError> {
        self.sink.on_summary(summary)
    }
}
