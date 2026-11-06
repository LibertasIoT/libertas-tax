// Coverage: Form 2555 — Supported (2026-10-09).
//   Page: the Work abroad income topic (TaxSection::WorkAbroad, IncomeTopics::WORK_ABROAD) offers
//   Form2555 (one per person, FORM_2555_TWICE; stored as ForeignEarnedClaim in
//   FederalScreening.foreign_earned, a spouse only on a joint return) and ForeignPay (pay from a
//   foreign employer with no W-2; stored as FederalIncome::ForeignEmployment with the employer as
//   the entry label). The source marks sit on their own pages: W2Answers.abroad (Wages; FederalWage
//   .abroad; W2_ABROAD_ABOVE_WAGES; never an import target) and BusinessDetails.source
//   (Self-employment, More about this business; BusinessSource::ForeignServices { confirmed }).
//   Import: none; no IRS document carries these facts. RareSituation::TaxTreaty (Form 8833) is
//   the remaining refusal of the old Work abroad situation.
//   Lines (2026 draft, the 2025 numbering): Part I lines 6c-9 and Part II lines 7, 13 and 15 are
//   the tax_home and Residence basis/nonresident-statement facts; line 10 Residence.began/ended (a period
//   that includes a whole calendar year); lines 16 and 18 Presence.starts_on and foreign_days, the
//   count entered as the form's total, the window ending starts_on + 364 days; Part IV lines 19, 20a,
//   24 and 26 (current-service W-2/foreign-employer pay + foreign-services receipts);
//   Removes actual direct2025 pay and noncurrent award portions from
//   PartIV using shared exact ratios. Optional2025 history caps the direct/award
//   prior exclusion by130000*days/365 less actual2025line42 and adds it once at
//   line45 after current-year housing/business expense owners. PriorYearOnly
//   requires actual2025 history and yields zero current exclusion/housing; no
//   invented2026 qualification. Keeps independent earned/housing election
//   revocations and actual IRS consent, residence citizenship/treaty
//   basis and the nonresident statement/end facts; reuses minimum_tax_credit.
//   Part V line 27; Part VI lines 28-36 (ForeignHousing::calculate: Notice 2026-25 locality limits,
//   the 16% base and 30% general limit derived from the year constant, line 35 by employer_exclusion);
//   Part VII lines 37-42; Part VIII lines 43-45 (foreign_business::exclude; the line 44 numerator is
//   the exclusion alone, Pub. 54 "Items Related to Excluded Income" and Reg. 1.911-6(a) Example 5);
//   Part IX lines 46-50 with the 2025 carryover worksheet. Excluded: Part I lines 1-5, 6a-6b, Part II
//   lines 11-12, Part III line 17, Part IV lines 20b, 21-23 and 25 (informational, or a source with
//   no record; the field descriptions say what is out). Feeds: Schedule 1 lines 8d and 24j, the
//   Foreign Earned Income Tax Worksheet and the Form 6251 worksheet, no EIC or ACTC, MAGIs, Form 8960
//   line 13, Form 1116 lines 1a, 3d, 3e, 4b and 12 (extended_rules::foreign_credit; the general
//   category, per owner through ForeignOrdinaryTax.owner, also for the AMT credit).
//   Current-year tax-home No takes no current exclusion or housing silently;
//   fewer than 330 full days adds the non-blocking
//   WARN_FOREIGN_PRESENCE_TEST_NOT_MET; an unconfirmed foreign-services business
//   is left out with its own note; a record with no foreign earned
//   income and no carryover claims nothing (WARN_FORM_2555_NO_FOREIGN_INCOME). All in unknowns.rs.
//   Refused by name: a foreign business with property, vehicles, a home office, intangibles,
//   inventory, a non-services source, or owner health/retirement deductions; Form 1116 with a
//   foreign business (FTC_FOREIGN_BUSINESS); a Form 2106 deduction of the same person
//   (ISSUE_FORM_2555_LINE_44_EMPLOYEE_EXPENSES); a separate return splitting community wages
//   (ISSUE_FORM_2555_COMMUNITY_SPLIT); housing stays outside the qualifying period or overlapping;
//   the spouses' household allocation (check_households). Not modeled: partnership earnings (line
//   20b), the section 911(d)(4) waiver, U.S. Government pay, a housing
//   deduction in the Form 1116 line 2 allocation (it joins the not-definitely-related deductions).
//   Evidence: M/foreign_housing.rs, foreign_business.rs, extended_model.rs ForeignWages and
//   ForeignQualification, extended_rules.rs foreign() and foreign_credit(), unknowns.rs
//   foreign_claims; T/work_abroad.rs (F3, F4, the two-spouse case, the 330-day boundary and window,
//   the unknowns and refusals, the page round trips and rules), T/foreign.rs, tests/returns.rs
//   fixtures bona-fide-resident-w2-at-limit (F1) and physical-presence-partial-year-housing (F2);
//   IRS i2555, Pub. 54, Notice 2026-25, Rev. Proc. 2025-32 section 4.39.

