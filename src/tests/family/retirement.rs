use super::*;

#[test]
fn student_identity_survives_roster_reordering_and_a_removed_person_keeps_the_source_for_correction()
 {
    let mut d = complete(FederalStatus::Joint);
    add_child(&mut d);
    add_child(&mut d);
    let second = d.dependents[1].id;
    let sid3_242 = sid(&d, 3);
    add_education(
        d.credits.as_mut().unwrap(),
        native_education(
            sid3_242,
            education_expenses(100_000),
            education_choice(EducationMethod::LifetimeLearning, Answer::No),
        ),
    );
    let before = supported(&d);
    d.dependents.remove(0);
    // The row names its person by identity; the catalog is read from the people
    // page, so nothing stored has to follow the reordering.
    assert_eq!(d.credits.as_ref().unwrap().education[0].student, second);
    assert!(crate::federal_rules::people_students(&d).iter().any(|s| s.id == second));
    assert_eq!(value(&supported(&d), "education_credit"), value(&before, "education_credit"));
    // The physical tuition source retains its original person identity after
    // household removal until the Document center rechecks its ownership.
    let sources = d.credits.as_ref().unwrap().education.clone();
    let mut annual = d.credits.as_ref().unwrap().students[0].clone();
    let household = d.clone();
    d.dependents.clear();
    all_ticked(&mut d);
    let pruned = crate::interview::prune(&mut d);
    assert_eq!(d.credits.as_ref().unwrap().education[0].student, second);
    assert_eq!(d.credits.as_ref().unwrap().education.len(), 1);
    assert_eq!(d.credits.as_ref().unwrap().education, sources);
    assert!(
        !pruned.removed.iter().any(|item| item.key() == "SAVING_KIND_EDUCATION_CREDIT"),
        "{pruned:?}"
    );

    // The normal household save must preserve the same source and permit a
    // cold restart while its student needs correction.
    let mut session = restart(household);
    let Reply::Edit(mut removal @ P::SaveChildren { .. }) =
        crate::tests::cross::interview::from_review(&mut session, TaxSection::Children)
    else {
        panic!("the actual household editor")
    };
    let P::SaveChildren { dependents, .. } = &mut removal else { unreachable!() };
    dependents.people.clear();
    crate::tests::cross::interview::accepted(&send(&mut session, removal));
    assert!(session.draft.dependents.is_empty());
    assert_eq!(session.draft.credits.as_ref().unwrap().education, sources);
    assert_eq!(session.draft.credits.as_ref().unwrap().students, vec![annual.clone()]);
    assert!(session.draft.valid_stored());
    let retained = session.draft.clone();
    session = restart(retained.clone());
    assert_eq!(session.draft, retained);
    let missing = estimate(&session.draft);
    assert_eq!(missing.state, ResultState::Incomplete);
    assert_eq!(
        (missing.tax, missing.payments, missing.refund, missing.balance),
        (None, None, None, None)
    );
    assert!(missing.lines.is_empty());
    let issue = missing
        .issues
        .iter()
        .find(|issue| {
            issue.display.key == "MESSAGE_A_STUDENT_IS_NOT_ON_THIS_RETURN_CHOOSE_THE_STUDENT_AGAIN"
        })
        .expect("the retained source must identify its unavailable student");
    assert_eq!(
        crate::interview::issue_pages(&session.draft, &issue.context, None),
        vec![TaxSection::Documents]
    );
    let center = crate::tests::support::client::income_pages::open_document_center(&mut session);
    // 2026-10-29: save the intact paper with Unknown ownership, then assign
    // it in the Document center before answering Education questions.
    let reply = send(&mut session, center);
    let Reply::Response(P::Review { remaining_items, .. }) = reply else {
        panic!("expected the pending-owner review, got {reply:?}")
    };
    assert!(remaining_items.iter().any(|item| item.section == TaxSection::Documents));
    let credits = session.draft.credits.as_ref().unwrap();
    assert_eq!(credits.education.len(), 1);
    let source_id = credits.education[0].id;
    assert_ne!(source_id, 0);
    assert_eq!(credits.education[0].form, sources[0].form);
    assert_eq!(credits.education[0].student, 0);
    assert_eq!(credits.education[0].assigned, crate::DocumentRecipient::Unknown);
    assert_eq!(credits.education[0].owner_source, None);
    assert!(credits.students.is_empty());
    assert!(session.draft.valid_stored());
    let pending = session.draft.clone();
    session = restart(pending.clone());
    assert_eq!(session.draft, pending);
    let missing = estimate(&session.draft);
    assert_eq!(missing.state, ResultState::Incomplete);
    assert_eq!(
        (missing.tax, missing.payments, missing.refund, missing.balance),
        (None, None, None, None)
    );
    assert!(missing.lines.is_empty());
    let issue = missing
        .issues
        .iter()
        .find(|issue| issue.display.key == "ISSUE_WHOSE_DOCUMENT_IS_THIS")
        .expect("the retained source must keep ownership pending");
    assert_eq!(
        crate::interview::issue_pages(&session.draft, &issue.context, None),
        vec![TaxSection::Documents]
    );
    let education =
        crate::tests::cross::interview::open_credits_topic(&mut session, TaxSection::Education);
    let P::SaveEducation { value: education, .. } = education else { unreachable!() };
    assert_eq!(education.documents.len(), 1);
    assert_eq!(education.documents[0].student, None);
    assert!(education.students.is_empty());
    let corrected = crate::tests::support::builders::added_documents_request(
        &mut session,
        TaxSection::Documents,
        -1,
        &[source_id],
        crate::AddedOwnership::Yours,
    );
    let P::SaveDocuments { document_owners, .. } = &corrected else { unreachable!() };
    let taxpayer = document_owners.iter().find(|person| person.person == -1).unwrap();
    assert!(taxpayer.choices.iter().any(|choice| choice.id == source_id));
    crate::tests::cross::interview::accepted(&send(&mut session, corrected));
    assert!(session.draft.valid_stored());
    let credits = session.draft.credits.as_ref().unwrap();
    assert_eq!(credits.education[0].id, source_id);
    assert_eq!(credits.education[0].form, sources[0].form);
    assert_eq!(credits.education[0].student, -1);
    assert_eq!(credits.education[0].assigned, crate::DocumentRecipient::Taxpayer);
    assert_eq!(credits.education[0].owner_source, Some(crate::OwnerSource::Added));
    assert_eq!(credits.students.len(), 1);
    assert_eq!(credits.students[0].student, -1);
    assert_eq!(credits.students[0].method, None);
    annual.student = -1;
    crate::tests::cross::interview::accepted(
        &crate::tests::cross::interview::save_education_answers(&mut session, vec![annual]),
    );
    assert_eq!(
        session.draft.credits.as_ref().unwrap().education[0].form.box_1,
        sources[0].form.box_1
    );
    assert_eq!(
        value(&supported(&session.draft), "education_credit"),
        value(&before, "education_credit")
    );
    let corrected = session.draft.clone();
    session = restart(corrected.clone());
    assert_eq!(session.draft, corrected);
    let mut deletion =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut deletion else { unreachable!() };
    documents.common.education.students.clear();
    crate::tests::cross::interview::accepted(&send(&mut session, deletion));
    assert!(session.draft.valid_stored());
    assert!(session.draft.credits.as_ref().unwrap().education.is_empty());
    assert!(session.draft.credits.as_ref().unwrap().students.is_empty());
    assert_eq!(value(&supported(&session.draft), "education_credit"), 0);
}

