//! Synthetic end-to-end soft-budget contract. Expectations concern operator
//! decisions, exact file identity and genuine validation audit, not implementation
//! structure or artificial normative outcomes.
use lintra::*;
use serde_json::json;
use std::{fs, num::NonZeroU64, path::Path};

struct Fixture {
    dir: tempfile::TempDir,
    service: ValidationService,
    request: ValidationRequest,
}
impl Fixture {
    fn new(reference: bool) -> Self {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("schemas/SYNTHETIC/v1")).unwrap();
        fs::create_dir(dir.path().join("reference-data")).unwrap();
        let mut schema: serde_json::Value =
            serde_json::from_slice(include_bytes!("fixtures/application/schema.json")).unwrap();
        if reference {
            schema["reference_data"] = json!([{"id":"DATA","dataset_id":"synthetic","version":"v1","path":"data.json","sha256":"0".repeat(64),"metadata_attributes":[]}]);
        }
        fs::write(
            dir.path().join("schemas/SYNTHETIC/v1/flow.json"),
            serde_json::to_vec(&schema).unwrap(),
        )
        .unwrap();
        let input = dir.path().join("input.txt");
        fs::write(&input, b"001A  \r\n002B  \n003C  ").unwrap();
        let service =
            build_default_local_service(dir.path().join("schemas"), RawResolvedEncoding::Utf8)
                .unwrap();
        let request = ValidationRequest {
            schema_key: SchemaKey {
                flow_id: FlowId("SYNTHETIC".into()),
                version: SchemaVersion("v1".into()),
            },
            scope: ValidationScope::Trace(TraceValidationInput {
                trace_id: TraceId("T1".into()),
                source: InputSource::Path(input),
            }),
            mode: ValidationMode::LocalOnly,
            options: ValidationOptions {
                record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                newline_policy: NewlinePolicy::AutoLfOrCrlf,
                max_errors: None,
            },
        };
        Self {
            dir,
            service,
            request,
        }
    }
    fn input(&self) -> &Path {
        let ValidationScope::Trace(input) = &self.request.scope else {
            unreachable!()
        };
        let InputSource::Path(path) = &input.source;
        path
    }
    fn prepare(&self, bytes: u64) -> Result<PreparedResourcePreflight, ResourcePreflightError> {
        self.service.resource_preflight(
            self.request.clone(),
            ResourceReferenceSource::Filesystem(&self.dir.path().join("reference-data")),
            &budget(bytes),
            &CancellationToken::default(),
        )
    }
}
fn budget(bytes: u64) -> ResourceBudget {
    ResourceBudget {
        version: 1,
        measurement_source: "synthetic test oracle, not a product threshold".into(),
        input_bytes: NonZeroU64::new(bytes),
        reference_bytes: NonZeroU64::new(bytes),
    }
}

#[test]
fn over_budget_requires_explicit_choice_and_override_keeps_eof_sha_and_all_records() {
    let f = Fixture::new(false);
    let warning = f.prepare(1).unwrap();
    assert_eq!(warning.audit().input_bytes, 21);
    assert!(warning.audit().input_budget_exceeded);
    assert!(matches!(
        warning.accept(false),
        Err(ResourcePreflightError::ConfirmationRequired)
    ));
    let accepted = f.prepare(1).unwrap().accept(true).unwrap();
    assert!(accepted.audit().proceeded_over_budget);
    accepted
        .verify_unchanged(&f.service, &f.request, &CancellationToken::default())
        .unwrap();
    let mut sink = VecSink::default();
    let result = f
        .service
        .validate(
            f.request.clone(),
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )
        .unwrap();
    assert_eq!(result.summary.records_processed, 3);
    assert!(!result.summary.truncated);
    assert!(matches!(
        result.summary.inputs[0].completion,
        InputAuditCompletion::Complete { .. }
    ));
    // Independent direct SHA over the entire original mixed-newline input.
    use sha2::{Digest, Sha256};
    let expected = Sha256Digest(Sha256::digest(fs::read(f.input()).unwrap()).into());
    assert_eq!(
        result.summary.inputs[0].completion,
        InputAuditCompletion::Complete { sha256: expected }
    );
}

#[test]
fn exact_boundary_does_not_require_override() {
    let f = Fixture::new(false);
    let accepted = f.prepare(21).unwrap().accept(false).unwrap();
    assert!(!accepted.audit().requires_confirmation());
    assert!(!accepted.audit().proceeded_over_budget);
}

#[test]
fn replacement_with_identical_bytes_invalidates_the_confirmation() {
    let f = Fixture::new(false);
    let accepted = f.prepare(1).unwrap().accept(true).unwrap();
    let copy = fs::read(f.input()).unwrap();
    fs::rename(f.input(), f.dir.path().join("previous.txt")).unwrap();
    fs::write(f.input(), copy).unwrap();
    assert!(matches!(
        accepted.verify_unchanged(&f.service, &f.request, &CancellationToken::default()),
        Err(ResourcePreflightError::ChangedFile)
    ));
}

