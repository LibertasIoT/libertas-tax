use alloc::string::ToString;
use libertas::SchemaPath;
// Captured full-suite refusals exposed posted-row locator gaps. These controls
// preserve the actual sole allocator/source owners and accepted Draft lifecycle.
use super::unit47_credit_source_validation::{credit_estimate_documents_from_save, credit_refusal};
use super::*;
use crate::tests::support::client::income_pages::{open_documents, save_documents};

const REPEATED: &str = "ISSUE_INTERVIEW_A_RECORD_IDENTITY_IS_REPEATED_IN_EE6DC425";
const FOREIGN: &str = "ISSUE_INTERVIEW_A_RECORD_NO_LONGER_BELONGS_TO_51E52C2C";

fn request_rows(request: &mut P) -> &mut Vec<crate::IncomeRecordItem> {
    let P::SaveIncomes { documents, .. } = request else { unreachable!() };
    &mut documents.documents
}
fn k1_rows(request: &mut P) -> &mut Vec<crate::K1DocumentRow> {
    let P::SaveDocuments { documents, .. } = request else { unreachable!() };
    &mut documents.common.k1.documents
}
fn reject_both(case: &str, session: &mut Interview, invalid: P, key: &str, path: &str) {
    let invalid = P::from_avro(&invalid.to_avro()).unwrap();
    let running = credit_estimate_documents_from_save(&invalid);
    credit_refusal(&alloc::format!("observed-gap-{case}-save"), session, invalid, key, path);
    credit_refusal(&alloc::format!("observed-gap-{case}-estimate"), session, running, key, path);
}
fn unchanged_correction(session: &mut Interview, corrected: P, topic: TaxSection) {
    let source = matches!(&corrected, P::SaveDocuments { .. });
    let saved = session.draft.clone();
    let dirty = session.dirty;
    let financial = estimate(&saved);
    let response = crate::tests::support::client::income_pages::candidate_save_review(
        session,
        credit_estimate_documents_from_save(&corrected),
    );
    assert!(matches!(
        (source, response),
        (true, Reply::Response(P::Review { .. }))
            | (false, Reply::Response(P::IncomeRunningEstimate { .. }))
    ));
    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, corrected));
    assert_eq!(session.draft, saved);
    let reopened = if source { open_documents(session) } else { open_topic(session, topic) };
    accepted(&send(session, reopened));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    assert_eq!(cold.draft, saved);
    assert_eq!(estimate(&cold.draft), financial);
    let reopened = if source { open_documents(&mut cold) } else { open_topic(&mut cold, topic) };
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
}
fn property_rows<T>(
    page: &mut crate::K1Document,
    edit: impl FnOnce(Vec<&mut crate::K1InvestmentProperty>) -> T,
) -> T {
    match page {
        crate::K1Document::K1Partnership(v) => {
            edit(v.statements.investment_property.iter_mut().collect())
        }
        crate::K1Document::K1Scorp(v) => {
            edit(v.statements.investment_property.iter_mut().collect())
        }
        crate::K1Document::K1EstateTrust(v) => {
            let mut page = edit_estate_document(v);
            edit(page.investment_property.iter_mut().map(|r| &mut r.property).collect())
        }
        _ => unreachable!(),
    }
}
fn property_page(kind: &str) -> IncomeSource {
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
            IncomeSource::K1 { value: crate::K1Document::K1Partnership((p).into()) }
        }
        "scorp" => {
            let mut p = k1_1120s("Actual property parent");
            p.more.investment_property = properties;
            p.more.investment_confirmed = Some(crate::K1InvestmentConfirmation::Yes);
            IncomeSource::K1 { value: crate::K1Document::K1Scorp((p).into()) }
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
            IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(estate_document(p)) }
        }
        _ => unreachable!(),
    }
}
#[test]
fn unit47_observed_nested_property_allocator_uses_actual_parent_and_later_row() {
    for kind in ["partner", "scorp", "estate"] {
        let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let request = save_documents(
            &mut s,
            vec![document(0, property_page(kind)), document(0, property_page(kind))],
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
                source.iter().map(|s| s.basis_decrease_cents).collect::<Vec<_>>(),
                vec![150_000, 300_000]
            );
        }
        let corrected = open_documents(&mut s);
        let mut invalid = corrected.clone();
        let ids = property_rows(&mut k1_rows(&mut invalid)[1].document, |rows| {
            rows.iter().map(|r| r.id()).collect::<Vec<_>>()
        });
        property_rows(&mut k1_rows(&mut invalid)[1].document, |mut rows| {
            *rows[1].id_mut() = ids[0];
        });
        let local = match kind {
            "estate" => "statements/investment_property/1/property",
            "partner" => "statements/investment_property/1",
            _ => "more/investment_property/1",
        };
        let path = alloc::format!("[documents/common/k1/documents/1/document/{local}]");
        reject_both(&alloc::format!("property-{kind}-duplicate"), &mut s, invalid, REPEATED, &path);
        let mut invalid = corrected.clone();
        let foreign = property_rows(&mut k1_rows(&mut invalid)[0].document, |rows| rows[0].id());
        property_rows(&mut k1_rows(&mut invalid)[1].document, |mut rows| {
            *rows[1].id_mut() = foreign;
        });
        reject_both(&alloc::format!("property-{kind}-foreign"), &mut s, invalid, FOREIGN, &path);
        unchanged_correction(&mut s, corrected, TaxSection::PassThrough);
    }
}
fn sale_rows(page: &mut crate::K1Document) -> &mut Vec<crate::K1Section179Sale> {
    match page {
        crate::K1Document::K1Partnership(v) => &mut v.statements.section179_sales,
        crate::K1Document::K1Scorp(v) => &mut v.statements.section179_sales,
        _ => unreachable!(),
    }
}
fn sale_page(partner: bool) -> IncomeSource {
    let mut statement = k1_sale_input_statement();
    statement.alternative =
        crate::K1Section179SaleAmtSource::from_known_regular(&statement.regular);
    if partner {
        let mut p = k1_1065("Actual sale parent");
        p.more.section179_sales = vec![statement.clone(), statement];
        IncomeSource::K1 { value: crate::K1Document::K1Partnership((p).into()) }
    } else {
        let mut p = k1_1120s("Actual sale parent");
        p.more.section179_sales = vec![statement.clone(), statement];
        IncomeSource::K1 { value: crate::K1Document::K1Scorp((p).into()) }
    }
}
#[test]
fn unit47_observed_nested_sale_allocator_uses_actual_parent_and_later_row() {
    for partner in [true, false] {
        let kind = if partner { "partner" } else { "scorp" };
        let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let request = save_documents(
            &mut s,
            vec![document(0, sale_page(partner)), document(0, sale_page(partner))],
        );
        accepted(&send(&mut s, request));
        let corrected = open_documents(&mut s);
        let mut invalid = corrected.clone();
        let rows = sale_rows(&mut k1_rows(&mut invalid)[1].document);
        rows[0].id = rows[1].id; // Sole seen-set fails at the actual later occurrence, row1.
        let path = if partner {
            "[documents/common/k1/documents/1/document/statements/section179_sales/1]"
        } else {
            "[documents/common/k1/documents/1/document/more/section179_sales/1]"
        };
        reject_both(&alloc::format!("sale-{kind}-duplicate"), &mut s, invalid, REPEATED, path);
        let mut invalid = corrected.clone();
        let foreign = sale_rows(&mut k1_rows(&mut invalid)[0].document)[0].id;
        sale_rows(&mut k1_rows(&mut invalid)[1].document)[1].id = foreign;
        reject_both(&alloc::format!("sale-{kind}-foreign"), &mut s, invalid, FOREIGN, path);
        unchanged_correction(&mut s, corrected, TaxSection::PassThrough);
    }
}
#[test]
fn unit47_observed_annual_allocator_keeps_global_flat_order_and_prepartition_document_index() {
    use crate::tests::investment::investment_statements::{annual_interest, annual_row};
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let source = || IncomeSource::InterestDividend {
        value: crate::InterestDividendDocument::Int1099(interest_document(annual_interest(vec![
            annual_row("First", 10_000, 0),
            annual_row("Second", 20_000, 0),
        ]))),
    };
    let request = save_documents(&mut s, vec![document(0, source()), document(0, source())]);
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut s, request,
    ));
    let request = save_topic(
        &mut s,
        TaxSection::InterestDividends,
        vec![document(
            0,
            IncomeSource::Record { value: crate::IncomeRecord::TaxableBondPremium(true) },
        )],
    );
    accepted(&send(&mut s, request));
    assert_eq!(return_line(&supported(&s.draft), "2b"), 60_000);
    // The separate election does not occupy a source category row.
    let corrected = open_documents(&mut s);
    let P::SaveDocuments { documents, .. } = &corrected else { unreachable!() };
    assert_eq!(documents.common.interest_dividends.documents.len(), 2);
    assert!(matches!(
        documents.common.interest_dividends.documents[1].document,
        crate::InterestDividendDocument::Int1099(_)
    ));
    let id = documents.common.interest_dividends.documents[1].id;
    let corrected = open_topic(&mut s, TaxSection::InterestDividends);
    let mut invalid = corrected.clone();
    let P::SaveIncomes { followups, .. } = &mut invalid else { unreachable!() };
    let index = followups
        .iter()
        .position(|row| row.key == alloc::format!("income/{id}/adjustments"))
        .unwrap();
    let crate::DocumentAnswer::InterestAdjustment { value: Some(v) } = &mut followups[index].answer
    else {
        unreachable!()
    };
    let rows = &mut v.instrument_records.as_mut().unwrap().records;
    rows[1].id = rows[0].id;
    let path = SchemaPath::root()
        .field("followups")
        .key(followups[index].key.as_str())
        .field("answer")
        .field("value")
        .field("instrument_records")
        .field("records")
        .index(1)
        .to_string();
    reject_both("annual-duplicate", &mut s, invalid, REPEATED, &path);
    let mut invalid = corrected.clone();
    let P::SaveIncomes { followups, .. } = &mut invalid else { unreachable!() };
    let crate::DocumentAnswer::InterestAdjustment { value: Some(v) } = &mut followups[index].answer
    else {
        unreachable!()
    };
    v.instrument_records.as_mut().unwrap().records[1].id = s.draft.next_id;
    reject_both("annual-foreign", &mut s, invalid, FOREIGN, &path);
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
            .position(|r| matches!(r.document, crate::IncomeRecord::FarmAveraging(_)))
            .unwrap(),
    );
    rows.push(election);
    assert_eq!(rows.len(), 2);
    let mut invalid = corrected.clone();
    let crate::IncomeRecord::FarmAveraging(v) = &mut request_rows(&mut invalid)[1].document else {
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
    let request =
        save_documents(
            &mut s,
            vec![
                document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::R1099(retirement_document(
                            disaster_r1099(2_000_000, 0, DisasterSpread::OverThreeYears),
                        )),
                    },
                ),
                document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::R1099(retirement_document(
                            disaster_r1099(200_000, 0, DisasterSpread::OverThreeYears),
                        )),
                    },
                ),
            ],
        );
    let desired = crate::tests::support::client::income_pages::page_documents(&request);
    accepted(&send(&mut s, request));
    let ids: Vec<_> = s
        .draft
        .income
        .iter()
        .filter(|entry| matches!(entry.income, FederalIncome::Retirement { .. }))
        .map(|entry| entry.id)
        .collect();
    let mut answers =
        crate::tests::support::client::income_pages::information_return_answers_for_topic(
            &mut s,
            &desired,
            TaxSection::Retirement,
        )
        .unwrap();
    let P::SaveIncomes { followups, .. } = &mut answers else { unreachable!() };
    for (id, amount) in ids.into_iter().zip([2_000_000, 200_000]) {
        let row = followups
            .iter_mut()
            .find(|row| row.key == alloc::format!("income/{id}/disaster"))
            .unwrap();
        row.answer = crate::DocumentAnswer::RetirementDisaster {
            value: Some(disaster_fold(amount, 0, DisasterSpread::OverThreeYears)),
        };
    }
    accepted(&send(&mut s, answers));
    let carry = save_topic(
        &mut s,
        TaxSection::Retirement,
        vec![document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::Form8915(earlier_8915(Form8915Year::Filed2024, 2024)),
            },
        )],
    );
    accepted(&send(&mut s, carry));
    let corrected_source = open_documents(&mut s);
    let corrected = open_topic(&mut s, TaxSection::Retirement);
    let source_ids: Vec<_> =
        crate::tests::support::client::income_pages::page_documents(&corrected_source)
            .into_iter()
            .filter(|row| matches!(row.document, IncomeSource::Retirement { .. }))
            .map(|row| row.id)
            .collect();
    for (case, key, leaf) in [
        (0, "DISASTER_ABOVE_LIMIT", "amount"),
        (1, "DISASTER_EARLIER_DIFFERS", "earlier"),
        (2, "DISASTER_ELECTION_DIFFERS", "spread"),
    ] {
        let mut invalid = corrected.clone();
        let P::SaveIncomes { followups, .. } = &mut invalid else { unreachable!() };
        let crate::DocumentAnswer::RetirementDisaster {
            value: Some(DisasterDistribution::Qualified { amount, earlier, spread, .. }),
        } = &mut followups
            .iter_mut()
            .find(|row| row.key == alloc::format!("income/{}/disaster", source_ids[1]))
            .unwrap()
            .answer
        else {
            unreachable!()
        };
        match case {
            0 => *amount += 1,
            1 => *earlier = 1,
            _ => *spread = DisasterSpread::AllInReturnYear,
        }
        let path =
            retirement_followup_path(&invalid, source_ids[1], &alloc::format!("disaster/{leaf}"));
        reject_both(&alloc::format!("disaster-{case}"), &mut s, invalid, key, &path);
    }
    for trailing in [0, 1] {
        let mut invalid = corrected_source.clone();
        let P::SaveDocuments { documents, .. } = &mut invalid else { unreachable!() };
        let mut row = documents.common.retirement.documents[1].clone();
        row.id = 0;
        row.document = crate::RetirementDocument::R1099(retirement_document(disaster_r1099(
            trailing,
            0,
            DisasterSpread::OverThreeYears,
        )));
        documents.common.retirement.documents.push(row);
        let mut candidate = restart(s.draft.clone());
        accepted(&send(&mut candidate, invalid));
        let trailing_id = candidate
            .draft
            .income
            .iter()
            .rfind(|entry| matches!(entry.income, FederalIncome::Retirement { .. }))
            .unwrap()
            .id;
        let mut invalid = open_topic(&mut candidate, TaxSection::Retirement);
        let P::SaveIncomes { followups, .. } = &mut invalid else { unreachable!() };
        let crate::DocumentAnswer::RetirementDisaster {
            value: Some(DisasterDistribution::Qualified { amount, .. }),
        } = &mut followups
            .iter_mut()
            .find(|row| row.key == alloc::format!("income/{}/disaster", source_ids[1]))
            .unwrap()
            .answer
        else {
            unreachable!()
        };
        *amount += 1;
        let row = followups
            .iter_mut()
            .find(|row| row.key == alloc::format!("income/{trailing_id}/disaster"))
            .unwrap();
        row.answer = crate::DocumentAnswer::RetirementDisaster {
            value: Some(disaster_fold(trailing, 0, DisasterSpread::OverThreeYears)),
        };
        let path = retirement_followup_path(&invalid, source_ids[1], "disaster/amount");
        reject_both(
            &alloc::format!("disaster-trailing-{trailing}"),
            &mut candidate,
            invalid,
            "DISASTER_ABOVE_LIMIT",
            &path,
        );
    }
    let record_correction = open_topic(&mut s, TaxSection::Retirement);
    let mut invalid = record_correction.clone();
    let repeated = request_rows(&mut invalid)[0].clone();
    request_rows(&mut invalid).push(repeated);
    reject_both(
        "disaster-form8915-duplicate",
        &mut s,
        invalid,
        "FORM_8915_TWICE",
        "[documents/documents/1/document]",
    );
    unchanged_correction(&mut s, record_correction, TaxSection::Retirement);
    unchanged_correction(&mut s, corrected, TaxSection::Retirement);
    unchanged_correction(&mut s, corrected_source, TaxSection::Retirement);
}
