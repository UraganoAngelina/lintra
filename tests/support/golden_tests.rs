//! Test the harness against disposable files, never mutate committed goldens.
use super::*;
use std::{
    cell::Cell,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// An exclusively created directory isolates each test and allows paths with
/// spaces/Unicode. Retrying collisions handles leftovers without deleting them.
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        loop {
            let path = std::env::temp_dir().join(format!(
                "lintra golden à {} {}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("cannot create synthetic fixture: {error}"),
            }
        }
    }
    /// Create a scenario with input only; expected files are optional so tests
    /// can prove Check refuses to invent them and Update can explicitly add them.
    fn scenario(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("input.txt"), b"synthetic input\r\n").unwrap();
        path
    }
}
impl Drop for Fixture {
    /// Only this exclusive test directory is removed after every file is closed.
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

/// None is safe read-only mode; only 1 authorizes updates. Direct parsing avoids
/// modifying the global environment while Rust runs tests in parallel.
#[test]
fn update_requires_exact_explicit_value() {
    assert_eq!(GoldenMode::from_value(None).unwrap(), GoldenMode::Check);
    assert_eq!(
        GoldenMode::from_value(Some(OsStr::new("1"))).unwrap(),
        GoldenMode::Update
    );
    for value in ["", "0", "true", "yes", " 1", "1 "] {
        assert_eq!(
            GoldenMode::from_value(Some(OsStr::new(value)))
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
    }
}

/// read_dir order is not stable; deliberately create z before a and require
/// sorted recursive traversal. Unrelated regular files are harmless metadata.
#[test]
fn discovery_recurses_in_sorted_order() {
    let fixture = Fixture::new();
    let z = fixture.scenario("nested/z");
    let a = fixture.scenario("nested/a");
    fs::write(fixture.0.join("README.md"), b"metadata").unwrap();
    assert_eq!(discover(&fixture.0).unwrap(), vec![a, z]);
}

/// An empty tree and orphan expected outputs are invalid suites: either could
/// otherwise turn a missing input into a silently passing regression run.
#[test]
fn empty_suite_and_orphan_outputs_are_rejected() {
    let fixture = Fixture::new();
    assert_eq!(
        discover(&fixture.0).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    fs::write(fixture.0.join("expected.errors.jsonl"), b"orphan").unwrap();
    assert_eq!(
        discover(&fixture.0).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}

/// Every mutation matters, even semantically equivalent JSON. Reading the
/// expected file again after each error proves comparison never heals a failure.
#[test]
fn comparison_rejects_bom_crlf_whitespace_reordering_and_prefix_changes() {
    let fixture = Fixture::new();
    let path = fixture.0.join("expected.errors.jsonl");
    let expected = b"{\"a\":1,\"b\":2}\n";
    fs::write(&path, expected).unwrap();
    compare_or_update(&path, expected, GoldenMode::Check).unwrap();
    for actual in [
        &b"\xef\xbb\xbf{\"a\":1,\"b\":2}\n"[..],
        &b"{\"a\":1,\"b\":2}\r\n"[..],
        &b"{\"a\": 1,\"b\":2}\n"[..],
        &b"{\"b\":2,\"a\":1}\n"[..],
        &b"{\"a\":1,\"b\":2}"[..],
        &b"{\"a\":1,\"b\":2}\n\n"[..],
    ] {
        let error = compare_or_update(&path, actual, GoldenMode::Check).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("at byte "));
        assert_eq!(fs::read(&path).unwrap(), expected);
    }
}

/// Check must fail on a missing expected document, leaving it missing.
#[test]
fn check_never_creates_missing_expected_files() {
    let fixture = Fixture::new();
    let path = fixture.0.join("expected.summary.json");
    let error = compare_or_update(&path, b"{}\n", GoldenMode::Check).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    assert!(!path.exists());
}

/// The mode derived from the exact opt-in writes both raw outputs. Then Check
/// succeeds with the same bytes, while original CRLF input remains unchanged.
#[test]
fn explicit_update_creates_both_documents_without_normalizing_bytes() {
    let fixture = Fixture::new();
    let scenario = fixture.scenario("case");
    let mode = GoldenMode::from_value(Some(OsStr::new("1"))).unwrap();
    let produce = |_: &Path| {
        Ok(GoldenOutput {
            errors: b"raw\r\n".to_vec(),
            summary: b"{}\n".to_vec(),
        })
    };
    assert_eq!(run(&fixture.0, mode, produce).unwrap(), 1);
    assert_eq!(run(&fixture.0, GoldenMode::Check, produce).unwrap(), 1);
    assert_eq!(
        fs::read(scenario.join("expected.errors.jsonl")).unwrap(),
        b"raw\r\n"
    );
    assert_eq!(
        fs::read(scenario.join("input.txt")).unwrap(),
        b"synthetic input\r\n"
    );
    // Exercise replacement as well as creation: Update is deliberate, not a
    // create-only path or a side effect of failing comparison.
    compare_or_update(&scenario.join("expected.summary.json"), b"new", mode).unwrap();
    assert_eq!(
        fs::read(scenario.join("expected.summary.json")).unwrap(),
        b"new"
    );
}

/// Even explicit Update cannot write when the producer failed. A second
/// scenario is not run after the first technical failure.
#[test]
fn producer_failure_preserves_existing_goldens_and_stops_execution() {
    let fixture = Fixture::new();
    let a = fixture.scenario("a");
    fixture.scenario("b");
    fs::write(a.join("expected.errors.jsonl"), b"old errors").unwrap();
    fs::write(a.join("expected.summary.json"), b"old summary").unwrap();
    let calls = Cell::new(0);
    let result = run(&fixture.0, GoldenMode::Update, |_| {
        calls.set(calls.get() + 1);
        Err(io::Error::other("synthetic producer failure").into())
    });
    assert!(result.is_err());
    assert_eq!(calls.get(), 1);
    assert_eq!(
        fs::read(a.join("expected.errors.jsonl")).unwrap(),
        b"old errors"
    );
    assert_eq!(
        fs::read(a.join("expected.summary.json")).unwrap(),
        b"old summary"
    );
}

/// Directories are never rewritten as expected outputs, including Update.
#[test]
fn expected_path_must_be_a_regular_file() {
    let fixture = Fixture::new();
    let path = fixture.0.join("expected.summary.json");
    fs::create_dir(&path).unwrap();
    for mode in [GoldenMode::Check, GoldenMode::Update] {
        assert_eq!(
            compare_or_update(&path, b"{}", mode).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert!(path.is_dir());
    }
}

/// A single mismatching summary fails the scenario even when the error stream
/// matches. This proves both required documents participate in the assertion.
#[test]
fn summary_mismatch_is_checked_without_modifying_either_document() {
    let fixture = Fixture::new();
    let scenario = fixture.scenario("case");
    fs::write(scenario.join("expected.errors.jsonl"), b"").unwrap();
    fs::write(scenario.join("expected.summary.json"), b"old").unwrap();
    let result = run(&fixture.0, GoldenMode::Check, |_| {
        Ok(GoldenOutput {
            errors: vec![],
            summary: b"different".to_vec(),
        })
    });
    assert!(result.is_err());
    assert_eq!(
        fs::read(scenario.join("expected.summary.json")).unwrap(),
        b"old"
    );
    assert!(
        fs::read(scenario.join("expected.errors.jsonl"))
            .unwrap()
            .is_empty()
    );
}
