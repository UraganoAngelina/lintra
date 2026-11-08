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
    /// Dependency indexes for a proven-local target predicate. The synthetic
    /// scope is only compiler bookkeeping; no runtime trace is fabricated.
    pub(crate) fn local_fields(
        expression: &CompiledExpr,
        derived: &[DerivedValuePlan],
    ) -> Result<BTreeSet<usize>, Issue> {
        let mut deps = Self::default();
        deps.expression(expression, 0, &[derived.to_vec()])?;
        if !deps.joins.is_empty() || deps.fields.iter().any(|(scope, _)| *scope != 0) {
            return Err((
                Code::UnsupportedConstruct,
                "predicate must remain local".into(),
            ));
        }
        Ok(deps.fields.into_iter().map(|(_, index)| index).collect())
    }
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
                CompiledAssertion::InactivityWithPauses(plan) => {
                    for (index, fields) in [
                        (
                            plan.evaluations_join,
                            vec![
                                plan.start_field,
                                plan.conclusion_date_field,
                                plan.conclusion_code_field,
                            ],
                        ),
                        (
                            plan.suspensions_join,
                            vec![plan.suspension_start_field, plan.suspension_end_field],
                        ),
                        (plan.accesses_join, {
                            let mut fields = plan.access_fields.clone();
                            fields.push(plan.access_date_field);
                            fields
                        }),
                        (plan.provisions_join, plan.provision_fields.clone()),
                    ] {
                        let join = joins.get(index).ok_or((
                            Code::InternalCompilerInvariant,
                            "inactivity dependency join is inconsistent".into(),
                        ))?;
                        dependencies.joins.insert(index);
                        dependencies
                            .fields
                            .extend(fields.into_iter().map(|field| (join.target_trace, field)));
                    }
                }
                CompiledAssertion::AfterDateRestriction(plan) => {
                    dependencies.joins.insert(plan.conclusion_join);
                    let join = joins.get(plan.conclusion_join).ok_or((
                        Code::InternalCompilerInvariant,
                        "temporal join is inconsistent".into(),
                    ))?;
                    dependencies.fields.insert((source, plan.date_field));
                    dependencies.fields.insert((source, plan.restricted_field));
                    dependencies
                        .fields
                        .insert((join.target_trace, plan.conclusion_date_field));
                    dependencies.expression(&plan.target_when, join.target_trace, tables)?;
                }
                CompiledAssertion::IntervalExclusion(plan) => {
                    dependencies.joins.insert(plan.interval_join);
                    let join = joins.get(plan.interval_join).ok_or((
                        Code::InternalCompilerInvariant,
                        "temporal join is inconsistent".into(),
                    ))?;
                    dependencies.fields.insert((source, plan.point_field));
                    dependencies
                        .fields
                        .insert((join.target_trace, plan.start_field));
                    dependencies
                        .fields
                        .insert((join.target_trace, plan.end_field));
                }
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
