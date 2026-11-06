use crate::k1_activity::*;
use crate::tests::cross::interview::{accepted, document};
use crate::tests::support::assert::k1res::assert_atomic_rejection;
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::builders::k1res::{
    documents, entity_dividend_borrowing, entity_dividend_estate, entity_dividend_index,
    entity_dividend_output, entity_dividend_parts, graph, input, set_entity_dividends,
};
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::support::client::k1res::{
    entity_dividend_session, entity_dividend_session_for_owner,
};
use crate::tests::*;

#[test]
fn ordinary_basis_once_qualified_subset_and_independent_risk_passive_sources() {
    for scorp in [false, true] {
        let session = entity_dividend_session(
            scorp,
            complete(FederalStatus::Single),
            true,
            [100_000, 60_000],
            Some([150_000, 80_000]),
        );
        let entry = &session.draft.income[entity_dividend_index(&session.draft)].income;
        let regular = entity_dividend_output(entry, false).unwrap();
        let alternative = entity_dividend_output(entry, true).unwrap();
        // Opening1000 + activity4000 + ordinary1000 =6000 against7000 loss.
        // Qualified600 is already in ordinary1000. AMT ordinary1500 yields6500.
        assert_eq!(
            (
                regular.components[1].basis_allowed.get(),
                regular.components[1].basis_suspended.get()
            ),
            (600_000, 100_000)
        );
        assert_eq!(
            (
                alternative.components[1].basis_allowed.get(),
                alternative.components[1].basis_suspended.get()
            ),
            (650_000, 50_000)
        );
        assert_eq!(
            (regular.components[1].risk_allowed.get(), regular.components[1].risk_suspended.get()),
            (50_000, 650_000)
        );
        assert_eq!(
            (
                alternative.components[1].risk_allowed.get(),
                alternative.components[1].risk_suspended.get()
            ),
            (50_000, 700_000)
        );
        assert_eq!(
            (regular.groups[0].passive, regular.groups[1].nonpassive, regular.groups[1].prior),
            (400_000, -50_000, 200_000)
        );
        assert_eq!(
            (
                alternative.groups[0].passive,
                alternative.groups[1].nonpassive,
                alternative.groups[1].prior
            ),
            (400_000, -50_000, 200_000)
        );
        let mut full_subset = entry.clone();
        set_entity_dividends(&mut full_subset, [100_000, 100_000], Some([150_000, 150_000]));
        assert!(crate::k1_activity::income_issue(&full_subset).is_none());
        assert_eq!(
            entity_dividend_output(&full_subset, false).unwrap().components[1].basis_allowed,
            regular.components[1].basis_allowed
        );
        assert_eq!(
            entity_dividend_output(&full_subset, true).unwrap().components[1].basis_allowed,
            alternative.components[1].basis_allowed
        );
        let result = supported(&session.draft);
        assert_eq!(return_line(&result, "3b"), 100_000);
        assert_eq!(return_line(&result, "3a"), 60_000);
    }
}

#[test]
fn native_preview_running_save_reopen_unchanged_restart_correction_and_allocator() {
    for scorp in [false, true] {
        let mut draft = complete(FederalStatus::Single);
        draft.income[0] = wage(1, Owner::Taxpayer, 15_000_000, 0);
        let mut session = entity_dividend_session(scorp, draft, false, [0, 0], None);
        let index = entity_dividend_index(&session.draft);
        let before = session.draft.clone();
        let dirty = session.dirty;
        let mut candidate = before.income[index].income.clone();
        set_entity_dividends(&mut candidate, [100_000, 60_000], None);
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document = input(&candidate);

        assert!(matches!(
            crate::tests::support::client::income_pages::candidate_save_review(
                &mut session,
                crate::tests::support::client::estimate_documents_from_save(&request)
            ),
            Reply::Response(P::Review { .. })
        ));
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        accepted(&send_fixture(&mut session, request));
        let saved = session.draft.clone();
        assert_eq!(saved.revision, before.revision + 2);
        assert_eq!(saved.next_id, before.next_id);
        assert_eq!(saved.income[0], before.income[0]);
        let result = supported(&saved);
        // 2026 single134900 taxable: ordinary24974;600 at15% rather than24% saves54.
        assert_eq!(value(&result, "income_tax"), 24_920);
        assert_eq!((return_line(&result, "3b"), return_line(&result, "3a")), (100_000, 60_000));
        let mut cold = restart(saved.clone());
        let mut reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        assert_eq!(
            documents(&mut reopened)[0].document,
            expected_input(&saved, &saved.income[index])
        );
        accepted(&send_fixture(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
        assert_eq!(estimate(&cold.draft), result);
        let mut corrected = saved.income[index].income.clone();
        set_entity_dividends(&mut corrected, [100_000, 0], Some([100_000, 60_000]));
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        documents(&mut request)[0].document = input(&corrected);
        accepted(&send_fixture(&mut cold, request));
        let corrected_result = supported(&cold.draft);
        assert_eq!(value(&corrected_result, "income_tax"), 24_974);
        assert_eq!(
            entity_dividend_output(&cold.draft.income[index].income, true)
                .unwrap()
                .entity_qualified_dividends,
            60_000
        );
        let expected_id = cold.draft.next_id;
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut cold,
            vec![document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(
                        native_int1099("Synthetic next source".into(), 0),
                    )),
                },
            )],
        );
        accepted(&send_fixture(&mut cold, request));
        assert_eq!(cold.draft.next_id, expected_id + 1);
        assert!(cold.draft.income.iter().any(|entry| entry.id == expected_id));
        assert_eq!(cold.draft.income[index].income, corrected);
        assert_eq!(estimate(&restart(cold.draft.clone()).draft), estimate(&cold.draft));
    }
}

