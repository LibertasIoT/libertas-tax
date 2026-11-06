// Coverage: Form 1040 lines 1b/1d/1h and Schedule 1 line 8 income with no form of its own, and Pub. 525 Worksheet 2;
//   i1040gi (2025) "Lines 8a Through 8z", Pub. 525 "Other Income", "Recoveries" and
//   "Unemployment Benefits", Rev. Rul. 2023-14, 26 U.S.C. 61, 74, 111.
//   Household employee wages without a W-2 enter 1b (a W-2-due notice at $3,000);
//   strike benefits enter 1h and EIC earned income but not IRA compensation; Medicaid
//   waiver pay enters 1d, with a matching 8s exclusion when care was in the home.
//   The same ownership applies to routed 1099-NEC/MISC source amounts.
//   Lines: the Other income page's "Other income" kind (OtherIncomeItem): Alaska Permanent Fund
//   dividends (8g), prizes and awards without a 1099-MISC (8i; the old Prize kind's rules),
//   Olympic and Paralympic medals and USOC prize money (8m, excluded on 24c when AGI before the
//   exclusion is $1,000,000 or less, $500,000 if married filing separately), pay for work while
//   an inmate that no W-2 reports (8u), digital assets received as income (8v), and line 8z:
//   recoveries of earlier-year itemized deductions under the tax-benefit rule, credit card
//   disability and unemployment insurance benefits above the premiums paid, private or union
//   unemployment fund benefits above the person's own payments, and other taxable income. The
//   same kinds are the destination of a 1099-NEC or 1099-MISC amount that is not business
//   income (IncomeUse::Income). Worksheet 2 (lines 1a-11 and the A/B/C split) is figured once
//   for each prior year over the year's state refund (a Form 1099-G refund of that year, with
//   Worksheet 2a for real estate and personal property tax refunded) and the year's other
//   recovery item. Not figured: a recovery of an amount refigured with unused credits, AMT, a
//   0%-rate capital gain recomputation, a dependent filer or an earlier refund of the same
//   deduction (the item is left out with WARN_RECOVERY_LEFT_OUT), and the Worksheet 2 line 5
//   "previously refunded" amount.
//   Evidence: this module, information_returns.rs, tax_refund.rs, federal_rules.rs, unknowns.rs;
//   tests/other_income.rs (fixtures D and E, with the 8g, 8i, 8m, 8u, 8v and 8z kinds).
pub use crate::prior_recovery_source::{
    PriorYearRecovery, VehicleInterestRecovery, VehicleRecoveryYear, VehicleRefundOrigin,
};

use crate::kernel::phase;
use crate::kernel::ratio::positive;
use crate::{FederalDraft, FederalIncome, PrizeDisposition};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Other income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct OtherIncomeItem {
    /// What it is
    pub kind: OtherIncomeKind,
    /// Amount received in 2026
    /// In cash, or the fair market value of property you received. For a prize, its cash part; the value of goods or services is asked with the prize. For a refund or reimbursement of an earlier year's deduction, the total you got back.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Federal income tax withheld
    /// From the payer's statement, if any.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
}

