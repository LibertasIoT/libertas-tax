use super::*;

#[test]
fn unit13_financial_income_context_joint_share_drives_native_income_withholding_and_preferences() {
    use crate::tests::cross::interview::accepted;
    use crate::tests::support::builders::native_income::native_int1099;
    use crate::tests::support::builders::{
        fixture_document_names, information_return_answers_and_owners, named_document,
        set_fixture_recipient, set_joint_financial_share,
    };
    use crate::tests::support::client::income_pages::save_documents;
    let draft = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
    // Printed for both spouses: the papers name them.
    let joint = fixture_document_names(&draft, -3).unwrap().recipient;
    let mut papers = vec![
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(native_int1099(
                "Synthetic joint bank".into(),
                100_000,
            ))),
        },
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Div1099(dividend_document(dividend_source(
                FederalDividend {
                    withholding: 10_001,
                    ..native_dividend(200_001, 100_001, 300_001)
                },
            ))),
        },
    ];
    for source in &mut papers {
        set_fixture_recipient(source, &joint);
    }
    let papers: Vec<_> =
        papers.into_iter().map(|source| named_document(&draft, -3, 0, source)).collect();
    let mut session = Interview::new(TEST_USER, draft);
    let request = save_documents(&mut session, papers.clone());
    accepted(&send(&mut session, request));
    let ids = session
        .draft
        .income
        .iter()
        .filter(|entry| {
            matches!(entry.income, FederalIncome::Interest { .. } | FederalIncome::Dividend { .. })
        })
        .map(|entry| entry.id)
        .collect::<Vec<_>>();
    let mut request =
        information_return_answers_and_owners(&mut session, &papers, TaxSection::InterestDividends)
            .unwrap();
    set_joint_financial_share(&mut session, &mut request, &ids, 40);
    accepted(&send(&mut session, request));
    // Instructions for Form 8379, lines 13b and 19: allocate joint income
    // and its withholding to the spouses who own it. Pub. 550 preserves the
    // qualified-dividend and capital-gain character of those same amounts.
    for (injured, income, withheld, taxes) in [
        (Owner::Taxpayer, [2_400, 3_600], [5_040, 560], [5_119, 513]),
        (Owner::Spouse, [3_600, 2_400], [560, 5_040], [513, 5_119]),
    ] {
        session.draft.injured_spouse.as_mut().unwrap().injured = injured;
        let result = supported(&session.draft);
        for (suffix, expected) in [("a", 6_000), ("b", income[0]), ("c", income[1])] {
            assert_eq!(value(&result, &alloc::format!("form_8379_line_13b_{suffix}")), expected);
        }
        for (suffix, expected) in [("a", 5_600), ("b", withheld[0]), ("c", withheld[1])] {
            assert_eq!(value(&result, &alloc::format!("form_8379_line_19_{suffix}")), expected);
        }
        // MFS taxable income is 46,300/7,500. Preferred income 1,600/2,400
        // leaves ordinary income 44,700/5,100: Tax Table taxes 5,119/513.
        let explanation = &result
            .lines
            .iter()
            .find(|row| row.id == "form_8379_injured_share_of_tax")
            .unwrap()
            .explanation;
        assert!(matches!(
            &explanation.args[..],
            [_, MessageArg::Currency { minor_units: first, .. }, MessageArg::Currency { minor_units: second, .. }]
                if [*first, *second] == taxes
        ));
        assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
    }
    // A normal joint return reports the complete source once: 80,000 wages
    // + 1,000 interest + 2,000 dividends + 3,000 capital gain = 86,000 AGI.
    session.draft.injured_spouse = None;
    session.draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 86_000);
    for (line, cents) in
        [("2b", 100_000), ("3b", 200_000), ("3a", 100_000), ("7a", 300_000), ("25b", 10_000)]
    {
        assert_eq!(return_line(&result, line), cents);
    }
    assert_eq!(result.tax, Some(548_300));
    assert_eq!(result.refund, Some(11_700));
}

