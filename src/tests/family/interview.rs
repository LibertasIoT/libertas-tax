// Record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::*;
use crate::interview::estimate_text;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::document;
use super::support::client::income_pages::{
    catalog_preview_document, open_document_center, open_topic, page_documents, save_documents,
    save_each_topic, save_topic,
};
use crate::tests::cross::interview::{
    PAGE_ADJUSTMENTS, PAGE_CHILDREN, PAGE_CREDITS, PAGE_PEOPLE, PAGE_REVIEW, PAGE_SETUP, adult,
    answer, assert_available_sections, at_the_credits_hub, back, checklist_of, continued,
    credits_hub, dependent_record, education_session, education_value, election_interest,
    election_page, fill_care_provider, from_review, go_on, go_to, household_return, hub_rows,
    hub_tail, import_unselected, m2_children, m2_household, more_credits_value, navigate,
    open_credits_topic, past_payments, plain_withdrawal, put_care_costs, q1099_page,
    reopened_people, row, row_of, save_care_costs, save_children, save_education,
    save_education_answers, save_more_credits, save_setup, save_you, saved_setup,
    send_past_less_common, shown, student, supporting, tick, value_of, with_tail,
};

// A household status saved from page 1 with the dependents answer Yes. The
// people page is unit M2-P2, so the return proceeds to the filer page.
fn save_household(filing: Filing) -> Interview {
    let mut s = Interview::new(TEST_USER, FederalDraft::empty("m2-household".into()));
    let request = save_setup(&s.draft, ReturnSetup { filing });
    let reply = send(&mut s, request);
    assert!(
        matches!(reply, Reply::Response(P::BeginPeople { page: PAGE_PEOPLE, .. })),
        "{reply:?}"
    );
    s
}
// The setup page as the review reopens it.
fn reopened_filing(s: &mut Interview) -> Filing {
    let back = P::Back {
        user: TEST_USER,
        cookie: s.draft.cookie.clone(),
        revision: s.draft.revision,
        previous: PAGE_SETUP,
    };
    let Reply::Edit(P::SaveSetup { value, .. }) = send(s, back) else {
        panic!("expected the setup page")
    };
    value.filing
}

#[test]
fn head_of_household_page_reaches_the_draft() {
    for (filing, status, marital_state, head) in [
        (
            Filing::HeadOfHousehold { paid_over_half_home: true, marital: HeadMarital::Unmarried },
            FilingChoice::Head,
            MaritalState::Unmarried,
            true,
        ),
        (
            Filing::HeadOfHousehold {
                paid_over_half_home: true,
                marital: HeadMarital::LivingApart,
            },
            FilingChoice::Head,
            MaritalState::Married { living: SpouseLiving::LastSixMonths },
            true,
        ),
        (
            Filing::QualifyingSurvivingSpouse {
                spouse_died: DeathYear::DiedPriorYear,
                paid_over_half_home: true,
                could_file_joint: true,
            },
            FilingChoice::Surviving,
            MaritalState::WidowedPriorYear { could_file_joint: true },
            false,
        ),
        (
            Filing::QualifyingSurvivingSpouse {
                spouse_died: DeathYear::DiedTwoYearsPrior,
                paid_over_half_home: true,
                could_file_joint: false,
            },
            FilingChoice::Surviving,
            MaritalState::WidowedTwoYearsPrior { could_file_joint: false },
            false,
        ),
    ] {
        let mut s = save_household(filing);
        assert_eq!(
            s.draft.setup,
            Some(FederalSetup {
                separate_spouse: SeparateSpouseBoxes::NotAnswered,
                label: crate::Message::new(
                    "LABEL_RETURN",
                    alloc::vec![crate::MessageArg::Number(2026)]
                )
                .encode(),
                basis: AmountBasis::Actual,
                status,
                marital_state,
                paid_over_half_home: true,
                community_property: CommunityProperty::NotAnswered,
            })
        );
        if head {
            assert_eq!(s.draft.revision, 1);
        }
        assert_eq!(reopened_filing(&mut s), filing);
    }
}

#[test]
fn household_status_needs_a_dependent_and_home_cost() {
    let head = |paid_over_half_home| Filing::HeadOfHousehold {
        paid_over_half_home,
        marital: HeadMarital::Unmarried,
    };
    let surviving = |paid_over_half_home| Filing::QualifyingSurvivingSpouse {
        spouse_died: DeathYear::DiedPriorYear,
        paid_over_half_home,
        could_file_joint: true,
    };
    // Gate fixes batch A: a household status's qualifying person is read
    // from the people page's list by the estimate, not asked on page 1.
    let cases = [(head(false), "HOH_NEEDS_HOME_COST"), (surviving(false), "QSS_NEEDS_HOME_COST")];
    for (_, key) in &cases {
        assert!(APP_STRINGS.iter().any(|(k, _)| k == key), "{key}");
    }
    // Server side: the save and the running estimate answer with the same key.
    let empty = FederalDraft::empty("m2-household-rules".into());
    let mut s = Interview::new(TEST_USER, empty.clone());
    let loaded = s.draft.clone();
    for (filing, key) in cases {
        let value = ReturnSetup { filing };
        let expected = libertas::libertas_formatted_text(key, &[]);
        for request in
            [save_setup(&empty, value.clone()), P::EstimateSetup { user: TEST_USER, value }]
        {
            let reply = send(&mut s, request);
            let Reply::Response(P::Problem { message, .. }) = reply else {
                panic!("expected a problem, got {reply:?}")
            };
            assert_eq!(message, expected, "{key}");
        }
        assert_eq!(s.draft, loaded);
        assert!(!s.dirty);
    }
}

#[test]
fn household_answer_is_stored_with_the_draft_and_reopened() {
    // A head-of-household return saved before any person is entered reopens
    // with its status, and the people page opens empty after the filer page.
    let filing =
        Filing::HeadOfHousehold { paid_over_half_home: true, marital: HeadMarital::Unmarried };
    let s = save_household(filing);
    assert!(s.draft.dependents().is_empty());
    let mut s = restart(s.draft.clone());
    let back = back(&s.draft, PAGE_SETUP);
    let Reply::Edit(P::SaveSetup { value, .. }) = send(&mut s, back) else {
        panic!("expected the setup page")
    };
    assert_eq!(value, ReturnSetup { filing });
    let request = save_you(&s.draft, adult("Robin", "Sample", 19850301));
    send(&mut s, request);
    let request = navigate(&s.draft, TaxSection::Children);
    assert!(matches!(
        send(&mut s, request),
        Reply::Response(P::BeginChildren { page: PAGE_CHILDREN, previous: PAGE_PEOPLE, .. })
    ));
    // Every return asks the people page once (gate fixes batch A): an empty
    // list is "no one", and a person can be added on any return.
    let mut single = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = navigate(&single.draft, TaxSection::Children);
    assert!(matches!(send(&mut single, request), Reply::Response(P::BeginChildren { .. })));
    let request = save_children(&single.draft.clone(), vec![row("Ava", "Sample", 20180615)]);
    accepted(&send(&mut single, request));
    assert_eq!(single.draft.dependents().len(), 1);
}

#[test]
fn empty_children_workflow_supplies_calculation_and_save_inputs() {
    use libertas::{AvroDecode, AvroEncode};

    for (death_date, expected_days, residence_choice) in [
        (None, 365, LivedWithYou::PartOfYear { months: 6 }),
        (Some(20260630), 181, LivedWithYou::Days { days: 90 }),
    ] {
        let mut draft = complete(FederalStatus::Single);
        if let Some(date) = death_date {
            draft.people.as_mut().unwrap().taxpayer.died =
                Some(FinalReturn { date, estate_medical: 0, bonds: None, ordinary: false });
        }
        let mut session = Interview::new(TEST_USER, draft.clone());
        let reply = send(&mut session, navigate(&draft, TaxSection::Children));
        let Reply::Response(P::BeginChildren { allowed_lived, days_alive, .. }) = &reply else {
            panic!("expected the first empty people page, got {reply:?}")
        };
        assert_eq!(*days_alive, expected_days);
        assert_eq!(
            *allowed_lived,
            vec![
                libertas_macros::variant_index!(LivedWithYou::AllYear) as i32,
                match residence_choice {
                    LivedWithYou::PartOfYear { .. } => {
                        libertas_macros::variant_index!(LivedWithYou::PartOfYear) as i32
                    }
                    LivedWithYou::Days { .. } => {
                        libertas_macros::variant_index!(LivedWithYou::Days) as i32
                    }
                    _ => unreachable!(),
                },
            ]
        );
        // Construct the workflow Request from the response's actual inputs,
        // without supplying the missing hidden values from this test.
        // Removing the only new person leaves this same empty payload.
        let request = answer(reply, Filing::Single);
        let P::SaveChildren { allowed_lived, days_alive, dependents, .. } = &request else {
            panic!("expected the people workflow Request")
        };
        assert!(dependents.people.is_empty());
        assert_eq!(*days_alive, expected_days);
        let calculation = P::EstimateDependents {
            user: TEST_USER,
            allowed_lived: allowed_lived.clone(),
            days_alive: *days_alive,
            dependents: dependents.clone(),
        };
        for value in [&calculation, &request] {
            let mut bytes = Vec::new();
            value.avro_encode(&mut bytes);
            assert_eq!(P::avro_decode(&bytes, &mut 0), Ok(value.clone()));
        }
        assert!(matches!(
            send(&mut session, calculation),
            Reply::Response(P::DependentsRunningEstimate { .. })
        ));
        assert_eq!(session.draft, draft);
        accepted(&send(&mut session, request));
        assert!(session.draft.dependents().is_empty());
    }
}

#[test]
fn dependents_page_maps_onto_dependent_records() {
    let mut s = supporting(FederalStatus::Single);
    let original = s.draft.clone();
    assert!(matches!(
        send(&mut s, navigate(&original, TaxSection::Children)),
        Reply::Response(P::BeginChildren { .. })
    ));
    let child = row("Ava B", "Sample", 20180615);
    let student = Dependent {
        situation: DependentSituation::Student,
        identification: Identification::Itin,
        ..row("Ben", "Sample", 20040310)
    };
    let parent = Dependent {
        relationship: Relationship::Parent,
        situation: DependentSituation::Supported {
            your_support_over_half: true,
            gross_income: 300_000,
        },
        identification: Identification::NoneByDueDate,
        ..row("Carol", "Van Sample", 19550101)
    };
    let part_year = Dependent {
        lived: LivedWithYou::PartOfYear { months: 5 },
        ..row("Dan", "Sample", 20190901)
    };
    let seven_months = Dependent {
        lived: LivedWithYou::PartOfYear { months: 7 },
        ..row("Eve", "Sample", 20170101)
    };
    let newborn = row("Finn", "Sample", 20260301);
    let people = vec![
        child.clone(),
        student.clone(),
        parent.clone(),
        part_year.clone(),
        seven_months.clone(),
        newborn.clone(),
    ];
    send(&mut s, save_children(&original, people));
    assert!(s.dirty);
    let first = original.next_id;
    let expected = vec![
        dependent_record(first, "Ava B", "Sample", 20180615),
        FederalDependent {
            situation: DependentSituation::Student,
            valid_ssn: false,
            ..dependent_record(first + 1, "Ben", "Sample", 20040310)
        },
        FederalDependent {
            relationship: Relationship::Parent,
            situation: DependentSituation::Supported {
                your_support_over_half: true,
                gross_income: 300_000,
            },
            valid_ssn: false,
            valid_tin: false,
            ..dependent_record(first + 2, "Carol", "Van Sample", 19550101)
        },
        // Five months is 150 days, not more than half the year.
        FederalDependent {
            residency: DependentResidency::Days { days: 150 },
            us_home_over_half_year: false,
            ..dependent_record(first + 3, "Dan", "Sample", 20190901)
        },
        // Seven months is 210 days, more than half the year.
        FederalDependent {
            residency: DependentResidency::Days { days: 210 },
            ..dependent_record(first + 4, "Eve", "Sample", 20170101)
        },
        // A child born in 2026 who lived with you all year uses the exception.
        FederalDependent {
            residency: DependentResidency::WholeYearException,
            ..dependent_record(first + 5, "Finn", "Sample", 20260301)
        },
    ];
    // A new person's care is not asked yet.
    let expected: Vec<_> =
        expected.into_iter().map(|record| FederalDependent { care: None, ..record }).collect();
    assert_eq!(s.draft.dependents, expected);
    assert_eq!(s.draft.next_id, first + 6);
    assert_eq!(s.draft.revision, original.revision + 1);
    assert_eq!(s.draft.credits.as_ref().unwrap().care, FederalCare::default());

    // Reopening shows every row as entered, with its identity.
    let accepted = s.draft.clone();
    let mut s = restart(accepted.clone());
    let reopened = reopened_people(&mut s);
    let ids: Vec<i64> = reopened.iter().map(|p| p.id).collect();
    assert_eq!(ids, (first..first + 6).collect::<Vec<_>>());
    let mut shown = vec![child, student, parent, part_year, seven_months, newborn];
    for (row, id) in shown.iter_mut().zip(first..) {
        row.id = id;
    }
    assert_eq!(reopened, shown);
    // An unchanged save stores nothing, and the running estimate is pure.
    send(&mut s, save_children(&accepted, reopened.clone()));
    assert_eq!(s.draft, accepted);
    assert!(!s.dirty);
    let reply = send(
        &mut s,
        P::EstimateDependents {
            user: TEST_USER,
            dependents: Dependents { people: reopened },
            allowed_lived: Vec::new(),
            days_alive: 365,
        },
    );
    assert!(matches!(reply, Reply::Response(P::DependentsRunningEstimate { .. })), "{reply:?}");
    assert_eq!(s.draft, accepted);
    assert!(!s.dirty);
}

