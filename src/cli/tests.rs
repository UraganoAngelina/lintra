//! No global environment/cwd mutation; native paths and real directory metadata.
use super::{args, exit_code, paths::*};
use lintra::*;
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
static NEXT: AtomicU64 = AtomicU64::new(0);

/// A real environment port whose values belong exclusively to one test.
#[derive(Default)]
struct FakeEnv(BTreeMap<&'static str, OsString>);
impl EnvProvider for FakeEnv {
    fn var_os(&self, key: &str) -> Option<OsString> {
        self.0.get(key).cloned()
    }
}
/// Exclusive temporary bundle tree, independent of the current working directory.
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lintra CLI à {} {} {}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    /// Create the complete pair; empty datasets are allowed because this is only
    /// path resolution, never a claim that any requested dataset exists.
    fn pair(&self, name: &str) -> CliDirs {
        let root = self.0.join(name);
        let schema = root.join("schemas");
        let reference = root.join("reference-data");
        fs::create_dir_all(&schema).unwrap();
        fs::create_dir(&reference).unwrap();
        CliDirs {
            schema_dir: Some(schema),
            reference_data_dir: Some(reference),
        }
    }
    /// Construct an explicit executable path without requiring an actual binary.
    fn exe(&self, name: &str) -> PathBuf {
        self.0.join(name).join("lintra.exe")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
/// All supported flags in one syntactically valid request; no file I/O follows.
fn argv() -> Vec<OsString> {
    [
        "lintra",
        "validate",
        "--flow",
        "SYNTHETIC",
        "--schema-version",
        "v1",
        "--trace",
        "T1",
        "--input",
        "input with spaces.txt",
        "--errors",
        "errors.jsonl",
        "--summary",
        "summary.json",
        "--encoding",
        "utf8",
    ]
    .into_iter()
    .map(OsString::from)
    .collect()
}
/// Assert both roots, rather than accepting a resolver that silently mixes pairs.
fn assert_pair(result: BundleSource, dirs: &CliDirs) {
    assert_eq!(
        result,
        BundleSource::External {
            schema_root: dirs.schema_dir.clone().unwrap(),
            reference_data_root: dirs.reference_data_dir.clone().unwrap()
        }
    );
}
/// Populate only the injected environment; production env remains untouched.
fn environment(dirs: &CliDirs) -> FakeEnv {
    FakeEnv(BTreeMap::from([
        (
            "LINTRA_SCHEMA_DIR",
            dirs.schema_dir.as_ref().unwrap().as_os_str().to_owned(),
        ),
        (
            "LINTRA_REFERENCE_DATA_DIR",
            dirs.reference_data_dir
                .as_ref()
                .unwrap()
                .as_os_str()
                .to_owned(),
        ),
    ]))
}
#[test]
fn parsed_arguments_map_exactly_to_local_request_and_native_outputs() {
    let parsed = parse_validate(argv()).unwrap();
    assert_eq!(parsed.request.mode, ValidationMode::LocalOnly);
    assert_eq!(
        parsed.request.schema_key.flow_id,
        FlowId("SYNTHETIC".into())
    );
    assert_eq!(
        parsed.request.schema_key.version,
        SchemaVersion("v1".into())
    );
    let ValidationScope::Trace(trace) = parsed.request.scope else {
        panic!("Trace required")
    };
    assert_eq!(trace.trace_id, TraceId("T1".into()));
    assert_eq!(
        trace.source,
        InputSource::Path("input with spaces.txt".into())
    );
    assert_eq!(parsed.encoding, RawResolvedEncoding::Utf8);
    assert_eq!(parsed.errors, Some(PathBuf::from("errors.jsonl")));
    assert_eq!(parsed.summary, PathBuf::from("summary.json"));
    assert_eq!(
        parsed.request.options.newline_policy,
        NewlinePolicy::AutoLfOrCrlf
    );
    assert_eq!(
        parsed.request.options.record_length_policy,
        RecordLengthPolicy::StrictSkipRecord
    );
    assert_eq!(parsed.request.options.max_errors, None);
    assert!(parsed.dirs.schema_dir.is_none() && parsed.dirs.reference_data_dir.is_none());
}
#[test]
fn syntax_errors_are_technical_and_help_is_successful() {
    assert_eq!(
        exit_code::from_parse_error(&parse_validate(["lintra", "--help"]).err().unwrap()),
        0
    );
    assert_eq!(
        exit_code::from_parse_error(&parse_validate(["lintra"]).err().unwrap()),
        1
    );
    for extra in [["--bom", "strip"], ["--mode", "full"]] {
        let mut arguments = argv();
        arguments.extend(extra.map(OsString::from));
        assert_eq!(
            exit_code::from_parse_error(&parse_validate(arguments).err().unwrap()),
            1
        );
    }
    let mut unsupported = argv();
    *unsupported.last_mut().unwrap() = "windows1252".into();
    assert!(parse_validate(unsupported).is_err());
    let mut missing = argv();
    missing.truncate(missing.len() - 2);
    assert!(parse_validate(missing).is_err());
}
#[test]
fn output_alias_and_explicit_directory_paths_preserve_spaces() {
    let mut arguments = argv();
    let index = arguments.iter().position(|arg| arg == "--errors").unwrap();
    arguments[index] = "--output".into();
    arguments.extend(
        [
            "--schema-dir",
            "a bundle/schemas",
            "--reference-data-dir",
            "a bundle/reference-data",
        ]
        .map(OsString::from),
    );
    let parsed = parse_validate(arguments).unwrap();
    assert_eq!(parsed.errors, Some(PathBuf::from("errors.jsonl")));
    assert_eq!(parsed.dirs.schema_dir, Some("a bundle/schemas".into()));
    assert_eq!(
        parsed.dirs.reference_data_dir,
        Some("a bundle/reference-data".into())
    );
}
#[test]
fn controlled_outcomes_have_the_documented_distinct_codes() {
    assert_eq!(exit_code::from_status(RunStatus::CompletedValid), 0);
    assert_eq!(exit_code::TECHNICAL, 1);
    assert_eq!(exit_code::from_status(RunStatus::CompletedInvalid), 2);
    assert_eq!(exit_code::from_status(RunStatus::Cancelled), 130);
}
#[test]
fn complete_pairs_resolve_at_each_level_with_strict_priority() {
    let file = Fixture::new();
    let cli = file.pair("cli");
    let env = file.pair("env");
    let adjacent = file.pair("exe");
    let exe = file.exe("exe");
    let environment = environment(&env);
    assert_pair(
        resolve_bundle_source(&cli, &environment, &exe).unwrap(),
        &cli,
    );
    assert_pair(
        resolve_bundle_source(&CliDirs::default(), &environment, &exe).unwrap(),
        &env,
    );
    assert_pair(
        resolve_bundle_source(&CliDirs::default(), &FakeEnv::default(), &exe).unwrap(),
        &adjacent,
    );
}
#[test]
fn incomplete_pairs_never_mix_levels_or_fall_back() {
    let file = Fixture::new();
    let valid = file.pair("valid");
    let adjacent = file.pair("exe");
    let partial = CliDirs {
        schema_dir: valid.schema_dir.clone(),
        reference_data_dir: None,
    };
    assert!(matches!(
        resolve_bundle_source(&partial, &environment(&valid), &file.exe("exe")),
        Err(BundleResolveError::Incomplete {
            level: BundleLevel::CommandLine,
            missing: DirectoryKind::ReferenceData
        })
    ));
    let env = FakeEnv(BTreeMap::from([(
        "LINTRA_REFERENCE_DATA_DIR",
        adjacent.reference_data_dir.unwrap().into_os_string(),
    )]));
    assert!(matches!(
        resolve_bundle_source(&CliDirs::default(), &env, &file.exe("exe")),
        Err(BundleResolveError::Incomplete {
            level: BundleLevel::Environment,
            missing: DirectoryKind::Schemas
        })
    ));
    fs::create_dir_all(file.0.join("incomplete/schemas")).unwrap();
    assert!(matches!(
        resolve_bundle_source(
            &CliDirs::default(),
            &FakeEnv::default(),
            &file.exe("incomplete")
        ),
        Err(BundleResolveError::Incomplete {
            level: BundleLevel::Executable,
            missing: DirectoryKind::ReferenceData
        })
    ));
}
#[test]
fn nonexistent_empty_and_file_roots_fail_without_fallback() {
    let file = Fixture::new();
    let valid = file.pair("exe");
    let env = environment(&valid);
    for schema in [
        file.0.join("absent"),
        PathBuf::new(),
        file.0.join("plain file"),
    ] {
        fs::write(file.0.join("plain file"), b"synthetic").unwrap();
        let dirs = CliDirs {
            schema_dir: Some(schema),
            reference_data_dir: valid.reference_data_dir.clone(),
        };
        assert!(resolve_bundle_source(&dirs, &env, &file.exe("exe")).is_err());
    }
    let mut env = env;
    env.0
        .insert("LINTRA_SCHEMA_DIR", file.0.join("absent").into_os_string());
    assert!(matches!(
        resolve_bundle_source(&CliDirs::default(), &env, &file.exe("exe")),
        Err(BundleResolveError::Inspect { .. })
    ));
}
#[test]
fn no_bundle_and_invalid_executable_do_not_discover_cwd_or_home() {
    let file = Fixture::new();
    // A complete unrelated pair cannot be chosen without its explicit path.
    let _unrelated = file.pair("unrelated");
    let result = resolve_bundle_source(
        &CliDirs::default(),
        &FakeEnv::default(),
        &file.exe("missing"),
    );
    #[cfg(not(feature = "embedded-bundle"))]
    assert!(matches!(result, Err(BundleResolveError::NoExternalBundle)));
    #[cfg(feature = "embedded-bundle")]
    assert_eq!(result.unwrap(), BundleSource::Embedded);
    assert!(matches!(
        resolve_bundle_source(
            &CliDirs::default(),
            &FakeEnv::default(),
            Path::new("lintra")
        ),
        Err(BundleResolveError::InvalidExecutablePath)
    ));
}

/// Existing validate mapping assertions continue to inspect the exact request;
/// the parser now dispatches multiple operations without changing that request.
fn parse_validate<I, T>(arguments: I) -> Result<args::ValidateArgs, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match args::parse(arguments)? {
        args::CliCommand::Validate(args) => Ok(*args),
        _ => panic!("test expected validate"),
    }
}

#[cfg(unix)]
#[test]
fn dangling_adjacent_link_does_not_enable_embedded_fallback() {
    use std::os::unix::fs::symlink;
    let file = Fixture::new();
    fs::create_dir_all(file.0.join("broken")).unwrap();
    symlink(file.0.join("absent"), file.0.join("broken/schemas")).unwrap();
    assert!(matches!(
        resolve_bundle_source(
            &CliDirs::default(),
            &FakeEnv::default(),
            &file.exe("broken")
        ),
        Err(BundleResolveError::Inspect { .. })
    ));
}
