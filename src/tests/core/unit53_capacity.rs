extern crate std;

use super::*;
use libertas::AvroEncode;
use libertas::data::{EncodedKey, wire};
use libertas_macros::{LibertasKeyCodec, LibertasKeyDecode};

// Mirrors the fixed-width return-record key layout for wire encoding checks.
#[derive(Debug, PartialEq, LibertasKeyCodec, LibertasKeyDecode)]
enum CapacityKey {
    ReturnRecord { user: libertas::LibertasUser, tax_year: i32, copy: i64, kind: u16, id: i64 },
}

#[test]
fn admitted_brokerage_lots_preserve_multiple_complete_records() {
    crate::tests::support::assert::admitted_brokerage_lots_capacity(2);
}

#[test]
fn admitted_sale_preserves_text_across_avro_length_prefix_boundary() {
    let mut previous_value_bytes = None;
    for label_bytes in [63, 64] {
        let mut draft = complete(FederalStatus::Single);
        add(&mut draft, income(personal_sale(900, 1000, GainTerm::Short)));
        let mut session = Interview::new(TEST_USER, draft);
        let entry = session.draft.income.last().unwrap();
        let mut candidate = entry.clone();
        // Avro's signed string length needs a second byte at 64 UTF-8 bytes.
        candidate.label = "X".repeat(label_bytes);
        // The stored paper prints the same text as the sale description.
        let FederalIncome::Sale { data } = &mut candidate.income else { unreachable!() };
        data.form.description = candidate.label.clone();
        let value_bytes = candidate.to_avro().len();
        if let Some(previous) = previous_value_bytes {
            // One more character and one more length-prefix byte, in each of the two texts.
            assert_eq!(value_bytes, previous + 4);
        }
        previous_value_bytes = Some(value_bytes);
        let mut page =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
        let crate::InvestmentSaleDocument::Sale1099B(sale) =
            &mut documents.common.investment_sales.documents[0].document
        else {
            unreachable!()
        };
        sale.description = candidate.label.clone();
        assert!(page.to_avro().len() < wire::MAX_REQUEST_BYTES);
        accepted(&send(&mut session, page));
        assert!(session.draft.valid_stored());
        let entry = session.draft.income.last().unwrap();
        assert_eq!(entry.label, candidate.label);
        let value = entry.to_avro();
        assert_eq!(value.len(), value_bytes);
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
        assert_eq!(wire::decode_request(&wire::encode(&request)), Ok(request));
    }
}

#[test]
fn stored_value_declared_length_rejects_first_byte_over_wire_limit() {
    let request = wire::Request {
        version: wire::VERSION,
        id: 1,
        checked: true,
        command: wire::Command::ExecuteInTransaction {
            transaction: 1,
            operation: wire::Operation::Put { key: vec![1], value: vec![] },
        },
    };
    let encoded = wire::encode(&request);
    assert_eq!(wire::decode_request(&encoded), Ok(request));
    for (length, error) in [
        (wire::MAX_VALUE_BYTES, "Truncated storage bytes"),
        (wire::MAX_VALUE_BYTES + 1, "Storage byte length exceeds limit"),
    ] {
        // The final field is the empty value's length. A declared length checks
        // the real wire boundary before payload allocation or truncation checks.
        let mut frame = encoded[..encoded.len() - 1].to_vec();
        (length as i64).avro_encode(&mut frame);
        assert_eq!(wire::decode_request(&frame), Err(error));
    }
}

#[test]
fn admitted_nontrader_appraisals_preserve_section_text_and_order() {
    use crate::tests::cross::interview::{at_the_gifts_page, open_itemized_topic};
    let mut session = at_the_gifts_page(Some(ItemizeOverride::Itemize), 0);
    let mut page = open_itemized_topic(&mut session, TaxSection::GiftCarryovers);
    let P::SaveGiftCarryovers { value, .. } = &mut page else { unreachable!() };
    for length in [63, 64] {
        let mut appraisal = gift_appraisal(0, 20261115);
        appraisal.appraiser = "X".repeat(length);
        value.appraisals.push(appraisal);
    }
    let expected = value.appraisals.clone();
    assert!(page.to_avro().len() < wire::MAX_REQUEST_BYTES);
    accepted(&send(&mut session, page));
    assert!(session.draft.valid_stored());
    let section = session.draft.deductions.as_ref().unwrap();
    assert_eq!(section.appraisals.len(), 2);
    // Saving assigns identities; every supplied appraisal fact is retained.
    for (actual, mut expected) in section.appraisals.iter().zip(expected) {
        assert!(actual.id > 0);
        expected.id = actual.id;
        assert_eq!(actual, &expected);
    }
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}
