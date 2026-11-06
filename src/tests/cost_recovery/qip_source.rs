use super::*;
use crate::macrs_source::MacrsElectionView;
use crate::tests::cross::interview::{
    PAGE_REVIEW, assert_round_trip, go_to, open_topic, reopened_businesses, save_businesses,
    stored_business,
};
use crate::tests::support::builders::ads::{qip_asset, qip_building, qip_draft};
use crate::{
    BuildingFirstService as Service, RealImprovementKind as Work, RecoveryBuildingKind as Kind,
};

#[test]
fn qip_gds_uses_15_year_sl_and_actual_form_4562_row_without_changing_source_class() {
    let draft = qip_draft(qip_asset(30), "Business");
    let result = supported(&draft);
    let line = result.lines.iter().find(|line| line.id == "business_2_asset_0_macrs").unwrap();
    assert_eq!(line.amount, 33_333);
    assert_eq!(line.form_line.as_ref().unwrap().line, "19e");
    let asset = &crate::nonmacrs::recovery_assets(&draft.income.last().unwrap().income).unwrap()[0];
    assert_eq!(asset.class, EquipmentClass::ThirtyNine);
    let view = MacrsElectionView::new(&draft);
    assert!(asset.needs_macrs_in_context(view.context(Owner::Taxpayer, Some(2), asset)));
}

#[test]
fn qip_required_ads_uses_20_year_sl_but_voluntary_ads_keeps_bonus() {
    for (required, expected_bonus, expected_regular) in [(true, 0, 25_000), (false, 1_000_000, 0)] {
        let mut asset = qip_asset(30);
        asset.history = EquipmentHistory::New {
            section179: Section179::Ineligible,
            bonus: BonusElection::Apply,
            tables: true,
        };
        let source = asset.macrs_source.as_mut().unwrap();
        if required {
            source.restrictions = Some(vec![crate::AdsRestrictionPeriod {
                first_year: 2026,
                last_year: None,
                kind: crate::AdsRestriction::TaxExemptUse,
            }]);
        } else {
            source.election = Some(crate::AdsElection { tax_year: 2026 });
        }
        let draft = qip_draft(asset, "Business");
        let result = supported(&draft);
        assert_eq!(
            (
                line_amount(&result, "business_2_asset_0_bonus"),
                line_amount(&result, "business_2_asset_0_macrs")
            ),
            (expected_bonus, expected_regular)
        );
        let line = result.lines.iter().find(|line| line.id == "business_2_asset_0_macrs").unwrap();
        assert_eq!(line.form_line.as_ref().unwrap().line, "20");
    }
}

#[test]
fn qip_exact_and_before_service_evidence_proves_only_its_actual_ordering() {
    let asset = qip_asset(30);
    for (evidence, expected) in [
        (Some(Service::ExactDate { date: 20260531 }), Ok(true)),
        (Some(Service::ExactDate { date: 20260601 }), Ok(false)),
        (Some(Service::ExactDate { date: 20260602 }), Ok(false)),
        (Some(Service::BeforeDate { date: 20260601 }), Ok(true)),
        (Some(Service::BeforeDate { date: 20260602 }), Err("ASSET_QIP_SOURCE_CONTEXT_UNAVAILABLE")),
        (Some(Service::BeforeDate { date: 20270101 }), Err("ASSET_QIP_SOURCE_CONTEXT_UNAVAILABLE")),
        (None, Err("ASSET_QIP_SOURCE_CONTEXT_UNAVAILABLE")),
    ] {
        let building =
            crate::BuildingUseSource { first_service: evidence, kind: Kind::Nonresidential };
        assert_eq!(
            asset.qualified_improvement(crate::macrs_source::AdsInvocation {
                building: Some(&building),
                ..Default::default()
            }),
            expected
        );
    }
}

