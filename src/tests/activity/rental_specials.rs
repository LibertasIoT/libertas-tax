// B10-3 fixtures A–H, checked against Pub. 527/925, IRC 613/613A and Form 6198.
use super::*;
pub(in crate::tests) fn entry(d: &mut FederalDraft, v: FederalIncome) -> i64 {
    add(d, income(v));
    d.income.last().unwrap().id
}
pub(in crate::tests) fn put(d: &mut FederalDraft, v: FederalRental) -> i64 {
    entry(d, FederalIncome::Rental { data: v })
}
pub(in crate::tests) fn wages(amount: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, amount * 100, 0);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    d
}
pub(in crate::tests) fn expense(category: RentalExpenseCategory, amount: i64) -> RentalExpense {
    RentalExpense { category, amount: amount * 100, treatment: BusinessExpenseKind::Ordinary }
}
pub(in crate::tests) fn amount(r: &FederalResult, id: i64, suffix: &str) -> i64 {
    crate::kernel::ratio::dollars(line_amount(r, &alloc::format!("rental_{id}_{suffix}")))
}

fn niit(d: &FederalDraft) -> i64 {
    let mut r = FederalResult::empty(0);
    crate::extended_rules::rentals(d, 0, &mut r).rental_niit_adjustment
}
#[test]
fn fixture_a_short_term_participation_and_services() {
    let mut d = wages(60000);
    let mut v = rental(48000, 30000);
    v.property_type = RentalPropertyType::VacationShortTerm {
        stays: ShortTermStays::SevenDaysOrLess { material_participation: true },
    };
    v.qbi_business = RentalBusiness::Yes { material_participation: true };
    let id = put(&mut d, v.clone());
    let r = supported(&d);
    assert_eq!(amount(&r, id, "depreciation"), 7051);
    assert_eq!(amount(&r, id, "line21_income"), 10949);
    assert_eq!(value(&r, "agi"), 70949);
    assert_eq!(value(&r, "qbi_deduction"), 2190);
    assert_eq!(niit(&d), -10949);
    v.rent = 20000 * 100;
    v.property_type = RentalPropertyType::VacationShortTerm {
        stays: ShortTermStays::SevenDaysOrLess { material_participation: false },
    };
    v.qbi_business = RentalBusiness::No;
    d.income.pop();
    let id = put(&mut d, v.clone());
    let r = supported(&d);
    assert_eq!(amount(&r, id, "suspended_loss"), 17051);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(value(&r, "form8582_line_2b"), -17051);
    // Inventor derives transient from stays: OverSevenDays returns to 27.5 years.
    v.property_type =
        RentalPropertyType::VacationShortTerm { stays: ShortTermStays::OverSevenDays };
    d.income.pop();
    put(&mut d, v.clone());
    assert_eq!(value(&supported(&d), "agi"), 40000);
    v.property_type =
        RentalPropertyType::VacationShortTerm { stays: ShortTermStays::SubstantialServices };
    d.income.pop();
    put(&mut d, v);
    let r = estimate(&d);
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_SHORT_TERM_RENTAL_IS_A_BUSINESS"));
    // Schedule C owns its normal SE computation; routing never constructs a duplicate business.
    d.income.pop();
    entry(&mut d, cash_business(10949));
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_self_employment_tax"), 1547);
    assert_eq!(value(&r, "agi"), 70175);
}
#[test]
fn fixture_b_land_income_does_not_absorb_other_losses() {
    let mut d = wages(160000);
    let mut v = rental(12000, 0);
    v.property_type = RentalPropertyType::Land;
    v.buildings[0].cost = 200000 * 100;
    v.buildings[0].land = v.buildings[0].cost;
    v.expenses = vec![
        expense(RentalExpenseCategory::TaxesAndLicenses, 1500),
        expense(RentalExpenseCategory::LegalAndProfessional, 500),
    ];
    let id = put(&mut d, v);
    let other = put(&mut d, rental(18000, 20000));
    let r = supported(&d);
    assert_eq!(amount(&r, id, "line21_income"), 10000);
    assert_eq!(amount(&r, id, "depreciation"), 0);
    assert_eq!(amount(&r, other, "suspended_loss"), 12000);
    assert_eq!(value(&r, "agi"), 170000);
}
fn mineral(expenses: i64) -> FederalRoyalty {
    FederalRoyalty {
        kind: RoyaltyKind::Mineral { rate: RoyaltyMineralRate::Ten },
        receipts: 20000 * 100,
        advance: 0,
        expenses: vec![
            expense(RentalExpenseCategory::LegalAndProfessional, expenses - 1000),
            expense(RentalExpenseCategory::TaxesAndLicenses, 1000),
        ],
        depletion: RoyaltyDepletion::Percentage {
            adjusted_basis: 800 * 100,
            // D-2026-10-15-13 (2026-10-16): full-width source units retain
            // the same 1/20 cost fraction and every original monetary result.
            recoverable_units: 8_000_000_000_000_000_000,
            units_sold: 400_000_000_000_000_000,
            carryover: 0,
            amt_carryover: 0,
        },
        ordinary: true,
    }
}
#[test]
fn fixture_c_mineral_and_oil_depletion_regular_and_amt() {
    let mut d = wages(50000);
    let id = entry(&mut d, FederalIncome::Royalty { data: mineral(3000) });
    let r = supported(&d);
    let val = |r: &FederalResult, s: &str| value(r, &alloc::format!("royalty_{id}_{s}"));
    assert_eq!(val(&r, "line18_depletion"), 2000);
    assert_eq!(val(&r, "line20_total_expenses"), 5000);
    assert_eq!(val(&r, "line21_income"), 15000);
    assert_eq!(val(&r, "basis_end"), 0);
    assert_eq!(value(&r, "agi"), 65000);
    assert_eq!(value(&r, "form6251_line2d_depletion"), 1200);
    d.income.last_mut().unwrap().income = FederalIncome::Royalty { data: mineral(17000) };
    let r = supported(&d);
    assert_eq!(val(&r, "line18_depletion"), 1500);
    assert_eq!(val(&r, "line21_income"), 1500);
    assert_eq!(value(&r, "form6251_line2d_depletion"), 700);
    let mut d = empty_return(FederalStatus::Single);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    let mut v = mineral(5000);
    v.kind = RoyaltyKind::OilGas;
    v.receipts = 40000 * 100;
    v.depletion = RoyaltyDepletion::Percentage {
        adjusted_basis: 0,
        recoverable_units: 0,
        units_sold: 0,
        carryover: 0,
        amt_carryover: 0,
    };
    let id = entry(&mut d, FederalIncome::Royalty { data: v });
    entry(&mut d, cash_business(-30000));
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("royalty_{id}_line18_depletion")), 0);
    assert_eq!(value(&r, &alloc::format!("royalty_{id}_depletion_carryover")), 6000);
    assert_eq!(value(&r, &alloc::format!("amt_royalty_{id}_depletion_carryover")), 2750);
    assert_eq!(value(&r, "form6251_line2d_depletion"), -3250);
}
#[test]
fn fixture_d_self_rental_income_and_loss() {
    let mut d = wages(160000);
    let mut v = rental(36000, 10000);
    v.property_type = RentalPropertyType::SelfRental;
    v.buildings[0].cost = 500000 * 100;
    v.buildings[0].land = 100000 * 100;
    v.buildings[0].in_service = 20190601;
    v.qbi_business = RentalBusiness::Yes { material_participation: true };
    let id = put(&mut d, v.clone());
    let other = put(&mut d, rental(18000, 20000));
    let r = supported(&d);
    assert_eq!(amount(&r, id, "depreciation"), 10256);
    assert_eq!(amount(&r, id, "line21_income"), 15744);
    assert_eq!(value(&r, "agi"), 175744);
    assert_eq!(amount(&r, other, "suspended_loss"), 12000);
    assert_eq!(value(&r, "qbi_deduction"), 3149);
    assert_eq!(niit(&d), -15744);
    v.rent = 18000 * 100;
    d.income[1].income = FederalIncome::Rental { data: v };
    let r = supported(&d);
    assert_eq!(amount(&r, id, "suspended_loss"), 2256);
    assert_eq!(amount(&r, other, "suspended_loss"), 12000);
    assert_eq!(value(&r, "agi"), 160000);
}
#[test]
fn fixture_e_professional_and_former_passive() {
    let mut d = wages(120000);
    make_professional(&mut d, Owner::Taxpayer, false);
    let mut v = rental(10000, 20000);
    v.participation = RentalParticipation::ProfessionalActive;
    let id = put(&mut d, v.clone());
    put(&mut d, rental(10000, 10000));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 90000);
    assert_eq!(value(&r, "rental_magi"), 120000);
    assert_eq!(value(&r, "schedule_e_line43_professional_rentals"), -20000);
    v.rent = 14000 * 100;
    v.expenses.clear();
    v.history = RentalHistory { regular: 6000 * 100, amt: 6000 * 100, active_in_loss_year: true };
    d.income[1].income = FederalIncome::Rental { data: v };
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 108000);
    assert_eq!(amount(&r, id, "line22_allowed_loss"), -6000);
    assert_eq!(value(&r, "form8582_line_1a"), 4000);
    assert_eq!(value(&r, "form8582_line_1b"), -10000);
    assert_eq!(value(&r, "form8582_line_1c"), -6000);
    assert_eq!(value(&r, "schedule_e_line43_professional_rentals"), -2000);
}
#[test]
fn fixture_f_at_risk_limit_precedes_passive_allowance() {
    let mut d = wages(60000);
    let mut v = rental(18000, 38000);
    v.at_risk = simplified_rental_risk(40000 * 100, 0, 22000 * 100, [0, 0, 0, 0]);
    let id = put(&mut d, v);
    let r = supported(&d);
    for (suffix, want) in [
        ("form6198_line1", -30000),
        ("form6198_line5", -30000),
        ("form6198_line6", 40000),
        ("form6198_line8", 40000),
        ("form6198_line9", 22000),
        ("form6198_line10a", 18000),
        ("form6198_line10b", 18000),
        ("form6198_line20", 18000),
        ("form6198_line21", 18000),
        ("at_risk_carryover", 12000),
        ("line21_income", -30000),
    ] {
        assert_eq!(amount(&r, id, suffix), want, "{suffix}");
    }
    assert_eq!(value(&r, "agi"), 42000);
    assert_eq!(value(&r, &alloc::format!("amt_rental_{id}_at_risk_carryover")), 12000);
}
pub(in crate::tests) fn home_fixture() -> FederalRental {
    let mut v = rental(6000, 0);
    v.expenses = vec![
        expense(RentalExpenseCategory::MortgageInterest, 12000),
        expense(RentalExpenseCategory::TaxesAndLicenses, 4000),
        expense(RentalExpenseCategory::Utilities, 3000),
    ];
    v.personal_use = RentalPersonalUse::Yes {
        fair_rental_days: 60,
        personal_days: 30,
        overlap_days: 0,
        direct_expenses: 500 * 100,
    };
    v
}