#[test]
fn unit13_financial_income_context_known_medicaid_class_tie_preserves_changed_filer_credit() {
    for (filer, allowed) in [(CarryFiler::You, 4884), (CarryFiler::Spouse, 479)] {
        for claim in [None, Some(Owner::Taxpayer), Some(Owner::Spouse)] {
            for reversed in [false, true] {
                let mut draft = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
                if let Some(injured) = claim {
                    draft.injured_spouse.as_mut().unwrap().injured = injured;
                } else {
                    draft.injured_spouse = None;
                    draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
                }
                let mut interest = income(savings_income(1000, 0));
                interest.owner = crate::DocumentOwner::Spouses;
                add(&mut draft, interest);
                changed_filer_carry(&mut draft, filer);
                let baseline = supported(&draft);
                // Original R20 native result: MFS5023/493 yields ratios9106/894;
                // joint limit5363 permits4884/479 of the changed-filer changed_filer_carry.
                assert_eq!(value(&baseline, "general_business_credit"), allowed);
                assert_eq!(value(&baseline, "agi"), 81_000);
                assert_eq!(baseline.tax, Some((5363 - allowed) * 100));

                let FederalIncome::Wage { data } = &mut draft.income[0].income else { panic!() };
                edit_wage(data).other_box12.push(W2CodeAmount { code: W2Code::II, amount: 50 });
                add(&mut draft, source_medicaid(Owner::Taxpayer, 50));
                if reversed {
                    draft.income.reverse();
                }
                let result = supported(&draft);
                // Both known sources have the same owner. Native1d+1 and8s−1
                // cancel in income. Only printed source-class rounding is tied.
                assert_eq!(return_line(&result, "1d"), 100);
                assert_eq!(return_line(&result, "8"), -100);
                assert_eq!(value(&result, "agi"), 81_000);
                assert_eq!(value(&result, "taxable_income"), 48_800);
                assert_eq!(value(&result, "general_business_credit"), allowed);
                assert_eq!(
                    (result.tax, result.refund, result.balance),
                    (baseline.tax, baseline.refund, baseline.balance)
                );
                if claim.is_some() {
                    // Approved W-2-first residual: native1d1 and8s−1
                    // remain W-2; both Other residuals are exactly zero.
                    assert_eq!(value(&result, "form_8379_line_13a_a"), 80_000);
                    assert_eq!(value(&result, "form_8379_line_13b_a"), 1000);
                }
                assert_eq!(estimate(&restart(draft).draft), result);
            }
        }
    }
}

