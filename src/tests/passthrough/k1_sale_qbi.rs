// Current ordinary sale QBI through actual source limits and business owners.
// Whole-return acceptance is exercised separately in k1_sale_final.rs.
use super::*;
use crate::k1_sale_qbi::CurrentSales;
use crate::tests::passthrough::k1_sale_return::{draft, passive_short, sources};

fn sale_rows(source: &mut FederalIncome, rows: Vec<K1Section179Sale>, basis: i64, risk: i64) {
    let capacity = ScorpRiskStart::Detailed { capacity: risk * 100, profit: 0, loss: 0 };
    match source {
        FederalIncome::Partnership { data } => {
            data.section179_sales = rows;
            data.regular.basis.opening = basis * 100;
            data.regular.risk.start = capacity;
            data.alternative = Some(data.regular.clone());
        }
        FederalIncome::Scorp { data } => {
            data.section179_sales = rows;
            data.opening = basis * 100;
            data.risk.start = capacity;
            data.amt = Some(amt_records(data));
            for sale in &mut data.section179_sales {
                sale.alternative =
                    crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
            }
        }
        _ => unreachable!(),
    }
}

fn sale(price: i64) -> K1Section179Sale {
    k1_reported_sale(
        "Synthetic equipment",
        20260301,
        [price * 100, 50_000 * 100, 8_000 * 100, 20_000 * 100],
    )
}

fn calculation(
    draft: &FederalDraft,
    id: i64,
    taxable: i64,
) -> (CurrentSales, crate::extended_rules::RentalResult) {
    let mut result = FederalResult::empty(draft.revision);
    let pool = crate::extended_rules::rentals_for(draft, 200_000, &mut result, false);
    assert!(pool.inputs_complete, "{:?}", result.issues);
    let entry = draft.income.iter().find(|entry| entry.id == id).unwrap();
    (crate::k1_sale_qbi::current(entry, &pool, taxable, FederalStatus::Single).unwrap(), pool)
}

fn business(
    entry: &FederalIncomeEntry,
    sales: &CurrentSales,
    taxable: i64,
    deduction: i64,
) -> crate::business_rules::QbiActivity {
    match &entry.income {
        FederalIncome::Partnership { data } => {
            data.qbi
                .as_ref()
                .unwrap()
                .calculate_with_sales(
                    data,
                    crate::kernel::carry::Amount::new(0).unwrap(),
                    deduction,
                    taxable,
                    FederalStatus::Single,
                    sales,
                )
                .unwrap()
                .current
        }
        FederalIncome::Scorp { data } => data
            .qbi_statement
            .as_ref()
            .unwrap()
            .combined_with_sales(
                data,
                [
                    crate::kernel::carry::Amount::new(0).unwrap(),
                    crate::kernel::carry::Amount::new(0).unwrap(),
                ],
                crate::section1231::PropertyCharacter::Capital,
                (Some(taxable), FederalStatus::Single),
                deduction,
                sales,
                0,
            )
            .unwrap()
            .remove(0),
        _ => unreachable!(),
    }
}

#[test]
fn k1_sale_qbi_f2_ordinary_recapture_joins_one_existing_business() {
    for mut source in sources(15_000, 2_000, 2_000, 30_000) {
        k1_qbi_statement(&mut source, 15_000, false);
        let (draft, id) = draft(source);
        let unchanged = draft.clone();
        let (sales, _) = calculation(&draft, id, 4_900);
        assert_eq!(
            (
                crate::kernel::ratio::dollars(sales.gain_cents),
                crate::kernel::ratio::dollars(sales.loss_cents.get())
            ),
            (6_000, 0)
        );
        let entry = draft.income.iter().find(|e| e.id == id).unwrap();
        assert_eq!(business(entry, &sales, 4_900, 0).income, 21_000);
        let result = estimate(&draft);
        assert_eq!(result.state, ResultState::EstimatedForSupportedScope);
        let key = match entry.income {
            FederalIncome::Partnership { .. } => "NOTE_PARTNERSHIP_CURRENT_QBI",
            _ => "NOTE_SCORP_QBI_AFTER_HISTORICAL",
        };
        let note = result.review_notes.iter().find(|note| note.display.key == key).unwrap();
        assert!(note.display.args.contains(&MessageArg::usd_dollars(21_000)), "{:?}", note.display);
        assert_eq!(draft, unchanged);
    }
}

