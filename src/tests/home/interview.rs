// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::*;
use crate::tests::cross::interview::{
    PAGE_DEDUCTIONS, PAGE_INCOME, PAGE_ITEMIZED, accepted_linked_mortgage, at_the_deductions_page,
    linked_mortgage_editor, linked_mortgage_form, linked_mortgage_rows, save_deductions,
    save_linked_mortgage_rows, send_past_less_common,
};

// 2026-10-17: canonical home/site records replace the embedded office page.
// The original financial controls remain; Save/reopen now exercises that owner.
#[test]
fn canonical_home_and_casualty_sources_survive_keyed_storage_codecs() {
    use libertas::data::{KeyCodec, KeyDecode};

    for original in
        [FederalDraft::empty("no-home-sources".into()), home_casualty_return(&[2_000_000])]
    {
        let records = crate::return_records::project(TEST_USER, &original, 1).unwrap();
        let decoded = records
            .into_iter()
            .map(|(key, value)| {
                (
                    TaxDataKey::decode_key(&key.to_key_bytes().unwrap()).unwrap(),
                    TaxAppData::from_avro(&value.to_avro()).unwrap(),
                )
            })
            .collect();
        let (restored, _) =
            crate::return_records::assemble(TEST_USER, original.tax_year, decoded).unwrap();
        // Includes accepted home claims, source/property identities, links,
        // exact costs/carry facts and the native identity allocator.
        assert_eq!(restored, original);
    }
}

#[test]
fn simplified_office_months_map_onto_twelve_entries() {
    let mut d =
        office_interview_return(50_000, actual_office(200, 2_000, HomeFixture::Rented { rent: 0 }));
    let source = d.income.last().unwrap().id;
    let mut claim = simplified_claim(200);
    claim.prior_operating = 0;
    claim.prior_depreciation = 0;
    let OfficeMethod::Simplified { months } = &mut claim.method else { panic!() };
    for month in months.iter_mut().filter(|month| month.month > 9) {
        month.exclusive_days = 0;
        month.exclusive_area = 0;
    }
    d.home_sites[0].claims[0] = claim.clone();
    d.home_sites[0].claims[0].activities = vec![OfficeActivity {
        activity: source,
        gross_share: 100.0,
        expense_period_share: 100.0,
        attribution_reason: alloc::string::String::new(),
        deduction_parts: 1,
    }];
    let mut session = restart(d);
    let sites = session.draft.home_sites.clone();
    let request = support::client::save_home_offices(&session.draft, sites.clone());
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.home_sites, sites);
    assert_eq!(
        value(&supported(&session.draft), &alloc::format!("business_{source}_home_office")),
        750
    );
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
    let mut sites = sites;
    sites[0].claims[0].method = simplified_claim(400).method;
    let request = support::client::save_home_offices(&session.draft, sites.clone());
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.home_sites, sites);
    assert_eq!(
        value(&supported(&session.draft), &alloc::format!("business_{source}_home_office")),
        1_500
    );
}

#[test]
fn actual_expense_office_maps_onto_form_8829() {
    let mut session = restart(office_interview_return(50_000, interview_office()));
    let source = session.draft.income.last().unwrap().id;
    let sites = session.draft.home_sites.clone();
    let request = support::client::save_home_offices(&session.draft, sites.clone());
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.home_sites, sites);
    // Standard route: 10% x $16,400 + $20,000 x 2.564% = $2,152.80.
    assert_eq!(
        line_amount(&supported(&session.draft), &alloc::format!("business_{source}_home_office")),
        215_300
    );
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

#[test]
fn actual_expense_office_rules_reject_on_both_sides() {
    let mut session = restart(office_interview_return(50_000, interview_office()));
    let before = session.draft.clone();
    let mut sites = before.home_sites.clone();
    sites[0].claims[0].method =
        OfficeMethod::Actual { exclusive_area: 2_001, shared_daycare_area: 0 };
    let request = support::client::save_home_offices(&session.draft, sites);
    rejected_with(send(&mut session, request), "HOME_BUSINESS_AREA_ABOVE_HOME");
    assert_eq!(session.draft, before);
    // Preserve the original above-basis land rejection under its new owner.
    let mut sites = before.home_sites.clone();
    let HomeTenure::Owned { building: Some(building), .. } = &mut sites[0].tenure else { panic!() };
    building.adjusted_basis = 100_000;
    building.land_basis = 100_001;
    let request = support::client::save_home_offices(&session.draft, sites);
    rejected_with(send(&mut session, request), "HOME_BUILDING_SOURCE_INVALID");
    assert_eq!(session.draft, before);
    let mut sites = before.home_sites.clone();
    sites[0].claims[0].activities[0].activity = i64::MAX;
    let request = support::client::save_home_offices(&session.draft, sites);
    rejected_with(send(&mut session, request), "HOME_ACTIVITY_SOURCE_INVALID");
    assert_eq!(session.draft, before);
}

