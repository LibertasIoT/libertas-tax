use crate::tests::support::assert::line_amount as cents;
// Form 3115 section 481(a) adjustment (unit 481a): fixtures A to E of the
// design map, hand-computed in the comments (Rev. Proc. 2015-13 section
// 7.03, cumulative `fraction(total, k, period)` so the yearly parts add to
// line 26 exactly), each also checked against the same return without the
// record. Every amount is synthetic.
use super::*;
use crate::tests::cross::interview::{
    accepted, business, cash_farm, document, farm_save, open_topic, page_documents, rejected_with,
    rental_page, save_businesses, save_rentals, save_topic, stored_business, stored_farm,
    stored_rentals,
};
use crate::{Adjustment481, MethodChange, MethodChangeConsent};
use alloc::{format, string::String};

// A calculation row in cents; the shared `value` helper divides by 100.
fn cents_of(r: &FederalResult, id: &str) -> Option<i64> {
    r.lines.iter().find(|l| l.id == id).map(|l| l.amount)
}
fn change(year: u32, consent: MethodChangeConsent, adjustment: Adjustment481) -> MethodChange {
    MethodChange { id: 0, year, consent, adjustment, amt: crate::MethodChangeAmt::Same }
}
fn dcn(dcn: u32) -> MethodChangeConsent {
    MethodChangeConsent::Automatic { dcn }
}
fn four(amount: i64, rest_in_current_year: bool) -> Adjustment481 {
    Adjustment481::FourYears { amount, rest_in_current_year }
}
fn save_farm(session: &mut Interview, farm: Farm) -> Reply {
    let request = farm_save(&session.draft.clone(), vec![document(0, IncomeDocument::Farm(farm))]);
    send(session, request)
}
fn other_expense(label: &str, amount: i64) -> FederalBusinessExpense {
    ordinary_business_expense(BusinessExpenseCategory::Other { label: label.into() }, amount)
}
// The Schedule C business at income position `at`.
fn business_mut(d: &mut FederalDraft, at: usize) -> &mut FederalBusiness {
    let FederalIncome::Business { data } = &mut d.income[at].income else {
        panic!("not a business")
    };
    data
}
// A single filer with one Schedule C business (income position 1) holding the
// changes.
fn with_business(
    receipts_dollars: i64,
    changes: Vec<MethodChange>,
    edit: impl FnOnce(&mut FederalBusiness),
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    let mut entry = cash_business(receipts_dollars);
    let FederalIncome::Business { data } = &mut entry else { panic!() };
    data.method_changes = changes;
    edit(data);
    add(&mut d, income(entry));
    d
}
fn receipts_mut(data: &mut FederalBusiness) -> &mut i64 {
    match &mut data.activity {
        BusinessActivity::Ordinary { receipts, .. } => receipts,
        _ => panic!("not an ordinary business"),
    }
}
// The printed return rows outside the lines where the record and its
// equivalent differ.
fn printed(r: &FederalResult, skip: &[(TaxForm, &str)]) -> Vec<(TaxForm, String, i64)> {
    let mut rows: Vec<_> = crate::draft_return(r)
        .into_iter()
        .filter_map(|row| {
            let line = row.form_line?;
            (!skip.iter().any(|(f, l)| *f == line.form && *l == line.line))
                .then_some((line.form, line.line, row.amount))
        })
        .collect();
    rows.sort();
    rows
}
// The equivalence: the same return without the record, with the 2026 part
// already in the receipts, expenses or rent, prints every other line alike
// (Schedule 1, Schedule SE, Form 8995 and Form 1040 included).
fn same_return(a: &FederalResult, b: &FederalResult, skip: &[(TaxForm, &str)]) {
    assert_eq!(printed(a, skip), printed(b, skip));
    assert_eq!(a.tax, b.tax);
    assert_eq!(a.refund, b.refund);
    assert_eq!(a.state, b.state);
    for id in ["agi", "self_employment_tax", "qbi_deduction", "total_tax", "taxable_income"] {
        assert_eq!(cents_of(a, id), cents_of(b, id), "{id}");
    }
}
fn notes(r: &FederalResult, key: &str) -> Vec<crate::federal_model::ReviewNote> {
    r.review_notes.iter().filter(|n| n.display.key == key).cloned().collect()
}
const SCHEDULE_C_INCOME: &[(TaxForm, &str)] = &[
    (TaxForm::ScheduleC, "1"),
    (TaxForm::ScheduleC, "3"),
    (TaxForm::ScheduleC, "5"),
    (TaxForm::ScheduleC, "6"),
    (TaxForm::ScheduleC, "27b"),
    (TaxForm::ScheduleC, "28"),
];

