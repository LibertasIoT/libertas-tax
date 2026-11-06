use super::*;
use crate::tests::support::client::income_pages::open_document_center;

#[test]
fn review_document_issue_reveals_its_k1_row_in_document_center() {
    let draft = complete(FederalStatus::Single);
    let mut k1 = crate::tests::support::builders::current_ptp_page(20_000, true);
    k1.reporting_period = Some(crate::K1EntityTaxPeriod { began: 20260101, ended: 20261231 });
    k1.more.section179 = 500_000;
    k1.more.amt_section179_income_carry = None;
    let (mut session, reply) = save_1065s(&draft, vec![k1]);
    accepted(&reply);
    let id = session.draft.income.last().unwrap().id;
    let result = estimate(&session.draft);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| { issue.display.key == "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE" }),
        "{:?}",
        result.issues
    );
    let (items, ..) = review_state(&mut session);
    let row = items
        .iter()
        .find(|row| {
            row.section == TaxSection::Documents
                && status_text(&row.status).contains("Section 179 requires")
        })
        .expect("Review retains the document-scoped section 179 issue");
    // Document Center's row lists are unkeyed: the locator uses their position.
    assert_eq!(row.field.as_deref(), Some("[documents/common/k1/documents/0]"));
    let accepted = session.draft.clone();
    let request = P::OpenSection {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        from: TaxSection::Review as i32,
        section: row.section,
        reveal_path: row.field.clone(),
    };
    let Reply::Edit(P::SaveDocuments { documents, .. }) = send(&mut session, request) else {
        panic!("the Review link opens the whole Document Center")
    };
    // Resolve the locator in the actual request that the link lands on.
    let target = &documents.common.k1.documents[0];
    assert_eq!(target.id, id);
    assert!(matches!(target.document, crate::K1Document::K1Partnership(_)));
    assert_eq!(session.draft, accepted);
}

#[test]
fn debt_paper_and_personal_answers_survive_correction_without_inference() {
    use crate::tests::support::client::income_pages::{information_return_answers, save_documents};
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let IncomeSource::CanceledDebt { value: crate::CanceledDebtDocument::Cancel1099C(mut form) } =
        car_1099c(150_000, CanceledDebtChoice::Taxable)
    else {
        unreachable!()
    };
    form.interest = Some(0);
    form.debt_description = "Synthetic vehicle loan".into();
    form.event_code = Some(crate::Form1099CEventCode::OtherActualDischarge);
    let property = TransferredProperty::PersonalUse {
        basis: 1_200_000,
        term: GainTerm::Long,
        collectible: false,
    };
    form.property_value = Some(700_000);
    let form = debt_document(crate::PersonalDebtSource {
        form,
        treatment: Some(CanceledDebtChoice::Taxable),
        property: Some(property.clone()),
    });
    let desired = vec![document(
        0,
        IncomeSource::CanceledDebt { value: crate::CanceledDebtDocument::Cancel1099C(form) },
    )];
    let request = save_documents(&mut session, desired.clone());
    accepted(&send(&mut session, request));
    let source = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::PersonalDebt { .. }))
        .unwrap();
    let id = source.id;
    let FederalIncome::PersonalDebt { data } = &source.income else { unreachable!() };
    assert_eq!(data.form.property_value, Some(700_000));
    assert_eq!(data.form.interest, Some(0));
    assert_eq!(data.form.debt_description, "Synthetic vehicle loan");
    assert_eq!(data.form.event_code, Some(crate::Form1099CEventCode::OtherActualDischarge));
    assert!(
        data.treatment.is_none() && crate::PersonalDebtCancellation::from(data).transfer.is_none(),
        "paper proves no personal answer"
    );
    assert!(session.draft.valid_stored());
    let (_, _, actions, _) = review_state(&mut session);
    assert!(!actions.contains(&(libertas_macros::variant_index!(P::StartFinish) as i32)));
    let answers = information_return_answers(&mut session, &desired);
    accepted(&send(&mut session, answers));
    let baseline = supported(&session.draft);
    let personal = session.draft.income.iter().find(|entry| entry.id == id).unwrap().income.clone();

    let mut correction = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut correction else { unreachable!() };
    let crate::CanceledDebtDocument::Cancel1099C(form) =
        &mut documents.other.canceled_debt.documents[0].document
    else {
        unreachable!()
    };
    form.property_value = None;
    accepted(&send(&mut session, correction));
    assert!(session.draft.valid_stored());
    session = restart(session.draft.clone());
    let FederalIncome::PersonalDebt { data } =
        &session.draft.income.iter().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    let FederalIncome::PersonalDebt { data: before } = &personal else { unreachable!() };
    assert_eq!(data.treatment, before.treatment);
    // The stored property answer survives; the derived record shows no
    // transfer while box 7 is blank.
    assert_eq!(data.property, before.property);
    assert_eq!(data.form.property_value, None);
    let answers = open_topic(&mut session, TaxSection::CanceledDebt);
    let unchanged = session.draft.clone();
    rejected_with(send(&mut session, answers), "DEBT_PROPERTY_FMV_NEEDED");
    assert_eq!(session.draft, unchanged);
    let mut restore = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut restore else { unreachable!() };
    let crate::CanceledDebtDocument::Cancel1099C(form) =
        &mut documents.other.canceled_debt.documents[0].document
    else {
        unreachable!()
    };
    form.property_value = Some(700_000);
    accepted(&send(&mut session, restore));
    assert_eq!(session.draft.income.iter().find(|entry| entry.id == id).unwrap().income, personal);
    let restored = supported(&session.draft);
    assert_eq!(restored.tax, baseline.tax);
    assert_eq!(restored.return_lines, baseline.return_lines);

    let desired = vec![document(
        0,
        IncomeSource::CanceledDebt {
            value: crate::CanceledDebtDocument::Acquisition1099A(acquisition_document(
                crate::PropertyAcquisitionSource {
                    form: crate::Form1099A {
                        lender: crate::PrintedParty::from_name("Synthetic lender".into()),
                        borrower: Default::default(),
                        description: "Synthetic car".into(),
                        date: 20260801,
                        balance: 800_000,
                        value: 700_000,
                        liable: true,
                        ..Default::default()
                    },
                    property: Some(AcquiredProperty::Property { kind: property }),
                },
            )),
        },
    )];
    let request = save_documents(&mut session, desired.clone());
    accepted(&send(&mut session, request));
    let FederalIncome::PropertyAcquisition { data } = &session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::PropertyAcquisition { .. }))
        .unwrap()
        .income
    else {
        unreachable!()
    };
    assert!(data.property.is_none() && crate::PropertyAcquisition::from(data).crop.is_none());
    assert_eq!(data.form.description, "Synthetic car");
    let answers = information_return_answers(&mut session, &desired);
    accepted(&send(&mut session, answers));
    supported(&session.draft);
}

#[test]
fn center_add_edit_remove_preserves_topic_records_and_source_order() {
    let mut draft = complete(FederalStatus::Single);
    add(
        &mut draft,
        income(document_income(&IncomeSource::Record {
            value: crate::IncomeRecord::OtherIncome(crate::OtherIncomeItem {
                kind: crate::OtherIncomeKind::AlaskaDividend,
                amount: 12_345,
                withholding: 0,
            }),
        })),
    );
    add(
        &mut draft,
        income(document_income(&IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(native_int1099(
                "Synthetic Bank".into(),
                54_321,
            ))),
        })),
    );
    // The stored 1099-INT paper prints its payer, which names the source.
    draft.income.last_mut().unwrap().label = "Synthetic Bank".into();
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let original = session.draft.income.clone();
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let mut added = documents.common.wages.documents[0].clone();
    added.id = 0;
    added.document = crate::WageDocument::W2(wage_document(w2(200_000)));
    documents.common.wages.documents.push(added);
    accepted(&send(&mut session, request));
    assert_eq!(&session.draft.income[..original.len()], original.as_slice());
    let added = session.draft.income.last().unwrap().id;
    assert!(added > 0 && original.iter().all(|entry| entry.id != added));
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    assert_eq!(documents.common.wages.documents[1].id, added);
    let crate::WageDocument::W2(form) = &mut documents.common.wages.documents[1].document else {
        unreachable!()
    };
    form.withholding = Some(0);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income.last().unwrap().id, added);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.common.wages.documents.retain(|row| row.id != added);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income, original);
    let saved = session.draft.clone();
    let mut session = restart(saved.clone());
    let request = open_document_center(&mut session);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft, saved);
}

#[test]
fn center_does_not_reactivate_an_unticked_topic_record() {
    // A business record whose box is unchecked.
    let mut draft = complete(FederalStatus::Single);
    // The first center save must not adopt an unrelated unticked record.
    draft.finished_sections.retain(|section| *section != TaxSection::Documents);
    add(
        &mut draft,
        income(document_income(&IncomeSource::Record {
            value: crate::IncomeRecord::Business(business(100_000)),
        })),
    );
    let unrelated = draft.income.last().unwrap().id;
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let request = open_document_center(&mut session);
    let before = session.draft.clone();
    let reply = send(&mut session, request);
    accepted(&reply);
    let warnings = match reply {
        Reply::Edit(
            P::SaveIncomes { warnings, .. }
            | P::SaveDocuments { warnings, .. }
            | P::SaveWagesAnswers { warnings, .. },
        ) => warnings,
        other => panic!("source Save must disclose the removal on its returned editor: {other:?}"),
    };
    assert!(!warnings.is_empty(), "the unticked record removal must be disclosed");
    assert_ne!(session.draft, before);
    assert_eq!(session.draft.checklist.activities & ActivityTopics::BUSINESS.0, 0);
    assert!(session.draft.income.iter().all(|entry| entry.id != unrelated));
    assert!(
        session.draft.income.iter().any(|entry| matches!(entry.income, FederalIncome::Wage { .. }))
    );
}