/// What it is
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum OtherIncomeKind {
    /// Alaska Permanent Fund dividend
    /// The whole dividend, including any energy relief part (Schedule 1 line 8g). Your own or your spouse's; a child's dividend you report for the child goes on that child's Form 8814.
    AlaskaDividend,
    /// Prize or award
    /// A contest, drawing, sweepstakes or quiz-show prize, or an award not for services (Schedule 1 line 8i).
    Prize {
        /// Value of goods or services won
        /// Their fair market value, in addition to the cash amount.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        noncash: i64,
        /// What happened to it
        #[libertas_default(Received)]
        disposition: PrizeDisposition,
    },
    /// Olympic or Paralympic medal or USOC prize money
    /// A medal's value and prize money from the United States Olympic Committee for competing in the Games (Schedule 1 line 8m). The App excludes it (line 24c) when your adjusted gross income is $1,000,000 or less ($500,000 if married filing separately).
    OlympicPrize,
    /// Digital assets received as income
    /// Their fair market value in U.S. dollars when you received them: staking or mining rewards, an airdrop or a hard fork (Schedule 1 line 8v). That value is your basis when you sell them. Digital assets paid as wages are on your W-2; mining or staking you do as a business goes on Self-employment.
    DigitalAssets,
    /// Pay for work while an inmate, not on a W-2
    /// Pay for work you did while an inmate in a penal institution (Schedule 1 line 8u). Pay shown on a W-2 is marked on that W-2 under Wages.
    InmatePay,
    /// Household employee pay without a W-2
    /// Pay for work in an employer's home, such as babysitting, cleaning or care (Form 1040 line 1b). An employer paying $3,000 or more in cash in 2026 owes a W-2.
    HouseholdWages,
    /// Strike or lockout benefits
    /// Union cash or property other than true gifts (Form 1040 line 1h).
    StrikeBenefits,
    /// Medicaid waiver payments without a W-2
    /// A home care business instead reports these as its business receipts.
    MedicaidWaiver {
        /// The person you cared for lived in your home with you
        /// Care under the person's plan in the home where you both live, for no more than 10 people under 19 or 5 people 19 or older (Notice 2014-7). Yes excludes the payment and counts it as earned income where it raises credits. No reports taxable wages on line 1d.
        in_your_home: bool,
    },
    /// Refund or reimbursement of an earlier year's itemized deduction
    /// For example insurance paying back medical costs you deducted. Only the part that lowered that year's tax is taxable (Schedule 1 line 8z). A state or local income tax refund is entered as its 1099-G; a refund of something deducted on a business schedule goes on that business.
    Recovery {
        /// That year's return
        worksheet: PriorYearRecovery,
    },
    /// Credit card disability or unemployment insurance benefits
    /// Payments the plan made on your card balance. Only the part above the premiums you paid in 2026 is taxable (Schedule 1 line 8z).
    CardInsurance {
        /// Premiums you paid for this insurance in 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        premiums: i64,
    },
    /// Benefits from a private or union unemployment fund
    /// Not state unemployment compensation, which is a 1099-G. Only the part above your own nondeductible payments into the fund is taxable (Schedule 1 line 8z); benefits paid from regular union dues are taxable in full.
    UnemploymentFund {
        /// Your payments into the fund not yet recovered
        /// Zero for benefits paid from regular union dues.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        contributions: i64,
    },
    /// Other taxable income
    /// Income not reported anywhere else on your return (Schedule 1 line 8z), such as a reward, found property, punitive or other taxable damages, fees as an executor who is not in that business, pay for a medical research study, an insurance salesperson's termination payments, a deceased worker's pay you received as beneficiary, or kickbacks or manufacturer incentives you got as an employee. Not self-employment income, wages, or deferred compensation under a plan that fails section 409A.
    Other {
        /// Type of income
        /// Shown with the amount on Schedule 1 line 8z.
        #[libertas_size(min = 1)]
        description: alloc::string::String,
    },
}

// Where one item's amount lands on Schedule 1 (cents). Digital assets, inmate pay,
// Alaska dividends, the Olympic prize before its exclusion, prizes and 8z income
// are each their own line; a recovery is figured with its year (`Worksheet`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Lines {
    pub household: i64,
    pub strike: i64,
    pub medicaid_taxable: i64,
    pub medicaid_excluded: i64,
    pub alaska: i64,
    pub prize: i64,
    pub olympic: i64,
    pub inmate: i64,
    pub digital: i64,
    pub other_8z: i64,
}

impl OtherIncomeItem {
    pub(crate) fn valid(&self) -> bool {
        let money = |v: i64| (0..=100_000_000_000).contains(&v);
        money(self.amount)
            && money(self.withholding)
            && i128::from(self.withholding)
                <= i128::from(self.amount)
                    + i128::from(match &self.kind {
                        OtherIncomeKind::Prize { noncash, .. } => *noncash,
                        _ => 0,
                    })
            && match &self.kind {
                OtherIncomeKind::Prize { noncash, .. } => money(*noncash),
                OtherIncomeKind::Recovery { worksheet } => worksheet.valid(),
                OtherIncomeKind::CardInsurance { premiums } => money(*premiums),
                OtherIncomeKind::UnemploymentFund { contributions } => money(*contributions),
                OtherIncomeKind::Other { description } => !description.is_empty(),
                _ => true,
            }
    }

