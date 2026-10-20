use crate::tests::*;
pub(in crate::tests) fn person() -> FederalPerson {
    FederalPerson {
        birth_date: 19800615,
        blind: false,
        dependency: FilerDependency::Independent,
        died: None,
        another_eitc_child: false,
        valid_ssn: true,
        valid_tin: true,
        us_home_over_half_year: true,
        full_time_student: false,
        disabled: false,
        incapable_self_care: false,
        support_share: SupportShare::MoreThanHalf,
        parent_alive: true,
        first_name: "Synthetic".into(),
        last_name: "Filer".into(),
        real_estate_professional: None,
    }
}
// Records the People page's real estate professional status of an owner.
pub(crate) fn make_professional(d: &mut FederalDraft, owner: Owner, over_500_hours: bool) {
    let people = d.people.as_mut().expect("people page");
    let person = match owner {
        Owner::Taxpayer => &mut people.taxpayer,
        Owner::Spouse => people.spouse.as_mut().expect("spouse"),
    };
    person.real_estate_professional =
        Some(ProfessionalRentals { over_500_hours, single_activity: false });
}
pub(in crate::tests) fn wage(
    id: i64,
    owner: Owner,
    amount: i64,
    withholding: i64,
) -> FederalIncomeEntry {
    FederalIncomeEntry {
        refund_management: None,
        state_source: Default::default(),
        id,
        label: alloc::format!("Employer {id}"),
        owner,
        income: FederalIncome::Wage {
            data: FederalWage {
                abroad: 0,
                deferred_pension: 0,
                inmate_pay: 0,
                disability: DisabilityIncome::None,
                wages: amount,
                withholding,
                social_wages: amount.min(18_450_000),
                social_tips: 0,
                employee_tips: None,
                work_deductions: FederalWorkDeductions::NONE,
                social_tax: amount.min(18_450_000) * 62 / 1000,
                medicare_wages: amount,
                medicare_tax: amount * 145 / 10000 + (amount - 20_000_000).max(0) * 9 / 1000,
                uncollected_tax: None,
                medicaid_waiver: 0,
                statutory_business: None,
                paying_scorporation: None,
                payroll: crate::W2Payroll::SocialSecurity,
                coverage: crate::W2Coverage::Ordinary,
                care_benefits: 0,
                statutory_employee: false,
                retirement_plan: false,
                other_box12: Vec::new(),
            },
        },
        community: CommunityShare::NotNeeded,
    }
}
pub(in crate::tests) fn filing_choice(status: FederalStatus) -> FilingChoice {
    match status {
        FederalStatus::Single => FilingChoice::Single,
        FederalStatus::Joint => FilingChoice::Joint,
        FederalStatus::Separate => {
            FilingChoice::Separate { section179: Section179Spouse::NotSupplied }
        }
        FederalStatus::Head => FilingChoice::Head,
        FederalStatus::Surviving => FilingChoice::Surviving,
    }
}
// A finished return with the filers and every page of `complete` but no
// income: the start of a fixture that enters only its own documents.
pub(in crate::tests) fn empty_return(status: FederalStatus) -> FederalDraft {
    let mut d = complete(status);
    d.income = vec![];
    d
}

