use super::*;
fn amount(r: &FederalResult, id: &str) -> i64 {
    r.lines.iter().find(|v| v.id == id).map_or(0, |v| v.amount / 100)
}
fn w2(d: &mut FederalDraft) -> &mut FederalWage {
    let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
    data
}
fn draft(status: FederalStatus, pay: i64, withheld: i64) -> FederalDraft {
    let mut d = empty_return(status);
    add(&mut d, wage(1, Owner::Taxpayer, pay * 100, withheld * 100));
    d
}
fn child_return(pay: i64) -> FederalDraft {
    let mut d = draft(FederalStatus::Head, pay, 0);
    add_child(&mut d);
    d
}
fn code(d: &mut FederalDraft, code: W2Code, amount: i64) {
    w2(d).other_box12.push(W2CodeAmount { code, amount: amount * 100 });
}
fn railroad(
    d: &mut FederalDraft,
    compensation: i64,
    tier_1: i64,
    tier_2: i64,
    medicare: i64,
    additional: i64,
) {
    let w = w2(d);
    w.social_wages = 0;
    w.social_tax = 0;
    w.medicare_wages = 0;
    w.medicare_tax = 0;
    w.payroll = W2Payroll::Railroad {
        compensation: compensation * 100,
        tier_1: tier_1 * 100,
        tier_2: tier_2 * 100,
        medicare: medicare * 100,
        additional_medicare: additional * 100,
    };
}
#[test]
fn wage_specials_a_allocated_tips_without_tp_and_noncash() {
    let mut d = draft(FederalStatus::Single, 26_000, 1500);
    w2(&mut d).employee_tips = Some(EmployeeTips {
        cash: CashTipRecords::Allocated { amount: 300_000 },
        noncash: 0,
        coverage: TipCoverage::SocialAndMedicare,
    });
    w2(&mut d).work_deductions.tips_claim =
        TipsClaim::Yes { w2_tips: 0, form4137_tips: 300_000, tips_eligible: true };
    let r = supported(&d);
    for (id, v) in [
        ("agi", 29_000),
        ("tips_deduction", 3000),
        ("taxable_income", 9900),
        ("income_tax", 993),
        ("total_tax", 1223),
    ] {
        assert_eq!(amount(&r, id), v, "{id}");
    }
    assert_eq!(r.refund, Some(27_700));
    w2(&mut d).employee_tips.as_mut().unwrap().noncash = 20_000;
    let r = supported(&d);
    assert_eq!(amount(&r, "agi"), 29_200);
    assert_eq!(amount(&r, "income_tax"), 1013);
    assert_eq!(amount(&r, "tips_deduction"), 3000);
}
#[test]
fn wage_specials_b_combat_election_and_decline() {
    let mut d = child_return(9000);
    let w = w2(&mut d);
    w.social_wages = 2_400_000;
    w.social_tax = 148_800;
    w.medicare_wages = 2_400_000;
    w.medicare_tax = 34_800;
    code(&mut d, W2Code::Q, 15_000);
    let r = supported(&d);
    assert_eq!(amount(&r, "agi"), 9000);
    assert_eq!(amount(&r, "earned_income_credit"), 4405);
    assert_eq!(amount(&r, "additional_child_credit"), 1700);
    assert_eq!(amount(&r, "combat_pay_election"), 15_000);
    d.credits.as_mut().unwrap().decline_earned_income_credit = true;
    let r = supported(&d);
    assert_eq!(amount(&r, "earned_income_credit"), 0);
    assert_eq!(amount(&r, "additional_child_credit"), 1700);
    assert_eq!(amount(&r, "combat_pay_election"), 0);
    let mut d = child_return(22_000);
    code(&mut d, W2Code::Q, 15_000);
    let r = supported(&d);
    assert_eq!(amount(&r, "earned_income_credit"), 4427);
    assert_eq!(amount(&r, "combat_pay_election"), 0);
    assert_eq!(amount(&r, "additional_child_credit"), 1700);
}
#[test]
fn wage_specials_c_statutory_pay_with_expenses_and_without() {
    let mut d = draft(FederalStatus::Single, 50_000, 0);
    w2(&mut d).statutory_employee = true;
    let r = supported(&d);
    assert_eq!(amount(&r, "agi"), 50_000);
    assert_eq!(amount(&r, "qbi_deduction"), 6780);
    assert_eq!(amount(&r, "income_tax"), 3007);
    assert_eq!(amount(&r, "self_employment_tax"), 0);
    assert_eq!(return_line(&r, "1a"), 0);
    add(&mut d, income(cash_business(0)));
    let id = d.income[1].id;
    w2(&mut d).statutory_business = Some(id);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.expenses = vec![
        ordinary_business_expense(BusinessExpenseCategory::Office, 150_000),
        ordinary_business_expense(BusinessExpenseCategory::Supplies, 50_000),
        ordinary_business_expense(BusinessExpenseCategory::Travel, 280_000),
        ordinary_business_expense(BusinessExpenseCategory::Utilities, 120_000),
    ];
    let r = supported(&d);
    assert_eq!(amount(&r, "agi"), 44_000);
    assert_eq!(amount(&r, "qbi_deduction"), 5580);
    assert_eq!(amount(&r, "income_tax"), 2431);
    assert_eq!(amount(&r, "self_employment_tax"), 0);
    assert_eq!(amount(&r, &alloc::format!("business_{id}_gross_receipts")), 50_000);
}
#[test]
fn wage_specials_c_statutory_earned_income_is_gross() {
    let mut d = child_return(10_000);
    w2(&mut d).statutory_employee = true;
    add(&mut d, income(cash_business(0)));
    let id = d.income[1].id;
    w2(&mut d).statutory_business = Some(id);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.expenses = vec![ordinary_business_expense(BusinessExpenseCategory::Office, 400_000)];
    let r = supported(&d);
    assert_eq!(amount(&r, "agi"), 6000);
    assert_eq!(amount(&r, "earned_income_credit"), 3409);
    assert_eq!(amount(&r, "self_employment_tax"), 0);
    if let FederalIncome::Business { data } = &mut d.income[1].income
        && let BusinessActivity::Ordinary { receipts, .. } = &mut data.activity
    {
        *receipts = 300_000;
    }
    let r = estimate(&d);
    assert!(r.issues.iter().any(|v| v.display.key == "ISSUE_STATUTORY_BUSINESS_HAS_OTHER_INCOME"));
}
#[test]
fn wage_specials_d_rrta_separate_threshold_and_actual_withholding() {
    let mut d = draft(FederalStatus::Joint, 260_000, 45_000);
    railroad(&mut d, 260_000, 11_439, 6700, 3770, 540);
    add(&mut d, wage(2, Owner::Spouse, 9_000_000, 1_200_000));
    let r = supported(&d);
    for (id, v) in [
        ("agi", 350_000),
        ("income_tax", 61_468),
        ("additional_medicare_payroll_wages", 0),
        ("additional_medicare_rrta_tax", 90),
        ("total_tax", 61_558),
        ("payments", 57_540),
    ] {
        assert_eq!(amount(&r, id), v, "{id}");
    }
    assert_eq!(r.balance, Some(401_800));
    assert_eq!(return_line(&r, "25c"), 54_000);
}
#[test]
fn wage_specials_d_rrta_tip_income_and_single_employer_limit() {
    let mut d = draft(FederalStatus::Single, 260_000, 0);
    railroad(&mut d, 260_000, 11_439, 0, 3770, 540);
    w2(&mut d).employee_tips = Some(EmployeeTips {
        cash: CashTipRecords::Allocated { amount: 200_000 },
        noncash: 0,
        coverage: TipCoverage::Railroad,
    });
    let r = supported(&d);
    assert_eq!(return_line(&r, "1c"), 200_000);
    assert_eq!(amount(&r, "employee_tips_0_social_tax"), 0);
    assert!(noted(&r, "NOTE_RAILROAD_TIPS_REPORT_TO_EMPLOYER"));
    if let W2Payroll::Railroad { tier_1, .. } = &mut w2(&mut d).payroll {
        *tier_1 = 1_200_000;
    }
    let r = estimate(&d);
    assert!(r.issues.iter().any(|v| v.display.key
        == "ISSUE_EXCESS_SOCIAL_SECURITY_WITHHOLDING_BY_ONE_EMPLOYER_REQUIRES_EMPLOYE_35C66F28"));
}
#[test]
fn wage_specials_e_medicaid_credit_election_and_box1_exclusion() {
    let mut d = child_return(0);
    let w = w2(&mut d);
    w.social_wages = 1_800_000;
    w.social_tax = 111_600;
    w.medicare_wages = 1_800_000;
    w.medicare_tax = 26_100;
    code(&mut d, W2Code::II, 18_000);
    let r = supported(&d);
    assert_eq!(amount(&r, "agi"), 0);
    assert_eq!(return_line(&r, "1d"), 1_800_000);
    assert_eq!(amount(&r, "medicaid_waiver_excluded"), -18_000);
    assert_eq!(amount(&r, "earned_income_credit"), 4427);
    assert_eq!(amount(&r, "additional_child_credit"), 1700);
    assert_eq!(r.refund, Some(612_700));
    add(&mut d, wage(3, Owner::Taxpayer, 2_000_000, 0));
    let r = supported(&d);
    assert_eq!(amount(&r, "agi"), 20_000);
    assert_eq!(amount(&r, "earned_income_credit"), 4427);
    assert_eq!(amount(&r, "additional_child_credit"), 1700);
    let mut d = child_return(18_000);
    w2(&mut d).medicaid_waiver = 1_800_000;
    let r = supported(&d);
    assert_eq!(amount(&r, "agi"), 0);
    assert_eq!(return_line(&r, "1a"), 1_800_000);
    assert_eq!(return_line(&r, "1d"), 0);
    assert_eq!(amount(&r, "earned_income_credit"), 4427);
}
#[test]
fn wage_specials_f_line1_kinds_ta_and_spouse_boxes() {
    let mut d = draft(FederalStatus::Single, 20_000, 0);
    code(&mut d, W2Code::TA, 1000);
    for (kind, pay) in
        [(OtherIncomeKind::HouseholdWages, 2400), (OtherIncomeKind::StrikeBenefits, 3000)]
    {
        add(
            &mut d,
            income(FederalIncome::OtherIncome {
                data: OtherIncomeItem { kind, amount: pay * 100, withholding: 0 },
            }),
        );
    }
    let r = supported(&d);
    assert_eq!(amount(&r, "agi"), 25_400);
    assert_eq!(return_line(&r, "1b"), 240_000);
    assert_eq!(return_line(&r, "1h"), 300_000);
    assert_eq!(amount(&r, "income_tax"), 933);
    let mut d = draft(FederalStatus::Separate, 60_000, 0);
    d.setup.as_mut().unwrap().separate_spouse = SeparateSpouseBoxes::Born1961OrEarlier;
    let r = supported(&d);
    assert_eq!(amount(&r, "deduction"), 17_750);
    assert_eq!(amount(&r, "income_tax"), 4825);
}

