use super::*;
use crate::tests::support::assert::accepted;
use crate::tests::support::client::income_pages::open_topic;

#[test]
fn unit17_foreign_employee_counts_pay_and_exclusion_once_in_both_systems() {
    // Reg.1.179-2(c)(1),(6)(iv) and Form2555: $200,000 employee pay,
    // less the $132,900 current exclusion, leaves $67,100 taxable pay.
    let mut draft = foreign_employee179_return(FederalStatus::Single, 100_000, 100_000);
    set_claim(&mut draft, foreign_claim(Owner::Taxpayer));
    for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
        let selected = crate::section179_return::prepare_with_home(
            &draft,
            system,
            &crate::home_office::HomeContext::default(),
        )
        .unwrap();
        assert_eq!(selected.allocation.income.business_income_limit_cents, 6_710_000);
        assert_eq!(selected.allocation.income.total_allowed_cents, 6_710_000);
        let mut result = FederalResult::empty(draft.revision);
        selected.report(&mut result).unwrap();
        assert!(result.issues.is_empty(), "{result:?}");
    }
    assert_eq!(crate::section179::calculate(&draft).income, 6_710_000);
    assert_eq!(value(&supported(&draft), "section179_business_income_limit"), 67_100);
    let mut session = Interview::new(TEST_USER, draft.clone());
    let initial = open_topic(&mut session, TaxSection::WorkAbroad);
    accepted(&send(&mut session, initial));
    let saved = session.draft.clone();
    session = restart(saved.clone());
    let mut correction = open_topic(&mut session, TaxSection::WorkAbroad);
    let P::SaveIncomes { documents, .. } = &mut correction else { unreachable!() };
    let pay = documents
        .documents
        .iter_mut()
        .find_map(|row| match &mut row.document {
            crate::IncomeRecord::ForeignPay(pay) => Some(pay),
            _ => None,
        })
        .unwrap();
    assert_eq!(pay.wages, 10_000_000);
    pay.wages = 5_000_000;
    accepted(&send(&mut session, correction));
    let corrected = supported(&session.draft);
    assert_eq!(value(&corrected, "section179_business_income_limit"), 17_100);
    assert_eq!(value(&corrected, "section179_carryforward"), 282_900);
    session = restart(session.draft.clone());
    let reopened = open_topic(&mut session, TaxSection::WorkAbroad);
    let P::SaveIncomes { documents, .. } = &reopened else { unreachable!() };
    assert!(documents.documents.iter().any(|row| {
        matches!(&row.document, crate::IncomeRecord::ForeignPay(pay) if pay.wages == 5_000_000)
    }));
    assert_eq!(estimate(&session.draft), corrected);
    draft.income.reverse();
    assert_eq!(crate::section179::calculate(&draft).income, 6_710_000);
}

#[test]
fn unit17_foreign_employee_no_election_revocation_and_native_rounding() {
    let mut draft = foreign_employee179_return(FederalStatus::Single, 0, 100_000);
    let FederalIncome::ForeignEmployment { data } = &mut draft.income[1].income else {
        unreachable!()
    };
    data.wages += 49;
    // The foreign income owner reports annual dollars, unlike W-2 source cents.
    assert_eq!(crate::section179::calculate(&draft).income, 10_000_000);
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.revocation = crate::ExclusionRevocation::Revoked {
        year: 2025,
        elections: crate::RevokedElections::Both,
        consent: false,
    };
    set_claim(&mut draft, claim);
    assert_eq!(crate::section179::foreign_employee_income_adjustment(&draft), Ok(10_000_000));
    claim_mut(&mut draft, Owner::Taxpayer).revocation = crate::ExclusionRevocation::None;
    assert_eq!(crate::section179::calculate(&draft).income, 0);
}

#[test]
fn unit17_foreign_employee_prior_service_awards_use_retained_capacity_once() {
    let mut draft = foreign_employee179_return(FederalStatus::Single, 200_000, 0);
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.prior_year_pay = 2_000_000;
    claim.prior_year_history =
        Some(PriorYearHistory { prior_year_days: 365, prior_year_excluded: 12_000_000 });
    claim.multi_year_pay.push(MultiYearPay {
        description: "Service award".into(),
        amount: 6_000_000,
        months_total: 36,
        months_in_return_year: 12,
        months_in_prior_year: 12,
    });
    set_claim(&mut draft, claim);
    // Current earned pay $140,000: exclusion $132,900. Prior pay $40,000
    // shares only $10,000 remaining 2025 exclusion capacity. Total $57,100.
    assert_eq!(crate::section179::calculate(&draft).income, 5_710_000);
    claim_mut(&mut draft, Owner::Taxpayer).qualification = ForeignQualification::PriorYearOnly;
    claim_mut(&mut draft, Owner::Taxpayer).multi_year_pay[0].months_in_return_year = 0;
    // The owner suppresses all current-year exclusion in a prior-only period.
    assert_eq!(crate::section179::calculate(&draft).income, 19_000_000);
}

