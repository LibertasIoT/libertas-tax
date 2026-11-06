use crate::tests::cross::interview::{document, open_topic, r1099};
use crate::tests::*;

pub(in crate::tests) fn retirement_entry(
    owner: Owner,
    kind: RetirementKind,
    gross: i64,
    treatment: RetirementTreatment,
) -> FederalIncomeEntry {
    let mut native = native_retirement(kind, gross * 100, (RetirementTaxable::Payer, gross * 100));
    native.withholding = 12300;
    let simple = matches!(treatment, RetirementTreatment::Exception { early_simple: true, .. });
    native.ordinary_distribution = Some(treatment);
    let mut source = retirement_source(native);
    if let RetirementSource::R1099(source) = &mut source
        && let Some(code) = &source.code
        && !matches!(code, DistributionCode::Other)
    {
        source.form.box7 = box7_of(code);
        if simple {
            // A SIMPLE IRA's early distribution is printed as code S alone.
            source.form.box7 = DistributionCodes::CODE_S.0;
        }
        if kind == RetirementKind::Nonqualified
            && source.form.box7 & DistributionCodes::CODE_W.0 == 0
        {
            source.form.box7 |= DistributionCodes::CODE_D.0;
        }
    }
    let mut entry = income(FederalIncome::Retirement { data: source });
    entry.owner = crate::DocumentOwner::from(owner);
    entry
}
pub(in crate::tests) fn qcd(owner: Owner, date: u32, amount: i64, prior: i64) -> QualifiedCharity {
    QualifiedCharity {
        owner,
        payments: vec![QcdPayment {
            split_interest: false,
            inherited: None,
            date,
            amount: amount * 100,
        }],
        prior_deductions: prior * 100,
    }
}

pub(in crate::tests) fn prior_ira_basis(owner: Owner, prior: i64, year_end: i64) -> IraBasis {
    IraBasis {
        owner,
        prior: prior * 100,
        year_end: year_end * 100,
        adjustment: 0,
        paid_next_year: 0,
        reservist_repayments: vec![],
        special_repayments: vec![],
    }
}
pub(in crate::tests) fn basis_distribution(
    owner: Owner,
    gross: i64,
    treatment: RetirementTreatment,
) -> FederalIncomeEntry {
    let mut e = retirement_entry(owner, RetirementKind::Ira, gross, treatment);
    let FederalIncome::Retirement { data } = &mut e.income else { panic!() };
    let mut data = edit_retirement(data);
    data.taxable = Some(RetirementTaxable::IraBasis);
    drop(data);
    e
}

pub(in crate::tests) fn ira_contribution_distribution(wages: i64, covered: Answer) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
    let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
    let mut data = edit_wage(data);
    data.retirement_plan = covered == Answer::Yes;
    drop(data);
    add(&mut d, basis_distribution(Owner::Taxpayer, 10000, RetirementTreatment::Early));
    d.adjustments.as_mut().unwrap().ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 0, 40000)];
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, covered);
    d
}

pub(in crate::tests) fn conversion(amount: i64, early: bool) -> RetirementTreatment {
    RetirementTreatment::Conversion { amount: amount * 100, early }
}

pub(in crate::tests) fn rollover(
    amount: i64,
    destination: RolloverDestination,
) -> RetirementTreatment {
    RetirementTreatment::Rollover { destination, amount: amount * 100, early: true }
}

pub(in crate::tests) fn qcd_with_basis(
    gross: i64,
    gifts: i64,
    prior: i64,
    year_end: i64,
    prior_deductions: i64,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    add(&mut d, basis_distribution(Owner::Taxpayer, gross, RetirementTreatment::Ordinary));
    let a = d.adjustments.as_mut().unwrap();
    a.ira_basis = vec![prior_ira_basis(Owner::Taxpayer, prior, year_end)];
    a.qualified_charity = vec![qcd(Owner::Taxpayer, 20261215, gifts, prior_deductions)];
    d
}

