use super::*;
use crate::kernel::{ratio::exact, source_fact::SourceFact};
use crate::tests::cross::interview::{
    assert_rentals_round_trip, open_topic, rental_page, save_rentals, stored_rentals,
};

#[test]
fn foreign_real_property_uses_original_required_ads_and_current_form_rows() {
    for (placed, kind, deduction, form_line) in [
        (20171231, RentalPropertyType::SingleFamily, 900_000, "17"),
        (20180101, RentalPropertyType::SingleFamily, 1_200_000, "17"),
        (20200101, RentalPropertyType::Commercial, 900_000, "17"),
        (20260201, RentalPropertyType::SingleFamily, 1_050_000, "20c"),
        (20260201, RentalPropertyType::Commercial, 787_500, "20d"),
    ] {
        let mut draft = complete(FederalStatus::Single);
        let mut source = foreign_rental(placed, kind);
        source.qbi_business = RentalBusiness::Yes { material_participation: true };
        add(&mut draft, income(FederalIncome::Rental { data: source }));
        let id = draft.income.last().unwrap().id;
        for alternative in [false, true] {
            let view = rental_source_view(&draft, 60_000, alternative);
            assert!(view.inputs_complete);
            assert_eq!(
                view.foreign_sources[&id].depreciation_cents,
                SourceFact::Reported(deduction)
            );
            assert_eq!(view.income_by_id[&id], 42_000 - deduction / 100);
            assert!(view.activities.is_empty(), "foreign real property is outside section199A");
        }
        if placed / 10000 == 2026 {
            let result = supported(&draft);
            let line = result
                .lines
                .iter()
                .find(|row| row.id == alloc::format!("rental_{id}_building_0_depreciation"))
                .unwrap();
            assert_eq!(line.form_line, Some(FormLine::new(TaxForm::Form4562, form_line)));
        }
    }
}

#[test]
fn foreign_source_retains_actual_expense_classes_and_native_annual_endpoint() {
    let mut draft = complete(FederalStatus::Single);
    let mut source = foreign_rental(20180101, RentalPropertyType::SingleFamily);
    source.rent += 49;
    source.expenses.extend(vec![
        RentalExpense {
            category: RentalExpenseCategory::Utilities,
            amount: 1,
            treatment: BusinessExpenseKind::Meals,
        };
        2
    ]);
    source.expenses.push(RentalExpense {
        category: RentalExpenseCategory::MortgageInterest,
        amount: 200_000,
        treatment: BusinessExpenseKind::Ordinary,
    });
    add(&mut draft, income(FederalIncome::Rental { data: source }));
    let id = draft.income.last().unwrap().id;
    for alternative in [false, true] {
        let view = rental_source_view(&draft, 60_000, alternative);
        let source = &view.foreign_sources[&id];
        assert_eq!(
            (source.source, source.owner, source.alternative, source.country.as_str()),
            (id, Owner::Taxpayer, alternative, "FR")
        );
        assert_eq!(source.gross_rents_cents, 4_800_049);
        assert_eq!(source.direct_expenses_cents, SourceFact::Reported(exact::whole(600_001)));
        assert_eq!(source.interest_cents, SourceFact::Reported(exact::whole(200_000)));
        assert_eq!(source.current_before_risk_cents, SourceFact::Reported(exact::whole(2_800_048)));
        assert_eq!(source.prior_at_risk_allowed_cents, SourceFact::Reported(0));
        assert_eq!(source.operating, SourceFact::Reported(view.operating_sources[&id].clone()));
        assert_eq!(source.filed_ordinary_dollars, SourceFact::Reported(28_000));
        assert_eq!(source.pal_current_net_dollars, SourceFact::Reported(28_000));
        assert_eq!(source.pal_prior_allowed_dollars, SourceFact::Reported(0));
    }
}

