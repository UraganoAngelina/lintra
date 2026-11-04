//! Fixed PUBLIC synthetic bytes only. All real files live in owned private
//! directories on the qualified workspace volume, never in a user's bundle.
use super::*;
use crate::*;
use std::{fs, fs::OpenOptions, io::Write};

const ID: &str = "synthetic-distribution-v1";
const KEY: &[u8; 32] = b"PUBLIC-SYNTHETIC-KEY-00000000001";

fn fixture() -> tempfile::TempDir {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/key-file-tests");
    fs::create_dir_all(&base).unwrap();
    tempfile::tempdir_in(base).unwrap()
}
fn private(parent: &Path, name: &str) -> PathBuf {
    let path = parent.join(name);
    drop(file_security::root(&path, true).unwrap());
    path
}
fn write(path: &Path, bytes: &[u8]) {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).unwrap().write_all(bytes).unwrap();
}

#[test]
fn exact_binary_key_is_redacted_and_only_the_configured_id_resolves() {
    let temp = fixture();
    let key = private(temp.path(), "bundle").join("hmac.key");
    write(&key, KEY);
    let provider = FilePersistenceKeyProvider::load(ID, &key).unwrap();
    let protector = IdentityProtector::preflight(Some(ID), &provider, None).unwrap();
    assert!(protector.audit().is_ok());
    assert!(matches!(
        provider.resolve("other"),
        Err(ProtectionError::KeyUnavailable)
    ));
    for text in [
        format!("{provider:?}"),
        format!("{:?}", provider.resolve(ID).unwrap()),
    ] {
        assert!(!text.contains(std::str::from_utf8(KEY).unwrap()));
        assert!(!text.contains(&key.to_string_lossy().to_string()));
    }
    assert!(matches!(
        FilePersistenceKeyProvider::load("", &key),
        Err(FileKeyError::InvalidKeyId)
    ));
    assert!(matches!(
        FilePersistenceKeyProvider::load(" bad ", &key),
        Err(FileKeyError::InvalidKeyId)
    ));
}

#[test]
fn absent_empty_short_long_and_non_regular_files_are_never_defaults() {
    let temp = fixture();
    let dir = private(temp.path(), "bundle");
    let key = dir.join("hmac.key");
    assert!(matches!(
        FilePersistenceKeyProvider::load(ID, &key),
        Err(FileKeyError::Unavailable)
    ));
    assert!(FilePersistenceKeyProvider::load(ID, &dir).is_err());
    for length in [0usize, 1, 31, 33, 1024 * 1024] {
        write(&key, &vec![0xaa; length]);
        assert!(matches!(
            FilePersistenceKeyProvider::load(ID, &key),
            Err(FileKeyError::InvalidLength)
        ));
        fs::remove_file(&key).unwrap();
    }
    assert!(!key.exists());
}

#[test]
fn portable_resolution_uses_only_the_supplied_executable_and_survives_relocation() {
    let temp = fixture();
    let bundle = private(temp.path(), "bundle");
    let executable = bundle.join("lintra-ui.exe");
    let key = portable_persistence_key_path(&executable).unwrap();
    assert_eq!(key, bundle.join("hmac.key"));
    write(&key, KEY);
    let original = FilePersistenceKeyProvider::load(ID, &key).unwrap();
    let moved = temp.path().join("moved bundle");
    fs::rename(&bundle, &moved).unwrap();
    let relocated_path = portable_persistence_key_path(&moved.join("lintra-ui.exe")).unwrap();
    let relocated = FilePersistenceKeyProvider::load(ID, &relocated_path).unwrap();
    let a = IdentityProtector::preflight(Some(ID), &original, None).unwrap();
    let b = IdentityProtector::preflight(Some(ID), &relocated, None).unwrap();
    b.verify_audit(&a.audit().unwrap()).unwrap();
    assert!(FilePersistenceKeyProvider::load(ID, &key).is_err());
    assert!(portable_persistence_key_path(Path::new("lintra-ui.exe")).is_err());
    assert!(portable_persistence_key_path(&moved.join("../elsewhere/lintra-ui.exe")).is_err());
    // A different executable parent never falls back to another bundle's key.
    let missing = private(temp.path(), "without key");
    assert!(
        FilePersistenceKeyProvider::load(
            ID,
            &portable_persistence_key_path(&missing.join("lintra-ui.exe")).unwrap()
        )
        .is_err()
    );
}

#[cfg(unix)]
#[test]
fn unix_permissions_links_fifo_and_real_replacement_are_rejected() {
    use std::os::unix::{
        ffi::OsStrExt,
        fs::{PermissionsExt, symlink},
    };
    let temp = fixture();
    let dir = private(temp.path(), "bundle");
    let key = dir.join("hmac.key");
    write(&key, KEY);
    fs::set_permissions(&key, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(matches!(
        FilePersistenceKeyProvider::load(ID, &key),
        Err(FileKeyError::UnsafeSource)
    ));
    fs::set_permissions(&key, fs::Permissions::from_mode(0o000)).unwrap();
    assert!(FilePersistenceKeyProvider::load(ID, &key).is_err());
    fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).unwrap();
    let alias = dir.join("alias");
    fs::hard_link(&key, &alias).unwrap();
    assert!(matches!(
        FilePersistenceKeyProvider::load(ID, &key),
        Err(FileKeyError::UnsafeSource)
    ));
    fs::remove_file(&alias).unwrap();
    symlink(&key, &alias).unwrap();
    assert!(FilePersistenceKeyProvider::load(ID, &alias).is_err());
    fs::remove_file(&alias).unwrap();
    let ancestor = temp.path().join("linked bundle");
    symlink(&dir, &ancestor).unwrap();
    assert!(FilePersistenceKeyProvider::load(ID, &ancestor.join("hmac.key")).is_err());
    let fifo = dir.join("fifo");
    let name = std::ffi::CString::new(fifo.as_os_str().as_bytes()).unwrap();
    // SAFETY: synthetic owned filename and fixed private permission bits.
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
    let started = std::time::Instant::now();
    assert!(FilePersistenceKeyProvider::load(ID, &fifo).is_err());
    assert!(started.elapsed() < std::time::Duration::from_secs(5));
    let result = file_security::key::read(&key, || {
        fs::rename(&key, dir.join("previous")).unwrap();
        write(&key, &[0xbb; 32]);
    });
    assert!(matches!(result, Err(FileKeyError::ChangedSource)));
    let result = file_security::key::read(&key, || fs::write(&key, [0xcc; 32]).unwrap());
    assert!(matches!(result, Err(FileKeyError::ChangedSource)));
}

