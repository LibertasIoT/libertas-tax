use crate::tests::cross::interview::{document, w2 as wage_document};
use crate::tests::*;
pub(in crate::tests) fn tip_fica_carryforward(amount: i64) -> EmployerTipFicaCarryforward {
    EmployerTipFicaCarryforward {
        prior_status: FederalStatus::Single,
        original_credit: amount + 1_000_000,
        used_origin_year: 500_000,
        used_origin_prior_year_carryback: 500_000,
        carryforward_to_current_year: amount,
        ordinary: true,
    }
}

pub(in crate::tests) fn restaurant_tip_credit_draft() -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    let mut months =
        vec![EmployerTipMonth { hours: 0.0, wages_excluding_tips: 0, reported_tips: 0 }; 12];
    // The October example in the draft 2026 Form 8846 instructions: 100
    // hours, $375 wages, $450 tips, and a $140 wage shortfall at $5.15/hour.
    months[9] =
        EmployerTipMonth { hours: 100.0, wages_excluding_tips: 37_500, reported_tips: 45_000 };
    add(
        &mut draft,
        income(FederalIncome::Business {
            data: FederalBusiness {
                interest_limit_election: None,
                activity: BusinessActivity::Ordinary {
                    returns: 0,
                    receipts: 10_000_000,
                    accounting: BusinessAccounting::Cash,
                },
                expenses: vec![ordinary_business_expense(
                    BusinessExpenseCategory::Other {
                        label: "Cash employee wages; tip FICA excluded".into(),
                    },
                    37_500,
                )],
                employer_tip_fica_credit: Some(EmployerTipFicaCredit {
                    service: EmployerTipService::FoodOrBeverage,
                    employees: vec![EmployerTipEmployee { label: "Server A".into(), months }],
                    // Four payroll periods with $50, $50, $50, and $300 of
                    // tips produce $34.44 employer FICA after period rounding,
                    // one cent above 7.65% of the annual $450 rounded once.
                    actual_employer_tip_fica_paid: 3_444,
                    used_prior_year_carryback: 0,
                    confirmed: true,
                }),
                payroll: BusinessPayroll::Yes {
                    statements: vec![PayrollW2 {
                        label: "Server A".into(),
                        wages: 82_500,
                        medicare: 82_500,
                    }],
                    attribution: crate::PayrollQbiAttribution::EntireBusiness,
                },
                ..support::builders::survey::survey_cash_business_data()
            },
        }),
    );
    draft
}

pub(in crate::tests) fn restaurant_business(d: &FederalDraft) -> &FederalBusiness {
    let FederalIncome::Business { data } = &d.income[1].income else {
        panic!("restaurant business missing")
    };
    data
}

pub(in crate::tests) fn restaurant_business_mut(d: &mut FederalDraft) -> &mut FederalBusiness {
    let FederalIncome::Business { data } = &mut d.income[1].income else {
        panic!("restaurant business missing")
    };
    data
}

pub(in crate::tests) fn beauty_tip_credit_draft() -> FederalDraft {
    let mut draft = restaurant_tip_credit_draft();
    let business = restaurant_business_mut(&mut draft);
    let claim = business.employer_tip_fica_credit.as_mut().unwrap();
    claim.service = EmployerTipService::BeautyServices {
        barbering_and_hair_care: true,
        nail_care: false,
        esthetics: false,
        body_and_spa_treatment: false,
        eligible_tips_only: true,
    };
    claim.employees[0].label = "Stylist A".into();
    let BusinessPayroll::Yes { statements, .. } = &mut business.payroll else { panic!() };
    statements[0].label = "Stylist A".into();
    draft
}

pub(in crate::tests) fn work_deductions(tips: i64, overtime: i64) -> FederalWorkDeductions {
    FederalWorkDeductions {
        tips_claim: TipsClaim::Yes { w2_tips: tips, form4137_tips: 0, tips_eligible: Some(true) },
        overtime_claim: OvertimeClaim::Yes { overtime, overtime_eligible: Some(true) },
    }
}

pub(in crate::tests) fn allocated_employee_tips(amount: i64) -> EmployeeTips {
    EmployeeTips {
        cash: CashTipRecords::Allocated { amount: amount * 100 },
        noncash: 0,
        coverage: TipCoverage::SocialAndMedicare,
    }
}
pub(in crate::tests) fn wage_with_employee_tips(wages: i64, tips: EmployeeTips) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
    if let FederalIncome::Wage { data } = &mut draft.income[0].income {
        let mut data = crate::tests::support::builders::edit_wage(data);
        data.employee_tips = Some(tips);
        drop(data);
    }
    draft
}