#[test]
fn unit13_financial_income_context_four_cells_keep_whole_native_owner_rounding() {
    use crate::injured_spouse::income_categories::{
        IncomeCategory, NativeIncomeCategories, NativeIncomeComponent,
    };
    use crate::kernel::ratio::exact;
    let mut base = injured_return((59_999, 5_000), (20_000, 500), 0, allocation());
    for entry in &mut base.income {
        let FederalIncome::Wage { data } = &mut entry.income else { panic!() };
        edit_wage(data).inmate_pay = 50;
    }
    let mut draft = base.clone();
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut entry = income(FederalIncome::OtherIncome {
            data: OtherIncomeItem { amount: 50, withholding: 0, kind: OtherIncomeKind::InmatePay },
        });
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut draft, entry);
    }
    let result = supported(&draft);
    assert_eq!(return_line(&result, "1a"), 7_999_800);
    assert_eq!(value(&result, "inmate_pay"), 2);
    assert_eq!(value(&result, "agi"), 80_000);
    assert_eq!(result.tax, Some(524_300));
    assert_eq!(result.refund, Some(25_700));
    let components: Vec<_> =
        draft
            .income
            .iter()
            .map(|entry| NativeIncomeComponent {
                id: entry.id,
                owner: usize::from(entry.owner == crate::DocumentOwner::Spouses),
                category: IncomeCategory::InmatePay,
                source_cents: Some(exact::whole(50)),
                rest_cents: Some(exact::whole(
                    if matches!(entry.income, FederalIncome::Wage { .. }) { 0 } else { 50 },
                )),
            })
            .collect();
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        let mut facts =
            liability_specialized_wage_facts(&base, Some([0; 2]), SpecializedWageFixture::Inmate);
        // Four accepted actual50c operands, one native8u categoryJ=2.
        // Source classes each haveJ=1, but rounding each class's50c injured
        // operand would incorrectly give that owner2 instead of native1.
        let mut categories = NativeIncomeCategories::collect(
            &draft,
            &components,
            &[(
                IncomeCategory::InmatePay,
                exact::whole(200),
                2,
                Some([100, 100].map(exact::whole)),
            )],
        );
        // The exhaustive base fixture proves the absent investment sources
        // known empty. Retain that native operand when adding this category.
        categories.investment_sources = facts.income_categories.investment_sources.clone();
        *facts.income_categories = categories;
        for entry in &draft.income[2..] {
            facts.other_sources.insert(entry.id, Some(crate::kernel::ratio::exact::whole(50)));
        }
        let mut claim = allocation();
        claim.injured = injured;
        let totals = actual_liability_totals(&result);
        let projection = crate::injured_spouse::owners::project_sources(
            crate::related_forms::RelatedFormContext::original(
                &draft,
                &crate::related_forms::effective_claimants(&draft),
            ),
            &result,
            Some(&claim),
            &facts,
            Some(&totals),
        )
        .unwrap();
        assert_eq!(
            projection.agi(),
            if injured == Owner::Taxpayer { [60_000, 20_000] } else { [59_999, 20_001] }
        );
        assert_eq!(
            crate::injured_spouse::separate_taxes(&draft, &projection),
            Some(if injured == Owner::Taxpayer { [5023, 393] } else { [5017, 393] })
        );
    }
}

#[test]
fn unit13_financial_income_context_unknown_capital_is_not_repaired_by_class_flag() {
    let draft = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
    let result = supported(&draft);
    let mut facts = liability_portfolio_facts(&draft, None);
    facts.income_categories.source_class_unavailable = true;
    let totals = actual_liability_totals(&result);
    let projection = crate::injured_spouse::owners::project_sources(
        crate::related_forms::RelatedFormContext::original(
            &draft,
            &crate::related_forms::effective_claimants(&draft),
        ),
        &result,
        None,
        &facts,
        Some(&totals),
    )
    .unwrap();
    assert_eq!(crate::injured_spouse::separate_taxes(&draft, &projection), None);
}