#[test]
fn taxable_alimony_is_ira_compensation_but_not_eic_earnings() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Alimony { data: alimony_record(20181231) }));
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 7500);
    assert_eq!(value(&r, "agi"), 2500);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    d.income[0].income = FederalIncome::Alimony { data: alimony_record(20190101) };
    unsupported(&d, "IRA");
}

#[test]
fn alimony_recapture_supplies_decree_related_ira_compensation_and_unknown_history_is_blocked() {
    let mut d = empty_return(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().alimony_paid = vec![paid_alimony(recapture_alimony())];
    d.adjustments.as_mut().unwrap().iras = ira_facts(100000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 1000);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    d.adjustments.as_mut().unwrap().alimony_paid[0].payment.decree = false;
    unsupported(&d, "IRA");
    d.adjustments.as_mut().unwrap().iras = FederalIras::NONE;
    let AlimonyRecapture::ThirdYear { death_or_remarriage, .. } =
        &mut d.adjustments.as_mut().unwrap().alimony_paid[0].payment.recapture
    else {
        unreachable!()
    };
    // Payments that ended with a death or remarriage recapture nothing.
    *death_or_remarriage = true;
    assert_eq!(value(&supported(&d), "alimony_recapture_income"), 0);
}
#[test]
fn dependent_care_plan_exclusion_coordinates_credit_expenses_and_return_wages() {
    let mut d = benefit_return(FederalStatus::Single);
    let r = supported(&d);
    assert_eq!(value(&r, "care_benefits_excluded"), 5000);
    assert_eq!(value(&r, "care_credit"), 350); // 6,000 cap less 5,000 excluded.
    assert_eq!(return_line(&r, "1a"), 6000000);
    assert_eq!(return_line(&r, "1e"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    care_box10(&mut d, 0, 750000);
    assert_eq!(value(&supported(&d), "care_credit"), 0);
}
#[test]
fn dependent_care_plan_limit_taxable_excess_and_existing_wages_are_not_duplicated() {
    let mut d = benefit_return(FederalStatus::Single);
    care_box10(&mut d, 0, 700000);
    d.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::Plans { maximum: 500000 };
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 62000);
    assert_eq!(return_line(&r, "1e"), 200000);
    assert_eq!(return_line(&r, "1z"), 6200000);
    d.credits.as_mut().unwrap().care.benefits[0].in_box_1 = Some(200000);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(return_line(&r, "1e"), 0);
}

#[test]
fn able_saver_shares_retirement_limit_and_distribution_lookback() {
    use crate::federal_rules::saver_potential;
    let d = complete(FederalStatus::Single);
    let p = d.people.as_ref().unwrap();
    let mut f = saver(100_000, 0);
    f.taxpayer_able = 150_000;
    assert_eq!(saver_potential(&f, p, FederalStatus::Single, 20000), Ok(1000));
    f.distributions.push(FederalSaverDistribution {
        owner: Owner::Taxpayer,
        amount: 100_000,
        year: SaverDistributionPeriod::TaxYear,
    });
    assert_eq!(saver_potential(&f, p, FederalStatus::Single, 20000), Ok(750));
    f.taxpayer_able = 0;
    assert_eq!(saver_potential(&f, p, FederalStatus::Single, 20000), Ok(0));
    f.spouse_able = 1;
    assert!(saver_potential(&f, p, FederalStatus::Single, 20000).is_err());
}

#[test]
fn birth_adoption_limits_are_separate_for_each_parent_and_child() {
    let mut d = complete(FederalStatus::Joint);
    for (owner, child) in
        [(Owner::Taxpayer, "Child A"), (Owner::Spouse, "Child A"), (Owner::Taxpayer, "Child B")]
    {
        add(&mut d, birth_withdrawal(owner, child, 5000, 0));
    }
    let r = supported(&d);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "birth_adoption_used_0_Child A"), 5000);
    assert_eq!(value(&r, "birth_adoption_used_1_Child A"), 5000);
    assert_eq!(value(&r, "birth_adoption_used_0_Child B"), 5000);
    add(&mut d, birth_withdrawal(Owner::Taxpayer, "Child A", 1000, 100));
    unsupported(&d, "Income");
}