#[test]
fn schedule_e_columns_and_totals_foot() {
    let mut d = wages(160000);
    put(&mut d, rental(48000, 30000));
    put(&mut d, rental(18000, 20000));
    entry(&mut d, FederalIncome::Royalty { data: mineral(3000) });
    let mut loss = mineral(25000);
    loss.depletion = RoyaltyDepletion::No;
    entry(&mut d, FederalIncome::Royalty { data: loss });
    let r = supported(&d);
    let sum = |printed: &str| -> i64 {
        r.lines
            .iter()
            .filter(|l| {
                l.form_line
                    .as_ref()
                    .is_some_and(|f| f.form == TaxForm::ScheduleE && f.line == printed)
            })
            .map(|l| l.amount)
            .sum()
    };
    assert_eq!(sum("23a"), sum("3"));
    assert_eq!(sum("23b"), sum("4"));
    assert_eq!(sum("23c"), sum("12"));
    assert_eq!(sum("23d"), sum("18"));
    assert_eq!(sum("23e"), sum("20"));
    assert_eq!(sum("26"), sum("24") + sum("25"));
    for e in &d.income {
        let prefix = match e.income {
            FederalIncome::Rental { .. } => "rental",
            FederalIncome::Royalty { .. } => "royalty",
            _ => continue,
        };
        let total = line_amount(&r, &alloc::format!("{prefix}_{}_line20_total_expenses", e.id));
        let expenses: i64 = r
            .lines
            .iter()
            .filter(|l| l.id.starts_with(&alloc::format!("{prefix}_{}_expense_", e.id)))
            .map(|l| l.amount)
            .sum();
        let depreciation = r
            .lines
            .iter()
            .find(|l| {
                l.id == alloc::format!(
                    "{prefix}_{}_line18_{}",
                    e.id,
                    if prefix == "rental" { "depreciation" } else { "depletion" }
                )
            })
            .map_or(0, |l| l.amount);
        assert_eq!(total, expenses + depreciation);
    }
}
#[test]
fn legal_fractional_mineral_rate_and_cost_floor() {
    let mut d = wages(60000);
    let mut v = mineral(3000);
    v.kind = RoyaltyKind::Mineral { rate: RoyaltyMineralRate::SevenAndHalf };
    let id = entry(&mut d, FederalIncome::Royalty { data: v.clone() });
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("royalty_{id}_line18_depletion")), 1500);
    v.depletion = RoyaltyDepletion::Percentage {
        adjusted_basis: 100000 * 100,
        recoverable_units: 100,
        units_sold: 5,
        carryover: 0,
        amt_carryover: 0,
    };
    d.income[1].income = FederalIncome::Royalty { data: v };
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("royalty_{id}_line18_depletion")), 5000);
    assert_eq!(value(&r, "form6251_line2d_depletion"), 0);
}
#[test]
fn rental_assets_and_converted_basis_reuse_depreciation() {
    let mut d = wages(60000);
    let mut v = rental(48000, 10000);
    v.buildings[0].acquired = RentalAcquisition::Converted { fair_market_value: 220000 * 100 };
    v.assets = vec![rental_equipment(5000, BonusElection::ElectOut)];
    let id = put(&mut d, v);
    let r = supported(&d);
    assert_eq!(amount(&r, id, "depreciation"), 9000); // 220000/27.5 + 20% of 5000
    assert_eq!(value(&r, &alloc::format!("rental_{id}_asset_0_macrs")), 1000);
    let FederalIncome::Rental { data } = &mut d.income[1].income else { unreachable!() };
    data.sale = RentalSale::Sold {
        date: 20261001,
        price: 400000 * 100,
        land_price: 55000 * 100,
        selling_expenses: 0,
        payment: SalePayment::InFull,
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
    assert!(estimate(&d).issues.iter().any(|i| i.display.key == "ISSUE_RENTAL_SALE_DUAL_BASIS"));
}
#[test]
fn professional_election_checks_all_rentals_and_retains_loss_on_partial_disposition() {
    let mut d = wages(160000);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    make_professional(&mut d, Owner::Taxpayer, false);
    let mut a = rental(10000, 10000);
    a.participation = RentalParticipation::Professional;
    a.history = RentalHistory { regular: 6000 * 100, amt: 6000 * 100, active_in_loss_year: true };
    a.sale = RentalSale::Sold {
        date: 20261001,
        price: 400000 * 100,
        land_price: 55000 * 100,
        selling_expenses: 0,
        payment: SalePayment::InFull,
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
    let id = put(&mut d, a);
    let mut b = rental(20000, 10000);
    b.participation = RentalParticipation::Professional;
    put(&mut d, b);
    d.people
        .as_mut()
        .unwrap()
        .taxpayer
        .real_estate_professional
        .as_mut()
        .unwrap()
        .single_activity = true;
    let r = supported(&d);
    // The original $400,000 member sale produces combined current income
    // $129,583 ($137,500 property gain less $7,917 operating loss). Total
    // depreciation is $67,500: $17,500 for the two partial years combined
    // plus five full $10,000 years; adjusted property basis is $262,500. The
    // $6,000 prior loss is absorbed under section469(f), even though the
    // member sale does not qualify as an entire-activity section469(g) sale.
    assert_eq!(amount(&r, id, "suspended_loss"), 0);
    assert_eq!(value(&r, "schedule_e_line43_professional_rentals"), 123_583);
    // A member sale with no combined current net income must still retain
    // the old loss. Reducing the same sale price to $230,000 gives combined
    // current loss $40,417, without disposing of the other elected member.
    let mut no_income = d.clone();
    let FederalIncome::Rental { data } = &mut no_income.income[1].income else { unreachable!() };
    let RentalSale::Sold { price, .. } = &mut data.sale else { unreachable!() };
    *price = 23_000_000;
    let no_income = supported(&no_income);
    assert_eq!(amount(&no_income, id, "suspended_loss"), 6_000);
    assert_eq!(value(&no_income, "schedule_e_line43_professional_rentals"), -40_417);
    let FederalIncome::Rental { data } = &mut d.income[2].income else { unreachable!() };
    data.participation = RentalParticipation::Active;
    assert_eq!(
        crate::extended_rules::professional_election_rule(&d),
        Some("ISSUE_PROFESSIONAL_ELECTION_ALL_RENTALS")
    );
    d.people
        .as_mut()
        .unwrap()
        .taxpayer
        .real_estate_professional
        .as_mut()
        .unwrap()
        .single_activity = false;
    let r = supported(&d);
    assert_eq!(amount(&r, id, "suspended_loss"), 0);
}
#[test]
fn self_rental_disposition_gain_is_excluded_from_investment_income() {
    let mut d = wages(300000);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let mut a = rental(36000, 10000);
    a.property_type = RentalPropertyType::SelfRental;
    a.sale = RentalSale::Sold {
        date: 20261001,
        price: 400000 * 100,
        land_price: 55000 * 100,
        selling_expenses: 0,
        payment: SalePayment::InFull,
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
    put(&mut d, a);
    let r = supported(&d);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
    assert!(value(&r, "niit_disposition_adjustment") < 0);
}

#[test]
fn oil_limit_recomputes_income_dependent_deductions_and_keeps_the_nol_deduction() {
    let mut d = empty_return(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    itemized(ded).medical = 30000 * 100;
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let mut v = mineral(0);
    v.expenses.clear();
    v.kind = RoyaltyKind::OilGas;
    v.receipts = 40000 * 100;
    v.depletion = RoyaltyDepletion::Percentage {
        adjusted_basis: 0,
        recoverable_units: 0,
        units_sold: 0,
        carryover: 30000 * 100,
        amt_carryover: 30000 * 100,
    };
    let id = entry(&mut d, FederalIncome::Royalty { data: v });
    // A wide taxable-income product does not prevent allocation of actual
    // source amounts: the existing $36,000 property total remains the limit.
    for alternative in [false, true] {
        let rows = crate::royalties::calculate(&d, Some(i64::MAX), alternative).unwrap();
        assert_eq!(rows[0].allowance, 3_600_000);
        assert_eq!(rows[0].carryover, 0);
    }
    // Before depletion: AGI 40,000; medical 30,000 - 3,000 = 27,000;
    // taxable income 13,000; 65% = 8,450. Adding depletion back to an
    // already-computed medical deduction would use the wrong AGI floor.
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("royalty_{id}_line18_depletion")), 8450);
    assert_eq!(value(&r, &alloc::format!("royalty_{id}_depletion_carryover")), 27550);
    // Section 613A(d)(1) disregards only NOL carrybacks: a 2025 loss carried
    // to 2026 stays in the 65% base, while its own 80% limit (section
    // 172(a)(2)) reads taxable income after the depletion. With D depletion
    // and N the NOL deduction, AGI = 40,000 - D - N and the medical floor
    // follows it: N = 0.8 x (13,000 - 1.075 D - 0.075 N) and D = 0.65 x
    // (13,000 - 1.075 N), which settle at D = 3,681 and N = 6,825.
    d.screening.as_mut().unwrap().nol = NolHistory::Records {
        years: vec![NolYear {
            ptp_ebl: crate::PtpEblNol::NoPtpEbl,
            year: 2025,
            balance: 10000 * 100,
            injured_owners: None,
            research_parts: None,
        }],
        confirmed: true,
    };
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("royalty_{id}_line18_depletion")), 3681);
    assert_eq!(value(&r, &alloc::format!("royalty_{id}_depletion_carryover")), 32318);
    assert_eq!(value(&r, "prior_nol_deduction"), 6824);
}

