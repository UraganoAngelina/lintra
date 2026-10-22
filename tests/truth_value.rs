//! Explicit DET §19 tables: expected values are independent of implementation.
use TruthValue::{False as F, True as T, Unknown as U};
use UnknownReason::*;
use lintra::{TruthValue, UnknownReason};

#[test]
fn complete_and_or_not_tables_cover_all_unknown_reasons() {
    for reason in [
        FieldUnavailable,
        RelatedRecordMissing,
        TraceInputUnavailable,
        AmbiguousJoin,
        UnsupportedPlan,
        InvalidFieldValue,
    ] {
        let unknown = U(reason);
        let values = [F, T, unknown];
        let and = [[F, F, F], [F, T, unknown], [F, unknown, unknown]];
        let or = [[F, T, unknown], [T, T, T], [unknown, T, unknown]];
        for (row, left) in values.iter().enumerate() {
            for (column, right) in values.iter().enumerate() {
                assert_eq!(left.and(*right), and[row][column]);
                assert_eq!(left.or(*right), or[row][column]);
            }
        }
        for (value, expected) in [(F, T), (T, F), (unknown, unknown)] {
            assert_eq!(!value, expected);
            assert_eq!(!(!value), value);
        }
    }
}
#[test]
fn distinct_unknown_reasons_use_declared_left_order() {
    let reasons = [
        FieldUnavailable,
        RelatedRecordMissing,
        TraceInputUnavailable,
        AmbiguousJoin,
        UnsupportedPlan,
        InvalidFieldValue,
    ];
    for left in reasons {
        for right in reasons {
            assert_eq!(U(left).and(U(right)), U(left));
            assert_eq!(U(left).or(U(right)), U(left));
            assert_eq!(F.and(U(right)), F);
            assert_eq!(T.or(U(right)), T);
        }
    }
}
