use super::*;
use crate::tests::support::assert::injured_income::additional_medicare_contributors::check;
use crate::tests::support::builders::additional_medicare::Source;
// Native Form8959 contributor controls: the source operands must follow the
// actual settled Parts I/II/SE, not raw ScheduleC profit or only W2 wages.

#[test]
fn unit13_additional_medicare_contributors_partner_se_opposite_w2() {
    check(Source::PartnerSe);
}
#[test]
fn unit13_additional_medicare_contributors_below_threshold_c_is_not_positive_se() {
    check(Source::SmallBusiness);
}
#[test]
fn unit13_additional_medicare_contributors_rrta_opposite_ordinary_w2() {
    check(Source::Railroad);
}

#[test]
fn unit13_additional_medicare_contributors_missing_employee_packet_is_not_observed_zero() {
    let mut before = injured_return((200000, 60000), (50000, 10000), 0, allocation());
    before.injured_spouse = None;
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut item = income(savings_income(10000, 0));
        item.owner = owner;
        add(&mut before, item);
    }
    let mut d = before.clone();
    let mut entry = income(FederalIncome::EmployeeWages {
        data: EmployeeWages {
            wages: 500000,
            withholding: 0,
            form1099: true,
            reason: EmployeeWageReason::Ss8Determination { date: 20260501 },
            complete: false,
        },
    });
    entry.label.clear();
    add(&mut d, entry);
    let id = d.income.last().unwrap().id;
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 275000);
    assert_eq!(value(&r, "regular_income_tax"), 43468);
    assert_eq!(r.tax, Some(4422800));
    assert_eq!(r.refund, Some(2577200));
    assert!(
        r.review_notes
            .iter()
            .any(|note| note.display.key == "WARN_EMPLOYEE_WAGES_PAYROLL_TAX_LEFT_OUT")
    );
    let mut facts = liability_portfolio_native_facts(&before, Some([0; 2]), Some(&r));
    // Explicitly supply this actual accepted noninvestment income component;
    // the older portfolio-only helper rejects EmployeeWages itself.
    facts.other_sources.insert(id, Some(crate::kernel::ratio::exact::whole(500000)));
    let totals = actual_liability_totals(&r);
    // A real closed native call omits covered payroll for complete=false and
    // retains observed8919 ownerzero. This differs from a missing packet.
    facts.income_categories.employee_wage_tax_dollars = Some([0; 2]);
    let known = crate::injured_spouse::owners::project_sources(&d, &r, None, &facts, Some(&totals))
        .unwrap();
    assert_eq!(crate::injured_spouse::separate_taxes(&d, &known), Some([40714, 5403]));
    facts.income_categories.employee_wage_tax_dollars = None;
    let missing =
        crate::injured_spouse::owners::project_sources(&d, &r, None, &facts, Some(&totals))
            .unwrap();
    assert_eq!(crate::injured_spouse::separate_taxes(&d, &missing), None);
    assert_eq!(supported(&d), r);
}

#[test]
fn unit13_additional_medicare_contributors_native_pools_keep_bounds_and_unavailability() {
    use crate::injured_spouse::income_categories::NativeAdditionalMedicare;
    let mut source = NativeAdditionalMedicare {
        tax_dollars: [450, 450, 0],
        payroll_categories: [(300000, [20000000, 10000000]), (0, [0, 0]), (0, [0, 0])],
        railroad: (300000, [0, 30000000]),
        se_joint_dollars: 0,
        se_dollars: Some([0, 0]),
    };
    // Native joint900: joint wage pool450 may go to either person, but the
    // actual exclusive RRTA pool450 belongs to spouse. No ratio/default.
    assert_eq!(source.capacities(900), Some([450, 900]));
    assert_eq!(
        crate::kernel::attribution::allowed_bounds(900, source.capacities(900).unwrap()),
        Some((0, 450))
    );
    source.tax_dollars = [0, 0, 381];
    source.se_joint_dollars = 92350;
    source.se_dollars = None;
    assert_eq!(source.capacities(381), None);
    source.se_dollars = Some([0, 92350]);
    assert_eq!(source.capacities(381), Some([0, 381]));
    source.se_dollars = Some([0, 92349]);
    assert_eq!(source.capacities(381), None); // Actual native SE footing required.
}
#[test]
fn unit13_additional_medicare_contributors_zero_rounded_category_is_not_another_owner() {
    use crate::injured_spouse::income_categories::NativeAdditionalMedicare;
    let mut source = NativeAdditionalMedicare {
        tax_dollars: [450, 0, 0],
        payroll_categories: [(300000, [30000000, 0]), (0, [0, 49]), (0, [0, 0])],
        railroad: (0, [0, 0]),
        se_joint_dollars: 0,
        se_dollars: Some([0, 0]),
    };
    // Actual tips49c is its own native annual zero, not a positive raw
    // contribution to the separately rounded300000 W2 category's tax.
    assert_eq!(source.capacities(450), Some([450, 0]));
    assert_eq!(source.capacities(451), None); // Actual native tax parts must foot.
    source.payroll_categories[1].0 = 1;
    assert_eq!(source.capacities(450), None); // No manufactured source footing.
}
