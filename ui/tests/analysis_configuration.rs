//! Real catalogs and synthetic files; no UI-side normative rule or validation run.
use lintra::*;
use lintra_ui::analysis::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, ffi::OsString, fs, path::PathBuf};

#[derive(Default)]
struct Environment(BTreeMap<String, OsString>);
impl BundleEnvironment for Environment {
    fn var_os(&self, key: &str) -> Option<OsString> {
        self.0.get(key).cloned()
    }
}
struct Fixture {
    root: tempfile::TempDir,
    dirs: BundleDirectories,
    input: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let schemas = root.path().join("schemas");
        let refs = root.path().join("reference-data");
        fs::create_dir(&schemas).unwrap();
        fs::create_dir(&refs).unwrap();
        for version in ["v1", "v2"] {
            let folder = schemas.join("SYNTHETIC").join(version);
            fs::create_dir_all(&folder).unwrap();
            let mut schema: Value = serde_json::from_slice(include_bytes!(
                "../../bundles/embedded/schemas/SYNTHETIC/v1/flow.json"
            ))
            .unwrap();
            schema["schema_version"] = json!(version);
            fs::write(
                folder.join("flow.json"),
                serde_json::to_vec(&schema).unwrap(),
            )
            .unwrap();
        }
        let input = root.path().join("input con spazi.txt");
        fs::write(&input, b"001A  \r\n").unwrap();
        Self {
            root,
            dirs: BundleDirectories {
                schema_dir: Some(schemas),
                reference_data_dir: Some(refs),
            },
            input,
        }
    }
    fn bundle(&self) -> AnalysisBundle {
        AnalysisBundle::resolve(
            &self.dirs,
            &Environment::default(),
            &self.root.path().join("gui.exe"),
        )
        .unwrap()
    }
    fn selection(&self) -> AnalysisSelection {
        AnalysisSelection {
            flow: Some("SYNTHETIC".into()),
            version: Some("v2".into()),
            trace: Some("T1".into()),
            encoding: Some(RawResolvedEncoding::Utf8),
            file: Some(self.input.clone()),
            reports: ReportSelection::default(),
        }
    }
}
#[test]
fn catalog_requires_explicit_version_and_preserves_declared_trace_selection() {
    let f = Fixture::new();
    let bundle = f.bundle();
    assert_eq!(bundle.flows(), ["SYNTHETIC"]);
    assert_eq!(bundle.versions("SYNTHETIC"), ["v1", "v2"]);
    let mut selection = AnalysisSelection::default();
    selection.select_flow(Some("SYNTHETIC".into()));
    assert!(selection.version.is_none());
    assert!(selection.trace.is_none());
    assert!(selection.key().is_err());
    selection.select_version(Some("v1".into()));
    assert!(selection.trace.is_none());
    selection.trace = Some("T1".into());
    selection.select_version(Some("v2".into()));
    assert!(selection.trace.is_none());
    assert_eq!(bundle.traces(&selection.key().unwrap()), ["T1"]);
    selection.select_flow(Some("OTHER".into()));
    assert!(selection.version.is_none());
    assert!(selection.trace.is_none());
}
#[test]
fn prepared_request_is_real_local_intent_without_running_validation_or_writing_reports() {
    let f = Fixture::new();
    let bundle = f.bundle();
    let original = fs::read(&f.input).unwrap();
    let mut selection = f.selection();
    selection.reports = ReportSelection {
        jsonl: false,
        xlsx: false,
        csv: false,
    };
    let p = selection.prepare(&bundle).unwrap();
    assert_eq!(p.request.schema_key.version, SchemaVersion("v2".into()));
    assert_eq!(p.request.mode, ValidationMode::LocalOnly);
    assert!(
        matches!(&p.request.scope,ValidationScope::Trace(trace)if trace.trace_id==TraceId("T1".into())&&trace.source==InputSource::Path(f.input.clone()))
    );
    assert_eq!(
        p.request.options.newline_policy,
        NewlinePolicy::AutoLfOrCrlf
    );
    assert_eq!(p.request.options.max_errors, None);
    assert_eq!(p.preview.crlf, 1);
    assert_eq!(p.audit.source(), "external");
    assert_eq!(p.audit.bundle_version(), None);
    assert!(bundle.origin_label().contains("non dichiarato"));
    assert_eq!(fs::read(&f.input).unwrap(), original);
    assert_eq!(fs::read_dir(f.root.path()).unwrap().count(), 3);
}
#[test]
fn invalid_selection_encoding_and_missing_file_never_prepare_a_request() {
    let f = Fixture::new();
    let bundle = f.bundle();
    let mut selection = f.selection();
    selection.version = None;
    assert!(selection.prepare(&bundle).is_err());
    selection = f.selection();
    selection.trace = Some("UNKNOWN".into());
    assert!(matches!(
        selection.prepare(&bundle),
        Err(ConfigurationError::InvalidSelection)
    ));
    selection = f.selection();
    selection.version = Some("missing".into());
    assert!(matches!(
        selection.prepare(&bundle),
        Err(ConfigurationError::InvalidSelection)
    ));
    selection = f.selection();
    selection.encoding = None;
    assert!(matches!(
        selection.prepare(&bundle),
        Err(ConfigurationError::Missing(_))
    ));
    selection = f.selection();
    selection.file = Some(f.root.path().join("missing.txt"));
    assert!(matches!(
        selection.prepare(&bundle),
        Err(ConfigurationError::Input(_))
    ));
}
#[test]
fn prefix_preflight_reports_actual_endings_without_decoding_or_rewriting() {
    let f = Fixture::new();
    let bytes = b"A\r\nB\nC\rD";
    fs::write(&f.input, bytes).unwrap();
    let p = InputPreview::inspect(&f.input).unwrap();
    assert_eq!((p.lf, p.crlf, p.lone_cr), (1, 1, 1));
    assert_eq!(p.sampled_bytes, bytes.len());
    assert!(p.reached_eof);
    assert_eq!(fs::read(&f.input).unwrap(), bytes);
    fs::write(&f.input, []).unwrap();
    let empty = InputPreview::inspect(&f.input).unwrap();
    assert_eq!(empty.sampled_bytes, 0);
    assert!(empty.reached_eof);
    assert!(matches!(
        InputPreview::inspect(f.root.path()),
        Err(ConfigurationError::Input(_))
    ));
}
#[test]
fn bom_rejection_reuses_core_longest_signatures_and_leaves_every_byte_untouched() {
    let f = Fixture::new();
    for bytes in [
        &b"\xef\xbb\xbf001A  \n"[..],
        &b"\xff\xfeA\0"[..],
        &b"\xff\xfe\0\0"[..],
        &b"\0\0\xfe\xff"[..],
    ] {
        fs::write(&f.input, bytes).unwrap();
        assert!(matches!(
            InputPreview::inspect(&f.input),
            Err(ConfigurationError::Bom(InputError::BomRejected(_)))
        ));
        assert_eq!(fs::read(&f.input).unwrap(), bytes);
    }
}
#[test]
fn preliminary_sample_is_bounded_and_is_not_a_total_record_count() {
    let f = Fixture::new();
    fs::write(&f.input, vec![b'\n'; 70000]).unwrap();
    let p = InputPreview::inspect(&f.input).unwrap();
    assert_eq!(p.file_size, 70000);
    assert_eq!(p.sampled_bytes, 65536);
    assert_eq!(p.lf, 65536);
    assert!(!p.reached_eof);
    assert!(p.label().contains("Campione 65536"));
}
#[test]
fn incomplete_or_corrupt_external_bundle_never_falls_back_to_embedded() {
    let f = Fixture::new();
    let dirs = BundleDirectories {
        schema_dir: f.dirs.schema_dir.clone(),
        reference_data_dir: None,
    };
    assert!(
        AnalysisBundle::resolve(
            &dirs,
            &Environment::default(),
            &f.root.path().join("gui.exe")
        )
        .is_err()
    );
    let manifest = json!({"bundle_version":"test-v1","schema_format_version":1,"files":{"schemas/SYNTHETIC/v1/flow.json":"00".repeat(32),"schemas/SYNTHETIC/v2/flow.json":"00".repeat(32)}});
    fs::write(
        f.root.path().join("MANIFEST.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    assert!(
        AnalysisBundle::resolve(
            &f.dirs,
            &Environment::default(),
            &f.root.path().join("gui.exe")
        )
        .is_err()
    );
}
#[test]
fn integrated_browser_lists_metadata_and_keeps_directory_navigation_explicit() {
    let f = Fixture::new();
    let listing = browse_directory(f.root.path()).unwrap();
    assert!(listing[0].directory);
    assert!(
        listing
            .iter()
            .any(|e| e.path == f.input && !e.directory && e.selectable)
    );
    assert!(browse_directory(std::path::Path::new("")).is_err());
    assert!(browse_directory(&f.root.path().join("missing")).is_err());
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let path = f
            .root
            .path()
            .join(OsString::from_vec(b"invalid-\xff".to_vec()));
        fs::write(&path, []).unwrap();
        let listing = browse_directory(f.root.path()).unwrap();
        assert!(listing.iter().any(|e| e.path == path && !e.selectable));
    }
}