pub(in crate::tests) fn w2_uncollected(social: i64, medicare: i64, life: bool) -> W2UncollectedTax {
    W2UncollectedTax {
        tip_social: if life { 0 } else { social * 100 },
        tip_medicare: if life { 0 } else { medicare * 100 },
        life_social: if life { social * 100 } else { 0 },
        life_medicare: if life { medicare * 100 } else { 0 },
    }
}
pub(in crate::tests) fn uncollected_wage(
    wages: i64,
    social: i64,
    medicare: i64,
    life: bool,
) -> FederalDraft {
    let mut draft = wage_with_employee_tips(wages, allocated_employee_tips(0));
    if let FederalIncome::Wage { data } = &mut draft.income[0].income {
        let mut data = crate::tests::support::builders::edit_wage(data);
        data.employee_tips = None;
        data.uncollected_tax = Some(w2_uncollected(social, medicare, life));
        data.social_tax -= social * 100;
        data.medicare_tax -= medicare * 100;
        if !life {
            data.social_wages -= 1000000;
            data.social_tips = 1000000;
        }
        drop(data);
    }
    draft
}

pub(in crate::tests) fn omitted_employee_wages(amount: i64) -> FederalIncome {
    FederalIncome::EmployeeWages {
        data: EmployeeWages {
            source_issuer: Default::default(),
            source_recipient: Default::default(),
            wages: amount * 100,
            withholding: 0,
            form1099: true,
            payroll: crate::EmployeePayrollSource::Form8919 {
                reason: EmployeeWageReason::SameFirmForms,
            },
        },
    }
}

pub(in crate::tests) fn household_pay(amount: i64, date: u32) -> HouseholdPay {
    HouseholdPay {
        date,
        cash: amount * 100,
        treatment: HouseholdWageKind::Ordinary,
        state: "TX".into(),
        state_covered: true,
    }
}
pub(in crate::tests) fn household_employer(amount: i64) -> HouseholdEmployer {
    HouseholdEmployer {
        owner: Owner::Taxpayer,
        workers: vec![HouseholdWorker {
            disability: None,
            label: "Synthetic worker".into(),
            pay: vec![household_pay(amount, 20260331)],
            withholding: 0,
        }],
        prior_quarters: (1..=4).map(|quarter| HouseholdQuarter { quarter, cash: 0 }).collect(),
        states: vec![HouseholdState {
            state: "TX".into(),
            required: 0,
            periods: vec![HouseholdStatePeriod {
                starts: 20260101,
                ends: 20261231,
                wages: amount * 100,
                rate: Some(0.0),
            }],
            payments: vec![],
            reduction: Some(0.0),
        }],
        complete: true,
    }
}
pub(in crate::tests) fn with_household(record: HouseholdEmployer) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().household_employment = vec![record];
    d
}

pub(in crate::tests) fn household_disability(
    amount: i64,
    social: i64,
    medicare: i64,
) -> HouseholdDisability {
    HouseholdDisability {
        wages: HouseholdSickWages::Reported { pay: vec![household_pay(amount, 20260630)] },
        social_withheld: social * 100,
        medicare_withheld: medicare * 100,
        complete: true,
    }
}

pub(in crate::tests) fn reported_sick_pay(
    notice: &mut HouseholdDisability,
) -> &mut alloc::vec::Vec<HouseholdPay> {
    let HouseholdSickWages::Reported { pay } = &mut notice.wages else {
        panic!("expected reported notice")
    };
    pay
}

pub(in crate::tests) fn raw_sick_payment(
    amount: i64,
    paid: u32,
    worked: u32,
) -> HouseholdSickPayment {
    HouseholdSickPayment {
        payment: household_pay(amount, paid),
        last_worked: worked,
        disability: None,
        ended: None,
    }
}
pub(in crate::tests) fn sick_plan_employer(
    funding: HouseholdSickFunding,
    pay: alloc::vec::Vec<HouseholdSickPayment>,
    social: i64,
    medicare: i64,
) -> HouseholdEmployer {
    let mut e = household_employer(0);
    e.workers[0].pay.clear();
    e.states[0].periods[0].wages = pay.iter().map(|p| p.payment.cash).sum();
    e.workers[0].disability = Some(HouseholdDisability {
        wages: HouseholdSickWages::Plan { funding, pay },
        social_withheld: social,
        medicare_withheld: medicare,
        complete: true,
    });
    e
}

