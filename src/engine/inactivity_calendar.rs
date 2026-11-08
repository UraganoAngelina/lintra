//! Gregorian calendar arithmetic independent of fields, trace names and storage.
use crate::{DateLiteral, UnknownReason};
use chrono::{Days, Months, NaiveDate};

pub(super) fn date(value: DateLiteral) -> Result<NaiveDate, UnknownReason> {
    if value.year == 0 || value.year > 9999 {
        return Err(UnknownReason::InvalidFieldValue);
    }
    NaiveDate::from_ymd_opt(
        i32::from(value.year),
        u32::from(value.month),
        u32::from(value.day),
    )
    .ok_or(UnknownReason::InvalidFieldValue)
}

/// Month addition clamps invalid days (31 Jan + 1 month = 28/29 Feb).
/// Convert each open interval to its inclusive interior *before* taking the
/// union: adjacent original endpoints must not invent an extra paused day.
pub(super) fn due(
    base: DateLiteral,
    months: u32,
    pauses: &[(DateLiteral, Option<DateLiteral>)],
    horizon: DateLiteral,
) -> Result<bool, UnknownReason> {
    let base = date(base)?;
    let horizon = date(horizon)?;
    if months == 0 {
        return Err(UnknownReason::UnsupportedPlan);
    }
    let mut deadline = base
        .checked_add_months(Months::new(months))
        .ok_or(UnknownReason::InvalidFieldValue)?;
    let mut interiors = Vec::with_capacity(pauses.len());
    for &(start, end) in pauses {
        let start = date(start)?;
        let end = end.map(date).transpose()?;
        if end.is_some_and(|end| end < start) {
            return Err(UnknownReason::InvalidFieldValue);
        }
        let lo = start
            .max(base)
            .checked_add_days(Days::new(1))
            .ok_or(UnknownReason::InvalidFieldValue)?;
        // A missing endpoint has no excluded terminal day: it covers the
        // horizon too. A declared endpoint is always excluded.
        let hi = match end {
            Some(end) => end
                .pred_opt()
                .ok_or(UnknownReason::InvalidFieldValue)?
                .min(horizon),
            None => horizon,
        };
        if lo <= hi {
            interiors.push((lo, hi));
        }
    }
    interiors.sort_unstable();
    let mut union: Vec<(NaiveDate, NaiveDate)> = Vec::with_capacity(interiors.len());
    for (lo, hi) in interiors {
        if let Some(previous) = union.last_mut()
            && lo.signed_duration_since(previous.1).num_days() <= 1
        {
            previous.1 = previous.1.max(hi);
        } else {
            union.push((lo, hi));
        }
    }
    // Each disjoint interval is consumed once. If its first paused day is
    // reached, skipping its whole length moves the deadline beyond its end;
    // newly reached later intervals are handled next. This is exactly the
    // fixed point of deadline = base+months+paused_days(base, deadline], without
    // iterating once per calendar day or double-counting overlapping pauses.
    for (lo, hi) in union {
        if deadline > horizon || lo > deadline {
            break;
        }
        let days = u64::try_from(hi.signed_duration_since(lo).num_days() + 1)
            .map_err(|_| UnknownReason::InvalidFieldValue)?;
        deadline = deadline
            .checked_add_days(Days::new(days))
            .ok_or(UnknownReason::InvalidFieldValue)?;
    }
    Ok(deadline <= horizon)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn d(s: &str) -> DateLiteral {
        DateLiteral::from_ddmmyyyy(s.as_bytes()).unwrap()
    }
    #[test]
    fn calendar_months_clamp_in_leap_and_non_leap_february() {
        for (base, months, before, at) in [
            ("30112023", 3, "28022024", "29022024"),
            ("30112022", 3, "27022023", "28022023"),
            ("31082023", 6, "28022024", "29022024"),
            ("31012026", 3, "29042026", "30042026"),
        ] {
            assert_eq!(due(d(base), months, &[], d(before)), Ok(false));
            assert_eq!(due(d(base), months, &[], d(at)), Ok(true));
        }
    }
    #[test]
    fn overlapping_nested_duplicate_pauses_count_the_union_once() {
        let p = [
            (d("02012026"), Some(d("10012026"))),
            (d("05012026"), Some(d("15012026"))),
            (d("02012026"), Some(d("10012026"))),
        ];
        // Union 3..14 January = 12 days; 1 April + 12 = 13 April.
        assert_eq!(due(d("01012026"), 3, &p, d("12042026")), Ok(false));
        assert_eq!(due(d("01012026"), 3, &p, d("13042026")), Ok(true));
    }
    #[test]
    fn touching_original_boundaries_do_not_create_a_paused_boundary_day() {
        let p = [
            (d("02012026"), Some(d("05012026"))),
            (d("05012026"), Some(d("08012026"))),
        ];
        // 3,4,6,7 only: the shared endpoint 5 is not suspended.
        assert_eq!(due(d("01012026"), 3, &p, d("04042026")), Ok(false));
        assert_eq!(due(d("01012026"), 3, &p, d("05042026")), Ok(true));
    }
    #[test]
    fn extension_reaches_later_pauses_until_fixed_point() {
        let p = [
            (d("30032026"), Some(d("05042026"))),
            (d("05042026"), Some(d("10042026"))),
        ];
        assert_eq!(due(d("01012026"), 3, &p, d("09042026")), Ok(false));
        assert_eq!(due(d("01012026"), 3, &p, d("10042026")), Ok(true));
    }
    #[test]
    fn open_pause_covers_horizon_and_irrelevant_pauses_do_not_reset_activity() {
        assert_eq!(
            due(d("01012026"), 3, &[(d("30032026"), None)], d("30042026")),
            Ok(false)
        );
        let p = [(d("01012025"), Some(d("31012025"))), (d("02042026"), None)];
        assert_eq!(due(d("01012026"), 3, &p, d("01042026")), Ok(true));
        assert_eq!(
            due(
                d("01012026"),
                3,
                &[(d("10012026"), Some(d("09012026")))],
                d("30062026")
            ),
            Err(UnknownReason::InvalidFieldValue)
        );
    }
    #[test]
    fn union_algorithm_matches_independent_day_by_day_fixed_point() {
        let base = d("01012026");
        let horizon = d("30062026");
        let start = date(base).unwrap();
        let end = date(horizon).unwrap();
        for seed in 0..100_u64 {
            let p: Vec<_> = (0..8_u64)
                .map(|i| {
                    let lo = start
                        .checked_add_days(Days::new((seed * 17 + i * 23) % 150))
                        .unwrap();
                    let hi = lo.checked_add_days(Days::new((seed + i * 7) % 18)).unwrap();
                    use chrono::Datelike;
                    let literal = |d: NaiveDate| DateLiteral {
                        year: d.year() as u16,
                        month: d.month() as u8,
                        day: d.day() as u8,
                    };
                    (literal(lo), Some(literal(hi)))
                })
                .collect();
            let nominal = start.checked_add_months(Months::new(3)).unwrap();
            let mut threshold = nominal;
            loop {
                let mut paused = 0;
                let mut day = start.succ_opt().unwrap();
                while day <= threshold.min(end) {
                    if p.iter().any(|&(lo, hi)| {
                        date(lo).unwrap() < day && hi.is_none_or(|hi| day < date(hi).unwrap())
                    }) {
                        paused += 1;
                    }
                    day = day.succ_opt().unwrap();
                }
                let next = nominal.checked_add_days(Days::new(paused)).unwrap();
                if next == threshold || next > end {
                    threshold = next;
                    break;
                }
                threshold = next;
            }
            assert_eq!(
                due(base, 3, &p, horizon),
                Ok(threshold <= end),
                "seed {seed}"
            );
        }
    }
}
