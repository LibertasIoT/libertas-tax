use super::*;
use crate::tests::credits::credits::{
    health_credit, health_credit_draft, health_employee, health_line, health_payroll, hours,
    leave_expense_line, leave_fixture_a, leave_line,
};

#[test]
fn specified_tip_fica_carryforward_offsets_amt_and_survives_restart() {
    let mut draft = complete(FederalStatus::Single);
    draft.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Synthetic ISO", 2000.0, 1000, 11000)];
    let baseline = supported(&draft);
    assert_eq!(value(&baseline, "alternative_minimum_tax"), 39151);
    assert_eq!(baseline.tax, Some(4_417_400));
    draft.credits.as_mut().unwrap().tip_fica_carryforward = Some(tip_fica_carryforward(5_000_000));
    let result = supported(&draft);
    // Form 8846 line 4f is specified: its Section C limit includes AMT.
    // An ordinary Part I credit would have no room below TMT here.
    assert_eq!(value(&result, "general_business_credit_limit"), 44174);
    assert_eq!(value(&result, "general_business_credit"), 44174);
    assert_eq!(value(&result, "tip_fica_credit_carryforward_to_next_year"), 5826);
    assert_eq!(return_line(&result, "20"), 4_417_400);
    assert_eq!(result.tax, Some(0));
    assert_eq!(supported(&restart(draft.clone()).draft), result);
    assert_eq!(restart(draft.clone()).draft, draft);
}
#[test]
fn tip_fica_2025_balance_uses_part_ii_quarter_excess_and_preserves_other_tax() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
    draft.credits.as_mut().unwrap().tip_fica_carryforward = Some(tip_fica_carryforward(10_000_000));
    let result = supported(&draft);
    let regular = value(&result, "regular_income_tax");
    let amt = value(&result, "alternative_minimum_tax");
    let floor = crate::kernel::ratio::fraction((regular - 25_000).max(0), 25, 100);
    assert_eq!(value(&result, "general_business_credit_limit"), regular + amt - floor);
    assert_eq!(value(&result, "general_business_credit"), regular + amt - floor);
    assert_eq!(
        value(&result, "tip_fica_credit_carryforward_to_next_year"),
        100_000 - regular - amt + floor
    );
    assert_eq!(value(&result, "tip_fica_origin_prior_year_carryback_used"), 5_000);
    assert_eq!(value(&result, "tip_fica_prior_year_credit_used"), 5_000);
    assert_eq!(value(&result, "tip_fica_prior_year_credit_origin"), 110_000);
    assert_eq!(supported(&restart(draft).draft), result);
}
#[test]
fn tip_fica_2025_source_and_competing_adjustments_are_guarded() {
    let mut draft = complete(FederalStatus::Single);
    let source = tip_fica_carryforward(200_000);
    draft.credits.as_mut().unwrap().tip_fica_carryforward = Some(source.clone());
    let mut invalid = source.clone();
    invalid.original_credit += 1;
    draft.credits.as_mut().unwrap().tip_fica_carryforward = Some(invalid);
    assert!(!draft.valid_stored());
    let mut invalid = source.clone();
    invalid.carryforward_to_current_year += 100;
    draft.credits.as_mut().unwrap().tip_fica_carryforward = Some(invalid);
    unsupported(&draft, "Employer tip FICA");
    let mut invalid = source.clone();
    invalid.ordinary = false;
    draft.credits.as_mut().unwrap().tip_fica_carryforward = Some(invalid);
    assert!(noted(&supported(&draft), "WARN_TIP_FICA_CARRYFORWARD_LEFT_OUT"));
    let mut invalid = source.clone();
    invalid.prior_status = FederalStatus::Joint;
    draft.credits.as_mut().unwrap().tip_fica_carryforward = Some(invalid);
    unsupported(&draft, "Employer tip FICA");
    draft.credits.as_mut().unwrap().tip_fica_carryforward = Some(source);
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(bba_interest_reduction()));
    let decrease = supported(&draft);
    // Tax5,023 minus the220 negative8978 credit reserves4,803 for3800.
    // The2,000 specified carry fits; final tax=5,023−220−2,000=2,803.
    assert_eq!(value(&decrease, "schedule_3_line_6l"), 220);
    assert_eq!(value(&decrease, "general_business_credit_limit"), 4803);
    assert_eq!(value(&decrease, "general_business_credit"), 2000);
    assert_eq!(decrease.tax, Some(280_300));
    assert_eq!(supported(&restart(draft.clone()).draft), decrease);
    // T3-3800: a married-separate return now uses section 38(c)(6)(A)'s
    // $12,500 line 13 amount. Tax on $43,900 (MFS column) is $5,023, below
    // $12,500, so line 13 is 0 and the $2,000 carryforward is used in full.
    let mut separate = complete(FederalStatus::Separate);
    let mut source = tip_fica_carryforward(200_000);
    source.prior_status = FederalStatus::Separate;
    separate.credits.as_mut().unwrap().tip_fica_carryforward = Some(source);
    let result = supported(&separate);
    assert_eq!(value(&result, "form_3800_line_13"), 0);
    assert_eq!(value(&result, "general_business_credit"), 2_000);
}

