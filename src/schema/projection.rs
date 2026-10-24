//! Compiler-owned dependency closure for the currently supported cross plans.
use super::*;

type Issue = (Code, String);
fn invariant() -> Issue {
    (
        Code::InternalCompilerInvariant,
        "projection dependency is inconsistent".into(),
    )
}
fn select(fields: &mut [bool], index: usize) -> Result<(), Issue> {
    *fields.get_mut(index).ok_or_else(invariant)? = true;
    Ok(())
}
fn field(fields: &mut [bool], reference: &ResolvedFieldRef) -> Result<(), Issue> {
    match reference {
        ResolvedFieldRef::Local { field_index, .. } => select(fields, *field_index),
        ResolvedFieldRef::External { .. } => Err((
            Code::UnsupportedConstruct,
            "external projection dependencies require cross-rule lowering".into(),
        )),
    }
}
fn derived(fields: &mut [bool], index: usize, plans: &[DerivedValuePlan]) -> Result<(), Issue> {
    match plans.get(index).ok_or_else(invariant)? {
        DerivedValuePlan::Substring { source, .. } => field(fields, source),
        DerivedValuePlan::AgeInCompletedYears {
            birth, reference, ..
        }
        | DerivedValuePlan::CalendarYearDelta {
            birth, reference, ..
        } => {
            field(fields, birth)?;
            field(fields, reference)
        }
    }
}
/// Exhaustive, iterative visitation: new expression variants require a Rust
/// match update instead of silently omitting their projected dependencies.
fn expression(
    fields: &mut [bool],
    root: &CompiledExpr,
    plans: &[DerivedValuePlan],
) -> Result<(), Issue> {
    let mut pending = vec![root];
    while let Some(expr) = pending.pop() {
        match expr {
            CompiledExpr::Const(_) => {}
            CompiledExpr::Eq(reference, _)
            | CompiledExpr::Ne(reference, _)
            | CompiledExpr::In(reference, _)
            | CompiledExpr::NotIn(reference, _)
            | CompiledExpr::IsPresent(reference)
            | CompiledExpr::IsMissing(reference) => field(fields, reference)?,
            CompiledExpr::And(children) | CompiledExpr::Or(children) => pending.extend(children),
            CompiledExpr::Not(child) => pending.push(child),
            CompiledExpr::DerivedEq {
                field: reference,
                derived_index,
            } => {
                field(fields, reference)?;
                derived(fields, *derived_index, plans)?;
            }
            CompiledExpr::DerivedCompare { derived_index, .. } => {
                derived(fields, *derived_index, plans)?
            }
        }
    }
    Ok(())
}

pub(super) fn compile(
    record: &RawRecordSchema,
    trace_index: usize,
    joins: &[CompiledJoinSpec],
    derived_plans: &[DerivedValuePlan],
    record_key: &[usize],
    additional_fields: &[usize],
) -> Result<ContextProjectionPlan, Issue> {
    let mut fields = vec![false; record.fields.len()];
    for &index in record_key {
        select(&mut fields, index)?;
    }
    for &index in additional_fields {
        select(&mut fields, index)?;
    }
    for binding in &record.identity {
        select(&mut fields, index(record, &binding.field)?)?;
    }
    let mut join_indexes = Vec::new();
    let mut join_plans = Vec::new();
    for (join_index, join) in joins.iter().enumerate() {
        let source = join.source_trace == trace_index;
        let target = join.target_trace == trace_index;
        if source || target {
            join_indexes.push(join_index);
        }
        for (applies, side, indexes) in [
            (source, JoinProjectionSide::Source, &join.source_fields),
            (target, JoinProjectionSide::Target, &join.target_fields),
        ] {
            if applies {
                for &index in indexes {
                    select(&mut fields, index)?;
                }
                join_plans.push(JoinProjectionPlan {
                    join_index,
                    key: JoinProjectionKey {
                        join_id: join.id.clone(),
                        side,
                    },
                    field_indexes: indexes.clone(),
                });
            }
        }
        if target && let Some(consensus) = &join.consensus {
            for &index in &consensus.fields {
                select(&mut fields, index)?;
            }
            expression(&mut fields, &consensus.include_if, derived_plans)?;
        }
    }
    Ok(ContextProjectionPlan {
        field_indexes: fields
            .iter()
            .enumerate()
            .filter_map(|(i, selected)| selected.then_some(i))
            .collect(),
        join_indexes,
        join_plans,
    })
}
