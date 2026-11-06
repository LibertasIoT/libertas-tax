// form_8615 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// Form 8615's age test (i8615 "Who Must File"): under 18, or 18, or a
// full-time student under 24, without earned income over half of support.
// The age is the day-before rule (2025 i1040: "A child born on January 1, 2008,
// is considered to be age 18 at the end of 2025"), not Rev. Rul. 2003-72.
pub(crate) fn kiddie_age(taxpayer: &FederalPerson) -> bool {
    let age = taxpayer.tax_age();
    age < 18
        || ((age == 18 || (age < 24 && taxpayer.full_time_student))
            && taxpayer.support_share != SupportShare::MoreThanHalf)
}
// Form 8615 applies: the child's age test, not filing jointly, and a parent
// alive at the end of 2026. Only then does a Schedule C or F business's
// income source enter the calculation (its earned income).
pub(crate) fn young_8615(joint: bool, p: &FederalPeople) -> bool {
    !joint && kiddie_age(&p.taxpayer) && p.taxpayer.parent_alive
}
// Form 8615 line 1's calculation line, in cents.
pub(crate) const FORM_8615_UNEARNED: &str = "kiddie_tax_unearned_income";
// Form 8615 applies to the return (i8615 "Who Must File"): the person
// tests and more than $2,700 of unearned income; None when the estimate
// cannot state line 1 yet.
pub(crate) fn form_8615_applies(result: &FederalResult) -> Option<bool> {
    result
        .lines
        .iter()
        .find(|l| l.id == FORM_8615_UNEARNED)
        .map(|l| l.amount > crate::year_constants::KIDDIE_UNEARNED_THRESHOLD_DOLLARS * 100)
}

pub(super) struct KiddieIncomeIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) business_half_se: &'a alloc::collections::BTreeMap<i64, i64>,
    pub(super) kiddie_age: bool,
    pub(super) young_8615: bool,
    pub(super) partner_young_complete: bool,
    pub(super) use_itemized: bool,
    pub(super) unearned: i64,
    pub(super) partner_young_loss_addback: i64,
    pub(super) partner_young_earned: i64,
    pub(super) savings_bond_exclusion: i64,
    pub(super) wages: i64,
    pub(super) iso_unreported: i64,
    pub(super) foreign_wages: i64,
    pub(super) benefit_wages: i64,
    pub(super) adoption_wages: i64,
    pub(super) agi: i64,
    pub(super) business: i64,
    pub(super) investment_property_taxes: i64,
    pub(super) foreign_portfolio_tax: i64,
    pub(super) itemized_reduction: i64,
    pub(super) itemized: i64,
    pub(super) investment_interest_deduction: i64,
    pub(super) gambling_loss: i64,
    pub(super) taxable: i64,
    pub(super) excluded: i64,
}

pub(super) struct KiddieIncomeOut {
    pub(super) service_businesses: bool,
    pub(super) classified_business_earned: Option<i64>,
    pub(super) unearned: i64,
    pub(super) investment_tax_deduction: i64,
    pub(super) child_direct_deductions: i64,
    pub(super) kiddie_deduction: i64,
    pub(super) parent_rate_required: bool,
}

