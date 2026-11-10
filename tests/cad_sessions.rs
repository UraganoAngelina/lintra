//! CAD session-policy acceptance at the existing core approval boundary.
//! Summary values are explicit component fixtures, NOT historical/clinical Full
//! execution. SQLite, metadata protection and approval transactions are real.
use chrono::{TimeZone, Utc};
use lintra::*;
use serde_json::{Value, json};
use sha2::Digest;
use std::{collections::BTreeMap, io, sync::Arc, time::Duration};

const KEY_ID: &str = "synthetic-cad-session-key";
struct Provider;
impl PersistenceKeyProvider for Provider {
    fn resolve(&self, id: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        if id != KEY_ID {
            return Err(ProtectionError::MissingKey);
        }
        // Public fixture material; never provision an operational credential.
        Ok(Arc::new(PersistenceKey::new(vec![0xaa; 32])?))
    }
}
fn protector() -> IdentityProtector {
    IdentityProtector::preflight(Some(KEY_ID), &Provider, None).unwrap()
}
fn declaration() -> Value {
    serde_json::from_str(include_str!(
        "../schemas/cad/components/2024-06.namespace.1/component.json"
    ))
    .unwrap()
}
/// Only PK-shaped synthetic fields are supplied. These widths/layouts do not
/// impersonate the 83-field regional schema or its clinical/temporal checks.
fn raw() -> RawFlowSchema {
    let c = declaration();
    let mut v: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    v["flow_id"] = json!("CAD");
    v["schema_version"] = json!("2024-06.sessions-synthetic.1");
    v["schema_format_version"] = c["schema_format_version"].clone();
    v["required_capabilities"] = c["required_capabilities"].clone();
    v["parameters"] = c["parameters"].clone();
    v["persistence_policy"] = c["persistence_policy"].clone();
    let template = v["traces"][0].clone();
    v["traces"] = json!([]);
    for id in ["A", "B1", "B2", "C", "D"] {
        let mut t = template.clone();
        t["id"] = json!(id);
        t["identity"] = json!([]);
        t["fields"] = json!([]);
        for (index, key) in c["record_keys"][id]["fields"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let mut f = template["fields"][0].clone();
            f["id"] = key.clone();
            f["physical"]["start"] = json!(index * 3 + 1);
            f["physical"]["length"] = json!(3);
            f["constraints"] = json!([]);
            if key == "1.1" {
                f = c["namespace_field"].clone();
            }
            t["fields"].as_array_mut().unwrap().push(f);
        }
        t["record_length"] = json!(3 * t["fields"].as_array().unwrap().len());
        t["record_key"] = c["record_keys"][id].clone();
        v["traces"].as_array_mut().unwrap().push(t);
    }
    let mut r: RawFlowSchema = serde_json::from_value(v).unwrap();
    r.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    r
}
fn profile() -> CompiledPersistenceProfile {
    SchemaCompiler
        .compile_persistence_profile(&raw(), &[])
        .unwrap()
}
fn parameters(unit: &str, year: i64, quarter: i64) -> BTreeMap<String, NamespaceValue> {
    [
        ("sending_ulss".into(), NamespaceValue::String(unit.into())),
        ("year".into(), NamespaceValue::Integer(year)),
        ("quarter".into(), NamespaceValue::Integer(quarter)),
    ]
    .into()
}
fn namespace(unit: &str) -> PersistenceNamespace {
    PersistenceNamespace::new(vec![(
        "sending_ulss".into(),
        NamespaceValue::String(unit.into()),
    )])
    .unwrap()
}
fn parts(
    p: &CompiledPersistenceProfile,
    unit: &str,
    id: &str,
    absent: bool,
) -> SourceSubmissionParts {
    SourceSubmissionParts {
        namespace: PersistenceNamespace::new(vec![(
            "sending_ulss".into(),
            NamespaceValue::String(unit.into()),
        )])
        .unwrap(),
        submission_id: id.into(),
        schema_key: p.schema_key().clone(),
        schema_hash: p.schema_hash().clone(),
        period: None,
        created_at: Utc.with_ymd_and_hms(2026, 1, 2, 3, 0, 0).unwrap(),
        validator_version: "synthetic-test".into(),
        validation_mode: ValidationMode::Full,
        validation_status: RunStatus::CompletedValid,
        reference_data_versions: vec![],
        traces: p
            .trace_ids()
            .iter()
            .map(|trace_id| SubmissionTrace {
                trace_id: trace_id.clone(),
                presence: if absent && trace_id.0 == "D" {
                    TracePresence::Absent
                } else {
                    TracePresence::Present { record_count: 1 }
                },
                input_sha256: if absent && trace_id.0 == "D" {
                    None
                } else {
                    Some(Sha256Digest(sha2::Sha256::digest(id.as_bytes()).into()))
                },
            })
            .collect(),
        fingerprint: Fingerprint::new("synthetic-preparation".into(), KEY_ID.into(), vec![1, 2, 3])
            .unwrap(),
        state: SessionState::Draft,
    }
}
fn outcome(
    parts: &SourceSubmissionParts,
    parameters: &BTreeMap<String, NamespaceValue>,
) -> ValidationRunOutcome {
    let present: Vec<_> = parts
        .traces
        .iter()
        .filter(|t| matches!(t.presence, TracePresence::Present { .. }))
        .collect();
    let summary = ValidationSummary {
        started_at: parts.created_at.into(),
        validation_mode: ValidationMode::Full,
        run_status: RunStatus::CompletedValid,
        schema_key: parts.schema_key.clone(),
        scope_summary: ScopeSummary::Flow {
            trace_ids: present.iter().map(|t| t.trace_id.clone()).collect(),
        },
        records_processed: parts
            .traces
            .iter()
            .filter(|t| t.presence != TracePresence::Absent)
            .count() as u64,
        valid_records: parts
            .traces
            .iter()
            .filter(|t| t.presence != TracePresence::Absent)
            .count() as u64,
        invalid_records: 0,
        total_errors: 0,
        errors_by_code: BTreeMap::new(),
        skipped_rules: 0,
        truncated: false,
        validator_version: parts.validator_version.clone(),
        schema_hash: parts.schema_hash.to_string(),
        reference_data_versions: BTreeMap::new(),
        reference_validation: None,
        full_validation: Some(FullValidationSummary {
            cross_run_validation: None,
            completeness: ValidationCompleteness::Complete,
            parameters: parameters
                .iter()
                .map(|(id, v)| {
                    (
                        id.clone(),
                        match v {
                            NamespaceValue::String(v) => RunParameterAudit::String(v.clone()),
                            NamespaceValue::Integer(v) => RunParameterAudit::Integer(v.to_string()),
                        },
                    )
                })
                .collect(),
            traces: parts
                .traces
                .iter()
                .map(|t| FlowTraceEvidence {
                    trace_id: t.trace_id.clone(),
                    coverage: if t.presence == TracePresence::Absent {
                        ContextCoverage::Unavailable
                    } else {
                        ContextCoverage::Complete
                    },
                    records_processed: u64::from(t.presence != TracePresence::Absent),
                    unprojectable_records: 0,
                    cross_rules_declared: 0,
                    record_keys_checked: u64::from(t.presence != TracePresence::Absent),
                    duplicate_record_keys: 0,
                    inconclusive_record_keys: 0,
                })
                .collect(),
            controls: vec![],
        }),
        elapsed: Duration::ZERO,
        inputs: present
            .iter()
            .map(|t| InputAudit {
                trace_id: t.trace_id.clone(),
                file_name: "synthetic.dat".into(),
                file_size_bytes: 0,
                bytes_read: 0,
                completion: InputAuditCompletion::Complete {
                    sha256: t.input_sha256.clone().unwrap(),
                },
            })
            .collect(),
    };
    summary.validate().unwrap();
    ValidationRunOutcome {
        status: summary.run_status,
        summary,
    }
}