#[test]
fn k1_sale_qbi_basis_and_risk_carries_retain_only_their_own_qualified_loss() {
    for (basis, risk, held_stage) in [(0, 100_000, 0), (4_000, 0, 1)] {
        for mut source in sources(0, 0, 0, 8_000) {
            k1_qbi_statement(&mut source, 0, false);
            sale_rows(&mut source, vec![sale(32_000), sale(8_000)], basis, risk);
            let (draft, id) = draft(source);
            let (sales, _) = calculation(&draft, id, 0);
            // $10,000 gain and $14,000 loss: either basis or amount at risk
            // permits $10,000, leaving $4,000 at just that one limit.
            assert_eq!(
                (
                    crate::kernel::ratio::dollars(sales.gain_cents),
                    crate::kernel::ratio::dollars(sales.loss_cents.get())
                ),
                (10_000, 10_000)
            );
            assert_eq!(sales.carries.len(), 1);
            assert_eq!(sales.carries[0].0, 2);
            for (stage, carry) in sales.carries[0].1.iter().enumerate() {
                let held = if stage == held_stage { 4_000 } else { 0 };
                assert_eq!(
                    crate::kernel::ratio::dollars(
                        carry.remaining.iter().map(|v| v.remaining.get()).sum::<i64>()
                    ),
                    held
                );
                assert_eq!(
                    crate::kernel::ratio::dollars(
                        carry.remaining.iter().map(|v| v.remaining_qbi.get()).sum::<i64>()
                    ),
                    held
                );
            }
            let mut result = FederalResult::empty(0);
            sales.report(id, &mut result);
            assert_eq!(result.review_notes.len(), 1);
            assert_eq!(result.review_notes[0].display.args[1], MessageArg::Number(2));
        }
    }
}

#[test]
fn k1_sale_qbi_passive_source_allocations_keep_suspended_qualification_separate() {
    for (mut loss, mut gain) in sources(0, 0, 0, 20_000).into_iter().zip(sources(0, 0, 0, 23_000)) {
        k1_qbi_statement(&mut loss, 0, false);
        k1_qbi_statement(&mut gain, 0, false);
        let (mut draft, loss_id) = draft(passive_short(loss));
        let gain_id = draft.next_id;
        add(&mut draft, income(passive_short(gain)));
        let (loss, _) = calculation(&draft, loss_id, 0);
        let (gain, _) = calculation(&draft, gain_id, 0);
        assert_eq!(
            (
                crate::kernel::ratio::dollars(loss.gain_cents),
                crate::kernel::ratio::dollars(loss.loss_cents.get())
            ),
            (0, 1_000)
        );
        assert_eq!(
            (
                crate::kernel::ratio::dollars(gain.gain_cents),
                crate::kernel::ratio::dollars(gain.loss_cents.get())
            ),
            (1_000, 0)
        );
        let carry = &loss.carries[0].1[2].remaining[0];
        assert_eq!(
            (
                crate::kernel::ratio::dollars(carry.remaining.get()),
                crate::kernel::ratio::dollars(carry.remaining_qbi.get())
            ),
            (1_000, 1_000)
        );
        assert_eq!(
            crate::kernel::ratio::dollars(loss.carries[0].1[0].remaining[0].remaining.get()),
            0
        );
        assert_eq!(
            crate::kernel::ratio::dollars(loss.carries[0].1[1].remaining[0].remaining.get()),
            0
        );
        let result = estimate(&draft);
        assert!(
            result.review_notes.iter().any(|n| n.display.key == "NOTE_K1_SALE_QBI_PASSIVE_CARRY")
        );
    }
}

#[test]
fn k1_sale_qbi_current_sstb_fraction_is_fixed_before_a_loss_limit() {
    for mut source in sources(0, 0, 0, 21_998) {
        k1_qbi_statement(&mut source, 0, true);
        sale_rows(&mut source, vec![sale(21_998)], 1, 1);
        let (draft, id) = draft(source);
        // 2026 single: $201,750 threshold + $50,000 in the $75,000 range.
        // $2 gross loss is 200c; its original one-third qualified share is
        // 67c. Basis allows 100c: 100 * 67 / 200 rounds to 34c,
        // leaving 33c qualified in the held half. Report QBI only afterward.
        let (sales, _) = calculation(&draft, id, 251_750);
        assert_eq!((sales.gain_cents, sales.loss_cents.get()), (0, 34));
        let carry = &sales.carries[0].1[0].remaining[0];
        assert_eq!((carry.original.get(), carry.original_qbi.get()), (200, 67));
        assert_eq!((carry.remaining.get(), carry.remaining_qbi.get()), (100, 33));
        let entry = draft.income.iter().find(|e| e.id == id).unwrap();
        assert_eq!(business(entry, &sales, 251_750, 0).income, 0);
    }
}

#[test]
fn k1_sale_qbi_gains_share_one_business_sstb_rounding_decision() {
    for mut source in sources(1, 0, 0, 22_001) {
        k1_qbi_statement(&mut source, 1, true);
        sale_rows(&mut source, vec![sale(22_001)], 0, 0);
        let (draft, id) = draft(source);
        let (sales, _) = calculation(&draft, id, 239_250);
        assert_eq!(crate::kernel::ratio::dollars(sales.gain_cents), 1);
        let entry = draft.income.iter().find(|e| e.id == id).unwrap();
        // Half the combined $2 is $1, not two independently rounded $1s.
        assert_eq!(business(entry, &sales, 239_250, 0).income, 1);
    }
}

