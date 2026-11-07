//! Protected projection/approval component acceptance. Public summary fixtures
//! do NOT attest a clinical Full CAD execution. Real compiler, HMAC, SQLite,
//! diff, restart and approval paths operate on synthetic original-width rows.
use chrono::{TimeZone, Utc};
use lintra::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::Arc, time::Duration};

const KEY_ID: &str = "synthetic-cad-progressive-key";
struct Provider(u8);
impl PersistenceKeyProvider for Provider {
    fn resolve(&self, id: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        if id != KEY_ID {
            return Err(ProtectionError::MissingKey);
        }
        // Public fixture bytes; this never provisions an operational key.
        Ok(Arc::new(PersistenceKey::new(vec![self.0; 32])?))
    }
}
fn protector() -> IdentityProtector {
    IdentityProtector::preflight(Some(KEY_ID), &Provider(0xaa), None).unwrap()
}
fn component() -> Value {
    serde_json::from_str(include_str!(
        "../schemas/cad/components/2024-06.progressives.1/component.json"
    ))
    .unwrap()
}
fn filler(start: usize, length: usize) -> Value {
    json!({"id":format!("SYNTHETIC-{start}"),"name":"Unvalidated synthetic filler",
        "physical":{"type":"alphanumeric","start":start,"length":length,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}},
        "missing_value":{"kind":"spaces"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[]})
}
fn model() -> Value {
    let c = component();
    let mut value: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    value["flow_id"] = json!("CAD-PROGRESSIVE-COMPONENT-TEST");
    value["schema_version"] = c["component_version"].clone();
    value["regulatory_version"] = c["source"]["regulatory_version"].clone();
    for key in [
        "schema_format_version",
        "required_capabilities",
        "parameters",
        "persistence_policy",
    ] {
        value[key] = c[key].clone();
    }
    let template = value["traces"][0].clone();
    value["traces"] = json!([]);
    for id in ["A", "B1", "B2", "C", "D"] {
        let source = &c["traces"][id];
        let mut trace = template.clone();
        trace["id"] = json!(id);
        trace["identity"] = json!([]);
        trace["record_length"] = source["record_length"].clone();
        trace["record_key"] = source["record_key"].clone();
        let mut fields = vec![];
        let mut next = 1usize;
        for field in source["fields"].as_array().unwrap() {
            let start = field["physical"]["start"].as_u64().unwrap() as usize;
            assert!(start >= next);
            if start > next {
                fields.push(filler(next, start - next));
            }
            next = start + field["physical"]["length"].as_u64().unwrap() as usize;
            fields.push(field.clone());
        }
        let length = source["record_length"].as_u64().unwrap() as usize;
        if next <= length {
            fields.push(filler(next, length + 1 - next));
        }
        trace["fields"] = json!(fields);
        value["traces"].as_array_mut().unwrap().push(trace);
    }
    value
}
fn raw(value: Value) -> RawFlowSchema {
    let mut result: RawFlowSchema = serde_json::from_value(value).unwrap();
    result.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    result
}
fn profile(value: Value) -> CompiledPersistenceProfile {
    SchemaCompiler
        .compile_persistence_profile(&raw(value), &[])
        .unwrap()
}
type Rows = BTreeMap<String, Vec<Vec<u8>>>;
fn row(
    trace: &str,
    unit: &str,
    event: u64,
    progressive: u64,
    attribute: &str,
    date: &str,
) -> Vec<u8> {
    let c = component();
    let source = &c["traces"][trace];
    let mut bytes = vec![b' '; source["record_length"].as_u64().unwrap() as usize];
    for field in source["fields"].as_array().unwrap() {
        let value = match field["id"].as_str().unwrap() {
            "1.1" => unit.to_string(),
            "1.23" | "2.1" | "3.1" | "4.1" | "5.1" => format!("{event:012}"),
            "2.36" | "3.5" | "4.11" | "5.7" => format!("{progressive:012}"),
            "4.2" | "5.2" => date.to_string(),
            "4.4" | "5.3" => "1".into(),
            "4.5" | "5.4" => "01".into(),
            "2.3" | "3.4" | "4.7" | "5.6" => attribute.to_string(),
            id => panic!("unexpected fixture field {id}"),
        };
        let start = field["physical"]["start"].as_u64().unwrap() as usize - 1;
        let length = field["physical"]["length"].as_u64().unwrap() as usize;
        assert_eq!(value.len(), length);
        bytes[start..start + length].copy_from_slice(value.as_bytes());
    }
    bytes
}
fn rows(unit: &str) -> Rows {
    ["A", "B1", "B2", "C", "D"]
        .into_iter()
        .map(|trace| {
            (
                trace.into(),
                vec![row(
                    trace,
                    unit,
                    1,
                    1,
                    if trace == "D" { "01" } else { "1" },
                    "01012026",
                )],
            )
        })
        .collect()
}
fn namespace(unit: &str) -> PersistenceNamespace {
    PersistenceNamespace::new(vec![(
        "sending_ulss".into(),
        NamespaceValue::String(unit.into()),
    )])
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
fn record_count(presence: TracePresence) -> u64 {
    match presence {
        TracePresence::Present { record_count } => record_count,
        TracePresence::Absent => 0,
    }
}
fn parts(
    p: &CompiledPersistenceProfile,
    data: &Rows,
    unit: &str,
    id: &str,
    year: i64,
    quarter: i64,
    absent: &[&str],
) -> SourceSubmissionParts {
    SourceSubmissionParts {
        namespace: namespace(unit),
        submission_id: id.into(),
        schema_key: p.schema_key().clone(),
        schema_hash: p.schema_hash().clone(),
        period: None,
        created_at: Utc
            .with_ymd_and_hms(year as i32, quarter as u32 * 3, 1, 0, 0, 0)
            .unwrap(),
        validator_version: "synthetic-progressive-component-test".into(),
        validation_mode: ValidationMode::Full,
        validation_status: RunStatus::CompletedValid,
        reference_data_versions: vec![],
        traces: p
            .trace_ids()
            .iter()
            .map(|trace| {
                let missing = absent.contains(&trace.0.as_str());
                let mut hash = Sha256::new();
                let records = data.get(&trace.0).map(Vec::as_slice).unwrap_or(&[]);
                for record in records {
                    hash.update(record);
                    hash.update(b"\n");
                }
                SubmissionTrace {
                    trace_id: trace.clone(),
                    presence: if missing {
                        TracePresence::Absent
                    } else {
                        TracePresence::Present {
                            record_count: records.len() as u64,
                        }
                    },
                    input_sha256: (!missing).then(|| Sha256Digest(hash.finalize().into())),
                }
            })
            .collect(),
        fingerprint: Fingerprint::new("synthetic-preparation".into(), KEY_ID.into(), vec![1, 2, 3])
            .unwrap(),
        state: SessionState::Draft,
    }
}
fn projected(
    p: &CompiledPersistenceProfile,
    key: &IdentityProtector,
    data: &Rows,
    unit: &str,
    absent: &[&str],
) -> Vec<ProjectedHistoricalRecord> {
    let ns = namespace(unit);
    data.iter()
        .filter(|(trace, _)| !absent.contains(&trace.as_str()))
        .flat_map(|(trace, records)| {
            records
                .iter()
                .enumerate()
                .map(|(index, bytes)| {
                    ProjectedHistoricalRecord::project(
                        p,
                        &ns,
                        &TraceId(trace.clone()),
                        RawRecord {
                            line_number: index as u64 + 1,
                            bytes,
                        },
                        key,
                    )
                    .unwrap()
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
fn candidate<'key>(
    p: &CompiledPersistenceProfile,
    key: &'key IdentityProtector,
    data: &Rows,
    unit: &str,
    id: &str,
    period: (i64, i64),
    absent: &[&str],
) -> Result<InvariantValidatedApproval<'key>, SessionApprovalError> {
    let (year, quarter) = period;
    let params = parameters(unit, year, quarter);
    let source = parts(p, data, unit, id, year, quarter, absent);
    let outcome = outcome(&source, &params, data);
    let at = source.created_at;
    let mut draft = ApprovalDraft::new(p, params, source, None)?;
    if !absent.is_empty() {
        draft = draft.with_proceed_consent(
            MissingTraceProceedConsent::new(
                absent
                    .iter()
                    .map(|trace| TraceId((*trace).into()))
                    .collect(),
                at,
            )
            .unwrap(),
        )?;
    }
    draft.validated_with_identity_records(&outcome, projected(p, key, data, unit, absent), key)
}
fn archive() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir_in(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target"))
        .unwrap();
    let root = dir.path().join("archive");
    SQLitePersistenceAdapter::create(&root, Some(KEY_ID), &Provider(0xaa)).unwrap();
    (dir, root)
}
fn open(root: &std::path::Path) -> SQLitePersistenceAdapter {
    SQLitePersistenceAdapter::open(root, Some(KEY_ID), &Provider(0xaa)).unwrap()
}
fn approve(value: &InvariantValidatedApproval<'_>, repo: &dyn PersistentContextRepository) {
    value
        .preview(repo)
        .unwrap()
        .approve_commit(&CancellationToken::default(), |_| Ok(()))
        .unwrap();
}
fn diffs(
    p: &CompiledPersistenceProfile,
    key: &IdentityProtector,
    repo: &dyn PersistentContextRepository,
    preview: &ApprovalPreview<'_>,
) -> (HistoricalAuditCounts, Vec<HistoricalDiff>) {
    let service = HistoricalAuditService::for_identity_invariants(p, key, repo);
    let prepared = service
        .prepare(preview.snapshot(), &CancellationToken::default())
        .unwrap();
    let mut rows = vec![];
    let result = prepared
        .write_diffs(&CancellationToken::default(), |diff| {
            rows.push(diff.clone());
            Ok(())
        })
        .unwrap();
    (result.counts, rows)
}

#[test]
fn real_quarterly_progressive_diffs_survive_restarts_and_attribute_changes_in_complete_pk() {
    let p = profile(model());
    let key = protector();
    let (_dir, root) = archive();
    let mut data = rows("501");
    {
        let repo = open(&root);
        let value = candidate(&p, &key, &data, "501", "T1", (2026, 1), &[]).unwrap();
        let preview = value.preview(&repo).unwrap();
        assert_eq!(preview.snapshot().identities().len(), 13);
        assert_eq!(diffs(&p, &key, &repo, &preview).0.inserted, 4);
        preview
            .snapshot()
            .source()
            .traces()
            .iter()
            .for_each(|trace| assert_eq!(record_count(trace.presence), 1));
        approve(&value, &repo);
    }
    data.insert("B1".into(), vec![row("B1", "501", 1, 1, "2", "01012026")]);
    data.insert("B2".into(), vec![]);
    data.insert("C".into(), vec![row("C", "501", 1, 1, "2", "02012026")]);
    data.insert("D".into(), vec![row("D", "501", 1, 1, "02", "03012026")]);
    {
        let repo = open(&root);
        let value = candidate(&p, &key, &data, "501", "T2", (2026, 2), &[]).unwrap();
        let preview = value.preview(&repo).unwrap();
        let (counts, items) = diffs(&p, &key, &repo, &preview);
        assert_eq!(
            (counts.updated, counts.disappeared, counts.inserted),
            (3, 1, 0)
        );
        for trace in ["C", "D"] {
            assert!(
                items
                    .iter()
                    .any(|item| item.identity().trace_id().0 == trace
                        && item.change() == HistoricalChange::Updated)
            );
        }
        approve(&value, &repo);
    }
    data.insert("B2".into(), vec![row("B2", "501", 1, 1, "1", "01012026")]);
    let repo = open(&root);
    let value = candidate(&p, &key, &data, "501", "T3", (2026, 3), &[]).unwrap();
    let preview = value.preview(&repo).unwrap();
    let counts = diffs(&p, &key, &repo, &preview).0;
    assert_eq!(
        (counts.reactivated, counts.unchanged, counts.inserted),
        (1, 3, 0)
    );
    approve(&value, &repo);
    assert_eq!(
        repo.current_generation(&namespace("501")).unwrap(),
        Some(SnapshotRevision::new(3))
    );
}

#[test]
fn every_trace_rejects_reassignment_before_and_after_annulment_with_no_commit() {
    for trace in ["B1", "B2", "C", "D"] {
        let p = profile(model());
        let key = protector();
        let (_dir, root) = archive();
        let mut data = rows("501");
        {
            let repo = open(&root);
            approve(
                &candidate(&p, &key, &data, "501", "T1", (2026, 1), &[]).unwrap(),
                &repo,
            );
        }
        let mut bad = data.clone();
        bad.insert(
            trace.into(),
            vec![row(
                trace,
                "501",
                2,
                1,
                if trace == "D" { "01" } else { "1" },
                "01012026",
            )],
        );
        {
            let repo = open(&root);
            let error = candidate(&p, &key, &bad, "501", "T2-bad", (2026, 2), &[])
                .unwrap()
                .preview(&repo)
                .unwrap_err();
            let errors = error.validation_errors();
            assert_eq!(errors.len(), 1);
            errors[0].validate().unwrap();
            assert_eq!(
                errors[0].code.as_symbol().unwrap(),
                "E_CAD_HISTORICAL_PROGRESSIVE_REUSE"
            );
            assert_eq!(
                (&errors[0].trace_id.0, errors[0].line_number),
                (&trace.to_string(), 1)
            );
            assert_eq!(
                repo.current_generation(&namespace("501")).unwrap(),
                Some(SnapshotRevision::new(1))
            );
        }
        data.insert(trace.into(), vec![]);
        {
            let repo = open(&root);
            approve(
                &candidate(&p, &key, &data, "501", "T2", (2026, 2), &[]).unwrap(),
                &repo,
            );
        }
        let repo = open(&root);
        let error = candidate(&p, &key, &bad, "501", "T3-bad", (2026, 3), &[])
            .unwrap()
            .preview(&repo)
            .unwrap_err();
        assert_eq!(
            error.validation_errors()[0].code.as_symbol().unwrap(),
            "E_CAD_HISTORICAL_PROGRESSIVE_REUSE"
        );
        assert_eq!(
            repo.current_generation(&namespace("501")).unwrap(),
            Some(SnapshotRevision::new(2))
        );
        assert!(
            repo.load_submission(&namespace("501"), "T3-bad")
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn complete_pk_audit_and_progressive_audit_have_explicit_distinct_meanings() {
    let mut value = model();
    value["persistence_policy"]["cross_run_audit"] = json!([
        {"trace":"C","key":"record_key"},{"trace":"D","key":"record_key"}]);
    let p = profile(value);
    let key = protector();
    let (_dir, root) = archive();
    let repo = open(&root);
    let mut data = rows("501");
    approve(
        &candidate(&p, &key, &data, "501", "T1", (2026, 1), &[]).unwrap(),
        &repo,
    );
    data.insert("C".into(), vec![row("C", "501", 1, 1, "1", "02012026")]);
    data.insert("D".into(), vec![row("D", "501", 1, 1, "01", "02012026")]);
    let value = candidate(&p, &key, &data, "501", "T2", (2026, 2), &[]).unwrap();
    let preview = value.preview(&repo).unwrap();
    let ordinary = HistoricalAuditService::new(&p, &key, &repo);
    let prepared = ordinary
        .prepare(preview.snapshot(), &CancellationToken::default())
        .unwrap();
    let counts = prepared
        .write_diffs(&CancellationToken::default(), |_| Ok(()))
        .unwrap()
        .counts;
    assert_eq!(
        (counts.inserted, counts.disappeared, counts.updated),
        (2, 2, 0)
    );
    let (counts, items) = diffs(&p, &key, &repo, &preview);
    assert_eq!(
        (counts.updated, counts.unchanged, counts.inserted),
        (2, 2, 0)
    );
    assert!(
        items
            .iter()
            .all(|item| item.identity().key_kind() == "history_identity.progressive_record.v1")
    );
}

#[test]
fn trace_and_ulss_scope_are_independent_and_event_binding_continues_between_years() {
    let p = profile(model());
    let key = protector();
    let (_dir, root) = archive();
    let repo = open(&root);
    approve(
        &candidate(&p, &key, &rows("501"), "501", "unit501", (2026, 1), &[]).unwrap(),
        &repo,
    );
    let mut other = rows("502");
    other.insert(
        "A".into(),
        (2..=5)
            .map(|event| row("A", "502", event, 1, "1", "01012026"))
            .collect(),
    );
    for (index, trace) in ["B1", "B2", "C", "D"].into_iter().enumerate() {
        other.insert(
            trace.into(),
            vec![row(
                trace,
                "502",
                index as u64 + 2,
                1,
                if trace == "D" { "01" } else { "1" },
                "01012026",
            )],
        );
    }
    let value = candidate(&p, &key, &other, "502", "unit502", (2026, 1), &[]).unwrap();
    let preview = value.preview(&repo).unwrap();
    assert_eq!(diffs(&p, &key, &repo, &preview).0.inserted, 4);
    approve(&value, &repo);
    let mut next = rows("501");
    next.insert("A".into(), vec![row("A", "501", 9, 1, "1", "01012026")]);
    next.insert("B1".into(), vec![row("B1", "501", 9, 1, "1", "01012026")]);
    let error = candidate(&p, &key, &next, "501", "next-year", (2027, 1), &[])
        .unwrap()
        .preview(&repo)
        .unwrap_err();
    assert_eq!(
        error.validation_errors()[0].code.as_symbol().unwrap(),
        "E_CAD_HISTORICAL_PROGRESSIVE_REUSE"
    );
    assert_eq!(
        repo.current_generation(&namespace("501")).unwrap(),
        Some(SnapshotRevision::new(1))
    );
    assert_eq!(
        repo.current_generation(&namespace("502")).unwrap(),
        Some(SnapshotRevision::new(1))
    );
}

#[test]
fn omitted_files_are_not_reported_as_progressive_annulments() {
    let p = profile(model());
    let key = protector();
    let (_dir, root) = archive();
    let data = rows("501");
    {
        let repo = open(&root);
        approve(
            &candidate(&p, &key, &data, "501", "T1", (2026, 1), &[]).unwrap(),
            &repo,
        );
    }
    let repo = open(&root);
    let value = candidate(&p, &key, &data, "501", "T2", (2026, 2), &["D"]).unwrap();
    let preview = value.preview(&repo).unwrap();
    let (counts, items) = diffs(&p, &key, &repo, &preview);
    assert_eq!((counts.unchanged, counts.disappeared), (3, 0));
    assert!(items.iter().all(|item| item.identity().trace_id().0 != "D"));
    approve(&value, &repo);
    drop(repo);
    let repo = open(&root);
    let value = candidate(&p, &key, &data, "501", "T3", (2026, 3), &[]).unwrap();
    let preview = value.preview(&repo).unwrap();
    let counts = diffs(&p, &key, &repo, &preview).0;
    assert_eq!(
        (counts.unchanged, counts.reactivated, counts.disappeared),
        (4, 0, 0)
    );
}

#[test]
fn raw_tokens_missing_projections_and_ambiguous_secondary_keys_cannot_enter_approval() {
    let p = profile(model());
    let key = protector();
    let data = rows("501");
    let params = parameters("501", 2026, 1);
    let source = parts(&p, &data, "501", "T1", 2026, 1, &[]);
    let out = outcome(&source, &params, &data);
    let error = ApprovalDraft::new(&p, params.clone(), source.clone(), None)
        .unwrap()
        .validated(&out, vec![], &key)
        .unwrap_err();
    assert!(matches!(
        error,
        SessionApprovalError::UnsupportedSessionPolicy
    ));
    let error = ApprovalDraft::new(&p, params.clone(), source.clone(), None)
        .unwrap()
        .validated_with_identity_records(&out, vec![], &key)
        .unwrap_err();
    assert!(matches!(
        error,
        SessionApprovalError::InvalidValidationEvidence
    ));
    let mut changed = model();
    changed["schema_version"] = json!("synthetic-other-definition");
    let other = profile(changed);
    let error = ApprovalDraft::new(&p, params, source, None)
        .unwrap()
        .validated_with_identity_records(&out, projected(&other, &key, &data, "501", &[]), &key)
        .unwrap_err();
    assert!(matches!(
        error,
        SessionApprovalError::InvalidValidationEvidence
    ));
    let bytes = row("C", "501", 1, 1, "1", "01012026");
    assert!(matches!(
        ProjectedHistoricalRecord::project(
            &p,
            &namespace("501"),
            &TraceId("C".into()),
            RawRecord {
                line_number: 0,
                bytes: &bytes
            },
            &key
        ),
        Err(HistoricalProjectionError::InvalidRecord)
    ));
    assert!(matches!(
        ProjectedHistoricalRecord::project(
            &p,
            &namespace("501"),
            &TraceId("C".into()),
            RawRecord {
                line_number: 1,
                bytes: &bytes[..bytes.len() - 1]
            },
            &key
        ),
        Err(HistoricalProjectionError::InvalidRecord)
    ));
    let mut ambiguous = data.clone();
    ambiguous.insert(
        "C".into(),
        vec![bytes, row("C", "501", 1, 1, "1", "02012026")],
    );
    assert!(matches!(
        candidate(&p, &key, &ambiguous, "501", "ambiguous", (2026, 1), &[]),
        Err(SessionApprovalError::Persistence(
            PersistenceError::DuplicateIdentity
        ))
    ));
}

#[test]
fn cancellation_report_failure_and_cas_preserve_the_checked_event_binding() {
    let p = profile(model());
    let key = protector();
    let (_dir, root) = archive();
    let repo = open(&root);
    let data = rows("501");
    let value = candidate(&p, &key, &data, "501", "first", (2026, 1), &[]).unwrap();
    let cancelled = CancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        value.preview_with_cancellation(&repo, &cancelled),
        Err(SessionApprovalError::Cancelled)
    ));
    let mut preview = value.preview(&repo).unwrap();
    assert!(matches!(
        preview.approve_commit(&CancellationToken::default(), |_| Err(ReportError::Write(
            std::io::Error::other("synthetic report failure")
        ))),
        Err(SessionApprovalError::Report(_))
    ));
    assert_eq!(repo.current_generation(&namespace("501")).unwrap(), None);
    let mut competing = data.clone();
    competing.insert("B1".into(), vec![row("B1", "501", 2, 1, "1", "01012026")]);
    let rival = candidate(&p, &key, &competing, "501", "rival", (2026, 1), &[]).unwrap();
    let mut stale = rival.preview(&repo).unwrap();
    approve(&value, &repo);
    assert!(matches!(
        stale.approve_commit(&CancellationToken::default(), |_| Ok(())),
        Err(SessionApprovalError::Persistence(
            PersistenceError::GenerationConflict { .. }
        ))
    ));
    assert_eq!(
        repo.current_generation(&namespace("501")).unwrap(),
        Some(SnapshotRevision::new(1))
    );
}

#[test]
fn incompatible_key_or_schema_history_is_a_technical_preflight_failure() {
    let p = profile(model());
    let key = protector();
    let (_dir, root) = archive();
    let repo = open(&root);
    let data = rows("501");
    let wrong = IdentityProtector::preflight(Some(KEY_ID), &Provider(0xbb), None).unwrap();
    let error = candidate(&p, &wrong, &data, "501", "wrong-key", (2026, 1), &[])
        .unwrap()
        .preview(&repo)
        .unwrap_err();
    assert!(matches!(
        error,
        SessionApprovalError::Persistence(PersistenceError::Protection(_))
    ));
    approve(
        &candidate(&p, &key, &data, "501", "T1", (2026, 1), &[]).unwrap(),
        &repo,
    );
    let mut changed = model();
    changed["schema_version"] = json!("synthetic-new-version");
    let other = profile(changed);
    let error = candidate(&other, &key, &data, "501", "T2", (2026, 2), &[])
        .unwrap()
        .preview(&repo)
        .unwrap_err();
    assert!(error.validation_errors().is_empty());
    assert!(matches!(
        error,
        SessionApprovalError::HistoricalContext(HistoricalAuditError::IncompatibleSchema)
    ));
    assert_eq!(
        repo.current_generation(&namespace("501")).unwrap(),
        Some(SnapshotRevision::new(1))
    );
}

#[test]
fn incomplete_secondary_history_is_not_accepted_as_a_verified_baseline() {
    let p = profile(model());
    let key = protector();
    let (_dir, root) = archive();
    let repo = open(&root);
    let data = rows("501");
    approve(
        &candidate(&p, &key, &data, "501", "T1", (2026, 1), &[]).unwrap(),
        &repo,
    );
    let original = repo
        .load_current_snapshot(&namespace("501"))
        .unwrap()
        .unwrap();
    let prepared = candidate(&p, &key, &data, "501", "incomplete-T2", (2026, 2), &[]).unwrap();
    let mut source = parts(&p, &data, "501", "incomplete-T2", 2026, 2, &[]);
    source.period = prepared.source().period().map(str::to_owned);
    source.state = SessionState::ApprovedImmutable;
    let source = key
        .protect_submission(&SourceSubmission::new(source).unwrap())
        .unwrap();
    // The generic storage port accepts authenticated values, not clinical proof.
    // Simulate an imported snapshot with valid MACs but missing secondary rows.
    let snapshot = HistoricalSnapshot::new(
        SnapshotRevision::new(2),
        source,
        original
            .identities()
            .iter()
            .filter(|row| row.identity().key_kind() == "record_key")
            .cloned()
            .collect(),
    )
    .unwrap();
    let mut transaction = repo
        .begin_transaction(&namespace("501"), Some(SnapshotRevision::new(1)))
        .unwrap();
    transaction.stage_snapshot(&snapshot).unwrap();
    transaction.commit().unwrap();
    let error = candidate(&p, &key, &data, "501", "T3", (2026, 3), &[])
        .unwrap()
        .preview(&repo)
        .unwrap_err();
    assert!(error.validation_errors().is_empty());
    assert!(matches!(
        error,
        SessionApprovalError::HistoricalContext(HistoricalAuditError::InconsistentHistory)
    ));
}

#[test]
fn original_line_and_private_deterministic_jsonl_survive_projection() {
    let p = profile(model());
    let key = protector();
    let (_dir, root) = archive();
    let repo = open(&root);
    let mut data = rows("501");
    data.insert(
        "B1".into(),
        vec![
            row("B1", "501", 1, 1, "1", "01012026"),
            row("B1", "501", 1, 812345678901, "1", "01012026"),
        ],
    );
    approve(
        &candidate(&p, &key, &data, "501", "T1", (2026, 1), &[]).unwrap(),
        &repo,
    );
    data.get_mut("B1").unwrap()[1] = row("B1", "501", 912345678901, 812345678901, "1", "01012026");
    let error = candidate(&p, &key, &data, "501", "T2", (2026, 2), &[])
        .unwrap()
        .preview(&repo)
        .unwrap_err();
    let diagnostic = &error.validation_errors()[0];
    diagnostic.validate().unwrap();
    assert_eq!(diagnostic.line_number, 2);
    assert!(diagnostic.actual.is_none() && diagnostic.context.is_empty());
    let emit = || {
        let mut writer = JsonlSink::new(Vec::new());
        writer.on_error(diagnostic).unwrap();
        writer.finish().unwrap()
    };
    let bytes = emit();
    assert_eq!(bytes, emit());
    assert!(!bytes.contains(&b'\r'));
    let output = String::from_utf8(bytes).unwrap();
    assert!(!output.contains("912345678901") && !output.contains("812345678901"));
    let debug = format!("{error:?}");
    assert!(!debug.contains("912345678901") && !debug.contains("812345678901"));
    let database = std::fs::read(root.join("lintra.db")).unwrap();
    for raw in [b"812345678901", b"912345678901"] {
        assert!(!database.windows(raw.len()).any(|window| window == raw));
    }
}

#[test]
fn component_provenance_and_unsupported_declarations_are_checked_before_input() {
    let c = component();
    assert_eq!(
        Sha256Digest(
            Sha256::digest(include_bytes!(
                "../schemas/cad/components/2024-06.namespace.1/component.json"
            ))
            .into()
        )
        .to_string(),
        c["source"]["namespace_component_sha256"].as_str().unwrap()
    );
    assert_eq!(c["full_normative_compliance"], false);
    for trace in ["A", "B1", "B2", "C", "D"] {
        for original in c["source_fields"][trace].as_array().unwrap() {
            let lowered = c["traces"][trace]["fields"]
                .as_array()
                .unwrap()
                .iter()
                .find(|field| field["id"] == original["id"])
                .unwrap();
            for key in ["start", "length"] {
                assert_eq!(lowered["physical"][key], original["physical"][key]);
            }
        }
    }
    let p = profile(model());
    assert_eq!(p.policy().unwrap().history_identity_invariants.len(), 4);
    assert_eq!(
        (
            p.record_key_fields(3).unwrap().len(),
            p.record_key_fields(4).unwrap().len()
        ),
        (5, 5)
    );
    assert!(SchemaCompiler.compile(raw(model())).is_err());
    assert!(
        SchemaCompiler
            .compile_historical_audit(raw(model()), &[])
            .is_err()
    );
    let mut cases = vec![];
    let mut v = model();
    v["persistence_policy"]["history_identity_invariants"][0]["progressive_fields"] = json!([]);
    cases.push(v);
    let mut v = model();
    v["persistence_policy"]["history_identity_invariants"][0]["event_fields"] = json!(["UNKNOWN"]);
    cases.push(v);
    let mut v = model();
    v["persistence_policy"]["history_identity_invariants"][0]["reactivate_same_event"] =
        json!(false);
    cases.push(v);
    let mut v = model();
    v["persistence_policy"]["history_identity_invariants"]
        .as_array_mut()
        .unwrap()
        .push(c["persistence_policy"]["history_identity_invariants"][0].clone());
    cases.push(v);
    let mut v = model();
    v["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .retain(|cap| cap != "persistence.audit.v1");
    cases.push(v);
    let mut v = model();
    v["persistence_policy"]["version_compatibility"] = json!({"kind":"declared_group","id":"unqualified","schema_versions":["2024-06.progressives.1"]});
    cases.push(v);
    for value in cases {
        assert!(
            SchemaCompiler
                .compile_persistence_profile(&raw(value), &[])
                .is_err()
        );
    }
}

#[test]
fn exact_old_quarter_retry_after_later_updates_does_not_publish_again() {
    let p = profile(model());
    let key = protector();
    let (_dir, root) = archive();
    let original = rows("501");
    {
        let repo = open(&root);
        approve(
            &candidate(&p, &key, &original, "501", "T1", (2026, 1), &[]).unwrap(),
            &repo,
        );
    }
    let mut updated = original.clone();
    updated.insert("B1".into(), vec![row("B1", "501", 1, 1, "2", "01012026")]);
    {
        let repo = open(&root);
        approve(
            &candidate(&p, &key, &updated, "501", "T2", (2026, 2), &[]).unwrap(),
            &repo,
        );
    }
    let repo = open(&root);
    let replay = candidate(&p, &key, &original, "501", "T1", (2026, 1), &[]).unwrap();
    let mut preview = replay.preview(&repo).unwrap();
    assert_eq!(preview.state(), Some(SessionState::ApprovedImmutable));
    assert_eq!(
        preview
            .approve_commit(&CancellationToken::default(), |_| Ok(()))
            .unwrap()
            .revision(),
        SnapshotRevision::new(1)
    );
    assert_eq!(
        repo.current_generation(&namespace("501")).unwrap(),
        Some(SnapshotRevision::new(2))
    );
}

#[test]
fn renamed_generic_flow_traces_fields_and_error_code_use_the_same_invariant_engine() {
    let mut value = model();
    value["flow_id"] = json!("GENERIC-IDENTITY");
    let names: BTreeMap<String, String> = value["traces"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, trace)| {
            (
                trace["id"].as_str().unwrap().to_owned(),
                format!("TRACE-{index}"),
            )
        })
        .collect();
    let mut fields = BTreeMap::<String, BTreeMap<String, String>>::new();
    for (index, trace) in value["traces"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        let old = trace["id"].as_str().unwrap().to_owned();
        let mapping: BTreeMap<String, String> = trace["fields"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(field_index, field)| {
                (
                    field["id"].as_str().unwrap().to_owned(),
                    format!("FIELD-{index}-{field_index}"),
                )
            })
            .collect();
        for field in trace["fields"].as_array_mut().unwrap() {
            field["id"] = json!(mapping[field["id"].as_str().unwrap()]);
        }
        for field in trace["record_key"]["fields"].as_array_mut().unwrap() {
            *field = json!(mapping[field.as_str().unwrap()]);
        }
        trace["id"] = json!(names[&old]);
        fields.insert(old, mapping);
    }
    for entry in value["persistence_policy"]["run_uniqueness"]
        .as_array_mut()
        .unwrap()
    {
        entry["trace"] = json!(names[entry["trace"].as_str().unwrap()]);
    }
    for entry in value["persistence_policy"]["namespace_bindings"]
        .as_array_mut()
        .unwrap()
    {
        let old = entry["trace"].as_str().unwrap().to_owned();
        entry["field"] = json!(fields[&old][entry["field"].as_str().unwrap()]);
        entry["trace"] = json!(names[&old]);
    }
    for entry in value["persistence_policy"]["history_identity_invariants"]
        .as_array_mut()
        .unwrap()
    {
        let old = entry["trace"].as_str().unwrap().to_owned();
        for key in ["progressive_fields", "event_fields"] {
            for field in entry[key].as_array_mut().unwrap() {
                *field = json!(fields[&old][field.as_str().unwrap()]);
            }
        }
        entry["trace"] = json!(names[&old]);
        entry["on_reassignment_error"] = json!("E_GENERIC_BINDING_CHANGE");
    }
    let p = profile(value);
    let key = protector();
    let (_dir, root) = archive();
    let repo = open(&root);
    let mut data: Rows = rows("501")
        .into_iter()
        .map(|(id, rows)| (names[&id].clone(), rows))
        .collect();
    approve(
        &candidate(&p, &key, &data, "501", "T1", (2026, 1), &[]).unwrap(),
        &repo,
    );
    data.get_mut(&names["B1"]).unwrap()[0][..12].copy_from_slice(b"000000000002");
    let error = candidate(&p, &key, &data, "501", "T2", (2026, 2), &[])
        .unwrap()
        .preview(&repo)
        .unwrap_err();
    let error = &error.validation_errors()[0];
    assert_eq!(error.flow_id.0, "GENERIC-IDENTITY");
    assert_eq!(error.trace_id.0, names["B1"]);
    assert_eq!(error.code.as_symbol().unwrap(), "E_GENERIC_BINDING_CHANGE");
}

fn outcome(
    parts: &SourceSubmissionParts,
    parameters: &BTreeMap<String, NamespaceValue>,
    data: &Rows,
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
            .map(|trace| record_count(trace.presence))
            .sum(),
        valid_records: parts
            .traces
            .iter()
            .map(|trace| record_count(trace.presence))
            .sum(),
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
                    records_processed: record_count(t.presence),
                    unprojectable_records: 0,
                    cross_rules_declared: 0,
                    record_keys_checked: record_count(t.presence),
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
                file_size_bytes: data
                    .get(&t.trace_id.0)
                    .into_iter()
                    .flatten()
                    .map(|record| record.len() as u64 + 1)
                    .sum(),
                bytes_read: data
                    .get(&t.trace_id.0)
                    .into_iter()
                    .flatten()
                    .map(|record| record.len() as u64 + 1)
                    .sum(),
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

#[test]
fn one_submission_cannot_bind_the_same_progressive_to_two_different_events() {
    let p = profile(model());
    let key = protector();
    for trace in ["B1", "B2", "C", "D"] {
        let mut data = rows("501");
        data.insert(
            "A".into(),
            vec![
                row("A", "501", 1, 1, "1", "01012026"),
                row("A", "501", 2, 1, "1", "01012026"),
            ],
        );
        data.insert(
            trace.into(),
            vec![
                row(
                    trace,
                    "501",
                    1,
                    1,
                    if trace == "D" { "01" } else { "1" },
                    "01012026",
                ),
                row(
                    trace,
                    "501",
                    2,
                    1,
                    if trace == "D" { "01" } else { "1" },
                    "01012026",
                ),
            ],
        );
        let error = candidate(
            &p,
            &key,
            &data,
            "501",
            "conflicting-current",
            (2026, 1),
            &[],
        )
        .unwrap_err();
        assert_eq!(
            error.validation_errors().len(),
            1,
            "expected declared normative rejection for {trace}: {error}"
        );
        let error = &error.validation_errors()[0];
        error.validate().unwrap();
        assert_eq!(
            error.code.as_symbol().unwrap(),
            "E_CAD_HISTORICAL_PROGRESSIVE_REUSE"
        );
        assert_eq!(error.trace_id.0, trace);
        assert_eq!(error.line_number, 2);
    }
}

#[test]
fn invalid_current_evidence_is_technical_before_binding_comparison() {
    let p = profile(model());
    let key = protector();
    let wrong = IdentityProtector::preflight(Some(KEY_ID), &Provider(0xbb), None).unwrap();
    let mut data = rows("501");
    data.insert(
        "B1".into(),
        vec![
            row("B1", "501", 1, 1, "1", "01012026"),
            row("B1", "501", 2, 1, "1", "01012026"),
        ],
    );
    let params = parameters("501", 2026, 1);
    let source = parts(&p, &data, "501", "bad-key", 2026, 1, &[]);
    let out = outcome(&source, &params, &data);
    let error = ApprovalDraft::new(&p, params.clone(), source, None)
        .unwrap()
        .validated_with_identity_records(&out, projected(&p, &wrong, &data, "501", &[]), &key)
        .unwrap_err();
    assert!(error.validation_errors().is_empty());
    assert!(matches!(
        error,
        SessionApprovalError::Persistence(PersistenceError::Protection(_))
    ));

    let mut source = parts(&p, &data, "501", "bad-count", 2026, 1, &[]);
    source
        .traces
        .iter_mut()
        .find(|trace| trace.trace_id.0 == "B1")
        .unwrap()
        .presence = TracePresence::Present { record_count: 1 };
    let out = outcome(&source, &params, &data);
    let error = ApprovalDraft::new(&p, params, source, None)
        .unwrap()
        .validated_with_identity_records(&out, projected(&p, &key, &data, "501", &[]), &key)
        .unwrap_err();
    assert!(error.validation_errors().is_empty());
    assert!(matches!(
        error,
        SessionApprovalError::InvalidValidationEvidence
    ));
}

/// CAD05 qualifies the existing warning/consent/omission contract. The public
/// Full summaries remain explicit component fixtures, not clinical executions.
mod missing_trace_acceptance {
    use super::*;
    use std::{
        fs,
        io::Write,
        path::{Path, PathBuf},
    };

    /// Obtain the same private inherited directory policy used by the qualified
    /// CLI fixture. All key bytes are PUBLIC synthetic material. The key bundle
    /// is distinct from the actual archive; no production credential is made.
    fn portable_archive() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let (dir, root) = archive();
        let bundle = dir.path().join("public synthetic portable bundle");
        drop(SQLitePersistenceAdapter::create(&bundle, Some(KEY_ID), &Provider(0xaa)).unwrap());
        let path = portable_persistence_key_path(&bundle.join("lintra-ui.exe")).unwrap();
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(&path).unwrap().write_all(&[0xaa; 32]).unwrap();
        (dir, root, path)
    }
    fn provider(path: &Path) -> FilePersistenceKeyProvider {
        FilePersistenceKeyProvider::load(KEY_ID, path).unwrap()
    }
    /// Reuse the exact qualified projection profile and original-width rows;
    /// do not delete snapshot/invariant obligations to make Full appear runnable.
    fn prepared(
        p: &CompiledPersistenceProfile,
        key: &IdentityProtector,
        data: &Rows,
        id: &str,
        quarter: i64,
        absent: &[&str],
    ) -> (
        ApprovalDraft,
        ValidationRunOutcome,
        Vec<ProjectedHistoricalRecord>,
    ) {
        let params = parameters("501", 2026, quarter);
        let source = parts(p, data, "501", id, 2026, quarter, absent);
        let result = outcome(&source, &params, data);
        let draft = ApprovalDraft::new(p, params, source, None).unwrap();
        (draft, result, projected(p, key, data, "501", absent))
    }
    fn decision(traces: &[&str], quarter: u32) -> MissingTraceProceedConsent {
        MissingTraceProceedConsent::new(
            traces.iter().map(|id| TraceId((*id).into())).collect(),
            Utc.with_ymd_and_hms(2026, quarter * 3, 1, 0, 1, 0).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn b2_warning_is_available_before_review_and_missing_consent_cannot_enter_it() {
        let (_dir, root, key_path) = portable_archive();
        let provider = provider(&key_path);
        let key = IdentityProtector::preflight(Some(KEY_ID), &provider, None).unwrap();
        let repo = SQLitePersistenceAdapter::open(&root, Some(KEY_ID), &provider).unwrap();
        let p = profile(model());
        let data = rows("501");
        let (draft, result, records) = prepared(&p, &key, &data, "missing-B2", 1, &["B2"]);
        assert_eq!(
            draft.missing_trace_warning_code(),
            Some("W_CAD_MISSING_TRACE")
        );
        assert_eq!(
            draft
                .absent_traces()
                .map(|id| id.0.as_str())
                .collect::<Vec<_>>(),
            ["B2"]
        );
        assert!(matches!(
            draft.validated_with_identity_records(&result, records, &key),
            Err(SessionApprovalError::Rejected(
                SessionRejection::MissingProceedConsent
            ))
        ));
        assert_eq!(repo.current_generation(&namespace("501")).unwrap(), None);
        assert!(
            repo.load_submission_page(&namespace("501"), None)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn abort_of_b2_omission_draft_and_review_preserves_the_approved_history() {
        let (_dir, root, key_path) = portable_archive();
        let provider = provider(&key_path);
        let key = IdentityProtector::preflight(Some(KEY_ID), &provider, None).unwrap();
        let p = profile(model());
        let data = rows("501");
        {
            let repo = SQLitePersistenceAdapter::open(&root, Some(KEY_ID), &provider).unwrap();
            approve(
                &candidate(&p, &key, &data, "501", "baseline", (2026, 1), &[]).unwrap(),
                &repo,
            );
            let (draft, _, _) = prepared(&p, &key, &data, "abort-draft", 2, &["B2"]);
            assert_eq!(
                draft.missing_trace_warning_code(),
                Some("W_CAD_MISSING_TRACE")
            );
            draft.cancel();
            let (draft, result, records) = prepared(&p, &key, &data, "abort-review", 2, &["B2"]);
            let value = draft
                .with_proceed_consent(decision(&["B2"], 2))
                .unwrap()
                .validated_with_identity_records(&result, records, &key)
                .unwrap();
            let mut review = value.preview(&repo).unwrap();
            review.cancel().unwrap();
            assert!(
                review
                    .approve_commit(&CancellationToken::default(), |_| panic!(
                        "ABORT must not publish a report"
                    ))
                    .is_err()
            );
            assert_eq!(
                repo.current_generation(&namespace("501")).unwrap(),
                Some(SnapshotRevision::new(1))
            );
        }
        let repo = SQLitePersistenceAdapter::open(&root, Some(KEY_ID), &provider).unwrap();
        let page = repo.load_submission_page(&namespace("501"), None).unwrap();
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].1.submission_id(), "baseline");
        assert!(page[0].1.missing_trace_consent().is_none());
    }

    #[test]
    fn proceed_persists_b2_absence_and_choice_with_file_key_and_no_annulment_after_restart() {
        let (_dir, root, key_path) = portable_archive();
        let p = profile(model());
        let data = rows("501");
        let consent = decision(&["B2"], 2);
        {
            let provider = provider(&key_path);
            let key = IdentityProtector::preflight(Some(KEY_ID), &provider, None).unwrap();
            let repo = SQLitePersistenceAdapter::open(&root, Some(KEY_ID), &provider).unwrap();
            approve(
                &candidate(&p, &key, &data, "501", "T1-all", (2026, 1), &[]).unwrap(),
                &repo,
            );
            let (draft, result, records) = prepared(&p, &key, &data, "T2-without-B2", 2, &["B2"]);
            let value = draft
                .with_proceed_consent(consent.clone())
                .unwrap()
                .validated_with_identity_records(&result, records, &key)
                .unwrap();
            let preview = value.preview(&repo).unwrap();
            let (counts, items) = diffs(&p, &key, &repo, &preview);
            assert_eq!((counts.unchanged, counts.disappeared), (3, 0));
            assert!(
                items
                    .iter()
                    .all(|item| item.identity().trace_id().0 != "B2")
            );
            assert_eq!(
                preview.snapshot().source().missing_trace_consent(),
                Some(&consent)
            );
            approve(&value, &repo);
        }
        // Reload the provider from the explicit portable file as well as SQLite.
        let provider = provider(&key_path);
        let key = IdentityProtector::preflight(Some(KEY_ID), &provider, None).unwrap();
        let repo = SQLitePersistenceAdapter::open(&root, Some(KEY_ID), &provider).unwrap();
        let current = repo
            .load_current_snapshot(&namespace("501"))
            .unwrap()
            .unwrap();
        let source = current.source();
        assert_eq!(source.missing_trace_consent(), Some(&consent));
        for trace in source.traces() {
            if trace.trace_id.0 == "B2" {
                assert_eq!(trace.presence, TracePresence::Absent);
                assert!(trace.input_sha256.is_none());
            } else {
                assert_eq!(trace.presence, TracePresence::Present { record_count: 1 });
                assert!(trace.input_sha256.is_some());
            }
        }
        key.verify_submission(source).unwrap();
        let wire = serde_json::to_value(source).unwrap();
        assert_eq!(wire["missing_trace_consent"]["decision"], "proceed");
        let restored: SourceSubmission = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(restored.missing_trace_consent(), Some(&consent));
        // Public JSON intentionally drops the private proof. It remains data,
        // never a restored approval/protection capability, even if unmodified.
        assert!(matches!(
            key.verify_submission(&restored),
            Err(ProtectionError::Unattested)
        ));
        assert!(matches!(
            key.protect_submission(&restored),
            Err(ProtectionError::Unattested)
        ));
        let mut tampered = wire;
        tampered["missing_trace_consent"]["recorded_at"] = json!("2026-06-01T00:02:00Z");
        let tampered: SourceSubmission = serde_json::from_value(tampered).unwrap();
        assert!(key.verify_submission(&tampered).is_err());
        let value = candidate(&p, &key, &data, "501", "T3-B2-returns", (2026, 3), &[]).unwrap();
        let preview = value.preview(&repo).unwrap();
        let counts = diffs(&p, &key, &repo, &preview).0;
        assert_eq!(
            (counts.unchanged, counts.reactivated, counts.disappeared),
            (4, 0, 0)
        );
    }

    #[test]
    fn present_empty_b2_is_not_missing_and_only_it_can_annul_the_previous_b2() {
        let (_dir, root, key_path) = portable_archive();
        let provider = provider(&key_path);
        let key = IdentityProtector::preflight(Some(KEY_ID), &provider, None).unwrap();
        let p = profile(model());
        let data = rows("501");
        {
            let repo = SQLitePersistenceAdapter::open(&root, Some(KEY_ID), &provider).unwrap();
            approve(
                &candidate(&p, &key, &data, "501", "T1-all", (2026, 1), &[]).unwrap(),
                &repo,
            );
            let mut empty = data.clone();
            empty.insert("B2".into(), vec![]);
            let (draft, _, _) = prepared(&p, &key, &empty, "wrong-empty-consent", 2, &[]);
            assert_eq!(draft.missing_trace_warning_code(), None);
            assert_eq!(draft.absent_traces().count(), 0);
            assert!(draft.with_proceed_consent(decision(&["B2"], 2)).is_err());
            let (draft, result, records) = prepared(&p, &key, &empty, "T2-B2-empty", 2, &[]);
            let value = draft
                .validated_with_identity_records(&result, records, &key)
                .unwrap();
            let preview = value.preview(&repo).unwrap();
            let (counts, items) = diffs(&p, &key, &repo, &preview);
            assert_eq!(counts.disappeared, 1);
            assert_eq!(
                items
                    .iter()
                    .filter(|item| item.change() == HistoricalChange::Removed)
                    .count(),
                1
            );
            assert!(
                preview
                    .snapshot()
                    .source()
                    .missing_trace_consent()
                    .is_none()
            );
            approve(&value, &repo);
        }
        let repo = SQLitePersistenceAdapter::open(&root, Some(KEY_ID), &provider).unwrap();
        let current = repo
            .load_current_snapshot(&namespace("501"))
            .unwrap()
            .unwrap();
        let trace = current
            .source()
            .traces()
            .iter()
            .find(|t| t.trace_id.0 == "B2")
            .unwrap();
        assert_eq!(trace.presence, TracePresence::Present { record_count: 0 });
        assert_eq!(
            trace.input_sha256,
            Some(Sha256Digest(Sha256::digest(b"").into()))
        );
        assert!(current.source().missing_trace_consent().is_none());
        let value = candidate(&p, &key, &data, "501", "T3-reactivation", (2026, 3), &[]).unwrap();
        let preview = value.preview(&repo).unwrap();
        let counts = diffs(&p, &key, &repo, &preview).0;
        assert_eq!((counts.reactivated, counts.disappeared), (1, 0));
    }

    #[test]
    fn every_omission_needs_consent_and_present_unknown_duplicate_or_empty_acknowledgements_fail() {
        let (_dir, _root, key_path) = portable_archive();
        let provider = provider(&key_path);
        let key = IdentityProtector::preflight(Some(KEY_ID), &provider, None).unwrap();
        let p = profile(model());
        let data = rows("501");
        for ids in [&["A"][..], &["UNKNOWN"][..]] {
            let (draft, _, _) = prepared(&p, &key, &data, "invalid-ack", 1, &["B1", "B2"]);
            assert!(draft.with_proceed_consent(decision(ids, 1)).is_err());
        }
        let (draft, result, records) = prepared(&p, &key, &data, "partial-ack", 1, &["B1", "B2"]);
        assert!(matches!(
            draft
                .with_proceed_consent(decision(&["B2"], 1))
                .unwrap()
                .validated_with_identity_records(&result, records, &key),
            Err(SessionApprovalError::Rejected(
                SessionRejection::MissingProceedConsent
            ))
        ));
        let (draft, result, records) = prepared(&p, &key, &data, "complete-ack", 1, &["B1", "B2"]);
        assert_eq!(
            draft
                .absent_traces()
                .map(|id| id.0.as_str())
                .collect::<Vec<_>>(),
            ["B1", "B2"]
        );
        let value = draft
            .with_proceed_consent(decision(&["B2", "B1"], 1))
            .unwrap()
            .validated_with_identity_records(&result, records, &key)
            .unwrap();
        assert_eq!(
            value.source().missing_trace_consent().unwrap().traces(),
            [TraceId("B1".into()), TraceId("B2".into())]
        );
        assert!(
            MissingTraceProceedConsent::new(
                vec![],
                Utc.with_ymd_and_hms(2026, 3, 1, 0, 1, 0).unwrap()
            )
            .is_err()
        );
        assert!(
            MissingTraceProceedConsent::new(
                vec![TraceId("B2".into()), TraceId("B2".into())],
                Utc.with_ymd_and_hms(2026, 3, 1, 0, 1, 0).unwrap()
            )
            .is_err()
        );
    }

    #[test]
    fn wire_has_only_present_or_absent_and_abort_cannot_deserialize_as_durable_proceed() {
        assert_eq!(
            serde_json::to_value(TracePresence::Present { record_count: 0 }).unwrap(),
            json!({"kind":"present","record_count":0})
        );
        assert_eq!(
            serde_json::to_value(TracePresence::Absent).unwrap(),
            json!({"kind":"absent"})
        );
        for value in [
            json!({"kind":"partial","record_count":1}),
            json!({"kind":"absent","record_count":0}),
        ] {
            assert!(serde_json::from_value::<TracePresence>(value).is_err());
        }
        assert!(
            serde_json::from_value::<MissingTraceProceedConsent>(
                json!({"decision":"abort","traces":["B2"],"recorded_at":"2026-06-01T00:01:00Z"})
            )
            .is_err()
        );
        assert!(serde_json::from_value::<MissingTraceProceedConsent>(json!({"decision":"proceed","traces":["B2"],"recorded_at":"2026-06-01T00:01:00Z","implicit":true})).is_err());
    }
}
