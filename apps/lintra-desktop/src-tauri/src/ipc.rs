use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value, json};
use std::fmt;
use std::io::{self, Write};

pub(crate) const MAX_BYTES: usize = 1_048_576;
const MAX_DEPTH: usize = 32;
const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErrorCode {
    UnsupportedVersion,
    InvalidMessage,
    PayloadTooLarge,
    UnknownOperation,
    Unauthorized,
    StaleContext,
    Busy,
    InvalidState,
    TechnicalFailure,
}

impl ErrorCode {
    fn wire(self) -> &'static str {
        match self {
            Self::UnsupportedVersion => "IPC_UNSUPPORTED_VERSION",
            Self::InvalidMessage => "IPC_INVALID_MESSAGE",
            Self::PayloadTooLarge => "IPC_PAYLOAD_TOO_LARGE",
            Self::UnknownOperation => "IPC_UNKNOWN_OPERATION",
            Self::Unauthorized => "IPC_UNAUTHORIZED",
            Self::StaleContext => "IPC_STALE_CONTEXT",
            Self::Busy => "IPC_BUSY",
            Self::InvalidState => "IPC_INVALID_STATE",
            Self::TechnicalFailure => "IPC_TECHNICAL_FAILURE",
        }
    }

    fn category(self) -> &'static str {
        if matches!(
            self,
            Self::UnsupportedVersion
                | Self::InvalidMessage
                | Self::PayloadTooLarge
                | Self::UnknownOperation
        ) {
            "protocol"
        } else {
            "technical"
        }
    }
}

pub(crate) struct Request {
    pub request_id: String,
    pub operation: String,
    pub payload: Map<String, Value>,
}

pub(crate) fn version(object: &Map<String, Value>) -> Result<(), ErrorCode> {
    match object.get("schemaVersion").and_then(Value::as_u64) {
        Some(1) => Ok(()),
        Some(2..=65535) => Err(ErrorCode::UnsupportedVersion),
        _ => Err(ErrorCode::InvalidMessage),
    }
}

fn check_json(text: &str) -> Result<(), ErrorCode> {
    let bytes = text.as_bytes();
    let (mut depth, mut quoted, mut escaped, mut index) = (0usize, false, false, 0);
    while index < bytes.len() {
        let byte = bytes[index];
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
        } else {
            match byte {
                b'"' => quoted = true,
                b'{' | b'[' => {
                    depth += 1;
                    if depth > MAX_DEPTH {
                        return Err(ErrorCode::InvalidMessage);
                    }
                }
                b'}' | b']' => depth = depth.checked_sub(1).ok_or(ErrorCode::InvalidMessage)?,
                b'-' | b'0'..=b'9' => {
                    let start = index;
                    while index < bytes.len() && !b" \t\r\n,]}:{[\"".contains(&bytes[index]) {
                        index += 1;
                    }
                    // Lexical check stays valid if the core later enables arbitrary_precision.
                    let value = text[start..index]
                        .parse::<i64>()
                        .map_err(|_| ErrorCode::InvalidMessage)?;
                    if !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&value) {
                        return Err(ErrorCode::InvalidMessage);
                    }
                    continue;
                }
                _ => {}
            }
        }
        index += 1;
    }
    if depth != 0 || quoted {
        return Err(ErrorCode::InvalidMessage);
    }
    Ok(())
}

struct UniqueValue(Value);
impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UniqueVisitor;
        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = Value;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("strict JSON")
            }
            fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value, E> {
                Ok(Value::Bool(value))
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Value, E> {
                Ok(value.into())
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Value, E> {
                Ok(value.into())
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Value, E> {
                Ok(value.into())
            }
            fn visit_string<E: de::Error>(self, value: String) -> Result<Value, E> {
                Ok(value.into())
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Value, A::Error> {
                let mut values = Vec::new();
                while let Some(UniqueValue(value)) = sequence.next_element()? {
                    values.push(value);
                }
                Ok(Value::Array(values))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Value, A::Error> {
                let mut values = Map::new();
                while let Some(key) = access.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom("duplicate JSON key"));
                    }
                    let UniqueValue(value) = access.next_value()?;
                    values.insert(key, value);
                }
                Ok(Value::Object(values))
            }
        }
        deserializer.deserialize_any(UniqueVisitor).map(Self)
    }
}

