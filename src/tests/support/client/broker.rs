use crate::tests::cross::interview::{accepted, open_topic};
use crate::tests::*;

pub(in crate::tests) fn admit_broker_paper(session: &mut Interview, paper: BrokerPaper) -> i64 {
    let source = BrokerDocument::new(paper).income_source();
    let row = named_document(&session.draft, -1, 0, source);
    let request = super::income_pages::save_documents(session, vec![row]);
    accepted(&send(session, request));
    let entry = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Broker { .. }))
        .expect("the actual document was accepted");
    assert_eq!(broker_source(&session.draft, entry.id).assigned, DocumentRecipient::Taxpayer);
    entry.id
}

pub(in crate::tests) fn broker_source(draft: &FederalDraft, id: i64) -> &BrokerDocument {
    let entry = draft.income.iter().find(|entry| entry.id == id).expect("stable document");
    let FederalIncome::Broker { data } = &entry.income else { panic!("actual broker paper") };
    data
}

pub(in crate::tests) fn broker_answers_page(
    session: &mut Interview,
    id: i64,
    mut edit: impl FnMut(&mut DocumentAnswer),
) -> P {
    let mut request = open_topic(session, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut request else { panic!("normal topic editor") };
    let prefix = alloc::format!("income/{id}/");
    for row in followups.iter_mut().filter(|row| row.key.starts_with(&prefix)) {
        edit(&mut row.answer);
    }
    request
}

// Explicit known fixture answers use the ordinary topic editor after the
// printed source and its matcher-owned association have been accepted.
pub(in crate::tests) fn ordinary_broker_answers(
    session: &mut Interview,
    id: i64,
    basis: Option<Option<i64>>,
) {
    let request = broker_answers_page(session, id, |answer| match answer {
        DocumentAnswer::BrokerSaleRecords { value } => {
            value.use_kind = Some(SaleUse::Investment { wash_sale: 0 });
            if let Some(amount) = basis {
                value.basis = Some(BrokerBasisAnswer { amount });
            }
        }
        DocumentAnswer::BrokerContractsTreatment { value } => *value = Some(false),
        _ => {}
    });
    accepted(&send(session, request));
}

pub(in crate::tests) fn broker_b_editor(page: &mut P, id: i64) -> &mut Form1099B {
    let P::SaveDocuments { documents, .. } = page else { panic!("normal Document center") };
    let row = documents
        .common
        .investment_sales
        .documents
        .iter_mut()
        .find(|row| row.id == id)
        .expect("stable source in its actual category");
    let InvestmentSaleDocument::Form1099B(source) = &mut row.document else { panic!("1099-B") };
    source
}

pub(in crate::tests) fn broker_da_editor(page: &mut P, id: i64) -> &mut Form1099DA {
    let P::SaveDocuments { documents, .. } = page else { panic!("normal Document center") };
    let row = documents
        .common
        .investment_sales
        .documents
        .iter_mut()
        .find(|row| row.id == id)
        .expect("stable source in its actual category");
    let InvestmentSaleDocument::Form1099DA(source) = &mut row.document else { panic!("1099-DA") };
    source
}
