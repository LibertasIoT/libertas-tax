// Coverage: Form 1116 and Schedule B (Form 1116) — Supported for the ordinary claim (unit b9-6,
//   2026-10-10; 2026 drafts of Form 1116 (created 7/1/26), its instructions (9/28/26) and
//   Schedule B (Rev. 12-2026), Instructions for Schedule B Rev. 12-2022, Pub. 514 (2025), IRC
//   901, 904, 905, 909, 26 CFR 1.861-8, 1.861-9T, 1.904-3, 1.904-4, 1.904(b)-1, 1.989(a)-1). One
//   typed Form 1116 for each category (general, passive, foreign branch, section 901(j)) and tax
//   system (regular, AMT) with a column for each country (IRS country codes, "RIC"), the HTKO
//   column of high-taxed passive income and the Total column; one Schedule B for each category
//   and system with the origin years 2016 to 2025 as columns. Every printed line is emitted with
//   its form line (TaxForm::Form1116, TaxForm::Form1116ScheduleB, LineKind::Ratio for lines 3f
//   and 19). Categories: employee pay, earlier-year wage and passive tax rows and a business run
//   abroad with no separate books are general; interest, dividends, royalties and foreign-source
//   sales are passive; a business run abroad with its own books is foreign branch; a row whose
//   country is on the section 901(j) list is category e, completed through line 17 with no
//   credit. Schedule K-3 rows join the column of their country, category by category, including
//   column (b) foreign branch and its capital lines 11, 12, 27 and 28. The qualified dividend and
//   capital gain adjustments are foreign_capital.rs. Allocation facts the person cannot know
//   take the conservative path with one note each (WARN_FTC_*_ASSUMED); a mix the App does not
//   figure leaves the credit out (WARN_FTC_CAPITAL_RATE_GROUPS_LEFT_OUT, WARN_FTC_LINE_18_LEFT_OUT,
//   WARN_FOREIGN_SITUATION_LEFT_OUT). Not calculated: section 951A (a), treaty re-sourced income
//   (f), lump sums (g), PTEP, section 962, Form 1116 line 22, Schedule C (Form 1116)
//   redeterminations beyond Schedule B line 2b, contested taxes, the provisional credit.
//   Evidence: form_1116.rs, foreign_capital.rs, foreign_credit_history.rs, foreign_losses.rs,
//   foreign_loss_history.rs, foreign_investment_assets.rs, foreign_state_tax.rs, schedule_k3.rs,
//   extended_rules.rs (foreign_credit), federal_rules.rs (CreditBasis assembly); tests/form_1116.rs
//   form_1116_f1 .. form_1116_f5, tests/foreign.rs.
//   F18 (2026-10-12): line3e replaces the property ordinary/long-term net
//   with positive recognized source gains, before losses, independently for
//   regular/AMT. Source risk/PAL nets determine only the amount being replaced;
//   suspended losses are never restored. K-3 and Schedule D footing retain
//   their existing owners. Evidence: tests/fix_f18.rs D1 (320160/328160,
//   independent AMT318943 and allowed/suspended rental140000/162000);
//   IRS i1116 "Lines 3d and 3e"; asset_sale/section1231 source projections.
#![allow(clippy::needless_range_loop, clippy::type_complexity)]
use crate::foreign_capital::{
    AMT_FACTORS, CapitalInput, Outcome, REGULAR_FACTORS, RateBands, RateFactors,
};
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::line_emission::emit as emit_lines;
use crate::line_emission::{form_line, form_ratio, line};
use crate::{
    FederalDraft, FederalIncome, FederalResult, ForeignAmtLimitation, ForeignCarryback,
    ForeignCreditCategory, ForeignCreditHistory, ForeignInterestAllocation,
    ForeignInvestmentAssets, ForeignLossHistory, ForeignOrdinaryTax, ForeignRateAdjustment,
    ForeignStateTaxes, ForeignTaxBasis, ForeignTaxCategory, FormLine, Message, Owner, TaxForm,
};
use alloc::{format, string::String, vec, vec::Vec};
use core::cmp::min;

const SOURCE: &str = "https://www.irs.gov/instructions/i1116";
const AMT_SOURCE: &str = "https://www.irs.gov/instructions/i6251";

// Groups: the unit income is classified and high-tax tested in. 0 is the
// general category, 1-4 the passive withholding groups (1.904-4(c)(3)), 5 the
// foreign branch category, 6 section 901(j) income; DOMESTIC is U.S. income in
// the allocation arrays.
pub(crate) const GENERAL: usize = 0;
pub(crate) const PASSIVE_GROUPS: core::ops::Range<usize> = 1..5;
pub(crate) use crate::foreign_source_basics::{BRANCH, SANCTIONED};
pub(crate) const DOMESTIC: usize = 7;
pub(crate) const SLOTS: usize = 8;
const FOREIGN_GROUPS: usize = 7;

// The category (and loss-account) slot of a group: general, passive, foreign
// branch, section 901(j).
fn slot_of(group: usize) -> usize {
    match group {
        GENERAL => 0,
        BRANCH => 2,
        SANCTIONED => 3,
        _ => 1,
    }
}
const NAMES: [&str; 4] = crate::foreign_losses::CATEGORY_NAMES;

