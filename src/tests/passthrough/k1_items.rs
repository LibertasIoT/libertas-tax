// K-1 box items (unit B9-1): the five fixtures of the k1-box-items map by
// hand, every code's destination, the order in which a sale of the whole
// interest releases the suspended losses, and the catalogs. Every amount is
// synthetic. Single filer with $60,000 wages unless a fixture says otherwise;
// 2026 standard deduction $16,100, QBI threshold $201,750, net investment
// income tax threshold $200,000.
use super::*;
extern crate std;
use crate::k1_items::K1ItemKind;
use crate::k1_loss::{K1LossClass, LossAmounts};
use crate::{
    K1Disposition, PartnerDeduction, PartnerDeductionCode, PartnerIncome, PartnerIncomeCode,
    PartnerQbiStatement, PartnerSpecial, ScorpDeduction, ScorpDeductionCode, ScorpIncome,
    ScorpIncomeCode, ScorpSpecial,
};

fn cents(dollars: i64) -> i64 {
    dollars * 100
}

fn section179_sale() -> crate::K1Section179Sale {
    k1_reported_sale(
        "Packaging line",
        20230301,
        [cents(30_000), cents(50_000), cents(8_000), cents(20_000)],
    )
}

#[test]
fn k1_section179_sale_f2_source_worksheet_restores_unused_carryover_to_basis() {
    let sale = section179_sale();
    let untouched = sale.clone();
    let values = crate::K1Section179Sale::worksheets(
        core::slice::from_ref(&sale),
        cents(2_000),
        cents(2_000),
    )
    .unwrap();
    assert_eq!(
        values,
        vec![crate::k1_items::Section179SaleResult {
            depreciation: 26_000,
            line4: 24_000,
            gain: 6_000,
            held_long: true,
            recapture: 6_000,
            section1231: 0,
            ordinary: 6_000,
        }]
    );
    assert_eq!(sale, untouched, "a worksheet must preserve accepted facts");
    let without_carry = sale.worksheet(0);
    assert_eq!(
        (without_carry.line4, without_carry.gain, without_carry.recapture),
        (22_000, 8_000, 8_000)
    );
}

#[test]
fn k1_section179_sale_f2b_long_held_loss_retains_section1231_character() {
    let mut sale = section179_sale();
    sale.regular.as_mut().unwrap().price = cents(20_000);
    let v = sale.worksheet(cents(2_000));
    assert_eq!(
        (v.line4, v.gain, v.recapture, v.section1231, v.ordinary),
        (24_000, -4_000, 0, -4_000, 0)
    );
}

#[test]
fn k1_section179_sale_f2c_multiple_properties_never_guess_carryover_allocation() {
    let sales = vec![section179_sale(), section179_sale()];
    assert!(crate::K1Section179Sale::valid_list(&sales));
    assert!(crate::K1Section179Sale::carryover_rule(&sales, cents(2_000), cents(2_000)).is_none());
    let error =
        crate::K1Section179Sale::worksheets(&sales, cents(2_000), cents(2_000)).unwrap_err();
    assert_eq!(error.0, "ISSUE_K1_SALE_CARRYOVER_SPLIT");
    let zero = crate::K1Section179Sale::worksheets(&sales, cents(2_000), 0).unwrap();
    assert_eq!(zero.len(), 2);
    assert_eq!(zero.iter().map(|v| v.ordinary).sum::<i64>(), 16_000);
}

#[test]
fn k1_section179_sale_f2d_carryover_relationships_use_exact_rule_keys() {
    let sales = vec![section179_sale()];
    for (carry, sold, expected) in [
        (2_000, 2_500, "K1_SALE_CARRYOVER_ABOVE_CARRYOVER"),
        (30_000, 20_001, "K1_SALE_CARRYOVER_ABOVE_SHARE"),
    ] {
        assert_eq!(
            crate::K1Section179Sale::carryover_rule(&sales, cents(carry), cents(sold)),
            Some(expected)
        );
        assert_eq!(
            crate::K1Section179Sale::worksheets(&sales, cents(carry), cents(sold)).unwrap_err().0,
            expected
        );
    }
    assert_eq!(
        crate::K1Section179Sale::carryover_rule(&[], cents(2_000), cents(1)),
        Some("K1_SALE_CARRYOVER_ABOVE_SHARE")
    );
    assert_eq!(
        crate::K1Section179Sale::worksheets(&sales, cents(20_000), cents(20_000)).unwrap()[0]
            .depreciation,
        8_000
    );
}

#[test]
fn k1_section179_sale_gain_above_recapture_preserves_capital_candidate() {
    let mut sale = section179_sale();
    sale.regular.as_mut().unwrap().price = cents(60_000);
    let v = sale.worksheet(cents(2_000));
    assert_eq!((v.gain, v.recapture, v.section1231, v.ordinary), (36_000, 26_000, 10_000, 26_000));
    assert_eq!(v.gain, v.ordinary + v.section1231);
}

#[test]
fn k1_section179_sale_strict_anniversary_and_same_year_are_ordinary() {
    let mut sale = section179_sale();
    for (acquired, long) in
        [(20250629, true), (20250630, false), (20250701, false), (20260630, false)]
    {
        sale.acquired = acquired;
        for (price, gain) in [(30_000, 6_000), (20_000, -4_000)] {
            sale.regular.as_mut().unwrap().price = cents(price);
            let v = sale.worksheet(cents(2_000));
            assert_eq!(v.held_long, long);
            assert_eq!(v.gain, gain);
            assert_eq!(v.ordinary, if long { gain.max(0) } else { gain });
            assert_eq!(v.section1231, if long { gain.min(0) } else { 0 });
        }
    }
}

#[test]
fn k1_section179_sale_source_validation_covers_dates_unicode_and_money_bounds() {
    let base = section179_sale();
    let valid = |sale: crate::K1Section179Sale| crate::K1Section179Sale::valid_list(&[sale]);
    assert!(valid(base.clone()));
    assert!(crate::K1Section179Sale::valid_list(&[]));
    assert!(crate::K1Section179Sale::valid_list(&vec![base.clone(); 20]));
    assert!(!crate::K1Section179Sale::valid_list(&vec![base.clone(); 21]));
    let mut missing = base.clone();
    missing.description.clear();
    assert!(!valid(missing));
    for name in ["é".repeat(40) + "a", "😀".repeat(100)] {
        let mut sale = base.clone();
        sale.description = name;
        assert!(valid(sale));
    }
    let mut sale = base.clone();
    sale.description = "é".repeat(40);
    assert!(valid(sale));
    for date in [20250229, 20261301, 0, u32::MAX] {
        let mut sale = base.clone();
        sale.acquired = date;
        assert!(!valid(sale));
    }
    for date in [20251231, 20260229, 20270101] {
        let mut sale = base.clone();
        sale.sold = date;
        assert!(!valid(sale));
    }
    let mut sale = base.clone();
    sale.acquired = 20260701;
    assert_eq!(sale.issue().map(|issue| issue.key), Some("K1_SALE_DATES"));
    assert!(!valid(sale));
    let mut sale = base.clone();
    sale.regular.as_mut().unwrap().depreciation = cents(30_001);
    assert_eq!(sale.issue().map(|issue| issue.key), Some("K1_SALE_BASIS_BELOW_ZERO"));
    assert!(!valid(sale));
    for amount in [-1, 100_000_000_001, i64::MIN, i64::MAX] {
        for field in 0..4 {
            let mut sale = base.clone();
            match field {
                0 => sale.regular.as_mut().unwrap().price = amount,
                1 => sale.regular.as_mut().unwrap().cost = amount,
                2 => sale.regular.as_mut().unwrap().depreciation = amount,
                _ => sale.regular.as_mut().unwrap().section179 = amount,
            }
            assert!(!valid(sale));
        }
    }
    let mut sale = base;
    sale.regular.as_mut().unwrap().depreciation = i64::MAX;
    sale.regular.as_mut().unwrap().section179 = i64::MAX;
    assert_eq!(sale.issue().map(|issue| issue.key), Some("K1_SALE_BASIS_BELOW_ZERO"));
}

#[test]
fn k1_section179_sale_rounds_each_worksheet_input_once_and_keeps_amt_independent() {
    let mut regular = section179_sale();
    regular.regular.as_mut().unwrap().price += 49;
    regular.regular.as_mut().unwrap().cost += 50;
    regular.regular.as_mut().unwrap().depreciation += 50;
    regular.regular.as_mut().unwrap().section179 += 49;
    let mut amt = regular.clone();
    amt.regular.as_mut().unwrap().depreciation = cents(6_000);
    let regular_value = regular.worksheet(cents(2_000) + 50);
    let amt_value = amt.worksheet(cents(2_000) + 50);
    assert_eq!(
        (regular_value.depreciation, regular_value.line4, regular_value.gain),
        (26_000, 24_001, 5_999)
    );
    assert_eq!((amt_value.depreciation, amt_value.line4, amt_value.gain), (23_999, 26_002, 3_998));
    assert_eq!(regular.regular.as_ref().unwrap().depreciation, cents(8_000) + 50);
}

#[test]
fn k1_section179_sale_invalid_personal_amounts_do_not_reach_worksheet() {
    let sales = vec![section179_sale()];
    for (carry, sold) in [(-1, 0), (0, -1), (100_000_000_001, 0), (0, i64::MAX)] {
        assert_eq!(
            crate::K1Section179Sale::worksheets(&sales, carry, sold).unwrap_err().0,
            "ISSUE_K1_SALE_RECORDS"
        );
    }
    let mut broken = section179_sale();
    broken.regular.as_mut().unwrap().cost = 0;
    assert_eq!(
        crate::K1Section179Sale::worksheets(&[broken], 0, 0).unwrap_err().0,
        "ISSUE_K1_SALE_RECORDS"
    );
}

fn gain(code: PartnerIncomeCode, dollars: i64) -> PartnerIncome {
    PartnerIncome { code, amount: cents(dollars) }
}

fn deduction(code: PartnerDeductionCode, dollars: i64) -> PartnerDeduction {
    PartnerDeduction { code, amount: cents(dollars) }
}

fn shareholder_gain(code: ScorpIncomeCode, dollars: i64) -> ScorpIncome {
    ScorpIncome { code, amount: cents(dollars) }
}

fn shareholder_deduction(code: ScorpDeductionCode, dollars: i64) -> ScorpDeduction {
    ScorpDeduction { code, amount: cents(dollars) }
}

// A limited partner: passive, not self-employed, box 14 code A zero.
fn limited(profit: i64) -> PartnerK1 {
    let mut partner = ordinary_partner_source(profit, false);
    partner.participation = PartnerParticipation::Limited {
        minutes: 0,
        prior_ten: 0,
        personal_service: false,
        service_years: 0,
    };
    partner
}

// A materially participating member: nonpassive.
fn member(profit: i64) -> PartnerK1 {
    ordinary_partner_source(profit, true)
}

// The opening outside basis and the detailed Form 6198 history of both tax
// systems, whole dollars.
fn start(partner: &mut PartnerK1, opening: i64, capacity: i64) {
    for record in core::iter::once(&mut partner.regular).chain(partner.alternative.as_mut()) {
        record.basis.opening = cents(opening);
        record.risk.start =
            ScorpRiskStart::Detailed { capacity: cents(capacity), profit: 0, loss: 0 };
    }
}

fn mirror(partner: &mut PartnerK1) {
    partner.alternative = Some(partner.regular.clone());
}

fn wages(d: &mut FederalDraft, amount: i64) {
    d.income[0] = wage(1, Owner::Taxpayer, cents(amount), cents(amount) / 10);
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
    add(&mut d, income(FederalIncome::Scorp { data: source }));
    (d, id)
}

fn refused_with(r: &FederalResult, key: &str) -> bool {
    r.state == ResultState::Unsupported && r.issues.iter().any(|i| i.display.key == key)
}

fn row(id: i64, code: &str) -> std::string::String {
    alloc::format!("partnership_{id}_item_{code}")
}

fn has_line(r: &FederalResult, id: &str) -> bool {
    r.lines.iter().any(|l| l.id == id)
}

fn totals_of(partner: &PartnerK1) -> crate::federal_rules::Totals {
    let entry = income(FederalIncome::Partnership { data: partner.clone() });
    let mut t = crate::federal_rules::Totals::default();
    let mut scratch = estimate(&complete(FederalStatus::Single));
    crate::k1_totals::apply(&partner.totals_facts(&entry), &mut t, &mut scratch);
    t
}

fn shareholder_totals_of(source: &ScorpK1) -> crate::federal_rules::Totals {
    let entry = income(FederalIncome::Scorp { data: source.clone() });
    let mut t = crate::federal_rules::Totals::default();
    let mut scratch = estimate(&complete(FederalStatus::Single));
    crate::k1_totals::apply(&source.totals_facts(&entry), &mut t, &mut scratch);
    t
}

// A shareholder with ample stock basis and amount at risk, no loans.
fn shareholder(material: bool) -> ScorpK1 {
    let mut source = complete_ordinary_scorp(0, material);
    source.loans.clear();
    source.opening = cents(100_000);
    source.risk.start = ScorpRiskStart::Detailed { capacity: cents(100_000), profit: 0, loss: 0 };
    source.risk.prior_loss = 0;
    source.risk.deducted = 0;
    source.qbi_statement = None;
    source.amt = Some(amt_records(&source));
    source
}

fn english(key: &str) -> &'static str {
    APP_STRINGS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, text)| *text)
        .unwrap_or_else(|| panic!("undeclared {key}"))
}

fn notices_of(d: &FederalDraft) -> alloc::vec::Vec<crate::federal_model::ReviewNote> {
    let mut notes = alloc::vec::Vec::new();
    crate::unknowns::supported(d, &mut notes);
    notes
}

// ---- The catalogs -------------------------------------------------------