#[test]
fn center_saves_printed_option_and_student_loan_documents_with_native_owners() {
    let mut draft = complete(FederalStatus::Single);
    name_accepted_income_documents(&mut draft);
    let recipient = fixture_document_names(&draft, -1).unwrap().recipient;
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, allowed_community, .. } = &mut request else {
        unreachable!()
    };
    documents.common.student_loans.documents.push(crate::StudentLoanDocument {
        id: 0,
        form: crate::Form1098EPaper {
            calendar_year: 2026,
            borrower: recipient,
            box_1: 50_000,
            ..Default::default()
        },
    });
    documents.common.investment_sales.documents.push(crate::InvestmentSaleDocumentRow {
        id: 0,
        owner: crate::DocumentOwner::Yours,
        document: crate::InvestmentSaleDocument::Form3921(form_3921(Form3921More::default())),
        community: CommunityShare::NotNeeded,
        refund_management: None,
        allowed_community: allowed_community.clone(),
        investment_caps: None,
        preview: None,
        refund_management_needed: None,
    });
    let request = fixture_blank_owner_inputs(&session.draft, request);
    let before = session.draft.clone();
    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            request.clone()
        ),
        Reply::Response(P::Review { .. })
    ));
    assert_eq!(session.draft, before);
    let desired = page_documents(&request);
    accepted(&send(&mut session, request));
    crate::tests::support::client::income_pages::save_information_return_answers(
        &mut session,
        &desired,
    );
    assert_eq!(
        session.draft.adjustments.as_ref().unwrap().student_loan.forms[0].document.box_1,
        50_000
    );
    assert_eq!(session.draft.screening.as_ref().unwrap().iso_exercises.len(), 1);
    let result = supported(&session.draft);
    assert_eq!(result.tax, Some(4_404_400));
    assert_eq!(result.balance, Some(3_754_400));
    let mut session = restart(session.draft);
    let request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = request else { unreachable!() };
    assert_eq!(documents.common.student_loans.documents[0].form.box_1, 50_000);
    assert!(matches!(
        documents.common.investment_sales.documents[0].document,
        crate::InvestmentSaleDocument::Form3921(_)
    ));
}

#[test]
fn center_unsaved_row_failure_keeps_its_actual_category_and_position() {
    let mut draft = complete(FederalStatus::Single);
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let mut row = crate::tests::support::builders::native_income::center_wage_row(
        100_000,
        documents.common.wages.documents[0].allowed_community.clone(),
    );
    let crate::WageDocument::W2(form) = &mut row.document else { unreachable!() };
    form.withholding = Some(form.wages.unwrap_or(0) + 1);
    documents.common.wages.documents.push(row);
    let before = session.draft.clone();
    let Reply::Response(P::Problem { message, field }) = send(&mut session, request) else {
        panic!("invalid W-2 was accepted")
    };
    assert_eq!(message, libertas::libertas_formatted_text("W2_WITHHOLDING_ABOVE_WAGES", &[]));
    assert_eq!(field.as_deref(), Some("[documents/common/wages/documents/1/document/withholding]"));
    assert_eq!(session.draft, before);
}

#[test]
fn center_saved_native_amount_is_available_without_mutating_the_original() {
    let mut draft = complete(FederalStatus::Single);
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let P::SaveDocuments { documents, .. } = open_document_center(&mut session) else {
        unreachable!()
    };
    let row = documents.common.wages.documents.into_iter().next().unwrap();
    let before = session.draft.clone();
    assert_eq!(
        crate::tests::support::client::income_pages::saved_document_amount(
            &mut session,
            IncomeSource::Wage { value: row.document }
        ),
        Some(6_000_000)
    );
    assert_eq!(session.draft, before);
}

#[test]
fn center_refuses_a_second_audit_statement_without_overwriting_the_first() {
    let mut draft = complete(FederalStatus::Single);
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let audit = crate::CreditDocument::PartnershipAudit(alloc::boxed::Box::new(
        crate::tests::support::builders::audit_row(bba_interest_increase()),
    ));
    documents.other.credits.documents.extend([audit.clone(), audit]);
    let before = session.draft.clone();
    let Reply::Response(P::Problem { message, field }) = send(&mut session, request) else {
        panic!("multiple audit statements were accepted")
    };
    assert_eq!(message, libertas::libertas_formatted_text("ISSUE_BBA_AUDIT_SCOPE", &[]));
    assert_eq!(field.as_deref(), Some("[documents/other/credits/documents/1]"));
    assert_eq!(session.draft, before);
}

#[test]
fn fresh_household_can_open_and_add_without_unrelated_topic_answers() {
    let household = complete(FederalStatus::Single);
    let mut draft = FederalDraft::empty("synthetic-center-household".into());
    draft.setup = household.setup;
    draft.people = household.people;
    draft.finished_sections.push(TaxSection::Children);
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, allowed_community, .. } = &mut request else {
        unreachable!()
    };
    assert!(documents.common.wages.documents.is_empty());
    let mut row = crate::tests::support::builders::native_income::center_wage_row(
        1_200_000,
        allowed_community.clone(),
    );
    // The W-2 prints the filer as its employee (box e).
    let crate::WageDocument::W2(form) = &mut row.document else { unreachable!() };
    form.employee = fixture_document_names(&session.draft, -1).unwrap().recipient;
    documents.common.wages.documents.push(row);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income.len(), 1);
    assert!(
        matches!(&session.draft.income[0].income, FederalIncome::Wage { data: crate::WageSource::W2(data) } if data.form.wages == Some(1_200_000))
    );
}

#[test]
fn saved_center_preserves_a_document_when_its_owner_leaves_the_household() {
    let mut draft = complete(FederalStatus::Joint);
    draft.people.as_mut().unwrap().spouse.as_mut().unwrap().first_name = "Jordan".into();
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let crate::WageDocument::W2(form) = &mut documents.common.wages.documents[0].document else {
        unreachable!()
    };
    form.employee = accepted_document_names(&session.draft, Owner::Spouse).recipient;
    accepted(&send(&mut session, request));
    let source = session.draft.income[0].clone();
    let request = save_setup(&session.draft, ReturnSetup { filing: Filing::Single });
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income[0], source);
    assert!(session.draft.finished_sections.contains(&TaxSection::Documents));
    assert!(!session.draft.finished_sections.contains(&TaxSection::Wages));
    // The spouse's W-2 has no owner on a single return: the Document center
    // offers it to you.
    use crate::tests::support::builders::document_owners_mut;
    let mut resumed = restart(session.draft.clone());
    let reply = send(&mut resumed, P::OpenInterview { user: TEST_USER });
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { review, .. } = &mut request else { unreachable!() };
    *review = false;
    accepted(&send(&mut session, request));
    let mut retained = source.clone();
    retained.printed.as_mut().unwrap().set_assignment(Default::default());
    retained.owner = crate::DocumentOwner::Unknown;
    assert_eq!(session.draft.income[0], retained);
    let Reply::Edit(mut wages) =
        crate::tests::cross::interview::from_review(&mut session, TaxSection::Documents)
    else {
        panic!("expected the document page")
    };
    let owners = document_owners_mut(&mut wages).unwrap();
    assert_eq!(owners.iter().map(|view| view.person).collect::<Vec<_>>(), vec![-1]);
    assert_eq!(
        owners[0].choices.iter().map(|choice| choice.id).collect::<Vec<_>>(),
        vec![source.id]
    );
}

#[test]
fn household_change_keeps_education_paper_and_returns_ineligible_owner_to_unknown() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let spouse = documents
        .common
        .education
        .people
        .iter()
        .find(|person| crate::Message::decode(&person.label).key == "PERSON_SPOUSE")
        .unwrap()
        .id;
    let mut statement = student(spouse, 300_000);
    statement.form.student = fixture_document_names(&session.draft, spouse).unwrap().recipient;
    documents.common.education.students.push(statement);
    accepted(&send(&mut session, request));
    let before = session.draft.clone();
    let request = save_setup(&session.draft, ReturnSetup { filing: Filing::Single });
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.setup.as_ref().unwrap().status, FilingChoice::Single);
    assert_eq!(
        session.draft.credits.as_ref().unwrap().education,
        before.credits.as_ref().unwrap().education
    );
    assert!(session.draft.valid_stored());
    let incomplete = estimate(&session.draft);
    assert_eq!(incomplete.state, ResultState::Incomplete);
    assert_eq!(incomplete.tax, None);
    let request = open_document_center(&mut session);
    accepted(&send(&mut session, request));
    let credits = session.draft.credits.as_ref().unwrap();
    assert_eq!(credits.education[0].student, 0);
    assert_eq!(credits.education[0].assigned, crate::DocumentRecipient::Unknown);
    assert_eq!(credits.education[0].form, before.credits.as_ref().unwrap().education[0].form);
    assert!(credits.students.is_empty());
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    assert_eq!(documents.common.education.students[0].student_id, 0);
    documents.common.education.students.clear();
    accepted(&send(&mut session, request));
    assert!(session.draft.credits.as_ref().unwrap().education.is_empty());
    assert_eq!(estimate(&session.draft).state, ResultState::EstimatedForSupportedScope);
}

