use crate::tests::support::client::survey::survey_refusal as admission_refusal;
// Actual admission requests over the existing native predicates. Local Union
// payload fields are transparent; source/cookie/state handling is the normal flow.
use super::*;

fn admission_estimate(request: &P) -> P {
    match request {
        P::SaveSetup { value, .. } => P::EstimateSetup { user: TEST_USER, value: value.clone() },
        P::SavePeople {
            you,
            professional_election,
            allowed_professional_elections,
            ..
        } => P::EstimateYou {
            user: TEST_USER,
            you: you.clone(),
            professional_election: *professional_election,
            allowed_professional_elections: allowed_professional_elections.clone(),
        },
        P::SaveSpouse { spouse, professional_election, allowed_professional_elections, .. } => {
            P::EstimateSpouse {
                user: TEST_USER,
                spouse: spouse.clone(),
                professional_election: *professional_election,
                allowed_professional_elections: allowed_professional_elections.clone(),
            }
        }
        P::SaveChildren { dependents, allowed_lived, days_alive, .. } => P::EstimateDependents {
            user: TEST_USER,
            dependents: dependents.clone(),
            allowed_lived: allowed_lived.clone(),
            days_alive: *days_alive,
        },
        P::SaveEducator { value, allowed_owners, .. } => P::EstimateEducator {
            user: TEST_USER,
            value: value.clone(),
            allowed_owners: allowed_owners.clone(),
        },
        P::SaveEducation { value, people, followups, .. } => P::EstimateEducation {
            user: TEST_USER,
            value: value.clone(),
            people: people.clone(),
            followups: followups.clone(),
            questions: Vec::new(),
        },
        P::SaveDocuments { .. } => request.clone(),
        other => panic!("unsupported admission request {other:?}"),
    }
}
fn admission_open(session: &mut Interview, topic: TaxSection) -> P {
    if topic == TaxSection::Documents {
        return crate::tests::support::client::income_pages::open_document_center(session);
    }
    let request = navigate(&session.draft, topic);
    let Reply::Edit(page) = send(session, request) else {
        panic!("expected admission editor {topic:?}")
    };
    page
}
fn admission_case(
    case: &str,
    session: &mut Interview,
    bad: P,
    corrected: P,
    key: &str,
    path: &str,
    topic: TaxSection,
) {
    let estimate = admission_estimate(&bad);
    admission_refusal(case, session, bad, key, path);
    admission_refusal(&alloc::format!("{case}-estimate"), session, estimate, key, path);
    let before = session.draft.clone();
    let dirty = session.dirty;
    let reply = crate::tests::support::client::income_pages::candidate_save_review(
        session,
        admission_estimate(&corrected),
    );
    assert!(
        match &corrected {
            P::SaveDocuments { .. } => {
                matches!(reply, Reply::Response(P::Review { .. }))
            }
            P::SaveChildren { .. } => {
                matches!(reply, Reply::Response(P::DependentsRunningEstimate { .. }))
            }
            _ => matches!(reply, Reply::Response(P::RunningEstimate { .. })),
        },
        "{case}: corrected estimate {reply:?}"
    );
    assert_eq!(session.draft, before, "{case}: corrected estimate changed source");
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, corrected));
    let saved = session.draft.clone();
    let reopened = admission_open(session, topic);
    accepted(&send(session, reopened));
    assert_eq!(session.draft, saved, "{case}: corrected reopen/resave");
    let mut restored = restart(saved.clone());
    let reopened = admission_open(&mut restored, topic);
    accepted(&send(&mut restored, reopened));
    assert_eq!(restored.draft, saved, "{case}: corrected restart/resave");
}

#[test]
fn unit47_people_setup_home_cost_and_unchanged_navigation_keep_native_priority() {
    for (name, filing, key) in [
        (
            "head",
            Filing::HeadOfHousehold { paid_over_half_home: true, marital: HeadMarital::Unmarried },
            "HOH_NEEDS_HOME_COST",
        ),
        (
            "surviving",
            Filing::QualifyingSurvivingSpouse {
                spouse_died: DeathYear::DiedPriorYear,
                paid_over_half_home: true,
                could_file_joint: true,
            },
            "QSS_NEEDS_HOME_COST",
        ),
    ] {
        let original = complete(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, original.clone());
        let corrected = ReturnSetup { filing };
        let mut bad = corrected.clone();
        match &mut bad.filing {
            Filing::HeadOfHousehold { paid_over_half_home, .. }
            | Filing::QualifyingSurvivingSpouse { paid_over_half_home, .. } => {
                *paid_over_half_home = false
            }
            _ => unreachable!(),
        }
        admission_case(
            &alloc::format!("people-setup-{name}"),
            &mut session,
            save_setup(&original, bad),
            save_setup(&original, corrected),
            key,
            "[value/filing/paid_over_half_home]",
            TaxSection::Setup,
        );
    }
    // Existing invalid saved home-cost answers may navigate unchanged. This
    // is the pre-existing changed-only policy, not an eligibility exception.
    for status in [FederalStatus::Head, FederalStatus::Surviving] {
        let mut original = complete(status);
        original.setup.as_mut().unwrap().paid_over_half_home = false;
        let value = return_setup(original.setup.as_ref().unwrap());
        let mut session = Interview::new(TEST_USER, original.clone());
        let request = save_setup(&original, value);
        let reply = crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            admission_estimate(&request),
        );
        assert!(
            matches!(reply, Reply::Response(P::RunningEstimate { .. })),
            "unchanged setup estimate {reply:?}"
        );
        assert_eq!(session.draft, original);
        accepted(&send(&mut session, request));
        assert_eq!(session.draft, original, "unchanged setup must not rewrite accepted source");
    }
}

