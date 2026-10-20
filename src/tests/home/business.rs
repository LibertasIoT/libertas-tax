use super::*;
use crate::tests::business::business::{acquisition, asset_sale_return, cents, press, sold};

#[test]
fn home_sale_allocates_nonqualified_gain_after_depreciation() {
    let mut d = complete(FederalStatus::Single);
    let mut home = home_sale(false);
    home.price = 70000000;
    home.expenses = 0;
    home.basis = 38000000;
    home.use_history =
        HomeSaleUse::Adjusted { depreciation: 2000000, owned_days: 2000, nonqualified_days: 800 };
    // 320,000 gain: 20,000 depreciation, 120,000 nonqualified use,
    // and 180,000 eligible for the exclusion (40% use allocation).
    assert_eq!(home.depreciation_gain(), 2000000);
    assert_eq!(home.nonqualified_gain(), 12000000);
    assert_eq!(home.excluded(FederalStatus::Single), Ok(18000000));
    add(&mut d, income(FederalIncome::HomeSale { data: home }));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 140000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 20000);
    assert_eq!(value(&r, "agi"), 200000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn home_depreciation_is_capped_at_gain_and_invalid_use_periods_are_rejected() {
    let mut home = home_sale(false);
    home.price = 26000000; // gain 10,000 after expenses and adjusted basis
    home.use_history =
        HomeSaleUse::Adjusted { depreciation: 2700000, owned_days: 1000, nonqualified_days: 400 };
    assert_eq!(home.depreciation_gain(), 1000000);
    assert_eq!(home.nonqualified_gain(), 0);
    assert_eq!(home.excluded(FederalStatus::Single), Ok(0));
    home.term = GainTerm::Short;
    assert!(home.excluded(FederalStatus::Single).is_err());
    let HomeSaleUse::Adjusted { owned_days, .. } = &mut home.use_history else { panic!() };
    *owned_days = 399;
    assert!(!home.valid());
    let HomeSaleUse::Adjusted { owned_days, .. } = &mut home.use_history else { panic!() };
    *owned_days = 0;
    assert!(!home.valid());
}

#[test]
fn former_home_mortgage_shortfall_is_nonbusiness_bad_debt_once_paid() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(former_home_mortgage_bad_debt(12000, 2000)));
    let r = supported(&d);
    assert_eq!(value(&r, "bad_debt_2_short_term_loss"), -10000);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 7000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn sole_qualified_home_improvement_loan_has_no_form_8828_recapture() {
    let mut d = complete(FederalStatus::Single);
    let ordinary_return = supported(&d);
    set_mortgage_subsidy(
        &mut d,
        MortgageSubsidyDisposition::ExemptHomeImprovementLoan {
            disposed_on: 20260701,
            original_principal: 1_500_000,
            qualified_sole_subsidy: true,
        },
    );
    let qualified = supported(&d);
    assert_eq!(value(&qualified, "mortgage_subsidy_recapture_tax"), 0);
    assert_eq!(qualified.tax, ordinary_return.tax);
    assert_eq!(supported(&restart(d.clone()).draft), qualified);

    if let MortgageSubsidyDisposition::ExemptHomeImprovementLoan {
        qualified_sole_subsidy, ..
    } = mortgage_disposition(&mut d)
    {
        *qualified_sole_subsidy = false;
    }
    unsupported(&d, "Mortgage subsidy recapture");
    if let MortgageSubsidyDisposition::ExemptHomeImprovementLoan {
        qualified_sole_subsidy,
        original_principal,
        ..
    } = mortgage_disposition(&mut d)
    {
        *qualified_sole_subsidy = true;
        *original_principal = 1_500_001;
    }
    assert!(!d.valid_stored());
}

// Boundary (3): the simplified home-office deduction caps at 300 square feet
// and separately at the business's net income before the deduction, so it can
// never create or increase a loss (the canonical home-use owner, i1040sc line 30).
#[test]
fn simplified_home_office_is_capped_by_area_and_income() {
    // $2,000 receipts, $1,500 ordinary expense: tentative profit (line 29) is
    // $500.00. A 400 sq ft home office is area-capped to 300 sq ft, so the
    // uncapped-by-income deduction is 300 x $5/sq ft = $1,500.00 for the year;
    // that exceeds the $500.00 tentative profit, so the deduction is further
    // capped at $500.00, leaving exactly $0 net profit (no loss is created).
    let mut d = empty_return(FederalStatus::Single);
    let mut business = cash_business(2000);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.expenses = vec![ordinary_business_expense(BusinessExpenseCategory::Supplies, 150_000)];
    add(&mut d, income(business));
    let source = d.income.last().unwrap().id;
    add_simplified_office(&mut d, source, 400);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_home_office"), 500);
    assert_eq!(return_line(&r, "8"), 0);
    assert_eq!(value(&r, "qbi_deduction"), 0);
}

