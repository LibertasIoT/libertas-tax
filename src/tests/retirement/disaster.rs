// Form 8915-F (T3-8915): hand-computed return and boundary tests. Every
// expected amount is worked from the Form 8915-F (Rev. December 2025) lines
// and its instructions, not copied from the App.
use super::*;

// A qualified disaster distribution for DR-4900 (began March 10, 2026,
// declared March 20, 2026, so the distribution period ends 179 days after
// March 20: September 15, 2026).
fn qualified(
    amount: i64,
    distributed: u32,
    spread: DisasterSpread,
    repaid: i64,
) -> DisasterDistribution {
    DisasterDistribution::Qualified {
        fema: 4900,
        began: 20260310,
        declared: 20260320,
        distributed,
        amount,
        earlier: 0,
        spread,
        repaid: if repaid == 0 {
            crate::DisasterRepayment::No
        } else {
            crate::DisasterRepayment::Yes { total: repaid, in_return_year: 0 }
        },
    }
}
fn distribution(
    kind: RetirementKind,
    gross: i64,
    withholding: i64,
    disaster: DisasterDistribution,
) -> FederalIncomeEntry {
    income(FederalIncome::Retirement {
        data: retirement_source(FederalRetirement {
            form_boxes: Default::default(),
            trump: false,
            box_2a: gross,
            box_3: 0,
            box_6: 0,
            box_8a: 0,
            box_8b_percent: None,
            box_9a_percent: None,
            ira: (kind).ira()
                && (crate::DistributionCodes::CODE_1.0) & crate::DistributionCodes::CODE_N.0 == 0,
            box_9b: None,
            box_10: 0,
            box_10_basis: None,
            disability: DisabilityIncome::None,
            disability_wages: crate::DisabilityWages::None,
            kind,
            gross,
            taxable: Some(RetirementTaxable::Payer),
            withholding,
            box7: crate::DistributionCodes::CODE_1.0,
            ordinary_distribution: Some(RetirementTreatment::Early),
            disaster,
            after_death: None,
        }),
    })
}
fn carry(
    year: Form8915Year,
    disaster_year: u32,
    line_11: i64,
    line_22: i64,
    repayments: Form8915Repayments,
) -> DisasterCarry {
    DisasterCarry {
        owner: Owner::Taxpayer,
        form: EarlierForm8915 { year, disaster_year, line_11, line_22, repayments },
    }
}
fn filed_2025() -> Form8915Year {
    Form8915Year::Filed2025 { line_13: 0, line_14: 0, line_24: 0, line_25: 0 }
}
// Single filer born 1980-06-15 (age 46: early distributions) with a W-2 of
// $60,000 and $6,000 withheld.
fn base() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 6_000_000, 600_000)];
    d.next_id = 2;
    d
}
fn one_plan(disaster: DisasterDistribution) -> FederalDraft {
    let mut d = base();
    add(&mut d, distribution(RetirementKind::Pension, 3_000_000, 0, disaster));
    d
}