#[test]
fn qip_exclusions_and_section1245_equipment_keep_their_actual_character() {
    let building = qip_building(30);
    let context = crate::macrs_source::AdsInvocation {
        building: Some(&building.facts),
        ..Default::default()
    };
    for work in [
        Work::Interior1250 { made_by_taxpayer: false },
        Work::Interior1245Equipment,
        Work::Enlargement,
        Work::ElevatorOrEscalator,
        Work::StructuralFramework,
        Work::Other,
    ] {
        let mut asset = qip_asset(30);
        asset.macrs_source.as_mut().unwrap().improvement.as_mut().unwrap().kind = work;
        if work == Work::Interior1245Equipment {
            asset.class = EquipmentClass::Five;
        }
        assert_eq!(asset.qualified_improvement(context), Ok(false));
        assert_eq!(asset.section1250(), work != Work::Interior1245Equipment);
    }
    for kind in [Kind::ResidentialRental, Kind::PersonalResidence] {
        let facts = crate::BuildingUseSource { first_service: building.facts.first_service, kind };
        assert_eq!(
            qip_asset(30).qualified_improvement(crate::macrs_source::AdsInvocation {
                building: Some(&facts),
                ..Default::default()
            }),
            Ok(false)
        );
    }
    let mut old = qip_asset(30);
    old.placed = 20171231;
    assert_eq!(old.qualified_improvement(context), Ok(false));
}

