//! The schema shown in diagnostics must be the schema actually validated.
#![cfg(feature = "embedded-bundle")]
use lintra::*;
use lintra_ui::{analysis::*, worker::*};
use std::{fs, sync::Arc};
#[test]
fn mismatched_service_schema_cannot_publish_a_report_with_stale_guidance() {
    let root = tempfile::tempdir().unwrap();
    let input = root.path().join("input.txt");
    fs::write(&input, b"001A  \n").unwrap();
    let mut prepared = AnalysisSelection {
        flow: Some("SYNTHETIC".into()),
        version: Some("v1".into()),
        trace: Some("T1".into()),
        encoding: Some(RawResolvedEncoding::Utf8),
        file: Some(input),
        reports: ReportSelection::default(),
    }
    .prepare(&AnalysisBundle::builtin().unwrap())
    .unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../bundles/embedded/schemas/SYNTHETIC/v1/flow.json"
    ))
    .unwrap();
    value["traces"][0]["fields"][1]["constraints"][0]["values"] = serde_json::json!(["B"]);
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let mut repo = InMemorySchemaRepository::new();
    repo.insert(raw).unwrap();
    prepared.service = Arc::new(ValidationService::new(Arc::new(SchemaRegistry::new(
        Arc::new(repo),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    ))));
    let result = WorkerHandle::start(prepared, CancellationToken::default(), Arc::new(|| {}))
        .unwrap()
        .wait();
    assert!(matches!(result, Err(WorkerError::SchemaChanged)));
}