// `ALL` is the one catalog of each code enum. The list bounds of the page and
// the stored record equal its size, every code has a unique printed row, a
// left-out code has its notice text, and the sign rule names exactly the codes
// whose amount can be a loss.
#[test]
fn the_code_catalogs_follow_their_texts() {
    use crate::{PartnerDeductionCode as PD, PartnerIncomeCode as PI};
    use crate::{ScorpDeductionCode as SD, ScorpIncomeCode as SI};
    let page = include_str!("../../passthrough/k1_page.rs");
    let partnership = include_str!("../../passthrough/partnership.rs");
    let scorp = include_str!("../../passthrough/scorp/mod.rs");
    let count = |source: &str, sentence: &str| source.matches(sentence).count();
    for (name, size) in [
        ("income: Vec<crate::PartnerIncome>", PI::ALL.len()),
        ("deductions: Vec<crate::PartnerDeduction>", PD::ALL.len()),
        ("income: Vec<crate::ScorpIncome>", SI::ALL.len()),
        ("deductions: Vec<crate::ScorpDeduction>", SD::ALL.len()),
    ] {
        let page_bound = alloc::format!("libertas_size(max = {size})]\n    pub {name}");
        let stored_bound = page_bound.replace(": Vec<", ": alloc::vec::Vec<");
        let found = count(page, &page_bound)
            + count(partnership, &stored_bound)
            + count(scorp, &page_bound);
        // The page and the stored record each declare the list once.
        assert_eq!(found, 2, "{name} with a bound of {size}");
    }
    assert_eq!((PI::ALL.len(), PD::ALL.len(), SI::ALL.len(), SD::ALL.len()), (12, 25, 7, 20));
    assert_eq!(PartnerSpecial::ALL.len(), 5);
    assert_eq!(ScorpSpecial::ALL.len(), 3);
    assert_eq!(count(page, "libertas_size(max = 5)]\n    pub special"), 1);
    assert_eq!(count(page, "libertas_size(max = 3)]\n    pub special"), 1);
    let resource = |key: &str| APP_STRINGS.iter().any(|(k, _)| *k == key);
    for code in PI::ALL {
        assert_eq!(code.may_be_loss(), code.class() == Some(K1LossClass::OtherLoss), "{code:?}");
        if code.kind() == K1ItemKind::LeftOut {
            assert!(resource(code.item()), "{}", code.item());
        }
    }
    for code in SI::ALL {
        if code.kind() == K1ItemKind::LeftOut {
            assert!(resource(code.item()), "{}", code.item());
        }
    }
    for code in PD::ALL {
        if code.kind() == K1ItemKind::LeftOut {
            assert!(resource(code.item()), "{}", code.item());
        }
    }
    for code in SD::ALL {
        if code.kind() == K1ItemKind::LeftOut {
            assert!(resource(code.item()), "{}", code.item());
        }
    }
    // The rows of one box are unique.
    for rows in [
        PI::ALL.map(|c| c.row()).to_vec(),
        PD::ALL.map(|c| c.row()).to_vec(),
        SI::ALL.map(|c| c.row()).to_vec(),
        SD::ALL.map(|c| c.row()).to_vec(),
    ] {
        let mut sorted = rows.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), rows.len());
    }
    // The page rule's code indices are the enum positions of the codes that
    // cannot be a loss (the client rule names them by index).
    let partner_indexes: alloc::vec::Vec<_> = PI::ALL
        .iter()
        .enumerate()
        .filter(|(_, code)| !code.may_be_loss())
        .map(|(index, _)| index)
        .collect();
    assert_eq!(partner_indexes, [0, 2, 3, 4, 5, 6]);
    let scorp_indexes: alloc::vec::Vec<_> = SI::ALL
        .iter()
        .enumerate()
        .filter(|(_, code)| !code.may_be_loss())
        .map(|(index, _)| index)
        .collect();
    assert_eq!(scorp_indexes, [0, 2, 3, 4]);
    // The class catalog carries the four classes.
    for class in [
        K1LossClass::OtherLoss,
        K1LossClass::OtherCharity,
        K1LossClass::InvestmentInterest,
        K1LossClass::OtherDeductions,
    ] {
        assert!(K1LossClass::ALL.contains(&class));
        assert!(resource(class.key()));
    }
    assert!(!K1LossClass::OtherCharity.at_risk());
    assert!(K1LossClass::OtherLoss.at_risk());
    assert!(K1LossClass::InvestmentInterest.at_risk());
    assert!(K1LossClass::OtherDeductions.at_risk());
}

// The removed refusals are named nowhere any longer, in the sources or in
// the resource table.
#[test]
fn no_source_names_a_removed_refusal() {
    let removed = [
        "ISSUE_PARTNER_K1_CREDITS",
        "ISSUE_SCORP_K1_CREDITS",
        "ISSUE_PARTNER_K1_ROYALTIES",
        "ISSUE_SCORP_K1_ROYALTIES",
        "ISSUE_SCORP_K1_OTHER_DEDUCTIONS",
        "ISSUE_PARTNERSHIP_SCHEDULE_K_1__1065__BOX_19_CODE_AEAC5F97",
        "ISSUE_SCORP_PASSIVE_DISTRIBUTION_GAIN",
        "ISSUE_SCORP_DISTRIBUTION_STOCK_HOLDING_PERIOD",
        "ISSUE_SCHEDULE_K_1_1120_S_BOX_16_CODE_D_DISTRIBUTIONS_ABOVE_STOCK_BASIS_W_63EB75A5",
    ];
    for key in removed {
        assert!(APP_STRINGS.iter().all(|(k, _)| *k != key), "{key}");
    }
    let mut directories = alloc::vec![std::path::PathBuf::from("src")];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                directories.push(path);
            } else if path.extension().is_some_and(|e| e == "rs")
                && !path.ends_with("tests/passthrough/k1_items.rs")
            {
                let text = std::fs::read_to_string(&path).unwrap();
                for key in removed {
                    assert!(!text.contains(key), "{} names {key}", path.display());
                }
            }
        }
    }
}

// ---- Fixture 1: box 11 code A computed; code I refused ------------------

#[test]
fn fixture_1_other_portfolio_income_reaches_line_8z_and_is_not_qbi() {
    let mut partner = limited(10_000);
    partner.qbi = Some(partner_qbi_statement(10_000));
    partner.income = vec![gain(PartnerIncomeCode::PortfolioIncome, 2_000)];
    start(&mut partner, 30_000, 30_000);
    mirror(&mut partner);
    // Basis: line 4 increases 12,000 (box 1 10,000 and box 11 code A 2,000).
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 42_000);
    let (d, id) = draft_of(partner);
    let r = supported(&d);
    // Schedule E line 28 column (h) 10,000; Schedule 1 line 8z 2,000.
    assert_eq!(value(&r, &row(id, "11A")), 2_000);
    // AGI 60,000 + 10,000 + 2,000; before QBI 72,000 - 16,100 = 55,900.
    assert_eq!(return_line(&r, "11a / 11b"), 7_200_000);
    // QBI deduction min(20% x 10,000, 20% x 55,900) = 2,000: code A is not QBI.
    assert_eq!(return_line(&r, "15"), 5_390_000);
    // The stored return reopens to the same estimate.
    assert_eq!(estimate(&restart(d.clone()).draft), r);
}

#[test]
fn fixture_1b_a_code_i_amount_is_refused_by_name_and_1c_a_code_a_loss_is_rejected() {
    let mut partner = limited(10_000);
    partner.qbi = Some(partner_qbi_statement(10_000));
    partner.income = vec![gain(PartnerIncomeCode::PortfolioIncome, 2_000)];
    partner.unentered = vec![crate::PartnerK1Item::OtherIncome];
    start(&mut partner, 30_000, 30_000);
    mirror(&mut partner);
    let (d, _) = draft_of(partner.clone());
    assert!(refused_with(&estimate(&d), "ISSUE_PARTNER_K1_OTHER_INCOME"));
    // Code A as -500: the sign rule rejects the list on both sides.
    partner.income = vec![gain(PartnerIncomeCode::PortfolioIncome, -500)];
    assert!(!partner.valid());
    assert_eq!(PartnerIncome::rule(&partner.income), Some("K1_INCOME_CODE_SIGN"));
    // A code listed twice and an amount of zero are rejected too.
    let twice = vec![
        gain(PartnerIncomeCode::PortfolioIncome, 5),
        gain(PartnerIncomeCode::PortfolioIncome, 6),
    ];
    assert_eq!(PartnerIncome::rule(&twice), Some("K1_INCOME_CODE_TWICE"));
    assert_eq!(
        PartnerIncome::rule(&[gain(PartnerIncomeCode::DebtCancellation, 0)]),
        Some("K1_INCOME_CODE_SIGN")
    );
    // A loss is allowed for the codes that can be one.
    assert_eq!(PartnerIncome::rule(&[gain(PartnerIncomeCode::Section1256, -5)]), None);
    let deductions = vec![
        deduction(PartnerDeductionCode::CashThirty, 5),
        deduction(PartnerDeductionCode::CashThirty, 6),
    ];
    assert_eq!(PartnerDeduction::rule(&deductions), Some("K1_DEDUCTION_CODE_TWICE"));
}

// ---- Fixture 2: box 13 code C at the 50% limit; code F left out ---------

// Wages $60,000; state and local tax $3,000 (a personal property tax here:
// the wage entry carries no state withholding) and main-home interest
// $14,000. 1065 K-1: limited partner, box 1 0, opening basis $50,000,
// box 13 code C $32,000 and code F $1,000.
#[test]
fn fixture_2_noncash_gift_at_the_fifty_percent_limit_and_a_twenty_percent_gift_left_out() {
    let mut partner = limited(0);
    partner.deductions = vec![
        deduction(PartnerDeductionCode::NoncashFifty, 32_000),
        deduction(PartnerDeductionCode::CapitalGainPropertyTwenty, 1_000),
    ];
    start(&mut partner, 50_000, 50_000);
    mirror(&mut partner);
    // Basis: OtherCharity 33,000 <= 50,000, all allowed; ending 17,000. The
    // class is outside Form 6198.
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 17_000);
    assert_eq!(
        crate::kernel::ratio::dollars(
            pool.activity.ledger.basis_allowed[K1LossClass::OtherCharity].get()
        ),
        33_000
    );
    assert_eq!(pool.activity.ledger.basis_suspended, LossAmounts::default());
    let (mut d, id) = draft_of(partner);
    {
        let v = d.deductions.as_mut().unwrap();
        itemized(v).personal_property_taxes = 300_000;
        fixture_mortgage_interest(v, 1_400_000);
    }
    let r = supported(&d);
    // AGI 60,000. (A) class claim 32,000; ceiling 50% x 60,000 = 30,000;
    // floor min(30,000, 0.5% x 60,000) = 300 charged to (A): line 12 29,700.
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 29_700);
    // 2027 (A)-class carryover 32,000 - 30,000 = 2,000 plus the floor 300
    // because the 50% limit was exceeded (section 170(d)(1)(C)) = 2,300.
    assert_eq!(value(&r, "charity_property_fifty_return_year_carryforward"), 2_300);
    // Itemized 3,000 + 14,000 + 29,700 = 46,700 > 16,100.
    assert_eq!(value(&r, "deduction"), 46_700);
    assert_eq!(return_line(&r, "15"), 1_330_000);
    // Code F is left out with its notice; no amount of it reaches Schedule A.
    assert!(r.review_notes.iter().any(|n| {
        n.display.key == "WARN_K1_ITEM_LEFT_OUT"
            && n.contains("capital gain property contributions (20%)")
    }));
    assert!(value(&r, &row(id, "13C")) == 32_000);
    assert!(!has_line(&r, &row(id, "13F")));
    // The stored return reopens to the same estimate.
    assert_eq!(estimate(&restart(d.clone()).draft), r);
}

// ---- Fixture 3: section 199A dividends and PTP items -------------------

fn fixture_3(reit: i64, ptp: i64) -> FederalDraft {
    let mut partner = member(40_000);
    partner.qbi = Some(PartnerQbiStatement {
        wages: cents(30_000),
        reit_dividends: cents(reit),
        ptp_income: cents(ptp),
        ..partner_qbi_statement(40_000)
    });
    partner.separate = Some(crate::PartnerSeparateItems {
        interest: 0,
        dividends: cents(1_000),
        qualified_dividends: 0,
        royalties: 0,
        short: 0,
        long: 0,
        collectibles: 0,
        unrecaptured_partnership: 0,
        unrecaptured_fund: 0,
        cash_charity: 0,
        backup_withholding: 0,
        exempt_interest: 0,
        foreign_taxes: 0,
    });
    start(&mut partner, 10_000, 10_000);
    mirror(&mut partner);
    let (mut d, _) = draft_of(partner);
    wages(&mut d, 150_000);
    d
}

#[test]
fn fixture_3_section_199a_dividends_and_ptp_items_join_the_reit_ptp_component() {
    // AGI 150,000 + 40,000 + 1,000 = 191,000; before QBI 174,900 <= 201,750.
    // Line 5 (QBI component) 20% x 40,000 = 8,000; line 6 1,000, line 9
    // 200; line 10 8,200; line 14 20% x 174,900 = 34,980: deduction 8,200;
    // taxable income 166,700. NII: MAGI 191,000 is below the threshold.
    let r = supported(&fixture_3(1_000, 0));
    assert_eq!(return_line(&r, "11a / 11b"), 19_100_000);
    assert_eq!(return_line(&r, "15"), 16_670_000);
    // Without the statement's dividends: 8,000 and 166,900.
    let r = supported(&fixture_3(0, 0));
    assert_eq!(return_line(&r, "15"), 16_690_000);
    // F3b: qualified PTP items -800: line 6 = 1,000 - 800 = 200, line 9 =
    // 40, deduction 8,040, taxable income 166,860.
    let r = supported(&fixture_3(1_000, -800));
    assert_eq!(return_line(&r, "15"), 16_686_000);
}

#[test]
fn fixture_3c_a_cooperative_statement_is_refused_by_name() {
    let mut d = fixture_3(1_000, 0);
    let FederalIncome::Partnership { data } = &mut d.income[1].income else { panic!() };
    data.unentered = vec![crate::PartnerK1Item::CooperativeQbi];
    assert!(refused_with(&estimate(&d), "ISSUE_K1_COOPERATIVE_QBI"));
}

// ---- Fixture 4: sale of the interest -----------------------------------

pub(in crate::tests) fn fixture_4() -> PartnerK1 {
    let mut partner = limited(-4_000);
    partner.final_k1 = true;
    partner.regular.basis.opening_liabilities = cents(20_000);
    partner.regular.basis.ending_liabilities = cents(20_000);
    start(&mut partner, 45_000, 25_000);
    partner.regular.passive_loss = cents(10_000);
    partner.qbi = Some(partner_qbi_statement(-4_000));
    partner.qbi_records.history = Some(crate::partnership_qbi::PartnerQbiHistory {
        basis: vec![],
        risk: vec![],
        passive: vec![crate::ScorpLossYear {
            year: 2025,
            original: cents(10_000),
            qualified: cents(10_000),
            remaining: cents(10_000),
            remaining_qualified: cents(10_000),
        }],
    });
    partner.sale = crate::PartnerSaleParts {
        ordinary: cents(8_000),
        collectibles: 0,
        unrecaptured: cents(3_000),
    };
    partner.disposition = K1Disposition::Sold { proceeds: cents(60_000) };
    partner.interest_term = Some(GainTerm::Long);
    mirror(&mut partner);
    partner
}