pub use crate::foreign_earned_source::{
    ExclusionRevocation, ForeignEarnedClaim, ForeignHousing, ForeignHousingHousehold,
    ForeignHousingLocation, ForeignHousingStay, HousingCarryover, MultiYearPay, PriorYearHistory,
    RevokedElections,
};
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::line_emission::emit as emit_lines;
use crate::{FederalResult, ForeignQualification, Owner};
use alloc::{format, string::String, vec::Vec};
use libertas_macros::*;

// Form 2555 lines 35–36: apply the employer share of total foreign earned
// income, then the employer-amount cap. Retain full ratio precision; the form
// requires at least three decimal places, not a mandatory three-place cut-off.
pub(crate) fn employer_exclusion(amount: i64, employer: i64, foreign_income: i64) -> i64 {
    if foreign_income <= 0 || employer <= 0 {
        return 0;
    }
    phase::cap(
        fraction(positive(amount), phase::cap(employer, [foreign_income]), foreign_income),
        [employer],
    )
}

#[derive(Default)]
pub(crate) struct HousingAmounts {
    pub amount: i64,
    pub exclusion: i64,
}

// Form 2555 Part IX and its prior-year carryover worksheet, in whole dollars.
pub(crate) fn deduction(
    amount: i64,
    income_limit: i64,
    prior: Option<&HousingCarryover>,
    person: &str,
    r: &mut FederalResult,
) -> i64 {
    let limit = positive(income_limit);
    let current = phase::cap(amount, [limit]);
    // Retained 2025 housing deduction less the amount already deducted.
    let previous = prior.map_or(0, |p| {
        positive(dollars(crate::kernel::carry::native_balance(p.amount, p.deducted)))
    });
    let used = phase::cap(previous, [positive(limit - current)]);
    let prior_settlement =
        crate::kernel::carry::settle_values::<crate::kernel::carry::Dollars>(previous, used, true)
            .expect("prior housing use is capped by its nonnegative balance");
    emit_lines(
        r,
        &mut (),
        [
            ("current_deduction", current),
            ("prior_deduction_used", used),
            (
                "deduction_carryforward",
                crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
                    amount, current,
                )
                .get(),
            ),
            ("prior_deduction_expired", prior_settlement.expired.get()),
        ],
        |(suffix, value), _, _| {
            Some(calculation_row(
                None,
                &format!("foreign_housing_{person}_{suffix}"),
                LineValue::MoneyDollars(value),
                &["income.business.source.housing"],
                "https://www.irs.gov/instructions/i2555",
                msg!("EXPLAIN_FOREIGN_HOUSING_N_N"),
            ))
        },
    );
    current + used
}

