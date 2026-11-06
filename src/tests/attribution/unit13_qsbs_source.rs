// Actual native QSBS issuer/cohort sources, with cents retained until the
// downstream owner projection. Expected amounts are hand-computed below.
use super::*;
use crate::tests::cross::interview::{accepted, document};
use crate::tests::support::builders::qsb_source_sale as sale;
use crate::tests::support::client::income_pages::{open_document_center, save_documents};

const ISSUER: &str = "Synthetic Source, Inc.";

fn add_sale(d: &mut FederalDraft, owner: Owner, sale: QsbIssuerSale) {
    let mut entry = income(FederalIncome::QualifiedSmallBusinessSale { data: sale });
    entry.owner = crate::DocumentOwner::from(owner);
    add(d, entry);
}
fn run(d: &FederalDraft) -> (crate::qsbs_return::QsbResult, FederalResult) {
    let mut r = FederalResult::empty(d.revision);
    let source = crate::qsbs_return::calculate(d, &mut r);
    (source, r)
}
fn source_values(source: &crate::qsbs_return::QsbResult) -> Vec<(i64, Owner, i64, Option<i64>)> {
    source.sources.iter().map(|v| (v.id, v.owner, v.taxable_gain, v.rate_28_gain)).collect()
}
fn taxable_rows(source: &crate::qsbs_return::QsbResult) -> Vec<i64> {
    source.rows.iter().map(|v| v.proceeds - v.basis + v.adjustment).collect()
}

#[test]
fn unit13_qsbs_sources_mixed_dates_keep_fully_excluded_owner_out_of_28_pool() {
    let mut d = complete(FederalStatus::Joint);
    add_sale(
        &mut d,
        Owner::Taxpayer,
        sale(ISSUER, 20080602, 1_000_000, BasisReporting::ReportedToIrs),
    );
    add_sale(&mut d, Owner::Spouse, sale(ISSUER, 20150302, 1_000_000, BasisReporting::NotReported));
    add_sale(&mut d, Owner::Spouse, sale(ISSUER, 20100927, 800_000, BasisReporting::NoForm1099B));
    let (q, r) = run(&d);
    assert!(r.issues.is_empty());
    // Gain28000: 50%of10000 +100%of10000 +75%of8000 =21000 excluded.
    assert_eq!(
        (q.excluded_gain, q.taxable_gain, q.rate_28_gain, q.amt_preference),
        (2_100_000, 700_000, 700_000, 77_000)
    );
    assert_eq!(
        source_values(&q),
        vec![
            (2, Owner::Taxpayer, 500_000, Some(500_000)),
            (3, Owner::Spouse, 0, Some(0)),
            (4, Owner::Spouse, 200_000, Some(200_000))
        ]
    );
    assert_eq!(taxable_rows(&q), vec![500_000, 0, 200_000]);
    assert_eq!(q.rows.iter().map(|v| v.box_letter).collect::<Vec<_>>(), vec!['D', 'E', 'F']);
    let joint = supported(&d);
    assert_eq!(value(&joint, "agi"), 107_000);
    assert_eq!(value(&joint, "rate_28_gain"), 7000);
}

#[test]
fn unit13_qsbs_sources_zone_increase_stays_with_its_actual_lot() {
    let mut d = complete(FederalStatus::Joint);
    let mut zone = sale(ISSUER, 20080602, 1_000_000, BasisReporting::ReportedToIrs);
    zone.zone_60_exclusion = Some(crate::QsbsZoneCertification::Yes);
    zone.lots[0].zone_gain_through_2018 =
        Some(crate::QsbsZoneValuation::Established { gain: 1_000_000 });
    add_sale(&mut d, Owner::Taxpayer, zone);
    add_sale(&mut d, Owner::Spouse, sale(ISSUER, 20080602, 1_000_000, BasisReporting::NoForm1099B));
    let (q, r) = run(&d);
    assert!(r.issues.is_empty());
    // Native issuer exclusion is10000 at50% +1000 zone increase. Source
    // inclusions are4000/5000, rather than spreading9000 as4500/4500.
    assert_eq!((q.excluded_gain, q.rate_28_gain, q.amt_preference), (1_100_000, 900_000, 77_000));
    assert_eq!(taxable_rows(&q), vec![400_000, 500_000]);
    assert_eq!(
        source_values(&q),
        vec![
            (2, Owner::Taxpayer, 400_000, Some(400_000)),
            (3, Owner::Spouse, 500_000, Some(500_000))
        ]
    );
}