pub(in crate::tests) fn income_record_boundary_documents() -> Vec<IncomeDocumentItem> {
    let mut documents = wage_documents(100, 10_000_000, 0, 0);
    documents.extend(wage_documents(1, 1_000_000, 0, 0));
    documents
}

pub(in crate::tests) fn wage_documents(
    count: usize,
    wages: i64,
    withholding: i64,
    care: i64,
) -> Vec<IncomeDocumentItem> {
    (0..count)
        .map(|_| {
            let mut wage = wage_document(wages);
            wage.withholding = withholding;
            wage.social_wages = wages.min(crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS);
            wage.social_tax = wage.social_wages * 62 / 1000;
            wage.more.care_benefits = care;
            document(
                0,
                IncomeSource::Wage {
                    value: crate::WageDocument::W2(super::native_income::wage_document(wage)),
                },
            )
        })
        .collect()
}

pub(in crate::tests) fn source_medicaid(owner: Owner, amount: i64) -> FederalIncomeEntry {
    let mut entry = income(FederalIncome::OtherIncome {
        data: OtherIncomeItem {
            amount,
            withholding: 0,
            kind: OtherIncomeKind::MedicaidWaiver { in_your_home: true },
        },
    });
    entry.owner = crate::DocumentOwner::from(owner);
    entry
}

pub(in crate::tests) fn source_statutory_wage(
    d: &mut FederalDraft,
    owner: Owner,
    target: Option<i64>,
) {
    let FederalIncome::Wage { data } = &mut d
        .income
        .iter_mut()
        .find(|e| e.owner == crate::DocumentOwner::from(owner))
        .unwrap()
        .income
    else {
        panic!()
    };
    let mut data = crate::tests::support::builders::edit_wage(data);
    data.statutory_employee = true;
    data.statutory_business = target;
    drop(data);
}

pub(in crate::tests) fn statutory_storage_return(linked: bool) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(cash_business(0)));
    let business_id = draft.income[1].id;
    let FederalIncome::Wage { data } = &mut draft.income[0].income else { unreachable!() };
    let mut data = crate::tests::support::builders::edit_wage(data);
    data.wages = 5_000_000;
    data.medicaid_waiver = 100_000;
    drop(data);
    source_statutory_wage(&mut draft, Owner::Taxpayer, linked.then_some(business_id));
    draft
}

