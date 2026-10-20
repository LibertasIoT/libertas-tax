use super::*;

#[test]
fn published_ordinary_bracket_edges_all_statuses() {
    let fixtures = [
        (
            FederalStatus::Single,
            vec![
                (12400, 1243),
                (50400, 5806),
                (105700, 17966),
                (201775, 41024),
                (256225, 58448),
                (640600, 192979),
            ],
        ),
        (
            FederalStatus::Joint,
            vec![
                (24800, 2483),
                (100800, 11600),
                (211400, 35932),
                (403550, 82048),
                (512450, 116896),
                (768700, 206584),
            ],
        ),
        (
            FederalStatus::Head,
            vec![
                (17700, 1773),
                (67450, 7746),
                (105700, 16155),
                (201750, 39207),
                (256200, 56631),
                (640600, 191171),
            ],
        ),
        (
            FederalStatus::Separate,
            vec![
                (12400, 1243),
                (50400, 5806),
                (105700, 17966),
                (201775, 41024),
                (256225, 58448),
                (384350, 103292),
            ],
        ),
    ];
    for (status, rows) in fixtures {
        for (income, tax) in rows {
            assert_eq!(preferential_tax(income, 0, status), tax, "{status:?} {income}");
            assert!(preferential_tax(income + 1, 0, status) >= tax);
        }
    }
    assert_eq!(preferential_tax(67800, 0, FederalStatus::Surviving), 7643);
}
#[test]
fn irs_social_security_worksheet_examples() {
    assert_eq!(social_taxable(5980, 18600 + 9400 + 990, 0, FederalStatus::Single, false), 2990);
    assert_eq!(social_taxable(5600, 15500 + 14000 + 250 - 1000, 0, FederalStatus::Joint, false), 0);
    assert_eq!(social_taxable(20000, 100000, 0, FederalStatus::Single, false), 17000);
    assert_eq!(social_taxable(20000, 0, 0, FederalStatus::Separate, false), 8500);
    assert_eq!(social_taxable(20000, -3000, 0, FederalStatus::Separate, false), 5950);
}
#[test]
fn gambling_withholding_and_unresolved_records_are_handled() {
    let mut d = complete(FederalStatus::Single);
    let mut entry = gambling(1000, 1000);
    if let FederalIncome::Gambling { data } = &mut entry.income {
        data.withholding = 24000;
    }
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(return_line(&r, "25d"), 674000);
    assert_eq!(value(&r, "gambling_loss_deduction"), 0); // Standard deduction is larger.
    if let FederalIncome::Gambling { data } = &mut d.income[1].income {
        data.ordinary = false;
    }
    let r = supported(&d);
    assert!(noted(&r, "WARN_GAMBLING_LEFT_OUT"));
    assert_eq!(return_line(&r, "25d"), 650000);
}

// T2-44f fixture: an itemizing single filer with two W-2Gs whose combined
// documented losses exceed combined winnings, plus box 4 withholding on
// both forms. By hand (2026 draft f1040, i1040sa, Tax Topic 419):
//   Wages (the fixture's default wage entry): $60,000, box 2 withholding
//   $6,500 (line 25a).
//   W-2G #1: box 1 $5,000, box 4 $500, and the $10,000 of documented
//   losses; W-2G #2: box 1 $3,500, box 4 $300, no separate losses entered.
//   Winnings sum to Schedule 1 line 8b and Form 1040 line 8: $5,000 +
//   $3,500 = $8,500. AGI = $60,000 + $8,500 = $68,500 (no adjustments).
//   2026 rule: the loss deduction is the smaller of winnings and 90% of
//   documented losses = min($8,500, 90% x $10,000 = $9,000) = $8,500 --
//   here the winnings ceiling governs, not the 90% factor, because losses
//   exceed winnings by more than the 10% haircut would matter.
//   Schedule A: SALT (property tax only, no income/sales tax election)
//   $8,000, comfortably under the 2026 $40,400 cap at this income, plus
//   the $8,500 gambling loss = $16,500, above the single standard
//   deduction $16,100, so the filer itemizes.
//   Taxable income = $68,500 - $16,500 = $52,000. 2026 Tax Table (below
//   $100,000, midpoint $52,025): 10% x $12,400 = $1,240.00; 12% x
//   ($50,400 - $12,400 = $38,000) = $4,560.00; 22% x ($52,025 - $50,400 =
//   $1,625) = $357.50; total $6,157.50, rounds to $6,158. No credits or
//   other taxes apply at this income.
//   Withholding: line 25a $6,500 (wages) + line 25c $500 + $300 = $800
//   (the two W-2Gs) = line 25d $7,300. Overpayment (line 34) = $7,300 -
//   $6,158 = $1,142; nothing is owed (line 37 is $0).
#[test]
fn itemizing_filer_with_two_w2gs_losses_above_winnings_and_withholding() {
    let mut d = complete(FederalStatus::Single);
    let mut entry1 = gambling(5000, 10000);
    if let FederalIncome::Gambling { data } = &mut entry1.income {
        data.withholding = 50_000;
    }
    add(&mut d, entry1);
    let mut entry2 = gambling(3500, 0);
    if let FederalIncome::Gambling { data } = &mut entry2.income {
        data.withholding = 30_000;
    }
    add(&mut d, entry2);
    let facts = itemized(d.deductions.as_mut().unwrap());
    facts.personal_property_taxes = 800_000;
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 68_500);
    assert_eq!(value(&r, "gambling_winnings"), 8_500);
    assert_eq!(value(&r, "gambling_loss_deduction"), 8_500);
    assert_eq!(value(&r, "taxable_income"), 52_000);
    assert_eq!(value(&r, "income_tax"), 6_158);
    assert_eq!(return_line(&r, "25a"), 650_000);
    assert_eq!(return_line(&r, "25c"), 80_000);
    assert_eq!(return_line(&r, "25d"), 730_000);
    assert_eq!(return_line(&r, "34"), 114_200);
    assert_eq!(return_line(&r, "37"), 0);
}

// T2-44f boundary: the same two W-2Gs under the standard deduction. The
// 2026 gambling-loss deduction is available only when itemizing (Schedule
// A line 16), so a filer who takes the standard deduction gets $0 of it
// even though documented losses exceed winnings.
//   AGI is unchanged at $68,500 (winnings enter gross income regardless of
//   the deduction choice). Standard deduction (single, 2026) $16,100.
//   Taxable income = $68,500 - $16,100 = $52,400. 2026 Tax Table midpoint
//   $52,425: 10% x $12,400 = $1,240.00; 12% x $38,000 = $4,560.00; 22% x
//   ($52,425 - $50,400 = $2,025) = $445.50; total $6,245.50, rounds to
//   $6,246. Withholding is unaffected by the deduction choice, so line 25d
//   stays $7,300 and the overpayment is $7,300 - $6,246 = $1,054.
#[test]
fn two_w2gs_under_the_standard_deduction_get_no_loss_deduction() {
    let mut d = complete(FederalStatus::Single);
    let mut entry1 = gambling(5000, 10000);
    if let FederalIncome::Gambling { data } = &mut entry1.income {
        data.withholding = 50_000;
    }
    add(&mut d, entry1);
    let mut entry2 = gambling(3500, 0);
    if let FederalIncome::Gambling { data } = &mut entry2.income {
        data.withholding = 30_000;
    }
    add(&mut d, entry2);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 68_500);
    assert_eq!(value(&r, "gambling_loss_deduction"), 0);
    assert_eq!(value(&r, "taxable_income"), 52_400);
    assert_eq!(value(&r, "income_tax"), 6_246);
    assert_eq!(return_line(&r, "25d"), 730_000);
    assert_eq!(return_line(&r, "34"), 105_400);
}

// T2-44f boundary: the 2026 rule limits the loss deduction to 90% of
// documented losses even well below the winnings ceiling. Winnings
// $40,000, losses $20,000: 90% x $20,000 = $18,000 < $40,000, so the 90%
// factor -- not the winnings cap -- governs the deduction.
//   AGI = $60,000 wages + $40,000 winnings = $100,000. The $18,000 loss
//   deduction alone exceeds the $16,100 single standard deduction, so the
//   filer itemizes; taxable income = $100,000 - $18,000 = $82,000.
#[test]
fn ninety_percent_of_losses_governs_when_losses_are_well_under_winnings() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, gambling(40_000, 20_000));
    let r = supported(&d);
    assert_eq!(value(&r, "gambling_loss_deduction"), 18_000);
    assert_eq!(value(&r, "agi"), 100_000);
    assert_eq!(value(&r, "taxable_income"), 82_000);
}

// T2-44f boundary: the deduction cannot exceed winnings even when 90% of
// documented losses is far larger. Winnings $20,000, losses $100,000: 90%
// x $100,000 = $90,000, but the deduction cannot exceed the $20,000 of
// winnings it offsets.
//   AGI = $60,000 + $20,000 = $80,000; the $20,000 itemized total exceeds
//   the standard deduction, so taxable income = $80,000 - $20,000 =
//   $60,000.
#[test]
fn deduction_cannot_exceed_winnings_even_at_ninety_percent_of_larger_losses() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, gambling(20_000, 100_000));
    let r = supported(&d);
    assert_eq!(value(&r, "gambling_loss_deduction"), 20_000);
    assert_eq!(value(&r, "agi"), 80_000);
    assert_eq!(value(&r, "taxable_income"), 60_000);
}

// T2-44f boundary: losses exactly equal to winnings still take the 90% cut
// rather than washing out dollar for dollar. Winnings $20,000, losses
// $20,000: 90% x $20,000 = $18,000 < $20,000, so the deduction is $18,000,
// not the full $20,000 of losses (or winnings).
//   AGI = $60,000 + $20,000 = $80,000; the $18,000 itemized total exceeds
//   the standard deduction, so taxable income = $80,000 - $18,000 =
//   $62,000.
#[test]
fn losses_exactly_equal_to_winnings_still_lose_ten_percent() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, gambling(20_000, 20_000));
    let r = supported(&d);
    assert_eq!(value(&r, "gambling_loss_deduction"), 18_000);
    assert_eq!(value(&r, "agi"), 80_000);
    assert_eq!(value(&r, "taxable_income"), 62_000);
}

// T2-44f boundary: Form W-2G box 4 withholding routes to Form 1040 line
// 25c ("other-form" withholding), separately from wage withholding on
// line 25a.
//   Line 25a stays the fixture's default $6,500 wage withholding; line 25c
//   is the $500 box 4 amount alone; line 25d = $6,500 + $500 = $7,000.
#[test]
fn box_4_withholding_reaches_line_25c_separately_from_wages() {
    let mut d = complete(FederalStatus::Single);
    let mut entry = gambling(2000, 0);
    if let FederalIncome::Gambling { data } = &mut entry.income {
        data.withholding = 50_000;
    }
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(return_line(&r, "25a"), 650_000);
    assert_eq!(return_line(&r, "25c"), 50_000);
    assert_eq!(return_line(&r, "25d"), 700_000);
}

// T2-44f boundary: several W-2Gs add up before any loss limit applies.
// W-2G #1: box 1 $5,000, box 4 $400. W-2G #2: box 1 $7,000, box 4 $600.
//   Winnings sum to Schedule 1 line 8b and Form 1040 line 8: $5,000 +
//   $7,000 = $12,000. Withholding sums to line 25c: $400 + $600 = $1,000.
//   AGI = $60,000 wages + $12,000 winnings = $72,000.
#[test]
fn several_w2gs_add_up_before_the_loss_limit() {
    let mut d = complete(FederalStatus::Single);
    let mut entry1 = gambling(5000, 0);
    if let FederalIncome::Gambling { data } = &mut entry1.income {
        data.withholding = 40_000;
    }
    add(&mut d, entry1);
    let mut entry2 = gambling(7000, 0);
    if let FederalIncome::Gambling { data } = &mut entry2.income {
        data.withholding = 60_000;
    }
    add(&mut d, entry2);
    let r = supported(&d);
    assert_eq!(value(&r, "gambling_winnings"), 12_000);
    assert_eq!(return_line(&r, "8"), 1_200_000);
    assert_eq!(return_line(&r, "25c"), 100_000);
    assert_eq!(value(&r, "agi"), 72_000);
}

#[test]
fn personal_sale_term_and_statement_withholding_flow_to_result() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(personal_sale(4000, 1000, GainTerm::Short)));
    let FederalIncome::Sale { data } = &mut d.income[1].income else { panic!() };
    data.withholding = 24000;
    let r = supported(&d);
    assert_eq!(value(&r, "income_tax"), 5383);
    assert_eq!(r.payments, Some(674000));
    let FederalIncome::Sale { data } = &mut d.income[1].income else { panic!() };
    data.special = crate::SaleSpecial::Another;
    assert!(noted(&supported(&d), "WARN_SALE_OTHER_LEFT_OUT"));
}

