use super::*;
use crate::related_forms::{Activity, DocumentKind};
use crate::tests::support::builders::related_forms::{RelatedBusiness, related_document};
use alloc::collections::{BTreeMap, BTreeSet};

#[test]
fn related_page_rebuilds_only_applicable_available_rows() {
    let center = vec![
        related_document(41, DocumentKind::Form1099Nec),
        related_document(72, DocumentKind::Form1099KPart),
        related_document(93, DocumentKind::Mortgage1098),
        related_document(105, DocumentKind::Form1099Nec),
    ];
    let mut activity = RelatedBusiness::default();
    *activity.related_mut() =
        RelatedForms::page::<RelatedBusiness>(&[72], &center, &BTreeSet::from([105]));
    let page = activity.related();
    assert_eq!(page.candidates.iter().map(|d| d.id).collect::<Vec<_>>(), vec![41, 72]);
    assert_eq!(
        page.forms,
        vec![RelatedFormView { key: 72, summary: Some(center[1].summary.clone()) }]
    );
    assert_eq!(RelatedForms::project(&page.candidates, 41), Some(center[0].summary.clone()));
    assert_eq!(RelatedForms::project(&page.candidates, 93), None);
}

#[test]
fn related_save_rejects_missing_inapplicable_claimed_and_duplicate_keys_at_the_member() {
    let center = vec![
        related_document(41, DocumentKind::Form1099Nec),
        related_document(72, DocumentKind::Form1099KPart),
        related_document(93, DocumentKind::Mortgage1098),
    ];
    let claimed = BTreeSet::from([72]);
    for bad in [999, 93, 72, 41] {
        assert_eq!(
            RelatedForms::validate::<RelatedBusiness>(&[41, bad], &center, &claimed),
            Err(1)
        );
    }
    assert_eq!(RelatedForms::validate::<RelatedBusiness>(&[41], &center, &claimed), Ok(()));
    assert_eq!(RelatedForms::validate::<RelatedBusiness>(&[], &center, &claimed), Ok(()));
}

#[test]
fn related_pruning_removes_relations_without_changing_documents() {
    let center = vec![
        related_document(41, DocumentKind::Form1099Nec),
        related_document(93, DocumentKind::Mortgage1098),
    ];
    let original = center.clone();
    let mut keys = vec![999, 41, 93];
    assert!(RelatedForms::prune::<RelatedBusiness>(&mut keys, &center));
    assert_eq!(keys, vec![41]);
    assert!(!RelatedForms::prune::<RelatedBusiness>(&mut keys, &center));
    keys.clear();
    assert_eq!(center, original);
}

#[test]
fn related_effective_claims_resolve_the_complete_cross_kind_cohort() {
    use crate::related_forms::{ActivityClaim, BUSINESS_KINDS, RENTAL_KINDS};
    let center = vec![related_document(72, DocumentKind::Form1099KPart)];
    let mut activities = vec![
        ActivityClaim { id: 8, related: vec![], kinds: BUSINESS_KINDS },
        ActivityClaim { id: 9, related: vec![], kinds: RENTAL_KINDS },
    ];
    assert!(RelatedForms::effective(&activities, &center).is_empty());
    activities[1].related.push(72);
    assert_eq!(RelatedForms::effective(&activities, &center), BTreeMap::from([(72, 9)]));
    activities[0].related.push(72);
    assert!(RelatedForms::effective(&activities, &center).is_empty());
    activities[0].related.clear();
    activities[1].related.clear();
    activities.pop();
    assert_eq!(RelatedForms::effective(&activities, &center), BTreeMap::from([(72, 8)]));
    assert!(activities[0].related.is_empty(), "defaults never create stored claims");
    let mut person_restricted = center.clone();
    person_restricted[0].eligible = vec![9];
    assert!(RelatedForms::effective(&activities, &person_restricted).is_empty());
}

#[test]
fn related_projection_and_view_use_the_verified_users_accepted_return_without_writes() {
    let mut draft = complete(FederalStatus::Single);
    let form = Form1099Nec {
        payer: crate::PrintedParty {
            name: "Synthetic Payer".into(),
            federal_id: "12-3456789".into(),
            address: "101 Example Way".into(),
        },
        recipient: fixture_document_names(&draft, -1).unwrap().recipient,
        cash_tips: 0,
        tip_occupation_codes: Default::default(),
        overtime: 0,
        compensation: 42_123,
        withholding: 123,
        golden_parachute: 0,
        belongs: None,
        state_details: Default::default(),
    };
    add(
        &mut draft,
        income(crate::interview::document_income(&IncomeSource::SelfEmployment {
            value: SelfEmploymentDocument::Form1099Nec(nec_document(form.clone())),
        })),
    );
    let source = draft.income.last_mut().unwrap();
    source.printed = Some(crate::PrintedDocumentNames {
        recipient: form.recipient.clone(),
        issuer: Some(form.payer.clone()),
        assigned: crate::DocumentRecipient::Taxpayer,
        owner_source: Some(crate::OwnerSource::Matched),
        ..Default::default()
    });
    let id = source.id;
    let mut session = Interview::new(TEST_USER, draft.clone());
    let reply = session.handle(P::ProjectRelatedForm { user: TEST_USER, key: id });
    let Some(Reply::Response(P::RelatedFormProjected { summary })) = reply else {
        panic!("expected a related form projection");
    };
    let summary = summary.expect("accepted source summary");
    assert_eq!(summary.amount, Some(42_123));
    assert_eq!(summary.payer, draft.income.last().unwrap().label);
    assert!(!summary.label.is_empty());
    assert!(matches!(
        session.handle(P::ProjectRelatedForm { user: TEST_USER, key: i64::MAX }),
        Some(Reply::Response(P::RelatedFormProjected { summary: None }))
    ));
    assert!(session.handle(P::ProjectRelatedForm { user: TEST_USER + 1, key: id }).is_none());
    let Some(Reply::Response(P::FormViewed {
        owner,
        form: RelatedForm::Form1099Nec { payer, recipient, document: viewed, .. },
        ..
    })) = session.handle(P::ViewRelatedForm { user: TEST_USER, key: id })
    else {
        panic!("expected the accepted form");
    };
    assert_eq!(owner, Some(Owner::Taxpayer));
    assert_eq!(viewed, FederalForm1099Nec::from(&form));
    assert_eq!(payer, form.payer);
    assert_eq!(recipient, form.recipient);
    assert!(matches!(
        session.handle(P::ViewRelatedForm { user: TEST_USER, key: i64::MAX }),
        Some(Reply::Response(P::Problem { .. }))
    ));
    assert_eq!(session.draft, draft);
    assert!(session.handle(P::ViewRelatedForm { user: TEST_USER + 1, key: id }).is_none());
}

