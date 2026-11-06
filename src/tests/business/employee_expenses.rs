// Form 2106 (employee_expenses.rs): the calculation by hand (fixtures F1-F4 and
// the performing-artist boundary variants), the W-2 code L import, the page's
// mapping and its server twins, and the Search destination text.
// Whole-return fixtures F1-F3 also run as data in tests/returns.rs.
use super::*;
use crate::federal_model::NoteIdentity;
use crate::tests::cross::interview::{PAGE_REVIEW, accepted, go_to, refusal, shown};

const ADJUSTMENTS: i32 = TaxSection::Adjustments as i32;

// A W-2 record with box 12 code L, when given. Amounts are cents.
fn w2_entry(id: i64, owner: Owner, label: &str, wages: i64, code_l: i64) -> FederalIncomeEntry {
    let mut entry = wage(id, owner, wages, wages / 20);
    entry.label = label.into();
    if code_l > 0
        && let FederalIncome::Wage { data } = &mut entry.income
    {
        let mut data = edit_wage(data);
        data.other_box12.push(W2CodeAmount { code: W2Code::L, amount: code_l });
    }
    entry
}
fn bank_interest(amount: i64) -> FederalIncomeEntry {
    income(FederalIncome::Interest { data: interest_source(native_interest(amount, 0)) })
}
fn claim(
    owner: Owner,
    occupation: &str,
    capacity: EmployeeCapacity,
    employers: &[i64],
) -> EmployeeExpenseClaim {
    EmployeeExpenseClaim {
        improvement_buildings: Vec::new(),
        equipment_assets: vec![],
        dot_hours_of_service: false,
        leased_vehicles: vec![],
        owner,
        occupation: occupation.into(),
        capacity,
        employers: employers.to_vec(),
        local_transportation: 0,
        travel: 0,
        other: 0,
        meals: 0,
        reimbursements: 0,
        mileage: vec![],
        vehicle_assets: vec![],
    }
}
fn with_claims(mut d: FederalDraft, claims: Vec<EmployeeExpenseClaim>) -> FederalDraft {
    d.adjustments.as_mut().unwrap().employee_expenses = claims;
    d
}
// F2: Stage actor, two W-2s (Example Theater box 1 $9,000 with code L $300, Example
// Opera box 1 `opera`), a 1099-INT of `interest`, travel $600, other `other`, meals $200.
fn artist_return(status: FederalStatus, interest: i64, opera: i64, other: i64) -> FederalDraft {
    let mut d = complete(status);
    d.income = vec![
        w2_entry(1, Owner::Taxpayer, "Example Theater", 900_000, 30_000),
        w2_entry(2, Owner::Taxpayer, "Example Opera", opera, 0),
    ];
    d.next_id = 3;
    add(&mut d, bank_interest(interest));
    let mut form =
        claim(Owner::Taxpayer, "Stage actor", EmployeeCapacity::PerformingArtist, &[1, 2]);
    form.travel = 60_000;
    form.other = other;
    form.meals = 20_000;
    with_claims(d, vec![form])
}
fn artist_lines(d: &FederalDraft) -> crate::employee_expenses::Lines {
    d.adjustments.as_ref().unwrap().employee_expenses[0].lines(d, false)
}

#[test]
fn reservist_travel_and_standard_mileage_reach_schedule_1_line_12() {
    // F1: Army Reserve sergeant. Standard mileage 1,200 miles January-June and 800
    // July-December, 9,000 miles in all, parking and tolls $40; travel $900; meals $300.
    //   Line 22 = 1,200 x 72.5 cents + 800 x 76 cents = 870.00 + 608.00 = 1,478.00 -> line 1.
    //   Line 2 = 0 + 40 = 40.00. Line 3 = 900.00. Line 5 = 300.00.
    //   6A = 1,478 + 40 + 900 + 0 = 2,418.00; 6B = 300.00; line 7 = 0.
    //   9A = 2,418.00; 9B = 50% x 300.00 = 150.00; line 10 = 2,568.00.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![
        w2_entry(1, Owner::Taxpayer, "Example Logistics", 5_800_000, 0),
        w2_entry(2, Owner::Taxpayer, "State Guard Pay", 420_000, 0),
    ];
    d.next_id = 3;
    let (mileage, _) = crate::interview::vehicle_and_asset_records(
        &[Vehicle {
            method: VehicleMethod::StandardMileage {
                id: 0,
                first_half: 1_200,
                second_half: 800,
                total: 9_000,
                more: MileageMore {
                    placed: None,
                    personal_use_available: true,
                    other_vehicle: true,
                    evidence: VehicleEvidence::Written,
                    commuting: 0,
                    parking_tolls: 4_000,
                    interest: 0,
                    property_tax: 0,
                    leased: false,
                },
            },
        }],
        &[],
    );
    let mut form =
        claim(Owner::Taxpayer, "Army Reserve sergeant", EmployeeCapacity::Reservist, &[2]);
    form.travel = 90_000;
    form.meals = 30_000;
    form.mileage = mileage;
    let d = with_claims(d, vec![form]);
    let lines = d.adjustments.as_ref().unwrap().employee_expenses[0].lines(&d, false);
    assert_eq!(
        (lines.line1, lines.line2, lines.line3, lines.line5, lines.line7, lines.line10),
        (147_800, 4_000, 90_000, 30_000, 0, 256_800)
    );
    let r = supported(&d);
    assert_eq!(value(&r, "employee_expenses_1_line_1"), 1_478);
    assert_eq!(value(&r, "employee_expenses_1_line_2"), 40);
    assert_eq!(value(&r, "employee_expenses_1_line_10"), 2_568);
    assert_eq!(value(&r, "schedule_1_employee_business_expenses"), 2_568);
    // Wages 62,200 less Schedule 1 line 12 of 2,568.
    assert_eq!(value(&r, "wages"), 62_200);
    assert_eq!(value(&r, "agi"), 59_632);
    assert_eq!(
        printed_line(&r, "schedule_1_employee_business_expenses"),
        (TaxForm::Schedule1, "12".into())
    );
    assert_eq!(printed_line(&r, "employee_expenses_1_line_10"), (TaxForm::Form2106, "10".into()));
    // The mileage basis reduction (2,000 x 35 cents = 700) is a later-sale fact.
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn performing_artist_at_the_16000_boundary_deducts_the_allocated_amount() {
    // F2: AGI before line 12 = 9,000 + 6,500 + 500 = 16,000.00, exactly the limit.
    //   6A = 0 + 0 + 600 + 1,400 = 2,000.00; 6B = 200.00; line 7 = code L 300.00.
    //   Ratio = 200 / 2,200 = 0.0909 -> 0.091; 7B = 300 x 0.091 = 27.30; 7A = 272.70.
    //   8A = 1,727.30; 8B = 172.70; 9B = 86.35; line 10 = 1,813.65 -> 1,814.
    let d = artist_return(FederalStatus::Single, 50_000, 650_000, 140_000);
    let lines = artist_lines(&d);
    assert_eq!(
        (lines.line7, lines.line8a, lines.line8b, lines.line10),
        (30_000, 172_730, 17_270, 181_365)
    );
    assert_eq!((lines.gross, lines.paying_200), (1_550_000, 2));
    let r = supported(&d);
    assert_eq!(value(&r, "schedule_1_employee_business_expenses"), 1_814);
    assert_eq!(value(&r, "agi"), 14_186);
    assert!(!r.review_notes.iter().any(|n| n.display.key.starts_with("WARN_PERFORMING_ARTIST")));
}