#[test]
fn wage_specials_a_actual_months_occupation_and_reported_tips() {
    let mut d = draft(FederalStatus::Single, 26_000, 1500);
    w2(&mut d).employee_tips = Some(EmployeeTips {
        cash: CashTipRecords::Actual {
            months: vec![
                CashTipMonth { month: 6, received: 150_000, reported: 0 },
                CashTipMonth { month: 7, received: 1500, reported: 0 },
            ],
        },
        noncash: 0,
        coverage: TipCoverage::SocialAndMedicare,
    });
    w2(&mut d).work_deductions.tips_claim =
        TipsClaim::Yes { w2_tips: 0, form4137_tips: 151_500, tips_eligible: true };
    let r = supported(&d);
    assert_eq!(return_line(&r, "1c"), 151_500);
    assert_eq!(amount(&r, "employee_tips_0_social_tax"), 93);
    assert_eq!(amount(&r, "employee_tips_0_medicare_tax"), 22);
    assert_eq!(amount(&r, "tips_deduction"), 1515);
    if let TipsClaim::Yes { tips_eligible, .. } = &mut w2(&mut d).work_deductions.tips_claim {
        *tips_eligible = false;
    }
    assert_eq!(amount(&supported(&d), "tips_deduction"), 0);
    w2(&mut d).social_tips = 600_000;
    w2(&mut d).social_tax = 198_400;
    w2(&mut d).medicare_wages = 3_200_000;
    w2(&mut d).medicare_tax = 46_400;
    w2(&mut d).employee_tips = Some(EmployeeTips {
        cash: CashTipRecords::Allocated { amount: 300_000 },
        noncash: 0,
        coverage: TipCoverage::SocialAndMedicare,
    });
    w2(&mut d).work_deductions.tips_claim =
        TipsClaim::Yes { w2_tips: 600_000, form4137_tips: 900_000, tips_eligible: true };
    assert_eq!(amount(&supported(&d), "tips_deduction"), 9000);
}

