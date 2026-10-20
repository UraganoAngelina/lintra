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
    let profile = root.join("synthetic/v1");
    let (schema, hash) = synthetic::load_schema(&profile.join("schema.json"))?;
    let count = golden::run(&root, mode, |directory| {
        // Discovery covers the whole tree, but this adapter understands only
        // its declared synthetic profile. Future flows must supply an adapter;
        // they must not accidentally inherit this six-byte length-only oracle.
        if !directory.starts_with(&profile) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "no golden producer registered for this profile",
            )
            .into());
        }
        synthetic::produce(directory, &schema, &hash)
    })?;
    assert!(
        count > 0,
        "an empty suite must never be a passing golden run"
    );
    Ok(())
}
