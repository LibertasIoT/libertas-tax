use super::*;
use crate::tests::foreign::foreign::{k3_answer, k3_blank, k3_partner, k3_row};

fn itemizing(status: FederalStatus) -> FederalDraft {
    let mut d = complete(status);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    d
}
fn tax(income: ForeignTaxIncome, dollars: i64) -> ForeignTaxDeducted {
    ForeignTaxDeducted {
        owners: None,
        label: "Synthetic foreign tax".into(),
        income,
        tax: dollars * 100,
    }
}
fn interest(amount: i64, tax: i64) -> FederalIncome {
    FederalIncome::Interest {
        data: interest_source(FederalInterest {
            taxable: amount * 100,
            foreign_tax: tax * 100,
            us_obligations: 0,
            early_penalty: 0,
            exempt: 0,
            private_activity: 0,
            withholding: 0,
            special: InterestSpecial::NoneOfThese,
            bond_premium: 0,
            treasury_bond_premium: 0,
            tax_exempt_bond_premium: 0,
            accrued_interest: 0,
            after_death: None,
            ..Default::default()
        }),
    }
}
fn deduction(d: &mut FederalDraft, taxes: Vec<ForeignTaxDeducted>) {
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Deduction { taxes };
}

// Schedule A specials fixture E. Derived from IRS Schedule A lines 5–7,
// Pub.514 Credit or Deduction, i6251 line2a, i8960 line9b; 2026 return.
// https://www.irs.gov/publications/p514 (2025 final),
// https://www.irs.gov/instructions/i6251 (2025 final),
// https://www.irs.gov/instructions/i8960 (2025 final).
#[test]
fn foreign_deduction_schedule_a_amt_and_niit_taxes_foot_once() {
    let mut d = itemizing(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 14_000_000, 0);
    add(&mut d, income(interest(10_000, 1_200)));
    let facts = itemized(d.deductions.as_mut().unwrap());
    facts.personal_tax = PersonalTaxElection::Income {
        paid: 800_000,
        allocation: TaxIncomeAllocation::SourceIncome { investment: 0, total: 1 },
    };
    facts.real_estate_taxes = 200_000;
    facts.personal_property_taxes = 30_000;
    deduction(&mut d, vec![tax(ForeignTaxIncome::Earned { owner: Owner::Taxpayer }, 3_000)]);
    let r = supported(&d);
    for (id, expected) in [
        ("agi", 150_000),
        ("salt_deduction", 10_300),
        ("other_taxes_deduction", 4_200),
        ("taxes_deduction", 14_500),
        ("foreign_tax_credit", 0),
        ("amt_itemized_taxes", 14_500),
        ("foreign_investment_tax_deduction", 1_200),
    ] {
        assert_eq!(value(&r, id), expected, "{id}");
    }
    assert_eq!(printed_line(&r, "other_taxes_deduction"), (TaxForm::ScheduleA, "6".into()));

    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.income[0] = wage(1, Owner::Taxpayer, 59_000_000, 0);
    itemized(d.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
        paid: 3_000_000,
        allocation: TaxIncomeAllocation::SourceIncome { investment: 0, total: 1 },
    };
    itemized(d.deductions.as_mut().unwrap()).real_estate_taxes = 0;
    itemized(d.deductions.as_mut().unwrap()).personal_property_taxes = 0;
    let r = supported(&d);
    assert_eq!(value(&r, "salt_deduction"), 11_900);
    assert_eq!(value(&r, "other_taxes_deduction"), 4_200);
    assert_eq!(value(&r, "taxes_deduction"), 16_100);
    assert_eq!(value(&r, "form_8960_income_and_property_taxes"), 1_200);
    assert_eq!(
        printed_line(&r, "form_8960_income_and_property_taxes"),
        (TaxForm::Form8960, "9b".into())
    );
}

#[test]
fn foreign_deduction_uses_each_owners_actual_net_exclusion() {
    let mut d = itemizing(FederalStatus::Joint);
    d.income.clear();
    add(&mut d, income(foreign_pay(200_000)));
    let mut spouse = income(foreign_pay(100_000));
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    set_claim(&mut d, foreign_claim(Owner::Spouse));
    deduction(
        &mut d,
        vec![
            tax(ForeignTaxIncome::Earned { owner: Owner::Taxpayer }, 20_000),
            tax(ForeignTaxIncome::Earned { owner: Owner::Spouse }, 3_000),
            tax(ForeignTaxIncome::Investment, 1_200),
        ],
    );
    let r = supported(&d);
    // Taxpayer:20,000−20,000×132,900/200,000=6,710; spouse fully excluded.
    assert_eq!(value(&r, "other_taxes_deduction"), 7_910);
    assert_eq!(value(&r, "foreign_investment_tax_deduction"), 1_200);
}

