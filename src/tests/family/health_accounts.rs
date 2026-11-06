// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn draft_2026_eic_table_matches_every_published_interval_and_column() {
    // IRS draft Publication 1040 (2026), pp. 16-27. The fixture retains each
    // published cell, including its eight starred partial-row footnotes.
    let table = include_str!("../../../tests/fixtures/irs-eic-2026-draft.csv");
    let mut previous = 1;
    let mut rows = 0;
    let mut ordinary_cells = 0;
    let mut starred_cells = 0;
    for line in table.lines().filter(|line| !line.starts_with('#')) {
        let cells: alloc::vec::Vec<&str> = line.split(',').collect();
        assert_eq!(cells.len(), 10);
        let lower: i64 = cells[0].parse().unwrap();
        let upper: i64 = cells[1].parse().unwrap();
        assert_eq!(lower, previous);
        assert_eq!(upper - lower, if lower == 1 { 49 } else { 50 });
        previous = upper;
        rows += 1;
        for column in 0..8 {
            if cells[column + 2] == "*" {
                starred_cells += 1;
                continue;
            }
            ordinary_cells += 1;
            let expected: i64 = cells[column + 2].parse().unwrap();
            let child_count = column % 4;
            let joint = column >= 4;
            for income in [lower, (lower + upper) / 2, upper - 1] {
                assert_eq!(
                    crate::forms::schedule_eic::eic_table_2026(income, child_count, joint),
                    expected,
                    "2026 EIC income={income}, child_count={child_count}, joint={joint}"
                );
            }
        }
    }
    assert_eq!((rows, ordinary_cells, starred_cells, previous), (1405, 11232, 8, 70250));
    assert_eq!(crate::forms::schedule_eic::eic_table_2026(0, 0, false), 0);
    assert_eq!(crate::forms::schedule_eic::eic_table_2026(70250, 3, true), 0);
    for (lower, cutoff, children, joint, credit) in [
        (19500, 19540, 0, false, 2),
        (26800, 26820, 0, true, 1),
        (51550, 51593, 1, false, 3),
        (58600, 58629, 2, false, 3),
        (58850, 58863, 1, true, 1),
        (62950, 62974, 3, false, 2),
        (65850, 65899, 2, true, 5),
        (70200, 70244, 3, true, 5),
    ] {
        assert_eq!(crate::forms::schedule_eic::eic_table_2026(lower, children, joint), credit);
        assert_eq!(crate::forms::schedule_eic::eic_table_2026(cutoff - 1, children, joint), credit);
        assert_eq!(crate::forms::schedule_eic::eic_table_2026(cutoff, children, joint), 0);
    }
}
#[test]
fn eic_worksheet_uses_earned_income_then_only_applicable_agi_lookup() {
    assert_eq!(crate::forms::schedule_eic::eic_table_2026(1, 0, false), 2);
    assert_eq!(crate::forms::schedule_eic::eic_table_2026(49, 0, false), 2);
    assert_eq!(crate::forms::schedule_eic::eic_table_2026(50, 0, false), 6);
    assert_eq!(eic_amount(5000, 0, 1, false), 1709);
    assert_eq!(eic_amount(5000, -10, 1, false), 1709);
    assert_eq!(eic_amount(10000, 5000, 0, false), 664);
    assert_eq!(eic_amount(30000, 25000, 1, false), 3447);
    assert_eq!(eic_amount(20000, 30000, 1, false), 3447);
    assert_eq!(eic_amount(0, 30000, 1, false), 0);
}
#[test]
fn eic_table_agi_lookup_changes_complete_return_and_survives_restart() {
    let mut d = complete(FederalStatus::Head);
    d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 0);
    add_child(&mut d);
    assert_eq!(value(&supported(&d), "earned_income_credit"), 4427);
    add(&mut d, income(savings_income(10000, 0)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 30000);
    assert_eq!(value(&r, "earned_income_credit"), 3447);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn adoption_phaseout_does_not_phase_out_prior_carries_or_make_them_refundable() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 28_508_000, 0);
    d.credits.as_mut().unwrap().adoptions.push(adoption_child(
        AdoptionKind::Domestic {
            finalized: AdoptionFinalization::Final { year: 2026 },
            special_needs: true,
        },
        &[],
    ));
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_refundable_credit"), 5120);
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 3715);
    d.credits.as_mut().unwrap().adoption_carry = vec![
        AdoptionCarry { year: 2021, amount: 200_000, eligible: true },
        AdoptionCarry { year: 2025, amount: 300_000, eligible: true },
    ];
    d.income[0] = wage(1, Owner::Taxpayer, 30_508_000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_refundable_credit"), 0);
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 5000);
    assert_eq!(value(&r, "adoption_carryforward"), 0);
    d.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    d.credits.as_mut().unwrap().adoptions.clear();
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_expired_2021"), 2000);
    assert_eq!(value(&r, "adoption_carryforward"), 3000);
    assert_eq!(value(&r, "adoption_refundable_credit"), 0);
    d.credits.as_mut().unwrap().adoption_carry.push(AdoptionCarry {
        year: 2025,
        amount: 0,
        eligible: true,
    });
    assert!(!d.valid_stored());
}