#[test]
fn accepted_components_keep_distinct_ids_and_parent_views_across_restart() {
    use crate::related_forms::{catalog, effective_claimants};
    use crate::tests::support::builders::related_forms::related_component_editor;
    let (mut session, request) = related_component_editor();
    let next_id = session.draft.next_id;
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    ));
    let center = catalog(&session.draft);
    assert_eq!(center.len(), 6);
    let ids: BTreeSet<_> = center.iter().map(|d| d.id).collect();
    assert_eq!(ids.len(), center.len());
    assert!(ids.iter().all(|id| *id >= next_id && *id < session.draft.next_id));
    assert!(center.iter().all(|d| d.id != d.form_id));
    let claims = effective_claimants(&session.draft);
    let grant = center.iter().find(|source| source.kind == DocumentKind::Form1099GGrant).unwrap();
    assert_eq!(grant.eligible.len(), 2);
    assert_eq!(
        claims.keys().copied().collect::<BTreeSet<_>>(),
        ids.difference(&BTreeSet::from([grant.id])).copied().collect()
    );
    for source in &center {
        let Some(Reply::Response(P::FormViewed { form, .. })) =
            session.handle(P::ViewRelatedForm { user: TEST_USER, key: source.id })
        else {
            panic!("expected the component's physical form");
        };
        let expected =
            &session.draft.income.iter().find(|e| e.id == source.form_id).unwrap().income;
        assert!(matches!(
            (form, expected),
            (RelatedForm::Form1099K { .. }, FederalIncome::K1099 { .. })
                | (RelatedForm::Form1099Misc { .. }, FederalIncome::Misc1099 { .. })
                | (RelatedForm::Form1099G { .. }, FederalIncome::GovernmentPayment { .. })
        ));
    }
    let mut restarted = restart(session.draft.clone());
    assert_eq!(catalog(&restarted.draft), center);
    let request = crate::tests::support::client::income_pages::open_document_center(&mut restarted);
    accepted(&send(&mut restarted, request));
    assert_eq!(restarted.draft, session.draft);
}

#[test]
fn payment_parts_reorder_by_identity_and_cannot_move_to_another_form() {
    use crate::tests::support::builders::related_forms::related_component_editor;
    use crate::tests::support::client::income_pages::open_document_center;
    let (mut session, request) = related_component_editor();
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    ));
    // A second 1099-K beside the first; the payment parts are the stored answers of each.
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let first = &mut documents.common.self_employment.documents[0];
    let first_id = first.id;
    let mut second = first.clone();
    second.id = 0;
    let SelfEmploymentDocument::Form1099K(form) = &mut second.document else { unreachable!() };
    *form.gross.as_mut().expect("printed gross payments") += 100;
    documents.common.self_employment.documents.push(second);
    // The second form's parts are its own answers, entered on the topic below.
    accepted(&send(&mut session, request));
    let parts_of = |draft: &FederalDraft, id: i64| {
        let FederalIncome::K1099 { data } =
            &draft.income.iter().find(|entry| entry.id == id).unwrap().income
        else {
            unreachable!()
        };
        data.parts.clone()
    };
    let second_id = session
        .draft
        .income
        .iter()
        .rev()
        .find(|entry| matches!(entry.income, FederalIncome::K1099 { .. }) && entry.id != first_id)
        .unwrap()
        .id;
    let original: Vec<_> =
        parts_of(&session.draft, first_id).iter().map(|p| (p.id, p.amount)).collect();
    // Each source composer accepts its own parts independently of the business.
    let first_key = alloc::format!("income/{first_id}/parts");
    let second_key = alloc::format!("income/{second_id}/parts");
    let mut moved = parts_of(&session.draft, first_id);
    let mut reversed = moved.clone();
    reversed.reverse();
    let request = P::SavePaymentParts {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        key: first_key,
        value: reversed,
    };
    accepted(&send(&mut session, request));
    for part in &mut moved {
        part.id = 0;
    }
    moved[0].amount += 100;
    let request = P::SavePaymentParts {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        key: second_key.clone(),
        value: moved,
    };
    accepted(&send(&mut session, request));
    assert_eq!(
        parts_of(&session.draft, first_id).iter().map(|p| (p.id, p.amount)).collect::<Vec<_>>(),
        original.into_iter().rev().collect::<Vec<_>>()
    );
    let stolen = parts_of(&session.draft, first_id)[0].id;
    let mut value = parts_of(&session.draft, second_id);
    value[0].id = stolen;
    let request = P::SavePaymentParts {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        key: second_key,
        value,
    };
    let before = session.draft.clone();
    error(send(&mut session, request));
    assert_eq!(session.draft, before);
    let removed = parts_of(&session.draft, first_id);
    let replacement = P::SavePaymentParts {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        key: alloc::format!("income/{first_id}/parts"),
        value: vec![crate::KPart {
            id: 0,
            amount: removed.iter().map(|part| part.amount).sum(),
            belongs: None,
            refunded: 0,
        }],
    };
    accepted(&send(&mut session, replacement));
    let replacement = parts_of(&session.draft, first_id);
    assert_eq!(replacement.len(), 1);
    assert!(removed.iter().all(|part| part.id != replacement[0].id));
}