// A code 3 employer disability pension whose payments are all before the plan's
// minimum retirement age: Form 1040 line 1h wages (the 1099-R merge, unit B6b).
pub(in crate::tests) fn disability_pension(amount: i64) -> FederalIncome {
    disability_pension_with(amount, DisabilityWages::All)
}
pub(in crate::tests) fn disability_pension_with(
    amount: i64,
    wages: DisabilityWages,
) -> FederalIncome {
    FederalIncome::Retirement {
        data: retirement_source(FederalRetirement {
            form_boxes: Default::default(),
            trump: false,
            box_2a: amount * 100,
            box_3: 0,
            box_6: 0,
            box_8a: 0,
            box_8b_percent: None,
            box_9a_percent: None,
            ira: (RetirementKind::Pension).ira()
                && (crate::DistributionCodes::CODE_3.0) & crate::DistributionCodes::CODE_N.0 == 0,
            box_9b: None,
            box_10: 0,
            box_10_basis: None,
            disability: DisabilityIncome::Qualifying { amount: amount * 100, eligible: true },
            kind: RetirementKind::Pension,
            disability_wages: wages,
            gross: amount * 100,
            taxable: Some(RetirementTaxable::Payer),
            withholding: 12000,
            box7: crate::DistributionCodes::CODE_3.0,
            ordinary_distribution: Some(RetirementTreatment::Exception {
                second: None,
                distributed: 20261231,
                facts: RetirementException::Disability { began: 20240301 },
                early_simple: false,
            }),
            disaster: crate::DisasterDistribution::No,
            after_death: None,
        }),
    }
}

pub(in crate::tests) fn education_withdrawal(
    education: i64,
    assistance: i64,
    half_time: bool,
) -> RetirementException {
    RetirementException::Education {
        education: education * 100,
        assistance: assistance * 100,
        room_board: 400000,
        allowance: 200000,
        school_housing: 300000,
        half_time,
        eligible: true,
    }
}

pub(in crate::tests) fn birth_withdrawal(
    owner: Owner,
    child: &str,
    amount: i64,
    prior: i64,
) -> FederalIncomeEntry {
    retirement_entry(
        owner,
        RetirementKind::Ira,
        amount,
        RetirementTreatment::Exception {
            second: None,
            distributed: 20260228,
            facts: RetirementException::BirthAdoption {
                child: child.into(),
                event: 20250301,
                prior_used: prior * 100,
                eligible: true,
            },
            early_simple: false,
        },
    )
}

pub(in crate::tests) fn emergency_withdrawal(
    balance: i64,
    amount: i64,
    history: EmergencyHistory,
) -> FederalIncomeEntry {
    retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Ira,
        amount,
        RetirementTreatment::Exception {
            second: None,
            distributed: 20260601,
            facts: RetirementException::Emergency {
                balance: balance * 100,
                history,
                eligible: true,
            },
            early_simple: false,
        },
    )
}

pub(in crate::tests) fn abuse_withdrawal_plan(
    plan: &str,
    balance: i64,
    amount: i64,
) -> FederalIncomeEntry {
    retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Ira,
        amount,
        RetirementTreatment::Exception {
            second: None,
            distributed: 20260228,
            facts: RetirementException::DomesticAbuse {
                plan: plan.into(),
                event: 20250301,
                balance: balance * 100,
                prior_used: 0,
                prior_plan_used: 0,
                eligible: true,
            },
            early_simple: false,
        },
    )
}

pub(in crate::tests) fn abuse_withdrawal(balance: i64, amount: i64) -> FederalIncomeEntry {
    abuse_withdrawal_plan("Plan A", balance, amount)
}

pub(in crate::tests) fn pension_basis() -> PensionBasis {
    PensionBasis {
        earlier_months: 0,
        changes: vec![],
        share: None,
        started: 20260101,
        cost: Some(3100000),
        months: 12,
        history: PensionHistory::New {
            duration: PensionDuration::Joint { age: 65, survivor_age: 65 },
        },
        eligible: true,
    }
}
pub(in crate::tests) fn pension_entry(data: PensionBasis, gross: i64) -> FederalIncomeEntry {
    let mut entry = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Pension,
        gross,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data: v } = &mut entry.income {
        let mut v = edit_retirement(v);
        v.taxable = Some(RetirementTaxable::Simplified { data });
        drop(v);
    }
    entry
}