// Wages $90,000. Basis 45,000 + (20,000 - 20,000); the box 1 loss 4,000 is
// allowed, ending basis 41,000; at risk 25,000 >= 4,000. Amount realized
// 60,000 + 20,000 = 80,000; gain 80,000 - 41,000 = 39,000: section 751
// 8,000 on Form 4797 Part II line 10 and Schedule 1 line 4, capital
// 39,000 - 8,000 = 31,000 long-term on Form 8949 box F (proceeds 72,000,
// basis 41,000) with 3,000 of unrecaptured section 1250 gain. Section
// 469(g)(1)(A): the activity's loss 4,000 + 10,000 is nonpassive (Schedule E
// -4,000 current and -10,000 prior). AGI 90,000 + 8,000 + 31,000 - 14,000 =
// 115,000; QBI 8,000 - 4,000 - 10,000 = -6,000: no deduction and a 2027
// qualified business loss of 6,000; taxable 115,000 - 16,100 = 98,900.
#[test]
fn fixture_4_the_sale_releases_the_suspended_loss_and_splits_the_gain() {
    let partner = fixture_4();
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 41_000);
    // Form 6198: 25,000 + the gain 39,000 (line 2a) - the loss 4,000.
    assert_eq!(pool.activity.risk_end.map(crate::kernel::ratio::dollars), Some(60_000));
    let disposal = pool.activity.disposal;
    assert_eq!(
        (
            crate::kernel::ratio::dollars(disposal.ordinary),
            crate::kernel::ratio::dollars(disposal.proceeds),
            crate::kernel::ratio::dollars(disposal.basis)
        ),
        (8_000, 72_000, 41_000)
    );
    assert_eq!(crate::kernel::ratio::dollars(disposal.capital()), 31_000);
    assert_eq!(crate::kernel::ratio::dollars(disposal.sale_gain()), 39_000);
    // The old loss is released with the sale; nothing stays passive.
    assert_eq!(
        (
            crate::kernel::ratio::dollars(pool.passive),
            crate::kernel::ratio::dollars(pool.nonpassive)
        ),
        (0, -14_000)
    );
    assert_eq!((pool.prior_released, pool.prior_pool), (1_000_000, 0));
    let (mut d, id) = draft_of(partner);
    wages(&mut d, 90_000);
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("partnership_{id}_sale_ordinary")), 8_000);
    assert_eq!(value(&r, &alloc::format!("partnership_{id}_sale_gain")), 31_000);
    assert_eq!(value(&r, "k1_ordinary_gains"), 8_000);
    assert_eq!(return_line(&r, "11a / 11b"), 11_500_000);
    assert_eq!(return_line(&r, "15"), 9_890_000);
    // Net capital gain 31,000 (Schedule D line 16 / Form 1040 line 7a).
    assert_eq!(return_line(&r, "7a"), 3_100_000);
    // No other passive activity has a loss: no warning; no suspended loss
    // remains to end.
    assert!(!noted(&r, "WARN_K1_GAIN_NOT_PASSIVE"));
    assert!(!noted(&r, "NOTE_K1_SUSPENDED_LOSS_ENDS"));
    // QBI: no deduction; the 2027 carryover is the qualified loss of 6,000.
    assert_eq!(value(&r, "qbi_deduction"), 0);
    assert_eq!(estimate(&restart(d.clone()).draft), r);
}

// The unrecaptured section 1250 and collectibles parts are the sale's, held
// more than one year only (Reg. 1.1(h)-1).
#[test]
fn fixture_4_sale_codes_reach_the_gain_worksheets_when_held_more_than_a_year() {
    use crate::federal_rules::Totals;
    let partner = fixture_4();
    let (d, _) = draft_of(partner.clone());
    let base = estimate(&d);
    let entry = income(FederalIncome::Partnership { data: partner.clone() });
    let mut t = Totals::default();
    let mut r = base.clone();
    crate::k1_totals::apply(&partner.totals_facts(&entry), &mut t, &mut r);
    assert_eq!(t.long, cents(31_000));
    assert_eq!(t.short, 0);
    assert_eq!(t.sale_line_10, cents(31_000));
    assert_eq!(t.sale_line_3, 0);
    assert_eq!(t.unrecaptured_long, cents(3_000));
    assert_eq!(t.collectible_long, 0);
    assert_eq!(t.k1_ordinary, [cents(8_000), cents(8_000)]);
    // Held one year or less: short-term Form 8949 box C and the 28% and
    // unrecaptured section 1250 parts are ignored.
    let mut short = partner;
    short.interest_term = Some(GainTerm::Short);
    short.sale.collectibles = cents(500);
    let mut t = Totals::default();
    crate::k1_totals::apply(&short.totals_facts(&entry), &mut t, &mut r);
    assert_eq!((t.short, t.long), (cents(31_000), 0));
    assert_eq!((t.sale_line_3, t.sale_line_10), (cents(31_000), 0));
    assert_eq!((t.unrecaptured_long, t.collectible_long), (0, 0));
}

#[test]
fn fixture_4b_to_4d_other_dispositions_and_the_page_rules() {
    // F4b: an installment sale (Other) is refused by name.
    let mut partner = fixture_4();
    partner.disposition = K1Disposition::Other;
    partner.sale = Default::default();
    let (d, _) = draft_of(partner);
    assert!(refused_with(&estimate(&d), "ISSUE_K1_DISPOSITION_OTHER"));
}

// ---- Fixture 5: box 19 code A above basis, passive, with another loss ----

#[test]
fn fixture_5_cash_above_basis_is_section_731_gain_with_the_not_passive_warning() {
    let build = |wage_amount: i64, participates: bool| {
        let mut p = if participates { member(2_000) } else { limited(2_000) };
        p.regular.basis.distributions = cents(30_000);
        p.interest_term = Some(GainTerm::Long);
        start(&mut p, 23_000, 23_000);
        mirror(&mut p);
        let mut q = limited(-3_000);
        start(&mut q, 100_000, 100_000);
        mirror(&mut q);
        let mut d = complete(FederalStatus::Single);
        wages(&mut d, wage_amount);
        add(&mut d, income(FederalIncome::Partnership { data: p }));
        add(&mut d, income(FederalIncome::Partnership { data: q }));
        d
    };
    let d = build(80_000, false);
    let r = supported(&d);
    // P: 23,000 + 2,000 = 25,000 before 30,000 of money: gain 5,000, long
    // term; Q: loss 3,000 against 2,000 of passive income: 1,000 unallowed.
    // AGI 80,000 + 2,000 - 2,000 + 5,000.
    assert_eq!(return_line(&r, "11a / 11b"), 8_500_000);
    assert_eq!(return_line(&r, "15"), 6_890_000);
    assert_eq!(return_line(&r, "7a"), 500_000);
    assert!(noted(&r, "WARN_K1_GAIN_NOT_PASSIVE"));
    // F5b: a materially participating partner in a business that is not
    // trading, with modified AGI above the threshold: refused by name.
    assert!(refused_with(&estimate(&build(250_000, true)), "ISSUE_K1_GAIN_NIIT_ADJUSTMENT"));
    let calculated = supported(&build(150_000, true));
    assert_eq!(return_line(&calculated, "7a"), 500_000);
}

// ---- Every code's destination -------------------------------------------

#[test]
fn every_partner_income_code_reaches_its_destination() {
    use PartnerIncomeCode as C;
    let ten = cents(1_000);
    // A materially participating member (a nonpassive business that is not
    // trading) and a limited partner (passive).
    let totals = |code: C, dollars: i64, passive: bool| {
        let mut p = if passive { limited(0) } else { member(0) };
        p.income = vec![gain(code, dollars)];
        start(&mut p, 100_000, 100_000);
        mirror(&mut p);
        (totals_of(&p), p)
    };
    // A: Schedule 1 line 8z, Form 8960 line 7 and Form 4952 line 4a.
    let (t, p) = totals(C::PortfolioIncome, 1_000, false);
    assert_eq!((t.other_8z, t.k1_line_7, t.k1_investment_income), (ten, ten, ten));
    // The basis worksheet line 4 and Form 6198 take the income.
    assert_eq!(
        crate::kernel::ratio::dollars(p.pool(&p.regular).unwrap().activity.basis.ending.get()),
        101_000
    );
    // C: one Form 6781 Part I line 1 row, a loss included.
    for amount in [1_000, -1_000] {
        let (_, p) = totals(C::Section1256, amount, false);
        let (d, _) = draft_of(p);
        assert_eq!(crate::section1256::form_6781(&d).net, amount);
    }
    // E: Schedule 1 line 8c; net investment income only for a passive or
    // trading activity.
    let (t, _) = totals(C::DebtCancellation, 1_000, false);
    assert_eq!((t.personal_debt, t.k1_line_7), (ten, 0));
    let (t, _) = totals(C::DebtCancellation, 1_000, true);
    assert_eq!((t.personal_debt, t.k1_line_7), (ten, ten));
    // F: part of box 1.
    let (_, p) = totals(C::BasisAdjustmentIncome, 1_000, false);
    assert_eq!(
        crate::kernel::ratio::dollars(p.pool(&p.regular).unwrap().activity.income.get()),
        1_000
    );
    // J: Schedule 1 line 8z and line 7 (the recovery is taxed in full).
    let (t, _) = totals(C::TaxBenefitRecovery, 1_000, false);
    assert_eq!((t.other_8z, t.k1_line_7), (ten, ten));
    // K: winnings Schedule 1 line 8b, losses Schedule A with the W-2G losses.
    let (t, _) = totals(C::GamblingWinnings, 1_000, false);
    assert_eq!((t.gambling, t.gambling_losses, t.k1_line_7), (ten, 0, 0));
    let (t, _) = totals(C::GamblingLosses, 1_000, false);
    assert_eq!((t.gambling, t.gambling_losses), (0, ten));
    // Q and S: Schedule D lines 5 and 12; S is an item of the activity.
    let (t, _) = totals(C::PreferredStockShort, 1_000, false);
    assert_eq!((t.short, t.long, t.investment_gains), (ten, 0, [ten, 0]));
    let (t, _) = totals(C::PreferredStockLong, 1_000, false);
    assert_eq!((t.short, t.long, t.investment_gains), (0, ten, [0, ten]));
    let (t, _) = totals(C::NonportfolioShort, 1_000, false);
    assert_eq!((t.short, t.investment_gains, t.k1_nii_excluded), (ten, [0, 0], -ten));
    let (t, _) = totals(C::NonportfolioLong, 1_000, true);
    assert_eq!((t.long, t.k1_nii_excluded), (ten, 0));
    // R: Form 4797 Part II line 10 and Schedule 1 line 4.
    let (t, _) = totals(C::SpeciallyAllocatedOrdinary, 1_000, false);
    assert_eq!((t.k1_ordinary[0], t.k1_nii_excluded), (ten, -ten));
    // A loss of a code that can be one passes the basis and joins its form.
    let (t, _) = totals(C::SpeciallyAllocatedOrdinary, -1_000, false);
    assert_eq!(t.k1_ordinary[0], -ten);
    let (t, _) = totals(C::NonportfolioLong, -1_000, false);
    assert_eq!(t.long, -ten);
}

#[test]
fn every_partner_deduction_code_reaches_its_destination() {
    use PartnerDeductionCode as D;
    let ten = cents(1_000);
    let mut left_out = 0;
    for code in D::ALL {
        let mut p = member(0);
        p.deductions = vec![deduction(code, 1_000)];
        start(&mut p, 100_000, 100_000);
        mirror(&mut p);
        // Every code reduces the outside basis by its amount, whether or not
        // the App deducts it (worksheet lines 15j through 15p).
        let pool = p.pool(&p.regular).unwrap();
        assert_eq!(
            crate::kernel::ratio::dollars(pool.activity.basis.ending.get()),
            99_000,
            "{code:?}"
        );
        let t = totals_of(&p);
        let expected = match code {
            D::CashThirty | D::NoncashThirty => [ten, 0, 0],
            D::NoncashFifty => [0, ten, 0],
            D::CapitalGainPropertyThirty => [0, 0, ten],
            _ => [0; 3],
        };
        assert_eq!(t.k1_charity, expected, "{code:?}");
        assert_eq!(t.k1_interest, if code == D::InvestmentInterest { ten } else { 0 });
        assert_eq!(t.early_penalty, if code == D::EarlyWithdrawalPenalty { ten } else { 0 });
        // Code AE is no deduction (section 67(h)); K reduces basis only.
        if matches!(code, D::PortfolioDeductions | D::ExcessBusinessInterest) {
            assert_eq!(t.k1_charity, [0; 3]);
        }
        let (d, _) = draft_of(p);
        let notes = notices_of(&d);
        let noticed = notes
            .iter()
            .any(|n| n.display.key == "WARN_K1_ITEM_LEFT_OUT" && n.contains(english(code.item())));
        assert_eq!(noticed, code.kind() == K1ItemKind::LeftOut, "{code:?}");
        left_out += usize::from(noticed);
    }
    // F, G, J, L, N, P, S, W, X, Y, Z, AA, AC, AD and ZZ.
    assert_eq!(left_out, 15);
    // Code V reduces box 1; investment interest is Form 4952 line 1.
    let mut p = member(0);
    p.deductions = vec![deduction(D::BasisAdjustmentDeduction, 1_000)];
    start(&mut p, 100_000, 100_000);
    let pool = p.pool(&p.regular).unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(
            pool.activity.ledger.basis_allowed[K1LossClass::Ordinary].get()
        ),
        1_000
    );
}