#[test]
fn actual_expense_office_something_else_is_left_out_with_a_notice() {
    let mut session = restart(office_interview_return(50_000, interview_office()));
    let mut sites = session.draft.home_sites.clone();
    sites[0].claims[0].use_kind = OfficeQualifiedUse::NotEstablished;
    let request = support::client::save_home_offices(&session.draft, sites.clone());
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.home_sites, sites);
    assert!(noted(&supported(&session.draft), "WARN_HOME_USE_NOT_ESTABLISHED"));
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

#[test]
fn mortgage_links_standard_route_opens_and_saves_without_itemizing() {
    let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
    let before = s.draft.clone();
    let P::SaveMortgages { page, previous, mortgage_loans, .. } = linked_mortgage_editor(&mut s)
    else {
        unreachable!()
    };
    assert_eq!((page, previous), (PAGE_DEDUCTIONS, PAGE_DEDUCTIONS));
    assert!(mortgage_loans.is_empty());
    let original = accepted_linked_mortgage(&mut s);
    assert_eq!(original.id, before.next_id);
    assert!(original.id > 0);
    assert_eq!(s.draft.next_id, before.next_id + 1);
    // Accept the printed source, then its nonprinted property answer.
    assert_eq!(s.draft.revision, before.revision + 2);
    assert_eq!(
        s.draft.deductions.as_ref().unwrap().itemize_override,
        before.deductions.unwrap().itemize_override
    );
    assert_eq!(original.more.property, "Original loan");
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
}

#[test]
fn mortgage_links_accepted_ids_survive_equal_names_reordering_and_renaming() {
    let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
    let reply = save_linked_mortgage_rows(
        &mut s,
        vec![linked_mortgage_form("Same name"), linked_mortgage_form("Same name")],
    );
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let mut rows = linked_mortgage_rows(&mut s);
    let ids: Vec<_> = rows.iter().map(|loan| loan.id).collect();
    assert!(ids[0] > 0 && ids[1] > 0 && ids[0] != ids[1]);
    let next_id = s.draft.next_id;
    rows[0].more.property = "Renamed original".into();
    rows.reverse();
    let reply = save_linked_mortgage_rows(&mut s, rows.clone());
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    assert_eq!(linked_mortgage_rows(&mut s), rows);
    assert_eq!(s.draft.next_id, next_id);
    let P::SaveMortgages { mortgage_loans, .. } = linked_mortgage_editor(&mut s) else {
        unreachable!()
    };
    assert_eq!(mortgage_loans.iter().map(|loan| loan.id).collect::<Vec<_>>(), vec![ids[1], ids[0]]);
    assert!(status_text(&mortgage_loans[1].label).contains("Renamed original"));
}

