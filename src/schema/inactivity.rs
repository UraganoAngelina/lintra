//! Resolve the existing closed inactivity DSL. Only a compiled plan escapes;
//! persistence publication is still a separate producer responsibility.
use super::*;

pub(super) fn compile(
    raw: &RawFlowSchema,
    joins: &[CompiledJoinSpec],
    plan: &RawTemporal,
) -> Result<TemporalPlan, Issue> {
    let RawTemporal::InactivityWithPauses {
        owner_trace,
        evaluations_join,
        suspensions_join,
        accesses_join,
        provisions_join,
        access_fields,
        provision_fields,
        start_field,
        access_date_field,
        suspension_start_field,
        suspension_end_field,
        conclusion_date_field,
        conclusion_code_field,
        required_conclusion_code,
        threshold,
        on_missing_trace,
        error_code,
        ..
    } = plan
    else {
        return Err(invalid("inactivity plan expected"));
    };
    let owner = raw
        .traces
        .iter()
        .position(|t| t.id == *owner_trace)
        .ok_or_else(|| invalid("inactivity owner is not declared"))?;
    let source = &raw.traces[owner];
    let bindings = ReferenceBindings::cross(raw, owner, joins)?;
    let policy = joins::policy(on_missing_trace)?;
    if !matches!(policy, RelatedPolicy::Unknown { .. }) {
        return Err(invalid(
            "inactivity missing input requires explicit Unknown",
        ));
    }
    let resolve = |name: &str| -> Result<(usize, &CompiledJoinSpec), Issue> {
        let (index, join) = bindings.join(name)?;
        let declaration = source
            .joins
            .iter()
            .find(|j| j.id == name)
            .ok_or_else(|| invalid("inactivity join is not declared"))?;
        if declaration.on_missing_trace != *on_missing_trace {
            return Err(invalid(
                "inactivity and join missing-trace policies must agree",
            ));
        }
        Ok((index, join))
    };
    let (evaluations_join, evaluation) = resolve(evaluations_join)?;
    let (suspensions_join, suspension) = resolve(suspensions_join)?;
    let (accesses_join, access) = resolve(accesses_join)?;
    let (provisions_join, provision) = resolve(provisions_join)?;
    if [
        evaluations_join,
        suspensions_join,
        accesses_join,
        provisions_join,
    ]
    .into_iter()
    .collect::<HashSet<_>>()
    .len()
        != 4
    {
        return Err(invalid("inactivity requires four distinct joins"));
    }
    for join in [suspension, access, provision] {
        if join.cardinality != JoinCardinality::ZeroOrMany || join.consensus.is_some() {
            return Err(invalid(
                "inactivity activity/pause joins require unreduced zero_or_many sets",
            ));
        }
    }
    let eval = &raw.traces[evaluation.target_trace];
    if evaluation.cardinality != JoinCardinality::ZeroOrMany {
        return Err(invalid(
            "inactivity evaluations require a zero_or_many set with consensus",
        ));
    }
    let start_field = date(eval, start_field)?;
    let conclusion_date_field = date(eval, conclusion_date_field)?;
    let conclusion_code_field = index(eval, conclusion_code_field)?;
    let consensus = evaluation
        .consensus
        .as_ref()
        .ok_or_else(|| invalid("inactivity evaluations require explicit tuple consensus"))?;
    // Open evaluations participate too: a conclusion-only filter could hide a
    // conflicting start date. Closed AST const-true is deliberately required.
    if !matches!(consensus.include_if, CompiledExpr::Const(true))
        || ![start_field, conclusion_date_field, conclusion_code_field]
            .iter()
            .all(|f| consensus.fields.contains(f))
    {
        return Err(invalid(
            "inactivity consensus must include every evaluation and its start/date/code tuple",
        ));
    }
    if access_fields.is_empty()
        || access_fields.len() != provision_fields.len()
        || access_fields.iter().collect::<HashSet<_>>().len() != access_fields.len()
        || provision_fields.iter().collect::<HashSet<_>>().len() != provision_fields.len()
    {
        return Err(invalid(
            "inactivity correlation requires equally sized nonempty unique field lists",
        ));
    }
    let a = &raw.traces[access.target_trace];
    let p = &raw.traces[provision.target_trace];
    let access_fields = access_fields
        .iter()
        .map(|id| index(a, id))
        .collect::<Result<Vec<_>, _>>()?;
    let provision_fields = provision_fields
        .iter()
        .map(|id| index(p, id))
        .collect::<Result<Vec<_>, _>>()?;
    for (&left, &right) in access_fields.iter().zip(&provision_fields) {
        if std::mem::discriminant(&a.fields[left].physical)
            != std::mem::discriminant(&p.fields[right].physical)
        {
            return Err(invalid("inactivity correlation field types must agree"));
        }
    }
    if threshold.calendar_months == 0
        || threshold.calendar_months > 119_987
        || required_conclusion_code.is_empty()
    {
        return Err(invalid(
            "inactivity requires a positive bounded month count and a conclusion code",
        ));
    }
    let period = &raw
        .persistence_policy
        .as_ref()
        .and_then(|p| p.snapshot.as_ref())
        .ok_or_else(|| invalid("session quarter horizon requires snapshot.period"))?
        .period;
    let parameter = |id: &str| {
        raw.parameters
            .iter()
            .position(|p| matches!(p, RawParameter::Integer { id: name, .. } if name == id))
            .ok_or_else(|| invalid("inactivity period requires declared integer parameters"))
    };
    let year_parameter = parameter(&period.year_parameter)?;
    let quarter_parameter = parameter(&period.quarter_parameter)?;
    if year_parameter == quarter_parameter {
        return Err(invalid("inactivity period parameters must be distinct"));
    }
    let s = &raw.traces[suspension.target_trace];
    Ok(TemporalPlan::InactivityWithPauses(Box::new(
        InactivityWithPausesPlan {
            owner_trace: owner,
            year_parameter,
            quarter_parameter,
            evaluations_join,
            suspensions_join,
            accesses_join,
            provisions_join,
            access_fields,
            provision_fields,
            start_field,
            access_date_field: date(a, access_date_field)?,
            suspension_start_field: date(s, suspension_start_field)?,
            suspension_end_field: date(s, suspension_end_field)?,
            conclusion_date_field,
            conclusion_code_field,
            required_conclusion_code: required_conclusion_code.as_bytes().to_vec(),
            calendar_months: threshold.calendar_months,
            on_missing_trace: policy,
            error_code: error_code.parse().map_err(|_| {
                (
                    Code::InvalidDiagnosticCode,
                    "invalid inactivity error code".into(),
                )
            })?,
        },
    )))
}
