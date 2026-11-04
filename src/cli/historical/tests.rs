//! Real filesystem/key/service/SQLite/report path, exclusively synthetic data.
use super::super::args::{self, CliCommand};
use super::*;
use std::{
    ffi::OsString,
    fs::OpenOptions,
    path::{Path, PathBuf},
    sync::Arc,
};

struct Bootstrap;
impl PersistenceKeyProvider for Bootstrap {
    fn resolve(&self, _: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        Ok(Arc::new(PersistenceKey::new(vec![0xaa; 32])?))
    }
}
struct Fixture {
    temp: tempfile::TempDir,
}
impl Fixture {
    fn new() -> Self {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/cli-history-tests");
        fs::create_dir_all(&base).unwrap();
        let value = Self {
            temp: tempfile::tempdir_in(base).unwrap(),
        };
        // Use the qualified backend's private directory ACL for test setup on
        // Windows as well as Linux. This bootstrap DB is NOT the tested archive.
        drop(
            SQLitePersistenceAdapter::create(
                &value.path("private bundle"),
                Some("bootstrap"),
                &Bootstrap,
            )
            .unwrap(),
        );
        write_key(&value.path("private bundle/hmac.key"));
        fs::create_dir_all(value.path("data/schemas/SYNTHETIC/v1")).unwrap();
        fs::create_dir_all(value.path("data/reference-data")).unwrap();
        let mut schema: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../tests/golden/synthetic/v1/schema.json"
        ))
        .unwrap();
        schema["parameters"] = json!([
            {"id":"unit","type":"string","allowed_values":["001","002"]},
            {"id":"year","type":"integer","min":2020,"max":2100}
        ]);
        schema["required_capabilities"] = json!(["local.v1", "persistence.audit.v1"]);
        schema["persistence_policy"] = json!({"namespace_parameters":["unit"],
            "version_compatibility":{"kind":"same_schema_version"},
            "run_uniqueness":[{"trace":"T1","key":"record_key"}],
            "cross_run_audit":[{"trace":"T1","key":"record_key"}],
            "history_identity_invariants":[],"snapshot":null,"temporal_rules":[],
            "storage_protection":"external_secure_configuration_required"});
        fs::write(
            value.path("data/schemas/SYNTHETIC/v1/flow.json"),
            serde_json::to_vec_pretty(&schema).unwrap(),
        )
        .unwrap();
        fs::write(value.path("input.dat"), b"001A  \n").unwrap();
        initialize(value.archive_args()).unwrap();
        value
    }
    fn path(&self, path: &str) -> PathBuf {
        self.temp.path().join(path)
    }
    fn archive_args(&self) -> ArchiveArgs {
        ArchiveArgs {
            archive: self.path("archive"),
            key_id: "synthetic-distribution".into(),
            key_file: self.path("private bundle/hmac.key"),
        }
    }
    fn args(&self, name: &str, review: bool) -> HistoricalArgs {
        let out = self.path(name);
        fs::create_dir_all(&out).unwrap();
        let mut argv: Vec<OsString> = ["lintra", "history", "preview"].map(Into::into).into();
        for (flag, value) in [
            ("--flow", OsString::from("SYNTHETIC")),
            ("--schema-version", "v1".into()),
            ("--encoding", "utf8".into()),
            ("--parameter", "unit=001".into()),
            ("--parameter", "year=2026".into()),
            ("--trace-input", {
                let mut arg = OsString::from("T1=");
                arg.push(self.path("input.dat"));
                arg
            }),
            ("--schema-dir", self.path("data/schemas").into_os_string()),
            (
                "--reference-data-dir",
                self.path("data/reference-data").into_os_string(),
            ),
            ("--key-id", "synthetic-distribution".into()),
            (
                "--key-file",
                self.path("private bundle/hmac.key").into_os_string(),
            ),
            ("--archive", self.path("archive").into_os_string()),
            ("--submission-id", name.into()),
            ("--errors", out.join("errors.jsonl").into_os_string()),
            ("--summary", out.join("summary.json").into_os_string()),
            ("--audit", out.join("audit.jsonl").into_os_string()),
            (
                "--audit-summary",
                out.join("audit-summary.json").into_os_string(),
            ),
        ] {
            argv.push(flag.into());
            argv.push(value);
        }
        if review {
            argv.push("--review".into());
        }
        let CliCommand::Historical(args) = args::parse(argv).unwrap() else {
            panic!("historical parser variant")
        };
        *args
    }
    fn repository(&self) -> SQLitePersistenceAdapter {
        let args = self.archive_args();
        let provider = FilePersistenceKeyProvider::load(&args.key_id, &args.key_file).unwrap();
        SQLitePersistenceAdapter::open(&args.archive, Some(&args.key_id), &provider).unwrap()
    }
    fn namespace(&self) -> PersistenceNamespace {
        PersistenceNamespace::new(vec![("unit".into(), NamespaceValue::String("001".into()))])
            .unwrap()
    }
    fn empty(&self) {
        assert_eq!(
            self.repository()
                .current_generation(&self.namespace())
                .unwrap(),
            None
        );
    }
    fn no_parts(&self, folder: &str) {
        for entry in fs::read_dir(self.path(folder)).unwrap() {
            assert!(
                !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".part")
            );
        }
    }
}
fn write_key(path: &Path) {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).unwrap().write_all(&[0xaa; 32]).unwrap();
}