    // The rule key the client rules also state, for the save to repeat.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        match &self.kind {
            OtherIncomeKind::Prize { noncash, .. }
                if i128::from(self.withholding)
                    > i128::from(self.amount) + i128::from(*noncash) =>
            {
                Some(
                    crate::validation::ValidationIssue::new("PRIZE_WITHHOLDING_ABOVE_AWARD")
                        .at_field("withholding"),
                )
            }
            OtherIncomeKind::Prize { .. } => None,
            _ if self.withholding > self.amount => Some(
                crate::validation::ValidationIssue::new("OTHER_INCOME_WITHHOLDING_ABOVE_AMOUNT")
                    .at_field("withholding"),
            ),
            OtherIncomeKind::Recovery { worksheet } if worksheet.deducted > worksheet.itemized => {
                Some(
                    crate::validation::ValidationIssue::new("RECOVERY_DEDUCTED_ABOVE_ITEMIZED")
                        .at_field("deducted")
                        .in_field("worksheet")
                        .in_field("kind"),
                )
            }
            _ => None,
        }
    }

    // The income left out of the estimate: a recovery whose tax-benefit
    // situation is not the ordinary one.
    pub(crate) fn left_out(&self) -> bool {
        matches!(&self.kind, OtherIncomeKind::Recovery { worksheet } if !worksheet.ordinary)
    }

    // The amount the item contributes before the return-wide steps (the
    // Olympic exclusion on line 24c, the recovery worksheet): the preview of a
    // document with no return behind it. Cents.
    pub(crate) fn lines(&self) -> Lines {
        let mut lines = Lines::default();
        match &self.kind {
            OtherIncomeKind::AlaskaDividend => lines.alaska = self.amount,
            OtherIncomeKind::Prize { noncash, disposition } => {
                lines.prize = prize_taxable(self.amount + noncash, disposition);
            }
            OtherIncomeKind::OlympicPrize => lines.olympic = self.amount,
            OtherIncomeKind::DigitalAssets => lines.digital = self.amount,
            OtherIncomeKind::InmatePay => lines.inmate = self.amount,
            OtherIncomeKind::HouseholdWages => lines.household = self.amount,
            OtherIncomeKind::StrikeBenefits => lines.strike = self.amount,
            OtherIncomeKind::MedicaidWaiver { in_your_home: true } => {
                lines.medicaid_excluded = self.amount
            }
            OtherIncomeKind::MedicaidWaiver { in_your_home: false } => {
                lines.medicaid_taxable = self.amount
            }
            // Figured with the year's other facts (`worksheets`).
            OtherIncomeKind::Recovery { .. } => {}
            OtherIncomeKind::CardInsurance { premiums } => {
                lines.other_8z = positive(self.amount - premiums)
            }
            OtherIncomeKind::UnemploymentFund { contributions } => {
                lines.other_8z = positive(self.amount - contributions)
            }
            OtherIncomeKind::Other { .. } => lines.other_8z = self.amount,
        }
        lines
    }

    // The amount shown beside the document: the taxable amount of a kind that
    // needs nothing else, the stand-alone Worksheet 2 line 11 of a recovery,
    // and zero for an Olympic prize that its exclusion removes.
    pub(crate) fn preview(&self) -> i64 {
        match &self.kind {
            OtherIncomeKind::Recovery { worksheet } if worksheet.ordinary => {
                worksheet.alone(self.amount).line_11
            }
            OtherIncomeKind::Recovery { .. } => 0,
            _ => {
                let lines = self.lines();
                lines.household
                    + lines.strike
                    + lines.medicaid_taxable
                    + lines.alaska
                    + lines.prize
                    + lines.olympic
                    + lines.inmate
                    + lines.digital
                    + lines.other_8z
            }
        }
    }

    // Whether the Olympic prize can be excluded at this adjusted gross
    // income (section 74(d)(2)(A)): `agi_excluded` is the AGI with the prize
    // excluded, in cents.
    pub(crate) fn olympic_excluded(prize: i64, agi_excluded: i64, separate: bool) -> bool {
        let limit = if separate {
            crate::year_constants::OLYMPIC_EXCLUSION_AGI_LIMIT_SEPARATE_CENTS
        } else {
            crate::year_constants::OLYMPIC_EXCLUSION_AGI_LIMIT_CENTS
        };
        prize > 0 && agi_excluded + prize <= limit
    }
}

// The taxable part of a prize under its explicit exclusion answer.
pub(crate) fn prize_taxable(amount: i64, disposition: &PrizeDisposition) -> i64 {
    match disposition {
        PrizeDisposition::Refused | PrizeDisposition::DirectCharity { eligible: true } => 0,
        PrizeDisposition::Received | PrizeDisposition::DirectCharity { eligible: false } => amount,
    }
}