#[test]
fn the_shareholder_codes_reach_their_destinations_and_leave_out_their_benefits() {
    use ScorpDeductionCode as D;
    use ScorpIncomeCode as C;
    let ten = cents(1_000);
    let totals = |code: C, dollars: i64, material: bool| {
        let mut s = shareholder(material);
        s.income = vec![shareholder_gain(code, dollars)];
        s.amt = Some(amt_records(&s));
        shareholder_totals_of(&s)
    };
    let t = totals(C::PortfolioIncome, 1_000, true);
    assert_eq!((t.other_8z, t.k1_line_7, t.k1_investment_income), (ten, ten, ten));
    let t = totals(C::TaxBenefitRecovery, 1_000, true);
    assert_eq!((t.other_8z, t.k1_line_7), (ten, ten));
    let t = totals(C::GamblingWinnings, 1_000, true);
    assert_eq!(t.gambling, ten);
    let t = totals(C::GamblingLosses, 1_000, true);
    assert_eq!(t.gambling_losses, ten);
    let t = totals(C::NonportfolioShort, 1_000, true);
    assert_eq!((t.short, t.k1_nii_excluded), (ten, -ten));
    let t = totals(C::NonportfolioLong, -1_000, false);
    assert_eq!((t.long, t.k1_nii_excluded), (-ten, 0));
    let mut left_out = 0;
    for code in D::ALL {
        let mut s = shareholder(true);
        s.deductions = vec![shareholder_deduction(code, 1_000)];
        s.amt = Some(amt_records(&s));
        let basis = s.basis_result().unwrap();
        assert_eq!(crate::kernel::ratio::dollars(basis.stock.get()), 99_000, "{code:?}");
        let t = shareholder_totals_of(&s);
        let expected = match code {
            D::CashThirty | D::NoncashThirty => [ten, 0, 0],
            D::NoncashFifty => [0, ten, 0],
            D::CapitalGainPropertyThirty => [0, 0, ten],
            _ => [0; 3],
        };
        assert_eq!(t.k1_charity, expected, "{code:?}");
        assert_eq!(t.k1_interest, if code == D::InvestmentInterest { ten } else { 0 });
        assert_eq!(t.early_penalty, if code == D::EarlyWithdrawalPenalty { ten } else { 0 });
        let (d, _) = shareholder_draft_of(s);
        let noticed = notices_of(&d)
            .iter()
            .any(|n| n.display.key == "WARN_K1_ITEM_LEFT_OUT" && n.contains(english(code.item())));
        assert_eq!(noticed, code.kind() == K1ItemKind::LeftOut, "{code:?}");
        left_out += usize::from(noticed);
    }
    // F, G, J, L, M, O, W, X, Y, Z, AA, AC and ZZ.
    assert_eq!(left_out, 13);
}

// The item-only situations keep the K-1 and leave their item out; a
// whole-K-1 situation leaves the K-1 out.
#[test]
fn the_situations_of_something_else_applies_leave_out_an_item_or_the_whole_k1() {
    let mut p = member(5_000);
    p.credits = vec![];
    start(&mut p, 100_000, 100_000);
    mirror(&mut p);
    for (situation, item) in [
        (PartnerSpecial::UnreimbursedExpenses, "K1_ITEM_UNREIMBURSED_EXPENSES"),
        (PartnerSpecial::HousingBefore1990, "K1_ITEM_HOUSING_BEFORE_1990"),
        (PartnerSpecial::ExcessInterestCarried, "K1_ITEM_EXCESS_INTEREST_CARRIED"),
    ] {
        let mut q = p.clone();
        q.special = vec![situation];
        assert!(q.complete());
        let (d, _) = draft_of(q);
        assert!(
            notices_of(&d)
                .iter()
                .any(|n| { n.display.key == "WARN_K1_ITEM_LEFT_OUT" && n.contains(english(item)) })
        );
        // The K-1 stays in the estimate.
        let r = supported(&d);
        assert_eq!(return_line(&r, "8"), 500_000);
    }
    for situation in [PartnerSpecial::PropertyContributed, PartnerSpecial::RetiringPayments] {
        let mut q = p.clone();
        q.special = vec![situation];
        assert!(!q.complete());
        let (d, _) = draft_of(q);
        assert!(noted(&supported(&d), "WARN_PARTNERSHIP_K1_LEFT_OUT"));
    }
    // Box 20 code AQ is a notice too.
    let mut q = p.clone();
    q.crp_payments = cents(300);
    let (d, _) = draft_of(q);
    assert!(notices_of(&d).iter().any(|n| n.contains(english("K1_ITEM_CRP_PAYMENTS"))));
    // The shareholder: farming leaves the K-1 out, the pre-1990 credit its item.
    let mut s = shareholder(true);
    s.special = vec![ScorpSpecial::HousingBefore1990];
    assert!(s.basis_complete());
    s.special = vec![ScorpSpecial::Farming];
    assert!(!s.basis_complete());
}

// ---- The sale releases the suspended losses in order --------------------

// Basis, then at risk, then passive. Opening basis 2,000 against a basis carry
// of 3,000 ordinary and 1,000 short-term capital loss: 1,500 and 500 allowed,
// 1,500 and 500 held back for good once the interest is sold; the sale
// (proceeds 10,000, ending basis 0) is a 10,000 long-term gain, which is Form
// 6198 income and releases the at-risk carry of 600; the 6,000 passive loss of
// the activity is nonpassive. AGI 60,000 - (1,500 + 600) - 6,000 - 500 + 10,000
// = 61,400.
#[test]
fn a_sale_releases_the_losses_basis_then_at_risk_then_passive() {
    let mut p = limited(0);
    start(&mut p, 2_000, 0);
    p.regular.basis.prior_loss = cents(3_000);
    p.regular.capital_carried.short.basis = cents(1_000);
    p.regular.risk.prior_loss = cents(600);
    p.regular.passive_loss = cents(6_000);
    p.disposition = K1Disposition::Sold { proceeds: cents(10_000) };
    p.interest_term = Some(GainTerm::Long);
    p.final_k1 = true;
    mirror(&mut p);
    let pool = p.pool(&p.regular).unwrap();
    let ledger = pool.activity.ledger;
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.basis_allowed[K1LossClass::Ordinary].get()),
        1_500
    );
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.basis_allowed[K1LossClass::ShortTerm].get()),
        500
    );
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.basis_suspended[K1LossClass::Ordinary].get()),
        1_500
    );
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.basis_suspended[K1LossClass::ShortTerm].get()),
        500
    );
    // The gain frees the at-risk carry; nothing is held back at risk.
    assert_eq!(ledger.risk_suspended, LossAmounts::default());
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.risk_allowed[K1LossClass::Ordinary].get()),
        2_100
    );
    // The passive loss is released by the sale.
    assert_eq!(
        (
            crate::kernel::ratio::dollars(pool.passive),
            crate::kernel::ratio::dollars(pool.prior_released)
        ),
        (0, 6_000)
    );
    assert_eq!(crate::kernel::ratio::dollars(pool.nonpassive), -8_100);
    let (d, _) = draft_of(p);
    let r = supported(&d);
    assert_eq!(return_line(&r, "11a / 11b"), 6_140_000);
    // What basis held back ends with the interest.
    assert!(noted(&r, "NOTE_K1_SUSPENDED_LOSS_ENDS"));
    assert!(!r.review_notes.iter().any(|n| n.contains("carried to 2027")));
}

#[test]
fn a_shareholder_sale_is_a_gain_over_the_ending_stock_basis_and_other_dispositions_are_refused() {
    let mut s = shareholder(true);
    s.opening = cents(4_000);
    s.disposition = K1Disposition::Sold { proceeds: cents(20_000) };
    s.stock_term = Some(GainTerm::Long);
    s.final_k1 = true;
    s.amt = Some(amt_records(&s));
    let basis = s.basis_result().unwrap();
    let disposal = s.disposal(&basis);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(disposal.proceeds),
            crate::kernel::ratio::dollars(disposal.basis),
            crate::kernel::ratio::dollars(disposal.capital())
        ),
        (20_000, 4_000, 16_000)
    );
    let (d, id) = shareholder_draft_of(s.clone());
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("scorp_{id}_sale_gain")), 16_000);
    assert_eq!(return_line(&r, "7a"), 1_600_000);
    let mut other = s;
    other.disposition = K1Disposition::Other;
    let (d, _) = shareholder_draft_of(other);
    assert!(refused_with(&estimate(&d), "ISSUE_K1_DISPOSITION_OTHER"));
}

// Investment interest with a loss the at-risk limit holds back needs Form
// 4952 and Form 6198 each after the other: refused by name.
#[test]
fn investment_interest_with_an_at_risk_limited_loss_is_refused_by_name() {
    let mut p = member(-5_000);
    p.deductions = vec![deduction(PartnerDeductionCode::InvestmentInterest, 1_000)];
    start(&mut p, 100_000, 1_000);
    mirror(&mut p);
    let (d, _) = draft_of(p.clone());
    assert!(refused_with(&estimate(&d), "ISSUE_K1_INVESTMENT_INTEREST_AT_RISK"));
    // Without the interest the same K-1 is limited but not refused for it.
    p.deductions.clear();
    let (d, _) = draft_of(p);
    assert!(!refused_with(&estimate(&d), "ISSUE_K1_INVESTMENT_INTEREST_AT_RISK"));
}

// Unit B9-7 R7: i1065sk1 (2025 final), box 7 and box 13 code I;
// i1120ssk (2025 final), box 6 and box 12 code I. Schedule E (2026 draft,
// created May 6, 2026), lines 4, 19–26. These independent figures exercise
// the portfolio path before the remaining F5 depletion-notice work.
fn royalty_partner(gross: i64, expense: i64) -> PartnerK1 {
    let mut p = limited(0);
    p.separate = Some(crate::PartnerSeparateItems {
        interest: 0,
        dividends: 0,
        qualified_dividends: 0,
        royalties: cents(gross),
        short: 0,
        long: 0,
        collectibles: 0,
        unrecaptured_partnership: 0,
        unrecaptured_fund: 0,
        cash_charity: 0,
        backup_withholding: 0,
        exempt_interest: 0,
        foreign_taxes: 0,
    });
    p.deductions = vec![deduction(PartnerDeductionCode::RoyaltyDeductions, expense)];
    start(&mut p, 0, 0);
    mirror(&mut p);
    p
}

fn royalty_form(r: &FederalResult, form: TaxForm, line: &str) -> i64 {
    r.lines
        .iter()
        .filter(|row| row.form_line.as_ref() == Some(&FormLine::new(form, line)))
        .map(|row| row.amount / 100)
        .sum()
}

#[test]
fn k1_royalty_partner_receipts_raise_basis_and_risk_without_passive_qbi_or_se_income() {
    let p = royalty_partner(8_000, 1_200);
    let pool = p.pool(&p.regular).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 6_800);
    assert_eq!(pool.activity.risk_end.map(crate::kernel::ratio::dollars), Some(6_800));
    assert_eq!(
        crate::kernel::ratio::dollars(
            pool.activity.ledger.risk_suspended[K1LossClass::OtherDeductions].get()
        ),
        0
    );
    let (mut d, id) = draft_of(p);
    wages(&mut d, 50_000);
    let r = supported(&d);
    for (suffix, amount) in [
        ("line4_royalties", 8_000),
        ("line19_royalty_deductions", 1_200),
        ("line20_royalty_expenses", 1_200),
        ("line21_royalty_income", 6_800),
    ] {
        assert_eq!(value(&r, &alloc::format!("partnership_{id}_{suffix}")), amount);
    }
    assert_eq!(return_line(&r, "11a / 11b"), cents(56_800));
    assert_eq!(value(&r, "schedule_e_line23b_royalties"), 8_000);
    assert_eq!(value(&r, "schedule_e_line24_income"), 6_800);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), 6_800);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
    assert!(!r.lines.iter().any(|row| {
        row.form_line
            .as_ref()
            .is_some_and(|f| matches!(f.form, TaxForm::Form8582 | TaxForm::ScheduleSe))
            && row.amount != 0
    }));
    assert_eq!(estimate(&restart(d.clone()).draft), r);
    wages(&mut d, 250_000);
    let high = supported(&d);
    assert_eq!(value(&high, "net_investment_income_tax"), 258);
    assert_eq!(royalty_form(&high, TaxForm::Form8960, "4a"), 6_800);
}

#[test]
fn k1_royalty_shareholder_uses_the_same_portfolio_path_for_both_participation_answers() {
    for material in [false, true] {
        let mut s = shareholder(material);
        s.opening = 0;
        s.risk.start = ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 };
        s.royalties = cents(8_000);
        s.deductions = vec![shareholder_deduction(ScorpDeductionCode::RoyaltyDeductions, 1_200)];
        assert_eq!(crate::kernel::ratio::dollars(s.basis_result().unwrap().stock.get()), 6_800);
        let (mut d, id) = shareholder_draft_of(s);
        wages(&mut d, 50_000);
        let r = supported(&d);
        assert_eq!(value(&r, &alloc::format!("scorp_{id}_line21_royalty_income")), 6_800);
        assert_eq!(return_line(&r, "11a / 11b"), cents(56_800));
        assert_eq!(value(&r, "qbi_deduction"), 0);
        assert_eq!(estimate(&restart(d.clone()).draft), r);
        wages(&mut d, 250_000);
        let high = supported(&d);
        assert_eq!(value(&high, "net_investment_income_tax"), 258);
        assert_eq!(royalty_form(&high, TaxForm::Form8960, "4a"), 6_800);
    }
}

#[test]
fn k1_royalty_deductions_wait_for_basis_and_at_risk_limits_not_passive_limits() {
    for (opening, capacity, basis_held, risk_held) in [(0, 20_000, 4_000, 0), (10_000, 0, 0, 4_000)]
    {
        let mut p = royalty_partner(8_000, 12_000);
        start(&mut p, opening, capacity);
        mirror(&mut p);
        let pool = p.pool(&p.regular).unwrap();
        assert_eq!(
            crate::kernel::ratio::dollars(
                pool.activity.ledger.basis_suspended[K1LossClass::OtherDeductions].get()
            ),
            basis_held
        );
        assert_eq!(
            crate::kernel::ratio::dollars(
                pool.activity.ledger.risk_suspended[K1LossClass::OtherDeductions].get()
            ),
            risk_held
        );
        let (mut d, id) = draft_of(p);
        wages(&mut d, 50_000);
        let r = supported(&d);
        assert_eq!(value(&r, &alloc::format!("partnership_{id}_line19_royalty_deductions")), 8_000);
        assert_eq!(return_line(&r, "11a / 11b"), cents(50_000));
        assert_eq!(value(&r, "schedule_e_rental_royalty_total"), 0);
    }
}