#[test]
fn unit47_people_filer_death_dates_locate_you_and_spouse_actual_payload() {
    for spouse in [false, true] {
        {
            let case = 1;
            let original =
                complete(if spouse { FederalStatus::Joint } else { FederalStatus::Single });
            let mut corrected = filer(&original.people.as_ref().unwrap().taxpayer);
            if spouse {
                corrected = filer(original.people.as_ref().unwrap().spouse.as_ref().unwrap());
            }
            corrected.birth_date = 20261231;
            corrected.details.more = MoreAboutYou::Died {
                date: 20261231,
                estate_medical: 0,
                bonds: None,
                ordinary: true,
            };
            let mut bad = corrected.clone();
            let MoreAboutYou::Died { date, .. } = &mut bad.details.more else { unreachable!() };
            *date = 20260101;
            let key = "DEATH_BEFORE_BIRTH";
            let name = if spouse { "spouse" } else { "you" };
            let make = |value| {
                if spouse { save_spouse(&original, value) } else { save_you(&original, value) }
            };
            let mut session = Interview::new(TEST_USER, original.clone());
            admission_case(
                &alloc::format!("people-{name}-death-{case}"),
                &mut session,
                make(bad),
                make(corrected),
                key,
                &alloc::format!("[{name}/details/more/date]"),
                if spouse { TaxSection::Spouse } else { TaxSection::People },
            );
        }
    }
}

#[test]
fn unit47_people_dependent_native_conditions_locate_actual_second_person() {
    // Cases 16 and 18 were the retired born-and-died fact's own rules, case
    // 13 the care costs' rule, now the care page's.
    for case in (0..20).filter(|case| ![13, 16, 18].contains(case)) {
        let mut original = complete(FederalStatus::Single);
        let mut corrected = row("Actual", "Dependent", 20180615);
        let supported = |support| DependentSituation::Supported {
            your_support_over_half: support,
            gross_income: 0,
        };
        match case {
            0 => corrected.facts = PersonFacts::ADOPTED_US_CITIZEN_PARENT.0,
            1 => corrected.facts = PersonFacts::MEDICAL_SEPARATED_PARENTS.0,
            2 | 7 => {
                corrected.birth_date = 19550101;
                corrected.situation = supported(true);
            }
            3 | 4 | 8 => corrected.facts = PersonFacts::YOU_RELEASED.0,
            5 => {
                corrected.facts = PersonFacts::RELEASED_TO_YOU.0;
                corrected.lived = LivedWithYou::PartOfYear { months: 6 };
            }
            6 | 9 => {
                corrected.birth_date = 19900615;
                corrected.situation = supported(false);
                corrected.facts = PersonFacts::MULTIPLE_SUPPORT.0;
            }
            10 => corrected.birth_date = 20080101,
            11 => {
                corrected.birth_date = 20030101;
                corrected.situation = DependentSituation::Student;
            }
            12 => {
                original.people.as_mut().unwrap().taxpayer.birth_date = 20100101;
                corrected.relationship = Relationship::Sibling;
            }
            14 | 15 => corrected.facts = PersonFacts::ADOPTED_ALL_YEAR.0,
            17 => {
                corrected.birth_date = 20260301;
                corrected.death_date = Some(20260401);
            }
            19 => {
                corrected.birth_date = 20000101;
                corrected.situation = supported(true);
                corrected.facts = PersonFacts::ANOTHERS_QUALIFYING_CHILD.0;
            }
            _ => unreachable!(),
        }
        let mut bad = corrected.clone();
        let (key, field) = match case {
            0 => {
                bad.relationship = Relationship::Sibling;
                ("MEDICAL_ADOPTED_CHILD_RELATIONSHIP", "relationship")
            }
            1 => {
                bad.relationship = Relationship::Sibling;
                ("MEDICAL_PARENT_SUPPORT_FACTS_CONFLICT", "facts")
            }
            2 => {
                bad.situation = supported(false);
                ("SUPPORTED_NEEDS_YOUR_SUPPORT", "situation/your_support_over_half")
            }
            3 => {
                bad.facts |= PersonFacts::RELEASED_TO_YOU.0;
                ("CUSTODY_RELEASE_ONE_WAY", "facts")
            }
            4 => {
                bad.relationship = Relationship::Sibling;
                ("CUSTODY_RELEASE_NEEDS_YOUR_CHILD", "relationship")
            }
            5 => {
                bad.lived = LivedWithYou::AllYear;
                ("CUSTODY_RELEASE_TO_YOU_NEEDS_FEWER_NIGHTS", "lived")
            }
            6 => {
                bad.situation = DependentSituation::ChildUnder19;
                ("MULTIPLE_SUPPORT_NEEDS_SHARED_SUPPORT", "situation")
            }
            7 => {
                bad.facts |= PersonFacts::OWN_SUPPORT.0;
                ("OWN_SUPPORT_NOT_WITH_YOUR_SUPPORT", "facts")
            }
            8 => {
                bad.facts |= PersonFacts::OWN_SUPPORT.0;
                ("OWN_SUPPORT_NOT_WITH_SHARED_SUPPORT", "facts")
            }
            9 => {
                bad.facts |= PersonFacts::YOU_RELEASED.0;
                ("MULTIPLE_SUPPORT_NOT_WITH_CUSTODY_RELEASE", "facts")
            }
            10 => {
                bad.birth_date = 20071231;
                ("DEPENDENT_AGE_SITUATION", "situation")
            }
            11 => {
                bad.birth_date = 20021231;
                ("DEPENDENT_AGE_SITUATION", "situation")
            }
            12 => {
                bad.birth_date = 20090101;
                ("DEPENDENT_OLDER_THAN_YOU", "birth_date")
            }
            14 => {
                bad.relationship = Relationship::Sibling;
                ("ADOPTED_NEEDS_CHILD_ALL_YEAR", "facts")
            }
            15 => {
                bad.lived = LivedWithYou::PartOfYear { months: 6 };
                ("ADOPTED_NEEDS_CHILD_ALL_YEAR", "facts")
            }
            17 => {
                bad.death_date = Some(20260228);
                ("DEATH_BEFORE_BIRTH", "death_date")
            }
            19 => {
                bad.situation = DependentSituation::ChildUnder19;
                ("DEPENDENT_AGE_SITUATION", "situation")
            }
            _ => unreachable!(),
        };
        // Case19 must reach its final flags/situation guard rather than an age guard.
        if case == 19 {
            corrected.birth_date = 20180615;
            bad.birth_date = 20180615;
        }
        let key = if case == 19 { "ANOTHERS_CHILD_NEEDS_SUPPORTED" } else { key };
        let mut earlier = row("Earlier", "Independent", 20190615);
        if case == 12 {
            earlier.relationship = Relationship::Sibling;
        }
        let mut session = Interview::new(TEST_USER, original.clone());
        admission_case(
            &alloc::format!("people-dependent-{case}"),
            &mut session,
            save_children(&original, vec![earlier.clone(), bad]),
            save_children(&original, vec![earlier, corrected]),
            key,
            &alloc::format!("[dependents/people/1/{field}]"),
            TaxSection::Children,
        );
    }
}