#[test]
fn shared_key_produces_scoped_identities_but_archives_remain_independent() {
    let temp = fixture();
    let a = private(temp.path(), "installation-a");
    let b = private(temp.path(), "installation-b");
    write(&a.join("hmac.key"), KEY);
    write(&b.join("hmac.key"), KEY);
    let pa = FilePersistenceKeyProvider::load(ID, &a.join("hmac.key")).unwrap();
    let pb = FilePersistenceKeyProvider::load(ID, &b.join("hmac.key")).unwrap();
    let ra = SQLitePersistenceAdapter::create(&a.join("archive"), Some(ID), &pa).unwrap();
    let rb = SQLitePersistenceAdapter::create(&b.join("archive"), Some(ID), &pb).unwrap();
    let mut raw: RawFlowSchema = serde_json::from_slice(include_bytes!(
        "../../../bundles/embedded/schemas/SYNTHETIC/v1/flow.json"
    ))
    .unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let schema = SchemaCompiler.compile(raw).unwrap();
    let trace = &schema.traces()[0];
    let pk = build_join_key(
        &RecordView::new(b"001A  "),
        &trace.record_key().field_indexes,
        trace,
    )
    .unwrap();
    let ns = |unit: &str| {
        PersistenceNamespace::new(vec![("unit".into(), NamespaceValue::String(unit.into()))])
            .unwrap()
    };
    let scope = |unit| {
        IdentityProtectionScope::new(
            ns(unit),
            FlowId("SYNTHETIC".into()),
            PersistenceSchemaScope::SchemaVersion {
                version: SchemaVersion("v1".into()),
            },
            TraceId("T1".into()),
            "record_key".into(),
        )
        .unwrap()
    };
    let row_a = ra
        .protector()
        .protect_record_bytes(&scope("A"), &pk, &Sha256Digest([1; 32]), b"001A  ")
        .unwrap();
    let row_b = rb
        .protector()
        .protect_record_bytes(&scope("A"), &pk, &Sha256Digest([1; 32]), b"001A  ")
        .unwrap();
    assert_eq!(row_a, row_b);
    assert_ne!(
        row_a.identity(),
        &rb.protector().protect_key(&scope("B"), &pk).unwrap()
    );
    let source = SourceSubmission::new(SourceSubmissionParts {
        namespace: ns("A"),
        submission_id: "synthetic-one".into(),
        schema_key: SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        schema_hash: Sha256Digest([1; 32]),
        period: None,
        created_at: chrono::DateTime::from_timestamp(0, 0).unwrap(),
        validator_version: "synthetic".into(),
        validation_mode: ValidationMode::Full,
        validation_status: RunStatus::CompletedValid,
        reference_data_versions: vec![],
        traces: vec![SubmissionTrace {
            trace_id: TraceId("T1".into()),
            presence: TracePresence::Present { record_count: 1 },
            input_sha256: Some(Sha256Digest([2; 32])),
        }],
        fingerprint: Fingerprint::new("synthetic".into(), "fixture".into(), vec![0]).unwrap(),
        state: SessionState::Validated,
    })
    .unwrap();
    let snapshot = HistoricalSnapshot::new(
        SnapshotRevision::new(1),
        ra.protector().protect_submission(&source).unwrap(),
        vec![row_a],
    )
    .unwrap();
    let mut tx = ra.begin_transaction(&ns("A"), None).unwrap();
    tx.stage_snapshot(&snapshot).unwrap();
    tx.commit().unwrap();
    assert_eq!(rb.current_generation(&ns("A")).unwrap(), None);
    assert_eq!(
        rb.lookup_identity(&ns("A"), row_b.identity()).unwrap(),
        None
    );
    drop(ra);
    let reopened = SQLitePersistenceAdapter::open(&a.join("archive"), Some(ID), &pb).unwrap();
    assert_eq!(
        reopened.load_current_snapshot(&ns("A")).unwrap(),
        Some(snapshot)
    );
    drop(reopened);
    let before = fs::read(a.join("archive/lintra.db")).unwrap();
    let other_file = b.join("wrong.key");
    write(&other_file, &[0xbb; 32]);
    let wrong = FilePersistenceKeyProvider::load(ID, &other_file).unwrap();
    assert!(matches!(
        SQLitePersistenceAdapter::open(&a.join("archive"), Some(ID), &wrong),
        Err(PersistenceError::Protection(ProtectionError::KeyMismatch))
    ));
    assert_eq!(fs::read(a.join("archive/lintra.db")).unwrap(), before);
}