#[test]
fn separated_filer_care_credit_uses_existing_home_and_living_facts() {
    let mut d = complete(FederalStatus::Separate);
    d.income[0] = wage(1, Owner::Taxpayer, 8000000, 0);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::LastSixMonths };
    d.setup.as_mut().unwrap().paid_over_half_home = true;
    add_child(&mut d);
    d.dependents[0].set_care_expenses(300000);
    assert_eq!(
        value(
            &{
                document_care(&mut d);
                supported(&d)
            },
            "care_credit"
        ),
        960
    ); // 32% nonjoint rate.
    assert_eq!(restart(d.clone()).draft, d);
    d.setup.as_mut().unwrap().paid_over_half_home = false;
    assert_eq!(
        value(
            &{
                document_care(&mut d);
                supported(&d)
            },
            "care_credit"
        ),
        0
    );
    d.setup.as_mut().unwrap().paid_over_half_home = true;
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::Together };
    assert_eq!(
        value(
            &{
                document_care(&mut d);
                supported(&d)
            },
            "care_credit"
        ),
        0
    );
}

#[test]
fn separated_care_benefits_use_unmarried_limit_and_coordinate_credit() {
    let mut d = benefit_return(FederalStatus::Separate);
    d.income[0] = wage(1, Owner::Taxpayer, 8000000, 0);
    care_box10(&mut d, 0, 500_000);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::LastSixMonths };
    d.setup.as_mut().unwrap().paid_over_half_home = true;
    let r = supported(&d);
    assert_eq!(value(&r, "care_credit"), 320); // $6,000 ceiling minus $5,000 excluded.
    assert_eq!(return_line(&r, "1e"), 0);
    assert_eq!(value(&r, "agi"), 80000);
    care_box10(&mut d, 0, 800000);
    let r = supported(&d);
    assert_eq!(return_line(&r, "1e"), 50000); // $7,500 limit, not $3,750.
    assert_eq!(value(&r, "care_credit"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    // Not considered unmarried: the spouse's earnings were not entered, so
    // they count as zero and nothing is excluded, with a warning.
    d.setup.as_mut().unwrap().paid_over_half_home = false;
    let r = supported(&d);
    assert!(noted(&r, "WARN_CARE_SPOUSE_EARNINGS_ASSUMED_ZERO"), "{r:?}");
    assert_eq!(value(&r, "care_benefits_excluded"), 0);
    // Not known: the same, the conservative path with the warning.
    d.credits.as_mut().unwrap().care.filing =
        CareFilingStatus::Separate { spouse_earned_income: None };
    assert_eq!(supported(&d), r);
}

#[test]
fn separate_care_benefits_apply_spouse_earnings_and_half_limit_without_adding_spouse_income() {
    for (spouse_wages, taxable) in [(0, 5000), (300000, 2000), (10000000, 1250)] {
        let mut d = benefit_return(FederalStatus::Separate);
        d.income[0] = wage(1, Owner::Taxpayer, 8000000, 0);
        care_box10(&mut d, 0, 500000);
        d.credits.as_mut().unwrap().care.filing =
            CareFilingStatus::Separate { spouse_earned_income: Some(spouse_wages) };
        let r = supported(&d);
        assert_eq!(return_line(&r, "1e"), taxable * 100);
        assert_eq!(value(&r, "agi"), 80000 + taxable);
        assert_eq!(value(&r, "care_credit"), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn separate_care_unknown_spouse_income_counts_zero_and_honors_plan_limit() {
    let mut d = benefit_return(FederalStatus::Separate);
    d.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::Plans { maximum: 200000 };
    d.credits.as_mut().unwrap().care.filing =
        CareFilingStatus::Separate { spouse_earned_income: None };
    let r = supported(&d);
    assert_eq!(value(&r, "care_benefits_excluded"), 0);
    assert!(noted(&r, "WARN_CARE_SPOUSE_EARNINGS_ASSUMED_ZERO"));
    d.credits.as_mut().unwrap().care.filing =
        CareFilingStatus::Separate { spouse_earned_income: Some(500000) };
    assert_eq!(return_line(&supported(&d), "1e"), 300000);
}

#[test]
fn care_plan_excess_already_in_w2_remains_earned_income_without_double_tax() {
    let mut d = benefit_return(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 100000, 0);
    care_w2(&mut d, 1, 150000, 50000, 0, 100000);
    let r = supported(&d);
    assert_eq!(value(&r, "care_benefits_excluded"), 1000);
    assert_eq!(value(&r, "care_benefits_taxable"), 0);
    assert_eq!(value(&r, "agi"), 1000);
    assert_eq!(value(&r, "care_earned_taxpayer"), 1000);
}

#[test]
fn separated_spouse_eic_uses_nonjoint_rates_without_home_cost_test() {
    let mut d = complete(FederalStatus::Separate);
    d.income[0] = wage(1, Owner::Taxpayer, 2000000, 0);
    add_child(&mut d);
    d.setup.as_mut().unwrap().paid_over_half_home = false;
    for living in
        [SpouseLiving::LastSixMonths, SpouseLiving::AllYear, SpouseLiving::AgreementAtYearEnd]
    {
        d.setup.as_mut().unwrap().marital_state = MaritalState::Married { living };
        let r = supported(&d);
        assert_eq!(value(&r, "earned_income_credit"), 4427);
        assert_eq!(restart(d.clone()).draft, d);
    }
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::Together };
    assert_eq!(value(&supported(&d), "earned_income_credit"), 0);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::AllYear };
    d.dependents.clear();
    assert_eq!(value(&supported(&d), "earned_income_credit"), 0);
}

#[test]
fn separated_spouse_eic_preserves_residence_and_ssn_requirements() {
    let mut d = complete(FederalStatus::Separate);
    d.income[0] = wage(1, Owner::Taxpayer, 2000000, 0);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::LastSixMonths };
    add_child(&mut d);
    d.dependents[0].residency = DependentResidency::Days { days: 182 };
    assert_eq!(value(&supported(&d), "earned_income_credit"), 0);
    d.dependents[0].residency = DependentResidency::Days { days: 183 };
    assert_eq!(value(&supported(&d), "earned_income_credit"), 4427);
    d.people.as_mut().unwrap().taxpayer.valid_ssn = false;
    assert_eq!(value(&supported(&d), "earned_income_credit"), 0);
}

#[test]
fn written_separation_eic_does_not_grant_care_unmarried_status() {
    let mut d = complete(FederalStatus::Separate);
    d.income[0] = wage(1, Owner::Taxpayer, 2000000, 0);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::AgreementAtYearEnd };
    d.setup.as_mut().unwrap().paid_over_half_home = true;
    add_child(&mut d);
    d.dependents[0].set_care_expenses(300000);
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&r, "earned_income_credit"), 4427);
    assert_eq!(value(&r, "care_credit"), 0);
    assert!(!d.setup.as_ref().unwrap().lived_apart_last_six_months());
    assert!(!d.setup.as_ref().unwrap().lived_apart_all_year());
}
#[test]
fn licensed_care_facility_residence_combines_with_service_suspension() {
    let mut d = complete(FederalStatus::Single);
    let mut home = home_sale_source(false);
    let closed = home.form.paper().unwrap().box_1_date_of_closing;
    let answers = home.facts_mut();
    answers.acquired = crate::dates::add_days(closed, -(1826 + 800));
    answers.lived = HomeLived::Days { days: 0 };
    answers.service = Some(HomeService {
        election: HomeServiceElection {
            service: HomeDutyKind::Uniformed {
                duty: QualifiedHomeDuty { extended_orders: true, remote_or_quarters: true },
            },
            one_property: true,
            suspended_days: 2192,
        },
        added_lived: 365,
        added_care: Some(crate::home_sale::HomeServiceCare {
            facility_days: 365,
            owned_residence_days: 365,
        }),
        spouse_added_lived: None,
        spouse_added_care: None,
    });
    add(&mut d, income(FederalIncome::HomeSale { data: home }));
    assert_eq!(value(&supported(&d), "capital_gain"), 300000);
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::HomeSale { data } = &mut d.income[1].income else { panic!() };
    let answers = data.facts_mut();
    answers.service.as_mut().unwrap().added_care.as_mut().unwrap().owned_residence_days = 364;
    unsupported(&d, "Income:");
    let FederalIncome::HomeSale { data } = &mut d.income[1].income else { panic!() };
    let answers = data.facts_mut();
    let care = answers.service.as_mut().unwrap().added_care.as_mut().unwrap();
    care.owned_residence_days = 365;
    care.facility_days = 2193;
    assert!(!d.valid_stored());
    let FederalIncome::HomeSale { data } = &mut d.income[1].income else { panic!() };
    let closed = data.form().closed;
    let answers = data.facts_mut();
    answers.acquired = crate::dates::add_days(closed, -(1826 + 700));
    answers.service.as_mut().unwrap().added_care.as_mut().unwrap().facility_days = 365;
    assert!(!d.valid_stored());
}