pub(in crate::tests) fn railroad_pension(
    contributory: i64,
    gross: i64,
    basis: Option<PensionBasis>,
) -> FederalIncomeEntry {
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Pension,
        gross,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut e.income {
        let mut data = edit_retirement(data);
        data.taxable = Some(RetirementTaxable::Railroad {
            repaid: 0,
            repayments: vec![],
            contributory: contributory * 100,
            box_5: (gross - contributory) * 100,
            box_6: 0,
            holder: Some(crate::RailroadHolder::Employee),
            basis,
        });
        drop(data);
    }
    e
}

pub(in crate::tests) fn fixed_annuity() -> GeneralAnnuity {
    GeneralAnnuity {
        frequency: AnnuityFrequency::Monthly,
        earlier_payments: 0,
        qualified_age: None,
        started: 20260101,
        cost: 3000000,
        increase: 0,
        periodic: 25000,
        duration: AnnuityDuration::Fixed { payments: 240 },
        received: 12,
        partial: 0,
        recovered: 0,
        eligible: true,
    }
}

pub(in crate::tests) fn joint_annuity(survivor: bool) -> GeneralAnnuity {
    GeneralAnnuity {
        frequency: AnnuityFrequency::Monthly,
        earlier_payments: 0,
        qualified_age: None,
        started: 20250101,
        cost: 6271200,
        increase: 0,
        periodic: 50000,
        duration: AnnuityDuration::JointLife {
            delay: 1,
            age: 70,
            survivor_age: 67,
            survivor_payment: 35000,
            survivor,
        },
        received: 12,
        partial: 0,
        recovered: 310200,
        eligible: true,
    }
}

pub(in crate::tests) fn variable_annuity() -> VariableAnnuity {
    VariableAnnuity {
        term: None,
        earlier: None,
        frequency: AnnuityFrequency::Annual,
        payments: 1,
        started: 20260101,
        age: 65,
        period: 20260101,
        cost: 1200000,
        history: VariableHistory::New { expected: VariablePayments::Life { age: 65, delay: 6 } },
        eligible: true,
    }
}

pub(in crate::tests) fn public_safety(
    d: &mut FederalDraft,
    id: i64,
    premiums: i64,
    funded: i64,
    coverage: PublicSafetyCoverage,
) {
    d.screening.as_mut().unwrap().public_safety.push(PublicSafetyPremium {
        income_id: id,
        premiums: premiums * 100,
        funded: funded * 100,
        coverage,
    });
}

pub(in crate::tests) fn old_general_annuity(started: u32, recovered: i64) -> GeneralAnnuity {
    GeneralAnnuity {
        frequency: AnnuityFrequency::Monthly,
        earlier_payments: 0,
        qualified_age: None,
        started,
        cost: 1000000,
        periodic: 100000,
        duration: AnnuityDuration::SingleLife { age: 65, delay: 1, guaranteed: 0 },
        received: 12,
        partial: 0,
        increase: 0,
        recovered,
        eligible: true,
    }
}
pub(in crate::tests) fn old_general_entry(
    kind: RetirementKind,
    data: GeneralAnnuity,
) -> FederalIncomeEntry {
    let mut e = retirement_entry(Owner::Taxpayer, kind, 12000, RetirementTreatment::Ordinary);
    let FederalIncome::Retirement { data: v } = &mut e.income else { unreachable!() };
    let mut v = edit_retirement(v);
    v.taxable = Some(RetirementTaxable::GeneralAnnuity { data });
    drop(v);
    e
}

pub(in crate::tests) fn qualified_fixed_pension(
    age: Option<u32>,
    payments: u32,
) -> FederalIncomeEntry {
    let mut data = fixed_annuity();
    data.cost = 600000;
    data.qualified_age = age;
    data.duration = AnnuityDuration::Fixed { payments };
    let mut entry = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Pension,
        3000,
        RetirementTreatment::Ordinary,
    );
    let FederalIncome::Retirement { data: v } = &mut entry.income else { unreachable!() };
    edit_retirement(v).taxable = Some(RetirementTaxable::GeneralAnnuity { data });
    entry
}

