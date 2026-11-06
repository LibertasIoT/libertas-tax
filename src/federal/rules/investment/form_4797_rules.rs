// form_4797 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// One business asset's Form 4797 rows and the Form 8949 row of the personal-use
// part of its sale.

#[inline(never)]
#[allow(clippy::too_many_arguments)]
pub(super) fn asset_disposition_rows_with_claims(
    d: &FederalDraft,
    t: &mut Totals,
    r: &mut FederalResult,
    #[cfg(test)] source: &str,
    mid_quarter: bool,
    id: i64,
    index: usize,
    assets: &[crate::BusinessAsset],
    mileage: usize,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
    macrs: Option<&crate::macrs_source::MacrsElectionView>,
) {
    let sale = match crate::cooperative_basis::sale_with_claims(
        d,
        id,
        &assets[index],
        mid_quarter,
        false,
        source_claims,
        macrs,
    ) {
        Ok(sale) => sale,
        Err(issue) => {
            crate::section179_return::Issue::Prepared(issue).report(r);
            return;
        }
    };
    let partial = match crate::cooperative_basis::partial_with_claims(
        d,
        id,
        &assets[index],
        mid_quarter,
        false,
        source_claims,
        macrs,
    ) {
        Ok(partial) => partial,
        Err(issue) => {
            crate::section179_return::Issue::Prepared(issue).report(r);
            return;
        }
    };
    crate::asset_sale::report_business_asset_source(
        r,
        #[cfg(test)]
        source,
        &crate::IssueContext::income_id(id),
        &alloc::format!("business_{id}_asset_{index}"),
        index,
        assets,
        mileage,
        sale,
        partial,
        "income.business.assets",
        crate::nonmacrs::rental_recovery_source(d, id, true).as_ref(),
    );
    t.personal_part(r, id, crate::BusinessAsset::display_arg(assets, index, mileage), sale);
}
// The sale of a business as a whole (Form 8594), beside each asset's own row:
// the inventory is Schedule C line 1 (already in the business's receipts), the
// self-created goodwill and going-concern value one long-term Form 8949 row (a
// capital asset with no basis, not section 1231 and not qualified business
// income, and outside net investment income of a business you materially
// participate in), and the covenant not to compete Schedule 1 line 8z.
#[inline(never)]
pub(super) fn business_sale_rows(
    t: &mut Totals,
    r: &mut FederalResult,
    entry: &FederalIncomeEntry,
    sale: &crate::BusinessSale,
) {
    let source = "https://www.irs.gov/instructions/i8594";
    // Rows without a form line of their own: the amounts are already in the
    // business's receipts and in Schedule 1 line 8z's total.
    for (suffix, amount, explanation) in [
        ("sale_inventory", sale.inventory, msg!("EXPLAIN_BUSINESS_N_SALE_INVENTORY")),
        ("sale_covenant", sale.covenant, msg!("EXPLAIN_BUSINESS_N_SALE_COVENANT")),
    ] {
        if amount != 0 {
            push_line(
                r,
                None,
                &alloc::format!("business_{}_{suffix}", entry.id),
                amount,
                &["income.business.more.sale"],
                source,
                explanation,
            );
        }
    }
    t.other_8z += sale.covenant;
    if sale.goodwill != 0 {
        let gain = sale.goodwill;
        t.property_row(
            r,
            entry.id,
            crate::interview::income_display_arg(entry),
            true,
            [dollars(gain), 0, 0],
            false,
        );
        // Form 8960 line 5b: gain on property held in a business you
        // materially participate in.
        t.k1_nii_excluded -= dollars(gain) * 100;
    }
}
// The sale, scrapping or condemnation of equipment rented out outside a
// business: depreciable property, but not property used in a trade or business
// (section 1231(b)(1)). Section 1245 still takes the depreciation allowed as
// ordinary income, Part III line 32 to line 13 (Schedule 1 line 4, not
// self-employment income); a gain above it, or a loss, is a capital gain or
// loss on Form 8949 (i4797 "Sale of Home Used for Business": an activity that
// does not rise to a trade or business is a capital gain or loss). Both are
// investment income. A loss on a sale to a related buyer is not deductible
// (section 267) and the whole gain of a controlled buyer's purchase is ordinary
// (section 1239).
#[inline(never)]
pub(super) fn personal_equipment_sale(
    t: &mut Totals,
    r: &mut FederalResult,
    entry: &FederalIncomeEntry,
    index: usize,
    assets: &[crate::BusinessAsset],
    mid_quarter: bool,
    invocation: crate::macrs_source::AdsInvocation<'_>,
) {
    let Some(sale) = assets[index].sale_in_context(mid_quarter, false, invocation) else {
        return;
    };
    let sale = &sale;
    let label = crate::BusinessAsset::display_arg(assets, index, 0);
    if sale.disallowed > 0 {
        note_once_scoped(
            r,
            Message::new("NOTE_SALE_TO_RELATIVE_LOSS_DISALLOWED", alloc::vec![label]),
            crate::interview::income_issue_context(entry),
        );
        return;
    }
    let ordinary = crate::asset_sale::equipment_ordinary(sale);
    let prefix = alloc::format!("personal_rental_{}_asset_{}_disposal", entry.id, index);
    if sale.gain > 0 {
        emit_lines(
            r,
            &mut (),
            [
                ("22", "sale_price", sale.realized, msg!("EXPLAIN_ASSET_SALE_N_6")),
                ("23", "cost", sale.basis, msg!("EXPLAIN_ASSET_SALE_N_7")),
                ("24", "depreciation", sale.depreciation, msg!("EXPLAIN_ASSET_SALE_N_8")),
                (
                    "25",
                    "adjusted_basis",
                    sale.basis - sale.depreciation,
                    msg!("EXPLAIN_ASSET_SALE_N_4"),
                ),
                ("26", "gain", sale.gain, msg!("EXPLAIN_ASSET_SALE_N_5")),
                ("27b", "1245_recapture", ordinary, msg!("EXPLAIN_N_N_RECAPTURE_2")),
            ],
            |(line, suffix, amount, explanation), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::Form4797, line)),
                    &alloc::format!("{prefix}_{suffix}"),
                    LineValue::MoneyCents(amount * 100),
                    &["income.nonbusiness_property.assets"],
                    "https://www.irs.gov/instructions/i4797",
                    explanation,
                ))
            },
        );
    }
    // The ordinary recapture joins Schedule 1 line 4 through the property
    // sources; the capital part is one Form 8949 row.
    t.property_row(
        r,
        entry.id,
        label,
        sale.long,
        [sale.realized, sale.basis - sale.depreciation, -ordinary],
        true,
    );
}