#[test]
fn performing_artist_agi_one_dollar_above_the_limit_leaves_the_form_out() {
    // Interest $501 makes AGI 16,001: the section 62(b)(1)(C) test fails, line 12
    // is 0, AGI stays 16,001 and the estimate proceeds with one note.
    let d = artist_return(FederalStatus::Single, 50_100, 650_000, 140_000);
    let r = supported(&d);
    assert!(noted(&r, "WARN_PERFORMING_ARTIST_AGI"));
    assert_eq!(
        r.review_notes
            .iter()
            .filter(|n| n.display.key.starts_with("WARN_PERFORMING_ARTIST"))
            .count(),
        1
    );
    assert!(!r.lines.iter().any(|l| l.id == "schedule_1_employee_business_expenses"));
    assert_eq!(value(&r, "agi"), 16_001);
    assert_eq!(value(&r, "employee_expenses_1_line_10"), 0);
    // The note names the occupation and the amount left out, and links to the page.
    let note =
        r.review_notes.iter().find(|n| n.display.key == "WARN_PERFORMING_ARTIST_AGI").unwrap();
    assert_eq!(
        note.display.args,
        vec![MessageArg::Text("Stage actor".into()), MessageArg::usd_dollars(1_814)]
    );
    assert_eq!(
        note.context.as_ref().unwrap().identity(),
        NoteIdentity::Item {
            parent: alloc::boxed::Box::new(NoteIdentity::Owner {
                section: TaxSection::EmployeeExpenses,
                owner: Owner::Taxpayer
            }),
            collection: "employee_expenses",
            position: 0
        }
    );
}

#[test]
fn performing_artist_needs_two_employers_paying_200() {
    // Example Opera box 1 $199.00: only one chosen W-2 pays $200 or more. AGI is
    // 9,000 + 199 + 500 = 9,699, and 10% of 9,199 = 919.90 is below the expenses.
    let d = artist_return(FederalStatus::Single, 50_000, 19_900, 140_000);
    assert_eq!(artist_lines(&d).paying_200, 1);
    let r = supported(&d);
    assert!(noted(&r, "WARN_PERFORMING_ARTIST_EMPLOYERS"));
    assert!(!noted(&r, "WARN_PERFORMING_ARTIST_TEN_PERCENT"));
    assert!(!noted(&r, "WARN_PERFORMING_ARTIST_AGI"));
    assert_eq!(value(&r, "agi"), 9_699);
    // $200.00 exactly passes the test.
    let d = artist_return(FederalStatus::Single, 50_000, 20_000, 140_000);
    assert_eq!(artist_lines(&d).paying_200, 2);
    assert!(!noted(&supported(&d), "WARN_PERFORMING_ARTIST_EMPLOYERS"));
}

#[test]
fn performing_artist_expenses_must_exceed_ten_percent_of_box_1() {
    // Other expenses $1,100: 6A = 1,700.00, ratio 200 / 1,900 = 0.105, 7B = 31.50,
    // 7A = 268.50, 8A = 1,431.50, 8B = 168.50, line 10 = 1,431.50 + 84.25 =
    // 1,515.75, which is not above 10% x 15,500 = 1,550.
    let d = artist_return(FederalStatus::Single, 50_000, 650_000, 110_000);
    let lines = artist_lines(&d);
    assert_eq!(
        (lines.line7, lines.line8a, lines.line8b, lines.line10),
        (30_000, 143_150, 16_850, 151_575)
    );
    let r = supported(&d);
    assert!(noted(&r, "WARN_PERFORMING_ARTIST_TEN_PERCENT"));
    assert_eq!(value(&r, "agi"), 16_000);
    // Exactly 10% fails too (the test says more than): Example Opera box 1 of
    // $2,636.50 makes the gross 11,636.50 ... the closed form is simply the
    // boundary at one cent either side of line 10 = 10% of box 1.
    let mut d = artist_return(FederalStatus::Single, 50_000, 650_000, 140_000);
    let line10 = artist_lines(&d).line10;
    assert_eq!(line10, 181_365);
    // Box 1 of 18,136.50 in all: Example Opera 9,136.50 -> 10% = 1,813.65 = line 10.
    let FederalIncome::Wage { data } = &mut d.income[1].income else { unreachable!() };
    let mut data = edit_wage(data);
    data.wages = 913_650;
    drop(data);
    assert_eq!(artist_lines(&d).gross, 1_813_650);
    assert_eq!(artist_lines(&d).line10 * 10, artist_lines(&d).gross);
    assert!(noted(&estimate(&d), "WARN_PERFORMING_ARTIST_TEN_PERCENT"));
    let FederalIncome::Wage { data } = &mut d.income[1].income else { unreachable!() };
    let mut data = edit_wage(data);
    data.wages = 913_649;
    drop(data);
    assert!(!noted(&estimate(&d), "WARN_PERFORMING_ARTIST_TEN_PERCENT"));
}

#[test]
fn married_performing_artist_files_jointly_or_lived_apart_all_year() {
    // Married filing separately, living together: the separate-return test fails.
    let d = artist_return(FederalStatus::Separate, 50_000, 650_000, 140_000);
    let r = supported(&d);
    assert!(noted(&r, "WARN_PERFORMING_ARTIST_SEPARATE_RETURN"));
    assert_eq!(value(&r, "agi"), 16_000);
    // Lived apart all year: the same return deducts 1,814.
    let mut apart = d.clone();
    apart.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::AllYear };
    let r = supported(&apart);
    assert!(!noted(&r, "WARN_PERFORMING_ARTIST_SEPARATE_RETURN"));
    assert_eq!(value(&r, "schedule_1_employee_business_expenses"), 1_814);
    // A joint return tests the combined AGI: the spouse's $1,000 W-2 makes it 17,000.
    let mut joint = artist_return(FederalStatus::Joint, 50_000, 650_000, 140_000);
    add(&mut joint, w2_entry(9, Owner::Spouse, "Spouse Co", 100_000, 0));
    let r = supported(&joint);
    assert!(noted(&r, "WARN_PERFORMING_ARTIST_AGI"));
    assert_eq!(value(&r, "agi"), 17_000);
}

#[test]
fn fee_basis_official_excess_reimbursement_is_wage_income() {
    // F3: standard mileage 2,000 miles July-December (76 cents) = 1,520.00; line 2 =
    // 100.00 + 30.00 parking = 130.00; line 4 = 250.00; 6A = 1,900.00.
    // Code L 2,000.00 with ratio 0 / 1,900 = 0: 7A = 2,000.00, 8A = 0, the excess is
    // 100.00, wage income on line 1a; line 10 = 0.
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![
        w2_entry(1, Owner::Taxpayer, "Example County Court", 5_200_000, 200_000),
        w2_entry(2, Owner::Spouse, "Example Co", 3_000_000, 0),
    ];
    d.next_id = 3;
    let (mileage, _) = crate::interview::vehicle_and_asset_records(
        &[Vehicle {
            method: VehicleMethod::StandardMileage {
                id: 0,
                first_half: 0,
                second_half: 2_000,
                total: 11_000,
                more: MileageMore {
                    placed: None,
                    personal_use_available: true,
                    other_vehicle: true,
                    evidence: VehicleEvidence::Written,
                    commuting: 2_400,
                    parking_tolls: 3_000,
                    interest: 0,
                    property_tax: 0,
                    leased: false,
                },
            },
        }],
        &[],
    );
    let mut form = claim(
        Owner::Taxpayer,
        "Fee-basis county magistrate",
        EmployeeCapacity::FeeBasisOfficial,
        &[1],
    );
    form.local_transportation = 10_000;
    form.other = 25_000;
    form.mileage = mileage;
    let d = with_claims(d, vec![form]);
    let lines = d.adjustments.as_ref().unwrap().employee_expenses[0].lines(&d, false);
    assert_eq!(
        (lines.line1, lines.line2, lines.line7, lines.line8a, lines.excess, lines.line10),
        (152_000, 13_000, 200_000, 0, 10_000, 0)
    );
    let r = supported(&d);
    assert_eq!(value(&r, "wages"), 82_100);
    assert_eq!(value(&r, "agi"), 82_100);
    assert_eq!(value(&r, "employee_expenses_1_excess_reimbursement"), 100);
    assert!(!r.lines.iter().any(|l| l.id == "schedule_1_employee_business_expenses"));
    // The note is a correction toward the form or the W-2.
    let note = r
        .review_notes
        .iter()
        .find(|n| n.display.key == "NOTE_EMPLOYEE_EXPENSES_EXCESS_REIMBURSEMENT")
        .unwrap();
    assert_eq!(
        note.display.args,
        vec![MessageArg::Text("Fee-basis county magistrate".into()), MessageArg::usd_dollars(100)]
    );
    let Some(IssueContext::Corrective { sections, .. }) = &note.context else {
        panic!("a corrective note, got {:?}", note.context);
    };
    assert_eq!(sections, &vec![TaxSection::EmployeeExpenses, TaxSection::Wages]);
    // Reimbursements entered with no W-2 code L reduce the expenses the same way.
    let mut d = d.clone();
    d.income[0] = w2_entry(1, Owner::Taxpayer, "Example County Court", 5_200_000, 0);
    let form = &mut d.adjustments.as_mut().unwrap().employee_expenses[0];
    form.reimbursements = 150_000;
    // 7A = 1,500.00; 8A = 1,900 - 1,500 = 400.00; no excess.
    let lines = d.adjustments.as_ref().unwrap().employee_expenses[0].lines(&d, false);
    assert_eq!((lines.line8a, lines.excess, lines.line10), (40_000, 0, 40_000));
    assert_eq!(value(&supported(&d), "schedule_1_employee_business_expenses"), 400);
}

