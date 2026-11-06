use crate::tests::support::assert::line_amount as cents_of;
// Forms 3468 and 4255 (unit form-3468): the fixtures of the Form 3468 / 4255
// map, by hand. Every amount is synthetic. 2026 rate schedules (Rev. Proc.
// 2025-32): single 10% to $12,400, 12% to $50,400, 22% to $105,700, 24% to
// $201,775; standard deduction $16,100. Amounts in these helpers are whole
// dollars unless a name says cents.
use super::*;
use crate::general_business_credit::{
    CreditEntry, PassiveCreditCapacity, RehabilitationAllowance, TaxCapacity,
};
use alloc::format;

fn dollars_of(amount: i64) -> i64 {
    amount * 100
}

fn year(year: u32, used: i64, carried: i64) -> CreditYear {
    CreditYear { year, used: dollars_of(used), carried: dollars_of(carried) }
}

// A 39-year building row holding only the qualified rehabilitation
// expenditures (the row's cost).
fn building_row(
    cost: i64,
    placed: u32,
    history: EquipmentHistory,
    claim: Option<RehabilitationCredit>,
    disposal: crate::AssetDisposal,
) -> BusinessAsset {
    BusinessAsset {
        cost: dollars_of(cost),
        acquired: placed,
        placed,
        class: EquipmentClass::ThirtyNine,
        history,
        method: EquipmentMethod::StraightLine,
        disposal,
        investment_credit: claim.map_or(InvestmentCredit::No, InvestmentCredit::Rehabilitation),
        ..bonus_equipment(0, 100)
    }
}

fn new_building(cost: i64, placed: u32, claim: Option<RehabilitationCredit>) -> BusinessAsset {
    building_row(
        cost,
        placed,
        EquipmentHistory::New {
            section179: Section179::Ineligible,
            bonus: BonusElection::ElectOut,
            tables: true,
        },
        claim,
        crate::AssetDisposal::No,
    )
}

fn continuing_building(
    cost: i64,
    placed: u32,
    prior_depreciation_cents: i64,
    claim: Option<RehabilitationCredit>,
    disposal: crate::AssetDisposal,
) -> BusinessAsset {
    building_row(
        cost,
        placed,
        EquipmentHistory::Continuing {
            initial_section179: 0,
            carryforward: 0,
            prior_depreciation: prior_depreciation_cents,
            amt: EquipmentAmtHistory::Same,
            mid_quarter: false,
            schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
        },
        claim,
        disposal,
    )
}

fn sale(date: u32, proceeds: i64) -> crate::AssetDisposal {
    crate::AssetDisposal::Disposed {
        date,
        how: crate::DisposalKind::Sold {
            livestock: crate::SoldLivestock::No,
            proceeds: dollars_of(proceeds),
            selling_expenses: 0,
            payment: crate::SalePayment::InFull,
            buyer: crate::Buyer::Unrelated,
        },
    }
}

fn no_lookback() -> Section1231History {
    Section1231History { regular: vec![], amt: vec![] }
}

// A Schedule C business (receipts $100,000) with one asset row. Its id is 2.
pub(in crate::tests) fn business_return(asset: BusinessAsset) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    let mut business = cash_business(100_000);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.assets.push(asset);
    add(&mut d, income(business));
    d.screening.as_mut().unwrap().section1231 = Some(no_lookback());
    d
}

fn carryforward(
    credit: BusinessCredit,
    origin_year: u32,
    amount: i64,
    passive: Option<PassiveWorksheet>,
) -> BusinessCreditCarryforward {
    BusinessCreditCarryforward {
        filer: crate::CarryFiler::SameFilers,
        research_source: None,
        credit,
        origin_year,
        amount: dollars_of(amount),
        passive,
    }
}

fn history(
    carryforwards: Vec<BusinessCreditCarryforward>,
    carrybacks: Vec<BusinessCreditCarryback>,
) -> BusinessCreditHistory {
    BusinessCreditHistory {
        credits: Vec::new(),
        carryforwards,
        carrybacks,
        spouse_without_credits: false,
    }
}

fn has(r: &FederalResult, id: &str) -> bool {
    r.lines.iter().any(|l| l.id == id)
}

// ---- Fixture A: the Form 3800 limits of an investment credit ------------------------------------

fn capacity() -> TaxCapacity {
    TaxCapacity {
        research_numerators: Vec::new(),
        taxable_income: 0,
        separate_ratios: None,
        status: FederalStatus::Single,
        spouse_without_credits: false,
        regular_tax_before_credits: 14_000,
        alternative_minimum_tax: 0,
        tentative_minimum_tax: 11_000,
        foreign_tax_credit: 0,
        other_nonrefundable_credits: 0,
        passive: PassiveCreditCapacity {
            active: crate::general_business_credit::RehabilitationAllowance::default(),
            net_passive_income_tax: 0,
            remaining_allowance: 0,
            remaining_allowance_tax: 0,
            rehabilitation: RehabilitationAllowance::default(),
        },
    }
}

fn entry(credit: BusinessCredit, amount: i64) -> CreditEntry {
    CreditEntry {
        source_owner: None,
        conditional_origin: None,
        transfer: 0,
        elective: 0,
        activity: None,
        filer: crate::CarryFiler::SameFilers,
        credit,
        year: 2026,
        amount,
        passive: None,
    }
}

fn form_3800(entries: Vec<CreditEntry>) -> FederalResult {
    let mut r = FederalResult::empty(1);
    crate::general_business_credit::calculate(None, entries, &[], capacity(), &mut r);
    r
}

// A K-1 (1065) box 20 code D credit of $20,000: a specified credit, so only
// the line 27 limit applies, not the tentative minimum tax floor of line 16.
//   Regular tax before credits $14,000, AMT 0, tentative minimum tax $11,000.
//   l7 = l9 = l11 = l12 = 14,000; l13 = 25% x max(0, 14,000 - 25,000) = 0;
//   l15 = max(0, 11,000) = 11,000; l16 = 3,000; l17 = 0 (no Part I credit);
//   l27 = 14,000 - 0; l29 = 14,000; l30 = 20,000; l36 = 20,000;
//   l37 = min(14,000, 20,000) = 14,000; l38 = 14,000. Used 14,000, 6,000 to 2027.
#[test]
fn fixture_a_a_specified_rehabilitation_credit_is_limited_only_by_line_27() {
    let r = form_3800(vec![entry(BusinessCredit::Rehabilitation, 20_000)]);
    for (id, amount) in [
        ("form_3800_line_1", 0),
        ("form_3800_line_6", 0),
        ("form_3800_line_7", 14_000),
        ("form_3800_line_11", 14_000),
        ("form_3800_line_13", 0),
        ("form_3800_line_14", 11_000),
        ("form_3800_line_15", 11_000),
        ("form_3800_line_16", 3_000),
        ("form_3800_line_17", 0),
        ("form_3800_line_27", 14_000),
        ("form_3800_line_29", 14_000),
        ("form_3800_line_30", 20_000),
        ("form_3800_line_36", 20_000),
        ("form_3800_line_37", 14_000),
        ("form_3800_line_38", 14_000),
        ("business_credit_rehabilitation_current_year_used", 14_000),
        ("business_credit_rehabilitation_current_year_to_next_year", 6_000),
    ] {
        assert_eq!(value(&r, id), amount, "{id}");
    }
    // Part III line 4k.
    assert_eq!(printed_line(&r, "business_credit_rehabilitation_current_year").1, "4k");
}

// A2: the same $20,000 as a clean electricity investment credit (K-1 code
// AV, not a specified credit) is Part I and stops at line 16 = $3,000: the
// tentative minimum tax floor binds. l17 = 3,000; l28 = 3,000; l29 = 11,000;
// l30 = 0; l38 = 3,000; $17,000 to 2027.
#[test]
fn fixture_a2_a_clean_electricity_credit_is_stuck_at_the_minimum_tax_floor() {
    let r = form_3800(vec![entry(BusinessCredit::CleanElectricity, 20_000)]);
    for (id, amount) in [
        ("form_3800_line_1", 20_000),
        ("form_3800_line_6", 20_000),
        ("form_3800_line_16", 3_000),
        ("form_3800_line_17", 3_000),
        ("form_3800_line_27", 14_000),
        ("form_3800_line_28", 3_000),
        ("form_3800_line_29", 11_000),
        ("form_3800_line_30", 0),
        ("form_3800_line_38", 3_000),
        ("business_credit_clean_electricity_current_year_used", 3_000),
        ("business_credit_clean_electricity_current_year_to_next_year", 17_000),
    ] {
        assert_eq!(value(&r, id), amount, "{id}");
    }
    assert_eq!(printed_line(&r, "business_credit_clean_electricity_current_year").1, "1v");
}

// A3: both items. Part I takes 3,000 first; line 29 = 14,000 - 3,000 = 11,000
// limits the specified rehabilitation credit: l37 = min(11,000, 20,000) =
// 11,000; l38 = 14,000. Rehabilitation used 11,000 and 9,000 carried.
#[test]
fn fixture_a3_the_clean_electricity_credit_is_used_before_the_specified_credit() {
    let r = form_3800(vec![
        entry(BusinessCredit::Rehabilitation, 20_000),
        entry(BusinessCredit::CleanElectricity, 20_000),
    ]);
    for (id, amount) in [
        ("form_3800_line_17", 3_000),
        ("form_3800_line_29", 11_000),
        ("form_3800_line_30", 20_000),
        ("form_3800_line_37", 11_000),
        ("form_3800_line_38", 14_000),
        ("business_credit_rehabilitation_current_year_used", 11_000),
        ("business_credit_rehabilitation_current_year_to_next_year", 9_000),
        ("business_credit_clean_electricity_current_year_used", 3_000),
        ("business_credit_clean_electricity_current_year_to_next_year", 17_000),
    ] {
        assert_eq!(value(&r, id), amount, "{id}");
    }
}