#[test]
fn request_and_schema_changes_cannot_reuse_a_choice() {
    let f = Fixture::new(false);
    let accepted = f.prepare(1).unwrap().accept(true).unwrap();
    let mut request = f.request.clone();
    request.options.max_errors = Some(2);
    assert!(matches!(
        accepted.verify_unchanged(&f.service, &request, &CancellationToken::default()),
        Err(ResourcePreflightError::ChangedRequest)
    ));
    let path = f.dir.path().join("schemas/SYNTHETIC/v1/flow.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["regulatory_version"] = json!("other synthetic revision");
    fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(matches!(
        accepted.verify_unchanged(&f.service, &f.request, &CancellationToken::default()),
        Err(ResourcePreflightError::ChangedRequest)
    ));
}

#[test]
fn references_are_counted_and_missing_is_not_a_verified_empty_dataset() {
    let f = Fixture::new(true);
    let accepted = f.prepare(1).unwrap().accept(true).unwrap();
    assert_eq!(accepted.audit().missing_reference_files, 1);
    fs::write(
        f.dir.path().join("reference-data/data.json"),
        b"synthetic reference",
    )
    .unwrap();
    assert!(matches!(
        accepted.verify_unchanged(&f.service, &f.request, &CancellationToken::default()),
        Err(ResourcePreflightError::ChangedFile)
    ));
    let warning = f.prepare(1).unwrap();
    assert_eq!(warning.audit().reference_bytes, 19);
    assert!(warning.audit().reference_budget_exceeded);
    assert_eq!(warning.audit().missing_reference_files, 0);
}

#[test]
fn explicit_override_never_bypasses_cancel_io_or_nonregular_input() {
    let f = Fixture::new(false);
    let accepted = f.prepare(1).unwrap().accept(true).unwrap();
    let cancellation = CancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        accepted.verify_unchanged(&f.service, &f.request, &cancellation),
        Err(ResourcePreflightError::Cancelled)
    ));
    fs::remove_file(f.input()).unwrap();
    assert!(matches!(f.prepare(1), Err(ResourcePreflightError::Io(_))));
    fs::create_dir(f.input()).unwrap();
    assert!(matches!(
        f.prepare(1),
        Err(ResourcePreflightError::NotRegularFile)
    ));
}

#[test]
fn invalid_configurations_are_rejected_without_silent_defaults() {
    assert!(
        serde_json::from_value::<ResourceBudget>(
            json!({"version":1,"measurement_source":"test","input_bytes":0})
        )
        .is_err()
    );
    let f = Fixture::new(false);
    let mut config = budget(1);
    config.measurement_source.clear();
    assert!(matches!(
        f.service.resource_preflight(
            f.request.clone(),
            ResourceReferenceSource::Filesystem(f.dir.path()),
            &config,
            &CancellationToken::default()
        ),
        Err(ResourcePreflightError::InvalidConfiguration)
    ));
}

#[cfg(unix)]
#[test]
fn reference_symlinks_are_rejected_before_reading_and_after_confirmation() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new(true);
    let target = f.dir.path().join("outside.json");
    let named = f.dir.path().join("reference-data/data.json");
    fs::write(&target, b"synthetic reference").unwrap();
    symlink(&target, &named).unwrap();
    assert!(matches!(
        f.prepare(1),
        Err(ResourcePreflightError::ReferenceBoundary)
    ));
    fs::remove_file(&named).unwrap();
    fs::write(&named, b"synthetic reference").unwrap();
    let accepted = f.prepare(1).unwrap().accept(true).unwrap();
    fs::remove_file(&named).unwrap();
    symlink(&target, &named).unwrap();
    assert!(matches!(
        accepted.verify_unchanged(&f.service, &f.request, &CancellationToken::default()),
        Err(ResourcePreflightError::ReferenceBoundary)
    ));
}

#[test]
fn full_scope_counts_every_explicit_trace_without_truncation() {
    let mut f = Fixture::new(false);
    let ValidationScope::Trace(input) = &f.request.scope else {
        unreachable!()
    };
    let mut second = input.clone();
    second.trace_id = TraceId("T2".into());
    f.request.scope = ValidationScope::Flow(FlowValidationInput {
        traces: vec![input.clone(), second],
    });
    f.request.mode = ValidationMode::Full;
    assert_eq!(f.prepare(1).unwrap().audit().input_bytes, 42);
}

#[test]
fn real_cli_warning_preserves_old_reports_and_explicit_override_runs_to_eof() {
    let f = Fixture::new(false);
    let budget_path = f.dir.path().join("budget.json");
    fs::write(&budget_path, br#"{"version":1,"measurement_source":"synthetic measured 21-byte input","input_bytes":1,"reference_bytes":null}"#).unwrap();
    let errors = f.dir.path().join("errors.jsonl");
    let summary = f.dir.path().join("summary.json");
    fs::write(&errors, b"old errors").unwrap();
    fs::write(&summary, b"old summary").unwrap();
    let run = |override_choice| {
        let mut c = std::process::Command::new(env!("CARGO_BIN_EXE_lintra"));
        c.args([
            "validate",
            "--flow",
            "SYNTHETIC",
            "--schema-version",
            "v1",
            "--trace",
            "T1",
            "--encoding",
            "utf8",
        ])
        .arg("--schema-dir")
        .arg(f.dir.path().join("schemas"))
        .arg("--reference-data-dir")
        .arg(f.dir.path().join("reference-data"))
        .arg("--input")
        .arg(f.input())
        .arg("--errors")
        .arg(&errors)
        .arg("--summary")
        .arg(&summary)
        .arg("--resource-budget")
        .arg(&budget_path);
        if override_choice {
            c.arg("--proceed-over-budget");
        }
        c.output().unwrap()
    };
    let refused = run(false);
    assert_eq!(refused.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("Procedi comunque"));
    assert_eq!(fs::read(&errors).unwrap(), b"old errors");
    assert_eq!(fs::read(&summary).unwrap(), b"old summary");
    let accepted = run(true);
    assert_eq!(
        accepted.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    assert!(String::from_utf8_lossy(&accepted.stderr).contains("\"proceeded_over_budget\":true"));
    let report: serde_json::Value = serde_json::from_slice(&fs::read(summary).unwrap()).unwrap();
    assert_eq!(report["records_processed"], 3);
    assert_eq!(report["result"], "INVALID");
    assert_eq!(report["total_errors"], 4);
}