#[test]
fn k1_sale_qbi_current_section179_deduction_is_taken_once_with_sale_gains() {
    for mut source in sources(15_000, 2_000, 2_000, 30_000) {
        k1_qbi_statement(&mut source, 15_000, false);
        match &mut source {
            FederalIncome::Partnership { data } => data.section179 = 2_000 * 100,
            FederalIncome::Scorp { data } => data.section179 = 2_000 * 100,
            _ => unreachable!(),
        }
        let (draft, id) = draft(source);
        assert_eq!(crate::section179::pass_through_allowed(&draft, id), 2_000 * 100);
        let (sales, _) = calculation(&draft, id, 0);
        let entry = draft.income.iter().find(|e| e.id == id).unwrap();
        assert_eq!(business(entry, &sales, 0, 2_000).income, 19_000);
    }
}

#[test]
fn k1_sale_qbi_absent_statement_omits_current_sale_and_warns_about_loss_direction() {
    for mut source in sources(0, 0, 0, 20_000) {
        sale_rows(&mut source, vec![sale(20_000)], 100_000, 100_000);
        let (draft, id) = draft(source);
        let (sales, _) = calculation(&draft, id, 0);
        assert_eq!(
            (
                crate::kernel::ratio::dollars(sales.gain_cents),
                crate::kernel::ratio::dollars(sales.loss_cents.get())
            ),
            (0, 0)
        );
        let result = estimate(&draft);
        assert!(
            result.review_notes.iter().any(|n| n.display.key == "WARN_K1_QBI_LOSS_NO_STATEMENT"),
            "{:?}",
            result.review_notes
        );
    }
}

#[test]
fn k1_sale_qbi_code_r_keeps_its_own_share_of_the_passive_cut() {
    let mut source = sources(0, 0, 0, 8_000).into_iter().next().unwrap();
    k1_qbi_statement(&mut source, 0, false);
    sale_rows(&mut source, vec![sale(32_000), sale(8_000)], 100_000, 100_000);
    let FederalIncome::Partnership { data } = &mut source else { unreachable!() };
    data.income = vec![PartnerIncome {
        code: PartnerIncomeCode::SpeciallyAllocatedOrdinary,
        amount: -6_000 * 100,
    }];
    let PartnerParticipation::General { facts } = &mut data.participation else { unreachable!() };
    facts.minutes = 0;
    let (draft, id) = draft(source);
    let (sales, pool) = calculation(&draft, id, 0);
    assert_eq!(crate::kernel::ratio::dollars(pool.k1_cuts.ordinary.get()), 10_000);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(sales.gain_cents),
            crate::kernel::ratio::dollars(sales.loss_cents.get())
        ),
        (10_000, 7_000)
    );
    assert_eq!(
        crate::kernel::ratio::dollars(sales.carries[0].1[2].remaining[0].remaining.get()),
        7_000
    );
    // The remaining $3,000 cut is code R's share, not a sale QBI carry.
    assert_eq!(
        crate::kernel::ratio::dollars(sales.gain_cents)
            - crate::kernel::ratio::dollars(sales.loss_cents.get()),
        3_000
    );
}

#[test]
fn k1_sale_qbi_existing_combined_property_business_keeps_its_section179_deduction() {
    let mut source = complete_ordinary_scorp(15_000, true);
    source.section179 = 2_000 * 100;
    source.section1231 = Some(k1_section1231(10_000, 0));
    source.property_qbi =
        ScorpPropertyQbi { gain: 10_000 * 100, complete: true, ..Default::default() };
    let statement = source.qbi_statement.as_ref().unwrap();
    let values = statement
        .combined_activities(
            &source,
            [crate::kernel::carry::Amount::default(); 2],
            crate::section1231::PropertyCharacter::Ordinary,
            Some(0),
            FederalStatus::Single,
            2_000,
            0,
        )
        .unwrap();
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].income, 23_000); // $15,000 + $10,000 - $2,000.
    assert_eq!(source.ordinary, 15_000 * 100);
    assert_eq!(source.property_qbi.gain, 10_000 * 100);
}