#[test]
fn section_1250_rates_fill_lower_brackets_before_the_25_percent_ceiling() {
    use crate::federal_rules::preferential_tax_with_special_gains as tax;
    assert_eq!(tax(10000, 10000, 0, 10000, FederalStatus::Single), 1003);
    let baseline = preferential_tax(201775, 0, FederalStatus::Single);
    assert_eq!(tax(202775, 1000, 0, 1000, FederalStatus::Single), baseline + 250);
    let baseline = preferential_tax(200775, 0, FederalStatus::Single);
    assert_eq!(tax(202775, 2000, 0, 2000, FederalStatus::Single), baseline + 490);
    // Deductions consume special-rate gain before ordinary preferential gains.
    assert_eq!(tax(5000, 20000, 5000, 5000, FederalStatus::Single), 0);
    // 100,000 ordinary base, 101,775 of 1250 gain in lower brackets,
    // 98,225 at 25%, 300,000 at 28%, and 100,000 at 15%.
    assert_eq!(tax(700000, 600000, 300000, 200000, FederalStatus::Single), 164580);
    // AMT has no exemption at this income. Its ordinary base is 400,000,
    // with 200,000 at 25% and 100,000 at 15%.
    assert_eq!(
        crate::extended_rules::amt_tax_with_special_gains(
            700000,
            100000,
            600000,
            FederalStatus::Single,
            0,
            [300000, 200000]
        ),
        172110
    );
    // Exemption leaves only 9,900 taxable; entirely in the 25% pool.
    assert_eq!(
        crate::extended_rules::amt_tax_with_special_gains(
            100000,
            0,
            100000,
            FederalStatus::Single,
            0,
            [0, 100000]
        ),
        2475
    );
}
#[test]
fn paid_guarantee_takes_no_loss_for_favors_live_rights_and_early_no_right_payment() {
    // F3 (2026-10-09): a failing guarantee fact is no loss (the estimate
    // proceeds); an unresolved right is assumed the same with one note.
    for case in 0..4 {
        let mut d = complete(FederalStatus::Single);
        let mut debt = bad_debt(5000, 0);
        let FederalIncome::NonbusinessBadDebt { data: crate::NonbusinessBadDebt { source, .. } } =
            &mut debt
        else {
            panic!()
        };
        *source = BadDebtSource::PaidGuarantee {
            paid: 500_000,
            reimbursed: 0,
            paid_year: if case == 2 { 2025 } else { 2026 },
            motive: if case == 0 { GuaranteeMotive::Favor } else { GuaranteeMotive::Profit },
            recovery_right: match case {
                1 => GuaranteeRight::Recoverable,
                3 => GuaranteeRight::NotSure,
                _ => GuaranteeRight::None,
            },
        };
        add(&mut d, income(debt));
        let r = supported(&d);
        assert!(!r.lines.iter().any(|line| line.id.starts_with("bad_debt_")), "{case}");
        assert_eq!(noted(&r, "WARN_BAD_DEBT_NOT_DEDUCTED"), case == 3, "{case}");
    }
    // The unresolved purpose is the same unknown; a protected investment
    // with a wholly worthless claim in 2026 takes the loss.
    for (motive, loss) in
        [(GuaranteeMotive::Other, false), (GuaranteeMotive::ProtectInvestment, true)]
    {
        let mut d = complete(FederalStatus::Single);
        let mut debt = bad_debt(5000, 0);
        let FederalIncome::NonbusinessBadDebt { data: crate::NonbusinessBadDebt { source, .. } } =
            &mut debt
        else {
            panic!()
        };
        *source = BadDebtSource::PaidGuarantee {
            paid: 500_000,
            reimbursed: 0,
            paid_year: 2026,
            motive,
            recovery_right: GuaranteeRight::None,
        };
        add(&mut d, income(debt));
        let r = supported(&d);
        assert_eq!(r.lines.iter().any(|line| line.id.starts_with("bad_debt_")), loss);
        assert_eq!(noted(&r, "WARN_BAD_DEBT_NOT_DEDUCTED"), !loss);
    }
}

#[test]
fn niit_unclassified_personal_income_tax_reduces_no_investment_income_with_a_warning() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
    add(&mut d, income(savings_income(100000, 0)));
    let facts = itemized(d.deductions.as_mut().unwrap());
    facts.investment_property_taxes = 5000000;
    facts.personal_tax =
        PersonalTaxElection::Income { paid: 1000000, allocation: TaxIncomeAllocation::Unresolved };
    let facts = facts.clone();
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Itemized { facts };
    // Not resolved: none of it reduces investment income, with a warning.
    let r = supported(&d);
    assert!(noted(&r, "WARN_NIIT_STATE_TAX_NOT_ALLOCATED"), "{r:?}");
}

#[test]
fn niit_income_tax_allocation_uses_current_income_before_expenses() {
    let d = niit_tax_source_return(PersonalTaxElection::Income {
        paid: 1000000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    });
    let r = supported(&d);
    // Gross investment income $100,000 / AGI $400,000, not the $50,000
    // remaining after investment-property taxes: $2,500 income-tax deduction.
    assert_eq!(value(&r, "niit_income_tax_deduction"), 2500);
    assert_eq!(value(&r, "net_investment_income_tax"), 1805);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 400000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn niit_income_tax_accepts_documented_source_income_and_checks_bounds() {
    let mut d = niit_tax_source_return(PersonalTaxElection::Income {
        paid: 1000000,
        allocation: TaxIncomeAllocation::SourceIncome { investment: 8000000, total: 10000000 },
    });
    let r = supported(&d);
    assert_eq!(value(&r, "niit_income_tax_deduction"), 8000);
    assert_eq!(value(&r, "net_investment_income_tax"), 1596);
    assert_eq!(restart(d.clone()).draft, d);
    itemized(d.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
        paid: 1000000,
        allocation: TaxIncomeAllocation::SourceIncome { investment: 200, total: 100 },
    };
    assert!(!d.valid_stored());
    itemized(d.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Sales { paid: -1 };
    assert!(!d.valid_stored());
}

#[test]
fn niit_section_68_uses_aggregate_cap_instead_of_proportional_tax_reduction() {
    let mut d = niit_tax_source_return(PersonalTaxElection::Income {
        paid: 1000000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    });
    d.income[0] = wage(1, Owner::Taxpayer, 80000000, 0);
    let r = supported(&d);
    // Personal SALT floor 10,000; the income-tax share is 1,111.
    // Eligible NII deductions 51,111 are below total itemized deductions
    // 56,757 after section 68, so the final regulation preserves all 51,111.
    assert_eq!(value(&r, "itemized_deduction_reduction"), 3243);
    assert_eq!(value(&r, "niit_income_tax_deduction"), 1111);
    assert_eq!(value(&r, "niit_itemized_deductions"), 51111);
    assert_eq!(value(&r, "niit_itemized_deduction_reduction"), 0);
    assert_eq!(value(&r, "net_investment_income_tax"), 1858);
}

#[test]
fn niit_unknown_income_tax_allocation_cannot_change_an_already_binding_aggregate_cap() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 80000000, 0);
    add(&mut d, income(savings_income(100000, 0)));
    investment_borrowing(&mut d, 40000, 0, 0);
    let facts = itemized(d.deductions.as_mut().unwrap());
    facts.investment_property_taxes = 5000000;
    facts.personal_tax =
        PersonalTaxElection::Income { paid: 100000, allocation: TaxIncomeAllocation::Unresolved };
    let r = supported(&d);
    // Known eligible deductions of 90,000 already exceed 91,000 minus
    // section 68's 4,919 reduction. Any income-tax share leaves the cap unchanged.
    assert_eq!(value(&r, "itemized_deduction_reduction"), 4919);
    assert_eq!(value(&r, "niit_itemized_deductions"), 86081);
    assert_eq!(value(&r, "net_investment_income_tax"), 529);
    assert!(!r.lines.iter().any(|line| matches!(
        line.id.as_str(),
        "niit_income_tax_deduction" | "niit_itemized_deduction_reduction"
    )));
}

#[test]
fn nonqualified_death_and_disability_remove_early_tax_not_income_or_niit() {
    for facts in [
        RetirementException::Death { died: 20260615 },
        RetirementException::Disability { began: 20260615 },
    ] {
        let d = nonqualified_exception(facts);
        let r = supported(&d);
        assert_eq!(value(&r, "early_distribution_tax"), 0);
        assert_eq!(value(&r, "agi"), 215000);
        assert_eq!(value(&r, "net_investment_income_tax"), 190);
        assert_eq!(return_line(&r, "5a"), 1000000);
        assert_eq!(return_line(&r, "5b"), 500000);
        assert_eq!(restart(d.clone()).draft, d);
        let mut ordinary = d.clone();
        let FederalIncome::Retirement { data } = &mut ordinary.income[1].income else { panic!() };
        data.ordinary_distribution = RetirementTreatment::Early;
        assert_eq!(value(&supported(&ordinary), "early_distribution_tax"), 500);
    }
}