#[test]
fn disability_w2_credit_preserves_income_once_and_checks_source_facts() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.disabled = true;
    d.income[0] = wage(1, Owner::Taxpayer, 1700000, 0);
    let names = accepted_document_names(&d, Owner::Taxpayer);
    prepare_income_fixture(&mut d.income[0], Some(names));
    let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
    let mut data = edit_wage(data);
    data.disability = DisabilityIncome::Qualifying { amount: 1700000, eligible: true };
    drop(data);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 17000);
    assert_eq!(return_line(&r, "1a"), 1700000);
    assert_eq!(value(&r, "elderly_credit"), 38);
    assert_eq!(restart(d.clone()).draft, d);
    d.people.as_mut().unwrap().taxpayer.disabled = false;
    unsupported(&d, "Disability retirement");
    d.people.as_mut().unwrap().taxpayer.disabled = true;
    let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
    let mut data = edit_wage(data);
    data.disability = DisabilityIncome::Qualifying { amount: 1700001, eligible: true };
    drop(data);
    assert!(d.valid_stored());
    unsupported(&d, "Income");
    let mut session = restart(d);
    let request =
        crate::tests::support::client::income_pages::open_topic(&mut session, TaxSection::Wages);
    let before = session.draft.clone();
    assert!(matches!(send(&mut session, request), Reply::Response(P::Problem { .. })));
    assert_eq!(session.draft, before);
}

#[test]
fn uncollected_social_tax_participates_in_multiple_employer_credit() {
    let mut draft = uncollected_wage(180000, 620, 145, false);
    add(&mut draft, wage(2, Owner::Taxpayer, 1000000, 0));
    let r = supported(&draft);
    // $10,540 + $620 withheld, plus $620 paid through Schedule 2,
    // less the $11,439 limit = $341 excess Social Security credit.
    assert_eq!(return_line(&r, "31"), 34100);
    assert_eq!(return_line(&r, "23"), 76500);
    assert_eq!(value(&r, "agi"), 190000);
    assert_eq!(return_line(&r, "25d"), 0);
}

