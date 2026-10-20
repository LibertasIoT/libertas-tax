// Independent approved Unit41 map fixtures: Pub517, ScheduleSE and Form8801.
use super::*;
use crate::tests::income::abroad_clergy::bonus_return;

#[test]
fn unit41_foreign_revocation_year_range_rejects_without_mutating_accepted_record() {
    use crate::tests::cross::interview::{accepted, document, from_review, rejected_with};
    let mut session = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
    let mut record = Form2555::from_record(&foreign_claim(Owner::Taxpayer));
    record.revocation = ExclusionRevocation::Revoked {
        year: 2021,
        elections: RevokedElections::Both,
        consent: false,
    };
    let Reply::Edit(mut page) = from_review(&mut session, TaxSection::WorkAbroad) else {
        panic!("Workabroad editor");
    };
    let P::SaveIncomes { documents, .. } = &mut page else {
        panic!("Workabroad save");
    };
    documents.documents = vec![document(0, IncomeDocument::Form2555(record))];
    accepted(&send(&mut session, page));
    let Reply::Edit(mut page) = from_review(&mut session, TaxSection::WorkAbroad) else {
        panic!("Workabroad reopen");
    };
    let P::SaveIncomes { documents, .. } = &mut page else {
        panic!("Workabroad save");
    };
    let IncomeDocument::Form2555(record) = &mut documents.documents[0].document else {
        panic!("Form2555");
    };
    record.revocation = ExclusionRevocation::Revoked {
        year: 2020,
        elections: RevokedElections::Both,
        consent: true,
    };
    let before = session.draft.clone();
    rejected_with(send(&mut session, page), "FOREIGN_REVOCATION_YEAR_OUTSIDE_PERIOD");
    assert_eq!(session.draft, before);
    let mut invalid = before;
    invalid.screening.as_mut().unwrap().foreign_earned[0].revocation =
        ExclusionRevocation::Revoked {
            year: 2026,
            elections: RevokedElections::Both,
            consent: true,
        };
    assert!(!invalid.valid_stored());
}

#[test]
fn unit41_residence_statement_withholds_only_its_owner_and_presence_can_qualify_instead() {
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, income(foreign_pay(50000)));
    let mut spouse = income(foreign_pay(40000));
    spouse.owner = Owner::Spouse;
    add(&mut d, spouse);
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    set_claim(&mut d, foreign_claim(Owner::Spouse));
    if let ForeignQualification::Residence { basis, nonresident_statement, .. } =
        &mut claim_mut(&mut d, Owner::Taxpayer).qualification
    {
        *basis = ResidenceBasis::TreatyNational;
        *nonresident_statement = true;
    }
    let blocked = supported(&d);
    assert_eq!(value(&blocked, "foreign_earned_exclusion"), 40000);
    let notes: Vec<_> = blocked
        .review_notes
        .iter()
        .filter(|n| n.display.key == "WARN_FOREIGN_RESIDENCE_STATEMENT")
        .collect();
    assert_eq!(notes.len(), 1);
    assert_eq!(
        notes[0].context,
        Some(IssueContext::owner(TaxSection::WorkAbroad, Owner::Taxpayer))
    );
    claim_mut(&mut d, Owner::Taxpayer).qualification =
        ForeignQualification::Presence { starts_on: 20260101, foreign_days: 330 };
    let present = supported(&d);
    assert_eq!(value(&present, "foreign_earned_exclusion"), 90000);
    assert!(!noted(&present, "WARN_FOREIGN_RESIDENCE_STATEMENT"));
}

#[test]
fn unit41_f2_award_years_and_prior_room_keep_original_monetary_oracles() {
    let d = bonus_return();
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_taxpayer_income"), 122500);
    assert_eq!(value(&r, "foreign_prior_year_taxpayer_exclusion"), 6219);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 128719);
    assert_eq!(value(&r, "agi"), 21281);
    assert_eq!(return_line(&r, "15"), 518100);
    assert_eq!(return_line(&r, "16"), 124300);
    assert_eq!(r.refund, Some(775700));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn unit41_prior_pay_and_awards_are_disjoint_and_optional_history_is_not_inferred() {
    let mut d = bonus_return();
    claim_mut(&mut d, Owner::Taxpayer).prior_year_history.as_mut().unwrap().prior_year_excluded = 0;
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_prior_year_taxpayer_exclusion"), 8750);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 131250);
    // Add an actual separate$1000 direct prior payment to source gross too.
    let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
    data.wages += 100000;
    data.abroad += 100000;
    claim_mut(&mut d, Owner::Taxpayer).prior_year_pay = 100000;
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_taxpayer_income"), 122500);
    assert_eq!(value(&r, "foreign_prior_year_taxpayer_exclusion"), 9750);
    claim_mut(&mut d, Owner::Taxpayer).prior_year_history = None;
    let r = supported(&d);
    assert!(!r.lines.iter().any(|l| l.id == "foreign_prior_year_taxpayer_exclusion"));
    // Declining history leaves the actual$1000 direct prior-pay split intact.
    // The award also still removes$27500 from current PartIV.
    assert_eq!(value(&r, "foreign_earned_taxpayer_income"), 122500);
}

