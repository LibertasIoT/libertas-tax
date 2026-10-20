use crate::tests::support::assert::line_amount as cents;
use crate::tests::support::assert::{accepted, rejected_with};
// u6-home: the sale of your main home through its page (Investment sales,
// kind "Sale of your main home (1099-S)"), and Form 8828's sale path. Every
// expected amount is worked by hand in the comment beside it from Pub. 523
// (Worksheets 1-3), the Schedule D and Form 8949 instructions (2025 drafts,
// the 2026 amounts the App uses: $16,100 single and $32,200 joint standard
// deduction; 10% to $12,400 / $24,800, 12% to $50,400 / $100,800, 22% to
// $105,700, 24% above; capital gain 0% up to $49,450 / $98,900; Tax Table
// rows $50 wide taxed at their midpoint) and Form 8828.
use super::*;
use crate::tests::cross::interview::{
    document, open_other_taxes, open_topic, other_taxes_request, page_documents, save_other_taxes,
    save_topic,
};

fn has_line(r: &FederalResult, id: &str) -> bool {
    r.lines.iter().any(|l| l.id == id)
}

// A single filer's home, bought 2015-03-01 and sold 2026-06-15 for $450,000
// with $25,000 of selling expenses and a $200,000 basis, lived in the whole
// time, with a 1099-S.
pub(in crate::tests) fn home(joint: bool) -> MainHomeSale {
    MainHomeSale {
        address: "12 Synthetic Lane".into(),
        closed: 20260615,
        proceeds: 45_000_000,
        property_received: false,
        foreign_person: false,
        form_1099s: true,
        acquired: 20150301,
        expenses: 2_500_000,
        basis: 20_000_000,
        lived: HomeLived::WholeTime,
        earlier_sale: EarlierHomeSale::No,
        spouse: if joint {
            HomeSpouse::Joint { lived: HomeLived::WholeTime, earlier_sale: EarlierHomeSale::No }
        } else {
            HomeSpouse::NotJoint
        },
        reason: HomeSaleReason::TwoYears,
        use_history: HomeRentalUse { depreciation: 0, nonqualified_days: 0 },
        more: HomeSaleMore::Nothing,
    }
}
fn save_homes(s: &mut Interview, homes: Vec<MainHomeSale>) -> Reply {
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        homes.into_iter().map(|h| document(0, IncomeDocument::MainHomeSale(h))).collect(),
    );
    send(s, request)
}
fn home_id(d: &FederalDraft) -> i64 {
    d.income.iter().find(|e| matches!(e.income, FederalIncome::HomeSale { .. })).unwrap().id
}
// The sale saved alone on a fresh return of this status, and its result.
fn sold(status: FederalStatus, sale: MainHomeSale) -> (Interview, FederalResult) {
    let mut s = Interview::new(TEST_USER, complete(status));
    accepted(&save_homes(&mut s, vec![sale]));
    let r = estimate(&s.draft);
    (s, r)
}
// The page reopens with the sale as saved, and saving it unchanged after a
// restart keeps the stored return.
fn round_trip(s: &Interview, sale: &MainHomeSale) {
    let stored = s.draft.clone();
    let mut again = restart(stored.clone());
    let page = open_topic(&mut again, TaxSection::InvestmentSales);
    let rows = page_documents(&page).to_vec();
    assert!(
        rows.iter().any(|row| row.document == IncomeDocument::MainHomeSale(sale.clone())),
        "{rows:?}"
    );
    let request = save_topic(&again.draft.clone(), TaxSection::InvestmentSales, rows);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft, stored);
}

