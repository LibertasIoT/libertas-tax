use crate::FederalResult;
use crate::farm_land::ResolvedFarmLandSale;
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use libertas_macros::*;

// The section 1062 election (FarmLandSale.deferral). Choosing it states the
// ten years of qualifying use, a qualified individual farmer buyer and the
// enforceable covenant; the page offers no other answer.
pub(crate) fn qualifies(sale: &ResolvedFarmLandSale) -> bool {
    sale.calculate().gain > 0
}

pub(crate) fn report(sale: &ResolvedFarmLandSale, r: &mut FederalResult) {
    crate::require_localized!(
        r,
        qualifies(sale),
        &crate::IssueContext::Section(crate::TaxSection::Farming),
        "Farmland tax installments",
        "ISSUE_CONFIRM_TEN_YEARS_OF_QUALIFYING_USE_A_QUALIFIED_INDIVIDUAL_BUYER_AN_E0DF1DF7",
        "Confirm ten years of qualifying use, a qualified individual buyer, an enforceable ten-year covenant and a recognized gain before electing section 1062.",
    );
}

/// Farmland tax installment election
/// Payment timing only: the full estimated federal tax remains unchanged. The first installment is included in the current amount due; the remaining balance is deferred, not forgiven. File Form 1062, a separate Schedule A and covenant for each elected sale. The App does not file or pay. Later installments are due on the unextended return deadlines for the next three years; default or death can accelerate the unpaid balance. Whole-dollar rounding may require a final installment adjustment.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmTaxInstallments {
    /// Tax attributable to all elected sales
    #[libertas_money("USD", 2)]
    pub applicable: i64,
    /// First installment due by the unextended return deadline
    #[libertas_money("USD", 2)]
    pub first: i64,
    /// Remaining liability deferred to later years
    #[libertas_money("USD", 2)]
    pub remaining: i64,
    /// Net current federal tax position before other payments and refundable credits
    /// Annual tax less the deferred balance. A negative amount contributes to the estimated overpayment; it is not negative annual tax or forgiveness of future installments.
    #[libertas_money("USD", 2)]
    pub current: i64,
}

pub(crate) struct TaxContext<'a> {
    pub pool: &'a crate::extended_rules::RentalResult,
    pub capital: crate::capital::CapitalAmounts,
    pub capital_prior: [i64; 2],
    pub capital_election: [i64; 2],
    pub regular: crate::income_tax::RegularTaxWorksheet,
    pub actual_regular: i64,
    pub actual_personal: i64,
    pub credits: crate::income_tax::PersonalCreditPotentials,
    pub foreign: &'a crate::ForeignTaxClaim,
    // The ordinary Form 1116 claim's inputs and its actual credit, for the
    // refigure on the reduced return.
    pub foreign_basis: Option<&'a crate::form_1116::CreditBasis>,
    pub actual_foreign: i64,
    // The payee-statement tax the return shows itself, in cents.
    pub statement_taxes: i64,
    pub premium_repayment: i64,
    pub averaging: Option<crate::farm_averaging::Worksheet<'a>>,
    pub total: i64,
    pub ready: bool,
}

