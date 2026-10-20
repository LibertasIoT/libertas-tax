// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;
use crate::tests::family::family::community_item_agi;
#[test]
fn self_employment_and_qbi_independent_fixture() {
    for (receipts, supplies, expected) in [
        (
            6_000_000,
            1_000_000,
            &[("self_employment_tax", 7065), ("agi", 46467), ("qbi_deduction", 6073)][..],
        ),
        (108_000, 0, &[("qbi_deduction", 400), ("taxable_income", 0)][..]),
    ] {
        let mut d = empty_return(FederalStatus::Single);
        let mut business = cash_business(0);
        let FederalIncome::Business { data } = &mut business else { unreachable!() };
        data.activity = BusinessActivity::Ordinary {
            returns: 0,
            receipts,
            accounting: BusinessAccounting::Cash,
        };
        if supplies != 0 {
            data.expenses.push(ordinary_business_expense(
                BusinessExpenseCategory::Other { label: "Supplies".into() },
                supplies,
            ));
        }
        add(&mut d, income(business));
        let r = supported(&d);
        for &(key, amount) in expected {
            assert_eq!(value(&r, key), amount);
        }
        if supplies != 0 {
            assert_eq!(r.tax, Some(973_000));
        }
    }
}

#[test]
fn monthly_schedule_c_loss_in_student_month_is_replaced_by_deemed_earnings() {
    let mut d = benefit_return(FederalStatus::Joint);
    clear_care_benefits(&mut d);
    d.dependents.pop();
    d.dependents[0].set_care_expenses(300_000);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    let mut business = income(cash_business(-100));
    business.owner = Owner::Spouse;
    add(&mut d, business);
    let mut months = care_months(Owner::Spouse, 5);
    // No positive earnings were received during the selected months. The
    // annual $100 loss remains in the actual earned-income source.
    months[0].business_net = 0;
    set_care_months(&mut d, months);

    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(
        value(&r, &alloc::format!("business_{}_net_profit", d.income.last().unwrap().id)),
        -100
    );
    assert_eq!(value(&r, "care_earned_spouse"), 1_150);
    assert_eq!(value(&r, "agi"), 99_900);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn aggregate_schedule_c_care_earnings_require_nonnegative_selected_amount() {
    let mut d = benefit_return(FederalStatus::Joint);
    clear_care_benefits(&mut d);
    d.dependents.pop();
    care_costs(&mut d, 300000);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    let mut b = income(cash_business(300));
    b.owner = Owner::Spouse;
    add(&mut d, b);
    let mut months = care_months(Owner::Spouse, 5);
    months[0].business_net = 30000;
    set_care_months(&mut d, months);
    let r = supported(&d);
    // The approved aggregate lower bound is1250, rather than the retired
    // per-month1300; no business/foreign confirmation gate survives.
    assert_eq!(value(&r, "care_earned_spouse"), 1250);
    assert_eq!(value(&r, "care_credit"), 438);
    assert!(noted(&r, "WARN_CARE_DEEMED_INCOME_MONTHLY"));
    let CareFilerSituation::Student { earned_in_months, .. } =
        &mut d.credits.as_mut().unwrap().care.filers[0].situation
    else {
        panic!()
    };
    *earned_in_months = Some(-1);
    assert!(!d.valid_stored());
}

#[test]
fn mixed_business_kiddie_earnings_cap_uses_profit_after_half_se() {
    let source = BusinessIncomeSource::Mixed { compensation: 600000 };
    assert_eq!(source.earned(10000, 707), Some(2788));
    assert_eq!(BusinessIncomeSource::Mixed { compensation: 100000 }.earned(10000, 707), Some(1000));
    assert_eq!(source.earned(0, 0), Some(6000));
    assert_eq!(source.earned(-1000, 0), Some(6000));
    assert_eq!(BusinessIncomeSource::Services.earned(10000, 707), Some(10000));
    assert_eq!(BusinessIncomeSource::Capital.earned(10000, 707), Some(0));
}

#[test]
fn independent_mixed_business_coordinates_parent_tax_and_statutory_amt_exemption() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    let mut source = cash_business(10000);
    let FederalIncome::Business { data } = &mut source else { panic!() };
    data.income_source = BusinessIncomeSource::Mixed { compensation: 600000 };
    add(&mut d, income(source));
    let r = supported(&d);
    assert_eq!(
        value(&r, &alloc::format!("business_{}_kiddie_earned_income", d.income[1].id)),
        2788
    );
    assert_eq!(value(&r, "agi"), 19293);
    // Standard deduction 16,100, then QBI taxable-income cap 639 => 2,554.
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 2554);
    assert_eq!(value(&r, "regular_income_tax"), 562);
    // AMTI 18,654 is below the ordinary 90,100 exemption; no child cap.
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn thirty_percent_business_reaches_the_form_8615_totals() {
    // Single 16-year-old, independent, a parent alive: $10,000 interest and a
    // $10,000 Schedule C profit whose owner's work is worth at least 30% of
    // the net profit. By hand:
    //   SE: 10,000 x 92.35% = 9,235 x 15.3% = 1,412.96 -> 1,413; half 707.
    //   AGI 10,000 + 10,000 - 707 = 19,293.
    //   Earned income (i8615): 30% x (10,000 - 707) = 2,787.90 -> 2,788.
    //   Taxable: 19,293 - 16,100 standard = 3,193; QBI deduction is the
    //   smaller of 20% x 9,293 = 1,859 and 20% x 3,193 = 639: 3,193 - 639
    //   = 2,554.
    //   Form 8615 line 1 worksheet: 20,000 - 2,788 = 17,212; line 2 2,700;
    //   line 3 14,512; line 4 2,554; line 5 (smaller) 2,554.
    //   Lines 6-11: parent joint taxable 150,000 and 152,554 both in the 22%
    //   bracket (100,800-211,400): 2,554 x 22% = 561.88 -> 562. Line 17 is
    //   10% x 2,554 = 255; line 18 the larger, 562.
    //   AMTI 18,654 is below the 90,100 exemption: no AMT.
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    let mut source = cash_business(10000);
    let FederalIncome::Business { data } = &mut source else { panic!() };
    data.income_source = BusinessIncomeSource::MixedThirtyPercent;
    add(&mut d, income(source));
    let r = supported(&d);
    assert_eq!(
        value(&r, &alloc::format!("business_{}_kiddie_earned_income", d.income[1].id)),
        2788
    );
    assert_eq!(value(&r, "agi"), 19293);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 2554);
    assert_eq!(value(&r, "regular_income_tax"), 562);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn independent_mixed_business_uses_actual_compensation_below_cap() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    let mut source = cash_business(10000);
    let FederalIncome::Business { data } = &mut source else { panic!() };
    data.income_source = BusinessIncomeSource::Mixed { compensation: 100000 };
    add(&mut d, income(source));
    add(&mut d, wage(0, Owner::Taxpayer, 5000000, 0));
    assert_eq!(value(&supported(&d), "kiddie_tax_net_unearned_income"), 16300);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.income_source = BusinessIncomeSource::Mixed { compensation: 600000 };
    assert_eq!(value(&supported(&d), "kiddie_tax_net_unearned_income"), 14512);
    d.income[0].income = savings_income(10000, 12000);
    assert_eq!(value(&supported(&d), "kiddie_tax_net_unearned_income"), 2512);
    add(&mut d, income(cash_business(1000)));
    assert_eq!(value(&supported(&d), "kiddie_tax_net_unearned_income"), 2512);
}