#[test]
fn injured_spouse_owns_management_and_other_topic_saves_remain_available() {
    use crate::document_center::InjuredSpouseDocumentAnswer as I;
    use crate::tests::support::builders::injured_credit::r8_employee_withholding::fixture;
    use crate::tests::support::client::injured_credit::r8_employee_withholding::employee_id;
    use crate::tests::support::client::native_editor;
    let mut draft = fixture(Owner::Taxpayer, Owner::Taxpayer, false, true);
    let id = employee_id(&draft);
    draft.income.iter_mut().find(|entry| entry.id == id).unwrap().refund_management = None;
    name_accepted_income_documents(&mut draft);
    let mut session = restart(draft);
    let center = open_document_center(&mut session);
    accepted(&send(&mut session, center));
    let wage_page = open_topic(&mut session, TaxSection::Wages);
    let P::SaveWagesAnswers { followups, .. } = &wage_page else { unreachable!() };
    assert!(followups.iter().all(|row| matches!(
        row.answer,
        crate::document_center::WagesDocumentAnswer::Community { .. }
    )));
    accepted(&send(&mut session, wage_page));
    let (pending, _, _, _) = review_state(&mut session);
    assert!(pending.iter().any(|row| row.section == TaxSection::InjuredSpouse
        && crate::Message::decode(&row.item).key == "DOCUMENT_PENDING_ANSWER"));
    let reply = from_review(&mut session, TaxSection::InjuredSpouse);
    let Reply::Edit(mut topic) = native_editor(reply, &[]) else { unreachable!() };
    let P::SaveInjuredSpouse { followups, .. } = &mut topic else { unreachable!() };
    let index = followups
        .iter()
        .position(|row| row.key == alloc::format!("income/{id}/management"))
        .unwrap();
    assert!(matches!(followups[index].answer, I::Management { value: None }));
    let path: alloc::string::String = libertas::SchemaPath::root()
        .field("followups")
        .key(followups[index].key.as_str())
        .field("answer")
        .field("value")
        .into();
    let before = session.draft.clone();
    let Reply::Response(P::Problem { message, field }) = send(&mut session, topic.clone()) else {
        panic!("management answer is required on Injured spouse")
    };
    assert_eq!(
        message,
        libertas::libertas_formatted_text("INJURED_SPOUSE_WITHHOLDING_MANAGEMENT_NEEDED", &[])
    );
    assert_eq!(field, Some(path));
    assert_eq!(session.draft, before);
    let P::SaveInjuredSpouse { followups, .. } = &mut topic else { unreachable!() };
    followups[index].answer = I::Management { value: Some(crate::RefundManagement::Joint) };
    accepted(&send(&mut session, topic));
    let mut session = restart(session.draft);
    assert_eq!(
        session.draft.income.iter().find(|entry| entry.id == id).unwrap().refund_management,
        Some(crate::RefundManagement::Joint)
    );
    let reply = from_review(&mut session, TaxSection::InjuredSpouse);
    let Reply::Edit(topic) = native_editor(reply, &[]) else { unreachable!() };
    let P::SaveInjuredSpouse { followups, .. } = &topic else { unreachable!() };
    assert!(followups.iter().any(|row| row.key == alloc::format!("income/{id}/management")
        && matches!(row.answer, I::Management { value: Some(crate::RefundManagement::Joint) })));
    accepted(&send(&mut session, topic));
}

#[test]
fn injured_community_context_changes_refresh_the_ordinary_question_construction() {
    use crate::tests::support::builders::injured_credit::r8_employee_withholding::fixture;
    use crate::tests::support::client::native_editor;
    let mut draft = fixture(Owner::Taxpayer, Owner::Taxpayer, false, false);
    name_accepted_income_documents(&mut draft);
    let mut session = restart(draft);
    let reply = from_review(&mut session, TaxSection::InjuredSpouse);
    let Reply::Edit(mut topic) = native_editor(reply, &[]) else { unreachable!() };
    let P::SaveInjuredSpouse { value, followups, .. } = &mut topic else { unreachable!() };
    assert!(followups.is_empty());
    let texas = value.states.iter().find(|state| status_text(&state.label) == "Texas").unwrap().id;
    value.debt = OffsetDebt::FederalTax;
    value.community =
        InjuredCommunity::Yes { states: vec![texas], debt_facts: InjuredDebtFacts::BeforeMarriage };
    let before = session.draft.clone();
    let Reply::Response(P::BeginInjuredSpouse { followups, value, .. }) = send(&mut session, topic)
    else {
        panic!("changed community facts must rebuild the ordinary editor")
    };
    assert_eq!(session.draft, before);
    assert!(
        matches!(value.community, InjuredCommunity::Yes { ref states, .. } if states == &[texas])
    );
    assert!(!followups.is_empty());
    for row in followups {
        assert!(row.key.ends_with("/management"));
        assert!(matches!(
            row.answer,
            crate::InjuredSpouseDocumentAnswer::Management { value: None }
        ));
        let id = row.key.split('/').nth(1).unwrap().parse::<i64>().unwrap();
        assert!(session.draft.income.iter().any(|entry| entry.id == id));
    }
    let wages = open_topic(&mut session, TaxSection::Wages);
    accepted(&send(&mut session, wages));
}

#[test]
fn long_term_care_topic_saves_its_records_and_source_answers_across_restart() {
    let (mut draft, id, paper) = ltc_identity_fixture();
    let FederalIncome::LtcBenefits { data } =
        &mut draft.income.iter_mut().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    data.benefit_origin = None;
    data.accelerated_origin = None;
    data.contract = LtcContract::Unknown;
    data.insured = None;
    let insured_id = draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::LtcInsured { .. }))
        .unwrap()
        .id;
    let mut session = restart(draft);
    let center = save_documents(
        &mut session,
        vec![document(
            id,
            IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(paper) },
        )],
    );
    assert!(matches!(center, P::SaveDocuments { .. }));
    accepted(&send(&mut session, center));
    let other = open_topic(&mut session, TaxSection::OtherIncome);
    let P::SaveIncomes { followups, documents, .. } = other else { unreachable!() };
    assert!(followups.iter().all(|row| !row.key.starts_with(&alloc::format!("income/{id}/"))));
    assert!(
        documents
            .documents
            .iter()
            .all(|row| !matches!(row.document, crate::IncomeRecord::LtcInsured(_)))
    );
    let (pending, _, _, _) = review_state(&mut session);
    assert!(pending.iter().any(|row| row.section == TaxSection::LongTermCareBenefits
        && crate::Message::decode(&row.item).key == "DOCUMENT_PENDING_ANSWER"));
    let mut topic = open_topic(&mut session, TaxSection::LongTermCareBenefits);
    let P::SaveIncomes { documents, followups, ltc_insured, .. } = &mut topic else {
        unreachable!()
    };
    assert!(ltc_insured.iter().any(|row| row.id == insured_id));
    let record = documents.documents.iter_mut().find(|row| row.id == insured_id).unwrap();
    let crate::IncomeRecord::LtcInsured(record) = &mut record.document else { unreachable!() };
    let LtcPeriodsInput::One { services, .. } = &mut record.periods else { unreachable!() };
    *services = 1_100_000;
    // Origin, contract, insured and the statement interest beside the paper.
    assert_eq!(followups.len(), 4);
    for row in followups {
        assert!(row.key.starts_with(&alloc::format!("income/{id}/")));
        match &mut row.answer {
            crate::DocumentAnswer::LtcBenefitOrigin { value } => {
                *value = Some(LtcPaymentOrigin::Insurance)
            }
            crate::DocumentAnswer::LtcContract { value } => {
                *value = LtcContractPage::Qualified {
                    qualification: CareContractQualification::Qualified,
                }
            }
            crate::DocumentAnswer::LtcInsured { value } => *value = Some(insured_id),
            crate::DocumentAnswer::LtcInterest { .. } => {}
            _ => panic!("only existing LTC questions belong here"),
        }
    }
    accepted(&send(&mut session, topic));
    let saved = session.draft.clone();
    session = restart(saved.clone());
    let topic = open_topic(&mut session, TaxSection::LongTermCareBenefits);
    let P::SaveIncomes { documents, followups, .. } = &topic else { unreachable!() };
    assert!(documents.documents.iter().any(|row| row.id == insured_id
        && matches!(
            row.document,
            crate::IncomeRecord::LtcInsured(LtcInsured {
                periods: LtcPeriodsInput::One { services: 1_100_000, .. },
                ..
            })
        )));
    assert!(followups.iter().any(|row| matches!(row.answer, crate::DocumentAnswer::LtcInsured { value: Some(value) } if value == insured_id)));
    accepted(&send(&mut session, topic));
    assert_eq!(session.draft, saved);
}

#[test]
fn distribution_only_archer_and_medicare_papers_use_the_typed_health_topic() {
    use crate::document_center::HsaDocumentAnswer as H;
    use crate::tests::support::client::income_pages::{information_return_answers, sa_document};
    for source in [archer_msa(1_000), medicare_msa(1_000, MedicareMsaPrior::None)] {
        let FederalIncome::MedicalDistribution { data } = source else { unreachable!() };
        let desired_answers =
            (data.payment.clone(), data.medicare_from_spouse, data.medicare_prior.clone());
        let mut session = restart(complete(FederalStatus::Single));
        assert_eq!(session.draft.checklist.paid & PaidTopics::HSA.0, 0);
        let desired = vec![document(
            0,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Sa1099(sa_document(data)),
            },
        )];
        let center = save_documents(&mut session, desired.clone());
        assert!(matches!(center, P::SaveDocuments { .. }));
        accepted(&send(&mut session, center));
        let id = session.draft.income.last().unwrap().id;
        let other = open_topic(&mut session, TaxSection::OtherIncome);
        let P::SaveIncomes { followups, .. } = other else { unreachable!() };
        assert!(followups.iter().all(|row| !row.key.starts_with(&alloc::format!("income/{id}/"))));
        let (pending, _, _, _) = review_state(&mut session);
        assert!(pending.iter().any(|row| row.section == TaxSection::Hsa
            && crate::Message::decode(&row.item).key == "DOCUMENT_PENDING_ANSWER"));
        let topic = information_return_answers(&mut session, &desired);
        let P::SaveHsa { value, followups, .. } = &topic else { unreachable!() };
        assert!(value.accounts.is_empty());
        assert!(
            followups.iter().any(|row| matches!(row.answer, H::MedicalPayment { value: Some(_) }))
        );
        assert_eq!(followups.len(), if desired_answers.1.is_some() { 3 } else { 1 });
        accepted(&send(&mut session, topic));
        session = restart(session.draft);
        assert_eq!(session.draft.checklist.paid & PaidTopics::HSA.0, 0);
        assert!(session.draft.adjustments.as_ref().unwrap().hsa.is_empty());
        let FederalIncome::MedicalDistribution { data } =
            &session.draft.income.iter().find(|row| row.id == id).unwrap().income
        else {
            unreachable!()
        };
        assert_eq!(
            (data.payment.clone(), data.medicare_from_spouse, data.medicare_prior.clone()),
            desired_answers
        );
        let Reply::Edit(topic) = from_review(&mut session, TaxSection::Hsa) else { unreachable!() };
        let P::SaveHsa { followups, .. } = &topic else { unreachable!() };
        assert!(followups.iter().any(
            |row| matches!(&row.answer, H::MedicalPayment { value } if value == &desired_answers.0)
        ));
        accepted(&send(&mut session, topic));
    }
}