// Publication 525 Worksheet 2, lines 3 to 11, in cents.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Worksheet {
    // Line 3: the recoveries, each no more than what was deducted for it.
    pub line_3: i64,
    // Line 8: the itemized deductions above the standard deduction.
    pub line_8: i64,
    // Line 9: the smaller of lines 3 and 8.
    pub line_9: i64,
    // Line 11: the amount to include, after a negative taxable income.
    pub line_11: i64,
    // Schedule 1 line 1 and line 8z shares of line 11.
    pub to_line_1: i64,
    pub to_8z: i64,
    // The part of `to_8z` that is line 1b, the refund of real estate or personal property tax:
    // a recovery Form 6251 line 2b subtracts (section 56(b)(1)(C)).
    pub to_8z_taxes: i64,
}

impl Worksheet {
    // Lines 3 through 11 for the lines 1a, 1b and 2 amounts (already capped at
    // what was deducted and, for 1a and 1b, by Worksheet 2a). The A/B/C split
    // sends line 1a's share of line 11 to Schedule 1 line 1 and the rest to
    // line 8z.
    pub(crate) fn figure(
        line_1a: i64,
        line_1b: i64,
        line_2: i64,
        itemized: i64,
        standard: i64,
        taxable_income: i64,
    ) -> Self {
        let line_3 = line_1a + line_1b + line_2;
        let line_8 = itemized - standard;
        if line_8 <= 0 || line_3 <= 0 {
            return Self { line_3, line_8: positive(line_8), ..Self::default() };
        }
        let line_9 = phase::cap(line_3, [line_8]);
        let line_11 = if taxable_income >= 0 { line_9 } else { positive(line_9 + taxable_income) };
        let to_line_1 = if line_11 == line_3 {
            line_1a
        } else if line_1a == 0 {
            0
        } else {
            // Line A is line 1a over line 3; line B is line 11 times it.
            (crate::kernel::ratio::trunc_i128(
                i128::from(line_11) * i128::from(line_1a),
                i128::from(line_3),
            )) as i64
        };
        let to_8z_taxes = if line_11 == line_3 {
            line_1b
        } else if line_1b == 0 {
            0
        } else {
            (crate::kernel::ratio::trunc_i128(
                i128::from(line_11) * i128::from(line_1b),
                i128::from(line_3),
            )) as i64
        };
        Self { line_3, line_8, line_9, line_11, to_line_1, to_8z: line_11 - to_line_1, to_8z_taxes }
    }
}

impl PriorYearRecovery {
    // The worksheet of this one item with no state refund of the year.
    pub(crate) fn alone(&self, amount: i64) -> Worksheet {
        Worksheet::figure(
            0,
            0,
            phase::cap(amount, [self.deducted]),
            self.itemized,
            self.standard,
            self.taxable_income,
        )
    }
}

// One prior year's worksheet over its state refund and its other recovery.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct YearWorksheet {
    pub year: i32,
    pub sheet: Worksheet,
    // A state refund of the year is part of it.
    pub with_refund: bool,
    // The refund and the recovery state different itemized or standard
    // deductions for the year.
    pub facts_differ: bool,
}

// The years the return recovers something in: the year of a Form 1099-G
// income tax refund with its Schedule A facts, and the year of an Other
// income recovery. Worksheet 2 is figured once for each year, so a refund
// of income tax and the same year's other recoveries share the one line 8
// limit, and a refund of real estate or personal property tax reaches line 8z
// through Worksheet 2a (Pub. 525 Worksheets 2 and 2a). A failing refund is
// reported where the refund is entered; here it adds nothing.
fn mortgage_recoveries(d: &FederalDraft) -> impl Iterator<Item = (i64, PriorYearRecovery)> {
    d.deductions.iter().flat_map(|d| d.mortgage_loans().unwrap_or_default()).filter_map(|loan| {
        match loan.refund_of_interest {
            crate::MortgageInterestRefund::Yes {
                amount,
                origin: Some(crate::MortgageRefundOrigin::Deducted(worksheet)),
            } if worksheet.ordinary => Some((amount, worksheet)),
            _ => None,
        }
    })
}
pub(crate) fn worksheets(d: &FederalDraft) -> Vec<YearWorksheet> {
    let mut years: Vec<i32> = Vec::new();
    for entry in &d.income {
        let year = match &entry.income {
            FederalIncome::GovernmentPayment { data }
                if data.form.state_or_local_income_tax_refunds != 0 =>
            {
                crate::FederalTaxRefund::from(data).history.single_year()
            }
            FederalIncome::OtherIncome { data } => match &data.kind {
                OtherIncomeKind::Recovery { worksheet } if worksheet.ordinary => {
                    Some(worksheet.year)
                }
                _ => None,
            },
            _ => None,
        };
        if let Some(year) = year
            && !years.contains(&year)
        {
            years.push(year);
        }
    }
    for (_, w) in mortgage_recoveries(d) {
        if !years.contains(&w.year) {
            years.push(w.year);
        }
    }
    years.sort_unstable();
    years.into_iter().filter_map(|year| year_worksheet(d, year)).collect()
}