// The energy credit is specified (line 4a): the same limits as the
// rehabilitation credit. The 2026 investment credits come first in the
// instructions' order of use: ahead of a Form 5884 credit of the same year.
#[test]
fn the_investment_credits_are_used_first_within_a_year() {
    let r = form_3800(vec![
        entry(BusinessCredit::WorkOpportunity, 5_000),
        entry(BusinessCredit::Energy, 6_000),
    ]);
    assert_eq!(value(&r, "business_credit_energy_current_year_used"), 6_000);
    assert_eq!(value(&r, "business_credit_work_opportunity_current_year_used"), 5_000);
    assert_eq!(printed_line(&r, "business_credit_energy_current_year").1, "4a");
    let r = form_3800(vec![
        entry(BusinessCredit::WorkOpportunity, 9_000),
        entry(BusinessCredit::Energy, 9_000),
    ]);
    assert_eq!(value(&r, "business_credit_energy_current_year_used"), 9_000);
    assert_eq!(value(&r, "business_credit_work_opportunity_current_year_used"), 5_000);
}

// ---- Fixture B: a direct rehabilitation credit with its basis reduction -------------------------

// Schedule C building row: cost $300,000 (all qualified rehabilitation
// expenditures), placed in service April 15, 2026, 39-year mid-month.
//   Measuring period: 2024-05-01 + 24 months - 1 day = 2026-04-30, in 2026.
//   Substantial: 300,000 > max(200,000, 5,000). Line 1g = 300,000; line 1j =
//   4% x 300,000 = 12,000; line 3 = 12,000 (Form 3800 line 4k).
//   Credit determined 20% x 300,000 = 60,000 = the basis reduction;
//   depreciable basis 240,000. April first-year 1.819%: 240,000 x 1.819% =
//   4,365.60 (5,457.00 without the credit).
fn fixture_b_claim(expenditures: i64, start: u32) -> RehabilitationCredit {
    rehabilitation(start, 200_000, expenditures, vec![], true)
}

#[test]
fn fixture_b_a_direct_rehabilitation_credit_reduces_the_depreciable_basis() {
    let d =
        business_return(new_building(300_000, 20260415, Some(fixture_b_claim(300_000, 20240501))));
    let r = supported(&d);
    for (id, amount) in [
        ("business_2_asset_0_form3468_building_basis", 200_000),
        ("business_2_asset_0_form3468_period_expenditures", 300_000),
        ("business_2_asset_0_form3468_basis", 300_000),
        ("business_2_asset_0_form3468_credit", 12_000),
        ("business_2_asset_0_basis_reduction", 60_000),
    ] {
        assert_eq!(value(&r, id), amount, "{id}");
    }
    assert_eq!(cents_of(&r, "business_2_asset_0_macrs"), 436_560);
    assert_eq!(
        printed_line(&r, "business_2_asset_0_form3468_credit"),
        (TaxForm::Form3468, "3".into())
    );
    assert_eq!(printed_line(&r, "business_2_asset_0_form3468_basis").1, "1g");
    // Form 3800 Part III line 4k, a specified credit; the return's tax is
    // well above $12,000, so all of it is used.
    assert_eq!(value(&r, "business_credit_rehabilitation_current_year"), 12_000);
    assert_eq!(printed_line(&r, "business_credit_rehabilitation_current_year").1, "4k");
    assert_eq!(value(&r, "form_3800_line_30"), 12_000);
    assert_eq!(value(&r, "business_credit_rehabilitation_current_year_used"), 12_000);
    assert_eq!(value(&r, "general_business_credit"), 12_000);
    // No recapture rows without a disposal.
    assert!(!has(&r, "business_2_asset_0_form4255_recapture"));
    assert!(!has(&r, "investment_credit_recapture_tax"));
    // The page reopens with the record.
    assert_eq!(supported(&restart(d.clone()).draft), r);
}

// B2/B3: the substantial-rehabilitation test and the measuring period.
#[test]
fn fixture_b2_the_rehabilitation_must_be_substantial_and_end_in_the_service_year() {
    let rule = |start: u32, expenditures: i64, placed: u32, basis: i64| {
        let asset = new_building(
            300_000,
            placed,
            Some(rehabilitation(start, basis, expenditures, vec![], true)),
        );
        let InvestmentCredit::Rehabilitation(claim) = &asset.investment_credit else {
            unreachable!()
        };
        claim.rule(asset.placed, asset.credit_cost())
    };
    // 190,000 does not exceed the building's 200,000 basis.
    assert_eq!(rule(20240501, 190_000, 20260415, 200_000), Some("REHABILITATION_NOT_SUBSTANTIAL"));
    // Equal to the greater of the basis and $5,000 is not enough either.
    assert_eq!(rule(20240501, 200_000, 20260415, 200_000), Some("REHABILITATION_NOT_SUBSTANTIAL"));
    assert_eq!(rule(20240501, 200_001, 20260415, 200_000), None);
    // Below the $5,000 floor when the basis is zero.
    assert_eq!(rule(20240501, 5_000, 20260415, 0), Some("REHABILITATION_NOT_SUBSTANTIAL"));
    // 2024-06-01 ends 2026-05-31, in 2026; 2023-12-01 ends 2025-11-30.
    assert_eq!(rule(20240601, 300_000, 20260415, 200_000), None);
    assert_eq!(
        rule(20231201, 300_000, 20260415, 200_000),
        Some("REHABILITATION_PERIOD_NOT_IN_SERVICE_YEAR")
    );
    // A phased rehabilitation measures 60 months: 2021-06-01 ends 2026-05-31.
    let mut asset = new_building(
        300_000,
        20260415,
        Some(rehabilitation(20210601, 200_000, 300_000, vec![], true)),
    );
    let InvestmentCredit::Rehabilitation(claim) = &mut asset.investment_credit else {
        unreachable!()
    };
    assert_eq!(
        claim.rule(asset.placed, dollars_of(300_000)),
        Some("REHABILITATION_PERIOD_NOT_IN_SERVICE_YEAR")
    );
    claim.phased = true;
    assert_eq!(claim.rule(asset.placed, dollars_of(300_000)), None);
}

// B4: the eligibility not confirmed leaves the 2026 credit out with a notice:
// the building is depreciated on its full cost, $5,457.00.
#[test]
fn fixture_b4_an_unconfirmed_rehabilitation_is_left_out_with_its_basis_reduction() {
    let mut claim = fixture_b_claim(300_000, 20240501);
    claim.confirmed = false;
    let d = business_return(new_building(300_000, 20260415, Some(claim)));
    let r = supported(&d);
    assert!(noted(&r, "WARN_REHABILITATION_CREDIT_LEFT_OUT"));
    assert_eq!(cents_of(&r, "business_2_asset_0_macrs"), 545_700);
    assert!(!has(&r, "business_2_asset_0_form3468_credit"));
    assert!(!has(&r, "business_credit_rehabilitation_current_year"));
}

// The rehabilitation credit of a building placed in service in 2022 is the
// fifth part of a five-year credit: its 2026 share is year 5 of 5; 2027 is
// outside the period.
#[test]
fn the_rehabilitation_credit_period_is_five_years_from_the_year_placed_in_service() {
    // 200,000 x 4% = 8,000 for 2022 through 2026: 2026 is the fifth year.
    let claim = rehabilitation(
        20200901,
        100_000,
        200_000,
        vec![
            year(2022, 8_000, 0),
            year(2023, 8_000, 0),
            year(2024, 8_000, 0),
            year(2025, 8_000, 0),
        ],
        true,
    );
    let asset =
        continuing_building(200_000, 20221215, 1_000_000, Some(claim), crate::AssetDisposal::No);
    assert_eq!(asset.current_year_investment_credit(), 8_000);
    // Placed in service in 2021: 2026 is a sixth year, no credit.
    let claim = rehabilitation(20190901, 100_000, 200_000, vec![], true);
    let asset =
        continuing_building(200_000, 20211215, 1_000_000, Some(claim), crate::AssetDisposal::No);
    assert_eq!(asset.current_year_investment_credit(), 0);
}

// ---- Fixture C: recapture in year 3 ----------------------------------------------------------