// The people page shows each person's child tax credit, credit for other
// dependents and earned income credit, worked out from its answers (IRC
// sections 24(c), 24(h)(4) and 32(c)(3); 2026 Schedule 8812 lines 5 and 7),
// while editing and when it reopens; nothing about them is asked or stored.
#[test]
fn people_page_shows_each_persons_credits() {
    use libertas::AvroDecode;
    let mut s = supporting(FederalStatus::Single);
    let original = s.draft.clone();
    let people = vec![
        row("Ava", "Sample", 20180615),
        // 17 at the end of 2026: no child tax credit, still an EIC child.
        row("Ben", "Sample", 20090101),
        Dependent {
            relationship: Relationship::Parent,
            situation: DependentSituation::Supported {
                your_support_over_half: true,
                gross_income: 0,
            },
            ..row("Pat", "Sample", 19550101)
        },
        // An ITIN: the credit for other dependents, no EIC child (Pub. 596 Rule 2).
        Dependent { identification: Identification::Itin, ..row("Dee", "Sample", 20180615) },
        Dependent { competing: CompetingClaim::Other, ..row("Cai", "Sample", 20180615) },
    ];
    let expected = [
        ("PERSON_CREDITS_CHILD_EIC", 2),
        ("PERSON_CREDITS_OTHER_EIC", 2),
        ("PERSON_CREDITS_OTHER", 2),
        ("PERSON_CREDITS_OTHER", 2),
        ("PERSON_CREDITS_NONE", 1),
    ];
    let keys = |credits: &[PersonCredits]| {
        credits
            .iter()
            .map(|row| {
                let (key, args) =
                    crate::common::FormattedMessage::avro_decode(&row.credits, &mut 0).unwrap();
                (key, args.len())
            })
            .collect::<Vec<_>>()
    };
    let shown = |credits: &[PersonCredits]| {
        keys(credits)
            .into_iter()
            .zip(expected)
            .all(|((key, args), (want, count))| key == want && args == count)
            && credits.len() == expected.len()
    };
    let reply = send(
        &mut s,
        P::EstimateDependents {
            user: TEST_USER,
            dependents: Dependents { people: people.clone() },
            allowed_lived: Vec::new(),
            days_alive: 365,
        },
    );
    let Reply::Response(P::DependentsRunningEstimate { credits, .. }) = reply else {
        panic!("expected the people page's running estimate, got {reply:?}")
    };
    assert!(shown(&credits), "{:?}", keys(&credits));
    assert_eq!(s.draft, original);
    accepted(&send(&mut s, save_children(&original, people)));
    let request = navigate(&s.draft, TaxSection::Children);
    let reply = send(&mut s, request);
    let Reply::Edit(P::SaveChildren { credits, .. }) = reply else {
        panic!("expected the people page, got {reply:?}")
    };
    assert!(shown(&credits), "{:?}", keys(&credits));
    // The estimate counts the same people: Ava at $2,200, and Ben, Pat and
    // Dee at $500 each, before the income limit.
    let r = crate::federal_rules::estimate(&s.draft);
    assert_eq!(super::support::assert::value(&r, "child_credit_children_amount"), 2_200);
    assert_eq!(super::support::assert::value(&r, "child_credit_other_dependents_amount"), 1_500);
}

#[test]
fn care_costs_open_the_household_care_claim() {
    let mut s = supporting(FederalStatus::Single);
    let original = s.draft.clone();
    let people = vec![row("Ava", "Sample", 20180615), row("Ben", "Sample", 20200101)];
    send(&mut s, save_children(&original, people));
    // The care page takes each person's costs.
    let ava = CareCosts::Yes { expenses: 300_000, incapable_self_care: false };
    save_care_costs(&mut s, &[ava, CareCosts::No]);
    let d = &s.draft;
    assert_eq!(d.dependents()[0].care, Some(DependentCare::Yes { care_expenses: 300_000 }));
    assert!(!d.dependents()[0].incapable_self_care);
    assert_eq!(d.dependents()[1].care, Some(DependentCare::No));
    assert_eq!(d.checklist.paid & PaidTopics::CARE.0, PaidTopics::CARE.0);

    // Care for a person 13 or older who cannot care for themselves; the
    // people page keeps the care answers.
    let accepted = s.draft.clone();
    let mut people = reopened_people(&mut s);
    people[1] =
        Dependent { situation: DependentSituation::Disabled, ..row("Ben", "Sample", 20050101) };
    people[1].id = accepted.dependents()[1].id;
    send(&mut s, save_children(&accepted, people.clone()));
    assert_eq!(s.draft.dependents[0].care, Some(DependentCare::Yes { care_expenses: 300_000 }));
    save_care_costs(
        &mut s,
        &[ava, CareCosts::Yes { expenses: 100_000, incapable_self_care: true }],
    );
    assert!(s.draft.dependents[1].incapable_self_care && s.draft.dependents[1].disabled());
    assert_eq!(reopened_people(&mut s)[1], people[1]);

    // Unchecking the care box removes the claim.
    let reply = from_review(&mut s, TaxSection::Checklist);
    let mut ticks = checklist_of(&reply);
    ticks.paid &= !PaidTopics::CARE.0;
    let reply = send(&mut s, tick(reply, ticks));
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    assert_eq!(s.draft.credits.as_ref().unwrap().care, FederalCare::default());
    // Each person's care is then unasked again.
    assert!(s.draft.dependents().iter().all(|d| d.care.is_none()));
}

#[test]
fn dependent_rules_reject_on_save() {
    for key in [
        "SUPPORTED_NEEDS_YOUR_SUPPORT",
        "CUSTODY_RELEASE_ONE_WAY",
        "CUSTODY_RELEASE_NEEDS_YOUR_CHILD",
        "CUSTODY_RELEASE_TO_YOU_NEEDS_FEWER_NIGHTS",
        "MULTIPLE_SUPPORT_NEEDS_SHARED_SUPPORT",
        "MULTIPLE_SUPPORT_NOT_WITH_CUSTODY_RELEASE",
        "DEPENDENT_AGE_SITUATION",
        "DEPENDENT_OLDER_THAN_YOU",
        "ADOPTED_NEEDS_CHILD_ALL_YEAR",
    ] {
        assert!(APP_STRINGS.iter().any(|(k, _)| *k == key), "{key}");
    }
    let supported = |your_support_over_half| DependentSituation::Supported {
        your_support_over_half,
        gross_income: 0,
    };
    // The filer of `supporting` was born June 15, 1980.
    let cases = [
        (
            Dependent { situation: supported(false), ..row("Pat", "Sample", 19550101) },
            "SUPPORTED_NEEDS_YOUR_SUPPORT",
        ),
        // IRC 152(c)(3)(A): under 19 at the end of 2026, and a child attains an
        // age on the birthday (Rev. Rul. 2003-72). Born December 31, 2007 the
        // 19th birthday is December 31, 2026: refused. Born January 1, 2008 is
        // still 18 (the 2026 Schedule EIC line 3, "If born after 2007").
        (row("Ava", "Sample", 20071231), "DEPENDENT_AGE_SITUATION"),
        // IRC 152(c)(3)(B): a student under 24; born December 31, 2002
        // attained 24 on December 31, 2026.
        (
            Dependent { situation: DependentSituation::Student, ..row("Ben", "Sample", 20021231) },
            "DEPENDENT_AGE_SITUATION",
        ),
        // The adopted-child exception is your own child living with you all year.
        (
            Dependent {
                facts: PersonFacts::ADOPTED_ALL_YEAR.0,
                relationship: Relationship::Sibling,
                ..row("Eli", "Sample", 20180101)
            },
            "ADOPTED_NEEDS_CHILD_ALL_YEAR",
        ),
        (
            Dependent {
                facts: PersonFacts::ADOPTED_ALL_YEAR.0,
                lived: LivedWithYou::PartOfYear { months: 6 },
                ..row("Fay", "Sample", 20180101)
            },
            "ADOPTED_NEEDS_CHILD_ALL_YEAR",
        ),
    ];
    let mut s = supporting(FederalStatus::Single);
    let loaded = s.draft.clone();
    let check = |s: &mut Interview, d: &FederalDraft, person: Dependent, key: &str| {
        let expected = libertas::libertas_formatted_text(key, &[]);
        for request in [
            save_children(d, vec![person.clone()]),
            P::EstimateDependents {
                user: TEST_USER,
                dependents: Dependents { people: vec![person.clone()] },
                allowed_lived: Vec::new(),
                days_alive: 365,
            },
        ] {
            let reply = send(s, request);
            let Reply::Response(P::Problem { message, .. }) = reply else {
                panic!("expected a problem, got {reply:?}")
            };
            assert_eq!(message, expected, "{key}");
        }
        assert_eq!(s.draft, *d, "{key}");
        assert!(!s.dirty, "{key}");
    };
    for (person, key) in cases {
        check(&mut s, &loaded, person, key);
    }
    // The other side of each boundary date is accepted.
    let accepted = vec![
        row("Ava", "Sample", 20080101),
        Dependent { situation: DependentSituation::Student, ..row("Ben", "Sample", 20030101) },
        Dependent {
            identification: Identification::NoneByDueDate,
            death_date: Some(20260401),
            ..row("Dov", "Sample", 20260301)
        },
        Dependent { situation: supported(true), ..row("Pat", "Sample", 19550101) },
        Dependent {
            facts: PersonFacts::ADOPTED_ALL_YEAR.0 | PersonFacts::ABROAD.0,
            ..row("Eli", "Sample", 20180101)
        },
    ];
    send(&mut s, save_children(&loaded, accepted));
    assert_eq!(s.draft.dependents().len(), 5);

    // A child under 19 or a student must be younger than you, or than either
    // spouse on a joint return.
    let young = |status| {
        let mut s = supporting(status);
        let people = s.draft.people.as_mut().unwrap();
        people.taxpayer.birth_date = 20100101;
        if let Some(spouse) = people.spouse.as_mut() {
            spouse.birth_date = 20100101;
        }
        let d = s.draft.clone();
        (s, d)
    };
    let older = row("Dee", "Sample", 20090101);
    let (mut s, d) = young(FederalStatus::Single);
    check(&mut s, &d, older.clone(), "DEPENDENT_OLDER_THAN_YOU");
    let student = Dependent { situation: DependentSituation::Student, ..older.clone() };
    let (mut s, d) = young(FederalStatus::Joint);
    check(&mut s, &d, student.clone(), "DEPENDENT_OLDER_THAN_YOU");
    // Younger than the joint return's other spouse is enough.
    s.draft.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19800615;
    let d = s.draft.clone();
    send(&mut s, save_children(&d, vec![student]));
    assert_eq!(s.draft.dependents().len(), 1);
    // A disabled person may be older than you.
    let (mut s, d) = young(FederalStatus::Single);
    let disabled = Dependent { situation: DependentSituation::Disabled, ..older };
    send(&mut s, save_children(&d, vec![disabled]));
    assert_eq!(s.draft.dependents().len(), 1);
}

