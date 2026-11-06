use crate::tests::*;

// Known fixture input: the printed identity comes from the actual household,
// while association stays unanswered for the production matcher to determine.
pub(in crate::tests) fn fixture_document_names(
    draft: &FederalDraft,
    person: i64,
) -> Option<crate::PrintedDocumentNames> {
    if person == -3 {
        let taxpayer = fixture_document_names(draft, -1)?;
        let spouse = fixture_document_names(draft, -2)?;
        return Some(crate::PrintedDocumentNames {
            recipient: crate::PrintedParty::from_name(alloc::format!(
                "{} & {}",
                taxpayer.recipient.name,
                spouse.recipient.name
            )),
            issuer: None,
            assigned: crate::DocumentRecipient::Unknown,
            ..Default::default()
        });
    }
    let (first, last) = if person > 0 {
        let dependent = draft.dependents.iter().find(|dependent| dependent.id == person)?;
        (&dependent.first_name, &dependent.last_name)
    } else {
        let owner = match person {
            -1 => Owner::Taxpayer,
            -2 => Owner::Spouse,
            _ => return None,
        };
        let person = draft.people.as_ref()?.person(owner)?;
        (&person.first_name, &person.last_name)
    };
    Some(crate::PrintedDocumentNames {
        recipient: crate::PrintedParty::from_name(alloc::format!("{first} {last}")),
        issuer: None,
        assigned: crate::DocumentRecipient::Unknown,
        ..Default::default()
    })
}

// Supply only the fixture's printed recipient; preserve every other paper party.
pub(in crate::tests) fn set_fixture_recipient(
    source: &mut IncomeSource,
    recipient: &crate::PrintedParty,
) {
    set_fixture_recipient_at(source, recipient, 0, 0, 1);
}

fn set_fixture_recipient_at(
    source: &mut IncomeSource,
    recipient: &crate::PrintedParty,
    id: i64,
    ordinal: usize,
    total: usize,
) {
    if let Some(mut names) = source.printed_names() {
        let before = source.clone();
        names.recipient.name = recipient.name.clone();
        source.set_printed_names(&names);
        rebind_paper_answers_at(&before, source, id, ordinal, total);
    }
}

// One person's fixture document row. Its paper stays as printed: a blank
// printed name stays blank (D-2026-10-30-06); ownership is the row's own.
pub(in crate::tests) fn named_document(
    _draft: &FederalDraft,
    person: i64,
    id: i64,
    source: IncomeSource,
) -> IncomeDocumentItem {
    let owner = if person == -2 { Owner::Spouse } else { Owner::Taxpayer };
    crate::tests::support::builders::native_income::owned(owner, id, source)
}

// Physical source fixtures acquire their household's printed identity on Save.
// Reopen expectations include that identity independently of the actual result.
pub(in crate::tests) fn expected_named_source(
    draft: &FederalDraft,
    owner: Owner,
    mut source: IncomeSource,
) -> IncomeSource {
    let person = if owner == Owner::Spouse { -2 } else { -1 };
    if let Some(mut names) = source.printed_names()
        && names.recipient.name.is_empty()
        && let Some(fixture) = fixture_document_names(draft, person)
    {
        let before = source.clone();
        names.recipient.name = fixture.recipient.name;
        source.set_printed_names(&names);
        rebind_paper_answers(&before, &source);
    }
    source
}

pub(in crate::tests) fn expected_named_sources(
    draft: &FederalDraft,
    owner: Owner,
    sources: Vec<IncomeSource>,
) -> Vec<IncomeSource> {
    sources.into_iter().map(|source| expected_named_source(draft, owner, source)).collect()
}

pub(in crate::tests) fn fixture_owned_rows(
    draft: &FederalDraft,
    mut rows: Vec<IncomeDocumentItem>,
) -> Vec<IncomeDocumentItem> {
    let contexts: Vec<_> = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let equal = |other: &&IncomeDocumentItem| {
                other.document == row.document && (row.id != 0 || other.id == 0)
            };
            (rows[..index].iter().filter(equal).count(), rows.iter().filter(equal).count())
        })
        .collect();
    for (row, (ordinal, total)) in rows.iter_mut().zip(contexts) {
        let key = if row.owner == crate::DocumentOwner::Spouses { -2 } else { -1 };
        if let Some(names) = fixture_document_names(draft, key) {
            set_fixture_recipient_at(&mut row.document, &names.recipient, row.id, ordinal, total);
        }
    }
    rows
}

