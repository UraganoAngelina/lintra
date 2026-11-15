//! Native catalog/selection only; schema compilation stays in the existing library.
use crate::ipc::{self, ErrorCode};
use lintra::*;
use serde_json::{Map, Value, json};
use std::sync::Arc;

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

    fn request(operation: &str, value: Value) -> ConfigurationRequest {
        ConfigurationRequest::decode(operation, value.as_object().unwrap()).unwrap()
    }
    fn call(configuration: &mut Configuration, request: ConfigurationRequest) -> Value {
        serde_json::from_slice(&configuration.dispatch(request, "test_1").unwrap()).unwrap()
    }
    fn fixture(count: usize) -> (tempfile::TempDir, Configuration) {
        let directory = tempfile::tempdir().unwrap();
        let schemas = directory.path().join("schemas");
        let reference = directory.path().join("reference-data");
        fs::create_dir(&schemas).unwrap();
        fs::create_dir(&reference).unwrap();
        let embedded = EmbeddedBundle::builtin().unwrap();
        let sample: Value =
            serde_json::from_slice(embedded.file("schemas/SYNTHETIC/v1/flow.json").unwrap())
                .unwrap();
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
            state.dispatch(ConfigurationRequest::Catalog { offset: 71 }, "test_1"),
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
                "test_1"
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
                "test_1"
            ),
            Err(ErrorCode::StaleContext)
        );
        assert_eq!(state.selected.as_ref().unwrap().reference, before);
        state.generation = u64::MAX;
        assert_eq!(
            state.dispatch(ConfigurationRequest::Select { schema_ref }, "test_1"),
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
}

pub(crate) struct SelectedConfiguration {
    pub reference: String,
    pub raw: RawFlowSchema,
    pub schema: Arc<CompiledFlowSchema>,
    pub schema_hash: Sha256Digest,
}

pub(crate) struct Configuration {
    bundle: AnalysisBundle,
    instance: String,
    generation: u64,
    selected: Option<SelectedConfiguration>,
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

    pub(crate) fn dispatch(
        &mut self,
        request: ConfigurationRequest,
        request_id: &str,
    ) -> Result<Vec<u8>, ErrorCode> {
        match request {
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
                Ok(bytes)
            }
        }
    }
}
