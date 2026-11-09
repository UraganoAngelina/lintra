//! CAD12 integration groundwork. Real input bytes, Full evidence and protected
//! approval share one compiled component; this is explicitly not the 83-field
//! normative CAD profile or a public executable snapshot entry point.
use super::*;
use chrono::{TimeZone, Utc};
use std::sync::Arc;

const KEY_ID: &str = "public-cad12-test-key";
struct PublicKey;
impl PersistenceKeyProvider for PublicKey {
    fn resolve(&self, id: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        if id != KEY_ID {
            return Err(ProtectionError::MissingKey);
        }
        // Public, fixed fixture bytes; never an operational credential.
        Ok(Arc::new(PersistenceKey::new(vec![0xaa; 32])?))
    }
}
fn namespace() -> PersistenceNamespace {
    PersistenceNamespace::new(vec![(
        "sending_ulss".into(),
        NamespaceValue::String("501".into()),
    )])
    .unwrap()
}
fn declaration() -> RawFlowSchema {
    let mut v = model();
    let progressive: Value = serde_json::from_str(include_str!(
        "../../schemas/cad/components/2024-06.progressives.1/component.json"
    ))
    .unwrap();
    v["persistence_policy"]["history_identity_invariants"] =
        progressive["persistence_policy"]["history_identity_invariants"].clone();
    v["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!("persistence.audit.v1"));
    // The source D→C obligation is explicit. Independent progressive fields
    // remain in the PKs and are deliberately absent from this join.
    v["traces"][4]["joins"].as_array_mut().unwrap().push(json!({
        "id":"cad12_provision_access", "target_trace":"C",
        "source_fields":["5.1","5.2","5.3","5.4"],
        "target_fields":["4.1","4.2","4.4","4.5"],
        "cardinality":"zero_or_many", "scope":"current_submission",
        "namespace_parameters":["sending_ulss"],
        "on_missing_record":{"policy":"violation","error_code":"E_CAD_MISSING_ACCESS_FOR_PROVISION",
            "diagnostic":{"code":"E_CAD_MISSING_ACCESS_FOR_PROVISION","message_template":"Prestazione priva di accesso correlato."}},
        "on_missing_trace":{"policy":"unknown","diagnostic":{"code":"W_CAD_INACTIVITY_INPUT_UNAVAILABLE",
            "message_template":"Input accessi non disponibile."}}
    }));
    v["traces"][4]["rules"].as_array_mut().unwrap().push(json!({
        "id":"CAD12-PROVISION-ACCESS", "when":{"const":true},
        "assertions":[{"kind":"set_exists","join":"cad12_provision_access","predicate":{"const":true}}],
        "error_code":"E_CAD_MISSING_ACCESS_FOR_PROVISION",
        "message_template":"Prestazione priva di accesso correlato."
    }));
    v["schema_version"] = json!("cad12-integration-component-test-v1");
    raw(v)
}
struct Projector<'a> {
    profile: &'a CompiledPersistenceProfile,
    protector: &'a IdentityProtector,
    records: Vec<ProjectedHistoricalRecord>,
}
impl crate::engine::flow_execution::FlowRecordObserver for Projector<'_> {
    fn on_record(
        &mut self,
        trace: &CompiledRecordSchema,
        record: &RawRecord<'_>,
        _: &JoinKey,
        _: &[crate::engine::record_validator::OperandValidity],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Called on the SAME borrowed row used by pass one, never a second scan.
        self.records.push(ProjectedHistoricalRecord::project(
            self.profile,
            &namespace(),
            trace.id(),
            *record,
            self.protector,
        )?);
        Ok(())
    }
}
struct Executed {
    outcome: ValidationRunOutcome,
    records: Vec<ProjectedHistoricalRecord>,
    errors: VecSink,
}
fn execute_real(
    profile: &CompiledPersistenceProfile,
    schema: &CompiledFlowSchema,
    protector: &IdentityProtector,
    rows: &[Option<Vec<Vec<u8>>>],
    quarter: u32,
) -> Executed {
    let dir = tempfile::tempdir_in("target").unwrap();
    let mut paths = vec![];
    for (i, rows) in rows.iter().enumerate() {
        if let Some(rows) = rows {
            let path = dir.path().join(format!("trace-{i}.dat"));
            let mut bytes = vec![];
            for row in rows {
                bytes.extend(row);
                bytes.push(b'\n');
            }
            fs::write(&path, bytes).unwrap();
            paths.push((i, path));
        }
    }
    let inputs = paths
        .iter()
        .map(|(i, path)| FlowTraceInput {
            trace_id: schema.traces()[*i].id(),
            path,
        })
        .collect::<Vec<_>>();
    let mut summary = ValidationSummaryBuilder::new(
        schema.key().clone(),
        ScopeSummary::Flow {
            trace_ids: inputs.iter().map(|input| input.trace_id.clone()).collect(),
        },
    );
    let audits = InputAuditSession::new();
    let parameters = parameters(2026, quarter);
    let mut projector = Projector {
        profile,
        protector,
        records: vec![],
    };
    let mut errors = VecSink::default();
    let artifacts = FlowValidator::new(schema, NewlinePolicy::AutoLfOrCrlf)
        .unwrap()
        .validate_with_record_observer(
            &inputs,
            &parameters,
            FlowExecutionPorts {
                sink: &mut errors,
                summary: &mut summary,
                audits: &audits,
                control: &mut FlowControl {
                    is_cancelled: &|| false,
                    on_event: &mut |_| {},
                },
                reference_observers: None,
                cross_observer: None,
            },
            Some(&mut projector),
        )
        .unwrap();
    // Reuse the production evidence conversion and builder. No public summary
    // fixture, assigned verdict, input digest or fake completeness is used.
    let full = crate::application::full::into_summary(artifacts, &parameters).unwrap();
    let metadata = audits
        .finish(RunMetadata {
            validation_mode: ValidationMode::Full,
            validator_version: "cad12-integration-component-test".into(),
            schema_hash: profile.schema_hash().to_string(),
            reference_data_versions: BTreeMap::new(),
            started_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap().into(),
            inputs: vec![],
        })
        .unwrap();
    let summary = summary.finish_full(metadata, full, None).unwrap();
    summary.validate().unwrap();
    Executed {
        outcome: ValidationRunOutcome {
            status: summary.run_status,
            summary,
        },
        records: projector.records,
        errors,
    }
}
fn approval<'a>(
    profile: &CompiledPersistenceProfile,
    protector: &'a IdentityProtector,
    execution: Executed,
    quarter: i64,
    proceed: bool,
) -> Result<InvariantValidatedApproval<'a>, SessionApprovalError> {
    let s = &execution.outcome.summary;
    let traces = profile
        .trace_ids()
        .iter()
        .map(|id| {
            let input = s.inputs.iter().find(|input| &input.trace_id == id);
            let (presence, input_sha256) = match input {
                Some(input) => {
                    let InputAuditCompletion::Complete { sha256 } = &input.completion else {
                        panic!("input must have reached EOF")
                    };
                    let trace = s
                        .full_validation
                        .as_ref()
                        .unwrap()
                        .traces
                        .iter()
                        .find(|t| &t.trace_id == id)
                        .unwrap();
                    (
                        TracePresence::Present {
                            record_count: trace.records_processed,
                        },
                        Some(sha256.clone()),
                    )
                }
                None => (TracePresence::Absent, None),
            };
            SubmissionTrace {
                trace_id: id.clone(),
                presence,
                input_sha256,
            }
        })
        .collect();
    let parts = SourceSubmissionParts {
        namespace: namespace(),
        submission_id: format!("cad12-T{quarter}"),
        schema_key: s.schema_key.clone(),
        schema_hash: profile.schema_hash().clone(),
        period: None,
        created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        validator_version: s.validator_version.clone(),
        validation_mode: s.validation_mode,
        validation_status: s.run_status,
        reference_data_versions: vec![],
        traces,
        fingerprint: protector.draft_fingerprint(profile.schema_hash()).unwrap(),
        state: SessionState::Draft,
    };
    let params = BTreeMap::from([
        ("sending_ulss".into(), NamespaceValue::String("501".into())),
        ("year".into(), NamespaceValue::Integer(2026)),
        ("quarter".into(), NamespaceValue::Integer(quarter)),
    ]);
    let mut draft = ApprovalDraft::new(profile, params, parts, None)?;
    if proceed {
        let absent = draft.absent_traces().cloned().collect();
        draft = draft.with_proceed_consent(MissingTraceProceedConsent::new(
            absent,
            Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        )?)?;
    }
    draft.validated_with_identity_records(&execution.outcome, execution.records, protector)
}
fn current(date: &str) -> Vec<Option<Vec<Vec<u8>>>> {
    let mut r = rows("01012026");
    // Raw marker is in explicitly unvalidated component filler. Its purpose is
    // to catch accidental row persistence, never to represent a clinical value.
    set(
        &mut r[1].as_mut().unwrap()[0],
        40,
        "PUBLIC-CAD12-RAW-MARKER",
    );
    r[2] = Some(vec![pause("10012026", "20012026", 1)]);
    r[3] = Some(vec![access(date, 7)]);
    r[4] = Some(vec![provision(date, 999)]);
    r
}
#[test]
fn cad12_actual_full_projection_approval_t1_t2_t3_and_missing_pause_restart() {
    let raw = declaration();
    let profile = SchemaCompiler
        .compile_persistence_profile(&raw, &[])
        .unwrap();
    let schema = SchemaCompiler
        .compile_scoped(raw.clone(), profile.policy().cloned())
        .unwrap();
    assert!(SchemaCompiler.compile(raw.clone()).is_err());
    assert!(SchemaCompiler.compile_historical_audit(raw, &[]).is_err());
    let protector = IdentityProtector::preflight(Some(KEY_ID), &PublicKey, None).unwrap();
    let dir = tempfile::tempdir_in("target").unwrap();
    let root = dir.path().join("archive");
    drop(SQLitePersistenceAdapter::create(&root, Some(KEY_ID), &PublicKey).unwrap());
    for (q, date) in [(1, "01032026"), (2, "01062026"), (3, "01092026")] {
        let mut rows = current(date);
        if q == 2 {
            rows[2] = None;
            let no_consent = execute_real(&profile, &schema, &protector, &rows, q);
            assert!(approval(&profile, &protector, no_consent, q.into(), false).is_err());
        }
        let execution = execute_real(&profile, &schema, &protector, &rows, q);
        assert!(execution.errors.errors().is_empty());
        assert_eq!(execution.outcome.status, RunStatus::CompletedValid);
        assert_eq!(
            execution.outcome.summary.completeness(),
            if q == 2 {
                ValidationCompleteness::Partial
            } else {
                ValidationCompleteness::Complete
            }
        );
        let value = approval(&profile, &protector, execution, q.into(), q == 2).unwrap();
        let repo = SQLitePersistenceAdapter::open(&root, Some(KEY_ID), &PublicKey).unwrap();
        let mut preview = value.preview(&repo).unwrap();
        let audit = HistoricalAuditService::for_identity_invariants(&profile, &protector, &repo);
        let result = audit
            .prepare(preview.snapshot(), &CancellationToken::default())
            .unwrap()
            .write_diffs(&CancellationToken::default(), |_| Ok(()))
            .unwrap();
        if q > 1 {
            assert_eq!(result.counts.disappeared, 0);
            assert_eq!(result.counts.reactivated, 0);
        }
        preview
            .approve_commit(&CancellationToken::default(), |_| Ok(()))
            .unwrap();
        drop(preview);
        drop(repo);
        let reopened = SQLitePersistenceAdapter::open(&root, Some(KEY_ID), &PublicKey).unwrap();
        assert_eq!(
            reopened.current_generation(&namespace()).unwrap(),
            Some(SnapshotRevision::new(q.into()))
        );
        let snapshot = reopened
            .load_snapshot(&namespace(), SnapshotRevision::new(q.into()))
            .unwrap()
            .unwrap();
        protector.verify_submission(snapshot.source()).unwrap();
        assert_eq!(snapshot.source().state(), SessionState::ApprovedImmutable);
        let b2 = snapshot
            .source()
            .traces()
            .iter()
            .find(|t| t.trace_id.0 == "B2")
            .unwrap();
        assert_eq!(
            b2.presence,
            if q == 2 {
                TracePresence::Absent
            } else {
                TracePresence::Present { record_count: 1 }
            }
        );
        assert_eq!(snapshot.source().missing_trace_consent().is_some(), q == 2);
        for entry in fs::read_dir(&root).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                let bytes = fs::read(path).unwrap();
                assert!(
                    !bytes
                        .windows(b"PUBLIC-CAD12-RAW-MARKER".len())
                        .any(|window| window == b"PUBLIC-CAD12-RAW-MARKER")
                );
            }
        }
    }
}

