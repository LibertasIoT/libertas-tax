use super::*;

// Schedule D carryover worksheet: loss use depends on unfloored taxable income.
// Current and prior years share this engine; short-term losses are used first.
// Section 1212(b)(2): taxable is unfloored taxable income before section
// 151 deductions (including the enhanced senior deduction), or AMT exemptions.
pub(crate) fn capital_carryover(short: i64, long: i64, deducted: i64, taxable: i64) -> [i64; 2] {
    let used = phase::cap(deducted, [positive(taxable + deducted)]);
    let short_loss = positive(-short);
    [
        crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
            short_loss - used,
            positive(long),
        )
        .get(),
        crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
            -long - positive(short),
            positive(used - short_loss),
        )
        .get(),
    ]
}

pub(crate) fn regular_capital_carry(v: &PriorCapitalReturn) -> [i64; 2] {
    capital_carryover(
        dollars(v.short),
        dollars(v.long),
        dollars(v.deducted_loss),
        dollars(v.agi) - dollars(v.deductions) + dollars(v.senior_deduction),
    )
}

pub(crate) fn capital_caps(v: &PriorCapitalReturn) -> crate::InvestmentCapitalCap {
    let regular = regular_capital_carry(v);
    let amt = v.amt.as_ref().map_or(regular, |a| {
        capital_carryover(
            dollars(a.short),
            dollars(a.long),
            dollars(a.deducted_loss),
            dollars(a.taxable_income),
        )
    });
    crate::InvestmentCapitalCap {
        short: regular[0] * 100,
        long: regular[1] * 100,
        amt_short: amt[0] * 100,
        amt_long: amt[1] * 100,
    }
}
// No loss-source engine is duplicated: these are portions of the carryover
// computed by the existing Schedule D owner. Missing portions have endpoints,
// never an affirmative zero classification.
pub(crate) fn investment_capital_parts(v: &PriorCapitalReturn) -> Option<[[i64; 2]; 2]> {
    let caps = capital_caps(v);
    let parts = v.investment_parts.as_ref()?;
    let regular = [dollars(parts.short), dollars(parts.long)];
    let amt = if let Some(a) = &parts.amt {
        [dollars(a.short), dollars(a.long)]
    } else if caps.short == caps.amt_short && caps.long == caps.amt_long {
        regular
    } else {
        return None;
    };
    Some([regular, amt])
}
// Relative to PriorCapitalReturn and its editable PriorCapitalReturnPage:
// investment_parts is copied field-for-field, without a Union variant segment.
pub(crate) fn investment_parts_issue(
    v: &PriorCapitalReturn,
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    let cap = capital_caps(v);
    let p = v.investment_parts.as_ref()?;
    if (cap.short != cap.amt_short || cap.long != cap.amt_long) && p.amt.is_none() {
        return Some(
            ValidationIssue::new("INVESTMENT_AMT_PARTS_REQUIRED")
                .at_field("amt")
                .in_field("investment_parts"),
        );
    }
    if !(0..=cap.short).contains(&p.short) {
        return Some(
            ValidationIssue::new("INVESTMENT_ST_ABOVE_CARRY")
                .at_field("short")
                .in_field("investment_parts"),
        );
    }
    if !(0..=cap.long).contains(&p.long) {
        return Some(
            ValidationIssue::new("INVESTMENT_LT_ABOVE_CARRY")
                .at_field("long")
                .in_field("investment_parts"),
        );
    }
    if let Some(a) = &p.amt {
        if !(0..=cap.amt_short).contains(&a.short) {
            return Some(
                ValidationIssue::new("INVESTMENT_AMT_ST_ABOVE_CARRY")
                    .at_field("short")
                    .in_field("amt")
                    .in_field("investment_parts"),
            );
        }
        if !(0..=cap.amt_long).contains(&a.long) {
            return Some(
                ValidationIssue::new("INVESTMENT_AMT_LT_ABOVE_CARRY")
                    .at_field("long")
                    .in_field("amt")
                    .in_field("investment_parts"),
            );
        }
    }
    None
}
pub(crate) fn investment_parts_valid(v: &PriorCapitalReturn) -> bool {
    investment_parts_issue(v).is_none()
}
pub(crate) fn all_capital_is_investment(v: &PriorCapitalReturn) -> bool {
    let c = capital_caps(v);
    investment_capital_parts(v)
        == Some([
            [
                crate::kernel::ratio::trunc_i64(c.short, 100),
                crate::kernel::ratio::trunc_i64(c.long, 100),
            ],
            [
                crate::kernel::ratio::trunc_i64(c.amt_short, 100),
                crate::kernel::ratio::trunc_i64(c.amt_long, 100),
            ],
        ])
}
// Retained repayment comparisons may carry source portions only when the
// actual calculated balances are unchanged, or all source loss was established
// as investment property. A changed mixed pool needs its actual source records.
pub(crate) fn revised_investment_parts(
    old: &PriorCapitalReturn,
    new: &PriorCapitalReturn,
) -> Option<crate::InvestmentCapitalParts> {
    let n = capital_caps(new);
    if all_capital_is_investment(old) {
        Some(crate::InvestmentCapitalParts {
            short: n.short,
            long: n.long,
            amt: Some(crate::InvestmentAmtCapitalParts { short: n.amt_short, long: n.amt_long }),
        })
    } else if capital_caps(old) == n {
        old.investment_parts.clone()
    } else {
        None
    }
}

