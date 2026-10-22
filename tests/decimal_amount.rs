//! Approved monetary examples, exact-byte validation and unchanged categorical ranges.
use lintra::*;
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
const EXAMPLE: &[u8] = include_bytes!("../docs/schema-authoring/examples/format2/amount-flow.json");
fn value() -> Value {
    serde_json::from_slice(EXAMPLE).unwrap()
}
fn bound(value: Value) -> RawFlowSchema {
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    raw
}
fn compiled(value: Value) -> CompiledFlowSchema {
    SchemaCompiler.compile(bound(value)).unwrap()
}
fn check(schema: &CompiledFlowSchema, bytes: &[u8]) -> (FieldPipelineResult, Vec<ValidationError>) {
    let trace = &schema.traces()[0];
    let factory = ErrorFactory {
        flow_id: &schema.key().flow_id,
        schema_version: &schema.key().version,
        trace_id: trace.id(),
        encoding: schema.encoding(),
    };
    let mut sink = VecSink::default();
    let mut summary = ValidationSummaryBuilder::new(
        schema.key().clone(),
        ScopeSummary::Trace {
            trace_id: trace.id().clone(),
        },
    );
    let result = FieldPipeline::new(factory, &mut sink, &mut summary)
        .run(
            &trace.fields()[0],
            FieldSlice { raw: bytes },
            1,
            &RecordIdentity::default(),
        )
        .unwrap();
    (result, sink.errors().to_vec())
}
#[test]
fn declared_plan_is_read_only_exact_and_keeps_categorical_range_numeric() {
    let schema = compiled(value());
    assert_eq!(schema.schema_format_version(), 2);
    let fields = schema.traces()[0].fields();
    let FieldType::Decimal(spec) = fields[0].field_type() else {
        panic!("explicit Decimal")
    };
    assert_eq!(
        (
            spec.total_length(),
            spec.scale(),
            spec.decimal_separator(),
            spec.sign_policy()
        ),
        (7, 2, b'.', SignPolicy::Unsigned)
    );
    assert_eq!(fields[0].alignment(), AlignmentRule::Exact);
    assert!(fields[0].lexical().is_none());
    assert_eq!(fields[1].field_type(), FieldType::Numeric);
    assert!(matches!(
        fields[1].constraints()[0],
        CompiledConstraint::Range { .. }
    ));
}
#[test]
fn approved_representation_table_including_zero_signs_and_fraction_position() {
    let schema = compiled(value());
    for (bytes, valid) in [
        (b"0012.34".as_slice(), true),
        (b"0000.00", true),
        (b"1234.00", true),
        (b"9999.99", true),
        (b"0012,34", false),
        (b"0012.3 ", false),
        (b"012.340", false),
        (b"-012.34", false),
        (b"+012.34", false),
        (b"1,23.45", false),
        (b"0012e34", false),
        (b"0012.3", false),
        (b"0012.340", false),
        (b" 012.34", false),
        (b"0012.\0\0", false),
        ("００12.34".as_bytes(), false),
    ] {
        let (result, errors) = check(&schema, bytes);
        assert_eq!(result == FieldPipelineResult::Valid, valid, "{bytes:?}");
        if !valid {
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].code, ValidationCode::InvalidType);
        }
    }
}
#[test]
fn arbitrary_width_amounts_do_not_parse_through_machine_integers_or_floats() {
    let mut v = value();
    v["traces"][0]["record_length"] = 203.into();
    v["traces"][0]["fields"][0]["physical"]["length"] = 200.into();
    v["traces"][0]["fields"][1]["physical"]["start"] = 201.into();
    let schema = compiled(v);
    let mut bytes = vec![b'9'; 200];
    bytes[197] = b'.';
    assert_eq!(check(&schema, &bytes).0, FieldPipelineResult::Valid);
    bytes[197] = b',';
    assert_eq!(
        check(&schema, &bytes).1[0].code,
        ValidationCode::InvalidType
    );
}
#[test]
fn presence_precedes_decimal_and_optional_missing_does_not_make_zero_missing() {
    let schema = compiled(value());
    assert_eq!(
        check(&schema, b"       ").1[0].code,
        ValidationCode::RequiredField
    );
    let mut v = value();
    v["traces"][0]["fields"][0]["presence"]["base"] = "optional".into();
    let schema = compiled(v);
    let (result, errors) = check(&schema, b"       ");
    assert_eq!(result, FieldPipelineResult::MissingOptional);
    assert!(errors.is_empty());
    assert_eq!(check(&schema, b"0000.00").0, FieldPipelineResult::Valid);
}
#[test]
fn bad_decimal_stops_before_domain_and_valid_amounts_still_obey_domain() {
    let mut v = value();
    v["traces"][0]["fields"][0]["constraints"] =
        json!([{"kind":"allowed_values","values":["0012.34"],"target":"logical"}]);
    let schema = compiled(v);
    for (bytes, code) in [
        (b"0012,34", ValidationCode::InvalidType),
        (b"0000.00", ValidationCode::InvalidDomain),
    ] {
        let (_, errors) = check(&schema, bytes);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, code);
    }
}
#[test]
fn compiler_rejects_v1_decimal_bad_scale_width_and_numeric_range_on_amounts() {
    for case in 0..4 {
        let mut v = value();
        match case {
            0 => v["schema_format_version"] = 1.into(),
            1 => v["traces"][0]["fields"][0]["physical"]["scale"] = 3.into(),
            2 => v["traces"][0]["fields"][0]["physical"]["length"] = 3.into(),
            _ => {
                v["traces"][0]["fields"][0]["constraints"] =
                    json!([{"kind":"numeric_range","min":1,"max":4}])
            }
        }
        let error = SchemaCompiler.compile(bound(v)).unwrap_err();
        assert!(
            error
                .issues
                .iter()
                .any(|i| i.field_id == Some(FieldId("AMOUNT".into()))
                    && matches!(
                        i.code,
                        SchemaIssueCode::UnsupportedConstruct
                            | SchemaIssueCode::InvalidTypeParameters
                    ))
        );
    }
}
#[test]
fn raw_shape_has_no_defaults_or_unknown_decimal_options() {
    for property in ["scale", "decimal_separator", "sign_policy", "alignment"] {
        let mut v = value();
        v["traces"][0]["fields"][0]["physical"]
            .as_object_mut()
            .unwrap()
            .remove(property);
        assert!(
            serde_json::from_value::<RawFlowSchema>(v).is_err(),
            "{property}"
        );
    }
    for (property, wrong) in [
        ("decimal_separator", json!(",")),
        ("sign_policy", json!("signed")),
        (
            "alignment",
            json!({"kind":"left_pad","fill_byte":48,"content_policy":"strip_leading_fill"}),
        ),
        ("extra", json!(true)),
    ] {
        let mut v = value();
        v["traces"][0]["fields"][0]["physical"][property] = wrong;
        assert!(
            serde_json::from_value::<RawFlowSchema>(v).is_err(),
            "{property}"
        );
    }
}
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(input: &[u8], schema: &Value) -> Self {
        let root = std::env::temp_dir().join(format!(
            "lintra amounts à {} {} {}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("schemas/SYNTHETIC-AMOUNTS/v2")).unwrap();
        fs::create_dir(root.join("reference-data")).unwrap();
        fs::create_dir(root.join("cwd")).unwrap();
        fs::write(
            root.join("schemas/SYNTHETIC-AMOUNTS/v2/flow.json"),
            serde_json::to_vec(schema).unwrap(),
        )
        .unwrap();
        fs::write(root.join("input.txt"), input).unwrap();
        Self(root)
    }
    fn run(&self) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_lintra"))
            .current_dir(self.0.join("cwd"))
            .env_remove("LINTRA_SCHEMA_DIR")
            .env_remove("LINTRA_REFERENCE_DATA_DIR")
            .args([
                "validate",
                "--flow",
                "SYNTHETIC-AMOUNTS",
                "--schema-version",
                "v2",
                "--trace",
                "T1",
                "--encoding",
                "utf8",
            ])
            .arg("--input")
            .arg(self.0.join("input.txt"))
            .arg("--schema-dir")
            .arg(self.0.join("schemas"))
            .arg("--reference-data-dir")
            .arg(self.0.join("reference-data"))
            .arg("--errors")
            .arg(self.0.join("errors.jsonl"))
            .arg("--summary")
            .arg(self.0.join("summary.json"))
            .output()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn actual_cli_runs_amounts_and_preserves_the_existing_categorical_range() {
    for (input, code, diagnostic) in [
        (b"0000.00001\n".as_slice(), 0, None),
        (b"0012,34001\n", 2, Some("E_INVALID_TYPE")),
        (b"0012.34005\n", 2, Some("E_OUT_OF_RANGE")),
    ] {
        let file = Fixture::new(input, &value());
        let out = file.run();
        assert_eq!(out.status.code(), Some(code), "{:?}", out.stderr);
        let errors = fs::read(file.0.join("errors.jsonl")).unwrap();
        if let Some(expected) = diagnostic {
            let row: Value = serde_json::from_slice(&errors).unwrap();
            assert_eq!(row["code"], expected);
        } else {
            assert!(errors.is_empty());
        }
        let summary: Value =
            serde_json::from_slice(&fs::read(file.0.join("summary.json")).unwrap()).unwrap();
        assert_eq!(summary["inputs"][0]["completion"]["kind"], "complete");
        assert_eq!(errors, {
            let again = file.run();
            assert_eq!(again.status.code(), Some(code));
            fs::read(file.0.join("errors.jsonl")).unwrap()
        });
        assert!(!file.0.join("errors.jsonl.part").exists());
        assert!(!file.0.join("summary.json.part").exists());
    }
}
#[test]
fn v1_amounts_are_a_technical_schema_failure_without_published_reports() {
    let mut v = value();
    v["schema_format_version"] = 1.into();
    let file = Fixture::new(b"0012.34001\n", &v);
    assert_eq!(file.run().status.code(), Some(1));
    assert!(!file.0.join("errors.jsonl").exists());
    assert!(!file.0.join("summary.json").exists());
    assert!(!file.0.join("errors.jsonl.part").exists());
    assert!(!file.0.join("summary.json.part").exists());
}

#[test]
fn smallest_amount_has_one_integral_digit_and_exactly_two_fractional_digits() {
    let mut v = value();
    v["traces"][0]["record_length"] = 7.into();
    v["traces"][0]["fields"][0]["physical"]["length"] = 4.into();
    v["traces"][0]["fields"][1]["physical"]["start"] = 5.into();
    let schema = compiled(v);
    for bytes in [b"0.00", b"9.99"] {
        assert_eq!(check(&schema, bytes).0, FieldPipelineResult::Valid);
    }
    assert_eq!(
        check(&schema, b".000").1[0].code,
        ValidationCode::InvalidType
    );
}

#[test]
fn names_do_not_classify_numeric_fields_as_amounts_in_either_format() {
    let original: Value =
        serde_json::from_slice(include_bytes!("fixtures/application/schema.json")).unwrap();
    for format in [1, 2] {
        let mut v = original.clone();
        v["schema_format_version"] = format.into();
        v["traces"][0]["fields"][0]["name"] =
            "Importo compartecipazione (synthetic code, not money)".into();
        let schema = compiled(v);
        assert_eq!(schema.schema_format_version(), format);
        assert_eq!(
            schema.traces()[0].fields()[0].field_type(),
            FieldType::Numeric
        );
        assert!(matches!(
            schema.traces()[0].fields()[0].constraints()[0],
            CompiledConstraint::AllowedValues { .. }
        ));
    }
}
