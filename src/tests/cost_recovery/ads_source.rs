use super::*;
use crate::tests::cross::interview::{
    assert_round_trip, open_topic, reopened_businesses, save_businesses, stored_business,
};
use crate::tests::support::builders::ads::{
    ads_editor_answers, ads_question_correction_draft, employee_with_inherited_ads_vehicle,
    farm_with_capitalization_election, income_with_ads_asset, required_ads_equipment,
    voluntary_ads_equipment, voluntary_ads_passenger_car,
};

#[test]
fn required_ads_uses_exact_lease_floor_and_withholds_bonus_once() {
    let asset = required_ads_equipment();
    // $10,000 * 6 / 61.25 = $979.5918..., projected once to97,959 cents.
    assert_eq!(asset.bonus_amount(), 0);
    assert_eq!(asset.macrs_amount(false), 97_959);
    assert_eq!(asset.deduction_for(false, false), 97_959);
    assert_eq!(asset.deduction_for(false, true), 97_959);
    let result = supported(&business_with_bonus(100_000, asset));
    assert_eq!(line_amount(&result, "business_2_asset_0_macrs"), 97_959);
    assert_eq!(line_amount(&result, "business_2_asset_0_bonus"), 0);
}

#[test]
fn voluntary_ads_preserves_bonus_and_uses_sl_when_bonus_is_elected_out() {
    let mut asset = voluntary_ads_equipment();
    // Form4562's bonus exception concerns required ADS, not a voluntary election.
    assert_eq!(asset.bonus_amount(), 1_000_000);
    assert_eq!(asset.deduction(false), 1_000_000);
    asset.history = EquipmentHistory::New {
        section179: Section179::Ineligible,
        bonus: BonusElection::ElectOut,
        tables: false,
    };
    assert_eq!(asset.bonus_amount(), 0);
    assert_eq!(asset.macrs_amount(false), 100_000);
    assert_eq!(asset.deduction(false), 100_000);
}

#[test]
fn accepted_ads_source_reopens_and_restarts_without_changing_answers() {
    for asset in [required_ads_equipment(), voluntary_ads_passenger_car()] {
        let expected_source = asset.macrs_source.clone();
        let pages = reopened_businesses(business_with_bonus(100_000, asset));
        assert_eq!(pages[0].assets.len() + pages[0].vehicles.len(), 1);
        let mut session = restart(complete(FederalStatus::Single));
        accepted(&save_businesses(&mut session, pages));
        let (_, stored) = stored_business(&session);
        assert!(stored.assets[0].id > 0);
        assert_eq!(stored.assets[0].macrs_source, expected_source);
        let saved = session.draft.clone();
        let pages = reopened_businesses(saved.clone());
        assert_round_trip(&mut session, &pages);
        let restarted = restart(saved);
        assert_eq!(stored_business(&restarted).1.assets[0].macrs_source, expected_source);
        supported(&restarted.draft);
    }
}

#[test]
fn absent_native_source_stays_absent_through_complete_editor_round_trip() {
    let mut vehicle = voluntary_ads_passenger_car();
    vehicle.macrs_source = None;
    let mut equipment = required_ads_equipment();
    equipment.macrs_source = None;
    for asset in [equipment, vehicle] {
        assert!(asset.macrs_source.is_none());
        let pages = reopened_businesses(business_with_bonus(100_000, asset));
        let mut session = restart(complete(FederalStatus::Single));
        accepted(&save_businesses(&mut session, pages));
        assert!(stored_business(&session).1.assets[0].macrs_source.is_none());
        let pages = reopened_businesses(session.draft.clone());
        assert_round_trip(&mut session, &pages);
        assert!(
            stored_business(&restart(session.draft.clone())).1.assets[0].macrs_source.is_none()
        );
    }
}