#[test]
fn education_document_followups_keep_the_named_paper_and_complete_typed_answers_together() {
    use crate::document_center::EducationDocumentAnswer as E;
    use crate::tests::support::client::income_pages::save_documents;
    let (mut session, child) = crate::tests::cross::interview::education_session();
    let mut source = crate::tests::cross::interview::q1099_page(
        100_000,
        EducationAccount::Coverdell { amounts: CoverdellAmounts::ValueOnly { value: 500_000 } },
        crate::tests::cross::interview::plain_withdrawal(),
        Q1099Beneficiary::Student { person: child },
    );
    source.form.payer_trustee = crate::PrintedParty::from_name("Synthetic Education Trust".into());
    let paper = source.form;
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Q1099(paper) },
        )],
    );
    accepted(&send(&mut session, request));
    let entry = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::EducationDistribution { .. }))
        .unwrap();
    let id = entry.id;
    let FederalIncome::EducationDistribution { data } = &entry.income else { unreachable!() };
    let paper = data.form.clone();
    let P::SaveIncomes { followups, .. } = open_topic(&mut session, TaxSection::OtherIncome) else {
        unreachable!()
    };
    assert!(followups.iter().all(|row| !row.key.starts_with(&alloc::format!("income/{id}/"))));
    let (pending, _, _, _) = review_state(&mut session);
    assert!(pending.iter().any(|row| row.section == TaxSection::Education
        && crate::Message::decode(&row.item).key == "DOCUMENT_PENDING_ANSWER"
        && status_text(&row.item).contains("Synthetic Education Trust")));

    let Reply::Edit(mut request) = from_review(&mut session, TaxSection::Education) else {
        panic!("expected Education credits")
    };
    let P::SaveEducation { followups, .. } = &mut request else { unreachable!() };
    assert_eq!(followups.len(), 3);
    for row in followups {
        assert!(status_text(&row.prompt).contains(&alloc::format!(
            "1099-Q from Synthetic Education Trust, printed for {}",
            paper.recipient.name
        )));
        match &mut row.answer {
            E::EducationBeneficiary { value } => {
                *value = Some(Q1099Beneficiary::Student { person: child });
            }
            E::CoverdellRecords { opening, contributions, late_rollovers } => {
                *opening = Some(100_000);
                *contributions = 25_000;
                *late_rollovers = Some(0);
            }
            E::EducationRedeposited { value } => *value = 0,
            _ => panic!("unexpected question"),
        }
    }
    accepted(&send(&mut session, request));
    let mut session = restart(session.draft);
    let FederalIncome::EducationDistribution { data } =
        &session.draft.income.iter().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    assert_eq!(data.form, paper);
    assert_eq!(
        data.beneficiary,
        Some(crate::FederalEducationBeneficiary::Student { person: child })
    );
    assert_eq!(
        (data.coverdell_basis_start, data.coverdell_contributions, data.late_rollovers),
        (Some(100_000), 25_000, Some(0))
    );
    let Reply::Edit(request) = from_review(&mut session, TaxSection::Education) else {
        panic!("expected Education credits")
    };
    let P::SaveEducation { followups, .. } = request else { unreachable!() };
    assert!(followups.iter().any(|row| matches!(
        row.answer,
        E::CoverdellRecords {
            opening: Some(100_000),
            contributions: 25_000,
            late_rollovers: Some(0)
        }
    )));
}

#[test]
fn education_topic_preserves_exact_annual_answers_and_center_keeps_only_printed_facts() {
    let mut draft = complete(FederalStatus::Single);
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    for half_time in [false, true] {
        for graduate in [false, true] {
            for used_up in [false, true] {
                let mut request = open_document_center(&mut session);
                let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
                let mut source = student(-1, 612_345);
                source.id = session
                    .draft
                    .credits
                    .as_ref()
                    .unwrap()
                    .education
                    .first()
                    .map_or(0, |row| row.id);
                source.form.student =
                    accepted_document_names(&session.draft, Owner::Taxpayer).recipient;
                source.form.box_5 = 123_456;
                source.form.box_8 = half_time;
                source.form.box_9 = graduate;
                let more = StudentMore {
                    books_from_school: 23_456,
                    other_books: 12_345,
                    refunds: 1_234,
                    insurance_refund_current_expenses: crate::SourceCents::NotReported,
                    tax_free_529: 2_345,
                    scholarship_in_income: ScholarshipInclusion::Amount { included: 3_456 },
                    american_opportunity_used_up: Some(used_up),
                };
                documents.common.education.students = vec![source.clone()];
                accepted(&send(&mut session, request));
                let expected_method = if half_time && !graduate && !used_up {
                    EducationChoice::AmericanOpportunity
                } else {
                    EducationChoice::LifetimeLearning
                };
                accepted(&crate::tests::cross::interview::save_education_answers(
                    &mut session,
                    vec![EducationStudent {
                        student: -1,
                        method: Some(expected_method),
                        more: more.clone(),
                    }],
                ));
                assert_eq!(session.draft.credits.as_ref().unwrap().students[0].more, more);
                assert_eq!(
                    session.draft.credits.as_ref().unwrap().students[0].method,
                    Some(expected_method)
                );
                source.id = session.draft.credits.as_ref().unwrap().education[0].id;
                assert!(source.id > 0);
                let saved = session.draft.clone();
                session = restart(saved.clone());
                let request = open_document_center(&mut session);
                let P::SaveDocuments { documents, .. } = &request else { unreachable!() };
                assert_eq!(documents.common.education.students, vec![source]);
                accepted(&send(&mut session, request));
                assert_eq!(session.draft, saved);
            }
        }
    }
}

#[test]
fn center_nonincome_sources_retain_payloads_and_assigned_ids_across_restart() {
    use crate::tests::cross::interview::{all_year, marketplace_fixture_page};
    use crate::tests::support::builders::native_income::first_lender_mortgage;

    let mut draft = complete(FederalStatus::Single);
    draft.credits.as_mut().unwrap().marketplace = MarketplaceClaim::Yes {
        facts: alloc::boxed::Box::new(marketplace_fixture_page(vec![]).into()),
    };
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let mut mortgage = first_lender_mortgage(
        123_456,
        12_345_678,
        AverageBalance::Statements { average: 11_234_567 },
    );
    let names = accepted_document_names(&session.draft, Owner::Taxpayer);
    mortgage.borrower = names.recipient.clone();
    mortgage.lender = names.issuer.clone().unwrap_or_default();
    mortgage.more.property = "Synthetic first home".into();
    mortgage.refund = 12_345;
    mortgage.more.refund_origin = Some(MortgageRefundOrigin::NotDeducted);
    mortgage.insurance = 1_234;
    mortgage.points = 23_456;
    documents.common.mortgages.documents.push(mortgage_document(mortgage.clone()));
    let mut no_refund = mortgage.clone();
    no_refund.refund = 0;
    no_refund.more.property = "Synthetic other loan".into();
    documents.common.mortgages.documents.push(mortgage_document(no_refund.clone()));
    let mut forms = vec![box_1(12_345), box_2(23_456, 3_456)];
    for form in &mut forms {
        form.document.borrower = names.recipient.clone();
        form.document.lender = names.issuer.clone().unwrap_or_default();
    }
    documents.common.student_loans.documents = student_documents(&forms);
    let mut vehicle = vehicle_loan("Synthetic vehicle lender", 56_789);
    vehicle.document.payer_of_record = names.recipient.clone();
    vehicle.business = 1_234;
    documents.other.vehicle_loans.documents.push(vehicle_document(vehicle.clone()));
    let mut policy = all_year("Synthetic insurer", 50_123, 60_234, 40_345);
    for month in 1..=12 {
        policy.value.month_mut(month).benchmark = None;
    }
    policy.value.recipient = (&names.recipient).into();
    policy.value.policy_number = "Exact-source-00042".into();
    documents
        .common
        .marketplace
        .documents
        .push(crate::Form1095ADocument { id: policy.id, value: policy.value.clone() });
    let mut bond = direct_tax_credit_bond(
        "SOURCE42",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        12_345,
    );
    bond.form.recipient = names.recipient.clone();
    bond.printed = Some(names.clone());
    bond.answers.prior_carry_dollars = 42;
    bond.answers.allowed_credit_dollars = Some(165);
    bond.answers.allowed_credit_confirmed = Some(true);
    let mut audit = bba_interest_increase();
    audit.reviewed_year_partner = names.recipient;
    audit.partnership = names.issuer.unwrap_or_default();
    documents.other.credits.documents = vec![
        crate::CreditDocument::CreditBond(alloc::boxed::Box::new(
            crate::TaxCreditBondDocument::from(bond.clone()),
        )),
        crate::CreditDocument::PartnershipAudit(alloc::boxed::Box::new(
            crate::tests::support::builders::audit_row(audit.clone()),
        )),
    ];
    accepted(&send(&mut session, request));

    for (form, saved) in
        forms.iter_mut().zip(&session.draft.adjustments.as_ref().unwrap().student_loan.forms)
    {
        form.id = saved.id;
        assert!(form.id > 0);
    }
    vehicle.id = session.draft.deductions.as_ref().unwrap().vehicle_loans[0].id;
    bond.id = session.draft.credits.as_ref().unwrap().tax_credit_bonds[0].id;
    audit.id = session.draft.credits.as_ref().unwrap().bba_audit_interest.as_ref().unwrap().id;
    let request = crate::tests::support::client::credit_documents::credit_document_answers_request(
        &mut session,
        core::slice::from_ref(&bond),
        Some(&audit),
    );
    accepted(&send(&mut session, request));
    mortgage.id = session.draft.deductions.as_ref().unwrap().mortgage[0].id;
    mortgage.more.allowed_refund_origins = vec![0, 1, 2];
    no_refund.id = session.draft.deductions.as_ref().unwrap().mortgage[1].id;
    // With no refund, its inactive origin has no source amount to describe.
    no_refund.more.refund_origin = None;
    let MarketplaceClaim::Yes { facts } = &session.draft.credits.as_ref().unwrap().marketplace
    else {
        unreachable!()
    };
    policy.id = facts.forms[0].id;
    assert!(mortgage.id > 0 && policy.id > 0 && mortgage.id != policy.id);
    accepted(&crate::tests::cross::interview::save_mortgage_answers(
        &mut session,
        &[
            (
                mortgage.more.clone(),
                crate::MortgagePointTreatment::Amortized { term_months: 360, ended: false },
            ),
            (
                no_refund.more.clone(),
                crate::MortgagePointTreatment::Amortized { term_months: 360, ended: false },
            ),
        ],
    ));
    let navigation =
        crate::tests::cross::interview::navigate(&session.draft, TaxSection::StudentLoan);
    let Reply::Edit(mut request @ P::SaveStudentLoan { .. }) = send(&mut session, navigation)
    else {
        panic!("expected student loan topic")
    };
    let P::SaveStudentLoan { value, .. } = &mut request else { unreachable!() };
    for (row, form) in value.forms.iter_mut().zip(&forms) {
        if form.document.box_2 {
            row.additional_interest = crate::StudentLoanAdditionalInterest::Checked {
                additional_interest: form.additional_interest,
            };
        }
    }
    accepted(&send(&mut session, request));
    let navigation =
        crate::tests::cross::interview::navigate(&session.draft, TaxSection::Deductions);
    let Reply::Edit(mut request @ P::SaveDeductions { .. }) = send(&mut session, navigation) else {
        panic!("expected deductions topic")
    };
    let P::SaveDeductions { value, .. } = &mut request else { unreachable!() };
    value.car_loans[0].business = vehicle.business;

    accepted(&send(&mut session, request));
    let navigation =
        crate::tests::cross::interview::navigate(&session.draft, TaxSection::Marketplace);
    let Reply::Edit(mut request @ P::SaveMarketplace { .. }) = send(&mut session, navigation)
    else {
        panic!("expected Marketplace topic")
    };
    let P::SaveMarketplace { value, .. } = &mut request else { unreachable!() };
    value.forms[0] = policy.clone();
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    session = restart(saved.clone());
    let request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &request else { unreachable!() };
    assert_eq!(
        documents.common.mortgages.documents,
        vec![mortgage_document(mortgage), mortgage_document(no_refund)]
    );
    assert_eq!(documents.common.student_loans.documents, student_documents(&forms));
    assert_eq!(documents.other.vehicle_loans.documents, vec![vehicle_document(vehicle)]);
    assert_eq!(
        documents.common.marketplace.documents,
        vec![crate::Form1095ADocument { id: policy.id, value: policy.value }]
    );
    assert_eq!(
        documents.other.credits.documents,
        vec![
            crate::CreditDocument::CreditBond(alloc::boxed::Box::new(
                crate::TaxCreditBondDocument::from(bond)
            )),
            crate::CreditDocument::PartnershipAudit(alloc::boxed::Box::new(
                crate::tests::support::builders::audit_row(audit)
            ))
        ]
    );
    accepted(&send(&mut session, request));
    assert_eq!(session.draft, saved);
}

