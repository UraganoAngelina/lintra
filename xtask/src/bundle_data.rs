//! Deterministic data packaging. Integrity checks do not certify normative content.
use crate::{Failure, invalid};
use lintra::BundleManifest;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use zip::{ZipWriter, write::SimpleFileOptions};

#[derive(Debug, PartialEq, Eq)]
struct Options {
    input: PathBuf,
    output_dir: PathBuf,
}
fn options(args: impl IntoIterator<Item = String>) -> Result<Options, Failure> {
    let mut args = args.into_iter();
    if args.next().as_deref() != Some("bundle-data") {
        return Err(invalid("expected bundle-data --input ROOT --output-dir DIR").into());
    }
    let (mut input, mut output_dir) = (None, None);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" if input.is_none() => {
                input = Some(
                    args.next()
                        .ok_or_else(|| invalid("missing input root"))?
                        .into(),
                )
            }
            "--output-dir" if output_dir.is_none() => {
                output_dir = Some(
                    args.next()
                        .ok_or_else(|| invalid("missing output directory"))?
                        .into(),
                )
            }
            _ => return Err(invalid("unknown or duplicate bundle option").into()),
        }
    }
    Ok(Options {
        input: input.ok_or_else(|| invalid("explicit input root required"))?,
        output_dir: output_dir.ok_or_else(|| invalid("explicit output directory required"))?,
    })
}
fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Manifest paths have already passed the shared portable-path validator. Inspect
/// every component to reject symlinks and nonregular payloads. No arbitrary scan;
/// no promise of resistance to hostile concurrent filesystem replacements.
fn file(root: &Path, relative: &str) -> Result<Vec<u8>, Failure> {
    let mut path = root.to_owned();
    let parts = relative.split('/').collect::<Vec<_>>();
    for (index, part) in parts.iter().enumerate() {
        path.push(part);
        let metadata = fs::symlink_metadata(&path)?;
        let correct_type = if index + 1 == parts.len() {
            metadata.is_file()
        } else {
            metadata.is_dir()
        };
        if metadata.file_type().is_symlink() || !correct_type {
            return Err(invalid(
                "bundle input must contain regular files and directories without symlinks",
            )
            .into());
        }
    }
    Ok(fs::read(path)?)
}
fn directory(path: &Path) -> Result<PathBuf, Failure> {
    if path.as_os_str().is_empty() {
        return Err(invalid("explicit nonempty directory required").into());
    }
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(invalid("expected a directory, not a symlink").into());
    }
    Ok(fs::canonicalize(path)?)
}

