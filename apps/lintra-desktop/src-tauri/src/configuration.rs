//! Native catalog/selection only; schema compilation stays in the existing library.
use crate::ipc::{self, ErrorCode};
use lintra::*;
use serde_json::{Map, Value, json};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

const PAGE_SIZE: usize = 64;

pub(crate) enum ConfigurationRequest {
    Catalog {
        offset: usize,
    },
    Select {
        schema_ref: String,
    },
    Descriptors {
        configuration_ref: String,
        offset: usize,
    },
    CreatePlan {
        configuration_ref: String,
        mode: ValidationMode,
        trace_ref: Option<String>,
    },
    ChooseInput {
        plan_ref: String,
        trace_ref: String,
    },
    SetParameter {
        plan_ref: String,
        parameter_ref: String,
        value: JoinParameterValue,
    },
    Prepare {
        plan_ref: String,
    },
}

fn fields(payload: &Map<String, Value>, names: &[&str]) -> Result<(), ErrorCode> {
    if payload.len() == names.len() + 1 && names.iter().all(|name| payload.contains_key(*name)) {
        Ok(())
    } else {
        Err(ErrorCode::InvalidMessage)
    }
}

fn handle(payload: &Map<String, Value>, key: &str) -> Result<String, ErrorCode> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| {
            !text.is_empty()
                && text.len() <= 128
                && text
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
        })
        .map(str::to_owned)
        .ok_or(ErrorCode::InvalidMessage)
}

fn offset(payload: &Map<String, Value>) -> Result<usize, ErrorCode> {
    let text = payload
        .get("offset")
        .and_then(Value::as_str)
        .ok_or(ErrorCode::InvalidMessage)?;
    if text.is_empty()
        || text.len() > 20
        || !text.bytes().all(|b| b.is_ascii_digit())
        || (text.len() > 1 && text.starts_with('0'))
    {
        return Err(ErrorCode::InvalidMessage);
    }
    text.parse().map_err(|_| ErrorCode::InvalidMessage)
}

impl ConfigurationRequest {
    /// Decode exact purpose-specific DTOs before catalog acquisition or other I/O.
    pub(crate) fn decode(operation: &str, payload: &Map<String, Value>) -> Result<Self, ErrorCode> {
        ipc::version(payload)?;
        match operation {
            "plan_parameter_set" => {
                fields(payload, &["plan_ref", "parameter_ref", "value"])?;
                let value = payload["value"]
                    .as_object()
                    .ok_or(ErrorCode::InvalidMessage)?;
                ipc::version(value)?;
                fields(value, &["kind", "text"])?;
                let text = value["text"].as_str().ok_or(ErrorCode::InvalidMessage)?;
                let value = match value["kind"].as_str() {
                    Some("text") => JoinParameterValue::Text(text.to_owned()),
                    Some("integer") => JoinParameterValue::Integer(
                        text.parse().map_err(|_| ErrorCode::InvalidMessage)?,
                    ),
                    _ => return Err(ErrorCode::InvalidMessage),
                };
                Ok(Self::SetParameter {
                    plan_ref: handle(payload, "plan_ref")?,
                    parameter_ref: handle(payload, "parameter_ref")?,
                    value,
                })
            }
            "plan_prepare" => {
                fields(payload, &["plan_ref"])?;
                Ok(Self::Prepare {
                    plan_ref: handle(payload, "plan_ref")?,
                })
            }
            "plan_create" => {
                fields(payload, &["configuration_ref", "mode", "trace_ref"])?;
                let mode = match payload["mode"].as_str() {
                    Some("local_only") => ValidationMode::LocalOnly,
                    Some("full") => ValidationMode::Full,
                    _ => return Err(ErrorCode::InvalidMessage),
                };
                let trace_ref = if payload["trace_ref"].is_null() {
                    None
                } else {
                    Some(handle(payload, "trace_ref")?)
                };
                Ok(Self::CreatePlan {
                    configuration_ref: handle(payload, "configuration_ref")?,
                    mode,
                    trace_ref,
                })
            }
            "plan_input_choose" => {
                fields(payload, &["plan_ref", "trace_ref"])?;
                Ok(Self::ChooseInput {
                    plan_ref: handle(payload, "plan_ref")?,
                    trace_ref: handle(payload, "trace_ref")?,
                })
            }
            "catalog_page" => {
                fields(payload, &["offset"])?;
                Ok(Self::Catalog {
                    offset: offset(payload)?,
                })
            }
            "configuration_select" => {
                fields(payload, &["schema_ref", "encoding"])?;
                if payload["encoding"] != "utf8" {
                    return Err(ErrorCode::InvalidMessage);
                }
                Ok(Self::Select {
                    schema_ref: handle(payload, "schema_ref")?,
                })
            }
            "configuration_page" => {
                fields(payload, &["configuration_ref", "offset"])?;
                Ok(Self::Descriptors {
                    configuration_ref: handle(payload, "configuration_ref")?,
                    offset: offset(payload)?,
                })
            }
            _ => Err(ErrorCode::UnknownOperation),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    mod full_schema {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../tests/support/full_schema.rs"
        ));
    }

    fn no_picker() -> Result<Option<PathBuf>, ErrorCode> {
        Err(ErrorCode::TechnicalFailure)
    }