pub(in crate::tests) fn statutory_multiple_return() -> FederalDraft {
    let mut d = injured_return((60_000, 5000), (20_000, 500), 0, allocation());
    let first = source_expenses_business(&mut d, Owner::Taxpayer, 100);
    let second = source_expenses_business(&mut d, Owner::Spouse, 100);
    let third = source_expenses_business(&mut d, Owner::Taxpayer, 0);
    for (owner, amount, target) in [
        (Owner::Taxpayer, 100_050, Some(first)),
        (Owner::Taxpayer, 200_050, Some(first)),
        (Owner::Taxpayer, 100_000, Some(third)),
        (Owner::Spouse, 300_050, Some(second)),
        (Owner::Spouse, 50, None),
    ] {
        let mut entry = wage(0, owner, amount, 0);
        let FederalIncome::Wage { data } = &mut entry.income else { panic!() };
        let mut data = crate::tests::support::builders::edit_wage(data);
        data.statutory_employee = true;
        data.statutory_business = target;
        drop(data);
        add(&mut d, entry);
    }
    let mut c = income(cash_business(0));
    let FederalIncome::Business { data } = &mut c.income else { panic!() };
    let BusinessActivity::Ordinary { receipts, .. } = &mut data.activity else { panic!() };
    *receipts = 50;
    add(&mut d, c);
    let mut f = income(raised_farm(0, 0));
    f.owner = crate::DocumentOwner::Spouses;
    let FederalIncome::Business { data } = &mut f.income else { panic!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { panic!() };
    let FarmAccounting::Cash { receipts } = &mut data.accounting else { panic!() };
    *receipts = vec![FarmReceipt::Raised { amount: 50 }];
    add(&mut d, f);
    d
}

#[derive(Clone, Copy, Debug)]
pub(in crate::tests) enum WageClassificationCase {
    Deferred,
    Inmate,
    Mixed,
    MedicaidBox1,
    MedicaidCodeII,
    MedicaidCombined,
}

pub(in crate::tests) fn wage_classification_fixture(
    case: WageClassificationCase,
    owner: Owner,
) -> FederalDraft {
    let mut draft = injured_return((60_000, 5_000), (20_000, 500), 1_000, allocation());
    let FederalIncome::Wage { data } = &mut draft
        .income
        .iter_mut()
        .find(|entry| entry.owner == crate::DocumentOwner::from(owner))
        .unwrap()
        .income
    else {
        panic!()
    };
    let mut data = crate::tests::support::builders::edit_wage(data);
    match case {
        WageClassificationCase::Deferred => data.deferred_pension = 600_050,
        WageClassificationCase::Inmate => data.inmate_pay = 600_050,
        WageClassificationCase::Mixed => {
            data.deferred_pension = 600_050;
            data.inmate_pay = 200_050;
        }
        WageClassificationCase::MedicaidBox1 => data.medicaid_waiver = 50,
        WageClassificationCase::MedicaidCodeII => {
            data.other_box12.push(W2CodeAmount { code: W2Code::II, amount: 50 })
        }
        WageClassificationCase::MedicaidCombined => {
            data.medicaid_waiver = 50;
            data.other_box12.push(W2CodeAmount { code: W2Code::II, amount: 50 });
        }
    }
    drop(data);
    if matches!(case, WageClassificationCase::MedicaidCombined) {
        // The same annual native category combines a real OtherIncome source
        // with the W-2 codeII operand:50c+50c rounds ONCE to1, not twice to2.
        let mut entry = income(FederalIncome::OtherIncome {
            data: OtherIncomeItem {
                amount: 50,
                withholding: 0,
                kind: OtherIncomeKind::MedicaidWaiver { in_your_home: true },
            },
        });
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut draft, entry);
    }
    if !matches!(case, WageClassificationCase::Deferred | WageClassificationCase::Inmate) {
        // Two actual records in DIFFERENT annual categories: the opposite
        // owner's50c prize and this owner's50c NEC not-for-profit payment
        // each report1. Their combined raw100c must not replace these2.
        let mut prize = income(FederalIncome::OtherIncome {
            data: OtherIncomeItem {
                amount: 50,
                withholding: 0,
                kind: OtherIncomeKind::Prize {
                    noncash: 0,
                    disposition: PrizeDisposition::Received,
                },
            },
        });
        prize.owner = if owner == Owner::Taxpayer {
            crate::DocumentOwner::Spouses
        } else {
            crate::DocumentOwner::Yours
        };
        add(&mut draft, prize);
        let mut nec = income(FederalIncome::Nec1099 {
            data: crate::tests::support::builders::nec_source(FederalForm1099Nec {
                cash_tips: 0,
                tip_occupation_codes: Default::default(),
                overtime: 0,
                compensation: 50,
                withholding: 0,
                golden_parachute: 0,
                belongs: Some(IncomeUse::NotForProfit),
            }),
        });
        nec.owner = crate::DocumentOwner::from(owner);
        // Form 1099-NEC has no editable payer-name field.
        nec.label.clear();
        add(&mut draft, nec);
    }
    draft
}

pub(in crate::tests) fn non_code_l_wage_fixture(owner: Owner) -> (FederalDraft, i64) {
    let mut original = injured_return((60_000, 5_000), (20_000, 500), 1_000, allocation());
    let employer = original
        .income
        .iter()
        .find(|entry| entry.owner == crate::DocumentOwner::from(owner))
        .unwrap()
        .id;
    original.adjustments.as_mut().unwrap().employee_expenses = vec![EmployeeExpenseClaim {
        improvement_buildings: Vec::new(),
        equipment_assets: vec![],
        dot_hours_of_service: false,
        leased_vehicles: vec![],
        owner,
        occupation: "Fee-basis official, non-code-L reimbursement".into(),
        capacity: EmployeeCapacity::FeeBasisOfficial,
        employers: vec![employer],
        local_transportation: 190_000,
        travel: 0,
        other: 0,
        meals: 0,
        reimbursements: 200_000,
        mileage: vec![],
        vehicle_assets: vec![],
    }];
    (original, employer)
}

pub(in crate::tests) mod additional_medicare {
    use crate::tests::attribution::injured_spouse::{allocation, injured_return};
    use crate::tests::{add, cash_business, income, ordinary_partner_source, savings_income, wage};
    use crate::{
        FederalDraft, FederalIncome, FederalIncomeEntry, Owner, PartnerQbiStatement,
        PartnerSeGross, ScorpRiskStart, TaxSection, W2Payroll,
    };

