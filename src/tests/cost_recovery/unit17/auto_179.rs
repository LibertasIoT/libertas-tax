use super::*;
use crate::tests::cross::interview::{
    accepted, document, reopened_businesses, save_topic, stored_business,
};
use crate::tests::support::builders::mixed_auto_179;

#[test]
fn mixed_auto_carry_replaces_macrs_inside_the_original_annual_cap() {
    // Derived control using Rev.Proc.2025-16 Table2:12200 in2025,
    //19600 in2026. A fixed19600 MACRS expense would incorrectly release0.
    let draft = mixed_auto_179(19_600, SourceCents::Reported { amount: 500_000 });
    let result = supported(&draft);
    assert_eq!(value(&result, "business_2_asset_0_section179"), 5_000);
    assert_eq!(value(&result, "business_2_asset_0_section179_carryforward"), 0);
    assert_eq!(value(&result, "business_2_asset_0_macrs"), 14_600);
    assert_eq!(value(&result, "business_2_depreciation_total"), 19_600);
    assert_eq!(value(&result, "business_2_asset_0_remaining_basis"), 68_200);
    // 2026-10-20: Form 4562 line 26(i) excludes prior-year elections;
    // the released carry is reported on line 10, not elected again.
    assert_eq!(value(&result, "business_2_asset_0_listed_section179"), 0);
}

#[test]
fn mixed_auto_negative_capacity_does_not_release_a_phantom_deduction() {
    let draft = mixed_auto_179(19_599, SourceCents::Reported { amount: 500_000 });
    let selected =
        crate::section179_return::prepare(&draft, Section179IncomeSystem::Regular).unwrap();
    assert_eq!(selected.allocation.allowed(2), Ok(0));
    assert_eq!(selected.allocation.operating(2), Ok(1_960_000));
    let result = supported(&draft);
    assert_eq!(value(&result, "business_2_asset_0_section179_carryforward"), 5_000);
    assert_eq!(value(&result, "business_2_asset_0_macrs"), 19_600);
}

#[test]
fn mixed_auto_amt_uses_its_own_actual_annual_179_and_never_an_aggregate_guess() {
    let mut draft = mixed_auto_179(100_000, SourceCents::Reported { amount: 300_000 });
    let regular =
        crate::section179_return::prepare(&draft, Section179IncomeSystem::Regular).unwrap();
    let alternative =
        crate::section179_return::prepare(&draft, Section179IncomeSystem::Alternative).unwrap();
    assert_eq!(regular.allocation.allowed(2), Ok(500_000));
    assert_eq!(alternative.allocation.allowed(2), Ok(700_000));
    assert_eq!(alternative.allocation.asset_recovery(2, 3).unwrap().allowance.get(), 1_260_000);
    let before = supported(&draft);
    let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
    data.assets[0].history_details.as_mut().unwrap().years[0].amt_section179_allowed =
        SourceCents::Unavailable;
    data.assets[0].amt_section179_income_carry = Some(700_000);
    assert!(
        crate::section179_return::prepare(&draft, Section179IncomeSystem::Alternative).is_err()
    );
    assert_eq!(estimate(&draft).state, ResultState::Unsupported);
    assert_eq!(
        estimate(
            &restart(mixed_auto_179(100_000, SourceCents::Reported { amount: 300_000 })).draft
        ),
        before
    );
}

#[test]
fn mixed_auto_disposal_restores_unused_179_to_basis_once() {
    let mut draft = mixed_auto_179(100_000, SourceCents::Reported { amount: 500_000 });
    let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
    data.assets[0].disposal =
        AssetDisposal::Disposed { date: 20260615, how: DisposalKind::Scrapped };
    let asset = &data.assets[0];
    let recovery = crate::auto_recovery::recovery(asset, 2026, 0).unwrap();
    //90k table basis ×32% ×half year =14400, below19600 annualcap.
    assert_eq!(recovery.allowance.get(), 1_440_000);
    assert_eq!(recovery.closing_section179_carry.get(), 0);
    assert_eq!(recovery.closing_basis.get(), 7_340_000);
    assert_eq!(asset.available179(), 0);
    assert_eq!(asset.sale_for(false, false).unwrap().gain, -73_400);
    let result = supported(&draft);
    assert_eq!(value(&result, "business_2_asset_0_remaining_basis"), 73_400);
}