#[test]
fn business_long_term_care_enforces_every_2026_age_boundary() {
    for (age, cap) in [
        (0, 500),
        (40, 500),
        (41, 930),
        (50, 930),
        (51, 1860),
        (60, 1860),
        (61, 4960),
        (70, 4960),
        (71, 6200),
        (120, 6200),
    ] {
        let mut d = complete(FederalStatus::Single);
        set_age(&mut d, -1, age);
        add(&mut d, income(care_business(50000, 10000, false)));
        let r = supported(&d);
        assert_eq!(value(&r, "business_2_health_insurance"), cap);
        assert_eq!(value(&r, "business_2_long_term_care_deduction"), cap);
        assert_eq!(value(&r, "long_term_care_1_eligible"), cap);
        assert_eq!(value(&r, "long_term_care_1_medical"), 0);
        assert_eq!(value(&r, "long_term_care_1_excluded"), 10000 - cap);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn optional_se_eic_and_child_refund_use_different_earned_income_rules() {
    let mut d = empty_return(FederalStatus::Single);
    add_child(&mut d);
    add(&mut d, income(raised_farm(12000, 11000)));
    elect_se(&mut d, SeMethod::Farm);
    let r = supported(&d);
    // EIC uses $7,560 optional earnings less $579 half-SE: $6,981.
    // ACTC retains the smaller $1,000 actual farm profit less $579: $421.
    assert_eq!(value(&r, "earned_income_credit"), 2372);
    assert_eq!(value(&r, "additional_child_credit"), 0);
    assert_eq!(value(&r, "agi"), 421);
    assert_eq!(r.tax, Some(115700));
    assert_eq!(r.refund, Some(121500));
}

#[test]
fn services_business_note_appears_only_under_form_8615() {
    // A 16-year-old with a parent alive (Form 8615 applies) and a Schedule C
    // business left on More: Nothing (stored Services): the review names it.
    let note = "Business Lawn care is treated as income from your own work for Form 8615. If equipment, inventory or other capital earns a material part of its income, open More \u{2192} Something else and change it.";
    let has_note = |d: &FederalDraft| estimate(d).review_notes.iter().any(|n| n == note);
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    add(&mut d, FederalIncomeEntry { label: "Lawn care".into(), ..income(cash_business(3000)) });
    assert!(has_note(&d));
    let result = estimate(&d);
    let message = &result.review_notes.iter().find(|item| item.english == note).unwrap().display;
    assert_eq!(message.key, "NOTE_YOUNG_FILER_SERVICE_BUSINESS");
    assert_eq!(message.args, vec![MessageArg::Text("Lawn care".into())]);
    // Another answer (capital) is the filer's own choice: no note.
    let mut capital = d.clone();
    let FederalIncome::Business { data } = &mut capital.income[1].income else { panic!() };
    data.income_source = BusinessIncomeSource::Capital;
    assert!(!has_note(&capital));
    // No parent alive at the end of 2026: Form 8615 does not apply.
    let mut orphan = d.clone();
    orphan.people.as_mut().unwrap().taxpayer.parent_alive = false;
    assert!(!has_note(&orphan));
    // Boundary: 24 at the end of 2026 and not a student fails the age test.
    let mut adult = d.clone();
    adult.people.as_mut().unwrap().taxpayer.birth_date = 20020615;
    assert!(!has_note(&adult));
}

#[test]
fn a_business_divides_its_income_but_the_operator_keeps_the_self_employment_tax() {
    // Pub. 555 and section 1402(a)(5): the filer's $40,000 Schedule C is half
    // community income, yet its self-employment tax and the half deduction are
    // figured on all of it; a spouse's business adds half its profit and no
    // self-employment tax to this return.
    let plain = {
        let mut d = complete(FederalStatus::Separate);
        d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
        d.income = vec![wage(1, Owner::Taxpayer, 3_000_000, 300_000)];
        d.next_id = 2;
        add(&mut d, income(cash_business(40_000)));
        supported(&d)
    };
    let own = community_item_agi(
        income(cash_business(40_000)),
        Owner::Taxpayer,
        CommunityShare::Community,
    );
    let own = {
        assert_eq!(own.state, ResultState::EstimatedForSupportedScope, "{own:?}");
        own
    };
    assert_eq!(value(&own, "self_employment_tax"), value(&plain, "self_employment_tax"));
    let half_se = value(&plain, "self_employment_tax") / 2;
    assert!(half_se > 0);
    assert_eq!(value(&own, "agi"), 15_000 + 20_000 - half_se);
    let theirs =
        community_item_agi(income(cash_business(40_000)), Owner::Spouse, CommunityShare::Community);
    assert_eq!(value(&theirs, "self_employment_tax"), 0);
    assert_eq!(value(&theirs, "agi"), 15_000 + 20_000);
    // The qualified business income deduction takes only this return's share.
    assert!(value(&theirs, "qbi_deduction") <= 4_000);
    let kept = community_item_agi(
        income(cash_business(40_000)),
        Owner::Taxpayer,
        CommunityShare::Separate,
    );
    assert_eq!(value(&kept, "self_employment_tax"), value(&plain, "self_employment_tax"));
    assert_eq!(value(&kept, "agi"), 15_000 + 40_000 - half_se);
    let not_theirs =
        community_item_agi(income(cash_business(40_000)), Owner::Spouse, CommunityShare::Separate);
    assert_eq!(value(&not_theirs, "agi"), 15_000);
    assert_eq!(value(&not_theirs, "qbi_deduction"), 0);
}
