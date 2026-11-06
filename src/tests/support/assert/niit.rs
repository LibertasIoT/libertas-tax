use super::injured::assert_no_claim_native_money;
use crate::tests::*;

pub(in crate::tests) fn assert_k1_portfolio_owners(
    source: K1PortfolioNiitSource,
    owner: Owner,
    code_a: bool,
    final_deduction: bool,
) {
    assert_k1_portfolio_owners_context(source, owner, code_a, final_deduction, false);
}

pub(in crate::tests) fn assert_k1_portfolio_owners_context(
    source: K1PortfolioNiitSource,
    owner: Owner,
    code_a: bool,
    final_deduction: bool,
    extra_medicare: bool,
) {
    assert!(!extra_medicare || !final_deduction);
    let spouse_wages = if extra_medicare { 100_000 } else { 50_000 };
    let mut original = injured_return((200_000, 100_000), (spouse_wages, 30_000), 0, allocation());
    // A separate accepted interest source belongs exclusively to the other
    // spouse. The K1 has no operating profit/loss, PAL, EBL, property or carry.
    let mut other = income(savings_income(10_000, 0));
    other.owner = if owner == Owner::Taxpayer {
        crate::DocumentOwner::Spouses
    } else {
        crate::DocumentOwner::Yours
    };
    add(&mut original, other);
    let mut entry = k1_niit_portfolio_entry(source, code_a, final_deduction);
    entry.owner = crate::DocumentOwner::from(owner);
    add(&mut original, entry);
    let code_a_income = if code_a { 2_000 } else { 0 };
    let deduction = if final_deduction { 2_000 } else { 0 };
    let nii_deduction = if final_deduction { 500 } else { 0 };
    let expected_nii = 26_000 + code_a_income - nii_deduction;
    let expected_tax = if final_deduction {
        912
    } else if code_a {
        1_064
    } else {
        988
    };
    let source_tax = if final_deduction {
        554
    } else if code_a {
        684
    } else {
        608
    };
    // Final67e leaves MAGI274000: the actual joint24000 NIIT ceiling is
    // below NII25500. Tax912 still uses actual NII15500/10000 shares:
    // 912*15500/25500 rounds554; the other column receives358.
    let other_tax = if final_deduction { 358 } else { 380 };
    let mut observed = Vec::new();
    let mut expected = Vec::new();
    for reversed in [false, true] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            let mut draft = original.clone();
            draft.injured_spouse.as_mut().unwrap().injured = injured;
            if extra_medicare {
                // Retain the300000-wage coexistence case with the existing
                // bounded agreement: actual450 AdditionalMedicare is300
                // taxpayer/150spouse, independent of NIIT source ownership.
                draft.injured_spouse.as_mut().unwrap().allocations.push(InjuredAmount {
                    component: "additional_medicare_tax".into(),
                    injured: if injured == Owner::Taxpayer { 30_000 } else { 15_000 },
                });
            }
            if reversed {
                draft.income.reverse();
            }
            let result = supported(&draft);
            // Qualified2000 is a subset of ordinary dividends6000. The native
            // gross portfolio is10000interest+6000dividends, never18000.
            assert_eq!(return_line(&result, "2b") / 100, 20_000);
            assert_eq!(return_line(&result, "3b") / 100, 6_000);
            assert_eq!(return_line(&result, "3a") / 100, 2_000);
            assert_eq!(return_line(&result, "7a") / 100, 0);
            assert_eq!(return_line(&result, "8") / 100, code_a_income);
            assert_eq!(return_line(&result, "10") / 100, deduction);
            assert_eq!(value(&result, "agi"), 226_000 + spouse_wages + code_a_income - deduction);
            assert_eq!(value(&result, "form_8960_net_investment_income"), expected_nii);
            assert_eq!(value(&result, "net_investment_income_tax"), expected_tax);
            assert_eq!(
                value(&result, "additional_medicare_tax"),
                if extra_medicare { 450 } else { 0 }
            );
            if final_deduction {
                assert_eq!(value(&result, "estate_trust_67e_deduction"), 2_000);
            }
            assert_portfolio_ordinary_unchanged(&draft, &result);
            let held = owner == injured;
            let medicare = if !extra_medicare {
                [0, 0]
            } else if injured == Owner::Taxpayer {
                [300, 150]
            } else {
                [150, 300]
            };
            observed.push((
                reversed,
                injured,
                niit_optional_column(&result, "13b", "b"),
                niit_optional_column(&result, "13b", "c"),
                niit_optional_column(&result, "14", "b"),
                niit_optional_column(&result, "14", "c"),
                niit_optional_column(&result, "18", "a"),
                niit_optional_column(&result, "18", "b"),
                niit_optional_column(&result, "18", "c"),
            ));
            expected.push((
                reversed,
                injured,
                Some(if held { 16_000 + code_a_income } else { 10_000 }),
                Some(if held { 10_000 } else { 16_000 + code_a_income }),
                Some(if held { deduction } else { 0 }),
                Some(if held { 0 } else { deduction }),
                Some(expected_tax + if extra_medicare { 450 } else { 0 }),
                Some((if held { source_tax } else { other_tax }) + medicare[0]),
                Some((if held { other_tax } else { source_tax }) + medicare[1]),
            ));
        }
    }
    assert_eq!(
        observed, expected,
        "{source:?}, source owner {owner:?}, codeA={code_a}, final67e={final_deduction}"
    );
}

