//! Distinct identifiers preserve the exact spelling supplied by the schema.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct TraceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct FieldId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaVersion(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct DatasetId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct DatasetVersion(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JoinId(pub String);

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn identifiers_preserve_spelling_and_support_value_semantics() {
        macro_rules! check {
            ($($kind:ident),+ $(,)?) => {$(
                let value = $kind(" 050.aA ".to_owned());
                assert_eq!(value.0, " 050.aA ");
                assert_eq!(value, value.clone());
                assert_ne!(value, $kind("050.AA".to_owned()));
                let mut set = HashSet::new();
                assert!(set.insert(value.clone()));
                assert!(!set.insert(value));
            )+};
        }
        check!(
            FlowId,
            TraceId,
            FieldId,
            SchemaVersion,
            DatasetId,
            DatasetVersion,
            JoinId
        );
    }
}
