// Independent partner property-sale QBI owner oracles. Whole-return
// acceptance is exercised separately in k1_sale_final.rs.
use super::*;
use crate::tests::passthrough::k1_sale_return::{draft, sources};

fn sale(gain: i64) -> K1Section179Sale {
    k1_reported_sale(
        "Synthetic business property",
        20230301,
        [(20_000 + gain) * 100, 2_000_000, 0, 0],
    )
}
fn property(net: i64, basis_loss: i64) -> K1Section1231 {
    K1Section1231 {
        net: net * 100,
        unrecaptured: 0,
        basis_loss: basis_loss * 100,
        risk_loss: 0,
        passive_loss: 0,
    }
}
fn source(ordinary: i64, rows: Vec<K1Section179Sale>, capacity: i64) -> PartnerK1 {
    let FederalIncome::Partnership { mut data } =
        sources(ordinary, 0, 0, 20_000).into_iter().next().unwrap()
    else {
        unreachable!()
    };
    data.qbi = Some(partner_qbi_statement(ordinary));
    data.section179_sales = rows;
    data.regular.basis.opening = capacity * 100;
    data.regular.risk.start =
        ScorpRiskStart::Detailed { capacity: capacity * 100, profit: 0, loss: 0 };
    data.alternative = Some(data.regular.clone());
    data
}
fn pool(d: &FederalDraft) -> crate::extended_rules::RentalResult {
    let mut r = FederalResult::empty(d.revision);
    let pool = crate::extended_rules::rentals_for(d, 200_000, &mut r, false);
    assert!(pool.inputs_complete, "{:?}", r.issues);
    pool
}
fn calculate(
    d: &FederalDraft,
    id: i64,
    taxable: i64,
) -> (crate::section1231::PartnerPropertyQbi, crate::partnership_qbi::PartnerQbiResult) {
    let pool = pool(d);
    let entry = d.income.iter().find(|v| v.id == id).unwrap();
    let FederalIncome::Partnership { data } = &entry.income else { unreachable!() };
    let sales = crate::k1_sale_qbi::current(entry, &pool, taxable, FederalStatus::Single).unwrap();
    let property = crate::section1231::partner_property_qbi_items(
        d,
        &pool,
        id,
        Some((taxable, FederalStatus::Single)),
    )
    .unwrap();
    let result = data
        .qbi
        .as_ref()
        .unwrap()
        .calculate_combined(
            data,
            pool.partners.iter().find(|v| v.id == id).unwrap().suspended,
            0,
            (taxable, FederalStatus::Single),
            &sales,
            &property,
        )
        .unwrap();
    (property, result)
}
fn draft_of(data: PartnerK1) -> (FederalDraft, i64) {
    draft(FederalIncome::Partnership { data })
}

#[test]
fn k1_sale_qbi_partner_property_f2b_without_original_box_reaches_the_coordinator() {
    let FederalIncome::Partnership { mut data } =
        sources(15_000, 2_000, 2_000, 20_000).into_iter().next().unwrap()
    else {
        unreachable!()
    };
    data.qbi = source(15_000, vec![], 0).qbi;
    let original = data.clone();
    let (d, id) = draft_of(data);
    let (property, business) = calculate(&d, id, 0);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(property.gain_cents),
            crate::kernel::ratio::dollars(property.current_loss_cents.get()),
            crate::kernel::ratio::dollars(property.historical_loss_cents.get())
        ),
        (0, 4_000, 0)
    );
    assert_eq!(business.current.income, 11_000);
    let r = estimate(&d);
    let note =
        r.review_notes.iter().find(|n| n.display.key == "NOTE_PARTNERSHIP_CURRENT_QBI").unwrap();
    assert_eq!(note.display.args[2], MessageArg::usd_dollars(11_000));
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope);
    assert!(r.issues.is_empty(), "{:?}", r.issues);
    assert_eq!(d.income[0].income, FederalIncome::Partnership { data: original.clone() });
    assert!(original.regular.section1231.is_none());
    assert!(original.qbi.as_ref().unwrap().valid(&original));
}