#[test]
fn two_children_have_separate_code_t_exclusions_and_credit_expenses() {
    let d = two_child_joint_adoption_return();
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_child_1_employer_payment_1"), 6000);
    assert_eq!(value(&r, "adoption_child_1_employer_payment_2"), 4000);
    assert_eq!(value(&r, "adoption_child_2_employer_payment_1"), 5000);
    assert_eq!(value(&r, "adoption_child_2_employer_payment_2"), 3000);
    assert_eq!(value(&r, "adoption_1_exclusion_limit"), 17_670);
    assert_eq!(value(&r, "adoption_2_exclusion_limit"), 17_670);
    assert_eq!(value(&r, "adoption_child_1_benefits_excluded"), 10_000);
    assert_eq!(value(&r, "adoption_child_2_benefits_excluded"), 8000);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 18_000); // Per-child limits, not one joint $17,670 cap.
    assert_eq!(value(&r, "adoption_exclusion_magi"), 138_000);
    assert_eq!(value(&r, "adoption_wage_adjustment"), 0);
    assert_eq!(value(&r, "adoption_refundable_credit"), 8120); // $5,120 + $3,000 from distinct expenses.
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 2880);
    assert_eq!(return_line(&r, "1a"), 12_000_000);
    assert_eq!(return_line(&r, "1f"), 0);
    assert_eq!(return_line(&r, "25a"), 1_200_000);
    assert_eq!(value(&r, "agi"), 120_000);
    let restored = restart(d.clone()).draft;
    assert_eq!(restored, d);
    assert_eq!(supported(&restored).return_lines, r.return_lines);
}

