//! Source-anchored cross diagnostics from real projected bytes only.
use super::{
    error_factory::ErrorFactory,
    projected_context::ProjectedEvaluationContext,
    rule_diagnostics::{RuleDiagnosticError, assertion_refs, condition_text, expression_refs},
};
use crate::{
    DiagnosticContextItem, FieldDiagnostic, FieldValueResolver, FieldViolation, ProjectedRecord,
    ResolvedFieldValue, ValidationError, render_diagnostic_value,
    schema::{
        compiled::*,
        rule_template::{self, Part, Token},
    },
};
use std::collections::BTreeSet;

pub(crate) struct CrossDiagnosticInput<'a> {
    pub flow: &'a CompiledFlowSchema,
    pub source: &'a ProjectedRecord,
    pub rule: &'a CompiledRule,
    pub assertion_index: Option<usize>,
    pub context: &'a ProjectedEvaluationContext<'a>,
    pub missing_join: Option<usize>,
    pub template: &'a str,
}
impl ErrorFactory<'_> {
    pub(crate) fn cross_rule_error(
        &self,
        input: CrossDiagnosticInput<'_>,
        violation: FieldViolation,
    ) -> Result<ValidationError, RuleDiagnosticError> {
        let schema = input
            .flow
            .trace(&input.source.trace_id)
            .ok_or(RuleDiagnosticError::InvalidPlan)?;
        let assertion = input
            .assertion_index
            .map(|index| {
                input
                    .rule
                    .assertions
                    .get(index)
                    .ok_or(RuleDiagnosticError::InvalidPlan)
            })
            .transpose()?;
        let primary_ref = assertion.and_then(CompiledAssertion::primary_ref);
        let primary = primary_ref
            .map(|reference| {
                match reference {
                    ResolvedFieldRef::Local {
                        field_index,
                        field_id,
                    } => schema
                        .fields()
                        .get(*field_index)
                        .filter(|field| field.id() == field_id),
                    ResolvedFieldRef::External {
                        trace_index,
                        field_index,
                        field_id,
                        ..
                    } => input
                        .flow
                        .traces()
                        .get(*trace_index)
                        .and_then(|trace| trace.fields().get(*field_index))
                        .filter(|field| field.id() == field_id),
                }
                .ok_or(RuleDiagnosticError::InvalidPlan)
            })
            .transpose()?;
        let condition = condition_text(&input.rule.when, schema)?;
        let mut message = String::new();
        for part in rule_template::parse(input.template)
            .map_err(|_| RuleDiagnosticError::InvalidTemplate)?
        {
            message.push_str(match part {
                Part::Text(text) => text,
                Part::Token(Token::RuleId) => &input.rule.id,
                Part::Token(Token::Condition) => &condition,
                Part::Token(Token::FieldId) => {
                    &primary.ok_or(RuleDiagnosticError::InvalidTemplate)?.id().0
                }
                Part::Token(Token::FieldName) => {
                    primary.ok_or(RuleDiagnosticError::InvalidTemplate)?.name()
                }
            });
        }
        let mut references = Vec::new();
        expression_refs(&input.rule.when, schema, &mut references)?;
        if let Some(assertion) = assertion
            && !matches!(
                assertion,
                CompiledAssertion::RecordExists { .. } | CompiledAssertion::SetExists { .. }
            )
        {
            assertion_refs(assertion, schema, &mut references)?;
        }
        let mut seen = BTreeSet::new();
        let mut context = Vec::new();
        for reference in references {
            let (trace, field, join) = match reference {
                ResolvedFieldRef::Local { field_id, .. } => (schema.id(), field_id, None),
                ResolvedFieldRef::External {
                    trace_index,
                    field_id,
                    join_index,
                    ..
                } => (
                    input
                        .flow
                        .traces()
                        .get(*trace_index)
                        .ok_or(RuleDiagnosticError::InvalidPlan)?
                        .id(),
                    field_id,
                    Some(*join_index),
                ),
            };
            if !seen.insert((trace.0.as_str(), field.0.as_str(), join)) {
                continue;
            }
            let bytes = match reference {
                ResolvedFieldRef::Local { .. } => input.source.values.get(field).map(Vec::as_slice),
                ResolvedFieldRef::External { .. } => {
                    match input.context.resolve_raw_field(reference) {
                        ResolvedFieldValue::Present(value) => Some(value),
                        _ => None,
                    }
                }
            };
            if let Some(bytes) = bytes {
                context.push(DiagnosticContextItem {
                    trace_id: trace.clone(),
                    field_id: field.clone(),
                    value: render_diagnostic_value(bytes, self.encoding),
                });
            }
        }
        let join_index = input.missing_join.or(match assertion {
            Some(
                CompiledAssertion::RecordExists { join_index }
                | CompiledAssertion::SetExists { join_index, .. },
            ) => Some(*join_index),
            _ => None,
        });
        if let Some(index) = join_index {
            let join = input
                .flow
                .cross_record_plan()
                .joins
                .get(index)
                .ok_or(RuleDiagnosticError::InvalidPlan)?;
            for &index in &join.source_fields {
                let field = schema
                    .fields()
                    .get(index)
                    .ok_or(RuleDiagnosticError::InvalidPlan)?;
                if seen.insert((schema.id().0.as_str(), field.id().0.as_str(), None)) {
                    let raw = input
                        .source
                        .values
                        .get(field.id())
                        .ok_or(RuleDiagnosticError::InvalidPlan)?;
                    context.push(DiagnosticContextItem {
                        trace_id: schema.id().clone(),
                        field_id: field.id().clone(),
                        value: render_diagnostic_value(raw, self.encoding),
                    });
                }
            }
        }
        // A target field never borrows source coordinates. Such a diagnostic is
        // anchored to the source record, with target values in tagged context.
        let local_primary = primary.filter(|_| {
            input.missing_join.is_none()
                && matches!(primary_ref, Some(ResolvedFieldRef::Local { .. }))
        });
        let actual = local_primary
            .and_then(|field| input.source.values.get(field.id()))
            .map(|bytes| render_diagnostic_value(bytes, self.encoding));
        Ok(ValidationError {
            code: violation.code,
            flow_id: self.flow_id.clone(),
            schema_version: self.schema_version.clone(),
            trace_id: input.source.trace_id.clone(),
            line_number: input.source.line_number,
            record_identity: input.source.identity.clone(),
            field: local_primary.map(|field| {
                FieldDiagnostic::from_range(field.id().clone(), field.name().into(), field.range())
            }),
            actual,
            expected: violation.expected,
            context,
            message,
        })
    }
}
