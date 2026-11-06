use crate::tests::support::assert::line_amount as cents;
use crate::tests::support::client::income_pages::open_document_center;
// u5-credits: the stored data no page asked before (Schedule H, Form 8828,
// Form 5329 Parts X-XI Trump accounts and the Form 4136 home kerosene
// credit), each reached through its page: the Other taxes list or the More
// credits list. Every expected amount is computed by hand in the comment
// beside it from the 2026 draft forms (f1040sh--dft, f8828, f5329--dft
// Parts X-XI, f4136--dft/i4136--dft line 4a).
use super::*;
use crate::tests::cross::interview::{
    go_to, open_other_taxes, other_taxes_request, refusal, save_other_taxes,
};
use crate::tests::federal::other_taxes::{gift, line23, ok, refused_key};
fn recapture(s: &mut Interview, home: MortgageSubsidyEntry) -> i64 {
    ok(&save_other_taxes(s, vec![OtherTax::MortgageSubsidyRecapture { homes: vec![home.into()] }]));
    value(&supported(&s.draft), "mortgage_subsidy_recapture_tax")
}

#[test]
fn form_8828_gift_through_the_other_taxes_page() {
    let d = complete(FederalStatus::Single);
    let before = line23(&d);
    let mut s = Interview::new(TEST_USER, d);
    // Gift of the home (FMV $300,000, basis $200,000), loan closed
    // 2020-06-01, AGI $60,000 against adjusted qualifying income 0:
    //   line 13: 100,000; line 14: 50,000; line 17: 60,000 -> line 18 100%;
    //   line 19: 200,000 x 6.25% = 12,500; line 20 60% -> line 21 7,500;
    //   line 22: 7,500 x 100% = 7,500; line 23: min(50,000, 7,500) = 7,500.
    assert_eq!(recapture(&mut s, gift(20200601, 300_000, 200_000, 0, 60)), 7_500);
    let r = supported(&s.draft);
    assert_eq!(value(&r, "mortgage_subsidy_1_disposition_gain"), 100_000);
    assert_eq!(value(&r, "mortgage_subsidy_1_amount"), 12_500);
    assert_eq!(line23(&s.draft) - before, 750_000);
    // Round trip.
    let stored = s.draft.clone();
    let mut again = restart(stored.clone());
    let page = open_other_taxes(&mut again);
    let P::SaveOtherTaxes { value: shown, .. } = &page else { unreachable!() };
    let request = other_taxes_request(&page, &again.draft.clone(), shown.taxes.clone());
    ok(&send(&mut again, request));
    assert_eq!(again.draft, stored);
}

#[test]
fn form_8828_nine_year_period_boundary() {
    // Disposed 2026-05-01: a loan closed 2017-05-01 is past the nine years
    // (no tax); closed 2017-05-02 it is inside them (7,500 as above).
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    assert_eq!(recapture(&mut s, gift(20170501, 300_000, 200_000, 0, 60)), 0);
    assert_eq!(recapture(&mut s, gift(20170502, 300_000, 200_000, 0, 60)), 7_500);
}

#[test]
fn form_8828_income_percentage_boundaries() {
    // Line 17 = AGI - adjusted qualifying income. AGI is $60,000 (the W-2).
    // $5,000 above -> 100% (7,500); $2,500 above -> 50% (3,750); zero -> no tax.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    assert_eq!(recapture(&mut s, gift(20200601, 300_000, 200_000, 55_000, 60)), 7_500);
    assert_eq!(recapture(&mut s, gift(20200601, 300_000, 200_000, 57_500, 60)), 3_750);
    let notes = supported(&s.draft).review_notes;
    assert!(
        notes.iter().any(|note| {
            note.display
                == Message::new(
                    "NOTE_WITH_LABEL",
                    vec![
                        MessageArg::Text("Maple Street".into()),
                        MessageArg::Nested(alloc::boxed::Box::new(Message::new(
                            "NOTE_FORM_8828_INCOME_AND_HOLDING_PERCENTAGES",
                            vec![MessageArg::Number(50), MessageArg::Number(60)],
                        ))),
                    ],
                )
        }),
        "{notes:?}"
    );
    assert_eq!(recapture(&mut s, gift(20200601, 300_000, 200_000, 60_000, 60)), 0);
}

#[test]
fn form_8828_half_the_gain_limits_the_tax() {
    // Gain $10,000 -> line 14 5,000 < line 22 7,500: tax 5,000. No gain -> 0.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    assert_eq!(recapture(&mut s, gift(20200601, 210_000, 200_000, 0, 60)), 5_000);
    assert_eq!(recapture(&mut s, gift(20200601, 200_000, 200_000, 0, 60)), 0);
}