pub(super) fn kiddie_income_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    i: KiddieIncomeIn<'_>,
) -> core::ops::ControlFlow<(), KiddieIncomeOut> {
    let &Pass { d, p, j, allocated_entries, selected_regular, office_context, context, .. } = pass;
    let source_claims = context.source_claims;
    let KiddieIncomeIn {
        rental,
        business_half_se,
        kiddie_age,
        young_8615,
        partner_young_complete,
        use_itemized,
        unearned,
        partner_young_loss_addback,
        partner_young_earned,
        savings_bond_exclusion,
        wages,
        iso_unreported,
        foreign_wages,
        benefit_wages,
        adoption_wages,
        agi,
        business,
        investment_property_taxes,
        foreign_portfolio_tax,
        itemized_reduction,
        itemized,
        investment_interest_deduction,
        gambling_loss,
        taxable,
        excluded,
    } = i;
    // Form 8615 line 5 cannot exceed line 4. Form 2555 substitutes the
    // foreign-income stacking amount for ordinary taxable income on line 4.
    // With no earned income the IRS directs line 1 to AGI; otherwise the
    // income worksheet deducts the savings penalty, not all AGI adjustments.
    let has_business =
        allocated_entries.iter().any(|e| matches!(e.income, FederalIncome::Business { .. }));
    // A farm's stored Services is its unasked source, not services
    // (BusinessIncomeSource::for_8615).
    let farming = |data: &crate::FederalBusiness| {
        matches!(data.activity, crate::BusinessActivity::Farming { .. })
    };
    let services = |data: &crate::FederalBusiness| {
        matches!(
            data.income_source.for_8615(farming(data), 0).0,
            crate::BusinessIncomeSource::Services
        )
    };
    let schedule_c_service_businesses = allocated_entries.iter().all(|e| match &e.income {
        FederalIncome::Business { data } => services(data),
        _ => true,
    });
    let service_businesses = schedule_c_service_businesses
        && allocated_entries.iter().all(|e| match &e.income {
            FederalIncome::Partnership { data } => {
                matches!(data.income_source, Some(crate::BusinessIncomeSource::Services))
            }
            _ => true,
        });
    let has_service_receipts = allocated_entries.iter().any(|e| {
        matches!(&e.income,
        FederalIncome::Business { data } if services(data) && data.receipts() > 0)
    });
    // Partnership net income is in the unearned-income base above.
    let unearned =
        unearned + partner_young_loss_addback - partner_young_earned - savings_bond_exclusion;
    let mut classified_business_earned = None;
    if !j && kiddie_age && p.taxpayer.parent_alive {
        let businesses: Vec<_> = allocated_entries
            .iter()
            .filter_map(|e| match &e.income {
                FederalIncome::Business { data } => Some((e, data)),
                _ => None,
            })
            .collect();
        let mut classified_total = Some(0);
        let mut service_profit = 0;
        for (entry, data) in &businesses {
            let id = &entry.id;
            let profit = dollars(
                data.receipts()
                    - match crate::section179_return::expenses_with_claims(
                        d,
                        *id,
                        data,
                        selected_regular,
                        Some(office_context),
                        source_claims,
                    )
                    .map(|expense| {
                        expense
                            - selected_regular
                                .map_or(0, |allocation| allocation.ordinary_limit_adjustment(*id))
                    }) {
                        Ok(value) => value,
                        Err(issue) => {
                            crate::section179_return::Issue::from(issue).report(r);
                            r.state = ResultState::Unsupported;
                            return core::ops::ControlFlow::Break(());
                        }
                    },
            );
            // A source that is not known is taken as Capital, with a warning.
            let (source, assumed) = data.income_source.for_8615(farming(data), profit);
            if assumed {
                note_once_scoped(
                    r,
                    Message::new(
                        "WARN_KIDDIE_SOURCE_ASSUMED_CAPITAL",
                        alloc::vec![crate::interview::income_display_arg(entry)],
                    ),
                    crate::interview::income_issue_context(entry),
                );
            }
            // A Schedule C business on More: Nothing stores Services (the
            // usual answer for a young filer): say so, and how to change it.
            if matches!(data.activity, crate::BusinessActivity::Ordinary { .. })
                && matches!(data.income_source, crate::BusinessIncomeSource::Services)
            {
                r.review_notes.push(ReviewNote::localized(
                    #[cfg(test)]
                    crate::kiddie::services_business_note(&if entry.label.trim().is_empty() {
                        alloc::format!("{}", entry.id)
                    } else {
                        entry.label.clone()
                    }),
                    Message::new(
                        "NOTE_YOUNG_FILER_SERVICE_BUSINESS",
                        alloc::vec![crate::interview::income_display_arg(entry)],
                    ),
                ));
            }
            let amount = source.earned(profit, business_half_se.get(id).copied().unwrap_or(0));
            if matches!(source, crate::BusinessIncomeSource::Services) {
                service_profit += dollars(
                    data.receipts()
                        - match crate::section179_return::expenses_with_claims(
                            d,
                            *id,
                            data,
                            selected_regular,
                            Some(office_context),
                            source_claims,
                        )
                        .map(|expense| {
                            expense
                                - selected_regular.map_or(0, |allocation| {
                                    allocation.ordinary_limit_adjustment(*id)
                                })
                        }) {
                            Ok(value) => value,
                            Err(issue) => {
                                crate::section179_return::Issue::from(issue).report(r);
                                r.state = ResultState::Unsupported;
                                return core::ops::ControlFlow::Break(());
                            }
                        },
                );
            } else {
                classified_total = classified_total.zip(amount).map(|(total, value)| total + value);
            }
            if let Some(amount) = amount {
                line(
                    r,
                    &alloc::format!("business_{}_kiddie_earned_income", id),
                    amount,
                    &["income.business.income_source", "self_employment_tax"],
                    "https://www.irs.gov/instructions/i8615",
                    msg!("EXPLAIN_BUSINESS_N_KIDDIE_EARNED_INCOME"),
                );
            }
        }
        classified_total = classified_total.map(|v| v + positive(service_profit));
        // Partner earnings were already removed from the unearned base;
        // this adjustment is only for Schedule C/F sources.
        if rental.partners.is_empty() || (partner_young_complete && rental.inputs_complete) {
            if businesses.is_empty() && !rental.partners.is_empty() {
                classified_business_earned = Some(0);
            } else if !businesses.is_empty() {
                // Form 8615 earned income is the classified amount for every
                // business; the dependent standard deduction keeps the
                // worksheet's own definition (no coupling between them).
                classified_business_earned = classified_total;
            }
        }
    }

    let unearned = if partner_young_earned == 0
        && wages == 0
        && iso_unreported == 0
        && foreign_wages == 0
        && benefit_wages == 0
        && adoption_wages == 0
        && (!has_business || (classified_business_earned == Some(0) && !has_service_receipts))
    {
        positive(agi)
    } else if let Some(business_earned) = classified_business_earned {
        // Alternate worksheet adds back a net business loss before subtracting
        // reasonable compensation; positive profit remains in gross income.
        positive(unearned + positive(business) - business_earned)
    } else {
        positive(unearned)
    };
    let investment_tax_deduction = if use_itemized {
        investment_property_taxes + foreign_portfolio_tax
            - fraction(
                itemized_reduction,
                investment_property_taxes + foreign_portfolio_tax,
                itemized.max(1),
            )
    } else {
        0
    };
    let child_investment_interest_deduction = investment_interest_deduction
        - fraction(itemized_reduction, investment_interest_deduction, itemized.max(1));
    // Casual wagering losses are connected to ordinary unearned winnings,
    // not qualified dividends or capital gains. Use only the deduction actually
    // taken, after the section 165(d) limit and the section 68 reduction.
    let child_gambling_deduction = if use_itemized {
        gambling_loss - fraction(itemized_reduction, gambling_loss, itemized.max(1))
    } else {
        0
    };
    let child_direct_deductions =
        investment_tax_deduction + child_investment_interest_deduction + child_gambling_deduction;
    let kiddie_deduction = phase::floor(
        crate::year_constants::KIDDIE_UNEARNED_THRESHOLD_DOLLARS,
        crate::year_constants::KIDDIE_BASE_DEDUCTION_DOLLARS + child_direct_deductions,
    );
    line(
        r,
        "investment_property_tax_deduction",
        investment_tax_deduction,
        &["deductions.choice", "itemized_deduction_reduction"],
        "https://www.irs.gov/instructions/i8960",
        msg!("EXPLAIN_INVESTMENT_PROPERTY_TAX_DEDUCTION"),
    );
    let parent_rate_required = unearned > crate::year_constants::KIDDIE_UNEARNED_THRESHOLD_DOLLARS
        && !j
        && p.taxpayer.parent_alive
        && kiddie_age;
    // Form 8615 line 1 whenever the filer passes Form 8615's person tests:
    // the walk asks the parent's return exactly when it exceeds $2,700
    // (interview::parent_tax_in_walk).
    if young_8615 {
        line(
            r,
            FORM_8615_UNEARNED,
            unearned,
            &["income"],
            "https://www.irs.gov/instructions/i8615",
            msg!("EXPLAIN_FEDERAL_RULES_9"),
        );
    }

    if parent_rate_required {
        line(
            r,
            "kiddie_tax_direct_gambling_deduction",
            child_gambling_deduction,
            &["income.gambling", "deduction", "itemized_deduction_reduction"],
            "https://www.irs.gov/instructions/i8615",
            msg!("EXPLAIN_KIDDIE_TAX_DIRECT_GAMBLING_DEDUCTION"),
        );
    }
    if parent_rate_required && (taxable + positive(excluded) == 0 || unearned <= kiddie_deduction) {
        line(
            r,
            "kiddie_tax_net_unearned_income",
            0,
            &["income", "taxable_income", "foreign_earned_exclusion"],
            "https://www.irs.gov/instructions/i8615",
            msg!("EXPLAIN_KIDDIE_TAX_NET_UNEARNED_INCOME"),
        );
    }
    core::ops::ControlFlow::Continue(KiddieIncomeOut {
        service_businesses,
        classified_business_earned,
        unearned,
        investment_tax_deduction,
        child_direct_deductions,
        kiddie_deduction,
        parent_rate_required,
    })
}

