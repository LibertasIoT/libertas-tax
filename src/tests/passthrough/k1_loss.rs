// K-1 loss carryforwards (unit B8): the fixtures of the k1-loss-carryforwards
// map, by hand. Every amount is synthetic. Single filer with $60,000 wages;
// 2026 standard deduction $16,100, the single brackets of Rev. Proc.
// 2025-32 (10% to $12,400, 12% to $50,400, 22% to $105,700) and the section
// 170(p) non-itemizer cash deduction up to $1,000. The pro rata allocation of
// every limit is exact in these fixtures.
use super::*;
extern crate std;
use crate::kernel::ratio::dollars;
use alloc::format;

fn amounts(items: &[(K1LossClass, i64)]) -> LossAmounts {
    LossAmounts::of(
        &items.iter().map(|(class, value)| (*class, k1_amount(*value))).collect::<Vec<_>>(),
    )
}

fn separate(
    interest: i64,
    short: i64,
    long: i64,
    cash_charity: i64,
    foreign_taxes: i64,
) -> PartnerSeparateTotals {
    PartnerSeparateTotals {
        interest: interest * 100,
        dividends: 0,
        qualified_dividends: 0,
        royalties: 0,
        short: short * 100,
        long: long * 100,
        collectibles: 0,
        unrecaptured_partnership: 0,
        unrecaptured_fund: 0,
        cash_charity: cash_charity * 100,
        backup_withholding: 0,
        exempt_interest: 0,
        foreign_taxes: foreign_taxes * 100,
    }
}

// The opening outside basis and the detailed Form 6198 history of both tax
// systems, whole dollars.
fn start(partner: &mut PartnerK1, opening: i64, capacity: i64, loss: i64) {
    for record in core::iter::once(&mut partner.regular).chain(partner.alternative.as_mut()) {
        record.basis.opening = Some(opening * 100);
        record.risk.start =
            ScorpRiskStart::Detailed { capacity: capacity * 100, profit: 0, loss: loss * 100 };
    }
}

fn mirror_amt(partner: &mut PartnerK1) {
    partner.alternative = Some(partner.regular.clone());
}

fn draft_of(partner: PartnerK1) -> (FederalDraft, i64) {
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    (d, id)
}

fn shareholder_draft_of(mut source: ScorpK1) -> (FederalDraft, i64) {
    source.amt = Some(amt_records(&source));
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    (d, id)
}

fn noted(r: &FederalResult, text: &str) -> bool {
    r.review_notes.iter().any(|v| v.contains(text))
}

fn refused_with(r: &FederalResult, key: &str) -> bool {
    r.state == ResultState::Unsupported && r.issues.iter().any(|i| i.display.key == key)
}

// A materially participating LLC member (box 14 code A blank): box 1 `profit`
// with no section 199A statement and no self-employment.
fn member(profit: i64) -> PartnerK1 {
    ordinary_partner_source(profit, true)
}

#[test]
fn unit19_basis_held_investment_interest_cannot_reappear_when_sources_are_added() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(savings_income(1_000, 0)));
    investment_borrowing(&mut d, 1, 0, 0);
    let control = supported(&d);
    for _ in 0..2 {
        let mut p = member(0);
        p.deductions =
            vec![PartnerDeduction { code: PartnerDeductionCode::InvestmentInterest, amount: 149 }];
        start(&mut p, 0, 0, 0);
        mirror_amt(&mut p);
        let pool = p.pool(&p.regular).unwrap();
        assert_eq!(
            dollars(pool.activity.ledger.basis_allowed[K1LossClass::InvestmentInterest].get()),
            0
        );
        assert_eq!(
            pool.activity.ledger.basis_suspended[K1LossClass::InvestmentInterest].get(),
            149
        );
        add(&mut d, income(FederalIncome::Partnership { data: p }));
    }
    let r = supported(&d);
    assert_eq!(value(&r, "investment_form_line1"), 1);
    assert_eq!(value(&r, "investment_form_line8"), 1);
    assert_eq!(value(&r, "amt_investment_form_line1"), 1);
    assert_eq!(value(&r, "agi"), value(&control, "agi"));
    assert_eq!(r.tax, control.tax);
    // No investment income or independent borrowing means no new Form4952
    // carry: fully basis-held sources must not create a future deduction.
    d.income.retain(|entry| !matches!(entry.income, FederalIncome::Interest { .. }));
    investment_borrowing(&mut d, 0, 0, 0);
    let no_income = supported(&d);
    for id in [
        "investment_form_line1",
        "investment_form_line7",
        "investment_form_line8",
        "amt_investment_form_line1",
        "amt_investment_form_line7",
        "amt_investment_form_line8",
    ] {
        assert_eq!(value(&no_income, id), 0, "{id}");
    }
}

