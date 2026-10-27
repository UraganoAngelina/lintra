//! Synthetic metadata only. Opaque test tokens are not a crypto implementation.
use chrono::{TimeZone, Utc};
use lintra::*;
use serde_json::{Value, json};

fn namespace(unit: &str) -> PersistenceNamespace {
    PersistenceNamespace::new(vec![(
        "organization".into(),
        NamespaceValue::String(unit.into()),
    )])
    .unwrap()
}
fn token(bytes: &[u8]) -> Fingerprint {
    Fingerprint::new("synthetic-opaque".into(), "test-key".into(), bytes.to_vec()).unwrap()
}
fn identity(unit: &str, version: &str) -> PersistentRecordIdentity {
    PersistentRecordIdentity::new(
        namespace(unit),
        FlowId("SYNTHETIC".into()),
        PersistenceSchemaScope::SchemaVersion {
            version: SchemaVersion(version.into()),
        },
        TraceId("T1".into()),
        "record_key".into(),
        token(&[0xa0, 0xb1]),
    )
    .unwrap()
}
fn source_parts() -> SourceSubmissionParts {
    SourceSubmissionParts {
        namespace: namespace("SYNTHETIC-UNIT"),
        submission_id: "synthetic-submission-v1".into(),
        schema_key: SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        schema_hash: Sha256Digest([0; 32]),
        period: Some("2026-Q1".into()),
        created_at: Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap(),
        validator_version: "test".into(),
        validation_mode: ValidationMode::LocalOnly,
        validation_status: RunStatus::CompletedValid,
        reference_data_versions: vec![
            (DatasetId("Z".into()), DatasetVersion("v2".into())),
            (DatasetId("A".into()), DatasetVersion("v1".into())),
        ],
        traces: vec![
            SubmissionTrace {
                trace_id: TraceId("T2".into()),
                presence: TracePresence::Absent,
                input_sha256: None,
            },
            SubmissionTrace {
                trace_id: TraceId("T1".into()),
                presence: TracePresence::Present { record_count: 0 },
                input_sha256: Some(
                    serde_json::from_str(
                        "\"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\"",
                    )
                    .unwrap(),
                ),
            },
        ],
        fingerprint: token(&[0xa0, 0xb1]),
        state: SessionState::ApprovedImmutable,
    }
}
#[test]
fn namespace_order_is_canonical_but_values_types_padding_and_case_are_exact() {
    let entries = vec![
        ("z".into(), NamespaceValue::String(" 050 ".into())),
        ("a".into(), NamespaceValue::Integer(50)),
    ];
    let a = PersistenceNamespace::new(entries.clone()).unwrap();
    let b = PersistenceNamespace::new(entries.into_iter().rev().collect()).unwrap();
    assert_eq!(a, b);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
    assert!(serde_json::to_string(&a).unwrap().contains(" 050 "));
    assert_ne!(namespace("050"), namespace(" 050 "));
    assert_ne!(namespace("a"), namespace("A"));
    let integer =
        PersistenceNamespace::new(vec![("organization".into(), NamespaceValue::Integer(50))])
            .unwrap();
    assert_ne!(integer, namespace("50"));
    assert!(PersistenceNamespace::new(vec![]).is_err());
    for value in ["", "\n", "\0", "   "] {
        assert!(
            PersistenceNamespace::new(vec![(
                "organization".into(),
                NamespaceValue::String(value.into())
            )])
            .is_err()
        );
    }
    assert!(
        PersistenceNamespace::new(vec![
            ("x".into(), NamespaceValue::Integer(1)),
            ("x".into(), NamespaceValue::Integer(2))
        ])
        .is_err()
    );
    assert!(serde_json::from_str::<PersistenceNamespace>("[]").is_err());
    let duplicate = json!([{"parameter":"x","value":{"type":"integer","value":1}},{"parameter":"x","value":{"type":"integer","value":2}}]);
    assert!(serde_json::from_value::<PersistenceNamespace>(duplicate).is_err());
    assert!(!format!("{:?}", namespace("SYNTHETIC-UNIT")).contains("SYNTHETIC-UNIT"));
}
#[test]
fn fingerprint_roundtrips_canonical_hex_and_retains_key_context_without_logging_bytes() {
    let raw =
        json!({"protection_scheme":"synthetic-opaque","key_id":"test-key","value_hex":"A0B1"});
    let fp: Fingerprint = serde_json::from_value(raw.clone()).unwrap();
    assert_eq!(fp, token(&[0xa0, 0xb1]));
    assert_eq!(serde_json::to_value(&fp).unwrap()["value_hex"], "a0b1");
    assert!(!format!("{fp:?}").contains("a0b1"));
    assert!(!format!("{fp:?}").contains("test-key"));
    for value in ["", "0", "gg", "ÉÉ", "\u{1f980}"] {
        let mut v = raw.clone();
        v["value_hex"] = json!(value);
        assert!(serde_json::from_value::<Fingerprint>(v).is_err());
    }
    let mut v = raw;
    v["key_id"] = json!("");
    assert!(serde_json::from_value::<Fingerprint>(v).is_err());
    assert!(
        !fp.same_protection_context(
            &Fingerprint::new(
                "synthetic-opaque".into(),
                "other-key".into(),
                vec![0xa0, 0xb1]
            )
            .unwrap()
        )
    );
}
#[test]
fn identity_equality_never_crosses_namespace_flow_version_trace_key_kind_or_key_context() {
    let a = identity("UNIT-A", "v1");
    assert_eq!(
        serde_json::from_str::<PersistentRecordIdentity>(&serde_json::to_string(&a).unwrap())
            .unwrap(),
        a
    );
    assert_ne!(a, identity("UNIT-B", "v1"));
    assert_ne!(a, identity("UNIT-A", "v2"));
    let base = serde_json::to_value(&a).unwrap();
    for (field, value) in [
        ("flow_id", json!("OTHER")),
        ("trace_id", json!("T2")),
        ("key_kind", json!("event_key")),
        (
            "schema_scope",
            json!({"kind":"compatibility_group","id":"explicit-group"}),
        ),
        (
            "key_fingerprint",
            json!({"protection_scheme":"synthetic-opaque","key_id":"other-key","value_hex":"a0b1"}),
        ),
    ] {
        let mut changed = base.clone();
        changed[field] = value;
        let b: PersistentRecordIdentity = serde_json::from_value(changed).unwrap();
        assert_ne!(a, b);
    }
    let mut empty = base;
    empty["trace_id"] = json!("");
    assert!(serde_json::from_value::<PersistentRecordIdentity>(empty).is_err());
    assert!(!format!("{a:?}").contains("UNIT-A"));
}
#[test]
fn presence_session_and_revision_have_stable_non_equivalent_states() {
    assert_eq!(
        serde_json::to_string(&TracePresence::Absent).unwrap(),
        r#"{"kind":"absent"}"#
    );
    assert_eq!(
        serde_json::to_string(&TracePresence::Present { record_count: 0 }).unwrap(),
        r#"{"kind":"present","record_count":0}"#
    );
    assert_ne!(
        TracePresence::Absent,
        TracePresence::Present { record_count: 0 }
    );
    assert!(
        serde_json::from_value::<TracePresence>(json!({
            "kind":"absent", "record_count":0
        }))
        .is_err()
    );
    for (state, text) in [
        (SessionState::Draft, "draft"),
        (SessionState::Validated, "validated"),
        (SessionState::ApprovedImmutable, "approved_immutable"),
    ] {
        assert_eq!(
            serde_json::to_string(&state).unwrap(),
            format!("\"{text}\"")
        );
        assert_eq!(
            serde_json::from_value::<SessionState>(json!(text)).unwrap(),
            state
        );
    }
    assert!(serde_json::from_value::<SessionState>(json!("unknown")).is_err());
    assert!(
        serde_json::from_value::<TracePresence>(json!({"kind":"present","record_count":-1}))
            .is_err()
    );
    assert_eq!(SnapshotRevision::new(0).next().unwrap().value(), 1);
    assert_eq!(
        SnapshotRevision::new(u64::MAX).next().unwrap_err(),
        PersistenceTypeError::RevisionOverflow
    );
    assert!(serde_json::from_str::<SnapshotRevision>("-1").is_err());
}
#[test]
fn source_submission_matches_independent_golden_and_roundtrips_exactly() {
    let source = SourceSubmission::new(source_parts()).unwrap();
    let text = serde_json::to_string(&source).unwrap();
    assert_eq!(
        text,
        include_str!("fixtures/persistence/source-submission.json").trim_end()
    );
    let restored: SourceSubmission = serde_json::from_str(&text).unwrap();
    assert_eq!(source, restored);
    assert_eq!(serde_json::to_string(&restored).unwrap(), text);
    assert_eq!(
        source.traces()[0].presence,
        TracePresence::Present { record_count: 0 }
    );
    assert_eq!(source.traces()[1].presence, TracePresence::Absent);
    assert_eq!(source.state(), SessionState::ApprovedImmutable);
    assert!(!format!("{source:?}").contains("synthetic-submission-v1"));
    let mut offset: Value = serde_json::from_str(&text).unwrap();
    offset["created_at"] = json!("2026-01-02T04:04:05+01:00");
    assert_eq!(
        serde_json::from_value::<SourceSubmission>(offset).unwrap(),
        source
    );
}
#[test]
fn source_rejects_ambiguous_presence_duplicate_inventory_and_invalid_audit() {
    let mut p = source_parts();
    p.traces[0].input_sha256 = Some(Sha256Digest([0; 32]));
    assert_eq!(
        SourceSubmission::new(p).unwrap_err(),
        PersistenceTypeError::InconsistentTraceAudit
    );
    let mut p = source_parts();
    p.traces[1].input_sha256 = None;
    assert_eq!(
        SourceSubmission::new(p).unwrap_err(),
        PersistenceTypeError::InconsistentTraceAudit
    );
    let mut p = source_parts();
    p.traces.push(p.traces[0].clone());
    assert_eq!(
        SourceSubmission::new(p).unwrap_err(),
        PersistenceTypeError::DuplicateTrace
    );
    let mut p = source_parts();
    p.reference_data_versions
        .push(p.reference_data_versions[0].clone());
    assert_eq!(
        SourceSubmission::new(p).unwrap_err(),
        PersistenceTypeError::DuplicateReferenceVersion
    );
    let mut v = serde_json::to_value(SourceSubmission::new(source_parts()).unwrap()).unwrap();
    v["created_at"] = json!("not-a-date");
    assert!(serde_json::from_value::<SourceSubmission>(v).is_err());
    for text in ["", "00", "GG", "ÉÉ"] {
        assert!(serde_json::from_value::<Sha256Digest>(json!(text)).is_err());
    }
    let upper = "AB".repeat(32);
    assert!(serde_json::from_value::<Sha256Digest>(json!("É".repeat(32))).is_err());
    let digest: Sha256Digest = serde_json::from_value(json!(upper)).unwrap();
    assert_eq!(digest.to_string(), "ab".repeat(32));
}
#[test]
fn per_record_historical_diff_is_explicit_typed_and_rejects_inconsistent_claims() {
    let id = identity("SYNTHETIC-UNIT", "v1");
    let old = token(&[1]);
    let new = token(&[2]);
    let revision = Some(SnapshotRevision::new(4));
    for (kind, base, previous, current) in [
        (HistoricalChange::Inserted, None, None, Some(new.clone())),
        (
            HistoricalChange::Updated,
            revision,
            Some(old.clone()),
            Some(new.clone()),
        ),
        (HistoricalChange::Removed, revision, Some(old.clone()), None),
        (
            HistoricalChange::Reactivated,
            revision,
            Some(old.clone()),
            Some(old.clone()),
        ),
        (
            HistoricalChange::Unchanged,
            revision,
            Some(old.clone()),
            Some(old.clone()),
        ),
    ] {
        let diff = HistoricalDiff::new(id.clone(), base, kind, previous, current).unwrap();
        let text = serde_json::to_string(&diff).unwrap();
        assert_eq!(serde_json::from_str::<HistoricalDiff>(&text).unwrap(), diff);
    }
    assert!(
        HistoricalDiff::new(
            id.clone(),
            None,
            HistoricalChange::Removed,
            Some(old.clone()),
            None
        )
        .is_err()
    );
    assert!(
        HistoricalDiff::new(
            id.clone(),
            revision,
            HistoricalChange::Updated,
            Some(old.clone()),
            Some(old.clone())
        )
        .is_err()
    );
    let foreign = Fingerprint::new("synthetic-opaque".into(), "other-key".into(), vec![2]).unwrap();
    assert!(
        HistoricalDiff::new(
            id,
            revision,
            HistoricalChange::Updated,
            Some(old),
            Some(foreign)
        )
        .is_err()
    );
}
#[test]
fn pk_normalization_reuses_actual_compiled_projection_without_persistence_or_hash_defaults() {
    let mut value: Value = serde_json::from_slice(include_bytes!(
        "../bundles/embedded/schemas/SYNTHETIC/v1/flow.json"
    ))
    .unwrap();
    value["traces"][0]["fields"][0]["constraints"][0]["values"] = json!(["1", "2"]);
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let compiled = SchemaCompiler.compile(raw).unwrap();
    let trace = &compiled.traces()[0];
    let fields = &trace.record_key().field_indexes;
    let a = build_join_key(&RecordView::new(b"001A  "), fields, trace).unwrap();
    let same = build_join_key(&RecordView::new(b"001\xc3\x89 "), fields, trace).unwrap();
    let different = build_join_key(&RecordView::new(b"002A  "), fields, trace).unwrap();
    let expected: Vec<u8> = 1u64.to_be_bytes().into_iter().chain(*b"1").collect();
    assert_eq!(a.as_bytes(), expected);
    assert_eq!(a, same);
    assert_ne!(a, different);
    assert!(matches!(
        build_join_key(&RecordView::new(b"   A  "), fields, trace),
        Err(JoinKeyError::MissingComponent(_))
    ));
    // No raw JoinKey is accepted as a persistable identity; protected tokens
    // above are deliberately separate synthetic metadata, never a SHA fallback.
}
