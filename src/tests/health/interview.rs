// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::support::client::unit50_interview_native::NativeInterviewCase;
use super::*;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::document;
use super::support::client::income_pages::{open_topic, save_topic};
use crate::tests::cross::interview::{
    PAGE_CHILDREN, accept_care_provider, adult, all_year, assert_round_trip,
    at_the_adjustments_hub, at_the_adjustments_hub_with_business, at_the_deductions_page,
    at_the_marketplace_page, benefits_request, credits_hub, education_value, family_options, go_on,
    health_credit_business, hsa_account, hsa_options, hsa_page_value, hub_of, hub_rows,
    itemized_hub, marketplace_fixture_page, marketplace_page_of, open_adjustments_hub,
    open_benefits, open_credits_topic, open_hsa_page, open_itemized_topic, progress_of, r1099,
    reopened_people, row, save_benefits, save_businesses, save_checklist, save_children,
    save_deductions, save_education, save_hsa, save_marketplace, save_marketplace_and_reopen,
    save_medical, save_setup, save_you, se_health_test_year, send_past_less_common, shown,
    stored_business, stored_facts, stored_marketplace, suggestion, supporting, value_of,
    w2_with_box_12_w,
};

#[test]
fn hsa_options_does_not_read_the_split() {
    // The calculation's request carries the accounts and the owners it needs
    // to resolve their constraint, and nothing else: an exhaustive pattern
    // stops compiling if the split or its allowed set is added as an input.
    let P::HsaOptions {
        user: _,
        hsa_failed_deposits: _,
        archer_failed_deposits: _,
        hsa_return_documents: _,
        archer_return_documents: _,
        allowed_owners: _,
        accounts: _,
        other_coverage: _,
        family_method: _,
    } = hsa_options(vec![])
    else {
        unreachable!()
    };
    // Its reply writes only the split's allowed set.
    let P::HsaFamilyOptions { allowed_family_split: _ } =
        (P::HsaFamilyOptions { allowed_family_split: vec![] })
    else {
        unreachable!()
    };
    // The same accounts give the same set whatever split the page holds or
    // the accepted return stores; the running estimate is what reads the
    // split, downstream.
    let mut s = at_the_adjustments_hub(FederalStatus::Joint, 0);
    let two = vec![
        hsa_account(Owner::Taxpayer, HsaCoverageYear::Family, 400_000),
        hsa_account(Owner::Spouse, HsaCoverageYear::Family, 400_000),
    ];
    let before = family_options(send(&mut s, hsa_options(two.clone())));
    assert_eq!(before, vec![1, 2]);
    let request = save_hsa(&s.draft, two.clone(), FamilySplit::Agreed { you: 1, spouse: 0 });
    accepted(&send(&mut s, request));
    assert_eq!(family_options(send(&mut s, hsa_options(two.clone()))), before);
    let estimate_with = |s: &mut Interview, family_split| {
        let reply = send(
            s,
            P::EstimateHsa {
                user: TEST_USER,
                hsa_failed_deposits: vec![],
                archer_failed_deposits: vec![],
                hsa_return_documents: vec![],
                archer_return_documents: vec![],
                other_coverage_allowed: false,
                allowed_owners: vec![0, 1],
                allowed_family_split: before.clone(),
                ira_sources: vec![],
                archer_income: vec![],
                value: HsaPage {
                    employer_sources: vec![],
                    linked_deposits: vec![],
                    other_coverage: None,
                    family_method: HsaAllocationMethod::Monthly,
                    accounts: two.clone(),
                    family_split,
                    funding: vec![],
                    archer: ArcherContributionsPage::None,
                },
            },
        );
        let Reply::Response(P::RunningEstimate { estimate, .. }) = reply else {
            panic!("expected the running estimate, got {reply:?}")
        };
        shown(&estimate)
    };
    // By hand: $4,000 each against the $8,750 family limit. Split equally,
    // each spouse's $4,375 share covers the deposit; split 1:0 the spouse
    // has no room, so the spouse's $4,000 is an excess the page does not
    // account for and the estimate stops.
    let equal = estimate_with(&mut s, FamilySplit::Equal);
    let one_to_zero = estimate_with(&mut s, FamilySplit::Agreed { you: 1, spouse: 0 });
    assert!(!equal.contains("Estimate not available"), "{equal}");
    assert!(one_to_zero.contains("Estimate not available"), "{one_to_zero}");
}

#[test]
fn long_term_care_premiums_follow_their_person() {
    // A joint return's premium for the spouse goes when the return stops
    // being joint (the spouse leaves the people catalog); yours stays.
    let (mut s, _) = at_the_deductions_page(FederalStatus::Joint);
    let request = save_deductions(&s.draft, DeductionRoute::Compare, 0);
    itemized_hub(send_past_less_common(&mut s, request));
    let P::SaveMedical { value, .. } = open_itemized_topic(&mut s, TaxSection::Medical) else {
        panic!("expected the medical page")
    };
    let labels: Vec<_> = value.people.iter().map(|p| status_text(&p.label)).collect();
    assert_eq!(labels, vec!["You", "Your spouse"]);
    let request = save_medical(
        &s.draft,
        Medical {
            expenses: 0,
            long_term_care: vec![
                CarePremium {
                    qualified: CareContractQualification::Qualified,
                    person: -1,
                    premiums: 50_000,
                },
                CarePremium {
                    qualified: CareContractQualification::Qualified,
                    person: -2,
                    premiums: 70_000,
                },
            ],
            people: value.people.clone(),
        },
    );
    itemized_hub(send_past_less_common(&mut s, request));
    assert_eq!(stored_facts(&s).long_term_care.len(), 2);
    // A stale key: a premium naming a person who is not in the catalog is
    // refused, and the stored premiums are unchanged.
    let request = save_medical(
        &s.draft,
        Medical {
            expenses: 0,
            long_term_care: vec![CarePremium {
                qualified: CareContractQualification::Qualified,
                person: -3,
                premiums: 50_000,
            }],
            people: value.people.clone(),
        },
    );
    error(current_source_refusal(
        "secondary-admission-2493",
        &mut s,
        request,
        "MESSAGE_A_PREMIUM_NAMES_SOMEONE_WHO_IS_NOT_ON_THIS_RETURN_CHOOSE_THE_INSURED_PERSON",
        Some("[value/long_term_care/0/person]"),
        SourceScope::Secondary(SecondaryCorrection::MedicalSource),
    ));
    assert_eq!(stored_facts(&s).long_term_care.len(), 2);
    // A stale catalog is refused rather than misread.
    let request =
        save_medical(&s.draft, Medical { expenses: 0, long_term_care: vec![], people: vec![] });
    error(current_source_refusal(
        "secondary-admission-2494",
        &mut s,
        request,
        "MESSAGE_THE_PEOPLE_ON_THIS_RETURN_CHANGED_REOPEN_THIS_PAGE_BEFORE_SAVING",
        None,
        SourceScope::Fieldless,
    ));
    let request = save_setup(&s.draft, ReturnSetup { filing: Filing::Single });
    send(&mut s, request);
    let premiums = stored_facts(&s).long_term_care;
    assert_eq!(premiums.len(), 1);
    assert_eq!(premiums[0].person, -1);
    assert_eq!(premiums[0].premiums, 50_000);
}

#[test]
fn derived_hsa_and_education_are_walked_without_a_tick() {
    // Nothing is ticked, but a W-2 reports box 12 W with no account and you
    // are a full-time student: the adjustments section walks the HSA page
    // and the credits section the education page, each opened directly.
    let mut d = complete(FederalStatus::Single);
    d.finished_sections = vec![TaxSection::Children, TaxSection::Checklist, TaxSection::Income];
    d.people.as_mut().unwrap().taxpayer.full_time_student = true;
    let mut s = Interview::new(TEST_USER, d);
    let request = save_topic(
        &s.draft,
        TaxSection::Wages,
        vec![document(0, IncomeDocument::W2(w2_with_box_12_w(6_000_000, 100_000)))],
    );
    accepted(&send(&mut s, request));
    assert_eq!(crate::interview::planned(&s.draft), Checklist::NONE);
    let Reply::Edit(page @ P::SaveHsa { .. }) = send(&mut s, P::OpenInterview { user: TEST_USER })
    else {
        panic!("expected the HSA page")
    };
    let (value, _) = hsa_page_value(&page);
    assert!(value.accounts.is_empty());
    // Saving the account walks on to More adjustments, whose Next ends the
    // adjustments walk (gate fixes batch A): the summary continues to the
    // credits section, which the student makes necessary.
    let request = save_hsa(
        &s.draft,
        vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 200_000)],
        FamilySplit::NotApplicable,
    );
    let reply = send_past_less_common(&mut s, request);
    assert!(progress_of(&reply).contains("· You're done with adjustments to income"));
    let hub = hub_of(reply);
    assert_eq!(suggestion(&hub), TaxSection::Credits);
    let Reply::Edit(page @ P::SaveEducation { .. }) = send(&mut s, go_on(&hub)) else {
        panic!("expected the education page")
    };
    // The deductions section, with nothing for this return, was passed.
    assert!(s.draft.finished_sections.contains(&TaxSection::Deductions));
    let request = save_education(
        &s.draft,
        EducationCredits { students: vec![], people: education_value(&page).people.clone() },
    );
    let reply = send_past_less_common(&mut s, request);
    assert!(progress_of(&reply).contains("· You're done with credits"));
    credits_hub(reply);
    assert_eq!(crate::interview::planned(&s.draft), Checklist::NONE);
}

