// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn long_term_care_shares_age_cap_without_reserving_it_for_loss_businesses() {
    let mut d = complete(FederalStatus::Joint);
    set_age(&mut d, -1, 45);
    add(&mut d, income(care_business(0, 800, false)));
    let mut spouse = income(care_business(50000, 800, false));
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_health_insurance"), 0);
    assert_eq!(value(&r, "business_3_health_insurance"), 800);
    assert_eq!(value(&r, "long_term_care_1_medical"), 130);
    assert_eq!(value(&r, "long_term_care_1_excluded"), 670);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    match &mut data.activity {
        BusinessActivity::Ordinary { receipts, .. }
        | BusinessActivity::Gambling { receipts, .. } => *receipts = 5000000,
        _ => panic!("expected nonfarm test business"),
    };
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_health_insurance"), 800);
    assert_eq!(value(&r, "business_3_health_insurance"), 130);
    assert_eq!(value(&r, "long_term_care_1_medical"), 0);
}

#[test]
fn long_term_care_rejects_unknown_people_and_leaves_out_unconfirmed_ones() {
    let mut d = complete(FederalStatus::Single);
    set_age(&mut d, -1, 45);
    add(&mut d, income(care_business(50000, 1000, false)));
    // A second business whose policy names someone who is not on the return.
    let mut stranger = income(care_business(50000, 1000, false));
    let FederalIncome::Business { data } = &mut stranger.income else { panic!() };
    let BusinessHealthInsurance::Plan { long_term_care, .. } = &mut data.health_insurance else {
        panic!()
    };
    long_term_care[0].person = 99;
    add(&mut d, stranger);
    // A retained dangling insured identity is invalid accepted source data.
    assert!(!d.valid_stored());
    assert_eq!(estimate(&d).state, ResultState::Incomplete);
    d.income.pop();
    {
        let qualification = CareContractQualification::Nonqualified;
        let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
        let BusinessHealthInsurance::Plan { long_term_care, .. } = &mut data.health_insurance
        else {
            panic!()
        };
        long_term_care[0].qualified = qualification;
        // Left out with a notice; nothing is deducted for the premium.
        let r = supported(&d);
        assert!(noted(&r, "WARN_LTC_PREMIUM_LEFT_OUT"));
        assert!(!r.lines.iter().any(|l| l.id.starts_with("long_term_care_")));
    }
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    let BusinessHealthInsurance::Plan { long_term_care, .. } = &mut data.health_insurance else {
        panic!()
    };
    long_term_care[0].qualified = CareContractQualification::Qualified;
    let person = health_coverage_child(&mut d, 25);
    d.dependents.last_mut().unwrap().relationship = Relationship::Unrelated;
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    let BusinessHealthInsurance::Plan { long_term_care, .. } = &mut data.health_insurance else {
        panic!()
    };
    long_term_care[0].person = person;
    assert!(noted(&supported(&d), "WARN_LTC_PREMIUM_LEFT_OUT"));
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    let BusinessHealthInsurance::Plan { long_term_care, .. } = &mut data.health_insurance else {
        panic!()
    };
    let duplicate = long_term_care[0].months[0].clone();
    long_term_care[0].months.push(duplicate);
    assert!(!d.valid_stored());
}