#[test]
fn unknown_ads_class_life_omits_only_the_deduction_and_preserves_source_records() {
    let mut asset = required_ads_equipment();
    asset.macrs_source.as_mut().unwrap().class_life = None;
    assert_eq!(asset.macrs_source_issue(), Some("ASSET_ADS_CLASS_LIFE_UNAVAILABLE"));
    let draft = business_with_bonus(100_000, asset);
    let original = draft.clone();
    let result = supported(&draft);
    let mut control = complete(FederalStatus::Single);
    add(&mut control, income(cash_business(100_000)));
    let control = supported(&control);
    for id in ["agi", "self_employment_tax", "qbi_deduction"] {
        assert_eq!(value(&result, id), value(&control, id), "{id}");
    }
    assert_eq!(result.tax, control.tax);
    assert_eq!(
        result
            .review_notes
            .iter()
            .filter(|note| note.display.key == "WARN_ASSET_MACRS_RECOVERY_LEFT_OUT")
            .count(),
        1,
    );
    assert_eq!(draft, original);
}

#[test]
fn one_class_election_applies_across_businesses_and_rentals_without_repeating_the_answer() {
    for (rental_activity, packet_supplied) in
        [(false, true), (false, false), (true, true), (true, false)]
    {
        let mut elected = voluntary_ads_equipment();
        elected.history = EquipmentHistory::New {
            section179: Section179::Ineligible,
            bonus: BonusElection::ElectOut,
            tables: false,
        };
        let mut sibling = elected.clone();
        if packet_supplied {
            let source = sibling.macrs_source.as_mut().unwrap();
            source.election = None;
            source.original_system = None;
        } else {
            sibling.macrs_source = None;
        }
        let mut draft = business_with_bonus(100_000, elected);
        add(&mut draft, income(income_with_ads_asset(sibling, rental_activity)));
        let original = draft.clone();
        let result = supported(&draft);
        assert_eq!(line_amount(&result, "business_2_asset_0_macrs"), 100_000);
        let expected = if packet_supplied { 100_000 } else { 0 };
        let prefix = if rental_activity { "rental" } else { "business" };
        let asset_line = alloc::format!("{prefix}_3_asset_0_macrs");
        if rental_activity && !packet_supplied {
            assert!(
                result.lines.iter().all(|line| line.id != asset_line),
                "unavailable rental recovery must not emit a supported asset amount",
            );
        } else {
            assert_eq!(line_amount(&result, &asset_line), expected);
        }
        if rental_activity {
            let mut no_equipment = draft.clone();
            let FederalIncome::Rental { data } = &mut no_equipment.income[2].income else {
                unreachable!()
            };
            data.assets.clear();
            let control = supported(&no_equipment);
            assert_eq!(
                line_amount(&result, "rental_3_depreciation")
                    - line_amount(&control, "rental_3_depreciation"),
                expected,
            );
        }
        assert_eq!(
            result
                .review_notes
                .iter()
                .filter(|note| note.display.key == "WARN_ASSET_MACRS_RECOVERY_LEFT_OUT")
                .count(),
            usize::from(!packet_supplied),
        );
        assert_eq!(draft, original);
    }
}

#[test]
fn retained_farm_capitalization_election_requires_ads_in_original_source_context() {
    for source_supplied in [true, false] {
        let mut asset = required_ads_equipment();
        let source = asset.macrs_source.as_mut().unwrap();
        source.restrictions = Some(vec![]);
        source.lease = None;
        if !source_supplied {
            asset.macrs_source = None;
        }
        let draft = farm_with_capitalization_election(asset);
        assert!(draft.valid_stored(), "farm source identities must use the accepted counter");
        let original = draft.clone();
        let claims = alloc::collections::BTreeMap::new();
        let original_view =
            crate::related_forms::RelatedFormContext::original(&draft, &claims).macrs();
        let mut working = draft.clone();
        let entry = working.income.last_mut().unwrap();
        let FederalIncome::Business { data } = &mut entry.income else { unreachable!() };
        let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
        farm.specials.production.clear();
        let view = crate::related_forms::RelatedFormContext::original(&working, &claims)
            .with_macrs(&original_view)
            .macrs();
        assert!(alloc::sync::Arc::ptr_eq(&view, &original_view));
        let entry = working.income.last().unwrap();
        let FederalIncome::Business { data } = &entry.income else { unreachable!() };
        let context = view.context(entry.owner.owned(), Some(entry.id), &data.assets[0]);
        assert!(
            context.required.is_some(),
            "actual original election survives omitted production cost"
        );
        assert_eq!(data.assets[0].source_bonus_amount_in_context(context), 0);
        assert_eq!(
            data.assets[0].source_macrs_amount_in_context(false, context),
            source_supplied.then_some(100_000)
        );
        assert_eq!(
            data.assets[0].macrs_source_issue_in_context(context),
            (!source_supplied).then_some("ASSET_ADS_SOURCE_UNAVAILABLE")
        );
        let result = estimate(&draft);
        assert!(
            result.tax.is_none(),
            "the production owner still needs its original election-out recovery facts"
        );
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.display.key == "ISSUE_FARM_PRODUCTION_BASIS_OWNER")
        );
        assert_eq!(draft, original);
    }
}