#[test]
fn k1_royalty_amt_uses_its_own_basis_limit_and_reaches_the_return_adjustment() {
    let mut p = royalty_partner(8_000, 12_000);
    start(&mut p, 0, 20_000);
    mirror(&mut p);
    p.alternative.as_mut().unwrap().basis.opening = cents(4_000);
    let t = totals_of(&p);
    assert_eq!(t.royalties, crate::kernel::ratio::exact::whole(0));
    assert_eq!(t.amt_royalty_adjustment, cents(-4_000));
    let (mut d, id) = draft_of(p);
    wages(&mut d, 50_000);
    let r = supported(&d);
    assert_eq!(
        value(&r, &alloc::format!("amt_partnership_{id}_line19_royalty_deductions")),
        12_000
    );
    assert_eq!(value(&r, "amt_rental_adjustment"), -4_000);
    assert_eq!(value(&r, "schedule_e_line25_royalty_losses"), 0);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), 0);
}

#[test]
fn k1_royalty_and_direct_royalty_columns_combine_once_in_part_one() {
    let mut p = royalty_partner(8_000, 12_000);
    start(&mut p, 10_000, 10_000);
    mirror(&mut p);
    let (mut d, _) = draft_of(p);
    wages(&mut d, 50_000);
    add(&mut d, income(royalty(2_000, 500)));
    let r = supported(&d);
    assert_eq!(royalty_form(&r, TaxForm::ScheduleE, "4"), 10_000);
    assert_eq!(royalty_form(&r, TaxForm::ScheduleE, "20"), 12_500);
    assert_eq!(royalty_form(&r, TaxForm::ScheduleE, "21"), -2_500);
    assert_eq!(value(&r, "schedule_e_line23b_royalties"), 10_000);
    assert_eq!(value(&r, "schedule_e_line23e_expenses"), 12_500);
    assert_eq!(value(&r, "schedule_e_line24_income"), 1_500);
    assert_eq!(value(&r, "schedule_e_line25_royalty_losses"), -4_000);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), -2_500);
    assert_eq!(
        royalty_form(&r, TaxForm::ScheduleE, "26"),
        royalty_form(&r, TaxForm::ScheduleE, "24") + royalty_form(&r, TaxForm::ScheduleE, "25")
    );
    assert_eq!(return_line(&r, "11a / 11b"), cents(47_500));
    assert_eq!(r.lines.iter().filter(|row| row.id == "schedule_e_line23b_royalties").count(), 1);
}

#[test]
fn k1_royalty_amt_exception_preserves_the_guard_for_other_deductions() {
    let mut p = royalty_partner(8_000, 12_000);
    p.deductions.push(deduction(PartnerDeductionCode::PortfolioOther, 12_000));
    start(&mut p, 0, 20_000);
    mirror(&mut p);
    p.alternative.as_mut().unwrap().basis.opening = cents(4_000);
    let (d, _) = draft_of(p);
    assert!(refused_with(&estimate(&d), "ISSUE_K1_AMT_LIMIT_DIFFERS"));

    let mut s = shareholder(false);
    s.royalties = cents(8_000);
    s.opening = 0;
    s.deductions = vec![
        shareholder_deduction(ScorpDeductionCode::RoyaltyDeductions, 12_000),
        shareholder_deduction(ScorpDeductionCode::PortfolioOther, 12_000),
    ];
    s.amt = Some(amt_records(&s));
    s.amt.as_mut().unwrap().stock = cents(4_000);
    let (mut d, _) = shareholder_draft_of(s.clone());
    let FederalIncome::Scorp { data } = &mut d.income[1].income else { unreachable!() };
    data.amt = s.amt;
    assert!(refused_with(&estimate(&d), "ISSUE_K1_AMT_LIMIT_DIFFERS"));
}

#[test]
fn k1_royalty_shareholder_amt_basis_changes_only_the_allowed_royalty_expense() {
    let mut s = shareholder(false);
    s.royalties = cents(8_000);
    s.opening = 0;
    s.deductions = vec![shareholder_deduction(ScorpDeductionCode::RoyaltyDeductions, 12_000)];
    let (mut d, id) = shareholder_draft_of(s);
    wages(&mut d, 50_000);
    let FederalIncome::Scorp { data } = &mut d.income[1].income else { unreachable!() };
    data.amt.as_mut().unwrap().stock = cents(4_000);
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("scorp_{id}_line19_royalty_deductions")), 8_000);
    assert_eq!(value(&r, &alloc::format!("amt_scorp_{id}_line19_royalty_deductions")), 12_000);
    assert_eq!(value(&r, "amt_rental_adjustment"), -4_000);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), 0);
}

#[test]
fn k1_royalty_form1116_uses_receipts_instead_of_net_income_for_gross_allocation() {
    let (mut d, _) = draft_of(royalty_partner(8_000, 1_200));
    wages(&mut d, 50_000);
    add(&mut d, income(foreign_pay(20_000)));
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Ordinary {
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::Unreviewed,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        amt_limitation: ForeignAmtLimitation::Normal,
        sources: vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Synthetic foreign employer".into(),
            category: ForeignTaxCategory::Employment,
            income: cents(20_000),
            tax: cents(2_000),
        }],
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment: ForeignRateAdjustment::Exception,
    };
    let r = supported(&d);
    assert_eq!(return_line(&r, "11a / 11b"), cents(76_800));
    let gross: Vec<_> =
        r.lines.iter().filter(|row| row.id.ends_with("_gross_income_all_sources")).collect();
    assert!(!gross.is_empty());
    for row in gross {
        assert_eq!(row.amount, cents(78_000), "{}", row.id);
    }
    // A K-3 with the same U.S. receipts replaces the K-1 gross once; it
    // must not add a second $8,000 to the allocation denominator.
    let mut k3 = crate::tests::foreign::foreign::k3_blank();
    k3.us_gross = cents(8_000);
    let FederalIncome::Partnership { data } = &mut d.income[1].income else { unreachable!() };
    data.schedule_k3 = Some(k3.clone());
    let with_k3 = supported(&d);
    assert!(with_k3.lines.iter().any(|row| row.id.ends_with("_gross_income_all_sources")));
    for row in with_k3.lines.iter().filter(|row| row.id.ends_with("_gross_income_all_sources")) {
        assert_eq!(row.amount, cents(78_000), "{}", row.id);
    }
    let mut s = shareholder(false);
    s.royalties = cents(8_000);
    s.deductions = vec![shareholder_deduction(ScorpDeductionCode::RoyaltyDeductions, 1_200)];
    s.schedule_k3 = Some(k3);
    s.amt = Some(amt_records(&s));
    d.income[1].income = FederalIncome::Scorp { data: s };
    for row in
        supported(&d).lines.iter().filter(|row| row.id.ends_with("_gross_income_all_sources"))
    {
        assert_eq!(row.amount, cents(78_000), "{}", row.id);
    }
}

// B9-7 F5/R11/R12. The K-1 instruction statements identify a deduction or
// credit, without adding a second receipt or reducing basis for unclaimed depletion.
#[test]
fn k1_benefit_notices_complete_f5_and_preserve_the_accepted_k1() {
    let mut p = royalty_partner(8_000, 1_200);
    p.unentered = vec![crate::PartnerK1Item::Depletion];
    let (mut d, _) = draft_of(p.clone());
    wages(&mut d, 50_000);
    let original = d.clone();
    let r = supported(&d);
    assert_eq!(return_line(&r, "11a / 11b"), cents(56_800));
    assert_eq!(value(&r, "schedule_e_line23b_royalties"), 8_000);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), 6_800);
    let label = english("K1_ITEM_PARTNER_DEPLETION");
    assert!(label.contains("box 20 code T"));
    assert_eq!(
        r.review_notes
            .iter()
            .filter(|n| n.display.key == "WARN_K1_ITEM_LEFT_OUT" && n.contains(label))
            .count(),
        1
    );
    assert_eq!(
        crate::kernel::ratio::dollars(p.pool(&p.regular).unwrap().activity.basis.ending.get()),
        6_800
    );
    assert_eq!(d, original);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_benefit_notices_keep_both_entity_kinds_and_name_each_omitted_item_once() {
    let mut p = member(10_000);
    p.unentered = vec![crate::PartnerK1Item::Credits, crate::PartnerK1Item::Depletion];
    let (d, _) = draft_of(p);
    let r = supported(&d);
    assert_eq!(return_line(&r, "11a / 11b"), cents(70_000));
    for key in ["K1_ITEM_PARTNER_CREDITS", "K1_ITEM_PARTNER_DEPLETION"] {
        assert_eq!(
            r.review_notes
                .iter()
                .filter(|n| n.display.key == "WARN_K1_ITEM_LEFT_OUT" && n.contains(english(key)))
                .count(),
            1
        );
    }
    let mut s = shareholder(true);
    s.ordinary = cents(10_000);
    s.unentered = vec![crate::ScorpK1Item::Credits, crate::ScorpK1Item::Depletion];
    let (d, _) = shareholder_draft_of(s);
    let r = supported(&d);
    assert_eq!(return_line(&r, "11a / 11b"), cents(70_000));
    for key in ["K1_ITEM_SCORP_CREDITS", "K1_ITEM_SCORP_DEPLETION"] {
        assert_eq!(
            r.review_notes
                .iter()
                .filter(|n| n.display.key == "WARN_K1_ITEM_LEFT_OUT" && n.contains(english(key)))
                .count(),
            1
        );
    }
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_benefit_notices_do_not_remove_code_ax_or_other_added_tax_refusals() {
    let mut p = member(10_000);
    p.unentered = vec![
        crate::PartnerK1Item::Credits,
        crate::PartnerK1Item::Depletion,
        crate::PartnerK1Item::CreditRecapture,
    ];
    let (d, _) = draft_of(p);
    assert!(refused_with(&estimate(&d), "ISSUE_K1_CREDIT_RECAPTURE"));
    let mut s = shareholder(true);
    s.unentered = vec![crate::ScorpK1Item::Depletion, crate::ScorpK1Item::CreditRecapture];
    let (d, _) = shareholder_draft_of(s);
    assert!(refused_with(&estimate(&d), "ISSUE_K1_CREDIT_RECAPTURE"));
    // Mineral-property dispositions and other recapture keep their existing refusal.
    for item in [crate::PartnerK1Item::OtherIncome, crate::PartnerK1Item::OtherInformation] {
        let mut p = member(10_000);
        p.unentered = vec![crate::PartnerK1Item::Depletion, item];
        let (d, _) = draft_of(p);
        assert!(refused_with(&estimate(&d), item.refusal_key()));
    }
}

#[test]
fn k1_benefit_depletion_notice_does_not_reduce_general_partner_se_or_basis() {
    let mut p = member(20_000);
    p.limited_for_se = false;
    p.reported_se = Some(cents(20_000));
    let expected_basis =
        crate::kernel::ratio::dollars(p.pool(&p.regular).unwrap().activity.basis.ending.get());
    let (d, _) = draft_of(p.clone());
    let without = supported(&d);
    p.unentered = vec![crate::PartnerK1Item::Depletion];
    assert_eq!(
        p.se_after_passive(
            crate::kernel::carry::Amount::new(0).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap()
        )
        .unwrap()
        .earnings,
        20_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(p.pool(&p.regular).unwrap().activity.basis.ending.get()),
        expected_basis
    );
    let (d, _) = draft_of(p);
    let with = supported(&d);
    assert_eq!(royalty_form(&with, TaxForm::ScheduleSe, "2"), 20_000);
    assert_eq!(value(&with, "self_employment_tax"), 2_826);
    assert_eq!(return_line(&with, "11a / 11b"), cents(78_587));
    assert_eq!(with.lines, without.lines);
}

fn partner_property(net: i64, unrecaptured: i64) -> crate::K1Section1231 {
    k1_section1231(net, unrecaptured)
}

fn partner_property_draft(net: i64) -> (FederalDraft, i64) {
    let mut partner = ordinary_partner_source(10_000, true);
    partner.limited_for_se = true;
    partner.reported_se = Some(0);
    partner.regular.section1231 = Some(partner_property(net, if net > 0 { 3_000 } else { 0 }));
    start(&mut partner, 100_000, 100_000);
    let mut qbi = partner_qbi_statement(10_000);
    qbi.section1231_qbi = cents(net);
    qbi.wages = cents(50_000);
    partner.qbi = Some(qbi);
    mirror(&mut partner);
    let (mut d, id) = draft_of(partner);
    wages(&mut d, 80_000);
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2024, 5_000, 5_000)], &[(2024, 5_000, 5_000)]));
    (d, id)
}

#[test]
fn k1_partner_property_f1_lookback_and_qbi_match_hand_amounts() {
    let (d, id) = partner_property_draft(12_000);
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{:?}", r.issues);
    for (name, amount) in [
        (alloc::format!("partnership_{id}_section1231"), 12_000),
        ("form4797_line8".into(), 12_000),
        ("form4797_line9".into(), 5_000),
        ("form4797_line10".into(), 7_000),
        ("form4797_line13".into(), 5_000),
    ] {
        assert_eq!(value(&r, &name), amount, "{name}");
    }
    assert_eq!(return_line(&r, "11a / 11b"), cents(102_000));
    assert_eq!(value(&r, "qbi_deduction"), 3_000);
    assert_eq!(return_line(&r, "15"), cents(82_900));
    let pool = crate::extended_rules::rentals(&d, 0, &mut estimate(&d));
    assert_eq!(pool.k1_amounts.iter().find(|v| v.id == id).unwrap().net, 10_000);
    let FederalIncome::Partnership { data } = &d.income.last().unwrap().income else { panic!() };
    let activity = data.regular.activity_result(&data.items()).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(activity.basis.ending.get()), 122_000);
}

#[test]
fn k1_partner_property_f1b_loss_keeps_ordinary_qbi_and_next_history() {
    let (d, id) = partner_property_draft(-4_000);
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{:?}", r.issues);
    assert_eq!(value(&r, &alloc::format!("partnership_{id}_section1231")), -4_000);
    assert_eq!(value(&r, "form4797_line12"), -4_000);
    assert_eq!(return_line(&r, "11a / 11b"), cents(86_000));
    assert_eq!(value(&r, "qbi_deduction"), 1_200);
    assert_eq!(value(&r, "section1231_current_loss_history"), 4_000);
    assert_eq!(value(&r, "section1231_2024_next_lookback"), 5_000);
}

#[test]
fn k1_partner_property_f1c_ptp_remains_named_refusal() {
    let (mut d, _) = partner_property_draft(12_000);
    let FederalIncome::Partnership { data } = &mut d.income.last_mut().unwrap().income else {
        panic!()
    };
    data.publicly_traded = true;
    data.qbi.as_mut().unwrap().wages = 0;
    let r = estimate(&d);
    assert!(refused_with(&r, "ISSUE_K1_PTP_PROPERTY"), "{:?}", r.issues);
}