// ---- The records ----

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ColumnKey {
    Country(String),
    Htko,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Column {
    pub key: Option<ColumnKey>,
    pub line_1a: i64,
    pub line_2: i64,
    pub line_3a: i64,
    pub line_3b: i64,
    pub line_3d: i64,
    pub line_3e: i64,
    pub line_3g: i64,
    pub line_4a: i64,
    pub line_4b: i64,
    pub line_5: i64,
    // The HTKO column carries only lines 1a and 6 (i1116 "line i").
    pub line_6_override: Option<i64>,
}

impl Column {
    pub(crate) fn line_3c(&self) -> i64 {
        self.line_3a + self.line_3b
    }
    pub(crate) fn line_6(&self) -> i64 {
        self.line_6_override
            .unwrap_or(self.line_2 + self.line_3g + self.line_4a + self.line_4b + self.line_5)
    }
    pub(crate) fn line_7(&self) -> i64 {
        self.line_1a - self.line_6()
    }
    // Line 3f in ten-thousandths: line 3d over line 3e, rounded to four
    // places, at most 1.
    pub(crate) fn line_3f(&self) -> i64 {
        if self.line_3e <= 0 || self.line_3d <= 0 {
            0
        } else {
            min(10000, fraction(10000, self.line_3d, self.line_3e))
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Form1116 {
    // 0 general (box d), 1 passive (c), 2 foreign branch (b), 3 section
    // 901(j) (e).
    pub slot: usize,
    pub amt: bool,
    pub columns: Vec<Column>,
    // Part II column (w): the taxes of each country, whole dollars.
    pub taxes: Vec<(String, i64)>,
    pub line_10: i64,
    // Negative: the reduction of line 12.
    pub line_12: i64,
    // Signed: minus the high-taxed passive tax in the passive record, plus it
    // in the record the income moved to.
    pub line_13: i64,
    pub line_16: i64,
    pub line_18: i64,
    pub line_20: i64,
    pub line_22: i64,
}

impl Form1116 {
    pub(crate) fn total_1a(&self) -> i64 {
        self.columns.iter().map(|c| c.line_1a).sum()
    }
    pub(crate) fn total_6(&self) -> i64 {
        self.columns.iter().map(|c| c.line_6()).sum()
    }
    pub(crate) fn total_7(&self) -> i64 {
        self.total_1a() - self.total_6()
    }
    pub(crate) fn line_8(&self) -> i64 {
        self.taxes.iter().map(|t| t.1).sum()
    }
    pub(crate) fn line_11(&self) -> i64 {
        self.line_8() + self.line_10
    }
    pub(crate) fn line_14(&self) -> i64 {
        self.line_11() + self.line_12 + self.line_13
    }
    pub(crate) fn line_15(&self) -> i64 {
        self.total_7()
    }
    pub(crate) fn line_17(&self) -> i64 {
        self.line_15() + self.line_16
    }
    // Line 19 in ten-thousandths, at most 1; zero when line 18 is zero.
    pub(crate) fn line_19(&self) -> i64 {
        if self.line_18 <= 0 || self.line_17() <= 0 {
            0
        } else {
            min(10000, fraction(10000, self.line_17(), self.line_18))
        }
    }
    pub(crate) fn line_21(&self) -> i64 {
        if self.line_18 <= 0 || self.line_17() <= 0 {
            0
        } else {
            fraction(self.line_20, phase::cap(self.line_17(), [self.line_18]), self.line_18)
        }
    }
    pub(crate) fn line_23(&self) -> i64 {
        self.line_21() + self.line_22
    }
    // The credit of the category: no credit for section 901(j) income.
    pub(crate) fn line_24(&self) -> i64 {
        if self.slot == 3 { 0 } else { positive(phase::cap(self.line_14(), [self.line_23()])) }
    }
}

// Schedule B: one column of origin year (2016 through 2025) or the 2026
// column; lines 3 and 8 and the column (xiv) totals are derived.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ScheduleBColumn {
    pub year: u32,
    pub line_1: i64,
    pub line_2a: i64,
    pub line_2b: i64,
    pub line_2c: i64,
    pub line_4: i64,
    pub line_5: i64,
    pub line_6: i64,
    pub line_7: i64,
}

impl ScheduleBColumn {
    pub(crate) fn line_3(&self) -> i64 {
        self.line_1 + self.line_2a + self.line_2b + self.line_2c
    }
    pub(crate) fn line_8(&self) -> i64 {
        self.line_3() - self.line_4 - self.line_5 + self.line_6 - self.line_7
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScheduleB {
    pub kind: ForeignCreditCategory,
    pub amt: bool,
    pub columns: Vec<ScheduleBColumn>,
    pub deduction_year: bool,
}

#[derive(Default)]
pub(crate) struct Form1116Return {
    pub forms: Vec<Form1116>,
    pub schedules: Vec<ScheduleB>,
}

// ---- The basis the return supplies ----

// One Schedule C or F business of the return, whole dollars unless marked.
#[derive(Clone)]
pub(crate) struct BusinessFacts {
    pub id: i64,
    pub owner: Owner,
    // Run abroad (a foreign services business) or not.
    pub abroad: bool,
    // Schedule C line 7 (receipts less cost of goods sold).
    pub gross: i64,
    // Schedule C line 7 less line 31: the expenses with the interest.
    pub expenses: i64,
    // Schedule C lines 16a and 16b.
    pub interest: i64,
    pub half_se: i64,
    // The owner's SEP, SIMPLE or qualified plan contribution and self-employed
    // health insurance deduction attributed to this business.
    pub plans: i64,
    // The net loss, positive, when the business lost money.
    pub loss: i64,
    // Receipts in cents, for the Form 2555 split.
    pub receipts: i64,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct BusinessExclusionFacts {
    pub business_gross: i64,
    pub related: i64,
    // The self-employed housing deduction (Form 2555 line 50).
    pub housing_deduction: i64,
}

#[derive(Clone)]
pub(crate) struct CreditBasis {
    // Form 1116 line 20: Form 1040 line 16 with the Form 8814 tax and Schedule
    // 2 line 1z, less the Form 4972 tax, decreased by a negative Form 8978
    // line 14, never below zero.
    pub regular: i64,
    // Form 6251 line 7.
    pub tentative: i64,
    // Line 18, excluding the enhanced senior deduction.
    pub worldwide: i64,
    pub agi: i64,
    // AMT base before the married-separate income addition.
    pub amt_base: i64,
    pub deduction: i64,
    // Deducted section 901(j) tax, definitely related to sanctioned income.
    pub foreign_tax_deduction: i64,
    pub sanctioned_taxes: alloc::collections::BTreeMap<String, i64>,
    pub ira_deduction: i64,
    // Deductions retained for AMT before allocating them to source income.
    pub amt_deduction: i64,
    pub mortgage: [i64; 2],
    pub investment: [i64; 2],
    pub charity: [i64; 2],
    pub state_paid: i64,
    pub state_deduction: i64,
    // Form 1116 line 3b: Schedule 1 Part II adjustments other than the IRA
    // deduction, student loan interest and `us_adjustments`.
    pub adjustments: i64,
    // Schedule 1 Part II adjustments definitely related to U.S. income.
    pub us_adjustments: i64,
    pub savings_penalty: i64,
    pub penalty_interest: i64,
    pub student_loan: i64,
    pub vehicle_interest: i64,
    // QBI and Schedule 1-A deductions (enhanced senior excluded).
    pub below_line: i64,
    // Line 3e gross income less the net amounts in AGI: U.S. gross income of
    // businesses, farms, rents, royalties and personal property over their
    // nets, plus section 1244 ordinary losses on domestic stock.
    pub gross_correction: i64,
    pub k3_correction: i64,
    // What each Schedule K-3's K-1 allows this year.
    pub k3_limits: Vec<crate::schedule_k3::K3Limits>,
    pub foreign_wages: [i64; 2],
    // Form 2555 line 45 for each person.
    pub excluded_income: [i64; 2],
    pub housing_deduction: [i64; 2],
    pub portfolio: i64,
    pub taxable: i64,
    pub qualified: i64,
    // The capped preference triple the return's tax worksheet used; regular,
    // then AMT.
    pub preference: [[i64; 3]; 2],
    // Gains before losses of every Schedule D and Form 4797 contributor, and
    // the net Schedule D amount already in AGI; AMT gross separately.
    pub capital_gross: [i64; 2],
    pub amt_capital_gross: i64,
    // Gross property gains less the ordinary/long-term projection already
    // restored below, independently for regular tax and AMT.
    pub property_gross_correction: [i64; 2],
    // The taxable state refund (not gross income for AMT) and the income-type
    // Form 6251 adjustments (incentive stock options, private activity bond
    // interest, AMT gain differences).
    pub tax_refund: i64,
    pub amt_income_adjustments: i64,
    // Personal casualty and theft losses and the disaster increase: line 3b.
    pub casualty: [i64; 2],
    // Itemized deductions definitely related to U.S. income, reduced.
    pub us_itemized: i64,
    // Itemized educator expenses (line 2 by the share of foreign employee pay).
    pub educator: i64,
    pub employee_pay: i64,
    pub nol_deduction: i64,
    pub excess_business_loss: i64,
    // Schedule D, regular then AMT.
    pub schedule_d: [crate::foreign_capital::ScheduleD; 2],
    // Form 4952 line 4g: [net capital gain, qualified dividends].
    pub elected: [i64; 2],
    pub businesses: Vec<BusinessFacts>,
    pub business_exclusions: [BusinessExclusionFacts; 2],
    // Test whether the return's tax came from the Qualified Dividends and
    // Capital Gain Tax Worksheet or the Schedule D Tax Worksheet (Form 8615
    // and Schedule J tax cannot be mapped to the Worksheet for Line 18).
    pub line_18_mapped: bool,
}

// ---- The claim and what it gathers ----

pub(crate) struct Claim<'a> {
    pub basis: &'a ForeignTaxBasis,
    pub sources: &'a [ForeignOrdinaryTax],
    pub history: &'a ForeignCreditHistory,
    pub carryback: &'a ForeignCarryback,
    pub amt_limitation: &'a ForeignAmtLimitation,
    pub interest_allocation: &'a ForeignInterestAllocation,
    pub investment_assets: &'a ForeignInvestmentAssets,
    pub loss_history: &'a ForeignLossHistory,
    pub state_taxes: &'a ForeignStateTaxes,
    pub rate_adjustment: &'a ForeignRateAdjustment,
}

// What one country column of one group holds before any allocation: cents
// unless marked.
#[derive(Clone, Default)]
struct Cell {
    gross: i64,
    tax: i64,
    reduction: i64,
    qualified: i64,
    // Whole dollars taken out of line 1a (Form 2555) and expenses assigned to
    // the column directly (a business run abroad).
    excluded: i64,
    line_2: i64,
    line_4b: i64,
    // Current tax source Cents after the same native reductions. These are
    // captured before country merging, never inferred from foreign income.
    tax_owners: [i64; 2],
    tax_unknown: i64,
}

#[derive(Default)]
struct Gathered {
    cells: alloc::collections::BTreeMap<(usize, String), Cell>,
    // Foreign capital gains and losses by group and term (short, long), cents.
    cap_gain: [[i64; 2]; FOREIGN_GROUPS],
    cap_loss: [[i64; 2]; FOREIGN_GROUPS],
    // Whole dollars of employee pay by owner and of passive row income.
    employment: [i64; 2],
    passive_income: i64,
    // Interest of every Schedule C business, given to the businesses run
    // abroad (conservative), and whether there are any.
    business_interest: i64,
    foreign_business_rows: bool,
    // Rows that name a business or a sale this return no longer has.
    stale: bool,
}

fn code_of(country: &str) -> String {
    country.trim().to_ascii_uppercase()
}

use crate::kernel::ratio::allocate_prefix as spread;

fn owner_index(owner: Owner) -> usize {
    usize::from(owner == Owner::Spouse)
}

// Pub. 514, Foreign Earned Income and Housing Exclusions: both credit and
// deduction remove the tax on net excluded earnings. Inputs are the owner's
// actual Form 2555 line 45 and net foreign earnings, not the gross exclusion.
pub(crate) fn excluded_earned_tax(tax: i64, excluded: i64, income: i64) -> i64 {
    if excluded > 0 && income > 0 {
        fraction(tax, phase::cap(excluded, [income]), income)
    } else {
        0
    }
}

fn gather(
    d: &FederalDraft,
    claim: &Claim<'_>,
    basis: &CreditBasis,
    k3: &crate::schedule_k3::K3Credit,
    sources: &[crate::estate_foreign::ForeignCatalogSource],
    small_interest: bool,
    r: &mut FederalResult,
) -> Option<Gathered> {
    let mut g = Gathered::default();
    let mut passive_cents = 0_i64;
    // The businesses run abroad of each owner, for the Form 2555 split.
    let abroad: Vec<&BusinessFacts> = basis.businesses.iter().filter(|b| b.abroad).collect();
    g.business_interest = basis.businesses.iter().map(|b| b.interest).sum();
    let mut business_cells: Vec<((usize, String), i64, usize, i64)> = Vec::new();
    // Each employment row by owner: its column, income and tax (cents).
    let mut employment_cells: [Vec<((usize, String), i64, i64)>; 2] = [Vec::new(), Vec::new()];
    for source in sources {
        // An earlier year's tax counts only when taxes are claimed when paid
        // (FOREIGN_PRIOR_TAX_NEEDS_PAID_BASIS keeps the page from saving one).
        if *claim.basis == ForeignTaxBasis::Accrued
            && matches!(
                source.category,
                ForeignTaxCategory::PriorEmployeeWageTax { .. }
                    | ForeignTaxCategory::PriorPassiveTax { .. }
            )
        {
            continue;
        }
        // Check the actual passive pool before updating any country cell.
        if matches!(source.category, ForeignTaxCategory::Passive { .. }) {
            let Some(total) = passive_cents.checked_add(source.income) else {
                crate::issue_localized!(
                    r,
                    &section(),
                    "Foreign passive gross income",
                    "FOREIGN_PASSIVE_GROSS_UNAVAILABLE",
                    "The required Form 1116 passive gross income cannot be represented in its native amount. No final tax can be reported.",
                );
                return None;
            };
            passive_cents = total;
        }
        let group = source.group();
        let key = (group, code_of(&source.country));
        let cell = g.cells.entry(key.clone()).or_default();
        cell.gross += source.income;
        cell.tax += source.tax;
        cell.tax_owners[owner_index(source.owner)] += source.tax;
        cell.qualified += source.qualified();
        match &source.category {
            ForeignTaxCategory::Employment => {
                g.employment[owner_index(source.owner)] += dollars(source.income);
                employment_cells[owner_index(source.owner)].push((key, source.income, source.tax));
            }
            ForeignTaxCategory::PriorEmployeeWageTax { .. }
            | ForeignTaxCategory::PriorPassiveTax { .. } => {}
            ForeignTaxCategory::Passive { sales, .. } => {
                for id in sales {
                    let found = d.income.iter().find(|entry| entry.id == *id).and_then(|entry| {
                        crate::broker_calculation::sale(&entry.income).map(|sale| (entry, sale))
                    });
                    let Some((entry, sale)) = found else {
                        g.stale = true;
                        continue;
                    };
                    let gain = if matches!(entry.income, FederalIncome::Broker { .. }) {
                        crate::community::income_amount(d, entry.id, sale.gain())
                    } else {
                        sale.gain()
                    };
                    let term = usize::from(sale.term != crate::GainTerm::Short);
                    let next = if gain > 0 {
                        g.cap_gain[group][term]
                            .checked_add(gain)
                            .zip(g.cells.get(&key).unwrap().gross.checked_add(gain))
                    } else {
                        gain.checked_neg()
                            .and_then(|loss| g.cap_loss[group][term].checked_add(loss))
                            .map(|loss| (loss, 0))
                    };
                    let Some((amount, gross)) = next else {
                        crate::issue_localized!(
                            r,
                            &section(),
                            "Foreign passive gross income",
                            "FOREIGN_PASSIVE_GROSS_UNAVAILABLE",
                            "The required Form 1116 passive gross income cannot be represented in its native amount. No final tax can be reported.",
                        );
                        return None;
                    };
                    if gain > 0 {
                        g.cap_gain[group][term] = amount;
                        g.cells.get_mut(&key).unwrap().gross = gross;
                    } else {
                        g.cap_loss[group][term] = amount;
                    }
                }
            }
            ForeignTaxCategory::Business { business, separate_books: _ } => {
                g.foreign_business_rows = true;
                let Some(facts) = basis
                    .businesses
                    .iter()
                    .find(|b| b.id == *business && b.abroad && b.owner == source.owner)
                else {
                    g.stale = true;
                    continue;
                };
                let owner = owner_index(facts.owner);
                let weights: Vec<i64> = abroad
                    .iter()
                    .filter(|b| owner_index(b.owner) == owner)
                    .map(|b| b.receipts)
                    .collect();
                let position = abroad
                    .iter()
                    .filter(|b| owner_index(b.owner) == owner)
                    .position(|b| b.id == facts.id)
                    .unwrap_or(0);
                let exclusion = basis.business_exclusions[owner];
                let excluded =
                    crate::kernel::ratio::allocate_remaining(exclusion.business_gross, &weights)
                        [position];
                let expense_weights: Vec<i64> = abroad
                    .iter()
                    .filter(|b| owner_index(b.owner) == owner)
                    .map(|b| b.expenses + b.half_se)
                    .collect();
                let related =
                    crate::kernel::ratio::allocate_remaining(exclusion.related, &expense_weights)
                        [position];
                let plans = facts.plans;
                let cell = g.cells.get_mut(&key).unwrap();
                cell.gross += facts.gross * 100;
                cell.excluded += excluded;
                cell.line_2 +=
                    positive(facts.expenses - facts.interest) + facts.half_se + plans - related;
                business_cells.push((key.clone(), facts.gross, owner, source.tax));
            }
        }
    }
    // The business interest of every Schedule C business, given to the
    // businesses run abroad by their gross income (Reg. 1.861-9T(d)(1)(i)).
    if !small_interest && g.business_interest > 0 && !business_cells.is_empty() {
        let weights: Vec<i64> = business_cells.iter().map(|c| c.1).collect();
        for ((key, _, _, _), share) in
            business_cells.iter().zip(spread(g.business_interest, &weights))
        {
            g.cells.get_mut(key).unwrap().line_4b += share;
        }
    }
    // Schedule K-3 rows join the column of their country.
    for row in &k3.rows {
        let cell = g.cells.entry((row.group, code_of(&row.country))).or_default();
        cell.gross += row.income;
        cell.tax += row.tax;
        cell.reduction += row.reduction;
        if let Some((_, owner)) = row.source {
            cell.tax_owners[owner_index(owner)] += row.tax - row.reduction;
        } else {
            cell.tax_unknown += row.tax - row.reduction;
        }
        cell.qualified += row.qualified;
    }
    for group in 0..FOREIGN_GROUPS {
        for term in 0..2 {
            g.cap_gain[group][term] += k3.capital[group][term];
            g.cap_loss[group][term] += k3.capital[group][2 + term];
        }
    }
    // Form 2555: the employee pay excluded comes off line 1a of the person's
    // employment rows (line 45 less the businesses' share), and the foreign
    // income tax on excluded earned income comes off the rows' tax (line 12).
    for owner in 0..2 {
        let excluded = positive(
            basis.excluded_income[owner] + basis.business_exclusions[owner].related
                - basis.business_exclusions[owner].business_gross,
        );
        let rows = &employment_cells[owner];
        if excluded > 0 && !rows.is_empty() {
            let weights: Vec<i64> = rows.iter().map(|row| row.1).collect();
            for (row, share) in rows.iter().zip(spread(excluded, &weights)) {
                g.cells.get_mut(&row.0).unwrap().excluded += share;
            }
        }
        let earned_net: i64 = basis
            .businesses
            .iter()
            .filter(|b| b.abroad && owner_index(b.owner) == owner)
            .map(|b| b.gross - b.expenses - b.half_se)
            .sum::<i64>();
        let income = basis.foreign_wages[owner] + positive(earned_net)
            - basis.housing_deduction[owner]
            - basis.business_exclusions[owner].housing_deduction;
        let excluded_total = basis.excluded_income[owner];
        if excluded_total > 0 && income > 0 {
            // The tax on the person's employment rows; the businesses' rows
            // follow, each by its own tax.
            let employment_tax: i64 = rows.iter().map(|row| row.2).sum();
            let reduction = excluded_earned_tax(employment_tax, excluded_total, income);
            if reduction > 0 {
                let weights: Vec<i64> = rows.iter().map(|row| row.2).collect();
                for (row, share) in rows.iter().zip(spread(reduction, &weights)) {
                    let cell = g.cells.get_mut(&row.0).unwrap();
                    cell.reduction += share;
                    cell.tax_owners[owner] -= share;
                }
            }
            for (key, _, business_owner, tax) in &business_cells {
                if *business_owner == owner {
                    let reduction = excluded_earned_tax(*tax, excluded_total, income);
                    let cell = g.cells.get_mut(key).unwrap();
                    cell.reduction += reduction;
                    cell.tax_owners[owner] -= reduction;
                }
            }
        }
    }
    // Reconcile actual source cents against the income owner before rounding.
    g.passive_income = dollars(passive_cents);
    Some(g)
}

// ---- One tax system's allocation, kickout and limits ----

// The sums of the cells by group, whole dollars.
#[derive(Default)]
struct Groups {
    // Line 3d: gross foreign income before the rate adjustments, with the
    // Form 2555 excluded income.
    gross: [i64; FOREIGN_GROUPS],
    // Taken out of line 1a (Form 2555).
    excluded: [i64; FOREIGN_GROUPS],
    // The taxes as the rows show them, and after the line 12 reductions.
    tax_gross: [i64; FOREIGN_GROUPS],
    tax_net: [i64; FOREIGN_GROUPS],
    tax_owners: [Option<[i64; 2]>; FOREIGN_GROUPS],
    qualified: [i64; FOREIGN_GROUPS],
    line_2: [i64; FOREIGN_GROUPS],
    line_4b: [i64; FOREIGN_GROUPS],
    // Foreign capital gains and losses (short, long).
    cap_gain: [[i64; 2]; FOREIGN_GROUPS],
    cap_loss: [[i64; 2]; FOREIGN_GROUPS],
}

fn groups_of(g: &Gathered, first: usize) -> Groups {
    let mut out = Groups::default();
    let mut cents = [[0_i64; 5]; FOREIGN_GROUPS];
    let mut owners = [[0_i64; 2]; FOREIGN_GROUPS];
    let mut unknown = [0_i64; FOREIGN_GROUPS];
    for ((group, _), cell) in &g.cells {
        cents[*group][0] += cell.gross;
        cents[*group][1] += cell.tax;
        cents[*group][2] += cell.tax - cell.reduction;
        cents[*group][3] += cell.qualified;
        for owner in 0..2 {
            owners[*group][owner] += cell.tax_owners[owner];
        }
        unknown[*group] += cell.tax_unknown;
        out.excluded[*group] += cell.excluded;
        out.line_2[*group] += cell.line_2;
        out.line_4b[*group] += cell.line_4b;
    }
    for group in 0..FOREIGN_GROUPS {
        out.gross[group] = dollars(cents[group][0]);
        out.tax_gross[group] = dollars(cents[group][1]);
        out.tax_net[group] = dollars(cents[group][2]);
        let complete = owners[group].iter().all(|amount| *amount >= 0)
            && unknown[group] >= 0
            && owners[group][0]
                .checked_add(owners[group][1])
                .and_then(|sum| sum.checked_add(unknown[group]))
                == Some(cents[group][2]);
        out.tax_owners[group] = if complete && out.tax_net[group] == 0 {
            // Closed nonnegative native group: even an unowned source below
            // its rounding boundary contributes no reported tax capacity.
            Some([0; 2])
        } else if complete && unknown[group] == 0 {
            crate::kernel::ratio::project_rounded_pair(out.tax_net[group], owners[group], first)
        } else {
            None
        };
        out.qualified[group] = dollars(cents[group][3]);
        for term in 0..2 {
            out.cap_gain[group][term] = dollars(g.cap_gain[group][term]);
            out.cap_loss[group][term] = dollars(g.cap_loss[group][term]);
        }
    }
    out
}

struct Ctx<'a> {
    cells: [&'a alloc::collections::BTreeMap<(usize, String), Cell>; 2],
    claim: &'a Claim<'a>,
    basis: &'a CreditBasis,
    k3: &'a crate::schedule_k3::K3Credit,
    status: crate::FederalStatus,
    small_interest: bool,
    groups: [Groups; 2],
    // Form 2555 excluded gross income of both people.
    excluded_total: i64,
    k3_partner: i64,
    state_expenses: [i64; SLOTS],
    investment: [[i64; SLOTS]; 2],
    // The early withdrawal penalty on possibly foreign interest, the student
    // and vehicle loan interest and the foreign share of itemized educator
    // expenses (whole dollars) before they are given to the groups.
    penalty: i64,
    loan_interest: i64,
    educator: i64,
    amt_rate: i64,
    foreign_qualified: [i64; 2],
    // Form 1116 line 16 adjustment 1: the section 461(l) disallowed loss added
    // back to the category of the business run abroad that lost it.
    excess_loss: [i64; 4],
}

// One system's results.
#[derive(Default)]
struct SystemOut {
    amt: bool,
    gross_all: i64,
    line_3a: i64,
    line_3b: i64,
    left_out: bool,
    // Per group: line 1a before and after the rate adjustments, lines 2, 3g,
    // 4a, 4b, the unadjusted and adjusted capital losses, and the net income
    // after the passive loss reallocation, before and after adjustments.
    l1a_pre: [i64; FOREIGN_GROUPS],
    l1a_fin: [i64; FOREIGN_GROUPS],
    l2: [i64; FOREIGN_GROUPS],
    l3g: [i64; FOREIGN_GROUPS],
    l4a: [i64; FOREIGN_GROUPS],
    l4b: [i64; FOREIGN_GROUPS],
    loss_pre: [i64; FOREIGN_GROUPS],
    l5_fin: [i64; FOREIGN_GROUPS],
    net_re: [i64; FOREIGN_GROUPS],
    net_fin: [i64; FOREIGN_GROUPS],
    high: [bool; FOREIGN_GROUPS],
    // By category slot: line 15 (as used), line 17, the taxes after the
    // reductions and the high-tax moves (lines 9 + 12 + 13), line 23.
    income: [i64; 4],
    line_17: [i64; 4],
    taxes: [i64; 4],
    tax_owners: [Option<[i64; 2]>; 4],
    limit: [i64; 4],
    kicked_taxes: i64,
    line_18: i64,
    line_18_reduction: i64,
    adjusted_dividends: i64,
    capital_reduction: i64,
    worksheet_lines: Vec<(&'static str, i64)>,
    created_loss: bool,
    amt_loss_created: bool,
    domestic: i64,
}

impl Ctx<'_> {
    fn system(
        &self,
        amt: bool,
        regular: Option<&SystemOut>,
        simplified: bool,
        r: &mut FederalResult,
    ) -> SystemOut {
        let basis = self.basis;
        let s = usize::from(amt);
        let mut out = SystemOut { amt, ..SystemOut::default() };
        let groups = &self.groups[s];
        let system = if amt { "amt" } else { "regular" };
        // ---- Line 3e and the deductions to apportion ----
        let restored = basis.ira_deduction
            + basis.adjustments
            + basis.us_adjustments
            + basis.student_loan
            + basis.gross_correction
            + basis.k3_correction
            + basis.capital_gross[0]
            - basis.capital_gross[1]
            + basis.excluded_income.iter().sum::<i64>()
            + basis.nol_deduction
            - basis.excess_business_loss;
        let gross_regular = basis.agi + restored + basis.property_gross_correction[0];
        out.gross_all = if amt {
            gross_regular - basis.tax_refund
                + basis.amt_income_adjustments
                + basis.amt_capital_gross
                - basis.capital_gross[0]
                + basis.property_gross_correction[1]
                - basis.property_gross_correction[0]
        } else {
            gross_regular
        };
        let charity_and_interest =
            basis.charity[s] + if self.small_interest { basis.mortgage[s] } else { 0 };
        let domestic = charity_and_interest
            + self.state_expenses[DOMESTIC] * i64::from(!amt)
            + self.investment[s][DOMESTIC]
            + if amt { 0 } else { basis.us_itemized };
        let foreign_state: i64 =
            if amt { 0 } else { self.state_expenses[..FOREIGN_GROUPS].iter().sum() };
        let foreign_investment: i64 = self.investment[s][..FOREIGN_GROUPS].iter().sum();
        let mortgage = if self.small_interest { 0 } else { basis.mortgage[s] };
        let deduction = if amt { basis.amt_deduction } else { basis.deduction };
        let apportioned = deduction + basis.ira_deduction + basis.adjustments + self.k3_partner
            - domestic
            - foreign_state
            - foreign_investment
            - mortgage
            - if amt { 0 } else { basis.foreign_tax_deduction }
            - if amt { 0 } else { basis.educator };
        out.domestic = domestic;
        out.line_3b = basis.ira_deduction + basis.adjustments + self.k3_partner + basis.casualty[s];
        out.line_3a = apportioned - out.line_3b;
        let excluded_gross = self.excluded_total;
        let worksheet_den = out.gross_all - excluded_gross;
        let any_excluded = excluded_gross != 0;
        // ---- Per group: lines 1a, 2, 3g, 4a, 4b and the unadjusted net ----
        let passive_gross: i64 = PASSIVE_GROUPS.map(|g| groups.gross[g]).sum();
        let foreign_gross: i64 = groups.gross.iter().sum();
        let loan = self.loan_interest;
        let mut net_pre = [0_i64; FOREIGN_GROUPS];
        for g in 0..FOREIGN_GROUPS {
            out.l1a_pre[g] = groups.gross[g] - groups.excluded[g];
            out.loss_pre[g] = groups.cap_loss[g].iter().sum();
            let share = |amount: i64, weight: i64, total: i64| {
                if total > 0 { fraction(amount, weight, total) } else { 0 }
            };
            out.l4a[g] = share(mortgage, groups.gross[g] - groups.excluded[g], worksheet_den);
            out.l3g[g] = if any_excluded {
                share(apportioned, groups.gross[g], out.gross_all)
            } else {
                share(apportioned + mortgage, groups.gross[g], out.gross_all) - out.l4a[g]
            };
            let penalty = if PASSIVE_GROUPS.contains(&g) {
                share(self.penalty, groups.gross[g], passive_gross)
            } else {
                0
            };
            out.l2[g] = groups.line_2[g]
                + dollars(self.k3.deductions[g])
                + penalty
                + if g == GENERAL && !amt { self.educator } else { 0 }
                + if amt { 0 } else { self.state_expenses[g] }
                + if g == SANCTIONED && !amt { basis.foreign_tax_deduction } else { 0 };
            out.l4b[g] = groups.line_4b[g]
                + dollars(self.k3.interest[g])
                + self.investment[s][g]
                + share(loan, groups.gross[g], foreign_gross);
            net_pre[g] = out.l1a_pre[g]
                - out.loss_pre[g]
                - (out.l2[g] + out.l3g[g] + out.l4a[g] + out.l4b[g]);
        }
        // ---- Passive losses reduce the other passive groups first ----
        let mut net_re = net_pre;
        let mut five: [i64; 5] = core::array::from_fn(|i| net_re[i]);
        let passive_loss = crate::foreign_losses::passive_groups(&mut five, system, r);
        for (i, v) in five.iter().enumerate() {
            net_re[i] = *v;
        }
        // ---- High-tax kickout (1.904-4(c)) ----
        let rate = if amt { self.amt_rate } else { 37 };
        let mut kicked = 0;
        for g in PASSIVE_GROUPS {
            let tax = groups.tax_net[g];
            out.high[g] = tax * 100 > positive(net_re[g]) * rate;
            if out.high[g] {
                kicked += tax;
            }
        }
        out.kicked_taxes = kicked;
        let landing = |g: usize| if out.high[g] { 0 } else { slot_of(g) };
        // ---- Rate bands and the adjustment exception ----
        let rates = crate::income_tax::AmtRates::for_year(2026, self.status).unwrap();
        let (mut regular_bands, [line5, line14, line21]) =
            RateBands::regular(basis.taxable, basis.preference[0], self.status);
        let (mut amt_bands, line17) =
            RateBands::amt(basis.amt_base, basis.preference[1], [line14, line21], self.status);
        let top_24 = crate::income_tax::Rates::for_year(2026, self.status).unwrap().ordinary[3];
        let foreign_long: i64 = groups.cap_gain.iter().map(|v| v[1]).sum::<i64>()
            - groups.cap_loss.iter().map(|v| v[1]).sum::<i64>();
        let foreign_short: i64 = groups.cap_gain.iter().map(|v| v[0]).sum::<i64>()
            - groups.cap_loss.iter().map(|v| v[0]).sum::<i64>();
        let foreign_net_gain = positive(foreign_long - positive(-foreign_short));
        let excepted = *self.claim.rate_adjustment == ForeignRateAdjustment::Exception
            && line5 <= top_24
            && foreign_net_gain + self.foreign_qualified[s]
                < crate::year_constants::FOREIGN_RATE_ADJUSTMENT_EXCEPTION_DOLLARS;
        regular_bands.adjust &= !excepted;
        amt_bands.adjust &= !(excepted && line17 <= rates.bracket);
        let (bands, factors): (&RateBands, &RateFactors) =
            if amt { (&amt_bands, &AMT_FACTORS) } else { (&regular_bands, &REGULAR_FACTORS) };
        let excepted_here = if amt { excepted && line17 <= rates.bracket } else { excepted };
        out.line_18_reduction = bands.line_18_reduction(factors);
        let worldwide = if amt { rates.income(basis.amt_base) } else { basis.worldwide };
        out.line_18 = positive(worldwide - out.line_18_reduction);
        // ---- Rate adjustments after the kickout ----
        let mut qual_red = [0_i64; FOREIGN_GROUPS];
        for g in 0..FOREIGN_GROUPS {
            qual_red[g] = bands.line_1a_reduction(groups.qualified[g], factors);
            out.adjusted_dividends += qual_red[g];
        }
        let mut cap_net = [[0_i64; 2]; 4];
        let mut gain_gross_slot = [0_i64; 4];
        let mut loss_pre_slot = [0_i64; 4];
        for g in 0..FOREIGN_GROUPS {
            let slot = landing(g);
            for t in 0..2 {
                cap_net[slot][t] += groups.cap_gain[g][t] - groups.cap_loss[g][t];
            }
            gain_gross_slot[slot] += groups.cap_gain[g].iter().sum::<i64>();
            loss_pre_slot[slot] += out.loss_pre[g];
        }
        let schedule_d = basis.schedule_d[s];
        let adjusted = crate::foreign_capital::adjust(
            &CapitalInput {
                net: cap_net,
                schedule_d,
                elected_gain: basis.elected[0],
                excepted: excepted_here,
                long_term_factor: if amt { 5357 } else { 4054 },
            },
            bands,
        );
        let adjusted = match adjusted {
            Outcome::Adjusted(adjusted) => adjusted,
            Outcome::LeftOut => {
                out.left_out = true;
                crate::foreign_capital::Adjusted::default()
            }
        };
        out.worksheet_lines = adjusted.lines.clone();
        let mut gain_red = [0_i64; FOREIGN_GROUPS];
        let mut loss_adj = [0_i64; FOREIGN_GROUPS];
        for slot in 0..4 {
            let members: Vec<usize> = (0..FOREIGN_GROUPS).filter(|&g| landing(g) == slot).collect();
            if members.is_empty() {
                continue;
            }
            let reduction = gain_gross_slot[slot] - adjusted.gains[slot];
            let gain_weights: Vec<i64> =
                members.iter().map(|&g| groups.cap_gain[g].iter().sum::<i64>()).collect();
            for (&g, share) in members.iter().zip(spread(reduction, &gain_weights)) {
                gain_red[g] = share;
            }
            let loss_weights: Vec<i64> = members.iter().map(|&g| out.loss_pre[g]).collect();
            for (&g, share) in members.iter().zip(spread(adjusted.losses[slot], &loss_weights)) {
                loss_adj[g] = share;
            }
            out.capital_reduction += reduction;
        }
        for g in 0..FOREIGN_GROUPS {
            out.l1a_fin[g] = out.l1a_pre[g] - qual_red[g] - gain_red[g];
            out.l5_fin[g] = loss_adj[g];
            out.net_re[g] = net_re[g];
            out.net_fin[g] =
                out.l1a_fin[g] - (out.l2[g] + out.l3g[g] + out.l4a[g] + out.l4b[g] + out.l5_fin[g]);
        }
        // ---- Category income, taxes and the loss accounts ----
        let mut income = [0_i64; 4];
        let mut taxes = [0_i64; 4];
        let mut tax_owners = [Some([0_i64; 2]); 4];
        for g in 0..FOREIGN_GROUPS {
            let slot = landing(g);
            taxes[slot] += groups.tax_net[g];
            tax_owners[slot] = tax_owners[slot].and_then(|total| {
                let source = groups.tax_owners[g]?;
                Some([total[0].checked_add(source[0])?, total[1].checked_add(source[1])?])
            });
            // A passive group's income is its reallocated net; the other
            // groups keep their sign.
            let base = if PASSIVE_GROUPS.contains(&g) { net_re[g] } else { net_pre[g] };
            income[slot] += base - qual_red[g] - gain_red[g] - (loss_adj[g] - out.loss_pre[g]);
        }
        income[1] -= passive_loss;
        for slot in 0..4 {
            income[slot] += self.excess_loss[slot];
        }
        out.income = income;
        out.taxes = taxes;
        out.tax_owners = tax_owners;
        if out.left_out {
            return out;
        }
        let loss_worldwide = if amt {
            worldwide - out.line_18_reduction
        } else {
            basis.agi - basis.deduction - basis.below_line - out.line_18_reduction
        };
        let current =
            crate::foreign_losses::current_categories(&mut income, loss_worldwide, system, r);
        out.created_loss = current.created;
        out.amt_loss_created = amt && current.created;
        crate::require_localized!(
            r,
            !(amt && simplified)
                || (!current.created && !self.claim.loss_history.has_amt_activity()),
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            "Simplified foreign loss accounts",
            "ISSUE_SEPARATE_NEW_OR_RETAINED_AMT_LOSS_ACCOUNTS_WITH_THE_SIMPLIFIED_ELEC_6A3F1F75",
            "Separate new or retained AMT loss accounts with the simplified election require coordinated account worksheets. Regular loss adjustments flow through the elected regular Form 1116 line 17 numerator.",
        );
        self.claim.loss_history.apply(&mut income, &current, amt, r);
        out.line_17 = income;
        // ---- The limitation of each creditable category ----
        for c in 0..3 {
            let numerator =
                if amt && simplified { regular.map_or(0, |reg| reg.line_17[c]) } else { income[c] };
            if amt && simplified {
                out.line_17[c] = numerator;
            }
            let (credit_base, w) =
                if amt { (basis.tentative, out.line_18) } else { (basis.regular, out.line_18) };
            out.limit[c] =
                if w > 0 { fraction(credit_base, phase::cap(numerator, [w]), w) } else { 0 };
        }
        out
    }
}

// ---- The claim, end to end ----

// The printed lines of every column: (form line, id name, explanation).
const COLUMN_LINES: [(&str, &str, &str); 15] = [
    ("1a", "gross_income", "EXPLAIN_FORM_1116_LINE_1A"),
    ("2", "related_expenses", "EXPLAIN_FORM_1116_LINE_2"),
    ("3a", "itemized_or_standard", "EXPLAIN_FORM_1116_LINE_3A"),
    ("3b", "other_deductions", "EXPLAIN_FORM_1116_LINE_3B"),
    ("3c", "deductions_subtotal", "EXPLAIN_FORM_1116_LINE_3C"),
    ("3d", "gross_foreign_income", "EXPLAIN_FORM_1116_LINE_3D"),
    ("3e", "gross_income_all_sources", "EXPLAIN_FORM_1116_LINE_3E"),
    ("3f", "gross_income_ratio", "EXPLAIN_FORM_1116_LINE_3F"),
    ("3g", "apportioned_deductions", "EXPLAIN_FORM_1116_LINE_3G"),
    ("4a", "mortgage_interest", "EXPLAIN_FORM_1116_LINE_4A"),
    ("4b", "other_interest", "EXPLAIN_FORM_1116_LINE_4B"),
    ("5", "foreign_losses", "EXPLAIN_FORM_1116_LINE_5"),
    ("6", "total_deductions", "EXPLAIN_FORM_1116_LINE_6"),
    ("7", "taxable_income", "EXPLAIN_FORM_1116_LINE_7"),
    ("(w)", "taxes", "EXPLAIN_FORM_1116_PART_II_W"),
];

impl Form1116 {
    fn prefix(&self) -> String {
        format!("foreign_{}{}", if self.amt { "amt_" } else { "" }, NAMES[self.slot])
    }

    fn column_value(column: &Column, name: &str) -> i64 {
        match name {
            "gross_income" => column.line_1a,
            "related_expenses" => column.line_2,
            "itemized_or_standard" => column.line_3a,
            "other_deductions" => column.line_3b,
            "deductions_subtotal" => column.line_3c(),
            "gross_foreign_income" => column.line_3d,
            "gross_income_all_sources" => column.line_3e,
            "gross_income_ratio" => column.line_3f(),
            "apportioned_deductions" => column.line_3g,
            "mortgage_interest" => column.line_4a,
            "other_interest" => column.line_4b,
            "foreign_losses" => column.line_5,
            "total_deductions" => column.line_6(),
            "taxable_income" => column.line_7(),
            _ => 0,
        }
    }

    // Every printed line of the record, zero lines included.
    fn emit(&self, r: &mut FederalResult) {
        let source = if self.amt { AMT_SOURCE } else { SOURCE };
        let prefix = self.prefix();
        let inputs: &[&str] = &["credits.foreign_tax"];
        let emit_column = |r: &mut FederalResult, key: &str, column: &Column, totals: bool| {
            for (printed, name, explanation) in COLUMN_LINES {
                if name == "taxes" {
                    continue;
                }
                // The HTKO column carries lines 1a and 6 and the Total column
                // lines 1a, 6 and 7 only.
                let only: &[&str] = if totals || column.key == Some(ColumnKey::Htko) {
                    &["gross_income", "total_deductions", "taxable_income"]
                } else {
                    &[]
                };
                if !only.is_empty() && !only.contains(&name) {
                    continue;
                }
                let id = format!("{prefix}_{key}_{name}");
                let value = Self::column_value(column, name);
                if name == "gross_income_ratio" {
                    form_ratio(
                        r,
                        FormLine::new(TaxForm::Form1116, printed),
                        &id,
                        value,
                        inputs,
                        source,
                        Message::new(explanation, vec![]),
                    );
                } else {
                    form_line(
                        r,
                        FormLine::new(TaxForm::Form1116, printed),
                        &id,
                        value,
                        inputs,
                        source,
                        Message::new(explanation, vec![]),
                    );
                }
            }
        };
        for column in &self.columns {
            let key = match column.key.as_ref().unwrap() {
                ColumnKey::Country(code) => code.clone(),
                ColumnKey::Htko => "htko".into(),
            };
            emit_column(r, &key, column, false);
        }
        let total = Column {
            key: None,
            line_1a: self.total_1a(),
            line_6_override: Some(self.total_6()),
            ..Column::default()
        };
        emit_column(r, "total", &total, true);
        emit_lines(r, &mut (), &self.taxes, |(country, tax), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Form1116, "(w)")),
                &format!("{prefix}_{country}_taxes"),
                LineValue::MoneyDollars(*tax),
                inputs,
                source,
                Message::new("EXPLAIN_FORM_1116_PART_II_W", vec![]),
            ))
        });
        let part_iii: [(&str, &str, i64, &str); 10] = [
            ("8", "taxes_total", self.line_8(), "EXPLAIN_FORM_1116_LINE_8"),
            ("9", "taxes_for_category", self.line_8(), "EXPLAIN_FORM_1116_LINE_9"),
            ("10", "carryover", self.line_10, "EXPLAIN_FORM_1116_LINE_10"),
            ("11", "taxes_with_carryover", self.line_11(), "EXPLAIN_FORM_1116_LINE_11"),
            ("12", "tax_reduction", self.line_12, "EXPLAIN_FORM_1116_LINE_12"),
            ("13", "high_tax_kickout", self.line_13, "EXPLAIN_FORM_1116_LINE_13"),
            ("14", "taxes_available", self.line_14(), "EXPLAIN_FORM_1116_LINE_14"),
            ("15", "foreign_income", self.line_15(), "EXPLAIN_FORM_1116_LINE_15"),
            ("16", "income_adjustments", self.line_16, "EXPLAIN_FORM_1116_LINE_16"),
            ("17", "net_foreign_income", self.line_17(), "EXPLAIN_FORM_1116_LINE_17"),
        ];
        for (printed, name, value, explanation) in part_iii {
            // A section 901(j) category completes Part I and Part II through
            // line 8 and then lines 15 to 17; no credit follows.
            if self.slot == 3 && !matches!(printed, "8" | "15" | "16" | "17") {
                continue;
            }
            form_line(
                r,
                FormLine::new(TaxForm::Form1116, printed),
                &format!("{prefix}_{name}"),
                value,
                inputs,
                source,
                Message::new(explanation, vec![]),
            );
        }
        if self.slot == 3 {
            return;
        }
        form_line(
            r,
            FormLine::new(TaxForm::Form1116, "18"),
            &format!("{prefix}_worldwide_income"),
            self.line_18,
            inputs,
            source,
            Message::new("EXPLAIN_FORM_1116_LINE_18", vec![]),
        );
        form_ratio(
            r,
            FormLine::new(TaxForm::Form1116, "19"),
            &format!("{prefix}_limitation_ratio"),
            self.line_19(),
            inputs,
            source,
            Message::new("EXPLAIN_FORM_1116_LINE_19", vec![]),
        );
        emit_lines(
            r,
            &mut (),
            [
                ("20", "us_income_tax", self.line_20, "EXPLAIN_FORM_1116_LINE_20"),
                ("21", "maximum_credit", self.line_21(), "EXPLAIN_FORM_1116_LINE_21"),
                ("22", "limitation_increase", self.line_22, "EXPLAIN_FORM_1116_LINE_22"),
                ("23", "credit_limitation", self.line_23(), "EXPLAIN_FORM_1116_LINE_23"),
                ("24", "category_credit", self.line_24(), "EXPLAIN_FORM_1116_LINE_24"),
            ],
            |(printed, name, value, explanation), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::Form1116, printed)),
                    &format!("{prefix}_{name}"),
                    LineValue::MoneyDollars(value),
                    inputs,
                    source,
                    Message::new(explanation, vec![]),
                ))
            },
        );
    }
}