#[test]
fn payment_parts_source_return_exposes_ids_and_rejects_stale_or_forged_composers() {
    use crate::tests::support::builders::related_forms::related_component_editor;
    use crate::tests::support::client::income_pages::open_topic;
    let (mut session, printed) = related_component_editor();
    accepted(&send(&mut session, printed));
    let source = session
        .draft
        .income
        .iter()
        .find_map(|entry| matches!(entry.income, FederalIncome::K1099 { .. }).then_some(entry.id))
        .unwrap();
    let parent = open_topic(&mut session, TaxSection::SelfEmployment);
    let before = session.draft.clone();
    let request = P::SavePaymentParts {
        user: TEST_USER,
        cookie: before.cookie.clone(),
        revision: before.revision,
        key: alloc::format!("income/{source}/parts"),
        value: vec![
            crate::KPart { id: 0, amount: 15_000, belongs: None, refunded: 0 },
            crate::KPart { id: 0, amount: 25_000, belongs: None, refunded: 0 },
        ],
    };
    let Reply::Response(P::PaymentPartsSaved { value, revision }) =
        send(&mut session, request.clone())
    else {
        panic!("source-only composer must return the accepted parts and revision")
    };
    assert_eq!(revision, before.revision + 1);
    assert!(value.iter().all(|part| part.id > 0 && part.id != source));
    assert_ne!(value[0].id, value[1].id);
    assert_eq!(session.draft.finished_sections, before.finished_sections);
    assert_eq!(
        session.draft.income.iter().filter(|entry| entry.id != source).collect::<Vec<_>>(),
        before.income.iter().filter(|entry| entry.id != source).collect::<Vec<_>>()
    );
    let options = P::BusinessRelatedOptions {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision,
        id: 0,
        owner: Owner::Taxpayer,
        source: BusinessSource::UnitedStates,
        gambling: None,
    };
    let Reply::Response(P::BusinessRelatedChoices { candidates }) = send(&mut session, options)
    else {
        panic!("the still-open business can refresh its related forms")
    };
    assert!(value.iter().all(|part| candidates.iter().any(|choice| choice.id == part.id)));
    let accepted_draft = session.draft.clone();
    // Neither an older composer nor its older parent can overwrite the accepted source.
    error(send(&mut session, request.clone()));
    assert_eq!(session.draft, accepted_draft);
    error(send(&mut session, parent));
    assert_eq!(session.draft, accepted_draft);
    let mut current_parent = open_topic(&mut session, TaxSection::SelfEmployment);
    let Reply::Response(expected) =
        send(&mut session, estimate_documents_from_save(&current_parent))
    else {
        panic!("accepted source answers must produce a running estimate")
    };
    assert!(matches!(expected, P::IncomeRunningEstimate { .. }));
    let P::SaveIncomes { followups, .. } = &mut current_parent else { unreachable!() };
    let crate::DocumentAnswer::PaymentParts(parts) = &mut followups
        .iter_mut()
        .find(|row| row.key == alloc::format!("income/{source}/parts"))
        .unwrap()
        .answer
    else {
        unreachable!()
    };
    parts.value[0].amount += 100;
    let Reply::Response(actual) = send(&mut session, estimate_documents_from_save(&current_parent))
    else {
        panic!("a forged read-only source view must not replace accepted answers")
    };
    assert_eq!(actual, expected);
    assert_eq!(session.draft, accepted_draft);
    let mut duplicate = value.clone();
    duplicate[1].id = duplicate[0].id;
    let mut request = request;
    let P::SavePaymentParts { revision: supplied_revision, value: supplied, .. } = &mut request
    else {
        unreachable!()
    };
    *supplied_revision = revision;
    *supplied = duplicate;
    error(send(&mut session, request.clone()));
    assert_eq!(session.draft, accepted_draft);
    let P::SavePaymentParts { key, value: supplied, .. } = &mut request else { unreachable!() };
    *key = alloc::format!("income/{source}/community");
    *supplied = value;
    error(send(&mut session, request));
    assert_eq!(session.draft, accepted_draft);
}

#[test]
fn a_removed_box_prunes_its_relation_without_removing_the_form_or_other_boxes() {
    use crate::related_forms::catalog;
    use crate::tests::support::builders::related_forms::related_activity_editor;
    use crate::tests::support::client::income_pages::open_document_center;
    let (mut session, business) =
        related_activity_editor(crate::related_forms::ActivityKind::Business);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let row = &mut documents.other.other_income.documents[0];
    let parent = row.id;
    let OtherIncomeDocument::Form1099Misc(form) = &mut row.document else { unreachable!() };
    let FederalIncome::Misc1099 { data: source } =
        &session.draft.income.iter().find(|entry| entry.id == parent).unwrap().income
    else {
        unreachable!()
    };
    let (rents, royalties) = (source.component_ids.rents, source.component_ids.royalties);
    claim_related_component(&mut session.draft, business, rents);
    claim_related_component(&mut session.draft, business, royalties);
    let retained = royalties;
    form.rents = Some(0);
    accepted(&send(&mut session, request));
    let center = catalog(&session.draft);
    let FederalIncome::Business { data } =
        &session.draft.income.iter().find(|entry| entry.id == business).unwrap().income
    else {
        unreachable!()
    };
    assert_eq!(data.related, vec![retained]);
    assert!(center.iter().any(|d| d.id == retained && d.form_id == parent));
    assert!(session.draft.income.iter().any(|e| e.id == parent));
}