#[test]
fn k1_partner_property_f1d_statement_bound_has_same_client_and_server_key() {
    let (d, _) = partner_property_draft(12_000);
    let FederalIncome::Partnership { data } = &d.income.last().unwrap().income else { panic!() };
    let mut page = crate::k1_page::K1Partnership::from_record("Synthetic partner", data).unwrap();
    assert_eq!(page.rule(), None);
    let crate::k1_page::PartnerQbi::Statement { section1231_qbi, .. } = &mut page.qbi else {
        panic!()
    };
    *section1231_qbi = cents(15_000);
    assert_eq!(page.rule(), Some("K1_SECTION1231_QBI_ABOVE_BOX"));
    let record = page.record();
    assert!(!record.valid());
    for (net, qualified, valid) in [
        (12_000, -1, false),
        (-4_000, 1, false),
        (-4_000, -4_001, false),
        (-4_000, -2_000, true),
        (0, 1, false),
        (0, 0, true),
    ] {
        page.more.section1231 = cents(net);
        let crate::k1_page::PartnerQbi::Statement { section1231_qbi, .. } = &mut page.qbi else {
            panic!()
        };
        *section1231_qbi = cents(qualified);
        assert_eq!(page.rule().is_none(), valid, "{net}/{qualified}");
        assert_eq!(page.record().valid(), valid, "{net}/{qualified}");
    }
}

#[test]
fn k1_partner_property_loss_shares_basis_then_risk_without_changing_character() {
    let mut p = member(-6_000);
    p.regular.section1231 = Some(partner_property(-6_000, 0));
    start(&mut p, 6_000, 3_000);
    let result = p.pool(&p.regular).unwrap();
    for class in [K1LossClass::Ordinary, K1LossClass::Section1231] {
        assert_eq!(
            crate::kernel::ratio::dollars(result.activity.ledger.basis_allowed[class].get()),
            3_000
        );
        assert_eq!(
            crate::kernel::ratio::dollars(result.activity.ledger.basis_suspended[class].get()),
            3_000
        );
        assert_eq!(
            crate::kernel::ratio::dollars(result.activity.ledger.risk_allowed[class].get()),
            1_500
        );
        assert_eq!(
            crate::kernel::ratio::dollars(result.activity.ledger.risk_suspended[class].get()),
            1_500
        );
    }
    let parts = result.characters.unwrap();
    assert_eq!(parts.nonpassive.map(crate::kernel::ratio::dollars), [-1_500, -1_500]);
    assert_eq!(crate::kernel::ratio::dollars(result.nonpassive), -1_500);
    assert_eq!(crate::kernel::ratio::dollars(result.activity.basis.ending.get()), 0);
    assert_eq!(result.activity.risk_end, Some(0));
}

#[test]
fn k1_partner_property_carries_release_in_their_own_class_and_current_listing() {
    let (mut d, id) = partner_property_draft(12_000);
    let FederalIncome::Partnership { data } = &mut d.income.last_mut().unwrap().income else {
        panic!()
    };
    start(data, 0, 0);
    let property = data.regular.section1231.as_mut().unwrap();
    property.basis_loss = cents(9_000);
    property.risk_loss = cents(2_000);
    property.passive_loss = cents(5_000);
    mirror(data);
    let page =
        crate::k1_page::K1Partnership::from_record("Synthetic retained property", data).unwrap();
    assert_eq!(page.record(), *data);
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("partnership_{id}_section1231")), -4_000);
    assert_eq!(return_line(&r, "11a / 11b"), cents(86_000));
    assert_eq!(value(&r, "qbi_deduction"), 1_200);
    let pool = crate::extended_rules::rentals(&d, 0, &mut estimate(&d));
    let ledger = &pool.k1_ledgers.iter().find(|v| v.0 == id).unwrap().1;
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.basis_allowed[K1LossClass::Section1231].get()),
        9_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.risk_allowed[K1LossClass::Section1231].get()),
        11_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.passive_suspended[K1LossClass::Section1231].get()),
        0
    );
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.passive_suspended[K1LossClass::Ordinary].get()),
        0
    );
}

#[test]
fn k1_partner_property_passive_loss_keeps_its_fraction_and_part_ix() {
    let (mut d, id) = partner_property_draft(-20_000);
    let FederalIncome::Partnership { data } = &mut d.income.last_mut().unwrap().income else {
        panic!()
    };
    let crate::PartnerParticipation::General { facts } = &mut data.participation else { panic!() };
    facts.minutes = 0;
    data.qbi.as_mut().unwrap().section1231_qbi = cents(-10_000);
    let mut s = shareholder(false);
    s.ordinary = cents(3_000);
    s.amt = Some(amt_records(&s));
    add(&mut d, income(FederalIncome::Scorp { data: s }));
    let r = supported(&d);
    assert_eq!(return_line(&r, "11a / 11b"), cents(80_000));
    assert_eq!(value(&r, &alloc::format!("partnership_{id}_section1231")), -13_000);
    assert_eq!(value(&r, &alloc::format!("form8582_part_ix_{id}_section1231_unallowed")), 7_000);
    assert_eq!(value(&r, &alloc::format!("form8582_part_ix_{id}_section1231_allowed")), 13_000);
    assert_eq!(value(&r, "qbi_deduction"), 700);
    let pool = crate::extended_rules::rentals(&d, 0, &mut estimate(&d));
    let ledger = &pool.k1_ledgers.iter().find(|v| v.0 == id).unwrap().1;
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.passive_suspended[K1LossClass::Section1231].get()),
        7_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.passive_suspended[K1LossClass::Ordinary].get()),
        0
    );
}

#[test]
fn common_pal_annual_item_sources_keep_the_native_character_residual() {
    let mut loss = limited(-1);
    loss.income = vec![
        gain(PartnerIncomeCode::NonportfolioShort, -1),
        gain(PartnerIncomeCode::NonportfolioLong, -5),
    ];
    start(&mut loss, 100, 100);
    mirror(&mut loss);
    let (mut d, id) = draft_of(loss);
    let mut profit = limited(3);
    start(&mut profit, 100, 100);
    mirror(&mut profit);
    add(&mut d, income(FederalIncome::Partnership { data: profit }));

    let result = supported(&d);
    let pool = crate::extended_rules::rentals(&d, 60_000, &mut result.clone());
    // Basis/risk allow all seven dollars of loss. Three dollars of other
    // passive income leave four held. Exact source weights [100,100,500]
    // retain the shared allocator's [57,57,286] cent residual recipe.
    assert_eq!(pool.passive_loss, 4);
    let precise = pool.post_pal.as_ref().unwrap();
    let source = precise
        .sources
        .iter()
        .find(|source| {
            source.income == id
                && source.component == crate::extended_rules::PostPalComponent::PartnershipBusiness
        })
        .unwrap();
    for (character, net, held) in [
        (crate::extended_rules::PostPalCharacter::ScheduleE, -100, 57),
        (crate::extended_rules::PostPalCharacter::ShortCapital, -100, 57),
        (crate::extended_rules::PostPalCharacter::LongCapital, -500, 286),
    ] {
        let amount = source.characters.iter().find(|amount| amount.character == character).unwrap();
        assert_eq!(
            (amount.net_cents, amount.suspended_cents, amount.allowed_net_cents),
            (net, held, net + held)
        );
    }
    let ledger = &pool.k1_ledgers.iter().find(|entry| entry.0 == id).unwrap().1;
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.passive_suspended[K1LossClass::Ordinary].get()),
        1
    );
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.passive_suspended[K1LossClass::OtherLoss].get()),
        3
    );
    assert_eq!(
        pool.k1_cuts
            .capital_by_id
            .iter()
            .find(|entry| entry.0 == id)
            .map(|(id, cuts)| (*id, cuts.map(|cents| cents.get()))),
        Some((id, [57, 286]))
    );
    assert_eq!(pool.income, 3);
    // 2026-10-19 MoneyPrecision: source rows -43c/-43c/-214c report 0/0/-2 dollars;
    // the separate three-dollar profit leaves one dollar of reported income.
    assert_eq!(return_line(&result, "7a"), -200);
    assert_eq!(value(&result, "agi"), 60_001);
    assert_eq!(supported(&restart(d.clone()).draft), result);
}

#[test]
fn k1_partner_property_niit_uses_passive_origin_and_trading_class() {
    for (material, trading, expected) in
        [(true, false, 0), (false, false, 836), (false, true, 836), (true, true, 836)]
    {
        let (mut d, _) = partner_property_draft(12_000);
        wages(&mut d, 250_000);
        let FederalIncome::Partnership { data } = &mut d.income.last_mut().unwrap().income else {
            panic!()
        };
        let crate::PartnerParticipation::General { facts } = &mut data.participation else {
            panic!()
        };
        facts.minutes = if material { 30_001 } else { 0 };
        data.trade_kind = if trading {
            crate::ScorpTradeKind::FinancialTrading
        } else {
            crate::ScorpTradeKind::Ordinary
        };
        let r = supported(&d);
        assert_eq!(value(&r, "net_investment_income_tax"), expected, "{material}/{trading}");
    }
}

#[test]
fn k1_partner_property_amt_uses_independent_property_and_lookback() {
    let (mut d, id) = partner_property_draft(12_000);
    let FederalIncome::Partnership { data } = &mut d.income.last_mut().unwrap().income else {
        panic!()
    };
    let amt = data.alternative.as_mut().unwrap();
    amt.section1231.as_mut().unwrap().net = cents(8_000);
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("partnership_{id}_section1231")), 12_000);
    assert_eq!(value(&r, &alloc::format!("amt_partnership_{id}_section1231")), 8_000);
    assert_eq!(lookback_worksheet(&d, false).unwrap().unwrap().long_term, 7_000);
    assert_eq!(lookback_worksheet(&d, true).unwrap().unwrap().long_term, 3_000);
    assert_eq!(return_line(&r, "11a / 11b"), cents(102_000));
    assert_eq!(value(&r, "qbi_deduction"), 3_000);
}

#[test]
fn k1_partner_property_combines_entity_kinds_without_prorating_lookback() {
    let (mut d, id) = partner_property_draft(12_000);
    let FederalIncome::Partnership { data } = &mut d.income.last_mut().unwrap().income else {
        panic!()
    };
    data.regular.section1231.as_mut().unwrap().unrecaptured = 0;
    mirror(data);
    let mut s = shareholder(true);
    s.section1231 = Some(partner_property(8_000, 8_000));
    s.amt = Some(amt_records(&s));
    add(&mut d, income(FederalIncome::Scorp { data: s }));
    let r = supported(&d);
    assert_eq!(value(&r, "form4797_line8"), 20_000);
    assert_eq!(value(&r, "form4797_line10"), 15_000);
    assert_eq!(value(&r, "qbi_deduction"), 2_000);
    let pool = crate::extended_rules::rentals(&d, 0, &mut estimate(&d));
    assert_eq!(crate::section1231::partner_property_qbi(&d, &pool, id).unwrap(), 0);
    // Both sources now have 25%-class gain. Five thousand of total recapture
    // cannot determine which source's 3,000/8,000 class supplied it.
    let FederalIncome::Partnership { data } = &mut d.income[1].income else { panic!() };
    data.regular.section1231.as_mut().unwrap().unrecaptured = cents(3_000);
    mirror(data);
    assert!(refused_with(
        &estimate(&d),
        "ISSUE_SECTION1231_SECTION_1231_LOOKBACK_RECAPTURE_NEEDS_BUSINESS_LEVEL_F77D46EA"
    ));
}

#[test]
fn k1_partner_property_partial_qualified_recapture_requires_determined_share() {
    let (mut d, id) = partner_property_draft(12_000);
    let FederalIncome::Partnership { data } = &mut d.income[1].income else { panic!() };
    data.qbi.as_mut().unwrap().section1231_qbi = cents(6_000);
    assert!(refused_with(&estimate(&d), "ISSUE_K1_SECTION1231_QBI_SPLIT"));
    // Full recapture determines the qualified amount without assuming a
    // proportional five-thousand share of the six-thousand qualified pool.
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2024, 12_000, 12_000)], &[(2024, 12_000, 12_000)]));
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_deduction"), 3_200);
    let pool = crate::extended_rules::rentals(&d, 0, &mut estimate(&d));
    assert_eq!(crate::section1231::partner_property_qbi(&d, &pool, id).unwrap(), 6_000);
}

#[test]
fn k1_partner_property_qualified_fraction_follows_each_loss_stage() {
    let mut p = member(0);
    p.regular.section1231 = Some(crate::K1Section1231 {
        basis_loss: cents(6_000),
        risk_loss: cents(4_000),
        passive_loss: cents(2_000),
        ..partner_property(-6_000, 0)
    });
    start(&mut p, 6_000, 5_000);
    let crate::PartnerParticipation::General { facts } = &mut p.participation else { panic!() };
    facts.minutes = 0;
    let mut qbi = partner_qbi_statement(0);
    qbi.section1231_qbi = cents(-3_000);
    p.qbi = Some(qbi);
    mirror(&mut p);
    let (mut d, id) = draft_of(p);
    wages(&mut d, 80_000);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let mut s = shareholder(false);
    s.ordinary = cents(5_000);
    s.amt = Some(amt_records(&s));
    add(&mut d, income(FederalIncome::Scorp { data: s }));
    let r = supported(&d);
    let pool = crate::extended_rules::rentals(&d, 0, &mut estimate(&d));
    let ledger = &pool.k1_ledgers.iter().find(|v| v.0 == id).unwrap().1;
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.basis_allowed[K1LossClass::Section1231].get()),
        6_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.risk_allowed[K1LossClass::Section1231].get()),
        5_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(ledger.passive_suspended[K1LossClass::Section1231].get()),
        2_000
    );
    // Basis: 6,000 * (3,000 + 6,000) / 12,000 = 4,500 qualified.
    // Risk: 5,000 * (4,500 + 4,000) / 10,000 = 4,250 qualified.
    // PAL: 5,000 * (4,250 + 2,000) / 7,000 = 4,464, rounded once.
    assert_eq!(crate::section1231::partner_property_qbi(&d, &pool, id).unwrap(), -4_464);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    assert_eq!(return_line(&r, "11a / 11b"), cents(80_000));
}