// Opt-in source-input construction for fixtures whose row owners are explicit
// controls. Raw transport deliberately never calls this: echoed owners alone
// are untrusted, and forgery/unknown-name tests must retain their exact inputs.
pub(in crate::tests) fn fixture_owner_inputs(draft: &FederalDraft, request: P) -> P {
    fixture_owner_inputs_inner(draft, request, false, false)
}

const MFS_PRINTED_RECIPIENT: &str = "Unmatched MFS spouse recipient";

pub(in crate::tests) fn fixture_blank_owner_inputs(draft: &FederalDraft, request: P) -> P {
    fixture_owner_inputs_inner(draft, request, false, true)
}

fn fixture_owner_inputs_inner(
    draft: &FederalDraft,
    mut request: P,
    manual_mfs: bool,
    blank_only: bool,
) -> P {
    let P::SaveDocuments { documents, .. } = &mut request else { return request };
    let printed_names = |key| {
        fixture_document_names(draft, key).or_else(|| {
            (manual_mfs && key == -2).then(|| crate::PrintedDocumentNames {
                recipient: crate::PrintedParty::from_name(MFS_PRINTED_RECIPIENT.into()),
                issuer: None,
                assigned: crate::DocumentRecipient::Unknown,
                ..Default::default()
            })
        })
    };
    macro_rules! name_rows {
        ($rows:expr, $variant:ident $(, $extra:ident)*) => {
            let contexts: Vec<_> = $rows
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    let ordinal = $rows[..index]
                        .iter()
                        .filter(|other| {
                            other.document == row.document && (row.id != 0 || other.id == 0)
                        })
                        .count();
                    let total = $rows
                        .iter()
                        .filter(|other| {
                            other.document == row.document && (row.id != 0 || other.id == 0)
                        })
                        .count();
                    (ordinal, total)
                })
                .collect();
            for (row, (ordinal, total)) in $rows.iter_mut().zip(contexts) {
                let key = if row.owner == crate::DocumentOwner::Spouses { -2 } else { -1 };
                let Some(names) = printed_names(key) else { continue };
                let mut source =
                    IncomeSource::$variant { value: row.document.clone() $(, $extra: row.$extra.clone())* };
                if !blank_only
                    || source.printed_names().is_some_and(|names| names.recipient.name.is_empty())
                {
                    set_fixture_recipient_at(&mut source, &names.recipient, row.id, ordinal, total);
                }
                let IncomeSource::$variant { value, .. } = source else { unreachable!() };
                row.document = value;
            }
        };
    }
    name_rows!(documents.common.wages.documents, Wage);
    name_rows!(documents.common.interest_dividends.documents, InterestDividend);
    name_rows!(documents.common.investment_sales.documents, InvestmentSale);
    name_rows!(documents.common.retirement.documents, Retirement);
    name_rows!(documents.common.self_employment.documents, SelfEmployment);
    name_rows!(
        documents.common.government_payments.documents,
        GovernmentPayment,
        refund_state,
        refund_locality
    );
    name_rows!(documents.common.k1.documents, K1);
    name_rows!(documents.other.other_income.documents, OtherIncome);
    name_rows!(documents.other.canceled_debt.documents, CanceledDebt);
    for row in &mut documents.common.gambling.documents {
        let key = if row.owner == crate::DocumentOwner::Spouses { -2 } else { -1 };
        if let Some(names) = printed_names(key)
            && (!blank_only || row.document.winner.name.is_empty())
        {
            row.document.winner.name = names.recipient.name;
        }
    }
    let contexts: Vec<_> = documents
        .other
        .farming
        .documents
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let equal = |other: &&crate::FarmingDocumentRow| {
                other.document == row.document && (row.id != 0 || other.id == 0)
            };
            (
                documents.other.farming.documents[..index].iter().filter(equal).count(),
                documents.other.farming.documents.iter().filter(equal).count(),
            )
        })
        .collect();
    for (row, (ordinal, total)) in documents.other.farming.documents.iter_mut().zip(contexts) {
        let key = if row.owner == crate::DocumentOwner::Spouses { -2 } else { -1 };
        if let Some(names) = printed_names(key) {
            let mut source = IncomeSource::Farming(row.document.clone());
            if !blank_only
                || source.printed_names().is_some_and(|names| names.recipient.name.is_empty())
            {
                set_fixture_recipient_at(&mut source, &names.recipient, row.id, ordinal, total);
            }
            let IncomeSource::Farming(value) = source else { unreachable!() };
            row.document = value;
        }
    }
    request
}

