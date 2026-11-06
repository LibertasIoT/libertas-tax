use super::*;
use crate::tests::cross::interview::{accepted, from_review, navigate};
use crate::tests::support::builders::{add_documents, document_owners_mut, unknown_documents};
use crate::tests::support::client::income_pages::open_document_center;

fn page(session: &mut Interview, section: TaxSection) -> P {
    let Reply::Edit(request) = from_review(session, section) else {
        panic!("expected the {section:?} page")
    };
    request
}

fn view(request: &mut P, person: i64) -> &mut crate::PersonDocuments {
    document_owners_mut(request).unwrap().iter_mut().find(|view| view.person == person).unwrap()
}

fn interest_statement(recipient: &str) -> IncomeDocumentItem {
    use crate::tests::support::builders::native_income::owned;
    owned(
        Owner::Taxpayer,
        0,
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(crate::Form1099IntPaper {
                payer: crate::PrintedParty::from_name("First Bank".into()),
                recipient: crate::PrintedParty::from_name(recipient.into()),
                interest_income: 10_000,
                ..Default::default()
            }),
        },
    )
}

#[test]
fn unnamed_fixture_documents_match_each_existing_owner_and_joint_names_match_both_people() {
    use crate::tests::support::builders::native_income::{native_w2, owned};
    use crate::tests::support::client::income_pages::save_documents;
    let draft = document_recipient_household(FederalStatus::Joint);
    let joint = fixture_document_names(&draft, -3).unwrap();
    assert_eq!(joint.assigned, crate::DocumentRecipient::Unknown);
    assert_eq!(
        crate::document_name_matcher::match_eligible_recipient(
            &draft,
            &joint.recipient.name,
            crate::document_name_matcher::RecipientDomain::Account,
        ),
        Some(-3)
    );
    let owners = [Owner::Taxpayer, Owner::Spouse];
    let documents = owners
        .iter()
        .map(|owner| {
            let mut form = native_w2(1_000_000);
            assert!(form.employee.name.is_empty());
            let person = if *owner == Owner::Spouse { -2 } else { -1 };
            form.employee = fixture_document_names(&draft, person).unwrap().recipient;
            form.employer.name = "Example Employer".into();
            owned(
                *owner,
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) },
            )
        })
        .collect::<Vec<_>>();
    let mut session = Interview::new(TEST_USER, draft);
    let request = save_documents(&mut session, documents);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income.len(), owners.len());
    for (source, owner) in session.draft.income.iter().zip(owners) {
        assert_eq!(source.owner, crate::DocumentOwner::from(owner));
        let names = source.printed_names().unwrap();
        assert_eq!(names.assigned.tax_owner(), Some(owner));
        assert_eq!(names.owner_source, Some(crate::OwnerSource::Matched));
        assert_eq!(names.joint_share, None);
    }
    // Each matched W-2 is listed with its person's name and is not offered to add.
    let ids: Vec<_> = session.draft.income.iter().map(|source| source.id).collect();
    let mut wages = page(&mut session, TaxSection::Documents);
    for (person, id) in [-1, -2].into_iter().zip(ids.iter().copied()) {
        let view = view(&mut wages, person);
        assert_eq!(view.documents.len(), 1);
        assert_eq!(view.documents[0].owner_source, crate::OwnerSource::Matched);
        assert_eq!(view.documents[0].ownership, crate::AddedOwnership::Yours);
        assert_eq!(view.choices.iter().map(|choice| choice.id).collect::<Vec<_>>(), ids);
        assert_eq!(view.available_documents, vec![id]);
        let summary =
            status_text(&view.choices.iter().find(|choice| choice.id == id).unwrap().label);
        assert!(summary.starts_with("W-2 from Example Employer, printed for "));
        assert!(summary.contains("Owner: "));
        assert!(summary.contains("Box 1: $10000.00"));
        assert!(summary.contains("Box 2: $1000.00"));
    }
}

#[test]
fn center_matches_w2_recipient_over_echoed_owner_and_reopens_printed_name() {
    let draft = document_owner_household(&["Unknown Recipient"]);
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, review, .. } = &mut request else { unreachable!() };
    *review = false;
    let row = &mut documents.common.wages.documents[0];
    row.owner = crate::DocumentOwner::Spouses;
    let crate::WageDocument::W2(form) = &mut row.document else { unreachable!() };
    form.employee.name = "ONEIL SMITH, ALEX M.".into();
    let expected_issuer = form.employer.clone();
    let reply = send(&mut session, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let source = &session.draft.income[0];
    assert_eq!(source.owner, crate::DocumentOwner::Yours);
    let names = source.printed_names().unwrap();
    assert_eq!(names.assigned, crate::DocumentRecipient::Taxpayer);
    assert_eq!(names.owner_source, Some(crate::OwnerSource::Matched));
    assert_eq!(names.issuer.as_ref(), Some(&expected_issuer));
    assert!(unknown_documents(&session.draft).is_empty());
    let mut restored = restart(session.draft);
    let request = open_document_center(&mut restored);
    let P::SaveDocuments { documents, .. } = request else { unreachable!() };
    let crate::WageDocument::W2(form) = &documents.common.wages.documents[0].document else {
        unreachable!()
    };
    assert_eq!(form.employee.name, "ONEIL SMITH, ALEX M.");
}