// Forms 8332 and 2120: the six client rules over the Bitflags facts, each
// refused on save and in the running estimate with its key; the server twin of
// each is dependent_rule. The facts and the stored situation round-trip
// through the page, including the released qualifying relative (the situation
// "Another person you support" with support No, which no age can tell apart
// from a child under 19).
#[test]
fn custody_release_and_multiple_support_rules_reject_on_save() {
    let supported = |your_support_over_half| DependentSituation::Supported {
        your_support_over_half,
        gross_income: 200_000,
    };
    let part_year = |months| LivedWithYou::PartOfYear { months };
    let with = |facts: PersonFacts, base: Dependent| Dependent { facts: facts.0, ..base };
    let released = |bits: u32| bits;
    let either = PersonFacts::YOU_RELEASED.0 | PersonFacts::RELEASED_TO_YOU.0;
    let cases = [
        // The support test: the agreement or the release waives it.
        (
            Dependent { situation: supported(false), ..row("Pat", "Sample", 19550101) },
            "SUPPORTED_NEEDS_YOUR_SUPPORT",
        ),
        (
            Dependent { facts: released(either), ..row("Ava", "Sample", 20180615) },
            "CUSTODY_RELEASE_ONE_WAY",
        ),
        (
            with(
                PersonFacts::YOU_RELEASED,
                Dependent {
                    relationship: Relationship::Grandchild,
                    ..row("Ben", "Sample", 20180615)
                },
            ),
            "CUSTODY_RELEASE_NEEDS_YOUR_CHILD",
        ),
        (
            with(
                PersonFacts::RELEASED_TO_YOU,
                Dependent {
                    relationship: Relationship::Foster,
                    lived: part_year(3),
                    ..row("Cai", "Sample", 20180615)
                },
            ),
            "CUSTODY_RELEASE_NEEDS_YOUR_CHILD",
        ),
        // A release to you needs the child to have lived with you fewer nights:
        // all year and seven months are refused, six months is accepted below.
        (
            with(PersonFacts::RELEASED_TO_YOU, row("Dee", "Sample", 20180615)),
            "CUSTODY_RELEASE_TO_YOU_NEEDS_FEWER_NIGHTS",
        ),
        (
            with(
                PersonFacts::RELEASED_TO_YOU,
                Dependent { lived: part_year(7), ..row("Eve", "Sample", 20180615) },
            ),
            "CUSTODY_RELEASE_TO_YOU_NEEDS_FEWER_NIGHTS",
        ),
        // The agreement is for another person you support with support No.
        (
            with(
                PersonFacts::MULTIPLE_SUPPORT,
                Dependent { relationship: Relationship::Parent, ..row("Fay", "Sample", 19500310) },
            ),
            "MULTIPLE_SUPPORT_NEEDS_SHARED_SUPPORT",
        ),
        (
            with(
                PersonFacts::MULTIPLE_SUPPORT,
                Dependent {
                    relationship: Relationship::Parent,
                    situation: supported(true),
                    ..row("Gil", "Sample", 19500310)
                },
            ),
            "MULTIPLE_SUPPORT_NEEDS_SHARED_SUPPORT",
        ),
        // IRC 152(e)(5): the agreement and a release exclude each other.
        (
            Dependent {
                facts: PersonFacts::MULTIPLE_SUPPORT.0 | PersonFacts::RELEASED_TO_YOU.0,
                lived: part_year(3),
                situation: supported(false),
                ..row("Hal", "Sample", 20050501)
            },
            "MULTIPLE_SUPPORT_NOT_WITH_CUSTODY_RELEASE",
        ),
        (
            Dependent {
                facts: PersonFacts::MULTIPLE_SUPPORT.0 | PersonFacts::YOU_RELEASED.0,
                situation: supported(false),
                ..row("Ida", "Sample", 20050501)
            },
            "MULTIPLE_SUPPORT_NOT_WITH_CUSTODY_RELEASE",
        ),
        // IRC 152(c)(1)(D): a person who paid more than half of their own
        // support cannot also be mostly supported by you, by the parents who
        // release the claim, or by several supporters together.
        (
            Dependent {
                facts: PersonFacts::OWN_SUPPORT.0,
                situation: supported(true),
                ..row("Jon", "Sample", 20050501)
            },
            "OWN_SUPPORT_NOT_WITH_YOUR_SUPPORT",
        ),
        (
            Dependent {
                facts: PersonFacts::OWN_SUPPORT.0 | PersonFacts::YOU_RELEASED.0,
                ..row("Kai", "Sample", 20050501)
            },
            "OWN_SUPPORT_NOT_WITH_SHARED_SUPPORT",
        ),
        (
            Dependent {
                facts: PersonFacts::OWN_SUPPORT.0 | PersonFacts::MULTIPLE_SUPPORT.0,
                situation: supported(false),
                ..row("Lee", "Sample", 20050501)
            },
            "OWN_SUPPORT_NOT_WITH_SHARED_SUPPORT",
        ),
    ];
    let mut s = supporting(FederalStatus::Single);
    let loaded = s.draft.clone();
    for (person, key) in cases {
        let expected = libertas::libertas_formatted_text(key, &[]);
        for request in [
            save_children(&loaded, vec![person.clone()]),
            P::EstimateDependents {
                user: TEST_USER,
                dependents: Dependents { people: vec![person.clone()] },
                allowed_lived: Vec::new(),
                days_alive: 365,
            },
        ] {
            let reply = send(&mut s, request);
            let Reply::Response(P::Problem { message, .. }) = reply else {
                panic!("expected a problem for {key}, got {reply:?}")
            };
            assert_eq!(message, expected, "{key}");
        }
        assert_eq!(s.draft, loaded, "{key}");
        assert!(!s.dirty, "{key}");
    }

    // The accepted combinations, each reopening exactly as entered.
    let people = vec![
        // The custodial parent who released.
        with(PersonFacts::YOU_RELEASED, row("Ava", "Sample", 20180615)),
        // The parent holding the release: six months is the most.
        with(
            PersonFacts::RELEASED_TO_YOU,
            Dependent { lived: part_year(6), ..row("Ben", "Sample", 20180615) },
        ),
        // Fixture E: the released qualifying relative aged 20, entered as
        // another person you support with support No.
        with(
            PersonFacts::RELEASED_TO_YOU,
            Dependent {
                lived: part_year(3),
                situation: supported(false),
                ..row("Cai", "Sample", 20050501)
            },
        ),
        // A released child under 19 may also be entered as supported.
        with(
            PersonFacts::RELEASED_TO_YOU,
            Dependent {
                lived: part_year(0),
                situation: supported(false),
                ..row("Dee", "Sample", 20120101)
            },
        ),
        // The multiple support agreement.
        with(
            PersonFacts::MULTIPLE_SUPPORT,
            Dependent {
                relationship: Relationship::Parent,
                lived: part_year(0),
                situation: supported(false),
                ..row("Eve", "Sample", 19500310)
            },
        ),
        // A married child, with the citizen or resident fact.
        Dependent {
            facts: PersonFacts::MARRIED_AT_YEAR_END.0 | PersonFacts::ABROAD.0,
            ..row("Fay", "Sample", 20100101)
        },
        // A working 17-year-old who pays more than half of their own support.
        with(PersonFacts::OWN_SUPPORT, row("Gus", "Sample", 20090115)),
    ];
    send(&mut s, save_children(&loaded, people.clone()));
    let stored = s.draft.dependents();
    assert_eq!(stored.len(), 7);
    assert!(stored[6].own_support_over_half());
    assert!(stored[..6].iter().all(|p| !p.own_support_over_half()));
    assert_eq!(stored[0].custody_release(), CustodyRelease::YouReleased);
    assert_eq!(stored[1].custody_release(), CustodyRelease::ReleasedToYou);
    assert_eq!(stored[2].custody_release(), CustodyRelease::ReleasedToYou);
    assert!(matches!(
        stored[2].situation,
        DependentSituation::Supported { your_support_over_half: false, .. }
    ));
    assert!(matches!(stored[3].situation, DependentSituation::Supported { .. }));
    assert!(stored[4].multiple_support());
    assert_eq!(stored[4].custody_release(), CustodyRelease::None);
    assert!(stored[5].married_at_year_end() && !stored[5].us_citizen_resident());
    assert!(
        stored[..5]
            .iter()
            .all(|p| p.competing == CompetingClaim::NoOne && !p.married_at_year_end())
    );
    let mut shown = people;
    let reopened = reopened_people(&mut s);
    for (row, record) in shown.iter_mut().zip(s.draft.dependents()) {
        row.id = record.id;
    }
    assert_eq!(reopened, shown);
    // An unchanged save stores nothing.
    let accepted = s.draft.clone();
    let mut s = restart(accepted.clone());
    send(&mut s, save_children(&accepted, reopened));
    assert_eq!(s.draft, accepted);
    assert!(!s.dirty);
}

#[test]
fn care_costs_are_accepted_for_a_child_turning_13_in_2026() {
    // Form 2441 credits care given before a child's 13th birthday, so a
    // child who turns 13 during 2026 (born after January 2, 2013: the age is
    // attained the day before the birthday, Pub. 501) still qualifies for
    // the part of the year before that birthday (federal_rules.rs family(),
    // under_13_in_year). The care page must accept the same child it
    // refuses one day earlier.
    let mut s = supporting(FederalStatus::Single);
    let original = s.draft.clone();
    // Turns 13 on 2026-06-15.
    send(&mut s, save_children(&original, vec![row("Cai", "Sample", 20130615)]));
    let care = CareCosts::Yes { expenses: 300_000, incapable_self_care: false };
    save_care_costs(&mut s, &[care]);
    assert_eq!(s.draft.dependents()[0].care, Some(DependentCare::Yes { care_expenses: 300_000 }));
    // The return credits the care: `supporting` (complete()) gives a single
    // filer $60,000 in wages and no other income or adjustments, so AGI =
    // $60,000. Form 2441 line 8 rate (forms/form_2441.rs care_rate): first
    // term max(35, 50 - ceiling($60,000-$15,000, $2,000)) = max(35, 50-23)
    // = 35; the AGI phase-down above $75,000 (single) does not apply
    // (ceiling of a negative amount is 0); rate = max(20, 35) = 35%. One
    // qualifying person (Cai) caps expenses at $3,000, all of which is under
    // the $60,000 earned-income limit, so care_cost = $3,000. Credit (line
    // 11, "care_credit") = $3,000 x 35% = $1,050.
    let r = supported(&s.draft);
    assert_eq!(value(&r, "care_credit"), 1050);
}

#[test]
fn care_costs_are_refused_for_a_child_already_13_on_january_1() {
    // IRC 21(b)(1)(A): a child born on or before January 1, 2013 had attained
    // 13 by January 1, 2026 (on the birthday), so no part of the year falls
    // before the 13th birthday (federal_rules.rs family(), under_13_in_year);
    // the care page refuses the costs one day before the child in
    // care_costs_are_accepted_for_a_child_turning_13_in_2026 who qualifies.
    let mut s = supporting(FederalStatus::Single);
    let original = s.draft.clone();
    send(&mut s, save_children(&original, vec![row("Cai", "Sample", 20130101)]));
    let loaded = s.draft.clone();
    let mut page = open_credits_topic(&mut s, TaxSection::Care);
    put_care_costs(&mut page, &[CareCosts::Yes { expenses: 300_000, incapable_self_care: false }]);
    rejected_with(send(&mut s, page), "CARE_NEEDS_UNDER_13_OR_SELF_CARE");
    assert_eq!(s.draft, loaded);
}

#[test]
fn dependents_page_calculates_the_running_estimate() {
    let mut s = household_return(vec![row("Ava", "Sample", 20180214)]);
    let accepted = s.draft.clone();
    let running = |s: &mut Interview, people: Vec<Dependent>| match send(
        s,
        P::EstimateDependents {
            user: TEST_USER,
            dependents: Dependents { people },
            allowed_lived: Vec::new(),
            days_alive: 365,
        },
    ) {
        Reply::Response(P::DependentsRunningEstimate { estimate, progress, .. }) => {
            (shown(&estimate), shown(&progress))
        }
        other => panic!("expected the running estimate, got {other:?}"),
    };
    // The accepted list estimates as the accepted return.
    let one = reopened_people(&mut s);
    let (text, progress) = running(&mut s, one.clone());
    assert_eq!(text, shown(&estimate_text(&estimate(&accepted))));
    // Documents and income without a form follow the household pages.
    assert!(progress.contains("section 3 of 11"), "{progress}");
    // The edited list is estimated merged into the accepted return, exactly
    // as its save stores it, and nothing is stored.
    let mut two = one;
    two.extend(m2_children().into_iter().skip(1));
    let (text, _) = running(&mut s, two.clone());
    assert_ne!(text, shown(&estimate_text(&estimate(&accepted))));
    assert_eq!(s.draft, accepted);
    assert!(!s.dirty);
    send(&mut s, save_children(&accepted, two));
    assert_eq!(text, shown(&estimate_text(&estimate(&s.draft))));
    // A refused row gets the reply its save would send.
    let older = row("Dee", "Sample", 19700101);
    error(send(
        &mut s,
        P::EstimateDependents {
            user: TEST_USER,
            dependents: Dependents { people: vec![older] },
            allowed_lived: Vec::new(),
            days_alive: 365,
        },
    ));
}

#[test]
fn dependent_names_reopen_split() {
    let people = vec![row("Carol", "Van Sample", 20150101), row("Mary Ann", "Lee", 20160101)];
    let s = household_return(people.clone());
    let stored: Vec<_> = s
        .draft
        .dependents()
        .iter()
        .map(|d| (d.first_name.as_str(), d.last_name.as_str(), status_text(&d.name())))
        .collect();
    assert_eq!(
        stored,
        vec![
            ("Carol", "Van Sample", alloc::string::String::from("Carol Van Sample")),
            ("Mary Ann", "Lee", alloc::string::String::from("Mary Ann Lee")),
        ]
    );
    // The label is the person-name resource with the typed first and last name.
    assert_eq!(
        s.draft.dependents()[0].name(),
        crate::Message::new(
            "PERSON_NAME",
            vec![
                crate::MessageArg::Text("Carol".into()),
                crate::MessageArg::Text("Van Sample".into()),
            ],
        )
        .encode()
    );
    let mut s = restart(s.draft);
    let reopened = reopened_people(&mut s);
    let names: Vec<_> =
        reopened.iter().map(|p| (p.first_name.as_str(), p.last_name.as_str())).collect();
    assert_eq!(names, vec![("Carol", "Van Sample"), ("Mary Ann", "Lee")]);
    // Saving the reopened page unchanged stores nothing.
    let accepted = s.draft.clone();
    send(&mut s, save_children(&accepted, reopened));
    assert_eq!(s.draft, accepted);
    assert!(!s.dirty);
}

