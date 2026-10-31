pub(crate) mod raw;

pub(crate) mod compiled;

pub(crate) mod repository;

pub(crate) mod linter;

pub use compiled::compiler::{SchemaCompilationError, SchemaCompiler};

pub(crate) mod compatibility;

pub(crate) mod canonical;
pub(crate) mod diff;
pub(crate) mod historical;
pub(crate) mod registry;

pub(crate) mod expression;
pub(crate) mod reference_binding;
pub(crate) mod rule_dependencies;

pub(crate) mod presence;

pub(crate) mod rule_template;

pub(crate) mod coordinates;
pub(crate) mod derived;

pub(crate) mod importer;

pub(crate) mod authoring;
#[cfg(feature = "embedded-bundle")]
pub(crate) mod embedded;