#[test]
fn unmatched_documents_save_without_an_owner_step_and_stay_pending_until_added() {
    let mut session =
        Interview::new(TEST_USER, document_owner_household(&["Unknown One", "Unknown Two"]));
    let request = open_document_center(&mut session);
    let reply = send(&mut session, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let ids: Vec<_> = session.draft.income.iter().map(|source| source.id).collect();
    assert!(session.draft.income.iter().all(|source| {
        source.owner == crate::DocumentOwner::Unknown
            && source.printed_names().unwrap().assigned == crate::DocumentRecipient::Unknown
    }));
    assert_eq!(
        unknown_documents(&session.draft),
        ids.iter().map(|id| (*id, TaxSection::Documents)).collect::<Vec<_>>()
    );
    // The estimate is never computed over a document without an owner.
    let result = estimate(&session.draft);
    assert_eq!(result.state, ResultState::Incomplete);
    assert!(result.lines.is_empty());
    for section in [TaxSection::Mortgage, TaxSection::Education] {
        let context = crate::IssueContext::Section(section);
        // 2026-10-29: correction ownership is typed, not inferred from a
        // message key. Actual paper corrections explicitly name Document Center.
        for key in ["ISSUE_WHOSE_DOCUMENT_IS_THIS", "PAPER_ENTRY_INVALID"] {
            assert_eq!(
                crate::interview::issue_page(&session.draft, &context, Some(key)),
                Some(section),
            );
            assert_eq!(
                crate::interview::issue_page(
                    &session.draft,
                    &context.clone().corrective(TaxSection::Documents),
                    Some(key),
                ),
                Some(TaxSection::Documents),
            );
        }
    }
    assert_eq!(
        result
            .issues
            .iter()
            .filter(|issue| issue.display.key == "ISSUE_WHOSE_DOCUMENT_IS_THIS")
            .count(),
        2
    );
    let request = navigate(&session.draft, TaxSection::Review);
    let Reply::Response(P::Review { remaining_items, .. }) = send(&mut session, request) else {
        panic!("expected review")
    };
    assert!(remaining_items.iter().any(|item| item.section == TaxSection::Documents));
    let mut wages = page(&mut session, TaxSection::Documents);
    let you = view(&mut wages, -1);
    assert!(you.documents.is_empty());
    assert_eq!(you.choices.iter().map(|choice| choice.id).collect::<Vec<_>>(), ids);
    assert_eq!(you.available_documents, ids);
    // Every refusal is atomic and names the row's field.
    for (case, field) in [
        ("unknown document", "document"),
        ("added twice", "document"),
        ("joint on a single return", "ownership"),
    ] {
        let mut changed = wages.clone();
        let you = view(&mut changed, -1);
        let row = |document, ownership| crate::SectionDocument::added(&wages, document, ownership);
        you.documents = match case {
            "unknown document" => vec![row(999, crate::AddedOwnership::Yours)],
            "added twice" => vec![
                row(ids[0], crate::AddedOwnership::Yours),
                row(ids[0], crate::AddedOwnership::Yours),
            ],
            _ => vec![row(ids[0], crate::AddedOwnership::Joint { share: 50 })],
        };
        let reply = exchange(&mut session, changed, case);
        let Reply::Response(P::Problem { field: actual, .. }) = reply else {
            panic!("{case}: expected refusal, got {reply:?}")
        };
        let row = if case == "added twice" { 1 } else { 0 };
        let expected = alloc::format!("[document_owners/0/documents/{row}/{field}]");
        assert_eq!(actual.as_deref(), Some(expected.as_str()), "{case}");
    }
    accepted(&add_documents(
        &mut session,
        TaxSection::Wages,
        -1,
        &ids,
        crate::AddedOwnership::Yours,
    ));
    for source in &session.draft.income {
        assert_eq!(source.owner, crate::DocumentOwner::Yours);
        let names = source.printed_names().unwrap();
        assert_eq!(names.assigned, crate::DocumentRecipient::Taxpayer);
        assert_eq!(names.owner_source, Some(crate::OwnerSource::Added));
    }
    assert!(unknown_documents(&session.draft).is_empty());
    assert!(
        !estimate(&session.draft)
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_WHOSE_DOCUMENT_IS_THIS")
    );
    // Removing the added row returns that document to Unknown.
    let mut wages = page(&mut session, TaxSection::Documents);
    let you = view(&mut wages, -1);
    assert_eq!(you.documents.len(), 2);
    // Output-only catalogs are empty on Submit; Save rebuilds every reference on reopen.
    you.choices.clear();
    you.available_documents.clear();
    accepted(&send(&mut session, wages));
    let mut session = restart(session.draft);
    let mut wages = page(&mut session, TaxSection::Documents);
    let you = view(&mut wages, -1);
    assert_eq!(you.choices.iter().map(|choice| choice.id).collect::<Vec<_>>(), ids);
    assert!(you.choices.iter().all(|choice| status_text(&choice.label).contains("W-2")));
    you.documents.retain(|row| row.document != ids[1]);
    accepted(&send(&mut session, wages));
    assert_eq!(unknown_documents(&session.draft), vec![(ids[1], TaxSection::Documents)]);
}

// D-2026-10-30-06: a printed name is stored as printed; a blank one stays
// blank, and the person assigns the document's owner separately.
#[test]
fn a_blank_printed_name_stays_blank_and_its_owner_is_added_separately() {
    use crate::tests::support::builders::named_document;
    use crate::tests::support::builders::native_income::native_w2;
    let mut draft = document_recipient_household(FederalStatus::Single);
    draft.income.clear();
    let mut session = Interview::new(TEST_USER, draft);
    let mut form = wage_document(native_w2(1_000_000));
    assert!(form.employee.name.is_empty());
    form.employer.name = "Example Employer".into();
    let source = IncomeSource::Wage { value: crate::WageDocument::W2(form) };
    // The fixture row keeps the paper as printed.
    let row = named_document(&session.draft, -1, 0, source.clone());
    assert_eq!(row.document, source);
    let IncomeSource::Wage { value: document } = row.document else { unreachable!() };
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, allowed_community, .. } = &mut request else {
        unreachable!()
    };
    documents.common.wages.documents = vec![crate::WageDocumentRow {
        id: row.id,
        owner: row.owner,
        document: document.clone(),
        community: row.community,
        preview: row.preview,
        refund_management: row.refund_management,
        refund_management_needed: row.refund_management_needed,
        investment_caps: row.investment_caps,
        allowed_community: allowed_community.clone(),
    }];
    let reply = send(&mut session, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    fn blank(session: &Interview) -> crate::PrintedDocumentNames {
        let names = session.draft.income[0].printed_names().unwrap().clone();
        assert!(names.recipient.name.is_empty());
        names
    }
    assert_eq!(session.draft.income[0].owner, crate::DocumentOwner::Unknown);
    assert_eq!(blank(&session).assigned, crate::DocumentRecipient::Unknown);
    let id = session.draft.income[0].id;
    accepted(&add_documents(
        &mut session,
        TaxSection::Wages,
        -1,
        &[id],
        crate::AddedOwnership::Yours,
    ));
    assert_eq!(session.draft.income[0].owner, crate::DocumentOwner::Yours);
    let names = blank(&session);
    assert_eq!(names.assigned, crate::DocumentRecipient::Taxpayer);
    assert_eq!(names.owner_source, Some(crate::OwnerSource::Added));
    // Reopened and saved again, the paper still prints no name and keeps its owner.
    let mut session = restart(session.draft);
    let request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &request else { unreachable!() };
    assert_eq!(documents.common.wages.documents[0].document, document);
    assert!(!matches!(send(&mut session, request), Reply::Response(P::Problem { .. })));
    assert_eq!(session.draft.income[0].owner, crate::DocumentOwner::Yours);
    assert_eq!(blank(&session).owner_source, Some(crate::OwnerSource::Added));
}

#[test]
fn added_owner_survives_resave_but_a_changed_printed_name_is_unknown_again() {
    let mut session = Interview::new(TEST_USER, document_owner_household(&["Unknown Recipient"]));
    let request = open_document_center(&mut session);
    accepted(&send(&mut session, request));
    let id = session.draft.income[0].id;
    accepted(&add_documents(
        &mut session,
        TaxSection::Wages,
        -1,
        &[id],
        crate::AddedOwnership::Yours,
    ));
    let request = open_document_center(&mut session);
    assert!(!matches!(send(&mut session, request), Reply::Response(P::Problem { .. })));
    let names = session.draft.income[0].printed_names().unwrap();
    assert_eq!(names.assigned, crate::DocumentRecipient::Taxpayer);
    assert_eq!(names.owner_source, Some(crate::OwnerSource::Added));
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, review, .. } = &mut request else { unreachable!() };
    *review = false;
    let crate::WageDocument::W2(form) = &mut documents.common.wages.documents[0].document else {
        unreachable!()
    };
    form.employee.name = "Another Unknown Recipient".into();
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income[0].owner, crate::DocumentOwner::Unknown);
    let request = navigate(&session.draft, TaxSection::Review);
    let Reply::Response(P::Review { remaining_items, .. }) = send(&mut session, request) else {
        panic!("expected review")
    };
    assert!(remaining_items.iter().any(|item| item.section == TaxSection::Documents));
}

