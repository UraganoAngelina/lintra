//! Exact-byte reference constraints; warnings are never ValidationError values.
use crate::{
    CompiledConstraint, FieldSlice, FieldViolation, MembershipEvaluation, MembershipPlanError,
    PreparedReferenceEvidence, ReferenceDependency, ValidationCode,
    reference_data::membership::evaluate_keys,
};
use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy)]
pub enum ReferenceConstraintEvaluation<'a> {
    /// The literal belongs to the compiled schema, not to a verified dataset.
    DeclaredFallback,
    Membership(MembershipEvaluation<'a>),
}
impl ReferenceConstraintEvaluation<'_> {
    pub fn violation(self) -> Option<FieldViolation> {
        matches!(self, Self::Membership(MembershipEvaluation::VerifiedAbsent)).then(|| {
            FieldViolation {
                code: ValidationCode::ReferenceData,
                expected: Some("appartenenza alle sorgenti dichiarate e verificate".into()),
                reason: String::new(),
                diagnostic_context: vec![],
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceEvaluationError {
    InvalidPlan,
    UnsupportedMetadata,
    Membership(MembershipPlanError),
}
impl fmt::Display for ReferenceEvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidPlan => "reference constraint plan is invalid",
            Self::UnsupportedMetadata => "reference metadata is not supported",
            Self::Membership(_) => "reference membership plan failed",
        })
    }
}
impl Error for ReferenceEvaluationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Membership(source) => Some(source),
            _ => None,
        }
    }
}

pub fn validate_reference_constraint<'a>(
    constraint: &CompiledConstraint,
    value: FieldSlice<'_>,
    dependencies: &[ReferenceDependency],
    prepared: &'a PreparedReferenceEvidence,
) -> Result<ReferenceConstraintEvaluation<'a>, ReferenceEvaluationError> {
    let CompiledConstraint::ReferenceData {
        dataset_indexes,
        fallback_values,
        metadata_bindings,
    } = constraint
    else {
        return Err(ReferenceEvaluationError::InvalidPlan);
    };
    if !metadata_bindings.is_empty() {
        return Err(ReferenceEvaluationError::UnsupportedMetadata);
    }
    if dataset_indexes.is_empty()
        || dataset_indexes
            .iter()
            .any(|index| *index >= dependencies.len())
    {
        return Err(ReferenceEvaluationError::InvalidPlan);
    }
    // Validate all dependencies before considering even a declared fallback.
    for &index in dataset_indexes {
        if prepared.evidence(&dependencies[index].key).is_none() {
            return Err(ReferenceEvaluationError::Membership(
                MembershipPlanError::UnpreparedDependency,
            ));
        }
    }
    if fallback_values
        .iter()
        .any(|fallback| fallback.as_slice() == value.as_bytes())
    {
        return Ok(ReferenceConstraintEvaluation::DeclaredFallback);
    }
    evaluate_keys(
        dataset_indexes
            .iter()
            .map(|&index| &dependencies[index].key),
        prepared,
        value.as_bytes(),
    )
    .map(ReferenceConstraintEvaluation::Membership)
    .map_err(ReferenceEvaluationError::Membership)
}