#[test]
fn unit19_fully_held_deduction_projection_keeps_unlimited_and_subdollar_inputs() {
    for (raw, basis, expected) in [(149, 0, 0), (149, 2, 149), (49, 0, 0), (150, 0, 0)] {
        let mut p = member(0);
        p.deductions =
            vec![PartnerDeduction { code: PartnerDeductionCode::InvestmentInterest, amount: raw }];
        start(&mut p, basis, basis, 0);
        mirror_amt(&mut p);
        for record in [&p.regular, p.alternative.as_ref().unwrap()] {
            assert_eq!(p.outcome(record).unwrap().0.investment_interest, expected);
        }
    }
}

// The class catalog is the only place a class is named: every slot is a
// position in ALL, so a class added there needs no change in the ledger.
#[test]
fn catalog_slots_are_positions() {
    for (position, class) in K1LossClass::ALL.into_iter().enumerate() {
        assert_eq!(class as usize, position);
    }
    assert_eq!(LossAmounts::default().as_slice().len(), K1LossClass::COUNT);
    // A pro rata share of every class, in catalog order, conserving the amount.
    let all = amounts(&[
        (K1LossClass::Ordinary, 6_000),
        (K1LossClass::ShortTerm, 2_000),
        (K1LossClass::LongTerm, 4_000),
        (K1LossClass::Section179, 8_000),
    ]);
    let (allowed, held) = all.limit(k1_amount(5_000));
    assert_eq!(allowed.total().get() + held.total().get(), all.total().get());
    assert_eq!(dollars(allowed.total().get()), 5_000);
    assert_eq!(dollars(allowed[K1LossClass::Section179].get()), 2_000);
    assert_eq!(dollars(held[K1LossClass::Ordinary].get()), 4_500);
    // The classes outside the Form 6198 activity.
    for class in [K1LossClass::Section179, K1LossClass::Charity, K1LossClass::ForeignTaxes] {
        assert!(!class.at_risk());
    }
}

// Every catalog class takes part in both stages without any class-specific
// code: a class added to ALL is limited by the basis, and by the at-risk
// amount when it is an activity deduction, and passes the stage it is outside.
#[test]
fn every_catalog_class_takes_part_in_both_stages() {
    use crate::k1_loss::limit_classes;
    for class in K1LossClass::ALL {
        let only = amounts(&[(class, 1_000)]);
        // Basis of 400 allows 400 of the class and holds back 600.
        let (allowed, held) = only.limit(k1_amount(400));
        assert_eq!((dollars(allowed[class].get()), dollars(held[class].get())), (400, 600));
        // An amount at risk of 250 limits an activity deduction further and
        // never touches a class outside Form 6198.
        let stage = limit_classes(
            k1_amount(0),
            k1_amount(250),
            &allowed,
            &LossAmounts::default(),
            k1_amount(0),
        )
        .unwrap();
        if class.at_risk() {
            assert_eq!(
                (dollars(stage.allowed[class].get()), dollars(stage.suspended[class].get())),
                (250, 150)
            );
        } else {
            assert_eq!(
                (dollars(stage.allowed[class].get()), dollars(stage.suspended[class].get())),
                (400, 0)
            );
        }
        // An at-risk carry already passed basis: it adds to the stage only.
        let carry = amounts(&[(class, 100)]);
        let stage = limit_classes(
            k1_amount(0),
            k1_amount(1_000),
            &LossAmounts::default(),
            &carry,
            k1_amount(0),
        )
        .unwrap();
        assert_eq!(dollars(stage.allowed[class].get()), if class.at_risk() { 100 } else { 0 });
        // Every class has a resource name.
        assert!(
            APP_STRINGS.iter().any(|(key, _)| *key == class.key()),
            "{} is undeclared",
            class.key()
        );
    }
}