pub(super) struct RentalMagiPreflightIn {
    pub(super) regular_capital_amounts: crate::capital::CapitalAmounts,
    pub(super) capital_in: [i64; 2],
    pub(super) modify_income: bool,
    pub(super) before_rental: i64,
    pub(super) half_se: i64,
    pub(super) adoption_excluded: i64,
}

pub(super) struct RentalMagiPreflightOut {
    pub(super) rental_magi_base: i64,
    pub(super) home_route: Option<bool>,
    pub(super) home_use: crate::extended_rules::HomeUseContext,
}

pub(super) fn rental_magi_preflight_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    partner_pass: &PartnerPass,
    i: RentalMagiPreflightIn,
) -> RentalMagiPreflightOut {
    let &Pass { d, selected_regular, context, .. } = pass;
    let source_claims = context.source_claims;
    let force_itemized = context.force_itemized;
    let RentalMagiPreflightIn {
        regular_capital_amounts,
        capital_in,
        modify_income,
        before_rental,
        half_se,
        adoption_excluded,
    } = i;
    let property_magi_adjustment = match crate::section1231::rental_magi_adjustment_with_claims(
        d,
        regular_capital_amounts,
        capital_in,
        modify_income,
        source_claims,
        selected_regular,
    ) {
        Ok(value) => value,
        Err(error) => {
            let (key, _message) = error.detail;
            crate::issue_localized!(
                r,
                &error.context,
                "Property income for rental MAGI",
                key,
                _message,
            );
            0
        }
    };
    if property_magi_adjustment != 0 {
        r.review_notes.push(crate::federal_model::ReviewNote::localized(
            #[cfg(test)] alloc::format!("Rental-allowance property-income adjustment: ${property_magi_adjustment}. Replaces raw nonpassive property income with its regular-tax effect after section 1231 lookback and capital-loss coordination. The same MAGI projection applies to the AMT allowance."),
            crate::Message::new("NOTE_RENTAL_MAGI_PROPERTY_ADJUSTMENT", alloc::vec![crate::MessageArg::Currency { currency_code: "USD".into(), scale: 0, minor_units: property_magi_adjustment }]),
        ));
    }
    let rental_magi_base = partner_pass.specials.rental.as_ref().map_or(
        before_rental + half_se + adoption_excluded + property_magi_adjustment,
        |snapshot| snapshot.magi_input,
    );
    if crate::extended_rules::professional_election_rule(d).is_some() {
        crate::issue_localized!(
            r,
            &crate::IssueContext::Section(crate::TaxSection::People),
            "Professional rental election",
            "ISSUE_PROFESSIONAL_ELECTION_ALL_RENTALS",
            "With the single-activity election choose Professional for every rental real estate activity or for none.",
        );
    }
    let home_route = crate::extended_rules::HomeUseContext::route(d, force_itemized);
    let home_use = partner_pass.home_agi.clone().unwrap_or_else(|| {
        crate::extended_rules::HomeUseContext::provisional(
            d,
            [rental_magi_base, rental_magi_base],
            home_route,
        )
    });
    RentalMagiPreflightOut { rental_magi_base, home_route, home_use }
}