/// Form 2555: foreign earned income exclusion
/// For a U.S. citizen or resident alien whose tax home was in a foreign country. Pay received in 2026 for qualifying 2025 services uses the retained 2025 limit. Partnership earnings for services abroad, advances for later years and the waiver for leaving a country because of war or unrest are not included. Your elections stay in effect for later years until you revoke them.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct Form2555 {
    /// How you qualify
    /// Form 2555 Part II or Part III; use one test. If you had to leave a country because of war or civil unrest before meeting it, the waiver for 2026 is not available yet.
    pub qualification: ForeignQualification,
    /// Your tax home was in a foreign country throughout this period
    /// Your regular place of work was abroad and your abode (family, economic and personal ties) was not in the United States, unless you served in a combat zone in support of the U.S. Armed Forces. Current-year pay and business income you marked as earned abroad was earned in this period; prior-service pay uses the separate 2025 records, none of it is pay from the U.S. Government or its agencies as their employee, none was earned in Cuba against U.S. travel rules. For a current-year qualification, No takes no current exclusion or housing amount. Check your work-location, residence and pay records for this period. The prior-service-only answer instead uses the actual 2025 qualifying history.
    pub tax_home: bool,
    /// Earlier revocation of a foreign exclusion
    #[libertas_default(None)]
    pub revocation: ExclusionRevocation,
    /// Exclude your foreign earned income
    /// Up to $132,900 for a full qualifying year, figured by days otherwise. You cannot take the earned income credit or the additional child tax credit, nor a credit for foreign taxes on excluded pay. Not claiming it in a later year, or claiming those credits, revokes it; a new choice then needs IRS consent for five years.
    /// [TrueText]
    /// Yes, exclude it
    /// [FalseText]
    /// No, housing only
    pub exclusion: bool,
    /// Direct pay received in 2026 for services performed in 2025
    /// Already included in your W-2 pay for work abroad or foreign employer pay. Exclude every award listed below: its 2025 portion is calculated separately. A normal year-end payroll period of 16 days or fewer paid in 2026 may be treated as 2026 pay instead. This amount leaves current-year foreign earned income even if you decline the optional prior-year exclusion below.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub prior_year_pay: i64,
    /// Exclusion for pay earned in 2025 and received in 2026
    /// The direct pay above and the awards' qualifying 2025 portions use this same retained 2025 exclusion capacity. Without these records, no prior-service exclusion is claimed; that pay still leaves current-year foreign earned income.
    /// [NullText]
    /// Do not claim an exclusion for pay earned in 2025
    /// [SetText]
    /// The exclusion for pay earned in 2025 uses these return records
    #[libertas_default($null)]
    pub prior_year_history: Option<PriorYearHistory>,
    /// Bonuses or stock awards earned over more than one year or partly in the United States
    /// Already included in your W-2 pay for work abroad or foreign employer pay. The App attributes each award by its service months, keeps the qualifying 2026 portion in current foreign earned income, and uses the records above for any 2025 exclusion. Do not enter an award again as direct prior-year pay.
    /// ----
    /// Award
    pub multi_year_pay: Vec<MultiYearPay>,
    /// Foreign housing
    /// Once for this person: employer-paid housing is excluded, and housing paid from self-employment earnings is deducted.
    #[libertas_default(None)]
    pub housing: ForeignHousing,
}