    #[derive(Clone, Copy, Debug)]
    pub(in crate::tests) enum Source {
        PartnerSe,
        SmallBusiness,
        Railroad,
    }
    impl Source {
        pub(in crate::tests) fn topic(self) -> TaxSection {
            match self {
                Self::PartnerSe => TaxSection::PassThrough,
                Self::SmallBusiness => TaxSection::SelfEmployment,
                Self::Railroad => TaxSection::Wages,
            }
        }
        fn entry(self, owner: Owner) -> FederalIncomeEntry {
            let mut entry = match self {
                Self::PartnerSe => {
                    // Same actual active K1/box14 source shape as k1_items. All
                    // ordinary and SE income is current, no suspended histories.
                    let mut data = ordinary_partner_source(100000, true);
                    data.regular.basis.opening = Some(50000000);
                    data.regular.risk.start =
                        ScorpRiskStart::Detailed { capacity: 50000000, profit: 0, loss: 0 };
                    data.alternative = Some(data.regular.clone());
                    data.limited_for_se = Some(false);
                    data.reported_se = Some(10000000);
                    data.se_gross = Some(PartnerSeGross::Nonfarm { gross: 10000000 });
                    data.qbi = Some(PartnerQbiStatement {
                        income: 10000000,
                        wages: 0,
                        property: 0,
                        service: false,
                        section1231_qbi: 0,
                        reit_dividends: 0,
                        ptp_income: 0,
                    });
                    data.qbi_records.qualified_se_gross = 10000000;
                    income(FederalIncome::Partnership { data })
                }
                Self::SmallBusiness => income(cash_business(300)),
                Self::Railroad => {
                    // Existing railroad_w2_computes_8959_part_ii_separately shape.
                    // Actual employer withholding is zero in this synthetic source;
                    // do not fabricate a withheld amount from the tax calculation.
                    let mut entry = wage(0, owner, 30000000, 0);
                    let FederalIncome::Wage { data } = &mut entry.income else { unreachable!() };
                    let mut data = crate::tests::support::builders::edit_wage(data);
                    data.social_wages = 0;
                    data.social_tax = 0;
                    data.medicare_wages = 0;
                    data.medicare_tax = 0;
                    data.payroll = W2Payroll::Railroad {
                        compensation: 30000000,
                        tier_1: 1143900,
                        tier_2: 0,
                        medicare: 435000,
                        additional_medicare: 0,
                    };
                    drop(data);
                    entry
                }
            };
            entry.owner = crate::DocumentOwner::from(owner);
            entry
        }
    }

    pub(in crate::tests) fn fixture(
        source: Source,
        owner: Owner,
        injured: Owner,
        reverse: bool,
    ) -> FederalDraft {
        let mut a = allocation();
        a.injured = injured;
        let wages = match source {
            Source::PartnerSe => 200000,
            Source::SmallBusiness => 300000,
            Source::Railroad => 100000,
        };
        let mut d = if owner == Owner::Taxpayer {
            injured_return((0, 0), (wages, 100000), 0, a)
        } else {
            injured_return((wages, 100000), (0, 0), 0, a)
        };
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut entry = income(savings_income(10000, 0));
            entry.owner = crate::DocumentOwner::from(owner);
            add(&mut d, entry);
        }
        add(&mut d, source.entry(owner));
        if reverse {
            d.income.reverse();
        }
        d
    }
}

pub(in crate::tests) mod home_employee {
    use crate::tests::attribution::injured_spouse::{allocation, injured_return};
    use crate::tests::{add, income, savings_income};
    use crate::{
        EmployeeWageReason, EmployeeWages, FederalDraft, FederalIncome, FederalIncomeEntry,
        InjuredAmount, Owner, TaxSection,
    };

    #[derive(Clone, Copy, Debug)]
    pub(in crate::tests) enum Source {
        ExcludedHome,
        TaxableHome,
        CoveredEmployee,
        OppositeEmployee,
        UncoveredEmployee,
    }
    impl Source {
        pub(in crate::tests) fn home(self) -> bool {
            matches!(self, Self::ExcludedHome | Self::TaxableHome)
        }
        pub(in crate::tests) fn topic(self) -> TaxSection {
            if self.home() { TaxSection::InvestmentSales } else { TaxSection::Wages }
        }
        pub(in crate::tests) fn entry(self) -> FederalIncomeEntry {
            if self.home() {
                // Existing real MainHomeSale page fixture: native closing dates,
                // whole-time residence and both joint residence tests. Full225000
                // exclusion; the second case has510000 gain less500000 exclusion.
                let mut page = crate::tests::home::home_sale_page::home(true);
                if matches!(self, Self::TaxableHome) {
                    page.form.paper_mut().unwrap().box_2a_total_gross_proceeds = 73_500_000;
                }
                let label = page.form().address;
                let mut entry = income(FederalIncome::HomeSale { data: page });
                entry.label = label;
                entry
            } else {
                let mut entry = income(FederalIncome::EmployeeWages {
                    data: EmployeeWages {
                        source_issuer: Default::default(),
                        source_recipient: Default::default(),
                        wages: if matches!(self, Self::OppositeEmployee) {
                            16_000_000
                        } else {
                            500_000
                        },
                        withholding: 0,
                        form1099: true,
                        payroll: if matches!(self, Self::CoveredEmployee | Self::OppositeEmployee) {
                            crate::EmployeePayrollSource::Form8919 {
                                reason: EmployeeWageReason::Ss8Determination { date: 20260501 },
                            }
                        } else {
                            crate::EmployeePayrollSource::Unavailable
                        },
                    },
                });
                entry.label.clear();
                entry
            }
        }
    }

