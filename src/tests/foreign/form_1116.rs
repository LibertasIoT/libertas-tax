use crate::tests::support::assert::line_amount as ratio;
// The five fixtures of the Form 1116 patterns map (batch B9, unit 6), worked
// by hand from the 2026 brackets, with every emitted line footing, the stored
// round trip and the corpus check of the refusals the unit removed.
use super::*;
use crate::tests::foreign::foreign::{
    add_named, fund_dividends, k3_answer, k3_blank, k3_partner, k3_row,
};

// A ratio line holds ten-thousandths, not cents.

pub(in crate::tests) fn row(
    country: &str,
    category: ForeignTaxCategory,
    income: i64,
    tax: i64,
) -> ForeignOrdinaryTax {
    ForeignOrdinaryTax {
        label: alloc::format!("Synthetic {country}"),
        country: country.into(),
        owner: Owner::Taxpayer,
        category,
        income: income * 100,
        tax: tax * 100,
    }
}

fn passive(withholding: ForeignWithholding) -> ForeignTaxCategory {
    ForeignTaxCategory::Passive { withholding, qualified: 0, sales: vec![] }
}

pub(in crate::tests) fn claim(
    sources: Vec<ForeignOrdinaryTax>,
    history: ForeignCreditHistory,
    carryback: ForeignCarryback,
) -> ForeignTaxClaim {
    ForeignTaxClaim::Ordinary {
        basis: ForeignTaxBasis::Paid,
        sources,
        situations: vec![],
        history,
        carryback,
        amt_limitation: ForeignAmtLimitation::Normal,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        rate_adjustment: ForeignRateAdjustment::Exception,
    }
}

pub(in crate::tests) fn no_carryback() -> ForeignCarryback {
    ForeignCarryback::PriorReturn { categories: vec![], confirmed: true }
}

// Every printed Form 1116 line of a record foots: 3c, 6, 7, 11, 14, 15, 17.
fn assert_lines_foot(r: &FederalResult) {
    let by = |id: &str| r.lines.iter().find(|l| l.id == id).map(|l| l.amount);
    for l in &r.lines {
        let Some(prefix) = l.id.strip_suffix("_total_taxable_income") else {
            continue;
        };
        let total_1a = by(&alloc::format!("{prefix}_total_gross_income")).unwrap();
        let total_6 = by(&alloc::format!("{prefix}_total_total_deductions")).unwrap();
        assert_eq!(total_1a - total_6, l.amount, "{prefix} line 7");
        // Line 15 is the Total column's line 7; 17 = 15 + 16.
        assert_eq!(
            by(&alloc::format!("{prefix}_foreign_income")).unwrap(),
            l.amount,
            "{prefix} line 15"
        );
        let line_16 = by(&alloc::format!("{prefix}_income_adjustments")).unwrap();
        assert_eq!(
            by(&alloc::format!("{prefix}_net_foreign_income")).unwrap(),
            l.amount + line_16,
            "{prefix} line 17"
        );
        let taxes = by(&alloc::format!("{prefix}_taxes_total")).unwrap();
        let carry = by(&alloc::format!("{prefix}_carryover")).unwrap();
        assert_eq!(
            by(&alloc::format!("{prefix}_taxes_with_carryover")).unwrap(),
            taxes + carry,
            "{prefix} line 11"
        );
        assert_eq!(
            by(&alloc::format!("{prefix}_taxes_available")).unwrap(),
            taxes
                + carry
                + by(&alloc::format!("{prefix}_tax_reduction")).unwrap()
                + by(&alloc::format!("{prefix}_high_tax_kickout")).unwrap(),
            "{prefix} line 14"
        );
        if let (Some(a), Some(b), Some(c)) = (
            by(&alloc::format!("{prefix}_credit_limitation")),
            by(&alloc::format!("{prefix}_taxes_available")),
            by(&alloc::format!("{prefix}_category_credit")),
        ) {
            assert_eq!(c, a.min(b), "{prefix} line 24");
        }
    }
    // Every column: 3c = 3a + 3b and 6 = 2 + 3g + 4a + 4b + 5.
    for l in r.lines.iter().filter(|l| l.id.ends_with("_total_deductions")) {
        let Some(prefix) = l.id.strip_suffix("_total_deductions") else {
            continue;
        };
        if prefix.ends_with("_total") || prefix.ends_with("_htko") {
            continue;
        }
        let part = |n: &str| by(&alloc::format!("{prefix}_{n}")).unwrap();
        assert_eq!(
            l.amount,
            part("related_expenses")
                + part("apportioned_deductions")
                + part("mortgage_interest")
                + part("other_interest")
                + part("foreign_losses"),
            "{prefix} line 6"
        );
        assert_eq!(
            part("deductions_subtotal"),
            part("itemized_or_standard") + part("other_deductions"),
            "{prefix} line 3c"
        );
    }
    // Part IV: 32 = sum of the categories; 33 = smaller of 20 and 32; 35.
    let credits: i64 = ["branch", "passive", "general"]
        .iter()
        .map(|c| by(&alloc::format!("foreign_credit_{c}")).unwrap())
        .sum();
    assert_eq!(by("foreign_credit_sum").unwrap(), credits);
    assert_eq!(by("foreign_credit_limited"), by("foreign_tax_credit"));
}