#[test]
fn macrs_flows_to_return_without_spurious_amt_adjustment_and_retains_ubia() {
    let mut d = business_with_bonus(
        100000,
        macrs_equipment(40000, EquipmentClass::Five, EquipmentMethod::Regular, 20260601),
    );
    d.income[0] = wage(1, Owner::Taxpayer, 50000000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_deduction"), 1000);
    let mut ordinary = d.clone();
    let FederalIncome::Business { data } = &mut ordinary.income[1].income else { panic!() };
    data.assets.clear();
    data.expenses.push(ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Equivalent deduction".into() },
        800000,
    ));
    let expected = supported(&ordinary);
    for id in ["agi", "self_employment_tax", "amt_rental_adjustment"] {
        assert_eq!(value(&r, id), value(&expected, id));
    }
    assert_eq!(crate::interview::document_amount(&d.income[1].income, false), None);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn seller_adjusted_later_sales_match_ordinary_sale_tax_and_withholding() {
    for (term, collectible, proceeds) in [
        (GainTerm::Short, false, 1200000),
        (GainTerm::Long, true, 2200000),
        (GainTerm::Long, false, 600000),
    ] {
        let mut d = complete(FederalStatus::Single);
        let mut entry = price_reduction_debt();
        if let FederalIncome::PersonalDebt { data } = &mut entry.income
            && let PersonalDebtTreatment::PriceReduction { sale, .. } = &mut data.treatment
        {
            *sale = Some(DebtPropertySale {
                date: 20261201,
                term,
                collectible,
                proceeds,
                withholding: 12345,
            });
        }
        add(&mut d, entry);
        let r = supported(&d);
        let mut ordinary = complete(FederalStatus::Single);
        add(
            &mut ordinary,
            income(FederalIncome::Sale {
                data: FederalSale {
                    collectible,
                    withholding: 12345,
                    use_kind: SaleUse::Personal,
                    basis_reporting: BasisReporting::NoForm1099B,
                    ..native_sale(term, proceeds, Some(800000))
                },
            }),
        );
        let reference = supported(&ordinary);
        for id in ["agi", "total_tax"] {
            assert_eq!(value(&r, id), value(&reference, id));
        }
        assert_eq!(return_line(&r, "25d"), return_line(&reference, "25d"));
        assert_eq!(line_amount(&r, "debt_2_sale_gain"), (proceeds - 800000).max(0));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn older_200_percent_and_pre1999_asset_histories_still_require_amt_records() {
    let mut asset = continuing_equipment(20170601, 850000, false);
    asset.cost = 1000000;
    asset.acquired = 20160101;
    asset.class = EquipmentClass::Ten;
    unsupported(&business_with_bonus(50000, asset.clone()), "Income:");
    // Exhausted regular basis cannot prove that the different old AMT
    // recovery period has also ended.
    asset.placed = 19980601;
    asset.acquired = 19980101;
    asset.method = EquipmentMethod::StraightLine;
    if let EquipmentHistory::Continuing {
        amt: EquipmentAmtHistory::Same, prior_depreciation, ..
    } = &mut asset.history
    {
        *prior_depreciation = asset.cost;
    }
    unsupported(&business_with_bonus(50000, asset), "Income:");
}

#[test]
fn young_filers_use_normal_amt_exemption_even_without_earned_income() {
    for (base, expected) in [(90100, 0), (100000, 2574), (200000, 28574)] {
        let mut d = young_interest_return();
        d.income[0].income =
            FederalIncome::Interest { data: native_exempt_interest(base * 100, base * 100) };
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 0);
        assert_eq!(value(&r, "alternative_minimum_taxable_income"), base);
        assert_eq!(value(&r, "alternative_minimum_tax"), expected);
        assert_eq!(r.tax, Some(expected * 100));
        assert!(r.review_notes.iter().any(|note| note.contains("55(d)(4)(A)(iii)")));
        let person = &mut d.people.as_mut().unwrap().taxpayer;
        person.birth_date = 19800615;
        person.dependency = FilerDependency::Independent;
        let adult = supported(&d);
        assert_eq!(adult.tax, r.tax);
    }
}

#[test]
fn minimum_tax_history_unknown_is_unclaimed_and_ownership_and_invalid_storage_are_not_zeroed() {
    let mut d = complete(FederalStatus::Single);
    let mut h = minimum_tax_history();
    h.mtcnol = crate::MinimumTaxLoss::Unavailable;
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(h));
    // Missing exclusion-only records leave only the credit unclaimed. A prior
    // joint status attached to an individual return unit is inconsistent input.
    assert!(noted(&supported(&d), "WARN_MTC_NOL_LEFT_OUT"));
    let h = &mut d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts;
    h.mtcnol = crate::MinimumTaxLoss::None;
    h.status = FederalStatus::Joint;
    assert!(!d.valid_stored());
    assert_eq!(estimate(&d).state, ResultState::Incomplete);
    let h = &mut d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts;
    h.status = FederalStatus::Single;
    h.carry = -1;
    assert!(!d.valid_stored());
    let h = &mut d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts;
    h.carry = 0;
    h.exclusions[0].source.clear();
    assert!(!d.valid_stored());
}
#[test]
fn material_trading_income_is_niit_but_not_form4952_income() {
    let d = trading_interest_draft(10_000, true, 5_000);
    k1_interest_note(&estimate(&d), 10_000, 10_000);
}
#[test]
fn dated_note_payments_must_reconcile_to_principal_and_valid_current_year_dates() {
    let original = mixed_note_repayments();
    assert!(original.valid());
    for (advanced, date, first, second) in [
        (20250615, 20260230, 200_000, 300_000),
        (20250615, 20251231, 200_000, 300_000),
        (20250615, 20270101, 200_000, 300_000),
        (20260101, 20260615, 200_000, 300_000),
        (20250229, 20260615, 200_000, 300_000),
        (20250615, 20260615, 199_999, 300_000),
        (20250615, 20260615, 0, 500_000),
        (20250615, 20260615, -1, 500_001),
    ] {
        let mut source = original.clone();
        source.loans[0].repayment_term = ScorpNoteHolding::Dated {
            advanced,
            payments: vec![
                ScorpNoteRepayment { date, principal: first },
                ScorpNoteRepayment { date: 20260616, principal: second },
            ],
        };
        assert!(!source.valid());
    }
}

#[test]
fn niit_history_distinguishes_missing_zero_and_conflicting_balances() {
    let mut draft = complete(FederalStatus::Single);
    draft.screening.as_mut().unwrap().capital_history = vec![niit_loss_history(None)];
    unsupported(&draft, "Investment-tax capital history");
    draft.screening.as_mut().unwrap().capital_history[0].niit_difference = Some(0);
    supported(&draft);
    draft.screening.as_mut().unwrap().capital_history[0].niit_difference = Some(1500000);
    unsupported(&draft, "Investment-tax capital history"); // larger than $14k carry-in
    draft.screening.as_mut().unwrap().capital_history[0].niit_difference = Some(700000);
    draft.screening.as_mut().unwrap().capital_history[0].investment_parts =
        Some(InvestmentCapitalParts { short: 400000, long: 1000000, amt: None });
    // Form 4952 source portions do not establish the separate section 1411 balance.
    supported(&draft);
}

#[test]
fn multiple_full_recapture_preserves_mixed_niit_classification() {
    let mut draft = multiple_fully_recaptured_gains(30000);
    draft.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
    let r = supported(&draft);
    assert_eq!(value(&r, "agi"), 350000);
    assert_eq!(value(&r, "qbi_deduction"), 10000);
    assert_eq!(value(&r, "regular_income_tax"), 82134);
    assert_eq!(value(&r, "net_investment_income_tax"), 760);
    assert_eq!(r.tax, Some(8379400));
}

#[test]
fn unemployment_fully_repaid_benefits_preserve_actual_withholding() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 6000000, 0);
    add(&mut d, government_unemployment(10000, &[10000], 1000));
    let r = supported(&d);
    assert_eq!(value(&r, "unemployment_2_taxable"), 0);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(r.tax, Some(502300));
    assert_eq!(r.payments, Some(100000));
    assert_eq!(r.balance, Some(402300));
}

#[test]
fn general_rule_payment_frequency_reproduces_irs_timing_adjustments() {
    for (frequency, count, delay, expected) in [
        (AnnuityFrequency::Quarterly, 4, 1, 11580000), // IRS age-66 example.
        (AnnuityFrequency::Quarterly, 4, 3, 11460000),
        (AnnuityFrequency::Semiannual, 2, 6, 11400000),
        (AnnuityFrequency::Annual, 1, 12, 11220000),
        (AnnuityFrequency::Monthly, 12, 1, 11520000),
    ] {
        let mut a = fixed_annuity();
        a.frequency = frequency;
        a.periodic = 600000 / i64::from(count);
        a.received = count;
        a.duration = AnnuityDuration::SingleLife { age: 66, delay, guaranteed: 0 };
        a.cost = expected / 5;
        assert!(a.valid(600000));
        assert_eq!(a.expected(), expected);
        assert_eq!(a.ratio(), 200);
        assert_eq!(a.excluded(), 120000);
        let mut d = complete(FederalStatus::Single);
        let mut entry = retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Nonqualified,
            6000,
            RetirementTreatment::Ordinary,
        );
        if let FederalIncome::Retirement { data } = &mut entry.income {
            data.taxable = RetirementTaxable::GeneralAnnuity { data: a };
        }
        add(&mut d, entry);
        assert_eq!(return_line(&supported(&d), "5b"), 480000);
        assert_eq!(restart(d.clone()).draft, d);
    }
    let mut a = fixed_annuity();
    a.frequency = AnnuityFrequency::Quarterly;
    a.received = 4;
    a.duration = AnnuityDuration::SingleLife { age: 66, delay: 4, guaranteed: 0 };
    assert!(!a.valid(a.periodic * 4));
}

#[test]
fn registered_vendor_government_diesel_reaches_form_1040_payment() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { data: mut business } = cash_business(10_000) else {
        unreachable!()
    };
    business.fuel_tax_credit = Some(FederalFuelTaxCredit {
        fuels: FederalFuelLines {
            vendor_state_diesel: Some(government_vendor_claim(100_000)),
            ..FederalFuelLines::default()
        },
        expense_treatment: FederalFuelExpenseTreatment::TaxExcludedFromExpenses,
        records_complete: true,
    });
    add(&mut draft, income(FederalIncome::Business { data: business }));

    let mut without_credit = draft.clone();
    let FederalIncome::Business { data } = &mut without_credit.income[1].income else {
        unreachable!()
    };
    data.fuel_tax_credit = None;
    let baseline = supported(&without_credit);
    let report = supported(&draft);
    assert_eq!(value(&report, "business_2_fuel_6a"), 24);
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 2_400);
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.vendor_state_diesel.as_mut().unwrap().sales[0]
        .buyer_certificate_current = false;
    fuel_left_out(&draft);
}

#[test]
fn form_1040_withholding_lines_round_each_document_category_once() {
    let mut draft = complete(FederalStatus::Single);
    draft.payments.as_mut().unwrap().extension = 12_345;
    let FederalIncome::Wage { data } = &mut draft.income[0].income else { unreachable!() };
    data.withholding = 49;
    add(
        &mut draft,
        income(FederalIncome::Interest {
            data: FederalInterest {
                taxable: 10_000,
                withholding: 49,
                special: InterestSpecial::NoneOfThese,
                ..Default::default()
            },
        }),
    );
    add(
        &mut draft,
        income(FederalIncome::Gambling {
            data: FederalGambling {
                winnings: 10_000,
                losses: 0,
                withholding: 49,
                own_records: false,
                ordinary: true,
            },
        }),
    );
    let below = supported(&draft);
    for line in ["25a", "25b", "25c", "25d"] {
        assert_eq!(return_line(&below, line), 0, "{line}");
    }
    assert_eq!(return_line(&below, "31"), 12_300);
    assert_eq!(return_line(&below, "32a"), 12_300);
    assert_eq!(return_line(&below, "32c"), 12_300);
    assert_eq!(
        return_line(&below, "33"),
        return_line(&below, "25d") + return_line(&below, "26") + return_line(&below, "32c")
    );

    let FederalIncome::Wage { data } = &mut draft.income[0].income else { unreachable!() };
    data.withholding = 50;
    let FederalIncome::Interest { data } = &mut draft.income[1].income else { unreachable!() };
    data.withholding = 50;
    let FederalIncome::Gambling { data } = &mut draft.income[2].income else { unreachable!() };
    data.withholding = 50;
    let above = supported(&draft);
    for line in ["25a", "25b", "25c"] {
        assert_eq!(return_line(&above, line), 100, "{line}");
    }
    assert_eq!(return_line(&above, "25d"), 300);
    assert_eq!(return_line(&above, "33"), return_line(&below, "33") + 300);
    assert_eq!(
        return_line(&above, "33"),
        return_line(&above, "25d") + return_line(&above, "26") + return_line(&above, "32c")
    );
    assert_eq!(above.tax, below.tax);
}

#[test]
fn single_employer_excess_social_security_is_refused_not_credited() {
    // W-2 map row 4/2026 wage base: one employer's box 3 at exactly the $184,500.00 2026
    // Social Security wage base, box 4 withheld $11,500.00 -- $61.00 more than 6.2% x
    // $184,500.00 = $11,439.00, the 2026 per-employer maximum. Even though the excess
    // ($61.00) is small and the total also exceeds the per-taxpayer cap, this is one
    // employer's own over-withholding, which forms/w2.rs `collect` refuses by name rather
    // than crediting it through the Schedule 3 line 11 multiple-employer excess Social
    // Security credit (that credit requires at least two employers, each within its own
    // limit).
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Wage { data } = &mut draft.income[0].income else { unreachable!() };
    data.wages = 18_450_000;
    data.social_wages = 18_450_000;
    data.social_tax = 1_150_000;
    data.medicare_wages = 18_450_000;
    data.medicare_tax = 267_525; // exactly 1.45% of $184,500.00; only the SS box is wrong.
    let r = estimate(&draft);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    assert!(
        r.issues.iter().any(|i| i.message.contains(
            "Excess Social Security withholding by one employer requires employer correction, not the multiple-employer credit."
        )),
        "{r:?}"
    );
    assert!(r.tax.is_none() && r.refund.is_none() && r.lines.is_empty());
}

#[test]
fn box6_medicare_withholding_below_1_45_percent_of_box5_is_refused() {
    // W-2 map row 6: box 6 Medicare tax withheld must be at least 1.45% of box 5 Medicare
    // wages (less a 1-cent rounding tolerance). Box 5 stays at the default $60,000.00 (1.45%
    // = $870.00); box 6 is dropped to $700.00, $170.00 short. forms/w2.rs `collect` refuses
    // this by name instead of silently accepting an under-withheld box 6.
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Wage { data } = &mut draft.income[0].income else { unreachable!() };
    data.medicare_tax = 70_000;
    let r = estimate(&draft);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    assert!(
        r.issues.iter().any(|i| i.message.contains(
            "Medicare withholding below ordinary payroll tax requires special payroll review."
        )),
        "{r:?}"
    );
    assert!(r.tax.is_none() && r.refund.is_none() && r.lines.is_empty());
}