#[test]
fn k1_sale_qbi_partner_property_combines_original_qualification_and_preserves_bounds() {
    let mut data = source(0, vec![sale(-2_000)], 3_000);
    data.regular.section1231 = Some(property(-3_000, 0));
    data.qbi.as_mut().unwrap().section1231_qbi = -100_000;
    let original = data.clone();
    let (d, id) = draft_of(data);
    let (property, business) = calculate(&d, id, 0);
    assert_eq!(crate::kernel::ratio::dollars(property.current_loss_cents.get()), 1_800);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(property.carries[0].0.get()),
            crate::kernel::ratio::dollars(property.carries[0].1.get())
        ),
        (2_000, 1_200)
    );
    assert_eq!(business.current.income, -1_800);
    let mut invalid = original.qbi.clone().unwrap();
    invalid.section1231_qbi = -300_001;
    assert!(!invalid.valid(&original));
    assert_eq!(d.income[0].income, FederalIncome::Partnership { data: original });
}

#[test]
fn k1_sale_qbi_partner_property_keeps_gross_sale_gain_and_loss_through_basis() {
    let (d, id) = draft_of(source(0, vec![sale(10_000), sale(-14_000)], 0));
    let (property, business) = calculate(&d, id, 0);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(property.gain_cents),
            crate::kernel::ratio::dollars(property.current_loss_cents.get())
        ),
        (10_000, 10_000)
    );
    assert_eq!(
        property.carries.map(|(loss, qbi)| (
            crate::kernel::ratio::dollars(loss.get()),
            crate::kernel::ratio::dollars(qbi.get())
        )),
        [(4_000, 4_000), (0, 0), (0, 0)]
    );
    assert_eq!(business.current.income, 0);
    let r = estimate(&d);
    let note = r
        .review_notes
        .iter()
        .find(|n| n.display.key == "NOTE_PARTNER_PROPERTY_QBI_BASIS_CARRY")
        .unwrap();
    assert_eq!(
        note.display.args,
        vec![
            MessageArg::Number(id),
            MessageArg::usd_cents(4_000 * 100),
            MessageArg::usd_cents(4_000 * 100)
        ]
    );
}

#[test]
fn k1_sale_qbi_partner_property_risk_limit_preserves_its_own_qualified_carry() {
    let mut data = source(0, vec![sale(-5_000)], 5_000);
    data.regular.risk.start = ScorpRiskStart::Detailed { capacity: 200_000, profit: 0, loss: 0 };
    let (d, id) = draft_of(data);
    let (property, business) = calculate(&d, id, 0);
    assert_eq!(
        (crate::kernel::ratio::dollars(property.current_loss_cents.get()), business.current.income),
        (2_000, -2_000)
    );
    assert_eq!(
        property.carries.map(|(loss, qbi)| (
            crate::kernel::ratio::dollars(loss.get()),
            crate::kernel::ratio::dollars(qbi.get())
        )),
        [(0, 0), (3_000, 3_000), (0, 0)]
    );
    let r = estimate(&d);
    assert!(r.review_notes.iter().any(|n| n.display.key == "NOTE_PARTNER_PROPERTY_QBI_RISK_CARRY"));
    assert!(
        !r.review_notes.iter().any(|n| n.display.key == "NOTE_PARTNER_PROPERTY_QBI_BASIS_CARRY")
    );
}

#[test]
fn k1_sale_qbi_partner_property_passive_limit_uses_its_character_cut() {
    let mut data = source(0, vec![sale(-2_000), sale(1_000)], 5_000);
    let PartnerParticipation::General { facts } = &mut data.participation else { unreachable!() };
    facts.minutes = 0;
    let (d, id) = draft_of(data);
    let (property, business) = calculate(&d, id, 0);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(property.gain_cents),
            crate::kernel::ratio::dollars(property.current_loss_cents.get()),
            business.current.income
        ),
        (1_000, 1_000, 0)
    );
    assert_eq!(
        property.carries.map(|(loss, qbi)| (
            crate::kernel::ratio::dollars(loss.get()),
            crate::kernel::ratio::dollars(qbi.get())
        )),
        [(0, 0), (0, 0), (1_000, 1_000)]
    );
    let r = estimate(&d);
    let note = r
        .review_notes
        .iter()
        .find(|n| n.display.key == "NOTE_PARTNER_PROPERTY_QBI_PASSIVE_CARRY")
        .unwrap();
    assert_eq!(
        note.display.args,
        vec![
            MessageArg::Number(id),
            MessageArg::usd_cents(1_000 * 100),
            MessageArg::usd_cents(1_000 * 100)
        ]
    );
}

