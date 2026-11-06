use crate::rrb_document::Rrb1099Paper;
use crate::tests::*;

pub(in crate::tests) fn espp_acquisition_paper(
    lot: &EsppSale,
    transferred_shares: f64,
    legal_transfer: u32,
    box8: Option<i64>,
) -> Form3922 {
    Form3922 {
        corporation: TaxPayerIdentity {
            name: "Example Stock Plan Corporation".into(),
            ..Default::default()
        },
        employee: Default::default(),
        account_number: "ESPP-01".into(),
        corrected: false,
        granted: lot.granted,
        exercised: lot.exercised,
        grant_market: lot.grant_market,
        market_price: lot.market_price,
        exercise_price: lot.exercise_price,
        shares: transferred_shares,
        transferred: legal_transfer,
        grant_price: box8,
    }
}

pub(in crate::tests) fn rrb_benefit_paper() -> Rrb1099Paper {
    Rrb1099Paper {
        issuer: TaxPayerIdentity { name: "Railroad Retirement Board".into(), ..Default::default() },
        year: 2026,
        claim_number: "A000123456-01".into(),
        recipient_id: "987-65-4320".into(),
        benefits_paid: 2_000_000,
        benefits_repaid: 200_000,
        benefits: 1_800_000,
        withholding: 10_000,
        medicare_premiums: 240_000,
        ..Default::default()
    }
}