// Schedule C building row: cost $200,000 (the expenditures), placed March
// 10, 2024, sold June 30, 2026 for $210,000. Credit determined 20% x 200,000
// = 40,000 (the basis reduction): depreciable basis 160,000. History:
// 2024 160,000 x 2.033% = 3,252.80; 2025 160,000 x 2.564% = 4,102.40; prior
// depreciation 7,355.20.
//   Form 4255 Part II: B6 = 8,000 + 8,000 = 16,000; D10 03/01/2024; D11
//   06/30/2026; D12 = 2 full years; D13 0; D14 16,000; D15 60%; D16 9,600.
//   Part I row 1m: (a) 16,000, g 60, h 9,600, i 0, j 9,600, r
//   9,600 -> Schedule 2 line 13a 9,600.
//   2026 share (Reg. 1.47-7(e)(3)): line 1g = 200,000 x 40% = 80,000;
//   1j = 3,200 to Form 3800 line 4k.
//   Basis increase (section 50(c)(2)) 9,600: net reduction 30,400.
//   2026 depreciation 4,102.40 x 11/24 = 1,880.27 (on the basis before the
//   increase); depreciation allowed 3,252.80 + 4,102.40 + 1,880.27 = 9,235.47.
//   Form 4797: line 22 210,000; line 23 200,000; line 24 = 9,235.47 +
//   30,400 = 39,635.47 -> 39,635; line 25 160,365; line 26 gain 49,635.
//   Straight line on the unreduced 200,000: 4,066.00 + 5,128.00 + 5,128.00 x
//   11/24 (2,350.33) = 11,544.33 -> 11,544; line 28a = 39,635 - 11,544 =
//   28,091 ordinary section 1250; section 1231 gain 49,635 - 28,091 =
//   21,544; unrecaptured section 1250 gain min(21,544, 39,635 - 28,091 =
//   11,544) = 11,544.
fn recapture_claim(freed: i64, last: (i64, i64)) -> RehabilitationCredit {
    let mut claim = rehabilitation(
        20220401,
        150_000,
        200_000,
        vec![year(2024, 8_000, 0), year(2025, last.0, last.1)],
        true,
    );
    claim.freed = dollars_of(freed);
    claim
}

fn year_three_return(sold: u32, claim: RehabilitationCredit) -> FederalDraft {
    business_return(continuing_building(
        200_000,
        20240310,
        735_520,
        Some(claim),
        sale(sold, 210_000),
    ))
}

#[test]
fn fixture_c_a_year_three_sale_recaptures_sixty_percent() {
    let d = year_three_return(20260630, recapture_claim(0, (8_000, 0)));
    let r = supported(&d);
    for (id, amount) in [
        ("business_2_asset_0_form4255_claimed", 16_000),
        ("business_2_asset_0_form4255_years", 2),
        ("business_2_asset_0_form4255_freed", 0),
        ("business_2_asset_0_form4255_percent", 60),
        ("business_2_asset_0_form4255_recapture", 9_600),
        ("business_2_asset_0_form4255_carryover", 0),
        ("business_2_asset_0_form4255_tax", 9_600),
        ("investment_credit_recapture_tax", 9_600),
        ("business_2_asset_0_form3468_basis", 80_000),
        ("business_2_asset_0_form3468_credit", 3_200),
        ("business_2_asset_0_basis_reduction", 40_000),
        ("business_credit_rehabilitation_current_year", 3_200),
        ("business_2_asset_0_disposal_sale_price", 210_000),
        ("business_2_asset_0_disposal_cost", 200_000),
        ("business_2_asset_0_disposal_depreciation", 39_635),
        ("business_2_asset_0_disposal_adjusted_basis", 160_365),
        ("business_2_asset_0_disposal_gain", 49_635),
        ("business_2_asset_0_disposal_additional_depreciation", 28_091),
        ("business_2_asset_0_disposal_1250_recapture", 28_091),
    ] {
        assert_eq!(value(&r, id), amount, "{id}");
    }
    assert_eq!(cents_of(&r, "business_2_asset_0_macrs"), 188_027);
    assert_eq!(
        printed_line(&r, "investment_credit_recapture_tax"),
        (TaxForm::Schedule2, "13a".into())
    );
    assert_eq!(
        printed_line(&r, "business_2_asset_0_form4255_percent"),
        (TaxForm::Form4255, "D15".into())
    );
    assert_eq!(printed_line(&r, "business_2_asset_0_form4255_tax").1, "1m(r)");
    assert_eq!(printed_line(&r, "business_2_asset_0_disposal_additional_depreciation").1, "28a");
    // The recapture is added to the tax outside the Form 3800 limits.
    let without_sale = {
        let mut keep = d.clone();
        let FederalIncome::Business { data } = &mut keep.income.last_mut().unwrap().income else {
            panic!()
        };
        data.assets[0].disposal = crate::AssetDisposal::No;
        keep
    };
    assert!(has(&supported(&without_sale), "business_2_asset_0_form3468_credit"));
    assert!(!has(&supported(&without_sale), "investment_credit_recapture_tax"));
    assert_eq!(supported(&restart(d.clone()).draft), r);
}

// The character of the gain: 28,091 ordinary, 21,544 section 1231 of which
// 11,544 is unrecaptured section 1250 gain.
#[test]
fn fixture_c_the_additional_depreciation_is_ordinary_and_the_rest_is_section_1231() {
    let asset = continuing_building(
        200_000,
        20240310,
        735_520,
        Some(recapture_claim(0, (8_000, 0))),
        sale(20260630, 210_000),
    );
    let gain = asset.sale_for(false, false).unwrap();
    assert_eq!(gain.depreciation, 39_635);
    assert_eq!(gain.gain, 49_635);
    assert_eq!(gain.additional, 28_091);
    assert_eq!(gain.ordinary, 28_091);
    assert_eq!(gain.recapture, 28_091);
    assert_eq!(gain.section1231, 21_544);
    assert_eq!(gain.unrecaptured, 11_544);
    assert_eq!(asset.investment_basis_reduction(), 4_000_000);
    assert_eq!(asset.investment_basis_increase(), 960_000);
}

// C2: the 2025 credit of 8,000 was used 5,000 and carried 3,000 (the
// carryforward comes from the property row, entered once). B6 = 8,000 + 5,000 = 13,000; D16 =
// 60% x 13,000 = 7,800; column (i) = 60% x 3,000 = 1,800 cuts the 2025
// carryforward to 1,200 on Form 3800; column (h) = 9,600; column (r) and
// Schedule 2 line 13a 7,800; the basis increase is 9,600, so Form 4797 is
// the same as fixture C.
#[test]
fn fixture_c2_the_recapture_cuts_the_property_s_carryforward() {
    let d = year_three_return(20260630, recapture_claim(0, (5_000, 3_000)));
    let r = supported(&d);
    for (id, amount) in [
        ("business_2_asset_0_form4255_claimed", 13_000),
        ("business_2_asset_0_form4255_recapture", 7_800),
        ("business_2_asset_0_form4255_carryover", 1_800),
        ("business_2_asset_0_form4255_tax", 7_800),
        ("investment_credit_recapture_tax", 7_800),
        ("business_credit_rehabilitation_2025", 1_200),
        ("business_2_asset_0_disposal_depreciation", 39_635),
        ("business_2_asset_0_disposal_gain", 49_635),
    ] {
        assert_eq!(value(&r, id), amount, "{id}");
    }
    // The part carried to 2026 is the property's own: nothing is entered under More credits,
    // and the cut applies to the derived entry (1,200 above).
    assert!(d.credits.as_ref().unwrap().business_credits.is_none());
}

// C3: the whole-year boundary. A February 28 sale is one full year (80%):
// D16 = 12,800 and the 2026 share 1,600; March 1 is two years (60%).
#[test]
fn fixture_c3_the_recapture_percent_changes_on_the_anniversary_month() {
    let r = supported(&year_three_return(20260228, recapture_claim(0, (8_000, 0))));
    assert_eq!(value(&r, "business_2_asset_0_form4255_years"), 1);
    assert_eq!(value(&r, "business_2_asset_0_form4255_percent"), 80);
    assert_eq!(value(&r, "business_2_asset_0_form4255_recapture"), 12_800);
    assert_eq!(value(&r, "business_2_asset_0_form3468_credit"), 1_600);
    let r = supported(&year_three_return(20260301, recapture_claim(0, (8_000, 0))));
    assert_eq!(value(&r, "business_2_asset_0_form4255_years"), 2);
    assert_eq!(value(&r, "business_2_asset_0_form4255_percent"), 60);
    // Worksheet 2 credits that could have been used reduce the recapture.
    let r = supported(&year_three_return(20260630, recapture_claim(1_000, (8_000, 0))));
    assert_eq!(value(&r, "business_2_asset_0_form4255_freed"), 1_000);
    assert_eq!(value(&r, "business_2_asset_0_form4255_recapture"), 9_000);
}

// C4: placed in service March 10, 2021 and sold June 30, 2026 is five full
// years: no Form 4255 and no 2026 share (2026 is outside the 2021-2025
// credit period), but the section 50(c) reduction stays in Form 4797 line 24
// and line 28a.
//   Depreciation allowed: 2021 160,000 x 2.033% = 3,252.80 + 4 x 4,102.40
//   = 19,662.40 prior; 2026 1,880.27; + reduction 40,000 = 61,542.67 -> 61,543.
//   Straight line on 200,000: 4,066.00 + 4 x 5,128.00 + 2,350.33 = 26,928.33
//   -> 26,928; line 28a = 61,543 - 26,928 = 34,615.
#[test]
fn fixture_c4_a_sale_after_five_years_has_no_recapture_but_keeps_the_reduction() {
    let claim = rehabilitation(
        20190401,
        100_000,
        200_000,
        (2021..=2025).map(|y| year(y, 8_000, 0)).collect(),
        true,
    );
    let d = business_return(continuing_building(
        200_000,
        20210310,
        1_966_240,
        Some(claim),
        sale(20260630, 210_000),
    ));
    let r = supported(&d);
    assert!(!has(&r, "business_2_asset_0_form4255_percent"));
    assert!(!has(&r, "investment_credit_recapture_tax"));
    assert!(!has(&r, "business_2_asset_0_form3468_credit"));
    assert_eq!(value(&r, "business_2_asset_0_disposal_cost"), 200_000);
    assert_eq!(value(&r, "business_2_asset_0_disposal_depreciation"), 61_543);
    assert_eq!(value(&r, "business_2_asset_0_disposal_additional_depreciation"), 34_615);
}

