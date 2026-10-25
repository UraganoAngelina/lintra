//! External consumer: actual filesystem schemas, source bytes and DI components.
use lintra::*;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
#[path = "support/progress.rs"]
mod progress;
use progress::{CollectingProgressObserver, NoopProgressObserver};
static NEXT: AtomicU64 = AtomicU64::new(0);

/// Owns a unique synthetic bundle; no cwd/environment mutation is needed.
struct Bundle {
    root: PathBuf,
    input: PathBuf,
}
impl Bundle {
    /// Writes the repository's real flow/version layout and a valid UTF-8 record.
    /// Two-byte É plus four ASCII bytes tests byte-oriented length six.
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lintra factory à {} {} {}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let raw = raw();
        let schema_dir = root.join(&raw.flow_id).join(&raw.schema_version);
        fs::create_dir_all(&schema_dir).unwrap();
        fs::write(
            schema_dir.join("flow.json"),
            serde_json::to_vec(&raw).unwrap(),
        )
        .unwrap();
        let input = root.join("synthetic.txt");
        fs::write(&input, "ÉABCD\n").unwrap();
        Self { root, input }
    }
}
impl Drop for Bundle {
    /// Only the directory created by this fixture is removed, after run handles close.
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
/// Reuse the approved synthetic schema without altering any golden artifact.
fn raw() -> RawFlowSchema {
    serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap()
}
/// Explicit selection derives IDs from the fixture, never from a production flow.
fn request(input: &Path) -> ValidationRequest {
    let raw = raw();
    ValidationRequest {
        schema_key: SchemaKey {
            flow_id: FlowId(raw.flow_id),
            version: SchemaVersion(raw.schema_version),
        },
        scope: ValidationScope::Trace(TraceValidationInput {
            trace_id: TraceId(raw.traces[0].id.clone()),
            source: InputSource::Path(input.into()),
        }),
        mode: ValidationMode::LocalOnly,
        options: ValidationOptions {
            record_length_policy: RecordLengthPolicy::StrictSkipRecord,
            newline_policy: NewlinePolicy::AutoLfOrCrlf,
            max_errors: None,
        },
    }
}
/// Use the real in-memory repository and compiler, rather than mocking the service.
fn registry() -> Arc<SchemaRegistry> {
    let mut repository = InMemorySchemaRepository::new();
    repository.insert(raw()).unwrap();
    Arc::new(SchemaRegistry::new(
        Arc::new(repository),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    ))
}

/// Executes factory and registry-only DI on both valid and malformed input.
/// Compare every summary field explicitly except the independent real clocks.
#[test]
fn factory_matches_memory_di_and_streams_real_progress_and_audits() {
    let bundle = Bundle::new();
    for bytes in ["ÉABCD\n".as_bytes(), b"x\nABCDEF\n"] {
        fs::write(&bundle.input, bytes).unwrap();
        let factory =
            build_default_local_service(bundle.root.clone(), RawResolvedEncoding::Utf8).unwrap();
        let injected = ValidationService::new(registry());
        let mut first_sink = VecSink::default();
        let mut second_sink = VecSink::default();
        let mut progress = CollectingProgressObserver::default();
        let before = SystemTime::now();
        let first = factory
            .validate(
                request(&bundle.input),
                &mut first_sink,
                &mut progress,
                &CancellationToken::default(),
            )
            .unwrap();
        let second = injected
            .validate(
                request(&bundle.input),
                &mut second_sink,
                &mut NoopProgressObserver,
                &CancellationToken::default(),
            )
            .unwrap();
        assert!(
            first.summary.started_at >= before && first.summary.started_at <= SystemTime::now()
        );
        assert_eq!(first.status, second.status);
        // Destructure exhaustively: adding a summary field forces this comparison
        // to be reviewed instead of silently omitting new semantic information.
        let ValidationSummary {
            started_at: _,
            elapsed: _,
            reference_validation: _,
            full_validation: _,
            validation_mode,
            run_status,
            schema_key,
            scope_summary,
            records_processed,
            valid_records,
            invalid_records,
            total_errors,
            errors_by_code,
            skipped_rules,
            truncated,
            validator_version,
            schema_hash,
            reference_data_versions,
            inputs,
        } = &first.summary;
        assert_eq!(
            (validation_mode, run_status, schema_key, scope_summary),
            (
                &second.summary.validation_mode,
                &second.summary.run_status,
                &second.summary.schema_key,
                &second.summary.scope_summary
            )
        );
        assert_eq!(
            (
                *records_processed,
                *valid_records,
                *invalid_records,
                *total_errors,
                *skipped_rules,
                *truncated
            ),
            (
                second.summary.records_processed,
                second.summary.valid_records,
                second.summary.invalid_records,
                second.summary.total_errors,
                second.summary.skipped_rules,
                second.summary.truncated
            )
        );
        assert_eq!(
            (
                errors_by_code,
                validator_version,
                schema_hash,
                reference_data_versions,
                inputs
            ),
            (
                &second.summary.errors_by_code,
                &second.summary.validator_version,
                &second.summary.schema_hash,
                &second.summary.reference_data_versions,
                &second.summary.inputs
            )
        );
        assert_eq!(first_sink.errors(), second_sink.errors());
        assert_eq!(first_sink.summaries(), std::slice::from_ref(&first.summary));
        assert_eq!(first.summary.inputs[0].bytes_read, bytes.len() as u64);
        assert!(matches!(
            first.summary.inputs[0].completion,
            InputAuditCompletion::Complete { .. }
        ));
        assert_eq!(progress.events.first(), Some(&ProgressEvent::RunStarted));
        assert_eq!(progress.events.last(), Some(&ProgressEvent::RunCompleted));
    }
}

/// Empty configuration is rejected immediately; a different explicit root cannot
/// borrow a valid sibling bundle. No global cwd/home changes make tests race.
#[test]
fn empty_root_rejected_and_wrong_explicit_root_never_falls_back() {
    assert!(matches!(
        build_default_local_service(PathBuf::new(), RawResolvedEncoding::Utf8),
        Err(SchemaLoadError::EmptyRoot)
    ));
    let bundle = Bundle::new();
    let service =
        build_default_local_service(bundle.root.join("missing root"), RawResolvedEncoding::Utf8)
            .unwrap();
    let mut sink = VecSink::default();
    assert!(matches!(
        service.validate(
            request(&bundle.input),
            &mut sink,
            &mut NoopProgressObserver,
            &CancellationToken::default()
        ),
        Err(ValidationRunError::SchemaLoad(
            SchemaLoadError::NotFound { .. }
        ))
    ));
    assert!(sink.errors().is_empty() && sink.summaries().is_empty());
}

/// Unsupported bindings reach the actual compiler and are never replaced by UTF-8.
#[test]
fn every_unsupported_encoding_binding_fails_explicitly() {
    let bundle = Bundle::new();
    for encoding in [
        RawResolvedEncoding::Windows1252,
        RawResolvedEncoding::Iso8859_1,
        RawResolvedEncoding::Other("synthetic-unsupported".into()),
    ] {
        let service = build_default_local_service(bundle.root.clone(), encoding).unwrap();
        let mut sink = VecSink::default();
        let error = service
            .validate(
                request(&bundle.input),
                &mut sink,
                &mut NoopProgressObserver,
                &CancellationToken::default(),
            )
            .unwrap_err();
        assert!(
            matches!(error, ValidationRunError::SchemaCompilation(ref error) if error.issues.iter().any(|issue| issue.code == SchemaIssueCode::UnsupportedEncoding))
        );
        assert!(sink.errors().is_empty() && sink.summaries().is_empty());
    }
}

/// Existing typed failures survive the factory: malformed JSON and forbidden Full
/// fail without a definitive summary or any synthetic success result.
#[test]
fn filesystem_parse_errors_and_full_mode_remain_typed_failures() {
    let bundle = Bundle::new();
    let service =
        build_default_local_service(bundle.root.clone(), RawResolvedEncoding::Utf8).unwrap();
    let mut req = request(&bundle.input);
    req.mode = ValidationMode::Full;
    let mut sink = VecSink::default();
    assert!(matches!(
        service.validate(
            req,
            &mut sink,
            &mut NoopProgressObserver,
            &CancellationToken::default()
        ),
        Err(ValidationRunError::InvalidRequest(
            InvalidRequestError::UnsupportedMode(ValidationMode::Full)
        ))
    ));
    let raw = raw();
    fs::write(
        bundle
            .root
            .join(raw.flow_id)
            .join(raw.schema_version)
            .join("flow.json"),
        b"{",
    )
    .unwrap();
    // A fresh service avoids the deliberate registry cache from the previous run.
    let fresh =
        build_default_local_service(bundle.root.clone(), RawResolvedEncoding::Utf8).unwrap();
    assert!(matches!(
        fresh.validate(
            request(&bundle.input),
            &mut sink,
            &mut NoopProgressObserver,
            &CancellationToken::default()
        ),
        Err(ValidationRunError::SchemaLoad(_))
    ));
    assert!(sink.summaries().is_empty());
}

/// Count real clock reads to prove the injected dependency controls metadata.
struct FixedClock(AtomicU64);
impl Clock for FixedClock {
    /// Return a fixed instant; the counter detects accidental multiple sampling.
    fn now(&self) -> SystemTime {
        self.0.fetch_add(1, Ordering::Relaxed);
        UNIX_EPOCH
    }
}
#[test]
fn with_clock_retains_deterministic_di_and_serialization() {
    let bundle = Bundle::new();
    let clock = Arc::new(FixedClock(AtomicU64::new(0)));
    let service = ValidationService::with_clock(registry(), clock.clone());
    let result = service
        .validate(
            request(&bundle.input),
            &mut VecSink::default(),
            &mut NoopProgressObserver,
            &CancellationToken::default(),
        )
        .unwrap();
    assert_eq!(clock.0.load(Ordering::Relaxed), 1);
    assert_eq!(result.summary.started_at, UNIX_EPOCH);
    let json: serde_json::Value =
        serde_json::from_slice(&write_run_summary(Vec::new(), &result.summary).unwrap()).unwrap();
    assert_eq!(json["started_at"], "1970-01-01T00:00:00Z");
    assert_eq!(json["validation_mode"], "local_only");
}
