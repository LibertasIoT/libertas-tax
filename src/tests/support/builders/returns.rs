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
        whole_return: 0,
    }
}
// The whole-return situations as the filers' pages store them: the
// YourSituations items on the taxpayer, the SpouseSituations items on the
// spouse.
pub(in crate::tests) trait FilerSituations {
    fn set_situations(&mut self, bits: u32, on: bool);
}
impl FilerSituations for FederalDraft {
    fn set_situations(&mut self, bits: u32, on: bool) {
        let people = self.people.as_mut().expect("people page");
        let spouse_bits =
            bits & !(YourSituations::DUAL_STATUS.0 | YourSituations::COMMUNITY_PROPERTY.0);
        assert!(
            spouse_bits == 0 || people.spouse.is_some(),
            "the spouse page's items need a spouse"
        );
        for (person, bits) in [
            (Some(&mut people.taxpayer), bits & !spouse_bits),
            (people.spouse.as_mut(), spouse_bits),
        ] {
            if let Some(person) = person.filter(|_| bits != 0) {
                if on {
                    person.whole_return |= bits;
                } else {
                    person.whole_return &= !bits;
                }
            }
        }
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
        printed: None,
        refund_management: None,
        state_source: Default::default(),
        id,
        label: alloc::format!("Employer {id}"),
        owner: crate::DocumentOwner::from(owner),
        income: FederalIncome::Wage {
            data: wage_source(FederalWage {
                tips_without_w2: false,
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
            }),
        },
        community: CommunityShare::NotNeeded,
    }
}
// What a separate return's own pages ask: community property and the spouse's
// standard deduction boxes. A draft turned into a separate return answers them.
pub(in crate::tests) fn answer_separate_pages(d: &mut FederalDraft) {
    let setup = d.setup.as_mut().unwrap();
    setup.community_property = CommunityProperty::No;
    setup.separate_spouse = SeparateSpouseBoxes::NoneOfThese;
    if let Some(ded) = d.deductions.as_mut() {
        ded.spouse_itemizes = Some(false);
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
        // A separate return holds the answers its own pages ask; every other
        // status holds them unanswered.
        separate_spouse: if status == FederalStatus::Separate {
            SeparateSpouseBoxes::NoneOfThese
        } else {
            SeparateSpouseBoxes::NotAnswered
        },
        label: crate::common::test_label("Synthetic return"),
        basis: AmountBasis::Estimate,
        status: filing_choice(status),
        marital_state: if joint || status == FederalStatus::Separate {
            MaritalState::Married { living: SpouseLiving::Together }
        } else {
            MaritalState::Unmarried
        },
        paid_over_half_home: true,
        community_property: if status == FederalStatus::Separate {
            CommunityProperty::No
        } else {
            CommunityProperty::NotAnswered
        },
    });
    d.people = Some(FederalPeople {
        parent_tax: None,
        taxpayer: person(),
        spouse: (if joint {
            let mut spouse = person();
            spouse.first_name = "Jordan".into();
            Some(spouse)
        } else {
            None
        })
        .into(),
    });
    // This is a finished return: its existing W-2 was accepted in the Document
    // center, the people page was passed with nobody on it, and each section's
    // less-common page with "None of these".
    d.without_form = Some(IncomeWithoutFormAnswers::default());
    d.finished_sections = vec![
        TaxSection::Children,
        TaxSection::Documents,
        TaxSection::IncomeWithoutForm,
        TaxSection::Checklist,
        TaxSection::Income,
        TaxSection::MoreAdjustments,
        TaxSection::Adjustments,
        TaxSection::Deductions,
        TaxSection::MoreCredits,
        TaxSection::Care,
        TaxSection::Credits,
        TaxSection::OtherTaxes,
        TaxSection::Payments,
        TaxSection::OftenMissed,
    ];
    d.income = vec![wage(1, Owner::Taxpayer, if joint { 10_000_000 } else { 6_000_000 }, 650_000)];
    d.next_id = 2;
    d.adjustments = Some(FederalAdjustments {
        health_statements: vec![],
        health_contribution_allocations: vec![],

        social_premiums: Default::default(),
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
        vehicle_gift_documents: vec![],
        vehicle_gifts: vec![],
        vehicle_recoveries: Default::default(),
        investment_interest: None,
        mortgage: vec![],
        repayment: RepaymentClaim::None,
        itemize_override: None,
        itemized: crate::tests::support::builders::empty_facts(),
        spouse_itemizes: (status == FederalStatus::Separate).then_some(false),
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
        students: vec![],
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
        espp_acquisitions: Vec::new(),
        espp_sale_answers: Vec::new(),
        iso_sale_answers: Vec::new(),
        nua_sale_answers: Vec::new(),
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
        estimated: crate::EstimatedPaymentEntryRecord::None,
        extension: 0,
        home_kerosene: None,

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
    name_accepted_income_documents(&mut d);
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
        care: Some(DependentCare::No),
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
    v.related = vec![101];
}
pub(in crate::tests) fn add(d: &mut FederalDraft, mut e: FederalIncomeEntry) {
    e.id = d.next_id;
    d.next_id += 1;
    crate::interview::related_forms::prepare_component_ids(
        core::slice::from_mut(&mut e),
        &[],
        &mut d.next_id,
    )
    .unwrap();
    // A community-property return divides every item; a fixture says Community.
    if e.community == CommunityShare::NotNeeded
        && d.setup.as_ref().is_some_and(crate::community::applies)
    {
        e.community = CommunityShare::Community;
    }
    for id in crate::rental_energy_records::ids_mut(&mut e.income) {
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
        if data.more.allocations.is_empty() {
            data.more
                .allocations
                .push(crate::cooperative::automatic(&FederalCooperative::from(&*data)));
        }
        for allocation in &mut data.more.allocations {
            if allocation.id == 0 {
                allocation.id = d.next_id;
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
    let person = if e.owner == crate::DocumentOwner::Spouses { -2 } else { -1 };
    let names = fixture_document_names(d, person).map(|mut names| {
        names.set_assignment(crate::document_names::Assignment::matched(
            if e.owner == crate::DocumentOwner::Spouses {
                crate::DocumentRecipient::Spouse
            } else {
                crate::DocumentRecipient::Taxpayer
            },
        ));
        names
    });
    prepare_income_fixture(&mut e, names);
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
        printed: None,
        refund_management: None,
        state_source: Default::default(),
        id: 0,
        label: "Synthetic document".into(),
        owner: crate::DocumentOwner::Yours,
        income: value,
        community: CommunityShare::NotNeeded,
    }
}

// Every checklist topic ticked, so a prune keeps the pages and documents of a
// fixture that was not built through the checklist.
pub(in crate::tests) fn all_ticked(d: &mut FederalDraft) {
    d.checklist = Checklist { activities: u32::MAX, paid: u32::MAX, carryovers: u32::MAX };
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

pub(in crate::tests) fn dated_payments_mut(
    pay: &mut FederalPayments,
) -> &mut Vec<crate::DatedPayment> {
    if matches!(pay.estimated, crate::EstimatedPaymentEntryRecord::None) {
        pay.estimated = crate::EstimatedPaymentEntryRecord::Sources { payments: vec![] };
    }
    let crate::EstimatedPaymentEntryRecord::Sources { payments } = &mut pay.estimated else {
        panic!("dated source required")
    };
    payments
}
pub(in crate::tests) fn page_estimated_sources_mut(
    entry: &mut crate::EstimatedPaymentEntry,
) -> &mut Vec<crate::InjuredDatedPayment> {
    let crate::EstimatedPaymentEntry::Sources { payments } = entry else {
        panic!("dated source required")
    };
    payments
}
pub(in crate::tests) fn unknown_estimated(
    amount: i64,
    payer: crate::PaymentPayer,
) -> crate::EstimatedPaymentEntryRecord {
    if amount == 0 {
        crate::EstimatedPaymentEntryRecord::None
    } else {
        crate::EstimatedPaymentEntryRecord::TimingUnknown {
            payments: vec![crate::AnnualEstimatedAmountRecord {
                amount,
                payer,
                refund_property: None,
            }],
        }
    }
}