// Fixture 1a (Reg. 1.704-1(d)(4) Example 3 extended). Opening basis $1,000;
// box 1 -6,000, box 5 4,000, box 8 -2,000, box 9a -4,000, box 12 8,000; the
// Form 6198 history is $1,000 of capacity. Basis before losses 1,000 + 4,000
// (line 4) = 5,000 against classes 6,000 / 2,000 / 4,000 / 8,000 (20,000):
// 1,500 / 500 / 1,000 / 2,000 allowed and 4,500 / 1,500 / 3,000 / 6,000
// carried (column E); ending basis 0. Form 6198: income 4,000 and losses
// 1,500 + 500 + 1,000 = 3,000 are allowed, and the amount at risk ends at
// 1,000 + 4,000 - 3,000 = 2,000. Form 4562 line 6 is the basis-allowed 2,000,
// not 8,000; line 11 = 60,000 - 1,500 = 58,500; 2,000 equals the remaining
// amount at risk (one more dollar would be ISSUE_SECTION179_RISK_CARRYFORWARD).
// Schedule B 4,000; Schedule E -1,500 and -2,000; Schedule D -500 and -1,000
// (-1,500). AGI 60,000 + 4,000 - 1,500 - 2,000 - 1,500 = 59,000; taxable
// 59,000 - 16,100 = 42,900.
fn fixture_1a_partner(interest: i64) -> PartnerK1 {
    let mut partner = member(-6_000);
    partner.separate = Some(separate(interest, -2_000, -4_000, 0, 0));
    partner.section179 = 800_000;
    start(&mut partner, 1_000, 1_000, 0);
    partner
}

#[test]
fn fixture_1a_partner_four_classes_limited_by_basis_share_the_basis_pro_rata() {
    let partner = fixture_1a_partner(4_000);
    let pool = partner.pool(&partner.regular).unwrap();
    let ledger = pool.activity.ledger;
    assert_eq!(
        ledger.basis_allowed,
        amounts(&[
            (K1LossClass::Ordinary, 1_500),
            (K1LossClass::ShortTerm, 500),
            (K1LossClass::LongTerm, 1_000),
            (K1LossClass::Section179, 2_000),
        ])
    );
    assert_eq!(
        ledger.basis_suspended,
        amounts(&[
            (K1LossClass::Ordinary, 4_500),
            (K1LossClass::ShortTerm, 1_500),
            (K1LossClass::LongTerm, 3_000),
            (K1LossClass::Section179, 6_000),
        ])
    );
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 0);
    // Form 6198 holds nothing back; the section 179 class is outside it.
    assert_eq!(ledger.risk_allowed, ledger.basis_allowed);
    assert_eq!(ledger.risk_suspended, LossAmounts::default());
    assert_eq!(pool.activity.risk_end.map(crate::kernel::ratio::dollars), Some(2_000));
    let (d, id) = draft_of(partner);
    let r = supported(&d);
    assert_eq!(value(&r, &format!("partnership_{id}_section179")), 2_000);
    assert_eq!(value(&r, "section179_business_income_limit"), 58_500);
    assert_eq!(return_line(&r, "2b"), 400_000);
    assert_eq!(return_line(&r, "7a"), -150_000);
    assert_eq!(return_line(&r, "8"), -350_000);
    assert_eq!(return_line(&r, "11a / 11b"), 5_900_000);
    assert_eq!(return_line(&r, "15"), 4_290_000);
    // The ordinary carry keeps its own record; the three classes this unit
    // adds each get one note.
    assert!(noted(
        &r,
        "short-term capital loss carried to 2027 — $1500 because of your basis, $0 because of the at-risk limits"
    ));
    assert!(noted(
        &r,
        "long-term capital loss carried to 2027 — $3000 because of your basis, $0 because of the at-risk limits"
    ));
    assert!(noted(
        &r,
        "section 179 deduction carried to 2027 — $6000 because of your basis, $0 because of the at-risk limits"
    ));
    assert!(!noted(&r, "ordinary business loss carried to 2027"));
    // The stored return reopens to the same ledger and the same estimate.
    assert_eq!(estimate(&restart(d.clone()).draft), r);
}

// Fixture 1a', the boundary: box 5 of 19,000 makes the basis 20,000, the
// total of the classes, so everything is allowed and nothing is carried.
#[test]
fn fixture_1a_prime_basis_equal_to_the_classes_carries_nothing() {
    let partner = fixture_1a_partner(19_000);
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(pool.activity.ledger.basis_suspended, LossAmounts::default());
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 0);
    let (d, _) = draft_of(partner);
    let r = supported(&d);
    assert!(!noted(&r, "carried to 2027"));
}

// One dollar of section 179 above the remaining amount at risk (fixture 1a
// at the boundary): the K-1 is refused by name with the narrowed reason.
#[test]
fn fixture_1a_section_179_one_dollar_above_the_amount_at_risk_is_refused() {
    let mut partner = fixture_1a_partner(4_000);
    // Capacity 999 leaves 1,999 at the end, below the allowed 2,000.
    start(&mut partner, 1_000, 999, 0);
    let (d, _) = draft_of(partner);
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(refused_with(&r, "ISSUE_SECTION179_RISK_CARRYFORWARD"));
}

