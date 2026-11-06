// schedule_j items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct ScheduleJAveragingIn<'a> {
    pub(super) business_half_se: &'a alloc::collections::BTreeMap<i64, i64>,
    pub(super) health: &'a crate::health::HealthResult,
    pub(super) sep: &'a crate::sep::SepResult,
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) property_character: Option<&'a crate::section1231::Result1231>,
    pub(super) foreign: &'a crate::extended_rules::ForeignResult,
    pub(super) activities: &'a [crate::business_rules::QbiActivity],
    pub(super) qbi: &'a crate::business_rules::QbiResult,
    pub(super) farm_qbi_indices: &'a [usize],
    pub(super) current_preference: crate::income_tax::Preference,
    pub(super) use_itemized: bool,
    pub(super) parent_rate_required: bool,
    pub(super) regular_tax: i64,
    pub(super) qbi_ded: i64,
    pub(super) prior_business: i64,
    pub(super) prior_reit: i64,
    pub(super) taxable: i64,
    pub(super) excluded: i64,
    pub(super) excess_business_loss: i64,
    pub(super) msa_deduction: i64,
}

pub(super) struct ScheduleJAveragingOut<'a> {
    pub(super) regular_tax: i64,
    pub(super) averaged_tax: crate::farm_averaging::Calculation<'a>,
}

pub(super) fn schedule_j_averaging_phase<'a>(
    pass: &Pass<'a>,
    r: &mut FederalResult,
    t: &Totals,
    input: ScheduleJAveragingIn<'_>,
) -> core::ops::ControlFlow<(), ScheduleJAveragingOut<'a>> {
    let &Pass {
        d,
        allocated_entries,
        office_context,
        s,
        ded,
        screen,
        selected_regular,
        context,
        ..
    } = pass;
    let source_claims = context.source_claims;
    let ScheduleJAveragingIn {
        business_half_se,
        health,
        sep,
        rental,
        property_character,
        foreign,
        activities,
        qbi,
        farm_qbi_indices,
        current_preference,
        use_itemized,
        parent_rate_required,
        mut regular_tax,
        qbi_ded,
        prior_business,
        prior_reit,
        taxable,
        excluded,
        excess_business_loss,
        msa_deduction,
    } = input;
    let farms: Vec<_> = allocated_entries
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Business { data }
                if matches!(data.activity, crate::BusinessActivity::Farming { .. }) =>
            {
                Some((e.id, e.owner, data))
            }
            _ => None,
        })
        .collect();
    let mut farm_net = 0;
    for (id, _, b) in &farms {
        farm_net += dollars(
            match farm_receipts_with_claims(d, *id, b, source_claims) {
                Ok(value) => value,
                Err(issue) => {
                    crate::section179_return::Issue::from(issue).report(r);
                    r.state = ResultState::Unsupported;
                    return core::ops::ControlFlow::Break(());
                }
            } - match crate::section179_return::expenses_with_claims(
                d,
                *id,
                b,
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
        ) - business_half_se.get(id).copied().unwrap_or(0)
            - health.by_business.get(id).copied().unwrap_or(0)
            - sep.by_business.get(id).copied().unwrap_or(0);
    }
    let farm_qbi = if qbi_ded == 0 {
        Some(0)
    } else if activities.len() == farms.len()
        && t.reit_dividends == 0
        && prior_business == 0
        && prior_reit == 0
    {
        Some(qbi_ded)
    } else {
        qbi.activity_deductions
            .as_ref()
            .map(|components| farm_qbi_indices.iter().map(|&i| components[i]).sum::<i64>())
    };
    if screen.farm_averaging.is_some()
        && let Some(amount) = farm_qbi
    {
        line(
            r,
            "farm_averaging_qbi_deduction",
            amount,
            &["income.business", "screening.qbi_history"],
            "https://www.irs.gov/instructions/i8995a",
            msg!("EXPLAIN_FARM_AVERAGING_QBI_DEDUCTION"),
        );
    }
    let averaged_tax = crate::farm_averaging::calculate(
        screen.farm_averaging.as_ref(),
        crate::farm_averaging::Current {
            status: s.filing_status(),
            taxable,
            regular_tax,
            preference: current_preference,
            eligible: farm_net - farm_qbi.unwrap_or(0),
            excluded,
            foreign_election: foreign.election(),
            shared_history_required: farms
                .iter()
                .any(|(_, owner, _)| *owner == crate::DocumentOwner::Yours)
                && farms.iter().any(|(_, owner, _)| *owner == crate::DocumentOwner::Spouses),
            scope_ready: !farms.is_empty()
                && farm_qbi.is_some()
                // Land-sale gains are excluded from Schedule J. Capital-only
                // dispositions add no QBI to allocate away from operating farm
                // income; nonzero ordinary land QBI still needs attribution.
                && match crate::asset_sale::allowed_direct_sources_with_claims(d, rental, false, source_claims) {
                    Ok(value) => value,
                    Err(issue) => {
                        issue.report(r);
                        return core::ops::ControlFlow::Break(());
                    }
                }.iter().all(|(id, _)| {
                    crate::farm_land::qbi_cents_with_claims(d, rental, property_character, *id, source_claims) == Ok(0)
                })
                // Personal deductions do not reduce farm income; income taxes
                // still need a source-supported farm allocation.
                && (!use_itemized || ded.income_tax_paid() == 0)
                && !screen.nol.present()
                && estate_final_nol(d).is_none()
                && excess_business_loss == 0
                && msa_deduction == 0
                && !parent_rate_required,
        },
        r,
    );
    regular_tax = averaged_tax.tax;
    let regular_tax_source = if parent_rate_required {
        "https://www.irs.gov/pub/irs-dft/i8615--dft.pdf"
    } else if averaged_tax.comparison.is_some() {
        "https://www.irs.gov/instructions/i1040sj"
    } else {
        TAX_TABLE
    };
    line(
        r,
        "regular_income_tax",
        regular_tax,
        &["taxable_income", "capital_gain", "foreign_earned_exclusion"],
        regular_tax_source,
        msg!("EXPLAIN_REGULAR_INCOME_TAX"),
    );
    core::ops::ControlFlow::Continue(ScheduleJAveragingOut { regular_tax, averaged_tax })
}