#[test]
fn topic_document_displays_are_ignored_by_estimate_and_save() {
    let mut draft = complete(FederalStatus::Single);
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let sources = session.draft.income.clone();
    let request = open_document_center(&mut session);
    accepted(&send(&mut session, request));
    let request = open_topic(&mut session, TaxSection::Wages);
    accepted(&send(&mut session, request));
    let mut request = open_topic(&mut session, TaxSection::Wages);
    let P::SaveWagesAnswers { wages, .. } = &mut request else { unreachable!() };
    wages[0].document = crate::Message::new("SYNTHETIC_IGNORED_DOCUMENT_SUMMARY", vec![]).encode();
    let before = session.draft.clone();
    let dirty = session.dirty;
    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            estimate_documents_from_save(&request)
        ),
        Reply::Response(P::RunningEstimate { .. })
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income, sources);
    let saved = session.draft.clone();
    session = restart(saved.clone());
    let request = open_document_center(&mut session);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft, saved);
}

#[test]
fn record_request_cannot_claim_the_document_center_domain() {
    let mut draft = complete(FederalStatus::Single);
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_topic(&mut session, TaxSection::InterestDividends);
    let P::SaveIncomes { topic, .. } = &mut request else { unreachable!() };
    *topic = TaxSection::Documents;
    let before = session.draft.clone();
    let dirty = session.dirty;
    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            estimate_documents_from_save(&request)
        ),
        Reply::Response(P::Problem { .. })
    ));
    assert_eq!(session.draft, before);
    assert!(matches!(send(&mut session, request), Reply::Response(P::Problem { .. })));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
}

#[test]
fn center_is_mandatory_after_household_and_plain_w2_skips_the_topic_walk() {
    let mut session = Interview::new(TEST_USER, FederalDraft::empty("center-walk".into()));
    let reply = send(&mut session, P::OpenInterview { user: TEST_USER });
    let reply = send(&mut session, answer(reply, Filing::Single));
    assert!(matches!(reply, Reply::Response(P::BeginPeople { .. })));
    let reply = send(&mut session, answer(reply, Filing::Single));
    assert!(matches!(reply, Reply::Response(P::BeginChildren { .. })));
    let reply = send(&mut session, answer(reply, Filing::Single));
    assert!(matches!(reply, Reply::Edit(P::SaveDocuments { .. })));
    let mut request = answer(reply, Filing::Single);
    let names = accepted_document_names(&session.draft, Owner::Taxpayer);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let crate::WageDocument::W2(form) = &mut documents.common.wages.documents[0].document else {
        unreachable!()
    };
    form.employee = names.recipient;
    let reply = send(&mut session, request);
    assert!(matches!(reply, Reply::Response(P::BeginIncomeWithoutForm { .. })), "{reply:?}");
    let reply = send(&mut session, answer(reply, Filing::Single));
    assert!(matches!(reply, Reply::Response(P::BeginChecklist { .. })), "{reply:?}");
    let checklist = tick(reply, Checklist::NONE);
    let reply = crate::tests::cross::interview::send_past_less_common(&mut session, checklist);
    // The W-2's situations question is asked once on the wages page.
    let Reply::Edit(wages @ P::SaveWagesAnswers { .. }) = reply else { panic!("{reply:?}") };
    let reply = crate::tests::cross::interview::send_past_less_common(&mut session, wages);
    assert!(matches!(reply, Reply::Response(P::Review { .. })), "{reply:?}");
    let source = session.draft.income.clone();
    let request = open_topic(&mut session, TaxSection::Wages);
    let P::SaveWagesAnswers { records, wages, .. } = &request else { unreachable!() };
    assert!(records.documents.is_empty());
    assert_eq!(wages.len(), 1);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income, source);
}

#[test]
fn checklist_change_retains_sources_before_the_center_has_been_visited() {
    let mut draft = complete(FederalStatus::Single);
    draft.finished_sections.retain(|section| *section != TaxSection::Documents);
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    assert!(!session.draft.finished_sections.contains(&TaxSection::Documents));
    let sources = session.draft.income.clone();
    let financial = estimate(&session.draft);
    let reply = from_review(&mut session, TaxSection::Checklist);
    let request = tick(reply, Checklist::NONE);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income, sources);
    assert_eq!(estimate(&session.draft).return_lines, financial.return_lines);
    assert!(!session.draft.finished_sections.contains(&TaxSection::Documents));
    session = restart(session.draft.clone());
    let request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = request else { unreachable!() };
    assert_eq!(documents.common.wages.documents[0].id, sources[0].id);
    let crate::WageDocument::W2(w2) = &documents.common.wages.documents[0].document else {
        unreachable!()
    };
    let FederalIncome::Wage { data } = &sources[0].income else { unreachable!() };
    let data = FederalWage::from(data);
    assert_eq!(w2.wages, Some(data.wages));
}

#[test]
fn entered_record_keeps_its_topic_walk_when_a_document_is_also_present() {
    let mut draft = complete(FederalStatus::Single);
    draft.finished_sections.retain(|section| *section != TaxSection::Documents);
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let source = session.draft.income[0].clone();
    let request = save_topic(
        &mut session,
        TaxSection::Wages,
        vec![document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::Clergy(crate::ClergyRecord {
                    role: crate::ClergyRole::ReligiousSect,
                }),
            },
        )],
    );
    accepted(&send(&mut session, request));
    // Model an unfinished record topic alongside its already accepted document.
    session.draft.finished_sections.retain(|section| *section != TaxSection::Wages);
    let Reply::Edit(center @ P::SaveDocuments { .. }) =
        send(&mut session, P::OpenInterview { user: TEST_USER })
    else {
        unreachable!()
    };
    let reply = send(&mut session, center);
    let checklist = crate::tests::cross::interview::past_people(&mut session, reply);
    let request = tick(checklist, Checklist::NONE);
    let reply = send(&mut session, request);
    assert!(matches!(reply, Reply::Edit(P::SaveWagesAnswers { .. })), "{reply:?}");
    assert_eq!(session.draft.income.iter().find(|entry| entry.id == source.id), Some(&source));
    let request = open_topic(&mut session, TaxSection::Wages);
    let P::SaveWagesAnswers { records, .. } = request else { unreachable!() };
    assert!(matches!(records.documents[0].document, crate::WagesRecord::Clergy(_)));
}

#[test]
fn transferring_schools_preserves_each_statement_and_the_students_explicit_credit_choice() {
    let mut d = complete(FederalStatus::Single);
    let (mut first, annual) =
        native_education(-1, education_expenses(200_000), EducationChoice::LifetimeLearning);
    first.id = 10;
    first.half_time = true;
    first.graduate = false;
    let (mut second, _) =
        native_education(-1, education_expenses(300_000), EducationChoice::AmericanOpportunity);
    second.id = 11;
    second.half_time = false;
    second.graduate = true;
    let credits = d.credits.as_mut().unwrap();
    credits.education = vec![education_source(first.clone()), education_source(second.clone())];
    credits.students = vec![annual.clone()];
    credits.reconcile_education_students();
    let inputs = credits.education_inputs();
    assert_eq!(credits.education, vec![education_source(first), education_source(second)]);
    assert_eq!(credits.students, vec![annual]);
    assert_eq!(inputs[0].method, EducationChoice::LifetimeLearning);
    assert_eq!(inputs[0].expenses.tuition, 500_000);
    assert_eq!(inputs[0].sources, vec![10, 11]);
}