// Fixture A. Accrual business since 2025: line 1 $80,000.00, office expense
// (line 18) $20,000.00; change year 2025, DCN 122, +$61,234.57 over four
// years, 2026 is year 2.
//   through(2025) = (6,123,457 + 2) / 4 = 1,530,864 ($15,308.64 on the 2025 return)
//   through(2026) = (12,246,914 + 2) / 4 = 3,061,729; 2026 part 1,530,865
//   remaining = 6,123,457 - 3,061,729 = 3,061,728
//   line 6 = 1,530,865; line 7 = 9,530,865; line 28 = 2,000,000; line 31 = 7,530,865
#[test]
fn a_four_year_spread_in_its_second_year_lands_on_schedule_c_line_6() {
    let record = change(2025, dcn(122), four(6_123_457, false));
    let d = with_business(80_000, vec![record.clone()], |b| {
        b.activity = BusinessActivity::Ordinary {
            returns: 0,
            receipts: 8_000_000,
            accounting: BusinessAccounting::Accrual,
        };
        b.expenses.push(ordinary_business_expense(BusinessExpenseCategory::Office, 2_000_000));
    });
    let id = d.income[1].id;
    let r = supported(&d);
    assert_eq!(cents(&r, &format!("business_{id}_gross_receipts")), 8_000_000);
    assert_eq!(cents(&r, &format!("business_{id}_other_income")), 1_530_865);
    assert_eq!(cents(&r, &format!("business_{id}_gross_income")), 9_530_865);
    assert_eq!(cents(&r, &format!("business_{id}_method_change_0")), 1_530_865);
    assert_eq!(cents(&r, &format!("business_{id}_method_change_0_remaining")), 3_061_728);
    assert_eq!(cents(&r, &format!("business_{id}_total_expenses")), 2_000_000);
    assert_eq!(cents(&r, &format!("business_{id}_net_profit")), 7_530_865);
    // The same return without the record and the part in line 1.
    let mut plain = with_business(80_000, vec![], |b| {
        b.activity = BusinessActivity::Ordinary {
            returns: 0,
            receipts: 9_530_865,
            accounting: BusinessAccounting::Accrual,
        };
        b.expenses.push(ordinary_business_expense(BusinessExpenseCategory::Office, 2_000_000));
    });
    plain.income[1].id = id;
    let equivalent = supported(&plain);
    same_return(&r, &equivalent, SCHEDULE_C_INCOME);
    assert!(value(&r, "qbi_deduction") > 0);
    // One review note names the year and the DCN, opens the record, and is
    // there once.
    let found = notes(&r, "NOTE_METHOD_CHANGE_FORM_3115");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].display.args[1], crate::MessageArg::Number(2025));
    let crate::MessageArg::Nested(consent) = &found[0].display.args[2] else {
        panic!("{:?}", found[0].display.args)
    };
    assert_eq!(consent.key, "METHOD_CHANGE_DCN");
    assert_eq!(consent.args, vec![crate::MessageArg::Number(122)]);
    assert!(matches!(
        &found[0].context,
        Some(crate::IssueContext::Scoped {
            identity: crate::federal_model::NoteIdentity::Item {
                collection: "method_changes",
                position: 0,
                ..
            },
            ..
        })
    ));
    assert!(notes(&equivalent, "NOTE_METHOD_CHANGE_FORM_3115").is_empty());
    // Reopening shows the same record; saving unchanged keeps the revision.
    let mut reopened = restart(d.clone()).draft;
    assert_eq!(reopened, d);
    let mut expected = record;
    expected.id = business_mut(&mut reopened, 1).method_changes[0].id;
    assert!(expected.id > 0);
    assert_eq!(business_mut(&mut reopened, 1).method_changes, vec![expected]);
}

// The yearly parts of 61,234.57 over four years are 15,308.64, 15,308.65,
// 15,308.64 and 15,308.64 and add to line 26 exactly, whichever year of change
// 2026 is.
#[test]
fn the_parts_of_a_spread_add_to_line_26_exactly() {
    let total = 6_123_457;
    for (year, part, remaining) in [
        (2023, 1_530_864, 0),
        (2024, 1_530_864, 1_530_864),
        (2025, 1_530_865, 3_061_728),
        (2026, 1_530_864, 4_592_593),
    ] {
        let record = change(year, dcn(7), four(total, false));
        assert_eq!(record.amount_current_year(), part, "{year}");
        assert_eq!(record.remaining_after_current_year(), remaining, "{year}");
        // Taken before 2026 + this year's part + the rest is line 26.
        let before = total - part - remaining;
        assert_eq!(
            before,
            match year {
                2023 => 4_592_593,
                2024 => 3_061_729,
                2025 => 1_530_864,
                _ => 0,
            }
        );
        assert_eq!(record.rule(), None);
    }
    // A decrease is negative and taken in full in its year.
    let decrease = change(2026, dcn(7), Adjustment481::Decrease { amount: 1_234_567 });
    assert_eq!(decrease.amount_current_year(), -1_234_567);
    assert_eq!(decrease.remaining_after_current_year(), 0);
}