// The Schedule B column of a year: Roman numerals as printed.
fn roman(year: u32) -> &'static str {
    match year {
        2016 => "(i)",
        2017 => "(ii)",
        2018 => "(iii)",
        2019 => "(iv)",
        2020 => "(v)",
        2021 => "(vi)",
        2022 => "(ix)",
        2023 => "(x)",
        2024 => "(xi)",
        2025 => "(xii)",
        2026 => "(xiii)",
        _ => "(xiv)",
    }
}

impl ScheduleB {
    fn emit(&self, r: &mut FederalResult) {
        let source = "https://www.irs.gov/instructions/i1116sb";
        let system = if self.amt { "amt" } else { "regular" };
        let name = self.kind.name();
        let inputs: &[&str] = &["credits.foreign_tax.history", "credits.foreign_tax.carryback"];
        let put = |r: &mut FederalResult,
                   column: &str,
                   printed: &str,
                   id: String,
                   value: i64,
                   explanation: &'static str| {
            form_line(
                r,
                FormLine::new(TaxForm::Form1116ScheduleB, &format!("{printed}{column}")),
                &id,
                value,
                inputs,
                source,
                Message::new(explanation, vec![]),
            );
        };
        let (mut total_1, mut total_3, mut total_4, mut total_5, mut retained) = (0, 0, 0, 0, 0);
        for column in self.columns.iter().filter(|c| c.year < 2026) {
            let year = column.year;
            let roman = roman(year);
            let prefix = format!("foreign_{name}_{system}_{year}");
            for (printed, suffix, value, explanation) in [
                ("1", "carry_opening", column.line_1, "EXPLAIN_SCHEDULE_B_LINE_1"),
                ("2a", "carryback_adjustment", column.line_2a, "EXPLAIN_SCHEDULE_B_LINE_2A"),
                ("2b", "redetermination", column.line_2b, "EXPLAIN_SCHEDULE_B_LINE_2B"),
                ("2c", "other_adjustments", column.line_2c, "EXPLAIN_SCHEDULE_B_LINE_2C"),
                ("3", "carry_adjusted", column.line_3(), "EXPLAIN_SCHEDULE_B_LINE_3"),
                ("4", "carry_used", column.line_4, "EXPLAIN_SCHEDULE_B_LINE_4"),
            ] {
                put(r, roman, printed, format!("{prefix}_{suffix}"), value, explanation);
            }
            if year == 2016 {
                put(
                    r,
                    roman,
                    "5",
                    format!("{prefix}_carry_expired"),
                    column.line_5,
                    "EXPLAIN_SCHEDULE_B_LINE_5",
                );
            }
            put(
                r,
                roman,
                "8",
                format!("{prefix}_carry_remaining"),
                column.line_8(),
                "EXPLAIN_SCHEDULE_B_LINE_8",
            );
            total_1 += column.line_1;
            total_3 += column.line_3();
            total_4 += column.line_4;
            total_5 += column.line_5;
            retained += column.line_8();
        }
        if let Some(column) = self.columns.iter().find(|c| c.year == 2026) {
            let id = if self.amt {
                format!("foreign_{name}_amt_unused_tax")
            } else {
                format!("foreign_{name}_unused_tax")
            };
            put(r, "(xiii)", "6", id, column.line_6, "EXPLAIN_SCHEDULE_B_LINE_6");
            put(
                r,
                "(xiii)",
                "7",
                format!(
                    "foreign_{name}_{system}_{}",
                    if self.deduction_year {
                        "deduction_year_absorbed_prior_year"
                    } else {
                        "carryback_prior_year"
                    }
                ),
                column.line_7,
                "EXPLAIN_SCHEDULE_B_LINE_7",
            );
            put(
                r,
                "(xiii)",
                "8",
                format!("foreign_{name}_{system}_new_carryforward"),
                column.line_8(),
                "EXPLAIN_SCHEDULE_B_LINE_8_RETURN_YEAR",
            );
        }
        for (printed, suffix, value, explanation) in [
            ("1", "carry_opening_total", total_1, "EXPLAIN_SCHEDULE_B_LINE_1_TOTAL"),
            ("3", "carry_adjusted_total", total_3, "EXPLAIN_SCHEDULE_B_LINE_3_TOTAL"),
            ("4", "carry_used", total_4, "EXPLAIN_SCHEDULE_B_LINE_4_TOTAL"),
            ("5", "carry_expired", total_5, "EXPLAIN_SCHEDULE_B_LINE_5_TOTAL"),
            ("8", "retained_carryforward", retained, "EXPLAIN_SCHEDULE_B_LINE_8_TOTAL"),
        ] {
            put(
                r,
                "(xiv)",
                printed,
                format!("foreign_{name}_{system}_{suffix}"),
                value,
                explanation,
            );
        }
    }
}