#[test]
fn mortgage_links_transfer_uses_saved_root_and_rejects_its_removal_or_conversion() {
    let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
    let original = accepted_linked_mortgage(&mut s);
    let mut second_root = linked_mortgage_form("Another loan");
    second_root.interest = 300_000;
    let mut transferred = linked_mortgage_form("Later lender");
    transferred.more.details = MortgageDetails::EarlierLender { loan: original.id };
    transferred.principal = 9_000_000;
    let reply =
        save_linked_mortgage_rows(&mut s, vec![original.clone(), second_root.clone(), transferred]);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let accepted = linked_mortgage_rows(&mut s);
    assert_eq!(accepted[2].more.details, MortgageDetails::EarlierLender { loan: original.id });
    assert!(accepted[2].id > 0 && accepted[2].id != original.id);
    let P::SaveMortgages { mortgage_loans, .. } = linked_mortgage_editor(&mut s) else {
        unreachable!()
    };
    assert_eq!(
        mortgage_loans.iter().map(|loan| loan.id).collect::<Vec<_>>(),
        vec![accepted[0].id, accepted[1].id]
    );
    let before = s.draft.clone();
    // The accepted catalog still lists the removed root; the edited graph must reject it.
    rejected_with(
        save_linked_mortgage_rows(&mut s, vec![accepted[1].clone(), accepted[2].clone()]),
        "MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED",
    );
    assert_eq!(s.draft, before);
    let mut converted = accepted.clone();
    converted[0].more.details = MortgageDetails::EarlierLender { loan: accepted[1].id };
    rejected_with(
        save_linked_mortgage_rows(&mut s, converted),
        "MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED",
    );
    assert_eq!(s.draft, before);
    // Explicitly deleting the transfer and retaining its original is permitted.
    let reply = save_linked_mortgage_rows(&mut s, vec![accepted[0].clone(), accepted[1].clone()]);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    second_root.id = accepted[1].id;
    second_root.borrower = accepted_document_names(&s.draft, Owner::Taxpayer).recipient;
    assert_eq!(linked_mortgage_rows(&mut s), vec![original, second_root]);
}

#[test]
fn mortgage_links_echoed_catalog_cannot_authorize_unsaved_or_foreign_roots() {
    let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
    let before = s.draft.clone();
    let forged_id = before.next_id + 1000;
    let mut transferred = linked_mortgage_form("Later lender");
    transferred.more.details = MortgageDetails::EarlierLender { loan: forged_id };
    let mut request = crate::tests::support::client::income_pages::open_document_center(&mut s);
    let P::SaveDocuments { mortgage_loans, rental_mortgage_loans, documents, .. } = &mut request
    else {
        unreachable!()
    };
    mortgage_loans.push(RecordChoice {
        id: forged_id,
        label: libertas::libertas_formatted_text("MORTGAGE_PROPERTY_UNNAMED", &[]),
    });
    rental_mortgage_loans.push(forged_id);
    documents.common.mortgages.documents = vec![mortgage_document(transferred.clone())];
    rejected_with(
        save_linked_mortgage_rows(&mut s, vec![transferred]),
        "MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED",
    );
    assert_eq!(s.draft, before);
    let mut request = crate::tests::support::client::income_pages::open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let root = linked_mortgage_form("Unsaved original");
    let mut transferred = root.clone();
    // Accepting a new paper root cannot authorize an unrelated loan identity.
    transferred.more.details = MortgageDetails::EarlierLender { loan: forged_id };
    documents.common.mortgages.documents =
        vec![mortgage_document(root.clone()), mortgage_document(transferred.clone())];
    rejected_with(
        save_linked_mortgage_rows(&mut s, vec![root, transferred]),
        "MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED",
    );
    assert_eq!(s.draft, before);
}

#[test]
fn mortgage_links_switching_deduction_method_preserves_source_records() {
    let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
    accepted_linked_mortgage(&mut s);
    let loans = s.draft.deductions.as_ref().unwrap().mortgage.clone();
    let next_id = s.draft.next_id;
    for route in [None, Some(ItemizeOverride::Itemize), None, None] {
        let request = save_deductions(&s.draft, route, 0);
        let reply = send_past_less_common(&mut s, request);
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        assert_eq!(s.draft.deductions.as_ref().unwrap().mortgage, loans);
        assert_eq!(s.draft.next_id, next_id);
        let P::SaveMortgages { page, .. } = linked_mortgage_editor(&mut s) else { unreachable!() };
        assert_eq!(page, PAGE_ITEMIZED);
    }
}

#[test]
fn mortgage_links_failed_candidate_does_not_consume_identity_and_preview_is_transient() {
    let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
    let before = s.draft.clone();
    let mut forged = linked_mortgage_form("Forged accepted identity");
    forged.id = before.next_id + 1000;
    let reply =
        save_linked_mortgage_rows(&mut s, vec![linked_mortgage_form("New original"), forged]);
    error(reply);
    assert_eq!(s.draft, before);
    let preview = crate::tests::cross::interview::save_mortgages(
        &mut s,
        MortgageDocuments {
            documents: vec![mortgage_document(linked_mortgage_form("Preview only"))],
        },
    );
    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(&mut s, preview),
        Reply::Response(P::Review { .. })
    ));
    assert_eq!(s.draft, before);
    let original = accepted_linked_mortgage(&mut s);
    assert_eq!(original.id, before.next_id);
    assert_eq!(s.draft.next_id, before.next_id + 1);
}