pub(in crate::tests) fn assert_niit_deduction_money(d: &FederalDraft) -> FederalResult {
    let r = supported(d);
    assert_eq!(return_line(&r, "1a"), 25_000_000);
    assert_eq!(return_line(&r, "2b"), 3_000_000);
    assert_eq!(value(&r, "agi"), 280_000);
    assert_eq!(value(&r, "taxable_income"), 279_000);
    assert_eq!(value(&r, "niit_itemized_deductions"), 1000);
    assert_eq!(value(&r, "form_8960_net_investment_income"), 29_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 1102);
    assert_eq!(value(&r, "additional_medicare_tax"), 0);
    assert_eq!(r.tax, Some(5_325_800));
    assert_eq!(r.refund, Some(1_674_200));
    assert_no_claim_native_money(d, &r);
    r
}

pub(in crate::tests) fn assert_niit_binding_money(d: &FederalDraft) -> FederalResult {
    assert_niit_binding_money_with_paid(d, 12000)
}

pub(in crate::tests) fn assert_niit_binding_money_with_paid(
    d: &FederalDraft,
    paid: i64,
) -> FederalResult {
    let r = supported(d);
    assert_eq!(return_line(&r, "1a"), 25_000_000);
    assert_eq!(return_line(&r, "2b"), 500_000);
    assert_eq!(return_line(&r, "3a"), 2_000_000);
    assert_eq!(return_line(&r, "3b"), 2_000_000);
    assert_eq!(value(&r, "agi"), 275_000);
    assert_eq!(value(&r, "taxable_income"), 270_000);
    assert_eq!(value(&r, "regular_income_tax"), 48_196);
    for (line, amount) in [
        ("1", paid),
        ("2", 0),
        ("3", paid),
        ("4a", 25000),
        ("4b", 20000),
        ("4c", 5000),
        ("4g", 0),
        ("4h", 5000),
        ("5", 0),
        ("6", 5000),
        ("7", paid - 5000),
        ("8", 5000),
    ] {
        assert_eq!(value(&r, &alloc::format!("investment_form_line{line}")), amount);
    }
    // Actual native ScheduleA component composition, not equality of totals.
    for component in [
        "medical_deduction",
        "taxes_deduction",
        "mortgage_interest_8e",
        "charity_deduction",
        "personal_casualty_deduction",
        "other_itemized_total",
    ] {
        assert_eq!(value(&r, component), 0, "{component}");
    }
    assert_eq!(value(&r, "interest_deduction"), 5000);
    assert_eq!(value(&r, "investment_interest_deduction"), 5000);
    assert_eq!(value(&r, "itemized_deduction_reduction"), 0);
    assert_eq!(value(&r, "itemized_deductions_allowed"), 5000);
    assert_eq!(return_line(&r, "12e"), 500_000);
    assert_eq!(value(&r, "niit_itemized_deductions"), 5000);
    assert_eq!(value(&r, "form_8960_net_investment_income"), 20000);
    assert_eq!(value(&r, "net_investment_income_tax"), 760);
    assert_eq!(value(&r, "additional_medicare_tax"), 0);
    assert_eq!(r.tax, Some(4_895_600));
    assert_eq!(r.refund, Some(2_104_400));
    assert_no_claim_native_money(d, &r);
    r
}

