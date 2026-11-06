// form_461 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
/// Section 461(l) operand and excess business loss over the threshold, from the business net after the
/// self-employment, retirement, health and Form 2555 adjustments.
pub(super) fn excess_business_loss_amount(
    adjusted_business: i64,
    rental: &crate::extended_rules::RentalResult,
    property_business_income: i64,
    j: bool,
) -> (i64, i64) {
    let operand = -adjusted_business
        - rental.qbi
        - rental.ordinary_pass_through_business()
        - property_business_income;
    let threshold = if j {
        crate::year_constants::EXCESS_BUSINESS_LOSS_THRESHOLD_JOINT_DOLLARS
    } else {
        crate::year_constants::EXCESS_BUSINESS_LOSS_THRESHOLD_OTHER_DOLLARS
    };
    (operand, positive(operand - threshold))
}

pub(super) struct BusinessLossClassificationIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) property_character: &'a Option<crate::section1231::Result1231>,
    pub(super) amt_property_character: &'a Option<crate::section1231::Result1231>,
    pub(super) direct_property_sources: &'a [(i64, crate::community::PropertyCents)],
    pub(super) health: &'a crate::health::HealthResult,
    pub(super) k1_ordinary: [i64; 2],
    pub(super) before_rental: i64,
    pub(super) portfolio_capital: i64,
    pub(super) capital: i64,
    pub(super) amt_capital: i64,
    pub(super) property_ordinary: i64,
    pub(super) amt_property_ordinary: i64,
    pub(super) k1_ordinary_passive_adjustment: i64,
    pub(super) amt_k1_ordinary_passive_adjustment: i64,
    pub(super) short: i64,
    pub(super) long: i64,
    pub(super) amt_short: i64,
    pub(super) amt_long: i64,
    pub(super) personal_rent_expenses: i64,
    pub(super) property_net: i64,
    pub(super) business: i64,
    pub(super) half_se: i64,
    pub(super) sep_deduction: i64,
    pub(super) business_excluded: i64,
    pub(super) direct_property_ordinary: i64,
}

pub(super) struct BusinessLossClassificationOut {
    pub(super) before_rental: i64,
    pub(super) k1_business_ordinary: i64,
    pub(super) amt_k1_business_ordinary: i64,
    pub(super) property_business_capital: i64,
    pub(super) property_business_income: i64,
    pub(super) amt_property_business_income: i64,
    pub(super) personal_rent_expenses: i64,
    pub(super) property_net: i64,
    pub(super) adjusted_business: i64,
    pub(super) excess_business_loss_operand: i64,
    pub(super) excess_business_loss: i64,
    pub(super) before_social: i64,
    pub(super) property_loss_deferred: Result<i64, (&'static str, &'static str)>,
}

