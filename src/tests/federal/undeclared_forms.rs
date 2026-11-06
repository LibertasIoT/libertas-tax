use super::*;
use crate::tests::cross::interview::document;
use crate::tests::support::client::income_pages::{
    open_document_center, open_topic, save_documents,
};
use crate::tests::support::client::undeclared_forms::{
    answer_fixed_grant_price, answer_wage_inclusion, answer_wage_inclusions, select_acquisition,
    select_acquisitions,
};

#[test]
fn historical_3922_remains_an_acquisition_through_correction_and_restart() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let income_count = session.draft.income.len();
    let form = espp_acquisition_paper(&ordinary_espp_lot(), 100.0, 20250801, Some(2000));
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::InvestmentSale {
                value: InvestmentSaleDocument::Form3922(espp_acquisition_document(form)),
            },
        )],
    );
    accepted(&send(&mut session, request));
    let source = &session.draft.screening.as_ref().unwrap().espp_acquisitions[0];
    assert_eq!(source.owner, Owner::Taxpayer);
    let source_id = source.id;
    let expected = source.form.clone();
    assert_eq!(session.draft.income.len(), income_count);
    assert_eq!(value(&supported(&session.draft), "agi"), 60_000);
    let mut session = restart(session.draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let form = documents
        .common
        .investment_sales
        .documents
        .iter_mut()
        .find_map(|row| match &mut row.document {
            InvestmentSaleDocument::Form3922(form) => Some(form),
            _ => None,
        })
        .unwrap();
    assert_eq!(*form, expected);
    form.grant_price = None;
    form.corrected = true;
    form.transferred = 20250901;
    accepted(&send(&mut session, request));
    let session = restart(session.draft);
    let source = &session.draft.screening.as_ref().unwrap().espp_acquisitions[0];
    assert_eq!(source.id, source_id);
    assert_eq!(source.form.grant_price, None);
    assert_eq!(source.form.transferred, 20250901);
    assert!(source.form.corrected);
    assert_eq!(session.draft.income.len(), income_count);
    assert_eq!(value(&supported(&session.draft), "agi"), 60_000);
}

#[test]
fn linked_3922_uses_sold_shares_and_reprojects_corrections_without_losing_wage_answer() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut lot = ordinary_espp_lot();
    lot.shares = 2.0;
    lot.proceeds = 6000;
    let form = espp_acquisition_paper(&lot, 100.0, 20250801, Some(2000));
    let request = save_documents(
        &mut session,
        vec![
            document(
                0,
                IncomeSource::InvestmentSale {
                    value: InvestmentSaleDocument::Form3922(espp_acquisition_document(form)),
                },
            ),
            document(
                0,
                IncomeSource::InvestmentSale {
                    value: InvestmentSaleDocument::EsppShareSale(espp_source_paper(&lot)),
                },
            ),
        ],
    );
    accepted(&send(&mut session, request));
    let acquisition_id = session.draft.screening.as_ref().unwrap().espp_acquisitions[0].id;
    let sale_id = session
        .draft
        .income
        .iter()
        .find(|row| matches!(row.income, FederalIncome::EsppSaleSource { .. }))
        .unwrap()
        .id;
    select_acquisition(&mut session, sale_id, acquisition_id);
    assert_eq!(answer_wage_inclusion(&mut session, sale_id), 0);
    let result = supported(&session.draft);
    assert_eq!(value(&result, &alloc::format!("espp_sale_{sale_id}_compensation")), 4);
    assert_eq!(value(&result, &alloc::format!("espp_sale_{sale_id}_basis")), 44);
    assert_eq!(value(&result, &alloc::format!("espp_sale_{sale_id}_capital_gain")), 16);
    assert_eq!(value(&result, "agi"), 60_016);
    let mut session = restart(session.draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let form = documents
        .common
        .investment_sales
        .documents
        .iter_mut()
        .find_map(|row| match &mut row.document {
            InvestmentSaleDocument::Form3922(form) => Some(form),
            _ => None,
        })
        .unwrap();
    form.grant_market = 2500;
    form.corrected = true;
    accepted(&send(&mut session, request));
    let mut session = restart(session.draft);
    let result = supported(&session.draft);
    assert_eq!(value(&result, &alloc::format!("espp_sale_{sale_id}_compensation")), 10);
    assert_eq!(value(&result, &alloc::format!("espp_sale_{sale_id}_basis")), 50);
    assert_eq!(value(&result, "agi"), 60_010);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents
        .common
        .investment_sales
        .documents
        .retain(|row| !matches!(row.document, InvestmentSaleDocument::Form3922(_)));
    accepted(&send(&mut session, request));
    let mut session = restart(session.draft);
    assert_eq!(
        session
            .draft
            .screening
            .as_ref()
            .unwrap()
            .espp_sale_answers
            .iter()
            .find(|row| row.document_id == sale_id)
            .unwrap()
            .wage_inclusion,
        Some(true)
    );
    let P::SaveIncomes { followups, .. } = open_topic(&mut session, TaxSection::InvestmentSales)
    else {
        unreachable!()
    };
    assert!(
        followups.iter().any(|row| row.key == alloc::format!("income/{sale_id}/espp_acquisition"))
    );
}