#[test]
fn unit17_foreign_employee_housing_exclusion_and_prior_deduction_use_owner_amounts() {
    let mut draft = foreign_employee179_return(FederalStatus::Joint, 0, 200_000);
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.exclusion = false;
    claim.housing = housing_claim(
        30_000,
        ForeignHousingLocation::General,
        Some(HousingCarryover { amount: 500_000, deducted: 100_000 }),
    );
    set_claim(&mut draft, claim);
    // Housing $30,000 less the 16% base $21,264 = $8,736 excluded.
    // The separately retained $4,000 prior deduction leaves $187,264.
    assert_eq!(crate::section179::calculate(&draft).income, 18_726_400);
    let ForeignHousing::Claim { household, .. } =
        &mut claim_mut(&mut draft, Owner::Taxpayer).housing
    else {
        unreachable!()
    };
    *household = crate::ForeignHousingHousehold::Shared { total_expenses: 6_000_000 };
    assert!(matches!(
        crate::section179::foreign_employee_income_adjustment(&draft),
        Err(crate::Section179PreparedIssue::OwnerIssues(issues))
            if issues.iter().any(|issue| issue.display.key ==
                "ISSUE_USE_ONE_HOUSEHOLD_CLAIMANT_INCLUDING_A_CONFIRMED_ADVERSE_CONDITION_7185B7A1")
    ));
    let mut spouse = foreign_claim(Owner::Spouse);
    spouse.housing = housing_claim(30_000, ForeignHousingLocation::General, None);
    let ForeignHousing::Claim { household, .. } = &mut spouse.housing else { unreachable!() };
    *household = crate::ForeignHousingHousehold::Shared { total_expenses: 6_000_000 };
    set_claim(&mut draft, spouse);
    // The spouse has no employee source, but the real matching housing claim
    // still supplies the household validation; neither person's expenses vanish.
    assert_eq!(crate::section179::foreign_employee_income_adjustment(&draft), Ok(18_726_400));
}

#[test]
fn unit17_foreign_employee_other_person_business_keeps_sources_independent() {
    let mut draft = foreign_employee179_return(FederalStatus::Joint, 0, 200_000);
    set_claim(&mut draft, foreign_claim(Owner::Taxpayer));
    let mut business = income(foreign_service_business(10_000));
    business.owner = crate::DocumentOwner::Spouses;
    add(&mut draft, business);
    let business_id = draft.income.last().unwrap().id;
    assert_eq!(crate::section179::foreign_employee_income_adjustment(&draft), Ok(6_710_000));
    // No spouse exclusion: the separate $10,000 business source is taxable.
    assert_eq!(crate::section179::calculate(&draft).income, 7_710_000);
    set_claim(&mut draft, foreign_claim(Owner::Spouse));
    // The employee source stays known; this business's exclusion has its own
    // selected-expense, half-SE and HOME dependency for the aggregate limit.
    assert_eq!(crate::section179::foreign_employee_income_adjustment(&draft), Ok(6_710_000));
    assert!(matches!(
        crate::section179_return::prepare_with_home(
            &draft,
            Section179IncomeSystem::Regular,
            &crate::home_office::HomeContext::default(),
        ),
        Err(crate::section179_return::Issue::Prepared(
            crate::Section179PreparedIssue::SourceBoundaryUnavailable(id, key)
        )) if id == business_id && key.ends_with("F72A67AD")
    ));
    draft.income.last_mut().unwrap().owner = crate::DocumentOwner::Yours;
    assert!(matches!(
        crate::section179::foreign_employee_income_adjustment(&draft),
        Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(id, key))
            if id == business_id && key.ends_with("F72A67AD")
    ));
}

#[test]
fn unit17_foreign_employee_missing_pay_source_is_not_zero_income() {
    let mut draft = foreign_employee179_return(FederalStatus::Single, 0, 10_000);
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.prior_year_pay = 1_000_001;
    set_claim(&mut draft, claim);
    let mut result = FederalResult::empty(draft.revision);
    crate::section179::report(&draft, &mut result);
    assert!(
        result.issues.iter().any(|issue| issue.display.key == "ISSUE_FOREIGN_AWARDS_ABOVE_PAY")
    );
    assert!(!result.lines.iter().any(|line| line.id == "section179_business_income_limit"));
    let FederalIncome::ForeignEmployment { data } = &mut draft.income[1].income else {
        unreachable!()
    };
    data.wages = 0;
    assert_eq!(
        crate::section179::foreign_employee_income_adjustment(&draft),
        Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
            draft.income[1].id,
            "ISSUE_FOREIGN_AWARDS_ABOVE_PAY",
        )),
    );
    draft.screening.as_mut().unwrap().foreign_earned.clear();
    assert_eq!(crate::section179::foreign_employee_income_adjustment(&draft), Ok(0));
    add(&mut draft, income(foreign_service_business(0)));
    set_claim(&mut draft, foreign_claim(Owner::Taxpayer));
    let selected = crate::section179_return::prepare_with_home(
        &draft,
        Section179IncomeSystem::Regular,
        &crate::home_office::HomeContext::default(),
    )
    .unwrap();
    assert_eq!(selected.allocation.income.business_income_limit_cents, 0);
}
