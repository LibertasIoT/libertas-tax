use super::*;
use crate::tests::support::assert::assert_nonallocation_ordinary_unchanged as ordinary_unchanged;
use crate::tests::support::client::{
    draft_from_native_records, native_draft_records, write_native_draft_records,
};

fn borrowing(d: &mut FederalDraft, source: i64, paid: i64) {
    d.deductions.as_mut().unwrap().investment_interest = Some(InvestmentBorrowing {
        loans: vec![InvestmentLoan {
            source,
            ..native_investment_loan("Royalty acquisition loan".into(), paid, 0, 0)
        }],
        election: InvestmentElection::None,
        income_use: InvestmentInterestUse::Ordinary,
        limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
    });
}

fn fixture() -> FederalDraft {
    let mut d = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
    let mut first = income(royalty(11_000, 1_000));
    let FederalIncome::Royalty { data } = &mut first.income else { panic!() };
    data.kind = RoyaltyKind::OilGas;
    data.depletion =
        RoyaltyDepletion::Cost { adjusted_basis: 2_000_000, recoverable_units: 10, units_sold: 1 };
    add(&mut d, first);
    let source = d.income.last().unwrap().id;
    let mut second = income(royalty(20_000, 4_000));
    second.owner = crate::DocumentOwner::Spouses;
    let FederalIncome::Royalty { data } = &mut second.income else { panic!() };
    data.kind = RoyaltyKind::Mineral { rate: RoyaltyMineralRate::Ten };
    data.depletion = RoyaltyDepletion::Percentage {
        adjusted_basis: 5_000_000,
        recoverable_units: 100,
        units_sold: 0,
        carryover: 0,
        amt_carryover: 0,
    };
    add(&mut d, second);
    borrowing(&mut d, source, 200_000);
    d
}

#[test]
fn unit13_royalty_native_depletion_and_routed_interest_keep_actual_owners() {
    let mut d = fixture();
    // First:11,000 -1,000 expenses -2,000 cost depletion -2,000
    // actual routed Form4952 interest =6,000. Second:20,000 -4,000
    // expenses -2,000 (10% mineral depletion) =14,000. Joint wages
    // reach the250,000 NIIT threshold; all20,000 NII is taxed at3.8%.
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 270_000);
        assert_eq!(value(&r, "schedule_e_rental_royalty_total"), 20_000);
        assert_eq!(value(&r, "form_8960_net_investment_income"), 20_000);
        assert_eq!(value(&r, "net_investment_income_tax"), 760);
        let i = usize::from(injured == Owner::Spouse);
        assert_eq!(
            [column(&r, "13b", "b"), column(&r, "13b", "c")],
            [[6000, 14000], [14000, 6000]][i]
        );
        assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], [[228, 532], [532, 228]][i]);
        assert!(
            !r.review_notes
                .iter()
                .any(|n| n.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
        );
        ordinary_unchanged(&d, &r);
    }
    d.income.reverse();
    assert_eq!(value(&supported(&d), "agi"), 270_000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn unit13_royalty_cents_follow_native_annual_interest_rounding_once() {
    let mut d = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut entry = income(royalty(1000, 0));
        entry.owner = crate::DocumentOwner::from(owner);
        let FederalIncome::Royalty { data } = &mut entry.income else { panic!() };
        data.receipts += 49;
        add(&mut d, entry);
    }
    let source = d.income[2].id;
    borrowing(&mut d, source, 149);
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        // The native annual charge is $1, not149c. Source cents before
        // final category rounding are99949 and100049; joint is2,000.
        assert_eq!(value(&r, &alloc::format!("royalty_{source}_line13_interest")), 1);
        assert_eq!(value(&r, "schedule_e_rental_royalty_total"), 2000);
        assert_eq!(value(&r, "form_8960_net_investment_income"), 2000);
        assert_eq!(value(&r, "net_investment_income_tax"), 76);
        let expected = if injured == Owner::Taxpayer { [999, 1001] } else { [1000, 1000] };
        assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], expected);
        assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], [38, 38]);
        ordinary_unchanged(&d, &r);
    }
}

#[test]
fn unit13_royalty_signed_nii_preserves_named_unavailable_tax_boundary() {
    let mut d = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
    add(&mut d, income(royalty(1000, 2000)));
    let mut other = income(royalty(3000, 0));
    other.owner = crate::DocumentOwner::Spouses;
    add(&mut d, other);
    let r = supported(&d);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), 2000);
    assert_eq!(value(&r, "form_8960_net_investment_income"), 2000);
    assert_eq!(value(&r, "net_investment_income_tax"), 76);
    assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [-1000, 3000]);
    assert!(
        r.review_notes.iter().any(|n| n.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
    );
    assert!(!r.lines.iter().any(|l| l.id == "form_8379_injured_refund_estimate"));
    ordinary_unchanged(&d, &r);
}