#[test]
fn marketplace_actual_child_cost_source_survives_save_and_restart() {
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    let child = &mut d.dependents[0];
    child.use_for = PersonUse::HealthCoverageOnly;
    child.birth_date = 20000102;
    child.situation =
        DependentSituation::Supported { your_support_over_half: false, gross_income: 0 };
    child.facts = PersonFacts::OWN_SUPPORT.0;
    let child_id = child.id;
    let (mut session, _) = at_the_marketplace_page(d);
    let mut value = marketplace_fixture_page(vec![all_year("Issuer", 100_000, 60_000, 0)]);
    for coverage in &mut value.forms[0].coverage {
        coverage.people.push(child_id);
    }
    value.memberships.push(MarketplacePersonFamilyPage {
        person: child_id,
        family: MarketplaceNonreturnFamilyPage::NoFamily,
    });
    value.attestations.push(MarketplaceAttestationPage {
        person: child_id,
        attester: MarketplaceAttesterPage::ThisReturn,
    });
    for month in 1..=12 {
        value.eligibility.push(MarketplaceEligibilityMonthPage {
            month,
            person: child_id,
            status: MarketplacePersonEligibility::OtherMec {
                kind: MarketplaceMecKind::EmployerEmployee,
            },
        });
        value.benchmarks.push(MarketplaceBenchmarkMonthPage {
            month,
            groups: vec![MarketplaceBenchmarkGroupPage {
                state: TaxState::CA,
                rating_area: "Actual eligible-parent area".into(),
                people: vec![-1],
                source: MarketplaceBenchmarkSourcePage::Marketplace { amount: 60000 },
            }],
            allocated: vec![],
        });
    }
    value.forms[0].medical_costs.push(MarketplaceMedicalCostMonth {
        month: 1,
        child_increase: None,
        tax_free_child: None,
    });
    let mut value = save_marketplace_and_reopen(&mut session, value);
    assert_eq!(value.forms[0].medical_costs[0].child_increase, None);
    value.forms[0].medical_costs[0].child_increase = Some(0);
    let mut value = save_marketplace_and_reopen(&mut session, value);
    assert_eq!(value.forms[0].medical_costs[0].child_increase, Some(0));
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
    let before = session.draft.clone();
    value.forms[0].medical_costs[0].child_increase = Some(25_000);
    let response = send(
        &mut session,
        P::EstimateMarketplace {
            user: TEST_USER,
            people: vec![],
            ptc_people: vec![],
            ptc_families: vec![],
            ptc_policies: vec![],
            ptc_w2: vec![],
            ptc_employees: vec![],
            ptc_nonreturn_people: vec![],
            ptc_claimed_dependents: vec![],
            allowed_marriage: vec![],
            allowed_separate: vec![],
            value: value.clone(),
        },
    );
    assert!(matches!(response, Reply::Response(P::RunningEstimate { .. })));
    assert_eq!(session.draft, before);
    // Actual source cost is $1,000: the server rejects excess cost even
    // though the echoed page permits ordinary money values this large.
    value.forms[0].medical_costs[0].child_increase = Some(100_001);
    let request = save_marketplace(&before, value);
    // Current typed Page-to-record conversion retains the stored-value locator,
    // and the earlier native cookie guard still wins when both are invalid.
    let P::SaveMarketplace { value, .. } = &request else { unreachable!() };
    let mut candidate = before.clone();
    candidate.credits.as_mut().unwrap().marketplace =
        crate::MarketplaceClaim::Yes { facts: alloc::boxed::Box::new(value.clone().into()) };
    let issue = candidate.stored_issue().unwrap();
    assert_eq!(issue.key, "ISSUE_INTERVIEW_REVIEW_THE_ACCEPTED_PAGE_VALUES_BEFORE_65B6D7B7");
    assert_eq!(
        issue.field.unwrap().square(),
        "[credits/marketplace/forms/0/medical_costs/0/child_increase]"
    );
    candidate.cookie.clear();
    let issue = candidate.stored_issue().unwrap();
    assert_eq!(issue.key, "ISSUE_INTERVIEW_REVIEW_THE_ACCEPTED_PAGE_VALUES_BEFORE_65B6D7B7");
    assert_eq!(issue.field, None);
    error(current_source_refusal(
        "health-context-11",
        &mut session,
        request,
        "ISSUE_INTERVIEW_REVIEW_THE_ACCEPTED_PAGE_VALUES_BEFORE_65B6D7B7",
        Some("[value/forms/0/medical_costs/0/child_increase]"),
        SourceScope::CommitOnly,
    ));
    assert_eq!(session.draft, before);
}

#[test]
fn marketplace_page_reproduces_the_row_27_fixture() {
    // 2026 i8962 arithmetic: $31,300 / $15,650=200%; contribution
    // $31,300*.066=$2,066; annual6000/7200 => credit5134, advance6000,
    // full uncapped repayment866. Explicit fixture coverage is taxpayer-only.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    let (mut s, empty) = at_the_marketplace_page(d);
    assert!(empty.forms.is_empty());
    let value = marketplace_fixture_page(vec![all_year("Issuer", 50_000, 60_000, 50_000)]);
    let saved = save_marketplace_and_reopen(&mut s, value.clone());
    assert!(saved.forms[0].id > 0);
    let mut expected = value;
    expected.forms[0].id = saved.forms[0].id;
    assert_eq!(saved, expected);
    let r = supported(&s.draft);
    assert_eq!(value_of(&r, "premium_tax_credit"), 5134);
    assert_eq!(value_of(&r, "premium_credit_advance"), 6000);
    assert_eq!(value_of(&r, "premium_credit_repayment"), 866);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
}

#[test]
fn marketplace_source_identity_survives_reorder_rename_and_same_issuer() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    let (mut s, _) = at_the_marketplace_page(d);
    let mut value = marketplace_fixture_page(vec![
        all_year("Same issuer", 25_000, 60_000, 0),
        all_year("Same issuer", 25_000, 60_000, 0),
    ]);
    value.forms[1].policy_number = "000124".into();
    // Two statements repeat one benchmark: explicit actual eligible group
    // supplies the family amount once, never 600+600.
    value.benchmarks = (1..=12)
        .map(|month| MarketplaceBenchmarkMonthPage {
            month,
            groups: vec![MarketplaceBenchmarkGroupPage {
                state: TaxState::CA,
                rating_area: "Actual area".into(),
                people: vec![-1],
                source: MarketplaceBenchmarkSourcePage::Marketplace { amount: 60_000 },
            }],
            allocated: vec![],
        })
        .collect();
    let mut saved = save_marketplace_and_reopen(&mut s, value);
    let ids = (saved.forms[0].id, saved.forms[1].id);
    assert_ne!(ids.0, ids.1);
    saved.forms.swap(0, 1);
    saved.forms[0].issuer = "Renamed issuer".into();
    let reopened = save_marketplace_and_reopen(&mut s, saved.clone());
    assert_eq!(reopened, saved);
    assert_eq!((reopened.forms[0].id, reopened.forms[1].id), (ids.1, ids.0));
    assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 5134);
}

#[test]
fn marketplace_links_require_a_saved_source_and_ignore_echoed_catalogs() {
    let d = complete(FederalStatus::Single);
    let (mut s, _) = at_the_marketplace_page(d);
    let mut value = marketplace_fixture_page(vec![all_year("Issuer", 50_000, 60_000, 0)]);
    value.forms[0].enrolling = MarketplaceFamilyRefPage::Other { family: s.draft.next_id + 1 };
    value.other_families.push(MarketplaceFamilyPage { id: 0, label: "Other taxpayer".into() });
    let before = s.draft.clone();
    let mut request = save_marketplace(&before, value.clone());
    if let P::SaveMarketplace { ptc_families, .. } = &mut request {
        ptc_families.push(RecordChoice {
            id: before.next_id + 1,
            label: crate::common::test_label("Forged"),
        });
    }
    error(send(&mut s, request));
    assert_eq!(s.draft, before);
    value.forms.clear();
    value.eligibility.clear();
    let saved = save_marketplace_and_reopen(&mut s, value);
    assert!(saved.other_families[0].id > 0);
    let family = saved.other_families[0].id;
    let mut value = saved;
    value.covered.push(MarketplaceCoveredPersonPage {
        id: 0,
        identity: MarketplacePersonIdentityPage::OtherFamily {
            family,
            label: "External enrollee".into(),
        },
    });
    let saved = save_marketplace_and_reopen(&mut s, value);
    assert_eq!(stored_marketplace(&s).covered[0].id, saved.covered[0].id);
}

#[test]
fn marketplace_source_removal_rejects_dangling_links_but_allows_both_sides() {
    let d = complete(FederalStatus::Single);
    let (mut s, _) = at_the_marketplace_page(d);
    let mut value = MarketplaceFactsPage::default();
    value.other_families.push(MarketplaceFamilyPage { id: 0, label: "Other taxpayer".into() });
    let mut value = save_marketplace_and_reopen(&mut s, value);
    value.covered.push(MarketplaceCoveredPersonPage {
        id: 0,
        identity: MarketplacePersonIdentityPage::OtherFamily {
            family: value.other_families[0].id,
            label: "External".into(),
        },
    });
    let mut value = save_marketplace_and_reopen(&mut s, value);
    let before = s.draft.clone();
    value.other_families.clear();
    let request = save_marketplace(&before, value.clone());
    error(send(&mut s, request));
    assert_eq!(s.draft, before);
    value.covered.clear();
    save_marketplace_and_reopen(&mut s, value);
    assert!(matches!(s.draft.credits.as_ref().unwrap().marketplace, MarketplaceClaim::No));
}

#[test]
fn marketplace_estimate_is_transient_and_blank_benchmark_is_preserved() {
    let d = complete(FederalStatus::Single);
    let (mut s, _) = at_the_marketplace_page(d);
    let mut value = marketplace_fixture_page(vec![all_year("Issuer", 50_000, 60_000, 40_000)]);
    let PolicyAmounts::AllYear { benchmark, .. } = &mut value.forms[0].amounts else {
        unreachable!()
    };
    *benchmark = None;
    let before = s.draft.clone();
    let request = P::EstimateMarketplace {
        user: TEST_USER,
        people: vec![],
        ptc_people: vec![],
        ptc_families: vec![],
        ptc_policies: vec![],
        ptc_w2: vec![],
        ptc_employees: vec![],
        ptc_nonreturn_people: vec![],
        ptc_claimed_dependents: vec![],
        allowed_marriage: vec![],
        allowed_separate: vec![],
        value: value.clone(),
    };
    let response = send(&mut s, request);
    assert!(matches!(response, Reply::Response(P::RunningEstimate { .. })));
    assert_eq!(s.draft, before);
    let reopened = save_marketplace_and_reopen(&mut s, value);
    assert!(matches!(reopened.forms[0].amounts, PolicyAmounts::AllYear { benchmark: None, .. }));
    let r = estimate(&s.draft);
    assert_ne!(r.state, ResultState::EstimatedForSupportedScope);
    assert!(r.issues.iter().any(|i| i.display.key == "PTC_SOURCE_BENCHMARK_REQUIRED"));
}