impl Form2555 {
    pub(crate) fn record(&self, owner: Owner) -> ForeignEarnedClaim {
        ForeignEarnedClaim {
            owner,
            qualification: self.qualification.clone(),
            tax_home: self.tax_home,
            revocation: self.revocation.clone(),
            exclusion: self.exclusion,
            prior_year_pay: self.prior_year_pay,
            prior_year_history: self.prior_year_history.clone(),
            multi_year_pay: self.multi_year_pay.clone(),
            housing: self.housing.clone(),
        }
    }
    pub(crate) fn from_record(record: &ForeignEarnedClaim) -> Self {
        Self {
            qualification: record.qualification.clone(),
            tax_home: record.tax_home,
            revocation: record.revocation.clone(),
            exclusion: record.exclusion,
            prior_year_pay: record.prior_year_pay,
            prior_year_history: record.prior_year_history.clone(),
            multi_year_pay: record.multi_year_pay.clone(),
            housing: record.housing.clone(),
        }
    }
    // The page's client rules, repeated on save in the same order.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if !self.revocation.valid() {
            return Some(
                ValidationIssue::new("FOREIGN_REVOCATION_YEAR_OUTSIDE_PERIOD")
                    .at_field("revocation"),
            );
        }
        if !(0..=100000000000).contains(&self.prior_year_pay) {
            return Some(ValidationIssue::new("PRIOR_YEAR_PAY_INVALID").at_field("prior_year_pay"));
        }
        if self.prior_year_history.as_ref().is_some_and(|p| !p.valid()) {
            return Some(
                ValidationIssue::new("PRIOR_YEAR_PAY_INVALID").at_field("prior_year_history"),
            );
        }
        // Temporary arithmetic exclusion: stored pay_valid lacks the source-pay
        // relation; employee_pay still expects a representable final Exact result.
        if self.multi_year_pay.len() > 10 {
            return Some(
                ValidationIssue::new("MULTI_YEAR_PAY_MONTHS_ABOVE_TOTAL")
                    .at_field("multi_year_pay"),
            );
        }
        if let Some((i, _)) = self.multi_year_pay.iter().enumerate().find(|(_, p)| !p.valid()) {
            return Some(
                ValidationIssue::new("MULTI_YEAR_PAY_MONTHS_ABOVE_TOTAL")
                    .at_list_item("multi_year_pay", i),
            );
        }
        if !self.exclusion && !matches!(self.housing, ForeignHousing::Claim { .. }) {
            return Some(ValidationIssue::new("FORM_2555_NOTHING_CLAIMED").at_field("exclusion"));
        }
        match &self.qualification {
            ForeignQualification::PriorYearOnly => {
                if self.prior_year_history.is_none() {
                    return Some(
                        ValidationIssue::new("FOREIGN_PRIOR_ONLY_NEEDS_HISTORY")
                            .at_field("prior_year_history"),
                    );
                }
                if self.housing != ForeignHousing::None {
                    return Some(
                        ValidationIssue::new("FOREIGN_PRIOR_ONLY_NEEDS_HISTORY")
                            .at_field("housing"),
                    );
                }
                if let Some((i, _)) = self
                    .multi_year_pay
                    .iter()
                    .enumerate()
                    .find(|(_, a)| a.months_in_return_year != 0)
                {
                    return Some(
                        ValidationIssue::new("FOREIGN_PRIOR_ONLY_NEEDS_HISTORY")
                            .at_field("months_in_return_year")
                            .in_list("multi_year_pay", i),
                    );
                }
            }
            ForeignQualification::Residence { began, ended, .. } => {
                if *began > 20261231 {
                    return Some(
                        ValidationIssue::new("FOREIGN_RESIDENCE_BEGAN_AFTER_RETURN_YEAR")
                            .at_field("began")
                            .in_field("qualification"),
                    );
                }
                if ended.is_some_and(|e| e < 20260101 || e <= *began) {
                    return Some(
                        ValidationIssue::new("FOREIGN_RESIDENCE_ENDED_BEFORE_RETURN_YEAR")
                            .at_field("ended")
                            .in_field("qualification"),
                    );
                }
                let whole_year = *began <= 20250101
                    || ended.is_none_or(|e| e >= 20271231)
                    || (*began <= 20260101 && ended.is_some_and(|e| e >= 20261231));
                if !whole_year {
                    return Some(
                        ValidationIssue::new("FOREIGN_RESIDENCE_NO_FULL_YEAR")
                            .at_field("qualification"),
                    );
                }
            }
            ForeignQualification::Presence { starts_on, foreign_days } => {
                if !(20250102..=20261231).contains(starts_on) {
                    return Some(
                        ValidationIssue::new("FOREIGN_PRESENCE_PERIOD_OUTSIDE_RETURN_YEAR")
                            .at_field("starts_on")
                            .in_field("qualification"),
                    );
                }
                if *foreign_days > 365 {
                    return Some(
                        ValidationIssue::new("FOREIGN_PRESENCE_DAYS_ABOVE_PERIOD")
                            .at_field("foreign_days")
                            .in_field("qualification"),
                    );
                }
            }
        }
        if let ForeignHousing::Claim { stays, prior, household, expenses, .. } = &self.housing {
            for (i, stay) in stays.iter().enumerate() {
                let field = if stay.starts_on > stay.ends_on {
                    Some("ends_on")
                } else if stay.starts_on < 20260101 {
                    Some("starts_on")
                } else if stay.ends_on > 20261231 {
                    Some("ends_on")
                } else {
                    None
                };
                if let Some(field) = field {
                    return Some(
                        ValidationIssue::new("FOREIGN_HOUSING_STAY_OUTSIDE_RETURN_YEAR")
                            .at_field(field)
                            .in_list("stays", i)
                            .in_field("housing"),
                    );
                }
            }
            for (i, stay) in stays.iter().enumerate() {
                let field = match &self.qualification {
                    ForeignQualification::PriorYearOnly => Some("starts_on"),
                    ForeignQualification::Presence { starts_on, .. } => {
                        if stay.starts_on < *starts_on {
                            Some("starts_on")
                        } else if crate::dates::days_between(*starts_on, stay.ends_on) > 364 {
                            Some("ends_on")
                        } else {
                            None
                        }
                    }
                    ForeignQualification::Residence { began, ended, .. } => {
                        if stay.starts_on < *began {
                            Some("starts_on")
                        } else if ended.is_some_and(|e| stay.ends_on > e) {
                            Some("ends_on")
                        } else {
                            None
                        }
                    }
                };
                if let Some(field) = field {
                    return Some(
                        ValidationIssue::new("FOREIGN_HOUSING_STAY_OUTSIDE_PERIOD")
                            .at_field(field)
                            .in_list("stays", i)
                            .in_field("housing"),
                    );
                }
            }
            if prior.as_ref().is_some_and(|p| p.deducted > p.amount) {
                return Some(
                    ValidationIssue::new("HOUSING_CARRYOVER_DEDUCTED_ABOVE_AMOUNT")
                        .at_field("deducted")
                        .in_field("prior")
                        .in_field("housing"),
                );
            }
            let above = match household {
                ForeignHousingHousehold::Shared { total_expenses } => expenses > total_expenses,
                ForeignHousingHousehold::SeparateReturnShared {
                    total_expenses,
                    spouse_expenses,
                    ..
                } => {
                    i128::from(*expenses) + i128::from(*spouse_expenses)
                        > i128::from(*total_expenses)
                }
                _ => false,
            };
            if above {
                return Some(
                    ValidationIssue::new("HOUSING_SHARES_ABOVE_TOTAL")
                        .at_field("expenses")
                        .in_field("housing"),
                );
            }
        }
        None
    }
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
}