// B9-7 R8, 2025 i1065sk1 box 14 and 2026 Schedule SE drafts:
// code A is independently computed; only an ordinary loss suspension
// requires it to fit the ordinary loss's allocation capacity.
fn box14_source(ordinary: i64, reported: i64) -> PartnerK1 {
    let mut source = member(ordinary);
    source.limited_for_se = false;
    source.reported_se = Some(cents(reported));
    start(&mut source, 100_000, 100_000);
    mirror(&mut source);
    source
}

#[test]
fn k1_box14_f3_and_f3c_match_wage_base_qbi_and_farm_lines() {
    for farm in [false, true] {
        let mut source = box14_source(15_000, 20_000);
        source.qbi = Some(partner_qbi_statement(15_000));
        // The existing half-SE/QBI owner also needs gross-allocation facts;
        // all of this synthetic source's $30,000 gross is qualified.
        source.se_gross = Some(if farm {
            crate::PartnerSeGross::Farm { gross: cents(30_000) }
        } else {
            crate::PartnerSeGross::Nonfarm { gross: cents(30_000) }
        });
        source.qbi_records.qualified_se_gross = cents(30_000);
        let (mut d, _) = draft_of(source);
        wages(&mut d, 180_000);
        let r = supported(&d);
        for (name, expected) in [
            ("taxpayer_se_combined_profit", 20_000),
            ("taxpayer_se_net_earnings_ordinary", 18_470),
            ("taxpayer_se_net_earnings", 18_470),
            ("taxpayer_self_employment_tax", 1_094),
            ("taxpayer_half_self_employment_tax", 547),
            ("qbi_deduction", 2_891),
        ] {
            assert_eq!(value(&r, name), expected, "{farm}: {name}");
        }
        assert_eq!(return_line(&r, "11a / 11b"), cents(194_453));
        assert_eq!(return_line(&r, "15"), cents(175_462));
        assert_eq!(value(&r, "additional_medicare_tax"), 0);
        assert_eq!(value(&r, "taxpayer_se_nonfarm_profit"), if farm { 0 } else { 20_000 });
        assert_eq!(
            r.lines.iter().find(|l| l.id == "taxpayer_se_farm_profit").map(|l| {
                assert_eq!(printed_line(&r, &l.id), (TaxForm::ScheduleSe, "1a".into()));
                l.amount
            }),
            farm.then_some(cents(20_000))
        );
    }
}

#[test]
fn k1_box14_without_ordinary_suspension_accepts_independent_signs_and_amounts() {
    for (ordinary, reported) in [
        (15_000, 20_000),
        (15_000, -20_000),
        (0, 20_000),
        (0, -20_000),
        (-15_000, -20_000),
        (-15_000, 20_000),
        (-15_000, 0),
        (-15_000, -15_000),
    ] {
        let source = box14_source(ordinary, reported);
        let result = source
            .se_after_passive(
                crate::kernel::carry::Amount::new(0).unwrap(),
                crate::kernel::carry::Amount::new(0).unwrap(),
            )
            .unwrap();
        assert_eq!(result.earnings, reported, "{ordinary}/{reported}");
        assert_eq!(
            [result.basis_suspended, result.risk_suspended, result.passive_suspended],
            [0, 0, 0]
        );
    }
}

#[test]
fn k1_box14_f3b_refuses_incompatible_share_at_each_binding_ordinary_limit() {
    let key = "ISSUE_PARTNERSHIP_RECONCILE_BOX_14_LESS_GUARANTEED_PAYMENTS_F10E8824";
    for stage in 0..3 {
        for reported in [-20_000, 1_000] {
            let mut source = box14_source(-15_000, reported);
            let suspended = k1_amount(match stage {
                0 => {
                    source.regular.basis.opening = cents(10_000);
                    0
                }
                1 => {
                    source.regular.risk.start =
                        ScorpRiskStart::Detailed { capacity: cents(10_000), profit: 0, loss: 0 };
                    0
                }
                _ => {
                    source.participation =
                        PartnerParticipation::General { facts: participation_facts() };
                    5_000
                }
            });
            assert_eq!(
                source
                    .se_after_passive(suspended, crate::kernel::carry::Amount::new(0).unwrap())
                    .unwrap_err()
                    .0,
                key
            );
            mirror(&mut source);
            let (d, _) = draft_of(source.clone());
            assert!(refused_with(&estimate(&d), key));
            // Exact fit is still apportioned by the normal owner.
            source.reported_se = Some(cents(-15_000));
            let result = source
                .se_after_passive(suspended, crate::kernel::carry::Amount::new(0).unwrap())
                .unwrap();
            assert_eq!(result.earnings, -10_000);
            assert_eq!(
                result.basis_suspended + result.risk_suspended + result.passive_suspended,
                5_000
            );
        }
    }
}

#[test]
fn k1_box14_unmatched_current_loss_does_not_corrupt_retained_character() {
    let mut source = box14_source(-1_000, -2_000);
    source.regular.basis.prior_loss = cents(600);
    source.regular.risk.prior_loss = cents(400);
    source.regular.passive_loss = cents(200);
    source.se_history =
        crate::PartnerSeHistory { basis: cents(300), risk: cents(100), passive: cents(50) };
    // Former-passive prior loss cannot offset a current loss on its own.
    // Other passive income releases the $200, supplied as zero suspension.
    let result = source
        .se_after_passive(
            crate::kernel::carry::Amount::new(0).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap(),
        )
        .unwrap();
    assert_eq!(result.earnings, -2_450);
    assert_eq!(result.passive_earnings, -50);
    assert_eq!(
        [result.basis_suspended, result.risk_suspended, result.passive_suspended],
        [0, 0, 0]
    );
}

#[test]
fn k1_box14_positive_amount_above_box1_keeps_limited_prior_loss_character() {
    let mut source = box14_source(1_000, 2_000);
    source.regular.passive_loss = cents(1_500);
    source.se_history.passive = cents(750);
    // Current income releases $1,000 of prior loss; $500 stays suspended.
    let result = source
        .se_after_passive(
            crate::kernel::carry::Amount::new(500 * 100).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap(),
        )
        .unwrap();
    assert_eq!(result.earnings, 1_500);
    assert_eq!(result.passive_suspended, 250);
    assert_eq!(result.passive_earnings, 0);
}

#[test]
fn k1_box14_limited_and_trading_checks_remain_before_allocation() {
    let key = "ISSUE_PARTNERSHIP_RECONCILE_BOX_14_LESS_GUARANTEED_PAYMENTS_F10E8824";
    for trading in [false, true] {
        let mut source = box14_source(15_000, 20_000);
        source.limited_for_se = !trading;
        if trading {
            source.trade_kind = crate::ScorpTradeKind::FinancialTrading;
        }
        assert_eq!(
            source
                .se_after_passive(
                    crate::kernel::carry::Amount::new(0).unwrap(),
                    crate::kernel::carry::Amount::new(0).unwrap()
                )
                .unwrap_err()
                .0,
            key
        );
    }
    let mut source = box14_source(15_000, 3_000);
    source.limited_for_se = true;
    source.guaranteed_services = cents(3_000);
    source.guaranteed_capital = cents(4_000);
    assert_eq!(
        source
            .se_after_passive(
                crate::kernel::carry::Amount::new(0).unwrap(),
                crate::kernel::carry::Amount::new(0).unwrap()
            )
            .unwrap()
            .earnings,
        3_000
    );
}

#[test]
fn k1_box14_other_class_limits_and_amt_do_not_restrict_regular_ordinary_source() {
    let mut source = box14_source(15_000, 20_000);
    source.regular.section1231 = Some(partner_property(-30_000, 0));
    source.regular.basis.opening = 0;
    // A property-only suspension does not limit the box 1 SE allocation.
    assert_eq!(
        source
            .se_after_passive(
                crate::kernel::carry::Amount::new(0).unwrap(),
                crate::kernel::carry::Amount::new(0).unwrap()
            )
            .unwrap()
            .earnings,
        20_000
    );
    source.alternative.as_mut().unwrap().ordinary = cents(-500_000);
    assert_eq!(
        source
            .se_after_passive(
                crate::kernel::carry::Amount::new(0).unwrap(),
                crate::kernel::carry::Amount::new(0).unwrap()
            )
            .unwrap()
            .earnings,
        20_000
    );
}

#[test]
fn k1_box14_farm_and_nonfarm_share_one_wage_base_per_owner() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, cents(180_000), 0);
    add(&mut d, wage(0, Owner::Spouse, cents(180_000), 0));
    let mut source = box14_source(15_000, 20_000);
    source.se_gross = Some(crate::PartnerSeGross::Farm { gross: cents(30_000) });
    add(&mut d, income(FederalIncome::Partnership { data: source.clone() }));
    let mut spouse = income(FederalIncome::Partnership { data: source });
    spouse.owner = Owner::Spouse;
    add(&mut d, spouse);
    add(&mut d, income(cash_business(10_000)));
    let r = supported(&d);
    for (owner, combined, nonfarm, net, tax, half) in [
        ("taxpayer", 30_000, 10_000, 27_705, 1_361, 681),
        ("spouse", 20_000, 0, 18_470, 1_094, 547),
    ] {
        for (suffix, expected) in [
            ("se_farm_profit", 20_000),
            ("se_nonfarm_profit", nonfarm),
            ("se_combined_profit", combined),
            ("se_net_earnings", net),
            ("self_employment_tax", tax),
            ("half_self_employment_tax", half),
        ] {
            assert_eq!(value(&r, &alloc::format!("{owner}_{suffix}")), expected);
        }
    }
    assert_eq!(value(&r, "self_employment_tax"), 2_455);
    assert_eq!(value(&r, "schedule_1_half_self_employment_tax"), 1_228);
}

#[test]
fn k1_box14_farm_prints_post_limit_loss_and_optional_election_skips_it() {
    let mut source = box14_source(-15_000, -15_000);
    source.regular.basis.opening = cents(10_000);
    source.se_gross = Some(crate::PartnerSeGross::Farm { gross: cents(30_000) });
    mirror(&mut source);
    let (mut d, _) = draft_of(source);
    add(&mut d, income(cash_business(30_000)));
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_se_farm_profit"), -10_000);
    assert_eq!(value(&r, "taxpayer_se_nonfarm_profit"), 30_000);
    assert_eq!(value(&r, "taxpayer_se_combined_profit"), 20_000);

    let mut source = box14_source(1_000, 2_000);
    source.se_gross = Some(crate::PartnerSeGross::Farm { gross: cents(9_000) });
    let (mut d, _) = draft_of(source);
    elect_se(&mut d, SeMethod::Farm);
    let r = supported(&d);
    assert!(
        !r.lines
            .iter()
            .any(|l| l.id == "taxpayer_se_farm_profit" || l.id == "taxpayer_se_crp_payments")
    );
    assert_eq!(value(&r, "taxpayer_se_combined_profit"), 0);
    assert_eq!(value(&r, "taxpayer_se_optional_earnings"), 6_000);
    assert_eq!(value(&r, "taxpayer_se_net_earnings"), 6_000);
    assert_eq!(value(&r, "taxpayer_self_employment_tax"), 918);
}

#[test]
fn k1_box14_unmatched_loss_reaches_owner_and_missing_gross_keeps_qbi_notice() {
    let source = box14_source(-15_000, -20_000);
    let (d, _) = draft_of(source);
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_se_combined_profit"), -20_000);
    assert_eq!(value(&r, "taxpayer_se_net_earnings"), 0);
    assert_eq!(value(&r, "taxpayer_self_employment_tax"), 0);
    assert_eq!(return_line(&r, "11a / 11b"), cents(45_000));

    let mut source = box14_source(15_000, 20_000);
    source.qbi = Some(partner_qbi_statement(15_000));
    let (d, _) = draft_of(source);
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_se_combined_profit"), 20_000);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    assert!(noted(&r, "WARN_QBI_GROSS_ALLOCATION_LEFT_OUT"));
}

#[test]
fn k1_section179_sale_canonical_union_preserves_twenty_per_monetary_view() {
    let shared = section179_sale();
    let mut regular = shared.clone();
    regular.alternative = crate::K1Section179SaleAmtSource::NotReported;
    let mut alternative = shared.clone();
    alternative.alternative =
        crate::K1Section179SaleAmtSource::from_known_regular(&alternative.regular);
    alternative.regular = None;
    let mut disjoint = vec![regular.clone(); 20];
    disjoint.extend(vec![alternative.clone(); 20]);
    assert!(crate::K1Section179Sale::valid_source_list(&disjoint));
    for amt in [false, true] {
        let view = crate::K1Section179Sale::views(&disjoint, amt).unwrap();
        assert_eq!(view.len(), 20);
        assert!(crate::K1Section179Sale::valid_list(&view));
        assert_eq!(crate::K1Section179Sale::worksheets(&view, 0, 0).unwrap().len(), 20);
    }
    assert!(crate::K1Section179Sale::valid_source_list(&vec![shared.clone(); 20]));
    assert!(!crate::K1Section179Sale::valid_source_list(&vec![shared; 21]));
    disjoint[20] = regular;
    assert!(!crate::K1Section179Sale::valid_source_list(&disjoint));
    disjoint[20] = alternative.clone();
    disjoint[0] = alternative;
    assert!(!crate::K1Section179Sale::valid_source_list(&disjoint));
}

fn common_cent_personal(
    receipts: i64,
    expenses: i64,
    regular: i64,
    alternative: i64,
) -> FederalIncome {
    let mut source = personal_rent_with_history(0, 0, 0, 0, false);
    let FederalIncome::NonbusinessProperty {
        receipts: actual_receipts,
        expenses: actual_expenses,
        kind,
        ..
    } = &mut source
    else {
        panic!()
    };
    *actual_receipts = receipts;
    *actual_expenses = expenses;
    let NonbusinessPropertyKind::PersonalRental { history, .. } = kind;
    history.regular = regular;
    history.amt = alternative;
    source
}