#[test]
fn k1_sale_qbi_required_sstb_choices_keep_loss_and_phase_out_income() {
    // 2026-10-16 D05: required No retains the $1 ordinary source and the $2 sale loss;
    // required Yes phases out both current SSTB amounts above the income threshold.
    // Positive sale QBI stays unphased until the combined business owner applies eligibility.
    // The removed unknown used different conservative assumptions for positive and negative amounts.
    for (price, service, expected, sale_amounts) in [
        (22_001, true, 0, (1, 0)),
        (22_001, false, 2, (1, 0)),
        (21_998, true, 0, (0, 0)),
        (21_998, false, -1, (0, 2)),
    ] {
        let mut source = sources(1, 0, 0, price).into_iter().nth(1).unwrap();
        k1_qbi_statement(&mut source, 1, false);
        sale_rows(&mut source, vec![sale(price)], 100_000, 100_000);
        let FederalIncome::Scorp { data } = &mut source else { unreachable!() };
        data.qbi_statement.as_mut().unwrap().service = service;
        assert_eq!(data.qbi_statement.as_ref().unwrap().income, 100);
        let (draft, id) = draft(source);
        let (sales, _) = calculation(&draft, id, 300_000);
        assert_eq!(
            (
                crate::kernel::ratio::dollars(sales.gain_cents),
                crate::kernel::ratio::dollars(sales.loss_cents.get())
            ),
            sale_amounts
        );
        let entry = draft.income.iter().find(|e| e.id == id).unwrap();
        assert_eq!(business(entry, &sales, 300_000, 0).income, expected);
    }
}

#[test]
fn k1_sale_qbi_declined_scorp_statement_omits_sale_qbi_with_one_loss_notice() {
    let mut source = sources(0, 0, 0, 20_000).into_iter().nth(1).unwrap();
    k1_qbi_statement(&mut source, 0, false);
    sale_rows(&mut source, vec![sale(20_000)], 100_000, 100_000);
    let FederalIncome::Scorp { data } = &mut source else { unreachable!() };
    data.qbi_statement.as_mut().unwrap().complete = false;
    let (draft, id) = draft(source);
    let (sales, _) = calculation(&draft, id, 0);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(sales.gain_cents),
            crate::kernel::ratio::dollars(sales.loss_cents.get())
        ),
        (0, 0)
    );
    let result = estimate(&draft);
    assert_eq!(
        result
            .review_notes
            .iter()
            .filter(|note| note.display.key == "WARN_SCORP_QBI_LOSS_LEFT_OUT")
            .count(),
        1
    );
}

fn property_box(net: i64, basis_loss: i64) -> K1Section1231 {
    K1Section1231 { net, unrecaptured: 0, basis_loss, risk_loss: 0, passive_loss: 0 }
}

fn property_sale(gain: i64) -> K1Section179Sale {
    k1_reported_sale(
        "Synthetic business property",
        20230301,
        [(20_000 + gain) * 100, 20_000 * 100, 0, 0],
    )
}

fn property_source(ordinary: i64, rows: Vec<K1Section179Sale>, capacity: i64) -> ScorpK1 {
    let mut source = sources(ordinary, 0, 0, 20_000).into_iter().nth(1).unwrap();
    k1_qbi_statement(&mut source, ordinary.max(0), false);
    sale_rows(&mut source, rows, capacity, capacity);
    let FederalIncome::Scorp { data } = source else { unreachable!() };
    data
}

fn property_business(
    source: &ScorpK1,
    taxable: i64,
    treatment: crate::section1231::PropertyCharacter,
    suspended: i64,
) -> Vec<crate::business_rules::QbiActivity> {
    source
        .qbi_statement
        .as_ref()
        .unwrap()
        .combined_with_sales(
            source,
            [
                crate::kernel::carry::Amount::default(),
                crate::tests::support::builders::k1_amount(suspended),
            ],
            treatment,
            (Some(taxable), FederalStatus::Single),
            0,
            &CurrentSales::default(),
            0,
        )
        .unwrap()
}

#[test]
fn k1_sale_qbi_property_sale_without_original_box_reaches_the_business_and_coordinator() {
    let mut source = sources(15_000, 2_000, 2_000, 20_000).into_iter().nth(1).unwrap();
    k1_qbi_statement(&mut source, 15_000, false);
    let original = source.clone();
    let (d, id) = draft(source);
    let (_, pool) = calculation(&d, id, 0);
    let preview = crate::section1231::property_qbi(&d, &pool).unwrap();
    assert_eq!(preview.len(), 1);
    assert_eq!((preview[0].gain, preview[0].loss, preview[0].current_loss), (0, 4_000, 4_000));
    let r = estimate(&d);
    let combined = r
        .review_notes
        .iter()
        .find(|n| n.display.key == "NOTE_SCORP_COMBINED_BUSINESS_QBI")
        .unwrap();
    assert_eq!(combined.display.args[1], MessageArg::usd_dollars(11_000));
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope);
    assert!(r.issues.is_empty(), "{:?}", r.issues);
    assert_eq!(d.income[0].income, original);
    let FederalIncome::Scorp { data } = &original else { unreachable!() };
    assert!(data.section1231.is_none());
    assert!(!data.property_qbi.complete); // No nonexistent-box confirmation required.
    assert!(data.property_qbi.valid(data));
}