// Explicit manual fixture control for reportable MFS spouse sources. There are
// no joint-filer personal facts on an MFS return, so the printed test source is
// deliberately unmatched and the actual owner handler must accept the answer.
pub(in crate::tests) fn save_mfs_owner_fixture(session: &mut Interview, request: P) -> Reply {
    use crate::tests::support::client::income_pages::{
        k1_fixture_answer_request, page_documents, save_information_return_answers,
    };
    let supplied = page_documents(&request);
    let (mut reply, unmatched) = save_mfs_source_owner_fixture(session, request);
    if let Some(request) = k1_fixture_answer_request(session, &supplied) {
        reply = send(session, request);
        crate::tests::cross::interview::accepted(&reply);
    }
    save_information_return_answers(session, &supplied);
    // No owner step: each unmatched MFS spouse document is a pending item
    // until the spouse's section on its own page adds it.
    let mut sections: Vec<TaxSection> = unmatched.iter().map(|(_, section)| *section).collect();
    sections.sort_by_key(|section| *section as i32);
    sections.dedup();
    for section in sections {
        let ids: Vec<_> =
            unmatched.iter().filter(|(_, s)| *s == section).map(|(id, _)| *id).collect();
        reply = add_documents(session, section, -2, &ids, crate::AddedOwnership::Yours);
        crate::tests::cross::interview::accepted(&reply);
    }
    reply
}

fn save_mfs_source_owner_fixture(
    session: &mut Interview,
    request: P,
) -> (Reply, Vec<(i64, TaxSection)>) {
    use crate::tests::cross::interview::{accepted, page_documents};
    let setup = session.draft.setup.as_ref().unwrap();
    assert_eq!(setup.filing_status(), FederalStatus::Separate);
    assert!(crate::community::applies(setup));
    assert!(session.draft.people.as_ref().unwrap().spouse.is_none());
    let request = fixture_owner_inputs_inner(&session.draft, request, true, false);
    let expected = page_documents(&request)
        .iter()
        .filter(|row| {
            row.owner == crate::DocumentOwner::Spouses
                && row.document.printed_names().is_some()
                && session
                    .draft
                    .income
                    .iter()
                    .find(|old| old.id == row.id)
                    .and_then(|old| old.printed_names())
                    .is_none_or(|names| {
                        names.recipient.name != MFS_PRINTED_RECIPIENT
                            || names.assigned != crate::DocumentRecipient::Spouse
                    })
        })
        .count();
    let reply = send(session, request);
    accepted(&reply);
    if expected == 0 {
        return (reply, Vec::new());
    }
    let unmatched: Vec<_> = unknown_documents(&session.draft)
        .into_iter()
        .filter(|(id, _)| {
            session.draft.income.iter().any(|source| {
                source.id == *id
                    && source
                        .printed_names()
                        .is_some_and(|names| names.recipient.name == MFS_PRINTED_RECIPIENT)
            })
        })
        .collect();
    assert_eq!(unmatched.len(), expected);
    (reply, unmatched)
}

// The documents whose owner is Unknown, with the page whose person sections
// add them, from the estimate's pending items.
pub(in crate::tests) fn unknown_documents(draft: &FederalDraft) -> Vec<(i64, TaxSection)> {
    crate::interview::unknown_owner_issues(draft)
        .into_iter()
        .map(|context| match context {
            crate::IssueContext::Corrective { source, sections } => match *source {
                crate::IssueContext::Income { id, .. }
                | crate::IssueContext::Record {
                    identity: crate::federal_model::NoteIdentity::Income(id),
                    ..
                } => (id, sections[0]),
                other => panic!("unexpected owner context {other:?}"),
            },
            crate::IssueContext::Record {
                section,
                identity: crate::federal_model::NoteIdentity::Income(id),
                ..
            } => (id, section),
            other => panic!("unexpected owner context {other:?}"),
        })
        .collect()
}