fn risk_limit(amount: i64) -> Option<RentalAtRisk> {
    simplified_rental_risk(amount * 100, 0, 0, [0, 0, 0, 0])
}
#[test]
fn at_risk_allocates_operations_and_disposition_losses_without_changing_character() {
    for operating_loss in [0, 10000] {
        let mut d = wages(60000);
        d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
        let mut v = rental(0, operating_loss);
        v.buildings[0].cost = 20000 * 100;
        v.buildings[0].land = 20000 * 100;
        v.property_type = RentalPropertyType::Land;
        v.at_risk = risk_limit(5000);
        v.sale = RentalSale::Sold {
            date: 20261001,
            price: 10000 * 100,
            land_price: 10000 * 100,
            selling_expenses: 0,
            payment: SalePayment::InFull,
            buyer: crate::Buyer::Unrelated,
            personal_share_percent: 0,
        };
        let id = put(&mut d, v);
        let r = supported(&d);
        let ordinary = if operating_loss == 0 { 0 } else { -2500 };
        let sale = -5000 - ordinary;
        assert_eq!(amount(&r, id, "line21_income"), -operating_loss);
        assert_eq!(amount(&r, id, "at_risk_allowed_ordinary"), ordinary);
        assert_eq!(amount(&r, id, "at_risk_allowed_section1231"), sale);
        assert_eq!(amount(&r, id, "at_risk_ordinary_carryover"), operating_loss + ordinary);
        assert_eq!(amount(&r, id, "at_risk_section1231_carryover"), 10000 + sale);
        assert_eq!(crate::asset_sale::rental_sources(&d, false).unwrap()[0].1.section1231, sale);
        assert_eq!(value(&r, "form4797_line8"), sale);
        assert_eq!(value(&r, "agi"), 55000);
        assert_eq!(r, estimate(&restart(d.clone()).draft));
    }
}
#[test]
fn prior_at_risk_carry_preserves_regular_and_amt_sale_character_without_new_sale() {
    let mut d = wages(60000);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let mut v = rental(0, 0);
    v.participation = RentalParticipation::Professional;
    make_professional(&mut d, Owner::Taxpayer, false);
    v.buildings[0].land = v.buildings[0].cost;
    v.at_risk =
        simplified_rental_risk(1000 * 100, 0, 0, [6000 * 100, 2000 * 100, 4000 * 100, 3000 * 100]);
    // 2026-10-18, i8995 QBI loss tracking: actual synthetic pre-2018
    // property-loss records establish zero QBI, independently of today's No.
    v.property_qbi_history = vec![rental_property_loss_year(2017, 400_000, 0, 400_000, 0)];
    let id = put(&mut d, v);
    let r = supported(&d);
    assert_eq!(amount(&r, id, "at_risk_allowed_ordinary"), -600);
    assert_eq!(amount(&r, id, "at_risk_allowed_section1231"), -400);
    assert_eq!(amount(&r, id, "at_risk_ordinary_carryover"), 5400);
    assert_eq!(amount(&r, id, "at_risk_section1231_carryover"), 3600);
    assert_eq!(value(&r, &alloc::format!("amt_rental_{id}_at_risk_ordinary_carryover")), 1600);
    assert_eq!(value(&r, &alloc::format!("amt_rental_{id}_at_risk_section1231_carryover")), 2400);
    assert_eq!(value(&r, "agi"), 59000);
    assert_eq!(crate::asset_sale::rental_sources(&d, true).unwrap()[0].1.section1231, -600);
}

