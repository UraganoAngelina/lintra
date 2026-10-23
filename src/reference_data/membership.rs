//! Trust-aware logical union: no copied sets, filesystem access or bool coercion.
use super::evidence::{DatasetAvailability, DatasetEvidence, PreparedReferenceEvidence};
use crate::DatasetKey;
use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy)]
pub enum MembershipEvaluation<'a> {
    VerifiedPresent(&'a DatasetEvidence),
    VerifiedAbsent,
    ProvisionalPresent(&'a DatasetEvidence),
    Inconclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembershipPlanError {
    EmptyUnion,
    UnpreparedDependency,
}
impl fmt::Display for MembershipPlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyUnion => "membership requires a nonempty union",
            Self::UnpreparedDependency => "membership dependency was not prepared",
        })
    }
}
impl Error for MembershipPlanError {}

/// Validate the whole requested union, even if an early source matches. A
/// missing preparation entry is a plan defect; a registered Unavailable entry
/// is data unavailability and contributes to Inconclusive instead.
pub fn evaluate_membership<'a>(
    required: &[DatasetKey],
    prepared: &'a PreparedReferenceEvidence,
    value: &[u8],
) -> Result<MembershipEvaluation<'a>, MembershipPlanError> {
    evaluate_keys(required.iter(), prepared, value)
}

pub(crate) fn evaluate_keys<'a, 'key>(
    keys: impl Iterator<Item = &'key DatasetKey> + Clone,
    prepared: &'a PreparedReferenceEvidence,
    value: &[u8],
) -> Result<MembershipEvaluation<'a>, MembershipPlanError> {
    if keys.clone().next().is_none() {
        return Err(MembershipPlanError::EmptyUnion);
    }
    for key in keys.clone() {
        if prepared.evidence(key).is_none() {
            return Err(MembershipPlanError::UnpreparedDependency);
        }
    }
    let mut provisional = None;
    let mut all_verified = true;
    for key in keys {
        let observation = prepared
            .observe(key, value)
            .ok_or(MembershipPlanError::UnpreparedDependency)?;
        match (observation.availability(), observation.observed_present()) {
            (DatasetAvailability::Verified, Some(true)) => {
                // Reborrow from the snapshot, not from the temporary observation.
                return Ok(MembershipEvaluation::VerifiedPresent(
                    prepared
                        .evidence(key)
                        .ok_or(MembershipPlanError::UnpreparedDependency)?,
                ));
            }
            (DatasetAvailability::Verified, Some(false)) => {}
            (DatasetAvailability::Degraded, Some(true)) => {
                all_verified = false;
                if provisional.is_none() {
                    provisional = prepared.evidence(key);
                }
            }
            _ => all_verified = false,
        }
    }
    Ok(if let Some(evidence) = provisional {
        MembershipEvaluation::ProvisionalPresent(evidence)
    } else if all_verified {
        MembershipEvaluation::VerifiedAbsent
    } else {
        MembershipEvaluation::Inconclusive
    })
}