    pub(in crate::tests) fn fixture(
        source: Source,
        owner: Owner,
        injured: Owner,
        reversed: bool,
        answered: bool,
    ) -> FederalDraft {
        let mut a = allocation();
        a.injured = injured;
        if answered && matches!(source, Source::CoveredEmployee | Source::OppositeEmployee) {
            // Actual native Form8959 joint tax45 (or90 for the opposite-source
            // case); an independent direct choice assigns27/18 (or54/36) to
            // taxpayer/spouse for either selector. The source invents no choice.
            a.allocations.push(InjuredAmount {
                component: "additional_medicare_tax".into(),
                injured: match (source, injured) {
                    (Source::OppositeEmployee, Owner::Taxpayer) => 5400,
                    (Source::OppositeEmployee, Owner::Spouse) => 3600,
                    (_, Owner::Taxpayer) => 2700,
                    (_, Owner::Spouse) => 1800,
                },
            });
        }
        let mut d = if matches!(source, Source::OppositeEmployee) {
            // One actual W2 source is opposite the actual Form8919 earner.
            if owner == Owner::Spouse {
                injured_return((100_000, 60_000), (0, 0), 0, a)
            } else {
                injured_return((0, 0), (100_000, 60_000), 0, a)
            }
        } else {
            injured_return((200_000, 60_000), (50_000, 10_000), 0, a)
        };
        for holder in [Owner::Taxpayer, Owner::Spouse] {
            let mut entry = income(savings_income(10_000, 0));
            entry.owner = crate::DocumentOwner::from(holder);
            add(&mut d, entry);
        }
        let mut entry = source.entry();
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut d, entry);
        if reversed {
            d.income.reverse();
        }
        d
    }
}

pub(in crate::tests) mod payroll_taxes {
    use crate::tests::attribution::injured_spouse::{allocation, injured_return};
    use crate::tests::{add, allocated_employee_tips, income, savings_income};
    use crate::{
        EmployeeWageReason, EmployeeWages, FederalDraft, FederalForm1099Nec, FederalIncome,
        IncomeUse, InjuredAmount, Owner, TaxSection, TipCoverage, TipsClaim, W2Code, W2CodeAmount,
        W2Payroll, W2UncollectedTax,
    };