// ---- Codex-branch review defects and suspected items (F16, review of 2026-10-10) ----

// D1: Pub. 925 "Rental of Nondepreciable Property" and self-rental: only the
// net passive income after the activity's own prior-year unallowed loss is
// nonpassive; the prior loss never reaches the Form 8582 pool.
#[test]
fn recharacterized_income_absorbs_the_activity_prior_unallowed_loss_first() {
    let mut d = wages(160000);
    let mut v = rental(36000, 10000);
    v.property_type = RentalPropertyType::SelfRental;
    v.buildings[0].cost = 500000 * 100;
    v.buildings[0].land = 100000 * 100;
    v.buildings[0].in_service = 20190601;
    v.history = RentalHistory { regular: 10000 * 100, amt: 10000 * 100, active_in_loss_year: true };
    let id = put(&mut d, v);
    let r = supported(&d);
    assert_eq!(amount(&r, id, "line21_income"), 15744);
    assert_eq!(amount(&r, id, "line22_allowed_loss"), -10000);
    assert_eq!(amount(&r, id, "suspended_loss"), 0);
    assert_eq!(value(&r, "agi"), 165744);
    assert_eq!(niit(&d), -5744);
    assert!(
        !r.lines
            .iter()
            .any(|l| { l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8582) })
    );
    // Land (the 30% test) with the same prior loss and no allowance room.
    let mut d = wages(160000);
    let mut l = rental(12000, 2000);
    l.property_type = RentalPropertyType::Land;
    l.buildings[0].cost = 200000 * 100;
    l.buildings[0].land = 200000 * 100;
    l.history = RentalHistory { regular: 10000 * 100, amt: 10000 * 100, active_in_loss_year: true };
    let id = put(&mut d, l);
    let r = supported(&d);
    assert_eq!(amount(&r, id, "line21_income"), 10000);
    assert_eq!(amount(&r, id, "suspended_loss"), 0);
    assert_eq!(value(&r, "agi"), 160000);
    assert_eq!(value(&r, "rental_magi"), 160000);
}