// T2-fx: the employer-paid interest (section 127 educational assistance)
// is stored on the claim and reopens unchanged; more than the interest
// entered is refused with the key the client rule shares.
#[test]
fn employer_paid_student_loan_interest_round_trips_and_is_checked() {
    let d = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d);
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::StudentLoan);
    assert!(matches!(send(&mut s, request), Reply::Edit(P::SaveStudentLoan { .. })));
    let mut forms = vec![box_2(180_000, 20_000)];
    forms[0].document.borrower = fixture_document_names(&s.draft, -1).unwrap().recipient;
    let save =
        |s: &mut Interview, forms: Vec<crate::StudentLoanSource>, other: i64, employer: i64| {
            let mut sources = crate::tests::support::client::income_pages::open_document_center(s);
            let P::SaveDocuments { documents, .. } = &mut sources else { unreachable!() };
            documents.common.student_loans.documents = student_documents(&forms);
            accepted(&send(s, sources));
            let navigation = go_to(&s.draft, PAGE_REVIEW, TaxSection::StudentLoan);
            let Reply::Edit(mut request @ P::SaveStudentLoan { .. }) = send(s, navigation) else {
                panic!("expected student loan topic")
            };
            let P::SaveStudentLoan { value, .. } = &mut request else { unreachable!() };
            value.other_interest = other;
            value.employer_paid_interest = employer;
            for (row, form) in value.forms.iter_mut().zip(forms) {
                if form.document.box_2 {
                    row.additional_interest = crate::StudentLoanAdditionalInterest::Checked {
                        additional_interest: form.additional_interest,
                    };
                }
            }
            send(s, request)
        };
    // $1,800.00 box 1 + $200.00 box 2 + $100.00 with no form = $2,100.00;
    // an employer payment of $2,100.01 is one cent too many (server only:
    // the client rule cannot sum the forms).
    let reply = save(&mut s, forms.clone(), 10_000, 210_001);
    rejected_with(reply, "STUDENT_LOAN_EMPLOYER_ABOVE_INTEREST");
    // With no form the client rule's case: more than the no-form interest.
    let reply = save(&mut s, vec![], 10_000, 10_001);
    rejected_with(reply, "STUDENT_LOAN_EMPLOYER_ABOVE_INTEREST");
    // Employer payment alone is not a claim of interest.
    let reply = save(&mut s, vec![], 0, 1);
    rejected_with(reply, "STUDENT_LOAN_EMPLOYER_ABOVE_INTEREST");
    // Exactly the whole $2,100.00 is accepted and stored.
    let accepted_form_id = s.draft.next_id;
    let reply = save(&mut s, forms.clone(), 10_000, 210_000);
    forms[0].id = accepted_form_id;
    assert!(matches!(reply, Reply::Response(P::Review { .. })));
    assert_eq!(
        s.draft.adjustments.as_ref().unwrap().student_loan,
        StudentLoanClaim {
            student_loan_interest: 10_000,
            forms: forms
                .iter()
                .cloned()
                .map(|source| crate::StudentLoanSource {
                    assigned: crate::DocumentRecipient::Taxpayer,
                    owner_source: Some(crate::OwnerSource::Matched),
                    ..source
                })
                .collect(),
            employer_paid_interest: 210_000,
        }
    );
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::StudentLoan);
    let Reply::Edit(P::SaveStudentLoan { value, .. }) = send(&mut s, request) else {
        panic!("expected the student loan page")
    };
    assert_eq!(
        value.forms.into_iter().map(|row| row.value).collect::<Vec<_>>(),
        forms.iter().map(|source| source.document.clone()).collect::<Vec<_>>()
    );
    assert_eq!(value.other_interest, 10_000);
    assert_eq!(value.employer_paid_interest, 210_000);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
}

#[test]
fn aotc_is_derived_for_an_undergraduate() {
    // Form 8863 Part III lines 27-30 and Part I lines 1-8 by hand: single,
    // MAGI $60,000 (below the $80,000 phase-out start, i8863 line 3), you as
    // an at-least-half-time undergraduate (1098-T box 8 checked, box 9 not):
    //   qualified expenses = tuition 6,000 + books from the school 500 +
    //   other required books 300 - scholarships 1,000 = 5,800
    //   line 30 = 2,000 + 25% x min(5,800 - 2,000, 2,000) = 2,500
    //   line 8 refundable = 40% x 2,500 = 1,000; line 9 = 1,500, which the
    //   $5,020 tax on $43,900 of taxable income covers.
    let (mut s, hub) = at_the_credits_hub(complete(FederalStatus::Single));
    // No student on this return, no contribution: the hub suggests Continue
    // and lists only the topics that come later.
    let P::Hub { suggested, allowed_sections, previous, progress, .. } = &hub else {
        unreachable!()
    };
    // M3-P19: Continue and Back go to the shown sections around it: other
    // taxes and adjustments, which every return walks for their less-common
    // pages (gate fixes batch A); deductions have nothing, and federal
    // payments is walked only with its box, so the last page follows.
    assert_eq!(*suggested, TaxSection::OftenMissed);
    assert_eq!(*previous, PAGE_ADJUSTMENTS);
    // M3-P18: every hub offers the checklist after its previous section.
    assert_available_sections(
        allowed_sections,
        &[
            TaxSection::Adjustments,
            TaxSection::Checklist,
            TaxSection::Care,
            TaxSection::Education,
            TaxSection::Marketplace,
            TaxSection::MoreCredits,
            TaxSection::OtherTaxes,
        ]
        .map(|t| t as i32),
    );
    // Documents and income without a form precede the checklist and financial sections.
    assert!(shown(progress).contains("section 9 of 11"));
    // Amended for M3-N1: every offered topic and Continue are rows.
    assert_eq!(
        hub_rows(&hub),
        with_tail(
            vec![row_of("Child and dependent care", "None", TaxSection::Care)],
            hub_tail(&[TaxSection::Education, TaxSection::Marketplace], TaxSection::MoreCredits)
        )
    );
    let page = crate::tests::support::client::income_pages::open_document_center(&mut s);
    let value = education_value(&page).clone();
    assert!(value.students.is_empty());
    let labels: Vec<_> = value.people.iter().map(|p| status_text(&p.label)).collect();
    assert_eq!(labels, vec!["You"]);
    let mut row = student(-1, 600_000);
    row.form.box_5 = 100_000;
    let student_document_id = s.draft.next_id;
    let request = save_education(
        &mut s,
        EducationCredits { students: vec![row.clone()], people: value.people.clone() },
    );
    accepted(&send(&mut s, request));
    let (_, mut annual) =
        native_education(-1, education_expenses(0), EducationChoice::AmericanOpportunity);
    annual.more.books_from_school = 50_000;
    annual.more.other_books = 30_000;
    accepted(&save_education_answers(&mut s, vec![annual]));
    let hub = credits_hub(crate::tests::cross::interview::from_review(&mut s, TaxSection::Credits));
    let credits = s.draft.credits.as_ref().unwrap();
    assert_eq!(
        credits.education,
        vec![EducationSource {
            id: student_document_id,
            student: -1,
            form: {
                let mut form = row.form.clone();
                form.student = accepted_document_names(&s.draft, Owner::Taxpayer).recipient;
                form
            },
            assigned: crate::DocumentRecipient::Taxpayer,
            owner_source: Some(crate::OwnerSource::Matched),
            joint_share: None,
        }]
    );
    assert_eq!(credits.students[0].more.books_from_school, 50_000);
    assert_eq!(credits.students[0].more.other_books, 30_000);
    assert_eq!(crate::federal_rules::people_students(&s.draft)[0].id, -1);
    assert!(s.draft.finished_sections.contains(&TaxSection::Documents));
    assert_eq!(
        hub_rows(&hub).into_iter().find(|row| row.2 == TaxSection::Education).unwrap(),
        row_of("Education credits", "1 student", TaxSection::Education)
    );
    let r = continued(&mut s);
    assert_eq!(value_of(&r, "refundable_education_credit"), 1_000);
    assert_eq!(value_of(&r, "education_credit"), 1_500);
    // The page reopens with the same row, and its running estimate agrees.
    let page = crate::tests::support::client::income_pages::open_document_center(&mut s);
    row.id = student_document_id;
    row.form.student = accepted_document_names(&s.draft, Owner::Taxpayer).recipient;
    assert_eq!(education_value(&page).students, vec![row]);
    let revision = s.draft.revision;
    accepted(&send(&mut s, page));
    assert_eq!(s.draft.revision, revision);
    // The completed summary remains available and its action returns to Review.
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::Credits);
    let hub = credits_hub(send_past_less_common(&mut s, request));
    let reply = send(&mut s, go_on(&hub));
    assert!(matches!(past_payments(&mut s, reply), Reply::Response(P::Review { .. })));
    assert!(s.draft.finished_sections.contains(&TaxSection::Credits));
}

#[test]
fn care_row_opens_the_people_page_and_returns_to_the_hub() {
    // Care owns each person's cost and the full Form 2441 editor.
    let mut session = m2_household();
    session.draft.credits.as_mut().unwrap().care.providers.clear();
    session.draft.finished_sections.retain(|section| *section != TaxSection::Care);
    let (mut s, hub) = at_the_credits_hub(session.draft);
    assert_eq!(
        hub_rows(&hub)[0],
        row_of("Child and dependent care", "Not started", TaxSection::Care)
    );
    let P::Hub { allowed_sections, .. } = &hub else { unreachable!() };
    assert!(allowed_sections.contains(&(TaxSection::Care as i32)));
    let mut page = open_credits_topic(&mut s, TaxSection::Care);
    let P::SaveCare { previous, review, value: facts, .. } = &page else { panic!() };
    assert_eq!((*previous, *review), (PAGE_CREDITS, false));
    assert_eq!(facts.qualifying_people.len(), 2);
    assert_eq!(facts.providers.len(), 0);
    let before = s.draft.clone();
    credits_hub(send_past_less_common(&mut s, back(&before, PAGE_CREDITS)));
    assert_eq!(s.draft, before);
    fill_care_provider(&mut page);
    accepted(&send(&mut s, page));
    assert_eq!(s.draft.credits.as_ref().unwrap().care.providers.len(), 1);
    assert!(s.draft.finished_sections.contains(&TaxSection::Care));
    assert_eq!(value(&supported(&s.draft), "care_credit"), 1388);
}

#[test]
fn education_is_suggested_while_a_student_is_on_the_return() {
    // Student status alone does not require entering a source document.
    // Education sources remain available through the document center.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.full_time_student = true;
    let (mut s, hub) = at_the_credits_hub(d);
    let P::Hub { suggested, .. } = &hub else { unreachable!() };
    assert_eq!(*suggested, TaxSection::OftenMissed);
    assert_eq!(
        hub_rows(&hub).into_iter().find(|row| row.2 == TaxSection::Education),
        Some(row_of("Education credits", "Not started", TaxSection::Education))
    );
    let request = open_document_center(&mut s);
    let education = education_value(&request);
    assert!(education.people.iter().any(|person| person.id == -1));
    assert!(education.students.is_empty());
    accepted(&send(&mut s, request));
    let hub = credits_hub(crate::tests::cross::interview::from_review(&mut s, TaxSection::Credits));
    let P::Hub { suggested, .. } = &hub else { unreachable!() };
    // Continue goes to the next shown section, the last page before the
    // review (federal payments only with its box).
    assert_eq!(*suggested, TaxSection::OftenMissed);
    assert_eq!(
        hub_rows(&hub).into_iter().find(|row| row.2 == TaxSection::Education),
        Some(row_of("Education credits", "Not started", TaxSection::Education))
    );
}

fn election_children(page: &P) -> Vec<(i64, alloc::string::String)> {
    let P::SaveIncomes { children, .. } = page else {
        panic!("expected the income documents page, got {page:?}")
    };
    children.iter().map(|c| (c.id, status_text(&c.label))).collect()
}