#[test]
fn form_8915_f_spreads_designated_distributions_and_carries_earlier_forms_by_hand() {
    // 1099-R #1, 401(k), code 1: box 1 = 2a = $30,000, box 4 $6,000.
    //   $20,000 designated for DR-4900, spread over 3 years, $3,000 repaid
    //   to the plan before filing.
    // 1099-R #2, traditional IRA, code 1: box 1 = 2a = $6,000.
    //   $2,000 designated for DR-4900 (with #1: $22,000, exactly line 1e).
    // Earlier forms: 2025 Form 8915-F (2025 disasters) line 11 $4,000,
    //   line 22 $1,500; 2024 Form 8915-F (2024 disasters) line 22 $900 with
    //   $300 repaid to an IRA in 2026.
    //
    // 2026 Form 8915-F (2026 disasters):
    //   line 6 = 20,000 + 2,000                         = $22,000
    //   line 8 = $20,000; line 9 cost = 20,000 - 30,000 x 20,000 / 30,000 = $0
    //   line 10 = $20,000; line 11 = 20,000 / 3          = $6,666.67
    //   line 13 = $6,666.67; line 14 (Worksheet 3 line 4) = $3,000
    //   line 15 = 6,666.67 - 3,000                      = $3,666.67
    //   line 20 = 6,000 x 2,000 / 6,000 = $2,000; line 22 = 2,000 / 3 = $666.67
    //   line 26                                         = $666.67
    //   2027 installment = 6,666.67 + 666.67 = $7,333.34; 2028 = 22,000
    //     - 6,666.67 - 666.67 - 7,333.34              = $7,333.32
    // 2026 Form 8915-F (2025 disasters): line 12 = $4,000 = line 15;
    //   line 23 = $1,500 = line 26.
    // 2026 Form 8915-F (2024 disasters): line 23 = $900, line 25 = $300,
    //   line 26 = $600.
    //
    // Form 1040:
    //   4a = $6,000; 4b = 6,000 - 2,000 + 666.67 + 1,500 + 600 = $6,766.67 -> $6,767
    //   5a = $30,000; 5b = 30,000 - 20,000 + 3,666.67 + 4,000 = $17,666.67 -> $17,667
    //   9 = 60,000 + 6,767 + 17,667                     = $84,434 = 11 (AGI)
    //   15 = 84,434 - 16,100                            = $68,334
    //   16: Tax Table row $68,300-$68,350, midpoint $68,325:
    //       1,240 + 4,560 + 22% x (68,325 - 50,400) = 1,240 + 4,560 + 3,943.50
    //                                                   = $9,743.50 -> $9,744
    //   Form 5329 line 1 (qualified disaster distributions left out, i5329):
    //       (30,000 - 20,000) + (6,000 - 2,000) = $14,000; 10% = $1,400
    //   Schedule 2 line 8 = $1,400; 24 = 9,744 + 1,400  = $11,144
    //   25a = $6,000; 25b = $6,000; 33 = $12,000; 34 = 12,000 - 11,144 = $856
    let mut d = base();
    add(
        &mut d,
        distribution(
            RetirementKind::Pension,
            3_000_000,
            600_000,
            qualified(2_000_000, 20260501, DisasterSpread::OverThreeYears, 300_000),
        ),
    );
    add(
        &mut d,
        distribution(
            RetirementKind::Ira,
            600_000,
            0,
            qualified(200_000, 20260601, DisasterSpread::OverThreeYears, 0),
        ),
    );
    d.screening.as_mut().unwrap().disaster_carries = vec![
        carry(filed_2025(), 2025, 400_000, 150_000, Form8915Repayments::default()),
        carry(
            Form8915Year::Filed2024,
            2024,
            0,
            90_000,
            Form8915Repayments {
                plan_repaid: 0,
                ira_repaid: 30_000,
                plan_carried_back: 0,
                ira_carried_back: 0,
            },
        ),
    ];
    let r = supported(&d);
    let line = |id: &str| {
        r.lines.iter().find(|l| l.id == id).unwrap_or_else(|| panic!("missing {id}")).amount
    };
    assert_eq!(line("form_8915_taxpayer_2026_line_6"), 2_200_000);
    assert_eq!(line("form_8915_taxpayer_2026_line_9"), 0);
    assert_eq!(line("form_8915_taxpayer_2026_line_11"), 666_667);
    assert_eq!(line("form_8915_taxpayer_2026_line_14"), 300_000);
    assert_eq!(line("form_8915_taxpayer_2026_line_15"), 366_667);
    assert_eq!(line("form_8915_taxpayer_2026_line_22"), 66_667);
    assert_eq!(line("form_8915_taxpayer_2026_line_26"), 66_667);
    assert_eq!(line("form_8915_taxpayer_2026_installment_following_year"), 733_334);
    assert_eq!(line("form_8915_taxpayer_2026_installment_second_following_year"), 733_332);
    assert_eq!(line("form_8915_taxpayer_2025_line_12"), 400_000);
    assert_eq!(line("form_8915_taxpayer_2025_line_15"), 400_000);
    assert_eq!(line("form_8915_taxpayer_2025_line_26"), 150_000);
    assert_eq!(line("form_8915_taxpayer_2024_line_25"), 30_000);
    assert_eq!(line("form_8915_taxpayer_2024_line_26"), 60_000);
    assert_eq!(return_line(&r, "4a"), 600_000);
    assert_eq!(return_line(&r, "4b"), 676_700);
    assert_eq!(return_line(&r, "5a"), 3_000_000);
    assert_eq!(return_line(&r, "5b"), 1_766_700);
    assert_eq!(return_line(&r, "9"), 8_443_400);
    assert_eq!(value(&r, "agi"), 84_434);
    assert_eq!(return_line(&r, "15"), 6_833_400);
    assert_eq!(return_line(&r, "16"), 974_400);
    assert_eq!(value(&r, "early_distribution_tax"), 1_400);
    assert_eq!(return_line(&r, "24a"), 1_114_400);
    assert_eq!(return_line(&r, "25b"), 600_000);
    assert_eq!(return_line(&r, "34"), 85_600);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn form_8915_f_election_includes_everything_in_2026_without_additional_tax() {
    // $20,000 of a $30,000 401(k) distribution, all in 2026 (line 11 box):
    // line 11 = line 10 = $20,000, line 15 = $20,000; 5b = 10,000 + 20,000
    // = $30,000. Form 5329 line 1 = $10,000, tax $1,000; no 2027 installment.
    let d = one_plan(qualified(2_000_000, 20260501, DisasterSpread::AllInReturnYear, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "form_8915_taxpayer_2026_line_11"), 20_000);
    assert_eq!(value(&r, "form_8915_taxpayer_2026_installment_following_year"), 0);
    assert_eq!(return_line(&r, "5b"), 3_000_000);
    assert_eq!(value(&r, "early_distribution_tax"), 1_000);
}

#[test]
fn form_8915_f_limit_is_22000_per_disaster() {
    // Boundary: $22,000 on one 1099-R is line 1e exactly; a second 1099-R
    // adding $0.01 for the same disaster exceeds it and is refused by name.
    let d = one_plan(qualified(2_200_000, 20260501, DisasterSpread::OverThreeYears, 0));
    supported(&d);
    let mut d = d;
    add(
        &mut d,
        distribution(
            RetirementKind::Ira,
            100_000,
            0,
            qualified(1, 20260501, DisasterSpread::OverThreeYears, 0),
        ),
    );
    unsupported(&d, "Form 8915-F");
}

#[test]
fn form_8915_f_distribution_period_is_the_pages_rule_not_a_refusal_of_stored_data() {
    // Boundary: declared March 20, 2026; March 20 + 179 days = September 15. The period is the
    // 1099-R page's rule (DISASTER_OUTSIDE_PERIOD, tests/retirement_special_codes.rs), so a
    // stored record spreads whatever its dates say instead of being refused in the estimate.
    for distributed in [20260915, 20260916, 20260309] {
        supported(&one_plan(qualified(1_000_000, distributed, DisasterSpread::OverThreeYears, 0)));
    }
}

#[test]
fn form_8915_f_repayment_above_income_is_excess_and_line_15_is_zero() {
    // Boundary: $9,000 of $9,000 designated and repaid; line 11 = $3,000,
    // line 14 = $9,000, line 15 = $0, excess repayment = $6,000.
    // 5b = 30,000 - 9,000 + 0 = $21,000.
    let d = one_plan(qualified(900_000, 20260501, DisasterSpread::OverThreeYears, 900_000));
    let r = supported(&d);
    assert_eq!(value(&r, "form_8915_taxpayer_2026_line_15"), 0);
    assert_eq!(value(&r, "form_8915_taxpayer_2026_excess_repayment"), 6_000);
    assert_eq!(return_line(&r, "5b"), 2_100_000);
}

#[test]
fn form_8915_f_carried_excess_uses_2025_line_14_less_13_and_carryback() {
    // Worksheet 3: 2025 line 14 $5,000, line 13 $2,000, $1,000 carried back:
    // line 3c = 5,000 - 2,000 - 1,000 = $2,000. With 2025 line 11 $2,000:
    // line 13 = $2,000, line 14 = $2,000, line 15 = $0.
    let mut d = base();
    d.screening.as_mut().unwrap().disaster_carries = vec![carry(
        Form8915Year::Filed2025 { line_13: 200_000, line_14: 500_000, line_24: 0, line_25: 0 },
        2025,
        200_000,
        0,
        Form8915Repayments {
            plan_repaid: 0,
            ira_repaid: 0,
            plan_carried_back: 100_000,
            ira_carried_back: 0,
        },
    )];
    let r = supported(&d);
    assert_eq!(value(&r, "form_8915_taxpayer_2025_line_14"), 2_000);
    assert_eq!(value(&r, "form_8915_taxpayer_2025_line_15"), 0);
    assert_eq!(return_line(&r, "5b"), 0);
}

#[test]
fn form_8915_f_situations_outside_the_common_path_are_refused_by_name() {
    let r = estimate(&one_plan(DisasterDistribution::NotDeclared));
    assert!(
        r.issues.iter().any(|i| i.message.contains("Form 8915-F")
            && i.display.key == "ISSUE_8915_MISSING_FEMA_DECLARATION"),
        "{r:?}"
    );
    let mut d = base();
    d.screening.as_mut().unwrap().disaster_carries =
        vec![carry(Form8915Year::Earlier, 2023, 0, 0, Form8915Repayments::default())];
    unsupported(&d, "Form 8915-F");
}

#[test]
fn form_8915_f_final_return_includes_the_rest_of_every_spread() {
    // The same filer as the hand-worked fixture, now dead at 2026-10-15: lines
    // 11 and 22 take the whole taxable amounts of the 2026 distributions
    // ($20,000 and $2,000) and lines 13 and 24 add the 2025 spread's remaining
    // third that 2027 would have brought ($4,000 and $1,500 again).
    let mut d = base();
    add(
        &mut d,
        distribution(
            RetirementKind::Pension,
            3_000_000,
            600_000,
            qualified(2_000_000, 20260501, DisasterSpread::OverThreeYears, 300_000),
        ),
    );
    add(
        &mut d,
        distribution(
            RetirementKind::Ira,
            600_000,
            0,
            qualified(200_000, 20260601, DisasterSpread::OverThreeYears, 0),
        ),
    );
    d.screening.as_mut().unwrap().disaster_carries = vec![
        carry(filed_2025(), 2025, 400_000, 150_000, Form8915Repayments::default()),
        carry(
            Form8915Year::Filed2024,
            2024,
            0,
            90_000,
            Form8915Repayments {
                plan_repaid: 0,
                ira_repaid: 30_000,
                plan_carried_back: 0,
                ira_carried_back: 0,
            },
        ),
    ];
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, estate_medical: 0, date: 20261015, ordinary: true });
    let r = supported(&d);
    let line = |id: &str| {
        r.lines.iter().find(|l| l.id == id).unwrap_or_else(|| panic!("missing {id}")).amount
    };
    assert_eq!(line("form_8915_taxpayer_2026_line_11"), 2_000_000);
    assert_eq!(line("form_8915_taxpayer_2026_line_15"), 1_700_000);
    assert_eq!(line("form_8915_taxpayer_2026_line_22"), 200_000);
    assert_eq!(line("form_8915_taxpayer_2026_line_26"), 200_000);
    assert_eq!(line("form_8915_taxpayer_2026_installment_following_year"), 0);
    assert_eq!(line("form_8915_taxpayer_2026_installment_second_following_year"), 0);
    assert_eq!(line("form_8915_taxpayer_2025_line_13"), 800_000);
    assert_eq!(line("form_8915_taxpayer_2025_line_15"), 800_000);
    assert_eq!(line("form_8915_taxpayer_2025_line_24"), 300_000);
    assert_eq!(line("form_8915_taxpayer_2025_line_26"), 300_000);
    // PRINT keeps the death remainder on13/24 instead of changing the prior
    // worksheets12/23; supporting amounts are included once in those totals.
    assert_eq!(printed_8915(&r, "taxpayer", 2025, "12"), 400_000);
    assert_eq!(printed_8915(&r, "taxpayer", 2025, "23"), 150_000);
    assert_eq!(line("form_8915_taxpayer_2025_terminal_plan"), 400_000);
    assert_eq!(line("form_8915_taxpayer_2025_terminal_ira"), 150_000);
    // The 2024 spread has no installment left to accelerate.
    assert_eq!(line("form_8915_taxpayer_2024_line_26"), 60_000);
    assert_eq!(return_line(&r, "4b"), 960_000);
    assert_eq!(return_line(&r, "5b"), 3_500_000);
    assert_eq!(value(&r, "agi"), 104_600);
}