#[test]
fn annual_student_order_survives_source_reordering_and_prunes_only_after_last_source() {
    let mut d = complete(FederalStatus::Joint);
    let credits = d.credits.as_mut().unwrap();
    set_education(
        credits,
        vec![
            native_education(-2, education_expenses(800_000), EducationChoice::LifetimeLearning),
            native_education(-1, education_expenses(600_000), EducationChoice::LifetimeLearning),
        ],
    );
    credits.education.reverse();
    credits.reconcile_education_students();
    let inputs = credits.education_inputs();
    assert_eq!(inputs.iter().map(|row| row.student).collect::<Vec<_>>(), vec![-2, -1]);
    assert_eq!(
        inputs.iter().map(|row| row.expenses.tuition).collect::<Vec<_>>(),
        vec![800_000, 600_000]
    );
    credits.education.retain(|source| source.student != -2);
    credits.reconcile_education_students();
    assert_eq!(credits.students.iter().map(|row| row.student).collect::<Vec<_>>(), vec![-1]);
}

#[test]
fn adjustment_only_second_statement_retains_boxes_without_reducing_current_tuition() {
    let mut d = complete(FederalStatus::Single);
    let credits = d.credits.as_mut().unwrap();
    add_education(
        credits,
        native_education(-1, education_expenses(400_000), EducationChoice::AmericanOpportunity),
    );
    let (mut adjustment, _) =
        native_education(-1, education_expenses(0), EducationChoice::AmericanOpportunity);
    adjustment.id = 12;
    adjustment.prior_tuition_adjustment = crate::SourceCents::Reported { amount: 50_000 };
    adjustment.prior_scholarship_adjustment = crate::SourceCents::Reported { amount: 20_000 };
    credits.education.push(education_source(adjustment.clone()));
    credits.reconcile_education_students();
    let inputs = credits.education_inputs();
    assert_eq!(credits.students.len(), 1);
    assert_eq!(inputs[0].expenses.tuition, 400_000);
    assert_eq!(inputs[0].expenses.refunds, 0);
    assert_eq!(inputs[0].expenses.assistance, 0);
    assert_eq!(credits.education[1], education_source(adjustment));
    assert!(credits.education_recapture.is_none());
}

#[test]
fn identical_document_labels_keep_distinct_pending_questions() {
    let mut draft = complete(FederalStatus::Single);
    draft.income.clear();
    let mut form = w2(6_000_000);
    form.more.box12 = vec![Box12Entry { code: W2Code::TP, amount: 100_000 }];
    for _ in 0..2 {
        add(
            &mut draft,
            income(document_income(&IncomeSource::Wage {
                value: crate::WageDocument::W2(wage_document(form.clone())),
            })),
        );
    }
    name_accepted_income_documents(&mut draft);
    assert_ne!(draft.income[0].id, draft.income[1].id);
    let pending: Vec<_> =
        crate::interview::remaining_items(&draft, &estimate(&draft), TaxSection::Wages)
            .into_iter()
            .filter(|row| row.section == TaxSection::Wages)
            .collect();
    assert_eq!(pending.len(), 2);
    assert_eq!(pending[0].item, pending[1].item);
}

#[test]
fn marketplace_printed_corrections_keep_topic_answers_pending_and_summary_echoes_inert() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut form = all_year("Synthetic policy issuer", 50_000, 60_000, 20_000);
    form.value.recipient = (&fixture_document_names(&session.draft, -1).unwrap().recipient).into();
    let mut source = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut source else { unreachable!() };
    documents
        .common
        .marketplace
        .documents
        .push(crate::Form1095ADocument { id: form.id, value: form.value.clone() });
    accepted(&send(&mut session, source));
    assert!(session.draft.valid_stored());
    session = restart(session.draft);
    let mut topic = open_credits_topic(&mut session, TaxSection::Marketplace);
    let P::SaveMarketplace { value, .. } = &mut topic else { unreachable!() };
    let id = value.forms[0].id;
    assert!(value.forms[0].coverage.is_empty());
    value.forms[0].coverage = form.coverage.clone();
    value.forms[0].value.set_amounts(PolicyAmounts::AllYear {
        premium: 1,
        benchmark: Some(1),
        advance: 0,
    });
    accepted(&send(&mut session, topic));
    assert_eq!(stored_marketplace(&session).forms[0].form.amounts(), form.value.amounts());

    let mut source = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut source else { unreachable!() };
    documents.common.marketplace.documents[0].value.set_amounts(PolicyAmounts::ByMonth {
        months: vec![PolicyMonth {
            month: 1,
            premium: 50_000,
            benchmark: Some(60_000),
            advance: 20_000,
        }],
    });
    accepted(&send(&mut session, source));
    assert!(session.draft.valid_stored());
    assert_eq!(stored_marketplace(&session).forms[0].coverage.len(), 12);
    session = restart(session.draft);
    let Reply::Response(P::Review { remaining_items, allowed_actions, .. }) =
        send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected review")
    };
    assert!(!allowed_actions.contains(&FINISH));
    assert!(remaining_items.iter().any(|row| row.section == TaxSection::Marketplace
        && status_text(&row.item).contains("Coverage and payment details for this policy")));
    let mut topic = open_credits_topic(&mut session, TaxSection::Marketplace);
    let before = session.draft.clone();
    error(send(&mut session, topic.clone()));
    assert_eq!(session.draft, before);
    let P::SaveMarketplace { value, .. } = &mut topic else { unreachable!() };
    assert_eq!(value.forms[0].id, id);
    value.forms[0].coverage.retain(|month| month.month == 1);
    accepted(&send(&mut session, topic));
    assert!(!crate::marketplace_sources::coverage_pending(&stored_marketplace(&session).forms[0]));
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

// G-51 (Smith round 3): the 1095-A prints Jane, Alice and Bob covered January through March; the
// policy opens with those people in each of those months. A covered person the return does not
// know leaves the whole policy for the person to answer.
#[test]
fn marketplace_policy_takes_covered_people_and_months_from_the_printed_part_ii() {
    let covered = |name: &str| crate::Form1095ACoveredIndividual {
        name: name.into(),
        ssn: "".into(),
        date_of_birth: None,
        coverage_start_date: Some(20260101),
        coverage_termination_date: Some(20260331),
    };
    let run = |names: [&str; 3]| {
        let mut d = complete(FederalStatus::Joint);
        add_child(&mut d);
        add_child(&mut d);
        let people = d.people.as_mut().unwrap();
        people.taxpayer.first_name = "John".into();
        people.taxpayer.last_name = "Smith".into();
        let spouse = people.spouse.as_mut().unwrap();
        spouse.first_name = "Jane".into();
        spouse.last_name = "Smith".into();
        for (child, first) in d.dependents.iter_mut().zip(["Alice", "Bob"]) {
            child.first_name = first.into();
            child.last_name = "Smith".into();
        }
        let ids: Vec<i64> = d.dependents.iter().map(|child| child.id).collect();
        let mut session = Interview::new(TEST_USER, d);
        let mut form = all_year("Synthetic policy issuer", 50_000, 60_000, 20_000);
        form.value.recipient =
            (&fixture_document_names(&session.draft, -1).unwrap().recipient).into();
        form.value.set_amounts(PolicyAmounts::ByMonth {
            months: (1..=3)
                .map(|month| PolicyMonth {
                    month,
                    premium: 50_000,
                    benchmark: Some(60_000),
                    advance: 20_000,
                })
                .collect(),
        });
        form.value.covered_individuals = names.map(covered).to_vec();
        let mut source = open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut source else { unreachable!() };
        documents
            .common
            .marketplace
            .documents
            .push(crate::Form1095ADocument { id: form.id, value: form.value.clone() });
        accepted(&send(&mut session, source));
        (session, ids)
    };
    let (session, ids) = run(["Jane Smith", "Alice Smith", "Bob Smith"]);
    let policy = &stored_marketplace(&session).forms[0];
    let mut expected = vec![-2, ids[0], ids[1]];
    expected.sort_unstable();
    assert_eq!(
        policy.coverage,
        (1..=3)
            .map(|month| crate::MarketplacePolicyCoverage {
                month,
                people: expected.clone(),
                entry: vec![]
            })
            .collect::<Vec<_>>()
    );
    assert!(!crate::marketplace_sources::coverage_pending(policy));
    // The people picker names the people.
    let topic = open_credits_topic(
        &mut Interview::new(TEST_USER, session.draft.clone()),
        TaxSection::Marketplace,
    );
    let P::SaveMarketplace { ptc_people, .. } = topic else { unreachable!() };
    assert_eq!(ptc_people[0].label, crate::person_names::person_name("John", "Smith"));
    // A covered person who is not on the return is left for the person to answer.
    let (session, _) = run(["Jane Smith", "Alice Smith", "Chris Outsider"]);
    assert!(stored_marketplace(&session).forms[0].coverage.is_empty());
}

