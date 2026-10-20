use crate::kernel::source_fact::SourceFact;
use crate::tests::cross::interview::{
    assert_rentals_round_trip, open_topic, rental_page, save_rentals, stored_rentals,
};
use crate::tests::support::builders::{
    rental_energy_building, rental_energy_claim, rental_energy_return,
};
use crate::tests::*;

#[test]
fn rental_energy_statutory_prior_cap_basis_and_original_qbi_ubia() {
    let mut d = rental_energy_return();
    let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.energy_buildings[0].deductions[2].amount = SourceCents::Reported { amount: 500_000 };
    // IRC179D(b)(1), D02 2026-10-19: (0.59+0.02*(35-25))*10000-5000
    // is $2900. The Form7205 full-rate shortcut's $6900 is not this cap.
    assert_eq!(crate::rental_energy::calculate(data, d.tax_year).reductions, vec![290_000]);
    // Dec TableA7a107/100000 of $97100, exact cents $103.897 -> $104.
    assert_eq!(
        crate::extended_rules::depreciation_for_year(
            &data.buildings[0],
            false,
            &data.property_type,
            2026,
            None,
            SourceFact::Reported(290_000)
        ),
        Ok(10_390)
    );
    assert_eq!(data.buildings[0].cost, 10_000_000);
    assert_eq!(data.buildings[0].credit_reduction, 0);
    assert!(d.valid_stored());
    let id = d.income.last().unwrap().id;
    let r = estimate(&d);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_line21_income")), 14_996);
    // Treas.Reg1.199A-2(c)(3)(i):179D expensing never reduces original UBIA.
    let mut result = FederalResult::empty(0);
    let rentals = crate::extended_rules::rentals(&d, 60_000, &mut result);
    assert_eq!(rentals.activities[0].property, 100_000);
    assert_eq!(rentals.activities[0].income, 14_996);
    assert!(r.tax.is_some(), "{:?}", r.issues);
}

#[test]
fn rental_energy_fractional_area_savings_round_once_to_the_shared_cent_amount() {
    let mut d = rental_energy_return();
    let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.energy_buildings[0].square_feet = "123.45".into();
    let crate::RentalEnergyProject::Eecbp { certification: Some(cert) } =
        &mut data.buildings[0].energy.as_mut().unwrap().project
    else {
        unreachable!()
    };
    cert.savings_percent = "35.55".into();
    // RP25-32: $.59+$.02*(35.55-25)=$.801/sf;123.45*.801=$98.88345.
    // D02 2026-10-19: once to$98.88, identical deduction/basis reduction.
    let prepared = crate::rental_energy::calculate(data, d.tax_year);
    assert!(prepared.boundaries.is_empty());
    assert_eq!(prepared.reductions, vec![9888]);
    assert_eq!(data.energy_buildings[0].square_feet, "123.45");
    assert_eq!(
        crate::extended_rules::depreciation_for_year(
            &data.buildings[0],
            false,
            &data.property_type,
            u32::try_from(d.tax_year).unwrap(),
            None,
            SourceFact::Reported(prepared.reductions[0])
        ),
        Ok(10_689)
    );
    let id = d.income.last().unwrap().id;
    let r = estimate(&d);
    assert!(r.tax.is_some(), "{:?}", r.issues);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_line21_income")), 17_794);
    assert_eq!(
        r.lines
            .iter()
            .find(|line| line.id == alloc::format!("rental_{id}_energy_deduction"))
            .unwrap()
            .amount,
        9888
    );
    // The display rounds each complete form line, including totals, only
    // after exact calculation. $2205.77 rounds to $2206, although the
    // independently rounded operands $2000 + $99 + $107 total $2206.
    let exact_lines = r.lines.clone();
    let displayed = crate::draft_return(&r);
    for form in [TaxForm::ScheduleE, TaxForm::Form4562] {
        assert!(displayed.iter().filter(|row| row.form_line.as_ref()
            .is_some_and(|line| line.form == form)).all(|row| row.money
            .is_none_or(|amount| amount % 100 == 0)));
    }
    for (form, line, amount) in [
        (TaxForm::Form4562, "19j", 10_700),
        (TaxForm::Form4562, "22", 10_700),
        (TaxForm::ScheduleE, "19", 9_900),
        (TaxForm::ScheduleE, "20", 220_600),
        (TaxForm::ScheduleE, "21", 1_779_400),
        (TaxForm::ScheduleE, "24", 1_779_400),
    ] {
        assert!(displayed.iter().any(|row| row.form_line.as_ref()
            .is_some_and(|tag| tag.form == form && tag.line == line) && row.amount == amount),
            "{form:?} {line}: {displayed:?}");
    }
    assert_eq!(r.lines, exact_lines);

}

