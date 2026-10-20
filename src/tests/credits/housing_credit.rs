// Form 8586 / Form 8582-CR (unit form-8586): the fixtures of the Form 8586
// map, by hand. Every amount is synthetic. 2026 rate schedules (Rev. Proc.
// 2025-32): single 10% to $12,400, 12% to $50,400, 22% to $105,700, 24% to
// $201,775; joint 10% to $24,800, 12% to $100,800, 22% to $211,400. Standard
// deduction $16,100 single, $32,200 joint. The tax table taxes the middle of
// each $50 row below $100,000. Each housing credit K-1 is a limited partner's
// K-1 (1065) with box 2 = 0 unless a fixture says otherwise.
use super::*;
use alloc::{format, string::ToString};

pub(in crate::tests) fn housing_partner(credits: &[(K1CreditCode, i64)]) -> PartnerK1 {
    let mut data = ordinary_partner_source(0, false);
    data.participation = PartnerParticipation::Limited {
        minutes: 0,
        prior_ten: 0,
        personal_service: false,
        service_years: 0,
    };
    data.credits = credits
        .iter()
        .map(|(code, dollars)| K1Credit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            line: None,
            code: *code,
            amount: dollars * 100,
        })
        .collect();
    // The interest was acquired after 1989 (the K-1's typed confirmation).
    data.housing_confirmed = Some(true);
    data
}

fn with_housing(
    wages: i64,
    status: FederalStatus,
    credits: &[(K1CreditCode, i64)],
) -> FederalDraft {
    let mut d = complete(status);
    d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
    add(&mut d, income(FederalIncome::Partnership { data: housing_partner(credits) }));
    d
}

fn history(carryforwards: Vec<BusinessCreditCarryforward>) -> BusinessCreditHistory {
    BusinessCreditHistory {
        credits: Vec::new(),
        carryforwards,
        carrybacks: vec![],
        spouse_without_credits: false,
    }
}

fn carry(
    credit: BusinessCredit,
    origin_year: u32,
    dollars: i64,
    passive: Option<PassiveWorksheet>,
) -> BusinessCreditCarryforward {
    BusinessCreditCarryforward {
        filer: crate::CarryFiler::SameFilers,
        research_source: None,
        credit,
        origin_year,
        amount: dollars * 100,
        passive,
    }
}

fn refused_with(r: &FederalResult, key: &str) -> bool {
    r.state == ResultState::Unsupported && r.issues.iter().any(|i| i.display.key == key)
}

// Fixture A. Single, W-2 $166,100, code D $4,500.
//   TI = 166,100 - 16,100 = 150,000; tax = 1,240 + 4,560 + 12,166 + 0.24 x 44,300 = 28,598.
//   Form 8582-CR: 3a 4,500, 3c 4,500, 4c 0, 5 4,500, 6 0 (no net passive income), 7 4,500,
//     31 4,500, 33 4,500, 34 4,500; line 35: A 150,000, B 28,598, C 25,000, D 0 (no rental),
//     E 25,000, F 125,000, G tax(125,000) = 17,966 + 0.24 x 19,300 = 22,598, H 6,000;
//     36 = min(4,500, 6,000) = 4,500; 37 = 4,500.
//   Form 3800: l12 28,598; l13 = 25% x 3,598 = 899.5 -> 900; AMTI 166,100 - 90,100 = 76,000,
//     tentative minimum tax 26% x 76,000 = 19,760; l27 = 28,598 - 900 = 27,698; l32 4,500;
//     l33 4,500; l36 4,500; l37 = 4,500; l38 = 4,500. Tax after the credit 24,098.
#[test]
fn fixture_a_the_special_allowance_allows_the_whole_credit() {
    let d = with_housing(
        166_100,
        FederalStatus::Single,
        &[(K1CreditCode::LowIncomeHousingOther, 4_500)],
    );
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_income"), 150_000);
    assert_eq!(value(&r, "regular_income_tax"), 28_598);
    for (id, expected) in [
        ("3a", 4_500),
        ("3b", 0),
        ("3c", 4_500),
        ("4c", 0),
        ("5", 4_500),
        ("6", 0),
        ("7", 4_500),
        ("31", 4_500),
        ("33", 4_500),
        ("34", 4_500),
        ("35", 6_000),
        ("36", 4_500),
        ("37", 4_500),
    ] {
        let id = format!("form8582cr_line_{id}");
        assert_eq!(value(&r, &id), expected, "{id}");
        assert_eq!(printed_line(&r, &id).0, TaxForm::Form8582Cr);
    }
    assert_eq!(value(&r, "form_3800_line_13"), 900);
    assert_eq!(value(&r, "form_3800_line_14"), 19_760);
    assert_eq!(value(&r, "form_3800_line_27"), 27_698);
    assert_eq!(value(&r, "form_3800_line_32"), 4_500);
    assert_eq!(value(&r, "form_3800_line_33"), 4_500);
    assert_eq!(value(&r, "form_3800_line_36"), 4_500);
    assert_eq!(value(&r, "form_3800_line_37"), 4_500);
    assert_eq!(value(&r, "form_3800_line_38"), 4_500);
    assert_eq!(value(&r, "general_business_credit"), 4_500);
    // Form 3800 Part III line 4d column (d), a specified credit.
    let id = "business_credit_low_income_housing_current_year_passive";
    assert_eq!(value(&r, id), 4_500);
    assert_eq!(printed_line(&r, id), (TaxForm::Form3800, "4d".to_string()));
    assert_eq!(value(&r, &format!("{id}_allowed")), 4_500);
    assert_eq!(value(&r, &format!("{id}_to_next_year")), 0);
    // What Form 8582-CR allowed is used under the nonpassive credit's own rows.
    let used = "business_credit_low_income_housing_current_year";
    assert_eq!(value(&r, &format!("{used}_used")), 4_500);
    assert_eq!(value(&r, &format!("{used}_to_next_year")), 0);
    assert!(
        r.lines.iter().all(|l| !l.id.starts_with(&format!("{id}_allowed_"))),
        "no separate passive-allowed use rows"
    );
    assert_eq!(r.tax, Some(24_098 * 100));
}