#[test]
fn care_for_incapable_adult_waives_income_and_joint_return_dependency_tests_only() {
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    let adult = &mut d.dependents[0];
    adult.relationship = Relationship::Parent;
    adult.birth_date = 19500101;
    adult.incapable_self_care = true;
    adult.situation =
        DependentSituation::Supported { your_support_over_half: true, gross_income: 2000000 };
    set_fact(adult, PersonFacts::FILED_JOINTLY, true);
    adult.set_care_expenses(300000);
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&r, "care_credit"), 1050);
    assert_eq!(restart(d.clone()).draft, d);
    // Without the filer's support, or without incapability of self-care, the
    // parent is not a qualifying person: the costs are left out with one
    // notice (they refused the estimate before the family unit, 2026-10-08).
    d.dependents[0].situation = DependentSituation::Supported {
        your_support_over_half: false,
        gross_income: d.dependents[0].gross_income(),
    };
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert!(noted(&r, "WARN_CARE_COSTS_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "care_credit"), 0);
    d.dependents[0].situation = DependentSituation::Supported {
        your_support_over_half: true,
        gross_income: d.dependents[0].gross_income(),
    };
    d.dependents[0].incapable_self_care = false;
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert!(noted(&r, "WARN_CARE_COSTS_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "care_credit"), 0);
}

#[test]
fn prior_care_is_additional_to_current_expense_cap_but_still_nonrefundable() {
    let mut d = complete(FederalStatus::Single);
    d.credits.as_mut().unwrap().care =
        FederalCare { prior_year: Some(prior_care_facts()), ..FederalCare::default() };
    document_care(&mut d);
    add_child(&mut d);
    d.dependents[0].set_care_expenses(300000);
    assert_eq!(
        value(
            &{
                document_care(&mut d);
                supported(&d)
            },
            "care_credit"
        ),
        1050 + 132
    );
    d.dependents.clear();
    d.income[0] = wage(1, Owner::Taxpayer, 1700000, 0);
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&r, "prior_year_care_credit"), 132);
    assert_eq!(value(&r, "care_credit"), 91);
    d.income[0] = wage(1, Owner::Taxpayer, 1600000, 0);
    assert_eq!(
        value(
            &{
                document_care(&mut d);
                supported(&d)
            },
            "care_credit"
        ),
        0
    );
}
#[test]
fn two_employee_care_benefits_above_joint_limit_reconcile_taxable_wages_once() {
    let mut d = joint_employer_care_return();
    care_box10(&mut d, 0, 500_000);
    care_box10(&mut d, 1, 500_000);
    let r = supported(&d);
    assert_eq!(value(&r, "care_form2441_line12"), 10_000);
    assert_eq!(value(&r, "care_benefits_excluded"), 7_500);
    assert_eq!(value(&r, "care_taxable_taxpayer"), 1_250);
    assert_eq!(value(&r, "care_taxable_spouse"), 1_250);
    assert_eq!(value(&r, "care_benefits_taxable"), 2_500);
    assert_eq!(value(&r, "care_earned_taxpayer"), 101_250);
    assert_eq!(value(&r, "care_earned_spouse"), 41_250);
    assert_eq!(return_line(&r, "1a"), 14_000_000);
    assert_eq!(return_line(&r, "1e"), 250_000);
    assert_eq!(return_line(&r, "25a"), 900_000);
    assert_eq!(value(&r, "care_credit"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn two_employee_care_expense_or_earnings_limit_can_leave_taxable_benefits() {
    let mut expense = joint_employer_care_return();
    care_costs(&mut expense, 300_000);
    let result = supported(&expense);
    assert_eq!(value(&result, "care_benefits_excluded"), 3_000);
    assert_eq!(value(&result, "care_taxable_taxpayer"), 500);
    assert_eq!(value(&result, "care_taxable_spouse"), 500);
    assert_eq!(return_line(&result, "1e"), 100_000);
    assert_eq!(return_line(&result, "1a"), 14_000_000);
    assert_eq!(return_line(&result, "25a"), 900_000);
    assert_eq!(supported(&restart(expense).draft), result);
    let mut earnings = joint_employer_care_return();
    if let FederalIncome::Wage { data } = &mut earnings.income[1].income {
        let mut data = edit_wage(data);
        data.wages = 250_000;
    }
    let result = supported(&earnings);
    assert_eq!(value(&result, "care_benefits_excluded"), 2_500);
    assert_eq!(value(&result, "care_taxable_taxpayer"), 750);
    assert_eq!(value(&result, "care_taxable_spouse"), 750);
    assert_eq!(value(&result, "care_earned_spouse"), 3_250);
    assert_eq!(return_line(&result, "1a"), 10_250_000);
    assert_eq!(return_line(&result, "1e"), 150_000);
    assert_eq!(return_line(&result, "25a"), 900_000);
    assert_eq!(supported(&restart(earnings).draft), result);
}
#[test]
fn joint_care_employer_source_and_fully_excludable_bounds_are_explicit() {
    let d = joint_employer_care_return();
    let mut missing = d.clone();
    missing.credits.as_mut().unwrap().care.benefits[1].wage = 9999;
    assert!(!missing.valid_stored());
    let mut duplicate = d.clone();
    let mut repeated = duplicate.income[1].clone();
    repeated.id = duplicate.next_id;
    duplicate.next_id += 1;
    let id = repeated.id;
    duplicate.income.push(repeated);
    care_w2(&mut duplicate, id, 200_000, 0, 0, 500_000);
    supported(&duplicate);
    let mut excess = d.clone();
    care_box10(&mut excess, 0, 600_000);
    excess.credits.as_mut().unwrap().care.plan_maximum =
        CarePlanMaximum::Plans { maximum: 750_000 };
    assert_eq!(value(&supported(&excess), "care_benefits_taxable"), 500);
    let mut costs = d.clone();
    care_costs(&mut costs, 300_000);
    assert_eq!(value(&supported(&costs), "care_benefits_taxable"), 1_000);
    let mut earnings = d.clone();
    if let FederalIncome::Wage { data } = &mut earnings.income[1].income {
        let mut data = edit_wage(data);
        data.wages = 100_000;
    }
    assert_eq!(value(&supported(&earnings), "care_benefits_taxable"), 3_000);
    let mut separate = d;
    separate.setup.as_mut().unwrap().status =
        FilingChoice::Separate { section179: Section179Spouse::NotSupplied };
    assert!(!separate.valid_stored()); // Departed spouse sources require normal pruning.
}
#[test]
fn care_student_months_use_actual_or_deemed_wages_without_adding_taxable_income() {
    let mut d = benefit_return(FederalStatus::Joint);
    clear_care_benefits(&mut d);
    d.dependents.pop();
    d.dependents[0].set_care_expenses(300000);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Spouse, 50000, 0));
    let mut months = care_months(Owner::Spouse, 5);
    months[0].wages = 10000;
    months[1].wages = 40000;
    set_care_months(&mut d, months);
    let r = supported(&d);
    assert_eq!(value(&r, "care_earned_spouse"), 1250);
    assert_eq!(value(&r, "care_credit"), 438);
    assert_eq!(value(&r, "agi"), 100500);
    assert_eq!(return_line(&r, "1z"), 10050000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn able_credit_preserves_income_and_joint_spouse_eligibility() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 5_000_000, 0);
    let before = supported(&d);
    let mut f = saver(0, 0);
    f.taxpayer_able = 200_000;
    f.spouse_able = 200_000;
    d.credits.as_mut().unwrap().saver = Some(f).into();
    let r = supported(&d);
    assert_eq!(value(&r, "saver_credit"), 800);
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    assert_eq!(restart(d.clone()).draft, d);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    assert_eq!(value(&supported(&d), "saver_credit"), 400);
}

#[test]
fn dependent_student_education_uses_actual_claim_with_nonrefundable_limit() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2000000, 0);
    add(&mut d, income(savings_income(1000, 0)));
    let p = &mut d.people.as_mut().unwrap().taxpayer;
    p.birth_date = 20060615;
    p.full_time_student = true;
    p.support_share = SupportShare::LessThanHalf;
    p.dependency = FilerDependency::CanBeClaimed { claimed: false };
    let sid0_1659 = -1; // You, the first person of the catalog
    add_education(d.credits.as_mut().unwrap(), {
        let mut pair = native_education(
            sid0_1659,
            education_expenses(400000),
            education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
        );
        pair.0.id = 0;
        pair.0.half_time = true;
        pair.0.graduate = false;
        pair.1.more.american_opportunity_used_up = Some(false);
        pair
    });
    let r = supported(&d);
    assert_eq!(value(&r, "education_credit"), 493);
    assert_eq!(value(&r, "refundable_education_credit"), 0);
    assert_eq!(r.tax, Some(0));
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    let r = supported(&d);
    assert_eq!(value(&r, "education_credit"), 0);
    assert_eq!(r.tax, Some(49300));
}

