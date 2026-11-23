//! Selected diagnostic, field guide and captured domains (T-MIG-P4-09).
//!
//! Contract: `docs/migration/ipc-v1/DETAIL.md` (IPC v1, ratified 14 Nov 2026).
//!
//! The renderer selects a `diagnostic_ref` of a retained page of the current
//! query. Rust resolves it to the row position registered with that page,
//! rereads the canonical diagnostic from the private spool and keeps that one
//! `ValidationError` pinned. The selection survives page eviction but not a
//! new query, a new published report or a new selection. Every text crosses
//! the IPC boundary only as a 256-byte preview or as an explicitly requested
//! chunk of at most 16 KiB; nothing is trimmed or normalized.
//!
//! Field coverage, guides and domains come exclusively from the compiled
//! schema and the reference evidence captured with the report (`Captured` in
//! `jobs.rs`), never from the current form or a reloaded catalog. Field values
//! (actual and context) are exposed only for fields that are a proper subset
//! of the record; a value covering the whole record stays in the local report.
//! Reference datasets are paged on a worker thread, never on the IPC thread.
use super::{Query, canonical_u64, identifier, nonce, preview};
use crate::{
    configuration::fields,
    ipc::{self, ErrorCode},
    jobs::Report,
};
use lintra::{
    CompiledAssertion, CompiledConstraint, CompiledFieldSpec, CompiledFlowSchema,
    DatasetAvailability, DatasetKey, DomainPosition, FieldId, LookupPage,
    PreparedReferenceEvidence, ResolvedFieldRef, TraceId, ValidationError,
};
use serde_json::{Map, Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

/// Maximum UTF-8 bytes returned by one text read.
pub(super) const MAX_TEXT_BYTES: u64 = 16_384;
/// Maximum context entries, domain sources or domain values per page.
pub(super) const MAX_PAGE_ENTRIES: u64 = 64;

/// Exact DTOs of the detail operations, decoded before any state is touched.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DetailRequest {
    Select {
        query_ref: String,
        diagnostic_ref: String,
    },
    Text {
        selection_ref: String,
        component_ref: String,
        offset: u64,
        limit: u64,
    },
    Context {
        selection_ref: String,
        offset: u64,
        limit: u64,
    },
    Sources {
        selection_ref: String,
        offset: u64,
        limit: u64,
    },
    Domain {
        selection_ref: String,
        source_ref: String,
        cursor_ref: Option<String>,
    },
    DomainState {
        selection_ref: String,
        read_ref: String,
    },
}
/// JSON integer within `1..=max`; strings, booleans, floats and 0 are rejected.
fn limit(value: &Value, max: u64) -> Result<u64, ErrorCode> {
    value
        .as_u64()
        .filter(|limit| (1..=max).contains(limit))
        .ok_or(ErrorCode::InvalidMessage)
}
impl DetailRequest {
    pub(crate) fn decode(operation: &str, dto: &Map<String, Value>) -> Result<Self, ErrorCode> {
        ipc::version(dto)?;
        match operation {
            "results_select" => {
                fields(dto, &["query_ref", "diagnostic_ref"])?;
                Ok(Self::Select {
                    query_ref: identifier(&dto["query_ref"])?,
                    diagnostic_ref: identifier(&dto["diagnostic_ref"])?,
                })
            }
            "results_text" => {
                fields(dto, &["selection_ref", "component_ref", "offset", "limit"])?;
                Ok(Self::Text {
                    selection_ref: identifier(&dto["selection_ref"])?,
                    component_ref: identifier(&dto["component_ref"])?,
                    offset: canonical_u64(&dto["offset"])?,
                    limit: limit(&dto["limit"], MAX_TEXT_BYTES)?,
                })
            }
            "results_context" => {
                fields(dto, &["selection_ref", "offset", "limit"])?;
                Ok(Self::Context {
                    selection_ref: identifier(&dto["selection_ref"])?,
                    offset: canonical_u64(&dto["offset"])?,
                    limit: limit(&dto["limit"], MAX_PAGE_ENTRIES)?,
                })
            }
            "results_sources" => {
                fields(dto, &["selection_ref", "offset", "limit"])?;
                Ok(Self::Sources {
                    selection_ref: identifier(&dto["selection_ref"])?,
                    offset: canonical_u64(&dto["offset"])?,
                    limit: limit(&dto["limit"], MAX_PAGE_ENTRIES)?,
                })
            }
            "results_domain" => {
                fields(dto, &["selection_ref", "source_ref", "cursor_ref"])?;
                Ok(Self::Domain {
                    selection_ref: identifier(&dto["selection_ref"])?,
                    source_ref: identifier(&dto["source_ref"])?,
                    cursor_ref: match &dto["cursor_ref"] {
                        Value::Null => None,
                        value => Some(identifier(value)?),
                    },
                })
            }
            "results_domain_state" => {
                fields(dto, &["selection_ref", "read_ref"])?;
                Ok(Self::DomainState {
                    selection_ref: identifier(&dto["selection_ref"])?,
                    read_ref: identifier(&dto["read_ref"])?,
                })
            }
            _ => Err(ErrorCode::UnknownOperation),
        }
    }
    pub(super) fn selection_ref(&self) -> Option<&str> {
        match self {
            Self::Select { .. } => None,
            Self::Text { selection_ref, .. }
            | Self::Context { selection_ref, .. }
            | Self::Sources { selection_ref, .. }
            | Self::Domain { selection_ref, .. }
            | Self::DomainState { selection_ref, .. } => Some(selection_ref),
        }
    }
}

/// One readable text with its opaque handle. The text is an exact copy of a
/// captured value: diagnostic metadata, schema guide or domain literal.
struct Component {
    token: String,
    text: String,
}

/// Mint a fresh handle that collides with none already registered for the
/// selection (OS randomness makes a collision a technical anomaly).
fn mint(prefix: &str, taken: &[&[Component]], local: &[Component]) -> Result<String, ErrorCode> {
    let token = nonce(prefix)?;
    if taken
        .iter()
        .chain(std::iter::once(&local))
        .any(|set| set.iter().any(|c| c.token == token))
    {
        return Err(ErrorCode::TechnicalFailure);
    }
    Ok(token)
}

/// Bounded chunk of an exact text. `offset` must be a UTF-8 boundary within
/// the text; the chunk ends on the last character boundary within `limit`
/// bytes. A limit that cannot hold the next character is rejected, never
/// stretched; the exact end returns an empty final chunk.
fn chunk(text: &str, offset: u64, limit: u64) -> Result<Value, ErrorCode> {
    let start = usize::try_from(offset).map_err(|_| ErrorCode::InvalidMessage)?;
    if start > text.len() || !text.is_char_boundary(start) {
        return Err(ErrorCode::InvalidMessage);
    }
    let limit = usize::try_from(limit).map_err(|_| ErrorCode::InvalidMessage)?;
    let mut end = start.saturating_add(limit).min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    if end == start && start < text.len() {
        return Err(ErrorCode::InvalidMessage);
    }
    Ok(
        json!({"schemaVersion":1,"offset":offset.to_string(),"text":&text[start..end],
        "next_offset":end.to_string(),"has_more":end < text.len()}),
    )
}

/// The compiled field of a diagnostic in the trace that produced it (also for
/// Full multi-trace reports), and whether it is a proper subset of the record.
fn compiled_field<'a>(
    schema: &'a CompiledFlowSchema,
    trace: &TraceId,
    field: &FieldId,
) -> Option<(&'a CompiledFieldSpec, bool)> {
    let record = schema.trace(trace)?;
    let spec = record.field(field)?;
    Some((spec, spec.range().len() < record.record_length()))
}

/// Kind of a domain source, as listed in the selection DTO.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceKind {
    AllowedValues,
    Range,
    Fallback,
    Reference,
    Regex,
    NonEnumerable,
}
impl SourceKind {
    fn name(self) -> &'static str {
        match self {
            Self::AllowedValues => "allowed_values",
            Self::Range => "range",
            Self::Fallback => "fallback",
            Self::Reference => "reference",
            Self::Regex => "regex",
            Self::NonEnumerable => "non_enumerable",
        }
    }
}

/// RuleId of a local rule and its position in the captured trace's rules.
type RuleRef = (Arc<str>, usize);

/// One declaration consultable for the selected field. A conditional source
/// keeps the RuleId and the position of its local rule in the captured trace,
/// so its guard is rendered natively with the source page: its values are
/// never presented as an unconditional field constraint. Reference sources
/// name the dataset only.
struct Source {
    token: String,
    kind: SourceKind,
    rule: Option<RuleRef>,
    constraint: CompiledConstraint,
    dataset: Option<DatasetKey>,
}
impl Source {
    /// `condition` is the guard component of a conditional source on the
    /// current source page, or `None` when the source is unconditional or
    /// its guard has no textual rendering (the source stays `conditional`).
    fn dto(&self, condition: Option<&Component>) -> Value {
        json!({"schemaVersion":1,"source_ref":self.token,"kind":self.kind.name(),
            "conditional":self.rule.is_some(),
            "rule":self.rule.as_ref().map(|(id, _)| preview(id)),
            "condition":condition.map(|c| json!({"schemaVersion":1,
                "component_ref":c.token,"preview":preview(&c.text)})),
            "dataset":self.dataset.as_ref().map(|key| json!({"schemaVersion":1,
                "id":preview(&key.id.0),"version":preview(&key.version.0)})),
            "comparison":preview(lintra::constraint_comparison_guide(&self.constraint))})
    }
}