#[test]
fn fully_excluded_single_sale_through_the_page() {
    // Pub. 523 Worksheet 2: $450,000 - $25,000 - $200,000 = $225,000 gain.
    // Owned and lived in 2 of the 5 years, no earlier sale: Worksheet 1
    // limit $250,000, so all $225,000 is excluded and nothing is taxable.
    // With a 1099-S the sale is still reported (Schedule D instructions):
    // Form 8949 Part II box F, (d) $450,000, (e) $200,000, (g) -$250,000
    // ($25,000 expenses + $225,000 exclusion, codes EH), (h) $0.
    let baseline = supported(&complete(FederalStatus::Single));
    let sale = home(false);
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    let id = home_id(&s.draft);
    assert_eq!(id, 2);
    let entry = &s.draft.income[1];
    assert_eq!(entry.label, "12 Synthetic Lane");
    let FederalIncome::HomeSale { data } = &entry.income else { panic!() };
    assert_eq!(data.term, GainTerm::Long);
    assert!(matches!(
        data.exclusion,
        HomeExclusion::Full {
            taxpayer: HomeUse { owned: 1826, lived: 1826, prior_exclusion: false, .. },
            spouse: None,
        }
    ));
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_realized_gain"), 22_500_000);
    assert_eq!(cents(&r, "home_2_excluded_gain"), 22_500_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 0);
    assert_eq!(cents(&r, "home_2_form_8949_proceeds"), 45_000_000);
    assert_eq!(cents(&r, "home_2_form_8949_basis"), 20_000_000);
    assert_eq!(cents(&r, "home_2_form_8949_adjustment"), -25_000_000);
    assert_eq!(cents(&r, "home_2_form_8949_gain"), 0);
    let explanation =
        &r.lines.iter().find(|l| l.id == "home_2_form_8949_adjustment").unwrap().explanation;
    assert!(
        explanation.english().contains("Part II box F")
            && explanation.english().contains("code EH")
    );
    assert_eq!(value(&r, "agi"), value(&baseline, "agi"));
    assert_eq!(r.tax, baseline.tax);
    round_trip(&s, &sale);

    // Without a 1099-S a fully excluded sale is not reported at all.
    let mut quiet = home(false);
    quiet.form_1099s = false;
    let (_, r) = sold(FederalStatus::Single, quiet);
    assert!(!has_line(&r, "home_2_form_8949_proceeds"));
    assert!(
        r.review_notes.iter().any(|n| n.contains("not reported on Form 8949")),
        "{:?}",
        r.review_notes
    );
    assert_eq!(r.tax, baseline.tax);
}

#[test]
fn joint_sale_uses_the_500000_exclusion() {
    // Joint, both lived there the whole time, bought 2010-05-01, sold
    // 2026-07-01 for $1,200,000 with $60,000 expenses and a $500,000
    // basis: gain $640,000. Worksheet 1: both meet residence and look-back,
    // one meets ownership: $500,000. Taxable long-term gain $140,000.
    // AGI $100,000 + $140,000 = $240,000 (below the $250,000 NIIT
    // threshold); taxable income $240,000 - $32,200 = $207,800, ordinary
    // $67,800. QDCG worksheet: 0% on $98,900 - $67,800 = $31,100; 15% on
    // $108,900 = $16,335; Tax Table (joint) on $67,800: $7,643. Tax
    // $23,978.
    let mut sale = home(true);
    sale.acquired = 20100501;
    sale.closed = 20260701;
    sale.proceeds = 120_000_000;
    sale.expenses = 6_000_000;
    sale.basis = 50_000_000;
    let (s, r) = sold(FederalStatus::Joint, sale.clone());
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_excluded_gain"), 50_000_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 14_000_000);
    assert_eq!(value(&r, "capital_gain_10"), 140_000);
    assert_eq!(value(&r, "agi"), 240_000);
    assert_eq!(r.tax, Some(2_397_800));
    round_trip(&s, &sale);

    // Only one spouse meets the residence test: that spouse's $250,000
    // (section 121(b)(2)(B)), taxable $390,000.
    let mut one = sale.clone();
    one.spouse = HomeSpouse::Joint {
        lived: HomeLived::Days { days: 400 },
        earlier_sale: EarlierHomeSale::No,
    };
    let (_, r) = sold(FederalStatus::Joint, one);
    assert_eq!(cents(&r, "home_2_excluded_gain"), 25_000_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 39_000_000);
}

