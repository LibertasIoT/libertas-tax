// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;
#[test]
fn employee_tips_and_overtime_change_taxable_income_not_payroll_or_agi() {
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    let FederalIncome::Wage { data: w } = &mut d.income[0].income else { panic!() };
    let mut w = edit_wage(w);
    w.social_wages = 4_000_000;
    w.social_tips = 2_000_000;
    w.work_deductions = work_deductions(2_000_000, 500_000);
    drop(w);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(value(&r, "tips_deduction"), 20000);
    assert_eq!(value(&r, "overtime_deduction"), 5000);
    assert_eq!(value(&r, "taxable_income"), 18900);
    assert_eq!(r.tax, Some(202_300));
    assert_eq!(r.payments, baseline.payments);
    let FederalIncome::Wage { data: w } = &mut d.income[0].income else { panic!() };
    let mut w = edit_wage(w);
    w.work_deductions.set_tips_eligible(false);
    drop(w);
    // Tips that do not qualify take no deduction.
    let r = supported(&d);
    assert_eq!(value(&r, "tips_deduction"), 0);
    assert_eq!(value(&r, "overtime_deduction"), 5000);
}
#[test]
fn both_spouses_w2_one_grace_balance_cap_and_taxable_shares() {
    let mut d = joint_employer_with_one_grace_return();
    care_costs(&mut d, 1_200_000);
    care_box10(&mut d, 0, 500_000);
    care_box10(&mut d, 1, 500_000);
    let r = supported(&d);
    assert_eq!(value(&r, "care_form2441_line12"), 10_000);
    assert_eq!(value(&r, "care_form2441_line13"), 2_000);
    assert_eq!(value(&r, "care_form2441_line15"), 12_000);
    assert_eq!(value(&r, "care_form2441_line20"), 12_000);
    assert_eq!(value(&r, "care_benefits_excluded"), 7_500);
    assert_eq!(value(&r, "care_benefits_taxable"), 4_500);
    // Approved automatic attribution: received amounts are $7,000 / $5,000,
    // so the $4,500 taxable total is $2,625 / $1,875.
    assert_eq!(value(&r, "care_taxable_taxpayer"), 2_625);
    assert_eq!(value(&r, "care_taxable_spouse"), 1_875);
    assert_eq!(value(&r, "care_credit"), 0);
    assert_eq!(return_line(&r, "1a"), 14_000_000);
    assert_eq!(return_line(&r, "1e"), 450_000);
    assert_eq!(return_line(&r, "25a"), 900_000);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    // Two distinct plans, each with a $2,000 maximum. The grace balance
    // belongs to an existing plan and must not add another plan maximum.
    care_box10(&mut d, 0, 200_000);
    care_box10(&mut d, 1, 200_000);
    d.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::Plans { maximum: 400_000 };
    let limited = supported(&d);
    assert_eq!(value(&limited, "care_form2441_line15"), 6_000);
    assert_eq!(value(&limited, "care_form2441_line21"), 4_000);
    assert_eq!(value(&limited, "care_benefits_excluded"), 4_000);
    assert_eq!(value(&limited, "care_benefits_taxable"), 2_000);
    assert_eq!(return_line(&limited, "1e"), 200_000);
}

#[test]
fn iso_compensation_above_surtax_threshold_is_not_investment_or_medicare_income() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
    let baseline = supported(&d);
    add(&mut d, income(FederalIncome::IsoSale { data: disqualifying_iso_sale() }));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 350000);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
    assert_eq!(value(&r, "additional_medicare_tax"), value(&baseline, "additional_medicare_tax"));
    assert_eq!(value(&r, "self_employment_tax"), 0);
}

#[test]
fn employee_tip_additional_medicare_does_not_invent_withholding() {
    let draft = wage_with_employee_tips(210000, allocated_employee_tips(10000));
    let r = supported(&draft);
    assert_eq!(value(&r, "employee_tips_0_social_tax"), 0);
    assert_eq!(value(&r, "employee_tips_0_medicare_tax"), 145);
    assert_eq!(value(&r, "additional_medicare_tax"), 180);
    assert_eq!(return_line(&r, "25d"), 9000);
    assert_eq!(return_line(&r, "23"), 32500);
    let mut tips = allocated_employee_tips(10000);
    tips.coverage = TipCoverage::MedicareOnlyGovernment;
    let mut government = wage_with_employee_tips(60000, tips);
    if let FederalIncome::Wage { data } = &mut government.income[0].income {
        let mut data = edit_wage(data);
        data.social_wages = 0;
        data.social_tax = 0;
    }
    let r = supported(&government);
    assert_eq!(value(&r, "employee_tips_0_social_tax"), 0);
    assert_eq!(r.tax, Some(672100));
}

#[test]
fn employee_tips_enter_linked_archer_msa_compensation_limit() {
    let mut draft = wage_with_employee_tips(1000, allocated_employee_tips(1000));
    set_archer(
        &mut draft,
        vec![archer_contributor(Owner::Taxpayer, archer_self(3000), 1500)],
        ArcherSharing::Equal,
    );
    let r = supported(&draft);
    assert_eq!(value(&r, "archer_msa_deduction"), 1500);
    assert_eq!(value(&r, "agi"), 500);
    assert_eq!(return_line(&r, "1z"), 200000);
    if let FederalIncome::Wage { data } = &mut draft.income[0].income {
        let mut data = edit_wage(data);
        data.employee_tips = None;
    }
    unsupported(&draft, "Archer MSA");
}

#[test]
fn uncollected_medicare_reconciliation_preserves_real_additional_withholding() {
    for life in [false, true] {
        let r = supported(&uncollected_wage(210000, 0, 145, life));
        assert_eq!(value(&r, "additional_medicare_tax"), 90);
        assert_eq!(value(&r, "w2_uncollected_payroll_tax"), 145);
        assert_eq!(return_line(&r, "25d"), 9000);
        assert_eq!(return_line(&r, "23"), 23500);
    }
}