#[test]
fn disqualifying_3922_sale_does_not_require_an_unused_blank_box8_price() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut lot = ordinary_espp_lot();
    lot.granted = 20240601;
    lot.exercised = 20251201;
    lot.shares = 2.0;
    lot.proceeds = 6000;
    let form = espp_acquisition_paper(&lot, 100.0, 20251215, None);
    let request = save_documents(
        &mut session,
        vec![
            document(
                0,
                IncomeSource::InvestmentSale {
                    value: InvestmentSaleDocument::Form3922(espp_acquisition_document(form)),
                },
            ),
            document(
                0,
                IncomeSource::InvestmentSale {
                    value: InvestmentSaleDocument::EsppShareSale(espp_source_paper(&lot)),
                },
            ),
        ],
    );
    accepted(&send(&mut session, request));
    let acquisition_id = session.draft.screening.as_ref().unwrap().espp_acquisitions[0].id;
    let sale_id = session
        .draft
        .income
        .iter()
        .find(|row| matches!(row.income, FederalIncome::EsppSaleSource { .. }))
        .unwrap()
        .id;
    select_acquisition(&mut session, sale_id, acquisition_id);
    assert_eq!(answer_wage_inclusion(&mut session, sale_id), 0);
    let result = supported(&session.draft);
    assert_eq!(value(&result, &alloc::format!("espp_sale_{sale_id}_compensation")), 6);
    assert_eq!(value(&result, &alloc::format!("espp_sale_{sale_id}_basis")), 46);
    assert_eq!(value(&result, "agi"), 60_014);
}

#[test]
fn qualifying_partial_sales_share_one_acquisition_price_answer() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut first = ordinary_espp_lot();
    first.shares = 2.0;
    first.proceeds = 6000;
    let mut second = first.clone();
    second.shares = 3.0;
    second.proceeds = 9000;
    let form = espp_acquisition_paper(&first, 100.0, 20250801, None);
    let request = save_documents(
        &mut session,
        vec![
            document(
                0,
                IncomeSource::InvestmentSale {
                    value: InvestmentSaleDocument::Form3922(espp_acquisition_document(form)),
                },
            ),
            document(
                0,
                IncomeSource::InvestmentSale {
                    value: InvestmentSaleDocument::EsppShareSale(espp_source_paper(&first)),
                },
            ),
            document(
                0,
                IncomeSource::InvestmentSale {
                    value: InvestmentSaleDocument::EsppShareSale(espp_source_paper(&second)),
                },
            ),
        ],
    );
    accepted(&send(&mut session, request));
    let acquisition_id = session.draft.screening.as_ref().unwrap().espp_acquisitions[0].id;
    let sale_ids: Vec<_> = session
        .draft
        .income
        .iter()
        .filter(|row| matches!(row.income, FederalIncome::EsppSaleSource { .. }))
        .map(|row| row.id)
        .collect();
    assert_eq!(sale_ids.len(), 2);
    let links: Vec<_> = sale_ids.iter().map(|id| (*id, acquisition_id)).collect();
    select_acquisitions(&mut session, &links);
    answer_fixed_grant_price(&mut session, 2000);
    assert_eq!(answer_wage_inclusions(&mut session, &sale_ids), 1);
    let session = restart(session.draft);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 60_040);
    assert_eq!(
        session.draft.screening.as_ref().unwrap().espp_acquisitions[0].fixed_grant_price,
        SourceCents::Reported { amount: 2000 }
    );
    assert_eq!(
        session.draft.screening.as_ref().unwrap().espp_acquisitions[0].form.grant_price,
        None
    );
}

