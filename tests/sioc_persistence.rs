//! Real Full/audit application execution on synthetic SIOC rows. The fixture
//! combines the already scoped referential component with the source audit
//! declarations; it makes no claim of complete normative SIOC validation.
use lintra::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
const IDS: [&str; 5] = ["1", "2", "3", "4.1", "4.2"];
const COMPONENT: &str = include_str!("golden/cross-trace/sioc-referential/v1/schema.json");
const PROFILE: &str =
    include_str!("../schemas/sioc/2026-04-20.structural-minimal.2/flow_schema.json");
fn model() -> Value {
    let mut value: Value = serde_json::from_str(COMPONENT).unwrap();
    let source: Value = serde_json::from_str(PROFILE).unwrap();
    value["schema_version"] = json!("2026-04-20.audit-synthetic.1");
    value["source"]["scope"] = json!("synthetic-referential-and-audit-only");
    value["persistence_policy"] = source["persistence_policy"].clone();
    value["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!("persistence.audit.v1"));
    value
}
fn raw(value: Value) -> RawFlowSchema {
    let mut value: RawFlowSchema = serde_json::from_value(value).unwrap();
    value.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    value
}
struct Provider(u8);
impl PersistenceKeyProvider for Provider {
    fn resolve(&self, id: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        if id != "synthetic-key" {
            return Err(ProtectionError::MissingKey);
        }
        Ok(Arc::new(PersistenceKey::new(vec![self.0; 64])?))
    }
}
fn protector() -> IdentityProtector {
    IdentityProtector::preflight(Some("synthetic-key"), &Provider(0xaa), None).unwrap()
}
struct FixedClock;
impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH
    }
}
struct CountingRepository {
    value: Value,
    reads: Arc<AtomicUsize>,
}
impl SchemaRepository for CountingRepository {
    fn load_raw(&self, _: &SchemaKey) -> Result<RawFlowSchema, SchemaLoadError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        Ok(raw(self.value.clone()))
    }
}
struct Fixture {
    inputs: tempfile::TempDir,
    registry: Arc<SchemaRegistry>,
    reads: Arc<AtomicUsize>,
}
impl Fixture {
    fn new() -> Self {
        let inputs = tempfile::tempdir().unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/golden/cross-trace/sioc-referential/v1/valid");
        for id in IDS {
            fs::copy(
                root.join(format!("trace-{id}.dat")),
                inputs.path().join(format!("trace-{id}.dat")),
            )
            .unwrap();
        }
        Self::with_model(inputs, model())
    }
    fn with_model(inputs: tempfile::TempDir, value: Value) -> Self {
        let reads = Arc::new(AtomicUsize::new(0));
        let registry = Arc::new(SchemaRegistry::new(
            Arc::new(CountingRepository {
                value,
                reads: reads.clone(),
            }),
            SchemaCompiler,
            RawResolvedEncoding::Utf8,
        ));
        Self {
            inputs,
            registry,
            reads,
        }
    }
    fn path(&self, id: &str) -> PathBuf {
        self.inputs.path().join(format!("trace-{id}.dat"))
    }
    fn service(&self) -> ValidationService {
        ValidationService::with_clock(self.registry.clone(), Arc::new(FixedClock))
    }
    fn request(&self, id: &str, namespace: i64) -> HistoricalValidationRequest {
        HistoricalValidationRequest {
            validation: ValidationRequest {
                schema_key: SchemaKey {
                    flow_id: FlowId("SIOC".into()),
                    version: SchemaVersion("2026-04-20.audit-synthetic.1".into()),
                },
                scope: ValidationScope::Flow(FlowValidationInput {
                    traces: IDS
                        .into_iter()
                        .rev()
                        .map(|id| TraceValidationInput {
                            trace_id: TraceId(id.into()),
                            source: InputSource::Path(self.path(id)),
                        })
                        .collect(),
                }),
                mode: ValidationMode::Full,
                options: ValidationOptions {
                    record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                    newline_policy: NewlinePolicy::AutoLfOrCrlf,
                    max_errors: None,
                },
            },
            parameters: BTreeMap::from([(
                "organisation_namespace".into(),
                NamespaceValue::Integer(namespace),
            )]),
            submission_id: id.into(),
        }
    }
}
#[derive(Default)]
struct AuditSink {
    diffs: Vec<HistoricalDiff>,
    summaries: usize,
    fail: bool,
}
impl HistoricalAuditSink for AuditSink {
    fn on_diff(&mut self, diff: &HistoricalDiff) -> Result<(), ReportError> {
        if self.fail {
            return Err(ReportError::Write(io::Error::other(
                "synthetic audit failure",
            )));
        }
        self.diffs.push(diff.clone());
        Ok(())
    }
    fn on_summary(&mut self, _: &HistoricalAuditSummary) -> Result<(), ReportError> {
        self.summaries += 1;
        Ok(())
    }
}
fn preview<'r>(
    fixture: &Fixture,
    repository: &'r dyn PersistentContextRepository,
    protector: &IdentityProtector,
    id: &str,
    org: i64,
) -> Box<HistoricalValidationPreview<'r>> {
    let service = fixture.service();
    let mut sink = VecSink::default();
    let mut audit = AuditSink::default();
    let result = HistoricalValidationService::new(&service, repository, protector)
        .validate(
            fixture.request(id, org),
            &mut sink,
            &mut audit,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    assert_eq!(sink.summaries().len(), 1);
    assert_eq!(audit.summaries, 1);
    match result {
        HistoricalValidationOutcome::Preview(p) => p,
        HistoricalValidationOutcome::ValidationOnly(out) => panic!(
            "unexpected validation-only: {:?} {:?}",
            out.status,
            sink.errors()
        ),
    }
}
fn commit(preview: &mut HistoricalValidationPreview<'_>) -> CommitReceipt {
    preview
        .approve_commit(&CancellationToken::default(), |_, _, _| Ok(()))
        .unwrap()
}