#[test]
fn cad12_provision_requires_access_on_four_fields_but_never_on_progressive() {
    let r = declaration();
    let p = SchemaCompiler.compile_persistence_profile(&r, &[]).unwrap();
    let s = SchemaCompiler
        .compile_scoped(r, p.policy().cloned())
        .unwrap();
    let key = IdentityProtector::preflight(Some(KEY_ID), &PublicKey, None).unwrap();
    let good = execute_real(&p, &s, &key, &current("01032026"), 1);
    assert_eq!(good.outcome.status, RunStatus::CompletedValid);
    assert!(good.errors.errors().is_empty());
    // C progressive7 and D progressive999 are independent and both survived.
    let mut rows = current("01032026");
    rows[3] = Some(vec![]);
    let missing = execute_real(&p, &s, &key, &rows, 1);
    assert_eq!(missing.outcome.status, RunStatus::CompletedInvalid);
    assert!(
        missing
            .errors
            .errors()
            .iter()
            .any(|e| e.code == "E_CAD_MISSING_ACCESS_FOR_PROVISION".parse().unwrap())
    );
    assert!(approval(&p, &key, missing, 1, false).is_err());
}

#[test]
fn cad12_real_jsonl_is_deterministic_and_exposes_the_documented_order_conflict() {
    let r = declaration();
    let p = SchemaCompiler.compile_persistence_profile(&r, &[]).unwrap();
    let s = SchemaCompiler
        .compile_scoped(r, p.policy().cloned())
        .unwrap();
    let key = IdentityProtector::preflight(Some(KEY_ID), &PublicKey, None).unwrap();
    let mut rows = rows("01102025");
    rows[1].as_mut().unwrap()[0][14] = b'9'; // Local B1 domain violation.
    let first = execute_real(&p, &s, &key, &rows, 1);
    let second = execute_real(&p, &s, &key, &rows, 1);
    assert_eq!(first.outcome.status, RunStatus::CompletedInvalid);
    assert_eq!(first.errors.errors().first().unwrap().trace_id.0, "B1");
    assert_eq!(first.errors.errors().last().unwrap().trace_id.0, "A");
    let encode = |value: &Executed| {
        let mut sink = JsonlSink::new(Vec::new());
        for error in value.errors.errors() {
            sink.on_error(error).unwrap();
        }
        sink.on_summary(&value.outcome.summary).unwrap();
        sink.finish().unwrap()
    };
    assert_eq!(encode(&first), encode(&second));
    // Local B1 precedes cross A: reproducibility is not a proof of a global
    // trace-first order. This preserves current output pending CNF ratification.
    assert!(approval(&p, &key, first, 1, false).is_err());
}

#[test]
fn cad12_history_obligation_is_rejected_instead_of_replaced_with_current_input() {
    let mut raw = declaration();
    raw.required_capabilities
        .push(RawFlowSchemaRequiredCapabilitiesItem::HistoryExistenceV1);
    let mut join = raw.traces[0].joins[0].clone();
    join.id = "cad12_initial_evaluation_history".into();
    join.scope = RawJoinSpecScope::HistoryIncludingCurrent;
    raw.traces[0].joins.push(join);
    // Metadata may describe history, but an unused history join and ordinary
    // execution still fail closed. No current-only plan is substituted.
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&raw, &[])
            .is_ok()
    );
    assert!(
        SchemaCompiler
            .compile_historical_snapshot(raw.clone(), &[])
            .is_err()
    );
    assert!(SchemaCompiler.compile(raw).is_err());
}