#[test]
fn church_employee_income_enters_schedule_se_line_5() {
    // Unit C10a; i1040sse 2026 draft line 5a: church employee income (this
    // W-2's box 1 wages; boxes 3 and 5 are zero because the church elected
    // out of Social Security/Medicare). Line 5b = 5a x 92.35%: $12,000 x
    // 0.9235 = $11,082.00 exactly. There is no Schedule C, so line 4c = 0;
    // line 6 = 4c + 5b = $11,082.
    let mut d = empty_return(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Wage {
            data: FederalWage {
                abroad: 0,
                deferred_pension: 0,
                inmate_pay: 0,
                disability: DisabilityIncome::None,
                wages: 1_200_000,
                withholding: 0,
                social_wages: 0,
                social_tips: 0,
                employee_tips: None,
                work_deductions: FederalWorkDeductions::NONE,
                social_tax: 0,
                medicare_wages: 0,
                medicare_tax: 0,
                uncollected_tax: None,
                medicaid_waiver: 0,
                statutory_business: None,
                paying_scorporation: None,
                payroll: crate::W2Payroll::SocialSecurity,
                coverage: crate::W2Coverage::ChurchEmployee,
                care_benefits: 0,
                statutory_employee: false,
                retirement_plan: false,
                other_box12: Vec::new(),
            },
        }),
    );
    add(&mut d, wage(2, Owner::Taxpayer, 3_000_000, 0));
    // Line 7's $184,500 wage base minus line 8d ($30,000 ordinary Social
    // Security wages) leaves $154,500 of headroom, above line 6's $11,082,
    // so the wage base does not limit either part. se_from_net computes the
    // 12.4% and 2.9% parts in cents, truncating each before summing, then
    // rounds the combined whole-dollar result once (federal_rules.rs):
    // 11,082 x 124 / 10 = 137,416 (cents, truncated from 137,416.8);
    // 11,082 x 29 / 10 = 32,137 (cents, truncated from 32,137.8);
    // (137,416 + 32,137 + 50) / 100 = $1,696 SE tax (Schedule 2 line 4).
    // Half-SE deduction (Schedule 1 line 15): (1,696 + 1) / 2 = $848.
    // AGI: $12,000 + $30,000 wages - $848 half-SE deduction = $41,152.
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 1696);
    assert_eq!(value(&r, "adjustments"), 848);
    assert_eq!(value(&r, "agi"), 41_152);
}