// A recapture cannot be calculated from a history that is not confirmed.
#[test]
fn an_unconfirmed_history_with_a_recapture_is_refused() {
    let mut claim = recapture_claim(0, (8_000, 0));
    claim.confirmed = false;
    let r = estimate(&year_three_return(20260630, claim));
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| {
        i.display.key == "ISSUE_INVESTMENT_CREDIT_RECAPTURE_NEEDS_A_CONFIRMED_HISTORY"
    }));
    // Outside the recapture period the history is not needed.
    let mut claim = rehabilitation(
        20190401,
        100_000,
        200_000,
        (2021..=2025).map(|y| year(y, 8_000, 0)).collect(),
        false,
    );
    claim.confirmed = false;
    let d = business_return(continuing_building(
        200_000,
        20210310,
        1_966_240,
        Some(claim),
        sale(20260630, 210_000),
    ));
    let r = supported(&d);
    assert!(noted(&r, "WARN_REHABILITATION_CREDIT_LEFT_OUT"));
}

// ---- Fixture D: energy property on a farm (section 48E) and its section 48 twin ---------------

// Schedule F asset: 5-year solar equipment, cost $120,000, placed in service
// May 1, 2026, bought October 1, 2025, first-year bonus. Construction began
// September 15, 2025 (section 48E), the 30% rate, energy community 10%.
//   Part V line 1a 120,000; 1b 30%; 1c 36,000; 1f 10%; 1g 12,000; lines 2,
//   5, 7, 8, 11 = 48,000 -> Form 3800 line 1v. Basis reduction 50% x 48,000 =
//   24,000; bonus (100%, bought after January 19, 2025) on 96,000.
fn solar(
    cost: i64,
    began: u32,
    increased_rate: bool,
    community: bool,
    confirmed: bool,
) -> BusinessAsset {
    BusinessAsset {
        cost: dollars_of(cost),
        acquired: 20251001,
        placed: 20260501,
        history: EquipmentHistory::New {
            section179: Section179::Eligible { elected: 0 },
            bonus: BonusElection::Apply,
            tables: false,
        },
        investment_credit: InvestmentCredit::NewEnergy {
            kind: EnergyProperty::Solar,
            construction_began: began,
            increased_rate,
            domestic_content: false,
            energy_community: community,
            confirmed,
        },
        ..bonus_equipment(0, 100)
    }
}

fn farm_return(asset: BusinessAsset) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    let mut farm = raised_farm(300_000, 20_000);
    let FederalIncome::Business { data } = &mut farm else { panic!() };
    data.assets.push(asset);
    add(&mut d, income(farm));
    d
}

#[test]
fn fixture_d_farm_solar_construction_after_2024_is_the_clean_electricity_credit() {
    let d = farm_return(solar(120_000, 20250915, true, true, true));
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_form3468_basis"), 120_000);
    assert_eq!(value(&r, "business_2_asset_0_form3468_credit"), 48_000);
    assert_eq!(value(&r, "business_2_asset_0_basis_reduction"), 24_000);
    assert_eq!(
        printed_line(&r, "business_2_asset_0_form3468_basis"),
        (TaxForm::Form3468, "1a".into())
    );
    // Part V (section 48E): Form 3800 line 1v, not a specified credit.
    assert_eq!(value(&r, "business_credit_clean_electricity_current_year"), 48_000);
    assert_eq!(printed_line(&r, "business_credit_clean_electricity_current_year").1, "1v");
    assert!(!has(&r, "business_credit_energy_current_year"));
    // Bonus depreciation on the cost less half of the credit: 96,000.
    assert_eq!(value(&r, "business_2_asset_0_bonus"), 96_000);
    assert_eq!(supported(&restart(d).draft), r);
}

// D2: construction on November 1, 2024 is the section 48 energy credit (Part
// VI Section B, Form 3800 line 4a, specified) with the same amounts.
#[test]
fn fixture_d2_the_same_solar_array_begun_in_2024_is_the_section_48_credit() {
    let r = supported(&farm_return(solar(120_000, 20241101, true, true, true)));
    assert_eq!(value(&r, "business_2_asset_0_form3468_credit"), 48_000);
    assert_eq!(value(&r, "business_credit_energy_current_year"), 48_000);
    assert_eq!(printed_line(&r, "business_credit_energy_current_year").1, "4a");
    assert!(!has(&r, "business_credit_clean_electricity_current_year"));
    assert_eq!(value(&r, "business_2_asset_0_basis_reduction"), 24_000);
    assert_eq!(value(&r, "business_2_asset_0_bonus"), 96_000);
}

// D3: without the 30% rate the base 6% and the 2% bonus: 8% x 120,000 =
// 9,600; the reduction is 4,800 and the bonus 115,200.
#[test]
fn fixture_d3_the_base_rate_has_the_small_bonus() {
    let r = supported(&farm_return(solar(120_000, 20250915, false, true, true)));
    assert_eq!(value(&r, "business_2_asset_0_form3468_credit"), 9_600);
    assert_eq!(value(&r, "business_2_asset_0_basis_reduction"), 4_800);
    assert_eq!(value(&r, "business_2_asset_0_bonus"), 115_200);
}

// D4: not confirmed: left out with a notice and depreciated on the full cost.
#[test]
fn fixture_d4_an_unconfirmed_energy_credit_is_left_out() {
    let r = supported(&farm_return(solar(120_000, 20250915, true, true, false)));
    assert!(noted(&r, "WARN_ENERGY_CREDIT_LEFT_OUT"));
    assert_eq!(value(&r, "business_2_asset_0_bonus"), 120_000);
    assert!(!has(&r, "business_2_asset_0_form3468_credit"));
}

// A section 179 deduction is not part of the credit base (i3468).
#[test]
fn the_section_179_deduction_is_not_a_credit_base() {
    let mut asset = solar(120_000, 20250915, true, false, true);
    asset.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: dollars_of(20_000) },
        bonus: BonusElection::Apply,
        tables: false,
    };
    // 30% x (120,000 - 20,000) = 30,000; the reduction 15,000.
    assert_eq!(asset.current_year_investment_credit(), 30_000);
    assert_eq!(asset.investment_basis_reduction(), 1_500_000);
    assert_eq!(asset.business_basis(), 10_500_000);
}

// A geothermal heat pump keeps the section 48 credit for construction after 2024.
#[test]
fn a_geothermal_heat_pump_is_always_the_section_48_credit() {
    let mut asset = solar(50_000, 20250915, true, false, true);
    let InvestmentCredit::NewEnergy { kind, .. } = &mut asset.investment_credit else {
        unreachable!()
    };
    *kind = EnergyProperty::GeothermalHeatPump;
    assert_eq!(asset.investment_section(), Some(BusinessCredit::Energy));
    assert_eq!(asset.current_year_investment_credit(), 15_000);
}