    #[derive(Clone, Copy, Debug)]
    pub(in crate::tests) enum Source {
        SocialTips,
        GovernmentTips,
        RailroadTips,
        Uncollected,
        CentPair,
        GoldenW2,
        GoldenNec,
        SameEmployee,
    }
    impl Source {
        pub(in crate::tests) fn tips(self) -> bool {
            matches!(self, Self::SocialTips | Self::GovernmentTips | Self::RailroadTips)
        }
        pub(in crate::tests) fn topic(self) -> TaxSection {
            if matches!(self, Self::GoldenNec) {
                TaxSection::SelfEmployment
            } else {
                TaxSection::Wages
            }
        }
    }
    pub(in crate::tests) fn fixture(
        source: Source,
        owner: Owner,
        injured: Owner,
        reverse: bool,
    ) -> FederalDraft {
        let mut a = allocation();
        a.injured = injured;
        if matches!(source, Source::SocialTips | Source::GovernmentTips) {
            a.allocations.push(InjuredAmount {
                component: "additional_medicare_tax".into(),
                injured: if injured == Owner::Taxpayer { 2700 } else { 1800 },
            });
        }
        let mut d = if matches!(source, Source::SameEmployee) {
            if owner == Owner::Taxpayer {
                injured_return((100000, 60000), (0, 0), 0, a)
            } else {
                injured_return((0, 0), (100000, 60000), 0, a)
            }
        } else {
            injured_return((200000, 60000), (50000, 10000), 0, a)
        };
        for holder in [Owner::Taxpayer, Owner::Spouse] {
            let mut item = income(savings_income(10000, 0));
            item.owner = crate::DocumentOwner::from(holder);
            add(&mut d, item);
        }
        for entry in &mut d.income {
            let FederalIncome::Wage { data } = &mut entry.income else {
                continue;
            };
            let mut data = crate::tests::support::builders::edit_wage(data);
            if source.tips() && entry.owner == crate::DocumentOwner::from(owner) {
                let mut tips = allocated_employee_tips(5000);
                tips.coverage = match source {
                    Source::GovernmentTips => TipCoverage::MedicareOnlyGovernment,
                    Source::RailroadTips => TipCoverage::Railroad,
                    _ => TipCoverage::SocialAndMedicare,
                };
                data.employee_tips = Some(tips);
                // Explicit synthetic occupation fact for the normal editor:
                // these tips are ineligible for Schedule1-A. Preserve their
                // full income/payroll amounts and every money oracle.
                data.work_deductions.tips_claim = TipsClaim::Yes {
                    w2_tips: 0,
                    form4137_tips: 500000,
                    tips_eligible: Some(false),
                };
                if matches!(source, Source::GovernmentTips) {
                    // This employer's government pay is Medicare-only, including
                    // its W2 wage boxes. Federal income/tip-tax figures stay the
                    // same; do not retain a generic helper's SS withholding.
                    data.social_wages = 0;
                    data.social_tax = 0;
                }
                if matches!(source, Source::RailroadTips) {
                    let social = data.social_tax;
                    let compensation = data.wages;
                    data.social_wages = 0;
                    data.social_tax = 0;
                    data.medicare_wages = 0;
                    data.medicare_tax = 0;
                    data.payroll = W2Payroll::Railroad {
                        compensation,
                        tier_1: social,
                        tier_2: 0,
                        medicare: compensation * 145 / 10000,
                        additional_medicare: 0,
                    };
                }
            }
            if matches!(source, Source::Uncollected)
                && entry.owner == crate::DocumentOwner::from(owner)
            {
                data.uncollected_tax = Some(W2UncollectedTax {
                    tip_social: 0,
                    tip_medicare: 0,
                    life_social: 6200,
                    life_medicare: 1450,
                });
                data.social_tax -= 6200;
                data.medicare_tax -= 1450;
            }
            if matches!(source, Source::CentPair) {
                // Actual opposite-owner source amounts49c+49c. Native one
                // combined payroll category rounds98c to1, never0+0.
                let social = if entry.owner == crate::DocumentOwner::from(owner) { 49 } else { 0 };
                let medicare = 49 - social;
                data.uncollected_tax = Some(W2UncollectedTax {
                    tip_social: 0,
                    tip_medicare: 0,
                    life_social: social,
                    life_medicare: medicare,
                });
                data.social_tax -= social;
                data.medicare_tax -= medicare;
            }
            if matches!(source, Source::GoldenW2)
                && entry.owner == crate::DocumentOwner::from(owner)
            {
                data.other_box12.push(W2CodeAmount { code: W2Code::K, amount: 100000 });
            }
            drop(data);
        }
        if matches!(source, Source::GoldenNec) {
            let mut item = income(FederalIncome::Nec1099 {
                data: crate::tests::support::builders::nec_source(FederalForm1099Nec {
                    cash_tips: 0,
                    tip_occupation_codes: Default::default(),
                    overtime: 0,
                    compensation: 2000000,
                    withholding: 0,
                    golden_parachute: 500000,
                    belongs: Some(IncomeUse::NotForProfit),
                }),
            });
            item.label.clear();
            item.owner = crate::DocumentOwner::from(owner);
            add(&mut d, item);
        }
        if matches!(source, Source::SameEmployee) {
            let mut item = income(FederalIncome::EmployeeWages {
                data: EmployeeWages {
                    source_issuer: Default::default(),
                    source_recipient: Default::default(),
                    wages: 16000000,
                    withholding: 0,
                    form1099: true,
                    payroll: crate::EmployeePayrollSource::Form8919 {
                        reason: EmployeeWageReason::Ss8Determination { date: 20260501 },
                    },
                },
            });
            item.label.clear();
            item.owner = crate::DocumentOwner::from(owner);
            add(&mut d, item);
        }
        if reverse {
            d.income.reverse();
        }
        d
    }
}

pub(in crate::tests) mod medicare_withholding {
    use crate::tests::attribution::injured_spouse::{allocation, injured_return};
    use crate::tests::{add, income, savings_income, wage};
    use crate::{FederalDraft, FederalIncome, Owner, W2Payroll, W2UncollectedTax};