// M3-F1 boundary (1 of 3): the simplified home-office deduction's 300 sq ft
// cap (schedule-c.md map; i1040sc line 30) binds by itself when business
// income is far above the uncapped area x $5 amount, isolated from the
// separate income cap that `simplified_home_office_is_capped_by_area_and_income`
// already covers.
#[test]
fn simplified_home_office_area_cap_binds_with_ample_income() {
    // 350 sq ft, all 12 months: uncapped would be 350 x $5 = $1,750.00;
    // capped at 300 sq ft x $5 = $1,500.00. Receipts $88,000, no other
    // expenses: tentative profit is $88,000.00, far above $1,500.00, so only
    // the area cap binds. Net profit: 88,000.00 - 1,500.00 = $86,500.00.
    let mut d = empty_return(FederalStatus::Single);
    let business = cash_business(88_000);
    add(&mut d, income(business));
    let source = d.income.last().unwrap().id;
    add_simplified_office(&mut d, source, 350);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_home_office"), 1_500);
    assert_eq!(return_line(&r, "8"), 8_650_000);
}

// Form 8829, actual-expense method (unit T2-39). Every expected line below is
// figured by hand from the 2025 Instructions for Form 8829 with the 2026
// amounts (state and local tax limit $40,400, floor $10,000; 39-year
// Table A-7a percentages), never from the App's output.

fn line_8829(r: &FederalResult, suffix: &str) -> i64 {
    cents(r, &office_line_id(r, suffix))
}
// The standard-route fixture's office: 300 of 2,000 square feet (15%), an
// owned home first used in 2020, and whole-home operating costs.

// Itemizing: the office's interest and taxes are also on Schedule A.

// The itemizing fixture's office: 250 of 2,500 square feet (10%).