#[test]
fn marketplace_appeal_deadline_calculation_and_server_twin_use_calendar_days() {
    let d = complete(FederalStatus::Single);
    let (mut s, _) = at_the_marketplace_page(d);
    let before = s.draft.clone();
    let response =
        send(&mut s, P::CalculateMarketplaceAppeal { user: TEST_USER, determined: 20261215 });
    assert!(matches!(
        response,
        Reply::Response(P::MarketplaceAppealDeadline { required_deadline: 20270414 })
    ));
    assert_eq!(s.draft, before);
    let mut value = marketplace_fixture_page(vec![all_year("Issuer", 50_000, 60_000, 40_000)]);
    value.forms[0].payments.push(MarketplacePaymentMonthPage {
        tax_free_premiums: 0,
        tax_free_coverage: None,
        month: 1,
        paid: 50_000,
        refunded: 0,
        unpaid: 0,
        coverage: MarketplacePaymentCoverage::PaidByDeadline,
        appeal: Some(MarketplaceAppealPage {
            determined: 20261215,
            enrolled: 20261220,
            paid_on: Some(20270414),
            required_deadline: 20270415,
        }),
    });
    let request = save_marketplace(&before, value);
    error(current_source_refusal(
        "health-context-10",
        &mut s,
        request,
        "MESSAGE_CHECK_THE_MONTHS_AND_AMOUNTS_OF_EACH_FORM_1095_A",
        Some("[value/forms/0/payments/0/appeal/required_deadline]"),
        SourceScope::Health,
    ));
    assert_eq!(s.draft, before);
}

#[test]
fn marketplace_dependent_returns_reach_household_income() {
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    let id = d.dependents[0].id;
    let (mut s, _) = at_the_marketplace_page(d);
    let mut value = marketplace_fixture_page(vec![all_year("Issuer", 50_000, 60_000, 0)]);
    value.dependents.push(DependentReturn {
        person: id,
        agi: 1_100_000,
        exempt_interest: 50_000,
        foreign_exclusion: 200_000,
        social: 400_000,
        taxable_social: 100_000,
    });
    save_marketplace_and_reopen(&mut s, value);
    assert_eq!(stored_marketplace(&s).dependents[0].person, id);
    assert_eq!(value_of(&supported(&s.draft), "marketplace_household_income"), 47800);
}

#[test]
fn marketplace_months_on_the_self_employed_page_count_the_premiums_once() {
    // Derived Pub974 Simplified calculation: actual saved Form1095-A supplies
    // $7200 once; the plan supplies only establishment/monthly eligibility.
    let mut s = at_the_adjustments_hub_with_business(4_000_000);
    let id = stored_business(&s).0;
    let request = open_credits_topic(&mut s, TaxSection::Marketplace);
    assert!(matches!(request, P::SaveMarketplace { .. }));
    let saved = save_marketplace_and_reopen(
        &mut s,
        marketplace_fixture_page(vec![all_year("Synthetic policy", 60_000, 55_000, 30_000)]),
    );
    let policy = saved.forms[0].id;
    let mut benefits = open_benefits(&mut s);
    benefits.businesses[0].health = se_health_test_year(0, false);
    benefits.policies = (1..=12)
        .map(|month| HealthPolicyAllocationPage {
            policy,
            month,
            activity: id,
            parts: 1,
            reimbursed: None,
        })
        .collect();
    accepted(&save_benefits(&mut s, benefits.clone()));
    assert_eq!(open_benefits(&mut s), benefits);
    let mut d = s.draft.clone();
    d.finished_sections = vec![
        TaxSection::Children,
        TaxSection::Checklist,
        TaxSection::Income,
        TaxSection::Adjustments,
        TaxSection::Deductions,
        TaxSection::OtherTaxes,
        TaxSection::Payments,
    ];
    let r = supported(&d);
    assert_eq!(value_of(&r, "marketplace_specified_premiums"), 7200);
    assert_eq!(value_of(&r, "marketplace_specified_deduction"), 2449);
    assert_eq!(value_of(&r, &alloc::format!("business_{id}_health_insurance")), 2449);
    assert_eq!(value_of(&r, "agi"), 34725);
    assert_eq!(value_of(&r, "premium_tax_credit"), 4041);
    assert_eq!(value_of(&r, "premium_credit_refundable"), 441);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn health_credit_round_trips_through_the_business_page_and_reduces_line_14() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let b = health_credit_business();
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let (id, data) = stored_business(&session);
    let claim = data.small_employer_health_credit.as_ref().unwrap();
    assert_eq!(claim.credit(&data.payroll), 10_080);
    let r = supported(&session.draft);
    assert_eq!(value(&r, &alloc::format!("business_{id}_form8941_line12")), 10_080);
    assert_eq!(
        value(&r, &alloc::format!("business_{id}_expense_line14_employee_benefit_programs")),
        10_080
    );
    assert_eq!(value(&r, &alloc::format!("business_{id}_net_profit")), 67_920);
    assert_eq!(value(&r, "business_credit_small_employer_health_current_year"), 10_080);
    assert_round_trip(&mut session, &[b]);
}

#[test]
fn health_credit_page_rules_are_checked_on_save() {
    // Exercise the actual native Save owner; declarations are server-only.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let edited = |edit: &dyn Fn(&mut SmallEmployerHealthCredit)| {
        let mut b = health_credit_business();
        edit(b.more.small_employer_health.as_mut().unwrap());
        b
    };
    // The county exception needs a 2025 first year.
    rejected_with(
        save_businesses(
            &mut session,
            vec![edited(&|c| c.coverage = HealthCreditCoverage::CountyException)],
        ),
        "SMALL_EMPLOYER_HEALTH_COUNTY_RELIEF_NEEDS_PRIOR_YEAR",
    );
    accepted(&save_businesses(
        &mut session,
        vec![edited(&|c| {
            c.coverage = HealthCreditCoverage::CountyException;
            c.first_credit_year = 2025;
        })],
    ));
    // An employee entered twice.
    rejected_with(
        save_businesses(&mut session, vec![edited(&|c| c.employees[3].label = "E1".into())]),
        "SMALL_EMPLOYER_HEALTH_EMPLOYEE_TWICE",
    );
    // Nobody enrolled.
    rejected_with(
        save_businesses(
            &mut session,
            vec![edited(&|c| c.employees.iter_mut().for_each(|e| e.coverage = None))],
        ),
        "SMALL_EMPLOYER_HEALTH_NOBODY_ENROLLED",
    );
    // The share paid above the total premium.
    rejected_with(
        save_businesses(
            &mut session,
            vec![edited(&|c| {
                c.employees[0].coverage.as_mut().unwrap().employer_paid = 720_001;
            })],
        ),
        "SMALL_EMPLOYER_HEALTH_SHARE_ABOVE_PREMIUM",
    );
    // The credit needs the payroll.
    let mut b = health_credit_business();
    b.more.payroll = BusinessPayroll::No;
    rejected_with(save_businesses(&mut session, vec![b]), "EMPLOYER_CREDITS_NEED_PAYROLL");
}

// Derived2026 marriage fixture; primary worksheet structure Pub974 pp38–44,
// separate actual recipient statements on one policy under Instructions1095A
// Statements to Individuals and PartIII A/C. Separate records, same issuer/number.
#[test]
fn marketplace_marriage_pages_consume_each_actual_recipient_statement_once() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 4_800_000, 0);
    let (mut s, _) = at_the_marketplace_page(d);
    let mut value = marketplace_fixture_page(vec![
        all_year("Same issuer", 120_000, 120_000, 110_000),
        all_year("Same issuer", 120_000, 120_000, 110_000),
    ]);
    for (form, person) in value.forms.iter_mut().zip([-1, -2]) {
        form.policy_number = "Same insurance policy".into();
        form.amounts = PolicyAmounts::ByMonth {
            months: vec![PolicyMonth {
                month: 1,
                premium: 120_000,
                benchmark: Some(120_000),
                advance: 110_000,
            }],
        };
        form.coverage =
            vec![MarketplacePolicyCoveragePage { month: 1, people: vec![person], entry: vec![] }];
    }
    value.eligibility = [-1, -2]
        .into_iter()
        .map(|person| MarketplaceEligibilityMonthPage {
            month: 1,
            person,
            status: MarketplacePersonEligibility::Eligible,
        })
        .collect();
    value.benchmarks = vec![MarketplaceBenchmarkMonthPage {
        month: 1,
        allocated: vec![],
        groups: vec![MarketplaceBenchmarkGroupPage {
            state: TaxState::CA,
            rating_area: "Actual joint coverage area".into(),
            people: vec![-1, -2],
            source: MarketplaceBenchmarkSourcePage::Marketplace { amount: 240_000 },
        }],
    }];
    let mut saved = save_marketplace_and_reopen(&mut s, value);
    let ids = [saved.forms[0].id, saved.forms[1].id];
    assert_ne!(ids[0], ids[1]);
    let family = |id| MarketplaceMarriageFamilyPage {
        dependents: vec![],
        premarriage_policies: vec![id],
        benchmarks: vec![],
    };
    saved.marriage = MarketplaceMarriagePage::Alternative {
        wedding: 20260215,
        unmarried_at_start: true,
        you: family(ids[0]),
        spouse: family(ids[1]),
    };
    let next = s.draft.next_id;
    let reopened = save_marketplace_and_reopen(&mut s, saved.clone());
    assert_eq!(reopened, saved);
    assert_eq!(s.draft.next_id, next);
    let result = supported(&s.draft);
    assert_eq!(value_of(&result, "premium_tax_credit"), 2226);
    assert_eq!(value_of(&result, "premium_credit_refundable"), 0);
    assert_eq!(value_of(&result, "premium_credit_repayment"), 0);
    let MarketplaceMarriagePage::Alternative { spouse, .. } = &mut saved.marriage else {
        unreachable!()
    };
    spouse.premarriage_policies = vec![ids[0]];
    let before = s.draft.clone();
    error(send(&mut s, save_marketplace(&before, saved)));
    assert_eq!(s.draft, before); // One original statement cannot be consumed twice.
}