#[test]
fn rental_energy_shared_physical_cap_ownership_and_separate_buildings() {
    let mut d = rental_energy_return();
    let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.energy_buildings[0].ownership_numerator = 1;
    data.energy_buildings[0].ownership_denominator = 2;
    data.buildings[0].energy.as_mut().unwrap().qualifying_cost = 1_200_000;
    let mut component = purchased_rental_building(800_000, 0, 20261201);
    component.id = d.next_id;
    d.next_id += 1;
    let mut claim = rental_energy_claim(data.energy_buildings[0].id);
    claim.project_source = "EB-01 lighting invoice group".into();
    claim.qualifying_cost = 800_000;
    claim.system = crate::RentalEnergySystem::InteriorLighting;
    component.energy = Some(claim);
    data.buildings.push(component);
    // One $7900 whole cap, ownership share1/2, then actual costs60/40.
    assert_eq!(crate::rental_energy::calculate(data, 2026).reductions, vec![237_000, 158_000]);
    let mut separate = rental_energy_building();
    separate.id = d.next_id;
    d.next_id += 1;
    separate.building_source = "Actual separate building B".into();
    data.buildings[1].energy.as_mut().unwrap().physical_building = separate.id;
    data.energy_buildings.push(separate);
    assert_eq!(crate::rental_energy::calculate(data, 2026).reductions, vec![395_000, 790_000]);
    assert!(crate::rental_energy::graph_issue(&d).is_none());
    let original = d.clone();
    let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.buildings.reverse();
    data.energy_buildings.reverse();
    assert_eq!(crate::rental_energy::calculate(data, 2026).reductions, vec![790_000, 395_000]);
    assert!(crate::rental_energy::graph_issue(&d).is_none());
    assert_eq!(value(&estimate(&d), "agi"), value(&estimate(&original), "agi"));
}

#[test]
fn rental_energy_missing_sources_are_unavailable_and_legal_ineligibility_is_zero() {
    for boundary in ["history", "certification", "basis", "historical", "year"] {
        let mut d = rental_energy_return();
        let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else {
            unreachable!()
        };
        match boundary {
            "history" => data.energy_buildings[0].deductions[0].amount = SourceCents::Unavailable,
            "certification" => {
                data.buildings[0].energy.as_mut().unwrap().project =
                    crate::RentalEnergyProject::Eecbp { certification: None }
            }
            "basis" => {
                data.buildings[0].acquired =
                    RentalAcquisition::Converted { fair_market_value: 10_000_000 };
            }
            "historical" => {
                data.energy_buildings[0].deductions[2].amount =
                    SourceCents::Reported { amount: 100 };
                data.buildings.push(purchased_rental_building(1_000_000, 0, 20200101));
                assert_eq!(
                    crate::rental_energy::calculate(data, 2026).boundaries,
                    vec![crate::rental_energy::HISTORICAL_BASIS]
                );
            }
            "year" => {
                assert_eq!(
                    crate::rental_energy::calculate(data, 2025).boundaries,
                    vec![crate::rental_energy::YEAR]
                );
                continue;
            }
            _ => unreachable!(),
        }
        let r = estimate(&d);
        assert!(r.tax.is_none(), "{boundary}: {:?}", r.issues);
        assert!(
            r.issues.iter().any(|n| n.display.key.starts_with("ISSUE_RENTAL_ENERGY_")),
            "{boundary}: {:?}",
            r.issues
        );
    }
    let mut d = rental_energy_return();
    let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let claim = data.buildings[0].energy.as_mut().unwrap();
    claim.construction_began = 20260701;
    claim.installation_began = 20260702;
    data.energy_buildings[0].deductions.clear();
    assert_eq!(crate::rental_energy::calculate(data, 2026).reductions, vec![0]);
    assert!(crate::rental_energy::calculate(data, 2026).boundaries.is_empty());
    assert!(estimate(&d).tax.is_some());
}

#[test]
fn rental_energy_unprepared_basis_and_wrong_physical_association_never_guess() {
    let mut d = rental_energy_return();
    let wrong = d.income[0].id;
    let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    assert_eq!(
        crate::extended_rules::depreciation_for_year(
            &data.buildings[0],
            false,
            &data.property_type,
            2026,
            None,
            SourceFact::NotReported
        )
        .unwrap_err()
        .0,
        crate::rental_energy::BASIS
    );
    data.buildings[0].energy.as_mut().unwrap().physical_building = wrong;
    assert_eq!(crate::rental_energy::graph_issue(&d), Some("RENTAL_ENERGY_BUILDING_ASSOCIATION"));
    assert!(!d.valid_stored());
    // A saved physical fact is a real record even before any claim is added.
    let mut no_claim = rental_energy_return();
    let FederalIncome::Rental { data } = &mut no_claim.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.buildings[0].energy = None;
    data.energy_buildings[0].id = 0;
    assert_eq!(
        crate::rental_energy::graph_issue(&no_claim),
        Some("RENTAL_ENERGY_BUILDING_ASSOCIATION")
    );
    assert!(!no_claim.valid_stored());
}