#[test]
fn scholarship_award_restrictions_and_degree_status_determine_taxable_income() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![income(scholarship(10000, 3000, 7000))];
    d.income[0].id = 1;
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_scholarships"), 3000);
    assert_eq!(value(&r, "agi"), 3000);
    assert_eq!(return_line(&r, "8"), 300000);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::Scholarship {
        data: Scholarship::Education { data: crate::EducationAward { degree_candidate, .. } },
    } = &mut d.income[0].income
    else {
        panic!()
    };
    *degree_candidate = false;
    assert_eq!(value(&supported(&d), "agi"), 10000);
    d.income[0].income = scholarship(10000, 3000, 7001);
    assert!(!d.valid_stored());
}

#[test]
fn taxable_scholarship_is_standard_deduction_earnings_but_not_eic_earnings() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![income(scholarship(5000, 5000, 0))];
    d.income[0].id = 1;
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    assert_eq!(value(&supported(&d), "deduction"), 5450);
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    assert_eq!(value(&supported(&d), "earned_income_credit"), 0);
    let p = &mut d.people.as_mut().unwrap().taxpayer;
    p.birth_date = 20060615;
    p.full_time_student = true;
    p.support_share = SupportShare::LessThanHalf;
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 0);
    assert_eq!(r.tax, Some(0));
    add(&mut d, income(savings_income(12000, 0)));
    unsupported(&d, "Investment tax");
}