#[test]
fn ordinary_and_qualified_category_rounding_zero_full_subset_and_direct_combination() {
    for scorp in [false, true] {
        for (cents, expected) in [(0, 0), (49, 0), (50, 1), (51, 1)] {
            let session = entity_dividend_session(
                scorp,
                complete(FederalStatus::Single),
                false,
                [cents, cents],
                None,
            );
            let result = supported(&session.draft);
            assert_eq!(return_line(&result, "3b"), expected * 100);
            assert_eq!(return_line(&result, "3a"), expected * 100);
            let index = entity_dividend_index(&session.draft);
            assert_eq!(
                entity_dividend_output(&session.draft.income[index].income, false)
                    .unwrap()
                    .basis
                    .ending
                    .get(),
                100_000 + cents
            );
        }
        let mut session = entity_dividend_session(
            scorp,
            complete(FederalStatus::Single),
            false,
            [100_025, 60_025],
            None,
        );
        add(
            &mut session.draft,
            income(FederalIncome::Dividend {
                data: dividend_source(native_dividend(100_025, 60_025, 0)),
            }),
        );
        let result = supported(&session.draft);
        assert_eq!((return_line(&result, "3b"), return_line(&result, "3a")), (200_100, 120_100));
        assert_eq!(return_line(&result, "2b"), 0);
        assert_eq!(value(&result, "self_employment_tax"), 0);
    }
}

#[test]
fn independent_amt_ordinary_qualified_and_both_feed_existing_4952_and_preference() {
    for scorp in [false, true] {
        for (amt, ordinary_capacity, amt_basis) in [
            ([200_000, 60_000], 1_400, 3_000),
            ([100_000, 80_000], 200, 2_000),
            ([200_000, 80_000], 1_200, 3_000),
        ] {
            let mut draft = complete(FederalStatus::Single);
            entity_dividend_borrowing(&mut draft);
            let session =
                entity_dividend_session(scorp, draft, false, [100_000, 60_000], Some(amt));
            let result = supported(&session.draft);
            assert_eq!(value(&result, "investment_interest_deduction"), 400);
            assert_eq!(value(&result, "amt_investment_interest_deduction"), ordinary_capacity);
            assert_eq!(value(&result, "amt_investment_form_line4a"), amt[0] / 100);
            assert_eq!(value(&result, "amt_investment_form_line4b"), amt[1] / 100);
            assert_eq!(value(&result, "amt_investment_form_line4c"), ordinary_capacity);
            assert_eq!(
                value(&result, "amt_investment_interest_carryforward"),
                3_000 - ordinary_capacity
            );
            let index = entity_dividend_index(&session.draft);
            let id = session.draft.income[index].id;
            let output = entity_dividend_output(&session.draft.income[index].income, true).unwrap();
            assert_eq!(output.basis.ending.get(), amt_basis * 100);
            assert_eq!(output.entity_qualified_dividends, amt[1]);
            let mut unborrowed = session.draft.clone();
            unborrowed.deductions.as_mut().unwrap().itemize_override = None;
            unborrowed.deductions.as_mut().unwrap().investment_interest = None;
            let unborrowed = supported(&unborrowed);
            assert_eq!(
                value(&unborrowed, "alternative_minimum_taxable_income"),
                60_000 + amt[0] / 100
            );
            let prefix = if scorp { "scorp" } else { "partnership" };
            assert_eq!(
                line_amount(&result, &alloc::format!("amt_{prefix}_{id}_dividends")),
                amt[0]
            );
            assert_eq!(
                line_amount(&result, &alloc::format!("amt_{prefix}_{id}_qualified_dividends")),
                amt[1]
            );
        }
    }
}