// Fixture B. Cash business, line 1 $50,000.00, Part V "Bank fees" $1,000.00;
// change year 2026, DCN 7, decrease $12,345.67.
//   line 27b = 100,000 + 1,234,567 = 1,334,567; line 31 = 5,000,000 - 1,334,567 = 3,665,433
#[test]
fn b_a_decrease_is_deducted_in_full_in_part_v() {
    let record = change(2026, dcn(7), Adjustment481::Decrease { amount: 1_234_567 });
    let d = with_business(50_000, vec![record], |b| {
        b.expenses.push(other_expense("Bank fees", 100_000));
    });
    let id = d.income[1].id;
    let r = supported(&d);
    assert_eq!(cents(&r, &format!("business_{id}_expense_line27b_other")), 1_334_567);
    assert_eq!(cents(&r, &format!("business_{id}_method_change_0")), -1_234_567);
    assert!(cents_of(&r, &format!("business_{id}_method_change_0_remaining")).is_none());
    assert_eq!(cents(&r, &format!("business_{id}_other_income")), 0);
    assert_eq!(cents(&r, &format!("business_{id}_net_profit")), 3_665_433);
    assert_eq!(
        printed(&r, &[])
            .iter()
            .find(|row| (row.0, row.1.as_str()) == (TaxForm::Schedule1, "3"))
            .unwrap()
            .2,
        3_665_400
    );
    let mut plain = with_business(50_000, vec![], |b| {
        b.expenses.push(other_expense("Bank fees", 100_000));
        b.expenses.push(other_expense("Equivalent deduction", 1_234_567));
    });
    plain.income[1].id = id;
    same_return(&r, &supported(&plain), &[(TaxForm::ScheduleC, "27b"), (TaxForm::ScheduleC, "28")]);
    let found = notes(&r, "NOTE_METHOD_CHANGE_FORM_3115");
    assert_eq!(found.len(), 1);
}

// Fixture B (ii) and (iii): the same decrease on a rental, and with personal
// use.
#[test]
fn b_a_rental_decrease_is_a_line_19_expense_and_personal_use_leaves_it_out() {
    let record = change(2026, dcn(107), Adjustment481::Decrease { amount: 500_000 });
    let mut base = complete(FederalStatus::Single);
    add(&mut base, income(FederalIncome::Rental { data: rental(24_000, 1_000) }));
    let id = base.income[1].id;
    let plain = supported(&base);
    let mut d = base.clone();
    let FederalIncome::Rental { data } = &mut d.income[1].income else { panic!() };
    data.method_changes = vec![record.clone()];
    data.method_changes[0].id = d.next_id;
    d.next_id += 1;
    let r = supported(&d);
    let row = |r: &FederalResult, suffix: &str| cents(r, &format!("rental_{id}_{suffix}"));
    assert_eq!(row(&r, "expense_line19_other"), row(&plain, "expense_line19_other") + 500_000);
    assert_eq!(row(&r, "line21_income"), row(&plain, "line21_income") - 500_000);
    assert_eq!(row(&r, "line3_rents"), row(&plain, "line3_rents"));
    assert_eq!(row(&r, "line20_total_expenses"), row(&plain, "line20_total_expenses") + 500_000);
    assert_eq!(row(&r, "method_change_0"), -500_000);
    assert_eq!(notes(&r, "NOTE_METHOD_CHANGE_FORM_3115").len(), 1);
    // The same return with the deduction as a line 19 expense.
    let mut equivalent = base.clone();
    let FederalIncome::Rental { data } = &mut equivalent.income[1].income else { panic!() };
    data.expenses.push(crate::extended_model::RentalExpense {
        category: crate::extended_model::RentalExpenseCategory::Other {
            label: "Equivalent".into(),
        },
        amount: 500_000,
        treatment: BusinessExpenseKind::Ordinary,
    });
    same_return(&r, &supported(&equivalent), &[]);
    // Personal use: the record is left out with one notice scoped to the
    // rental and the record; the result is the no-record result.
    let mut personal = d.clone();
    let FederalIncome::Rental { data } = &mut personal.income[1].income else { panic!() };
    data.personal_use = crate::extended_model::RentalPersonalUse::Yes {
        fair_rental_days: 300,
        personal_days: 10,
        overlap_days: 0,
        direct_expenses: 0,
    };
    let mut personal_plain = personal.clone();
    let FederalIncome::Rental { data } = &mut personal_plain.income[1].income else { panic!() };
    data.method_changes.clear();
    let left_out = supported(&personal);
    let expected = supported(&personal_plain);
    assert_eq!(notes(&left_out, "WARN_METHOD_CHANGE_LEFT_OUT").len(), 1);
    assert!(notes(&left_out, "NOTE_METHOD_CHANGE_FORM_3115").is_empty());
    assert_eq!(
        cents(&left_out, &format!("rental_{id}_line21_income")),
        cents(&expected, &format!("rental_{id}_line21_income"))
    );
    assert!(cents_of(&left_out, &format!("rental_{id}_method_change_0")).is_none());
    assert_eq!(left_out.state, expected.state);
    same_return(&left_out, &expected, &[]);
    // Home-use carryovers with no personal use this year are stored beside a
    // plain No (F3, 2026-10-09), so nothing personal happened and the record
    // stays: no left-out notice, the 481(a) deduction applies, and the
    // carryovers (1,000 operating, 2,000 depreciation) are deducted in full.
    let mut carried = d.clone();
    let FederalIncome::Rental { data } = &mut carried.income[1].income else { panic!() };
    data.home_operating_carryover = 100_000;
    //2026-10-19: both synthetic tax worksheets explicitly carry depreciation only.
    data.home_depreciation_carryover = crate::SourceCents::Reported { amount: 200_000 };
    data.amt_home_depreciation_carryover = crate::SourceCents::Reported { amount: 200_000 };
    let r_carried = supported(&carried);
    assert!(notes(&r_carried, "WARN_METHOD_CHANGE_LEFT_OUT").is_empty());
    assert_eq!(notes(&r_carried, "NOTE_METHOD_CHANGE_FORM_3115").len(), 1);
    assert_eq!(
        cents(&r_carried, &format!("rental_{id}_line21_income")),
        cents(&r, &format!("rental_{id}_line21_income")) - 100_000 - 200_000
    );
    assert_eq!(cents(&r_carried, &format!("rental_{id}_method_change_0")), -500_000);
}

