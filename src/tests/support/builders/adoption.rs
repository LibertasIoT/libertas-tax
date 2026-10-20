use crate::tests::*;

pub(in crate::tests) fn adoption_child(
    kind: AdoptionKind,
    payments: &[(u32, i64)],
) -> AdoptionChild {
    AdoptionChild {
        label: "Synthetic adoption".into(),
        kind,
        expenses: payments
            .iter()
            .map(|&(year, dollars)| AdoptionExpense { year, amount: dollars * 100 })
            .collect(),
        prior: AdoptionPriorCredit { previous: 0, expenses: 0 },
        benefits: AdoptionBenefits::None,
        limits: AdoptionLimits::Full,
        home_over_half_year: false,
        eligible: true,
    }
}

pub(in crate::tests) fn employer_adoption(
    d: &mut FederalDraft,
    received: i64,
    prior: i64,
    special: bool,
) {
    let mut child = adoption_child(
        AdoptionKind::Domestic {
            finalized: AdoptionFinalization::Final { year: 2026 },
            special_needs: special,
        },
        &[],
    );
    child.benefits = AdoptionBenefits::Program {
        owner: Owner::Taxpayer,
        received: received * 100,
        history: if prior == 0 {
            AdoptionBenefitHistory::None
        } else {
            AdoptionBenefitHistory::Exclusion { used: prior * 100 }
        },
        ordinary: true,
    };
    d.credits.as_mut().unwrap().adoptions.push(child);
}

pub(in crate::tests) fn joint_employer_adoption_return() -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 650_000);
    add(&mut d, wage(2, Owner::Taxpayer, 2_000_000, 100_000));
    add(&mut d, wage(3, Owner::Spouse, 2_000_000, 200_000));
    for (entry, payment) in d.income.iter_mut().zip([600_000, 200_000, 400_000]) {
        let FederalIncome::Wage { data } = &mut entry.income else { unreachable!() };
        data.social_wages += payment;
        data.medicare_wages += payment;
        data.social_tax = data.social_wages * 62 / 1000;
        data.medicare_tax = data.medicare_wages * 145 / 10000;
    }
    let mut child = adoption_child(
        AdoptionKind::Domestic {
            finalized: AdoptionFinalization::Final { year: 2026 },
            special_needs: false,
        },
        &[(2026, 8000)],
    );
    child.benefits = AdoptionBenefits::JointEmployees {
        payments: vec![
            AdoptionEmployerPayment { owner: Owner::Taxpayer, wage: w2(&d, 1), received: 600_000 },
            AdoptionEmployerPayment { owner: Owner::Taxpayer, wage: w2(&d, 2), received: 200_000 },
            AdoptionEmployerPayment { owner: Owner::Spouse, wage: w2(&d, 3), received: 400_000 },
        ],
        allocation: JointAdoptionExclusion::Automatic,
        history: AdoptionBenefitHistory::None,
        ordinary: true,
    };
    d.credits.as_mut().unwrap().adoptions.push(child);
    d
}

pub(in crate::tests) fn set_joint_adoption_w2(
    d: &mut FederalDraft,
    entry: usize,
    wages: i64,
    code_t: i64,
) {
    let FederalIncome::Wage { data } = &mut d.income[entry].income else { unreachable!() };
    data.wages = wages;
    data.social_wages = (wages + code_t).min(18_450_000);
    data.medicare_wages = wages + code_t;
    data.social_tax = data.social_wages * 62 / 1000;
    data.medicare_tax =
        data.medicare_wages * 145 / 10000 + (data.medicare_wages - 20_000_000).max(0) * 9 / 1000;
}

pub(in crate::tests) fn set_joint_adoption_exclusion(
    d: &mut FederalDraft,
    taxpayer: i64,
    spouse: i64,
) {
    set_joint_adoption_exclusion_for_child(d, 0, taxpayer, spouse);
}

pub(in crate::tests) fn set_joint_adoption_exclusion_for_child(
    d: &mut FederalDraft,
    child_index: usize,
    taxpayer: i64,
    spouse: i64,
) {
    let AdoptionBenefits::JointEmployees { allocation, .. } =
        &mut d.credits.as_mut().unwrap().adoptions[child_index].benefits
    else {
        unreachable!()
    };
    *allocation = JointAdoptionExclusion::Attributed {
        taxpayer: taxpayer * 100,
        spouse: spouse * 100,
        confirmed: true,
    };
}