// Fixture A2 (boundary). The allowance tax H is $6,000: code D $6,000 is all
// allowed; $6,001 allows $6,000 and sends $1 to 2027 as a passive credit.
#[test]
fn fixture_a2_the_credit_stops_at_the_deduction_equivalent_of_the_allowance() {
    let id = "business_credit_low_income_housing_current_year_passive";
    let at = supported(&with_housing(
        166_100,
        FederalStatus::Single,
        &[(K1CreditCode::LowIncomeHousingOther, 6_000)],
    ));
    assert_eq!(value(&at, "form8582cr_line_36"), 6_000);
    assert_eq!(value(&at, &format!("{id}_allowed")), 6_000);
    assert_eq!(value(&at, &format!("{id}_to_next_year")), 0);
    let over = supported(&with_housing(
        166_100,
        FederalStatus::Single,
        &[(K1CreditCode::LowIncomeHousingOther, 6_001)],
    ));
    assert_eq!(value(&over, "form8582cr_line_36"), 6_000);
    assert_eq!(value(&over, "form8582cr_line_37"), 6_000);
    assert_eq!(value(&over, &format!("{id}_allowed")), 6_000);
    assert_eq!(value(&over, &format!("{id}_to_next_year")), 1);
    assert_eq!(value(&over, "general_business_credit"), 6_000);
}

// Fixture A3 (pre-2008 carryforward). As A with a $1,000 nonpassive 2012
// carryforward on Part IV line 2b: Part I line 4 = 1,000, line 6 = 1,000, line 16 =
// 28,598 - 19,760 = 8,838, line 17 = 1,000 (used); Section C line 28 = 1,000, line 29 =
// 27,698 - 1,000 = 26,698, line 37 = 4,500 and line 38 = 5,500.
#[test]
fn fixture_a3_a_pre_2008_carryforward_is_part_i_line_2b() {
    let mut d = with_housing(
        166_100,
        FederalStatus::Single,
        &[(K1CreditCode::LowIncomeHousingOther, 4_500)],
    );
    d.credits.as_mut().unwrap().business_credits =
        Some(history(vec![carry(BusinessCredit::LowIncomeHousingBefore2008, 2012, 1_000, None)]));
    let r = supported(&d);
    let id = "business_credit_low_income_housing_pre2008_2012";
    assert_eq!(value(&r, id), 1_000);
    assert_eq!(printed_line(&r, id), (TaxForm::Form3800, "2b".to_string()));
    assert_eq!(value(&r, &format!("{id}_used")), 1_000);
    assert_eq!(value(&r, "form_3800_line_4"), 1_000);
    assert_eq!(value(&r, "form_3800_line_6"), 1_000);
    assert_eq!(value(&r, "form_3800_line_16"), 8_838);
    assert_eq!(value(&r, "form_3800_line_17"), 1_000);
    assert_eq!(value(&r, "form_3800_line_28"), 1_000);
    assert_eq!(value(&r, "form_3800_line_29"), 26_698);
    assert_eq!(value(&r, "form_3800_line_37"), 4_500);
    assert_eq!(value(&r, "form_3800_line_38"), 5_500);
    assert_eq!(value(&r, "general_business_credit"), 5_500);
    // A pre-2008 carryforward is not specified, and is not a 2026 credit.
    assert!(!BusinessCredit::LowIncomeHousingBefore2008.specified(2012));
    assert!(BusinessCredit::LowIncomeHousing.specified(2026));
}