#[test]
fn source_pk_tuples_are_exact_and_compiler_does_not_enable_history_implicitly() {
    let value = model();
    let source: Value = serde_json::from_str(PROFILE).unwrap();
    let expected = [
        json!(["1.1", "1.17", "1.18"]),
        json!(["2.1", "2.49", "2.50"]),
        json!(["3.1", "3.2", "3.35", "3.36"]),
        json!(["4.1", "4.7", "4.8", "4.19.1"]),
        json!(["4.20", "4.22"]),
    ];
    for (i, pk) in expected.iter().enumerate() {
        assert_eq!(
            value["traces"][i]["record_key"],
            source["traces"][i]["record_key"]
        );
        assert_eq!(&value["traces"][i]["record_key"]["fields"], pk);
    }
    assert!(SchemaCompiler.compile(raw(value.clone())).is_err());
    let compiled = SchemaCompiler
        .compile_historical_audit(raw(value), &[])
        .unwrap();
    assert_eq!(compiled.record_projections().len(), 5);
    for definition in compiled.record_projections() {
        assert_eq!(
            &Sha256Digest(Sha256::digest(definition.canonical_descriptor_bytes().unwrap()).into()),
            definition.definition_hash()
        );
    }
    // Full source retains its outstanding capabilities/field checks. This task
    // never strips those checks just to obtain a supposedly compliant run.
    assert!(
        SchemaCompiler
            .compile_historical_audit(raw(source), &[])
            .is_err()
    );
}

#[test]
fn five_trace_pk_tokens_survive_sqlite_restart_and_cross_run_is_audit_only() {
    let fixture = Fixture::new();
    let protector = protector();
    let root = tempfile::tempdir_in(Path::new(env!("CARGO_MANIFEST_DIR")).join("target")).unwrap();
    let store = root.path().join("archive");
    let repository =
        SQLitePersistenceAdapter::create(&store, Some("synthetic-key"), &Provider(0xaa)).unwrap();
    let mut first = preview(&fixture, &repository, &protector, "submission-1", 1);
    assert_eq!(first.audit().counts.inserted, 7); // 1+1+1+2+2 synthetic rows
    assert_eq!(first.snapshot().identities().len(), 7);
    assert_eq!(
        repository
            .current_generation(first.snapshot().namespace())
            .unwrap(),
        None
    );
    assert!(
        first
            .snapshot()
            .source()
            .period()
            .unwrap()
            .starts_with("lintra-audit-session-v1:")
    );
    for row in first.snapshot().identities() {
        protector.verify_stored_identity(row).unwrap();
    }
    let namespace = first.snapshot().namespace().clone();
    let tokens = first.snapshot().identities().to_vec();
    assert_eq!(commit(&mut first).revision().value(), 1);
    drop(first);
    drop(repository);
    let repository =
        SQLitePersistenceAdapter::open(&store, Some("synthetic-key"), &Provider(0xaa)).unwrap();
    for token in &tokens {
        assert_eq!(
            repository
                .lookup_identity(&namespace, token.identity())
                .unwrap()
                .as_ref(),
            Some(token)
        );
    }
    let mut second = preview(&fixture, &repository, &protector, "submission-2", 1);
    assert_eq!(second.audit().counts.unchanged, 7);
    assert_eq!(second.outcome().summary.total_errors, 0);
    assert_eq!(commit(&mut second).revision().value(), 2);
    assert_eq!(fixture.reads.load(Ordering::SeqCst), 2); // exactly one schema load per actual execution
    let stored = fs::read(store.join("lintra.db")).unwrap();
    for id in IDS {
        let bytes = fs::read(fixture.path(id)).unwrap();
        let row = bytes.strip_suffix(b"\n").unwrap_or(&bytes);
        assert!(!stored.windows(row.len()).any(|w| w == row));
    }
    assert!(!stored.windows(64).any(|w| w == [0xaa; 64]));
}

