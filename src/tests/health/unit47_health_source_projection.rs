// Actual source edits versus retained off-request health links. These reuse
// the original account/source fixtures and sole native tax owners.
use super::unit47_retained_source_gaps::retained_case;
use super::*;
use crate::tests::health::account_documents::{item, save_items};
use crate::tests::support::builders::contribution_documents::{
    attempted_hsa_rollover, ordinary_hsa_deposit,
};
use crate::tests::support::client::estimate_documents_from_save as credit_estimate_documents_from_save;
use crate::tests::support::client::income_pages::open_documents;
use alloc::string::ToString;

fn stable_source(mut session: Interview, _topic: TaxSection) -> (Interview, P) {
    let page = open_documents(&mut session);
    accepted(&send(&mut session, page));
    let page = open_documents(&mut session);
    (session, page)
}
fn failed_deposit_source() -> (Interview, P) {
    let mut session = save_items(
        &complete(FederalStatus::Single),
        vec![item(
            Owner::Taxpayer,
            0,
            attempted_hsa_rollover(20260101, vec![ordinary_hsa_deposit(20260303, 1000, 2026)]),
        )],
    );
    let source = session.draft.income.last().unwrap().id;
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value: input, .. } = &mut page else { unreachable!() };
    let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0);
    let mut correction = hsa_correction(2026, 20260401, 40_000, 0);
    correction.contributed_on = 20260303;
    correction.failed_deposit = Some(source);
    account.corrections = vec![correction.page()];
    input.accounts = vec![account];
    accepted(&send(&mut session, page));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "hsa_taxable_distributions"), 1000);
    assert_eq!(value(&result, "adjustments"), 600);
    assert_eq!(value(&result, "agi"), 60_400);
    stable_source(session, TaxSection::OtherIncome)
}
#[test]
fn unit47_health_source_failed_deposit_edits_have_actual_source_targets() {
    let (mut session, _) = failed_deposit_source();
    let good = open_hsa_page(&mut session);
    let mut bad = good.clone();
    let P::SaveHsa { followups, .. } = &mut bad else { unreachable!() };
    let index = followups
        .iter()
        .position(|row| matches!(row.answer, crate::HsaDocumentAnswer::MedicalPayment { .. }))
        .unwrap();
    let crate::HsaDocumentAnswer::MedicalPayment {
        value: Some(MedicalSaPayment::Own { rollover: Some(rollover), .. }),
    } = &mut followups[index].answer
    else {
        unreachable!()
    };
    rollover.receipts[0].received = 20260102;
    let path = libertas::SchemaPath::root()
        .field("followups")
        .key(followups[index].key.as_str())
        .field("answer")
        .field("value")
        .field("rollover")
        .field("receipts")
        .index(0)
        .field("received")
        .to_string();
    retained_case(
        session,
        bad,
        good,
        TaxSection::Hsa,
        "health-source-failed-date",
        "HEALTH_FAILED_DEPOSIT_UNAVAILABLE",
        &path,
    );

    let (mut session, source_good) = failed_deposit_source();
    let baseline = session.draft.clone();
    let mut result = supported(&baseline);
    let mut correction = source_good.clone();
    let P::SaveDocuments { documents, .. } = &mut correction else { unreachable!() };
    let crate::OtherIncomeDocument::Sa1099(data) =
        &mut documents.other.other_income.documents[0].document
    else {
        unreachable!()
    };
    data.gross_distribution = 39_900;
    accepted(&send(&mut session, correction));
    assert!(session.draft.valid_stored());
    session = restart(session.draft);
    assert!(!estimate(&session.draft).financial_source_scope_complete);
    let FederalIncome::MedicalDistribution { data } = &session.draft.income.last().unwrap().income
    else {
        unreachable!()
    };
    let FederalIncome::MedicalDistribution { data: before } =
        &baseline.income.last().unwrap().income
    else {
        unreachable!()
    };
    assert_eq!(data.payment, before.payment);
    assert_eq!(data.record().answer_issue().unwrap().key, "SA1099_MEDICAL_ABOVE_GROSS");
    let mut topic = open_hsa_page(&mut session);
    let P::SaveHsa { followups, .. } = &mut topic else { unreachable!() };
    let index = followups
        .iter()
        .position(|row| matches!(row.answer, crate::HsaDocumentAnswer::MedicalPayment { .. }))
        .unwrap();
    let crate::HsaDocumentAnswer::MedicalPayment {
        value: Some(MedicalSaPayment::Own { rollover: Some(rollover), .. }),
    } = &mut followups[index].answer
    else {
        unreachable!()
    };
    rollover.receipts[0].deposits[0].amount = 39_900;
    let path = libertas::SchemaPath::root()
        .field("followups")
        .key(followups[index].key.as_str())
        .field("answer")
        .field("value")
        .field("rollover")
        .field("receipts")
        .index(0)
        .field("deposits")
        .index(0)
        .field("amount")
        .to_string();
    for request in [topic.clone(), credit_estimate_documents_from_save(&topic)] {
        super::unit47_retained_source_gaps::retained_refuse(
            &mut session,
            request,
            "health-source-failed-capacity",
            "HEALTH_FAILED_DEPOSIT_RETURN_ABOVE_DEPOSIT",
            &path,
        );
    }
    let source = baseline.income.last().unwrap().id;
    let original_gross = before.internal().gross;
    let mut restore = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut restore else { unreachable!() };
    let crate::OtherIncomeDocument::Sa1099(paper) = &mut documents
        .other
        .other_income
        .documents
        .iter_mut()
        .find(|row| row.id == source)
        .unwrap()
        .document
    else {
        unreachable!()
    };
    paper.gross_distribution = original_gross;
    accepted(&send(&mut session, restore));
    let mut expected = baseline;
    // Restoring the paper preserves its answers and figures, but does not save
    // the correction page that reopen_refused reopened after the source edit.
    for section in [TaxSection::Adjustments, TaxSection::Hsa] {
        assert!(expected.finished_sections.contains(&section));
        expected.finished_sections.retain(|finished| *finished != section);
    }
    expected.revision = session.draft.revision;
    result.revision = session.draft.revision;
    assert_eq!(session.draft, expected);
    assert_eq!(supported(&session.draft), result);
    assert_eq!(restart(session.draft).draft, expected);
}

