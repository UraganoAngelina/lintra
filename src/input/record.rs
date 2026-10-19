//! A borrowed raw record, with no decoding or validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawRecord<'a> {
    pub line_number: u64,
    pub bytes: &'a [u8],
}

use crate::schema::compiled::CompiledFieldSpec;

/// A zero-copy view over the original physical record bytes.
#[derive(Debug, Clone, Copy)]
pub struct RecordView<'a> {
    raw: &'a [u8],
}
impl<'a> RecordView<'a> {
    pub fn new(raw: &'a [u8]) -> Self {
        Self { raw }
    }
    pub fn raw(&self) -> &'a [u8] {
        self.raw
    }
    pub fn field(&self, spec: &CompiledFieldSpec) -> Option<FieldSlice<'a>> {
        spec.range().slice(self.raw).map(|raw| FieldSlice { raw })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldSlice<'a> {
    pub raw: &'a [u8],
}
impl<'a> FieldSlice<'a> {
    pub fn as_bytes(&self) -> &'a [u8] {
        self.raw
    }
    pub fn len(&self) -> usize {
        self.raw.len()
    }
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }
    pub fn is_ascii(&self) -> bool {
        self.raw.is_ascii()
    }
}