#[test]
fn mixed_estate_qualification_preserves_aggregate_amt_cent_crossings_in_both_directions() {
    for scorp in [false, true] {
        for (regular_o, amt_o, estate_q, amt_q, expected_q, expected_4a) in [
            (100_049, 100_050, 20_049, 60_001, 901, 2_101),
            (100_050, 100_049, 20_050, 59_999, 900, 2_100),
        ] {
            let mut draft = complete(FederalStatus::Single);
            entity_dividend_borrowing(&mut draft);
            let mut session = entity_dividend_session(
                scorp,
                draft,
                false,
                [regular_o, 60_000],
                Some([amt_o, amt_q]),
            );
            add(
                &mut session.draft,
                income(FederalIncome::EstateTrustPortfolio {
                    data: crate::tests::support::builders::estate_source(entity_dividend_estate(
                        estate_q,
                    )),
                }),
            );
            let result = supported(&session.draft);
            assert_eq!(value(&result, "investment_interest_deduction"), 1_200);
            assert_eq!(value(&result, "amt_investment_form_line4a"), expected_4a);
            assert_eq!(value(&result, "amt_investment_form_line4b"), expected_q);
            assert_eq!(value(&result, "amt_investment_form_line4c"), 1_200);
            assert_eq!(value(&result, "amt_investment_interest_deduction"), 1_200);
        }
    }
}

#[test]
fn dividends_feed_niit_eic_and_social_security_once() {
    for scorp in [false, true] {
        let mut draft = complete(FederalStatus::Single);
        draft.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 0);
        let session = entity_dividend_session(scorp, draft, false, [100_000, 60_000], None);
        let result = supported(&session.draft);
        assert_eq!(value(&result, "form_8960_net_investment_income"), 1_000);
        assert_eq!(value(&result, "net_investment_income_tax"), 38);
        assert_eq!(value(&result, "eic_investment_income"), 1_000);
        let mut draft = complete(FederalStatus::Single);
        draft.income[0] = wage(1, Owner::Taxpayer, 2_400_000, 0);
        add(&mut draft, income(social_security_net(1_000_000, 0)));
        let mut session = entity_dividend_session(scorp, draft, false, [0, 0], None);
        let before = supported(&session.draft);
        assert_eq!(
            (value(&before, "taxable_social_security"), value(&before, "agi")),
            (2_000, 26_000)
        );
        let index = entity_dividend_index(&session.draft);
        set_entity_dividends(&mut session.draft.income[index].income, [200_000, 0], None);
        assert!(crate::k1_activity::income_issue(&session.draft.income[index].income).is_none());
        let result = supported(&session.draft);
        assert_eq!(
            (value(&result, "taxable_social_security"), value(&result, "agi")),
            (3_000, 29_000)
        );
    }
}

#[test]
fn joint_niit_and_injured_spouse_use_actual_source_owner_in_both_directions() {
    for scorp in [false, true] {
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let draft = injured_return((200_000, 100_000), (50_000, 30_000), 0, allocation());
            let mut session = entity_dividend_session_for_owner(
                scorp,
                draft,
                owner,
                false,
                [100_000, 60_000],
                None,
            );
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                session.draft.injured_spouse.as_mut().unwrap().injured = injured;
                let result = supported(&session.draft);
                assert_eq!(value(&result, "form_8960_net_investment_income"), 1_000);
                assert_eq!(value(&result, "net_investment_income_tax"), 38);
                assert_eq!(
                    niit_optional_column(&result, "13b", "b"),
                    Some(if owner == injured { 1_000 } else { 0 })
                );
                assert_eq!(
                    niit_optional_column(&result, "18", "b"),
                    Some(if owner == injured { 38 } else { 0 })
                );
                assert_injured_ordinary_unchanged(&session.draft, &result, supported);
            }
        }
    }
}