#[test]
fn k1_sale_qbi_property_combines_source_qualification_without_widening_the_original_bounds() {
    let mut source = property_source(0, vec![property_sale(-2_000)], 3_000);
    source.section1231 = Some(property_box(-300_000, 0));
    source.property_qbi = ScorpPropertyQbi {
        losses: ScorpLossQbi { current: 100_000, ..Default::default() },
        complete: true,
        ..Default::default()
    };
    let original = source.clone();
    let view = source.property_qbi.calculation(&source, Some((0, FederalStatus::Single))).unwrap();
    let release = view.calculate(&source, &source.basis_result().unwrap()).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(release.amount.get()),
            crate::kernel::ratio::dollars(release.qbi.get())
        ),
        (3_000, 1_800)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(release.remaining[0].remaining.get()),
            crate::kernel::ratio::dollars(release.remaining[0].remaining_qbi.get())
        ),
        (2_000, 1_200)
    );
    assert_eq!(
        property_business(&source, 0, crate::section1231::PropertyCharacter::Ordinary, 0)[0].income,
        -1_800
    );
    let mut invalid = source.property_qbi.clone();
    invalid.losses.current = 300_001; // Sale losses cannot enlarge the entered box's bound.
    assert!(!invalid.valid(&source));
    assert!(invalid.calculation(&source, None).is_err());
    assert_eq!(source, original);
}

#[test]
fn k1_sale_qbi_property_keeps_gross_gain_and_loss_until_the_shared_limits() {
    let source = property_source(0, vec![property_sale(10_000), property_sale(-14_000)], 0);
    let original = source.clone();
    let view = source.property_qbi.calculation(&source, None).unwrap();
    let release =
        view.after_passive(&source, crate::kernel::carry::Amount::new(0).unwrap()).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(view.gain_cents), 10_000);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(release.amount.get()),
            crate::kernel::ratio::dollars(release.qbi.get())
        ),
        (10_000, 10_000)
    );
    let basis = view.calculate(&source, &source.basis_result().unwrap()).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(basis.remaining[0].remaining.get()),
            crate::kernel::ratio::dollars(basis.remaining[0].remaining_qbi.get())
        ),
        (4_000, 4_000)
    );
    assert_eq!(
        property_business(&source, 0, crate::section1231::PropertyCharacter::Ordinary, 0)[0].income,
        0
    );
    assert_eq!(source, original);
}

#[test]
fn k1_sale_qbi_property_fixes_combined_current_sstb_qualification_before_basis_rounding() {
    let mut source = property_source(0, vec![property_sale(-1)], 1);
    source.section1231 = Some(property_box(-100, 0));
    source.property_qbi = ScorpPropertyQbi {
        losses: ScorpLossQbi { current: 100, ..Default::default() },
        complete: true,
        ..Default::default()
    };
    source.qbi_statement.as_mut().unwrap().service = true;
    // One third of combined 200c is 67c. Basis permits 100c: the existing
    // ratio allocates 34c qualified and retains 33c, before report projection.
    let view =
        source.property_qbi.calculation(&source, Some((251_750, FederalStatus::Single))).unwrap();
    let basis = view.calculate(&source, &source.basis_result().unwrap()).unwrap();
    assert_eq!((basis.amount.get(), basis.qbi.get()), (100, 34));
    assert_eq!(
        (basis.remaining[0].remaining.get(), basis.remaining[0].remaining_qbi.get()),
        (100, 33)
    );
    assert_eq!(
        property_business(&source, 251_750, crate::section1231::PropertyCharacter::Ordinary, 0)[0]
            .income,
        0
    );
    assert_eq!(source.property_qbi.losses.current, 100);
}

#[test]
fn k1_sale_qbi_property_gain_joins_the_same_business_before_sstb_rounding() {
    let mut source = property_source(1, vec![property_sale(1)], 0);
    source.section1231 = Some(property_box(100, 0));
    source.property_qbi = ScorpPropertyQbi { gain: 100, complete: true, ..Default::default() };
    source.qbi_statement.as_mut().unwrap().service = true;
    let values =
        property_business(&source, 239_250, crate::section1231::PropertyCharacter::Ordinary, 0);
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].income, 2); // Half of $1 ordinary + $1 box + $1 sale.
    assert_eq!(source.property_qbi.gain, 100);
}

#[test]
fn k1_sale_qbi_property_capital_character_excludes_sale_gains_and_losses() {
    let source = property_source(0, vec![property_sale(10_000), property_sale(-4_000)], 0);
    let (d, id) = draft(FederalIncome::Scorp { data: source.clone() });
    let (_, pool) = calculation(&d, id, 0);
    let preview = crate::section1231::property_qbi(&d, &pool).unwrap();
    assert_eq!((preview[0].gain, preview[0].loss, preview[0].current_loss), (0, 0, 0));
    assert_eq!(
        property_business(&source, 0, crate::section1231::PropertyCharacter::Capital, 0)[0].income,
        0
    );
}