#[test]
fn foreign_deduction_business_uses_net_line_45_and_half_se() {
    let mut d = itemizing(FederalStatus::Single);
    d.income.clear();
    let FederalIncome::Business { mut data } = foreign_service_business(200_000) else {
        unreachable!()
    };
    data.source = BusinessSource::ForeignServices { totalization_exempt: false, confirmed: true };
    data.expenses.push(ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Synthetic service costs".into() },
        5_000_000,
    ));
    add(&mut d, income(FederalIncome::Business { data }));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    deduction(&mut d, vec![tax(ForeignTaxIncome::Earned { owner: Owner::Taxpayer }, 30_000)]);
    let r = supported(&d);
    // Existing Form2555 fixture: half-SE10,597; excluded-related expense40,267;
    // line45=92,633; net earnings=200,000−50,000−10,597=139,403.
    assert_eq!(value(&r, "foreign_business_taxpayer_net_exclusion"), 92_633);
    assert_eq!(value(&r, "other_taxes_deduction"), 10_065);
}

// Pub.514 published net-exclusion example: numerator115,143, denominator155,000,
// $30,000 tax; test the shared reduction directly, without treating suspended
// employee travel deductions as deductible on a 2026 return.
#[test]
fn foreign_deduction_shares_published_net_earned_tax_fraction() {
    assert_eq!(crate::form_1116::excluded_earned_tax(3_000_000, 115_143, 155_000), 2_228_574);
    assert_eq!(crate::form_1116::excluded_earned_tax(3_000_000, 200_000, 155_000), 3_000_000);
}

#[test]
fn foreign_deduction_all_k3_categories_survive_credit_only_omissions() {
    let mut d = itemizing(FederalStatus::Single);
    let mut k3 = k3_blank();
    k3.countries = vec![
        k3_row("JA", K3Category::Passive, 2_000, 0, 100),
        k3_row("CA", K3Category::General, 2_000, 0, 200),
        k3_row("GM", K3Category::ForeignBranch, 2_000, 0, 300),
    ];
    k3.items = vec![K3Item::Research];
    k3.withholding = vec![k3_answer("JA", ForeignWithholding::Below15)];
    add(&mut d, income(FederalIncome::Partnership { data: k3_partner(6_000, 0, 600, Some(k3)) }));
    deduction(&mut d, vec![tax(ForeignTaxIncome::Other, 50)]);
    let r = supported(&d);
    assert_eq!(value(&r, "other_taxes_deduction"), 650);
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert!(
        !r.review_notes.iter().any(|n| n.display.key == "WARN_K3_FOREIGN_TAX_CREDIT_NOT_CLAIMED")
    );
}

#[test]
fn foreign_deduction_k3_tax_respects_basis_and_does_not_guess_niit() {
    let mut d = itemizing(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 0);
    let mut k3 = k3_blank();
    k3.countries = vec![k3_row("JA", K3Category::Passive, 2_000, 0, 20_000)];
    k3.withholding = vec![k3_answer("JA", ForeignWithholding::AtLeast15)];
    // Partner opening basis10,000+interest2,000 allows12,000 of20,000 taxes.
    add(
        &mut d,
        income(FederalIncome::Partnership { data: k3_partner(2_000, 0, 20_000, Some(k3)) }),
    );
    deduction(&mut d, vec![]);
    let r = supported(&d);
    assert_eq!(value(&r, "other_taxes_deduction"), 12_000);
    assert_eq!(value(&r, "foreign_investment_tax_deduction"), 0);
    assert!(
        r.review_notes
            .iter()
            .any(|n| { n.display.key == "WARN_FOREIGN_K3_NIIT_TAX_NOT_ALLOCATED" })
    );
}