// Owner projection only: the existing Form8915-F computation stays unchanged.
// Each filer elects thirds: taxpayer pension12,000 ->4,000; spouse pension
// 6,000 ->2,000. Thus category adjustment -12,000 consists of -8,000/-4,000.
#[test]
pub(in crate::tests) fn injured_spouse_disaster_projection_keeps_two_owner_adjustments() {
    let mut d = complete(FederalStatus::Joint);
    add(
        &mut d,
        distribution(
            RetirementKind::Pension,
            1_200_000,
            0,
            qualified(1_200_000, 20260401, DisasterSpread::OverThreeYears, 0),
        ),
    );
    let mut spouse = distribution(
        RetirementKind::Pension,
        600_000,
        0,
        qualified(600_000, 20260401, DisasterSpread::OverThreeYears, 0),
    );
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    let r = supported(&d);
    let _ = FederalEstimate::from(r.clone());
    assert_eq!(return_line(&r, "5b"), 600_000);
    let mut probe = r.clone();
    let qcd = crate::retirement_return::QcdResult::default();
    let funding = crate::hsa_funding::FundingResult::default();
    let mut basis = crate::retirement_return::basis(&d, &mut probe, None, &qcd, &funding)
        .unwrap_or_else(|_| panic!("native disaster IRA basis"));
    basis.disaster = crate::disaster::designated(&d, &basis, &qcd);
    let actual = crate::disaster::calculate(&d, &mut probe, &basis, &qcd);
    assert_eq!(actual.categories, [0, -1_200_000]);
    assert_eq!(actual.owners, [[0, 0], [-800_000, -400_000]]);
}

