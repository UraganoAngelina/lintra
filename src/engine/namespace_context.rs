//! A checked immutable parameter set for one compiled flow. Namespace values
//! are supplied once; the engine never looks at application/storage configuration.
use crate::{
    CompiledFlowSchema, CrossRecordInputs, JoinParameterValue, ParameterDefinition,
    RelatedContextError, ValidationMode,
};
use std::collections::BTreeMap;

pub struct VerifiedRunContext<'schema> {
    pub(crate) schema: &'schema CompiledFlowSchema,
    parameters: BTreeMap<String, JoinParameterValue>,
}
impl<'schema> VerifiedRunContext<'schema> {
    pub fn new(
        schema: &'schema CompiledFlowSchema,
        parameters: BTreeMap<String, JoinParameterValue>,
        mode: ValidationMode,
    ) -> Result<Self, RelatedContextError> {
        super::projected_context::validate_cross_inputs(
            schema,
            &CrossRecordInputs {
                available_traces: &[],
                parameters: &parameters,
            },
        )?;
        if !schema.namespace_bindings().is_empty() {
            let policy = schema
                .persistence_policy()
                .as_ref()
                .ok_or(RelatedContextError::InvalidParameter)?;
            for &index in &policy.namespace_parameters {
                let definition = schema
                    .parameters()
                    .get(index)
                    .ok_or(RelatedContextError::InvalidParameter)?;
                let id = match definition {
                    ParameterDefinition::String { id, .. }
                    | ParameterDefinition::Integer { id, .. } => id,
                };
                let Some(value) = parameters.get(id.as_ref()) else {
                    if mode == ValidationMode::Full {
                        return Err(RelatedContextError::MissingNamespaceParameter);
                    }
                    continue;
                };
                // Use the same domain constructor as historical/session binding.
                // It validates labels without trimming or changing their bytes.
                let value = match value {
                    JoinParameterValue::Text(v) => crate::NamespaceValue::String(v.clone()),
                    JoinParameterValue::Integer(v) => crate::NamespaceValue::Integer(
                        v.to_string()
                            .parse()
                            .map_err(|_| RelatedContextError::InvalidParameter)?,
                    ),
                };
                crate::PersistenceNamespace::new(vec![(id.to_string(), value)])
                    .map_err(|_| RelatedContextError::InvalidParameter)?;
            }
        }
        Ok(Self { schema, parameters })
    }
    pub fn parameters(&self) -> &BTreeMap<String, JoinParameterValue> {
        &self.parameters
    }
    pub(crate) fn text(&self, index: usize) -> Option<&str> {
        let ParameterDefinition::String { id, .. } = &self.schema.parameters()[index] else {
            return None;
        };
        match self.parameters.get(id.as_ref()) {
            Some(JoinParameterValue::Text(v)) => Some(v),
            _ => None,
        }
    }
}

/// Resolved once per trace, bounded by the number of schema bindings, not records.
#[derive(Clone)]
pub(crate) struct BindingExecution {
    pub plan: crate::NamespaceBindingPlan,
    pub text: Option<String>,
}
