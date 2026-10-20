// Coverage: Form 8606 special repayments (birth/adoption, emergency, domestic-abuse and
//   terminal-illness distributions); see retirement.rs's Form 8606 block. Form 8915-F
//   (qualified disaster distributions and their repayments) moved to disaster.rs (T3-8915,
//   2026-09-27), which owns its Coverage block; these repayments still exclude disaster
//   distributions, but Form 8606 line 6 also drops 2026 disaster repayments (disaster.rs
//   repaid_in_return_year, via IraBasis::adjusted_year_end).

use crate::federal_rules::valid_date;

use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{self, Exact};
use crate::kernel::ratio::fraction;
use crate::retirement::IraAmountIssue;
use crate::{
    FederalDraft, FederalIncome, FederalResult, IraBasis, Owner, RetirementException,
    RetirementKind, RetirementTaxable, RetirementTreatment,
};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Repayment of a special 2026 traditional IRA distribution
/// For qualified birth/adoption, emergency, domestic-abuse or terminal-illness distributions only. Keep the original gross distribution in Income and select the owner-level IRA basis worksheet. Repayments reduce taxable income under the separate Form 8606 worksheet; they are not regular contributions or ordinary 60-day rollovers.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IraRepayment {
    /// Original 2026 retirement Income entry ID
    /// Must belong to this IRA owner and use one of the four supported exceptions. Use one record for all repayments of this entry. Prior-year distributions and employer-plan, Roth or inherited sources need their separate treatment.
    #[libertas_number(min = 1)]
    pub income: i64,
    /// Actual deposits accepted as repayment
    /// Include later-year deposits only after they have actually occurred; these may require amending the original return. Never enter planned deposits. Amounts cannot exceed the original qualified gross amount.
    /// ----
    /// Deposit to this owner's eligible traditional IRA
    #[libertas_size(min = 1)]
    pub payments: Vec<IraRepaymentDeposit>,
    /// Eligible original distribution and accepted traditional IRA repayments confirmed
    /// Check the receiving trustee’s repayment acceptance records and original distribution statement.
    /// The receiving trustee accepted these as repayments of the linked distribution within its three-year period. No refunded or duplicated repayment, regular contribution, Roth conversion, disaster distribution or other repayment treatment. Include 2026 deposits in the actual December 31 value; do not adjust that value yourself. No credits none of these deposits as repayments, so the original distribution stays taxable.
    pub confirmed: bool,
}

/// Special IRA repayment deposit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IraRepaymentDeposit {
    /// Actual date deposited
    #[libertas_date_only]
    pub date: u32,
    /// Amount deposited
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amount: i64,
}
impl IraRepayment {
    pub(crate) fn valid(&self) -> bool {
        self.income > 0
            && !self.payments.is_empty()
            && self.payments.iter().all(|p| {
                valid_date(p.date) && (2026..=2029).contains(&(p.date / 10000)) && p.amount >= 0
            })
    }
    fn amount_exact(&self) -> Exact {
        deposit_amount(&self.payments)
    }
}
pub(crate) fn deposit_amount(payments: &[IraRepaymentDeposit]) -> Exact {
    payments.iter().map(|p| exact::whole(p.amount)).sum()
}
pub(crate) fn special(facts: &RetirementException) -> bool {
    matches!(
        facts,
        RetirementException::BirthAdoption { .. }
            | RetirementException::Emergency { .. }
            | RetirementException::DomesticAbuse { .. }
            | RetirementException::TerminalIllness { .. }
    )
}
impl IraBasis {
    // Only confirmed repayments; an unconfirmed deposit stays in the
    // year-end value like any other.
    pub(crate) fn repaid_in_year_exact(&self) -> Exact {
        self.special_repayments
            .iter()
            .filter(|p| p.confirmed)
            .flat_map(|p| &p.payments)
            .filter(|p| p.date / 10000 == 2026)
            .map(|p| exact::whole(p.amount))
            .sum()
    }
    // Form 8606 line 6: the December 31 value less 2026 repayments of 2026 special
    // distributions and of 2026 qualified disaster distributions.
    pub(crate) fn adjusted_year_end_exact(&self, d: &FederalDraft) -> Exact {
        (exact::whole(self.year_end)
            - self.repaid_in_year_exact()
            - exact::whole(crate::disaster::repaid_in_return_year(d, self.owner)))
        .max(exact::whole(0))
    }
}

