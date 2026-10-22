//! Actual closed-plan evaluation over real compiled scalar fields, without
//! advertising the still-gated full derived schema path before its integration.
use lintra::*;
use serde_json::{Value, json};
fn schema() -> CompiledFlowSchema {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("golden/dsl/v1/schema.json")).unwrap();
    let trace = &mut value["traces"][0];
    let base = trace["fields"][0].clone();
    let mut fields = vec![];
    for (id, start, length, kind) in [
        ("BIRTH", 1, 8, "date"),
        ("REF", 9, 8, "date"),
        ("AGE", 17, 3, "numeric"),
        ("CODE", 20, 5, "alphanumeric"),
    ] {
        let mut field = base.clone();
        field["id"] = json!(id);
        field["name"] = json!(id);
        field["physical"] = if kind == "date" {
            json!({"type":"date","start":start,"length":length,"format":"DDMMYYYY","alignment":{"kind":"exact"}})
        } else {
            json!({"type":kind,"start":start,"length":length,"alignment":if kind=="alphanumeric"{json!({"kind":"left_pad","fill_byte":32,"content_policy":"strip_leading_fill"})}else{json!({"kind":"exact"})},"lexical":{"kind":"encoded_text"}})
        };
        fields.push(field);
    }
    trace["record_length"] = json!(24);
    trace["fields"] = json!(fields);
    trace["rules"] = json!([]);
    trace["record_key"]["fields"] = json!(["CODE"]);
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw).unwrap()
}
fn field(index: usize) -> ResolvedFieldRef {
    ResolvedFieldRef::Local {
        field_index: index,
        field_id: FieldId(["BIRTH", "REF", "AGE", "CODE"][index].into()),
    }
}
fn age(policy: LeapDayAnniversary) -> DerivedValuePlan {
    DerivedValuePlan::AgeInCompletedYears {
        birth: field(0),
        reference: field(1),
        format: DateFormat::DdMmYyyy,
        leap_day: policy,
    }
}
fn delta() -> DerivedValuePlan {
    DerivedValuePlan::CalendarYearDelta {
        birth: field(0),
        reference: field(1),
        format: DateFormat::DdMmYyyy,
    }
}
fn bytes(birth: &str, reference: &str, age: &str) -> Vec<u8> {
    format!("{birth}{reference}{age}  AB ").into_bytes()
}
fn number(value: DerivedValue<'_>) -> IntegerLiteral {
    let DerivedValue::Integer(value) = value else {
        panic!("integer required")
    };
    value
}
fn evaluate<'a>(
    plan: &DerivedValuePlan,
    local: &'a LocalEvaluationContext<'_>,
) -> DerivedValue<'a> {
    DerivedValueEvaluator.evaluate(plan, &EvaluationContext { fields: local })
}
#[test]
fn completed_years_and_calendar_year_delta_are_distinct_and_never_use_host_date() {
    let schema = schema();
    for (birth, reference, completed, year_delta) in [
        ("31012000", "30012021", "20", "21"),
        ("31012000", "31012021", "21", "21"),
        ("31122000", "01012025", "24", "25"),
        ("01010001", "31129999", "9998", "9998"),
        ("01012025", "01012021", "-4", "-4"),
        ("31012025", "30012021", "-5", "-4"),
        ("31129999", "01010001", "-9999", "-9998"),
    ] {
        let bytes = bytes(birth, reference, "021");
        let original = bytes.clone();
        let view = RecordView::new(&bytes);
        let local = LocalEvaluationContext {
            record: &view,
            schema: &schema.traces()[0],
        };
        assert_eq!(
            number(evaluate(&age(LeapDayAnniversary::Mar01), &local)),
            completed.parse().unwrap()
        );
        assert_eq!(
            number(evaluate(&delta(), &local)),
            year_delta.parse().unwrap()
        );
        assert_eq!(bytes, original);
    }
}
#[test]
fn february_29_anniversary_uses_only_the_explicit_policy_including_centuries() {
    let schema = schema();
    for (birth, reference, feb28, mar01) in [
        ("29022000", "28022021", "21", "20"),
        ("29022000", "01032021", "21", "21"),
        ("29022000", "28022020", "19", "19"),
        ("29022000", "29022020", "20", "20"),
        ("29021896", "28021900", "4", "3"),
        ("29021896", "29022000", "104", "104"),
    ] {
        let bytes = bytes(birth, reference, "021");
        let view = RecordView::new(&bytes);
        let local = LocalEvaluationContext {
            record: &view,
            schema: &schema.traces()[0],
        };
        assert_eq!(
            number(evaluate(&age(LeapDayAnniversary::Feb28), &local)),
            feb28.parse().unwrap()
        );
        assert_eq!(
            number(evaluate(&age(LeapDayAnniversary::Mar01), &local)),
            mar01.parse().unwrap()
        );
    }
}
#[test]
fn invalid_missing_and_short_dates_are_not_fabricated_ages() {
    let schema = schema();
    for (birth, reference, reason) in [
        ("31022000", "01012021", UnknownReason::InvalidFieldValue),
        ("        ", "01012021", UnknownReason::FieldUnavailable),
    ] {
        let bytes = bytes(birth, reference, "021");
        let view = RecordView::new(&bytes);
        let local = LocalEvaluationContext {
            record: &view,
            schema: &schema.traces()[0],
        };
        assert_eq!(
            evaluate(&age(LeapDayAnniversary::Feb28), &local),
            DerivedValue::Unknown(reason)
        );
    }
    let view = RecordView::new(b"short");
    let local = LocalEvaluationContext {
        record: &view,
        schema: &schema.traces()[0],
    };
    assert_eq!(
        evaluate(&delta(), &local),
        DerivedValue::Unknown(UnknownReason::FieldUnavailable)
    );
}
#[test]
fn substring_borrows_exact_raw_or_logical_bytes_and_checks_runtime_logical_length() {
    let schema = schema();
    let bytes = bytes("01012000", "01012021", "021");
    let view = RecordView::new(&bytes);
    let local = LocalEvaluationContext {
        record: &view,
        schema: &schema.traces()[0],
    };
    for (target, expected) in [
        (ValueTarget::Raw, b"  ".as_slice()),
        (ValueTarget::Logical, b"AB".as_slice()),
    ] {
        let plan = DerivedValuePlan::Substring {
            source: field(3),
            target,
            range: ByteRange::new(0, 2).unwrap(),
        };
        let value = evaluate(&plan, &local);
        assert_eq!(value, DerivedValue::Bytes(expected));
        let DerivedValue::Bytes(slice) = value else {
            panic!("bytes")
        };
        let base = if target == ValueTarget::Raw { 19 } else { 21 };
        assert_eq!(slice.as_ptr(), bytes[base..].as_ptr());
    }
    let plan = DerivedValuePlan::Substring {
        source: field(3),
        target: ValueTarget::Logical,
        range: ByteRange::new(0, 4).unwrap(),
    };
    assert_eq!(
        evaluate(&plan, &local),
        DerivedValue::Unknown(UnknownReason::FieldUnavailable)
    );
}
// This composition resolves actual plans through the real evaluator. It is a
// test adapter over the context port, not a runtime provider or fake age value.
struct Context<'a> {
    local: LocalEvaluationContext<'a>,
    plans: &'a [DerivedValuePlan],
}
impl FieldValueResolver for Context<'_> {
    fn resolve_field(&self, reference: &ResolvedFieldRef) -> ResolvedFieldValue<'_> {
        self.local.resolve_field(reference)
    }
    fn resolve_raw_field(&self, reference: &ResolvedFieldRef) -> ResolvedFieldValue<'_> {
        self.local.resolve_raw_field(reference)
    }
    fn resolve_derived(&self, index: usize) -> DerivedValue<'_> {
        match self.plans.get(index) {
            Some(plan) => evaluate(plan, &self.local),
            None => DerivedValue::Unknown(UnknownReason::FieldUnavailable),
        }
    }
}
#[test]
fn derived_eq_compares_bytes_or_exact_numeric_value_without_signed_field_syntax() {
    let schema = schema();
    let plans = [
        age(LeapDayAnniversary::Mar01),
        DerivedValuePlan::Substring {
            source: field(3),
            target: ValueTarget::Logical,
            range: ByteRange::new(0, 3).unwrap(),
        },
    ];
    for (input, expected) in [
        ("021", TruthValue::True),
        ("020", TruthValue::False),
        ("ABC", TruthValue::Unknown(UnknownReason::InvalidFieldValue)),
        ("-21", TruthValue::Unknown(UnknownReason::InvalidFieldValue)),
        ("+21", TruthValue::Unknown(UnknownReason::InvalidFieldValue)),
        ("   ", TruthValue::Unknown(UnknownReason::FieldUnavailable)),
    ] {
        let bytes = bytes("01012000", "01012021", input);
        let view = RecordView::new(&bytes);
        let resolver = Context {
            local: LocalEvaluationContext {
                record: &view,
                schema: &schema.traces()[0],
            },
            plans: &plans,
        };
        let ctx = EvaluationContext { fields: &resolver };
        assert_eq!(
            ExpressionEvaluator.evaluate(
                &CompiledExpr::DerivedEq {
                    field: field(2),
                    derived_index: 0
                },
                &ctx
            ),
            expected
        );
        assert_eq!(
            ExpressionEvaluator.evaluate(
                &CompiledExpr::DerivedEq {
                    field: field(3),
                    derived_index: 1
                },
                &ctx
            ),
            TruthValue::True
        );
    }
}
#[test]
fn derived_compare_covers_all_operators_typed_mismatches_and_unknown_propagation() {
    let schema = schema();
    let bytes = bytes("01012000", "01012021", "021");
    let view = RecordView::new(&bytes);
    let plans = [age(LeapDayAnniversary::Feb28)];
    let resolver = Context {
        local: LocalEvaluationContext {
            record: &view,
            schema: &schema.traces()[0],
        },
        plans: &plans,
    };
    let ctx = EvaluationContext { fields: &resolver };
    for (operator, valid) in [
        (ComparisonOperator::Eq, true),
        (ComparisonOperator::Ne, false),
        (ComparisonOperator::Lt, false),
        (ComparisonOperator::Le, true),
        (ComparisonOperator::Gt, false),
        (ComparisonOperator::Ge, true),
    ] {
        let expression = CompiledExpr::DerivedCompare {
            derived_index: 0,
            operator,
            value: CompiledValue::Integer("21".parse().unwrap()),
        };
        assert_eq!(
            ExpressionEvaluator.evaluate(&expression, &ctx),
            if valid {
                TruthValue::True
            } else {
                TruthValue::False
            }
        );
    }
    for (index, value, reason) in [
        (
            0,
            CompiledValue::Bytes(b"21".to_vec()),
            UnknownReason::UnsupportedPlan,
        ),
        (
            99,
            CompiledValue::Integer("0".parse().unwrap()),
            UnknownReason::FieldUnavailable,
        ),
    ] {
        assert_eq!(
            ExpressionEvaluator.evaluate(
                &CompiledExpr::DerivedCompare {
                    derived_index: index,
                    operator: ComparisonOperator::Eq,
                    value
                },
                &ctx
            ),
            TruthValue::Unknown(reason)
        );
    }
    assert_eq!(
        resolver.local.resolve_derived(0),
        DerivedValue::Unknown(UnknownReason::FieldUnavailable)
    );
}