pub(super) struct KiddieTaxIn<'a> {
    pub(super) foreign: &'a crate::extended_rules::ForeignResult,
    pub(super) investment_facts: Option<&'a crate::InvestmentInterest>,
    pub(super) parent_rate_required: bool,
    pub(super) use_itemized: bool,
    pub(super) service_businesses: bool,
    pub(super) classified_business_earned: Option<i64>,
    pub(super) elected: [i64; 2],
    pub(super) regular_tax: i64,
    pub(super) taxable: i64,
    pub(super) excluded: i64,
    pub(super) unearned: i64,
    pub(super) kiddie_deduction: i64,
    pub(super) pref: i64,
    pub(super) investment_interest_deduction: i64,
    pub(super) itemized_reduction: i64,
    pub(super) itemized: i64,
    pub(super) annuity_loss: i64,
    pub(super) agi: i64,
    pub(super) deduction: i64,
    pub(super) standard: i64,
    pub(super) qualified: i64,
    pub(super) child_direct_deductions: i64,
    pub(super) investment_tax_deduction: i64,
    pub(super) net_capital_gain: i64,
    pub(super) schedule_d_18: i64,
    pub(super) schedule_d_19: i64,
}

pub(super) struct KiddieTaxOut<'a> {
    pub(super) regular_tax: i64,
    pub(super) kiddie_credit_tax: Option<crate::kiddie::TaxWorksheet<'a>>,
}

