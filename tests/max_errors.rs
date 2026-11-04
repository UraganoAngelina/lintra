//! DEC-16 A: actual emitted prefixes, controlled stops and complete/unlimited
//! counterexamples. No CLI flag, normative schema or persistent format change.
use lintra::*;
use serde_json::{Value, json};
use std::{fs, io};
#[path = "support/full_fixture.rs"]
mod fixture;
#[path = "support/full_schema.rs"]
mod full_schema;
use fixture::Fixture;

fn local_model() -> Value {
    let mut raw: Value =
        serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
    let field = raw["traces"][0]["fields"][0].clone();
    raw["traces"][0]["record_length"] = json!(2);
    raw["traces"][0]["fields"] = json!(
        ["A", "B"]
            .into_iter()
            .enumerate()
            .map(|(i, id)| {
                let mut f = field.clone();
                f["id"] = json!(id);
                f["physical"]["start"] = json!(i + 1);
                f["physical"]["length"] = json!(1);
                f["constraints"] = json!([{"kind":"allowed_values","target":"raw","values":["A"]}]);
                f
            })
            .collect::<Vec<_>>()
    );
    raw["traces"][0]["record_key"]["fields"] = json!(["A"]);
    raw
}
fn local_request(f: &Fixture, maximum: Option<u64>) -> ValidationRequest {
    let mut request = f.request();
    request.mode = ValidationMode::LocalOnly;
    request.scope = ValidationScope::Trace(TraceValidationInput {
        trace_id: TraceId("T1".into()),
        source: InputSource::Path(f.source.clone()),
    });
    request.options.max_errors = maximum;
    request
}
fn execute(f: &Fixture, request: ValidationRequest) -> (ValidationRunOutcome, VecSink) {
    let mut sink = VecSink::default();
    let outcome = f
        .service()
        .validate(
            request,
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    assert_eq!(sink.summaries(), std::slice::from_ref(&outcome.summary));
    (outcome, sink)
}
fn assert_limited(out: &ValidationRunOutcome, errors: u64) {
    assert_eq!(out.status, RunStatus::CompletedInvalid);
    assert!(out.summary.truncated);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
    assert_eq!(out.summary.total_errors, errors);
    let json: Value =
        serde_json::from_slice(&write_run_summary(Vec::new(), &out.summary).unwrap()).unwrap();
    assert_eq!(json["result"], "INVALID");
    assert_eq!(json["truncated"], true);
    if out.summary.full_validation.is_some() || out.summary.reference_validation.is_some() {
        assert_eq!(json["completeness"], "partial");
    } else {
        // Preserve the legacy LocalOnly document shape: its existing truncated
        // flag conveys incompleteness; no new JSON field is silently added.
        assert!(json.get("completeness").is_none());
    }
    assert_eq!(
        serde_json::to_value(out.status).unwrap(),
        "completed_invalid"
    );
}

#[test]
fn many_local_errors_stop_at_the_exact_emitted_prefix_and_none_remains_unlimited() {
    let f = Fixture::new(local_model(), &b"xy\n".repeat(1500), b"");
    for limit in [Some(1), Some(3), Some(4), Some(3000), Some(5000), None] {
        let (out, sink) = execute(&f, local_request(&f, limit));
        let expected = limit.unwrap_or(3000).min(3000);
        assert_eq!(sink.errors().len() as u64, expected);
        for (i, error) in sink.errors().iter().enumerate() {
            assert_eq!(
                error.field.as_ref().unwrap().id.0,
                if i % 2 == 0 { "A" } else { "B" }
            );
            assert_eq!(error.line_number, (i / 2 + 1) as u64);
        }
        assert_eq!(out.summary.records_processed, expected.div_ceil(2));
        assert_eq!(out.summary.invalid_records, out.summary.records_processed);
        assert_eq!(out.summary.valid_records, 0);
        if limit.is_some_and(|n| n <= 3000) {
            assert_limited(&out, expected);
        } else {
            assert!(!out.summary.truncated);
            assert_eq!(out.summary.completeness(), ValidationCompleteness::Complete);
            assert_eq!(out.summary.total_errors, 3000);
        }
    }
}

#[test]
fn valid_inputs_and_ending_before_the_budget_remain_complete() {
    let f = Fixture::new(local_model(), b"AA\nAA\n", b"");
    let (out, sink) = execute(&f, local_request(&f, Some(1)));
    assert_eq!(out.status, RunStatus::CompletedValid);
    assert!(!out.summary.truncated);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Complete);
    assert!(sink.errors().is_empty());
    let f = Fixture::new(local_model(), b"xy", b""); // EOF observed while framing.
    let (out, _) = execute(&f, local_request(&f, Some(3)));
    assert!(!out.summary.truncated);
    assert_eq!(out.summary.total_errors, 2);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Complete);
    let (out, _) = execute(&f, local_request(&f, Some(2)));
    assert_limited(&out, 2); // Exact EOF is still a conservative operational stop.
}

