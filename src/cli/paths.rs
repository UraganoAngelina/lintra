//! Shared adapter policy; CLI and UI execute the same bundle resolution rules.
pub use lintra::{
    BundleDirectories as CliDirs, BundleSelection as BundleSource,
    SystemBundleEnvironment as SystemEnv, resolve_bundle_source,
};
#[cfg(test)]
pub use lintra::{
    BundleDirectoryKind as DirectoryKind, BundleEnvironment as EnvProvider, BundleResolveError,
    BundleSelectionLevel as BundleLevel,
};