#[test]
fn wage_specials_c_other_owner_and_unsupported_business() {
    let mut d = draft(FederalStatus::Joint, 50_000, 5000);
    w2(&mut d).statutory_employee = true;
    let mut entry = income(cash_business(0));
    entry.owner = Owner::Spouse;
    add(&mut d, entry);
    let id = d.income[1].id;
    w2(&mut d).statutory_business = Some(id);
    assert!(
        estimate(&d).issues.iter().any(|v| v.display.key == "ISSUE_STATUTORY_BUSINESS_OTHER_OWNER")
    );
    d.income[1].owner = Owner::Taxpayer;
    if let FederalIncome::Business { data } = &mut d.income[1].income {
        data.material_participation = false;
    }
    let r = supported(&d);
    assert_eq!(amount(&r, "agi"), 0);
    assert_eq!(amount(&r, "payments"), 5000);
    assert!(noted(&r, "WARN_STATUTORY_PAY_LEFT_OUT"));
}

#[test]
fn wage_specials_d_two_railroads_excess_tier1_and_se_base() {
    let mut d = draft(FederalStatus::Single, 120_000, 0);
    railroad(&mut d, 120_000, 7440, 1000, 1740, 0);
    let mut other = draft(FederalStatus::Single, 100_000, 0);
    railroad(&mut other, 100_000, 6200, 1000, 1450, 0);
    add(&mut d, other.income.remove(0));
    add(&mut d, income(cash_business(10_000)));
    let r = supported(&d);
    assert_eq!(amount(&r, "schedule_3_excess_social_security"), 2201);
    assert_eq!(amount(&r, "self_employment_tax"), 268);
    assert!(noted(&r, "NOTE_RAILROAD_TIER_2_FORM_843"));
}