fn section() -> crate::IssueContext {
    crate::IssueContext::Section(crate::TaxSection::MoreCredits)
}

// The correction context of the nth Schedule K-3 on the return (its K-1).
fn k3_context(d: &FederalDraft, position: usize) -> crate::IssueContext {
    d.income
        .iter()
        .filter(|entry| match &entry.income {
            FederalIncome::Partnership { data } => data.schedule_k3.is_some(),
            FederalIncome::Scorp { data } => {
                let data = &crate::ScorpK1::from(data);
                data.schedule_k3.is_some()
            }
            _ => false,
        })
        .nth(position)
        .map_or_else(section, |entry| {
            crate::interview::income_issue_context(entry)
                .alternatives(&[crate::TaxSection::PassThrough, crate::TaxSection::MoreCredits])
        })
}

// A conservative assumption: one note.
fn assumed(r: &mut FederalResult, key: &'static str) {
    crate::federal_rules::note_once_scoped(r, Message::new(key, vec![]), section());
}

// The ordinary Form 1116 claim: both tax systems, every record and Schedule B.
// Returns the regular and the AMT foreign tax credit.
pub(crate) fn ordinary(
    d: &FederalDraft,
    claim: &Claim<'_>,
    basis: &CreditBasis,
    r: &mut FederalResult,
) -> (i64, i64, crate::injured_spouse::income_categories::NativeForeignCreditSources) {
    use crate::injured_spouse::income_categories::NativeForeignCreditSources;
    let status = d.setup.as_ref().unwrap().filing_status();
    let small_interest =
        matches!(claim.interest_allocation, ForeignInterestAllocation::SmallForeignIncome);
    let k3_entries = crate::schedule_k3::attached_sources(d);
    let k3 = crate::schedule_k3::owned_credit(&k3_entries, small_interest, &basis.k3_limits);
    let simplified = matches!(claim.amt_limitation, ForeignAmtLimitation::Simplified { .. });
    // ---- Checks the person can correct ----
    let prior_passive_tax = claim
        .sources
        .iter()
        .any(|s| matches!(s.category, ForeignTaxCategory::PriorPassiveTax { .. }));
    crate::require_localized!(
        r,
        !prior_passive_tax
            || !claim.sources.iter().any(|source| {
                matches!(
                    source.category,
                    ForeignTaxCategory::Passive {
                        withholding: crate::ForeignWithholding::None,
                        ..
                    }
                ) && source.tax > 0
                    && source.income > 0
            }),
        &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
        "Prior passive foreign tax",
        "ISSUE_THE_TAX_ON_EARLIER_PASSIVE_INCOME_HAS_ZERO_CURRENT_INCOME_AND_BELON_BFDCCCF9",
        "The tax on earlier passive income has zero current income and belongs to the no-withholding/net-basis-tax group. Current income in that same group needs separate source-year coordination; current passive income in the other withholding groups is calculated independently.",
    );
    crate::require_localized!(
        r,
        !claim.sources.is_empty()
            || k3.row_count() > 0
            || crate::estate_foreign::requested(d)
            || claim.history.present()
            || claim.loss_history.present(),
        &section(),
        "Foreign tax credit",
        "ISSUE_FTC_NO_SOURCE",
        crate::extended_rules::FTC_NO_SOURCE,
    );
    claim.history.check(r);
    if let ForeignAmtLimitation::Simplified { confirmed, .. } = claim.amt_limitation {
        crate::require_localized!(
            r,
            // No was taken as the normal method (unknowns::supported).
            claim.amt_limitation.valid() && *confirmed,
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            "AMT foreign-credit election",
            "ISSUE_CONFIRM_A_SIMPLIFIED_LIMITATION_ELECTION_MADE_FOR_THE_FIRST_POST_19_ABA00D30",
            "Confirm a simplified limitation election made for the first post-1997 AMT foreign-credit claim year and still in force. A later first election or unconfirmed history cannot be assumed.",
        );
    }
    for source in claim.sources {
        crate::require_localized!(
            r,
            !matches!(source.group(), 1 | 2) || source.tax > 0,
            &section(),
            "Foreign withholding",
            "ISSUE_A_POSITIVE_WITHHOLDING_RATE_REQUIRES_PAID_ELIGIBLE_TAX_REVIEW_THE_P_9F62D5AC",
            "A positive withholding rate requires paid eligible tax; review the payment record and credit eligibility.",
        );
    }
    for (position, k3_entry) in k3_entries.iter().enumerate() {
        crate::require_localized!(
            r,
            k3_entry
                .schedule
                .countries
                .iter()
                .all(|row| !matches!(k3_entry.schedule.group(row), 1 | 2) || row.tax > 0),
            &k3_context(d, position),
            "Foreign withholding",
            "ISSUE_A_POSITIVE_WITHHOLDING_RATE_REQUIRES_PAID_ELIGIBLE_TAX_REVIEW_THE_P_9F62D5AC",
            "A positive withholding rate requires paid eligible tax; review the payment record and credit eligibility.",
        );
    }
    // ---- What the rows and the schedules give ----
    let estate =
        [crate::estate_foreign::sources(d, false), crate::estate_foreign::sources(d, true)];
    let catalogs: [Vec<_>; 2] = core::array::from_fn(|system| {
        claim
            .sources
            .iter()
            .map(crate::estate_foreign::ForeignCatalogSource::direct)
            .chain(estate[system].iter().cloned())
            .collect()
    });
    let catalog = &catalogs[0];
    let Some(gathered) = gather(d, claim, basis, &k3, &catalogs[0], small_interest, r) else {
        return (0, 0, NativeForeignCreditSources::unavailable(0));
    };
    let Some(gathered_amt) = gather(d, claim, basis, &k3, &catalogs[1], small_interest, r) else {
        return (0, 0, NativeForeignCreditSources::unavailable(0));
    };
    crate::require_localized!(
        r,
        !gathered.stale,
        &section(),
        "Foreign tax credit",
        "ISSUE_FTC_SOURCE_RECORD_MISSING",
        "A foreign source row names a business run abroad or an investment sale that is no longer on this return; choose it again or remove the row.",
    );
    let first =
        usize::from(d.injured_spouse.as_ref().is_some_and(|claim| claim.injured == Owner::Spouse));
    let groups = groups_of(&gathered, first);
    let foreign_gross: i64 = groups.gross.iter().sum();
    let qualified_rows: i64 = claim.sources.iter().map(|s| s.qualified()).sum();
    let qualified_by_system: [i64; 2] = core::array::from_fn(|system| {
        dollars(
            qualified_rows
                + estate[system].iter().map(|source| source.qualified()).sum::<i64>()
                + k3.qualified.iter().sum::<i64>(),
        )
    });
    let foreign_qualified = qualified_by_system[0];
    let employment_by_owner = gathered.employment;
    let employment: i64 = employment_by_owner.iter().sum();
    let reconciled = (0..2).all(|i| {
        // With a Form 2555 record the employment sources equal the W-2 pay
        // marked abroad plus the foreign-employer pay; without one the marks
        // are not read, so the sources need cover only the foreign-employer
        // pay (and all income is bounded below).
        if crate::extended_rules::foreign_claim(d, i).is_some() {
            employment_by_owner[i] == basis.foreign_wages[i]
        } else {
            employment_by_owner[i] >= basis.foreign_wages[i]
        }
    }) && gathered.passive_income <= basis.portfolio
        && foreign_qualified <= basis.qualified + dollars(k3.qualified.iter().sum::<i64>());
    crate::require_localized!(
        r,
        !small_interest
            || (basis.mortgage == [0; 2]
                && basis.investment == [0; 2]
                && basis.student_loan + basis.vehicle_interest == 0
                && !k3_entries.iter().any(|k| k.schedule.has_interest()))
            || gathered.cells.values().map(|c| c.gross).sum::<i64>()
                <= crate::year_constants::FOREIGN_SMALL_INCOME_INTEREST_DOLLARS * 100,
        &section(),
        "Foreign interest allocation",
        "ISSUE_ALLOCATING_INTEREST_ENTIRELY_TO_U_S_INCOME_REQUIRES_COMBINED_GROSS_E555DB95",
        "Allocating interest entirely to U.S. income requires combined gross foreign income of at most $5,000, across all sources and categories.",
    );
    // ---- Conservative assumptions, one note each ----
    for &position in &k3.interest_assumed {
        crate::federal_rules::note_once_scoped(
            r,
            Message::new(
                "WARN_FTC_K3_INTEREST_ASSUMED",
                vec![crate::MessageArg::Nested(alloc::boxed::Box::new(
                    k3_context(d, position).label(),
                ))],
            ),
            k3_context(d, position),
        );
    }
    let loan_interest =
        if small_interest { 0 } else { basis.student_loan + basis.vehicle_interest };
    if loan_interest > 0 && groups.gross.iter().sum::<i64>() > 0 {
        assumed(r, "WARN_FTC_PERSONAL_INTEREST_ASSUMED");
    }
    if !small_interest && gathered.business_interest > 0 && gathered.foreign_business_rows {
        assumed(r, "WARN_FTC_BUSINESS_INTEREST_ASSUMED");
    }
    let passive_gross: i64 = PASSIVE_GROUPS.map(|g| groups.gross[g]).sum();
    let penalty = if basis.savings_penalty > 0
        && gathered.passive_income > basis.portfolio - basis.penalty_interest
    {
        assumed(r, "WARN_FTC_SAVINGS_PENALTY_ASSUMED");
        basis.savings_penalty
    } else {
        0
    };
    if basis.elected != [0; 2]
        && (foreign_qualified > 0 || groups.cap_gain.iter().any(|g| g.iter().any(|v| *v > 0)))
    {
        assumed(r, "WARN_FTC_4952_ELECTION_ASSUMED");
    }
    // ---- Deductions assigned to sources ----
    let investment = if small_interest {
        let mut allocated = [[0; SLOTS]; 2];
        allocated[0][DOMESTIC] = basis.investment[0];
        allocated[1][DOMESTIC] = basis.investment[1];
        allocated
    } else {
        let mut passive_by_group = [[0; SLOTS]; 2];
        passive_by_group[0][..FOREIGN_GROUPS].copy_from_slice(&groups.gross);
        passive_by_group[1][..FOREIGN_GROUPS]
            .copy_from_slice(&groups_of(&gathered_amt, first).gross);
        claim.investment_assets.allocate(
            basis.investment,
            [&catalogs[0], &catalogs[1]],
            positive(basis.portfolio - gathered.passive_income),
            passive_by_group,
            r,
        )
    };
    let state_expenses =
        claim.state_taxes.allocate(basis.state_paid, basis.state_deduction, catalog, r);
    let educator = if basis.employee_pay > 0 && basis.educator > 0 {
        fraction(basis.educator, employment, basis.employee_pay)
    } else {
        0
    };
    let rates = crate::income_tax::AmtRates::for_year(2026, status).unwrap();
    let amt_rate = if rates.taxable(basis.amt_base) > rates.bracket { 28 } else { 26 };
    let excluded_total: i64 = (0..2)
        .map(|i| positive(basis.excluded_income[i] + basis.business_exclusions[i].related))
        .sum();
    // The disallowed excess business loss is shared by the businesses that lost
    // money, by their loss; a business run abroad's share goes to its category.
    let mut excess_loss = [0_i64; 4];
    let total_loss: i64 = basis.businesses.iter().map(|b| b.loss).sum();
    if basis.excess_business_loss > 0 && total_loss > 0 {
        for source in claim.sources {
            if let ForeignTaxCategory::Business { business, .. } = &source.category
                && let Some(facts) = basis.businesses.iter().find(|b| b.id == *business)
            {
                excess_loss[slot_of(source.group())] +=
                    fraction(basis.excess_business_loss, facts.loss, total_loss);
            }
        }
    }
    let ctx = Ctx {
        claim,
        basis,
        k3: &k3,
        cells: [&gathered.cells, &gathered_amt.cells],
        status,
        small_interest,
        groups: [groups, groups_of(&gathered_amt, first)],
        excluded_total,
        k3_partner: dollars(k3.partner_deductions),
        state_expenses,
        investment,
        penalty: {
            // The penalty spreads over the passive groups by their income.
            if passive_gross > 0 { penalty } else { 0 }
        },
        loan_interest,
        educator,
        amt_rate,
        foreign_qualified: qualified_by_system,
        excess_loss,
    };
    let regular = ctx.system(false, None, simplified, r);
    let amt = ctx.system(true, Some(&regular), simplified, r);
    crate::require_localized!(
        r,
        reconciled && foreign_gross <= regular.gross_all,
        &section(),
        "Foreign tax credit",
        "ISSUE_FOREIGN_SOURCE_RECORDS_MUST_RECONCILE_WITH_ENTERED_FOREIGN_PAY_AND_298ECA4C",
        "Foreign source records must reconcile with entered foreign pay and interest or ordinary dividends; enter income once in Income and only its source allocation here.",
    );
    if regular.left_out || amt.left_out {
        crate::federal_rules::note_once_scoped(
            r,
            Message::new("WARN_FTC_CAPITAL_RATE_GROUPS_LEFT_OUT", vec![]),
            section(),
        );
        return (0, 0, NativeForeignCreditSources::unavailable(0));
    }
    if !basis.line_18_mapped {
        crate::federal_rules::note_once_scoped(
            r,
            Message::new("WARN_FTC_LINE_18_LEFT_OUT", vec![]),
            section(),
        );
        return (0, 0, NativeForeignCreditSources::unavailable(0));
    }
    // ---- Schedule B, the records and the credit ----
    let mut return_ = Form1116Return::default();
    let mut credits = [0_i64; 2];
    let kinds = [
        ForeignCreditCategory::General,
        ForeignCreditCategory::Passive,
        ForeignCreditCategory::ForeignBranch,
    ];
    // The carryover each system brings in and the usage by category.
    let mut opening_totals = [[0_i64; 3]; 2];
    let mut backed = [[0_i64; 3]; 2];
    let mut schedule_records: Vec<ScheduleB> = Vec::new();
    for (c, kind) in kinds.iter().enumerate() {
        let excess =
            [positive(regular.taxes[c] - regular.limit[c]), positive(amt.taxes[c] - amt.limit[c])];
        let present = claim.history.schedule(*kind).is_some();
        let record_exists = ctx.slot_present(c);
        if !(present || record_exists || excess != [0; 2] || c == 0) {
            continue;
        }
        let capacity = claim.carryback.capacity(*kind, excess, claim.history, r);
        for (s, out) in [&regular, &amt].into_iter().enumerate() {
            let available = positive(out.limit[c] - out.taxes[c]);
            let mut columns = Vec::new();
            if let Some(schedule) = claim.history.schedule(*kind) {
                for year in 2016..=2025 {
                    let Some(column) = schedule.columns.iter().find(|v| v.year == year) else {
                        continue;
                    };
                    let opening = column.opening(s == 1);
                    let value = ScheduleBColumn {
                        year,
                        line_1: dollars(opening.carryover),
                        line_2a: dollars(opening.carryback_adjustment),
                        line_2b: dollars(opening.redetermination),
                        line_2c: dollars(opening.other_adjustments),
                        ..ScheduleBColumn::default()
                    };
                    opening_totals[s][c] += positive(value.line_3());
                    columns.push(value);
                }
            }
            use crate::kernel::carry::{Amount, Dollars, Entry, consume_by_key};
            let entries: Vec<_> = columns
                .iter()
                .map(|column| Entry {
                    key: column,
                    amount: Amount::<Dollars>::new(positive(column.line_3()))
                        .expect("nonnegative adjusted foreign-credit column"),
                    last_use_year: Some(column.year + 10),
                })
                .collect();
            let settlements = consume_by_key(
                &entries,
                Amount::new(available).expect("nonnegative foreign-credit capacity"),
                2026,
                |column| column.year,
            )
            .expect("valid foreign-credit cohorts");
            for (column, settlement) in columns.iter_mut().zip(settlements) {
                column.line_4 = settlement.used.get();
                column.line_5 = settlement.expired.get();
            }
            columns.push(ScheduleBColumn {
                year: 2026,
                line_6: excess[s],
                line_7: capacity[s],
                ..ScheduleBColumn::default()
            });
            backed[s][c] = capacity[s];
            schedule_records.push(ScheduleB {
                kind: *kind,
                amt: s == 1,
                columns,
                deduction_year: claim.carryback.deduction_year(),
            });
        }
    }
    // The Form 1116 of each category and system.
    for (s, out) in [&regular, &amt].into_iter().enumerate() {
        for slot in 0..4 {
            if let Some(form) = build_form(&ctx, out, slot, opening_totals[s][slot.min(2)]) {
                if slot < 3 {
                    credits[s] += form.line_24();
                }
                return_.forms.push(form);
            }
        }
    }
    let limit = [basis.regular, basis.tentative];
    let credit = [phase::cap(credits[0], [limit[0]]), phase::cap(credits[1], [limit[1]])];
    return_.schedules = schedule_records;
    let sources = current_credit_sources(credit[0], &regular, &return_);
    return_.emit(credit[0], r);
    emit_components(&ctx, [&regular, &amt], r);
    let _ = backed;
    (credit[0], credit[1], sources)
}

