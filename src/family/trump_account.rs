// 2026 Form 5329 Parts X and XI; section 530A(d)(5) and 4973(b).
// Coverage: Form 5329 Parts X-XI (Trump accounts) — Supported (audited 2026-09-28, u5-credits).
//   Lines 56-63 of the 2026 draft Form 5329 (line 56 is zero for 2026; line 61 6% to Schedule 2
//   line 18; line 63 100% of 2026 returned-excess earnings to Schedule 2 line 5). The $5,000
//   limit excludes pilot (section 6434), qualified general and rollover contributions; section 128
//   employer deposits count. Page: the Other taxes list's Trump account item
//   (OtherTax::TrumpAccounts, screening.trump_accounts, rows by owner), both-sides rules
//   TRUMP_EXEMPT_ABOVE_CONTRIBUTIONS and TRUMP_LOSS_ABOVE_PRINCIPAL. Not calculated here: the
//   Form 4547 election (no tax effect; a filing item deferred with printing) and W-2 box 12
//   code TA (refused on the W-2). Tests: T/other_taxes.rs
//   trump_account_excess_through_the_other_taxes_page,
//   trump_account_five_thousand_dollar_limit_boundary, trump_account_page_rules_are_checked_on_save.
// Form8379 line18 receives each beneficiary's final line61/63 tax, retaining
// the native annual rounding. T/injured_spouse.rs unit13_trump_account_excess.
use crate::federal_rules::valid_date;
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::{FederalPeople, FederalResult, FederalStatus, Owner};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Trump-account value included after a beneficiary's death during the growth period
/// Written from a Form 1099-R with code 4 and box 7c checked, on the acquirer's own return. If the beneficiary's estate acquired the account, it goes on the beneficiary's 2026 final return. The account ceases to be both a Trump account and an IRA on the death date.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TrumpDeathAcquisition {
    /// The beneficiary's estate acquired the account
    /// Yes only on the deceased beneficiary's own final return. Otherwise this income belongs on the individual acquirer's return, not the beneficiary's or the estate's return.
    pub beneficiary_estate: bool,
    /// Account fair market value on the beneficiary's date of death — Form 1099-R box 1
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub value: i64,
    /// Account basis on the date of death
    /// Use the trustee's basis record. Pilot, qualified general and section 128 employer deposits do not create basis; qualifying other-source deposits and transferred basis do. Basis above value produces no deductible loss here.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub basis: i64,
    /// Federal income tax withheld on the death distribution, if any
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub withholding: i64,
}

impl TrumpDeathAcquisition {
    pub(crate) fn valid(&self) -> bool {
        (0..=100_000_000_000).contains(&self.value)
            && (0..=100_000_000_000).contains(&self.basis)
            && (0..=100_000_000_000).contains(&self.withholding)
    }

    pub(crate) fn taxable(&self) -> i64 {
        positive(self.value - self.basis)
    }
}

/// Qualified 2026 Trump-account transfer to the same beneficiary's ABLE account
/// The trustee transfers the entire source balance directly in the calendar year the beneficiary turns 17. Form 1099-R reports the gross transfer with zero taxable income; the receiving ABLE program reports its own rollover. This is not an annual ABLE contribution.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TrumpAbleRollover {
    /// Entire source-account balance transferred — Form 1099-R box 1
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub amount: i64,
    /// Direct trustee-to-trustee transfer date in 2026
    #[libertas_date_only]
    pub transferred: u32,
}

impl TrumpAbleRollover {
    pub(crate) fn valid(&self) -> bool {
        (1..=100_000_000_000).contains(&self.amount)
            && valid_date(self.transferred)
            && (20260704..=20261231).contains(&self.transferred)
    }
}

