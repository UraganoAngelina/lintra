//! T-P5-07: execute committed synthetic scenarios and compare exact bytes.
mod support;

use std::{io, path::Path};
use support::{
    golden::{self, GoldenMode, HarnessResult, UPDATE_VARIABLE},
    synthetic,
};

/// The only environment read is at the test boundary. Helpers receive a mode
/// explicitly, avoiding environment races across tests. CARGO_MANIFEST_DIR is
/// a test-only fixture locator; production gains no build-machine path.
#[test]
fn committed_scenarios_match_byte_for_byte() -> HarnessResult<()> {
    let mode = GoldenMode::from_value(std::env::var_os(UPDATE_VARIABLE).as_deref())?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let profiles = ["synthetic/v1", "semantic/v1", "dsl/v1", "derived/v1"];
    let schemas = profiles
        .iter()
        .map(|profile| {
            let directory = root.join(profile);
            let (schema, hash) = synthetic::load_schema(&directory.join("schema.json"))?;
            Ok((directory, schema, hash))
        })
        .collect::<HarnessResult<Vec<_>>>()?;
    let mut registered = schemas
        .iter()
        .map(|(directory, _, _)| directory.clone())
        .collect::<Vec<_>>();
    // Multi-input normative scenarios have a dedicated service/oracle runner
    // in sioc_referential.rs. Never let the single-reader producer update them.
    registered.push(root.join("cross-trace/sioc-referential/v1"));
    registered_roots(&root, &registered)?;
    let mut count = 0;
    for (directory, schema, hash) in &schemas {
        count += golden::run(directory, mode, |scenario| {
            synthetic::produce(scenario, schema, hash)
        })?;
    }
    assert!(
        count > 0,
        "an empty suite must never be a passing golden run"
    );
    Ok(())
}

/// Reject unknown families/versions instead of silently dropping their tests.
/// Once an exact producer root is reached, its own scenario discovery applies.
fn registered_roots(directory: &Path, registered: &[std::path::PathBuf]) -> io::Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_symlink() || (!kind.is_file() && !kind.is_dir()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "golden roots require regular entries",
            ));
        }
        if kind.is_file()
            && matches!(
                entry.file_name().to_str(),
                Some("input.txt" | "expected.errors.jsonl" | "expected.summary.json")
            )
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "scenario outside a registered golden producer root",
            ));
        }
        if kind.is_dir() {
            let path = entry.path();
            if !registered.iter().any(|root| root.starts_with(&path)) {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "no golden producer registered for this profile",
                ));
            }
            if !registered.contains(&path) {
                registered_roots(&path, registered)?;
            }
        }
    }
    Ok(())
}

/// Reopening the same input with fresh validator/audit instances must reproduce
/// every byte. No result masking occurs after either report is serialized.
#[test]
fn semantic_reports_are_identical_across_independent_runs() -> HarnessResult<()> {
    let profile = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/semantic/v1");
    let (schema, hash) = synthetic::load_schema(&profile.join("schema.json"))?;
    let first = synthetic::produce(&profile.join("invalid"), &schema, &hash)?;
    let second = synthetic::produce(&profile.join("invalid"), &schema, &hash)?;
    assert_eq!(first.errors, second.errors);
    assert_eq!(first.summary, second.summary);
    Ok(())
}

#[test]
fn dsl_reports_are_identical_across_independent_runs() -> HarnessResult<()> {
    let profile = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/dsl/v1");
    let (schema, hash) = synthetic::load_schema(&profile.join("schema.json"))?;
    let first = synthetic::produce(&profile.join("invalid"), &schema, &hash)?;
    let second = synthetic::produce(&profile.join("invalid"), &schema, &hash)?;
    assert_eq!(first.errors, second.errors);
    assert_eq!(first.summary, second.summary);
    Ok(())
}

#[test]
fn derived_reports_are_identical_across_independent_runs() -> HarnessResult<()> {
    let profile = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/derived/v1");
    let (schema, hash) = synthetic::load_schema(&profile.join("schema.json"))?;
    let first = synthetic::produce(&profile.join("invalid"), &schema, &hash)?;
    let second = synthetic::produce(&profile.join("invalid"), &schema, &hash)?;
    assert_eq!(first.errors, second.errors);
    assert_eq!(first.summary, second.summary);
    Ok(())
}