// A page vehicle with actual expenses, placed March 15, 2026: cost $40,000, 10,000
// miles of which 8,000 are business, operating costs $3,000.
fn actual_page_vehicle(first_year: FirstYearDeduction) -> Vehicle {
    Vehicle {
        method: VehicleMethod::Actual {
            macrs_source: None,
            id: 0,
            history_details: None,
            amt_section179_income_carry: None,
            allowed_amt_carry: Vec::new(),
            cost: 4_000_000,
            credit_reduction: 0,
            acquired: None,
            placed: 20260315,
            total_miles: 10_000,
            business_miles: 8_000,
            operating_costs: 300_000,
            loan_interest: 0,
            kind: crate::VehicleKind::PassengerCar,
            history: AssetHistory::New { first_year },
            disposal: AssetDisposal::No,
            earlier_business_percent: None,
        },
    }
}
fn vehicle_return(vehicle: Vehicle) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![w2_entry(1, Owner::Taxpayer, "County", 6_000_000, 0)];
    d.next_id = 2;
    let (mileage, vehicle_assets) = crate::interview::vehicle_and_asset_records(&[vehicle], &[]);
    let mut form =
        claim(Owner::Taxpayer, "County assessor", EmployeeCapacity::FeeBasisOfficial, &[1]);
    form.mileage = mileage;
    form.vehicle_assets = vehicle_assets;
    with_claims(d, vec![form])
}

#[test]
fn a_vehicle_credit_reduces_the_actual_expense_vehicle_basis() {
    // Cost 20,000 less a clean-vehicle credit of 5,000 (sections 30D(f)(1),
    // 45W(d)(1)): basis 15,000, 80% business (8,000 of 10,000 miles) = 12,000.
    //   Bonus 100% = 12,000.00, below the 280F first-year cap (20,300 x 80% =
    //   16,240.00), so line 28 = 12,000.00 (16,000.00 without the credit).
    //   Line 27 = 3,000 x 80% = 2,400.00; line 1 = 14,400.00.
    let vehicle = |credit| Vehicle {
        method: VehicleMethod::Actual {
            macrs_source: None,
            id: 0,
            history_details: None,
            amt_section179_income_carry: None,
            allowed_amt_carry: Vec::new(),
            cost: 2_000_000,
            credit_reduction: credit,
            acquired: None,
            placed: 20260315,
            total_miles: 10_000,
            business_miles: 8_000,
            operating_costs: 300_000,
            loan_interest: 0,
            kind: crate::VehicleKind::PassengerCar,
            history: AssetHistory::New { first_year: FirstYearDeduction::Bonus },
            disposal: AssetDisposal::No,
            earlier_business_percent: None,
        },
    };
    let without = vehicle_return(vehicle(0));
    let lines = without.adjustments.as_ref().unwrap().employee_expenses[0].lines(&without, false);
    assert_eq!(lines.depreciation, vec![1_600_000]);
    let d = vehicle_return(vehicle(500_000));
    let lines = d.adjustments.as_ref().unwrap().employee_expenses[0].lines(&d, false);
    assert_eq!(lines.operating, vec![240_000]);
    assert_eq!(lines.depreciation, vec![1_200_000]);
    assert_eq!(lines.line1, 1_440_000);
    let r = supported(&d);
    assert_eq!(value(&r, "schedule_1_employee_business_expenses"), 14_400);
    // The page record maps back with its credit.
    let stored = &d.adjustments.as_ref().unwrap().employee_expenses[0].vehicle_assets[0];
    assert_eq!(stored.credit_reduction, 500_000);
    assert_eq!(crate::interview::vehicle_and_asset_records(&[vehicle(500_000)], &[]).1[0], *stored);
}

#[test]
fn actual_expense_vehicle_uses_the_2026_280f_table_on_the_business_share() {
    // F4: cost 40,000, 8,000 of 10,000 miles (80%), operating costs 3,000, bonus.
    //   Line 27 = 3,000 x 80% = 2,400.00.
    //   Line 31 bonus = 100% x 32,000 = 32,000 against line 37 = 20,300 x 80% =
    //   16,240.00 (Rev. Proc. 2026-15 Table 1; the 2025 table would give 16,160).
    //   Line 28 = 16,240.00; line 29 = 18,640.00 -> line 1.
    let d = vehicle_return(actual_page_vehicle(FirstYearDeduction::Bonus));
    let lines = d.adjustments.as_ref().unwrap().employee_expenses[0].lines(&d, false);
    assert_eq!(lines.operating, vec![240_000]);
    assert_eq!(lines.depreciation, vec![1_624_000]);
    assert_eq!(lines.line1, 1_864_000);
    // Fee-basis: line 10 = 18,640 (no meals), a Schedule 1 line 12 deduction.
    let r = supported(&d);
    assert_eq!(value(&r, "employee_expenses_1_line_1"), 18_640);
    assert_eq!(value(&r, "employee_expenses_1_vehicle_1_operating"), 2_400);
    assert_eq!(value(&r, "employee_expenses_1_vehicle_1_depreciation"), 16_240);
    assert_eq!(value(&r, "schedule_1_employee_business_expenses"), 18_640);
    assert_eq!(
        printed_line(&r, "employee_expenses_1_vehicle_1_depreciation"),
        (TaxForm::Form2106, "28".into())
    );
    assert_eq!(restart(d.clone()).draft, d);
    // Without bonus: MACRS 5-year half-year, 32,000 x 20% = 6,400 under the
    // 12,300 x 80% = 9,840 first-year limit. A new post-2017 vehicle uses the same
    // method for AMT, so the regular-AMT difference (as a Schedule C asset's) is 0.
    let d = vehicle_return(actual_page_vehicle(FirstYearDeduction::Regular));
    let lines = d.adjustments.as_ref().unwrap().employee_expenses[0].lines(&d, false);
    assert_eq!(lines.depreciation, vec![640_000]);
    let mut r = FederalResult::empty(0);
    let calculation = crate::employee_expenses::calculate(&d, &mut r);
    assert_eq!(calculation.amt_adjustment(&d), 0);
}

#[test]
fn section_179_on_an_employee_vehicle_is_left_out_with_a_note() {
    // The elected $10,000 is left out; the vehicle takes its bonus (16,240.00 under
    // the 280F limit), as F4 does, and one WARN names the occupation.
    let d = vehicle_return(actual_page_vehicle(FirstYearDeduction::Section179 {
        description: "County car".into(),
        elected: 1_000_000,
    }));
    let r = supported(&d);
    assert!(noted(&r, "WARN_EMPLOYEE_VEHICLE_179_LEFT_OUT"));
    assert_eq!(value(&r, "employee_expenses_1_vehicle_1_depreciation"), 16_240);
    // No section 179 reaches the return-wide Form 4562 allocation.
    assert!(!r.lines.iter().any(|l| l.id.contains("section179")));
}

