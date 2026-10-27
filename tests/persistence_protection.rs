//! Public RFC/synthetic material only; no patient data or production credentials.
use lintra::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::Arc;

struct Provider {
    id: String,
    key: Arc<PersistenceKey>,
}
impl Provider {
    fn new(id: &str, byte: u8) -> Self {
        Self {
            id: id.into(),
            key: Arc::new(PersistenceKey::new(vec![byte; 131]).unwrap()),
        }
    }
}
impl PersistenceKeyProvider for Provider {
    fn resolve(&self, id: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        if id == self.id {
            Ok(self.key.clone())
        } else {
            Err(ProtectionError::KeyUnavailable)
        }
    }
}
fn ns(value: NamespaceValue) -> PersistenceNamespace {
    PersistenceNamespace::new(vec![("unit".into(), value)]).unwrap()
}
fn scope() -> IdentityProtectionScope {
    IdentityProtectionScope::new(
        ns(NamespaceValue::String("UNIT-A".into())),
        FlowId("SYNTHETIC".into()),
        PersistenceSchemaScope::SchemaVersion {
            version: SchemaVersion("v1".into()),
        },
        TraceId("T1".into()),
        "record_key".into(),
    )
    .unwrap()
}
fn definition() -> Sha256Digest {
    Sha256Digest(Sha256::digest(b"SYNTHETIC/v1/T1:F2:logical:utf8").into())
}
fn keys(record: &[u8]) -> (JoinKey, JoinKey) {
    let mut raw: RawFlowSchema = serde_json::from_slice(include_bytes!(
        "../bundles/embedded/schemas/SYNTHETIC/v1/flow.json"
    ))
    .unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let schema = SchemaCompiler.compile(raw).unwrap();
    let trace = &schema.traces()[0];
    (
        build_join_key(
            &RecordView::new(record),
            &trace.record_key().field_indexes,
            trace,
        )
        .unwrap(),
        build_join_key(&RecordView::new(record), &[1], trace).unwrap(),
    )
}
#[test]
fn protected_projection_matches_a_golden_computed_with_independent_python_hmac() {
    let provider = Provider::new("test-key", 0xaa);
    let protector = IdentityProtector::preflight(Some("test-key"), &provider, None).unwrap();
    let (pk, projection) = keys(b"001A  ");
    let protected = protector
        .protect_projection(&scope(), &pk, &definition(), &projection)
        .unwrap();
    let actual = json!({"projection_definition":definition(),"audit":protector.audit().unwrap(),"key_fingerprint":protected.identity().key_fingerprint(),"record_fingerprint":protected.record_fingerprint()});
    let expected: Value = serde_json::from_slice(include_bytes!(
        "fixtures/persistence/protected-projection.json"
    ))
    .unwrap();
    assert_eq!(actual, expected);
    let again = protector
        .protect_projection(&scope(), &pk, &definition(), &projection)
        .unwrap();
    assert_eq!(protected, again);
    let (same_pk, utf8_projection) = keys(b"001\xc3\x89 ");
    let utf8 = protector
        .protect_projection(&scope(), &same_pk, &definition(), &utf8_projection)
        .unwrap();
    assert_eq!(protected.identity(), utf8.identity());
    assert_ne!(protected.record_fingerprint(), utf8.record_fingerprint());
}
#[test]
fn namespace_order_is_canonical_but_values_types_and_padding_are_not_normalized() {
    let provider = Provider::new("test-key", 0xaa);
    let protector = IdentityProtector::preflight(Some("test-key"), &provider, None).unwrap();
    let (pk, _) = keys(b"001A  ");
    let contexts = [
        NamespaceValue::String("050".into()),
        NamespaceValue::String(" 050 ".into()),
        NamespaceValue::String("UNIT-A".into()),
        NamespaceValue::String("unit-a".into()),
        NamespaceValue::Integer(50),
    ];
    let hashes: Vec<_> = contexts
        .into_iter()
        .map(|n| {
            protector
                .protect_key(
                    &IdentityProtectionScope::new(
                        ns(n),
                        FlowId("SYNTHETIC".into()),
                        PersistenceSchemaScope::SchemaVersion {
                            version: SchemaVersion("v1".into()),
                        },
                        TraceId("T1".into()),
                        "record_key".into(),
                    )
                    .unwrap(),
                    &pk,
                )
                .unwrap()
                .key_fingerprint()
                .as_bytes()
                .to_vec()
        })
        .collect();
    for (i, a) in hashes.iter().enumerate() {
        for b in &hashes[i + 1..] {
            assert_ne!(a, b);
        }
    }
    let parameters = vec![
        ("z".into(), NamespaceValue::Integer(1)),
        ("a".into(), NamespaceValue::String("exact".into())),
    ];
    let a = PersistenceNamespace::new(parameters.clone()).unwrap();
    let b = PersistenceNamespace::new(parameters.into_iter().rev().collect()).unwrap();
    let make = |namespace| {
        IdentityProtectionScope::new(
            namespace,
            FlowId("SYNTHETIC".into()),
            PersistenceSchemaScope::SchemaVersion {
                version: SchemaVersion("v1".into()),
            },
            TraceId("T1".into()),
            "record_key".into(),
        )
        .unwrap()
    };
    assert_eq!(
        protector.protect_key(&make(a), &pk).unwrap(),
        protector.protect_key(&make(b), &pk).unwrap()
    );
}
#[test]
fn flow_trace_version_group_and_key_kind_are_bound_inside_the_token() {
    let provider = Provider::new("test-key", 0xaa);
    let protector = IdentityProtector::preflight(Some("test-key"), &provider, None).unwrap();
    let (pk, _) = keys(b"001A  ");
    let base = protector.protect_key(&scope(), &pk).unwrap();
    for (flow, schema, trace, kind) in [
        (
            "OTHER",
            PersistenceSchemaScope::SchemaVersion {
                version: SchemaVersion("v1".into()),
            },
            "T1",
            "record_key",
        ),
        (
            "SYNTHETIC",
            PersistenceSchemaScope::SchemaVersion {
                version: SchemaVersion("v2".into()),
            },
            "T1",
            "record_key",
        ),
        (
            "SYNTHETIC",
            PersistenceSchemaScope::CompatibilityGroup { id: "v1".into() },
            "T1",
            "record_key",
        ),
        (
            "SYNTHETIC",
            PersistenceSchemaScope::SchemaVersion {
                version: SchemaVersion("v1".into()),
            },
            "T2",
            "record_key",
        ),
        (
            "SYNTHETIC",
            PersistenceSchemaScope::SchemaVersion {
                version: SchemaVersion("v1".into()),
            },
            "T1",
            "other_key",
        ),
    ] {
        let other = IdentityProtectionScope::new(
            ns(NamespaceValue::String("UNIT-A".into())),
            FlowId(flow.into()),
            schema,
            TraceId(trace.into()),
            kind.into(),
        )
        .unwrap();
        assert_ne!(
            base.key_fingerprint().as_bytes(),
            protector
                .protect_key(&other, &pk)
                .unwrap()
                .key_fingerprint()
                .as_bytes()
        );
    }
}
#[test]
fn record_tokens_bind_pk_payload_purpose_and_projection_definition() {
    let provider = Provider::new("test-key", 0xaa);
    let protector = IdentityProtector::preflight(Some("test-key"), &provider, None).unwrap();
    let (pk, a) = keys(b"001A  ");
    let (_, b) = keys(b"001B  ");
    let (pk2, _) = keys(b"002A  ");
    let base = protector
        .protect_projection(&scope(), &pk, &definition(), &a)
        .unwrap();
    let changed = protector
        .protect_projection(&scope(), &pk, &definition(), &b)
        .unwrap();
    assert_eq!(base.identity(), changed.identity());
    assert_ne!(base.record_fingerprint(), changed.record_fingerprint());
    let changed_key = protector
        .protect_projection(&scope(), &pk2, &definition(), &a)
        .unwrap();
    assert_ne!(base.identity(), changed_key.identity());
    assert_ne!(
        base.record_fingerprint().as_bytes(),
        changed_key.record_fingerprint().as_bytes()
    );
    let changed_definition = protector
        .protect_projection(&scope(), &pk, &Sha256Digest([1; 32]), &a)
        .unwrap();
    assert_eq!(base.identity(), changed_definition.identity());
    assert!(
        !base
            .record_fingerprint()
            .same_protection_context(changed_definition.record_fingerprint())
    );
    assert!(
        !base
            .record_fingerprint()
            .same_protection_context(base.identity().key_fingerprint())
    );
}
#[test]
fn missing_unavailable_short_and_changed_keys_are_preflight_errors_without_fallback() {
    let provider = Provider::new("test-key", 0xaa);
    assert!(matches!(
        IdentityProtector::preflight(None, &provider, None),
        Err(ProtectionError::MissingKey)
    ));
    assert!(matches!(
        IdentityProtector::preflight(Some(""), &provider, None),
        Err(ProtectionError::MissingKey)
    ));
    assert!(matches!(
        IdentityProtector::preflight(Some("unavailable"), &provider, None),
        Err(ProtectionError::KeyUnavailable)
    ));
    assert!(matches!(
        PersistenceKey::new(vec![0; 31]),
        Err(ProtectionError::ShortKey)
    ));
    let first = IdentityProtector::preflight(Some("test-key"), &provider, None).unwrap();
    let audit = first.audit().unwrap();
    IdentityProtector::preflight(Some("test-key"), &provider, Some(&audit)).unwrap();
    let wrong = Provider::new("test-key", 0xbb);
    assert!(matches!(
        IdentityProtector::preflight(Some("test-key"), &wrong, Some(&audit)),
        Err(ProtectionError::KeyMismatch)
    ));
    let (pk, _) = keys(b"001A  ");
    let second = IdentityProtector::preflight(Some("test-key"), &wrong, None).unwrap();
    assert_ne!(
        first.protect_key(&scope(), &pk).unwrap(),
        second.protect_key(&scope(), &pk).unwrap()
    );
    let different_id = Provider::new("different-key", 0xaa);
    assert!(matches!(
        IdentityProtector::preflight(Some("different-key"), &different_id, Some(&audit)),
        Err(ProtectionError::KeyMismatch)
    ));
}
#[test]
fn historical_audit_must_match_policy_role_length_and_the_actual_key() {
    let provider = Provider::new("test-key", 0xaa);
    let protector = IdentityProtector::preflight(Some("test-key"), &provider, None).unwrap();
    let encoded = serde_json::to_value(protector.audit().unwrap()).unwrap();
    let restored: ProtectionAudit = serde_json::from_value(encoded.clone()).unwrap();
    protector.verify_audit(&restored).unwrap();
    for (field, value, error) in [
        (
            "policy_id",
            json!("unsupported-version"),
            ProtectionError::InvalidAudit,
        ),
        (
            "protection_scheme",
            json!("other-role"),
            ProtectionError::InvalidAudit,
        ),
        ("value_hex", json!("aa"), ProtectionError::InvalidAudit),
    ] {
        let mut changed = encoded.clone();
        if field == "policy_id" {
            changed[field] = value;
        } else {
            changed["key_confirmation"][field] = value;
        }
        let audit: ProtectionAudit = serde_json::from_value(changed).unwrap();
        assert_eq!(protector.verify_audit(&audit).unwrap_err(), error);
    }
    let mut altered = encoded;
    let raw = altered["key_confirmation"]["value_hex"].as_str().unwrap();
    let mut bytes = raw.as_bytes().to_vec();
    bytes[0] = if bytes[0] == b'0' { b'1' } else { b'0' };
    altered["key_confirmation"]["value_hex"] = json!(String::from_utf8(bytes).unwrap());
    let audit: ProtectionAudit = serde_json::from_value(altered).unwrap();
    assert_eq!(
        protector.verify_audit(&audit).unwrap_err(),
        ProtectionError::KeyMismatch
    );
}
#[test]
fn technical_debug_and_outputs_do_not_contain_raw_key_or_record_material() {
    let provider = Provider::new("test-key", 0xaa);
    let protector = IdentityProtector::preflight(Some("test-key"), &provider, None).unwrap();
    let (pk, payload) = keys(b"001A  ");
    let protected = protector
        .protect_projection(&scope(), &pk, &definition(), &payload)
        .unwrap();
    let text = format!(
        "{:?} {:?} {:?} {:?}",
        provider.key,
        scope(),
        protector,
        protected
    );
    assert!(text.contains("[redacted]"));
    assert!(!text.contains("001A  "));
    assert!(!text.contains("170, 170"));
    let wire=serde_json::to_string(&json!({"identity":protected.identity(),"record":protected.record_fingerprint(),"audit":protector.audit().unwrap()})).unwrap();
    assert!(!wire.contains("001A  "));
    assert!(!wire.contains(&"aa".repeat(131)));
    assert!(matches!(
        IdentityProtectionScope::new(
            ns(NamespaceValue::Integer(1)),
            FlowId("bad\nmetadata".into()),
            PersistenceSchemaScope::SchemaVersion {
                version: SchemaVersion("v1".into())
            },
            TraceId("T1".into()),
            "record_key".into()
        ),
        Err(ProtectionError::InvalidContext)
    ));
    assert!(
        !ProtectionError::KeyMismatch
            .to_string()
            .contains("test-key")
    );
}
