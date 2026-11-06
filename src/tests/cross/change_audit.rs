// Accepted changes prune answers their walk no longer owns, reopen newly
// required pages, and retain document-center source facts.
use super::*;
use crate::interview::{filer, return_setup};
use crate::tests::cross::interview::{
    PAGE_PEOPLE, PAGE_REVIEW, PAGE_SETUP, adult, back, dependent_record, from_review, go_to,
    opened, reopened_people, reviewing, row, save_checklist, save_children, save_setup,
    save_spouse, save_you,
};
use alloc::string::String;

// Establish the accepted source inventory and its ordinary visit before
// changing an already answered return. Native record facts are admitted first.
fn source_baseline(d: FederalDraft) -> FederalDraft {
    let mut session = Interview::new(TEST_USER, d);
    crate::tests::support::client::admit_native_fixture_facts(&mut session);
    let request = crate::tests::support::client::income_pages::open_document_center(&mut session);
    accepted(&send(&mut session, request));
    session.draft
}

fn setup_value(filing: Filing) -> ReturnSetup {
    ReturnSetup { filing }
}
fn issues(d: &FederalDraft) -> Vec<String> {
    estimate(d).issues.iter().map(|i| alloc::format!("{}: {}", i.section, i.message)).collect()
}
fn has_issue(d: &FederalDraft, needle: &str) -> bool {
    issues(d).iter().any(|i| i.contains(needle))
}
// The review's remaining items, "item: status".
fn remaining(s: &mut Interview) -> Vec<String> {
    match send(s, P::ReviewAccepted { user: TEST_USER }) {
        Reply::Response(P::Review { remaining_items, .. }) => remaining_items
            .iter()
            .map(|r| alloc::format!("{}: {}", status_text(&r.item), status_text(&r.status)))
            .collect(),
        other => panic!("expected the review, got {other:?}"),
    }
}
fn accepted(reply: &Reply) {
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
}
// The page a reply opens, by its request's name ("SavePayments"), a hub by
// its page and a topic page by its topic.
fn name_of(reply: &Reply) -> String {
    let (Reply::Response(p) | Reply::Edit(p)) = reply;
    match p {
        P::Hub { page, .. } => alloc::format!("Hub {page}"),
        P::SaveIncomes { topic, .. } => alloc::format!("SaveIncomes {topic:?}"),
        _ => alloc::format!("{p:?}").chars().take_while(|c| c.is_alphanumeric()).collect(),
    }
}
// Presses Next on each page the walk opens, with its answers as it opened
// them (a hub goes on to its suggestion, an empty situations page is saved
// empty), until the review: the pages in the order they opened.
fn walk_to_review(s: &mut Interview, mut reply: Reply) -> Vec<String> {
    let mut walked = Vec::new();
    for _ in 0..64 {
        // Complete the method selector as a client, preserving supplied sources.
        reply = crate::tests::support::client::answer_no_estimates(reply);
        if matches!(reply, Reply::Response(P::Review { .. })) {
            return walked;
        }
        walked.push(name_of(&reply));
        let request = match reply {
            Reply::Edit(request) => request,
            Reply::Response(hub @ P::Hub { .. }) => crate::tests::cross::interview::go_on(&hub),
            // Every return asks the people page once (gate fixes batch A).
            Reply::Response(P::BeginChildren { .. }) => save_children(&s.draft, vec![]),
            // The unsaved income without a form page, answered No twice.
            begin @ Reply::Response(P::BeginIncomeWithoutForm { .. }) => {
                crate::tests::cross::interview::without_form_editor(begin)
            }
            other => panic!("the walk stopped at {other:?}"),
        };
        reply = send(s, request);
        accepted(&reply);
    }
    panic!("the walk did not reach the review: {walked:?}")
}
fn spouse_ira(traditional: i64) -> IraSpouse {
    IraSpouse::Joint {
        facts: FederalIraContribution {
            nondeductible_election: 0,
            traditional,
            roth: 0,
            covered_at_work: false,
            returned_next_year: vec![],
            excess: None,
            next_year_deposits: None,
            qtp_roth_lifetime_prior: 0,
        },
    }
}
// A complete joint return with each spouse answer later pages hold.
fn joint_with_spouse_answers() -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Spouse, 3_000_000, 300_000));
    d
}
fn separate_setup() -> ReturnSetup {
    setup_value(Filing::MarriedSeparately)
}
fn head_setup() -> ReturnSetup {
    setup_value(Filing::HeadOfHousehold {
        paid_over_half_home: true,
        marital: HeadMarital::Unmarried,
    })
}
// A complete head-of-household return with one child.
fn head_with_child() -> FederalDraft {
    let mut d = complete(FederalStatus::Head);
    let id = d.next_id;
    d.dependents = vec![dependent_record(id, "Ava", "Sample", 20180214)];
    d.dependents[0].situation = DependentSituation::Supported {
        your_support_over_half: true,
        gross_income: d.dependents[0].gross_income(),
    };
    d.next_id += 1;
    d
}

// ---------------------------------------------------------------------------
// Concern 1: a conflicting change removes what it must, named first.

// A joint return with an answer on every page that holds one for the
// spouse: W-2 and 1099-INT, HSA with the family split, IRA, Saver's
// Credit, educator expenses, Form 2210 box E and Form 8379.
fn joint_with_every_spouse_answer() -> FederalDraft {
    let mut d = joint_with_spouse_answers();
    let id = d.next_id;
    add(
        &mut d,
        FederalIncomeEntry {
            printed: None,
            refund_management: None,
            state_source: Default::default(),
            id,
            label: "Spouse Bank".into(),
            owner: crate::DocumentOwner::Spouses,
            income: savings_income(400, 0),
            community: CommunityShare::NotNeeded,
        },
    );
    let a = d.adjustments.as_mut().unwrap();
    a.hsa = vec![
        hsa_entry(Owner::Taxpayer, HsaCoverage::Family, 300_000),
        hsa_entry(Owner::Spouse, HsaCoverage::Family, 200_000),
    ];
    a.hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::Monthly,
        taxpayer_parts: 1,
        spouse_parts: 1,
        other_coverage: None,
    });
    let mut iras = ira_facts(300_000, 0, Answer::No);
    iras.spouse = spouse_ira(200_000);
    a.iras = iras;
    a.educators = EducatorClaim {
        taxpayer: Some(EducatorRole::Educator { classroom: 10_000, instructional: 0 }),
        spouse: Some(EducatorRole::Educator { classroom: 20_000, instructional: 0 }),
    };
    d.credits.as_mut().unwrap().saver = Some(saver(0, 100_000)).into();
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year = Some(PriorYearTax {
        full_year: true,
        tax: 500_000,
        agi: Some(6_000_000),
        spouse_tax: 300_000,
    });
    d.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    d.injured_spouse = Some(InjuredSpouseAllocation {
        injured: Owner::Taxpayer,
        debt: OffsetDebt::ChildSupport,
        obligated: false,
        community: InjuredCommunityFacts::No,
        student_loan_borrower: None,
        estimated: EstimatedSplit::IrsFormula,
        allocations: vec![],
        claimants: vec![],
    });
    crate::interview::with_ticks(d)
}
// The warning items returned beside page 1's running estimate.
fn setup_warnings(s: &mut Interview, value: ReturnSetup) -> Vec<Message> {
    match send(s, P::EstimateSetup { user: TEST_USER, value }) {
        Reply::Response(P::RunningEstimate { warnings, .. }) => {
            warnings.iter().map(|warning| super::status_message(&warning.message)).collect()
        }
        other => panic!("expected a running estimate, got {other:?}"),
    }
}
fn assert_warning(warnings: &[Message], key: &str, args: Vec<MessageArg>) {
    let expected = Message::new(key, args);
    assert!(warnings.contains(&expected), "missing {expected:?}: {warnings:?}");
}

#[test]
fn joint_to_single_erases_every_spouse_answer_named_first() {
    // Joint -> Single on the first pass (review=false): the running estimate
    // names the record and fact answers the save removes; the save is
    // accepted and your page opens next. Source documents remain unchanged.
    let d = joint_with_every_spouse_answer();
    let mut s = Interview::new(TEST_USER, d.clone());
    let warnings = setup_warnings(&mut s, setup_value(Filing::Single));
    for key in [
        "SAVING_REMOVES_YOUR_SPOUSE",
        "SAVING_REMOVES_THE_HSA_FAMILY_SPLIT",
        "SAVING_REMOVES_YOUR_SPOUSE_S_IRA_FACTS",
        "SAVING_REMOVES_YOUR_SPOUSE_S_EDUCATOR_EXPENSES",
        "SAVING_REMOVES_YOUR_SPOUSE_S_SAVER_S_CREDIT_CONTRIBUTIONS",
        "SAVING_REMOVES_YOUR_FEDERAL_PAYMENTS",
        "SAVING_REMOVES_YOUR_INJURED_SPOUSE_ALLOCATION_FORM_8379",
    ] {
        assert_warning(&warnings, key, vec![]);
    }
    assert_warning(
        &warnings,
        "SAVING_REMOVES_ONE_NAMED",
        vec![
            MessageArg::Count(1),
            MessageArg::Resource("SAVING_KIND_HSA_ONE".into()),
            MessageArg::Resource("SAVING_OWNER_SPOUSE".into()),
        ],
    );
    for (kind, name) in
        [("SAVING_KIND_W2_ONE", "Employer 2"), ("SAVING_KIND_1099_INT_ONE", "Spouse Bank")]
    {
        assert!(!warnings.contains(&Message::new(
            "SAVING_REMOVES_ONE_NAMED",
            vec![
                MessageArg::Count(1),
                MessageArg::Resource(kind.into()),
                MessageArg::Text(name.into())
            ],
        )));
    }
    let reply = send(&mut s, save_setup(&d, setup_value(Filing::Single)));
    assert_eq!(opened(&reply).0, "you");
    let kept = &s.draft;
    assert!(kept.people.as_ref().unwrap().spouse.is_none());
    assert_eq!(kept.income, d.income);
    let a = kept.adjustments.as_ref().unwrap();
    assert!(a.hsa.iter().all(|h| h.owner == Owner::Taxpayer) && a.hsa.len() == 1);
    assert_eq!(
        a.hsa_family_share,
        Some(HsaFamilyShare {
            method: HsaAllocationMethod::Monthly,
            taxpayer_parts: 1,
            spouse_parts: 1,
            other_coverage: None,
        })
    );
    assert_eq!(a.iras.spouse, IraSpouse::NotApplicable);
    assert_eq!(a.educator_spouse(), 0);
    let saver = kept.credits.as_ref().unwrap().saver.as_ref().unwrap();
    assert_eq!(saver.spouse_contributions, 0);
    let pay = kept.payments.as_ref().unwrap();
    assert!(pay.prior_year.is_none() && pay.nothing_entered());
    assert!(kept.often_missed.unprepared.is_empty() && kept.injured_spouse.is_none());
    // Pruning is stable, including retained source documents.
    let mut again = kept.clone();
    assert_eq!(crate::interview::prune(&mut again), Default::default());
    assert_eq!(again, *kept);
    assert!(has_issue(kept, "Spouse-owned income requires a joint return"), "{:?}", issues(kept));
}