pub(crate) fn prior_niit_difference(d: &FederalDraft, r: &mut FederalResult) -> i64 {
    let mut total = 0;
    for prior in &d.screening.as_ref().unwrap().capital_history {
        let context = crate::IssueContext::Record {
            section: crate::TaxSection::InvestmentSales,
            identity: crate::federal_model::NoteIdentity::FilingYear {
                section: crate::TaxSection::InvestmentSales,
                year: 2025,
                filers: match prior.owners {
                    PriorReturnOwners::Taxpayer => crate::RepaymentFilers::Taxpayer,
                    PriorReturnOwners::Spouse => crate::RepaymentFilers::Spouse,
                    PriorReturnOwners::Joint => crate::RepaymentFilers::Both,
                },
            },
            label: crate::Message::new(
                "ISSUE_REPAYMENT_SOURCE_YEAR",
                alloc::vec![
                    crate::MessageArg::Number(2025),
                    crate::MessageArg::Resource(
                        match prior.owners {
                            PriorReturnOwners::Taxpayer => "PERSON_YOU",
                            PriorReturnOwners::Spouse => "PERSON_SPOUSE",
                            PriorReturnOwners::Joint => "PERSON_YOU_AND_SPOUSE",
                        }
                        .into()
                    ),
                ],
            ),
        };
        let carry = regular_capital_carry(prior).iter().sum::<i64>();
        let difference = match prior.niit_difference {
            Some(value) => dollars(value),
            None if carry == 0 => 0,
            None => {
                crate::issue_localized!(
                    r,
                    &context,
                    "Investment-tax capital history",
                    "ISSUE_ENTER_THE_RETAINED_FORM_8960_EXCLUDED_LOSS_DIFFERENCE_INCLUDING_CON_A89352C6",
                    "Enter the retained Form 8960 excluded-loss difference, including confirmed zero; Schedule D alone does not establish the investment-tax balance.",
                );
                continue;
            }
        };
        crate::require_localized!(
            r,
            difference <= carry,
            &context,
            "Investment-tax capital history",
            "ISSUE_THE_EXCLUDED_LOSS_DIFFERENCE_MUST_FIT_THE_REGULAR_CAPITAL_CARRYOVER_C50F91A2",
            "The Form 8960 excluded-loss difference must fit the regular capital carryover; it is independent of Form 4952 investment-property attribution.",
        );
        total += difference;
    }
    total
}

