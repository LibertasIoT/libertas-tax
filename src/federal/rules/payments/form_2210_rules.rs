// form_2210 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn estimated_timing_only(result: &FederalResult) -> bool {
    !result.issues.is_empty()
        && result
            .issues
            .iter()
            .all(|issue| issue.display.key == "ISSUE_2210_ESTIMATED_PAYMENT_DATES")
}

pub(super) struct Form2210In<'a> {
    pub(super) farmer_capital_correction: Option<i64>,
    pub(super) capital_gross: i64,
    pub(super) ordinary_gain_gross: i64,
    pub(super) unemployment: i64,
    pub(super) additional_medicare: i64,
    pub(super) niit: i64,
    pub(super) early_tax: i64,
    pub(super) account_tax: i64,
    pub(super) hsa_chapter_one_tax: i64,
    pub(super) archer_msa_tax: i64,
    pub(super) medicare_msa_tax: i64,
    pub(super) investment_recapture_tax: i64,
    pub(super) trump_earnings_tax: i64,
    pub(super) bba_schedule_2_adjustment: i64,
    pub(super) wages: i64,
    pub(super) adoption_wages: i64,
    pub(super) benefit_wages: i64,
    pub(super) foreign_wages: i64,
    pub(super) interest: i64,
    pub(super) dividends: i64,
    pub(super) social: i64,
    pub(super) iso_unreported: i64,
    pub(super) hsa_income: i64,
    pub(super) alimony: i64,
    pub(super) gambling: i64,
    pub(super) jury: i64,
    pub(super) scholarship: i64,
    pub(super) prizes: i64,
    pub(super) other_line_8: i64,
    pub(super) olympic_taxable: i64,
    pub(super) taxable_grants: i64,
    pub(super) family_leave: i64,
    pub(super) casualty_living_insurance: i64,
    pub(super) personal_debt: i64,
    pub(super) care_benefits: i64,
    pub(super) medicare_msa: i64,
    pub(super) archer_total_income: i64,
    pub(super) able_income: i64,
    pub(super) education_income: i64,
    pub(super) trump_death: i64,
    pub(super) tax_refund: i64,
    pub(super) rental_gross: i64,
    pub(super) royalty_gross: i64,
    pub(super) personal_rent: i64,
    pub(super) hobby: i64,
    pub(super) child_nonrefund: i64,
    pub(super) actc: i64,
    pub(super) income_tax: i64,
    pub(super) credits: i64,
    pub(super) se_tax: i64,
    pub(super) household_tax: i64,
    pub(super) refundable: i64,
    pub(super) fuel_credit: i64,
    pub(super) withholding_25d: i64,
    pub(super) excess_social: i64,
    pub(super) fund_credit: i64,
    pub(super) agi: i64,
    pub(super) taxable: i64,
    pub(super) standard: i64,
    pub(super) regular_tax: i64,
    pub(super) child_tax: i64,
    pub(super) education_recapture_tax: i64,
    pub(super) excluded: i64,
    pub(super) regular_tax_pref: i64,
    pub(super) amt: i64,
    pub(super) bba_additional_tax: i64,
    pub(super) lump_sum_tax: i64,
    pub(super) premium_repayment: i64,
    pub(super) clean_vehicle_repayment: i64,
    pub(super) retirement_taxable: [i64; 2],
    pub(super) se: [i64; 2],
    pub(super) use_itemized: bool,
    pub(super) parent_rate_required: bool,
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) estate_estimated_cents: &'a crate::kernel::ratio::exact::Exact,
    pub(super) installments: &'a Option<crate::farm_deferral::FarmTaxInstallments>,
    pub(super) source_claims: &'a alloc::collections::BTreeMap<i64, i64>,
}

pub(super) struct Form2210Out {
    pub(super) line_2_other: i64,
    pub(super) farming_return_year: Result<bool, &'static str>,
    pub(super) penalty: Option<i64>,
    pub(super) penalty_explanation: crate::Message,
}