// Fixture 1b: the same partner in 2026 releasing the carries. Continuing,
// opening basis 0; carried 4,500 / 1,500 / 3,000 / 6,000; box 1 +10,000 and a
// Form 6198 history of zero. Basis 0 + 10,000 = 10,000 against 15,000 of
// carried classes: 3,000 / 1,000 / 2,000 / 4,000 allowed and 1,500 / 500 /
// 1,000 / 2,000 carried to 2027. Form 6198: income 10,000 and losses 6,000
// are allowed; 4,000 remains at risk. Form 4562 line 6 is 4,000 (the
// released share joins the current year, Reg. 1.1366-2(a)(3)); line 11 = 60,000
// + (10,000 - 3,000) = 67,000. Schedule E +7,000 and -4,000; Schedule D
// -1,000 + -2,000 = -3,000, exactly the section 1211(b) limit. AGI
// 60,000 + 7,000 - 4,000 - 3,000 = 60,000; taxable 43,900.
fn fixture_1b_partner() -> PartnerK1 {
    let mut partner = member(10_000);
    partner.regular.basis.prior_loss = 450_000;
    partner.regular.capital_carried.short.basis = 150_000;
    partner.regular.capital_carried.long.basis = 300_000;
    partner.regular.carried.section179 = 600_000;
    start(&mut partner, 0, 0, 0);
    mirror_amt(&mut partner);
    partner
}

#[test]
fn fixture_1b_partner_releases_the_carries_of_every_class() {
    let partner = fixture_1b_partner();
    let pool = partner.pool(&partner.regular).unwrap();
    let ledger = pool.activity.ledger;
    assert_eq!(
        ledger.basis_allowed,
        amounts(&[
            (K1LossClass::Ordinary, 3_000),
            (K1LossClass::ShortTerm, 1_000),
            (K1LossClass::LongTerm, 2_000),
            (K1LossClass::Section179, 4_000),
        ])
    );
    assert_eq!(
        ledger.basis_suspended,
        amounts(&[
            (K1LossClass::Ordinary, 1_500),
            (K1LossClass::ShortTerm, 500),
            (K1LossClass::LongTerm, 1_000),
            (K1LossClass::Section179, 2_000),
        ])
    );
    assert_eq!(pool.activity.risk_end.map(crate::kernel::ratio::dollars), Some(4_000));
    let (d, id) = draft_of(partner);
    let r = supported(&d);
    assert_eq!(value(&r, &format!("partnership_{id}_section179")), 4_000);
    assert_eq!(value(&r, "section179_business_income_limit"), 67_000);
    assert_eq!(return_line(&r, "7a"), -300_000);
    assert_eq!(return_line(&r, "8"), 300_000);
    assert_eq!(return_line(&r, "11a / 11b"), 6_000_000);
    assert_eq!(return_line(&r, "15"), 4_390_000);
    assert!(noted(&r, "short-term capital loss carried to 2027 — $500 because of your basis"));
    assert!(noted(&r, "section 179 deduction carried to 2027 — $2000 because of your basis"));
}

// A box 9b amount with a limited long-term class, and with a long-term loss
// carried from 2025, is refused by name: the instructions do not say how the
// allowed part divides among 9a, 9b and the 9c parts.
#[test]
fn partner_box_9b_with_a_limited_or_released_long_term_loss_is_refused_by_name() {
    let mut partner = fixture_1a_partner(4_000);
    partner.section179 = 0;
    partner.separate.as_mut().unwrap().collectibles = -50_000;
    let (d, _) = draft_of(partner.clone());
    let r = estimate(&d);
    assert!(refused_with(&r, "ISSUE_PARTNER_LONG_TERM_PARTS_LIMITED"));
    // Fully absorbed by basis, box 9b is the ordinary part of box 9a.
    let mut absorbed = partner.clone();
    start(&mut absorbed, 50_000, 50_000, 0);
    let (d, _) = draft_of(absorbed);
    assert!(!refused_with(&estimate(&d), "ISSUE_PARTNER_LONG_TERM_PARTS_LIMITED"));
    // A long-term loss carried from 2025 with a box 9c part.
    let mut released = member(10_000);
    released.separate = Some(separate(0, 0, 0, 0, 0));
    released.separate.as_mut().unwrap().unrecaptured_fund = 100_000;
    released.regular.capital_carried.long.basis = 100_000;
    start(&mut released, 50_000, 50_000, 0);
    mirror_amt(&mut released);
    let (d, _) = draft_of(released);
    assert!(refused_with(&estimate(&d), "ISSUE_PARTNER_LONG_TERM_PARTS_LIMITED"));
}