#[test]
fn kiddie_parent_allocation_is_limited_to_child_taxable_income() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    d.income[0].income = savings_income(20000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 3900);
    assert_eq!(value(&r, "regular_income_tax"), 858);
}
#[test]
fn kiddie_parent_preferential_income_is_not_capped_at_parent_taxable_income() {
    let mut p = ordinary_parent();
    p.status = FederalStatus::Single;
    p.taxable = 1000000;
    p.gains = Some(ParentGains {
        investment_election: 0,
        collectibles: 0,
        unrecaptured: 0,
        qualified: 2000000,
        long_term: 0,
        combined: 0,
    });
    // Carry the full preference into the combined computation. Both remain
    // within the zero-rate band; the child still owes their own-rate minimum.
    assert_eq!(p.calculate(7300, 8650, FederalStatus::Single, [0, 0], 0, [0; 3]), (0, 868));
}

#[test]
fn kiddie_medical_itemizing_uses_deduction_in_gain_allocation() {
    let mut d = young_interest_return();
    d.income[0].income =
        FederalIncome::Dividend { data: dividend_source(native_dividend(600000, 600000, 400000)) };
    itemized(d.deductions.as_mut().unwrap()).medical = 400000;
    let r = supported(&d);
    // $4,000 medical less $750 AGI floor => $3,250 itemized deduction.
    assert_eq!(return_line(&r, "12e"), 325000);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 6750);
    assert_eq!(value(&r, "kiddie_tax_allocated_dividends"), 4050);
    assert_eq!(value(&r, "kiddie_tax_allocated_capital_gain"), 2700);
    assert_eq!(return_line(&r, "16"), 101300);
    assert_eq!(restart(d.clone()).draft, d);
    d.income[0].income = savings_income(10000, 0);
    assert_eq!(return_line(&supported(&d), "16"), 148500);
}
#[test]
fn kiddie_personal_itemized_taxes_reduce_taxable_not_gross_unearned_income() {
    let mut d = young_interest_return();
    let ded = d.deductions.as_mut().unwrap();
    itemized(ded).personal_property_taxes = 400000;
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 6000);
    assert_eq!(value(&r, "regular_income_tax"), 1320);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn child_interest_tracing_applies_when_taxable_income_limits_net_unearned_income() {
    let mut d =
        child_interest_gains(InvestmentInterestUse::Mixed { paid: 100000, carryover: 200000 });
    itemized(d.deductions.as_mut().unwrap()).medical = 1150000;
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 4000);
    assert_eq!(value(&r, "kiddie_tax_preferential_interest"), 3000);
    // Worksheet #3: 3,000 direct + (16,000 - 3,000) * .5 indirect;
    // divide the total 9,500 deduction .4/.6 between dividends and gains.
    assert_eq!(value(&r, "kiddie_tax_allocated_dividends"), 200);
    assert_eq!(value(&r, "kiddie_tax_allocated_capital_gain"), 300);
}