#[test]
fn k1_sale_qbi_property_recapture_of_sale_only_gain_uses_the_known_qualified_share() {
    let source = property_source(0, vec![property_sale(10_000)], 0);
    let (mut d, id) = draft(FederalIncome::Scorp { data: source.clone() });
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 3_000, 3_000)], &[]));
    let (_, pool) = calculation(&d, id, 0);
    let preview = crate::section1231::property_qbi(&d, &pool).unwrap();
    assert_eq!((preview[0].gain, preview[0].loss), (3_000, 0));
    assert_eq!(
        property_business(&source, 0, crate::section1231::PropertyCharacter::Recaptured(3_000), 0)
            [0]
        .income,
        3_000
    );
}

#[test]
fn k1_sale_qbi_property_mixed_qualified_partial_lookback_keeps_the_named_refusal() {
    let mut source = property_source(0, vec![property_sale(10_000)], 0);
    source.section1231 = Some(property_box(1_000_000, 0));
    source.property_qbi = ScorpPropertyQbi { gain: 500_000, complete: true, ..Default::default() };
    let view = source.property_qbi.calculation(&source, None).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(view.gain_cents), 15_000);
    assert_eq!(
        view.qualified_recapture(&source, 5_000).unwrap_err().0,
        "ISSUE_SCORP_QBI_PARTIAL_LOOKBACK_RECAPTURE_OF_MIXED_QUALIFIED_NONQUALIFIED_9084BCB0"
    );
    let mut invalid = source.property_qbi.clone();
    invalid.gain = 1_000_001;
    assert!(!invalid.valid(&source));
    assert!(invalid.calculation(&source, None).is_err());
}

#[test]
fn k1_sale_qbi_property_released_history_keeps_its_original_year_qualification() {
    for (year, old_qualified) in [(2017, 0), (2025, 1_000)] {
        let mut source = property_source(0, vec![property_sale(-4_000)], 5_000);
        source.section1231 = Some(property_box(0, 200_000));
        source.property_qbi = ScorpPropertyQbi {
            losses: ScorpLossQbi {
                history: vec![loss_year(year, 2_000, old_qualified, 2_000, old_qualified)],
                ..Default::default()
            },
            complete: true,
            ..Default::default()
        };
        source.qbi_statement.as_mut().unwrap().service = true;
        let original = source.clone();
        let values =
            property_business(&source, 300_000, crate::section1231::PropertyCharacter::Ordinary, 0);
        assert_eq!(values[0].income, 0); // New SSTB loss has zero qualification.
        assert_eq!(-values.iter().skip(1).map(|v| v.income).sum::<i64>(), old_qualified);
        assert!(values.iter().skip(1).all(|v| v.service == Some(false)));
        let view = source
            .property_qbi
            .calculation(&source, Some((300_000, FederalStatus::Single)))
            .unwrap();
        let basis = view.calculate(&source, &source.basis_result().unwrap()).unwrap();
        assert_eq!(
            (
                crate::kernel::ratio::dollars(basis.amount.get()),
                crate::kernel::ratio::dollars(basis.qbi.get())
            ),
            (5_000, old_qualified)
        );
        let current = basis.remaining.iter().find(|v| v.year == 2026).unwrap();
        assert_eq!(
            (
                crate::kernel::ratio::dollars(current.remaining.get()),
                crate::kernel::ratio::dollars(current.remaining_qbi.get())
            ),
            (1_000, 0)
        );
        assert_eq!(source, original);
    }
}

#[test]
fn k1_sale_qbi_property_missing_statement_omits_new_sale_qualification_and_retains_history() {
    let mut source = property_source(0, vec![property_sale(-4_000)], 5_000);
    source.section1231 = Some(property_box(0, 200_000));
    source.property_qbi = ScorpPropertyQbi {
        losses: ScorpLossQbi {
            history: vec![loss_year(2025, 2_000, 1_000, 2_000, 1_000)],
            ..Default::default()
        },
        complete: true,
        ..Default::default()
    };
    source.qbi_statement = None;
    let values = source
        .historical_qbi_activities(
            [crate::kernel::carry::Amount::default(), crate::kernel::carry::Amount::default()],
            Some(crate::section1231::PropertyCharacter::Ordinary),
        )
        .unwrap();
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].income, -1_000);
    let projection = source.qbi_projection();
    let view = projection.property_qbi.calculation(&projection, None).unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(
            view.after_passive(&projection, crate::kernel::carry::Amount::new(0).unwrap())
                .unwrap()
                .qbi
                .get()
        ),
        1_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            view.current_allowed(&projection, crate::kernel::carry::Amount::new(0).unwrap())
                .unwrap()
        ),
        0
    );
}