pub(in crate::tests) fn nonqualified_exception(facts: RetirementException) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 21000000, 0);
    let mut entry = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        10000,
        RetirementTreatment::Exception {
            second: None,
            distributed: 20260615,
            facts,
            early_simple: false,
        },
    );
    if let FederalIncome::Retirement { data } = &mut entry.income {
        let mut data = edit_retirement(data);
        data.box_2a = 500000;
        data.taxable = Some(RetirementTaxable::Payer);
        drop(data);
    }
    add(&mut d, entry);
    d
}

pub(in crate::tests) fn immediate_annuity(purchased: u32, started: u32) -> RetirementException {
    RetirementException::ImmediateAnnuity {
        purchased,
        started,
        single_premium: true,
        periodic: true,
    }
}

pub(in crate::tests) fn deferred_withdrawal(value: i64, investment: i64) -> FederalDraft {
    deferred_withdrawal_group(value * 100, investment * 100, vec![], false)
}

pub(in crate::tests) fn deferred_withdrawal_group(
    value: i64,
    investment: i64,
    related: Vec<AnnuityBalance>,
    loan_pledge: bool,
) -> FederalDraft {
    let mut d = nonqualified_exception(RetirementException::Disability { began: 20250101 });
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    let mut data = edit_retirement(data);
    data.gross = 700000;
    data.taxable = Some(RetirementTaxable::DeferredWithdrawal {
        data: crate::AnnuityDistribution {
            loan_pledge,
            related,
            value,
            investment,
            eligible: true,
        },
    });
    data.ordinary_distribution = Some(RetirementTreatment::Early);
    drop(data);
    d
}

pub(in crate::tests) fn qualified_withdrawal(
    gross: i64,
    value: i64,
    investment: i64,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 21000000, 0);
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Pension,
        gross,
        RetirementTreatment::Early,
    );
    let FederalIncome::Retirement { data } = &mut e.income else { panic!() };
    let mut data = edit_retirement(data);
    data.taxable = Some(RetirementTaxable::QualifiedWithdrawal {
        data: crate::PlanDistribution {
            value: value * 100,
            investment: investment * 100,
            eligible: true,
        },
    });
    drop(data);
    add(&mut d, e);
    d
}

pub(in crate::tests) fn incoming_plan_rollover() -> FederalDraft {
    let mut d = qualified_withdrawal(5000, 12500, 10000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    let mut data = edit_retirement(data);
    data.ordinary_distribution = Some(RetirementTreatment::Rollover {
        destination: RolloverDestination::Ira,
        amount: 500000,
        early: false,
    });
    drop(data);
    d.adjustments.as_mut().unwrap().ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 0, 15000)];
    d
}

pub(in crate::tests) fn contract_discharge(
    kind: RetirementKind,
    gross: i64,
    investment: i64,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 21000000, 0);
    let mut e = retirement_entry(Owner::Taxpayer, kind, gross, RetirementTreatment::Early);
    let FederalIncome::Retirement { data } = &mut e.income else { panic!() };
    let mut data = edit_retirement(data);
    data.taxable = Some(RetirementTaxable::FullDischarge {
        data: crate::ContractDischarge { investment: investment * 100, eligible: true },
    });
    drop(data);
    add(&mut d, e);
    d
}

pub(in crate::tests) fn ira_basis_exception_return(
    facts: RetirementException,
    simple: bool,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 12_000_000, 0);
    add(
        &mut d,
        basis_distribution(
            Owner::Taxpayer,
            20000,
            RetirementTreatment::Exception {
                second: None,
                distributed: 20260601,
                facts,
                early_simple: simple,
            },
        ),
    );
    d.adjustments.as_mut().unwrap().ira_basis =
        vec![prior_ira_basis(Owner::Taxpayer, 20000, 80000)];
    d
}

pub(in crate::tests) fn gross_home_claim(prior: i64) -> RetirementException {
    RetirementException::FirstHome {
        costs: 1000000,
        paid_on: 20260630,
        prior_used: prior * 100,
        eligible: true,
    }
}
pub(in crate::tests) fn gross_birth_claim(prior: i64) -> RetirementException {
    RetirementException::BirthAdoption {
        child: "Synthetic child".into(),
        event: 20251201,
        prior_used: prior * 100,
        eligible: true,
    }
}