#[test]
fn local_rule_errors_share_the_same_budget_without_a_second_rule_leaking() {
    let mut raw = local_model();
    raw["required_capabilities"] = json!(["local.v1", "expressions.v1"]);
    raw["traces"][0]["rules"]=json!(["LOCAL-1","LOCAL-2"].map(|id|json!({
        "id":id,"when":{"const":true},"assertions":[{"kind":"field_compare",
        "left":{"ref":{"field":"A"}},"operator":"!=","right":{"ref":{"field":"B"}},"value_type":"bytes"}],
        "error_code":"E_CROSS_FIELD","message_template":"Synthetic {rule_id}: {condition}"
    })));
    let f = Fixture::new(raw, b"AA\nAA\n", b"");
    let (out, sink) = execute(&f, local_request(&f, Some(1)));
    assert_limited(&out, 1);
    assert_eq!(out.summary.records_processed, 1);
    assert_eq!(sink.errors().len(), 1);
    assert!(sink.errors()[0].message.contains("LOCAL-1"));
}

fn two_cross_rules() -> Value {
    let mut raw = full_schema::raw();
    let mut second = raw["traces"][0]["rules"][0].clone();
    second["id"] = json!("COMPARE-2");
    raw["traces"][0]["rules"]
        .as_array_mut()
        .unwrap()
        .push(second);
    raw
}
fn full_request(f: &Fixture, limit: u64) -> ValidationRequest {
    let mut r = f.request();
    r.options.max_errors = Some(limit);
    r
}

#[test]
fn cross_pass_stops_between_rules_and_keeps_complete_input_hashes_separate() {
    let f = Fixture::new(two_cross_rules(), b"01aaNO\n02bbNO\n", b"11aaOK\n12bbOK\n");
    for maximum in 1..=5 {
        let (out, sink) = execute(&f, full_request(&f, maximum));
        assert_eq!(out.summary.records_processed, 4);
        assert_eq!(sink.errors().len() as u64, maximum.min(4));
        assert!(
            out.summary
                .inputs
                .iter()
                .all(|i| matches!(i.completion, InputAuditCompletion::Complete { .. }))
        );
        let full = out.summary.full_validation.as_ref().unwrap();
        assert_eq!(
            full.controls.iter().map(|c| c.evaluations).sum::<u64>(),
            maximum.min(4)
        );
        assert_eq!(out.summary.invalid_records, maximum.min(4).div_ceil(2));
        if maximum <= 4 {
            assert_limited(&out, maximum);
        } else {
            assert!(!out.summary.truncated);
            assert_eq!(out.summary.completeness(), ValidationCompleteness::Complete);
        }
    }
}