fn year_worksheet(d: &FederalDraft, year: i32) -> Option<YearWorksheet> {
    let mut refund = None;
    let mut line_2 = 0;
    let mut recovery_facts: Option<&PriorYearRecovery> = None;
    let mut recovery_facts_differ = false;
    for entry in &d.income {
        match &entry.income {
            FederalIncome::GovernmentPayment { data }
                if refund.is_none()
                    && data.form.state_or_local_income_tax_refunds != 0
                    && crate::FederalTaxRefund::from(data).history.single_year() == Some(year) =>
            {
                refund = Some(crate::FederalTaxRefund::from(data));
            }
            FederalIncome::OtherIncome { data } => {
                if let OtherIncomeKind::Recovery { worksheet } = &data.kind
                    && worksheet.ordinary
                    && worksheet.year == year
                {
                    line_2 += phase::cap(data.amount, [worksheet.deducted]);
                    if let Some(prior) = recovery_facts {
                        recovery_facts_differ |= prior.itemized != worksheet.itemized
                            || prior.standard != worksheet.standard
                            || prior.taxable_income != worksheet.taxable_income;
                    } else {
                        recovery_facts = Some(worksheet);
                    }
                }
            }
            _ => {}
        }
    }
    let mortgage_recoveries: Vec<_> =
        mortgage_recoveries(d).filter(|(_, w)| w.year == year).collect();
    for (amount, worksheet) in &mortgage_recoveries {
        line_2 += phase::cap(*amount, [worksheet.deducted]);
        if let Some(prior) = recovery_facts {
            recovery_facts_differ |= prior.itemized != worksheet.itemized
                || prior.standard != worksheet.standard
                || prior.taxable_income != worksheet.taxable_income;
        } else {
            recovery_facts = Some(worksheet);
        }
    }
    let (line_1a, line_1b) = match &refund {
        Some(refund) => refund.history.state_local_lost(refund.amount).ok().flatten()?,
        None => (0, 0),
    };
    let (itemized, standard, taxable_income, facts_differ) = match (&refund, recovery_facts) {
        (Some(refund), Some(worksheet)) => {
            let (itemized, standard) = refund.history.year_deductions()?;
            (
                itemized,
                standard,
                worksheet.taxable_income,
                itemized != worksheet.itemized || standard != worksheet.standard,
            )
        }
        (Some(refund), None) => {
            let (itemized, standard) = refund.history.year_deductions()?;
            (itemized, standard, 0, false)
        }
        (None, Some(worksheet)) => {
            (worksheet.itemized, worksheet.standard, worksheet.taxable_income, false)
        }
        (None, None) => return None,
    };
    Some(YearWorksheet {
        year,
        sheet: Worksheet::figure(line_1a, line_1b, line_2, itemized, standard, taxable_income),
        with_refund: refund.is_some(),
        facts_differ: facts_differ || recovery_facts_differ,
    })
}

// ---- Collection into the return's totals ----

// Where one Other income amount lands while the income records are collected.
pub(crate) struct Ctx<'a> {
    pub totals: &'a mut crate::federal_rules::Totals,
    pub result: &'a mut crate::FederalResult,
    // The identity of the record that holds the amount (its review rows).
    pub id: i64,
    pub owner: usize,
    pub context: crate::IssueContext,
}

// One line 8z row of a record, named by what the amount is.
fn row(ctx: &mut Ctx<'_>, kind: &str, amount: i64, explanation: crate::Message) {
    crate::line_emission::line_cents(
        ctx.result,
        &alloc::format!("other_income_{}_{kind}", ctx.id),
        amount,
        &[&alloc::format!("income.{}", ctx.id)],
        "https://www.irs.gov/publications/p525",
        explanation,
    );
}

