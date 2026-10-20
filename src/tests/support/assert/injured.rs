use crate::tests::cross::interview::{accepted, open_topic};
use crate::tests::*;

pub(in crate::tests) fn column(r: &FederalResult, line: &str, column: &str) -> i64 {
    value(r, &alloc::format!("form_8379_line_{line}_{column}"))
}

pub(in crate::tests) fn assert_rest_rounding_sources(
    sources: &[(RestRoundingSource, Owner, i64)],
    categories: [i64; 4],
    owners: [i64; 2],
    injured: Owner,
) {
    let mut plan = allocation();
    plan.injured = injured;
    let mut draft = injured_return((60_000, 5_000), (20_000, 500), 1_000, plan);
    for &(kind, owner, cents) in sources {
        add(&mut draft, rest_rounding_source(kind, owner, cents));
    }
    let result = supported(&draft);
    let joint: i64 = categories.iter().sum();

    // Establish ordinary native joint income before any Form8379 assertion.
    assert_eq!(return_line(&result, "1a"), 8_000_000);
    assert_eq!(return_line(&result, "2b"), categories[0] * 100);
    assert_eq!(return_line(&result, "3b"), categories[1] * 100);
    assert_eq!(value(&result, "jury_pay"), categories[2]);
    // The ordinary native result emits the unemployment rest_rounding_source rows and the
    // combined1040 line8, not the optional PRINT Schedule1 line7 projection.
    // These fixtures have no line8 sources other than jury/unemployment.
    assert_eq!(return_line(&result, "8"), (categories[2] + categories[3]) * 100);
    assert_eq!(return_line(&result, "9"), (80_000 + joint) * 100);
    assert_eq!(value(&result, "agi"), 80_000 + joint);
    let mut plain = draft.clone();
    plain.injured_spouse = None;
    let ordinary = supported(&plain);
    assert_eq!(result.return_lines, ordinary.return_lines);
    assert_eq!(
        (result.tax, result.refund, result.balance),
        (ordinary.tax, ordinary.refund, ordinary.balance)
    );

    let expected = if injured == Owner::Taxpayer { owners } else { [owners[1], owners[0]] };
    let observed = ["a", "b", "c"].map(|column| {
        result
            .lines
            .iter()
            .find(|line| line.id == alloc::format!("form_8379_line_13b_{column}"))
            .map(|line| line.amount / 100)
    });
    // A withheld row is unknown, not an observed zero.
    assert_eq!(
        observed,
        [Some(joint), Some(expected[0]), Some(expected[1])],
        "ordinary joint categories verified {categories:?}; exclusive sources {sources:?}; injured {injured:?}; notes {:?}",
        result.review_notes
    );
    assert!(
        !result
            .review_notes
            .iter()
            .any(|note| { note.display.key == "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE" })
    );
}

pub(in crate::tests) fn assert_resolved_income_sources(
    mut draft: FederalDraft,
    topics: &[TaxSection],
    owners: [i64; 2],
    ordinary: impl Fn(&FederalResult),
) {
    let mut observed = vec![];
    let mut expected = vec![];
    for reversed in [false, true] {
        if reversed {
            draft.income.reverse();
        }
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            draft.injured_spouse.as_mut().unwrap().injured = injured;
            let before = supported(&draft);
            ordinary(&before);
            let mut plain = draft.clone();
            plain.injured_spouse = None;
            let native = supported(&plain);
            assert_eq!(before.return_lines, native.return_lines);
            assert_eq!(
                (before.tax, before.refund, before.balance),
                (native.tax, native.refund, native.balance)
            );
            // A native total of79999 falls in the47750–47800 Tax Table
            // row:2480+12%*(47775-24800)=5237. At80000 the next row is5243.
            // The expected category amounts establish which row applies.
            let tax = if owners.iter().sum::<i64>() == -1 { 523_700 } else { 524_300 };
            assert_eq!(before.tax, Some(tax));
            assert_eq!(before.refund, Some(650_000 - tax));

            let mut session = Interview::new(TEST_USER, draft.clone());
            for &topic in topics {
                let request = open_topic(&mut session, topic);
                let bytes = request.to_avro();
                let decoded = P::from_avro(&bytes).unwrap();
                assert_eq!(decoded.to_avro(), bytes);
                accepted(&send(&mut session, decoded));
            }
            let saved = supported(&session.draft);
            ordinary(&saved);
            assert_eq!(saved.return_lines, before.return_lines);
            let mut restored = restart(session.draft.clone());
            assert_eq!(restored.draft, session.draft);
            assert_eq!(estimate(&restored.draft), saved);
            for &topic in topics {
                let request = open_topic(&mut restored, topic);
                accepted(&send(&mut restored, request));
            }
            assert_eq!(estimate(&restored.draft), saved);

            // Financial and normal Save/restart/reopen assertions above run
            // before the independently derived owner-category expectations.
            observed.push(["a", "b", "c"].map(|column| {
                saved
                    .lines
                    .iter()
                    .find(|line| line.id == alloc::format!("form_8379_line_13b_{column}"))
                    .map(|line| line.amount / 100)
            }));
            let pair = if injured == Owner::Taxpayer { owners } else { [owners[1], owners[0]] };
            expected.push([Some(owners.iter().sum()), Some(pair[0]), Some(pair[1])]);
        }
    }
    assert_eq!(observed, expected);
}