// Fixture 2a, a shareholder: basis, then at-risk, then passive. Materially
// participating in the trade, box 2 activity Active. Opening stock basis
// 10,000 (no loans); the Form 6198 history is 5,000 of capacity (5,000 of the
// stock was bought with a related-party nonrecourse loan). Box 1 -12,000,
// box 2 -4,000, box 12 code A 4,000. Form 7203 Part III lines 35 / 36 / 42
// (20,000) against 10,000: 6,000 / 2,000 / 2,000 allowed and carried. Form
// 6198 (the gift is outside): income 0, losses 6,000 + 2,000 = 8,000 against
// 5,000 at risk: 3,750 and 1,250 allowed, 2,250 and 750 carried at risk.
// Form 8582: box 2 -1,250 in Part IV, MAGI 60,000 - 3,750 = 56,250, line 7
// 93,750 x 50% = 46,875 so the 25,000 allowance covers it. Schedule E -3,750
// and -1,250: AGI 55,000; the allowed gift 2,000 is the 1,000 section 170(p)
// deduction; taxable 55,000 - 16,100 - 1,000 = 37,900.
fn fixture_2a_shareholder() -> ScorpK1 {
    let mut source = complete_ordinary_scorp(0, true);
    source.qbi_statement = None;
    source.loans.clear();
    source.opening = Some(1_000_000);
    source.ordinary = -1_200_000;
    source.rentals.real_estate = -400_000;
    source.rental_facts.participation = Some(K1RentalParticipation::Active);
    source.cash_charity = 400_000;
    source.risk.deducted = 0;
    source.risk.start = ScorpRiskStart::Detailed { capacity: 500_000, profit: 0, loss: 0 };
    source.amt = Some(amt_records(&source));
    source
}

#[test]
fn fixture_2a_shareholder_loss_passes_basis_then_at_risk_then_passive() {
    let source = fixture_2a_shareholder();
    let ledger = source.ledger().unwrap();
    let classes = |ordinary, rental, charity| {
        amounts(&[
            (K1LossClass::Ordinary, ordinary),
            (K1LossClass::RentalRealEstate, rental),
            (K1LossClass::Charity, charity),
        ])
    };
    assert_eq!(ledger.basis_allowed, classes(6_000, 2_000, 2_000));
    assert_eq!(ledger.basis_suspended, classes(6_000, 2_000, 2_000));
    assert_eq!(ledger.risk_allowed, classes(3_750, 1_250, 2_000));
    assert_eq!(ledger.risk_suspended, classes(2_250, 750, 0));
    let (d, id) = shareholder_draft_of(source);
    let r = supported(&d);
    // The passive stage allows the whole 1,250 rental loss.
    assert_eq!(value(&r, &format!("scorp_{id}_rental_real_estate_net")), -1_250);
    assert_eq!(value(&r, &format!("scorp_{id}_rental_real_estate_suspended")), 0);
    assert_eq!(return_line(&r, "11a / 11b"), 5_500_000);
    assert_eq!(return_line(&r, "12f"), 100_000);
    assert_eq!(return_line(&r, "15"), 3_790_000);
    assert!(noted(
        &r,
        "rental real estate loss carried to 2027 — $2000 because of your basis, $750 because of the at-risk limits"
    ));
    assert!(noted(
        &r,
        "cash contributions carried to 2027 — $2000 because of your basis, $0 because of the at-risk limits"
    ));
    assert_eq!(estimate(&restart(d.clone()).draft), r);
}

