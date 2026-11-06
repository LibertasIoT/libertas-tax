// Form 4835, farm rental income and expenses (unit B2). Every amount is
// synthetic; the arithmetic of each fixture is in its comments.
use super::*;
use crate::tests::foreign::foreign::passive_fund_return;
use alloc::{format, string::String};

fn rental_expense(category: BusinessExpenseCategory, dollars: i64) -> FederalBusinessExpense {
    ordinary_business_expense(category, dollars * 100)
}
fn rental_income(data: FederalFarmRental) -> FederalIncome {
    FederalIncome::FarmRental { data }
}
// A return with one wage and the given entries.
fn rental_return(wages: i64, entries: Vec<FederalIncome>) -> FederalDraft {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(0, Owner::Taxpayer, wages * 100, 0));
    for entry in entries {
        add(&mut d, income(entry));
    }
    d
}
fn line_id(id: i64, suffix: &str) -> String {
    alloc::format!("farm_rental_{id}_{suffix}")
}
fn form_row(r: &FederalResult, form: TaxForm, line: &str) -> Option<i64> {
    r.lines
        .iter()
        .find(|l| l.form_line.as_ref().is_some_and(|f| f.form == form && f.line == line))
        .map(|l| l.amount)
}

#[test]
fn fixture_a_profit_has_no_self_employment_tax_and_the_qbi_variant_lowers_tax() {
    // Single; W-2 $40,000. Line 1 $24,000, line 3 $1,500; fertilizers $3,000,
    // chemicals $1,200, taxes $2,400, insurance $600, repairs $800.
    //   line 7  = 24,000 + 1,500                       = $25,500
    //   line 31 = 3,000 + 1,200 + 2,400 + 600 + 800    = $8,000
    //   line 32 = 25,500 - 8,000                       = $17,500
    //   AGI     = 40,000 + 17,500                      = $57,500
    //   line 15 = 57,500 - 16,100                      = $41,400
    //   line 16 = row 41,400-41,450, midpoint 41,425: 1,240 + 12% x 29,025 = $4,723
    // A Schedule F farm would owe 17,500 x 0.9235 x 15.3% = $2,473 of SE tax.
    let mut data = farm_rental_data(24_000, 1_500);
    data.expenses = vec![
        rental_expense(BusinessExpenseCategory::Fertilizers, 3_000),
        rental_expense(BusinessExpenseCategory::Chemicals, 1_200),
        rental_expense(BusinessExpenseCategory::TaxesAndLicenses, 2_400),
        rental_expense(BusinessExpenseCategory::Insurance, 600),
        rental_expense(BusinessExpenseCategory::Repairs, 800),
    ];
    let d = rental_return(40_000, vec![rental_income(data.clone())]);
    let id = d.income[1].id;
    let r = supported(&d);
    assert_eq!(value(&r, &line_id(id, "gross_income")), 25_500);
    assert_eq!(value(&r, &line_id(id, "expenses")), 8_000);
    assert_eq!(value(&r, &line_id(id, "net_income")), 17_500);
    assert_eq!(value(&r, "schedule_e_farm_rental_total"), 17_500);
    assert_eq!(value(&r, "schedule_e_farming_fishing_gross"), 25_500);
    assert_eq!(value(&r, "schedule_1_rental_income"), 17_500);
    assert_eq!(value(&r, "agi"), 57_500);
    assert_eq!(return_line(&r, "16"), 472_300);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    // No Schedule E Part I property row: line 40, not lines 3 to 26.
    assert!(r.lines.iter().all(|l| !l.id.starts_with(&alloc::format!("rental_{id}_"))));
    assert_eq!(form_row(&r, TaxForm::ScheduleE, "26"), None);
    assert_eq!(form_row(&r, TaxForm::Form4835, "32"), Some(1_750_000), "{:?}", r.lines);
    assert_eq!(form_row(&r, TaxForm::Form4835, "7"), Some(2_550_000));
    assert_eq!(form_row(&r, TaxForm::Form4835, "31"), Some(800_000));
    assert_eq!(form_row(&r, TaxForm::Form4835, "9"), Some(120_000));
    assert_eq!(form_row(&r, TaxForm::Form4835, "27"), Some(240_000));
    assert_eq!(form_row(&r, TaxForm::ScheduleE, "40"), Some(1_750_000));
    // The same return reopens: the stored rental is reproduced by its page.
    assert_eq!(restart(d.clone()).draft, d);
}

fn rental_a2_expenses(data: &mut FederalFarmRental) {
    data.expenses = vec![
        rental_expense(BusinessExpenseCategory::Fertilizers, 3_000),
        rental_expense(BusinessExpenseCategory::Chemicals, 1_200),
        rental_expense(BusinessExpenseCategory::TaxesAndLicenses, 2_400),
        rental_expense(BusinessExpenseCategory::Insurance, 600),
        rental_expense(BusinessExpenseCategory::Repairs, 800),
    ];
}

#[test]
fn fixture_a2_qbi_yes_deducts_twenty_percent_of_the_rental_profit() {
    // Fixture A with the rental a qualified trade or business (qbi Yes, no
    // material participation): QBI = 17,500, 20% = 3,500, below 20% of
    // taxable income before QBI (41,400 x 20% = 8,280).
    //   AGI 57,500; line 15 = 57,500 - 16,100 = 41,400; QBI deduction 3,500
    //   taxable = 37,900; row 37,900-37,950, midpoint 37,925:
    //   1,240 + 12% x (37,925 - 12,400) = 1,240 + 3,063 = $4,303
    let mut data = farm_rental_data(24_000, 1_500);
    rental_a2_expenses(&mut data);
    data.qbi_business = RentalBusiness::Yes { material_participation: false };
    let d = rental_return(40_000, vec![rental_income(data)]);
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_deduction"), 3_500);
    assert_eq!(return_line(&r, "13b"), 350_000);
    assert_eq!(return_line(&r, "16"), 430_300);
}

#[test]
fn fixture_b_loss_is_limited_by_the_special_allowance_and_the_rest_is_suspended() {
    // Single; W-2 $130,000. Line 1 $8,000; fertilizers $6,000, taxes $3,000,
    // mortgage interest $9,000, repairs $4,000; active participation.
    //   line 31 = 22,000; line 32 = 8,000 - 22,000      = ($14,000)
    //   MAGI 130,000; allowance = 25,000 - 50% x 30,000  = $10,000
    //   line 34c = ($10,000); suspended to 2027          = $4,000
    //   AGI = 130,000 - 10,000                           = $120,000
    //   line 15 = 120,000 - 16,100 = 103,900; Tax Computation Worksheet:
    //   103,900 x 22% - 5,288                            = $17,570
    let mut data = farm_rental_data(8_000, 0);
    data.expenses = vec![
        rental_expense(BusinessExpenseCategory::Fertilizers, 6_000),
        rental_expense(BusinessExpenseCategory::TaxesAndLicenses, 3_000),
        rental_expense(BusinessExpenseCategory::MortgageInterest, 9_000),
        rental_expense(BusinessExpenseCategory::Repairs, 4_000),
    ];
    let d = rental_return(130_000, vec![rental_income(data.clone())]);
    let id = d.income[1].id;
    let r = supported(&d);
    // Form 4835 line 32 is blank for a loss ("don't enter your loss on line
    // 32", 2026 i4835); the loss goes through Form 8582 to line 34c.
    assert_eq!(value(&r, &line_id(id, "net_income")), 0);
    assert_eq!(value(&r, &line_id(id, "line34c_allowed_loss")), -10_000);
    assert_eq!(value(&r, &line_id(id, "suspended_loss")), 4_000);
    // The regular-tax line carries the regular explanation, not the AMT one.
    assert_eq!(
        r.lines.iter().find(|l| l.id == line_id(id, "suspended_loss")).unwrap().explanation.key,
        "EXPLAIN_FARM_RENTAL_N_SUSPENDED_LOSS"
    );
    assert_eq!(form_row(&r, TaxForm::Form4835, "34c"), Some(-1_000_000));
    assert_eq!(value(&r, "schedule_e_farm_rental_total"), -10_000);
    assert_eq!(value(&r, "agi"), 120_000);
    assert_eq!(return_line(&r, "16"), 1_757_000);

    // B2: no active participation. Form 8582 line 2 has no allowance:
    //   allowed 0, suspended 14,000, AGI 130,000, line 15 113,900,
    //   tax = 5,800 + 22% x 55,300 + 24% x 8,200 = 5,800 + 12,166 + 1,968
    //   = $19,934
    let mut passive = data.clone();
    passive.active_participation = false;
    let d2 = rental_return(130_000, vec![rental_income(passive)]);
    let id2 = d2.income[1].id;
    let r2 = supported(&d2);
    assert_eq!(value(&r2, &line_id(id2, "suspended_loss")), 14_000);
    assert_eq!(value(&r2, "schedule_e_farm_rental_total"), 0);
    assert_eq!(value(&r2, "agi"), 130_000);
    assert_eq!(return_line(&r2, "16"), 1_993_400);

    // B3: not fully at risk with a loss (line 34b): the rental is left out
    // with a notice; AGI and tax are those of a return without it. The note
    // does not change the result state.
    let mut at_risk = data.clone();
    // Retain answered No as typed Limited without
    // method facts, preserving these original known-profit/loss oracles.
    at_risk.at_risk = rental_risk_without_method();
    let d3 = rental_return(130_000, vec![rental_income(at_risk)]);
    let r3 = supported(&d3);
    assert!(noted(&r3, "WARN_FARM_RENTAL_LOSS_LEFT_OUT"));
    assert_eq!(value(&r3, "agi"), 130_000);
    assert_eq!(return_line(&r3, "16"), 1_993_400);
    // The same answered No with income computes normally.
    let mut profitable = farm_rental_data(24_000, 0);
    profitable.at_risk = rental_risk_without_method();
    let r4 = supported(&rental_return(40_000, vec![rental_income(profitable)]));
    assert!(!noted(&r4, "WARN_FARM_RENTAL_LOSS_LEFT_OUT"));
    assert_eq!(value(&r4, "agi"), 64_000);
}