// Schedule B lines 3 and 8 foot in every column.
fn assert_schedule_b_foots(r: &FederalResult, category: &str, system: &str) {
    let by = |id: &str| r.lines.iter().find(|l| l.id == id).map_or(0, |l| l.amount);
    for l in &r.lines {
        let Some(prefix) = l.id.strip_suffix("_carry_opening") else {
            continue;
        };
        if !prefix.starts_with(&alloc::format!("foreign_{category}_{system}_")) {
            continue;
        }
        let adjusted = by(&alloc::format!("{prefix}_carry_adjusted"));
        assert_eq!(
            adjusted,
            l.amount
                + by(&alloc::format!("{prefix}_carryback_adjustment"))
                + by(&alloc::format!("{prefix}_redetermination"))
                + by(&alloc::format!("{prefix}_other_adjustments"))
        );
        assert_eq!(
            by(&alloc::format!("{prefix}_carry_remaining")),
            adjusted
                - by(&alloc::format!("{prefix}_carry_used"))
                - by(&alloc::format!("{prefix}_carry_expired"))
        );
    }
}

#[test]
fn form_1116_f1_two_categories_with_a_2024_general_carryover() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(1, Owner::Taxpayer, 12_000_000, 0));
    add(&mut d, income(foreign_pay(40_000)));
    add(&mut d, income(fund_dividends(2_000, 0, 0)));
    d.credits.as_mut().unwrap().foreign_tax = claim(
        vec![
            row("FR", ForeignTaxCategory::Employment, 40_000, 5_000),
            row("GM", passive(ForeignWithholding::AtLeast15), 2_000, 300),
        ],
        foreign_history_of(&[foreign_balance(2024, ForeignCreditCategory::General, 1_500, 0)]),
        no_carryback(),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 162_000);
    assert_eq!(value(&r, "taxable_income"), 145_900);
    // Line 20 and line 3e.
    assert_eq!(value(&r, "foreign_general_us_income_tax"), 27_614);
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 162_000);
    // General (FR).
    assert_eq!(value(&r, "foreign_general_FR_gross_income"), 40_000);
    assert_eq!(value(&r, "foreign_general_FR_itemized_or_standard"), 16_100);
    assert_eq!(ratio(&r, "foreign_general_FR_gross_income_ratio"), 2_469);
    assert_eq!(value(&r, "foreign_general_FR_apportioned_deductions"), 3_975);
    assert_eq!(value(&r, "foreign_general_total_total_deductions"), 3_975);
    assert_eq!(value(&r, "foreign_general_total_taxable_income"), 36_025);
    assert_eq!(value(&r, "foreign_general_FR_taxes"), 5_000);
    assert_eq!(value(&r, "foreign_general_carryover"), 1_500);
    assert_eq!(value(&r, "foreign_general_taxes_available"), 6_500);
    assert_eq!(ratio(&r, "foreign_general_limitation_ratio"), 2_469);
    assert_eq!(value(&r, "foreign_general_maximum_credit"), 6_818);
    assert_eq!(value(&r, "foreign_general_category_credit"), 6_500);
    // Schedule B, general, regular: the 2024 column is used up.
    assert_eq!(value(&r, "foreign_general_regular_2024_carry_opening"), 1_500);
    assert_eq!(value(&r, "foreign_general_regular_2024_carry_used"), 1_500);
    assert_eq!(value(&r, "foreign_general_regular_2024_carry_remaining"), 0);
    assert_eq!(value(&r, "foreign_general_regular_carry_adjusted_total"), 1_500);
    assert_eq!(value(&r, "foreign_general_unused_tax"), 0);
    // Passive (GM): stays passive (300 is under 37% of 1,801).
    assert_eq!(value(&r, "foreign_passive_GM_gross_income"), 2_000);
    assert_eq!(value(&r, "foreign_passive_GM_apportioned_deductions"), 199);
    assert_eq!(value(&r, "foreign_passive_total_taxable_income"), 1_801);
    assert_eq!(value(&r, "foreign_passive_maximum_credit"), 341);
    assert_eq!(value(&r, "foreign_passive_category_credit"), 300);
    // Part IV and the tax.
    assert_eq!(value(&r, "foreign_credit_passive"), 300);
    assert_eq!(value(&r, "foreign_credit_general"), 6_500);
    assert_eq!(value(&r, "foreign_credit_sum"), 6_800);
    assert_eq!(value(&r, "foreign_tax_credit"), 6_800);
    assert_eq!(r.tax, Some(2_081_400));
    // AMT: TMT 26% x 71,900 = 18,694, general 4,616, passive 231; no AMT; the
    // AMT Schedule B excess is general 384 and passive 69.
    assert_eq!(value(&r, "foreign_amt_general_maximum_credit"), 4_616);
    assert_eq!(value(&r, "foreign_amt_passive_maximum_credit"), 231);
    assert_eq!(value(&r, "foreign_general_amt_unused_tax"), 384);
    assert_eq!(value(&r, "foreign_passive_amt_unused_tax"), 69);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    assert_lines_foot(&r);
    assert_schedule_b_foots(&r, "general", "regular");
    assert_eq!(supported(&restart(d.clone()).draft), r);
    // A balance that arose on a joint return with someone else is left out.
    if let ForeignTaxClaim::Ordinary { history: ForeignCreditHistory::Retained(records), .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        records.same_filing_unit = false;
    }
    let r = supported(&d);
    assert!(noted(&r, "WARN_FOREIGN_CREDIT_CARRYOVER_LEFT_OUT"));
    assert_eq!(value(&r, "foreign_general_category_credit"), 5_000);
}

