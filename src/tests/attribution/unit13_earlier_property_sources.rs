use super::*;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;
use crate::tests::support::client::{draft_from_native_records, native_draft_records};

fn fixture() -> FederalDraft {
    let mut d = injured_return((60_000, 10_000), (20_000, 5000), 0, allocation());
    let s = d.screening.as_mut().unwrap();
    s.section1231 = Some(Section1231History { regular: vec![], amt: vec![] });
    s.installment_payments = vec![EarlierInstallment {
        owner: Owner::Spouse,
        sale: InstallmentPayments {
            property: "Earlier business parcel".into(),
            sold: 20230601,
            gross_profit: 600_000,
            contract_price: 1_000_000,
            earlier_payments: 100_000,
            payments: 500_000,
            kind: crate::PropertyCharacter::BusinessProperty {
                used: crate::PropertyUse::Business,
                unrecaptured: 0,
            },
            special: false,
            qbi: InstallmentQbi::NotQualified,
        },
    }];
    s.related_exchanges = vec![EarlierRelatedExchange {
        owner: Owner::Taxpayer,
        exchange: RelatedExchange {
            property: "Earlier exchanged business parcel".into(),
            last_transfer: 20240501,
            disposed: 20260430,
            exception: RelatedException::None,
            deferred: 200_000,
            kind: crate::PropertyCharacter::BusinessProperty {
                used: crate::PropertyUse::Business,
                unrecaptured: 0,
            },
            special: false,
        },
    }];
    d
}

#[test]
fn unit13_earlier_property_final_capital_keeps_both_actual_sellers() {
    let mut d = fixture();
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain"), 5000);
        assert_eq!(value(&r, "agi"), 85_000);
        let expected = if injured == Owner::Taxpayer { [2000, 3000] } else { [3000, 2000] };
        assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], expected);
        assert_injured_ordinary_unchanged(&d, &r, estimate);
    }
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn unit13_earlier_property_lookback_keeps_the_actual_single_gain_seller() {
    let mut d = fixture();
    let s = d.screening.as_mut().unwrap();
    s.related_exchanges.clear();
    s.section1231 = Some(section1231_history(&[(2025, 1000, 1000)], &[]));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 2000);
    assert_eq!(value(&r, "form4797_line19"), 1000);
    assert_eq!(value(&r, "agi"), 83_000);
    assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [0, 3000]);
    assert_injured_ordinary_unchanged(&d, &r, estimate);
}

#[test]
fn unit13_earlier_property_unrecaptured_class_keeps_retained_source_owner() {
    let mut d = fixture();
    let s = d.screening.as_mut().unwrap();
    let crate::PropertyCharacter::BusinessProperty { unrecaptured, .. } =
        &mut s.installment_payments[0].sale.kind
    else {
        panic!()
    };
    *unrecaptured = 100_000;
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 5000);
    assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [2000, 3000]);
    assert_injured_ordinary_unchanged(&d, &r, estimate);
}

#[test]
fn unit13_earlier_property_identity_survives_capital_prefix_and_reordering() {
    let mut d = fixture();
    let s = d.screening.as_mut().unwrap();
    let mut capital = s.installment_payments[0].clone();
    capital.owner = Owner::Taxpayer;
    capital.sale.kind = crate::PropertyCharacter::LongTermCapital;
    s.installment_payments.insert(0, capital);
    // The same labels and amounts never substitute for the native source
    // index and the retained seller; a preceding capital record is skipped.
    for reverse in [false, true] {
        if reverse {
            d.screening.as_mut().unwrap().installment_payments.reverse();
        }
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain"), 8000);
        assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [5000, 3000]);
        assert_injured_ordinary_unchanged(&d, &r, estimate);
    }
}

#[test]
fn unit13_earlier_property_ambiguous_exchange_lookback_keeps_native_boundary() {
    let mut d = fixture();
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 1000, 1000)], &[]));
    // The existing exchange/QBI boundary remains an ordinary-return issue;
    // owner attribution must not turn it into a supported joint return.
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|issue| issue.display.key
        == "ISSUE_RELATED_EXCHANGE_BUSINESS_GAIN_WITH_SECTION_1231_LOOKBACK"));
    assert!(!r.lines.iter().any(|l| l.id == "form_8379_injured_refund_estimate"));
    assert_injured_ordinary_unchanged(&d, &r, estimate);
}

#[test]
fn unit13_earlier_property_actual_client_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::{accepted, open_topic};
    let selected = std::env::var("TAX_UNIT13_EARLIER_PROFILE").ok();
    for profile in ["installment", "exchange"] {
        if selected.as_deref().is_some_and(|p| p != profile) {
            continue;
        }
        let mut session = Interview::new(TEST_USER, fixture());
        let mut page = open_topic(&mut session, TaxSection::InvestmentSales);
        let native = NativeInterviewCase("TAX_UNIT13_EARLIER");
        if native.required_checked_calculation(&mut session) {
            return;
        }
        if native.seed(&page) {
            return;
        }
        let request = if let Some(request) = native.checked_input("PAYLOAD") {
            request
        } else {
            let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
            if profile == "installment" {
                let row = documents
                    .documents
                    .iter_mut()
                    .find(|d| matches!(d.document, IncomeDocument::InstallmentPayments(_)))
                    .unwrap();
                row.owner = Owner::Taxpayer;
                let IncomeDocument::InstallmentPayments(sale) = &mut row.document else { panic!() };
                sale.payments = 700_000;
            } else {
                let row = documents
                    .documents
                    .iter_mut()
                    .find(|d| matches!(d.document, IncomeDocument::RelatedExchange(_)))
                    .unwrap();
                row.owner = Owner::Spouse;
                let IncomeDocument::RelatedExchange(exchange) = &mut row.document else { panic!() };
                exchange.deferred = 400_000;
            }
            page
        };
        accepted(&send(&mut session, request));
        let check = |d: &FederalDraft| {
            let r = supported(d);
            let parts = if profile == "installment" { [6200, 0] } else { [0, 7000] };
            assert_eq!(value(&r, "capital_gain"), parts.iter().sum::<i64>());
            assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], parts);
            assert_injured_ordinary_unchanged(d, &r, estimate);
        };
        check(&session.draft);
        let tax_year = session.draft.tax_year;
        let stored = native_draft_records(&session.draft);
        let draft = draft_from_native_records(tax_year, &stored);
        assert_eq!(draft, session.draft);
        let mut reopened = restart(draft);
        let shown = open_topic(&mut reopened, TaxSection::InvestmentSales);
        native.output(&shown, &stored);
        let accepted_draft = reopened.draft.clone();
        accepted(&send(&mut reopened, shown));
        assert_eq!(reopened.draft, accepted_draft);
        check(&reopened.draft);
    }
}
