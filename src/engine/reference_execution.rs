//! Per-record evidence delivery. The application owns bounded aggregation.
use crate::{
    CompiledConstraint, DatasetAvailability, FieldId, FieldSlice, MembershipEvaluation,
    PreparedReferenceEvidence, ReferenceConstraintEvaluation, ReferenceDependency,
    ReferenceEvaluationError, validate_reference_constraint,
};
use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceConclusion {
    Definitive,
    Provisional,
    Inconclusive,
}

#[derive(Debug, Clone, Copy)]
pub struct ReferenceControl<'a> {
    pub field_id: &'a FieldId,
    pub rule_id: Option<&'a str>,
    /// Field constraint index, or assertion index when rule_id is Some.
    pub index: usize,
}

pub trait ReferenceEvaluationObserver {
    fn on_evaluation(
        &mut self,
        control: ReferenceControl<'_>,
        evaluation: ReferenceConstraintEvaluation<'_>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[derive(Debug)]
pub enum ReferenceExecutionError {
    Evaluation(ReferenceEvaluationError),
    Observer(Box<dyn Error + Send + Sync>),
    InconsistentPreparation,
}
impl fmt::Display for ReferenceExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Evaluation(_) => "reference evaluation failed",
            Self::Observer(_) => "reference evidence observer failed",
            Self::InconsistentPreparation => {
                "reference preparation differs from compiled dependencies"
            }
        })
    }
}
impl Error for ReferenceExecutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Evaluation(e) => Some(e),
            Self::Observer(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

pub(crate) struct ReferenceExecution<'a> {
    pub prepared: &'a PreparedReferenceEvidence,
    pub dependencies: &'a [ReferenceDependency],
    pub observer: &'a mut dyn ReferenceEvaluationObserver,
    pub conclusion: ReferenceConclusion,
}
impl ReferenceExecution<'_> {
    pub fn evaluate(
        &mut self,
        constraint: &CompiledConstraint,
        value: FieldSlice<'_>,
        control: ReferenceControl<'_>,
    ) -> Result<ReferenceConstraintEvaluation<'_>, ReferenceExecutionError> {
        let evaluation =
            validate_reference_constraint(constraint, value, self.dependencies, self.prepared)
                .map_err(ReferenceExecutionError::Evaluation)?;
        match evaluation {
            ReferenceConstraintEvaluation::Membership(MembershipEvaluation::Inconclusive) => {
                self.conclusion = ReferenceConclusion::Inconclusive
            }
            ReferenceConstraintEvaluation::Membership(
                MembershipEvaluation::ProvisionalPresent(evidence),
            ) => {
                debug_assert_eq!(evidence.availability(), DatasetAvailability::Degraded);
                if self.conclusion == ReferenceConclusion::Definitive {
                    self.conclusion = ReferenceConclusion::Provisional;
                }
            }
            _ => {}
        }
        self.observer
            .on_evaluation(control, evaluation)
            .map_err(ReferenceExecutionError::Observer)?;
        Ok(evaluation)
    }
}