pub(in crate::tests) fn assert_income_source_columns(
    r: &FederalResult,
    injured: Owner,
    wages: [i64; 2],
    other: [i64; 2],
) {
    let i = usize::from(injured == Owner::Spouse);
    for (line, sources) in [("13a", wages), ("13b", other)] {
        for (name, amount) in
            [("a", sources.iter().sum()), ("b", sources[i]), ("c", sources[1 - i])]
        {
            assert_eq!(column(r, line, name), amount, "{line}_{name}");
        }
    }
}

pub(in crate::tests) fn assert_settled_care_sources(case: SettledCareCase) {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            let mut draft = injured_return((60_000, 6000), (40_000, 4000), 0, allocation());
            let wage = draft.income.iter_mut().find(|entry| entry.owner == owner).unwrap();
            let FederalIncome::Wage { data } = &mut wage.income else { panic!() };
            let w2_benefits = match case {
                SettledCareCase::W2 | SettledCareCase::Forfeited => 600_050,
                SettledCareCase::Mixed => 400_050,
                SettledCareCase::Grace => 0,
            };
            data.care_benefits = w2_benefits;
            let care = &mut draft.credits.as_mut().unwrap().care;
            care.plan_maximum = CarePlanMaximum::Plans { maximum: 750_000 };
            if w2_benefits > 0 {
                care.benefits.push(CareBenefitAnswers {
                    wage: wage.id,
                    in_box_1: Some(0),
                    forfeited: Some(if matches!(case, SettledCareCase::Forfeited) {
                        w2_benefits
                    } else {
                        0
                    }),
                });
            }
            if matches!(case, SettledCareCase::Grace | SettledCareCase::Mixed) {
                care.grace.push(CareGraceBalance { owner, used_in_return_year: 200_050 });
            }
            if reversed {
                draft.income.reverse();
            }
            let taxable = match case {
                SettledCareCase::W2 => 6001,
                SettledCareCase::Grace => 2001,
                SettledCareCase::Mixed => 6002,
                SettledCareCase::Forfeited => 0,
            };
            let tax = match case {
                SettledCareCase::W2 | SettledCareCase::Mixed => 8363,
                SettledCareCase::Grace => 7883,
                SettledCareCase::Forfeited => 7643,
            };
            // Independently reported line12/13 round before this native sum:
            // Mixed is4001+2001=6002, not a rerounded6001 raw-cent bucket.
            let ordinary = supported(&draft);
            assert_eq!(return_line(&ordinary, "1a"), 10_000_000);
            assert_eq!(return_line(&ordinary, "1e"), taxable * 100);
            assert_eq!(value(&ordinary, "agi"), 100_000 + taxable);
            assert_eq!(value(&ordinary, "taxable_income"), 67_800 + taxable);
            assert_eq!(ordinary.tax, Some(tax * 100));
            assert_eq!(ordinary.refund, Some((10_000 - tax) * 100));
            let saved = save_reopen_native_sources(
                draft.clone(),
                &[TaxSection::Wages, TaxSection::Care],
                &[TaxSection::Care],
            );
            let again = restart(saved);
            assert_eq!(estimate(&again.draft).return_lines, ordinary.return_lines);

            // Inspect the native owner output only after ordinary figures and
            // complete public Save/persistence passed. No displayed preview
            // or calculated annual Dollar lifted into fake source cents.
            let context = crate::forms::form_2441::Form2441Context {
                draft: &again.draft,
                setup: again.draft.setup.as_ref().unwrap(),
                credits: again.draft.credits.as_ref().unwrap(),
                joint: true,
                care_people: 0,
                care_expenses: 0,
                earned: [60_000, 40_000],
                optional_eic: [0; 2],
                deduction_owners: [0; 2],
            };
            let result = crate::forms::form_2441::compute(
                &context,
                &mut FederalResult::empty(again.draft.revision),
            )
            .unwrap();
            let index = usize::from(owner == Owner::Spouse);
            let mut expected = [0; 2];
            expected[index] = taxable;
            assert_eq!(result.taxable, expected);
            assert_eq!(
                result.taxable_from_w2[index],
                matches!(case, SettledCareCase::W2 | SettledCareCase::Forfeited)
            );
            assert!(result.taxable_from_w2[1 - index]);

            for injured in [Owner::Taxpayer, Owner::Spouse] {
                let mut claim = allocation();
                claim.injured = injured;
                let mut facts = liability_specialized_wage_facts(
                    &again.draft,
                    Some([0; 2]),
                    SpecializedWageFixture::Care,
                );
                // The same actual native owner output is already annual
                // Dollars. Its grace provenance does not make owners unknown.
                facts.income_categories.care_taxable_dollars = result.taxable;
                facts.income_categories.care_taxable_from_w2 = result.taxable_from_w2;
                facts.income_categories.source_class_unavailable =
                    taxable > 0 && !result.taxable_from_w2[index];
                for i in 0..2 {
                    facts.earned[i] += result.taxable[i];
                }
                let totals = actual_liability_totals(&ordinary);
                let projection = crate::injured_spouse::owners::project_sources(
                    &again.draft,
                    &ordinary,
                    Some(&claim),
                    &facts,
                    Some(&totals),
                )
                .unwrap();
                let mut income = [60_000, 40_000];
                income[index] += taxable;
                assert_eq!(projection.agi(), income);
                let mut taxes = [5023, 2623];
                taxes[index] = match (case, index) {
                    (SettledCareCase::W2 | SettledCareCase::Mixed, 0) => 5743,
                    (SettledCareCase::W2 | SettledCareCase::Mixed, _) => 3343,
                    (SettledCareCase::Grace, 0) => 5263,
                    (SettledCareCase::Grace, _) => 2863,
                    (SettledCareCase::Forfeited, 0) => 5023,
                    (SettledCareCase::Forfeited, _) => 2623,
                };
                assert_eq!(
                    crate::injured_spouse::separate_taxes(&again.draft, &projection),
                    Some(taxes)
                );

                let mut classified = again.draft.clone();
                classified.injured_spouse = Some(claim);
                let printed = supported(&classified);
                assert_eq!(printed.return_lines, ordinary.return_lines);
                let column = |line: &str, column: &str| {
                    printed
                        .lines
                        .iter()
                        .find(|row| row.id == alloc::format!("form_8379_line_{line}_{column}"))
                        .map(|row| row.amount / 100)
                };
                if matches!(case, SettledCareCase::W2 | SettledCareCase::Forfeited) {
                    let first = usize::from(injured == Owner::Spouse);
                    for (column_name, amount) in
                        [("a", income.iter().sum()), ("b", income[first]), ("c", income[1 - first])]
                    {
                        assert_eq!(column("13a", column_name), Some(amount));
                        assert_eq!(column("13b", column_name), Some(0));
                    }
                } else {
                    for line in ["13a", "13b"] {
                        for name in ["a", "b", "c"] {
                            assert_eq!(column(line, name), None);
                        }
                    }
                }
            }
        }
    }
}

