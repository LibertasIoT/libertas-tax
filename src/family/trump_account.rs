// 2026 Form 5329 Parts X and XI; section 530A(d)(5) and 4973(b).
// Coverage: Form 5329 Parts X-XI (Trump accounts).
//   Lines 56-63 of the 2026 draft Form 5329 (line 56 is zero for 2026; line 61 6% to Schedule 2
//   line 18; line 63 100% of 2026 returned-excess earnings to Schedule 2 line 5). The $5,000
//   limit excludes pilot (section 6434), qualified general and rollover contributions; section 128
//   employer deposits count. Page: the Other taxes list's Trump account item
//   (OtherTax::TrumpAccounts, screening.trump_accounts, rows by owner), both-sides rules
//   TRUMP_EXEMPT_ABOVE_CONTRIBUTIONS and TRUMP_LOSS_ABOVE_PRINCIPAL. Not calculated here: the
//   Form 4547 election (no tax effect; a filing item deferred with printing). The W-2
//   owner handles code TA separately. Printed 1099-R boxes 1 and 7d bind corrective
//   returns by source identity; personal receipt and deadline facts stay on Other taxes.
//   Tests: tests/family/other_taxes.rs and tests/federal/printed_form_boxes.rs.
//   trump_account_excess_through_the_other_taxes_page,
//   trump_account_five_thousand_dollar_limit_boundary, trump_account_page_rules_are_checked_on_save.
// Form8379 line18 receives each beneficiary's final line61/63 tax, retaining
// the native annual rounding. T/injured_spouse.rs unit13_trump_account_excess.
use crate::federal_rules::valid_date;
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::line_emission::emit as emit_lines;
use crate::{FederalPeople, FederalResult, FederalStatus, Owner};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Trump-account value included after a beneficiary's death during the growth period
/// Written from a Form 1099-R with code 4 and box 7c checked, on the acquirer's own return. If the beneficiary's estate acquired the account, it goes on the beneficiary's 2026 final return. The account ceases to be both a Trump account and an IRA on the death date.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TrumpDeathAcquisition {
    /// Additional printed retirement-statement boxes
    pub form_boxes: crate::RetirementPrintedBoxes,
    /// Form 1099-R box 2a taxable amount as printed
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_2a: i64,
    /// Form 1099-R box 3 capital gain
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_3: i64,
    /// Form 1099-R box 6 net unrealized appreciation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_6: i64,
    /// Form 1099-R box 8a other
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_8a: i64,
    /// Form 1099-R box 8b share
    /// [NullText]
    /// Box 8b is blank
    /// [SetText]
    /// Printed annuity-contract share
    #[libertas_number(min = 0.01, max = 99.99, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub box_8b_percent: Option<f64>,
    /// Form 1099-R box 9a share
    /// [NullText]
    /// Box 9a is blank
    /// [SetText]
    /// Printed distribution share
    #[libertas_number(min = 0.01, max = 99.99, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub box_9a_percent: Option<f64>,
    /// Form 1099-R box 9b employee contributions
    /// [NullText]
    /// Box 9b is blank
    /// [SetText]
    /// Printed employee contributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_9b: Option<i64>,
    /// Form 1099-R box 10 in-plan Roth rollover amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_10: i64,
    /// Form 1099-R box 7a distribution codes as printed
    #[libertas_bitflags(crate::DistributionCodes)]
    pub box7: u32,
    /// Form 1099-R box 7b IRA/SEP/SIMPLE as printed
    pub ira: bool,
    /// Form 1099-R box 7c Trump account as printed
    pub trump: bool,

    /// The beneficiary's estate acquired the account
    /// Yes only on the deceased beneficiary's own final return. Otherwise this income belongs on the individual acquirer's return, not the beneficiary's or the estate's return.
    pub beneficiary_estate: bool,
    /// Account fair market value on the beneficiary's date of death — Form 1099-R box 1
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub value: i64,
    /// Account basis on the date of death
    /// Use the trustee's basis record. Pilot, qualified general and section 128 employer deposits do not create basis; qualifying other-source deposits and transferred basis do. Basis above value produces no deductible loss here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    /// [NullText]
    /// Trustee basis not known
    /// [SetText]
    /// Actual trustee basis
    #[libertas_default($null)]
    pub basis: Option<i64>,
    /// Federal income tax withheld on the death distribution, if any
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for TrumpDeathAcquisition {}