#[test]
fn post_recovery_disposal_has_no_unrecovered_basis_deduction() {
    let mut asset = ordinary_original_auto(HistoricalAutoKind::Car);
    asset.disposal = AssetDisposal::Disposed { date: 20260615, how: DisposalKind::Scrapped };
    //26CFR1.280F-2T(c)(4), Example3: no postperiod deduction on disposal.
    let recovery = crate::auto_recovery::recovery(&asset, 2026, 0).unwrap();
    assert_eq!(recovery.allowance.get(), 0);
    assert_eq!(recovery.closing_basis.get(), recovery.opening_basis.get());
}

#[test]
fn mixed_auto_actual_sources_survive_native_save_reopen_and_restart() {
    let draft = mixed_auto_179(19_600, SourceCents::Reported { amount: 500_000 });
    let expected = supported(&draft);
    // 2026-10-20: i4562 line 26(e) retains original depreciable basis,
    // without subtracting prior regular depreciation or the released carry.
    assert_eq!(value(&expected, "business_2_asset_0_listed_depreciation_basis"), 90_000);
    assert_eq!(value(&expected, "business_2_asset_0_listed_section179"), 0);
    assert_eq!(value(&expected, "section179_prior_carryover_total"), 5_000);
    let mut session = restart(draft);
    let page = reopened_businesses(session.draft.clone()).remove(0);
    let (id, _) = stored_business(&session);
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(id, IncomeSource::Record { value: crate::IncomeRecord::Business(page) })],
    );
    accepted(&send(&mut session, request));
    // The first save of the applying topic records the visit (a revision).
    let mut visited = expected.clone();
    visited.revision = session.draft.revision;
    assert_eq!(supported(&session.draft), visited);
    let mut restarted = restart(session.draft.clone());
    let reopened = reopened_businesses(restarted.draft.clone()).remove(0);
    let request = save_topic(
        &mut restarted,
        TaxSection::SelfEmployment,
        vec![document(id, IncomeSource::Record { value: crate::IncomeRecord::Business(reopened) })],
    );
    accepted(&send(&mut restarted, request));
    assert_eq!(restarted.draft, session.draft);
    assert_eq!(supported(&restarted.draft), visited);
}

#[test]
fn uncapped_auto_retains_existing_income_limit_and_partial_carry() {
    let draft = crate::tests::support::builders::uncapped_auto_179();
    let result = supported(&draft);
    assert_eq!(value(&result, "business_2_asset_0_section179"), 3_000);
    assert_eq!(value(&result, "business_2_asset_0_section179_carryforward"), 2_000);
    assert_eq!(value(&result, "business_2_asset_0_macrs"), 6_400);
    assert_eq!(value(&result, "business_2_depreciation_total"), 9_400);
}

#[test]
fn actual_later_179_releases_preserve_each_historical_target_reserve() {
    let asset = crate::tests::support::builders::auto_with_retained_179_release();
    let first = crate::auto_recovery::recovery(&asset, 2023, 0).unwrap();
    let second = crate::auto_recovery::recovery(&asset, 2024, 0).unwrap();
    assert_eq!(first.section179.get(), 500_000);
    assert_eq!(first.closing_section179_carry.get(), 500_000);
    assert_eq!(second.section179.get(), 200_000);
    assert_eq!(second.closing_section179_carry.get(), 300_000);
    let current = crate::auto_recovery::recovery(&asset, 2026, 300_000).unwrap();
    assert_eq!(current.allowance.get(), 396_000);
    assert_eq!(current.closing_basis.get(), 4_964_000);
}