    fn request(operation: &str, value: Value) -> ConfigurationRequest {
        ConfigurationRequest::decode(operation, value.as_object().unwrap()).unwrap()
    }
    fn call(configuration: &mut Configuration, request: ConfigurationRequest) -> Value {
        serde_json::from_slice(
            &configuration
                .dispatch(request, "test_1", &no_picker)
                .unwrap(),
        )
        .unwrap()
    }
    fn fixture(count: usize) -> (tempfile::TempDir, Configuration) {
        let embedded = EmbeddedBundle::builtin().unwrap();
        let sample: Value =
            serde_json::from_slice(embedded.file("schemas/SYNTHETIC/v1/flow.json").unwrap())
                .unwrap();
        fixture_model(count, sample)
    }
    fn fixture_model(count: usize, sample: Value) -> (tempfile::TempDir, Configuration) {
        let directory = tempfile::tempdir().unwrap();
        let schemas = directory.path().join("schemas");
        let reference = directory.path().join("reference-data");
        fs::create_dir(&schemas).unwrap();
        fs::create_dir(&reference).unwrap();
        for index in 0..count {
            let version = format!("v{index:03}");
            let folder = schemas.join("SYNTHETIC").join(&version);
            fs::create_dir_all(&folder).unwrap();
            let mut raw = sample.clone();
            raw["schema_version"] = json!(version);
            fs::write(folder.join("flow.json"), serde_json::to_vec(&raw).unwrap()).unwrap();
        }
        let bundle = AnalysisBundle::from_selection(BundleSelection::External {
            schema_root: schemas,
            reference_data_root: reference,
        })
        .unwrap();
        (directory, Configuration::new(bundle).unwrap())
    }

    #[test]
    fn catalog_is_paged_in_acquisition_order_with_real_provenance_and_no_paths() {
        let (directory, mut state) = fixture(70);
        let page = call(&mut state, ConfigurationRequest::Catalog { offset: 0 });
        assert_eq!(
            page["payload"]["entries"].as_array().unwrap().len(),
            PAGE_SIZE
        );
        assert_eq!(page["payload"]["entries"][0]["schema_version"], "v000");
        assert_eq!(page["payload"]["next_offset"], "64");
        assert_eq!(
            page["payload"]["provenance"],
            json!({"schemaVersion":1,"bundle_source":"external","bundle_version":null,"bundle_manifest_status":"absent"})
        );
        assert!(
            !page
                .to_string()
                .contains(directory.path().to_str().unwrap())
        );
        let page = call(&mut state, ConfigurationRequest::Catalog { offset: 64 });
        assert_eq!(page["payload"]["entries"].as_array().unwrap().len(), 6);
        assert_eq!(page["payload"]["next_offset"], Value::Null);
        assert_eq!(
            call(&mut state, ConfigurationRequest::Catalog { offset: 70 })["payload"]["entries"],
            json!([])
        );
        assert_eq!(
            state.dispatch(
                ConfigurationRequest::Catalog { offset: 71 },
                "test_1",
                &no_picker
            ),
            Err(ErrorCode::InvalidMessage)
        );
        assert!(state.selected.is_none());
    }

    #[test]
    fn selected_snapshot_hash_and_descriptors_survive_external_change_but_old_refs_expire() {
        let (directory, mut state) = fixture(2);
        let schema_ref = state.schema_ref(0);
        let selected = call(
            &mut state,
            request(
                "configuration_select",
                json!({"schemaVersion":1,"schema_ref":schema_ref,"encoding":"utf8"}),
            ),
        );
        let selected_ref = selected["payload"]["configuration_ref"]
            .as_str()
            .unwrap()
            .to_owned();
        let expected = schema_fingerprint(&state.selected.as_ref().unwrap().raw)
            .unwrap()
            .to_string();
        assert_eq!(selected["payload"]["schema_hash"], expected);
        fs::remove_file(directory.path().join("schemas/SYNTHETIC/v000/flow.json")).unwrap();
        let page = call(
            &mut state,
            ConfigurationRequest::Descriptors {
                configuration_ref: selected_ref.clone(),
                offset: 0,
            },
        );
        assert_eq!(page["payload"]["descriptors"][0]["trace_id"], "T1");
        assert_eq!(page["payload"]["descriptors"][0]["kind"], "trace");
        let next = state.schema_ref(1);
        call(
            &mut state,
            ConfigurationRequest::Select { schema_ref: next },
        );
        assert_eq!(
            state.dispatch(
                ConfigurationRequest::Descriptors {
                    configuration_ref: selected_ref,
                    offset: 0
                },
                "test_1",
                &no_picker
            ),
            Err(ErrorCode::StaleContext)
        );
    }

    #[test]
    fn failed_selection_overflow_and_foreign_handles_preserve_current_configuration() {
        let (_directory, mut state) = fixture(1);
        let schema_ref = state.schema_ref(0);
        let selected = call(
            &mut state,
            ConfigurationRequest::Select {
                schema_ref: schema_ref.clone(),
            },
        );
        let before = selected["payload"]["configuration_ref"].as_str().unwrap();
        let (_other_directory, other) = fixture(1);
        assert_eq!(
            state.dispatch(
                ConfigurationRequest::Select {
                    schema_ref: other.schema_ref(0)
                },
                "test_1",
                &no_picker
            ),
            Err(ErrorCode::StaleContext)
        );
        assert_eq!(state.selected.as_ref().unwrap().reference, before);
        state.generation = u64::MAX;
        assert_eq!(
            state.dispatch(
                ConfigurationRequest::Select { schema_ref },
                "test_1",
                &no_picker
            ),
            Err(ErrorCode::TechnicalFailure)
        );
        assert_eq!(state.selected.as_ref().unwrap().reference, before);
    }

