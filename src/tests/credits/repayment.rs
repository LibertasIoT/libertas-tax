// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn beauty_tip_credit_uses_prior_credit_first_and_carries_unused_current_credit() {
    let mut draft = beauty_tip_credit_draft();
    draft.credits.as_mut().unwrap().tip_fica_carryforward = Some(tip_fica_carryforward(10_000_000));
    restaurant_business_mut(&mut draft)
        .employer_tip_fica_credit
        .as_mut()
        .unwrap()
        .used_prior_year_carryback = 300;
    let result = supported(&draft);
    let limit = value(&result, "general_business_credit_limit");
    assert!(limit > 0 && limit < 100_000);
    assert_eq!(value(&result, "general_business_credit"), limit);
    assert_eq!(value(&result, "tip_fica_current_credit"), 8);
    assert_eq!(value(&result, "tip_fica_current_credit_used"), 0);
    assert_eq!(value(&result, "tip_fica_current_credit_carryback_to_prior_year"), 3);
    assert_eq!(value(&result, "tip_fica_current_credit_carryforward_to_next_year"), 5);
    assert_eq!(supported(&restart(draft).draft), result);
}

#[test]
fn fuel_credit_requires_matching_activity_and_unclaimed_qualified_use() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { data } = cash_business(10_000) else { unreachable!() };
    let mut business = data;
    business.fuel_tax_credit = Some(FederalFuelTaxCredit {
        fuels: FederalFuelLines {
            gasoline_farm: Some(fuel_use(10_000, 500)),
            ..empty_fuel_lines()
        },
        expense_treatment: FederalFuelExpenseTreatment::TaxExcludedFromExpenses,
        records_complete: true,
    });
    add(&mut draft, income(FederalIncome::Business { data: business }));
    unsupported(&draft, "Fuel tax credit");
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let claim = data.fuel_tax_credit.as_mut().unwrap();
    claim.fuels.gasoline_off_highway = claim.fuels.gasoline_farm.take();
    claim.fuels.gasoline_off_highway.as_mut().unwrap().qualified = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let claim = data.fuel_tax_credit.as_mut().unwrap();
    claim.fuels.gasoline_off_highway.as_mut().unwrap().qualified = true;
    claim.fuels.gasoline_off_highway.as_mut().unwrap().not_claimed_elsewhere = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .gasoline_off_highway
        .as_mut()
        .unwrap()
        .thousandths_of_gallon = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
}

#[test]
fn cash_fuel_tax_recovery_requires_cash_method_and_valid_amount() {
    let FederalIncome::Business { data: mut business } = cash_business(10_000) else {
        unreachable!()
    };
    business.cash_fuel_tax_recovery = Some(cash_fuel_recovery(72_900));
    assert_eq!(business.receipts(), 1_072_900);
    assert!(
        crate::federal_rules::validate_income_amount(&FederalIncome::Business {
            data: business.clone(),
        })
        .is_ok()
    );
    if let BusinessActivity::Ordinary { accounting, .. } = &mut business.activity {
        *accounting = BusinessAccounting::Accrual;
    }
    assert!(
        crate::federal_rules::validate_income_amount(&FederalIncome::Business {
            data: business.clone(),
        })
        .is_err()
    );
    if let BusinessActivity::Ordinary { accounting, .. } = &mut business.activity {
        *accounting = BusinessAccounting::Cash;
    }
    business.cash_fuel_tax_recovery = Some(cash_fuel_recovery(0));
    assert!(
        crate::federal_rules::validate_income_amount(&FederalIncome::Business { data: business })
            .is_err()
    );
}