impl Ctx<'_> {
    // Whether the return has a source in the category slot.
    fn slot_present(&self, slot: usize) -> bool {
        self.cells.iter().any(|cells| cells.keys().any(|(g, _)| slot_of(*g) == slot))
    }
}

// The Form 1116 of one category and system, from the group results.
// Retain current-source use at the actual built category limits before row
// emission. A historical-used amount has no person in ScheduleB and cannot
// inherit the current payer. Category ceilings meet only after each native
// category has applied its own limit and source footing.
fn current_credit_sources(
    credit: i64,
    native: &SystemOut,
    result: &Form1116Return,
) -> crate::injured_spouse::income_categories::NativeForeignCreditSources {
    use crate::injured_spouse::income_categories::NativeForeignCreditSources;
    let unavailable = || NativeForeignCreditSources::unavailable(credit);
    if credit == 0 {
        return unavailable();
    }
    let kinds = [
        ForeignCreditCategory::General,
        ForeignCreditCategory::Passive,
        ForeignCreditCategory::ForeignBranch,
    ];
    let mut ceilings = [0_i64; 2];
    let mut category_total = 0_i64;
    for form in result.forms.iter().filter(|form| !form.amt && form.slot < 3) {
        let allowed = form.line_24();
        if allowed == 0 {
            continue;
        }
        let current = phase::cap(positive(native.taxes[form.slot]), [allowed]);
        let historical: i64 = result
            .schedules
            .iter()
            .filter(|schedule| !schedule.amt && schedule.kind == kinds[form.slot])
            .flat_map(|schedule| &schedule.columns)
            .map(|column| column.line_4)
            .sum();
        if allowed.checked_sub(current) != Some(historical) || historical != 0 {
            return unavailable();
        }
        let Some(owners) = native.tax_owners[form.slot]
            .filter(|owners| owners[0].checked_add(owners[1]) == Some(native.taxes[form.slot]))
        else {
            return unavailable();
        };
        let Some((minimum, maximum)) = crate::kernel::attribution::allowed_bounds(allowed, owners)
        else {
            return unavailable();
        };
        let category_ceilings = [maximum, allowed - minimum];
        for owner in 0..2 {
            let Some(sum) = ceilings[owner].checked_add(category_ceilings[owner]) else {
                return unavailable();
            };
            ceilings[owner] = sum;
        }
        let Some(sum) = category_total.checked_add(allowed) else {
            return unavailable();
        };
        category_total = sum;
    }
    if category_total < credit {
        return unavailable();
    }
    if crate::kernel::attribution::allowed_bounds(credit, ceilings).is_none() {
        return unavailable();
    }
    NativeForeignCreditSources {
        joint: credit,
        owners: crate::kernel::attribution::unique_allowed(credit, ceilings),
        capacity: Some(
            crate::injured_spouse::income_categories::NativeForeignCreditCapacity::CategoryDollars(
                ceilings,
            ),
        ),
    }
}