fn ccc_lot(origin: CccOrigin, principal: i64, disposition: CccDisposition) -> CccLot {
    CccLot { id: 0, origin, principal: principal * 100, disposition }
}
fn fixture_c(method: CccMethod) -> FederalDraft {
    let mut data = farm_rental_data(6_000, 0);
    data.expenses = vec![
        rental_expense(BusinessExpenseCategory::Fertilizers, 2_500),
        rental_expense(BusinessExpenseCategory::TaxesAndLicenses, 1_500),
    ];
    let source = ccc_source(&method);
    data.farm.ccc = Some(FarmCcc {
        source,
        method,
        lots: vec![
            ccc_lot(CccOrigin::Current, 12_000, CccDisposition::Outstanding),
            ccc_lot(
                CccOrigin::PriorExcluded,
                8_000,
                CccDisposition::Forfeited { year: 2026, surplus: 0, deficiency: 0 },
            ),
        ],
        withholding: 0,
    });
    rental_return(30_000, vec![rental_income(data)])
}

#[test]
fn fixture_c_crop_share_with_a_ccc_loan_under_the_section_77_election() {
    // Single; W-2 $30,000. Line 1 $6,000; CCC election (method Income); lot (i)
    // a 2026 loan of $12,000 still pledged; lot (ii) a 2025 loan of $8,000
    // never in income, forfeited in 2026; fertilizers $2,500, taxes $1,500.
    //   line 4a = $12,000 (the current loan, included under section 77)
    //   line 4b = $8,000 forfeited; 4c = $8,000 (not included earlier)
    //   line 7  = 6,000 + 12,000 + 8,000                  = $26,000
    //   line 32 = 26,000 - 4,000                          = $22,000
    //   AGI 52,000; line 15 = 35,900; row 35,900-35,950, midpoint 35,925:
    //   1,240 + 12% x 23,525 = 1,240 + 2,823              = $4,063
    let d = fixture_c(CccMethod::Income);
    let id = d.income[1].id;
    let r = supported(&d);
    assert_eq!(form_row(&r, TaxForm::Form4835, "4a"), Some(1_200_000));
    assert_eq!(form_row(&r, TaxForm::Form4835, "4b"), Some(800_000));
    assert_eq!(form_row(&r, TaxForm::Form4835, "4c"), Some(800_000));
    assert_eq!(value(&r, &line_id(id, "gross_income")), 26_000);
    assert_eq!(value(&r, &line_id(id, "net_income")), 22_000);
    assert_eq!(value(&r, "agi"), 52_000);
    assert_eq!(return_line(&r, "16"), 406_300);
    assert!(noted(
        &r,
        "NOTE_CCC_SECTION_77_TREATMENT_RETAIN_THE_LOAN_DETAIL_ELECTION_STATEME_BA3452BD"
    ));

    // C2: loan treatment. The current advance is not income; the forfeited
    // 2025 loan is still the sale of $8,000.
    //   line 7 = 6,000 + 8,000 = $14,000; line 32 = $10,000; AGI 40,000
    //   line 15 = 23,900; row 23,900-23,950, midpoint 23,925:
    //   1,240 + 12% x 11,525 = 1,240 + 1,383                = $2,623
    let d2 = fixture_c(CccMethod::Loan);
    let id2 = d2.income[1].id;
    let r2 = supported(&d2);
    assert_eq!(form_row(&r2, TaxForm::Form4835, "4a"), Some(0));
    assert_eq!(value(&r2, &line_id(id2, "gross_income")), 14_000);
    assert_eq!(value(&r2, &line_id(id2, "net_income")), 10_000);
    assert_eq!(value(&r2, "agi"), 40_000);
    assert_eq!(return_line(&r2, "16"), 262_300);

    // C3: a Schedule F farm of the same owner on the other method raises
    // the one-method issue (section 77(b)).
    let mut d3 = fixture_c(CccMethod::Income);
    let mut farm = raised_farm(50_000, 10_000);
    let FederalIncome::Business { data } = &mut farm else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.ccc = Some(FarmCcc {
        method: CccMethod::Loan,
        lots: vec![ccc_lot(CccOrigin::Current, 1_000, CccDisposition::Outstanding)],
        withholding: 0,
        source: ccc_source(&CccMethod::Loan),
    });
    add(&mut d3, income(farm));
    unsupported(&d3, "CCC loan election");
}

#[test]
fn fixture_d_prior_year_loss_beyond_income_prints_pal_and_goes_to_form_8582() {
    // Single; W-2 $60,000. Line 1 $10,000, expenses $7,000, $4,000 suspended
    // from earlier years. Current net 3,000 less 4,000 = (1,000).
    //   line 32 = 0 ("PAL"); the 1,000 loss is on Form 8582 and line 34c;
    //   MAGI 60,000, allowance 25,000, allowed 1,000, suspended 0
    //   AGI = 60,000 - 1,000 = 59,000; line 15 = 42,900; row 42,900-42,950,
    //   midpoint 42,925: 1,240 + 12% x 30,525 = 1,240 + 3,663 = $4,903
    let mut data = farm_rental_data(10_000, 0);
    data.expenses = vec![rental_expense(BusinessExpenseCategory::Repairs, 7_000)];
    data.history = RentalHistory { regular: 400_000, amt: 400_000, ..RentalHistory::NONE };
    let d = rental_return(60_000, vec![rental_income(data)]);
    let id = d.income[1].id;
    let r = supported(&d);
    assert_eq!(form_row(&r, TaxForm::Form4835, "32"), Some(0));
    assert_eq!(form_row(&r, TaxForm::Form4835, "34c"), Some(-100_000));
    assert_eq!(value(&r, &line_id(id, "suspended_loss")), 0);
    assert_eq!(value(&r, "schedule_e_farm_rental_total"), -1_000);
    assert_eq!(value(&r, "agi"), 59_000);
    assert_eq!(return_line(&r, "16"), 490_300);
}

fn patr(patronage: i64) -> FederalIncome {
    let mut data = FederalCooperative { patronage: patronage * 100, ..Default::default() };
    data.allocations.push(crate::cooperative::automatic(&data));
    FederalIncome::Cooperative { data: cooperative_source(data) }
}

fn government_program(payment: i64) -> FederalIncome {
    let FederalIncome::GovernmentPayment { mut data } = government_unemployment(0, &[], 0).income
    else {
        unreachable!()
    };
    data.form.agriculture_payments = Some(payment * 100);
    FederalIncome::GovernmentPayment { data }
}
fn issue_keys(r: &FederalResult) -> Vec<&str> {
    r.issues.iter().map(|i| i.display.key.as_str()).collect()
}

#[test]
fn removing_a_farm_removes_its_relations_and_preserves_printed_documents() {
    let mut draft = rental_return(
        40_000,
        vec![rental_income(farm_rental_data(10_000, 0)), patr(2_000), government_program(1_000)],
    );
    let activity = draft.income[1].id;
    let keys = crate::related_forms::catalog_for(&draft, activity)
        .iter()
        .map(|source| source.id)
        .collect::<Vec<_>>();
    for key in keys {
        claim_related_component(&mut draft, activity, key);
    }
    let printed =
        draft.income.iter().filter(|entry| entry.id != activity).cloned().collect::<Vec<_>>();
    draft.income.retain(|entry| entry.id != activity);
    all_ticked(&mut draft);
    crate::interview::prune(&mut draft);
    assert_eq!(draft.income, printed);
}

