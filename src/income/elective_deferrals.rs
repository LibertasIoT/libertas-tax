// Coverage: Form 1040 line 1h excess elective deferrals (2025 Instructions for Form 1040,
//   line 1h; IRC 402(g)(1) and (2); Notice 2025-67 for the 2026 limits). Status: Supported for
//   W-2 box 12 codes D, E, F and S (traditional 401(k), 403(b), SEP and SIMPLE deferrals) and
//   AA and BB (designated Roth 401(k) and 403(b), subject to the same limit). Each person's
//   deferrals over all of their W-2s are compared once with that person's limit: $24,500, or
//   $17,000 when every deferral is SIMPLE, plus the age-50 catch-up ($8,000; $4,000 SIMPLE) or
//   the age 60 to 63 catch-up ($11,250; $5,250 SIMPLE) by age at the end of 2026, because the
//   catch-up "isn't subject to the overall limit on elective deferrals" and the W-2 does not
//   tell it from a regular deferral. The excess (not the Roth share of it, which box 1 already
//   includes) goes to line 1h, joins wages for AGI and is earned income and compensation. A
//   notice says so, and when the deferrals come from several W-2s it says to ask the plans for
//   the excess to be paid back. Excluded: the 403(b) 15-year-rule $3,000 increase and the
//   governmental 457(b) codes G and EE, which have their own limit, and the 501(c)(18)(D) code
//   H (pension_contributions.rs). Tests: src/tests/income/employee_wages.rs.
use crate::federal_model::age_at_year_end;
use crate::kernel::phase;
use crate::year_constants::{
    ELECTIVE_CATCH_UP_2026_CENTS, ELECTIVE_CATCH_UP_AGES_60_63_2026_CENTS,
    ELECTIVE_DEFERRAL_LIMIT_2026_CENTS, SIMPLE_CATCH_UP_2026_CENTS,
    SIMPLE_CATCH_UP_AGES_60_63_2026_CENTS, SIMPLE_DEFERRAL_LIMIT_2026_CENTS,
};
use crate::{FederalDraft, FederalIncome, Owner, W2Code};

/// One person's excess elective deferrals.
#[derive(Clone, Copy, Default)]
pub(crate) struct Excess {
    /// Taxable excess in cents (line 1h).
    pub taxable: i64,
    /// The deferrals came from more than one W-2.
    pub several: bool,
}

fn regular(code: W2Code) -> bool {
    matches!(code, W2Code::D | W2Code::E | W2Code::F | W2Code::S)
}

fn roth(code: W2Code) -> bool {
    matches!(code, W2Code::AA | W2Code::BB)
}

// One owner's excess over every W-2 of that owner; `birth_date` fixes the catch-up.
pub(crate) fn excess(d: &FederalDraft, owner: Owner, birth_date: u32) -> Excess {
    let (mut total, mut roth_total, mut simple_only, mut forms) = (0, 0, true, 0);
    for entry in d.income.iter().filter(|e| e.owner == crate::DocumentOwner::from(owner)) {
        let FederalIncome::Wage { data } = &entry.income else {
            continue;
        };
        let data = crate::FederalWage::from(data);
        let mut counted = false;
        for c in &data.other_box12 {
            if c.amount <= 0 || !(regular(c.code) || roth(c.code)) {
                continue;
            }
            counted = true;
            total += c.amount;
            if roth(c.code) {
                roth_total += c.amount;
            }
            simple_only &= c.code == W2Code::S;
        }
        forms += i32::from(counted);
    }
    let (limit, regular_catch_up, high_catch_up) = if simple_only {
        (
            SIMPLE_DEFERRAL_LIMIT_2026_CENTS,
            SIMPLE_CATCH_UP_2026_CENTS,
            SIMPLE_CATCH_UP_AGES_60_63_2026_CENTS,
        )
    } else {
        (
            ELECTIVE_DEFERRAL_LIMIT_2026_CENTS,
            ELECTIVE_CATCH_UP_2026_CENTS,
            ELECTIVE_CATCH_UP_AGES_60_63_2026_CENTS,
        )
    };
    let catch_up = match age_at_year_end(birth_date) {
        60..=63 => high_catch_up,
        50.. => regular_catch_up,
        _ => 0,
    };
    let over = phase::floor(total - limit - catch_up, 0);
    // The Roth share of the excess is already in box 1 (pro rata to the deferrals).
    let roth_share = if total > 0 {
        (crate::kernel::ratio::trunc_i128(
            i128::from(over) * i128::from(roth_total),
            i128::from(total),
        )) as i64
    } else {
        0
    };
    Excess { taxable: over - roth_share, several: forms > 1 }
}

// The same eligible W-2 box12 codes consume an owner's shared 402(g) room.
pub(crate) fn total(d: &FederalDraft, owner: Owner) -> i64 {
    d.income
        .iter()
        .filter(|e| e.owner == crate::DocumentOwner::from(owner))
        .filter_map(|e| {
            if let FederalIncome::Wage { data } = &e.income {
                Some(crate::FederalWage::from(data))
            } else {
                None
            }
        })
        .flat_map(|w| w.other_box12)
        .filter(|c| regular(c.code) || roth(c.code))
        .map(|c| c.amount)
        .sum()
}
