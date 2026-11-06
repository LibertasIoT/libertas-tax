// Printed statements and independently supplied QSBS facts exercise both editors.
use crate::tests::*;

pub(in crate::tests) fn remember_qsbs_answers(source: &QsbIssuerSale, paper: QsbsSale) -> QsbsSale {
    let mut answers = source.clone();
    // Legacy fixtures' explicit missing records mean the established conservative
    // answer, rather than an unfinished person-facing control.
    if answers.zone_60_exclusion.is_none() {
        answers.zone_60_exclusion = Some(crate::QsbsZoneCertification::NoRecords);
    }
    for lot in &mut answers.lots {
        if lot.zone_gain_through_2018.is_none() {
            lot.zone_gain_through_2018 = Some(crate::QsbsZoneValuation::NoRecords);
        }
    }
    super::paper_documents::remember(
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::QsbsSale(paper.clone()),
        },
        crate::qsbs_followups::questions(&answers)
            .into_iter()
            .map(|(_, _, answer)| answer)
            .collect(),
    );
    paper
}

pub(in crate::tests) fn qsbs_document(source: &QsbIssuerSale) -> QsbsSale {
    remember_qsbs_answers(source, QsbsSale::from_record(source))
}

// Build the actual offered topic input, or a well-formed unoffered row for
// reachability regressions, using the real accepted document and lot identity.
pub(in crate::tests) fn qsbs_zone_topic_request(
    session: &mut Interview,
    source: &QsbIssuerSale,
    lot: usize,
) -> P {
    let desired = source.lots[lot].zone_gain_through_2018.expect("fixture supplies a zone answer");
    let mut baseline = source.clone();
    baseline.zone_60_exclusion = Some(crate::QsbsZoneCertification::Yes);
    for lot in &mut baseline.lots {
        lot.zone_gain_through_2018 = Some(crate::QsbsZoneValuation::NoRecords);
    }
    let paper = qsbs_document(&baseline);
    let row = crate::tests::cross::interview::document(
        0,
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::QsbsSale(paper) },
    );
    let reply =
        crate::tests::support::client::income_pages::save_source_fixtures(session, vec![row]);
    crate::tests::cross::interview::accepted(&reply);
    let id = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::QualifiedSmallBusinessSale { .. }))
        .unwrap()
        .id;
    let mut request = crate::tests::support::client::income_pages::open_topic(
        session,
        TaxSection::InvestmentSales,
    );
    let P::SaveIncomes { followups, .. } = &mut request else { panic!("QSBS topic") };
    let key = alloc::format!("income/{id}/qsbs_zone_gain/{lot}");
    let answer = crate::DocumentAnswer::QsbsZoneGain {
        value: crate::QsbsValuationAnswer {
            lot: crate::QsbsPaperLot::from(&source.lots[lot]),
            value: Some(desired),
            known: vec![desired],
        },
    };
    if let Some(row) = followups.iter_mut().find(|row| row.key == key) {
        row.answer = answer;
    } else {
        let mut row = followups[0].clone();
        row.key = key;
        row.answer = answer;
        row.allowed_answers =
            vec![libertas_macros::variant_index!(crate::DocumentAnswer::QsbsZoneGain) as i32];
        followups.push(row);
    }
    request
}