#[test]
fn a_name_matched_document_is_listed_locked_and_never_written_by_the_view() {
    use crate::tests::support::builders::native_income::{native_w2, owned};
    use crate::tests::support::client::income_pages::save_documents;
    let draft = document_recipient_household(FederalStatus::Single);
    let mut form = native_w2(1_000_000);
    form.employee = fixture_document_names(&draft, -1).unwrap().recipient;
    let w2 = owned(
        Owner::Taxpayer,
        0,
        IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) },
    );
    let mut session = Interview::new(TEST_USER, draft);
    let request = save_documents(&mut session, vec![w2]);
    accepted(&send(&mut session, request));
    let id = session.draft.income[0].id;
    let before = session.draft.income.clone();
    // The matched row is locked: it is not removable, and an echo that drops
    // it or changes its answer changes nothing.
    let mut wages = page(&mut session, TaxSection::Documents);
    let you = view(&mut wages, -1);
    assert_eq!(you.documents.len(), 1);
    assert_eq!(you.documents[0].document, id);
    assert_eq!(you.documents[0].owner_source, crate::OwnerSource::Matched);
    you.documents.clear();
    accepted(&send(&mut session, wages));
    assert_eq!(session.draft.income, before);
    let mut wages = page(&mut session, TaxSection::Documents);
    view(&mut wages, -1).documents[0].owner_source = crate::OwnerSource::Added;
    accepted(&send(&mut session, wages));
    assert_eq!(session.draft.income, before);
}