#[test]
fn foreign_current_prior_pal_release_is_independent_and_unknown_stays_unknown() {
    for (rent, expense, magi, current, prior_allowed) in [
        (10_000, 0, 200_000, SourceFact::Reported(10_000), SourceFact::Reported(10_000)),
        (0, 20_000, 125_000, SourceFact::Unavailable, SourceFact::Unavailable),
    ] {
        let mut draft = complete(FederalStatus::Single);
        let mut source = foreign_rental(20200101, RentalPropertyType::Land);
        source.buildings[0].land = source.buildings[0].cost;
        source.rent = rent * 100;
        source.expenses[0].amount = expense * 100;
        source.history =
            RentalHistory { regular: 2_000_000, amt: 3_000_000, active_in_loss_year: true };
        add(&mut draft, income(FederalIncome::Rental { data: source }));
        let id = draft.income.last().unwrap().id;
        for (alternative, prior) in [(false, 2_000_000), (true, 3_000_000)] {
            let view = rental_source_view(&draft, magi, alternative);
            let source = &view.foreign_sources[&id];
            assert_eq!(source.pal_current_net_dollars, current);
            assert_eq!(source.pal_prior_allowed_dollars, prior_allowed);
            assert_eq!(view.operating_sources[&id].prior_passive_cents, prior);
            let SourceFact::Reported(packet) = &source.post_pal else {
                panic!("actual PAL packet")
            };
            assert_eq!(
                packet,
                view.post_pal
                    .as_ref()
                    .unwrap()
                    .sources
                    .iter()
                    .find(|packet| packet.income == id)
                    .unwrap()
            );
            assert_eq!(
                source.filed_ordinary_dollars,
                SourceFact::Reported(if expense == 0 { 0 } else { -12_500 })
            );
        }
    }
}

#[test]
fn foreign_missing_asset_ads_preserves_source_facts_and_unrelated_operating_source() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = rental_page(4_800_000);
    page.location = crate::RentalLocation::Foreign { country: "FR".into() };
    let mut asset = bonus_equipment(1000, 100);
    asset.history = EquipmentHistory::New {
        section179: Section179::Ineligible,
        bonus: BonusElection::Apply,
        tables: false,
    };
    page.assets.push(asset.into());
    accepted(&save_rentals(&mut session, vec![page.clone(), rental_page(1_200_000)]));
    let sources = stored_rentals(&session);
    let view = rental_source_view(&session.draft, 60_000, false);
    let foreign = &view.foreign_sources[&sources[0].0];
    assert_eq!(foreign.gross_rents_cents, 4_800_000);
    assert_eq!(foreign.boundary, Some("RENTAL_FOREIGN_ASSET_ADS_UNAVAILABLE"));
    assert_eq!(foreign.depreciation_cents, SourceFact::Unavailable);
    assert!(view.operating_sources.contains_key(&sources[1].0));
    assert_eq!(sources[0].2.assets.len(), 1);
    assert!(sources[0].2.assets[0].id > 0);
    let CanonicalAssetPage::Gds { facts } = &mut page.assets[0] else { unreachable!() };
    facts.financial.id = sources[0].2.assets[0].id;
    assert_rentals_round_trip(&mut session, &[page, rental_page(1_200_000)]);
}