fn printed_8915(r: &FederalResult, who: &str, year: u32, number: &str) -> i64 {
    r.lines
        .iter()
        .find(|row| row.id == alloc::format!("form_8915_{who}_{year}_print_{number}"))
        .unwrap_or_else(|| panic!("missing printed8915 {who}/{year}/{number}"))
        .amount
}

#[test]
fn print_form_8915_all_part_i_columns_use_source_gross_and_one_disaster_cap() {
    let mut d = base();
    add(
        &mut d,
        distribution(
            RetirementKind::Pension,
            3_000_000,
            0,
            qualified(2_000_000, 20260501, DisasterSpread::OverThreeYears, 300_000),
        ),
    );
    add(
        &mut d,
        distribution(
            RetirementKind::Ira,
            600_000,
            0,
            qualified(200_000, 20260601, DisasterSpread::OverThreeYears, 0),
        ),
    );
    let r = supported(&d);
    let at = |n| printed_8915(&r, "taxpayer", 2026, n);
    for (n, amount) in [
        ("1e", 2_200_000),
        ("2(a)", 3_000_000),
        ("2(b)", 2_000_000),
        ("3(a)", 600_000),
        ("3(b)", 200_000),
        ("4(a)", 0),
        ("4(b)", 0),
        ("5a", 3_600_000),
        ("5b", 1_400_000),
        ("5c(a)", 2_200_000),
        ("5c(b)", 2_200_000),
        ("6", 2_200_000),
        ("7", 1_400_000),
        ("15", 366_667),
        ("20", 200_000),
        ("21", 200_000),
        ("26", 66_667),
    ] {
        assert_eq!(at(n), amount, "{n}");
    }
    assert!(r.lines.iter().all(|row| row.id != "form_8915_taxpayer_2026_print_1a"));
    assert!(r.lines.iter().any(|row| row.id == "form_8915_taxpayer_2026_choice_17_no"));
    assert_eq!(restart(d.clone()).draft, d);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}