#[test]
fn k1_sale_qbi_property_full_recapture_keeps_current_loss_and_history_separate() {
    let mut source = property_source(0, vec![property_sale(10_000), property_sale(-4_000)], 0);
    source.qbi_statement.as_mut().unwrap().service = true;
    // Fully ordinary net $6,000 retains distinct gross current items. Their
    // eligibility is applied once, and no partial allocation is inferred.
    assert_eq!(
        property_business(
            &source,
            239_250,
            crate::section1231::PropertyCharacter::Recaptured(6_000),
            0
        )[0]
        .income,
        3_000
    );
}

#[test]
fn k1_sale_qbi_property_preserves_minor_unit_history_reconciliation_and_rounds_current_once() {
    let mut source = property_source(0, vec![property_sale(-2)], 5);
    source.section1231 = Some(property_box(-149, 149));
    source.property_qbi = ScorpPropertyQbi {
        losses: ScorpLossQbi {
            current: 149,
            history: vec![ScorpLossYear {
                year: 2025,
                original: 149,
                qualified: 49,
                remaining: 149,
                remaining_qualified: 49,
            }],
            ..Default::default()
        },
        complete: true,
        ..Default::default()
    };
    assert!(source.property_qbi.valid(&source));
    let view = source.property_qbi.calculation(&source, None).unwrap();
    let release = view.calculate(&source, &source.basis_result().unwrap()).unwrap();
    // 200c sale +149c current +149c history; qualified349c+49c.
    assert_eq!((release.amount.get(), release.qbi.get()), (498, 398));
    assert_eq!(source.property_qbi.losses.history[0].remaining, 149);
    assert_eq!(source.property_qbi.losses.current, 149);
}

#[test]
fn k1_sale_qbi_property_sale_only_passive_loss_uses_the_property_character_cut() {
    let mut source = property_source(0, vec![property_sale(-2_000), property_sale(1_000)], 5_000);
    source.participation.as_mut().unwrap().minutes = 0;
    let (d, id) = draft(FederalIncome::Scorp { data: source.clone() });
    let (_, pool) = calculation(&d, id, 0);
    let item = pool.k1_section1231.iter().find(|v| v.id == id).unwrap();
    assert_eq!((item.net, item.suspended), (0, 1_000));
    let view = source.property_qbi.calculation(&source, None).unwrap();
    let release = view
        .after_passive(&source, crate::kernel::carry::Amount::new(1_000 * 100).unwrap())
        .unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(release.amount.get()),
            crate::kernel::ratio::dollars(release.qbi.get())
        ),
        (1_000, 1_000)
    );
    let current = release.remaining.iter().find(|v| v.year == 2026).unwrap();
    assert_eq!((current.remaining.get(), current.remaining_qbi.get()), (100_000, 100_000));
    assert_eq!(
        property_business(&source, 0, crate::section1231::PropertyCharacter::Ordinary, 1_000)[0]
            .income,
        0
    );
}

#[test]
fn k1_sale_qbi_property_new_qualified_loss_passes_basis_then_stays_at_risk() {
    let mut source = property_source(0, vec![property_sale(-5_000)], 5_000);
    source.risk.start = ScorpRiskStart::Detailed { capacity: 200_000, profit: 0, loss: 0 };
    let view = source.property_qbi.calculation(&source, None).unwrap();
    let basis = source.basis_result().unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(view.calculate(&source, &basis).unwrap().amount.get()),
        5_000
    );
    let risk = view.after_at_risk(&source, &basis).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(risk.amount.get()),
            crate::kernel::ratio::dollars(risk.qbi.get())
        ),
        (2_000, 2_000)
    );
    let current = risk.remaining.iter().find(|v| v.year == 2026).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(current.remaining.get()),
            crate::kernel::ratio::dollars(current.remaining_qbi.get())
        ),
        (3_000, 3_000)
    );
    assert_eq!(
        property_business(&source, 0, crate::section1231::PropertyCharacter::Ordinary, 0)[0].income,
        -2_000
    );
}

#[test]
fn k1_sale_qbi_property_required_service_keeps_losses_and_declined_statement_omits_new_items() {
    for (amount, expected) in [(-2, -2), (2, 0)] {
        let mut source = property_source(0, vec![property_sale(amount)], 5);
        source.qbi_statement.as_mut().unwrap().service = amount > 0;
        assert_eq!(
            property_business(&source, 300_000, crate::section1231::PropertyCharacter::Ordinary, 0)
                [0]
            .income,
            expected
        );
        source.qbi_statement.as_mut().unwrap().complete = false;
        let view = source.property_qbi.calculation(&source, None).unwrap();
        assert_eq!(crate::kernel::ratio::dollars(view.gain_cents), 0);
        assert_eq!(
            crate::kernel::ratio::dollars(
                view.after_passive(&source, crate::kernel::carry::Amount::new(0).unwrap())
                    .unwrap()
                    .qbi
                    .get()
            ),
            0
        );
    }
}