// An earlier energy credit: half of the credit stays out of the depreciable
// basis; a sale within five years recaptures it. 5-year equipment placed
// June 1, 2023, credit 12,000 (6,000 used, 2,000 carried), sold March 20,
// 2026: 2 full years (June 2023 to June 2025 is two; March 2026 is 2) -> 60%.
pub(in crate::tests) fn earlier_energy_sale_asset() -> BusinessAsset {
    let mut asset = solar(40_000, 20220101, true, false, true);
    asset.acquired = 20230501;
    asset.placed = 20230601;
    asset.history = EquipmentHistory::Continuing {
        initial_section179: 0,
        carryforward: 0,
        prior_depreciation: 1_000_000,
        amt: EquipmentAmtHistory::Same,
        mid_quarter: false,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    asset.investment_credit = InvestmentCredit::EarlierEnergy {
        section: EnergySection::Energy,
        credit: dollars_of(12_000),
        used: dollars_of(6_000),
        carried: dollars_of(2_000),
        freed: 0,
        unchanged: true,
    };
    asset.disposal = sale(20260320, 20_000);
    asset
}

#[test]
fn an_earlier_energy_credit_is_recaptured_on_a_sale_within_five_years() {
    let asset = earlier_energy_sale_asset();
    let recapture = asset.investment_recapture().unwrap();
    assert_eq!((recapture.years, recapture.percent), (2, 60));
    assert_eq!(recapture.tax, 3_600);
    assert_eq!(recapture.carryover, 1_200);
    // Half of the recapture (3,600 + 1,200) returns to basis.
    assert_eq!(recapture.basis_increase, 240_000);
    assert_eq!(asset.investment_basis_reduction(), 600_000);
    let mut d = business_return(asset);
    let r = estimate(&d);
    // The carried part comes from the property itself (F3, 2026-10-09; entered
    // once, there) and is cut to 800.
    assert_eq!(value(&r, "business_credit_energy_2023"), 800);
    assert_eq!(value(&r, "investment_credit_recapture_tax"), 3_600);
    // A history whose later change is not confirmed is refused by name.
    let FederalIncome::Business { data } = &mut d.income.last_mut().unwrap().income else {
        panic!()
    };
    let InvestmentCredit::EarlierEnergy { unchanged, .. } = &mut data.assets[0].investment_credit
    else {
        unreachable!()
    };
    *unchanged = false;
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
}

// ---- Fixture E: a rental improvement with the Form 8582-CR special allowance ---------------------

// Single, wages $210,000 (modified AGI $210,000); one commercial rental with a
// net loss that Form 8582 suspends (above $150,000 there is no allowance for
// rental losses: line 9 = 0, net passive income below zero). An improvement
// row: cost $150,000, no land, in service July 1, 2026, with a confirmed
// rehabilitation credit (2024-07-01 + 24 months ends 2026-06-30).
//   Credit 150,000 x 4% = 6,000, passive rental real estate (Worksheet 2).
//   Form 8582-CR: 2a 6,000; 2c 6,000; 5 6,000; 6 0; 7 6,000.
//   Part III: 17 6,000; 18 0; 19 6,000; 20 6,000; 21 250,000; 22 210,000;
//   23 40,000; 24 min(20,000, 25,000) = 20,000; 25 0; 26 20,000; taxable
//   income 210,000 - 16,100 = 193,900; 27 = tax(193,900) - tax(173,900) =
//   24% x 20,000 = 4,800; 28 0; 29 4,800; 30 min(6,000, 4,800) = 4,800;
//   37 4,800.
//   Form 3800: line 32 6,000; line 33 4,800; regular tax on 193,900 =
//   1,240 + 4,560 + 12,166 + 21,168 = 39,134; l13 = 25% x 14,134 = 3,533.50
//   -> 3,534; l27 = 39,134 - 3,534 = 35,600; l37 = min(35,600, 4,800) = 4,800.
//   Allowed 4,800; 1,200 to 2027 as a prior-year unallowed credit.
//   Depreciable basis 150,000 - 30,000 = 120,000; July first-year 1.177% ->
//   1,412.40.
fn improvement(cost: i64, in_service: u32, claim: Option<RehabilitationCredit>) -> RentalBuilding {
    RentalBuilding {
        rehabilitation: claim,
        ..purchased_rental_building(dollars_of(cost), 0, in_service)
    }
}

fn rental_return(wages: i64, status: FederalStatus) -> FederalDraft {
    let mut d = complete(status);
    d.income[0] = wage(1, Owner::Taxpayer, dollars_of(wages), 0);
    let mut property = rental(10_000, 20_000);
    property.property_type = RentalPropertyType::Commercial;
    property.participation = crate::RentalParticipation::Active;
    property.buildings = vec![
        purchased_rental_building(dollars_of(500_000), dollars_of(100_000), 20190301),
        improvement(
            150_000,
            20260701,
            Some(rehabilitation(20240701, 100_000, 150_000, vec![], true)),
        ),
    ];
    add(&mut d, income(FederalIncome::Rental { data: property }));
    d
}

#[test]
fn fixture_e_the_rehabilitation_special_allowance_allows_part_of_a_rental_credit() {
    let d = rental_return(210_000, FederalStatus::Single);
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_income"), 193_900);
    assert_eq!(value(&r, "regular_income_tax"), 39_134);
    for (id, expected) in [
        ("2a", 6_000),
        ("2c", 6_000),
        ("5", 6_000),
        ("6", 0),
        ("7", 6_000),
        ("17", 6_000),
        ("18", 0),
        ("19", 6_000),
        ("20", 6_000),
        ("21", 250_000),
        ("22", 210_000),
        ("23", 40_000),
        ("24", 20_000),
        ("25", 0),
        ("26", 20_000),
        ("27", 4_800),
        ("28", 0),
        ("29", 4_800),
        ("30", 4_800),
        ("31", 6_000),
        ("32", 4_800),
        ("33", 1_200),
        ("37", 4_800),
    ] {
        let id = format!("form8582cr_line_{id}");
        assert_eq!(value(&r, &id), expected, "{id}");
        assert_eq!(printed_line(&r, &id).0, TaxForm::Form8582Cr, "{id}");
    }
    for (id, expected) in [
        ("form_3800_line_32", 6_000),
        ("form_3800_line_33", 4_800),
        ("form_3800_line_13", 3_534),
        ("form_3800_line_27", 35_600),
        ("form_3800_line_37", 4_800),
        ("form_3800_line_38", 4_800),
    ] {
        assert_eq!(value(&r, id), expected, "{id}");
    }
    assert_eq!(value(&r, "business_credit_rehabilitation_current_year_passive"), 6_000);
    assert_eq!(value(&r, "business_credit_rehabilitation_current_year_passive_allowed"), 4_800);
    assert_eq!(
        value(&r, "business_credit_rehabilitation_current_year_passive_to_next_year"),
        1_200
    );
    assert_eq!(value(&r, "general_business_credit"), 4_800);
    // The improvement's Form 3468 rows and the reduced depreciation.
    assert_eq!(value(&r, "rental_2_building_1_form3468_credit"), 6_000);
    assert_eq!(value(&r, "rental_2_building_1_basis_reduction"), 30_000);
}

// E2: modified AGI of $260,000: line 23 = 0, so the allowance is zero and the
// whole credit carries (line 30 = 0).
#[test]
fn fixture_e2_no_allowance_above_250000_of_income() {
    let r = supported(&rental_return(260_000, FederalStatus::Single));
    assert_eq!(value(&r, "form8582cr_line_23"), 0);
    assert_eq!(value(&r, "form8582cr_line_30"), 0);
    assert_eq!(value(&r, "form8582cr_line_37"), 0);
    assert_eq!(value(&r, "general_business_credit"), 0);
    assert_eq!(
        value(&r, "business_credit_rehabilitation_current_year_passive_to_next_year"),
        6_000
    );
}

// The allowance phases out at half of the income above $200,000: $240,000 of
// income leaves 50% x 10,000 = 5,000 (line 24) and tax 32% x 5,000 = 1,600.
#[test]
fn the_allowance_is_half_of_the_income_below_250000() {
    let r = supported(&rental_return(240_000, FederalStatus::Single));
    assert_eq!(value(&r, "form8582cr_line_23"), 10_000);
    assert_eq!(value(&r, "form8582cr_line_24"), 5_000);
    // Taxable income 223,900 less 5,000 stays above $201,775: 32% x 5,000.
    assert_eq!(value(&r, "form8582cr_line_27"), 1_600);
    assert_eq!(value(&r, "form8582cr_line_30"), 1_600);
}

// The carryback year: a 2026 energy credit may be carried back to 2023, 2024
// and 2025; an ordinary credit only to 2025.
#[test]
fn the_energy_credits_carry_back_three_years() {
    let mut r = FederalResult::empty(1);
    let carrybacks = vec![
        (BusinessCredit::Energy, 2024, 1_000, None),
        (BusinessCredit::Energy, 2025, 500, None),
    ];
    crate::general_business_credit::calculate(
        None,
        vec![entry(BusinessCredit::Energy, 20_000)],
        &carrybacks,
        TaxCapacity { regular_tax_before_credits: 5_000, ..capacity() },
        &mut r,
    );
    // Tax use 5,000 (line 27 = 5,000); 1,000 back to 2024 (the earliest year
    // is used first) and 500 to 2025; 20,000 - 5,000 - 1,500 to 2027.
    let _ = &r;
    assert_eq!(value(&r, "business_credit_energy_current_year_used"), 5_000);
    assert_eq!(value(&r, "business_credit_energy_current_year_carryback_2024"), 1_000);
    assert_eq!(value(&r, "business_credit_energy_current_year_carryback_2025"), 500);
    assert_eq!(value(&r, "business_credit_energy_current_year_to_next_year"), 13_500);
    let history = BusinessCreditHistory {
        credits: Vec::new(),
        carryforwards: vec![],
        carrybacks: vec![BusinessCreditCarryback {
            research_source: None,
            credit: BusinessCredit::WorkOpportunity,
            year: 2024,
            amount: dollars_of(100),
        }],
        spouse_without_credits: false,
    };
    assert_eq!(history.rule(), Some("BUSINESS_CARRYBACK_YEAR_NOT_ALLOWED"));
    let history = BusinessCreditHistory {
        carrybacks: vec![
            BusinessCreditCarryback {
                research_source: None,
                credit: BusinessCredit::CleanElectricity,
                year: 2025,
                amount: dollars_of(100),
            },
            BusinessCreditCarryback {
                research_source: None,
                credit: BusinessCredit::CleanElectricity,
                year: 2025,
                amount: dollars_of(200),
            },
        ],
        ..history
    };
    assert_eq!(history.rule(), Some("BUSINESS_CARRYBACK_TWICE"));
}

// ---- Fixture F: a rental sold within five years of its improvement -----------------------------

// The commercial rental of fixture E with an improvement of $150,000 in service
// June 15, 2024 (measuring period 2022-07-01 to 2024-06-30), $6,000 of credit
// used on each of the 2024 and 2025 returns, sold August 15, 2026 for $800,000
// ($150,000 of it land).
//   Form 4255: 2 full years from 2024-06-01 -> 60%: B6 12,000, D16 7,200;
//   the 2026 share 150,000 x 4% x 40% = 2,400.
//   The improvement's depreciation on 150,000 - 30,000 = 120,000: 2024 June
//   1.391% 1,669.20; 2025 2.564% 3,076.80; 2026 mid-month to August (15/24)
//   1,923.00: 6,669.00. Net section 50(c) reduction 30,000 - 7,200 = 22,800.
//   Straight line on 150,000: 2,086.50 + 3,846.00 + 2,403.75 = 8,336.25.
//   Additional depreciation (6,669.00 + 22,800.00) - 8,336.25 = 21,132.75.
fn rental_sale_return(claim_confirmed: bool) -> FederalDraft {
    let mut d = rental_return(210_000, FederalStatus::Single);
    let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else { panic!() };
    data.buildings[1] = improvement(
        150_000,
        20240615,
        Some(RehabilitationCredit {
            period_start: 20220701,
            phased: false,
            building_basis: dollars_of(100_000),
            period_expenditures: dollars_of(150_000),
            earlier: vec![year(2024, 6_000, 0), year(2025, 6_000, 0)],
            freed: 0,
            confirmed: claim_confirmed,
        }),
    );
    data.sale = crate::RentalSale::Sold {
        date: 20260815,
        price: dollars_of(800_000),
        land_price: dollars_of(150_000),
        selling_expenses: 0,
        payment: crate::SalePayment::InFull,
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
    d.screening.as_mut().unwrap().section1231 = Some(no_lookback());
    d
}

#[test]
fn fixture_f_a_rental_sale_recaptures_the_improvement_s_credit() {
    let d = rental_sale_return(true);
    let r = supported(&d);
    for (id, amount) in [
        ("rental_2_building_1_form4255_claimed", 12_000),
        ("rental_2_building_1_form4255_years", 2),
        ("rental_2_building_1_form4255_percent", 60),
        ("rental_2_building_1_form4255_recapture", 7_200),
        ("investment_credit_recapture_tax", 7_200),
        ("rental_2_building_1_form3468_credit", 2_400),
        ("rental_2_building_1_basis_reduction", 30_000),
        ("rental_2_sale_building_additional_depreciation", 21_133),
        ("rental_2_sale_building_1250_recapture", 21_133),
    ] {
        assert_eq!(value(&r, id), amount, "{id}");
    }
    // The 2026 share is a passive Worksheet 2 credit.
    assert_eq!(value(&r, "business_credit_rehabilitation_current_year_passive"), 2_400);
    // The improvement's own recapture percent shrinks only its own share.
    assert!(!has(&r, "rental_2_building_0_form4255_percent"));
}

#[test]
fn a_rental_with_a_rehabilitation_credit_exchanged_or_sold_on_installments_is_refused() {
    let mut d = rental_sale_return(true);
    let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else { panic!() };
    data.sale = crate::RentalSale::Sold {
        date: 20260815,
        price: dollars_of(800_000),
        land_price: dollars_of(150_000),
        selling_expenses: 0,
        payment: crate::SalePayment::Installment {
            assumed_debt: 0,
            payments: dollars_of(100_000),
            interest: crate::NoteInterest::Adequate,
            special: false,
        },
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(
        r.issues.iter().any(|i| i.display.key == "ISSUE_RENTAL_SALE_WITH_REHABILITATION_CREDIT"),
        "{:?}",
        r.issues
    );
}

// An unconfirmed history of a rental improvement sold within the period is
// refused: leaving it out would drop a recapture tax.
#[test]
fn an_unconfirmed_rental_improvement_history_with_a_sale_is_refused() {
    let r = estimate(&rental_sale_return(false));
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| {
        i.display.key == "ISSUE_INVESTMENT_CREDIT_RECAPTURE_NEEDS_A_CONFIRMED_HISTORY"
    }));
}

// ---- Schedule K-1 items ------------------------------------------------------------------------

fn investment_partner(items: Vec<K1InvestmentProperty>, active: bool) -> PartnerK1 {
    let mut data = ordinary_partner_source(0, active);
    data.investment_property = items;
    data.investment_confirmed = Some(true);
    // Rental real estate property: participation is answered.
    data.rental_facts.participation = Some(crate::K1RentalParticipation::NotActive);
    data
}

fn with_partner(data: PartnerK1) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data }));
    d
}