#[test]
fn investment_printed_corrections_retain_nominees_until_actual_topic_correction() {
    for kind in 0..3 {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let names = fixture_document_names(&session.draft, -1);
        let value = match kind {
            0 => {
                let mut value = native_int1099("Synthetic bank".into(), 100_000);
                value.recipient = names.unwrap().recipient;
                crate::InterestDividendDocument::Int1099(interest_document(value))
            }
            1 => {
                let mut value = native_oid1099("Synthetic OID payer".into(), 100_000, 0);
                value.recipient = names.unwrap().recipient;
                crate::InterestDividendDocument::Oid1099(oid_document(value))
            }
            _ => {
                let mut value = native_div1099("Synthetic fund".into(), 100_000, 0);
                value.recipient = names.unwrap().recipient;
                crate::InterestDividendDocument::Div1099(dividend_document(value))
            }
        };
        let source = save_documents(
            &mut session,
            vec![document(0, IncomeSource::InterestDividend { value })],
        );
        accepted(&send(&mut session, source));
        let id = session.draft.income.last().unwrap().id;
        let mut topic = open_topic(&mut session, TaxSection::InterestDividends);
        let P::SaveIncomes { followups, .. } = &mut topic else { unreachable!() };
        let key = alloc::format!("income/{id}/nominee");
        let row = followups.iter_mut().find(|row| row.key == key).unwrap();
        row.answer = match kind {
            0 => crate::DocumentAnswer::InterestNominee {
                value: Some(
                    crate::InterestNominee { taxable: 40_000, ..Default::default() }.into(),
                ),
            },
            1 => crate::DocumentAnswer::OidNominee {
                value: Some(crate::OidNominee { oid: 40_000, ..Default::default() }.into()),
            },
            _ => crate::DocumentAnswer::DividendNominee {
                value: Some(
                    crate::DividendNominee { ordinary: 40_000, ..Default::default() }.into(),
                ),
            },
        };
        accepted(&send(&mut session, topic));
        let mut source = open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut source else { unreachable!() };
        let row = documents
            .common
            .interest_dividends
            .documents
            .iter_mut()
            .find(|row| row.id == id)
            .unwrap();
        match &mut row.document {
            crate::InterestDividendDocument::Int1099(value) => value.interest_income = 20_000,
            crate::InterestDividendDocument::Oid1099(value) => {
                value.original_issue_discount_for_the_year = 20_000
            }
            crate::InterestDividendDocument::Div1099(value) => value.ordinary = Some(20_000),
            _ => unreachable!(),
        }
        accepted(&send(&mut session, source));
        assert!(session.draft.valid_stored());
        let entry = session.draft.income.iter().find(|entry| entry.id == id).unwrap();
        let (printed, retained) = match &entry.income {
            FederalIncome::Interest { data } => {
                let data = FederalInterest::from(data);
                (data.taxable, data.nominee.as_ref().unwrap().taxable)
            }
            FederalIncome::Oid { data } => {
                let data = FederalOid::from(data);
                (data.oid, data.nominee.as_ref().unwrap().oid)
            }
            FederalIncome::Dividend { data } => {
                let data = FederalDividend::from(data);
                (data.ordinary, data.nominee.as_ref().unwrap().ordinary)
            }
            _ => unreachable!(),
        };
        assert_eq!((printed, retained), (20_000, 40_000));
        session = restart(session.draft);
        let Reply::Response(P::Review { remaining_items, allowed_actions, .. }) =
            send(&mut session, P::ReviewAccepted { user: TEST_USER })
        else {
            panic!("expected Review")
        };
        assert!(!allowed_actions.contains(&FINISH));
        assert!(remaining_items.iter().any(|row| row.section == TaxSection::InterestDividends
            && crate::Message::decode(&row.item).key == "DOCUMENT_PENDING_ANSWER"));
        let mut topic = open_topic(&mut session, TaxSection::InterestDividends);
        let P::SaveIncomes { followups, .. } = &mut topic else { unreachable!() };
        followups.reverse();
        let index = followups.iter().position(|row| row.key == key).unwrap();
        let path = libertas::SchemaPath::root()
            .field("followups")
            .key(followups[index].key.as_str())
            .field("answer")
            .field("value");
        let path: alloc::string::String =
            if kind == 1 { path.field("value").field("oid") } else { path }.into();
        let before = session.draft.clone();
        let Reply::Response(P::Problem { field, .. }) = send(&mut session, topic.clone()) else {
            panic!("conflicting nominee must be corrected on the topic")
        };
        assert_eq!(field, Some(path));
        assert_eq!(session.draft, before);
        let P::SaveIncomes { followups, .. } = &mut topic else { unreachable!() };
        match &mut followups[index].answer {
            crate::DocumentAnswer::InterestNominee { value } => *value = None,
            crate::DocumentAnswer::OidNominee { value } => *value = None,
            crate::DocumentAnswer::DividendNominee { value } => *value = None,
            _ => unreachable!(),
        }
        accepted(&send(&mut session, topic));
        assert!(session.draft.valid_stored());
        let restored = restart(session.draft.clone());
        assert_eq!(restored.draft, session.draft);
    }
}

#[test]
fn wage_printed_correction_preserves_personal_pay_and_requires_topic_fix() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 100_000, 0);
    let FederalIncome::Wage { data: crate::WageSource::W2(data) } = &mut draft.income[0].income
    else {
        unreachable!()
    };
    data.answers.deferred_pension = W2Amount::of(60_000);
    name_accepted_income_documents(&mut draft);
    assert!(draft.valid_stored());
    let mut session = restart(draft);
    let mut source = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut source else { unreachable!() };
    let crate::WageDocument::W2(value) = &mut documents.common.wages.documents[0].document else {
        unreachable!()
    };
    value.wages = Some(50_000);
    accepted(&send(&mut session, source));
    assert!(session.draft.valid_stored());
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { unreachable!() };
    let data = FederalWage::from(data);
    assert_eq!((data.wages, data.deferred_pension), (50_000, 60_000));
    session = restart(session.draft);
    let Reply::Response(P::Review { remaining_items, allowed_actions, .. }) =
        send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected Review")
    };
    assert!(!allowed_actions.contains(&FINISH));
    assert!(remaining_items.iter().any(|row| row.section == TaxSection::Wages
        && crate::Message::decode(&row.item).key == "DOCUMENT_PENDING_ANSWER"));
    let mut topic = open_topic(&mut session, TaxSection::Wages);
    let P::SaveWagesAnswers { wages, .. } = &topic else { unreachable!() };
    let index = wages
        .iter()
        .position(|row| row.key == alloc::format!("income/{}/wage", session.draft.income[0].id))
        .unwrap();
    let path: alloc::string::String = libertas::SchemaPath::root()
        .field("wages")
        .key(wages[index].key.as_str())
        .field("answers")
        .field("deferred_pension")
        .into();
    let before = session.draft.clone();
    let Reply::Response(P::Problem { field, .. }) = send(&mut session, topic.clone()) else {
        panic!("topic must resolve the retained amount")
    };
    assert_eq!(field, Some(path));
    assert_eq!(session.draft, before);
    let P::SaveWagesAnswers { wages, .. } = &mut topic else { unreachable!() };
    wages[index].answers.deferred_pension = W2Amount::of(30_000);
    accepted(&send(&mut session, topic));
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { unreachable!() };
    let data = FederalWage::from(data);
    assert_eq!((data.wages, data.deferred_pension), (50_000, 30_000));
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

#[test]
fn miscellaneous_printed_correction_keeps_client_funds_for_topic_fix() {
    let mut draft = complete(FederalStatus::Single);
    let source = crate::Form1099Misc {
        payer: Default::default(),
        recipient: Default::default(),
        cash_tips: 0,
        tip_occupation_codes: Default::default(),
        overtime: 0,
        component_ids: Default::default(),
        rents: 0,
        royalties: 0,
        other: 0,
        other_use: None,
        withholding: 0,
        fishing: 0,
        medical: 0,
        substitute: 0,
        crop_insurance: 0,
        attorney: 100_000,
        client_funds: 60_000,
        fish_sales: 0,
        deferred: 0,
        state_details: Default::default(),
    };
    add(&mut draft, income(FederalIncome::Misc1099 { data: misc_paper(source) }));
    name_accepted_income_documents(&mut draft);
    let id = draft.income.last().unwrap().id;
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let row = documents.other.other_income.documents.iter_mut().find(|row| row.id == id).unwrap();
    let crate::OtherIncomeDocument::Form1099Misc(value) = &mut row.document else { unreachable!() };
    value.attorney = Some(40_000);
    accepted(&send(&mut session, request));
    let FederalIncome::Misc1099 { data } =
        &session.draft.income.iter().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    assert_eq!((data.form.attorney, data.client_funds), (Some(40_000), 60_000));
    assert!(session.draft.valid_stored());
    session = restart(session.draft);
    let Reply::Response(P::Review { remaining_items, allowed_actions, .. }) =
        send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected Review")
    };
    assert!(!allowed_actions.contains(&FINISH));
    assert!(remaining_items.iter().any(|row| row.section == TaxSection::OtherIncome
        && crate::Message::decode(&row.item).key == "DOCUMENT_PENDING_ANSWER"));
    let mut topic = open_topic(&mut session, TaxSection::OtherIncome);
    let P::SaveIncomes { followups, .. } = &topic else { unreachable!() };
    let index = followups
        .iter()
        .position(|row| row.key == alloc::format!("income/{id}/client_funds"))
        .unwrap();
    let path: alloc::string::String = libertas::SchemaPath::root()
        .field("followups")
        .key(followups[index].key.as_str())
        .field("answer")
        .field("value")
        .into();
    let before = session.draft.clone();
    let Reply::Response(P::Problem { field, .. }) = send(&mut session, topic.clone()) else {
        panic!("expected topic refusal")
    };
    assert_eq!(field, Some(path));
    assert_eq!(session.draft, before);
    let P::SaveIncomes { followups, .. } = &mut topic else { unreachable!() };
    followups[index].answer = crate::DocumentAnswer::AttorneyClientFunds { value: 20_000 };
    accepted(&send(&mut session, topic));
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

