//! Real filesystem/provider tests. Payload hashes in cases.json come from
//! independent Python hashlib, never from the Rust provider being tested.
use lintra::*;
use serde::Deserialize;
use std::{
    error::Error,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Deserialize)]
struct Case {
    name: String,
    bytes_hex: String,
    sha256: String,
}
fn case(name: &str) -> Case {
    serde_json::from_slice::<Vec<Case>>(include_bytes!("fixtures/reference_membership/cases.json"))
        .unwrap()
        .into_iter()
        .find(|case| case.name == name)
        .unwrap()
}
fn decode(hex: &str) -> Vec<u8> {
    let (pairs, remainder) = hex.as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty());
    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn key(id: &str, version: &str) -> DatasetKey {
    DatasetKey {
        id: DatasetId(id.into()),
        version: DatasetVersion(version.into()),
    }
}
fn binding(key: DatasetKey, path: &str, sha: &str) -> ReferenceDependency {
    ReferenceDependency {
        key,
        bundle_path: path.into(),
        sha256: decode(sha).try_into().unwrap(),
        metadata_attributes: vec![],
    }
}
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lintra reference é {} {} {}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("data")).unwrap();
        Self { root }
    }
    fn put(&self, case: &Case, path: &str, key: DatasetKey) -> ReferenceDependency {
        let target = self.root.join("data").join(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, decode(&case.bytes_hex)).unwrap();
        binding(key, path, &case.sha256)
    }
    fn provider(&self, deps: Vec<ReferenceDependency>) -> FileSystemReferenceDataProvider {
        FileSystemReferenceDataProvider::new(self.root.join("data"), deps).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn format_issue(evidence: &DatasetEvidence) -> Option<&MembershipFormatIssue> {
    evidence
        .anomalies()
        .iter()
        .find_map(|anomaly| match anomaly {
            DatasetAnomaly::Format { issue, .. } => Some(issue),
            _ => None,
        })
}

#[test]
fn verified_dataset_preserves_bytes_empty_key_and_independently_known_digest() {
    let fixture = Fixture::new();
    let case = case("valid");
    let id = key("synthetic", "1");
    let dep = fixture.put(&case, "nested/codes.json", id.clone());
    let provider = fixture.provider(vec![dep]);
    let prepared = provider.prepare(DatasetPreparationPolicy::Strict).unwrap();
    assert_eq!(prepared.policy(), DatasetPreparationPolicy::Strict);
    let evidence = prepared.evidence(&id).unwrap();
    assert_eq!(evidence.availability(), DatasetAvailability::Verified);
    assert!(evidence.anomalies().is_empty());
    assert_eq!(evidence.actual_sha256(), Some(evidence.expected_sha256()));
    assert_eq!(evidence.actual_sha256().unwrap().to_string(), case.sha256);
    assert_eq!(evidence.declared_key(), Some(&id));
    for bytes in [b"001 ".as_slice(), b"Ab", &[0, 255], b""] {
        let found = prepared.observe(&id, bytes).unwrap();
        assert_eq!(found.availability(), DatasetAvailability::Verified);
        assert_eq!(found.observed_present(), Some(true));
    }
    for bytes in [b"001".as_slice(), b"1", b"ab", &[0, 254]] {
        assert_eq!(
            prepared.observe(&id, bytes).unwrap().observed_present(),
            Some(false)
        );
    }
    assert!(prepared.observe(&key("synthetic", "2"), b"001 ").is_none());
    // The old port remains usable for fully verified datasets only.
    assert!(provider.load(&id).unwrap().contains(b"001 "));
    assert!(
        prepare_reference_data(std::slice::from_ref(&id), &provider)
            .unwrap()
            .get(&id)
            .unwrap()
            .contains(b"001 ")
    );
}

#[test]
fn digest_mismatch_is_degraded_with_both_hashes_and_never_leaks_through_strict_api() {
    let fixture = Fixture::new();
    let case = case("valid");
    let id = key("synthetic", "1");
    let mut dep = fixture.put(&case, "codes.json", id.clone());
    dep.sha256 = [0; 32];
    let provider = fixture.provider(vec![dep]);
    let prepared = provider
        .prepare(DatasetPreparationPolicy::ContinueWithAvailableEvidence)
        .unwrap();
    let evidence = prepared.evidence(&id).unwrap();
    assert_eq!(evidence.availability(), DatasetAvailability::Degraded);
    assert_eq!(evidence.expected_sha256().0, [0; 32]);
    assert_eq!(evidence.actual_sha256().unwrap().to_string(), case.sha256);
    assert!(matches!(
        evidence.anomalies(),
        [DatasetAnomaly::IntegrityMismatch]
    ));
    for (bytes, present) in [(b"001 ".as_slice(), true), (b"missing".as_slice(), false)] {
        let found = prepared.observe(&id, bytes).unwrap();
        assert_eq!(found.availability(), DatasetAvailability::Degraded);
        assert_eq!(found.observed_present(), Some(present));
        assert_eq!(
            found.evidence().policy(),
            DatasetPreparationPolicy::ContinueWithAvailableEvidence
        );
    }
    let error = match provider.prepare(DatasetPreparationPolicy::Strict) {
        Ok(_) => panic!("strict accepted a mismatch"),
        Err(error) => error,
    };
    assert!(
        matches!(error,FilesystemReferenceError::StrictPreparation{evidence} if evidence.availability()==DatasetAvailability::Degraded)
    );
    let error = match provider.load(&id) {
        Ok(_) => panic!("legacy leaked degraded data"),
        Err(error) => error,
    };
    assert!(matches!(error, ReferenceDataError::Provider { .. }));
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<FilesystemReferenceError>()
            .is_some()
    );
}