#[test]
fn fixture_e_documents_route_to_the_only_farm_rental_and_to_the_chosen_farm() {
    // (i) The owner has only a farm rental: a 1099-PATR with box 1 $2,000 and a
    // 1099-G with box 7 $1,000 land on lines 2a/2b and 3a/3b and line 7 rises
    // by $3,000. The documents name no farm: the only one receives them.
    let mut base = farm_rental_data(10_000, 0);
    base.expenses = vec![rental_expense(BusinessExpenseCategory::Repairs, 1_000)];
    let d = rental_return(
        40_000,
        vec![rental_income(base.clone()), patr(2_000), government_program(1_000)],
    );
    let id = d.income[1].id;
    let r = supported(&d);
    assert_eq!(value(&r, &line_id(id, "cooperative_distributions")), 2_000);
    assert_eq!(value(&r, &line_id(id, "cooperative_taxable")), 2_000);
    assert_eq!(value(&r, &line_id(id, "program_payments")), 1_000);
    assert_eq!(value(&r, &line_id(id, "program_taxable")), 1_000);
    assert_eq!(value(&r, &line_id(id, "gross_income")), 13_000);
    assert_eq!(value(&r, &line_id(id, "net_income")), 12_000);
    // The routed payments are counted once: AGI = 40,000 + 12,000.
    assert_eq!(value(&r, "agi"), 52_000);
    assert_eq!(form_row(&r, TaxForm::Form4835, "2b"), Some(200_000));
    assert_eq!(form_row(&r, TaxForm::Form4835, "3b"), Some(100_000));

    // With a Schedule F farm added, a document that names none is ambiguous:
    // the estimate asks for the choice instead of using the lowest identity.
    let mut two = d.clone();
    add(&mut two, income(raised_farm(30_000, 5_000)));
    // The previously accepted cooperative recipient stays the rental; only
    // the unselected 1099-G becomes ambiguous when another farm is added.
    let farm_id = two.income.last().unwrap().id;
    let r = estimate(&two);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(issue_keys(&r).contains(&"ISSUE_RELATED_FORM_ACTIVITY_NOT_CHOSEN"));
    // Relating the source components to the farm moves only their tax recipient.
    let components = crate::related_forms::catalog_for(&two, farm_id)
        .iter()
        .map(|source| source.id)
        .collect::<Vec<_>>();
    for component in &components {
        claim_related_component(&mut two, farm_id, *component);
    }
    let r = supported(&two);
    assert_eq!(value(&r, &line_id(id, "gross_income")), 10_000);
    assert_eq!(value(&r, &format!("farm_{farm_id}_cooperative_taxable")), 2_000);
    assert_eq!(value(&r, &format!("farm_{farm_id}_program_payments")), 1_000);
    let FederalIncome::Business { data } =
        &mut two.income.iter_mut().find(|e| e.id == farm_id).unwrap().income
    else {
        unreachable!()
    };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.related.clear();
    for component in components {
        claim_related_component(&mut two, id, component);
    }
    let r = supported(&two);
    assert_eq!(value(&r, &line_id(id, "gross_income")), 13_000);
    assert_eq!(value(&r, &format!("farm_{farm_id}_cooperative_taxable")), 0);

    // A forged component key cannot mutate the accepted activity or its sources.
    let mut session = Interview::new(TEST_USER, two);
    let before = session.draft.clone();
    let figures = estimate(&before);
    let editor =
        crate::tests::support::client::income_pages::open_topic(&mut session, TaxSection::Farming);
    let mut rows = crate::tests::support::client::income_pages::page_documents(&editor);
    let row = rows.iter_mut().find(|row| row.id == id).unwrap();
    let IncomeSource::Record { value: crate::IncomeRecord::FarmRental(page) } = &mut row.document
    else {
        unreachable!()
    };
    page.related.forms[0].key = i64::MAX;
    let request = crate::tests::support::client::income_pages::save_topic(
        &mut session,
        TaxSection::Farming,
        rows,
    );
    crate::tests::support::assert::rejected_with(
        send(&mut session, request),
        "PROBLEM_RELATED_FORM_UNAVAILABLE",
    );
    assert_eq!(session.draft, before);
    assert_eq!(estimate(&session.draft), figures);
}

#[test]
fn a_farm_rental_transition_retains_known_routed_documents() {
    // An unentered required transition keeps the actual rental and its
    // documents, while completed tax waits for the named timing source.
    let mut special = farm_rental_data(10_000, 0);
    special.farm.specials.limitations = vec![3];
    let d =
        rental_return(40_000, vec![rental_income(special), patr(2_000), government_program(1_000)]);
    let r = estimate(&d);
    assert_eq!(r.tax, None);
    assert!(
        r.issues.iter().any(|issue| issue.display.key == "ISSUE_FARM_METHOD_TRANSITION_SOURCE")
    );
    let mut notices = vec![];
    let working = crate::unknowns::supported(&d, &mut notices).unwrap_or_else(|| d.clone());
    assert_eq!(working.income.len(), d.income.len());
    assert!(
        working
            .income
            .iter()
            .any(|entry| matches!(entry.income, FederalIncome::Cooperative { .. }))
    );
}

#[test]
fn a_farm_rentals_property_is_itemized_on_form_4562_like_a_businesss() {
    // A $10,000 five-year asset placed April 1, 2026 (bonus elected out, no
    // section 179): half-year 200% declining balance, 20% x 10,000 = $2,000 on
    // Form 4562 Part III line 19b (5-year class), line 22 total $2,000 to Form
    // 4835 line 12; the remaining basis is 10,000 - 2,000 = $8,000 and the
    // bonus row (line 14a) is zero.
    let mut asset =
        macrs_equipment(10_000, EquipmentClass::Five, EquipmentMethod::Regular, 20260401);
    asset.history = EquipmentHistory::New {
        section179: Section179::Ineligible,
        bonus: BonusElection::ElectOut,
        tables: false,
    };
    let mut data = farm_rental_data(12_000, 0);
    data.assets = vec![asset];
    let d = rental_return(40_000, vec![rental_income(data)]);
    let id = d.income[1].id;
    let r = supported(&d);
    assert_eq!(form_row(&r, TaxForm::Form4562, "19b"), Some(200_000));
    assert_eq!(form_row(&r, TaxForm::Form4562, "14a"), Some(0));
    assert_eq!(form_row(&r, TaxForm::Form4562, "22"), Some(200_000));
    assert_eq!(form_row(&r, TaxForm::Form4562, "12"), Some(0));
    assert_eq!(form_row(&r, TaxForm::Form4835, "12"), Some(200_000));
    assert_eq!(value(&r, &format!("business_{id}_asset_0_remaining_basis")), 8_000);
}

#[test]
fn section_179_on_a_farm_rental_is_zeroed_and_the_property_is_depreciated() {
    // (ii) A $10,000 five-year asset placed April 1, 2026 with a $5,000
    // section 179 election: the election is zeroed with a note and the asset
    // takes ordinary 200% half-year depreciation, 20% x 10,000 = $2,000.
    let mut asset =
        macrs_equipment(10_000, EquipmentClass::Five, EquipmentMethod::Regular, 20260401);
    asset.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 500_000 },
        bonus: BonusElection::ElectOut,
        tables: false,
    };
    let mut data = farm_rental_data(12_000, 0);
    data.assets = vec![asset];
    let d = rental_return(40_000, vec![rental_income(data)]);
    let id = d.income[1].id;
    let r = supported(&d);
    assert!(noted(&r, "WARN_FARM_RENTAL_SECTION_179_LEFT_OUT"));
    assert_eq!(value(&r, &line_id(id, "depreciation")), 2_000);
    assert_eq!(form_row(&r, TaxForm::Form4835, "12"), Some(200_000));
    assert_eq!(value(&r, &line_id(id, "net_income")), 10_000);
}

#[test]
fn farm_rental_vehicle_loan_interest_is_line_19b_and_line_8_holds_the_rest() {
    // Vehicle: 1,000 + 2,000 business miles of 4,000. Mileage 1,000 x 72.5c +
    // 2,000 x 76c = $2,245. Loan interest 2,000 x 3,000 / 4,000 = $1,500
    // (19b). Value-based tax 400 x 3,000 / 4,000 = $300 stays on line 8:
    // line 8 = $2,545. Mortgage interest $300 is 19a, other interest $500 19c.
    // Line 31 = 10,000 + 300 + 500 + 2,545 + 1,500 = $14,845; line 32 =
    // 40,000 - 14,845 = $25,155.
    let mut data = farm_rental_data(40_000, 0);
    data.vehicles.push(BusinessMileage {
        id: 0,
        interest: 200_000,
        property_tax: 40_000,
        ..native_business_mileage(1_000, 2_000, 4_000)
    });
    data.expenses = vec![
        rental_expense(BusinessExpenseCategory::Other { label: "Soil testing".into() }, 10_000),
        rental_expense(BusinessExpenseCategory::MortgageInterest, 300),
        rental_expense(BusinessExpenseCategory::OtherInterest, 500),
    ];
    let d = rental_return(40_000, vec![rental_income(data)]);
    let r = supported(&d);
    assert_eq!(form_row(&r, TaxForm::Form4835, "8"), Some(254_500));
    assert_eq!(form_row(&r, TaxForm::Form4835, "19a"), Some(30_000));
    assert_eq!(form_row(&r, TaxForm::Form4835, "19b"), Some(150_000));
    assert_eq!(form_row(&r, TaxForm::Form4835, "19c"), Some(50_000));
    assert_eq!(form_row(&r, TaxForm::Form4835, "31"), Some(1_484_500));
    assert_eq!(form_row(&r, TaxForm::Form4835, "32"), Some(2_515_500));
}

