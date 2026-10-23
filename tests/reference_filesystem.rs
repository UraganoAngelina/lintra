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
fn explicit_normalization_preserves_verified_degraded_and_unavailable_evidence() {
    let fixture = Fixture::new();
    // Independent published SHA-256 of these exact bytes, checked with hashlib.
    let bytes = br#"{"dataset_format_version":1,"dataset_id":"synthetic","version":"1","keys_hex":["4142"]}"#;
    fs::write(fixture.root.join("data/codes.json"), bytes).unwrap();
    let id = key("synthetic", "1");
    let verified_binding = binding(
        id.clone(),
        "codes.json",
        "98a052b12f38e5060f8d9eb965cbe178e2abeef5dc9316e0b7d9dcb50e4bb510",
    );
    for degraded in [false, true] {
        let mut dependency = verified_binding.clone();
        if degraded {
            dependency.sha256 = [0; 32];
        }
        let prepared = fixture
            .provider(vec![dependency])
            .prepare(DatasetPreparationPolicy::ContinueWithAvailableEvidence)
            .unwrap();
        let original = prepared.evidence(&id).unwrap();
        for (policy, expected) in [
            (LookupNormalization::Exact, false),
            (LookupNormalization::AsciiUppercase, true),
        ] {
            let observation = prepared
                .observe_with_normalization(&id, b"ab", policy)
                .unwrap();
            assert_eq!(observation.observed_present(), Some(expected));
            assert!(std::ptr::eq(observation.evidence(), original));
            assert_eq!(
                observation.availability(),
                if degraded {
                    DatasetAvailability::Degraded
                } else {
                    DatasetAvailability::Verified
                }
            );
        }
        assert_eq!(
            prepared.observe(&id, b"ab").unwrap().observed_present(),
            Some(false)
        );
    }
    fs::remove_file(fixture.root.join("data/codes.json")).unwrap();
    let unavailable = fixture
        .provider(vec![verified_binding])
        .prepare(DatasetPreparationPolicy::ContinueWithAvailableEvidence)
        .unwrap();
    let observation = unavailable
        .observe_with_normalization(&id, b"ab", LookupNormalization::AsciiUppercase)
        .unwrap();
    assert_eq!(observation.availability(), DatasetAvailability::Unavailable);
    assert_eq!(observation.observed_present(), None);
    assert!(
        unavailable
            .observe_with_normalization(
                &key("other", "1"),
                b"ab",
                LookupNormalization::AsciiUppercase
            )
            .is_none()
    );
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

fn union_cases() -> Vec<Case> {
    serde_json::from_slice(include_bytes!(
        "fixtures/reference_membership/union_cases.json"
    ))
    .unwrap()
}
fn union_fixture(
    states: [DatasetAvailability; 3],
) -> (Fixture, Vec<ReferenceDependency>, PreparedReferenceEvidence) {
    let fixture = Fixture::new();
    let dependencies = union_cases()
        .iter()
        .zip(states)
        .map(|(case, state)| {
            let id = key(&format!("union-{}", case.name), "1");
            let path = format!("{}.json", case.name);
            let mut dependency = if state == DatasetAvailability::Unavailable {
                binding(id, &path, &case.sha256)
            } else {
                fixture.put(case, &path, id)
            };
            if state == DatasetAvailability::Degraded {
                dependency.sha256 = [0; 32];
            }
            dependency
        })
        .collect::<Vec<_>>();
    let prepared = fixture
        .provider(dependencies.clone())
        .prepare(DatasetPreparationPolicy::ContinueWithAvailableEvidence)
        .unwrap();
    (fixture, dependencies, prepared)
}
fn membership_name(evaluation: MembershipEvaluation<'_>) -> &'static str {
    match evaluation {
        MembershipEvaluation::VerifiedPresent(_) => "verified-present",
        MembershipEvaluation::VerifiedAbsent => "verified-absent",
        MembershipEvaluation::ProvisionalPresent(_) => "provisional-present",
        MembershipEvaluation::Inconclusive => "inconclusive",
    }
}

#[test]
fn union_uses_verified_provisional_and_inconclusive_evidence_without_false_absence() {
    use DatasetAvailability::{Degraded as D, Unavailable as U, Verified as V};
    for (states, expected) in [
        (
            [V, V, V],
            [
                "verified-present",
                "verified-present",
                "verified-present",
                "verified-absent",
            ],
        ),
        (
            [V, V, U],
            [
                "verified-present",
                "verified-present",
                "inconclusive",
                "inconclusive",
            ],
        ),
        (
            [V, V, D],
            [
                "verified-present",
                "verified-present",
                "provisional-present",
                "inconclusive",
            ],
        ),
        (
            [D, V, D],
            [
                "provisional-present",
                "verified-present",
                "provisional-present",
                "inconclusive",
            ],
        ),
        (
            [D, D, U],
            [
                "provisional-present",
                "provisional-present",
                "inconclusive",
                "inconclusive",
            ],
        ),
        (
            [U, U, U],
            [
                "inconclusive",
                "inconclusive",
                "inconclusive",
                "inconclusive",
            ],
        ),
    ] {
        let (_fixture, dependencies, prepared) = union_fixture(states);
        let keys: Vec<_> = dependencies.iter().map(|d| d.key.clone()).collect();
        for (query, expected) in [b"AA", b"BB", b"CC", b"XX"].into_iter().zip(expected) {
            let result = evaluate_membership(&keys, &prepared, query).unwrap();
            assert_eq!(membership_name(result), expected, "{states:?} {query:?}");
            match result {
                MembershipEvaluation::VerifiedPresent(evidence) => {
                    assert_eq!(evidence.availability(), V)
                }
                MembershipEvaluation::ProvisionalPresent(evidence) => {
                    assert_eq!(evidence.availability(), D)
                }
                _ => {}
            }
        }
        assert!(matches!(
            evaluate_membership(&[], &prepared, b"AA"),
            Err(MembershipPlanError::EmptyUnion)
        ));
        let mut invalid = keys;
        invalid.push(key("unprepared", "1"));
        assert!(matches!(
            evaluate_membership(&invalid, &prepared, b"AA"),
            Err(MembershipPlanError::UnpreparedDependency)
        ));
    }
}
fn reference_schema(dependencies: &[ReferenceDependency]) -> serde_json::Value {
    use serde_json::json;
    let constraint = json!({"kind":"reference_data","datasets":["A","B","C"],"fallback_values":["ZZ"],"metadata_bindings":[]});
    let mut fields = vec![];
    for (index, id) in ["CODE", "OTHER"].into_iter().enumerate() {
        fields.push(json!({"id":id,"name":id,"physical":{"type":"alphanumeric","start":index*2+1,"length":2,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":if index==0 {vec![constraint.clone()]}else{vec![json!({"kind":"allowed_values","target":"raw","values":["OK"]})]}}));
    }
    json!({"schema_format_version":1,"flow_id":"SYNTHETIC","schema_version":"v1","regulatory_version":"test","source":{},"encoding":{"mode":"runtime_required","position_unit":"byte"},"required_capabilities":["local.v1","expressions.v1"],"parameters":[],"reference_data":dependencies.iter().zip(["A","B","C"]).map(|(d,id)|json!({"id":id,"dataset_id":d.key.id.0,"version":d.key.version.0,"path":d.bundle_path.as_ref(),"sha256":Sha256Digest(d.sha256).to_string(),"metadata_attributes":[]})).collect::<Vec<_>>(),"persistence_policy":null,"traces":[{"id":"T","name":"test","record_length":4,"fields":fields,"record_key":{"fields":["CODE"],"uniqueness_scope":"trace","error_code":"E_DUPLICATE_RECORD_KEY"},"identity":[],"joins":[],"named_conditions":{},"derived_values":{},"repeated_groups":[],"rules":[],"documented_non_executable_rules":[]}]})
}
fn compile_reference_schema(
    value: serde_json::Value,
) -> Result<CompiledFlowSchema, SchemaCompilationError> {
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw)
}
#[derive(Default)]
struct EvidenceObserver {
    observations: Vec<(String, Option<String>, usize, &'static str)>,
}
impl ReferenceEvaluationObserver for EvidenceObserver {
    fn on_evaluation(
        &mut self,
        control: ReferenceControl<'_>,
        evaluation: ReferenceConstraintEvaluation<'_>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let result = match evaluation {
            ReferenceConstraintEvaluation::DeclaredFallback => "fallback",
            ReferenceConstraintEvaluation::Membership(value) => membership_name(value),
        };
        self.observations.push((
            control.field_id.0.clone(),
            control.rule_id.map(str::to_owned),
            control.index,
            result,
        ));
        Ok(())
    }
}
fn reference_metadata(bytes: &[u8]) -> RunMetadata {
    let audits = InputAuditSession::new();
    let mut reader = audits
        .wrap(
            TraceId("T".into()),
            "synthetic.txt".into(),
            bytes.len() as u64,
            std::io::Cursor::new(bytes),
        )
        .unwrap();
    std::io::copy(&mut reader, &mut std::io::sink()).unwrap();
    drop(reader);
    audits
        .finish(RunMetadata {
            validation_mode: ValidationMode::LocalOnly,
            started_at: std::time::UNIX_EPOCH,
            validator_version: "test".into(),
            schema_hash: "test".into(),
            reference_data_versions: Default::default(),
            inputs: vec![],
        })
        .unwrap()
}
#[test]
fn reference_records_continue_independent_checks_and_keep_nondefinitive_results_out_of_errors() {
    use DatasetAvailability::{Degraded as D, Unavailable as U, Verified as V};
    for (states, bytes, conclusion, codes, observation) in [
        (
            [V, V, V],
            b"AAOK",
            ReferenceConclusion::Definitive,
            vec![],
            "verified-present",
        ),
        (
            [V, V, V],
            b"XXNO",
            ReferenceConclusion::Definitive,
            vec![ValidationCode::ReferenceData, ValidationCode::InvalidDomain],
            "verified-absent",
        ),
        (
            [V, V, U],
            b"XXOK",
            ReferenceConclusion::Inconclusive,
            vec![],
            "inconclusive",
        ),
        (
            [V, V, U],
            b"AANO",
            ReferenceConclusion::Definitive,
            vec![ValidationCode::InvalidDomain],
            "verified-present",
        ),
        (
            [V, V, D],
            b"CCNO",
            ReferenceConclusion::Provisional,
            vec![ValidationCode::InvalidDomain],
            "provisional-present",
        ),
        (
            [V, V, D],
            b"XXNO",
            ReferenceConclusion::Inconclusive,
            vec![ValidationCode::InvalidDomain],
            "inconclusive",
        ),
        (
            [U, U, U],
            b"ZZOK",
            ReferenceConclusion::Definitive,
            vec![],
            "fallback",
        ),
    ] {
        let (_fixture, dependencies, prepared) = union_fixture(states);
        let flow = compile_reference_schema(reference_schema(&dependencies)).unwrap();
        assert_eq!(flow.required_reference_data().len(), 3);
        assert!(RecordValidator::new(&flow, &TraceId("T".into())).is_err());
        let mut validator =
            ReferenceRecordValidator::new(&flow, &TraceId("T".into()), &prepared).unwrap();
        let mut sink = VecSink::default();
        let mut builder = ValidationSummaryBuilder::new(
            flow.key().clone(),
            ScopeSummary::Trace {
                trace_id: TraceId("T".into()),
            },
        );
        let mut observer = EvidenceObserver::default();
        let result = validator
            .validate_local(
                RawRecord {
                    bytes,
                    line_number: 1,
                },
                &mut sink,
                &mut builder,
                &mut observer,
            )
            .unwrap();
        assert_eq!(result.reference_conclusion, conclusion);
        assert_eq!(result.normative_violations_detected, !codes.is_empty());
        assert_eq!(
            sink.errors()
                .iter()
                .map(|e| e.code.clone())
                .collect::<Vec<_>>(),
            codes
        );
        assert_eq!(
            observer.observations,
            vec![("CODE".into(), None, 0, observation)]
        );
        if conclusion == ReferenceConclusion::Definitive {
            let summary = builder.finish(reference_metadata(bytes)).unwrap();
            assert_eq!(summary.total_errors, codes.len() as u64);
        } else {
            assert!(matches!(
                builder.finish(reference_metadata(bytes)),
                Err(SummaryBuildError::UnreportedReferenceEvidence)
            ));
        }
    }
}

#[test]
fn compiler_checks_reference_bindings_resolution_fallback_and_last_stage() {
    use serde_json::json;
    let (_fixture, deps, _prepared) = union_fixture([DatasetAvailability::Verified; 3]);
    let base = reference_schema(&deps);
    for (mut value, code) in [
        (
            {
                let mut v = base.clone();
                v["traces"][0]["fields"][0]["constraints"][0]["datasets"] = json!(["ABSENT"]);
                v
            },
            SchemaIssueCode::MissingReferenceDatasetDeclaration,
        ),
        (
            {
                let mut v = base.clone();
                v["reference_data"][0]["sha256"] = json!("A".repeat(64));
                v
            },
            SchemaIssueCode::InvalidSchemaValue,
        ),
        (
            {
                let mut v = base.clone();
                v["reference_data"][0]["path"] = json!("../escape");
                v
            },
            SchemaIssueCode::InvalidSchemaValue,
        ),
        (
            {
                let mut v = base.clone();
                v["reference_data"][0]["metadata_attributes"] = json!(["age"]);
                v
            },
            SchemaIssueCode::UnsupportedConstruct,
        ),
        (
            {
                let mut v = base.clone();
                v["traces"][0]["fields"][0]["constraints"][0]["datasets"] = json!(["A", "A"]);
                v
            },
            SchemaIssueCode::InvalidSchemaValue,
        ),
        (
            {
                let mut v = base.clone();
                v["traces"][0]["fields"][0]["constraints"][0]["fallback_values"] =
                    json!(["TOO LONG"]);
                v
            },
            SchemaIssueCode::InvalidTypeParameters,
        ),
    ] {
        let result = compile_reference_schema(value.take()).unwrap_err();
        assert!(
            result.issues.iter().any(|issue| issue.code == code),
            "{result:?}"
        );
    }
    let mut wrong_order = base;
    wrong_order["traces"][0]["fields"][0]["constraints"]
        .as_array_mut()
        .unwrap()
        .push(json!({"kind":"allowed_values","target":"raw","values":["AA"]}));
    let compiled = compile_reference_schema(wrong_order).unwrap();
    let constraints = compiled.trace(&TraceId("T".into())).unwrap().fields()[0].constraints();
    assert!(matches!(
        constraints[0],
        CompiledConstraint::AllowedValues { .. }
    ));
    assert!(matches!(
        constraints[1],
        CompiledConstraint::ReferenceData { .. }
    ));
}

#[test]
fn verified_match_dominates_earlier_degraded_match_and_preserves_match_provenance() {
    use DatasetAvailability::{Degraded as D, Unavailable as U, Verified as V};
    let (fixture, mut deps, _old) = union_fixture([D, V, U]);
    let overlap = union_cases()
        .into_iter()
        .find(|c| c.name == "overlap-B")
        .unwrap();
    deps[1] = fixture.put(&overlap, "B.json", key("union-B", "1"));
    let prepared = fixture
        .provider(deps.clone())
        .prepare(DatasetPreparationPolicy::ContinueWithAvailableEvidence)
        .unwrap();
    let keys: Vec<_> = deps.iter().map(|d| d.key.clone()).collect();
    match evaluate_membership(&keys, &prepared, b"AA").unwrap() {
        MembershipEvaluation::VerifiedPresent(evidence) => assert_eq!(evidence.key(), &deps[1].key),
        other => panic!("wrong union precedence: {other:?}"),
    }
    assert_eq!(prepared.datasets()[0].availability(), D);
    assert_eq!(prepared.datasets()[2].availability(), U);
}

#[test]
fn rule_reference_unknown_continues_and_a_real_assertion_violation_remains_normative() {
    use DatasetAvailability::{Degraded as D, Verified as V};
    use serde_json::json;
    for (bytes, expected) in [(b"CCOK", "provisional-present"), (b"XXOK", "inconclusive")] {
        let (_fixture, deps, prepared) = union_fixture([V, V, D]);
        let mut schema = reference_schema(&deps);
        let constraint = schema["traces"][0]["fields"][0]["constraints"][0].clone();
        schema["traces"][0]["fields"][0]["constraints"] = json!([]);
        schema["traces"][0]["rules"] = json!([
            {"id":"MEMBERSHIP","when":{"const":true},"assertions":[{"kind":"constraint","field":{"field":"CODE"},"constraint":constraint}],"error_code":"E_CROSS_FIELD","message_template":"{rule_id}"},
            {"id":"INDEPENDENT","when":{"const":true},"assertions":[{"kind":"condition_true","condition":{"const":false}}],"error_code":"E_CROSS_FIELD","message_template":"{rule_id}"}
        ]);
        let flow = compile_reference_schema(schema.clone()).unwrap();
        let mut validator =
            ReferenceRecordValidator::new(&flow, &TraceId("T".into()), &prepared).unwrap();
        let mut sink = VecSink::default();
        let mut summary = ValidationSummaryBuilder::new(
            flow.key().clone(),
            ScopeSummary::Trace {
                trace_id: TraceId("T".into()),
            },
        );
        let mut observer = EvidenceObserver::default();
        let result = validator
            .validate_local(
                RawRecord {
                    bytes,
                    line_number: 1,
                },
                &mut sink,
                &mut summary,
                &mut observer,
            )
            .unwrap();
        assert!(result.normative_violations_detected);
        assert_eq!(sink.errors().len(), 1);
        assert_eq!(sink.errors()[0].message, "INDEPENDENT");
        assert_eq!(
            observer.observations,
            vec![("CODE".into(), Some("MEMBERSHIP".into()), 0, expected)]
        );
        // A known false assertion in the same composed rule must still win.
        schema["traces"][0]["rules"][0]["assertions"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"condition_true","condition":{"const":false}}));
        schema["traces"][0]["rules"].as_array_mut().unwrap().pop();
        let flow = compile_reference_schema(schema).unwrap();
        let mut validator =
            ReferenceRecordValidator::new(&flow, &TraceId("T".into()), &prepared).unwrap();
        let mut sink = VecSink::default();
        let mut summary = ValidationSummaryBuilder::new(
            flow.key().clone(),
            ScopeSummary::Trace {
                trace_id: TraceId("T".into()),
            },
        );
        validator
            .validate_local(
                RawRecord {
                    bytes,
                    line_number: 1,
                },
                &mut sink,
                &mut summary,
                &mut EvidenceObserver::default(),
            )
            .unwrap();
        assert_eq!(sink.errors().len(), 1);
        assert_eq!(sink.errors()[0].message, "MEMBERSHIP");
    }
}

#[test]
fn inconsistent_preparation_and_observer_failure_are_technical_and_terminal() {
    let (_fixture, deps, prepared) = union_fixture([DatasetAvailability::Verified; 3]);
    let mut schema = reference_schema(&deps);
    schema["reference_data"][0]["sha256"] = serde_json::json!("0".repeat(64));
    let wrong = compile_reference_schema(schema).unwrap();
    assert!(matches!(
        ReferenceRecordValidator::new(&wrong, &TraceId("T".into()), &prepared),
        Err(RecordValidationError::Reference(
            ReferenceExecutionError::InconsistentPreparation
        ))
    ));
    struct FailedObserver;
    impl ReferenceEvaluationObserver for FailedObserver {
        fn on_evaluation(
            &mut self,
            _: ReferenceControl<'_>,
            _: ReferenceConstraintEvaluation<'_>,
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::StorageFull,
                "synthetic observer",
            )))
        }
    }
    let flow = compile_reference_schema(reference_schema(&deps)).unwrap();
    let mut validator =
        ReferenceRecordValidator::new(&flow, &TraceId("T".into()), &prepared).unwrap();
    let mut sink = VecSink::default();
    let mut summary = ValidationSummaryBuilder::new(
        flow.key().clone(),
        ScopeSummary::Trace {
            trace_id: TraceId("T".into()),
        },
    );
    let error = validator
        .validate_local(
            RawRecord {
                bytes: b"AAOK",
                line_number: 1,
            },
            &mut sink,
            &mut summary,
            &mut FailedObserver,
        )
        .unwrap_err();
    assert!(matches!(
        error,
        RecordValidationError::Pipeline(FieldPipelineError::Reference(
            ReferenceExecutionError::Observer(_)
        ))
    ));
    assert!(sink.errors().is_empty());
    assert!(matches!(
        validator.validate_local(
            RawRecord {
                bytes: b"AAOK",
                line_number: 2
            },
            &mut sink,
            &mut summary,
            &mut EvidenceObserver::default()
        ),
        Err(RecordValidationError::PreviousFailure)
    ));
}