#[test]
fn building_registration_then_link_save_reopen_and_restart_preserves_source_and_global_ids() {
    let mut asset = qip_asset(30);
    asset.id = 0;
    asset.macrs_source.as_mut().unwrap().improvement = None;
    let mut pages = reopened_businesses(business_with_bonus(100_000, asset));
    pages[0].improvement_buildings.push(qip_building(0));
    let mut session = restart(complete(FederalStatus::Single));
    accepted(&save_businesses(&mut session, pages));
    let (_, business) = stored_business(&session);
    let id = business.improvement_buildings[0].id;
    assert!(id > 0 && id != business.assets[0].id);
    let mut linking = open_topic(&mut session, TaxSection::SelfEmployment);
    let P::SaveIncomes { rental_energy_buildings, documents, .. } = &mut linking else {
        unreachable!()
    };
    assert!(rental_energy_buildings.iter().any(|choice| choice.id == id));
    let crate::IncomeRecord::Business(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    let crate::AssetRecoveryPage::Gds { facts } = &mut page.assets[0].recovery else {
        unreachable!()
    };
    facts.source.as_mut().unwrap().improvement = Some(crate::RealImprovementPage {
        building_id: id,
        kind: Work::Interior1250 { made_by_taxpayer: true },
    });
    accepted(&send(&mut session, linking));
    let saved = session.draft.clone();
    assert!(saved.valid_stored());
    let mut restarted = restart(saved.clone());
    let pages = reopened_businesses(saved.clone());
    assert_round_trip(&mut restarted, &pages);
    assert_eq!(stored_business(&restarted).1.improvement_buildings[0].id, id);
    assert_eq!(line_amount(&supported(&restarted.draft), "business_2_asset_0_macrs"), 33_333);
}

#[test]
fn physical_building_facts_are_return_wide_and_survive_omitted_source_income() {
    let mut draft = qip_draft(qip_asset(30), "Business");
    draft.setup = complete(FederalStatus::Joint).setup;
    let mut source = cash_business(0);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    data.improvement_buildings.push(qip_building(30));
    let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.improvement_buildings.clear();
    let mut entry = income(source);
    entry.owner = crate::DocumentOwner::Spouses;
    add(&mut draft, entry);
    let view = alloc::sync::Arc::new(MacrsElectionView::new(&draft));
    let mut working = draft.clone();
    working.income.pop();
    let claims = crate::related_forms::effective_claimants(&working);
    let original = crate::related_forms::RelatedFormContext::original(&working, &claims)
        .with_macrs(&view)
        .macrs();
    let asset =
        &crate::nonmacrs::recovery_assets(&working.income.last().unwrap().income).unwrap()[0];
    assert_eq!(
        asset.source_macrs_amount_in_context(
            false,
            original.context(Owner::Taxpayer, Some(2), asset)
        ),
        Some(33_333)
    );
    assert_eq!(
        asset.macrs_source_issue_in_context(MacrsElectionView::new(&working).context(
            Owner::Taxpayer,
            Some(2),
            asset
        )),
        Some("ASSET_QIP_SOURCE_CONTEXT_UNAVAILABLE")
    );
}

#[test]
fn registered_qip_feeds_actual_farm_rental_and_schedule_e_equipment_consumers() {
    for family in ["Farm", "FarmRental", "Rental"] {
        let draft = qip_draft(qip_asset(30), family);
        let entry = draft.income.last().unwrap();
        let assets = crate::nonmacrs::recovery_assets(&entry.income).unwrap();
        let amounts = crate::nonmacrs::asset_depreciation_amounts_for(
            &draft, entry.id, assets, false, false, None,
        )
        .unwrap();
        assert_eq!(amounts.expense_cents(0), Some(33_333), "{family}");
        supported(&draft);
    }
}

#[test]
fn qip_mid_quarter_test_uses_the_actual_15_year_property_basis() {
    let mut asset = qip_asset(30);
    asset.placed = 20261001;
    let draft = qip_draft(asset, "Business");
    assert!(crate::depreciation::mid_quarter(&draft));
    assert_eq!(line_amount(&supported(&draft), "business_2_asset_0_macrs"), 8_333);
}

#[test]
fn qip_179_and_excluded_roof_keep_separate_bonus_and_residual_periods() {
    for (work, expected_bonus, expected_macrs) in
        [(Work::Interior1250 { made_by_taxpayer: true }, 800_000, 0), (Work::Roof, 0, 11_111)]
    {
        let mut asset = qip_asset(30);
        asset.description = "Separate improvement".into();
        asset.history = EquipmentHistory::New {
            section179: Section179::Eligible { elected: 200_000 },
            bonus: BonusElection::Apply,
            tables: true,
        };
        asset.macrs_source.as_mut().unwrap().improvement.as_mut().unwrap().kind = work;
        let draft = qip_draft(asset, "Business");
        let pages = reopened_businesses(draft.clone());
        let mut session = restart(draft);
        assert_round_trip(&mut session, &pages);
        let result = supported(&session.draft);
        assert_eq!(line_amount(&result, "business_2_asset_0_macrs"), expected_macrs);
        assert_eq!(line_amount(&result, "business_2_asset_0_bonus"), expected_bonus);
        assert_eq!(value(&result, "section179_deduction"), 2_000);
    }
}

#[test]
fn qip_missing_or_ambiguous_physical_source_omits_only_current_recovery() {
    let mut draft = qip_draft(qip_asset(30), "Business");
    let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.improvement_buildings.clear();
    let before = draft.clone();
    let result = supported(&draft);
    assert_eq!(line_amount(&result, "business_2_asset_0_macrs"), 0);
    assert_eq!(
        result
            .review_notes
            .iter()
            .filter(|n| n.display.key == "WARN_ASSET_MACRS_RECOVERY_LEFT_OUT")
            .count(),
        1
    );
    assert_eq!(draft, before);
    let mut eligible = draft.clone();
    let FederalIncome::Business { data } = &mut eligible.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let EquipmentHistory::New { section179, .. } = &mut data.assets[0].history else {
        unreachable!()
    };
    *section179 = Section179::Eligible { elected: 0 };
    assert!(estimate(&eligible).issues.iter().any(|issue| {
        issue.display.key == "ASSET_QIP_SOURCE_CONTEXT_UNAVAILABLE"
            && issue.section == "Section 179 source"
    }));
    let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.improvement_buildings.extend([qip_building(30), qip_building(30)]);
    let view = MacrsElectionView::new(&draft);
    assert_eq!(
        qip_asset(30).qualified_improvement(view.building_context(&qip_asset(30))),
        Err("ASSET_QIP_SOURCE_CONTEXT_UNAVAILABLE")
    );
    assert!(
        crate::source_associations::SourceAssociations::new(&draft)
            .physical_building_choices()
            .is_empty()
    );
}

#[test]
fn original_building_evidence_keeps_mid_quarter_at_the_selected_179_stage() {
    let mut asset = qip_asset(30);
    asset.placed = 20261001;
    let draft = qip_draft(asset, "Business");
    let view = MacrsElectionView::new(&draft);
    let mut working = draft.clone();
    let FederalIncome::Business { data } = &mut working.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.improvement_buildings.clear();
    assert!(crate::depreciation::mid_quarter_with_macrs(&working, Some(&view)));
    let mut early =
        macrs_equipment(20_000, EquipmentClass::Five, EquipmentMethod::Regular, 20260101);
    early.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 2_000_000 },
        bonus: BonusElection::ElectOut,
        tables: true,
    };
    let FederalIncome::Business { data } = &mut working.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.assets.push(early);
    assert!(
        crate::depreciation::mid_quarter_with_macrs(&working, Some(&view)),
        "selected full179 removes the early property's convention basis"
    );
    let FederalIncome::Business { data } = &mut working.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let EquipmentHistory::New { section179, .. } = &mut data.assets[1].history else {
        unreachable!()
    };
    *section179 = Section179::Eligible { elected: 0 };
    assert!(
        !crate::depreciation::mid_quarter_with_macrs(&working, Some(&view)),
        "the actual stage's selected allowance controls basis"
    );
}