// IRS Form4952 (2025 final), line5: deductible portfolio expenses reduce the
// regular limit; expenses used in passive income/loss do not. AMT adds back
// foreign income taxes, so both cases allow the full$10,000 AMT interest.
// https://www.irs.gov/pub/irs-pdf/f4952.pdf page4.
#[test]
fn foreign_deduction_portfolio_and_passive_taxes_have_distinct_4952_footings() {
    let mut d = itemizing(FederalStatus::Single);
    add(&mut d, income(interest(10_000, 0)));
    investment_borrowing(&mut d, 10000, 0, 0);
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().income_use =
        InvestmentInterestUse::Ordinary;
    deduction(&mut d, vec![tax(ForeignTaxIncome::Investment, 3_000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 7_000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 10_000);
    deduction(&mut d, vec![tax(ForeignTaxIncome::Passive, 3_000)]);
    assert_eq!(value(&supported(&d), "investment_interest_deduction"), 10_000);
}

#[test]
fn foreign_deduction_rounds_aggregate_source_cents_once() {
    let mut d = itemizing(FederalStatus::Single);
    let FederalIncome::Interest { mut data } = interest(100, 0) else { unreachable!() };
    edit_interest(&mut data).foreign_tax = 50;
    add(&mut d, income(FederalIncome::Interest { data }));
    let mut row = tax(ForeignTaxIncome::Investment, 1);
    row.tax = 50;
    deduction(&mut d, vec![row]);
    assert_eq!(value(&supported(&d), "other_taxes_deduction"), 1);
}

// §901(j)(3), Pub.514 Choice Applies to All Qualified Foreign Taxes and
// Form1116 line2: sanctioned tax is deductible with the ordinary credit and
// definitely related to sanctioned income, not a common expense of Country A.
#[test]
fn foreign_deduction_sanctioned_tax_stays_out_of_other_credit_income() {
    let mut d = itemizing(FederalStatus::Single);
    add(&mut d, income(foreign_pay(100_000)));
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Ordinary {
        basis: ForeignTaxBasis::Paid,
        sources: vec![
            ForeignOrdinaryTax {
                label: "Synthetic Country A employer".into(),
                country: "ZZ".into(),
                owner: Owner::Taxpayer,
                category: ForeignTaxCategory::Employment,
                income: 9_000_000,
                tax: 1_800_000,
            },
            ForeignOrdinaryTax {
                label: "Synthetic Iran employer".into(),
                country: "IR".into(),
                owner: Owner::Taxpayer,
                category: ForeignTaxCategory::Employment,
                income: 1_000_000,
                tax: 50_000,
            },
        ],
        situations: vec![],
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::Unreviewed,
        amt_limitation: ForeignAmtLimitation::Normal,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        rate_adjustment: ForeignRateAdjustment::Exception,
    };
    let r = supported(&d);
    assert_eq!(value(&r, "other_taxes_deduction"), 500);
    assert_eq!(value(&r, "foreign_general_ZZ_apportioned_deductions"), 0);
    assert_eq!(value(&r, "foreign_sanctioned_IR_related_expenses"), 500);
    assert_eq!(value(&r, "foreign_amt_sanctioned_IR_related_expenses"), 0);
    assert!(!noted(&r, "WARN_FTC_SANCTIONED_COUNTRY_TAX"));
    // Pub.514 Credit for Taxes Paid or Accrued: an accrued FTC election
    // does not establish cash payment for this separate deduction. Entity
    // K-3 taxes are separately stated under 1.702-1(a)(6)/1.1366-1(a)(2)(iv).
    let mut accrued = d.clone();
    if let ForeignTaxClaim::Ordinary { basis, .. } =
        &mut accrued.credits.as_mut().unwrap().foreign_tax
    {
        *basis = ForeignTaxBasis::Accrued;
    }
    let r = supported(&accrued);
    assert_eq!(value(&r, "other_taxes_deduction"), 0);
    assert_eq!(value(&r, "foreign_sanctioned_IR_related_expenses"), 0);
    assert!(noted(&r, "WARN_FOREIGN_ACCRUED_SANCTIONED_TAX_NOT_DEDUCTED"));
    let mut entity = accrued.clone();
    let mut k3 = k3_blank();
    k3.countries = vec![k3_row("IR", K3Category::General, 1_000, 0, 200)];
    add(
        &mut entity,
        income(FederalIncome::Partnership { data: k3_partner(1_000, 0, 200, Some(k3)) }),
    );
    assert_eq!(value(&supported(&entity), "other_taxes_deduction"), 200);

    // An omitted potential portfolio expense must not inflate Form4952.
    add(&mut accrued, income(interest(10_000, 0)));
    if let ForeignTaxClaim::Ordinary { sources, .. } =
        &mut accrued.credits.as_mut().unwrap().foreign_tax
    {
        sources[1].category = ForeignTaxCategory::Passive {
            withholding: ForeignWithholding::Below15,
            qualified: 0,
            sales: vec![],
        };
    }
    investment_borrowing(&mut accrued, 10000, 0, 0);
    accrued.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().income_use =
        InvestmentInterestUse::Ordinary;
    assert_eq!(estimate(&accrued).state, ResultState::Unsupported);

    d.deductions.as_mut().unwrap().itemize_override = None;
    let r = supported(&d);
    assert_eq!(value(&r, "other_taxes_deduction"), 0);
    assert!(noted(&r, "WARN_FTC_SANCTIONED_COUNTRY_TAX"));
}

#[test]
fn foreign_deduction_missing_released_k1_tax_cannot_overstate_4952() {
    let mut d = itemizing(FederalStatus::Single);
    let mut partner = k3_partner(1_000, 0, 0, Some(k3_blank()));
    partner.regular.carried.foreign_taxes = 500;
    partner.alternative = Some(partner.regular.clone());
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    deduction(&mut d, vec![]);
    assert!(noted(&supported(&d), "WARN_FOREIGN_K1_TAX_SOURCE_MISSING"));
    investment_borrowing(&mut d, 1000, 0, 0);
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().income_use =
        InvestmentInterestUse::Ordinary;
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|issue| issue.message.contains("Income-tax expense attribution")));
}

