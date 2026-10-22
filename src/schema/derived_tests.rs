use super::*;
use crate::schema::expression::compile_local_expr;
use serde_json::{Value, json};
fn record() -> RawRecordSchema {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("../../tests/golden/dsl/v1/schema.json")).unwrap();
    let trace = &mut value["traces"][0];
    let base = trace["fields"][0].clone();
    let mut fields = Vec::new();
    for (id, start, length, kind) in [
        ("BIRTH", 1, 8, "date"),
        ("REF", 9, 8, "date"),
        ("AGE", 17, 3, "numeric"),
        ("CODE", 20, 10, "alphanumeric"),
    ] {
        let mut field = base.clone();
        field["id"] = json!(id);
        field["name"] = json!(id);
        field["physical"] = if kind == "date" {
            json!({"type":"date","start":start,"length":length,"alignment":{"kind":"exact"},"format":"DDMMYYYY"})
        } else {
            json!({"type":kind,"start":start,"length":length,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}})
        };
        fields.push(field);
    }
    trace["fields"] = json!(fields);
    trace["record_length"] = json!(29);
    trace["record_key"]["fields"] = json!(["CODE"]);
    trace["rules"] = json!([]);
    trace["derived_values"] = json!({
        "AGE":{"kind":"age_in_completed_years","birth_date_ref":{"field":"BIRTH"},"reference_date_ref":{"field":"REF"},"date_format":"DDMMYYYY","leap_day_anniversary":"feb28"},
        "PREFIX":{"kind":"substring","source_ref":{"field":"CODE"},"target":"logical","start":1,"length":3},
        "YEAR":{"kind":"calendar_year_delta","birth_date_ref":{"field":"BIRTH"},"reference_date_ref":{"field":"REF"},"date_format":"DDMMYYYY"}
    });
    serde_json::from_value::<RawFlowSchema>(value)
        .unwrap()
        .traces
        .remove(0)
}
fn expr(value: Value) -> RawExpr {
    serde_json::from_value(value).unwrap()
}
#[test]
fn three_closed_plans_have_exact_local_indices_ranges_and_explicit_calendar_policy() {
    let record = record();
    let plans = compile_derived_values(&record).unwrap();
    assert_eq!(plans.len(), 3);
    assert!(matches!(
        &plans[0],
        DerivedValuePlan::AgeInCompletedYears {
            birth: ResolvedFieldRef::Local { field_index: 0, .. },
            reference: ResolvedFieldRef::Local { field_index: 1, .. },
            format: DateFormat::DdMmYyyy,
            leap_day: LeapDayAnniversary::Feb28
        }
    ));
    assert!(
        matches!(&plans[1],DerivedValuePlan::Substring{source:ResolvedFieldRef::Local{field_index:3,..},target:ValueTarget::Logical,range} if range.start()==0&&range.len()==3)
    );
    assert!(matches!(
        &plans[2],
        DerivedValuePlan::CalendarYearDelta {
            birth: ResolvedFieldRef::Local { field_index: 0, .. },
            reference: ResolvedFieldRef::Local { field_index: 1, .. },
            format: DateFormat::DdMmYyyy
        }
    ));
    let mut value = serde_json::to_value(&record).unwrap();
    value["derived_values"]["AGE"]["leap_day_anniversary"] = json!("mar01");
    value["derived_values"]["PREFIX"]["target"] = json!("raw");
    let record = serde_json::from_value(value).unwrap();
    let plans = compile_derived_values(&record).unwrap();
    assert!(matches!(
        &plans[0],
        DerivedValuePlan::AgeInCompletedYears {
            leap_day: LeapDayAnniversary::Mar01,
            ..
        }
    ));
    assert!(matches!(
        &plans[1],
        DerivedValuePlan::Substring {
            target: ValueTarget::Raw,
            ..
        }
    ));
}
#[test]
fn substring_coordinates_and_field_type_errors_never_create_partial_plans() {
    for (start, length, code) in [
        (0, 1, Code::InvalidPosition),
        (1, 0, Code::ZeroLength),
        (9, 3, Code::InvalidTypeParameters),
    ] {
        let mut value = serde_json::to_value(record()).unwrap();
        value["derived_values"]["PREFIX"]["start"] = json!(start);
        value["derived_values"]["PREFIX"]["length"] = json!(length);
        let record = serde_json::from_value(value).unwrap();
        assert_eq!(compile_derived_values(&record).unwrap_err().0, code);
    }
    let mut value = serde_json::to_value(record()).unwrap();
    value["derived_values"]["AGE"]["birth_date_ref"] = json!({"field":"CODE"});
    let record = serde_json::from_value(value).unwrap();
    assert_eq!(
        compile_derived_values(&record).unwrap_err().0,
        Code::ConstraintTypeMismatch
    );
}
#[test]
fn missing_and_external_sources_are_explicit_and_no_leap_policy_is_defaulted() {
    for (source, code) in [
        (json!({"field":"ABSENT"}), Code::MissingFieldReference),
        (
            json!({"trace":"T2","field":"BIRTH","join":"J"}),
            Code::UnsupportedConstruct,
        ),
    ] {
        let mut value = serde_json::to_value(record()).unwrap();
        value["derived_values"]["AGE"]["birth_date_ref"] = source;
        let record = serde_json::from_value(value).unwrap();
        assert_eq!(compile_derived_values(&record).unwrap_err().0, code);
    }
    let mut value = serde_json::to_value(record()).unwrap();
    value["derived_values"]["AGE"]
        .as_object_mut()
        .unwrap()
        .remove("leap_day_anniversary");
    assert!(serde_json::from_value::<RawRecordSchema>(value).is_err());
}
#[test]
fn derived_expression_lowering_is_typed_and_uses_the_same_canonical_map_indices() {
    let record = record();
    let expression = compile_local_expr(
        &expr(json!({"derived_eq":{"ref":{"field":"AGE"},"derived_value":"AGE"}})),
        &record,
    )
    .unwrap();
    assert!(matches!(
        expression,
        CompiledExpr::DerivedEq {
            field: ResolvedFieldRef::Local { field_index: 2, .. },
            derived_index: 0
        }
    ));
    let expression = compile_local_expr(
        &expr(json!({"derived_eq":{"ref":{"field":"CODE"},"derived_value":"PREFIX"}})),
        &record,
    )
    .unwrap();
    assert!(matches!(
        expression,
        CompiledExpr::DerivedEq {
            derived_index: 1,
            ..
        }
    ));
    let expression = compile_local_expr(
        &expr(json!({"derived_compare":{"derived_value":"YEAR","operator":">=","value":-1}})),
        &record,
    )
    .unwrap();
    assert!(
        matches!(expression,CompiledExpr::DerivedCompare{derived_index:2,operator:ComparisonOperator::Ge,value:CompiledValue::Integer(value)} if value=="-1".parse().unwrap())
    );
}
#[test]
fn derived_names_literals_and_integer_targets_cannot_be_guessed_or_coerced() {
    let record = record();
    for (expression, code) in [
        (
            json!({"derived_compare":{"derived_value":"ABSENT","operator":"==","value":1}}),
            Code::MissingDerivedValueReference,
        ),
        (
            json!({"derived_compare":{"derived_value":"AGE","operator":"==","value":"1"}}),
            Code::ConstraintTypeMismatch,
        ),
        (
            json!({"derived_compare":{"derived_value":"PREFIX","operator":"==","value":1}}),
            Code::ConstraintTypeMismatch,
        ),
        (
            json!({"derived_eq":{"ref":{"field":"CODE"},"derived_value":"AGE"}}),
            Code::ConstraintTypeMismatch,
        ),
    ] {
        assert_eq!(
            compile_local_expr(&expr(expression), &record)
                .unwrap_err()
                .0,
            code
        );
    }
}