// D2: a short-stay business variant is not rental real estate (map R2), so a
// real estate professional's participation answer neither gates nor counts.
#[test]
fn short_stay_business_ignores_the_professional_participation_answer() {
    let mut d = wages(60000);
    make_professional(&mut d, Owner::Taxpayer, false);
    let mut v = rental(48000, 30000);
    v.property_type = RentalPropertyType::VacationShortTerm {
        stays: ShortTermStays::SevenDaysOrLess { material_participation: true },
    };
    v.qbi_business = RentalBusiness::Yes { material_participation: true };
    v.participation = RentalParticipation::Professional;
    let id = put(&mut d, v);
    let r = supported(&d);
    assert_eq!(amount(&r, id, "line21_income"), 10949);
    assert_eq!(value(&r, "agi"), 70949);
    assert_eq!(value(&r, "schedule_e_line43_professional_rentals"), 0);
    // Without the recorded status the answer is still ignored.
    d.people.as_mut().unwrap().taxpayer.real_estate_professional = None;
    assert_eq!(value(&supported(&d), "agi"), 70949);
}

// D4: the AMT at-risk character rows carry label resources.
#[test]
fn amt_at_risk_character_rows_are_registered() {
    let mut d = wages(60000);
    let mut v = rental(18000, 38000);
    v.at_risk = simplified_rental_risk(40000 * 100, 0, 22000 * 100, [5000 * 100, 5000 * 100, 0, 0]);
    let id = put(&mut d, v);
    let mut oil = mineral(5000);
    oil.kind = RoyaltyKind::OilGas;
    entry(&mut d, FederalIncome::Royalty { data: oil });
    let r = supported(&d);
    let unregistered: Vec<_> = r
        .lines
        .iter()
        .filter(|l| !crate::federal_model::IdKeys::new(&l.id).registered())
        .map(|l| l.id.clone())
        .collect();
    assert!(unregistered.is_empty(), "{unregistered:?}");
    for suffix in [
        "at_risk_ordinary_carryover",
        "at_risk_section1231_carryover",
        "at_risk_allowed_ordinary",
        "at_risk_allowed_section1231",
    ] {
        let id = alloc::format!("amt_rental_{id}_{suffix}");
        assert_eq!(
            crate::line_label(&id).key,
            alloc::format!("LINE_AMT_RENTAL_N_{}", suffix.to_uppercase())
        );
        value(&r, &id);
    }
}