#[test]
fn rrb1099_printed_net_withholding_and_medicare_follow_source_corrections() {
    let mut draft = complete(FederalStatus::Single);
    elect_itemized(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let issuer_address = "10 Example Plaza\nSuite 200\nExample City, NY 10001";
    let recipient_address = "20 Sample Street\nExample Town, NY 10002";
    let mut form = rrb_benefit_paper();
    form.issuer.address = issuer_address.into();
    form.recipient.address = recipient_address.into();
    let request = save_documents(
        &mut session,
        vec![document(0, IncomeSource::Retirement { value: RetirementDocument::Rrb1099(form) })],
    );
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(return_line(&result, "6a"), 1_800_000);
    assert_eq!(return_line(&result, "25b"), 10_000);
    assert_eq!(value(&result, "medical_expenses"), 2400);
    let expected = session.draft.income.clone();
    let mut session = restart(session.draft);
    assert_eq!(session.draft.income, expected);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let form = documents
        .common
        .retirement
        .documents
        .iter_mut()
        .find_map(|row| match &mut row.document {
            RetirementDocument::Rrb1099(form) => Some(form),
            _ => None,
        })
        .unwrap();
    assert_eq!(form.claim_number, "A000123456-01");
    assert_eq!(form.recipient_id, "987-65-4320");
    assert_eq!(form.issuer.address, issuer_address);
    assert_eq!(form.recipient.address, recipient_address);
    assert_eq!(form.benefits, 1_800_000);
    form.benefits_paid = 100_000;
    form.benefits_repaid = 150_000;
    form.benefits = -50_000;
    form.withholding = 2500;
    form.medicare_premiums = 0;
    form.corrected = true;
    accepted(&send(&mut session, request));
    let session = restart(session.draft);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "social_security_net_repayments"), 500);
    assert_eq!(value(&result, "medical_expenses"), 0);
    assert_eq!(return_line(&result, "25b"), 2500);
    assert_eq!(value(&result, "agi"), 60_000);
}

#[test]
fn replacing_a_printed_acquisition_does_not_move_its_old_price_answer() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut lot = ordinary_espp_lot();
    lot.shares = 2.0;
    lot.proceeds = 6000;
    let first = espp_acquisition_paper(&lot, 100.0, 20250801, None);
    let mut second = first.clone();
    second.account_number = "ESPP-02".into();
    second.corporation.name = "Second Stock Plan Corporation".into();
    let request = save_documents(
        &mut session,
        vec![
            document(
                0,
                IncomeSource::InvestmentSale {
                    value: InvestmentSaleDocument::Form3922(espp_acquisition_document(first)),
                },
            ),
            document(
                0,
                IncomeSource::InvestmentSale {
                    value: InvestmentSaleDocument::Form3922(espp_acquisition_document(second)),
                },
            ),
            document(
                0,
                IncomeSource::InvestmentSale {
                    value: InvestmentSaleDocument::EsppShareSale(espp_source_paper(&lot)),
                },
            ),
        ],
    );
    accepted(&send(&mut session, request));
    let acquisitions = &session.draft.screening.as_ref().unwrap().espp_acquisitions;
    let first_id = acquisitions.iter().find(|row| row.form.account_number == "ESPP-01").unwrap().id;
    let second_id =
        acquisitions.iter().find(|row| row.form.account_number == "ESPP-02").unwrap().id;
    let sale_id = session
        .draft
        .income
        .iter()
        .find(|row| matches!(row.income, FederalIncome::EsppSaleSource { .. }))
        .unwrap()
        .id;
    select_acquisition(&mut session, sale_id, first_id);
    answer_fixed_grant_price(&mut session, 2000);
    assert_eq!(answer_wage_inclusion(&mut session, sale_id), 1);
    assert_eq!(value(&supported(&session.draft), "agi"), 60_016);
    let mut request = open_topic(&mut session, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    for row in followups.iter_mut() {
        if row.key == alloc::format!("income/{sale_id}/espp_acquisition") {
            row.answer = DocumentAnswer::EsppAcquisition {
                value: Some(EsppAcquisitionChoice::Printed { document_id: second_id }.into()),
            };
        } else if matches!(row.answer, DocumentAnswer::EsppFixedGrantPrice { .. }) {
            row.answer = DocumentAnswer::EsppFixedGrantPrice {
                value: SourceCents::Reported { amount: 1800 },
            };
        }
    }
    followups.reverse();
    accepted(&send(&mut session, request));
    let mut session = restart(session.draft);
    let acquisitions = &session.draft.screening.as_ref().unwrap().espp_acquisitions;
    assert_eq!(
        acquisitions.iter().find(|row| row.id == first_id).unwrap().fixed_grant_price,
        SourceCents::Reported { amount: 2000 }
    );
    assert_eq!(
        acquisitions.iter().find(|row| row.id == second_id).unwrap().fixed_grant_price,
        SourceCents::Unavailable
    );
    let P::SaveIncomes { followups, .. } = open_topic(&mut session, TaxSection::InvestmentSales)
    else {
        unreachable!()
    };
    assert!(followups.iter().any(|row| matches!(
        row.answer,
        DocumentAnswer::EsppFixedGrantPrice { value: SourceCents::Unavailable }
    )));
    answer_fixed_grant_price(&mut session, 2100);
    let session = restart(session.draft);
    assert_eq!(value(&supported(&session.draft), "agi"), 60_018);
    assert_eq!(
        session
            .draft
            .screening
            .as_ref()
            .unwrap()
            .espp_sale_answers
            .iter()
            .find(|row| row.document_id == sale_id)
            .unwrap()
            .wage_inclusion,
        Some(true)
    );
}