#[test]
fn k1_sale_qbi_partner_property_sstb_qualification_precedes_basis_and_combines_current_sources() {
    let mut data = source(0, vec![sale(-1)], 1);
    data.regular.section1231 = Some(property(-1, 0));
    let statement = data.qbi.as_mut().unwrap();
    statement.section1231_qbi = -100;
    statement.service = true;
    let (d, id) = draft_of(data);
    let (property, business) = calculate(&d, id, 251_750);
    // 200c current loss has 67c original qualification. The 100c basis
    // release admits 34c qualified and holds 33c, before report rounding.
    assert_eq!(property.current_loss_cents.get(), 34);
    assert_eq!((property.carries[0].0.get(), property.carries[0].1.get()), (100, 33));
    assert_eq!(business.current.income, 0);
}

#[test]
fn k1_sale_qbi_partner_property_current_gains_share_one_sstb_rounding_and_wage_amount() {
    let mut data = source(1, vec![sale(1)], 0);
    data.regular.section1231 = Some(property(1, 0));
    let statement = data.qbi.as_mut().unwrap();
    statement.section1231_qbi = 100;
    statement.service = true;
    statement.wages = 200;
    statement.property = 400;
    let (mut d, id) = draft_of(data);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[(2025, 2, 2)], &[]));
    let (_, business) = calculate(&d, id, 239_250);
    assert_eq!(
        (business.current.income, business.current.wages, business.current.property),
        (2, 1, 2)
    );
}

#[test]
fn k1_sale_qbi_partner_property_capital_character_excludes_gross_components() {
    let (d, id) = draft_of(source(0, vec![sale(10_000), sale(-4_000)], 0));
    let (property, business) = calculate(&d, id, 0);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(property.gain_cents),
            crate::kernel::ratio::dollars(property.current_loss_cents.get()),
            crate::kernel::ratio::dollars(property.historical_loss_cents.get())
        ),
        (0, 0, 0)
    );
    assert_eq!(business.current.income, 0);
}

#[test]
fn k1_sale_qbi_partner_property_known_partial_lookback_uses_the_qualified_current_net() {
    for rows in [vec![sale(10_000)], vec![sale(10_000), sale(-4_000)]] {
        let mut data = source(0, rows, 0);
        data.qbi.as_mut().unwrap().service = true;
        let (mut d, id) = draft_of(data);
        d.screening.as_mut().unwrap().section1231 =
            Some(section1231_history(&[(2025, 3_000, 3_000)], &[]));
        let (property, business) = calculate(&d, id, 239_250);
        assert_eq!(
            (
                crate::kernel::ratio::dollars(property.gain_cents),
                crate::kernel::ratio::dollars(property.current_loss_cents.get()),
                crate::kernel::ratio::dollars(property.historical_loss_cents.get())
            ),
            (3_000, 0, 0)
        );
        assert_eq!(business.current.income, 1_500);
    }
}

