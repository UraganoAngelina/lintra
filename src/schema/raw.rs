//! Raw format-1/2 DTO superset, generated from the approved flow-v2.schema.json.
//! Format-specific acceptance belongs to lint/compiler; raw data is not executable.
//! Not executable schemas: bounds, references and compatibility require lint/compile.
//! Regenerate using docs/verification/generate_raw_model.py --write.

use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Number, Value};
use std::{collections::BTreeMap, fmt, marker::PhantomData};

/// Adapter-supplied binding, excluded from authoring JSON and never inferred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawResolvedEncoding {
    Utf8,
    Windows1252,
    Iso8859_1,
    Other(String),
}

/// Exact JSON integer, including values beyond i64/u64; no rounding via float.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Number", into = "Number")]
pub struct RawInteger(pub Number);

impl TryFrom<Number> for RawInteger {
    type Error = &'static str;
    fn try_from(number: Number) -> Result<Self, Self::Error> {
        let text = number.to_string();
        let digits = text.strip_prefix('-').unwrap_or(&text);
        if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
            Ok(Self(number))
        } else {
            Err("expected a JSON integer without fraction or exponent")
        }
    }
}
impl From<RawInteger> for Number {
    fn from(value: RawInteger) -> Self {
        value.0
    }
}

fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}
fn required_option<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}
fn unique_map<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<BTreeMap<String, T>, D::Error> {
    struct Unique<T>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>> Visitor<'de> for Unique<T> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("an object with unique keys")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut values = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, T>()? {
                if values.insert(key, value).is_some() {
                    return Err(A::Error::custom("duplicate map key"));
                }
            }
            Ok(values)
        }
    }
    d.deserialize_map(Unique(PhantomData))
}

// Grammar SHA-256: c1a01f9533bc6679b29f789776181544f94a6a7d570b66bb590bd2a0d7017f92

pub type RawByte = u8;