#[test]
fn form_8828_homes_are_named_once() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let home = gift(20200601, 300_000, 200_000, 0, 60);
    refused_key(
        save_other_taxes(
            &mut s,
            vec![OtherTax::MortgageSubsidyRecapture {
                homes: vec![home.clone().into(), home.into()],
            }],
        ),
        "MORTGAGE_SUBSIDY_HOME_TWICE",
    );
}

// ---------------------------------------------------------------------------
// Form 4136 home kerosene through the More credits list.

fn kerosene(thousandths: u32, cost_cents: i64) -> FederalHomeKerosene {
    FederalHomeKerosene {
        gallons: crate::kernel::natural::natural(i64::from(thousandths), 1_000),
        actual_cost: cost_cents,
        qualified: true,
        not_claimed_elsewhere: true,
        records_complete: true,
    }
}
fn save_more_credits(s: &mut Interview, credits: Vec<MoreCredit>) -> Reply {
    let d = s.draft.clone();
    let opened = send(s, go_to(&d, TaxSection::Credits as i32, TaxSection::MoreCredits));
    assert!(matches!(opened, Reply::Edit(P::SaveMoreCredits { .. })), "{opened:?}");
    let d = s.draft.clone();
    send(
        s,
        P::SaveMoreCredits {
            document_questions: Vec::new(),
            credit_income_sources: crate::tax_credit_bonds::income_sources(&d),
            allowed_credit_owners: vec![Owner::Taxpayer as i32],
            initial_carbon_oxide_origins: vec![],
            initial_renewable_facility_origins: Vec::new(),
            businesses: vec![],
            farms: vec![],
            clean_vehicle_properties: vec![],
            rentals: vec![],
            research_businesses: vec![],
            entered_credits: vec![],
            carryback_credits: vec![],
            allowed_carry_filers: vec![],
            initial_credit_elections: vec![],
            initial_research_choices: vec![],
            user: TEST_USER,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: TaxSection::Credits as i32,
            previous: TaxSection::Credits as i32,
            review: true,
            estimate: None,
            warnings: vec![],
            progress: None,
            allowed_owners: vec![Owner::Taxpayer as i32],
            foreign_businesses: vec![],
            estate_foreign_sources: vec![],
            capital_sales: vec![],
            foreign_situations: vec![],
            mortgage_loans: vec![],
            document_facts: DocumentFacts { year: 2026 },
            value: MoreCredits { answers: Vec::new(), credits, documents: Vec::new() },
        },
    )
}

#[test]
fn home_kerosene_through_the_more_credits_list() {
    let d = complete(FederalStatus::Single);
    let before = supported(&d);
    let mut s = Interview::new(TEST_USER, d);
    // 250.5 gallons x $0.243 = $60.8715 -> 60.87 on Form 4136 line 4a
    // (column (e)), line 17 and Schedule 3 line 12: $61 more paid.
    ok(&save_more_credits(&mut s, vec![MoreCredit::HomeKerosene(kerosene(250_500, 100_000))]));
    let r = supported(&s.draft);
    assert_eq!(cents(&r, "home_kerosene_fuel_4a"), 6_087);
    assert_eq!(cents(&r, "fuel_tax_credit"), 6_087);
    assert_eq!(return_line(&r, "33") - return_line(&before, "33"), 6_100);
    // Stored with the payments, reopened on More credits.
    assert_eq!(s.draft.payments.as_ref().unwrap().home_kerosene, Some(kerosene(250_500, 100_000)));
    let stored = s.draft.clone();
    let mut again = restart(stored.clone());
    let d = again.draft.clone();
    let Reply::Edit(P::SaveMoreCredits { value, .. }) =
        send(&mut again, go_to(&d, TaxSection::Credits as i32, TaxSection::MoreCredits))
    else {
        panic!("expected More credits")
    };
    assert_eq!(value.credits, vec![MoreCredit::HomeKerosene(kerosene(250_500, 100_000))]);
    ok(&save_more_credits(&mut again, value.credits));
    assert_eq!(again.draft, stored);
    // Removing the item clears it; the payments page keeps its own answers.
    ok(&save_more_credits(&mut again, vec![]));
    assert_eq!(again.draft.payments.as_ref().unwrap().home_kerosene, None);
}