pub(in crate::tests) fn plan_gross_exception(
    facts: RetirementException,
    investment: i64,
) -> FederalDraft {
    let mut d = qualified_withdrawal(20000, 100000, investment);
    d.income[0] = wage(1, Owner::Taxpayer, 12000000, 0);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    let mut data = edit_retirement(data);
    data.ordinary_distribution = Some(RetirementTreatment::Exception {
        second: None,
        distributed: 20260601,
        facts,
        early_simple: false,
    });
    drop(data);
    d
}

pub(in crate::tests) fn reservist_repayment(amount: i64) -> ReservistRepayment {
    ReservistRepayment {
        ordered: 20240101,
        term: ReserveTerm::Days { days: 366 },
        released: 20241231,
        distributed: 20240601,
        gross: 3000000,
        prior: 0,
        payments: vec![ReservistPayment { date: 20260115, amount: amount * 100 }],
    }
}
pub(in crate::tests) fn reservist_basis_return(amount: i64, year_end: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 12000000, 0);
    let mut b = prior_ira_basis(Owner::Taxpayer, 0, year_end);
    b.reservist_repayments.push(reservist_repayment(amount));
    d.adjustments.as_mut().unwrap().ira_basis.push(b);
    d
}

pub(in crate::tests) fn special_ira_repayment(income: i64, amount: i64, date: u32) -> IraRepayment {
    IraRepayment {
        income,
        payments: vec![IraRepaymentDeposit { date, amount: amount * 100 }],
        confirmed: true,
    }
}
pub(in crate::tests) fn special_ira_repayment_example(repaid: i64, date: u32) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 12000000, 0);
    add(&mut d, basis_distribution(Owner::Taxpayer, 7500, RetirementTreatment::Early));
    add(
        &mut d,
        basis_distribution(
            Owner::Taxpayer,
            5000,
            RetirementTreatment::Exception {
                second: None,
                distributed: 20260601,
                facts: gross_birth_claim(0),
                early_simple: false,
            },
        ),
    );
    let mut b =
        prior_ira_basis(Owner::Taxpayer, 2500, if date / 10000 == 2026 { repaid } else { 0 });
    b.special_repayments.push(special_ira_repayment(3, repaid, date));
    d.adjustments.as_mut().unwrap().ira_basis.push(b);
    d
}

pub(in crate::tests) fn care_return(facts: RetirementException, investment: i64) -> FederalDraft {
    let mut d = plan_gross_exception(facts, investment);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    let mut data = edit_retirement(data);
    // Payer source already incorporates the special pretax-first allocation.
    data.box_2a = 1600000;
    data.taxable = Some(RetirementTaxable::Payer);
    drop(data);
    d
}

pub(in crate::tests) fn care_claim(
    premiums: i64,
    balance: i64,
    spouse: bool,
) -> RetirementException {
    RetirementException::LongTermCare {
        data: CareDistribution {
            plan: "Synthetic employer plan".into(),
            balance: balance * 100,
            spouse_coverage: spouse,
            premiums: premiums * 100,
            statement_filed: 20260115,
            confirmed: true,
        },
    }
}

pub(in crate::tests) fn automatic_enrollment_income(
    owner: Owner,
    gross: i64,
    prior_year: bool,
) -> FederalIncomeEntry {
    retirement_entry(
        owner,
        RetirementKind::Pension,
        gross,
        RetirementTreatment::Exception {
            second: None,
            distributed: 20260415,
            facts: RetirementException::AutomaticEnrollment {
                first_contribution: if prior_year { 20251201 } else { 20260101 },
                elected: if prior_year { 20260115 } else { 20260301 },
                window_days: 90,
                eligible: true,
            },
            early_simple: false,
        },
    )
}

pub(in crate::tests) fn pension_contribution(
    owner: Owner,
    amount: i64,
    compensation: i64,
    other: i64,
) -> PensionContributions {
    PensionContributions {
        owner,
        contributions: amount * 100,
        compensation: compensation * 100,
        other_deferrals: other * 100,
        confirmed: true,
    }
}