#[test]
fn conservation_costs_share_one_limit_with_the_owners_schedule_f_farms() {
    // Section 175 allows 25% of gross income from farming (Reg. 1.175-3 counts
    // a production-based rent). A farm with $40,000 of sales and a rental with
    // $20,000 of shares give a pooled limit of 25% x 60,000 = $15,000: the
    // rental's $12,000 of costs are all allowed. The rental alone would be
    // held to 25% x 20,000 = $5,000.
    let conservation = |amount: i64| {
        Some(FarmConservation::Deduct {
            costs: vec![ConservationCost {
                property: 0,
                paid_or_incurred: None,
                facts: conservation_facts(),
                label: "Terraces".into(),
                amount: amount * 100,
            }],
            assessments: vec![],
            carry: vec![],
            method: conservation_method(),
        })
    };
    let mut data = farm_rental_data(20_000, 0);
    data.farm.conservation = conservation(12_000);
    let alone = rental_return(40_000, vec![rental_income(data.clone())]);
    let id = alone.income[1].id;
    let r = supported(&alone);
    assert_eq!(value(&r, &line_id(id, "expenses")), 5_000);
    assert_eq!(value(&r, &format!("farm_{id}_conservation_carryforward")), 7_000);
    let together = rental_return(40_000, vec![rental_income(data), raised_farm(40_000, 0)]);
    let r = supported(&together);
    assert_eq!(value(&r, &line_id(id, "expenses")), 12_000);
    assert_eq!(value(&r, &format!("farm_{id}_conservation_deduction")), 12_000);
    assert_eq!(value(&r, &format!("farm_{id}_conservation_carryforward")), 0);
}

#[test]
fn a_crop_share_landlords_program_payments_count_in_the_conservation_limit() {
    // A crop-share landlord has $20,000 of shares and $5,000 of line 3 program
    // payments (gross farm income 25,000) beside the owner's $40,000 Schedule F
    // farm: the pooled section 175 limit is 25% x 65,000 = $16,250, so the
    // rental's $12,000 of costs are allowed whole (no "confirm the section 175
    // method" refusal). The rental alone is held to 25% x 25,000 = $6,250.
    let mut data = farm_rental_data(20_000, 5_000);
    data.farm.conservation = Some(FarmConservation::Deduct {
        costs: vec![ConservationCost {
            property: 0,
            paid_or_incurred: None,
            facts: conservation_facts(),
            label: "Terraces".into(),
            amount: 1_200_000,
        }],
        assessments: vec![],
        carry: vec![],
        method: conservation_method(),
    });
    let alone = rental_return(40_000, vec![rental_income(data.clone())]);
    let id = alone.income[1].id;
    let r = supported(&alone);
    assert_eq!(value(&r, &line_id(id, "expenses")), 6_250);
    assert_eq!(value(&r, &format!("farm_{id}_conservation_carryforward")), 5_750);
    let together = rental_return(40_000, vec![rental_income(data), raised_farm(40_000, 0)]);
    let r = supported(&together);
    assert_eq!(value(&r, &line_id(id, "expenses")), 12_000);
    assert_eq!(value(&r, &format!("farm_{id}_conservation_carryforward")), 0);
}

#[test]
fn a_farm_rental_loss_is_a_farming_loss_that_needs_the_carryback_choice() {
    // W-2 $10,000; rental line 1 $5,000 less $45,000 of repairs = ($40,000),
    // allowed $25,000 (active participation, MAGI below $100,000): AGI
    // (15,000), a net operating loss of the farming kind that needs the explicit
    // carryback choice (section 172(b)(1)(B)).
    let mut data = farm_rental_data(5_000, 0);
    data.expenses = vec![rental_expense(BusinessExpenseCategory::Repairs, 45_000)];
    let mut d = rental_return(10_000, vec![rental_income(data)]);
    unsupported(&d, "Farm loss");
    d.screening.as_mut().unwrap().farm_nol = Some(FarmNolElection::Waive);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), -15_000);
}

#[test]
fn a_farm_rental_counts_its_gross_income_in_the_farmer_estimated_tax_test() {
    // Pub. 505 chapter 2 and Schedule E line 42: Form 4835 line 7 is gross
    // farming income. W-2 $10,000 and a rental with $25,500 of gross income
    // and $17,500 of net: farming 25,500 of a gross 35,500 (the rental's net is
    // replaced by its gross), 25,500 x 3 >= 35,500 x 2, so the test is met.
    let mut data = farm_rental_data(24_000, 1_500);
    rental_a2_expenses(&mut data);
    let d = rental_return(10_000, vec![rental_income(data)]);
    let r = supported(&d);
    assert_eq!(value(&r, "farming_fishing_gross_income_return_year"), 25_500);
    assert_eq!(value(&r, "gross_income_return_year"), 35_500);
}

#[test]
fn a_farm_rental_enters_form_1116_line_3e_at_its_gross_income() {
    // Form 1116 line 3e: like a Schedule F farm, a farm rental's gross income
    // (Form 4835 line 7) replaces its net amount.
    let baseline = supported(&passive_fund_return(60_000));
    let mut d = passive_fund_return(60_000);
    add(&mut d, income(rental_income(farm_rental_data(24_000, 0))));
    let r = supported(&d);
    assert_eq!(
        form_1116_any(&r, false, "_gross_income_all_sources")
            - form_1116_any(&baseline, false, "_gross_income_all_sources"),
        24_000
    );
}

#[test]
fn land_sold_by_a_farm_rental_is_property_income_outside_the_passive_pool() {
    // A $150,000 sale of land held since 2016 less $10,000 of selling expenses
    // and a $100,000 basis (no conservation deductions): a $40,000 section
    // 1231 gain, long-term, in AGI beside the rental's own result. The
    // rental loses $14,000 and is suspended in part ($4,000 at MAGI 130,000
    // plus the gain): the gain does not release the rental's passive loss and
    // a note says so.
    let mut data = farm_rental_data(8_000, 0);
    data.expenses = vec![
        rental_expense(BusinessExpenseCategory::Fertilizers, 6_000),
        rental_expense(BusinessExpenseCategory::TaxesAndLicenses, 3_000),
        rental_expense(BusinessExpenseCategory::MortgageInterest, 9_000),
        rental_expense(BusinessExpenseCategory::Repairs, 4_000),
    ];
    let mut sale = farm_land_sale(20160616, 20260616);
    sale.conservation = 0;
    sale.deferral = false;

    let mut d = rental_return(130_000, vec![rental_income(data.clone())]);
    let FederalIncome::FarmRental { data: linked } = &mut d.income[1].income else {
        unreachable!()
    };
    set_land_sales(&mut linked.farm, vec![sale], &mut d.next_id);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let id = d.income[1].id;
    let r = supported(&d);
    assert_eq!(value(&r, &format!("farm_{id}_land_0_gain")), 40_000);
    assert_eq!(value(&r, &format!("farm_{id}_land_0_section1231")), 40_000);
    // The rental's passive limit is figured on its own: MAGI includes the
    // gain, so the allowance is 25,000 - 50% x 70,000 = 0 and all 14,000 is
    // suspended.
    assert_eq!(value(&r, &line_id(id, "suspended_loss")), 14_000);
    assert!(noted(&r, "WARN_FARM_RENTAL_PROPERTY_PAL_SOURCE"));
    assert_eq!(value(&r, "agi"), 170_000);
    // No land sale: the same return has the fixture B allowance.
    data.farm.land_sales.clear();
    let r = supported(&rental_return(130_000, vec![rental_income(data)]));
    assert_eq!(value(&r, "agi"), 120_000);
}

// ---- The Farm rental page and its neighbors on the Farming, Unemployment and
// Canceled debt pages.