#[test]
fn cash_fuel_recovery_requires_timing_tax_benefit_and_single_reporting() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { data: mut business } = cash_business(10_000) else {
        unreachable!()
    };
    business.cash_fuel_tax_recovery = Some(cash_fuel_recovery(10_000));
    add(&mut draft, income(FederalIncome::Business { data: business }));
    let mut baseline = draft.clone();
    let FederalIncome::Business { data } = &mut baseline.income[1].income else { unreachable!() };
    data.cash_fuel_tax_recovery = None;
    let baseline = supported(&baseline);
    let report = supported(&draft);
    // Schedule C rises by $100; the related half-SE deduction lowers AGI by $7.
    assert_eq!(return_line(&report, "11a / 11b"), return_line(&baseline, "11a / 11b") + 9_300);
    assert_eq!(value(&report, "business_2_cash_fuel_tax_recovery"), 100);
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31"));
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    fn recovery_mut(draft: &mut FederalDraft) -> &mut FederalCashFuelRecovery {
        let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
        data.cash_fuel_tax_recovery.as_mut().unwrap()
    }
    for event in [
        FederalCashFuelRecoveryEvent::AmendedReturnCreditReceived,
        FederalCashFuelRecoveryEvent::RefundReceived,
    ] {
        recovery_mut(&mut draft).event = event;
        assert_eq!(supported(&draft).tax, report.tax);
    }
    // Not received in 2026: left out with a notice. Already in receipts: not
    // added again. No on the full benefit without the partial worksheet:
    // the whole recovery is income. A recovery that reduced NIIT
    // keeps its regular-tax part with a notice.
    recovery_mut(&mut draft).occurred_in_year = false;
    let r = supported(&draft);
    assert!(noted(&r, "WARN_FUEL_RECOVERY_LEFT_OUT"));
    assert_eq!(r.tax, baseline.tax);
    recovery_mut(&mut draft).occurred_in_year = true;
    {
        let (answer, key) = (false, "WARN_PARTNER_FUEL_PARTIAL_BENEFIT_LEFT_OUT");
        recovery_mut(&mut draft).full_tax_benefit = answer;
        let r = supported(&draft);
        assert!(noted(&r, key));
        assert_eq!(r.tax, report.tax);
    }
    recovery_mut(&mut draft).full_tax_benefit = true;
    recovery_mut(&mut draft).niit_recovery_excluded = false;
    let r = supported(&draft);
    assert!(noted(&r, "WARN_PARTNER_FUEL_NIIT_RECOVERY_LEFT_OUT"));
    assert_eq!(r.tax, report.tax);
    recovery_mut(&mut draft).niit_recovery_excluded = true;
    recovery_mut(&mut draft).not_reported_elsewhere = false;
    assert_eq!(supported(&draft).tax, baseline.tax);
    recovery_mut(&mut draft).not_reported_elsewhere = true;
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.material_participation = false;
    assert!(noted(&supported(&draft), "WARN_BUSINESS_LEFT_OUT"));
}

#[test]
fn commercial_aviation_kerosene_uses_taxed_rate_and_retained_claim_right() {
    use super::support::assert::unit50_retirement::assert_aviation_sources;
    let fuels = FederalFuelLines {
        aviation_commercial: Some(FederalCommercialAviationKerosene {
            taxed_at_244: Some(fuel_use(50_500, 15_000)),
            taxed_at_219: Some(fuel_use(20_000, 10_000)),
            no_waiver: true,
        }),
        ..empty_fuel_lines()
    };
    // Commercial aviation reduces 24.4¢ tax to 4.4¢ on 50.5 gallons,
    // and 21.9¢ tax to 4.4¢ on 20 gallons: $10.10 + $3.50.
    let mut draft = assert_aviation_sources(fuels, [("5a", 1010), ("5b", 350)], 2_700);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_commercial.as_mut().unwrap().no_waiver =
        false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let aviation =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_commercial.as_mut().unwrap();
    aviation.no_waiver = true;
    aviation.taxed_at_219.as_mut().unwrap().actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
}

#[test]
fn dc_homebuyer_credit_precedes_minimum_tax_credit_without_double_claim() {
    let mut d = complete(FederalStatus::Single);
    d.credits.as_mut().unwrap().dc_homebuyer_carry = Some(400_000);
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(minimum_tax_history()));
    let r = supported(&d);
    assert_eq!(value(&r, "dc_homebuyer_credit"), 4000);
    assert_eq!(value(&r, "minimum_tax_credit"), 1023);
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 13977);
    assert_eq!(return_line(&r, "20"), 502_300);
    assert_eq!(r.tax, Some(0));
}