#[test]
fn unmatched_after_death_statement_checks_the_added_owner_before_accepting() {
    let (draft, mut document) = after_death_owner_document();
    document.owner = crate::DocumentOwner::Spouses;
    set_fixture_recipient(
        &mut document.document,
        &fixture_document_names(&draft, -2).unwrap().recipient,
    );
    let mut session = Interview::new(TEST_USER, draft);
    accepted(&crate::tests::support::client::income_pages::save_source_fixtures(
        &mut session,
        vec![document],
    ));
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, review, .. } = &mut request else { unreachable!() };
    *review = false;
    let crate::RetirementDocument::R1099(form) =
        &mut documents.common.retirement.documents[0].document
    else {
        unreachable!()
    };
    form.recipient.name = "Unmatched Recipient".into();
    accepted(&send(&mut session, request));
    let id = session
        .draft
        .income
        .iter()
        .find(|source| matches!(source.income, FederalIncome::Retirement { .. }))
        .unwrap()
        .id;
    assert_eq!(unknown_documents(&session.draft), vec![(id, TaxSection::Documents)]);
    let before = session.draft.clone();
    let request = crate::tests::support::builders::added_documents_request(
        &mut session,
        TaxSection::Documents,
        -1,
        &[id],
        crate::AddedOwnership::Yours,
    );
    let P::SaveDocuments { document_owners, .. } = &request else { unreachable!() };
    let row = document_owners[0].documents.iter().position(|row| row.document == id).unwrap();
    let Reply::Response(P::Problem { field, .. }) = send(&mut session, request) else {
        panic!("the after-death statement requires its eligible owner")
    };
    assert_eq!(
        field,
        crate::validation::ValidationIssue::new("ISSUE_WHOSE_DOCUMENT_IS_THIS")
            .at_field("ownership")
            .in_list("documents", row)
            .in_list("document_owners", 0)
            .field
            .map(crate::validation::FieldPath::square)
    );
    assert_eq!(session.draft, before);
    accepted(&add_documents(
        &mut session,
        TaxSection::Documents,
        -2,
        &[id],
        crate::AddedOwnership::Yours,
    ));
    let source = session.draft.income.iter().find(|source| source.id == id).unwrap();
    assert_eq!(source.owner, crate::DocumentOwner::Spouses);
    assert_eq!(source.printed_names().unwrap().assigned, crate::DocumentRecipient::Spouse);
    let FederalIncome::Retirement { data } = &source.income else { unreachable!() };
    assert!(FederalRetirement::from(data).after_death.is_some());
}

