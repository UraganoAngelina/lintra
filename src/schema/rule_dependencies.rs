//! Exhaustive dependency closure shared by cross classification and projection.
use super::{compiled::*, linter::SchemaIssueCode as Code};
use std::collections::BTreeSet;
type Issue = (Code, String);

#[derive(Default)]
pub(crate) struct RuleDependencies {
    pub fields: BTreeSet<(usize, usize)>,
    pub joins: BTreeSet<usize>,
}
impl RuleDependencies {
    fn field(&mut self, reference: &ResolvedFieldRef, scope: usize) {
        match reference {
            ResolvedFieldRef::Local { field_index, .. } => {
                self.fields.insert((scope, *field_index));
            }
            ResolvedFieldRef::External {
                trace_index,
                field_index,
                join_index,
                ..
            } => {
                self.fields.insert((*trace_index, *field_index));
                self.joins.insert(*join_index);
            }
        }
    }
    fn derived(
        &mut self,
        index: usize,
        scope: usize,
        tables: &[Vec<DerivedValuePlan>],
    ) -> Result<(), Issue> {
        let plan = tables
            .get(scope)
            .and_then(|plans| plans.get(index))
            .ok_or((
                Code::InternalCompilerInvariant,
                "derived projection dependency is inconsistent".into(),
            ))?;
        match plan {
            DerivedValuePlan::Substring { source, .. } => self.field(source, scope),
            DerivedValuePlan::AgeInCompletedYears {
                birth, reference, ..
            }
            | DerivedValuePlan::CalendarYearDelta {
                birth, reference, ..
            } => {
                self.field(birth, scope);
                self.field(reference, scope);
            }
        }
        Ok(())
    }
    fn expression(
        &mut self,
        root: &CompiledExpr,
        scope: usize,
        tables: &[Vec<DerivedValuePlan>],
    ) -> Result<(), Issue> {
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            match node {
                CompiledExpr::Const(_) => {}
                CompiledExpr::Eq(field, _)
                | CompiledExpr::Ne(field, _)
                | CompiledExpr::In(field, _)
                | CompiledExpr::NotIn(field, _)
                | CompiledExpr::IsPresent(field)
                | CompiledExpr::IsMissing(field) => self.field(field, scope),
                CompiledExpr::And(children) | CompiledExpr::Or(children) => {
                    pending.extend(children)
                }
                CompiledExpr::Not(child) => pending.push(child),
                CompiledExpr::DerivedEq {
                    field,
                    derived_index,
                } => {
                    self.field(field, scope);
                    self.derived(*derived_index, scope, tables)?;
                }
                CompiledExpr::DerivedCompare { derived_index, .. } => {
                    self.derived(*derived_index, scope, tables)?
                }
            }
        }
        Ok(())
    }
    fn operand(&mut self, value: &CompiledOperand, scope: usize) {
        if let CompiledOperand::Field(field) = value {
            self.field(field, scope);
        }
    }
    fn constraint(
        &mut self,
        constraint: &CompiledConstraint,
        scope: usize,
        tables: &[Vec<DerivedValuePlan>],
    ) -> Result<(), Issue> {
        match constraint {
            CompiledConstraint::ReferenceData {
                metadata_bindings, ..
            } => {
                for binding in metadata_bindings {
                    match &binding.operand {
                        MetadataOperand::Field(field) => self.field(field, scope),
                        MetadataOperand::Derived(index) => self.derived(*index, scope, tables)?,
                    }
                }
            }
            CompiledConstraint::PositionalBitset { positions } => {
                for position in positions {
                    self.expression(&position.when, scope, tables)?;
                }
            }
            CompiledConstraint::AllowedValues { .. }
            | CompiledConstraint::Range { .. }
            | CompiledConstraint::Regex { .. }
            | CompiledConstraint::LogicalLength { .. }
            | CompiledConstraint::SubstringNotEquals { .. }
            | CompiledConstraint::NoLowercase
            | CompiledConstraint::Date { .. }
            | CompiledConstraint::CompositeSegments { .. }
            | CompiledConstraint::NonEmpty => {}
        }
        Ok(())
    }
    pub(crate) fn collect(
        rule: &CompiledRule,
        source: usize,
        joins: &[CompiledJoinSpec],
        tables: &[Vec<DerivedValuePlan>],
    ) -> Result<Self, Issue> {
        let mut dependencies = Self::default();
        dependencies.expression(&rule.when, source, tables)?;
        for assertion in &rule.assertions {
            match assertion {
                CompiledAssertion::FieldPresent(field) => dependencies.field(field, source),
                CompiledAssertion::Constraint { field, constraint } => {
                    dependencies.field(field, source);
                    dependencies.constraint(constraint, source, tables)?;
                }
                CompiledAssertion::FieldCompare { left, right, .. } => {
                    dependencies.operand(left, source);
                    dependencies.operand(right, source);
                }
                CompiledAssertion::EqualsConcat { field, parts } => {
                    dependencies.field(field, source);
                    for part in parts {
                        dependencies.field(part, source);
                    }
                }
                CompiledAssertion::ConditionTrue(condition) => {
                    dependencies.expression(condition, source, tables)?
                }
                CompiledAssertion::DerivedCompare { derived_index, .. } => {
                    dependencies.derived(*derived_index, source, tables)?
                }
                CompiledAssertion::RecordExists { join_index }
                | CompiledAssertion::SetExists { join_index, .. } => {
                    dependencies.joins.insert(*join_index);
                    let join = joins.get(*join_index).ok_or((
                        Code::InternalCompilerInvariant,
                        "assertion join dependency is inconsistent".into(),
                    ))?;
                    if let CompiledAssertion::SetExists { predicate, .. } = assertion {
                        dependencies.expression(predicate, join.target_trace, tables)?;
                    }
                }
                CompiledAssertion::SubstringEqualsDerivedYear {
                    field,
                    source: date,
                    ..
                } => {
                    dependencies.field(field, source);
                    dependencies.field(date, source);
                }
            }
        }
        Ok(dependencies)
    }
}