#[test]
fn walk_prunes_all_converted_activity_keys_by_catalog_eligibility_and_preserves_forms() {
    use crate::related_forms::{ActivityKind, catalog_for, draft_claims};
    use crate::tests::support::builders::related_forms::related_activity_editor;
    for kind in [
        ActivityKind::Business,
        ActivityKind::Farm,
        ActivityKind::FarmRental,
        ActivityKind::Rental,
        ActivityKind::Hobby,
        ActivityKind::Royalty,
        ActivityKind::Equipment,
    ] {
        let (session, activity) = related_activity_editor(kind);
        let mut draft = session.draft;
        let activity_ids: BTreeSet<_> = draft_claims(&draft).iter().map(|claim| claim.id).collect();
        let form_ids: BTreeSet<_> = draft
            .income
            .iter()
            .filter(|entry| !activity_ids.contains(&entry.id))
            .map(|entry| entry.id)
            .collect();
        let applicable = catalog_for(&draft, activity);
        let keys: Vec<_> = applicable.iter().map(|source| source.id).collect();
        assert!(!keys.is_empty());
        let inapplicable = crate::related_forms::catalog(&draft)
            .into_iter()
            .find(|source| !keys.contains(&source.id))
            .unwrap()
            .id;
        for key in keys.iter().copied().chain([inapplicable, i64::MAX]) {
            claim_related_component(&mut draft, activity, key);
        }
        let forms: Vec<_> =
            draft.income.iter().filter(|entry| form_ids.contains(&entry.id)).cloned().collect();
        let pruned = crate::interview::prune(&mut draft);
        assert!(pruned.removed.iter().any(|row| row.key() == "SAVING_KIND_RELATED_FORM"));
        assert_eq!(
            draft_claims(&draft).iter().find(|claim| claim.id == activity).unwrap().related,
            keys
        );
        assert_eq!(
            draft
                .income
                .iter()
                .filter(|entry| form_ids.contains(&entry.id))
                .cloned()
                .collect::<Vec<_>>(),
            forms
        );
        assert!(crate::interview::prune(&mut draft).removed.is_empty());

        for source in draft.income.iter_mut().filter(|entry| form_ids.contains(&entry.id)) {
            source.owner = crate::DocumentOwner::Spouses;
        }
        let forms: Vec<_> =
            draft.income.iter().filter(|entry| form_ids.contains(&entry.id)).cloned().collect();
        crate::interview::prune(&mut draft);
        assert!(
            draft_claims(&draft)
                .iter()
                .find(|claim| claim.id == activity)
                .unwrap()
                .related
                .is_empty()
        );
        assert_eq!(
            draft
                .income
                .iter()
                .filter(|entry| form_ids.contains(&entry.id))
                .cloned()
                .collect::<Vec<_>>(),
            forms
        );
        draft.income.retain(|entry| entry.id != activity);
        crate::interview::prune(&mut draft);
        for form in forms {
            assert!(draft.income.contains(&form));
        }
    }
}

#[test]
fn review_shows_the_activity_editors_existing_related_form_rows() {
    use crate::related_forms::ActivityKind;
    use crate::tests::support::builders::related_forms::related_activity_editor;
    for kind in [
        ActivityKind::Business,
        ActivityKind::Farm,
        ActivityKind::FarmRental,
        ActivityKind::Rental,
        ActivityKind::Hobby,
        ActivityKind::Royalty,
        ActivityKind::Equipment,
    ] {
        let (mut session, activity) = related_activity_editor(kind);
        let center = crate::related_forms::catalog_for(&session.draft, activity);
        let key = center[0].id;
        claim_related_component(&mut session.draft, activity, key);
        let page = match kind {
            ActivityKind::Business => {
                RelatedForms::page::<Business>(&[key], &center, &BTreeSet::new())
            }
            ActivityKind::Farm => RelatedForms::page::<Farm>(&[key], &center, &BTreeSet::new()),
            ActivityKind::FarmRental => {
                RelatedForms::page::<FarmRental>(&[key], &center, &BTreeSet::new())
            }
            ActivityKind::Rental => RelatedForms::page::<Rental>(&[key], &center, &BTreeSet::new()),
            ActivityKind::Hobby => {
                RelatedForms::page::<HobbySales>(&[key], &center, &BTreeSet::new())
            }
            ActivityKind::Royalty => {
                RelatedForms::page::<RoyaltyProperty>(&[key], &center, &BTreeSet::new())
            }
            ActivityKind::Equipment => {
                RelatedForms::page::<EquipmentRent>(&[key], &center, &BTreeSet::new())
            }
        };
        let before = session.draft.clone();
        let Reply::Response(P::Review { related_forms, allowed_actions, primary_actions, .. }) =
            send(&mut session, P::ReviewAccepted { user: TEST_USER })
        else {
            panic!("expected review")
        };
        let viewer = libertas_macros::variant_index!(P::ViewReviewedRelatedForm) as i32;
        assert!(allowed_actions.contains(&viewer));
        assert!(!primary_actions.contains(&viewer));
        assert_eq!(related_forms.len(), 1);
        assert!(!related_forms[0].activity.is_empty());
        assert_eq!(related_forms[0].forms.len(), 1);
        assert_eq!(related_forms[0].forms[0].key, key);
        assert_eq!(related_forms[0].forms[0].summary, page.forms[0].summary.clone().unwrap());
        let Reply::Response(reviewed) =
            send(&mut session, P::ViewReviewedRelatedForm { user: TEST_USER, key })
        else {
            unreachable!()
        };
        let Reply::Response(edited) =
            send(&mut session, P::ViewRelatedForm { user: TEST_USER, key })
        else {
            unreachable!()
        };
        assert!(matches!(reviewed, P::FormViewed { .. }));
        assert_eq!(reviewed, edited);
        error(send(&mut session, P::ViewReviewedRelatedForm { user: TEST_USER, key: i64::MAX }));
        assert!(session.handle(P::ViewReviewedRelatedForm { user: TEST_USER + 1, key }).is_none());
        assert_eq!(session.draft, before);
        let mut restarted = restart(session.draft);
        let Reply::Response(P::Review { related_forms: reopened, .. }) =
            send(&mut restarted, P::ReviewAccepted { user: TEST_USER })
        else {
            unreachable!()
        };
        assert_eq!(reopened, related_forms);
        let remaining: Vec<_> = crate::related_forms::catalog(&restarted.draft)
            .into_iter()
            .filter(|source| source.id != key)
            .collect();
        let (request, parent) =
            crate::tests::support::builders::related_forms::remove_related_component_request(
                &mut restarted,
                key,
            );
        let reply = crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut restarted,
            request,
        );
        assert!(
            !matches!(reply, Reply::Response(P::Problem { .. })),
            "{kind:?}, activity {activity}, removed component {key}: {reply:?}"
        );
        assert!(
            crate::related_forms::draft_claims(&restarted.draft)
                .iter()
                .find(|claim| claim.id == activity)
                .unwrap()
                .related
                .is_empty()
        );
        assert_eq!(crate::related_forms::catalog(&restarted.draft), remaining);
        assert!(restarted.draft.income.iter().any(|entry| entry.id == parent));
        let Reply::Response(P::Review { related_forms, .. }) =
            send(&mut restarted, P::ReviewAccepted { user: TEST_USER })
        else {
            unreachable!()
        };
        assert!(related_forms.is_empty());
    }
}