// The 1099-R onto the internal death acquisition: code 4 the acquirer's income at the child's
// death (section 530A(d)(6)), box 1 the value and box 2a its excess over the basis.
impl From<&crate::R1099Source> for TrumpDeathAcquisition {
    fn from(source: &crate::R1099Source) -> Self {
        let crate::R1099Source {
            form,
            code,
            // A Trump account row asks only its own code.
            taxable_method: _,
            disability: _,
            box_10_basis: _,
            disaster: _,
            after_death: _,
            ira_basis: _,
        } = source;
        let Some(crate::DistributionCode::TrumpDeath { estate, basis }) = code else {
            unreachable!("only a Trump account death code files a 1099-R as an acquisition")
        };
        let crate::R1099 {
            payer: _,
            payer_telephone: _,
            account_number: _,
            recipient: _,
            gross,
            taxable,
            // Printed 2b, 5, 7d, 11, 12 and 13 are retained through form_boxes.
            taxable_not_determined: _,
            total_distribution: _,
            box_3,
            withholding,
            employee_contributions: _,
            box_6,
            box7,
            ira,
            trump,
            excess_contribution_earnings: _,
            box_8a,
            box_8b_percent,
            box_9a_percent,
            box_9b,
            box_10,
            first_roth_year: _,
            fatca: _,
            payment_date: _,
            state_details: _,
        } = form;
        Self {
            form_boxes: form.form_boxes(),
            box_2a: taxable.unwrap_or(0),
            box_3: box_3.unwrap_or(0),
            box_6: box_6.unwrap_or(0),
            box_8a: box_8a.unwrap_or(0),
            box_8b_percent: *box_8b_percent,
            box_9a_percent: *box_9a_percent,
            box_9b: *box_9b,
            box_10: box_10.unwrap_or(0),
            box7: *box7,
            ira: *ira,
            trump: *trump,
            beneficiary_estate: *estate,
            value: *gross,
            basis: *basis,
            withholding: withholding.unwrap_or(0),
        }
    }
}

impl TrumpDeathAcquisition {
    pub(crate) fn valid(&self) -> bool {
        [self.box_2a, self.box_3, self.box_6, self.box_8a, self.box_10, self.withholding]
            .into_iter()
            .all(|v| (0..=100_000_000_000).contains(&v))
            && self.box_9b.is_none_or(|v| (0..=100_000_000_000).contains(&v))
            && crate::lump_sum::percent_valid(self.box_8b_percent)
            && crate::lump_sum::percent_valid(self.box_9a_percent)
            && (0..=100_000_000_000).contains(&self.value)
            && self.basis.is_none_or(|basis| (0..=100_000_000_000).contains(&basis))
            && (0..=100_000_000_000).contains(&self.withholding)
    }

    pub(crate) fn taxable(&self) -> Option<i64> {
        self.basis.map(|basis| positive(self.value - basis))
    }
}

/// Qualified 2026 Trump-account transfer to the same beneficiary's ABLE account
/// The trustee transfers the entire source balance directly in the calendar year the beneficiary turns 17. Form 1099-R reports the gross transfer with zero taxable income; the receiving ABLE program reports its own rollover. This is not an annual ABLE contribution.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TrumpAbleRollover {
    /// Additional printed retirement-statement boxes
    pub form_boxes: crate::RetirementPrintedBoxes,
    /// Form 1099-R box 2a taxable amount as printed
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_2a: i64,
    /// Form 1099-R box 3 capital gain
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_3: i64,
    /// Form 1099-R box 6 net unrealized appreciation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_6: i64,
    /// Form 1099-R box 8a other
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_8a: i64,
    /// Form 1099-R box 8b share
    /// [NullText]
    /// Box 8b is blank
    /// [SetText]
    /// Printed annuity-contract share
    #[libertas_number(min = 0.01, max = 99.99, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub box_8b_percent: Option<f64>,
    /// Form 1099-R box 9a share
    /// [NullText]
    /// Box 9a is blank
    /// [SetText]
    /// Printed distribution share
    #[libertas_number(min = 0.01, max = 99.99, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub box_9a_percent: Option<f64>,
    /// Form 1099-R box 9b employee contributions
    /// [NullText]
    /// Box 9b is blank
    /// [SetText]
    /// Printed employee contributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_9b: Option<i64>,
    /// Form 1099-R box 10 in-plan Roth rollover amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_10: i64,
    /// Form 1099-R box 7a distribution codes as printed
    #[libertas_bitflags(crate::DistributionCodes)]
    pub box7: u32,
    /// Form 1099-R box 7b IRA/SEP/SIMPLE as printed
    pub ira: bool,
    /// Form 1099-R box 7c Trump account as printed
    pub trump: bool,
    /// Form 1099-R box 4 federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,

    /// Entire source-account balance transferred — Form 1099-R box 1
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Direct trustee-to-trustee transfer date in 2026
    #[libertas_date_only]
    pub transferred: u32,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for TrumpAbleRollover {}

