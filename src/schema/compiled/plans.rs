//! Closed, resolved plan payloads. These types do not execute rules or storage.

use crate::domain::{
    ByteRange, DatasetId, DatasetVersion, FieldId, JoinId, SchemaVersion, ValidationCode,
};
use regex::bytes::Regex;
use std::{cmp::Ordering, collections::BTreeMap, error::Error, fmt, str::FromStr, sync::Arc};

/// Exact parsed decimal literal: canonical ASCII magnitude, no machine-size bound.
/// Normalization applies to this numeric value, never to source record bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegerLiteral {
    negative: bool,
    magnitude: Arc<[u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidIntegerLiteral;

impl fmt::Display for InvalidIntegerLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expected a decimal integer literal")
    }
}
impl Error for InvalidIntegerLiteral {}

impl FromStr for IntegerLiteral {
    type Err = InvalidIntegerLiteral;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let digits = text.strip_prefix('-').unwrap_or(text).as_bytes();
        if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
            return Err(InvalidIntegerLiteral);
        }
        let first = digits
            .iter()
            .position(|&digit| digit != b'0')
            .unwrap_or(digits.len() - 1);
        let magnitude = &digits[first..];
        Ok(Self {
            negative: text.starts_with('-') && magnitude != b"0",
            magnitude: Arc::from(magnitude),
        })
    }
}

impl IntegerLiteral {
    pub fn is_negative(&self) -> bool {
        self.negative
    }
    pub fn magnitude(&self) -> &[u8] {
        &self.magnitude
    }
}