#[test]
fn wage_specials_e_without_w2_home_fact_controls_taxability() {
    for (home, agi) in [(true, 0), (false, 18_000)] {
        let mut d = child_return(0);
        add(
            &mut d,
            income(FederalIncome::OtherIncome {
                data: OtherIncomeItem {
                    kind: OtherIncomeKind::MedicaidWaiver { in_your_home: home },
                    amount: 1_800_000,
                    withholding: 0,
                },
            }),
        );
        let r = supported(&d);
        assert_eq!(amount(&r, "agi"), agi);
        assert_eq!(return_line(&r, "1d"), 1_800_000);
        assert_eq!(amount(&r, "earned_income_credit"), 4427);
        assert_eq!(amount(&r, "medicaid_waiver_excluded"), if home { -18_000 } else { 0 });
    }
}

#[test]
fn wage_specials_e_ira_difficulty_of_care_is_nondeductible_only() {
    let mut d = draft(FederalStatus::Single, 0, 0);
    code(&mut d, W2Code::II, 18_000);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750_000, 0, Answer::No);
    let r = supported(&d);
    assert_eq!(amount(&r, "ira_deduction"), 0);
    assert_eq!(amount(&r, "taxpayer_nondeductible_ira"), 7500);
    assert_eq!(amount(&r, "ira_excess_taxpayer_line17_traditional_excess_tax"), 0);
    let mut combat = draft(FederalStatus::Single, 0, 0);
    code(&mut combat, W2Code::Q, 18_000);
    combat.adjustments.as_mut().unwrap().iras = ira_facts(750_000, 0, Answer::No);
    assert_eq!(amount(&supported(&combat), "ira_deduction"), 7500);
}

#[test]
fn wage_specials_f_household_threshold_and_both_spouse_boxes() {
    let mut d = draft(FederalStatus::Single, 20_000, 0);
    add(
        &mut d,
        income(FederalIncome::OtherIncome {
            data: OtherIncomeItem {
                kind: OtherIncomeKind::HouseholdWages,
                amount: 320_000,
                withholding: 0,
            },
        }),
    );
    assert!(noted(&supported(&d), "NOTE_HOUSEHOLD_PAY_W2_DUE"));
    let mut d = draft(FederalStatus::Separate, 60_000, 0);
    d.setup.as_mut().unwrap().separate_spouse = SeparateSpouseBoxes::BornAndBlind;
    assert_eq!(amount(&supported(&d), "deduction"), 19_400);
    d.deductions.as_mut().unwrap().spouse_itemizes = true;
    assert!(
        estimate(&d)
            .issues
            .iter()
            .any(|v| v.display.key == "ISSUE_SEPARATE_SPOUSE_BOXES_WITH_SPOUSE_ITEMIZING")
    );
}