// Fixture B. The allowance binds: as A with code C $8,000.
//   3c 8,000, 5 8,000, 6 0, 7 8,000, 34 8,000, 35 6,000, 36 6,000, 37 6,000. Form 3800
//   line 33 6,000; the unallowed $2,000 carries to 2027 as a passive credit.
#[test]
fn fixture_b_the_allowance_limits_the_credit_and_the_rest_carries_as_passive() {
    let d =
        with_housing(166_100, FederalStatus::Single, &[(K1CreditCode::LowIncomeHousingJ5, 8_000)]);
    let r = supported(&d);
    for (line, expected) in [
        ("3c", 8_000),
        ("5", 8_000),
        ("6", 0),
        ("7", 8_000),
        ("34", 8_000),
        ("35", 6_000),
        ("36", 6_000),
        ("37", 6_000),
    ] {
        assert_eq!(value(&r, &format!("form8582cr_line_{line}")), expected, "{line}");
    }
    assert_eq!(value(&r, "form_3800_line_33"), 6_000);
    let id = "business_credit_low_income_housing_current_year_passive";
    assert_eq!(value(&r, &format!("{id}_allowed")), 6_000);
    assert_eq!(value(&r, &format!("{id}_to_next_year")), 2_000);
    assert_eq!(value(&r, "business_credit_low_income_housing_current_year_used"), 6_000);
    assert_eq!(value(&r, "general_business_credit"), 6_000);
}

// A general partner's K-1 with box 2 = -$10,000 and active participation.
fn general_partner(box2: i64) -> PartnerK1 {
    let mut data = ordinary_partner_source(0, false);
    data.regular.basis.opening = 5_000_000;
    data.regular.risk.start = ScorpRiskStart::Detailed { capacity: 5_000_000, profit: 0, loss: 0 };
    data.regular.rentals = K1RentalAmounts {
        real_estate_property: Default::default(),
        other_property: Default::default(),
        real_estate: box2 * 100,
        ..Default::default()
    };
    data.rental_facts.participation = K1RentalParticipation::Active;
    data.alternative = Some(data.regular.clone());
    data
}

// Fixture C. A rental loss uses part of the $25,000 first; the tax table
// applies below $100,000. Married filing jointly, W-2 $140,000, a general
// partner's K-1 with box 2 -$10,000 and active participation, a limited
// partner's code D $3,000.
//   Form 8582: MAGI 140,000, allowance 25,000 - 50% x 40,000 = 5,000; line 9 = min(10,000,
//     5,000) = 5,000; $5,000 suspended. AGI 135,000; TI 135,000 - 32,200 = 102,800;
//     B = tax(102,800) = 2,480 + 9,120 + 0.22 x 2,000 = 12,040.
//   Form 8582-CR line 35: E = 25,000 - 5,000 = 20,000 (no phase-out for the credit,
//     section 469(i)(3)(C)); F = 82,800; G = the table tax at 82,800 (middle 82,825) =
//     2,480 + 0.12 x 58,025 = 9,443; H = 12,040 - 9,443 = 2,597. Line 36 = min(3,000, 2,597).
//   Form 3800: l13 0 (12,040 < 25,000); tentative minimum tax 0 (AMTI 135,000 < 140,200
//     exemption); l37 = 2,597; $403 to 2027.
#[test]
fn fixture_c_the_rental_loss_uses_part_of_the_allowance_first() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 14_000_000, 0);
    add(&mut d, income(FederalIncome::Partnership { data: general_partner(-10_000) }));
    add(
        &mut d,
        income(FederalIncome::Partnership {
            data: housing_partner(&[(K1CreditCode::LowIncomeHousingOther, 3_000)]),
        }),
    );
    let r = supported(&d);
    for (line, expected) in [
        ("1d", -10_000),
        ("3", -10_000),
        ("4", 10_000),
        ("5", 150_000),
        ("6", 140_000),
        ("7", 10_000),
        ("8", 5_000),
        ("9", 5_000),
        ("10", 0),
        ("11", 5_000),
    ] {
        let id = format!("form8582_line_{line}");
        assert_eq!(value(&r, &id), expected, "{id}");
        assert_eq!(printed_line(&r, &id).0, TaxForm::Form8582);
    }
    assert_eq!(value(&r, "agi"), 135_000);
    assert_eq!(value(&r, "taxable_income"), 102_800);
    assert_eq!(value(&r, "regular_income_tax"), 12_040);
    assert_eq!(value(&r, "form8582cr_line_35"), 2_597);
    assert_eq!(value(&r, "form8582cr_line_36"), 2_597);
    assert_eq!(value(&r, "form8582cr_line_37"), 2_597);
    assert_eq!(value(&r, "form_3800_line_13"), 0);
    assert_eq!(value(&r, "form_3800_line_14"), 0);
    assert_eq!(value(&r, "form_3800_line_33"), 2_597);
    assert_eq!(value(&r, "form_3800_line_38"), 2_597);
    let id = "business_credit_low_income_housing_current_year_passive";
    assert_eq!(value(&r, &format!("{id}_allowed")), 2_597);
    assert_eq!(value(&r, &format!("{id}_to_next_year")), 403);
}