#[test]
fn payment_source_waits_for_actual_parts_and_retains_them_after_printed_correction() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::SelfEmployment {
                value: crate::SelfEmploymentDocument::Form1099K(k_document(crate::Form1099K {
                    payer: Default::default(),
                    recipient: Default::default(),
                    cash_tips: 0,
                    tip_occupation_codes: Default::default(),
                    gross: 100_000,
                    withholding: 0,
                    parts: vec![crate::KPart {
                        id: 0,
                        amount: 100_000,
                        belongs: Some(crate::KUse::NotIncome),
                        refunded: 0,
                    }],
                    state_details: Default::default(),
                })),
            },
        )],
    );
    accepted(&send(&mut session, request));
    let id = session.draft.income.last().unwrap().id;
    let FederalIncome::K1099 { data } = &session.draft.income.last().unwrap().income else {
        unreachable!()
    };
    assert!(data.parts.is_empty(), "hidden source answers cannot supply a personal allocation");
    assert!(session.draft.valid_stored());
    assert!(
        estimate(&session.draft)
            .issues
            .iter()
            .any(|issue| issue.display.key == "K_PARTS_NOT_GROSS")
    );
    let key = alloc::format!("income/{id}/parts");
    let request = P::SavePaymentParts {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        key: key.clone(),
        value: vec![crate::KPart {
            id: 0,
            amount: 100_000,
            belongs: Some(crate::KUse::NotIncome),
            refunded: 0,
        }],
    };
    accepted(&send(&mut session, request));
    let FederalIncome::K1099 { data } =
        &session.draft.income.iter().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    let part_id = data.parts[0].id;
    assert!(part_id > 0 && part_id != id);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let row =
        documents.common.self_employment.documents.iter_mut().find(|row| row.id == id).unwrap();
    let crate::SelfEmploymentDocument::Form1099K(value) = &mut row.document else { unreachable!() };
    value.gross = Some(80_000);
    accepted(&send(&mut session, request));
    session = restart(session.draft);
    let FederalIncome::K1099 { data } =
        &session.draft.income.iter().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    assert_eq!(
        (data.form.gross, data.parts[0].amount, data.parts[0].id),
        (Some(80_000), 100_000, part_id)
    );
    let Reply::Response(P::Review { remaining_items, allowed_actions, .. }) =
        send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected Review")
    };
    assert!(!allowed_actions.contains(&FINISH));
    assert!(remaining_items.iter().any(|row| row.section == TaxSection::SelfEmployment
        && crate::Message::decode(&row.item).key == "DOCUMENT_PENDING_ANSWER"));
    let FederalIncome::K1099 { data } =
        &session.draft.income.iter().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    let mut request = P::SavePaymentParts {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        key,
        value: data.parts.clone(),
    };
    let before = session.draft.clone();
    let Reply::Response(P::Problem { field, .. }) = send(&mut session, request.clone()) else {
        panic!("expected source composer refusal")
    };
    assert_eq!(field.as_deref(), Some("[value]"));
    assert_eq!(session.draft, before);
    let P::SavePaymentParts { value, .. } = &mut request else { unreachable!() };
    value[0].amount = 80_000;
    accepted(&send(&mut session, request));
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

#[test]
fn center_education_printed_name_match_owns_the_document_over_an_earlier_addition() {
    // A printed student matching you owns the statement; additions cannot override it.
    let draft = education_recipient_name_collision();
    let mut session = Interview::new(TEST_USER, draft);
    let request = open_document_center(&mut session);
    accepted(&send(&mut session, request));
    let source = &session.draft.credits.as_ref().unwrap().education[0];
    let id = source.id;
    assert_eq!(source.student, -1);
    assert_eq!(source.assigned, crate::DocumentRecipient::Taxpayer);
    assert_eq!(source.owner_source, Some(crate::OwnerSource::Matched));
    let mut restored = restart(session.draft);
    let mut owners = open_document_center(&mut restored);
    let P::SaveDocuments { documents, document_owners, .. } = &mut owners else { unreachable!() };
    assert_eq!(documents.common.education.students[0].student_id, -1);
    let you = document_owners.iter_mut().find(|person| person.person == -1).unwrap();
    let row = you.documents.iter().find(|row| row.document == id).unwrap();
    assert_eq!(row.owner_source, crate::OwnerSource::Matched);
    you.documents.clear();
    accepted(&send(&mut restored, owners));
    let Reply::Edit(mut education) = from_review(&mut restored, TaxSection::Education) else {
        panic!("expected education")
    };
    let P::SaveEducation { value, .. } = &mut education else { unreachable!() };
    value.students[0].method = Some(EducationChoice::LifetimeLearning);
    value.students[0].more.american_opportunity_used_up = Some(false);
    accepted(&send(&mut restored, education));
    let source = &restored.draft.credits.as_ref().unwrap().education[0];
    assert_eq!(source.student, -1);
    assert_eq!(source.owner_source, Some(crate::OwnerSource::Matched));
}

#[test]
fn person_summary_edit_targets_name_actual_papers_and_do_not_change_save_semantics() {
    let mut session = Interview::new(TEST_USER, education_recipient_name_collision());
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, document_owners, .. } = &mut request else { unreachable!() };
    let targets = [
        (documents.common.wages.documents[0].id, "[documents/common/wages/documents/0]"),
        (documents.common.education.students[0].id, "[documents/common/education/students/0]"),
    ];
    for (id, expected) in targets {
        let row = document_owners
            .iter_mut()
            .flat_map(|person| &mut person.documents)
            .find(|row| row.document == id)
            .unwrap();
        assert_eq!(row.edit_document.len(), 1);
        assert_eq!(row.edit_document[0].field.as_deref(), Some(expected));
        assert_eq!(status_text(&row.edit_document[0].action), "Edit document");
        // Navigation is output, never an instruction to save another record.
        row.edit_document.clear();
    }
    accepted(&send(&mut session, request));
    let mut restored = restart(session.draft);
    let P::SaveDocuments { documents, document_owners, .. } = open_document_center(&mut restored)
    else {
        unreachable!()
    };
    for (id, expected) in [
        (documents.common.wages.documents[0].id, "[documents/common/wages/documents/0]"),
        (documents.common.education.students[0].id, "[documents/common/education/students/0]"),
    ] {
        let row = document_owners
            .iter()
            .flat_map(|person| &person.documents)
            .find(|row| row.document == id)
            .unwrap();
        assert_eq!(row.edit_document[0].field.as_deref(), Some(expected));
    }
}

#[test]
fn an_empty_paper_in_ordinary_initialization_is_refused_until_filled_or_deleted() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    // The hand entry a failed import leaves: the kind selected, the payload initialized.
    documents.other.other_income.documents.push(crate::OtherIncomeDocumentRow {
        id: 0,
        owner: crate::DocumentOwner::Unknown,
        document: crate::OtherIncomeDocument::Form1099Misc(Default::default()),
        community: crate::CommunityShare::NotNeeded,
        refund_management: None,
        allowed_community: Vec::new(),
        preview: None,
        investment_caps: None,
        refund_management_needed: None,
    });
    let before = session.draft.clone();
    let Reply::Response(P::Problem { message, field }) = send(&mut session, request.clone()) else {
        panic!("an empty paper was saved without a question")
    };
    assert_eq!(message, libertas::libertas_formatted_text("ISSUE_PAPER_NOTHING_ENTERED", &[]));
    assert_eq!(field.as_deref(), Some("[documents/other/other_income/documents/0/document]"));
    assert_eq!(session.draft, before);
    // Deleting the item lets the Document center save.
    let mut deleted = request.clone();
    let P::SaveDocuments { documents, .. } = &mut deleted else { unreachable!() };
    documents.other.other_income.documents.clear();
    accepted(&send(&mut session, deleted));
    // Entering the printed boxes lets it save.
    let mut filled = request;
    let P::SaveDocuments { documents, .. } = &mut filled else { unreachable!() };
    let crate::OtherIncomeDocument::Form1099Misc(form) =
        &mut documents.other.other_income.documents[0].document
    else {
        unreachable!()
    };
    form.crop_insurance = Some(250_000);
    accepted(&send(&mut session, filled));
    assert!(
        session
            .draft
            .income
            .iter()
            .any(|entry| matches!(entry.income, FederalIncome::Misc1099 { .. }))
    );
}

#[test]
fn an_empty_typed_paper_in_ordinary_initialization_is_refused_until_filled_or_deleted() {
    // The paper as it opens blank: every field initialized to its zero encoding.
    fn blank<T: libertas::AvroDecode>() -> T {
        T::avro_decode(&[0u8; 1 << 16], &mut 0).unwrap()
    }
    let nothing = libertas::libertas_formatted_text("ISSUE_PAPER_NOTHING_ENTERED", &[]);
    // Each typed admission path: the blank item is added, then removed again.
    type Edit = fn(&mut crate::DocumentCenter, bool);
    let paths: [(&str, Edit); 7] = [
        ("[documents/common/mortgages/documents/0/form]", |d, add| {
            d.common.mortgages.documents.clear();
            if add {
                d.common.mortgages.documents.push(crate::Form1098Document {
                    id: 0,
                    form: blank(),
                    lender: crate::MortgageLender::Institution,
                });
            }
        }),
        ("[documents/common/student_loans/documents/0/form]", |d, add| {
            d.common.student_loans.documents.clear();
            if add {
                d.common
                    .student_loans
                    .documents
                    .push(crate::StudentLoanDocument { id: 0, form: blank() });
            }
        }),
        ("[documents/common/education/students/0/form]", |d, add| {
            d.common.education.students.clear();
            if add {
                d.common.education.students.push(crate::StudentCreditDocument {
                    id: 0,
                    student_id: 0,
                    form: blank(),
                });
            }
        }),
        ("[documents/common/marketplace/documents/0/value]", |d, add| {
            d.common.marketplace.documents.clear();
            if add {
                d.common
                    .marketplace
                    .documents
                    .push(crate::Form1095ADocument { id: 0, value: blank() });
            }
        }),
        ("[documents/other/vehicle_loans/documents/0/form]", |d, add| {
            d.other.vehicle_loans.documents.clear();
            if add {
                d.other
                    .vehicle_loans
                    .documents
                    .push(crate::Form1098VliDocument { id: 0, form: blank() });
            }
        }),
        ("[documents/other/vehicle_gifts/documents/0/form]", |d, add| {
            d.other.vehicle_gifts.documents.clear();
            if add {
                d.other.vehicle_gifts.documents.push(crate::Form1098CDocument {
                    id: 0,
                    form: blank(),
                    assigned: crate::DocumentRecipient::Unknown,
                    owner_source: None,
                    joint_share: None,
                });
            }
        }),
        ("[documents/other/credits/documents/0/form]", |d, add| {
            d.other.credits.documents.clear();
            if add {
                d.other.credits.documents.push(crate::CreditDocument::CreditBond(
                    alloc::boxed::Box::new(crate::TaxCreditBondDocument { form: blank(), id: 0 }),
                ));
            }
        }),
    ];
    for (anchor, edit) in paths {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut request = open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        edit(documents, true);
        let before = session.draft.clone();
        let Reply::Response(P::Problem { message, field }) = send(&mut session, request.clone())
        else {
            panic!("{anchor}: an empty paper was saved without a question")
        };
        assert_eq!(message, nothing, "{anchor}");
        assert_eq!(field.as_deref(), Some(anchor));
        assert_eq!(session.draft, before);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        edit(documents, false);
        accepted(&send(&mut session, request));
    }
}