#[test]
fn service_year_qip_cooperative_refund_uses_settled_cost_in_the_same_sl_schedule() {
    let (mut draft, activity, _) = cooperative_asset_refunds(qip_asset(30), &[[200_000, 0, 0, 0]]);
    let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
    data.improvement_buildings.push(qip_building(30));
    let FederalIncome::Business { data } = &draft.income[0].income else { unreachable!() };
    let asset = &data.assets[0];
    for amt in [false, true] {
        let amounts =
            crate::cooperative_basis::source_amounts(&draft, activity, asset, false, amt).unwrap();
        assert_eq!((amounts.opening, amounts.bonus, amounts.macrs), (800_000, 0, 26_667));
    }
}

#[test]
fn qip_actual_prior_bonus_and_ads_histories_feed_1250_disposal_once() {
    for (ads, initial_bonus, prior, expected) in [
        (false, 0, 33_333, (667, 0, 667)),
        (true, 0, 25_000, (500, 0, 500)),
        (false, 1_000_000, 1_000_000, (10_000, 9_333, 667)),
    ] {
        let mut asset = qip_asset(30);
        asset.acquired = 20250120;
        asset.placed = 20250201;
        asset.history = table_history(prior, initial_bonus, false);
        asset.disposal = crate::AssetDisposal::Disposed {
            date: 20261001,
            how: crate::DisposalKind::Sold {
                proceeds: 1_000_000,
                selling_expenses: 0,
                buyer: crate::Buyer::Unrelated,
                livestock: crate::SoldLivestock::No,
                payment: crate::SalePayment::InFull,
            },
        };
        let source = asset.macrs_source.as_mut().unwrap();
        source.original_system = Some(if ads {
            crate::OriginalMacrsSystem::Ads { began_year: 2025 }
        } else {
            crate::OriginalMacrsSystem::Gds
        });
        if ads {
            source.restrictions = Some(vec![crate::AdsRestrictionPeriod {
                first_year: 2025,
                last_year: None,
                kind: crate::AdsRestriction::TaxExemptUse,
            }]);
        }
        let mut draft = qip_draft(asset, "Business");
        draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
        let asset =
            &crate::nonmacrs::recovery_assets(&draft.income.last().unwrap().income).unwrap()[0];
        let sale = crate::cooperative_basis::sale(&draft, 2, asset, false, false).unwrap().unwrap();
        assert_eq!((sale.depreciation, sale.ordinary, sale.unrecaptured), expected);
        supported(&draft);
    }
}

#[test]
fn foreign_rental_location_requires_qip_ads_despite_a_conflicting_domestic_packet_year() {
    let mut asset = qip_asset(30);
    asset.history = EquipmentHistory::New {
        section179: Section179::Ineligible,
        bonus: BonusElection::Apply,
        tables: true,
    };
    let mut draft = qip_draft(asset, "Rental");
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.location = crate::RentalLocation::Foreign { country: "CA".into() };
    let original = draft.clone();
    let result = supported(&draft);
    assert_eq!(
        (
            line_amount(&result, "rental_2_asset_0_bonus"),
            line_amount(&result, "rental_2_asset_0_macrs")
        ),
        (0, 25_000)
    );
    assert_eq!(draft, original);
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.improvement_buildings.clear();
    assert!(
        estimate(&draft)
            .issues
            .iter()
            .any(|issue| { issue.display.key == "RENTAL_FOREIGN_ASSET_ADS_UNAVAILABLE" })
    );
}