// Fixture D. Net passive income (line 6) and two credit types (Worksheets
// 7-9). Single, W-2 $156,100; K-1 #1 a limited partner with box 1 $10,000 passive
// profit and code J $1,000 (Worksheet 4); K-1 #2 code D $8,000 (Worksheet 3).
//   TI = 156,100 + 10,000 - 16,100 = 150,000; line 6 = tax(150,000) - tax(140,000) =
//   0.24 x 10,000 = 2,400. 3c 8,000, 4c 1,000, 5 9,000, 7 6,600, 31 6,600, 33 6,600,
//   34 = min(8,000, 6,600) = 6,600; 35 = 28,598 - tax(125,000) 22,598 = 6,000; 36 = 6,000;
//   37 = 2,400 + 6,000 = 8,400; unallowed 600.
//   Worksheet 7: housing column (c) 6,000, column (d) 2,000. Worksheet 8: weights code J
//   1,000 and housing 2,000, so 200 and 400. Worksheet 9: code J allowed 800, housing
//   1,600 + 6,000 = 7,600 (together 8,400). Form 3800: l32 9,000 (both specified), l33 8,400.
#[test]
fn fixture_d_net_passive_income_and_two_worksheets_share_the_unallowed_credit() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 15_610_000, 0);
    let mut fund = housing_partner(&[(K1CreditCode::WorkOpportunity, 1_000)]);
    fund.regular.ordinary = 1_000_000;
    fund.alternative = Some(fund.regular.clone());
    add(&mut d, income(FederalIncome::Partnership { data: fund }));
    add(
        &mut d,
        income(FederalIncome::Partnership {
            data: housing_partner(&[(K1CreditCode::LowIncomeHousingOther, 8_000)]),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_income"), 150_000);
    for (line, expected) in [
        ("3c", 8_000),
        ("4c", 1_000),
        ("5", 9_000),
        ("6", 2_400),
        ("7", 6_600),
        ("31", 6_600),
        ("33", 6_600),
        ("34", 6_600),
        ("35", 6_000),
        ("36", 6_000),
        ("37", 8_400),
    ] {
        let id = format!("form8582cr_line_{line}");
        assert_eq!(value(&r, &id), expected, "{id}");
    }
    assert_eq!(value(&r, "form_3800_line_32"), 9_000);
    assert_eq!(value(&r, "form_3800_line_33"), 8_400);
    assert_eq!(value(&r, "form_3800_line_37"), 8_400);
    assert_eq!(value(&r, "general_business_credit"), 8_400);
    let wotc = "business_credit_work_opportunity_current_year_passive";
    assert_eq!(value(&r, &format!("{wotc}_allowed")), 800);
    assert_eq!(value(&r, &format!("{wotc}_to_next_year")), 200);
    let housing = "business_credit_low_income_housing_current_year_passive";
    assert_eq!(value(&r, &format!("{housing}_allowed")), 7_600);
    assert_eq!(value(&r, &format!("{housing}_to_next_year")), 400);
}

// Fixture E (i). Married filing separately and living together all year: no
// special allowance (section 469(i)(5)), so line 35 and line 36 are 0, the
// credit is not allowed (line 37 = line 6 = 0) and the whole $4,500 carries to
// 2027. Married separate rates equal the single ones below $201,775.
#[test]
fn fixture_e_separate_filers_living_together_get_no_allowance() {
    let d = with_housing(
        166_100,
        FederalStatus::Separate,
        &[(K1CreditCode::LowIncomeHousingOther, 4_500)],
    );
    let r = supported(&d);
    assert_eq!(value(&r, "form8582cr_line_5"), 4_500);
    assert_eq!(value(&r, "form8582cr_line_6"), 0);
    assert_eq!(value(&r, "form8582cr_line_35"), 0);
    assert_eq!(value(&r, "form8582cr_line_36"), 0);
    assert_eq!(value(&r, "form8582cr_line_37"), 0);
    assert_eq!(value(&r, "form_3800_line_33"), 0);
    let id = "business_credit_low_income_housing_current_year_passive";
    assert_eq!(value(&r, &format!("{id}_allowed")), 0);
    assert_eq!(value(&r, &format!("{id}_to_next_year")), 4_500);
    assert_eq!(value(&r, "general_business_credit"), 0);
}

// Fixture E (ii). Married filing separately and living apart all year: the
// allowance is $12,500. TI 150,000; line 35 = 28,598 - tax(137,500) = 0.24 x
// 12,500 = 3,000; line 36 = min(4,500, 3,000) = 3,000: $3,000 allowed, $1,500
// to 2027.
#[test]
fn fixture_e_separate_filers_living_apart_get_12500() {
    let mut d = with_housing(
        166_100,
        FederalStatus::Separate,
        &[(K1CreditCode::LowIncomeHousingOther, 4_500)],
    );
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::AllYear };
    let r = supported(&d);
    assert_eq!(value(&r, "form8582cr_line_35"), 3_000);
    assert_eq!(value(&r, "form8582cr_line_36"), 3_000);
    assert_eq!(value(&r, "form8582cr_line_37"), 3_000);
    let id = "business_credit_low_income_housing_current_year_passive";
    assert_eq!(value(&r, &format!("{id}_allowed")), 3_000);
    assert_eq!(value(&r, &format!("{id}_to_next_year")), 1_500);
}

// Fixture E (iii). A passive credit on a return taxed through Form 8615 or
// Schedule J uses the same tax method for the reduced-income comparison.
#[test]
fn fixture_e_form_8615_and_schedule_j_returns_use_their_own_reduced_tax() {
    let key = "ISSUE_FORM_8582_CR_NEEDS_THE_FORM_8615_OR_SCHEDULE_J_TAX";
    let credit = housing_partner(&[(K1CreditCode::LowIncomeHousingOther, 4_500)]);
    let mut child = young_interest_return();
    add(&mut child, income(FederalIncome::Partnership { data: credit.clone() }));
    let child_result = supported(&child);
    assert!(!refused_with(&child_result, key));
    // Parent's22% share of7300 is1606; child's1350–1375 TaxTable row is136.
    assert_eq!(value(&child_result, "form8582cr_line_35"), 1_742);
    assert_eq!(value(&child_result, "general_business_credit"), 1_742);
    let mut averaged = averaging_return(60_000);
    add(&mut averaged, income(FederalIncome::Partnership { data: credit.clone() }));
    let averaged_result = supported(&averaged);
    assert!(!refused_with(&averaged_result, key));
    // TI118248, unchanged net election60000 and base-year delta7101:
    // current-year tax at58248 is7522; at33248 after25000 allowance is3739.
    // Both comparisons keep the same base delta:14623−10840=3783.
    assert_eq!(value(&averaged_result, "regular_income_tax"), 14623);
    assert_eq!(value(&averaged_result, "form8582cr_line_37"), 3_783);
    assert_eq!(value(&averaged_result, "general_business_credit"), 3_783);
    let plain = young_interest_return();
    assert!(!refused_with(&estimate(&plain), key));
    assert!(supported(&averaging_return(60_000)).tax.is_some());
}

// Fixture E (iv). Codes C and D on one K-1 are two entries merged on Form
// 3800 line 4d; the same code twice is refused on both sides.
#[test]
fn fixture_e_codes_c_and_d_merge_on_line_4d_and_each_code_once() {
    let d = with_housing(
        166_100,
        FederalStatus::Single,
        &[(K1CreditCode::LowIncomeHousingJ5, 2_000), (K1CreditCode::LowIncomeHousingOther, 3_000)],
    );
    let r = supported(&d);
    assert_eq!(value(&r, "business_credit_low_income_housing_current_year_passive"), 5_000);
    assert_eq!(value(&r, "form8582cr_line_3a"), 5_000);
    assert_eq!(value(&r, "form8582cr_line_37"), 5_000);
    let twice = [
        K1Credit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            line: None,
            code: K1CreditCode::LowIncomeHousingOther,
            amount: 100,
        },
        K1Credit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            line: None,
            code: K1CreditCode::LowIncomeHousingOther,
            amount: 200,
        },
    ];
    assert!(!K1Credit::valid_list(&twice));
    assert_eq!(K1CreditCode::LowIncomeHousingJ5.credit(), BusinessCredit::LowIncomeHousing);
    assert_eq!(K1CreditCode::LowIncomeHousingOther.credit(), BusinessCredit::LowIncomeHousing);
    // All nine codes fit the list bound, each once.
    let all = [
        K1CreditCode::WorkOpportunity,
        K1CreditCode::EmployerTipFica,
        K1CreditCode::PensionStartup,
        K1CreditCode::AutoEnrollment,
        K1CreditCode::MilitarySpouse,
        K1CreditCode::SmallEmployerHealth,
        K1CreditCode::PaidLeave,
        K1CreditCode::LowIncomeHousingJ5,
        K1CreditCode::LowIncomeHousingOther,
    ];
    let list: Vec<K1Credit> = all
        .iter()
        .map(|code| K1Credit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            line: None,
            code: *code,
            amount: 100,
        })
        .collect();
    assert!(K1Credit::valid_list(&list));
}