#[test]
fn two_children_one_cap_and_both_phaseouts_keep_child_limits_separate() {
    let mut d = two_child_joint_adoption_return();
    let AdoptionBenefits::JointEmployees { payments, .. } =
        &mut d.credits.as_mut().unwrap().adoptions[1].benefits
    else {
        unreachable!()
    };
    payments[0].received = 1_600_000; // Second child's $19,000 gross binds at $17,670.
    set_joint_adoption_w2(&mut d, 0, 23_408_000, 600_000);
    set_joint_adoption_w2(&mut d, 1, 100_000, 400_000);
    set_joint_adoption_w2(&mut d, 2, 2_000_000, 1_600_000);
    set_joint_adoption_w2(&mut d, 3, 100_000, 300_000);
    set_joint_adoption_exclusion_for_child(&mut d, 0, 1000, 4000);
    set_joint_adoption_exclusion_for_child(&mut d, 1, 5835, 3000);
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_exclusion_magi"), 285_080);
    assert_eq!(value(&r, "adoption_child_1_benefits_excluded"), 5000);
    assert_eq!(value(&r, "adoption_child_2_benefits_excluded"), 8835);
    assert_eq!(value(&r, "adoption_child_2_benefits_taxable"), 10_165);
    assert_eq!(value(&r, "adoption_child_2_employer_taxable_taxpayer"), 10_165);
    assert_eq!(value(&r, "adoption_child_2_employer_taxable_spouse"), 0);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 13_835);
    assert_eq!(value(&r, "adoption_wage_adjustment"), 15_165);
    assert_eq!(return_line(&r, "1a"), 25_608_000);
    assert_eq!(return_line(&r, "1f"), 1_516_500);
    assert_eq!(return_line(&r, "25a"), 1_200_000);
    assert_eq!(value(&r, "agi"), 271_245);
    let restored = restart(d.clone()).draft;
    assert_eq!(restored, d);
    assert_eq!(supported(&restored).return_lines, r.return_lines);
}

#[test]
fn adoption_exclusion_is_added_back_for_ira_and_rental_limits() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 8100000, 0);
    employer_adoption(&mut d, 10000, 0, false);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::Yes);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 0);
    assert_eq!(value(&r, "adoption_exclusion_magi"), 91000);
    assert_eq!(value(&r, "agi"), 81000);
    d.adjustments.as_mut().unwrap().iras = FederalIras::NONE;
    d.income[0] = covered_wage(1, Owner::Taxpayer, 11000000, 0);
    add(&mut d, income(FederalIncome::Rental { data: rental(9000, 30000) }));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), -15000);
    assert_eq!(value(&r, "agi"), 95000);
}

#[test]
fn adoption_benefits_reject_unresolved_plan_and_invalid_stored_sources() {
    let mut d = complete(FederalStatus::Single);
    employer_adoption(&mut d, 10000, 0, false);
    let before = supported(&d);
    // Unconfirmed plan facts: the benefits stay taxable wages.
    {
        let (answer, key) = (false, "WARN_ADOPTION_BENEFITS_LEFT_OUT");
        if let AdoptionBenefits::Program { ordinary, .. } =
            &mut d.credits.as_mut().unwrap().adoptions[0].benefits
        {
            *ordinary = answer;
        }
        let r = supported(&d);
        assert!(noted(&r, key), "{r:?}");
        assert_eq!(value(&r, "adoption_benefits_excluded"), 0);
        assert_eq!(value(&r, "agi"), value(&before, "agi") + 10000);
    }
}

#[test]
fn separated_adoption_exception_uses_home_and_residence_for_credit_and_exclusion() {
    let d = separated_adoption_return();
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 17670);
    assert_eq!(return_line(&r, "1f"), 233000);
    assert_eq!(value(&r, "adoption_refundable_credit"), 5120);
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 880);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn separated_adoption_failure_returns_taxable_benefits_and_no_new_credit() {
    for condition in 0..3 {
        let mut d = separated_adoption_return();
        match condition {
            0 => d.setup.as_mut().unwrap().paid_over_half_home = false,
            1 => {
                d.setup.as_mut().unwrap().marital_state =
                    MaritalState::Married { living: SpouseLiving::AgreementAtYearEnd }
            }
            _ => d.credits.as_mut().unwrap().adoptions[0].home_over_half_year = false,
        }
        let r = supported(&d);
        assert_eq!(value(&r, "adoption_benefits_excluded"), 0);
        assert_eq!(return_line(&r, "1f"), 2000000);
        assert_eq!(value(&r, "adoption_refundable_credit"), 0);
        assert_eq!(value(&r, "adoption_nonrefundable_credit"), 0);
        assert!(
            r.review_notes.iter().any(|n| n.contains("living-apart, home-cost or child-residence"))
        );
        assert_eq!(
            r.review_notes
                .iter()
                .filter(|note| {
                    let message = &note.display;
                    message.key == "NOTE_ADOPTION_SEPARATE_RETURN_EXCEPTION_NOT_MET"
                })
                .count(),
            1,
        );
    }
}

#[test]
fn separated_adoption_child_outside_the_home_gets_no_current_claim() {
    let mut d = separated_adoption_return();
    d.credits.as_mut().unwrap().adoptions[0].home_over_half_year = false;
    let r = supported(&d);
    assert!(noted(&r, "NOTE_ADOPTION_SEPARATE_RETURN_EXCEPTION_NOT_MET"), "{r:?}");
    assert_eq!(value(&r, "adoption_benefits_excluded"), 0);
    d.setup.as_mut().unwrap().paid_over_half_home = false;
    assert_eq!(value(&supported(&d), "adoption_benefits_excluded"), 0);
    d.setup.as_mut().unwrap().paid_over_half_home = true;
    let child = &mut d.credits.as_mut().unwrap().adoptions[0];
    child.kind =
        AdoptionKind::Domestic { finalized: AdoptionFinalization::Pending, special_needs: false };
    child.benefits = AdoptionBenefits::None;
    // Only a 2026 expense for a pending domestic adoption: no current claim.
    assert_eq!(value(&supported(&d), "adoption_refundable_credit"), 0);
}