#[test]
fn home_kerosene_cost_boundary() {
    // The actual cost may equal the credit (6,087 cents) but not be less.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    ok(&save_more_credits(&mut s, vec![MoreCredit::HomeKerosene(kerosene(250_500, 6_087))]));
    let before = s.draft.clone();
    refused_key(
        save_more_credits(&mut s, vec![MoreCredit::HomeKerosene(kerosene(250_500, 6_086))]),
        "KEROSENE_COST_BELOW_CREDIT",
    );
    assert_eq!(s.draft, before);
}

#[test]
fn home_kerosene_gallon_limit_boundary() {
    // 2,000,000 gallons (2,000,000,000 thousandths) is the most a claim
    // takes; one thousandth more is refused.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    ok(&save_more_credits(
        &mut s,
        vec![MoreCredit::HomeKerosene(kerosene(2_000_000_000, 10_000_000_000))],
    ));
    assert!(
        refusal(save_more_credits(
            &mut s,
            vec![MoreCredit::HomeKerosene(kerosene(2_000_000_001, 10_000_000_000,))],
        ))
        .contains("Form 4136")
    );
}

// ---------------------------------------------------------------------------
// Form 8912 bonds keep their assigned owner in each person's More credits
// section, and a spouse's bonds go with the joint return.

#[test]
fn tax_credit_bond_owner_is_checked_on_save_and_pruned_off_joint() {
    let joint = complete(FederalStatus::Joint);
    let mut spouse_bond = FederalTaxCreditBond1097 {
        owner: Owner::Spouse,
        ..direct_tax_credit_bond(
            "SPOUSE8912",
            FederalTaxCreditBondKind::QualifiedSchoolConstruction,
            1_000,
        )
    };
    spouse_bond.form.recipient = accepted_document_names(&joint, Owner::Spouse).recipient;
    let mut own_bond = direct_tax_credit_bond(
        "OWN8912",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        2_000,
    );
    own_bond.form.recipient = accepted_document_names(&joint, Owner::Taxpayer).recipient;
    // A joint return keeps both filers' bonds.
    let mut s = Interview::new(TEST_USER, joint);
    let mut request = open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.other.credits.documents.extend([
        crate::CreditDocument::CreditBond(alloc::boxed::Box::new(
            crate::TaxCreditBondDocument::from(own_bond.clone()),
        )),
        crate::CreditDocument::CreditBond(alloc::boxed::Box::new(
            crate::TaxCreditBondDocument::from(spouse_bond.clone()),
        )),
    ]);
    ok(&send(&mut s, request));
    assert_eq!(s.draft.credits.as_ref().unwrap().tax_credit_bonds.len(), 2);
    own_bond.id = s.draft.credits.as_ref().unwrap().tax_credit_bonds[0].id;
    spouse_bond.id = s.draft.credits.as_ref().unwrap().tax_credit_bonds[1].id;
    assert_eq!(
        s.draft
            .credits
            .as_ref()
            .unwrap()
            .tax_credit_bonds
            .iter()
            .map(|bond| bond.owner)
            .collect::<Vec<_>>(),
        vec![Owner::Taxpayer, Owner::Spouse],
    );
    let request = crate::tests::support::client::credit_documents::credit_document_answers_request(
        &mut s,
        &[own_bond, spouse_bond],
        None,
    );
    ok(&send(&mut s, request));
    // Source facts survive a change in owner context so they remain available
    // for correction after the household answers change.
    let accepted_bonds = s.draft.credits.as_ref().unwrap().tax_credit_bonds.clone();
    let mut d = s.draft.clone();
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Single);
    d.setup.as_mut().unwrap().marital_state = MaritalState::Unmarried;
    d.people.as_mut().unwrap().spouse = None.into();
    let mut pruned = d.clone();
    let result = crate::interview::prune(&mut pruned);
    assert_eq!(pruned.credits.as_ref().unwrap().tax_credit_bonds, accepted_bonds);
    assert!(
        !result.removed.iter().any(|item| item.key() == "SAVING_KIND_TAX_CREDIT_BOND"),
        "{result:?}"
    );
}