pub(in crate::tests) fn document_owners_mut(
    request: &mut P,
) -> Option<&mut Vec<crate::PersonDocuments>> {
    match request {
        P::SaveDocuments { document_owners, .. } => Some(document_owners),
        _ => None,
    }
}

impl crate::SectionDocument {
    pub(in crate::tests) fn added(
        request: &P,
        document: i64,
        ownership: crate::AddedOwnership,
    ) -> Self {
        let P::SaveDocuments { allowed_document_ownership: allowed, .. } = request else {
            panic!("Income documents supplies ownership choices")
        };
        Self {
            document,
            ownership,
            owner_source: crate::OwnerSource::Added,
            allowed_ownership: allowed.clone(),
            joint_share_needed: None,
            edit_document: Vec::new(),
        }
    }
}

// Save the paper's topic answers, then edit ownership on Income documents.
pub(in crate::tests) fn information_return_answers_and_owners(
    session: &mut Interview,
    papers: &[IncomeDocumentItem],
    topic: TaxSection,
) -> Option<P> {
    let answers =
        crate::tests::support::client::income_pages::information_return_answers_for_topic(
            session, papers, topic,
        )?;
    crate::tests::cross::interview::accepted(&send(session, answers));
    Some(crate::tests::support::client::income_pages::open_document_center(session))
}

// Populate the choices through the same calculation the client invokes after selection.
fn refresh_document_ownership_choices(session: &mut Interview, request: &mut P, person: i64) {
    let selected = document_owners_mut(request)
        .unwrap()
        .iter()
        .find(|view| view.person == person)
        .unwrap()
        .documents
        .iter()
        .map(|row| crate::DocumentOwnerSelection { document: row.document })
        .collect();
    let Reply::Response(P::DocumentOwnershipChoices { documents }) =
        send(session, P::DocumentOwnershipOptions { user: TEST_USER, person, documents: selected })
    else {
        panic!("selected papers supply their ownership choices")
    };
    let view = document_owners_mut(request)
        .unwrap()
        .iter_mut()
        .find(|view| view.person == person)
        .unwrap();
    assert_eq!(view.documents.len(), documents.len());
    for (row, derived) in view.documents.iter_mut().zip(documents) {
        row.allowed_ownership = derived.allowed_ownership;
        row.joint_share_needed = derived.joint_share_needed;
    }
}

// Apply joint ownership through the Income documents person's rows and choices.
pub(in crate::tests) fn set_joint_financial_share(
    session: &mut Interview,
    request: &mut P,
    ids: &[i64],
    taxpayer_share: i32,
) {
    let shown = document_owners_mut(request)
        .expect("a page with person sections")
        .iter()
        .flat_map(|person| person.documents.iter().map(|row| row.document))
        .collect::<Vec<_>>();
    let additions = ids
        .iter()
        .filter(|id| !shown.contains(id))
        .map(|id| crate::SectionDocument::added(request, *id, crate::AddedOwnership::Yours))
        .collect::<Vec<_>>();
    let taxpayer = document_owners_mut(request)
        .unwrap()
        .iter_mut()
        .find(|person| person.person == -1)
        .unwrap();
    for row in additions {
        assert!(
            taxpayer.available_documents.contains(&row.document),
            "document {} is offered",
            row.document
        );
        taxpayer.documents.push(row);
    }
    let people = document_owners_mut(request)
        .unwrap()
        .iter()
        .map(|person| person.person)
        .collect::<Vec<_>>();
    for person in people {
        refresh_document_ownership_choices(session, request, person);
    }
    for person in document_owners_mut(request).unwrap() {
        let share = match person.person {
            -1 => taxpayer_share,
            -2 => 100 - taxpayer_share,
            _ => continue,
        };
        for row in &mut person.documents {
            if ids.contains(&row.document) {
                row.ownership = crate::AddedOwnership::Joint { share };
            }
        }
    }
}