#[test]
fn health_marketplace_links_use_saved_sources_and_actual_activity_catalog() {
    let mut s = at_the_adjustments_hub_with_business(4_000_000);
    let activity = stored_business(&s).0;
    let mut benefits = open_benefits(&mut s);
    benefits.policies.push(HealthPolicyAllocationPage {
        policy: s.draft.next_id,
        month: 1,
        activity,
        parts: 1,
        reimbursed: None,
    });
    let before = s.draft.clone();
    error(save_benefits(&mut s, benefits));
    assert_eq!(s.draft, before); // Save the source before linking it.
    let request = open_credits_topic(&mut s, TaxSection::Marketplace);
    assert!(matches!(request, P::SaveMarketplace { .. }));
    let saved = save_marketplace_and_reopen(
        &mut s,
        marketplace_fixture_page(vec![all_year("Actual source", 60_000, 55_000, 30_000)]),
    );
    let policy = saved.forms[0].id;
    let mut benefits = open_benefits(&mut s);
    benefits.policies =
        vec![HealthPolicyAllocationPage { policy, month: 1, activity, parts: 1, reimbursed: None }];
    accepted(&save_benefits(&mut s, benefits.clone()));
    assert_eq!(open_benefits(&mut s), benefits);
    assert_eq!(s.draft.adjustments.as_ref().unwrap().health_marketplace.policies[0].policy, policy);
    let before = s.draft.clone();
    let mut forged = benefits.clone();
    forged.policies[0].activity = 999_999;
    error(save_benefits(&mut s, forged));
    assert_eq!(s.draft, before);
    let mut zero = benefits.clone();
    zero.policies[0].parts = 0;
    error(save_benefits(&mut s, zero));
    assert_eq!(s.draft, before);
    let mut duplicate = benefits.clone();
    duplicate.policies.push(duplicate.policies[0].clone());
    error(save_benefits(&mut s, duplicate));
    assert_eq!(s.draft, before);
    let mut removed = saved;
    removed.forms.clear();
    let request = save_marketplace(&s.draft, removed);
    // Original08 removes the actual saved Marketplace policy while its
    // accepted self-employed allocation still refers to that source.
    error(current_source_refusal(
        "health-context-08",
        &mut s,
        request,
        "ISSUE_INTERVIEW_REVIEW_THE_ACCEPTED_PAGE_VALUES_BEFORE_65B6D7B7",
        None,
        SourceScope::Health,
    ));
    assert_eq!(s.draft, before); // Never retain a dangling source assignment.
}

#[test]
fn marketplace_actual_tax_free_funding_reopens_and_rejects_impossible_offsets() {
    // Derived Pub502 Insurance Premiums reimbursement boundary, through
    // actual accepted source pages. Gross Form1095-A amounts stay intact.
    let (mut s, _) = at_the_marketplace_page(complete(FederalStatus::Single));
    let mut page = marketplace_fixture_page(vec![all_year("Actual insurer", 50_000, 60_000, 0)]);
    page.forms[0].payments.push(MarketplacePaymentMonthPage {
        month: 1,
        paid: 50_000,
        refunded: 0,
        tax_free_premiums: 10_000,
        tax_free_coverage: Some(10_000),
        unpaid: 0,
        coverage: MarketplacePaymentCoverage::PaidByDeadline,
        appeal: None,
    });
    let saved = save_marketplace_and_reopen(&mut s, page);
    assert_eq!(saved.forms[0].payments[0].tax_free_premiums, 10_000);
    assert_eq!(saved.forms[0].payments[0].tax_free_coverage, Some(10_000));
    assert_eq!(stored_marketplace(&s).forms[0].payments[0].tax_free_premiums, 10_000);
    let before = s.draft.clone();
    let mut invalid = saved.clone();
    invalid.forms[0].payments[0].tax_free_premiums = 50_001;
    let request = save_marketplace(&s.draft, invalid);
    error(send(&mut s, request));
    assert_eq!(s.draft, before);
    let mut invalid = saved;
    invalid.forms[0].payments[0].tax_free_coverage = Some(10_001);
    let request = save_marketplace(&s.draft, invalid);
    error(send(&mut s, request));
    assert_eq!(s.draft, before);
}

#[test]
fn complete_health_plan_page_reopens_ltc_employer_eligibility_and_child_increase() {
    let mut s = at_the_adjustments_hub_with_business(4_000_000);
    let mut value = open_benefits(&mut s);
    value.businesses[0].health = SeHealth::Plan {
        establishment: HealthEstablishmentPage::SoleProprietor,
        months: vec![BusinessHealthMonth {
            month: 1,
            premiums: 100_000,
            child_increase: 10_000,
            employer_eligible: true,
            reimbursed: None,
        }],
        long_term_care: vec![BusinessCarePremiumPage {
            person: -1,
            qualified: CareContractQualification::Qualified,
            months: vec![BusinessCareMonth {
                month: 2,
                premiums: 50_000,
                employer_eligible: false,
                reimbursed: None,
            }],
        }],
    };
    accepted(&save_benefits(&mut s, value.clone()));
    assert_eq!(open_benefits(&mut s), value);
    let mut restarted = restart(s.draft.clone());
    assert_eq!(open_benefits(&mut restarted), value);
    let before = s.draft.clone();
    let mut invalid = value.clone();
    let SeHealth::Plan { establishment, .. } = &mut invalid.businesses[0].health else { panic!() };
    *establishment =
        HealthEstablishmentPage::Partnership { payment: EntityPremiumPayment::EntityPaid };
    error(current_source_refusal(
        "health-context-04",
        &mut s,
        benefits_request(&before, invalid),
        "PROBLEM_INVALID_HEALTH_PLAN_SOURCE",
        Some("[value/businesses/0/health/establishment]"),
        SourceScope::Health,
    ));
    assert_eq!(s.draft, before);
    let mut invalid = value;
    let SeHealth::Plan { long_term_care, .. } = &mut invalid.businesses[0].health else { panic!() };
    long_term_care[0].person = 999_999;
    error(current_source_refusal(
        "health-context-05",
        &mut s,
        benefits_request(&before, invalid),
        "PROBLEM_INVALID_HEALTH_PLAN_SOURCE",
        Some("[value/businesses/0/health/long_term_care/0/person]"),
        SourceScope::Health,
    ));
    assert_eq!(s.draft, before);
}

#[test]
fn empty_health_plan_establishment_survives_working_source_normalization() {
    let mut s = at_the_adjustments_hub_with_business(4_000_000);
    let mut benefits = open_benefits(&mut s);
    benefits.businesses[0].health = SeHealth::Plan {
        establishment: HealthEstablishmentPage::SoleProprietor,
        months: vec![],
        long_term_care: vec![],
    };
    accepted(&save_benefits(&mut s, benefits));
    let mut notes = vec![];
    let working =
        crate::unknowns::supported(&s.draft, &mut notes).unwrap_or_else(|| s.draft.clone());
    let FederalIncome::Business { data } = &working.income[0].income else { panic!() };
    assert!(matches!(data.health_insurance, BusinessHealthInsurance::Plan { .. }));
}

/// Derived qualified-premium boundary, using the shared 2026 age/earned-income
/// owners. IRS instructions 8853 (2025 final), Qualified LTC Insurance Contract:
/// pre-1997 issue, contemporaneous state compliance and no material change.
/// https://www.irs.gov/instructions/i8853
/// Synthetic completion: one existing profitable sole proprietor, $500 LTC
/// paid for taxpayer in January, no employer LTC eligibility or other policy.
#[test]
fn typed_ltc_contract_qualification_reopens_and_only_allows_established_benefit() {
    for qualified in [
        CareContractQualification::Qualified,
        CareContractQualification::Grandfathered {
            issued: 19961231,
            state_compliant: true,
            materially_changed: false,
        },
        CareContractQualification::Grandfathered {
            issued: 19961231,
            state_compliant: false,
            materially_changed: false,
        },
        CareContractQualification::Grandfathered {
            issued: 19961231,
            state_compliant: true,
            materially_changed: true,
        },
        CareContractQualification::Nonqualified,
        CareContractQualification::Nonqualified,
    ] {
        let mut s = at_the_adjustments_hub_with_business(4_000_000);
        let mut benefits = open_benefits(&mut s);
        let id = benefits.businesses[0].id;
        benefits.businesses[0].health = SeHealth::Plan {
            establishment: HealthEstablishmentPage::SoleProprietor,
            months: vec![],
            long_term_care: vec![BusinessCarePremiumPage {
                person: -1,
                qualified: qualified.clone(),
                months: vec![BusinessCareMonth {
                    month: 1,
                    premiums: 50_000,
                    employer_eligible: false,
                    reimbursed: None,
                }],
            }],
        };
        accepted(&save_benefits(&mut s, benefits.clone()));
        assert_eq!(open_benefits(&mut s), benefits);
        let mut restarted = restart(s.draft.clone());
        assert_eq!(open_benefits(&mut restarted), benefits);
        let r = supported(&s.draft);
        assert_eq!(
            value(&r, &alloc::format!("business_{id}_health_insurance")),
            if qualified.qualified() { 500 } else { 0 }
        );
        assert_eq!(noted(&r, "WARN_LTC_PREMIUM_LEFT_OUT"), !qualified.qualified());
        let before = s.draft.clone();
        let SeHealth::Plan { long_term_care, .. } = &mut benefits.businesses[0].health else {
            panic!()
        };
        long_term_care[0].qualified = CareContractQualification::Grandfathered {
            issued: 19970101,
            state_compliant: true,
            materially_changed: false,
        };
        error(save_benefits(&mut s, benefits.clone()));
        assert_eq!(s.draft, before);
        let SeHealth::Plan { long_term_care, .. } = &mut benefits.businesses[0].health else {
            panic!()
        };
        long_term_care[0].qualified = CareContractQualification::Grandfathered {
            issued: 19960230,
            state_compliant: true,
            materially_changed: false,
        };
        error(save_benefits(&mut s, benefits));
        assert_eq!(s.draft, before);
    }
}