#[test]
fn employee_vehicle_sale_takes_the_business_asset_rows_and_unsupported_history_is_refused() {
    // A $40,000 vehicle (80% business) placed 2026-03-15 and sold 2026-12-01 for
    // $20,000 takes no 2026 depreciation (placed and disposed of in the year). The
    // business part: net price $16,000 against $32,000 of cost: a $16,000 loss on
    // Part II line 11 (held one year or less), no self-employment tax; the personal
    // part's $4,000 against $8,000 is not deductible (Pub. 463).
    let mut d = vehicle_return(actual_page_vehicle(FirstYearDeduction::Regular));
    let form = &mut d.adjustments.as_mut().unwrap().employee_expenses[0];
    form.vehicle_assets[0].disposal = AssetDisposal::Disposed {
        date: 20261201,
        how: crate::DisposalKind::Sold {
            livestock: crate::SoldLivestock::No,
            proceeds: 2_000_000,
            selling_expenses: 0,
            payment: crate::SalePayment::InFull,
            buyer: crate::Buyer::Unrelated,
        },
    };
    let r = supported(&d);
    assert_eq!(value(&r, "employee_expenses_1_vehicle_1_disposal_ordinary"), -16_000);
    assert_eq!(value(&r, "employee_expenses_1_vehicle_1_depreciation"), 0);
    assert_eq!(
        printed_line(&r, "employee_expenses_1_vehicle_1_disposal_ordinary"),
        (TaxForm::Form4797, "11".into())
    );
    // A passenger automobile placed before 2025 is outside the supported tables.
    let mut d = vehicle_return(actual_page_vehicle(FirstYearDeduction::Bonus));
    let form = &mut d.adjustments.as_mut().unwrap().employee_expenses[0];
    form.vehicle_assets[0].placed = 20240601;
    form.vehicle_assets[0].acquired = 20240601;
    form.vehicle_assets[0].history = EquipmentHistory::Continuing {
        initial_section179: 0,
        carryforward: 0,
        prior_depreciation: 800_000,
        amt: EquipmentAmtHistory::Same,
        mid_quarter: false,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    unsupported(&d, "Form 2106 vehicle");
}

#[test]
fn employee_vehicles_join_the_return_wide_mid_quarter_test() {
    // A vehicle placed November 5, 2026 is the only 2026 depreciable basis, so all
    // of it is in the fourth quarter (more than 40%): mid-quarter applies.
    let mut d = vehicle_return(actual_page_vehicle(FirstYearDeduction::Regular));
    assert!(!crate::depreciation::mid_quarter(&d));
    let form = &mut d.adjustments.as_mut().unwrap().employee_expenses[0];
    form.vehicle_assets[0].placed = 20261105;
    form.vehicle_assets[0].acquired = 20261105;
    assert!(crate::depreciation::mid_quarter(&d));
    // Without the claim the draft has no depreciable asset.
    d.adjustments.as_mut().unwrap().employee_expenses.clear();
    assert!(!crate::depreciation::mid_quarter(&d));
}

#[test]
fn employee_vehicle_interest_property_tax_and_reservist_rules_hold_in_storage() {
    let ok = {
        let d = vehicle_return(actual_page_vehicle(FirstYearDeduction::Bonus));
        d.adjustments.as_ref().unwrap().employee_expenses[0].clone()
    };
    assert!(ok.valid());
    // A reservist deducts only the standard mileage rate.
    let mut reservist = ok.clone();
    reservist.capacity = EmployeeCapacity::Reservist;
    assert!(!reservist.valid());
    // Loan interest on an actual-expense vehicle and on a mileage vehicle.
    let mut interest = ok.clone();
    let ListedProperty::Vehicle { loan_interest, .. } = &mut interest.vehicle_assets[0].listed
    else {
        unreachable!()
    };
    *loan_interest = 100;
    assert!(!interest.valid());
    let mut mileage = ok.clone();
    let (m, _) = crate::interview::vehicle_and_asset_records(
        &[Vehicle {
            method: VehicleMethod::StandardMileage {
                id: 0,
                first_half: 100,
                second_half: 0,
                total: 1_000,
                more: MileageMore {
                    placed: None,
                    personal_use_available: true,
                    other_vehicle: true,
                    evidence: VehicleEvidence::Written,
                    commuting: 0,
                    parking_tolls: 0,
                    interest: 100,
                    property_tax: 0,
                    leased: false,
                },
            },
        }],
        &[],
    );
    mileage.mileage = m;
    assert!(!mileage.valid());
    // An invalid stored claim makes the saved return invalid rather than computed.
    let d = with_claims(complete(FederalStatus::Single), vec![reservist]);
    assert!(!d.valid_stored());
}

#[test]
fn missing_employer_is_a_correction_item_not_zero() {
    let mut d = artist_return(FederalStatus::Single, 50_000, 650_000, 140_000);
    d.adjustments.as_mut().unwrap().employee_expenses[0].employers = vec![1, 77];
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.section == "Form 2106"));
    // A W-2 of the other spouse is not this owner's.
    let mut joint = artist_return(FederalStatus::Joint, 50_000, 650_000, 140_000);
    add(&mut joint, w2_entry(9, Owner::Spouse, "Spouse Co", 100_000, 0));
    joint.adjustments.as_mut().unwrap().employee_expenses[0].employers = vec![1, 9];
    assert_eq!(estimate(&joint).state, ResultState::Unsupported);
}

// ---- the page, its server twins and the W-2 code L import ----

// A FormattedText status: its resource key and argument count.
fn status_key(value: &[u8]) -> (alloc::string::String, usize) {
    use libertas::AvroDecode;
    let mut offset = 0;
    let name = crate::common::FormattedMessage::avro_decode(value, &mut offset)
        .expect("a FormattedText status");
    (name.0, name.1.len())
}
fn open_page(s: &mut Interview) -> P {
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::EmployeeExpenses);
    match send(s, request) {
        Reply::Edit(page @ P::SaveEmployeeExpenses { .. }) => page,
        other => panic!("expected the Form 2106 page, got {other:?}"),
    }
}
fn save_page(s: &Interview, template: &P, forms: Vec<EmployeeExpenseForm>) -> P {
    let P::SaveEmployeeExpenses { allowed_owners, value, .. } = template else { unreachable!() };
    P::SaveEmployeeExpenses {
        rental_energy_buildings: Vec::new(),
        initial_asset_amt_carry_choices: Vec::new(),
        user: TEST_USER,
        cookie: s.draft.cookie.clone(),
        revision: s.draft.revision,
        page: ADJUSTMENTS,
        previous: ADJUSTMENTS,
        review: true,
        estimate: None,
        warnings: vec![],
        progress: vec![],
        allowed_owners: allowed_owners.clone(),
        value: EmployeeExpenses { forms, wages: value.wages.clone() },
    }
}
fn page_form(
    capacity: EmployeeCapacity,
    employers: &[i64],
    vehicles: Vec<Vehicle>,
) -> EmployeeExpenseForm {
    EmployeeExpenseForm {
        improvement_buildings: Vec::new(),
        equipment_assets: vec![],
        dot_hours_of_service: false,
        owner: Owner::Taxpayer,
        occupation: "Army Reserve sergeant".into(),
        capacity,
        employers: employers.to_vec(),
        local_transportation: 0,
        travel: 90_000,
        other: 0,
        meals: 30_000,
        reimbursements: 0,
        vehicles,
    }
}
fn mileage_vehicle(interest: i64) -> Vehicle {
    Vehicle {
        method: VehicleMethod::StandardMileage {
            id: 0,
            first_half: 1_200,
            second_half: 800,
            total: 9_000,
            more: MileageMore {
                placed: None,
                personal_use_available: true,
                other_vehicle: true,
                evidence: VehicleEvidence::Written,
                commuting: 0,
                parking_tolls: 4_000,
                interest,
                property_tax: 0,
                leased: false,
            },
        },
    }
}

