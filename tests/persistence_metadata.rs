//! Synthetic metadata only. Consent is recorded evidence, never automatic approval.
use chrono::{TimeZone, Utc};
use lintra::*;
use serde_json::json;
fn parts() -> SourceSubmissionParts {
    serde_json::from_slice(include_bytes!(
        "fixtures/persistence/source-submission.json"
    ))
    .unwrap()
}
fn time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, 2, 4, 0, 0).unwrap()
}
#[test]
fn legacy_approved_metadata_does_not_infer_consent_and_retains_exact_bytes() {
    let raw = include_str!("fixtures/persistence/source-submission.json").trim();
    let source: SourceSubmission = serde_json::from_str(raw).unwrap();
    assert_eq!(source.state(), SessionState::ApprovedImmutable);
    assert!(source.missing_trace_consent().is_none());
    assert_eq!(serde_json::to_string(&source).unwrap(), raw);
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/persistence/legacy-storage-v1.json")).unwrap();
    let raw = fixture["source_utf8"].as_str().unwrap();
    let source: SourceSubmission = serde_json::from_str(raw).unwrap();
    assert!(source.missing_trace_consent().is_none());
    assert_eq!(serde_json::to_string(&source).unwrap(), raw);
}
#[test]
fn proceed_has_explicit_scope_and_utc_but_does_not_change_declared_session_state() {
    let mut p = parts();
    p.state = SessionState::Validated;
    let receipt = MissingTraceProceedConsent::new(vec![TraceId("T2".into())], time()).unwrap();
    let source = SourceSubmission::new_with_missing_trace_consent(p, receipt.clone()).unwrap();
    assert_eq!(source.state(), SessionState::Validated);
    assert_eq!(source.missing_trace_consent(), Some(&receipt));
    assert_eq!(receipt.recorded_at(), time());
    assert_eq!(receipt.traces(), &[TraceId("T2".into())]);
    let encoded = serde_json::to_value(&source).unwrap();
    assert_eq!(encoded["missing_trace_consent"]["decision"], "proceed");
    let roundtrip: SourceSubmission = serde_json::from_value(encoded).unwrap();
    assert_eq!(roundtrip, source);
}
#[test]
fn present_empty_and_unknown_traces_cannot_be_acknowledged_as_absent() {
    for trace in ["T1", "UNKNOWN"] {
        let receipt = MissingTraceProceedConsent::new(vec![TraceId(trace.into())], time()).unwrap();
        assert_eq!(
            SourceSubmission::new_with_missing_trace_consent(parts(), receipt).unwrap_err(),
            PersistenceTypeError::InvalidOmissionConsent
        );
    }
    assert!(MissingTraceProceedConsent::new(vec![], time()).is_err());
    assert!(MissingTraceProceedConsent::new(vec![TraceId("T2".into()); 2], time()).is_err());
}
#[test]
fn abort_missing_timestamp_unknown_fields_and_invalid_time_are_not_durable_proceed() {
    for value in [
        json!({"decision":"abort","traces":["T2"],"recorded_at":"2026-01-02T04:00:00Z"}),
        json!({"decision":"proceed","traces":["T2"]}),
        json!({"decision":"proceed","traces":["T2"],"recorded_at":"invalid-timestamp"}),
        json!({"decision":"proceed","traces":["T2"],"recorded_at":"2026-01-02T04:00:00Z","automatic":true}),
    ] {
        assert!(serde_json::from_value::<MissingTraceProceedConsent>(value).is_err());
    }
}

#[test]
fn explicit_offset_is_normalized_to_utc_using_the_existing_timestamp_contract() {
    let receipt: MissingTraceProceedConsent = serde_json::from_value(
        json!({"decision":"proceed","traces":["T2"],"recorded_at":"2026-01-02T05:00:00+01:00"}),
    )
    .unwrap();
    assert_eq!(receipt.recorded_at(), time());
    assert_eq!(
        serde_json::to_value(receipt).unwrap()["recorded_at"],
        "2026-01-02T04:00:00.000000000Z"
    );
}