#[test]
fn zero_age_and_unavailable_sources_remain_typed_without_defaults() {
    let schema = schema();
    let bytes = bytes("01012000", "01012000", "000");
    let view = RecordView::new(&bytes);
    let plans = [age(LeapDayAnniversary::Mar01)];
    let resolver = Context {
        local: LocalEvaluationContext {
            record: &view,
            schema: &schema.traces()[0],
        },
        plans: &plans,
    };
    assert_eq!(
        ExpressionEvaluator.evaluate(
            &CompiledExpr::DerivedEq {
                field: field(2),
                derived_index: 0
            },
            &EvaluationContext { fields: &resolver }
        ),
        TruthValue::True
    );
    struct Missing(UnknownReason);
    impl FieldValueResolver for Missing {
        fn resolve_field(&self, _: &ResolvedFieldRef) -> ResolvedFieldValue<'_> {
            ResolvedFieldValue::Unavailable(self.0)
        }
    }
    for reason in [
        UnknownReason::FieldUnavailable,
        UnknownReason::RelatedRecordMissing,
        UnknownReason::TraceInputUnavailable,
        UnknownReason::AmbiguousJoin,
        UnknownReason::UnsupportedPlan,
        UnknownReason::InvalidFieldValue,
    ] {
        let fields = Missing(reason);
        assert_eq!(
            DerivedValueEvaluator.evaluate(&plans[0], &EvaluationContext { fields: &fields }),
            DerivedValue::Unknown(reason)
        );
    }
}