#[test]
fn able_distribution_matches_irs_example_and_flows_to_income_and_additional_tax() {
    for corrective in [false, true] {
        let mut d = complete(FederalStatus::Single);
        let (entry, raw_amount, taxable, additional, agi, income_line, tax_line) = if corrective {
            (able_correction(2026, 300), 30000, 300, 0, 60300, 30000, 0)
        } else {
            (able_distribution(2400, 400, 1600), 13333, 133, 13, 60133, 13300, 1300)
        };
        assert_eq!(crate::interview::document_amount(&entry, false), Some(raw_amount));
        add(&mut d, income(entry));
        let r = supported(&d);
        assert_eq!(value(&r, "able_taxable_distributions"), taxable);
        assert_eq!(value(&r, "account_additional_tax"), additional);
        assert_eq!(value(&r, "agi"), agi);
        if corrective {
            assert_eq!(value(&r, "able_corrective_earnings"), 300);
        }
        assert_eq!(return_line(&r, "8"), income_line);
        assert_eq!(return_line(&r, "23"), tax_line);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn able_qualified_expenses_exclude_earnings_without_deducting_principal() {
    for (gross, earnings, expenses) in
        [(2400, 400, 2400), (2400, 400, 5000), (2400, 0, 0), (0, 0, 0)]
    {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(able_distribution(gross, earnings, expenses)));
        let r = supported(&d);
        assert_eq!(value(&r, "able_taxable_distributions"), 0);
        assert_eq!(value(&r, "account_additional_tax"), 0);
        assert_eq!(value(&r, "agi"), 60000);
    }
}

#[test]
fn able_joint_additional_tax_is_rounded_per_recipient_not_combined() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, income(able_distribution(100, 5, 0)));
    let mut spouse = income(able_distribution(100, 5, 0));
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    let r = supported(&d);
    assert_eq!(value(&r, "able_taxable_distributions"), 10);
    assert_eq!(value(&r, "account_additional_tax"), 2);
    assert_eq!(return_line(&r, "23"), 200);
}

#[test]
fn able_invalid_earnings_and_high_income_niit_are_distinguished() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(able_distribution(100, 101, 0)));
    assert!(!d.valid_stored());
    d.income.pop();
    d.income[0] = wage(1, Owner::Taxpayer, 21000000, 0);
    add(&mut d, income(able_distribution(1000, 500, 0)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 210500);
    assert_eq!(value(&r, "able_taxable_distributions"), 500);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn taxable_able_earnings_raise_niit_magi_but_are_not_niit_income() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 19_500_000, 0);
    add(&mut d, income(savings_income(1_000, 0)));
    let before = supported(&d);
    assert_eq!(value(&before, "agi"), 196_000);
    assert_eq!(value(&before, "net_investment_income_tax"), 0);

    add(&mut d, income(able_distribution(20_000, 10_000, 0)));
    let after = supported(&d);
    assert_eq!(value(&after, "agi"), 206_000);
    assert_eq!(value(&after, "able_taxable_distributions"), 10_000);
    assert_eq!(value(&after, "net_investment_income_tax"), 38);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn able_investment_interest_coordination_does_not_silently_drop_income() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(able_distribution(1000, 500, 0)));
    investment_borrowing(&mut d, 10000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "able_taxable_distributions"), 500);
    assert_eq!(value(&r, "investment_interest_deduction"), 500);
    assert_eq!(value(&r, "investment_interest_carryforward"), 9500);
}

#[test]
fn able_investment_interest_with_identical_endpoint_worksheets_is_supported() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(savings_income(1_000, 0)));
    add(&mut d, income(able_distribution(1_000, 500, 0)));
    investment_borrowing(&mut d, 400, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "able_taxable_distributions"), 500);
    assert_eq!(value(&r, "investment_interest_deduction"), 400);
    assert_eq!(value(&r, "investment_interest_carryforward"), 0);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 400);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 0);
    assert_eq!(estimate(&restart(d.clone()).draft), r);

    // The existing education-account income owner supplies the actual
    // taxable earnings once even when the borrowing limit binds.
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().loans[0].paid = 120_000;
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 1200);
    assert_eq!(value(&r, "investment_interest_carryforward"), 0);
    assert_eq!(value(&r, "agi"), 61500);
}

#[test]
fn able_correction_for_prior_contribution_year_does_not_inflate_current_income() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(able_correction(2025, 300)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(value(&r, "able_corrective_earnings"), 0);
    assert_eq!(value(&r, "account_additional_tax"), 0);
    assert!(r.review_notes.iter().any(|n| n.contains("2025 return") && n.contains("amendment")));
    let note = &r
        .review_notes
        .iter()
        .find(|n| n.contains("ABLE correction") && n.contains("2025 return"))
        .unwrap()
        .display;
    assert_eq!(note.key, "NOTE_ABLE_CORRECTION_PRIOR_RETURN");
    assert_eq!(note.args[1], MessageArg::Number(2025));
}