#[test]
fn duplicates_in_each_source_trace_are_normative_invalid_with_no_candidate() {
    for id in IDS {
        let fixture = Fixture::new();
        let protector = protector();
        let repository =
            InMemoryPersistenceRepository::with_protection_audit(protector.audit().unwrap());
        let mut bytes = fs::read(fixture.path(id)).unwrap();
        let row = bytes.clone();
        if !bytes.ends_with(b"\n") {
            bytes.push(b'\n');
        }
        bytes.extend(row);
        fs::write(fixture.path(id), bytes).unwrap();
        let service = fixture.service();
        let mut sink = VecSink::default();
        let mut audit = AuditSink::default();
        let result = HistoricalValidationService::new(&service, &repository, &protector)
            .validate(
                fixture.request("duplicate", 1),
                &mut sink,
                &mut audit,
                &mut |_| {},
                &CancellationToken::default(),
            )
            .unwrap();
        let HistoricalValidationOutcome::ValidationOnly(outcome) = result else {
            panic!("duplicate created a candidate");
        };
        assert_eq!(outcome.status, RunStatus::CompletedInvalid);
        assert!(
            sink.errors()
                .iter()
                .any(|e| e.code == ValidationCode::DuplicateRecordKey
                    && e.trace_id == TraceId(id.into()))
        );
        assert!(audit.diffs.is_empty());
        assert_eq!(audit.summaries, 0);
    }
}

#[test]
fn namespace_changes_produce_separate_protected_universes() {
    let fixture = Fixture::new();
    let protector = protector();
    let repository =
        InMemoryPersistenceRepository::with_protection_audit(protector.audit().unwrap());
    let mut a = preview(&fixture, &repository, &protector, "same-id", 1);
    commit(&mut a);
    let mut b = preview(&fixture, &repository, &protector, "same-id", 2);
    assert_eq!(b.audit().counts.inserted, 7);
    assert_ne!(
        a.snapshot().identities()[0].identity(),
        b.snapshot().identities()[0].identity()
    );
    assert_eq!(commit(&mut b).revision().value(), 1);
}

#[test]
fn non_key_physical_byte_change_is_updated_without_normative_inter_run_error() {
    let fixture = Fixture::new();
    let protector = protector();
    let repository =
        InMemoryPersistenceRepository::with_protection_audit(protector.audit().unwrap());
    let mut first = preview(&fixture, &repository, &protector, "first", 1);
    commit(&mut first);
    let mut bytes = fs::read(fixture.path("1")).unwrap();
    bytes[100] = b'X';
    fs::write(fixture.path("1"), bytes).unwrap();
    let second = preview(&fixture, &repository, &protector, "changed", 1);
    assert_eq!(second.audit().counts.updated, 1);
    assert_eq!(second.audit().counts.unchanged, 6);
    assert_eq!(second.outcome().summary.total_errors, 0);
}