pub type RawSymbolicError = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDiagnostic {
    pub code: String,
    pub message_template: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRefLocal {
    pub field: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRefExternal {
    pub trace: String,
    pub field: String,
    pub join: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawRef {
    Local(RawRefLocal),
    External(RawRefExternal),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawValue {
    Text(String),
    Integer(RawInteger),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawExprEq {
    pub r#ref: RawRef,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawExprNe {
    pub r#ref: RawRef,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawExprIn {
    pub r#ref: RawRef,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawExprNotIn {
    pub r#ref: RawRef,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawExprIsPresent {
    pub r#ref: RawRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawExprIsMissing {
    pub r#ref: RawRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawExprDerivedEq {
    pub r#ref: RawRef,
    pub derived_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawExprDerivedCompare {
    pub derived_value: String,
    pub operator: RawOperator,
    pub value: RawValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawExpr {
    #[serde(rename = "eq")]
    Eq(RawExprEq),
    #[serde(rename = "ne")]
    Ne(RawExprNe),
    #[serde(rename = "in")]
    In(RawExprIn),
    #[serde(rename = "not_in")]
    NotIn(RawExprNotIn),
    #[serde(rename = "is_present")]
    IsPresent(RawExprIsPresent),
    #[serde(rename = "is_missing")]
    IsMissing(RawExprIsMissing),
    #[serde(rename = "and")]
    And(Vec<RawExpr>),
    #[serde(rename = "or")]
    Or(Vec<RawExpr>),
    #[serde(rename = "not")]
    Not(Box<RawExpr>),
    #[serde(rename = "condition")]
    Condition(String),
    #[serde(rename = "const")]
    Const(bool),
    #[serde(rename = "derived_eq")]
    DerivedEq(RawExprDerivedEq),
    #[serde(rename = "derived_compare")]
    DerivedCompare(RawExprDerivedCompare),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawPresenceBase {
    #[serde(rename = "required")]
    Required,
    #[serde(rename = "optional")]
    Optional,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPresence {
    pub base: RawPresenceBase,
    pub required_if: Vec<RawExpr>,
    pub forbidden_if: Vec<RawExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum RawMissing {
    #[serde(rename = "spaces")]
    Spaces {},
    #[serde(rename = "exact")]
    Exact { raw: String },
    #[serde(rename = "any_of")]
    AnyOf { raw_values: Vec<String> },
    #[serde(rename = "never_missing")]
    NeverMissing {},
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawAlignmentRightPadContentPolicy {
    #[serde(rename = "strip_trailing_fill")]
    StripTrailingFill,
    #[serde(rename = "preserve_width")]
    PreserveWidth,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawAlignmentLeftPadContentPolicy {
    #[serde(rename = "strip_leading_fill")]
    StripLeadingFill,
    #[serde(rename = "preserve_width")]
    PreserveWidth,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum RawAlignment {
    #[serde(rename = "right_pad")]
    RightPad {
        fill_byte: RawByte,
        content_policy: RawAlignmentRightPadContentPolicy,
    },
    #[serde(rename = "left_pad")]
    LeftPad {
        fill_byte: RawByte,
        content_policy: RawAlignmentLeftPadContentPolicy,
    },
    #[serde(rename = "exact")]
    Exact {},
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum RawLexical {
    #[serde(rename = "encoded_text")]
    EncodedText {},
    #[serde(rename = "byte_set")]
    ByteSet {
        allowed_bytes: Vec<RawByte>,
        lowercase_bytes: Vec<RawByte>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawPhysicalDateFormat {
    #[serde(rename = "DDMMYYYY")]
    Ddmmyyyy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawPhysicalDateAlignmentKind {
    #[serde(rename = "exact")]
    Exact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPhysicalDateAlignment {
    pub kind: RawPhysicalDateAlignmentKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawPhysicalDecimalDecimalSeparator {
    #[serde(rename = ".")]
    Dot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawPhysicalDecimalSignPolicy {
    #[serde(rename = "unsigned")]
    Unsigned,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPhysicalDecimalAlignment {
    pub kind: RawPhysicalDateAlignmentKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum RawPhysical {
    #[serde(rename = "numeric")]
    Numeric {
        start: u32,
        length: u32,
        alignment: RawAlignment,
        lexical: RawLexical,
    },
    #[serde(rename = "alphanumeric")]
    Alphanumeric {
        start: u32,
        length: u32,
        alignment: RawAlignment,
        lexical: RawLexical,
    },
    #[serde(rename = "date")]
    Date {
        start: u32,
        length: u32,
        format: RawPhysicalDateFormat,
        alignment: RawPhysicalDateAlignment,
    },
    #[serde(rename = "decimal")]
    Decimal {
        start: u32,
        length: u32,
        scale: u32,
        decimal_separator: RawPhysicalDecimalDecimalSeparator,
        sign_policy: RawPhysicalDecimalSignPolicy,
        alignment: RawPhysicalDecimalAlignment,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawMetadataBindingOperator {
    #[serde(rename = "eq")]
    Eq,
    #[serde(rename = "ne")]
    Ne,
    #[serde(rename = "lt")]
    Lt,
    #[serde(rename = "le")]
    Le,
    #[serde(rename = "gt")]
    Gt,
    #[serde(rename = "ge")]
    Ge,
    #[serde(rename = "in")]
    In,
    #[serde(rename = "not_in")]
    NotIn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawMetadataBindingValueType {
    #[serde(rename = "bytes")]
    Bytes,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "date")]
    Date,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMetadataBindingOperandVariant1 {
    pub r#ref: RawRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMetadataBindingOperandVariant2 {
    pub derived_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawMetadataBindingOperand {
    Variant1(RawMetadataBindingOperandVariant1),
    Variant2(RawMetadataBindingOperandVariant2),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMetadataBinding {
    pub attribute: String,
    pub operator: RawMetadataBindingOperator,
    pub value_type: RawMetadataBindingValueType,
    pub operand: RawMetadataBindingOperand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSegmentCheckVariant1 {
    pub equals: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSegmentCheckVariant2 {
    pub dataset: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawSegmentCheck {
    Variant1(RawSegmentCheckVariant1),
    Variant2(RawSegmentCheckVariant2),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSegment {
    pub start: u32,
    pub length: u32,
    pub check: RawSegmentCheck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPositions {
    pub start: u32,
    pub length: u32,
    pub values: Vec<String>,
    pub when: Box<RawExpr>,
    pub otherwise_values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawConstraintAllowedValuesTarget {
    #[serde(rename = "raw")]
    Raw,
    #[serde(rename = "logical")]
    Logical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawConstraintRegexMatch {
    #[serde(rename = "required")]
    Required,
    #[serde(rename = "forbidden")]
    Forbidden,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum RawConstraint {
    #[serde(rename = "allowed_values")]
    AllowedValues {
        values: Vec<String>,
        target: RawConstraintAllowedValuesTarget,
    },
    #[serde(rename = "numeric_range")]
    NumericRange { min: RawInteger, max: RawInteger },
    #[serde(rename = "regex")]
    Regex {
        pattern: String,
        r#match: RawConstraintRegexMatch,
        target: RawConstraintAllowedValuesTarget,
    },
    #[serde(rename = "reference_data")]
    ReferenceData {
        datasets: Vec<String>,
        fallback_values: Vec<String>,
        metadata_bindings: Vec<RawMetadataBinding>,
    },
    #[serde(rename = "logical_length")]
    LogicalLength { equals: u32 },
    #[serde(rename = "substring_not_equals")]
    SubstringNotEquals {
        start: u32,
        length: u32,
        value: String,
        target: RawConstraintAllowedValuesTarget,
    },
    #[serde(rename = "no_lowercase")]
    NoLowercase {},
    #[serde(rename = "date")]
    Date {
        format: RawPhysicalDateFormat,
        calendar_valid: bool,
    },
    #[serde(rename = "composite_segments")]
    CompositeSegments {
        target: RawConstraintAllowedValuesTarget,
        segments: Vec<RawSegment>,
    },
    #[serde(rename = "positional_bitset")]
    PositionalBitset { positions: Vec<RawPositions> },
    #[serde(rename = "non_empty")]
    NonEmpty {},
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFieldSpec {
    pub id: String,
    pub name: String,
    pub physical: RawPhysical,
    pub missing_value: RawMissing,
    pub presence: RawPresence,
    pub constraints: Vec<RawConstraint>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub regulatory_metadata: Option<BTreeMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawDerivedAgeInCompletedYearsLeapDayAnniversary {
    #[serde(rename = "feb28")]
    Feb28,
    #[serde(rename = "mar01")]
    Mar01,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum RawDerived {
    #[serde(rename = "substring")]
    Substring {
        source_ref: RawRef,
        target: RawConstraintAllowedValuesTarget,
        start: u32,
        length: u32,
    },
    #[serde(rename = "age_in_completed_years")]
    AgeInCompletedYears {
        birth_date_ref: RawRef,
        reference_date_ref: RawRef,
        date_format: RawPhysicalDateFormat,
        leap_day_anniversary: RawDerivedAgeInCompletedYearsLeapDayAnniversary,
    },
    #[serde(rename = "calendar_year_delta")]
    CalendarYearDelta {
        birth_date_ref: RawRef,
        reference_date_ref: RawRef,
        date_format: RawPhysicalDateFormat,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawRelatedPolicyPolicy {
    #[serde(rename = "violation")]
    Violation,
    #[serde(rename = "not_applicable")]
    NotApplicable,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRelatedPolicy {
    pub policy: RawRelatedPolicyPolicy,
    pub diagnostic: RawDiagnostic,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub error_code: Option<RawSymbolicError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawReductionKind {
    #[serde(rename = "consensus")]
    Consensus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawReductionOnConflict {
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawReduction {
    pub kind: RawReductionKind,
    pub fields: Vec<String>,
    pub include_if: Box<RawExpr>,
    pub on_conflict: RawReductionOnConflict,
    pub diagnostic: RawDiagnostic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawJoinSpecCardinality {
    #[serde(rename = "zero_or_one")]
    ZeroOrOne,
    #[serde(rename = "exactly_one")]
    ExactlyOne,
    #[serde(rename = "one_or_many")]
    OneOrMany,
    #[serde(rename = "zero_or_many")]
    ZeroOrMany,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawJoinSpecScope {
    #[serde(rename = "current_submission")]
    CurrentSubmission,
    #[serde(rename = "history_including_current")]
    HistoryIncludingCurrent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawJoinSpec {
    pub id: String,
    pub target_trace: String,
    pub source_fields: Vec<String>,
    pub target_fields: Vec<String>,
    pub cardinality: RawJoinSpecCardinality,
    pub scope: RawJoinSpecScope,
    pub namespace_parameters: Vec<String>,
    pub on_missing_record: RawRelatedPolicy,
    pub on_missing_trace: RawRelatedPolicy,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub reduction: Option<RawReduction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawOperand {
    #[serde(rename = "ref")]
    Ref(RawRef),
    #[serde(rename = "literal")]
    Literal(RawValue),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawOperator {
    #[serde(rename = "==")]
    Eq,
    #[serde(rename = "!=")]
    Ne,
    #[serde(rename = "<")]
    Lt,
    #[serde(rename = "<=")]
    Le,
    #[serde(rename = ">")]
    Gt,
    #[serde(rename = ">=")]
    Ge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum RawAssertion {
    #[serde(rename = "constraint")]
    Constraint {
        field: RawRef,
        constraint: RawConstraint,
    },
    #[serde(rename = "field_present")]
    FieldPresent { field: RawRef },
    #[serde(rename = "field_compare")]
    FieldCompare {
        left: RawOperand,
        operator: RawOperator,
        right: RawOperand,
        value_type: RawMetadataBindingValueType,
    },
    #[serde(rename = "equals_concat")]
    EqualsConcat { field: RawRef, parts: Vec<RawRef> },
    #[serde(rename = "condition_true")]
    ConditionTrue { condition: Box<RawExpr> },
    #[serde(rename = "derived_compare")]
    DerivedCompare {
        derived_value: String,
        operator: RawOperator,
        value: RawValue,
    },
    #[serde(rename = "record_exists")]
    RecordExists { join: String },
    #[serde(rename = "set_exists")]
    SetExists {
        join: String,
        predicate: Box<RawExpr>,
    },
    #[serde(rename = "substring_equals_derived_year")]
    SubstringEqualsDerivedYear {
        field: RawRef,
        start: u32,
        length: u32,
        source_ref: RawRef,
        source_format: RawPhysicalDateFormat,
        year_digits: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRule {
    pub id: String,
    pub when: Box<RawExpr>,
    pub assertions: Vec<RawAssertion>,
    pub error_code: RawSymbolicError,
    pub message_template: String,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub regulatory_metadata: Option<BTreeMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawRecordKeyUniquenessScope {
    #[serde(rename = "trace")]
    Trace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRecordKey {
    pub fields: Vec<String>,
    pub uniqueness_scope: RawRecordKeyUniquenessScope,
    pub error_code: RawSymbolicError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawIdentitySpecTarget {
    #[serde(rename = "sioc_record_id")]
    SiocRecordId,
    #[serde(rename = "fiscal_code")]
    FiscalCode,
    #[serde(rename = "episode_id")]
    EpisodeId,
    #[serde(rename = "technical")]
    Technical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawIdentitySpec {
    pub target: RawIdentitySpecTarget,
    pub field: String,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub technical_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawGroup {
    pub id: String,
    pub fields: Vec<String>,
    pub required_slots: Vec<String>,
    pub optional_slots: Vec<String>,
    pub contiguity_required: bool,
    pub unique_non_missing: bool,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub unused_slot_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawDeferredRulePolicy {
    #[serde(rename = "not_enforced_in_this_version")]
    NotEnforcedInThisVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDeferredRule {
    pub id: String,
    pub fields: Vec<String>,
    pub policy: RawDeferredRulePolicy,
    pub reason: String,
    pub decision_reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRecordSchema {
    pub id: String,
    pub name: String,
    pub record_length: u32,
    pub fields: Vec<RawFieldSpec>,
    pub record_key: RawRecordKey,
    pub joins: Vec<RawJoinSpec>,
    pub identity: Vec<RawIdentitySpec>,
    #[serde(deserialize_with = "unique_map")]
    pub named_conditions: BTreeMap<String, Box<RawExpr>>,
    #[serde(deserialize_with = "unique_map")]
    pub derived_values: BTreeMap<String, RawDerived>,
    pub repeated_groups: Vec<RawGroup>,
    pub rules: Vec<RawRule>,
    pub documented_non_executable_rules: Vec<RawDeferredRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum RawParameter {
    #[serde(rename = "string")]
    String {
        id: String,
        allowed_values: Vec<String>,
    },
    #[serde(rename = "integer")]
    Integer {
        id: String,
        min: RawInteger,
        max: RawInteger,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawReferenceDataDependency {
    pub id: String,
    pub dataset_id: String,
    pub version: String,
    pub sha256: String,
    pub path: RawBundlePath,
    pub metadata_attributes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawPolicyKeyKey {
    #[serde(rename = "record_key")]
    RecordKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPolicyKey {
    pub trace: String,
    pub key: RawPolicyKeyKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawInvariant {
    pub trace: String,
    pub progressive_fields: Vec<String>,
    pub event_fields: Vec<String>,
    pub on_reassignment_error: RawSymbolicError,
    pub reactivate_same_event: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSnapshotPeriod {
    pub year_parameter: String,
    pub quarter_parameter: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawSnapshotCommit {
    #[serde(rename = "atomic_idempotent")]
    AtomicIdempotent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawSnapshotMissingTracePolicyConsent {
    #[serde(rename = "proceed_or_abort")]
    ProceedOrAbort,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawSnapshotMissingTracePolicyOnAbsentTrace {
    #[serde(rename = "preserve")]
    Preserve,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSnapshotMissingTracePolicy {
    pub consent: RawSnapshotMissingTracePolicyConsent,
    pub warning_code: String,
    pub on_absent_trace: RawSnapshotMissingTracePolicyOnAbsentTrace,
    pub present_trace_is_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSnapshot {
    pub scope_parameters: Vec<String>,
    pub period: RawSnapshotPeriod,
    pub approved_immutable: bool,
    pub require_explicit_approval: bool,
    pub commit: RawSnapshotCommit,
    pub missing_trace_policy: RawSnapshotMissingTracePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawTemporalIntervalExclusionBoundaries {
    #[serde(rename = "open")]
    Open,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawTemporalIntervalExclusionMissingEnd {
    #[serde(rename = "open_until_horizon")]
    OpenUntilHorizon,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawTemporalAfterDateRestrictionComparison {
    #[serde(rename = "strict_after")]
    StrictAfter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawTemporalInactivityWithPausesThresholdInvalidDay {
    #[serde(rename = "clamp_to_month_end")]
    ClampToMonthEnd,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawTemporalInactivityWithPausesThresholdPauseDays {
    #[serde(rename = "strictly_inside_union")]
    StrictlyInsideUnion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawTemporalInactivityWithPausesThresholdExtension {
    #[serde(rename = "until_fixed_point")]
    UntilFixedPoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTemporalInactivityWithPausesThreshold {
    pub calendar_months: u32,
    pub invalid_day: RawTemporalInactivityWithPausesThresholdInvalidDay,
    pub pause_days: RawTemporalInactivityWithPausesThresholdPauseDays,
    pub extension: RawTemporalInactivityWithPausesThresholdExtension,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawTemporalInactivityWithPausesHorizon {
    #[serde(rename = "session_quarter_end")]
    SessionQuarterEnd,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawTemporalInactivityWithPausesNoActivity {
    #[serde(rename = "use_event_start")]
    UseEventStart,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawTemporalInactivityWithPausesAlreadyClosed {
    #[serde(rename = "no_second_closure")]
    NoSecondClosure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum RawTemporal {
    #[serde(rename = "interval_exclusion")]
    IntervalExclusion {
        owner_trace: String,
        when: Box<RawExpr>,
        point_field: String,
        interval_join: String,
        start_field: String,
        end_field: String,
        boundaries: RawTemporalIntervalExclusionBoundaries,
        missing_end: RawTemporalIntervalExclusionMissingEnd,
        on_missing_trace: RawRelatedPolicy,
        error_code: RawSymbolicError,
    },
    #[serde(rename = "after_date_restriction")]
    AfterDateRestriction {
        owner_trace: String,
        when: Box<RawExpr>,
        date_field: String,
        conclusion_join: String,
        conclusion_date_field: String,
        target_when: Box<RawExpr>,
        restricted_field: String,
        forbidden_values: Vec<String>,
        comparison: RawTemporalAfterDateRestrictionComparison,
        on_missing_trace: RawRelatedPolicy,
        error_code: RawSymbolicError,
    },
    #[serde(rename = "inactivity_with_pauses")]
    InactivityWithPauses {
        owner_trace: String,
        evaluations_join: String,
        suspensions_join: String,
        accesses_join: String,
        provisions_join: String,
        access_fields: Vec<String>,
        provision_fields: Vec<String>,
        start_field: String,
        access_date_field: String,
        suspension_start_field: String,
        suspension_end_field: String,
        conclusion_date_field: String,
        conclusion_code_field: String,
        required_conclusion_code: String,
        threshold: RawTemporalInactivityWithPausesThreshold,
        horizon: RawTemporalInactivityWithPausesHorizon,
        no_activity: RawTemporalInactivityWithPausesNoActivity,
        already_closed: RawTemporalInactivityWithPausesAlreadyClosed,
        on_missing_trace: RawRelatedPolicy,
        error_code: RawSymbolicError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum RawPersistenceVersionCompatibility {
    #[serde(rename = "same_schema_version")]
    SameSchemaVersion {},
    #[serde(rename = "declared_group")]
    DeclaredGroup {
        id: String,
        schema_versions: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawPersistenceStorageProtection {
    #[serde(rename = "external_secure_configuration_required")]
    ExternalSecureConfigurationRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPersistence {
    pub namespace_parameters: Vec<String>,
    pub version_compatibility: RawPersistenceVersionCompatibility,
    pub run_uniqueness: Vec<RawPolicyKey>,
    pub cross_run_audit: Vec<RawPolicyKey>,
    pub history_identity_invariants: Vec<RawInvariant>,
    #[serde(deserialize_with = "required_option")]
    pub snapshot: Option<RawSnapshot>,
    pub temporal_rules: Vec<RawTemporal>,
    pub storage_protection: RawPersistenceStorageProtection,
}

pub type RawBundlePath = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawEncodingSpecMode {
    #[serde(rename = "runtime_required")]
    RuntimeRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawEncodingSpecPositionUnit {
    #[serde(rename = "byte")]
    Byte,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawEncodingSpec {
    pub mode: RawEncodingSpecMode,
    pub position_unit: RawEncodingSpecPositionUnit,
    /// Explicit adapter binding, never part of authoring JSON.
    #[serde(skip)]
    pub resolved: Option<RawResolvedEncoding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawFlowSchemaRequiredCapabilitiesItem {
    #[serde(rename = "local.v1")]
    LocalV1,
    #[serde(rename = "expressions.v1")]
    ExpressionsV1,
    #[serde(rename = "joins.v1")]
    JoinsV1,
    #[serde(rename = "joins.sets.v1")]
    JoinsSetsV1,
    #[serde(rename = "joins.consensus.v1")]
    JoinsConsensusV1,
    #[serde(rename = "derived.substring.v1")]
    DerivedSubstringV1,
    #[serde(rename = "derived.age_in_completed_years.v1")]
    DerivedAgeInCompletedYearsV1,
    #[serde(rename = "derived.calendar_year_delta.v1")]
    DerivedCalendarYearDeltaV1,
    #[serde(rename = "reference_metadata.v1")]
    ReferenceMetadataV1,
    #[serde(rename = "persistence.audit.v1")]
    PersistenceAuditV1,
    #[serde(rename = "persistence.snapshot.v1")]
    PersistenceSnapshotV1,
    #[serde(rename = "history.existence.v1")]
    HistoryExistenceV1,
    #[serde(rename = "temporal.intervals.v1")]
    TemporalIntervalsV1,
    #[serde(rename = "temporal.after_date.v1")]
    TemporalAfterDateV1,
    #[serde(rename = "temporal.inactivity.v1")]
    TemporalInactivityV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFlowSchema {
    pub schema_format_version: u32,
    pub flow_id: String,
    pub schema_version: String,
    pub regulatory_version: String,
    #[serde(deserialize_with = "unique_map")]
    pub source: BTreeMap<String, Value>,
    pub encoding: RawEncodingSpec,
    pub required_capabilities: Vec<RawFlowSchemaRequiredCapabilitiesItem>,
    pub parameters: Vec<RawParameter>,
    pub reference_data: Vec<RawReferenceDataDependency>,
    pub traces: Vec<RawRecordSchema>,
    #[serde(deserialize_with = "required_option")]
    pub persistence_policy: Option<RawPersistence>,
}

#[cfg(test)]
#[path = "raw_tests.rs"]
mod tests;
