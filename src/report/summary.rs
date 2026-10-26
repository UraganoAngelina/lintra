use super::sink::ReportError;
use crate::domain::{
    InputAuditCompletion, RunStatus, ScopeSummary, ValidationMode, ValidationSummary,
};
use chrono::{DateTime, Datelike, SecondsFormat, Utc};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize)]
struct SummaryDocument<'a> {
    #[serde(flatten)]
    bundle: Option<&'a crate::BundleAudit>,
    result: &'static str,
    validation_mode: ValidationMode,
    flow: &'a str,
    schema_version: &'a str,
    scope: ScopeDocument<'a>,
    records_processed: u64,
    valid_records: u64,
    invalid_records: u64,
    total_errors: u64,
    errors_by_code: BTreeMap<&'a str, u64>,
    skipped_rules: u64,
    truncated: bool,
    validator_version: &'a str,
    schema_hash: &'a str,
    reference_data_versions: BTreeMap<&'a str, &'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference_validation: Option<&'a crate::ReferenceValidationSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    full_validation: Option<&'a crate::FullValidationSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    completeness: Option<crate::ValidationCompleteness>,
    started_at: String,
    elapsed: Elapsed,
    inputs: Vec<InputDocument<'a>>,
}
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ScopeDocument<'a> {
    Trace { trace_id: &'a str },
    Flow { trace_ids: Vec<&'a str> },
}
#[derive(Serialize)]
struct Elapsed {
    seconds: u64,
    nanoseconds: u32,
}
#[derive(Serialize)]
struct InputDocument<'a> {
    trace_id: &'a str,
    file_name: &'a str,
    file_size_bytes: u64,
    bytes_read: u64,
    completion: CompletionDocument,
}
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum CompletionDocument {
    Complete { sha256: String },
    Incomplete,
}

fn utc_timestamp(value: SystemTime) -> Result<String, ReportError> {
    let invalid = || ReportError::InvalidTimestamp;
    let (seconds, nanos) = match value.duration_since(UNIX_EPOCH) {
        Ok(duration) => (
            i64::try_from(duration.as_secs()).map_err(|_| invalid())?,
            duration.subsec_nanos(),
        ),
        Err(error) => {
            let duration = error.duration();
            let seconds = i64::try_from(duration.as_secs())
                .map_err(|_| invalid())?
                .checked_neg()
                .ok_or_else(invalid)?;
            if duration.subsec_nanos() == 0 {
                (seconds, 0)
            } else {
                (
                    seconds.checked_sub(1).ok_or_else(invalid)?,
                    1_000_000_000 - duration.subsec_nanos(),
                )
            }
        }
    };
    let timestamp = DateTime::<Utc>::from_timestamp(seconds, nanos).ok_or_else(invalid)?;
    if !(0..=9999).contains(&timestamp.year()) {
        return Err(invalid());
    }
    Ok(timestamp.to_rfc3339_opts(SecondsFormat::AutoSi, true))
}

/// Writes the separate errors.summary.json document into a caller-owned writer.
/// Captures no host clock/timezone. All checks precede the first output byte.
/// Consume metadata in the builder; preserve its Copy started_at for this call.
/// File publication/rollback remains the responsibility of T-P5-06.
pub fn write_summary<W: Write>(
    writer: W,
    summary: &ValidationSummary,
    started_at: SystemTime,
) -> Result<W, ReportError> {
    write_summary_document(writer, summary, started_at, None)
}

fn write_summary_document<W: Write>(
    mut writer: W,
    summary: &ValidationSummary,
    started_at: SystemTime,
    bundle: Option<&crate::BundleAudit>,
) -> Result<W, ReportError> {
    summary.validate().map_err(ReportError::InvalidSummary)?;
    let started_at = utc_timestamp(started_at)?;
    let scope = match &summary.scope_summary {
        ScopeSummary::Trace { trace_id } => ScopeDocument::Trace {
            trace_id: &trace_id.0,
        },
        ScopeSummary::Flow { trace_ids } => ScopeDocument::Flow {
            trace_ids: trace_ids.iter().map(|id| id.0.as_str()).collect(),
        },
    };
    let mut errors_by_code = BTreeMap::new();
    for (code, &count) in &summary.errors_by_code {
        let symbol = code.as_symbol().map_err(|e| {
            ReportError::InvalidSummary(crate::domain::SummaryBuildError::InvalidCode(e))
        })?;
        errors_by_code.insert(symbol, count);
    }
    let document = SummaryDocument {
        bundle,
        result: match summary.run_status {
            RunStatus::CompletedValid => "VALID",
            RunStatus::CompletedInvalid => "INVALID",
            RunStatus::Cancelled => "CANCELLED",
        },
        validation_mode: summary.validation_mode,
        flow: &summary.schema_key.flow_id.0,
        schema_version: &summary.schema_key.version.0,
        scope,
        records_processed: summary.records_processed,
        valid_records: summary.valid_records,
        invalid_records: summary.invalid_records,
        total_errors: summary.total_errors,
        errors_by_code,
        skipped_rules: summary.skipped_rules,
        truncated: summary.truncated,
        validator_version: &summary.validator_version,
        schema_hash: &summary.schema_hash,
        reference_data_versions: summary
            .reference_data_versions
            .iter()
            .map(|(id, version)| (id.0.as_str(), version.0.as_str()))
            .collect(),
        reference_validation: summary.reference_validation.as_ref(),
        full_validation: summary.full_validation.as_ref(),
        completeness: summary
            .full_validation
            .as_ref()
            .map(|_| summary.completeness()),
        started_at,
        elapsed: Elapsed {
            seconds: summary.elapsed.as_secs(),
            nanoseconds: summary.elapsed.subsec_nanos(),
        },
        inputs: summary
            .inputs
            .iter()
            .map(|input| InputDocument {
                trace_id: &input.trace_id.0,
                file_name: &input.file_name,
                file_size_bytes: input.file_size_bytes,
                bytes_read: input.bytes_read,
                completion: match &input.completion {
                    InputAuditCompletion::Complete { sha256 } => CompletionDocument::Complete {
                        sha256: sha256.to_string(),
                    },
                    InputAuditCompletion::Incomplete => CompletionDocument::Incomplete,
                },
            })
            .collect(),
    };
    serde_json::to_writer(&mut writer, &document).map_err(ReportError::Serialize)?;
    writer.write_all(b"\n").map_err(ReportError::Write)?;
    writer.flush().map_err(ReportError::Write)?;
    Ok(writer)
}

/// Serialize a service outcome using the timestamp retained from RunMetadata.
/// Unlike the legacy explicit-timestamp helper, no second clock is required.
pub fn write_run_summary<W: Write>(
    writer: W,
    summary: &ValidationSummary,
) -> Result<W, ReportError> {
    write_summary(writer, summary, summary.started_at)
}

/// Add bundle provenance only when supplied by a selecting adapter. Legacy
/// writers retain their exact document shape for callers with opaque repositories.
pub fn write_run_summary_with_bundle<W: Write>(
    writer: W,
    summary: &ValidationSummary,
    bundle: &crate::BundleAudit,
) -> Result<W, ReportError> {
    write_summary_document(writer, summary, summary.started_at, Some(bundle))
}