#[test]
fn unit13_qsbs_sources_uniform_binding_cap_preserves_native_proportions() {
    for (issued, excluded, rate, source_rate, amt) in [
        (20080602, 50_000_000, 50_000_000, [33_333_333, 16_666_667], 3_500_000),
        (20100927, 75_000_000, 25_000_000, [16_666_667, 8_333_333], 5_250_000),
    ] {
        let mut d = complete(FederalStatus::Joint);
        for (owner, gain) in [(Owner::Taxpayer, 800_000_000), (Owner::Spouse, 400_000_000)] {
            let mut v = sale(ISSUER, issued, gain, BasisReporting::NoForm1099B);
            v.prior_eligible_gain = Some(900_000_000); // Remaining issuer dollar cap1000000.
            add_sale(&mut d, owner, v);
        }
        let (q, r) = run(&d);
        assert!(r.issues.is_empty());
        assert_eq!((q.excluded_gain, q.rate_28_gain, q.amt_preference), (excluded, rate, amt));
        assert_eq!(q.taxable_gain, 1_200_000_000 - excluded);
        assert_eq!(
            q.sources.iter().map(|v| v.rate_28_gain.unwrap()).collect::<Vec<_>>(),
            source_rate
        );
        assert_eq!(
            taxable_rows(&q),
            if issued == 20080602 {
                vec![766_666_667, 383_333_333]
            } else {
                vec![750_000_000, 375_000_000]
            }
        );
        assert_eq!(q.sources.iter().map(|v| v.taxable_gain).sum::<i64>(), q.taxable_gain);
    }
}

#[test]
fn unit13_qsbs_sources_existing_refused_caps_do_not_publish_guessed_28_owners() {
    for partial_zone in [false, true] {
        let mut d = complete(FederalStatus::Joint);
        for (index, (owner, gain)) in
            [(Owner::Taxpayer, 800_000_000), (Owner::Spouse, 400_000_000)].into_iter().enumerate()
        {
            let mut v = sale(
                ISSUER,
                if index == 0 || partial_zone { 20080602 } else { 20150302 },
                gain,
                BasisReporting::NoForm1099B,
            );
            v.prior_eligible_gain = Some(900_000_000);
            if partial_zone && index == 0 {
                v.zone_60_exclusion = Some(crate::QsbsZoneCertification::Yes);
                v.lots[0].zone_gain_through_2018 =
                    Some(crate::QsbsZoneValuation::Established { gain });
            }
            add_sale(&mut d, owner, v);
        }
        let (q, r) = run(&d);
        assert!(!r.issues.is_empty());
        assert_eq!(q.rate_28_gain, 50_000_000); // Existing provisional joint arithmetic unchanged.
        assert!(q.sources.iter().all(|v| v.rate_28_gain.is_none()));
        assert_eq!(taxable_rows(&q), vec![766_666_667, 383_333_333]);
    }
}

#[test]
fn unit13_qsbs_sources_keep_cents_and_source_order_without_early_owner_rounding() {
    for first in [Owner::Taxpayer, Owner::Spouse] {
        let mut d = complete(FederalStatus::Joint);
        for owner in [first, if first == Owner::Taxpayer { Owner::Spouse } else { Owner::Taxpayer }]
        {
            add_sale(&mut d, owner, sale(ISSUER, 20080602, 1, BasisReporting::NoForm1099B));
        }
        let (q, r) = run(&d);
        assert!(r.issues.is_empty());
        assert_eq!((q.excluded_gain, q.taxable_gain, q.rate_28_gain), (1, 1, 1));
        assert_eq!(
            source_values(&q),
            vec![
                (2, first, 1, Some(1)),
                (
                    3,
                    if first == Owner::Taxpayer { Owner::Spouse } else { Owner::Taxpayer },
                    0,
                    Some(0)
                )
            ]
        );
    }
}

#[test]
fn unit13_qsbs_sources_subdollar_owner_amounts_and_zone_residual_are_retained() {
    let mut d = complete(FederalStatus::Joint);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        add_sale(&mut d, owner, sale(ISSUER, 20080602, 98, BasisReporting::NoForm1099B));
    }
    let (q, r) = run(&d);
    assert!(r.issues.is_empty());
    assert_eq!(q.rate_28_gain, 98);
    assert_eq!(
        source_values(&q),
        vec![(2, Owner::Taxpayer, 49, Some(49)), (3, Owner::Spouse, 49, Some(49))]
    );
    // Native issuer fractions: 50%of5c rounds3c; extra10%of5c rounds1c.
    // Only1c remains taxable, with no negative or larger-than-gain source.
    let mut d = complete(FederalStatus::Joint);
    for index in 0..5 {
        let mut v = sale(ISSUER, 20080602, 1, BasisReporting::NoForm1099B);
        v.zone_60_exclusion = Some(crate::QsbsZoneCertification::Yes);
        v.lots[0].zone_gain_through_2018 = Some(crate::QsbsZoneValuation::Established { gain: 1 });
        add_sale(&mut d, if index % 2 == 0 { Owner::Spouse } else { Owner::Taxpayer }, v);
    }
    let (q, r) = run(&d);
    assert!(r.issues.is_empty());
    assert_eq!((q.excluded_gain, q.rate_28_gain), (4, 1));
    assert_eq!(taxable_rows(&q), vec![0, 0, 0, 1, 0]);
    assert_eq!(q.sources.iter().map(|v| v.rate_28_gain.unwrap()).sum::<i64>(), 1);
    assert!(q.sources.iter().all(|v| (0..=1).contains(&v.taxable_gain)));
}