#[test]
fn form_2106_page_maps_the_form_its_vehicles_and_its_employers() {
    let mut d = complete(FederalStatus::Single);
    d.income.push(w2_entry(2, Owner::Taxpayer, "State Guard Pay", 420_000, 0));
    d.next_id = 3;
    let mut s = Interview::new(TEST_USER, d);
    // The page offers every W-2 of the return, labelled by employer and owner.
    let page = open_page(&mut s);
    let P::SaveEmployeeExpenses { value, .. } = &page else { unreachable!() };
    assert!(value.forms.is_empty());
    assert_eq!(value.wages.iter().map(|w| w.id).collect::<Vec<_>>(), vec![1, 2]);
    assert_eq!(shown(&value.wages[1].label), "State Guard Pay — You");
    // A reservist form with a standard-mileage vehicle and the chosen W-2 is stored.
    let form = page_form(EmployeeCapacity::Reservist, &[2], vec![mileage_vehicle(0)]);
    let request = save_page(&s, &page, vec![form.clone()]);
    accepted(&send(&mut s, request));
    let stored = &s.draft.adjustments.as_ref().unwrap().employee_expenses;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].capacity, EmployeeCapacity::Reservist);
    assert_eq!(stored[0].employers, vec![2]);
    assert_eq!(stored[0].mileage.len(), 1);
    assert!(stored[0].vehicle_assets.is_empty());
    assert_eq!((stored[0].travel, stored[0].meals), (90_000, 30_000));
    // Reopening shows the same form (employers restored by identity).
    let P::SaveEmployeeExpenses { value, .. } = open_page(&mut s) else { unreachable!() };
    assert_eq!(value.forms, vec![form]);
    // The topic is a hub row counting its forms, and the stored return round-trips.
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
    let hub = go_to(&s.draft, PAGE_REVIEW, TaxSection::Adjustments);
    let Reply::Response(hub) = send(&mut s, hub) else { panic!("expected the adjustments hub") };
    let P::Hub { topics, .. } = &hub else { panic!("expected the adjustments hub, got {hub:?}") };
    let row = topics
        .items
        .iter()
        .find(|r| r.section == TaxSection::EmployeeExpenses)
        .expect("a Form 2106 row");
    assert_eq!(status_key(&row.status), ("REVIEW_FORMS".into(), 1));
    // Deleting the W-2 drops the pick on reopen and the estimate reports it.
    s.draft.income.retain(|e| e.id != 2);
    let P::SaveEmployeeExpenses { value, .. } = open_page(&mut s) else { unreachable!() };
    assert!(value.forms[0].employers.is_empty());
    assert_eq!(estimate(&s.draft).state, ResultState::Unsupported);
}

#[test]
fn form_2106_capacity_is_one_required_choice_of_three() {
    // No impairment-related capacity and no default: each of the three stores.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let page = open_page(&mut s);
    for capacity in [
        EmployeeCapacity::Reservist,
        EmployeeCapacity::PerformingArtist,
        EmployeeCapacity::FeeBasisOfficial,
    ] {
        let request = save_page(&s, &page, vec![page_form(capacity, &[1], vec![])]);
        accepted(&send(&mut s, request));
        assert_eq!(s.draft.adjustments.as_ref().unwrap().employee_expenses[0].capacity, capacity);
    }
    let model = crate::federal_model::HUB_SOURCE;
    let form = &model[model.find("pub struct EmployeeExpenseForm").unwrap()..];
    let capacity =
        &form[form.find("pub capacity").unwrap() - 120..form.find("pub capacity").unwrap()];
    assert!(!capacity.contains("libertas_default"), "{capacity}");
    let module = include_str!("../../business/employee_expense_model.rs");
    let variants = &module[module.find("pub enum EmployeeCapacity").unwrap()..];
    let variants = &variants[..variants.find("\n}").unwrap()];
    assert_eq!(
        ["Reservist,", "PerformingArtist,", "FeeBasisOfficial,"]
            .iter()
            .filter(|v| variants.contains(*v))
            .count(),
        3
    );
    assert!(!variants.contains("Impairment"));
}

#[test]
fn form_2106_server_twins_reject_what_the_client_rules_reject() {
    let mut d = complete(FederalStatus::Joint);
    d.income.push(w2_entry(2, Owner::Spouse, "Spouse Co", 420_000, 0));
    d.next_id = 3;
    let mut s = Interview::new(TEST_USER, d);
    let page = open_page(&mut s);
    let actual = actual_page_vehicle(FirstYearDeduction::Bonus);
    // EMPLOYEE_EXPENSES_EMPLOYER_NEEDED
    let request =
        save_page(&s, &page, vec![page_form(EmployeeCapacity::FeeBasisOfficial, &[], vec![])]);
    crate::tests::cross::interview::rejected_with(
        send(&mut s, request),
        "EMPLOYEE_EXPENSES_EMPLOYER_NEEDED",
    );
    // EMPLOYEE_EXPENSES_EMPLOYER_TWICE
    let request =
        save_page(&s, &page, vec![page_form(EmployeeCapacity::FeeBasisOfficial, &[1, 1], vec![])]);
    crate::tests::cross::interview::rejected_with(
        current_source_refusal(
            "secondary-admission-399",
            &mut s,
            request,
            "EMPLOYEE_EXPENSES_EMPLOYER_TWICE",
            Some("[value/forms/0/employers/1]"),
            SourceScope::Secondary(SecondaryCorrection::Employer),
        ),
        "EMPLOYEE_EXPENSES_EMPLOYER_TWICE",
    );
    // A stale element of the employers host names no selectable W-2, nor does
    // another owner's: the save is refused and nothing is dropped or remapped.
    for (case, stale) in
        [("secondary-admission-400", &[99][..]), ("secondary-admission-401", &[2][..])]
    {
        let request = save_page(
            &s,
            &page,
            vec![page_form(EmployeeCapacity::FeeBasisOfficial, stale, vec![])],
        );
        crate::tests::cross::interview::rejected_with(
            current_source_refusal(
                case,
                &mut s,
                request,
                "MESSAGE_EMPLOYER_MUST_BE_YOUR_W_2",
                Some("[value/forms/0/employers/0]"),
                SourceScope::Secondary(SecondaryCorrection::Employer),
            ),
            "MESSAGE_EMPLOYER_MUST_BE_YOUR_W_2",
        );
    }
    // RESERVIST_VEHICLE_STANDARD_MILEAGE
    let request = save_page(
        &s,
        &page,
        vec![page_form(EmployeeCapacity::Reservist, &[1], vec![actual.clone()])],
    );
    crate::tests::cross::interview::rejected_with(
        send(&mut s, request),
        "RESERVIST_VEHICLE_STANDARD_MILEAGE",
    );
    // EMPLOYEE_VEHICLE_NO_INTEREST_OR_TAX on a mileage vehicle and on an actual one.
    let request = save_page(
        &s,
        &page,
        vec![page_form(EmployeeCapacity::FeeBasisOfficial, &[1], vec![mileage_vehicle(30_000)])],
    );
    crate::tests::cross::interview::rejected_with(
        send(&mut s, request),
        "EMPLOYEE_VEHICLE_NO_INTEREST_OR_TAX",
    );
    let VehicleMethod::Actual {
        cost,
        placed,
        total_miles,
        business_miles,
        operating_costs,
        kind,
        history,
        disposal,
        ..
    } = actual.method.clone()
    else {
        unreachable!()
    };
    let with_interest = Vehicle {
        method: VehicleMethod::Actual {
            macrs_source: None,
            id: 0,
            history_details: None,
            amt_section179_income_carry: None,
            allowed_amt_carry: Vec::new(),
            cost,
            credit_reduction: 0,
            acquired: None,
            placed,
            total_miles,
            business_miles,
            operating_costs,
            loan_interest: 100,
            kind,
            history,
            disposal,
            earlier_business_percent: None,
        },
    };
    let request = save_page(
        &s,
        &page,
        vec![page_form(EmployeeCapacity::FeeBasisOfficial, &[1], vec![with_interest])],
    );
    crate::tests::cross::interview::rejected_with(
        send(&mut s, request),
        "EMPLOYEE_VEHICLE_NO_INTEREST_OR_TAX",
    );
    // The row owner's own W-2 only (the spouse's W-2 is id 2), and one form per W-2.
    let request =
        save_page(&s, &page, vec![page_form(EmployeeCapacity::FeeBasisOfficial, &[2], vec![])]);
    assert!(
        refusal(current_source_refusal(
            "secondary-admission-405",
            &mut s,
            request,
            "MESSAGE_EMPLOYER_MUST_BE_YOUR_W_2",
            Some("[value/forms/0/employers/0]"),
            SourceScope::Secondary(SecondaryCorrection::Employer)
        ))
        .contains("W-2 of the person")
    );
    let request = save_page(
        &s,
        &page,
        vec![
            page_form(EmployeeCapacity::FeeBasisOfficial, &[1], vec![]),
            page_form(EmployeeCapacity::FeeBasisOfficial, &[1], vec![]),
        ],
    );
    assert!(
        refusal(current_source_refusal(
            "secondary-admission-406",
            &mut s,
            request,
            "MESSAGE_EMPLOYER_CHOSEN_ON_ONE_FORM_2106_ONLY",
            Some("[value/forms/1/employers/0]"),
            SourceScope::Secondary(SecondaryCorrection::SharedEmployer)
        ))
        .contains("one Form 2106 only")
    );
    // Nothing was stored by any refusal.
    assert!(s.draft.adjustments.as_ref().unwrap().employee_expenses.is_empty());
}