fn zero_expenses() -> FarmExpenses {
    FarmExpenses {
        chemicals: 0,
        custom_hire: 0,
        employee_benefits: 0,
        feed: 0,
        fertilizers: 0,
        freight: 0,
        fuel: 0,
        insurance: 0,
        mortgage_interest: 0,
        other_interest: 0,
        labor: 0,
        pension: 0,
        equipment_rent: 0,
        other_rent: 0,
        repairs: 0,
        seeds: 0,
        storage: 0,
        supplies: 0,
        taxes: 0,
        utilities: 0,
        veterinary: 0,
        meals: 0,
        other: vec![],
    }
}
pub(in crate::tests) fn rental_page(production: i64) -> FarmRental {
    FarmRental {
        improvement_buildings: Vec::new(),
        interest_limit_election: None,
        allowed_interest_limit_election: Vec::new(),
        related: Default::default(),
        state_documents: Default::default(),
        name: alloc::string::String::new(),
        production: production * 100,
        program: 0,
        insurance: CropInsurance::None,
        other: 0,
        expenses: zero_expenses(),
        vehicles: vec![],
        assets: vec![],
        more: FarmRentalDetails {
            specials: (&farm_specials()).into(),
            land_basis: vec![],
            ccc: CccLoans::None,
            conservation: Conservation::None,
            land_sales: vec![],
            active_participation: true,
            suspended: SuspendedLoss { regular: 0, amt: 0, active_in_loss_year: true },
            qbi: RentalBusiness::No,
            qbi_group: alloc::string::String::new(),
            payroll: BusinessPayroll::No,
            at_risk: Some(crate::RentalAtRisk::All),
        },
    }
}
fn full_rental_page() -> FarmRental {
    let mut page = rental_page(24_000);
    page.name = "North 80 shares".into();
    page.program = 150_000;
    page.other = 50_000;
    page.insurance = CropInsurance::Payments {
        physical: vec![CropLoss { crop: "Grain".into(), amount: 700_000, year: 2026 }],
        price_only: 100_000,
        from_prior_year: 50_000,
        withholding: 10_000,
    };
    page.expenses = FarmExpenses {
        fertilizers: 300_000,
        chemicals: 120_000,
        taxes: 240_000,
        labor: 60_000,
        meals: 20_000,
        other: vec![OtherExpense { description: "Soil testing".into(), amount: 30_000 }],
        ..zero_expenses()
    };
    page.vehicles = vec![Vehicle {
        method: VehicleMethod::StandardMileage {
            id: 0,
            first_half: 1_000,
            second_half: 1_500,
            total: 2_500,
            more: MileageMore::default(),
        },
    }];
    page.assets = vec![Asset {
        farm_rental_use: None,
        allowed_farm_rental_use: alloc::vec::Vec::new(),
        macrs_source: None,
        id: 0,
        history_details: None,
        amt_section179_income_carry: None,
        allowed_amt_carry: Vec::new(),
        cost: 5_000_000,
        credit_reduction: 0,
        placed: 20150601,
        recovery: crate::AssetRecoveryPage::Gds {
            facts: crate::GdsAssetRecovery {
                macrs_question_context: None,
                source: None,
                class: AssetClass::Seven,
                history: AssetHistory::Continuing {
                    prior_depreciation: 4_000_000,
                    first_year_179: 0,
                    first_year_bonus: 0,
                    carryforward: 0,
                    mid_quarter: false,
                },
            },
        },
        business_percent: 100,
        acquired: None,
        disposal: AssetDisposal::No,
        investment_credit: crate::InvestmentCredit::No,
        earlier_business_percent: None,
        partial: None,
    }];
    page.more = FarmRentalDetails {
        specials: (&farm_specials()).into(),
        land_basis: vec![],
        ccc: CccLoans::Loans {
            method: CccMethod::Income,
            loans: vec![
                CccLoan::New { id: 0, principal: 1_200_000, outcome: CccOutcome::Outstanding },
                CccLoan::Earlier {
                    id: 0,
                    taxed: false,
                    principal: 800_000,
                    outcome: CccDisposition::Forfeited { year: 2026, surplus: 0, deficiency: 0 },
                },
            ],
            withholding: 0,
            source: ccc_source_input(&CccMethod::Income),
        },
        conservation: Conservation::Deduct {
            costs: vec![ConservationCostInput {
                property: 0,
                paid_or_incurred: None,
                facts: conservation_facts(),
                label: "Terraces".into(),
                amount: 100_000,
            }],
            assessments: vec![],
            carry: vec![],
            method: conservation_method(),
        },
        land_sales: vec![],
        active_participation: false,
        suspended: SuspendedLoss { regular: 100_000, amt: 90_000, active_in_loss_year: true },
        qbi: RentalBusiness::Yes { material_participation: false },
        qbi_group: "Farm group".into(),
        payroll: BusinessPayroll::No,
        at_risk: Some(crate::RentalAtRisk::All),
    };
    page
}
fn farming_page_documents(session: &mut Interview) -> (Vec<i32>, Vec<IncomeDocumentItem>, P) {
    let page = crate::tests::cross::interview::open_topic(session, TaxSection::Farming);
    let P::SaveIncomes { allowed_documents, documents, .. } = page.clone() else { unreachable!() };
    (
        allowed_documents,
        documents.documents.into_iter().map(crate::IncomeRecordItem::into_income).collect(),
        page,
    )
}

#[test]
fn farm_rental_page_maps_every_branch_onto_the_stored_rental_and_back() {
    use crate::tests::cross::interview::{accepted, document, save_topic};
    let original = empty_return(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut documents = vec![document(
        0,
        IncomeSource::Record { value: crate::IncomeRecord::FarmRental(full_rental_page()) },
    )];
    crate::tests::farm::interview::accept_farm_sale_parcels(&mut session, &mut documents);
    let IncomeSource::Record { value: crate::IncomeRecord::FarmRental(page) } =
        documents[0].document.clone()
    else {
        unreachable!()
    };
    accepted(
        &({
            let request = save_topic(&mut session, TaxSection::Farming, documents);
            send(&mut session, request)
        }),
    );
    accepted(&answer_crop_timing(&mut session, CropPaperTiming::Defer));
    accepted(&answer_crop_practice(&mut session, "Grain", 750_000));
    let d = session.draft.clone();
    assert_eq!(d.income.len(), 1);
    assert_eq!(d.income[0].label, "North 80 shares");
    let FederalIncome::FarmRental { data } = &d.income[0].income else {
        panic!("{:?}", d.income[0].income)
    };
    // Line 1, 3 and 6 receipts in form order on the cash record.
    assert_eq!(
        data.farm.accounting,
        FarmAccounting::Cash {
            receipts: vec![
                FarmReceipt::Raised { amount: 2_400_000 },
                FarmReceipt::Program { amount: 150_000 },
                FarmReceipt::Other { amount: 50_000 },
            ]
        }
    );
    assert!(data.farm.crp.is_empty() && data.farm.supplies.is_none());
    assert!(data.farm.specials.limitations.is_empty());
    // Line A, the passive history and line 34a are the rental's own answers.
    assert!(!data.active_participation && data.at_risk == Some(RentalAtRisk::All));
    assert_eq!(
        data.history,
        RentalHistory { regular: 100_000, amt: 90_000, ..RentalHistory::NONE }
    );
    assert_eq!(data.qbi_aggregation_group.as_deref(), Some("Farm group"));
    // F9: the 2015 seven-year asset uses 150% declining balance.
    assert_eq!(data.assets[0].method, EquipmentMethod::Declining150);
    assert_eq!(data.vehicles.len(), 1);
    let ccc = data.farm.ccc.as_ref().unwrap();
    assert_eq!(ccc.lots.len(), 2);
    // Saving assigns each loan its own identity; the reopened page carries them.
    assert!(ccc.lots.iter().all(|lot| lot.id != 0));
    assert_ne!(ccc.lots[0].id, ccc.lots[1].id);
    let mut page = page;
    assert_eq!(page.assets.len(), data.assets.len());
    for (expected, accepted) in page.assets.iter_mut().zip(&data.assets) {
        assert!(accepted.id > 0);
        expected.id = accepted.id;
    }
    crate::tests::support::client::bind_saved_mileage_ids(&mut page.vehicles, &data.vehicles);
    if let CccLoans::Loans { loans, .. } = &mut page.more.ccc {
        for (loan, lot) in loans.iter_mut().zip(&ccc.lots) {
            let (CccLoan::New { id, .. } | CccLoan::Earlier { id, .. }) = loan;
            *id = lot.id;
        }
    }
    // The reverse: the Farming page shows the same rental, the kind among its
    // choices and the farm and loan catalogs.
    let (allowed, documents, page_reply) = farming_page_documents(&mut session);
    assert!(
        allowed.contains(
            &(libertas_macros::variant_index!(crate::IncomeSourceKind::FarmRental) as i32)
        )
    );
    assert_eq!(documents.len(), 1);
    assert_eq!(
        documents[0].document,
        IncomeSource::Record { value: crate::IncomeRecord::FarmRental(page) }
    );
    let P::SaveIncomes { farms, ccc_lots, .. } = page_reply else { unreachable!() };
    let id = d.income[0].id;
    assert_eq!(farms.len(), 1);
    assert_eq!(farms[0].id, id);
    assert_eq!(
        status_text(&farms[0].label),
        "North 80 shares",
        "a named rental is listed by its name"
    );
    // The forfeited 2025 loan is the rental's second loan; its key is the
    // loan's own record identity.
    assert_eq!(ccc_lots.len(), 1);
    assert_eq!(ccc_lots[0].id, lot_id(&d, 0, 1));
    assert_eq!(
        status_text(&ccc_lots[0].label),
        "North 80 shares: loan 2, $8000.00 forfeited in 2026"
    );
}

#[test]
fn farming_page_row_names_an_unnamed_rental_and_previews_its_line_32() {
    use crate::tests::cross::interview::{accepted, document, save_topic};
    let original = empty_return(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut page = rental_page(24_000);
    page.expenses.repairs = 600_000;
    accepted(
        &({
            let request = save_topic(
                &mut session,
                TaxSection::Farming,
                vec![document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::FarmRental(page) },
                )],
            );
            send(&mut session, request)
        }),
    );
    assert_eq!(session.draft.income[0].label, "");
    let (_, documents, page_reply) = farming_page_documents(&mut session);
    // Line 32 = 24,000 - 6,000 = $18,000 in cents.
    assert_eq!(documents[0].preview, Some(1_800_000));
    let P::SaveIncomes { farms, .. } = page_reply else { unreachable!() };
    assert_eq!(status_text(&farms[0].label), format!("Farm rental {}", session.draft.income[0].id));
}