#[test]
fn home_casualty_binding_saves_in_either_page_order_without_an_identity_cycle() {
    for home_first in [true, false] {
        let mut draft = home_casualty_return(&[2_000_000]);
        let home_id = draft.home_sites[0].id;
        let event = &mut draft.deductions.as_mut().unwrap().personal_casualties[0];
        let (event_id, property_id) = (event.id, event.properties[0].id);
        event.properties[0].home_site = None;
        event.properties[0].business_damage = None;
        draft.home_sites[0].casualties.clear();
        let next_id = draft.next_id;
        let mut session = restart(draft);
        for step in 0..2 {
            let home_page = (step == 0) == home_first;
            let section = if home_page { TaxSection::HomeOffices } else { TaxSection::Deductions };
            let request = P::Navigate {
                user: TEST_USER,
                cookie: session.draft.cookie.clone(),
                revision: session.draft.revision,
                from: TaxSection::Income as i32,
                section: section.into(),
                navigation: NavigationAvailability::default(),
            };
            let Reply::Edit(mut editor) = send(&mut session, request) else { panic!() };
            match &mut editor {
                P::SaveHomeOffices { value, .. } if home_page => {
                    assert_eq!(value.sites[0].id, home_id);
                    value.sites[0].casualties = vec![event_id];
                }
                P::SaveDeductions { value, .. } if !home_page => {
                    assert_eq!(value.casualties[0].id, event_id);
                    assert_eq!(value.casualties[0].properties[0].id, property_id);
                    value.casualties[0].properties[0].home_link = Some(CasualtyHomeLinkPage {
                        home_site: home_id,
                        business_damage: HomeCasualtyDamage::Damaged,
                    });
                }
                _ => panic!("unexpected page {editor:?}"),
            }
            accepted(&send(&mut session, editor));
            assert!(session.draft.valid_stored());
            assert_eq!(session.draft.next_id, next_id);
            let saved = session.draft.clone();
            session = restart(saved.clone());
            assert_eq!(session.draft, saved);
            if step == 0 {
                assert_eq!(
                    crate::home_sites::casualty_binding(&session.draft).unwrap(),
                    Some(home_id)
                );
                let result = estimate(&session.draft);
                assert_eq!(result.state, ResultState::Unsupported);
                assert!(result.tax.is_none());
                assert!(
                    result
                        .issues
                        .iter()
                        .any(|issue| issue.display.key == "HOME_CASUALTY_PROPERTY_LINK_REQUIRED")
                );
                assert!(matches!(
                    crate::home_sites::personal_casualties(&session.draft),
                    Err(Section179PreparedIssue::SourceBoundaryUnavailable(
                        _,
                        "HOME_CASUALTY_PROPERTY_LINK_REQUIRED"
                    ))
                ));
            }
        }
        assert_eq!(crate::home_sites::casualty_binding(&session.draft).unwrap(), None);
        supported(&session.draft);
    }
}

// 2026-10-19 acceptance defect 1: a claim refusal names the field that causes it.
#[test]
fn home_claim_refusals_are_located_on_their_field() {
    use crate::tests::support::assert::k1res::assert_located_rejection;
    let mut session = restart(office_interview_return(50_000, interview_office()));
    let mut sites = session.draft.home_sites.clone();
    sites[0].claims[0].use_kind =
        OfficeQualifiedUse::Daycare { license: DaycareLicense::Approved, hours: 8_761 };
    sites[0].claims[0].method =
        OfficeMethod::Actual { exclusive_area: 0, shared_daycare_area: 1_500 };
    let request = support::client::save_home_offices(&session.draft, sites);
    assert_located_rejection(
        &mut session,
        request,
        "HOME_DAYCARE_TIME_INVALID",
        "[value/sites/0/claims/0/use_kind/hours]",
    );
    let mut sites = session.draft.home_sites.clone();
    sites[0].claims[0].method = simplified_claim(400).method;
    let mut second = sites[0].clone();
    second.id = 0;
    second.mortgage_loans.clear();
    sites.push(second);
    let request = support::client::save_home_offices(&session.draft, sites);
    assert_located_rejection(
        &mut session,
        request,
        "HOME_SIMPLIFIED_ONE_HOME_PER_OWNER",
        "[value/sites/1/claims/0/method]",
    );
}