/// Derived 2026 age-limit fixture, not a published complete return. IRS
/// instructions for Form 7206, line 2, share the insured person's annual cap:
/// https://www.irs.gov/instructions/i7206
/// Synthetic facts: taxpayer age 45, one profitable sole proprietor, two
/// established qualified contracts ($800 each) and one unknown contract
/// ($1,000), all paid in January with no employer LTC eligibility. The
/// established contracts together receive $930; unknown premiums receive no
/// benefit and do not reserve any of the shared age limit.
#[test]
fn health_ltc_separate_contract_rows_share_insured_age_limit() {
    let mut s = at_the_adjustments_hub_with_business(4_000_000);
    set_age(&mut s.draft, -1, 45);
    let mut benefits = open_benefits(&mut s);
    let id = benefits.businesses[0].id;
    let contract = |qualified, premiums| BusinessCarePremiumPage {
        person: -1,

        qualified,
        months: vec![BusinessCareMonth {
            month: 1,
            premiums,
            employer_eligible: false,
            reimbursed: None,
        }],
    };
    benefits.businesses[0].health = SeHealth::Plan {
        establishment: HealthEstablishmentPage::SoleProprietor,
        months: vec![],
        long_term_care: vec![
            contract(CareContractQualification::Nonqualified, 100_000),
            contract(CareContractQualification::Qualified, 80_000),
            contract(
                CareContractQualification::Grandfathered {
                    issued: 19961231,
                    state_compliant: true,
                    materially_changed: false,
                },
                80_000,
            ),
        ],
    };
    accepted(&save_benefits(&mut s, benefits.clone()));
    assert_eq!(open_benefits(&mut s), benefits);
    for _ in 0..2 {
        let result = supported(&s.draft);
        assert_eq!(value(&result, &alloc::format!("business_{id}_health_insurance")), 930);
        assert_eq!(value(&result, "long_term_care_1_eligible"), 930);
        assert!(noted(&result, "WARN_LTC_PREMIUM_LEFT_OUT"));
        let SeHealth::Plan { long_term_care, .. } = &mut benefits.businesses[0].health else {
            panic!()
        };
        long_term_care.reverse();
        accepted(&save_benefits(&mut s, benefits.clone()));
    }
    let mut restarted = restart(s.draft.clone());
    assert_eq!(open_benefits(&mut restarted), benefits);
}

/// Actual People editor lifecycle for the approved coverage-only source
/// purpose. This is a derived fixture with neutral synthetic household facts;
/// it does not elect tax benefits merely because a covered person is retained.
#[test]
fn health_coverage_only_people_reopen_without_dependency_care_or_filing_benefits() {
    let mut s = supporting(FederalStatus::Single);
    let FederalIncome::Wage { data } = &mut s.draft.income[0].income else { panic!() };
    data.wages = 2_000_000;
    data.medicare_wages = 2_000_000;
    let mut person = row("Covered", "Child", 20180615);
    person.use_for = PersonUse::HealthCoverageOnly;
    person.care = CareCosts::Yes { expenses: 100_000, incapable_self_care: false };
    let request = save_children(&s.draft, vec![person]);
    accepted(&send(&mut s, request));
    let id = s.draft.dependents[0].id;
    assert_eq!(s.draft.credits.as_ref().unwrap().care, FederalCare::default());
    let mut result = FederalResult::empty(s.draft.revision);
    let family =
        crate::federal_rules::family(&s.draft, &mut result, s.draft.people.as_ref().unwrap());
    assert!(family.dependent_ids().is_empty());
    assert!(!family.qualifying_head);
    let result = supported(&s.draft);
    assert_eq!(value(&result, "child_credit"), 0);
    assert_eq!(value(&result, "additional_child_credit"), 0);
    assert_eq!(value(&result, "earned_income_credit"), 0);
    assert!(crate::interview::medical_people(&s.draft).iter().any(|(p, _)| p.id == id));
    let mut reopened = reopened_people(&mut s);
    assert_eq!(reopened[0].use_for, PersonUse::HealthCoverageOnly);
    let mut restarted = restart(s.draft.clone());
    assert_eq!(reopened_people(&mut restarted), reopened);
    reopened[0].use_for = PersonUse::TaxDependent;
    let request = save_children(&s.draft, reopened);
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.dependents[0].id, id);
    let mut result = FederalResult::empty(s.draft.revision);
    let family =
        crate::federal_rules::family(&s.draft, &mut result, s.draft.people.as_ref().unwrap());
    assert_eq!(family.dependent_ids(), &[id]);
    assert!(family.qualifying_head);
    accept_care_provider(&mut s);
    let accepted_care = s.draft.credits.as_ref().unwrap().care.clone();
    let result = supported(&s.draft);
    assert!(value(&result, "child_credit") + value(&result, "additional_child_credit") > 0);
    assert!(value(&result, "earned_income_credit") > 0);
    // The people page retains its own cost facts and does not overwrite the
    // independently owned household Care record.
    assert_eq!(s.draft.credits.as_ref().unwrap().care, accepted_care);
    let mut adult = row("Covered", "Adult", 20000615);
    adult.use_for = PersonUse::HealthCoverageOnly;
    adult.situation =
        DependentSituation::Supported { your_support_over_half: false, gross_income: 4_000_000 };
    adult.facts = PersonFacts::OWN_SUPPORT.0;
    let request = save_children(&s.draft, vec![adult]);
    accepted(&send(&mut s, request));
    assert_eq!(reopened_people(&mut s)[0].use_for, PersonUse::HealthCoverageOnly);
}

/// Derived source-fact fixtures under IRC213 and162(l), not published complete
/// returns. Pub502 (2025 final), Dependent / Qualifying Relative / separated
/// parents / adopted child: https://www.irs.gov/publications/p502
/// Uniform2026 source completion: one sole proprietor with $200 profit, $500
/// qualified LTC paid in January, no employer eligibility or other policy.
/// Medical gross-income/joint-return/filer-claimability exceptions do not
/// elect dependency benefits; a child under27 has a separate162(l) route.
#[test]
fn health_ltc_eligibility_uses_actual_people_relationship_and_medical_exceptions() {
    let parent = Dependent {
        use_for: PersonUse::HealthCoverageOnly,
        relationship: Relationship::Parent,
        situation: DependentSituation::Supported {
            your_support_over_half: true,
            gross_income: 6_000_000,
        },
        facts: PersonFacts::FILED_JOINTLY.0,
        ..row("Covered", "Parent", 19550102)
    };
    let young = Dependent {
        use_for: PersonUse::HealthCoverageOnly,
        situation: DependentSituation::Supported {
            your_support_over_half: false,
            gross_income: 4_000_000,
        },
        facts: PersonFacts::OWN_SUPPORT.0,
        ..row("Covered", "Child", 20000102)
    };
    let adopted = Dependent {
        use_for: PersonUse::HealthCoverageOnly,
        facts: PersonFacts::ABROAD.0 | PersonFacts::ADOPTED_US_CITIZEN_PARENT.0,
        ..row("Covered", "Adopted", 20180102)
    };
    let cases = vec![
        (parent.clone(), true, false, true),
        (
            Dependent {
                situation: DependentSituation::Supported {
                    your_support_over_half: true,
                    gross_income: 0,
                },
                facts: 0,
                ..parent.clone()
            },
            true,
            true,
            false,
        ),
        (
            Dependent {
                facts: PersonFacts::ABROAD.0 | PersonFacts::RESIDENT_CANADA_MEXICO.0,
                ..parent.clone()
            },
            true,
            false,
            false,
        ),
        (Dependent { facts: PersonFacts::ABROAD.0, ..parent.clone() }, false, false, false),
        (
            Dependent {
                situation: DependentSituation::Supported {
                    your_support_over_half: false,
                    gross_income: 6_000_000,
                },
                facts: PersonFacts::MULTIPLE_SUPPORT.0,
                ..parent
            },
            true,
            false,
            false,
        ),
        (young.clone(), false, true, false),
        (Dependent { birth_date: 19991231, ..young.clone() }, false, false, false),
        (Dependent { relationship: Relationship::Foster, ..young.clone() }, false, true, false),
        (
            Dependent { relationship: Relationship::Grandchild, ..young.clone() },
            false,
            false,
            false,
        ),
        (Dependent { relationship: Relationship::Unrelated, ..young }, false, false, false),
        (adopted.clone(), true, true, false),
        (Dependent { facts: PersonFacts::ABROAD.0, ..adopted }, false, true, false),
        (
            Dependent {
                use_for: PersonUse::HealthCoverageOnly,
                lived: LivedWithYou::PartOfYear { months: 0 },
                facts: PersonFacts::MEDICAL_SEPARATED_PARENTS.0,
                ..row("Covered", "Separated", 20180102)
            },
            true,
            true,
            false,
        ),
        (
            Dependent {
                use_for: PersonUse::HealthCoverageOnly,
                relationship: Relationship::Grandchild,
                competing: CompetingClaim::Other,
                ..row("Covered", "OtherClaimant", 20180102)
            },
            false,
            false,
            false,
        ),
    ];
    for (person, medical, self_employed, claimable_filer) in cases {
        let mut s = at_the_adjustments_hub_with_business(20_000);
        if claimable_filer {
            s.draft.people.as_mut().unwrap().taxpayer.dependency =
                FilerDependency::CanBeClaimed { claimed: true };
        }
        let request = save_children(&s.draft, vec![person]);
        accepted(&send(&mut s, request));
        let person = s.draft.dependents[0].id;
        let eligibility = crate::health::person_eligibility(&s.draft)[&person];
        assert_eq!(eligibility.medical, medical, "{:?}", s.draft.dependents[0]);
        assert_eq!(eligibility.self_employed, self_employed);
        let mut benefits = open_benefits(&mut s);
        let business = benefits.businesses[0].id;
        benefits.businesses[0].health = SeHealth::Plan {
            establishment: HealthEstablishmentPage::SoleProprietor,
            months: vec![],
            long_term_care: vec![BusinessCarePremiumPage {
                person,
                qualified: CareContractQualification::Qualified,
                months: vec![BusinessCareMonth {
                    month: 1,
                    premiums: 50_000,
                    employer_eligible: false,
                    reimbursed: None,
                }],
            }],
        };
        accepted(&save_benefits(&mut s, benefits.clone()));
        assert_eq!(open_benefits(&mut s), benefits);
        let result = supported(&s.draft);
        assert_eq!(
            value(&result, &alloc::format!("business_{business}_health_insurance")),
            if self_employed { 200 } else { 0 }
        );
        let medical_amount = result
            .lines
            .iter()
            .find(|v| v.id == "long_term_care_1_medical")
            .map_or(0, |v| v.amount / 100);
        assert_eq!(medical_amount, if medical { if self_employed { 300 } else { 500 } } else { 0 });
        assert_eq!(noted(&result, "WARN_LTC_PREMIUM_LEFT_OUT"), !medical && !self_employed);
        assert!(crate::marketplace::family_people(&s.draft).iter().all(|v| *v != person));
    }
}

