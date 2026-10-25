//! Ratified SIOC structural obligations, synthetic rows, real production service.
//! This is an explicit reference-only component, not complete SIOC validation.
use lintra::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
#[path = "support/full_schema.rs"]
mod synthetic_schema;
const PROFILE: &str =
    include_str!("../schemas/sioc/2026-04-20.structural-minimal.2/flow_schema.json");
const ORIGINAL: &str =
    include_str!("../schemas/sioc/2026-04-20.structural-minimal.1/flow_schema.json");
const COMPONENT: &str = include_str!("golden/cross-trace/sioc-referential/v1/schema.json");
const LEDGER: &str =
    include_str!("../schemas/sioc/2026-04-20.structural-minimal.2/scope-ledger.json");
const IDS: [&str; 5] = ["1", "2", "3", "4.1", "4.2"];
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/cross-trace/sioc-referential/v1")
}
struct FixedClock;
impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH
    }
}
struct Attempt {
    outcome: Result<ValidationRunOutcome, ValidationRunError>,
    errors: Vec<u8>,
    events: Vec<ProgressEvent>,
}
fn execute(schema: &Value, directory: &Path, omit: Option<&str>) -> Attempt {
    let bundle = tempfile::tempdir().unwrap();
    let folder = bundle
        .path()
        .join(schema["flow_id"].as_str().unwrap())
        .join(schema["schema_version"].as_str().unwrap());
    fs::create_dir_all(&folder).unwrap();
    fs::write(
        folder.join("flow.json"),
        serde_json::to_vec(schema).unwrap(),
    )
    .unwrap();
    let registry = SchemaRegistry::new(
        Arc::new(FileSystemSchemaRepository::new(bundle.path().to_path_buf()).unwrap()),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    );
    let service = ValidationService::with_clock(Arc::new(registry), Arc::new(FixedClock));
    let request = ValidationRequest {
        schema_key: SchemaKey {
            flow_id: FlowId(schema["flow_id"].as_str().unwrap().into()),
            version: SchemaVersion(schema["schema_version"].as_str().unwrap().into()),
        },
        scope: ValidationScope::Flow(FlowValidationInput {
            traces: IDS
                .into_iter()
                .rev()
                .filter(|id| Some(*id) != omit)
                .map(|id| TraceValidationInput {
                    trace_id: TraceId(id.into()),
                    source: InputSource::Path(directory.join(format!("trace-{id}.dat"))),
                })
                .collect(),
        }),
        mode: ValidationMode::Full,
        options: ValidationOptions {
            record_length_policy: RecordLengthPolicy::StrictSkipRecord,
            newline_policy: NewlinePolicy::AutoLfOrCrlf,
            max_errors: None,
        },
    };
    let mut sink = JsonlSink::new(Vec::new());
    let mut events = vec![];
    let outcome = service.validate_with_parameters(
        request,
        BTreeMap::from([("organisation_namespace".into(), "1".into())]),
        &mut sink,
        &mut |event| events.push(event),
        &CancellationToken::default(),
    );
    Attempt {
        outcome,
        errors: sink.finish().unwrap(),
        events,
    }
}
fn assert_bytes(expected: &[u8], actual: &[u8], label: &str) {
    if expected != actual {
        let position = expected
            .iter()
            .zip(actual)
            .position(|(a, b)| a != b)
            .unwrap_or(expected.len().min(actual.len()));
        let start = position.saturating_sub(40);
        panic!(
            "{label}: mismatch at byte {position}; expected {:?}, actual {:?}",
            String::from_utf8_lossy(&expected[start..expected.len().min(position + 200)]),
            String::from_utf8_lossy(&actual[start..actual.len().min(position + 200)])
        );
    }
}
#[test]
fn approved_cross_trace_rules_match_independent_normative_goldens_byte_for_byte() {
    let schema: Value = serde_json::from_str(COMPONENT).unwrap();
    let mut discovered = fs::read_dir(root())
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| entry.file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    discovered.sort();
    assert_eq!(discovered, ["cardinality", "missing", "valid"]);
    for case in ["valid", "missing", "cardinality"] {
        let directory = root().join(case);
        let attempt = execute(&schema, &directory, None);
        let mut summary = attempt.outcome.unwrap().summary;
        // Only elapsed wall time is nondeterministic. Schema/input hashes,
        // timestamp (actual injected clock), version and every outcome stay real.
        summary.elapsed = Duration::ZERO;
        assert_bytes(
            &fs::read(directory.join("expected.errors.jsonl")).unwrap(),
            &attempt.errors,
            &format!("{case} errors"),
        );
        assert_bytes(
            &fs::read(directory.join("expected.summary.json")).unwrap(),
            &write_run_summary(Vec::new(), &summary).unwrap(),
            &format!("{case} summary"),
        );
        assert!(attempt.events.contains(&ProgressEvent::PhaseStarted(
            ValidationPhase::CrossRecordValidation
        )));
        assert!(matches!(
            attempt.events.last(),
            Some(ProgressEvent::RunCompleted)
        ));
    }
}
#[test]
fn derived_profile_changes_only_ratified_rules_and_revoking_policies() {
    let original: Value = serde_json::from_str(ORIGINAL).unwrap();
    let derived: Value = serde_json::from_str(PROFILE).unwrap();
    let ledger: Value = serde_json::from_str(LEDGER).unwrap();
    let digest = |bytes: &[u8]| Sha256Digest(Sha256::digest(bytes).into()).to_string();
    assert_eq!(
        digest(ORIGINAL.as_bytes()),
        "2592c46889a2c06fc9dd5da5b9220f2f1c2a3ac42985f56bd24e476058128c31"
    );
    assert_eq!(
        digest(PROFILE.as_bytes()),
        ledger["artifacts"]["flow_schema.json"]
    );
    assert_eq!(
        digest(COMPONENT.as_bytes()),
        ledger["artifacts"]["golden_subset_schema.json"]
    );
    let component: Value = serde_json::from_str(COMPONENT).unwrap();
    assert_eq!(
        component["source"]["parent_profile_sha256"],
        digest(PROFILE.as_bytes())
    );
    let obligations = [
        ("2", "trace2_to_trace1"),
        ("3", "trace3_to_trace2"),
        ("3", "trace3_to_trace1"),
        ("4.2", "trace42_to_trace41"),
    ];
    assert_eq!(ledger["rules"].as_array().unwrap().len(), obligations.len());
    for (index, (source, join)) in obligations.into_iter().enumerate() {
        let trace = derived["traces"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["id"] == source)
            .unwrap();
        let id = format!("REL-{join}");
        let rule = trace["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        assert_eq!(rule["when"], json!({"const":true}));
        assert_eq!(
            rule["assertions"],
            json!([{"kind":"record_exists","join":join}])
        );
        assert_eq!(rule["error_code"], "E_MISSING_PARENT_RECORD");
        assert_eq!(
            ledger["rules"][index]["source_join_snapshot"]["required"],
            true
        );
        assert_eq!(ledger["rules"][index]["rule_id"], id);
        assert_eq!(ledger["rules"][index]["join"], join);
        assert_eq!(ledger["rules"][index]["applicability"], rule["when"]);
    }
    let mut restored = derived.clone();
    restored["schema_version"] = original["schema_version"].clone();
    restored["source"] = original["source"].clone();
    for (trace, source) in restored["traces"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .zip(original["traces"].as_array().unwrap())
    {
        let original_rules = source["rules"].as_array().unwrap();
        let expected_added = obligations
            .iter()
            .filter(|(id, _)| trace["id"] == *id)
            .count();
        assert_eq!(
            trace["rules"].as_array().unwrap().len(),
            original_rules.len() + expected_added
        );
        assert_eq!(
            &trace["rules"].as_array().unwrap()[..original_rules.len()],
            original_rules
        );
        trace["rules"] = source["rules"].clone();
        for join in trace["joins"].as_array_mut().unwrap() {
            if join["id"] == "revoking_act" {
                let old = source["joins"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|old| old["id"] == "revoking_act")
                    .unwrap();
                join["on_missing_record"] = old["on_missing_record"].clone();
                join["on_missing_trace"] = old["on_missing_trace"].clone();
            }
        }
    }
    assert_eq!(restored, original); // all fields/layout/persistence/capabilities untouched
    assert_eq!(
        derived["traces"][0]["joins"],
        original["traces"][0]["joins"]
    ); // approved optional parent exactly preserved
    let mut authored: RawFlowSchema = serde_json::from_value(derived).unwrap();
    authored.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let error = SchemaCompiler.compile(authored).unwrap_err();
    assert!(
        error
            .issues
            .iter()
            .any(|issue| issue.code == SchemaIssueCode::UnsupportedCapability
                && issue.message.contains("PersistenceAuditV1"))
    );
}
#[test]
fn full_missing_indispensable_trace_fails_before_reading_other_sources() {
    let schema: Value = serde_json::from_str(COMPONENT).unwrap();
    let attempt = execute(&schema, &root().join("directory-does-not-exist"), Some("1"));
    assert!(
        matches!(attempt.outcome,Err(ValidationRunError::InvalidRequest(InvalidRequestError::MissingRequiredTrace(trace))) if trace==TraceId("1".into()))
    );
    assert!(attempt.errors.is_empty());
    assert!(attempt.events.is_empty());
}
fn temporary_case(case: &str) -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    for id in IDS {
        fs::copy(
            root().join(case).join(format!("trace-{id}.dat")),
            temp.path().join(format!("trace-{id}.dat")),
        )
        .unwrap();
    }
    temp
}
#[test]
fn partial_revoking_universe_is_unknown_without_a_false_missing_act_violation() {
    let temp = temporary_case("valid");
    let mut records = fs::read(root().join("missing/trace-4.2.dat")).unwrap();
    records.extend_from_slice(b"BAD\n");
    fs::write(temp.path().join("trace-4.2.dat"), records).unwrap();
    let attempt = execute(&serde_json::from_str(COMPONENT).unwrap(), temp.path(), None);
    let summary = attempt.outcome.unwrap().summary;
    let errors = String::from_utf8(attempt.errors).unwrap();
    assert_eq!(errors.lines().count(), 1);
    assert_eq!(
        serde_json::from_str::<Value>(errors.lines().next().unwrap()).unwrap()["code"],
        "E_RECORD_LENGTH"
    );
    assert_eq!(summary.skipped_rules, 2);
    assert_eq!(summary.completeness(), ValidationCompleteness::Partial);
    let control = summary
        .full_validation
        .as_ref()
        .unwrap()
        .controls
        .iter()
        .find(|c| c.rule_id == "R-4.30-REFERENCES-4.22")
        .unwrap();
    assert_eq!(
        (
            control.evaluations,
            control.unknown,
            control.violations,
            control.skipped
        ),
        (1, 1, 0, 1)
    );
    assert_eq!(
        control.causes[0].result,
        CrossCauseKind::Unknown(CrossUncertaintyReason::IncompleteRelatedContext)
    );
    assert_eq!(control.causes[0].join_id.as_deref(), Some("revoking_act"));
}
fn regression_schema() -> Value {
    let mut schema: Value = serde_json::from_str(COMPONENT).unwrap();
    schema["flow_id"] = json!("SYNTHETIC-POLICY-REGRESSION");
    schema["source"]["scope"] = json!("synthetic-runtime-regression-only");
    for trace in schema["traces"].as_array_mut().unwrap() {
        trace["rules"] = json!([]);
    }
    schema
}
#[test]
fn a_join_declaration_never_synthesizes_a_normative_control() {
    // Test-only variant, never serialized as an approved SIOC profile.
    let attempt = execute(&regression_schema(), &root().join("missing"), None);
    let summary = attempt.outcome.unwrap().summary;
    assert!(attempt.errors.is_empty());
    assert_eq!(summary.run_status, RunStatus::CompletedValid);
    assert_eq!(summary.completeness(), ValidationCompleteness::Complete);
    assert!(summary.full_validation.unwrap().controls.is_empty());
}
#[test]
fn optional_anagraphic_policy_remains_not_applicable_without_inverse_obligation() {
    // The synthetic assertion only probes the unchanged policy; it is not added
    // to the normative SIOC component and creates no new anagraphic obligation.
    let mut schema = regression_schema();
    schema["traces"][0]["rules"] = json!([{"id":"SYNTHETIC-OPTIONAL-POLICY","when":{"const":true},"assertions":[{"kind":"record_exists","join":"trace1_to_trace2_episode"}],"error_code":"E_RELATED_RECORD_MISSING","message_template":"Synthetic policy regression"}]);
    let temp = temporary_case("valid");
    for id in ["2", "3", "4.1", "4.2"] {
        fs::write(temp.path().join(format!("trace-{id}.dat")), b"").unwrap();
    }
    let attempt = execute(&schema, temp.path(), None);
    let summary = attempt.outcome.unwrap().summary;
    assert!(attempt.errors.is_empty());
    assert_eq!(summary.completeness(), ValidationCompleteness::Complete);
    let control = &summary.full_validation.as_ref().unwrap().controls[0];
    assert_eq!(
        (
            control.satisfied,
            control.not_applicable,
            control.unknown,
            control.violations
        ),
        (0, 1, 0, 0)
    );
    assert_eq!(
        control.causes[0].join_id.as_deref(),
        Some("trace1_to_trace2_episode")
    );
    assert_eq!(
        control.causes[0].message_template.as_deref(),
        Some("Anagrafica presente; dati del ricovero correlato assenti")
    );
}
#[test]
fn corrected_unavailable_trace_policy_remains_unknown_in_standalone_executor() {
    let derived: Value = serde_json::from_str(PROFILE).unwrap();
    let policy = derived["traces"][4]["joins"]
        .as_array()
        .unwrap()
        .iter()
        .find(|join| join["id"] == "revoking_act")
        .unwrap()["on_missing_trace"]
        .clone();
    let mut raw = synthetic_schema::raw();
    raw["traces"][0]["joins"][0]["on_missing_trace"] = policy;
    raw["traces"][0]["rules"][0]["assertions"] = json!([{"kind":"record_exists","join":"S_T"}]);
    let mut raw: RawFlowSchema = serde_json::from_value(raw).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let schema = SchemaCompiler.compile(raw).unwrap();
    let mut store = InMemoryContextStore::new();
    store
        .insert(
            project_record(
                &RecordView::new(b"01AAOK"),
                &schema.traces()[0],
                RecordIdentity::default(),
                1,
            )
            .unwrap(),
        )
        .unwrap();
    struct Observation {
        reason: Option<UnknownReason>,
    }
    impl CrossRecordEvaluationObserver for Observation {
        fn on_evaluation(
            &mut self,
            _: &TraceId,
            _: u64,
            _: &str,
            outcome: &CrossRuleOutcome<'_>,
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            let CrossRuleOutcome::Unknown {
                reason, diagnostic, ..
            } = outcome
            else {
                panic!("unavailable universe is not determinate");
            };
            self.reason = Some(*reason);
            assert_eq!(
                diagnostic.unwrap().code.as_ref(),
                "RELATED_TRACE_UNAVAILABLE"
            );
            Ok(())
        }
    }
    let mut observation = Observation { reason: None };
    let mut sink = VecSink::default();
    let parameters = BTreeMap::new();
    let counts = validate_cross_record_rules(
        &schema,
        &store,
        CrossRecordInputs {
            available_traces: &[TraceId("S".into())],
            parameters: &parameters,
        },
        &mut sink,
        &mut observation,
    )
    .unwrap();
    assert_eq!(counts.unknown, 1);
    assert_eq!(
        observation.reason,
        Some(UnknownReason::TraceInputUnavailable)
    );
    assert!(sink.errors().is_empty());
}

#[test]
fn operative_component_preserves_selected_definitions_and_lists_every_exclusion() {
    let full: Value = serde_json::from_str(PROFILE).unwrap();
    let component: Value = serde_json::from_str(COMPONENT).unwrap();
    let exclusions: Value =
        serde_json::from_slice(&fs::read(root().join("scope-ledger.json")).unwrap()).unwrap();
    assert_eq!(component["source"]["full_normative_compliance"], false);
    assert_eq!(component["source"]["scope"], "referential-golden-only");
    assert_eq!(exclusions["full_profile_persistence_unchanged"], true);
    assert!(
        full["required_capabilities"]
            .as_array()
            .unwrap()
            .contains(&json!("persistence.audit.v1"))
    );
    assert!(
        !component["required_capabilities"]
            .as_array()
            .unwrap()
            .contains(&json!("persistence.audit.v1"))
    );
    for (selected, source) in component["traces"]
        .as_array()
        .unwrap()
        .iter()
        .zip(full["traces"].as_array().unwrap())
    {
        for property in [
            "id",
            "record_length",
            "record_key",
            "joins",
            "identity",
            "rules",
        ] {
            assert_eq!(selected[property], source[property]);
        }
        for field in selected["fields"].as_array().unwrap() {
            if field["id"].as_str().unwrap().starts_with("EXCLUDED-") {
                assert_eq!(field["regulatory_metadata"]["test_only_opaque_span"], true);
                assert!(field["constraints"].as_array().unwrap().is_empty());
                continue;
            }
            let original = source["fields"]
                .as_array()
                .unwrap()
                .iter()
                .find(|original| original["id"] == field["id"])
                .unwrap();
            let mut restored = field.clone();
            restored["constraints"] = original["constraints"].clone();
            assert_eq!(&restored, original);
            for (index, constraint) in original["constraints"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                let omitted = exclusions["excluded_constraints"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|item| {
                        item["trace"] == source["id"]
                            && item["field"] == field["id"]
                            && item["index"] == index
                    });
                assert_eq!(
                    field["constraints"]
                        .as_array()
                        .unwrap()
                        .contains(constraint),
                    !omitted
                );
            }
        }
        for original in source["fields"].as_array().unwrap() {
            let retained = selected["fields"]
                .as_array()
                .unwrap()
                .iter()
                .any(|field| field["id"] == original["id"]);
            let ledger = if retained {
                "selected_fields"
            } else {
                "excluded_fields"
            };
            assert!(
                exclusions[ledger]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|item| item["trace"] == source["id"] && item["field"] == original["id"])
            );
        }
    }
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas/sioc/2026-04-20.structural-minimal.2");
    let ledger: Value = serde_json::from_str(LEDGER).unwrap();
    for trace in full["traces"].as_array().unwrap() {
        let name = format!("trace-{}.json", trace["id"].as_str().unwrap());
        let bytes = fs::read(directory.join(&name)).unwrap();
        assert_eq!(
            Sha256Digest(Sha256::digest(&bytes).into()).to_string(),
            ledger["artifacts"][&name]
        );
        assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), *trace);
    }
}