pub(super) struct OrdinaryDispositionIncomeOut {
    pub(super) pure_loan_gain: i64,
    pub(super) pure_amt_loan_gain: i64,
    pub(super) k1_ordinary: [i64; 2],
    pub(super) loan_gain: i64,
    pub(super) amt_loan_gain: i64,
    pub(super) k1_investment_ordinary: [i64; 2],
    pub(super) stock_loss: i64,
}

pub(super) fn ordinary_disposition_income_phase(
    r: &mut FederalResult,
    t: &Totals,
) -> OrdinaryDispositionIncomeOut {
    let [pure_loan_gain, pure_amt_loan_gain] = t.scorp_loan_gain.map(dollars);
    // The K-1 ordinary gains (Form 4797 Part II line 11, Schedule 1
    // line 4) join the shareholder loan gains; they are the same kind of
    // ordinary gain for every total below, except where Form 4952 asks
    // whether the property is investment property.
    let k1_ordinary = t.k1_ordinary.map(dollars);
    let [loan_gain, amt_loan_gain] =
        [pure_loan_gain + k1_ordinary[0], pure_amt_loan_gain + k1_ordinary[1]];
    let k1_investment_ordinary = t.k1_investment_ordinary.map(dollars);
    emit_lines(
        r,
        &mut (),
        [
            (
                "scorp_ordinary_loan_gain",
                pure_loan_gain,
                msg!("EXPLAIN_SCORP_ORDINARY_LOAN_GAIN"),
                Some(FormLine::new(TaxForm::Form4797, "11")),
            ),
            (
                "scorp_amt_loan_gain_adjustment",
                pure_amt_loan_gain - pure_loan_gain,
                msg!("EXPLAIN_SCORP_AMT_LOAN_GAIN_ADJUSTMENT"),
                None,
            ),
        ],
        |(id, amount, explanation, form), _, _| {
            Some(calculation_row(
                form,
                id,
                LineValue::MoneyCents(amount * 100),
                &["income.scorp.loans", "income.scorp.amt.loans"],
                "https://www.irs.gov/instructions/i7203",
                explanation,
            ))
        },
    );
    let stock_loss = dollars(t.stock_loss);
    // Form 4797 Part II line 11 (2026 draft; line 10 before 2026): "Losses on
    // Section 1244 (Small Business) Stock".
    form_line(
        r,
        FormLine::new(TaxForm::Form4797, "11"),
        "section_1244_ordinary_loss",
        stock_loss,
        &["income.small_business_stock"],
        "https://www.irs.gov/publications/p550",
        msg!("EXPLAIN_SECTION_N_ORDINARY_LOSS"),
    );
    OrdinaryDispositionIncomeOut {
        pure_loan_gain,
        pure_amt_loan_gain,
        k1_ordinary,
        loan_gain,
        amt_loan_gain,
        k1_investment_ordinary,
        stock_loss,
    }
}

pub(super) struct OrdinaryPropertySourcesIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) amt_rental: &'a mut crate::extended_rules::RentalResult,
    pub(super) property_character: &'a Option<crate::section1231::Result1231>,
    pub(super) regular_issue_count: usize,
    pub(super) k1_ordinary: [i64; 2],
    pub(super) k1_sale_ordinary: i64,
    pub(super) loan_gain: i64,
    pub(super) amt_loan_gain: i64,
}

pub(super) struct OrdinaryPropertySourcesOut {
    pub(super) k1_ordinary_passive_adjustment: i64,
    pub(super) amt_k1_ordinary_passive_adjustment: i64,
    pub(super) loan_gain: i64,
    pub(super) amt_loan_gain: i64,
    pub(super) direct_property_ordinary: i64,
    pub(super) amt_direct_property_ordinary: i64,
    pub(super) property_ordinary: i64,
    pub(super) amt_property_ordinary: i64,
    pub(super) amt_property_character: Option<crate::section1231::Result1231>,
    pub(super) amt_activity_issues: Vec<crate::federal_model::FederalIssue>,
    pub(super) direct_property_sources: Vec<(i64, crate::community::PropertyCents)>,
    pub(super) amt_direct_property_sources: Vec<(i64, crate::community::PropertyCents)>,
    pub(super) rental_property_sources: Vec<(i64, crate::farm_land::LandGain)>,
    pub(super) injured_property_sources: Option<crate::section1231::PropertyOwnerAmounts>,
}