#[test]
fn church_employee_income_under_the_floor_is_zero() {
    // Unit C10a; i1040sse 2026 draft line 5b: 92.35% of $100 church employee
    // income is $92.35, which rounds to $92 and is under the $100 floor, so
    // line 5b (and, with no other self-employment income, line 6) is 0 and
    // no self-employment tax is owed.
    let mut d = empty_return(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Wage {
            data: FederalWage {
                abroad: 0,
                deferred_pension: 0,
                inmate_pay: 0,
                disability: DisabilityIncome::None,
                wages: 10_000,
                withholding: 0,
                social_wages: 0,
                social_tips: 0,
                employee_tips: None,
                work_deductions: FederalWorkDeductions::NONE,
                social_tax: 0,
                medicare_wages: 0,
                medicare_tax: 0,
                uncollected_tax: None,
                medicaid_waiver: 0,
                statutory_business: None,
                paying_scorporation: None,
                payroll: crate::W2Payroll::SocialSecurity,
                coverage: crate::W2Coverage::ChurchEmployee,
                care_benefits: 0,
                statutory_employee: false,
                retirement_plan: false,
                other_box12: Vec::new(),
            },
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(value(&r, "agi"), 100);
}

#[test]
fn church_w2_with_social_security_wages_is_refused() {
    // Unit C10a: a church employee W-2 elects out of Social Security and
    // Medicare, so boxes 3 and 5 must be zero. `complete()`'s default W-2
    // carries nonzero box 3 (Social Security wages); marking it as church
    // employee income without clearing box 3/5 is refused by name, naming
    // Schedule SE line 5a, instead of silently computing both a payroll tax
    // base and church employee income from the same wages.
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Wage { data } = &mut draft.income[0].income else { unreachable!() };
    data.coverage = crate::W2Coverage::ChurchEmployee;
    let r = estimate(&draft);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    assert!(r.issues.iter().any(|i| i.message.contains("Schedule SE line 5a")), "{r:?}");
    assert!(r.tax.is_none() && r.refund.is_none() && r.lines.is_empty());
}

#[test]
fn form_8919_withholding_uses_the_reported_document_category() {
    let mut draft = complete(FederalStatus::Single);
    let mut wages = omitted_employee_wages(1_000);
    let FederalIncome::EmployeeWages { data } = &mut wages else { unreachable!() };
    data.reason = EmployeeWageReason::Ss8Determination { date: 20260901 };
    data.form1099 = false;
    data.withholding = 50_000;
    add(&mut draft, income(wages));

    let other = supported(&draft);
    assert_eq!(return_line(&other, "25b"), 0);
    assert_eq!(return_line(&other, "25c"), 50_000);
    let FederalIncome::EmployeeWages { data } = &mut draft.income[1].income else { unreachable!() };
    data.form1099 = true;
    let form1099 = supported(&draft);
    assert_eq!(return_line(&form1099, "25b"), 50_000);
    assert_eq!(return_line(&form1099, "25c"), 0);
    assert_eq!(return_line(&form1099, "25d"), return_line(&other, "25d"));
    assert_eq!(form1099.tax, other.tax);
}

#[test]
fn trading_nonpassive_status_does_not_establish_active_qbi_minimum() {
    let mut partner = ordinary_partner_source(1000, false);
    partner.trade_kind = ScorpTradeKind::FinancialTrading;
    let statement = partner_qbi_statement(1000);
    let activity = statement
        .activity(
            &partner,
            crate::tests::support::builders::k1_amount(0),
            0,
            50000,
            FederalStatus::Single,
        )
        .unwrap();
    assert!(!activity.active);
    let mut report = supported(&complete(FederalStatus::Single));
    assert_eq!(
        crate::business_rules::qbi(&[activity], 50000, 0, FederalStatus::Single, &mut report).0,
        200
    );
}

#[test]
fn qualified_wildfire_relief_is_excluded_but_1099_withholding_is_paid() {
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    add(
        &mut d,
        income(FederalIncome::WildfireRelief {
            data: FederalWildfireRelief {
                paid: 20260912,
                declared: 20260801,
                reference: "DR-1234, synthetic forest fire".into(),
                amount: 2_000_000,
                withholding: 35_000,
                qualified: true,
            },
        }),
    );
    let relief = supported(&d);
    let id = d.income.last().unwrap().id;
    assert_eq!(value(&relief, &alloc::format!("wildfire_relief_{id}_excluded")), 20_000);
    assert_eq!(value(&relief, "agi"), value(&baseline, "agi"));
    assert_eq!(relief.tax, baseline.tax);
    assert_eq!(return_line(&relief, "25b"), 35_000);
    assert_eq!(return_line(&relief, "25d"), return_line(&baseline, "25d") + 35_000);
    assert_eq!(estimate(&restart(d.clone()).draft), relief);

    let FederalIncome::WildfireRelief { data } = &mut d.income.last_mut().unwrap().income else {
        panic!("wildfire source changed");
    };
    // No: the payment is other income; the withholding is still paid.
    data.qualified = false;
    let taxed = supported(&d);
    assert!(value(&taxed, "taxable_relief_payments") > 0);
    assert!(value(&taxed, "agi") > value(&baseline, "agi"));

    let FederalIncome::WildfireRelief { data } = &mut d.income.last_mut().unwrap().income else {
        panic!("wildfire source changed");
    };
    data.qualified = true;
    data.declared = 20141231;
    assert!(!d.valid_stored());
    let FederalIncome::WildfireRelief { data } = &mut d.income.last_mut().unwrap().income else {
        panic!("wildfire source changed");
    };
    data.declared = 20260801;
    data.withholding = data.amount + 1;
    assert!(!d.valid_stored());
}

#[test]
fn qualified_disaster_relief_is_recorded_without_taxing_the_payment() {
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    let first_relief = d.income.len();
    for purpose in [
        DisasterReliefPurpose::PersonalExpense,
        DisasterReliefPurpose::ResidenceOrContents,
        DisasterReliefPurpose::GovernmentWelfare,
    ] {
        add(
            &mut d,
            income(FederalIncome::DisasterRelief {
                data: FederalDisasterRelief {
                    received: 20260912,
                    qualification: DisasterQualification::FederalDeclaration {
                        reference: "DR-1234, synthetic federal disaster".into(),
                    },
                    purpose,
                    amount: 125_000,
                    withholding: 0,
                    qualified: true,
                },
            }),
        );
    }
    let relief = supported(&d);
    for entry in &d.income[first_relief..] {
        assert_eq!(value(&relief, &alloc::format!("disaster_relief_{}_excluded", entry.id)), 1_250);
    }
    assert_eq!(value(&relief, "agi"), value(&baseline, "agi"));
    assert_eq!(relief.tax, baseline.tax);
    assert_eq!(return_line(&relief, "25d"), return_line(&baseline, "25d"));
    assert_eq!(estimate(&restart(d.clone()).draft), relief);

    if let FederalIncome::DisasterRelief { data } = &mut d.income.last_mut().unwrap().income {
        data.qualified = false;
    }
    let taxed = supported(&d);

    assert!(value(&taxed, "agi") > value(&baseline, "agi"));
    if let FederalIncome::DisasterRelief { data } = &mut d.income.last_mut().unwrap().income {
        data.qualified = true;
        data.received = 20270201;
    }
    assert!(!d.valid_stored());
    if let FederalIncome::DisasterRelief { data } = &mut d.income.last_mut().unwrap().income {
        data.received = 20260912;
        data.qualification = DisasterQualification::FederalDeclaration { reference: "".into() };
    }
    assert!(!d.valid_stored());
}

#[test]
fn carrier_disaster_death_or_physical_injury_payment_is_excluded() {
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    let first = d.income.len();
    for loss in [DisasterCarrierLoss::Death, DisasterCarrierLoss::PhysicalInjury] {
        add(
            &mut d,
            income(FederalIncome::DisasterRelief {
                data: FederalDisasterRelief {
                    received: 20261001,
                    qualification: DisasterQualification::CarrierAccident {
                        event: "Synthetic 2026 rail accident".into(),
                    },
                    purpose: DisasterReliefPurpose::CommonCarrier {
                        carrier: "Synthetic passenger railway".into(),
                        loss,
                    },
                    amount: 1_500_000,
                    withholding: 0,
                    qualified: true,
                },
            }),
        );
    }
    let result = supported(&d);
    for entry in &d.income[first..] {
        assert_eq!(
            value(&result, &alloc::format!("disaster_relief_{}_excluded", entry.id)),
            15_000
        );
    }
    assert_eq!(value(&result, "agi"), value(&baseline, "agi"));
    assert_eq!(result.tax, baseline.tax);
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    let FederalIncome::DisasterRelief { data } = &mut d.income[first].income else {
        unreachable!()
    };
    data.withholding = 15_000;
    let reported = supported(&d);
    assert_eq!(value(&reported, "agi"), value(&baseline, "agi"));
    assert_eq!(reported.tax, baseline.tax);
    assert_eq!(return_line(&reported, "25b"), return_line(&baseline, "25b") + 15_000);
    assert_eq!(
        value(&reported, &alloc::format!("disaster_relief_{}_withholding", d.income[first].id)),
        150
    );
    assert_eq!(estimate(&restart(d.clone()).draft), reported);

    let FederalIncome::DisasterRelief { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.qualified = false;

    let FederalIncome::DisasterRelief { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.qualified = true;
    data.purpose = DisasterReliefPurpose::PersonalExpense;
    assert!(!d.valid_stored());
    let FederalIncome::DisasterRelief { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.purpose = DisasterReliefPurpose::CommonCarrier {
        carrier: "".into(),
        loss: DisasterCarrierLoss::PhysicalInjury,
    };
    assert!(!d.valid_stored());
    let FederalIncome::DisasterRelief { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.purpose = DisasterReliefPurpose::CommonCarrier {
        carrier: "Synthetic passenger railway".into(),
        loss: DisasterCarrierLoss::PhysicalInjury,
    };
    data.withholding = data.amount + 1;
    assert!(!d.valid_stored());
}
#[test]
fn schedule_2_other_taxes_are_refused_by_line() {
    // Unit A2/A17 through the situations list: line 9 and line 13n are named.
    let d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    assert_situation_refused(
        &d,
        OtherTax::HousingCreditRecapture,
        &["Schedule 2 line 9", "line 13n"],
    );
    // An empty list keeps today's result.
    assert_eq!(estimate(&d).tax, baseline.tax);
}

#[test]
fn schedule_2_part_ii_other_taxes_are_refused_by_line() {
    let d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    assert_situation_refused(
        &d,
        OtherTax::OtherAdditionalTaxes { kind: RareAdditionalTax::AnotherIdentifiedAddedTax },
        &["An additional federal tax whose computation is outside the App scope"],
    );
    assert_eq!(estimate(&d).tax, baseline.tax);
}

#[test]
fn niit_refuses_cfc_pfic_and_other_modifications() {
    let d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    assert_situation_refused(
        &d,
        RareSituation::ForeignCorporation,
        &["Form 8960 line 6", "line 10"],
    );
    assert_eq!(estimate(&d).tax, baseline.tax);
}

#[test]
fn form_1040_elections_are_refused_by_line() {
    // Unit A7's single answer is one list choice per election, each naming its line.
    let d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    assert_situation_refused(
        &d,
        Chosen::from(RareSituation::RailroadRepresentative),
        &["Form CT-2"],
    );
    assert_eq!(estimate(&d).tax, baseline.tax);
}

#[test]
fn schedule_d_other_form_gains_are_refused_by_line() {
    let d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    for (situation, needles) in [
        (RareSituation::InstallmentSale, &["Form 6252", "Schedule D lines 4 and 11"][..]),
        (RareSituation::Straddles, &["Form 6781", "Schedule D lines 4 and 11"][..]),
        (
            RareSituation::LikeKindExchange,
            &["Form 8824", "that is not a rental", "conflict-of-interest"][..],
        ),
    ] {
        assert_situation_refused(&d, situation, needles);
    }
    assert_eq!(estimate(&d).tax, baseline.tax);
}

#[test]
fn additional_medicare_splits_between_schedule_2_lines_11_and_17b() {
    // Hand computation (i8959 Parts I and II; joint Additional Medicare
    // threshold $250,000):
    // W-2 Medicare wages (box 5) = $200,000.
    // Schedule C profit = $108,284; Schedule SE net earnings = profit x
    // 92.35%, rounded to the nearest dollar per federal_rules.rs `fraction`:
    // 108,284 x 9235 = 1,000,002,740; (1,000,002,740 + 5,000) / 10,000 =
    // 100,000 (integer division) -> net self-employment earnings = $100,000.
    // Form 8959 Part I (wages) uses the threshold first: wages $200,000 is
    // below the $250,000 threshold, so the wages part of the tax is $0, and
    // the remaining threshold carried to Part II is $250,000 - $200,000 =
    // $50,000.
    // Form 8959 Part II (self-employment): ($100,000 - $50,000) x 0.9% =
    // $450.
    // Total Additional Medicare tax = $0 (wages, line 17b) + $450
    // (self-employment, line 11) = $450.
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 20_000_000, 0);
    add(&mut d, income(cash_business(108_284)));
    let r = supported(&d);
    assert_eq!(value(&r, "additional_medicare_wages"), 0);
    assert_eq!(value(&r, "additional_medicare_self_employment"), 450);
    assert_eq!(value(&r, "additional_medicare_tax"), 450);
}

// C-fixture (matrix row 10, Schedule SE): a single filer with one W-2 above
// the wage base and one sole proprietorship, worked by hand from i1040sse,
// i1040gi and Pub 334.
#[test]
fn self_employment_with_the_wage_base_binding_matches_hand_computation() {
    // Schedule C: receipts $60,000.00, one Supplies expense $5,000.00.
    //   Line 28 total expenses: $5,000.00. Line 29/31 net profit:
    //   60,000.00 - 5,000.00 = $55,000.00.
    //
    // Schedule SE (ordinary nonfarm method, single business, no church income):
    //   Line 2/3: $55,000. Line 4a-4c: 55,000 x 92.35% = $50,792.50;
    //   `federal_rules.rs`'s `fraction` rounds a .50 remainder up:
    //   (55,000 x 9,235 + 5,000) / 10,000 = 507,930,000 / 10,000 = $50,793.
    //   Line 6: $50,793 (no church employee income). Line 7: $184,500 (2026
    //   wage base). Line 8a-8d: this W-2's box 3 Social Security wages,
    //   $150,000 (box 1 $150,000 is under the wage base, so box 3 equals
    //   box 1). Line 9: 184,500 - 150,000 = $34,500.
    //   Line 10 (12.4% of the smaller of line 6 and line 9): the smaller is
    //   line 9, $34,500; 34,500 x 12.4% = $4,278.00 exactly (427,800 cents).
    //   Line 11 (2.9% of line 6, never capped by the wage base): 50,793 x
    //   2.9% = $1,472.997, truncated by the App's integer cents arithmetic
    //   to $1,472.99 (147,299 cents).
    //   Combined cents: 427,800 + 147,299 = 575,099; (575,099 + 50) / 100 =
    //   5,751 (line 12 self-employment tax, Schedule 2 line 4).
    //   Line 13 half-SE deduction: (5,751 + 1) / 2 = $2,876 (Schedule 1
    //   line 15).
    //
    // Form 8995 (2026 single QBI threshold $201,750, far above this return,
    // so no SSTB/wage-limit phase-in applies):
    //   QBI per business = net profit less its attributable half-SE share:
    //   55,000 - 2,876 = $52,124. Line 5: 52,124 x 20% = $10,424.80, rounds
    //   to $10,425. Taxable income before QBI: AGI $202,124 - standard
    //   deduction $16,100 = $186,024; 20% of that is $37,204.80, rounds to
    //   $37,205, far above the $10,425 business component, so the business
    //   component controls. QBI deduction: $10,425.
    //
    // Additional Medicare tax (Form 8959; single $200,000 threshold):
    //   combined Medicare wages ($150,000, box 5) plus Schedule SE line 6
    //   ($50,793) = $200,793, which is $793 above the $200,000 threshold, so
    //   Additional Medicare tax applies: 793 x 0.9% = $7.137, rounds to $7.
    //   Form 8959 Part I applies the threshold to wages first: $150,000 is
    //   itself under $200,000, so the wages part (Schedule 2 line 17b) is
    //   $0 and the entire $7 falls on the self-employment part (Schedule 2
    //   line 11).
    //
    // Schedule 2 line 4 (self-employment tax) $5,751 plus line 11
    // (Additional Medicare) $7 = $5,758 additional taxes (Form 1040 line 23).
    //
    // Form 1040:
    //   Line 1a W-2 wages: $150,000. Line 8 additional income (Schedule 1
    //   line 3/10, the business profit only): $55,000. Line 9 total income:
    //   150,000 + 55,000 = $205,000.
    //   Line 10 adjustments (Schedule 1 line 15/26, the half-SE deduction
    //   only): $2,876. Line 11 AGI: 205,000 - 2,876 = $202,124.
    //   Line 12e standard deduction (single, born 1980, not 65 in 2026):
    //   $16,100. Line 13b QBI deduction: $10,425.
    //   Line 15 taxable income: 202,124 - 16,100 - 10,425 = $175,599.
    //   Line 16 regular tax ($175,599 is above the $100,000 Tax Table
    //   ceiling, so the 2026 rate schedule applies directly): 12,400 x 10% +
    //   (50,400-12,400) x 12% + (105,700-50,400) x 22% + (175,599-105,700)
    //   x 24% = 1,240.00 + 4,560.00 + 12,166.00 + 16,775.76 = $34,741.76,
    //   rounds to $34,742. No credits apply.
    //   Line 23 additional taxes: $5,758. Line 24a total tax: 34,742 +
    //   5,758 = $40,500.
    //   Line 25a withholding (box 2): $30,000. Line 33 total payments:
    //   $30,000 (no estimated payments, no refundable credits).
    //   Line 37 balance due: 40,500 - 30,000 = $10,500.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(1, Owner::Taxpayer, 15_000_000, 3_000_000));
    add(
        &mut d,
        income(FederalIncome::Business {
            data: FederalBusiness {
                activity: BusinessActivity::Ordinary {
                    returns: 0,
                    receipts: 6_000_000,
                    accounting: BusinessAccounting::Cash,
                },
                expenses: vec![ordinary_business_expense(
                    BusinessExpenseCategory::Supplies,
                    500_000,
                )],
                ..support::builders::survey::survey_cash_business_data()
            },
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 5_751);
    assert_eq!(value(&r, "adjustments"), 2_876);
    assert_eq!(value(&r, "agi"), 202_124);
    assert_eq!(value(&r, "qbi_deduction"), 10_425);
    assert_eq!(value(&r, "taxable_income"), 175_599);
    assert_eq!(value(&r, "additional_medicare_wages"), 0);
    assert_eq!(value(&r, "additional_medicare_self_employment"), 7);
    assert_eq!(value(&r, "additional_medicare_tax"), 7);

    assert_eq!(return_line(&r, "1a"), 15_000_000);
    assert_eq!(return_line(&r, "8"), 5_500_000);
    assert_eq!(return_line(&r, "9"), 20_500_000);
    assert_eq!(return_line(&r, "10"), 287_600);
    assert_eq!(return_line(&r, "11a / 11b"), 20_212_400);
    assert_eq!(return_line(&r, "12e"), 1_610_000);
    assert_eq!(return_line(&r, "13b"), 1_042_500);
    assert_eq!(return_line(&r, "15"), 17_559_900);
    assert_eq!(return_line(&r, "16"), 3_474_200);
    assert_eq!(return_line(&r, "24a"), 4_050_000);
    assert_eq!(return_line(&r, "25a"), 3_000_000);
    assert_eq!(return_line(&r, "33"), 3_000_000);
    assert_eq!(return_line(&r, "37"), 1_050_000);
    assert_eq!(r.tax, Some(4_050_000));
    assert_eq!(r.payments, Some(3_000_000));
    assert_eq!(r.balance, Some(1_050_000));
    assert_eq!(restart(d.clone()).draft, d);
}

// Boundary (1): W-2 Social Security wages exactly at the $184,500 2026 wage
// base leave no line 9 headroom, so line 10 (12.4%) is zero while line 11
// (2.9%) still taxes the full line 6 net earnings (i1040sse lines 7-11).
#[test]
fn wage_base_exactly_met_leaves_only_the_medicare_part() {
    // W-2 box 1/box 3: $184,500 exactly (the 2026 Social Security wage
    // base). Schedule C profit: $50,000 (cash_business, no expenses).
    //   Line 3: $50,000. Line 4a-4c: 50,000 x 92.35% = $46,175.00 exactly
    //   (50,000 x 9,235 = 461,750,000, evenly divisible by 10,000).
    //   Line 6: $46,175. Line 7: $184,500. Line 8d: $184,500 (this W-2's
    //   box 3). Line 9: 184,500 - 184,500 = $0.
    //   Line 10: 12.4% of min(46,175, 0) = $0.00.
    //   Line 11: 2.9% of the full $46,175 = $1,339.075, truncated by the
    //   App's integer cents arithmetic to $1,339.07 (133,907 cents).
    //   Combined cents: 0 + 133,907 = 133,907; (133,907 + 50) / 100 =
    //   $1,339 self-employment tax -- entirely the 2.9% Medicare part,
    //   since the 12.4% Social Security part contributed nothing.
    //   Half-SE deduction: (1,339 + 1) / 2 = $670.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(1, Owner::Taxpayer, 18_450_000, 0));
    add(&mut d, income(cash_business(50_000)));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 1_339);
    assert_eq!(value(&r, "adjustments"), 670);
}

// Boundary (2): the nonfarm optional method (i1040sse Part II) caps the
// elected earnings at $7,560 (the 2026 combined farm/nonfarm optional-method
// ceiling) even though two-thirds of this business's gross income is higher.
#[test]
fn nonfarm_optional_method_caps_at_7560() {
    // Schedule C: receipts $13,000.00, one "Other" ordinary expense
    // $8,000.00. Net profit (line 31): 13,000 - 8,000 = $5,000, which is
    // both under the $8,186 2026 nonfarm-method profit gate and under
    // 72.189% of the $13,000 gross ($9,384.57), so `optional_se::calculate`
    // allows the election (`se_history()`'s $400 actual earnings in 2023
    // and 2024 satisfy the two-of-three-prior-years test, and there are no
    // prior nonfarm uses).
    // Nonfarm optional earnings (line 17): two-thirds of the $13,000 gross
    // income (no COGS, so gross income equals receipts) = (13,000 x 2 + 1)
    // / 3 = $8,667, capped at the $7,560 ceiling (line 14/16), so the
    // elected net (line 4c) is $7,560, not the actual $5,000 profit or its
    // 92.35% equivalent ($4,617.50 -> $4,618).
    // Schedule SE line 6 = $7,560 (no wages, so no wage-base limit at all):
    //   Line 10: 7,560 x 12.4% = $937.44 exactly (93,744 cents).
    //   Line 11: 7,560 x 2.9% = $219.24 exactly (21,924 cents).
    //   Combined: 93,744 + 21,924 = 115,668 cents; (115,668 + 50) / 100 =
    //   $1,157 self-employment tax.
    //   Half-SE deduction: (1,157 + 1) / 2 = $579.
    let mut d = empty_return(FederalStatus::Single);
    let mut b = cash_business(5_000);
    let FederalIncome::Business { data } = &mut b else { unreachable!() };
    data.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: 1_300_000,
        accounting: BusinessAccounting::Cash,
    };
    data.expenses = vec![ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Operating expenses".into() },
        800_000,
    )];
    add(&mut d, income(b));
    elect_se(&mut d, SeMethod::Nonfarm { history: se_history() });
    let r = supported(&d);
    assert_eq!(value(&r, "optional_se_taxpayer_nonfarm"), 7_560);
    assert_eq!(value(&r, "optional_se_taxpayer_net"), 7_560);
    assert_eq!(value(&r, "self_employment_tax"), 1_157);
    assert_eq!(value(&r, "adjustments"), 579);
}

// Boundary (3): each owner on a joint return computes an independent
// Schedule SE with an independent $184,500 wage-base headroom (i1040sse's
// per-person computation) -- a spouse's own line 9 is not reduced by the
// other spouse's separate W-2 wages.
#[test]
fn joint_owners_each_have_their_own_schedule_se() {
    // Taxpayer: one W-2, box 1/box 3 $150,000, no self-employment income at
    // all (no Schedule SE net earnings, so lines 2-13 are all zero for the
    // taxpayer: $150,000 alone is not self-employment income).
    // Spouse: one sole proprietorship, $220,000 net profit, no W-2 wages of
    // her own.
    //   Line 3: $220,000. Line 4a-4c: 220,000 x 92.35% = $203,170.00 exactly
    //   (220,000 x 9,235 = 2,031,700,000, evenly divisible by 10,000).
    //   Line 6: $203,170. Line 7: $184,500. Line 8d: $0 (the spouse has no
    //   W-2 of her own; the taxpayer's $150,000 box 3 wages belong to a
    //   different person's Schedule SE and do not reduce the spouse's own
    //   line 9). Line 9: 184,500 - 0 = $184,500.
    //   Line 10: 12.4% of min(203,170, 184,500) = 184,500 x 12.4% =
    //   $22,878.00 exactly (2,287,800 cents).
    //   Line 11: 2.9% of the full $203,170 = $5,891.93 exactly (589,193
    //   cents, since 203,170 x 29 = 5,891,930, evenly divisible by 10).
    //   Combined cents: 2,287,800 + 589,193 = 2,876,993; (2,876,993 + 50) /
    //   100 = $28,770 self-employment tax.
    //   Half-SE deduction: (28,770 + 0) / 2 = $14,385 (28,770 is even, so
    //   the App's rounding adds nothing).
    //
    // Combined Schedule 2 line 4 (both owners' Schedule SE lines 12 summed):
    //   $0 (taxpayer) + $28,770 (spouse) = $28,770. If the App wrongly
    //   shared one $184,500 wage base across the household (subtracting the
    //   taxpayer's $150,000 wages from the spouse's own line 9), the
    //   spouse's line 9 would fall to $34,500 and the combined tax would be
    //   far smaller than $28,770; the actual combined value below confirms
    //   each owner keeps an independent wage base.
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, wage(1, Owner::Taxpayer, 15_000_000, 2_000_000));
    let mut spouse_business = income(cash_business(220_000));
    spouse_business.owner = Owner::Spouse;
    add(&mut d, spouse_business);
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 28_770);
    assert_eq!(value(&r, "adjustments"), 14_385);
}

// Boundary (4): a church employee W-2 (unit C10a, lines 5a-5b) and a
// separate Schedule C business combine on line 6, which is then subject to
// the shared $184,500 wage base together with a third, ordinary W-2's real
// Social Security wages.
#[test]
fn church_income_and_business_profit_share_line_6() {
    // Church W-2: box 1 $20,000, boxes 3/5 $0 (the church elected out of
    // Social Security/Medicare, as unit C10a requires).
    //   Line 5a: $20,000. Line 5b: 20,000 x 92.35% = $18,470.00 exactly
    //   (20,000 x 9,235 = 184,700,000, evenly divisible by 10,000), above
    //   the $100 floor.
    // Schedule C business: $30,000 net profit (cash_business, no expenses).
    //   Line 2/3: $30,000. Line 4a-4c: 30,000 x 92.35% = $27,705.00 exactly
    //   (30,000 x 9,235 = 277,050,000, evenly divisible by 10,000).
    // Line 6 = 4c + 5b = 27,705 + 18,470 = $46,175 (the church and business
    // amounts share one combined line, per unit C10a).
    //
    // A second, ordinary W-2: box 1/box 3 $170,000 (real Social Security
    // wages, unrelated to the church wages). Line 7: $184,500. Line 8d:
    // $170,000 (only the ordinary W-2's box 3; the church W-2 contributes
    // $0 to box 3). Line 9: 184,500 - 170,000 = $14,500.
    //   Line 10: 12.4% of min(46,175, 14,500) = 14,500 x 12.4% = $1,798.00
    //   exactly (179,800 cents).
    //   Line 11: 2.9% of the full combined $46,175 (never capped by the
    //   wage base) = $1,339.075, truncated to $1,339.07 (133,907 cents).
    //   Combined cents: 179,800 + 133,907 = 313,707; (313,707 + 50) / 100 =
    //   $3,137 self-employment tax.
    //   Half-SE deduction: (3,137 + 1) / 2 = $1,569.
    let mut d = empty_return(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Wage {
            data: FederalWage {
                abroad: 0,
                deferred_pension: 0,
                inmate_pay: 0,
                disability: DisabilityIncome::None,
                wages: 2_000_000,
                withholding: 0,
                social_wages: 0,
                social_tips: 0,
                employee_tips: None,
                work_deductions: FederalWorkDeductions::NONE,
                social_tax: 0,
                medicare_wages: 0,
                medicare_tax: 0,
                uncollected_tax: None,
                medicaid_waiver: 0,
                statutory_business: None,
                paying_scorporation: None,
                payroll: crate::W2Payroll::SocialSecurity,
                coverage: crate::W2Coverage::ChurchEmployee,
                care_benefits: 0,
                statutory_employee: false,
                retirement_plan: false,
                other_box12: Vec::new(),
            },
        }),
    );
    add(&mut d, income(cash_business(30_000)));
    add(&mut d, wage(3, Owner::Taxpayer, 17_000_000, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 3_137);
    assert_eq!(value(&r, "adjustments"), 1_569);
}

#[test]
fn schedule_2_part_ii_taxes_match_hand_computation() {
    // Independent full-return fixture for Schedule 2's typical Part II mix
    // (matrix row 15): a single filer with self-employment, an early IRA
    // distribution and a nonmedical HSA withdrawal, alongside Additional
    // Medicare tax split between wages and self-employment earnings.
    //
    // Single filer born 1979 (age 47 in 2026, not 65, no age/blind addition).
    //
    // W-2: box 1 $215,000, box 2 $45,000. `wage()` derives box 3 Social
    // Security wages capped at the $184,500 2026 wage base, box 5 Medicare
    // wages equal to the full $215,000, and box 6 Medicare tax as 1.45% of
    // box 5 plus the employer's 0.9% Additional Medicare withholding on the
    // $15,000 above the $200,000 threshold: 215,000 x 1.45% = $3,117.50,
    // plus 15,000 x 0.9% = $135.00, for $3,252.50 total (325,250 cents).
    //
    // Schedule C: receipts $40,000, one Supplies expense $4,000. Net profit
    // (line 31) = 40,000 - 4,000 = $36,000, material participation, no
    // employees (no W-2 wages, so QBI's wage/UBIA limit is $0), fully at
    // risk.
    //
    // Schedule SE (i1040sse, ordinary nonfarm method):
    //   Line 2/3: $36,000. Line 4a-4c: 36,000 x 92.35% = $33,246.00 exactly
    //   (36,000 x 9,235 = 332,460,000, evenly divisible by 10,000).
    //   Line 6: $33,246 (no church income). Line 7: $184,500 (2026 wage
    //   base). Line 8d: this W-2's box 3, $184,500 (capped, since box 1 is
    //   above the wage base). Line 9: 184,500 - 184,500 = $0.
    //   Line 10 (12.4% of min(33,246, 0)): $0.00.
    //   Line 11 (2.9% of the full $33,246): 33,246 x 2.9% = $964.134,
    //   truncated by the App's integer cents arithmetic to $964.13 (96,413
    //   cents).
    //   Combined cents: 0 + 96,413 = 96,413; (96,413 + 50) / 100 = $964
    //   self-employment tax (Schedule 2 line 4).
    //   Half-SE deduction (Schedule 1 line 15): (964 + 1) / 2 = $482.
    //
    // Form 5329 Part I (i5329): an early traditional IRA distribution, code
    // 1, age 47, no exception -- fully taxable, 10% of $8,000 = $800
    // (Schedule 2 line 8).
    //
    // Form 8889 Part II (i8889): a nonmedical HSA withdrawal of $1,500
    // (self-only coverage, age 47, not disabled, no contributions this
    // year). The full $1,500 enters Schedule 1 line 8f other income, and
    // 20% of $1,500 = $300 additional tax (Schedule 2 line 13c).
    //
    // Form 8959 (i8959; Additional Medicare tax, single $200,000
    // threshold):
    //   Medicare wages: $215,000 (box 5, uncapped). Schedule SE line 6 (the
    //   self-employment earnings base): $33,246.
    //   Part I (wages/RRTA, Schedule 2 line 17b): the threshold applies to
    //   wages first: 0.9% x (215,000 - 200,000) = 0.9% x $15,000 = $135.00.
    //   No wage-threshold headroom remains for Part II.
    //   Part II (self-employment, Schedule 2 line 11): 0.9% x (33,246 - 0)
    //   = 0.9% x $33,246 = $299.214, rounds to $299.
    //   Combined 0.9% x (215,000 + 33,246 - 200,000) = 0.9% x $48,246 =
    //   $434.214, rounds to $434 = $135 + $299 (no rounding mismatch).
    //
    // Schedule 2 Part II total (line 21, Form 1040 line 23):
    //   SE tax $964 + early-distribution tax $800 + Additional Medicare
    //   ($135 + $299 = $434) + HSA additional tax $300 = $2,498. Lines 9,
    //   12 and 18 (housing-credit recapture, section 965, other excise
    //   taxes) are all $0 in this fixture.
    //
    // Form 8995 QBI (i8995): this business's income before the wage/UBIA
    // limit is 36,000 - 482 (attributable half-SE) = $35,518, and it pays
    // no wages and owns no depreciable property, so its wage/UBIA limit is
    // $0. Taxable income before QBI (before_qbi) is AGI $260,018 -
    // standard deduction $16,100 = $243,918, above the 2026 single
    // $201,750 threshold and inside the $75,000 phase-in range (not a
    // specified-service business, so the phase-in reduces the tentative
    // deduction toward the wage/UBIA limit rather than reducing QBI income
    // itself):
    //   phased = 243,918 - 201,750 = $42,168 (of the $75,000 range).
    //   Tentative 20%: 35,518 x 20% = $7,103.60, rounds to $7,104.
    //   Wage/UBIA limit: $0 (no wages, no property).
    //   Reduction: 7,104 x 42,168 / 75,000 = $3,994 (rounded).
    //   QBI deduction (line 13b): 7,104 - 3,994 = $3,110, far under the
    //   20%-of-taxable-income cap (20% x $243,918 = $48,784).
    //
    // Form 1040 (i1040gi):
    //   Line 1a wages: $215,000. Line 4a/4b IRA gross/taxable: $8,000.
    //   Line 8 additional income (Schedule 1: business $36,000 + HSA
    //   $1,500): $37,500.
    //   Line 9 total income: 215,000 + 8,000 + 37,500 = $260,500.
    //   Line 10 adjustments (half-SE only): $482.
    //   Line 11 AGI: 260,500 - 482 = $260,018.
    //   Line 12e standard deduction (single, not 65): $16,100.
    //   Line 13b QBI deduction: $3,110.
    //   Line 15 taxable income: 260,018 - 16,100 - 3,110 = $240,808,
    //   between the 2026 single $201,775 and $256,225 rate-schedule
    //   breakpoints (24% and 32%), so the direct rate schedule applies
    //   (above the $100,000 Tax Table ceiling):
    //     10% x 12,400 + 12% x (50,400-12,400) + 22% x (105,700-50,400)
    //     + 24% x (201,775-105,700) + 32% x (240,808-201,775)
    //     = 1,240.00 + 4,560.00 + 12,166.00 + 23,058.00 + 12,490.56
    //     = $53,514.56, rounds to $53,515 (line 16).
    //   Line 17 AMT: $0 (no preference items in this fixture; a private-
    //   activity-interest AMT case is verified separately in
    //   amt_from_private_activity_interest_reaches_line_2).
    //   Line 18/22: $53,515 (no credits).
    //   Line 23 additional taxes: $2,498 (Schedule 2 line 21 above).
    //   Line 24a total tax: 53,515 + 2,498 = $56,013.
    //   Line 25a W-2 withholding: $45,000. Line 25c Form 8959 withholding
    //   (Medicare tax withheld exceeds 1.45% of Medicare wages by the
    //   employer's Additional Medicare withholding): $135.
    //   Line 33 total payments: 45,000 + 135 = $45,135.
    //   Line 37 balance due: 56,013 - 45,135 = $10,878.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19790615;
    d.income.clear();
    add(&mut d, wage(1, Owner::Taxpayer, 21_500_000, 4_500_000));
    add(
        &mut d,
        income(FederalIncome::Business {
            data: FederalBusiness {
                activity: BusinessActivity::Ordinary {
                    returns: 0,
                    receipts: 4_000_000,
                    accounting: BusinessAccounting::Cash,
                },
                expenses: vec![ordinary_business_expense(
                    BusinessExpenseCategory::Supplies,
                    400_000,
                )],
                ..support::builders::survey::survey_cash_business_data()
            },
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Retirement {
            data: FederalRetirement {
                box_9b: None,
                box_10: 0,
                disability: DisabilityIncome::None,
                kind: RetirementKind::Ira,
                disability_wages: crate::DisabilityWages::None,
                gross: 800_000,
                taxable: RetirementTaxable::Payer { amount: 800_000 },
                withholding: 0,
                box7: crate::DistributionCodes::CODE_1.0,
                ordinary_distribution: RetirementTreatment::Early,
                disaster: crate::DisasterDistribution::No,
                after_death: None,
            },
        }),
    );
    let h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 0);
    add_hsa_distribution(
        &mut d,
        Owner::Taxpayer,
        0,
        vec![MedicalAccountWithdrawal { date: 20260615, amount: 150_000 }],
        None,
        None,
    );
    d.adjustments.as_mut().unwrap().hsa.push(h);
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 964);
    assert_eq!(value(&r, "adjustments"), 482);
    assert_eq!(value(&r, "early_distribution_tax"), 800);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 1_500);
    assert_eq!(value(&r, "hsa_additional_tax"), 300);
    assert_eq!(value(&r, "additional_medicare_wages"), 135);
    assert_eq!(value(&r, "additional_medicare_self_employment"), 299);
    assert_eq!(value(&r, "additional_medicare_tax"), 434);
    assert_eq!(value(&r, "agi"), 260_018);
    assert_eq!(value(&r, "qbi_deduction"), 3_110);
    assert_eq!(value(&r, "taxable_income"), 240_808);

    assert_eq!(return_line(&r, "1a"), 21_500_000);
    assert_eq!(return_line(&r, "4b"), 800_000);
    assert_eq!(return_line(&r, "8"), 3_750_000);
    assert_eq!(return_line(&r, "9"), 26_050_000);
    assert_eq!(return_line(&r, "10"), 48_200);
    assert_eq!(return_line(&r, "11a / 11b"), 26_001_800);
    assert_eq!(return_line(&r, "12e"), 1_610_000);
    assert_eq!(return_line(&r, "13b"), 311_000);
    assert_eq!(return_line(&r, "15"), 24_080_800);
    assert_eq!(return_line(&r, "16"), 5_351_500);
    assert_eq!(return_line(&r, "17"), 0);
    assert_eq!(return_line(&r, "23"), 249_800);
    assert_eq!(return_line(&r, "24a"), 5_601_300);
    assert_eq!(return_line(&r, "25a"), 4_500_000);
    assert_eq!(return_line(&r, "25c"), 13_500);
    assert_eq!(return_line(&r, "33"), 4_513_500);
    assert_eq!(return_line(&r, "37"), 1_087_800);
    assert_eq!(r.tax, Some(5_601_300));
    assert_eq!(r.payments, Some(4_513_500));
    assert_eq!(r.balance, Some(1_087_800));
    assert_eq!(restart(d.clone()).draft, d);
}

