use lintra::{
    EngineCapabilities, RawFlowSchema, SchemaFormatVersion as Format, SchemaIssueCode,
    lint_raw_schema,
};
use serde_json::Value;

#[test]
fn current_descriptor_declares_exact_numeric_format_and_package_version() {
    let current = EngineCapabilities::current();
    assert_eq!(current.supported_schema_formats, Format(1)..=Format(1));
    assert_eq!(current.engine_version, env!("CARGO_PKG_VERSION"));
    assert!(current.supports_schema_format(Format(1)));
    for version in [0, 2, u32::MAX] {
        assert!(!current.supports_schema_format(Format(version)));
    }
}

#[test]
fn typed_ranges_have_inclusive_boundaries_without_version_string_comparison() {
    let descriptor = EngineCapabilities {
        supported_schema_formats: Format(9)..=Format(11),
        engine_version: "synthetic-descriptor".into(),
    };
    for version in [9, 10, 11] {
        assert!(descriptor.supports_schema_format(Format(version)));
    }
    for version in [0, 8, 12, u32::MAX] {
        assert!(!descriptor.supports_schema_format(Format(version)));
    }
    assert_eq!(Format(u32::MAX).to_string(), u32::MAX.to_string());
    assert!(Format(10) > Format(9));
}

#[test]
fn caller_descriptor_cannot_enable_an_unimplemented_format_in_lint() {
    let caller = EngineCapabilities {
        supported_schema_formats: Format(1)..=Format(u32::MAX),
        engine_version: "synthetic-descriptor".into(),
    };
    assert!(caller.supports_schema_format(Format(2)));
    let mut value: Value = serde_json::from_str(include_str!(
        "../docs/schema-authoring/examples/schemas/local-flow.json"
    ))
    .unwrap();
    value["schema_format_version"] = serde_json::json!(2);
    let raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    assert!(
        lint_raw_schema(&raw)
            .iter()
            .any(|i| i.code == SchemaIssueCode::UnsupportedSchemaFormat)
    );
    assert!(!EngineCapabilities::current().supports_schema_format(Format(2)));
}