pub(in crate::tests) fn niit_optional_column(
    result: &FederalResult,
    line: &str,
    column: &str,
) -> Option<i64> {
    let id = alloc::format!("form_8379_line_{line}_{column}");
    result.lines.iter().find(|row| row.id == id).map(|row| row.amount / 100)
}

pub(in crate::tests) fn assert_injured_ordinary_unchanged(
    draft: &FederalDraft,
    result: &FederalResult,
    calculate: fn(&FederalDraft) -> FederalResult,
) {
    let mut plain = draft.clone();
    plain.injured_spouse = None;
    let ordinary = calculate(&plain);
    assert_eq!(result.state, ordinary.state);
    assert_eq!(result.issues, ordinary.issues);
    super::injured_ordinary_money_unchanged(result, &ordinary);
}

pub(in crate::tests) fn assert_nonallocation_ordinary_unchanged(
    draft: &FederalDraft,
    result: &FederalResult,
) {
    let mut plain = draft.clone();
    plain.injured_spouse = None;
    let ordinary = supported(&plain);
    assert_eq!(
        (result.tax, result.refund, result.balance),
        (ordinary.tax, ordinary.refund, ordinary.balance)
    );
    assert_eq!(
        result.lines.iter().filter(|row| !row.id.starts_with("form_8379_")).collect::<Vec<_>>(),
        ordinary.lines.iter().filter(|row| !row.id.starts_with("form_8379_")).collect::<Vec<_>>()
    );
}