pub(crate) fn source_pay_rule(d: &crate::FederalDraft) -> Option<&'static str> {
    source_pay_issue(d).map(|(_, issue)| issue.key)
}
pub(crate) fn source_pay_issue(
    d: &crate::FederalDraft,
) -> Option<(crate::Owner, crate::validation::ValidationIssue)> {
    source_pay_issue_for_owners(d, [true; 2])
}
pub(crate) fn source_pay_issue_for_owners(
    d: &crate::FederalDraft,
    owners: [bool; 2],
) -> Option<(crate::Owner, crate::validation::ValidationIssue)> {
    for claim in &d.screening.as_ref()?.foreign_earned {
        if !owners[usize::from(claim.owner == crate::Owner::Spouse)] {
            continue;
        }
        let gross: i128 = d
            .income
            .iter()
            .filter(|e| e.owner == crate::DocumentOwner::from(claim.owner))
            .filter_map(|e| match &e.income {
                crate::FederalIncome::Wage { data } => {
                    Some(i128::from(crate::FederalWage::from(data).abroad))
                }
                crate::FederalIncome::ForeignEmployment { data } => Some(i128::from(data.wages)),
                _ => None,
            })
            .sum();
        let entered = i128::from(claim.prior_year_pay)
            + claim.multi_year_pay.iter().map(|p| i128::from(p.amount)).sum::<i128>();
        if entered > gross {
            return Some((
                claim.owner,
                crate::validation::ValidationIssue::new("ISSUE_FOREIGN_AWARDS_ABOVE_PAY").at_field(
                    if i128::from(claim.prior_year_pay) > gross {
                        "prior_year_pay"
                    } else {
                        "multi_year_pay"
                    },
                ),
            ));
        }
    }
    None
}

/// Pay from a foreign employer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignPay {
    /// Employer
    /// Leave empty to name this pay by its kind and number.
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub employer: String,
    /// Pay received in 2026 for work you did abroad, in U.S. dollars
    /// Include cash allowances, reimbursements and tax paid for you that are part of the pay, at the exchange rate when you received it. Include the fair rental value of employer-provided housing (also enter its eligible housing expenses), qualifying 2025-service pay and the complete awards listed on Form 2555. Do not enter pay a W-2 reports, ordinary pay for U.S. work, advances for later services, government employee pay or pensions. List a mixed-place award in full on Form 2555 so its U.S. portion is removed once.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub wages: i64,
}