pub(crate) fn calculate(
    d: &FederalDraft,
    r: &mut FederalResult,
    basis: &crate::retirement::BasisResult,
) -> Result<[i64; 2], IraAmountIssue> {
    let mut excluded = [0, 0];
    let Some(a) = &d.adjustments else {
        return Ok(excluded);
    };
    for b in &a.ira_basis {
        if b.special_repayments.is_empty() {
            continue;
        }
        let owner = usize::from(b.owner == Owner::Spouse);
        let mut valid_sources = true;
        for (i, repayment) in b.special_repayments.iter().enumerate() {
            let unique = !b.special_repayments[..i].iter().any(|p| p.income == repayment.income);
            valid_sources &= unique;
            crate::require_localized!(
                r,
                unique,
                &crate::IssueContext::Review,
                "Special IRA repayment",
                "ISSUE_CONFIRM_ACTUAL_ACCEPTED_REPAYMENTS_AND_USE_ONE_RECORD_PER_ORIGINAL_6D4178D6",
                "Use one repayment record per original Income entry.",
            );
            // No: nothing is credited; the distribution stays taxable.
            if !repayment.confirmed {
                continue;
            }
            let source = d.income.iter().find(|e| e.id == repayment.income && e.owner == b.owner);
            let distribution = source.and_then(|e| match &e.income {
                FederalIncome::Retirement { data }
                    if data.kind == RetirementKind::Ira
                        && data.taxable == RetirementTaxable::IraBasis =>
                {
                    match &data.ordinary_distribution {
                        RetirementTreatment::Exception { distributed, facts, .. }
                            if special(facts) =>
                        {
                            Some(*distributed)
                        }
                        _ => None,
                    }
                }
                _ => None,
            });
            let qualified = basis.gross.amounts.get(&repayment.income).map_or(0, |v| v.0);
            let valid = distribution.is_some_and(|date| {
                repayment.payments.iter().all(|p| p.date > date && p.date <= date + 30000)
            }) && repayment.amount_exact() <= exact::whole(qualified);
            valid_sources &= valid;
            crate::require_localized!(
                r,
                valid,
                &crate::IssueContext::Review,
                "Special IRA repayment",
                "ISSUE_LINK_A_SAME_OWNER_2026_TRADITIONAL_IRA_BIRTH_ADOPTION_EMERGENCY_DOM_D69B7407",
                "Link a same-owner 2026 traditional IRA birth/adoption, emergency, domestic-abuse or terminal-illness distribution. Repay after distribution and within three years, no more than its qualified gross amount.",
            );
        }
        if !valid_sources {
            // Keep the existing source diagnostics; do not report or credit an
            // amount from a mismatched original distribution.
            continue;
        }
        // The worksheet pools every applicable qualified distribution of this
        // owner's year, including those for which no repayment was made.
        let qualified: Exact = d
            .income
            .iter()
            .filter_map(|e| {
                if e.owner != b.owner {
                    return None;
                }
                match &e.income {
                    FederalIncome::Retirement { data } if data.kind == RetirementKind::Ira => {
                        match &data.ordinary_distribution {
                            RetirementTreatment::Exception { facts, .. } if special(facts) => {
                                basis.gross.amounts.get(&e.id).map(|v| exact::whole(v.0))
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            })
            .sum();
        let qualified = exact::dollars(&qualified).ok_or(IraAmountIssue)?;
        IraAmountIssue::reported(qualified)?;
        let gross = basis.regular_gross[owner];
        let pool = if gross == 0 {
            0
        } else {
            fraction(basis.regular_taxable[owner], phase::cap(qualified, [gross]), gross)
        };
        let credited = || b.special_repayments.iter().filter(|p| p.confirmed);
        let paid =
            exact::dollars(&credited().map(|p| p.amount_exact()).sum()).ok_or(IraAmountIssue)?;
        let reduction = phase::cap(paid, [pool]);
        excluded[owner] = reduction.checked_mul(100).ok_or(IraAmountIssue)?;
        let prefix = if owner == 0 { "taxpayer" } else { "spouse" };
        let rows = [
            (
                "year_end",
                exact::dollars(&b.adjusted_year_end_exact(d)).ok_or(IraAmountIssue)?,
                msg!("EXPLAIN_N_IRA_REPAYMENT_N_YEAR_END"),
            ),
            ("qualified_gross", qualified, msg!("EXPLAIN_N_IRA_REPAYMENT_N_QUALIFIED_GROSS")),
            ("qualified_taxable", pool, msg!("EXPLAIN_N_IRA_REPAYMENT_N_QUALIFIED_TAXABLE")),
            ("income_reduction", reduction, msg!("EXPLAIN_N_IRA_REPAYMENT_N_INCOME_REDUCTION")),
            (
                "taxable_after",
                basis.regular_taxable[owner] - reduction,
                msg!("EXPLAIN_N_IRA_REPAYMENT_N_TAXABLE_AFTER"),
            ),
            (
                "later_deposits",
                exact::dollars(
                    &credited()
                        .flat_map(|p| &p.payments)
                        .filter(|p| p.date / 10000 > 2026)
                        .map(|p| exact::whole(p.amount))
                        .sum(),
                )
                .ok_or(IraAmountIssue)?,
                msg!("EXPLAIN_N_IRA_REPAYMENT_N_LATER_DEPOSITS"),
            ),
        ];
        for (_, amount, _) in &rows {
            IraAmountIssue::reported(*amount)?;
        }
        emit_lines(r, &mut (), rows, |(suffix, amount, note), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("{prefix}_ira_repayment_{suffix}"),
                LineValue::MoneyDollars(amount),
                &["adjustments.ira_basis.special_repayments", "income.retirement"],
                "https://www.irs.gov/instructions/i8606",
                note,
            ))
        });
        // Line 6 of the 2026 Line 15c Worksheet: repayments above this year's taxable amount are
        // basis for next year's Form 8606 line 2 (the repayment is not income this year).
        if paid > reduction {
            IraAmountIssue::reported(paid - reduction)?;
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new(
                    "NOTE_LINE_15C_NEXT_YEAR_BASIS",
                    alloc::vec![
                        crate::MessageArg::Nested(alloc::boxed::Box::new(
                            crate::IssueContext::owner(crate::TaxSection::Retirement, b.owner)
                                .label()
                        )),
                        crate::MessageArg::usd_dollars(paid - reduction)
                    ],
                ),
                crate::IssueContext::owner(crate::TaxSection::Retirement, b.owner),
            );
        }
    }
    Ok(excluded)
}