#[test]
fn wage_specials_b_each_spouses_combat_election_and_exact_tie() {
    let mut d = draft(FederalStatus::Joint, 9000, 0);
    add_child(&mut d);
    code(&mut d, W2Code::Q, 15_000);
    let mut spouse = wage(2, Owner::Spouse, 0, 0);
    if let FederalIncome::Wage { data } = &mut spouse.income {
        data.other_box12.push(W2CodeAmount { code: W2Code::Q, amount: 2_000_000 });
    }
    add(&mut d, spouse);
    // Either owner's pay alone reaches the joint maximum; both reaches phase-out.
    // Exclude-all gives $3,069. The first maximum candidate elects $15,000.
    let r = supported(&d);
    assert_eq!(amount(&r, "earned_income_credit"), 4427);
    assert_eq!(amount(&r, "additional_child_credit"), 1700);
    assert_eq!(return_line(&r, "1i"), 1_500_000);
    let mut d = draft(FederalStatus::Single, 100_000, 0);
    code(&mut d, W2Code::Q, 15_000);
    assert_eq!(amount(&supported(&d), "combat_pay_election"), 0);
}

#[test]
fn wage_specials_d_rrta_counts_for_three_child_payroll_alternative() {
    let mut d = draft(FederalStatus::Head, 260_000, 0);
    railroad(&mut d, 260_000, 11_439, 6700, 3770, 540);
    for _ in 0..3 {
        add_child(&mut d);
    }
    add(&mut d, income(cash_business(-255_000)));
    let r = supported(&d);
    let payroll = r.lines.iter().find(|v| {
        v.form_line.as_ref().is_some_and(|v| v.form == TaxForm::Schedule8812 && v.line == "21")
    });
    // Tier 1 + ordinary RRTA Medicare + the $540 actual tax, with no subtraction
    // of the separately reported $540 RRTA withholding: 11,439 + 3,770 + 540.
    assert_eq!(payroll.map(|v| v.amount / 100), Some(15_749));
}

#[test]
fn wage_specials_8959_ordinary_bn_sources_and_rrta_exclusion() {
    for life in [false, true] {
        let r = supported(&uncollected_wage(210_000, 0, 145, life));
        assert_eq!(amount(&r, "additional_medicare_ordinary_withheld"), 3135);
        assert_eq!(amount(&r, "additional_medicare_regular_withholding"), 3045);
        assert_eq!(amount(&r, "additional_medicare_ordinary_additional_withheld"), 90);
        assert_eq!(return_line(&r, "25c"), 9000);
        assert_eq!(return_line(&r, "23"), 23500);
    }
    let mut d = draft(FederalStatus::Joint, 260_000, 45_000);
    railroad(&mut d, 260_000, 11_439, 6700, 3625, 540);
    w2(&mut d).uncollected_tax = Some(w2_uncollected(0, 145, false));
    add(&mut d, wage(2, Owner::Spouse, 9_000_000, 1_200_000));
    let r = supported(&d);
    // Line 19 is only the spouse's ordinary $1,305 box 6: railroad B must not
    // become an invented $145 Additional Medicare withholding payment.
    assert_eq!(amount(&r, "additional_medicare_ordinary_withheld"), 1305);
    assert_eq!(amount(&r, "additional_medicare_ordinary_additional_withheld"), 0);
    assert_eq!(amount(&r, "additional_medicare_total_withheld"), 540);
    assert_eq!(return_line(&r, "25c"), 54_000);
    assert_eq!(amount(&r, "w2_uncollected_payroll_tax"), 145);
}

#[test]
fn wage_specials_statutory_unsupported_business_omits_box1_partitions_once() {
    let mut d = draft(FederalStatus::Single, 50_000, 5000);
    w2(&mut d).statutory_employee = true;
    w2(&mut d).medicaid_waiver = 1_000_000;
    add(&mut d, income(cash_business(0)));
    let id = d.income[1].id;
    w2(&mut d).statutory_business = Some(id);
    if let FederalIncome::Business { data } = &mut d.income[1].income {
        data.material_participation = false;
    }
    let original = d.clone();
    let r = supported(&d);
    assert_eq!(amount(&r, "agi"), 0);
    assert_eq!(amount(&r, "medicaid_waiver_excluded"), 0);
    assert_eq!(amount(&r, "payments"), 5000);
    assert!(noted(&r, "WARN_STATUTORY_PAY_LEFT_OUT"));
    assert_eq!(d, original, "omission changes only the supported working copy");
}