#[test]
fn separated_adoption_residence_is_evaluated_for_each_child() {
    let mut d = separated_adoption_return();
    let mut second = d.credits.as_ref().unwrap().adoptions[0].clone();
    second.label = "Second synthetic child".into();
    second.home_over_half_year = false;
    d.credits.as_mut().unwrap().adoptions.push(second);
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 17670);
    assert_eq!(return_line(&r, "1f"), 2233000);
    assert_eq!(value(&r, "adoption_refundable_credit"), 5120);
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 880);
}

#[test]
fn adoption_shared_limits_do_not_override_separate_return_requirements() {
    let mut d = separated_adoption_return();
    d.credits.as_mut().unwrap().adoptions[0].limits =
        AdoptionLimits::Shared { credit: 600000, exclusion: 400000, agreed: true };
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 4000);
    assert_eq!(value(&r, "adoption_refundable_credit"), 5120);
    assert_eq!(return_line(&r, "1f"), 1600000);
    d.credits.as_mut().unwrap().adoptions[0].home_over_half_year = false;
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_refundable_credit"), 0);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 0);
}

#[test]
fn able_taxable_earnings_are_unearned_not_wages_or_ira_compensation() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![income(able_distribution(10000, 5000, 0))];
    d.income[0].id = 1;
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    let r = supported(&d);
    assert_eq!(value(&r, "deduction"), 1350);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(value(&r, "account_additional_tax"), 500);
    assert_eq!(return_line(&r, "1z"), 0);
}

#[test]
fn able_corrections_do_not_dilute_annual_expenses_or_ordinary_penalty() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, income(able_distribution(2400, 400, 1600)));
    add(&mut d, income(able_correction(2026, 300)));
    let mut spouse = income(able_correction(2026, 200));
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    let r = supported(&d);
    assert_eq!(value(&r, "able_taxable_distributions"), 633);
    assert_eq!(value(&r, "able_corrective_earnings"), 500);
    assert_eq!(value(&r, "account_additional_tax"), 13);
    assert_eq!(return_line(&r, "8"), 63300);
    assert_eq!(return_line(&r, "23"), 1300);
}

#[test]
fn able_retirement_plan_eligibility_controls_only_additional_allowance() {
    let mut d = complete(FederalStatus::Single);
    let mut entry = able_contributions(8000, 20000, 30000);
    entry.work = AbleWork::Working {
        compensation: 1000000,
        residence: MarketplaceResidence::Contiguous,
        no_plan: false,
    };
    d.screening.as_mut().unwrap().able_contributions = vec![entry];
    assert_eq!(value(&supported(&d), "able_taxpayer_excess_contributions"), 8000);
    d.screening.as_mut().unwrap().able_contributions[0].other = 0;
    assert_eq!(value(&supported(&d), "able_taxpayer_excess_tax"), 0);
}

#[test]
fn able_partial_rollover_removes_its_own_earnings_before_expense_proration() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(mixed_able(4000, 800, 3000)));
    let r = supported(&d);
    // Remaining withdrawals: 6,000 gross and 2,200 earnings.
    // Expenses cover half, so 1,100 is taxable, rather than using the
    // unrelated aggregate 30% earnings ratio for the rolled payment.
    assert_eq!(value(&r, "able_taxable_distributions"), 1100);
    assert_eq!(value(&r, "account_additional_tax"), 110);
    assert_eq!(value(&r, "agi"), 61100);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn able_partial_rollover_expense_boundaries_and_full_rollover() {
    for (amount, earnings, expenses, taxable) in
        [(4000, 800, 0, 2200), (4000, 800, 6000, 0), (10000, 3000, 0, 0)]
    {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(mixed_able(amount, earnings, expenses)));
        assert_eq!(value(&supported(&d), "able_taxable_distributions"), taxable);
    }
}

#[test]
fn household_futa_quarter_test_aggregates_workers_not_annual_pay() {
    let mut e = household_employer(500);
    e.workers.push(e.workers[0].clone());
    e.workers[1].label = "Second worker".into();
    let r = supported(&with_household(e));
    assert_eq!(value(&r, "household_0_social_tax"), 0);
    assert_eq!(value(&r, "household_0_futa_tax"), 6);
}

#[test]
fn household_disability_wages_count_toward_exact_annual_fica_threshold() {
    let mut e = household_employer(1000);
    let mut notice = household_disability(2000, 0, 0);
    reported_sick_pay(&mut notice)[0].cash -= 1;
    e.workers[0].disability = Some(notice);
    let r = supported(&with_household(e.clone()));
    assert_eq!(value(&r, "household_0_social_tax"), 0);
    assert_eq!(value(&r, "household_0_medicare_tax"), 0);
    assert_eq!(r.tax, Some(504100));
    e.workers[0].disability = Some(household_disability(2000, 124, 29));
    let r = supported(&with_household(e));
    assert_eq!(value(&r, "household_0_social_tax"), 372);
    assert_eq!(value(&r, "household_0_medicare_tax"), 87);
    assert_eq!(value(&r, "household_0_state_disability_tax_offset"), 153);
    assert_eq!(r.tax, Some(534700));
}

#[test]
fn temporary_child_annuity_uses_and_carries_unused_exclusion() {
    let d = child_annuity_draft();
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 150000);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(value(&r, "child_annuity_2_income_exclusion"), 1500);
    assert_eq!(value(&r, "child_annuity_2_deferred_exclusion"), 300);
    assert_eq!(value(&r, "child_annuity_2_remaining_cost"), 300);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    let mut exhausted = d;
    {
        let FederalIncome::Retirement { data } = &mut exhausted.income[1].income else {
            unreachable!()
        };
        let mut data = edit_retirement(data);
        data.gross = 240000;
        let Some(RetirementTaxable::ChildAnnuity { data }) = &mut data.taxable else {
            unreachable!()
        };
        for payment in &mut data.payments {
            payment.amount = 20000;
        }
    }
    let r = supported(&exhausted);
    assert_eq!(return_line(&r, "5b"), 60000);
    assert_eq!(value(&r, "child_annuity_2_deferred_exclusion"), 0);
    assert_eq!(value(&r, "child_annuity_2_remaining_cost"), 0);
}