// The section's page with each listed document added to `person`, as the
// person adds it in "Documents you added".
pub(in crate::tests) fn added_documents_request(
    session: &mut Interview,
    section: TaxSection,
    person: i64,
    ids: &[i64],
    ownership: crate::AddedOwnership,
) -> P {
    use crate::tests::cross::interview::from_review;
    let Reply::Edit(mut request) = from_review(session, TaxSection::Documents) else {
        panic!("expected Income documents for {section:?} ownership")
    };
    let rows = ids
        .iter()
        .map(|id| crate::SectionDocument::added(&request, *id, crate::AddedOwnership::Yours))
        .collect::<Vec<_>>();
    let owners = document_owners_mut(&mut request).expect("a page with person sections");
    let view = owners
        .iter_mut()
        .find(|view| view.person == person)
        .unwrap_or_else(|| panic!("person {person} has a section on {section:?}"));
    for row in rows {
        let id = row.document;
        assert!(view.choices.iter().any(|choice| choice.id == id), "document {id} is offered");
        view.documents.push(row);
    }
    refresh_document_ownership_choices(session, &mut request, person);
    let view = document_owners_mut(&mut request)
        .unwrap()
        .iter_mut()
        .find(|view| view.person == person)
        .unwrap();
    for row in &mut view.documents {
        if ids.contains(&row.document) {
            row.ownership = ownership;
        }
    }
    request
}

// Explicit fixture ownership uses the same accepted Save as the document page.
pub(in crate::tests) fn assign_documents(
    session: &mut Interview,
    section: TaxSection,
    person: i64,
    ids: &[i64],
    ownership: crate::AddedOwnership,
) {
    let request = added_documents_request(session, section, person, ids, ownership);
    crate::tests::cross::interview::accepted(&send(session, request));
}

pub(in crate::tests) fn add_documents(
    session: &mut Interview,
    section: TaxSection,
    person: i64,
    ids: &[i64],
    ownership: crate::AddedOwnership,
) -> Reply {
    let request = added_documents_request(session, section, person, ids, ownership);
    send(session, request)
}

pub(in crate::tests) fn joint_financial_session(
    status: FederalStatus,
    sources: Vec<IncomeSource>,
) -> (Interview, Vec<IncomeDocumentItem>) {
    use crate::tests::cross::interview::accepted;
    use crate::tests::support::client::income_pages::save_documents;
    let mut draft = document_recipient_household(FederalStatus::Joint);
    draft.income.clear();
    draft.dependents.clear();
    // Printed for both spouses: the papers name them.
    let joint = fixture_document_names(&draft, -3).unwrap().recipient;
    let papers = sources
        .into_iter()
        .map(|mut source| {
            set_fixture_recipient(&mut source, &joint);
            named_document(&draft, -3, 0, source)
        })
        .collect::<Vec<_>>();
    if status != FederalStatus::Joint {
        draft.setup = complete(status).setup;
        if status == FederalStatus::Separate {
            answer_separate_pages(&mut draft);
        }
        draft.people.as_mut().unwrap().spouse = None.into();
    }
    let mut session = Interview::new(TEST_USER, draft);
    let request = save_documents(&mut session, papers.clone());
    accepted(&send(&mut session, request));
    (session, papers)
}

pub(in crate::tests) fn document_recipient_household(status: FederalStatus) -> FederalDraft {
    let mut draft = complete(status);
    let people = draft.people.as_mut().unwrap();
    people.taxpayer.first_name = "Alex Morgan".into();
    people.taxpayer.last_name = "O'Neil-Smith".into();
    if let Some(spouse) = people.spouse.as_mut() {
        spouse.first_name = "Jordan Lee".into();
        spouse.last_name = "O'Neil-Smith".into();
    }
    add_child(&mut draft);
    let child = draft.dependents.last_mut().unwrap();
    child.first_name = "Casey".into();
    child.last_name = "O'Neil-Smith".into();
    draft
}

pub(in crate::tests) fn document_owner_household(recipients: &[&str]) -> FederalDraft {
    let mut draft = document_recipient_household(FederalStatus::Single);
    let source = draft.income[0].clone();
    let mut next_id = draft.next_id;
    draft.income = recipients
        .iter()
        .enumerate()
        .map(|(index, recipient)| {
            let mut source = source.clone();
            if index > 0 {
                source.id = next_id;
                next_id += 1;
            }
            source.printed = Some(crate::PrintedDocumentNames {
                recipient: crate::PrintedParty::from_name((*recipient).into()),
                issuer: Some(Default::default()),
                assigned: crate::DocumentRecipient::Unknown,
                ..Default::default()
            });
            // Save reads the printed parties from the stored paper.
            prepare_income_fixture(&mut source, None);
            source.owner = crate::DocumentOwner::Unknown;
            source
        })
        .collect();
    draft.next_id = next_id;
    draft
}