// The part of the 2025 capital loss carryover that has section 1256 character
// (Pub. 550: treated in the carryover year as a loss from section 1256
// contracts), whole dollars, never more than the regular carryover itself.
pub(crate) fn prior_section_1256(d: &FederalDraft) -> i64 {
    d.screening.as_ref().map_or(0, |s| {
        s.capital_history
            .iter()
            .filter(|v| v.ordinary_history)
            .map(|v| {
                let carry = regular_capital_carry(v);
                phase::cap(dollars(v.section_1256), [carry[0] + carry[1]])
            })
            .sum()
    })
}

pub(crate) fn prior_capital(d: &FederalDraft, r: &mut FederalResult) -> [[i64; 2]; 2] {
    let entries = &d.screening.as_ref().unwrap().capital_history;
    let joint = d.setup.as_ref().unwrap().filing_status() == FederalStatus::Joint;
    let mut seen = [false; 2];
    let mut result = [[0; 2]; 2];
    for v in entries {
        let owners = match v.owners {
            PriorReturnOwners::Taxpayer => [true, false],
            PriorReturnOwners::Spouse => [false, true],
            PriorReturnOwners::Joint => [true, true],
        };
        crate::require_localized!(
            r,
            !(owners[0] && seen[0] || owners[1] && seen[1])
                && (!owners[1] || joint)
                && (matches!(v.owners, PriorReturnOwners::Joint)
                    == (v.status == FederalStatus::Joint))
                // An unsupported history was left out before the rules ran.
                && v.ordinary_history,
            &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
            "Capital loss history",
            "ISSUE_REVIEW_PRIOR_RETURN_OWNERSHIP_FILING_STATUS_AND_SPECIAL_LOSS_ADJUST_5C27A4B6",
            "Review prior-return ownership, filing status and special loss adjustments; do not duplicate a return or allocate a former spouse's loss here.",
        );
        seen[0] |= owners[0];
        seen[1] |= owners[1];
        let short = dollars(v.short);
        let long = dollars(v.long);
        let deducted = dollars(v.deducted_loss);
        crate::require_localized!(
            r,
            deducted
                == phase::cap(
                    positive(-short - long),
                    [if v.status == FederalStatus::Separate {
                        crate::year_constants::CAPITAL_LOSS_LIMIT_SEPARATE_DOLLARS
                    } else {
                        crate::year_constants::CAPITAL_LOSS_LIMIT_DOLLARS
                    }],
                ),
            &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
            "Capital loss history",
            "ISSUE_THE_PRIOR_SCHEDULE_D_NET_AMOUNTS_AND_LINE_21_LOSS_MUST_RECONCILE_11918C45",
            "The prior Schedule D net amounts and line 21 loss must reconcile.",
        );
        crate::require_localized!(
            r,
            v.senior_deduction <= v.deductions
                && v.senior_deduction
                    <= match v.status {
                        FederalStatus::Joint => {
                            2 * crate::year_constants::SENIOR_DEDUCTION_PER_PERSON_CENTS
                        }
                        FederalStatus::Separate => 0,
                        _ => crate::year_constants::SENIOR_DEDUCTION_PER_PERSON_CENTS,
                    },
            &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
            "Capital loss history",
            "ISSUE_THE_PRIOR_ENHANCED_SENIOR_DEDUCTION_MUST_BE_INCLUDED_IN_TOTAL_DEDUC_A4F69A67",
            "The prior enhanced senior deduction must be included in total deductions and respect that return's filing-status limit. Do not enter the additional standard deduction here.",
        );
        let carry = regular_capital_carry(v);
        // The section 1256 part is a part of the regular carryover, not more.
        crate::require_localized!(
            r,
            dollars(v.section_1256) <= carry[0] + carry[1],
            &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
            "Capital loss history",
            "ISSUE_THE_SECTION_1256_PART_OF_THE_PRIOR_CARRYOVER_EXCEEDS_THE_CARRYOVER",
            "The part of the 2025 carryover from section 1256 contracts cannot be more than the short- and long-term carryovers calculated from that return.",
        );
        if !v.origins.is_empty() {
            let totals = v.origins.iter().fold([0i64; 2], |mut total, origin| {
                total[0] += origin.short;
                total[1] += origin.long;
                total
            });
            crate::require_localized!(
                r,
                [dollars(totals[0]), dollars(totals[1])] == carry,
                &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
                "Capital loss history",
                "ISSUE_LOSS_ORIGIN_WORKSHEET_BALANCES_MUST_RECONCILE_SEPARATELY_TO_THE_CAL_CA991D60",
                "Loss-origin worksheet balances must reconcile separately to the calculated short- and long-term carryovers after 2025 use. Supply all surviving years or leave the optional history empty when unavailable.",
            );
        }
        let alternative = if let Some(amt) = &v.amt {
            let short = dollars(amt.short);
            let long = dollars(amt.long);
            let deducted = dollars(amt.deducted_loss);
            crate::require_localized!(
                r,
                deducted
                    == phase::cap(
                        positive(-short - long),
                        [if v.status == FederalStatus::Separate {
                            crate::year_constants::CAPITAL_LOSS_LIMIT_SEPARATE_DOLLARS
                        } else {
                            crate::year_constants::CAPITAL_LOSS_LIMIT_DOLLARS
                        }],
                    ),
                &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
                "Capital loss history",
                "ISSUE_THE_PRIOR_AMT_SCHEDULE_D_NET_GAINS_LOSSES_AND_LINE_21_DEDUCTION_MUS_AE0A8857",
                "The prior AMT Schedule D net gains/losses and line 21 deduction must reconcile independently from regular tax.",
            );
            capital_carryover(short, long, deducted, dollars(amt.taxable_income))
        } else {
            carry
        };
        for (sum, amounts) in result.iter_mut().zip([carry, alternative]) {
            sum[0] += amounts[0];
            sum[1] += amounts[1];
        }
    }
    result
}