#[test]
fn child_election_page_offers_the_kind_only_with_a_child_the_return_can_report() {
    let kind = libertas_macros::variant_index!(crate::IncomeSourceKind::Form8814) as i32;
    // No child: the kind is not offered.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let page = open_topic(&mut s, TaxSection::InterestDividends);
    let P::SaveIncomes { allowed_documents, .. } = &page else { unreachable!() };
    assert!(!allowed_documents.contains(&kind));
    assert!(election_children(&page).is_empty());
    // The catalog is the children, adopted children and stepchildren who are
    // under 19, or full-time students under 24, at the end of 2026 by the
    // day-exact rule (born 2008-01-01 is 19, born 2003-01-01 is 24), with no
    // joint return of their own; a foster child files their own return.
    let mut d = complete(FederalStatus::Single);
    let mut people = Vec::new();
    for (name, birth, student, foster, joint) in [
        ("Eighteen", 20080102, false, false, false),
        ("Nineteen", 20080101, false, false, false),
        ("Student", 20030102, true, false, false),
        ("Old student", 20030101, true, false, false),
        ("Not a student", 20050615, false, false, false),
        ("Foster", 20150615, false, true, false),
        ("Married", 20100615, false, false, true),
    ] {
        let id = d.next_id;
        add_child(&mut d);
        let dependent = d.dependents.last_mut().unwrap();
        dependent.first_name = name.into();
        dependent.last_name = "Test".into();
        dependent.birth_date = birth;
        dependent.situation =
            if student { DependentSituation::Student } else { DependentSituation::ChildUnder19 };
        set_fact(dependent, PersonFacts::FILED_JOINTLY, joint);
        if foster {
            dependent.relationship = Relationship::Foster;
        }
        people.push((id, name));
    }
    let mut s = Interview::new(TEST_USER, d);
    let page = open_topic(&mut s, TaxSection::InterestDividends);
    let P::SaveIncomes { allowed_documents, allowed_other_parent, .. } = &page else {
        unreachable!()
    };
    assert!(allowed_documents.contains(&kind));
    assert_eq!(*allowed_other_parent, OtherParentChoice::allowed(true));
    assert_eq!(
        election_children(&page),
        vec![
            (people[0].0, alloc::string::String::from("Eighteen Test")),
            (people[2].0, alloc::string::String::from("Student Test"))
        ]
    );
    // Only Interest and dividends offers the kind and the catalog.
    let wages = open_topic(&mut s, TaxSection::Wages);
    // Wages uses its own typed editor, with no child-election kind or catalog.
    assert!(matches!(wages, P::SaveWagesAnswers { .. }));
    // A joint return needs no answer about the other parent.
    let mut joint = complete(FederalStatus::Joint);
    add_child(&mut joint);
    let mut s = Interview::new(TEST_USER, joint);
    let page = open_topic(&mut s, TaxSection::InterestDividends);
    let P::SaveIncomes { allowed_other_parent, .. } = &page else { unreachable!() };
    assert_eq!(*allowed_other_parent, OtherParentChoice::allowed(false));
}

#[test]
fn child_election_row_saves_reopens_and_keeps_what_an_import_cannot_write() {
    let mut d = complete(FederalStatus::Single);
    let child = d.next_id;
    add_child(&mut d);
    let mut s = Interview::new(TEST_USER, d.clone());
    let mut interest = election_interest(300_000);
    interest.more.foreign_tax = 1_000;
    interest.more.exempt = 20_000;
    interest.payer.federal_id = "12-3456789".into();
    let mut dividends = native_div1099("Child's Fund".into(), 250_000, 200_000);
    dividends.capital_distributions = 40_000;
    dividends.collectible_distributions = 10_000;
    let dividends = div1099_source(dividends);
    let dividends =
        crate::child_election::ChildDividendStatement { id: 0, document: dividends.form };
    let mut row = election_page(
        child,
        vec![
            ChildStatement::Int1099(interest.interest_source()),
            ChildStatement::Div1099(dividends),
        ],
    );
    row.alaska_dividend = 150_000;
    row.nontaxable_social = 120_000;
    row.facts = ChildElectionFacts::FOREIGN_ACCOUNT.0;
    accepted(&{
        let request = save_topic(
            &mut s,
            TaxSection::InterestDividends,
            vec![document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Form8814(row.clone()) },
            )],
        );
        send(&mut s, request)
    });
    // The stored row is named by its child; the small foreign tax of a
    // child's statement is not claimed on the parent's Form 1116 election.
    let stored = s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::ChildElection { .. }))
        .unwrap();
    assert_eq!(stored.label, "");
    assert_eq!(s.draft.credits.as_ref().unwrap().foreign_tax, ForeignTaxClaim::No);
    let FederalIncome::ChildElection { data } = &stored.income else { unreachable!() };
    assert_eq!(data.child_id, child);
    assert_eq!(data.statements.len(), 2);
    // Reopened as typed, payer, foreign tax and state boxes included.
    let page = open_topic(&mut s, TaxSection::InterestDividends);
    let reopened = page_documents(&page).to_vec();
    assert_eq!(reopened.len(), 1);
    let ChildStatement::Int1099(expected_interest) = &mut row.statements[0] else { unreachable!() };
    expected_interest.calls = IntCalls { nominee: vec![0], ..Default::default() };
    // The saved dividend statement carries its accepted identity.
    let IncomeSource::Record { value: crate::IncomeRecord::Form8814(reopened_row) } =
        &reopened[0].document
    else {
        unreachable!()
    };
    let ChildStatement::Div1099(saved_dividends) = &reopened_row.statements[1] else {
        unreachable!()
    };
    let ChildStatement::Div1099(expected_dividends) = &mut row.statements[1] else {
        unreachable!()
    };
    expected_dividends.id = saved_dividends.id;
    assert_eq!(
        reopened[0].document,
        IncomeSource::Record { value: crate::IncomeRecord::Form8814(row.clone()) }
    );
    // Applying an import to one statement (its boxes replaced, as the import
    // contract applies them) leaves the person's own answers as typed.
    let IncomeSource::Record { value: crate::IncomeRecord::Form8814(mut imported) } =
        reopened[0].document.clone()
    else {
        unreachable!()
    };
    let ChildStatement::Int1099(statement) = &mut imported.statements[0] else { unreachable!() };
    statement.form.interest_income = 310_000;
    statement.form.payer.name = "Imported Bank".into();
    let request = save_topic(
        &mut s,
        TaxSection::InterestDividends,
        vec![document(
            reopened[0].id,
            IncomeSource::Record { value: crate::IncomeRecord::Form8814(imported) },
        )],
    );
    accepted(&send(&mut s, request));
    let page = open_topic(&mut s, TaxSection::InterestDividends);
    let IncomeSource::Record { value: crate::IncomeRecord::Form8814(now) } =
        &page_documents(&page)[0].document
    else {
        unreachable!()
    };
    assert_eq!(now.alaska_dividend, 150_000);
    assert_eq!(now.nontaxable_social, 120_000);
    assert_eq!(now.other_parent, OtherParentChoice::LivedApart);
    assert_eq!(now.facts, ChildElectionFacts::FOREIGN_ACCOUNT.0);
    let ChildStatement::Int1099(kept) = &now.statements[0] else { unreachable!() };
    assert_eq!(
        (kept.form.interest_income, kept.form.payer.name.as_str()),
        (310_000, "Imported Bank")
    );
    assert_eq!(kept.form.foreign_tax_paid, 1_000);
    // The person-supplied fields have no import marker; the three statements
    // are the filer's own importable structs.
    let source = include_str!("../../family/child_election.rs");
    assert!(import_unselected(source, "pub struct Form8814 {"));
    let model = crate::federal_model::HUB_SOURCE;
    for item in [
        "pub struct Form1099IntPaper {",
        "pub struct Form1099DivPaper {",
        "pub struct Form1099OidPaper {",
    ] {
        let head = &model[..model.find(item).unwrap()];
        assert!(
            head[head.rfind("#[libertas_llm_doc_trans(").unwrap()..]
                .contains("validation = \"$.document_facts\""),
            "{item}"
        );
    }
    assert!(source.contains("Int1099(crate::Form1099IntSource)"));
    assert!(source.contains("Div1099(ChildDividendStatement)"));
    assert!(source.contains("Oid1099(crate::Form1099OidSource)"));
}

#[test]
fn child_dividend_answers_follow_statement_ids_through_correction_and_restart() {
    use crate::tests::support::client::exchange;

    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    let child = d.dependents[0].id;
    let foreign_id = d.income[0].id;
    let first = crate::Form1099DivPaper {
        calendar_year: Some(2026),
        payer: crate::PrintedParty::from_name("Sample Child Fund".into()),
        recipient: crate::PrintedParty::from_name("Ava Sample".into()),
        payer_telephone: "555-0100".into(),
        account_number: "001-A".into(),
        ordinary: Some(180_000),
        qualified: Some(60_000),
        capital_distributions: Some(3_000),
        section_1202_gain: Some(1_000),
        section199a: Some(800),
        nondividend_distributions: Some(12_000),
        investment_expenses: Some(400),
        fatca: true,
        ..Default::default()
    };
    let second = crate::Form1099DivPaper {
        account_number: "002-B".into(),
        ordinary: Some(170_000),
        nondividend_distributions: Some(8_000),
        investment_expenses: Some(500),
        ..first.clone()
    };
    let row = election_page(
        child,
        [first.clone(), second.clone()]
            .into_iter()
            .map(|document| {
                ChildStatement::Div1099(crate::child_election::ChildDividendStatement {
                    id: 0,
                    document,
                })
            })
            .collect(),
    );
    add(&mut d, child_election(child, row.record().statements, 0, OtherParentChoice::LivedApart));
    let election = d.income.last().unwrap();
    let election_id = election.id;
    let FederalIncome::ChildElection { data } = &election.income else { unreachable!() };
    let ids: Vec<_> = data
        .statements
        .iter()
        .map(|statement| match statement {
            ChildIncome::Dividend { statement_id, .. } => *statement_id,
            _ => unreachable!(),
        })
        .collect();
    assert!(ids.iter().all(|id| *id > 0));
    assert_ne!(ids[0], ids[1]);
    let first_lot = crate::CapitalReturn1099Div {
        kind: crate::CapitalDistributionKind::Nondividend,
        amount: 12_000,
        property: vec![],
        basis: 20_000,
        term: GainTerm::Long,
        special: false,
    };
    let second_lot = crate::CapitalReturn1099Div {
        amount: 8_000,
        basis: 15_000,
        term: GainTerm::Short,
        ..first_lot.clone()
    };
    let answers = [
        (ids[0], first_lot, crate::DividendSpecial::NoneOfThese),
        (ids[1], second_lot, crate::DividendSpecial::Another),
    ];
    let mut s = Interview::new(TEST_USER, d);
    let mut request = open_topic(&mut s, TaxSection::InterestDividends);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    // Form 8814 consumes these answers, even when the paper also prints REIT
    // and section 1202 amounts. It asks no unused nominee or fund questions.
    // Keyed questions retain their complete identity independently of display order.
    let mut actual_keys = followups.iter().map(|row| row.key.clone()).collect::<Vec<_>>();
    let mut expected_keys = ids
        .iter()
        .flat_map(|id| {
            ["capital_returns", "special"]
                .map(|suffix| alloc::format!("income/{election_id}/statement/{id}/{suffix}"))
        })
        .collect::<Vec<_>>();
    actual_keys.sort();
    expected_keys.sort();
    assert_eq!(actual_keys, expected_keys);
    for (id, lot, treatment) in &answers {
        let key = alloc::format!("income/{election_id}/statement/{id}");
        followups
            .iter_mut()
            .find(|row| row.key == alloc::format!("{key}/capital_returns"))
            .unwrap()
            .answer = crate::DocumentAnswer::DividendCapitalReturns { value: vec![lot.clone()] };
        followups
            .iter_mut()
            .find(|row| row.key == alloc::format!("{key}/special"))
            .unwrap()
            .answer = crate::DocumentAnswer::DividendTreatment { value: *treatment };
    }
    accepted(&send(&mut s, request));

    let mut request = open_topic(&mut s, TaxSection::InterestDividends);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let crate::IncomeRecord::Form8814(row) = &mut documents.documents[0].document else {
        unreachable!()
    };
    for (statement, (id, paper)) in row.statements.iter().zip([(ids[0], &first), (ids[1], &second)])
    {
        let ChildStatement::Div1099(statement) = statement else { unreachable!() };
        assert_eq!((statement.id, &statement.document), (id, paper));
    }
    row.statements.reverse();
    let ChildStatement::Div1099(corrected) = &mut row.statements[1] else { unreachable!() };
    corrected.document.corrected = true;
    corrected.document.ordinary = Some(corrected.document.ordinary.unwrap_or(0) + 1_000);
    accepted(&send(&mut s, request));

    let corrected_first = crate::Form1099DivPaper {
        corrected: true,
        ordinary: Some(first.ordinary.unwrap_or(0) + 1_000),
        ..first
    };
    let election = s.draft.income.iter().find(|entry| entry.id == election_id).unwrap();
    let FederalIncome::ChildElection { data } = &election.income else { unreachable!() };
    for (statement, (id, paper)) in
        data.statements.iter().zip([(ids[1], &second), (ids[0], &corrected_first)])
    {
        let ChildIncome::Dividend { statement_id, data } = statement else { unreachable!() };
        assert_eq!((*statement_id, &data.form), (id, paper));
        assert_eq!(data.form.ordinary, paper.ordinary);
        let (_, lot, treatment) = answers.iter().find(|(source, ..)| *source == id).unwrap();
        assert_eq!(data.capital_returns, vec![lot.clone()]);
        assert_eq!(data.special, *treatment);
    }
    let reopened = open_topic(&mut s, TaxSection::InterestDividends);
    // restart projects real return_records keys and Avro values, decodes them,
    // and assembles the accepted return before opening the editor again.
    let mut loaded = restart(s.draft.clone());
    assert_eq!(loaded.draft, s.draft);
    let restarted = open_topic(&mut loaded, TaxSection::InterestDividends);
    assert_eq!(page_documents(&restarted), page_documents(&reopened));
    let P::SaveIncomes { followups: expected, .. } = reopened else { unreachable!() };
    let P::SaveIncomes { followups: actual, .. } = restarted else { unreachable!() };
    assert_eq!(actual, expected);

    // Both are valid positive IDs; neither may transfer another row's answers.
    for (id, key) in [
        (ids[1], "ISSUE_INTERVIEW_A_RECORD_IDENTITY_IS_REPEATED_IN_EE6DC425"),
        (foreign_id, "ISSUE_INTERVIEW_A_RECORD_NO_LONGER_BELONGS_TO_51E52C2C"),
    ] {
        let mut request = open_topic(&mut loaded, TaxSection::InterestDividends);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        let crate::IncomeRecord::Form8814(row) = &mut documents.documents[0].document else {
            unreachable!()
        };
        let ChildStatement::Div1099(statement) = &mut row.statements[1] else { unreachable!() };
        statement.id = id;
        rejected_with(exchange(&mut loaded, request, "child dividend identity"), key);
    }
}