fn build_form(ctx: &Ctx<'_>, out: &SystemOut, slot: usize, opening: i64) -> Option<Form1116> {
    let groups = &ctx.groups[usize::from(out.amt)];
    let members: Vec<usize> = match slot {
        0 => vec![GENERAL],
        1 => PASSIVE_GROUPS.collect(),
        2 => vec![BRANCH],
        _ => vec![SANCTIONED],
    };
    let kicked: Vec<usize> = PASSIVE_GROUPS.filter(|&g| out.high[g]).collect();
    let htko = (slot == 0 || slot == 1) && !kicked.is_empty();
    let present = ctx.slot_present(slot) || (slot == 0 && htko);
    if !present && opening == 0 && out.line_17[slot] == 0 && out.income[slot] == 0 {
        return None;
    }
    let mut countries: alloc::collections::BTreeMap<String, Column> =
        alloc::collections::BTreeMap::new();
    let mut taxes: alloc::collections::BTreeMap<String, i64> = alloc::collections::BTreeMap::new();
    for &g in &members {
        let cells: Vec<(&String, &Cell)> = ctx.cells[usize::from(out.amt)]
            .iter()
            .filter(|((group, _), _)| *group == g)
            .map(|((_, country), cell)| (country, cell))
            .collect();
        if cells.is_empty() {
            continue;
        }
        let weights: Vec<i64> = cells.iter().map(|(_, c)| c.gross).collect();
        let tax_weights: Vec<i64> = cells.iter().map(|(_, c)| c.tax).collect();
        // A group moved to the general category stays in its own record as
        // entered; the HTKO column takes it out.
        let moved = out.high[g];
        let amounts = [
            if moved { out.l1a_pre[g] } else { out.l1a_fin[g] },
            out.l2[g]
                - if g == SANCTIONED && !out.amt { ctx.basis.foreign_tax_deduction } else { 0 },
            out.l3g[g],
            out.l4a[g],
            out.l4b[g],
            if moved { out.loss_pre[g] } else { out.l5_fin[g] },
            groups.gross[g],
        ];
        let shares: Vec<Vec<i64>> = amounts.iter().map(|a| spread(*a, &weights)).collect();
        let tax_shares = spread(groups.tax_gross[g], &tax_weights);
        for (i, (country, _)) in cells.iter().enumerate() {
            let column = countries.entry((*country).clone()).or_insert_with(|| Column {
                key: Some(ColumnKey::Country((*country).clone())),
                line_3a: out.line_3a,
                line_3b: out.line_3b,
                line_3e: out.gross_all,
                ..Column::default()
            });
            column.line_1a += shares[0][i];
            column.line_2 += shares[1][i]
                + if g == SANCTIONED && !out.amt {
                    ctx.basis.sanctioned_taxes.get(*country).copied().unwrap_or(0)
                } else {
                    0
                };
            column.line_3g += shares[2][i];
            column.line_4a += shares[3][i];
            column.line_4b += shares[4][i];
            column.line_5 += shares[5][i];
            column.line_3d += shares[6][i];
            *taxes.entry((*country).clone()).or_default() += tax_shares[i];
        }
    }
    let mut columns: Vec<Column> = countries.into_values().collect();
    if htko {
        let mut l1a = 0;
        let mut l6 = 0;
        for &g in &kicked {
            if slot == 1 {
                l1a -= out.l1a_pre[g];
                l6 -= out.l2[g] + out.l3g[g] + out.l4a[g] + out.l4b[g] + out.loss_pre[g];
            } else {
                let net_fin = out.net_re[g]
                    - (out.l1a_pre[g] - out.l1a_fin[g])
                    - (out.l5_fin[g] - out.loss_pre[g]);
                l1a += out.l1a_fin[g];
                l6 += out.l1a_fin[g] - net_fin;
            }
        }
        columns.push(Column {
            key: Some(ColumnKey::Htko),
            line_1a: l1a,
            line_6_override: Some(l6),
            ..Column::default()
        });
    }
    let reductions: i64 = members.iter().map(|&g| groups.tax_gross[g] - groups.tax_net[g]).sum();
    let amt = out.amt;
    let mut form = Form1116 {
        slot,
        amt,
        columns,
        taxes: taxes.into_iter().collect(),
        line_10: if slot < 3 { opening } else { 0 },
        line_12: -reductions,
        line_13: match slot {
            0 => out.kicked_taxes,
            1 => -out.kicked_taxes,
            _ => 0,
        },
        line_16: 0,
        line_18: out.line_18,
        line_20: if amt { ctx.basis.tentative } else { ctx.basis.regular },
        line_22: 0,
    };
    form.line_16 = out.line_17[slot] - form.total_7();
    Some(form)
}