#[test]
fn undoing_a_preview_change_clears_save_warnings_without_accepting_it() {
    let d = joint_with_spouse_answers();
    let mut s = Interview::new(TEST_USER, d.clone());
    assert!(!setup_warnings(&mut s, setup_value(Filing::Single)).is_empty());
    assert!(setup_warnings(&mut s, setup_value(Filing::MarriedJointly)).is_empty());
    assert_eq!(s.draft, d);
}

#[test]
fn removing_every_person_from_a_head_of_household_return_is_an_end_of_walk_item() {
    // SaveChildren with no rows on a head-of-household return is accepted:
    // page 1's answer is valid only given a later page's (a backward
    // dependency), so it is not pruned. The whole-return check names the
    // people page that fixes it, as an estimate issue and a review item.
    let d = head_with_child();
    assert_eq!(estimate(&d).state, ResultState::EstimatedForSupportedScope, "{:?}", issues(&d));
    let mut s = Interview::new(TEST_USER, d.clone());
    let reply = send(&mut s, reviewing(save_children(&d, vec![])));
    accepted(&reply);
    assert!(s.draft.dependents().is_empty());
    assert_eq!(s.draft.setup.as_ref().unwrap().status, FilingChoice::Head);
    assert!(has_issue(&s.draft, "Filing status"), "{:?}", issues(&s.draft));
    assert_ne!(estimate(&s.draft).state, ResultState::EstimatedForSupportedScope);
    assert!(
        has_issue(&s.draft, "Children and people you support: Your filing status needs a person"),
        "{:?}",
        issues(&s.draft)
    );
    let items = remaining(&mut s);
    assert!(items.iter().any(|i| i.starts_with("Children and people you support")), "{items:?}");
}

// ---------------------------------------------------------------------------
// Concern 2: later answers that depend on the earlier one.

// Household removal retains the physical education source for correction;
// deleting it explicitly must leave independent care facts unchanged.
#[test]
fn deleting_a_person_erases_their_education_rows_and_retains_independent_care_facts() {
    let mut d = head_with_child();
    let child = d.dependents[0].id;
    let c = d.credits.as_mut().unwrap();
    let student = child;
    set_education(
        c,
        vec![{
            let mut pair = native_education(
                student,
                education_expenses(300_000),
                EducationChoice::LifetimeLearning,
            );

            pair.0.id = 0;
            pair.0.half_time = true;
            pair.0.graduate = true;
            pair.1.more.american_opportunity_used_up = Some(false);
            pair
        }],
    );
    c.care = FederalCare {
        prior_year: Some(prior_care_facts()),
        unpaid_return_year_care: Some(25_000),
        ..FederalCare::default()
    };
    d.dependents[0].care = Some(DependentCare::Yes { care_expenses: 300_000 });
    document_care(&mut d);
    let care = d.credits.as_ref().unwrap().care.clone();
    let mut s = Interview::new(TEST_USER, d.clone());
    let mut shown = reopened_people(&mut s);
    assert_eq!(shown.len(), 1);
    shown[0] = row("Ben", "Sample", 20190101);
    let before = s.draft.clone();
    let reply = send(&mut s, save_children(&before, shown));
    accepted(&reply);
    assert!(s.draft.dependents().iter().all(|person| person.id != child));
    assert!(s.draft.valid_stored());
    let incomplete = estimate(&s.draft);
    assert_eq!(incomplete.state, ResultState::Incomplete);
    assert_eq!(incomplete.tax, None);
    assert!(remaining(&mut s).iter().any(|item| item.starts_with("Document center")));
    let c = s.draft.credits.as_ref().unwrap();
    assert_eq!(c.education, d.credits.as_ref().unwrap().education);
    assert_eq!(c.students, d.credits.as_ref().unwrap().students);
    assert_eq!(c.care, care);
    let request = crate::tests::support::client::income_pages::open_document_center(&mut s);
    // Unresolved source ownership is accepted and reviewed in the Document center.
    let reply = send(&mut s, request);
    let Reply::Response(P::Review { remaining_items, .. }) = reply else {
        panic!("expected the pending-owner review, got {reply:?}")
    };
    assert!(remaining_items.iter().any(|item| item.section == TaxSection::Documents));
    assert!(s.draft.valid_stored());
    let c = s.draft.credits.as_ref().unwrap();
    assert_eq!(c.education.len(), 1);
    assert_ne!(c.education[0].id, 0);
    assert_eq!(c.education[0].form, d.credits.as_ref().unwrap().education[0].form);
    assert_eq!(c.education[0].student, 0);
    assert_eq!(c.education[0].assigned, crate::DocumentRecipient::Unknown);
    assert_eq!(c.education[0].owner_source, None);
    assert!(c.students.is_empty());
    assert_eq!(c.care, care);
    let pending = estimate(&s.draft);
    assert_eq!(pending.state, ResultState::Incomplete);
    assert_eq!(pending.tax, None);
    let issue = pending
        .issues
        .iter()
        .find(|issue| issue.display.key == "ISSUE_WHOSE_DOCUMENT_IS_THIS")
        .expect("the retained tuition statement needs a student");
    assert_eq!(
        crate::interview::issue_pages(&s.draft, &issue.context, Some(&issue.display.key)),
        vec![TaxSection::Documents]
    );
    let mut request = crate::tests::support::client::income_pages::open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut request else {
        panic!("expected the document center, got {request:?}")
    };
    documents.common.education.students.clear();
    accepted(&send(&mut s, request));
    assert!(s.draft.dependents().iter().all(|p| p.id != child));
    let c = s.draft.credits.as_ref().unwrap();
    assert!(c.education.is_empty());
    assert!(c.students.is_empty());
    assert_eq!(c.care, care);
    assert_eq!(value(&supported(&s.draft), "prior_year_care_credit"), 132);
}

// Kept and contradictory, surfaced: head of household or qualifying
// surviving spouse with every person removed stays stored; the estimate
// refuses (Filing status) and the review lists it. No save-time warning.
#[test]
fn household_status_without_a_person_is_kept_and_refused_by_the_estimate() {
    for status in [FilingChoice::Head, FilingChoice::Surviving] {
        let mut d = head_with_child();
        if status == FilingChoice::Surviving {
            d.setup = Some(FederalSetup {
                status,
                marital_state: MaritalState::WidowedPriorYear { could_file_joint: true },
                ..d.setup.clone().unwrap()
            });
        }
        supported(&d);
        let mut s = Interview::new(TEST_USER, d.clone());
        let reply = send(&mut s, save_children(&d, vec![]));
        // Next follows the page order (review=false): the document center.
        assert_eq!(opened(&reply).0, "documents");
        assert_eq!(s.draft.setup.as_ref().unwrap().status, status);
        assert!(s.draft.dependents().is_empty());
        assert_eq!(estimate(&s.draft).state, ResultState::Unsupported);
        assert!(has_issue(&s.draft, "Filing status"), "{:?}", issues(&s.draft));
        // The page-1 rule that a household status needs a person is not
        // re-applied to the people page: an empty list is accepted.
    }
}

// Spouse amounts on later pages are spouse-owned: leaving a joint return
// erases them (the IRA spouse part, Saver's Credit and educator expenses),
// so the estimate has nothing to refuse.
#[test]
fn spouse_amounts_are_erased_leaving_joint() {
    // Spouse IRA facts (IraSpouse::Joint), to Single and to separate.
    for filing in [Filing::Single, separate_setup().filing] {
        let mut d = complete(FederalStatus::Joint);
        let mut iras = ira_facts(0, 0, Answer::No);
        iras.spouse = spouse_ira(750_000);
        d.adjustments.as_mut().unwrap().iras = iras;
        assert_eq!(value(&supported(&d), "ira_deduction"), 7500);
        let d = crate::interview::with_ticks(d);
        let mut s = Interview::new(TEST_USER, d.clone());
        accepted(&send(&mut s, save_setup(&d, setup_value(filing))));
        let kept = &s.draft.adjustments.as_ref().unwrap().iras;
        assert_eq!(kept.spouse, IraSpouse::NotApplicable);
        assert!(!has_issue(&s.draft, "remove spouse IRA facts"));
    }
    // Saver's Credit spouse contributions.
    let mut d = complete(FederalStatus::Joint);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 100_000)).into();
    let mut s = Interview::new(TEST_USER, d.clone());
    accepted(&send(&mut s, save_setup(&d, setup_value(Filing::Single))));
    let kept = s.draft.credits.as_ref().unwrap().saver.as_ref().unwrap();
    assert_eq!(kept.spouse_contributions, 0);
    assert!(!has_issue(&s.draft, "Revisit Saver’s Credit after a filing-status change"));
    // Educator expenses of the spouse.
    let mut d = complete(FederalStatus::Joint);
    d.adjustments.as_mut().unwrap().educators = EducatorClaim {
        taxpayer: None,
        spouse: Some(EducatorRole::Educator { classroom: 20_000, instructional: 0 }),
    };
    let d = crate::interview::with_ticks(d);
    let mut s = Interview::new(TEST_USER, d.clone());
    accepted(&send(&mut s, save_setup(&d, setup_value(Filing::Single))));
    assert_eq!(s.draft.adjustments.as_ref().unwrap().educator_spouse(), 0);
    assert!(!has_issue(&s.draft, "Spouse educator expenses require a joint return."));
}

// A separately filing spouse's itemizing answer requires itemization without
// creating a voluntary election. Filing-status corrections retain other facts.
fn spouse_itemizes_return() -> FederalDraft {
    let mut d = complete(FederalStatus::Separate);
    d.deductions.as_mut().unwrap().spouse_itemizes = Some(true);
    {
        d.deductions.as_mut().unwrap().itemized = ItemizedExpenses {
            income_tax_owners: None,
            investment_property_tax_owners: None,
            investment_income_tax: crate::InvestmentIncomeTax::Unresolved,
            real_estate_taxes: 0,
            other_lines: vec![],
            long_term_care: vec![],
            medical: 0,
            personal_property_taxes: 0,
            personal_tax: PersonalTaxElection::None,
            investment_property_taxes: 0,
        };
        d.deductions.as_mut().unwrap().itemize_override = None;
    };
    d.finished_sections.push(TaxSection::Itemized);
    d.credits.as_mut().unwrap().care.filing =
        crate::CareFilingStatus::Separate { spouse_earned_income: None };
    crate::interview::with_ticks(d)
}
// 2026-11-02: leaving separate filing removes only its spouse fact.
#[test]
fn leaving_separate_clears_spouse_itemizing_and_preserves_deduction_facts() {
    let d = source_baseline(spouse_itemizes_return());
    let mut s = Interview::new(TEST_USER, d.clone());
    let reply = send(&mut s, reviewing(save_setup(&d, setup_value(Filing::Single))));
    assert_eq!(opened(&reply).0, "review");
    assert_eq!(s.draft.deductions.as_ref().unwrap().spouse_itemizes, None);
    assert_eq!(s.draft.deductions.as_ref().unwrap().itemize_override, None);
    assert_eq!(
        s.draft.deductions.as_ref().unwrap().itemized,
        d.deductions.as_ref().unwrap().itemized
    );
    assert_eq!(s.draft.income, d.income);
    assert_eq!(value(&supported(&s.draft), "deduction"), 16_100);
    assert_eq!(s.draft.credits.as_ref().unwrap().care.filing, crate::CareFilingStatus::Other);
}