// Fixture 2b: the same shareholder in 2026. Continuing, opening 0; carried
// basis 6,000 (ordinary), 2,000 (rental), 2,000 (gift); at-risk 2,250
// (ordinary) and 750 (rental); the Form 6198 history is capacity 5,000 less a
// prior line 21 loss of 5,000 (zero now); box 1 +9,000. Basis 9,000 against
// 10,000: 5,400 / 1,800 / 1,800 allowed, 600 / 200 / 200 carried. Form 6198:
// income 9,000, amount at risk 0; ordinary 5,400 + 2,250 = 7,650 and rental
// 1,800 + 750 = 2,550 (10,200) share 9,000: 6,750 and 2,250 allowed, 900 and
// 300 carried at risk. Passive: box 1 nonpassive 9,000 - 6,750 = +2,250; box
// 2 -2,250 (Active, MAGI 62,250, within the allowance). AGI 60,000 + 2,250 -
// 2,250 = 60,000; the gift 1,800 is the 1,000 deduction; taxable 42,900.
fn fixture_2b_shareholder() -> ScorpK1 {
    let mut source = complete_ordinary_scorp(0, true);
    source.qbi_statement = None;
    source.loans.clear();
    source.opening = Some(0);
    source.ordinary = 900_000;
    source.rental_facts.participation = Some(K1RentalParticipation::Active);
    source.prior_loss = 600_000;
    source.carried.real_estate.basis = 200_000;
    source.carried.charity = 200_000;
    source.risk.prior_loss = 225_000;
    source.carried.real_estate.risk = 75_000;
    source.risk.deducted = 0;
    source.risk.start = ScorpRiskStart::Detailed { capacity: 500_000, profit: 0, loss: 500_000 };
    source.amt = Some(amt_records(&source));
    source
}

#[test]
fn fixture_2b_shareholder_releases_basis_and_at_risk_carries_together() {
    let source = fixture_2b_shareholder();
    let ledger = source.ledger().unwrap();
    let classes = |ordinary, rental, charity| {
        amounts(&[
            (K1LossClass::Ordinary, ordinary),
            (K1LossClass::RentalRealEstate, rental),
            (K1LossClass::Charity, charity),
        ])
    };
    assert_eq!(ledger.basis_allowed, classes(5_400, 1_800, 1_800));
    assert_eq!(ledger.basis_suspended, classes(600, 200, 200));
    assert_eq!(ledger.risk_allowed, classes(6_750, 2_250, 1_800));
    assert_eq!(ledger.risk_suspended, classes(900, 300, 0));
    let (d, id) = shareholder_draft_of(source);
    let r = supported(&d);
    assert_eq!(value(&r, &format!("scorp_{id}_rental_real_estate_net")), -2_250);
    assert_eq!(return_line(&r, "11a / 11b"), 6_000_000);
    assert_eq!(return_line(&r, "12f"), 100_000);
    assert_eq!(return_line(&r, "15"), 4_290_000);
}

// The release order across the three limits: a class amount reaches the
// at-risk stage only to the extent basis allowed it, and the passive stage
// only to the extent the at-risk stage allowed it. A limited partner's box 2
// loss of 30,000 meets a 20,000 basis, a 12,000 amount at risk and no passive
// income: 20,000 / 12,000 / 0 pass, and the three carries are 10,000 (basis),
// 8,000 (at risk) and 12,000 (Form 8582).
#[test]
fn the_three_limits_release_in_order_basis_then_at_risk_then_passive() {
    let mut partner = member(0);
    partner.participation = Some(PartnerParticipation::Limited {
        minutes: 0,
        prior_ten: 0,
        personal_service: false,
        service_years: 0,
    });
    partner.regular.rentals.real_estate = -3_000_000;
    partner.rental_facts.participation = Some(crate::K1RentalParticipation::NotActive);
    start(&mut partner, 20_000, 12_000, 0);
    mirror_amt(&mut partner);
    let pool = partner.pool(&partner.regular).unwrap();
    let ledger = pool.activity.ledger;
    assert_eq!(dollars(ledger.basis_allowed[K1LossClass::RentalRealEstate].get()), 20_000);
    assert_eq!(dollars(ledger.basis_suspended[K1LossClass::RentalRealEstate].get()), 10_000);
    assert_eq!(dollars(ledger.risk_allowed[K1LossClass::RentalRealEstate].get()), 12_000);
    assert_eq!(dollars(ledger.risk_suspended[K1LossClass::RentalRealEstate].get()), 8_000);
    let (d, id) = draft_of(partner);
    let r = supported(&d);
    assert_eq!(value(&r, &format!("partnership_{id}_rental_real_estate_net")), 0);
    assert_eq!(value(&r, &format!("partnership_{id}_rental_real_estate_suspended")), 12_000);
    assert_eq!(return_line(&r, "11a / 11b"), 6_000_000);
    assert!(noted(
        &r,
        "rental real estate loss carried to 2027 — $10000 because of your basis, $8000 because of the at-risk limits"
    ));
}