impl Form1116Return {
    fn emit(&self, credit: i64, r: &mut FederalResult) {
        for form in &self.forms {
            form.emit(r);
        }
        for schedule in &self.schedules {
            schedule.emit(r);
        }
        // Part IV once, from the regular forms.
        let part_iv = |slot: usize| {
            self.forms.iter().find(|f| f.slot == slot && !f.amt).map_or(0, |f| f.line_24())
        };
        let sum: i64 = (0..3).map(part_iv).sum();
        emit_lines(
            r,
            &mut (),
            [
                ("25", "foreign_credit_section_951a", 0, "EXPLAIN_FORM_1116_LINE_25"),
                ("26", "foreign_credit_branch", part_iv(2), "EXPLAIN_FORM_1116_LINE_26"),
                ("27", "foreign_credit_passive", part_iv(1), "EXPLAIN_FORM_1116_LINE_27"),
                ("28", "foreign_credit_general", part_iv(0), "EXPLAIN_FORM_1116_LINE_28"),
                ("29", "foreign_credit_sanctioned", 0, "EXPLAIN_FORM_1116_LINE_29"),
                ("30", "foreign_credit_treaty", 0, "EXPLAIN_FORM_1116_LINE_30"),
                ("31", "foreign_credit_lump_sum", 0, "EXPLAIN_FORM_1116_LINE_31"),
                ("32", "foreign_credit_sum", sum, "EXPLAIN_FORM_1116_LINE_32"),
                ("33", "foreign_credit_limited", credit, "EXPLAIN_FORM_1116_LINE_33"),
                ("34", "foreign_credit_boycott", 0, "EXPLAIN_FORM_1116_LINE_34"),
            ],
            |(printed, name, value, explanation), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::Form1116, printed)),
                    name,
                    LineValue::MoneyDollars(value),
                    &["credits.foreign_tax"],
                    SOURCE,
                    Message::new(explanation, vec![]),
                ))
            },
        );
    }
}