pub(in crate::tests) fn after_death_owner_document() -> (FederalDraft, IncomeDocumentItem) {
    let mut draft = document_recipient_household(FederalStatus::Joint);
    let mut final_return = death(20260615).unwrap();
    final_return.ordinary = true;
    draft.people.as_mut().unwrap().spouse.as_mut().unwrap().died = Some(final_return);
    draft.income[0].printed = Some(crate::PrintedDocumentNames {
        recipient: crate::PrintedParty::from_name("Alex Morgan ONeilSmith".into()),
        issuer: None,
        assigned: crate::DocumentRecipient::Taxpayer,
        ..Default::default()
    });
    // Save reads the printed parties from the stored paper.
    prepare_income_fixture(&mut draft.income[0], None);
    let mut source = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Pension,
        10_000,
        RetirementTreatment::Ordinary,
    );
    let FederalIncome::Retirement { data } = &mut source.income else { unreachable!() };
    let mut data = edit_retirement(data);
    data.after_death = Some(crate::RetirementAfterDeath { gross: 100_000, taxable: 100_000 });
    drop(data);
    // The paper is registered with its topic answers; only its recipient is then named.
    let FederalIncome::Retirement { data: crate::RetirementSource::R1099(stored) } = &source.income
    else {
        unreachable!()
    };
    let paper = crate::tests::support::builders::retirement_document(stored.clone());
    let mut document = crate::tests::cross::interview::document(
        0,
        IncomeSource::Retirement { value: crate::RetirementDocument::R1099(paper) },
    );
    set_fixture_recipient(
        &mut document.document,
        &crate::PrintedParty::from_name("Unmatched Recipient".into()),
    );
    (draft, document)
}

pub(in crate::tests) fn accepted_document_names(
    draft: &FederalDraft,
    owner: Owner,
) -> crate::PrintedDocumentNames {
    let recipient = match owner {
        Owner::Taxpayer => crate::DocumentRecipient::Taxpayer,
        Owner::Spouse => crate::DocumentRecipient::Spouse,
    };
    let mut names = fixture_document_names(draft, recipient.key())
        .or_else(|| {
            draft.income.iter().find_map(|source| {
                source.printed_names().filter(|names| names.assigned == recipient).cloned()
            })
        })
        .expect("actual canonical or accepted document identity");
    names.set_assignment(crate::document_names::Assignment::matched(recipient));
    names
}