pub(crate) fn valid_capital_history(entries: &[PriorCapitalReturn]) -> bool {
    // Tax structure: one prior-return capital record per taxpayer or spouse.
    entries.len() <= 2
        && entries.iter().all(|v| {
            [v.agi, v.short, v.long]
                .iter()
                .all(|n| (-100_000_000_000..=100_000_000_000).contains(n))
                && (0..=100_000_000_000).contains(&v.deductions)
                && (0..=300_000).contains(&v.deducted_loss)
                && (0..=100_000_000_000).contains(&v.section_1256)
                && (0..=1_200_000).contains(&v.senior_deduction)
                && v.niit_difference.is_none_or(|amount| (0..=100_000_000_000).contains(&amount))
                && v.amt.as_ref().is_none_or(|a| {
                    [a.taxable_income, a.short, a.long]
                        .iter()
                        .all(|n| (-100_000_000_000..=100_000_000_000).contains(n))
                        && (0..=300_000).contains(&a.deducted_loss)
                })
                && investment_parts_valid(v)
                && v.injured_owners.as_ref().is_none_or(|owners| {
                    matches!(v.owners, PriorReturnOwners::Joint)
                        && owners.valid(regular_capital_carry(v).map(|amount| amount * 100))
                })
                && v.origins.iter().enumerate().all(|(i, origin)| {
                    (1900..=2025).contains(&origin.year)
                        // Arithmetic safety: the unique finite source years and
                        // these scalar bounds protect unchecked origin totals.
                        && (0..=100_000_000_000).contains(&origin.short)
                        && (0..=100_000_000_000).contains(&origin.long)
                        && !v.origins[..i].iter().any(|prior| prior.year == origin.year)
                })
        })
}