// The component lines that stay beside the printed ones.
fn emit_components(ctx: &Ctx<'_>, systems: [&SystemOut; 2], r: &mut FederalResult) {
    if !ctx.k3_entries_empty() {
        emit_lines(
            r,
            &mut (),
            [
                (
                    "foreign_k3_gross_income_correction",
                    ctx.basis.k3_correction,
                    msg!("EXPLAIN_FOREIGN_K3_GROSS_INCOME_CORRECTION"),
                ),
                (
                    "foreign_k3_definitely_related_deductions",
                    dollars(ctx.k3.deductions.iter().sum()),
                    msg!("EXPLAIN_FOREIGN_K3_DEFINITELY_RELATED_DEDUCTIONS"),
                ),
                (
                    "foreign_k3_interest_expense",
                    dollars(ctx.k3.interest.iter().sum()),
                    msg!("EXPLAIN_FOREIGN_K3_INTEREST_EXPENSE"),
                ),
                (
                    "foreign_k3_tax_reduction",
                    dollars(ctx.k3.reductions),
                    msg!("EXPLAIN_FOREIGN_K3_TAX_REDUCTION"),
                ),
            ],
            |(id, amount, detail), _, _| {
                Some(calculation_row(
                    None,
                    id,
                    LineValue::MoneyDollars(amount),
                    &["income.partnership.schedule_k3"],
                    SOURCE,
                    detail,
                ))
            },
        );
    }
    if ctx.basis.excluded_income.iter().sum::<i64>() > 0 {
        let reduction: i64 = ctx.cells[0].values().map(|c| c.reduction).sum::<i64>()
            - dollars_to_cents(ctx.k3.reductions);
        emit_lines(
            r,
            &mut (),
            [
                (
                    "foreign_general_excluded_tax_reduction",
                    msg!("EXPLAIN_FOREIGN_GENERAL_EXCLUDED_TAX_REDUCTION"),
                ),
                (
                    "foreign_general_amt_excluded_tax_reduction",
                    msg!("EXPLAIN_FOREIGN_GENERAL_AMT_EXCLUDED_TAX_REDUCTION"),
                ),
            ],
            |(id, detail), _, _| {
                Some(calculation_row(
                    None,
                    id,
                    LineValue::MoneyDollars(dollars(reduction)),
                    &["credits.foreign_tax.sources", "screening.foreign_earned"],
                    SOURCE,
                    detail,
                ))
            },
        );
    }
    for out in systems {
        let name = if out.amt { "amt" } else { "regular" };
        line(
            r,
            &format!("foreign_{name}_domestic_deductions"),
            out.domestic,
            &["deductions", "credits.foreign_tax.interest_allocation"],
            "https://www.irs.gov/publications/p514",
            msg!("EXPLAIN_FOREIGN_N_DOMESTIC_DEDUCTIONS"),
        );
        let (adjustment, line_18, high_income, high_taxes, source) = if out.amt {
            (
                "foreign_amt_qualified_dividend_adjustment",
                "foreign_amt_line_18_rate_adjustment",
                "foreign_amt_high_tax_net_income",
                "foreign_amt_high_tax_taxes",
                AMT_SOURCE,
            )
        } else {
            (
                "foreign_qualified_dividend_adjustment",
                "foreign_line_18_rate_adjustment",
                "foreign_high_tax_net_income",
                "foreign_high_tax_taxes",
                SOURCE,
            )
        };
        line(
            r,
            adjustment,
            out.adjusted_dividends,
            &["credits.foreign_tax.sources"],
            source,
            if out.amt {
                msg!("EXPLAIN_FOREIGN_AMT_QUALIFIED_DIVIDEND_ADJUSTMENT")
            } else {
                msg!("EXPLAIN_FOREIGN_QUALIFIED_DIVIDEND_ADJUSTMENT")
            },
        );
        line(
            r,
            line_18,
            out.line_18_reduction,
            &["credits.foreign_tax.rate_adjustment"],
            source,
            if out.amt {
                msg!("EXPLAIN_FOREIGN_AMT_LINE_N_RATE_ADJUSTMENT")
            } else {
                msg!("EXPLAIN_FOREIGN_LINE_N_RATE_ADJUSTMENT")
            },
        );
        let kicked_income: i64 =
            PASSIVE_GROUPS.filter(|&g| out.high[g]).map(|g| positive(out.net_re[g])).sum();
        line(
            r,
            high_income,
            kicked_income,
            &["credits.foreign_tax"],
            SOURCE,
            if out.amt {
                msg!("EXPLAIN_FOREIGN_AMT_HIGH_TAX_NET_INCOME")
            } else {
                msg!("EXPLAIN_FOREIGN_HIGH_TAX_NET_INCOME")
            },
        );
        line(
            r,
            high_taxes,
            out.kicked_taxes,
            &["credits.foreign_tax"],
            SOURCE,
            if out.amt {
                msg!("EXPLAIN_FOREIGN_AMT_HIGH_TAX_TAXES")
            } else {
                msg!("EXPLAIN_FOREIGN_HIGH_TAX_TAXES")
            },
        );
        for (id, value) in &out.worksheet_lines {
            let prefix = if out.amt { "foreign_amt_" } else { "foreign_" };
            line(
                r,
                &format!("{prefix}{id}"),
                *value,
                &["credits.foreign_tax.sources"],
                source,
                Message::new("EXPLAIN_FOREIGN_CAPITAL_WORKSHEET", vec![]),
            );
        }
    }
}

fn dollars_to_cents(v: i64) -> i64 {
    v
}

impl Ctx<'_> {
    fn k3_entries_empty(&self) -> bool {
        self.k3.row_count() == 0 && self.k3.deductions.iter().all(|v| *v == 0)
    }
}

// The situations the App does not figure (R29): a checked one leaves the whole
// Form 1116 claim out. The code is the stored key of the catalog entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ForeignTaxSituation {
    FinancialServices = 1,
    TenPercentCompany,
    OilAndGas,
    MineralDepletion,
    Boycott,
    SuspendedTaxes,
    UnfiledInformationReturn,
    PropertyDisposition,
    DirectlyAllocatedDebt,
}

impl ForeignTaxSituation {
    pub(crate) const ALL: [Self; 9] = [
        Self::FinancialServices,
        Self::TenPercentCompany,
        Self::OilAndGas,
        Self::MineralDepletion,
        Self::Boycott,
        Self::SuspendedTaxes,
        Self::UnfiledInformationReturn,
        Self::PropertyDisposition,
        Self::DirectlyAllocatedDebt,
    ];
    pub(crate) fn code(self) -> u32 {
        self as u32
    }
    pub(crate) fn from_code(code: u32) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.code() == code)
    }
    pub(crate) fn label_key(self) -> &'static str {
        match self {
            Self::FinancialServices => "FOREIGN_SITUATION_1",
            Self::TenPercentCompany => "FOREIGN_SITUATION_2",
            Self::OilAndGas => "FOREIGN_SITUATION_3",
            Self::MineralDepletion => "FOREIGN_SITUATION_4",
            Self::Boycott => "FOREIGN_SITUATION_5",
            Self::SuspendedTaxes => "FOREIGN_SITUATION_6",
            Self::UnfiledInformationReturn => "FOREIGN_SITUATION_7",
            Self::PropertyDisposition => "FOREIGN_SITUATION_8",
            Self::DirectlyAllocatedDebt => "FOREIGN_SITUATION_9",
        }
    }
}

// The accepted and Page country fields both declare ordinary String Size2..3.
// Keep their separate two-codepoint-or-RIC format rule in the source walk.
pub(crate) use crate::foreign_allocation_source::country_size_valid;
// Native source authority; country failures carry a relative source-row path.
// `businesses` comes from actual accepted abroad-business identities and owners.
pub(crate) fn claim_rule(
    claim: &crate::ForeignTaxClaim,
    businesses: &[(i64, Owner)],
) -> Option<crate::validation::ValidationIssue> {
    let crate::ForeignTaxClaim::Ordinary { basis, sources, history, .. } = claim else {
        return None;
    };
    if *basis != ForeignTaxBasis::Paid
        && sources.iter().any(|s| {
            matches!(
                s.category,
                ForeignTaxCategory::PriorEmployeeWageTax { .. }
                    | ForeignTaxCategory::PriorPassiveTax { .. }
            )
        })
    {
        return Some(
            crate::validation::ValidationIssue::new("FOREIGN_PRIOR_TAX_NEEDS_PAID_BASIS")
                .at_field("basis"),
        );
    }
    let mut seen_business: Vec<i64> = Vec::new();
    let mut seen_sale: Vec<i64> = Vec::new();
    for (index, source) in sources.iter().enumerate() {
        if source.qualified() > source.income {
            return Some(
                crate::validation::ValidationIssue::new("FOREIGN_QUALIFIED_ABOVE_INCOME")
                    .at_field("qualified")
                    .in_field("category")
                    .in_list("sources", index),
            );
        }
        // String UTF-8 byte Size and the source's codepoint format are independent.
        if !country_size_valid(&source.country) {
            return Some(
                crate::validation::ValidationIssue::new("FOREIGN_COUNTRY_CODE")
                    .at_field("country")
                    .in_list("sources", index),
            );
        }
        if source.country.chars().count() != 2 && source.country != "RIC" {
            return Some(
                crate::validation::ValidationIssue::new("FOREIGN_COUNTRY_CODE")
                    .at_field("country")
                    .in_list("sources", index),
            );
        }
        match &source.category {
            ForeignTaxCategory::Business { business, .. } => {
                if source.income != 0 {
                    return Some(
                        crate::validation::ValidationIssue::new(
                            "FOREIGN_BUSINESS_INCOME_FROM_SCHEDULE_C",
                        )
                        .at_field("income")
                        .in_list("sources", index),
                    );
                }
                if seen_business.contains(business) {
                    return Some(
                        crate::validation::ValidationIssue::new("FOREIGN_BUSINESS_TWICE")
                            .at_field("business")
                            .in_field("category")
                            .in_list("sources", index),
                    );
                }
                seen_business.push(*business);
                if !businesses.iter().any(|(id, owner)| id == business && *owner == source.owner) {
                    return Some(
                        crate::validation::ValidationIssue::new("FOREIGN_BUSINESS_OWNER")
                            .at_field("owner")
                            .in_list("sources", index),
                    );
                }
            }
            ForeignTaxCategory::Passive { sales, .. } => {
                for (sale_index, sale) in sales.iter().enumerate() {
                    if seen_sale.contains(sale) {
                        return Some(
                            crate::validation::ValidationIssue::new("FOREIGN_SALE_TWICE")
                                .at_list_item("sales", sale_index)
                                .in_field("category")
                                .in_list("sources", index),
                        );
                    }
                    seen_sale.push(*sale);
                }
            }
            _ => {}
        }
    }
    history.issue().map(|issue| issue.in_field("history"))
}

// What Form 1062 changes in the credit's inputs (section 1062(d)(1)): the
// return refigured without the elected farmland sales.
pub(crate) struct Reduced {
    pub taxable: i64,
    pub preference: [i64; 3],
    pub regular: i64,
    pub gain: i64,
    pub schedule_d: crate::foreign_capital::ScheduleD,
}

// The regular foreign tax credit of the ordinary claim on the reduced return,
// figured by the same records with the reduced line 3e, line 18, rate bands and
// line 20; nothing is written to the estimate.
pub(crate) fn credit_without_elected_gain(
    d: &FederalDraft,
    claim: &crate::ForeignTaxClaim,
    basis: &CreditBasis,
    reduced: Reduced,
) -> Option<i64> {
    let crate::ForeignTaxClaim::Ordinary {
        basis: tax_basis,
        sources,
        history,
        carryback,
        amt_limitation,
        interest_allocation,
        investment_assets,
        loss_history,
        state_taxes,
        rate_adjustment,
        ..
    } = claim
    else {
        return None;
    };
    let mut basis = basis.clone();
    basis.taxable = reduced.taxable;
    basis.preference[0] = reduced.preference;
    basis.regular = reduced.regular;
    basis.worldwide = positive(basis.worldwide - reduced.gain);
    basis.capital_gross[0] = positive(basis.capital_gross[0] - reduced.gain);
    basis.schedule_d[0] = reduced.schedule_d;
    let mut scratch = FederalResult::empty(0);
    let (credit, _, _) = ordinary(
        d,
        &Claim {
            basis: tax_basis,
            sources,
            history,
            carryback,
            amt_limitation,
            interest_allocation,
            investment_assets,
            loss_history,
            state_taxes,
            rate_adjustment,
        },
        &basis,
        &mut scratch,
    );
    Some(credit)
}
