//! Incremental compiler. CNF-COMPILER-01 permits only the verified local subset.
use super::*;
use crate::domain::{FlowId, SchemaVersion, ValidationCode};
use crate::schema::coordinates::source_range;
use crate::schema::{
    linter::{SchemaIssue, SchemaIssueCode as Code, SchemaIssueSeverity, lint_raw_schema},
    raw::*,
};
#[path = "joins.rs"]
mod joins;
#[path = "persistence_profile.rs"]
mod persistence_profile;
#[path = "projection.rs"]
mod projection;
#[path = "reference.rs"]
mod reference;
pub use persistence_profile::CompiledPersistenceProfile;

/// DEC-18 tooling validates only reference declarations. No CompiledFlowSchema
/// is produced, so this cannot make an unsupported schema executable.
pub fn compile_reference_dependencies(
    raw: &[RawReferenceDataDependency],
) -> Result<Vec<ReferenceDependency>, ReferenceDeclarationError> {
    reference::compile_dependencies(raw)
        .map_err(|(code, message)| ReferenceDeclarationError { code, message })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceDeclarationError {
    pub code: Code,
    pub message: String,
}
impl fmt::Display for ReferenceDeclarationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl Error for ReferenceDeclarationError {}

use std::{
    collections::{BTreeMap, HashSet},
    error::Error,
    fmt,
};

/// Per-compilation cache only: immutable regex programs (or their parse errors).
type RegexPlans = BTreeMap<String, Result<regex::bytes::Regex, String>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaCompilationError {
    pub issues: Vec<SchemaIssue>,
}
impl fmt::Display for SchemaCompilationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "schema compilation failed with {} issue(s)",
            self.issues.len()
        )
    }
}
impl Error for SchemaCompilationError {}

/// Compiles only supported AST constructs, not a declaration of full local.v1.
#[derive(Debug, Clone, Copy, Default)]
pub struct SchemaCompiler;