impl ForeignHousing {
    pub(crate) fn calculate(
        &self,
        qualification: &ForeignQualification,
        wages: i64,
        foreign_income: i64,
        spouse: bool,
        r: &mut FederalResult,
    ) -> HousingAmounts {
        let Self::Claim { stays, expenses, nontaxable_allowances, .. } = self else {
            return HousingAmounts::default();
        };
        let context = crate::IssueContext::owner(
            crate::TaxSection::WorkAbroad,
            if spouse { Owner::Spouse } else { Owner::Taxpayer },
        );
        crate::require_localized!(
            r,
            self.valid(),
            &context,
            "Foreign housing",
            "ISSUE_CONFIRM_ELIGIBLE_HOUSING_EXPENSES_QUALIFYING_PERIOD_AND_ELECTION_CO_2CE0C4D2",
            "Enter valid housing expenses, allowances, household amounts and 2026 stay dates.",
        );
        let days = match crate::extended_rules::exclusion_days(qualification) {
            Ok(days) if days > 0 => days,
            _ => {
                crate::require_localized!(
                    r,
                    false,
                    &context,
                    "Foreign housing",
                    "ISSUE_HOUSING_REQUIRES_A_VALID_QUALIFYING_PERIOD_OVERLAPPING_RETURN_YEAR_8239D8F6",
                    "Housing requires a valid qualifying period overlapping 2026.",
                );
                return HousingAmounts::default();
            }
        };
        if !self.valid() {
            return HousingAmounts::default();
        }
        // The qualifying period's first and last day as ordinals, clipped to
        // 2026 at the start. The client rule on Form2555 states the same
        // bounds.
        let (start, last) = crate::extended_rules::period_ordinals(qualification);
        let dates_ok = stays.iter().enumerate().all(|(i, s)| {
            s.starts_on <= s.ends_on
                && crate::dates::ordinal(s.starts_on) >= start
                && crate::dates::ordinal(s.ends_on) <= last
                && stays[..i].iter().all(|p| s.ends_on < p.starts_on || s.starts_on > p.ends_on)
        });
        crate::require_localized!(
            r,
            dates_ok,
            &context,
            "Foreign housing dates",
            "ISSUE_HOUSING_STAYS_MUST_HAVE_NONOVERLAPPING_INCLUSIVE_DATES_WITHIN_THE_2_E1279FA2",
            "Housing stays must have nonoverlapping inclusive dates within the 2026 qualifying period.",
        );
        if !dates_ok {
            return HousingAmounts::default();
        }
        let mut limit = 0;
        for (i, stay) in stays.iter().enumerate() {
            if stays[..i].iter().any(|p| p.location == stay.location) {
                continue;
            }
            let location_days: i64 = stays
                .iter()
                .filter(|p| p.location == stay.location)
                .map(|p| i64::from(crate::dates::days_between(p.starts_on, p.ends_on) + 1))
                .sum();
            let (annual, daily) = stay.location.limits();
            limit += if location_days == 365 { annual } else { dollars(daily * location_days) };
        }
        let base = if days == 365 {
            crate::year_constants::FOREIGN_BASE_HOUSING_DOLLARS
        } else {
            dollars(crate::year_constants::FOREIGN_BASE_HOUSING_DAILY_CENTS * days)
        };
        let allowance = dollars(*nontaxable_allowances);
        let housing_amount = positive(phase::cap(dollars(*expenses), [limit]) - base - allowance);
        let excluded = employer_exclusion(housing_amount, wages, foreign_income);
        let owner = if spouse { "spouse" } else { "taxpayer" };
        emit_lines(
            r,
            &mut (),
            [
                ("expenses", dollars(*expenses), msg!("EXPLAIN_FOREIGN_HOUSING_N_N_EXPENSES")),
                ("limit", limit, msg!("EXPLAIN_FOREIGN_HOUSING_N_N_LIMIT")),
                ("base", base, msg!("EXPLAIN_FOREIGN_HOUSING_N_N_BASE")),
                (
                    "nontaxable_allowances",
                    allowance,
                    msg!("EXPLAIN_FOREIGN_HOUSING_N_N_NONTAXABLE_ALLOWANCES"),
                ),
                ("exclusion", excluded, msg!("EXPLAIN_FOREIGN_HOUSING_N_N_EXCLUSION")),
            ],
            |(suffix, amount, detail), _, _| {
                Some(calculation_row(
                    None,
                    &format!("foreign_housing_{owner}_{suffix}"),
                    LineValue::MoneyDollars(amount),
                    &["income.foreign_employment.housing", "income.business.source.housing"],
                    "https://www.irs.gov/instructions/i2555",
                    detail,
                ))
            },
        );
        HousingAmounts { amount: housing_amount, exclusion: excluded }
    }
}