// Fixture B (i): a decrease needs its year of change to be 2026. The client
// rule and the server twin both reject it and the draft is unchanged.
#[test]
fn b_a_decrease_from_an_earlier_year_has_nothing_in_2026() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = session.draft.clone();
    let mut page = business(5_000_000);
    page.more.method_changes =
        vec![change(2025, dcn(7), Adjustment481::Decrease { amount: 1_234_567 })];
    rejected_with(
        save_businesses(&mut session, vec![page]),
        "METHOD_CHANGE_NOTHING_IN_CURRENT_YEAR",
    );
    assert_eq!(session.draft, before);
    let mut page = business(5_000_000);
    page.more.method_changes =
        vec![change(2026, dcn(7), Adjustment481::Decrease { amount: 1_234_567 })];
    accepted(&save_businesses(&mut session, vec![page.clone()]));
    let (_, stored) = stored_business(&session);
    page.more.method_changes[0].id = stored.method_changes[0].id;
    assert!(page.more.method_changes[0].id > 0);
    assert_eq!(stored.method_changes, page.more.method_changes);
    // The page reopens with the same record and saving it again changes nothing.
    let reopened = crate::tests::cross::interview::reopened_businesses(session.draft.clone());
    assert_eq!(reopened, vec![page]);
    let revision = session.draft.revision;
    let mut again = restart(session.draft.clone());
    let items = page_documents(&open_topic(&mut again, TaxSection::SelfEmployment)).to_vec();
    let request = save_topic(&again.draft.clone(), TaxSection::SelfEmployment, items);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft.revision, revision);
}

// Fixture C. Line 1 $60,000.00, no expenses; year 2026, DCN 233, one-year
// election of $49,999.99.
//   line 6 = 4,999,999; line 31 = 10,999,999; Schedule 1 line 3 = $110,000
#[test]
fn c_the_de_minimis_election_is_one_year_under_50000() {
    let record = change(2026, dcn(233), Adjustment481::OneYear { amount: 4_999_999 });
    let d = with_business(60_000, vec![record.clone()], |_| {});
    let id = d.income[1].id;
    let r = supported(&d);
    assert_eq!(cents(&r, &format!("business_{id}_other_income")), 4_999_999);
    assert_eq!(cents(&r, &format!("business_{id}_net_profit")), 10_999_999);
    assert!(cents_of(&r, &format!("business_{id}_method_change_0_remaining")).is_none());
    assert_eq!(
        printed(&r, &[])
            .iter()
            .find(|row| (row.0, row.1.as_str()) == (TaxForm::Schedule1, "3"))
            .unwrap()
            .2,
        11_000_000
    );
    // $50,000 is rejected: not "less than $50,000".
    assert_eq!(
        change(2026, dcn(233), Adjustment481::OneYear { amount: 5_000_000 }).rule(),
        Some("METHOD_CHANGE_DE_MINIMIS_LIMIT")
    );
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = business(6_000_000);
    page.more.method_changes =
        vec![change(2026, dcn(233), Adjustment481::OneYear { amount: 5_000_000 })];
    rejected_with(
        save_businesses(&mut session, vec![page.clone()]),
        "METHOD_CHANGE_DE_MINIMIS_LIMIT",
    );
    page.more.method_changes = vec![record];
    accepted(&save_businesses(&mut session, vec![page.clone()]));
    // Without the election the same amount is spread over four years.
    //   through(2026) = (4,999,999 + 2) / 4 = 1,250,000; remaining 3,749,999
    let d = with_business(60_000, vec![change(2026, dcn(233), four(4_999_999, false))], |_| {});
    let r = supported(&d);
    let id = d.income[1].id;
    assert_eq!(cents(&r, &format!("business_{id}_method_change_0")), 1_250_000);
    assert_eq!(cents(&r, &format!("business_{id}_method_change_0_remaining")), 3_749_999);
    assert_eq!(cents(&r, &format!("business_{id}_net_profit")), 7_250_000);
    assert_eq!(
        printed(&r, &[])
            .iter()
            .find(|row| (row.0, row.1.as_str()) == (TaxForm::Schedule1, "3"))
            .unwrap()
            .2,
        7_250_000
    );
    // The election applies to its year of change only.
    assert_eq!(
        change(2025, dcn(233), Adjustment481::OneYear { amount: 4_999_999 }).rule(),
        Some("METHOD_CHANGE_NOTHING_IN_CURRENT_YEAR")
    );
}