#[test]
fn preview_publishes_real_audit_without_approval_and_explicit_review_commits() {
    let fixture = Fixture::new();
    let token = CancellationToken::default();
    let mut output = Vec::new();
    let mut args = fixture.args("preview", false);
    args.validation.errors = None; // exercise the secure canonical JSONL spool
    args.validation.xlsx = Some(fixture.path("preview/report.xlsx"));
    assert_eq!(
        execute(args, &token, &mut output, |_, _| panic!(
            "no stdin for preview"
        ))
        .unwrap(),
        RunStatus::CompletedValid
    );
    fixture.empty();
    fixture.no_parts("preview");
    let workbook =
        zip::ZipArchive::new(fs::File::open(fixture.path("preview/report.xlsx")).unwrap()).unwrap();
    assert!(
        workbook
            .file_names()
            .any(|name| name.unwrap() == "xl/workbook.xml")
    );
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.path("preview/audit-summary.json")).unwrap())
            .unwrap();
    assert_eq!(summary["phase"], "historical_preview");
    assert_eq!(summary["audit"]["counts"]["inserted"], 1);
    assert_eq!(
        summary["audit"]["metadata"]["coverage"],
        "local_repository_only"
    );
    assert_eq!(
        execute(
            fixture.args("approved", true),
            &token,
            &mut output,
            |review, _| Ok(Some(review.confirmation()))
        )
        .unwrap(),
        RunStatus::CompletedValid
    );
    let repository = fixture.repository();
    let snapshot = repository
        .load_current_snapshot(&fixture.namespace())
        .unwrap()
        .unwrap();
    assert_eq!(snapshot.revision(), SnapshotRevision::new(1));
    assert_eq!(snapshot.source().submission_id(), "approved");
    assert_eq!(snapshot.source().namespace(), &fixture.namespace());
    assert!(
        String::from_utf8(output)
            .unwrap()
            .contains("historical_commit")
    );
    fixture.no_parts("approved");
}

#[test]
fn abort_eof_wrong_confirmation_and_cancellation_never_publish_or_commit() {
    for mode in ["abort", "eof", "mismatch", "cancel"] {
        let fixture = Fixture::new();
        let token = CancellationToken::default();
        let mut args = fixture.args(mode, true);
        args.validation.xlsx = Some(fixture.path(&format!("{mode}/report.xlsx")));
        let result = execute(args, &token, &mut Vec::new(), |review, cancellation| {
            Ok(match mode {
                "abort" => Some("ABORT".into()),
                "eof" => None,
                "mismatch" => Some("APPROVE wrong candidate".into()),
                _ => {
                    cancellation.cancel();
                    Some(review.confirmation())
                }
            })
        });
        if mode == "mismatch" {
            assert!(result.is_err());
        } else {
            assert_eq!(result.unwrap(), RunStatus::Cancelled);
        }
        fixture.empty();
        fixture.no_parts(mode);
        assert!(!fixture.path(&format!("{mode}/summary.json")).exists());
        assert!(!fixture.path(&format!("{mode}/report.xlsx")).exists());
    }
}

#[test]
fn failed_report_creation_and_failed_approval_publication_leave_history_unchanged() {
    let fixture = Fixture::new();
    let args = fixture.args("creation", true);
    let stale = fixture.path("creation/audit-summary.json.part");
    fs::write(&stale, b"existing marker").unwrap();
    assert!(
        execute(
            args,
            &CancellationToken::default(),
            &mut Vec::new(),
            |_, _| panic!("no review after report failure")
        )
        .is_err()
    );
    assert_eq!(fs::read(&stale).unwrap(), b"existing marker");
    fixture.empty();
    assert!(!fixture.path("creation/errors.jsonl.part").exists());
    let args = fixture.args("publication", true);
    assert!(
        execute(
            args,
            &CancellationToken::default(),
            &mut Vec::new(),
            |review, _| {
                fs::create_dir(fixture.path("publication/audit-summary.json")).unwrap();
                Ok(Some(review.confirmation()))
            }
        )
        .is_err()
    );
    fixture.empty();
    fixture.no_parts("publication");
}