#[test]
fn form_1116_f2_k3_part_iii_feed_with_two_countries() {
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, wage(1, Owner::Taxpayer, 15_000_000, 0));
    let mut k3 = k3_blank();
    k3.countries = vec![
        k3_row("JA", K3Category::Passive, 5_000, 0, 750),
        k3_row("UK", K3Category::Passive, 3_000, 0, 150),
    ];
    k3.withholding = vec![
        k3_answer("JA", ForeignWithholding::AtLeast15),
        k3_answer("UK", ForeignWithholding::Below15),
    ];
    k3.passive.reduction = 8_000;
    add_named(
        &mut d,
        "Synthetic Global Fund LP",
        FederalIncome::Partnership { data: k3_partner(0, 8_000, 900, Some(k3)) },
    );
    d.credits.as_mut().unwrap().foreign_tax =
        claim(vec![], ForeignCreditHistory::None, ForeignCarryback::Unreviewed);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 158_000);
    assert_eq!(value(&r, "taxable_income"), 125_800);
    assert_eq!(value(&r, "regular_income_tax"), 17_100);
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 158_000);
    // The reduction is spread by income: JA 50, UK 30.
    assert_eq!(value(&r, "foreign_passive_JA_gross_income"), 5_000);
    assert_eq!(value(&r, "foreign_passive_JA_apportioned_deductions"), 1_019);
    assert_eq!(value(&r, "foreign_passive_UK_gross_income"), 3_000);
    assert_eq!(value(&r, "foreign_passive_UK_apportioned_deductions"), 611);
    assert_eq!(value(&r, "foreign_passive_total_gross_income"), 8_000);
    assert_eq!(value(&r, "foreign_passive_total_total_deductions"), 1_630);
    assert_eq!(value(&r, "foreign_passive_total_taxable_income"), 6_370);
    assert_eq!(value(&r, "foreign_passive_JA_taxes"), 750);
    assert_eq!(value(&r, "foreign_passive_UK_taxes"), 150);
    assert_eq!(value(&r, "foreign_passive_taxes_total"), 900);
    assert_eq!(value(&r, "foreign_passive_tax_reduction"), -80);
    assert_eq!(value(&r, "foreign_passive_taxes_available"), 820);
    assert_eq!(value(&r, "foreign_passive_maximum_credit"), 866);
    assert_eq!(value(&r, "foreign_passive_category_credit"), 820);
    assert_eq!(value(&r, "foreign_tax_credit"), 820);
    assert_eq!(r.tax, Some(1_628_000));
    assert_lines_foot(&r);
    assert_eq!(supported(&restart(d.clone()).draft), r);
}