#[test]
fn component_diagnostics_use_the_original_catalog_and_preserve_source_boundaries() {
    use crate::related_forms::{catalog, destination_issues_with_claims, effective_claimants};
    use crate::tests::support::builders::related_forms::related_receipt_draft;
    let mut draft = related_receipt_draft();
    add(&mut draft, income(FederalIncome::FarmRental { data: farm_rental_data(0, 0) }));
    let claims = effective_claimants(&draft);
    let center = catalog(&draft);
    let before = draft.clone();
    let mut result = FederalResult::empty(draft.revision);
    destination_issues_with_claims(&draft, &claims, &mut result);
    let expected: Vec<_> = center
        .iter()
        .filter(|source| {
            matches!(
                source.kind,
                DocumentKind::Form1099GGrant
                    | DocumentKind::Form1099MiscCropInsurance
                    | DocumentKind::Form1099GAgriculture
                    | DocumentKind::Patr1099
            )
        })
        .collect();
    assert_eq!(result.issues.len(), expected.len());
    assert_eq!(expected.len(), 4);
    for (issue, source) in result.issues.iter().zip(expected) {
        assert_eq!(issue.display.key, "ISSUE_RELATED_FORM_ACTIVITY_NOT_CHOSEN");
        assert_eq!(issue.display.args, vec![MessageArg::Encoded(source.part.clone())]);
        let sections =
            if matches!(source.kind, DocumentKind::Patr1099 | DocumentKind::Form1099GGrant) {
                vec![TaxSection::SelfEmployment, TaxSection::Farming]
            } else {
                vec![TaxSection::Farming]
            };
        assert!(
            matches!(&issue.context, IssueContext::Corrective { sections: actual, .. } if actual == &sections)
        );
    }
    assert_eq!(draft, before);
    // Original unresolved claims remain unresolved even if a later working
    // copy would leave only one activity.
    let mut working = draft.clone();
    working.income.retain(|entry| !matches!(entry.income, FederalIncome::FarmRental { .. }));
    assert!(effective_claimants(&working).len() > claims.len());
    let result = estimate(&draft);
    assert_eq!(
        result
            .issues
            .iter()
            .filter(|issue| issue.display.key == "ISSUE_RELATED_FORM_ACTIVITY_NOT_CHOSEN")
            .count(),
        4
    );
    assert!(!result.issues.iter().any(|issue| matches!(
        issue.display.key.as_str(),
        "ISSUE_COOP_ACTIVITY_NEEDED"
            | "ISSUE_FORM_1099_G_BOX_7_AGRICULTURE_PAYMENTS_NEED_A_CHOSEN_FARM"
    )));

    draft.income.retain(|entry| {
        !matches!(entry.income, FederalIncome::Business { .. } | FederalIncome::FarmRental { .. })
    });
    let mut result = FederalResult::empty(draft.revision);
    destination_issues_with_claims(&draft, &effective_claimants(&draft), &mut result);
    assert_eq!(result.issues.len(), 8);
    assert!(
        result
            .issues
            .iter()
            .all(|issue| issue.display.key == "ISSUE_RELATED_FORM_ACTIVITY_MISSING")
    );
}

#[test]
fn personal_only_cooperative_and_zero_net_parts_need_no_invented_activity() {
    use crate::related_forms::{destination_issues_with_claims, effective_claimants};
    use crate::tests::support::builders::related_forms::related_receipt_draft;
    let mut draft = related_receipt_draft();
    draft.income.retain(|entry| matches!(entry.income, FederalIncome::K1099 { .. }));
    let FederalIncome::K1099 { data } = &mut draft.income[0].income else { unreachable!() };
    for part in &mut data.parts {
        part.refunded = part.amount;
    }
    add(&mut draft, income(cooperative_routed_income(80_000, 80_000)));
    let before = draft.clone();
    let mut result = FederalResult::empty(draft.revision);
    destination_issues_with_claims(&draft, &effective_claimants(&draft), &mut result);
    assert!(result.issues.is_empty());
    assert_eq!(draft, before);
}

