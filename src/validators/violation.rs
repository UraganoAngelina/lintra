use crate::domain::{DiagnosticContextItem, ValidationCode};

/// Lightweight field failure before record metadata is attached (DET §43).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldViolation {
    pub code: ValidationCode,
    pub expected: Option<String>,
    pub reason: String,
    pub diagnostic_context: Vec<DiagnosticContextItem>,
}