// Rehabilitation expenditures reported with the rental real estate activity
// (box 15 code E) are a Worksheet 2 passive credit with the special
// allowance; the same amount reported for another activity (box 20 code D)
// follows the partner's material participation.
#[test]
fn k1_rental_rehabilitation_is_a_worksheet_2_credit_and_other_property_is_not() {
    let rental = K1InvestmentProperty::RehabilitationRental {
        id: 0,
        own_form: None,
        expenditures: dollars_of(500_000),
        placed_year: 2025,
    };
    let r = supported(&with_partner(investment_partner(vec![rental], true)));
    // Wages $60,000: the allowance is the full $25,000 (half of 250,000 -
    // 60,000 is above it), so the tax saved on that deduction equivalent
    // allows the credit up to it.
    assert_eq!(value(&r, "form8582cr_line_2a"), 20_000);
    assert_eq!(value(&r, "form8582cr_line_24"), 25_000);
    let allowed = value(&r, "form8582cr_line_30");
    assert!(allowed > 0 && allowed < 20_000, "{allowed}");
    assert_eq!(value(&r, "business_credit_rehabilitation_current_year_passive_allowed"), allowed);
    let other = K1InvestmentProperty::RehabilitationOther {
        id: 0,
        own_form: None,
        expenditures: dollars_of(500_000),
        placed_year: 2025,
    };
    // Without material participation the other activity's credit is a
    // Worksheet 4 credit: with no net passive income none is allowed.
    let r = supported(&with_partner(investment_partner(vec![other], false)));
    assert!(!has(&r, "form8582cr_line_2a"));
    assert_eq!(value(&r, "form8582cr_line_4a"), 20_000);
    assert_eq!(
        value(&r, "business_credit_rehabilitation_current_year_passive_to_next_year"),
        20_000
    );
}

// The energy and clean electricity items use the basis and the entity's
// rates; the energy credit is a specified credit and the clean electricity
// investment credit is not.
#[test]
fn k1_energy_items_use_the_basis_and_the_entity_s_rates() {
    let rate =
        crate::EnergyRate { increased_rate: true, domestic_content: false, energy_community: true };
    let items = vec![
        K1InvestmentProperty::Energy {
            id: 0,
            own_form: None,
            basis: dollars_of(10_000),
            rate: rate.clone(),
        },
        K1InvestmentProperty::CleanElectricity {
            id: 0,
            own_form: None,
            basis: dollars_of(50_000),
            storage: false,
            rate,
        },
    ];
    assert_eq!(items[0].current_year_credit(), 4_000);
    assert_eq!(items[1].current_year_credit(), 20_000);
    assert_eq!(items[0].section(), BusinessCredit::Energy);
    assert_eq!(items[1].section(), BusinessCredit::CleanElectricity);
    // Half of each energy credit reduces the owner's basis (section 50(c)(5)).
    assert_eq!(crate::investment_credit::k1_basis_decrease(&items), 1_200_000);
    let r = supported(&with_partner(investment_partner(items, true)));
    assert_eq!(printed_line(&r, "business_credit_energy_current_year").1, "4a");
    assert_eq!(printed_line(&r, "business_credit_clean_electricity_current_year").1, "1v");
    assert_eq!(value(&r, "partner_2_investment_0_basis_reduction"), 2_000);
    assert_eq!(value(&r, "partner_2_investment_1_basis_reduction"), 10_000);
}

// Rehabilitation expenditures of a building placed in service in 2026 reduce
// the owner's basis by the whole credit determined; an earlier year's do not
// reduce it again.
#[test]
fn the_basis_decrease_is_taken_in_the_year_the_building_is_placed_in_service() {
    let item = |year| K1InvestmentProperty::RehabilitationOther {
        id: 0,
        own_form: None,
        expenditures: dollars_of(100_000),
        placed_year: year,
    };
    assert_eq!(item(2026).basis_decrease(), 2_000_000);
    assert_eq!(item(2025).basis_decrease(), 0);
    assert_eq!(item(2022).current_year_credit(), 4_000);
}

// The partner's outside basis and the shareholder's stock basis fall by the
// section 50(c)(5) reduction, not below zero and never carried.
#[test]
fn the_reduction_lowers_the_basis_without_carrying() {
    let empty = crate::k1_loss::LossAmounts::default();
    let partner = |reduction: i64| crate::partnership_basis::PartnerBasis {
        deemed_money: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
            .expect("finite fixture dollars"),
        opening: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new((10_000) * 100)
            .expect("finite fixture dollars"),
        contributions: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
            .expect("finite fixture dollars"),
        opening_liabilities: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
            .expect("finite fixture dollars"),
        ending_liabilities: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
            .expect("finite fixture dollars"),
        partnership_liabilities_assumed:
            crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
                .expect("finite fixture dollars"),
        personal_liabilities_transferred:
            crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
                .expect("finite fixture dollars"),
        income: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
            .expect("finite fixture dollars"),
        cash_distributions: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
            .expect("finite fixture dollars"),
        distributed_property_basis:
            crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
                .expect("finite fixture dollars"),
        nondeductible: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
            .expect("finite fixture dollars"),
        credit_reduction: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
            (reduction) * 100,
        )
        .expect("finite fixture dollars"),
        depletion: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
            .expect("finite fixture dollars"),
        losses: &empty,
        business_interest: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
            .expect("finite fixture dollars"),
        excess_business_interest: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
            0,
        )
        .expect("finite fixture dollars"),
    };
    assert_eq!(crate::kernel::ratio::dollars(partner(0).calculate().unwrap().ending.get()), 10_000);
    assert_eq!(
        crate::kernel::ratio::dollars(partner(4_000).calculate().unwrap().ending.get()),
        6_000
    );
    // Only the remaining basis is reduced, and nothing carries.
    let result = partner(15_000).calculate().unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(result.ending.get()),
            crate::kernel::ratio::dollars(result.nondeductible_carry.get())
        ),
        (0, 0)
    );
    let stock = |reduction: i64| crate::scorp_basis::StockBasis {
        opening: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new((10_000) * 100)
            .expect("finite fixture dollars"),
        additions: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
            .expect("finite fixture dollars"),
        income: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
            .expect("finite fixture dollars"),
        distributions: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
            .expect("finite fixture dollars"),
        nondeductible: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
            .expect("finite fixture dollars"),
        credit_reduction: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
            (reduction) * 100,
        )
        .expect("finite fixture dollars"),
        prior_nondeductible: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(0)
            .expect("finite fixture dollars"),
        losses_first: false,
        losses: &empty,
    };
    assert_eq!(
        crate::kernel::ratio::dollars(stock(0).calculate().unwrap().remaining_stock.get()),
        10_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(stock(4_000).calculate().unwrap().remaining_stock.get()),
        6_000
    );
    let result = stock(15_000).calculate().unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(result.remaining_stock.get()),
            crate::kernel::ratio::dollars(result.nondeductible_to_debt.get())
        ),
        (0, 0)
    );
}