// Current source-specific borrowing owner, with independent regular/AMT
// foreign-tax expense treatment: §163(d), §56(b)(1)(A)(ii), Form4952 lines5–8.
fn royalty_foreign_tax_return(gross: i64, portfolio: i64, royalty_paid: i64) -> FederalDraft {
    let mut d = itemizing(FederalStatus::Single);
    add(&mut d, income(royalty(gross, 1_000)));
    let id = d.income.last().unwrap().id;
    let mut loans = vec![InvestmentLoan {
        source: id,
        ..native_investment_loan("Royalty loan".into(), royalty_paid * 100, 0, 0)
    }];
    if portfolio > 0 {
        loans.push(native_investment_loan("Portfolio loan".into(), portfolio * 100, 0, 0));
    }
    d.deductions.as_mut().unwrap().investment_interest = Some(InvestmentBorrowing {
        loans,
        election: InvestmentElection::None,
        income_use: InvestmentInterestUse::Ordinary,
        limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
    });
    deduction(&mut d, vec![tax(ForeignTaxIncome::Investment, 1_000)]);
    d
}

#[test]
fn foreign_deduction_royalty_route_keeps_actual_regular_and_amt_expenses() {
    // Gross4000−ordinary costs1000−foreign portfolio tax1000=>regular
    // capacity2000; AMT adds the tax back=>capacity3000. Exclusive royalty
    // borrowing4000 gives regularcarry2000/AMTcarry1000 with no guessed split.
    let mut d = royalty_foreign_tax_return(4_000, 0, 4_000);
    let r = supported(&d);
    for (id, expected) in [
        ("agi", 61_000),
        ("investment_form_line5", 2_000),
        ("investment_form_line8", 2_000),
        ("investment_form_line7", 2_000),
        ("amt_investment_form_line5", 1_000),
        ("amt_investment_form_line8", 3_000),
        ("amt_investment_form_line7", 1_000),
        ("investment_interest_deduction", 0),
        ("schedule_e_rental_royalty_total", 1_000),
        ("other_taxes_deduction", 1_000),
        ("amt_investment_interest_adjustment", -1_000),
    ] {
        assert_eq!(value(&r, id), expected, "{id}");
    }
    assert!(!noted(&r, "NOTE_INVESTMENT_DESTINATION_UNAVAILABLE"));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.deductions.as_mut().unwrap().itemize_override = None;
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "investment_form_line5"), 1_000);
    assert_eq!(value(&r, "investment_form_line8"), 3_000);
    assert_eq!(value(&r, "investment_form_line7"), 1_000);
    assert_eq!(value(&r, "other_taxes_deduction"), 0);
    assert!(noted(&r, "WARN_FOREIGN_TAX_DEDUCTION_UNUSED"));
}