#[test]
fn documents_name_their_farm_and_their_forfeited_loan_through_the_pages() {
    use crate::tests::cross::interview::{accepted, document, save_topic};
    let original = empty_return(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    // Two farms: a Schedule F farm and a rental with a forfeited CCC loan.
    let mut page = rental_page(24_000);
    page.more.ccc = CccLoans::Loans {
        method: CccMethod::Loan,
        loans: vec![CccLoan::Earlier {
            id: 0,
            taxed: true,
            principal: 800_000,
            outcome: CccDisposition::Forfeited { year: 2026, surplus: 0, deficiency: 0 },
        }],
        withholding: 0,
        source: ccc_source_input(&CccMethod::Loan),
    };
    accepted(
        &({
            let request = save_topic(
                &mut session,
                TaxSection::Farming,
                vec![
                    document(
                        0,
                        IncomeSource::Record {
                            value: crate::IncomeRecord::Farm(
                                crate::tests::cross::interview::cash_farm(3_000_000, 0),
                            ),
                        },
                    ),
                    document(
                        0,
                        IncomeSource::Record { value: crate::IncomeRecord::FarmRental(page) },
                    ),
                ],
            );
            send(&mut session, request)
        }),
    );
    let (farm_id, rental_id) = (session.draft.income[0].id, session.draft.income[1].id);
    // A 1099-PATR naming the rental, a 1099-G naming the farm and a 1099-A
    // naming the rental's forfeited loan.
    let mut patr = Patr1099 {
        payer: Default::default(),
        recipient: Default::default(),
        patronage: 200_000,
        nonpatronage: 0,
        per_unit_retains: 0,
        withholding: 0,
        redeemed: 0,
        more: PatrMore::default(),
    };
    let raw = FederalCooperative { patronage: 200_000, ..Default::default() };
    patr.more.allocations.push(crate::cooperative::automatic(&raw).into());
    accepted(
        &({
            let request = save_topic(
                &mut session,
                TaxSection::Farming,
                vec![
                    document(
                        farm_id,
                        IncomeSource::Record {
                            value: crate::IncomeRecord::Farm(
                                crate::tests::cross::interview::cash_farm(3_000_000, 0),
                            ),
                        },
                    ),
                    document(
                        rental_id,
                        IncomeSource::Record {
                            value: crate::IncomeRecord::FarmRental(rental_page(24_000)),
                        },
                    ),
                ],
            );
            send(&mut session, request)
        }),
    );
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![document(0, IncomeSource::Farming(cooperative_document(patr.clone())))],
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    ));
    let stored = session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::Cooperative { data } => Some(data.clone()),
            _ => None,
        })
        .unwrap();
    claim_related_component(&mut session.draft, rental_id, stored.more.allocations[0].id);
    assert_eq!(
        crate::related_forms::effective_claimants(&session.draft)
            .get(&stored.more.allocations[0].id),
        Some(&rental_id)
    );
    patr.more.allocations[0].id = stored.more.allocations[0].id;
    assert_eq!(Patr1099::from(&stored).more, patr.more);
    let documents = crate::tests::support::client::income_pages::page_documents(
        &crate::tests::support::client::income_pages::open_documents(&mut session),
    );
    assert!(documents.iter().any(|item| item.document
        == expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::Farming(cooperative_document(patr.clone()))
        )));
}

#[test]
fn a_form_1099a_names_its_forfeited_lot_exactly_even_when_two_lots_share_a_principal() {
    // Two 2026 forfeitures of $8,000 each: the first lot (a 2025 loan never in
    // income) and the second (a 2025 loan included under section 77). The old
    // match by amount always took the first; the form names its lot by the
    // lot's own record identity, so reordering the farm's loan list does not
    // change which lot a stored form means.
    let mut data = farm_rental_data(10_000, 0);
    data.farm.ccc = Some(FarmCcc {
        method: CccMethod::Income,
        lots: vec![
            ccc_lot(
                CccOrigin::PriorExcluded,
                8_000,
                CccDisposition::Forfeited { year: 2026, surplus: 0, deficiency: 0 },
            ),
            ccc_lot(
                CccOrigin::PriorIncluded,
                8_000,
                CccDisposition::Forfeited { year: 2026, surplus: 0, deficiency: 0 },
            ),
        ],
        withholding: 0,
        source: ccc_source(&CccMethod::Income),
    });
    let base = rental_return(40_000, vec![rental_income(data)]);
    let without = supported(&base);
    let (first, second) = (lot_id(&base, 1, 0), lot_id(&base, 1, 1));
    // The first lot: never in income, so the forfeiture is the sale (Form 4835 4b/4c).
    let mut sale = base.clone();
    add(&mut sale, crate::tests::business::business::crop_acquisition(8_000, 6_000, first));
    let r = supported(&sale);
    assert!(noted(&r, "NOTE_CCC_FORFEITURE_IS_THE_SALE_RENTAL"));
    assert!(!noted(&r, "NOTE_CCC_FORFEITURE_ALREADY_REPORTED_RENTAL"));
    assert_eq!(r.tax, without.tax);
    // The second lot: reported under section 77 earlier, so it is not counted again.
    let mut reported = base.clone();
    add(&mut reported, crate::tests::business::business::crop_acquisition(8_000, 6_000, second));
    let r = supported(&reported);
    assert!(noted(&r, "NOTE_CCC_FORFEITURE_ALREADY_REPORTED_RENTAL"));
    assert!(!noted(&r, "NOTE_CCC_FORFEITURE_IS_THE_SALE_RENTAL"));
    assert_eq!(r.tax, without.tax);
    // Editing the loan list (here the order) keeps each key on its own lot: the
    // first lot is still the sale and the second still the reported loan.
    let mut swapped = base.clone();
    let FederalIncome::FarmRental { data } = &mut swapped.income[1].income else { unreachable!() };
    data.farm.ccc.as_mut().unwrap().lots.reverse();
    let mut sale = swapped.clone();
    add(&mut sale, crate::tests::business::business::crop_acquisition(8_000, 6_000, first));
    assert!(noted(&supported(&sale), "NOTE_CCC_FORFEITURE_IS_THE_SALE_RENTAL"));
    let mut reported = swapped.clone();
    add(&mut reported, crate::tests::business::business::crop_acquisition(8_000, 6_000, second));
    assert!(noted(&supported(&reported), "NOTE_CCC_FORFEITURE_ALREADY_REPORTED_RENTAL"));
    // An identity no lot has, or the wrong balance, is a correction note.
    for wrong in [
        crate::tests::business::business::crop_acquisition(8_000, 6_000, second + 100),
        crate::tests::business::business::crop_acquisition(7_999, 6_000, second),
    ] {
        let mut d = base.clone();
        add(&mut d, wrong);
        let r = estimate(&d);
        assert!(issue_keys(&r).contains(
            &"ISSUE_A_FORM_1099_A_FOR_A_CROP_MUST_NAME_A_CCC_LOAN_FORFEITED_IN_CURRENT_YEAR_WITH_THE_SAME_PRINCIPAL"
        ));
    }
}