// Boundary (1): a large specified private-activity bond interest amount
// (Form 6251 line 2g) creates AMT even though nothing else in the return
// does, reaching Schedule 2 line 2.
#[test]
fn amt_from_private_activity_interest_reaches_line_2() {
    // Single filer, W-2 wages $150,000 withholding $30,000 (`wage()`), plus
    // $200,000 of tax-exempt interest, all of it specified private-activity
    // bond interest.
    //
    // Regular return: AGI = $150,000 (tax-exempt interest is not taxable
    // income). Standard deduction (single, not 65): $16,100. Taxable
    // income (line 15) = 150,000 - 16,100 = $133,900, above the $100,000
    // Tax Table ceiling, between the 2026 single $105,700/$201,775 rate
    // breakpoints (22%/24%):
    //   10% x 12,400 + 12% x (50,400-12,400) + 22% x (105,700-50,400)
    //   + 24% x (133,900-105,700)
    //   = 1,240.00 + 4,560.00 + 12,166.00 + 6,768.00 = $24,734.00 exactly
    //   (line 16/regular tax).
    //
    // Form 6251 (i6251; 2026 single exemption $90,100, phase-out starting
    // at $500,000, 26%/28% breakpoint $244,500, as the App records them):
    //   Line 1 taxable income: $133,900. Line 2a standard-deduction
    //   add-back: $16,100 (a standard-deduction filer's line 1 + 2a nets
    //   back to AGI). Line 2g private-activity bond interest: $200,000.
    //   Line 4 AMTI: 133,900 + 16,100 + 200,000 = $350,000 (well under the
    //   $500,000 phase-out, so the full exemption applies).
    //   Line 5 exemption: $90,100. Line 6: 350,000 - 90,100 = $259,900.
    //   Line 7 tentative minimum tax: the $244,500 26%/28% breakpoint
    //   splits this into 244,500 x 26% + (259,900-244,500) x 28%
    //   = $63,570.00 + $4,312.00 = $67,882.00 exactly.
    //   Line 9 AMT foreign tax credit: $0. Line 10: $67,882.
    //   Line 11 comparison (this return's regular tax for AMT purposes, no
    //   Schedule J/8615/foreign averaging): $24,734.
    //   AMT (Schedule 2 line 2): 67,882 - 24,734 = $43,148.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 15_000_000, 3_000_000);
    add(
        &mut d,
        income(FederalIncome::Interest { data: native_exempt_interest(20_000_000, 20_000_000) }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_income"), 133_900);
    assert_eq!(value(&r, "regular_income_tax"), 24_734);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 350_000);
    assert_eq!(value(&r, "alternative_minimum_tax"), 43_148);
    assert_eq!(return_line(&r, "17"), 4_314_800);
}

// Boundary (2): wages exactly at the single $200,000 Additional Medicare
// threshold leave no headroom for the self-employment part (i8959 Parts
// I-II apply the threshold to wages first).
#[test]
fn additional_medicare_splits_at_the_threshold() {
    // W-2 Medicare wages (box 5): $200,000 exactly, no withholding. A
    // sole proprietorship with $10,828 net profit (receipts only, no
    // expenses) produces Schedule SE line 6 of exactly $10,000:
    // 10,828 x 92.35% = $10,000.158, and the App's fraction() rounds
    // .158 down to $10,000.
    //
    // Part I (wages, Schedule 2 line 17b): 0.9% x (200,000 - 200,000) =
    // $0; the full threshold is used by wages alone. Part II
    // (self-employment, Schedule 2 line 11): no threshold headroom
    // remains, so 0.9% x (10,000 - 0) = $90.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(1, Owner::Taxpayer, 20_000_000, 0));
    add(&mut d, income(cash_business(10_828)));
    let r = supported(&d);
    assert_eq!(value(&r, "additional_medicare_wages"), 0);
    assert_eq!(value(&r, "additional_medicare_self_employment"), 90);
    assert_eq!(value(&r, "additional_medicare_tax"), 90);
}

// Each uncomputed added-tax category retains its identified source, without
// claiming that the source history is computed or that filing is prohibited.
#[test]
fn schedule_2_unmodeled_lines_stay_refused() {
    let d = complete(FederalStatus::Single);
    for kind in RareAdditionalTax::ALL {
        let (_, _, lines, _, _) = kind.refusal_info();
        assert_situation_refused(&d, OtherTax::OtherAdditionalTaxes { kind }, &[lines]);
    }
}

// C-fixture (matrix row 36, Form 8959): a single filer with wages exactly at
// the 2026 $200,000 Additional Medicare threshold, worked by hand from i8959
// Parts I and IV. Boundary: wages exactly at the threshold.
#[test]
fn single_wages_exactly_at_the_200000_additional_medicare_threshold() {
    // i8959 Part I, single $200,000 threshold: Medicare wages (box 5)
    // exactly $200,000 leave no excess: 0.9% x (200,000 - 200,000) = $0.
    // The `wage()` fixture helper models an employer that itself withholds
    // an extra 0.9% only on Medicare wages above $200,000 (the payroll
    // threshold IRC 3102(f) applies to every employer regardless of the
    // employee's actual filing status); at exactly $200,000 that extra
    // withholding is also $0, so box 6 is exactly 1.45% x $200,000 and
    // Form 8959 Part IV's reconciliation (box 6 minus 1.45% of box 5) is
    // $0 -- no credit on Schedule 2/Form 1040 line 25c.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 20_000_000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "additional_medicare_wages"), 0);
    assert_eq!(value(&r, "additional_medicare_tax"), 0);
    assert_eq!(return_line(&r, "25c"), 0);

    // One thousand dollars above the threshold: 0.9% x $1,000 = $9.00 owed
    // (Schedule 2 line 17b). Because the single filer's own $200,000
    // threshold matches the employer's flat $200,000 payroll-withholding
    // line exactly, the employer withholds the same $9.00 extra on this
    // wage (100,000 cents x 0.9% = 900 cents = $9.00 employer withholding,
    // box 6 above 1.45% of box 5), so Form 8959 line 24 credits $9.00 on
    // Form 1040 line 25c -- the amount owed and the amount already
    // withheld toward it are equal for a single filer's wage-only income.
    d.income[0] = wage(1, Owner::Taxpayer, 20_100_000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "additional_medicare_wages"), 9);
    assert_eq!(value(&r, "additional_medicare_tax"), 9);
    assert_eq!(return_line(&r, "25c"), 900);
}