#[test]
fn report_failures_and_cancel_never_publish_history_or_a_completed_success() {
    let fixture = Fixture::new();
    let protector = protector();
    let repository =
        InMemoryPersistenceRepository::with_protection_audit(protector.audit().unwrap());
    let mut prepared = preview(&fixture, &repository, &protector, "report-failed", 1);
    assert!(matches!(
        prepared.approve_commit(&CancellationToken::default(), |_, _, _| Err(
            ReportError::Write(io::Error::other("synthetic publication failure"))
        )),
        Err(SessionApprovalError::Report(_))
    ));
    assert_eq!(
        repository
            .current_generation(prepared.snapshot().namespace())
            .unwrap(),
        None
    );
    prepared.cancel().unwrap();
    assert!(matches!(
        prepared.approve_commit(&CancellationToken::default(), |_, _, _| Ok(())),
        Err(SessionApprovalError::Cancelled)
    ));
    let service = fixture.service();
    let mut sink = VecSink::default();
    let mut audit = AuditSink {
        fail: true,
        ..AuditSink::default()
    };
    let mut events = Vec::new();
    let result = HistoricalValidationService::new(&service, &repository, &protector).validate(
        fixture.request("audit-failed", 1),
        &mut sink,
        &mut audit,
        &mut |e| events.push(e),
        &CancellationToken::default(),
    );
    assert!(matches!(
        result,
        Err(HistoricalValidationError::Audit(
            HistoricalAuditError::Report(_)
        ))
    ));
    assert!(sink.summaries().is_empty());
    assert!(!events.contains(&ProgressEvent::RunCompleted));
}

#[test]
fn key_mismatch_and_bad_namespace_fail_before_input_access_even_in_empty_namespace() {
    let fixture = Fixture::new();
    let protector = protector();
    let repository =
        InMemoryPersistenceRepository::with_protection_audit(protector.audit().unwrap());
    for id in IDS {
        fs::remove_file(fixture.path(id)).unwrap();
    }
    let wrong = IdentityProtector::preflight(Some("synthetic-key"), &Provider(0xbb), None).unwrap();
    let service = fixture.service();
    let mut sink = VecSink::default();
    let mut audit = AuditSink::default();
    let result = HistoricalValidationService::new(&service, &repository, &wrong).validate(
        fixture.request("wrong-key", 1),
        &mut sink,
        &mut audit,
        &mut |_| {},
        &CancellationToken::default(),
    );
    assert!(matches!(
        result,
        Err(HistoricalValidationError::Protection(
            ProtectionError::KeyMismatch
        ))
    ));
    let result = HistoricalValidationService::new(&service, &repository, &protector).validate(
        fixture.request("wrong-org", 0),
        &mut sink,
        &mut audit,
        &mut |_| {},
        &CancellationToken::default(),
    );
    assert!(matches!(
        result,
        Err(HistoricalValidationError::Approval(
            SessionApprovalError::InvalidBinding
        ))
    ));
    assert!(sink.summaries().is_empty());
}

#[test]
fn stale_preview_conflicts_and_exact_approved_replay_is_idempotent() {
    let fixture = Fixture::new();
    let protector = protector();
    let repository =
        InMemoryPersistenceRepository::with_protection_audit(protector.audit().unwrap());
    let mut first = preview(&fixture, &repository, &protector, "first", 1);
    let mut stale = preview(&fixture, &repository, &protector, "second", 1);
    let receipt = commit(&mut first);
    assert!(matches!(
        stale.approve_commit(&CancellationToken::default(), |_, _, _| Ok(())),
        Err(SessionApprovalError::Persistence(
            PersistenceError::GenerationConflict { .. }
        ))
    ));
    let mut replay = preview(&fixture, &repository, &protector, "first", 1);
    assert_eq!(replay.state(), Some(SessionState::ApprovedImmutable));
    assert_eq!(commit(&mut replay), receipt);
    let mut changed = fs::read(fixture.path("1")).unwrap();
    changed[100] = b'Y';
    fs::write(fixture.path("1"), changed).unwrap();
    let service = fixture.service();
    let result = HistoricalValidationService::new(&service, &repository, &protector).validate(
        fixture.request("first", 1),
        &mut VecSink::default(),
        &mut AuditSink::default(),
        &mut |_| {},
        &CancellationToken::default(),
    );
    assert!(matches!(
        result,
        Err(HistoricalValidationError::Approval(
            SessionApprovalError::Rejected(SessionRejection::ApprovedSessionImmutable)
        ))
    ));
}

