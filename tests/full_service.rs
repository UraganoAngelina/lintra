use lintra::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs};
#[path = "support/full_fixture.rs"]
mod full_fixture;
#[path = "support/full_schema.rs"]
mod full_schema;
use full_fixture::Fixture;
fn validate(fixture: &Fixture) -> (ValidationRunOutcome, VecSink) {
    let mut sink = VecSink::default();
    let out = fixture
        .service()
        .validate(
            fixture.request(),
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    (out, sink)
}

#[test]
fn full_service_reconciles_real_cross_errors_and_persists_counts() {
    let mut raw = full_schema::raw();
    let mut second = raw["traces"][0]["rules"][0].clone();
    second["id"] = json!("SECOND");
    raw["traces"][0]["rules"]
        .as_array_mut()
        .unwrap()
        .push(second);
    let fixture = Fixture::new(raw, b"01AANO\n02BBOK", b"11AAOK\n12BBOK");
    let (out, sink) = validate(&fixture);
    assert_eq!(out.status, RunStatus::CompletedInvalid);
    assert_eq!(
        (
            out.summary.records_processed,
            out.summary.valid_records,
            out.summary.invalid_records,
            out.summary.total_errors
        ),
        (4, 3, 1, 2)
    );
    assert_eq!(out.summary.skipped_rules, 0);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Complete);
    assert_eq!(sink.summaries().len(), 1);
    let bytes = write_run_summary(Vec::new(), &out.summary).unwrap();
    let document: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(document["validation_mode"], "full");
    assert_eq!(document["completeness"], "complete");
    assert_eq!(document["scope"]["trace_ids"], json!(["S", "T"]));
    assert_eq!(document["full_validation"]["controls"][0]["violations"], 1);
    assert_eq!(document["full_validation"]["controls"][1]["violations"], 1);
    // Mutating any part of a public summary must be rejected before serialization.
    for mutation in 0..5 {
        let mut bad = out.summary.clone();
        match mutation {
            0 => bad.full_validation.as_mut().unwrap().controls[0].satisfied += 1,
            1 => bad.full_validation.as_mut().unwrap().controls[0].skipped = 1,
            2 => bad.full_validation.as_mut().unwrap().traces[0].records_processed += 1,
            3 => bad.full_validation.as_mut().unwrap().controls.clear(),
            _ => bad.full_validation = None,
        };
        assert!(matches!(
            write_run_summary(Vec::new(), &bad),
            Err(ReportError::InvalidSummary(_))
        ));
    }
}
#[test]
fn incomplete_context_is_explicit_in_summary_audit_and_human_report() {
    let fixture = Fixture::new(full_schema::raw(), b"01AAOK", b"bad\n11BBOK");
    let (out, sink) = validate(&fixture);
    assert_eq!(out.summary.total_errors, 1);
    assert_eq!(sink.errors()[0].code, ValidationCode::RecordLength);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
    let audit = out.summary.full_validation.as_ref().unwrap();
    assert_eq!(audit.controls[0].unknown, 1);
    assert_eq!(audit.controls[0].causes[0].join_id.as_deref(), Some("S_T"));
    assert_eq!(
        audit.controls[0].causes[0].result,
        CrossCauseKind::Unknown(CrossUncertaintyReason::IncompleteRelatedContext)
    );
    assert_eq!(audit.traces[1].coverage, ContextCoverage::Partial);
    assert!(
        out.summary
            .inputs
            .iter()
            .all(|a| matches!(a.completion, InputAuditCompletion::Complete { .. }))
    );
    let mut human = HumanReadableSink::new(Vec::new());
    human.on_summary(&out.summary).unwrap();
    let text = String::from_utf8(human.finish().unwrap()).unwrap();
    assert!(text.contains("Completeness: Partial"));
    assert!(text.contains("inconclusive 1"));
    let mut bad = out.summary.clone();
    bad.full_validation.as_mut().unwrap().completeness = ValidationCompleteness::Complete;
    assert!(bad.validate().is_err());
}
#[test]
fn not_applicable_is_neither_unknown_nor_a_normative_violation() {
    let mut raw = full_schema::raw();
    raw["traces"][0]["joins"][0]["on_missing_record"] = json!({"policy":"not_applicable","diagnostic":{"code":"OUTSIDE_SCOPE","message_template":"Not applicable for {rule_id}"}});
    let fixture = Fixture::new(raw, b"01AAOK", b"");
    let (out, sink) = validate(&fixture);
    assert_eq!(out.status, RunStatus::CompletedValid);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Complete);
    assert!(sink.errors().is_empty());
    let control = &out.summary.full_validation.as_ref().unwrap().controls[0];
    assert_eq!(
        (
            control.satisfied,
            control.not_applicable,
            control.unknown,
            control.violations
        ),
        (0, 1, 0, 0)
    );
    assert_eq!(control.causes[0].result, CrossCauseKind::NotApplicable);
    assert_eq!(
        control.causes[0].diagnostic_code.as_deref(),
        Some("OUTSIDE_SCOPE")
    );
}
#[test]
fn runtime_parameters_follow_declared_types_and_are_audited_without_defaults() {
    let mut raw = full_schema::raw();
    raw["parameters"] = json!([{"id":"org","type":"string","allowed_values":["001"]},{"id":"year","type":"integer","min":2020,"max":2030}]);
    raw["traces"][0]["joins"][0]["namespace_parameters"] = json!(["org", "year"]);
    let fixture = Fixture::new(raw, b"01AAOK", b"11AAOK");
    let mut sink = VecSink::default();
    let error = fixture
        .service()
        .validate(
            fixture.request(),
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap_err();
    assert!(matches!(
        error,
        ValidationRunError::Flow(FlowValidationError::Inputs(
            RelatedContextError::MissingNamespaceParameter
        ))
    ));
    assert!(sink.summaries().is_empty());
    let out = fixture
        .service()
        .validate_with_parameters(
            fixture.request(),
            BTreeMap::from([
                ("org".into(), "001".into()),
                ("year".into(), "02025".into()),
            ]),
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    assert_eq!(out.status, RunStatus::CompletedValid);
    let parameters = &out.summary.full_validation.as_ref().unwrap().parameters;
    assert_eq!(parameters["org"], RunParameterAudit::String("001".into()));
    assert_eq!(
        parameters["year"],
        RunParameterAudit::Integer("2025".into())
    );
    for (key, value) in [("org", "1"), ("year", "2031"), ("other", "001")] {
        let mut parameters =
            BTreeMap::from([("org".into(), "001".into()), ("year".into(), "2025".into())]);
        parameters.insert(key.into(), value.into());
        assert!(matches!(
            fixture.service().validate_with_parameters(
                fixture.request(),
                parameters,
                &mut VecSink::default(),
                &mut |_| {},
                &CancellationToken::default()
            ),
            Err(ValidationRunError::Flow(FlowValidationError::Inputs(
                RelatedContextError::InvalidParameter
            )))
        ));
    }
}
#[test]
fn cancellation_in_full_returns_partial_outcome_without_inventing_evaluations() {
    for checkpoint in [0, 1, 2] {
        let fixture = Fixture::new(full_schema::raw(), b"01AAOK\n02AAOK", b"11AAOK");
        let token = CancellationToken::default();
        if checkpoint == 0 {
            token.cancel();
        }
        let mut events = vec![];
        let out = fixture
            .service()
            .validate(
                fixture.request(),
                &mut VecSink::default(),
                &mut |event| {
                    if matches!(
                        (&event, checkpoint),
                        (
                            ProgressEvent::PhaseStarted(ValidationPhase::CrossRecordValidation),
                            1
                        ) | (ProgressEvent::CrossRecordsProcessed { count: 1 }, 2)
                    ) {
                        token.cancel();
                    }
                    events.push(event);
                },
                &token,
            )
            .unwrap();
        assert_eq!(out.status, RunStatus::Cancelled);
        assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
        let audit = out.summary.full_validation.as_ref().unwrap();
        assert_eq!(audit.controls[0].evaluations, u64::from(checkpoint == 2));
        assert_eq!(
            out.summary.skipped_rules,
            if checkpoint == 0 {
                0
            } else if checkpoint == 1 {
                2
            } else {
                1
            }
        );
        assert!(matches!(events.last(), Some(ProgressEvent::RunCompleted)));
        out.summary.validate().unwrap();
    }
}
#[test]
fn full_reference_audit_merges_sources_once_and_keeps_each_trace_control() {
    use sha2::{Digest, Sha256};
    let data=b"{\"dataset_format_version\":1,\"dataset_id\":\"synthetic\",\"version\":\"1\",\"keys_hex\":[\"4f4b\"]}";
    for state in 0..3 {
        let mut raw = full_schema::raw();
        let digest = if state == 1 {
            "0".repeat(64)
        } else {
            Sha256::digest(data)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        };
        raw["reference_data"] = json!([{"id":"CODES","dataset_id":"synthetic","version":"1","path":"codes.json","sha256":digest,"metadata_attributes":[]}]);
        for trace in raw["traces"].as_array_mut().unwrap() {
            trace["fields"][2]["constraints"] = json!([{"kind":"reference_data","datasets":["CODES"],"fallback_values":[],"metadata_bindings":[]}]);
        }
        let fixture = Fixture::new(raw, b"01AAOK", b"11AAOK");
        if state != 2 {
            fs::write(fixture.reference_root.join("codes.json"), data).unwrap();
        }
        let (out, sink) = validate(&fixture);
        assert_eq!(out.status, RunStatus::CompletedValid);
        assert!(sink.errors().is_empty());
        assert_eq!(out.summary.total_errors, 0);
        let audit = out.summary.reference_validation.as_ref().unwrap();
        assert_eq!(audit.datasets.len(), 1);
        assert_eq!(audit.controls.len(), 2);
        assert_eq!(
            audit
                .controls
                .iter()
                .map(|c| c.trace_id.0.as_str())
                .collect::<Vec<_>>(),
            ["S", "T"]
        );
        assert_eq!(
            out.summary.completeness(),
            if state == 0 {
                ValidationCompleteness::Complete
            } else {
                ValidationCompleteness::Partial
            }
        );
        assert_eq!(
            out.summary.full_validation.as_ref().unwrap().controls[0].satisfied,
            1
        );
        let document: Value =
            serde_json::from_slice(&write_run_summary(Vec::new(), &out.summary).unwrap()).unwrap();
        assert_eq!(
            document["completeness"],
            if state == 0 { "complete" } else { "partial" }
        );
    }
}

#[test]
fn full_input_and_sink_failures_remain_technical_and_never_emit_completed_outcomes() {
    for failure in 0..2 {
        let fixture = Fixture::new(
            full_schema::raw(),
            if failure == 0 {
                b"\xef\xbb\xbf01AAOK"
            } else {
                b"01AAOK"
            },
            b"11AAOK",
        );
        if failure == 1 {
            fs::remove_file(&fixture.target).unwrap();
        }
        let mut sink = VecSink::default();
        let mut events = vec![];
        let error = fixture
            .service()
            .validate(
                fixture.request(),
                &mut sink,
                &mut |event| events.push(event),
                &CancellationToken::default(),
            )
            .unwrap_err();
        assert!(matches!(error, ValidationRunError::Input(_)));
        assert!(sink.summaries().is_empty());
        assert!(!events.contains(&ProgressEvent::RunCompleted));
    }
    struct Failing {
        on_error: bool,
    }
    impl ValidationSink for Failing {
        fn on_error(&mut self, _: &ValidationError) -> Result<(), ReportError> {
            if self.on_error {
                Err(ReportError::Write(std::io::Error::other(
                    "synthetic Full write",
                )))
            } else {
                Ok(())
            }
        }
        fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
            Err(ReportError::Write(std::io::Error::other(
                "synthetic Full summary",
            )))
        }
    }
    let fixture = Fixture::new(full_schema::raw(), b"01AANO", b"11AAOK");
    for on_error in [true, false] {
        let mut events = vec![];
        let error = fixture
            .service()
            .validate(
                fixture.request(),
                &mut Failing { on_error },
                &mut |event| events.push(event),
                &CancellationToken::default(),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            ValidationRunError::Report(ReportError::Write(_))
        ));
        assert!(!events.contains(&ProgressEvent::RunCompleted));
    }
}
