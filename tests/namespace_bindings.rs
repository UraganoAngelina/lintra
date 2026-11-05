//! Real service executions using only synthetic records. No CAD clinical rule
//! or completed snapshot reconciliation is inferred from these component tests.
use lintra::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, sync::Arc};

fn model() -> Value {
    let mut v: Value =
        serde_json::from_str(include_str!("fixtures/application/schema.json")).unwrap();
    v["schema_format_version"] = json!(3);
    v["required_capabilities"] = json!(["local.v1", "persistence.namespace_binding.v1"]);
    v["parameters"] = json!([{"id":"unit","type":"string","allowed_values":["501","502"]}]);
    v["traces"][0]["fields"][0]["constraints"][0]["values"] = json!(["501", "502"]);
    v["persistence_policy"] = json!({
        "namespace_parameters":["unit"], "version_compatibility":{"kind":"same_schema_version"},
        "run_uniqueness":[{"trace":"T1","key":"record_key"}], "cross_run_audit":[],
        "history_identity_invariants":[], "snapshot":null, "temporal_rules":[],
        "storage_protection":"external_secure_configuration_required",
        "namespace_bindings":[{"trace":"T1","field":"F1","parameter":"unit",
            "comparison":"logical_text_exact","error_code":"E_NAMESPACE_VALUE_MISMATCH",
            "message_template":"Organizzazione diversa da quella dell’invio."}]
    });
    v
}
fn raw(v: Value) -> RawFlowSchema {
    let mut r: RawFlowSchema = serde_json::from_value(v).unwrap();
    r.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    r
}
fn service(v: Value) -> ValidationService {
    let mut repo = InMemorySchemaRepository::new();
    repo.insert(raw(v)).unwrap();
    ValidationService::new(Arc::new(SchemaRegistry::new(
        Arc::new(repo),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    )))
}
struct Fixture {
    dir: tempfile::TempDir,
}
impl Fixture {
    fn new(bytes: &[u8]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("input.txt"), bytes).unwrap();
        Self { dir }
    }
    fn request(&self, mode: ValidationMode) -> ValidationRequest {
        let input = TraceValidationInput {
            trace_id: TraceId("T1".into()),
            source: InputSource::Path(self.dir.path().join("input.txt")),
        };
        ValidationRequest {
            schema_key: SchemaKey {
                flow_id: FlowId("SYNTHETIC".into()),
                version: SchemaVersion("v1".into()),
            },
            scope: if mode == ValidationMode::Full {
                ValidationScope::Flow(FlowValidationInput {
                    traces: vec![input],
                })
            } else {
                ValidationScope::Trace(input)
            },
            mode,
            options: ValidationOptions {
                record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                newline_policy: NewlinePolicy::AutoLfOrCrlf,
                max_errors: None,
            },
        }
    }
}
fn values(v: &str) -> BTreeMap<String, JoinParameterValue> {
    [("unit".into(), JoinParameterValue::Text(v.into()))].into()
}
fn run(
    v: Value,
    bytes: &[u8],
    mode: ValidationMode,
    p: BTreeMap<String, JoinParameterValue>,
) -> (ValidationRunOutcome, VecSink) {
    let f = Fixture::new(bytes);
    let mut sink = VecSink::default();
    let out = service(v)
        .validate_with_typed_parameters(
            f.request(mode),
            p,
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    (out, sink)
}
#[test]
fn match_mismatch_and_primary_error_preserve_real_summary_jsonl_and_order() {
    for mode in [ValidationMode::LocalOnly, ValidationMode::Full] {
        let (out, sink) = run(model(), b"501A  \n502A  \n999A  ", mode, values("501"));
        assert_eq!(out.status, RunStatus::CompletedInvalid);
        assert_eq!(
            (
                out.summary.records_processed,
                out.summary.valid_records,
                out.summary.invalid_records,
                out.summary.total_errors
            ),
            (3, 1, 2, 2)
        );
        assert_eq!(
            sink.errors()
                .iter()
                .map(|e| (e.line_number, e.code.as_symbol().unwrap().to_owned()))
                .collect::<Vec<_>>(),
            vec![
                (2, "E_NAMESPACE_VALUE_MISMATCH".into()),
                (3, "E_INVALID_DOMAIN".into())
            ]
        );
        let e = &sink.errors()[0];
        assert_eq!(e.trace_id, TraceId("T1".into()));
        assert_eq!(e.field.as_ref().unwrap().id, FieldId("F1".into()));
        let mut jsonl = JsonlSink::new(Vec::new());
        for e in sink.errors() {
            jsonl.on_error(e).unwrap();
        }
        let bytes = jsonl.finish().unwrap();
        let lines = String::from_utf8(bytes).unwrap();
        assert_eq!(lines.lines().count(), 2);
        assert!(lines.contains("E_NAMESPACE_VALUE_MISMATCH"));
        write_run_summary(Vec::new(), &out.summary).unwrap();
    }
}
#[test]
fn local_without_namespace_is_diagnostic_including_empty_input() {
    for bytes in [b"501A  ".as_slice(), b"".as_slice()] {
        let (out, sink) = run(model(), bytes, ValidationMode::LocalOnly, BTreeMap::new());
        assert_eq!(out.status, RunStatus::CompletedValid);
        assert!(sink.errors().is_empty());
        assert!(out.summary.skipped_rules > 0);
        assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
    }
}
#[test]
fn full_missing_and_supplied_invalid_parameters_fail_before_input_or_callbacks() {
    let f = Fixture::new(b"501A  ");
    fs::remove_file(f.dir.path().join("input.txt")).unwrap();
    for mode in [ValidationMode::LocalOnly, ValidationMode::Full] {
        for p in [
            values("999"),
            [(
                "unit".into(),
                JoinParameterValue::Integer("501".parse().unwrap()),
            )]
            .into(),
            [("unknown".into(), JoinParameterValue::Text("501".into()))].into(),
        ] {
            let mut sink = VecSink::default();
            let mut events = vec![];
            let error = service(model())
                .validate_with_typed_parameters(
                    f.request(mode),
                    p,
                    &mut sink,
                    &mut |e| events.push(e),
                    &CancellationToken::default(),
                )
                .unwrap_err();
            assert!(matches!(
                error,
                ValidationRunError::Flow(FlowValidationError::Inputs(
                    RelatedContextError::InvalidParameter
                ))
            ));
            assert!(events.is_empty() && sink.errors().is_empty() && sink.summaries().is_empty());
        }
    }
    let error = service(model())
        .validate_with_typed_parameters(
            f.request(ValidationMode::Full),
            BTreeMap::new(),
            &mut VecSink::default(),
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
}
#[test]
fn logical_comparison_uses_padding_but_never_numeric_coercion() {
    let mut v = model();
    v["parameters"][0]["allowed_values"] = json!(["1", "001"]);
    v["traces"][0]["fields"][0]["constraints"][0]["values"] = json!(["1"]);
    let (out, _) = run(v.clone(), b"001A  ", ValidationMode::Full, values("1"));
    assert_eq!(out.status, RunStatus::CompletedValid);
    let (out, sink) = run(v, b"001A  ", ValidationMode::Full, values("001"));
    assert_eq!(out.status, RunStatus::CompletedInvalid);
    assert_eq!(
        sink.errors()[0].code.as_symbol().unwrap().to_owned(),
        "E_NAMESPACE_VALUE_MISMATCH"
    );
}
#[test]
fn budget_and_dynamic_cancellation_keep_the_sequential_contract() {
    let f = Fixture::new(b"502A  \n501A  ");
    let s = service(model());
    let mut request = f.request(ValidationMode::Full);
    request.options.max_errors = Some(1);
    let mut sink = VecSink::default();
    let out = s
        .validate_with_typed_parameters(
            request,
            values("501"),
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    assert!(out.summary.truncated);
    assert_eq!(out.summary.records_processed, 1);
    assert_eq!(sink.errors().len(), 1);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
    let cancel = CancellationToken::default();
    let mut progress = |e| {
        if matches!(e, ProgressEvent::TraceStarted { .. }) {
            cancel.cancel();
        }
    };
    let out = s
        .validate_with_typed_parameters(
            f.request(ValidationMode::Full),
            values("501"),
            &mut VecSink::default(),
            &mut progress,
            &cancel,
        )
        .unwrap();
    assert_eq!(out.status, RunStatus::Cancelled);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
}
#[test]
fn binding_contract_rejects_old_formats_bad_references_types_duplicates_and_capabilities() {
    let good = model();
    assert!(SchemaCompiler.compile(raw(good.clone())).is_ok());
    for mutation in 0..11 {
        let mut v = good.clone();
        match mutation {
            0 => v["schema_format_version"] = json!(1),
            1 => v["schema_format_version"] = json!(2),
            2 => v["persistence_policy"]["namespace_bindings"][0]["trace"] = json!("no"),
            3 => v["persistence_policy"]["namespace_bindings"][0]["field"] = json!("no"),
            4 => v["persistence_policy"]["namespace_bindings"][0]["parameter"] = json!("no"),
            5 => v["parameters"][0] = json!({"id":"unit","type":"integer","min":1,"max":999}),
            6 => {
                let b = v["persistence_policy"]["namespace_bindings"][0].clone();
                v["persistence_policy"]["namespace_bindings"]
                    .as_array_mut()
                    .unwrap()
                    .push(b);
            }
            7 => v["required_capabilities"] = json!(["local.v1"]),
            8 => v["persistence_policy"]["namespace_bindings"] = json!([]),
            9 => v["persistence_policy"]["namespace_bindings"][0]["error_code"] = json!("bad"),
            _ => v["persistence_policy"]["namespace_bindings"][0]["message_template"] = json!(""),
        }
        let r = raw(v);
        assert!(
            SchemaCompiler.compile(r.clone()).is_err(),
            "mutation {mutation}"
        );
        assert!(SchemaCompiler.compile_persistence_profile(&r, &[]).is_err());
    }
    for key in ["typo", "comparison"] {
        let mut v = good.clone();
        v["persistence_policy"]["namespace_bindings"][0][key] = json!("unknown");
        assert!(serde_json::from_value::<RawFlowSchema>(v).is_err());
    }
}
#[test]
fn binding_changes_fingerprint_and_cross_version_compatibility_but_run_values_do_not() {
    let a = raw(model());
    let mut b = a.clone();
    b.persistence_policy
        .as_mut()
        .unwrap()
        .namespace_bindings
        .as_mut()
        .unwrap()[0]
        .message_template = "Altro messaggio".into();
    assert_ne!(
        schema_fingerprint(&a).unwrap(),
        schema_fingerprint(&b).unwrap()
    );
    let schema = SchemaCompiler.compile(a.clone()).unwrap();
    let before = schema_fingerprint(&a).unwrap();
    for unit in ["501", "502"] {
        VerifiedRunContext::new(&schema, values(unit), ValidationMode::Full).unwrap();
        assert_eq!(schema_fingerprint(&a).unwrap(), before);
    }
    let mut v = model();
    v["persistence_policy"]["version_compatibility"] =
        json!({"kind":"declared_group","id":"g","schema_versions":["v1","v2"]});
    let mut other = v.clone();
    other["schema_version"] = json!("v2");
    other["persistence_policy"]["namespace_bindings"][0]["parameter"] = json!("other");
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&raw(v), &[raw(other)])
            .is_err()
    );
}

struct Provider;
impl PersistenceKeyProvider for Provider {
    fn resolve(&self, _: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        Ok(Arc::new(PersistenceKey::new(vec![0xaa; 64])?))
    }
}
#[derive(Default)]
struct AuditSink {
    callbacks: usize,
}
impl HistoricalAuditSink for AuditSink {
    fn on_diff(&mut self, _: &HistoricalDiff) -> Result<(), ReportError> {
        self.callbacks += 1;
        Ok(())
    }
    fn on_summary(&mut self, _: &HistoricalAuditSummary) -> Result<(), ReportError> {
        self.callbacks += 1;
        Ok(())
    }
}
fn audit_model() -> Value {
    let mut v = model();
    // Same synthetic event key in different organizational namespaces.
    v["traces"][0]["record_key"]["fields"] = json!(["F2"]);
    v["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!("persistence.audit.v1"));
    v["persistence_policy"]["cross_run_audit"] = json!([{"trace":"T1","key":"record_key"}]);
    v
}
#[test]
fn historical_binding_mismatch_never_produces_a_preview_or_mutates_storage() {
    let f = Fixture::new(b"502A  ");
    let s = service(audit_model());
    let protector = IdentityProtector::preflight(Some("synthetic-key"), &Provider, None).unwrap();
    let repo = InMemoryPersistenceRepository::with_protection_audit(protector.audit().unwrap());
    let namespace =
        PersistenceNamespace::new(vec![("unit".into(), NamespaceValue::String("501".into()))])
            .unwrap();
    let mut sink = VecSink::default();
    let mut audit = AuditSink::default();
    let outcome = HistoricalValidationService::new(&s, &repo, &protector)
        .validate(
            HistoricalValidationRequest {
                validation: f.request(ValidationMode::Full),
                parameters: [("unit".into(), NamespaceValue::String("501".into()))].into(),
                submission_id: "synthetic-mismatch".into(),
            },
            &mut sink,
            &mut audit,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    let HistoricalValidationOutcome::ValidationOnly(out) = outcome else {
        panic!("namespace mismatch must not create an approval token")
    };
    assert_eq!(out.status, RunStatus::CompletedInvalid);
    assert_eq!(audit.callbacks, 0);
    assert_eq!(repo.current_generation(&namespace).unwrap(), None);
    assert_eq!(
        sink.errors()[0].code.as_symbol().unwrap(),
        "E_NAMESPACE_VALUE_MISMATCH"
    );
}
#[test]
fn historical_uses_the_same_namespace_for_validation_review_and_commit() {
    let f = Fixture::new(b"501A  ");
    let s = service(audit_model());
    let protector = IdentityProtector::preflight(Some("synthetic-key"), &Provider, None).unwrap();
    let repo = InMemoryPersistenceRepository::with_protection_audit(protector.audit().unwrap());
    let hist = HistoricalValidationService::new(&s, &repo, &protector);
    let namespace = |unit: &str| {
        PersistenceNamespace::new(vec![("unit".into(), NamespaceValue::String(unit.into()))])
            .unwrap()
    };
    for unit in ["501", "502"] {
        fs::write(f.dir.path().join("input.txt"), format!("{unit}A  ")).unwrap();
        let outcome = hist
            .validate(
                HistoricalValidationRequest {
                    validation: f.request(ValidationMode::Full),
                    parameters: [("unit".into(), NamespaceValue::String(unit.into()))].into(),
                    submission_id: format!("synthetic-{unit}"),
                },
                &mut VecSink::default(),
                &mut AuditSink::default(),
                &mut |_| {},
                &CancellationToken::default(),
            )
            .unwrap();
        let HistoricalValidationOutcome::Preview(mut preview) = outcome else {
            panic!("valid bound run should create a review")
        };
        preview
            .approve_commit(&CancellationToken::default(), |_, _, _| Ok(()))
            .unwrap();
        assert!(repo.current_generation(&namespace(unit)).unwrap().is_some());
    }
    assert!(
        repo.current_generation(&namespace("501"))
            .unwrap()
            .is_some()
    );
    // Wrong type, missing or invalid explicit parameter must fail before IO.
    fs::remove_file(f.dir.path().join("input.txt")).unwrap();
    for parameters in [
        BTreeMap::new(),
        [("unit".into(), NamespaceValue::Integer(501))].into(),
        [("unit".into(), NamespaceValue::String("999".into()))].into(),
    ] {
        let mut sink = VecSink::default();
        let result = hist.validate(
            HistoricalValidationRequest {
                validation: f.request(ValidationMode::Full),
                parameters,
                submission_id: "invalid".into(),
            },
            &mut sink,
            &mut AuditSink::default(),
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
}
#[test]
fn context_cannot_be_reused_with_another_compiled_artifact_having_the_same_key() {
    let a = SchemaCompiler.compile(raw(model())).unwrap();
    let b = SchemaCompiler.compile(raw(model())).unwrap();
    let c = VerifiedRunContext::new(&a, values("501"), ValidationMode::Full).unwrap();
    assert!(
        RecordValidator::new(&b, &TraceId("T1".into()))
            .unwrap()
            .with_run_context(&c)
            .is_err()
    );
}

#[test]
fn missing_parent_trace_empty_parent_and_real_parent_keep_existing_join_semantics() {
    let v = parent_model();
    let s = service(v);
    let f = Fixture::new(b"501A  ");
    let child_path = f.dir.path().join("child.txt");
    fs::write(&child_path, b"501A  ").unwrap();
    for parent in [None, Some(b"".as_slice()), Some(b"501A  ".as_slice())] {
        let mut request = f.request(ValidationMode::Full);
        let ValidationScope::Flow(ref mut input) = request.scope else {
            unreachable!()
        };
        if let Some(bytes) = parent {
            fs::write(f.dir.path().join("input.txt"), bytes).unwrap();
        } else {
            input.traces.clear();
        }
        input.traces.push(TraceValidationInput {
            trace_id: TraceId("T2".into()),
            source: InputSource::Path(child_path.clone()),
        });
        let mut sink = VecSink::default();
        let out = s
            .validate_with_typed_parameters(
                request,
                values("501"),
                &mut sink,
                &mut |_| {},
                &CancellationToken::default(),
            )
            .unwrap();
        assert!(
            sink.errors()
                .iter()
                .all(|e| e.code.as_symbol().unwrap() != "E_NAMESPACE_VALUE_MISMATCH")
        );
        match parent {
            None => {
                assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
                assert_eq!(
                    out.summary.full_validation.as_ref().unwrap().controls[0].unknown,
                    1
                );
            }
            Some([]) => {
                assert_eq!(out.status, RunStatus::CompletedInvalid);
                assert_eq!(sink.errors()[0].code, ValidationCode::RelatedRecordMissing);
            }
            _ => assert_eq!(out.status, RunStatus::CompletedValid),
        }
    }
}

#[test]
fn cad_component_compiles_normative_keys_namespace_and_quarter_without_claiming_full_runtime() {
    let c: Value = serde_json::from_str(include_str!(
        "../schemas/cad/components/2024-06.namespace.1/component.json"
    ))
    .unwrap();
    let mut v = model();
    v["required_capabilities"] = c["required_capabilities"].clone();
    v["parameters"] = c["parameters"].clone();
    v["persistence_policy"] = c["persistence_policy"].clone();
    let trace_template = v["traces"][0].clone();
    v["traces"] = json!([]);
    let expected = [
        ("A", vec!["1.1", "1.23"]),
        ("B1", vec!["2.1", "2.36"]),
        ("B2", vec!["3.1", "3.5"]),
        ("C", vec!["4.1", "4.2", "4.4", "4.5", "4.11"]),
        ("D", vec!["5.1", "5.2", "5.3", "5.4", "5.7"]),
    ];
    for (id, keys) in &expected {
        assert_eq!(c["record_keys"][*id]["fields"], json!(keys));
        let mut t = trace_template.clone();
        t["id"] = json!(id);
        t["identity"] = json!([]);
        t["fields"] = json!([]);
        let mut start = 1;
        for field in keys {
            let mut f = trace_template["fields"][0].clone();
            f["id"] = json!(field);
            f["physical"]["start"] = json!(start);
            f["constraints"] = json!([]);
            if *field == "1.1" {
                f = c["namespace_field"].clone();
            }
            t["fields"].as_array_mut().unwrap().push(f);
            start += 3;
        }
        t["record_length"] = json!(start - 1);
        t["record_key"] = c["record_keys"][*id].clone();
        v["traces"].as_array_mut().unwrap().push(t);
    }
    let r = raw(v);
    let profile = SchemaCompiler.compile_persistence_profile(&r, &[]).unwrap();
    assert_eq!(profile.namespace_bindings()[0].trace_index, 0);
    assert_eq!(profile.namespace_bindings()[0].field_index, 0);
    let policy = profile.policy().unwrap();
    assert_eq!(policy.namespace_parameters, vec![0]);
    let snapshot = policy.snapshot.as_ref().unwrap();
    assert_eq!(snapshot.scope_parameters, vec![0]);
    assert_eq!(
        (snapshot.year_parameter, snapshot.quarter_parameter),
        (1, 2)
    );
    for (i, (_, keys)) in expected.iter().enumerate() {
        assert_eq!(profile.record_key_fields(i).unwrap().len(), keys.len());
    }
    assert!(
        SchemaCompiler.compile(r).is_err(),
        "snapshot execution remains gated until its implementation task"
    );
}

#[test]
fn optional_missing_namespace_field_is_not_a_positive_namespace_verification() {
    let mut v = model();
    v["traces"][0]["fields"][0]["presence"]["base"] = json!("optional");
    let (out, sink) = run(v, b"   A  ", ValidationMode::LocalOnly, values("501"));
    assert!(sink.errors().is_empty());
    assert!(out.summary.skipped_rules > 0);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
}
#[test]
fn compatibility_checks_bound_field_value_semantics_even_outside_the_pk() {
    let mut v = model();
    v["traces"][0]["record_key"]["fields"] = json!(["F2"]);
    v["persistence_policy"]["version_compatibility"] =
        json!({"kind":"declared_group","id":"g","schema_versions":["v1","v2"]});
    let mut other = v.clone();
    other["schema_version"] = json!("v2");
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&raw(v.clone()), &[raw(other.clone())])
            .is_ok()
    );
    other["traces"][0]["fields"][0]["physical"]["alignment"] = json!({"kind":"exact"});
    assert!(
        SchemaCompiler
            .compile_persistence_profile(&raw(v), &[raw(other)])
            .is_err()
    );
    let mut invalid = model();
    invalid["persistence_policy"]["namespace_bindings"] = Value::Null;
    assert!(serde_json::from_value::<RawFlowSchema>(invalid).is_err());
}

#[test]
fn a_started_record_executor_cannot_switch_organizational_context() {
    let schema = SchemaCompiler.compile(raw(model())).unwrap();
    let a = VerifiedRunContext::new(&schema, values("501"), ValidationMode::LocalOnly).unwrap();
    let b = VerifiedRunContext::new(&schema, values("502"), ValidationMode::LocalOnly).unwrap();
    let mut validator = RecordValidator::new(&schema, &TraceId("T1".into()))
        .unwrap()
        .with_run_context(&a)
        .unwrap();
    let mut summary = ValidationSummaryBuilder::new(
        schema.key().clone(),
        ScopeSummary::Trace {
            trace_id: TraceId("T1".into()),
        },
    );
    validator
        .validate_local(
            RawRecord {
                bytes: b"501A  ",
                line_number: 1,
            },
            &mut VecSink::default(),
            &mut summary,
        )
        .unwrap();
    assert!(validator.with_run_context(&b).is_err());
}

fn parent_model() -> Value {
    let mut v = model();
    let mut child = v["traces"][0].clone();
    child["id"] = json!("T2");
    let unknown = json!({"policy":"unknown","diagnostic":{"code":"SYNTHETIC_UNAVAILABLE","message_template":"Parent unavailable"}});
    let missing = json!({"policy":"violation","error_code":"E_RELATED_RECORD_MISSING","diagnostic":{"code":"SYNTHETIC_MISSING","message_template":"Parent missing"}});
    child["joins"] = json!([{"id":"child_parent","target_trace":"T1","source_fields":["F2"],"target_fields":["F2"],"cardinality":"zero_or_one","scope":"current_submission","namespace_parameters":["unit"],"on_missing_record":missing,"on_missing_trace":unknown}]);
    child["rules"] = json!([{"id":"PARENT","when":{"const":true},"assertions":[{"kind":"field_compare","left":{"ref":{"field":"F2"}},"operator":"==","right":{"ref":{"trace":"T1","field":"F2","join":"child_parent"}},"value_type":"bytes"}],"error_code":"E_CROSS_FIELD","message_template":"Synthetic {rule_id}: {condition}"}]);
    v["traces"].as_array_mut().unwrap().push(child);
    v["required_capabilities"] = json!([
        "local.v1",
        "expressions.v1",
        "joins.v1",
        "persistence.namespace_binding.v1"
    ]);
    v
}

#[test]
fn wrong_namespace_parent_cannot_satisfy_a_child_join() {
    let f = Fixture::new(b"502A  ");
    let child = f.dir.path().join("child.txt");
    fs::write(&child, b"501A  ").unwrap();
    let mut request = f.request(ValidationMode::Full);
    let ValidationScope::Flow(ref mut input) = request.scope else {
        unreachable!()
    };
    input.traces.push(TraceValidationInput {
        trace_id: TraceId("T2".into()),
        source: InputSource::Path(child),
    });
    let mut sink = VecSink::default();
    let out = service(parent_model())
        .validate_with_typed_parameters(
            request,
            values("501"),
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    assert_eq!(out.status, RunStatus::CompletedInvalid);
    let control = &out.summary.full_validation.as_ref().unwrap().controls[0];
    assert_eq!(
        control.satisfied, 0,
        "an out-of-namespace parent must not supply trusted join evidence"
    );
    assert_eq!(control.unknown, 1);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
}

#[test]
fn validation_and_storage_share_namespace_structural_validity() {
    for text in ["", "   ", "X\0Y"] {
        let mut v = model();
        v["parameters"][0]["allowed_values"] = json!([text]);
        let schema = SchemaCompiler.compile(raw(v)).unwrap();
        assert!(
            PersistenceNamespace::new(vec![("unit".into(), NamespaceValue::String(text.into()))])
                .is_err()
        );
        for mode in [ValidationMode::LocalOnly, ValidationMode::Full] {
            assert!(
                matches!(
                    VerifiedRunContext::new(&schema, values(text), mode),
                    Err(RelatedContextError::InvalidParameter)
                ),
                "mode {mode:?} accepted a namespace rejected by storage"
            );
        }
    }
}
