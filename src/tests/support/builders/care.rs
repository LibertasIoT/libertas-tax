use crate::tests::*;

// Ordinary Form 2441 fixture facts. Amounts here are cents; all numerical
// expectations remain in the individual source tests, independent of the App.
pub(in crate::tests) fn prior_care_facts() -> PriorYearCare {
    PriorYearCare {
        two_or_more: false,
        prior_line_3: 200000,
        paid_in_return_year: 60000,
        prior_benefits: 0,
        prior_earned_smaller: 4000000,
        prior_line_6: 200000,
        prior_agi: 4000000,
        injured_claimants: None,
    }
}
pub(in crate::tests) fn document_care(d: &mut FederalDraft) {
    let total = d.dependents().iter().map(|p| p.care_expenses()).sum::<i64>()
        + d.credits.as_ref().map_or(0, |c| c.care.filer_expenses());
    let c = &mut d.credits.as_mut().unwrap().care;
    if total > 0 && c.providers.is_empty() {
        c.providers.push(CareProvider {
            id: 0,
            name: "Synthetic care provider".into(),
            address: "1 Example Way".into(),
            identification: ProviderIdentification::Organization,
            household_employee: false,
            paid: total,
            relationship: ProviderRelationship::NotRelated,
        });
    }
    if d.setup.as_ref().unwrap().filing_status() == FederalStatus::Separate
        && !matches!(c.filing, CareFilingStatus::Separate { .. })
    {
        c.filing = CareFilingStatus::Separate { spouse_earned_income: None };
    }
}
pub(in crate::tests) fn care_w2(
    d: &mut FederalDraft,
    id: i64,
    box10: i64,
    in_box_1: i64,
    forfeited: i64,
    plan: i64,
) {
    let e = d.income.iter_mut().find(|e| e.id == id).unwrap();
    let FederalIncome::Wage { data } = &mut e.income else { panic!("W-2 fixture source") };
    let mut data = edit_wage(data);
    data.care_benefits = box10;
    drop(data);
    let c = &mut d.credits.as_mut().unwrap().care;
    c.plan_maximum = CarePlanMaximum::Plans { maximum: plan };
    c.benefits.retain(|b| b.wage != id);
    if box10 > 0 {
        c.benefits.push(CareBenefitAnswers {
            wage: id,
            in_box_1: Some(in_box_1),
            forfeited: Some(forfeited),
        });
    }
}
pub(in crate::tests) fn care_box10(d: &mut FederalDraft, index: usize, amount: i64) {
    let id = d.credits.as_ref().unwrap().care.benefits[index].wage;
    let FederalIncome::Wage { data } =
        &mut d.income.iter_mut().find(|e| e.id == id).unwrap().income
    else {
        panic!()
    };
    let mut data = edit_wage(data);
    data.care_benefits = amount;
    drop(data);
}
pub(in crate::tests) fn care_costs(d: &mut FederalDraft, amount: i64) {
    for (i, p) in d.dependents.iter_mut().enumerate() {
        p.set_care_expenses(if i == 0 { amount } else { 0 });
    }
    if let Some(p) = d.credits.as_mut().unwrap().care.providers.first_mut() {
        p.paid = amount;
    }
    document_care(d);
}
pub(in crate::tests) fn clear_care_benefits(d: &mut FederalDraft) {
    for e in &mut d.income {
        if let FederalIncome::Wage { data } = &mut e.income {
            let mut data = edit_wage(data);
            data.care_benefits = 0;
            drop(data);
        }
    }
    let c = &mut d.credits.as_mut().unwrap().care;
    c.benefits.clear();
    c.grace.clear();
    c.own_business.clear();
    c.plan_maximum = CarePlanMaximum::NoBenefits;
}
pub(in crate::tests) fn benefit_return(status: FederalStatus) -> FederalDraft {
    let mut d = complete(status);
    add_child(&mut d);
    add_child(&mut d);
    d.dependents[0].set_care_expenses(800000);
    document_care(&mut d);
    let id = w2(&d, 1);
    care_w2(&mut d, id, 500000, 0, 0, 750000);
    d
}
pub(in crate::tests) fn joint_employer_care_return() -> FederalDraft {
    let mut d = benefit_return(FederalStatus::Joint);
    add(&mut d, wage(2, Owner::Spouse, 4000000, 250000));
    let a = w2(&d, 1);
    let b = w2(&d, 2);
    care_w2(&mut d, a, 200000, 0, 0, 500000);
    care_w2(&mut d, b, 200000, 0, 0, 500000);
    d.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::Plans { maximum: 1_000_000 };
    d
}
pub(in crate::tests) fn joint_employer_with_one_grace_return() -> FederalDraft {
    let mut d = joint_employer_care_return();
    let a = w2(&d, 1);
    let b = w2(&d, 2);
    care_w2(&mut d, a, 100000, 0, 0, 500000);
    care_w2(&mut d, b, 100000, 0, 0, 500000);
    d.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::Plans { maximum: 750_000 };
    d.credits.as_mut().unwrap().care.grace =
        vec![CareGraceBalance { owner: Owner::Taxpayer, used_in_return_year: 200000 }];
    d
}
pub(in crate::tests) fn grace_only_benefits(d: &mut FederalDraft) {
    clear_care_benefits(d);
    d.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::Plans { maximum: 750_000 };
    d.credits.as_mut().unwrap().care.grace =
        vec![CareGraceBalance { owner: Owner::Taxpayer, used_in_return_year: 200000 }];
}
pub(in crate::tests) fn grace_and_current_benefits(d: &mut FederalDraft) {
    clear_care_benefits(d);
    let id = w2(d, 1);
    care_w2(d, id, 300000, 0, 0, 750000);
    d.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::Plans { maximum: 750_000 };
    d.credits.as_mut().unwrap().care.grace =
        vec![CareGraceBalance { owner: Owner::Taxpayer, used_in_return_year: 200000 }];
}
// These builders retain the historical input records solely for translating
// fixture facts. The accepted App stores a month list and aggregate earnings.
#[derive(Clone)]
pub(in crate::tests) struct CareMonth {
    pub(in crate::tests) owner: Owner,
    pub(in crate::tests) month: u32,
    pub(in crate::tests) wages: i64,
    pub(in crate::tests) foreign_pay: i64,
    pub(in crate::tests) business_net: i64,
    pub(in crate::tests) student: bool,
    pub(in crate::tests) incapable: bool,
}
pub(in crate::tests) fn care_months(owner: Owner, student_months: u32) -> Vec<CareMonth> {
    (1..=12)
        .map(|month| CareMonth {
            owner,
            month,
            wages: 0,
            foreign_pay: 0,
            business_net: 0,
            student: month <= student_months,
            incapable: false,
        })
        .collect()
}
pub(in crate::tests) fn set_care_months(d: &mut FederalDraft, months: Vec<CareMonth>) {
    let c = &mut d.credits.as_mut().unwrap().care;
    c.filers.clear();
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let selected: Vec<_> =
            months.iter().filter(|m| m.owner == owner && (m.student || m.incapable)).collect();
        if selected.is_empty() {
            continue;
        }
        let earned = selected.iter().map(|m| m.wages + m.foreign_pay + m.business_net).sum::<i64>();
        let keys = selected.iter().map(|m| i64::from(m.month)).collect();
        let situation = if selected.iter().any(|m| m.incapable) {
            CareFilerSituation::Incapable {
                months: keys,
                earned_in_months: Some(earned),
                same_home_over_half_year: true,
                care_paid: 0,
            }
        } else {
            CareFilerSituation::Student { months: keys, earned_in_months: Some(earned) }
        };
        c.filers.push(CareFiler { owner, situation });
    }
    document_care(d);
}