// Fixture D. Line 1 $30,000.00, no expenses; year 2024, DCN 7, four-year
// +$40,000.00.
//   through(2025) = 2,000,000
//   rest_in_current_year No:  part = 3,000,000 - 2,000,000 = 1,000,000; remaining 1,000,000; line 31 = 4,000,000
//   rest_in_current_year Yes: part = 4,000,000 - 2,000,000 = 2,000,000; remaining 0;         line 31 = 5,000,000
#[test]
fn d_the_rest_is_taken_in_2026_when_the_business_stops() {
    for (rest, part, remaining) in [(false, 1_000_000, 1_000_000), (true, 2_000_000, 0)] {
        let record = change(2024, dcn(7), four(4_000_000, rest));
        let d = with_business(30_000, vec![record], |_| {});
        let id = d.income[1].id;
        let r = supported(&d);
        assert_eq!(cents(&r, &format!("business_{id}_method_change_0")), part);
        assert_eq!(
            cents_of(&r, &format!("business_{id}_method_change_0_remaining")).unwrap_or(0),
            remaining
        );
        assert_eq!(cents(&r, &format!("business_{id}_net_profit")), 3_000_000 + part);
        let mut plain = with_business(30_000, vec![], |b| {
            *receipts_mut(b) = 3_000_000 + part;
        });
        plain.income[1].id = id;
        same_return(&r, &supported(&plain), SCHEDULE_C_INCOME);
    }
    // The owner's death in 2026, from the existing death date, is a stop:
    // the rest is taken without the question.
    let mut dead =
        with_business(30_000, vec![change(2024, dcn(7), four(4_000_000, false))], |_| {});
    dead.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, date: 20260615, ordinary: true, estate_medical: 0 });
    let id = dead.income[1].id;
    let r = supported(&dead);
    assert_eq!(cents(&r, &format!("business_{id}_method_change_0")), 2_000_000);
    assert!(cents_of(&r, &format!("business_{id}_method_change_0_remaining")).is_none());
    // The accepted answer is not changed by the stop.
    assert_eq!(business_mut(&mut dead, 1).method_changes[0].adjustment, four(4_000_000, false));
    // A decrease has no rest and an earlier year of change cannot be entered.
    assert_eq!(
        change(2022, dcn(7), four(4_000_000, true)).rule(),
        Some("METHOD_CHANGE_NOTHING_IN_CURRENT_YEAR")
    );
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = business(3_000_000);
    page.more.method_changes = vec![change(2022, dcn(7), four(4_000_000, true))];
    rejected_with(
        save_businesses(&mut session, vec![page]),
        "METHOD_CHANGE_NOTHING_IN_CURRENT_YEAR",
    );
}

// Fixture D (i): the same record on a cash farm moves Schedule F gross income
// and net profit by exactly the 2026 part.
#[test]
fn d_a_farm_takes_the_part_in_schedule_f_gross_income() {
    for (rest, part) in [(false, 1_000_000), (true, 2_000_000)] {
        let mut base = empty_return(FederalStatus::Single);
        add(&mut base, income(farm_business(vec![FarmReceipt::Raised { amount: 3_000_000 }], 0)));
        let id = base.income[0].id;
        let plain = supported(&base);
        let mut d = base.clone();
        business_mut(&mut d, 0).method_changes = vec![change(2024, dcn(7), four(4_000_000, rest))];
        business_mut(&mut d, 0).method_changes[0].id = d.next_id;
        d.next_id += 1;
        let r = supported(&d);
        let prefix = format!("farm_{id}");
        assert_eq!(
            cents(&r, &format!("{prefix}_gross_income")),
            cents(&plain, &format!("{prefix}_gross_income")) + part
        );
        assert_eq!(
            cents(&r, &format!("{prefix}_net_profit")),
            cents(&plain, &format!("{prefix}_net_profit")) + part
        );
        assert_eq!(cents(&r, &format!("{prefix}_method_change_0")), part);
        assert_eq!(notes(&r, "NOTE_METHOD_CHANGE_FORM_3115").len(), 1);
        // The farm decrease is an expense and never a gross-income change.
        let mut down = base.clone();
        business_mut(&mut down, 0).method_changes =
            vec![change(2026, dcn(7), Adjustment481::Decrease { amount: 700_000 })];
        business_mut(&mut down, 0).method_changes[0].id = down.next_id;
        down.next_id += 1;
        let r = supported(&down);
        assert_eq!(
            cents(&r, &format!("{prefix}_gross_income")),
            cents(&plain, &format!("{prefix}_gross_income"))
        );
        assert_eq!(
            cents(&r, &format!("{prefix}_net_profit")),
            cents(&plain, &format!("{prefix}_net_profit")) - 700_000
        );
    }
}

