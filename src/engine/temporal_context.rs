//! Borrowed view of the existing qualified scalar/consensus tuple. Target
//! predicates retain their local field indexes; no representative row is chosen.
use super::projected_context::ProjectedEvaluationContext;
use crate::{
    CompiledJoinSpec, CompiledRecordSchema, DerivedValue, DerivedValueEvaluator, EvaluationContext,
    FieldValueResolver, ResolvedFieldRef, ResolvedFieldValue, UnknownReason,
};

pub(super) struct QualifiedTarget<'a, 'flow, 'data> {
    pub context: &'a ProjectedEvaluationContext<'data>,
    pub schema: &'flow CompiledRecordSchema,
    pub join: &'flow CompiledJoinSpec,
    pub join_index: usize,
}
impl QualifiedTarget<'_, '_, '_> {
    fn external(&self, reference: &ResolvedFieldRef) -> Option<ResolvedFieldRef> {
        let ResolvedFieldRef::Local {
            field_index,
            field_id,
        } = reference
        else {
            return None;
        };
        if !self
            .schema
            .fields()
            .get(*field_index)
            .is_some_and(|f| f.id() == field_id)
        {
            return None;
        }
        Some(ResolvedFieldRef::External {
            trace_index: self.join.target_trace,
            field_index: *field_index,
            field_id: field_id.clone(),
            join_index: self.join_index,
            join_id: self.join.id.clone(),
        })
    }
}
impl FieldValueResolver for QualifiedTarget<'_, '_, '_> {
    fn resolve_field(&self, reference: &ResolvedFieldRef) -> ResolvedFieldValue<'_> {
        match self.external(reference) {
            Some(reference) => self.context.resolve_field(&reference),
            None => ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan),
        }
    }
    fn resolve_raw_field(&self, reference: &ResolvedFieldRef) -> ResolvedFieldValue<'_> {
        match self.external(reference) {
            Some(reference) => self.context.resolve_raw_field(&reference),
            None => ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan),
        }
    }
    fn resolve_derived(&self, index: usize) -> DerivedValue<'_> {
        match self.schema.derived_values().get(index) {
            Some(plan) => DerivedValueEvaluator.evaluate(plan, &EvaluationContext { fields: self }),
            None => DerivedValue::Unknown(UnknownReason::UnsupportedPlan),
        }
    }
}