// The 1099-R onto the internal transfer: code G the whole account moved to the child's ABLE
// account (section 530A(d)(4)).
impl From<&crate::R1099Source> for TrumpAbleRollover {
    fn from(source: &crate::R1099Source) -> Self {
        let crate::R1099Source {
            form,
            code,
            // A Trump account row asks only its own code.
            taxable_method: _,
            disability: _,
            box_10_basis: _,
            disaster: _,
            after_death: _,
            ira_basis: _,
        } = source;
        let Some(crate::DistributionCode::TrumpToAble { transferred }) = code else {
            unreachable!("only a Trump account ABLE code files a 1099-R as a transfer")
        };
        let crate::R1099 {
            payer: _,
            payer_telephone: _,
            account_number: _,
            recipient: _,
            gross,
            taxable,
            // Printed 2b, 5, 7d, 11, 12 and 13 are retained through form_boxes.
            taxable_not_determined: _,
            total_distribution: _,
            box_3,
            withholding,
            employee_contributions: _,
            box_6,
            box7,
            ira,
            trump,
            excess_contribution_earnings: _,
            box_8a,
            box_8b_percent,
            box_9a_percent,
            box_9b,
            box_10,
            first_roth_year: _,
            fatca: _,
            payment_date: _,
            state_details: _,
        } = form;
        Self {
            form_boxes: form.form_boxes(),
            box_2a: taxable.unwrap_or(0),
            box_3: box_3.unwrap_or(0),
            box_6: box_6.unwrap_or(0),
            box_8a: box_8a.unwrap_or(0),
            box_8b_percent: *box_8b_percent,
            box_9a_percent: *box_9a_percent,
            box_9b: *box_9b,
            box_10: box_10.unwrap_or(0),
            box7: *box7,
            ira: *ira,
            trump: *trump,
            withholding: withholding.unwrap_or(0),
            amount: *gross,
            transferred: *transferred,
        }
    }
}

impl TrumpAbleRollover {
    pub(crate) fn valid(&self) -> bool {
        [self.box_2a, self.box_3, self.box_6, self.box_8a, self.box_10, self.withholding]
            .into_iter()
            .all(|v| (0..=100_000_000_000).contains(&v))
            && self.box_9b.is_none_or(|v| (0..=100_000_000_000).contains(&v))
            && crate::lump_sum::percent_valid(self.box_8b_percent)
            && crate::lump_sum::percent_valid(self.box_9a_percent)
            && (1..=100_000_000_000).contains(&self.amount)
            && valid_date(self.transferred)
            && (20260704..=20261231).contains(&self.transferred)
    }
}

pub use crate::trump_account_source::{TrumpAccount, TrumpExcessReturn};
pub(crate) use crate::trump_account_source::{accounts_issue, valid_accounts};

/// One account beneficiary's 2026 Trump-account contributions
/// Use only on the beneficiary's own return, not on a parent's return for a dependent. The pilot payment, qualified general contributions and qualified rollovers are exempt from the $5,000 annual limit; eligible section 128 employer deposits are not. Contributions to another person's account do not create this return's Form 5329 tax.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TrumpAccountView {
    /// Account beneficiary on this return
    #[libertas_default(Taxpayer)]
    #[libertas_constrained_by("$.allowed_owners")]
    pub owner: Owner,
    /// Total contributions accepted in 2026
    /// Include original principal before the corrective returns below. Count a full trustee-to-trustee rollover accepted by a successor Trump account as another accepted contribution; its entire incoming amount also belongs in the exempt amount below. Contributions could not begin before July 4, 2026.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub contributions: i64,
    /// Contributions exempt from the $5,000 limit
    /// Include only section 6434 pilot payments, qualified general contributions and qualified rollover contributions. A qualified Trump-to-Trump rollover is a direct trustee transfer of the entire source balance for the same beneficiary; it is not a Form 1099-R distribution. Include the incoming rollover once here and once in total accepted contributions above, even if it contains earlier 2026 deposits. Section 128 employer contributions count toward the limit and stay in the total above, not here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub exempt: i64,
    /// Completed returns of excess principal and attributable income
    /// Enter a correction received in 2026 or later, including after the documented 2026-return filing deadline. Only principal returned by that deadline reduces 2026 excess; later distributions and their earnings belong to their receipt-year return.
    pub returns: Vec<TrumpExcessReturn>,
    /// Complete contribution and return records confirmed
    /// Confirm this beneficiary's eligible Trump account, all accepted contributions after July 3, 2026, the exempt-source classification, all corrective distributions and their Forms 1099-R. An employer's section 128 wage exclusion belongs on the contributing employee's return, which can be a different person's return. If the beneficiary died during 2026, enter the final-return death facts and a separate death-distribution income source on the acquirer's return, or the beneficiary's final return when the estate acquired it. A nonqualified account or another distribution requires separate treatment. The trustee's statements and Forms 1099-R show it. No stops the estimate: the Form 5329 additional taxes are not calculated without these records.
    pub confirmed: bool,
}