impl Ord for IntegerLiteral {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.negative != other.negative {
            return other.negative.cmp(&self.negative);
        }
        let magnitude = self
            .magnitude
            .len()
            .cmp(&other.magnitude.len())
            .then_with(|| self.magnitude.cmp(&other.magnitude));
        if self.negative {
            magnitude.reverse()
        } else {
            magnitude
        }
    }
}
impl PartialOrd for IntegerLiteral {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// An explicitly resolved encoding, never inferred from the host locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodingSpec {
    Utf8,
    Windows1252,
    Iso8859_1,
    Other(Arc<str>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateFormat {
    DdMmYyyy,
}

/// Sign syntax explicitly selected by a compiled Decimal plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignPolicy {
    Unsigned,
}

/// Verified fixed-width decimal representation. Read-only outside compilation;
/// no host locale, inferred scale, floating-point representation or field name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecimalSpec {
    total_length: usize,
    scale: usize,
    decimal_separator: u8,
    sign_policy: SignPolicy,
}
impl DecimalSpec {
    /// Structural safety only; the authoring profile supplies the actual scale
    /// and separator. At least one integral digit and one fractional digit fit.
    pub(super) fn new(
        total_length: usize,
        scale: usize,
        decimal_separator: u8,
        sign_policy: SignPolicy,
    ) -> Option<Self> {
        if scale == 0
            || scale.checked_add(2)? > total_length
            || !decimal_separator.is_ascii_punctuation()
            || matches!(decimal_separator, b'+' | b'-')
        {
            return None;
        }
        Some(Self {
            total_length,
            scale,
            decimal_separator,
            sign_policy,
        })
    }
    pub fn total_length(self) -> usize {
        self.total_length
    }
    pub fn scale(self) -> usize {
        self.scale
    }
    pub fn decimal_separator(self) -> u8 {
        self.decimal_separator
    }
    pub fn sign_policy(self) -> SignPolicy {
        self.sign_policy
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Numeric,
    Alphanumeric,
    Date(DateFormat),
    Decimal(DecimalSpec),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentBoundaryPolicy {
    StripTrailingFill,
    StripLeadingFill,
    PreserveWidth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignmentRule {
    RightPad {
        fill: u8,
        content_policy: ContentBoundaryPolicy,
    },
    LeftPad {
        fill: u8,
        content_policy: ContentBoundaryPolicy,
    },
    Exact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MissingValuePolicy {
    Spaces,
    Exact(Vec<u8>),
    AnyOf(Vec<Vec<u8>>),
    NeverMissing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteClass {
    pub allowed: [bool; 256],
    pub lowercase: [bool; 256],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexicalPlan {
    EncodedText,
    ByteSet(Box<ByteClass>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatasetKey {
    pub id: DatasetId,
    pub version: DatasetVersion,
}

#[derive(Debug, Clone)]
pub struct ReferenceDependency {
    pub key: DatasetKey,
    pub sha256: [u8; 32],
    pub bundle_path: Arc<str>,
    pub metadata_attributes: Vec<Arc<str>>,
}

#[derive(Debug, Clone)]
pub enum ParameterDefinition {
    String {
        id: Arc<str>,
        allowed_values: Vec<Arc<str>>,
    },
    Integer {
        id: Arc<str>,
        min: IntegerLiteral,
        max: IntegerLiteral,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueTarget {
    Raw,
    Logical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOperator {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Bytes,
    Integer,
    Date,
}

/// Parsed literal date; calendar validity is established by the compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateLiteral {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl DateLiteral {
    /// Shared explicit DDMMYYYY value parsing for schema literals and runtime
    /// comparisons. Fixed width/ASCII precede all indexing and calendar checks.
    pub(crate) fn from_ddmmyyyy(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 8 || !bytes.iter().all(u8::is_ascii_digit) {
            return None;
        }
        let number = |digits: &[u8]| {
            digits
                .iter()
                .fold(0_u32, |value, digit| value * 10 + u32::from(digit - b'0'))
        };
        let day = number(&bytes[..2]);
        let month = number(&bytes[2..4]);
        let year = number(&bytes[4..]);
        if year == 0 {
            return None;
        }
        chrono::NaiveDate::from_ymd_opt(i32::try_from(year).ok()?, month, day)?;
        Some(Self {
            year: u16::try_from(year).ok()?,
            month: u8::try_from(month).ok()?,
            day: u8::try_from(day).ok()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompiledValue {
    Bytes(Vec<u8>),
    Integer(IntegerLiteral),
    Date(DateLiteral),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedFieldRef {
    Local {
        field_index: usize,
        field_id: FieldId,
    },
    External {
        trace_index: usize,
        field_index: usize,
        field_id: FieldId,
        join_index: usize,
        join_id: JoinId,
    },
}

#[derive(Debug, Clone)]
pub enum CompiledExpr {
    Const(bool),
    Eq(ResolvedFieldRef, Vec<u8>),
    Ne(ResolvedFieldRef, Vec<u8>),
    In(ResolvedFieldRef, Vec<Vec<u8>>),
    NotIn(ResolvedFieldRef, Vec<Vec<u8>>),
    IsPresent(ResolvedFieldRef),
    IsMissing(ResolvedFieldRef),
    And(Vec<CompiledExpr>),
    Or(Vec<CompiledExpr>),
    Not(Box<CompiledExpr>),
    DerivedEq {
        field: ResolvedFieldRef,
        derived_index: usize,
    },
    DerivedCompare {
        derived_index: usize,
        operator: ComparisonOperator,
        value: CompiledValue,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasePresence {
    Required,
    Optional,
}

#[derive(Debug, Clone)]
pub struct CompiledPresencePolicy {
    pub base: BasePresence,
    pub required_if: Vec<CompiledExpr>,
    pub forbidden_if: Vec<CompiledExpr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeapDayAnniversary {
    Feb28,
    Mar01,
}

#[derive(Debug, Clone)]
pub enum DerivedValuePlan {
    Substring {
        source: ResolvedFieldRef,
        target: ValueTarget,
        range: ByteRange,
    },
    AgeInCompletedYears {
        birth: ResolvedFieldRef,
        reference: ResolvedFieldRef,
        format: DateFormat,
        leap_day: LeapDayAnniversary,
    },
    CalendarYearDelta {
        birth: ResolvedFieldRef,
        reference: ResolvedFieldRef,
        format: DateFormat,
    },
}

#[derive(Debug, Clone)]
pub enum CompiledOperand {
    Field(ResolvedFieldRef),
    Literal(CompiledValue),
}

#[derive(Debug, Clone)]
pub enum MetadataOperand {
    Field(ResolvedFieldRef),
    Derived(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataOperator {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    In,
    NotIn,
}

#[derive(Debug, Clone)]
pub struct MetadataBinding {
    pub attribute: Arc<str>,
    pub operand: MetadataOperand,
    pub operator: MetadataOperator,
    pub value_type: ValueType,
}

#[derive(Debug, Clone)]
pub enum SegmentCheck {
    Equals(Vec<u8>),
    Dataset(usize),
}

#[derive(Debug, Clone)]
pub struct CompiledSegment {
    pub range: ByteRange,
    pub check: SegmentCheck,
}

#[derive(Debug, Clone)]
pub struct CompiledBitPositions {
    pub range: ByteRange,
    pub values: Vec<u8>,
    pub when: CompiledExpr,
    pub otherwise_values: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegexMatch {
    Required,
    Forbidden,
}

#[derive(Debug, Clone)]
pub enum CompiledConstraint {
    AllowedValues {
        values: Vec<Vec<u8>>,
        target: ValueTarget,
    },
    Range {
        min: IntegerLiteral,
        max: IntegerLiteral,
    },
    Regex {
        regex: Regex,
        mode: RegexMatch,
        target: ValueTarget,
    },
    ReferenceData {
        dataset_indexes: Vec<usize>,
        fallback_values: Vec<Vec<u8>>,
        metadata_bindings: Vec<MetadataBinding>,
    },
    LogicalLength {
        equals: usize,
    },
    SubstringNotEquals {
        range: ByteRange,
        value: Vec<u8>,
        target: ValueTarget,
    },
    NoLowercase,
    Date {
        format: DateFormat,
    },
    CompositeSegments {
        target: ValueTarget,
        segments: Vec<CompiledSegment>,
    },
    PositionalBitset {
        positions: Vec<CompiledBitPositions>,
    },
    NonEmpty,
}

#[derive(Debug, Clone)]
pub enum CompiledAssertion {
    Constraint {
        field: ResolvedFieldRef,
        constraint: CompiledConstraint,
    },
    FieldPresent(ResolvedFieldRef),
    FieldCompare {
        left: CompiledOperand,
        operator: ComparisonOperator,
        right: CompiledOperand,
        value_type: ValueType,
    },
    EqualsConcat {
        field: ResolvedFieldRef,
        parts: Vec<ResolvedFieldRef>,
    },
    ConditionTrue(CompiledExpr),
    DerivedCompare {
        derived_index: usize,
        operator: ComparisonOperator,
        value: CompiledValue,
    },
    RecordExists {
        join_index: usize,
    },
    /// Local references in predicate belong to the join target trace.
    SetExists {
        join_index: usize,
        predicate: CompiledExpr,
    },
    SubstringEqualsDerivedYear {
        field: ResolvedFieldRef,
        range: ByteRange,
        source: ResolvedFieldRef,
        format: DateFormat,
        year_digits: YearDigits,
    },
}

impl CompiledAssertion {
    /// A diagnostic target exists only for assertions with a natural operand.
    pub(crate) fn primary_ref(&self) -> Option<&ResolvedFieldRef> {
        match self {
            Self::FieldPresent(field)
            | Self::Constraint { field, .. }
            | Self::SubstringEqualsDerivedYear { field, .. }
            | Self::EqualsConcat { field, .. } => Some(field),
            Self::FieldCompare {
                left: CompiledOperand::Field(field),
                ..
            }
            | Self::FieldCompare {
                right: CompiledOperand::Field(field),
                ..
            } => Some(field),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YearDigits {
    Two,
    Four,
}

#[derive(Debug, Clone)]
pub struct CompiledRule {
    pub id: Arc<str>,
    pub when: CompiledExpr,
    pub assertions: Vec<CompiledAssertion>,
    pub error_code: ValidationCode,
    pub message_template: Arc<str>,
}

#[derive(Debug, Clone)]
pub struct RelatedDiagnostic {
    pub code: Arc<str>,
    pub message_template: Arc<str>,
}

#[derive(Debug, Clone)]
pub enum RelatedPolicy {
    Violation {
        diagnostic: RelatedDiagnostic,
        error_code: ValidationCode,
    },
    NotApplicable {
        diagnostic: RelatedDiagnostic,
    },
    Unknown {
        diagnostic: RelatedDiagnostic,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinCardinality {
    ZeroOrOne,
    ExactlyOne,
    OneOrMany,
    ZeroOrMany,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinScope {
    CurrentSubmission,
    HistoryIncludingCurrent,
}

#[derive(Debug, Clone)]
pub struct ConsensusPlan {
    pub fields: Vec<usize>,
    pub include_if: CompiledExpr,
    /// Conflicting tuples yield Unknown, never an arbitrary scalar value.
    pub conflict_diagnostic: RelatedDiagnostic,
}

#[derive(Debug, Clone)]
pub struct CompiledJoinSpec {
    pub id: JoinId,
    pub source_trace: usize,
    pub target_trace: usize,
    pub source_fields: Vec<usize>,
    pub target_fields: Vec<usize>,
    pub cardinality: JoinCardinality,
    pub scope: JoinScope,
    pub namespace_parameters: Vec<usize>,
    pub on_missing_record: RelatedPolicy,
    pub on_missing_trace: RelatedPolicy,
    pub consensus: Option<ConsensusPlan>,
}

#[derive(Debug, Clone)]
pub enum IdentityTarget {
    SiocRecordId,
    FiscalCode,
    EpisodeId,
    Technical(Arc<str>),
}

#[derive(Debug, Clone)]
pub struct IdentityBinding {
    pub target: IdentityTarget,
    pub field_index: usize,
}

#[derive(Debug, Clone)]
pub struct IdentityExtractionPlan {
    pub bindings: Vec<IdentityBinding>,
}

#[derive(Debug, Clone)]
pub struct ContextProjectionPlan {
    pub field_indexes: Vec<usize>,
    pub join_indexes: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct RecordKeyPlan {
    pub field_indexes: Vec<usize>,
    pub error_code: ValidationCode,
}

#[derive(Debug, Clone)]
pub struct RepeatedGroupPlan {
    pub id: Arc<str>,
    pub fields: Vec<usize>,
    pub required_slots: Vec<usize>,
    pub optional_slots: Vec<usize>,
    pub contiguity_required: bool,
    pub unique_non_missing: bool,
    pub unused_slot_value: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct DocumentedNonExecutableRule {
    pub id: Arc<str>,
    pub field_indexes: Vec<usize>,
    pub reason: Arc<str>,
    pub decision_reference: Arc<str>,
}

#[derive(Debug, Clone)]
pub struct CompiledCrossRecordRule {
    pub source_trace: usize,
    pub rule: CompiledRule,
}

#[derive(Debug, Clone)]
pub struct CrossRecordValidationPlan {
    pub joins: Vec<CompiledJoinSpec>,
    pub rules: Vec<CompiledCrossRecordRule>,
    pub required_traces: Vec<usize>,
}

#[derive(Debug, Clone)]
pub enum VersionCompatibility {
    SameSchemaVersion,
    DeclaredGroup {
        id: Arc<str>,
        schema_versions: Vec<SchemaVersion>,
    },
}

#[derive(Debug, Clone)]
pub struct PolicyKey {
    pub trace_index: usize,
}

#[derive(Debug, Clone)]
pub struct HistoricalIdentityInvariant {
    pub trace_index: usize,
    pub progressive_fields: Vec<usize>,
    pub event_fields: Vec<usize>,
    pub on_reassignment_error: ValidationCode,
    pub reactivate_same_event: bool,
}

/// The only supported snapshot contract: explicit consent, immutable approval,
/// atomic/idempotent commit, absent traces preserved, present traces complete.
#[derive(Debug, Clone)]
pub struct SnapshotPlan {
    pub scope_parameters: Vec<usize>,
    pub year_parameter: usize,
    pub quarter_parameter: usize,
    pub missing_trace_warning_code: Arc<str>,
}

/// Open interval; a missing end extends to the evaluation horizon.
#[derive(Debug, Clone)]
pub struct IntervalExclusionPlan {
    pub owner_trace: usize,
    pub when: CompiledExpr,
    pub point_field: usize,
    pub interval_join: usize,
    pub start_field: usize,
    pub end_field: usize,
    pub on_missing_trace: RelatedPolicy,
    pub error_code: ValidationCode,
}

/// Strictly after the conclusion date; equality is outside the restriction.
#[derive(Debug, Clone)]
pub struct AfterDateRestrictionPlan {
    pub owner_trace: usize,
    pub when: CompiledExpr,
    pub date_field: usize,
    pub conclusion_join: usize,
    pub conclusion_date_field: usize,
    pub target_when: CompiledExpr,
    pub restricted_field: usize,
    pub forbidden_values: Vec<Vec<u8>>,
    pub on_missing_trace: RelatedPolicy,
    pub error_code: ValidationCode,
}

/// Calendar months with end-of-month clamp; pause union strictly inside,
/// extension to fixed point; quarter-end horizon and no second closure.
#[derive(Debug, Clone)]
pub struct InactivityWithPausesPlan {
    pub owner_trace: usize,
    pub evaluations_join: usize,
    pub suspensions_join: usize,
    pub accesses_join: usize,
    pub provisions_join: usize,
    pub access_fields: Vec<usize>,
    pub provision_fields: Vec<usize>,
    pub start_field: usize,
    pub access_date_field: usize,
    pub suspension_start_field: usize,
    pub suspension_end_field: usize,
    pub conclusion_date_field: usize,
    pub conclusion_code_field: usize,
    pub required_conclusion_code: Vec<u8>,
    pub calendar_months: u32,
    pub on_missing_trace: RelatedPolicy,
    pub error_code: ValidationCode,
}

#[derive(Debug, Clone)]
pub enum TemporalPlan {
    IntervalExclusion(IntervalExclusionPlan),
    AfterDateRestriction(AfterDateRestrictionPlan),
    InactivityWithPauses(Box<InactivityWithPausesPlan>),
}

/// All plans require externally configured secure storage; no storage driver here.
#[derive(Debug, Clone)]
pub struct PersistencePolicy {
    pub namespace_parameters: Vec<usize>,
    pub version_compatibility: VersionCompatibility,
    pub run_uniqueness: Vec<PolicyKey>,
    pub cross_run_audit: Vec<PolicyKey>,
    pub history_identity_invariants: Vec<HistoricalIdentityInvariant>,
    pub snapshot: Option<SnapshotPlan>,
    pub temporal_rules: Vec<TemporalPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    LocalV1,
    ExpressionsV1,
    JoinsV1,
    JoinSetsV1,
    JoinConsensusV1,
    DerivedSubstringV1,
    DerivedAgeInCompletedYearsV1,
    DerivedCalendarYearDeltaV1,
    ReferenceMetadataV1,
    PersistenceAuditV1,
    PersistenceSnapshotV1,
    HistoryExistenceV1,
    TemporalIntervalsV1,
    TemporalAfterDateV1,
    TemporalInactivityV1,
}

pub type RegulatoryMetadata = BTreeMap<String, serde_json::Value>;
