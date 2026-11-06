// form_1099_b items of src/tests/support/client/income_pages.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(in crate::tests) fn save_espp_lot(session: &mut Interview, lot: &EsppSale) -> Reply {
    let request = save_documents(
        session,
        vec![crate::tests::cross::interview::document(
            0,
            IncomeSource::InvestmentSale {
                value: InvestmentSaleDocument::EsppShareSale(espp_source_paper(lot)),
            },
        )],
    );
    let mut reply = send(session, request);
    if matches!(reply, Reply::Response(P::Problem { .. })) {
        return reply;
    }
    let id = session
        .draft
        .income
        .iter()
        .rev()
        .find(|entry| matches!(entry.income, FederalIncome::EsppSaleSource { .. }))
        .unwrap()
        .id;
    let answers = espp_source_answers(lot, id);
    // Acquisition records can make the wage-inclusion question newly applicable.
    for _ in 0..3 {
        let mut request = open_topic(session, TaxSection::InvestmentSales);
        let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
        for row in followups {
            if row.key == alloc::format!("income/{id}/espp_acquisition") {
                row.answer = DocumentAnswer::EsppAcquisition {
                    value: answers.acquisition.clone().map(Into::into),
                };
            } else if row.key == alloc::format!("income/{id}/espp_fixed_grant_price") {
                row.answer = DocumentAnswer::EsppFixedGrantPrice {
                    value: lot.grant_price.map_or(SourceCents::Unavailable, |amount| {
                        SourceCents::Reported { amount }
                    }),
                };
            } else if row.key == alloc::format!("income/{id}/espp_adjustments") {
                row.answer = DocumentAnswer::EsppAdjustments { value: answers.more.clone() };
            } else if row.key == alloc::format!("income/{id}/espp_wage_inclusion") {
                row.answer = DocumentAnswer::EsppWageInclusion { value: answers.wage_inclusion };
            }
        }
        reply = send(session, request);
        if matches!(reply, Reply::Response(P::Problem { .. })) {
            return reply;
        }
    }
    reply
}

// A printed ISO sale's Investment sales questions answered as a client: the
// fees and special treatment, then the W-2 answer once the sale has
// compensation. The last printed ISO sale on the return.
pub(in crate::tests) fn answer_iso_sale(
    session: &mut Interview,
    more: &crate::IsoShareSaleMore,
    in_wages: bool,
) -> Reply {
    let id = session
        .draft
        .income
        .iter()
        .rev()
        .find(|entry| matches!(entry.income, FederalIncome::IsoSaleSource { .. }))
        .unwrap()
        .id;
    answer_plan_sale(session, |key, answer| {
        if key == alloc::format!("income/{id}/iso_adjustments") {
            *answer = DocumentAnswer::IsoAdjustments { value: Some(more.clone()) };
        } else if key == alloc::format!("income/{id}/iso_wage_inclusion") {
            *answer = DocumentAnswer::IsoWageInclusion { value: Some(in_wages) };
        }
    })
}
// A printed NUA sale's Investment sales questions answered as a client. The
// last printed NUA sale on the return.
pub(in crate::tests) fn answer_nua_sale(
    session: &mut Interview,
    more: &crate::NuaShareSaleMore,
    ownership: &crate::NuaOwnership,
) -> Reply {
    let id = session
        .draft
        .income
        .iter()
        .rev()
        .find(|entry| matches!(entry.income, FederalIncome::NuaSaleSource { .. }))
        .unwrap()
        .id;
    answer_plan_sale(session, |key, answer| {
        if key == alloc::format!("income/{id}/nua_adjustments") {
            *answer = DocumentAnswer::NuaAdjustments { value: Some(more.clone()) };
        } else if key == alloc::format!("income/{id}/nua_ownership") {
            *answer = DocumentAnswer::NuaOwnership { value: Some(ownership.clone()) };
        }
    })
}
fn answer_plan_sale(session: &mut Interview, answer: impl Fn(&str, &mut DocumentAnswer)) -> Reply {
    // The second question appears once the first is answered.
    let mut reply = None;
    for _ in 0..2 {
        let mut request = open_topic(session, TaxSection::InvestmentSales);
        let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
        for row in followups.iter_mut() {
            answer(&row.key, &mut row.answer);
            // Any other plan sale on the page keeps the fixture's ordinary answers.
            match &mut row.answer {
                DocumentAnswer::IsoAdjustments { value: value @ None } => {
                    *value = Some(crate::IsoShareSaleMore {
                        selling_fees: 0,
                        acquisition_costs: 0,
                        option_cost: 0,
                        special: false,
                    })
                }
                DocumentAnswer::IsoWageInclusion { value: value @ None } => *value = Some(true),
                DocumentAnswer::NuaAdjustments { value: value @ None } => {
                    *value = Some(crate::NuaShareSaleMore { wash_sale: 0, special: false })
                }
                DocumentAnswer::NuaOwnership { value: value @ None } => {
                    *value = Some(crate::NuaOwnership::Own)
                }
                _ => {}
            }
        }
        let sent = send(session, request);
        if matches!(sent, Reply::Response(P::Problem { .. })) {
            return sent;
        }
        reply = Some(sent);
    }
    reply.unwrap()
}
