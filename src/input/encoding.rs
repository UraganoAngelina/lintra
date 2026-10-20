//! Diagnostic-only rendering. Never changes the byte-oriented validation input.
pub use crate::domain::DiagnosticValue;
use crate::schema::compiled::EncodingSpec;

// Unicode/Microsoft mapping 2.01, CP1252.TXT (1998-04-15):
// https://www.unicode.org/Public/MAPPINGS/VENDORS/MICSFT/WINDOWS/CP1252.TXT
// Undefined entries are retained as replacement + exact raw hex, not guessed.
const CP1252_SPECIAL: [Option<char>; 32] = [
    Some('\u{20ac}'),
    None,
    Some('\u{201a}'),
    Some('\u{0192}'),
    Some('\u{201e}'),
    Some('\u{2026}'),
    Some('\u{2020}'),
    Some('\u{2021}'),
    Some('\u{02c6}'),
    Some('\u{2030}'),
    Some('\u{0160}'),
    Some('\u{2039}'),
    Some('\u{0152}'),
    None,
    Some('\u{017d}'),
    None,
    None,
    Some('\u{2018}'),
    Some('\u{2019}'),
    Some('\u{201c}'),
    Some('\u{201d}'),
    Some('\u{2022}'),
    Some('\u{2013}'),
    Some('\u{2014}'),
    Some('\u{02dc}'),
    Some('\u{2122}'),
    Some('\u{0161}'),
    Some('\u{203a}'),
    Some('\u{0153}'),
    None,
    Some('\u{017e}'),
    Some('\u{0178}'),
];

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789ABCDEF";
    let mut output = String::new();
    for (index, &byte) in bytes.iter().enumerate() {
        if index > 0 {
            output.push(' ');
        }
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 15)]));
    }
    output
}
fn escaped_display(text: &str) -> (String, bool) {
    let mut display = String::new();
    let mut changed = false;
    for ch in text.chars() {
        let escape = match ch {
            '\\' => Some("\\\\"),
            '\n' => Some("\\n"),
            '\r' => Some("\\r"),
            '\t' => Some("\\t"),
            '\0' => Some("\\0"),
            _ => None,
        };
        if let Some(escape) = escape {
            display.push_str(escape);
            changed = true;
        } else if ch.is_control() {
            display.extend(ch.escape_default());
            changed = true;
        } else {
            display.push(ch);
        }
    }
    (display, changed)
}

/// Rendering is explicitly selected by the caller and intended for diagnostics.
/// Controls/backslashes are escaped; lossy or escaped values include exact hex.
/// Unknown profiles use byte escapes and hex instead of silently decoding as UTF-8.
pub fn render_diagnostic_value(bytes: &[u8], encoding: &EncodingSpec) -> DiagnosticValue {
    let (decoded, lossy) = match encoding {
        EncodingSpec::Utf8 => match std::str::from_utf8(bytes) {
            Ok(text) => (text.to_owned(), false),
            Err(_) => (String::from_utf8_lossy(bytes).into_owned(), true),
        },
        EncodingSpec::Iso8859_1 => (bytes.iter().copied().map(char::from).collect(), false),
        EncodingSpec::Windows1252 => {
            let mut text = String::new();
            let mut lossy = false;
            for &byte in bytes {
                if (0x80..=0x9f).contains(&byte) {
                    match CP1252_SPECIAL[usize::from(byte - 0x80)] {
                        Some(ch) => text.push(ch),
                        None => {
                            text.push('\u{fffd}');
                            lossy = true;
                        }
                    }
                } else {
                    text.push(char::from(byte));
                }
            }
            (text, lossy)
        }
        EncodingSpec::Other(_) => {
            return DiagnosticValue {
                display: bytes
                    .iter()
                    .flat_map(|byte| std::ascii::escape_default(*byte))
                    .map(char::from)
                    .collect(),
                raw_hex: Some(hex(bytes)),
            };
        }
    };
    let (display, escaped) = escaped_display(&decoded);
    DiagnosticValue {
        display,
        raw_hex: (lossy || escaped).then(|| hex(bytes)),
    }
}