#[test]
fn an_unavailable_financial_owner_is_offered_only_to_eligible_people() {
    let draft = dependent_recipient_with_unavailable_financial_owner();
    let child = draft.dependents[0].id;
    let original = draft.income.last().unwrap().clone();
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { review, .. } = &mut request else { unreachable!() };
    *review = false;
    accepted(&send(&mut session, request));
    let source = session.draft.income.iter().find(|source| source.id == original.id).unwrap();
    assert_eq!(source.owner, crate::DocumentOwner::Unknown);
    assert_eq!(source.income, original.income);
    let (_, section) =
        *unknown_documents(&session.draft).iter().find(|(id, _)| *id == original.id).unwrap();
    let mut topic = page(&mut session, TaxSection::Documents);
    let owners = document_owners_mut(&mut topic).unwrap();
    assert!(!owners.iter().any(|view| view.person == -2));
    let student = owners.iter().find(|view| view.person == child).unwrap();
    assert!(student.choices.iter().any(|choice| choice.id == original.id));
    assert!(student.available_documents.contains(&original.id));
    accepted(&add_documents(
        &mut session,
        section,
        -1,
        &[original.id],
        crate::AddedOwnership::Yours,
    ));
    let source = session.draft.income.iter().find(|source| source.id == original.id).unwrap();
    assert_eq!(source.owner, crate::DocumentOwner::Yours);
    assert_eq!(source.income, original.income);
    let names = source.printed_names().unwrap();
    assert_eq!(names.recipient.name, "Unmatched Recipient");
    assert_eq!(names.assigned, crate::DocumentRecipient::Taxpayer);
}

#[test]
fn community_mfs_offers_and_accepts_the_spouse_without_joint_spouse_facts() {
    let mut draft = document_owner_household(&["Unmatched spouse recipient"]);
    draft.income[0].community = CommunityShare::Separate;
    let setup = complete(FederalStatus::Separate).setup.unwrap();
    draft.setup = Some(setup);
    draft.setup.as_mut().unwrap().community_property =
        CommunityProperty::Yes { state: CommunityState::Texas, rule: CommunityRule::Halves };
    assert!(draft.people.as_ref().unwrap().spouse.is_none());
    let mut session = Interview::new(TEST_USER, draft);
    let id = session.draft.income[0].id;
    let mut wages = page(&mut session, TaxSection::Documents);
    let owners = document_owners_mut(&mut wages).unwrap();
    assert_eq!(owners.iter().map(|view| view.person).collect::<Vec<_>>(), vec![-1, -2]);
    accepted(&add_documents(
        &mut session,
        TaxSection::Wages,
        -2,
        &[id],
        crate::AddedOwnership::Yours,
    ));
    assert_eq!(session.draft.income[0].owner, crate::DocumentOwner::Spouses);
    assert_eq!(
        session.draft.income[0].printed_names().unwrap().assigned,
        crate::DocumentRecipient::Spouse
    );
    assert!(session.draft.people.as_ref().unwrap().spouse.is_none());
    let expected = session.draft.clone();
    let request = open_document_center(&mut session);
    assert!(!matches!(send(&mut session, request), Reply::Response(P::Problem { .. })));
    assert_eq!(session.draft, expected);
}