#[test]
fn employee_qip_keeps_registered_job_building_and_actual_2106_recovery_after_reopen() {
    let mut session = restart(crate::tests::support::builders::ads::employee_qip_draft());
    let request = go_to(&session.draft, PAGE_REVIEW, TaxSection::EmployeeExpenses);
    let Reply::Edit(page @ P::SaveEmployeeExpenses { .. }) = send(&mut session, request) else {
        unreachable!()
    };
    accepted(&send(&mut session, page));
    let saved = session.draft.clone();
    let mut restarted = restart(saved.clone());
    let request = go_to(&restarted.draft, PAGE_REVIEW, TaxSection::EmployeeExpenses);
    let Reply::Edit(page @ P::SaveEmployeeExpenses { .. }) = send(&mut restarted, request) else {
        unreachable!()
    };
    accepted(&send(&mut restarted, page));
    assert_eq!(restarted.draft, saved);
    assert_eq!(
        line_amount(
            &supported(&restarted.draft),
            "employee_expenses_1_equipment_depreciation_class_19E"
        ),
        33_300
    );
}

#[test]
fn rental_energy_building_asks_recovery_only_after_actual_improvement_link_and_preserves_correction()
 {
    let mut session = restart(rental_energy_return());
    let mut submitted = open_topic(&mut session, TaxSection::Rentals);
    let P::SaveIncomes { documents, .. } = &mut submitted else { unreachable!() };
    let crate::IncomeRecord::Rental(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    assert!(page.more.energy_buildings[0].allowed_recovery.is_empty());
    let physical = page.more.energy_buildings[0].id;
    let before_building: crate::RentalEnergyBuildingFacts =
        page.more.energy_buildings[0].clone().into();
    let mut asset = qip_asset(physical);
    asset.id = 0;
    page.assets.push(asset.into());
    let before = session.draft.clone();
    let Reply::Edit(mut corrected @ P::SaveIncomes { .. }) = send(&mut session, submitted) else {
        panic!("new building source question needs correction editor")
    };
    assert_eq!(session.draft, before);
    let P::SaveIncomes { documents, revision, warnings, .. } = &mut corrected else {
        unreachable!()
    };
    assert_eq!(*revision, before.revision);
    assert_eq!(
        status_message(&warnings[0].message).key,
        "BUILDING_RECOVERY_SOURCE_QUESTIONS_UPDATED"
    );
    let crate::IncomeRecord::Rental(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    assert_eq!(page.more.energy_buildings[0].allowed_recovery, vec![0]);
    let unchanged: crate::RentalEnergyBuildingFacts = page.more.energy_buildings[0].clone().into();
    assert_eq!(unchanged, before_building);
    page.more.energy_buildings[0].recovery =
        Some(crate::BuildingRecoveryRecord::Record { facts: qip_building(physical).facts });
    accepted(&send(&mut session, corrected));
    let saved = session.draft.clone();
    let mut restarted = restart(saved);
    let mut reopened = open_topic(&mut restarted, TaxSection::Rentals);
    let P::SaveIncomes { documents, .. } = &mut reopened else { unreachable!() };
    let crate::IncomeRecord::Rental(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    assert_eq!(page.more.energy_buildings[0].id, physical);
    assert!(page.more.energy_buildings[0].recovery.is_some());
    page.assets.clear();
    accepted(&send(&mut restarted, reopened));
    let reopened = open_topic(&mut restarted, TaxSection::Rentals);
    let P::SaveIncomes { documents, .. } = reopened else { unreachable!() };
    let crate::IncomeRecord::Rental(page) = &documents.documents[0].document else {
        unreachable!()
    };
    assert_eq!(page.more.energy_buildings[0].allowed_recovery, vec![0]);
    assert!(page.more.energy_buildings[0].recovery.is_some());
}

#[test]
fn qip_bonus_election_uses_15_year_class_without_imposing_its_statutory_sl_method() {
    let mut draft = qip_draft(qip_asset(30), "Business");
    let mut ordinary =
        macrs_equipment(10_000, EquipmentClass::Fifteen, EquipmentMethod::StraightLine, 20260601);
    ordinary.id = 32;
    ordinary.history = EquipmentHistory::New {
        section179: Section179::Ineligible,
        bonus: BonusElection::Apply,
        tables: true,
    };
    let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.assets.push(ordinary);
    assert!(crate::depreciation::election_conflict(&draft).is_some());
    let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let EquipmentHistory::New { bonus, .. } = &mut data.assets[1].history else { unreachable!() };
    *bonus = BonusElection::ElectOut;
    assert!(crate::depreciation::election_conflict(&draft).is_none());
}