#[test]
fn partial_exclusion_for_a_work_move() {
    // Single, bought 2025-06-01, sold 2026-06-01 (365 days, held exactly
    // one year: short-term) after a new job 70 miles from the home (the old
    // one was 10 miles away: at least 50 miles farther). Gain $500,000 -
    // $30,000 - $320,000 = $150,000. Worksheet 1 Section B: the shortest of
    // residence (365), ownership (365) and time since an earlier sale (none)
    // is 365; 365 / 730 x $250,000 = $125,000. Taxable short-term gain
    // $25,000; AGI $85,000; taxable income $68,900, all ordinary; Tax Table
    // (single) $9,876.
    let mut sale = home(false);
    sale.acquired = 20250601;
    sale.closed = 20260601;
    sale.proceeds = 50_000_000;
    sale.expenses = 3_000_000;
    sale.basis = 32_000_000;
    sale.reason = HomeSaleReason::Event {
        reason: HomeSaleTrigger::WorkMove { old_miles: 10, new_miles: 70 },
    };
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    let FederalIncome::HomeSale { data } = &s.draft.income[1].income else { panic!() };
    assert_eq!(data.term, GainTerm::Short);
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_excluded_gain"), 12_500_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 2_500_000);
    assert_eq!(value(&r, "capital_gain_3"), 25_000);
    assert_eq!(value(&r, "agi"), 85_000);
    assert_eq!(r.tax, Some(987_600));
    let explanation =
        &r.lines.iter().find(|l| l.id == "home_2_form_8949_adjustment").unwrap().explanation;
    assert!(explanation.english().contains("Part I box C"));
    round_trip(&s, &sale);

    // A new job only 49 miles farther is not the work-move safe harbor.
    let mut near = sale.clone();
    near.reason = HomeSaleReason::Event {
        reason: HomeSaleTrigger::WorkMove { old_miles: 10, new_miles: 59 },
    };
    let (_, r) = sold(FederalStatus::Single, near);
    assert_eq!(r.state, ResultState::Unsupported);
}

#[test]
fn gain_above_the_exclusion_with_depreciation_and_nonqualified_use() {
    // Single, bought 2012-01-10, sold 2026-04-20: 5,214 days owned. Rented
    // part of the time: $20,000 straight-line depreciation after May 6,
    // 1997 and 869 days of nonqualified use after 2008. Gain $700,000 -
    // $40,000 - $280,000 = $380,000. Pub. 523 Worksheet 3: the $20,000
    // depreciation is taxable (unrecaptured section 1250 gain); of the other
    // $360,000, 869 / 5,214 = 1/6 ($60,000) is nonqualified; $300,000 could
    // be excluded, limited to $250,000. Taxable $130,000 long-term.
    // AGI $190,000; taxable income $173,900. Schedule D Tax Worksheet:
    // line 10 $130,000, line 11 $20,000, line 13 $110,000, line 14 $63,900,
    // lines 16-17 $49,450, line 18 $43,900, line 21 $63,900, line 22 $0,
    // line 30 $110,000 x 15% = $16,500, line 39 $0 (line 38 $20,000), Tax
    // Table on $63,900 $8,776: line 45 $25,276, below $34,334 on line 46.
    let mut sale = home(false);
    sale.acquired = 20120110;
    sale.closed = 20260420;
    sale.proceeds = 70_000_000;
    sale.expenses = 4_000_000;
    sale.basis = 28_000_000;
    sale.use_history = HomeRentalUse { depreciation: 2_000_000, nonqualified_days: 869 };
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_depreciation_gain"), 2_000_000);
    assert_eq!(cents(&r, "home_2_nonqualified_gain"), 6_000_000);
    assert_eq!(cents(&r, "home_2_excluded_gain"), 25_000_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 13_000_000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 20_000);
    assert_eq!(value(&r, "agi"), 190_000);
    assert_eq!(r.tax, Some(2_527_600));
    round_trip(&s, &sale);
}

#[test]
fn a_loss_on_the_home_is_not_deductible() {
    // $300,000 - $20,000 - $350,000: a $70,000 loss. Nothing is deducted;
    // with a 1099-S, Form 8949 shows (d) $300,000, (e) $350,000 and (g)
    // +$50,000 (codes EL), (h) $0.
    let baseline = supported(&complete(FederalStatus::Single));
    let mut sale = home(false);
    sale.proceeds = 30_000_000;
    sale.expenses = 2_000_000;
    sale.basis = 35_000_000;
    let (_, r) = sold(FederalStatus::Single, sale);
    assert_eq!(cents(&r, "home_2_realized_gain"), -7_000_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 0);
    assert_eq!(cents(&r, "home_2_form_8949_adjustment"), 5_000_000);
    let explanation =
        &r.lines.iter().find(|l| l.id == "home_2_form_8949_adjustment").unwrap().explanation;
    assert!(explanation.english().contains("code EL"), "{}", explanation.english());
    assert_eq!(r.tax, baseline.tax);
}