pub(in crate::tests) fn railroad_repayment_draft() -> FederalDraft {
    let mut d = repayment_social_draft(2023, 15700, 1199);
    d.income.pop(); // Replace the unemployment repayment source.
    let mut e = railroad_pension(0, 0, None);
    let FederalIncome::Retirement { data } = &mut e.income else { unreachable!() };
    let mut data = edit_retirement(data);
    let Some(RetirementTaxable::Railroad { repayments, repaid, .. }) = &mut data.taxable else {
        unreachable!()
    };
    *repaid = 500000;
    repayments.push(RailroadRepayment { received_year: 2023, amount: 500000, contributory: false });
    drop(data);
    add(&mut d, e);
    let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment else {
        unreachable!()
    };
    years[0].wages = 0;
    years[0].benefits = 1000000;
    years[0].retirement = vec![RepaymentRetirement::Railroad {
        owner: Owner::Taxpayer,
        gross: 1500000,
        contributory: 0,
        basis: None,
        normal: true,
    }];
    d
}

pub(in crate::tests) fn child_annuity_draft() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20050101;
    let mut entry = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Pension,
        1500,
        RetirementTreatment::Exception {
            second: None,
            distributed: 20261201,
            facts: RetirementException::Death { died: 20241231 },
            early_simple: false,
        },
    );
    let FederalIncome::Retirement { data } = &mut entry.income else { unreachable!() };
    let mut data = edit_retirement(data);
    data.taxable = Some(RetirementTaxable::ChildAnnuity {
        data: ChildAnnuity {
            ended: None,
            started: 20250101,
            term: 24,
            cost: 240000,
            recovered: 60000,
            deferred: 60000,
            payments: (0..12)
                .map(|i| ChildAnnuityPayment { amount: if i < 6 { 5000 } else { 20000 } })
                .collect(),
            confirmed: true,
        },
    });
    drop(data);
    add(&mut d, entry);
    d
}

// The box 7a codes as printed, such as "4G".
pub(in crate::tests) fn box7(printed: &str) -> u32 {
    printed.chars().map(DistributionCodes::of).sum()
}

pub(in crate::tests) fn retirement_question_catalog(
    session: &Interview,
    box7: u32,
    ira: bool,
    trump: bool,
    box_10: i64,
) -> (Vec<i32>, Vec<i32>, crate::RCalls) {
    use crate::tests::support::client::income_pages::save_documents;
    let mut cold = restart(session.draft.clone());
    let row = R1099 {
        payer: Default::default(),
        payer_telephone: alloc::string::String::new(),
        account_number: alloc::string::String::new(),
        recipient: Default::default(),
        taxable_not_determined: true,
        total_distribution: false,
        employee_contributions: Some(0),
        excess_contribution_earnings: Some(0),
        state_details: Default::default(),
        gross: 1_000_000,
        taxable: Some(0),
        withholding: Some(0),
        box7,
        ira,
        trump,
        box_3: Some(0),
        box_6: Some(0),
        box_8a: Some(0),
        box_8b_percent: None,
        box_9a_percent: None,
        box_9b: None,
        box_10: Some(box_10),
        first_roth_year: 0,
        fatca: false,
        payment_date: None,
    };
    let expected_methods = DistributionCodes::methods(&row);
    let request = save_documents(
        &mut cold,
        vec![document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_paper(row)),
            },
        )],
    );
    accepted(&send(&mut cold, request));
    let page = open_topic(&mut cold, TaxSection::Retirement);
    let P::SaveIncomes {
        followups,
        source_documents: Some(crate::TopicDocumentViews::Retirement { documents }),
        ..
    } = page
    else {
        unreachable!()
    };
    let treatment = followups
        .iter()
        .find(|row| matches!(row.answer, crate::DocumentAnswer::RetirementTreatment { .. }))
        .unwrap()
        .allowed_values
        .clone();
    let method = followups
        .iter()
        .find(|row| matches!(row.answer, crate::DocumentAnswer::RetirementMethod { .. }))
        .map_or_else(
            || {
                assert_eq!(expected_methods, vec![0], "only the settled method may be omitted");
                expected_methods
            },
            |row| row.allowed_values.clone(),
        );
    let crate::RetirementDocument::R1099(row) = &documents[0].value.document else {
        unreachable!()
    };
    (
        treatment,
        method,
        crate::RCalls::for_boxes(row.box7, row.ira, row.trump, row.box_10.unwrap_or(0)),
    )
}