pub(super) fn kiddie_tax_phase<'a>(
    pass: &Pass<'a>,
    r: &mut FederalResult,
    t: &Totals,
    i: KiddieTaxIn<'_>,
) -> KiddieTaxOut<'a> {
    let &Pass { p, s, .. } = pass;
    let KiddieTaxIn {
        foreign,
        investment_facts,
        parent_rate_required,
        use_itemized,
        service_businesses,
        classified_business_earned,
        elected,
        mut regular_tax,
        taxable,
        excluded,
        unearned,
        kiddie_deduction,
        pref,
        investment_interest_deduction,
        itemized_reduction,
        itemized,
        annuity_loss,
        agi,
        deduction,
        standard,
        qualified,
        child_direct_deductions,
        investment_tax_deduction,
        net_capital_gain,
        schedule_d_18,
        schedule_d_19,
    } = i;
    let mut kiddie_credit_tax = None;
    if parent_rate_required && taxable + positive(excluded) > 0 && unearned > kiddie_deduction {
        let preferential_interest = if pref == 0 || investment_interest_deduction == 0 {
            Some(0)
        } else {
            investment_facts
                .as_ref()
                .and_then(|v| v.preferential_deduction(investment_interest_deduction))
                .map(|amount| amount - fraction(itemized_reduction, amount, itemized.max(1)))
        };
        let ordinary_child = excluded == 0
            && (!use_itemized
                || annuity_loss == 0
                    && t.inherited_estate_tax == 0
                    && t.estate_interest_ird_tax == 0)
            && elected == [0; 2]
            && preferential_interest.is_some()
            && !foreign.election()
            && (service_businesses || classified_business_earned.is_some());
        if let Some(parent) = &p.parent_tax {
            let ready = parent.complete && parent.ordinary && ordinary_child;
            // Each refusal names its Form 8615 lines; the parent's answers
            // are the page "Your parent's return (Form 8615)".
            crate::require_localized!(
                r,
                parent.complete,
                &crate::IssueContext::Section(crate::TaxSection::ParentTax),
                "Investment tax",
                "ISSUE_FORM_8615_LINES_A_C_6_AND_7_WHICH_PARENT_S_RETURN_TO_USE_OR_A_BROTH_AE8D3F1B",
                "Form 8615 lines A–C, 6 and 7: which parent's return to use, or a brother's or sister's Form 8615, is not settled on your parent's return page; the tax at your parent's rate waits for it.",
            );
            crate::require_localized!(
                r,
                parent.ordinary,
                &crate::IssueContext::Section(crate::TaxSection::ParentTax),
                "Investment tax",
                "ISSUE_FORM_8615_LINES_9_AND_15_WITH_SCHEDULE_J_ON_THE_PARENT_S_RETURN",
                "Form 8615 lines 9 and 15 with Schedule J on your parent's or a brother's or sister's return (on your parent's return page) are not calculated yet.",
            );
            crate::require_localized!(
                r,
                ordinary_child,
                &crate::IssueContext::Section(crate::TaxSection::OtherItemized),
                "Investment tax",
                "ISSUE_ANNUITY_LOSSES_INHERITED_ESTATE_TAX_DEDUCTIONS_PREFERENTIAL_INCOME_58601C46",
                "Annuity losses, inherited estate-tax deductions, preferential-income elections, unresolved investment-interest attribution (including limited mixed expenses), capital-dependent business income and foreign exclusions need additional Form 8615 worksheets.",
            );
            if ready {
                let worksheet = crate::kiddie::TaxWorksheet {
                    parent,
                    unearned,
                    deduction: kiddie_deduction,
                    agi,
                    return_deduction: if use_itemized { deduction } else { standard },
                    qualified,
                    preference: pref,
                    direct: [
                        child_direct_deductions,
                        investment_tax_deduction + preferential_interest.unwrap_or(0),
                    ],
                    capital: [net_capital_gain, schedule_d_18, schedule_d_19],
                    status: s.filing_status(),
                };
                let (net, gains, allocated, tax) = worksheet.calculate(taxable);
                kiddie_credit_tax = Some(worksheet);
                line(
                    r,
                    "kiddie_tax_preferential_interest",
                    preferential_interest.unwrap_or(0),
                    &["deduction"],
                    "https://www.irs.gov/instructions/i8615",
                    msg!("EXPLAIN_KIDDIE_TAX_PREFERENTIAL_INTEREST"),
                );
                emit_lines(
                    r,
                    &mut (),
                    [
                        ("kiddie_tax_allocated_dividends", gains[0]),
                        ("kiddie_tax_allocated_capital_gain", gains[1]),
                        (
                            "kiddie_tax_allocated_28_percent_gain",
                            crate::kiddie::special_allocation(
                                schedule_d_18,
                                gains[1],
                                net_capital_gain,
                            ),
                        ),
                        (
                            "kiddie_tax_allocated_25_percent_gain",
                            crate::kiddie::special_allocation(
                                schedule_d_19,
                                gains[1],
                                net_capital_gain,
                            ),
                        ),
                    ],
                    |(id, amount), _, _| {
                        Some(calculation_row(
                            None,
                            id,
                            LineValue::MoneyDollars(amount),
                            &["income", "deduction"],
                            "https://www.irs.gov/instructions/i8615",
                            msg!("EXPLAIN_FEDERAL_RULES_11"),
                        ))
                    },
                );
                emit_lines(
                    r,
                    &mut (),
                    [
                        ("kiddie_tax_original_net_capital_gain", net_capital_gain),
                        ("kiddie_tax_original_28_percent_gain", schedule_d_18),
                        ("kiddie_tax_original_25_percent_gain", schedule_d_19),
                    ],
                    |(id, amount), _, _| {
                        Some(calculation_row(
                            None,
                            id,
                            LineValue::MoneyDollars(amount),
                            &["income", "screening.capital_history"],
                            "https://www.irs.gov/instructions/i8615",
                            msg!("EXPLAIN_FEDERAL_RULES_12"),
                        ))
                    },
                );
                regular_tax = tax;
                line(
                    r,
                    "kiddie_tax_net_unearned_income",
                    net,
                    &["income", "taxable_income"],
                    "https://www.irs.gov/instructions/i8615",
                    msg!("EXPLAIN_KIDDIE_TAX_NET_UNEARNED_INCOME_2"),
                );
                line(
                    r,
                    "kiddie_tax_parent_share",
                    allocated,
                    &["people.parent_tax", "kiddie_tax_net_unearned_income"],
                    "https://www.irs.gov/instructions/i8615",
                    msg!("EXPLAIN_KIDDIE_TAX_PARENT_SHARE"),
                );
            }
        } else {
            crate::issue_localized!(
                r,
                &crate::IssueContext::Section(crate::TaxSection::ParentTax),
                "Investment tax",
                "ISSUE_FORM_8615_LINES_6_TO_8_ENTER_YOUR_PARENT_S_RETURN_AND_ANY_BROTHER_S_6F06ED02",
                "Form 8615 lines 6 to 8: enter your parent's return, and any brother's or sister's Form 8615 that uses it, on your parent's return page.",
            );
        }
    }
    KiddieTaxOut { regular_tax, kiddie_credit_tax }
}

pub(super) struct PartnershipEarnedSourcesIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) business_half_se: &'a alloc::collections::BTreeMap<i64, i64>,
}

pub(super) struct PartnershipEarnedSourcesOut {
    pub(super) partner_young_earned: i64,
    pub(super) partner_young_loss_addback: i64,
    pub(super) partner_standard_earned: i64,
    pub(super) partner_young_complete: bool,
    pub(super) partner_standard_complete: bool,
    pub(super) young_8615: bool,
    pub(super) dependent_standard: bool,
}

pub(super) fn partnership_earned_sources_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    i: PartnershipEarnedSourcesIn<'_>,
) -> PartnershipEarnedSourcesOut {
    let &Pass { p, j, allocated_entries, .. } = pass;
    let PartnershipEarnedSourcesIn { rental, business_half_se } = i;
    let mut partner_young_earned = 0;
    let mut partner_young_complete = true;
    let mut partner_young_loss_addback = 0;
    let mut partner_standard_earned = 0;
    let mut partner_standard_complete = true;
    let young_8615 = young_8615(j, p);
    let dependent_standard = dependent_standard(p);
    if young_8615 || dependent_standard {
        for pool in &rental.partners {
            let Some(entry) = allocated_entries.iter().find(|e| e.id == pool.id) else {
                continue;
            };
            let FederalIncome::Partnership { data } = &entry.income else {
                continue;
            };
            let earned_source_net = pool.net
                - data
                    .prior_fuel_credit
                    .as_ref()
                    .map_or(0, |prior| dollars(prior.taxable_amount()));
            if dependent_standard {
                match data.young_earned(earned_source_net, 0) {
                    Ok((amount, assumed)) => {
                        partner_standard_earned += amount;
                        if assumed {
                            note_once_scoped(
                                r,
                                Message::new(
                                    "WARN_KIDDIE_SOURCE_ASSUMED_CAPITAL",
                                    alloc::vec![crate::interview::income_display_arg(entry)],
                                ),
                                crate::interview::income_issue_context(entry),
                            );
                        }
                    }
                    Err((key, _message)) => {
                        partner_standard_complete = false;
                        crate::issue_localized!(
                            r,
                            &if key == "ISSUE_YOUNG_PARTNER_HISTORICAL_LOSSES" {
                                crate::interview::income_issue_context(entry).manual()
                            } else {
                                crate::interview::income_issue_context(entry)
                            },
                            "Dependent partner earned income",
                            key,
                            _message,
                        );
                    }
                }
            }
            if young_8615 {
                match data.young_earned(
                    earned_source_net,
                    business_half_se.get(&pool.id).copied().unwrap_or(0),
                ) {
                    Ok((amount, assumed)) => {
                        partner_young_earned += amount;
                        if assumed {
                            note_once_scoped(
                                r,
                                Message::new(
                                    "WARN_KIDDIE_SOURCE_ASSUMED_CAPITAL",
                                    alloc::vec![crate::interview::income_display_arg(entry)],
                                ),
                                crate::interview::income_issue_context(entry),
                            );
                        }
                        if let Some((_, se)) =
                            rental.partner_se.iter().find(|(id, _)| *id == pool.id)
                        {
                            let addback = positive(-se.earnings);
                            partner_young_loss_addback += addback;
                            r.review_notes.push(ReviewNote::localized(
                                #[cfg(test)] alloc::format!("Partnership document {}, young-filer earned income: ${amount}; net self-employment loss addback: ${addback}. Personal-service classification for Form 8615 is separate from Schedule SE and passive participation. Dependent standard-deduction earnings use the same source facts and a separate half-SE adjustment.", pool.id),
                                Message::new("NOTE_PARTNERSHIP_YOUNG_FILER_EARNED", alloc::vec![
                                    MessageArg::Number(pool.id), MessageArg::usd_dollars(amount), MessageArg::usd_dollars(addback),
                                ]),
                            ));
                        } else {
                            partner_young_complete = false;
                            crate::issue_localized!(
                                r,
                                &crate::interview::income_issue_context(entry),
                                "Young-partner earned income",
                                "ISSUE_RESOLVE_THE_PARTNERSHIP_SELF_EMPLOYMENT_RESULT_BEFORE_APPLYING_THE_D0F267FA",
                                "Resolve the partnership self-employment result before applying the Form 8615 alternate worksheet.",
                            );
                        }
                    }
                    Err((key, _message)) => {
                        partner_young_complete = false;
                        crate::issue_localized!(
                            r,
                            &if key == "ISSUE_YOUNG_PARTNER_HISTORICAL_LOSSES" {
                                crate::interview::income_issue_context(entry).manual()
                            } else {
                                crate::interview::income_issue_context(entry)
                            },
                            "Young-partner earned income",
                            key,
                            _message,
                        );
                    }
                }
            }
        }
    }
    PartnershipEarnedSourcesOut {
        partner_young_earned,
        partner_young_loss_addback,
        partner_standard_earned,
        partner_young_complete,
        partner_standard_complete,
        young_8615,
        dependent_standard,
    }
}