#[test]
fn print_form_8915_traditional_roth_basis_and_payer_amounts_are_separate() {
    let mut d = base();
    let a = add_inherited_ira(&mut d, 10_000, 5_000, 40_000);
    let b = add_inherited_ira(&mut d, 2000, 0, 0);
    let c = add_inherited_roth(&mut d, 2022, 5000, 2000, 0);
    for (id, amount) in [(a + 1, 600_000), (b + 1, 200_000), (c + 1, 250_000)] {
        let disaster = qualified(amount, 20260501, DisasterSpread::OverThreeYears, 0);
        match &mut d.income.iter_mut().find(|e| e.id == id).unwrap().income {
            FederalIncome::Retirement { data } => edit_retirement(data).disaster = disaster,
            FederalIncome::Roth { data } => edit_roth(data).disaster = disaster,
            _ => unreachable!(),
        }
    }
    let r = supported(&d);
    let at = |n| printed_8915(&r, "taxpayer", 2026, n);
    for (n, amount) in [
        ("3(a)", 1_200_000),
        ("4(a)", 500_000),
        ("3(b)", 800_000),
        ("4(b)", 250_000),
        ("18", 540_000),
        ("19", 150_000),
        ("20", 200_000),
        ("21", 890_000),
        ("22", 296_667),
        ("26", 296_667),
        ("7", 650_000),
    ] {
        assert_eq!(at(n), amount, "{n}");
    }
    assert_eq!(return_line(&r, "4b"), 806_700);
    assert!(r.lines.iter().any(|row| row.id == "form_8915_taxpayer_2026_choice_17_yes"));
    assert_eq!(restart(d.clone()).draft, d);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}