#[test]
fn form_1116_f3_capital_gain_rate_differential_with_net_long_term_gain() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(1, Owner::Taxpayer, 18_000_000, 0));
    add(&mut d, income(ordinary_investment_sale(50_000, 30_000, GainTerm::Long)));
    let sale = d.income.last().unwrap().id;
    d.credits.as_mut().unwrap().foreign_tax = claim(
        vec![row(
            "CA",
            ForeignTaxCategory::Passive {
                withholding: ForeignWithholding::None,
                qualified: 0,
                sales: vec![sale],
            },
            0,
            3_000,
        )],
        ForeignCreditHistory::None,
        no_carryback(),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 200_000);
    assert_eq!(value(&r, "taxable_income"), 183_900);
    assert_eq!(value(&r, "regular_income_tax"), 34_934);
    // Worksheet B: 20,000 x 0.4054 = 8,108 on line 1a.
    assert_eq!(value(&r, "foreign_passive_CA_gross_income"), 8_108);
    assert_eq!(value(&r, "foreign_passive_CA_gross_foreign_income"), 20_000);
    assert_eq!(value(&r, "foreign_passive_CA_gross_income_all_sources"), 200_000);
    assert_eq!(ratio(&r, "foreign_passive_CA_gross_income_ratio"), 1_000);
    assert_eq!(value(&r, "foreign_passive_CA_apportioned_deductions"), 1_610);
    assert_eq!(value(&r, "foreign_passive_total_taxable_income"), 6_498);
    // The Worksheet for Line 18: 183,900 - 20,000 x 0.5946 = 172,008.
    assert_eq!(value(&r, "foreign_passive_worldwide_income"), 172_008);
    assert_eq!(ratio(&r, "foreign_passive_limitation_ratio"), 378);
    assert_eq!(value(&r, "foreign_passive_maximum_credit"), 1_320);
    assert_eq!(value(&r, "foreign_tax_credit"), 1_320);
    assert_eq!(value(&r, "foreign_passive_unused_tax"), 1_680);
    assert_eq!(r.tax, Some(3_361_400));
    assert_lines_foot(&r);
    // Without the sale chosen the gain is U.S. income: no foreign income.
    // The exception at $19,999 of foreign gain: no adjustment, no worksheet.
    let mut near = d.clone();
    near.income.last_mut().unwrap().income =
        ordinary_investment_sale(49_999, 30_000, GainTerm::Long);
    let r = supported(&near);
    assert_eq!(value(&r, "foreign_passive_CA_gross_income"), 19_999);
}