pub(crate) fn decode(bytes: &[u8]) -> Result<Request, ErrorCode> {
    if bytes.len() > MAX_BYTES {
        return Err(ErrorCode::PayloadTooLarge);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| ErrorCode::InvalidMessage)?;
    check_json(text)?;
    let UniqueValue(value) = serde_json::from_str(text).map_err(|_| ErrorCode::InvalidMessage)?;
    let Value::Object(mut envelope) = value else {
        return Err(ErrorCode::InvalidMessage);
    };
    version(&envelope)?;
    let fields = [
        "schemaVersion",
        "request_id",
        "kind",
        "operation",
        "payload",
    ];
    if envelope.len() != fields.len()
        || !fields.iter().all(|key| envelope.contains_key(*key))
        || envelope["kind"] != "request"
    {
        return Err(ErrorCode::InvalidMessage);
    }
    let request_id = envelope["request_id"]
        .as_str()
        .filter(|id| {
            !id.is_empty()
                && id.len() <= 64
                && id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
        })
        .ok_or(ErrorCode::InvalidMessage)?
        .to_owned();
    let operation = envelope["operation"]
        .as_str()
        .filter(|operation| {
            !operation.is_empty()
                && operation.len() <= 64
                && operation.as_bytes()[0].is_ascii_lowercase()
                && operation
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        })
        .ok_or(ErrorCode::InvalidMessage)?
        .to_owned();
    let Some(Value::Object(payload)) = envelope.remove("payload") else {
        return Err(ErrorCode::InvalidMessage);
    };
    version(&payload)?;
    Ok(Request {
        request_id,
        operation,
        payload,
    })
}

pub(crate) fn response(request_id: &str, payload: Value) -> Result<Vec<u8>, ErrorCode> {
    version(payload.as_object().ok_or(ErrorCode::TechnicalFailure)?)?;
    let mut writer = BoundedWriter(Vec::new());
    serde_json::to_writer(
        &mut writer,
        &json!({"schemaVersion":1,"request_id":request_id,"kind":"result","payload":payload}),
    )
    .map_err(|error| {
        if error.is_io() {
            ErrorCode::PayloadTooLarge
        } else {
            ErrorCode::TechnicalFailure
        }
    })?;
    let bytes = writer.0;
    // Producer mistakes must not emit a frame outside the same JSON contract.
    check_json(std::str::from_utf8(&bytes).map_err(|_| ErrorCode::TechnicalFailure)?)?;
    Ok(bytes)
}

struct BoundedWriter(Vec<u8>);

impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > MAX_BYTES - self.0.len() {
            return Err(io::Error::other("IPC response exceeds byte limit"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) fn error(request_id: Option<&str>, code: ErrorCode) -> Vec<u8> {
    serde_json::to_vec(
        &json!({"schemaVersion":1,"request_id":request_id,"kind":"error",
        "error":{"schemaVersion":1,"category":code.category(),"code":code.wire()}}),
    )
    .expect("static bounded IPC error is serializable")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(payload: Value) -> Vec<u8> {
        serde_json::to_vec(&json!({"schemaVersion":1,"request_id":"test_1","kind":"request","operation":"app_info","payload":payload})).unwrap()
    }

    #[test]
    fn raw_boundaries_apply_to_entire_request_and_response() {
        let empty = frame(json!({"schemaVersion":1,"padding":""}));
        let bytes = frame(json!({"schemaVersion":1,"padding":"x".repeat(MAX_BYTES-empty.len())}));
        assert_eq!(bytes.len(), MAX_BYTES);
        assert!(decode(&bytes).is_ok());
        assert_eq!(
            decode(&[bytes, b" ".to_vec()].concat()).err(),
            Some(ErrorCode::PayloadTooLarge)
        );
        let overhead = response("test_1", json!({"schemaVersion":1,"text":""}))
            .unwrap()
            .len();
        let bytes = response(
            "test_1",
            json!({"schemaVersion":1,"text":"x".repeat(MAX_BYTES-overhead)}),
        )
        .unwrap();
        assert_eq!(bytes.len(), MAX_BYTES);
        assert_eq!(
            response(
                "test_1",
                json!({"schemaVersion":1,"text":"x".repeat(MAX_BYTES-overhead+1)})
            ),
            Err(ErrorCode::PayloadTooLarge)
        );
    }

    #[test]
    fn depth_is_checked_before_deserialization_and_ignores_quoted_delimiters() {
        for (count, valid) in [(30, true), (31, false)] {
            let bytes = format!(
                r#"{{"schemaVersion":1,"request_id":"d","kind":"request","operation":"app_info","payload":{{"schemaVersion":1,"nested":{}0{}}}}}"#,
                "[".repeat(count),
                "]".repeat(count)
            );
            assert_eq!(decode(bytes.as_bytes()).is_ok(), valid);
        }
        assert!(decode(&frame(json!({"schemaVersion":1,"text":"{[\\\""}))).is_ok());
    }

    #[test]
    fn invalid_json_scalars_and_duplicates_never_enter_a_handler() {
        for text in [
            r#"{"schemaVersion":1,"schemaVersion":1}"#,
            r#"{"a":{"key":1,"key":2}}"#,
            r#"{"key":1,"\u006bey":2}"#,
            "\u{feff}{}",
            "{} trailing",
            "{",
            "[]",
            "{\"n\":NaN}",
            "{\"n\":1.0}",
            "{\"n\":1e0}",
            "{\"n\":9007199254740992}",
            "{\"n\":-9007199254740992}",
            "{\"s\":\"\\ud800\"}",
        ] {
            assert!(decode(text.as_bytes()).is_err(), "{text}");
        }
        assert_eq!(decode(&[0xff]).err(), Some(ErrorCode::InvalidMessage));
        assert!(decode(&frame(json!({"schemaVersion":1,"number":MAX_SAFE_INTEGER}))).is_ok());
        assert!(
            decode(&frame(
                json!({"schemaVersion":1,"number":-MAX_SAFE_INTEGER})
            ))
            .is_ok()
        );
    }

    #[test]
    fn versions_types_and_unknown_fields_are_explicit() {
        for value in [json!(0), json!(65536), json!(true), json!("1"), Value::Null] {
            assert_eq!(
                decode(&frame(json!({"schemaVersion":value}))).err(),
                Some(ErrorCode::InvalidMessage)
            );
        }
        assert_eq!(
            decode(&frame(json!({"schemaVersion":2}))).err(),
            Some(ErrorCode::UnsupportedVersion)
        );
        let mut value: Value = serde_json::from_slice(&frame(json!({"schemaVersion":1}))).unwrap();
        value["schemaVersion"] = json!(2);
        assert_eq!(
            decode(&serde_json::to_vec(&value).unwrap()).err(),
            Some(ErrorCode::UnsupportedVersion)
        );
        value["schemaVersion"] = json!(1);
        value["extra"] = json!(true);
        assert_eq!(
            decode(&serde_json::to_vec(&value).unwrap()).err(),
            Some(ErrorCode::InvalidMessage)
        );
    }

    #[test]
    fn all_error_codes_keep_protocol_and_technical_categories_separate() {
        for code in [
            ErrorCode::UnsupportedVersion,
            ErrorCode::InvalidMessage,
            ErrorCode::PayloadTooLarge,
            ErrorCode::UnknownOperation,
            ErrorCode::Unauthorized,
            ErrorCode::StaleContext,
            ErrorCode::Busy,
            ErrorCode::InvalidState,
            ErrorCode::TechnicalFailure,
        ] {
            let bytes = error(Some(&"x".repeat(64)), code);
            assert!(bytes.len() < MAX_BYTES);
            let value: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(value["error"]["code"], code.wire());
            assert_eq!(value["error"]["category"], code.category());
            assert_eq!(value["kind"], "error");
            assert!(value.get("run_status").is_none());
        }
    }
}
