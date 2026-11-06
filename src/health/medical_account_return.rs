use crate::FederalResult;
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{self, Exact, whole};
use crate::line_emission::emit as emit_lines;
use crate::medical_account::{
    MedicalSaAccount, MedicalSaPayment, MedicareForm, OutputFailure, required_output,
};
use alloc::vec::Vec;

// Worksheet operands without a required form destination may omit only their
// explanation, with this owning notice. Required form amounts use the fallible
// projection above and never this optional path.
pub(crate) fn explanation_output(
    value: &Exact,
    context: crate::IssueContext,
    r: &mut FederalResult,
) -> Option<i64> {
    let amount = exact::rounded(value);
    if amount.is_none() {
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new("MEDICAL_EXPLANATION_UNAVAILABLE", alloc::vec![]),
            context,
        );
    }
    amount
}

impl OutputFailure {
    pub(crate) fn report(self, r: &mut FederalResult) {
        crate::issue_localized!(
            r,
            &self.context,
            "Medical account output",
            "MEDICAL_REQUIRED_OUTPUT_UNAVAILABLE",
            "An actual required medical-account output exceeds its native amount representation.",
        );
        r.state = crate::ResultState::Unsupported;
        crate::federal_rules::withhold_required_source_result(r);
    }
}

// Section B per spouse (whole dollars): the taxable distributions, the part
// without the additional tax, the prior-balance relief and the 50% tax. The
// relief is the spouse's, taken once from the first form.
pub(crate) fn medicare_totals(
    forms: &[MedicareForm],
    r: &mut FederalResult,
) -> Result<([i64; 2], [i64; 2]), OutputFailure> {
    let mut total = [0; 2];
    let mut tax = [0; 2];
    for owner in 0..2 {
        let mine: Vec<&MedicareForm> = forms.iter().filter(|f| f.owner == owner).collect();
        let Some(first) = mine.first() else {
            continue;
        };
        crate::require_localized!(
            r,
            mine.iter().all(|f| f.prior == first.prior),
            &first.context,
            "1099-SA",
            "ISSUE_MEDICARE_MSA_PRIOR_DIFFERS",
            "Two Medicare Advantage MSA forms for the same person show different December 31, 2025 balances. Use the same account value on each.",
        );
        let taxable = exact::dollars_exact(&mine.iter().map(|f| &f.taxable).sum::<Exact>());
        let penalized = phase::cap(
            exact::dollars_exact(&mine.iter().map(|f| &f.penalized).sum::<Exact>()),
            [taxable.clone()],
        );
        let relief = first.prior.relief();
        let owner_tax =
            exact::rounded_exact(&(phase::floor(&penalized - &relief, whole(0)) / whole(2)));
        total[owner] = required_output(&taxable, first.context.clone())?;
        tax[owner] = required_output(&owner_tax, first.context.clone())?;
        let tax_cents = required_output(&(&owner_tax * whole(100)), first.context.clone())?;
        let mut explanation_unavailable = false;
        emit_lines(
            r,
            &mut explanation_unavailable,
            [
                ("exception", taxable - penalized),
                ("balance_relief", relief),
                ("additional_tax", owner_tax),
            ],
            |(suffix, amount), unavailable, _| {
                // These two worksheet explanations are not required form
                // amounts. Preserve the known income/tax if a displayed
                // explanation cannot fit; the actual tax row remains required.
                let cents = if suffix == "additional_tax" {
                    tax_cents
                } else if let Some(cents) = exact::rounded(&(amount * whole(100))) {
                    cents
                } else {
                    *unavailable = true;
                    return None;
                };
                Some(calculation_row(
                    None,
                    &alloc::format!("medicare_msa_{}_{suffix}", first.id),
                    LineValue::MoneyCents(cents),
                    &["income.medical_distribution"],
                    "https://www.irs.gov/instructions/i8853",
                    msg!("EXPLAIN_MEDICARE_MSA_N_N"),
                ))
            },
        );
        if explanation_unavailable {
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new("MEDICAL_EXPLANATION_UNAVAILABLE", alloc::vec![]),
                first.context.clone(),
            );
        }
    }
    Ok((total, tax))
}

// Qualified expense source amounts already accepted by the HSA payment owner.
// Consumers allocate an actual shared expense; they never subtract all HSA
// spending from a different medical source.
pub(crate) fn qualified_hsa_spending(draft: &crate::FederalDraft) -> i128 {
    draft
        .income
        .iter()
        .filter_map(|entry| match &entry.income {
            crate::FederalIncome::MedicalDistribution { data }
                if *data.account() == MedicalSaAccount::Hsa =>
            {
                match &data.payment {
                    Some(MedicalSaPayment::Own { medical, .. }) => Some(i128::from(*medical)),
                    _ => None,
                }
            }
            _ => None,
        })
        .sum()
}