// Add `amount` (cents) of one Other income kind: a form's box that belongs to
// the kind, or an Other income record itself.
pub(crate) fn collect(ctx: &mut Ctx<'_>, kind: &OtherIncomeKind, amount: i64) {
    let owner = ctx.owner;
    let amount = match kind {
        OtherIncomeKind::AlaskaDividend => {
            ctx.totals.alaska += amount;
            Some(amount)
        }
        OtherIncomeKind::Prize { noncash, disposition } => {
            let taxable = prize_taxable(amount + noncash, disposition);
            ctx.totals.prizes += taxable;
            Some(taxable)
        }
        OtherIncomeKind::OlympicPrize => {
            ctx.totals.olympic += amount;
            ctx.totals.injured_olympic[owner] += amount;
            Some(amount)
        }
        OtherIncomeKind::DigitalAssets => {
            ctx.totals.digital += amount;
            Some(amount)
        }
        OtherIncomeKind::InmatePay => {
            ctx.totals.inmate_pay[owner] += amount;
            Some(amount)
        }
        OtherIncomeKind::HouseholdWages => {
            ctx.totals.household_wages[owner] += amount;
            if amount >= crate::year_constants::HOUSEHOLD_CASH_WAGE_THRESHOLD_CENTS {
                crate::federal_rules::note_once_scoped(
                    ctx.result,
                    crate::Message::new("NOTE_HOUSEHOLD_PAY_W2_DUE", Vec::new()),
                    ctx.context.clone(),
                );
            }
            Some(amount)
        }
        OtherIncomeKind::StrikeBenefits => {
            ctx.totals.strike_benefits[owner] += amount;
            Some(amount)
        }
        OtherIncomeKind::MedicaidWaiver { in_your_home: true } => {
            ctx.totals.medicaid_excluded[owner] += amount;
            ctx.totals.injured_other_amount(ctx.id, Some(amount));
            Some(-amount)
        }
        OtherIncomeKind::MedicaidWaiver { in_your_home: false } => {
            ctx.totals.medicaid_taxable[owner] += amount;
            Some(amount)
        }
        // Figured once for each prior year, beside that year's state refund.
        OtherIncomeKind::Recovery { .. } => {
            // The actual combined-year worksheet may later resolve this
            // source. Do not retain a stale unknown category before it runs.
            ctx.totals.injured_other_amount(ctx.id, None);
            return;
        }
        OtherIncomeKind::CardInsurance { premiums } => {
            let taxable = positive(amount - premiums);
            ctx.totals.other_8z += taxable;
            row(
                ctx,
                "card_insurance",
                taxable,
                crate::common::Message::new("EXPLAIN_OTHER_INCOME_N_CARD_INSURANCE", Vec::new()),
            );
            Some(taxable)
        }
        OtherIncomeKind::UnemploymentFund { contributions } => {
            let taxable = positive(amount - contributions);
            ctx.totals.other_8z += taxable;
            row(
                ctx,
                "unemployment_fund",
                taxable,
                crate::common::Message::new("EXPLAIN_OTHER_INCOME_N_UNEMPLOYMENT_FUND", Vec::new()),
            );
            Some(taxable)
        }
        OtherIncomeKind::Other { description } => {
            ctx.totals.other_8z += amount;
            row(
                ctx,
                "other",
                amount,
                crate::common::Message::new(
                    "EXPLAIN_OTHER_INCOME_N_OTHER",
                    alloc::vec![crate::common::MessageArg::Text(description.clone())],
                ),
            );
            Some(amount)
        }
    };
    ctx.totals.injured_other_amount(ctx.id, amount);
}

// A line 8z amount that is not one of the kinds (1099-MISC box 8 substitute
// payments, 1099-G box 5 RTAA payments).
pub(crate) fn collect_8z(ctx: &mut Ctx<'_>, what: &str, amount: i64, explanation: &str) {
    ctx.totals.other_8z += amount;
    ctx.totals.injured_other_amount(ctx.id, Some(amount));
    row(ctx, what, amount, crate::common::Message::new(explanation, Vec::new()));
}