    #[test]
    fn malformed_dtos_and_forged_path_or_owner_never_reach_acquisition() {
        for payload in [
            json!({"schemaVersion":1,"offset":0}),
            json!({"schemaVersion":1,"offset":"01"}),
            json!({"schemaVersion":1,"offset":"-1"}),
            json!({"schemaVersion":1,"offset":"18446744073709551616"}),
            json!({"schemaVersion":1,"offset":"0","path":"SYNTHETIC"}),
            json!({"offset":"0","owner":"main"}),
        ] {
            assert!(
                ConfigurationRequest::decode("catalog_page", payload.as_object().unwrap()).is_err()
            );
        }
        for payload in [
            json!({"schemaVersion":1,"schema_ref":"x","encoding":"utf8","owner":"main"}),
            json!({"schemaVersion":1,"schema_ref":"x","encoding":"auto"}),
            json!({"schemaVersion":1,"schema_ref":"../file","encoding":"utf8"}),
        ] {
            assert!(
                ConfigurationRequest::decode("configuration_select", payload.as_object().unwrap())
                    .is_err()
            );
        }
    }

    fn local_plan(state: &mut Configuration) -> (String, String) {
        let schema_ref = state.schema_ref(0);
        let selected = call(state, ConfigurationRequest::Select { schema_ref });
        let reference = selected["payload"]["configuration_ref"]
            .as_str()
            .unwrap()
            .to_owned();
        let trace_ref = format!("{reference}_trace_0");
        let plan = call(
            state,
            ConfigurationRequest::CreatePlan {
                configuration_ref: reference,
                mode: ValidationMode::LocalOnly,
                trace_ref: Some(trace_ref.clone()),
            },
        );
        (
            plan["payload"]["plan_ref"].as_str().unwrap().to_owned(),
            trace_ref,
        )
    }