#[test]
fn able_excess_uses_twenty_thousand_base_and_year_end_tax_cap() {
    let mut d = complete(FederalStatus::Single);
    for (amount, balance, excess, tax) in [
        (20000, 25000, 0, 0),
        (25000, 30000, 5000, 300),
        (25000, 2000, 5000, 120),
        (25000, 0, 5000, 0),
    ] {
        d.screening.as_mut().unwrap().able_contributions =
            vec![able_contributions(0, amount, balance)];
        let r = supported(&d);
        assert_eq!(value(&r, "able_taxpayer_excess_contributions"), excess);
        assert_eq!(value(&r, "able_taxpayer_excess_tax"), tax);
        assert_eq!(return_line(&r, "23"), tax * 100);
        assert_eq!(value(&r, "agi"), 60000);
    }
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn uncorrected_additional_able_account_keeps_2026_aggregate_tax_calculation() {
    let mut d = complete(FederalStatus::Single);
    let mut accounts = able_contributions(16_000, 6_000, 2_500);
    accounts.additional_open = true;
    d.screening.as_mut().unwrap().able_contributions = vec![accounts];
    add(&mut d, income(able_distribution(1_000, 200, 600)));
    let r = supported(&d);
    assert_eq!(value(&r, "able_taxpayer_excess_contributions"), 2_000);
    assert_eq!(value(&r, "able_taxpayer_excess_tax"), 120);
    assert_eq!(value(&r, "able_taxable_distributions"), 80);
    assert!(r.review_notes.iter().any(|note| note.contains("immediately after that deadline")));
    assert_eq!(estimate(&restart(d.clone()).draft), r);

    d.screening.as_mut().unwrap().able_contributions[0].additional_open = false;
    let one_account = supported(&d);
    assert_eq!(value(&one_account, "able_taxpayer_excess_tax"), 120);
    assert_eq!(value(&one_account, "able_taxable_distributions"), 80);
}

#[test]
fn able_work_allowance_uses_prior_year_poverty_by_residence() {
    for (residence, allowance, excess, tax) in [
        (MarketplaceResidence::Contiguous, 15650, 4350, 261),
        (MarketplaceResidence::Alaska, 19550, 450, 27),
        (MarketplaceResidence::Hawaii, 17990, 2010, 121),
    ] {
        let mut d = complete(FederalStatus::Single);
        let mut entry = able_contributions(20000, 20000, 40000);
        entry.work = AbleWork::Working { compensation: 2000000, residence, no_plan: true };
        d.screening.as_mut().unwrap().able_contributions = vec![entry];
        let r = supported(&d);
        assert_eq!(value(&r, "able_taxpayer_contribution_limit"), 20000 + allowance);
        assert_eq!(value(&r, "able_taxpayer_excess_contributions"), excess);
        assert_eq!(return_line(&r, "23"), tax * 100);
    }
}

#[test]
fn able_work_allowance_cannot_shelter_others_contributions_or_exceed_compensation() {
    let mut d = complete(FederalStatus::Single);
    let mut entry = able_contributions(0, 25000, 40000);
    entry.work = AbleWork::Working {
        compensation: 300000,
        residence: MarketplaceResidence::Contiguous,
        no_plan: true,
    };
    d.screening.as_mut().unwrap().able_contributions = vec![entry];
    assert_eq!(value(&supported(&d), "able_taxpayer_excess_contributions"), 5000);
    let entry = &mut d.screening.as_mut().unwrap().able_contributions[0];
    entry.own = 800000;
    entry.other = 2000000;
    let r = supported(&d);
    assert_eq!(value(&r, "able_taxpayer_contribution_limit"), 23000);
    assert_eq!(value(&r, "able_taxpayer_excess_contributions"), 5000);
}

#[test]
fn able_excess_rejects_a_nonjoint_spouse_and_an_invalid_value() {
    let mut d = complete(FederalStatus::Single);
    let mut entry = able_contributions(0, 25000, 30000);
    entry.owner = Owner::Spouse;
    d.screening.as_mut().unwrap().able_contributions = vec![entry];
    unsupported(&d, "ABLE contributions");
}

#[test]
fn able_following_year_expense_election_reduces_tax_without_a_second_deduction() {
    let mut d = complete(FederalStatus::Single);
    let mut entry = income(able_distribution(2400, 400, 1600));
    if let FederalIncome::Able { data } = &mut entry.income
        && let Some(AbleTreatment::Annual { following_expenses, .. }) = &mut data.treatment
    {
        *following_expenses = 80000;
    }
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(value(&r, "able_taxable_distributions"), 0);
    assert_eq!(value(&r, "account_additional_tax"), 0);
    assert_eq!(value(&r, "agi"), 60000);
    assert!(
        r.review_notes
            .iter()
            .any(|note| note.contains("do not count the elected expenses again in 2027"))
    );
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn student_discharge_is_exempt_and_ordinary_2026_debt_remains_taxable() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        canceled_personal_debt(
            30000,
            PersonalDebtTreatment::StudentDischarge {
                reason: StudentDischargeReason::PublicService,
            },
        ),
    );
    let before = supported(&complete(FederalStatus::Single));
    // The page's discharge choice states the completed qualifying discharge.
    assert_eq!(value(&supported(&d), "agi"), value(&before, "agi"));
    if let FederalIncome::PersonalDebt { data } = &mut d.income[1].income {
        data.treatment = Some(CanceledDebtChoice::Taxable);
    }
    assert_eq!(value(&supported(&d), "agi"), value(&before, "agi") + 30000);
}