struct Checks<'a> {
    raw: &'a RawFlowSchema,
    issues: Vec<SchemaIssue>,
    regexes: RegexPlans,
}
impl Checks<'_> {
    fn error(
        &mut self,
        code: Code,
        trace: Option<&str>,
        field: Option<&str>,
        message: impl Into<String>,
    ) {
        self.issues.push(SchemaIssue {
            severity: SchemaIssueSeverity::Error,
            code,
            flow_id: Some(FlowId(self.raw.flow_id.clone())),
            trace_id: trace.map(|id| TraceId(id.to_owned())),
            field_id: field.map(|id| FieldId(id.to_owned())),
            message: message.into(),
            source_note: None,
        });
    }
    fn unsupported(&mut self, present: bool, trace: Option<&str>, field: Option<&str>, name: &str) {
        if present {
            self.error(
                Code::UnsupportedConstruct,
                trace,
                field,
                format!("{name} is not supported by this compiler subset"),
            );
        }
    }
    fn record(&mut self, record: &RawRecordSchema, trace_index: usize, joins: &[CompiledJoinSpec]) {
        let trace = Some(record.id.as_str());
        if record.name.is_empty() {
            self.error(
                Code::InvalidSchemaValue,
                trace,
                None,
                "trace name must be nonempty",
            );
        }
        self.unsupported(
            !record.repeated_groups.is_empty(),
            trace,
            None,
            "repeated_groups",
        );
        if let Err((code, message)) = crate::schema::derived::compile_derived_values(record) {
            self.error(code, trace, None, message);
        }
        // Validate every declared definition, including unused ones. Presence
        // policies below retain their fully lowered conditions; unsupported
        // expression nodes still prevent a successful compiled schema.
        for expression in record.named_conditions.values() {
            if let Err((code, message)) =
                crate::schema::reference_binding::ReferenceBindings::cross(
                    self.raw,
                    trace_index,
                    joins,
                )
                .and_then(|bindings| crate::schema::expression::compile_expr(expression, &bindings))
            {
                self.error(code, trace, None, message);
            }
        }
        self.rules(record, trace_index, joins);
        // Resolve identity like other local references. A destination can hold
        // only one value: reject collisions instead of silently overwriting.
        let mut identity_targets = HashSet::new();
        for binding in &record.identity {
            self.reference(record, &binding.field);
            match identity_target(binding) {
                Ok(target) => {
                    let key = match target {
                        IdentityTarget::SiocRecordId => (0, String::new()),
                        IdentityTarget::FiscalCode => (1, String::new()),
                        IdentityTarget::EpisodeId => (2, String::new()),
                        IdentityTarget::Technical(name) => (3, name.to_string()),
                    };
                    if !identity_targets.insert(key) {
                        self.error(
                            Code::InvalidSchemaValue,
                            trace,
                            Some(&binding.field),
                            "identity destination must be unique",
                        );
                    }
                }
                Err((code, message)) => self.error(code, trace, Some(&binding.field), message),
            }
        }
        if record.record_key.fields.is_empty() {
            self.error(
                Code::InvalidRecordKey,
                trace,
                None,
                "record_key must contain at least one field",
            );
        }
        let mut keys = HashSet::new();
        for id in &record.record_key.fields {
            if !keys.insert(id) {
                self.error(
                    Code::InvalidRecordKey,
                    trace,
                    Some(id),
                    "duplicate record_key field",
                );
            }
            self.reference(record, id);
        }
        if record
            .record_key
            .error_code
            .parse::<ValidationCode>()
            .is_err()
        {
            self.error(
                Code::InvalidDiagnosticCode,
                trace,
                None,
                "invalid record_key diagnostic symbol",
            );
        }
        let mut deferred_ids = HashSet::new();
        for rule in &record.documented_non_executable_rules {
            if rule.id.is_empty()
                || rule.id.trim() != rule.id
                || !deferred_ids.insert(&rule.id)
                || rule.reason.is_empty()
                || rule.decision_reference.is_empty()
            {
                self.error(Code::InvalidSchemaValue, trace, None, "deferred rules require unique nonempty identifiers, reason and decision_reference");
            }
            let mut fields = HashSet::new();
            for id in &rule.fields {
                if !fields.insert(id) {
                    self.error(
                        Code::InvalidSchemaValue,
                        trace,
                        Some(id),
                        "duplicate deferred-rule field",
                    );
                }
                self.reference(record, id);
            }
        }
        for field in &record.fields {
            let id = Some(field.id.as_str());
            if field.name.is_empty() {
                self.error(
                    Code::InvalidSchemaValue,
                    trace,
                    id,
                    "field name must be nonempty",
                );
            }
            for expression in field
                .presence
                .required_if
                .iter()
                .chain(&field.presence.forbidden_if)
            {
                if let Err((code, message)) =
                    crate::schema::expression::compile_local_expr(expression, record)
                {
                    self.error(code, trace, id, message);
                }
            }
            if let Err((Code::PresenceConflict, message)) =
                crate::schema::presence::compile_presence(&field.presence, record)
            {
                self.error(Code::PresenceConflict, trace, id, message);
            }
            let length = match &field.physical {
                RawPhysical::Date { length, .. } | RawPhysical::Decimal { length, .. } => *length,
                RawPhysical::Numeric {
                    length, alignment, ..
                }
                | RawPhysical::Alphanumeric {
                    length, alignment, ..
                } => {
                    let fill = match alignment {
                        RawAlignment::RightPad { fill_byte, .. }
                        | RawAlignment::LeftPad { fill_byte, .. } => Some(fill_byte),
                        RawAlignment::Exact {} => None,
                    };
                    if matches!(self.raw.encoding.resolved, Some(RawResolvedEncoding::Utf8))
                        && fill.is_some_and(|b| !b.is_ascii())
                    {
                        self.error(
                            Code::InvalidTypeParameters,
                            trace,
                            id,
                            "UTF-8 padding must be one complete encoded byte",
                        );
                    }
                    *length
                }
            };
            self.constraints(field, length, trace);
            let values = match &field.missing_value {
                RawMissing::Exact { raw } => vec![raw],
                RawMissing::AnyOf { raw_values } => {
                    let unique: HashSet<_> = raw_values.iter().collect();
                    if raw_values.is_empty() || unique.len() != raw_values.len() {
                        self.error(
                            Code::InvalidSchemaValue,
                            trace,
                            id,
                            "missing any_of requires nonempty unique raw values",
                        );
                    }
                    raw_values.iter().collect()
                }
                RawMissing::Spaces {} | RawMissing::NeverMissing {} => Vec::new(),
            };
            // Literal encoding checks require the explicit binding; no UTF-8 default.
            if matches!(self.raw.encoding.resolved, Some(RawResolvedEncoding::Utf8)) {
                for value in values {
                    if u64::try_from(value.len()).ok() != Some(u64::from(length)) {
                        self.error(
                            Code::InvalidTypeParameters,
                            trace,
                            id,
                            "encoded missing literal must exactly match field width in bytes",
                        );
                    }
                }
            }
        }
    }
    /// Check the delivered byte-domain, integer-range and calendar subset. Unsupported nodes
    /// remain blocking issues, even when another constraint is supported.
    /// Literal checks use the explicit UTF-8 binding currently accepted by the
    /// compiler; there is no implicit encoding selection for other profiles.
    /// Type mismatches are distinct from malformed parameters: Date requires
    /// Date, integer Range requires Numeric (never Decimal). Byte-domain and
    /// Regex predicates apply to any implemented type, on their declared target.
    fn constraints(&mut self, field: &RawFieldSpec, width: u32, trace: Option<&str>) {
        let id = Some(field.id.as_str());
        let alignment = match &field.physical {
            RawPhysical::Numeric { alignment, .. }
            | RawPhysical::Alphanumeric { alignment, .. } => alignment,
            RawPhysical::Date { .. } | RawPhysical::Decimal { .. } => &RawAlignment::Exact {},
        };
        for constraint in &field.constraints {
            if let RawConstraint::ReferenceData {
                datasets,
                fallback_values,
                metadata_bindings,
            } = constraint
            {
                self.reference_constraint(datasets, metadata_bindings, trace, id);
                if !fallback_values.is_empty() {
                    let mut fallback = field.clone();
                    fallback.constraints = vec![RawConstraint::AllowedValues {
                        values: fallback_values.clone(),
                        target: RawConstraintAllowedValuesTarget::Logical,
                    }];
                    self.constraints(&fallback, width, trace);
                }
                continue;
            }
            if let RawConstraint::Regex { pattern, .. } = constraint {
                // Compile each distinct pattern once per schema, including
                // failures. Programs are cloned cheaply into frozen field plans.
                let result = self
                    .regexes
                    .entry(pattern.clone())
                    .or_insert_with(|| {
                        if pattern.is_empty() {
                            Err("regex pattern must be nonempty".to_owned())
                        } else {
                            regex::bytes::Regex::new(pattern).map_err(|error| error.to_string())
                        }
                    })
                    .clone();
                if let Err(message) = result {
                    self.error(Code::InvalidRegex, trace, id, message);
                }
                continue;
            }
            if let RawConstraint::Date { calendar_valid, .. } = constraint {
                if !matches!(field.physical, RawPhysical::Date { .. }) {
                    self.error(
                        Code::ConstraintTypeMismatch,
                        trace,
                        id,
                        "date constraint requires a date field",
                    );
                }
                if !calendar_valid {
                    self.error(
                        Code::InvalidSchemaValue,
                        trace,
                        id,
                        "date calendar validation cannot be disabled",
                    );
                }
                continue;
            }
            if let RawConstraint::NumericRange { min, max } = constraint {
                if !matches!(field.physical, RawPhysical::Numeric { .. }) {
                    self.error(
                        Code::ConstraintTypeMismatch,
                        trace,
                        id,
                        "numeric_range requires a numeric field",
                    );
                }
                let min = min.0.to_string().parse::<IntegerLiteral>();
                let max = max.0.to_string().parse::<IntegerLiteral>();
                match (min, max) {
                    (Ok(min), Ok(max)) if min <= max => (),
                    _ => self.error(
                        Code::InvalidSchemaValue,
                        trace,
                        id,
                        "numeric_range requires integer min <= max",
                    ),
                }
                continue;
            }
            let RawConstraint::AllowedValues { values, target } = constraint else {
                self.unsupported(true, trace, id, "constraint kind");
                continue;
            };
            let unique: HashSet<_> = values.iter().collect();
            if values.is_empty() || unique.len() != values.len() {
                self.error(
                    Code::InvalidSchemaValue,
                    trace,
                    id,
                    "allowed_values requires nonempty unique values",
                );
            }
            if !matches!(self.raw.encoding.resolved, Some(RawResolvedEncoding::Utf8)) {
                continue; // Binding errors are emitted globally; no wrong encoding is assumed.
            }
            let Ok(width) = usize::try_from(width) else {
                self.error(
                    Code::RangeOverflow,
                    trace,
                    id,
                    "domain field width is not representable",
                );
                continue;
            };
            let fixed = matches!(target, RawConstraintAllowedValuesTarget::Raw)
                || matches!(
                    alignment,
                    RawAlignment::Exact {}
                        | RawAlignment::RightPad {
                            content_policy: RawAlignmentRightPadContentPolicy::PreserveWidth,
                            ..
                        }
                        | RawAlignment::LeftPad {
                            content_policy: RawAlignmentLeftPadContentPolicy::PreserveWidth,
                            ..
                        }
                );
            for value in values {
                let bytes = value.as_bytes();
                if (fixed && bytes.len() != width) || (!fixed && bytes.len() > width) {
                    self.error(
                        Code::InvalidTypeParameters,
                        trace,
                        id,
                        "domain literal byte width is incompatible with its target",
                    );
                }
                // A logical literal is never padded/stripped by the compiler.
                // A fill on its stripping edge cannot occur in any extracted
                // logical value, so reject this unreachable literal as schema.
                if matches!(target, RawConstraintAllowedValuesTarget::Logical) {
                    let noncanonical = match alignment {
                        RawAlignment::RightPad {
                            fill_byte,
                            content_policy: RawAlignmentRightPadContentPolicy::StripTrailingFill,
                        } => bytes.last() == Some(fill_byte),
                        RawAlignment::LeftPad {
                            fill_byte,
                            content_policy: RawAlignmentLeftPadContentPolicy::StripLeadingFill,
                        } => bytes.first() == Some(fill_byte),
                        _ => false,
                    };
                    if noncanonical {
                        self.error(Code::InvalidTypeParameters, trace, id, "logical domain literal must already have its declared logical representation");
                    }
                }
            }
        }
    }

    fn reference(&mut self, record: &RawRecordSchema, id: &str) {
        if !record.fields.iter().any(|field| field.id == id) {
            self.error(
                Code::MissingFieldReference,
                Some(&record.id),
                Some(id),
                "local field reference does not exist",
            );
        }
    }
    fn finish(mut self) -> (Vec<SchemaIssue>, RegexPlans) {
        // Stable merge of structural and compiler diagnostics. Field IDs are lookup
        // labels, never lexical sort keys. Invalid duplicate IDs keep insertion ties.
        self.issues.sort_by_key(|issue| {
            let trace = issue
                .trace_id
                .as_ref()
                .and_then(|id| self.raw.traces.iter().position(|t| t.id == id.0));
            let trace_order = trace.map_or(0, |index| index + 1);
            let field_order = match (trace, &issue.field_id) {
                (Some(index), Some(id)) => self.raw.traces[index]
                    .fields
                    .iter()
                    .position(|f| f.id == id.0)
                    .map_or(usize::MAX - 1, |i| i + 1),
                (Some(_), None) if issue.code == Code::RecordLengthMismatch => usize::MAX,
                _ => 0,
            };
            (trace_order, field_order)
        });
        (self.issues, self.regexes)
    }
}

