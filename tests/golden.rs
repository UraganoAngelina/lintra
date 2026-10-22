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
    let profiles = ["synthetic/v1", "semantic/v1"];
    let schemas = profiles
        .iter()
        .map(|profile| {
            let directory = root.join(profile);
            let (schema, hash) = synthetic::load_schema(&directory.join("schema.json"))?;
            Ok((directory, schema, hash))
        })
        .collect::<HarnessResult<Vec<_>>>()?;
    let count = golden::run(&root, mode, |directory| {
        // A profile must be registered explicitly; an unknown future profile
        // cannot inherit a schema or silently fall back to length-only checks.
        let Some((_, schema, hash)) = schemas
            .iter()
            .find(|(profile, _, _)| directory.starts_with(profile))
        else {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "no golden producer registered for this profile",
            )
            .into());
        };
        synthetic::produce(directory, schema, hash)
    })?;
    assert!(
        count > 0,
        "an empty suite must never be a passing golden run"
    );
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