#[test]
fn a_first_rental_adds_component_keys_then_rebuilds_rows_after_save_and_restart() {
    use crate::tests::cross::interview::rental_page;
    use crate::tests::support::builders::native_income::document;
    use crate::tests::support::builders::related_forms::{related_component_editor, related_keys};
    use crate::tests::support::client::income_pages::{open_topic, page_documents, save_topic};
    let (mut session, request) = related_component_editor();
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    ));
    assert!(!session.draft.income.iter().any(|e| matches!(e.income, FederalIncome::Rental { .. })));
    let before = session.draft.clone();
    let Some(Reply::Response(P::RentalRelatedChoices { candidates })) =
        session.handle(P::RentalRelatedOptions {
            user: TEST_USER,
            id: 0,
            owner: Owner::Taxpayer,
            personal_use: PersonalUse::No,
        })
    else {
        panic!("expected choices for the new rental")
    };
    assert_eq!(candidates.len(), 3);
    assert_eq!(session.draft, before);
    let candidate = candidates.iter().find(|row| row.summary.amount == Some(12_000)).unwrap();
    assert_eq!(candidate.summary.payer, "Fable Land Tenant");
    let key = candidate.id;
    let source = crate::related_forms::catalog(&session.draft)
        .into_iter()
        .find(|row| row.id == key)
        .unwrap();
    assert!(
        session.draft.income.iter().find(|row| row.id == source.form_id).unwrap().label.is_empty()
    );
    let Some(Reply::Response(P::RelatedFormProjected { summary: Some(summary) })) =
        session.handle(P::ProjectRelatedForm { user: TEST_USER, key })
    else {
        panic!("expected the chosen component's row")
    };
    assert_eq!(summary, candidate.summary);
    let Some(Reply::Response(P::FormViewed {
        payer,
        form: RelatedForm::Form1099Misc { payer: printed, document: viewed, .. },
        ..
    })) = session.handle(P::ViewRelatedForm { user: TEST_USER, key })
    else {
        panic!("expected the selected rental's printed form")
    };
    assert_eq!(payer, "Fable Land Tenant");
    assert_eq!(printed.name, payer);
    assert_eq!(viewed.rents, 12_000);
    let mut rental = rental_page(0);
    rental.related = related_keys(&[key]);
    let request = save_topic(
        &mut session,
        TaxSection::Rentals,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Rental(rental) })],
    );
    accepted(&send(&mut session, request));
    let (activity, stored) = session
        .draft
        .income
        .iter()
        .find_map(|entry| match &entry.income {
            FederalIncome::Rental { data } => Some((entry.id, data.related.clone())),
            _ => None,
        })
        .unwrap();
    assert!(activity > 0);
    assert_eq!(stored, vec![key]);
    assert_eq!(
        crate::related_forms::effective_claimants(&session.draft).get(&key),
        Some(&activity)
    );
    let accepted_draft = session.draft.clone();
    session = restart(accepted_draft.clone());
    let rows = page_documents(&open_topic(&mut session, TaxSection::Rentals));
    assert_eq!(rows[0].id, activity);
    let IncomeSource::Record { value: crate::IncomeRecord::Rental(page) } = &rows[0].document
    else {
        unreachable!()
    };
    assert_eq!(page.related.forms, vec![RelatedFormView { key, summary: Some(summary) }]);
    assert!(page.related.candidates.iter().any(|row| row.id == key));
    assert_eq!(session.draft, accepted_draft);
}

#[test]
fn rental_candidates_and_save_respect_another_activitys_component_claim() {
    use crate::tests::cross::interview::rental_page;
    use crate::tests::support::builders::native_income::document;
    use crate::tests::support::builders::related_forms::{related_component_editor, related_keys};
    use crate::tests::support::client::income_pages::save_topic;
    let (mut session, request) = related_component_editor();
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    ));
    let key = crate::related_forms::catalog(&session.draft)
        .iter()
        .find(|source| source.kind == DocumentKind::Form1099KPart)
        .unwrap()
        .id;
    let business = session
        .draft
        .income
        .iter_mut()
        .find_map(|entry| match &mut entry.income {
            FederalIncome::Business { data }
                if matches!(data.activity, BusinessActivity::Ordinary { .. }) =>
            {
                Some(data)
            }
            _ => None,
        })
        .unwrap();
    business.related.push(key);
    let Some(Reply::Response(P::RentalRelatedChoices { candidates })) =
        session.handle(P::RentalRelatedOptions {
            user: TEST_USER,
            id: 0,
            owner: Owner::Taxpayer,
            personal_use: PersonalUse::No,
        })
    else {
        panic!("expected rental choices")
    };
    assert!(!candidates.iter().any(|row| row.id == key));
    let mut rental = rental_page(0);
    rental.related = related_keys(&[key]);
    let request = save_topic(
        &mut session,
        TaxSection::Rentals,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Rental(rental) })],
    );
    let before = session.draft.clone();
    let Reply::Response(P::Problem { message, field, .. }) = send(&mut session, request) else {
        panic!("expected the claimed component to be refused")
    };
    assert_eq!(message, libertas::libertas_formatted_text("PROBLEM_RELATED_FORM_UNAVAILABLE", &[]));
    assert!(field.as_ref().is_some_and(|field| field.ends_with("related/forms/0/key]")));
    assert_eq!(session.draft, before);
}

#[test]
fn filtered_rentals_keep_original_explicit_and_unresolved_mortgage_claims() {
    use crate::related_forms::{RelatedFormContext, effective_claimants};
    use crate::tests::support::builders::related_forms::rental_mortgage_context_fixture;
    for explicit in [true, false] {
        let (mut draft, removed) = rental_mortgage_context_fixture(explicit);
        let claims = effective_claimants(&draft);
        let before = crate::extended_rules::home_mortgage_shares(RelatedFormContext::original(
            &draft, &claims,
        ));
        assert_eq!(before.iter().any(|amount| *amount > 0), explicit);
        draft.income.retain(|entry| entry.id != removed);
        assert_eq!(
            crate::extended_rules::home_mortgage_shares(RelatedFormContext::original(
                &draft, &claims
            )),
            [0; 6]
        );
    }
}

#[test]
fn farm_grant_choices_follow_owner_and_explicit_claim_without_generic_routing() {
    use crate::related_forms::{ActivityKind, catalog_for, effective_claimants};
    use crate::tests::support::builders::related_forms::related_activity_editor;
    let (session, farm) = related_activity_editor(ActivityKind::Farm);
    let mut draft = session.draft;
    let grant = draft
        .income
        .iter_mut()
        .find_map(|entry| {
            let FederalIncome::GovernmentPayment { data } = &mut entry.income else {
                return None;
            };
            data.form.taxable_grants = Some(50_000);
            Some(data.component_ids.grant)
        })
        .unwrap();
    let source = catalog_for(&draft, farm).into_iter().find(|row| row.id == grant).unwrap();
    assert_eq!(source.kind, DocumentKind::Form1099GGrant);
    assert_eq!(source.summary.amount, Some(50_000));
    let business = draft
        .income
        .iter()
        .find(|entry| {
            matches!(&entry.income, FederalIncome::Business { data }
            if matches!(data.activity, BusinessActivity::Ordinary { .. }))
        })
        .unwrap()
        .id;
    let business_claim = BTreeMap::from([(grant, business)]);
    assert_eq!(
        crate::information_returns::farm_grants_with_claims(&draft, business, &business_claim),
        0
    );
    draft.income.retain(|entry| entry.id == farm || entry.id == source.form_id);
    draft.income.iter_mut().find(|entry| entry.id == source.form_id).unwrap().owner =
        crate::DocumentOwner::Spouses;
    assert!(!catalog_for(&draft, farm).iter().any(|row| row.id == grant));
    draft.income.iter_mut().find(|entry| entry.id == source.form_id).unwrap().owner =
        crate::DocumentOwner::Yours;
    claim_related_component(&mut draft, farm, grant);
    let claims = effective_claimants(&draft);
    assert_eq!(claims.get(&grant), Some(&farm));
    assert_eq!(crate::information_returns::farm_grants_with_claims(&draft, farm, &claims), 50_000);
    draft.income.iter_mut().find(|entry| entry.id == source.form_id).unwrap().owner =
        crate::DocumentOwner::Spouses;
    assert_eq!(crate::information_returns::farm_grants_with_claims(&draft, farm, &claims), 0);
    draft.income.iter_mut().find(|entry| entry.id == source.form_id).unwrap().owner =
        crate::DocumentOwner::Yours;
    let original = draft.clone();
    crate::information_returns::route(&mut draft, &claims);
    assert_eq!(draft, original);
    let FederalIncome::GovernmentPayment { data } =
        &mut draft.income.iter_mut().find(|entry| entry.id == source.form_id).unwrap().income
    else {
        unreachable!()
    };
    data.grant_use = Some(crate::GrantUse::You);
    assert!(!catalog_for(&draft, farm).iter().any(|row| row.id == grant));
    assert_eq!(crate::information_returns::farm_grants_with_claims(&draft, farm, &claims), 0);
}