// S1: a lease bonus or advance royalty is excluded from oil and gas percentage
// depletion only (section 613A(d)(5)); a mineral's rate applies to it.
#[test]
fn advance_royalties_leave_oil_and_gas_percentage_depletion_only() {
    let mut d = wages(50000);
    let mut coal = mineral(3000);
    coal.advance = 10000 * 100;
    let id = entry(&mut d, FederalIncome::Royalty { data: coal });
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("royalty_{id}_line18_depletion")), 2000);
    let mut oil = mineral(3000);
    oil.kind = RoyaltyKind::OilGas;
    oil.advance = 10000 * 100;
    d.income.last_mut().unwrap().income = FederalIncome::Royalty { data: oil };
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("royalty_{id}_line18_depletion")), 1500);
}

// S3: the AMT twin of Worksheet 5-1 has no line 2b (taxes are not AMT
// deductions), so the rental share of taxes is limited with line 4.
#[test]
fn amt_home_worksheet_limits_the_rental_share_of_taxes_with_operating_expenses() {
    let mut d = wages(60000);
    let mut home = home_fixture();
    link_home_fixture(&mut d, &mut home);
    let id = put(&mut d, home);
    d.deductions.as_mut().unwrap().choice =
        DeductionSelection::Itemized { facts: itemized(d.deductions.as_mut().unwrap()).clone() };
    let r = supported(&d);
    assert_eq!(amount(&r, id, "home_operating_carryover"), 2000);
    assert_eq!(value(&r, &alloc::format!("amt_rental_{id}_home_operating_carryover")), 4666);
    assert_eq!(value(&r, &alloc::format!("amt_rental_{id}_home_depreciation_carryover")), 6666);
}

// S4: a section 280A home year is outside section 469, so the single-activity
// election does not require its Professional answer.
#[test]
fn professional_election_ignores_a_home_year() {
    let mut d = wages(160000);
    make_professional(&mut d, Owner::Taxpayer, false);
    d.people
        .as_mut()
        .unwrap()
        .taxpayer
        .real_estate_professional
        .as_mut()
        .unwrap()
        .single_activity = true;
    let mut a = rental(20000, 10000);
    a.participation = RentalParticipation::Professional;
    put(&mut d, a);
    let mut home = home_fixture();
    link_home_fixture(&mut d, &mut home);
    put(&mut d, home);
    assert_eq!(crate::extended_rules::professional_election_rule(&d), None);
    supported(&d);
    let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.personal_use = RentalPersonalUse::No;
    data.mortgage_loans.clear();
    assert_eq!(
        crate::extended_rules::professional_election_rule(&d),
        Some("ISSUE_PROFESSIONAL_ELECTION_ALL_RENTALS")
    );
}

#[test]
fn unit4_direct_former_professional_remainder_requires_current_and_loss_year_active() {
    for (participation, active_in_loss_year) in [
        (RentalParticipation::Professional, true),
        (RentalParticipation::ProfessionalActive, false),
    ] {
        let mut d = wages(120_000);
        make_professional(&mut d, Owner::Taxpayer, false);
        let mut former = rental(14_000, 0);
        former.participation = participation;
        former.history = RentalHistory { regular: 600_000, amt: 600_000, active_in_loss_year };
        let id = put(&mut d, former);
        put(&mut d, rental(10_000, 10_000));
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 110_000);
        assert_eq!(amount(&r, id, "suspended_loss"), 2_000);
        assert_eq!(amount(&r, id, "line22_allowed_loss"), -4_000);
        assert_eq!(value(&r, "schedule_e_line43_professional_rentals"), 0);
        assert_eq!(value(&r, &alloc::format!("amt_rental_{id}_suspended_loss")), 2_000);
    }
}

