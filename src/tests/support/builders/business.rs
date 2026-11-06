use crate::tests::*;

pub(in crate::tests) fn cash_business(profit: i64) -> FederalIncome {
    FederalIncome::Business {
        data: FederalBusiness {
            improvement_buildings: Vec::new(),
            interest_limit_election: None,
            related: Default::default(),
            principal_business: Default::default(),
            business_code: None,
            ein: Default::default(),
            address: Default::default(),
            started_current_year: false,
            information_returns: InformationReturnFiling::NoneRequired,
            other_income: vec![],
            de_minimis: None,
            leased_vehicles: vec![],
            depletion: None,
            ministerial: false,
            notary_profit: 0,
            notary_gross: None,
            source: BusinessSource::UnitedStates,
            income_source: BusinessIncomeSource::Services,
            activity: BusinessActivity::Ordinary {
                returns: 0,
                receipts: profit.max(0) * 100,
                accounting: BusinessAccounting::Cash,
            },
            inventory: BusinessInventory::None,
            startup: BusinessStartup::None,
            method_changes: Vec::new(),
            intangibles: vec![],
            timber: vec![],
            other_amortization: Vec::new(),
            health_insurance: BusinessHealthInsurance::None,
            retirement: BusinessRetirement::None,
            vehicles: vec![],
            assets: vec![],
            expenses: if profit < 0 {
                vec![ordinary_business_expense(
                    BusinessExpenseCategory::Other { label: "Operating costs".into() },
                    -profit * 100,
                )]
            } else {
                vec![]
            },
            fuel_tax_credit: None,
            cash_fuel_tax_recovery: None,
            employer_tip_fica_credit: None,
            work_opportunity_credit: None,
            retirement_plan_credit: None,
            small_employer_health_credit: None,
            paid_leave_credit: None,
            tips: None,
            overtime: None,
            material_participation: crate::ParticipationFacts {
                hours: 501,
                test: crate::ParticipationTest::Over500,
            },
            at_risk: crate::RentalAtRisk::All,
            specified_service: false,
            payroll: BusinessPayroll::No,
            special_treatment: false,
            qbi_aggregation_group: None,
            withholding: 0,
            commercial_fishing: false,
            sale: None,
        },
    }
}

pub(in crate::tests) fn health_business(
    profit: i64,
    premiums: i64,
    employer: bool,
) -> FederalIncome {
    let mut income = cash_business(profit);
    let FederalIncome::Business { data } = &mut income else { unreachable!() };
    data.health_insurance = BusinessHealthInsurance::Plan {
        long_term_care: vec![],
        months: vec![BusinessHealthMonth {
            child_increase: 0,
            month: 1,
            premiums: premiums * 100,
            employer_eligible: employer,
            reimbursed: None,
        }],
        establishment: crate::HealthEstablishment::SoleProprietor,
    };
    income
}

pub(in crate::tests) fn sep_business(profit: i64, contribution: i64) -> FederalIncome {
    let mut entry = cash_business(profit);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    data.retirement = BusinessRetirement::Sep { rate: 25.0, contribution: contribution * 100 };
    entry
}

pub(in crate::tests) fn capital_business(profit: i64) -> FederalIncome {
    let mut source = cash_business(profit);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    data.income_source = BusinessIncomeSource::Capital;
    source
}

pub(in crate::tests) fn mixed_business(profit: i64, compensation: i64) -> FederalIncome {
    let mut result = cash_business(profit);
    let FederalIncome::Business { data } = &mut result else { unreachable!() };
    data.income_source = BusinessIncomeSource::Mixed { compensation: compensation * 100 };
    result
}

pub(in crate::tests) fn self_employed_insurance(
    d: &mut FederalDraft,
    confirmed: bool,
    weeks: u32,
    distributed: u32,
) {
    add(
        d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            10000,
            RetirementTreatment::Exception {
                second: None,
                distributed,
                facts: RetirementException::UnemploymentInsurance {
                    premiums: 400000,
                    qualification: UnemploymentQualification::SelfEmployed { confirmed },
                    benefit_year: 2025,
                    weeks,
                    reemployment: Reemployment::Started { date: 20260101 },
                },
                early_simple: false,
            },
        ),
    );
}

pub(in crate::tests) fn care_business(profit: i64, paid: i64, employer: bool) -> FederalIncome {
    let mut business = cash_business(profit);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.health_insurance = BusinessHealthInsurance::Plan {
        months: vec![],
        long_term_care: vec![BusinessCarePremium {
            person: -1,
            months: vec![BusinessCareMonth {
                month: 1,
                premiums: paid * 100,
                employer_eligible: employer,
                reimbursed: None,
            }],
            qualified: CareContractQualification::Qualified,
        }],
        establishment: crate::HealthEstablishment::SoleProprietor,
    };
    business
}

pub(in crate::tests) fn source_expenses_business(
    d: &mut FederalDraft,
    owner: Owner,
    amount: i64,
) -> i64 {
    let mut entry = income(cash_business(-amount));
    entry.owner = crate::DocumentOwner::from(owner);
    add(d, entry);
    d.income.last().unwrap().id
}

pub(in crate::tests) fn add_ebl_business(d: &mut FederalDraft, owner: Owner, net: i64) {
    let mut entry = income(cash_business(net));
    entry.owner = crate::DocumentOwner::from(owner);
    add(d, entry);
}

pub(in crate::tests) fn ordinary_business_expense(
    category: BusinessExpenseCategory,
    amount: i64,
) -> FederalBusinessExpense {
    FederalBusinessExpense { category, amount, treatment: BusinessExpenseKind::Ordinary }
}

// Current ordinary Schedule C source, sharing the existing typed6198 fixture.
pub(in crate::tests) fn current_risk_business(profit: i64, capacity_cents: i64) -> FederalIncome {
    let mut source = cash_business(profit);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    data.at_risk = simplified_rental_risk(capacity_cents, 0, 0, [0; 4]).unwrap();
    source
}

pub(in crate::tests) fn direct_participation_with_omitted_k1(
    mut draft: FederalDraft,
) -> (FederalDraft, i64) {
    add(&mut draft, income(cash_business(1000)));
    let direct = draft.income.last().unwrap().id;
    let mut other = scorp_document();
    let mut facts = participation_facts();
    facts.minutes = 300 * 60;
    facts.other_business_over_100 = true;
    other.participation = Some(facts);
    add(&mut draft, income(FederalIncome::Scorp { data: scorp_source(other) }));
    (draft, direct)
}

// Current ordinary domestic source with accepted absence of prior losses and
// unlisted special treatment, but no material-participation test established.
pub(in crate::tests) fn passive_business(profit: i64) -> FederalIncome {
    let mut source = cash_business(profit);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    data.material_participation =
        crate::ParticipationFacts { hours: 0, test: crate::ParticipationTest::None };
    source
}

pub(in crate::tests) fn ministerial_housing_business_return(
    passive: bool,
    exempt: bool,
) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 26_000_000, 0);
    let mut source = if passive { passive_business(20_000) } else { cash_business(20_000) };
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    data.ministerial = true;
    data.expenses = vec![ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Ministerial operating costs".into() },
        1_000_000,
    )];
    add(&mut draft, income(source));
    draft.screening.as_mut().unwrap().clergy = vec![ClergyClaim {
        owner: Owner::Taxpayer,
        role: ClergyRole::Minister {
            housing: MinisterHousing::Allowance {
                designated: 1_000_000,
                spent: 1_000_000,
                fair_rental_value: 1_000_000,
            },
            form_4361: exempt,
            employee_expenses: 0,
        },
    }];
    draft
}