#[test]
fn wage_specials_statutory_section179_counts_linked_active_net_once() {
    let mut d = draft(FederalStatus::Single, 50_000, 0);
    w2(&mut d).statutory_employee = true;
    add(&mut d, income(cash_business(0)));
    let id = d.income[1].id;
    w2(&mut d).statutory_business = Some(id);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.expenses = vec![ordinary_business_expense(BusinessExpenseCategory::Office, 600_000)];
    data.assets = vec![elected_asset(100_000, 100_000, BonusElection::ElectOut)];
    let r = supported(&d);
    assert_eq!(amount(&r, "section179_business_income_limit"), 44_000);
    assert_eq!(amount(&r, "section179_deduction"), 44_000);
    assert_eq!(amount(&r, "section179_carryforward"), 56_000);
    assert_eq!(amount(&r, "self_employment_tax"), 0);
    assert_eq!(amount(&r, "agi"), 0);
}

#[test]
fn wage_specials_statutory_foreign_services_use_business_exclusion_once() {
    let mut d = draft(FederalStatus::Single, 50_000, 0);
    w2(&mut d).statutory_employee = true;
    add(&mut d, income(foreign_service_business(0)));
    let id = d.income[1].id;
    w2(&mut d).statutory_business = Some(id);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.expenses = vec![ordinary_business_expense(BusinessExpenseCategory::Office, 600_000)];
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(amount(&r, "agi"), 0);
    assert_eq!(amount(&r, "self_employment_tax"), 0);
    assert_eq!(amount(&r, "qbi_deduction"), 0);
    // The $50,000 gross exclusion has $6,000 related expense disallowance;
    // $44,000 net foreign exclusion offsets the $44,000 statutory Schedule C.
    assert_eq!(amount(&r, &alloc::format!("business_{id}_net_profit")), 44_000);
    let mut ira = d.clone();
    ira.adjustments.as_mut().unwrap().iras = ira_facts(750_000, 0, Answer::No);
    ira.adjustments.as_mut().unwrap().iras.taxpayer.excess = Some(IraExcess {
        traditional_value: Some(750_000),
        roth_value: 0,
        prior_traditional: 0,
        prior_roth: 0,
        returned_prior_excess: 0,
    });
    let ira_result = supported(&ira);
    assert_eq!(amount(&ira_result, "ira_deduction"), 0);
    assert_eq!(amount(&ira_result, "ira_excess_taxpayer_line17_traditional_excess_tax"), 450);
    let mut housing = d.clone();
    employee_housing(&mut housing, 40_000, ForeignHousingLocation::General);
    assert!(
        estimate(&housing)
            .issues
            .iter()
            .any(|i| i.display.key == "ISSUE_STATUTORY_FOREIGN_HOUSING")
    );
    w2(&mut d).abroad = 5_000_000;
    assert!(
        estimate(&d).issues.iter().any(|i| i.display.key == "W2_STATUTORY_ABROAD_USE_BUSINESS")
    );
}

#[test]
fn wage_specials_statutory_depreciation_keeps_amt_and_no_se() {
    let mut d = draft(FederalStatus::Single, 50_000, 0);
    w2(&mut d).statutory_employee = true;
    add(&mut d, income(cash_business(0)));
    let id = d.income[1].id;
    w2(&mut d).statutory_business = Some(id);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    // Pub946 five-year half-year 200% DB is 20%; AMT150% DB is15%.
    let mut asset = elected_asset(10_000, 0, BonusElection::ElectOut);
    asset.acquired = 20170927;
    data.assets = vec![asset];
    let r = supported(&d);
    assert_eq!(amount(&r, "agi"), 48_000);
    assert_eq!(amount(&r, "self_employment_tax"), 0);
    assert_eq!(amount(&r, &alloc::format!("business_{id}_asset_0_macrs")), 2000);
    assert_eq!(amount(&r, &alloc::format!("business_{id}_asset_0_amt_depreciation")), 1500);
}