#[test]
fn malformed_documents_are_classified_without_permissive_or_selective_recovery() {
    let fixture = Fixture::new();
    let id = key("synthetic", "1");
    for name in [
        "malformed_json",
        "bom",
        "unknown_property",
        "duplicate_property",
        "duplicate_escaped_property",
        "invalid_hex_uppercase",
        "invalid_hex_odd",
        "invalid_hex_character",
        "duplicate_entries",
        "duplicate_empty_keys",
        "missing_property",
        "invalid_property",
        "unsupported_format",
        "trailing_json",
        "not_an_object",
        "invalid_utf8",
    ] {
        let c = case(name);
        let dep = fixture.put(&c, "bad.json", id.clone());
        let provider = fixture.provider(vec![dep]);
        let prepared = provider
            .prepare(DatasetPreparationPolicy::ContinueWithAvailableEvidence)
            .unwrap();
        let evidence = prepared.evidence(&id).unwrap();
        assert_eq!(
            evidence.availability(),
            DatasetAvailability::Unavailable,
            "{name}"
        );
        assert_eq!(
            evidence.actual_sha256().unwrap().to_string(),
            c.sha256,
            "{name}"
        );
        assert_eq!(evidence.actual_sha256(), Some(evidence.expected_sha256()));
        assert_eq!(
            prepared.observe(&id, b"001 ").unwrap().observed_present(),
            None,
            "{name}"
        );
        let issue = format_issue(evidence).unwrap();
        match name {
            "bom" => assert_eq!(issue, &MembershipFormatIssue::Bom),
            "unknown_property" => assert!(
                matches!(issue,MembershipFormatIssue::UnknownProperty(name) if name=="metadata")
            ),
            "duplicate_property" | "duplicate_escaped_property" => assert!(
                matches!(issue,MembershipFormatIssue::DuplicateProperty(name) if name=="version")
            ),
            "invalid_hex_uppercase" | "invalid_hex_odd" | "invalid_hex_character" => assert!(
                matches!(issue, MembershipFormatIssue::InvalidHex { entry: 0 })
            ),
            "duplicate_entries" | "duplicate_empty_keys" => assert!(matches!(
                issue,
                MembershipFormatIssue::DuplicateEntry { entry: 1 }
            )),
            "missing_property" => assert_eq!(
                issue,
                &MembershipFormatIssue::MissingProperty(MembershipProperty::Version)
            ),
            "invalid_property" => assert_eq!(
                issue,
                &MembershipFormatIssue::InvalidProperty(MembershipProperty::KeysHex)
            ),
            "unsupported_format" => {
                assert_eq!(issue, &MembershipFormatIssue::UnsupportedVersion(2))
            }
            _ => assert_eq!(issue, &MembershipFormatIssue::InvalidJson),
        }
        assert!(matches!(
            provider.prepare(DatasetPreparationPolicy::Strict),
            Err(FilesystemReferenceError::StrictPreparation { .. })
        ));
    }
}

