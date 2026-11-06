// Approved Schedule C specials map A-G. Expected figures are hand-worked from
// IRS Schedule C, Pub.463/560 and the2026lease/plan constants, not snapshots.
use super::*;
fn draft(receipts: i64) -> FederalDraft {
    let mut d = empty_return(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19810101;
    add(&mut d, income(cash_business(receipts)));
    d
}
fn business(d: &mut FederalDraft) -> &mut FederalBusiness {
    d.income
        .iter_mut()
        .find_map(|e| {
            if let FederalIncome::Business { data } = &mut e.income { Some(data) } else { None }
        })
        .unwrap()
}
fn amount(r: &FederalResult, suffix: &str) -> i64 {
    let id = alloc::format!("business_2_{suffix}");
    let cents =
        r.lines.iter().find(|l| l.id == id).unwrap_or_else(|| panic!("missing {id}")).amount;
    crate::kernel::ratio::dollars(cents)
}
fn expense(category: BusinessExpenseCategory, amount: i64) -> FederalBusinessExpense {
    ordinary_business_expense(category, amount * 100)
}
fn lease() -> BusinessLeasedVehicle {
    BusinessLeasedVehicle {
        began: 20260301,
        ends: None,
        fair_market_value: 8_400_000,
        lease_payments: 780_000,
        total_miles: 16_000,
        business_miles: 12_000,
        commuting: 0,
        operating_costs: 320_000,
        parking_tolls: 15_000,
        passenger_automobile: true,
        personal_use_available: true,
        other_vehicle: true,
        evidence: VehicleEvidence::Written,
    }
}
#[test]
fn schedule_c_specials_a_de_minimis_election_and_omission() {
    let mut d = draft(90_000);
    let b = business(&mut d);
    b.expenses = vec![
        expense(BusinessExpenseCategory::Supplies, 6_000),
        expense(BusinessExpenseCategory::Office, 2_000),
        expense(BusinessExpenseCategory::Advertising, 1_500),
        expense(BusinessExpenseCategory::Insurance, 2_500),
        expense(BusinessExpenseCategory::Other { label: "Software subscriptions".into() }, 8_000),
    ];
    b.expenses.sort_by_key(|expense| expense.category.index());
    b.de_minimis = Some(DeMinimisElection {
        purchases: 420_000,
        largest_item: 180_000,
        financial_statement: false,
        eligible: true,
    });
    let r = supported(&d);
    assert_eq!(amount(&r, "expense_line27b_other"), 12_200);
    assert_eq!(amount(&r, "total_expenses"), 24_200);
    assert_eq!(amount(&r, "net_profit"), 65_800);
    assert_eq!(value(&r, "self_employment_tax"), 9_297);
    assert_eq!(value(&r, "agi"), 61_151);
    let e = business(&mut d).de_minimis.as_mut().unwrap();
    e.largest_item = 260_000;
    assert_eq!(e.rule(), Some("DE_MINIMIS_ITEM_ABOVE_LIMIT"));
    e.financial_statement = true;
    assert!(e.rule().is_none());
    e.eligible = false;
    let r = supported(&d);
    assert_eq!(amount(&r, "expense_line27b_other"), 8_000);
    assert_eq!(amount(&r, "net_profit"), 70_000);
    assert!(r.review_notes.iter().any(|n| n.display.key == "WARN_DE_MINIMIS_EXPENSE_LEFT_OUT"));
    assert_eq!(round_trip(&d), d);
}

#[test]
fn f03_de_minimis_business_share_saves_reopens_and_retains_full_cost_limit() {
    // One $2,000 invoice, 50% documented business use: the whole invoice is
    // below $2,500 and only its $1,000 business share is deductible.
    let election = DeMinimisElection {
        purchases: 100_000,
        largest_item: 200_000,
        financial_statement: false,
        eligible: true,
    };
    assert!(election.rule().is_none());
    assert!(election.valid());
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = crate::tests::cross::interview::business(8_800_000);
    page.more.de_minimis = Some(election.clone());
    crate::tests::cross::interview::accepted(&crate::tests::cross::interview::save_businesses(
        &mut session,
        vec![page.clone()],
    ));
    let (id, saved) = crate::tests::cross::interview::stored_business(&session);
    assert_eq!(saved.de_minimis, Some(election));
    let result = supported(&session.draft);
    assert_eq!(
        line_amount(&result, &alloc::format!("business_{id}_expense_line27b_other")),
        100_000
    );
    assert_eq!(line_amount(&result, &alloc::format!("business_{id}_net_profit")), 8_700_000);
    assert_eq!(
        crate::tests::cross::interview::reopened_businesses(session.draft.clone()),
        vec![page.clone()]
    );
    let before = session.draft.clone();
    // The smaller claimed business share cannot rescue an over-limit invoice.
    for (financial_statement, largest_item) in [(false, 250_001), (true, 500_001)] {
        let claim = page.more.de_minimis.as_mut().unwrap();
        claim.financial_statement = financial_statement;
        claim.largest_item = largest_item;
        assert_eq!(claim.rule(), Some("DE_MINIMIS_ITEM_ABOVE_LIMIT"));
        assert!(!claim.valid());
        crate::tests::cross::interview::rejected_with(
            crate::tests::cross::interview::save_businesses(&mut session, vec![page.clone()]),
            "DE_MINIMIS_ITEM_ABOVE_LIMIT",
        );
        assert_eq!(session.draft, before);
    }
    // The actual full-cost ceilings remain inclusive for both statement choices.
    for (financial_statement, largest_item) in [(false, 250_000), (true, 500_000)] {
        let claim = page.more.de_minimis.as_mut().unwrap();
        claim.financial_statement = financial_statement;
        claim.largest_item = largest_item;
        assert!(claim.valid());
    }
}
#[test]
fn schedule_c_specials_b_leases_all_years_and_last_year() {
    let mut v = lease();
    let a = v.amounts();
    assert_eq!((a.payments, a.inclusion, a.operating), (585_000, 7_000, 255_000));
    let mut d = draft(100_000);
    business(&mut d).leased_vehicles = vec![v.clone()];
    let r = supported(&d);
    assert_eq!(amount(&r, "expense_line20a_rent_vehicles_equipment"), 5_780);
    assert_eq!(amount(&r, "car_and_truck"), 2_550);
    v.began = 20250615;
    v.fair_market_value = 7_050_000;
    v.lease_payments = 900_000;
    v.total_miles = 15_000;
    v.business_miles = 9_000;
    assert_eq!(v.amounts().inclusion, 8_200);
    assert_eq!(v.amounts().payments - v.amounts().inclusion, 531_800);
    v.fair_market_value = 6_100_000;
    assert_eq!(v.amounts().inclusion, 0);
    v.began = 20240201;
    v.ends = Some(20260131);
    v.fair_market_value = 6_650_000;
    v.total_miles = 10_000;
    v.business_miles = 10_000;
    assert_eq!(v.amounts().inclusion, 700);
    v.passenger_automobile = false;
    assert_eq!(v.amounts().inclusion, 0);
    // Independently transcribed terminal-band column applicable in 2026,
    // from the nine IRS procedures named in year_constants.rs.
    for (year, inclusion) in [
        (2018, 2406),
        (2019, 3842),
        (2020, 2139),
        (2021, 293),
        (2022, 1061),
        (2023, 4664),
        (2024, 4128),
        (2025, 2540),
        (2026, 2368),
    ] {
        v.began = year * 10000 + 101;
        v.ends = None;
        v.passenger_automobile = true;
        v.fair_market_value = 80_000_000;
        assert!(v.valid());
        assert_eq!(v.amounts().inclusion, inclusion * 100, "lease began {year}");
    }
    v.began = 20170101;
    assert_eq!(v.rule(), Some("LEASE_BEFORE_2018"));
    assert_eq!(round_trip(&d), d);
}
#[test]
fn schedule_c_specials_c_mixed_meals() {
    let mut d = draft(60_000);
    business(&mut d).expenses = vec![
        FederalBusinessExpense {
            category: BusinessExpenseCategory::Meals,
            amount: 400_000,
            treatment: BusinessExpenseKind::Meals,
        },
        FederalBusinessExpense {
            category: BusinessExpenseCategory::Meals,
            amount: 900_000,
            treatment: BusinessExpenseKind::TransportationMeals,
        },
        expense(BusinessExpenseCategory::Meals, 1_500),
    ];
    let r = supported(&d);
    assert_eq!(amount(&r, "expense_line24b_meals"), 10_700);
    assert_eq!(amount(&r, "net_profit"), 49_300);
    assert_eq!(round_trip(&d), d);
}
#[test]
fn schedule_c_specials_d_simple_owner_and_employee_plans() {
    let mut d = draft(120_000);
    let b = business(&mut d);
    b.expenses = vec![
        expense(BusinessExpenseCategory::Wages, 30_000),
        expense(BusinessExpenseCategory::EmployeePension, 1_950),
        expense(BusinessExpenseCategory::Other { label: "Operating costs".into() }, 8_050),
    ];
    b.expenses.sort_by_key(|expense| expense.category.index());
    b.retirement = BusinessRetirement::Simple {
        deferral: 1_700_000,
        employer: SimpleEmployer::Match { rate: 3.0 },
        employer_contribution: 221_640,
    };
    let r = supported(&d);
    assert_eq!(amount(&r, "expense_line19_pension_plans"), 1_950);
    assert_eq!(amount(&r, "net_profit"), 80_000);
    assert_eq!(amount(&r, "owner_plan_deduction"), 19_216);
    assert_eq!(value(&r, "agi"), 55_132);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19650101;
    let BusinessRetirement::Simple { deferral, .. } = &mut business(&mut d).retirement else {
        panic!()
    };
    *deferral = 2_225_000;
    assert_eq!(amount(&supported(&d), "owner_plan_deduction"), 24_466);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19810101;
    let BusinessRetirement::Simple { deferral, .. } = &mut business(&mut d).retirement else {
        panic!()
    };
    *deferral = 2_000_000;
    let r = supported(&d);
    assert_eq!(amount(&r, "owner_plan_deduction"), 19_216);
    assert!(r.review_notes.iter().any(|n| n.display.key == "NOTE_OWNER_PLAN_LIMITED"));
    assert_eq!(round_trip(&d), d);
}
#[test]
fn schedule_c_specials_d_qualified_worksheet_and_shared_deferral_room() {
    let mut d = draft(100_000);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19740101;
    business(&mut d).retirement = BusinessRetirement::Qualified {
        rate: 25.0,
        contribution: 1_858_700,
        deferral: 2_450_000,
        catch_up: 800_000,
        roth: 0,
    };
    let r = supported(&d);
    assert_eq!(amount(&r, "owner_plan_deduction"), 51_087);
    assert_eq!(value(&r, "owner_plan_2_step21"), 51_087);
    let mut w = wage(3, Owner::Taxpayer, 0, 0);
    let FederalIncome::Wage { data } = &mut w.income else { panic!() };
    let mut data = edit_wage(data);
    data.other_box12.push(W2CodeAmount { code: W2Code::D, amount: 1_000_000 });
    drop(data);
    add(&mut d, w);
    let r = supported(&d);
    assert_eq!(amount(&r, "owner_plan_deduction"), 41_087);
    assert!(r.review_notes.iter().any(|n| n.display.key == "NOTE_OWNER_PLAN_LIMITED"));
    assert_eq!(round_trip(&d), d);
}
#[test]
fn schedule_c_specials_e_inventory_methods() {
    let mut d = draft(110_000);
    business(&mut d).inventory = BusinessInventory::Cost {
        method: InventoryMethod::LowerOfCostOrMarket,
        opening: 1_200_000,
        purchases: 4_000_000,
        personal: 50_000,
        labor: 0,
        materials: 0,
        other: 0,
        closing: 900_000,
        eligible: true,
    };
    let r = supported(&d);
    assert_eq!(amount(&r, "cost_of_goods_sold"), 42_500);
    assert_eq!(amount(&r, "gross_income"), 67_500);
    assert_eq!(amount(&r, "inventory_line40"), 51_500);
    assert_eq!(amount(&r, "inventory_line42"), 42_500);
    let BusinessInventory::Cost { method, .. } = &mut business(&mut d).inventory else { panic!() };
    *method = InventoryMethod::MaterialsAndSupplies;
    assert_eq!(amount(&supported(&d), "cost_of_goods_sold"), 42_500);
    let BusinessInventory::Cost { eligible, .. } = &mut business(&mut d).inventory else {
        panic!()
    };
    *eligible = false;
    let r = supported(&d);
    assert_eq!(amount(&r, "cost_of_goods_sold"), 0);
    assert!(r.review_notes.iter().any(|n| n.display.key == "WARN_INVENTORY_LEFT_OUT"));
    assert_eq!(round_trip(&d), d);
}
#[test]
fn schedule_c_specials_f_header_returns_other_income_and_overtime() {
    let mut d = draft(50_000);
    let b = business(&mut d);
    let BusinessActivity::Ordinary { returns, .. } = &mut b.activity else { panic!() };
    *returns = 120_000;
    b.ein = "12-3456789".into();
    b.principal_business = "Consulting".into();
    b.business_code = Some(541990);
    b.other_income = vec![
        OtherBusinessIncome { description: "Scrap sales".into(), amount: 40_000 },
        OtherBusinessIncome { description: "Recovered bad debt".into(), amount: 30_000 },
    ];
    b.expenses =
        vec![expense(BusinessExpenseCategory::Other { label: "Operating costs".into() }, 9_500)];
    b.overtime = Some(FederalBusinessOvertime { amount: 300_000, eligible: true });
    add(
        &mut d,
        income(FederalIncome::Nec1099 {
            data: nec_source(FederalForm1099Nec {
                cash_tips: 0,
                tip_occupation_codes: Default::default(),
                overtime: 0,
                compensation: 2_000_000,
                withholding: 0,
                golden_parachute: 0,
                belongs: None,
            }),
        }),
    );
    d.income.last_mut().unwrap().label.clear();
    let r = supported(&d);
    for (suffix, expected) in [
        ("gross_receipts", 70_000),
        ("returns", 1_200),
        ("net_receipts", 68_800),
        ("gross_profit", 68_800),
        ("other_income", 700),
        ("gross_income", 69_500),
        ("net_profit", 60_000),
    ] {
        assert_eq!(amount(&r, suffix), expected, "{suffix}");
    }
    assert_eq!(value(&r, "overtime_deduction"), 3_000);
    assert_eq!(round_trip(&d), d);
}
#[test]
fn schedule_c_specials_g_working_interest_shared_depletion_limit() {
    let mut d = draft(60_000);
    let b = business(&mut d);
    b.expenses =
        vec![expense(BusinessExpenseCategory::Other { label: "Extraction costs".into() }, 35_000)];
    b.depletion = Some(BusinessDepletion {
        kind: RoyaltyKind::OilGas,
        depletion: RoyaltyDepletion::Percentage {
            adjusted_basis: 2_000_000,
            recoverable_units: 50_000,
            units_sold: 2_000,
            carryover: 0,
            amt_carryover: 0,
        },
    });
    let r = supported(&d);
    assert_eq!(amount(&r, "depletion"), 4_637);
    assert_eq!(
        amount(&r, "depletion_carryover"),
        4_363,
        "G diagnostic {:?}",
        r.lines
            .iter()
            .filter(|l| [
                "agi",
                "schedule_1_half_self_employment_tax",
                "self_employment_tax",
                "business_2_depletion",
                "business_2_depletion_carryover",
                "taxable_income"
            ]
            .contains(&l.id.as_str()))
            .map(|l| (&l.id, l.amount))
            .collect::<Vec<_>>()
    );
    assert_eq!(amount(&r, "net_profit"), 20_363);
    assert_eq!(value(&r, "form6251_line2d_depletion"), -4_363);
    assert!(r.review_notes.iter().any(|n| n.display.key == "NOTE_DEPLETION_65_PERCENT_LIMIT"));
    assert_eq!(round_trip(&d), d);
}
fn round_trip(d: &FederalDraft) -> FederalDraft {
    use libertas::{AvroDecode, AvroEncode};
    let mut bytes = vec![];
    d.avro_encode(&mut bytes);
    let mut offset = 0;
    let decoded = FederalDraft::avro_decode(&bytes, &mut offset).unwrap();
    assert_eq!(offset, bytes.len());
    assert_eq!(&decoded, d);
    let mut session = restart(decoded.clone());
    let request =
        crate::tests::cross::interview::open_topic(&mut session, TaxSection::SelfEmployment);
    crate::tests::cross::interview::accepted(&send(&mut session, request));
    let mut expected = decoded.clone();
    expected.revision = session.draft.revision;
    // The first save of the applying topic records the visit (and the walk
    // passes what it leaves empty); nothing else changes.
    expected.finished_sections = session.draft.finished_sections.clone();
    assert!(expected.finished_sections.contains(&TaxSection::SelfEmployment));
    // A visited business topic is also ticked as a planned activity.
    expected.checklist.activities |= ActivityTopics::BUSINESS.0;
    assert_eq!(session.draft, expected);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
    assert_eq!(supported(&session.draft).tax, supported(&decoded).tax);
    decoded
}

#[test]
fn schedule_c_specials_employee_lease_rows_and_dot_meals() {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().employee_expenses = vec![EmployeeExpenseClaim {
        improvement_buildings: Vec::new(),
        equipment_assets: vec![],
        dot_hours_of_service: true,
        leased_vehicles: vec![lease()],
        owner: Owner::Taxpayer,
        occupation: "Fee-basis official".into(),
        capacity: EmployeeCapacity::FeeBasisOfficial,
        employers: vec![1],
        local_transportation: 0,
        travel: 0,
        other: 0,
        meals: 100_000,
        reimbursements: 0,
        mileage: vec![],
        vehicle_assets: vec![],
    }];
    let r = supported(&d);
    for (line, amount) in [("24a", 5850), ("24b", 70), ("24c", 5780), ("29", 8330)] {
        assert_eq!(value(&r, &alloc::format!("employee_expenses_1_lease_1_line_{line}")), amount);
    }
    assert_eq!(value(&r, "employee_expenses_1_line_9"), 800);
    assert_eq!(value(&r, "employee_expenses_1_line_10"), 9130);
    let mut session = restart(d.clone());
    let request = crate::tests::cross::interview::go_to(
        &session.draft,
        crate::tests::cross::interview::PAGE_ADJUSTMENTS,
        TaxSection::EmployeeExpenses,
    );
    let Reply::Edit(request @ P::SaveEmployeeExpenses { .. }) = send(&mut session, request) else {
        panic!("expected Form2106 page")
    };
    crate::tests::cross::interview::accepted(&send(&mut session, request));
    assert_eq!(
        session.draft.adjustments.as_ref().unwrap().employee_expenses,
        d.adjustments.as_ref().unwrap().employee_expenses
    );
}

#[test]
fn schedule_c_lease_inclusion_reduces_only_the_lease_deduction() {
    // Pub.463, Leasing a Car: inclusion reduces the lease-payment deduction,
    // rather than adding income. At zero or $30 allocated payments, the $70
    // inclusion cannot create a negative deduction or consume operating costs.
    let mut d = draft(100_000);
    add(&mut d, wage(1, Owner::Taxpayer, 0, 0));
    let employer = d.income.last().unwrap().id;
    let mut v = lease();
    for payments in [0, 4_000] {
        v.lease_payments = payments;
        assert_eq!(v.amounts().inclusion, 7_000);
        assert_eq!(v.amounts().net_payments(), 0);
        assert_eq!(v.amounts().total(), 255_000);
        business(&mut d).leased_vehicles = vec![v.clone()];
        let r = supported(&d);
        assert!(
            !r.lines
                .iter()
                .any(|line| line.id == "business_2_expense_line20a_rent_vehicles_equipment")
        );
        assert_eq!(amount(&r, "car_and_truck"), 2_550);
        assert_eq!(amount(&r, "net_profit"), 97_450);
        d.adjustments.as_mut().unwrap().employee_expenses = vec![EmployeeExpenseClaim {
            improvement_buildings: Vec::new(),
            equipment_assets: vec![],
            dot_hours_of_service: false,
            leased_vehicles: vec![v.clone()],
            owner: Owner::Taxpayer,
            occupation: "Fee-basis official".into(),
            capacity: EmployeeCapacity::FeeBasisOfficial,
            employers: vec![employer],
            local_transportation: 0,
            travel: 0,
            other: 0,
            meals: 0,
            reimbursements: 0,
            mileage: vec![],
            vehicle_assets: vec![],
        }];
        let r = supported(&d);
        assert_eq!(value(&r, "employee_expenses_1_lease_1_line_24c"), 0);
        assert_eq!(value(&r, "employee_expenses_1_lease_1_line_29"), 2_550);
    }
}

#[test]
fn schedule_c_lease_term_and_calendar_bounds_match_the_typed_rules() {
    let mut v = lease();
    v.began = 20260101;
    v.ends = Some(20260129);
    assert_eq!(v.rule(), Some("LEASE_TERM_UNDER_30_DAYS"));
    v.ends = Some(20260130);
    assert!(v.valid());
    v.ends = Some(20251231);
    assert_eq!(v.rule(), Some("LEASE_ENDS_BEFORE_BEGAN"));
    v.ends = None;
    v.commuting = 5_000;
    assert_eq!(v.rule(), Some("VEHICLE_BUSINESS_MILES_ABOVE_TOTAL"));
}

#[test]
fn schedule_c_de_minimis_election_uses_return_wide_afs_facts() {
    let mut d = draft(90_000);
    business(&mut d).de_minimis = Some(DeMinimisElection {
        purchases: 200_000,
        largest_item: 200_000,
        financial_statement: false,
        eligible: true,
    });
    let mut rental = rental(12_000, 0);
    rental.expenses = vec![crate::extended_model::RentalExpense {
        category: crate::extended_model::RentalExpenseCategory::Other {
            label: "Book-expensed tools".into(),
        },
        amount: 100_000,
        treatment: BusinessExpenseKind::DeMinimis {
            largest_cost: 100_000,
            financial_statement: true,
            eligible: true,
        },
    }];
    add(&mut d, income(FederalIncome::Rental { data: rental }));
    let r = estimate(&d);
    assert!(r.issues.iter().any(|i| i.display.key
        == "ISSUE_USE_CONSISTENT_APPLICABLE_FINANCIAL_STATEMENT_FACTS_FOR_THIS_ACTIVI_5F5D61F3"));
    business(&mut d).de_minimis.as_mut().unwrap().financial_statement = true;
    supported(&d);
}

#[test]
fn schedule_c_depletion_is_separate_from_the_royalty_schedule_e_owner() {
    let mut d = draft(60_000);
    business(&mut d).expenses =
        vec![expense(BusinessExpenseCategory::Other { label: "Extraction".into() }, 35_000)];
    business(&mut d).depletion = Some(BusinessDepletion {
        kind: RoyaltyKind::OilGas,
        depletion: RoyaltyDepletion::Percentage {
            adjusted_basis: 2_000_000,
            recoverable_units: 50_000,
            units_sold: 2_000,
            carryover: 0,
            amt_carryover: 0,
        },
    });
    add(&mut d, wage(1, Owner::Taxpayer, 5_000_000, 0));
    let r = supported(&d);
    assert_eq!(amount(&r, "depletion"), 9_000);
    assert!(!r.lines.iter().any(|line| line.id.starts_with("schedule_e_line23")));
    add(
        &mut d,
        income(FederalIncome::Royalty {
            data: FederalRoyalty {
                related: alloc::vec![],
                kind: RoyaltyKind::Mineral { rate: RoyaltyMineralRate::Ten },
                receipts: 2_000_000,
                advance: 0,
                expenses: vec![crate::extended_model::RentalExpense {
                    category: crate::extended_model::RentalExpenseCategory::Other {
                        label: "Property costs".into(),
                    },
                    amount: 300_000,
                    treatment: BusinessExpenseKind::Ordinary,
                }],
                depletion: RoyaltyDepletion::Percentage {
                    adjusted_basis: 80_000,
                    recoverable_units: 100_000,
                    units_sold: 5_000,
                    carryover: 0,
                    amt_carryover: 0,
                },
                ordinary: true,
            },
        }),
    );
    let id = d.income.last().unwrap().id;
    let r = supported(&d);
    assert_eq!(amount(&r, "depletion"), 9_000);
    assert_eq!(value(&r, &alloc::format!("royalty_{id}_line18_depletion")), 2_000);
    assert_eq!(value(&r, &alloc::format!("royalty_{id}_line21_income")), 15_000);
    assert_eq!(value(&r, "agi"), 79_869); // 50000 +16000 +15000 -1131 half-SE.
}