fn archer_source(earnings: i64) -> (Interview, P) {
    let mut draft = complete(FederalStatus::Joint);
    let people = draft.people.as_mut().unwrap();
    people.taxpayer.first_name = "Alex".into();
    people.spouse.as_mut().unwrap().first_name = "Jordan".into();
    name_accepted_income_documents(&mut draft);
    let printed = Some(accepted_document_names(&draft, Owner::Taxpayer));
    let mut session = save_items(
        &draft,
        vec![item(
            Owner::Taxpayer,
            0,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Sa1099(
                    crate::tests::support::client::income_pages::sa_document(crate::SaSource {
                        medicare_prior: if matches!(
                            &MedicalSaAccount::Archer,
                            MedicalSaAccount::MedicareAdvantage
                        ) {
                            Some(MedicareMsaPrior::None)
                        } else {
                            None
                        },
                        medicare_from_spouse: if matches!(
                            &MedicalSaAccount::Archer,
                            MedicalSaAccount::MedicareAdvantage
                        ) {
                            Some(false)
                        } else {
                            None
                        },
                        payment: None,
                        ..sa_source(Sa1099 {
                            trustee_payer: (printed)
                                .as_ref()
                                .and_then(|names| names.issuer.clone())
                                .unwrap_or_default(),
                            recipient: (printed)
                                .as_ref()
                                .map(|names| names.recipient.clone())
                                .unwrap_or_default(),
                            origin: HealthDocumentOrigin::Form1099Sa,
                            gross: 40_000 + earnings,
                            box2_earnings: (earnings > 0).then_some(earnings),
                            box3_code: Some(2),
                            box4_value: None,
                            account: MedicalSaAccount::Archer,
                        })
                    }),
                ),
            },
        )],
    );
    let id = session.draft.income.last().unwrap().id;
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value: input, .. } = &mut page else { unreachable!() };
    input.archer = ArcherContributionsPage::Records {
        owners: vec![ArcherContributorRow {
            corrections: vec![ArcherCorrectionPage {
                failed_deposit: None,
                document: Some(id),
                year: 2026,
                contributed_on: 20260601,
                principal: 40_000,
                earnings,
                returned: 20260930,
                deadline: 20270415,
                payer: ArcherCorrectionPayer::Personal,
            }],
            ..archer_contributor_row(
                Owner::Taxpayer,
                300_000,
                0,
                ArcherEligibility::FullYear { coverage: archer_self(4000) },
                vec![1],
            )
        }],
        sharing: ArcherSharing::Equal,
    };
    accepted(&send(&mut session, page));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "archer_msa_deduction"), 2600);
    assert_eq!(value(&result, "agi"), 97_400 + (earnings / 100).max(0));
    stable_source(session, TaxSection::OtherIncome)
}
#[test]
fn unit47_health_source_archer_owner_edits_have_actual_owner_targets() {
    for earnings in [2000, -2000] {
        let (session, good) = archer_source(earnings);
        let mut bad = good.clone();
        let P::SaveDocuments { documents, .. } = &mut bad else { unreachable!() };
        let target = &mut documents.other.other_income.documents[0];
        target.owner = crate::DocumentOwner::Spouses;
        let crate::OtherIncomeDocument::Sa1099(form) = &mut target.document else { unreachable!() };
        form.recipient = accepted_document_names(&session.draft, Owner::Spouse).recipient;
        retained_case(
            session,
            bad,
            good,
            TaxSection::Documents,
            if earnings > 0 {
                "health-source-archer-positive-owner"
            } else {
                "health-source-archer-negative-owner"
            },
            "HEALTH_CORRECTION_DOCUMENT_UNAVAILABLE",
            // The printed recipient that names the new owner is on the stored
            // paper, so the reassignment changes the submitted document itself.
            "[documents/other/other_income/documents/0/document]",
        );
    }
}
fn inherited_source() -> (Interview, P) {
    let mut draft = inherited_hsa_funding(1000, 0, 6000, 0, 4000);
    let inherited_id = draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::InheritedIra { .. }))
        .unwrap()
        .id;
    add(&mut draft, inherited_payment(Owner::Taxpayer, Some(inherited_id), 5000));
    let payment_id = draft.income.last().unwrap().id;
    let mut transfer = draft.adjustments.as_ref().unwrap().hsa[0].funding[0].row(Owner::Taxpayer);
    transfer.source = payment_id;
    draft.adjustments.as_mut().unwrap().hsa[0].funding.clear();
    let mut session = restart(draft);
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value: input, .. } = &mut page else { unreachable!() };
    input.funding = vec![transfer];
    accepted(&send(&mut session, page));
    assert_eq!(return_line(&supported(&session.draft), "4b"), 200_000);
    stable_source(session, TaxSection::Retirement)
}
#[test]
fn unit47_health_source_funding_payment_kind_edit_has_actual_document_target() {
    let (session, good) = inherited_source();
    let mut bad = good.clone();
    let P::SaveDocuments { documents, .. } = &mut bad else { unreachable!() };
    let payment_id = session.draft.adjustments.as_ref().unwrap().hsa[0].funding[0].income;
    let index =
        documents.common.retirement.documents.iter().position(|row| row.id == payment_id).unwrap();
    documents.common.retirement.documents[index].document =
        crate::RetirementDocument::R1099(retirement_document(r1099(
            500_000,
            DistributionCode::Normal { what_happened: Disposition::Kept },
            false,
        )));
    retained_case(
        session,
        bad,
        good,
        TaxSection::Documents,
        "health-source-funding-kind",
        "HSA_FUNDING_SOURCE_NOT_OFFERED",
        &alloc::format!("[documents/common/retirement/documents/{index}/document]"),
    );
}
#[test]
fn unit47_health_source_roth_distribution_edit_has_actual_document_target() {
    let (draft, funding) = crate::tests::unit46_hsa_funding_sources::fixture("roth");
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value: input, .. } = &mut page else { unreachable!() };
    input.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0)];
    input.family_split = FamilySplit::NotApplicable;
    input.funding = vec![funding.clone()];
    accepted(&send(&mut session, page));
    let result = supported(&session.draft);
    assert_eq!(return_line(&result, "4b"), 0);
    assert_eq!(value(&result, "agi"), 60_000);
    let (mut session, _) = stable_source(session, TaxSection::Retirement);
    let request = crate::tests::support::client::income_pages::open_topic(
        &mut session,
        TaxSection::Retirement,
    );
    accepted(&send(&mut session, request));
    let good = crate::tests::support::client::income_pages::open_topic(
        &mut session,
        TaxSection::Retirement,
    );
    let mut bad = good.clone();
    let P::SaveIncomes { followups, .. } = &mut bad else { unreachable!() };
    let key = alloc::format!("income/{}/treatment", funding.source);
    let index = followups.iter().position(|row| row.key == key).unwrap();
    let crate::DocumentAnswer::RetirementTreatment {
        code:
            Some(crate::RetirementTreatmentView::RothIra {
                roth: crate::RothIraCodeView::Early { distributed, .. },
                ..
            }),
    } = &mut followups[index].answer
    else {
        unreachable!()
    };
    *distributed = 20260703;
    let path = libertas::SchemaPath::root()
        .field("followups")
        .key(followups[index].key.as_str())
        .field("answer")
        .field("code")
        .field("roth")
        .field("distributed")
        .to_string();
    retained_case(
        session,
        bad,
        good,
        TaxSection::Retirement,
        "health-source-roth-distribution",
        "HSA_FUNDING_DISTRIBUTION_AFTER_RECEIPT",
        &path,
    );
}
fn off_request_case(mut session: Interview, bad: P, good: P, topic: TaxSection, key: &str) {
    let before = session.draft.clone();
    let dirty = session.dirty;
    let result = supported(&before);
    for request in [bad.clone(), credit_estimate_documents_from_save(&bad)] {
        let request = P::from_avro(&request.to_avro()).unwrap();
        let Reply::Response(P::Problem { message, field }) = send(&mut session, request) else {
            panic!("expected retained-source refusal")
        };
        assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
        assert_eq!(field, None);
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
    }
    accepted(&send(&mut session, good));
    assert_eq!(session.draft, before);
    let mut restored = restart(before.clone());
    let page = if topic == TaxSection::Documents {
        open_documents(&mut restored)
    } else {
        open_topic(&mut restored, topic)
    };
    accepted(&send(&mut restored, page));
    assert_eq!(restored.draft, before);
    assert_eq!(supported(&restored.draft), result);
}
#[test]
fn unit47_health_source_deleted_failed_deposit_keeps_off_request_null() {
    let (session, good) = failed_deposit_source();
    let mut bad = good.clone();
    let P::SaveDocuments { documents, .. } = &mut bad else { unreachable!() };
    documents.other.other_income.documents.clear();
    off_request_case(
        session,
        bad,
        good,
        TaxSection::Documents,
        "HEALTH_FAILED_DEPOSIT_UNAVAILABLE",
    );
}
#[test]
fn unit47_health_source_deleted_inherited_group_does_not_borrow_unchanged_payment() {
    let (mut session, _) = inherited_source();
    let inherited_id = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::InheritedIra { .. }))
        .unwrap()
        .id;
    let payment_id = session.draft.adjustments.as_ref().unwrap().hsa[0].funding[0].income;
    let good = open_topic(&mut session, TaxSection::Retirement);
    accepted(&send(&mut session, good));
    let good = open_topic(&mut session, TaxSection::Retirement);
    let mut bad = good.clone();
    let P::SaveIncomes { documents, .. } = &mut bad else { unreachable!() };
    documents.documents.retain(|row| row.id != inherited_id);
    let P::SaveDocuments { documents, .. } = open_documents(&mut session) else { unreachable!() };
    assert!(documents.common.retirement.documents.iter().any(|row| row.id == payment_id));
    off_request_case(session, bad, good, TaxSection::Retirement, "HSA_FUNDING_SOURCE_NOT_OFFERED");
}