// A K-1 list needs the personal confirmation (stored records are valid only
// with it).
#[test]
fn a_k1_investment_list_is_valid_only_with_the_confirmation() {
    let item = K1InvestmentProperty::RehabilitationOther {
        id: 0,
        own_form: None,
        expenditures: dollars_of(100_000),
        placed_year: 2025,
    };
    let mut data = investment_partner(vec![item], true);
    assert!(data.valid());
    data.investment_confirmed = None;
    assert!(!data.valid());
    data.investment_property.clear();
    assert!(data.valid());
}

// E3: married filing separately and living together all year: Part III is
// not completed (section 469(i)(5)), so line 30 is 0 and the whole credit
// carries. E4: living apart all year the threshold is $125,000 and the cap
// $12,500: wages $115,000 leave line 23 = 10,000, line 24 = 5,000, and the
// tax on it is 22% x 5,000 = 1,100 (taxable income 98,900, all in the 22% band).
#[test]
fn fixture_e3_separate_filers_living_together_complete_no_part_iii() {
    let r = supported(&rental_return(115_000, FederalStatus::Separate));
    assert_eq!(value(&r, "form8582cr_line_2a"), 6_000);
    assert_eq!(value(&r, "form8582cr_line_30"), 0);
    assert_eq!(value(&r, "form8582cr_line_37"), 0);
    assert!(!has(&r, "form8582cr_line_24"));
    assert_eq!(
        value(&r, "business_credit_rehabilitation_current_year_passive_to_next_year"),
        6_000
    );
}

#[test]
fn fixture_e4_separate_filers_living_apart_have_the_125000_threshold() {
    let mut d = rental_return(115_000, FederalStatus::Separate);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::AllYear };
    let r = supported(&d);
    assert_eq!(value(&r, "form8582cr_line_21"), 125_000);
    assert_eq!(value(&r, "form8582cr_line_22"), 115_000);
    assert_eq!(value(&r, "form8582cr_line_23"), 10_000);
    assert_eq!(value(&r, "form8582cr_line_24"), 5_000);
    assert_eq!(value(&r, "form8582cr_line_27"), 1_100);
    assert_eq!(value(&r, "form8582cr_line_30"), 1_100);
    assert_eq!(value(&r, "form8582cr_line_37"), 1_100);
}

// A prior-year unallowed rehabilitation credit of a rental real estate
// activity (2025 Form 8582-CR Worksheet 2) joins line 2b and the allowance; the
// worksheet is refused for any other credit and is not allowed on a
// housing credit.
#[test]
fn a_prior_year_unallowed_rehabilitation_credit_uses_worksheet_2() {
    let mut d = rental_return(210_000, FederalStatus::Single);
    d.credits.as_mut().unwrap().business_credits = Some(history(
        vec![carryforward(
            BusinessCredit::Rehabilitation,
            2025,
            1_000,
            Some(PassiveWorksheet::Rehabilitation),
        )],
        vec![],
    ));
    let r = supported(&d);
    assert_eq!(value(&r, "form8582cr_line_2a"), 6_000);
    assert_eq!(value(&r, "form8582cr_line_2b"), 1_000);
    assert_eq!(value(&r, "form8582cr_line_2c"), 7_000);
    // The allowance (line 24 = 20,000, tax 4,800) takes the credits pro rata.
    assert_eq!(value(&r, "form8582cr_line_30"), 4_800);
    let history = BusinessCreditHistory {
        credits: Vec::new(),
        carryforwards: vec![carryforward(
            BusinessCredit::LowIncomeHousing,
            2025,
            1_000,
            Some(PassiveWorksheet::Rehabilitation),
        )],
        carrybacks: vec![],
        spouse_without_credits: false,
    };
    assert_eq!(history.rule(), Some("BUSINESS_CREDIT_WORKSHEET_MISMATCH"));
    let history = BusinessCreditHistory {
        carryforwards: vec![carryforward(BusinessCredit::Rehabilitation, 2007, 1_000, None)],
        ..history
    };
    assert_eq!(history.rule(), Some("BUSINESS_CREDIT_BEFORE_ITS_FIRST_YEAR"));
}

// The page rule of the carryover: a rehabilitation credit of 2007 and a clean
// electricity credit before 2025 are refused by the first-year rules.
#[test]
fn the_investment_credits_have_first_years() {
    assert_eq!(BusinessCredit::Rehabilitation.first_year(), 2008);
    assert_eq!(BusinessCredit::Energy.first_year(), 2006);
    assert_eq!(BusinessCredit::CleanElectricity.first_year(), 2025);
    assert!(BusinessCredit::Rehabilitation.specified(2008));
    assert!(!BusinessCredit::Rehabilitation.specified(2007));
    assert!(BusinessCredit::Energy.specified(2026));
    assert!(!BusinessCredit::CleanElectricity.specified(2026));
    assert_eq!(BusinessCredit::Energy.carryback_first_year(), 2023);
    assert_eq!(BusinessCredit::Rehabilitation.carryback_first_year(), 2025);
}

#[test]
fn form_4255_recapture_is_on_form_2210_line_2_and_form_1040_es_lines_10_and_12b() {
    // Fixture C (Form 4255 recapture $9,600, self-employment tax $13,864, withholding $6,500)
    // with a 2025 tax of $1,000,000 (line 9 is 90% of this year): the 2025 i2210 line 2 list
    // names Schedule 2 line 17a (2026 line 13a) and the 2026 Form 1040-ES line 10 takes the
    // same Schedule 2 lines. Line 2 $23,464, line 4 $50,570, line 9 $45,513, four installments
    // of $11,378.25, penalty $1,791.91, Form 1040 line 38 $1,792. 1040-ES: line 10 $9,600,
    // 11a $50,570, 12a $45,513, 12b 110% x 50,570 = $55,627, 12c $45,513, 14a $39,013, line 15
    // $9,753.
    let mut d = year_three_return(20260630, recapture_claim(0, (8_000, 0)));
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year = Some(PriorYearTax {
        full_year: true,
        tax: 100_000_000,
        agi: Some(10_000_000),
        spouse_tax: 0,
    });
    pay.next_year =
        NextYearEstimateRecord::SameAsThisReturn { farmer_fisherman_next_year: FarmerNextYear::No };
    let r = supported(&d);
    let cents =
        |id: &str| r.lines.iter().find(|l| l.id == id).unwrap_or_else(|| panic!("{id}")).amount;
    assert_eq!(cents("form_2210_line_2"), 2_346_400);
    assert_eq!(cents("form_2210_line_4"), 5_057_000);
    assert_eq!(cents("form_2210_line_9"), 4_551_300);
    assert_eq!(cents("form_2210_line_19"), 179_191);
    assert_eq!(return_line(&r, "38"), 179_200);
    assert_eq!(cents("form_1040_es_line_10"), 960_000);
    assert_eq!(cents("form_1040_es_line_11a"), 5_057_000);
    assert_eq!(cents("form_1040_es_line_12a"), 4_551_300);
    assert_eq!(cents("form_1040_es_line_12b"), 5_562_700);
    assert_eq!(cents("form_1040_es_line_12c"), 4_551_300);
    assert_eq!(cents("form_1040_es_line_14a"), 3_901_300);
    assert_eq!(cents("form_1040_es_line_15"), 975_300);
}