    #[test]
    fn native_choice_returns_only_probe_counters_and_rejects_old_plan_before_picker() {
        let (directory, mut state) = fixture(1);
        let (plan_ref, trace_ref) = local_plan(&mut state);
        let path = directory.path().join("synthetic input = file.txt");
        fs::write(&path, b"001A  \r\n").unwrap();
        let bytes = state
            .dispatch(
                ConfigurationRequest::ChooseInput {
                    plan_ref: plan_ref.clone(),
                    trace_ref: trace_ref.clone(),
                },
                "test_1",
                &|| Ok(Some(path.clone())),
            )
            .unwrap();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["payload"]["disposition"], "selected");
        assert_eq!(value["payload"]["preview"]["file_size"], "8");
        assert_eq!(value["payload"]["preview"]["crlf"], "1");
        assert_eq!(value["payload"]["assigned_input_count"], "1");
        assert_ne!(value["payload"]["plan_ref"], plan_ref);
        assert!(
            !String::from_utf8(bytes)
                .unwrap()
                .contains("synthetic input")
        );
        assert_eq!(fs::read(&path).unwrap(), b"001A  \r\n");
        let called = std::cell::Cell::new(false);
        assert_eq!(
            state.dispatch(
                ConfigurationRequest::ChooseInput {
                    plan_ref,
                    trace_ref
                },
                "test_1",
                &|| {
                    called.set(true);
                    Ok(Some(path.clone()))
                }
            ),
            Err(ErrorCode::StaleContext)
        );
        assert!(!called.get());
    }

    #[test]
    fn cancellation_bom_failure_and_forged_fields_do_not_replace_the_plan() {
        let (directory, mut state) = fixture(1);
        let (plan_ref, trace_ref) = local_plan(&mut state);
        let response = state
            .dispatch(
                ConfigurationRequest::ChooseInput {
                    plan_ref: plan_ref.clone(),
                    trace_ref: trace_ref.clone(),
                },
                "test_1",
                &|| Ok(None),
            )
            .unwrap();
        let value: Value = serde_json::from_slice(&response).unwrap();
        assert_eq!(value["payload"]["disposition"], "cancelled");
        assert_eq!(state.plan.as_ref().unwrap().reference, plan_ref);
        let path = directory.path().join("synthetic-bom.txt");
        fs::write(&path, [0xff, 0xfe, 0, 0]).unwrap();
        assert_eq!(
            state.dispatch(
                ConfigurationRequest::ChooseInput {
                    plan_ref: plan_ref.clone(),
                    trace_ref: trace_ref.clone()
                },
                "test_1",
                &|| Ok(Some(path.clone()))
            ),
            Err(ErrorCode::TechnicalFailure)
        );
        assert_eq!(state.plan.as_ref().unwrap().reference, plan_ref);
        assert_eq!(state.plan.as_ref().unwrap().assigned(), 0);
        assert!(ConfigurationRequest::decode("plan_input_choose",json!({"schemaVersion":1,"plan_ref":plan_ref,"trace_ref":trace_ref,"path":"forged"}).as_object().unwrap()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn native_non_utf8_input_path_is_retained_without_a_lossy_wire_conversion() {
        use std::os::unix::ffi::OsStringExt;
        let (directory, mut state) = fixture(1);
        let (plan_ref, trace_ref) = local_plan(&mut state);
        let path = directory
            .path()
            .join(std::ffi::OsString::from_vec(b"synthetic-\xff.txt".to_vec()));
        fs::write(&path, b"001A  \n").unwrap();
        state
            .dispatch(
                ConfigurationRequest::ChooseInput {
                    plan_ref,
                    trace_ref,
                },
                "test_1",
                &|| Ok(Some(path.clone())),
            )
            .unwrap();
        assert_eq!(
            state.plan.as_ref().unwrap().inputs[0]
                .as_ref()
                .unwrap()
                .path,
            path
        );
    }

    #[test]
    fn prepared_intent_is_frozen_and_new_selection_cannot_change_its_schema_or_request() {
        let (directory, mut state) = fixture(2);
        let (plan_ref, trace_ref) = local_plan(&mut state);
        let path = directory.path().join("synthetic.txt");
        fs::write(&path, b"001A  \n").unwrap();
        let response: Value = serde_json::from_slice(
            &state
                .dispatch(
                    ConfigurationRequest::ChooseInput {
                        plan_ref,
                        trace_ref,
                    },
                    "test_1",
                    &|| Ok(Some(path.clone())),
                )
                .unwrap(),
        )
        .unwrap();
        let plan_ref = response["payload"]["plan_ref"].as_str().unwrap().to_owned();
        let ready = call(
            &mut state,
            ConfigurationRequest::Prepare {
                plan_ref: plan_ref.clone(),
            },
        );
        assert_eq!(ready["payload"]["ready"], true);
        let captured = state
            .plan
            .as_ref()
            .unwrap()
            .prepared
            .as_ref()
            .unwrap()
            .clone();
        fs::remove_file(directory.path().join("schemas/SYNTHETIC/v000/flow.json")).unwrap();
        let next = state.schema_ref(1);
        call(
            &mut state,
            ConfigurationRequest::Select { schema_ref: next },
        );
        assert!(state.plan.is_none());
        assert_eq!(captured.raw.schema_version, "v000");
        assert_eq!(
            captured.request.schema_key.version,
            SchemaVersion("v000".into())
        );
        assert_eq!(captured.request.mode, ValidationMode::LocalOnly);
        assert!(
            matches!(&captured.request.scope,ValidationScope::Trace(input) if input.source==InputSource::Path(path.clone()))
        );
        assert_eq!(
            schema_fingerprint(&captured.raw).unwrap().to_string(),
            ready["payload"]["schema_hash"]
        );
        assert_eq!(
            state.dispatch(
                ConfigurationRequest::Prepare { plan_ref },
                "test_1",
                &no_picker
            ),
            Err(ErrorCode::StaleContext)
        );
        assert_eq!(fs::read(&path).unwrap(), b"001A  \n");
    }

    #[test]
    fn full_preparation_uses_all_required_inputs_and_never_downgrades_to_local_only() {
        let (directory, mut state) = fixture_model(1, full_schema::raw());
        let reference = state.schema_ref(0);
        call(
            &mut state,
            ConfigurationRequest::Select {
                schema_ref: reference,
            },
        );
        let selected = state.selected.as_ref().unwrap().reference.clone();
        let draft = call(
            &mut state,
            ConfigurationRequest::CreatePlan {
                configuration_ref: selected.clone(),
                mode: ValidationMode::Full,
                trace_ref: None,
            },
        );
        let mut plan_ref = draft["payload"]["plan_ref"].as_str().unwrap().to_owned();
        assert_eq!(
            state.dispatch(
                ConfigurationRequest::Prepare {
                    plan_ref: plan_ref.clone()
                },
                "test_1",
                &no_picker
            ),
            Err(ErrorCode::InvalidState)
        );
        for index in 0..2 {
            let path = directory.path().join(format!("synthetic-{index}.txt"));
            fs::write(&path, b"01AAXX\n").unwrap();
            let trace_ref = format!("{selected}_trace_{index}");
            let response: Value = serde_json::from_slice(
                &state
                    .dispatch(
                        ConfigurationRequest::ChooseInput {
                            plan_ref,
                            trace_ref,
                        },
                        "test_1",
                        &|| Ok(Some(path.clone())),
                    )
                    .unwrap(),
            )
            .unwrap();
            plan_ref = response["payload"]["plan_ref"].as_str().unwrap().to_owned();
            if index == 0 {
                assert_eq!(
                    state.dispatch(
                        ConfigurationRequest::Prepare {
                            plan_ref: plan_ref.clone()
                        },
                        "test_1",
                        &no_picker
                    ),
                    Err(ErrorCode::InvalidState)
                );
            }
        }
        let response = call(&mut state, ConfigurationRequest::Prepare { plan_ref });
        assert_eq!(response["payload"]["mode"], "full");
        assert_eq!(response["payload"]["input_count"], "2");
        let prepared = state.plan.as_ref().unwrap().prepared.as_ref().unwrap();
        assert!(
            matches!(&prepared.request.scope,ValidationScope::Flow(flow) if flow.traces.iter().map(|t|t.trace_id.0.as_str()).collect::<Vec<_>>()==["S","T"])
        );
    }

    #[test]
    fn typed_parameter_domains_reuse_core_validation_and_keep_large_integers_exact() {
        let mut raw = full_schema::raw();
        raw["parameters"] = json!([{"id":"unit","type":"string","allowed_values":["A","B"]},
            {"id":"large","type":"integer","min":-2,"max":99999999999999999999_u128}]);
        raw["traces"][0]["joins"][0]["namespace_parameters"] = json!(["unit"]);
        let (directory, mut state) = fixture_model(1, raw);
        let (plan_ref, trace_ref) = local_plan(&mut state);
        let path = directory.path().join("synthetic.txt");
        fs::write(&path, b"01AAXX\n").unwrap();
        let result: Value = serde_json::from_slice(
            &state
                .dispatch(
                    ConfigurationRequest::ChooseInput {
                        plan_ref,
                        trace_ref,
                    },
                    "test_1",
                    &|| Ok(Some(path.clone())),
                )
                .unwrap(),
        )
        .unwrap();
        let old_ref = result["payload"]["plan_ref"].as_str().unwrap().to_owned();
        let configuration = state.selected.as_ref().unwrap().reference.clone();
        assert_eq!(
            state.dispatch(
                ConfigurationRequest::SetParameter {
                    plan_ref: old_ref.clone(),
                    parameter_ref: format!("{configuration}_parameter_0"),
                    value: JoinParameterValue::Text("unknown".into())
                },
                "test_1",
                &no_picker
            ),
            Err(ErrorCode::InvalidState)
        );
        let result = call(
            &mut state,
            ConfigurationRequest::SetParameter {
                plan_ref: old_ref.clone(),
                parameter_ref: format!("{configuration}_parameter_0"),
                value: JoinParameterValue::Text("A".into()),
            },
        );
        let reference = result["payload"]["plan_ref"].as_str().unwrap().to_owned();
        assert_ne!(reference, old_ref);
        let value: IntegerLiteral = "18446744073709551615".parse().unwrap();
        let result = call(
            &mut state,
            ConfigurationRequest::SetParameter {
                plan_ref: reference,
                parameter_ref: format!("{configuration}_parameter_1"),
                value: JoinParameterValue::Integer(value.clone()),
            },
        );
        let reference = result["payload"]["plan_ref"].as_str().unwrap().to_owned();
        call(
            &mut state,
            ConfigurationRequest::Prepare {
                plan_ref: reference.clone(),
            },
        );
        assert_eq!(
            state
                .plan
                .as_ref()
                .unwrap()
                .prepared
                .as_ref()
                .unwrap()
                .parameters["large"],
            JoinParameterValue::Integer(value)
        );
        fs::remove_file(path).unwrap();
        assert_eq!(
            state.dispatch(
                ConfigurationRequest::Prepare {
                    plan_ref: reference
                },
                "test_1",
                &no_picker
            ),
            Err(ErrorCode::TechnicalFailure)
        );
        assert!(state.plan.as_ref().unwrap().prepared.is_none());
        let draft = call(
            &mut state,
            ConfigurationRequest::CreatePlan {
                configuration_ref: configuration.clone(),
                mode: ValidationMode::Full,
                trace_ref: None,
            },
        );
        let mut plan_ref = draft["payload"]["plan_ref"].as_str().unwrap().to_owned();
        let mut paths = Vec::new();
        for index in 0..2 {
            let path = directory
                .path()
                .join(format!("missing-parameter-{index}.txt"));
            fs::write(&path, b"01AAXX\n").unwrap();
            let response: Value = serde_json::from_slice(
                &state
                    .dispatch(
                        ConfigurationRequest::ChooseInput {
                            plan_ref,
                            trace_ref: format!("{configuration}_trace_{index}"),
                        },
                        "test_1",
                        &|| Ok(Some(path.clone())),
                    )
                    .unwrap(),
            )
            .unwrap();
            plan_ref = response["payload"]["plan_ref"].as_str().unwrap().to_owned();
            paths.push(path);
        }
        for path in paths {
            fs::remove_file(path).unwrap();
        }
        // Missing Full namespace wins over nonexistent files: no file probe ran first.
        assert_eq!(
            state.dispatch(
                ConfigurationRequest::Prepare { plan_ref },
                "test_1",
                &no_picker
            ),
            Err(ErrorCode::InvalidState)
        );
    }
}

pub(crate) struct SelectedConfiguration {
    pub reference: String,
    pub raw: RawFlowSchema,
    pub schema: Arc<CompiledFlowSchema>,
    pub schema_hash: Sha256Digest,
}

#[derive(Clone)]
struct ConfirmedInput {
    path: PathBuf,
    preview: InputPreview,
}

#[derive(Clone)]
struct DraftPlan {
    reference: String,
    configuration_ref: String,
    mode: ValidationMode,
    trace: Option<usize>,
    inputs: Vec<Option<ConfirmedInput>>,
    parameters: BTreeMap<String, JoinParameterValue>,
    prepared: Option<Arc<PreparedIntent>>,
}

/// Owned, immutable execution input. No record validation or output allocation here.
pub(crate) struct PreparedIntent {
    pub request: ValidationRequest,
    pub parameters: BTreeMap<String, JoinParameterValue>,
    pub raw: Arc<RawFlowSchema>,
    pub audit: BundleAudit,
}

impl DraftPlan {
    fn assigned(&self) -> usize {
        self.inputs
            .iter()
            .filter(|input| {
                input
                    .as_ref()
                    .is_some_and(|i| !i.path.as_os_str().is_empty())
            })
            .count()
    }
    fn mode_name(&self) -> &'static str {
        match self.mode {
            ValidationMode::LocalOnly => "local_only",
            ValidationMode::Full => "full",
        }
    }
    fn dto(&self) -> Value {
        json!({"schemaVersion":1,"plan_ref":self.reference,"configuration_ref":self.configuration_ref,
            "mode":self.mode_name(),"assigned_input_count":self.assigned().to_string()})
    }
}

pub(crate) struct Configuration {
    bundle: AnalysisBundle,
    instance: String,
    generation: u64,
    selected: Option<SelectedConfiguration>,
    plan_number: u64,
    plan: Option<DraftPlan>,
}

impl Configuration {
    pub(crate) fn resolve() -> Result<Self, ErrorCode> {
        let executable = std::env::current_exe().map_err(|_| ErrorCode::TechnicalFailure)?;
        let bundle = AnalysisBundle::resolve(
            &BundleDirectories::default(),
            &SystemBundleEnvironment,
            &executable,
        )
        .map_err(|_| ErrorCode::TechnicalFailure)?;
        Self::new(bundle)
    }

    pub(crate) fn new(bundle: AnalysisBundle) -> Result<Self, ErrorCode> {
        let mut nonce = [0u8; 16];
        getrandom::fill(&mut nonce).map_err(|_| ErrorCode::TechnicalFailure)?;
        let instance = nonce.iter().map(|byte| format!("{byte:02x}")).collect();
        Ok(Self {
            bundle,
            instance,
            generation: 0,
            selected: None,
            plan_number: 0,
            plan: None,
        })
    }

    fn provenance(&self) -> Value {
        let audit = self.bundle.audit();
        json!({"schemaVersion":1,"bundle_source":audit.source(),
            "bundle_version":audit.bundle_version(),"bundle_manifest_status":audit.manifest_status()})
    }

    fn schema_ref(&self, index: usize) -> String {
        format!("{}_schema_{index}", self.instance)
    }

    fn page_end(offset: usize, count: usize) -> Result<usize, ErrorCode> {
        if offset > count {
            return Err(ErrorCode::InvalidMessage);
        }
        Ok(offset.saturating_add(PAGE_SIZE).min(count))
    }

    fn catalog_page(&self, offset: usize) -> Result<Value, ErrorCode> {
        let entries = self.bundle.entries();
        let end = Self::page_end(offset, entries.len())?;
        let rows: Vec<_> = entries[offset..end]
            .iter()
            .enumerate()
            .map(|(position, entry)| {
                json!({"schemaVersion":1,"schema_ref":self.schema_ref(offset+position),
                "flow_id":entry.key.flow_id.0,"schema_version":entry.key.version.0,
                "trace_count":entry.traces.len().to_string()})
            })
            .collect();
        Ok(
            json!({"schemaVersion":1,"offset":offset.to_string(),"entries":rows,
            "next_offset":(end<entries.len()).then(||end.to_string()),"provenance":self.provenance()}),
        )
    }

    fn prepare_selection(&self, schema_ref: &str) -> Result<SelectedConfiguration, ErrorCode> {
        let index = self
            .bundle
            .entries()
            .iter()
            .enumerate()
            .find_map(|(index, _)| (self.schema_ref(index) == schema_ref).then_some(index))
            .ok_or(ErrorCode::StaleContext)?;
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(ErrorCode::TechnicalFailure)?;
        let key = &self.bundle.entries()[index].key;
        let raw = self
            .bundle
            .repository()
            .load_raw(key)
            .map_err(|_| ErrorCode::TechnicalFailure)?;
        // One acquired raw snapshot drives compilation, metadata, hash and later plans.
        let mut repository = InMemorySchemaRepository::new();
        repository
            .insert(raw.clone())
            .map_err(|_| ErrorCode::TechnicalFailure)?;
        let registry = SchemaRegistry::new(
            Arc::new(repository),
            SchemaCompiler,
            RawResolvedEncoding::Utf8,
        );
        let resolved = registry
            .resolve_with_hash(key)
            .map_err(|_| ErrorCode::TechnicalFailure)?;
        Ok(SelectedConfiguration {
            reference: format!("{}_config_{generation}", self.instance),
            raw,
            schema: resolved.schema().clone(),
            schema_hash: resolved.schema_hash().clone(),
        })
    }

    fn selection_dto(&self, selected: &SelectedConfiguration) -> Value {
        json!({"schemaVersion":1,"configuration_ref":selected.reference,
            "flow_id":selected.raw.flow_id,"schema_version":selected.raw.schema_version,
            "schema_hash":selected.schema_hash.to_string(),"encoding":"utf8",
            "trace_count":selected.schema.traces().len().to_string(),
            "parameter_count":selected.schema.parameters().len().to_string(),"provenance":self.provenance()})
    }

    fn descriptors(&self, configuration_ref: &str, offset: usize) -> Result<Value, ErrorCode> {
        let selected = self
            .selected
            .as_ref()
            .filter(|s| s.reference == configuration_ref)
            .ok_or(ErrorCode::StaleContext)?;
        let traces = selected.schema.traces();
        let parameters = selected.schema.parameters();
        let count = traces
            .len()
            .checked_add(parameters.len())
            .ok_or(ErrorCode::TechnicalFailure)?;
        let end = Self::page_end(offset, count)?;
        let rows: Vec<_> = (offset..end).map(|index| {
            if let Some(trace) = traces.get(index) {
                json!({"schemaVersion":1,"kind":"trace","trace_ref":format!("{}_trace_{index}",selected.reference),"trace_id":trace.id().0})
            } else {
                let position = index - traces.len();
                let (id, kind) = match &parameters[position] {
                    ParameterDefinition::String { id, .. } => (id, "text"),
                    ParameterDefinition::Integer { id, .. } => (id, "integer"),
                };
                json!({"schemaVersion":1,"kind":"parameter","parameter_ref":format!("{}_parameter_{position}",selected.reference),"parameter_id":id,"value_kind":kind})
            }
        }).collect();
        Ok(
            json!({"schemaVersion":1,"configuration_ref":configuration_ref,"offset":offset.to_string(),
            "descriptors":rows,"next_offset":(end<count).then(||end.to_string())}),
        )
    }

    fn trace_index(selected: &SelectedConfiguration, trace_ref: &str) -> Result<usize, ErrorCode> {
        selected
            .schema
            .traces()
            .iter()
            .enumerate()
            .find_map(|(index, _)| {
                (format!("{}_trace_{index}", selected.reference) == trace_ref).then_some(index)
            })
            .ok_or(ErrorCode::StaleContext)
    }

    fn create_plan(
        &mut self,
        configuration_ref: &str,
        mode: ValidationMode,
        trace_ref: Option<&str>,
        request_id: &str,
    ) -> Result<Vec<u8>, ErrorCode> {
        let selected = self
            .selected
            .as_ref()
            .filter(|s| s.reference == configuration_ref)
            .ok_or(ErrorCode::StaleContext)?;
        let trace = match (mode, trace_ref) {
            (ValidationMode::LocalOnly, Some(trace)) => Some(Self::trace_index(selected, trace)?),
            (ValidationMode::Full, None) => None,
            _ => return Err(ErrorCode::InvalidMessage),
        };
        let number = self
            .plan_number
            .checked_add(1)
            .ok_or(ErrorCode::TechnicalFailure)?;
        let plan = DraftPlan {
            reference: format!("{}_plan_{number}", self.instance),
            configuration_ref: configuration_ref.to_owned(),
            mode,
            trace,
            inputs: vec![None; selected.schema.traces().len()],
            parameters: BTreeMap::new(),
            prepared: None,
        };
        let bytes = ipc::response(request_id, plan.dto())?;
        self.plan_number = number;
        self.plan = Some(plan);
        Ok(bytes)
    }

    fn choose_input(
        &mut self,
        plan_ref: &str,
        trace_ref: &str,
        request_id: &str,
        choose: &dyn Fn() -> Result<Option<PathBuf>, ErrorCode>,
    ) -> Result<Vec<u8>, ErrorCode> {
        let selected = self.selected.as_ref().ok_or(ErrorCode::StaleContext)?;
        let plan = self
            .plan
            .as_ref()
            .filter(|p| p.reference == plan_ref && p.configuration_ref == selected.reference)
            .ok_or(ErrorCode::StaleContext)?;
        let index = Self::trace_index(selected, trace_ref)?;
        if plan.mode == ValidationMode::LocalOnly && plan.trace != Some(index) {
            return Err(ErrorCode::InvalidState);
        }
        let number = self
            .plan_number
            .checked_add(1)
            .ok_or(ErrorCode::TechnicalFailure)?;
        let Some(path) = choose()? else {
            return ipc::response(
                request_id,
                json!({"schemaVersion":1,"plan_ref":plan_ref,"trace_ref":trace_ref,
                "disposition":"cancelled","assigned_input_count":plan.assigned().to_string(),"preview":null}),
            );
        };
        let preview = InputPreview::inspect(&path).map_err(|_| ErrorCode::TechnicalFailure)?;
        let mut next = plan.clone();
        next.reference = format!("{}_plan_{number}", self.instance);
        next.inputs[index] = Some(ConfirmedInput { path, preview });
        next.prepared = None;
        let preview = &next.inputs[index]
            .as_ref()
            .expect("confirmed input assigned")
            .preview;
        let dto = json!({"schemaVersion":1,"plan_ref":next.reference,"trace_ref":trace_ref,"disposition":"selected",
            "assigned_input_count":next.assigned().to_string(),"preview":{"schemaVersion":1,"file_size":preview.file_size.to_string(),
            "sampled_bytes":preview.sampled_bytes.to_string(),"lf":preview.lf.to_string(),"crlf":preview.crlf.to_string(),
            "lone_cr":preview.lone_cr.to_string(),"reached_eof":preview.reached_eof}});
        let bytes = ipc::response(request_id, dto)?;
        self.plan_number = number;
        self.plan = Some(next);
        Ok(bytes)
    }

    fn set_parameter(
        &mut self,
        plan_ref: &str,
        parameter_ref: &str,
        value: JoinParameterValue,
        request_id: &str,
    ) -> Result<Vec<u8>, ErrorCode> {
        let selected = self.selected.as_ref().ok_or(ErrorCode::StaleContext)?;
        let plan = self
            .plan
            .as_ref()
            .filter(|p| p.reference == plan_ref && p.configuration_ref == selected.reference)
            .ok_or(ErrorCode::StaleContext)?;
        let definition = selected
            .schema
            .parameters()
            .iter()
            .enumerate()
            .find_map(|(index, definition)| {
                (format!("{}_parameter_{index}", selected.reference) == parameter_ref)
                    .then_some(definition)
            })
            .ok_or(ErrorCode::StaleContext)?;
        let id = match (definition, &value) {
            (ParameterDefinition::String { id, .. }, JoinParameterValue::Text(_))
            | (ParameterDefinition::Integer { id, .. }, JoinParameterValue::Integer(_)) => {
                id.to_string()
            }
            _ => return Err(ErrorCode::InvalidMessage),
        };
        // Validate this partial binding with the core, without requiring a Full horizon yet.
        VerifiedRunContext::new(
            &selected.schema,
            BTreeMap::from([(id.clone(), value.clone())]),
            ValidationMode::LocalOnly,
        )
        .map_err(|_| ErrorCode::InvalidState)?;
        let number = self
            .plan_number
            .checked_add(1)
            .ok_or(ErrorCode::TechnicalFailure)?;
        let mut next = plan.clone();
        next.reference = format!("{}_plan_{number}", self.instance);
        next.parameters.insert(id, value);
        next.prepared = None;
        let bytes = ipc::response(request_id, next.dto())?;
        self.plan_number = number;
        self.plan = Some(next);
        Ok(bytes)
    }

    fn capture(
        &self,
        plan: &DraftPlan,
        selected: &SelectedConfiguration,
    ) -> Result<PreparedIntent, ErrorCode> {
        let traces = selected.schema.traces();
        let input = |index: usize| -> Result<TraceValidationInput, ErrorCode> {
            let chosen = plan
                .inputs
                .get(index)
                .and_then(Option::as_ref)
                .ok_or(ErrorCode::InvalidState)?;
            Ok(TraceValidationInput {
                trace_id: traces[index].id().clone(),
                source: InputSource::Path(chosen.path.clone()),
            })
        };
        let scope = match plan.mode {
            ValidationMode::LocalOnly => {
                ValidationScope::Trace(input(plan.trace.ok_or(ErrorCode::InvalidState)?)?)
            }
            ValidationMode::Full => {
                let inputs = plan
                    .inputs
                    .iter()
                    .enumerate()
                    .filter(|(_, source)| source.is_some())
                    .map(|(index, _)| input(index))
                    .collect::<Result<Vec<_>, _>>()?;
                if inputs.is_empty() {
                    return Err(ErrorCode::InvalidState);
                }
                ValidationScope::Flow(FlowValidationInput { traces: inputs })
            }
        };
        let request = ValidationRequest {
            schema_key: selected.schema.key().clone(),
            scope,
            mode: plan.mode,
            options: ValidationOptions {
                record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                newline_policy: NewlinePolicy::AutoLfOrCrlf,
                max_errors: None,
            },
        };
        VerifiedRunContext::new(&selected.schema, plan.parameters.clone(), plan.mode)
            .map_err(|_| ErrorCode::InvalidState)?;
        match &request.scope {
            ValidationScope::Trace(_) => {
                preflight(&request, &selected.schema).map_err(|_| ErrorCode::InvalidState)?;
            }
            ValidationScope::Flow(flow) => {
                let inputs = flow
                    .traces
                    .iter()
                    .map(|input| {
                        let InputSource::Path(path) = &input.source;
                        FlowTraceInput {
                            trace_id: &input.trace_id,
                            path,
                        }
                    })
                    .collect::<Vec<_>>();
                preflight_flow_inputs(&selected.schema, &inputs, &plan.parameters)
                    .map_err(|_| ErrorCode::InvalidState)?;
            }
        }
        // Static schema/parameter/scope checks above precede all repeated file probes.
        for input in plan.inputs.iter().flatten() {
            InputPreview::inspect(&input.path).map_err(|_| ErrorCode::TechnicalFailure)?;
        }
        if schema_fingerprint(&selected.raw).map_err(|_| ErrorCode::TechnicalFailure)?
            != selected.schema_hash
        {
            return Err(ErrorCode::TechnicalFailure);
        }
        Ok(PreparedIntent {
            request,
            parameters: plan.parameters.clone(),
            raw: Arc::new(selected.raw.clone()),
            audit: self.bundle.audit().clone(),
        })
    }

    fn prepare(&mut self, plan_ref: &str, request_id: &str) -> Result<Vec<u8>, ErrorCode> {
        let selected = self.selected.as_ref().ok_or(ErrorCode::StaleContext)?;
        let plan = self
            .plan
            .as_ref()
            .filter(|p| p.reference == plan_ref && p.configuration_ref == selected.reference)
            .ok_or(ErrorCode::StaleContext)?;
        let prepared = match self.capture(plan, selected) {
            Ok(prepared) => Arc::new(prepared),
            Err(code) => {
                self.plan.as_mut().expect("current plan checked").prepared = None;
                return Err(code);
            }
        };
        let input_count = match &prepared.request.scope {
            ValidationScope::Trace(_) => 1,
            ValidationScope::Flow(flow) => flow.traces.len(),
        };
        let bytes = ipc::response(
            request_id,
            json!({"schemaVersion":1,"plan_ref":plan_ref,"mode":plan.mode_name(),"ready":true,
            "schema_hash":schema_fingerprint(&prepared.raw).map_err(|_|ErrorCode::TechnicalFailure)?.to_string(),
            "input_count":input_count.to_string(),"parameter_count":prepared.parameters.len().to_string(),
            "provenance":{"schemaVersion":1,"bundle_source":prepared.audit.source(),"bundle_version":prepared.audit.bundle_version(),"bundle_manifest_status":prepared.audit.manifest_status()}}),
        )?;
        self.plan.as_mut().expect("current plan checked").prepared = Some(prepared);
        Ok(bytes)
    }

    pub(crate) fn dispatch(
        &mut self,
        request: ConfigurationRequest,
        request_id: &str,
        choose: &dyn Fn() -> Result<Option<PathBuf>, ErrorCode>,
    ) -> Result<Vec<u8>, ErrorCode> {
        match request {
            ConfigurationRequest::SetParameter {
                plan_ref,
                parameter_ref,
                value,
            } => self.set_parameter(&plan_ref, &parameter_ref, value, request_id),
            ConfigurationRequest::Prepare { plan_ref } => self.prepare(&plan_ref, request_id),
            ConfigurationRequest::CreatePlan {
                configuration_ref,
                mode,
                trace_ref,
            } => self.create_plan(&configuration_ref, mode, trace_ref.as_deref(), request_id),
            ConfigurationRequest::ChooseInput {
                plan_ref,
                trace_ref,
            } => self.choose_input(&plan_ref, &trace_ref, request_id, choose),
            ConfigurationRequest::Catalog { offset } => {
                ipc::response(request_id, self.catalog_page(offset)?)
            }
            ConfigurationRequest::Descriptors {
                configuration_ref,
                offset,
            } => ipc::response(request_id, self.descriptors(&configuration_ref, offset)?),
            ConfigurationRequest::Select { schema_ref } => {
                let selected = self.prepare_selection(&schema_ref)?;
                let bytes = ipc::response(request_id, self.selection_dto(&selected))?;
                // Oversize/serialization failure leaves the previous selection and IDs intact.
                self.generation += 1;
                self.selected = Some(selected);
                self.plan = None;
                Ok(bytes)
            }
        }
    }
}