// Derived source-reconciliation fixture, not a published worked example.
// IRC853A(b)(2) treats a RIC shareholder's designated credit distribution as
// a distribution, while Form8912 instructions Rev.December2024 pp.2–4 route
// annual1097-BTC reports to PartIII and other entity statements to PartIV.
// https://uscode.house.gov/view.xhtml?req=granuleid:USC-prelim-title26-section853A
// https://www.irs.gov/instructions/i8912
#[test]
fn tax_credit_entity_distribution_links_retained_dividends_once_and_survives_restart() {
    use crate::tests::cross::interview::{accepted, rejected_with};
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let dividend = native_div1099("RIC statement".into(), 50_000, 10_000);
    let previous_sources = s.draft.income.iter().map(|row| row.id).collect::<Vec<_>>();
    let mut request = open_document_center(&mut s);
    let P::SaveDocuments { documents, allowed_community, .. } = &mut request else {
        unreachable!()
    };
    documents.common.interest_dividends.documents.push(crate::InterestDividendDocumentRow {
        id: 0,
        owner: crate::DocumentOwner::Yours,
        document: crate::InterestDividendDocument::Div1099(dividend_document(dividend)),
        community: crate::CommunityShare::NotNeeded,
        refund_management: None,
        allowed_community: allowed_community.clone(),
        preview: None,
        investment_caps: None,
        refund_management_needed: None,
    });
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut s, request,
    ));
    let source_id =
        s.draft.income.iter().find(|row| !previous_sources.contains(&row.id)).unwrap().id;
    let mut answers =
        crate::tests::cross::interview::open_topic(&mut s, TaxSection::InterestDividends);
    let P::SaveIncomes { followups, .. } = &mut answers else { unreachable!() };
    let nominee = followups
        .iter_mut()
        .find(|row| row.key == alloc::format!("income/{source_id}/nominee"))
        .unwrap();
    let crate::DocumentAnswer::DividendNominee { value: nominee_value } = &mut nominee.answer
    else {
        unreachable!()
    };
    *nominee_value =
        Some(DividendNominee { ordinary: 10_000, qualified: 2_000, ..Default::default() }.into());
    accepted(&send(&mut s, answers));
    let entity = |amount, already, source| TaxCreditEntity {
        entity: "RIC statement".into(),
        designated_income: amount,
        income_already_reported: already,
        income_kind: TaxCreditIncomeKind::RicDistribution,
        income_source: source,
    };
    let mut form = direct_tax_credit_bond(
        "RIC1097",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        30_000,
    );
    form.answers.origin =
        Some(TaxCreditOrigin::Entity1097(entity(30_000, 20_000, Some(source_id))));
    let mut statement = direct_tax_credit_bond(
        "RICOTHER",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        20_000,
    );
    statement.answers.origin = Some(TaxCreditOrigin::EntityStatement(entity(20_000, 0, None)));
    // Each bond's credit is allowed in full: the share confirmation is never asked.
    form.answers.allowed_credit_confirmed = None;
    statement.answers.allowed_credit_confirmed = None;
    let first_credit_id = s.draft.next_id;
    let items = vec![
        crate::CreditDocument::CreditBond(alloc::boxed::Box::new(
            crate::TaxCreditBondDocument::from(form.clone()),
        )),
        crate::CreditDocument::CreditBond(alloc::boxed::Box::new(
            crate::TaxCreditBondDocument::from(statement.clone()),
        )),
    ];
    let mut request = open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.other.credits.documents.extend(items.clone());
    ok(&send(&mut s, request));
    form.id = first_credit_id;
    statement.id = first_credit_id + 1;
    // No printed recipient: both forms are pending until your section adds them.
    ok(&crate::tests::support::builders::add_documents(
        &mut s,
        TaxSection::MoreCredits,
        -1,
        &[form.id, statement.id],
        crate::AddedOwnership::Yours,
    ));
    let request = crate::tests::support::client::credit_documents::credit_document_answers_request(
        &mut s,
        &[form.clone(), statement.clone()],
        None,
    );
    ok(&send(&mut s, request));
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "2b"), 0);
    // RetainedDIV400 + (designated300-already200) + statement200 =700.
    assert_eq!(return_line(&r, "3b"), 70_000);
    assert_eq!(return_line(&r, "3a"), 8_000);
    assert_eq!(value(&r, "form_8912_line1"), 300);
    assert_eq!(value(&r, "form_8912_line_2"), 200);
    assert_eq!(value(&r, "form_8912_line_14"), 300);
    assert_eq!(value(&r, "form_8912_line_20"), 200);
    assert_eq!(value(&r, "tax_credit_bond_credit"), 500);
    let mut s = restart(s.draft);
    let saved = s.draft.clone();
    assert_eq!(estimate(&saved), r);
    let Reply::Edit(P::SaveMoreCredits { value, credit_income_sources, .. }) =
        send(&mut s, go_to(&saved, TaxSection::Credits as i32, TaxSection::MoreCredits))
    else {
        panic!("more credits")
    };
    assert_eq!(
        value.documents,
        vec![
            crate::CreditDocumentView::CreditBond { value: crate::TaxCreditBondPage::from(&form) },
            crate::CreditDocumentView::CreditBond {
                value: crate::TaxCreditBondPage::from(&statement)
            },
        ]
    );
    assert_eq!(credit_income_sources.len(), 1);
    assert_eq!(credit_income_sources[0].id, source_id);
    assert_eq!(credit_income_sources[0].amount, 40_000); //retained ordinary, including actual qualified designation
    // Two credits cannot independently claim the same already-reported income.
    statement.answers.origin =
        Some(TaxCreditOrigin::EntityStatement(entity(30_000, 30_000, Some(source_id))));
    let mut request =
        crate::tests::support::client::credit_documents::credit_document_answers_request(
            &mut s,
            &[form.clone(), statement.clone()],
            None,
        );
    let (_, answered) =
        crate::tests::support::client::credit_documents::answered_origin(&mut request, 1);
    let crate::TaxCreditOrigin::EntityStatement(reused) = statement.answers.origin.clone().unwrap()
    else {
        unreachable!()
    };
    *answered = reused.into();
    refused_key(send(&mut s, request), "CREDIT_INCOME_SOURCE_INVALID");
    assert_eq!(s.draft, saved);
    // Answered for the first time on the questions page, the reused link is refused.
    let mut open = saved.clone();
    open.credits.as_mut().unwrap().tax_credit_bonds[1].answers.origin = None;
    let mut t = restart(open.clone());
    let request = crate::tests::support::client::credit_documents::origin_question(
        &mut t,
        &[form.clone(), statement],
        1,
    );
    refused_key(send(&mut t, request), "CREDIT_INCOME_SOURCE_INVALID");
    assert_eq!(t.draft, open);
    // Deleting a linked dividend fails without changing accepted state or IDs.
    let mut request = open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.common.interest_dividends.documents.retain(|row| row.id != source_id);
    rejected_with(send(&mut s, request), "CREDIT_INCOME_SOURCE_INVALID");
    assert_eq!(s.draft, saved);
    // A restored wrong link fails the same graph rule.
    let mut bad = saved.clone();
    let Some(TaxCreditOrigin::Entity1097(ref mut e)) =
        bad.credits.as_mut().unwrap().tax_credit_bonds[0].answers.origin
    else {
        unreachable!()
    };
    e.income_source = Some(source_id + 100);
    assert!(!bad.valid_stored());
    assert_eq!(estimate(&bad).state, ResultState::Incomplete);
    // Clear the credit before removing its source through the ordinary page.
    let mut request = open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.other.credits.documents.clear();
    ok(&send(&mut s, request));
    let mut request = open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    assert_eq!(documents.common.interest_dividends.documents.len(), 1);
    documents.common.interest_dividends.documents.retain(|row| row.id != source_id);
    accepted(&send(&mut s, request));
}

