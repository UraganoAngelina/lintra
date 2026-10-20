use crate::{
    domain::{
        DiagnosticValue, FieldDiagnostic, FlowId, RecordIdentity, SchemaVersion, TraceId,
        ValidationCode, ValidationError,
    },
    input::encoding::render_diagnostic_value,
    schema::compiled::{CompiledFieldSpec, EncodingSpec},
    validators::FieldViolation,
};

/// Central diagnostic construction (DET §50).
/// Callers supply preflight-validated IDs and the reader's one-based line number.
/// Manually assembled metadata can be checked with ValidationError::validate.
pub struct ErrorFactory<'a> {
    pub flow_id: &'a FlowId,
    pub schema_version: &'a SchemaVersion,
    pub trace_id: &'a TraceId,
    pub encoding: &'a EncodingSpec,
}

impl ErrorFactory<'_> {
    pub fn field_error(
        &self,
        line_number: u64,
        identity: &RecordIdentity,
        field: &CompiledFieldSpec,
        raw_value: &[u8],
        violation: FieldViolation,
    ) -> ValidationError {
        let message = field_message(field, &violation);
        ValidationError {
            code: violation.code,
            flow_id: self.flow_id.clone(),
            schema_version: self.schema_version.clone(),
            trace_id: self.trace_id.clone(),
            line_number,
            record_identity: identity.clone(),
            field: Some(FieldDiagnostic::from_range(
                field.id().clone(),
                field.name().into(),
                field.range(),
            )),
            actual: Some(render_diagnostic_value(raw_value, self.encoding)),
            expected: violation.expected,
            context: violation.diagnostic_context,
            message,
        }
    }

    pub fn record_length_error(
        &self,
        line_number: u64,
        identity: Option<&RecordIdentity>,
        expected: usize,
        actual: usize,
    ) -> ValidationError {
        ValidationError {
            code: ValidationCode::RecordLength,
            flow_id: self.flow_id.clone(),
            schema_version: self.schema_version.clone(),
            trace_id: self.trace_id.clone(),
            line_number,
            record_identity: identity.cloned().unwrap_or_default(),
            field: None,
            actual: Some(DiagnosticValue {
                display: actual.to_string(),
                raw_hex: None,
            }),
            expected: Some(format!("{expected} byte")),
            context: Vec::new(),
            message: format!(
                "Lunghezza record non valida: attesi {expected} byte, letti {actual}."
            ),
        }
    }
}

fn field_message(field: &CompiledFieldSpec, violation: &FieldViolation) -> String {
    let description = match &violation.code {
        ValidationCode::RecordLength => "lunghezza non valida",
        ValidationCode::RequiredField => "valore obbligatorio mancante",
        ValidationCode::ConditionalRequired => "valore richiesto da una condizione mancante",
        ValidationCode::ForbiddenField => "valore presente ma vietato",
        ValidationCode::Alignment => "allineamento non valido",
        ValidationCode::InvalidType => "tipo di dato non valido",
        ValidationCode::InvalidDomain => "valore fuori dominio",
        ValidationCode::OutOfRange => "valore fuori intervallo",
        ValidationCode::InvalidDate => "data non valida",
        ValidationCode::CrossField => "vincolo tra campi non rispettato",
        ValidationCode::ReferenceData => "valore non valido rispetto ai dati di riferimento",
        ValidationCode::CrossRecord => "vincolo tra record non rispettato",
        ValidationCode::RuleConflict => "regole in conflitto",
        ValidationCode::RelatedRecordMissing => "record correlato mancante",
        ValidationCode::DuplicateRecordKey => "chiave record duplicata",
        ValidationCode::RegulatoryRule(_) => "regola normativa non rispettata",
    };
    let mut message = format!("Campo {} ({}): {description}.", field.id().0, field.name());
    if !violation.reason.is_empty() {
        message.push(' ');
        message.push_str(&violation.reason);
    }
    message
}