#[test]
fn a_joint_document_is_stored_once_and_shown_in_both_spouses_sections_with_each_share() {
    use crate::tests::support::client::income_pages::save_documents;
    let draft = document_recipient_household(FederalStatus::Separate);
    let mut session = Interview::new(TEST_USER, draft);
    let request =
        save_documents(&mut session, vec![interest_statement("Unmatched Account Holder")]);
    accepted(&send(&mut session, request));
    let [(id, section)] = unknown_documents(&session.draft)[..] else {
        panic!("one unknown interest statement")
    };
    assert_eq!(section, TaxSection::Documents);
    // The spouse adds it as joint with a 60% share: the taxpayer's share is 40%.
    accepted(&add_documents(
        &mut session,
        section,
        -2,
        &[id],
        crate::AddedOwnership::Joint { share: 60 },
    ));
    assert_eq!(session.draft.income.iter().filter(|entry| entry.id == id).count(), 1);
    let entry = session.draft.income.iter().find(|entry| entry.id == id).unwrap();
    let names = entry.printed_names().unwrap();
    assert_eq!(names.assigned, crate::DocumentRecipient::Joint);
    assert_eq!(names.owner_source, Some(crate::OwnerSource::Added));
    assert_eq!(names.joint_share, Some(40));
    let mut topic = page(&mut session, TaxSection::Documents);
    for (person, share) in [(-1, 40), (-2, 60)] {
        let view = view(&mut topic, person);
        assert_eq!(view.documents.len(), if person == -1 { 2 } else { 1 });
        let row = view.documents.iter().find(|row| row.document == id).unwrap();
        assert_eq!(row.owner_source, crate::OwnerSource::Added);
        assert_eq!(row.ownership, crate::AddedOwnership::Joint { share });
        // The central total also includes the taxpayer's existing $60,000 W-2.
        let other_income = if person == -1 { 6_000_000 } else { 0 };
        assert_eq!(
            view.income_share.map(|total| total - other_income),
            Some(i64::from(share) * 100)
        );
        assert_eq!(row.joint_share_needed, Some(true));
    }
    // Removing it from either section returns it to Unknown.
    view(&mut topic, -1).documents.retain(|row| row.document != id);
    accepted(&send(&mut session, topic));
    assert_eq!(unknown_documents(&session.draft), vec![(id, section)]);
}

#[test]
fn a_both_names_document_is_matched_joint_in_both_read_only_lists() {
    use crate::tests::support::client::income_pages::save_documents;
    let draft = document_recipient_household(FederalStatus::Joint);
    let joint = fixture_document_names(&draft, -3).unwrap();
    let mut session = Interview::new(TEST_USER, draft);
    let request = save_documents(&mut session, vec![interest_statement(&joint.recipient.name)]);
    accepted(&send(&mut session, request));
    assert!(unknown_documents(&session.draft).is_empty());
    let entry = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Interest { .. }))
        .unwrap();
    let id = entry.id;
    let names = entry.printed_names().unwrap();
    assert_eq!(names.assigned, crate::DocumentRecipient::Joint);
    assert_eq!(names.owner_source, Some(crate::OwnerSource::Matched));
    assert_eq!(names.joint_share, Some(50));
    let mut topic = page(&mut session, TaxSection::Documents);
    for person in [-1, -2] {
        let view = view(&mut topic, person);
        assert_eq!(view.documents.len(), if person == -1 { 2 } else { 1 });
        let row = view.documents.iter().find(|row| row.document == id).unwrap();
        assert_eq!(row.owner_source, crate::OwnerSource::Matched);
        assert_eq!(row.ownership, crate::AddedOwnership::Joint { share: 50 });
        // The central total also includes the taxpayer's existing $100,000 W-2.
        let other_income = if person == -1 { 10_000_000 } else { 0 };
        assert_eq!(view.income_share.map(|total| total - other_income), Some(5_000));
        assert_eq!(row.joint_share_needed, Some(false));
    }
}

#[test]
fn joint_section_totals_round_the_aggregate_once_and_conserve_cents() {
    use crate::tests::support::builders::named_document;
    use crate::tests::support::builders::native_income::native_int1099;
    use crate::tests::support::client::income_pages::save_documents;
    for (amounts, expected) in [(vec![1], [1, 0]), (vec![1, 1], [1, 1]), (vec![1, 2], [2, 1])] {
        let draft = document_recipient_household(FederalStatus::Joint);
        let papers = amounts
            .into_iter()
            .map(|amount| {
                // Printed for both spouses: the paper names them.
                let mut form =
                    interest_document(native_int1099("Synthetic joint bank".into(), amount));
                form.recipient = fixture_document_names(&draft, -3).unwrap().recipient;
                named_document(
                    &draft,
                    -3,
                    0,
                    IncomeSource::InterestDividend {
                        value: crate::InterestDividendDocument::Int1099(form),
                    },
                )
            })
            .collect();
        let mut session = Interview::new(TEST_USER, draft);
        let request = save_documents(&mut session, papers);
        accepted(&send(&mut session, request));
        let mut topic = page(&mut session, TaxSection::Documents);
        // Preserve the interest-cent check within the complete document total.
        assert_eq!(
            view(&mut topic, -1).income_share.map(|total| total - 10_000_000),
            Some(expected[0])
        );
        assert_eq!(view(&mut topic, -2).income_share, Some(expected[1]));
    }
}