#[test]
fn malformed_hash_mismatch_keeps_both_anomalies_and_the_original_digest() {
    let fixture = Fixture::new();
    let c = case("malformed_json");
    let id = key("synthetic", "1");
    let mut dep = fixture.put(&c, "bad.json", id.clone());
    dep.sha256 = [0; 32];
    let prepared = fixture
        .provider(vec![dep])
        .prepare(DatasetPreparationPolicy::ContinueWithAvailableEvidence)
        .unwrap();
    let evidence = prepared.evidence(&id).unwrap();
    assert_eq!(evidence.availability(), DatasetAvailability::Unavailable);
    assert!(matches!(
        evidence.anomalies(),
        [
            DatasetAnomaly::IntegrityMismatch,
            DatasetAnomaly::Format {
                issue: MembershipFormatIssue::InvalidJson,
                ..
            }
        ]
    ));
    assert_eq!(evidence.actual_sha256().unwrap().to_string(), c.sha256);
    assert!(
        evidence.anomalies()[1]
            .source()
            .unwrap()
            .downcast_ref::<serde_json::Error>()
            .is_some()
    );
}

#[test]
fn reliable_identity_is_required_even_when_the_file_hash_matches() {
    let fixture = Fixture::new();
    let id = key("synthetic", "1");
    for name in ["wrong_id", "wrong_version", "invalid_id"] {
        let c = case(name);
        let dep = fixture.put(&c, "wrong.json", id.clone());
        let prepared = fixture
            .provider(vec![dep])
            .prepare(DatasetPreparationPolicy::ContinueWithAvailableEvidence)
            .unwrap();
        let evidence = prepared.evidence(&id).unwrap();
        assert_eq!(evidence.availability(), DatasetAvailability::Unavailable);
        assert_ne!(evidence.declared_key(), Some(&id));
        assert_eq!(evidence.actual_sha256(), Some(evidence.expected_sha256()));
        assert!(matches!(
            evidence.anomalies(),
            [DatasetAnomaly::IdentityMismatch] | [DatasetAnomaly::InvalidIdentity]
        ));
        assert_eq!(
            prepared.observe(&id, b"001 ").unwrap().observed_present(),
            None
        );
    }
}