impl SchemaCompiler {
    pub fn compile(
        &self,
        raw: RawFlowSchema,
    ) -> Result<CompiledFlowSchema, SchemaCompilationError> {
        self.compile_scoped(raw, None)
    }
    /// Only the checked historical compiler supplies this authorization. The
    /// public ordinary path retains its existing fail-closed capability checks.
    pub(crate) fn compile_scoped(
        &self,
        raw: RawFlowSchema,
        audit_policy: Option<PersistencePolicy>,
    ) -> Result<CompiledFlowSchema, SchemaCompilationError> {
        let mut checks = Checks {
            raw: &raw,
            issues: lint_raw_schema(&raw),
            regexes: RegexPlans::new(),
        };
        if raw.regulatory_version.is_empty()
            || raw.regulatory_version.trim() != raw.regulatory_version
        {
            checks.error(
                Code::InvalidIdentifier,
                None,
                None,
                "regulatory_version must be nonempty without surrounding whitespace",
            );
        }
        match &raw.encoding.resolved {
            None => checks.error(
                Code::MissingEncodingBinding,
                None,
                None,
                "explicit in-memory encoding binding is required",
            ),
            Some(RawResolvedEncoding::Utf8) => (),
            Some(_) => checks.error(
                Code::UnsupportedEncoding,
                None,
                None,
                "this compiler subset supports only explicit UTF-8",
            ),
        }
        if !raw
            .required_capabilities
            .contains(&RawFlowSchemaRequiredCapabilitiesItem::LocalV1)
        {
            checks.error(
                Code::MissingCapability,
                None,
                None,
                "local.v1 is required by the physical AST",
            );
        }
        for (index, capability) in raw.required_capabilities.iter().enumerate() {
            if raw.required_capabilities[..index].contains(capability) {
                checks.error(
                    Code::InvalidSchemaValue,
                    None,
                    None,
                    "duplicate required capability",
                );
            }
            if !matches!(
                capability,
                RawFlowSchemaRequiredCapabilitiesItem::LocalV1
                    | RawFlowSchemaRequiredCapabilitiesItem::ExpressionsV1
                    | RawFlowSchemaRequiredCapabilitiesItem::JoinsV1
                    | RawFlowSchemaRequiredCapabilitiesItem::JoinsSetsV1
                    | RawFlowSchemaRequiredCapabilitiesItem::JoinsConsensusV1
                    | RawFlowSchemaRequiredCapabilitiesItem::DerivedSubstringV1
                    | RawFlowSchemaRequiredCapabilitiesItem::DerivedAgeInCompletedYearsV1
                    | RawFlowSchemaRequiredCapabilitiesItem::DerivedCalendarYearDeltaV1
            ) && !(audit_policy.is_some()
                && *capability == RawFlowSchemaRequiredCapabilitiesItem::PersistenceAuditV1)
            {
                checks.error(
                    Code::UnsupportedCapability,
                    None,
                    None,
                    format!("capability {capability:?} is not supported by this compiler subset"),
                );
            }
        }
        checks.unsupported(
            raw.persistence_policy.is_some() && audit_policy.is_none(),
            None,
            None,
            "persistence_policy",
        );
        let dependencies = match reference::compile_dependencies(&raw.reference_data) {
            Ok(dependencies) => dependencies,
            Err((code, message)) => {
                checks.error(code, None, None, message);
                vec![]
            }
        };
        let parameters = match joins::compile_parameters(&raw.parameters) {
            Ok(parameters) => parameters,
            Err((code, message)) => {
                checks.error(code, None, None, message);
                vec![]
            }
        };
        let joins = match joins::compile_joins(&raw) {
            Ok(joins) => joins,
            Err((code, message)) => {
                checks.error(code, None, None, message);
                vec![]
            }
        };
        for (trace_index, record) in raw.traces.iter().enumerate() {
            checks.record(record, trace_index, &joins);
        }
        let (issues, regexes) = checks.finish();
        if issues
            .iter()
            .any(|issue| issue.severity == SchemaIssueSeverity::Error)
        {
            return Err(SchemaCompilationError { issues });
        }
        let fail = |trace: Option<&str>, field: Option<&str>, code, message: String| {
            SchemaCompilationError {
                issues: vec![SchemaIssue {
                    severity: SchemaIssueSeverity::Error,
                    code,
                    flow_id: Some(FlowId(raw.flow_id.clone())),
                    trace_id: trace.map(|s| TraceId(s.to_owned())),
                    field_id: field.map(|s| FieldId(s.to_owned())),
                    message,
                    source_note: None,
                }],
            }
        };
        let derived_tables = raw
            .traces
            .iter()
            .map(crate::schema::derived::compile_derived_values)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|(code, message)| fail(None, None, code, message))?;
        let mut local_rules = vec![Vec::new(); raw.traces.len()];
        let mut cross_rules = Vec::new();
        let mut cross_fields = vec![std::collections::BTreeSet::new(); raw.traces.len()];
        for (trace_index, record) in raw.traces.iter().enumerate() {
            let bindings = crate::schema::reference_binding::ReferenceBindings::cross(
                &raw,
                trace_index,
                &joins,
            )
            .map_err(|(code, message)| fail(Some(&record.id), None, code, message))?;
            for rule in rules::compile_rules(record, &regexes, &raw.reference_data, &bindings)
                .map_err(|(code, message)| fail(Some(&record.id), None, code, message))?
            {
                let dependencies = crate::schema::rule_dependencies::RuleDependencies::collect(
                    &rule,
                    trace_index,
                    &joins,
                    &derived_tables,
                )
                .map_err(|(code, message)| fail(Some(&record.id), None, code, message))?;
                if dependencies.joins.is_empty() {
                    local_rules[trace_index].push(rule);
                } else {
                    for &index in &dependencies.joins {
                        let join = joins.get(index).ok_or_else(|| {
                            fail(
                                Some(&record.id),
                                None,
                                Code::InternalCompilerInvariant,
                                "cross join index is inconsistent".into(),
                            )
                        })?;
                        for diagnostic in [
                            Some(join.on_missing_record.diagnostic()),
                            Some(join.on_missing_trace.diagnostic()),
                            join.consensus
                                .as_ref()
                                .map(|plan| &plan.conflict_diagnostic),
                        ]
                        .into_iter()
                        .flatten()
                        {
                            let parts =
                                crate::schema::rule_template::parse(&diagnostic.message_template)
                                    .map_err(|message| {
                                    fail(
                                        Some(&record.id),
                                        None,
                                        Code::InvalidSchemaValue,
                                        message.into(),
                                    )
                                })?;
                            if parts.iter().any(|part| {
                                matches!(
                                    part,
                                    crate::schema::rule_template::Part::Token(
                                        crate::schema::rule_template::Token::FieldId
                                            | crate::schema::rule_template::Token::FieldName
                                    )
                                )
                            }) {
                                return Err(fail(Some(&record.id),None,Code::UnsupportedConstruct,"join diagnostic field placeholders require an explicit field target".into()));
                            }
                        }
                    }
                    if rule.assertions.iter().any(|assertion| {
                        matches!(
                            assertion,
                            CompiledAssertion::Constraint {
                                constraint: CompiledConstraint::ReferenceData { .. },
                                ..
                            }
                        )
                    }) {
                        return Err(fail(
                            Some(&record.id),
                            None,
                            Code::UnsupportedConstruct,
                            "reference assertions in cross rules require full audit integration"
                                .into(),
                        ));
                    }
                    for (trace, field) in dependencies.fields {
                        cross_fields
                            .get_mut(trace)
                            .ok_or_else(|| {
                                fail(
                                    None,
                                    None,
                                    Code::InternalCompilerInvariant,
                                    "cross projection trace is inconsistent".into(),
                                )
                            })?
                            .insert(field);
                    }
                    cross_rules.push(CompiledCrossRecordRule {
                        source_trace: trace_index,
                        rule,
                    });
                }
            }
        }
        let mut traces = Vec::with_capacity(raw.traces.len());
        for (trace_index, ((record, rules), derived)) in raw
            .traces
            .iter()
            .zip(local_rules)
            .zip(derived_tables)
            .enumerate()
        {
            traces.push(
                compile_record(
                    record,
                    &regexes,
                    &raw.reference_data,
                    RecordLowering {
                        trace_index,
                        joins: &joins,
                        rules,
                        derived,
                        additional_fields: cross_fields[trace_index].iter().copied().collect(),
                    },
                )
                .map_err(|(code, message)| fail(Some(&record.id), None, code, message))?,
            );
        }
        CompiledFlowSchema::from_parts(FlowParts {
            key: SchemaKey {
                flow_id: FlowId(raw.flow_id.clone()),
                version: SchemaVersion(raw.schema_version.clone()),
            },
            schema_format_version: raw.schema_format_version,
            regulatory_version: raw.regulatory_version.clone().into(),
            source: raw.source.clone(),
            encoding: EncodingSpec::Utf8, // Only reachable after validating the explicit binding.
            traces,
            required_reference_data: dependencies,
            cross_record_plan: CrossRecordValidationPlan {
                required_traces: (0..raw.traces.len())
                    .filter(|index| {
                        joins
                            .iter()
                            .any(|join| join.source_trace == *index || join.target_trace == *index)
                    })
                    .collect(),
                joins,
                rules: cross_rules,
            },
            parameters,
            required_capabilities: raw
                .required_capabilities
                .iter()
                .map(|capability| {
                    Ok(match capability {
                        RawFlowSchemaRequiredCapabilitiesItem::LocalV1 => Capability::LocalV1,
                        RawFlowSchemaRequiredCapabilitiesItem::JoinsConsensusV1 => {
                            Capability::JoinConsensusV1
                        }
                        RawFlowSchemaRequiredCapabilitiesItem::JoinsV1 => Capability::JoinsV1,
                        RawFlowSchemaRequiredCapabilitiesItem::JoinsSetsV1 => {
                            Capability::JoinSetsV1
                        }
                        RawFlowSchemaRequiredCapabilitiesItem::ExpressionsV1 => {
                            Capability::ExpressionsV1
                        }
                        RawFlowSchemaRequiredCapabilitiesItem::DerivedSubstringV1 => {
                            Capability::DerivedSubstringV1
                        }
                        RawFlowSchemaRequiredCapabilitiesItem::DerivedAgeInCompletedYearsV1 => {
                            Capability::DerivedAgeInCompletedYearsV1
                        }
                        RawFlowSchemaRequiredCapabilitiesItem::DerivedCalendarYearDeltaV1 => {
                            Capability::DerivedCalendarYearDeltaV1
                        }
                        RawFlowSchemaRequiredCapabilitiesItem::PersistenceAuditV1
                            if audit_policy.is_some() =>
                        {
                            Capability::PersistenceAuditV1
                        }
                        _ => {
                            return Err(fail(
                                None,
                                None,
                                Code::UnsupportedCapability,
                                "unsupported capability reached lowering".into(),
                            ));
                        }
                    })
                })
                .collect::<Result<Vec<_>, SchemaCompilationError>>()?,
            persistence_policy: audit_policy,
        })
        .map_err(|error| {
            fail(
                None,
                None,
                Code::InternalCompilerInvariant,
                format!("compiled model invariant failed: {error:?}"),
            )
        })
    }
}

