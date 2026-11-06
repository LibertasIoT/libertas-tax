use super::*;
use crate::tests::support::assert::injured_income::native_payroll_other_tax_sources::check;
use crate::tests::support::builders::payroll_taxes::{Source, fixture};
// R6 actual native Form4137, W2 codes A/B/M/N/K and NEC box3 sources.
// Native money/public Save/persistence is checked before source attribution.

#[test]
fn unit13_native_payroll_other_social_tips() {
    check(Source::SocialTips);
}
#[test]
fn unit13_native_payroll_other_government_tips() {
    check(Source::GovernmentTips);
}
#[test]
fn unit13_native_payroll_other_railroad_tips_preserve_native_no4137() {
    check(Source::RailroadTips);
}
#[test]
fn unit13_native_payroll_other_uncollected_actual_owner() {
    check(Source::Uncollected);
}
#[test]
fn unit13_native_payroll_other_uncollected_two49c_sources() {
    check(Source::CentPair);
}
#[test]
fn unit13_native_payroll_other_golden_w2_code_k() {
    check(Source::GoldenW2);
}
#[test]
fn unit13_native_payroll_other_golden_nec_actual_payee() {
    check(Source::GoldenNec);
}
#[test]
fn unit13_native_payroll_other_same_owner_w2_employee_is_unique() {
    check(Source::SameEmployee);
}

#[test]
fn unit13_native_payroll_other_preserves_category_boundaries_and_unknown_footing() {
    use crate::injured_spouse::income_categories::NativeOtherPayrollTaxes;
    let mut source = NativeOtherPayrollTaxes {
        tips: (456, [73, 383]),
        uncollected: (1, [49, 49]),
        golden: (1, [49, 49]),
    };
    // Each actual annual category has its own dollar; rounding the combined
    // cents or manufacturing source Dollars would erase one category here.
    assert_eq!(source.joint(), 458);
    assert_eq!(source.owners(0), Some([73, 385]));
    assert_eq!(source.owners(1), Some([75, 383]));
    assert_eq!(source.owners(2), None);
    source.tips.0 += 1;
    assert_eq!(source.owners(0), None);
    source.tips.0 -= 1;
    source.uncollected.0 = 2;
    assert_eq!(source.owners(0), None);
    source.uncollected.0 = 1;
    source.golden.0 = 0;
    assert_eq!(source.owners(0), None);
    assert_eq!(NativeOtherPayrollTaxes::default().owners(0), Some([0; 2]));
}

#[test]
fn unit13_native_payroll_other_missing_source_packet_is_not_observed_zero() {
    use crate::injured_spouse::income_categories::NativeOtherPayrollTaxes;
    let before = fixture(Source::RailroadTips, Owner::Taxpayer, Owner::Taxpayer, false);
    let mut d = before.clone();
    // Remove tips but retain an explicitly present zero uncollected source.
    // The actual closed producer knows zero; a missing synthetic packet does not.
    for entry in &mut d.income {
        if let FederalIncome::Wage { data } = &mut entry.income {
            edit_wage(data).employee_tips = None;
            edit_wage(data).work_deductions.tips_claim = TipsClaim::No;
            edit_wage(data).uncollected_tax = Some(W2UncollectedTax {
                tip_social: 0,
                tip_medicare: 0,
                life_social: 0,
                life_medicare: 0,
            });
        }
    }
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 270000);
    assert_eq!(value(&r, "regular_income_tax"), 42268);
    assert_eq!(r.tax, Some(4302800));
    assert_eq!(r.refund, Some(2697200));
    let mut facts = liability_portfolio_native_facts(&d, Some([0; 2]), Some(&r));
    facts.income_categories.other_payroll_taxes =
        Some(Box::new(NativeOtherPayrollTaxes::default()));
    let totals = actual_liability_totals(&r);
    let known = crate::injured_spouse::owners::project_sources(
        crate::related_forms::RelatedFormContext::original(
            &d,
            &crate::related_forms::effective_claimants(&d),
        ),
        &r,
        None,
        &facts,
        Some(&totals),
    )
    .unwrap();
    assert!(crate::injured_spouse::separate_taxes(&d, &known).is_some());
    facts.income_categories.other_payroll_taxes = None;
    let missing = crate::injured_spouse::owners::project_sources(
        crate::related_forms::RelatedFormContext::original(
            &d,
            &crate::related_forms::effective_claimants(&d),
        ),
        &r,
        None,
        &facts,
        Some(&totals),
    )
    .unwrap();
    assert_eq!(crate::injured_spouse::separate_taxes(&d, &missing), None);
    assert_eq!(supported(&d), r);
}