pub(super) fn form_2210_phase(
    pass: &Pass,
    r: &mut FederalResult,
    i: Form2210In<'_>,
) -> Result<Form2210Out, crate::Section179PreparedIssue> {
    let &Pass { d, context, s, family, pay, .. } = pass;
    let Form2210In {
        farmer_capital_correction,
        capital_gross,
        ordinary_gain_gross,
        unemployment,
        additional_medicare,
        niit,
        early_tax,
        account_tax,
        hsa_chapter_one_tax,
        archer_msa_tax,
        medicare_msa_tax,
        investment_recapture_tax,
        trump_earnings_tax,
        bba_schedule_2_adjustment,
        wages,
        adoption_wages,
        benefit_wages,
        foreign_wages,
        interest,
        dividends,
        social,
        iso_unreported,
        hsa_income,
        alimony,
        gambling,
        jury,
        scholarship,
        prizes,
        other_line_8,
        olympic_taxable,
        taxable_grants,
        family_leave,
        casualty_living_insurance,
        personal_debt,
        care_benefits,
        medicare_msa,
        archer_total_income,
        able_income,
        education_income,
        trump_death,
        tax_refund,
        rental_gross,
        royalty_gross,
        personal_rent,
        hobby,
        child_nonrefund,
        actc,
        income_tax,
        credits,
        se_tax,
        household_tax,
        refundable,
        fuel_credit,
        withholding_25d,
        excess_social,
        fund_credit,
        agi,
        taxable,
        standard,
        regular_tax,
        child_tax,
        education_recapture_tax,
        excluded,
        regular_tax_pref,
        amt,
        bba_additional_tax,
        lump_sum_tax,
        premium_repayment,
        clean_vehicle_repayment,
        retirement_taxable,
        se,
        use_itemized,
        parent_rate_required,
        rental,
        estate_estimated_cents,
        installments,
        source_claims,
    } = i;
    // Form 2210 reads the finished return: line 2 keeps only the Schedule 2
    // taxes its instructions list, and line 3 takes the fuel tax credit, which
    // Form 1040 still reports among the line 31 payments.
    let line_2_other = additional_medicare
        + niit
        + early_tax
        + account_tax
        + hsa_chapter_one_tax
        + archer_msa_tax
        + medicare_msa_tax
        // Schedule 2 line 13a (2025 line 17a), the Form 4255 recapture, and Form 5329 line 63,
        // the Trump account distributions tax on line 5 (2025 "Line 8 (additional tax on
        // distributions only)"): both are on the 2025 Form 2210 line 2 list.
        + investment_recapture_tax
        + trump_earnings_tax
        - bba_schedule_2_adjustment;
    // The farmer and fisherman two-thirds test for 2026, from this
    // return: the income (not loss) amounts of Form 1040 lines 1z-7 and of
    // Schedule 1 Part I as this return computes them, with each business's
    // gross income in place of its net profit (underpayment::two_thirds_return_year).
    // The net operating loss deduction, the foreign exclusion, the ordinary
    // stock loss and the excess business loss adjustment are not income.
    let farmer_capital_gross = farmer_capital_correction
        .and_then(|correction| capital_gross.checked_add(correction))
        .map(dollars)
        .zip(crate::section1231::farmer_gross_property_with_claims(d, rental, source_claims)?)
        .and_then(|(capital, property)| {
            crate::kernel::ratio::checked_sum([capital, property, dollars(ordinary_gain_gross)])
        });
    let other_income = farmer_capital_gross
        .and_then(|capital| {
            crate::kernel::ratio::checked_sum(
                [
                    wages + adoption_wages + benefit_wages + foreign_wages,
                    interest,
                    dividends,
                    retirement_taxable[0],
                    retirement_taxable[1],
                    social,
                    // Gross income counts each gain, never a loss (Pub. 505 (2026), Farming and Fishing):
                    // admitted source gains stay gross after their actual exclusions/timing.
                    capital,
                    iso_unreported,
                    dollars(unemployment),
                    hsa_income,
                    alimony,
                    gambling,
                    jury,
                    scholarship,
                    prizes,
                    other_line_8,
                    olympic_taxable,
                    taxable_grants,
                    family_leave,
                    casualty_living_insurance,
                    personal_debt,
                    care_benefits,
                    medicare_msa,
                    archer_total_income,
                    able_income,
                    education_income,
                    trump_death,
                    tax_refund,
                    // The native FTC owners retain gross source receipts, not their
                    // expense/depletion/PAL-reduced amounts. Personal rent already is gross.
                    rental_gross,
                    dollars(royalty_gross),
                    personal_rent,
                    hobby,
                ]
                .into_iter()
                .map(positive),
            )
        })
        .and_then(|amount| amount.checked_mul(100));
    let two_thirds = crate::underpayment::two_thirds_return_year_with_claims(
        d,
        r,
        other_income,
        &rental.income_by_id,
        source_claims,
    )?;
    if let Ok(two_thirds) = two_thirds
        && two_thirds.farming > 0
    {
        emit_lines(
            r,
            &mut (),
            [
                (
                    "farming_fishing_gross_income_return_year",
                    two_thirds.farming,
                    msg!("EXPLAIN_FARMING_FISHING_GROSS_INCOME_N"),
                ),
                ("gross_income_return_year", two_thirds.gross, msg!("EXPLAIN_GROSS_INCOME_N")),
            ],
            |(id, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    id,
                    LineValue::MoneyCents(amount),
                    &["income", "payments"],
                    "https://www.irs.gov/pub/irs-pdf/p505.pdf",
                    explanation,
                ))
            },
        );
    }
    let farming_return_year = two_thirds.map(|gross| gross.qualifies());
    // Schedule AI line 18: a child's credit belongs to the columns that end on or after the
    // child's birth ("credits you're entitled to because of events that occurred during the
    // months shown in the column headings"). Other credits stay at their annual amounts.
    let child_credit = child_nonrefund + actc;
    let child_credit_by_column = {
        let births: Vec<u32> = family
            .ctc_ids
            .iter()
            .chain(&family.odc_ids)
            .filter_map(|id| d.dependents().iter().find(|v| v.id == *id))
            .map(|v| v.birth_date)
            .collect();
        crate::underpayment::PERIOD_ENDS.map(|end| {
            fraction(
                child_credit,
                births.iter().filter(|b| **b <= end).count() as i64,
                births.len().max(1) as i64,
            )
        })
    };
    // Form 1040 line 38's explanation follows the form that figured it.
    let mut penalty_explanation = msg!("EXPLAIN_1040_38_2");
    let penalty = crate::underpayment::calculate(
        pay,
        &crate::underpayment::Inputs {
            line_1: positive(income_tax - credits),
            line_2: se_tax + line_2_other,
            household_tax,
            line_3: refundable + fuel_credit,
            withholding: withholding_25d,
            excess_social,
            estate_estimated: estate_estimated_cents.clone(),
            separate: s.filing_status() == FederalStatus::Separate,
            farming_return_year,
            farmland_applicable: {
                use crate::kernel::source_fact::SourceFact;
                // The original election survives omission of an unsupported
                // activity from the calculation view. A partial owner result
                // cannot establish liability for omitted elected sales.
                let selected: alloc::collections::BTreeSet<_> =
                    crate::farm::sources(context.source_draft)
                        .flat_map(|source| {
                            source
                                .farm
                                .land_sales
                                .iter()
                                .filter(|sale| sale.deferral)
                                .map(move |sale| (source.entry.id, sale.id))
                        })
                        .collect();
                if selected.is_empty() {
                    SourceFact::NotReported
                } else {
                    let retained: alloc::collections::BTreeSet<_> = crate::farm::sources(d)
                        .flat_map(|source| {
                            source
                                .farm
                                .land_sales
                                .iter()
                                .filter(|sale| sale.deferral)
                                .map(move |sale| (source.entry.id, sale.id))
                        })
                        .collect();
                    match installments.as_ref() {
                        Some(value) if selected.is_subset(&retained) => {
                            SourceFact::Reported(value.applicable)
                        }
                        _ => SourceFact::Unavailable,
                    }
                }
            },
            fund_credit: fund_credit > 0,
            status: s.filing_status(),
            agi,
            taxable,
            standard,
            itemizing: use_itemized,
            credits: income_tax - positive(income_tax - credits),
            child_credit,
            child_credit_by_column,
            se_tax: se,
            ordinary_tax: regular_tax == preferential_tax(taxable, 0, s.filing_status())
                && child_tax == 0
                && education_recapture_tax == 0
                && excluded == 0
                && regular_tax_pref == 0
                && !parent_rate_required,
            other_taxes: line_2_other != 0
                || bba_schedule_2_adjustment != 0
                || household_tax != 0
                || amt != 0
                || bba_additional_tax != 0
                || child_tax != 0
                || education_recapture_tax != 0
                || lump_sum_tax != 0
                || premium_repayment != 0
                || clean_vehicle_repayment != 0,
        },
        r,
    )
    .map(|(cents, explanation)| {
        penalty_explanation = explanation;
        dollars(cents)
    });
    Ok(Form2210Out { line_2_other, farming_return_year, penalty, penalty_explanation })
}