#[test]
fn concurrent_approval_after_review_triggers_cas_and_never_loses_the_winner() {
    let fixture = Fixture::new();
    let result = execute(
        fixture.args("stale", true),
        &CancellationToken::default(),
        &mut Vec::new(),
        |review, _| {
            assert_eq!(
                execute(
                    fixture.args("winner", true),
                    &CancellationToken::default(),
                    &mut Vec::new(),
                    |other, _| Ok(Some(other.confirmation()))
                )
                .unwrap(),
                RunStatus::CompletedValid
            );
            Ok(Some(review.confirmation()))
        },
    );
    assert!(result.is_err());
    let repository = fixture.repository();
    assert_eq!(
        repository
            .load_current_snapshot(&fixture.namespace())
            .unwrap()
            .unwrap()
            .source()
            .submission_id(),
        "winner"
    );
    assert_eq!(
        repository
            .load_submission(&fixture.namespace(), "stale")
            .unwrap(),
        None
    );
    fixture.no_parts("stale");
    fixture.no_parts("winner");
}

#[test]
fn ambiguous_ack_is_reported_without_retry_or_a_false_success_or_rollback() {
    let fixture = Fixture::new();
    execute(
        fixture.args("published", true),
        &CancellationToken::default(),
        &mut Vec::new(),
        |review, _| Ok(Some(review.confirmation())),
    )
    .unwrap();
    let repository = fixture.repository();
    let snapshot = repository
        .load_current_snapshot(&fixture.namespace())
        .unwrap()
        .unwrap();
    let mut lookups = 0;
    let mut output = Vec::new();
    let result = report_approval(
        Err(SessionApprovalError::Persistence(
            PersistenceError::CommitOutcomeUnknown {
                revision: snapshot.revision(),
            },
        )),
        || {
            lookups += 1;
            repository.resolve_commit(&snapshot).map_err(Into::into)
        },
        &mut output,
    );
    assert!(result.is_err());
    assert_eq!(lookups, 1);
    let event: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(event["event"], "commit_outcome_unknown");
    assert_eq!(event["read_only_resolution"], "published");
    assert_eq!(
        repository.current_generation(&fixture.namespace()).unwrap(),
        Some(snapshot.revision())
    );
    assert!(
        report_approval(
            Err(SessionApprovalError::Persistence(
                PersistenceError::CommitOutcomeUnknown {
                    revision: snapshot.revision(),
                }
            )),
            || Err(PersistenceError::StorageUnavailable.into()),
            &mut Vec::new()
        )
        .is_err()
    );
    assert!(
        report_approval(
            Err(SessionApprovalError::Persistence(
                PersistenceError::StorageBusy
            )),
            || panic!("ordinary technical errors do not trigger reconciliation"),
            &mut Vec::new()
        )
        .is_err()
    );
}

#[test]
fn normative_invalidity_has_no_audit_and_missing_or_wrong_key_never_initializes() {
    let fixture = Fixture::new();
    fs::write(fixture.path("input.dat"), b"short\n").unwrap();
    assert_eq!(
        execute(
            fixture.args("invalid", true),
            &CancellationToken::default(),
            &mut Vec::new(),
            |_, _| panic!("invalid run cannot approve")
        )
        .unwrap(),
        RunStatus::CompletedInvalid
    );
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.path("invalid/audit-summary.json")).unwrap())
            .unwrap();
    assert_eq!(summary["historical_audit"], "not_executed");
    fixture.empty();
    let before = fs::read(fixture.path("archive/lintra.db")).unwrap();
    let mut args = fixture.args("wrong", false);
    args.archive.key_id = "wrong-id".into();
    assert!(
        execute(
            args,
            &CancellationToken::default(),
            &mut Vec::new(),
            |_, _| unreachable!()
        )
        .is_err()
    );
    assert_eq!(fs::read(fixture.path("archive/lintra.db")).unwrap(), before);
    let mut args = fixture.args("missing", false);
    args.archive.archive = fixture.path("not-initialized");
    assert!(
        execute(
            args,
            &CancellationToken::default(),
            &mut Vec::new(),
            |_, _| unreachable!()
        )
        .is_err()
    );
    assert!(!fixture.path("not-initialized").exists());
}

