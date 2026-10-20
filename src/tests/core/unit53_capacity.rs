extern crate std;

use super::*;
use crate::tests::cross::interview::{document, open_topic};
use libertas::data::{EncodedKey, wire};
use libertas_macros::{LibertasKeyCodec, LibertasKeyDecode};

// Measurement of the approved candidate layout, not a published App key.
#[derive(Debug, PartialEq, LibertasKeyCodec, LibertasKeyDecode)]
enum CapacityKey {
    ReturnRecord { user: libertas::LibertasUser, tax_year: i32, copy: i64, kind: u16, id: i64 },
}

#[test]
fn admitted_brokerage_lots_measure_semantic_transaction_capacity() {
    for count in [2048, 4097] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut page = open_topic(&mut session, TaxSection::InvestmentSales);
        let P::SaveIncomes { documents, .. } = &mut page else { unreachable!() };
        documents.documents = (0..count)
            .map(|index| {
                document(
                    0,
                    IncomeDocument::Sale1099B(Sale1099B {
                        // Descriptions have no App maximum. This deliberately long
                        // synthetic substitute-statement label identifies each lot.
                        description: alloc::format!(
                            "Synthetic brokerage lot {index:04} {}",
                            "X".repeat(512)
                        ),
                        digital_asset: false,
                        term: Some(GainTerm::Short),
                        proceeds: 90_000,
                        basis: Some(100_000),
                        reporting: BasisReporting::ReportedToIrs,
                        use_kind: SaleUse::Investment { wash_sale: 1_000 },
                        collectible: false,
                        market_discount: 0,
                        several_sales: false,
                        withholding: 0,
                        adjustment: SaleAdjustment::BasisCorrection { reported_basis: 99_000 },
                        special: SaleSpecial::NoneOfThese,
                        state_details: crate::StateWithholdingFields {
                            payer: crate::TaxPayerIdentity {
                                name: "Synthetic Brokerage".into(),
                                federal_id: "00-0000000".into(),
                                address: "123 Synthetic Street, Albany NY 00000".into(),
                            },
                            states: vec![crate::StateTaxWithholding {
                                state: crate::TaxState::NY,
                                payer_state_id: "SYNTHETIC-0000".into(),
                                withholding: 0,
                            }],
                        },
                    }),
                )
            })
            .collect();
        let expected: Vec<_> =
            documents.documents.iter().map(|item| item.document.clone()).collect();
        let original_revision = session.draft.revision;
        let request_bytes = page.to_avro().len();
        accepted(&send(&mut session, page));
        assert_eq!(session.draft.revision, original_revision + 1);
        assert_eq!(session.draft.income.len(), count + 1, "the existing W-2 is retained");
        let reopened = open_topic(&mut session, TaxSection::InvestmentSales);
        let P::SaveIncomes { documents, .. } = reopened else { unreachable!() };
        assert_eq!(
            documents.documents.iter().map(|item| &item.document).collect::<Vec<_>>(),
            expected.iter().collect::<Vec<_>>(),
            "every long label, correction and nested state fact survives native projection"
        );
        assert!(session.draft.valid_stored(), "{:?}", session.draft.stored_issue());
        crate::tests::core_tests::storage::rehearse_heavy(&session.draft);
        let lots: Vec<_> = session
            .draft
            .income
            .iter()
            .filter(|entry| matches!(entry.income, FederalIncome::Sale { .. }))
            .collect();
        assert_eq!(lots.len(), count);
        let records = crate::income_records::project(TEST_USER, 0, &session.draft).unwrap();
        let mut payload_bytes = 0;
        let mut max_value = 0;
        let mut cold_records = Vec::new();
        for (key, value) in records {
            let encoded = EncodedKey::new(&key).unwrap();
            assert_eq!(encoded.as_bytes().len(), 32);
            assert_eq!(encoded.decode().unwrap(), key);
            let bytes = value.to_avro();
            assert!(bytes.len() < wire::MAX_VALUE_BYTES);
            max_value = max_value.max(bytes.len());
            // Hub mutation accounting is encoded key bytes plus encoded value bytes.
            payload_bytes += bytes.len() + encoded.as_bytes().len();
            cold_records.push((encoded.decode().unwrap(), TaxAppData::from_avro(&bytes).unwrap()));
        }
        cold_records.reverse();
        let mut cold = session.draft.clone();
        cold.income.clear();
        crate::income_records::assemble(
            TEST_USER,
            0,
            &mut cold,
            session.draft.income.len(),
            cold_records,
        )
        .unwrap();
        assert_eq!(cold, session.draft);
        let monolith =
            TaxAppData::Draft { draft: session.draft.clone().try_into().unwrap() }.to_avro();
        let restored = restart(session.draft.clone());
        assert_eq!(restored.draft, session.draft);
        assert!(request_bytes < wire::MAX_REQUEST_BYTES);
        assert!(payload_bytes < wire::MAX_REQUEST_BYTES);
        // Income records alone are a lower bound: headers, people, sections,
        // order metadata, copy counter and migration deletions add work.
        std::println!(
            "Unit53 admitted lots={count} request={request_bytes} monolith={} income_record_mutation_bytes={payload_bytes} income_mutations={} max_value={max_value}",
            monolith.len(),
            session.draft.income.len()
        );
    }
}