#[test]
fn rrb_prior_year_boxes_preserve_an_unsplit_aggregate_until_actual_year_detail_arrives() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut form = rrb_benefit_paper();
    form.workers_compensation = 40_000;
    form.prior_year = 50_000;
    form.two_years_prior = 30_000;
    form.earlier_years = 20_000;
    let request = save_documents(
        &mut session,
        vec![document(0, IncomeSource::Retirement { value: RetirementDocument::Rrb1099(form) })],
    );
    accepted(&send(&mut session, request));
    assert_eq!(return_line(&supported(&session.draft), "6a"), 1_800_000);
    let source = session
        .draft
        .income
        .iter()
        .find_map(|row| match &row.income {
            FederalIncome::SocialSecurity { data } => Some(FederalSocial::from(data)),
            _ => None,
        })
        .unwrap();
    let years: Vec<_> = source
        .form_boxes
        .paid_description
        .iter()
        .filter_map(|row| match &row.description {
            SocialBenefitDescriptionKind::PriorYear { year } => Some((*year, row.amount)),
            _ => None,
        })
        .collect();
    assert_eq!(years, vec![(2025, 50_000), (2024, 30_000)]);
    let mut session = restart(session.draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let form = documents
        .common
        .retirement
        .documents
        .iter_mut()
        .find_map(|row| match &mut row.document {
            RetirementDocument::Rrb1099(form) => Some(form),
            _ => None,
        })
        .unwrap();
    assert_eq!(form.earlier_years, 20_000);
    assert!(form.earlier_year_details.is_empty());
    form.earlier_year_details.push(RrbBenefitYear { year: 2023, amount: 20_000 });
    form.corrected = true;
    accepted(&send(&mut session, request));
    let session = restart(session.draft);
    let result = supported(&session.draft);
    assert_eq!(return_line(&result, "6a"), 1_800_000);
    let source = session
        .draft
        .income
        .iter()
        .find_map(|row| match &row.income {
            FederalIncome::SocialSecurity { data } => Some(FederalSocial::from(data)),
            _ => None,
        })
        .unwrap();
    let years: Vec<_> = source
        .form_boxes
        .paid_description
        .iter()
        .filter_map(|row| match &row.description {
            SocialBenefitDescriptionKind::PriorYear { year } => Some((*year, row.amount)),
            _ => None,
        })
        .collect();
    assert_eq!(years, vec![(2025, 50_000), (2024, 30_000), (2023, 20_000)]);
}
