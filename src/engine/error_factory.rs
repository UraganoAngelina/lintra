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
        let mut message = field_message(field, &violation);
        let guide = super::field_guide::field_format_guide(field, self.encoding);
        message.push(' ');
        message.push_str(&guide);
        let mut expected = violation.expected.clone().unwrap_or_default();
        // Include small literal domains in every consumer (CLI and exports too).
        // A bounded preview avoids copying a large enum into every failed row;
        // the UI enumerates the complete declaration separately from this text.
        for constraint in field.constraints() {
            if let Some(mut domain) = super::field_guide::declared_domain(constraint, self.encoding)
            {
                let values: Vec<_> = domain.by_ref().take(16).collect();
                if !expected.is_empty() {
                    expected.push_str(". ");
                }
                expected.push_str(super::field_guide::constraint_comparison_guide(constraint));
                expected.push_str(" Dominio dichiarato: ");
                expected.push_str(&values.join(", "));
                if domain.next().is_some() {
                    expected.push_str(
                        " … (anteprima; elenco completo consultabile nel dettaglio del campo)",
                    );
                }
            }
        }
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
            expected: (!expected.is_empty()).then_some(expected),
            context: violation.diagnostic_context,
            message,
        }
    }

    /// Duplicate is anchored to the later physical occurrence. The original
    /// record remains available to joins; no first-record overwrite/dedup.
    pub fn duplicate_record_key_error(
        &self,
        line_number: u64,
        identity: &RecordIdentity,
        code: &ValidationCode,
        first_line: u64,
    ) -> ValidationError {
        ValidationError {
            code: code.clone(),
            flow_id: self.flow_id.clone(),
            schema_version: self.schema_version.clone(),
            trace_id: self.trace_id.clone(),
            line_number,
            record_identity: identity.clone(),
            field: None,
            actual: None,
            expected: Some("chiave univoca nel tracciato dell’invio corrente".into()),
            context: Vec::new(),
            message: format!("Chiave record duplicata; prima occorrenza alla riga {first_line}."),
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
