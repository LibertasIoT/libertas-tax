use super::income_pages::open_topic;
use crate::tests::*;

pub(in crate::tests) fn select_acquisition(
    session: &mut Interview,
    sale_id: i64,
    acquisition_id: i64,
) {
    select_acquisitions(session, &[(sale_id, acquisition_id)]);
}

pub(in crate::tests) fn select_acquisitions(session: &mut Interview, links: &[(i64, i64)]) {
    let mut request = open_topic(session, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    for (sale_id, acquisition_id) in links {
        let row = followups
            .iter_mut()
            .find(|row| row.key == alloc::format!("income/{sale_id}/espp_acquisition"))
            .unwrap();
        row.answer = DocumentAnswer::EsppAcquisition {
            value: Some(EsppAcquisitionChoice::Printed { document_id: *acquisition_id }.into()),
        };
    }
    accepted(&send(session, request));
}

pub(in crate::tests) fn answer_wage_inclusion(session: &mut Interview, sale_id: i64) -> usize {
    answer_wage_inclusions(session, &[sale_id])
}

pub(in crate::tests) fn answer_wage_inclusions(session: &mut Interview, sale_ids: &[i64]) -> usize {
    let mut request = open_topic(session, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    let price_questions = followups
        .iter()
        .filter(|row| matches!(row.answer, DocumentAnswer::EsppFixedGrantPrice { .. }))
        .count();
    for sale_id in sale_ids {
        let row = followups
            .iter_mut()
            .find(|row| row.key == alloc::format!("income/{sale_id}/espp_wage_inclusion"))
            .unwrap();
        row.answer = DocumentAnswer::EsppWageInclusion { value: Some(true) };
    }
    accepted(&send(session, request));
    price_questions
}

pub(in crate::tests) fn answer_fixed_grant_price(session: &mut Interview, amount: i64) {
    let mut request = open_topic(session, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    let mut rows = followups
        .iter_mut()
        .filter(|row| matches!(row.answer, DocumentAnswer::EsppFixedGrantPrice { .. }));
    rows.next().unwrap().answer =
        DocumentAnswer::EsppFixedGrantPrice { value: SourceCents::Reported { amount } };
    assert!(rows.next().is_none());
    accepted(&send(session, request));
}
