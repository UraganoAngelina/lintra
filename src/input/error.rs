use std::{error::Error, fmt, io};

/// Recognized Unicode byte signatures; recognition does not select an encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BomKind {
    Utf8,
    Utf16Le,
    Utf16Be,
    Utf32Le,
    Utf32Be,
}
impl BomKind {
    pub(crate) fn bytes(self) -> &'static [u8] {
        match self {
            Self::Utf8 => b"\xef\xbb\xbf",
            Self::Utf16Le => b"\xff\xfe",
            Self::Utf16Be => b"\xfe\xff",
            Self::Utf32Le => b"\xff\xfe\x00\x00",
            Self::Utf32Be => b"\x00\x00\xfe\xff",
        }
    }
}
impl fmt::Display for BomKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Utf8 => "UTF-8",
            Self::Utf16Le => "UTF-16LE",
            Self::Utf16Be => "UTF-16BE",
            Self::Utf32Le => "UTF-32LE",
            Self::Utf32Be => "UTF-32BE",
        })
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum InputError {
    Read(io::Error),
    BomRejected(BomKind),
    LineNumberOverflow,
    PreviousFailure,
}
impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(_) => f.write_str("cannot read input record"),
            Self::BomRejected(kind) => write!(
                f,
                "input begins with a {kind} BOM; policy Reject forbids it; explicitly select BomPolicy::StripKnownBom to remove the initial marker"
            ),
            Self::LineNumberOverflow => f.write_str("input record number exceeds u64::MAX"),
            Self::PreviousFailure => {
                f.write_str("record reader cannot resume after a previous failure")
            }
        }
    }
}
impl Error for InputError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read(source) => Some(source),
            _ => None,
        }
    }
}