/// Field declarations in schema order: field constraints, then constraints
/// that local rules of the same trace assert on this field (conditional).
/// This is the order the Slint detail visits; a reference constraint yields
/// its fallback literals first and then one source per declared dataset.
fn sources(
    schema: &CompiledFlowSchema,
    trace: &TraceId,
    field: &FieldId,
) -> Result<Vec<Source>, ErrorCode> {
    let Some(record) = schema.trace(trace) else {
        return Ok(Vec::new());
    };
    let Some(spec) = record.field(field) else {
        return Ok(Vec::new());
    };
    let mut declarations: Vec<(&CompiledConstraint, Option<RuleRef>)> =
        spec.constraints().iter().map(|c| (c, None)).collect();
    for (position, rule) in record.local_rules().iter().enumerate() {
        for assertion in &rule.assertions {
            if let CompiledAssertion::Constraint {
                field: ResolvedFieldRef::Local { field_id, .. },
                constraint,
            } = assertion
                && field_id == field
            {
                declarations.push((constraint, Some((rule.id.clone(), position))));
            }
        }
    }
    let mut result: Vec<Source> = Vec::new();
    // No count limit: sources are listed in pages of at most 64 entries, so
    // the number of declarations never prevents opening the diagnostic.
    let mut push = |kind, constraint: &CompiledConstraint, rule: &Option<RuleRef>, dataset| {
        let token = nonce("o")?;
        if result.iter().any(|s| s.token == token) {
            return Err(ErrorCode::TechnicalFailure);
        }
        result.push(Source {
            token,
            kind,
            rule: rule.clone(),
            constraint: constraint.clone(),
            dataset,
        });
        Ok(())
    };
    for (constraint, rule) in declarations {
        match constraint {
            CompiledConstraint::AllowedValues { values, .. } if !values.is_empty() => {
                push(SourceKind::AllowedValues, constraint, &rule, None)?
            }
            CompiledConstraint::Range { .. } => push(SourceKind::Range, constraint, &rule, None)?,
            CompiledConstraint::Regex { .. } => push(SourceKind::Regex, constraint, &rule, None)?,
            CompiledConstraint::ReferenceData {
                dataset_indexes,
                fallback_values,
                ..
            } => {
                if !fallback_values.is_empty() {
                    push(SourceKind::Fallback, constraint, &rule, None)?;
                }
                for index in dataset_indexes {
                    let key = schema
                        .required_reference_data()
                        .get(*index)
                        .ok_or(ErrorCode::TechnicalFailure)?
                        .key
                        .clone();
                    push(SourceKind::Reference, constraint, &rule, Some(key))?;
                }
            }
            _ => push(SourceKind::NonEnumerable, constraint, &rule, None)?,
        }
    }
    Ok(result)
}

