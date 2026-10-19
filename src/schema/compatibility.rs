//! Data-format support is independent from normative AST/runtime capabilities.
use std::{fmt, ops::RangeInclusive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaFormatVersion(pub u32);

// Official development baseline ratified by the author in CNF-FORMAT-01.
// This is not inferred from the current package version or a future format number.
const FORMAT_ONE_MINIMUM_ENGINE_VERSION: &str = "0.1.0";
const FORMAT_ONE_BASELINE_COMPILER: &str = "b658e05";

impl SchemaFormatVersion {
    /// None means the format has no specification or compatible executable declared.
    pub fn minimum_engine_version(self) -> Option<&'static str> {
        match self.0 {
            1 => Some(FORMAT_ONE_MINIMUM_ENGINE_VERSION),
            _ => None,
        }
    }
}

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

    /// An unsupported but defined format reports its approved minimum; an undefined
    /// format never recommends an invented release. Supported formats return None.
    pub fn incompatibility_message(&self, requested: SchemaFormatVersion) -> Option<String> {
        if self.supports_schema_format(requested) {
            return None;
        }
        let requirement = match requested.minimum_engine_version() {
            Some(minimum) => format!(
                "minimum required executable version: {minimum} (development baseline; verified compiler {FORMAT_ONE_BASELINE_COMPILER})"
            ),
            None => format!(
                "format is undefined; no compatible executable is declared; minimum required executable version: not applicable to an undefined format; defined format 1 minimum engine version: {FORMAT_ONE_MINIMUM_ENGINE_VERSION} (development baseline; verified compiler {FORMAT_ONE_BASELINE_COMPILER})"
            ),
        };
        Some(format!(
            "unsupported schema_format_version {requested}; engine version {}; supported schema formats {}..={}; {requirement}",
            self.engine_version,
            self.supported_schema_formats.start(),
            self.supported_schema_formats.end(),
        ))
    }

    pub fn supports_schema_format(&self, version: SchemaFormatVersion) -> bool {
        self.supported_schema_formats.contains(&version)
    }
}