#[test]
fn hsa_retained_default_reopens_as_equal_and_preserves_actual_split_choice() {
    let mut s = at_the_adjustments_hub(FederalStatus::Joint, 0);
    s.draft.adjustments.as_mut().unwrap().hsa = vec![
        hsa_entry(Owner::Taxpayer, HsaCoverage::Family, 437_500),
        hsa_entry(Owner::Spouse, HsaCoverage::SelfOnly, 437_500),
    ];
    assert!(s.draft.adjustments.as_ref().unwrap().hsa_family_share.is_none());
    let page = open_hsa_page(&mut s);
    let (input, allowed) = hsa_page_value(&page);
    assert_eq!(input.family_split, FamilySplit::Equal);
    assert_eq!(allowed, &[1, 2]);
    accepted(&send(&mut s, page));
    assert_eq!(value(&supported(&s.draft), "adjustments"), 8750);
    let reopened = open_hsa_page(&mut s);
    assert_eq!(hsa_page_value(&reopened).0.family_split, FamilySplit::Equal);
    assert_eq!(supported(&restart(s.draft.clone()).draft), supported(&s.draft));
}

// Source-derived HSA boundaries: Pub969 Rules for married people / i8889
// line6 and line3 last-month rule. Neutral completion facts explicitly identify
// the other filer's eligibility; no other-filer HSA or income is invented.
#[test]
fn hsa_actual_other_filer_context_saves_reopens_and_uses_equal_default() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut s = at_the_adjustments_hub(FederalStatus::Joint, 0);
        let mut page = open_hsa_page(&mut s);
        let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
        let mut account = hsa_account(owner, HsaCoverageYear::Family, 500_000);
        account.excess_accounting = Some(HsaExcessAccounting {
            prior_year: None,
            employer_excess_in_wages: 0,
            year_end_value: 100_000,
            later_return_year_deposits: 0,
            later_employer_deposits: 0,
        });
        value.accounts = vec![account];
        value.other_coverage = Some(HsaEligibility::FullYear { coverage: HsaCoverage::SelfOnly });
        value.family_split = FamilySplit::Equal;
        accepted(&send(&mut s, page));
        let r = supported(&s.draft);
        assert_eq!(super::value(&r, "adjustments"), 4375);
        assert_eq!(super::value(&r, "hsa_return_year_excess_contribution_tax"), 38);
        let other = s
            .draft
            .adjustments
            .as_ref()
            .unwrap()
            .hsa_family_share
            .as_ref()
            .unwrap()
            .other_coverage
            .clone();
        assert_eq!(other, Some(HsaEligibility::FullYear { coverage: HsaCoverage::SelfOnly }));
        assert_eq!(s.draft.adjustments.as_ref().unwrap().hsa.len(), 1);
        assert_eq!(s.draft.adjustments.as_ref().unwrap().hsa[0].owner, owner);
        let mut restarted = Interview::new(TEST_USER, restart(s.draft.clone()).draft);
        let reopened = open_hsa_page(&mut restarted);
        let (input, choices) = hsa_page_value(&reopened);
        assert_eq!(input.other_coverage, other);
        assert_eq!(input.family_method, HsaAllocationMethod::Monthly);
        assert_eq!(choices, &[1, 2]);
        assert_eq!(supported(&restarted.draft), r);
    }
}

#[test]
fn hsa_missing_other_eligibility_is_required_even_for_self_only_and_cannot_duplicate_an_owner() {
    let mut s = at_the_adjustments_hub(FederalStatus::Joint, 0);
    let mut page = open_hsa_page(&mut s);
    let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
    value.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 440_000)];
    value.other_coverage = None;
    value.family_split = FamilySplit::NotApplicable;
    accepted(&send(&mut s, page));
    let r = estimate(&s.draft);
    assert!(r.tax.is_none());
    let issue =
        r.issues.iter().find(|i| i.display.key == "HSA_OTHER_OWNER_ELIGIBILITY_REQUIRED").unwrap();
    assert_eq!(
        crate::interview::issue_page(&s.draft, &issue.context, Some(&issue.display.key)),
        Some(TaxSection::Hsa)
    );
    let mut page = open_hsa_page(&mut s);
    let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
    value.other_coverage = Some(HsaEligibility::FullYear { coverage: HsaCoverage::None });
    accepted(&send(&mut s, page));
    assert_eq!(super::value(&supported(&s.draft), "adjustments"), 4400);
    let before = s.draft.clone();
    let mut page = open_hsa_page(&mut s);
    let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
    value.accounts.push(hsa_account(Owner::Spouse, HsaCoverageYear::SelfOnly, 0));
    // Two independent sources for the same person's eligibility are rejected.
    rejected_with(send(&mut s, page), "HSA_OTHER_OWNER_CONTEXT_INVALID");
    assert_eq!(s.draft, before);
}

#[test]
fn hsa_monthly_and_december_choices_use_same_actual_account_and_restart() {
    let mut s = at_the_adjustments_hub(FederalStatus::Single, 0);
    let mut page = open_hsa_page(&mut s);
    let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
    let mut months = vec![HsaCoverage::None; 12];
    months[11] = HsaCoverage::SelfOnly;
    value.accounts =
        vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::Changed { months }, 440_000)];
    value.family_method = HsaAllocationMethod::Monthly;
    value.family_split = FamilySplit::NotApplicable;
    accepted(&send(&mut s, page));
    assert!(estimate(&s.draft).tax.is_none()); // Actual monthly capacity is366.67.
    let mut page = open_hsa_page(&mut s);
    let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
    value.family_method = HsaAllocationMethod::December;
    accepted(&send(&mut s, page));
    let r = supported(&s.draft);
    assert_eq!(super::value(&r, "adjustments"), 4400);
    assert_eq!(super::value(&r, "hsa_1_last_month_contributions"), 4033);
    let reopened = open_hsa_page(&mut s);
    assert_eq!(hsa_page_value(&reopened).0.family_method, HsaAllocationMethod::December);
    assert_eq!(supported(&restart(s.draft.clone()).draft), r);
}

#[test]
fn hsa_separate_return_can_elect_december_with_actual_external_spouse_context() {
    let mut s = at_the_adjustments_hub(FederalStatus::Separate, 0);
    let mut page = open_hsa_page(&mut s);
    let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
    let mut months = vec![HsaCoverage::None; 12];
    months[11] = HsaCoverage::Family;
    value.accounts =
        vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::Changed { months }, 437_500)];
    value.other_coverage = Some(HsaEligibility::FullYear { coverage: HsaCoverage::Family });
    value.family_split = FamilySplit::Equal;
    value.family_method = HsaAllocationMethod::December;
    accepted(&send(&mut s, page));
    let r = supported(&s.draft);
    assert_eq!(super::value(&r, "adjustments"), 4375);
    assert_eq!(super::value(&r, "hsa_1_last_month_contributions"), 4010);
    assert_eq!(s.draft.adjustments.as_ref().unwrap().hsa.len(), 1);
    assert!(s.draft.people.as_ref().unwrap().spouse.is_none());
    let reopened = open_hsa_page(&mut s);
    let reopened = hsa_page_value(&reopened).0;
    assert_eq!(reopened.family_method, HsaAllocationMethod::December);
    assert_eq!(reopened.family_split, FamilySplit::Equal);
    assert_eq!(
        reopened.other_coverage,
        Some(HsaEligibility::FullYear { coverage: HsaCoverage::Family })
    );
    assert_eq!(supported(&restart(s.draft.clone()).draft), r);
}

#[test]
fn hsa_limit_context_validation_ignores_echoed_marriage_permission() {
    let mut s = at_the_adjustments_hub(FederalStatus::Single, 0);
    let before = s.draft.clone();
    let mut page = open_hsa_page(&mut s);
    let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
    value.family_method = HsaAllocationMethod::December;
    rejected_with(send(&mut s, page), "HSA_LIMIT_METHOD_NEEDS_ACCOUNT");
    assert_eq!(s.draft, before);
    let mut page = open_hsa_page(&mut s);
    let P::SaveHsa { value, other_coverage_allowed, .. } = &mut page else { unreachable!() };
    *other_coverage_allowed = true;
    value.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 100_000)];
    value.other_coverage = Some(HsaEligibility::FullYear { coverage: HsaCoverage::None });
    rejected_with(send(&mut s, page), "HSA_OTHER_OWNER_CONTEXT_INVALID");
    assert_eq!(s.draft, before);
}

