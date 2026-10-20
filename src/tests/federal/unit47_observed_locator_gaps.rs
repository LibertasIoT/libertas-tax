// Captured full-suite refusals exposed posted-row locator gaps. These controls
// preserve the actual sole allocator/source owners and accepted Draft lifecycle.
use super::unit47_credit_source_validation::{credit_estimate_documents_from_save, credit_refusal};
use super::*;

const REPEATED: &str = "ISSUE_INTERVIEW_A_RECORD_IDENTITY_IS_REPEATED_IN_EE6DC425";
const FOREIGN: &str = "ISSUE_INTERVIEW_A_RECORD_NO_LONGER_BELONGS_TO_51E52C2C";

fn request_rows(request: &mut P) -> &mut Vec<IncomeDocumentItem> {
    let P::SaveIncomes { documents, .. } = request else {
        unreachable!()
    };
    &mut documents.documents
}
fn reject_both(case: &str, session: &mut Interview, invalid: P, key: &str, path: &str) {
    let invalid = P::from_avro(&invalid.to_avro()).unwrap();
    let running = credit_estimate_documents_from_save(&invalid);
    credit_refusal(
        &alloc::format!("observed-gap-{case}-save"),
        session,
        invalid,
        key,
        path,
    );
    credit_refusal(
        &alloc::format!("observed-gap-{case}-estimate"),
        session,
        running,
        key,
        path,
    );
}
fn unchanged_correction(session: &mut Interview, corrected: P, topic: TaxSection) {
    let saved = session.draft.clone();
    let dirty = session.dirty;
    let financial = estimate(&saved);
    assert!(matches!(
        send(session, credit_estimate_documents_from_save(&corrected)),
        Reply::Response(P::IncomeRunningEstimate { .. })
    ));
    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, corrected));
    assert_eq!(session.draft, saved);
    let reopened = open_topic(session, topic);
    accepted(&send(session, reopened));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    assert_eq!(cold.draft, saved);
    assert_eq!(estimate(&cold.draft), financial);
    let reopened = open_topic(&mut cold, topic);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
}
fn property_rows(page: &mut IncomeDocument) -> Vec<&mut crate::K1InvestmentProperty> {
    match page {
        IncomeDocument::K1Partnership(v) => v.more.investment_property.iter_mut().collect(),
        IncomeDocument::K1Scorp(v) => v.more.investment_property.iter_mut().collect(),
        IncomeDocument::K1EstateTrust(v) => v
            .investment_property
            .iter_mut()
            .map(|r| &mut r.property)
            .collect(),
        _ => unreachable!(),
    }
}
fn property_page(kind: &str) -> IncomeDocument {
    let properties = [1_000_001, 2_000_001]
        .into_iter()
        .map(|basis| crate::K1InvestmentProperty::Energy {
            own_form: None,
            basis,
            rate: crate::EnergyRate {
                increased_rate: true,
                domestic_content: false,
                energy_community: false,
            },
            id: 0,
        })
        .collect::<Vec<_>>();
    match kind {
        "partner" => {
            let mut p = k1_1065("Actual property parent");
            p.more.investment_property = properties;
            p.more.investment_confirmed = Some(crate::K1InvestmentConfirmation::Yes);
            IncomeDocument::K1Partnership(p)
        }
        "scorp" => {
            let mut p = k1_1120s("Actual property parent");
            p.more.investment_property = properties;
            p.more.investment_confirmed = Some(crate::K1InvestmentConfirmation::Yes);
            IncomeDocument::K1Scorp(p)
        }
        "estate" => {
            let mut p = k1_1041("Actual property parent");
            p.investment_property = properties
                .into_iter()
                .map(|property| crate::EstateInvestmentPropertyPage {
                    property,
                    activities: vec![],
                })
                .collect();
            IncomeDocument::K1EstateTrust(p)
        }
        _ => unreachable!(),
    }
}
#[test]
fn unit47_observed_nested_property_allocator_uses_actual_parent_and_later_row() {
    for kind in ["partner", "scorp", "estate"] {
        let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let request = save_topic(
            &s.draft,
            TaxSection::PassThrough,
            vec![
                document(0, property_page(kind)),
                document(0, property_page(kind)),
            ],
        );
        accepted(&send(&mut s, request));
        for entry in s
            .draft
            .income
            .iter()
            .filter(|e| !crate::investment_credit::k1_property_ids(&e.income).is_empty())
        {
            let source =
                crate::investment_credit::k1_property_credit_sources(&entry.income).unwrap();
            assert_eq!(
                source
                    .iter()
                    .map(|s| s.basis_decrease_cents)
                    .collect::<Vec<_>>(),
                vec![150_000, 300_000]
            );
        }
        let corrected = open_topic(&mut s, TaxSection::PassThrough);
        let mut invalid = corrected.clone();
        let ids = property_rows(&mut request_rows(&mut invalid)[1].document)
            .iter()
            .map(|r| r.id())
            .collect::<Vec<_>>();
        *property_rows(&mut request_rows(&mut invalid)[1].document)[1].id_mut() = ids[0];
        let local = if kind == "estate" {
            "investment_property/1/property"
        } else {
            "more/investment_property/1"
        };
        let path = alloc::format!("[documents/documents/1/document/{local}]");
        reject_both(
            &alloc::format!("property-{kind}-duplicate"),
            &mut s,
            invalid,
            REPEATED,
            &path,
        );
        let mut invalid = corrected.clone();
        let foreign = property_rows(&mut request_rows(&mut invalid)[0].document)[0].id();
        *property_rows(&mut request_rows(&mut invalid)[1].document)[1].id_mut() = foreign;
        reject_both(
            &alloc::format!("property-{kind}-foreign"),
            &mut s,
            invalid,
            FOREIGN,
            &path,
        );
        unchanged_correction(&mut s, corrected, TaxSection::PassThrough);
    }
}
fn sale_rows(page: &mut IncomeDocument) -> &mut Vec<crate::K1Section179Sale> {
    match page {
        IncomeDocument::K1Partnership(v) => &mut v.more.section179_sales,
        IncomeDocument::K1Scorp(v) => &mut v.more.section179_sales,
        _ => unreachable!(),
    }
}
fn sale_page(partner: bool) -> IncomeDocument {
    let mut statement = k1_sale_input_statement();
    statement.alternative =
        crate::K1Section179SaleAmtSource::from_known_regular(&statement.regular);
    if partner {
        let mut p = k1_1065("Actual sale parent");
        p.more.section179_sales = vec![statement.clone(), statement];
        IncomeDocument::K1Partnership(p)
    } else {
        let mut p = k1_1120s("Actual sale parent");
        p.more.section179_sales = vec![statement.clone(), statement];
        IncomeDocument::K1Scorp(p)
    }
}
#[test]
fn unit47_observed_nested_sale_allocator_uses_actual_parent_and_later_row() {
    for partner in [true, false] {
        let kind = if partner { "partner" } else { "scorp" };
        let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let request = save_topic(
            &s.draft,
            TaxSection::PassThrough,
            vec![
                document(0, sale_page(partner)),
                document(0, sale_page(partner)),
            ],
        );
        accepted(&send(&mut s, request));
        let corrected = open_topic(&mut s, TaxSection::PassThrough);
        let mut invalid = corrected.clone();
        let rows = sale_rows(&mut request_rows(&mut invalid)[1].document);
        rows[0].id = rows[1].id; // Sole seen-set fails at the actual later occurrence, row1.
        let path = "[documents/documents/1/document/more/section179_sales/1]";
        reject_both(
            &alloc::format!("sale-{kind}-duplicate"),
            &mut s,
            invalid,
            REPEATED,
            path,
        );
        let mut invalid = corrected.clone();
        let foreign = sale_rows(&mut request_rows(&mut invalid)[0].document)[0].id;
        sale_rows(&mut request_rows(&mut invalid)[1].document)[1].id = foreign;
        reject_both(
            &alloc::format!("sale-{kind}-foreign"),
            &mut s,
            invalid,
            FOREIGN,
            path,
        );
        unchanged_correction(&mut s, corrected, TaxSection::PassThrough);
    }
}
#[test]
fn unit47_observed_annual_allocator_keeps_global_flat_order_and_prepartition_document_index() {
    use crate::tests::investment::investment_statements::{annual_interest, annual_row};
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let source = || {
        IncomeDocument::Int1099(annual_interest(vec![
            annual_row("First", 10_000, 0),
            annual_row("Second", 20_000, 0),
        ]))
    };
    let request = save_topic(
        &s.draft,
        TaxSection::InterestDividends,
        vec![
            document(0, source()),
            document(0, IncomeDocument::TaxableBondPremium(true)),
            document(0, source()),
        ],
    );
    accepted(&send(&mut s, request));
    assert_eq!(return_line(&supported(&s.draft), "2b"), 60_000);
    let mut corrected = open_topic(&mut s, TaxSection::InterestDividends);
    let rows = request_rows(&mut corrected);
    let election = rows.remove(
        rows.iter()
            .position(|r| matches!(r.document, IncomeDocument::TaxableBondPremium(_)))
            .unwrap(),
    );
    rows.insert(1, election);
    assert!(matches!(rows[2].document, IncomeDocument::Int1099(_)));
    let mut invalid = corrected.clone();
    let IncomeDocument::Int1099(v) = &mut request_rows(&mut invalid)[2].document else {
        unreachable!()
    };
    let rows = &mut v
        .adjustments
        .as_mut()
        .unwrap()
        .instrument_records
        .as_mut()
        .unwrap()
        .records;
    rows[1].id = rows[0].id;
    let path = "[documents/documents/2/document/adjustments/instrument_records/records/1]";
    reject_both("annual-duplicate", &mut s, invalid, REPEATED, path);
    let mut invalid = corrected.clone();
    let IncomeDocument::Int1099(v) = &mut request_rows(&mut invalid)[2].document else {
        unreachable!()
    };
    v.adjustments
        .as_mut()
        .unwrap()
        .instrument_records
        .as_mut()
        .unwrap()
        .records[1]
        .id = s.draft.next_id;
    reject_both("annual-foreign", &mut s, invalid, FOREIGN, path);
    unchanged_correction(&mut s, corrected, TaxSection::InterestDividends);
}
#[test]
fn unit47_observed_farm_averaging_catalog_failure_targets_submitted_primitive_source() {
    let mut s = Interview::new(TEST_USER, averaging_return(60_000));
    // Admit the historical source fixture through the public writer before
    // binding an unchanged editor/restart comparison.
    let positive = open_topic(&mut s, TaxSection::Farming);
    accepted(&send(&mut s, positive));
    let mut corrected = open_topic(&mut s, TaxSection::Farming);
    let rows = request_rows(&mut corrected);
    let election = rows.remove(
        rows.iter()
            .position(|r| matches!(r.document, IncomeDocument::FarmAveraging(_)))
            .unwrap(),
    );
    rows.push(election);
    assert_eq!(rows.len(), 2);
    let mut invalid = corrected.clone();
    let IncomeDocument::FarmAveraging(v) = &mut request_rows(&mut invalid)[1].document else {
        unreachable!()
    };
    v.selection.sources = vec![99_999_999_999];
    reject_both(
        "farm-averaging-unoffered",
        &mut s,
        invalid,
        "FARM_AVERAGING_SOURCES_INVALID",
        "[documents/documents/1/document/selection/sources/0]",
    );
    unchanged_correction(&mut s, corrected, TaxSection::Farming);
}
#[test]
fn unit47_observed_disaster_aggregate_and_group_metadata_keep_original_request_positions() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_topic(
        &s.draft,
        TaxSection::Retirement,
        vec![
            document(
                0,
                IncomeDocument::R1099(disaster_r1099(2_000_000, 0, DisasterSpread::OverThreeYears)),
            ),
            document(
                0,
                IncomeDocument::Form8915(earlier_8915(Form8915Year::Filed2024, 2024)),
            ),
            document(
                0,
                IncomeDocument::R1099(disaster_r1099(200_000, 0, DisasterSpread::OverThreeYears)),
            ),
        ],
    );
    accepted(&send(&mut s, request));
    let mut corrected = open_topic(&mut s, TaxSection::Retirement);
    let rows = request_rows(&mut corrected);
    let carry = rows.remove(
        rows.iter()
            .position(|r| matches!(r.document, IncomeDocument::Form8915(_)))
            .unwrap(),
    );
    rows.insert(1, carry);
    for (case, key, leaf) in [
        (0, "DISASTER_ABOVE_LIMIT", "amount"),
        (1, "DISASTER_EARLIER_DIFFERS", "earlier"),
        (2, "DISASTER_ELECTION_DIFFERS", "spread"),
    ] {
        let mut invalid = corrected.clone();
        let IncomeDocument::R1099(v) = &mut request_rows(&mut invalid)[2].document else {
            unreachable!()
        };
        let DisasterDistribution::Qualified {
            amount,
            earlier,
            spread,
            ..
        } = &mut v.disaster
        else {
            unreachable!()
        };
        match case {
            0 => *amount += 1,
            1 => *earlier = 1,
            _ => *spread = DisasterSpread::AllInReturnYear,
        }
        reject_both(
            &alloc::format!("disaster-{case}"),
            &mut s,
            invalid,
            key,
            &alloc::format!("[documents/documents/2/document/disaster/{leaf}]"),
        );
    }
    for trailing in [0, 1] {
        let mut invalid = corrected.clone();
        let IncomeDocument::R1099(v) = &mut request_rows(&mut invalid)[2].document else {
            unreachable!()
        };
        let DisasterDistribution::Qualified { amount, .. } = &mut v.disaster else {
            unreachable!()
        };
        *amount += 1;
        request_rows(&mut invalid).push(document(
            0,
            IncomeDocument::R1099(disaster_r1099(trailing, 0, DisasterSpread::OverThreeYears)),
        ));
        reject_both(
            &alloc::format!("disaster-trailing-{trailing}"),
            &mut s,
            invalid,
            "DISASTER_ABOVE_LIMIT",
            "[documents/documents/2/document/disaster/amount]",
        );
    }
    let mut invalid = corrected.clone();
    let repeated = request_rows(&mut invalid)[1].clone();
    request_rows(&mut invalid).push(repeated);
    reject_both(
        "disaster-form8915-duplicate",
        &mut s,
        invalid,
        "FORM_8915_TWICE",
        "[documents/documents/3/document]",
    );
    unchanged_correction(&mut s, corrected, TaxSection::Retirement);
}