// The review rows of the prior-year recovery worksheets and the totals they
// add: line 1 of Schedule 1 (the state refund share) and line 8z.
pub(crate) fn collect_years(
    d: &FederalDraft,
    totals: &mut crate::federal_rules::Totals,
    result: &mut crate::FederalResult,
) {
    use crate::injured_spouse::income_categories::IncomeCategory;
    if let Some(deductions) = &d.deductions {
        let mut vehicle_years = alloc::collections::BTreeMap::<
            i32,
            Vec<(&crate::VehicleLoanSource, &VehicleRecoveryYear)>,
        >::new();
        for loan in deductions.vehicle_loans.iter().filter(|loan| loan.model().refund > 0) {
            let origin = deductions
                .vehicle_recoveries
                .iter()
                .find(|row| row.id == loan.id)
                .map(|row| &row.origin);
            match origin {
                Some(crate::VehicleRefundOrigin::NotDeducted) => {}
                Some(crate::VehicleRefundOrigin::Deducted { prior }) => {
                    vehicle_years.entry(prior.year).or_default().push((loan, prior));
                }
                None => {
                    crate::require_localized!(
                        result,
                        false,
                        &crate::IssueContext::Section(crate::TaxSection::Deductions),
                        "Vehicle-interest refund",
                        "DOCUMENT_VEHICLE_REFUND_BENEFIT",
                        "Establish whether this refunded interest was deducted on the prior return's Schedule 1-A.",
                    );
                }
            }
        }
        for (year, sources) in vehicle_years {
            let prior = sources[0].1;
            let consistent = sources.iter().all(|(_, facts)| *facts == prior);
            let refund = i64::try_from(
                sources.iter().map(|(loan, _)| i128::from(loan.model().refund)).sum::<i128>(),
            )
            .ok();
            let taxable = refund
                .filter(|_| consistent)
                .and_then(|refund| PriorYearRecovery::vehicle_recovery(prior, refund));
            crate::require_localized!(
                result,
                taxable.is_some(),
                &crate::IssueContext::Section(crate::TaxSection::Deductions),
                "Vehicle-interest refund",
                "DOCUMENT_VEHICLE_REFUND_BENEFIT",
                "Establish the prior Schedule 1-A deduction's tax benefit using the same prior-return facts for every refund from that year. Unused credits, AMT, zero-rate gains or other dependent effects need that return's tax-benefit recomputation.",
            );
            if let Some(taxable) = taxable {
                totals.recovery_8z += taxable;
                crate::line_emission::line_cents(
                    result,
                    &alloc::format!("vehicle_interest_recovery_{year}"),
                    taxable,
                    &["deductions.vehicle_recoveries"],
                    "https://www.irs.gov/pub/irs-dft/f1098vli--dft.pdf",
                    crate::Message::new("EXPLAIN_VEHICLE_INTEREST_RECOVERY", Vec::new()),
                );
            }
        }
    }
    for year in worksheets(d) {
        // Record the actual combined worksheet outputs at the point the
        // one native owner assigns them. A stand-alone recovery preview
        // cannot explain the shared prior-year tax-benefit ceiling.
        if let Some(refund) = d.income.iter().find(|entry| {
            matches!(
                &entry.income, FederalIncome::GovernmentPayment { data }
                    if data.form.state_or_local_income_tax_refunds != 0
                        && crate::FederalTaxRefund::from(data).history.single_year() == Some(year.year)
            )
        }) {
            totals
                .injured_other_sources
                .entry(refund.id)
                .or_insert_with(|| Some(crate::kernel::ratio::exact::whole(0)));
            let owner = usize::from(refund.owner == crate::DocumentOwner::Spouses);
            totals.record_income_component(
                refund.id,
                owner,
                IncomeCategory::TaxableRefunds,
                Some(year.sheet.to_line_1),
                Some(year.sheet.to_line_1),
            );
            totals.injured_other_amount(refund.id, Some(year.sheet.to_line_1));
            totals.record_income_component(
                refund.id,
                owner,
                IncomeCategory::Other8z,
                Some(year.sheet.to_8z_taxes),
                Some(year.sheet.to_8z_taxes),
            );
            totals.injured_other_amount(refund.id, Some(year.sheet.to_8z_taxes));
        }
        let mut sources = d.income.iter().filter(|entry| {
            matches!(
                &entry.income, FederalIncome::OtherIncome { data }
                    if matches!(&data.kind, OtherIncomeKind::Recovery { worksheet }
                        if worksheet.ordinary && worksheet.year == year.year)
            )
        });
        if let Some(recovery) = sources.next()
            && sources.next().is_none()
            && !mortgage_recoveries(d).any(|(_, worksheet)| worksheet.year == year.year)
        {
            totals
                .injured_other_sources
                .insert(recovery.id, Some(crate::kernel::ratio::exact::whole(0)));
            totals.record_income_component(
                recovery.id,
                usize::from(recovery.owner == crate::DocumentOwner::Spouses),
                IncomeCategory::Other8z,
                Some(year.sheet.to_8z - year.sheet.to_8z_taxes),
                Some(year.sheet.to_8z - year.sheet.to_8z_taxes),
            );
            totals
                .injured_other_amount(recovery.id, Some(year.sheet.to_8z - year.sheet.to_8z_taxes));
        }
        totals.tax_refund += year.sheet.to_line_1;
        totals.recovery_8z += year.sheet.to_8z;
        totals.recovery_8z_taxes += year.sheet.to_8z_taxes;
        if year.sheet.to_8z != 0 || year.sheet.line_3 != 0 {
            crate::line_emission::line_cents(
                result,
                &alloc::format!("recovery_{}_taxable", year.year),
                year.sheet.line_11,
                &["income.other_income", "income.tax_refund"],
                "https://www.irs.gov/publications/p525",
                crate::common::Message::new("EXPLAIN_RECOVERY_N_TAXABLE", Vec::new()),
            );
        }
    }
}