#[test]
fn form_1116_f4_high_taxed_passive_interest_moves_to_general() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(120_000)));
    add(&mut d, income(savings_income(4_000, 0)));
    add(&mut d, income(savings_income(1_000, 0)));
    d.credits.as_mut().unwrap().foreign_tax = claim(
        vec![
            row("UK", ForeignTaxCategory::Employment, 120_000, 22_000),
            row("UK", passive(ForeignWithholding::None), 4_000, 1_600),
        ],
        ForeignCreditHistory::None,
        no_carryback(),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 125_000);
    assert_eq!(value(&r, "taxable_income"), 108_900);
    assert_eq!(value(&r, "regular_income_tax"), 18_734);
    // Passive record: the UK column keeps the interest, the HTKO column takes
    // it out.
    assert_eq!(value(&r, "foreign_passive_UK_gross_income"), 4_000);
    assert_eq!(value(&r, "foreign_passive_UK_total_deductions"), 515);
    assert_eq!(value(&r, "foreign_passive_UK_taxable_income"), 3_485);
    assert_eq!(value(&r, "foreign_passive_htko_gross_income"), -4_000);
    assert_eq!(value(&r, "foreign_passive_htko_total_deductions"), -515);
    assert_eq!(value(&r, "foreign_passive_total_taxable_income"), 0);
    assert_eq!(value(&r, "foreign_passive_taxes_for_category"), 1_600);
    assert_eq!(value(&r, "foreign_passive_high_tax_kickout"), -1_600);
    assert_eq!(value(&r, "foreign_passive_taxes_available"), 0);
    assert_eq!(value(&r, "foreign_passive_category_credit"), 0);
    // General record: the HTKO column is positive.
    assert_eq!(value(&r, "foreign_general_UK_gross_income"), 120_000);
    assert_eq!(value(&r, "foreign_general_UK_apportioned_deductions"), 15_456);
    assert_eq!(value(&r, "foreign_general_UK_taxable_income"), 104_544);
    assert_eq!(value(&r, "foreign_general_htko_gross_income"), 4_000);
    assert_eq!(value(&r, "foreign_general_htko_total_deductions"), 515);
    assert_eq!(value(&r, "foreign_general_total_taxable_income"), 108_029);
    assert_eq!(value(&r, "foreign_general_taxes_for_category"), 22_000);
    assert_eq!(value(&r, "foreign_general_high_tax_kickout"), 1_600);
    assert_eq!(value(&r, "foreign_general_taxes_available"), 23_600);
    assert_eq!(value(&r, "foreign_general_maximum_credit"), 18_584);
    assert_eq!(value(&r, "foreign_tax_credit"), 18_584);
    assert_eq!(r.tax, Some(15_000));
    assert_eq!(value(&r, "foreign_general_unused_tax"), 5_016);
    // AMT: the interest is high-taxed at 26% too; AMT general 9,001.
    assert_eq!(value(&r, "foreign_amt_general_maximum_credit"), 9_001);
    assert_lines_foot(&r);
    assert_eq!(supported(&restart(d.clone()).draft), r);
}