// A rental carry of the earlier stage joins the later one in the order the
// map gives: the at-risk carry already passed basis and is never limited by
// basis again.
#[test]
fn an_at_risk_carry_is_never_limited_by_basis_again() {
    let mut partner = member(0);
    partner.regular.carried.real_estate.risk = 800_000;
    partner.regular.rentals.real_estate = 1_000_000;
    // Basis 0: the 10,000 of rental income raises it; the carry is at risk
    // only, so the whole basis is free for the (zero) basis classes.
    start(&mut partner, 0, 8_000, 8_000);
    mirror_amt(&mut partner);
    let ledger = partner.ledger(&partner.regular).unwrap();
    assert_eq!(dollars(ledger.basis_allowed[K1LossClass::RentalRealEstate].get()), 0);
    assert_eq!(dollars(ledger.risk_allowed[K1LossClass::RentalRealEstate].get()), 8_000);
    assert_eq!(ledger.risk_suspended, LossAmounts::default());
}

// The Form 6198 history is unknown: a basis-limited separate class carries
// without it (improvement row 241), while any at-risk carry needs the 2025
// Form 6198.
#[test]
fn an_unknown_at_risk_history_needs_nothing_for_a_basis_carry_and_the_form_for_a_risk_carry() {
    let mut partner = member(5_000);
    partner.separate = Some(separate(0, -2_000, 0, 1_000, 0));
    partner.regular.basis.opening = Some(100_000);
    partner.regular.risk.start = ScorpRiskStart::Unknown;
    mirror_amt(&mut partner);
    let ledger = partner.ledger(&partner.regular).unwrap();
    // 1,000 + 5,000 = 6,000 of basis against 3,000 of classes: nothing held.
    assert!(!ledger.limited());
    // 1,000 of basis against box 8 and a box 13 gift: carried, and the
    // activity income (box 1, 5,000) absorbs the allowed loss, so no history.
    partner.regular.basis.opening = Some(100_000);
    partner.regular.basis.contributions = 0;
    partner.regular.carried.charity = 500_000;
    mirror_amt(&mut partner);
    let ledger = partner.ledger(&partner.regular).unwrap();
    assert_eq!(ledger.risk_suspended, LossAmounts::default());
    assert!(dollars(ledger.basis_suspended[K1LossClass::Charity].get()) != 0);
    // One dollar carried at risk needs the history.
    partner.regular.carried.real_estate.risk = 100;
    mirror_amt(&mut partner);
    assert_eq!(
        partner.ledger(&partner.regular).unwrap_err().0,
        "ISSUE_PARTNER_AT_RISK_HISTORY_NEEDED"
    );
    let mut source = fixture_2b_shareholder();
    source.risk.start = ScorpRiskStart::Unknown;
    source.risk.prior_loss = 0;
    source.carried.real_estate.risk = 100;
    source.amt = Some(amt_records(&source));
    assert_eq!(source.ledger().unwrap_err().0, "ISSUE_SCORP_RISK_HISTORY_NEEDED");
}

// A new activity cannot carry anything.
#[test]
fn a_new_activity_with_any_carry_is_refused() {
    let mut partner = member(1_000);
    partner.regular.carried.charity = 100;
    start(&mut partner, 0, 0, 0);
    partner.regular.risk.start = ScorpRiskStart::New;
    assert_eq!(
        partner.ledger(&partner.regular).unwrap_err().0,
        "ISSUE_PARTNERSHIP_A_NEW_ACTIVITY_CANNOT_CONTAIN_PRIOR_822BC4EF"
    );
    let mut source = fixture_2b_shareholder();
    source.risk.start = ScorpRiskStart::New;
    assert!(!source.risk.valid(&source));
    let mut clean = fixture_2a_shareholder();
    clean.opening = Some(0);
    clean.ordinary = 100_000;
    clean.rentals.real_estate = 0;
    clean.cash_charity = 0;
    clean.risk.start = ScorpRiskStart::New;
    assert!(clean.risk.valid(&clean));
    clean.carried.foreign_taxes = 100;
    assert!(!clean.risk.valid(&clean));
}

// Other carried tax characters still need their own PTP qualified-source worksheet.
#[test]
fn a_publicly_traded_partnership_with_other_carried_characters_is_refused_by_name() {
    let mut partner = fixture_1b_partner();
    // A released section 179 carry is refused by its own PTP reason; the
    // other classes meet the QBI statement's.
    partner.regular.carried.section179 = 0;
    partner.regular.basis.prior_loss = 0;
    partner.publicly_traded = true;
    partner.qbi = Some(partner_qbi_statement(10_000));
    mirror_amt(&mut partner);
    let (d, _) = draft_of(partner);
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    assert!(
        refused_with(
            &r,
            "ISSUE_PARTNERSHIP_QBI_A_PUBLICLY_TRADED_PARTNERSHIP_INTEREST_WITH_77211554"
        ),
        "{:?}",
        r.issues
    );
}