// The issues of a Recovery record: one recovery item for each year, none
// beside a refund that spans several years, and the same Schedule A facts as
// the year's state refund.
pub(crate) fn recovery_issues(
    d: &FederalDraft,
    entry: &crate::FederalIncomeEntry,
    result: &mut crate::FederalResult,
    context: &crate::IssueContext,
    #[cfg(test)] source: &str,
) {
    let FederalIncome::OtherIncome { data } = &entry.income else {
        return;
    };
    let OtherIncomeKind::Recovery { worksheet } = &data.kind else {
        return;
    };
    let year = worksheet.year;
    crate::require_localized!(
        result,
        !d.income.iter().take_while(|other| other.id != entry.id).any(|other| {
            matches!(&other.income, FederalIncome::OtherIncome { data }
                if matches!(&data.kind, OtherIncomeKind::Recovery { worksheet } if worksheet.year == year))
        }),
        context,
        source,
        "ISSUE_RECOVERY_YEAR_TWICE",
        "Combine every refund or reimbursement of the same year's itemized deductions into one Other income recovery: Publication 525 Worksheet 2 is figured once for each year.",
    );
    crate::require_localized!(
        result,
        !d.income.iter().any(|other| {
            matches!(&other.income, FederalIncome::GovernmentPayment { data }
            if {
                let history = crate::FederalTaxRefund::from(data).history;
                data.form.state_or_local_income_tax_refunds != 0
                    && history.single_year().is_none() && history.deduction_years().contains(&year)
            })
        }),
        context,
        source,
        "ISSUE_RECOVERY_WITH_MULTIPLE_YEAR_REFUND",
        "A state or local tax refund allocated across several prior years cannot be figured with another recovery of the same year here; combine them on a review of that year's Worksheet 2.",
    );
    if let Some(found) = worksheets(d).into_iter().find(|w| w.year == year) {
        crate::require_localized!(
            result,
            !found.facts_differ,
            context,
            source,
            "ISSUE_RECOVERY_REFUND_YEAR_FACTS_DIFFER",
            "This recovery and the state or local tax refund of the same year state different itemized or standard deductions for that year. Both come from that year's return: correct whichever is wrong.",
        );
    }
}

impl PriorYearRecovery {
    // IRC 111 and the Form 1098-VLI box 5 instructions: refigure the
    // nonitemized deduction, rather than apply the Schedule A worksheet.
    pub(crate) fn vehicle_recovery(prior: &VehicleRecoveryYear, refund: i64) -> Option<i64> {
        if !prior.ordinary {
            return None;
        }
        use crate::kernel::ratio::dollars;
        let deduction = |interest| {
            crate::federal_rules::additional_deductions(
                dollars(prior.magi),
                prior.status,
                0,
                0,
                dollars(interest),
            )[2] * 100
        };
        let lost = positive(
            deduction(prior.eligible_interest)
                - deduction(positive(prior.eligible_interest - refund)),
        );
        Some(positive(lost + prior.taxable_income.min(0)))
    }
}