/// Identity/projection tuples are protected through the real core. This fixture
/// supplies synthetic canonical tuple values, not unvalidated regional records.
fn rows(p: &CompiledPersistenceProfile, unit: &str, absent: bool) -> Vec<StoredIdentity> {
    let protector = protector();
    p.trace_ids()
        .iter()
        .enumerate()
        .filter(|(_, id)| !absent || id.0 != "D")
        .map(|(index, id)| {
            // A standalone synthetic tuple extractor has the declared PK arity. It
            // never strips a policy/capability from the received CAD declaration.
            let mut v: Value =
                serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
            let template = v["traces"][0]["fields"][0].clone();
            let arity = p.record_key_fields(index).unwrap().len();
            let fields: Vec<_> = (0..arity)
                .map(|i| {
                    let mut f = template.clone();
                    f["id"] = json!(format!("K{i}"));
                    f["physical"]["start"] = json!(3 * i + 1);
                    f["physical"]["length"] = json!(3);
                    f["constraints"] = json!([]);
                    f
                })
                .collect();
            v["traces"][0]["fields"] = json!(fields);
            v["traces"][0]["record_length"] = json!(3 * arity);
            v["traces"][0]["identity"] = json!([]);
            v["traces"][0]["record_key"]["fields"] =
                json!((0..arity).map(|i| format!("K{i}")).collect::<Vec<_>>());
            let mut r: RawFlowSchema = serde_json::from_value(v).unwrap();
            r.encoding.resolved = Some(RawResolvedEncoding::Utf8);
            let schema = SchemaCompiler.compile(r).unwrap();
            let t = &schema.traces()[0];
            let bytes = "001".repeat(arity);
            let pk = build_join_key(
                &RecordView::new(bytes.as_bytes()),
                &t.record_key().field_indexes,
                t,
            )
            .unwrap();
            let scope = IdentityProtectionScope::new(
                namespace(unit),
                p.schema_key().flow_id.clone(),
                PersistenceSchemaScope::SchemaVersion {
                    version: p.schema_key().version.clone(),
                },
                id.clone(),
                "record_key".into(),
            )
            .unwrap();
            protector
                .protect_projection(&scope, &pk, p.schema_hash(), &pk)
                .unwrap()
        })
        .collect()
}
fn candidate(unit: &str, id: &str, year: i64, quarter: i64) -> ValidatedApproval {
    let p = profile();
    let params = parameters(unit, year, quarter);
    let parts = parts(&p, unit, id, false);
    let result = outcome(&parts, &params);
    ApprovalDraft::new(&p, params, parts, None)
        .unwrap()
        .validated(&result, rows(&p, unit, false), &protector())
        .unwrap()
}
fn approve(v: &ValidatedApproval, repo: &dyn PersistentContextRepository) -> CommitReceipt {
    v.preview(repo)
        .unwrap()
        .approve_commit(&CancellationToken::default(), |_| Ok(()))
        .unwrap()
}
fn immutable(v: &ValidatedApproval, repo: &dyn PersistentContextRepository) {
    match v.preview(repo) {
        Err(SessionApprovalError::Rejected(e)) => {
            assert_eq!(e.code(), Some("E_APPROVED_SESSION_IMMUTABLE"))
        }
        _ => panic!("an approved CAD quarter must reject replacement"),
    }
}

