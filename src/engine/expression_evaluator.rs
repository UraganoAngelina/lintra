//! Three-valued logic (DET §19); expression execution follows in T-P10-03.
use std::ops::Not;

/// Unavailability is diagnostic information, never a false value or a pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownReason {
    FieldUnavailable,
    RelatedRecordMissing,
    TraceInputUnavailable,
    AmbiguousJoin,
}

/// A determinate truth value or an explicitly unavailable decision.
/// No implicit conversion to bool is provided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruthValue {
    True,
    False,
    Unknown(UnknownReason),
}
impl TruthValue {
    /// False determines AND even when the other operand is unavailable.
    /// Otherwise preserve Unknown; two Unknowns retain the left operand's
    /// reason, following declared expression order deterministically.
    pub const fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::False, _) | (_, Self::False) => Self::False,
            (Self::Unknown(reason), _) | (_, Self::Unknown(reason)) => Self::Unknown(reason),
            (Self::True, Self::True) => Self::True,
        }
    }

    /// True determines OR even when the other operand is unavailable.
    /// The Unknown tie policy is the same stable left-first choice as AND.
    pub const fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::True, _) | (_, Self::True) => Self::True,
            (Self::Unknown(reason), _) | (_, Self::Unknown(reason)) => Self::Unknown(reason),
            (Self::False, Self::False) => Self::False,
        }
    }
}
impl Not for TruthValue {
    type Output = Self;

    /// Negation changes determinate truth only, retaining unavailability.
    fn not(self) -> Self::Output {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown(reason) => Self::Unknown(reason),
        }
    }
}
