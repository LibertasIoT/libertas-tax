pub use crate::foreign_investment_source::{
    ForeignInvestmentAsset, ForeignInvestmentAssets, InvestmentAssetBasis, InvestmentAssetSource,
    InvestmentAssetYield, InvestmentBasisChange,
};
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::ratio::{allocate_remaining, dollars};
use crate::line_emission::emit as emit_lines;
use crate::{FederalResult, ForeignTaxCategory};
use alloc::format;
impl ForeignInvestmentAssets {
    // Without portfolio records the interest cannot be apportioned: all of it
    // is treated as relating to the foreign passive income, by that income
    // (Pub. 514, "Interest expense"; conservative), with one note.
    pub(crate) fn allocate(
        &self,
        allowed: [i64; 2],
        catalogs: [&[crate::estate_foreign::ForeignCatalogSource]; 2],
        domestic_income: i64,
        passive_gross: [[i64; crate::form_1116::SLOTS]; 2],
        r: &mut FederalResult,
    ) -> [[i64; crate::form_1116::SLOTS]; 2] {
        if allowed == [0; 2] {
            return [[0; crate::form_1116::SLOTS]; 2];
        }
        let sources = catalogs[0];
        let Self::TaxBook { assets } = self else {
            let mut weights = [[0; crate::form_1116::SLOTS]; 2];
            for context in 0..2 {
                for group in crate::form_1116::PASSIVE_GROUPS {
                    weights[context][group] = passive_gross[context][group];
                }
            }
            if weights.iter().all(|system| system.iter().sum::<i64>() <= 0) {
                return [[0; crate::form_1116::SLOTS]; 2];
            }
            crate::federal_rules::note_once_scoped(
                r,
                msg!("WARN_INVESTMENT_INTEREST_SOURCE_ASSUMED"),
                crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            );
            let mut result = [[0; crate::form_1116::SLOTS]; 2];
            for context in 0..2 {
                if weights[context].iter().sum::<i64>() > 0 {
                    result[context] =
                        allocate_remaining(allowed[context], &weights[context]).try_into().unwrap();
                }
            }
            return result;
        };
        let valid = self.valid()
            && sources
                .iter()
                .enumerate()
                .all(|(i, s)| !sources[..i].iter().any(|v| v.reference.same_as(&s.reference)));
        crate::require_localized!(
            r,
            valid,
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            "Foreign investment interest allocation",
            "ISSUE_CONFIRM_COMPLETE_ORDINARY_PORTFOLIO_BASIS_RECORDS_APPROPRIATE_COMPL_4BD36A4B",
            "Enter valid portfolio basis records and unique foreign-income labels before allocating investment interest.",
        );
        if !valid {
            return [[0; crate::form_1116::SLOTS]; 2];
        }
        let source_group = |reference: &crate::ForeignAllocationSource| {
            sources
                .iter()
                .find(|s| {
                    reference.matches(s) && matches!(s.category, ForeignTaxCategory::Passive { .. })
                })
                .map(|s| s.group())
        };
        let mut weights = [[0; crate::form_1116::SLOTS]; 2];
        let mut domestic_yield = 0;
        let mut reported_yields = alloc::vec![0; sources.len()];
        for asset in assets {
            let mut yields = [0; crate::form_1116::SLOTS];
            match &asset.source {
                InvestmentAssetSource::Domestic => yields[crate::form_1116::DOMESTIC] = 1,
                InvestmentAssetSource::Foreign { source } => {
                    let Some(group) = source_group(source) else {
                        crate::require_localized!(
                            r,
                            false,
                            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                            "Foreign investment interest allocation",
                            "ISSUE_AN_INVESTMENT_ASSET_MUST_MATCH_A_UNIQUE_PASSIVE_FOREIGN_INCOME_SOUR_A971F68A",
                            "An investment asset must match a unique passive foreign-income source; employment income is not portfolio income.",
                        );
                        return [[0; crate::form_1116::SLOTS]; 2];
                    };
                    yields[group] = 1;
                }
                InvestmentAssetSource::Mixed { domestic, foreign } => {
                    yields[crate::form_1116::DOMESTIC] = *domestic;
                    domestic_yield += domestic;
                    for item in foreign {
                        let Some(group) = source_group(&item.source) else {
                            crate::require_localized!(
                                r,
                                false,
                                &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                                "Foreign investment interest allocation",
                                "ISSUE_A_MIXED_ASSET_S_INCOME_MUST_MATCH_A_UNIQUE_PASSIVE_FOREIGN_INCOME_S_7FB18835",
                                "A mixed asset's income must match a unique passive foreign-income source.",
                            );
                            return [[0; crate::form_1116::SLOTS]; 2];
                        };
                        yields[group] += item.income;
                        let index = sources.iter().position(|s| item.source.matches(s)).unwrap();
                        reported_yields[index] += item.income;
                    }
                    if yields.iter().sum::<i64>() == 0 {
                        crate::require_localized!(
                            r,
                            false,
                            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                            "Foreign investment interest allocation",
                            "ISSUE_A_MIXED_SOURCE_ASSET_NEEDS_A_POSITIVE_CURRENT_YEAR_GROSS_INCOME_YIE_C7BB559B",
                            "A mixed-source asset needs a positive current-year gross-income yield to characterize its basis; unresolved or zero yield requires a separate worksheet.",
                        );
                        return [[0; crate::form_1116::SLOTS]; 2];
                    }
                }
            }
            // Both averaging methods use one common scale; mixed-source
            // characterization precedes whole-dollar interest allocation.
            for (context, basis) in asset.basis.weights().unwrap().into_iter().enumerate() {
                for (group, value) in allocate_remaining(basis, &yields).into_iter().enumerate() {
                    weights[context][group] += value;
                }
            }
        }
        crate::require_localized!(
            r,
            catalogs.iter().all(|sources| reported_yields
                .iter()
                .zip(*sources)
                .all(|(yield_amount, source)| *yield_amount <= source.income)),
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            "Foreign investment interest allocation",
            "ISSUE_COMBINED_MIXED_ASSET_GROSS_YIELDS_CANNOT_EXCEED_THEIR_ENTERED_FOREI_7E9019CA",
            "Combined mixed-asset gross yields cannot exceed their entered foreign-source income. Reconcile the source records without duplicating income.",
        );
        crate::require_localized!(
            r,
            dollars(domestic_yield) <= domestic_income,
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            "Foreign investment interest allocation",
            "ISSUE_COMBINED_DOMESTIC_MIXED_ASSET_YIELD_CANNOT_EXCEED_ENTERED_DOMESTIC_710F1444",
            "Combined domestic mixed-asset yield cannot exceed entered domestic portfolio income.",
        );
        let mut result = [[0; crate::form_1116::SLOTS]; 2];
        for context in 0..2 {
            let total = weights[context].iter().sum::<i64>();
            crate::require_localized!(
                r,
                allowed[context] == 0 || total > 0,
                &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                "Foreign investment interest allocation",
                "ISSUE_A_DEDUCTIBLE_INTEREST_AMOUNT_REQUIRES_A_POSITIVE_INVESTMENT_ASSET_B_16FA0C18",
                "A deductible interest amount requires a positive investment-asset basis for that tax system; zero-basis or substantially distorted holdings require another valuation worksheet.",
            );
            if total == 0 {
                continue;
            }
            result[context] =
                allocate_remaining(allowed[context], &weights[context]).try_into().unwrap();
            let name = if context == 0 { "regular" } else { "amt" };
            emit_lines(r, &mut (), result[context].iter().enumerate(), |(group, amount), _, _| {
                Some(calculation_row(
                    None,
                    &format!("foreign_investment_{name}_group_{group}"),
                    LineValue::MoneyDollars(*amount),
                    &["credits.foreign_tax.investment_assets", "investment_interest_deduction"],
                    "https://www.irs.gov/publications/p514",
                    msg!("EXPLAIN_FOREIGN_INVESTMENT_N_GROUP_N"),
                ))
            });
        }
        result
    }
}