#[test]
fn unknown_sources_are_validated_then_unavailable_without_erasing_known_facts() {
    for scorp in [false, true] {
        let session = entity_dividend_session(
            scorp,
            complete(FederalStatus::Single),
            false,
            [100_000, 60_000],
            Some([150_000, 80_000]),
        );
        let index = entity_dividend_index(&session.draft);
        for boundary in 0..10 {
            let mut draft = session.draft.clone();
            let income = &mut draft.income[index].income;
            let (source, _, alternative) = entity_dividend_parts(income);
            match boundary {
                0 => source.entity_only = None,
                1 => source.alternative_entity_only = K1EntityOnlyAmt::Unavailable,
                2 => *alternative = None,
                3 => source.components[0].alternative = K1StatementAmt::Unavailable,
                4 => source.risk_groups[0].source = K1RiskSource::Unavailable,
                5 => source.passive_groups[0].participation = K1ActivityParticipation::Unavailable,
                6 => source.qbi_blocks[0].allocations = None,
                7 => source.entity_only.as_mut().unwrap().charity_basis_carry = None,
                8 => {
                    source.entity_only.as_mut().unwrap().capital_carry =
                        K1StatementCapitalCarry::Unavailable
                }
                9 => source.components[0].regular = None,
                _ => unreachable!(),
            }
            // These are validated estimate candidates, not claims of native Saved Unknowns.
            assert!(
                crate::k1_activity::income_issue(income).is_none(),
                "{scorp} boundary{boundary}"
            );
            if matches!(boundary, 1..=3) {
                let regular = entity_dividend_output(income, false).unwrap();
                assert_eq!(
                    (regular.entity_dividends, regular.entity_qualified_dividends),
                    (100_000, 60_000)
                );
            }
            let result = estimate(&draft);
            assert!(result.tax.is_none(), "{scorp} boundary{boundary}");
            assert_eq!(
                result
                    .issues
                    .iter()
                    .filter(
                        |issue| issue.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE"
                    )
                    .count(),
                1,
                "{scorp} boundary{boundary}: {:?}",
                result.issues
            );
            assert_eq!(
                entity_dividend_output(&session.draft.income[index].income, false)
                    .unwrap()
                    .entity_dividends,
                100_000
            );
        }
    }
}

#[test]
fn invalid_subset_negative_reconciliation_and_duplicate_ids_fail_save_atomically() {
    for scorp in [false, true] {
        let mut session = entity_dividend_session(
            scorp,
            complete(FederalStatus::Single),
            false,
            [100_000, 60_000],
            Some([150_000, 80_000]),
        );
        for invalid in 1..5 {
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            let document = &mut documents(&mut request)[0].document;
            let source = graph(document);
            let key = match invalid {
                1 => {
                    source.entity_only.as_mut().unwrap().portfolio.qualified_dividends = 100_001;
                    "K1_ACTIVITY_SOURCE_AMOUNT_INVALID"
                }
                2 => {
                    let K1EntityOnlyAmt::Separate { amounts } = &mut source.alternative_entity_only
                    else {
                        unreachable!()
                    };
                    amounts.portfolio.qualified_dividends = 150_001;
                    "K1_ACTIVITY_SOURCE_AMOUNT_INVALID"
                }
                3 => {
                    source.entity_only.as_mut().unwrap().portfolio.dividends += 1;
                    "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE"
                }
                4 => {
                    source.components[1].id = source.components[0].id;
                    "K1_ACTIVITY_SOURCE_ID_REPEATED"
                }
                _ => unreachable!(),
            };
            assert_atomic_rejection(&mut session, request, key);
        }
    }
}

#[test]
fn differing_amt_qualified_amount_reaches_existing_6251_preferential_engine() {
    for scorp in [false, true] {
        let mut draft = complete(FederalStatus::Single);
        // 2026-10-18: Form 6251 lines 19–23 use REGULAR ordinary income for the 0% band.
        // https://www.irs.gov/pub/irs-dft/f6251--dft.pdf
        // Explicit $100,000 wages fill that band; AMT qualified income then uses 15%.
        draft.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
        draft.screening.as_mut().unwrap().iso_exercises.push(iso_exercise(
            "Synthetic held ISO shares",
            1.0,
            100,
            20_000_100,
        ));
        let mut session = entity_dividend_session(
            scorp,
            draft,
            false,
            [100_000, 60_000],
            Some([100_000, 60_000]),
        );
        let before = supported(&session.draft);
        let index = entity_dividend_index(&session.draft);
        set_entity_dividends(
            &mut session.draft.income[index].income,
            [100_000, 60_000],
            Some([100_000, 80_000]),
        );
        assert!(crate::k1_activity::income_issue(&session.draft.income[index].income).is_none());
        let after = supported(&session.draft);
        assert_eq!(value(&before, "alternative_minimum_taxable_income"), 301_000);
        assert_eq!(value(&after, "alternative_minimum_taxable_income"), 301_000);
        assert_eq!(value(&before, "amt_capital_line_13"), 600);
        assert_eq!(value(&after, "amt_capital_line_13"), 800);
        // Same AMTI: another200 of qualified income uses15% rather than26%, saving22.
        assert_eq!(
            value(&before, "amt_tax_before_foreign_credit")
                - value(&after, "amt_tax_before_foreign_credit"),
            22
        );
        assert_eq!(return_line(&after, "3a"), 60_000);
    }
}

