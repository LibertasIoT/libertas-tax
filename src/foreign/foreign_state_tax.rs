pub use crate::foreign_state_source::{ForeignStateIncome, ForeignStatePayment, ForeignStateTaxes};
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::ratio::allocate_remaining;
use crate::line_emission::emit as emit_lines;

use crate::FederalResult;
use alloc::format;

impl ForeignStateTaxes {
    // Five existing foreign withholding groups, followed by domestic income.
    pub(crate) fn allocate(
        &self,
        paid: i64,
        allowed: i64,
        sources: &[crate::estate_foreign::ForeignCatalogSource],
        r: &mut FederalResult,
    ) -> [i64; crate::form_1116::SLOTS] {
        if paid == 0 || allowed == 0 {
            return [0; crate::form_1116::SLOTS];
        }
        let Self::Records { payments, confirmed } = self else {
            crate::require_localized!(
                r,
                false,
                &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                "Foreign state tax allocation",
                "ISSUE_SUPPLY_THE_SOURCE_INCOME_UNDERLYING_ITEMIZED_STATE_LOCAL_INCOME_TAX_5BB69CE3",
                "Supply the source income underlying itemized state/local income-tax payments before allocating their foreign deduction.",
            );
            return [0; crate::form_1116::SLOTS];
        };
        if !*confirmed {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("WARN_STATE_TAX_SOURCE_LEFT_OUT"),
                crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            );
            return [0; crate::form_1116::SLOTS];
        }
        let consistent = self.valid()
            && payments.iter().map(|p| p.paid).sum::<i64>() == paid
            && sources
                .iter()
                .enumerate()
                .all(|(i, s)| !sources[..i].iter().any(|v| v.reference.same_as(&s.reference)));
        crate::require_localized!(
            r,
            consistent,
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            "Foreign state tax allocation",
            "ISSUE_CONFIRM_COMPLETE_STATE_RECORDS_UNIQUE_FOREIGN_SOURCE_LABELS_AND_PAY_D368FEA4",
            "Confirm complete state records, unique foreign-source labels and payments matching the itemized income-tax amount exactly.",
        );
        if !consistent {
            return [0; crate::form_1116::SLOTS];
        }
        let mut weights = [0; crate::form_1116::SLOTS];
        for p in payments {
            let mut income = [0; crate::form_1116::SLOTS];
            income[crate::form_1116::DOMESTIC] = p.domestic;
            for f in &p.foreign {
                let Some(source) = sources.iter().find(|v| f.source.matches(v)) else {
                    crate::require_localized!(
                        r,
                        false,
                        &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                        "Foreign state tax allocation",
                        "ISSUE_A_STATE_INCOME_SOURCE_LABEL_DOES_NOT_MATCH_A_UNIQUE_FOREIGN_INCOME_7C424E95",
                        "A state-income source label does not match a unique foreign-income record.",
                    );
                    return [0; crate::form_1116::SLOTS];
                };
                income[source.group()] += f.income;
            }
            crate::require_localized!(
                r,
                income.iter().sum::<i64>() > 0,
                &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                "Foreign state tax allocation",
                "ISSUE_EACH_STATE_PAYMENT_REQUIRES_A_POSITIVE_INCOME_BASE_UNRESOLVED_LOSS_C1E83BC5",
                "Each state payment requires a positive income base; unresolved loss or minimum-tax attribution needs a separate worksheet.",
            );
            if income.iter().sum::<i64>() == 0 {
                return [0; crate::form_1116::SLOTS];
            }
            // Retain cents through each jurisdiction, then apportion the allowed
            // whole-dollar deduction once. Fixed group order is row independent.
            for (i, amount) in allocate_remaining(p.paid, &income).into_iter().enumerate() {
                weights[i] += amount;
            }
        }
        let result: [i64; crate::form_1116::SLOTS] =
            allocate_remaining(allowed, &weights).try_into().unwrap();
        emit_lines(r, &mut (), result.iter().enumerate(), |(i, amount), _, _| {
            Some(calculation_row(
                None,
                &format!("foreign_state_income_tax_group_{i}"),
                LineValue::MoneyDollars(*amount),
                &["deductions", "credits.foreign_tax.state_taxes"],
                "https://www.irs.gov/publications/p514",
                msg!("EXPLAIN_FOREIGN_STATE_INCOME_TAX_GROUP_N"),
            ))
        });
        result
    }
}