#[test]
fn w2_box_12_code_l_imports_stores_and_reopens() {
    // Box 12 code L is an import target (the W-2 instruction returns each printed code
    // the result schema supports) and maps onto the stored code list.
    let model = crate::federal_model::HUB_SOURCE;
    // The code list is the unified W2Code enum; its entry's code and amount are import targets.
    let entry = &model[model.find("pub struct Box12Entry").unwrap()..];
    let entry = &entry[..entry.find("\n}").unwrap()];
    assert!(entry.contains("libertas_llm_doc_field(blank = 0)"));
    assert!(entry.contains("libertas_default(0)"));
    assert!(
        model
            .contains("    /// L — substantiated employee business expense reimbursements\n    #[libertas_form_code(\"L\")]\n    L,")
    );
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut w = crate::tests::cross::interview::w2(6_000_000);
    w.more.box12 = vec![
        Box12Entry { code: W2Code::L, amount: 250_000 },
        Box12Entry { code: W2Code::D, amount: 100_000 },
    ];
    let request = crate::tests::cross::interview::save_documents(
        &mut session,
        vec![crate::tests::cross::interview::document(
            0,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w.clone())) },
        )],
    );
    accepted(&send(&mut session, request));
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { unreachable!() };
    let data = &FederalWage::from(data);
    assert_eq!(
        data.other_box12,
        vec![
            W2CodeAmount { code: W2Code::L, amount: 250_000 },
            W2CodeAmount { code: W2Code::D, amount: 100_000 },
        ]
    );
    // Code L alone refuses nothing and changes nothing on the return.
    let plain = {
        let mut d = session.draft.clone();
        let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
        let mut data = edit_wage(data);
        data.other_box12.retain(|c| c.code != W2Code::L);
        drop(data);
        d
    };
    let with_l = supported(&session.draft);
    assert_eq!(with_l.tax, supported(&plain).tax);
    // Reopening rebuilds the identical page.
    let mut session = restart(session.draft);
    let page = crate::tests::cross::interview::open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = page else {
        panic!("expected the wages source editor")
    };
    assert_eq!(documents.common.wages.documents.len(), 1);
    let source = documents.common.wages.documents[0].id;
    let IncomeSource::Wage { value: expected } = expected_named_source(
        &session.draft,
        Owner::Taxpayer,
        IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w.clone())) },
    ) else {
        unreachable!()
    };
    assert_eq!(documents.common.wages.documents[0].document, expected);
    // Two code L entries are one W2_BOX12_CODE_TWICE.
    let mut twice = w;
    twice.more.box12 = vec![
        Box12Entry { code: W2Code::L, amount: 1_000 },
        Box12Entry { code: W2Code::L, amount: 2_000 },
    ];
    let request = crate::tests::cross::interview::save_documents(
        &mut session,
        vec![crate::tests::cross::interview::document(
            source,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(twice)) },
        )],
    );
    crate::tests::cross::interview::rejected_with(
        send(&mut session, request),
        "W2_BOX12_CODE_TWICE",
    );
}

#[test]
fn search_destination_names_the_form_its_filers_and_the_other_page() {
    let source = include_str!("../../core/shared/navigation.rs");
    let at = source.find("    EmployeeExpenses,").unwrap();
    let doc = &source[source[..at].rfind("    /// Reservist, performing artist").unwrap()..at];
    for term in [
        "Form 2106",
        "Armed Forces reserve",
        "National Guard",
        "100 miles",
        "performing artist",
        "actor, musician, dancer",
        "fee-basis state or local government official",
        "standard mileage",
        "code L",
        "Investment interest and other deductions",
    ] {
        assert!(doc.contains(term), "the Search text lacks {term}");
    }
    assert!(doc.contains("#[libertas_protocol_variant(\"SaveEmployeeExpenses\")]"));
    // The retired refusal is gone from the more-adjustments description.
    assert!(!source.contains("business expenses of reservists, performing artists"));
    // The destination is offered in the adjustments group.
    let availability = NavigationAvailability::from_sections(&[TaxSection::EmployeeExpenses]);
    assert_eq!(availability.adjustments, vec![TaxSection::EmployeeExpenses as i32]);
}

#[test]
fn employee_nonvehicle_recovery_saves_corrects_reopens_and_restarts() {
    use crate::tests::support::builders::nonmacrs::property;
    let mut session = restart(complete(FederalStatus::Single));
    let page = open_page(&mut session);
    let mut form = page_form(EmployeeCapacity::FeeBasisOfficial, &[1], vec![]);
    form.travel = 0;
    form.meals = 0;
    let patent = property(
        510_000,
        20260401,
        NonMacrsRecovery::FiniteLife {
            facts: FiniteLifeRecovery::PatentCopyright {
                legal_remaining_months: 204,
                useful_months: 204,
                salvage: 0,
            },
        },
    );
    let software = property(
        3_600_000,
        20260701,
        NonMacrsRecovery::Software {
            facts: SoftwareRecovery {
                classification: SoftwareKind::Standalone,
                exempt_lease: None,
                bonus_eligible: false,
            },
        },
    );
    form.equipment_assets =
        crate::interview::vehicle_and_asset_items(&[], &[patent, software]).unwrap().1;
    let request = save_page(&session, &page, vec![form]);
    accepted(&send(&mut session, request));
    let sources =
        session.draft.adjustments.as_ref().unwrap().employee_expenses[0].equipment_assets.clone();
    assert!(sources[0].id > 0 && sources[1].id > 0 && sources[0].id != sources[1].id);
    let result = supported(&session.draft);
    // Pub946's $225 patent + six software months at $1,000 = $6,225.
    assert_eq!(value(&result, "employee_expenses_1_line_4"), 6_225);
    assert_eq!(value(&result, "employee_expenses_1_line_1"), 0);
    assert_eq!(value(&result, "schedule_1_employee_business_expenses"), 6_225);
    assert_eq!(value(&result, "employee_expenses_1_equipment_depreciation_class_16"), 6_225);
    let reopened = open_page(&mut session);
    let P::SaveEmployeeExpenses { value: page, .. } = &reopened else { unreachable!() };
    let mut corrected = page.forms.clone();
    corrected[0].equipment_assets[1].cost = 1_800_000;
    let request = save_page(&session, &reopened, corrected);
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "employee_expenses_1_line_4"), 3_225);
    let accepted_draft = session.draft.clone();
    let mut again = restart(accepted_draft.clone());
    let page = open_page(&mut again);
    let P::SaveEmployeeExpenses { value: page, .. } = page else { unreachable!() };
    assert_eq!(page.forms[0].equipment_assets[0].id, sources[0].id);
    assert_eq!(page.forms[0].equipment_assets[1].id, sources[1].id);
    assert_eq!(page.forms[0].equipment_assets[1].cost, 1_800_000);
    assert_eq!(again.draft, accepted_draft);
    assert_eq!(supported(&again.draft), result);
}

