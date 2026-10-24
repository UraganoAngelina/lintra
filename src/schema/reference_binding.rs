//! Compile-time reference binding, with explicit local or flow context.
use super::{compiled::*, linter::SchemaIssueCode as Code, raw::*};
use crate::{FieldId, JoinId};
type Issue = (Code, String);

pub(crate) struct ReferenceBindings<'a> {
    pub record: &'a RawRecordSchema,
    flow: Option<(&'a RawFlowSchema, &'a [CompiledJoinSpec], usize)>,
}
impl<'a> ReferenceBindings<'a> {
    pub(crate) fn require_sets(&self) -> Result<(), Issue> {
        let Some((flow, _, _)) = self.flow else {
            return Err((
                Code::UnsupportedConstruct,
                "set assertions require cross-record bindings".into(),
            ));
        };
        if !flow
            .required_capabilities
            .contains(&RawFlowSchemaRequiredCapabilitiesItem::JoinsSetsV1)
        {
            return Err((
                Code::MissingCapability,
                "set_exists requires joins.sets.v1".into(),
            ));
        }
        Ok(())
    }
    pub(crate) fn local(record: &'a RawRecordSchema) -> Self {
        Self { record, flow: None }
    }
    pub(crate) fn cross(
        flow: &'a RawFlowSchema,
        source: usize,
        joins: &'a [CompiledJoinSpec],
    ) -> Result<Self, Issue> {
        let record = flow.traces.get(source).ok_or((
            Code::InternalCompilerInvariant,
            "source trace binding is inconsistent".into(),
        ))?;
        Ok(Self {
            record,
            flow: Some((flow, joins, source)),
        })
    }
    pub(crate) fn join(&self, id: &str) -> Result<(usize, &'a CompiledJoinSpec), Issue> {
        let (_, joins, source) = self.flow.ok_or((
            Code::UnsupportedConstruct,
            "join requires cross-record bindings".into(),
        ))?;
        let (index, join) = joins
            .iter()
            .enumerate()
            .find(|(_, join)| join.id.0 == id)
            .ok_or((
                Code::InvalidSchemaValue,
                "referenced join is not declared".into(),
            ))?;
        if join.source_trace != source {
            return Err((
                Code::InvalidSchemaValue,
                "referenced join belongs to another source trace".into(),
            ));
        }
        Ok((index, join))
    }
    pub(crate) fn target(&self, trace: usize) -> Result<Self, Issue> {
        let (flow, joins, _) = self.flow.ok_or((
            Code::UnsupportedConstruct,
            "target predicate requires cross-record bindings".into(),
        ))?;
        Self::cross(flow, trace, joins)
    }
    pub(crate) fn resolve(&self, reference: &RawRef) -> Result<ResolvedFieldRef, Issue> {
        match reference {
            RawRef::Local(reference) => {
                let field_index = self
                    .record
                    .fields
                    .iter()
                    .position(|field| field.id == reference.field)
                    .ok_or((
                        Code::MissingFieldReference,
                        "local expression field does not exist".into(),
                    ))?;
                Ok(ResolvedFieldRef::Local {
                    field_index,
                    field_id: FieldId(reference.field.clone()),
                })
            }
            RawRef::External(reference) => {
                let (join_index, join) = self.join(&reference.join)?;
                let (flow, _, _) = self.flow.ok_or((
                    Code::InternalCompilerInvariant,
                    "external binding has no flow".into(),
                ))?;
                let trace = flow.traces.get(join.target_trace).ok_or((
                    Code::MissingTraceReference,
                    "external target trace does not exist".into(),
                ))?;
                if trace.id != reference.trace {
                    return Err((
                        Code::InvalidSchemaValue,
                        "external reference target does not match its join".into(),
                    ));
                }
                let field_index = trace
                    .fields
                    .iter()
                    .position(|field| field.id == reference.field)
                    .ok_or((
                        Code::MissingFieldReference,
                        "external expression field does not exist".into(),
                    ))?;
                if let Some(consensus) = &join.consensus {
                    if !consensus.fields.contains(&field_index) {
                        return Err((
                            Code::UnsupportedConstruct,
                            "scalar reference is outside the consensus tuple".into(),
                        ));
                    }
                } else if matches!(
                    join.cardinality,
                    JoinCardinality::OneOrMany | JoinCardinality::ZeroOrMany
                ) {
                    return Err((
                        Code::UnsupportedConstruct,
                        "scalar reference on a set requires consensus".into(),
                    ));
                }
                Ok(ResolvedFieldRef::External {
                    trace_index: join.target_trace,
                    field_index,
                    field_id: FieldId(reference.field.clone()),
                    join_index,
                    join_id: JoinId(reference.join.clone()),
                })
            }
        }
    }
    pub(crate) fn field(&self, reference: &ResolvedFieldRef) -> Result<&'a RawFieldSpec, Issue> {
        let (record, index, id) = match reference {
            ResolvedFieldRef::Local {
                field_index,
                field_id,
            } => (self.record, *field_index, field_id),
            ResolvedFieldRef::External {
                trace_index,
                field_index,
                field_id,
                ..
            } => {
                let (flow, _, _) = self.flow.ok_or((
                    Code::UnsupportedConstruct,
                    "external field requires flow context".into(),
                ))?;
                (
                    flow.traces.get(*trace_index).ok_or((
                        Code::MissingTraceReference,
                        "external trace does not exist".into(),
                    ))?,
                    *field_index,
                    field_id,
                )
            }
        };
        record
            .fields
            .get(index)
            .filter(|field| field.id == id.0)
            .ok_or((
                Code::InternalCompilerInvariant,
                "resolved field metadata is inconsistent".into(),
            ))
    }
}
