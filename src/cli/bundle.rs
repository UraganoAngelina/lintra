//! Adapter composition for a selected whole bundle. No file-level fallback.
use super::{args::ValidateArgs, paths::BundleSource};
use lintra::*;
#[cfg(feature = "embedded-bundle")]
use std::io;
use std::{error::Error, fs, path::PathBuf, sync::Arc};
type Failure = Box<dyn Error>;

/// Returns the actual service and audit plus filesystem inputs to protect.
/// Embedded has no fictitious filesystem roots or temporary extraction.
pub(super) struct SelectedBundle {
    pub service: ValidationService,
    pub audit: BundleAudit,
    pub roots: Vec<PathBuf>,
    pub manifest_path: Option<PathBuf>,
}

/// Only the standard sibling pair admits implicit manifest metadata. Explicit
/// paths remain valid for roots with unrelated parents. Missing implicit metadata
/// means absent; malformed/unreadable/present symlinks never mean absent.
fn manifest_path(
    args: &ValidateArgs,
    schema: &std::path::Path,
    reference: &std::path::Path,
) -> Result<Option<PathBuf>, Failure> {
    external_bundle_manifest_path(args.bundle_manifest.as_deref(), schema, reference)
        .map_err(Into::into)
}

pub(super) fn compose(
    args: &ValidateArgs,
    source: BundleSource,
) -> Result<SelectedBundle, Failure> {
    match source {
        BundleSource::External {
            schema_root,
            reference_data_root,
        } => {
            let path = manifest_path(args, &schema_root, &reference_data_root)?;
            let manifest = path
                .as_ref()
                .map(|p| -> Result<_, Failure> { Ok(BundleManifest::parse(&fs::read(p)?)?) })
                .transpose()?;
            let audit = BundleAudit::external(manifest.as_ref());
            let service = match manifest {
                None => build_local_service_with_reference_data(
                    schema_root.clone(),
                    reference_data_root.clone(),
                    args.encoding.clone(),
                    args.reference_policy,
                )?,
                Some(manifest) => {
                    let repository =
                        ManifestSchemaRepository::new(schema_root.clone(), manifest.clone())?;
                    let registry = SchemaRegistry::new(
                        Arc::new(repository),
                        SchemaCompiler,
                        args.encoding.clone(),
                    );
                    ValidationService::new(Arc::new(registry)).with_reference_data(
                        Arc::new(ManifestReferenceDataPreparer::new(
                            reference_data_root.clone(),
                            manifest,
                        )?),
                        args.reference_policy,
                    )
                }
            };
            Ok(SelectedBundle {
                service,
                audit,
                roots: vec![schema_root, reference_data_root],
                manifest_path: path,
            })
        }
        #[cfg(feature = "embedded-bundle")]
        BundleSource::Embedded => {
            // An explicit external manifest cannot describe a different snapshot.
            if args.bundle_manifest.is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "--bundle-manifest requires an external bundle",
                )
                .into());
            }
            let bundle = EmbeddedBundle::builtin()?;
            let audit = BundleAudit::embedded(&bundle);
            let repository = EmbeddedSchemaRepository::new(bundle.clone())?;
            let registry =
                SchemaRegistry::new(Arc::new(repository), SchemaCompiler, args.encoding.clone());
            let service = ValidationService::new(Arc::new(registry)).with_reference_data(
                Arc::new(EmbeddedReferenceDataPreparer::new(bundle)),
                args.reference_policy,
            );
            Ok(SelectedBundle {
                service,
                audit,
                roots: vec![],
                manifest_path: None,
            })
        }
    }
}