#[test]
fn entity_characters_preserve_unsupported_sources_and_compose_positive_capital() {
    for scorp in [false, true] {
        let session = entity_dividend_session(
            scorp,
            complete(FederalStatus::Single),
            false,
            [100_000, 60_000],
            None,
        );
        let index = entity_dividend_index(&session.draft);
        // 2026-10-20: IRS i1065sk1/i7203 include interest with dividends
        // and positive capital income in basis; royalties retain their own boundary.
        for character in [0, 2, 3, 4, 5, 6] {
            let mut draft = session.draft.clone();
            let income = &mut draft.income[index].income;
            if character == 5 {
                set_entity_dividends(income, [100_000, 60_000], Some([0, 0]));
            } else if character == 6 {
                set_entity_dividends(income, [0, 0], Some([100_000, 60_000]));
            }
            let (source, current, alternative) = entity_dividend_parts(income);
            let entity = source.entity_only.as_mut().unwrap();
            match character {
                0 => {
                    entity.portfolio.royalties = 1;
                    current.portfolio.royalties = 1;
                }
                2 => {
                    entity.portfolio.interest = 1;
                    current.portfolio.interest = 1;
                }
                3 => {
                    entity.portfolio.short = 1;
                    current.portfolio.short = 1;
                }
                4 => {
                    entity.portfolio.long = 1;
                    current.portfolio.long = 1;
                }
                5 => {
                    let K1EntityOnlyAmt::Separate { amounts } = &mut source.alternative_entity_only
                    else {
                        unreachable!()
                    };
                    amounts.portfolio.short = 1;
                    alternative.as_mut().unwrap().portfolio.short = 1;
                }
                6 => {
                    entity.portfolio.short = 1;
                    current.portfolio.short = 1;
                }
                _ => unreachable!(),
            }
            // SameAsRegular is an affirmative source assertion: its control also changes.
            if character < 5 {
                let (_, current, alternative) = entity_dividend_parts(income);
                *alternative = Some(current.clone());
            }
            assert!(crate::k1_activity::income_issue(income).is_none());
            if character >= 2 {
                for amt in [false, true] {
                    let output = entity_dividend_output(income, amt).unwrap();
                    let dividends = if (character == 5 && amt) || (character == 6 && !amt) {
                        [0, 0]
                    } else {
                        [100_000, 60_000]
                    };
                    let capital = match (character, amt) {
                        (2, _) => [0, 0],
                        (4, _) => [0, 1],
                        (5, false) | (6, true) => [0, 0],
                        _ => [1, 0],
                    };
                    let interest = i64::from(character == 2);
                    assert_eq!(output.entity_interest, interest);
                    assert_eq!(output.entity_capital, capital);
                    assert_eq!(output.entity_dividends, dividends[0]);
                    assert_eq!(output.entity_qualified_dividends, dividends[1]);
                    assert_eq!(
                        output.basis.ending.get(),
                        100_000 + dividends[0] + capital.iter().sum::<i64>() + interest
                    );
                }
                let result = supported(&draft);
                assert_eq!(return_line(&result, "2b"), 0);
                assert_eq!(return_line(&result, "3b"), if character == 6 { 0 } else { 100_000 });
                assert_eq!(return_line(&result, "3a"), if character == 6 { 0 } else { 60_000 });
                assert_eq!(value(&result, "agi"), if character == 6 { 60_000 } else { 61_000 });
                continue;
            }
            assert!(entity_dividend_output(income, false).is_err());
            let result = estimate(&draft);
            assert!(result.tax.is_none());
            assert_eq!(
                result
                    .issues
                    .iter()
                    .filter(
                        |issue| issue.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE"
                    )
                    .count(),
                1
            );
        }
    }
}
