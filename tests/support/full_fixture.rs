use lintra::*;
use serde_json::Value;
use std::{fs, path::PathBuf, process::Command};
pub struct Fixture {
    pub _root: tempfile::TempDir,
    pub source: PathBuf,
    pub target: PathBuf,
    pub schema_root: PathBuf,
    pub reference_root: PathBuf,
}
impl Fixture {
    pub fn new(raw: Value, source: &[u8], target: &[u8]) -> Self {
        let root = tempfile::tempdir().unwrap();
        let schema = root.path().join("schemas");
        let references = root.path().join("reference-data");
        fs::create_dir_all(schema.join("SYNTHETIC/v1")).unwrap();
        fs::create_dir(&references).unwrap();
        fs::write(
            schema.join("SYNTHETIC/v1/flow.json"),
            serde_json::to_vec(&raw).unwrap(),
        )
        .unwrap();
        let s = root.path().join("sorgente é=input.txt");
        let t = root.path().join("target=input.txt");
        fs::write(&s, source).unwrap();
        fs::write(&t, target).unwrap();
        Self {
            _root: root,
            source: s,
            target: t,
            schema_root: schema,
            reference_root: references,
        }
    }
    #[allow(dead_code)]
    pub fn request(&self) -> ValidationRequest {
        ValidationRequest {
            schema_key: SchemaKey {
                flow_id: FlowId("SYNTHETIC".into()),
                version: SchemaVersion("v1".into()),
            },
            scope: ValidationScope::Flow(FlowValidationInput {
                traces: vec![
                    TraceValidationInput {
                        trace_id: TraceId("T".into()),
                        source: InputSource::Path(self.target.clone()),
                    },
                    TraceValidationInput {
                        trace_id: TraceId("S".into()),
                        source: InputSource::Path(self.source.clone()),
                    },
                ],
            }),
            mode: ValidationMode::Full,
            options: ValidationOptions {
                record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                newline_policy: NewlinePolicy::AutoLfOrCrlf,
                max_errors: None,
            },
        }
    }
    #[allow(dead_code)]
    pub fn service(&self) -> ValidationService {
        build_local_service_with_reference_data(
            self.schema_root.clone(),
            self.reference_root.clone(),
            RawResolvedEncoding::Utf8,
            DatasetPreparationPolicy::ContinueWithAvailableEvidence,
        )
        .unwrap()
    }
    #[allow(dead_code)]
    pub fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lintra"));
        command
            .current_dir(self._root.path())
            .env_remove("LINTRA_SCHEMA_DIR")
            .env_remove("LINTRA_REFERENCE_DATA_DIR")
            .args([
                "validate-flow",
                "--flow",
                "SYNTHETIC",
                "--schema-version",
                "v1",
                "--encoding",
                "utf8",
            ])
            .arg("--schema-dir")
            .arg(&self.schema_root)
            .arg("--reference-data-dir")
            .arg(&self.reference_root);
        command
    }
    #[allow(dead_code)]
    pub fn trace_argument(&self, id: &str, path: &std::path::Path) -> std::ffi::OsString {
        let mut value = std::ffi::OsString::from(format!("{id}="));
        value.push(path);
        value
    }
}