#[test]
fn incomplete_reference_evidence_does_not_become_a_successful_historical_command() {
    let fixture = Fixture::new();
    let path = fixture.path("data/schemas/SYNTHETIC/v1/flow.json");
    let mut schema: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    schema["reference_data"] = json!([{"id":"MISSING","dataset_id":"synthetic-missing","version":"1",
        "path":"missing.json","sha256":"0".repeat(64),"metadata_attributes":[]}]);
    schema["traces"][0]["fields"][0]["constraints"] = json!([{"kind":"reference_data","datasets":["MISSING"],
        "fallback_values":[],"metadata_bindings":[]}]);
    fs::write(path, serde_json::to_vec(&schema).unwrap()).unwrap();
    let result = execute(
        fixture.args("incomplete", true),
        &CancellationToken::default(),
        &mut Vec::new(),
        |_, _| panic!("incomplete evidence cannot authorize a review"),
    );
    assert!(result.is_err());
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.path("incomplete/summary.json")).unwrap())
            .unwrap();
    assert_eq!(summary["completeness"], "partial");
    assert_eq!(summary["total_errors"], 0);
    let audit: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.path("incomplete/audit-summary.json")).unwrap())
            .unwrap();
    assert_eq!(audit["historical_audit"], "not_executed");
    fixture.empty();
    fixture.no_parts("incomplete");
}

#[test]
fn key_input_archive_output_and_part_collisions_are_rejected_before_writes() {
    for mode in ["key", "input", "archive", "outputs", "part"] {
        let fixture = Fixture::new();
        let mut args = fixture.args("collision", false);
        let original_key = fs::read(&args.archive.key_file).unwrap();
        match mode {
            "key" => args.audit = args.archive.key_file.clone(),
            "input" => {
                let ValidationScope::Flow(flow) = &mut args.validation.request.scope else {
                    unreachable!()
                };
                flow.traces[0].source = InputSource::Path(args.archive.key_file.clone());
            }
            "archive" => args.validation.summary = fixture.path("archive/lintra.db"),
            "outputs" => args.audit = args.validation.summary.clone(),
            _ => {
                args.validation.errors = Some(fixture.path("private bundle/report"));
                args.archive.key_file = fixture.path("private bundle/report.part");
                write_key(&args.archive.key_file);
            }
        }
        let key = args.archive.key_file.clone();
        assert!(
            execute(
                args,
                &CancellationToken::default(),
                &mut Vec::new(),
                |_, _| unreachable!()
            )
            .is_err()
        );
        assert_eq!(fs::read(key).unwrap(), original_key);
        fixture.empty();
        fixture.no_parts("collision");
    }
}

#[test]
fn secret_file_cannot_be_parsed_as_an_explicit_or_implicit_bundle_manifest() {
    for implicit in [false, true] {
        let fixture = Fixture::new();
        let mut args = fixture.args("manifest-collision", false);
        if implicit {
            let parent = fixture.path("private bundle");
            let schema = parent.join("schemas/SYNTHETIC/v1");
            fs::create_dir_all(&schema).unwrap();
            fs::create_dir(parent.join("reference-data")).unwrap();
            fs::copy(
                fixture.path("data/schemas/SYNTHETIC/v1/flow.json"),
                schema.join("flow.json"),
            )
            .unwrap();
            args.validation.dirs.schema_dir = Some(parent.join("schemas"));
            args.validation.dirs.reference_data_dir = Some(parent.join("reference-data"));
            args.archive.key_file = parent.join("MANIFEST.json");
            write_key(&args.archive.key_file);
        } else {
            args.validation.bundle_manifest = Some(args.archive.key_file.clone());
        }
        let key = args.archive.key_file.clone();
        let error = execute(
            args,
            &CancellationToken::default(),
            &mut Vec::new(),
            |_, _| unreachable!(),
        )
        .unwrap_err();
        // Bundle parsing of the binary key would instead yield BundleError.
        // Require the path preflight error, before a non-zeroizing JSON read.
        assert_eq!(
            error.downcast_ref::<io::Error>().map(io::Error::kind),
            Some(io::ErrorKind::InvalidInput)
        );
        assert_eq!(fs::read(key).unwrap(), vec![0xaa; 32]);
        fixture.empty();
        fixture.no_parts("manifest-collision");
    }
}