#[test]
fn household_disability_combines_regular_wages_before_social_and_medicare_caps() {
    let mut e = household_employer(180000);
    e.workers[0].disability = Some(household_disability(10000, 279, 145));
    let r = supported(&with_household(e.clone()));
    assert_eq!(value(&r, "household_0_social_tax"), 22878);
    assert_eq!(value(&r, "household_0_medicare_tax"), 5510);
    assert_eq!(value(&r, "household_0_state_disability_tax_offset"), 424);
    assert_eq!(r.tax, Some(3302900));
    e.workers[0].pay[0].cash = 19500000;
    e.workers[0].disability = Some(household_disability(10000, 0, 190));
    let r = supported(&with_household(e));
    // $45 Additional Medicare paid by the state is included in its $190 deposit.
    assert_eq!(value(&r, "household_0_additional_medicare"), 45);
    assert_eq!(value(&r, "household_0_state_disability_tax_offset"), 190);
    assert_eq!(value(&r, "additional_medicare_tax"), 0);
    assert_eq!(return_line(&r, "25d"), 650000);
    assert_eq!(r.tax, Some(3374300));
}

#[test]
fn household_disability_medicare_rounding_can_include_two_withheld_components() {
    let mut e = household_employer(210000);
    let mut payment = household_pay(0, 20260630);
    payment.cash = 61;
    e.workers[0].disability = Some(HouseholdDisability {
        wages: HouseholdSickWages::Reported { pay: vec![payment; 40] },
        social_withheld: 0,
        medicare_withheld: 80,
        complete: true,
    });
    // Each payment has one cent ordinary and one cent Additional Medicare.
    let r = supported(&with_household(e));
    assert_eq!(value(&r, "household_0_state_disability_tax_offset"), 1);
    assert_eq!(value(&r, "household_0_medicare_tax"), 6091);
    assert_eq!(value(&r, "household_0_additional_medicare"), 90);
    assert_eq!(r.tax, Some(3412300));
}

#[test]
fn historical_childless_eic_matches_every_published_table_interval() {
    for (year, csv, count) in [
        (2023, include_str!("../../../tests/fixtures/irs-childless-eic-2023.csv"), 485),
        (2024, include_str!("../../../tests/fixtures/irs-childless-eic-2024.csv"), 511),
        (2025, include_str!("../../../tests/fixtures/irs-childless-eic-2025.csv"), 525),
    ] {
        let mut previous = 1;
        let mut seen = 0;
        for row in csv.lines() {
            let v: alloc::vec::Vec<i64> = row.split(',').map(|v| v.parse().unwrap()).collect();
            assert_eq!(v.len(), 4);
            assert_eq!(v[0], previous);
            previous = v[1];
            seen += 1;
            for joint in [false, true] {
                let cutoff = crate::historical_eic::cutoff(year, joint).unwrap();
                for income in [v[0], (v[0] + v[1]) / 2, v[1] - 1] {
                    let expected = if income >= cutoff { 0 } else { v[2 + usize::from(joint)] };
                    assert_eq!(
                        crate::historical_eic::table(year, income, joint),
                        Some(expected),
                        "year={year} income={income} joint={joint}"
                    );
                }
            }
        }
        assert_eq!(seen, count);
        assert_eq!(crate::historical_eic::table(year, 0, false), Some(0));
    }
    // Published final partial-row footnotes and a 2023 rounding difference.
    for (year, joint, cutoff, credit) in [
        (2023, false, 17640, 2),
        (2023, true, 24210, 0),
        (2024, false, 18591, 2),
        (2024, true, 25511, 0),
        (2025, false, 19104, 0),
        (2025, true, 26214, 1),
    ] {
        assert_eq!(crate::historical_eic::table(year, cutoff - 1, joint), Some(credit));
        assert_eq!(crate::historical_eic::table(year, cutoff, joint), Some(0));
    }
    assert_eq!(crate::historical_eic::table(2023, 9950, false), Some(586));
    assert_eq!(crate::historical_eic::table(2026, 10000, false), None);
}

#[test]
fn full_trump_to_able_transfer_at_seventeen_is_not_income_or_able_contribution() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20090704;
    let baseline = supported(&d);
    add(
        &mut d,
        income(FederalIncome::TrumpAbleRollover {
            data: trump_able_source(TrumpAbleRollover {
                form_boxes: Default::default(),
                box_2a: 0,
                box_3: 0,
                box_6: 0,
                box_8a: 0,
                box_8b_percent: None,
                box_9a_percent: None,
                box_9b: None,
                box_10: 0,
                box7: crate::DistributionCodes::CODE_G.0,
                ira: false,
                trump: true,
                withholding: 0,
                amount: 200_000,
                transferred: 20260801,
            }),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "trump_able_rollover_2_gross"), 2_000);
    assert_eq!(value(&r, "agi"), value(&baseline, "agi"));
    assert_eq!(r.tax, baseline.tax);
    assert_eq!(estimate(&restart(d.clone()).draft), r);

    d.people.as_mut().unwrap().taxpayer.birth_date = 20100704;
    unsupported(&d, "Synthetic document");
    // The calendar year the beneficiary turns 17, day-exact (Pub. 501): born
    // 2010-01-01 the 17th birthday is 2027-01-01, so 17 is attained on
    // 2026-12-31 and the transfer qualifies; born 2009-01-01 attained 17 on
    // 2025-12-31 (and 18 on 2026-12-31), so 2026 is not the year.
    d.people.as_mut().unwrap().taxpayer.birth_date = 20100101;
    assert_eq!(value(&supported(&d), "trump_able_rollover_2_gross"), 2_000);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20090101;
    unsupported(&d, "Synthetic document");
    d.people.as_mut().unwrap().taxpayer.birth_date = 20090704;
    d.screening.as_mut().unwrap().trump_accounts.push(TrumpAccount {
        owner: Owner::Taxpayer,
        contributions: 600_000,
        exempt: 0,
        returns: vec![],
        confirmed: true,
    });
    unsupported(&d, "Synthetic document");
}