/// One account beneficiary's 2026 Trump-account contributions
/// Use only on the beneficiary's own return, not on a parent's return for a dependent. The pilot payment, qualified general contributions and qualified rollovers are exempt from the $5,000 annual limit; eligible section 128 employer deposits are not. Contributions to another person's account do not create this return's Form 5329 tax.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TrumpAccount {
    /// Account beneficiary on this return
    #[libertas_default(Taxpayer)]
    #[libertas_constrained_by("$.allowed_owners")]
    pub owner: Owner,
    /// Total contributions accepted in 2026
    /// Include original principal before the corrective returns below. Count a full trustee-to-trustee rollover accepted by a successor Trump account as another accepted contribution; its entire incoming amount also belongs in the exempt amount below. Contributions could not begin before July 4, 2026.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub contributions: i64,
    /// Contributions exempt from the $5,000 limit
    /// Include only section 6434 pilot payments, qualified general contributions and qualified rollover contributions. A qualified Trump-to-Trump rollover is a direct trustee transfer of the entire source balance for the same beneficiary; it is not a Form 1099-R distribution. Include the incoming rollover once here and once in total accepted contributions above, even if it contains earlier 2026 deposits. Section 128 employer contributions count toward the limit and stay in the total above, not here.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub exempt: i64,
    /// Completed returns of excess principal and attributable income
    /// Enter a correction received in 2026 or later, including after the documented 2026-return filing deadline. Only principal returned by that deadline reduces 2026 excess; later distributions and their earnings belong to their receipt-year return.
    pub returns: Vec<TrumpExcessReturn>,
    /// Complete contribution and return records confirmed
    /// Confirm this beneficiary's eligible Trump account, all accepted contributions after July 3, 2026, the exempt-source classification, all corrective distributions and their Forms 1099-R. An employer's section 128 wage exclusion belongs on the contributing employee's return, which can be a different person's return. If the beneficiary died during 2026, enter the final-return death facts and a separate death-distribution income source on the acquirer's return, or the beneficiary's final return when the estate acquired it. A nonqualified account or another distribution requires separate treatment. The trustee's statements and Forms 1099-R show it. No stops the estimate: the Form 5329 additional taxes are not calculated without these records.
    pub confirmed: bool,
}

/// Returned excess Trump-account contribution
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TrumpExcessReturn {
    /// Excess contribution principal, Form 1099-R box 1
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub principal: i64,
    /// Net income attributable to the returned excess
    /// A loss reduces cash returned but cannot exceed principal. Positive 2026 Form 1099-R box 7d income incurs the separate 100% additional tax in 2026; a 2027 return belongs to 2027 instead.
    #[libertas_money("USD", 2)]
    pub earnings: i64,
    /// Date the completed distribution was received
    #[libertas_date_only]
    pub received: u32,
    /// Applicable 2026-return due date, including any documented extension or relief
    #[libertas_date_only]
    pub deadline: u32,
    /// The custodian's excess-contribution distribution and income allocation are confirmed
    /// The custodian's corrective-distribution statement shows it. No stops the estimate: the Form 5329 additional taxes are not calculated without it.
    pub confirmed: bool,
}

impl TrumpExcessReturn {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (i128::from(self.earnings) + i128::from(self.principal) < 0).then(|| {
            crate::validation::ValidationIssue::new("TRUMP_LOSS_ABOVE_PRINCIPAL")
                .at_field("earnings")
        })
    }
    fn valid(&self) -> bool {
        (1..=100_000_000_000).contains(&self.principal)
            && (-100_000_000_000..=100_000_000_000).contains(&self.earnings)
            && self.issue().is_none()
            && valid_date(self.received)
            && valid_date(self.deadline)
            && (20260704..=20281231).contains(&self.received)
            && (20270101..=20281231).contains(&self.deadline)
    }
}
impl TrumpAccount {
    fn contribution_issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.exempt > self.contributions).then(|| {
            crate::validation::ValidationIssue::new("TRUMP_EXEMPT_ABOVE_CONTRIBUTIONS")
                .at_field("exempt")
        })
    }
    fn returns_issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.returns.iter().enumerate().find_map(|(index, returned)| {
            returned.issue().map(|issue| issue.in_list("returns", index))
        })
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.contribution_issue().or_else(|| self.returns_issue())
    }
    pub(crate) fn valid(&self) -> bool {
        (0..=100_000_000_000).contains(&self.contributions)
            && (0..=100_000_000_000).contains(&self.exempt)
            && self.issue().is_none()
            && self.returns.len() <= 100
            && self.returns.iter().all(TrumpExcessReturn::valid)
    }
}
// Preserve the normal Save's group-wide failure order, then keep the retained
// shape policy in the existing valid_accounts owner.
pub(crate) fn accounts_issue(
    values: &[TrumpAccount],
) -> Option<crate::validation::ValidationIssue> {
    values
        .iter()
        .enumerate()
        .find_map(|(index, account)| {
            account.contribution_issue().map(|issue| issue.in_list("accounts", index))
        })
        .or_else(|| {
            values.iter().enumerate().find_map(|(index, account)| {
                account.returns_issue().map(|issue| issue.in_list("accounts", index))
            })
        })
        .or_else(|| {
            values.iter().enumerate().find_map(|(index, account)| {
                values[..index].iter().any(|prior| prior.owner == account.owner).then(|| {
                    crate::validation::ValidationIssue::new("TRUMP_ACCOUNT_TWICE")
                        .at_field("owner")
                        .in_list("accounts", index)
                })
            })
        })
}

pub(crate) fn valid_accounts(values: &[TrumpAccount]) -> bool {
    values
        .iter()
        .enumerate()
        .all(|(i, v)| v.valid() && !values[..i].iter().any(|old| old.owner == v.owner))
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
