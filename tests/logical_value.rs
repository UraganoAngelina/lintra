//! Author-ratified extraction oracles; no physical-padding violations exist.
use lintra::{AlignmentRule, ContentBoundaryPolicy, FieldSlice, LogicalValueError, logical_value};
use std::error::Error;

/// Right stripping removes the final run only: internal and opposite-edge
/// spaces are content, including the author's ` ROSSI ` example.
#[test]
fn trailing_fill_keeps_internal_and_leading_blanks() {
    let rule = AlignmentRule::RightPad {
        fill: b' ',
        content_policy: ContentBoundaryPolicy::StripTrailingFill,
    };
    for (raw, expected) in [
        (b" ROSSI ".as_slice(), b" ROSSI".as_slice()),
        (b"DE ROSSI  ", b"DE ROSSI"),
        (b" AB", b" AB"),
        (b"ABC", b"ABC"),
    ] {
        let result = logical_value(&rule, FieldSlice { raw }).unwrap();
        assert_eq!(result.raw, expected);
        assert_eq!(result.raw.as_ptr(), raw.as_ptr());
    }
}

/// Leading zeros are removed only when explicitly selected. Final/internal
/// zeros are not evidence of bad alignment or a value with a different meaning.
#[test]
fn leading_fill_keeps_internal_and_final_zeros() {
    let rule = AlignmentRule::LeftPad {
        fill: b'0',
        content_policy: ContentBoundaryPolicy::StripLeadingFill,
    };
    for (raw, expected, offset) in [
        (b"0010".as_slice(), b"10".as_slice(), 2),
        (b"1000", b"1000", 0),
        (b"001001", b"1001", 2),
    ] {
        let result = logical_value(&rule, FieldSlice { raw }).unwrap();
        assert_eq!(result.raw, expected);
        assert_eq!(result.raw.as_ptr(), raw[offset..].as_ptr());
    }
}

/// PreserveWidth and Exact are identities, even on all-fill or empty input.
/// Equality of pointers proves that no replacement buffer was returned.
#[test]
fn preserve_and_exact_keep_every_original_byte() {
    for rule in [
        AlignmentRule::Exact,
        AlignmentRule::LeftPad {
            fill: b'0',
            content_policy: ContentBoundaryPolicy::PreserveWidth,
        },
        AlignmentRule::RightPad {
            fill: b' ',
            content_policy: ContentBoundaryPolicy::PreserveWidth,
        },
    ] {
        for raw in [b"0000".as_slice(), b"    ", b" A ", b""] {
            let result = logical_value(&rule, FieldSlice { raw }).unwrap();
            assert_eq!(result.raw, raw);
            assert_eq!(result.raw.as_ptr(), raw.as_ptr());
        }
    }
}

/// Both directions handle all-fill and empty slices safely. There is no
/// synthetic numeric zero and no reclassification of missing/presence.
#[test]
fn all_fill_strips_to_empty_without_panics() {
    for rule in [
        AlignmentRule::RightPad {
            fill: b'0',
            content_policy: ContentBoundaryPolicy::StripTrailingFill,
        },
        AlignmentRule::LeftPad {
            fill: b'0',
            content_policy: ContentBoundaryPolicy::StripLeadingFill,
        },
    ] {
        for raw in [b"0000".as_slice(), b""] {
            assert!(logical_value(&rule, FieldSlice { raw }).unwrap().is_empty());
        }
    }
}

/// Only byte identity matters: NUL/non-UTF-8 content is preserved, and tabs or
/// NBSP are not silently treated as ASCII space. Source bytes remain unchanged.
#[test]
fn binary_data_and_other_whitespace_are_not_decoded_or_trimmed() {
    let raw = vec![0xff, 0, b' ', b' '];
    let rule = AlignmentRule::RightPad {
        fill: b' ',
        content_policy: ContentBoundaryPolicy::StripTrailingFill,
    };
    assert_eq!(
        logical_value(&rule, FieldSlice { raw: &raw }).unwrap().raw,
        &[0xff, 0]
    );
    assert_eq!(raw, [0xff, 0, b' ', b' ']);
    for value in [b"A \t".as_slice(), &[b'A', 0xc2, 0xa0]] {
        assert_eq!(
            logical_value(&rule, FieldSlice { raw: value }).unwrap().raw,
            value
        );
    }
    let nul = AlignmentRule::LeftPad {
        fill: 0,
        content_policy: ContentBoundaryPolicy::StripLeadingFill,
    };
    assert_eq!(
        logical_value(&nul, FieldSlice { raw: &[0, 0xff, 0] })
            .unwrap()
            .raw,
        &[0xff, 0]
    );
}

/// Public rules can be assembled incorrectly; report a typed configuration
/// error instead of changing the requested side or emitting FieldViolation.
#[test]
fn incompatible_policy_is_a_technical_error() {
    for rule in [
        AlignmentRule::RightPad {
            fill: b' ',
            content_policy: ContentBoundaryPolicy::StripLeadingFill,
        },
        AlignmentRule::LeftPad {
            fill: b'0',
            content_policy: ContentBoundaryPolicy::StripTrailingFill,
        },
    ] {
        let error = logical_value(&rule, FieldSlice { raw: b"ABC" }).unwrap_err();
        assert_eq!(error, LogicalValueError::IncompatibleContentPolicy);
        assert!(error.source().is_none());
        assert!(!error.to_string().contains("ABC"));
    }
}

/// The rule can leave scope before its result: the lifetime borrows only data.
/// This also verifies that metadata ownership has not leaked into the view.
#[test]
fn result_borrows_input_not_rule() {
    let raw = b"ABC  ";
    let result = {
        let rule = AlignmentRule::RightPad {
            fill: b' ',
            content_policy: ContentBoundaryPolicy::StripTrailingFill,
        };
        logical_value(&rule, FieldSlice { raw }).unwrap()
    };
    assert_eq!(result.raw, b"ABC");
    assert_eq!(result.raw.as_ptr(), raw.as_ptr());
}