pub(crate) fn calculate(
    draft: &crate::FederalDraft,
    context: TaxContext<'_>,
    r: &mut FederalResult,
) -> Option<FarmTaxInstallments> {
    use crate::kernel::ratio::{fraction, positive};
    let mut gain = 0;
    let mut elected = false;
    // Schedule F farms and Form 4835 farm rentals alike: Form 1062 covers land
    // "leased to a farmer" as well as farmed land.
    for source in crate::farm::sources(draft) {
        for record in &source.farm.land_sales {
            let sale = record.resolve(draft, source.entry.id, source.farm)?;
            if sale.deferral {
                elected = true;
                if !qualifies(&sale) {
                    return None; // The source report owns the eligibility diagnostic.
                }
                gain += sale.calculate().gain;
            }
        }
    }
    if !elected {
        return None;
    }
    if !context.ready {
        crate::issue_localized!(
            r,
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "Farmland tax installments",
            "ISSUE_THIS_ELECTION_STILL_NEEDS_COORDINATION_WITH_FULL_FORM_1116_PARENT_R_44C75BFA",
            "This election still needs coordination with parent-rate tax or a final return. No installment totals are shown until the applicable worksheet is supported.",
        );
        return None;
    }
    let property = match crate::section1231::without_elected_land(draft, context.pool) {
        Ok(value) => value,
        Err(error) => {
            error.report_at(
                r,
                &crate::IssueContext::Section(crate::TaxSection::Farming),
                #[cfg(test)]
                "Farmland tax installments",
            );
            return None;
        }
    };
    let capital = crate::capital::CapitalAmounts {
        long: context.capital.long + property.as_ref().map_or(0, |v| v.long_term),
        unrecaptured: context.capital.unrecaptured
            + property.as_ref().map_or(0, |v| v.unrecaptured),
        ..context.capital
    }
    .calculate(
        context.capital_prior,
        context.regular.status == crate::FederalStatus::Separate,
        false,
    );
    let reduced = crate::income_tax::RegularTaxWorksheet {
        preference: crate::income_tax::Preference {
            dividends: context.regular.preference.dividends,
            gain: positive(capital.net_gain - context.capital_election[0]),
            line18: capital.line18,
            line19: capital.line19,
        },
        ..context.regular.less(gain)
    };
    // Line 1 comes from the actual return, including its elected tax method.
    let actual_tax = context.actual_regular;
    let reduced_tax = if let Some(averaging) = context.averaging {
        averaging.tax(reduced, r)
    } else {
        reduced.tax()
    };
    // Form 1062 changes taxable income, not the household MAGI, coverage or
    // advance payments used by Form 8962. Reuse the actual repayment in lines
    // 2 and 10; never rerun AGI/deductions by deleting elected sale income.
    let actual_liability = actual_tax + context.premium_repayment;
    let reduced_liability = reduced_tax + context.premium_repayment;
    // The payee-statement election refigures its cap; the ordinary Form 1116
    // claim reruns its records on the reduced return.
    let ordinary = context.foreign_basis.and_then(|basis| {
        crate::form_1116::credit_without_elected_gain(
            draft,
            context.foreign,
            basis,
            crate::form_1116::Reduced {
                taxable: reduced.taxable,
                preference: reduced.preference.capped(),
                regular: positive(basis.regular - (actual_tax - reduced_tax)),
                gain,
                schedule_d: crate::foreign_capital::ScheduleD {
                    short: capital.short,
                    long: capital.long,
                    net_gain: capital.net_gain,
                },
            },
        )
    });
    let (actual_foreign, reduced_foreign) = match ordinary {
        Some(reduced_credit) => (context.actual_foreign, reduced_credit),
        None => (
            context
                .foreign
                .statement_credit(actual_liability, context.statement_taxes)
                .expect("payee-statement or no credit"),
            context
                .foreign
                .statement_credit(reduced_liability, context.statement_taxes)
                .expect("payee-statement or no credit"),
        ),
    };
    // Part I uses credits actually allowed on the return, including credits
    // against AMT. Do not reconstruct them using only regular liability.
    // Section1062(d)(1)(B) includes only credits in subparts A, B and D.
    // Section53 is in subpart G; Form1062 line4b explicitly removes
    // Schedule3 line6b. Actual minimum-tax credit remains on the return.
    let actual_credit = actual_foreign + context.actual_personal;
    // Part II line 12z caps ALL refigured credits at regular liability. For
    // these section 26(a) personal credits, limiting directly to that capacity
    // gives the same aggregate as including any nonnegative hypothetical AMT
    // and then applying 12z. No hypothetical AMT or carryforward is created.
    let reduced_credit = reduced_foreign
        + context.credits.limit(reduced_liability - reduced_foreign).nonrefundable_total();
    let applicable =
        positive((actual_liability - actual_credit) - (reduced_liability - reduced_credit));
    let first = fraction(applicable, 1, 4);
    let remaining = applicable - first;
    emit_lines(
        r,
        &mut (),
        [
            ("elected_gain", gain, msg!("EXPLAIN_FARM_INSTALLMENT_N_ELECTED_GAIN")),
            (
                "reduced_taxable",
                reduced.taxable,
                msg!("EXPLAIN_FARM_INSTALLMENT_N_REDUCED_TAXABLE"),
            ),
            (
                "reduced_regular_tax",
                reduced_tax,
                msg!("EXPLAIN_FARM_INSTALLMENT_N_REDUCED_REGULAR_TAX"),
            ),
            (
                "regular_additions",
                context.premium_repayment,
                msg!("EXPLAIN_FARM_INSTALLMENT_N_REGULAR_ADDITIONS"),
            ),
            (
                "actual_regular_liability",
                actual_liability,
                msg!("EXPLAIN_FARM_INSTALLMENT_N_ACTUAL_REGULAR_LIABILITY"),
            ),
            (
                "reduced_regular_liability",
                reduced_liability,
                msg!("EXPLAIN_FARM_INSTALLMENT_N_REDUCED_REGULAR_LIABILITY"),
            ),
            (
                "actual_foreign_credit",
                actual_foreign,
                msg!("EXPLAIN_FARM_INSTALLMENT_N_ACTUAL_FOREIGN_CREDIT"),
            ),
            (
                "reduced_foreign_credit",
                reduced_foreign,
                msg!("EXPLAIN_FARM_INSTALLMENT_N_REDUCED_FOREIGN_CREDIT"),
            ),
            ("actual_credits", actual_credit, msg!("EXPLAIN_FARM_INSTALLMENT_N_ACTUAL_CREDITS")),
            ("reduced_credits", reduced_credit, msg!("EXPLAIN_FARM_INSTALLMENT_N_REDUCED_CREDITS")),
            ("applicable", applicable, msg!("EXPLAIN_FARM_INSTALLMENT_N_APPLICABLE")),
            ("first", first, msg!("EXPLAIN_FARM_INSTALLMENT_N_FIRST")),
            ("remaining", remaining, msg!("EXPLAIN_FARM_INSTALLMENT_N_REMAINING")),
            ("current", context.total - remaining, msg!("EXPLAIN_FARM_INSTALLMENT_N_CURRENT")),
        ],
        |(name, value, explanation), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("farm_installment_{name}"),
                LineValue::MoneyDollars(value),
                &["income.business.activity.farming.land_sales.deferral"],
                "https://www.irs.gov/instructions/i1062",
                explanation,
            ))
        },
    );
    // The 2026 draft Schedule 3 line 13e includes the full applicable amount;
    // Form 1040 line 24b adds the first installment. Neither form caps the
    // resulting payment offset at annual tax. Keep the signed net position;
    // the complete return computes overpayment without changing annual tax.
    Some(FarmTaxInstallments {
        applicable: applicable * 100,
        first: first * 100,
        remaining: remaining * 100,
        current: (context.total - remaining) * 100,
    })
}