// Boundary: the other two Form 8959 filing-status thresholds ($250,000
// joint, $125,000 separate) worked by hand from i8959 Part I, each exactly
// at its own line 7 amount.
#[test]
fn additional_medicare_threshold_exactly_at_250000_joint_and_125000_separate() {
    // Joint $250,000 threshold, Medicare wages exactly $250,000: 0.9% x
    // (250,000 - 250,000) = $0 owed. The employer's own withholding line is
    // still the flat $200,000 payroll threshold (IRC 3102(f) does not know
    // the employee's filing status), so it withholds an extra 0.9% on the
    // $50,000 above $200,000: 50,000 x 0.9% = $450.00 (calculated in cents
    // by the `wage()` helper: 5,000,000 x 9 / 1,000 = 45,000 cents). Box 6
    // is $4,075.00 (1.45% x $250,000 = $3,625.00, plus the $450.00 extra
    // withholding); box 6 minus 1.45% of box 5 = $450.00, so Form 8959 Part
    // IV credits $450.00 on Form 1040 line 25c even though line 11/17b owe
    // nothing this year.
    let mut joint = complete(FederalStatus::Joint);
    joint.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 0);
    let r = supported(&joint);
    assert_eq!(value(&r, "additional_medicare_wages"), 0);
    assert_eq!(value(&r, "additional_medicare_tax"), 0);
    assert_eq!(return_line(&r, "25c"), 45_000);

    // Separate $125,000 threshold, Medicare wages exactly $125,000: 0.9% x
    // (125,000 - 125,000) = $0 owed. $125,000 is under the employer's own
    // $200,000 withholding line, so box 6 is exactly 1.45% x $125,000 with
    // no extra withholding, and line 25c is $0.
    let mut separate = complete(FederalStatus::Separate);
    separate.income[0] = wage(1, Owner::Taxpayer, 12_500_000, 0);
    let r = supported(&separate);
    assert_eq!(value(&r, "additional_medicare_wages"), 0);
    assert_eq!(value(&r, "additional_medicare_tax"), 0);
    assert_eq!(return_line(&r, "25c"), 0);
}