#[test]
fn tax_credit_old_and_k1_scopes_retain_known_income_without_claiming_unavailable_credit() {
    let mut d = complete(FederalStatus::Single);
    let mut old = direct_tax_credit_bond(
        "OLDCREB",
        FederalTaxCreditBondKind::OriginalCleanRenewableEnergy,
        10_000,
    );
    old.answers.issued_on = Some(20070101);
    old.form.calendar_year = Some(2026);
    old.form.identifier_code = Some(Form1097BtcIdentifierCode::OtherIdentifier);
    old.form.bond_type = Some(Form1097BtcBondType::CleanRenewableEnergy);
    let mut k1 = direct_tax_credit_bond(
        "K1SOURCE",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        20_000,
    );
    k1.answers.origin = Some(TaxCreditOrigin::EntityStatement(TaxCreditEntity {
        entity: "Documented partnership statement".into(),
        designated_income: 20_000,
        income_already_reported: 0,
        income_kind: TaxCreditIncomeKind::Interest,
        income_source: None,
    }));
    d.credits.as_mut().unwrap().tax_credit_bonds = vec![old, k1];
    let r = supported(&d);
    assert_eq!(return_line(&r, "2b"), 30_000);
    assert_eq!(value(&r, "tax_credit_bond_credit"), 0);
    assert!(noted(&r, "WARN_TAX_CREDIT_BOND_LEFT_OUT"));
    assert!(noted(&r, "WARN_TAX_CREDIT_K1_INTERFACE_LEFT_OUT"));
    let restored = restart(d.clone());
    assert_eq!(restored.draft, d);
    assert_eq!(estimate(&restored.draft), r);
}