// The accepted source and its editable projection retain the same printed facts.
pub(in crate::tests) fn prepare_income_fixture(
    source: &mut FederalIncomeEntry,
    household: Option<crate::PrintedDocumentNames>,
) {
    use crate::document_names::AcceptedPartySource;
    let Some(mut names) = source.accepted_parties().or_else(|| {
        crate::interview::document_item(source, &[], false)
            .and_then(|row| row.document.printed_names())
    }) else {
        return;
    };
    if let Some(household) = household {
        // A fixture that moved the entry to the other filer prints that
        // filer's name: the printed name decides the owner.
        let other_filer = names.assigned.is_known()
            && household.assigned.is_known()
            && names.assigned != household.assigned;
        if names.recipient.name.is_empty() || other_filer {
            names.recipient.name = household.recipient.name.clone();
            if other_filer {
                names.set_assignment(household.assignment());
            }
        }
        if !names.assigned.is_known() && names.recipient.name == household.recipient.name {
            names.set_assignment(household.assignment());
        }
    }
    match &mut source.income {
        FederalIncome::Partnership { data } => {
            if let Some(source) = data.source.as_deref_mut() {
                source.paper.partner = names.recipient.clone();
                source.paper.partnership = names.issuer.clone().unwrap_or_default();
                names.assignment().apply_to(
                    &mut source.assigned,
                    &mut source.owner_source,
                    &mut source.joint_share,
                );
            }
        }
        FederalIncome::Scorp { data } => {
            data.paper.shareholder = names.recipient.clone();
            data.paper.corporation = label_issuer(&mut names, &source.label);
            names.assignment().apply_to(
                &mut data.assigned,
                &mut data.owner_source,
                &mut data.joint_share,
            );
        }
        FederalIncome::EmployeeWages { data } => {
            data.source_recipient = names.recipient.clone();
            data.source_issuer = names.issuer.clone().unwrap_or_default();
        }
        FederalIncome::WildfireRelief { data } => {
            data.source_recipient = names.recipient.clone();
            data.source_issuer = names.issuer.clone().unwrap_or_default();
        }
        FederalIncome::DisasterRelief { data } => {
            data.source_recipient = names.recipient.clone();
            data.source_issuer = names.issuer.clone().unwrap_or_default();
        }
        FederalIncome::OpportunityFund { data } => {
            data.source_recipient = names.recipient.clone();
        }
        // Stored papers print their parties: a W-2's employee, and a 1099-INT,
        // -DIV or -OID payer and recipient (the payer names the source on Save).
        FederalIncome::Wage { data: crate::WageSource::W2(data) } => {
            data.form.employee = names.recipient.clone();
        }
        FederalIncome::Interest { data: crate::InterestSource::Int1099(data) } => {
            let form = &mut data.form;
            paper_parties(&mut names, &source.label, &mut form.payer, &mut form.recipient);
        }
        FederalIncome::Dividend { data } => {
            let form = &mut data.form;
            paper_parties(&mut names, &source.label, &mut form.payer, &mut form.recipient);
        }
        FederalIncome::Oid { data } => {
            let form = &mut data.form;
            paper_parties(&mut names, &source.label, &mut form.payer, &mut form.recipient);
        }
        FederalIncome::PersonalDebt { data } => {
            data.form.creditor = label_issuer(&mut names, &source.label);
            data.form.debtor = names.recipient.clone();
        }
        FederalIncome::PropertyAcquisition { data } => {
            data.form.lender = label_issuer(&mut names, &source.label);
            data.form.borrower = names.recipient.clone();
        }
        FederalIncome::EstateTrustPortfolio { data } => {
            data.form.estate_trust = label_issuer(&mut names, &source.label);
            data.form.beneficiary = names.recipient.clone();
        }
        FederalIncome::Sale { data } => {
            data.form.payer = names.issuer.clone().unwrap_or_default();
            data.form.recipient = names.recipient.clone();
            if !data.form.description.is_empty() {
                source.label = data.form.description.clone();
            }
        }
        FederalIncome::HomeSale { data } => {
            let (filer, transferor) = match &mut data.form {
                crate::HomeSaleSource::Paper(form) => (&mut form.filer, &mut form.transferor),
                crate::HomeSaleSource::Records(form) => (&mut form.filer, &mut form.transferor),
            };
            *filer = names.issuer.clone().unwrap_or_default();
            *transferor = names.recipient.clone();
        }
        FederalIncome::Retirement { data: crate::RetirementSource::R1099(data) }
        | FederalIncome::Roth { data }
        | FederalIncome::TrumpDeath { data }
        | FederalIncome::TrumpAbleRollover { data } => {
            data.form.payer = names.issuer.clone().unwrap_or_default();
            data.form.recipient = names.recipient.clone();
        }
        FederalIncome::Nec1099 { data } => {
            data.form.payer = names.issuer.clone().unwrap_or_default();
            data.form.recipient = names.recipient.clone();
        }
        _ => {}
    }
    source.printed = Some(names);
}

fn paper_parties(
    names: &mut crate::PrintedDocumentNames,
    label: &str,
    payer: &mut crate::PrintedParty,
    recipient: &mut crate::PrintedParty,
) {
    *payer = label_issuer(names, label);
    *recipient = names.recipient.clone();
}

// A paper whose issuer names the source on Save prints the fixture's label there.
fn label_issuer(names: &mut crate::PrintedDocumentNames, label: &str) -> crate::PrintedParty {
    let issuer = names.issuer.get_or_insert_with(Default::default);
    if issuer.name.is_empty() {
        issuer.name = label.into();
    }
    issuer.clone()
}