#[test]
fn reference_loop_uses_only_snapshot_and_preserves_independent_stage_priority() {
    use serde_json::json;
    let (fixture, deps, prepared) = union_fixture([DatasetAvailability::Verified; 3]);
    let mut schema = reference_schema(&deps);
    // The declared reference appears first, but the compiler schedules it last.
    schema["traces"][0]["fields"][0]["constraints"]
        .as_array_mut()
        .unwrap()
        .push(json!({"kind":"allowed_values","target":"raw","values":["AA"]}));
    let flow = compile_reference_schema(schema).unwrap();
    drop(fixture);
    let mut validator =
        ReferenceRecordValidator::new(&flow, &TraceId("T".into()), &prepared).unwrap();
    let mut sink = VecSink::default();
    let mut summary = ValidationSummaryBuilder::new(
        flow.key().clone(),
        ScopeSummary::Trace {
            trace_id: TraceId("T".into()),
        },
    );
    let mut observer = EvidenceObserver::default();
    for line in 1..=1000 {
        validator
            .validate_local(
                RawRecord {
                    bytes: b"AAOK",
                    line_number: line,
                },
                &mut sink,
                &mut summary,
                &mut observer,
            )
            .unwrap();
    }
    assert_eq!(observer.observations.len(), 1000);
    observer.observations.clear();
    validator
        .validate_local(
            RawRecord {
                bytes: b"XXOK",
                line_number: 1001,
            },
            &mut sink,
            &mut summary,
            &mut observer,
        )
        .unwrap();
    assert!(observer.observations.is_empty());
    assert_eq!(sink.errors().len(), 1);
    assert_eq!(sink.errors()[0].code, ValidationCode::InvalidDomain);
}

#[test]
fn bare_reference_constraint_rejects_invalid_plans_before_a_matching_fallback() {
    let (_fixture, deps, prepared) = union_fixture([DatasetAvailability::Verified; 3]);
    let field = FieldSlice { raw: b"ZZ" };
    for indexes in [vec![], vec![usize::MAX]] {
        let constraint = CompiledConstraint::ReferenceData {
            dataset_indexes: indexes,
            fallback_values: vec![b"ZZ".to_vec()],
            metadata_bindings: vec![],
        };
        assert!(matches!(
            validate_reference_constraint(&constraint, field, &deps, &prepared),
            Err(ReferenceEvaluationError::InvalidPlan)
        ));
    }
    assert!(matches!(
        validate_reference_constraint(&CompiledConstraint::NonEmpty, field, &deps, &prepared),
        Err(ReferenceEvaluationError::InvalidPlan)
    ));
}