#[test]
fn form_1116_f5_business_run_abroad_with_its_own_books_is_foreign_branch() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(excluded_foreign_business(160_000, 30_000)));
    let business = d.income[0].id;
    add(&mut d, income(savings_income(2_000, 0)));
    d.credits.as_mut().unwrap().foreign_tax = claim(
        vec![row("MX", ForeignTaxCategory::Business { business, separate_books: true }, 0, 10_500)],
        ForeignCreditHistory::None,
        no_carryback(),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 18_368);
    assert_eq!(value(&r, "agi"), 122_816);
    assert_eq!(value(&r, "taxable_income"), 106_716);
    assert_eq!(value(&r, "regular_income_tax"), 18_210);
    // Foreign branch category (box b), Mexico.
    assert_eq!(value(&r, "foreign_branch_MX_gross_income"), 160_000);
    assert_eq!(value(&r, "foreign_branch_MX_related_expenses"), 39_184);
    assert_eq!(value(&r, "foreign_branch_MX_itemized_or_standard"), 16_100);
    assert_eq!(value(&r, "foreign_branch_MX_gross_foreign_income"), 160_000);
    assert_eq!(value(&r, "foreign_branch_MX_gross_income_all_sources"), 162_000);
    assert_eq!(value(&r, "foreign_branch_MX_apportioned_deductions"), 15_901);
    assert_eq!(value(&r, "foreign_branch_total_total_deductions"), 55_085);
    assert_eq!(value(&r, "foreign_branch_total_taxable_income"), 104_915);
    assert_eq!(value(&r, "foreign_branch_taxes_available"), 10_500);
    assert_eq!(value(&r, "foreign_branch_maximum_credit"), 17_903);
    assert_eq!(value(&r, "foreign_branch_category_credit"), 10_500);
    assert_eq!(value(&r, "foreign_credit_branch"), 10_500);
    assert_eq!(value(&r, "foreign_tax_credit"), 10_500);
    // AMT: TMT 26% x 32,716 = 8,506; AMT branch 8,367; excess 2,133.
    assert_eq!(value(&r, "foreign_amt_branch_maximum_credit"), 8_367);
    assert_eq!(value(&r, "foreign_branch_amt_unused_tax"), 2_133);
    assert_lines_foot(&r);
    // Without separate books the same business is general category income.
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources[0].category = ForeignTaxCategory::Business { business, separate_books: false };
    }
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_general_MX_gross_income"), 160_000);
    assert_eq!(value(&r, "foreign_credit_general"), value(&r, "foreign_tax_credit"));
    assert_eq!(value(&r, "foreign_credit_branch"), 0);
}

#[test]
fn a_row_on_a_section_901j_country_is_category_e_with_no_credit() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(1, Owner::Taxpayer, 10_000_000, 0));
    add(&mut d, income(fund_dividends(2_000, 0, 0)));
    d.credits.as_mut().unwrap().foreign_tax = claim(
        vec![row("IR", passive(ForeignWithholding::AtLeast15), 2_000, 300)],
        ForeignCreditHistory::None,
        no_carryback(),
    );
    let r = supported(&d);
    assert!(noted(&r, "WARN_FTC_SANCTIONED_COUNTRY_TAX"));
    assert_eq!(value(&r, "foreign_sanctioned_IR_gross_income"), 2_000);
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert_eq!(value(&r, "foreign_credit_sanctioned"), 0);
}

#[test]
fn accrued_taxes_refuse_an_earlier_years_tax_on_both_sides() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(1)));
    let mut accrued = claim(
        vec![row("FR", ForeignTaxCategory::PriorEmployeeWageTax { income_year: 2025 }, 0, 100)],
        ForeignCreditHistory::None,
        no_carryback(),
    );
    assert_eq!(crate::form_1116::claim_rule(&accrued, &[]).map(|issue| issue.key), None);
    if let ForeignTaxClaim::Ordinary { basis, .. } = &mut accrued {
        *basis = ForeignTaxBasis::Accrued;
    }
    assert_eq!(
        crate::form_1116::claim_rule(&accrued, &[]).map(|issue| issue.key),
        Some("FOREIGN_PRIOR_TAX_NEEDS_PAID_BASIS")
    );
    // The page's server twin refuses the save with the same key.
    let mut s = Interview::new(TEST_USER, d);
    let reply = s
        .handle(P::EstimateMoreCredits {
            credit_income_sources: crate::tax_credit_bonds::income_sources(&s.draft),
            allowed_credit_owners: vec![Owner::Taxpayer as i32],
            initial_carbon_oxide_origins: vec![],
            initial_renewable_facility_origins: Vec::new(),
            businesses: vec![],
            farms: vec![],
            clean_vehicle_properties: vec![],
            rentals: vec![],
            research_businesses: vec![],
            entered_credits: vec![],
            carryback_credits: vec![],
            allowed_carry_filers: vec![],
            initial_credit_elections: vec![],
            initial_research_choices: vec![],
            user: TEST_USER,
            allowed_owners: vec![Owner::Taxpayer as i32],
            foreign_businesses: vec![],
            estate_foreign_sources: vec![],
            capital_sales: vec![],
            foreign_situations: vec![],
            mortgage_loans: vec![],
            value: MoreCredits {
                answers: Vec::new(),
                credits: vec![MoreCredit::ForeignTax(accrued.try_into().unwrap())],

                documents: Vec::new(),
            },
        })
        .unwrap();
    let Reply::Response(P::Problem { .. }) = reply else {
        panic!("the estimate is refused: {reply:?}")
    };
}