// A different AMT basis that allows another section 179 or cash gift amount
// is refused by name only when the two systems differ: the App has no complete
// AMT projection for this class yet. The carries stay per system.
#[test]
fn a_different_amt_limit_for_a_gift_or_section_179_is_refused_only_when_it_differs() {
    let mut partner = member(0);
    partner.separate = Some(separate(0, 0, 0, 2_000, 0));
    start(&mut partner, 1_000, 1_000, 0);
    mirror_amt(&mut partner);
    let (d, _) = draft_of(partner.clone());
    let r = supported(&d);
    assert!(noted(&r, "cash contributions carried to 2027 — $1000"));
    // The AMT record has 500 of basis: its gift is 500, not 1,000.
    let alternative = partner.alternative.as_mut().unwrap();
    alternative.basis.opening = Some(50_000);
    let (d, _) = draft_of(partner);
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(refused_with(&r, "ISSUE_K1_AMT_LIMIT_DIFFERS"));
}

// Corpus check: every fixture of the K-1 test files that is still refused is
// refused for a reason other than the retired "does not calculate the
// carryforward" refusals, which no longer exist.
#[test]
fn the_retired_carryforward_refusals_are_gone_from_the_source_and_the_strings() {
    let retired = [
        "ISSUE_PARTNER_SEPARATE_ITEMS_BASIS_OR_RISK_LIMIT",
        "ISSUE_PARTNER_SECTION179_BASIS_LIMIT",
        "ISSUE_SCORP_CHARITY_BASIS_LIMIT",
        "ISSUE_SCORP_SECTION179_BASIS_LIMIT",
        "ISSUE_SCORP_FOREIGN_TAX_BASIS_LIMIT",
        "ISSUE_K1_RENTAL_LIMITED",
    ];
    for key in retired {
        assert!(APP_STRINGS.iter().all(|(k, _)| *k != key), "{key} is still declared");
    }
    // No fixture of the test files still expects a retired refusal.
    let mut dirs = alloc::vec![std::path::PathBuf::from("src/tests")];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs") || path.ends_with("k1_loss.rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            for key in retired {
                assert!(!text.contains(key), "{} still expects {key}", path.display());
            }
            for name in [
                "CHARITY_LIMITED",
                "SECTION179_LIMITED",
                "FOREIGN_TAX_LIMITED",
                "PartnerSeparateItems::LIMITED",
                "CHARITY_INDEX",
                "SECTION179_INDEX",
            ] {
                assert!(!text.contains(name), "{} still names {name}", path.display());
            }
        }
    }
    // The index constants and parallel arrays are gone from the engines.
    for (name, text) in [
        ("partnership.rs", include_str!("../../passthrough/partnership.rs")),
        ("scorp.rs", include_str!("../../passthrough/scorp/mod.rs")),
        ("scorp_capital.rs", include_str!("../../passthrough/scorp_capital.rs")),
        ("scorp_risk.rs", include_str!("../../passthrough/scorp_risk.rs")),
        ("scorp_1231.rs", include_str!("../../passthrough/scorp_1231.rs")),
        ("scorp_qbi.rs", include_str!("../../passthrough/scorp_qbi.rs")),
        ("k1_rental.rs", include_str!("../../passthrough/k1_rental.rs")),
        ("schedule_k3.rs", include_str!("../../passthrough/schedule_k3.rs")),
    ] {
        for retired in [
            "CHARITY_INDEX",
            "SECTION179_INDEX",
            "FOREIGN_TAX_INDEX",
            "RENTAL_INDEX",
            "BASIS_INDEX",
            "FOREIGN_TAX_CLASS",
            "rental_classes",
            ".resize(",
            "RentalLimited",
            "SECTION179_LIMITED",
            "CHARITY_LIMITED",
            "FOREIGN_TAX_LIMITED",
            "crate::tests::passthrough::k1_rental::LIMITED",
            "limited_this_year",
        ] {
            // The coverage blocks may name a retired constant in history.
            let code: alloc::string::String = text
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
                .collect::<alloc::vec::Vec<_>>()
                .join("\n");
            assert!(!code.contains(retired), "{name} still names {retired}");
        }
    }
}