#[test]
fn unconfirmed_child_annuity_is_taxed_in_full() {
    let mut d = child_annuity_draft();
    {
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        let mut data = edit_retirement(data);
        let Some(RetirementTaxable::ChildAnnuity { data }) = &mut data.taxable else {
            unreachable!()
        };
        data.confirmed = false;
    }
    let r = supported(&d);
    assert!(noted(&r, "WARN_RETIREMENT_BASIS_LEFT_OUT"));
    assert_eq!(return_line(&r, "5b"), return_line(&r, "5a"));
}

#[test]
fn temporary_child_annuity_rejects_wrong_term_history_and_treatment() {
    let mut d = child_annuity_draft();
    d.people.as_mut().unwrap().taxpayer.birth_date = 20040101;
    unsupported(&d, "Income:");
    d.people.as_mut().unwrap().taxpayer.birth_date = 20050101;
    {
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        let mut data = edit_retirement(data);
        data.ordinary_distribution = Some(RetirementTreatment::Ordinary);
    }
    unsupported(&d, "Income:");
    {
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        let mut data = edit_retirement(data);
        let Some(RetirementTaxable::ChildAnnuity { data }) = &mut data.taxable else {
            unreachable!()
        };
        data.deferred += 1;
    }
    assert!(!d.valid_stored());
}

#[test]
fn temporary_child_annuity_handles_new_year_start_zero_payments_and_final_months() {
    let mut p = ChildAnnuity {
        ended: None,
        started: 20260701,
        term: 18,
        cost: 180000,
        recovered: 0,
        deferred: 0,
        payments: [0, 0, 30000, 0, 20000, 10000]
            .into_iter()
            .map(|amount| ChildAnnuityPayment { amount })
            .collect(),
        confirmed: true,
    };
    assert!(p.valid(60000));
    assert!(p.matches_birth(20060101));
    assert_eq!(p.result(), (60000, 0, 120000));
    for payment in &mut p.payments {
        payment.amount = 0;
    }
    assert!(p.valid(0));
    assert_eq!(p.result(), (0, 60000, 180000));
    p.payments.pop();
    assert!(!p.valid(0));
    p.started = 20250501;
    p.term = 12;
    p.cost = 120000;
    p.recovered = 40000;
    p.deferred = 40000;
    p.payments = (0..4).map(|_| ChildAnnuityPayment { amount: 20000 }).collect();
    assert!(p.valid(80000));
    assert!(p.matches_birth(20040501));
    assert_eq!(p.result(), (80000, 0, 0));
}

#[test]
fn disabled_child_survivor_uses_lifetime_pension_not_disability_wages() {
    for shared in [false, true] {
        let mut d = complete(FederalStatus::Single);
        let mut parent = ordinary_parent();
        parent.status = FederalStatus::Single;
        d.people.as_mut().unwrap().parent_tax = Some(parent);
        let person = &mut d.people.as_mut().unwrap().taxpayer;
        person.birth_date = 20110101;
        person.disabled = true;
        let mut p = pension_basis();
        p.cost = Some(3600000);
        p.history = PensionHistory::New { duration: PensionDuration::Single { age: 15 } };
        if shared {
            p.share = Some(PensionShare { preceding: 0, payment: 50000, total: 100000 });
        }
        let mut entry = pension_entry(p, 6000);
        {
            let FederalIncome::Retirement { data } = &mut entry.income else { unreachable!() };
            let mut data = edit_retirement(data);
            data.ordinary_distribution = Some(retirement_exception(
                20261201,
                RetirementException::Death { died: 20251231 },
                false,
            ));
        }
        add(&mut d, entry);
        let r = supported(&d);
        assert_eq!(return_line(&r, "5b"), if shared { 540000 } else { 480000 });
        assert_eq!(return_line(&r, "1h"), 0);
        assert_eq!(value(&r, "early_distribution_tax"), 0);
        assert_eq!(value(&r, "pension_2_remaining_basis"), 34800);
        assert_eq!(supported(&restart(d).draft), r);
    }
}

#[test]
fn temporary_child_annuity_early_termination_preserves_original_divisor() {
    let mut d = child_annuity_draft();
    {
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        let mut data = edit_retirement(data);
        data.gross = 30000;
        let Some(RetirementTaxable::ChildAnnuity { data }) = &mut data.taxable else {
            unreachable!()
        };
        data.ended = Some(20260630);
        data.payments.truncate(6);
        assert_eq!(data.monthly(), 10000);
        assert!(data.valid(30000));
        assert_eq!(data.result(), (30000, 90000, 150000));
    }
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 30000);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(value(&r, "child_annuity_2_deferred_exclusion"), 900);
    assert_eq!(value(&r, "child_annuity_2_remaining_cost"), 1500);
    assert_eq!(value(&r, "agi"), 60000); // No taxable pension income is added.
    assert_eq!(supported(&restart(d.clone()).draft), r);
    {
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        let mut data = edit_retirement(data);
        let Some(RetirementTaxable::ChildAnnuity { data }) = &mut data.taxable else {
            unreachable!()
        };
        data.ended = None;
        assert!(!data.valid(30000)); // Missing months need the source termination date.
        data.ended = Some(20250630);
        assert!(!data.valid(30000));
        data.started = 20260701;
        data.recovered = 0;
        data.deferred = 0;
        data.ended = Some(20260630);
        assert!(!data.valid(30000));
    }
}

