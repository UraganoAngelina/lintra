//! Diagnostic identity values; schema bindings decide where they come from.

use std::{collections::BTreeMap, sync::Arc};

/// Optional diagnostic identity, independent of a source field or flow policy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecordIdentity {
    pub sioc_record_id: Option<Arc<str>>,
    pub fiscal_code: Option<Arc<str>>,
    pub episode_id: Option<Arc<str>>,
    pub technical_ids: BTreeMap<Arc<str>, Arc<str>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_identity_remains_absent_and_technical_ids_are_ordered() {
        let mut identity = RecordIdentity::default();
        assert!(identity.sioc_record_id.is_none());
        assert!(identity.fiscal_code.is_none());
        assert!(identity.episode_id.is_none());
        assert!(identity.technical_ids.is_empty());
        identity
            .technical_ids
            .insert("Z".into(), "SYNTHETIC-Z".into());
        identity
            .technical_ids
            .insert("A".into(), "SYNTHETIC-A".into());
        assert_eq!(
            identity
                .technical_ids
                .keys()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>(),
            ["A", "Z"]
        );
        let cloned = identity.clone();
        assert!(Arc::ptr_eq(
            &identity.technical_ids["A"],
            &cloned.technical_ids["A"]
        ));
    }
}