impl From<&TrumpAccount> for TrumpAccountView {
    fn from(value: &TrumpAccount) -> Self {
        Self {
            owner: value.owner,
            contributions: value.contributions,
            exempt: value.exempt,
            returns: value.returns.clone(),
            confirmed: value.confirmed,
        }
    }
}

impl From<&TrumpAccountView> for TrumpAccount {
    fn from(value: &TrumpAccountView) -> Self {
        Self {
            owner: value.owner,
            contributions: value.contributions,
            exempt: value.exempt,
            returns: value.returns.clone(),
            confirmed: value.confirmed,
        }
    }
}

// Dollars on Form 5329, after the annual form-line rounding.
pub(crate) fn calculate(
    values: &[TrumpAccount],
    people: &FederalPeople,
    status: FederalStatus,
    r: &mut FederalResult,
) -> ([i64; 2], [i64; 2]) {
    let mut excess_tax = [0; 2];
    let mut returned_earnings_tax = [0; 2];
    for account in values {
        let person = if account.owner == Owner::Taxpayer {
            Some(&people.taxpayer)
        } else if status == FederalStatus::Joint {
            people.spouse.as_ref()
        } else {
            None
        };
        let supported = account.valid()
            && account.confirmed
            && person.is_some_and(|p| {
                // Under age 18 at year end (the message below), day-exact: a
                // beneficiary born January 1, 2009 attained 18 on December 31, 2026.
                crate::federal_model::age_at_year_end(p.birth_date) < 18
                    && p.death_date().is_none_or(|date| {
                        date >= 20260704 && account.returns.iter().all(|v| v.received <= date)
                    })
            })
            && account.returns.iter().all(|v| v.confirmed);
        crate::require_localized!(
            r,
            supported,
            &crate::IssueContext::Section(crate::TaxSection::OtherTaxes),
            "Trump account",
            "ISSUE_USE_THE_ACCOUNT_BENEFICIARY_S_OWN_RETURN_YEAR_RETURN_CONFIRM_THEY_WOULD_BE_11017AE6",
            "Use the account beneficiary's own 2026 return, confirm they would be under age 18 at year-end, and enter complete qualified-account and excess-distribution records. A deceased beneficiary needs final-return facts, with all corrections before death and any death-date income reported separately on the proper return. A distribution after the 2026-return deadline does not reduce this year's excess.",
        );
        if !supported {
            continue;
        }
        let limit_excess = positive(
            account.contributions
                - account.exempt
                - crate::year_constants::TRUMP_ACCOUNT_ANNUAL_LIMIT_CENTS,
        );
        let total_returned: i64 = account.returns.iter().map(|v| v.principal).sum();
        let returned: i64 =
            account.returns.iter().filter(|v| v.received <= v.deadline).map(|v| v.principal).sum();
        crate::require_localized!(
            r,
            total_returned <= limit_excess,
            &crate::IssueContext::Section(crate::TaxSection::OtherTaxes),
            "Trump account",
            "ISSUE_CORRECTIVE_PRINCIPAL_CANNOT_EXCEED_THIS_BENEFICIARY_S_NONEXEMPT_202_9A2F2791",
            "Corrective principal cannot exceed this beneficiary's nonexempt 2026 contributions above the $5,000 limit.",
        );
        if total_returned > limit_excess {
            continue;
        }
        let label = if account.owner == Owner::Taxpayer { "taxpayer" } else { "spouse" };
        if total_returned > returned {
            r.review_notes.push(crate::federal_model::ReviewNote::localized(
                #[cfg(test)] alloc::format!("Trump account — {label}: ${} of excess principal was returned after the 2026-return deadline. It does not reduce 2026 Form 5329 line 60 or the 6% tax; review the receipt-year distribution and earnings separately.", dollars(total_returned - returned)),
                crate::Message::new(
                    if account.owner == Owner::Taxpayer {
                        "NOTE_TRUMP_ACCOUNT_LATE_EXCESS_TAXPAYER"
                    } else {
                        "NOTE_TRUMP_ACCOUNT_LATE_EXCESS_SPOUSE"
                    },
                    alloc::vec![crate::MessageArg::usd_dollars(dollars(total_returned - returned))],
                ),
            ));
        }
        let remaining = limit_excess - returned;
        let owner_excess_tax = fraction(dollars(remaining), 6, 100);
        let earnings_return_year: i64 = account
            .returns
            .iter()
            .filter(|v| v.received / 10000 == 2026)
            .map(|v| positive(v.earnings))
            .sum();
        let owner_earnings_tax = dollars(earnings_return_year);
        let owner = usize::from(account.owner == Owner::Spouse);
        excess_tax[owner] += owner_excess_tax;
        returned_earnings_tax[owner] += owner_earnings_tax;
        emit_lines(
            r,
            &mut (),
            [
                (
                    "line57_excess",
                    dollars(limit_excess),
                    msg!("EXPLAIN_TRUMP_ACCOUNT_N_N_LINE57_EXCESS"),
                ),
                (
                    "line59_returned",
                    dollars(returned),
                    msg!("EXPLAIN_TRUMP_ACCOUNT_N_N_LINE59_RETURNED"),
                ),
                (
                    "line60_remaining",
                    dollars(remaining),
                    msg!("EXPLAIN_TRUMP_ACCOUNT_N_N_LINE60_REMAINING"),
                ),
                ("line61_tax", owner_excess_tax, msg!("EXPLAIN_TRUMP_ACCOUNT_N_N_LINE61_TAX")),
                (
                    "line62_earnings",
                    owner_earnings_tax,
                    msg!("EXPLAIN_TRUMP_ACCOUNT_N_N_LINE62_EARNINGS"),
                ),
                ("line63_tax", owner_earnings_tax, msg!("EXPLAIN_TRUMP_ACCOUNT_N_N_LINE63_TAX")),
            ],
            |(suffix, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("trump_account_{label}_{suffix}"),
                    LineValue::MoneyDollars(amount),
                    &["screening.trump_accounts"],
                    "https://www.irs.gov/pub/irs-dft/f5329--dft.pdf",
                    explanation,
                ))
            },
        );
    }
    (excess_tax, returned_earnings_tax)
}