#[test]
fn unit19_common_pal_mixed_cents_packet_keeps_one_exact_pool() {
    use crate::extended_rules::PostPalComponent;
    let mut loss = limited(-1);
    start(&mut loss, 100, 100);
    mirror(&mut loss);
    let (mut d, partner) = draft_of(loss);
    add(&mut d, income(common_cent_personal(50, 0, 0, 0)));
    let profit = d.income.last().unwrap().id;
    add(&mut d, income(common_cent_personal(0, 149, 0, 0)));
    let personal = d.income.last().unwrap().id;
    for alternative in [false, true] {
        let mut result = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 60_000, &mut result, alternative);
        assert!(pool.inputs_complete && result.issues.is_empty(), "{:?}", result.issues);
        let packet = pool.post_pal.as_ref().unwrap();
        assert_eq!(packet.alternative, alternative);
        assert_eq!(
            (packet.total_loss_cents, packet.suspended_cents, packet.allowed_loss_cents),
            (249, 199, 50)
        );
        let source = |id, component| {
            packet
                .sources
                .iter()
                .find(|source| source.income == id && source.component == component)
                .unwrap()
        };
        let k1 = source(partner, PostPalComponent::PartnershipBusiness);
        let cost = source(personal, PostPalComponent::PersonalProperty);
        let receipt = source(profit, PostPalComponent::PersonalProperty);
        assert_eq!((k1.net_cents, k1.suspended_cents, k1.allowed_net_cents), (-100, 80, -20));
        assert_eq!(
            (cost.net_cents, cost.suspended_cents, cost.allowed_net_cents),
            (-149, 119, -30)
        );
        assert_eq!(
            (receipt.net_cents, receipt.suspended_cents, receipt.allowed_net_cents),
            (50, 0, 50)
        );
        assert_eq!(pool.personal_inputs[&personal].expenses_cents - cost.suspended_cents, 30);
        assert_eq!(packet.sources.iter().map(|source| source.allowed_net_cents).sum::<i64>(), 0);
        if !alternative && std::env::var_os("TAX_UNIT19_COMMON_TRACE").is_some() {
            for name in [
                "form8582_line_2a",
                "form8582_line_2b",
                "form8582_line_2c",
                "form8582_line_2d",
                "form8582_line_3",
                "form8582_line_10",
                "form8582_line_11",
            ] {
                std::println!("{alternative} {name} {}", value(&result, name));
            }
            std::println!(
                "{alternative} annual_suspension={} native_income={} packet={packet:?}",
                pool.passive_loss,
                pool.income
            );
        }
    }
    let reopened = restart(d.clone());
    assert_eq!(reopened.draft, d);
}

#[test]
fn unit19_common_pal_cents_packet_keeps_actual_regular_and_amt_priors() {
    use crate::extended_rules::PostPalComponent;
    let mut loss = limited(-1);
    start(&mut loss, 100, 100);
    mirror(&mut loss);
    let (mut d, partner) = draft_of(loss);
    add(&mut d, income(common_cent_personal(50, 0, 0, 0)));
    add(&mut d, income(common_cent_personal(0, 149, 51, 151)));
    let personal = d.income.last().unwrap().id;
    for (alternative, prior, total_loss, held, personal_held, partner_held) in
        [(false, 51, 300, 250, 167, 83), (true, 151, 400, 350, 263, 87)]
    {
        let mut result = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 60_000, &mut result, alternative);
        assert!(pool.inputs_complete && result.issues.is_empty(), "{:?}", result.issues);
        let packet = pool.post_pal.as_ref().unwrap();
        assert_eq!(
            (packet.total_loss_cents, packet.suspended_cents, packet.allowed_loss_cents),
            (total_loss, held, 50)
        );
        let personal_source = packet
            .sources
            .iter()
            .find(|source| {
                source.income == personal && source.component == PostPalComponent::PersonalProperty
            })
            .unwrap();
        let partner_source = packet
            .sources
            .iter()
            .find(|source| {
                source.income == partner
                    && source.component == PostPalComponent::PartnershipBusiness
            })
            .unwrap();
        assert_eq!(personal_source.prior_cents, prior);
        assert_eq!(personal_source.suspended_cents, personal_held);
        assert_eq!(partner_source.suspended_cents, partner_held);
        assert_eq!(
            pool.personal_inputs[&personal].expenses_cents + prior - personal_held,
            if alternative { 37 } else { 33 }
        );
    }
}

#[test]
fn unit19_common_pal_packet_distinguishes_components_on_one_actual_parent() {
    use crate::extended_rules::PostPalComponent;
    let mut loss = limited(-1);
    loss.regular.rentals.real_estate = -100;
    loss.regular.rentals.other = -200;
    start(&mut loss, 100, 100);
    mirror(&mut loss);
    let (mut d, parent) = draft_of(loss);
    add(&mut d, income(common_cent_personal(50, 0, 0, 0)));
    for alternative in [false, true] {
        let mut result = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 60_000, &mut result, alternative);
        assert!(pool.inputs_complete && result.issues.is_empty(), "{:?}", result.issues);
        let packet = pool.post_pal.as_ref().unwrap();
        let sources: Vec<_> =
            packet.sources.iter().filter(|source| source.income == parent).collect();
        assert_eq!(sources.len(), 3);
        for component in [
            PostPalComponent::PartnershipBusiness,
            PostPalComponent::PartnershipRealEstate,
            PostPalComponent::PartnershipOtherRental,
        ] {
            assert_eq!(sources.iter().filter(|source| source.component == component).count(), 1);
        }
        assert_eq!(sources.iter().map(|source| source.net_cents).sum::<i64>(), -400);
        assert_eq!(sources.iter().map(|source| source.suspended_cents).sum::<i64>(), 350);
        assert_eq!(sources.iter().map(|source| source.allowed_net_cents).sum::<i64>(), -50);
    }
}

#[test]
fn unit19_common_pal_packet_keeps_recharacterized_zero_and_missing_source_distinct() {
    use crate::extended_rules::PostPalComponent;
    let mut d = complete(FederalStatus::Single);
    let mut investment = common_cent_personal(1000, 0, 0, 0);
    let FederalIncome::NonbusinessProperty { kind, .. } = &mut investment else { panic!() };
    let NonbusinessPropertyKind::PersonalRental { depreciable_basis, .. } = kind;
    *depreciable_basis = 0;
    add(&mut d, income(investment));
    let investment = d.income.last().unwrap().id;
    add(&mut d, income(common_cent_personal(0, 49, 0, 0)));
    let cost = d.income.last().unwrap().id;
    for alternative in [false, true] {
        let mut result = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 60_000, &mut result, alternative);
        let packet = pool.post_pal.as_ref().unwrap();
        assert_eq!(
            (packet.total_loss_cents, packet.suspended_cents, packet.allowed_loss_cents),
            (49, 49, 0)
        );
        let source = |id| {
            packet
                .sources
                .iter()
                .find(|source| {
                    source.income == id && source.component == PostPalComponent::PersonalProperty
                })
                .unwrap()
        };
        assert!(!source(investment).common_pool);
        assert_eq!(source(investment).suspended_cents, 0);
        assert_eq!(source(cost).suspended_cents, 49);
    }
    let mut missing = rental(0, 1);
    missing.buildings.clear();
    add(&mut d, income(FederalIncome::Rental { data: missing }));
    let mut result = FederalResult::empty(d.revision);
    let pool = crate::extended_rules::rentals_for(&d, 60_000, &mut result, false);
    assert!(!pool.inputs_complete);
    assert!(pool.post_pal.is_none());
}

#[test]
fn unit19_common_pal_fractional_loss_retains_native_health_classification() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(common_cent_personal(50, 0, 0, 0)));
    add(&mut d, income(common_cent_personal(0, 49, 0, 0)));
    add(&mut d, income(FederalIncome::Rental { data: rental(0, 1) }));
    let mut result = FederalResult::empty(d.revision);
    let pool = crate::extended_rules::rentals_for(&d, 60_000, &mut result, false);
    assert!(pool.inputs_complete);
    assert!(pool.health_ptc.mixed_losses);
}

#[test]
fn unit19_common_pal_packet_does_not_advertise_a_ptp_common_zero() {
    let mut source = limited(-1);
    start(&mut source, 100, 100);
    mirror(&mut source);
    source.publicly_traded = true;
    let (mut d, parent) = draft_of(source);
    add(&mut d, income(common_cent_personal(1000, 0, 0, 0)));
    for alternative in [false, true] {
        let mut result = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 60_000, &mut result, alternative);
        assert!(pool.inputs_complete && result.issues.is_empty(), "{:?}", result.issues);
        let packet = pool.post_pal.as_ref().unwrap();
        assert_eq!((packet.total_loss_cents, packet.suspended_cents), (0, 0));
        assert!(!packet.sources.iter().any(|source| source.income == parent));
        assert_eq!(pool.passive_loss, 1);
    }
}

#[test]
fn unit19_common_pal_rental_gain_is_retained_in_its_own_source_character() {
    use crate::extended_rules::{PostPalCharacter, PostPalComponent};
    let mut d = complete(FederalStatus::Single);
    wages(&mut d, 200_000);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let mut property = rental(0, 2000);
    property.property_type = RentalPropertyType::Land;
    property.buildings[0].cost = 100_000;
    property.buildings[0].land = 100_000;
    // The continuing like-kind property recognizes $1,000 cash boot. Its
    // $2,000 operating loss stays in the common pool rather than the sold
    // property's existing fully taxable disposition owner.
    property.sale = RentalSale::Exchanged(crate::RentalExchange {
        transferred: 20260101,
        identified: 20260201,
        received: 20260301,
        extended: None,
        replacement: crate::ExchangeReplacement {
            address: "9 Oak Lane".into(),
            value: 100_000,
            land: 100_000,
            used: crate::ReplacementUse::Investment,
        },
        cash_received: 100_000,
        other_received: 0,
        debt_relieved: 0,
        debt_assumed: 0,
        cash_paid: 0,
        expenses: 0,
        land_value: 200_000,
        party: crate::ExchangeParty::Unrelated,
        special: false,
    });
    add(&mut d, income(FederalIncome::Rental { data: property }));
    let id = d.income.last().unwrap().id;
    for alternative in [false, true] {
        let mut result = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 200_000, &mut result, alternative);
        assert!(pool.inputs_complete && result.issues.is_empty(), "{:?}", result.issues);
        let packet = pool.post_pal.as_ref().unwrap();
        let source = packet
            .sources
            .iter()
            .find(|source| {
                source.income == id && source.component == PostPalComponent::DirectRental
            })
            .unwrap();
        let gain = source
            .characters
            .iter()
            .find(|amount| amount.character == PostPalCharacter::PropertyDisposition)
            .unwrap();
        let operating = source
            .characters
            .iter()
            .find(|amount| amount.character == PostPalCharacter::ScheduleE)
            .unwrap();
        assert_eq!(
            (gain.net_cents, gain.suspended_cents, gain.allowed_net_cents),
            (100_000, 0, 100_000)
        );
        assert_eq!(
            (operating.net_cents, operating.suspended_cents, operating.allowed_net_cents),
            (-200_000, 100_000, -100_000)
        );
        assert_eq!(source.allowed_net_cents, 0);
    }
}

#[test]
fn positive_item_source_cents_fund_zero_opening_basis_and_risk_once() {
    for receipts in [49, 100] {
        let mut partner = limited(0);
        start(&mut partner, 0, 0);
        partner.regular.ordinary = -149;
        partner.income =
            vec![PartnerIncome { code: PartnerIncomeCode::PortfolioIncome, amount: receipts }];
        mirror(&mut partner);
        let output = partner.pool(&partner.regular).unwrap();
        let ledger = output.activity.ledger;
        assert_eq!(ledger.basis_allowed[K1LossClass::Ordinary].get(), receipts);
        assert_eq!(ledger.basis_suspended[K1LossClass::Ordinary].get(), 149 - receipts);
        assert_eq!(ledger.risk_allowed[K1LossClass::Ordinary].get(), receipts);
        assert_eq!(ledger.risk_suspended[K1LossClass::Ordinary].get(), 0);
        assert_eq!(output.activity.basis.ending.get(), 0);
    }
}

#[test]
fn partner_self_employment_consumes_actual_subdollar_section179_allowance() {
    use crate::kernel::carry::{Amount, Cents};
    for deduction in [50, 149] {
        let mut partner = member(2);
        partner.limited_for_se = false;
        partner.reported_se = Some(200);
        start(&mut partner, 1_000, 1_000);
        partner.section179 = deduction;
        mirror(&mut partner);
        let result = partner
            .se_after_passive(Amount::<Cents>::default(), Amount::new(deduction).unwrap())
            .unwrap();
        assert_eq!(result.earnings_cents, 200 - deduction);
    }
}

#[test]
fn ptp_ordinary_and_code_r_source_holds_are_retained_once_per_character() {
    let mut partner = limited(0);
    partner.publicly_traded = true;
    partner.regular.ordinary = -149;
    partner.income =
        vec![PartnerIncome { code: PartnerIncomeCode::SpeciallyAllocatedOrdinary, amount: -100 }];
    start(&mut partner, 1_000, 1_000);
    mirror(&mut partner);
    let (draft, id) = draft_of(partner);
    for alternative in [false, true] {
        let mut report = FederalResult::empty(draft.revision);
        let result = crate::extended_rules::rentals_for(&draft, 200_000, &mut report, alternative);
        assert!(result.inputs_complete, "{:?}", report.issues);
        assert_eq!(
            result.partners.iter().find(|source| source.id == id).unwrap().suspended.get(),
            149
        );
        if !alternative {
            let ledger = &result.k1_ledgers.iter().find(|(source, _)| *source == id).unwrap().1;
            assert_eq!(ledger.passive_suspended[K1LossClass::Ordinary].get(), 149);
            assert_eq!(ledger.passive_suspended[K1LossClass::OtherLoss].get(), 100);
        }
    }
}

#[test]
fn guaranteed_payment_items_keep_source_cents_and_separate_schedule_e_rows() {
    let mut partner = member(0);
    partner.regular.ordinary = -1;
    partner.guaranteed_services = 49;
    partner.guaranteed_capital = 49;
    partner.reported_se = Some(49);
    start(&mut partner, 1_000, 1_000);
    mirror(&mut partner);
    let (draft, id) = draft_of(partner);
    for alternative in [false, true] {
        let mut report = FederalResult::empty(draft.revision);
        let result = crate::extended_rules::rentals_for(&draft, 200_000, &mut report, alternative);
        assert!(result.inputs_complete, "{:?}", report.issues);
        // 2026 i1040se Line28: separately stated 4a/4b items retain separate rows.
        // They are nonpassive, but neither source payment adds outside basis.
        let source = result.partners.iter().find(|source| source.id == id).unwrap();
        assert_eq!((source.net, source.nonpassive), (0, 0));
        assert_eq!(
            result.k1_amounts.iter().find(|source| source.id == id).unwrap().source_net_cents,
            Some(97)
        );
    }
}