// The refusals this unit removed: no fixture of the corpus reaches one, and
// their resources are gone.
#[test]
fn no_fixture_still_reaches_a_removed_form_1116_refusal() {
    let removed = [
        "ISSUE_FTC_FOREIGN_BUSINESS",
        "ISSUE_FTC_FARM_WAGERING",
        "ISSUE_FTC_LOSS_DEDUCTIONS",
        "ISSUE_FTC_PASS_THROUGH",
        "ISSUE_FTC_K3_INTEREST",
        "ISSUE_FTC_PROPERTY",
        "ISSUE_FTC_OTHER_ITEMIZED",
        "ISSUE_FTC_ITEMIZED_ABOVE_AGI",
        "ISSUE_FTC_STANDARD_ADDITION",
        "ISSUE_FTC_AMT_ADJUSTMENTS",
        "ISSUE_FTC_OTHER_INTEREST",
        "ISSUE_FTC_FOREIGN_PENALTY",
        "ISSUE_NOT_CALCULATED_TREATY_RE_SOURCED_INCOME_LINE_1A_FOREIGN_TAX_REFUNDS_04F5C6AE",
        "ISSUE_FORM_1116_LINE_3E_AND_THE_LINE_18_WORKSHEET_ARE_CALCULATED_FOR_QUAL_06A2BA00",
        "ISSUE_REFIGURE_THE_FULL_FORM_1116_ALLOCATION_BEFORE_CALCULATING_THIS_ELEC_3B427F2A",
        "ISSUE_SUPPLY_CURRENT_YEAR_INVESTMENT_ASSET_BASES_OR_SELECT_THE_ELIGIBLE_S_09C57D69",
        "WARN_PRIOR_FOREIGN_TAX_LEFT_OUT",
        "WARN_INVESTMENT_INTEREST_SOURCE_LEFT_OUT",
    ];
    for key in removed {
        assert!(!crate::APP_STRINGS.iter().any(|(k, _)| *k == key), "{key} is still declared");
    }
    for fixture in [
        include_str!("../../../tests/fixtures/returns/m1-single-w2.json"),
        include_str!("../../../tests/fixtures/returns/bona-fide-resident-w2-at-limit.json"),
        include_str!("../../../tests/fixtures/returns/physical-presence-partial-year-housing.json"),
        include_str!("../../../tests/fixtures/returns/mfj-three-children-magi-phaseout.json"),
    ] {
        assert!(removed.iter().all(|key| !fixture.contains(key)));
    }
    // The five fixtures' claims and the old reproductions reach none of them.
    let r = estimate(&fixture_f1_claim_return());
    assert!(r.issues.iter().all(|i| !removed.contains(&i.display.key.as_str())));
}

fn fixture_f1_claim_return() -> FederalDraft {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(10_000)));
    d.credits.as_mut().unwrap().foreign_tax = claim(
        vec![row("FR", ForeignTaxCategory::Employment, 10_000, 1_000)],
        ForeignCreditHistory::None,
        ForeignCarryback::Unreviewed,
    );
    d
}