#[test]
fn rental_energy_native_parent_save_claim_reopen_clear_and_restart_keep_identity() {
    let mut session = restart(complete(FederalStatus::Single));
    let mut page = rental_page(2_000_000);
    page.kind = RentalPropertyType::Commercial;
    page.building.cost = 10_000_000;
    page.building.land = 0;
    page.building.in_service = 20261201;
    page.expenses = crate::tests::cross::interview::no_rental_expenses();
    page.expenses.repairs = 200_000;
    page.more.energy_buildings = vec![rental_energy_building()];
    accepted(&save_rentals(&mut session, vec![page]));
    let saved = stored_rentals(&session);
    let physical = saved[0].2.energy_buildings[0].id;
    let cost = saved[0].2.buildings[0].id;
    assert!(physical > 0 && cost > 0 && physical != cost);
    let mut request = open_topic(&mut session, TaxSection::Rentals);
    let P::SaveIncomes { documents, rental_energy_buildings, .. } = &mut request else { panic!() };
    assert!(rental_energy_buildings.iter().any(|choice| choice.id == physical));
    let IncomeDocument::Rental(page) = &mut documents.documents[0].document else { panic!() };
    page.building.energy = Some((&rental_energy_claim(physical)).into());
    let accepted_before = session.draft.clone();
    let callback = estimate_documents_from_save(&request);
    let Reply::Response(P::IncomeRunningEstimate { estimate, .. }) = send(&mut session, callback)
    else {
        panic!("native current-claim callback")
    };
    assert!(!status_message(&estimate).key.starts_with("ESTIMATE_UNAVAILABLE"));
    assert_eq!(session.draft, accepted_before);
    accepted(&send(&mut session, request));
    let reopened = crate::tests::cross::interview::reopened_rentals(session.draft.clone());
    assert_eq!(reopened[0].1, Some(1_000_145));
    let current = reopened[0].0.clone();
    assert_eq!(current.building.id, cost);
    assert_eq!(current.building.energy.as_ref().unwrap().physical_building, physical);
    assert_rentals_round_trip(&mut session, &[current]);
    let before = session.draft.clone();
    for invalid in ["cost", "building", "decimal"] {
        let mut request = open_topic(&mut session, TaxSection::Rentals);
        let P::SaveIncomes { documents, .. } = &mut request else { panic!() };
        let IncomeDocument::Rental(page) = &mut documents.documents[0].document else { panic!() };
        match invalid {
            "cost" => page.building.energy.as_mut().unwrap().qualifying_cost = 10_000_001,
            "building" => page.building.energy.as_mut().unwrap().physical_building = cost,
            "decimal" => page.more.energy_buildings[0].square_feet = "1.".into(),
            _ => unreachable!(),
        }
        assert!(
            matches!(send(&mut session, request), Reply::Response(P::Problem { .. })),
            "{invalid}"
        );
        assert_eq!(session.draft, before);
    }
    let mut request = open_topic(&mut session, TaxSection::Rentals);
    let P::SaveIncomes { documents, .. } = &mut request else { panic!() };
    let IncomeDocument::Rental(page) = &mut documents.documents[0].document else { panic!() };
    page.building.energy = None;
    page.more.energy_buildings.clear();
    accepted(&send(&mut session, request));
    let cleared =
        crate::tests::cross::interview::reopened_rentals(session.draft.clone())[0].0.clone();
    assert_eq!(cleared.building.id, cost);
    assert!(cleared.building.energy.is_none());
    assert_rentals_round_trip(&mut session, &[cleared]);
}

#[test]
fn rental_form_depreciation_total_rounds_the_sum_once_after_native_save() {
    let mut session = restart(complete(FederalStatus::Single));
    let mut page = rental_page(100_000);
    page.building.cost = 1_348;
    page.building.land = 0;
    page.building.in_service = 20200101;
    accepted(&save_rentals(&mut session, vec![page.clone(), page]));
    let saved = session.draft.clone();
    let result = supported(&saved);
    let depreciation: Vec<_> = result.lines.iter().filter(|row| row.form_line.as_ref()
        .is_some_and(|line| line.form == TaxForm::ScheduleE && line.line == "18"))
        .map(|row| row.amount).collect();
    assert_eq!(depreciation, vec![49, 49]);
    let displayed = crate::draft_return(&result);
    assert!(!displayed.iter().any(|row| row.form_line.as_ref()
        .is_some_and(|line| line.form == TaxForm::ScheduleE && line.line == "18")));
    let total = displayed.iter().find(|row| row.form_line.as_ref()
        .is_some_and(|line| line.form == TaxForm::ScheduleE && line.line == "23d")).unwrap();
    // D-2026-10-19-01: 49c+49c=98c prints$1; separately rounded
    // source lines print$0. The accepted sources and exact totals stay intact.
    assert_eq!(total.money, Some(100));
    assert_eq!(line_amount(&result, "schedule_e_line23d_depreciation_depletion"), 98);
    assert_eq!(session.draft, saved);
    assert_eq!(estimate(&restart(saved).draft), result);
}