#[test]
fn unit13_native_investment_recapture_keeps_actual_property_owners_and_order() {
    let mut d = complete(FederalStatus::Joint);
    let mut first = business_return(earlier_energy_sale_asset()).income.pop().unwrap();
    first.owner = crate::DocumentOwner::Yours;
    add(&mut d, first.clone());
    let mut second = first;
    second.owner = crate::DocumentOwner::Spouses;
    if let FederalIncome::Business { data } = &mut second.income
        && let InvestmentCredit::EarlierEnergy { used, .. } = &mut data.assets[0].investment_credit
    {
        *used = 400_000;
    } else {
        panic!()
    }
    add(&mut d, second);
    for reverse in [false, true] {
        if reverse {
            d.income.reverse();
        }
        let mut r = FederalResult::empty(d.revision);
        let claims = crate::investment_credit::claims(&d, &mut r);
        assert!(r.issues.is_empty(), "{:?}", r.issues);
        // Existing native60% recapture of6000/4000 previously used credits.
        assert_eq!(claims.recapture_by_owner, [3600, 2400]);
        assert_eq!(claims.recapture_tax, 6000);
        assert_eq!(claims.recapture_by_owner.iter().sum::<i64>(), claims.recapture_tax);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn unit13_native_rental_recapture_keeps_its_actual_owner_and_no_claim_zero() {
    let mut d = rental_sale_return(true);
    d.setup = complete(FederalStatus::Joint).setup;
    d.people = complete(FederalStatus::Joint).people;
    d.income.last_mut().unwrap().owner = crate::DocumentOwner::Spouses;
    let mut r = FederalResult::empty(d.revision);
    let claims = crate::investment_credit::claims(&d, &mut r);
    assert!(r.issues.is_empty(), "{:?}", r.issues);
    // Original fixtureF: one improvement,12000 used credit x60% =7200.
    assert_eq!(claims.recapture_by_owner, [0, 7200]);
    assert_eq!(claims.recapture_tax, 7200);
    let mut r = FederalResult::empty(1);
    let empty = crate::investment_credit::claims(&complete(FederalStatus::Joint), &mut r);
    assert!(r.issues.is_empty());
    assert_eq!(empty.recapture_tax, 0);
    assert_eq!(empty.recapture_by_owner, [0; 2]);
}

fn unit13_direct_recapture_return(injured: Owner) -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![
        wage(1, Owner::Taxpayer, 6_000_000, 1_000_000),
        wage(2, Owner::Spouse, 2_000_000, 500_000),
    ];
    d.next_id = 3;
    let mut business = business_return(earlier_energy_sale_asset()).income.pop().unwrap();
    let FederalIncome::Business { data } = &mut business.income else { panic!() };
    data.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: 0,
        accounting: BusinessAccounting::Cash,
    };
    add(&mut d, business);
    d.screening.as_mut().unwrap().section1231 = Some(no_lookback());
    d.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    d.injured_spouse = Some(InjuredSpouseAllocation {
        injured,
        debt: OffsetDebt::ChildSupport,
        obligated: false,
        community: InjuredCommunityFacts::No,
        student_loan_borrower: None,
        estimated: EstimatedSplit::IrsFormula,
        allocations: vec![],
        claimants: vec![],
    });
    d
}

#[test]
fn unit13_direct_recapture_reaches_shared_line18_without_claiming_final_property_sources() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for property_owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut d = unit13_direct_recapture_return(injured);
            d.income.last_mut().unwrap().owner = crate::DocumentOwner::from(property_owner);
            let r = supported(&d);
            assert_eq!(value(&r, "self_employment_tax"), 0);
            assert_eq!(value(&r, "investment_credit_recapture_tax"), 3600);
            assert_eq!(value(&r, "form_8379_line_18_a"), 3600);
            assert_eq!(
                value(&r, "form_8379_line_18_b"),
                if injured == property_owner { 3600 } else { 0 }
            );
            assert_eq!(
                value(&r, "form_8379_line_18_c"),
                if injured == property_owner { 0 } else { 3600 }
            );
            // The source-tax routing does not invent the still-missing final
            // property/1231 income attribution or a positive refund estimate.
            assert!(!has(&r, "form_8379_injured_refund_estimate"));
            let mut ordinary = d.clone();
            ordinary.injured_spouse = None;
            let plain = supported(&ordinary);
            assert_eq!((r.tax, r.refund, r.balance), (plain.tax, plain.refund, plain.balance));
        }
    }
}

fn unit13_direct_recapture_client_fixture(profile: &str) -> (FederalDraft, TaxSection, i64) {
    let mut draft = unit13_direct_recapture_return(Owner::Taxpayer);
    match profile {
        "business" => (draft, TaxSection::SelfEmployment, 2400),
        "rental" => {
            let mut entry = rental_sale_return(true).income.pop().unwrap();
            let FederalIncome::Rental { data } = &mut entry.income else { panic!() };
            let RentalSale::Sold { price, .. } = &mut data.sale else { panic!() };
            // A new lower-income variation of fixture F keeps NIIT absent;
            // the original fixture and its 7200 recapture assertion stay intact.
            *price = dollars_of(650_000);
            draft.income.pop();
            draft.next_id = 3;
            add(&mut draft, entry);
            (draft, TaxSection::Rentals, 4800)
        }
        _ => panic!("unknown direct recapture profile"),
    }
}

#[test]
fn unit13_direct_recapture_actual_client_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::{accepted, open_topic};
    let profile =
        std::env::var("TAX_UNIT13_RECAPTURE_PROFILE").unwrap_or_else(|_| "business".into());
    let (draft, topic, expected) = unit13_direct_recapture_client_fixture(&profile);
    let before = supported(&draft);
    assert_eq!(value(&before, "net_investment_income_tax"), 0);
    assert_eq!(
        value(&before, "form_8379_line_18_b"),
        if profile == "business" { 3600 } else { 7200 }
    );
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = open_topic(&mut session, topic);
    if let Ok(input) = std::env::var("TAX_UNIT13_RECAPTURE_CALC_INPUT") {
        let bytes = std::fs::read(input).unwrap();
        let request = P::from_avro(&bytes).unwrap();
        assert_eq!(request.to_avro(), bytes);
        let Reply::Response(response) = send(&mut session, request) else { panic!() };
        std::fs::write(
            std::env::var("TAX_UNIT13_RECAPTURE_CALC_OUTPUT").unwrap(),
            response.to_avro(),
        )
        .unwrap();
        return;
    }
    if let Ok(dir) = std::env::var("TAX_UNIT13_RECAPTURE_SEED_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(std::path::Path::new(&dir).join("seed.avro"), page.to_avro()).unwrap();
        return;
    }
    let request = if let Ok(input) = std::env::var("TAX_UNIT13_RECAPTURE_PAYLOAD") {
        let bytes = std::fs::read(input).unwrap();
        let request = P::from_avro(&bytes).unwrap();
        assert_eq!(request.to_avro(), bytes);
        request
    } else {
        let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
        let row = &mut documents.documents[0];
        row.owner = Owner::Spouse;
        match &mut row.document {
            crate::IncomeRecord::Business(business) => {
                let InvestmentCredit::EarlierEnergy { used, .. } =
                    &mut business.assets[0].investment_credit
                else {
                    panic!()
                };
                *used = dollars_of(4000);
            }
            crate::IncomeRecord::Rental(rental) => {
                rental.more.improvements[0].rehabilitation.as_mut().unwrap().earlier[1].used =
                    dollars_of(2000);
            }
            _ => panic!(),
        }
        page
    };
    accepted(&send(&mut session, request));
    let check = |draft: &FederalDraft| {
        let result = supported(draft);
        assert_eq!(value(&result, "investment_credit_recapture_tax"), expected);
        assert_eq!(value(&result, "form_8379_line_18_a"), expected);
        assert_eq!(value(&result, "form_8379_line_18_b"), 0);
        assert_eq!(value(&result, "form_8379_line_18_c"), expected);
        assert!(!has(&result, "form_8379_injured_refund_estimate"));
        let mut spouse_injured = draft.clone();
        spouse_injured.injured_spouse.as_mut().unwrap().injured = Owner::Spouse;
        let swapped = supported(&spouse_injured);
        assert_eq!(value(&swapped, "form_8379_line_18_b"), expected);
        assert_eq!(value(&swapped, "form_8379_line_18_c"), 0);
        let mut plain = draft.clone();
        plain.injured_spouse = None;
        let ordinary = supported(&plain);
        assert_eq!(
            (ordinary.tax, ordinary.refund, ordinary.balance),
            (result.tax, result.refund, result.balance)
        );
        result
    };
    let result = check(&session.draft);
    let stored = native_draft_records(&session.draft);
    let draft = draft_from_native_records(session.draft.tax_year, &stored);
    assert_eq!(native_draft_records(&draft), stored);
    assert_eq!(draft, session.draft);
    let mut reopened = restart(draft);
    let shown = open_topic(&mut reopened, topic);
    if let Ok(dir) = std::env::var("TAX_UNIT13_RECAPTURE_OUTPUT") {
        std::fs::create_dir_all(&dir).unwrap();
        let dir = std::path::Path::new(&dir);
        std::fs::write(dir.join("reopened.avro"), shown.to_avro()).unwrap();
        write_native_draft_records(dir.join("persisted.avro"), &stored);
    }
    let accepted_draft = reopened.draft.clone();
    accepted(&send(&mut reopened, shown));
    assert_eq!(reopened.draft, accepted_draft);
    let after = check(&reopened.draft);
    assert_eq!((after.tax, after.refund), (result.tax, result.refund));
}

#[test]
fn straight_line_on_cost_is_the_kernel_schedule_with_one_rounding_per_year() {
    // $200,000.15 placed March 2024, disposed June 2026: 2024 and 2025 are rounded full years;
    // the 11/24 disposal year is the exact kernel slice, rounded once (not 11/24 of a rounded year).
    let mut building = new_building(200_000, 20240310, None);
    building.cost = 20_000_015;
    assert_eq!(building.straight_line_on_cost(20260630), 1_154_434);
}
