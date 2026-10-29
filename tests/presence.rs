//! Synthetic unconditional presence: real compilation and diagnostic enrichment.
use lintra::{
    CompiledFlowSchema, ErrorFactory, FieldSlice, PresenceValidationResult, RawFlowSchema,
    RawResolvedEncoding, RecordIdentity, SchemaCompiler, ValidationCode, validate_presence,
};
use serde_json::{Value, json};

/// Compile a three-byte synthetic field with an explicit base/missing policy.
/// JSON never constructs compiled types directly; no domain or datatype rules
/// are evaluated by these tests. Each call owns an independent immutable schema.
fn schema(base: &str, missing: Value) -> CompiledFlowSchema {
    let mut raw: RawFlowSchema = serde_json::from_value(json!({
        "schema_format_version":1,"flow_id":"SYNTHETIC","schema_version":"v1","regulatory_version":"synthetic",
        "source":{},"encoding":{"mode":"runtime_required","position_unit":"byte"},"required_capabilities":["local.v1"],
        "parameters":[],"reference_data":[],"persistence_policy":null,
        "traces":[{"id":"T1","name":"Synthetic presence","record_length":3,
            "fields":[{"id":"F1","name":"Synthetic field",
                "physical":{"type":"alphanumeric","start":1,"length":3,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}},
                "missing_value":missing,"presence":{"base":base,"required_if":[],"forbidden_if":[]},"constraints":[]}],
            "record_key":{"fields":["F1"],"uniqueness_scope":"trace","error_code":"E_DUPLICATE_RECORD_KEY"},
            "joins":[],"identity":[],"named_conditions":{},"derived_values":{},"repeated_groups":[],"rules":[],"documented_non_executable_rules":[]}]
    })).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw).unwrap()
}

/// Required ASCII spaces emit the single lightweight primary violation; the
/// validator does not manufacture downstream datatype/domain failures.
#[test]
fn missing_required_has_only_the_presence_violation() {
    let schema = schema("required", json!({"kind":"spaces"}));
    let field = &schema.traces()[0].fields()[0];
    let error = validate_presence(field, FieldSlice { raw: b"   " }).unwrap_err();
    assert_eq!(error.code, ValidationCode::RequiredField);
    assert_eq!(error.expected.as_deref(), Some("valore presente"));
    assert!(error.reason.is_empty());
    assert!(error.diagnostic_context.is_empty());
}

/// MissingOptional is distinguishable from Present so the future pipeline can
/// stop successfully instead of checking the datatype of an absent value.
#[test]
fn optional_missing_and_present_are_distinct_successes() {
    let schema = schema("optional", json!({"kind":"spaces"}));
    let field = &schema.traces()[0].fields()[0];
    assert_eq!(
        validate_presence(field, FieldSlice { raw: b"   " }),
        Ok(PresenceValidationResult::MissingOptional)
    );
    assert_eq!(
        validate_presence(field, FieldSlice { raw: b"ABC" }),
        Ok(PresenceValidationResult::Present)
    );
}

/// Spaces recognizes only ASCII space: zeros, tabs, NUL and non-UTF-8 bytes
/// remain present here. Their lexical validity belongs to a later validator.
#[test]
fn presence_does_not_trim_decode_or_validate_the_value() {
    let schema = schema("required", json!({"kind":"spaces"}));
    let field = &schema.traces()[0].fields()[0];
    for raw in [
        b"000".as_slice(),
        b" A ",
        b"\t\t\t",
        &[0, 0xff, 0],
        &[0xc2, 0xa0, b' '],
    ] {
        assert_eq!(
            validate_presence(field, FieldSlice { raw }),
            Ok(PresenceValidationResult::Present)
        );
    }
}

/// Only the schema can declare zeros/literals missing. Spaces and NeverMissing
/// therefore produce different outcomes for exactly the same source bytes.
#[test]
fn all_missing_policies_are_used_without_a_global_zero_rule() {
    for (policy, raw, missing) in [
        (json!({"kind":"exact","raw":"000"}), b"000".as_slice(), true),
        (json!({"kind":"exact","raw":"000"}), b"   ", false),
        (
            json!({"kind":"any_of","raw_values":["   ","---"]}),
            b"---",
            true,
        ),
        (
            json!({"kind":"any_of","raw_values":["   ","---"]}),
            b"000",
            false,
        ),
        (json!({"kind":"never_missing"}), b"   ", false),
    ] {
        let schema = schema("required", policy);
        let result = validate_presence(&schema.traces()[0].fields()[0], FieldSlice { raw });
        if missing {
            assert_eq!(result.unwrap_err().code, ValidationCode::RequiredField);
        } else {
            assert_eq!(result, Ok(PresenceValidationResult::Present));
        }
    }
}

/// Enrichment stays in ErrorFactory: presence supplies only FieldViolation,
/// and positions/identity come from the compiled field and caller metadata.
#[test]
fn presence_violation_integrates_with_the_existing_error_factory() {
    let schema = schema("required", json!({"kind":"spaces"}));
    let trace = &schema.traces()[0];
    let field = &trace.fields()[0];
    let violation = validate_presence(field, FieldSlice { raw: b"   " }).unwrap_err();
    let error = ErrorFactory {
        flow_id: &schema.key().flow_id,
        schema_version: &schema.key().version,
        trace_id: trace.id(),
        encoding: schema.encoding(),
    }
    .field_error(9, &RecordIdentity::default(), field, b"   ", violation);
    assert_eq!(error.validate(), Ok(()));
    assert_eq!(error.code, ValidationCode::RequiredField);
    assert_eq!(error.line_number, 9);
    assert_eq!(error.actual.unwrap().display, "   ");
    let field = error.field.unwrap();
    assert_eq!(
        (field.start_one_based, field.end_one_based_inclusive),
        (1, 3)
    );
    assert_eq!(
        error.message,
        format!(
            "Campo F1 (Synthetic field): valore obbligatorio mancante. {}",
            lintra::field_format_guide(&schema.traces()[0].fields()[0], schema.encoding())
        )
    );
}