fn receiving_activity_items(estimate: &crate::EstimateCarry) -> usize {
    fn receiving(message: &crate::Message) -> bool {
        matches!(
            message.key.as_str(),
            "ISSUE_RELATED_FORM_ACTIVITY_MISSING" | "ISSUE_RELATED_FORM_ACTIVITY_NOT_CHOSEN"
        ) || message.args.iter().any(|arg| match arg {
            crate::MessageArg::Nested(message) => receiving(message),
            _ => false,
        })
    }
    estimate
        .remaining_items
        .iter()
        .filter(|row| receiving(&crate::Message::decode(&row.item)))
        .count()
}

#[test]
fn farm_receiving_activity_issues_follow_unsaved_links_and_unlinks() {
    use crate::tests::income::information_returns::{misc, misc_form};
    use crate::tests::support::client::{estimate_documents_from_save, income_pages::open_topic};
    for mixed_sources in [false, true] {
        let mut draft = complete(FederalStatus::Single);
        for _ in 0..2 {
            add(&mut draft, income(raised_farm(10_000, 0)));
        }
        let activity = draft.income.iter().find(|entry| matches!(&entry.income,
            FederalIncome::Business { data } if matches!(data.activity, BusinessActivity::Farming { .. })
        )).unwrap().id;
        for amount in if mixed_sources { vec![120_000] } else { vec![50_000, 60_000, 70_000] } {
            let mut paper = government_unemployment(0, &[], 0);
            let FederalIncome::GovernmentPayment { data } = &mut paper.income else {
                unreachable!()
            };
            data.form.agriculture_payments = Some(amount);
            add(&mut draft, paper);
        }
        if mixed_sources {
            // R4-06's three papers: G box 7 $1,200, PATR $700 and MISC box 9 $2,200.
            // No manual crop payment or timing answer is supplied just to link the paper.
            add(&mut draft, income(cooperative_routed_income(70_000, 0)));
            add(
                &mut draft,
                income(misc(FederalForm1099Misc { crop_insurance: 220_000, ..misc_form() })),
            );
        }
        let keys = crate::related_forms::catalog_for(&draft, activity)
            .into_iter()
            .filter(|row| {
                matches!(
                    row.kind,
                    DocumentKind::Form1099GAgriculture
                        | DocumentKind::Patr1099
                        | DocumentKind::Form1099MiscCropInsurance
                )
            })
            .map(|row| row.id)
            .collect::<Vec<_>>();
        assert_eq!(keys.len(), 3);
        let unlink = if mixed_sources {
            crate::related_forms::catalog_for(&draft, activity)
                .into_iter()
                .find(|row| row.kind == DocumentKind::Form1099MiscCropInsurance)
                .unwrap()
                .id
        } else {
            *keys.last().unwrap()
        };
        let mut session = restart(draft);
        let mut request = open_topic(&mut session, TaxSection::Farming);
        let original = session.draft.clone();
        let P::SaveIncomes { estimate: Some(before), .. } = &request else { unreachable!() };
        assert_eq!(receiving_activity_items(&before.clone().into()), 3);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        let row = documents.documents.iter_mut().find(|row| row.id == activity).unwrap();
        let crate::IncomeRecord::Farm(farm) = &mut row.document else { unreachable!() };
        farm.related.forms =
            keys.iter().map(|key| crate::RelatedFormView { key: *key, summary: None }).collect();
        let Reply::Response(P::IncomeRunningEstimate { estimate: Some(linked), .. }) =
            send(&mut session, estimate_documents_from_save(&request))
        else {
            panic!("complete unsaved links must produce the current estimate")
        };
        assert_eq!(receiving_activity_items(&linked), 0);
        assert_eq!(session.draft, original, "calculation never saves pending links");
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        let row = documents.documents.iter_mut().find(|row| row.id == activity).unwrap();
        let crate::IncomeRecord::Farm(farm) = &mut row.document else { unreachable!() };
        farm.related.forms.retain(|row| row.key != unlink);
        assert_eq!(farm.related.forms.len(), 2);
        let Reply::Response(P::IncomeRunningEstimate { estimate: Some(unlinked), .. }) =
            send(&mut session, estimate_documents_from_save(&request))
        else {
            panic!("complete unsaved unlink must produce the current estimate")
        };
        assert_eq!(receiving_activity_items(&unlinked), 1);
        assert_eq!(session.draft, original);
    }
}