// Fixture E (v). The carryover rules: a pre-2008 housing credit has no 2026
// amount to carry back; a housing carryforward cannot come from before 2008;
// a worksheet must fit its credit.
#[test]
fn fixture_e_carryover_rules_of_the_housing_credits() {
    let carryback = BusinessCreditHistory {
        credits: Vec::new(),
        carryforwards: vec![],
        carrybacks: vec![BusinessCreditCarryback {
            research_source: None,
            credit: BusinessCredit::LowIncomeHousingBefore2008,
            amount: 10_000,
            year: 2025,
        }],
        spouse_without_credits: false,
    };
    assert_eq!(carryback.rule(), Some("BUSINESS_CREDIT_CARRYFORWARD_ONLY"));
    let early = history(vec![carry(BusinessCredit::LowIncomeHousing, 2007, 1_000, None)]);
    assert_eq!(early.rule(), Some("BUSINESS_CREDIT_BEFORE_ITS_FIRST_YEAR"));
    let fine = history(vec![carry(BusinessCredit::LowIncomeHousing, 2008, 1_000, None)]);
    assert_eq!(fine.rule(), None);
    let before = history(vec![carry(
        BusinessCredit::LowIncomeHousingBefore2008,
        2006,
        1_000,
        Some(PassiveWorksheet::LowIncomeHousing),
    )]);
    assert_eq!(before.rule(), None);
    let wrong = history(vec![carry(
        BusinessCredit::LowIncomeHousing,
        2020,
        1_000,
        Some(PassiveWorksheet::Other),
    )]);
    assert_eq!(wrong.rule(), Some("BUSINESS_CREDIT_WORKSHEET_MISMATCH"));
    let wrong = history(vec![carry(
        BusinessCredit::WorkOpportunity,
        2020,
        1_000,
        Some(PassiveWorksheet::LowIncomeHousing),
    )]);
    assert_eq!(wrong.rule(), Some("BUSINESS_CREDIT_WORKSHEET_MISMATCH"));
    // A prior-year unallowed housing credit joins the 2026 credit on
    // Worksheet 3 (line 3b), and both use the allowance.
    let mut d = with_housing(
        166_100,
        FederalStatus::Single,
        &[(K1CreditCode::LowIncomeHousingOther, 2_000)],
    );
    d.credits.as_mut().unwrap().business_credits = Some(history(vec![carry(
        BusinessCredit::LowIncomeHousing,
        2024,
        3_000,
        Some(PassiveWorksheet::LowIncomeHousing),
    )]));
    let r = supported(&d);
    assert_eq!(value(&r, "form8582cr_line_3a"), 2_000);
    assert_eq!(value(&r, "form8582cr_line_3b"), 3_000);
    assert_eq!(value(&r, "form8582cr_line_3c"), 5_000);
    assert_eq!(value(&r, "form8582cr_line_36"), 5_000);
    let carried = "business_credit_low_income_housing_2024_passive";
    assert_eq!(printed_line(&r, carried), (TaxForm::Form3800, "4d".to_string()));
    assert_eq!(value(&r, &format!("{carried}_allowed")), 3_000);
    assert_eq!(value(&r, &format!("{carried}_to_next_year")), 0);
    assert_eq!(value(&r, "general_business_credit"), 5_000);
}