#[test]
fn continuation_preserves_healthy_degraded_and_missing_dependencies_in_declaration_order() {
    let fixture = Fixture::new();
    let healthy = key("healthy", "2");
    let degraded = key("synthetic", "1");
    let missing = key("missing", "1");
    let broken = key("broken", "1");
    let good = fixture.put(&case("healthy"), "healthy.json", healthy.clone());
    let mut altered = fixture.put(&case("valid"), "altered.json", degraded.clone());
    altered.sha256 = [0; 32];
    let missing_dep = binding(missing.clone(), "missing.json", &"00".repeat(32));
    let bad = fixture.put(&case("malformed_json"), "broken.json", broken.clone());
    let provider = fixture.provider(vec![good, missing_dep, altered, bad]);
    let prepared = provider
        .prepare(DatasetPreparationPolicy::ContinueWithAvailableEvidence)
        .unwrap();
    assert_eq!(
        prepared
            .datasets()
            .iter()
            .map(|entry| entry.key())
            .collect::<Vec<_>>(),
        [&healthy, &missing, &degraded, &broken]
    );
    assert_eq!(
        prepared
            .datasets()
            .iter()
            .map(|entry| entry.availability())
            .collect::<Vec<_>>(),
        [
            DatasetAvailability::Verified,
            DatasetAvailability::Unavailable,
            DatasetAvailability::Degraded,
            DatasetAvailability::Unavailable
        ]
    );
    assert_eq!(
        prepared.observe(&healthy, b"B").unwrap().observed_present(),
        Some(true)
    );
    assert_eq!(
        prepared
            .observe(&degraded, b"001 ")
            .unwrap()
            .observed_present(),
        Some(true)
    );
    assert_eq!(
        prepared
            .observe(&missing, b"001 ")
            .unwrap()
            .observed_present(),
        None
    );
    let unavailable = prepared.evidence(&missing).unwrap();
    assert!(unavailable.actual_sha256().is_none());
    assert!(unavailable.declared_key().is_none());
    assert!(
        matches!(unavailable.anomalies(),[DatasetAnomaly::Io{source,..}] if source.kind()==std::io::ErrorKind::NotFound)
    );
}

#[test]
fn empty_dataset_is_verified_but_absent_dataset_never_becomes_an_empty_set() {
    let fixture = Fixture::new();
    let id = key("synthetic", "1");
    let dep = fixture.put(&case("empty_dataset"), "empty.json", id.clone());
    let prepared = fixture
        .provider(vec![dep])
        .prepare(DatasetPreparationPolicy::ContinueWithAvailableEvidence)
        .unwrap();
    let empty = prepared.observe(&id, b"").unwrap();
    assert_eq!(empty.availability(), DatasetAvailability::Verified);
    assert_eq!(empty.observed_present(), Some(false));
    fs::remove_file(fixture.root.join("data/empty.json")).unwrap();
    let provider = fixture.provider(vec![binding(
        id.clone(),
        "empty.json",
        &case("empty_dataset").sha256,
    )]);
    let missing = provider
        .prepare(DatasetPreparationPolicy::ContinueWithAvailableEvidence)
        .unwrap();
    assert_eq!(
        missing.observe(&id, b"").unwrap().availability(),
        DatasetAvailability::Unavailable
    );
    assert_eq!(missing.observe(&id, b"").unwrap().observed_present(), None);
}

#[test]
fn snapshots_outlive_provider_and_files_and_lookup_never_reopens_them() {
    let fixture = Fixture::new();
    let c = case("valid");
    let id = key("synthetic", "1");
    let dep = fixture.put(&c, "codes.json", id.clone());
    let provider = fixture.provider(vec![dep]);
    let a = provider.prepare(DatasetPreparationPolicy::Strict).unwrap();
    let b = provider.prepare(DatasetPreparationPolicy::Strict).unwrap();
    assert_eq!(
        a.datasets()[0].actual_sha256(),
        b.datasets()[0].actual_sha256()
    );
    drop(provider);
    fs::remove_dir_all(fixture.root.join("data")).unwrap();
    for _ in 0..1000 {
        assert_eq!(
            a.observe(&id, b"001 ").unwrap().observed_present(),
            Some(true)
        );
        assert_eq!(
            b.observe(&id, b"absent").unwrap().observed_present(),
            Some(false)
        );
    }
}

