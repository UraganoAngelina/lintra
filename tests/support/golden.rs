//! Byte-exact golden discovery/comparison, independent of the producer.
use std::{
    error::Error,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

pub const UPDATE_VARIABLE: &str = "LINTRA_UPDATE_GOLDEN";
pub type HarnessResult<T> = Result<T, Box<dyn Error>>;

/// Check is read-only. Update is an explicit capability passed once to the
/// runner, so helper tests need no unsafe process-environment mutation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoldenMode {
    Check,
    Update,
}
impl GoldenMode {
    /// Only the exact native string "1" enables writes. Unset means Check;
    /// misspellings, empty values and non-UTF-8 values fail rather than update.
    pub fn from_value(value: Option<&OsStr>) -> io::Result<Self> {
        match value {
            None => Ok(Self::Check),
            Some(value) if value == OsStr::new("1") => Ok(Self::Update),
            Some(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{UPDATE_VARIABLE} must be unset or exactly 1"),
            )),
        }
    }
}

/// These are the encoder's actual bytes. The harness never parses/reformats
/// them: BOM, CR, whitespace, ordering and final newline remain observable.
pub struct GoldenOutput {
    pub errors: Vec<u8>,
    pub summary: Vec<u8>,
}

/// Recursively find scenario directories in sorted order. Every scenario needs
/// input.txt; orphan expected files are errors, not silently skipped fixtures.
/// Reject links before traversal/writes to avoid cycles and out-of-tree targets.
pub fn discover(root: &Path) -> io::Result<Vec<PathBuf>> {
    if !fs::symlink_metadata(root)?.file_type().is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "golden root must be a directory",
        ));
    }
    let mut scenarios = Vec::new();
    visit(root, &mut scenarios)?;
    if scenarios.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "no golden scenarios found",
        ));
    }
    Ok(scenarios)
}

/// Validate regular files while collecting children, then recurse in sorted
/// native path order. The committed scenario names are ASCII on both OSes.
fn visit(directory: &Path, scenarios: &mut Vec<PathBuf>) -> io::Result<()> {
    let mut children = Vec::new();
    let mut input = false;
    let mut expected = false;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "golden symlinks are not supported",
            ));
        }
        if kind.is_dir() {
            children.push(entry.path());
        } else if kind.is_file() {
            let name = entry.file_name();
            input |= name == "input.txt";
            expected |= name == "expected.errors.jsonl" || name == "expected.summary.json";
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "golden entries must be regular files or directories",
            ));
        }
    }
    if expected && !input {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("orphan golden output in {}", directory.display()),
        ));
    }
    if input {
        scenarios.push(directory.to_owned());
    }
    children.sort();
    for child in children {
        visit(&child, scenarios)?;
    }
    Ok(())
}

/// Run the supplied adapter for each discovered input. Producer failures abort
/// immediately; expected files are touched only after both outputs exist.
pub fn run<F>(root: &Path, mode: GoldenMode, mut produce: F) -> HarnessResult<usize>
where
    F: FnMut(&Path) -> HarnessResult<GoldenOutput>,
{
    let scenarios = discover(root)?;
    for scenario in &scenarios {
        let output = produce(scenario)?;
        compare_or_update(
            &scenario.join("expected.errors.jsonl"),
            &output.errors,
            mode,
        )?;
        compare_or_update(
            &scenario.join("expected.summary.json"),
            &output.summary,
            mode,
        )?;
    }
    Ok(scenarios.len())
}

/// Check compares raw bytes and reports only the first differing offset and
/// lengths, never fixture contents. Missing expected files fail in Check mode.
/// Update deliberately writes verbatim bytes; it is never a mismatch fallback.
pub fn compare_or_update(path: &Path, actual: &[u8], mode: GoldenMode) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.file_type().is_file() => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "golden expected output must be a regular file",
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    if mode == GoldenMode::Update {
        return fs::write(path, actual);
    }
    let expected = fs::read(path)?;
    if expected == actual {
        return Ok(());
    }
    let offset = expected
        .iter()
        .zip(actual)
        .position(|(a, b)| a != b)
        .unwrap_or(expected.len().min(actual.len()));
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "golden mismatch {} at byte {offset}: expected {} bytes, actual {} bytes; review before {UPDATE_VARIABLE}=1",
            path.display(),
            expected.len(),
            actual.len()
        ),
    ))
}

#[cfg(test)]
#[path = "golden_tests.rs"]
mod tests;
