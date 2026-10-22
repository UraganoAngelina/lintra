//! Real compiled local contexts and all operators, including unavailable data.
use TruthValue::{False as F, True as T, Unknown as U};
use lintra::*;
fn schema() -> CompiledFlowSchema {
    let mut raw: RawFlowSchema =
        serde_json::from_slice(include_bytes!("fixtures/application/schema.json")).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw).unwrap()
}
fn reference(index: usize) -> ResolvedFieldRef {
    ResolvedFieldRef::Local {
        field_index: index,
        field_id: FieldId(format!("F{}", index + 1)),
    }
}
fn evaluate(schema: &CompiledFlowSchema, bytes: &[u8], expression: &CompiledExpr) -> TruthValue {
    let record = RecordView::new(bytes);
    let local = LocalEvaluationContext {
        record: &record,
        schema: &schema.traces()[0],
    };
    ExpressionEvaluator.evaluate(expression, &EvaluationContext { fields: &local })
}
#[test]
fn every_comparison_operator_covers_true_false_and_missing_unknown() {
    let schema = schema();
    let expressions = [
        (CompiledExpr::Eq(reference(1), b"A".to_vec()), T, F),
        (CompiledExpr::Ne(reference(1), b"A".to_vec()), F, T),
        (
            CompiledExpr::In(reference(1), vec![b"A".to_vec(), b"B".to_vec()]),
            T,
            F,
        ),
        (
            CompiledExpr::NotIn(reference(1), vec![b"A".to_vec(), b"B".to_vec()]),
            F,
            T,
        ),
    ];
    for (expression, first, second) in expressions {
        assert_eq!(evaluate(&schema, b"001A  ", &expression), first);
        assert_eq!(evaluate(&schema, b"001C  ", &expression), second);
        assert_eq!(
            evaluate(&schema, b"001   ", &expression),
            U(UnknownReason::FieldUnavailable)
        );
        assert_eq!(
            evaluate(&schema, b"001A", &expression),
            U(UnknownReason::FieldUnavailable)
        );
    }
}
#[test]
fn presence_predicates_distinguish_missing_present_and_unavailable() {
    let schema = schema();
    for (expression, present, missing) in [
        (CompiledExpr::IsPresent(reference(1)), T, F),
        (CompiledExpr::IsMissing(reference(1)), F, T),
    ] {
        assert_eq!(evaluate(&schema, b"001A  ", &expression), present);
        assert_eq!(evaluate(&schema, b"001   ", &expression), missing);
        assert_eq!(
            evaluate(&schema, b"001", &expression),
            U(UnknownReason::FieldUnavailable)
        );
    }
}
#[test]
fn literal_matching_uses_declared_logical_bytes_without_generic_trim_or_parsing() {
    let schema = schema();
    for (bytes, index, literal, expected) in [
        (b"001A  ".as_slice(), 0, b"1".as_slice(), T),
        (b"001A  ", 0, b"001", F),
        (b"001 A ", 1, b"A", F),
        (b"001 A ", 1, b" A", T),
        (b"001a  ", 1, b"A", F),
        // All-zero Numeric is not missing under the declared spaces policy.
        (b"000A  ", 0, b"", T),
        (b"   A  ", 0, b"", U(UnknownReason::FieldUnavailable)),
    ] {
        let original = bytes.to_vec();
        assert_eq!(
            evaluate(
                &schema,
                bytes,
                &CompiledExpr::Eq(reference(index), literal.to_vec())
            ),
            expected
        );
        assert_eq!(bytes, original);
    }
}
#[test]
fn boolean_operators_execute_complete_three_valued_tables() {
    let schema = schema();
    let unknown = U(UnknownReason::FieldUnavailable);
    let operands = [
        CompiledExpr::Const(false),
        CompiledExpr::Const(true),
        CompiledExpr::Eq(reference(1), b"A".to_vec()),
    ];
    let and = [[F, F, F], [F, T, unknown], [F, unknown, unknown]];
    let or = [[F, T, unknown], [T, T, T], [unknown, T, unknown]];
    for (row, left) in operands.iter().enumerate() {
        for (column, right) in operands.iter().enumerate() {
            assert_eq!(
                evaluate(
                    &schema,
                    b"001   ",
                    &CompiledExpr::And(vec![left.clone(), right.clone()])
                ),
                and[row][column]
            );
            assert_eq!(
                evaluate(
                    &schema,
                    b"001   ",
                    &CompiledExpr::Or(vec![left.clone(), right.clone()])
                ),
                or[row][column]
            );
        }
        let expected = [T, F, unknown][row];
        assert_eq!(
            evaluate(
                &schema,
                b"001   ",
                &CompiledExpr::Not(Box::new(left.clone()))
            ),
            expected
        );
    }
}
#[test]
fn invalid_indices_ids_widths_and_unsupported_plans_are_explicit() {
    let schema = schema();
    for reference in [
        reference(99),
        ResolvedFieldRef::Local {
            field_index: 1,
            field_id: FieldId("F1".into()),
        },
    ] {
        assert_eq!(
            evaluate(&schema, b"001A  ", &CompiledExpr::IsPresent(reference)),
            U(UnknownReason::FieldUnavailable)
        );
    }
    let external = ResolvedFieldRef::External {
        trace_index: 1,
        field_index: 0,
        field_id: FieldId("F1".into()),
        join_index: 0,
        join_id: JoinId("J".into()),
    };
    for expression in [
        CompiledExpr::IsPresent(external),
        CompiledExpr::DerivedEq {
            field: reference(0),
            derived_index: 0,
        },
        CompiledExpr::And(vec![]),
        CompiledExpr::Or(vec![]),
        CompiledExpr::In(reference(0), vec![]),
    ] {
        assert_eq!(
            evaluate(&schema, b"001A  ", &expression),
            U(UnknownReason::UnsupportedPlan)
        );
    }
    let record = RecordView::new(b"001A  ");
    let local = LocalEvaluationContext {
        record: &record,
        schema: &schema.traces()[0],
    };
    assert_eq!(local.resolve_local(0), Some(b"001".as_slice()));
    assert_eq!(local.resolve_local(99), None);
}