pub(crate) fn check_households(
    housing: [Option<&ForeignHousing>; 2],
    separate_return: bool,
    r: &mut FederalResult,
) {
    use ForeignHousingHousehold::*;
    let claims: Vec<_> = housing
        .into_iter()
        .flatten()
        .filter_map(|h| match h {
            ForeignHousing::Claim { household, expenses, stays, .. } => {
                Some((household, *expenses, stays))
            }
            ForeignHousing::None => None,
        })
        .collect();
    let valid = match claims.as_slice() {
        [] => true,
        [(OnlyClaim, _, _)] => true,
        [(Adverse { confirmed: true }, _, stays)] => {
            stays.iter().all(|s| s.location == ForeignHousingLocation::General)
        }
        [(AdverseSameLocation { second_location, confirmed: true }, _, stays)] => {
            *second_location != ForeignHousingLocation::General
                && stays.iter().all(|s| s.location == *second_location)
        }
        [(AdverseSecondOnly { primary_location, second_location, confirmed: true }, _, stays)] => {
            *primary_location != ForeignHousingLocation::General
                && *second_location != ForeignHousingLocation::General
                && primary_location != second_location
                && stays.iter().all(|s| s.location == *second_location)
        }
        [(Separate { qualified: true }, _, _)] => separate_return,
        [(SeparateReturnShared { total_expenses, spouse_expenses, confirmed: true }, own, _)] => {
            separate_return && own + spouse_expenses <= *total_expenses
        }
        [(Shared { total_expenses: a }, x, first), (Shared { total_expenses: b }, y, second)] => {
            a == b
                && x + y <= *a
                && first.iter().all(|p| {
                    second.iter().all(|q| {
                        p.ends_on < q.starts_on
                            || q.ends_on < p.starts_on
                            || p.location == q.location
                    })
                })
        }
        [(Separate { qualified: true }, _, _), (Separate { qualified: true }, _, _)] => true,
        _ => false,
    };
    crate::require_localized!(
        r,
        valid,
        &crate::IssueContext::Section(crate::TaxSection::WorkAbroad),
        "Foreign housing allocation",
        "ISSUE_USE_ONE_HOUSEHOLD_CLAIMANT_INCLUDING_A_CONFIRMED_ADVERSE_CONDITION_7185B7A1",
        "Use one household claimant (including a confirmed adverse-condition second household at general-limit locations, in one matching listed locality, or with all expenses at a second listed locality), or matching shared-household totals with nonduplicated expense shares and consistent locations on overlapping dates, or two independently qualifying separate households. Joint-return separate-claim choices need both spouse entries; the external shared-allocation choice requires married filing separately and confirmed spouse expense records.",
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResultState;
    use alloc::vec;

    fn result() -> FederalResult {
        FederalResult {
            rental_home_context: None,
            payroll_known_rows: None,
            payment_withholding: None,
            credit_capacity: None,
            automatic_itemizes: false,
            financial_source_scope_complete: false,
            elective_sources: None,
            withholding_sources: None,
            expense_roles: None,
            injured_spouse: None,
            state: ResultState::Incomplete,
            revision: 0,
            issues: vec![],
            review_notes: vec![],
            lines: vec![],
            return_lines: vec![],
            tax: None,
            farmland_installments: None,
            refundable_credits: None,
            payments: None,
            refund: None,
            balance: None,
            capital_footing: None,
        }
    }

    fn residence() -> ForeignQualification {
        ForeignQualification::Residence {
            began: 20200101,
            ended: None,
            basis: crate::ResidenceBasis::Citizen,
            nonresident_statement: false,
        }
    }

    fn adverse_claim(
        location: ForeignHousingLocation,
        second_location: ForeignHousingLocation,
        confirmed: bool,
    ) -> ForeignHousing {
        ForeignHousing::Claim {
            stays: vec![ForeignHousingStay { location, starts_on: 20260101, ends_on: 20261231 }],
            expenses: 20_000_000,
            nontaxable_allowances: 0,
            prior: None,
            household: ForeignHousingHousehold::AdverseSameLocation { second_location, confirmed },
        }
    }

    #[test]
    fn adverse_second_household_shares_one_high_cost_limit_and_base() {
        let claim = adverse_claim(
            ForeignHousingLocation::ChinaHongKong,
            ForeignHousingLocation::ChinaHongKong,
            true,
        );
        assert_eq!(ForeignHousing::from_avro(&claim.to_avro()).unwrap(), claim);
        let mut r = result();
        check_households([Some(&claim), None], false, &mut r);
        let housing = claim.calculate(&residence(), 200_000, 200_000, false, &mut r);
        assert!(r.issues.is_empty(), "{:#?}", r.issues);
        // Notice 2026-25: one $114,300 Hong Kong cap, minus one $21,264 base.
        assert_eq!(housing.amount, 93_036);
        assert_eq!(housing.exclusion, 93_036);
        assert_eq!(
            r.lines.iter().find(|line| line.id == "foreign_housing_taxpayer_limit").unwrap().amount,
            11_430_000
        );
    }

    #[test]
    fn adverse_second_only_household_uses_its_own_location_limit() {
        let mut claim = adverse_claim(
            ForeignHousingLocation::AustraliaSydney,
            ForeignHousingLocation::AustraliaSydney,
            true,
        );
        let ForeignHousing::Claim { household, .. } = &mut claim else { unreachable!() };
        *household = ForeignHousingHousehold::AdverseSecondOnly {
            primary_location: ForeignHousingLocation::ChinaHongKong,
            second_location: ForeignHousingLocation::AustraliaSydney,
            confirmed: true,
        };
        assert_eq!(ForeignHousing::from_avro(&claim.to_avro()).unwrap(), claim);
        let mut r = result();
        check_households([Some(&claim), None], false, &mut r);
        let housing = claim.calculate(&residence(), 200_000, 200_000, false, &mut r);
        assert!(r.issues.is_empty(), "{:#?}", r.issues);
        // Only the Sydney household has entered costs: one $65,600 limit
        // less one $21,264 base. Hong Kong's $114,300 limit is not additive.
        assert_eq!(housing.amount, 44_336);
        assert_eq!(housing.exclusion, 44_336);
        assert_eq!(
            r.lines.iter().find(|line| line.id == "foreign_housing_taxpayer_limit").unwrap().amount,
            6_560_000
        );
        for (primary, second, stay, confirmed) in [
            (
                ForeignHousingLocation::ChinaHongKong,
                ForeignHousingLocation::ChinaHongKong,
                ForeignHousingLocation::ChinaHongKong,
                true,
            ),
            (
                ForeignHousingLocation::General,
                ForeignHousingLocation::AustraliaSydney,
                ForeignHousingLocation::AustraliaSydney,
                true,
            ),
            (
                ForeignHousingLocation::ChinaHongKong,
                ForeignHousingLocation::AustraliaSydney,
                ForeignHousingLocation::ChinaHongKong,
                true,
            ),
            (
                ForeignHousingLocation::ChinaHongKong,
                ForeignHousingLocation::AustraliaSydney,
                ForeignHousingLocation::AustraliaSydney,
                false,
            ),
        ] {
            let mut invalid = claim.clone();
            let ForeignHousing::Claim { household, stays, .. } = &mut invalid else {
                unreachable!()
            };
            *household = ForeignHousingHousehold::AdverseSecondOnly {
                primary_location: primary,
                second_location: second,
                confirmed,
            };
            stays[0].location = stay;
            let mut r = result();
            check_households([Some(&invalid), None], false, &mut r);
            assert!(
                r.issues
                    .iter()
                    .any(|issue| issue.display.key == "ISSUE_USE_ONE_HOUSEHOLD_CLAIMANT_INCLUDING_A_CONFIRMED_ADVERSE_CONDITION_7185B7A1")
            );
        }
        let mut r = result();
        check_households([Some(&claim), Some(&claim)], false, &mut r);
        assert!(
            r.issues
                .iter()
                .any(|issue| issue.display.key == "ISSUE_USE_ONE_HOUSEHOLD_CLAIMANT_INCLUDING_A_CONFIRMED_ADVERSE_CONDITION_7185B7A1")
        );
    }

    #[test]
    fn adverse_second_household_rejects_unresolved_location_or_duplicate_claim() {
        for (primary, second, confirmed) in [
            (ForeignHousingLocation::ChinaHongKong, ForeignHousingLocation::AustraliaSydney, true),
            (ForeignHousingLocation::General, ForeignHousingLocation::General, true),
            (ForeignHousingLocation::ChinaHongKong, ForeignHousingLocation::ChinaHongKong, false),
        ] {
            let claim = adverse_claim(primary, second, confirmed);
            let mut r = result();
            check_households([Some(&claim), None], false, &mut r);
            assert!(
                r.issues
                    .iter()
                    .any(|issue| issue.display.key == "ISSUE_USE_ONE_HOUSEHOLD_CLAIMANT_INCLUDING_A_CONFIRMED_ADVERSE_CONDITION_7185B7A1")
            );
        }
        let claim = adverse_claim(
            ForeignHousingLocation::ChinaHongKong,
            ForeignHousingLocation::ChinaHongKong,
            true,
        );
        let mut r = result();
        check_households([Some(&claim), Some(&claim)], false, &mut r);
        assert!(
            r.issues
                .iter()
                .any(|issue| issue.display.key == "ISSUE_USE_ONE_HOUSEHOLD_CLAIMANT_INCLUDING_A_CONFIRMED_ADVERSE_CONDITION_7185B7A1")
        );
    }
}
