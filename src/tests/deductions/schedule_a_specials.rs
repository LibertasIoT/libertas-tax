use super::*;

// Fixtures A–H from the Schedule A specials map; foreign-tax fixture E is
// covered in foreign_deduction.rs. Amount inputs are cents, expectations
// below are the whole-dollar amounts printed on the return.
fn loan(home: MortgageHome, incurred: u32, average: i64, interest: i64) -> MortgageLoan {
    MortgageLoan {
        id: 1,
        property: "Synthetic home".into(),
        lender: MortgageLender::Institution,
        interest: interest * 100,
        outstanding_principal: average * 100,
        incurred,
        refund_of_interest: MortgageInterestRefund::No,
        mortgage_insurance_premiums: 0,
        points: MortgagePoints::No,
        details: MortgageLoanDetails::FirstLender(MortgageLedger {
            home,
            purpose: MortgageUse::Home,
            average: MortgageAverageBalance::Statements { average_balance: average * 100 },
        }),
    }
}

fn itemizing(status: FederalStatus) -> FederalDraft {
    let mut d = complete(status);
    let deductions = d.deductions.as_mut().unwrap();
    deductions.choice = DeductionSelection::Itemized { facts: itemized(deductions).clone() };
    d
}

fn assert_a(r: &FederalResult, id: &str, number: &str, dollars: i64) {
    assert_eq!(value(r, id), dollars, "{id}");
    assert_eq!(printed_line(r, id), (TaxForm::ScheduleA, number.into()));
}

fn assert_foots(r: &FederalResult) {
    let v = |id| value(r, id);
    assert_eq!(
        v("salt_before_limit"),
        v("salt_income_or_sales_tax") + v("salt_real_estate_tax") + v("salt_personal_property_tax")
    );
    assert_eq!(v("taxes_deduction"), v("salt_deduction") + v("other_taxes_deduction"));
    assert_eq!(
        v("mortgage_interest_8e"),
        v("mortgage_interest_8a")
            + v("mortgage_interest_8b")
            + v("mortgage_points_8c")
            + v("mortgage_insurance_premiums_8d")
    );
    assert_eq!(
        v("interest_deduction"),
        v("mortgage_interest_8e") + v("investment_interest_deduction")
    );
    assert_eq!(
        v("charity_deduction"),
        v("charity_current_allowed") + v("charity_carryover_allowed")
    );
    let other: i64 = r
        .lines
        .iter()
        .filter(|row| {
            row.form_line.as_ref().is_some_and(|f| {
                f.form == TaxForm::ScheduleA
                    && matches!(
                        f.line.as_str(),
                        "17a"
                            | "17b"
                            | "17c"
                            | "17d"
                            | "17e"
                            | "17f"
                            | "17g"
                            | "17h"
                            | "17i"
                            | "17j"
                            | "17k"
                    )
            })
        })
        .map(|row| row.amount / 100)
        .sum();
    assert_eq!(v("other_itemized_total"), other);
    assert_eq!(
        v("itemized_deductions_allowed"),
        v("medical_deduction")
            + v("taxes_deduction")
            + v("interest_deduction")
            + v("charity_deduction")
            + v("personal_casualty_deduction")
            + other
            - v("itemized_deduction_reduction")
    );
}