pub(in crate::tests) fn assert_niit_person_money(
    draft: &FederalDraft,
    source: NiitExpenseSource,
) -> FederalResult {
    if !matches!(source, NiitExpenseSource::IncomeTax) {
        return assert_niit_deduction_money(draft);
    }
    let r = supported(draft);
    assert_eq!(return_line(&r, "1a"), 25_000_000);
    assert_eq!(return_line(&r, "2b"), 3_000_000);
    assert_eq!(value(&r, "agi"), 280_000);
    assert_eq!(value(&r, "taxable_income"), 279_000);
    assert_eq!(value(&r, "regular_income_tax"), 52_156);
    assert_eq!(value(&r, "niit_income_tax_deduction"), 107);
    assert_eq!(value(&r, "niit_itemized_deductions"), 107);
    assert_eq!(value(&r, "form_8960_net_investment_income"), 29_893);
    assert_eq!(value(&r, "net_investment_income_tax"), 1136);
    assert_eq!(value(&r, "additional_medicare_tax"), 0);
    assert_eq!(r.tax, Some(5_329_200));
    assert_eq!(r.refund, Some(1_670_800));
    assert_no_claim_native_money(draft, &r);
    r
}

pub(in crate::tests) fn assert_niit_zero_money(
    d: &FederalDraft,
    deduction_dollars: i64,
) -> FederalResult {
    let r = supported(d);
    assert_eq!(return_line(&r, "1a"), 25_000_000);
    assert_eq!(return_line(&r, "2b"), 3_000_000);
    assert_eq!(value(&r, "agi"), 280_000);
    assert_eq!(value(&r, "taxable_income"), 280_000 - deduction_dollars);
    assert_eq!(value(&r, "regular_income_tax"), 52_396);
    assert_eq!(value(&r, "investment_interest_deduction"), deduction_dollars);
    assert_eq!(value(&r, "niit_itemized_deductions"), deduction_dollars);
    assert_eq!(value(&r, "form_8960_net_investment_income"), 30_000 - deduction_dollars);
    assert_eq!(value(&r, "net_investment_income_tax"), 1140);
    assert_eq!(value(&r, "additional_medicare_tax"), 0);
    assert_eq!(r.tax, Some(5_353_600));
    assert_eq!(r.refund, Some(1_646_400));
    assert_no_claim_native_money(d, &r);
    r
}

pub(in crate::tests) fn assert_retained_niit_money(d: &FederalDraft) -> FederalResult {
    // Exhaustive actual current source admission, not scalar exclusions==0:
    // ordinary Investment sales are all included, and no NUA/retirement,
    // K1/property/self-rental/estate-current exclusion source is present.
    assert!(d.income.iter().all(|entry| match &entry.income {
        FederalIncome::Wage { .. } | FederalIncome::Interest { .. } => true,
        FederalIncome::Sale { data } =>
            matches!(data.form.use_kind, SaleUse::Investment { wash_sale: 0 }),
        _ => false,
    }));
    let r = supported(d);
    // Actual2026 native money is asserted BEFORE the missing owner control.
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 5000);
    assert_eq!(value(&r, "niit_disposition_adjustment"), 5000);
    assert_eq!(value(&r, "niit_capital_carry_adjustment"), 5000);
    assert_eq!(value(&r, "form_8960_net_investment_income"), 12_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 456);
    assert_eq!(value(&r, "additional_medicare_tax"), 1350);
    assert_eq!(value(&r, "agi"), 407_000);
    assert_eq!(value(&r, "taxable_income"), 374_800);
    assert_eq!(return_line(&r, "16"), 7_514_800);
    assert_eq!(r.tax, Some(7_695_400));
    // The existing W2 helper also reports900 actual AdditionalMedicare
    // withholding above the200000 employee threshold: payments80900.
    assert_eq!(r.refund, Some(394_600));
    let mut plain = d.clone();
    plain.injured_spouse = None;
    plain.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
    let plain = supported(&plain);
    assert_eq!(plain.return_lines, r.return_lines);
    assert_eq!((plain.tax, plain.refund), (r.tax, r.refund));
    r
}

