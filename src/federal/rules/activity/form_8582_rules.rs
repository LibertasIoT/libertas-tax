// form_8582 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn collect_royalty(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    _r: &mut FederalResult,
    data: &crate::FederalRoyalty,
) -> Result<(), CollectFailure> {
    let Source { e, .. } = *src;
    let amount = exact::whole(data.receipts) - RentalExpense::total_exact(&data.expenses);
    t.royalties += &amount;
    t.injured_other_sources.insert(e.id, Some(amount));
    Ok(())
}

pub(super) fn collect_nonbusiness_property(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    (kind, receipts, expenses): (&NonbusinessPropertyKind, &i64, &i64),
) -> Result<(), CollectFailure> {
    let Collect { d, form_context, .. } = *cx;
    let Source {
        e,
        context,
        #[cfg(test)]
        source,
        ..
    } = *src;
    // An unsupported property was left out before the rules ran.
    match kind {
        NonbusinessPropertyKind::PersonalRental { assets, .. } => {
            let mid_quarter =
                crate::depreciation::mid_quarter_with_macrs(d, Some(form_context.macrs().as_ref()));
            crate::require_localized!(
                r,
                crate::depreciation::consistent_elections(d),
                context,
                source,
                "ISSUE_USE_CONSISTENT_BONUS_AND_GDS_METHOD_ELECTIONS_FOR_EACH_RECOVERY_CLA_1A530B8D",
                "Use consistent bonus and GDS method elections for each recovery class across businesses and nonbusiness rentals.",
            );
            for (index, asset) in assets.iter().enumerate() {
                crate::require_localized!(
                    r,
                    asset.qualifies_nonbusiness_rental(),
                    context,
                    source,
                    asset.nonmacrs_issue().unwrap_or("ISSUE_CONFIRM_QUALIFYING_PURCHASED_RENTAL_EQUIPMENT_100_INCOME_PRODUCING_65B44F46"),
                    "Confirm qualifying purchased rental equipment, 100% income-producing use, no section 179 election or carryforward, and complete depreciation records. Listed property, vehicles, ADS, changed use and other basis treatment need separate worksheets.",
                );
                let macrs = form_context.macrs();
                let invocation = macrs.context(e.owner.owned(), Some(e.id), asset);
                let history_needed = asset.disposal != crate::AssetDisposal::No
                    || asset.partial.is_some()
                    || asset.earlier_business_percent.is_some();
                crate::require_localized!(
                    r,
                    !history_needed || asset.macrs_history_issue_in_context(invocation).is_none(),
                    context,
                    source,
                    "ASSET_ADS_ORIGINAL_HISTORY_UNAVAILABLE",
                    "The property's actual recovery history is needed for disposition or recapture."
                );
                let unavailable = asset.macrs_source_issue_in_context(invocation).is_some();
                let regular = crate::nonmacrs::asset_depreciation_amounts_for_view(
                    e.owner.owned(),
                    Some(e.id),
                    core::slice::from_ref(asset),
                    mid_quarter,
                    false,
                    &macrs,
                );
                let alternative = crate::nonmacrs::asset_depreciation_amounts_for_view(
                    e.owner.owned(),
                    Some(e.id),
                    core::slice::from_ref(asset),
                    mid_quarter,
                    true,
                    &macrs,
                );
                let bonus = regular
                    .classes
                    .get("14a")
                    .and_then(crate::kernel::ratio::exact::rounded)
                    .unwrap_or(0);
                let regular = regular.expense_cents(0).expect("validated rental equipment amount");
                let alternative =
                    alternative.expense_cents(0).expect("validated rental equipment amount");
                emit_lines(
                    r,
                    &mut (),
                    [
                        ("bonus", bonus),
                        ("macrs", regular - bonus),
                        ("amt_depreciation", alternative),
                        (
                            "amt_remaining_basis",
                            asset.amt_projection_or_self().opening_basis() - alternative,
                        ),
                        ("remaining_basis", asset.opening_basis() - regular),
                    ]
                    .into_iter()
                    .filter(|(suffix, _)| !unavailable || !suffix.ends_with("basis")),
                    |(suffix, amount), _, _| {
                        Some(calculation_row(
                            (asset.nonmacrs()
                                && crate::depreciation::assets_need_form_4562(assets))
                            .then(|| crate::depreciation::form_4562_line(asset, suffix))
                            .flatten()
                            .map(|line| FormLine::subrow(TaxForm::Form4562, line)),
                            &alloc::format!("personal_rental_{}_asset_{}_{suffix}", e.id, index),
                            LineValue::MoneyCents(amount),
                            &["income.nonbusiness_property.assets"],
                            "https://www.irs.gov/publications/p946",
                            msg!("EXPLAIN_PERSONAL_RENTAL_N_ASSET_N_N"),
                        ))
                    },
                );
                personal_equipment_sale(t, r, e, index, assets, mid_quarter, invocation);
            }
            if let Err(issue) = form_4562_asset_class_lines(
                d,
                r,
                e.id,
                assets,
                "personal_rental",
                "income.nonbusiness_property.assets",
                Some(form_context.macrs().as_ref()),
            ) {
                crate::section179_return::Issue::Prepared(issue).report(r);
            }
            let depreciation = kind.rental_depreciation(
                mid_quarter,
                false,
                e.owner.owned(),
                e.id,
                form_context.macrs().as_ref(),
            );
            t.personal_rent += receipts;
            t.personal_rent_expenses += expenses + depreciation;
            t.personal_rent_depreciation += depreciation;
            // Pub. 925: recharacterize positive rental income when
            // less than 30% of unadjusted basis is depreciable.
            let investment = [false, true].map(|alternative| {
                if kind.investment_rental() {
                    positive(
                        receipts
                            - expenses
                            - kind.rental_depreciation(
                                mid_quarter,
                                alternative,
                                e.owner.owned(),
                                e.id,
                                form_context.macrs().as_ref(),
                            )
                            - kind.prior_rental_loss(alternative),
                    )
                } else {
                    0
                }
            });
            for (total, amount) in t.investment_rent.iter_mut().zip(investment) {
                *total += amount;
            }
            line_cents(
                r,
                &alloc::format!("personal_rent_{}_investment_income", e.id),
                investment[0],
                &["income.nonbusiness_property"],
                "https://www.irs.gov/publications/p925",
                msg!("EXPLAIN_PERSONAL_RENT_N_INVESTMENT_INCOME"),
            );
        }
    }
    Ok(())
}