/// Uses a caller-supplied manifest, preserving its exact bytes and version. Files
/// are read once, checked, then written; only one payload is buffered at a time.
/// Fixed metadata, Stored compression and ordered entries make identical inputs
/// yield identical archive bytes. RAII owns unpublished staging;
/// persist_noclobber publishes only a finished ZIP and refuses existing targets.
/// As documented by tempfile, crash durability and universal atomic cleanup of
/// the old temporary name are not guaranteed on every filesystem.
fn build(input: &Path, output_dir: &Path) -> Result<PathBuf, Failure> {
    let root = directory(input)?;
    let output_dir = directory(output_dir)?;
    if output_dir.starts_with(&root) {
        return Err(invalid("output directory must be outside the input bundle").into());
    }
    for name in ["schemas", "reference-data"] {
        directory(&root.join(name))?;
    }
    let manifest_bytes = file(&root, "MANIFEST.json")?;
    let manifest = BundleManifest::parse(&manifest_bytes)?;
    let version = manifest.bundle_version();
    // This is archive filename safety, not a new schema/version grammar.
    if version.chars().any(|c| {
        c.is_control() || matches!(c, '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*')
    }) {
        return Err(invalid("bundle version cannot be used in a portable archive filename").into());
    }
    let destination = output_dir.join(format!("schemas-{version}.zip"));
    match fs::symlink_metadata(&destination) {
        Ok(_) => {
            return Err(invalid(
                "bundle archive already exists; choose a new destination or version",
            )
            .into());
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    let mut temporary = tempfile::NamedTempFile::new_in(&output_dir)?;
    let mut writer = ZipWriter::new(temporary.as_file_mut());
    let fixed_time = zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0)?;
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .last_modified_time(fixed_time)
        .unix_permissions(0o644);
    for name in ["reference-data/", "schemas/"] {
        writer.add_directory(name, options.unix_permissions(0o755))?;
    }
    let mut checksums = BTreeMap::new();
    checksums.insert("MANIFEST.json".to_owned(), sha256(&manifest_bytes));
    for (path, digest) in manifest.files() {
        let bytes = file(&root, path)?;
        manifest.verify_bytes(path, &bytes)?;
        writer.start_file(path, options)?;
        writer.write_all(&bytes)?;
        checksums.insert(path.clone(), digest.clone());
    }
    writer.start_file("MANIFEST.json", options)?;
    writer.write_all(&manifest_bytes)?;
    writer.start_file("SHA256SUMS", options)?;
    for (path, digest) in checksums {
        writeln!(writer, "{digest}  {path}")?;
    }
    writer.finish()?.sync_all()?;
    temporary.persist_noclobber(&destination)?;
    Ok(destination)
}

pub(super) fn cli(args: Vec<String>) -> Result<(), Failure> {
    let args = options(args)?;
    let archive = build(&args.input, &args.output_dir)?;
    writeln!(
        std::io::stdout().lock(),
        "bundle_archive={}",
        archive.display()
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::{io::Read, process::Command};
    fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let input = temp.path().join("input bundle");
        let output = temp.path().join("output bundle");
        fs::create_dir_all(input.join("schemas/SYNTHETIC/v1")).unwrap();
        fs::create_dir(input.join("reference-data")).unwrap();
        fs::create_dir(&output).unwrap();
        fs::write(
            input.join("schemas/SYNTHETIC/v1/flow.json"),
            include_bytes!("../../bundles/embedded/schemas/SYNTHETIC/v1/flow.json"),
        )
        .unwrap();
        fs::write(
            input.join("reference-data/example-codes.json"),
            include_bytes!("../../bundles/embedded/reference-data/example-codes.json"),
        )
        .unwrap();
        fs::write(
            input.join("MANIFEST.json"),
            include_bytes!("../../bundles/embedded/MANIFEST.json"),
        )
        .unwrap();
        (temp, input, output)
    }
    fn manifest(input: &Path, edit: impl FnOnce(&mut Value)) {
        let path = input.join("MANIFEST.json");
        let mut value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        edit(&mut value);
        fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    }
    #[test]
    fn deterministic_zip_has_exact_inventory_and_verified_checksums() {
        let (temp, input, output) = fixture();
        fs::write(input.join("private-unlisted-file"), b"not packaged").unwrap();
        let second = temp.path().join("second");
        fs::create_dir(&second).unwrap();
        let first = build(&input, &output).unwrap();
        let other = build(&input, &second).unwrap();
        assert_eq!(fs::read(&first).unwrap(), fs::read(other).unwrap());
        let mut archive = zip::ZipArchive::new(fs::File::open(first).unwrap()).unwrap();
        let names = archive
            .file_names()
            .map(|name| name.unwrap().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec![
                "reference-data/",
                "schemas/",
                "reference-data/example-codes.json",
                "schemas/SYNTHETIC/v1/flow.json",
                "MANIFEST.json",
                "SHA256SUMS"
            ]
        );
        let mut sums = String::new();
        archive
            .by_name("SHA256SUMS")
            .unwrap()
            .read_to_string(&mut sums)
            .unwrap();
        assert_eq!(sums.lines().count(), 3);
        for line in sums.lines() {
            let (digest, path) = line.split_once("  ").unwrap();
            let mut bytes = Vec::new();
            archive
                .by_name(path)
                .unwrap()
                .read_to_end(&mut bytes)
                .unwrap();
            assert_eq!(sha256(&bytes), digest);
        }
        let mut bytes = Vec::new();
        archive
            .by_name("MANIFEST.json")
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(bytes, fs::read(input.join("MANIFEST.json")).unwrap());
    }
    #[test]
    fn corrupt_missing_unsafe_and_ambiguous_inputs_leave_no_archive_or_staging() {
        for mode in 0..8 {
            let (_temp, input, output) = fixture();
            match mode {
                0 => fs::write(input.join("schemas/SYNTHETIC/v1/flow.json"), b"{}").unwrap(),
                1 => fs::remove_file(input.join("reference-data/example-codes.json")).unwrap(),
                2 => manifest(&input, |m| {
                    m["files"]["schemas/../escape"] = json!("0".repeat(64))
                }),
                3 => manifest(&input, |m| m["bundle_version"] = json!("../escape")),
                4 => fs::write(
                    input.join("MANIFEST.json"),
                    br#"{"bundle_version":"a","bundle_version":"b"}"#,
                )
                .unwrap(),
                5 => fs::remove_dir_all(input.join("reference-data")).unwrap(),
                6 => fs::write(input.join("MANIFEST.json"), b"{").unwrap(),
                7 => manifest(&input, |m| m["schema_format_version"] = json!(999)),
                _ => unreachable!(),
            }
            assert!(build(&input, &output).is_err(), "mode {mode}");
            assert_eq!(fs::read_dir(&output).unwrap().count(), 0, "mode {mode}");
        }
    }
    #[test]
    fn existing_target_and_input_output_collision_preserve_inputs() {
        let (_temp, input, output) = fixture();
        let path = build(&input, &output).unwrap();
        let before = fs::read(&path).unwrap();
        assert!(build(&input, &output).is_err());
        assert_eq!(fs::read(path).unwrap(), before);
        assert!(build(&input, &input).is_err());
        assert_eq!(fs::read_dir(output).unwrap().count(), 1);
    }
    #[cfg(unix)]
    #[test]
    fn symlink_file_and_directory_are_not_packaged() {
        use std::os::unix::fs::symlink;
        for relative in [
            "MANIFEST.json",
            "reference-data/example-codes.json",
            "schemas/SYNTHETIC",
        ] {
            let (temp, input, output) = fixture();
            let path = input.join(relative);
            let moved = temp.path().join("moved");
            fs::rename(&path, &moved).unwrap();
            symlink(&moved, &path).unwrap();
            assert!(build(&input, &output).is_err());
            assert_eq!(fs::read_dir(output).unwrap().count(), 0);
        }
    }
    #[test]
    fn bundle_cli_requires_explicit_unique_options() {
        for args in [
            vec!["bundle-data"],
            vec!["bundle-data", "--input", "a"],
            vec![
                "bundle-data",
                "--input",
                "a",
                "--input",
                "b",
                "--output-dir",
                "c",
            ],
            vec![
                "bundle-data",
                "--input",
                "a",
                "--output-dir",
                "b",
                "--unknown",
            ],
        ] {
            assert!(options(args.into_iter().map(str::to_owned)).is_err());
        }
    }
    /// The normal tooling suite keeps this requirement explicit instead of
    /// silently treating a missing binary as a successful acceptance test.
    #[test]
    #[ignore = "requires LINTRA_TEST_BIN pointing to an already built native executable"]
    fn extracted_bundle_is_accepted_by_existing_executable() {
        let (_temp, input, output) = fixture();
        manifest(&input, |m| {
            m["bundle_version"] = json!("synthetic-package-v2")
        });
        let path = build(&input, &output).unwrap();
        let unpacked = output.join("estratto con spazi");
        fs::create_dir(&unpacked).unwrap();
        zip::ZipArchive::new(fs::File::open(path).unwrap())
            .unwrap()
            .extract(&unpacked)
            .unwrap();
        let work = output.join("cwd differente");
        fs::create_dir(&work).unwrap();
        fs::write(work.join("input.txt"), b"001A  \n").unwrap();
        let binary = std::env::var_os("LINTRA_TEST_BIN")
            .expect("acceptance test needs an explicit existing binary");
        let binary = fs::canonicalize(binary).unwrap();
        let before = sha256(&fs::read(&binary).unwrap());
        let result = Command::new(&binary)
            .current_dir(&work)
            .env_remove("LINTRA_SCHEMA_DIR")
            .env_remove("LINTRA_REFERENCE_DATA_DIR")
            .args([
                "validate",
                "--flow",
                "SYNTHETIC",
                "--schema-version",
                "v1",
                "--trace",
                "T1",
                "--encoding",
                "utf8",
                "--input",
                "input.txt",
                "--errors",
                "errors.jsonl",
                "--summary",
                "summary.json",
            ])
            .arg("--schema-dir")
            .arg(unpacked.join("schemas"))
            .arg("--reference-data-dir")
            .arg(unpacked.join("reference-data"))
            .output()
            .unwrap();
        assert_eq!(
            result.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let summary: Value =
            serde_json::from_slice(&fs::read(work.join("summary.json")).unwrap()).unwrap();
        assert_eq!(summary["bundle_source"], "external");
        assert_eq!(summary["bundle_version"], "synthetic-package-v2");
        assert_eq!(summary["bundle_manifest_status"], "present");
        assert_eq!(fs::read(work.join("errors.jsonl")).unwrap(), b"");
        assert_eq!(sha256(&fs::read(binary).unwrap()), before);
    }
}
