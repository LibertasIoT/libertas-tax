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
        item.owner = crate::DocumentOwner::from(owner);
        add(&mut before, item);
    }
    let mut d = before.clone();
    let mut entry = income(FederalIncome::EmployeeWages {
        data: EmployeeWages {
            source_issuer: Default::default(),
            source_recipient: Default::default(),
            wages: 500000,
            withholding: 0,
            form1099: true,
            payroll: crate::EmployeePayrollSource::Unavailable,
        },
    });
    entry.label.clear();
    add(&mut d, entry);
    let id = d.income.last().unwrap().id;
    // 2026-10-19: missing actual coverage cannot supply an observed zero tax.
    let r = estimate(&d);
    assert_eq!(value(&r, "agi"), 275000);
    assert!(!r.lines.iter().any(|line| line.id == "regular_income_tax"));
    assert_eq!(r.state, ResultState::Incomplete);
    assert_eq!(r.tax, None);
    let source = d.income.iter().find(|entry| entry.id == id).unwrap();
    assert!(
        r.issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_EMPLOYEE_PAYROLL_COVERAGE_UNAVAILABLE"
                && issue.context == crate::interview::income_issue_context(source))
    );
    assert_eq!(estimate(&restart(d.clone()).draft), r);
    // Independent actual-source correction: documented domestic neither-covered
    // employment supplies a real zero payroll result, unlike unavailable coverage.
    if let FederalIncome::EmployeeWages { data } = &mut d.income.last_mut().unwrap().income {
        data.payroll = crate::EmployeePayrollSource::Government {
            social: crate::EmployeeCoverage::NotCovered,
            medicare: crate::EmployeeCoverage::NotCovered,
            reason: None,
        };
    }
    let known_result = supported(&d);
    assert_eq!(known_result.tax, Some(4_422_800));
    let mut facts = liability_portfolio_native_facts(&before, Some([0; 2]), Some(&known_result));
    facts.other_sources.insert(id, Some(crate::kernel::ratio::exact::whole(500000)));
    let totals = actual_liability_totals(&known_result);
    facts.income_categories.employee_wage_tax_dollars = Some([0; 2]);
    let known = crate::injured_spouse::owners::project_sources(
        crate::related_forms::RelatedFormContext::original(
            &d,
            &crate::related_forms::effective_claimants(&d),
        ),
        &known_result,
        None,
        &facts,
        Some(&totals),
    )
    .unwrap();
    assert_eq!(crate::injured_spouse::separate_taxes(&d, &known), Some([40714, 5403]));
    facts.income_categories.employee_wage_tax_dollars = None;
    let missing = crate::injured_spouse::owners::project_sources(
        crate::related_forms::RelatedFormContext::original(
            &d,
            &crate::related_forms::effective_claimants(&d),
        ),
        &known_result,
        None,
        &facts,
        Some(&totals),
    )
    .unwrap();
    assert_eq!(crate::injured_spouse::separate_taxes(&d, &missing), None);
}

#[test]
fn unit13_additional_medicare_contributors_native_pools_keep_bounds_and_unavailability() {
    use crate::injured_spouse::income_categories::NativeAdditionalMedicare;
    let mut source = NativeAdditionalMedicare {
        unavailable_parts: [false; 3],
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
        unavailable_parts: [false; 3],
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
