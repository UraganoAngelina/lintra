//! Strict v1 decoding. No repair, last-wins map or selective recovery.
use super::evidence::{MembershipFormatIssue as Issue, MembershipProperty as Property};
use serde::de::{DeserializeSeed, Error as _, MapAccess, Visitor};
use std::{collections::HashSet, fmt, sync::Arc};

pub(super) struct MembershipDocument {
    pub format: u32,
    pub id: String,
    pub version: String,
    pub keys: Vec<String>,
}
pub(super) struct ParseFailure {
    pub issue: Issue,
    pub source: Arc<serde_json::Error>,
}
struct Root<'a>(&'a mut Option<Issue>);
impl<'de> DeserializeSeed<'de> for Root<'_> {
    type Value = MembershipDocument;
    fn deserialize<D: serde::Deserializer<'de>>(self, parser: D) -> Result<Self::Value, D::Error> {
        parser.deserialize_map(self)
    }
}
impl<'de> Visitor<'de> for Root<'_> {
    type Value = MembershipDocument;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a closed membership v1 object")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let (mut format, mut id, mut version, mut keys) = (None, None, None, None);
        let mut seen = HashSet::new();
        while let Some(name) = map.next_key::<String>()? {
            if !seen.insert(name.clone()) {
                *self.0 = Some(Issue::DuplicateProperty(name));
                return Err(M::Error::custom("duplicate membership property"));
            }
            let property = match name.as_str() {
                "dataset_format_version" => Property::FormatVersion,
                "dataset_id" => Property::DatasetId,
                "version" => Property::Version,
                "keys_hex" => Property::KeysHex,
                _ => {
                    *self.0 = Some(Issue::UnknownProperty(name));
                    return Err(M::Error::custom("unknown membership property"));
                }
            };
            *self.0 = Some(Issue::InvalidProperty(property));
            match property {
                Property::FormatVersion => format = Some(map.next_value()?),
                Property::DatasetId => id = Some(map.next_value()?),
                Property::Version => version = Some(map.next_value()?),
                Property::KeysHex => keys = Some(map.next_value()?),
            }
            *self.0 = None;
        }
        let mut required = |property| {
            *self.0 = Some(Issue::MissingProperty(property));
            M::Error::custom("missing membership property")
        };
        Ok(MembershipDocument {
            format: format.ok_or_else(|| required(Property::FormatVersion))?,
            id: id.ok_or_else(|| required(Property::DatasetId))?,
            version: version.ok_or_else(|| required(Property::Version))?,
            keys: keys.ok_or_else(|| required(Property::KeysHex))?,
        })
    }
}
pub(super) fn parse(bytes: &[u8]) -> Result<MembershipDocument, ParseFailure> {
    let mut issue = None;
    let mut parser = serde_json::Deserializer::from_slice(bytes);
    let result = Root(&mut issue)
        .deserialize(&mut parser)
        .and_then(|document| {
            parser.end()?;
            Ok(document)
        });
    result.map_err(|source| ParseFailure {
        issue: if source.is_syntax() || source.is_eof() {
            Issue::InvalidJson
        } else {
            issue.unwrap_or(Issue::InvalidJson)
        },
        source: Arc::new(source),
    })
}

pub(super) fn decode_keys(keys: Vec<String>) -> Result<HashSet<Vec<u8>>, Issue> {
    let mut values = HashSet::with_capacity(keys.len());
    for (entry, text) in keys.into_iter().enumerate() {
        let invalid = || Issue::InvalidHex { entry };
        let (pairs, remainder) = text.as_bytes().as_chunks::<2>();
        if !remainder.is_empty() {
            return Err(invalid());
        }
        let value = pairs
            .iter()
            .map(|pair| {
                let high = nibble(pair[0]).ok_or_else(invalid)?;
                let low = nibble(pair[1]).ok_or_else(invalid)?;
                Ok((high << 4) | low)
            })
            .collect::<Result<Vec<_>, Issue>>()?;
        if !values.insert(value) {
            return Err(Issue::DuplicateEntry { entry });
        }
    }
    Ok(values)
}
fn nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}