pub(in crate::tests) fn retirement_followup_path(
    page: &P,
    source: i64,
    local: &str,
) -> alloc::string::String {
    let P::SaveIncomes { followups, .. } = page else { unreachable!() };
    let (field, prefix, target) = [
        // 2026-10-27: question_field maps box 3 to RetirementTreatment's `code`.
        ("treatment", "more/box_3", "code"),
        ("treatment", "more/box_6", "code"),
        ("treatment", "more/box_8a", "code"),
        ("treatment", "more/box_8b_percent", "code"),
        ("treatment", "more/box_9a_percent", "code"),
        ("treatment", "more/box_9b", "code"),
        ("treatment", "more/box_10", "code"),
        ("treatment", "code", "code"),
        ("method", "more/taxable_method", "method"),
        ("basis", "more/box_10_basis", "basis"),
        ("disability", "more/disability", "disability"),
        ("disaster", "disaster", "value"),
        ("public_safety", "public_safety", "value"),
        ("after_death", "after_death", "value"),
    ]
    .into_iter()
    .find(|(_, prefix, _)| local == *prefix || local.starts_with(&alloc::format!("{prefix}/")))
    .unwrap_or(("treatment", "", ""));
    let key = alloc::format!("income/{source}/{field}");
    let row = followups.iter().find(|row| row.key == key).unwrap();
    let tail = local.strip_prefix(prefix).unwrap();
    let mapped = if prefix.is_empty() { local.into() } else { alloc::format!("{target}{tail}") };
    mapped
        .split('/')
        .fold(
            libertas::SchemaPath::root().field("followups").key(row.key.as_str()).field("answer"),
            |path, step| match step.parse::<usize>() {
                Ok(index) => path.index(index),
                Err(_) => path.field(step),
            },
        )
        .into()
}
// The 1099-R reprinted with these box 7a codes and the treatments they
// lead to, using the same owner as the topic catalog.
pub(in crate::tests) fn printed_as(mut r: R1099Answered, printed: &str) -> R1099Answered {
    r.source_mut().form.box7 = box7(printed);
    r
}
// The one box 7a code a calculated treatment branch is printed as.
pub(in crate::tests) fn box7_of(code: &DistributionCode) -> u32 {
    box7(match code {
        DistributionCode::Normal { .. } => "7",
        DistributionCode::Early { .. } => "1",
        DistributionCode::Disability { .. } => "3",
        DistributionCode::Death { .. } => "4",
        DistributionCode::DirectRollover { .. } => "G",
        DistributionCode::EarlySimple { .. } => "S",
        DistributionCode::Nonqualified => "D",
        DistributionCode::RothIra { roth: RothIraCode::Qualified, .. } => "Q",
        DistributionCode::RothIra { roth: RothIraCode::Early { .. }, .. } => "J",
        DistributionCode::RothIra { roth: RothIraCode::ExceptionApplies { .. }, .. } => "T",
        DistributionCode::NormalLumpSum { .. } => "7A",
        DistributionCode::DeathLumpSum { .. } => "4A",
        DistributionCode::Exchange => "6",
        DistributionCode::QualifiedCareCharge => "W",
        DistributionCode::Inherited { .. } | DistributionCode::TrumpDeath { .. } => "4",
        DistributionCode::TrumpToAble { .. } => "G",
        DistributionCode::Corrective { .. } => "8",
        DistributionCode::NotIncome => "N",
        DistributionCode::NoAdditionalTax => "9",
        DistributionCode::ProhibitedTransaction { .. } => "5",
        DistributionCode::EmployerRothContribution => "7",
        DistributionCode::Other => panic!("Another code names its printed codes"),
    })
}