#[test]
fn unit47_people_educator_and_scholarship_admission_locate_actual_second_row() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let mut corrected = admission_open(&mut session, TaxSection::Educator);
    let P::SaveEducator { value, .. } = &mut corrected else { unreachable!() };
    value.people = vec![
        Educator {
            owner: Owner::Taxpayer,
            role: EducatorRole::Educator { classroom: 10_000, instructional: 0 },
        },
        Educator {
            owner: Owner::Spouse,
            role: EducatorRole::Educator { classroom: 20_000, instructional: 0 },
        },
    ];
    let mut bad = corrected.clone();
    let P::SaveEducator { value, .. } = &mut bad else { unreachable!() };
    value.people[1].owner = Owner::Taxpayer;
    admission_case(
        "people-educator-duplicate",
        &mut session,
        bad,
        corrected,
        "EDUCATOR_TWICE",
        "[value/people/1/owner]",
        TaxSection::Educator,
    );
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let taxpayer = accepted_document_names(&session.draft, Owner::Taxpayer).recipient;
    let spouse = accepted_document_names(&session.draft, Owner::Spouse).recipient;
    let mut sources = admission_open(&mut session, TaxSection::Documents);
    let P::SaveDocuments { documents, .. } = &mut sources else { unreachable!() };
    documents.common.education.students = vec![student(-1, 300_000), student(-2, 300_000)];
    documents.common.education.students[0].form.student = taxpayer;
    documents.common.education.students[1].form.student = spouse;
    documents.common.education.students[1].form.box_5 = 100_000;
    accepted(&send(&mut session, sources));
    let mut corrected = admission_open(&mut session, TaxSection::Education);
    let P::SaveEducation { value, .. } = &mut corrected else { unreachable!() };
    for row in &mut value.students {
        row.method = Some(EducationChoice::AmericanOpportunity);
        row.more.american_opportunity_used_up = Some(false);
    }
    value.students[1].more.scholarship_in_income =
        ScholarshipInclusion::Amount { included: 100_000 };
    let mut bad = corrected.clone();
    let P::SaveEducation { value, .. } = &mut bad else { unreachable!() };
    value.students[1].more.scholarship_in_income =
        ScholarshipInclusion::Amount { included: 100_001 };
    admission_case(
        "people-scholarship-included",
        &mut session,
        bad,
        corrected,
        "SCHOLARSHIP_INCLUDED_ABOVE_BOX_5",
        "[value/students/1/more/scholarship_in_income/included]",
        TaxSection::Education,
    );
}