pub(super) fn ordinary_property_sources_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    i: OrdinaryPropertySourcesIn<'_>,
) -> core::ops::ControlFlow<(), OrdinaryPropertySourcesOut> {
    let &Pass { d, context, selected_regular, selected_alternative, office_context, .. } = pass;
    let source_claims = context.source_claims;
    let OrdinaryPropertySourcesIn {
        rental,
        amt_rental,
        property_character,
        regular_issue_count,
        k1_ordinary,
        k1_sale_ordinary,
        loan_gain,
        amt_loan_gain,
    } = i;
    macro_rules! selected179 {
        ($value:expr) => {
            match $value {
                Ok(value) => value,
                Err(issue) => {
                    crate::section179_return::Issue::from(issue).report(r);
                    r.state = ResultState::Unsupported;
                    return core::ops::ControlFlow::Break(());
                }
            }
        };
    }
    crate::k1_totals::gain_not_passive(d, rental, r);
    // The part of a code R loss the passive rules hold back does
    // not reach Form 4797 this year.
    let k1_ordinary_passive_adjustment =
        dollars(t.k1_ordinary[0] + rental.k1_cuts.ordinary.get()) - k1_ordinary[0];
    let amt_k1_ordinary_passive_adjustment =
        dollars(t.k1_ordinary[1] + amt_rental.k1_cuts.ordinary.get()) - k1_ordinary[1];
    let loan_gain = loan_gain + k1_ordinary_passive_adjustment;
    let amt_loan_gain = amt_loan_gain + amt_k1_ordinary_passive_adjustment;
    crate::k1_sales::report(d, amt_rental, true, r);
    crate::k1_sales::report_recapture(r);
    form_line(
        r,
        FormLine::new(TaxForm::Form4797, "11"),
        "k1_ordinary_gains",
        k1_ordinary[0] + k1_ordinary_passive_adjustment - k1_sale_ordinary,
        &["income.partnership", "income.scorp"],
        "https://www.irs.gov/instructions/i4797",
        msg!("EXPLAIN_K1_ORDINARY_GAINS"),
    );
    crate::community::SourceProjection::bind_business(amt_rental, &t.business_sources, true);
    let amt_property_character =
        crate::section1231::report_with_claims(d, amt_rental, true, r, source_claims);
    // Keep AMT-only input issues from suppressing independent regular-tax
    // intermediate output; merge them at the AMT reporting stage below.
    let amt_activity_issues = r.issues.split_off(regular_issue_count);
    let direct_property_sources = match crate::asset_sale::allowed_direct_sources_with_claims(
        d,
        rental,
        false,
        source_claims,
    ) {
        Ok(value) => value,
        Err(issue) => {
            issue.report(r);
            return core::ops::ControlFlow::Break(());
        }
    };
    let direct_property_ordinary_cents =
        direct_property_sources.iter().map(|(_, value)| value.ordinary).sum::<i64>();
    let direct_property_ordinary = dollars(direct_property_ordinary_cents);
    let amt_direct_property_sources = match crate::asset_sale::allowed_direct_sources_with_claims(
        d,
        amt_rental,
        true,
        source_claims,
    ) {
        Ok(value) => value,
        Err(issue) => {
            issue.report(r);
            return core::ops::ControlFlow::Break(());
        }
    };
    let amt_direct_property_ordinary_cents =
        amt_direct_property_sources.iter().map(|(_, value)| value.ordinary).sum::<i64>();
    let amt_direct_property_ordinary = dollars(amt_direct_property_ordinary_cents);
    // A sold rental's section 1250 additional depreciation (the section 50(c)
    // reduction of a rehabilitation credit counts as depreciation) is ordinary.
    let rental_property_sources =
        match crate::asset_sale::allowed_rental_sources(context.forms(d), rental, false) {
            Ok(value) => value,
            Err(issue) => {
                issue.report(r);
                return core::ops::ControlFlow::Break(());
            }
        };
    let rental_property_ordinary =
        rental_property_sources.iter().map(|(_, v)| v.ordinary).sum::<i64>();
    let mut injured_property_sources = crate::section1231::direct_owner_amounts(
        d,
        rental,
        property_character.as_ref(),
        &direct_property_sources,
        &rental_property_sources,
    );
    let amt_rental_property_ordinary =
        match crate::asset_sale::allowed_rental_sources(context.forms(d), amt_rental, true) {
            Ok(value) => value,
            Err(issue) => {
                issue.report(r);
                return core::ops::ControlFlow::Break(());
            }
        }
        .iter()
        .map(|(_, v)| v.ordinary)
        .sum::<i64>();
    // The business casualty component leaves Form 8829 separately from the
    // office expense deduction. It reaches Form 4797, never Schedule C/F/SE.
    let casualty_overflow = || crate::Section179PreparedIssue::AggregateAmountOverflow;
    let office_casualty_owners =
        selected179!(crate::home_office::casualty_by_owner_dollars_with_claims(
            d,
            selected_regular,
            office_context,
            source_claims
        ));
    let office_casualty = office_casualty_owners.iter().sum::<i64>();
    let amt_office_casualty =
        dollars(selected179!(crate::home_office::casualty_ordinary_with_claims(
            d,
            selected_alternative,
            office_context,
            source_claims
        )));
    if let Some(property) = &mut injured_property_sources {
        for (ordinary, casualty) in property.ordinary.iter_mut().zip(office_casualty_owners) {
            *ordinary += selected179!(
                crate::extended_rules::annual_dollars_to_working_cents(casualty)
                    .map_err(|_| casualty_overflow())
            );
        }
    }
    if office_casualty != 0 {
        form_line(
            r,
            FormLine::new(TaxForm::Form4797, "14"),
            "home_office_casualty_ordinary",
            office_casualty,
            &["home_sites"],
            "https://www.irs.gov/instructions/i4684",
            msg!("EXPLAIN_CANONICAL_HOME_WORKSHEET"),
        );
    }
    let rental_casualty_cents = rental.prior_home_casualty_by_owner;
    let rental_casualty = dollars(selected179!(
        crate::kernel::ratio::checked_sum(rental_casualty_cents).ok_or_else(casualty_overflow)
    ));
    let rental_casualty_owners = selected179!(
        crate::kernel::ratio::project_rounded_pair(rental_casualty, rental_casualty_cents, 0)
            .ok_or_else(casualty_overflow)
    );
    let amt_rental_casualty = dollars(selected179!(
        crate::kernel::ratio::checked_sum(amt_rental.prior_home_casualty_by_owner)
            .ok_or_else(casualty_overflow)
    ));
    if let Some(property) = &mut injured_property_sources {
        for (ordinary, casualty) in property.ordinary.iter_mut().zip(rental_casualty_owners) {
            *ordinary = selected179!(
                ordinary
                    .checked_add(selected179!(
                        crate::extended_rules::annual_dollars_to_working_cents(casualty)
                            .map_err(|_| casualty_overflow())
                    ))
                    .ok_or_else(casualty_overflow)
            );
        }
    }
    if rental_casualty != 0 {
        form_line(
            r,
            FormLine::new(TaxForm::Form4684, "27"),
            "rental_home_prior_casualty",
            -rental_casualty,
            &["income.rental.home_casualty_carryover"],
            "https://www.irs.gov/instructions/i4684",
            msg!("EXPLAIN_RENTAL_HOME_CASUALTY_ALLOWED"),
        );
        form_line(
            r,
            FormLine::new(TaxForm::Form4797, "14"),
            "rental_home_prior_casualty_ordinary",
            rental_casualty,
            &["income.rental.home_casualty_carryover"],
            "https://www.irs.gov/instructions/i4684",
            msg!("EXPLAIN_RENTAL_HOME_CASUALTY_ALLOWED"),
        );
    }
    let lift_property = |amount| {
        crate::extended_rules::annual_dollars_to_working_cents(amount)
            .map_err(|_| casualty_overflow())
    };
    let property_ordinary = dollars(selected179!(
        crate::kernel::ratio::checked_sum([
            direct_property_ordinary_cents,
            selected179!(lift_property(rental_property_ordinary)),
            selected179!(property_character.as_ref().map_or(Ok(0), |value| {
                value.in_cents().map(|value| value.ordinary).map_err(|_| casualty_overflow())
            })),
            selected179!(lift_property(office_casualty)),
            selected179!(lift_property(rental_casualty)),
        ])
        .ok_or_else(casualty_overflow)
    ));
    let amt_property_ordinary = dollars(selected179!(
        crate::kernel::ratio::checked_sum([
            amt_direct_property_ordinary_cents,
            selected179!(lift_property(amt_rental_property_ordinary)),
            selected179!(amt_property_character.as_ref().map_or(Ok(0), |value| {
                value.in_cents().map(|value| value.ordinary).map_err(|_| casualty_overflow())
            })),
            selected179!(lift_property(amt_office_casualty)),
            selected179!(lift_property(amt_rental_casualty)),
        ])
        .ok_or_else(casualty_overflow)
    ));
    core::ops::ControlFlow::Continue(OrdinaryPropertySourcesOut {
        k1_ordinary_passive_adjustment,
        amt_k1_ordinary_passive_adjustment,
        loan_gain,
        amt_loan_gain,
        direct_property_ordinary,
        amt_direct_property_ordinary,
        property_ordinary,
        amt_property_ordinary,
        amt_property_character,
        amt_activity_issues,
        direct_property_sources,
        amt_direct_property_sources,
        rental_property_sources,
        injured_property_sources,
    })
}