pub(in crate::tests) fn assert_portfolio_ordinary_unchanged(
    draft: &FederalDraft,
    result: &FederalResult,
) {
    assert_nonallocation_ordinary_unchanged(draft, result);
    assert_eq!(estimate(&restart(draft.clone()).draft), *result);
}

pub(in crate::tests) fn assert_native_money_unchanged(
    result: &FederalResult,
    ordinary: &FederalResult,
) {
    assert_eq!(result.return_lines, ordinary.return_lines);
    assert_eq!(
        (result.tax, result.refund, result.balance),
        (ordinary.tax, ordinary.refund, ordinary.balance)
    );
}

pub(super) fn assert_no_claim_native_money(draft: &FederalDraft, result: &FederalResult) {
    let mut plain = draft.clone();
    plain.injured_spouse = None;
    assert_native_money_unchanged(result, &supported(&plain));
}

pub(in crate::tests) fn assert_ebl_money(d: &FederalDraft, expected_ebl: i64) -> FederalResult {
    let result = supported(d);
    assert_eq!(value(&result, "excess_business_loss"), expected_ebl);
    assert_eq!(value(&result, "agi"), -432_000);
    assert_eq!(value(&result, "taxable_income"), 0);
    assert_eq!(value(&result, "self_employment_tax"), 0);
    assert_eq!(result.tax, Some(0));
    assert_eq!(result.refund, Some(550_000));
    let mut plain = d.clone();
    plain.injured_spouse = None;
    plain.checklist.whole_return &= !WholeReturnTopics::INJURED_SPOUSE.0;
    let ordinary = supported(&plain);
    assert_eq!(result.return_lines, ordinary.return_lines);
    assert_eq!((result.tax, result.refund), (ordinary.tax, ordinary.refund));
    result
}

pub(in crate::tests) fn assert_child_election_owner_source(owner: Owner, mixed: bool) {
    let original = child_election_owner_fixture(owner, mixed);
    let mut observed = Vec::new();
    let mut expected = Vec::new();
    for reversed in [false, true] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            let mut draft = original.clone();
            draft.injured_spouse.as_mut().unwrap().injured = injured;
            if reversed {
                draft.income.reverse();
            }
            let result = supported(&draft);
            // Independent native Form8814 arithmetic: mixed gross4200 gives
            // line6=1500,9=286,10=714,12=500,15=135. Interest2000 gives
            // no reported income and fixed tax65. Both keep wages80000.
            let (dividends, capital, other, child_tax, agi, tax) = if mixed {
                (286, 714, 500, 135, 81_500, 5_438)
            } else {
                (0, 0, 0, 65, 80_000, 5_308)
            };
            assert_eq!(value(&result, "agi"), agi);
            assert_eq!(return_line(&result, "3a") / 100, dividends);
            assert_eq!(return_line(&result, "3b") / 100, dividends);
            assert_eq!(return_line(&result, "7a") / 100, capital);
            assert_eq!(return_line(&result, "8") / 100, other);
            assert_eq!(value(&result, "child_election_tax"), child_tax);
            assert_eq!(return_line(&result, "16") / 100, tax);
            assert_portfolio_ordinary_unchanged(&draft, &result);
            let total = dividends + capital + other;
            let share = result
                .lines
                .iter()
                .find(|row| row.id == "form_8379_injured_share_of_tax")
                .map(|row| row.explanation.args[1..].to_vec());
            let taxes = match (mixed, owner) {
                (true, Owner::Taxpayer) => [5_218, 393],
                (true, Owner::Spouse) => [5_023, 578],
                (false, Owner::Taxpayer) => [5_088, 393],
                (false, Owner::Spouse) => [5_023, 458],
            };
            let index = usize::from(injured == Owner::Spouse);
            observed.push((
                reversed,
                injured,
                niit_optional_column(&result, "13b", "a"),
                niit_optional_column(&result, "13b", "b"),
                niit_optional_column(&result, "13b", "c"),
                share,
            ));
            expected.push((
                reversed,
                injured,
                Some(total),
                Some(if owner == injured { total } else { 0 }),
                Some(if owner != injured { total } else { 0 }),
                Some(vec![
                    MessageArg::usd_dollars(taxes[index]),
                    MessageArg::usd_dollars(taxes[1 - index]),
                ]),
            ));
        }
    }
    // Fixed child tax belongs to the actual elected source parent in the
    // separate-tax worksheet, even when there is no line13b income.
    assert_eq!(observed, expected, "actual child owner {owner:?}; mixed={mixed}");
}