#[test]
fn print_form_8915_prior_year_forms_conserve_available_gross_per_filer() {
    let mut d = complete(FederalStatus::Joint);
    let mut old = qualified(1_000_000, 20260115, DisasterSpread::OverThreeYears, 0);
    if let DisasterDistribution::Qualified { fema, began, declared, earlier, .. } = &mut old {
        *fema = 4800;
        *began = 20251201;
        *declared = 20251231;
        *earlier = 500_000;
    }
    add(&mut d, distribution(RetirementKind::Pension, 3_000_000, 0, old));
    add(
        &mut d,
        distribution(
            RetirementKind::Ira,
            1_000_000,
            0,
            qualified(400_000, 20260501, DisasterSpread::OverThreeYears, 0),
        ),
    );
    let mut spouse = distribution(
        RetirementKind::Pension,
        600_000,
        0,
        qualified(300_000, 20260501, DisasterSpread::OverThreeYears, 0),
    );
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    let r = supported(&d);
    for (year, n, amount) in [
        (2025, "1a", 2_200_000),
        (2025, "1b", 500_000),
        (2025, "1c", 1_700_000),
        (2025, "1d", 0),
        (2025, "1e", 1_700_000),
        (2025, "2(a)", 3_000_000),
        (2025, "3(a)", 1_000_000),
        (2025, "7", 0),
        (2026, "2(a)", 2_000_000),
        (2026, "3(a)", 1_000_000),
        (2026, "5a", 3_000_000),
        (2026, "7", 2_600_000),
    ] {
        assert_eq!(printed_8915(&r, "taxpayer", year, n), amount, "{year}/{n}");
    }
    assert_eq!(printed_8915(&r, "spouse", 2026, "2(a)"), 600_000);
    assert_eq!(printed_8915(&r, "spouse", 2026, "7"), 300_000);
    let review: Vec<_> = crate::draft_return(&r)
        .into_iter()
        .filter(|row| row.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8915F))
        .collect();
    let mut starts: Vec<_> = review
        .iter()
        .enumerate()
        .filter_map(|(i, row)| (row.form_line.as_ref().unwrap().line == "A").then_some(i))
        .collect();
    assert_eq!(starts.len(), 3);
    starts.push(review.len());
    for pair in starts.windows(2) {
        assert_eq!(
            review[pair[0]..pair[1]]
                .iter()
                .filter(|row| row.form_line.as_ref().unwrap().line == "B")
                .count(),
            1
        );
    }
    assert_eq!(restart(d.clone()).draft, d);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}

fn print_home(year: u32, repaid: i64) -> DisasterDistribution {
    let (began, ended, declared, distributed) = if year == 2025 {
        (20251201, 20251220, 20260115, 20260110)
    } else {
        (20260301, 20260320, 20260415, 20260210)
    };
    DisasterDistribution::HomePurchase {
        fema: 4800,
        began,
        ended,
        declared,
        distributed,
        taxable_repaid: repaid,
    }
}