#[test]
fn form_8829_standard_deduction_return_matches_hand_computation() {
    // Single, standard deduction, one business with $50,000 of receipts and
    // no other expense: Schedule C line 29 is $50,000.
    let d = office_return(50_000, standard_office());
    let r = supported(&d);
    // Part I: 300 / 2,000 = 15%. Line 8 $50,000. No amount on lines 10-11
    // with the standard deduction, so line 14 is $0 and line 15 $50,000.
    // Lines 16(b) $12,000 interest, 17(b) $4,000 taxes, 18 $1,200,
    // 20(a) $400, 20(b) $800, 21 $3,000; 23(a) $400; 23(b) $21,000;
    // 24 = 15% x $21,000 = $3,150; 26 = $400 + $3,150 = $3,550; 27 $3,550;
    // 28 = $50,000 - $3,550 = $46,450. Part III: 37 $300,000, 38 $60,000,
    // 39 $240,000, 40 = 15% = $36,000, 41 2.564% (first used 2020),
    // 42 = $923.04; 30-33 $923.04; 36 = $0 + $3,550 + $923.04 = $4,473.04.
    for (suffix, amount) in [
        ("income", 5_000_000),
        ("mortgage_interest", 0),
        ("real_estate_taxes", 0),
        ("unlimited", 0),
        ("limit", 5_000_000),
        ("excess_interest", 1_200_000),
        ("excess_taxes", 400_000),
        ("direct_total", 40_000),
        ("indirect_total", 2_100_000),
        ("operating_share", 315_000),
        ("operating_total", 355_000),
        ("operating_allowed", 355_000),
        ("depreciation_limit", 4_645_000),
        ("building", 24_000_000),
        ("business_basis", 3_600_000),
        ("depreciation_allowable", 92_304),
        ("depreciation_allowed", 92_304),
        ("allowed", 447_304),
        ("operating_carryover", 0),
        ("depreciation_carryover", 0),
    ] {
        assert_eq!(line_8829(&r, suffix), amount, "{suffix}");
    }
    let depreciation_line = office_line_id(&r, "depreciation_allowable");
    let explanation =
        &r.lines.iter().find(|line| line.id == depreciation_line).unwrap().explanation;
    assert_eq!(explanation.args, vec![MessageArg::Decimal { units: 2564, scale: 3 }]);
    assert!(explanation.english().contains("2.564%"));
    let encoded = explanation.encode();
    let mut offset = 0;
    let decoded = <crate::common::FormattedMessage as libertas::AvroDecode>::avro_decode(
        &encoded,
        &mut offset,
    )
    .unwrap();
    assert_eq!(offset, encoded.len());
    assert!(
        matches!(decoded.1[0], libertas::LibertasMessageArgumentDecode::Double(value) if (value - 2.564).abs() < 0.000001)
    );
    // Schedule C line 30 is line 36, shown in whole dollars ($4,473); line
    // 31 = $50,000 - $4,473.04 in cents.
    assert_eq!(cents(&r, "business_2_home_office"), 447_300);
    assert_eq!(cents(&r, "business_2_net_profit"), 4_552_696);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn form_8829_itemizing_return_moves_the_office_share_off_schedule_a() {
    // Single, itemizing: state income tax $9,000, real estate and personal
    // property taxes $8,000 (the home's $6,000 among them), Form 1098
    // interest $15,000 on the main home. Receipts $60,000, line 29 $60,000.
    let mut d = office_return(60_000, itemizing_office(20180105));
    office_itemize(&mut d, 900_000, 800_000, 1_500_000, 0);
    let r = supported(&d);
    // Part I: 250 / 2,500 = 10%. Line 10(b) $15,000 (within the Pub 936
    // limit). Line 11 Worksheet: 5 = $9,000 + $8,000 = $17,000; 6 = 10% of
    // $6,000 = $600; 7a $16,400; 7b No ($17,000 > $10,000); 7c No; 7d Yes;
    // 8 $40,400; 9 $24,000; 10 = the smaller of $600 and $24,000 = $600 on
    // line 11 column (a); line 17 = $0. Line 12(b) $15,000, 13 = $1,500,
    // 14 = $600 + $1,500 = $2,100, 15 = $57,900. 18 $1,000, 21 $2,400,
    // 23(b) $3,400, 24 $340, 26 $340, 27 $340, 28 $57,560. Part III: 39 =
    // $400,000 - $100,000 = $300,000, 40 $30,000, 41 2.564%, 42 $769.20;
    // 33 $769.20; 36 = $2,100 + $340 + $769.20 = $3,209.20.
    for (suffix, amount) in [
        ("income", 6_000_000),
        ("mortgage_interest", 1_500_000),
        ("real_estate_taxes", 60_000),
        ("indirect_share", 150_000),
        ("unlimited", 210_000),
        ("limit", 5_790_000),
        ("excess_interest", 0),
        ("excess_taxes", 0),
        ("indirect_total", 340_000),
        ("operating_share", 34_000),
        ("operating_allowed", 34_000),
        ("depreciation_allowable", 76_920),
        ("allowed", 320_920),
        ("worksheet_business_taxes", 60_000),
    ] {
        assert_eq!(line_8829(&r, suffix), amount, "{suffix}");
    }
    assert_eq!(cents(&r, "business_2_net_profit"), 5_679_080);
    // Schedule A: line 8a = $15,000 - 10% x $15,000 = $13,500; line 5 =
    // $17,000 - $600 = $16,400, under the $40,400 limit. Deduction
    // $16,400 + $13,500 = $29,900.
    assert_eq!(value(&r, "mortgage_interest_8a"), 13_500);
    assert_eq!(value(&r, "deduction"), 29_900);
}

// Boundary: the income limit (line 15) holds back operating expenses and
// depreciation, which carry forward on lines 43 and 44 with the prior
// carryovers (lines 25 and 31).
#[test]
fn form_8829_income_limit_carries_operating_and_depreciation_forward() {
    // Standard route, the fixture office with a $500 operating and a $100
    // depreciation carryover, and line 29 of only $2,000.
    let mut office = standard_office();
    office.prior_operating = 50_000;
    office.prior_depreciation = 10_000;
    // 2026-10-17 synthetic completion: the prior $100 is documented
    // depreciation only; the newly separate casualty-carry answer is zero.
    office.prior_casualty = 0;
    let r = supported(&office_return(2_000, office));
    // 15 = $2,000; 26 = $400 + $3,150 + $500 = $4,050; 27 = $2,000;
    // 28 = $0; 32 = $923.04 + $100 = $1,023.04; 33 = $0; 36 = $2,000;
    // 43 = $4,050 - $2,000 = $2,050; 44 = $1,023.04. Line 31 is $0.
    for (suffix, amount) in [
        ("limit", 200_000),
        ("operating_total", 405_000),
        ("operating_allowed", 200_000),
        ("depreciation_limit", 0),
        ("depreciation_total", 102_304),
        ("depreciation_allowed", 0),
        ("allowed", 200_000),
        ("operating_carryover", 205_000),
        ("depreciation_carryover", 102_304),
    ] {
        assert_eq!(line_8829(&r, suffix), amount, "{suffix}");
    }
    assert_eq!(cents(&r, "business_2_net_profit"), 0);
}

// Boundary: on the itemizing route lines 10-11 are not limited by line 8, so
// the office can create a loss of that amount and nothing more.
#[test]
fn form_8829_itemized_interest_and_taxes_can_exceed_line_8() {
    // A $50,000 W-2 as well, so the business loss is not a net operating loss.
    let mut d = office_return(1_000, itemizing_office(20180105));
    add(&mut d, wage(0, Owner::Taxpayer, 5_000_000, 0));
    office_itemize(&mut d, 900_000, 800_000, 1_500_000, 0);
    let r = supported(&d);
    // 8 = $1,000; 14 = $2,100; 15 = $0; 27 = $0; 33 = $0; 36 = $2,100;
    // 43 = $340; 44 = $769.20; Schedule C line 31 = $1,000 - $2,100.
    for (suffix, amount) in [
        ("limit", 0),
        ("operating_allowed", 0),
        ("depreciation_allowed", 0),
        ("allowed", 210_000),
        ("operating_carryover", 34_000),
        ("depreciation_carryover", 76_920),
    ] {
        assert_eq!(line_8829(&r, suffix), amount, "{suffix}");
    }
    assert_eq!(cents(&r, "business_2_net_profit"), -110_000);
}

// Boundary: Line 11 Worksheet line 7c. Personal state and local taxes already
// at the $40,400 limit leave no room, so the business share moves to line 17.
#[test]
fn form_8829_taxes_above_the_salt_limit_move_to_line_17() {
    let mut d = office_return(60_000, itemizing_office(20180105));
    office_itemize(&mut d, 4_000_000, 800_000, 1_500_000, 0);
    let r = supported(&d);
    // 5 = $48,000; 6 = $600; 7a = $47,400 >= $40,400, so 8 = $10,000;
    // 9 = $0; line 11 = $0; line 17(a) = $600. 14 = $1,500; 23(a) = $600;
    // 26 = $600 + $340 = $940; 36 = $1,500 + $940 + $769.20 = $3,209.20.
    for (suffix, amount) in [
        ("real_estate_taxes", 0),
        ("excess_taxes", 60_000),
        ("unlimited", 150_000),
        ("direct_total", 60_000),
        ("operating_total", 94_000),
        ("allowed", 320_920),
    ] {
        assert_eq!(line_8829(&r, suffix), amount, "{suffix}");
    }
}

// Boundary: line 41 for a home first used for business in 2026 uses the
// first-use month's percentage (July, 1.177%).
#[test]
fn form_8829_first_use_in_2026_uses_the_month_percentage() {
    let mut d = office_return(60_000, itemizing_office(20260710));
    office_itemize(&mut d, 900_000, 800_000, 1_500_000, 0);
    let r = supported(&d);
    // 40 = $30,000; 42 = $30,000 x 1.177% = $353.10.
    assert_eq!(line_8829(&r, "depreciation_allowable"), 35_310);
    assert_eq!(line_8829(&r, "allowed"), 210_000 + 34_000 + 35_310);
}

// Boundary: line 8 subtracts this business's Form 4797 loss on an asset,
// which does not come from the use of the home.
#[test]
fn form_8829_line_8_subtracts_the_business_asset_loss() {
    // The T2-35 press sold for $1,000: adjusted basis $3,840, a $2,840 loss.
    // Line 29 = $150,000 - $960 of 2026 depreciation = $149,040; line 8 =
    // $149,040 - $2,840 = $146,200.
    let mut d = asset_sale_return(vec![press(sold(20260715, 100_000, 0))]);
    let source = d
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Business { .. }))
        .unwrap()
        .id;
    add_actual_office(&mut d, source, standard_office());
    let r = supported(&d);
    assert_eq!(line_8829(&r, "income"), 14_620_000);
}