/// A reference page being read off the IPC thread from the captured lookup.
struct PendingRead {
    read_ref: String,
    thread: Option<JoinHandle<Option<LookupPage>>>,
    cancelled: Arc<AtomicBool>,
}
impl Drop for PendingRead {
    /// A superseded or closed read is cancelled and joined: the lookup scan
    /// observes the flag between values, so no detached worker survives.
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Private continuation of the current domain page.
enum Position {
    Declared(DomainPosition),
    Reference(Vec<u8>),
}

/// The one domain source being consulted for the selection: its current page
/// of value handles, its single opaque continuation and any pending read.
struct DomainRead {
    source: usize,
    components: Vec<Component>,
    cursor: Option<(String, Position)>,
    pending: Option<PendingRead>,
}

/// The pinned selected diagnostic and every handle derived from it.
pub(super) struct Selection {
    pub(super) selection_ref: String,
    error: ValidationError,
    components: Vec<Component>,
    context: Vec<Component>,
    sources: Vec<Source>,
    // Guard components of the conditional sources on the current source
    // page; a new page replaces them, like context and domain values.
    conditions: Vec<Component>,
    domain: Option<DomainRead>,
}
impl Selection {
    fn text(&self, token: &str) -> Option<&str> {
        self.components
            .iter()
            .chain(&self.context)
            .chain(&self.conditions)
            .chain(self.domain.iter().flat_map(|d| &d.components))
            .find(|c| c.token == token)
            .map(|c| c.text.as_str())
    }
    /// Cancel a pending reference read (application close).
    pub(super) fn cancel(&self) {
        if let Some(pending) = self.domain.as_ref().and_then(|d| d.pending.as_ref()) {
            pending.cancelled.store(true, Ordering::Release);
        }
    }
}

/// Resolve a diagnostic handle of a retained page and pin its diagnostic.
/// The previous selection is replaced only after the new DTO is encoded.
pub(super) fn select(
    query: &mut Query,
    diagnostic_ref: &str,
    id: &str,
) -> Result<Vec<u8>, ErrorCode> {
    let row = query
        .pages
        .iter()
        .flatten()
        .find(|(token, _)| token == diagnostic_ref)
        .map(|(_, row)| *row)
        .ok_or(ErrorCode::StaleContext)?;
    let error = query
        .read(|index| index.read_row(row).map_err(|_| ErrorCode::TechnicalFailure))?
        .ok_or(ErrorCode::TechnicalFailure)?;
    // The spool is immutable: a row outside the query means it changed.
    if !query.filter.matches(&error) {
        return Err(ErrorCode::TechnicalFailure);
    }
    let report = query.report.clone();
    let trace = report
        .traces()
        .iter()
        .position(|trace| *trace == error.trace_id)
        .ok_or(ErrorCode::TechnicalFailure)?;
    let schema = report.schema();
    let field = error.field.as_ref().and_then(|field| {
        compiled_field(schema, &error.trace_id, &field.id).map(|(spec, proper)| {
            // Coverage holds only if the compiled range is the reported one.
            let same = spec.range().start() + 1 == field.start_one_based
                && spec.range().end_exclusive() == field.end_one_based_inclusive;
            (spec, proper && same)
        })
    });
    let selection_ref = nonce("s")?;
    // Components in a fixed order; `kinds[i]` is the kind of `components[i]`.
    let mut components: Vec<Component> = Vec::new();
    let mut kinds: Vec<&'static str> = Vec::new();
    let push = |components: &mut Vec<Component>,
                kinds: &mut Vec<&'static str>,
                kind: &'static str,
                text: Option<String>|
     -> Result<(), ErrorCode> {
        if let Some(text) = text {
            let token = mint("x", &[], components)?;
            components.push(Component { token, text });
            kinds.push(kind);
        }
        Ok(())
    };
    let code = error
        .code
        .as_symbol()
        .map_err(|_| ErrorCode::TechnicalFailure)?
        .to_owned();
    push(&mut components, &mut kinds, "code", Some(code.clone()))?;
    push(
        &mut components,
        &mut kinds,
        "message",
        Some(error.message.clone()),
    )?;
    push(
        &mut components,
        &mut kinds,
        "expected",
        error.expected.clone(),
    )?;
    if let Some(diagnostic) = &error.field {
        push(
            &mut components,
            &mut kinds,
            "field_name",
            Some(diagnostic.name.to_string()),
        )?;
        push(
            &mut components,
            &mut kinds,
            "guide",
            field.map(|(spec, _)| lintra::field_format_guide(spec, schema.encoding())),
        )?;
    }
    // A generic record-length diagnostic carries a byte count, not input.
    let count_metadata = error.field.is_none()
        && code == "E_RECORD_LENGTH"
        && error.actual.as_ref().is_some_and(|actual| {
            actual.raw_hex.is_none()
                && !actual.display.is_empty()
                && actual.display.bytes().all(|byte| byte.is_ascii_digit())
        });
    let visible = field.is_some_and(|(_, covered)| covered) || count_metadata;
    let visibility = match &error.actual {
        None => "not_present",
        Some(_) if visible => "available",
        Some(_) => "local_report_only_record_scope",
    };
    if visible {
        // Display only: raw_hex never crosses IPC.
        push(
            &mut components,
            &mut kinds,
            "actual",
            error.actual.as_ref().map(|actual| actual.display.clone()),
        )?;
    }
    // Individual identity components of this one diagnostic; no wildcard
    // technical identifier map and no physical record line.
    let identity = &error.record_identity;
    for (kind, value) in [
        ("sioc_record_id", &identity.sioc_record_id),
        ("fiscal_code", &identity.fiscal_code),
        ("episode_id", &identity.episode_id),
    ] {
        push(
            &mut components,
            &mut kinds,
            kind,
            value.as_deref().map(str::to_owned),
        )?;
    }
    let sources = match &error.field {
        Some(diagnostic) => sources(schema, &error.trace_id, &diagnostic.id)?,
        None => Vec::new(),
    };
    let dto = json!({"schemaVersion":1,"selection_ref":selection_ref,
        "query_ref":query.query_ref,"diagnostic_ref":diagnostic_ref,"run":query.run.dto(),
        "line":error.line_number.to_string(),"trace_ref":format!("trace_{trace}"),
        "field_position":error.field.as_ref().map(|field| json!({"schemaVersion":1,
            "start":field.start_one_based.to_string(),
            "end":field.end_one_based_inclusive.to_string(),
            "units":"one_based_inclusive_bytes"})),
        "actual_visibility":visibility,
        "components":components.iter().zip(&kinds).map(|(c, kind)| json!({"schemaVersion":1,
            "component_ref":c.token,"kind":kind,"preview":preview(&c.text)})).collect::<Vec<_>>(),
        "source_count":sources.len().to_string()});
    let bytes = ipc::response(id, dto)?;
    // Replacing the selection drops (cancels and joins) its pending read.
    query.selection = Some(Selection {
        selection_ref,
        error,
        components,
        context: Vec::new(),
        sources,
        conditions: Vec::new(),
        domain: None,
    });
    Ok(bytes)
}

/// Bounded chunk of one component of the selection, its context page or its
/// domain page. An unknown or evicted handle is stale, never a path or offset.
pub(super) fn text(
    selection: &Selection,
    component_ref: &str,
    offset: u64,
    limit: u64,
    id: &str,
) -> Result<Vec<u8>, ErrorCode> {
    let text = selection
        .text(component_ref)
        .ok_or(ErrorCode::StaleContext)?;
    ipc::response(id, chunk(text, offset, limit)?)
}

/// One page of at most 64 context entries. A value is exposed only for a
/// captured field that is a proper subset of its record; a new page replaces
/// the previous page's handles only once its frame has been encoded.
pub(super) fn context(
    selection: &mut Selection,
    report: &Report,
    offset: u64,
    limit: u64,
    id: &str,
) -> Result<Vec<u8>, ErrorCode> {
    let total = selection.error.context.len();
    let start = usize::try_from(offset).map_err(|_| ErrorCode::InvalidMessage)?;
    if start > total {
        return Err(ErrorCode::InvalidMessage);
    }
    let limit = usize::try_from(limit).map_err(|_| ErrorCode::InvalidMessage)?;
    let end = start.saturating_add(limit).min(total);
    let mut page: Vec<Component> = Vec::new();
    let mut entries = Vec::new();
    for item in &selection.error.context[start..end] {
        let allowed = compiled_field(report.schema(), &item.trace_id, &item.field_id)
            .is_some_and(|(_, proper)| proper);
        let token = if allowed {
            let token = mint("k", &[&selection.components, &selection.conditions], &page)?;
            page.push(Component {
                token: token.clone(),
                text: item.value.display.clone(),
            });
            Some(token)
        } else {
            None
        };
        entries.push(json!({"schemaVersion":1,"trace":preview(&item.trace_id.0),
            "field":preview(&item.field_id.0),"component_ref":token,
            "value":allowed.then(|| preview(&item.value.display)),
            "visibility":if allowed { "available" } else { "local_report_only_record_scope" }}));
    }
    let bytes = ipc::response(
        id,
        json!({"schemaVersion":1,"selection_ref":selection.selection_ref,
            "offset":offset.to_string(),"entries":entries,"next_offset":end.to_string(),
            "has_more":end < total}),
    )?;
    selection.context = page;
    Ok(bytes)
}

/// One page of at most 64 domain sources of the selected field. Each
/// conditional source carries its guard as a native guide component (the
/// rule's condition text, readable by `results_text`); a new page replaces
/// the previous page's guard handles once its frame has been encoded.
pub(super) fn sources_page(
    selection: &mut Selection,
    report: &Report,
    offset: u64,
    limit: u64,
    id: &str,
) -> Result<Vec<u8>, ErrorCode> {
    let total = selection.sources.len();
    let start = usize::try_from(offset).map_err(|_| ErrorCode::InvalidMessage)?;
    if start > total {
        return Err(ErrorCode::InvalidMessage);
    }
    let limit = usize::try_from(limit).map_err(|_| ErrorCode::InvalidMessage)?;
    let end = start.saturating_add(limit).min(total);
    let record = report.schema().trace(&selection.error.trace_id);
    let mut page: Vec<Component> = Vec::new();
    let mut entries = Vec::new();
    for source in &selection.sources[start..end] {
        let guard = source.rule.as_ref().and_then(|(_, position)| {
            let record = record?;
            lintra::rule_condition_guide(record.local_rules().get(*position)?, record)
        });
        let condition = match guard {
            Some(text) => {
                let token = mint("g", &[&selection.components, &selection.context], &page)?;
                page.push(Component { token, text });
                page.last()
            }
            None => None,
        };
        entries.push(source.dto(condition));
    }
    let bytes = ipc::response(
        id,
        json!({"schemaVersion":1,"selection_ref":selection.selection_ref,
            "offset":offset.to_string(),"sources":entries,"next_offset":end.to_string(),
            "has_more":end < total}),
    )?;
    selection.conditions = page;
    Ok(bytes)
}

fn digest_pair(
    source: &Source,
    references: Option<&Arc<PreparedReferenceEvidence>>,
) -> (Option<String>, Option<String>) {
    match (&source.dataset, references) {
        (Some(key), Some(references)) => references
            .evidence(key)
            .map(|evidence| {
                (
                    Some(evidence.expected_sha256().to_string()),
                    evidence.actual_sha256().map(ToString::to_string),
                )
            })
            .unwrap_or((None, None)),
        _ => (None, None),
    }
}

/// Domain page DTO. `entries` is null for every status that is not a
/// successful read, so a missing or unreadable domain is never shown empty.
fn domain_dto(
    source: &Source,
    references: Option<&Arc<PreparedReferenceEvidence>>,
    status: &str,
    entries: Option<&[Component]>,
    cursor: Option<&str>,
) -> Value {
    let (expected, actual) = digest_pair(source, references);
    json!({"schemaVersion":1,"kind":source.kind.name(),"conditional":source.rule.is_some(),
        "expected_sha256":expected,"actual_sha256":actual,"status":status,
        "entries":entries.map(|values| values.iter().map(|c| json!({"schemaVersion":1,
            "component_ref":c.token,"preview":preview(&c.text)})).collect::<Vec<_>>()),
        "next_cursor":cursor})
}

fn envelope(selection_ref: &str, source_ref: &str, read_ref: Option<&str>, domain: Value) -> Value {
    json!({"schemaVersion":1,"selection_ref":selection_ref,"source_ref":source_ref,
        "state":if read_ref.is_some() { "loading" } else { "ready" },
        "read_ref":read_ref,"domain":if read_ref.is_some() { Value::Null } else { domain }})
}

/// Publish a page of literals as value handles plus an optional continuation.
/// An oversized provider page or an empty page with a continuation breaks the
/// native page contract and is a technical failure, never an empty domain.
fn page_components(
    selection: &Selection,
    values: Vec<String>,
    more: bool,
) -> Result<(Vec<Component>, Option<String>), ErrorCode> {
    if values.len() > MAX_PAGE_ENTRIES as usize || (values.is_empty() && more) {
        return Err(ErrorCode::TechnicalFailure);
    }
    let mut page: Vec<Component> = Vec::new();
    for text in values {
        let token = mint(
            "v",
            &[
                &selection.components,
                &selection.context,
                &selection.conditions,
            ],
            &page,
        )?;
        page.push(Component { token, text });
    }
    let cursor = more.then(|| nonce("c")).transpose()?;
    Ok((page, cursor))
}

/// Open a source (`cursor_ref` null) or continue it with its current opaque
/// cursor. Declared literals are paged at once (at most 64, O(64) work);
/// a verified reference dataset is paged on a worker and polled through
/// `results_domain_state`. Opening any source invalidates the previous
/// source's values, cursor and pending read.
pub(super) fn domain(
    selection: &mut Selection,
    report: &Report,
    source_ref: &str,
    cursor_ref: Option<&str>,
    id: &str,
) -> Result<Vec<u8>, ErrorCode> {
    let index = selection
        .sources
        .iter()
        .position(|s| s.token == source_ref)
        .ok_or(ErrorCode::StaleContext)?;
    let position = match cursor_ref {
        None => None,
        Some(cursor) => {
            let current = selection
                .domain
                .as_ref()
                .filter(|d| d.source == index && d.pending.is_none())
                .and_then(|d| d.cursor.as_ref())
                .filter(|(token, _)| token == cursor)
                .ok_or(ErrorCode::StaleContext)?;
            Some(&current.1)
        }
    };
    let source = &selection.sources[index];
    let references = report.references();
    let schema = report.schema();
    let fixed = |status: &str| domain_dto(source, references, status, None, None);
    let (dto, read) = match source.kind {
        SourceKind::Regex | SourceKind::NonEnumerable => (fixed("not_enumerable"), None),
        SourceKind::AllowedValues | SourceKind::Range | SourceKind::Fallback => {
            let position = match position {
                None => None,
                Some(Position::Declared(position)) => Some(position),
                Some(Position::Reference(_)) => return Err(ErrorCode::TechnicalFailure),
            };
            let page =
                lintra::declared_domain_page(&source.constraint, schema.encoding(), position)
                    .ok_or(ErrorCode::TechnicalFailure)?;
            let more = page.next.is_some();
            let (components, cursor) = page_components(selection, page.values, more)?;
            let status = if components.is_empty() {
                "available_empty"
            } else {
                "available"
            };
            let dto = domain_dto(
                source,
                references,
                status,
                Some(&components),
                cursor.as_deref(),
            );
            let next = cursor.zip(page.next.map(Position::Declared));
            (
                dto,
                Some(DomainRead {
                    source: index,
                    components,
                    cursor: next,
                    pending: None,
                }),
            )
        }
        SourceKind::Reference => {
            let key = source.dataset.clone().ok_or(ErrorCode::TechnicalFailure)?;
            match references.and_then(|r| r.evidence(&key).map(|e| (r, e))) {
                None => (fixed("not_captured"), None),
                Some((_, evidence)) if evidence.availability() == DatasetAvailability::Degraded => {
                    (fixed("degraded"), None)
                }
                Some((_, evidence))
                    if evidence.availability() == DatasetAvailability::Unavailable =>
                {
                    (fixed("unavailable"), None)
                }
                Some((references, evidence)) => {
                    // Only Verified evidence whose digest matches is readable.
                    if evidence.actual_sha256() != Some(evidence.expected_sha256()) {
                        return Err(ErrorCode::TechnicalFailure);
                    }
                    let after = match position {
                        None => None,
                        Some(Position::Reference(bytes)) => Some(bytes.clone()),
                        Some(Position::Declared(_)) => return Err(ErrorCode::TechnicalFailure),
                    };
                    let read_ref = nonce("r")?;
                    let cancelled = Arc::new(AtomicBool::new(false));
                    let (worker, flag) = (references.clone(), cancelled.clone());
                    let thread = thread::Builder::new()
                        .name("lintra-desktop-domain".into())
                        .spawn(move || worker.domain_page(&key, after.as_deref(), &flag))
                        .map_err(|_| ErrorCode::TechnicalFailure)?;
                    let pending = PendingRead {
                        read_ref: read_ref.clone(),
                        thread: Some(thread),
                        cancelled,
                    };
                    // On an encoding failure `pending` is dropped, which
                    // cancels and joins the worker before returning.
                    let bytes = ipc::response(
                        id,
                        envelope(
                            &selection.selection_ref,
                            source_ref,
                            Some(&read_ref),
                            Value::Null,
                        ),
                    )?;
                    selection.domain = Some(DomainRead {
                        source: index,
                        components: Vec::new(),
                        cursor: None,
                        pending: Some(pending),
                    });
                    return Ok(bytes);
                }
            }
        }
    };
    let bytes = ipc::response(
        id,
        envelope(&selection.selection_ref, source_ref, None, dto),
    )?;
    // A fixed status keeps no values or cursor of the previous source.
    selection.domain = read.or(Some(DomainRead {
        source: index,
        components: Vec::new(),
        cursor: None,
        pending: None,
    }));
    Ok(bytes)
}

/// Poll a pending reference read. While the worker runs the answer is
/// `loading`; a finished read is joined (never blocking on a running one) and
/// published once. Its handle is then consumed: polling it again is stale.
pub(super) fn domain_state(
    selection: &mut Selection,
    report: &Report,
    read_ref: &str,
    id: &str,
) -> Result<Vec<u8>, ErrorCode> {
    let read = selection
        .domain
        .as_mut()
        .filter(|d| d.pending.as_ref().is_some_and(|p| p.read_ref == read_ref))
        .ok_or(ErrorCode::StaleContext)?;
    let source_index = read.source;
    let source_ref = selection.sources[source_index].token.clone();
    let pending = read.pending.as_mut().ok_or(ErrorCode::StaleContext)?;
    if !pending.thread.as_ref().is_some_and(JoinHandle::is_finished) {
        return ipc::response(
            id,
            envelope(
                &selection.selection_ref,
                &source_ref,
                Some(read_ref),
                Value::Null,
            ),
        );
    }
    let cancelled = pending.cancelled.load(Ordering::Acquire);
    let page = pending
        .thread
        .take()
        .map(JoinHandle::join)
        .ok_or(ErrorCode::TechnicalFailure)?
        .map_err(|_| ErrorCode::TechnicalFailure);
    // The worker is joined: whatever happens next, this read is consumed.
    if let Some(read) = selection.domain.as_mut() {
        read.pending = None;
    }
    let page = page?;
    let source = &selection.sources[source_index];
    let references = report.references();
    let schema = report.schema();
    let (dto, components, cursor) = match page {
        None if cancelled => (
            domain_dto(source, references, "cancelled", None, None),
            Vec::new(),
            None,
        ),
        None => (
            domain_dto(source, references, "unsupported", None, None),
            Vec::new(),
            None,
        ),
        Some(page) => {
            let values = page
                .values
                .iter()
                .map(|bytes| lintra::domain_literal(bytes, schema.encoding()))
                .collect();
            let more = page.next.is_some();
            let (components, cursor) = page_components(selection, values, more)?;
            let status = if components.is_empty() {
                "available_empty"
            } else {
                "available"
            };
            let dto = domain_dto(
                source,
                references,
                status,
                Some(&components),
                cursor.as_deref(),
            );
            (
                dto,
                components,
                cursor.zip(page.next.map(Position::Reference)),
            )
        }
    };
    let bytes = ipc::response(
        id,
        envelope(&selection.selection_ref, &source_ref, None, dto),
    )?;
    selection.domain = Some(DomainRead {
        source: source_index,
        components,
        cursor,
        pending: None,
    });
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::super::{
        Results, ResultsRequest,
        tests::{base, page, query, report, rows, settle},
    };
    use super::*;
    use crate::{
        configuration::PreparedIntent,
        jobs::{JobRequest, Jobs, RunRef},
    };
    use lintra::*;
    use std::{
        fs,
        path::PathBuf,
        time::{Duration, Instant},
    };

    const INSTANCE: &str = "cccccccccccccccccccccccccccccccc";
    const FLOW: &[u8] =
        include_bytes!("../../../../../bundles/embedded/schemas/SYNTHETIC/v1/flow.json");
    const CASES: &[u8] =
        include_bytes!("../../../../../tests/fixtures/reference_membership/union_cases.json");

    struct NoEnvironment;
    impl BundleEnvironment for NoEnvironment {
        fn var_os(&self, _: &str) -> Option<std::ffi::OsString> {
            None
        }
    }

    fn detail(results: &mut Results, operation: &str, payload: Value) -> Result<Value, ErrorCode> {
        let request = DetailRequest::decode(operation, payload.as_object().unwrap())?;
        let bytes = results.detail(request, "test_1")?;
        assert!(bytes.len() <= ipc::MAX_BYTES);
        let frame: Value = serde_json::from_slice(&bytes).unwrap();
        Ok(frame["payload"].clone())
    }
    fn job_detail(jobs: &mut Jobs, operation: &str, payload: Value) -> Result<Value, ErrorCode> {
        let request = DetailRequest::decode(operation, payload.as_object().unwrap())?;
        let bytes = jobs.detail(request, "test_1")?;
        assert!(bytes.len() <= ipc::MAX_BYTES);
        Ok(serde_json::from_slice::<Value>(&bytes).unwrap()["payload"].clone())
    }
    fn job_results(jobs: &mut Jobs, operation: &str, payload: Value) -> Value {
        let request = ResultsRequest::decode(operation, payload.as_object().unwrap()).unwrap();
        serde_json::from_slice::<Value>(&jobs.results(request, "test_1").unwrap()).unwrap()
            ["payload"]
            .clone()
    }
    /// Reassemble a component through bounded reads; every chunk advances.
    fn read_all(
        call: &mut dyn FnMut(Value) -> Result<Value, ErrorCode>,
        selection: &Value,
        component: &Value,
        limit: u64,
    ) -> String {
        let (mut text, mut offset) = (String::new(), "0".to_owned());
        loop {
            let chunk = call(json!({"schemaVersion":1,"selection_ref":selection,
                "component_ref":component,"offset":offset,"limit":limit}))
            .unwrap();
            assert!(chunk["text"].as_str().unwrap().len() as u64 <= limit);
            text.push_str(chunk["text"].as_str().unwrap());
            if chunk["has_more"] == false {
                return text;
            }
            assert_ne!(chunk["next_offset"].as_str().unwrap(), offset);
            offset = chunk["next_offset"].as_str().unwrap().to_owned();
        }
    }
    /// Every domain source of a selection, read page by page (64 at most).
    fn all_sources(jobs: &mut Jobs, selection: &Value) -> Vec<Value> {
        let (mut sources, mut offset) = (Vec::new(), "0".to_owned());
        loop {
            let page = job_detail(
                jobs,
                "results_sources",
                json!({"schemaVersion":1,"selection_ref":selection["selection_ref"],
                    "offset":offset,"limit":64}),
            )
            .unwrap();
            let entries = page["sources"].as_array().unwrap();
            assert!(entries.len() <= 64);
            sources.extend(entries.iter().cloned());
            if page["has_more"] == false {
                assert_eq!(sources.len().to_string(), selection["source_count"]);
                return sources;
            }
            offset = page["next_offset"].as_str().unwrap().to_owned();
        }
    }
    fn component<'a>(selection: &'a Value, kind: &str) -> Option<&'a Value> {
        selection["components"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["kind"] == kind)
    }

    /// Synthetic bundle on disk: F1 (bytes 1-3) declares allowed values, an
    /// integer range and a regex; F2 (bytes 4-6) declares reference dataset A
    /// with fallback "ZZ", and local rule COND-F2 conditionally restricts F2.
    struct Bundle {
        _root: tempfile::TempDir,
        dataset: PathBuf,
        intent: Arc<PreparedIntent>,
        references: Arc<dyn ReferenceDataPreparer>,
        sha256: String,
    }
    fn bundle(verified: bool) -> Bundle {
        bundle_with(verified, |_| {})
    }
    /// `edit` adjusts the synthetic schema before it is written and resolved.
    fn bundle_with(verified: bool, edit: impl FnOnce(&mut Value)) -> Bundle {
        let cases: Vec<Value> = serde_json::from_slice(CASES).unwrap();
        let case = &cases[0];
        let hex = case["bytes_hex"].as_str().unwrap();
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let sha256 = case["sha256"].as_str().unwrap().to_owned();
        let root = tempfile::tempdir().unwrap();
        let (schemas, references) = (root.path().join("schemas"), root.path().join("references"));
        let folder = schemas.join("SYNTHETIC/v1");
        fs::create_dir_all(&folder).unwrap();
        fs::create_dir(&references).unwrap();
        let dataset = references.join("A.json");
        fs::write(&dataset, &bytes).unwrap();
        let mut schema: Value = serde_json::from_slice(FLOW).unwrap();
        schema["reference_data"] = json!([{"id":"A","dataset_id":"union-A","version":"1",
            "path":"A.json","sha256":if verified { sha256.clone() } else { "0".repeat(64) },
            "metadata_attributes":[]}]);
        schema["traces"][0]["fields"][0]["constraints"] = json!([
            {"kind":"allowed_values","target":"logical","values":["1"]},
            {"kind":"numeric_range","min":0,"max":200},
            {"kind":"regex","pattern":"^[0-9]+$","match":"required","target":"logical"}]);
        schema["traces"][0]["fields"][1]["constraints"] = json!([{"kind":"reference_data",
            "datasets":["A"],"fallback_values":["ZZ"],"metadata_bindings":[]}]);
        schema["traces"][0]["rules"] = json!([{"id":"COND-F2",
            "when":{"eq":{"ref":{"field":"F1"},"value":"1"}},
            "assertions":[{"kind":"constraint","field":{"field":"F2"},
                "constraint":{"kind":"allowed_values","values":["A"],"target":"logical"}}],
            "error_code":"E_CROSS_FIELD","message_template":"Regola {rule_id}."}]);
        edit(&mut schema);
        fs::write(
            folder.join("flow.json"),
            serde_json::to_vec(&schema).unwrap(),
        )
        .unwrap();
        let input = root.path().join("input.txt");
        // Line 1: F2 outside dataset/fallback and COND-F2; line 2: F1 not
        // allowed; line 3: record length.
        fs::write(&input, b"001XX \n002AA \nX\n").unwrap();
        let bundle = AnalysisBundle::resolve(
            &BundleDirectories {
                schema_dir: Some(schemas),
                reference_data_dir: Some(references),
            },
            &NoEnvironment,
            &root.path().join("lintra-desktop"),
        )
        .unwrap();
        let raw: RawFlowSchema = serde_json::from_value(schema).unwrap();
        let intent = Arc::new(PreparedIntent {
            trace_refs: vec!["config_1_trace_0".into()],
            request: ValidationRequest {
                schema_key: SchemaKey {
                    flow_id: FlowId(raw.flow_id.clone()),
                    version: SchemaVersion(raw.schema_version.clone()),
                },
                scope: ValidationScope::Trace(TraceValidationInput {
                    trace_id: TraceId("T1".into()),
                    source: InputSource::Path(input),
                }),
                mode: ValidationMode::LocalOnly,
                options: ValidationOptions {
                    record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                    newline_policy: NewlinePolicy::AutoLfOrCrlf,
                    max_errors: None,
                },
            },
            parameters: Default::default(),
            raw: Arc::new(raw),
            audit: bundle.audit().clone(),
        });
        Bundle {
            _root: root,
            dataset,
            intent,
            references: bundle.references(),
            sha256,
        }
    }
    /// Run the production worker to its terminal state and open the
    /// unfiltered query with one page of every diagnostic.
    fn run_bundle(bundle: &Bundle) -> (Jobs, Value, Value) {
        let mut jobs = Jobs::default();
        jobs.start(
            INSTANCE,
            "start_1",
            "plan_1".into(),
            bundle.intent.clone(),
            bundle.references.clone(),
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        let state = loop {
            let bytes = jobs
                .dispatch(JobRequest::State { run: None }, "state")
                .unwrap();
            let state = serde_json::from_slice::<Value>(&bytes).unwrap()["payload"].clone();
            if state["state"] == "terminal" {
                break state;
            }
            assert!(Instant::now() < deadline, "worker did not terminate");
            thread::yield_now();
        };
        assert_eq!(state["outcome"]["report_available"], true, "{state}");
        let run = RunRef::decode(&state["run"]).unwrap();
        let status = job_results(
            &mut jobs,
            "results_query",
            json!({"schemaVersion":1,"run":run.dto(),"filters":{"schemaVersion":1}}),
        );
        assert_eq!(status["state"], "ready");
        let page = job_results(
            &mut jobs,
            "results_page",
            json!({"schemaVersion":1,"query_ref":status["query_ref"],"offset":"0","limit":128}),
        );
        (jobs, status["query_ref"].clone(), page)
    }
    fn canonical(jobs: &mut Jobs, row: usize) -> ValidationError {
        let (_, report) = jobs.published().unwrap();
        report
            .with_index(|index| index.read_row(row).map_err(|_| ErrorCode::TechnicalFailure))
            .unwrap()
            .unwrap()
    }
    fn poll_domain(jobs: &mut Jobs, selection: &Value, read: &Value) -> Value {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let value = job_detail(
                jobs,
                "results_domain_state",
                json!({"schemaVersion":1,"selection_ref":selection,"read_ref":read}),
            )
            .unwrap();
            if value["state"] == "ready" {
                return value;
            }
            assert_eq!(value["domain"], Value::Null);
            assert!(Instant::now() < deadline, "domain worker did not finish");
            thread::sleep(Duration::from_millis(2));
        }
    }

    #[test]
    fn selected_detail_matches_the_canonical_diagnostic_and_captured_schema() {
        let bundle = bundle(true);
        let (mut jobs, query_ref, page) = run_bundle(&bundle);
        let rows = page["rows"].as_array().unwrap().clone();
        assert!(rows.len() >= 3, "{page}");
        let (_, report) = jobs.published().unwrap();
        let schema = report.schema();
        let mut previous: Option<Value> = None;
        for (position, row) in rows.iter().enumerate() {
            let error = canonical(&mut jobs, position);
            let selection = job_detail(
                &mut jobs,
                "results_select",
                json!({"schemaVersion":1,"query_ref":query_ref,"diagnostic_ref":row["diagnostic_ref"]}),
            )
            .unwrap();
            let text = serde_json::to_string(&selection).unwrap();
            assert!(!text.contains("raw_hex") && !text.contains("technical_ids"));
            assert!(!text.contains(bundle.dataset.parent().unwrap().to_str().unwrap()));
            assert_eq!(selection["line"], error.line_number.to_string());
            assert_eq!(selection["trace_ref"], "trace_0");
            assert_eq!(selection["query_ref"], query_ref);
            let sel = selection["selection_ref"].clone();
            let mut read = |payload| job_detail(&mut jobs, "results_text", payload);
            let exact = |read: &mut dyn FnMut(Value) -> Result<Value, ErrorCode>, kind| {
                component(&selection, kind).map(|c| read_all(read, &sel, &c["component_ref"], 7))
            };
            assert_eq!(
                exact(&mut read, "code").unwrap(),
                error.code.as_symbol().unwrap()
            );
            assert_eq!(exact(&mut read, "message").unwrap(), error.message);
            assert_eq!(exact(&mut read, "expected"), error.expected);
            match &error.field {
                Some(field) => {
                    let spec = schema
                        .trace(&error.trace_id)
                        .unwrap()
                        .field(&field.id)
                        .unwrap();
                    assert_eq!(exact(&mut read, "field_name").unwrap(), &*field.name);
                    assert_eq!(
                        exact(&mut read, "guide").unwrap(),
                        lintra::field_format_guide(spec, schema.encoding())
                    );
                    assert_eq!(
                        selection["field_position"],
                        json!({"schemaVersion":1,"start":field.start_one_based.to_string(),
                            "end":field.end_one_based_inclusive.to_string(),
                            "units":"one_based_inclusive_bytes"})
                    );
                    // Fields 1-3 and 4-6 are proper subsets of the 6-byte record.
                    assert_eq!(selection["actual_visibility"], "available");
                    assert_eq!(
                        exact(&mut read, "actual").unwrap(),
                        error.actual.as_ref().unwrap().display
                    );
                    assert_ne!(selection["source_count"], "0");
                }
                None => {
                    assert_eq!(selection["field_position"], Value::Null);
                    assert_eq!(selection["source_count"], "0");
                    assert!(component(&selection, "guide").is_none());
                    let count = error.actual.as_ref().is_some_and(|a| {
                        a.raw_hex.is_none()
                            && !a.display.is_empty()
                            && a.display.bytes().all(|b| b.is_ascii_digit())
                    });
                    match (&error.actual, count) {
                        (None, _) => assert_eq!(selection["actual_visibility"], "not_present"),
                        (Some(actual), true) => {
                            assert_eq!(selection["actual_visibility"], "available");
                            assert_eq!(exact(&mut read, "actual").unwrap(), actual.display);
                        }
                        (Some(_), false) => assert_eq!(
                            selection["actual_visibility"],
                            "local_report_only_record_scope"
                        ),
                    }
                }
            }
            // A new selection invalidates every handle of the previous one.
            if let Some(old) = previous.take() {
                let code = component(&old, "code").unwrap()["component_ref"].clone();
                assert_eq!(
                    job_detail(
                        &mut jobs,
                        "results_text",
                        json!({"schemaVersion":1,
                        "selection_ref":old["selection_ref"],"component_ref":code,
                        "offset":"0","limit":16})
                    ),
                    Err(ErrorCode::StaleContext)
                );
            }
            previous = Some(selection);
        }
    }

    #[test]
    fn domains_page_declarations_and_the_verified_reference_snapshot() {
        let bundle = bundle(true);
        let (mut jobs, query_ref, page) = run_bundle(&bundle);
        // The run's lookup snapshot is retained: later disk changes are unseen.
        fs::write(&bundle.dataset, b"changed after validation").unwrap();
        let rows = page["rows"].as_array().unwrap().clone();
        let select = |jobs: &mut Jobs, field: &str| {
            let row = rows
                .iter()
                .find(|row| row["field"]["text"] == field)
                .unwrap();
            job_detail(
                jobs,
                "results_select",
                json!({"schemaVersion":1,"query_ref":query_ref,
                "diagnostic_ref":row["diagnostic_ref"]}),
            )
            .unwrap()
        };
        let open = |jobs: &mut Jobs, selection: &Value, source: &Value, cursor: Value| {
            job_detail(
                jobs,
                "results_domain",
                json!({"schemaVersion":1,
                "selection_ref":selection["selection_ref"],"source_ref":source["source_ref"],
                "cursor_ref":cursor}),
            )
        };
        let values = |domain: &Value| -> Vec<String> {
            domain["entries"]
                .as_array()
                .unwrap()
                .iter()
                .map(|e| e["preview"]["text"].as_str().unwrap().to_owned())
                .collect()
        };

        let f2 = select(&mut jobs, "Synthetic text code");
        let sources = all_sources(&mut jobs, &f2);
        let f2_first = sources[0].clone();
        let kinds: Vec<_> = sources
            .iter()
            .map(|s| (s["kind"].clone(), s["conditional"].clone()))
            .collect();
        assert_eq!(
            kinds,
            [
                (json!("fallback"), json!(false)),
                (json!("reference"), json!(false)),
                (json!("allowed_values"), json!(true))
            ]
        );
        assert_eq!(sources[2]["rule"]["text"], "COND-F2");
        // The guard is the engine's own condition text, never omitted.
        assert_eq!(sources[2]["condition"]["preview"]["text"], "F1 = \"1\"");
        assert!(sources[..2].iter().all(|s| s["condition"].is_null()));
        assert_eq!(sources[1]["dataset"]["id"]["text"], "union-A");
        assert_eq!(sources[1]["dataset"]["version"]["text"], "1");
        let fallback = open(&mut jobs, &f2, &sources[0], Value::Null).unwrap();
        assert_eq!(fallback["state"], "ready");
        assert_eq!(fallback["domain"]["kind"], "fallback");
        assert_eq!(fallback["domain"]["status"], "available");
        assert_eq!(fallback["domain"]["actual_sha256"], Value::Null);
        assert_eq!(values(&fallback["domain"]), ["\"ZZ\""]);
        let fallback_value = fallback["domain"]["entries"][0]["component_ref"].clone();

        let pending = open(&mut jobs, &f2, &sources[1], Value::Null).unwrap();
        assert_eq!(pending["state"], "loading");
        assert_eq!(pending["domain"], Value::Null);
        let read = pending["read_ref"].clone();
        let reference = poll_domain(&mut jobs, &f2["selection_ref"], &read);
        assert_eq!(reference["read_ref"], Value::Null);
        assert_eq!(reference["domain"]["status"], "available");
        assert_eq!(reference["domain"]["expected_sha256"], bundle.sha256);
        assert_eq!(reference["domain"]["actual_sha256"], bundle.sha256);
        assert_eq!(values(&reference["domain"]), ["\"AA\""]);
        assert_eq!(reference["domain"]["next_cursor"], Value::Null);
        // The read handle is consumed; the previous source's values are stale.
        assert_eq!(
            job_detail(
                &mut jobs,
                "results_domain_state",
                json!({"schemaVersion":1,
                "selection_ref":f2["selection_ref"],"read_ref":read})
            ),
            Err(ErrorCode::StaleContext)
        );
        assert_eq!(
            job_detail(
                &mut jobs,
                "results_text",
                json!({"schemaVersion":1,
                "selection_ref":f2["selection_ref"],"component_ref":fallback_value,
                "offset":"0","limit":64})
            ),
            Err(ErrorCode::StaleContext)
        );
        let value = reference["domain"]["entries"][0]["component_ref"].clone();
        assert_eq!(
            job_detail(
                &mut jobs,
                "results_text",
                json!({"schemaVersion":1,
                "selection_ref":f2["selection_ref"],"component_ref":value,
                "offset":"0","limit":64})
            )
            .unwrap()["text"],
            "\"AA\""
        );
        let conditional = open(&mut jobs, &f2, &sources[2], Value::Null).unwrap();
        assert_eq!(conditional["domain"]["conditional"], true);
        assert_eq!(values(&conditional["domain"]), ["\"A\""]);

        let f1 = select(&mut jobs, "Synthetic numeric code");
        let sources = all_sources(&mut jobs, &f1);
        let kinds: Vec<_> = sources
            .iter()
            .map(|s| s["kind"].as_str().unwrap())
            .collect();
        assert_eq!(kinds, ["allowed_values", "range", "regex"]);
        let regex = open(&mut jobs, &f1, &sources[2], Value::Null).unwrap();
        assert_eq!(regex["domain"]["status"], "not_enumerable");
        assert_eq!(regex["domain"]["entries"], Value::Null);
        // 0..=200 in pages of 64 with one private continuation at a time.
        let mut all = Vec::new();
        let mut cursor = Value::Null;
        let mut previous: Option<(Value, Value)> = None;
        loop {
            let page = open(&mut jobs, &f1, &sources[1], cursor.clone()).unwrap();
            let domain = &page["domain"];
            assert!(domain["entries"].as_array().unwrap().len() <= 64);
            all.extend(values(domain));
            if let Some((old_cursor, old_value)) = previous.take() {
                assert_eq!(
                    open(&mut jobs, &f1, &sources[1], old_cursor),
                    Err(ErrorCode::StaleContext)
                );
                assert_eq!(
                    job_detail(
                        &mut jobs,
                        "results_text",
                        json!({"schemaVersion":1,
                        "selection_ref":f1["selection_ref"],"component_ref":old_value,
                        "offset":"0","limit":64})
                    ),
                    Err(ErrorCode::StaleContext)
                );
            }
            if domain["next_cursor"].is_null() {
                break;
            }
            cursor = domain["next_cursor"].clone();
            previous = Some((
                cursor.clone(),
                domain["entries"][0]["component_ref"].clone(),
            ));
        }
        assert_eq!(all, (0..=200).map(|v| v.to_string()).collect::<Vec<_>>());
        // A cursor belongs to its source only.
        let range = open(&mut jobs, &f1, &sources[1], Value::Null).unwrap();
        assert_eq!(
            open(
                &mut jobs,
                &f1,
                &sources[0],
                range["domain"]["next_cursor"].clone()
            ),
            Err(ErrorCode::StaleContext)
        );
        // Selecting another diagnostic makes the old sources stale.
        assert_eq!(
            open(&mut jobs, &f2, &f2_first, Value::Null),
            Err(ErrorCode::StaleContext)
        );
    }

    /// More than 64 declarations never prevent opening a diagnostic: sources
    /// are paged, every conditional source carries its own native guard, and
    /// a change of the guard alone changes the readable guide.
    #[test]
    fn sources_are_paged_without_a_count_limit_and_carry_native_guards() {
        let long = "à😀".repeat(400);
        let rules = 70;
        let bundle = bundle_with(true, |schema| {
            let template = schema["traces"][0]["rules"][0].clone();
            let rules_json = schema["traces"][0]["rules"].as_array_mut().unwrap();
            for i in 0..rules {
                let mut rule = template.clone();
                rule["id"] = json!(format!("COND-{i:02}"));
                let value = if i == rules - 1 {
                    long.clone()
                } else {
                    format!("{}", i + 2)
                };
                rule["when"] = json!({"eq":{"ref":{"field":"F1"},"value":value}});
                rule["assertions"][0]["constraint"]["values"] = json!([format!("B{i}")]);
                rules_json.push(rule);
            }
        });
        let (mut jobs, query_ref, page) = run_bundle(&bundle);
        let row = page["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["field"]["text"] == "Synthetic text code")
            .unwrap()
            .clone();
        let selection = job_detail(
            &mut jobs,
            "results_select",
            json!({"schemaVersion":1,
            "query_ref":query_ref,"diagnostic_ref":row["diagnostic_ref"]}),
        )
        .unwrap();
        // fallback + reference + COND-F2 + 70 conditional rules.
        assert_eq!(selection["source_count"], (3 + rules).to_string());
        let sel = selection["selection_ref"].clone();
        let sources_page = |jobs: &mut Jobs, offset: &str, limit: u64| {
            job_detail(
                jobs,
                "results_sources",
                json!({"schemaVersion":1,
                "selection_ref":sel,"offset":offset,"limit":limit}),
            )
        };
        let first = sources_page(&mut jobs, "0", 64).unwrap();
        assert_eq!(first["sources"].as_array().unwrap().len(), 64);
        assert_eq!(
            (first["next_offset"].clone(), first["has_more"].clone()),
            (json!("64"), json!(true))
        );
        let guard = first["sources"][10]["condition"]["component_ref"].clone();
        let read = |jobs: &mut Jobs, component: &Value| {
            job_detail(
                jobs,
                "results_text",
                json!({"schemaVersion":1,"selection_ref":sel,
                "component_ref":component,"offset":"0","limit":16384}),
            )
        };
        // Source 10 is COND-07 (index 3 + 7), guarded by F1 = "9".
        assert_eq!(first["sources"][10]["rule"]["text"], "COND-07");
        assert_eq!(read(&mut jobs, &guard).unwrap()["text"], "F1 = \"9\"");
        let second = sources_page(&mut jobs, "64", 64).unwrap();
        let tail = second["sources"].as_array().unwrap().clone();
        assert_eq!(tail.len(), 9);
        assert_eq!(second["has_more"], false);
        // A new page replaces the previous page's guard handles.
        assert_eq!(read(&mut jobs, &guard), Err(ErrorCode::StaleContext));
        // The long Unicode guard is a bounded preview plus exact chunked text.
        let last = tail.last().unwrap();
        assert_eq!(last["rule"]["text"], format!("COND-{:02}", rules - 1));
        assert_eq!(last["condition"]["preview"]["complete"], false);
        let mut reader = |payload| job_detail(&mut jobs, "results_text", payload);
        assert_eq!(
            read_all(&mut reader, &sel, &last["condition"]["component_ref"], 100),
            format!("F1 = {}", serde_json::to_string(&long).unwrap())
        );
        // A source beyond the first page opens its own domain.
        let domain = job_detail(
            &mut jobs,
            "results_domain",
            json!({"schemaVersion":1,
            "selection_ref":sel,"source_ref":tail[0]["source_ref"],"cursor_ref":null}),
        )
        .unwrap();
        assert_eq!(domain["domain"]["conditional"], true);
        assert_eq!(domain["domain"]["entries"][0]["preview"]["text"], "\"B61\"");
        assert_eq!(
            sources_page(&mut jobs, "74", 1),
            Err(ErrorCode::InvalidMessage)
        );
        assert_eq!(
            DetailRequest::decode(
                "results_sources",
                json!({"schemaVersion":1,
                "selection_ref":"s","offset":"0","limit":65})
                .as_object()
                .unwrap()
            ),
            Err(ErrorCode::InvalidMessage)
        );

        // Only the guard differs: the guide differs, the rest of the source does not.
        let guide = |value: &str| {
            let bundle = bundle_with(true, |schema| {
                schema["traces"][0]["rules"][0]["when"] =
                    json!({"eq":{"ref":{"field":"F1"},"value":value}});
            });
            let (mut jobs, query_ref, page) = run_bundle(&bundle);
            let row = page["rows"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["field"]["text"] == "Synthetic text code")
                .unwrap()
                .clone();
            let selection = job_detail(
                &mut jobs,
                "results_select",
                json!({"schemaVersion":1,
                "query_ref":query_ref,"diagnostic_ref":row["diagnostic_ref"]}),
            )
            .unwrap();
            let mut source = all_sources(&mut jobs, &selection)
                .into_iter()
                .find(|s| s["conditional"] == true)
                .unwrap();
            let text = job_detail(
                &mut jobs,
                "results_text",
                json!({"schemaVersion":1,
                "selection_ref":selection["selection_ref"],
                "component_ref":source["condition"]["component_ref"],"offset":"0","limit":64}),
            )
            .unwrap()["text"]
                .clone();
            for key in ["source_ref", "condition"] {
                source.as_object_mut().unwrap().remove(key);
            }
            (source, text)
        };
        let (one, one_text) = guide("1");
        let (two, two_text) = guide("2");
        assert_eq!(one, two);
        assert_eq!(
            (one_text, two_text),
            (json!("F1 = \"1\""), json!("F1 = \"2\""))
        );
    }

    #[test]
    fn unverified_reference_evidence_is_never_an_empty_valid_domain() {
        let bundle = bundle(false);
        let (mut jobs, query_ref, page) = run_bundle(&bundle);
        let row = page["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["field"]["text"] == "Synthetic text code")
            .unwrap()
            .clone();
        let selection = job_detail(
            &mut jobs,
            "results_select",
            json!({"schemaVersion":1,
            "query_ref":query_ref,"diagnostic_ref":row["diagnostic_ref"]}),
        )
        .unwrap();
        let source = all_sources(&mut jobs, &selection)
            .into_iter()
            .find(|s| s["kind"] == "reference")
            .unwrap()
            .clone();
        let domain = job_detail(
            &mut jobs,
            "results_domain",
            json!({"schemaVersion":1,
            "selection_ref":selection["selection_ref"],"source_ref":source["source_ref"],
            "cursor_ref":null}),
        )
        .unwrap();
        // No worker is started: the answer is immediate and explicit.
        assert_eq!(domain["state"], "ready");
        assert_eq!(domain["domain"]["status"], "degraded");
        assert_eq!(domain["domain"]["entries"], Value::Null);
        assert_eq!(domain["domain"]["expected_sha256"], "0".repeat(64));
        assert_eq!(domain["domain"]["actual_sha256"], bundle.sha256);
    }

    /// Synthetic report on the golden schema whose single field F1 covers the
    /// whole 6-byte record of T1 (no constraints, no reference dependency).
    fn synthetic(rows: &[Value]) -> (Results, (RunRef, Arc<Report>), Value) {
        let (report, _) = report(rows, &["T1"]);
        let published = (super::super::tests::run(1), report);
        let mut results = Results::default();
        let status = query(&mut results, &published, json!({"schemaVersion":1})).unwrap();
        (results, published, status["query_ref"].clone())
    }

    #[test]
    fn whole_record_values_unknown_context_and_texts_stay_bounded() {
        let mut rows = base();
        rows[0]["message"] = json!("  Sintetico\nà😀\t".repeat(2500));
        rows[0]["record_identity"]["fiscal_code"] = json!("SYNTHETIC_CF");
        let context = rows[0]["context"][0].clone();
        rows[0]["context"] = Value::Array(vec![context; 140]);
        rows[1]["actual"] = json!({"display":"42","raw_hex":null});
        let mut whole = rows[0].clone();
        whole["field"] = json!({"id":"F1","name":"Intero record","start":1,"end":6});
        whole["context"] =
            json!([{"trace":"T1","field_id":"F1","value":{"display":"WHOLE1","raw_hex":null}}]);
        whole["line"] = json!(9);
        rows.push(whole);
        let (mut results, _published, query_ref) = synthetic(&rows);
        let page = page(&mut results, &query_ref, "0", 128).unwrap();
        let select = |results: &mut Results, row: usize| {
            detail(
                results,
                "results_select",
                json!({"schemaVersion":1,"query_ref":query_ref,
                "diagnostic_ref":page["rows"][row]["diagnostic_ref"]}),
            )
            .unwrap()
        };

        // Reported range 1-3 differs from compiled F1 (1-6): no actual value.
        let first = select(&mut results, 0);
        assert_eq!(first["actual_visibility"], "local_report_only_record_scope");
        assert!(component(&first, "actual").is_none());
        assert!(
            !serde_json::to_string(&first)
                .unwrap()
                .contains("SYNTHETIC-A")
        );
        assert_eq!(
            component(&first, "fiscal_code").unwrap()["preview"]["text"],
            "SYNTHETIC_CF"
        );
        // F1 exists in the captured schema: its guide is native, but no domain.
        assert!(component(&first, "guide").is_some());
        assert_eq!(first["source_count"], "0");
        let sel = first["selection_ref"].clone();
        let message = component(&first, "message").unwrap();
        assert_eq!(message["preview"]["complete"], false);
        let mut read = |payload| detail(&mut results, "results_text", payload);
        assert_eq!(
            read_all(&mut read, &sel, &message["component_ref"], MAX_TEXT_BYTES),
            rows[0]["message"].as_str().unwrap()
        );
        let length = rows[0]["message"].as_str().unwrap().len();
        let end = read(json!({"schemaVersion":1,"selection_ref":sel,
            "component_ref":message["component_ref"],"offset":length.to_string(),"limit":4}))
        .unwrap();
        assert_eq!(
            (end["text"].clone(), end["has_more"].clone()),
            (json!(""), json!(false))
        );
        // "  Sintetico\nà": 'à' starts at byte 12 and is two bytes long.
        for (offset, limit) in [("13", 4), ("12", 1), (&*(length + 1).to_string(), 4)] {
            assert_eq!(
                read(json!({"schemaVersion":1,"selection_ref":sel,
                    "component_ref":message["component_ref"],"offset":offset,"limit":limit})),
                Err(ErrorCode::InvalidMessage),
                "{offset}/{limit}"
            );
        }
        assert_eq!(
            read(json!({"schemaVersion":1,"selection_ref":sel,
                "component_ref":"path_to_file","offset":"0","limit":4})),
            Err(ErrorCode::StaleContext)
        );

        // Context of an unknown trace is withheld, 64 entries per page.
        let context = |results: &mut Results, offset: &str, limit: u64| {
            detail(
                results,
                "results_context",
                json!({"schemaVersion":1,
                "selection_ref":sel,"offset":offset,"limit":limit}),
            )
        };
        let first_page = context(&mut results, "0", 64).unwrap();
        let entries = first_page["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 64);
        assert_eq!(first_page["has_more"], true);
        assert!(entries.iter().all(|e| e["value"].is_null()
            && e["component_ref"].is_null()
            && e["visibility"] == "local_report_only_record_scope"));
        assert_eq!(
            context(&mut results, "128", 64).unwrap()["entries"]
                .as_array()
                .unwrap()
                .len(),
            12
        );
        assert_eq!(
            context(&mut results, "141", 64),
            Err(ErrorCode::InvalidMessage)
        );
        assert_eq!(
            context(&mut results, "0", 65),
            Err(ErrorCode::InvalidMessage)
        );

        // A generic record-length count is metadata, not a record payload.
        let count = select(&mut results, 1);
        assert_eq!(count["field_position"], Value::Null);
        assert_eq!(count["actual_visibility"], "available");
        assert_eq!(
            component(&count, "actual").unwrap()["preview"]["text"],
            "42"
        );
        // The previous selection is gone with all its handles.
        assert_eq!(context(&mut results, "0", 1), Err(ErrorCode::StaleContext));

        // Exact compiled range, but it covers the whole record: withheld.
        let whole = select(&mut results, 2);
        assert_eq!(whole["actual_visibility"], "local_report_only_record_scope");
        assert!(!serde_json::to_string(&whole).unwrap().contains("WHOLE1"));
        let whole_context = detail(
            &mut results,
            "results_context",
            json!({"schemaVersion":1,
            "selection_ref":whole["selection_ref"],"offset":"0","limit":64}),
        )
        .unwrap();
        assert_eq!(
            whole_context["entries"][0]["visibility"],
            "local_report_only_record_scope"
        );
        assert!(
            !serde_json::to_string(&whole_context)
                .unwrap()
                .contains("WHOLE1")
        );
    }

    #[test]
    fn selection_is_pinned_across_eviction_and_dropped_by_a_new_query() {
        let (mut results, published, query_ref) = synthetic(&rows(300, &["T1"]));
        let first = page(&mut results, &query_ref, "0", 128).unwrap();
        let selection = detail(
            &mut results,
            "results_select",
            json!({"schemaVersion":1,
            "query_ref":query_ref,"diagnostic_ref":first["rows"][3]["diagnostic_ref"]}),
        )
        .unwrap();
        assert_eq!(selection["line"], "4");
        let code = component(&selection, "code").unwrap()["component_ref"].clone();
        let read = |results: &mut Results| {
            detail(
                results,
                "results_text",
                json!({"schemaVersion":1,
                "selection_ref":selection["selection_ref"],"component_ref":code,
                "offset":"0","limit":64}),
            )
        };
        page(&mut results, &query_ref, "128", 128).unwrap();
        page(&mut results, &query_ref, "256", 44).unwrap();
        // The first page is evicted: its references are stale, the pin is not.
        assert_eq!(
            detail(
                &mut results,
                "results_select",
                json!({"schemaVersion":1,
                "query_ref":query_ref,"diagnostic_ref":first["rows"][0]["diagnostic_ref"]})
            ),
            Err(ErrorCode::StaleContext)
        );
        assert_eq!(read(&mut results).unwrap()["text"], "E_RECORD_LENGTH");
        assert_eq!(
            detail(
                &mut results,
                "results_select",
                json!({"schemaVersion":1,
                "query_ref":"q_unknown","diagnostic_ref":first["rows"][0]["diagnostic_ref"]})
            ),
            Err(ErrorCode::StaleContext)
        );
        // A filtered candidate still building keeps the current query and pin.
        let building = query(
            &mut results,
            &published,
            json!({"schemaVersion":1,"line":"1"}),
        )
        .unwrap();
        assert_eq!(
            detail(
                &mut results,
                "results_select",
                json!({"schemaVersion":1,
                "query_ref":building["query_ref"],"diagnostic_ref":"d_x"})
            )
            .err(),
            if building["state"] == "building" {
                Some(ErrorCode::InvalidState)
            } else {
                Some(ErrorCode::StaleContext)
            }
        );
        settle(&mut results, &building["query_ref"]);
        // Publishing the new query drops the selection with all its handles.
        assert_eq!(read(&mut results), Err(ErrorCode::StaleContext));
        let unfiltered = query(&mut results, &published, json!({"schemaVersion":1})).unwrap();
        assert_eq!(unfiltered["state"], "ready");
        assert_eq!(read(&mut results), Err(ErrorCode::StaleContext));
    }

    #[test]
    fn provider_pages_violating_the_native_contract_are_technical_failures() {
        let (mut results, _published, query_ref) = synthetic(&base());
        let page = page(&mut results, &query_ref, "0", 2).unwrap();
        let selection = detail(
            &mut results,
            "results_select",
            json!({"schemaVersion":1,
            "query_ref":query_ref,"diagnostic_ref":page["rows"][0]["diagnostic_ref"]}),
        )
        .unwrap();
        let key = DatasetKey {
            id: DatasetId("A".into()),
            version: DatasetVersion("1".into()),
        };
        let cases: [(Option<LookupPage>, bool, Result<&str, ErrorCode>); 5] = [
            (None, false, Ok("unsupported")),
            (None, true, Ok("cancelled")),
            (
                Some(LookupPage {
                    values: Vec::new(),
                    next: None,
                }),
                false,
                Ok("available_empty"),
            ),
            (
                Some(LookupPage {
                    values: vec![b"X".to_vec(); 65],
                    next: None,
                }),
                false,
                Err(ErrorCode::TechnicalFailure),
            ),
            (
                Some(LookupPage {
                    values: Vec::new(),
                    next: Some(b"X".to_vec()),
                }),
                false,
                Err(ErrorCode::TechnicalFailure),
            ),
        ];
        for (outcome, cancelled, expected) in cases {
            let selected = results
                .current
                .as_mut()
                .unwrap()
                .selection
                .as_mut()
                .unwrap();
            selected.sources = vec![Source {
                token: "o_test".into(),
                kind: SourceKind::Reference,
                rule: None,
                constraint: CompiledConstraint::NonEmpty,
                dataset: Some(key.clone()),
            }];
            let flag = Arc::new(AtomicBool::new(cancelled));
            selected.domain = Some(DomainRead {
                source: 0,
                components: Vec::new(),
                cursor: None,
                pending: Some(PendingRead {
                    read_ref: "r_test".into(),
                    thread: Some(thread::spawn(move || outcome)),
                    cancelled: flag,
                }),
            });
            while !selected
                .domain
                .as_ref()
                .unwrap()
                .pending
                .as_ref()
                .unwrap()
                .thread
                .as_ref()
                .unwrap()
                .is_finished()
            {
                thread::yield_now();
            }
            let answer = detail(
                &mut results,
                "results_domain_state",
                json!({"schemaVersion":1,
                "selection_ref":selection["selection_ref"],"read_ref":"r_test"}),
            );
            match expected {
                Ok(status) => {
                    let answer = answer.unwrap();
                    assert_eq!(answer["domain"]["status"], status);
                    // Not captured evidence: no digest is invented.
                    assert_eq!(answer["domain"]["expected_sha256"], Value::Null);
                    if status != "available_empty" {
                        assert_eq!(answer["domain"]["entries"], Value::Null);
                    } else {
                        assert_eq!(answer["domain"]["entries"], json!([]));
                    }
                }
                Err(code) => assert_eq!(answer, Err(code)),
            }
            // Joined either way: the read handle cannot be replayed.
            assert_eq!(
                detail(
                    &mut results,
                    "results_domain_state",
                    json!({"schemaVersion":1,
                    "selection_ref":selection["selection_ref"],"read_ref":"r_test"})
                ),
                Err(ErrorCode::StaleContext)
            );
        }
    }

    #[test]
    fn detail_dtos_are_exact_and_reject_noncanonical_values() {
        let decode = |operation: &str, payload: Value| {
            DetailRequest::decode(operation, payload.as_object().unwrap())
        };
        assert!(
            decode(
                "results_select",
                json!({"schemaVersion":1,"query_ref":"q_a","diagnostic_ref":"d_b"})
            )
            .is_ok()
        );
        for (operation, payload) in [
            (
                "results_select",
                json!({"schemaVersion":1,"query_ref":"q_a"}),
            ),
            (
                "results_select",
                json!({"schemaVersion":1,"query_ref":"q_a","diagnostic_ref":"d/b"}),
            ),
            (
                "results_select",
                json!({"schemaVersion":1,"query_ref":"q_a","diagnostic_ref":"d_b","row":0}),
            ),
            (
                "results_text",
                json!({"schemaVersion":1,"selection_ref":"s","component_ref":"x","offset":"0","limit":0}),
            ),
            (
                "results_text",
                json!({"schemaVersion":1,"selection_ref":"s","component_ref":"x","offset":"0","limit":16385}),
            ),
            (
                "results_text",
                json!({"schemaVersion":1,"selection_ref":"s","component_ref":"x","offset":"00","limit":4}),
            ),
            (
                "results_text",
                json!({"schemaVersion":1,"selection_ref":"s","component_ref":"x","offset":0,"limit":4}),
            ),
            (
                "results_text",
                json!({"schemaVersion":1,"selection_ref":"s","component_ref":"x","offset":"0","limit":"4"}),
            ),
            (
                "results_text",
                json!({"schemaVersion":1,"selection_ref":"s","component_ref":[],"offset":"0","limit":4}),
            ),
            (
                "results_context",
                json!({"schemaVersion":1,"selection_ref":"s","offset":"0","limit":65}),
            ),
            (
                "results_context",
                json!({"schemaVersion":1,"selection_ref":"s","offset":"-1","limit":1}),
            ),
            (
                "results_domain",
                json!({"schemaVersion":1,"selection_ref":"s","source_ref":"o"}),
            ),
            (
                "results_domain",
                json!({"schemaVersion":1,"selection_ref":"s","source_ref":"o","cursor_ref":1}),
            ),
            (
                "results_domain_state",
                json!({"schemaVersion":1,"selection_ref":"s"}),
            ),
        ] {
            assert_eq!(
                decode(operation, payload.clone()),
                Err(ErrorCode::InvalidMessage),
                "{payload}"
            );
        }
        assert_eq!(
            decode(
                "results_select",
                json!({"schemaVersion":2,"query_ref":"q","diagnostic_ref":"d"})
            ),
            Err(ErrorCode::UnsupportedVersion)
        );
        assert_eq!(
            decode("results_detail", json!({"schemaVersion":1})),
            Err(ErrorCode::UnknownOperation)
        );
        // Text chunks never split a character and stop exactly at the end.
        assert_eq!(chunk("😀x", 0, 4).unwrap()["text"], "😀");
        assert_eq!(chunk("😀x", 4, 4).unwrap()["text"], "x");
        for (offset, limit) in [(1, 1), (0, 1), (0, 2), (6, 4)] {
            assert_eq!(chunk("😀x", offset, limit), Err(ErrorCode::InvalidMessage));
        }
        let end = chunk("😀x", 5, 4).unwrap();
        assert_eq!(
            (end["text"].clone(), end["has_more"].clone()),
            (json!(""), json!(false))
        );
        // Worst case control characters still fit one frame once escaped.
        let control = "\u{1}".repeat(MAX_TEXT_BYTES as usize);
        let frame = ipc::response("x", chunk(&control, 0, MAX_TEXT_BYTES).unwrap()).unwrap();
        assert!(frame.len() <= ipc::MAX_BYTES);
    }
}