#[test]
fn admitted_sale_avro_crosses_value_boundary_without_a_source_text_cap() {
    for extra in [0, 1] {
        let mut draft = complete(FederalStatus::Single);
        add(&mut draft, income(personal_sale(900, 1000, GainTerm::Short)));
        let mut session = Interview::new(TEST_USER, draft);
        let entry = session.draft.income.last().unwrap();
        let target = wire::MAX_VALUE_BYTES + extra;
        // Compute the description size with the actual stored source encoder,
        // including the Avro string-length prefix changing width.
        let mut candidate = entry.clone();
        candidate.label = "X".repeat(target);
        let overhead = candidate.to_avro().len() - candidate.label.len();
        candidate.label.truncate(target - overhead);
        assert_eq!(candidate.to_avro().len(), target);
        let mut page = open_topic(&mut session, TaxSection::InvestmentSales);
        let P::SaveIncomes { documents, .. } = &mut page else { unreachable!() };
        let IncomeDocument::Sale1099B(sale) = &mut documents.documents[0].document else {
            unreachable!()
        };
        sale.description = candidate.label.clone();
        assert!(page.to_avro().len() < wire::MAX_REQUEST_BYTES);
        accepted(&send(&mut session, page));
        assert!(session.draft.valid_stored());
        let entry = session.draft.income.last().unwrap();
        assert_eq!(entry.label, candidate.label);
        let value = entry.to_avro();
        assert_eq!(value.len(), target);
        let request = wire::Request {
            version: wire::VERSION,
            id: 1,
            checked: true,
            command: wire::Command::ExecuteInTransaction {
                transaction: 1,
                operation: wire::Operation::Put {
                    key: EncodedKey::new(&CapacityKey::ReturnRecord {
                        user: TEST_USER,
                        tax_year: 2026,
                        copy: 0,
                        kind: 4,
                        id: entry.id,
                    })
                    .unwrap()
                    .as_bytes()
                    .to_vec(),
                    value,
                },
            },
        };
        assert_eq!(wire::decode_request(&wire::encode(&request)).is_ok(), extra == 0);
        std::println!(
            "Unit53 native admitted individual sale payload={target} wire_accepts={}",
            extra == 0
        );
    }
}

#[test]
fn admitted_nontrader_deductions_also_need_section_capacity_review() {
    use crate::tests::cross::interview::{at_the_gifts_page, open_itemized_topic};
    let mut session = at_the_gifts_page(DeductionRoute::Itemize, 0);
    let mut page = open_itemized_topic(&mut session, TaxSection::GiftCarryovers);
    let P::SaveGiftCarryovers { value, .. } = &mut page else { unreachable!() };
    let mut appraisal = gift_appraisal(0, 20261115);
    // A synthetic boundary witness, not a realistic appraiser name or a
    // claimed maximum: source text and section collections have no App cap.
    appraisal.appraiser = "X".repeat(wire::MAX_VALUE_BYTES);
    value.appraisals.push(appraisal);
    let request_bytes = page.to_avro().len();
    assert!(request_bytes < wire::MAX_REQUEST_BYTES);
    accepted(&send(&mut session, page));
    assert!(session.draft.valid_stored());
    let section = session.draft.deductions.as_ref().unwrap();
    assert_eq!(section.appraisals.len(), 1);
    assert_eq!(section.appraisals[0].appraiser.len(), wire::MAX_VALUE_BYTES);
    let section_bytes = section.to_avro().len();
    assert!(section_bytes > wire::MAX_VALUE_BYTES);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
    std::println!(
        "Unit53 admitted nontrader deductions request={request_bytes} section_payload={section_bytes}"
    );
}