#[test]
fn a_matched_joint_share_corrects_for_form8379_without_changing_ownership_or_unknowns() {
    use crate::tests::support::client::income_pages::save_documents;
    let mut draft = document_recipient_household(FederalStatus::Joint);
    draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    draft.injured_spouse = Some(allocation());
    let joint = fixture_document_names(&draft, -3).unwrap();
    let mut session = Interview::new(TEST_USER, draft);
    let request = save_documents(
        &mut session,
        vec![
            interest_statement(&joint.recipient.name),
            interest_statement("Unmatched Account Holder"),
        ],
    );
    accepted(&send(&mut session, request));
    let unknown = unknown_documents(&session.draft);
    assert_eq!(unknown.len(), 1);
    let id = session
        .draft
        .income
        .iter()
        .find(|entry| {
            entry
                .printed_names()
                .is_some_and(|names| names.assigned == crate::DocumentRecipient::Joint)
        })
        .unwrap()
        .id;
    // Correct from either person's section, including the valid endpoints.
    for (person, share, taxpayer) in [(-2, 60, 40), (-1, 0, 0), (-1, 100, 100)] {
        let mut topic = page(&mut session, TaxSection::Documents);
        let row =
            view(&mut topic, person).documents.iter_mut().find(|row| row.document == id).unwrap();
        row.ownership = crate::AddedOwnership::Joint { share };
        accepted(&send(&mut session, topic));
        session = restart(session.draft);
        let names = session
            .draft
            .income
            .iter()
            .find(|entry| entry.id == id)
            .unwrap()
            .printed_names()
            .unwrap();
        assert_eq!(
            (names.assigned, names.owner_source, names.joint_share),
            (crate::DocumentRecipient::Joint, Some(crate::OwnerSource::Matched), Some(taxpayer)),
        );
        let mut reopened = page(&mut session, TaxSection::Documents);
        for (person, share) in [(-1, taxpayer), (-2, 100 - taxpayer)] {
            let row = view(&mut reopened, person)
                .documents
                .iter()
                .find(|row| row.document == id)
                .unwrap();
            assert_eq!(row.ownership, crate::AddedOwnership::Joint { share });
            assert_eq!(
                row.allowed_ownership,
                vec![libertas_macros::variant_index!(crate::AddedOwnership::Joint) as i32],
            );
        }
    }
    // Resaving unchanged printed names retains the corrected matched share.
    let expected = session.draft.income.clone();
    let request = open_document_center(&mut session);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income, expected);
    assert_eq!(unknown_documents(&session.draft), unknown);
    assert_eq!(estimate(&session.draft).state, ResultState::Incomplete);
}

#[test]
fn education_unknown_assignment_creates_annual_answers_only_after_person_adds_it() {
    use crate::tests::cross::interview::student;
    let mut draft = complete(FederalStatus::Single);
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let mut statement = student(-1, 300_000);
    statement.form.student.name = "Unmatched Learner".into();
    documents.common.education.students.push(statement.clone());
    accepted(&send(&mut session, request));
    let credits = session.draft.credits.as_ref().unwrap();
    let source = &credits.education[0];
    let id = source.id;
    statement.id = id;
    assert_eq!(source.student, 0);
    assert_eq!(source.assigned, crate::DocumentRecipient::Unknown);
    assert_eq!(source.owner_source, None);
    assert_eq!(source.form, statement.form);
    assert!(credits.students.is_empty());
    assert!(credits.education_inputs().is_empty());
    assert!(credits.education_pending().is_empty());
    assert!(session.draft.valid_stored());
    assert_eq!(unknown_documents(&session.draft), vec![(id, TaxSection::Documents)]);
    let result = estimate(&session.draft);
    assert_eq!(result.state, ResultState::Incomplete);
    assert!(result.lines.is_empty());
    let mut session = restart(session.draft);
    let education = page(&mut session, TaxSection::Education);
    let P::SaveEducation { value, .. } = &education else { unreachable!() };
    assert_eq!(value.documents[0].student, None);
    let income_ids = session.draft.income.iter().map(|source| source.id).collect::<Vec<_>>();
    let mut documents = page(&mut session, TaxSection::Documents);
    let you = view(&mut documents, -1);
    assert_eq!(you.documents.iter().map(|row| row.document).collect::<Vec<_>>(), income_ids);
    let mut all_ids = income_ids;
    all_ids.push(id);
    assert_eq!(you.choices.iter().map(|choice| choice.id).collect::<Vec<_>>(), all_ids);
    accepted(&add_documents(
        &mut session,
        TaxSection::Education,
        -1,
        &[id],
        crate::AddedOwnership::Yours,
    ));
    let credits = session.draft.credits.as_ref().unwrap();
    assert_eq!(credits.education[0].student, -1);
    assert_eq!(credits.education[0].owner_source, Some(crate::OwnerSource::Added));
    assert_eq!(credits.students.len(), 1);
    assert_eq!(credits.students[0].method, None);
    assert_eq!(credits.students[0].more.american_opportunity_used_up, None);
    assert_eq!(
        credits.education_pending(),
        vec![(-1, "EDUCATION_METHOD_NEEDED"), (-1, "EDUCATION_AOTC_HISTORY_NEEDED"),]
    );
    assert!(credits.education_inputs().is_empty());
    let mut education = page(&mut session, TaxSection::Education);
    let before = session.draft.clone();
    rejected_with(send(&mut session, education.clone()), "EDUCATION_METHOD_NEEDED");
    assert_eq!(session.draft, before);
    let P::SaveEducation { value, .. } = &mut education else { unreachable!() };
    assert_eq!(value.documents[0].student, Some(-1));
    value.students[0].method = Some(EducationChoice::LifetimeLearning);
    rejected_with(send(&mut session, education.clone()), "EDUCATION_AOTC_HISTORY_NEEDED");
    assert_eq!(session.draft, before);
    let P::SaveEducation { value, .. } = &mut education else { unreachable!() };
    value.students[0].more.american_opportunity_used_up = Some(false);
    accepted(&send(&mut session, education));
    let inputs = session.draft.credits.as_ref().unwrap().education_inputs();
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].expenses.tuition, 300_000);
    let annual = session.draft.credits.as_ref().unwrap().students.clone();
    // Resaving the unchanged paper preserves the accepted addition and answers.
    let center = open_document_center(&mut session);
    accepted(&send(&mut session, center));
    assert_eq!(session.draft.credits.as_ref().unwrap().students, annual);
    let mut documents = page(&mut session, TaxSection::Documents);
    view(&mut documents, -1).documents.clear();
    accepted(&send(&mut session, documents));
    let credits = session.draft.credits.as_ref().unwrap();
    assert_eq!(credits.education[0].student, 0);
    assert_eq!(credits.education[0].assigned, crate::DocumentRecipient::Unknown);
    assert_eq!(credits.education[0].form, statement.form);
    assert!(credits.students.is_empty());
    assert!(credits.education_inputs().is_empty());
    assert_eq!(unknown_documents(&session.draft), vec![(id, TaxSection::Documents)]);
}