#[test]
fn k1_sale_qbi_partner_property_mixed_qualification_keeps_named_partial_refusal_and_full_amount() {
    let mut data = source(0, vec![sale(10_000)], 0);
    data.regular.section1231 = Some(property(10_000, 0));
    data.qbi.as_mut().unwrap().section1231_qbi = 500_000;
    let original = data.clone();
    let (mut d, id) = draft_of(data);
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 5_000, 5_000)], &[]));
    let p = pool(&d);
    assert_eq!(
        crate::section1231::partner_property_qbi_items(
            &d,
            &p,
            id,
            Some((0, FederalStatus::Single))
        )
        .err()
        .unwrap()
        .detail
        .0,
        "ISSUE_K1_SECTION1231_QBI_SPLIT"
    );
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 20_000, 20_000)], &[]));
    let (property, business) = calculate(&d, id, 0);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(property.gain_cents),
            crate::kernel::ratio::dollars(property.current_loss_cents.get())
        ),
        (15_000, 0)
    );
    assert_eq!(business.current.income, 15_000);
    let mut invalid = original.qbi.clone().unwrap();
    invalid.section1231_qbi = 1_000_001;
    assert!(!invalid.valid(&original));
}

#[test]
fn k1_sale_qbi_partner_property_retained_listing_qualification_is_not_rephased_with_current_loss() {
    let mut data = source(0, vec![sale(-4_000)], 5_000);
    data.regular.section1231 = Some(property(0, 2_000));
    data.qbi.as_mut().unwrap().service = true;
    let original = data.clone();
    let (d, id) = draft_of(data);
    let (property, business) = calculate(&d, id, 300_000);
    // Approved partner convention: prior $2,000 is qualified by the current
    // listing. Linear basis release is 5,000 * 2,000 / 6,000 = 1,667.
    // Current-source eligibility is zero, and prior qualification is retained.
    assert_eq!(
        (
            crate::kernel::ratio::dollars(property.current_loss_cents.get()),
            crate::kernel::ratio::dollars(property.historical_loss_cents.get())
        ),
        (0, 1_667)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(property.carries[0].0.get()),
            crate::kernel::ratio::dollars(property.carries[0].1.get())
        ),
        (1_000, 333)
    );
    assert_eq!((business.current.income, business.historical), (0, 1_667));
    assert_eq!(d.income[0].income, FederalIncome::Partnership { data: original });
}

#[test]
fn k1_sale_qbi_partner_property_missing_statement_keeps_current_sale_out_with_loss_notice() {
    let mut data = source(0, vec![sale(-4_000)], 5_000);
    data.qbi = None;
    let (d, id) = draft_of(data);
    let p = pool(&d);
    let property = crate::section1231::partner_property_qbi_items(
        &d,
        &p,
        id,
        Some((0, FederalStatus::Single)),
    )
    .unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(property.gain_cents),
            crate::kernel::ratio::dollars(property.current_loss_cents.get()),
            crate::kernel::ratio::dollars(property.historical_loss_cents.get())
        ),
        (0, 0, 0)
    );
    let r = estimate(&d);
    assert!(r.review_notes.iter().any(|n| n.display.key == "WARN_K1_QBI_LOSS_NO_STATEMENT"));
}

#[test]
fn k1_sale_qbi_partner_property_former_passive_current_loss_bypasses_old_passive_suspension() {
    let mut data = source(0, vec![sale(-4_000)], 5_000);
    data.regular.section1231 = Some(K1Section1231 { passive_loss: 600_000, ..property(0, 0) });
    data.qbi.as_mut().unwrap().service = true;
    let (d, id) = draft_of(data);
    let (property, business) = calculate(&d, id, 239_250);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(property.current_loss_cents.get()),
            crate::kernel::ratio::dollars(property.historical_loss_cents.get())
        ),
        (2_000, 0)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(property.carries[2].0.get()),
            crate::kernel::ratio::dollars(property.carries[2].1.get())
        ),
        (6_000, 6_000)
    );
    assert_eq!((business.current.income, business.historical), (-2_000, 0));
}

#[test]
fn k1_sale_qbi_partner_property_independent_amt_sales_do_not_change_regular_qbi() {
    let mut data = source(0, vec![sale(-4_000)], 5_000);
    data.section179_sales[0].alternative =
        crate::K1Section179SaleAmtSource::from_known_regular(&sale(10_000).regular);
    let original = data.clone();
    let (d, id) = draft_of(data);
    let (property, business) = calculate(&d, id, 0);
    assert_eq!(
        (crate::kernel::ratio::dollars(property.current_loss_cents.get()), business.current.income),
        (4_000, -4_000)
    );
    assert_eq!(d.income[0].income, FederalIncome::Partnership { data: original });
}