#[test]
fn child_election_save_checks_the_child_the_choice_and_the_statements() {
    let mut d = complete(FederalStatus::Single);
    let child = d.next_id;
    add_child(&mut d);
    let row = |edit: &dyn Fn(&mut Form8814)| {
        let mut row = election_page(
            child,
            vec![ChildStatement::Int1099(election_interest(300_000).interest_source())],
        );
        edit(&mut row);
        document(0, IncomeSource::Record { value: crate::IncomeRecord::Form8814(row) })
    };
    let save = |d: &FederalDraft, rows: Vec<IncomeDocumentItem>| {
        let mut s = Interview::new(TEST_USER, d.clone());
        {
            let request = save_topic(&mut s, TaxSection::InterestDividends, rows);
            send(&mut s, request)
        }
    };
    rejected_with(save(&d, vec![row(&|_| {}), row(&|_| {})]), "CHILD_ELECTION_TWICE");
    rejected_with(
        {
            let mut source = Interview::new(TEST_USER, d.clone());
            current_document_refusal(
                "income-source-1267",
                &mut source,
                vec![row(&|r| r.child = 999)],
                "CHILD_ELECTION_CHILD_NOT_ELIGIBLE",
                "[documents/documents/0/document/child]",
            )
        },
        "CHILD_ELECTION_CHILD_NOT_ELIGIBLE",
    );
    rejected_with(
        {
            let mut source = Interview::new(TEST_USER, d.clone());
            current_document_refusal(
                "income-source-1268",
                &mut source,
                vec![row(&|r| r.other_parent = OtherParentChoice::NotNeeded)],
                "CHILD_ELECTION_PARENT_NEEDED",
                "[documents/documents/0/document/other_parent]",
            )
        },
        "CHILD_ELECTION_PARENT_NEEDED",
    );
    // A statement keeps the rules of the filer's own 1099 page.
    rejected_with(
        save(
            &d,
            vec![row(&|r| {
                r.statements =
                    vec![ChildStatement::Div1099(crate::child_election::ChildDividendStatement {
                        id: 0,
                        document: dividend_document(native_div1099("".into(), 100, 200)),
                    })]
            })],
        ),
        "DIV_QUALIFIED_ABOVE_ORDINARY",
    );
    // A joint return needs no answer about the other parent.
    let mut joint = complete(FederalStatus::Joint);
    let child = joint.next_id;
    add_child(&mut joint);
    let mut page = election_page(child, vec![]);
    rejected_with(
        {
            let mut source = Interview::new(TEST_USER, joint.clone());
            current_document_refusal(
                "income-source-1273",
                &mut source,
                vec![document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::Form8814(page.clone()) },
                )],
                "CHILD_ELECTION_PARENT_NOT_NEEDED",
                "[documents/documents/0/document/other_parent]",
            )
        },
        "CHILD_ELECTION_PARENT_NOT_NEEDED",
    );
    page.other_parent = OtherParentChoice::NotNeeded;
    accepted(&save(
        &joint,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Form8814(page) })],
    ));
}

#[test]
fn child_election_preview_is_line_6_and_zero_for_a_row_left_out() {
    let mut d = complete(FederalStatus::Single);
    let child = d.next_id;
    add_child(&mut d);
    let mut s = Interview::new(TEST_USER, d);
    let preview = |s: &mut Interview, row: Form8814| {
        let request = catalog_preview_document(IncomeSource::Record {
            value: crate::IncomeRecord::Form8814(row),
        });
        match send(s, request) {
            Reply::Response(P::IncomePreview { preview, .. }) => preview,
            other => panic!("expected a preview, got {other:?}"),
        }
    };
    // Interest of $4,000: line 6 = 4,000 - 2,700 = 1,300.
    let row = election_page(
        child,
        vec![ChildStatement::Int1099(election_interest(400_000).interest_source())],
    );
    assert_eq!(preview(&mut s, row.clone()), Some(130_000));
    // At most $1,350 nothing is reported; a row left out previews zero.
    let small = election_page(
        child,
        vec![ChildStatement::Int1099(election_interest(130_000).interest_source())],
    );
    assert_eq!(preview(&mut s, small), Some(0));
    let mut out = row;
    out.facts = ChildElectionFacts::OTHER_INCOME.0;
    assert_eq!(preview(&mut s, out), Some(0));
}

#[test]
fn child_election_is_named_by_its_child_and_is_removed_with_it() {
    let mut s = supporting(FederalStatus::Single);
    let original = s.draft.clone();
    accepted(&send(&mut s, save_children(&original, vec![row("Ava", "Sample", 20180615)])));
    let child = s.draft.dependents()[0].id;
    let page = election_page(
        child,
        vec![ChildStatement::Int1099(election_interest(300_000).interest_source())],
    );
    let request = save_topic(
        &mut s,
        TaxSection::InterestDividends,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Form8814(page) })],
    );
    accepted(&send(&mut s, request));
    let label = |s: &Interview| {
        s.draft
            .income
            .iter()
            .find(|e| matches!(e.income, FederalIncome::ChildElection { .. }))
            .map(|e| e.label.clone())
    };
    // The row stores no name: it is named by its child on the people page, so a
    // rename there needs nothing carried to the row.
    assert_eq!(label(&s).as_deref(), Some(""));
    let mut people = reopened_people(&mut s);
    people[0].first_name = "Avery".into();
    let request = save_children(&s.draft.clone(), people);
    accepted(&send(&mut s, request));
    assert_eq!(label(&s).as_deref(), Some(""));
    // A child the return can no longer report (here a foster child, who files
    // their own return) takes the row with them, named in the running
    // estimate's warning before the save.
    let mut people = reopened_people(&mut s);
    people[0].relationship = Relationship::Foster;
    let Reply::Response(P::DependentsRunningEstimate { warnings, .. }) = send(
        &mut s,
        P::EstimateDependents {
            user: TEST_USER,
            dependents: Dependents { people: people.clone() },
            allowed_lived: Vec::new(),
            days_alive: 365,
        },
    ) else {
        panic!("expected a running estimate")
    };
    let warnings: Vec<Message> = warnings.iter().map(|w| status_message(&w.message)).collect();
    assert!(warnings.contains(&Message::new(
        "SAVING_REMOVES_ONE_NAMED",
        vec![
            MessageArg::Count(1),
            MessageArg::Resource("SAVING_KIND_FORM_8814_ONE".into()),
            MessageArg::Nested(alloc::boxed::Box::new(Message::new(
                "TEXT",
                vec![MessageArg::Nested(alloc::boxed::Box::new(Message::new(
                    "PERSON_NAME",
                    vec![MessageArg::Text("Avery".into()), MessageArg::Text("Sample".into()),],
                )))],
            ))),
        ],
    )));
    let request = save_children(&s.draft.clone(), people);
    accepted(&send(&mut s, request));
    assert_eq!(label(&s), None);
}