pub(super) fn collect_rental(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::FederalRental,
    mid_quarter: bool,
) -> Result<(), CollectFailure> {
    let Collect { form_context, .. } = *cx;
    let Source {
        e,
        context,
        #[cfg(test)]
        source,
        ..
    } = *src;
    crate::extended_rules::rental_expense_elections(
        r,
        &data.expenses,
        context,
        #[cfg(test)]
        source,
        &alloc::format!("rental_{}", e.id),
    );
    // Rentals outside ordinary scope are left out with a notice
    // by unknowns::supported.
    rental_personal_part(form_context, t, r, e, data);
    for (index, asset) in data.assets.iter().enumerate() {
        t.personal_part(
            r,
            e.id,
            crate::BusinessAsset::display_arg(&data.assets, index, 0),
            asset.sale_for(mid_quarter, false),
        );
    }
    Ok(())
}

pub(super) struct Form8582CreditCapacityIn<'a> {
    pub(super) regular_tax_worksheet: &'a crate::income_tax::RegularTaxWorksheet,
    pub(super) kiddie_credit_tax: &'a Option<crate::kiddie::TaxWorksheet<'a>>,
    pub(super) averaged_tax: &'a crate::farm_averaging::Calculation<'a>,
    pub(super) rental: &'a crate::extended_rules::RentalResult,
}

pub(super) fn form_8582_credit_capacity_phase(
    pass: &Pass,
    i: Form8582CreditCapacityIn<'_>,
) -> crate::general_business_credit::PassiveCreditCapacity {
    let &Pass { s, .. } = pass;
    let Form8582CreditCapacityIn { regular_tax_worksheet, kiddie_credit_tax, averaged_tax, rental } =
        i;
    // i8582cr: retain the tax method used on the return and hold
    // AGI-dependent items fixed when reducing taxable income.
    let tax_less = |amount: i64| {
        let current = regular_tax_worksheet.less(amount);
        if let Some(worksheet) = &kiddie_credit_tax {
            worksheet.calculate(current.taxable).3
        } else if let Some(worksheet) = averaged_tax.comparison {
            worksheet.tax_value(current)
        } else {
            current.tax()
        }
    };
    let base = tax_less(0);
    let separate = s.filing_status() == FederalStatus::Separate;
    // Section 469(i)(5): $25,000, or $12,500 living apart all year when
    // married filing separately and nothing when living together; the
    // rental losses Form 8582 line 9 allowed use it first.
    let remaining_allowance = if separate && !s.lived_apart_all_year() {
        0
    } else {
        positive(
            if separate {
                crate::year_constants::PASSIVE_ALLOWANCE_SEPARATE_DOLLARS
            } else {
                crate::year_constants::PASSIVE_ALLOWANCE_DOLLARS
            } - rental.special_allowance,
        )
    };
    // Form 8582-CR Part III (section 469(i)(3)): the rehabilitation credit's
    // $25,000 allowance, reduced by half of the income above $200,000
    // ($100,000 married filing separately; the form's lines 21-24 halve
    // $250,000 and $125,000), after the losses' allowance.
    let completed = !separate || s.lived_apart_all_year();
    let threshold = if !completed {
        0
    } else if separate {
        125_000
    } else {
        250_000
    };
    let excess = positive(threshold - rental.modified_agi);
    let limit = if completed {
        phase::cap(
            phase::remaining_fraction(excess, 1, 2),
            [if separate {
                crate::year_constants::PASSIVE_ALLOWANCE_SEPARATE_DOLLARS
            } else {
                crate::year_constants::PASSIVE_ALLOWANCE_DOLLARS
            }],
        )
    } else {
        0
    };
    let remaining = positive(limit - rental.special_allowance);
    crate::general_business_credit::PassiveCreditCapacity {
        active: {
            let threshold = if completed { if separate { 75_000 } else { 150_000 } } else { 0 };
            let excess = positive(threshold - rental.modified_agi);
            let limit = if completed {
                phase::cap(
                    fraction(excess, 1, 2),
                    [if separate {
                        crate::year_constants::PASSIVE_ALLOWANCE_SEPARATE_DOLLARS
                    } else {
                        crate::year_constants::PASSIVE_ALLOWANCE_DOLLARS
                    }],
                )
            } else {
                0
            };
            let remaining = positive(limit - rental.special_allowance);
            crate::general_business_credit::RehabilitationAllowance {
                completed,
                threshold,
                income: rental.modified_agi,
                excess,
                limit,
                used: rental.special_allowance,
                remaining,
                tax: base - tax_less(remaining),
            }
        },
        net_passive_income_tax: if rental.net_passive_income > 0 {
            base - tax_less(rental.net_passive_income)
        } else {
            0
        },
        remaining_allowance,
        remaining_allowance_tax: base - tax_less(remaining_allowance),
        rehabilitation: crate::general_business_credit::RehabilitationAllowance {
            completed,
            threshold,
            income: rental.modified_agi,
            excess,
            limit,
            used: rental.special_allowance,
            remaining,
            tax: base - tax_less(remaining),
        },
    }
}