#[test]
fn contextual_ads_passenger_car_keeps_the_existing_280f_cap() {
    let asset = voluntary_ads_passenger_car();
    // The $20,000 straight-line amount exceeds the existing $12,300 first-year limit.
    assert_eq!(asset.macrs_amount(false), 1_230_000);
    let result = supported(&business_with_bonus(100_000, asset));
    assert_eq!(line_amount(&result, "business_2_asset_0_macrs"), 1_230_000);
}

#[test]
fn employee_vehicle_borrows_class_election_and_keeps_operating_costs_when_records_are_missing() {
    for packet_supplied in [true, false] {
        let draft = employee_with_inherited_ads_vehicle(packet_supplied);
        let original = draft.clone();
        let result = supported(&draft);
        let expected = 400_000 + if packet_supplied { 1_230_000 } else { 0 };
        assert_eq!(line_amount(&result, "employee_expenses_1_line_1"), expected);
        assert_eq!(line_amount(&result, "schedule_1_employee_business_expenses"), expected);
        assert_eq!(
            result
                .review_notes
                .iter()
                .filter(|note| { note.display.key == "WARN_ASSET_MACRS_RECOVERY_LEFT_OUT" })
                .count(),
            usize::from(!packet_supplied),
        );
        assert_eq!(draft, original);
    }
}