// A union change that leaves a later page valid does not reopen it:
// separate (the spouse does not itemize) to single keeps the itemizing
// route, which a single return offers too.
#[test]
fn a_union_change_that_keeps_a_later_page_valid_does_not_reopen_it() {
    let mut d = spouse_itemizes_return();
    d.deductions.as_mut().unwrap().itemize_override = Some(ItemizeOverride::Itemize);
    d.deductions.as_mut().unwrap().spouse_itemizes = Some(false);
    let d = source_baseline(d);
    let mut s = Interview::new(TEST_USER, d.clone());
    let warnings = setup_warnings(&mut s, setup_value(Filing::Single));
    // The accepted separate-return care context is reset; deductions remain valid.
    assert_eq!(
        d.credits.as_ref().unwrap().care.filing,
        crate::CareFilingStatus::Separate { spouse_earned_income: None },
    );
    assert_eq!(warnings, vec![Message::new("SAVING_REMOVES_CARE_SOURCE_ANSWERS", vec![])]);
    let reply = send(&mut s, reviewing(save_setup(&d, setup_value(Filing::Single))));
    assert_eq!(opened(&reply).0, "review");
    assert_eq!(s.draft.credits.as_ref().unwrap().care.filing, crate::CareFilingStatus::Other);
    assert_eq!(s.draft.income, d.income);
    assert!(s.draft.finished_sections.contains(&TaxSection::Deductions));
    assert_eq!(
        s.draft.deductions.as_ref().unwrap().itemize_override,
        Some(ItemizeOverride::Itemize)
    );
}

// Kept but harmless: the Form 3800 "spouse has no business credit" answer
// (married filing separately only) stays stored after a change to joint;
// the calculation reads it only on a separate return.
#[test]
fn separate_only_business_credit_answer_is_kept_but_ignored() {
    let mut d = complete(FederalStatus::Separate);
    d.credits.as_mut().unwrap().business_credits = Some(BusinessCreditHistory {
        credits: Vec::new(),
        carryforwards: vec![],
        carrybacks: vec![],
        spouse_without_credits: true,
    });
    supported(&d);
    let mut s = Interview::new(TEST_USER, d.clone());
    let reply = send(&mut s, save_setup(&d, setup_value(Filing::MarriedJointly)));
    assert_eq!(opened(&reply).0, "you");
    let kept = s.draft.credits.as_ref().unwrap().business_credits.as_ref();
    assert!(kept.is_some_and(|h| h.spouse_without_credits));
    let source = include_str!("../../credits/general_business_credit.rs");
    assert!(source.contains("status == FederalStatus::Separate && !spouse_without_credits"));
}

// Community property (married filing separately only). It is asked on the
// people page, not by SaveSetup. An unchanged Next on page 1 is
// navigation: page 1 as it opened is accepted whatever it holds. Leaving
// separate resets the scope to unanswered. Source documents retain their owners
// and facts, and an incompatible owner becomes a review issue.
#[test]
fn stored_community_property_return_passes_page_one_unchanged() {
    let d = community_w2_draft(4_000_000, 6_000_000, 400_000, 600_000);
    supported(&d);
    let mut s = Interview::new(TEST_USER, d.clone());
    let same = return_setup(d.setup.as_ref().unwrap());
    let reply = send(&mut s, save_setup(&d, same));
    assert_eq!(opened(&reply).0, "you");
    assert_eq!(s.draft, d);
    assert!(!s.dirty);
    let reply = send(&mut s, save_setup(&d, setup_value(Filing::Single)));
    assert_eq!(opened(&reply).0, "you");
    assert_eq!(s.draft.setup.as_ref().unwrap().community_property, CommunityProperty::NotAnswered);
    assert_eq!(s.draft.income, d.income);
    assert!(has_issue(&s.draft, "Spouse-owned income requires a joint return"));
}

// Estimated payments entered while filing jointly are the couple's
// (FederalPayments.joint, the one context field: no page rule sees it).
// A separate return must divide them (Pub. 505), so the change erases them,
// named first, and the walk asks the payments page again.
fn joint_estimated_payments() -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    let pay = d.payments.as_mut().unwrap();
    pay.estimated = crate::EstimatedPaymentEntryRecord::Sources {
        payments: vec![
            DatedPayment {
                payer: crate::PaymentPayer::Joint,
                refund_property: None,
                date: 20260415,
                amount: 400_000,
            },
            DatedPayment {
                payer: crate::PaymentPayer::Joint,
                refund_property: None,
                date: 20260615,
                amount: 400_000,
            },
        ],
    };

    crate::interview::with_ticks(d)
}
#[test]
fn joint_estimated_payments_are_asked_again_on_a_separate_return() {
    let d = joint_estimated_payments();
    assert_eq!(return_line(&supported(&d), "26"), 800_000);
    let d = source_baseline(d);
    let mut s = Interview::new(TEST_USER, d.clone());
    let warnings = setup_warnings(&mut s, separate_setup());
    assert_warning(&warnings, "SAVING_REMOVES_YOUR_SPOUSE", vec![]);
    assert_warning(&warnings, "SAVING_REMOVES_YOUR_FEDERAL_PAYMENTS", vec![]);
    let reply = send(&mut s, reviewing(save_setup(&d, separate_setup())));
    // 2026-10-30: the separate return's community-property step, unanswered,
    // is the first open step, so the walk asks it first.
    assert_eq!(opened(&reply).0, "community property");
    let pay = s.draft.payments.as_ref().unwrap();
    assert_eq!(pay.estimated, crate::EstimatedPaymentEntryRecord::None);
    assert!(!s.draft.finished_sections.contains(&TaxSection::Payments));
    // The separate return's own pages ask the community and spouse facts.
    answer_separate_pages(&mut s.draft);
    assert_eq!(return_line(&supported(&s.draft), "26"), 0);
}

// One filer's payments stay valid on a joint return, but the payments page
// now asks both spouses': Single -> Joint keeps them and reopens the page
// (level 2), after the spouse page, in walk order.
#[test]
fn single_to_joint_opens_the_spouse_page_then_reasks_in_walk_order() {
    let mut d = complete(FederalStatus::Single);
    d.payments.as_mut().unwrap().estimated = crate::EstimatedPaymentEntryRecord::Sources {
        payments: vec![DatedPayment {
            payer: crate::PaymentPayer::Joint,
            refund_property: None,
            date: 20260415,
            amount: 300_000,
        }],
    };

    let d = crate::interview::with_ticks(d);
    let d = source_baseline(d);
    let mut s = Interview::new(TEST_USER, d.clone());
    let joint = setup_value(Filing::MarriedJointly);
    let warnings = setup_warnings(&mut s, joint.clone());
    assert_warning(
        &warnings,
        "SAVING_REOPENS",
        vec![MessageArg::Number(TaxSection::Payments as i64)],
    );
    let reply = send(&mut s, reviewing(save_setup(&d, joint)));
    assert_eq!(opened(&reply).0, "spouse");
    assert_eq!(
        crate::kernel::ratio::exact::rounded(
            &s.draft.payments.as_ref().unwrap().estimated_source()
        ),
        Some(300_000)
    );
    let request = save_spouse(&s.draft, adult("Robin", "Sample", 19820202));
    let reply = send(&mut s, request);
    let walked = walk_to_review(&mut s, reply);
    // Next follows the page order: people, documents, direct income, then checklist.
    assert_eq!(walked.first().map(String::as_str), Some("BeginChildren"));
    assert_eq!(walked.get(1).map(String::as_str), Some("SaveDocuments"));
    assert_eq!(walked.get(2).map(String::as_str), Some("SaveIncomeWithoutForm"));
    assert_eq!(walked.get(3).map(String::as_str), Some("SaveChecklist"));
    let payments = walked.iter().position(|p| p == "SavePayments").unwrap();
    assert_eq!(payments, walked.len() - 1, "{walked:?}");
    // The payments page saved again records the joint household.
    assert!(s.draft.payments.as_ref().unwrap().joint);
    assert!(s.draft.finished_sections.contains(&TaxSection::Payments));
}

// Kept without a warning: a filer or spouse marked as died in 2026 is a final
// return the estimate computes; a filer who can be claimed while claiming
// people is treated as having no dependents, with a note.
#[test]
fn person_fact_changes_are_accepted_and_refused_by_the_estimate() {
    let d = complete(FederalStatus::Joint);
    let mut s = Interview::new(TEST_USER, d.clone());
    let mut spouse = filer(d.people.as_ref().unwrap().spouse.as_ref().unwrap());
    spouse.details.more = died_more(20260615);
    let reply = send(&mut s, save_spouse(&d, spouse));
    assert_eq!(opened(&reply).0, "people");
    assert!(!has_issue(&s.draft, "About your spouse"));
    assert!(s.draft.people.as_ref().unwrap().spouse.as_ref().unwrap().died.is_some());

    let d = head_with_child();
    let mut s = Interview::new(TEST_USER, d.clone());
    let mut you = filer(&d.people.as_ref().unwrap().taxpayer);
    you.details.claimable = Claimable::Possibly {
        another_eitc_child: false,
        dependency: FilerDependency::CanBeClaimed { claimed: true },
        us_home_over_half_year: true,
        full_time_student: false,
        disabled: false,
        incapable_self_care: false,
        support_share: SupportShare::MoreThanHalf,
        parent_alive: true,
    };
    let reply = send(&mut s, save_you(&d, you));
    assert_eq!(opened(&reply).0, "people");
    assert!(!has_issue(&s.draft, "A dependent filer with potential dependents"));
    assert!(noted(&estimate(&s.draft), "NOTE_DEPENDENT_FILER_HAS_NO_DEPENDENTS"));
}

// Walk-owned (u5-losses): Form 8615's parent facts belong to the parent's
// return page, in the walk while Form 8615 applies. A birth date that ends
// it removes them, named by the running estimate before the save; the
// return is the same as one that never had them.
#[test]
fn parent_tax_facts_go_when_form_8615_no_longer_applies() {
    let d = young_interest_return();
    let mut s = Interview::new(TEST_USER, d.clone());
    let mut you = filer(&d.people.as_ref().unwrap().taxpayer);
    you.birth_date = 19800101;
    let warnings = match send(
        &mut s,
        P::EstimateYou {
            professional_election: crate::ProfessionalRentalElection::SeparateActivities,
            allowed_professional_elections: vec![0, 1],
            user: TEST_USER,
            you: you.clone(),
        },
    ) {
        Reply::Response(P::RunningEstimate { warnings, .. }) => warnings
            .iter()
            .map(|warning| super::status_message(&warning.message))
            .collect::<Vec<_>>(),
        other => panic!("expected a running estimate, got {other:?}"),
    };
    assert_warning(&warnings, "SAVING_REMOVES_YOUR_PARENT_S_RETURN_FORM_8615", vec![]);
    accepted(&send(&mut s, save_you(&d, you)));
    assert!(s.draft.people.as_ref().unwrap().parent_tax.is_none());
    let mut with = s.draft.clone();
    with.people.as_mut().unwrap().parent_tax = Some(ordinary_parent());
    assert_eq!(supported(&s.draft).tax, supported(&with).tax);
}