#[test]
fn cancelled_real_execution_and_missing_input_never_produce_an_approval() {
    let fixture = Fixture::new();
    let protector = protector();
    let repository =
        InMemoryPersistenceRepository::with_protection_audit(protector.audit().unwrap());
    let service = fixture.service();
    let cancellation = CancellationToken::default();
    let mut sink = VecSink::default();
    let mut audit = AuditSink::default();
    let result = HistoricalValidationService::new(&service, &repository, &protector)
        .validate(
            fixture.request("cancelled", 1),
            &mut sink,
            &mut audit,
            &mut |e| {
                if matches!(e, ProgressEvent::TraceStarted { .. }) {
                    cancellation.cancel();
                }
            },
            &cancellation,
        )
        .unwrap();
    let HistoricalValidationOutcome::ValidationOnly(outcome) = result else {
        panic!("cancelled run created preview");
    };
    assert_eq!(outcome.status, RunStatus::Cancelled);
    assert!(audit.diffs.is_empty());
    fs::remove_file(fixture.path("1")).unwrap();
    let result = HistoricalValidationService::new(&service, &repository, &protector).validate(
        fixture.request("missing", 1),
        &mut VecSink::default(),
        &mut audit,
        &mut |_| {},
        &CancellationToken::default(),
    );
    assert!(matches!(
        result,
        Err(HistoricalValidationError::Validation(_))
    ));
    assert_eq!(audit.summaries, 0);
}

#[test]
fn optional_absent_file_preserves_baseline_and_present_empty_is_distinct() {
    let mut value = model();
    // Test-only absence policy: remove cross controls/joins, preserving every
    // physical field, source PK and audit declaration. No normative edit.
    for trace in value["traces"].as_array_mut().unwrap() {
        trace["joins"] = json!([]);
        trace["rules"] = json!([]);
    }
    let original = Fixture::new();
    let fixture = Fixture::with_model(original.inputs, value);
    let protector = protector();
    let repository =
        InMemoryPersistenceRepository::with_protection_audit(protector.audit().unwrap());
    let mut first = preview(&fixture, &repository, &protector, "first", 1);
    commit(&mut first);
    let service = fixture.service();
    let mut request = fixture.request("absent", 1);
    let ValidationScope::Flow(flow) = &mut request.validation.scope else {
        unreachable!()
    };
    flow.traces.retain(|t| t.trace_id.0 != "1");
    let result = HistoricalValidationService::new(&service, &repository, &protector)
        .validate(
            request,
            &mut VecSink::default(),
            &mut AuditSink::default(),
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    let HistoricalValidationOutcome::Preview(mut absent) = result else {
        panic!("optional absence prevents audit");
    };
    assert_eq!(absent.audit().counts.disappeared, 0);
    assert_eq!(
        absent
            .snapshot()
            .source()
            .traces()
            .iter()
            .find(|t| t.trace_id.0 == "1")
            .unwrap()
            .presence,
        TracePresence::Absent
    );
    assert!(absent.snapshot().source().missing_trace_consent().is_none());
    commit(&mut absent);
    fs::write(fixture.path("1"), b"").unwrap();
    let empty = preview(&fixture, &repository, &protector, "empty", 1);
    assert_eq!(empty.audit().counts.disappeared, 1);
    assert_eq!(
        empty
            .audit()
            .metadata
            .traces
            .iter()
            .find(|t| t.trace_id.0 == "1")
            .unwrap()
            .revision,
        Some(SnapshotRevision::new(1))
    );
}

#[test]
fn projection_definition_is_deterministic_and_unsupported_encodings_and_policies_fail_closed() {
    let value = model();
    let first = SchemaCompiler
        .compile_historical_audit(raw(value.clone()), &[])
        .unwrap();
    let mut reordered = value.clone();
    reordered["persistence_policy"]["cross_run_audit"]
        .as_array_mut()
        .unwrap()
        .reverse();
    let second = SchemaCompiler
        .compile_historical_audit(raw(reordered), &[])
        .unwrap();
    assert_eq!(
        first
            .record_projections()
            .iter()
            .map(|v| v.definition_hash())
            .collect::<Vec<_>>(),
        second
            .record_projections()
            .iter()
            .map(|v| v.definition_hash())
            .collect::<Vec<_>>()
    );
    let mut encoded = raw(value.clone());
    encoded.encoding.resolved = Some(RawResolvedEncoding::Windows1252);
    let Err(error) = SchemaCompiler.compile_historical_audit(encoded, &[]) else {
        panic!("unsupported encoding was executed");
    };
    assert!(
        error
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::UnsupportedEncoding)
    );
    let mut disabled = value.clone();
    disabled["persistence_policy"] = Value::Null;
    assert!(
        SchemaCompiler
            .compile_historical_audit(raw(disabled), &[])
            .is_err()
    );
    let mut unsupported = value;
    unsupported["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!("persistence.snapshot.v1"));
    assert!(
        SchemaCompiler
            .compile_historical_audit(raw(unsupported), &[])
            .is_err()
    );
}