pub(in crate::tests) fn assert_debt_owner_source(owner: Owner, transfer: bool, excluded: bool) {
    let original = personal_debt_owner_fixture(owner, transfer, excluded);
    let debt_id = original.income.last().unwrap().id;
    let mut observed = Vec::new();
    let mut expected = Vec::new();
    for reversed in [false, true] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            let mut draft = original.clone();
            draft.injured_spouse.as_mut().unwrap().injured = injured;
            if reversed {
                draft.income.reverse();
            }
            let result = supported(&draft);
            let (ordinary, capital, agi, tax) = if excluded {
                (0, 0, 80_000, 5_243)
            } else if transfer {
                (2_000, 2_000, 84_000, 5_483)
            } else {
                (5_000, 0, 85_000, 5_843)
            };
            assert_eq!(value(&result, "agi"), agi);
            assert_eq!(return_line(&result, "3a") / 100, 0);
            assert_eq!(return_line(&result, "3b") / 100, 0);
            assert_eq!(return_line(&result, "7a") / 100, capital);
            assert_eq!(return_line(&result, "8") / 100, ordinary);
            assert_eq!(return_line(&result, "16") / 100, tax);
            if transfer {
                assert_eq!(value(&result, &alloc::format!("debt_{debt_id}_transfer_gain")), 2_000);
            }
            assert_portfolio_ordinary_unchanged(&draft, &result);
            let total = ordinary + capital;
            observed.push((
                reversed,
                injured,
                niit_optional_column(&result, "13b", "a"),
                niit_optional_column(&result, "13b", "b"),
                niit_optional_column(&result, "13b", "c"),
            ));
            expected.push((
                reversed,
                injured,
                Some(total),
                Some(if owner == injured { total } else { 0 }),
                Some(if owner != injured { total } else { 0 }),
            ));
        }
    }
    // The transfer's native capital producer already exists; its2000 gain
    // must join the same owner's2000 COD without being duplicated as COD.
    // An actually excluded gift is a complete zero, not an unknown source.
    assert_eq!(
        observed, expected,
        "actual debt owner {owner:?}; transfer={transfer}; excluded={excluded}"
    );
}

pub(in crate::tests) fn assert_wage_classification_money(
    result: &FederalResult,
    case: WageClassificationCase,
) {
    let (wages, deferred, inmate, positive_medicaid, negative_medicaid, other, agi) = match case {
        // Native line1a rounds73999.50 to74000; the independently reported
        // Schedule1 line8t/8u rounds6000.50 to6001. AGI is80001, not80000.
        WageClassificationCase::Deferred => (74_000, 6001, 0, 0, 0, 6001, 80_001),
        WageClassificationCase::Inmate => (74_000, 0, 6001, 0, 0, 6001, 80_001),
        // Line1a71999; separate8t6001+8u2001+prize1+NEC1=8004.
        WageClassificationCase::Mixed => (71_999, 6001, 2001, 0, 0, 8004, 80_003),
        // Box1 Medicaid stays in1a and has only the negative8s source.
        WageClassificationCase::MedicaidBox1 => (80_000, 0, 0, 0, -1, 1, 80_001),
        // CodeII is outside box1: positive1d1 and negative8s−1 cancel,
        // alongside two separately rounded Other/1099 annual categories.
        WageClassificationCase::MedicaidCodeII => (80_000, 0, 0, 1, -1, 1, 80_002),
        // Positive1d:codeII50c+Other50c=100c→1. Negative8s includes
        // those100c PLUS box1Medicaid50c:150c→2, independently rounded.
        WageClassificationCase::MedicaidCombined => (80_000, 0, 0, 1, -2, 0, 80_001),
    };
    assert_eq!(return_line(result, "1a"), wages * 100);
    assert_eq!(return_line(result, "1d"), positive_medicaid * 100);
    assert_eq!(value(result, "deferred_pension"), deferred);
    assert_eq!(value(result, "inmate_pay"), inmate);
    assert_eq!(value(result, "medicaid_waiver_excluded"), negative_medicaid);
    assert_eq!(return_line(result, "8"), other * 100);
    assert_eq!(return_line(result, "9"), agi * 100);
    assert_eq!(value(result, "agi"), agi);
    assert_eq!(value(result, "taxable_income"), agi - 32_200);
    assert_eq!(value(result, "self_employment_tax"), 0);
    if !matches!(case, WageClassificationCase::Deferred | WageClassificationCase::Inmate) {
        assert_eq!(value(result, "taxable_prizes"), 1);
        assert_eq!(value(result, "hobby_sales_income"), 1);
    }
    // All six independently pinned taxable incomes remain in the actual
    //47800–47850 Tax Table row:2480+12%×(47825−24800)=5243.
    assert_eq!(return_line(result, "16"), 524_300);
    assert_eq!(result.tax, Some(524_300));
    assert_eq!(result.refund, Some(125_700));
}