#[test]
fn budget_is_global_across_traces_and_across_both_passes() {
    let mut raw = two_cross_rules();
    raw["traces"][0]["fields"][0]["constraints"] =
        json!([{"kind":"allowed_values","target":"raw","values":["01"]}]);
    let f = Fixture::new(raw, b"01aaNO\n02bbNO\n", b"11aaOK\n12bbOK\n");
    let (out, sink) = execute(&f, full_request(&f, 2));
    assert_limited(&out, 2);
    assert_eq!(sink.errors()[0].field.as_ref().unwrap().id.0, "PK");
    assert_eq!(sink.errors()[0].line_number, 2);
    assert_eq!(
        out.summary
            .full_validation
            .as_ref()
            .unwrap()
            .controls
            .iter()
            .map(|c| c.evaluations)
            .sum::<u64>(),
        1
    );

    let mut raw = two_cross_rules();
    for i in 0..2 {
        raw["traces"][i]["fields"][2]["constraints"] =
            json!([{"kind":"allowed_values","target":"raw","values":["OK"]}]);
    }
    let f = Fixture::new(raw, b"01aaNO\n02bbNO\n", b"11aaNO\n12bbNO\n");
    let (out, _) = execute(&f, full_request(&f, 3));
    assert_limited(&out, 3);
    assert_eq!(out.summary.records_processed, 3);
    let full = out.summary.full_validation.as_ref().unwrap();
    assert_eq!(full.traces[0].coverage, ContextCoverage::Complete);
    assert_eq!(full.traces[1].coverage, ContextCoverage::Partial);
    assert_eq!(full.traces[1].unprojectable_records, 0);
    assert_eq!(full.traces[1].inconclusive_record_keys, 1);
    assert!(full.controls.iter().all(|c| c.evaluations == 0));
}

#[test]
fn length_and_duplicate_limits_do_not_open_the_next_trace_or_evaluate_partial_context() {
    for source in [b"x\n".as_slice(), b"01aaOK\n01bbOK\n".as_slice()] {
        let f = Fixture::new(two_cross_rules(), source, b"11aaOK\n12bbOK\n");
        fs::remove_file(&f.target).unwrap();
        let (out, sink) = execute(&f, full_request(&f, 1));
        assert_limited(&out, 1);
        assert_eq!(sink.errors().len(), 1);
        assert_eq!(out.summary.inputs.len(), 1);
        assert!(
            out.summary
                .full_validation
                .as_ref()
                .unwrap()
                .controls
                .iter()
                .all(|c| c.evaluations == 0)
        );
        // A larger budget must still expose the real unread target I/O error.
        let mut sink = VecSink::default();
        assert!(matches!(
            f.service().validate(
                full_request(&f, 5),
                &mut sink,
                &mut |_| {},
                &CancellationToken::default()
            ),
            Err(ValidationRunError::Input(_))
        ));
    }
}

#[test]
fn unknown_evidence_does_not_consume_a_normative_error_budget() {
    let mut raw = two_cross_rules();
    raw["traces"][0]["joins"][0]["on_missing_record"] = json!({"policy":"unknown","diagnostic":{"code":"SYNTHETIC_UNKNOWN","message_template":"Synthetic missing context"}});
    let f = Fixture::new(raw, b"01aaNO\n", b"11bbOK\n");
    let (out, sink) = execute(&f, full_request(&f, 1));
    assert_eq!(out.status, RunStatus::CompletedValid);
    assert!(!out.summary.truncated);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
    assert!(sink.errors().is_empty());
    assert_eq!(
        out.summary
            .full_validation
            .as_ref()
            .unwrap()
            .controls
            .iter()
            .map(|c| c.unknown)
            .sum::<u64>(),
        2
    );
}

#[test]
fn zero_is_rejected_before_missing_inputs_in_both_modes() {
    for full in [false, true] {
        let f = Fixture::new(
            if full {
                two_cross_rules()
            } else {
                local_model()
            },
            b"x\n",
            b"",
        );
        fs::remove_file(&f.source).unwrap();
        let request = if full {
            full_request(&f, 0)
        } else {
            local_request(&f, Some(0))
        };
        let mut sink = VecSink::default();
        assert!(matches!(
            f.service().validate(
                request,
                &mut sink,
                &mut |_| {},
                &CancellationToken::default()
            ),
            Err(ValidationRunError::InvalidRequest(
                InvalidRequestError::UnsupportedMaxErrors { limit: 0 }
            ))
        ));
        assert!(sink.errors().is_empty() && sink.summaries().is_empty());
    }
}