#[test]
fn ownership_and_use_boundary_is_exactly_two_years() {
    // Bought 2024-06-01, sold 2026-06-01: 730 days (2024 is a leap year),
    // exactly 2 years: the full exclusion. Bought a day later: 729 days,
    // no full exclusion without a reason, named by the estimate.
    let mut sale = home(false);
    sale.acquired = 20240601;
    sale.closed = 20260601;
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    let FederalIncome::HomeSale { data } = &s.draft.income[1].income else { panic!() };
    assert!(matches!(
        data.exclusion,
        HomeExclusion::Full { taxpayer: HomeUse { owned: 730, lived: 730, .. }, .. }
    ));
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_taxable_gain"), 0);
    sale.acquired = 20240602;
    let (_, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(
        r.issues.iter().any(|i| i.message.contains("full ownership/residence exclusion")),
        "{r:?}"
    );
    // Lived in it 730 of the days owned (bought earlier): full; 729: not.
    sale.acquired = 20200101;
    sale.lived = HomeLived::Days { days: 730 };
    assert_eq!(
        sold(FederalStatus::Single, sale.clone()).1.state,
        ResultState::EstimatedForSupportedScope
    );
    sale.lived = HomeLived::Days { days: 729 };
    assert_eq!(sold(FederalStatus::Single, sale).1.state, ResultState::Unsupported);
}

#[test]
fn exclusion_limit_boundary_is_250000_dollars() {
    // Gain exactly $250,000.00: all excluded. $250,000.01: one cent taxable.
    let mut sale = home(false);
    sale.proceeds = 47_500_000;
    let (_, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(cents(&r, "home_2_realized_gain"), 25_000_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 0);
    sale.proceeds = 47_500_001;
    let (_, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(cents(&r, "home_2_excluded_gain"), 25_000_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 1);
    // Joint: $500,000.00 excluded, $500,000.01 leaves one cent.
    let mut sale = home(true);
    sale.proceeds = 72_500_000;
    let (_, r) = sold(FederalStatus::Joint, sale.clone());
    assert_eq!(cents(&r, "home_2_taxable_gain"), 0);
    sale.proceeds = 72_500_001;
    let (_, r) = sold(FederalStatus::Joint, sale);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 1);
}

#[test]
fn earlier_excluded_sale_boundary_is_two_years() {
    // An earlier excluded sale 730 days before this one (2024-06-15 to
    // 2026-06-15) is outside the look-back: full exclusion. 729 days: the
    // look-back fails; with a qualifying event the reduced exclusion uses
    // 729 / 730 of $250,000 = $249,657.53 (rounded to the cent).
    let mut sale = home(false);
    sale.proceeds = 55_000_000; // gain $325,000
    sale.earlier_sale = EarlierHomeSale::Yes { sold: 20240615 };
    let (_, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(cents(&r, "home_2_excluded_gain"), 25_000_000);
    sale.earlier_sale = EarlierHomeSale::Yes { sold: 20240616 };
    let (_, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(r.state, ResultState::Unsupported);
    sale.reason = HomeSaleReason::Event { reason: HomeSaleTrigger::MedicalCare };
    let (_, r) = sold(FederalStatus::Single, sale);
    assert_eq!(cents(&r, "home_2_excluded_gain"), 24_965_753);
}

#[test]
fn holding_period_boundary_is_more_than_one_year() {
    // Sold on the first anniversary: short-term; a day later: long-term.
    let mut sale = home(false);
    sale.acquired = 20250615;
    sale.reason = HomeSaleReason::Event { reason: HomeSaleTrigger::Unemployment };
    let term = |sale: &MainHomeSale| sale.record().term;
    assert_eq!(term(&sale), GainTerm::Short);
    sale.acquired = 20250614;
    assert_eq!(term(&sale), GainTerm::Long);
}

#[test]
fn surviving_spouse_and_decline_through_the_page() {
    // A widow (single return) sells within 2 years of the death, both met
    // the tests just before it: $500,000. Gain $325,000 is all excluded.
    let mut sale = home(false);
    sale.proceeds = 55_000_000;
    sale.more = HomeSaleMore::SurvivingSpouse {
        died: 20250301,
        both_qualified: true,
        spouse_earlier_sale: false,
    };
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(cents(&r, "home_2_excluded_gain"), 32_500_000);
    round_trip(&s, &sale);
    // Reporting the whole gain instead: $325,000 taxable.
    sale.more = HomeSaleMore::Decline;
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(cents(&r, "home_2_taxable_gain"), 32_500_000);
    round_trip(&s, &sale);
}

#[test]
fn rare_parts_are_left_out_with_a_notice_and_expatriation_refused() {
    // Answers and unknowns: a named situation the page does not calculate
    // leaves the sale out with its notice; the expatriation tax, an added
    // tax, is refused by name.
    let refused = MainHomeSale {
        more: HomeSaleMore::Refused { situation: HomeSaleSpecial::Expatriate },
        ..home(false)
    };
    let (s, r) = sold(FederalStatus::Single, refused.clone());
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.message.contains("Form 8854")), "{r:?}");
    round_trip(&s, &refused);
    let situation =
        |situation| MainHomeSale { more: HomeSaleMore::Refused { situation }, ..home(false) };
    for (sale, key) in [
        (situation(HomeSaleSpecial::LikeKindExchange), "WARN_HOME_SALE_LIKE_KIND_LEFT_OUT"),
        (situation(HomeSaleSpecial::BusinessPart), "WARN_HOME_SALE_BUSINESS_PART_LEFT_OUT"),
        (situation(HomeSaleSpecial::ServiceDuty), "WARN_HOME_SALE_SERVICE_DUTY_LEFT_OUT"),
        (situation(HomeSaleSpecial::Installment), "WARN_HOME_SALE_INSTALLMENT_LEFT_OUT"),
        (situation(HomeSaleSpecial::Other), "WARN_HOME_SALE_OTHER_LEFT_OUT"),
        (
            MainHomeSale { property_received: true, ..home(false) },
            "WARN_HOME_SALE_PROPERTY_RECEIVED_LEFT_OUT",
        ),
        (
            MainHomeSale { foreign_person: true, ..home(false) },
            "WARN_HOME_SALE_FOREIGN_PERSON_LEFT_OUT",
        ),
    ] {
        let (s, r) = sold(FederalStatus::Single, sale.clone());
        assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{key}: {r:?}");
        assert!(noted(&r, key), "{key}: {r:?}");
        assert!(!r.lines.iter().any(|l| l.id.starts_with("home_2_")), "{key}");
        // The page states the situation in its facts; it never stores a
        // Not sure placeholder in the sale's ordinary-treatment answer.
        let FederalIncome::HomeSale { data } = &s.draft.income.last().unwrap().income else {
            panic!()
        };
        assert!(data.ordinary);
        round_trip(&s, &sale);
    }
}

#[test]
fn home_sale_page_rules_are_checked_on_save() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let bad = |f: &dyn Fn(&mut MainHomeSale)| {
        let mut v = home(false);
        f(&mut v);
        v
    };
    for (sale, key) in [
        (bad(&|v| v.closed = 20251231), "HOME_SALE_NOT_IN_CURRENT_YEAR"),
        (bad(&|v| v.acquired = 20260615), "HOME_ACQUIRED_AFTER_SALE"),
        (
            bad(&|v| {
                v.acquired = 20250615;
                v.lived = HomeLived::Days { days: 366 }
            }),
            "HOME_LIVED_ABOVE_OWNED",
        ),
        (
            bad(&|v| v.lived = HomeLived::CareFacility { home_days: 1000, facility_days: 827 }),
            "HOME_LIVED_ABOVE_OWNED",
        ),
        (
            bad(&|v| v.earlier_sale = EarlierHomeSale::Yes { sold: 20260615 }),
            "HOME_EARLIER_SALE_AFTER_SALE",
        ),
        (
            bad(&|v| {
                v.acquired = 20250615;
                v.use_history = HomeRentalUse { depreciation: 0, nonqualified_days: 366 }
            }),
            "HOME_NONQUALIFIED_ABOVE_OWNED",
        ),
        (
            bad(&|v| {
                v.more = HomeSaleMore::SurvivingSpouse {
                    died: 20260616,
                    both_qualified: true,
                    spouse_earlier_sale: false,
                }
            }),
            "HOME_DEATH_AFTER_SALE",
        ),
        (
            bad(&|v| {
                v.more = HomeSaleMore::SurvivingSpouse {
                    died: 20250301,
                    both_qualified: true,
                    spouse_earlier_sale: false,
                };
                v.reason = HomeSaleReason::Event { reason: HomeSaleTrigger::Divorce }
            }),
            "HOME_SURVIVING_WITH_REASON",
        ),
        (home(true), "HOME_SPOUSE_NOT_JOINT"),
    ] {
        let reply = if key == "HOME_SPOUSE_NOT_JOINT" {
            current_document_refusal(
                "income-source-898",
                &mut s,
                vec![document(0, IncomeDocument::MainHomeSale(sale))],
                "HOME_SPOUSE_NOT_JOINT",
                "[documents/documents/0/document/spouse]",
            )
        } else {
            save_homes(&mut s, vec![sale])
        };
        rejected_with(reply, key);
    }
    let mut joint = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    rejected_with(
        current_document_refusal(
            "income-source-899",
            &mut joint,
            vec![document(0, IncomeDocument::MainHomeSale(home(false)))],
            "HOME_SPOUSE_NEEDED",
            "[documents/documents/0/document/spouse]",
        ),
        "HOME_SPOUSE_NEEDED",
    );
    // The page offers exactly the spouse answer the return asks.
    let P::SaveIncomes { allowed_home_spouse, allowed_documents, .. } =
        open_topic(&mut joint, TaxSection::InvestmentSales)
    else {
        unreachable!()
    };
    assert_eq!(allowed_home_spouse, HomeSpouse::allowed(true));
    assert!(
        allowed_documents
            .contains(&(libertas_macros::variant_index!(IncomeDocument::MainHomeSale) as i32))
    );
    let P::SaveIncomes { allowed_home_spouse, .. } =
        open_topic(&mut s, TaxSection::InvestmentSales)
    else {
        unreachable!()
    };
    assert_eq!(allowed_home_spouse, HomeSpouse::allowed(false));
}

#[test]
fn home_sale_is_no_longer_a_less_common_refusal() {
    assert!(
        !RareSituation::allowed(TaxSection::MoreIncome).iter().any(|i| RareSituation::ALL
            [*i as usize]
            .refused()
            .lines
            .contains("main home"))
    );
}

#[test]
fn leaving_joint_erases_the_spouses_home_answers_and_joining_reopens_the_page() {
    let mut d = complete(FederalStatus::Joint);
    let mut s = Interview::new(TEST_USER, d.clone());
    let mut sale = home(true);
    sale.spouse = HomeSpouse::Joint {
        lived: HomeLived::WholeTime,
        earlier_sale: EarlierHomeSale::Yes { sold: 20250101 },
    };
    accepted(&save_homes(&mut s, vec![sale]));
    d = crate::interview::with_ticks(s.draft.clone());
    let mut stored = d.clone();
    assert_eq!(crate::interview::prune(&mut stored), Default::default());
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Single);
    d.people.as_mut().unwrap().spouse = SpouseFiler::No;
    let pruned = crate::interview::prune(&mut d);
    assert!(
        pruned.removed.iter().any(|item| item.key() == "SAVING_KIND_HOME_SALE_S_SPOUSE_ANSWERS"),
        "{pruned:?}"
    );
    let FederalIncome::HomeSale { data } = &d.income[1].income else { panic!() };
    assert!(matches!(data.exclusion, HomeExclusion::Full { spouse: None, .. }));
    assert_eq!(data.facts.as_ref().unwrap().spouse_earlier, None);
    // A single return's sale on a return that becomes joint: its page
    // refuses (the spouse's answers are needed) and reopens.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_homes(&mut s, vec![home(false)]));
    let mut d = crate::interview::with_ticks(s.draft.clone());
    d.finished_sections.push(TaxSection::InvestmentSales);
    let joint = complete(FederalStatus::Joint);
    d.setup = joint.setup.clone();
    d.people = joint.people.clone();
    let pruned = crate::interview::prune(&mut d);
    assert!(pruned.reopened.contains(&TaxSection::InvestmentSales), "{pruned:?}");
}