fn alignment(raw: &RawAlignment) -> AlignmentRule {
    match raw {
        RawAlignment::Exact {} => AlignmentRule::Exact,
        RawAlignment::RightPad {
            fill_byte,
            content_policy,
        } => AlignmentRule::RightPad {
            fill: *fill_byte,
            content_policy: match content_policy {
                RawAlignmentRightPadContentPolicy::StripTrailingFill => {
                    ContentBoundaryPolicy::StripTrailingFill
                }
                RawAlignmentRightPadContentPolicy::PreserveWidth => {
                    ContentBoundaryPolicy::PreserveWidth
                }
            },
        },
        RawAlignment::LeftPad {
            fill_byte,
            content_policy,
        } => AlignmentRule::LeftPad {
            fill: *fill_byte,
            content_policy: match content_policy {
                RawAlignmentLeftPadContentPolicy::StripLeadingFill => {
                    ContentBoundaryPolicy::StripLeadingFill
                }
                RawAlignmentLeftPadContentPolicy::PreserveWidth => {
                    ContentBoundaryPolicy::PreserveWidth
                }
            },
        },
    }
}
fn lexical(raw: &RawLexical) -> LexicalPlan {
    match raw {
        RawLexical::EncodedText {} => LexicalPlan::EncodedText,
        RawLexical::ByteSet {
            allowed_bytes,
            lowercase_bytes,
        } => {
            let mut class = ByteClass {
                allowed: [false; 256],
                lowercase: [false; 256],
            };
            for &byte in allowed_bytes {
                class.allowed[usize::from(byte)] = true;
            }
            for &byte in lowercase_bytes {
                class.lowercase[usize::from(byte)] = true;
            }
            LexicalPlan::ByteSet(Box::new(class))
        }
    }
}
fn compile_field(
    raw: &RawFieldSpec,
    regexes: &RegexPlans,
    record: &RawRecordSchema,
    dependencies: &[RawReferenceDataDependency],
) -> Result<CompiledFieldSpec, Code> {
    let (start, length, field_type, raw_alignment, raw_lexical) = match &raw.physical {
        RawPhysical::Numeric {
            start,
            length,
            alignment,
            lexical,
        } => (
            *start,
            *length,
            FieldType::Numeric,
            alignment,
            Some(lexical),
        ),
        RawPhysical::Alphanumeric {
            start,
            length,
            alignment,
            lexical,
        } => (
            *start,
            *length,
            FieldType::Alphanumeric,
            alignment,
            Some(lexical),
        ),
        RawPhysical::Decimal {
            start,
            length,
            scale,
            decimal_separator,
            sign_policy,
            ..
        } => (
            *start,
            *length,
            FieldType::Decimal(
                DecimalSpec::new(
                    usize::try_from(*length).map_err(|_| Code::RangeOverflow)?,
                    usize::try_from(*scale).map_err(|_| Code::RangeOverflow)?,
                    match decimal_separator {
                        RawPhysicalDecimalDecimalSeparator::Dot => b'.',
                    },
                    match sign_policy {
                        RawPhysicalDecimalSignPolicy::Unsigned => SignPolicy::Unsigned,
                    },
                )
                .ok_or(Code::InvalidTypeParameters)?,
            ),
            &RawAlignment::Exact {},
            None,
        ),
        RawPhysical::Date {
            start,
            length,
            format,
            ..
        } => (
            *start,
            *length,
            FieldType::Date(match format {
                RawPhysicalDateFormat::Ddmmyyyy => DateFormat::DdMmYyyy,
            }),
            &RawAlignment::Exact {},
            None,
        ),
    };
    let missing_value_policy = match &raw.missing_value {
        RawMissing::Spaces {} => MissingValuePolicy::Spaces,
        RawMissing::NeverMissing {} => MissingValuePolicy::NeverMissing,
        RawMissing::Exact { raw } => MissingValuePolicy::Exact(raw.as_bytes().to_vec()),
        RawMissing::AnyOf { raw_values } => {
            MissingValuePolicy::AnyOf(raw_values.iter().map(|s| s.as_bytes().to_vec()).collect())
        }
    };
    Ok(CompiledFieldSpec {
        id: FieldId(raw.id.clone()),
        name: raw.name.clone().into(),
        range: source_range(start, length)?,
        field_type,
        missing_value_policy,
        presence_policy: crate::schema::presence::compile_presence(&raw.presence, record)
            .map_err(|(code, _)| code)?,
        alignment: alignment(raw_alignment),
        lexical: raw_lexical.map(lexical),
        // Preflight checked shape/encoding/width; lower each literal once,
        // preserving bytes, target and declared order. Keep a typed refusal
        // here as well instead of silently dropping a future unsupported node.
        constraints: raw
            .constraints
            .iter()
            .filter(|constraint| !matches!(constraint, RawConstraint::ReferenceData { .. }))
            .chain(
                raw.constraints
                    .iter()
                    .filter(|constraint| matches!(constraint, RawConstraint::ReferenceData { .. })),
            )
            .map(|constraint| compile_constraint(constraint, regexes, dependencies))
            .collect::<Result<_, _>>()?,
        regulatory_metadata: raw.regulatory_metadata.clone(),
    })
}
fn compile_constraint(
    constraint: &RawConstraint,
    regexes: &RegexPlans,
    dependencies: &[RawReferenceDataDependency],
) -> Result<CompiledConstraint, Code> {
    match constraint {
        RawConstraint::ReferenceData {
            datasets,
            fallback_values,
            metadata_bindings,
        } => {
            if !metadata_bindings.is_empty() {
                return Err(Code::UnsupportedConstruct);
            }
            if datasets.is_empty() {
                return Err(Code::InvalidSchemaValue);
            }
            Ok(CompiledConstraint::ReferenceData {
                dataset_indexes: datasets
                    .iter()
                    .map(|id| {
                        dependencies
                            .iter()
                            .position(|d| d.id == *id)
                            .ok_or(Code::MissingReferenceDatasetDeclaration)
                    })
                    .collect::<Result<_, _>>()?,
                fallback_values: fallback_values
                    .iter()
                    .map(|value| value.as_bytes().to_vec())
                    .collect(),
                metadata_bindings: vec![],
            })
        }
        RawConstraint::AllowedValues { values, target } => Ok(CompiledConstraint::AllowedValues {
            values: values
                .iter()
                .map(|value| value.as_bytes().to_vec())
                .collect(),
            target: match target {
                RawConstraintAllowedValuesTarget::Raw => ValueTarget::Raw,
                RawConstraintAllowedValuesTarget::Logical => ValueTarget::Logical,
            },
        }),
        RawConstraint::NumericRange { min, max } => Ok(CompiledConstraint::Range {
            min: min
                .0
                .to_string()
                .parse()
                .map_err(|_| Code::InvalidSchemaValue)?,
            max: max
                .0
                .to_string()
                .parse()
                .map_err(|_| Code::InvalidSchemaValue)?,
        }),
        RawConstraint::Regex {
            pattern,
            r#match,
            target,
        } => Ok(CompiledConstraint::Regex {
            regex: regexes
                .get(pattern)
                .ok_or(Code::InternalCompilerInvariant)?
                .as_ref()
                .map_err(|_| Code::InvalidRegex)?
                .clone(),
            mode: match r#match {
                RawConstraintRegexMatch::Required => RegexMatch::Required,
                RawConstraintRegexMatch::Forbidden => RegexMatch::Forbidden,
            },
            target: match target {
                RawConstraintAllowedValuesTarget::Raw => ValueTarget::Raw,
                RawConstraintAllowedValuesTarget::Logical => ValueTarget::Logical,
            },
        }),
        RawConstraint::Date { format, .. } => Ok(CompiledConstraint::Date {
            format: match format {
                RawPhysicalDateFormat::Ddmmyyyy => DateFormat::DdMmYyyy,
            },
        }),
        _ => Err(Code::UnsupportedConstruct),
    }
}