pub(in crate::tests) fn name_accepted_income_documents(draft: &mut FederalDraft) {
    let names: Vec<_> = draft
        .income
        .iter()
        .map(|source| {
            let key = if source.owner == crate::DocumentOwner::Spouses { -2 } else { -1 };
            fixture_document_names(draft, key).map(|mut names| {
                names.set_assignment(crate::document_names::Assignment::matched(
                    if source.owner == crate::DocumentOwner::Spouses {
                        crate::DocumentRecipient::Spouse
                    } else {
                        crate::DocumentRecipient::Taxpayer
                    },
                ));
                names
            })
        })
        .collect();
    for (source, names) in draft.income.iter_mut().zip(names) {
        prepare_income_fixture(source, names);
    }
}

pub(in crate::tests) fn dependent_recipient_with_unavailable_financial_owner() -> FederalDraft {
    let mut draft = document_recipient_household(FederalStatus::Single);
    name_accepted_income_documents(&mut draft);
    let mut source = income(FederalIncome::EducationDistribution {
        data: education_distribution_source(crate::FederalEducationDistribution {
            recipient_not_beneficiary: false,
            coverdell_basis_start: None,
            coverdell_contributions: 0,
            redeposited: 0,
            taxable_transfer_part: 0,
            roth_rollover: None,
            late_rollovers: None,
            gross: 100_000,
            account: EducationAccount::Qtp { earnings: 20_000, basis: 80_000 },
            transfer: crate::tests::cross::interview::plain_withdrawal(),
            beneficiary: Some(crate::FederalEducationBeneficiary::Recipient),
        }),
    });
    source.id = draft.next_id;
    draft.next_id += 1;
    source.label.clear();
    source.owner = crate::DocumentOwner::Spouses;
    if let FederalIncome::EducationDistribution { data } = &mut source.income {
        data.form.recipient = crate::PrintedParty::from_name("Unmatched Recipient".into());
    }
    source.printed = Some(crate::PrintedDocumentNames {
        recipient: crate::PrintedParty::from_name("Unmatched Recipient".into()),
        issuer: None,
        assigned: crate::DocumentRecipient::Unknown,
        ..Default::default()
    });
    draft.income.push(source);
    draft
}

// An accepted manual student choice remains authoritative when household names change.
pub(in crate::tests) fn education_recipient_name_collision() -> FederalDraft {
    let mut draft = document_recipient_household(FederalStatus::Joint);
    name_accepted_income_documents(&mut draft);
    let mut names = fixture_document_names(&draft, -1).unwrap();
    names.issuer = Some(crate::PrintedParty::default());
    names.set_assignment(crate::document_names::Assignment::added(
        crate::DocumentRecipient::Spouse,
        None,
    ));
    let (mut source, answer) =
        native_education(-2, education_expenses(10_000), EducationChoice::LifetimeLearning);
    source.id = draft.next_id;
    draft.next_id += 1;
    let mut source = education_source(source);
    source.form.student = names.recipient;
    source.form.filer = names.issuer.unwrap_or_default();
    crate::document_names::Assignment {
        owner: names.assigned,
        owner_source: names.owner_source,
        joint_share: names.joint_share,
    }
    .apply_to(&mut source.assigned, &mut source.owner_source, &mut source.joint_share);
    let credits = draft.credits.as_mut().unwrap();
    credits.education.push(source);
    credits.students.push(answer);
    draft
}

pub(in crate::tests) fn estimate_issue_household() -> FederalDraft {
    let mut draft = document_recipient_household(FederalStatus::Joint);
    let person = &mut draft.people.as_mut().unwrap().taxpayer;
    person.first_name = "Alice".into();
    person.last_name = "Smith".into();
    name_accepted_income_documents(&mut draft);
    let recipient = fixture_document_names(&draft, -1).unwrap().recipient;
    let (mut tuition, mut annual) =
        native_education(-1, education_expenses(300_000), EducationChoice::AmericanOpportunity);
    tuition.id = draft.next_id;
    draft.next_id += 1;
    let mut tuition = education_source(tuition);
    tuition.form.student = recipient.clone();
    annual.method = None;
    annual.more.american_opportunity_used_up = None;
    let credits = draft.credits.as_mut().unwrap();
    credits.education = vec![tuition];
    credits.students = vec![annual];
    let mut mortgage = super::native_income::mortgage_1098_source();
    mortgage.id = draft.next_id;
    draft.next_id += 1;
    mortgage.form.borrower = recipient;
    mortgage.refund_origin = None;
    draft.deductions.as_mut().unwrap().mortgage = vec![mortgage];
    draft
}
