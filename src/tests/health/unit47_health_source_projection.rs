// Actual source edits versus retained off-request health links. These reuse
// the original account/source fixtures and sole native tax owners.
use super::unit47_credit_source_validation::credit_estimate_documents_from_save;
use super::unit47_retained_source_gaps::retained_case;
use super::*;
use crate::tests::health::account_documents::{
    attempted_hsa_rollover, item, ordinary_hsa_deposit, save_items,
};

fn stable_source(mut session: Interview, topic: TaxSection) -> (Interview, P) {
    let page = open_topic(&mut session, topic);
    accepted(&send(&mut session, page));
    let page = open_topic(&mut session, topic);
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
    let P::SaveHsa { value: input, .. } = &mut page else {
        unreachable!()
    };
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
    for receipt_date in [true, false] {
        let (session, good) = failed_deposit_source();
        let mut bad = good.clone();
        let P::SaveIncomes { documents, .. } = &mut bad else {
            unreachable!()
        };
        let IncomeDocument::Sa1099(data) = &mut documents.documents[0].document else {
            unreachable!()
        };
        let MedicalSaPayment::Own {
            rollover: Some(rollover),
            ..
        } = &mut data.payment
        else {
            unreachable!()
        };
        let (case, key) = if receipt_date {
            rollover.receipts[0].received = 20260102;
            (
                "health-source-failed-date",
                "HEALTH_FAILED_DEPOSIT_UNAVAILABLE",
            )
        } else {
            data.gross = 39_900;
            rollover.receipts[0].deposits[0].amount = 39_900;
            (
                "health-source-failed-capacity",
                "HEALTH_FAILED_DEPOSIT_RETURN_ABOVE_DEPOSIT",
            )
        };
        retained_case(
            session,
            bad,
            good,
            TaxSection::OtherIncome,
            case,
            key,
            "[documents/documents/0/document]",
        );
    }
}
fn archer_source(earnings: i64) -> (Interview, P) {
    let mut session = save_items(
        &complete(FederalStatus::Joint),
        vec![item(
            Owner::Taxpayer,
            0,
            IncomeDocument::Sa1099(Sa1099 {
                origin: HealthDocumentOrigin::Form1099Sa,
                gross: 40_000 + earnings,
                account: MedicalSaAccount::Archer,
                payment: MedicalSaPayment::ExcessReturned {
                    earnings: (earnings > 0).then_some(earnings),
                },
            }),
        )],
    );
    let id = session.draft.income.last().unwrap().id;
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value: input, .. } = &mut page else {
        unreachable!()
    };
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
        let P::SaveIncomes { documents, .. } = &mut bad else {
            unreachable!()
        };
        documents.documents[0].owner = Owner::Spouse;
        retained_case(
            session,
            bad,
            good,
            TaxSection::OtherIncome,
            if earnings > 0 {
                "health-source-archer-positive-owner"
            } else {
                "health-source-archer-negative-owner"
            },
            "HEALTH_CORRECTION_DOCUMENT_UNAVAILABLE",
            "[documents/documents/0/owner]",
        );
    }
}
fn inherited_source() -> (Interview, P) {
    let mut draft = inherited_hsa_funding(1000, 0, 6000, 0, 4000);
    add(
        &mut draft,
        inherited_payment(Owner::Taxpayer, Some(2), 5000),
    );
    let mut transfer = draft.adjustments.as_ref().unwrap().hsa[0].funding[0].row(Owner::Taxpayer);
    transfer.source = 4;
    draft.adjustments.as_mut().unwrap().hsa[0].funding.clear();
    let mut session = restart(draft);
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value: input, .. } = &mut page else {
        unreachable!()
    };
    input.funding = vec![transfer];
    accepted(&send(&mut session, page));
    assert_eq!(return_line(&supported(&session.draft), "4b"), 200_000);
    stable_source(session, TaxSection::Retirement)
}
#[test]
fn unit47_health_source_funding_payment_kind_edit_has_actual_document_target() {
    let (session, good) = inherited_source();
    let mut bad = good.clone();
    let P::SaveIncomes { documents, .. } = &mut bad else {
        unreachable!()
    };
    let index = documents
        .documents
        .iter()
        .position(|row| row.id == 4)
        .unwrap();
    documents.documents[index].document = IncomeDocument::R1099(r1099(
        500_000,
        DistributionCode::Normal {
            what_happened: Disposition::Kept,
        },
        false,
    ));
    retained_case(
        session,
        bad,
        good,
        TaxSection::Retirement,
        "health-source-funding-kind",
        "HSA_FUNDING_SOURCE_NOT_OFFERED",
        &alloc::format!("[documents/documents/{index}/document]"),
    );
}
#[test]
fn unit47_health_source_roth_distribution_edit_has_actual_document_target() {
    let (draft, funding) = crate::tests::unit46_hsa_funding_sources::fixture("roth");
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value: input, .. } = &mut page else {
        unreachable!()
    };
    input.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0)];
    input.family_split = FamilySplit::NotApplicable;
    input.funding = vec![funding.clone()];
    accepted(&send(&mut session, page));
    let result = supported(&session.draft);
    assert_eq!(return_line(&result, "4b"), 0);
    assert_eq!(value(&result, "agi"), 60_000);
    let (session, good) = stable_source(session, TaxSection::Retirement);
    let mut bad = good.clone();
    let P::SaveIncomes { documents, .. } = &mut bad else {
        unreachable!()
    };
    let index = documents
        .documents
        .iter()
        .position(|row| row.id == funding.source)
        .unwrap();
    let IncomeDocument::R1099(data) = &mut documents.documents[index].document else {
        unreachable!()
    };
    let DistributionCode::RothIra {
        roth: RothIraCode::Early { distributed, .. },
        ..
    } = &mut data.code
    else {
        unreachable!()
    };
    *distributed = 20260703;
    retained_case(
        session,
        bad,
        good,
        TaxSection::Retirement,
        "health-source-roth-distribution",
        "HSA_FUNDING_DISTRIBUTION_AFTER_RECEIPT",
        &alloc::format!("[documents/documents/{index}/document]"),
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
    let page = open_topic(&mut restored, topic);
    accepted(&send(&mut restored, page));
    assert_eq!(restored.draft, before);
    assert_eq!(supported(&restored.draft), result);
}
#[test]
fn unit47_health_source_deleted_failed_deposit_keeps_off_request_null() {
    let (session, good) = failed_deposit_source();
    let mut bad = good.clone();
    let P::SaveIncomes { documents, .. } = &mut bad else {
        unreachable!()
    };
    documents.documents.clear();
    off_request_case(
        session,
        bad,
        good,
        TaxSection::OtherIncome,
        "HEALTH_FAILED_DEPOSIT_UNAVAILABLE",
    );
}
#[test]
fn unit47_health_source_deleted_inherited_group_does_not_borrow_unchanged_payment() {
    let (session, good) = inherited_source();
    let mut bad = good.clone();
    let P::SaveIncomes { documents, .. } = &mut bad else {
        unreachable!()
    };
    documents.documents.retain(|row| row.id != 2);
    assert!(documents.documents.iter().any(|row| row.id == 4));
    off_request_case(
        session,
        bad,
        good,
        TaxSection::Retirement,
        "HSA_FUNDING_SOURCE_NOT_OFFERED",
    );
}