#[test]
fn unit41_award_fractional_cents_and_bounds_never_require_an_inclusive_total() {
    let mut d = bonus_return();
    let claim = claim_mut(&mut d, Owner::Taxpayer);
    claim.prior_year_history =
        Some(PriorYearHistory { prior_year_days: 365, prior_year_excluded: 0 });
    claim.multi_year_pay = vec![MultiYearPay {
        description: "One cent".into(),
        amount: 1,
        months_total: 2,
        months_in_return_year: 1,
        months_in_prior_year: 1,
    }];
    // Exact half-cent service portions; one final dollar rounding at each line.
    assert_eq!(claim.employee_pay(1), (0, 0));
    let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
    data.wages = 1;
    data.abroad = 1;
    assert_eq!(crate::foreign_housing::source_pay_rule(&d), None);
    claim_mut(&mut d, Owner::Taxpayer).multi_year_pay[0].amount = 2;
    assert_eq!(crate::foreign_housing::source_pay_rule(&d), Some("ISSUE_FOREIGN_AWARDS_ABOVE_PAY"));
    let form = Form2555::from_record(claim_mut(&mut d, Owner::Taxpayer));
    let mut bad = form.clone();
    bad.multi_year_pay[0].months_in_prior_year = 2;
    assert_eq!(bad.rule(), Some("MULTI_YEAR_PAY_MONTHS_ABOVE_TOTAL"));
    // D-2026-10-15-13 (2026-10-16): a service period has no arbitrary
    // ten-year limit; its positive native denominator feeds the Exact owner.
    bad = form.clone();
    bad.multi_year_pay[0].months_total = i32::MAX;
    assert!(bad.rule().is_none());
    assert_eq!(bad.record(Owner::Taxpayer).employee_pay(1), (0, 0));
    bad.multi_year_pay[0].months_total = 0;
    assert_eq!(bad.rule(), Some("MULTI_YEAR_PAY_MONTHS_ABOVE_TOTAL"));
    bad = form;
    bad.prior_year_history.as_mut().unwrap().prior_year_days = 366;
    assert_eq!(bad.rule(), Some("PRIOR_YEAR_PAY_INVALID"));
}

#[test]
fn unit41_award_sources_are_owner_specific_and_order_independent() {
    let mut d = bonus_return();
    let before = supported(&d);
    let mut other = wage(0, Owner::Taxpayer, 0, 0);
    if let FederalIncome::Wage { data } = &mut other.income {
        data.wages = 5_000_000;
        data.abroad = 5_000_000;
    }
    add(&mut d, other);
    d.income.reverse();
    // Removing the added current source restores the same original figures,
    // regardless of which actual source holds the employee total.
    let FederalIncome::Wage { data } = &mut d.income[1].income else { unreachable!() };
    data.wages -= 5_000_000;
    data.abroad -= 5_000_000;
    let after = supported(&d);
    for id in [
        "foreign_earned_taxpayer_income",
        "foreign_prior_year_taxpayer_exclusion",
        "foreign_earned_exclusion",
        "agi",
    ] {
        assert_eq!(value(&before, id), value(&after, id), "{id}");
    }
    let mut joint = empty_return(FederalStatus::Joint);
    let mut pay = wage(1, Owner::Spouse, 15_000_000, 0);
    if let FederalIncome::Wage { data } = &mut pay.income {
        data.abroad = 15_000_000;
    }
    add(&mut joint, pay);
    set_claim(&mut joint, foreign_claim(Owner::Taxpayer));
    claim_mut(&mut joint, Owner::Taxpayer).multi_year_pay =
        claim_mut(&mut d, Owner::Taxpayer).multi_year_pay.clone();
    assert_eq!(
        crate::foreign_housing::source_pay_rule(&joint),
        Some("ISSUE_FOREIGN_AWARDS_ABOVE_PAY")
    );
}