pub(in crate::tests) fn care_aggregate(d: &mut FederalDraft, owner: Owner, cents: i64) {
    let facts =
        d.credits.as_mut().unwrap().care.filers.iter_mut().find(|f| f.owner == owner).unwrap();
    match &mut facts.situation {
        CareFilerSituation::Student { earned_in_months, .. }
        | CareFilerSituation::Incapable { earned_in_months, .. } => *earned_in_months = Some(cents),
    }
}

#[derive(Clone, Copy)]
pub(in crate::tests) enum SettledCareCase {
    W2,
    Grace,
    Mixed,
    Forfeited,
}

pub(in crate::tests) fn care_providers(count: usize) -> Vec<CareProvider> {
    (0..count)
        .map(|index| CareProvider {
            id: 0,
            name: alloc::format!("Synthetic provider {index}"),
            address: "1 Example Way".into(),
            identification: ProviderIdentification::Organization,
            household_employee: false,
            paid: 100,
            relationship: ProviderRelationship::NotRelated,
        })
        .collect()
}
pub(in crate::tests) fn care_own_plans(count: usize) -> FederalDraft {
    let mut draft = super::complete(FederalStatus::Single);
    for _ in 0..count {
        super::add(&mut draft, super::income(super::cash_business(0)));
        let id = draft.income.last().unwrap().id;
        draft.credits.as_mut().unwrap().care.own_business.push(CareOwnBusinessBenefit {
            owner: Owner::Taxpayer,
            source: CareBenefitSource::Business { business: id },
            received: 100,
        });
    }
    draft.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::Plans { maximum: 750_000 };
    draft
}

pub(in crate::tests) fn section179_care_return() -> FederalDraft {
    section179_care_with_profit(1_000)
}

fn section179_care_with_profit(profit: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 100_000, 0);
    let mut business = cash_business(profit);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.assets.push(elected_asset(5_000, 5_000, BonusElection::ElectOut));
    add(&mut d, income(business));
    d = save_elected_business_asset(d, 2, 5_000, 5_000);
    add_child(&mut d);
    care_costs(&mut d, 500_000);
    care_w2(&mut d, 1, 500_000, 0, 0, 750_000);
    d
}

pub(in crate::tests) fn section179_care_employee_reimbursements(profit: i64) -> FederalDraft {
    let mut d = section179_care_with_profit(profit);
    d.adjustments.as_mut().unwrap().employee_expenses.push(EmployeeExpenseClaim {
        improvement_buildings: Vec::new(),
        owner: Owner::Taxpayer,
        occupation: "Synthetic fee-basis official".into(),
        capacity: EmployeeCapacity::FeeBasisOfficial,
        employers: vec![1],
        dot_hours_of_service: false,
        local_transportation: 100_000,
        travel: 0,
        other: 0,
        meals: 0,
        reimbursements: 200_000,
        mileage: vec![],
        vehicle_assets: vec![],
        equipment_assets: vec![],
        leased_vehicles: vec![],
    });
    d
}