#[test]
fn form_8941_wage_phaseout_and_the_state_credit() {
    // Fixture C: four employees G1-G4, W-2 $50,000, $52,000, $48,000, $51,000,
    // each 2,080 h, employee-only 12 months, total $8,000, paid $4,000 (50%);
    // averages $9,000 / $22,000; line 14 expense $16,000; state tax credit
    // $2,000.
    //   l2 = 4; l3 = 201,000 / 4 = 50,250 -> 50,000
    //   cap each = 9,000 x 4,000/8,000 = 4,500; l4 = 16,000; l5 = 18,000;
    //   l6 = 16,000; l7 = 8,000; l8 = 8,000
    //   WS6: 8,000 x (50,000 - 34,100)/34,100 = 8,000 x 15,900/34,100 =
    //   3,730.21 -> 3,730; l9 = 4,270 (a three-place ratio, 0.466, would
    //   give 3,728; the App keeps full precision, which the form allows)
    //   l10 = 2,000; l11 = 14,000; l12 = min(4,270, 14,000) = 4,270.
    let rows = |state| {
        health_credit(
            ["G1", "G2", "G3", "G4"]
                .map(|label| {
                    health_employee(
                        label,
                        hours(2_080),
                        Some((HealthCoverageTier::EmployeeOnly, 12, 8_000, 4_000)),
                    )
                })
                .to_vec(),
            9_000,
            22_000,
            state,
        )
    };
    let w2s = [("G1", 50_000), ("G2", 52_000), ("G3", 48_000), ("G4", 51_000)];
    let draft = health_credit_draft(rows(2_000), &w2s, 400_000, 201_000, 16_000);
    let id = draft.income[1].id;
    let r = supported(&draft);
    for (line, expected) in [
        ("2", 4),
        ("3", 50_000),
        ("4", 16_000),
        ("5", 18_000),
        ("6", 16_000),
        ("7", 8_000),
        ("8", 8_000),
        ("9", 4_270),
        ("10", 2_000),
        ("11", 14_000),
        ("12", 4_270),
    ] {
        assert_eq!(health_line(&r, id, line), expected, "line {line}");
    }
    // C2: a $15,000 state credit leaves line 11 = 1,000, which binds.
    let draft = health_credit_draft(rows(15_000), &w2s, 400_000, 201_000, 16_000);
    let r = supported(&draft);
    assert_eq!(health_line(&r, id, "11"), 1_000);
    assert_eq!(health_line(&r, id, "12"), 1_000);
    // C3: the wage boundary. One employee, 2,080 h, employee-only 12 months,
    // total $8,000, paid $4,000 (cap $4,500, l7 $2,000).
    //   W-2 $68,500 -> l3 = 68,000: 2,000 x 33,900/34,100 = 1,988.27 -> 1,988;
    //   l12 = 12. W-2 $68,999 -> still 68,000. W-2 $69,000 -> l3 = 69,000,
    //   above twice $34,100: l12 = 0.
    let single = |wages: i64| {
        let credit = health_credit(
            vec![health_employee(
                "H1",
                hours(2_080),
                Some((HealthCoverageTier::EmployeeOnly, 12, 8_000, 4_000)),
            )],
            9_000,
            22_000,
            0,
        );
        credit.form_lines(&health_payroll(&[("H1", wages)]))
    };
    assert_eq!((single(68_500)[2], single(68_500)[11]), (68_000, 12));
    assert_eq!((single(68_999)[2], single(68_999)[11]), (68_000, 12));
    assert_eq!((single(69_000)[2], single(69_000)[11]), (69_000, 0));
    assert_eq!(single(69_000)[3], 0);
    // At and just above the start of the phaseout: $34,999 rounds to
    // $34,000 and costs nothing; $35,000 costs 2,000 x 900/34,100 = 52.786
    // -> 53 of line 7, so l9 = 1,947.
    assert_eq!((single(34_999)[2], single(34_999)[8]), (34_000, 2_000));
    assert_eq!((single(35_000)[2], single(35_000)[8]), (35_000, 1_947));
    // A seasonal worker's premiums count, their hours and wages do not:
    // 1 FTE from the first employee alone, wages $40,000 over 1.
    let credit = health_credit(
        vec![
            health_employee(
                "H1",
                hours(2_080),
                Some((HealthCoverageTier::EmployeeOnly, 12, 8_000, 4_000)),
            ),
            health_employee(
                "S1",
                HealthCreditService::Seasonal,
                Some((HealthCoverageTier::Family, 3, 3_000, 1_500)),
            ),
        ],
        9_000,
        22_000,
        0,
    );
    // l4 = 4,000 + 1,500 = 5,500; seasonal cap = 22,000 x 3/12 x 1,500/3,000
    // = 2,750; l5 = 4,500 + 2,750 = 7,250; l6 = 5,500; l7 = 2,750; l3 =
    // 40,000 -> 34,100 start: 2,750 x 5,900/34,100 = 475.80 -> 476; l9 = 2,274.
    let lines = credit.form_lines(&health_payroll(&[("H1", 40_000), ("S1", 9_000)]));
    assert_eq!(&lines[..3], &[2, 1, 40_000]);
    assert_eq!(&lines[3..9], &[5_500, 7_250, 5_500, 2_750, 2_750, 2_274]);
    assert_eq!(lines[11], 2_274);
    // Enrolled counts: both rows are enrolled; FTEs of enrolled hours = 1.
    assert_eq!((lines[12], lines[13]), (2, 1));
}

#[test]
fn form_8994_wage_method_follows_the_instructions_example() {
    // Fixture A (i8994 "Figuring the credit", Example 2), amounts in cents.
    //   A: cap 25.00 x 160 h = 4,000.00; credit wages 4,000.00;
    //      4,000.00 x 10000 / 40000 = 1,000.00 (25%)
    //   B: cap 25.00 x 80 h = 2,000.00; credit wages 1,500.00 (below the cap);
    //      1,500.00 x 7500 / 40000 = 281.25 (18.75%)
    //   l1a = dollars(128,125) = 1,281; l1b 0; l2 0; l3 = 1,281 to Form 3800
    //   Part III line 4j. Schedule C line 26 = 104,000 - 1,281 = 102,719;
    //   net profit = 300,000 - 102,719 = 197,281.
    let draft = leave_fixture_a();
    let id = draft.income[1].id;
    let r = supported(&draft);
    for (line, expected) in [("1a", 1_281), ("1b", 0), ("2", 0), ("3", 1_281)] {
        assert_eq!(leave_line(&r, id, line), expected, "line {line}");
    }
    assert_eq!(
        printed_line(&r, &alloc::format!("business_{id}_form8994_line1a")),
        (TaxForm::Form8994, "1a".into())
    );
    assert_eq!(leave_expense_line(&r, id, "line26_wages"), 102_719);
    assert_eq!(value(&r, &alloc::format!("business_{id}_net_profit")), 197_281);
    assert_eq!(value(&r, "business_credit_paid_leave_current_year"), 1_281);
    assert_eq!(
        printed_line(&r, "business_credit_paid_leave_current_year"),
        (TaxForm::Form3800, "4j".into())
    );
    // The same estimate through the stored record's own arithmetic.
    let claim = restaurant_business(&draft).paid_leave_credit.as_ref().unwrap();
    assert_eq!(claim.form_lines(), [1_281, 0]);
    assert_eq!(claim.credit(), 1_281);
}