#[test]
fn hsa_external_eligibility_uses_actual_marriage_not_filing_status_label() {
    let mut s = at_the_adjustments_hub(FederalStatus::Head, 0);
    // Explicit household source context: a married nonresident spouse does
    // not stop a qualifying filer using Head, but HSA sharing still sees marriage.
    s.draft.setup.as_mut().unwrap().marital_state = MaritalState::MarriedNonresidentSpouse;
    add_child(&mut s.draft);
    let mut page = open_hsa_page(&mut s);
    let P::SaveHsa { value, other_coverage_allowed, .. } = &mut page else { unreachable!() };
    assert!(*other_coverage_allowed);
    value.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::Family, 400_000)];
    value.other_coverage = Some(HsaEligibility::FullYear { coverage: HsaCoverage::SelfOnly });
    value.family_split = FamilySplit::Equal;
    accepted(&send(&mut s, page));
    assert_eq!(super::value(&supported(&s.draft), "adjustments"), 4000);
    assert!(s.draft.adjustments.as_ref().unwrap().hsa_family_share.is_some());
    assert_eq!(hsa_page_value(&open_hsa_page(&mut s)).0.family_split, FamilySplit::Equal);
}

#[test]
fn archer_only_page_keeps_recorded_hub_status_and_normal_checklist_removal() {
    let d = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d);
    let mut request = open_hsa_page(&mut s);
    let P::SaveHsa { value: input, .. } = &mut request else { panic!() };
    input.archer = ArcherContributionsPage::Records {
        sharing: ArcherSharing::Equal,
        owners: vec![archer_contributor_row(
            Owner::Taxpayer,
            100_000,
            0,
            ArcherEligibility::FullYear {
                coverage: ArcherCoverage::SelfOnly { deductible: 300_000, out_of_pocket: 400_000 },
            },
            vec![1],
        )],
    };
    accepted(&send(&mut s, request));
    assert_eq!(value(&supported(&s.draft), "adjustments"), 1000);
    assert_ne!(s.draft.checklist.adjustments & AdjustmentTopics::HSA.0, 0);
    let hub = open_adjustments_hub(&mut s);
    let rows = hub_rows(&hub);
    let health = rows.iter().find(|row| row.2 == TaxSection::Hsa).unwrap();
    assert_eq!(health.1, "Recorded");
    let before = s.draft.clone();
    let mut ticks = before.checklist;
    ticks.adjustments &= !AdjustmentTopics::HSA.0;
    accepted(&send(&mut s, save_checklist(&before, PAGE_CHILDREN, ticks)));
    assert!(matches!(s.draft.adjustments.as_ref().unwrap().archer, ArcherContributions::None));
    assert_eq!(value(&supported(&s.draft), "adjustments"), 0);
}

// Notice 2008-51 pp. 4–5: actual lifetime transfer history, with 2025 facts
// entered only in the existing testing-period editor. Neutral completion:
// single eligible owner, manual contribution1000, no current IRA transfer.
#[test]
fn hsa_earlier_funding_history_saves_reopens_and_estimates_without_mutation() {
    let mut s = at_the_adjustments_hub(FederalStatus::Single, 0);
    let mut page = open_hsa_page(&mut s);
    let P::SaveHsa { value: input, .. } = &mut page else { unreachable!() };
    let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 100_000);
    account.uses = vec![HsaFundingUseHistory {
        funded: 20240615,
        coverage: HsaCoverage::Family,
        amount: 200_000,
    }];
    input.accounts = vec![account];
    accepted(&send(&mut s, page));
    let accepted_draft = s.draft.clone();
    assert_eq!(value(&supported(&s.draft), "adjustments"), 1000);
    let mut restarted = Interview::new(TEST_USER, restart(s.draft.clone()).draft);
    let mut page = open_hsa_page(&mut restarted);
    let P::SaveHsa { value: input, .. } = &mut page else { unreachable!() };
    assert_eq!(input.accounts[0].uses, s.draft.adjustments.as_ref().unwrap().hsa[0].uses);
    let mut estimated = input.clone();
    estimated.accounts[0].uses[0].amount = 123_456;
    // Route the actual page candidate through its normal transient estimate.
    let reply = send(
        &mut restarted,
        P::EstimateHsa {
            user: TEST_USER,
            hsa_failed_deposits: vec![],
            archer_failed_deposits: vec![],
            hsa_return_documents: vec![],
            archer_return_documents: vec![],
            other_coverage_allowed: false,
            allowed_owners: vec![],
            allowed_family_split: vec![],
            ira_sources: vec![],
            archer_income: vec![],
            value: estimated,
        },
    );
    assert!(matches!(reply, Reply::Response(P::RunningEstimate { .. })));
    assert_eq!(restarted.draft, accepted_draft);
    // Neither a forged prior-year row nor an impossible date can bypass
    // the page's historical date bounds through the server endpoint.
    for (date, coverage, key) in [
        (20250101, HsaCoverage::Family, "HSA_RECORD_INVALID"),
        (20240230, HsaCoverage::Family, "HSA_RECORD_INVALID"),
        (20240615, HsaCoverage::None, "HSA_FUNDING_NEEDS_COVERAGE"),
    ] {
        let mut candidate = page.clone();
        let P::SaveHsa { value: input, .. } = &mut candidate else { unreachable!() };
        input.accounts[0].uses[0].funded = date;
        input.accounts[0].uses[0].coverage = coverage;
        let reply = match date {
            20250101 => current_source_refusal(
                "health-context-06",
                &mut restarted,
                candidate,
                "HSA_RECORD_INVALID",
                Some("[value/accounts/0/uses/0/funded]"),
                SourceScope::Health,
            ),
            20240230 => current_source_refusal(
                "health-context-07",
                &mut restarted,
                candidate,
                "HSA_RECORD_INVALID",
                Some("[value/accounts/0/uses/0/funded]"),
                SourceScope::Health,
            ),
            _ => send(&mut restarted, candidate),
        };
        rejected_with(reply, key);
        assert_eq!(restarted.draft, accepted_draft);
    }
    let P::SaveHsa { value: input, .. } = &mut page else { unreachable!() };
    let duplicate = input.accounts[0].uses[0].clone();
    input.accounts[0].uses.push(duplicate);
    let reply = send(&mut restarted, page);
    rejected_with(reply, "HSA_FUNDING_HISTORY_SAME_MONTH");
    assert_eq!(restarted.draft, accepted_draft);
}

// Pub 969 (2025 final, printed February 11, 2026), printed p. 6,
// Examples 1 and 2: https://www.irs.gov/pub/irs-pdf/p969.pdf .
// The published 2025 contributions and 2026 eligibility losses need no year
// shift. Neutral completion: single filer, no current deposits/distributions,
// no Archer deposits, and no other income. Birthdays preserve published ages.
#[test]
fn published_hsa_last_month_examples_keep_recapture_cents_through_public_page() {
    for (birth, self_months, eligible_months, loss, monthly, recapture, printed_income, tax) in [
        (19720615, 0, 1, 20260601, 71_250, 783_750, 7838, 784),
        (19860615, 10, 12, 20260301, 500_833, 354_167, 3542, 354),
    ] {
        let mut s = at_the_adjustments_hub(FederalStatus::Single, 0);
        let you = save_you(&s.draft, adult("Synthetic", "Owner", birth));
        accepted(&send(&mut s, you));
        let mut months = vec![HsaCoverage::None; 12 - eligible_months];
        months.extend(vec![HsaCoverage::SelfOnly; self_months]);
        months.resize(12, HsaCoverage::Family);
        let history = HsaLastMonthHistory {
            months,
            contributions: 855_000,
            archer: 0,
            family: None,
            loss: Some(HsaEligibilityLoss { date: loss, reason: HsaLossReason::Other }),
        };
        assert_eq!(history.recapture_cents(birth, 0), recapture);
        let mut page = open_hsa_page(&mut s);
        let P::SaveHsa { value: input, .. } = &mut page else { unreachable!() };
        let mut account = hsa_account(
            Owner::Taxpayer,
            HsaCoverageYear::Changed { months: vec![HsaCoverage::None; 12] },
            0,
        );
        account.last_month_history = Some(history.clone());
        input.accounts = vec![account];
        accepted(&send(&mut s, page));
        let r = supported(&s.draft);
        assert_eq!(
            r.lines.iter().find(|v| v.id == "hsa_1_prior_monthly_limit").unwrap().amount,
            monthly
        );
        assert_eq!(value(&r, "hsa_1_testing_last_month_income"), printed_income);
        assert_eq!(
            printed_line(&r, "hsa_1_testing_last_month_income"),
            (TaxForm::Form8889, "18".into())
        );
        assert_eq!(value(&r, "hsa_1_testing_tax"), tax);
        assert_eq!(return_line(&r, "8"), printed_income * 100);
        assert_eq!(value(&r, "adjustments"), 0);
        // The precise source result is consumed once by Marketplace's income
        // projection, independently of its whole-dollar filing rows.
        let mut source_result = FederalResult::empty(s.draft.revision);
        let adjustments = s.draft.adjustments.as_ref().unwrap();
        let hsa = crate::hsa::calculate(
            &adjustments.hsa,
            s.draft.people.as_ref().unwrap(),
            FederalStatus::Single,
            &crate::hsa::ContributionContext {
                msa: core::array::from_fn(|_| crate::kernel::ratio::exact::whole(0)),
                failed: &Default::default(),
                married: false,
                family_share: None,
            },
            &core::array::from_fn(|_| crate::medical_account::OwnedHsaAmounts::default()),
            &s.draft.income,
            &mut source_result,
        )
        .unwrap();
        assert_eq!(hsa.work_income_cents, crate::kernel::ratio::exact::whole(recapture));
        let before = s.draft.clone();
        let mut restarted = restart(before.clone());
        let reopened = open_hsa_page(&mut restarted);
        let (input, _) = hsa_page_value(&reopened);
        assert_eq!(input.accounts[0].personal, 0);
        assert_eq!(input.accounts[0].last_month_history, Some(history));
        assert_eq!(restarted.draft, before);
        assert_eq!(supported(&restarted.draft), r);
    }
}

