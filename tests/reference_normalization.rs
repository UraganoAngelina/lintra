//! Explicit technical normalization; no implicit change to normative codes.
use lintra::*;
use std::borrow::Cow;

#[test]
fn exact_policy_borrows_and_preserves_every_byte() {
    let value = b" 00ab\0\xff ";
    let result = LookupNormalization::Exact.apply(value);
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result.as_ref().as_ptr(), value.as_ptr());
    assert_eq!(result.as_ref(), value);
}

#[test]
fn explicit_ascii_uppercase_changes_only_lowercase_ascii() {
    let value = b" 00aZz\0\xff\xc3\xa9 ";
    let result = LookupNormalization::AsciiUppercase.apply(value);
    assert!(matches!(result, Cow::Owned(_)));
    assert_eq!(result.as_ref(), b" 00AZZ\0\xff\xc3\xa9 ");
    assert_eq!(value, b" 00aZz\0\xff\xc3\xa9 ");
    assert_eq!(result.len(), value.len());
}

#[test]
fn unchanged_uppercase_empty_and_binary_queries_are_borrowed() {
    for value in [b"".as_slice(), b"00AZ ", &[0, 255, 128]] {
        assert!(matches!(
            LookupNormalization::AsciiUppercase.apply(value),
            Cow::Borrowed(_)
        ));
    }
}

#[test]
fn same_query_requires_explicit_policy_and_preserves_zeros_and_width() {
    let key = DatasetKey {
        id: DatasetId("synthetic".into()),
        version: DatasetVersion("1".into()),
    };
    let mut provider = InMemoryReferenceDataProvider::new();
    provider.insert(key.clone(), [b"00AB ".to_vec()]).unwrap();
    let set = provider.load(&key).unwrap();
    let query = b"00ab ";
    assert!(!set.contains(query));
    assert!(!set.contains(LookupNormalization::Exact.apply(query).as_ref()));
    assert!(set.contains(LookupNormalization::AsciiUppercase.apply(query).as_ref()));
    for other in [b"AB ".as_slice(), b"00ab", b" 00ab ", b"000ab "] {
        assert!(!set.contains(LookupNormalization::AsciiUppercase.apply(other).as_ref()));
    }
    // Query normalization does not rewrite the stored spelling or future calls.
    assert!(set.contains(b"00AB "));
    assert!(!set.contains(query));
}

#[test]
fn normalization_is_idempotent_for_all_byte_values() {
    let bytes: Vec<u8> = (0..=255).collect();
    for policy in [
        LookupNormalization::Exact,
        LookupNormalization::AsciiUppercase,
    ] {
        let once = policy.apply(&bytes);
        let twice = policy.apply(&once);
        assert_eq!(once.as_ref(), twice.as_ref());
        assert_eq!(once.len(), 256);
        assert!(matches!(twice, Cow::Borrowed(_)));
    }
}
