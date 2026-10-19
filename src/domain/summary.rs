//! Neutral run results and audit metadata. No execution or persistence policy.

use super::{DatasetId, DatasetVersion, FlowId, SchemaVersion, TraceId, ValidationCode};
use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime},
};

/// Flow schema identity; a trace selection is deliberately outside this key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaKey {
    pub flow_id: FlowId,
    pub version: SchemaVersion,
}

/// Only identifiers in the selected scope, preserving the supplied order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeSummary {
    Trace { trace_id: TraceId },
    Flow { trace_ids: Vec<TraceId> },
}

/// A completed validation or a controlled cancellation, never a technical error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    CompletedValid,
    CompletedInvalid,
    Cancelled,
}

/// Audit metadata supplied by the application; no implicit clock or hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunMetadata {
    pub validator_version: String,
    pub schema_hash: String,
    pub reference_data_versions: BTreeMap<DatasetId, DatasetVersion>,
    pub started_at: SystemTime,
}

/// Counters and metadata for the actual extent of a validation run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationSummary {
    pub run_status: RunStatus,
    pub schema_key: SchemaKey,
    pub scope_summary: ScopeSummary,
    pub records_processed: u64,
    pub valid_records: u64,
    pub invalid_records: u64,
    pub total_errors: u64,
    pub errors_by_code: BTreeMap<ValidationCode, u64>,
    pub skipped_rules: u64,
    /// True when validation stopped at an operational limit.
    pub truncated: bool,
    pub validator_version: String,
    pub schema_hash: String,
    pub reference_data_versions: BTreeMap<DatasetId, DatasetVersion>,
    pub elapsed: Duration,
}

/// Neutral controlled outcome. Technical failures belong to the application boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationRunOutcome {
    pub status: RunStatus,
    pub summary: ValidationSummary,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn schema_keys_distinguish_flow_and_version_without_trace_identity() {
        let key = SchemaKey {
            flow_id: FlowId("EXAMPLE".into()),
            version: SchemaVersion("v1".into()),
        };
        let mut keys = HashSet::from([key.clone()]);
        assert!(!keys.insert(key.clone()));
        assert!(keys.insert(SchemaKey {
            version: SchemaVersion("v2".into()),
            ..key.clone()
        }));
        assert!(keys.insert(SchemaKey {
            flow_id: FlowId("OTHER".into()),
            ..key
        }));
    }

    #[test]
    fn summary_preserves_scope_order_and_orders_dataset_and_error_maps() {
        let metadata = RunMetadata {
            validator_version: "synthetic-build".into(),
            schema_hash: "synthetic-hash-not-computed-by-this-test".into(),
            reference_data_versions: BTreeMap::from([
                (DatasetId("Z".into()), DatasetVersion("v2".into())),
                (DatasetId("A".into()), DatasetVersion("v1".into())),
            ]),
            started_at: SystemTime::UNIX_EPOCH,
        };
        let summary = ValidationSummary {
            run_status: RunStatus::Cancelled,
            schema_key: SchemaKey {
                flow_id: FlowId("EXAMPLE".into()),
                version: SchemaVersion("v1".into()),
            },
            scope_summary: ScopeSummary::Flow {
                trace_ids: vec![TraceId("Z".into()), TraceId("A".into())],
            },
            records_processed: 3,
            valid_records: 1,
            invalid_records: 2,
            total_errors: 4,
            errors_by_code: BTreeMap::from([
                (ValidationCode::InvalidType, 3),
                (ValidationCode::RequiredField, 1),
            ]),
            skipped_rules: 2,
            truncated: true,
            validator_version: metadata.validator_version.clone(),
            schema_hash: metadata.schema_hash.clone(),
            reference_data_versions: metadata.reference_data_versions.clone(),
            elapsed: Duration::from_millis(12),
        };
        let outcome = ValidationRunOutcome {
            status: RunStatus::Cancelled,
            summary,
        };
        assert_eq!(outcome.summary.run_status, outcome.status);
        assert!(outcome.summary.truncated);
        assert_eq!(outcome.summary.schema_hash, metadata.schema_hash);
        assert_eq!(metadata.started_at, SystemTime::UNIX_EPOCH);
        assert_eq!(
            outcome.summary.scope_summary,
            ScopeSummary::Flow {
                trace_ids: vec![TraceId("Z".into()), TraceId("A".into())]
            }
        );
        assert_eq!(
            outcome
                .summary
                .reference_data_versions
                .keys()
                .map(|id| id.0.as_str())
                .collect::<Vec<_>>(),
            ["A", "Z"]
        );
        assert_eq!(
            outcome
                .summary
                .errors_by_code
                .into_iter()
                .collect::<Vec<_>>(),
            [
                (ValidationCode::RequiredField, 1),
                (ValidationCode::InvalidType, 3)
            ]
        );
    }
}