#[test]
fn cad_policy_binds_the_approved_scope_without_enabling_incomplete_runtime() {
    let p = profile();
    let plan = p.policy().unwrap().snapshot.as_ref().unwrap();
    assert_eq!(plan.scope_parameters, [0]);
    assert_eq!((plan.year_parameter, plan.quarter_parameter), (1, 2));
    assert_eq!(
        plan.missing_trace_warning_code.as_ref(),
        "W_CAD_MISSING_TRACE"
    );
    // Public compiler/audit service must keep refusing the unimplemented whole
    // CAD pipeline. Approval policy acceptance does not bypass these gates.
    assert!(SchemaCompiler.compile(raw()).is_err());
    assert!(SchemaCompiler.compile_historical_audit(raw(), &[]).is_err());
    assert_eq!(p.record_key_fields(3).unwrap().len(), 5);
    assert_eq!(p.record_key_fields(4).unwrap().len(), 5);
}

#[test]
fn three_quarters_survive_restart_and_cannot_be_replaced_even_by_new_schema_version() {
    let tmp = tempfile::tempdir_in(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target"))
        .unwrap();
    let root = tmp.path().join("history");
    let ns = namespace("501");
    let mut originals = Vec::new();
    for q in 1..=3 {
        let repo = if q == 1 {
            SQLitePersistenceAdapter::create(&root, Some(KEY_ID), &Provider).unwrap()
        } else {
            SQLitePersistenceAdapter::open(&root, Some(KEY_ID), &Provider).unwrap()
        };
        let v = candidate("501", &format!("synthetic-T{q}"), 2026, q);
        assert_eq!(approve(&v, &repo).revision().value(), q as u64);
        originals.push(v);
    }
    let repo = SQLitePersistenceAdapter::open(&root, Some(KEY_ID), &Provider).unwrap();
    let page = repo.load_submission_page(&ns, None).unwrap();
    assert_eq!(page.len(), 3);
    let mut periods = std::collections::BTreeSet::new();
    for (index, (revision, source)) in page.iter().enumerate() {
        assert_eq!(revision.value(), index as u64 + 1);
        assert_eq!(source.state(), SessionState::ApprovedImmutable);
        assert!(periods.insert(source.period().unwrap()));
        assert_eq!(source.traces().len(), 5);
        assert!(
            source
                .traces()
                .iter()
                .all(|t| t.presence == TracePresence::Present { record_count: 1 })
        );
        protector().verify_submission(source).unwrap();
        let snapshot = repo.load_snapshot(&ns, *revision).unwrap().unwrap();
        assert_eq!(snapshot.identities().len(), 5);
        for row in snapshot.identities() {
            protector().verify_stored_identity(row).unwrap();
        }
        immutable(
            &candidate(
                "501",
                &format!("replacement-{index}"),
                2026,
                index as i64 + 1,
            ),
            &repo,
        );
        // Exact retry is the original receipt, not a fourth publication.
        assert_eq!(approve(&originals[index], &repo).revision(), *revision);
    }
    // A new schema/validator version must not unlock the same approved quarter.
    let mut r = raw();
    r.schema_version = "2024-06.sessions-synthetic.2".into();
    let p = SchemaCompiler.compile_persistence_profile(&r, &[]).unwrap();
    let params = parameters("501", 2026, 1);
    let s = parts(&p, "501", "upgraded-replacement", false);
    let v = ApprovalDraft::new(&p, params.clone(), s.clone(), None)
        .unwrap()
        .validated(&outcome(&s, &params), rows(&p, "501", false), &protector())
        .unwrap();
    immutable(&v, &repo);
    assert_eq!(
        repo.current_generation(&ns).unwrap(),
        Some(SnapshotRevision::new(3))
    );
}

#[test]
fn revision_before_approval_and_abort_are_private_and_the_reviewed_candidate_is_exact() {
    let tmp = tempfile::tempdir_in(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target"))
        .unwrap();
    let root = tmp.path().join("history");
    let repo = SQLitePersistenceAdapter::create(&root, Some(KEY_ID), &Provider).unwrap();
    let p = profile();
    let params = parameters("501", 2026, 1);
    let old = parts(&p, "501", "draft-old", false);
    ApprovalDraft::new(&p, params.clone(), old, None)
        .unwrap()
        .cancel();
    let original = candidate("501", "validated-old", 2026, 1);
    original.preview(&repo).unwrap().cancel().unwrap();
    // The author may replace a preparation; dropping it does not create a
    // hidden historical version. Only the explicitly approved candidate exists.
    assert_eq!(repo.current_generation(&namespace("501")).unwrap(), None);
    let revised = candidate("501", "revised", 2026, 1);
    let mut review = revised.preview(&repo).unwrap();
    assert_eq!(review.snapshot().source().submission_id(), "revised");
    review
        .approve_commit(&CancellationToken::default(), |snapshot| {
            assert_eq!(snapshot.source().submission_id(), "revised");
            Ok(())
        })
        .unwrap();
    let page = repo.load_submission_page(&namespace("501"), None).unwrap();
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].1.submission_id(), "revised");
    immutable(&original, &repo);
}