pub(in crate::tests) fn complete(status: FederalStatus) -> FederalDraft {
    let mut d = FederalDraft::empty("synthetic-return".into());
    let joint = status == FederalStatus::Joint;
    d.setup = Some(FederalSetup {
        separate_spouse: SeparateSpouseBoxes::NoneOfThese,
        label: crate::common::test_label("Synthetic return"),
        basis: AmountBasis::Estimate,
        status: filing_choice(status),
        marital_state: if joint || status == FederalStatus::Separate {
            MaritalState::Married { living: SpouseLiving::Together }
        } else {
            MaritalState::Unmarried
        },
        paid_over_half_home: true,
        community_property: CommunityProperty::No,
    });
    d.people = Some(FederalPeople {
        parent_tax: None,
        taxpayer: person(),
        spouse: (if joint { Some(person()) } else { None }).into(),
    });
    // The people page was passed with nobody on it, and each section's
    // less-common page with "None of these" (gate fixes batch A).
    d.finished_sections = vec![
        TaxSection::Children,
        TaxSection::Checklist,
        TaxSection::MoreIncome,
        TaxSection::Income,
        TaxSection::MoreAdjustments,
        TaxSection::Adjustments,
        TaxSection::Deductions,
        TaxSection::MoreCredits,
        TaxSection::Care,
        TaxSection::Credits,
        TaxSection::OtherTaxes,
        TaxSection::Payments,
    ];
    d.income = vec![wage(1, Owner::Taxpayer, if joint { 10_000_000 } else { 6_000_000 }, 650_000)];
    d.next_id = 2;
    d.adjustments = Some(FederalAdjustments {
        health_employer_w2: vec![],
        health_marketplace: HealthMarketplace::default(),
        social_election: SocialElection::Regular,
        alimony_paid: vec![],
        moving: vec![],
        pension_contributions: vec![],
        hsa: vec![],
        hsa_family_share: None,
        archer: ArcherContributions::None,
        qualified_charity: vec![],
        ira_basis: vec![],
        roth_history: vec![],
        required_distributions: vec![],
        iras: FederalIras::NONE,
        student_loan: StudentLoanClaim::NONE,
        educators: EducatorClaim::NONE,
        employee_expenses: vec![],
    });
    d.deductions = Some(FederalDeductions {
        investment_interest: None,
        mortgage: vec![],
        repayment: RepaymentClaim::None,
        choice: DeductionSelection::Automatic { expenses: ItemizedChoice::No },
        spouse_itemizes: false,
        personal_casualties: vec![],
        property_gifts: vec![],
        appraisals: vec![],
        basis_election: false,
        annuity_losses: vec![],
        vehicle_loans: vec![],
        cash_charity: 0,
        fund_cash_charity: 0,
        other_cash_charity: 0,
        volunteer_expenses: vec![],
        hosted_students: vec![],
        whaling_captains: vec![],
        charity_carryovers: vec![],
    });
    d.credits = Some(FederalCredits {
        decline_earned_income_credit: false,
        tip_fica_carryforward: None,
        business_credits: None,
        bba_audit_interest: None,
        tax_credit_bonds: vec![],
        after_disallowance: None,
        minimum_tax: None,
        mortgage_credit: None,
        mortgage_subsidies: vec![],
        dc_homebuyer_carry: None,
        residential_energy_carry: None,
        personal_refueling: vec![],
        clean_vehicles: None,
        legacy_ev_passive_allowed: None,
        education: vec![],
        education_recapture: None,
        saver: (None).into(),
        foreign_tax: ForeignTaxClaim::No,
        exempt_pensions: vec![],
        adoptions: vec![],
        adoption_carry: vec![],
        marketplace: MarketplaceClaim::No,
        care: FederalCare::default(),
    });
    d.screening = Some(FederalScreening {
        qbi_aggregations: Vec::new(),
        clergy: Vec::new(),
        farm_averaging: None,
        optional_se: vec![],
        foreign_earned: vec![],
        farm_nol: None,
        household_employment: vec![],
        able_contributions: vec![],
        trump_accounts: vec![],
        public_safety: vec![],
        iso_exercises: vec![],
        capital_history: vec![],
        section1231: None,
        atnol: AtnolHistory::None,
        nol: NolHistory::None,
        qbi_history: QbiHistory::NONE,
        foreign_account: false,
        fbar_required: None,
        fbar_countries: Vec::new(),
        foreign_trust: false,
        disaster_carries: vec![],
        foreign_asset_max: None,
        installment_payments: vec![],
        section_1256_carryback: None,
        section_1256_carryback_owners: None,
        related_exchanges: vec![],

        taxable_premium_election: None,
    });
    d.payments = Some(FederalPayments {
        credit_applied_next_year: 0,
        credit_recipient: None,
        extension_payer: None,
        extension_refund_property: None,
        estimated: Some(0),
        extension: 0,
        home_kerosene: None,
        estimated_payments: vec![],
        prior_year: None,
        withholding_by_date: false,
        withholding_periods: None,
        annualized: false,
        annualized_income: None,
        farmer_fisherman_prior_year: false,
        farmer_paid_by_march: false,
        waiver: None,
        next_year: NextYearEstimateRecord::NotNeeded,
        joint,
    });
    d
}
pub(in crate::tests) fn child(id: i64) -> FederalDependent {
    FederalDependent {
        use_for: PersonUse::TaxDependent,
        id,
        first_name: "Child".into(),
        last_name: alloc::format!("{id}"),
        birth_date: 20180615,
        death_date: None,
        relationship: Relationship::Child,
        residency: DependentResidency::Days { days: 365 },
        us_home_over_half_year: true,
        incapable_self_care: false,
        valid_ssn: true,
        valid_tin: true,
        situation: DependentSituation::Supported { your_support_over_half: true, gross_income: 0 },
        facts: PersonFacts::NONE.0,
        care: DependentCare::No,
        competing: CompetingClaim::NoOne,
    }
}
pub(in crate::tests) fn link_home_fixture(d: &mut FederalDraft, v: &mut FederalRental) {
    let interest = v
        .expenses
        .iter()
        .filter(|e| matches!(e.category, RentalExpenseCategory::MortgageInterest))
        .map(|e| e.amount)
        .sum();
    v.expenses.retain(|e| !matches!(e.category, RentalExpenseCategory::MortgageInterest));
    d.deductions.as_mut().unwrap().set_mortgage_interest(interest);
    d.deductions.as_mut().unwrap().mortgage[0].id = 101;
    v.mortgage_loans = vec![101];
}
pub(in crate::tests) fn add(d: &mut FederalDraft, mut e: FederalIncomeEntry) {
    e.id = d.next_id;
    d.next_id += 1;
    // A community-property return divides every item; a fixture says Community.
    if e.community == CommunityShare::NotNeeded
        && d.setup.as_ref().is_some_and(crate::community::applies)
    {
        e.community = CommunityShare::Community;
    }
    for id in crate::rental_energy::ids_mut(&mut e.income) {
        if *id == 0 {
            *id = d.next_id;
            d.next_id += 1;
        }
    }
    // A farm's CCC loans receive their identities the way a saved farm's do.
    for id in crate::farm::ccc_lot_ids_mut(&mut e.income) {
        if *id == 0 {
            *id = d.next_id;
            d.next_id += 1;
        }
    }
    if let Some(records) = crate::method_change::records_mut(&mut e.income) {
        for record in records {
            if record.id == 0 {
                record.id = d.next_id;
                d.next_id += 1;
            }
        }
    }
    if let FederalIncome::Cooperative { data } = &mut e.income {
        if data.allocations.is_empty() {
            let eligible = crate::cooperative::eligible_activities(d, e.owner);
            data.allocations.push(crate::cooperative::automatic(
                data,
                (eligible.len() == 1).then(|| eligible[0]),
            ));
        }
        let eligible = crate::cooperative::eligible_activities(d, e.owner);
        for a in &mut data.allocations {
            if a.activity.is_none() && eligible.len() == 1 {
                a.activity = Some(eligible[0]);
            }
            if a.id == 0 {
                a.id = d.next_id;
                d.next_id += 1;
            }
        }
    }
    if let Some(farm) = match &mut e.income {
        FederalIncome::Business { data } => match &mut data.activity {
            BusinessActivity::Farming { data } => Some(data),
            _ => None,
        },
        FederalIncome::FarmRental { data } => Some(&mut data.farm),
        _ => None,
    } {
        fixture_conservation_parcel(farm, &mut d.next_id);
    }
    d.income.push(e);
}
// The record identity of one CCC loan of the farm at an income position.
pub(in crate::tests) fn lot_id(d: &FederalDraft, entry: usize, lot: usize) -> i64 {
    crate::farm::ccc_lot_ids(&d.income[entry].income)[lot]
}
// The identity of the fixture W-2 labelled "Employer <n>" (add gives each entry
// the next identity): how the care and adoption fixtures name their W-2s.
pub(in crate::tests) fn w2(d: &FederalDraft, n: i64) -> i64 {
    let label = alloc::format!("Employer {n}");
    d.income.iter().find(|e| e.label == label).unwrap_or_else(|| panic!("no W-2 {label}")).id
}
pub(in crate::tests) fn add_child(d: &mut FederalDraft) {
    let c = child(d.next_id);
    d.next_id += 1;
    d.dependents.push(c);
}
pub(in crate::tests) fn income(value: FederalIncome) -> FederalIncomeEntry {
    FederalIncomeEntry {
        refund_management: None,
        state_source: Default::default(),
        id: 0,
        label: "Synthetic document".into(),
        owner: Owner::Taxpayer,
        income: value,
        community: CommunityShare::NotNeeded,
    }
}

