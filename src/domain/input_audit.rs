use super::TraceId;
use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sha256Digest(pub [u8; 32]);
impl serde::Serialize for Sha256Digest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
/// Decode only the fixed-size audit representation; no lossy parsing/default.
impl<'de> serde::Deserialize<'de> for Sha256Digest {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        let text = <String as serde::Deserialize>::deserialize(decoder)?;
        if text.len() != 64 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(serde::de::Error::custom(
                "expected a 64-character SHA256 digest",
            ));
        }
        let mut bytes = [0; 32];
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)
                .map_err(|_| serde::de::Error::custom("invalid SHA256 digest"))?;
        }
        Ok(Self(bytes))
    }
}
impl fmt::Display for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAuditCompletion {
    Complete { sha256: Sha256Digest },
    Incomplete,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputAudit {
    pub trace_id: TraceId,
    pub file_name: String,
    pub file_size_bytes: u64,
    pub bytes_read: u64,
    pub completion: InputAuditCompletion,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidInputAudit {
    EmptyTraceId,
    InvalidFileName,
    CompleteSizeMismatch,
}
impl fmt::Display for InvalidInputAudit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyTraceId => "input audit trace ID is empty",
            Self::InvalidFileName => {
                "input audit requires a UTF-8 basename without path components"
            }
            Self::CompleteSizeMismatch => "complete input audit disagrees with initial file size",
        })
    }
}
impl Error for InvalidInputAudit {}
impl InputAudit {
    pub fn validate(&self) -> Result<(), InvalidInputAudit> {
        if self.trace_id.0.is_empty() {
            return Err(InvalidInputAudit::EmptyTraceId);
        }
        let name = &self.file_name;
        if name.is_empty()
            || name == "."
            || name == ".."
            || name.contains(['/', '\\', '\0'])
            || (name.as_bytes().get(1) == Some(&b':') && name.as_bytes()[0].is_ascii_alphabetic())
        {
            return Err(InvalidInputAudit::InvalidFileName);
        }
        if matches!(self.completion, InputAuditCompletion::Complete { .. })
            && self.bytes_read != self.file_size_bytes
        {
            return Err(InvalidInputAudit::CompleteSizeMismatch);
        }
        Ok(())
    }
}