#[test]
fn form_8829_automatic_route_uses_each_routes_columns() {
    // The itemizing fixture under the automatic comparison: itemizing
    // ($29,900) beats the $16,100 standard deduction, so the itemizing
    // route's lines are the result.
    let mut d = office_return(60_000, itemizing_office(20180105));
    office_itemize(&mut d, 900_000, 800_000, 1_500_000, 0);
    let v = d.deductions.as_mut().unwrap();
    let DeductionSelection::Itemized { facts } = v.choice.clone() else { panic!() };
    v.choice = DeductionSelection::Automatic { expenses: ItemizedChoice::Yes { facts } };
    let r = supported(&d);
    assert_eq!(line_8829(&r, "mortgage_interest"), 1_500_000);
    assert_eq!(line_8829(&r, "allowed"), 320_920);
    assert_eq!(value(&r, "deduction"), 29_900);
    // With small itemized amounts the standard route wins, with every
    // interest and tax amount on lines 16 and 17.
    let mut small =
        actual_office(250, 2_500, owned_office_home(100_000, 0, 50_000, 0, 0, 20180105));
    small.utilities = 0;
    let mut d = office_return(60_000, small);
    office_itemize(&mut d, 0, 50_000, 100_000, 0);
    let v = d.deductions.as_mut().unwrap();
    let DeductionSelection::Itemized { facts } = v.choice.clone() else { panic!() };
    v.choice = DeductionSelection::Automatic { expenses: ItemizedChoice::Yes { facts } };
    let r = supported(&d);
    // 16 = $1,000; 17 = $500; 24 = 10% x $1,500 = $150; 36 = $150.
    assert_eq!(line_8829(&r, "mortgage_interest"), 0);
    assert_eq!(line_8829(&r, "excess_interest"), 100_000);
    assert_eq!(line_8829(&r, "excess_taxes"), 50_000);
    assert_eq!(line_8829(&r, "allowed"), 15_000);
}