#[test]
fn k1_sale_qbi_property_sale_low_rate_class_preserves_known_1250_qualification() {
    let mut source = property_source(0, vec![property_sale(5_000)], 0);
    source.section1231 = Some(k1_section1231(10000, 5000));
    source.property_qbi = ScorpPropertyQbi {
        gain: 500_000,
        qualified_1250: Some(0),
        complete: true,
        ..Default::default()
    };
    let view = source.property_qbi.calculation(&source, None).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(view.gain_cents), 10_000);
    // $5,000 unqualified high-rate gain is recaptured first. The remaining
    // $3,000 is from a wholly qualified $10,000 low-rate class, including SALE.
    assert_eq!(view.qualified_recapture(&source, 8_000), Ok(3_000));
    assert_eq!(
        property_business(&source, 0, crate::section1231::PropertyCharacter::Recaptured(8_000), 0)
            [0]
        .income,
        3_000
    );
    assert_eq!(source.property_qbi.qualified_1250, Some(0));
}

#[test]
fn k1_sale_qbi_property_full_recapture_never_rephases_released_historical_losses() {
    let mut source = property_source(0, vec![property_sale(10_000), property_sale(-4_000)], 0);
    source.section1231 = Some(property_box(0, 200_000));
    source.property_qbi = ScorpPropertyQbi {
        losses: ScorpLossQbi {
            history: vec![loss_year(2025, 2_000, 2_000, 2_000, 2_000)],
            ..Default::default()
        },
        complete: true,
        ..Default::default()
    };
    source.qbi_statement.as_mut().unwrap().service = true;
    let values = property_business(
        &source,
        239_250,
        crate::section1231::PropertyCharacter::Recaptured(4_000),
        0,
    );
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].income, 3_000); // Half of gain $10k less qualified current loss $2k.
    assert_eq!(values[1].income, -2_000); // Prior loss keeps 100% qualification.
    assert_eq!(values[1].service, Some(false));
    assert_eq!(values.iter().map(|v| v.income).sum::<i64>(), 1_000);
}

#[test]
fn k1_sale_qbi_property_coordinator_uses_the_business_component_of_shared_lookback() {
    let mut source = property_source(0, vec![property_sale(10_000)], 0);
    source.rentals.real_estate_property =
        K1RentalProperty { net: 500_000, unrecaptured: 500_000, ..Default::default() };
    source.rental_facts.real_estate_qbi = Some(K1RentalQbi { wages: 0, property: 0 });
    let (mut d, id) = draft(FederalIncome::Scorp { data: source });
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 5_000, 5_000)], &[]));
    let (_, pool) = calculation(&d, id, 0);
    assert_eq!(
        crate::section1231::rental_property_qbi(&d, &pool).unwrap().current,
        vec![(0, 5_000, Some(500_000))]
    );
    // The rental's entire $5,000 high-rate class is recaptured first. The
    // separate $10,000 SALE low-rate business gain remains capital, so it
    // must not receive the rental's ordinary/QBI share a second time.
    let r = estimate(&d);
    let note = r
        .review_notes
        .iter()
        .find(|n| n.display.key == "NOTE_SCORP_COMBINED_BUSINESS_QBI")
        .unwrap();
    assert_eq!(note.display.args[1], MessageArg::usd_dollars(0));
    assert!(
        !r.issues.iter().any(|i| i.display.key.starts_with("ISSUE_SCORP_QBI_")),
        "{:?}",
        r.issues
    );
}

#[test]
fn k1_sale_qbi_property_partial_recapture_of_wholly_qualified_current_net_needs_no_allocation() {
    let mut source = property_source(0, vec![property_sale(10_000), property_sale(-4_000)], 0);
    source.qbi_statement.as_mut().unwrap().service = true;
    for (taxable, expected) in [(0, 3_000), (239_250, 1_500)] {
        let view = source
            .property_qbi
            .calculation(&source, Some((taxable, FederalStatus::Single)))
            .unwrap();
        assert_eq!(view.qualified_recapture(&source, 3_000), Ok(3_000));
        assert_eq!(
            property_business(
                &source,
                taxable,
                crate::section1231::PropertyCharacter::Recaptured(3_000),
                0
            )[0]
            .income,
            expected
        );
    }
    let (mut d, id) = draft(FederalIncome::Scorp { data: source });
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 3_000, 3_000)], &[]));
    let (_, pool) = calculation(&d, id, 0);
    let preview = crate::section1231::property_qbi(&d, &pool).unwrap();
    assert_eq!((preview[0].gain, preview[0].loss), (3_000, 0));
}