#[test]
fn clean_vehicle_prior_year_magi_and_acquisition_year_gate_credit() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 16_000_000, 0);
    d.credits.as_mut().unwrap().clean_vehicles =
        Some(clean_vehicle_claim(vec![new_clean_vehicle()]));
    unsupported(&d, "Clean vehicle");

    d.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap().prior_year =
        Some(FederalCleanVehiclePrior { status: FederalStatus::Single, magi: 14_000_000 });
    let prior_qualifies = supported(&d);
    assert_eq!(value(&prior_qualifies, "new_clean_vehicle_credit"), 7500);
    d.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap().prior_year =
        Some(FederalCleanVehiclePrior { status: FederalStatus::Single, magi: 15_000_100 });
    let high_both_years = supported(&d);
    assert_eq!(value(&high_both_years, "new_clean_vehicle_credit"), 0);

    let claim = d.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap();
    let FederalCleanVehicle::New { details, .. } = &mut claim.vehicles[0] else { unreachable!() };
    details.acquired = 20251001;
    unsupported(&d, "Clean vehicle");
}

#[test]
fn clean_vehicle_dealer_transfer_reconciles_without_claiming_credit_twice() {
    let mut d = complete(FederalStatus::Single);
    let mut vehicle = new_clean_vehicle();
    let FederalCleanVehicle::New { details, .. } = &mut vehicle else { unreachable!() };
    details.dealer_transfer = Some(750_000);
    d.credits.as_mut().unwrap().clean_vehicles = Some(clean_vehicle_claim(vec![vehicle]));
    let eligible = supported(&d);
    assert_eq!(value(&eligible, "new_clean_vehicle_credit"), 0);
    assert_eq!(value(&eligible, "new_clean_vehicle_transfer_repayment"), 0);
    assert_eq!(return_line(&eligible, "17"), 0);
    assert_eq!(eligible.tax, Some(502_300));
    assert_eq!(supported(&restart(d.clone()).draft).tax, eligible.tax);

    let claim = d.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap();
    let FederalCleanVehicle::New { details, .. } = &mut claim.vehicles[0] else { unreachable!() };
    details.disposition = crate::CleanVehicleDisposition::ResoldWithin30Days;
    let resold = supported(&d);
    assert_eq!(value(&resold, "new_clean_vehicle_transfer_repayment"), 7500);
    assert_eq!(return_line(&resold, "17"), 750_000);
    assert_eq!(return_line(&resold, "18"), 1_252_300);
    assert_eq!(resold.tax, Some(1_252_300));

    d.income[0] = wage(1, Owner::Taxpayer, 16_000_000, 0);
    let resold_without_prior_magi = supported(&d);
    assert_eq!(value(&resold_without_prior_magi, "new_clean_vehicle_transfer_repayment"), 7500);

    let claim = d.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap();
    let FederalCleanVehicle::New { details, .. } = &mut claim.vehicles[0] else { unreachable!() };
    details.disposition = crate::CleanVehicleDisposition::Kept;
    d.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap().prior_year =
        Some(FederalCleanVehiclePrior { status: FederalStatus::Single, magi: 16_000_000 });
    let income_ineligible = supported(&d);
    assert_eq!(value(&income_ineligible, "new_clean_vehicle_transfer_repayment"), 7500);
    assert_eq!(return_line(&income_ineligible, "17"), 750_000);
}

#[test]
fn clean_vehicle_rejects_duplicate_vin_without_consuming_ineligible_used_claim() {
    let mut d = complete(FederalStatus::Single);
    let mut ineligible = used_clean_vehicle();
    let FederalCleanVehicle::PreviouslyOwned { sales_price, .. } = &mut ineligible else {
        unreachable!()
    };
    *sales_price = 2_600_000;
    let mut eligible = used_clean_vehicle();
    let FederalCleanVehicle::PreviouslyOwned { details, .. } = &mut eligible else {
        unreachable!()
    };
    details.vin = "1HGBH41JXMN109188".into();
    d.credits.as_mut().unwrap().clean_vehicles =
        Some(clean_vehicle_claim(vec![ineligible, eligible]));
    let mixed = estimate(&d);
    assert!(mixed.issues.iter().any(|issue| issue.section.starts_with("Clean vehicle")));
    let mut direct_result = supported(&complete(FederalStatus::Single));
    let selected = crate::clean_vehicle::calculate(
        &d,
        d.credits.as_ref().unwrap().clean_vehicles.as_ref(),
        d.people.as_ref().unwrap(),
        FederalStatus::Single,
        60_000,
        &mut direct_result,
    );
    assert_eq!(selected.used, 4000);

    let claim = d.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap();
    let FederalCleanVehicle::PreviouslyOwned { details, .. } = &mut claim.vehicles[1] else {
        unreachable!()
    };
    details.vin = "1hgbh41jxmn109187".into();
    assert!(!d.valid_stored());
}

