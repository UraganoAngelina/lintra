//! Compiler step 12: type compatibility and parameter validity stay distinct.
use lintra::{RawFlowSchema, RawResolvedEncoding, SchemaCompiler, SchemaIssueCode};
use serde_json::{Value, json};

// Use the approved format-2 synthetic amount profile, retaining its untouched
// categorical participation range, while substituting one explicit field type.
fn schema(kind: &str, constraints: Value) -> RawFlowSchema {
    let mut value: Value = serde_json::from_slice(include_bytes!(
        "../docs/schema-authoring/examples/format2/amount-flow.json"
    ))
    .unwrap();
    let field = &mut value["traces"][0]["fields"][0];
    field["physical"] = match kind {
        "date" => {
            json!({"type":"date","start":1,"length":8,"format":"DDMMYYYY","alignment":{"kind":"exact"}})
        }
        "decimal" => field["physical"].clone(),
        _ => {
            json!({"type":kind,"start":1,"length":7,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}})
        }
    };
    field["constraints"] = constraints;
    if kind == "date" {
        value["traces"][0]["record_length"] = json!(11);
        value["traces"][0]["fields"][1]["physical"]["start"] = json!(9);
    }
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    raw
}
#[test]
fn constraint_type_mismatch_matrix_covers_every_implemented_type() {
    for kind in ["numeric", "alphanumeric", "date", "decimal"] {
        let literal = match kind {
            "date" => "01012026",
            "decimal" => "0000.00",
            "numeric" => "0000001",
            _ => "EXAMPLE",
        };
        for (constraint, allowed) in [
            (
                json!({"kind":"allowed_values","target":"raw","values":[literal]}),
                true,
            ),
            (
                json!({"kind":"numeric_range","min":0,"max":100}),
                kind == "numeric",
            ),
            (
                json!({"kind":"date","format":"DDMMYYYY","calendar_valid":true}),
                kind == "date",
            ),
            (
                json!({"kind":"regex","pattern":".","match":"required","target":"raw"}),
                true,
            ),
        ] {
            let result = SchemaCompiler.compile(schema(kind, json!([constraint])));
            if allowed {
                result.unwrap();
            } else {
                let error = result.unwrap_err();
                assert_eq!(error.issues.len(), 1, "{kind}: {error:?}");
                let issue = &error.issues[0];
                assert_eq!(issue.code, SchemaIssueCode::ConstraintTypeMismatch);
                assert!(issue.trace_id.as_ref().is_some_and(|id| id.0 == "T1"));
                assert!(issue.field_id.as_ref().is_some_and(|id| id.0 == "AMOUNT"));
            }
        }
    }
}
#[test]
fn domain_width_and_reversed_range_remain_parameter_errors() {
    for (constraint, code) in [
        (
            json!({"kind":"allowed_values","target":"raw","values":["123"]}),
            SchemaIssueCode::InvalidTypeParameters,
        ),
        (
            json!({"kind":"numeric_range","min":10,"max":1}),
            SchemaIssueCode::InvalidSchemaValue,
        ),
    ] {
        let error = SchemaCompiler
            .compile(schema("numeric", json!([constraint])))
            .unwrap_err();
        assert!(error.issues.iter().any(|issue| issue.code == code));
        assert!(
            !error
                .issues
                .iter()
                .any(|issue| issue.code == SchemaIssueCode::ConstraintTypeMismatch)
        );
    }
    SchemaCompiler
        .compile(schema(
            "numeric",
            json!([{"kind":"allowed_values","target":"raw","values":["0000001"]}]),
        ))
        .unwrap();
}
#[test]
fn type_and_parameter_issues_are_both_reported_without_runtime_fallback() {
    let raw = schema(
        "decimal",
        json!([{"kind":"numeric_range","min":10,"max":1}]),
    );
    let error = SchemaCompiler.compile(raw).unwrap_err();
    assert_eq!(
        error
            .issues
            .iter()
            .map(|issue| issue.code)
            .collect::<Vec<_>>(),
        [
            SchemaIssueCode::ConstraintTypeMismatch,
            SchemaIssueCode::InvalidSchemaValue
        ]
    );
}