// Fixture F. One activity: a limited partner's K-1 with box 2 -$30,000 and
// code D $4,500. The box 2 loss is suspended whole (a limited partner never
// actively participates), so Form 8582 line 9 is 0 and the credit keeps the
// full $25,000 allowance: as fixture A, all $4,500 allowed. Net passive income
// is negative, so line 6 is 0.
#[test]
fn fixture_f_a_k1_with_a_box_2_loss_and_a_credit_is_one_activity() {
    let mut data = housing_partner(&[(K1CreditCode::LowIncomeHousingOther, 4_500)]);
    data.regular.basis.opening = 5_000_000;
    data.regular.risk.start = ScorpRiskStart::Detailed { capacity: 5_000_000, profit: 0, loss: 0 };
    data.regular.rentals = K1RentalAmounts {
        real_estate_property: Default::default(),
        other_property: Default::default(),
        real_estate: -3_000_000,
        ..Default::default()
    };
    data.alternative = Some(data.regular.clone());
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 16_610_000, 0);
    add(&mut d, income(FederalIncome::Partnership { data: data.clone() }));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 166_100);
    assert_eq!(value(&r, "form8582_line_3"), -30_000);
    assert_eq!(value(&r, "form8582_line_11"), 0);
    assert!(r.lines.iter().all(|l| l.id != "form8582_line_9"));
    assert_eq!(value(&r, "form8582cr_line_6"), 0);
    assert_eq!(value(&r, "form8582cr_line_35"), 6_000);
    assert_eq!(value(&r, "form8582cr_line_37"), 4_500);
    assert_eq!(value(&r, "business_credit_low_income_housing_current_year_passive_allowed"), 4_500);
    // A real estate professional who materially participated has a
    // nonpassive activity: the loss reduces AGI and the credit skips Form
    // 8582-CR (Part III column (e)).
    data.rental_facts.participation = K1RentalParticipation::Professional;
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 16_610_000, 0);
    add(&mut d, income(FederalIncome::Partnership { data }));
    make_professional(&mut d, Owner::Taxpayer, false);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 136_100);
    assert!(r.lines.iter().all(|l| l.id != "form8582cr_line_5"));
    let id = "business_credit_low_income_housing_current_year";
    assert_eq!(value(&r, id), 4_500);
    assert_eq!(printed_line(&r, id), (TaxForm::Form3800, "4d".to_string()));
    assert_eq!(value(&r, &format!("{id}_used")), 4_500);
    assert_eq!(value(&r, "form_3800_line_30"), 4_500);
    assert_eq!(value(&r, "general_business_credit"), 4_500);
}