#[test]
fn foreign_country_native_save_reopen_restart_correction_and_atomic_rejection() {
    use crate::tests::support::client::{draft_from_native_records, native_draft_records};
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = rental_page(4_800_000);
    page.location = crate::RentalLocation::Foreign { country: "FR".into() };
    accepted(&save_rentals(&mut session, vec![page.clone()]));
    assert_rentals_round_trip(&mut session, core::slice::from_ref(&page));
    let saved = session.draft.clone();
    let id = stored_rentals(&session)[0].0;
    let records = native_draft_records(&saved);
    session = restart(draft_from_native_records(saved.tax_year, &records));
    assert_eq!(session.draft, saved);
    let mut change = open_topic(&mut session, TaxSection::Rentals);
    let P::SaveIncomes { documents, .. } = &mut change else { unreachable!() };
    let crate::IncomeRecord::Rental(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    page.location = crate::RentalLocation::Foreign { country: "JA".into() };
    accepted(&send(&mut session, change));
    assert_eq!(stored_rentals(&session)[0].0, id);
    assert_eq!(
        rental_source_view(&session.draft, 60_000, false).foreign_sources[&id].country,
        "JA"
    );
    let saved = session.draft.clone();
    let mut invalid = open_topic(&mut session, TaxSection::Rentals);
    let P::SaveIncomes { documents, .. } = &mut invalid else { unreachable!() };
    let crate::IncomeRecord::Rental(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    page.location = crate::RentalLocation::Foreign { country: "US".into() };
    let reply = send(&mut session, invalid);
    let Reply::Response(P::Problem { field, .. }) = &reply else { panic!("located country error") };
    assert_eq!(field.as_deref(), Some("[documents/documents/0/document/location/Foreign/country]"));
    rejected_with(reply, "RENTAL_FOREIGN_COUNTRY_INVALID");
    assert_eq!(session.draft, saved);
    assert_eq!(restart(saved.clone()).draft, saved);
}

#[test]
fn foreign_location_removes_domestic_historical_choices_on_current_rows() {
    let input = RentalRecoveryInputs {
        location: crate::RentalLocation::Foreign { country: "FR".into() },
        kind: RentalPropertyType::Commercial,
        building: RentalRecoveryDate { in_service: 19930701 },
        more: RentalRecoveryMoreDates {
            improvements: vec![RentalRecoveryDate { in_service: 19930601 }],
        },
    };
    let choices = crate::rental_recovery::choices(&input);
    assert!(choices.building.allowed_commercial_1993_transition.is_empty());
    assert!(choices.more.improvements[0].allowed_commercial_1993_transition.is_empty());
    let mut page = rental_page(0);
    page.location = input.location;
    page.kind = input.kind;
    page.building.in_service = 19930701;
    page.building.commercial_1993_transition = Some(Commercial1993Transition::Ordinary39Year);
    let issue = crate::rental_recovery::page_issue(&page).unwrap();
    assert_eq!(issue.key, "RENTAL_1993_TRANSITION_NOT_APPLICABLE");
}

#[test]
fn foreign_risk_history_retains_each_systems_actual_source_and_split_availability() {
    let mut draft = complete(FederalStatus::Single);
    let mut source = foreign_rental(20200101, RentalPropertyType::Land);
    source.rent = 0;
    source.expenses[0].amount = 249;
    source.buildings[0].land = source.buildings[0].cost;
    source.at_risk = simplified_rental_risk(149, 0, 0, [50, 75, 0, 0]);
    add(&mut draft, income(FederalIncome::Rental { data: source }));
    let id = draft.income.last().unwrap().id;
    for (alternative, prior, before) in [(false, 50, -299), (true, 75, -324)] {
        let view = rental_source_view(&draft, 200_000, alternative);
        let source = &view.foreign_sources[&id];
        assert_eq!(source.current_before_risk_cents, SourceFact::Reported(exact::whole(-249)));
        assert_eq!(source.prior_at_risk_allowed_cents, SourceFact::Unavailable);
        let SourceFact::Reported(operating) = &source.operating else {
            panic!("same risk invocation")
        };
        assert_eq!(
            (
                operating.before_at_risk_cents,
                operating.after_at_risk_cents,
                operating.prior_at_risk_cents
            ),
            (before, -149, prior)
        );
    }
}

#[test]
fn foreign_pre_macrs_history_is_retained_as_an_actual_source_boundary() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = rental_page(4_800_000);
    page.location = crate::RentalLocation::Foreign { country: "FR".into() };
    page.building.in_service = 19861231;
    accepted(&save_rentals(&mut session, vec![page.clone()]));
    let sources = stored_rentals(&session);
    assert_eq!(sources[0].2.buildings[0].in_service, 19861231);
    assert!(crate::extended_rules::rental_source_valid(&sources[0].2));
    let mut domestic = sources[0].2.clone();
    domestic.location = crate::RentalLocation::UnitedStates;
    assert!(!crate::extended_rules::rental_source_valid(&domestic));
    let view = rental_source_view(&session.draft, 60_000, false);
    let source = &view.foreign_sources[&sources[0].0];
    assert_eq!(source.gross_rents_cents, 4_800_000);
    assert_eq!(source.depreciation_cents, SourceFact::Unavailable);
    assert_eq!(source.boundary, Some("RENTAL_FOREIGN_RECOVERY_HISTORY_UNAVAILABLE"));
    assert_rentals_round_trip(&mut session, &[page]);
}
