use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::ratio::allocate_remaining;

use crate::{FederalResult, ForeignOrdinaryTax};
use alloc::{format, string::String, vec::Vec};
use libertas_macros::*;

/// Foreign income included in a state income-tax base
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignStateIncome {
    /// Matching country and payer label from the foreign-income records
    /// Use the exact unique label. This links the income to its existing category and passive withholding group, before federal high-tax reclassification.
    #[libertas_size(min = 1)]
    pub source: String,
    /// Income attributed to this source by the state's tax law
    /// Use the state return and supporting income records underlying this payment, not a computed tax share. Exclude income exempted by that state; federal and state income amounts may differ.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub income: i64,
}

/// State or local income-tax payment and its source income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignStatePayment {
    /// Taxing jurisdiction and source year
    #[libertas_size(min = 1)]
    pub label: String,
    /// Eligible income tax paid, already entered in itemized expenses
    /// Exclude sales and property taxes. Split payments only where their underlying state income differs; do not enter a federal deduction or duplicate the payment.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub paid: i64,
    /// U.S.-source income in this payment's state tax base
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub domestic: i64,
    /// Foreign-source income in that state tax base
    /// Leave empty only when the tax relates entirely to U.S. income, including when the state exempts all foreign income. Identify each source once; all domestic and foreign amounts must reconcile to the income underlying this payment.
    pub foreign: Vec<ForeignStateIncome>,
}

/// Source allocation of itemized state and local income taxes
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignStateTaxes {
    // This carries unavailable state-source-income workpapers, not an unknown
    // confirmation. Federal AGI cannot establish another year's state tax base.
    // When needed, allocate stops at the source-fact boundary
    // ISSUE_SUPPLY_THE_SOURCE_INCOME_UNDERLYING_ITEMIZED_STATE_LOCAL_INCOME_TAX_5BB69CE3;
    // it never invents source income or treats unavailable records as zero.
    /// Not needed or not yet reviewed
    /// Required only when an itemized state/local income-tax deduction is used. Standard deduction and general sales taxes do not require these records.
    Unreviewed,
    /// Allocate from the state income records
    Records {
        /// Payments included in the itemized income-tax amount
        #[libertas_size(min = 1)]
        payments: Vec<ForeignStatePayment>,
        /// Complete source attribution under each taxing jurisdiction's law
        /// Check each state return's source-income worksheet. No apportions the tax with your other deductions instead, with a notice. All payments reconcile to the itemized income-tax entry. Source income is nonnegative and reflects the tax base underlying each payment, including exemptions and resolved differences from federal law. No unresolved state apportionment, special election or loss allocation. Do not substitute current federal AGI when the payment relates to other income.
        confirmed: bool,
    },
}
impl ForeignStateTaxes {
    pub(crate) fn valid(&self) -> bool {
        let money = |v| (0..=100000000000).contains(&v);
        let label = |s: &str| !s.trim().is_empty();
        match self {
            Self::Unreviewed => true,
            Self::Records { payments, .. } => {
                !payments.is_empty()
                    // Temporary arithmetic exclusion: allocate sums native payment
                    // amounts before reconciliation and accumulates native weights.
                    && payments.len() <= 100
                    && payments.iter().all(|p| {
                        label(&p.label)
                            && p.paid > 0
                            && money(p.paid)
                            && money(p.domestic)
                            // Temporary arithmetic exclusion: allocate builds and
                            // totals each payment's native income-group vector.
                            && p.foreign.len() <= 100
                            && p.foreign.iter().enumerate().all(|(i, f)| {
                                label(&f.source)
                                    && money(f.income)
                                    && !p.foreign[..i].iter().any(|v| v.source == f.source)
                            })
                    })
            }
        }
    }

    // Five existing foreign withholding groups, followed by domestic income.
    pub(crate) fn allocate(
        &self,
        paid: i64,
        allowed: i64,
        sources: &[ForeignOrdinaryTax],
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
                .all(|(i, s)| !sources[..i].iter().any(|v| v.label == s.label));
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
                let Some(source) = sources.iter().find(|v| v.label == f.source) else {
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