// A pure test implementation demonstrates the actual context port; it does
// not stand in for a production join/provider or advertise Full support.
struct Fixed(ResolvedFieldValue<'static>);
impl FieldValueResolver for Fixed {
    fn resolve_field(&self, _: &ResolvedFieldRef) -> ResolvedFieldValue<'_> {
        self.0
    }
}
#[test]
fn abstract_context_preserves_every_unavailability_reason() {
    for reason in [
        UnknownReason::FieldUnavailable,
        UnknownReason::RelatedRecordMissing,
        UnknownReason::TraceInputUnavailable,
        UnknownReason::AmbiguousJoin,
        UnknownReason::UnsupportedPlan,
    ] {
        let resolver = Fixed(ResolvedFieldValue::Unavailable(reason));
        let ctx = EvaluationContext { fields: &resolver };
        assert_eq!(
            ExpressionEvaluator.evaluate(&CompiledExpr::IsMissing(reference(0)), &ctx),
            U(reason)
        );
        assert_eq!(
            ExpressionEvaluator.evaluate(&CompiledExpr::Ne(reference(0), vec![]), &ctx),
            U(reason)
        );
    }
}
#[test]
fn repeated_queries_and_nested_expressions_have_no_validation_order_state() {
    let schema = schema();
    let first = CompiledExpr::Eq(reference(0), b"1".to_vec());
    let second = CompiledExpr::Eq(reference(1), b"A".to_vec());
    assert_eq!(evaluate(&schema, b"001A  ", &second), T);
    assert_eq!(evaluate(&schema, b"001A  ", &first), T);
    assert_eq!(evaluate(&schema, b"001A  ", &second), T);
    let mut nested = CompiledExpr::And(vec![first, second]);
    for _ in 0..256 {
        nested = CompiledExpr::Not(Box::new(nested));
    }
    assert_eq!(evaluate(&schema, b"001A  ", &nested), T);
}

#[test]
fn nested_unknown_diagnostics_follow_declared_operand_order() {
    struct Reasons;
    impl FieldValueResolver for Reasons {
        fn resolve_field(&self, reference: &ResolvedFieldRef) -> ResolvedFieldValue<'_> {
            let reason = match reference {
                ResolvedFieldRef::Local { field_index: 0, .. } => UnknownReason::AmbiguousJoin,
                _ => UnknownReason::TraceInputUnavailable,
            };
            ResolvedFieldValue::Unavailable(reason)
        }
    }
    let resolver = Reasons;
    let ctx = EvaluationContext { fields: &resolver };
    for (first, second, expected) in [
        (0, 1, UnknownReason::AmbiguousJoin),
        (1, 0, UnknownReason::TraceInputUnavailable),
    ] {
        let operands = vec![
            CompiledExpr::IsPresent(reference(first)),
            CompiledExpr::Not(Box::new(CompiledExpr::IsMissing(reference(second)))),
        ];
        assert_eq!(
            ExpressionEvaluator.evaluate(&CompiledExpr::And(operands.clone()), &ctx),
            U(expected)
        );
        assert_eq!(
            ExpressionEvaluator.evaluate(&CompiledExpr::Or(operands), &ctx),
            U(expected)
        );
    }
}
