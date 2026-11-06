// form_4835 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn collect_farm_rental(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    v: &crate::FederalFarmRental,
    direct_participation: &alloc::sync::Arc<crate::direct_participation::DirectParticipationView>,
) -> Result<(), CollectFailure> {
    let Collect { d, form_context, source_claims, selected_pair, home, .. } = *cx;
    let Source {
        e,
        context,
        #[cfg(test)]
        source,
        ..
    } = *src;
    t.withholding_1099 +=
        v.farm.insurance.withholding() + v.farm.ccc.as_ref().map_or(0, |ccc| ccc.withholding);
    crate::farm_rental::report_with_claims(
        d,
        e,
        v,
        r,
        source_claims,
        Some(form_context.macrs().as_ref()),
    );
    if let Some(ccc) = &v.farm.ccc {
        crate::farm::require_same_ccc_method(d, e.owner.owned(), ccc, r);
    }
    expense_elections(
        r,
        &v.expenses,
        context,
        #[cfg(test)]
        source,
        &alloc::format!("business_{}", e.id),
    );
    crate::require_localized!(
        r,
        crate::depreciation::consistent_elections(d),
        &context,
        &source,
        "ISSUE_USE_ONE_CONSISTENT_BONUS_GDS_METHOD_ELECTION_FOR_EACH_RECOVERY_CLAS_5ABD1EF0",
        "Use one consistent bonus/GDS method election for each recovery class across all businesses and nonbusiness rentals on this return.",
    );
    // The rental's property is itemized on Form 4562 like a business's.
    let form_4562 = crate::depreciation::assets_need_form_4562(&v.assets);
    for (index, asset) in v.assets.iter().enumerate() {
        let actual_source = d
            .income
            .iter()
            .find_map(|original| match &original.income {
                FederalIncome::FarmRental { data } if original.id == e.id => data.assets.get(index),
                _ => None,
            })
            .filter(|original| original.nonmacrs())
            .unwrap_or(asset);
        // The farm rules for an actual-expense vehicle are the
        // Schedule C ones: farm vehicles need separate rules.
        crate::require_localized!(
            r,
            !asset.is_vehicle(),
            &context.item("business_assets", index).manual(),
            "Actual-expense vehicle",
            "ISSUE_CONFIRM_AN_OWNED_5_YEAR_VEHICLE_ON_SCHEDULE_C_WITH_ACTUAL_EXPENSES_93473DC7",
            "Confirm an owned 5-year vehicle on Schedule C or Schedule F with actual expenses and its actual vehicle design. New vehicles need more than 50% business use for section 179; a heavy passenger SUV without a design exception has a $32,000 election limit. Continuing passenger automobiles need a supported original schedule and annual cap history; unsupported changed-use, carryforward, lease and employer-provided cases need their own rules (Form 4562 Part V).",
        );
        crate::require_localized!(
            r,
            actual_source.qualifies(),
            &context,
            &source,
            actual_source.nonmacrs_issue().unwrap_or(
                "ISSUE_CONFIRM_QUALIFYING_PURCHASED_EQUIPMENT_WITH_THE_ENTERED_SERVICE_DAT_EDE7B97E"
            ),
            "Confirm qualifying purchased equipment with the entered service date, first-year election or complete continuing history with identical regular/AMT basis or the separately recorded required AMT schedule. A first-use 2026 asset acquired before September 28, 2017 receives no bonus; ordinary 200% GDS for a 3-, 5-, 7-, or 10-year class uses the same basis, recovery period, and convention with 150% for AMT. Continuing earlier purchases require post-1998 service and 150%/straight-line GDS or a supported separate 200%/150% history. Unrecovered basis after the recovery period or an unsupported history needs its separate worksheet.",
        );
        asset_disposition_rows_with_claims(
            d,
            t,
            r,
            #[cfg(test)]
            source,
            crate::depreciation::mid_quarter_with_macrs(d, Some(form_context.macrs().as_ref())),
            e.id,
            index,
            &v.assets,
            v.vehicles.len(),
            source_claims,
            Some(form_context.macrs().as_ref()),
        );
        form_4562_asset_lines_with_claims(
            crate::related_forms::RelatedFormContext::bound(d, source_claims, direct_participation)
                .with_macrs(&form_context.macrs()),
            r,
            e.id,
            index,
            asset,
            form_4562,
            selected_pair,
        );
    }
    if form_4562 {
        form_4562_totals_with_section179_with_claims(
            d,
            r,
            e.id,
            &v.assets,
            None,
            home,
            source_claims,
        )?;
    }
    Ok(())
}