fn index(record: &RawRecordSchema, id: &str) -> Result<usize, (Code, String)> {
    record
        .fields
        .iter()
        .position(|field| field.id == id)
        .ok_or((
            Code::MissingFieldReference,
            "local field reference does not exist".to_owned(),
        ))
}
/// Match the approved raw target vocabulary without deriving identity from a
/// field name. Only technical targets accept a nonempty technical_name.
fn identity_target(raw: &RawIdentitySpec) -> Result<IdentityTarget, (Code, String)> {
    match (&raw.target, &raw.technical_name) {
        (RawIdentitySpecTarget::SiocRecordId, None) => Ok(IdentityTarget::SiocRecordId),
        (RawIdentitySpecTarget::FiscalCode, None) => Ok(IdentityTarget::FiscalCode),
        (RawIdentitySpecTarget::EpisodeId, None) => Ok(IdentityTarget::EpisodeId),
        (RawIdentitySpecTarget::Technical, Some(name)) if !name.is_empty() => {
            Ok(IdentityTarget::Technical(name.as_str().into()))
        }
        _ => Err((
            Code::InvalidSchemaValue,
            "identity technical_name must be nonempty and present only for technical targets"
                .into(),
        )),
    }
}
struct RecordLowering<'a> {
    trace_index: usize,
    joins: &'a [CompiledJoinSpec],
    rules: Vec<CompiledRule>,
    derived: Vec<DerivedValuePlan>,
    additional_fields: Vec<usize>,
}
fn compile_record(
    raw: &RawRecordSchema,
    regexes: &RegexPlans,
    dependencies: &[RawReferenceDataDependency],
    lowering: RecordLowering<'_>,
) -> Result<CompiledRecordSchema, (Code, String)> {
    let fields = raw
        .fields
        .iter()
        .map(|field| compile_field(field, regexes, raw, dependencies))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|code| (code, "field construction failed after preflight".to_owned()))?;
    let key = raw
        .record_key
        .fields
        .iter()
        .map(|id| index(raw, id))
        .collect::<Result<Vec<_>, _>>()?;
    let derived_values = lowering.derived;
    let projection_plan = projection::compile(
        raw,
        lowering.trace_index,
        lowering.joins,
        &derived_values,
        &key,
        &lowering.additional_fields,
    )?;
    let error_code = raw
        .record_key
        .error_code
        .parse::<ValidationCode>()
        .map_err(|_| {
            (
                Code::InvalidDiagnosticCode,
                "invalid record_key diagnostic symbol".to_owned(),
            )
        })?;
    let deferred = raw
        .documented_non_executable_rules
        .iter()
        .map(|rule| {
            Ok(DocumentedNonExecutableRule {
                id: rule.id.clone().into(),
                field_indexes: rule
                    .fields
                    .iter()
                    .map(|id| index(raw, id))
                    .collect::<Result<Vec<_>, _>>()?,
                reason: rule.reason.clone().into(),
                decision_reference: rule.decision_reference.clone().into(),
            })
        })
        .collect::<Result<Vec<_>, (Code, String)>>()?;
    CompiledRecordSchema::from_parts(RecordParts {
        id: TraceId(raw.id.clone()),
        name: raw.name.clone().into(),
        record_length: usize::try_from(raw.record_length).map_err(|_| {
            (
                Code::RangeOverflow,
                "record_length exceeds usize".to_owned(),
            )
        })?,
        fields,
        local_rules: lowering.rules,
        derived_values,
        identity_plan: IdentityExtractionPlan {
            // Preflight checked targets and references; retain typed defensive
            // failures here rather than dropping an unresolved binding.
            bindings: raw
                .identity
                .iter()
                .map(|binding| {
                    Ok(IdentityBinding {
                        target: identity_target(binding)?,
                        field_index: index(raw, &binding.field)?,
                    })
                })
                .collect::<Result<Vec<_>, (Code, String)>>()?,
        },
        projection_plan,
        record_key: RecordKeyPlan {
            field_indexes: key,
            error_code,
        },
        repeated_groups: vec![],
        documented_non_executable_rules: deferred,
    })
    .map_err(|error| {
        (
            Code::InternalCompilerInvariant,
            format!("compiled model invariant failed: {error:?}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_coordinates_are_checked_once_at_the_conversion_boundary() {
        assert_eq!(source_range(1, 3).unwrap(), ByteRange::new(0, 3).unwrap());
        assert_eq!(source_range(4, 2).unwrap(), ByteRange::new(3, 2).unwrap());
        assert_eq!(source_range(0, 3), Err(Code::InvalidPosition));
        assert_eq!(source_range(1, 0), Err(Code::ZeroLength));
        assert_eq!(source_range(u32::MAX, u32::MAX).is_err(), usize::BITS == 32);
    }
    #[test]
    fn construction_invariant_failures_are_typed_without_panics() {
        // Deliberately bypass preflight inside the private construction boundary.
        let mut raw: RawFlowSchema = serde_json::from_str(include_str!(
            "../../docs/schema-authoring/examples/schemas/local-flow.json"
        ))
        .unwrap();
        let record = &mut raw.traces[0];
        record.fields.truncate(1);
        record.record_key.fields = vec![record.fields[0].id.clone()];
        record.documented_non_executable_rules.clear();
        record.rules.clear();
        record.record_length = 1;
        let (code, message) = compile_record(
            record,
            &RegexPlans::new(),
            &[],
            RecordLowering {
                trace_index: 0,
                joins: &[],
                rules: vec![],
                derived: vec![],
                additional_fields: vec![],
            },
        )
        .unwrap_err();
        assert_eq!(code, Code::InternalCompilerInvariant);
        assert!(message.contains("FieldOutsideRecord"));
    }
}

#[path = "rules.rs"]
mod rules;