#[test]
fn bonus_positive_auto_uses_amt179_to_compute_its_own_capped_bonus() {
    let draft = crate::tests::support::builders::bonus_auto_179();
    let FederalIncome::Business { data } = &draft.income[0].income else { unreachable!() };
    let projected = crate::auto_recovery::amt_projection(&data.assets[0]).unwrap();
    let records = projected.history_details.as_ref().unwrap();
    // The same original100% nominal allowance meets18000 firstcap; actual
    // regular1795000 leaves13000bonus, AMT1793000 leaves15000bonus.
    assert_eq!(records.original_bonus, 1_500_000);
    assert_eq!(records.years[0].bonus, 1_500_000);
    let regular =
        crate::section179_return::prepare(&draft, Section179IncomeSystem::Regular).unwrap();
    let alternative =
        crate::section179_return::prepare(&draft, Section179IncomeSystem::Alternative).unwrap();
    assert_eq!(regular.allocation.allowed(2), Ok(500_000));
    assert_eq!(alternative.allocation.allowed(2), Ok(576_000));
    assert_eq!(
        alternative.allocation.asset_recovery(2, 3).unwrap().closing_section179_carry.get(),
        124_000
    );
    supported(&draft);
}

#[test]
fn missing_amt_history_stays_unavailable_even_when_regular_carry_is_zero() {
    let mut draft = crate::tests::support::builders::bonus_auto_179();
    let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
    let asset = &mut data.assets[0];
    // Complete regular records exhaust the election in2018; they do not
    // establish the independently income-limited AMT179 source.
    if let EquipmentHistory::Continuing { carryforward, .. } = &mut asset.history {
        *carryforward = 0;
    }
    let records = asset.history_details.as_mut().unwrap();
    records.years[0].section179_allowed = 1_000_000;
    records.years[0].bonus = 800_000;
    records.original_bonus = 800_000;
    records.carryovers.clear();
    records.auto_history.as_mut().unwrap().post_recovery_basis = 4_200_000;
    for row in &mut records.years {
        row.amt_section179_allowed = SourceCents::Unavailable;
    }
    if let EquipmentHistory::Continuing { schedule, .. } = &mut asset.history {
        *schedule = EquipmentSchedule::Tables { initial_bonus: 800_000 };
    }
    asset.amt_section179_income_carry = Some(700_000);
    assert!(crate::auto_recovery::amount(asset, 2026).is_some());
    assert!(
        crate::section179_return::needed(crate::related_forms::RelatedFormContext::original(
            &draft,
            &crate::related_forms::effective_claimants(&draft)
        ))
        .unwrap()
    );
    assert!(
        crate::section179_return::prepare(&draft, Section179IncomeSystem::Alternative).is_err()
    );
    assert_eq!(estimate(&draft).state, ResultState::Unsupported);
}

#[test]
fn contradictory_actual_amt_annual_totals_are_rejected_by_native_save() {
    let mut draft = mixed_auto_179(100_000, SourceCents::Reported { amount: 500_000 });
    let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
    data.assets[0] = crate::tests::support::builders::auto_with_retained_179_release();
    let mut session = restart(draft);
    let before = session.draft.clone();
    let mut page = reopened_businesses(before.clone()).remove(0);
    let VehicleMethod::Actual { history_details: Some(records), .. } = &mut page.vehicles[0].method
    else {
        unreachable!()
    };
    records.years[1].amt_section179_allowed = SourceCents::Reported { amount: 600_000 };
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(2, IncomeSource::Record { value: crate::IncomeRecord::Business(page) })],
    );
    // The annual history owner identifies this cross-row inconsistency at the
    // submitted history list, preserving the accepted return on refusal.
    crate::tests::support::assert::k1res::assert_located_rejection(
        &mut session,
        request,
        "AUTO_RECOVERY_HISTORY_INCONSISTENT",
        "[documents/documents/0/document/vehicles/0/method/history_details/years]",
    );
    assert_eq!(session.draft, before);
}

#[test]
fn community_auto_requires_own_work_and_keeps_the_existing_odd_cent_share_once() {
    let mut draft = crate::tests::support::builders::community_auto_179();
    // 2026-10-23: Reg. 1.179-2(c)(6), (8): combined hours do not
    // establish individual conduct; physical community sharing stays exact.
    crate::tests::support::assert::assert_mfs_section179_work_refused(
        &draft,
        2,
        TaxSection::SelfEmployment,
    );
    let owner = crate::community::income_amount(&draft, 2, 8_040_001);
    assert_eq!(owner, 4_020_001);
    draft.income[0].owner = crate::DocumentOwner::Spouses;
    let other = crate::community::income_amount(&draft, 2, 8_040_001);
    assert_eq!(other, 4_020_000);
    assert_eq!(owner + other, 8_040_001);
}