// ---------------------------------------------------------------------------
// Concern 3: where Next goes after page 1 is saved.

// The page-1 editor a reply opened, resent with `value`.
fn resend_setup(reply: Reply, value: ReturnSetup) -> (bool, P) {
    let Reply::Edit(P::SaveSetup {
        user,
        cookie,
        revision,
        page,
        previous,
        review,
        estimate,
        progress,
        ..
    }) = reply
    else {
        panic!("expected page 1, got {reply:?}")
    };
    (
        review,
        P::SaveSetup {
            user,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            warnings: vec![],
            progress,
            value,
        },
    )
}
fn open_section(d: &FederalDraft, from: i32, section: TaxSection) -> P {
    P::OpenSection {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        from,
        section,
        reveal_path: None,
    }
}
fn single() -> ReturnSetup {
    setup_value(Filing::Single)
}

#[test]
fn first_pass_next_on_page_one_opens_your_page_changed_or_not() {
    // (a) First pass, review=false.
    let mut s = Interview::new(TEST_USER, FederalDraft::empty("audit-first".into()));
    let request = save_setup(&s.draft, single());
    let reply = send(&mut s, request);
    assert_eq!(opened(&reply).0, "you");
    let request = save_you(&s.draft, adult("Casey", "Sample", 19900101));
    let reply = send(&mut s, request);
    // Gate fixes batch A: every return asks the people page next.
    assert_eq!(opened(&reply).0, "people");
    // Back from your page reopens page 1 with review=false.
    let request = back(&s.draft, PAGE_SETUP);
    let reply = send(&mut s, request);
    let (review, request) = resend_setup(reply, single());
    assert!(!review);
    // Unchanged Next: your page.
    let revision = s.draft.revision;
    assert_eq!(opened(&send(&mut s, request)).0, "you");
    assert_eq!(s.draft.revision, revision);
    // Changed Next: your page too.
    let request = back(&s.draft, PAGE_SETUP);
    let reply = send(&mut s, request);
    let (_, request) = resend_setup(reply, separate_setup());
    assert_eq!(opened(&send(&mut s, request)).0, "you");
    assert_eq!(s.draft.revision, revision + 1);
}

#[test]
fn completed_return_back_to_page_one_follows_the_page_order() {
    // A finished return reopened through Back (review=false): changed or
    // unchanged, Next opens your page, never the review.
    for value in [single(), separate_setup()] {
        let mut d = complete(FederalStatus::Single);
        d.finished = true;
        let mut s = Interview::new(TEST_USER, d.clone());
        let reply = send(&mut s, back(&d, PAGE_SETUP));
        let (review, request) = resend_setup(reply, value);
        assert!(!review);
        assert_eq!(opened(&send(&mut s, request)).0, "you");
    }
    // Navigate from a page that is not the review (here your page's place)
    // also opens page 1 without the review flag.
    let d = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d.clone());
    let reply = send(&mut s, go_to(&d, PAGE_PEOPLE, TaxSection::Setup));
    let (review, request) = resend_setup(reply, separate_setup());
    assert!(!review);
    assert_eq!(opened(&send(&mut s, request)).0, "you");
}

#[test]
fn page_one_opened_from_the_review_returns_to_the_review() {
    // (b) Opened from the review by Go to (Navigate) or a row link
    // (OpenSection): review=true. Unchanged Next returns to the review;
    // a change returns to the review too unless it opened a page (the
    // spouse page for joint). Your page, the people page (asked once on
    // every return) and the checklist are not shown again.
    let d = source_baseline(complete(FederalStatus::Single));
    let requests =
        [from_review as fn(&mut Interview, TaxSection) -> Reply, |s: &mut Interview, section| {
            let request = open_section(&s.draft, PAGE_REVIEW, section);
            send(s, request)
        }];
    for open in requests {
        for (value, next) in [
            (single(), "review"),
            // 2026-10-30: a separate return first asks its community-property step.
            (separate_setup(), "community property"),
            (head_setup(), "review"),
            (setup_value(Filing::MarriedJointly), "spouse"),
        ] {
            let mut s = Interview::new(TEST_USER, d.clone());
            let reply = open(&mut s, TaxSection::Setup);
            let (review, request) = resend_setup(reply, value);
            assert!(review);
            assert_eq!(opened(&send(&mut s, request)).0, next);
        }
    }
}

#[test]
fn page_one_changed_from_the_review_mid_interview_resumes_at_the_first_open_section() {
    // Mid-interview (income not yet continued), page 1 opened from the
    // review and changed: Next resumes at the first unfinished step, not at
    // your page or Beyond your documents. 2026-10-30: becoming a separate
    // return opens its community-property step first; once it is answered,
    // the return resumes at the self-employment walk.
    let mut s = Interview::new(TEST_USER, FederalDraft::empty("audit-mid".into()));
    let request = save_setup(&s.draft, single());
    send(&mut s, request);
    let request = save_you(&s.draft.clone(), adult("Casey", "Sample", 19900101));
    send(&mut s, request);
    let request = save_children(&s.draft.clone(), vec![]);
    let reply = send(&mut s, request);
    let Reply::Edit(request @ P::SaveDocuments { .. }) = reply else {
        panic!("expected the document center, got {reply:?}")
    };
    let reply = send(&mut s, request);
    let Reply::Response(P::BeginIncomeWithoutForm { .. }) = reply else {
        panic!("expected income without a form, unanswered")
    };
    let mut request = crate::tests::cross::interview::without_form_editor(reply);
    let P::SaveIncomeWithoutForm { value, .. } = &mut request else { unreachable!() };
    value.self_work = true;
    let reply = send(&mut s, request);
    let request = crate::tests::cross::interview::tick(reply, Checklist::NONE);
    accepted(&send(&mut s, request));
    let reply = from_review(&mut s, TaxSection::Setup);
    let (review, request) = resend_setup(reply, separate_setup());
    assert!(review);
    let next = send(&mut s, request);
    assert_eq!(opened(&next).0, "community property");
    let request = crate::tests::cross::interview::answer(next, Filing::MarriedSeparately);
    accepted(&send(&mut s, request));
    assert_eq!(opened(&send(&mut s, P::OpenInterview { user: TEST_USER })).0, "self-employment");
}

// ---------------------------------------------------------------------------
// Additional scope: unchecking a box that has entries.

#[test]
fn checklist_preserves_business_entries_and_their_canonical_tick() {
    let mut d = complete(FederalStatus::Single);
    for name in ["Bakery", "Studio"] {
        add(&mut d, FederalIncomeEntry { label: name.into(), ..income(cash_business(1_000_000)) });
    }
    let d = source_baseline(crate::interview::with_ticks(d));
    let mut s = Interview::new(TEST_USER, d.clone());
    let reply = from_review(&mut s, TaxSection::Checklist);
    let Reply::Edit(P::SaveChecklist { value, .. }) = &reply else {
        panic!("expected the checklist, got {reply:?}")
    };
    assert_eq!(value.activities & ActivityTopics::BUSINESS.0, 0);
    let calculation = send(&mut s, P::EstimateChecklist { user: TEST_USER, value: *value });
    let Reply::Response(P::RunningEstimate { warnings, .. }) = calculation else {
        panic!("expected a running estimate, got {calculation:?}")
    };
    assert!(warnings.is_empty(), "{warnings:?}");
    let request = crate::tests::cross::interview::tick(reply, Checklist::NONE);
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.income, d.income);
    assert_ne!(s.draft.checklist.activities & ActivityTopics::BUSINESS.0, 0);
    assert_eq!(s.draft.revision, d.revision);
}

// ---------------------------------------------------------------------------
// An unchanged Next is navigation: a page reopened from the review and
// saved as it opened keeps the revision, shows no warning and returns to
// the review (page 1 and the checklist included).

#[test]
fn unchanged_next_on_any_page_keeps_the_revision() {
    // Every page answered (a planned topic's first save records it visited).
    let mut d = rich(FederalStatus::Joint, true, SpouseFacts::Plain, false);
    // A topic the return's facts make apply is recorded visited by its first
    // save (G-47), so an answered return has every page finished.
    for section in SEQUENCE_UNDER_TEST {
        if !d.finished_sections.contains(&section) {
            d.finished_sections.push(section);
        }
    }
    let mut s = Interview::new(TEST_USER, d.clone());
    let warnings = setup_warnings(&mut s, return_setup(d.setup.as_ref().unwrap()));
    assert!(warnings.is_empty(), "{warnings:?}");
    let Reply::Response(P::Review { allowed_sections, .. }) =
        send(&mut s, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected the review")
    };
    let mut saved = 0;
    for section in SEQUENCE_UNDER_TEST {
        if !allowed_sections.contains(&(section as i32)) {
            continue;
        }
        let Reply::Edit(request) = from_review(&mut s, section) else {
            continue;
        };
        let reply = send(&mut s, request);
        assert_eq!(opened(&reply).0, "review", "{section:?}: {reply:?}");
        assert_eq!(s.draft, d, "{section:?}");
        assert!(!s.dirty);
        saved += 1;
    }
    assert!(saved >= 15, "{saved}");
}
const SEQUENCE_UNDER_TEST: [TaxSection; 30] = [
    TaxSection::Setup,
    TaxSection::People,
    TaxSection::Spouse,
    TaxSection::Children,
    TaxSection::Checklist,
    TaxSection::Wages,
    TaxSection::InterestDividends,
    TaxSection::SelfEmployment,
    TaxSection::HomeOffices,
    TaxSection::Retirement,
    TaxSection::Hsa,
    TaxSection::SelfEmployedBenefits,
    TaxSection::IraContributions,
    TaxSection::Educator,
    TaxSection::StudentLoan,
    TaxSection::Deductions,
    TaxSection::Medical,
    TaxSection::Taxes,
    TaxSection::Mortgage,
    TaxSection::GiftCarryovers,
    TaxSection::OtherItemized,
    TaxSection::Education,
    TaxSection::Saver,
    TaxSection::Marketplace,
    TaxSection::MoreCredits,
    TaxSection::OtherTaxes,
    TaxSection::Payments,
    TaxSection::IncomeByPeriod,
    TaxSection::InjuredSpouse,
    TaxSection::OftenMissed,
];

// ---------------------------------------------------------------------------
// The two generated properties of walk-owned data. Every stored return is
// generated over the five filing statuses, with and without people you
// support (a household status always with them), the spouse's facts on a
// joint return and two checklists, with an answer on every page its walk
// asks; each earlier-answer change is applied through the protocol.

#[derive(Clone, Copy, Debug)]
enum SpouseFacts {
    Plain,
    Died,
    Claimable,
    BlindSenior,
}

