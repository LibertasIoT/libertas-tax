// form_1116 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct ForeignCreditIn<'a> {
    pub(super) use_itemized: bool,
    pub(super) property_character: Option<&'a crate::section1231::Result1231>,
    pub(super) amt_property_character: Option<&'a crate::section1231::Result1231>,
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) amt_rental: &'a crate::extended_rules::RentalResult,
    pub(super) business_half_se: &'a alloc::collections::BTreeMap<i64, i64>,
    pub(super) sep: &'a crate::sep::SepResult,
    pub(super) health: &'a crate::health::HealthResult,
    pub(super) royalty_total: &'a crate::kernel::ratio::exact::Exact,
    pub(super) salt_eligible: &'a crate::kernel::ratio::exact::Exact,
    pub(super) foreign_tax_deduction: &'a crate::extended_rules::ForeignTaxDeduction,
    pub(super) ira: &'a crate::ira::IraResult,
    pub(super) foreign: &'a crate::extended_rules::ForeignResult,
    pub(super) regular_capital: &'a crate::capital::CapitalResult,
    pub(super) amt_capital_result: &'a crate::capital::CapitalResult,
    pub(super) business_exclusions: &'a [crate::foreign_business::BusinessExclusion; 2],
    pub(super) k3_basis: &'a crate::schedule_k3::K3Basis,
    pub(super) regular_tax_worksheet: crate::income_tax::RegularTaxWorksheet,
    pub(super) royalty_interest: [i64; 2],
    pub(super) portfolio_interest: [i64; 2],
    pub(super) elected: [i64; 2],
    pub(super) additional: [i64; 3],
    pub(super) mortgage_deductible_interest: i64,
    pub(super) charity_deduction: i64,
    pub(super) cash_standard: i64,
    pub(super) amt_medical_deduction: i64,
    pub(super) casualty_deduction: i64,
    pub(super) qualified_net_casualty: i64,
    pub(super) itemized_reduction: i64,
    pub(super) itemized: i64,
    pub(super) amt_loan_gain: i64,
    pub(super) loan_gain: i64,
    pub(super) gambling_loss: i64,
    pub(super) annuity_loss: i64,
    pub(super) investment_property_taxes: i64,
    pub(super) regular_liability: i64,
    pub(super) lump_sum_tax: i64,
    pub(super) bba_credit: i64,
    pub(super) tentative: i64,
    pub(super) agi: i64,
    pub(super) deduction: i64,
    pub(super) qbi_ded: i64,
    pub(super) amt_base: i64,
    pub(super) foreign_tax_deductible: i64,
    pub(super) salt: i64,
    pub(super) interest: i64,
    pub(super) dividends: i64,
    pub(super) taxable: i64,
    pub(super) qualified: i64,
    pub(super) regular_tax_pref: i64,
    pub(super) regular_collectible: i64,
    pub(super) regular_unrecaptured: i64,
    pub(super) amt_tax_pref: i64,
    pub(super) amt_tax_collectible: i64,
    pub(super) amt_tax_unrecaptured: i64,
    pub(super) capital: i64,
    pub(super) property_ordinary: i64,
    pub(super) amt_tax_recovery: i64,
    pub(super) iso: i64,
    pub(super) educator_itemized: i64,
    pub(super) wages: i64,
    pub(super) foreign_wages: i64,
    pub(super) excess_business_loss: i64,
    pub(super) regular_tax: i64,
    pub(super) educator: i64,
    pub(super) housing_deduction: i64,
    pub(super) hsa: i64,
    pub(super) msa_deduction: i64,
    pub(super) alimony_paid: i64,
    pub(super) estate_67e: i64,
    pub(super) half_se: i64,
    pub(super) sep_deduction: i64,
    pub(super) savings_penalty: i64,
    pub(super) jury_remitted: i64,
    pub(super) employee_deduction: i64,
    pub(super) moving_deduction: i64,
    pub(super) loan: i64,
    pub(super) business: i64,
    pub(super) stock_loss: i64,
}