#[test]
fn the_1099_a_and_1099_g_pages_keep_their_chosen_loan_and_farm() {
    use crate::tests::cross::interview::{accepted, document, save_topic};
    let original = empty_return(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut page = rental_page(24_000);
    page.more.ccc = CccLoans::Loans {
        method: CccMethod::Loan,
        loans: vec![CccLoan::Earlier {
            id: 0,
            taxed: false,
            principal: 800_000,
            outcome: CccDisposition::Forfeited { year: 2026, surplus: 0, deficiency: 0 },
        }],
        withholding: 0,
        source: ccc_source_input(&CccMethod::Loan),
    };
    accepted(
        &({
            let request = save_topic(
                &mut session,
                TaxSection::Farming,
                vec![document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::FarmRental(page) },
                )],
            );
            send(&mut session, request)
        }),
    );
    let rental_id = session.draft.income[0].id;
    // The 1099-G page names the rental for its box 7 payment.
    let g = crate::Form1099GPaper { agriculture_payments: Some(100_000), ..Default::default() };
    accepted(
        &({
            let request = crate::tests::support::client::income_pages::save_documents(
                &mut session,
                vec![document(
                    0,
                    IncomeSource::GovernmentPayment {
                        refund_state: None,
                        refund_locality: Default::default(),
                        value: g.clone(),
                    },
                )],
            );
            send(&mut session, request)
        }),
    );
    let stored = session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::GovernmentPayment { data } => Some(data.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        crate::related_forms::effective_claimants(&session.draft)
            .get(&stored.component_ids.agriculture),
        Some(&rental_id)
    );
    assert_ne!(stored.component_ids.agriculture, 0);
    let page = crate::tests::support::client::income_pages::open_documents(&mut session);
    assert_eq!(
        crate::tests::cross::interview::page_documents(&page)
            .into_iter()
            .find(|row| matches!(row.document, IncomeSource::GovernmentPayment { .. }))
            .unwrap()
            .document,
        expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::GovernmentPayment {
                refund_state: None,
                refund_locality: Default::default(),
                value: g
            }
        )
    );
    // The 1099-A page picks the rental's forfeited loan.
    let FederalIncome::FarmRental { data } = &session.draft.income[0].income else {
        unreachable!()
    };
    let lot = data.farm.ccc.as_ref().unwrap().lots[0].id;
    assert_ne!(lot, 0, "saving the farm assigns the loan its identity");
    let acquisition = acquisition_document(property_acquisition_source(Acquisition1099A {
        lender: crate::PrintedParty::from_name("Commodity Credit Corporation".into()),
        borrower: Default::default(),
        description: alloc::string::String::new(),
        date: 20260801,
        balance: 800_000,
        value: 600_000,
        liable: false,
        property: Some(AcquiredProperty::CccCrop { loan: lot }),
    }));
    accepted(
        &({
            let request = crate::tests::support::client::income_pages::save_documents(
                &mut session,
                vec![document(
                    0,
                    IncomeSource::CanceledDebt {
                        value: crate::CanceledDebtDocument::Acquisition1099A(acquisition.clone()),
                    },
                )],
            );
            crate::tests::support::client::income_pages::save_source_fixture_request(
                &mut session,
                request,
            )
        }),
    );
    let stored = session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::PropertyAcquisition { data } => Some(data.clone()),
            _ => None,
        })
        .unwrap();
    let stored = crate::PropertyAcquisition::from(&stored);
    assert_eq!(stored.crop, Some(lot));
    assert_eq!(stored.property, Some(TransferredProperty::Business));
    let page = crate::tests::support::client::income_pages::open_documents(&mut session);
    assert_eq!(
        crate::tests::cross::interview::page_documents(&page)
            .into_iter()
            .find(|row| matches!(row.document, IncomeSource::CanceledDebt { .. }))
            .unwrap()
            .document,
        expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::CanceledDebt {
                value: crate::CanceledDebtDocument::Acquisition1099A(acquisition)
            }
        )
    );
    // The reopened return is supported: the form is the loan's information
    // return, counted once.
    assert!(noted(&estimate(&session.draft), "NOTE_CCC_FORFEITURE_IS_THE_SALE_RENTAL"));
}

#[test]
fn a_real_estate_professional_keeps_a_farm_rental_passive_with_a_note() {
    // $200,000 of wages leave no special allowance. A qualified real estate
    // professional's Schedule E rentals are nonpassive, but a landlord paid in
    // crop shares did not take part in production, so the farm rental stays
    // passive: its $12,000 loss is suspended and one note says so.
    let mut data = farm_rental_data(8_000, 0);
    data.expenses = vec![rental_expense(BusinessExpenseCategory::Repairs, 20_000)];
    let mut d = rental_return(200_000, vec![rental_income(data)]);
    make_professional(&mut d, Owner::Taxpayer, false);
    let id = d.income[1].id;
    let r = supported(&d);
    assert_eq!(value(&r, &line_id(id, "suspended_loss")), 12_000);
    assert_eq!(value(&r, "agi"), 200_000);
    assert!(noted(&r, "WARN_FARM_RENTAL_PASSIVE_ASSUMED"));
}

// Line audit QBI-D6 (fix unit F12): a Form 4835 asset is qualified property for the UBIA limit
// only while its depreciable period has not ended before the close of 2026 (2025 i8995a,
// Determining your UBIA): the later of 10 years after it was placed in service and the last full
// year of its recovery period. A 7-year asset placed June 1, 2016 ended June 1, 2026 and counts
// for nothing; placed December 31, 2016 (the period ends at the close of 2026) or in 2017 it
// counts. Rent 500,000 (line 33 is 493,900, above the phase-in), no payroll, not materially
// participating: the component is 2.5% x 1,000,000 = 25,000 with the asset, 0 without (and no
// $400 minimum, which needs active QBI).
#[test]
fn qbi_d6_a_farm_rental_asset_keeps_ubia_through_the_close_of_its_tenth_year() {
    for (placed, expected) in [(20160601u32, 0), (20161231, 25_000), (20170601, 25_000)] {
        let mut data = farm_rental_data(500_000, 0);
        data.qbi_business = RentalBusiness::Yes { material_participation: false };
        data.assets.push(BusinessAsset {
            cost: 100_000_000,
            acquired: placed,
            placed,
            class: EquipmentClass::Seven,
            history: EquipmentHistory::Continuing {
                initial_section179: 0,
                carryforward: 0,
                prior_depreciation: 100_000_000,
                amt: EquipmentAmtHistory::Same,
                mid_quarter: false,
                schedule: EquipmentSchedule::Manual,
            },
            method: EquipmentMethod::StraightLine,
            ..bonus_equipment(0, 100)
        });
        let d = rental_return(10_000, vec![rental_income(data)]);
        let r = estimate(&d);
        assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{placed}: {:?}", r.issues);
        assert_eq!(value(&r, "qbi_deduction"), expected, "placed {placed}");
    }
}

#[test]
fn unit21_farm_rental_income_and_actual_land_sale_use_independent_fractions() {
    // A spouse's separate Texas parcel earns $12,000 rent and realizes
    // $50,000 gain ($150,000 proceeds less $100,000 basis), with no recapture.
    // Rental income divides; the separate-property gain remains the owner's.
    // A community parcel instead allocates both rent and gain in halves.
    for (state, answer, this_rent, this_gain) in [
        (CommunityState::Texas, CommunityShare::Separate, 6_000, 0),
        (CommunityState::Arizona, CommunityShare::Community, 6_000, 25_000),
    ] {
        for refund in [0, 1_000] {
            let mut d = complete(FederalStatus::Separate);
            d.setup.as_mut().unwrap().community_property =
                CommunityProperty::Yes { state, rule: CommunityRule::Halves };
            d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
            d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 600_000);
            d.income[0].community = CommunityShare::Community;
            let mut source = farm_rental_data(12_000, 0);
            let mut sale = farm_land_sale(20200101, 20260601);
            sale.selling = 0;
            sale.conservation = 0;
            set_land_sales(&mut source.farm, vec![sale], &mut d.next_id);
            let parcel = source.farm.land_basis[0].id;
            let mut entry = income(FederalIncome::FarmRental { data: source });
            entry.owner = crate::DocumentOwner::Spouses;
            entry.community = answer;
            add(&mut d, entry);
            let id = d.income.last().unwrap().id;
            if refund != 0 {
                let mut cooperative =
                    FederalCooperative { patronage: refund * 100, ..Default::default() };
                let mut allocation = crate::cooperative::automatic(&cooperative);
                allocation.properties.push(CoopPropertyRefund {
                    distribution: CoopDistributionBox::Patronage,
                    property: Some(parcel),
                    source: "Synthetic parcel purchase rebate".into(),
                    amount: refund * 100,
                    qualified_items: 0,
                    sstb_items: 0,
                    qualified_payments: 0,
                    treatment: CoopPropertyTreatment::Purchase,
                });
                cooperative.allocations.push(allocation);
                let mut entry =
                    income(FederalIncome::Cooperative { data: cooperative_source(cooperative) });
                entry.owner = crate::DocumentOwner::Spouses;
                entry.community = answer;
                add(&mut d, entry);
            }
            let this_gain = this_gain + if this_gain == 0 { 0 } else { refund / 2 };
            let r = supported(&d);
            assert_eq!(value(&r, &format!("farm_rental_{id}_gross_income")), this_rent);
            let property = crate::farm_land::sources(&d, false)
                .into_iter()
                .find(|(source, _)| *source == id)
                .unwrap()
                .1;
            assert_eq!(property.gain, this_gain);
            assert_eq!(
                crate::section1231::farmer_gross_property(&d, &Default::default()).unwrap(),
                Some(this_gain),
            );
            assert_eq!(value(&r, "agi"), 30_000 + this_rent + this_gain);
            assert_eq!(value(&r, &format!("form8958_gain_{id}_total")), 50_000 + refund);
            assert_eq!(value(&r, &format!("form8958_gain_{id}_this_return")), this_gain);
            crate::tests::federal::schedules_1_3::assert_form_8958_divided_sources_conserve(&d, &r);
        }
    }
}

#[test]
fn current_property_gain_and_operating_loss_share_one_passive_source_in_each_system() {
    let d = farm_rental_passive_property_return();
    let entry = d
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::FarmRental { .. }))
        .unwrap();
    let id = entry.id;
    let result = supported(&d);
    assert_eq!(value(&result, "agi"), 130_000);
    assert_eq!(value(&result, "schedule_e_farm_rental_total"), -10_000);
    assert!(!noted(&result, "WARN_FARM_RENTAL_PROPERTY_PAL_SOURCE"));
    assert_eq!(form_row(&result, TaxForm::Form4835, "34c"), Some(-1_000_000));
    for (amt, gain, operating, suspended) in
        [(false, 10_000, -10_000, 4_000), (true, 12_000, -12_000, 4_000)]
    {
        let mut report = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 130_000, &mut report, amt);
        assert!(report.issues.is_empty(), "{:?}", report.issues);
        assert_eq!(pool.farm_rental_property[&id].gain, gain);
        assert_eq!(pool.farm_rental, operating);
        assert_eq!(pool.post_pal.as_ref().unwrap().suspended_cents, suspended * 100);
    }
}