// C-fixture (matrix row 36, Form 8959): a joint return with one high W-2
// and self-employment income, worked by hand from i8959 Parts I, II and IV.
// Boundary: the Part II self-employment reduction by wages, and the
// withheld amount on line 24 reaching Form 1040 line 25c.
#[test]
fn joint_self_employment_reduces_the_remaining_threshold_after_wages_and_reconciles_withholding() {
    // W-2 (taxpayer): Medicare wages (box 5) $230,000, no other box 12/13
    // facts. `wage()`'s employer withholds an extra 0.9% on the $30,000
    // above its own flat $200,000 payroll line: box 6 = 1.45% x $230,000
    // + 0.9% x $30,000 = $3,335.00 + $270.00 = $3,605.00 (in cents:
    // 23,000,000 x 145 / 10,000 = 333,500, plus (23,000,000 - 20,000,000)
    // x 9 / 1,000 = 27,000; total 360,500 cents).
    //
    // Schedule C: receipts $40,000, no expenses, net profit $40,000.
    // Schedule SE (i1040sse) line 4a-4c: 40,000 x 92.35% = $36,940.00
    // exactly (40,000 x 9,235 = 369,400,000, evenly divisible by 10,000).
    // Line 6 (no church income): $36,940 -- this is the self-employment
    // base Form 8959 Part II uses, unreduced by the wage-base coordination
    // that only applies to the 12.4% Social Security part of Schedule SE
    // tax itself.
    //
    // Form 8959 (joint $250,000 threshold):
    //   Part I (wages, Schedule 2 line 17b) applies the threshold to wages
    //   first: wages $230,000 are under $250,000, so the wages part is $0,
    //   and $250,000 - $230,000 = $20,000 of threshold remains for Part II.
    //   Part II (self-employment, Schedule 2 line 11): ($36,940 - $20,000)
    //   x 0.9% = $16,940 x 0.9% = $152.46, rounds to $152 (the App's
    //   `fraction` rounds .46 down: 16,940 x 9 = 152,460; (152,460 + 500) /
    //   1,000 = 152 by integer division).
    //   Combined check: 0.9% x ($230,000 + $36,940 - $250,000) = 0.9% x
    //   $16,940 = $152 -- matches Part I ($0) + Part II ($152) exactly.
    //
    // Form 8959 Part IV reconciliation (line 24, credited on Form 1040 line
    // 25c): box 6 $3,605.00 minus 1.45% of box 5 ($3,335.00) = $270.00,
    // the employer's own extra withholding on wages above its $200,000
    // payroll line. This is unrelated to the $250,000 joint liability
    // threshold and is credited in full regardless.
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 23_000_000, 0);
    add(&mut d, income(cash_business(40_000)));
    let r = supported(&d);
    assert_eq!(value(&r, "additional_medicare_wages"), 0);
    assert_eq!(value(&r, "additional_medicare_self_employment"), 152);
    assert_eq!(value(&r, "additional_medicare_tax"), 152);
    assert_eq!(return_line(&r, "25c"), 27_000);
}

// The 2026 Form 8959 gives RRTA its own Part II threshold, separately from
// ordinary wages in Part I and self-employment in Part IV.
#[test]
fn railroad_w2_computes_8959_part_ii_separately() {
    // $260,000 RRTA compensation exceeds the single $200,000 threshold by
    // $60,000: Part II tax is $540 while ordinary Part I tax stays zero.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 26_000_000, 0);
    let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
    data.social_wages = 0;
    data.social_tax = 0;
    data.medicare_wages = 0;
    data.medicare_tax = 0;
    data.payroll = W2Payroll::Railroad {
        compensation: 26_000_000,
        tier_1: 1_143_900,
        tier_2: 0,
        medicare: 377_000,
        additional_medicare: 54_000,
    };
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 260_000);
    assert_eq!(value(&r, "additional_medicare_payroll_wages"), 0);
    assert_eq!(value(&r, "additional_medicare_rrta_tax"), 540);
}

// Form 8960's printed lines, whole dollars, in the order the return emits them.
fn form_8960(r: &FederalResult) -> Vec<(alloc::string::String, i64)> {
    r.lines
        .iter()
        .filter(|l| l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8960))
        .map(|l| (l.form_line.as_ref().unwrap().line.clone(), l.amount / 100))
        .collect()
}
fn form_8960_line(r: &FederalResult, line: &str) -> i64 {
    form_8960(r)
        .into_iter()
        .find(|(l, _)| l == line)
        .unwrap_or_else(|| panic!("Form 8960 line {line} missing: {:?}", form_8960(r)))
        .1
}
fn portfolio_dividend(ordinary: i64, qualified: i64, capital: i64) -> FederalIncome {
    FederalIncome::Dividend {
        data: native_dividend(ordinary * 100, qualified * 100, capital * 100),
    }
}

#[test]
fn form_8960_hand_worked_return_prints_every_line() {
    // Single; wages $300,000; interest $30,000 with a $200 early-withdrawal
    // penalty; 1099-DIV box 1a $20,000 (box 1b $15,000) and box 2a $4,000;
    // one rental, rent $30,000 less $5,000 expenses and $10,000 depreciation
    // (275,000 x 24/660) = $15,000; itemized: $20,000 state income tax
    // allocated by the current return, $3,000 investment interest.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
    add(&mut d, income(savings_income(30_000, 200)));
    add(&mut d, income(portfolio_dividend(20_000, 15_000, 4_000)));
    add(&mut d, income(FederalIncome::Rental { data: rental(30_000, 5_000) }));
    investment_borrowing(&mut d, 3_000, 0, 0);
    itemized(d.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
        paid: 2_000_000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    let r = supported(&d);
    // AGI = 300,000 + 30,000 + 20,000 + 4,000 + 15,000 - 200 = 368,800.
    assert_eq!(value(&r, "agi"), 368_800);
    // Line 8 = 30,000 + 20,000 + 15,000 (4c) + 4,000 (5d) = 69,000.
    // Form 4952: investment income 30,000 + (20,000 - 15,000) = 35,000 covers
    // the $3,000 interest (9a) even after the whole state tax. SALT cap
    // $40,400 below $505,000 MAGI keeps the $20,000; its investment share
    // 20,000 x 69,000 / 368,800 = 3,741.87 -> 3,742 (9b). Itemized 23,000
    // exceeds the $16,100 standard deduction and the 37 percent bracket
    // (section 68) is not reached. Line 10 is the $200 penalty; line 11 =
    // 3,000 + 3,742 + 200 = 6,942; line 12 = 62,058. Line 15 = 368,800 -
    // 200,000 = 168,800; line 17 = 62,058 x 3.8% = 2,358.20 -> 2,358.
    let expected: Vec<(&str, i64)> = vec![
        ("1", 30_000),
        ("2", 20_000),
        ("3", 0),
        ("4a", 15_000),
        ("4b", 0),
        ("4c", 15_000),
        ("5a", 4_000),
        ("5b", 0),
        ("5d", 4_000),
        ("7", 0),
        ("8", 69_000),
        ("9a", 3_000),
        ("9b", 3_742),
        ("9c", 0),
        ("9d", 6_742),
        ("10", 200),
        ("11", 6_942),
        ("12", 62_058),
        ("13", 368_800),
        ("14", 200_000),
        ("15", 168_800),
        ("16", 62_058),
        ("17", 2_358),
    ];
    let actual = form_8960(&r);
    assert_eq!(actual.iter().map(|(l, a)| (l.as_str(), *a)).collect::<Vec<_>>(), expected);
    assert_eq!(value(&r, "net_investment_income_tax"), 2_358);
    // The review shows the nonzero lines under Form 8960, with line 17
    // matching Schedule 2 line 6.
    let review = crate::draft_return(&r);
    assert!(review.iter().any(|row| status_text(&row.label)
        == "Form 8960 17 — Form 8960 net investment income tax"
        && row.amount == 235_800));
    assert!(
        review
            .iter()
            .any(|row| status_text(&row.label).starts_with("Schedule 2 6 — ")
                && row.amount == 235_800)
    );
}

#[test]
fn form_8960_threshold_boundary_for_every_status() {
    // Modified AGI exactly at the threshold owes nothing and files no Form
    // 8960; $100 above owes 100 x 3.8% = 3.80 -> 4. A surviving spouse uses
    // the $250,000 joint threshold (section 1411(b)), not Form 8959's $200,000.
    for (status, threshold) in [
        (FederalStatus::Single, 200_000),
        (FederalStatus::Head, 200_000),
        (FederalStatus::Joint, 250_000),
        (FederalStatus::Separate, 125_000),
        (FederalStatus::Surviving, 250_000),
    ] {
        for (over, tax) in [(0, 0), (100, 4)] {
            let mut d = complete(status);
            if status == FederalStatus::Surviving {
                d.setup.as_mut().unwrap().marital_state =
                    MaritalState::WidowedPriorYear { could_file_joint: true };
                add_child(&mut d);
            }
            if status == FederalStatus::Head {
                add_child(&mut d);
            }
            d.income[0] = wage(1, Owner::Taxpayer, (threshold - 10_000) * 100, 0);
            add(&mut d, income(savings_income(10_000 + over, 0)));
            let r = supported(&d);
            assert_eq!(value(&r, "net_investment_income_tax"), tax, "{status:?} {over}");
            if over == 0 {
                assert!(form_8960(&r).is_empty(), "{status:?}");
            } else {
                assert_eq!(form_8960_line(&r, "14"), threshold, "{status:?}");
                assert_eq!(form_8960_line(&r, "15"), 100, "{status:?}");
                assert_eq!(form_8960_line(&r, "16"), 100, "{status:?}");
            }
        }
    }
}

#[test]
fn form_8960_line_16_takes_the_smaller_of_income_and_excess() {
    // Single, interest $10,000 (line 12). Wages 199,999 / 200,000 / 200,001
    // put line 15 at 9,999 / 10,000 / 10,001; line 16 is 9,999 / 10,000 /
    // 10,000, taxed at 3.8%: 379.96 -> 380, 380, 380.
    for (wages, excess, smaller) in
        [(199_999, 9_999, 9_999), (200_000, 10_000, 10_000), (200_001, 10_001, 10_000)]
    {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
        add(&mut d, income(savings_income(10_000, 0)));
        let r = supported(&d);
        assert_eq!(form_8960_line(&r, "12"), 10_000);
        assert_eq!(form_8960_line(&r, "15"), excess);
        assert_eq!(form_8960_line(&r, "16"), smaller);
        assert_eq!(form_8960_line(&r, "17"), 380);
    }
}

#[test]
fn form_8960_real_estate_professional_safe_harbor_moves_rentals_to_line_4b() {
    // Profit: wages $250,000, interest $5,000, rental net $15,000; MAGI
    // 270,000, line 15 = 70,000. Without the 500-hour safe harbor the rent
    // stays investment income: line 12 = 20,000, tax 760. With it line 4b
    // is -15,000: line 12 = 5,000, tax 190.
    // Loss: rent 0, expenses 30,000 and depreciation 10,000 = -40,000,
    // nonpassive for the professional; MAGI 215,000, line 15 = 15,000.
    // Without the safe harbor line 8 = 5,000 - 40,000 = -35,000 and line 12
    // is floored at zero (no tax); with it line 4b adds the loss back:
    // line 12 = 5,000, line 16 = 5,000, tax 190.
    for (rent, expenses, safe_harbor, line_4b, line_8, line_12, tax) in [
        (30_000, 5_000, Answer::No, 0, 20_000, 20_000, 760),
        (30_000, 5_000, Answer::Yes, -15_000, 5_000, 5_000, 190),
        (0, 30_000, Answer::No, 0, -35_000, 0, 0),
        (0, 30_000, Answer::Yes, 40_000, 5_000, 5_000, 190),
    ] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 0);
        add(&mut d, income(savings_income(5_000, 0)));
        let mut data = rental(rent, expenses);
        data.participation = crate::RentalParticipation::Professional;
        add(&mut d, income(FederalIncome::Rental { data }));
        make_professional(&mut d, Owner::Taxpayer, safe_harbor == Answer::Yes);
        let r = supported(&d);
        let rental_net = rent - expenses - 10_000;
        assert_eq!(value(&r, "agi"), 255_000 + rental_net);
        assert_eq!(form_8960_line(&r, "4a"), rental_net);
        assert_eq!(form_8960_line(&r, "4b"), line_4b);
        assert_eq!(form_8960_line(&r, "8"), line_8);
        assert_eq!(form_8960_line(&r, "12"), line_12);
        assert_eq!(value(&r, "net_investment_income_tax"), tax);
    }
}

#[test]
fn form_8959_withholding_lines_foot_on_the_printed_dollars() {
    // D-2026-10-19-12, ATS TY2026 scenario 14: box 5 $35,000.00 and box 6 $508.00. Line 21 =
    // 1.45% x 35,000 = 507.50, printed 508, so line 22 = 508 - 508 = 0 and Form 1040 line 25c
    // stays blank. Exact cents would leave $0.50, rounded to $1 on line 24.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_500_000, 0);
    let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
    data.medicare_wages = 3_500_000;
    data.medicare_tax = 50_800;
    let r = supported(&d);
    assert_eq!(return_line(&r, "25c"), 0);
}