pub(in crate::tests) fn two_child_joint_adoption_return() -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 600_000);
    add(&mut d, wage(2, Owner::Spouse, 3_000_000, 300_000));
    add(&mut d, wage(3, Owner::Taxpayer, 2_000_000, 200_000));
    add(&mut d, wage(4, Owner::Spouse, 1_000_000, 100_000));
    for (index, (wages, code_t)) in
        [(6_000_000, 600_000), (3_000_000, 400_000), (2_000_000, 500_000), (1_000_000, 300_000)]
            .into_iter()
            .enumerate()
    {
        set_joint_adoption_w2(&mut d, index, wages, code_t);
    }
    let mut first = adoption_child(
        AdoptionKind::Domestic {
            finalized: AdoptionFinalization::Final { year: 2026 },
            special_needs: false,
        },
        &[(2026, 8000)],
    );
    first.label = "First child".into();
    first.benefits = AdoptionBenefits::JointEmployees {
        payments: vec![
            AdoptionEmployerPayment { owner: Owner::Taxpayer, wage: w2(&d, 1), received: 600_000 },
            AdoptionEmployerPayment { owner: Owner::Spouse, wage: w2(&d, 2), received: 400_000 },
        ],
        allocation: JointAdoptionExclusion::Automatic,
        history: AdoptionBenefitHistory::None,
        ordinary: true,
    };
    let mut second = adoption_child(
        AdoptionKind::Domestic {
            finalized: AdoptionFinalization::Final { year: 2026 },
            special_needs: false,
        },
        &[(2026, 3000)],
    );
    second.label = "Second child".into();
    second.benefits = AdoptionBenefits::JointEmployees {
        payments: vec![
            AdoptionEmployerPayment { owner: Owner::Taxpayer, wage: w2(&d, 3), received: 500_000 },
            AdoptionEmployerPayment { owner: Owner::Spouse, wage: w2(&d, 4), received: 300_000 },
        ],
        allocation: JointAdoptionExclusion::Automatic,
        history: AdoptionBenefitHistory::None,
        ordinary: true,
    };
    d.credits.as_mut().unwrap().adoptions = vec![first, second];
    d
}

pub(in crate::tests) fn ordinary_and_special_joint_adoption_return() -> FederalDraft {
    let mut d = two_child_joint_adoption_return();
    let special = &mut d.credits.as_mut().unwrap().adoptions[1];
    special.kind = AdoptionKind::Domestic {
        finalized: AdoptionFinalization::Final { year: 2026 },
        special_needs: true,
    };
    special.expenses.clear(); // Deemed special-needs credit is independent of reimbursed expenses.
    d
}

pub(in crate::tests) fn two_special_joint_adoption_return() -> FederalDraft {
    let mut d = ordinary_and_special_joint_adoption_return();
    let first = &mut d.credits.as_mut().unwrap().adoptions[0];
    first.kind = AdoptionKind::Domestic {
        finalized: AdoptionFinalization::Final { year: 2026 },
        special_needs: true,
    };
    first.expenses.clear();
    d
}

pub(in crate::tests) fn joint_special_needs_below_code_t_cap() -> FederalDraft {
    let mut d = joint_employer_adoption_return();
    let child = &mut d.credits.as_mut().unwrap().adoptions[0];
    child.kind = AdoptionKind::Domestic {
        finalized: AdoptionFinalization::Final { year: 2026 },
        special_needs: true,
    };
    child.expenses.clear(); // The special-needs credit is independent of reimbursements.
    d
}

pub(in crate::tests) fn separated_adoption_return() -> FederalDraft {
    let mut d = complete(FederalStatus::Separate);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::LastSixMonths };
    d.setup.as_mut().unwrap().paid_over_half_home = true;
    employer_adoption(&mut d, 20000, 0, false);
    let child = &mut d.credits.as_mut().unwrap().adoptions[0];
    child.home_over_half_year = true;
    child.expenses.push(AdoptionExpense { year: 2026, amount: 600000 });
    d
}

pub(in crate::tests) fn shared_adoption_return(special: bool) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    employer_adoption(&mut d, if special { 0 } else { 10000 }, 0, special);
    let child = &mut d.credits.as_mut().unwrap().adoptions[0];
    child.limits = AdoptionLimits::Shared { credit: 800000, exclusion: 500000, agreed: true };
    child.expenses.push(AdoptionExpense { year: 2026, amount: 1200000 });
    d
}

pub(in crate::tests) fn foreign_adoption_benefits(current: i64, earlier: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    employer_adoption(&mut d, current, 0, false);
    let child = &mut d.credits.as_mut().unwrap().adoptions[0];
    child.kind = AdoptionKind::Foreign { finalized: AdoptionFinalization::Final { year: 2026 } };
    if let AdoptionBenefits::Program { history, .. } = &mut child.benefits {
        *history = AdoptionBenefitHistory::TaxedForeign { received: earlier * 100 };
    }
    d
}

pub(in crate::tests) fn source_adoption_code_t(d: &mut FederalDraft, owner: Owner, cents: i64) {
    let index = d.income.iter().position(|entry| entry.owner == owner).unwrap();
    let FederalIncome::Wage { data } = &mut d.income[index].income else { panic!() };
    data.other_box12.push(W2CodeAmount { code: W2Code::T, amount: cents });
    let wages = data.wages;
    set_joint_adoption_w2(d, index, wages, cents);
}

pub(in crate::tests) fn joint_adoption_payments_mut(
    d: &mut FederalDraft,
    child_index: usize,
) -> &mut Vec<AdoptionEmployerPayment> {
    let AdoptionBenefits::JointEmployees { payments, .. } =
        &mut d.credits.as_mut().unwrap().adoptions[child_index].benefits
    else {
        unreachable!()
    };
    payments
}

pub(in crate::tests) fn joint_adoption_history_mut(
    d: &mut FederalDraft,
    child_index: usize,
) -> &mut AdoptionBenefitHistory {
    let AdoptionBenefits::JointEmployees { history, .. } =
        &mut d.credits.as_mut().unwrap().adoptions[child_index].benefits
    else {
        unreachable!()
    };
    history
}