pub(in crate::tests) fn expected_wage_classification_columns(
    case: WageClassificationCase,
    owner: Owner,
) -> ([i64; 2], [i64; 2]) {
    let mut wages = [60_000, 20_000];
    let index = usize::from(owner == Owner::Spouse);
    match case {
        // Native1a74000 + pure W-2 native8t/8u6001 remains W-2 source80001.
        WageClassificationCase::Deferred
        | WageClassificationCase::Inmate
        | WageClassificationCase::Mixed => wages[index] += 1,
        // The native pure W-2 exclusion−1 reduces its own source category.
        WageClassificationCase::MedicaidBox1 => wages[index] -= 1,
        // Pure codeII's native1d1/8s−1 cancels within the W-2 source.
        // Combined preserves its original dated hypothesis, not a new oracle.
        WageClassificationCase::MedicaidCodeII | WageClassificationCase::MedicaidCombined => {}
    }
    let (selected, opposite) = match case {
        WageClassificationCase::Deferred | WageClassificationCase::Inmate => (0, 0),
        WageClassificationCase::Mixed
        | WageClassificationCase::MedicaidBox1
        | WageClassificationCase::MedicaidCodeII => (1, 1),
        WageClassificationCase::MedicaidCombined => (0, 1),
    };
    let mut others = [0; 2];
    others[index] = selected;
    others[1 - index] = opposite;
    (wages, others)
}

pub(in crate::tests) fn assert_noninvestment_election_money(d: &FederalDraft) -> FederalResult {
    let r = supported(d);
    assert_eq!(value(&r, "agi"), 93_000);
    assert_eq!(value(&r, "taxable_income"), 60_800);
    assert_eq!(value(&r, "capital_gain"), 13_000);
    assert_eq!(value(&r, "investment_interest_net_capital_gain"), 10_000);
    assert_eq!(value(&r, "investment_elected_capital_gain"), 10_000);
    // Ordinary57800 uses filing-table midpoint57825: 2480+33025*.12
    // rounds6443. The remaining3000 is in the joint0% gain band.
    assert_eq!(return_line(&r, "16"), 644_300);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
    assert_eq!(r.tax, Some(644_300));
    assert_eq!(r.refund, Some(2_355_700));
    r
}

pub(in crate::tests) fn assert_noninvestment_election_taxes(r: &FederalResult, injured: Owner) {
    assert!(
        !r.review_notes
            .iter()
            .any(|note| note.display.key == "WARN_8379_INVESTMENT_ELECTION_OWNER_UNAVAILABLE")
    );
    let explanation =
        &r.lines.iter().find(|row| row.id == "form_8379_injured_share_of_tax").unwrap().explanation;
    // Original native single-owner election6576; the separately owned
    // noninvestment3000 is preferential at15%=450, not elected again.
    let expected = if injured == Owner::Taxpayer { [7026, 393] } else { [393, 7026] };
    assert_eq!(explanation.args[1], MessageArg::usd_dollars(expected[0]));
    assert_eq!(explanation.args[2], MessageArg::usd_dollars(expected[1]));
}