#[test]
fn changed_owner_reprojects_complete_source_editor_without_accepting_or_losing_answers() {
    let mut session = restart(ads_question_correction_draft(true, Some(Owner::Spouse)));
    let mut submitted = open_topic(&mut session, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, warnings, .. } = &mut submitted else { unreachable!() };
    assert!(warnings.is_empty());
    let target = &mut documents.documents[1];
    target.owner = Owner::Spouse;
    let crate::IncomeRecord::Business(business) = &mut target.document else { unreachable!() };
    business.assets[0].cost += 12_345;
    let facts = business.assets[0].gds().unwrap();
    assert!(facts.source.as_ref().unwrap().allowed_class_life.is_empty());
    assert!(facts.source.as_ref().unwrap().class_life.is_none());
    let accepted_draft = session.draft.clone();
    let accepted_dirty = session.dirty;
    let before = submitted.clone();
    let Reply::Edit(corrected @ P::SaveIncomes { .. }) = send(&mut session, submitted) else {
        panic!("owner-dependent source questions need a complete correction editor")
    };
    assert_eq!(session.draft, accepted_draft);
    assert_eq!(session.dirty, accepted_dirty);
    assert_eq!(ads_editor_answers(corrected.clone()), ads_editor_answers(before));
    let P::SaveIncomes { documents, warnings, cookie, revision, .. } = &corrected else {
        unreachable!()
    };
    assert_eq!(cookie, &accepted_draft.cookie);
    assert_eq!(*revision, accepted_draft.revision);
    assert_eq!(warnings.len(), 1);
    assert_eq!(status_message(&warnings[0].message).key, "ASSET_MACRS_SOURCE_QUESTIONS_UPDATED");
    assert!(status_text(&warnings[0].message).starts_with("Record 2, property 1:"));
    let crate::IncomeRecord::Business(target) = &documents.documents[1].document else {
        unreachable!()
    };
    let source = target.assets[0].gds().unwrap().source.as_ref().unwrap();
    assert_eq!(source.allowed_class_life, vec![0]);
    assert!(source.class_life.is_none(), "explicit unknown is still the person's answer");
    let expected_source: crate::MacrsRecoverySource = source.clone().into();
    let expected_cost = target.assets[0].cost;
    let reply = send(&mut session, corrected);
    accepted(&reply);
    assert!(
        session.draft.revision > accepted_draft.revision,
        "a second Save must not loop through correction"
    );
    let saved = session.draft.clone();
    let target = saved.income.iter().find(|entry| entry.id == accepted_draft.income[2].id).unwrap();
    assert_eq!(target.owner, crate::DocumentOwner::Spouses);
    let FederalIncome::Business { data } = &target.income else { unreachable!() };
    assert_eq!(data.assets[0].cost, expected_cost);
    assert_eq!(data.assets[0].macrs_source, Some(expected_source));
    let mut restarted = restart(saved.clone());
    let reopened = open_topic(&mut restarted, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &reopened else { unreachable!() };
    let crate::IncomeRecord::Business(target) = &documents.documents[1].document else {
        unreachable!()
    };
    let source = target.assets[0].gds().unwrap().source.as_ref().unwrap();
    assert_eq!(source.allowed_class_life, vec![0]);
    assert!(source.class_life.is_none());
    accepted(&send(&mut restarted, reopened));
    assert_eq!(restarted.draft, saved);
}

#[test]
fn same_edit_class_election_rebuilds_sibling_questions_and_local_cohort_choices() {
    let mut session = restart(ads_question_correction_draft(false, None));
    let mut submitted = open_topic(&mut session, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut submitted else { unreachable!() };
    for (index, record) in documents.documents.iter_mut().enumerate() {
        let crate::IncomeRecord::Business(business) = &mut record.document else { unreachable!() };
        let asset = &mut business.assets[0];
        asset.placed = 20261201;
        asset.cost += 12_345 * (index as i64 + 1);
        let facts = asset.gds_mut().unwrap();
        facts.class = AssetClass::Seven;
        let source = facts.source.as_mut().unwrap();
        assert!(source.allowed_class_life.is_empty());
        if index == 0 {
            source.election = Some(crate::AdsElection { tax_year: 2026 });
            source.class_life =
                Some(crate::AdsClassLife { table_asset_class: "00.11".into(), months: 120 }.into());
            // The edited source's own local calculation has already offered this
            // record; only its sibling depends on the newly entered election.
            source.allowed_class_life = vec![0];
        }
    }
    let accepted_draft = session.draft.clone();
    let accepted_dirty = session.dirty;
    let before = submitted.clone();
    let Reply::Edit(mut corrected @ P::SaveIncomes { .. }) = send(&mut session, submitted) else {
        panic!("same-edit class election must refresh the sibling's source questions")
    };
    assert_eq!(session.draft, accepted_draft);
    assert_eq!(session.dirty, accepted_dirty);
    assert_eq!(ads_editor_answers(corrected.clone()), ads_editor_answers(before));
    let P::SaveIncomes { documents, warnings, cookie, revision, .. } = &mut corrected else {
        unreachable!()
    };
    assert_eq!(cookie, &accepted_draft.cookie);
    assert_eq!(*revision, accepted_draft.revision);
    assert_eq!(warnings.len(), 1);
    assert!(status_text(&warnings[0].message).starts_with("Record 2, property 1:"));
    let crate::IncomeRecord::Business(target) = &mut documents.documents[1].document else {
        unreachable!()
    };
    let facts = target.assets[0].gds_mut().unwrap();
    assert_eq!(facts.source.as_ref().unwrap().allowed_class_life, vec![0]);
    let context = facts.macrs_question_context.clone().unwrap();
    assert_eq!(context.placed_year, 2026);
    assert!(
        context.elections.iter().any(
            |row| row.class == crate::GdsEquipmentClass::Seven && row.election.tax_year == 2026
        )
    );
    for (class, expected) in [(AssetClass::Five, vec![]), (AssetClass::Seven, vec![0])] {
        let reply = send(
            &mut session,
            P::AssetMacrsClassLifeOptions {
                user: TEST_USER,
                class,
                source: facts.source.as_ref().unwrap().clone().into(),
                original_system: facts.source.as_ref().unwrap().original_system,
                context: Some(context.clone()),
            },
        );
        let Reply::Response(P::AssetMacrsClassLifeChoices { class_life }) = reply else {
            panic!("expected source choices")
        };
        assert_eq!(class_life, expected, "local class edits select the actual election cohort");
    }
    assert_eq!(session.draft, accepted_draft);
    assert_eq!(session.dirty, accepted_dirty);
    facts.source.as_mut().unwrap().class_life =
        Some(crate::AdsClassLife { table_asset_class: "00.11".into(), months: 120 }.into());
    let expected = ads_editor_answers(corrected.clone());
    let reply = send(&mut session, corrected);
    accepted(&reply);
    assert!(
        session.draft.revision > accepted_draft.revision,
        "actual supplied records must save without another correction"
    );
    let saved = session.draft.clone();
    let mut restarted = restart(saved.clone());
    let reopened = open_topic(&mut restarted, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents: mut expected, .. } = expected else { unreachable!() };
    for item in &mut expected.documents {
        let crate::IncomeRecord::Business(page) = &mut item.document else { unreachable!() };
        let entry = saved.income.iter().find(|entry| entry.id == item.id).unwrap();
        let FederalIncome::Business { data } = &entry.income else { unreachable!() };
        crate::tests::support::client::bind_saved_property_ids(
            &mut page.assets,
            &mut page.vehicles,
            &data.assets,
        );
    }
    let P::SaveIncomes { documents: actual, .. } = ads_editor_answers(reopened.clone()) else {
        unreachable!()
    };
    assert_eq!(actual, expected, "all document answers survive Save, reopen and restart");
    accepted(&send(&mut restarted, reopened));
    assert_eq!(restarted.draft, saved);
}

#[test]
fn interest_election_is_activity_scoped_and_requires_ads_without_voluntary_bonus() {
    use crate::tests::support::builders::ads::interest_activity_draft;
    use crate::{InterestLimitElection, InterestLimitElectionKind as Kind};
    for (family, kind) in [
        ("Business", Kind::RealProperty),
        ("Farm", Kind::Farming),
        ("Rental", Kind::RealProperty),
        ("FarmRental", Kind::Farming),
        ("FarmRental", Kind::RealProperty),
    ] {
        let draft = interest_activity_draft(
            family,
            Some(InterestLimitElection { kind, effective_year: 2026 }),
        );
        let original = draft.clone();
        let entry = draft.income.last().unwrap();
        let asset = &crate::nonmacrs::recovery_assets(&entry.income).unwrap()[0];
        let view = crate::macrs_source::MacrsElectionView::new(&draft);
        let invocation = view.context(entry.owner.owned(), Some(entry.id), asset);
        let affected = !(family == "FarmRental" && kind == Kind::RealProperty);
        assert_eq!(invocation.required.is_some(), affected, "{family} {kind:?}");
        assert!(
            view.context(entry.owner.owned(), None, asset).required.is_none(),
            "employees do not borrow an activity election"
        );
        assert!(view.context(Owner::Spouse, Some(entry.id), asset).required.is_none());
        assert!(view.context(entry.owner.owned(), Some(entry.id + 100), asset).required.is_none());
        if affected {
            assert_eq!(asset.source_bonus_amount_in_context(invocation), 0);
            assert!(asset.source_macrs_amount_in_context(false, invocation).unwrap() > 0);
        }
        if family == "Farm" {
            let result = supported(&draft);
            assert_eq!(line_amount(&result, "business_2_asset_0_bonus"), 0);
            assert_eq!(line_amount(&result, "business_2_asset_0_macrs"), 25_000);
            let control = interest_activity_draft(family, None);
            assert_eq!(line_amount(&supported(&control), "business_2_asset_0_bonus"), 1_000_000);
        }
        assert_eq!(draft, original);
    }
}

#[test]
fn interest_election_uses_tax_year_and_preserves_unavailable_conversion_history() {
    use crate::tests::support::builders::ads::interest_activity_draft;
    use crate::{InterestLimitElection, InterestLimitElectionKind as Kind};
    let mut draft = interest_activity_draft(
        "Farm",
        Some(InterestLimitElection { kind: Kind::Farming, effective_year: 2026 }),
    );
    let entry = draft.income.last_mut().unwrap();
    let FederalIncome::Business { data } = &mut entry.income else { unreachable!() };
    let asset = &mut data.assets[0];
    asset.placed = 20250601;
    asset.acquired = asset.placed;
    asset.history = EquipmentHistory::Continuing {
        prior_depreciation: 50_000,
        initial_section179: 0,
        carryforward: 0,
        mid_quarter: false,
        amt: EquipmentAmtHistory::Same,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    let source = asset.macrs_source.as_mut().unwrap();
    source.election = None;
    source.original_system = Some(crate::OriginalMacrsSystem::Gds);
    let original = draft.clone();
    let entry = draft.income.last().unwrap();
    let asset = &crate::nonmacrs::recovery_assets(&entry.income).unwrap()[0];
    let view = crate::macrs_source::MacrsElectionView::new(&draft);
    let invocation = view.context(entry.owner.owned(), Some(entry.id), asset);
    assert!(
        invocation.required.is_some(),
        "election applies to property held in the election year"
    );
    assert_eq!(
        asset.macrs_source_issue_in_context(invocation),
        Some("ASSET_ADS_ORIGINAL_HISTORY_UNAVAILABLE")
    );
    assert!(asset.source_macrs_amount_in_context(false, invocation).is_none());
    assert_eq!(asset.source_bonus_amount_in_context(invocation), 0);
    assert_eq!(draft, original, "actual GDS history must not be backdated or rewritten");
    for year in [2017, 2027] {
        let mut invalid = draft.clone();
        let FederalIncome::Business { data } = &mut invalid.income.last_mut().unwrap().income
        else {
            unreachable!()
        };
        data.interest_limit_election.as_mut().unwrap().effective_year = year;
        assert_eq!(
            crate::interest_limit::issue(&invalid.income.last().unwrap().income, 2026).unwrap().key,
            "INTEREST_LIMIT_ELECTION_SOURCE"
        );
    }
}

#[test]
fn activity_interest_election_reopens_and_retains_accepted_record_after_property_removal() {
    use crate::tests::support::builders::ads::interest_activity_draft;
    use crate::{InterestLimitElection, InterestLimitElectionKind as Kind};
    for (family, kind, topic) in [
        ("Business", Kind::RealProperty, TaxSection::SelfEmployment),
        ("Farm", Kind::Farming, TaxSection::Farming),
        ("Rental", Kind::RealProperty, TaxSection::Rentals),
        ("FarmRental", Kind::RealProperty, TaxSection::Farming),
    ] {
        let election = InterestLimitElection { kind, effective_year: 2025 };
        let mut session = restart(interest_activity_draft(family, Some(election.clone())));
        let mut request = open_topic(&mut session, topic);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        let item = &mut documents.documents[0];
        let id = item.id;
        let allowed = match &mut item.document {
            crate::IncomeRecord::Business(p) => {
                p.assets.clear();
                &p.allowed_interest_limit_election
            }
            crate::IncomeRecord::Farm(p) => {
                p.assets.clear();
                &p.allowed_interest_limit_election
            }
            crate::IncomeRecord::Rental(p) => {
                p.assets.clear();
                p.building.land = p.building.cost;
                &p.allowed_interest_limit_election
            }
            crate::IncomeRecord::FarmRental(p) => {
                p.assets.clear();
                &p.allowed_interest_limit_election
            }
            _ => unreachable!(),
        };
        assert_eq!(allowed, &vec![0]);
        assert!(!crate::interest_limit::needed(&item.document));
        let before = session.draft.clone();
        assert!(
            before
                .income
                .iter()
                .find(|entry| entry.id == id)
                .is_some_and(|entry| crate::interest_limit::source(&entry.income).is_some())
        );
        accepted(&send(&mut session, request));
        let saved = session.draft.clone();
        let entry = saved.income.iter().find(|entry| entry.id == id).unwrap();
        assert_eq!(crate::interest_limit::source(&entry.income), Some(&election));
        let mut restarted = restart(saved.clone());
        let reopened = open_topic(&mut restarted, topic);
        let P::SaveIncomes { documents, .. } = &reopened else { unreachable!() };
        let allowed = match &documents.documents[0].document {
            crate::IncomeRecord::Business(p) => &p.allowed_interest_limit_election,
            crate::IncomeRecord::Farm(p) => &p.allowed_interest_limit_election,
            crate::IncomeRecord::Rental(p) => &p.allowed_interest_limit_election,
            crate::IncomeRecord::FarmRental(p) => &p.allowed_interest_limit_election,
            _ => unreachable!(),
        };
        assert_eq!(allowed, &vec![0], "accepted election stays offered after property removal");
        accepted(&send(&mut restarted, reopened));
        assert_eq!(restarted.draft, saved);
        let mut fresh = saved.clone();
        match &mut fresh.income.iter_mut().find(|entry| entry.id == id).unwrap().income {
            FederalIncome::Business { data } => data.interest_limit_election = None,
            FederalIncome::Rental { data } => data.interest_limit_election = None,
            FederalIncome::FarmRental { data } => data.interest_limit_election = None,
            _ => unreachable!(),
        }
        let mut no_election = restart(fresh);
        let page = open_topic(&mut no_election, topic);
        let P::SaveIncomes { documents, .. } = &page else { unreachable!() };
        let allowed = match &documents.documents[0].document {
            crate::IncomeRecord::Business(p) => &p.allowed_interest_limit_election,
            crate::IncomeRecord::Farm(p) => &p.allowed_interest_limit_election,
            crate::IncomeRecord::Rental(p) => &p.allowed_interest_limit_election,
            crate::IncomeRecord::FarmRental(p) => &p.allowed_interest_limit_election,
            _ => unreachable!(),
        };
        assert!(
            allowed.is_empty(),
            "no affected property and no retained election has no question"
        );
        accepted(&send(&mut no_election, page));
    }
}

#[test]
fn added_affected_property_returns_complete_election_question_then_explicit_null_saves() {
    use crate::tests::support::builders::ads::interest_activity_draft;
    let mut session = restart(interest_activity_draft("Business", None));
    let mut initial = open_topic(&mut session, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut initial else { unreachable!() };
    let crate::IncomeRecord::Business(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    page.assets.clear();
    accepted(&send(&mut session, initial));
    let mut submitted = open_topic(&mut session, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut submitted else { unreachable!() };
    let crate::IncomeRecord::Business(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    assert!(page.allowed_interest_limit_election.is_empty());
    let mut asset = Vec::new();
    let source = interest_activity_draft("Business", None);
    let pages = reopened_businesses(source);
    asset.extend(pages[0].assets.clone());
    for a in &mut asset {
        a.id = 0;
    }
    page.assets = asset;
    page.name = "Candidate property activity".into();
    page.allowed_interest_limit_election = vec![999];
    let expected = ads_editor_answers(submitted.clone());
    let original = session.draft.clone();
    let dirty = session.dirty;
    let Reply::Edit(corrected @ P::SaveIncomes { .. }) = send(&mut session, submitted) else {
        panic!("complete correction")
    };
    assert_eq!(session.draft, original);
    assert_eq!(session.dirty, dirty);
    let P::SaveIncomes { documents, warnings, cookie, revision, .. } = &corrected else {
        unreachable!()
    };
    assert_eq!(cookie, &original.cookie);
    assert_eq!(*revision, original.revision);
    assert!(
        warnings
            .iter()
            .any(|w| status_message(&w.message).key == "INTEREST_LIMIT_ELECTION_QUESTIONS_UPDATED")
    );
    let crate::IncomeRecord::Business(page) = &documents.documents[0].document else {
        unreachable!()
    };
    assert_eq!(page.allowed_interest_limit_election, vec![0]);
    assert!(page.interest_limit_election.is_none(), "deliberate null is the no-election answer");
    let expected_cost = page.assets[0].cost;
    let expected_source: Option<crate::MacrsRecoverySource> =
        page.assets[0].gds().unwrap().source.clone().map(Into::into);
    assert_eq!(ads_editor_answers(corrected.clone()), expected);
    accepted(&send(&mut session, corrected));
    let saved = session.draft.clone();
    assert!(saved.revision > original.revision, "the answered correction must commit");
    let entry = saved.income.last().unwrap();
    assert_eq!(entry.label, "Candidate property activity");
    let FederalIncome::Business { data } = &entry.income else { unreachable!() };
    assert_eq!(data.assets.len(), 1);
    assert_eq!(data.assets[0].cost, expected_cost);
    assert_eq!(data.assets[0].macrs_source, expected_source);
    assert!(data.assets[0].id > 0);
    assert!(crate::interest_limit::source(&saved.income.last().unwrap().income).is_none());
    let mut restarted = restart(saved.clone());
    let reopened = open_topic(&mut restarted, TaxSection::SelfEmployment);
    accepted(&send(&mut restarted, reopened));
    assert_eq!(restarted.draft, saved);
}

#[test]
fn positive_rental_election_saves_but_unresolved_building_ads_never_reports_gds() {
    use crate::tests::support::builders::ads::interest_activity_draft;
    use crate::{InterestLimitElection, InterestLimitElectionKind as Kind};
    let control = interest_activity_draft("Rental", None);
    assert!(supported(&control).tax.is_some());
    let mut session = restart(control);
    let mut page = open_topic(&mut session, TaxSection::Rentals);
    let P::SaveIncomes { documents, .. } = &mut page else { unreachable!() };
    let crate::IncomeRecord::Rental(rental) = &mut documents.documents[0].document else {
        unreachable!()
    };
    let election = InterestLimitElection { kind: Kind::RealProperty, effective_year: 2025 };
    rental.interest_limit_election = Some(election.clone().into());
    let building_before = rental.building.clone();
    accepted(&send(&mut session, page));
    let saved = session.draft.clone();
    let result = estimate(&saved);
    assert!(result.tax.is_none());
    assert_eq!(
        result
            .issues
            .iter()
            .filter(|i| i.display.key == "RENTAL_INTEREST_ELECTION_ADS_HISTORY_UNAVAILABLE")
            .count(),
        1
    );
    assert!(
        !result
            .lines
            .iter()
            .any(|line| line.id.starts_with("rental_2_") && line.id.contains("depreciation"))
    );
    assert_eq!(session.draft, saved);
    let mut restarted = restart(saved.clone());
    let reopened = open_topic(&mut restarted, TaxSection::Rentals);
    let P::SaveIncomes { documents, .. } = &reopened else { unreachable!() };
    let crate::IncomeRecord::Rental(rental) = &documents.documents[0].document else {
        unreachable!()
    };
    assert_eq!(
        rental.interest_limit_election.clone().map(InterestLimitElection::from),
        Some(election)
    );
    assert_eq!(rental.building, building_before);
    accepted(&send(&mut restarted, reopened));
    assert_eq!(restarted.draft, saved);
    let mut foreign =
        crate::tests::support::builders::foreign_rental(20260101, RentalPropertyType::SingleFamily);
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Rental { data: foreign.clone() }));
    let before = supported(&draft);
    foreign.interest_limit_election =
        Some(InterestLimitElection { kind: Kind::RealProperty, effective_year: 2025 });
    assert!(crate::interest_limit::rental_building_issue(&foreign).is_none());
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.interest_limit_election = foreign.interest_limit_election;
    let after = supported(&draft);
    assert_eq!(after.tax, before.tax, "independently mandatory foreign ADS is unchanged");
}
