use super::*;
use crate::schema::raw::RawFlowSchema;
use serde_json::{Value, json};

fn record() -> RawRecordSchema {
    let raw: RawFlowSchema = serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/application/schema.json"
    ))
    .unwrap();
    raw.traces.into_iter().next().unwrap()
}
fn expr(value: Value) -> RawExpr {
    serde_json::from_value(value).unwrap()
}
fn local(reference: &ResolvedFieldRef, index: usize, id: &str) {
    assert_eq!(
        reference,
        &ResolvedFieldRef::Local {
            field_index: index,
            field_id: FieldId(id.into())
        }
    );
}
#[test]
fn every_local_leaf_lowers_to_indices_and_exact_literal_bytes() {
    let record = record();
    for operator in ["eq", "ne", "in", "not_in", "is_present", "is_missing"] {
        let mut node = json!({"ref":{"field":"F2"}});
        if matches!(operator, "eq" | "ne") {
            node["value"] = json!(" é ");
        }
        if matches!(operator, "in" | "not_in") {
            node["values"] = json!(["01", "1", " é "]);
        }
        let compiled = compile_local_expr(&expr(json!({operator:node})), &record).unwrap();
        assert!(match operator {
            "eq" => matches!(compiled, CompiledExpr::Eq(..)),
            "ne" => matches!(compiled, CompiledExpr::Ne(..)),
            "in" => matches!(compiled, CompiledExpr::In(..)),
            "not_in" => matches!(compiled, CompiledExpr::NotIn(..)),
            "is_present" => matches!(compiled, CompiledExpr::IsPresent(..)),
            "is_missing" => matches!(compiled, CompiledExpr::IsMissing(..)),
            _ => false,
        });
        match compiled {
            CompiledExpr::Eq(reference, bytes) | CompiledExpr::Ne(reference, bytes) => {
                local(&reference, 1, "F2");
                assert_eq!(bytes, " é ".as_bytes());
            }
            CompiledExpr::In(reference, bytes) | CompiledExpr::NotIn(reference, bytes) => {
                local(&reference, 1, "F2");
                assert_eq!(
                    bytes,
                    [b"01".to_vec(), b"1".to_vec(), " é ".as_bytes().to_vec()]
                );
            }
            CompiledExpr::IsPresent(reference) | CompiledExpr::IsMissing(reference) => {
                local(&reference, 1, "F2")
            }
            _ => panic!("leaf required"),
        }
    }
}
#[test]
fn nested_boolean_tree_preserves_order_and_forward_field_references() {
    let record = record();
    let compiled = compile_local_expr(
        &expr(json!({"and":[
            {"is_present":{"ref":{"field":"F2"}}},
            {"or":[{"is_missing":{"ref":{"field":"F1"}}},{"not":{"const":false}}]}
        ]})),
        &record,
    )
    .unwrap();
    let CompiledExpr::And(children) = compiled else {
        panic!("and")
    };
    assert!(
        matches!(&children[0], CompiledExpr::IsPresent(reference) if *reference == ResolvedFieldRef::Local{field_index:1,field_id:FieldId("F2".into())})
    );
    let CompiledExpr::Or(children) = &children[1] else {
        panic!("or")
    };
    assert!(matches!(&children[0], CompiledExpr::IsMissing(_)));
    assert!(
        matches!(&children[1], CompiledExpr::Not(child) if matches!(**child, CompiledExpr::Const(false)))
    );
}
#[test]
fn unknown_local_and_external_references_never_fall_back() {
    for (reference, code) in [
        (json!({"field":"ABSENT"}), Code::MissingFieldReference),
        (
            json!({"trace":"T2","field":"F1","join":"J"}),
            Code::UnsupportedConstruct,
        ),
    ] {
        let error = compile_local_expr(
            &expr(json!({"eq":{"ref":reference,"value":"A"}})),
            &record(),
        )
        .unwrap_err();
        assert_eq!(error.0, code);
    }
}
#[test]
fn named_conditions_expand_and_detect_direct_and_indirect_cycles() {
    let mut record = record();
    record
        .named_conditions
        .insert("A".into(), Box::new(expr(json!({"condition":"B"}))));
    record
        .named_conditions
        .insert("B".into(), Box::new(expr(json!({"const":true}))));
    let repeated = expr(json!({"and":[{"condition":"A"},{"condition":"A"}]}));
    let CompiledExpr::And(children) = compile_local_expr(&repeated, &record).unwrap() else {
        panic!("and")
    };
    assert!(
        children
            .iter()
            .all(|node| matches!(node, CompiledExpr::Const(true)))
    );
    record
        .named_conditions
        .insert("B".into(), Box::new(expr(json!({"condition":"A"}))));
    assert_eq!(
        compile_local_expr(&repeated, &record).unwrap_err().0,
        Code::SelfReferentialRule
    );
    record
        .named_conditions
        .insert("A".into(), Box::new(expr(json!({"condition":"A"}))));
    assert_eq!(
        compile_local_expr(&repeated, &record).unwrap_err().0,
        Code::SelfReferentialRule
    );
    assert_eq!(
        compile_local_expr(&expr(json!({"condition":"ABSENT"})), &record)
            .unwrap_err()
            .0,
        Code::InvalidSchemaValue
    );
}
#[test]
fn empty_boolean_and_membership_nodes_and_duplicate_sets_are_invalid() {
    for expression in [
        json!({"and":[]}),
        json!({"or":[]}),
        json!({"in":{"ref":{"field":"F1"},"values":[]}}),
        json!({"not_in":{"ref":{"field":"F1"},"values":["A","A"]}}),
    ] {
        assert_eq!(
            compile_local_expr(&expr(expression), &record())
                .unwrap_err()
                .0,
            Code::InvalidSchemaValue
        );
    }
}
#[test]
fn long_named_chain_uses_explicit_work_stack() {
    let mut record = record();
    for index in 0..4096 {
        record.named_conditions.insert(
            index.to_string(),
            Box::new(expr(json!({"condition":(index+1).to_string()}))),
        );
    }
    record
        .named_conditions
        .insert("4096".into(), Box::new(expr(json!({"const":true}))));
    assert!(matches!(
        compile_local_expr(&expr(json!({"condition":"0"})), &record).unwrap(),
        CompiledExpr::Const(true)
    ));
}