#[test]
fn print_form_8915_part_iv_excludes_repayments_already_on_form8606() {
    let mut d = base();
    let mut ira = basis_distribution(Owner::Taxpayer, 10_000, RetirementTreatment::Ordinary);
    if let FederalIncome::Retirement { data } = &mut ira.income {
        edit_retirement(data).disaster = print_home(2026, 600_000);
    }
    add(&mut d, ira);
    d.adjustments.as_mut().unwrap().ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 2000, 40000)];
    let baseline = supported(&d);
    assert!(baseline.lines.iter().any(|row| row.id == "form_8915_taxpayer_2026_choice_27_yes"));
    assert!(baseline.lines.iter().all(|row| row.id != "form_8915_taxpayer_2026_print_28"));
    add(&mut d, distribution(RetirementKind::Pension, 300_000, 0, print_home(2026, 100_000)));
    let r = supported(&d);
    for (n, amount) in
        [("28", 300_000), ("29", 0), ("30", 300_000), ("31", 100_000), ("32", 200_000)]
    {
        assert_eq!(printed_8915(&r, "taxpayer", 2026, n), amount, "{n}");
    }
    assert_eq!(return_line(&r, "4b"), return_line(&baseline, "4b"));
    assert_eq!(return_line(&r, "5b"), 200_000);
    assert_eq!(restart(d.clone()).draft, d);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}
#[test]
fn print_form_8915_part_iv_remaining_income_reaches_the_later_part_i() {
    let mut d = base();
    add(&mut d, distribution(RetirementKind::Pension, 300_000, 0, print_home(2025, 100_000)));
    add(
        &mut d,
        distribution(
            RetirementKind::Ira,
            1_000_000,
            0,
            qualified(400_000, 20260501, DisasterSpread::OverThreeYears, 0),
        ),
    );
    let r = supported(&d);
    assert_eq!(printed_8915(&r, "taxpayer", 2025, "32"), 0);
    assert_eq!(printed_8915(&r, "taxpayer", 2026, "2(a)"), 200_000);
    assert_eq!(printed_8915(&r, "taxpayer", 2026, "5a"), 1_200_000);
    assert_eq!(printed_8915(&r, "taxpayer", 2026, "7"), 800_000);
    assert_eq!(return_line(&r, "5b"), 200_000);
    assert_eq!(restart(d.clone()).draft, d);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}

#[test]
fn print_form_8915_two_disasters_use_actual_per_fema_worksheet_limits() {
    let mut d = base();
    for (kind, gross, amount, fema, earlier) in [
        (RetirementKind::Pension, 4_000_000, 2_000_000, 4800, 0),
        (RetirementKind::Ira, 3_000_000, 1_500_000, 4801, 500_000),
    ] {
        let disaster = DisasterDistribution::Qualified {
            fema,
            began: 20251201,
            declared: 20260115,
            distributed: 20260120,
            amount,
            earlier,
            spread: DisasterSpread::OverThreeYears,
            repaid: DisasterRepayment::No,
        };
        add(&mut d, distribution(kind, gross, 0, disaster));
    }
    let r = supported(&d);
    assert!(r.lines.iter().all(|row| {
        !["1a", "1b", "1c", "1d", "1e"]
            .iter()
            .any(|n| row.id == alloc::format!("form_8915_taxpayer_2025_print_{n}"))
    }));
    assert!(r.lines.iter().any(|row| row.id == "form_8915_taxpayer_2025_choice_1_worksheet"));
    for (fema, prior, limit, qualified) in
        [(4800, 0, 2_200_000, 2_000_000), (4801, 500_000, 1_700_000, 1_500_000)]
    {
        for (kind, amount) in [("prior", prior), ("limit", limit), ("qualified", qualified)] {
            assert_eq!(
                line_amount(&r, &alloc::format!("form_8915_taxpayer_2025_worksheet_{kind}_{fema}")),
                amount
            );
        }
    }
    for (n, amount) in [
        ("5a", 7_000_000),
        ("5b", 3_500_000),
        ("5c(a)", 3_500_000),
        ("5c(b)", 3_500_000),
        ("6", 3_500_000),
        ("7", 3_500_000),
    ] {
        assert_eq!(printed_8915(&r, "taxpayer", 2025, n), amount);
    }
    assert!(d.valid_stored());
    assert_eq!(restart(d.clone()).draft, d);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}