// 2026-10-19 acceptance defects 4 and 5: Schedule C and Form 8829 print whole dollars
// while their calculation rows keep cents, and their text names the owner and the year.
#[test]
fn schedule_c_and_form_8829_print_whole_dollars_with_plain_owner_and_year_text() {
    let mut session = restart(office_interview_return(50_000, interview_office()));
    let mut sites = session.draft.home_sites.clone();
    sites[0].claims[0].use_kind =
        OfficeQualifiedUse::Daycare { license: DaycareLicense::Approved, hours: 4_000 };
    sites[0].claims[0].method =
        OfficeMethod::Actual { exclusive_area: 0, shared_daycare_area: 500 };
    let request = support::client::save_home_offices(&session.draft, sites);
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    let form = |line: &FormLine| matches!(line.form, TaxForm::ScheduleC | TaxForm::Form8829);
    assert!(
        result.lines.iter().any(|l| l.form_line.as_ref().is_some_and(form) && l.amount % 100 != 0),
        "the calculation keeps cents"
    );
    let printed: Vec<_> = crate::draft_return(&result)
        .into_iter()
        .filter(|r| r.form_line.as_ref().is_some_and(form))
        .collect();
    assert!(!printed.is_empty());
    assert!(printed.iter().all(|r| r.amount % 100 == 0 && r.money.is_none_or(|m| m % 100 == 0)));
    let (_, args) = crate::federal_model::id_pattern("home_1_owner_0_form_8829_total");
    assert!(args.contains(&MessageArg::Resource("PERSON_YOU".into())));
    // A year prints without a thousands separator.
    for key in ["NOTE_METHOD_CHANGE_FORM_3115", "EXPLAIN_METHOD_CHANGE_INCREASE"] {
        let text = crate::APP_STRINGS.iter().find(|(k, _)| *k == key).unwrap().1;
        assert!(text.contains(",number,0}"), "{key}");
    }
}

#[test]
fn beneficiary_salt_and_home_sources_survive_native_save_reopen_and_restart() {
    let mut session = restart(office_estate_tax_return(3_400_000, [70_000, 0, 0, 0]));
    crate::tests::support::client::admit_native_fixture_facts(&mut session);
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    // The fiduciary's statement supplies the beneficiary's personal property tax allocation.
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let document = documents
        .common
        .k1
        .documents
        .iter_mut()
        .find_map(|row| match &mut row.document {
            crate::K1Document::K1EstateTrust(document) => Some(document),
            _ => None,
        })
        .unwrap();
    let mut estate = crate::tests::support::builders::edit_estate_document(document);
    let K1FinalYear::Final { personal_property_tax, .. } = &mut estate.more.final_year else {
        unreachable!()
    };
    *personal_property_tax = 70_001;
    drop(estate);
    accepted(&send(&mut session, request));
    let sources = session.draft.income.clone();
    let homes = session.draft.home_sites.clone();
    let request = support::client::save_home_offices(&session.draft, homes.clone());
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income, sources);
    assert_eq!(session.draft.home_sites, homes);
    let before = session.draft.clone();
    let request = P::Navigate {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        from: PAGE_INCOME,
        section: TaxSection::HomeOffices.into(),
        navigation: NavigationAvailability::default(),
    };
    let Reply::Edit(request @ P::SaveHomeOffices { .. }) = send(&mut session, request) else {
        unreachable!()
    };
    accepted(&send(&mut session, request));
    let expected = supported(&session.draft);
    assert_eq!(session.draft, before);
    assert_eq!(value(&expected, "salt_deduction"), 40_100);
    assert_eq!(line_amount(&expected, &office_line_id(&expected, "excess_taxes")), 30_001);
    let mut cold = restart(session.draft.clone());
    let request = crate::tests::support::client::income_pages::open_documents(&mut cold);
    accepted(&send(&mut cold, request));
    let request =
        crate::tests::cross::interview::go_to(&cold.draft, PAGE_INCOME, TaxSection::HomeOffices);
    let Reply::Edit(request @ P::SaveHomeOffices { .. }) = send(&mut cold, request) else {
        unreachable!()
    };
    accepted(&send(&mut cold, request));
    assert_eq!(cold.draft, before);
    assert_eq!(supported(&cold.draft), expected);
}