#[test]
fn foreign_deduction_preserves_mixed_royalty_omission_and_safe_automatic_endpoint() {
    // Capacity2000 cannot choose between2000portfolio/2000royalty borrowing.
    // Keep ordinary royalty income and foreign tax, omit only the unsupported
    // interest benefit/carry. No priority or whole-return retry is invented.
    let d = royalty_foreign_tax_return(4_000, 2_000, 2_000);
    let r = supported(&d);
    assert!(noted(&r, "NOTE_INVESTMENT_DESTINATION_UNAVAILABLE"));
    assert_eq!(value(&r, "agi"), 63_000);
    assert_eq!(value(&r, "other_taxes_deduction"), 1_000);
    assert!(!r.lines.iter().any(|row| row.id == "investment_interest_carryforward"));
    // With10000capacity, both foreign-expense endpoints fully allow4000.
    // Automatic can safely retain the same source routing before its choice.
    let mut d = royalty_foreign_tax_return(11_000, 2_000, 2_000);
    let facts = itemized(d.deductions.as_mut().unwrap()).clone();
    {
        d.deductions.as_mut().unwrap().itemized = facts;
        d.deductions.as_mut().unwrap().itemize_override = None;
    };
    let r = supported(&d);
    assert!(!noted(&r, "NOTE_INVESTMENT_DESTINATION_UNAVAILABLE"));
    assert_eq!(value(&r, "agi"), 68_000);
    assert_eq!(value(&r, "investment_form_line8"), 4_000);
    assert_eq!(value(&r, "other_taxes_deduction"), 0);
}

// The choice is a deduction, not a Form1116 credit requiring expense
// allocation: preserve existing §642(h), §691(c), AMT and gift worksheets.
#[test]
fn foreign_deduction_keeps_credit_neutral_estate_and_charity_combinations() {
    let mut d = estate_trust_code_c_draft(0, 0);
    d.deductions.as_mut().unwrap().itemize_override = Some(ItemizeOverride::Itemize);
    let mut estate = estate_trust_code_d_data(&mut d);
    estate.interest = 100_000;
    estate.box_10_interest_ird_estate_tax = 10_000;
    estate.box_10_interest_ird_only = true;
    estate.final_k1 = true;
    estate.final_year_successor = Some(true);
    estate.final_67e_deduction = 20_000;
    estate.final_67e_no_niit_allocation = true;
    estate.final_state_income_tax = 30_000;
    estate.final_taxes_only = true;
    estate.amt_box12a = 40_000;
    estate.amt_box12a_standalone = true;
    drop(estate);
    let before = supported(&d);
    deduction(&mut d, vec![tax(ForeignTaxIncome::Investment, 50)]);
    let after = supported(&d);
    for id in ["agi", "estate_trust_67e_deduction", "salt_deduction"] {
        assert_eq!(value(&after, id), value(&before, id), "{id}");
    }
    assert_eq!(value(&after, "other_taxes_deduction"), 50);
    assert_eq!(value(&after, "foreign_tax_credit"), 0);
    assert_eq!(supported(&restart(d).draft), after);

    let mut gifts = cash_carry_with_current_property(10_000_000, 100_000, 2_000);
    let before = supported(&gifts);
    deduction(&mut gifts, vec![tax(ForeignTaxIncome::Investment, 50)]);
    let after = supported(&gifts);
    assert_eq!(value(&after, "charity_deduction"), value(&before, "charity_deduction"));
    assert_eq!(value(&after, "other_taxes_deduction"), 50);
    assert_eq!(supported(&restart(gifts).draft), after);
}

#[test]
fn foreign_deduction_complete_subdollar_k3_is_not_a_missing_source() {
    let mut d = itemizing(FederalStatus::Single);
    let mut k3 = k3_blank();
    let mut row = k3_row("JA", K3Category::Passive, 1_000, 0, 1);
    row.tax = 99;
    k3.countries = vec![row];
    k3.withholding = vec![k3_answer("JA", ForeignWithholding::Below15)];
    let mut partner = k3_partner(1_000, 0, 1, Some(k3));
    partner.separate.as_mut().unwrap().foreign_taxes = 99;
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    deduction(&mut d, vec![]);
    let result = supported(&d);
    assert_eq!(value(&result, "other_taxes_deduction"), 1);
    assert!(!noted(&result, "WARN_FOREIGN_K1_TAX_SOURCE_MISSING"));
}