// ---------------------------------------------------------------------------
// Form 8828 on a sale: the Other taxes item's hidden home sale ID is filled
// from the return's home sales.

fn subsidized_sale() -> MortgageSubsidyEntry {
    MortgageSubsidyEntry {
        label: "Maple Street".into(),
        disposition: MortgageSubsidyDisposition::Ordinary {
            subsidy: MortgageSubsidyKind::Mcc,
            transfer: MortgageSubsidyTransfer::Sale {
                home_sale_income_id: 0,
                qsml_gain_share: None,
            },
            facts: ordinary_subsidy_facts(),
        },
    }
}
// Bought 2020-01-15 with the subsidized loan, sold 2026-07-01 (the Form
// 8828 disposition date) for $400,000 with $20,000 expenses and a $300,000
// basis: an $80,000 gain, all excluded.
fn subsidized_home() -> MainHomeSale {
    MainHomeSale {
        closed: 20260701,
        acquired: 20200115,
        proceeds: 40_000_000,
        expenses: 2_000_000,
        basis: 30_000_000,
        ..home(false)
    }
}
fn bound_id(d: &FederalDraft) -> i64 {
    d.credits.as_ref().unwrap().mortgage_subsidies[0].home_sale_income_id().unwrap()
}

#[test]
fn form_8828_recapture_on_a_sale_through_both_pages() {
    // Form 8828: line 13 gain $80,000 (the section 121 exclusion does not
    // reduce it); line 14 $40,000; line 15 modified AGI $60,000 (the W-2;
    // the excluded gain is not in AGI); line 16 $50,000; line 17 $10,000;
    // line 18 $10,000 / $5,000 = 100% (capped); line 19 $100,000 x 6.25% =
    // $6,250; line 20 60% -> line 21 $3,750; line 22 $3,750; line 23 the
    // smaller of $40,000 and $3,750: $3,750 to Schedule 2 line 13b.
    let baseline = supported(&complete(FederalStatus::Single));
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_homes(&mut s, vec![subsidized_home()]));
    accepted(&save_other_taxes(
        &mut s,
        vec![OtherTax::MortgageSubsidyRecapture { homes: vec![subsidized_sale()] }],
    ));
    assert_eq!(bound_id(&s.draft), home_id(&s.draft));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "mortgage_subsidy_1_disposition_gain"), 80_000);
    assert_eq!(value(&r, "mortgage_subsidy_1_modified_income"), 60_000);
    assert_eq!(value(&r, "mortgage_subsidy_recapture_tax"), 3_750);
    assert_eq!(r.tax, baseline.tax.map(|t| t + 375_000));
    // Round trip of both pages.
    let stored = s.draft.clone();
    let mut again = restart(stored.clone());
    let page = open_other_taxes(&mut again);
    let P::SaveOtherTaxes { value: shown, .. } = &page else { unreachable!() };
    let request = other_taxes_request(&page, &again.draft.clone(), shown.taxes.clone());
    accepted(&send(&mut again, request));
    assert_eq!(again.draft, stored);
    round_trip(&again, &subsidized_home());
    // A closing date other than the disposition date is named.
    let mut other_day = subsidized_home();
    other_day.closed = 20260630;
    accepted(&save_homes(&mut s, vec![other_day]));
    let r = estimate(&s.draft);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.section == "Mortgage subsidy recapture"), "{r:?}");
}

#[test]
fn form_8828_sale_entered_before_the_home_sale_binds_on_the_next_save() {
    // The Form 8828 row saved first has no home sale (ID 0, named by the
    // estimate); saving the home sale binds it without reopening Other
    // taxes, and removing the home sale unbinds it.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_other_taxes(
        &mut s,
        vec![OtherTax::MortgageSubsidyRecapture { homes: vec![subsidized_sale()] }],
    ));
    assert_eq!(bound_id(&s.draft), 0);
    assert_eq!(estimate(&s.draft).state, ResultState::Unsupported);
    accepted(&save_homes(&mut s, vec![subsidized_home()]));
    assert_eq!(bound_id(&s.draft), home_id(&s.draft));
    assert_eq!(value(&supported(&s.draft), "mortgage_subsidy_recapture_tax"), 3_750);
    accepted(&save_homes(&mut s, vec![]));
    assert_eq!(bound_id(&s.draft), 0);
}