#[test]
fn same_quarter_is_independent_across_ulss_and_next_year_preserves_the_namespace() {
    let repo = InMemoryPersistenceRepository::default();
    assert_eq!(
        approve(&candidate("501", "unit501-2026", 2026, 1), &repo)
            .revision()
            .value(),
        1
    );
    assert_eq!(
        approve(&candidate("502", "unit502-2026", 2026, 1), &repo)
            .revision()
            .value(),
        1
    );
    assert_eq!(
        approve(&candidate("501", "unit501-2027", 2027, 1), &repo)
            .revision()
            .value(),
        2
    );
    assert_eq!(
        repo.load_submission_page(&namespace("501"), None)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        repo.load_submission_page(&namespace("502"), None)
            .unwrap()
            .len(),
        1
    );
    immutable(&candidate("501", "retro2026", 2026, 1), &repo);
}

#[test]
fn report_failure_cancellation_and_stale_approval_leave_sqlite_generation_unchanged() {
    let tmp = tempfile::tempdir_in(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target"))
        .unwrap();
    let root = tmp.path().join("history");
    let repo = SQLitePersistenceAdapter::create(&root, Some(KEY_ID), &Provider).unwrap();
    let v = candidate("501", "one", 2026, 1);
    let other = candidate("501", "other", 2026, 1);
    let mut review = v.preview(&repo).unwrap();
    assert!(matches!(
        review.approve_commit(&CancellationToken::default(), |_| {
            Err(ReportError::Write(io::Error::other(
                "synthetic report failure",
            )))
        }),
        Err(SessionApprovalError::Report(_))
    ));
    let token = CancellationToken::default();
    assert!(matches!(
        review.approve_commit(&token, |_| {
            token.cancel();
            Ok(())
        }),
        Err(SessionApprovalError::Cancelled)
    ));
    assert_eq!(repo.current_generation(&namespace("501")).unwrap(), None);
    let mut first = v.preview(&repo).unwrap();
    let mut stale = other.preview(&repo).unwrap();
    first
        .approve_commit(&CancellationToken::default(), |_| Ok(()))
        .unwrap();
    assert!(matches!(
        stale.approve_commit(&CancellationToken::default(), |_| Ok(())),
        Err(SessionApprovalError::Persistence(
            PersistenceError::GenerationConflict { .. }
        ))
    ));
    drop(stale);
    drop(first);
    drop(review);
    drop(repo);
    let repo = SQLitePersistenceAdapter::open(&root, Some(KEY_ID), &Provider).unwrap();
    assert_eq!(
        repo.current_generation(&namespace("501")).unwrap(),
        Some(SnapshotRevision::new(1))
    );
    assert_eq!(
        repo.load_submission_page(&namespace("501"), None)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn invalid_or_unbound_cad_preparations_never_receive_an_approval() {
    let p = profile();
    for params in [
        parameters("500", 2026, 1),
        parameters("501", 2026, 0),
        parameters("501", 2026, 5),
        parameters("501", 0, 1),
        parameters("501", 10000, 1),
        parameters("502", 2026, 1),
    ] {
        assert!(ApprovalDraft::new(&p, params, parts(&p, "501", "bad", false), None).is_err());
    }
    let params = parameters("501", 2026, 1);
    for mutation in 0..4 {
        let s = parts(&p, "501", "invalid-outcome", false);
        let mut result = outcome(&s, &params);
        match mutation {
            0 => result.status = RunStatus::Cancelled,
            1 => result.summary.truncated = true,
            2 => {
                result
                    .summary
                    .full_validation
                    .as_mut()
                    .unwrap()
                    .completeness = ValidationCompleteness::Partial
            }
            _ => result
                .summary
                .full_validation
                .as_mut()
                .unwrap()
                .parameters
                .insert(
                    "sending_ulss".into(),
                    RunParameterAudit::String("502".into()),
                )
                .map(|_| ())
                .unwrap(),
        }
        assert!(
            ApprovalDraft::new(&p, params.clone(), s, None)
                .unwrap()
                .validated(&result, rows(&p, "501", false), &protector())
                .is_err()
        );
    }
}