#[test]
fn unit13_financial_income_context_half_cent_2106_uses_native_line1a_class_residual() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                let mut draft = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
                draft.injured_spouse.as_mut().unwrap().injured = injured;
                let entry = draft
                    .income
                    .iter_mut()
                    .find(|entry| entry.owner == crate::DocumentOwner::from(owner))
                    .unwrap();
                let FederalIncome::Wage { data } = &mut entry.income else { panic!() };
                edit_wage(data).wages += 50;
                let employer = entry.id;
                draft.adjustments.as_mut().unwrap().employee_expenses =
                    vec![EmployeeExpenseClaim {
                        improvement_buildings: Vec::new(),
                        equipment_assets: vec![],
                        dot_hours_of_service: false,
                        leased_vehicles: vec![],
                        owner,
                        occupation: "Fee-basis official, actual half-cent excess".into(),
                        capacity: EmployeeCapacity::FeeBasisOfficial,
                        employers: vec![employer],
                        local_transportation: 190_000,
                        travel: 0,
                        other: 0,
                        meals: 0,
                        reimbursements: 190_050,
                        mileage: vec![],
                        vehicle_assets: vec![],
                    }];
                assert_eq!(
                    draft.adjustments.as_ref().unwrap().employee_expenses[0]
                        .lines(&draft, false)
                        .excess,
                    50
                );
                if reversed {
                    draft.income.reverse();
                }
                let result = supported(&draft);
                // Actual80000.50 W-2 +0.50 non-W2 excess makes native1a80001.
                // The approved W-2-first class target is80001; Other is zero.
                assert_eq!(return_line(&result, "1a"), 8_000_100);
                assert_eq!(value(&result, "agi"), 80_001);
                assert_eq!(value(&result, "taxable_income"), 47_801);
                assert_eq!(result.tax, Some(524_300));
                assert_eq!(result.refund, Some(25_700));
                assert_eq!(estimate(&restart(draft).draft), result);
                let mut owners = [60_000, 20_000];
                owners[usize::from(owner == Owner::Spouse)] += 1;
                let first = usize::from(injured == Owner::Spouse);
                for (name, amount) in
                    [("a", 80_001), ("b", owners[first]), ("c", owners[1 - first])]
                {
                    assert_eq!(
                        value(&result, &alloc::format!("form_8379_line_13a_{name}")),
                        amount
                    );
                    assert_eq!(value(&result, &alloc::format!("form_8379_line_13b_{name}")), 0);
                }
            }
        }
    }
}

#[test]
fn unit13_financial_income_context_four_cells_changed_filer_claim_diagnostic() {
    extern crate std;
    // Diagnostic of the original whole-category owner policy, not a choice of
    // a new financial rounding order. Keep the four-cell native oracle above.
    for filer in [CarryFiler::You, CarryFiler::Spouse] {
        let mut observed = vec![];
        for claim in [None, Some(Owner::Taxpayer), Some(Owner::Spouse)] {
            let mut by_order = vec![];
            for reversed in [false, true] {
                let mut draft = injured_return((59_999, 5_000), (20_000, 500), 0, allocation());
                for entry in &mut draft.income {
                    let FederalIncome::Wage { data } = &mut entry.income else { panic!() };
                    edit_wage(data).inmate_pay = 50;
                }
                for owner in [Owner::Taxpayer, Owner::Spouse] {
                    let mut entry = income(FederalIncome::OtherIncome {
                        data: OtherIncomeItem {
                            amount: 50,
                            withholding: 0,
                            kind: OtherIncomeKind::InmatePay,
                        },
                    });
                    entry.owner = crate::DocumentOwner::from(owner);
                    add(&mut draft, entry);
                }
                if let Some(injured) = claim {
                    draft.injured_spouse.as_mut().unwrap().injured = injured;
                } else {
                    draft.injured_spouse = None;
                    draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
                }
                let ordinary = supported(&draft);
                assert_eq!(value(&ordinary, "agi"), 80_000);
                assert_eq!(ordinary.tax, Some(524_300));
                assert_eq!(ordinary.refund, Some(25_700));
                changed_filer_carry(&mut draft, filer);
                if reversed {
                    draft.income.reverse();
                }
                let result = supported(&draft);
                let allowed = value(&result, "general_business_credit");
                assert_eq!(return_line(&result, "1a"), 7_999_800);
                assert_eq!(value(&result, "inmate_pay"), 2);
                assert_eq!(value(&result, "agi"), 80_000);
                assert_eq!(value(&result, "taxable_income"), 47_800);
                assert!(allowed > 0 && allowed <= 5000);
                assert_eq!(result.tax, Some((5243 - allowed) * 100));
                assert_eq!(result.refund, Some((257 + allowed) * 100));
                assert_eq!(estimate(&restart(draft).draft), result);
                by_order.push((allowed, result.tax, result.refund));
            }
            assert_eq!(by_order[0], by_order[1]);
            observed.push(by_order[0]);
        }
        std::println!("actual four-cell changed-filer {filer:?}, None/You/Spouse: {observed:?}");
    }
}