#[test]
fn employee_software_disposal_reuses_recapture_without_self_employment_or_qbi() {
    use crate::tests::support::builders::nonmacrs::{continuing, property};
    let mut software = property(
        3_600_000,
        20250701,
        NonMacrsRecovery::Software {
            facts: SoftwareRecovery {
                classification: SoftwareKind::Standalone,
                exempt_lease: None,
                bonus_eligible: false,
            },
        },
    );
    continuing(&mut software, 600_000);
    software.disposal = AssetDisposal::Disposed {
        date: 20261001,
        how: DisposalKind::Sold {
            proceeds: 3_000_000,
            selling_expenses: 0,
            payment: SalePayment::InFull,
            buyer: Buyer::Unrelated,
            livestock: crate::SoldLivestock::No,
        },
    };
    let mut form = claim(
        Owner::Taxpayer,
        "Fictional county official",
        EmployeeCapacity::FeeBasisOfficial,
        &[1],
    );
    form.equipment_assets = vec![software];
    let draft = with_claims(complete(FederalStatus::Single), vec![form]);
    let result = supported(&draft);
    // Nine service months recover $9,000; $30,000 price less $21,000 basis
    // is $9,000 ordinary recapture. Neither amount acquires SE/QBI character.
    assert_eq!(value(&result, "employee_expenses_1_line_4"), 9_000);
    assert_eq!(value(&result, "employee_expenses_1_equipment_1_disposal_1245_recapture"), 9_000);
    assert_eq!(value(&result, "self_employment_tax"), 0);
    assert_eq!(value(&result, "qbi_deduction"), 0);
    let mut report = FederalResult::empty(draft.revision);
    assert_eq!(crate::employee_expenses::calculate(&draft, &mut report).amt_adjustment(&draft), 0);
    assert_eq!(supported(&restart(draft).draft), result);
}

#[test]
fn employee_equipment_election_saves_corrects_and_required_lookback_stays_unknown() {
    use crate::tests::support::builders::nonmacrs::property;
    let mut software = property(
        3_600_000,
        20260701,
        NonMacrsRecovery::Software {
            facts: SoftwareRecovery {
                classification: SoftwareKind::PublicNonexclusive,
                exempt_lease: None,
                bonus_eligible: false,
            },
        },
    );
    software.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 100_000 },
        bonus: BonusElection::ElectOut,
        tables: false,
    };
    let forecast = property(
        20_000_000,
        20260101,
        NonMacrsRecovery::IncomeForecast {
            facts: IncomeForecastRecovery {
                kind: ForecastPropertyKind::BookCopyright,
                current_income: 2_000_000,
                forecast_total_income: 10_000_000,
                remaining_basis: 20_000_000,
                participations: ForecastParticipations::None,
                lookback: ForecastLookback::WorksheetNeeded,
            },
        },
    );
    for source in [software, forecast] {
        let mut session = restart(complete(FederalStatus::Single));
        let page = open_page(&mut session);
        let mut form = page_form(EmployeeCapacity::FeeBasisOfficial, &[1], vec![]);
        form.equipment_assets =
            crate::interview::vehicle_and_asset_items(&[], core::slice::from_ref(&source))
                .unwrap()
                .1;
        let request = save_page(&session, &page, vec![form]);
        accepted(&send(&mut session, request));
        let stored =
            &session.draft.adjustments.as_ref().unwrap().employee_expenses[0].equipment_assets[0];
        assert_eq!(stored.history, source.history);
        assert_eq!(stored.class, source.class);
        if source.available179() > 0 {
            // Pub946 software: ($36,000-$1,000) x6/36 = $5,833;
            // line4 includes the single $1,000 allowance.
            let expected = supported(&session.draft);
            assert_eq!(value(&expected, "employee_expenses_1_line_4"), 6_833);
            assert_eq!(value(&expected, "employee_expenses_1_equipment_1_section179"), 1_000);
            assert_eq!(value(&expected, "section179_carryforward"), 0);
            assert_eq!(value(&expected, "self_employment_tax"), 0);
            assert_eq!(value(&expected, "qbi_deduction"), 0);
            let mut again = restart(session.draft.clone());
            assert_eq!(supported(&again.draft), expected);
            let source_id = again.draft.adjustments.as_ref().unwrap().employee_expenses[0]
                .equipment_assets[0]
                .id;
            let reopened = open_page(&mut again);
            let P::SaveEmployeeExpenses { value: editor, .. } = &reopened else { unreachable!() };
            let mut forms = editor.forms.clone();
            let AssetRecoveryPage::NonMacrs {
                facts: NonMacrsPropertyRecovery::Software { history, .. },
            } = &mut forms[0].equipment_assets[0].recovery
            else {
                unreachable!()
            };
            *history = NonMacrsHistory::New {
                section179: Section179::Eligible { elected: 200_000 },
                bonus: BonusElection::ElectOut,
            };
            let request = save_page(&again, &reopened, forms);
            accepted(&send(&mut again, request));
            let revised = supported(&again.draft);
            assert_eq!(value(&revised, "employee_expenses_1_line_4"), 7_667);
            assert_eq!(
                again.draft.adjustments.as_ref().unwrap().employee_expenses[0].equipment_assets[0]
                    .id,
                source_id
            );
            assert_eq!(supported(&restart(again.draft).draft), revised);
        } else {
            unsupported(&session.draft, "Form 2106 equipment");
            // The native source remains unavailable at its own equipment page.
            // A projection adapter must not restart that failure and discard
            // independently known zero capital/expense source categories.
            let result = estimate(&session.draft);
            assert_eq!(result.capital_footing, Some([0, 0]));
            assert_eq!(result.expense_roles.as_ref().unwrap().connected_dollars, Some(0));
            assert_eq!(
                result
                    .issues
                    .iter()
                    .filter(|issue| issue.display.key == "ASSET_FORECAST_LOOKBACK_UNAVAILABLE")
                    .count(),
                1
            );
            assert_eq!(crate::employee_expenses::before179_compensation(&session.draft), Ok(0));
            let mut reimbursed = session.draft.clone();
            reimbursed.adjustments.as_mut().unwrap().employee_expenses[0].reimbursements = 1;
            assert_eq!(
                crate::employee_expenses::before179_compensation(&reimbursed),
                Err(Section179PreparedIssue::SourceBoundaryUnavailable(
                    crate::asset_sale::employee_source_id(0),
                    "ASSET_FORECAST_LOOKBACK_UNAVAILABLE"
                ))
            );
            let again = restart(session.draft.clone());
            assert_eq!(again.draft, session.draft);
            unsupported(&again.draft, "Form 2106 equipment");
        }
    }
}