// Form 8582 rows exist only for a return with passive activities, and only
// the regular pass names them.
#[test]
fn form_8582_rows_need_a_passive_activity() {
    let r = supported(&complete(FederalStatus::Single));
    assert!(r.lines.iter().all(|l| !l.id.starts_with("form8582_line_")));
    assert!(r.lines.iter().all(|l| !l.id.starts_with("form8582cr_line_")));
}

// Every housing and other business credit has its row labels, so a calculation
// row of any credit renders.
#[test]
fn every_business_credit_row_has_its_labels() {
    for credit in BusinessCredit::ALL {
        let key = credit.key();
        for suffix in [
            "2026",
            "2026_used",
            "2026_to_next_year",
            "2026_passive",
            "2026_passive_to_next_year",
            "2026_passive_allowed",
        ] {
            let id = format!("business_credit_{key}_{suffix}");
            crate::line_label(&id);
            crate::FormLine::new(TaxForm::Form3800, "4d").row_label(&id);
        }
    }
}

// Fixture G. The part Form 8582-CR allows is used like a nonpassive credit:
// FIFO with the other credits of its year, a 2025 carryback, then 2027. Single,
// W-2 $60,000: TI 43,900, tax 5,023 (table row 43,900-43,950). Code D $3,000 is
// allowed in full (line 35 = 5,023 - tax(18,900) 2,023 = 3,000). A materially
// participating shareholder's code J $4,000 is used first (credit order: work
// opportunity, then housing); housing is used 5,023 - 4,000 = 1,023. Of the
// 3,000 - 1,023 = 1,977 left, $1,000 is carried back and $977 goes to 2027.
#[test]
fn fixture_g_the_allowed_part_is_used_carried_back_and_carried_forward() {
    let mut d = with_housing(
        60_000,
        FederalStatus::Single,
        &[(K1CreditCode::LowIncomeHousingOther, 3_000)],
    );
    let mut source = complete_ordinary_scorp(0, true);
    source.credits = vec![K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        line: None,
        code: K1CreditCode::WorkOpportunity,
        amount: 400_000,
    }];
    add(&mut d, income(FederalIncome::Scorp { data: source }));
    d.credits.as_mut().unwrap().business_credits = Some(BusinessCreditHistory {
        credits: Vec::new(),
        carryforwards: vec![],
        carrybacks: vec![BusinessCreditCarryback {
            research_source: None,
            credit: BusinessCredit::LowIncomeHousing,
            amount: 100_000,
            year: 2025,
        }],
        spouse_without_credits: false,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "form8582cr_line_37"), 3_000);
    assert_eq!(value(&r, "form_3800_line_30"), 4_000);
    assert_eq!(value(&r, "form_3800_line_33"), 3_000);
    assert_eq!(value(&r, "form_3800_line_37"), 5_023);
    assert_eq!(value(&r, "business_credit_low_income_housing_current_year_passive_allowed"), 3_000);
    let id = "business_credit_low_income_housing_current_year";
    assert_eq!(value(&r, &format!("{id}_used")), 1_023);
    assert_eq!(value(&r, &format!("{id}_carryback_2025")), 1_000);
    assert_eq!(value(&r, &format!("{id}_to_next_year")), 977);
    assert_eq!(
        value(&r, "business_credit_low_income_housing_current_year_passive_to_next_year"),
        0
    );
    // A carryback above what is left is refused by name.
    d.credits.as_mut().unwrap().business_credits.as_mut().unwrap().carrybacks[0].amount = 200_000;
    assert!(refused_with(&estimate(&d), "ISSUE_CREDIT_CARRYBACK_LIMIT"));
}