#[test]
fn unit13_royalty_source_uses_final_aggregate_oil_depletion_limit() {
    let mut d = injured_return((30_000, 20_000), (30_000, 0), 0, allocation());
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut entry = income(royalty(20_000, 0));
        entry.owner = crate::DocumentOwner::from(owner);
        let FederalIncome::Royalty { data } = &mut entry.income else { panic!() };
        data.kind = RoyaltyKind::OilGas;
        data.depletion = RoyaltyDepletion::Percentage {
            adjusted_basis: 0,
            recoverable_units: 0,
            units_sold: 0,
            carryover: 3_000_000,
            amt_carryover: 3_000_000,
        };
        add(&mut d, entry);
    }
    let r = supported(&d);
    // The actual joint65% base is60,000+40,000-32,200=67,800;
    // allowed44,070,22,035 each, instead of each nominal33,000.
    for entry in &d.income[2..] {
        assert_eq!(value(&r, &alloc::format!("royalty_{}_line18_depletion", entry.id)), 22_035);
        assert_eq!(value(&r, &alloc::format!("royalty_{}_depletion_carryover", entry.id)), 10_965);
    }
    assert_eq!(value(&r, "agi"), 55_930);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), -4070);
    assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [-2035, -2035]);
    ordinary_unchanged(&d, &r);
}

#[test]
fn unit13_royalty_actual_client_save_reopen_restart() {
    extern crate std;
    if let Ok(profile) = std::env::var("TAX_UNIT13_ROYALTY_PROFILE") {
        actual_client_save_reopen_restart(&profile);
    } else {
        for profile in ["royalty", "borrowing"] {
            actual_client_save_reopen_restart(profile);
        }
    }
}

fn actual_client_save_reopen_restart(profile: &str) {
    extern crate std;
    use crate::tests::cross::interview::{accepted, from_review, open_topic};
    assert!(matches!(profile, "royalty" | "borrowing"));
    let topic = if profile == "royalty" { TaxSection::Rentals } else { TaxSection::OtherItemized };
    let open = |session: &mut Interview| {
        if profile == "royalty" {
            open_topic(session, topic)
        } else {
            let Reply::Edit(page @ P::SaveOtherItemized { .. }) = from_review(session, topic)
            else {
                panic!()
            };
            page
        }
    };
    let mut session = Interview::new(TEST_USER, fixture());
    let mut page = open(&mut session);
    if let Ok(input) = std::env::var("TAX_UNIT13_ROYALTY_CALC_INPUT") {
        let bytes = std::fs::read(input).unwrap();
        let request = P::from_avro(&bytes).unwrap();
        assert_eq!(request.to_avro(), bytes);
        let Reply::Response(response) = send(&mut session, request) else { panic!() };
        std::fs::write(
            std::env::var("TAX_UNIT13_ROYALTY_CALC_OUTPUT").unwrap(),
            response.to_avro(),
        )
        .unwrap();
        return;
    }
    if let Ok(dir) = std::env::var("TAX_UNIT13_ROYALTY_SEED_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(std::path::Path::new(&dir).join("seed.avro"), page.to_avro()).unwrap();
        return;
    }
    let request = if let Ok(input) = std::env::var("TAX_UNIT13_ROYALTY_PAYLOAD") {
        let bytes = std::fs::read(input).unwrap();
        let request = P::from_avro(&bytes).unwrap();
        assert_eq!(request.to_avro(), bytes);
        request
    } else {
        if profile == "royalty" {
            let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
            documents.documents[0].owner = Owner::Spouse;
            let crate::IncomeRecord::Royalties(data) = &mut documents.documents[0].document else {
                panic!()
            };
            data.receipts = 1_200_000;
        } else {
            let P::SaveOtherItemized { value, .. } = &mut page else { panic!() };
            value.investment_interest.as_mut().unwrap().loans[0].paid = 300_000;
        }
        page
    };
    accepted(&send(&mut session, request));
    let check = |draft: &FederalDraft| {
        let result = supported(draft);
        let (net, niit, income, taxes) = if profile == "royalty" {
            (21_000, 798, [0, 21_000], [0, 798])
        } else {
            (19_000, 722, [5000, 14_000], [190, 532])
        };
        assert_eq!(value(&result, "schedule_e_rental_royalty_total"), net);
        assert_eq!(value(&result, "form_8960_net_investment_income"), net);
        assert_eq!(value(&result, "net_investment_income_tax"), niit);
        assert_eq!([column(&result, "13b", "b"), column(&result, "13b", "c")], income);
        assert_eq!([column(&result, "18", "b"), column(&result, "18", "c")], taxes);
        assert!(
            !result
                .review_notes
                .iter()
                .any(|n| n.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
        );
        ordinary_unchanged(draft, &result);
    };
    check(&session.draft);
    let tax_year = session.draft.tax_year;
    let stored = native_draft_records(&session.draft);
    let draft = draft_from_native_records(tax_year, &stored);
    assert_eq!(draft, session.draft);
    let mut reopened = restart(draft);
    let shown = open(&mut reopened);
    if let Ok(dir) = std::env::var("TAX_UNIT13_ROYALTY_OUTPUT") {
        std::fs::create_dir_all(&dir).unwrap();
        let dir = std::path::Path::new(&dir);
        std::fs::write(dir.join("reopened.avro"), shown.to_avro()).unwrap();
        write_native_draft_records(dir.join("persisted.avro"), &stored);
    }
    let accepted_draft = reopened.draft.clone();
    accepted(&send(&mut reopened, shown));
    assert_eq!(reopened.draft, accepted_draft);
    check(&reopened.draft);
}