#[test]
fn k1_sale_qbi_partner_property_rounds_source_boxes_once_without_rewriting_minor_units() {
    let mut data = source(0, vec![sale(-2)], 5);
    data.regular.section1231 = Some(K1Section1231 { net: -149, basis_loss: 149, ..property(0, 0) });
    data.qbi.as_mut().unwrap().section1231_qbi = -49;
    let original = data.clone();
    let (d, id) = draft_of(data);
    let (property, business) = calculate(&d, id, 0);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(property.current_loss_cents.get()),
            crate::kernel::ratio::dollars(property.historical_loss_cents.get())
        ),
        (2, 1)
    );
    assert_eq!((business.current.income, business.historical), (-2, 1));
    assert_eq!(d.income[0].income, FederalIncome::Partnership { data: original });
}

#[test]
fn k1_sale_qbi_partner_property_allowed_section179_is_deducted_once_before_combined_eligibility() {
    let mut data = source(15_000, vec![sale(10_000)], 0);
    data.regular.section1231 = Some(property(10_000, 0));
    data.section179 = 200_000;
    let statement = data.qbi.as_mut().unwrap();
    statement.section1231_qbi = 1_000_000;
    statement.service = true;
    let (mut d, id) = draft_of(data);
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 20000, 20000)], &[]));
    let (property, _) = calculate(&d, id, 239_250);
    let FederalIncome::Partnership { data } = &d.income[0].income else { unreachable!() };
    let value = data
        .qbi
        .as_ref()
        .unwrap()
        .calculate_combined(
            data,
            crate::kernel::carry::Amount::new(0).unwrap(),
            2_000,
            (239_250, FederalStatus::Single),
            &Default::default(),
            &property,
        )
        .unwrap();
    assert_eq!(value.current.income, 16_500); // Half of 15k + 10k + 10k - 2k.
}

#[test]
fn k1_sale_qbi_partner_property_shared_lookback_keeps_the_rental_recapture_in_its_own_business() {
    let mut data = source(0, vec![sale(10_000)], 0);
    data.regular.rentals.real_estate_property =
        K1RentalProperty { net: 500_000, unrecaptured: 500_000, ..Default::default() };
    data.rental_facts.real_estate_qbi = Some(K1RentalQbi { wages: 0, property: 0 });
    let (mut d, id) = draft_of(data);
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 5_000, 5_000)], &[]));
    let p = pool(&d);
    assert_eq!(
        crate::section1231::rental_property_qbi(&d, &p).unwrap().current,
        vec![(0, 5_000, Some(500_000))]
    );
    let (property, business) = calculate(&d, id, 0);
    assert_eq!(
        (crate::kernel::ratio::dollars(property.gain_cents), business.current.income),
        (0, 0)
    );
    let r = estimate(&d);
    let note =
        r.review_notes.iter().find(|n| n.display.key == "NOTE_PARTNERSHIP_CURRENT_QBI").unwrap();
    assert_eq!(note.display.args[2], MessageArg::usd_dollars(0));
}

#[test]
fn k1_sale_qbi_partner_property_fuel_recovery_joins_one_current_business_eligibility_pass() {
    let mut data = source(1, vec![sale(1)], 0);
    data.regular.section1231 = Some(property(1, 0));
    let statement = data.qbi.as_mut().unwrap();
    statement.section1231_qbi = 100;
    statement.service = true;
    data.prior_fuel_credit = Some(PartnerPriorFuelCredit {
        claimed_credit: 100,
        filed_in_return_year: true,
        cash_method: true,
        full_prior_tax_benefit: true,
        partial_prior_year_benefit: None,
        prior_niit_recovery_excluded: true,
        not_reported_elsewhere: true,
    });
    let (mut d, id) = draft_of(data);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[(2025, 2, 2)], &[]));
    let (_, business) = calculate(&d, id, 239_250);
    assert_eq!(business.current.income, 2); // Half of four $1 current gains, rounded once.
}