// Reg. 1.469-9(e)(4)'s U/V/W amounts, with the annual facts advanced to
// 2026. The regulation gives combined income $10,000, prior loss $40,000,
// release $10,000 and closing loss $30,000 independently of the buildings
// that earned the income. No special allowance is elected in this control.
pub(in crate::tests) fn unit4_three_buildings() -> (FederalDraft, [i64; 3]) {
    let mut d = wages(60_000);
    make_professional(&mut d, Owner::Taxpayer, true);
    d.people
        .as_mut()
        .unwrap()
        .taxpayer
        .real_estate_professional
        .as_mut()
        .unwrap()
        .single_activity = true;
    let mut ids = [0; 3];
    for (index, (rent, expenses, regular, amt)) in
        [(5_000, 0, 30_000, 24_000), (0, 5_000, 10_000, 6_000), (10_000, 0, 0, 0)]
            .into_iter()
            .enumerate()
    {
        // The existing building depreciates $10,000 in each tax system;
        // receipts include that amount so the three current nets are exact.
        let mut data = rental(rent + 10_000, expenses);
        data.participation = RentalParticipation::Professional;
        data.history =
            RentalHistory { regular: regular * 100, amt: amt * 100, active_in_loss_year: false };
        ids[index] = put(&mut d, data);
    }
    (d, ids)
}

#[test]
fn unit4_elected_former_passive_matches_regulation_three_building_example() {
    let (d, [u, v, w]) = unit4_three_buildings();
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "schedule_e_line43_professional_rentals"), 0);
    assert_eq!(amount(&r, u, "suspended_loss"), 22_500);
    assert_eq!(amount(&r, v, "suspended_loss"), 7_500);
    assert_eq!(amount(&r, u, "line22_allowed_loss"), -7_500);
    assert_eq!(amount(&r, v, "line22_allowed_loss"), -7_500);
    assert_eq!(value(&r, &alloc::format!("amt_rental_{u}_suspended_loss")), 16_000);
    assert_eq!(value(&r, &alloc::format!("amt_rental_{v}_suspended_loss")), 4_000);
    assert_eq!(niit(&d), 0);
    assert!(
        r.lines
            .iter()
            .filter(|line| line.id == alloc::format!("rental_{v}_line22_allowed_loss"))
            .count()
            == 1
    );
    assert_eq!(amount(&r, w, "line21_income"), 10_000);
    let mut reordered = d.clone();
    reordered.income.reverse();
    for entry in &mut reordered.income {
        entry.label = "Renamed synthetic source".into();
    }
    let reordered = supported(&reordered);
    for id in [
        "agi",
        "schedule_e_line43_professional_rentals",
        "rental_suspended_loss",
        "amt_rental_suspended_loss",
    ] {
        assert_eq!(value(&reordered, id), value(&r, id));
    }
    for id in [u, v] {
        assert_eq!(amount(&reordered, id, "suspended_loss"), amount(&r, id, "suspended_loss"));
    }
    let mut separate = d.clone();
    separate
        .people
        .as_mut()
        .unwrap()
        .taxpayer
        .real_estate_professional
        .as_mut()
        .unwrap()
        .single_activity = false;
    let separate = supported(&separate);
    assert_eq!(value(&separate, "agi"), 65_000);
    assert_eq!(value(&separate, "rental_suspended_loss"), 35_000);
    assert_eq!(value(&separate, "schedule_e_line43_professional_rentals"), 5_000);
    // The accepted histories are never replaced by the annual projection.
    let FederalIncome::Rental { data } = &d.income[1].income else { unreachable!() };
    assert_eq!(data.history.regular, 3_000_000);
    assert_eq!(data.history.amt, 2_400_000);
}

#[test]
fn unit4_joint_elected_safe_harbor_reaches_both_source_owners_without_copying_hours() {
    let mut d = complete(FederalStatus::Joint);
    make_professional(&mut d, Owner::Taxpayer, true);
    d.people
        .as_mut()
        .unwrap()
        .taxpayer
        .real_estate_professional
        .as_mut()
        .unwrap()
        .single_activity = true;
    for (owner, receipts) in [(Owner::Taxpayer, 20_000), (Owner::Spouse, 30_000)] {
        let mut data = rental(receipts, 0);
        data.participation = RentalParticipation::Professional;
        add(&mut d, income(FederalIncome::Rental { data }));
        d.income.last_mut().unwrap().owner = owner;
    }
    assert_eq!(value(&supported(&d), "schedule_e_line43_professional_rentals"), 30_000);
    assert_eq!(niit(&d), -30_000);
    assert!(d.people.as_ref().unwrap().professional(Owner::Spouse).is_none());
    // Without documented hours, professional status alone is not the NIIT safe harbor.
    d.people.as_mut().unwrap().taxpayer.real_estate_professional.as_mut().unwrap().over_500_hours =
        false;
    assert_eq!(niit(&d), 0);
    // Separate returns never acquire the other person's qualification/election.
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Separate);
    assert!(d.professional(Owner::Spouse).is_none());
    assert_eq!(d.professional_group_owner(Owner::Spouse), Owner::Spouse);
}

#[test]
fn unit42_prior_home_carry_floors_fractional_capacity_once() {
    let mut source = rental(0, 0);
    source.home_depreciation_carryover = crate::SourceCents::Reported { amount: 300_000 };
    source.home_casualty_carryover = crate::SourceCents::Reported { amount: 150_000 };
    let capacity = crate::kernel::ratio::exact::ratio(69_999_999, 100).unwrap();
    let result = crate::rental_home_carry::tiers(&source, false, 500_000, &capacity).unwrap();
    // $6,999.9999 of capacity permits $6,999.99, never $7,000.00.
    assert_eq!(result.total_allowed_cents, 699_999);
    assert_eq!(result.allowed_current_depreciation_cents, 500_000);
    assert_eq!(result.allowed_prior_casualty_cents, Some(150_000));
    assert_eq!(result.allowed_prior_depreciation_cents, Some(49_999));
    assert_eq!(result.carry.total_cents, 100_001);
    assert_eq!(result.carry.casualty_cents, Some(0));
}

