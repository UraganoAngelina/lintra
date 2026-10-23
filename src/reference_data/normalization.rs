//! Explicit query transformations. Dataset keys and audit bytes stay unchanged.
use std::borrow::Cow;

/// A caller must choose the policy; there is deliberately no implicit Default.
/// This technical API does not add a property to flow grammar v1 or v2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupNormalization {
    Exact,
    /// Only ASCII a..z become A..Z. No Unicode decoding, trimming or zero removal.
    AsciiUppercase,
}

impl LookupNormalization {
    /// Borrow unchanged input. Allocate only when the explicit transformation
    /// actually changes a byte; applying the policy never mutates the source.
    pub fn apply(self, value: &[u8]) -> Cow<'_, [u8]> {
        match self {
            Self::Exact => Cow::Borrowed(value),
            Self::AsciiUppercase if value.iter().any(u8::is_ascii_lowercase) => {
                Cow::Owned(value.to_ascii_uppercase())
            }
            Self::AsciiUppercase => Cow::Borrowed(value),
        }
    }
}