#[test]
fn original_byte_digest_changes_with_whitespace_even_when_membership_is_identical() {
    let fixture = Fixture::new();
    let id = key("synthetic", "1");
    let original = case("valid");
    let changed = case("whitespace_valid");
    let mut dep = fixture.put(&changed, "pretty.json", id.clone());
    dep.sha256 = decode(&original.sha256).try_into().unwrap();
    let prepared = fixture
        .provider(vec![dep])
        .prepare(DatasetPreparationPolicy::ContinueWithAvailableEvidence)
        .unwrap();
    let evidence = prepared.evidence(&id).unwrap();
    assert_eq!(evidence.availability(), DatasetAvailability::Degraded);
    assert_eq!(
        evidence.actual_sha256().unwrap().to_string(),
        changed.sha256
    );
    assert_eq!(evidence.expected_sha256().to_string(), original.sha256);
    assert_eq!(
        prepared.observe(&id, b"001 ").unwrap().observed_present(),
        Some(true)
    );
}

#[test]
fn invalid_bindings_unsupported_metadata_and_duplicate_keys_are_fatal_before_reads() {
    let fixture = Fixture::new();
    let id = key("synthetic", "1");
    for path in [
        "",
        "../escape.json",
        "/absolute.json",
        "C:/escape.json",
        "a\\b.json",
        "a/../b",
        "./a",
        "a//b",
        "a/",
        "NUL",
        "con.json",
        "COM1",
        "a\0b",
    ] {
        assert!(
            matches!(
                FileSystemReferenceDataProvider::new(
                    fixture.root.join("missing root"),
                    vec![binding(id.clone(), path, &case("valid").sha256)]
                ),
                Err(FilesystemReferenceError::InvalidBinding {
                    issue: ReferenceBindingIssue::UnsafeRelativePath,
                    ..
                })
            ),
            "{path:?}"
        );
    }
    let dep = binding(id.clone(), "codes.json", &case("valid").sha256);
    assert!(matches!(
        FileSystemReferenceDataProvider::new(fixture.root.clone(), vec![dep.clone(), dep.clone()]),
        Err(FilesystemReferenceError::InvalidBinding {
            issue: ReferenceBindingIssue::DuplicateIdentity,
            ..
        })
    ));
    let mut rich = dep.clone();
    rich.metadata_attributes.push("sex".into());
    assert!(matches!(
        FileSystemReferenceDataProvider::new(fixture.root.clone(), vec![rich]),
        Err(FilesystemReferenceError::InvalidBinding {
            issue: ReferenceBindingIssue::UnsupportedMetadata,
            ..
        })
    ));
    let mut invalid = dep;
    invalid.key = key(" padded", "1");
    assert!(matches!(
        FileSystemReferenceDataProvider::new(fixture.root.clone(), vec![invalid]),
        Err(FilesystemReferenceError::InvalidBinding {
            issue: ReferenceBindingIssue::InvalidIdentity,
            ..
        })
    ));
    assert!(matches!(
        FileSystemReferenceDataProvider::new(PathBuf::new(), vec![]),
        Err(FilesystemReferenceError::EmptyRoot)
    ));
}

#[test]
fn missing_root_and_empty_plan_have_explicit_different_meanings() {
    let fixture = Fixture::new();
    let id = key("synthetic", "1");
    let provider = FileSystemReferenceDataProvider::new(
        fixture.root.join("missing root"),
        vec![binding(id.clone(), "codes.json", &case("valid").sha256)],
    )
    .unwrap();
    let prepared = provider
        .prepare(DatasetPreparationPolicy::ContinueWithAvailableEvidence)
        .unwrap();
    assert!(matches!(
        prepared.datasets()[0].anomalies(),
        [DatasetAnomaly::Io {
            operation: DatasetIoOperation::ResolveRoot,
            ..
        }]
    ));
    assert_eq!(
        prepared.observe(&id, b"001 ").unwrap().observed_present(),
        None
    );
    let error = match provider.prepare(DatasetPreparationPolicy::Strict) {
        Ok(_) => panic!("strict accepted missing root"),
        Err(error) => error,
    };
    assert_eq!(
        error
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::NotFound
    );
    let empty = FileSystemReferenceDataProvider::new(fixture.root.join("missing root"), vec![])
        .unwrap()
        .prepare(DatasetPreparationPolicy::Strict)
        .unwrap();
    assert!(empty.datasets().is_empty());
}

