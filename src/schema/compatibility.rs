//! Data-format support is independent from normative AST/runtime capabilities.
use std::{fmt, ops::RangeInclusive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaFormatVersion(pub u32);

impl fmt::Display for SchemaFormatVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Descriptor of data formats, not a declaration of complete local.v1 support.
/// Creating another descriptor does not reconfigure SchemaCompiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineCapabilities {
    pub supported_schema_formats: RangeInclusive<SchemaFormatVersion>,
    pub engine_version: String,
}

impl EngineCapabilities {
    pub fn current() -> Self {
        Self {
            supported_schema_formats: SchemaFormatVersion(1)..=SchemaFormatVersion(1),
            engine_version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }

    pub fn supports_schema_format(&self, version: SchemaFormatVersion) -> bool {
        self.supported_schema_formats.contains(&version)
    }
}