#[test]
fn education_account_page_round_trips_forms_and_expenses_by_identity() {
    let (mut session, child) = education_session();
    let page = open_topic(&mut session, TaxSection::OtherIncome);
    let P::SaveIncomes { people, source_choices, .. } = &page else { unreachable!() };
    // You, your spouse and the dependent, keyed by identity.
    assert_eq!(people.iter().map(|p| p.id).collect::<Vec<_>>(), vec![-1, -2, child]);
    assert!(source_choices.record.contains(
        &(libertas_macros::variant_index!(crate::IncomeRecord::EducationAccountExpenses) as i32)
    ));
    let center = open_document_center(&mut session);
    let P::SaveDocuments { allowed_documents, .. } = &center else { unreachable!() };
    assert!(
        allowed_documents
            .contains(&(libertas_macros::variant_index!(crate::IncomeSourceKind::Q1099) as i32))
    );
    let owned = |owner, id, document| IncomeDocumentItem {
        refund_management: None,
        refund_management_needed: None,
        allowed_community: Vec::new(),
        investment_caps: None,
        id,
        owner,
        document,
        preview: None,
        community: CommunityShare::NotNeeded,
    };
    let mut for_child = q1099_page(
        2_500_000,
        EducationAccount::Qtp { earnings: 500_000, basis: 2_000_000 },
        plain_withdrawal(),
        Q1099Beneficiary::Student { person: child },
    );
    let mut spouse_form = {
        let mut value = q1099_page(
            100_000,
            EducationAccount::Coverdell { amounts: CoverdellAmounts::ValueOnly { value: 180_000 } },
            Q1099Transfer::Withdrawal { reason: DistributionReason::Death },
            Q1099Beneficiary::OtherStudent {
                expenses: 900_000,
                assistance: 100_000,
                credit_expenses: 0,
            },
        );
        value.coverdell_basis_start = Some(210_000);
        value.coverdell_contributions = 40_000;
        value.late_rollovers = Some(0);
        value.redeposited = 10_000;
        value
    };
    let mut roth = {
        let mut value = q1099_page(
            600_000,
            EducationAccount::Qtp { earnings: 200_000, basis: 400_000 },
            Q1099Transfer::RothIra,
            Q1099Beneficiary::Recipient,
        );
        value.roth_rollover = Some(qtp_roth_rollover().rollover);
        value
    };
    let record = EducationAccountExpenses {
        students: vec![
            EducationAccountStudent {
                person: child,
                college: CollegeCosts::EducationCredit,
                other_college: 120_000,
                k12: 2_500_000,
                k12_coverdell: 100_000,
                apprenticeship: 5_000,
                loan_payments: 700_000,
                loan_payments_before: 600_000,
                academy_costs: 3_000,
                used_elsewhere: 4_000,
            },
            EducationAccountStudent {
                person: -2,
                college: CollegeCosts::Entered {
                    tuition_books: 2_000_000,
                    assistance: 300_000,
                    credit_elsewhere: 400_000,
                },
                other_college: 0,
                k12: 0,
                k12_coverdell: 0,
                apprenticeship: 0,
                loan_payments: 0,
                loan_payments_before: 0,
                academy_costs: 0,
                used_elsewhere: 0,
            },
        ],
    };
    let documents = vec![
        owned(
            crate::DocumentOwner::Yours,
            0,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Q1099(education_distribution_document(
                    for_child.clone(),
                )),
            },
        ),
        owned(
            crate::DocumentOwner::Spouses,
            0,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Q1099(education_distribution_document(
                    spouse_form.clone(),
                )),
            },
        ),
        owned(
            crate::DocumentOwner::Yours,
            0,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Q1099(education_distribution_document(
                    roth.clone(),
                )),
            },
        ),
        owned(
            crate::DocumentOwner::Yours,
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::EducationAccountExpenses(record.clone()),
            },
        ),
    ];
    let documents = fixture_owned_rows(&session.draft, documents);
    save_each_topic(&mut session, documents);
    // The beneficiary answer remains separate from printed box 6; a recipient who is the
    // beneficiary is not asked, so the stored answer stays empty.
    assert!(matches!(
        session.draft.income.iter().find_map(|entry| match &entry.income {
            FederalIncome::EducationDistribution { data } if data.form.qtp_to_roth_ira =>
                Some(&data.beneficiary),
            _ => None,
        }),
        Some(Some(FederalEducationBeneficiary::Recipient))
    ));
    // Stored by identity, labelled by kind, owned as entered.
    let entries: Vec<_> = session.draft.income.iter().skip(1).collect();
    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0].label, "");
    assert_eq!(entries[1].owner, crate::DocumentOwner::Spouses);
    assert_eq!(entries[3].label, "");
    let FederalIncome::EducationDistribution { data } = &entries[0].income else { unreachable!() };
    assert_eq!(data.beneficiary, Some(FederalEducationBeneficiary::Student { person: child }));
    let FederalIncome::EducationAccountExpenses { data } = &entries[3].income else {
        unreachable!()
    };
    assert_eq!(data.students.iter().map(|s| s.person).collect::<Vec<_>>(), vec![child, -2]);
    // The stored return reopens as the same page rows, with the previews the
    // forms can state: none for a withdrawal, zero for a Roth transfer.
    session.draft = FederalDraft::from_avro(&session.draft.to_avro()).unwrap();
    let mut rows: Vec<_> = page_documents(&open_document_center(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::OtherIncome { .. }))
        .collect();
    rows.extend(page_documents(&open_topic(&mut session, TaxSection::OtherIncome)));
    let reopened: Vec<_> =
        rows.iter().map(|item| (item.owner.owned(), item.document.clone(), item.preview)).collect();
    assert_eq!(reopened.len(), 4);
    for_child.form.recipient = accepted_document_names(&session.draft, Owner::Taxpayer).recipient;
    spouse_form.form.recipient = accepted_document_names(&session.draft, Owner::Spouse).recipient;
    roth.form.recipient = accepted_document_names(&session.draft, Owner::Taxpayer).recipient;
    assert_eq!(
        reopened[0],
        (
            Owner::Taxpayer,
            IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Q1099(for_child.form) },
            None
        )
    );
    assert_eq!(
        reopened[1],
        (
            Owner::Spouse,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Q1099(spouse_form.form)
            },
            None
        )
    );
    assert_eq!(
        reopened[2].1,
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Q1099(roth.form) }
    );
    assert_eq!(reopened[2].2, Some(0));
    assert_eq!(
        reopened[3].1,
        IncomeSource::Record {
            value: crate::IncomeRecord::EducationAccountExpenses(record.clone())
        }
    );
    assert_eq!(reopened[3].2, Some(0));

    // The page rules, on the server with their client keys.
    let base = session.draft.clone();
    let reject =
        |(documents, answers): (Vec<IncomeDocumentItem>, Option<Vec<crate::DocumentAnswer>>),
         key: &str| {
            let mut session = Interview::new(TEST_USER, base.clone());
            if matches!(key, "Q1099_PART_ABOVE_GROSS" | "Q1099_ROTH_ROLLOVER_NOT_STUDENT") {
                let request = save_documents(&mut session, documents.clone());
                accepted(&send(&mut session, request));
                let before = session.draft.clone();
                assert_eq!(fixture_paper_answers(&documents[0].document), answers.unwrap());
                let request =
                    crate::tests::support::client::income_pages::information_return_answers(
                        &mut session,
                        &documents,
                    );
                rejected_with(send(&mut session, request), key);
                assert_eq!(session.draft, before);
                return;
            }
            let before = session.draft.clone();
            rejected_with(
                {
                    let request = if documents
                        .iter()
                        .all(|row| matches!(row.document, IncomeSource::Record { .. }))
                    {
                        save_topic(&mut session, TaxSection::OtherIncome, documents)
                    } else {
                        assert!(
                            documents
                                .iter()
                                .all(|row| !matches!(row.document, IncomeSource::Record { .. }))
                        );
                        save_documents(&mut session, documents)
                    };
                    send(&mut session, request)
                },
                key,
            );
            assert_eq!(session.draft, before);
        };
    let form = |q: EducationDistributionSource| {
        let paper = IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::Q1099(education_distribution_document(q)),
        };
        let answers = fixture_paper_answers(&paper);
        (vec![owned(crate::DocumentOwner::Yours, 0, paper)], Some(answers))
    };
    let qtp = |earnings, basis| EducationAccount::Qtp { earnings, basis };
    reject(
        form(q1099_page(
            1_000_000,
            qtp(100_000, 800_000),
            plain_withdrawal(),
            Q1099Beneficiary::Recipient,
        )),
        "Q1099_BOXES_DO_NOT_ADD_UP",
    );
    reject(
        form(q1099_page(
            1_000_000,
            EducationAccount::Coverdell {
                amounts: CoverdellAmounts::Reported { earnings: 100_000, basis: 800_000 },
            },
            plain_withdrawal(),
            Q1099Beneficiary::Recipient,
        )),
        "Q1099_BOXES_DO_NOT_ADD_UP",
    );
    reject(
        form({
            let mut value = q1099_page(
                1_000_000,
                qtp(100_000, 900_000),
                Q1099Transfer::Withdrawal { reason: DistributionReason::Withdrawal },
                Q1099Beneficiary::Recipient,
            );
            value.redeposited = 1_000_001;
            value
        }),
        "Q1099_PART_ABOVE_GROSS",
    );
    reject(
        form({
            let mut value = q1099_page(
                600_000,
                EducationAccount::Coverdell {
                    amounts: CoverdellAmounts::Reported { earnings: 200_000, basis: 400_000 },
                },
                Q1099Transfer::RothIra,
                Q1099Beneficiary::Recipient,
            );
            value.roth_rollover = Some(qtp_roth_rollover().rollover);
            value
        }),
        "Q1099_ROTH_ROLLOVER_NOT_529",
    );
    reject(
        form({
            let mut value = q1099_page(
                600_000,
                qtp(200_000, 400_000),
                Q1099Transfer::RothIra,
                Q1099Beneficiary::Student { person: child },
            );
            value.roth_rollover = Some(qtp_roth_rollover().rollover);
            value
        }),
        "Q1099_ROTH_ROLLOVER_NOT_STUDENT",
    );
    reject(
        form({
            let mut value = q1099_page(
                600_000,
                qtp(-200_000, 800_000),
                Q1099Transfer::RothIra,
                Q1099Beneficiary::Recipient,
            );
            value.roth_rollover = Some(qtp_roth_rollover().rollover);
            value
        }),
        "Q1099_ROTH_ROLLOVER_LOSS",
    );
    reject(
        form(q1099_page(
            600_000,
            qtp(200_000, 400_000),
            Q1099Transfer::ExcessThisYear,
            Q1099Beneficiary::Recipient,
        )),
        "Q1099_CODE_NOT_COVERDELL",
    );
    reject(
        form({
            let mut value = q1099_page(
                600_000,
                EducationAccount::Coverdell { amounts: CoverdellAmounts::ValueOnly { value: 1 } },
                Q1099Transfer::ExcessLastYear,
                Q1099Beneficiary::Recipient,
            );
            value.coverdell_basis_start = Some(0);
            value.late_rollovers = Some(0);
            value
        }),
        "Q1099_EXCESS_NEEDS_BOXES",
    );
    reject(
        (
            vec![
                owned(
                    crate::DocumentOwner::Yours,
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::EducationAccountExpenses(record.clone()),
                    },
                ),
                owned(
                    crate::DocumentOwner::Yours,
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::EducationAccountExpenses(record.clone()),
                    },
                ),
            ],
            None,
        ),
        "EDUCATION_ACCOUNT_EXPENSES_TWICE",
    );
    assert_eq!(
        crate::tests::support::client::income_pages::saved_document_amount(
            &mut session,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Q1099(education_distribution_document(
                    q1099_page(
                        1_000_000,
                        qtp(100_000, 900_000),
                        plain_withdrawal(),
                        Q1099Beneficiary::Recipient
                    )
                ))
            }
        ),
        None
    );
}

// Topic-owned expense rows follow the people on the return. A source 1099-Q
// retains its own beneficiary answer and amounts for explicit correction.
#[test]
fn education_account_rows_leave_with_their_person() {
    use crate::tests::family::education_accounts::{
        entered, expense_record, q1099_form, qtp, student_row, withdrawal,
    };
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    let child = d.dependents[0].id;
    add(
        &mut d,
        q1099_form(
            Owner::Taxpayer,
            5_000,
            qtp(1_000, 4_000),
            withdrawal(),
            FederalEducationBeneficiary::Student { person: child },
        ),
    );
    add(
        &mut d,
        expense_record(vec![
            student_row(child, entered(5_000, 0, 0)),
            student_row(-1, entered(1_000, 0, 0)),
        ]),
    );
    let mut kept = d.clone();
    let first = crate::interview::prune(&mut kept);
    assert!(first.removed.is_empty(), "{:?}", first.removed);
    d.dependents.clear();
    let pruned = crate::interview::prune(&mut d);
    assert!(!pruned.removed.is_empty());
    let FederalIncome::EducationDistribution { data } = &d.income[1].income else { unreachable!() };
    assert_eq!(data.beneficiary, Some(FederalEducationBeneficiary::Student { person: child }));
    assert_eq!(data.form.gross_distribution, 500_000);
    let FederalIncome::EducationAccountExpenses { data } = &d.income[2].income else {
        unreachable!()
    };
    assert_eq!(data.students.len(), 1);
    assert_eq!(data.students[0].person, -1);
    // A record whose only student is gone is no record.
    let FederalIncome::EducationAccountExpenses { data } = &mut d.income[2].income else {
        unreachable!()
    };
    data.students[0].person = -2;
    crate::interview::prune(&mut d);
    assert!(
        d.income
            .iter()
            .all(|e| !matches!(e.income, FederalIncome::EducationAccountExpenses { .. }))
    );
}

// W-2 references by identity (adoption employer payments, dependent-care
// W-2s): the More credits request carries the keyed catalog of the return's
// W-2s; a payment naming anything else is refused on save.
#[test]
fn adoption_employer_payments_name_their_w2_by_identity() {
    let d = joint_employer_adoption_return();
    let mut s = restart(d.clone());
    let page = open_credits_topic(&mut s, TaxSection::MoreCredits);
    let value = more_credits_value(&page).clone();
    let MoreCredit::Adoption { adoptions, .. } = &value.credits[0] else {
        panic!("expected the adoption item")
    };
    let wage_ids: Vec<i64> = adoptions[0].wages.iter().map(|w| w.id).collect();
    assert_eq!(wage_ids, vec![1, 2, 3]);
    // The page child is the stored child with each payment choosing its W-2.
    assert_eq!(adoptions[0].record(), d.credits.as_ref().unwrap().adoptions[0]);
    assert_eq!(adoptions[0].record().page(&adoptions[0].wages), adoptions[0]);
    let AdoptionBenefitsPage::JointEmployees { payments, .. } = &adoptions[0].benefits else {
        panic!("expected joint employees")
    };
    assert_eq!(payments.iter().map(|p| p.wage).collect::<Vec<_>>(), vec![1, 2, 3]);
    // Unchanged, the page saves.
    let request = save_more_credits(&s.draft.clone(), value.clone());
    accepted(&send(&mut s, request));
    // A W-2 that is not on the return, or is the other spouse's, is unanswered.
    for (index, wage) in [(1, 9999), (0, 3)] {
        let mut changed = value.clone();
        let MoreCredit::Adoption { adoptions, .. } = &mut changed.credits[0] else {
            unreachable!()
        };
        let AdoptionBenefitsPage::JointEmployees { payments, .. } = &mut adoptions[0].benefits
        else {
            unreachable!()
        };
        payments[index].wage = wage;
        let mut again = restart(d.clone());
        let request = save_more_credits(&again.draft.clone(), changed);
        rejected_with(send(&mut again, request), "ADOPTION_EMPLOYER_NOT_THEIR_W2");
    }
}

#[test]
fn head_of_household_married_to_a_nonresident_alien_counts_as_unmarried() {
    // Section 2(b)(2)(B): a spouse who was a nonresident alien leaves the filer
    // unmarried for head of household, with a qualifying child.
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    let mut s = Interview::new(TEST_USER, d);
    saved_setup(
        &mut s,
        ReturnSetup {
            filing: Filing::HeadOfHousehold {
                paid_over_half_home: true,
                marital: HeadMarital::NonresidentSpouse,
            },
        },
    );
    let setup = s.draft.setup.clone().unwrap();
    assert_eq!(setup.marital_state, MaritalState::MarriedNonresidentSpouse);
    assert_eq!(setup.filing_status(), FederalStatus::Head);
    let r = supported(&s.draft);
    assert_eq!(value(&r, "deduction"), 24_150);
    assert_eq!(
        crate::interview::return_setup(&setup).filing,
        Filing::HeadOfHousehold {
            paid_over_half_home: true,
            marital: HeadMarital::NonresidentSpouse
        }
    );
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
    // Without a qualifying person the status still needs one.
    let mut none = s.draft.clone();
    none.dependents.clear();
    assert_eq!(estimate(&none).state, ResultState::Unsupported);
    // Single afterwards: the marital state resets.
    saved_setup(&mut s, ReturnSetup { filing: Filing::Single });
    assert_eq!(s.draft.setup.as_ref().unwrap().marital_state, MaritalState::Unmarried);
}