#[test]
fn form_8829_canonical_links_and_unavailable_history_are_explicit() {
    let mut d = office_return(50_000, standard_office());
    d.home_sites[0].claims[0].use_kind = OfficeQualifiedUse::NotEstablished;
    assert!(noted(&supported(&d), "WARN_HOME_USE_NOT_ESTABLISHED"));
    // 2026-10-17: the approved canonical HOME map supports multiple actual
    // homes. It replaces the one-office refusal with explicit unique sources.
    let mut d = office_return(50_000, standard_office());
    let mut duplicate = d.home_sites[0].clone();
    duplicate.id = d.next_id;
    d.next_id += 1;
    d.home_sites.push(duplicate);
    assert!(!d.valid_stored()); // the same mortgage source cannot fund two homes.
    // The original old-history boundary remains explicit, now in its actual
    // source schedule instead of an inferred acquisition-date refusal.
    let mut d = office_return(50_000, standard_office());
    let HomeTenure::Owned { building, .. } = &mut d.home_sites[0].tenure else { panic!() };
    let building = building.as_mut().unwrap();
    building.first_used = 19930512;
    building.schedule.recovery = OfficeRecovery::Macrs315;
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(
        r.issues.iter().any(|issue| issue.display.key == "HOME_DEPRECIATION_HISTORY_UNAVAILABLE")
    );
    // Home mortgage interest and home property tax are now entered once at
    // their canonical sources, so contradictory duplicate totals are not inputs.
}

#[test]
fn main_home_foreclosure_gain_boundary_and_business_property_needs_its_sale() {
    // Realized $140,000 (recourse, value $150,000 above the balance): a
    // $100,000 basis is a gain, refused by name (home-sale exclusion); at a
    // $140,000 basis the loss is zero and accepted.
    for (basis, ok) in [(14_000_000, true), (13_999_999, false)] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, acquisition(140000, 150000, true, TransferredProperty::MainHome { basis }));
        if ok {
            assert_eq!(value(&supported(&d), "acquisition_2_nondeductible_loss"), 0);
        } else {
            unsupported(&d, "Income: ");
        }
    }
    // Business property: the press sold for $7,000 on its own page matches a
    // $7,000 amount realized; a cent more names Form 4797.
    let mut d = asset_sale_return(vec![press(sold(20260601, 700_000, 0))]);
    let mut entry = acquisition(9000, 7000, true, TransferredProperty::Business);
    add(&mut d, entry.clone());
    supported(&d);
    d.income.pop();
    if let FederalIncome::PropertyAcquisition { data } = &mut entry.income {
        data.value += 1;
    }
    add(&mut d, entry);
    unsupported(&d, "Income: ");
}
