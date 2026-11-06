use super::*;

#[test]
fn unit13_settled_adoption_unmatched_code_t_stays_with_actual_employee() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                let mut claim = allocation();
                claim.injured = injured;
                let mut d = injured_return((60_000, 6000), (40_000, 4000), 0, claim);
                source_adoption_code_t(&mut d, owner, 600_000);
                if reversed {
                    d.income.reverse();
                }
                let r = supported(&d);
                assert_eq!(return_line(&r, "1a"), 10_000_000);
                assert_eq!(return_line(&r, "1f"), 600_000);
                assert_eq!(value(&r, "agi"), 106_000);
                assert_eq!(value(&r, "taxable_income"), 73_800);
                assert_eq!(r.tax, Some(836_300));
                assert_eq!(r.refund, Some(163_700));
                let d = save_reopen_native_sources(
                    d,
                    &[TaxSection::Documents],
                    &[TaxSection::Documents],
                );
                let r = supported(&d);
                let mut owners = [60_000, 40_000];
                owners[usize::from(owner == Owner::Spouse)] += 6000;
                assert_eq!(column(&r, "13a", "a"), 106_000);
                assert_eq!(column(&r, "13a", "b"), owners[usize::from(injured == Owner::Spouse)]);
                assert_eq!(column(&r, "13a", "c"), owners[usize::from(injured != Owner::Spouse)]);
                assert_eq!(column(&r, "13b", "a"), 0);
                assert_eq!(column(&r, "13b", "b"), 0);
                assert_eq!(column(&r, "13b", "c"), 0);
            }
        }
    }
}

#[test]
fn unit13_settled_adoption_program_keeps_financial_owner_when_source_class_unknown() {
    use crate::injured_spouse::income_categories::AdoptionIncomeDollars;
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for matched in [false, true] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                for reversed in [false, true] {
                    let mut claim = allocation();
                    claim.injured = injured;
                    let mut d = injured_return((265_080, 25_000), (0, 0), 0, claim);
                    employer_adoption(&mut d, 20_000, 0, false);
                    let AdoptionBenefits::Program { owner: employee, .. } =
                        &mut d.credits.as_mut().unwrap().adoptions[0].benefits
                    else {
                        panic!()
                    };
                    *employee = owner;
                    if matched {
                        source_adoption_code_t(&mut d, owner, 2_000_000);
                    }
                    if reversed {
                        d.income.reverse();
                    }
                    let r = supported(&d);
                    // Original native8839 phaseout: MAGI285080, exclusion8835,
                    // taxable20000−8835=11165. Native benefit is not a raw-cent
                    // source group and is not independently rounded again.
                    assert_eq!(value(&r, "adoption_exclusion_magi"), 285_080);
                    assert_eq!(value(&r, "adoption_benefits_excluded"), 8835);
                    assert_eq!(return_line(&r, "1a"), 26_508_000);
                    assert_eq!(return_line(&r, "1f"), 1_116_500);
                    assert_eq!(value(&r, "agi"), 276_245);
                    assert_eq!(value(&r, "taxable_income"), 244_045);
                    if matched && owner == Owner::Spouse {
                        // Actual codeT creates a second positive payroll
                        // source. Answer the existing independent bounded
                        // Additional Medicare allocation, assigning the tax
                        // to the taxpayer consistently for either claimant.
                        // Adoption ownership must not fabricate that answer.
                        let tax = value(&r, "additional_medicare_tax") * 100;
                        d.injured_spouse.as_mut().unwrap().allocations.push(InjuredAmount {
                            component: "additional_medicare_tax".into(),
                            injured: if injured == Owner::Taxpayer { tax } else { 0 },
                        });
                        let chosen = supported(&d);
                        assert_eq!(chosen.return_lines, r.return_lines);
                        assert_eq!(
                            (chosen.tax, chosen.refund, chosen.balance),
                            (r.tax, r.refund, r.balance)
                        );
                    }
                    let d = save_reopen_native_sources(
                        d,
                        &[TaxSection::Documents, TaxSection::MoreCredits],
                        &[TaxSection::Documents],
                    );
                    let r = supported(&d);
                    let mut output = FederalResult::empty(d.revision);
                    let actual = crate::adoption::benefits(
                        &d.credits.as_ref().unwrap().adoptions,
                        &d.income,
                        d.setup.as_ref().unwrap(),
                        Some(285_080),
                        &mut output,
                    );
                    assert_eq!(actual.source_class_complete, matched);
                    let mut expected = [265_080, 0];
                    expected[usize::from(owner == Owner::Spouse)] += 11_165;
                    let mut facts = liability_specialized_wage_facts(
                        &d,
                        Some([0; 2]),
                        SpecializedWageFixture::Adoption,
                    );
                    for (earned, amount) in facts.earned.iter_mut().zip(actual.wages) {
                        *earned += amount;
                    }
                    facts.income_categories.adoption =
                        Some(alloc::boxed::Box::new(AdoptionIncomeDollars::from(&actual)));
                    facts.income_categories.source_class_unavailable = !matched;
                    facts.income_categories.unclassified_dollars = if matched { 0 } else { 11_165 };
                    let totals = actual_liability_totals(&r);
                    let projection = crate::injured_spouse::owners::project_sources(
                        crate::related_forms::RelatedFormContext::original(
                            &d,
                            &crate::related_forms::effective_claimants(&d),
                        ),
                        &r,
                        d.injured_spouse.as_ref(),
                        &facts,
                        Some(&totals),
                    )
                    .unwrap();
                    assert_eq!(projection.agi(), expected);
                    assert!(
                        crate::injured_spouse::owners::separate_taxes(&d, &projection).is_some(),
                        "{owner:?}/matched{matched}/{injured:?}/reversed{reversed}"
                    );
                    if matched {
                        assert_eq!(column(&r, "13a", "a"), 276_245);
                        assert_eq!(
                            column(&r, "13a", "b"),
                            expected[usize::from(injured == Owner::Spouse)]
                        );
                        assert_eq!(
                            column(&r, "13a", "c"),
                            expected[usize::from(injured != Owner::Spouse)]
                        );
                        assert_eq!(column(&r, "13b", "a"), 0);
                    } else {
                        assert!(r.lines.iter().all(|line| !matches!(
                            line.id.as_str(),
                            "form_8379_line_13a_a"
                                | "form_8379_line_13a_b"
                                | "form_8379_line_13a_c"
                                | "form_8379_line_13b_a"
                                | "form_8379_line_13b_b"
                                | "form_8379_line_13b_c"
                        )));
                    }
                }
            }
        }
    }
}

#[test]
fn unit13_settled_adoption_joint_deemed_excess_remains_separately_unavailable() {
    let mut d = joint_special_needs_below_code_t_cap();
    d.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    d.injured_spouse = Some(allocation());
    let r = supported(&d);
    assert_eq!(return_line(&r, "1a"), 10_000_000);
    assert_eq!(return_line(&r, "1f"), -567_000);
    assert_eq!(value(&r, "agi"), 94_330);
    assert!(r.lines.iter().all(|line| !matches!(
        line.id.as_str(),
        "form_8379_line_13a_a"
            | "form_8379_line_13a_b"
            | "form_8379_line_13a_c"
            | "form_8379_line_13b_a"
            | "form_8379_line_13b_b"
            | "form_8379_line_13b_c"
    )));
    let mut ordinary = d.clone();
    ordinary.injured_spouse = None;
    let plain = supported(&ordinary);
    assert_eq!(r.return_lines, plain.return_lines);
    assert_eq!((r.tax, r.refund, r.balance), (plain.tax, plain.refund, plain.balance));
}