pub(super) fn business_loss_classification_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    i: BusinessLossClassificationIn<'_>,
) -> BusinessLossClassificationOut {
    let &Pass { d, j, allocated_entries, .. } = pass;
    let BusinessLossClassificationIn {
        rental,
        property_character,
        amt_property_character,
        direct_property_sources,
        health,
        k1_ordinary,
        before_rental,
        portfolio_capital,
        capital,
        amt_capital,
        property_ordinary,
        amt_property_ordinary,
        k1_ordinary_passive_adjustment,
        amt_k1_ordinary_passive_adjustment,
        short,
        long,
        amt_short,
        amt_long,
        personal_rent_expenses,
        property_net,
        business,
        half_se,
        sep_deduction,
        business_excluded,
        direct_property_ordinary,
    } = i;
    // The part of a code R loss the passive rules hold back is
    // in `loan_gain` from here on, but not in the total income built above.
    let before_rental = before_rental - portfolio_capital
        + capital
        + property_ordinary
        + k1_ordinary_passive_adjustment;
    // Separately stated ordinary K-1 business dispositions are already in
    // AGI but also belong in Form461 after their passive cuts. Shareholder
    // loan gains remain investment amounts outside this business total.
    let k1_business_ordinary = k1_ordinary[0] + k1_ordinary_passive_adjustment;
    let amt_k1_business_ordinary = k1_ordinary[1] + amt_k1_ordinary_passive_adjustment;
    let property_business_capital = property_character
        .as_ref()
        .map_or(0, |v| phase::cap(v.long_term, [positive(short + long)]));
    let property_business_income =
        property_ordinary + k1_business_ordinary + property_business_capital;
    let amt_property_business_income = amt_property_ordinary
        + amt_k1_business_ordinary
        + amt_property_character
            .as_ref()
            .map_or(0, |v| phase::cap(v.long_term, [positive(amt_short + amt_long)]));
    if property_character.is_some() {
        r.review_notes.push(crate::federal_model::ReviewNote::localized(
            #[cfg(test)] alloc::format!("Property return-income stage: regular ordinary ${property_ordinary}; regular capital income after all netting ${capital}; AMT ordinary ${amt_property_ordinary}; AMT capital income ${amt_capital}. These amounts feed the income calculation; final totals require complete source records and all applicable return checks."),
            crate::Message::new("NOTE_PROPERTY_RETURN_INCOME_STAGE", alloc::vec![
                crate::MessageArg::Currency { currency_code: "USD".into(), scale: 0, minor_units: property_ordinary },
                crate::MessageArg::Currency { currency_code: "USD".into(), scale: 0, minor_units: capital },
                crate::MessageArg::Currency { currency_code: "USD".into(), scale: 0, minor_units: amt_property_ordinary },
                crate::MessageArg::Currency { currency_code: "USD".into(), scale: 0, minor_units: amt_capital },
            ]),
        ));
    }

    let personal_rent_expenses = personal_rent_expenses - rental.personal_expense_adjustment;
    let property_net = property_net + rental.personal_expense_adjustment;
    line(
        r,
        "nonbusiness_personal_rent_expenses",
        personal_rent_expenses,
        &["income.nonbusiness_property", "rental_suspended_loss"],
        "https://www.irs.gov/publications/p525",
        msg!("EXPLAIN_NONBUSINESS_PERSONAL_RENT_EXPENSES"),
    );
    // Section 461(l) follows the at-risk and passive-activity limits. Wages
    // and portfolio capital gains do not absorb an excess business loss.
    let adjusted_business =
        business - half_se - sep_deduction - health.deduction - business_excluded;
    let (excess_business_loss_operand, excess_business_loss) =
        excess_business_loss_amount(adjusted_business, rental, property_business_income, j);
    if rental.inputs_complete && !rental.partners.is_empty() {
        r.review_notes.push(ReviewNote::localized(#[cfg(test)] alloc::format!(
            "Partnership-inclusive regular excess-business-loss worksheet: ordinary pass-through business net ${}; excess business loss ${excess_business_loss}. Uses amounts after activity limits; wages do not offset these business losses. Additional partnership items require their own source records and review.",
            rental.ordinary_pass_through_business(),
        ), Message::new("NOTE_PARTNERSHIP_REGULAR_EXCESS_BUSINESS_LOSS", alloc::vec![
            MessageArg::usd_dollars(rental.ordinary_pass_through_business()), MessageArg::usd_dollars(excess_business_loss),
        ])));
    }
    let property_loss_deferred = property_character.as_ref().map_or(Ok(0), |character| {
        crate::section1231::property_excess_allocation(
            d,
            rental,
            character,
            excess_business_loss,
            false,
            adjusted_business + direct_property_ordinary,
            direct_property_sources,
        )
    });

    if allocated_entries.iter().any(|e| matches!(e.income, FederalIncome::Scorp { .. }))
        && r.issues.is_empty()
    {
        r.review_notes.push(ReviewNote::localized(
            #[cfg(test)] alloc::format!("K-1 excess-business-loss worksheet: ordinary K-1 net after activity limits ${}; return excess business loss ${excess_business_loss}. Employee wages do not absorb business losses; QBI qualification does not determine this business-income amount.", rental.scorp_business),
            Message::new("NOTE_K1_EXCESS_BUSINESS_LOSS_WORKSHEET", alloc::vec![
                MessageArg::usd_dollars(rental.scorp_business), MessageArg::usd_dollars(excess_business_loss),
            ]),
        ));
    }
    // These source entries describe investment stock, not a securities trade
    // or business. Form 461 Part II removes nonbusiness losses. Section 1244
    // treats its ordinary investment loss as a business loss only for the
    // section 172 NOL calculation; that does not expand section 461(l).
    let before_social =
        before_rental + rental.personal_expense_adjustment + rental.income + excess_business_loss;
    line(
        r,
        "excess_business_loss",
        excess_business_loss,
        &["income.business", "rental_income"],
        ANNUAL,
        msg!("EXPLAIN_EXCESS_BUSINESS_LOSS"),
    );
    BusinessLossClassificationOut {
        before_rental,
        k1_business_ordinary,
        amt_k1_business_ordinary,
        property_business_capital,
        property_business_income,
        amt_property_business_income,
        personal_rent_expenses,
        property_net,
        adjusted_business,
        excess_business_loss_operand,
        excess_business_loss,
        before_social,
        property_loss_deferred,
    }
}