#[test]
fn the_children_page_asks_days_on_the_final_return_of_a_person_who_died() {
    // Gray (head of household) died 2026-04-30: 31 + 28 + 31 + 30 = 120 days.
    // The page counts days through the death (61 x 2 = 122 > 120 passes, 60
    // does not); any other return keeps asking months.
    let mut d = complete(FederalStatus::Head);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, estate_medical: 0, date: 20260430, ordinary: true });
    d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 50_000);
    let mut s = Interview::new(TEST_USER, d);
    let with_days = |days: u32| {
        let mut pat = row("Pat", "Sample", 20180214);
        pat.lived = LivedWithYou::Days { days };
        pat
    };
    // 61 days is more than half of 120: Pat qualifies, the page reopens with
    // the days, and the stored record is the existing days residence.
    let request = save_children(&s.draft.clone(), vec![with_days(61)]);
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.dependents[0].residency, DependentResidency::Days { days: 61 });
    assert!(s.draft.dependents[0].us_home_over_half_year);
    let request = navigate(&s.draft, TaxSection::Children);
    let page = match send(&mut s, request) {
        Reply::Edit(page @ P::SaveChildren { .. }) => page,
        other => panic!("expected the people page, got {other:?}"),
    };
    let P::SaveChildren { allowed_lived, days_alive, .. } = &page else { unreachable!() };
    assert_eq!(days_alive, &120);
    assert_eq!(
        allowed_lived,
        &vec![
            libertas_macros::variant_index!(LivedWithYou::AllYear) as i32,
            libertas_macros::variant_index!(LivedWithYou::Days) as i32,
        ]
    );
    assert_eq!(reopened_people(&mut s)[0].lived, LivedWithYou::Days { days: 61 });
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "28"), 170_000);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
    // 60 days is not: head of household then has no person who qualifies.
    let request = save_children(&s.draft.clone(), vec![with_days(60)]);
    accepted(&send(&mut s, request));
    assert!(!s.draft.dependents[0].us_home_over_half_year);
    assert_eq!(estimate(&s.draft).state, ResultState::Unsupported);
    // More days than the days alive, and months on this return, are refused.
    let request = save_children(&s.draft.clone(), vec![with_days(121)]);
    rejected_with(send(&mut s, request), "RESIDENCE_DAYS_ABOVE_DAYS_ALIVE");
    let mut months = row("Pat", "Sample", 20180214);
    months.lived = LivedWithYou::PartOfYear { months: 3 };
    let request = save_children(&s.draft.clone(), vec![months]);
    rejected_with(send(&mut s, request), "RESIDENCE_ANSWER_NOT_ALLOWED");
    // A return where nobody died asks months, never days.
    let mut alive = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_children(&alive.draft.clone(), vec![row("Ava", "Sample", 20180214)]);
    accepted(&send(&mut alive, request));
    let request = navigate(&alive.draft, TaxSection::Children);
    let page = match send(&mut alive, request) {
        Reply::Edit(page @ P::SaveChildren { .. }) => page,
        other => panic!("expected the people page, got {other:?}"),
    };
    let P::SaveChildren { allowed_lived, days_alive, .. } = &page else { unreachable!() };
    assert_eq!(days_alive, &365);
    assert_eq!(
        allowed_lived,
        &vec![
            libertas_macros::variant_index!(LivedWithYou::AllYear) as i32,
            libertas_macros::variant_index!(LivedWithYou::PartOfYear) as i32,
        ]
    );
    let request = save_children(&alive.draft.clone(), vec![with_days(61)]);
    rejected_with(send(&mut alive, request), "RESIDENCE_ANSWER_NOT_ALLOWED");
    // A joint return counts the whole year: months, even after a death.
    let mut joint = complete(FederalStatus::Joint);
    joint.people.as_mut().unwrap().spouse.as_mut().unwrap().died =
        Some(FinalReturn { bonds: None, estate_medical: 0, date: 20260430, ordinary: true });
    let mut s = Interview::new(TEST_USER, joint);
    let request = save_children(&s.draft.clone(), vec![with_days(61)]);
    rejected_with(send(&mut s, request), "RESIDENCE_ANSWER_NOT_ALLOWED");
}

// Choosing head of household on the first page starts its branch unanswered:
// the marital situation is an eligibility fact the rules test, so it has no
// Default and is never pre-answered (the home-cost answer has none either).
#[test]
fn head_of_household_marital_situation_starts_unanswered() {
    let source = include_str!("../../family/filers_model.rs");
    let branch = &source[source.find("    HeadOfHousehold {").unwrap()..];
    let branch = &branch[..branch.find("    },").unwrap()];
    assert!(branch.contains("marital: HeadMarital,"));
    assert!(branch.contains("paid_over_half_home: bool,"));
    assert!(!branch.contains("libertas_default"));
}

// A field's declaration with its doc and attribute lines, from the struct
// that holds it: the lines above `pub <field>:` up to the previous field.
fn declared<'a>(source: &'a str, owner: &str, field: &str) -> &'a str {
    let body = &source[source.find(&alloc::format!("pub struct {owner} {{")).unwrap()..];
    let body = &body[..body.find("\n}\n").unwrap()];
    let end = body.find(&alloc::format!("pub {field}:")).unwrap();
    let start = body[..end].rfind(",\n").map_or(0, |i| i + 2);
    &body[start..end]
}

// Blindness is a fact the standard deduction tests (section 63(f)), so it has
// no Default (app.default.decided) on About you or the spouse page: each
// unsaved page opens unanswered by WorkflowRequest, and only the person's own
// answer is saved and reopened by EditRequest.
#[test]
fn blindness_starts_unanswered_on_about_you_and_the_spouse_page() {
    let source = include_str!("../../family/filers_model.rs");
    for owner in ["FilerDetails", "SpouseDetails"] {
        let field = declared(source, owner, "blind");
        assert!(field.contains("Legally blind"), "{owner}: {field}");
        assert!(!field.contains("libertas_default"), "{owner}: {field}");
    }
    let mut s = Interview::new(TEST_USER, FederalDraft::empty("blind".into()));
    let request = save_setup(&s.draft, ReturnSetup { filing: Filing::MarriedJointly });
    assert!(matches!(send(&mut s, request), Reply::Response(P::BeginPeople { .. })));
    let mut you = adult("Robin", "Sample", 19850301);
    you.details.blind = true;
    let request = save_you(&s.draft, you);
    assert!(matches!(send(&mut s, request), Reply::Response(P::BeginSpouse { .. })));
    let mut spouse = adult("Jordan", "Sample", 19860101);
    spouse.details.blind = true;
    let request = crate::tests::cross::interview::save_spouse(&s.draft, spouse);
    accepted(&send(&mut s, request));
    let people = s.draft.people.as_ref().unwrap();
    assert!(people.taxpayer.blind && people.spouse.as_ref().unwrap().blind);
    let Reply::Edit(P::SavePeople { you, .. }) = navigate_to(&mut s, TaxSection::People) else {
        panic!("About you reopens with its saved answer")
    };
    assert!(you.details.blind);
    let Reply::Edit(P::SaveSpouse { spouse, .. }) = navigate_to(&mut s, TaxSection::Spouse) else {
        panic!("the spouse page reopens with its saved answer")
    };
    assert!(spouse.details.blind);
}
fn navigate_to(s: &mut Interview, section: TaxSection) -> Reply {
    let request = navigate(&s.draft, section);
    send(s, request)
}

// A person added on the Children page initializes fresh: relationship, time
// lived with you, situation and a competing claim are facts the dependency
// tests read (section 152), so none has a Default and each starts unanswered.
// The age rule then waits for a situation the person chose: a 2007 birth
// date is no refusal as a student and is refused only as a child under 19.
#[test]
fn a_new_person_starts_unanswered_and_the_age_rule_waits_for_a_chosen_situation() {
    let source = include_str!("../../family/dependents_model.rs");
    for field in ["relationship", "lived", "situation", "competing"] {
        let declaration = declared(source, "Dependent", field);
        assert!(!declaration.contains("libertas_default"), "{field}: {declaration}");
    }
    let mut s = supporting(FederalStatus::Single);
    assert!(matches!(
        navigate_to(&mut s, TaxSection::Children),
        Reply::Response(P::BeginChildren { .. })
    ));
    let loaded = s.draft.clone();
    let estimate_for = |s: &mut Interview, person: Dependent| {
        send(
            s,
            P::EstimateDependents {
                user: TEST_USER,
                dependents: Dependents { people: vec![person] },
                allowed_lived: Vec::new(),
                days_alive: 365,
            },
        )
    };
    let alice = row("Alice", "Sample", 20070612);
    let student = Dependent { situation: DependentSituation::Student, ..alice.clone() };
    let reply = estimate_for(&mut s, student.clone());
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let reply = estimate_for(&mut s, alice);
    let Reply::Response(P::Problem { message, .. }) = reply else {
        panic!("a chosen child under 19 born in 2007 is refused, got {reply:?}")
    };
    assert_eq!(message, libertas::libertas_formatted_text("DEPENDENT_AGE_SITUATION", &[]));
    assert_eq!(s.draft, loaded, "estimates accept nothing");
    accepted(&send(&mut s, save_children(&loaded, vec![student])));
    assert_eq!(s.draft.dependents()[0].situation, DependentSituation::Student);
}

// Every other fact a rule tests on About you, the spouse page and a person
// on the Children page has no Default either: whether someone can claim the
// filer, a death in 2026, the identification number, what a person's facts
// are used for and the person's rare situations ("None of these" is chosen,
// never assumed).
#[test]
fn rule_tested_people_facts_have_no_default() {
    let filers = include_str!("../../family/filers_model.rs");
    for (owner, field) in [
        ("Filer", "identification"),
        ("SpouseAnswers", "identification"),
        ("FilerDetails", "claimable"),
        ("FilerDetails", "more"),
        ("SpouseDetails", "claimable"),
        ("SpouseDetails", "more"),
    ] {
        let declaration = declared(filers, owner, field);
        assert!(!declaration.contains("libertas_default"), "{owner}.{field}: {declaration}");
    }
    let people = include_str!("../../family/dependents_model.rs");
    for field in ["use_for", "identification", "facts"] {
        let declaration = declared(people, "Dependent", field);
        assert!(!declaration.contains("libertas_default"), "{field}: {declaration}");
    }
    for owner in ["Filer", "SpouseAnswers"] {
        assert!(!declared(filers, owner, "situations").contains("libertas_default"), "{owner}");
    }
    // A Bitflags question is never Nullable: its zero answer, "None of
    // these", is chosen like any other and saved.
    assert!(!declared(people, "Dependent", "facts").contains("Option<"));
    let mut s = supporting(FederalStatus::Single);
    let loaded = s.draft.clone();
    let none = Dependent { facts: PersonFacts::NONE.0, ..row("Ava", "Sample", 20180615) };
    accepted(&send(&mut s, save_children(&loaded, vec![none])));
    assert_eq!(s.draft.dependents()[0].facts, 0);
}

// Care for each person is a fact Form 2441 tests: a person added on the
// Children page has no care answer stored until Child and dependent care
// answers it. While one is unanswered the page opens by WorkflowRequest
// (BeginCare) with every row's care unanswered; once all are answered it
// reopens by EditRequest with the answers.
#[test]
fn care_for_a_new_person_starts_unanswered() {
    let model = include_str!("../../family/form_2441_model.rs");
    assert!(!declared(model, "CareQualifyingRow", "care").contains("libertas_default"));
    let mut s = supporting(FederalStatus::Single);
    let loaded = s.draft.clone();
    accepted(&send(&mut s, save_children(&loaded, vec![row("Ava", "Sample", 20180615)])));
    assert_eq!(s.draft.dependents()[0].care, None, "nothing is pre-answered");
    let Reply::Response(P::BeginCare { value, allowed_filing, .. }) =
        from_review(&mut s, TaxSection::Care)
    else {
        panic!("the unanswered care page opens by WorkflowRequest")
    };
    assert!(value.qualifying_people.is_empty());
    assert_eq!(value.new_people.len(), 1);
    // The filing status is the return's (single): one allowed choice.
    assert_eq!(allowed_filing.len(), 1);
    save_care_costs(&mut s, &[CareCosts::Yes { expenses: 300_000, incapable_self_care: false }]);
    assert_eq!(s.draft.dependents()[0].care, Some(DependentCare::Yes { care_expenses: 300_000 }));
    let Reply::Edit(P::SaveCare { value, .. }) = from_review(&mut s, TaxSection::Care) else {
        panic!("the answered care page reopens by EditRequest")
    };
    assert_eq!(
        value.qualifying_people[0].care,
        CareCosts::Yes { expenses: 300_000, incapable_self_care: false }
    );
    // A second person added later: the page asks again, and the first
    // person's stored answer stays until the page is saved.
    let d = s.draft.clone();
    let mut people = reopened_people(&mut s);
    people.push(row("Ben", "Sample", 20200301));
    accepted(&send(&mut s, save_children(&d, people)));
    assert_eq!(s.draft.dependents()[1].care, None);
    assert_eq!(s.draft.dependents()[0].care, Some(DependentCare::Yes { care_expenses: 300_000 }));
    // The answered person arrives with the saved care; only the new one is
    // unanswered, and saving keeps the first person's care.
    let Reply::Response(P::BeginCare { value, .. }) = from_review(&mut s, TaxSection::Care) else {
        panic!("a new person's care is asked by WorkflowRequest")
    };
    assert_eq!(value.qualifying_people.len(), 1);
    assert_eq!(
        value.qualifying_people[0].care,
        CareCosts::Yes { expenses: 300_000, incapable_self_care: false }
    );
    assert_eq!(value.new_people.len(), 1);
    let mut page =
        crate::tests::cross::interview::care_editor(from_review(&mut s, TaxSection::Care));
    let P::SaveCare { value, .. } = &mut page else { unreachable!() };
    value.new_people[0].care = CareCosts::No;
    accepted(&send(&mut s, page));
    assert_eq!(s.draft.dependents()[0].care, Some(DependentCare::Yes { care_expenses: 300_000 }));
    assert_eq!(s.draft.dependents()[1].care, Some(DependentCare::No));
    assert!(matches!(from_review(&mut s, TaxSection::Care), Reply::Edit(P::SaveCare { .. })));
}
