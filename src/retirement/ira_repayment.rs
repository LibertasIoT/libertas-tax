pub use crate::ira_repayment_sources::{IraRepayment, IraRepaymentDeposit};
pub(crate) use crate::retirement::special;
// Coverage: Form 8606 special repayments (birth/adoption, emergency, domestic-abuse and
//   terminal-illness distributions); see retirement.rs's Form 8606 block. Form 8915-F
//   (qualified disaster distributions and their repayments) moved to disaster.rs (T3-8915,
//   2026-09-27), which owns its Coverage block; these repayments still exclude disaster
//   distributions, but Form 8606 line 6 also drops 2026 disaster repayments (disaster.rs
//   repaid_in_return_year, via IraBasis::adjusted_year_end).

use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{self, Exact};
use crate::kernel::ratio::fraction;
use crate::line_emission::emit as emit_lines;
use crate::retirement_return::IraAmountIssue;
use crate::{
    FederalDraft, FederalIncome, FederalResult, IraBasis, Owner, RetirementKind, RetirementTaxable,
    RetirementTreatment,
};

impl IraBasis {
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
    basis: &crate::retirement_return::BasisResult,
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
                &crate::IssueContext::owner(crate::TaxSection::Retirement, b.owner),
                "Special IRA repayment",
                "ISSUE_CONFIRM_ACTUAL_ACCEPTED_REPAYMENTS_AND_USE_ONE_RECORD_PER_ORIGINAL_6D4178D6",
                "Use one repayment record per original Income entry.",
            );
            // No: nothing is credited; the distribution stays taxable.
            if !repayment.confirmed {
                continue;
            }
            let source = d.income.iter().find(|e| {
                e.id == repayment.income && e.owner == crate::DocumentOwner::from(b.owner)
            });
            let distribution = source.and_then(|e| match &e.income {
                FederalIncome::Retirement { data: source }
                    if {
                        let data = &crate::FederalRetirement::from(source);
                        data.kind == RetirementKind::Ira
                            && data.taxable == Some(RetirementTaxable::IraBasis)
                    } =>
                {
                    let data = &crate::FederalRetirement::from(source);
                    match &data.ordinary_distribution {
                        Some(RetirementTreatment::Exception { distributed, facts, .. })
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
                &crate::IssueContext::owner(crate::TaxSection::Retirement, b.owner),
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
                if e.owner != crate::DocumentOwner::from(b.owner) {
                    return None;
                }
                match &e.income {
                    FederalIncome::Retirement { data: source }
                        if {
                            let data = &crate::FederalRetirement::from(source);
                            data.kind == RetirementKind::Ira
                        } =>
                    {
                        let data = &crate::FederalRetirement::from(source);
                        match &data.ordinary_distribution {
                            Some(RetirementTreatment::Exception { facts, .. })
                                if special(facts) =>
                            {
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