#[test]
fn unit13_qsbs_sources_qualification_assumption_and_history_mismatch_stay_distinct() {
    let mut d = complete(FederalStatus::Joint);
    let mut v = sale(ISSUER, 20080602, 1_000_000, BasisReporting::NoForm1099B);
    v.issuer_qualified = Some(false);
    add_sale(&mut d, Owner::Taxpayer, v);
    let (q, r) = run(&d);
    assert!(r.issues.is_empty());
    assert_eq!(source_values(&q), vec![(2, Owner::Taxpayer, 1_000_000, Some(0))]);

    let mut d = complete(FederalStatus::Joint);
    add_sale(
        &mut d,
        Owner::Taxpayer,
        sale(ISSUER, 20080602, 1_000_000, BasisReporting::NoForm1099B),
    );
    let mut other = sale(ISSUER, 20080602, 1_000_000, BasisReporting::NoForm1099B);
    other.prior_eligible_gain = Some(100);
    add_sale(&mut d, Owner::Spouse, other);
    let (q, r) = run(&d);
    assert!(!r.issues.is_empty());
    assert!(q.sources.iter().all(|v| v.rate_28_gain.is_none()));
}

#[test]
fn unit13_qsbs_sources_public_save_reopen_restart_and_date_correction() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let mut rows = vec![];
    for (owner, issued, gain) in [
        (Owner::Taxpayer, 20080602, 1_000_000),
        (Owner::Spouse, 20150302, 1_000_000),
        (Owner::Spouse, 20100927, 800_000),
    ] {
        let mut row = document(
            0,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::QsbsSale(
                    crate::tests::support::builders::qsbs_document(&sale(
                        ISSUER,
                        issued,
                        gain,
                        BasisReporting::NoForm1099B,
                    )),
                ),
            },
        );
        row.owner = crate::DocumentOwner::from(owner);
        rows.push(row);
    }
    let request = save_documents(&mut session, rows);
    let request = fixture_owner_inputs(&session.draft, request);
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    ));
    let saved = session.draft.clone();
    let before = source_values(&run(&saved).0);
    assert_eq!(before.iter().map(|v| v.3.unwrap()).collect::<Vec<_>>(), vec![500_000, 0, 200_000]);
    session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    let mut page = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
    let rows = &mut documents.common.investment_sales.documents;
    let source = rows.iter_mut().find(|v| v.owner == crate::DocumentOwner::Spouses && matches!(&v.document, crate::InvestmentSaleDocument::QsbsSale(data) if data.lots[0].issued == 20150302)).unwrap();
    let crate::InvestmentSaleDocument::QsbsSale(v) = &mut source.document else { panic!() };
    v.lots[0].issued = 20100927;
    crate::tests::support::builders::remember_qsbs_answers(
        &sale(ISSUER, 20100927, 1_000_000, BasisReporting::NoForm1099B),
        v.clone(),
    );
    let page = fixture_owner_inputs(&session.draft, page);
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        page,
    ));
    let corrected = session.draft.clone();
    let q = run(&corrected).0;
    assert_eq!(
        (q.taxable_gain, q.excluded_gain, q.rate_28_gain, q.amt_preference),
        (950_000, 1_850_000, 950_000, 129_500)
    );
    assert_eq!(
        q.sources.iter().map(|v| v.rate_28_gain.unwrap()).collect::<Vec<_>>(),
        vec![500_000, 250_000, 200_000]
    );
    assert_eq!(value(&supported(&corrected), "agi"), 109_500);
    session = restart(corrected.clone());
    let page = open_document_center(&mut session);
    let page = fixture_owner_inputs(&session.draft, page);
    accepted(&send(&mut session, page));
    assert_eq!(session.draft.income, corrected.income);
    assert_eq!(source_values(&run(&session.draft).0), source_values(&q));
}