#[test]
fn new_unsaved_farm_receives_mixed_papers_without_retyping_crop_insurance() {
    use crate::tests::cross::interview::cash_farm;
    use crate::tests::income::information_returns::{misc, misc_form};
    use crate::tests::support::builders::native_income::document;
    use crate::tests::support::client::{estimate_documents_from_save, income_pages::open_topic};
    let mut draft = complete(FederalStatus::Single);
    let mut government = government_unemployment(0, &[], 0);
    let FederalIncome::GovernmentPayment { data } = &mut government.income else { unreachable!() };
    data.form.agriculture_payments = Some(120_000);
    add(&mut draft, government);
    add(&mut draft, income(cooperative_routed_income(70_000, 0)));
    add(&mut draft, income(misc(FederalForm1099Misc { crop_insurance: 220_000, ..misc_form() })));
    let mut session = restart(draft);
    let mut request = open_topic(&mut session, TaxSection::Farming);
    let original = session.draft.clone();
    let P::SaveIncomes { documents, estimate: Some(before), .. } = &request else { unreachable!() };
    assert!(documents.documents.is_empty());
    assert_eq!(receiving_activity_items(&before.clone().into()), 3);

    let mut farm = cash_farm(0, 0);
    farm.name = "New crop farm".into();
    assert!(matches!(farm.insurance, crate::CropInsurance::None));
    let Reply::Response(P::FarmRelatedChoices { candidates }) = send(
        &mut session,
        P::FarmRelatedOptions {
            user: TEST_USER,
            id: 0,
            owner: Owner::Taxpayer,
            name: farm.name.clone(),
        },
    ) else {
        panic!("new farm must receive its actual related-form choices")
    };
    assert_eq!(candidates.len(), 3);
    farm.related.forms = candidates
        .iter()
        .map(|choice| crate::RelatedFormView {
            key: choice.id,
            summary: Some(choice.summary.clone()),
        })
        .collect();
    farm.related.candidates = candidates;
    let mut row = crate::IncomeRecordItem::from_income(document(
        0,
        IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) },
    ))
    .unwrap();
    crate::interest_limit::refresh(&mut row.document, false);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    documents.documents.push(row);
    let reply = send(&mut session, estimate_documents_from_save(&request));
    let Reply::Response(P::IncomeRunningEstimate { estimate: Some(linked), .. }) = &reply else {
        panic!("new linked farm must calculate before save: {reply:?}")
    };
    assert_eq!(receiving_activity_items(linked), 0);
    assert_eq!(session.draft, original, "preview must not create an accepted farm or revision");

    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    documents.documents.clear();
    let reply = send(&mut session, estimate_documents_from_save(&request));
    let Reply::Response(P::IncomeRunningEstimate { estimate: Some(removed), .. }) = &reply else {
        panic!("removing the unsaved farm must calculate: {reply:?}")
    };
    assert_eq!(receiving_activity_items(removed), 3);
    assert_eq!(session.draft, original);
}

// A topic page that owns an activity kind refuses its save while a saved
// paper's amount needs a receiving activity of that kind and no record links
// it; adding and linking the record lets the page save.
#[test]
fn activity_topic_refuses_save_until_a_record_receives_the_paper() {
    use crate::tests::cross::interview::{business, cash_farm, rental_page};
    use crate::tests::income::information_returns::{misc, misc_form, nec_form};
    use crate::tests::support::builders::native_income::document;
    use crate::tests::support::builders::related_forms::related_keys;
    use crate::tests::support::client::income_pages::save_topic;
    let rents = || {
        let mut paper = misc_form();
        paper.rents = 120_000;
        misc(paper)
    };
    let agriculture = || {
        let mut paper = government_unemployment(0, &[], 0);
        let FederalIncome::GovernmentPayment { data } = &mut paper.income else { unreachable!() };
        data.form.agriculture_payments = Some(120_000);
        paper.income
    };
    type Make = fn(i64) -> crate::IncomeRecord;
    let cases: [(TaxSection, FederalIncome, DocumentKind, Make); 3] = [
        (TaxSection::Rentals, rents(), DocumentKind::Form1099MiscRents, |key| {
            let mut rental = rental_page(0);
            rental.related = related_keys(&[key]);
            crate::IncomeRecord::Rental(rental)
        }),
        (TaxSection::Farming, agriculture(), DocumentKind::Form1099GAgriculture, |key| {
            let mut farm = cash_farm(0, 0);
            farm.related = related_keys(&[key]);
            crate::IncomeRecord::Farm(farm)
        }),
        (TaxSection::SelfEmployment, nec_form(10_000, 0, None), DocumentKind::Form1099Nec, |key| {
            let mut shop = business(100_000);
            shop.related = related_keys(&[key]);
            crate::IncomeRecord::Business(shop)
        }),
    ];
    for (topic, paper, kind, record) in cases {
        let mut draft = complete(FederalStatus::Single);
        add(&mut draft, income(paper));
        let mut session = restart(draft);
        let key = crate::related_forms::catalog(&session.draft)
            .into_iter()
            .find(|row| row.kind == kind)
            .unwrap()
            .id;
        let before = session.draft.clone();
        let request = save_topic(&mut session, topic, vec![]);
        let Reply::Response(P::Problem { message, field, .. }) = send(&mut session, request) else {
            panic!("{topic:?} must refuse while the paper has no receiving activity")
        };
        let expected = crate::Message::new(
            match topic {
                TaxSection::Rentals => "ISSUE_RELATED_FORM_PAGE_RENTALS",
                TaxSection::Farming => "ISSUE_RELATED_FORM_PAGE_FARMING",
                _ => "ISSUE_RELATED_FORM_PAGE_BUSINESS",
            },
            vec![crate::MessageArg::Encoded(
                crate::related_forms::catalog(&before)
                    .into_iter()
                    .find(|row| row.id == key)
                    .unwrap()
                    .part,
            )],
        )
        .encode();
        assert_eq!(message, expected, "{topic:?}");
        assert!(field.is_some(), "{topic:?} issue is linked to the records list");
        assert_eq!(session.draft, before);
        let request = save_topic(
            &mut session,
            topic,
            vec![document(0, IncomeSource::Record { value: record(key) })],
        );
        let reply = send(&mut session, request);
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{topic:?}: {reply:?}");
        assert!(crate::related_forms::effective_claimants(&session.draft).contains_key(&key));
    }
}