// A stored return with an answer on every page this household's walk asks:
// a W-2, a 1099-INT and a business for each filer, HSA, IRA and educator
// expenses, itemized deductions with a cash gift, Saver's Credit, the
// Form 3800 history, dated estimated and extension payments with the 2025
// facts (box E on a joint return), the injured spouse item on a joint
// return; with people you support a child in care and a student with an
// education credit. `planned` adds ticks with nothing entered.
fn rich(
    status: FederalStatus,
    dependents: bool,
    spouse: SpouseFacts,
    planned: bool,
) -> FederalDraft {
    let joint = status == FederalStatus::Joint;
    let mut d = complete(status);
    if joint {
        let facts = d.people.as_mut().unwrap().spouse.as_mut().unwrap();
        match spouse {
            SpouseFacts::Plain => {}
            SpouseFacts::Died => {
                facts.died = death(20260601).map(|f| FinalReturn { ordinary: true, ..f });
            }
            SpouseFacts::Claimable => {
                facts.dependency = FilerDependency::CanBeClaimed { claimed: false };
            }
            SpouseFacts::BlindSenior => {
                facts.blind = true;
                facts.birth_date = 19550101;
            }
        }
    }
    let owners: &[Owner] =
        if joint { &[Owner::Taxpayer, Owner::Spouse] } else { &[Owner::Taxpayer] };
    for owner in owners {
        let who = if *owner == Owner::Taxpayer { "Your" } else { "Spouse" };
        if *owner == Owner::Spouse {
            let id = d.next_id;
            add(&mut d, wage(id, Owner::Spouse, 3_000_000, 300_000));
        }
        let id = d.next_id;
        add(
            &mut d,
            FederalIncomeEntry {
                printed: None,
                refund_management: None,
                state_source: Default::default(),
                id,
                label: alloc::format!("{who} Bank"),
                owner: crate::DocumentOwner::from(*owner),
                income: savings_income(300, 0),
                community: CommunityShare::NotNeeded,
            },
        );
        let id = d.next_id;
        add(
            &mut d,
            FederalIncomeEntry {
                printed: None,
                refund_management: None,
                state_source: Default::default(),
                id,
                label: alloc::format!("{who} Shop"),
                owner: crate::DocumentOwner::from(*owner),
                income: cash_business(20_000),
                community: CommunityShare::NotNeeded,
            },
        );
    }
    // The pages of unit u5-adjust: for each filer a governmental pension
    // 1099-R with public safety officer premiums, a missed required
    // distribution and an alimony agreement paid; the return's lump-sum
    // election. Unit u6-alimony: an alimony agreement received.
    for owner in owners {
        let id = d.next_id;
        let mut pension = retirement_entry(
            *owner,
            RetirementKind::Pension,
            20_000,
            RetirementTreatment::Ordinary,
        );
        pension.label = "".into();
        add(&mut d, pension);
        public_safety(&mut d, id, 2_000, 1_500, PublicSafetyCoverage::Medical);
        let a = d.adjustments.as_mut().unwrap();
        a.alimony_paid.push(AlimonyPaid { owner: *owner, payment: alimony_record(20150301) });
        a.required_distributions
            .push(RequiredDistribution { owner: *owner, ..rmd_record(5_000, 1_000) });
        // Unit form-3903: a move under military orders, by owner.
        a.moving.push(MovingClaim {
            owner: *owner,
            record: MilitaryMove {
                reason: MoveReason::ArmedForcesOrders,
                goods: 100_000,
                trip: None,
                reimbursed: 0,
            },
        });
        // Unit u6-alimony: alimony received under a pre-2019 agreement, on
        // the prizes, royalties, equipment rent and alimony topic.
        let id = d.next_id;
        add(
            &mut d,
            FederalIncomeEntry {
                printed: None,
                refund_management: None,
                state_source: Default::default(),
                id,
                label: "".into(),
                owner: crate::DocumentOwner::from(*owner),
                income: FederalIncome::Alimony { data: alimony_record(20160301) },
                community: CommunityShare::NotNeeded,
            },
        );
    }
    // The Roth IRA pages of unit u6-roth: for each filer a code J 1099-R
    // (the Roth income kind) and that filer's Roth IRA history.
    for owner in owners {
        let mut e = roth_withdrawal(*owner, 3_000, 20260301);
        e.label = "".into();
        add(&mut d, e);
        d.adjustments.as_mut().unwrap().roth_history.push(roth_history(*owner, 2012, 5_000));
    }
    d.adjustments.as_mut().unwrap().social_election = SocialElection::Compare {
        years: vec![SocialBenefitYear {
            year: 2024,
            status: FederalStatus::Single,
            apart: false,
            received: 300_000,
            benefits: 0,
            earlier_lumps: 0,
            agi: 1_000_000,
            taxable: 0,
            refigured: 0,
            exempt: 0,
            addbacks: 0,
        }],
        complete: true,
    };
    let setup = d.setup.clone().unwrap();
    let a = d.adjustments.as_mut().unwrap();
    a.hsa = owners.iter().map(|o| hsa_entry(*o, HsaCoverage::SelfOnly, 200_000)).collect();
    let mut iras = ira_facts(300_000, 0, Answer::No);
    iras.spouse = match setup.ira_spouse_index() {
        1 => spouse_ira(200_000),
        2 => IraSpouse::SeparateTogether { covered_at_work: false },
        _ => IraSpouse::NotApplicable,
    };
    a.iras = iras;
    a.educators = EducatorClaim {
        taxpayer: Some(EducatorRole::Educator { classroom: 10_000, instructional: 0 }),
        spouse: joint.then_some(EducatorRole::Educator { classroom: 20_000, instructional: 0 }),
    };
    let v = d.deductions.as_mut().unwrap();
    {
        v.itemized = ItemizedExpenses {
            income_tax_owners: None,
            investment_property_tax_owners: None,
            investment_income_tax: crate::InvestmentIncomeTax::Unresolved,
            real_estate_taxes: 0,
            other_lines: vec![],
            long_term_care: vec![],
            medical: 0,
            personal_property_taxes: 500_000,
            personal_tax: PersonalTaxElection::None,
            investment_property_taxes: 0,
        };
        v.itemize_override = None;
    };
    v.cash_charity = 50_000;
    // T3-deduct: the deductions page's vehicle loan and casualty event, and
    // the gift page's volunteer cost, 2025 household carryover and a whaling
    // captain for each filer.
    v.vehicle_loans = vec![vehicle_loan("Synthetic Credit Union", 150_000)];
    v.personal_casualties = vec![personal_disaster("Storm", 500_000)];
    v.volunteer_expenses = vec![volunteer_travel(VolunteerCarCost::Mileage { miles: 100 }, 0)];
    v.charity_carryovers.push(prior_household_carry());
    v.whaling_captains = owners
        .iter()
        .map(|o| WhalingCaptainClaim {
            owner: *o,
            ..whaling_claim(vec![whaling_expense(WhalingExpenseKind::CrewAndProvisions, 100_000)])
        })
        .collect();
    // The repayment comparison, offered by $5,000 of 2025 unemployment
    // repaid in 2026.
    add(&mut d, prior_unemployment_repayment(500_000, Owner::Taxpayer));
    d.deductions.as_mut().unwrap().repayment =
        repayment_comparison_draft(RepaymentMethod::Automatic).deductions.unwrap().repayment;
    // The other itemized page's annuity cost, on the return of a spouse who
    // died in 2026.
    if matches!(spouse, SpouseFacts::Died) {
        let id = d.next_id;
        add(
            &mut d,
            retirement_entry(
                Owner::Spouse,
                RetirementKind::Pension,
                1_000,
                RetirementTreatment::Ordinary,
            ),
        );
        d.deductions.as_mut().unwrap().annuity_losses = vec![AnnuityLoss {
            income_id: id,
            started: 20200101,
            cost: 5_000_000,
            recovered: 1_000_000,
            refunds: 0,
            eligible: true,
        }];
    }
    // Walked: the itemized hub and every topic the return makes necessary.
    d.finished_sections.extend([
        TaxSection::HomeOffices,
        TaxSection::Itemized,
        TaxSection::Medical,
        TaxSection::Taxes,
        TaxSection::Mortgage,
        TaxSection::GiftCarryovers,
        TaxSection::OtherItemized,
        TaxSection::SelfEmployedBenefits,
        TaxSection::IraContributions,
        TaxSection::Education,
        TaxSection::Saver,
        TaxSection::Marketplace,
        TaxSection::MoreCredits,
    ]);
    let c = d.credits.as_mut().unwrap();
    c.saver = Some(saver(300_000, 0)).into();
    c.business_credits = Some(BusinessCreditHistory {
        credits: Vec::new(),
        carryforwards: vec![],
        carrybacks: vec![],
        spouse_without_credits: status == FederalStatus::Separate,
    });
    if dependents {
        let id = d.next_id;
        let mut ava = dependent_record(id, "Ava", "Sample", 20180214);
        ava.situation = DependentSituation::Supported {
            your_support_over_half: true,
            gross_income: ava.gross_income(),
        };
        ava.care = Some(DependentCare::Yes { care_expenses: 300_000 });
        let mut ben = dependent_record(id + 1, "Ben", "Sample", 20040301);
        ben.situation = DependentSituation::Student;
        d.dependents = vec![ava, ben];
        d.next_id += 2;
        let c = d.credits.as_mut().unwrap();
        c.care = FederalCare::default();
        let student = id + 1;
        set_education(
            c,
            vec![{
                let mut pair = native_education(
                    student,
                    education_expenses(300_000),
                    EducationChoice::LifetimeLearning,
                );

                pair.0.id = 0;
                pair.0.half_time = true;
                pair.0.graduate = true;
                pair.1.more.american_opportunity_used_up = Some(false);
                pair
            }],
        );
    }
    // The other taxes page (u5-credits): a household employer and a Trump
    // account for each filer, and a Form 8828 gift; More credits' home
    // kerosene item, stored with the payments.
    let screening = d.screening.as_mut().unwrap();
    screening.household_employment = owners
        .iter()
        .map(|o| HouseholdEmployer { owner: *o, ..household_employer(3_500) })
        .collect();
    screening.trump_accounts = owners
        .iter()
        .map(|o| TrumpAccount {
            owner: *o,
            contributions: 600_000,
            exempt: 0,
            returns: vec![],
            confirmed: true,
        })
        .collect();
    // The Investment sales topic's home sale (u6-home), with the spouse's
    // answers on a joint return, and the Form 8828 sale bound to it.
    let home_id = d.next_id;
    add(
        &mut d,
        FederalIncomeEntry {
            printed: None,
            refund_management: None,
            state_source: Default::default(),
            id: home_id,
            label: "Oak Street".into(),
            owner: crate::DocumentOwner::Yours,
            income: FederalIncome::HomeSale {
                data: MainHomeSale {
                    form: crate::HomeSaleSource::Paper(home_sale_paper(Form1099S {
                        filer: Default::default(),
                        transferor: Default::default(),
                        address: "Oak Street".into(),
                        closed: 20260701,
                        proceeds: 40_000_000,
                        property_received: false,
                        foreign_person: false,
                    })),
                    answers: Some(crate::RealEstateSaleUse::MainHome(HomeSaleAnswers {
                        property_value: SourceCents::NotReported,
                        firpta_withheld: SourceCents::NotReported,
                        form_1099s: true,
                        acquired: 20200115,
                        received: HomeReceived::Bought,
                        payment: SalePayment::InFull,
                        exchange: None,
                        expenses: 2_000_000,
                        basis: 30_000_000,
                        lived: HomeLived::WholeTime,
                        earlier_sale: EarlierHomeSale::No,
                        spouse: if joint {
                            HomeSpouse::Joint {
                                lived: HomeLived::Days { days: 900 },
                                earlier_sale: EarlierHomeSale::Yes { sold: 20250101 },
                            }
                        } else {
                            HomeSpouse::NotJoint
                        },
                        reason: HomeSaleReason::TwoYears,
                        use_history: HomeRentalUse {
                            depreciation: 0,
                            nonqualified_days: 0,
                            separate: None,
                        },
                        service: None,
                        more: HomeSaleMore::Nothing,
                    })),
                },
            },
            community: CommunityShare::NotNeeded,
        },
    );
    d.credits.as_mut().unwrap().mortgage_subsidies = vec![MortgageSubsidyEntry {
        label: "Oak Street".into(),
        disposition: MortgageSubsidyDisposition::Ordinary {
            subsidy: MortgageSubsidyKind::Qmb,
            transfer: MortgageSubsidyTransfer::Sale {
                home_sale_income_id: home_id,
                qsml_gain_share: None,
            },
            facts: ordinary_subsidy_facts(),
        },
    }];
    d.credits.as_mut().unwrap().mortgage_subsidies.push(MortgageSubsidyEntry {
        label: "Maple Street".into(),
        disposition: MortgageSubsidyDisposition::Ordinary {
            subsidy: MortgageSubsidyKind::Mcc,
            transfer: MortgageSubsidyTransfer::GiftToOther {
                owners: None,
                fair_market_value: 30_000_000,
                adjusted_basis: 20_000_000,
                no_consideration_or_debt_relief: true,
            },
            facts: ordinary_subsidy_facts(),
        },
    });
    let pay = d.payments.as_mut().unwrap();
    pay.home_kerosene = Some(FederalHomeKerosene {
        gallons: 100.0,
        actual_cost: 50_000,
        qualified: true,
        not_claimed_elsewhere: true,
        records_complete: true,
    });
    pay.estimated = crate::EstimatedPaymentEntryRecord::Sources {
        payments: vec![DatedPayment {
            payer: crate::PaymentPayer::Joint,
            refund_property: None,
            date: 20260415,
            amount: 200_000,
        }],
    };

    pay.extension = 50_000;
    pay.extension_payer = Some(if joint { PaymentPayer::Joint } else { PaymentPayer::Taxpayer });
    pay.prior_year = Some(PriorYearTax {
        full_year: true,
        tax: 500_000,
        agi: Some(6_000_000),
        spouse_tax: if joint { 100_000 } else { 0 },
    });
    if joint {
        let mut allocation = InjuredSpouseAllocation::defaults();
        // This builder represents answered pages. Both dependent claimants
        // were explicitly chosen as the taxpayer on the allocation page.
        allocation.claimants = d
            .dependents
            .iter()
            .map(|p| InjuredClaimantFacts { person: p.id, claimant: Owner::Taxpayer })
            .collect();
        d.injured_spouse = Some(allocation);
        d.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    }
    if planned {
        // The earlier-year operating-loss kind on Self-employment
        // (u5-losses): regular and AMT balances.
        let s = d.screening.as_mut().unwrap();
        s.nol = NolHistory::Records {
            years: vec![NolYear {
                ptp_ebl: crate::PtpEblNol::NoPtpEbl,
                year: 2022,
                balance: 500_000,
                injured_owners: None,
                research_parts: None,
            }],
            confirmed: true,
        };
        s.atnol = AtnolHistory::Records {
            years: vec![AtnolYear { year: 2022, balance: 400_000, special_exception: false }],
            confirmed: true,
        };
    }
    document_care(&mut d);
    // 2026-10-17: an answered casualty page already has server-assigned
    // event/property identities; its unchanged Next must not allocate them.
    for event in &mut d.deductions.as_mut().unwrap().personal_casualties {
        event.id = d.next_id;
        d.next_id += 1;
        for property in &mut event.properties {
            property.id = d.next_id;
            d.next_id += 1;
        }
    }
    let mut d = crate::interview::with_ticks(d);
    if planned {
        d.checklist.carryovers |= CarryoverTopics::MINIMUM_TAX_CREDIT.0;
        d.set_situations(YourSituations::DUAL_STATUS.0, true);
    }
    // Admit the assembled source inventory before asserting the saved invariant.
    let mut d = source_baseline(d);
    // Stored as a save leaves it: the builder asks nothing its walk does
    // not, so the prune removes nothing (a finished page its own save
    // refuses would be reopened, which a stored return may hold).
    let pruned = crate::interview::prune(&mut d);
    assert!(pruned.removed.is_empty(), "{status:?} {pruned:?}");
    assert!(
        pruned.reopened.iter().all(|p| *p == TaxSection::Marketplace),
        "a page refuses its stored answers: {status:?} {pruned:?}"
    );
    d
}
// A young filer (u5-losses): Form 8615 applies with $10,000 of interest,
// the parent's return page answered with a brother on it; with $2,000 it
// does not and the page is not in the walk.
fn young(applies: bool, planned: bool) -> FederalDraft {
    let mut d = young_interest_return();
    let parent = d.people.as_mut().unwrap().parent_tax.as_mut().unwrap();
    parent.others.push(OtherChildTax { label: "Brother".into(), net: 150_000, gains: None });
    if !applies {
        d.income[0].income = savings_income(2000, 0);
        d.people.as_mut().unwrap().parent_tax = None;
    }
    let mut d = crate::interview::with_ticks(d);
    if planned {
        d.set_situations(YourSituations::DUAL_STATUS.0, true);
    }
    let pruned = crate::interview::prune(&mut d);
    assert!(pruned.removed.is_empty(), "{pruned:?}");
    d
}
// Every generated stored return: 28 households and 4 young filers.
fn household_shard(shard: usize) -> Vec<(alloc::string::String, FederalDraft)> {
    let mut out = Vec::new();
    let mut push = |group, name, build: &dyn Fn() -> FederalDraft| {
        if group == shard {
            out.push((name, source_baseline(build())));
        }
    };
    for applies in [false, true] {
        for planned in [false, true] {
            push(0, alloc::format!("young form_8615={applies} planned={planned}"), &|| {
                young(applies, planned)
            });
        }
    }
    for status in [
        FederalStatus::Single,
        FederalStatus::Joint,
        FederalStatus::Separate,
        FederalStatus::Head,
        FederalStatus::Surviving,
    ] {
        for dependents in [false, true] {
            if !dependents && matches!(status, FederalStatus::Head | FederalStatus::Surviving) {
                continue;
            }
            let spouses: &[SpouseFacts] = if status == FederalStatus::Joint {
                &[
                    SpouseFacts::Plain,
                    SpouseFacts::Died,
                    SpouseFacts::Claimable,
                    SpouseFacts::BlindSenior,
                ]
            } else {
                &[SpouseFacts::Plain]
            };
            for spouse in spouses {
                for planned in [false, true] {
                    let group = match status {
                        FederalStatus::Single => 1,
                        FederalStatus::Joint => match spouse {
                            SpouseFacts::Plain => 2,
                            SpouseFacts::Died => 3,
                            SpouseFacts::Claimable => 4,
                            SpouseFacts::BlindSenior => 5,
                        },
                        FederalStatus::Separate => 6,
                        FederalStatus::Head | FederalStatus::Surviving => 7,
                    };
                    push(
                        group,
                        alloc::format!(
                            "{status:?} dependents={dependents} {spouse:?} planned={planned}"
                        ),
                        &|| rich(status, dependents, *spouse, planned),
                    );
                }
            }
        }
    }
    assert_eq!(out.len(), 4);
    out
}