// The reduced-income tax is needed only for line 6 (net passive income) and
// line 35 (a housing credit). A Worksheet 4 credit on a Schedule J return with
// no net passive income is simply not allowed, as before this unit.
#[test]
fn a_passive_credit_needs_the_reduced_tax_only_where_a_line_uses_it() {
    let key = "ISSUE_FORM_8582_CR_NEEDS_THE_FORM_8615_OR_SCHEDULE_J_TAX";
    let mut d = averaging_return(60_000);
    add(
        &mut d,
        income(FederalIncome::Partnership {
            data: housing_partner(&[(K1CreditCode::WorkOpportunity, 1_000)]),
        }),
    );
    let r = estimate(&d);
    assert!(!refused_with(&r, key), "{:?}", r.issues);
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope);
    assert_eq!(
        value(&r, "business_credit_work_opportunity_current_year_passive_to_next_year"),
        1_000
    );
}

// The reduced taxable income of lines 6 and 35 may fall below the return's
// preferential income (qualified dividends and gains stack on top): the
// worksheet then taxes only what is left, as if the preferential income were
// capped at the reduced taxable income.
#[test]
fn reduced_taxable_income_below_the_preferential_income_is_capped() {
    use crate::income_tax::{Preference, RegularTaxWorksheet};
    let worksheet = |taxable, dividends| RegularTaxWorksheet {
        taxable,
        preference: Preference { dividends, gain: 0, line18: 0, line19: 0 },
        excluded: 0,
        foreign_election: false,
        status: FederalStatus::Single,
    };
    assert_eq!(worksheet(5_000, 28_000).tax(), worksheet(5_000, 5_000).tax());
    assert_eq!(worksheet(48_000, 60_000).tax(), worksheet(48_000, 48_000).tax());
}

// F3 (2026-10-09), Form 8582-CR Part III: an interest acquired before 1990
// (the K-1's typed confirmation answered No) has its own phase-out, which is
// not calculated, so its housing credit is left out with one notice. Yes
// keeps the post-1989 credit.
#[test]
fn a_pre_1990_interest_uses_the_part_three_allowance() {
    let credits = [(K1CreditCode::LowIncomeHousingOther, 3_000)];
    let kept = with_housing(60_000, FederalStatus::Single, &credits);
    let r_kept = supported(&kept);
    assert_eq!(value(&r_kept, "general_business_credit"), 3_000);
    assert!(!noted(&r_kept, "WARN_K1_HOUSING_CREDIT_LEFT_OUT"));
    let mut pre_1990 = kept.clone();
    let FederalIncome::Partnership { data } = &mut pre_1990.income[1].income else {
        unreachable!()
    };
    data.housing_confirmed = Some(false);
    let r = supported(&pre_1990);
    assert!(!noted(&r, "WARN_K1_HOUSING_CREDIT_LEFT_OUT"));
    assert_eq!(value(&r, "form8582cr_line_2a"), 3_000);
    assert_eq!(value(&r, "form8582cr_line_30"), 3_000);
    assert_eq!(value(&r, "general_business_credit"), 3_000);
    // The credit changes tax while the income inputs remain the same.
    let mut none = kept.clone();
    let FederalIncome::Partnership { data } = &mut none.income[1].income else { unreachable!() };
    data.credits.clear();
    data.housing_confirmed = None;
    assert_eq!(value(&r, "agi"), value(&supported(&none), "agi"));
    // An unanswered confirmation with a housing credit is not a valid record.
    let mut unanswered = kept.clone();
    let FederalIncome::Partnership { data } = &mut unanswered.income[1].income else {
        unreachable!()
    };
    data.housing_confirmed = None;
    assert!(!data.valid());
}