#[test]
fn a_negative_form_8978_line_14_decreases_line_20_and_the_credit_no_longer_competes() {
    // Wages low enough that the tentative minimum tax is zero: no AMT foreign credit, which
    // stays in the competition condition.
    let fixture = || {
        let mut d = wholly_foreign_credit_return(FederalStatus::Single, 60_000);
        if let ForeignTaxClaim::Ordinary { sources, .. } =
            &mut d.credits.as_mut().unwrap().foreign_tax
        {
            sources[0].tax = 500_000;
        }
        d
    };
    let base = supported(&fixture());
    let mut d = fixture();
    d.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(bba_interest_reduction()));
    let r = supported(&d);
    let decrease = -value(&r, "form_8978_line_14");
    assert!(decrease > 0);
    assert_eq!(
        value(&r, "foreign_general_us_income_tax"),
        value(&base, "foreign_general_us_income_tax") - decrease
    );
}

#[test]
fn a_k3_foreign_branch_column_and_its_capital_gain_feed_the_records() {
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, wage(1, Owner::Taxpayer, 15_000_000, 0));
    let mut k3 = k3_blank();
    k3.countries = vec![
        k3_row("MX", K3Category::ForeignBranch, 5_000, 0, 500),
        k3_row("JA", K3Category::Passive, 4_000, 0, 600),
    ];
    k3.withholding = vec![k3_answer("JA", ForeignWithholding::AtLeast15)];
    k3.branch.deductions = 100_000;
    // Part II line 12 of the passive column: the 4,000 of Japan is a long-term gain.
    k3.passive.capital.long_gain = 400_000;
    let mut partner = k3_partner(0, 0, 1_100, Some(k3));
    partner.separate.as_mut().unwrap().long = 400_000;
    add_named(&mut d, "Synthetic Global Fund LP", FederalIncome::Partnership { data: partner });
    d.credits.as_mut().unwrap().foreign_tax =
        claim(vec![], ForeignCreditHistory::None, ForeignCarryback::Unreviewed);
    // Below the adjustment exception's dollar limits the adjustment is elected away;
    // choosing to adjust applies Worksheet B.
    if let ForeignTaxClaim::Ordinary { rate_adjustment, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *rate_adjustment = ForeignRateAdjustment::Adjust;
    }
    let r = estimate(&d);
    assert!(r.issues.iter().all(|i| i.section != "Foreign tax credit"), "{:?}", r.issues);
    // Column (b): foreign branch category, line 1a its line 24 amount, line 2
    // its Section 2 deductions.
    assert_eq!(value(&r, "foreign_branch_MX_gross_income"), 5_000);
    assert_eq!(value(&r, "foreign_branch_MX_related_expenses"), 1_000);
    assert_eq!(value(&r, "foreign_branch_MX_taxes"), 500);
    // Line 12 of the passive column goes through Worksheet B: 4,000 x 0.4054.
    assert_eq!(value(&r, "foreign_passive_JA_gross_foreign_income"), 4_000);
    assert_eq!(value(&r, "foreign_passive_JA_gross_income"), 1_622);
    assert!(value(&r, "foreign_credit_branch") > 0);
}

#[test]
fn a_foreign_gain_taxed_at_a_second_rate_group_leaves_the_credit_out_with_a_note() {
    // A foreign long-term gain with a 0% band in the return needs Pub. 514's
    // rate-group method: no credit, one note.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(1, Owner::Taxpayer, 3_000_000, 0));
    add(&mut d, income(ordinary_investment_sale(60_000, 10_000, GainTerm::Long)));
    let sale = d.income.last().unwrap().id;
    d.credits.as_mut().unwrap().foreign_tax = claim(
        vec![row(
            "CA",
            ForeignTaxCategory::Passive {
                withholding: ForeignWithholding::None,
                qualified: 0,
                sales: vec![sale],
            },
            0,
            4_000,
        )],
        ForeignCreditHistory::None,
        no_carryback(),
    );
    let r = supported(&d);
    assert!(noted(&r, "WARN_FTC_CAPITAL_RATE_GROUPS_LEFT_OUT"));
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
}