#[test]
fn long_term_care_combines_three_sources_once_after_public_safety_exclusion() {
    let mut d = complete(FederalStatus::Single);
    set_age(&mut d, -1, 50);
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            10000,
            RetirementTreatment::Ordinary,
        ),
    );
    let pension = d.income.last().unwrap().id;
    public_safety(&mut d, pension, 5000, 3000, PublicSafetyCoverage::LongTermCare { person: -1 });
    add(&mut d, income(care_business(200, 1000, false)));
    let expenses = itemized(d.deductions.as_mut().unwrap());
    expenses.medical = 2000000;
    expenses.long_term_care = vec![personal_care(-1, 30000)];
    let r = supported(&d);
    assert_eq!(value(&r, "public_safety_1_excluded"), 3000);
    assert_eq!(value(&r, "business_3_health_insurance"), 200);
    assert_eq!(value(&r, "long_term_care_1_eligible"), 930);
    assert_eq!(value(&r, "long_term_care_1_medical"), 730);
    assert_eq!(value(&r, "long_term_care_1_excluded"), 2370);
    let mut reference = d.clone();
    reference.screening.as_mut().unwrap().public_safety[0].coverage = PublicSafetyCoverage::Other;
    reference.income[2].income = health_business(200, 200, false);
    let expenses = itemized(reference.deductions.as_mut().unwrap());
    expenses.long_term_care.clear();
    expenses.medical += 73000;
    let expected = supported(&reference);
    for id in ["agi", "deduction", "self_employment_tax", "qbi_deduction"] {
        assert_eq!(value(&r, id), value(&expected, id));
    }
    assert_eq!(r.tax, expected.tax);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn long_term_care_personal_and_public_safety_cents_round_only_after_aggregation() {
    let mut d = complete(FederalStatus::Single);
    set_age(&mut d, -1, 50);
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            10000,
            RetirementTreatment::Ordinary,
        ),
    );
    let pension = d.income.last().unwrap().id;
    public_safety(&mut d, pension, 1, 0, PublicSafetyCoverage::LongTermCare { person: -1 });
    d.screening.as_mut().unwrap().public_safety[0].premiums = 49;
    let expenses = itemized(d.deductions.as_mut().unwrap());
    expenses.medical = 2000000;
    expenses.long_term_care = vec![personal_care(-1, 49)];
    let r = supported(&d);
    assert_eq!(value(&r, "long_term_care_1_medical"), 1);
    let mut reference = d.clone();
    reference.screening.as_mut().unwrap().public_safety[0].coverage = PublicSafetyCoverage::Other;
    let expenses = itemized(reference.deductions.as_mut().unwrap());
    expenses.long_term_care.clear();
    expenses.medical += 98;
    assert_eq!(r.tax, supported(&reference).tax);
    assert_eq!(value(&r, "deduction"), value(&supported(&reference), "deduction"));
}

#[test]
fn hsa_required_gross_and_last_month_outputs_have_owner_failures() {
    let draft = complete(FederalStatus::Joint);
    let mut sources: [crate::medical_account::OwnedHsaAmounts; 2] = Default::default();
    sources[1].gross =
        crate::kernel::ratio::exact::whole(i64::MAX) * crate::kernel::ratio::exact::whole(2);
    sources[1].medical = sources[1].gross.clone();
    let mut result = FederalResult::empty(draft.revision);
    let error = match crate::hsa::calculate(
        &[],
        draft.people.as_ref().unwrap(),
        FederalStatus::Joint,
        &crate::hsa::ContributionContext {
            statements: &Default::default(),

            msa: Default::default(),
            failed: &Default::default(),
            married: true,
            family_share: None,
        },
        &sources,
        &draft.income,
        &mut result,
    ) {
        Err(error) => error,
        Ok(_) => panic!("required native output cannot fit"),
    };
    assert!(matches!(
        error.context.identity(),
        crate::federal_model::NoteIdentity::Owner {
            section: TaxSection::Hsa,
            owner: Owner::Spouse
        }
    ));
    error.report(&mut result);
    assert_eq!(result.state, ResultState::Unsupported);
    assert_eq!(result.issues.last().unwrap().display.key, "MEDICAL_REQUIRED_OUTPUT_UNAVAILABLE");
}

#[test]
fn archer_wide_compensation_reuses_limit_and_required_distribution_output() {
    let mut draft = complete(FederalStatus::Single);
    let mut contributor = archer_contributor(Owner::Taxpayer, archer_self(3000), 1000);
    contributor.income = draft.income.iter().map(|entry| entry.id).collect();
    set_archer(&mut draft, vec![contributor], ArcherSharing::Equal);
    let owner = archer_owner_mut(&mut draft);
    owner.personal = 0;
    owner.excess_accounting = Some(ArcherExcessAccounting {
        prior_year: Some(ArcherPriorExcess { line_40: 100, line_41: 6 }),
        year_end_value: 0,
        later_return_year_deposits: 0,
        later_employer_deposits: 0,
        employer_excess_in_wages: 0,
    });
    let withdrawals = [
        crate::kernel::ratio::exact::whole(i64::MAX) * crate::kernel::ratio::exact::whole(2),
        crate::kernel::ratio::exact::whole(0),
    ];
    assert!(matches!(
        crate::archer_contributions::calculate_with_section179(
            &draft,
            [0; 2],
            &Default::default(),
            withdrawals,
            None,
            &Default::default(),
            &mut FederalResult::empty(draft.revision),
            None,
        ),
        Err(crate::archer_contributions::CalculationIssue::Output(_))
    ));
}
