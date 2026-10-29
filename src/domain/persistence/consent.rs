//! A recorded PROCEED for an explicit set of absent traces. This is audit data,
//! not an approval capability and not a policy deciding which traces require it.
use super::{PersistenceTypeError, SubmissionTrace, TraceId, TracePresence, label};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "ConsentWire", try_from = "ConsentWire")]
pub struct MissingTraceProceedConsent {
    traces: Vec<TraceId>,
    recorded_at: DateTime<Utc>,
}
// Internally tagged struct variant rejects unsupported decisions and extra
// fields. In particular ABORT can never deserialize as a durable PROCEED.
#[derive(Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case", deny_unknown_fields)]
enum ConsentWire {
    Proceed {
        traces: Vec<TraceId>,
        #[serde(with = "super::utc")]
        recorded_at: DateTime<Utc>,
    },
}
impl MissingTraceProceedConsent {
    pub fn new(
        mut traces: Vec<TraceId>,
        recorded_at: DateTime<Utc>,
    ) -> Result<Self, PersistenceTypeError> {
        if traces.is_empty() {
            return Err(PersistenceTypeError::InvalidOmissionConsent);
        }
        for trace in &traces {
            label(&trace.0)?;
        }
        traces.sort_by(|a, b| a.0.cmp(&b.0));
        if traces.windows(2).any(|p| p[0] == p[1]) {
            return Err(PersistenceTypeError::InvalidOmissionConsent);
        }
        Ok(Self {
            traces,
            recorded_at,
        })
    }
    pub fn traces(&self) -> &[TraceId] {
        &self.traces
    }
    pub fn recorded_at(&self) -> DateTime<Utc> {
        self.recorded_at
    }
    pub(super) fn validate_inventory(
        &self,
        inventory: &[SubmissionTrace],
    ) -> Result<(), PersistenceTypeError> {
        // A present-empty file is still PRESENT. Consent must name actual
        // omissions; application policy later checks any required coverage.
        if self.traces.iter().any(|id| {
            !inventory
                .iter()
                .any(|t| &t.trace_id == id && t.presence == TracePresence::Absent)
        }) {
            return Err(PersistenceTypeError::InvalidOmissionConsent);
        }
        Ok(())
    }
}
impl fmt::Debug for MissingTraceProceedConsent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MissingTraceProceedConsent { .. }")
    }
}
impl From<MissingTraceProceedConsent> for ConsentWire {
    fn from(value: MissingTraceProceedConsent) -> Self {
        Self::Proceed {
            traces: value.traces,
            recorded_at: value.recorded_at,
        }
    }
}
impl TryFrom<ConsentWire> for MissingTraceProceedConsent {
    type Error = PersistenceTypeError;
    fn try_from(value: ConsentWire) -> Result<Self, Self::Error> {
        let ConsentWire::Proceed {
            traces,
            recorded_at,
        } = value;
        Self::new(traces, recorded_at)
    }
}