#[test]
fn joint_taxable_adoption_and_care_benefits_keep_distinct_wage_lines() {
    let mut d = joint_employer_adoption_return();
    if let AdoptionBenefits::JointEmployees { payments, .. } =
        &mut d.credits.as_mut().unwrap().adoptions[0].benefits
    {
        payments[0].received = 1_400_000;
    }
    set_joint_adoption_w2(&mut d, 0, 6_000_000, 1_400_000);
    set_joint_adoption_exclusion(&mut d, 13_670, 4_000);

    add_child(&mut d);
    add_child(&mut d);
    d.dependents[0].set_care_expenses(800_000);
    let taxpayer_w2 = w2(&d, 1);
    let spouse_w2 = w2(&d, 3);
    care_w2(&mut d, taxpayer_w2, 500_000, 0, 0, 500_000);
    care_w2(&mut d, spouse_w2, 500_000, 0, 0, 500_000);
    d.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::Plans { maximum: 1_000_000 };
    document_care(&mut d);

    let r = supported(&d);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 17_670);
    assert_eq!(value(&r, "adoption_wage_adjustment"), 2_330);
    assert_eq!(value(&r, "care_benefits_excluded"), 7_500);
    assert_eq!(value(&r, "care_benefits_taxable"), 2_500);
    assert_eq!(value(&r, "care_earned_taxpayer"), 83_580);
    assert_eq!(value(&r, "care_earned_spouse"), 21_250);
    assert_eq!(return_line(&r, "1a"), 10_000_000);
    assert_eq!(return_line(&r, "1e"), 250_000);
    assert_eq!(return_line(&r, "1f"), 233_000);
    assert_eq!(return_line(&r, "25a"), 950_000);
    assert_eq!(value(&r, "agi"), 104_830);
    assert_eq!(supported(&restart(d).draft), r);
}

// Form 2441 instructions (2026 draft) "Qualifying Person(s)" and Part III
// lines 12-26; IRC 21(b)(1), 129(e)(1): the benefit side counts only a
// qualifying person's costs, the same total the credit side uses.
#[test]
fn care_benefit_sources_leave_out_non_qualifying_persons_costs() {
    for case in [SettledCareCase::W2, SettledCareCase::Grace, SettledCareCase::Mixed] {
        let mut control = benefit_return(FederalStatus::Single);
        match case {
            SettledCareCase::Grace => grace_only_benefits(&mut control),
            SettledCareCase::Mixed => grace_and_current_benefits(&mut control),
            _ => {}
        }
        let mut d = control.clone();
        // Age 16 in 2026 and capable of self-care: sibling costs stay out.
        d.dependents[1].birth_date = 20100101;
        d.dependents[1].set_care_expenses(300_000);
        let r = supported(&d);
        let c = supported(&control);
        assert!(noted(&r, "WARN_CARE_COSTS_LEFT_OUT"), "{r:?}");
        assert_eq!(value(&r, "care_benefits_excluded"), value(&c, "care_benefits_excluded"));
        if !matches!(case, SettledCareCase::Mixed) {
            assert!(!noted(&c, "WARN_CARE_COSTS_LEFT_OUT"), "{c:?}");
            assert_eq!(value(&r, "care_benefits_taxable"), value(&c, "care_benefits_taxable"));
            let excluded = if matches!(case, SettledCareCase::W2) { 5_000 } else { 2_000 };
            assert_eq!(value(&r, "care_benefits_excluded"), excluded);
            assert_eq!(value(&r, "care_benefits_taxable"), 0);
        }
        if matches!(case, SettledCareCase::W2) {
            assert_eq!(supported(&restart(d).draft), r);
        }
        let mut both = control.clone();
        both.dependents[1].set_care_expenses(300_000);
        assert_eq!(
            value(&supported(&both), "care_benefits_excluded"),
            value(&c, "care_benefits_excluded")
        );
    }
}