#[test]
fn property_use_unknown_other_tenant_and_development_facts_never_release_operating_pal() {
    for answer in [
        None,
        Some(FarmRentalPropertyUse::NoneOfThese),
        Some(FarmRentalPropertyUse::OnlyThisRental {
            tenant_connection: None,
            value_enhancement: Some(FarmRentalValueEnhancement::NoParticipation),
        }),
        Some(FarmRentalPropertyUse::OnlyThisRental {
            tenant_connection: Some(FarmRentalTenantConnection::NoOwnershipOrServices),
            value_enhancement: None,
        }),
        Some(FarmRentalPropertyUse::OnlyThisRental {
            tenant_connection: Some(FarmRentalTenantConnection::OwnershipOrServices),
            value_enhancement: Some(FarmRentalValueEnhancement::NoParticipation),
        }),
        Some(FarmRentalPropertyUse::OnlyThisRental {
            tenant_connection: Some(FarmRentalTenantConnection::NoOwnershipOrServices),
            value_enhancement: Some(FarmRentalValueEnhancement::Participation),
        }),
    ] {
        let mut d = farm_rental_passive_property_return();
        let FederalIncome::FarmRental { data } = &mut d.income.last_mut().unwrap().income else {
            unreachable!()
        };
        data.farm.land_sales[0].farm_rental_use = answer;
        let result = supported(&d);
        assert_eq!(value(&result, "agi"), 140_000);
        assert_eq!(value(&result, "schedule_e_farm_rental_total"), 0);
        assert_eq!(
            result
                .review_notes
                .iter()
                .filter(|note| note.display.key == "WARN_FARM_RENTAL_PROPERTY_PAL_SOURCE")
                .count(),
            1
        );
    }
}

#[test]
fn property_gain_correction_zero_and_positive_net_preserve_named_cohort_boundaries() {
    for (proceeds, agi, operating, notice) in [
        (10_000_000, 130_000, 0, false),
        (11_500_000, 145_000, 0, true),
        (10_500_000, 130_000, -5_000, false),
    ] {
        let mut d = farm_rental_passive_property_return();
        let FederalIncome::FarmRental { data } = &mut d.income.last_mut().unwrap().income else {
            unreachable!()
        };
        data.farm.land_sales[0].proceeds = proceeds;
        // Both tax systems use the actual corrected basis for the zero-gain boundary.
        data.farm.land_basis[0].amt = data.farm.land_basis[0].regular;
        let result = supported(&d);
        assert_eq!(value(&result, "agi"), agi);
        assert_eq!(value(&result, "schedule_e_farm_rental_total"), operating);
        assert_eq!(noted(&result, "WARN_FARM_RENTAL_PROPERTY_PAL_SOURCE"), notice);
    }
}

#[test]
fn newly_linked_sold_property_returns_an_unaccepted_complete_editor_with_questions() {
    use crate::tests::support::client::income_pages::open_topic;
    let d = farm_rental_passive_property_editor_return();
    let entry = d.income.last().unwrap();
    let FederalIncome::FarmRental { data } = &entry.income else { unreachable!() };
    let mut page = crate::farm_page::farm_rental_item(&entry.label, data).unwrap();
    let mut accepted_draft = d.clone();
    let FederalIncome::FarmRental { data } = &mut accepted_draft.income.last_mut().unwrap().income
    else {
        unreachable!()
    };
    data.farm.land_sales.clear();
    let mut session = Interview::new(TEST_USER, accepted_draft.clone());
    page.more.land_sales[0].id = 0;
    page.more.land_sales[0].farm_rental_use = None;
    page.more.land_sales[0].allowed_farm_rental_use.clear();
    // Edit the actual complete Farming request, retaining its separately owned
    // Business-property losses answer from the accepted lookback records.
    let mut request = open_topic(&mut session, TaxSection::Farming);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let row = documents.documents.iter_mut().find(|row| row.id == entry.id).unwrap();
    row.document = crate::IncomeRecord::FarmRental(page);
    let Reply::Edit(corrected) = send(&mut session, request) else {
        panic!("new source questions must return the complete editor");
    };
    let P::SaveIncomes { documents, warnings, .. } = &corrected else { unreachable!() };
    assert_eq!(session.draft, accepted_draft);
    assert!(!warnings.is_empty());
    let row = documents.documents.iter().find(|row| row.id == entry.id).unwrap();
    let crate::IncomeRecord::FarmRental(page) = &row.document else { unreachable!() };
    assert_eq!(page.more.land_sales[0].allowed_farm_rental_use, vec![0, 1]);
    assert!(page.more.land_sales[0].farm_rental_use.is_none());
    // Explicit unavailable is optional: the complete corrective editor can save
    // with null, and the recorded unknown cannot unlock passive gain.
    crate::tests::cross::interview::accepted(&send(&mut session, corrected));
    assert!(noted(&supported(&session.draft), "WARN_FARM_RENTAL_PROPERTY_PAL_SOURCE"));
}

#[test]
fn admitted_property_uses_regular_passive_magi_before_the_special_allowance() {
    let mut d = farm_rental_passive_property_return();
    let FederalIncome::FarmRental { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.active_participation = true;
    let result = supported(&d);
    assert_eq!(value(&result, "rental_magi"), 130_000);
    assert_eq!(value(&result, "schedule_e_farm_rental_total"), -14_000);
    assert_eq!(value(&result, "agi"), 126_000);
}

#[test]
fn a_whole_asset_sale_offsets_current_loss_without_releasing_the_activity_remainder() {
    let d = farm_rental_passive_asset_return();
    let result = supported(&d);
    assert_eq!(value(&result, "schedule_e_farm_rental_total"), -10_000);
    assert_eq!(value(&result, "agi"), 130_000);
    let id = d.income.last().unwrap().id;
    assert_eq!(value(&result, &line_id(id, "suspended_loss")), 4_000);
}

#[test]
fn inapplicable_property_projection_hides_the_question_and_retains_accepted_use_records() {
    let d = farm_rental_passive_asset_return();
    let entry = d.income.last().unwrap();
    let FederalIncome::FarmRental { data } = &entry.income else { unreachable!() };
    let sold =
        data.assets.iter().position(|asset| !matches!(asset.disposal, AssetDisposal::No)).unwrap();
    let canonical: crate::CanonicalAssetPage = data.assets[sold].clone().into();
    let round_trip: BusinessAsset = canonical.into();
    assert_eq!(round_trip.farm_rental_use, data.assets[sold].farm_rental_use);
    let mut page = crate::farm_page::farm_rental_item(&entry.label, data).unwrap();
    page.assets[sold].disposal = AssetDisposal::No;
    let mut record = crate::IncomeRecord::FarmRental(page);
    assert!(!crate::farm_rental::refresh_property_questions(&mut record));
    let crate::IncomeRecord::FarmRental(page) = record else { unreachable!() };
    assert!(page.assets[sold].allowed_farm_rental_use.is_empty());
    assert!(page.assets[sold].farm_rental_use.is_none());
    let mut next = entry.clone();
    next.income = FederalIncome::FarmRental { data: crate::farm_page::farm_rental_record(&page) };
    crate::farm_rental::retain_inapplicable_property_facts(&mut next, &d.income);
    let FederalIncome::FarmRental { data: after } = next.income else { unreachable!() };
    assert_eq!(after.assets[sold].farm_rental_use, data.assets[sold].farm_rental_use);
}

#[test]
fn positive_amt_net_or_retained_passive_loss_keeps_both_systems_outside_current_gain_cohort() {
    for prior in [false, true] {
        let mut d = farm_rental_passive_property_return();
        let FederalIncome::FarmRental { data } = &mut d.income.last_mut().unwrap().income else {
            unreachable!()
        };
        if prior {
            data.history.regular = 100;
        } else {
            data.farm.land_basis[0].amt = 9_000_000;
        }
        let result = supported(&d);
        assert_eq!(value(&result, "agi"), 140_000);
        assert_eq!(value(&result, "schedule_e_farm_rental_total"), 0);
        assert!(noted(&result, "WARN_FARM_RENTAL_PROPERTY_PAL_SOURCE"));
    }
    let mut d = farm_rental_passive_property_return();
    let FederalIncome::FarmRental { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.farm.land_sales.clear();
    let result = supported(&d);
    assert_eq!(value(&result, "agi"), 130_000);
    assert_eq!(value(&result, "schedule_e_farm_rental_total"), 0);
    assert!(!noted(&result, "WARN_FARM_RENTAL_PROPERTY_PAL_SOURCE"));
}