#[test]
fn unit41_prior_exclusion_does_not_consume_current_housing_carryover_room() {
    let mut d = bonus_return();
    if let FederalIncome::Wage { data } = &mut d.income[0].income {
        data.wages = 17_000_000;
        data.abroad = 17_000_000;
    }
    claim_mut(&mut d, Owner::Taxpayer).housing = housing_claim(
        0,
        ForeignHousingLocation::General,
        Some(HousingCarryover { amount: 400000, deducted: 0 }),
    );
    // Current pay142500 minus current exclusion132900 leaves9600 of current
    // housing carryover capacity. Prior exclusion6219 must not consume it.
    let with_prior = supported(&d);
    assert_eq!(value(&with_prior, "foreign_prior_year_taxpayer_exclusion"), 6219);
    assert_eq!(value(&with_prior, "foreign_housing_deduction"), 4000);
    claim_mut(&mut d, Owner::Taxpayer).prior_year_history = None;
    let without_prior = supported(&d);
    assert_eq!(value(&without_prior, "foreign_housing_deduction"), 4000);
    assert_eq!(value(&without_prior, "foreign_earned_exclusion"), 132900);
    assert_eq!(value(&with_prior, "foreign_earned_exclusion"), 139119);
}

#[test]
fn unit41_totalization_credits_keep_profit_but_never_restore_other_exempt_services() {
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = cash_business(4000);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    data.source = BusinessSource::ForeignServices { confirmed: true, totalization_exempt: true };
    add(&mut d, income(entry));
    d.dependents.push(child(d.next_id));
    d.next_id += 1;
    // Approved R12 statutory treatment, not a claim of resolved IRS worksheet
    // administrative wording: EAS preserves ordinary taxable earned profit.
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "earned_income_credit"), 1369);
    assert_eq!(value(&r, "additional_child_credit"), 225);
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.notary_profit = 100000;
    let r = supported(&d);
    assert_eq!(value(&r, "additional_child_credit_earned_income"), 3000);
    assert_eq!(value(&r, "additional_child_credit"), 75);
    d.screening.as_mut().unwrap().clergy =
        vec![ClergyClaim { owner: Owner::Taxpayer, role: ClergyRole::ReligiousSect }];
    let r = supported(&d);
    assert_eq!(value(&r, "additional_child_credit_earned_income"), 0);
    d.screening.as_mut().unwrap().clergy[0].role = ClergyRole::Minister {
        housing: MinisterHousing::None,
        form_4361: true,
        employee_expenses: 0,
    };
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.ministerial = true;
    let r = supported(&d);
    assert_eq!(value(&r, "additional_child_credit_earned_income"), 0);
}

#[test]
fn unit41_prior_exclusion_is_not_cancelled_by_current_foreign_business_replacement() {
    let mut d = bonus_return();
    add(&mut d, income(excluded_foreign_business(20000, 0)));
    let with_prior = supported(&d);
    assert_eq!(value(&with_prior, "foreign_prior_year_taxpayer_exclusion"), 6219);
    claim_mut(&mut d, Owner::Taxpayer).prior_year_history = None;
    let without_prior = supported(&d);
    assert_eq!(
        value(&with_prior, "foreign_earned_exclusion")
            - value(&without_prior, "foreign_earned_exclusion"),
        6219
    );
    assert_eq!(value(&without_prior, "agi") - value(&with_prior, "agi"), 6219);
    assert_eq!(
        value(&with_prior, "foreign_business_taxpayer_disallowed_deductions"),
        value(&without_prior, "foreign_business_taxpayer_disallowed_deductions")
    );
}

#[test]
fn unit41_prior_award_exclusion_respects_its_independent_revocation() {
    let mut d = bonus_return();
    claim_mut(&mut d, Owner::Taxpayer).revocation = ExclusionRevocation::Revoked {
        elections: RevokedElections::EarnedIncome,
        year: 2025,
        consent: false,
    };
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 0);
    assert!(!r.lines.iter().any(|l| l.id == "foreign_prior_year_taxpayer_exclusion"));
    assert_eq!(value(&r, "agi"), 150000);
    claim_mut(&mut d, Owner::Taxpayer).revocation = ExclusionRevocation::None;
    assert_eq!(value(&supported(&d), "foreign_prior_year_taxpayer_exclusion"), 6219);
}