// Every checklist topic ticked, so a prune keeps the pages and documents of a
// fixture that was not built through the checklist.
pub(in crate::tests) fn all_ticked(d: &mut FederalDraft) {
    d.checklist = Checklist {
        income: u32::MAX,
        adjustments: u32::MAX,
        deductions: u32::MAX,
        credit_topics: u32::MAX,
        other_taxes: u32::MAX,
        payments: u32::MAX,
        whole_return: u32::MAX,
    };
}
// Sets the age at the end of 2026 of a person on the return (-1 you, -2 your
// spouse) through the birth date on the people page: the one source of an
// insured person's long-term-care limit. A birthday on January 2 attains the
// age on January 1 and the next one a year later (the day-exact rule).
pub(in crate::tests) fn set_age(d: &mut FederalDraft, person: i64, age: u32) {
    let birth = (2026 - age) * 10000 + 102;
    let people = d.people.as_mut().unwrap();
    match person {
        -1 => people.taxpayer.birth_date = birth,
        -2 => people.spouse.as_mut().unwrap().birth_date = birth,
        _ => panic!("a dependent's age is set on the dependent"),
    }
}
// The identity of the person at a position of the hidden student catalog:
// the stored education and Marketplace rows name the person, not a position.
pub(in crate::tests) fn sid(d: &FederalDraft, index: usize) -> i64 {
    crate::federal_rules::people_students(d)[index].id
}