#[test]
fn clean_vehicle_magi_uses_current_territory_addbacks_and_prior_status() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 14_950_000, 0);
    let mut claim = clean_vehicle_claim(vec![new_clean_vehicle()]);
    claim.current_territory_addbacks = 100_000;
    d.credits.as_mut().unwrap().clean_vehicles = Some(claim);
    unsupported(&d, "Clean vehicle");
    let claim = d.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap();
    claim.prior_year =
        Some(FederalCleanVehiclePrior { status: FederalStatus::Head, magi: 22_500_000 });
    let eligible = supported(&d);
    assert_eq!(value(&eligible, "clean_vehicle_modified_agi_current_year"), 150_500);
    assert_eq!(value(&eligible, "new_clean_vehicle_credit"), 7500);
    d.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap().prior_year =
        Some(FederalCleanVehiclePrior { status: FederalStatus::Single, magi: 22_500_000 });
    let ineligible = supported(&d);
    assert_eq!(value(&ineligible, "new_clean_vehicle_credit"), 0);
}

#[test]
fn clean_vehicle_thirty_day_dispositions_distinguish_return_from_resale() {
    // 2026-10-18: 26 CFR 1.30D-4(e)(1), 1.25E-2(c)(1).
    // A return nullifies a transfer even when MAGI otherwise requires repayment.
    for used in [false, true] {
        for transfer in [false, true] {
            for disposition in [
                crate::CleanVehicleDisposition::Kept,
                crate::CleanVehicleDisposition::ReturnedWithin30Days,
                crate::CleanVehicleDisposition::ResoldWithin30Days,
            ] {
                let mut d = complete(FederalStatus::Single);
                // Enough personal income-tax capacity for the entire credit, below either MAGI limit.
                d.income[0] =
                    wage(1, Owner::Taxpayer, if used { 7_000_000 } else { 10_000_000 }, 0);
                let mut vehicle = if used { used_clean_vehicle() } else { new_clean_vehicle() };
                let details = match &mut vehicle {
                    FederalCleanVehicle::New { details, .. }
                    | FederalCleanVehicle::PreviouslyOwned { details, .. } => details,
                };
                details.disposition = disposition;
                let credit = if used { 4000 } else { 7500 };
                details.dealer_transfer = transfer.then_some(credit * 100);
                d.credits.as_mut().unwrap().clean_vehicles =
                    Some(clean_vehicle_claim(vec![vehicle]));
                let r = supported(&d);
                let kept = disposition == crate::CleanVehicleDisposition::Kept;
                let prefix = if used { "previously_owned" } else { "new" };
                assert_eq!(
                    value(&r, &alloc::format!("{prefix}_clean_vehicle_credit")),
                    if kept && !transfer { credit } else { 0 }
                );
                assert_eq!(
                    value(&r, &alloc::format!("{prefix}_clean_vehicle_transfer_repayment")),
                    if transfer && disposition == crate::CleanVehicleDisposition::ResoldWithin30Days
                    {
                        credit
                    } else {
                        0
                    }
                );
                let returned = disposition == crate::CleanVehicleDisposition::ReturnedWithin30Days;
                assert_eq!(
                    noted(&r, "NOTE_CLEAN_VEHICLE_RETURNED_DEALER_RECOVERS_TRANSFER"),
                    returned
                );
                if returned && transfer {
                    d.income[0] = wage(1, Owner::Taxpayer, 16_000_000, 0);
                    let high_magi = supported(&d);
                    assert_eq!(return_line(&high_magi, "17"), 0);
                }
            }
        }
    }
}
