use lintra::{MissingValuePolicy, is_missing};

#[test]
fn spaces_means_only_ascii_spaces_without_trimming_or_decoding() {
    for raw in [b"".as_slice(), b" ", b"   "] {
        assert!(is_missing(&MissingValuePolicy::Spaces, raw));
    }
    for raw in [
        b"000".as_slice(),
        b" A ",
        b"\t",
        b"\r\n",
        b"\0",
        b"\xc2\xa0",
        b"\xff",
    ] {
        assert!(!is_missing(&MissingValuePolicy::Spaces, raw));
    }
}

#[test]
fn exact_requires_every_byte_and_width_and_can_explicitly_declare_zero_missing() {
    let policy = MissingValuePolicy::Exact(b"000".to_vec());
    assert!(is_missing(&policy, b"000"));
    for raw in [
        b"".as_slice(),
        b"0",
        b"00",
        b"0000",
        b" 000",
        b"000 ",
        b"   ",
        b"001",
    ] {
        assert!(!is_missing(&policy, raw));
    }
    assert!(is_missing(&MissingValuePolicy::Exact(vec![]), b""));
    assert!(!is_missing(&MissingValuePolicy::Exact(vec![]), b" "));
    assert!(is_missing(
        &MissingValuePolicy::Exact(vec![0, 0xff]),
        &[0, 0xff]
    ));
}

#[test]
fn any_of_matches_only_explicit_full_representations_including_binary() {
    let policy =
        MissingValuePolicy::AnyOf(vec![b"   ".to_vec(), b"---".to_vec(), vec![0, 0xff, 0]]);
    for raw in [b"   ".as_slice(), b"---", &[0, 0xff, 0]] {
        assert!(is_missing(&policy, raw));
    }
    for raw in [b"".as_slice(), b"--", b"----", b"000", b" - "] {
        assert!(!is_missing(&policy, raw));
    }
    assert!(!is_missing(&MissingValuePolicy::AnyOf(vec![]), b""));
    assert!(is_missing(&MissingValuePolicy::AnyOf(vec![vec![]]), b""));
}

#[test]
fn never_missing_has_no_implicit_empty_space_zero_or_binary_exception() {
    for raw in [b"".as_slice(), b"   ", b"000", b"ABC", &[0, 0xff]] {
        assert!(!is_missing(&MissingValuePolicy::NeverMissing, raw));
    }
}