pub(crate) fn public_safety_pensions(count: usize) -> Vec<IncomeDocumentItem> {
    (0..count)
        .map(|_| {
            let mut pension = r1099(
                100_000,
                DistributionCode::Normal { what_happened: Disposition::Kept },
                false,
            );
            // The ordinary source question affirms the pension is from the
            // officer's governmental employer after qualifying retirement. These
            // are two distinct actual coverage kinds, not duplicate premiums.
            pension.set_public_safety(vec![
                PublicSafetyPaid {
                    coverage: PublicSafetyPaidCoverage::Medical,
                    premiums: 100,
                    funded: 100,
                },
                PublicSafetyPaid {
                    coverage: PublicSafetyPaidCoverage::Other,
                    premiums: 100,
                    funded: 100,
                },
            ]);
            document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(pension)),
                },
            )
        })
        .collect()
}

// Native record-cap regressions retain real original distributions and dates.
pub(in crate::tests) fn reservist_capacity_return(records: usize, deposits: usize) -> FederalDraft {
    let mut d = reservist_basis_return(0, 0);
    let b = &mut d.adjustments.as_mut().unwrap().ira_basis[0];
    b.reservist_repayments = (0..records)
        .map(|index| {
            let mut source = reservist_repayment(0);
            source.distributed = crate::dates::add_days(20240601, index as i32);
            source.payments = (0..deposits)
                .map(|_| crate::ReservistPayment { date: 20260115, amount: 100 })
                .collect();
            source
        })
        .collect();
    b.year_end = (records * deposits) as i64 * 100;
    d
}
pub(in crate::tests) fn special_repayment_capacity_return() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    let mut b = prior_ira_basis(Owner::Taxpayer, 0, 1_010_000);
    for _ in 0..101 {
        let mut entry = basis_distribution(
            Owner::Taxpayer,
            100,
            RetirementTreatment::Exception {
                second: None,
                distributed: 20260601,
                facts: RetirementException::TerminalIllness { certified: 20260101 },
                early_simple: false,
            },
        );
        let FederalIncome::Retirement { data } = &mut entry.income else { unreachable!() };
        let mut data = edit_retirement(data);
        data.withholding = 0;
        data.box7 = crate::DistributionCodes::CODE_1.0;
        drop(data);
        let id = d.next_id;
        add(&mut d, entry);
        b.special_repayments.push(special_ira_repayment(id, 100, 20261201));
    }
    d.adjustments.as_mut().unwrap().ira_basis = vec![b];
    d
}

// Native source cents, including a separately supplied taxable treatment.
pub(in crate::tests) fn native_retirement(
    kind: RetirementKind,
    gross: i64,
    taxable: (RetirementTaxable, i64),
) -> FederalRetirement {
    FederalRetirement {
        form_boxes: Default::default(),
        trump: false,
        box_2a: taxable.1,
        box_3: 0,
        box_6: 0,
        box_8a: 0,
        box_8b_percent: None,
        box_9a_percent: None,
        ira: kind.ira(),
        kind,
        gross,
        taxable: Some(taxable.0),
        box_9b: None,
        box_10: 0,
        box_10_basis: None,
        disability: DisabilityIncome::None,
        disability_wages: crate::DisabilityWages::None,
        withholding: 0,
        box7: match kind {
            RetirementKind::Inherited => DistributionCodes::CODE_4.0,
            RetirementKind::Nonqualified => {
                DistributionCodes::CODE_7.0 | DistributionCodes::CODE_D.0
            }
            RetirementKind::Ira | RetirementKind::Pension => DistributionCodes::CODE_7.0,
        },
        ordinary_distribution: Some(RetirementTreatment::Ordinary),
        disaster: crate::DisasterDistribution::No,
        after_death: None,
    }
}

// Exception dates and facts remain native; gross follows retirement_entry's dollars.
pub(in crate::tests) fn retirement_exception(
    distributed: u32,
    facts: RetirementException,
    early_simple: bool,
) -> RetirementTreatment {
    RetirementTreatment::Exception { second: None, distributed, facts, early_simple }
}

pub(in crate::tests) fn retirement_exception_entry(
    owner: Owner,
    kind: RetirementKind,
    gross: i64,
    distributed: u32,
    facts: RetirementException,
    early_simple: bool,
) -> FederalIncomeEntry {
    retirement_entry(owner, kind, gross, retirement_exception(distributed, facts, early_simple))
}