    #[derive(Clone, Copy, Debug)]
    pub(in crate::tests) enum Source {
        Railroad,
        HalfCategories,
        Uncollected,
        SignedOrdinary,
        ZeroUnderage,
    }
    pub(in crate::tests) fn fixture(
        source: Source,
        owner: Owner,
        injured: Owner,
        reverse: bool,
    ) -> FederalDraft {
        let mut a = allocation();
        a.injured = injured;
        let railroad = matches!(source, Source::Railroad | Source::HalfCategories);
        let mut d = if railroad {
            if owner == Owner::Taxpayer {
                injured_return((0, 0), (100000, 100000), 0, a)
            } else {
                injured_return((100000, 100000), (0, 0), 0, a)
            }
        } else {
            injured_return((200000, 60000), (50000, 10000), 0, a)
        };
        for holder in [Owner::Taxpayer, Owner::Spouse] {
            let mut entry = income(savings_income(10000, 0));
            entry.owner = crate::DocumentOwner::from(holder);
            add(&mut d, entry);
        }
        for entry in &mut d.income {
            if let FederalIncome::Wage { data } = &mut entry.income {
                let mut data = crate::tests::support::builders::edit_wage(data);
                if railroad && entry.owner != crate::DocumentOwner::from(owner) {
                    data.medicare_tax +=
                        if matches!(source, Source::HalfCategories) { 50 } else { 4900 };
                } else if matches!(source, Source::Uncollected)
                    && entry.owner == crate::DocumentOwner::from(owner)
                {
                    data.medicare_tax -= 1450;
                    data.uncollected_tax = Some(W2UncollectedTax {
                        tip_social: 0,
                        tip_medicare: 0,
                        life_social: 0,
                        life_medicare: 1550,
                    });
                } else if matches!(source, Source::SignedOrdinary | Source::ZeroUnderage) {
                    data.medicare_tax += if matches!(source, Source::ZeroUnderage) {
                        if entry.owner == crate::DocumentOwner::from(owner) { -1 } else { 0 }
                    } else if entry.owner == crate::DocumentOwner::from(owner) {
                        150
                    } else {
                        -1
                    };
                }
                drop(data);
            }
        }
        if railroad {
            let mut entry = wage(0, owner, 30000000, 0);
            let FederalIncome::Wage { data } = &mut entry.income else { unreachable!() };
            let mut data = crate::tests::support::builders::edit_wage(data);
            data.social_wages = 0;
            data.social_tax = 0;
            data.medicare_wages = 0;
            data.medicare_tax = 0;
            data.payroll = W2Payroll::Railroad {
                compensation: 30000000,
                tier_1: 1143900,
                tier_2: 0,
                medicare: 435000,
                additional_medicare: if matches!(source, Source::HalfCategories) {
                    50
                } else {
                    4900
                },
            };
            drop(data);
            add(&mut d, entry);
        }
        if reverse {
            d.income.reverse();
        }
        d
    }
}

// Workplace retirement coverage comes from the actual W-2 box 13 source.
pub(in crate::tests) fn covered_wage(
    id: i64,
    owner: Owner,
    amount: i64,
    withholding: i64,
) -> FederalIncomeEntry {
    let mut entry = wage(id, owner, amount, withholding);
    let FederalIncome::Wage { data } = &mut entry.income else { unreachable!() };
    let mut data = crate::tests::support::builders::edit_wage(data);
    data.retirement_plan = true;
    drop(data);
    entry
}

// One actual RRTA source; callers state each component's availability explicitly.
pub(in crate::tests) fn employee_railroad_source(
    amounts: [SourceCents; 5],
) -> EmployeeRailroadSource {
    let [compensation, tier_1, tier_2, medicare, additional_medicare] = amounts;
    EmployeeRailroadSource { compensation, tier_1, tier_2, medicare, additional_medicare }
}

pub(in crate::tests) fn informational_wage_document() -> W2 {
    let mut form = crate::tests::support::builders::native_income::native_w2(6_000_000);
    form.state_details.control_number = "00-SYNTHETIC/7".into();
    form.more.box12 = [
        (W2Code::C, 101),
        (W2Code::J, 202),
        (W2Code::V, 303),
        (W2Code::Y, 404),
        (W2Code::DD, 505),
        (W2Code::GG, 606),
        (W2Code::HH, 0),
    ]
    .into_iter()
    .map(|(code, amount)| Box12Entry { code, amount })
    .collect();
    form
}