#[test]
fn k1_sale_qbi_partner_property_linear_current_and_prior_shares_survive_all_three_limits() {
    for (service, taxable, current, historical) in
        [(false, 0, 714, 3_929), (true, 239_250, 357, 3_929)]
    {
        let mut data = source(0, vec![sale(-2_000)], 6_000);
        data.regular.section1231 = Some(K1Section1231 {
            risk_loss: 400_000,
            passive_loss: 200_000,
            ..property(-4_000, 6_000)
        });
        data.regular.risk.start =
            ScorpRiskStart::Detailed { capacity: 500_000, profit: 0, loss: 0 };
        let statement = data.qbi.as_mut().unwrap();
        statement.section1231_qbi = -200_000;
        statement.service = service;
        let PartnerParticipation::General { facts } = &mut data.participation else {
            unreachable!()
        };
        facts.minutes = 0;
        let (mut d, id) = draft_of(data);
        add(&mut d, income(FederalIncome::Scorp { data: complete_ordinary_scorp(5_000, false) }));
        let (property, business) = calculate(&d, id, taxable);
        // Current $6k loss has $4k potential QBI; prior basis/risk/passive
        // $6k/$4k/$2k qualify under the approved listing convention.
        // Basis/risk/PAL allow $6k/$5k/$5k. Below threshold, their allowed
        // QBI totals are $5k/$4.5k/$4,643; at 50%, $4k/$4k/$4,286.
        assert_eq!(
            (
                crate::kernel::ratio::dollars(property.current_loss_cents.get()),
                crate::kernel::ratio::dollars(property.historical_loss_cents.get())
            ),
            (current, historical)
        );
        assert_eq!((business.current.income, business.historical), (-current, historical));
        assert_eq!(
            property
                .carries
                .iter()
                .map(|v| crate::kernel::ratio::dollars(v.0.get()))
                .collect::<Vec<_>>(),
            vec![6_000, 5_000, 2_000]
        );
    }
}

#[test]
fn k1_sale_qbi_partner_property_full_recapture_keeps_qualified_carry_out_of_current_sstb() {
    let mut data = source(0, vec![sale(10_000), sale(-4_000)], 0);
    data.regular.section1231 = Some(property(0, 2_000));
    data.qbi.as_mut().unwrap().service = true;
    let (mut d, id) = draft_of(data);
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 4_000, 4_000)], &[]));
    let (property, business) = calculate(&d, id, 239_250);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(property.gain_cents),
            crate::kernel::ratio::dollars(property.current_loss_cents.get()),
            crate::kernel::ratio::dollars(property.historical_loss_cents.get())
        ),
        (10_000, 2_000, 2_000)
    );
    assert_eq!((business.current.income, business.historical), (3_000, 2_000));
    assert_eq!(business.current.income - business.historical, 1_000);
}

#[test]
fn current_business_qbi_combines_exact_qualified_loss_and_property_gain_once() {
    let mut data = source(0, vec![], 5);
    data.regular.ordinary = -49;
    data.regular.section1231 = Some(K1Section1231 { net: 50, ..property(0, 0) });
    let statement = data.qbi.as_mut().unwrap();
    statement.income = -49;
    statement.section1231_qbi = 50;
    data.alternative = Some(data.regular.clone());
    let (mut draft, id) = draft_of(data);
    let mut other = source(0, vec![], 5);
    other.regular.section1231 = Some(K1Section1231 { net: -100, ..property(0, 0) });
    other.qbi = None;
    other.alternative = Some(other.regular.clone());
    add(&mut draft, income(FederalIncome::Partnership { data: other }));
    let (property, business) = calculate(&draft, id, 0);
    assert_eq!(property.gain_cents, 50);
    assert_eq!(business.carries[2].qbi.get(), 49);
    // One cent of current qualified net reaches one reported business amount.
    assert_eq!(business.current.income, 0);
}