/// Receipt and correction facts for a printed Trump-account excess return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TrumpExcessDocumentView {
    /// Accepted Form 1099-R identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub id: i64,
    /// Printed payer
    #[libertas_read_only]
    pub label: alloc::string::String,
    /// Printed box 1 excess-contribution principal
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub principal: i64,
    /// Printed box 7d earnings
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub earnings: i64,
    /// Date the corrective distribution was received
    /// [NullText]
    /// Not answered yet
    /// [SetText]
    /// The corrective distribution was received on
    #[libertas_date_only]
    #[libertas_default($null)]
    pub received: Option<u32>,
    /// Applicable return due date, including extension or relief
    /// [NullText]
    /// Not answered yet
    /// [SetText]
    /// The applicable return deadline was
    #[libertas_date_only]
    #[libertas_default($null)]
    pub deadline: Option<u32>,
    /// The custodian's corrective distribution is confirmed
    pub confirmed: bool,
}

pub(crate) fn printed_excess(entry: &crate::FederalIncomeEntry) -> Option<(i64, i64)> {
    match &entry.income {
        crate::FederalIncome::Retirement { data: source }
            if {
                let data = &crate::FederalRetirement::from(source);
                data.trump && data.box7 == crate::DistributionCodes::CODE_8.0
            } =>
        {
            let data = &crate::FederalRetirement::from(source);
            Some((data.gross, data.form_boxes.excess_contribution_earnings))
        }
        _ => None,
    }
}

// Project retained source identities on each calculation and document edit.
// Receipt dates and due dates remain the account owner's independent facts.
pub(crate) fn project_accounts(d: &crate::FederalDraft) -> Vec<TrumpAccount> {
    let mut accounts = d.screening.as_ref().map_or_else(Vec::new, |s| s.trump_accounts.clone());
    for account in &mut accounts {
        account.returns.retain_mut(|payment| {
            let Some(id) = payment.document_id else {
                return true;
            };
            let Some(entry) = d.income.iter().find(|entry| {
                entry.id == id && entry.owner == crate::DocumentOwner::from(account.owner)
            }) else {
                return false;
            };
            let Some((principal, earnings)) = printed_excess(entry) else {
                return false;
            };
            payment.principal = principal;
            payment.earnings = earnings;
            true
        });
    }
    accounts
}