pub(in crate::tests) fn assert_current_nua_money(d: &FederalDraft) -> FederalResult {
    let r = supported(d);
    assert_eq!(return_line(&r, "1a"), 25_000_000);
    assert_eq!(return_line(&r, "2b"), 2_000_000);
    assert_eq!(value(&r, "capital_gain"), 95_000);
    assert_eq!(value(&r, "agi"), 365_000);
    assert_eq!(value(&r, "taxable_income"), 332_800);
    // Joint ordinary taxable income237800: 2480+9120+24332+6336;
    // the entire95000 long-term gain is taxed at15%, adding14250.
    assert_eq!(value(&r, "regular_income_tax"), 56_518);
    assert_eq!(value(&r, "niit_disposition_adjustment"), -75_000);
    assert_eq!(value(&r, "form_8960_net_investment_income"), 40_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 1520);
    assert_eq!(value(&r, "additional_medicare_tax"), 0);
    assert_eq!(r.tax, Some(5_803_800));
    assert_eq!(r.refund, Some(1_196_200));
    assert_no_claim_native_money(d, &r);
    r
}

pub(in crate::tests) fn assert_hobby_niit_money(d: &FederalDraft) -> FederalResult {
    let r = supported(d);
    assert_eq!(return_line(&r, "1a"), 25000000);
    assert_eq!(return_line(&r, "2b"), 2000000);
    assert_eq!(value(&r, "hobby_sales_income"), 4000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "additional_medicare_tax"), 0);
    assert_eq!(value(&r, "agi"), 274000);
    assert_eq!(value(&r, "taxable_income"), 241800);
    // The independently verified mixed fixture's234800 taxable/41548 regular
    // tax increases by7000 at24%:43228. NII24000 equals MFJ MAGI excess24000.
    assert_eq!(value(&r, "regular_income_tax"), 43228);
    assert_eq!(value(&r, "form_8960_net_investment_income"), 24000);
    assert_eq!(value(&r, "net_investment_income_tax"), 912);
    assert_eq!(r.tax, Some(4414000));
    assert_eq!(r.refund, Some(2586000));
    let mut plain = d.clone();
    plain.injured_spouse = None;
    let p = supported(&plain);
    assert_eq!(p.return_lines, r.return_lines);
    assert_eq!((p.tax, p.refund), (r.tax, r.refund));
    r
}

pub(in crate::tests) fn assert_hobby_owner_columns(
    d: &FederalDraft,
    owner: Owner,
) -> FederalResult {
    let r = assert_hobby_niit_money(d);
    let i = usize::from(d.injured_spouse.as_ref().unwrap().injured == Owner::Spouse);
    let mut other = [10000, 10000];
    let mut niit = [380, 380];
    let own = usize::from(owner == Owner::Spouse);
    other[own] += 4000;
    niit[own] = 532;
    assert_eq!(column(&r, "13b", "a"), 24000);
    assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [other[i], other[1 - i]]);
    assert_eq!(column(&r, "18", "a"), 912);
    assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], [niit[i], niit[1 - i]]);
    r
}

pub(in crate::tests) fn assert_mixed_niit_money(draft: &FederalDraft) -> FederalResult {
    let r = supported(draft);
    assert_eq!(return_line(&r, "1a"), 25_000_000);
    assert_eq!(return_line(&r, "2b"), 2_000_000);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 10_000);
    assert_eq!(value(&r, "niit_disposition_adjustment"), 0);
    assert_eq!(value(&r, "niit_capital_carry_adjustment"), 5000);
    assert_eq!(value(&r, "form_8960_net_investment_income"), 17_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 646);
    assert_eq!(value(&r, "agi"), 267_000);
    assert_eq!(value(&r, "taxable_income"), 234_800);
    assert_eq!(value(&r, "regular_income_tax"), 41_548);
    assert_eq!(r.tax, Some(4_219_400));
    assert_eq!(r.refund, Some(2_780_600));
    let mut plain = draft.clone();
    plain.injured_spouse = None;
    let ordinary = supported(&plain);
    assert_eq!(r.return_lines, ordinary.return_lines);
    assert_eq!((r.tax, r.refund), (ordinary.tax, ordinary.refund));
    r
}