// The earlier-answer changes applied through the protocol.
#[derive(Clone, Copy, Debug)]
enum Change {
    Setup(u8),
    YouYoung,
    YouClaimable,
    SpouseDied,
    SpouseBlind,
    NoPeople,
    RemoveBusinesses,
    UntickActivities,
    TickWholeReturn,
    Route(Option<ItemizeOverride>),
    SpouseItemizes,
    BoxC,
    InjuredSpouse,
    FamilyCoverage,
    // u5-losses: the filer no longer a child, $10,000 of interest, the
    // parent's return not settled, the operating losses deleted.
    YouAdult,
    InterestUp,
    ParentUnsettled,
    NoLosses,
}
fn changes() -> Vec<Change> {
    let mut all: Vec<Change> = (0..5u8).map(Change::Setup).collect();
    all.extend([
        Change::YouYoung,
        Change::YouClaimable,
        Change::SpouseDied,
        Change::SpouseBlind,
        Change::NoPeople,
        Change::RemoveBusinesses,
        Change::UntickActivities,
        Change::TickWholeReturn,
        Change::Route(None),
        Change::Route(Some(ItemizeOverride::Itemize)),
        Change::SpouseItemizes,
        Change::BoxC,
        Change::InjuredSpouse,
        Change::FamilyCoverage,
        Change::YouAdult,
        Change::InterestUp,
        Change::ParentUnsettled,
        Change::NoLosses,
    ]);
    all
}
fn filing(index: u8) -> Filing {
    match index {
        0 => Filing::Single,
        1 => Filing::MarriedJointly,
        2 => separate_setup().filing,
        3 => Filing::HeadOfHousehold { paid_over_half_home: true, marital: HeadMarital::Unmarried },
        _ => Filing::QualifyingSurvivingSpouse {
            spouse_died: DeathYear::DiedPriorYear,
            paid_over_half_home: true,
            could_file_joint: true,
        },
    }
}
// Applies one change as the filer would: the page opened from the review,
// its value changed, and saved. None when this return has no such page or
// the change leaves the page as it was.
fn apply(s: &mut Interview, change: Change) -> Option<Reply> {
    let edit = |s: &mut Interview, section: TaxSection| match from_review(s, section) {
        Reply::Edit(request) => Some(request),
        _ => None,
    };
    let request = match change {
        Change::Setup(index) => {
            let value = setup_value(filing(index));
            // A household status needs a person on the people page.
            if index >= 3 && s.draft.dependents().is_empty() {
                return None;
            }
            resend_setup(from_review(s, TaxSection::Setup), value).1
        }
        Change::YouYoung | Change::YouClaimable | Change::YouAdult => {
            let P::SavePeople { mut you, .. } = edit(s, TaxSection::People)? else {
                return None;
            };
            if matches!(change, Change::YouAdult) {
                if you.birth_date == 19800101 {
                    return None;
                }
                you.birth_date = 19800101;
            } else if matches!(change, Change::YouYoung) {
                you.birth_date = 20080101;
            } else {
                you.details.claimable = Claimable::Possibly {
                    another_eitc_child: false,
                    dependency: FilerDependency::CanBeClaimed { claimed: false },
                    us_home_over_half_year: true,
                    full_time_student: false,
                    disabled: false,
                    incapable_self_care: false,
                    support_share: SupportShare::MoreThanHalf,
                    parent_alive: true,
                };
            }
            reviewing(save_you(&s.draft, you))
        }
        Change::SpouseDied | Change::SpouseBlind => {
            let P::SaveSpouse { spouse, .. } = edit(s, TaxSection::Spouse)? else {
                return None;
            };
            let mut spouse = Filer::from(spouse);
            if matches!(change, Change::SpouseDied) {
                spouse.details.more = died_more(20260615);
            } else {
                spouse.details.blind = !spouse.details.blind;
            }
            reviewing(save_spouse(&s.draft, spouse))
        }
        Change::NoPeople => {
            let removed: Vec<_> = s.draft.dependents().iter().map(|person| person.id).collect();
            let mut request = crate::tests::support::client::income_pages::open_document_center(s);
            let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
            let before = documents.common.education.students.len();
            documents
                .common
                .education
                .students
                .retain(|student| !removed.contains(&student.student_id));
            if documents.common.education.students.len() != before {
                // A household edit cannot discard another owner's source.
                // Correct the affected source explicitly before removing its student.
                accepted(&send(s, request));
            }
            edit(s, TaxSection::Children)?;
            let before = s.draft.clone();
            let reply = send(s, reviewing(save_children(&before, vec![])));
            if matches!(reply, Reply::Response(P::Problem { .. })) {
                assert_eq!(s.draft, before);
            }
            accepted(&reply);
            return Some(reply);
        }
        Change::RemoveBusinesses => {
            let mut request = edit(s, TaxSection::SelfEmployment)?;
            let P::SaveIncomes { documents, .. } = &mut request else { return None };
            documents
                .documents
                .retain(|row| !matches!(row.document, crate::IncomeRecord::Business(_)));
            request
        }
        Change::UntickActivities => {
            let reply = from_review(s, TaxSection::Checklist);
            let Reply::Edit(P::SaveChecklist { value, .. }) = &reply else {
                return None;
            };
            let value = Checklist { activities: 0, paid: value.paid, carryovers: value.carryovers };
            crate::tests::cross::interview::tick(reply, value)
        }
        Change::TickWholeReturn => {
            let P::SavePeople { mut you, .. } = edit(s, TaxSection::People)? else {
                return None;
            };
            you.situations |= YourSituations::DUAL_STATUS.0;
            reviewing(save_you(&s.draft, you))
        }
        Change::Route(route) => {
            let P::SaveDeductions {
                user,
                cookie,
                revision,
                page,
                previous,
                review,
                estimate,
                warnings: _,
                progress,
                allowed_repayment,
                document_facts,
                home_sites,
                value,
            } = edit(s, TaxSection::Deductions)?
            else {
                return None;
            };
            P::SaveDeductions {
                user,
                cookie,
                revision,
                page,
                previous,
                review,
                estimate,
                warnings: vec![],
                progress,
                allowed_repayment,
                document_facts,
                home_sites,
                value: Deductions { itemize_override: route, ..value },
            }
        }
        Change::SpouseItemizes => {
            if s.draft.setup.as_ref()?.filing_status() != FederalStatus::Separate {
                return None;
            }
            let current = s.draft.deductions.as_ref()?.spouse_itemizes?;
            let mut request = spouse_itemizes_request(&s.draft, !current);
            let P::SaveSpouseItemizes { review, .. } = &mut request else { unreachable!() };
            *review = true;
            request
        }
        Change::BoxC => {
            let mut request = edit(s, TaxSection::Payments)?;
            let P::SavePayments { value, .. } = &mut request else {
                return None;
            };
            let PenaltyChoice::Figure { income, .. } = &mut value.more.penalty else {
                return None;
            };
            *income = IncomeTiming::ByPeriod;
            request
        }
        Change::InjuredSpouse => {
            // The injured spouse item is on the spouse page of a joint return.
            let P::SaveSpouse { spouse, .. } = edit(s, TaxSection::Spouse)? else {
                return None;
            };
            let mut spouse = Filer::from(spouse);
            spouse.situations ^= SpouseSituations::INJURED_SPOUSE.0;
            reviewing(save_spouse(&s.draft, spouse))
        }
        Change::InterestUp => {
            let mut request = crate::tests::support::client::income_pages::open_document_center(s);
            let P::SaveDocuments { documents, .. } = &mut request else {
                return None;
            };
            let int =
                documents.common.interest_dividends.documents.iter_mut().find_map(|item| {
                    match &mut item.document {
                        crate::InterestDividendDocument::Int1099(v) => Some(v),
                        _ => None,
                    }
                })?;
            if int.interest_income == 1_000_000 {
                return None;
            }
            int.interest_income = 1_000_000;
            request
        }
        Change::ParentUnsettled => {
            let mut request = edit(s, TaxSection::ParentTax)?;
            let P::SaveParentTax { value, .. } = &mut request else {
                return None;
            };
            value.special = ParentSituation::Unsettled;
            request
        }
        Change::NoLosses => {
            let mut request = edit(s, TaxSection::SelfEmployment)?;
            let P::SaveIncomes { documents, .. } = &mut request else {
                return None;
            };
            let before = documents.documents.len();
            documents
                .documents
                .retain(|item| !matches!(item.document, crate::IncomeRecord::NetOperatingLoss(_)));
            if documents.documents.len() == before {
                return None;
            }
            request
        }
        Change::FamilyCoverage => {
            let mut request = edit(s, TaxSection::Hsa)?;
            let P::SaveHsa { value, .. } = &mut request else {
                return None;
            };
            value.accounts.first_mut()?.coverage = HsaCoverageYear::Family;
            request
        }
    };
    Some(send(s, request))
}