// Fixture D (ii): a rental sold in 2026 takes the rest from the sale itself,
// without a second question.
#[test]
fn d_a_rental_sold_in_2026_takes_the_rest_of_an_increase() {
    let mut base = complete(FederalStatus::Single);
    let mut property = rental(24_000, 1_000);
    property.sale = crate::RentalSale::Sold {
        date: 20260630,
        price: 30_000_000,
        land_price: 5_000_000,
        selling_expenses: 0,
        payment: crate::SalePayment::InFull,
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
    add(&mut base, income(FederalIncome::Rental { data: property }));
    base.screening.as_mut().unwrap().section1231 =
        Some(crate::Section1231History { regular: vec![], amt: vec![] });
    let id = base.income[1].id;
    let plain = supported(&base);
    let mut d = base.clone();
    let FederalIncome::Rental { data } = &mut d.income[1].income else { panic!() };
    data.method_changes = vec![change(2024, dcn(7), four(4_000_000, false))];
    data.method_changes[0].id = d.next_id;
    d.next_id += 1;
    let r = supported(&d);
    let line3 = |r: &FederalResult| cents(r, &format!("rental_{id}_line3_rents"));
    assert_eq!(line3(&r), line3(&plain) + 2_000_000);
    assert_eq!(
        cents(&r, &format!("rental_{id}_line21_income")),
        cents(&plain, &format!("rental_{id}_line21_income")) + 2_000_000
    );
    assert!(cents_of(&r, &format!("rental_{id}_method_change_0_remaining")).is_none());
    // The unsold rental takes only its fourth.
    let FederalIncome::Rental { data } = &mut d.income[1].income else { panic!() };
    data.sale = crate::RentalSale::No;
    let unsold = supported(&d);
    let line3_unsold = cents(&unsold, &format!("rental_{id}_line3_rents"));
    assert_eq!(line3_unsold, 2_400_000 + 1_000_000);
    assert_eq!(cents(&unsold, &format!("rental_{id}_method_change_0_remaining")), 1_000_000);
}

// Fixture E. Business line 1 $40,000.00; year 2025, no DCN, two years
// (under examination) +$9,000.01.
//   through(2025) = (900,001 + 1) / 2 = 450,001; part = 450,000; remaining 0; line 31 = 4,450,000
#[test]
fn e_two_years_under_examination_and_the_scope_confirmation() {
    let record = change(
        2025,
        MethodChangeConsent::WithoutDcn,
        Adjustment481::TwoYears { amount: 900_001, rest_in_current_year: false },
    );
    let d = with_business(40_000, vec![record.clone()], |_| {});
    let id = d.income[1].id;
    let r = supported(&d);
    assert_eq!(cents(&r, &format!("business_{id}_method_change_0")), 450_000);
    assert!(cents_of(&r, &format!("business_{id}_method_change_0_remaining")).is_none());
    assert_eq!(cents(&r, &format!("business_{id}_net_profit")), 4_450_000);
    let found = notes(&r, "NOTE_METHOD_CHANGE_FORM_3115");
    assert_eq!(found.len(), 1);
    let crate::MessageArg::Nested(consent) = &found[0].display.args[2] else { panic!() };
    assert_eq!(consent.key, "METHOD_CHANGE_NO_DCN");
    let mut plain = with_business(40_000, vec![], |b| *receipts_mut(b) = 4_450_000);
    plain.income[1].id = id;
    same_return(&r, &supported(&plain), SCHEDULE_C_INCOME);
    // 2024 has nothing left in 2026.
    let mut early = record.clone();
    early.year = 2024;
    assert_eq!(early.rule(), Some("METHOD_CHANGE_NOTHING_IN_CURRENT_YEAR"));
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = business(4_000_000);
    page.more.method_changes = vec![early];
    rejected_with(
        save_businesses(&mut session, vec![page]),
        "METHOD_CHANGE_NOTHING_IN_CURRENT_YEAR",
    );
    // A distinct AMT source amount keeps the regular adjustment and normal lifecycle.
    let mut separate = record;
    separate.amt = crate::MethodChangeAmt::Different { amount: 800_000 };
    let mut d = with_business(40_000, vec![separate], |_| {});
    d.income[1].id = id;
    let out = supported(&d);
    assert!(notes(&out, "WARN_METHOD_CHANGE_LEFT_OUT").is_empty());
    assert_eq!(cents(&out, &format!("business_{id}_method_change_0")), 450_000);
    assert_eq!(cents(&out, &format!("business_{id}_net_profit")), 4_450_000);
    assert_eq!(
        crate::method_change::amt_difference(&business_mut(&mut d, 1).method_changes),
        -50_000
    );
    assert_eq!(restart(d.clone()).draft, d);
}

// A later record keeps its position after an earlier one is left out, so its
// note still opens that record.
#[test]
fn a_distinct_amt_record_does_not_move_the_notes_of_the_later_ones() {
    let mut first = change(2026, dcn(7), Adjustment481::Decrease { amount: 100_000 });
    first.amt = crate::MethodChangeAmt::Different { amount: 0 };
    let second = change(2026, dcn(7), Adjustment481::Decrease { amount: 200_000 });
    let d = with_business(40_000, vec![first, second], |_| {});
    let id = d.income[1].id;
    let r = supported(&d);
    assert_eq!(cents(&r, &format!("business_{id}_net_profit")), 3_700_000);
    assert_eq!(cents(&r, &format!("business_{id}_method_change_0")), -100_000);
    assert_eq!(cents(&r, &format!("business_{id}_method_change_1")), -200_000);
    let positions: Vec<usize> = r
        .review_notes
        .iter()
        .filter_map(|n| match (&n.context, n.display.key.as_str()) {
            (
                Some(crate::IssueContext::Scoped {
                    identity: crate::federal_model::NoteIdentity::Item { position, .. },
                    ..
                }),
                "NOTE_METHOD_CHANGE_FORM_3115" | "WARN_METHOD_CHANGE_LEFT_OUT",
            ) => Some(*position),
            _ => None,
        })
        .collect();
    assert_eq!(positions.len(), 2);
    assert!(positions.contains(&0) && positions.contains(&1));
}

// The farm and rental pages keep the records, reopen them and reject what the
// client rule rejects; the count limit is the same on all three pages.
#[test]
fn the_farm_and_rental_pages_store_reopen_and_reject_the_records() {
    // Farm.
    let mut session = Interview::new(TEST_USER, crate::tests::cross::interview::farm_only_return());
    let mut farm = cash_farm(3_000_000, 0);
    farm.more.method_changes = vec![change(2024, dcn(122), four(4_000_000, true))];
    accepted(&save_farm(&mut session, farm.clone()));
    let (stored, _) = stored_farm(&session.draft);
    farm.more.method_changes[0].id = stored.method_changes[0].id;
    assert!(farm.more.method_changes[0].id > 0);
    assert_eq!(stored.method_changes, farm.more.method_changes);
    let (_, reopened) =
        crate::tests::cross::interview::reopened_farming(&mut restart(session.draft.clone()));
    let [IncomeDocument::Farm(again)] = reopened.as_slice() else { panic!("{reopened:?}") };
    assert_eq!(again.more.method_changes, farm.more.method_changes);
    farm.more.method_changes =
        vec![change(2025, dcn(7), Adjustment481::Decrease { amount: 100_000 })];
    rejected_with(save_farm(&mut session, farm.clone()), "METHOD_CHANGE_NOTHING_IN_CURRENT_YEAR");
    // Rental.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = rental_page(2_400_000);
    page.more.method_changes =
        vec![change(2026, dcn(107), Adjustment481::Decrease { amount: 500_000 })];
    accepted(&save_rentals(&mut session, vec![page.clone()]));
    page.more.method_changes[0].id = stored_rentals(&session)[0].2.method_changes[0].id;
    assert!(page.more.method_changes[0].id > 0);
    assert_eq!(stored_rentals(&session)[0].2.method_changes, page.more.method_changes);
    crate::tests::cross::interview::assert_rentals_round_trip(&mut session, &[page.clone()]);
    page.more.method_changes =
        vec![change(2025, dcn(107), Adjustment481::Decrease { amount: 500_000 })];
    rejected_with(
        save_rentals(&mut session, vec![page.clone()]),
        "METHOD_CHANGE_NOTHING_IN_CURRENT_YEAR",
    );
    // The count limit (11 changes) on the three pages.
    let eleven: Vec<MethodChange> =
        (0..11).map(|_| change(2026, dcn(7), Adjustment481::Decrease { amount: 100 })).collect();
    page.more.method_changes = eleven.clone();
    rejected_with(
        save_rentals(&mut session, vec![page]),
        "MESSAGE_ENTER_AT_MOST_10_METHOD_CHANGES",
    );
    let mut business_page = business(1_000_000);
    business_page.more.method_changes = eleven.clone();
    rejected_with(
        save_businesses(&mut session, vec![business_page]),
        "MESSAGE_ENTER_AT_MOST_10_METHOD_CHANGES",
    );
    farm.more.method_changes = eleven;
    rejected_with(save_farm(&mut session, farm), "MESSAGE_ENTER_AT_MOST_10_METHOD_CHANGES");
}

// Stored records outside the limits are not calculated.
#[test]
fn stored_records_outside_the_limits_block_the_estimate() {
    let d = with_business(40_000, vec![change(2022, dcn(7), four(4_000_000, false))], |_| {});
    assert_eq!(estimate(&d).state, ResultState::Incomplete);
    let d = with_business(
        40_000,
        vec![change(2026, dcn(1000), Adjustment481::Decrease { amount: 1 })],
        |_| {},
    );
    assert_eq!(estimate(&d).state, ResultState::Incomplete);
}

// The remaining "no accounting-method change" confirmations have one owner
// (improvement row 139): each says that a change entered under
// Accounting-method changes is not excluded, and none keeps the unnarrowed
// words.
#[test]
fn every_no_method_change_confirmation_points_at_the_list() {
    // CCC now collects actual original election/change history instead of a
    // no-method-change confirmation; its source-history tests cover that path.
    let phrase = crate::method_change::ENTERED_UNDER_LIST;
    for (name, source, anchors) in [
        (
            "inventory",
            include_str!("../../federal/extended_model.rs"),
            &["an inventory method change is"][..],
        ),
        (
            "start-up costs",
            include_str!("../../cost_recovery/startup.rs"),
            &["or accounting-method change other than one"],
        ),
        (
            "prior depreciation",
            include_str!("../../cost_recovery/depreciation.rs"),
            &["as a section 481(a) adjustment"],
        ),
        (
            "farm supplies",
            include_str!("../../farm/farm_supplies.rs"),
            &[
                "An accounting-method change other than one",
                "Transfers, prior-year refunds and an accounting-method change other than one",
                "or accounting-method change other than one",
            ],
        ),
    ] {
        for anchor in anchors {
            let at = source.find(anchor).unwrap_or_else(|| panic!("{name}: no \"{anchor}\""));
            let sentence = &source[at..source[at..].find('\n').map_or(source.len(), |n| at + n)];
            assert!(sentence.contains(phrase), "{name}: {anchor}");
        }
    }
    for (name, source) in [
        ("farm.rs", include_str!("../../farm/farm.rs")),
        ("farm_page.rs", include_str!("../../farm/farm_page.rs")),
        ("extended_model.rs", include_str!("../../federal/extended_model.rs")),
        ("startup.rs", include_str!("../../cost_recovery/startup.rs")),
        ("depreciation.rs", include_str!("../../cost_recovery/depreciation.rs")),
        ("farm_supplies.rs", include_str!("../../farm/farm_supplies.rs")),
        ("farm_ccc.rs", include_str!("../../farm/farm_ccc.rs")),
    ] {
        for old in [
            "No accounting-method change,",
            "A change of accounting method (Form 3115)",
            "No method change, LIFO",
            "or accounting-method correction.",
            "accounting-method corrections need separate review",
            "changes of accounting method need separate timing worksheets",
            "An accounting-method change needs its own worksheet",
            "capitalization election or accounting-method change. Not old",
            "and no accounting-method change. Confirm",
        ] {
            assert!(!source.contains(old), "{name}: {old}");
        }
    }
    // The inventory notice no longer names a method change, which an entered
    // change no longer excludes.
    let warn = APP_STRINGS.iter().find(|(key, _)| *key == "WARN_INVENTORY_LEFT_OUT").unwrap().1;
    assert!(!warn.contains("method change"));
}

// Search wording and the string resources the validation rules name.
#[test]
fn search_wording_and_rule_resources_are_declared() {
    let protocol = include_str!("../../core/protocol.rs");
    for (section, marker) in [
        ("SelfEmployment", "Schedule C line 6 or Part V"),
        ("Rentals", "missed or excess rental depreciation (DCN 7 or 107)"),
        ("Farming", "Schedule F other income or other expenses"),
    ] {
        let at = protocol.find(marker).unwrap_or_else(|| panic!("{section}"));
        let line = &protocol[protocol[..at].rfind('\n').unwrap()..at];
        assert!(line.contains("Accounting-method changes (Form 3115)"), "{section}");
        assert!(protocol[at..].contains(&format!("    {section},")), "{section}");
    }
    let prompt = APP_STRINGS.iter().find(|(key, _)| *key == "TAX_NAVIGATION_PROMPT").unwrap().1;
    assert!(
        prompt.contains(
            "Form 3115, an accounting-method change, a section 481(a) or 481a adjustment"
        )
    );
    for key in [
        "METHOD_CHANGE_NOTHING_IN_CURRENT_YEAR",
        "METHOD_CHANGE_DE_MINIMIS_LIMIT",
        "METHOD_CHANGE_OUT_OF_LIMITS",
        "MESSAGE_ENTER_AT_MOST_10_METHOD_CHANGES",
        "METHOD_CHANGE_DCN",
        "METHOD_CHANGE_NO_DCN",
    ] {
        assert!(APP_STRINGS.iter().any(|(k, _)| *k == key), "{key}");
    }
}