// Notice 2008-51 pp. 4–5 (direct transfer and beneficiary ownership),
// https://www.irs.gov/pub/irs-drop/n-08-51.pdf . Neutral completion:
// single eligible owner, two documented payments in one inherited group,
// no prior funding use and no earlier IRA basis activity.
#[test]
fn hsa_actual_payment_picker_saves_reopens_restarts_and_rejects_forged_sources() {
    let mut d = inherited_hsa_funding(1000, 0, 6000, 0, 4000);
    add(&mut d, inherited_payment(Owner::Taxpayer, Some(2), 5000));
    let mut transfer = d.adjustments.as_ref().unwrap().hsa[0].funding[0].row(Owner::Taxpayer);
    transfer.source = 4;
    d.adjustments.as_mut().unwrap().hsa[0].funding.clear();
    let mut s = restart(d);
    let mut page = open_hsa_page(&mut s);
    let P::SaveHsa { value, ira_sources, .. } = &mut page else { unreachable!() };
    assert_eq!(ira_sources.iter().map(|row| row.id).collect::<Vec<_>>(), vec![3, 4]);
    value.funding = vec![transfer];
    accepted(&send(&mut s, page));
    let saved = s.draft.clone();
    assert_eq!(saved.adjustments.as_ref().unwrap().hsa[0].funding[0].income, 4);
    assert_eq!(return_line(&supported(&saved), "4b"), 200000);
    let mut restored = restart(saved.clone());
    let page = open_hsa_page(&mut restored);
    let P::SaveHsa { value, .. } = &page else { unreachable!() };
    assert_eq!(value.funding[0].source, 4);
    assert_eq!(restored.draft, saved);
    for source in [2, 1, 99999] {
        let mut forged = page.clone();
        let P::SaveHsa { value, ira_sources, .. } = &mut forged else { unreachable!() };
        value.funding[0].source = source;
        let mut fake = ira_sources[0].clone();
        fake.id = source;
        ira_sources.push(fake);
        let mut input = value.clone();
        let echoed_sources = ira_sources.clone();
        rejected_with(send(&mut restored, forged), "HSA_FUNDING_SOURCE_NOT_OFFERED");
        assert_eq!(restored.draft, saved); // Includes revision and next_id.
        input.funding[0].source = source;
        rejected_with(
            send(
                &mut restored,
                P::EstimateHsa {
                    user: TEST_USER,
                    hsa_failed_deposits: vec![],
                    archer_failed_deposits: vec![],
                    hsa_return_documents: vec![],
                    archer_return_documents: vec![],
                    other_coverage_allowed: false,
                    allowed_owners: vec![],
                    allowed_family_split: vec![],
                    ira_sources: echoed_sources,
                    archer_income: vec![],
                    value: input,
                },
            ),
            "HSA_FUNDING_SOURCE_NOT_OFFERED",
        );
        assert_eq!(restored.draft, saved);
    }
    let mut estimate_input = value.clone();
    estimate_input.funding[0].source = 3;
    estimate_input.funding[0].amount = 100000;
    assert!(matches!(
        send(
            &mut restored,
            P::EstimateHsa {
                user: TEST_USER,
                hsa_failed_deposits: vec![],
                archer_failed_deposits: vec![],
                hsa_return_documents: vec![],
                archer_return_documents: vec![],
                other_coverage_allowed: false,
                allowed_owners: vec![],
                allowed_family_split: vec![],
                ira_sources: vec![],
                archer_income: vec![],
                value: estimate_input,
            }
        ),
        Reply::Response(P::RunningEstimate { .. })
    ));
    assert_eq!(restored.draft, saved);
    // Removing or changing a named payment/group must not erase funding.
    for remove in [2, 4] {
        let mut income_page = open_topic(&mut restored, TaxSection::Retirement);
        let P::SaveIncomes { documents, .. } = &mut income_page else { unreachable!() };
        documents.documents.retain(|row| row.id != remove);
        rejected_with(send(&mut restored, income_page), "HSA_FUNDING_SOURCE_NOT_OFFERED");
        assert_eq!(restored.draft, saved);
    }
    let mut income_page = open_topic(&mut restored, TaxSection::Retirement);
    let P::SaveIncomes { documents, .. } = &mut income_page else { unreachable!() };
    let row = documents.documents.iter_mut().find(|row| row.id == 4).unwrap();
    row.document = IncomeDocument::R1099(r1099(
        500000,
        DistributionCode::Normal { what_happened: Disposition::Kept },
        false,
    ));
    rejected_with(send(&mut restored, income_page), "HSA_FUNDING_SOURCE_NOT_OFFERED");
    assert_eq!(restored.draft, saved);
    let mut income_page = open_topic(&mut restored, TaxSection::Retirement);
    let P::SaveIncomes { documents, .. } = &mut income_page else { unreachable!() };
    documents.documents.reverse();
    if let IncomeDocument::InheritedIra(group) =
        &mut documents.documents.iter_mut().find(|row| row.id == 2).unwrap().document
    {
        group.decedent = "Renamed source owner".into();
    }
    accepted(&send(&mut restored, income_page));
    assert_eq!(restored.draft.adjustments.as_ref().unwrap().hsa[0].funding[0].income, 4);
    assert_eq!(return_line(&supported(&restored.draft), "4b"), 200000);
    assert_eq!(restart(restored.draft.clone()).draft, restored.draft);
}

#[test]
fn marketplace_claimable_filer_attestation_survives_save_reopen_and_restart() {
    let client = NativeInterviewCase("TAX_UNIT46_G9");
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: false };
    let (mut s, _) = at_the_marketplace_page(d);
    if let Some(request) = client.input("CALC_INPUT") {
        let before = s.draft.clone();
        assert!(matches!(request, P::EstimateMarketplace { .. }));
        let Reply::Response(response) = send(&mut s, request) else {
            panic!("calculation response");
        };
        assert!(matches!(response, P::RunningEstimate { .. }));
        assert_eq!(s.draft, before);
        client.required_write("CALC_OUTPUT", &response);
        return;
    }
    let request = open_credits_topic(&mut s, TaxSection::Marketplace);
    client.write("SEED", &request);
    let P::SaveMarketplace { ptc_nonreturn_people, .. } = request else {
        panic!("Marketplace page");
    };
    assert!(ptc_nonreturn_people.iter().any(|person| person.id == -1));
    let mut value =
        marketplace_fixture_page(vec![all_year("Actual statement", 100_000, 120_000, 60_000)]);
    value.memberships.push(MarketplacePersonFamilyPage {
        person: -1,
        family: MarketplaceNonreturnFamilyPage::NoFamily,
    });
    value.attestations.push(MarketplaceAttestationPage {
        person: -1,
        attester: MarketplaceAttesterPage::ThisReturn,
    });
    client.write("ANSWERS", &save_marketplace(&s.draft, value.clone()));
    if let Some(submitted) = client.input("PAYLOAD") {
        credits_hub(send_past_less_common(&mut s, submitted));
        let reopened = open_credits_topic(&mut s, TaxSection::Marketplace);
        let saved = marketplace_page_of(&reopened);
        assert!(saved.forms[0].id > 0);
        assert_eq!(saved.memberships, value.memberships);
        assert_eq!(saved.attestations, value.attestations);
        assert_eq!(value_of(&supported(&s.draft), "premium_credit_repayment"), 7200);
        assert_eq!(restart(s.draft.clone()).draft, s.draft);
        client.write("REOPEN", &reopened);
        return;
    }
    let mut value = save_marketplace_and_reopen(&mut s, value);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
    assert_eq!(value_of(&supported(&s.draft), "premium_credit_repayment"), 7200);
    assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 0);
    let policy = value.forms[0].id;
    value
        .other_families
        .push(MarketplaceFamilyPage { id: 0, label: "Actual attesting parent".into() });
    value = save_marketplace_and_reopen(&mut s, value);
    let family = value.other_families[0].id;
    value.attestations[0].attester = MarketplaceAttesterPage::OtherFamily { family };
    let before_preview = s.draft.clone();
    let preview = send(
        &mut s,
        P::EstimateMarketplace {
            user: TEST_USER,
            people: vec![],
            ptc_people: vec![],
            ptc_families: vec![],
            ptc_policies: vec![],
            ptc_w2: vec![],
            ptc_employees: vec![],
            ptc_nonreturn_people: vec![],
            ptc_claimed_dependents: vec![],
            allowed_marriage: vec![],
            allowed_separate: vec![],
            value: value.clone(),
        },
    );
    assert!(matches!(preview, Reply::Response(P::RunningEstimate { .. })));
    assert_eq!(s.draft, before_preview);
    value = save_marketplace_and_reopen(&mut s, value);
    assert_eq!(value.forms[0].id, policy);
    assert_eq!(value_of(&supported(&s.draft), "premium_credit_advance"), 0);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
    let before = s.draft.clone();
    let mut forged = value.clone();
    forged.attestations[0].attester =
        MarketplaceAttesterPage::OtherFamily { family: before.next_id + 100 };
    let mut request = save_marketplace(&before, forged);
    if let P::SaveMarketplace { ptc_families, .. } = &mut request {
        ptc_families.push(RecordChoice {
            id: before.next_id + 100,
            label: crate::common::test_label("Forged"),
        });
    }
    error(send(&mut s, request));
    assert_eq!(s.draft, before);
    let mut dangling = value.clone();
    dangling.other_families.clear();
    error(send(&mut s, save_marketplace(&before, dangling)));
    assert_eq!(s.draft, before);
    value.attestations[0].attester = MarketplaceAttesterPage::Unknown;
    value = save_marketplace_and_reopen(&mut s, value);
    unsupported(&s.draft, "Marketplace source");
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
    value.forms[0].amounts =
        PolicyAmounts::AllYear { premium: 100_000, benchmark: Some(120_000), advance: 0 };
    let value = save_marketplace_and_reopen(&mut s, value);
    assert_eq!(value.attestations[0].attester, MarketplaceAttesterPage::Unknown);
    assert_eq!(value_of(&supported(&s.draft), "premium_credit_repayment"), 0);
    assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 0);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
}
