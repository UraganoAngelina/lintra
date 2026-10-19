use lintra::{EncodingSpec, render_diagnostic_value};

#[test]
fn valid_utf8_preserves_case_unicode_and_significant_trailing_spaces() {
    let bytes = "Ècho  ".as_bytes();
    let value = render_diagnostic_value(bytes, &EncodingSpec::Utf8);
    assert_eq!(value.display, "Ècho  ");
    assert_eq!(value.raw_hex, None);
    let literal_replacement = render_diagnostic_value("�".as_bytes(), &EncodingSpec::Utf8);
    assert_eq!(literal_replacement.display, "�");
    assert_eq!(literal_replacement.raw_hex, None);
}
#[test]
fn invalid_utf8_has_a_readable_display_and_exact_deterministic_hex() {
    let value = render_diagnostic_value(b"A\xff \xc3", &EncodingSpec::Utf8);
    assert_eq!(value.display, "A� �");
    assert_eq!(value.raw_hex.as_deref(), Some("41 FF 20 C3"));
}
#[test]
fn control_characters_and_backslashes_are_visible_without_conflating_literal_escapes() {
    let value = render_diagnostic_value(b"A\\\n\r\t\0\x7f ", &EncodingSpec::Utf8);
    assert_eq!(value.display, r"A\\\n\r\t\0\u{7f} ");
    assert_eq!(value.raw_hex.as_deref(), Some("41 5C 0A 0D 09 00 7F 20"));
    let literal = render_diagnostic_value(br"\n", &EncodingSpec::Utf8);
    assert_eq!(literal.display, r"\\n");
    assert_eq!(literal.raw_hex.as_deref(), Some("5C 6E"));
}
#[test]
fn windows_1252_follows_the_official_special_byte_mapping_and_preserves_undefined_bytes() {
    let special: Vec<u8> = (0x80..=0x9f).collect();
    let value = render_diagnostic_value(&special, &EncodingSpec::Windows1252);
    assert_eq!(value.display, "€�‚ƒ„…†‡ˆ‰Š‹Œ�Ž��‘’“”•–—˜™š›œ�žŸ");
    assert_eq!(
        value.raw_hex.as_deref(),
        Some(
            "80 81 82 83 84 85 86 87 88 89 8A 8B 8C 8D 8E 8F 90 91 92 93 94 95 96 97 98 99 9A 9B 9C 9D 9E 9F"
        )
    );
    let plain = render_diagnostic_value(b"\x80\x93\xe9\x94 ", &EncodingSpec::Windows1252);
    assert_eq!(plain.display, "€“é” ");
    assert_eq!(plain.raw_hex, None);
}
#[test]
fn latin1_and_unknown_profiles_are_explicit_and_never_fall_back_to_utf8() {
    let latin = render_diagnostic_value(b"\xe9\xff", &EncodingSpec::Iso8859_1);
    assert_eq!(latin.display, "éÿ");
    assert_eq!(latin.raw_hex, None);
    let latin_control = render_diagnostic_value(&[0x80], &EncodingSpec::Iso8859_1);
    assert_eq!(latin_control.display, r"\u{80}");
    assert_eq!(latin_control.raw_hex.as_deref(), Some("80"));
    let unknown = render_diagnostic_value(b"A\xff\n\\x", &EncodingSpec::Other("UTF-8".into()));
    assert_eq!(unknown.display, r"A\xff\n\\x");
    assert_eq!(unknown.raw_hex.as_deref(), Some("41 FF 0A 5C 78"));
}
#[test]
fn empty_diagnostics_are_stable_for_each_profile() {
    for encoding in [
        EncodingSpec::Utf8,
        EncodingSpec::Windows1252,
        EncodingSpec::Iso8859_1,
    ] {
        let value = render_diagnostic_value(&[], &encoding);
        assert!(value.display.is_empty());
        assert_eq!(value.raw_hex, None);
    }
    let unknown = render_diagnostic_value(&[], &EncodingSpec::Other("synthetic".into()));
    assert!(unknown.display.is_empty());
    assert_eq!(unknown.raw_hex.as_deref(), Some(""));
}