#[test]
fn reservist_generic_equipment_does_not_establish_an_overnight_travel_source() {
    use crate::tests::support::builders::nonmacrs::property;
    let source = property(
        3_600_000,
        20260701,
        NonMacrsRecovery::Software {
            facts: SoftwareRecovery {
                classification: SoftwareKind::Standalone,
                exempt_lease: None,
                bonus_eligible: false,
            },
        },
    );
    let mut form = claim(Owner::Taxpayer, "Fictional reservist", EmployeeCapacity::Reservist, &[1]);
    form.equipment_assets = vec![source.clone()];
    let draft = with_claims(complete(FederalStatus::Single), vec![form]);
    assert_eq!(
        draft.adjustments.as_ref().unwrap().employee_expenses[0].equipment_assets,
        vec![source]
    );
    unsupported(&draft, "Form 2106 equipment");
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn employee_section179_shares_direct_pool_and_current_precedes_carry() {
    let mut draft = complete(FederalStatus::Single);
    draft.income = vec![w2_entry(1, Owner::Taxpayer, "Synthetic office", 100_000, 0)];
    let mut business = cash_business(0);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    let mut direct = elected_asset(1_000, 1_000, BonusElection::ElectOut);
    direct.id = 100;
    data.assets = vec![direct];
    add(&mut draft, income(business));
    let source = draft.income[1].id;
    let mut employee =
        claim(Owner::Taxpayer, "Synthetic official", EmployeeCapacity::FeeBasisOfficial, &[1]);
    let mut equipment = elected_asset(1_000, 1_000, BonusElection::ElectOut);
    equipment.id = 101;
    employee.equipment_assets = vec![equipment];
    draft = with_claims(draft, vec![employee]);
    draft.next_id = 102;
    let result = supported(&draft);
    assert_eq!(value(&result, "section179_business_income_limit"), 1_000);
    assert_eq!(value(&result, "section179_deduction"), 1_000);
    assert_eq!(value(&result, "employee_expenses_1_equipment_1_section179"), 500);
    assert_eq!(value(&result, &alloc::format!("business_{source}_asset_0_section179")), 500);
    assert_eq!(value(&result, "employee_expenses_1_line_4"), 500);
    assert_eq!(value(&result, "section179_carryforward"), 1_000);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let mut prior = carried_asset(2025, 1_000);
    prior.id = 100;
    prior.amt_section179_income_carry = Some(100_000);
    data.assets = vec![prior];
    let result = supported(&draft);
    assert_eq!(value(&result, "section179_prior_carryover_total"), 1_000);
    assert_eq!(value(&result, "employee_expenses_1_equipment_1_section179"), 1_000);
    assert_eq!(value(&result, &alloc::format!("business_{source}_asset_0_section179")), 0);
    assert_eq!(value(&result, "section179_carryforward"), 1_000);
}

#[test]
fn employee_section179_uses_before179_reimbursement_compensation_once() {
    let mut draft = complete(FederalStatus::Single);
    draft.income = vec![w2_entry(1, Owner::Taxpayer, "Synthetic office", 10_000, 0)];
    let mut employee =
        claim(Owner::Taxpayer, "Synthetic official", EmployeeCapacity::FeeBasisOfficial, &[1]);
    let mut equipment = elected_asset(1_000, 1_000, BonusElection::ElectOut);
    equipment.id = 100;
    employee.equipment_assets = vec![equipment];
    employee.other = 20_000;
    employee.meals = 10_000;
    employee.reimbursements = 100_000;
    draft = with_claims(draft, vec![employee]);
    draft.next_id = 101;
    // Without179: A=$200,B=$100, ratio=.333, reimbursementB=$333;
    // excessA=$467 supplies compensation, so line11=$100+$467=$567.
    // Actual179=$567: A=$767, ratio=.115, reimbursementB=$115;
    // excessA=$118 is the final wage addition. Neither worksheet is iterated.
    let result = supported(&draft);
    assert_eq!(value(&result, "section179_business_income_limit"), 567);
    assert_eq!(value(&result, "employee_expenses_1_equipment_1_section179"), 567);
    assert_eq!(value(&result, "employee_expenses_1_equipment_1_section179_carryforward"), 433);
    assert_eq!(value(&result, "employee_expenses_1_line_4"), 767);
    assert_eq!(value(&result, "employee_expenses_1_excess_reimbursement"), 118);
    assert_eq!(value(&result, "wages"), 218);
    assert_eq!(value(&result, "self_employment_tax"), 0);
    // Reg.1.179-2(c)(1), (6)(iv): independently sourced foreign wages
    // join the same before179 compensation once, not the final reimbursement.
    add(&mut draft, income(foreign_pay(200)));
    assert_eq!(crate::employee_expenses::before179_compensation(&draft), Ok(46_700));
    for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
        let selected = crate::section179_return::prepare(&draft, system).unwrap();
        assert_eq!(selected.allocation.income.business_income_limit_cents, 76_700);
        assert_eq!(selected.allocation.employee_allowed(0), Ok(76_700));
    }
    assert_eq!(value(&supported(&draft), "section179_business_income_limit"), 767);
}

#[test]
fn employee_section179_retains_independent_amt_carry_and_depreciation_adjustment() {
    let mut draft = complete(FederalStatus::Single);
    draft.income = vec![w2_entry(1, Owner::Taxpayer, "Synthetic office", 100_000, 0)];
    let mut employee =
        claim(Owner::Taxpayer, "Synthetic official", EmployeeCapacity::FeeBasisOfficial, &[1]);
    let mut equipment = carried_asset(2025, 2_000);
    equipment.id = 100;
    equipment.amt_section179_income_carry = Some(50_000);
    employee.equipment_assets = vec![equipment];
    employee.other = 20_000;
    employee.meals = 10_000;
    employee.reimbursements = 100_000;
    draft = with_claims(draft, vec![employee]);
    draft.next_id = 101;
    // The same before179 compensation is $467 in each system. Regular179
    // uses $1,467 of $2,000; AMT uses its separately reported $500 carry.
    let pair = [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative]
        .map(|system| crate::section179_return::prepare(&draft, system).unwrap());
    assert_eq!(pair[0].allocation.employee_allowed(0), Ok(146_700));
    assert_eq!(pair[1].allocation.employee_allowed(0), Ok(50_000));
    let mut report = FederalResult::empty(draft.revision);
    let calculated =
        crate::employee_expenses::calculate_with_section179(&draft, &mut report, Some(&pair), None);
    // i6251 line2l compares depreciation deductions; no duplicate adjustment
    // from separately recomputing reimbursement allocations belongs here.
    assert_eq!(calculated.amt_adjustment(&draft), 96_700);
    let result = supported(&draft);
    assert_eq!(value(&result, "employee_expenses_1_equipment_1_section179_carryforward"), 533);
    assert_eq!(value(&result, "amt_business_depreciation_adjustment"), 967);
    let original = draft.clone();
    draft.adjustments.as_mut().unwrap().employee_expenses[0].equipment_assets[0]
        .amt_section179_income_carry = None;
    unsupported(&draft, "Section 179 source");
    // A zero REG carry does not certify that the independent AMT carry is zero.
    let asset = &mut draft.adjustments.as_mut().unwrap().employee_expenses[0].equipment_assets[0];
    let EquipmentHistory::Continuing { carryforward, prior_depreciation, .. } = &mut asset.history
    else {
        unreachable!()
    };
    *carryforward = 0;
    *prior_depreciation = 200_000;
    assert!(
        crate::section179_return::needed(crate::related_forms::RelatedFormContext::original(
            &draft,
            &crate::related_forms::effective_claimants(&draft)
        ))
        .unwrap()
    );
    unsupported(&draft, "Section 179 source");
    assert_eq!(supported(&restart(original).draft), result);
}

#[test]
fn employee_section179_runs_performing_artist_qualification_after_allocation() {
    let mut draft = artist_return(FederalStatus::Single, 0, 300_000, 0);
    let mut equipment = elected_asset(2_000, 2_000, BonusElection::ElectOut);
    equipment.id = 100;
    let employee = &mut draft.adjustments.as_mut().unwrap().employee_expenses[0];
    employee.travel = 0;
    employee.meals = 0;
    employee.equipment_assets = vec![equipment];
    draft.next_id = 101;
    let result = supported(&draft);
    assert_eq!(value(&result, "employee_expenses_1_line_4"), 2_000);
    assert_eq!(value(&result, "schedule_1_employee_business_expenses"), 1_700);
    let FederalIncome::Wage { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_wage(data);
    data.wages = 800_000;
    drop(data);
    let result = supported(&draft);
    assert!(noted(&result, "WARN_PERFORMING_ARTIST_AGI"));
    assert_eq!(value(&result, "employee_expenses_1_equipment_1_section179"), 2_000);
    assert!(!result.lines.iter().any(|line| line.id == "schedule_1_employee_business_expenses"));
}

#[test]
fn employee_section179_without_spouse_records_reuses_ordinary_recovery_policy() {
    let mut draft = complete(FederalStatus::Separate);
    let mut employee =
        claim(Owner::Taxpayer, "Synthetic official", EmployeeCapacity::FeeBasisOfficial, &[1]);
    let mut equipment = elected_asset(1_000, 1_000, BonusElection::ElectOut);
    equipment.id = 100;
    employee.equipment_assets = vec![equipment];
    draft = with_claims(draft, vec![employee]);
    draft.next_id = 101;
    let original = draft.clone();
    let result = supported(&draft);
    assert!(noted(&result, "WARN_SECTION_179_SPOUSE_RECORDS_LEFT_OUT"));
    assert_eq!(value(&result, "employee_expenses_1_line_4"), 200);
    assert_eq!(draft, original);
    assert_eq!(
        draft.adjustments.as_ref().unwrap().employee_expenses[0].equipment_assets[0].elected179(),
        100_000
    );
}