// The answers a walk can keep without asking them, each where it is not
// asked; changing one must leave the calculated return unchanged.
type Perturbation = (&'static str, fn(&FederalDraft) -> bool, fn(&mut FederalDraft));
const UNASKED: [Perturbation; 5] = [
    // A business's income source: read only for Form 8615; the page's
    // "Nothing else" answer stores Services.
    (
        "business income source",
        |d| !d.people.as_ref().is_some_and(|p| crate::federal_rules::young_8615(joint_of(d), p)),
        |d| {
            for e in &mut d.income {
                if let FederalIncome::Business { data } = &mut e.income {
                    data.income_source = BusinessIncomeSource::Capital;
                }
            }
        },
    ),
    // The Form 3800 answer asked on a separate return only.
    (
        "Form 3800 spouse answer",
        |d| status_of(d) != FederalStatus::Separate,
        |d| {
            if let Some(h) = d.credits.as_mut().unwrap().business_credits.as_mut() {
                h.spouse_without_credits = !h.spouse_without_credits;
            }
        },
    ),
    // Form 8615's parent facts where the walk does not ask them: the parent's
    // return page is in the walk exactly while Form 8615 applies (u5-losses),
    // including a young filer with $2,700 or less of unearned income.
    (
        "Form 8615 parent",
        |d| !crate::interview::parent_tax_in_walk(d),
        |d| {
            let p = d.people.as_mut().unwrap();
            p.parent_tax = match p.parent_tax {
                Some(_) => None,
                None => Some(ordinary_parent()),
            };
        },
    ),
    // "My spouse itemizes" off a separate return, kept while its page is
    // reopened.
    (
        "spouse itemizes",
        |d| status_of(d) != FederalStatus::Separate,
        |d| {
            let v = d.deductions.as_mut().unwrap();
            v.spouse_itemizes = Some(!v.spouse_itemizes.unwrap_or(false));
        },
    ),
    // A tick with nothing entered.
    ("planned tick", |_| true, |d| d.checklist.carryovers ^= CarryoverTopics::MINIMUM_TAX_CREDIT.0),
];
// The lines where two returns' debug forms differ.
fn diff(a: &FederalDraft, b: &FederalDraft) -> String {
    let (a, b) = (alloc::format!("{a:#?}"), alloc::format!("{b:#?}"));
    a.lines()
        .zip(b.lines())
        .enumerate()
        .filter(|(_, (x, y))| x != y)
        .take(12)
        .map(|(i, (x, y))| alloc::format!("{i}: {x} | {y}"))
        .collect::<Vec<_>>()
        .join("\n")
}
fn joint_of(d: &FederalDraft) -> bool {
    status_of(d) == FederalStatus::Joint
}
fn status_of(d: &FederalDraft) -> FederalStatus {
    d.setup.as_ref().unwrap().filing_status()
}

// Actual 2026-10-15 discovery, groups: young, single, joint plain/died/
// claimable/blind senior, separate, head/surviving. Columns are saves,
// parent_saves, loss_saves, parent_opened, home_saves,
// home_spouse_pruned_outside_joint. The final columns cover the retained
// home sale source and spouse-answer pruning after leaving joint.
const SAVE_COUNTS: [[i32; 6]; 8] = [
    [48, 2, 0, 2, 0, 0],
    [64, 0, 2, 0, 64, 0],
    [72, 0, 2, 0, 72, 12],
    [72, 0, 2, 0, 72, 12],
    [76, 0, 2, 0, 76, 12],
    [72, 0, 2, 0, 72, 12],
    [68, 0, 2, 0, 68, 0],
    [70, 0, 2, 0, 70, 0],
];
// Columns are checked and young_checked, in the same household order.
// Young coverage includes explicit interest-source deletions that make the
// parent facts unasked.
const UNASKED_COUNTS: [[i32; 2]; 8] =
    [[178, 18], [312, 0], [372, 0], [372, 0], [372, 0], [372, 0], [228, 0], [342, 0]];
// Every runtime shard asserts its exact row; these sums therefore preserve the
// original aggregate checks without cross-process test state or a second walk.
const _: () = {
    let mut saves_total = [0; 6];
    let mut unasked_total = [0; 2];
    let mut group = 0;
    while group < 8 {
        let mut column = 0;
        while column < 6 {
            saves_total[column] += SAVE_COUNTS[group][column];
            column += 1;
        }
        column = 0;
        while column < 2 {
            unasked_total[column] += UNASKED_COUNTS[group][column];
            column += 1;
        }
        group += 1;
    }
    let [
        saves,
        parent_saves,
        loss_saves,
        parent_opened,
        home_saves,
        home_spouse_pruned_outside_joint,
    ] = saves_total;
    assert!(saves >= 500);
    assert!(home_saves >= 300 && home_spouse_pruned_outside_joint >= 5);
    assert!(parent_saves >= 2 && loss_saves >= 10 && parent_opened >= 2);
    let [checked, young_checked] = unasked_total;
    assert!(checked >= 2000);
    assert!(young_checked >= 10);
};

// Each household shard checks its observed counters
// independently. The constant matrices retain the original global floors.
fn saves_keep_only_walked_data_cases(shard: usize) -> [i32; 6] {
    // (1) After every save the stored return is its own prune: record and
    // fact cleanup is complete, while source documents retain their facts.
    let mut saves = 0;
    // The u5-losses pages are saved, and a save opens the parent's return.
    let (mut parent_saves, mut loss_saves, mut parent_opened) = (0, 0, 0);
    // Source retention is asserted for every accepted save, including an
    // untick. A non-joint household prunes the spouse's topic answers while
    // retaining the printed home-sale form and every other accepted answer.
    let (mut home_saves, mut home_spouse_pruned_outside_joint) = (0, 0);
    for (name, d) in household_shard(shard) {
        for change in changes() {
            let mut s = Interview::new(TEST_USER, d.clone());
            let Some(reply) = apply(&mut s, change) else {
                continue;
            };
            match change {
                Change::ParentUnsettled => parent_saves += 1,
                Change::NoLosses => loss_saves += 1,
                _ => {}
            }
            if matches!(reply, Reply::Edit(P::SaveParentTax { .. })) {
                parent_opened += 1;
            }
            if matches!(reply, Reply::Response(P::Problem { .. })) {
                // The page's own rules refuse the change; nothing is stored.
                assert_eq!(s.draft, d, "{name} {change:?} {reply:?}");
                continue;
            }
            saves += 1;
            let stored = s.draft.clone();
            if let Some(source) =
                d.income.iter().find(|e| matches!(&e.income, FederalIncome::HomeSale { .. }))
            {
                // 2026-10-27: spouse history is a topic answer, not a printed
                // Form 1099-S fact. Only that answer changes off a joint return.
                let mut expected = source.clone();
                if !joint_of(&stored) {
                    let FederalIncome::HomeSale { data } = &mut expected.income else {
                        unreachable!()
                    };
                    if let Some(crate::RealEstateSaleUse::MainHome(answers)) = &mut data.answers {
                        answers.spouse = HomeSpouse::NotJoint;
                    }
                }
                assert_eq!(
                    stored.income.iter().find(|e| e.id == source.id),
                    Some(&expected),
                    "{name} {change:?}: home sale source changed",
                );
                home_saves += 1;
                let FederalIncome::HomeSale { data } = &source.income else { unreachable!() };
                let native = HomeSale::from(data);
                let had = native.facts.as_ref().and_then(|f| f.spouse_earlier.as_ref());
                if had.is_some() && !joint_of(&stored) {
                    home_spouse_pruned_outside_joint += 1;
                }
            }
            assert!(stored.valid_stored(), "{name} {change:?}");
            let mut again = stored.clone();
            let pruned = crate::interview::prune(&mut again);
            assert_eq!(pruned, Default::default(), "{name} {change:?}");
            assert_eq!(again, stored, "{name} {change:?}: {}", diff(&again, &stored));
        }
    }
    [saves, parent_saves, loss_saves, parent_opened, home_saves, home_spouse_pruned_outside_joint]
}

#[test]
fn saves_keep_only_walked_data_young() {
    assert_eq!(saves_keep_only_walked_data_cases(0), SAVE_COUNTS[0]);
}

#[test]
fn saves_keep_only_walked_data_single() {
    assert_eq!(saves_keep_only_walked_data_cases(1), SAVE_COUNTS[1]);
}

#[test]
fn saves_keep_only_walked_data_joint_plain() {
    assert_eq!(saves_keep_only_walked_data_cases(2), SAVE_COUNTS[2]);
}

#[test]
fn saves_keep_only_walked_data_joint_died() {
    assert_eq!(saves_keep_only_walked_data_cases(3), SAVE_COUNTS[3]);
}

#[test]
fn saves_keep_only_walked_data_joint_claimable() {
    assert_eq!(saves_keep_only_walked_data_cases(4), SAVE_COUNTS[4]);
}

#[test]
fn saves_keep_only_walked_data_joint_blind_senior() {
    assert_eq!(saves_keep_only_walked_data_cases(5), SAVE_COUNTS[5]);
}

#[test]
fn saves_keep_only_walked_data_separate() {
    assert_eq!(saves_keep_only_walked_data_cases(6), SAVE_COUNTS[6]);
}

#[test]
fn saves_keep_only_walked_data_head_and_surviving() {
    assert_eq!(saves_keep_only_walked_data_cases(7), SAVE_COUNTS[7]);
}

// Unit50, 2026-10-15: each household shard checks its observed counters
// independently. The constant matrices retain the original global floors.
fn unasked_data_leaves_the_return_unchanged_cases(shard: usize) -> [i32; 2] {
    // (2) Anything the walk keeps without asking it, changed, leaves the
    // calculated return the same: the walk asks everything the calculation
    // uses.
    let mut checked = 0;
    let mut young_checked = 0;
    for (name, d) in household_shard(shard) {
        for change in changes() {
            let mut s = Interview::new(TEST_USER, d.clone());
            if apply(&mut s, change).is_none() {
                continue;
            }
            let stored = s.draft.clone();
            let lines = crate::draft_return(&estimate(&stored));
            for (what, unasked, perturb) in UNASKED {
                if !unasked(&stored) {
                    continue;
                }
                // The Form 8615 case: a child the person tests reach, with
                // $2,700 or less of unearned income.
                if what == "Form 8615 parent"
                    && stored
                        .people
                        .as_ref()
                        .is_some_and(|p| crate::federal_rules::young_8615(joint_of(&stored), p))
                {
                    young_checked += 1;
                }
                let mut changed = stored.clone();
                perturb(&mut changed);
                assert_eq!(
                    crate::draft_return(&estimate(&changed)),
                    lines,
                    "{name} {change:?}: {what}"
                );
                checked += 1;
            }
            // An untick retains sources, so the two high-interest young cases
            // still need Form 8615. Exercise their source-absence parent-fact
            // coverage separately through an explicit document deletion.
            if shard == 0
                && matches!(change, Change::UntickActivities)
                && crate::interview::parent_tax_in_walk(&stored)
            {
                assert_eq!(stored.income, d.income);
                assert_eq!(crate::federal_rules::form_8615_applies(&estimate(&stored)), Some(true));
                let mut deletion = Interview::new(TEST_USER, stored.clone());
                let mut request = crate::tests::support::client::income_pages::open_document_center(
                    &mut deletion,
                );
                let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
                let removed: Vec<_> = documents
                    .common
                    .interest_dividends
                    .documents
                    .iter()
                    .map(|row| row.id)
                    .collect();
                assert!(!removed.is_empty());
                documents.common.interest_dividends.documents.clear();
                accepted(&send(&mut deletion, request));
                assert!(deletion.draft.income.iter().all(|row| !removed.contains(&row.id)));
                assert_eq!(
                    crate::federal_rules::form_8615_applies(&estimate(&deletion.draft)),
                    Some(false)
                );
                assert!(!crate::interview::parent_tax_in_walk(&deletion.draft));
                assert!(deletion.draft.people.as_ref().unwrap().parent_tax.is_none());
                assert!(deletion.draft.people.as_ref().is_some_and(|people| {
                    crate::federal_rules::young_8615(joint_of(&deletion.draft), people)
                }));
                let (what, unasked, perturb) =
                    UNASKED.iter().find(|(what, _, _)| *what == "Form 8615 parent").unwrap();
                assert!(unasked(&deletion.draft));
                let lines = crate::draft_return(&estimate(&deletion.draft));
                let mut changed = deletion.draft.clone();
                perturb(&mut changed);
                assert_eq!(
                    crate::draft_return(&estimate(&changed)),
                    lines,
                    "{name} {change:?}: explicit source deletion: {what}"
                );
                checked += 1;
                young_checked += 1;
            }
        }
    }
    [checked, young_checked]
}

#[test]
fn unasked_data_leaves_the_return_unchanged_young() {
    assert_eq!(unasked_data_leaves_the_return_unchanged_cases(0), UNASKED_COUNTS[0]);
}

#[test]
fn unasked_data_leaves_the_return_unchanged_single() {
    assert_eq!(unasked_data_leaves_the_return_unchanged_cases(1), UNASKED_COUNTS[1]);
}

#[test]
fn unasked_data_leaves_the_return_unchanged_joint_plain() {
    assert_eq!(unasked_data_leaves_the_return_unchanged_cases(2), UNASKED_COUNTS[2]);
}

#[test]
fn unasked_data_leaves_the_return_unchanged_joint_died() {
    assert_eq!(unasked_data_leaves_the_return_unchanged_cases(3), UNASKED_COUNTS[3]);
}

#[test]
fn unasked_data_leaves_the_return_unchanged_joint_claimable() {
    assert_eq!(unasked_data_leaves_the_return_unchanged_cases(4), UNASKED_COUNTS[4]);
}

#[test]
fn unasked_data_leaves_the_return_unchanged_joint_blind_senior() {
    assert_eq!(unasked_data_leaves_the_return_unchanged_cases(5), UNASKED_COUNTS[5]);
}

#[test]
fn unasked_data_leaves_the_return_unchanged_separate() {
    assert_eq!(unasked_data_leaves_the_return_unchanged_cases(6), UNASKED_COUNTS[6]);
}

#[test]
fn unasked_data_leaves_the_return_unchanged_head_and_surviving() {
    assert_eq!(unasked_data_leaves_the_return_unchanged_cases(7), UNASKED_COUNTS[7]);
}

// A filer change that breaks the people page's own rule (a child no longer
// younger than you): the people page has no mark to reopen, so the review
// names it with the rule's message; nothing is erased.
#[test]
fn a_filer_change_that_breaks_the_people_page_is_a_review_item() {
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    d.dependents = vec![dependent_record(id, "Ava", "Sample", 20180214)];
    d.next_id += 1;
    let d = crate::interview::with_ticks(d);
    let mut s = Interview::new(TEST_USER, d.clone());
    assert!(remaining(&mut s).iter().all(|i| !i.starts_with("Children")));
    let mut you = filer(&d.people.as_ref().unwrap().taxpayer);
    you.birth_date = 20190101;
    accepted(&send(&mut s, reviewing(save_you(&d, you))));
    assert_eq!(s.draft.dependents().len(), 1);
    let items = remaining(&mut s);
    assert!(
        items.iter().any(|i| i.starts_with("Children and people you support: ")
            && i.contains(crate::interview::resource_text("DEPENDENT_OLDER_THAN_YOU"))),
        "{items:?}"
    );
}

// Unit u6-alimony: a generated joint household saved as a single return
// through page 1 keeps the filer's alimony received and erases the
// spouse's (outside allowed_owners).
#[test]
fn leaving_joint_through_page_1_erases_the_spouses_alimony_received() {
    let d = rich(FederalStatus::Joint, false, SpouseFacts::Plain, false);
    let alimony = |d: &FederalDraft| -> Vec<Owner> {
        d.income
            .iter()
            .filter(|e| matches!(e.income, FederalIncome::Alimony { .. }))
            .map(|e| e.owner.owned())
            .collect()
    };
    assert_eq!(alimony(&d), vec![Owner::Taxpayer, Owner::Spouse]);
    let mut s = Interview::new(TEST_USER, d);
    let reply = apply(&mut s, Change::Setup(0)).expect("page 1 changes");
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    assert_eq!(alimony(&s.draft), vec![Owner::Taxpayer]);
}

// A document saved with no box checked, opened from the return page: the
// walk never removes a document, and no box becomes the person's answer.
#[test]
fn a_document_saved_with_no_box_checked_is_kept() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let d = s.draft.clone();
    accepted(&send(&mut s, save_checklist(&d, PAGE_REVIEW, Checklist::NONE)));
    let reply = from_review(&mut s, TaxSection::InterestDividends);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let interest = crate::tests::cross::interview::document(
        0,
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(native_int1099(
                "Bank".into(),
                20_000,
            ))),
        },
    );
    accepted(&{
        let request = crate::tests::cross::interview::save_documents(&mut s, vec![interest]);
        send(&mut s, request)
    });
    assert!(
        s.draft.income.iter().any(|e| matches!(e.income, FederalIncome::Interest { .. })),
        "the 1099-INT was pruned: {:?}",
        s.draft.income
    );
    assert_eq!(s.draft.checklist, Checklist::NONE);
    let mut pruned = s.draft.clone();
    assert_eq!(crate::interview::prune(&mut pruned), Default::default());
    assert_eq!(pruned, s.draft);
}