pub(super) struct ForeignCreditOut {
    pub(super) foreign_basis: crate::form_1116::CreditBasis,
    pub(super) foreign_credit: i64,
    pub(super) amt_credit: i64,
    pub(super) rental_gross: i64,
    pub(super) royalty_gross: i64,
    pub(super) foreign_credit_sources:
        crate::injured_spouse::income_categories::NativeForeignCreditSources,
}

pub(super) fn foreign_credit_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    nol_run: &crate::nol::NolRun,
    input: ForeignCreditIn<'_>,
) -> core::ops::ControlFlow<(), ForeignCreditOut> {
    let ForeignCreditIn {
        use_itemized,
        property_character,
        amt_property_character,
        rental,
        amt_rental,
        business_half_se,
        sep,
        health,
        royalty_total,
        salt_eligible,
        foreign_tax_deduction,
        ira,
        foreign,
        regular_capital,
        amt_capital_result,
        business_exclusions,
        k3_basis,
        regular_tax_worksheet,
        royalty_interest,
        portfolio_interest,
        elected,
        additional,
        mortgage_deductible_interest,
        charity_deduction,
        cash_standard,
        amt_medical_deduction,
        casualty_deduction,
        qualified_net_casualty,
        itemized_reduction,
        itemized,
        amt_loan_gain,
        loan_gain,
        gambling_loss,
        annuity_loss,
        investment_property_taxes,
        regular_liability,
        lump_sum_tax,
        bba_credit,
        tentative,
        agi,
        deduction,
        qbi_ded,
        amt_base,
        foreign_tax_deductible,
        salt,
        interest,
        dividends,
        taxable,
        qualified,
        regular_tax_pref,
        regular_collectible,
        regular_unrecaptured,
        amt_tax_pref,
        amt_tax_collectible,
        amt_tax_unrecaptured,
        capital,
        property_ordinary,
        amt_tax_recovery,
        iso,
        educator_itemized,
        wages,
        foreign_wages,
        excess_business_loss,
        regular_tax,
        educator,
        housing_deduction,
        hsa,
        msa_deduction,
        alimony_paid,
        estate_67e,
        half_se,
        sep_deduction,
        savings_penalty,
        jury_remitted,
        employee_deduction,
        moving_deduction,
        loan,
        business,
        stock_loss,
    } = input;
    let d = pass.d;
    let allocated_entries = pass.allocated_entries;
    let office_context = pass.office_context;
    let selected_regular = pass.selected_regular;
    let ded = pass.ded;
    let c = pass.c;
    let repayment = pass.repayment;
    let pension = pass.pension;
    let source_claims = pass.context.source_claims;
    // Reuse allowed deductions, including section68's proportional reduction.
    // The foreign worksheet owns source allocation, not deduction eligibility.
    let foreign_mortgage = if use_itemized { mortgage_deductible_interest } else { 0 };
    let foreign_charity = if use_itemized { charity_deduction } else { cash_standard };
    let foreign_investment =
        if use_itemized { [portfolio_interest[0], portfolio_interest[1]] } else { [0; 2] };
    let foreign_amt_deductions = if use_itemized {
        amt_medical_deduction
            + foreign_mortgage
            + foreign_charity
            + foreign_investment[1]
            + casualty_deduction
    } else {
        foreign_charity + qualified_net_casualty
    };
    let foreign_reduced = |amount| {
        if use_itemized {
            amount - fraction(itemized_reduction, amount, itemized.max(1))
        } else {
            amount
        }
    };
    // Form 1116 line 3e counts gains before losses: every Schedule D
    // contributor's positive amounts (the per-source totals collect() holds),
    // the Form 4797 Part I gain and each Form 4797 Part II ordinary gain (the S corporation
    // loan-repayment gain, a K-1's code R and section 751 gains), each whole. AGI holds their
    // net (`loan_gain`), which `restored` subtracts again, so a loss among them is restored
    // and a gain is counted once.
    let other_capital_gross = property_character.as_ref().map_or(0, |v| positive(v.long_term))
        + dollars(t.ordinary_gain_gross);
    let capital_gross = dollars(t.capital_gross) + other_capital_gross;
    #[cfg(test)]
    {
        r.capital_footing = Some([
            t.capital_gross,
            match i64::try_from(
                i128::from(t.short) + i128::from(t.long) + i128::from(t.netted_losses)
                    - i128::from(t.bypassed_gross),
            )
            .ok()
            {
                Some(value) => value,
                None => {
                    CollectFailure::Broker(crate::IssueContext::Section(
                        crate::TaxSection::InvestmentSales,
                    ))
                    .report(r);
                    withhold_required_source_result(r);
                    return core::ops::ControlFlow::Break(());
                }
            },
        ]);
    }
    let Some([amt_capital_cents, residual_sale_cents]) = t.amt_capital_operands(r) else {
        r.lines.clear();
        r.return_lines.clear();
        return core::ops::ControlFlow::Break(());
    };
    let amt_capital_gross = positive(
        // 2026-10-19: i1116 lines 3d/3e count gains before losses, and
        // i6251 line 8 uses the AMT's own amounts. Replace K-1 net changes
        // with gross changes in cents before rounding the capital subtotal.
        dollars(amt_capital_cents)
            + other_capital_gross
            + dollars(residual_sale_cents)
            + amt_property_character.as_ref().map_or(0, |v| v.long_term)
            - property_character.as_ref().map_or(0, |v| v.long_term)
            + amt_loan_gain
            - loan_gain,
    );
    // Form 1116 line 3e: Schedule C line 7 gross income (receipts less cost of
    // goods sold), Schedule F line 9, a professional gambler's winnings,
    // Schedule E line 3 rents, Form 4835 line 7 and royalty and personal
    // property rent receipts replace their net amounts in AGI.
    let mut business_gross = 0;
    let mut rental_gross = 0;
    let mut property_expenses = exact::whole(royalty_interest[0]) * exact::whole(100)
        + exact::whole(t.k1_royalty_deductions);
    let mut royalty_gross = t.k1_royalty_gross;
    let mut business_facts: Vec<crate::form_1116::BusinessFacts> = Vec::new();
    for e in allocated_entries {
        match &e.income {
            FederalIncome::Business { data } => match data.activity {
                crate::BusinessActivity::Ordinary { .. } => {
                    let goods = data.inventory.cost();
                    let gross = data.receipts() - goods;
                    business_gross += gross;
                    let deductions = match crate::section179_return::expenses_with_claims(
                        d,
                        e.id,
                        data,
                        selected_regular,
                        Some(office_context),
                        source_claims,
                    )
                    .map(|expense| {
                        expense
                            - selected_regular
                                .map_or(0, |allocation| allocation.ordinary_limit_adjustment(e.id))
                    }) {
                        Ok(value) => value,
                        Err(issue) => {
                            crate::section179_return::Issue::from(issue).report(r);
                            r.state = ResultState::Unsupported;
                            return core::ops::ControlFlow::Break(());
                        }
                    } - goods;
                    let totals = crate::FederalBusinessExpense::category_totals(&data.expenses);
                    business_facts.push(crate::form_1116::BusinessFacts {
                        id: e.id,
                        owner: e.owner.owned(),
                        abroad: matches!(
                            data.source,
                            crate::BusinessSource::ForeignServices { .. }
                        ),
                        gross: dollars(gross),
                        expenses: dollars(deductions),
                        interest: dollars(totals[5] + totals[6]),
                        half_se: business_half_se.get(&e.id).copied().unwrap_or(0),
                        plans: sep.by_business.get(&e.id).copied().unwrap_or(0)
                            + health.by_business.get(&e.id).copied().unwrap_or(0),
                        loss: positive(dollars(deductions) - dollars(gross)),
                        receipts: data.receipts(),
                    });
                }
                crate::BusinessActivity::Gambling { .. } => business_gross += data.receipts(),
                crate::BusinessActivity::Farming { .. } => {}
            },
            FederalIncome::Rental { data } => {
                rental_gross += data.rent + crate::method_change::increase(&data.method_changes);
            }
            FederalIncome::NonbusinessProperty { expenses, .. } => {
                property_expenses += exact::whole(*expenses);
            }
            FederalIncome::Royalty { data } => {
                royalty_gross += crate::royalties::filer_part(d, e, data.receipts);
                property_expenses += crate::royalties::filer_part_exact(
                    d,
                    e,
                    &crate::RentalExpense::total_exact(&data.expenses),
                );
            }
            _ => {}
        }
    }
    // Depletion also reduces the royalty net in AGI; line 3e counts the gross.
    property_expenses += royalty_total;
    let mut farm_rental_gross = 0;
    for source in crate::farm::sources(d) {
        if source.rental {
            farm_rental_gross += source.gross()
                + crate::farm_insurance_paper::income_delta(d, source.entry.id, source_claims);
        } else {
            business_gross += source.gross()
                + crate::farm_insurance_paper::income_delta(d, source.entry.id, source_claims);
        }
    }
    let business_gross = dollars(business_gross);
    let rental_gross = dollars(rental_gross);
    // A K-1 without a Schedule K-3 keeps its net amount as the gross income
    // on line 3e (conservative, one note naming it).
    if matches!(c.foreign_tax, ForeignTaxClaim::Ordinary { .. }) {
        for entry in allocated_entries {
            let without_k3 = match &entry.income {
                FederalIncome::Partnership { data } => data.schedule_k3.is_none(),
                FederalIncome::PartnershipActivities { data } => data.entity.schedule_k3.is_none(),
                FederalIncome::Scorp { data } => {
                    let data = &crate::ScorpK1::from(data);
                    data.schedule_k3.is_none()
                }
                _ => false,
            };
            let net: i64 =
                rental.k1_amounts.iter().filter(|v| v.id == entry.id).map(|v| v.net).sum();
            if without_k3 && net != 0 {
                let context = crate::interview::income_issue_context(entry);
                note_once_scoped(
                    r,
                    Message::new(
                        "WARN_FTC_K1_GROSS_ASSUMED",
                        alloc::vec![MessageArg::Nested(alloc::boxed::Box::new(context.label()))],
                    ),
                    context.alternatives(&[
                        crate::TaxSection::PassThrough,
                        crate::TaxSection::MoreCredits,
                    ]),
                );
            }
        }
    }
    let itemized_us = if use_itemized {
        foreign_reduced(
            repayment.deduction
                + gambling_loss
                + annuity_loss
                + investment_property_taxes
                + dollars(t.inherited_estate_tax + t.estate_interest_ird_tax),
        )
    } else {
        0
    };
    // The one regular-liability base (it already includes the Form 8814 tax) less the Form 4972
    // tax, which Form 1116 line 20 excludes, and a negative Form 8978 line 14.
    #[cfg(test)]
    {
        CREDIT_BASES.with(|bases| bases.borrow_mut().clear());
        record_credit_base("Form 1116 line 20", regular_liability);
    }
    let credit_regular = positive(regular_liability - lump_sum_tax - bba_credit);
    let foreign_basis = crate::form_1116::CreditBasis {
        regular: credit_regular,
        tentative,
        // Form 1116 line 18: Form 1040 line 11b less line 14 plus the enhanced
        // senior deduction, so the QBI and other Schedule 1-A deductions
        // reduce it; compute before the Form 1040 zero floor.
        worldwide: positive(agi - deduction - qbi_ded - additional.iter().sum::<i64>()),
        agi,
        amt_base,
        deduction,
        foreign_tax_deduction: if use_itemized {
            foreign_reduced(foreign_tax_deductible)
        } else {
            0
        },
        sanctioned_taxes: {
            let weights: Vec<i64> = foreign_tax_deduction.sanctioned.values().copied().collect();
            foreign_tax_deduction
                .sanctioned
                .keys()
                .cloned()
                .zip(crate::kernel::ratio::allocate_prefix(
                    if use_itemized { foreign_reduced(foreign_tax_deductible) } else { 0 },
                    &weights,
                ))
                .collect()
        },
        ira_deduction: ira.deduction,
        amt_deduction: foreign_amt_deductions,
        mortgage: [foreign_reduced(foreign_mortgage), foreign_mortgage],
        investment: [foreign_reduced(foreign_investment[0]), foreign_investment[1]],
        charity: [foreign_reduced(foreign_charity), foreign_charity],
        state_paid: if use_itemized { ded.income_tax_paid() } else { 0 },
        state_deduction: if use_itemized {
            foreign_reduced(
                exact::rounded(
                    &(exact::whole(salt) * exact::whole(dollars(ded.income_tax_paid()))
                        / phase::floor(salt_eligible.clone(), exact::whole(1))),
                )
                .expect("state tax allocation is bounded by its native SALT deduction"),
            )
        } else {
            0
        },
        foreign_wages: foreign.earned_wages,
        excluded_income: foreign.excluded,
        housing_deduction: foreign.housing_deduction,
        portfolio: interest + dividends + dollars(royalty_gross),
        taxable,
        qualified,
        preference: [
            [regular_tax_pref, regular_collectible, regular_unrecaptured],
            [amt_tax_pref, amt_tax_collectible, amt_tax_unrecaptured],
        ],
        capital_gross: [capital_gross, capital + loan_gain + dollars(t.k3_agi_items)],
        amt_capital_gross,
        // The AMT gross worksheet starts with regular AGI, so both systems
        // remove its regular property ordinary amount. Offset the existing
        // AMT long-term adjustment separately; do not subtract recapture twice.
        property_gross_correction: [
            crate::kernel::carry::native_balance(
                crate::kernel::carry::native_balance(
                    match crate::section1231::gross_property_with_claims(
                        d,
                        rental,
                        false,
                        source_claims,
                    ) {
                        Ok(value) => value,
                        Err(issue) => {
                            crate::section179_return::Issue::from(issue).report(r);
                            r.state = ResultState::Unsupported;
                            return core::ops::ControlFlow::Break(());
                        }
                    },
                    property_ordinary,
                ),
                property_character.as_ref().map_or(0, |v| positive(v.long_term)),
            ),
            crate::kernel::carry::native_balance(
                crate::kernel::carry::native_balance(
                    crate::kernel::carry::native_balance(
                        match crate::section1231::gross_property_with_claims(
                            d,
                            amt_rental,
                            true,
                            source_claims,
                        ) {
                            Ok(value) => value,
                            Err(issue) => {
                                crate::section179_return::Issue::from(issue).report(r);
                                r.state = ResultState::Unsupported;
                                return core::ops::ControlFlow::Break(());
                            }
                        },
                        property_ordinary,
                    ),
                    property_character.as_ref().map_or(0, |v| positive(v.long_term)),
                ),
                amt_property_character.as_ref().map_or(0, |v| v.long_term),
            ) + property_character.as_ref().map_or(0, |v| v.long_term),
        ],
        tax_refund: amt_tax_recovery,
        amt_income_adjustments: iso + dollars(t.private_activity),
        casualty: [
            if use_itemized { foreign_reduced(casualty_deduction) } else { 0 },
            if use_itemized { casualty_deduction } else { qualified_net_casualty },
        ],
        us_itemized: itemized_us,
        educator: if use_itemized { foreign_reduced(educator_itemized) } else { 0 },
        employee_pay: wages + foreign_wages,
        nol_deduction: nol_run.deduction,
        excess_business_loss,
        schedule_d: [
            crate::foreign_capital::ScheduleD {
                short: regular_capital.short,
                long: regular_capital.long,
                net_gain: regular_capital.net_gain,
            },
            crate::foreign_capital::ScheduleD {
                short: amt_capital_result.short,
                long: amt_capital_result.long,
                net_gain: amt_capital_result.net_gain,
            },
        ],
        elected,
        businesses: business_facts,
        business_exclusions: core::array::from_fn(|i| crate::form_1116::BusinessExclusionFacts {
            business_gross: business_exclusions[i].business_gross,
            related: business_exclusions[i].related,
            housing_deduction: business_exclusions[i].housing_deduction,
        }),
        // The tax came from the Qualified Dividends and Capital Gain Tax
        // Worksheet or the Schedule D Tax Worksheet (Form 8615 and
        // Schedule J tax cannot be mapped to the Worksheet for Line 18).
        line_18_mapped: regular_tax_pref == 0 || regular_tax == regular_tax_worksheet.tax(),
        // Form 1116 line 3b: Schedule 1 Part II adjustments not definitely related to any class of
        // income go on line 3b; the half SE tax, owner SEP, SE health, remitted
        // jury pay and savings penalty are definitely related to the U.S.
        // business, jury service and bank interest (Pub 514) and stay off Form
        // 1116, restored only to line 3e; student loan and vehicle loan
        // interest on line 4b; the Form 3903 moving deduction is incurred as a result
        // of, or incident to, the new post's compensation (Pub 514 "Definitely
        // related"; its source rule for a moving reimbursement is the new principal
        // place of work), so it also stays off Form 1116; the QBI and Schedule 1-A
        // deductions are definitely related to U.S. income and reduce only line 18; U.S.
        // Schedule C and rental expenses are definitely related to U.S. income,
        // so line 3e takes their gross income.
        adjustments: educator
            + housing_deduction
            + dollars(hsa)
            + msa_deduction
            + pension.deduction
            + alimony_paid
            + estate_67e,
        us_adjustments: half_se
            + sep_deduction
            + health.deduction
            + savings_penalty
            + jury_remitted
            + employee_deduction
            + moving_deduction,
        savings_penalty,
        penalty_interest: dollars(
            allocated_entries
                .iter()
                .map(|e| match &e.income {
                    FederalIncome::Interest { data: v }
                        if crate::FederalInterest::from(v).early_penalty != 0 =>
                    {
                        crate::FederalInterest::from(v).net_taxable()
                    }
                    FederalIncome::Oid { data: v }
                        if crate::FederalOid::from(v).early_penalty != 0 =>
                    {
                        crate::FederalOid::from(v).net_taxable()
                    }
                    _ => 0,
                })
                .sum(),
        ),
        student_loan: loan,
        vehicle_interest: additional[2],
        below_line: qbi_ded + additional.iter().sum::<i64>(),
        gross_correction: business_gross - business + rental_gross - rental.part_one
            + dollars(farm_rental_gross)
            - rental.farm_rental
            + match exact::dollars(&property_expenses)
                .filter(|amount| amount.checked_mul(100).is_some())
            {
                Some(amount) => amount,
                None => {
                    crate::issue_localized!(
                        r,
                        &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                        "Foreign tax credit gross income",
                        "FOREIGN_GROSS_CORRECTION_UNAVAILABLE",
                        "The required Form 1116 gross-income correction for royalties and nonbusiness property cannot be represented in its native amount. No final tax can be reported.",
                    );
                    withhold_required_source_result(r);
                    return core::ops::ControlFlow::Break(());
                }
            }
            + stock_loss,
        k3_correction: k3_basis.correction,
        k3_limits: k3_basis.limits.clone(),
    };
    let (foreign_credit, amt_credit, foreign_credit_sources) =
        crate::extended_rules::foreign_credit(d, foreign_basis.clone(), r);
    core::ops::ControlFlow::Continue(ForeignCreditOut {
        foreign_basis,
        foreign_credit,
        amt_credit,
        rental_gross,
        royalty_gross,
        foreign_credit_sources,
    })
}
