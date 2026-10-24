//! Input byte audit and index coverage are independent facts.
use crate::TraceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceContextCoverage {
    Complete,
    Partial,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceCoverageEvidence {
    pub trace_id: TraceId,
    pub coverage: TraceContextCoverage,
    pub records_processed: u64,
    pub unprojectable_records: u64,
}