#[test]
fn a_mixed_use_heloc_repayments_retire_other_use_before_acquisition() {
    let mut d = itemizing(FederalStatus::Single);
    let first = loan(MortgageHome::Main, 20190501, 700_000, 35_000);
    let mut second = loan(MortgageHome::Main, 20260302, 200_000, 10_800);
    mortgage_ledger_mut(&mut second).purpose = MortgageUse::Partly(MortgageSplit {
        improvement: 18_000_000,
        improvement_incurred: 20260302,
        other: 2_000_000,
    });
    mortgage_ledger_mut(&mut second).average =
        MortgageAverageBalance::FirstAndLast { last_balance: 19_000_000 };
    d.deductions.as_mut().unwrap().mortgage = identified_mortgages(vec![first, second]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_post_2017_balance"), 850_000);
    assert_eq!(value(&r, "mortgage_qualified_loan_limit"), 750_000);
    assert_eq!(value(&r, "mortgage_average_balance"), 850_000);
    assert_eq!(value(&r, "mortgage_qualified_interest"), 44_994);
    assert_eq!(value(&r, "mortgage_other_use_interest"), 805);
    assert_eq!(value(&r, "mortgage_nondeductible_interest"), 5_310);
    assert_a(&r, "mortgage_interest_8a", "8a", 39_685);
    assert_foots(&r);
}

#[test]
fn b_cash_out_refinance_splits_old_debt_improvements_and_points() {
    let mut d = itemizing(FederalStatus::Single);
    let old = loan(MortgageHome::Main, 20120601, 77_250, 4_000);
    let mut new = loan(MortgageHome::Main, 20260415, 400_000, 15_600);
    mortgage_ledger_mut(&mut new).purpose = MortgageUse::Refinance(MortgageRefinance {
        refinanced: 30_700_000,
        earlier_incurred: 20120601,
        earlier_term_ends: Some(20420601),
        improvement: 6_000_000,
        other: 3_300_000,
    });
    mortgage_ledger_mut(&mut new).average =
        MortgageAverageBalance::FirstAndLast { last_balance: 39_500_000 };
    new.points = MortgagePoints::Improvement { amount: 800_000, term_months: 360 };
    d.deductions.as_mut().unwrap().mortgage = identified_mortgages(vec![old, new]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_pre_2018_balance"), 307_500);
    assert_eq!(value(&r, "mortgage_post_2017_balance"), 45_000);
    assert_eq!(value(&r, "mortgage_average_balance"), 352_500);
    assert_eq!(value(&r, "mortgage_other_use_interest"), 1_186);
    assert_a(&r, "mortgage_interest_8a", "8a", 18_413);
    assert_a(&r, "mortgage_points_8c", "8c", 1_354);
    assert_a(&r, "mortgage_interest_8e", "8e", 19_767);
    assert_foots(&r);
}

#[test]
fn c_third_home_is_excluded_and_houseboat_interest_is_an_amt_adjustment() {
    let mut d = itemizing(FederalStatus::Joint);
    d.deductions.as_mut().unwrap().mortgage = identified_mortgages(vec![
        loan(MortgageHome::Main, 20200601, 500_000, 20_000),
        loan(MortgageHome::Second, 20210601, 200_000, 9_000),
        loan(MortgageHome::NotQualified, 20220601, 150_000, 6_000),
    ]);
    let r = supported(&d);
    assert_a(&r, "mortgage_interest_8a", "8a", 29_000);
    assert_eq!(value(&r, "mortgage_not_qualified_home_interest"), 6_000);
    assert_eq!(value(&r, "amt_mortgage_dwelling_interest"), 0);
    assert_foots(&r);
    mortgage_ledger_mut(&mut d.deductions.as_mut().unwrap().mortgage[1]).home =
        MortgageHome::SecondVehicle;
    let r = supported(&d);
    assert_a(&r, "mortgage_interest_8a", "8a", 29_000);
    assert_eq!(value(&r, "amt_mortgage_dwelling_interest"), 9_000);
    assert_eq!(printed_line(&r, "amt_related_adjustments"), (TaxForm::Form6251, "3".into()));
    assert_foots(&r);
}

#[test]
fn d_box_4_recovery_uses_prior_tax_benefit_without_reducing_current_interest() {
    let mut d = itemizing(FederalStatus::Single);
    let mut mortgage = loan(MortgageHome::Main, 20200601, 100_000, 9_000);
    mortgage.refund_of_interest =
        MortgageInterestRefund::Yes { amount: 40_000, origin: MortgageRefundOrigin::NotDeducted };
    d.deductions.as_mut().unwrap().mortgage = identified_mortgages(vec![mortgage]);
    let no_benefit = supported(&d);
    assert_a(&no_benefit, "mortgage_interest_8a", "8a", 9_000);
    assert_eq!(value(&no_benefit, "agi"), 60_000);
    d.deductions.as_mut().unwrap().mortgage[0].refund_of_interest = MortgageInterestRefund::Yes {
        amount: 40_000,
        origin: MortgageRefundOrigin::Deducted(PriorYearRecovery {
            year: 2025,
            deducted: 1_200_000,
            itemized: 1_600_000,
            standard: 1_575_000,
            taxable_income: 5_000_000,
            ordinary: true,
        }),
    };
    let partial = supported(&d);
    assert_a(&partial, "mortgage_interest_8a", "8a", 9_000);
    assert_eq!(value(&partial, "recovery_2025_taxable"), 250);
    assert_eq!(value(&partial, "agi"), 60_250);
    assert_foots(&partial);
}

#[test]
fn f_teacher_and_coach_expenses_split_above_line_and_itemized_deductions() {
    let mut d = itemizing(FederalStatus::Joint);
    d.adjustments.as_mut().unwrap().educators = EducatorClaim {
        taxpayer: Some(EducatorRole::Educator { classroom: 50_000, instructional: 30_000 }),
        spouse: Some(EducatorRole::Coach { expenses: 90_000 }),
    };
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 350);
    assert_a(&r, "educator_itemized_deduction", "17k", 1_350);
    assert_foots(&r);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    let standard = supported(&d);
    assert_eq!(value(&standard, "adjustments"), 350);
    assert_a(&standard, "educator_itemized_deduction", "17k", 0);
    let mut early = itemizing(FederalStatus::Single);
    early.adjustments.as_mut().unwrap().educators.taxpayer =
        Some(EducatorRole::Educator { classroom: 40_000, instructional: 0 });
    let r = supported(&early);
    assert_eq!(value(&r, "adjustments"), 350);
    assert_a(&r, "educator_itemized_deduction", "17k", 50);
    assert_foots(&r);
}

#[test]
fn g_refinanced_old_debt_changes_category_at_original_term_end() {
    let mut d = itemizing(FederalStatus::Joint);
    let mut mortgage = loan(MortgageHome::Main, 20210310, 860_000, 34_000);
    mortgage_ledger_mut(&mut mortgage).purpose = MortgageUse::Refinance(MortgageRefinance {
        refinanced: 90_000_000,
        earlier_incurred: 19960501,
        earlier_term_ends: Some(20260501),
        improvement: 0,
        other: 0,
    });
    mortgage_ledger_mut(&mut mortgage).average =
        MortgageAverageBalance::FirstAndLast { last_balance: 83_600_000 };
    d.deductions.as_mut().unwrap().mortgage = identified_mortgages(vec![mortgage]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_pre_2018_balance"), 285_000);
    assert_eq!(value(&r, "mortgage_post_2017_balance"), 562_000);
    assert_eq!(value(&r, "mortgage_average_balance"), 847_000);
    assert_a(&r, "mortgage_interest_8a", "8a", 30_090);
    assert_foots(&r);
}

#[test]
fn h_other_itemized_lines_and_gst_have_distinct_amt_treatment() {
    let mut d = itemizing(FederalStatus::Joint);
    let facts = itemized(d.deductions.as_mut().unwrap());
    facts.real_estate_taxes = 800_000;
    facts.personal_property_taxes = 200_000;
    facts.other_lines = vec![
        OtherItemizedLine::GenerationSkippingTax { amount: 70_000 },
        OtherItemizedLine::BondPremium {
            amount: 25_000,
            source: BondPremiumSource::OtherDocumented {
                instrument: "Different documented historical bond".into(),
            },
        },
        OtherItemizedLine::DebtInstrumentLoss { amount: 12_000 },
        OtherItemizedLine::ImpairmentWork { owner: Owner::Spouse, amount: 360_000 },
    ];
    let r = supported(&d);
    assert_a(&r, "salt_real_estate_tax", "5b", 8_000);
    assert_a(&r, "salt_personal_property_tax", "5c", 2_000);
    assert_a(&r, "other_taxes_deduction", "6", 700);
    assert_a(&r, "taxes_deduction", "7", 10_700);
    assert_a(&r, "bond_premium_deduction", "17f", 250);
    assert_a(&r, "debt_instrument_loss_deduction", "17g", 120);
    assert_a(&r, "impairment_work_deduction", "17j", 3_600);
    assert_a(&r, "other_itemized_total", "17z", 3_970);
    assert_eq!(value(&r, "amt_itemized_taxes"), 10_000);
    assert_eq!(printed_line(&r, "amt_itemized_taxes"), (TaxForm::Form6251, "2a".into()));
    assert_foots(&r);
}

#[test]
fn monthly_balances_retire_other_use_first_and_payoff_deducts_remaining_points() {
    let mut d = itemizing(FederalStatus::Single);
    let mut mixed = loan(MortgageHome::Main, 20260101, 100_000, 1_600);
    mortgage_ledger_mut(&mut mixed).purpose = MortgageUse::Partly(MortgageSplit {
        improvement: 8_000_000,
        improvement_incurred: 20260101,
        other: 2_000_000,
    });
    mortgage_ledger_mut(&mut mixed).average = MortgageAverageBalance::Monthly {
        balances: vec![
            MortgageMonthBalance { month: 1, balance: 9_000_000 },
            MortgageMonthBalance { month: 2, balance: 7_000_000 },
        ],
    };
    let mut paid_off = loan(MortgageHome::Main, 20240101, 10_000, 1_000);
    paid_off.points = MortgagePoints::Amortized { amount: 360_000, term_months: 360, ended: true };
    d.deductions.as_mut().unwrap().mortgage = identified_mortgages(vec![mixed, paid_off]);
    let r = supported(&d);
    // Qualified month-end balances: 80,000 + 70,000, divided by 12;
    // interest: 1,600 * 150,000 / 160,000 = 1,500. The other loan
    // contributes 10,000 of annual average balance and 1,000 interest.
    assert_eq!(value(&r, "mortgage_post_2017_balance"), 22_500);
    assert_eq!(value(&r, "mortgage_other_use_interest"), 100);
    assert_a(&r, "mortgage_interest_8a", "8a", 2_500);
    // 2024–2025 deducted 24/360 * 3,600 = 240; payoff releases 3,360.
    assert_a(&r, "mortgage_points_8c", "8c", 3_360);
    assert_a(&r, "mortgage_interest_8e", "8e", 5_860);
    assert_foots(&r);
}

#[test]
fn box_4_recoveries_share_one_year_limit_and_reject_conflicting_year_facts() {
    let mut d = itemizing(FederalStatus::Single);
    let history = PriorYearRecovery {
        year: 2025,
        deducted: 1_200_000,
        itemized: 1_600_000,
        standard: 1_575_000,
        taxable_income: 5_000_000,
        ordinary: true,
    };
    let mut mortgage = loan(MortgageHome::Main, 20200601, 100_000, 9_000);
    mortgage.refund_of_interest = MortgageInterestRefund::Yes {
        amount: 40_000,
        origin: MortgageRefundOrigin::Deducted(history.clone()),
    };
    d.deductions.as_mut().unwrap().mortgage =
        identified_mortgages(vec![mortgage.clone(), mortgage]);
    add(
        &mut d,
        income(FederalIncome::OtherIncome {
            data: OtherIncomeItem {
                kind: OtherIncomeKind::Recovery { worksheet: history },
                amount: 30_000,
                withholding: 0,
            },
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "recovery_2025_taxable"), 250);
    assert_eq!(value(&r, "agi"), 60_250);
    let loans = &mut d.deductions.as_mut().unwrap().mortgage;
    let MortgageInterestRefund::Yes { origin: MortgageRefundOrigin::Deducted(history), .. } =
        &mut loans[1].refund_of_interest
    else {
        panic!()
    };
    history.taxable_income += 100;
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|issue| issue.display.key
        == "ISSUE_RECOVERY_REFUND_YEAR_FACTS_DIFFER"
        && matches!(
            issue.context.source(),
            IssueContext::Record { section: TaxSection::Mortgage, .. }
                | IssueContext::Section(TaxSection::Mortgage)
        )));
}