// AI Import shape of the Smith family's Form 8997 (round 3, batch 6): Part IV
// column (a) prints the QOF's EIN; the form has no issuer, so its fund names it.
#[test]
fn form_8997_keeps_its_qof_ein_and_is_named_by_its_fund() {
    use crate::tests::support::builders::native_income::owned;
    use crate::tests::support::client::income_pages::save_documents;
    let draft = document_recipient_household(FederalStatus::Single);
    let mut recipient = fixture_document_names(&draft, -1).unwrap().recipient;
    recipient.address.clear();
    let paper = crate::QofInvestmentPaper {
        calendar_year: 0,
        source_recipient: recipient,
        fund_ein: "00-0000310".into(),
        invested_on: 20210601,
        fund: "100 units Example Meadow Opportunity Fund".into(),
        gain_code: crate::QofGainCode::None,
        short_term: Some(0),
        long_term: Some(400_000),
    };
    let document = IncomeSource::InvestmentSale {
        value: crate::InvestmentSaleDocument::OpportunityFund(paper.clone()),
    };
    let mut session = Interview::new(TEST_USER, draft);
    let request = save_documents(&mut session, vec![owned(Owner::Taxpayer, 0, document)]);
    accepted(&send(&mut session, request));
    let source = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::OpportunityFund { .. }))
        .unwrap();
    let FederalIncome::OpportunityFund { data } = &source.income else { unreachable!() };
    assert_eq!(data.fund_ein, "00-0000310");
    let issuer = source.printed_names().unwrap().issuer.clone().unwrap();
    assert_eq!(issuer.name, paper.fund);
    assert_eq!(issuer.federal_id, "00-0000310");
    let id = source.id;
    let mut documents = page(&mut session, TaxSection::Documents);
    let view = view(&mut documents, -1);
    let label = status_text(&view.choices.iter().find(|choice| choice.id == id).unwrap().label);
    assert!(
        label.contains(" from 100 units Example Meadow Opportunity Fund, printed for "),
        "{label}"
    );
    assert!(!label.contains("Treasury"), "{label}");
    let mut cold = restart(session.draft);
    let reopened = open_document_center(&mut cold);
    let P::SaveDocuments { documents, .. } = &reopened else { unreachable!() };
    let row = documents.common.investment_sales.documents.iter().find(|row| row.id == id).unwrap();
    assert_eq!(row.document, crate::InvestmentSaleDocument::OpportunityFund(paper));
}