#[test]
fn a_limit_preserves_prefix_audit_and_never_announces_trace_completion() {
    for full in [false, true] {
        let bytes = if full {
            b"x\n".repeat(50_000)
        } else {
            b"xy\n".repeat(50_000)
        };
        let f = Fixture::new(
            if full {
                two_cross_rules()
            } else {
                local_model()
            },
            &bytes,
            b"11aaOK\n",
        );
        let request = if full {
            full_request(&f, 1)
        } else {
            local_request(&f, Some(1))
        };
        let mut sink = VecSink::default();
        let mut events = Vec::new();
        let out = f
            .service()
            .validate(
                request,
                &mut sink,
                &mut |event| events.push(event),
                &CancellationToken::default(),
            )
            .unwrap();
        assert_limited(&out, 1);
        assert_eq!(out.summary.records_processed, 1);
        assert_eq!(out.summary.inputs.len(), 1);
        assert!(matches!(
            out.summary.inputs[0].completion,
            InputAuditCompletion::Incomplete
        ));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, ProgressEvent::TraceCompleted { .. }))
        );
        assert!(!events.iter().any(|event| matches!(
            event,
            ProgressEvent::PhaseStarted(ValidationPhase::CrossRecordValidation)
        )));
        assert_eq!(events.last(), Some(&ProgressEvent::RunCompleted));
    }
}

#[test]
fn cancellation_before_the_limit_and_after_its_checkpoint_keep_distinct_causes() {
    let f = Fixture::new(local_model(), b"xy\nxy\n", b"");
    for before in [true, false] {
        let cancel = CancellationToken::default();
        let mut sink = VecSink::default();
        let mut progress = |event| {
            if (before && matches!(event, ProgressEvent::RunStarted))
                || (!before && matches!(event, ProgressEvent::RecordsProcessed { count: 1, .. }))
            {
                cancel.cancel();
            }
        };
        let out = f
            .service()
            .validate(
                local_request(&f, Some(1)),
                &mut sink,
                &mut progress,
                &cancel,
            )
            .unwrap();
        if before {
            assert_eq!(out.status, RunStatus::Cancelled);
            assert!(!out.summary.truncated);
            assert!(sink.errors().is_empty());
        } else {
            assert_limited(&out, 1);
        }
    }
}

#[test]
fn an_input_size_change_observed_before_the_limit_remains_a_technical_error() {
    let f = Fixture::new(local_model(), b"xy\nxy\n", b"");
    let mut sink = VecSink::default();
    let result = f.service().validate(
        local_request(&f, Some(1)),
        &mut sink,
        &mut |event| {
            if matches!(event, ProgressEvent::TraceStarted { .. }) {
                fs::write(&f.source, b"").unwrap();
            }
        },
        &CancellationToken::default(),
    );
    assert!(matches!(result, Err(ValidationRunError::Input(_))));
    assert!(sink.errors().is_empty());
    assert!(sink.summaries().is_empty());
}

struct FailingSink {
    calls: u64,
    accepted: u64,
    fail_error_at: u64,
    fail_summary: bool,
    summaries: u64,
}
impl ValidationSink for FailingSink {
    fn on_error(&mut self, _: &ValidationError) -> Result<(), ReportError> {
        self.calls += 1;
        if self.calls == self.fail_error_at {
            return Err(ReportError::Write(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "synthetic error sink failure",
            )));
        }
        self.accepted += 1;
        Ok(())
    }
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        self.summaries += 1;
        if self.fail_summary {
            Err(ReportError::Write(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "synthetic summary sink failure",
            )))
        } else {
            Ok(())
        }
    }
}
#[test]
fn nth_error_and_final_summary_failures_are_technical_not_controlled_limits() {
    for full in [false, true] {
        let f = if full {
            Fixture::new(two_cross_rules(), b"01aaNO\n", b"11aaOK\n")
        } else {
            Fixture::new(local_model(), b"xy\n", b"")
        };
        for final_summary in [false, true] {
            let mut sink = FailingSink {
                calls: 0,
                accepted: 0,
                fail_error_at: if final_summary { u64::MAX } else { 2 },
                fail_summary: final_summary,
                summaries: 0,
            };
            let maximum = if final_summary { 1 } else { 2 };
            let request = if full {
                full_request(&f, maximum)
            } else {
                local_request(&f, Some(maximum))
            };
            let result = f.service().validate(
                request,
                &mut sink,
                &mut |_| {},
                &CancellationToken::default(),
            );
            assert!(
                matches!(result,Err(ValidationRunError::Report(ReportError::Write(ref e))) if e.kind()==io::ErrorKind::PermissionDenied)
            );
            assert_eq!(sink.accepted, 1);
            assert_eq!(sink.summaries, u64::from(final_summary));
        }
    }
}