#[test]
fn unit42_prior_home_carry_keeps_independent_amt_character_and_deducts_once() {
    let mut draft = wages(60_000);
    let mut source = rental(7_000, 0);
    source.personal_use = RentalPersonalUse::Yes {
        fair_rental_days: 100,
        personal_days: 100,
        overlap_days: 0,
        direct_expenses: 0,
    };
    source.home_depreciation_carryover = crate::SourceCents::Reported { amount: 300_000 };
    source.amt_home_depreciation_carryover = crate::SourceCents::Reported { amount: 400_000 };
    source.home_casualty_carryover = crate::SourceCents::Reported { amount: 150_000 };
    source.amt_home_casualty_carryover = crate::SourceCents::Reported { amount: 250_000 };
    let id = put(&mut draft, source);
    let result = supported(&draft);
    assert_eq!(amount(&result, id, "depreciation"), 5_500);
    assert_eq!(amount(&result, id, "home_depreciation_carryover"), 1_000);
    assert_eq!(amount(&result, id, "home_casualty_allowed"), 1_500);
    assert_eq!(value(&result, &alloc::format!("amt_rental_{id}_depreciation")), 5_000);
    assert_eq!(value(&result, &alloc::format!("amt_rental_{id}_home_casualty_allowed")), 2_000);
    assert_eq!(value(&result, &alloc::format!("amt_rental_{id}_home_casualty_carryover")), 500);
    assert_eq!(value(&result, "rental_home_prior_casualty_ordinary"), -1_500);
    assert_eq!(value(&result, "agi"), 60_000);
}

#[test]
fn unit42_prior_home_carry_amt_only_former_home_uses_its_own_records() {
    let mut draft = wages(60_000);
    let mut source = rental(12_000, 0);
    source.amt_home_depreciation_carryover = crate::SourceCents::Reported { amount: 300_000 };
    let id = put(&mut draft, source);
    let result = supported(&draft);
    assert!(
        !result
            .lines
            .iter()
            .any(|line| line.id == alloc::format!("rental_{id}_home_depreciation_carryover"))
    );
    assert_eq!(
        value(&result, &alloc::format!("amt_rental_{id}_home_depreciation_carryover")),
        1_000
    );
    assert_eq!(value(&result, &alloc::format!("amt_rental_{id}_depreciation")), 12_000);
    assert_eq!(value(&result, "agi"), 62_000);
}

#[test]
fn unit42_prior_home_carry_unknown_character_keeps_only_known_aggregate() {
    let mut draft = wages(60_000);
    let mut source = rental(7_000, 0);
    source.personal_use = RentalPersonalUse::Yes {
        fair_rental_days: 100,
        personal_days: 100,
        overlap_days: 0,
        direct_expenses: 0,
    };
    source.home_depreciation_carryover = crate::SourceCents::Reported { amount: 300_000 };
    source.home_casualty_carryover = crate::SourceCents::Unavailable;
    let id = put(&mut draft, source);
    let mut result = FederalResult::empty(0);
    let calculated = crate::extended_rules::rentals(&draft, 60_000, &mut result);
    assert!(calculated.source_issue.is_some());
    assert_eq!(amount(&result, id, "home_third_tier_allowed"), 7_000);
    assert_eq!(amount(&result, id, "home_depreciation_carryover"), 1_000);
    assert!(!result.lines.iter().any(|line| line.id == alloc::format!("rental_{id}_depreciation")
        || line.id == alloc::format!("rental_{id}_home_casualty_allowed")));
}

#[test]
fn unit42_prior_home_carry_unknown_amt_never_uses_the_regular_total() {
    let mut draft = wages(60_000);
    let mut source = rental(12_000, 0);
    source.home_depreciation_carryover = crate::SourceCents::Reported { amount: 100_000 };
    source.amt_home_depreciation_carryover = crate::SourceCents::Unavailable;
    put(&mut draft, source);
    let mut regular = FederalResult::empty(0);
    let regular = crate::extended_rules::rentals_for(&draft, 60_000, &mut regular, false);
    assert_eq!(regular.income, 1_000);
    assert!(regular.source_issue.is_none());
    let mut alternative = FederalResult::empty(0);
    let alternative = crate::extended_rules::rentals_for(&draft, 60_000, &mut alternative, true);
    assert!(alternative.source_issue.is_some());
    assert!(!alternative.inputs_complete);
}

#[test]
fn unit42_prior_home_carry_unrepresentable_close_is_unavailable_without_a_cap() {
    let mut source = rental(0, 0);
    source.home_depreciation_carryover = crate::SourceCents::Reported { amount: i64::MAX };
    let none = crate::kernel::ratio::exact::whole(0);
    assert_eq!(
        crate::rental_home_carry::tiers(&source, false, 1, &none),
        Err(crate::rental_home_carry::OVERFLOW)
    );
    // One cent of room allows the current cent, leaving the representable MAX prior balance.
    let one = crate::kernel::ratio::exact::whole(1);
    assert_eq!(
        crate::rental_home_carry::tiers(&source, false, 1, &one).unwrap().carry.total_cents,
        i64::MAX
    );
}