#[test]
fn non_regular_files_and_intermediate_files_cannot_be_read_as_datasets() {
    let fixture = Fixture::new();
    let id = key("synthetic", "1");
    fs::create_dir(fixture.root.join("data/directory.json")).unwrap();
    fs::write(fixture.root.join("data/ordinary.json"), b"{}").unwrap();
    for policy in [
        DatasetPreparationPolicy::Strict,
        DatasetPreparationPolicy::ContinueWithAvailableEvidence,
    ] {
        for (path, expected) in [
            ("directory.json", DatasetFileTypeIssue::NonRegularFile),
            (
                "ordinary.json/child.json",
                DatasetFileTypeIssue::NonDirectoryComponent,
            ),
        ] {
            let provider = fixture.provider(vec![binding(id.clone(), path, &case("valid").sha256)]);
            if policy == DatasetPreparationPolicy::Strict {
                assert!(
                    matches!(provider.prepare(policy),Err(FilesystemReferenceError::StrictPreparation{evidence}) if matches!(evidence.anomalies(),[DatasetAnomaly::FileType(found)] if *found==expected))
                );
            } else {
                let prepared = provider.prepare(policy).unwrap();
                assert_eq!(
                    prepared.datasets()[0].availability(),
                    DatasetAvailability::Unavailable
                );
                assert!(
                    matches!(prepared.datasets()[0].anomalies(),[DatasetAnomaly::FileType(found)] if *found==expected)
                );
                assert_eq!(prepared.observe(&id, b"").unwrap().observed_present(), None);
            }
        }
        let provider = FileSystemReferenceDataProvider::new(
            fixture.root.join("data/ordinary.json"),
            vec![binding(id.clone(), "x.json", &case("valid").sha256)],
        )
        .unwrap();
        if policy == DatasetPreparationPolicy::Strict {
            assert!(matches!(
                provider.prepare(policy),
                Err(FilesystemReferenceError::StrictPreparation { .. })
            ));
        } else {
            let prepared = provider.prepare(policy).unwrap();
            assert_eq!(
                prepared.datasets()[0].availability(),
                DatasetAvailability::Unavailable
            );
            assert!(matches!(
                prepared.datasets()[0].anomalies(),
                [DatasetAnomaly::Io { .. }]
            ));
        }
    }
}

#[cfg(any(unix, windows))]
#[test]
fn file_and_directory_symlink_escape_is_fatal_under_both_policies() {
    let fixture = Fixture::new();
    let id = key("synthetic", "1");
    fs::create_dir(fixture.root.join("outside")).unwrap();
    fs::write(
        fixture.root.join("outside/codes.json"),
        decode(&case("valid").bytes_hex),
    )
    .unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            fixture.root.join("outside/codes.json"),
            fixture.root.join("data/link.json"),
        )
        .unwrap();
        std::os::unix::fs::symlink(
            fixture.root.join("outside"),
            fixture.root.join("data/linkdir"),
        )
        .unwrap();
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(
            fixture.root.join("outside/codes.json"),
            fixture.root.join("data/link.json"),
        )
        .unwrap();
        std::os::windows::fs::symlink_dir(
            fixture.root.join("outside"),
            fixture.root.join("data/linkdir"),
        )
        .unwrap();
    }
    for path in ["link.json", "linkdir/codes.json"] {
        for policy in [
            DatasetPreparationPolicy::Strict,
            DatasetPreparationPolicy::ContinueWithAvailableEvidence,
        ] {
            let provider = fixture.provider(vec![binding(id.clone(), path, &case("valid").sha256)]);
            assert!(matches!(
                provider.prepare(policy),
                Err(FilesystemReferenceError::Boundary {
                    violation: FilesystemBoundaryViolation::Symlink
                        | FilesystemBoundaryViolation::OutsideRoot,
                    ..
                })
            ));
        }
    }
}
