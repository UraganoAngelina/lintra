//! DEC-18 adapters. Inventory success certifies only completion of the operation.
use super::args::{ImportArgs, ReferenceListArgs};
use lintra::*;
use serde_json::{Value, json};
use std::{
    error::Error,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
type Failure = Box<dyn Error>;

/// Assemble JSON before touching stdout; write/flush errors still propagate.
fn emit(value: &Value, output: &mut dyn Write) -> Result<(), Failure> {
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    output.write_all(&bytes)?;
    output.flush()?;
    Ok(())
}
pub(super) fn issue_json(issue: &SchemaIssue) -> Value {
    json!({"severity":match issue.severity { SchemaIssueSeverity::Error => "error", SchemaIssueSeverity::Warning => "warning" },
        "code":format!("{:?}", issue.code), "flow_id":issue.flow_id.as_ref().map(|id| &id.0),
        "trace_id":issue.trace_id.as_ref().map(|id| &id.0), "field_id":issue.field_id.as_ref().map(|id| &id.0),
        "message":issue.message, "source_note":issue.source_note})
}
pub fn schema_list(root: PathBuf, output: &mut dyn Write) -> Result<(), Failure> {
    let entries = FileSystemSchemaRepository::new(root)?.list_schemas()?;
    let schemas = entries.into_iter().map(|entry| json!({"flow_id":entry.key.flow_id.0,
        "schema_version":entry.key.version.0,"traces":entry.traces.into_iter().map(|id| id.0).collect::<Vec<_>>()})).collect::<Vec<_>>();
    emit(&json!({"schemas":schemas}), output)
}

/// Fail root configuration even when there are zero dependencies. read_dir also
/// exercises directory access; mere successful metadata is not proof of access.
fn directory(root: &Path) -> io::Result<()> {
    if root.as_os_str().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "explicit nonempty root required",
        ));
    }
    // Open the directory for read access without scanning unrelated datasets.
    fs::read_dir(root)?;
    Ok(())
}
fn diagnostic(anomaly: &DatasetAnomaly) -> ReferenceDatasetDiagnostic {
    match anomaly {
        DatasetAnomaly::FileType(issue) => ReferenceDatasetDiagnostic::FileType(*issue),
        DatasetAnomaly::Io { operation, source } => ReferenceDatasetDiagnostic::Io {
            operation: *operation,
            error: match source.kind() {
                io::ErrorKind::NotFound => ReferenceIoKind::NotFound,
                io::ErrorKind::PermissionDenied => ReferenceIoKind::PermissionDenied,
                _ => ReferenceIoKind::Other,
            },
        },
        DatasetAnomaly::IntegrityMismatch => ReferenceDatasetDiagnostic::IntegrityMismatch,
        DatasetAnomaly::InvalidIdentity => ReferenceDatasetDiagnostic::InvalidIdentity,
        DatasetAnomaly::IdentityMismatch => ReferenceDatasetDiagnostic::IdentityMismatch,
        DatasetAnomaly::Format { issue, .. } => ReferenceDatasetDiagnostic::Format(issue.clone()),
    }
}
pub fn reference_list(args: ReferenceListArgs, output: &mut dyn Write) -> Result<(), Failure> {
    directory(&args.schema_root)?;
    directory(&args.reference_root)?;
    let raw = FileSystemSchemaRepository::new(args.schema_root)?.load_raw(&args.key)?;
    let dependencies = compile_reference_dependencies(&raw.reference_data)?;
    let policy = DatasetPreparationPolicy::ContinueWithAvailableEvidence;
    let prepared = FileSystemReferenceDataProvider::new(args.reference_root, dependencies.clone())?
        .prepare(policy)?;
    // Keep declaration and evidence together; sorting cannot misassociate hashes.
    let mut rows = raw
        .reference_data
        .iter()
        .zip(&dependencies)
        .map(|(decl, dep)| {
            let evidence = prepared
                .evidence(&dep.key)
                .expect("provider returns one evidence per binding");
            let audit = ReferenceDatasetAudit {
                requested: dep.key.clone(),
                declared: evidence.declared_key().cloned(),
                relative_path: dep.bundle_path.to_string(),
                expected_sha256: evidence.expected_sha256().clone(),
                actual_sha256: evidence.actual_sha256().cloned(),
                availability: evidence.availability(),
                diagnostics: evidence.anomalies().iter().map(diagnostic).collect(),
            };
            (decl, audit)
        })
        .collect::<Vec<_>>();
    rows.sort_by(|(a, _), (b, _)| {
        (&a.dataset_id, &a.version, &a.id).cmp(&(&b.dataset_id, &b.version, &b.id))
    });
    let datasets = rows
        .into_iter()
        .map(|(decl, audit)| {
            json!({"declaration_id":decl.id,
        "dataset_id":audit.requested.id.0,"version":audit.requested.version.0,
        "declared":audit.declared,"relative_path":audit.relative_path,"policy":policy,
        "availability":audit.availability,"expected_sha256":audit.expected_sha256,
        "actual_sha256":audit.actual_sha256,"diagnostics":audit.diagnostics})
        })
        .collect::<Vec<_>>();
    emit(
        &json!({"schema":{"flow_id":args.key.flow_id.0,"schema_version":args.key.version.0},
        "policy":policy,"datasets":datasets}),
        output,
    )
}

/// No input is opened for writing. Existing outputs (including symlinks/hard
/// links to inputs) are rejected before authoring, and persist_noclobber also
/// prevents a late destination from being replaced during publication.
pub fn schema_import(args: ImportArgs, diagnostics: &mut dyn Write) -> Result<(), Failure> {
    match fs::symlink_metadata(&args.output) {
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "import output already exists",
            )
            .into());
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let blueprint = read_raw_schema_file(&args.blueprint)?;
    let mut profile: XlsxImportProfile = serde_json::from_slice(&fs::read(&args.profile)?)?;
    if let Some(path) = args.aliases {
        let aliases: Vec<TypeAlias> = serde_json::from_slice(&fs::read(path)?)?;
        profile = normalize_import_profile(&profile, &aliases)?.profile;
    }
    let raw = XlsxSchemaImporter::new(blueprint, profile)?.import(&args.input)?;
    let issues = lint_raw_schema(&raw);
    for issue in &issues {
        emit(&issue_json(issue), diagnostics)?;
    }
    if issues
        .iter()
        .any(|issue| issue.severity == SchemaIssueSeverity::Error)
    {
        return Err(
            io::Error::new(io::ErrorKind::InvalidData, "imported draft has lint errors").into(),
        );
    }
    let bytes = canonical_schema_bytes(&raw)?;
    let parent = args
        .output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    // Same-directory temp file: final name becomes visible only after complete
    // bytes are flushed/synced. Failure drops the owned temporary file.
    let mut pending = tempfile::NamedTempFile::new_in(parent)?;
    pending.write_all(&bytes)?;
    pending.flush()?;
    pending.as_file().sync_all()?;
    pending.persist_noclobber(&args.output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct FailedWriter;
    impl Write for FailedWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed consumer"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn listing_propagates_stdout_io_failure() {
        let root = tempfile::tempdir().unwrap();
        let error = schema_list(root.path().into(), &mut FailedWriter).unwrap_err();
        assert_eq!(
            error.downcast_ref::<io::Error>().unwrap().kind(),
            io::ErrorKind::BrokenPipe
        );
    }
    #[test]
    fn invalid_catalog_never_touches_output() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("A/v1");
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("flow.json"), b"invalid").unwrap();
        let mut output = Vec::new();
        assert!(schema_list(root.path().into(), &mut output).is_err());
        assert!(output.is_empty());
    }
}