#[test]
fn final_child_annuitant_deduction_links_periodic_recovery_and_matching_cost() {
    let mut d = child_annuity_draft();
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { date: 20260630, bonds: None, ordinary: true, estate_medical: 0 });
    {
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        let mut data = edit_retirement(data);
        data.gross = 3000000;
        data.form_boxes.payment_date = Some(20260601);
        if let Some(RetirementTreatment::Exception { distributed, .. }) =
            &mut data.ordinary_distribution
        {
            *distributed = 20260601;
        }
        let Some(RetirementTaxable::ChildAnnuity { data }) = &mut data.taxable else {
            unreachable!()
        };
        data.ended = Some(20260630);
        data.cost *= 100;
        data.recovered *= 100;
        data.deferred *= 100;
        data.payments.truncate(6);
        for payment in &mut data.payments {
            payment.amount *= 100;
        }
    }
    d.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
        income_id: 2,
        started: 20250101,
        cost: 24000000,
        recovered: 6000000,
        refunds: 0,
        eligible: true,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "annuity_loss_2"), 150000);
    assert_eq!(value(&r, "unrecovered_annuity_deduction"), 150000);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.deductions.as_mut().unwrap().annuity_losses[0].cost += 100;
    unsupported(&d, "Annuity loss:");
    d.deductions.as_mut().unwrap().annuity_losses[0].cost -= 100;
    {
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        let mut data = edit_retirement(data);
        data.taxable = Some(RetirementTaxable::CivilServiceDeath {
            data: crate::SurvivorLumpSum { cost: 24000000, recovered: 6000000, eligible: true },
        });
    }
    unsupported(&d, "Annuity loss:"); // A beneficiary's lump sum is not this annuity.
}

#[test]
fn trump_account_excess_and_returned_earnings_use_separate_form_5329_taxes() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20090704;
    let baseline = supported(&d);
    d.screening.as_mut().unwrap().trump_accounts.push(TrumpAccount {
        owner: Owner::Taxpayer,
        contributions: 720_000,
        exempt: 120_000,
        returns: vec![TrumpExcessReturn {
            document_id: None,
            principal: 60_000,
            earnings: 2_000,
            received: 20261201,
            deadline: 20270415,
            confirmed: true,
        }],
        confirmed: true,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "trump_account_taxpayer_line57_excess"), 1_000);
    assert_eq!(value(&r, "trump_account_taxpayer_line59_returned"), 600);
    assert_eq!(value(&r, "trump_account_taxpayer_line60_remaining"), 400);
    assert_eq!(value(&r, "trump_account_taxpayer_line61_tax"), 24);
    assert_eq!(value(&r, "trump_account_taxpayer_line62_earnings"), 20);
    assert_eq!(value(&r, "trump_account_taxpayer_line63_tax"), 20);
    assert_eq!(value(&r, "agi"), value(&baseline, "agi"));
    assert_eq!(value(&r, "chapter_one_net_tax"), value(&baseline, "chapter_one_net_tax") + 20);
    assert_eq!(value(&r, "total_tax"), value(&baseline, "total_tax") + 44);
    assert_eq!(estimate(&restart(d.clone()).draft), r);

    d.screening.as_mut().unwrap().trump_accounts[0].returns.push(TrumpExcessReturn {
        document_id: None,
        principal: 40_000,
        earnings: 3_000,
        received: 20270401,
        deadline: 20270415,
        confirmed: true,
    });
    let later = supported(&d);
    assert_eq!(value(&later, "trump_account_taxpayer_line60_remaining"), 0);
    assert_eq!(value(&later, "trump_account_taxpayer_line61_tax"), 0);
    assert_eq!(value(&later, "trump_account_taxpayer_line63_tax"), 20);
    assert_eq!(value(&later, "total_tax"), value(&baseline, "total_tax") + 20);

    d.screening.as_mut().unwrap().trump_accounts[0].returns[1].received = 20270416;
    let late = supported(&d);
    assert_eq!(value(&late, "trump_account_taxpayer_line59_returned"), 600);
    assert_eq!(value(&late, "trump_account_taxpayer_line60_remaining"), 400);
    assert_eq!(value(&late, "trump_account_taxpayer_line61_tax"), 24);
    assert_eq!(value(&late, "trump_account_taxpayer_line63_tax"), 20);
    assert!(late.review_notes.iter().any(|note| note.contains("after the 2026-return deadline")));
    d.screening.as_mut().unwrap().trump_accounts[0].returns.pop();
    d.people.as_mut().unwrap().taxpayer.birth_date = 20080101;
    unsupported(&d, "Trump account");
}

#[test]
fn full_trump_to_trump_transfer_carries_deposits_without_a_second_annual_limit() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20180615;
    let baseline = supported(&d);
    // $5,000 first-account cash plus $200 growth moves in full to a successor
    // account. It then accepts $1,000 more cash. The $5,200 incoming transfer
    // is both an accepted and exempt contribution, not another $5,200 of cash.
    d.screening.as_mut().unwrap().trump_accounts.push(TrumpAccount {
        owner: Owner::Taxpayer,
        contributions: 1_120_000,
        exempt: 520_000,
        returns: vec![],
        confirmed: true,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "trump_account_taxpayer_line57_excess"), 1_000);
    assert_eq!(value(&r, "trump_account_taxpayer_line61_tax"), 60);
    assert_eq!(value(&r, "agi"), value(&baseline, "agi"));
    assert_eq!(value(&r, "total_tax"), value(&baseline, "total_tax") + 60);
    assert_eq!(estimate(&restart(d.clone()).draft), r);
}
