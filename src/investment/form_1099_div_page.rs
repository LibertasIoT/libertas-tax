#[allow(unused_imports)]
use super::*;

pub(super) fn project_dividend_issue(issue: ValidationIssue) -> ValidationIssue {
    issue
        .map_field_prefix(&["nominee"], &["nominee", "value"])
        .map_field_prefix(&["after_death"], &["answers", "after_death", "value"])
}

// IRS Form 1099-DIV instructions, boxes 2c and 5, and Schedule B nominee reporting.
// https://www.irs.gov/instructions/i1099div
pub(super) fn div_calls(final_return: bool, w: &crate::DivBoxes) -> crate::DivCalls {
    let open = |called: bool, choices: &[i32]| if called { choices.to_vec() } else { Vec::new() };
    let reported = [
        w.ordinary,
        w.qualified,
        w.more.capital_distributions,
        w.more.unrecaptured_distributions,
        w.more.collectible_distributions,
        w.more.section_1202_gain,
        w.more.section199a,
        w.more.exempt_dividends,
        w.more.private_activity_dividends,
        w.more.foreign_tax,
        w.more.nondividend_distributions,
        w.more.cash_liquidation,
        w.more.noncash_liquidation,
    ]
    .iter()
    .any(|value| *value != 0);
    crate::DivCalls {
        nominee: open(reported, &[0]),
        // Pub. 559: only a final-return statement can include income of the estate.
        // https://www.irs.gov/publications/p559
        after_death: open(reported && final_return, &[0]),
        reit_eligible: open(w.more.section199a > 0, &[0, 1]),
        section_1202_exclusion: open(w.more.section_1202_gain > 0, &[0, 1, 2, 3]),
    }
}
