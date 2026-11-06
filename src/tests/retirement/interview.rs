use crate::tests::support::client::income_pages::open_topic;
// Record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::*;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::{document, owned};
use super::support::client::income_pages::{
    open_documents, page_documents, save_each_topic, save_topic,
};
use crate::tests::cross::interview::{
    PAGE_ADJUSTMENTS, PAGE_CREDITS, PAGE_REVIEW, assert_available_sections,
    at_the_adjustments_hub_with_business, at_the_credits_hub, continued, credits_hub, early_kept,
    excess_entry, go_on, go_to, hsa_account, hub_rows, ira_row, normal_kept, open_adjustments_hub,
    open_credits_topic, open_hsa_page, open_ira_page, other_1099, r1099, roth_item_for_tests,
    row_of, se_health_test_year, send_past_less_common, value_of, walk_on_to_review,
    with_form_8606,
};
const PAGE_SAVER: i32 = TaxSection::Saver as i32;
pub(in crate::tests) fn retirement_treatment(
    code: DistributionCode,
    ira: bool,
) -> RetirementTreatment {
    crate::FederalRetirement::from(&r1099(1_000_000, code, ira).into_source())
        .ordinary_distribution
        .expect("fixture supplied its treatment")
}

#[test]
fn distribution_codes_map_onto_treatments() {
    // 7, kept.
    assert_eq!(
        retirement_treatment(DistributionCode::Normal { what_happened: Disposition::Kept }, false),
        RetirementTreatment::Ordinary
    );
    // 7, rolled over in full to a traditional IRA.
    assert_eq!(
        retirement_treatment(
            DistributionCode::Normal {
                what_happened: Disposition::RolledOver {
                    amount: 1_000_000,
                    into: crate::RolloverDestination::Ira,
                },
            },
            true
        ),
        RetirementTreatment::Rollover {
            destination: crate::RolloverDestination::Ira,
            amount: 1_000_000,
            early: false,
        }
    );
    // 1 or 2, kept with an exception.
    assert_eq!(
        retirement_treatment(
            DistributionCode::Early {
                what_happened: Disposition::Kept,
                exception: EarlyException::Applies {
                    second: None,
                    repaid: None,
                    distributed: 20260601,
                    facts: crate::RetirementException::Disability { began: 20260101 },
                },
            },
            false
        ),
        RetirementTreatment::Exception {
            second: None,
            distributed: 20260601,
            facts: crate::RetirementException::Disability { began: 20260101 },
            early_simple: false,
        }
    );
    // G, direct rollover to an employer plan.
    // (Box 2a is 0 for pretax money rolled to a traditional IRA or plan; a taxable box 2a on
    // code G is a Roth rollover and stays ordinary income, see line_audit_f10.rs.)
    let mut direct = r1099(
        1_000_000,
        DistributionCode::DirectRollover { to: RolloverTarget::EmployerPlan },
        false,
    );
    direct.source_mut().form.taxable = Some(0);
    let data = RetirementSource::R1099(direct.into_source());
    assert_eq!(
        crate::FederalRetirement::from(&data).ordinary_distribution,
        Some(RetirementTreatment::Rollover {
            destination: crate::RolloverDestination::EmployerPlan,
            amount: 1_000_000,
            early: false,
        })
    );
    // S, kept.
    assert_eq!(
        retirement_treatment(
            DistributionCode::EarlySimple {
                what_happened: SimpleDisposition::Kept { exception: EarlyException::None }
            },
            true
        ),
        RetirementTreatment::EarlySimple
    );
    // D, a nonqualified annuity.
    let data = RetirementSource::R1099(
        r1099(1_000_000, DistributionCode::Nonqualified, false).into_source(),
    );
    assert_eq!(crate::FederalRetirement::from(&data).kind, RetirementKind::Nonqualified);
    assert_eq!(
        crate::FederalRetirement::from(&data).ordinary_distribution,
        Some(RetirementTreatment::Ordinary)
    );
}

#[test]
fn form_8606_switches_that_owners_iras_to_the_basis_worksheet() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let ira_r1099 =
        r1099(1_000_000, DistributionCode::Normal { what_happened: Disposition::Kept }, true);
    let mut ira_r1099 = ira_r1099;
    ira_r1099.source_mut().form.taxable = Some(800_000);
    {
        accepted(&crate::tests::support::client::save_retirement_fixture(
            &mut session,
            vec![document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(ira_r1099.clone())),
                },
            )],
        ));
    };
    let entry_id = session.draft.income[1].id;
    let FederalIncome::Retirement { data } = &session.draft.income[1].income else {
        unreachable!()
    };
    // Without a Form 8606, the IRA distribution uses the payer's box 2a.
    assert_eq!(
        (
            crate::FederalRetirement::from(data).taxable.clone(),
            crate::FederalRetirement::from(data).box_2a
        ),
        (Some(RetirementTaxable::Payer), 800_000)
    );

    // Adding this owner's Form 8606 switches the same IRA distribution to
    // the basis worksheet, never asking the box 2a question again.
    {
        let request = save_topic(
            &mut session,
            TaxSection::Retirement,
            vec![document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::Form8606(Form8606 {
                        adjustment: 0,
                        reservist_repayments: vec![],
                        basis: 500_000,
                        year_end: 900_000,
                    }),
                },
            )],
        );
        accepted(&send(&mut session, request));
    };
    let FederalIncome::Retirement { data } = &session.draft.income[1].income else {
        unreachable!()
    };
    assert_eq!(crate::FederalRetirement::from(data).taxable, Some(RetirementTaxable::IraBasis));
    assert_eq!(
        session.draft.adjustments.as_ref().unwrap().ira_basis,
        vec![crate::IraBasis {
            owner: Owner::Taxpayer,
            prior: 500_000,
            year_end: 900_000,
            paid_next_year: 0,
            adjustment: 0,
            reservist_repayments: vec![],
            special_repayments: vec![],
        }]
    );

    // Remove the basis facts before editing the source's now-editable box 2a.
    let request = save_topic(&mut session, TaxSection::Retirement, vec![]);
    accepted(&send(&mut session, request));
    ira_r1099.source_mut().form.taxable = Some(750_000);
    {
        accepted(&crate::tests::support::client::save_retirement_fixture(
            &mut session,
            vec![document(
                entry_id,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(ira_r1099)),
                },
            )],
        ));
    };
    let FederalIncome::Retirement { data } = &session.draft.income[1].income else {
        unreachable!()
    };
    assert_eq!(
        (
            crate::FederalRetirement::from(data).taxable.clone(),
            crate::FederalRetirement::from(data).box_2a
        ),
        (Some(RetirementTaxable::Payer), 750_000)
    );
    assert!(session.draft.adjustments.as_ref().unwrap().ira_basis.is_empty());
}

#[test]
fn form_8606_twice_for_the_same_owner_is_rejected() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let form =
        Form8606 { adjustment: 0, reservist_repayments: vec![], basis: 100_000, year_end: 200_000 };
    let reply = {
        let request = save_topic(
            &mut session,
            TaxSection::Retirement,
            vec![
                document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::Form8606(form.clone()) },
                ),
                document(0, IncomeSource::Record { value: crate::IncomeRecord::Form8606(form) }),
            ],
        );
        send(&mut session, request)
    };
    rejected_with(reply, "FORM_8606_TWICE");
    assert_eq!(session.draft, original);
}

#[test]
fn qcd_gifts_group_by_owner() {
    let original = complete(FederalStatus::Joint);
    let mut session = Interview::new(TEST_USER, original.clone());
    {
        let request = save_topic(
            &mut session,
            TaxSection::Retirement,
            vec![
                owned(
                    Owner::Taxpayer,
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::Qcd(QcdGift {
                            split_interest: false,
                            date: 20260301,
                            amount: 100_000,
                            inherited: None,
                        }),
                    },
                ),
                owned(
                    Owner::Taxpayer,
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::Qcd(QcdGift {
                            split_interest: false,
                            date: 20260601,
                            amount: 50_000,
                            inherited: None,
                        }),
                    },
                ),
                owned(
                    Owner::Spouse,
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::Qcd(QcdGift {
                            split_interest: false,
                            date: 20260815,
                            amount: 75_000,
                            inherited: None,
                        }),
                    },
                ),
            ],
        );
        send(&mut session, request)
    };
    let charities = &session.draft.adjustments.as_ref().unwrap().qualified_charity;
    assert_eq!(charities.len(), 2);
    let yours = charities.iter().find(|c| c.owner == Owner::Taxpayer).unwrap();
    assert_eq!(yours.payments.len(), 2);
    assert_eq!(yours.payments[0].amount, 100_000);
    assert_eq!(yours.payments[1].amount, 50_000);
    let spouses = charities.iter().find(|c| c.owner == Owner::Spouse).unwrap();
    assert_eq!(spouses.payments.len(), 1);
    assert_eq!(spouses.payments[0].amount, 75_000);
    assert!(charities.iter().all(|c| c.prior_deductions == 0));
}

#[test]
fn r1099_box_relationships_reject_on_both_sides() {
    let original = complete(FederalStatus::Single);
    let reject = |r: R1099Answered, key: &str| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = {
            crate::tests::support::client::save_retirement_fixture(
                &mut session,
                vec![document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::R1099(retirement_document(r)),
                    },
                )],
            )
        };
        let Reply::Response(P::Problem { message, .. }) = reply else {
            panic!("expected a problem for {key}, got {reply:?}")
        };
        assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
        assert_eq!(session.draft, original);
    };
    // Box 2a cannot exceed box 1.
    let mut over_taxable =
        r1099(1_000_000, DistributionCode::Normal { what_happened: Disposition::Kept }, false);
    over_taxable.source_mut().form.taxable = Some(1_000_001);
    reject(over_taxable, "R_TAXABLE_ABOVE_GROSS");
    // Box 4 cannot exceed box 1.
    let mut over_withheld =
        r1099(1_000_000, DistributionCode::Normal { what_happened: Disposition::Kept }, false);
    over_withheld.source_mut().form.withholding = Some(1_000_001);
    reject(over_withheld, "R_WITHHOLDING_ABOVE_GROSS");
    // Box 9a is accepted with every treatment: boxes 1 and 2a are already this recipient's share
    // (the 1099-R instructions, Beneficiaries), and the percentage is stored only on a Form 4972
    // record. It is entered as printed, with up to two decimals; converted once to exact
    // hundredths, 33.33% is 3333 and a $1,000,000.00 (100_000_000 cents) distribution gives
    // 100_000_000 * 3333 / 10_000 = 33_330_000 cents, $333,300.00. The 0.01 and 99.99 ends are 1
    // and 9_999 hundredths; 100.00 is outside the range, since box 9a is printed only for a
    // distribution to more than one person.
    assert_eq!((33.33_f64 * 100.0).round() as i64, 3333);
    assert_eq!(100_000_000_i64 * 3333 / 10_000, 33_330_000);
    assert_eq!((0.01_f64 * 100.0).round() as i64, 1);
    assert_eq!((99.99_f64 * 100.0).round() as i64, 9_999);
    for percent in [0.01, 33.33, 50.0, 99.99] {
        let mut with_box_9a =
            r1099(1_000_000, DistributionCode::Normal { what_happened: Disposition::Kept }, false);
        with_box_9a.source_mut().form.box_9a_percent = Some(percent);
        let mut session = Interview::new(TEST_USER, original.clone());
        accepted(&{
            crate::tests::support::client::save_retirement_fixture(
                &mut session,
                vec![document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::R1099(retirement_document(with_box_9a)),
                    },
                )],
            )
        });
    }
    // Box 10 is for a designated Roth account distribution (code B), and no more than box 1.
    let mut with_box_10 =
        r1099(1_000_000, DistributionCode::Normal { what_happened: Disposition::Kept }, false);
    with_box_10.source_mut().form.box_10 = Some(1);
    reject(with_box_10.clone(), "R_BOX10_WITHOUT_B");
    let mut designated = printed_as(with_box_10, "7B");
    let mut session = Interview::new(TEST_USER, original.clone());
    accepted(&{
        crate::tests::support::client::save_retirement_fixture(
            &mut session,
            vec![document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(
                        designated.clone(),
                    )),
                },
            )],
        )
    });
    designated.source_mut().form.box_10 = Some(1_000_001);
    reject(designated, "R_BOX10_ABOVE_GROSS");
}

#[test]
fn box_7a_rules_reject_on_both_sides_at_their_boundaries() {
    let original = complete(FederalStatus::Single);
    let reject = |r: R1099Answered, key: &str| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = {
            crate::tests::support::client::save_retirement_fixture(
                &mut session,
                vec![document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::R1099(retirement_document(r)),
                    },
                )],
            )
        };
        let Reply::Response(P::Problem { message, .. }) = reply else {
            panic!("expected a problem for {key}, got {reply:?}")
        };
        assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
        assert_eq!(session.draft, original);
    };
    let accept = |r: R1099Answered| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = {
            crate::tests::support::client::save_retirement_fixture(
                &mut session,
                vec![document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::R1099(retirement_document(r)),
                    },
                )],
            )
        };
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    };
    // No code, then three codes (two is the most).
    reject(other_1099(1_000_000, "", false), "R_BOX7_NO_CODE");
    reject(other_1099(1_000_000, "18B", false), "R_BOX7_TOO_MANY_CODES");
    accept(printed_as(r1099(1_000_000, early_kept(), false), "1B"));
    // Two codes Table 1 never combines, and Y alone.
    reject(other_1099(1_000_000, "7G", false), "R_BOX7_CODES_NOT_COMBINED");
    reject(other_1099(1_000_000, "12", false), "R_BOX7_CODES_NOT_COMBINED");
    reject(other_1099(1_000_000, "Y", true), "R_BOX7_CODES_NOT_COMBINED");
    accept(printed_as(r1099(1_000_000, normal_kept(), true), "7Y"));
    // Box 7b: never with D, N, R or a plan-only code; always with K, S or Y.
    reject(other_1099(1_000_000, "N", true), "R_BOX7B_NOT_IRA");
    reject(other_1099(1_000_000, "R", true), "R_BOX7B_NOT_IRA");
    reject(other_1099(1_000_000, "L", true), "R_BOX7B_NOT_IRA");
    reject(other_1099(1_000_000, "6", true), "R_BOX7B_NOT_IRA");
    reject(other_1099(1_000_000, "7Y", false), "R_BOX7B_NEEDED");
    let mut recharacterized = printed_as(r1099(1_000_000, DistributionCode::NotIncome, false), "N");
    recharacterized.source_mut().form.taxable = Some(0);
    accept(recharacterized);
    let mut annuity = r1099(1_000_000, DistributionCode::Nonqualified, false);
    accept(annuity.clone());
    annuity.source_mut().form.ira = true;
    reject(annuity, "R_BOX7B_NOT_IRA");
    reject(other_1099(1_000_000, "K", false), "R_BOX7B_NEEDED");
    accept(printed_as(r1099(1_000_000, normal_kept(), true), "K"));
    let mut simple = r1099(
        1_000_000,
        DistributionCode::EarlySimple {
            what_happened: SimpleDisposition::Kept { exception: EarlyException::None },
        },
        true,
    );
    accept(simple.clone());
    simple.source_mut().form.ira = false;
    reject(simple, "R_BOX7B_NEEDED");
    // The treatment follows the printed codes.
    let normal = DistributionCode::Normal { what_happened: Disposition::Kept };
    reject(printed_as(r1099(1_000_000, normal.clone(), false), "G"), "R_BOX7_TREATMENT");
    reject(printed_as(other_1099(1_000_000, "7", false), "7"), "R_BOX7_TREATMENT");
    reject(r1099(1_000_000, DistributionCode::Death { died: 20260301 }, true), "R_BOX7_TREATMENT");
    // Code 4 with box 7b checked is an inherited IRA, no longer Another code.
    reject(other_1099(1_000_000, "4", true), "R_BOX7_TREATMENT");
    accept(r1099(1_000_000, DistributionCode::Inherited { account: None }, true));
    // The Roth IRA code follows the printed Q, J or T.
    reject(
        printed_as(
            r1099(
                1_000_000,
                DistributionCode::RothIra {
                    sequence: None,
                    rolled_over: 0,
                    roth: RothIraCode::Qualified,
                },
                false,
            ),
            "J",
        ),
        "R_ROTH_CODE_NOT_BOX7",
    );
}

#[test]
fn box_7a_codes_lead_to_one_treatment() {
    let branches = [
        DistributionCode::Normal { what_happened: Disposition::Kept },
        DistributionCode::Early {
            what_happened: Disposition::Kept,
            exception: EarlyException::None,
        },
        DistributionCode::Disability { began: 20250101, wages: DisabilityWages::None },
        DistributionCode::Death { died: 20250101 },
        DistributionCode::DirectRollover { to: RolloverTarget::Ira },
        DistributionCode::EarlySimple {
            what_happened: SimpleDisposition::Kept { exception: EarlyException::None },
        },
        DistributionCode::Nonqualified,
        DistributionCode::RothIra { sequence: None, rolled_over: 0, roth: RothIraCode::Qualified },
        DistributionCode::Other,
        DistributionCode::NormalLumpSum {
            tax: crate::LumpSumTax::WithOtherIncome { what_happened: Disposition::Kept },
        },
        DistributionCode::DeathLumpSum {
            died: 20250101,
            tax: crate::InheritedLumpSumTax::WithOtherIncome,
        },
        DistributionCode::Exchange,
        DistributionCode::Inherited { account: None },
        DistributionCode::TrumpDeath { estate: false, basis: Some(0) },
        DistributionCode::TrumpToAble { transferred: 20260801 },
        DistributionCode::Corrective { excess_deferral: 0 },
        DistributionCode::NotIncome,
        DistributionCode::NoAdditionalTax,
        DistributionCode::ProhibitedTransaction { exception: EarlyException::None },
        DistributionCode::QualifiedCareCharge,
    ];
    // The one branch (by position, which is the discriminant) each printed box 7a leads to.
    let led_trump = |printed: &str, ira: bool, trump: bool| -> Vec<usize> {
        (0..branches.len())
            .filter(|&i| DistributionCodes::leads_to(box7(printed), ira, trump, &branches[i]))
            .collect()
    };
    let led = |printed: &str, ira: bool| led_trump(printed, ira, false);
    for (printed, ira, branch) in [
        ("7", false, 0),
        ("1", false, 1),
        ("2", true, 1),
        ("3", false, 2),
        ("4", false, 3),
        ("G", false, 4),
        ("S", true, 5),
        ("D", false, 6),
        ("Q", false, 7),
        ("J", false, 7),
        ("T", true, 7),
        ("5", false, 8),
        ("5", true, 18),
        // A source-only code takes the branch of the code printed with it, and alone Normal.
        ("B", false, 0),
        ("4G", false, 4),
        ("1B", false, 1),
        ("8J", false, 15),
        ("7Y", true, 0),
        ("6W", false, 19),
        ("W", false, 19),
        ("W", true, 8),
        ("CD", false, 8),
        ("JS", true, 7),
        ("N", false, 16),
        ("9", false, 17),
        // Code A with 7 or 4 (box 7b unchecked): the 10-year tax option; with box 7b an IRA.
        ("7A", false, 9),
        ("4A", false, 10),
        ("7A", true, 0),
        ("4A", true, 12),
        // Code 6 is a section 1035 exchange; 4 with box 7b is an inherited IRA, 4Y too.
        ("6", false, 11),
        ("4", true, 12),
        ("4Y", true, 12),
        // Code D printed with 1, 2, 3, 4 or 7 leads to that numeric code's branch.
        ("7D", false, 0),
        ("1D", false, 1),
        ("2D", false, 1),
        ("3D", false, 2),
        ("4D", false, 3),
    ] {
        assert_eq!(led(printed, ira), vec![branch], "{printed} {ira}");
    }
    // Box 7c: a Trump account death (4), transfer to the child's ABLE account (G),
    // or returned excess (8). Other codes and box 7b with it remain Another code.
    for (printed, ira, branch) in
        [("4", false, 13), ("G", false, 14), ("8", false, 15), ("7", false, 8), ("4", true, 8)]
    {
        assert_eq!(led_trump(printed, ira, true), vec![branch], "{printed} {ira} 7c");
    }
    // The Roth IRA code follows the printed Q, J or T.
    let roth = |code: &RothIraCode| {
        ["Q", "J", "T"]
            .into_iter()
            .filter(|p| DistributionCodes::leads_to_roth(box7(p), code))
            .collect::<Vec<_>>()
    };
    assert_eq!(roth(&RothIraCode::Qualified), ["Q"]);
    assert_eq!(
        roth(&RothIraCode::Early { repaid: None, distributed: 20260101, exception: None }),
        ["J"]
    );
    assert_eq!(
        roth(&RothIraCode::ExceptionApplies { distributed: 20260101, reason: RothReason::Age }),
        ["T"]
    );
}

#[test]
fn box_7a_client_rules_spell_the_server_combinations() {
    // Table 1's "Used with code" column is symmetric, and the client rule
    // lists exactly the two-code masks the server accepts.
    let codes = "123456789ABCDEFGHJKLMNPQRSTUWY";
    let valid = |printed: &str| {
        DistributionCodes::rule(box7(printed), true).is_none()
            || DistributionCodes::rule(box7(printed), false).is_none()
    };
    let mut pairs = Vec::new();
    for (i, a) in codes.chars().enumerate() {
        for b in codes.chars().skip(i + 1) {
            let both = alloc::format!("{a}{b}");
            if valid(&both) {
                pairs.push(box7(&both));
            }
            assert_eq!(valid(&both), valid(&alloc::format!("{b}{a}")));
        }
    }
    assert_eq!(pairs.len(), 47);
    // Every single code stands alone except Y.
    for c in codes.chars() {
        assert_eq!(valid(&alloc::format!("{c}")), c != 'Y', "{c}");
    }
}

#[test]
fn box_7a_calculation_replies_with_one_treatment_per_row() {
    // Actual topic rows expose the printed codes' allowed treatment
    // discriminants (Normal 0, DirectRollover 4, RothIra 7,
    // Other 8).
    let original = complete(FederalStatus::Single);
    let session = Interview::new(TEST_USER, original.clone());
    for (printed, ira, allowed) in [
        ("7", false, 0),
        ("G", false, 4),
        ("4", false, 3),
        ("4", true, 12),
        ("4G", false, 4),
        ("1B", false, 1),
        ("Q", false, 7),
        ("S", true, 5),
        ("6", false, 11),
        ("7D", false, 0),
    ] {
        let (allowed_treatments, _, _) =
            retirement_question_catalog(&session, box7(printed), ira, false, 0);
        assert_eq!(allowed_treatments, vec![allowed], "{printed} {ira}");
    }
    // A pure calculation: the return is untouched.
    assert_eq!(session.draft, original);
    // Every one- and two-code box 7a leads to exactly one treatment.
    let codes = "123456789ABCDEFGHJKLMNPQRSTUWY";
    for a in codes.chars() {
        for b in codes.chars() {
            for ira in [false, true] {
                let mask = box7(&alloc::format!("{a}{b}"));
                // Code A alone is the one row with two: the participant's own plan or a decedent's.
                // Code A printed with other codes than 4 and 7 is no Table 1 pair.
                if mask & box7("A") != 0 && mask != box7("A") && mask & box7("47") == 0 {
                    continue;
                }
                // Box 7b with 2 or 7 also offers an employer's Roth SEP or Roth SIMPLE contribution.
                let expected =
                    if mask == box7("A") || (ira && (mask == box7("2") || mask == box7("7"))) {
                        2
                    } else {
                        1
                    };
                assert_eq!(DistributionCodes::treatments(mask, ira, false).len(), expected);
            }
        }
    }
}

#[test]
fn reopened_1099_r_rows_carry_their_own_treatment_sets() {
    // Two 1099-Rs with different codes reopen each with its own singleton
    // set, holding the row's treatment.
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let normal =
        r1099(1_000_000, DistributionCode::Normal { what_happened: Disposition::Kept }, false);
    let rollover =
        r1099(2_000_000, DistributionCode::DirectRollover { to: RolloverTarget::Ira }, false);
    let other = other_1099(3_000_000, "C", false);
    assert_eq!(DistributionCodes::treatments(normal.box7, normal.ira, normal.trump), vec![0]);
    assert_eq!(DistributionCodes::treatments(rollover.box7, rollover.ira, rollover.trump), vec![4]);
    assert_eq!(DistributionCodes::treatments(other.box7, other.ira, other.trump), vec![8]);
    let reply = {
        crate::tests::support::client::save_retirement_fixture(
            &mut session,
            vec![
                document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::R1099(retirement_document(normal)),
                    },
                ),
                document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::R1099(retirement_document(rollover)),
                    },
                ),
                document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::R1099(retirement_document(other)),
                    },
                ),
            ],
        )
    };
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let mut session = restart(session.draft);
    let documents = page_documents(&open_documents(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::Retirement { .. }))
        .collect::<Vec<_>>();
    let mut sets: Vec<(u32, Vec<i32>)> = documents
        .iter()
        .map(|d| {
            let IncomeSource::Retirement { value: crate::RetirementDocument::R1099(r) } =
                &d.document
            else {
                unreachable!()
            };
            let (allowed, _, _) = retirement_question_catalog(
                &session,
                r.box7,
                r.ira,
                r.trump,
                r.box_10.unwrap_or(0),
            );
            let stored = session.draft.income.iter().find(|entry| entry.id == d.id).unwrap();
            let FederalIncome::Retirement { data: RetirementSource::R1099(source) } =
                &stored.income
            else {
                unreachable!()
            };
            assert!(DistributionCodes::leads_to(
                r.box7,
                r.ira,
                r.trump,
                source.code.as_ref().unwrap()
            ));
            (r.box7, allowed)
        })
        .collect();
    sets.sort();
    assert_eq!(sets, vec![(box7("7"), vec![0]), (box7("C"), vec![8]), (box7("G"), vec![4]),]);
}

#[test]
fn box_7a_combinations_are_stored_and_refused_by_their_codes() {
    let original = complete(FederalStatus::Single);
    for (printed, ira, needle, key) in [
        (
            "CD",
            false,
            "Form 1099-R codes C and D printed together in box 7a",
            "ISSUE_1099R_CODES_PAIR",
        ),
        (
            "C",
            false,
            "Form 1099-R code C (reportable death benefits, section 6050Y)",
            "ISSUE_1099R_CODE_C",
        ),
    ] {
        let mut session = Interview::new(TEST_USER, original.clone());
        let r = other_1099(1_000_000, printed, ira);
        let reply = {
            crate::tests::support::client::save_retirement_fixture(
                &mut session,
                vec![document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::R1099(retirement_document(r.clone())),
                    },
                )],
            )
        };
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        let FederalIncome::Retirement { data } = &session.draft.income[1].income else {
            unreachable!()
        };
        assert_eq!(crate::FederalRetirement::from(data).box7, box7(printed));
        assert_eq!(
            crate::FederalRetirement::from(data).ordinary_distribution,
            Some(RetirementTreatment::Unsupported)
        );
        let result = estimate(&session.draft);
        assert_eq!(result.state, ResultState::Unsupported, "{result:?}");
        assert!(result.issues.iter().any(|i| i.message.contains(needle)), "{printed}: {result:?}");
        assert!(
            result.issues.iter().any(|i| {
                let m = &i.display;
                m.key == key
            }),
            "{printed}: {result:?}"
        );
        let mut session = restart(session.draft);
        let documents = page_documents(&open_documents(&mut session))
            .into_iter()
            .filter(|row| matches!(row.document, IncomeSource::Retirement { .. }))
            .collect::<Vec<_>>();
        assert_eq!(
            documents[0].document,
            expected_named_source(
                &session.draft,
                Owner::Taxpayer,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(r))
                }
            )
        );
    }
}

#[test]
fn box_7a_codes_round_trip_through_the_interview() {
    // Code 2 with an exception and code 1 with the same exception store the
    // same treatment; the printed code keeps each page as it was entered, as
    // it does a code 3 whose record matches an exception dated December 31.
    let original = complete(FederalStatus::Single);
    let early = |printed: &str| {
        printed_as(
            r1099(
                1_000_000,
                DistributionCode::Early {
                    what_happened: Disposition::Kept,
                    exception: EarlyException::Applies {
                        second: None,
                        repaid: None,
                        distributed: 20261231,
                        facts: crate::RetirementException::Disability { began: 20250101 },
                    },
                },
                false,
            ),
            printed,
        )
    };
    let sent = vec![
        document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(early("1"))),
            },
        ),
        document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(early("2"))),
            },
        ),
        document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(r1099(
                    1_000_000,
                    DistributionCode::Disability { began: 20250101, wages: DisabilityWages::None },
                    false,
                ))),
            },
        ),
        document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(r1099(
                    1_000_000,
                    DistributionCode::Normal {
                        what_happened: Disposition::RolledOver {
                            amount: 1_000_000,
                            into: crate::RolloverDestination::Ira,
                        },
                    },
                    true,
                ))),
            },
        ),
        document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(r1099(
                    1_000_000,
                    DistributionCode::DirectRollover { to: RolloverTarget::Ira },
                    false,
                ))),
            },
        ),
    ];
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply =
        { crate::tests::support::client::save_retirement_fixture(&mut session, sent.clone()) };
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let saved = session.draft.clone();
    let mut session = restart(saved.clone());
    let documents = page_documents(&open_documents(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::Retirement { .. }))
        .collect::<Vec<_>>();
    let mut got: Vec<_> = documents.iter().map(|d| d.document.clone()).collect();
    let want: Vec<_> = sent.iter().map(|d| d.document.clone()).collect();
    let key = |d: &IncomeSource| alloc::format!("{d:?}");
    let mut want = expected_named_sources(&session.draft, Owner::Taxpayer, want);
    got.sort_by_key(key);
    want.sort_by_key(key);
    assert_eq!(got, want);
}

#[test]
fn ssa_1099_reaches_line_6() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    crate::tests::support::client::save_retirement_fixture(
        &mut session,
        vec![document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::Ssa1099(ssa_document(ssa_1099_net(
                    1_800_000, 40_000,
                ))),
            },
        )],
    );
    let FederalIncome::SocialSecurity { data } = &session.draft.income[1].income else {
        unreachable!()
    };
    assert_eq!(FederalSocial::from(data), FederalSocial::from(&ssa_1099_net(1_800_000, 40_000)));
    // Wages $60,000 + half of $18,000 benefits = $69,000 combined income,
    // $44,000 over the single base amount ($25,000) and $35,000 over the
    // second threshold ($34,000): the lesser of 85% of the excess over the
    // second threshold plus the smaller of half the benefits or half the
    // first excess ($29,750 + $4,500 = $34,250) and 85% of the benefits
    // ($15,300) is taxable. Box 6a is entered in full.
    let r = supported(&session.draft);
    assert_eq!(return_line(&r, "6a"), 1_800_000);
    assert_eq!(return_line(&r, "6b"), 1_530_000);
    // Reopening rebuilds the identical document.
    let mut session = restart(session.draft);
    let documents = page_documents(&open_documents(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::Retirement { .. }))
        .collect::<Vec<_>>();
    assert_eq!(
        documents[0].document,
        expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::Ssa1099(ssa_document(ssa_1099_net(
                    1_800_000, 40_000
                )))
            }
        )
    );
}
// A minimal Schedule C business with only an owner-only SEP, for the IRA
// "covered by a plan at work" derivation.
fn business_with_sep(receipts: i64, contribution: i64, rate: u32) -> FederalIncome {
    FederalIncome::Business {
        data: FederalBusiness {
            interest_limit_election: None,
            activity: BusinessActivity::Ordinary {
                returns: 0,
                receipts,
                accounting: BusinessAccounting::Cash,
            },
            retirement: BusinessRetirement::Sep { contribution, rate: f64::from(rate) / 100.0 },
            ..support::builders::survey::survey_cash_business_data()
        },
    }
}

#[test]
fn defined_benefit_owner_plan_is_stored_and_calculated() {
    // Pub.560 defined-benefit scope: the actuary's deposited amount is
    // preserved on reopen and deducts $5,000 within net earnings. Its note
    // states that the App does not compute the actuarial limit.
    let mut s = at_the_adjustments_hub_with_business(8_800_000);
    let hub = open_adjustments_hub(&mut s);
    let Reply::Edit(P::SaveSelfEmployedBenefits { value, .. }) = send(&mut s, go_on(&hub)) else {
        panic!("expected the self-employed benefits page")
    };
    let id = value.businesses[0].id;
    let request = P::SaveSelfEmployedBenefits {
        user: TEST_USER,
        ptc_policies: vec![],
        health_activities: vec![],
        health_people: vec![],
        health_wages: vec![],
        cookie: s.draft.cookie.clone(),
        revision: s.draft.revision,
        page: PAGE_ADJUSTMENTS,
        previous: PAGE_ADJUSTMENTS,
        review: false,
        estimate: None,
        warnings: vec![],
        progress: Some(vec![]),
        value: SelfEmployedBenefits {
            activity_partnerships: vec![],
            activity_scorporations: vec![],
            scorporations: vec![],
            ptc_method: HealthPtcMethod::Simplified,
            policies: vec![],
            benchmarks: vec![],
            partnerships: vec![],
            businesses: vec![BusinessBenefits {
                id,
                name: crate::common::test_label("Business"),
                health: se_health_test_year(600_000, false),
                retirement: SeRetirement::DefinedBenefit { contribution: 500_000 },
            }],
        },
    };
    let reply = send(&mut s, request);
    assert!(matches!(
        reply,
        Reply::Response(P::BeginOftenMissed { .. }) | Reply::Edit(P::SaveOftenMissed { .. })
    ));
    assert!(s.draft.finished_sections.contains(&TaxSection::Adjustments));
    open_adjustments_hub(&mut s);
    let FederalIncome::Business { data } = &s.draft.income[0].income else {
        panic!("expected the business record")
    };
    assert_eq!(data.retirement, BusinessRetirement::DefinedBenefit { contribution: 500_000 });
    let r = estimate(&s.draft);
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope);
    assert_eq!(super::value(&r, &alloc::format!("business_{id}_owner_plan_deduction")), 5_000);
    assert!(r.review_notes.iter().any(|n| n.display.key == "NOTE_DEFINED_BENEFIT_ACTUARY"));
    let request = go_to(&s.draft, PAGE_ADJUSTMENTS, TaxSection::SelfEmployedBenefits);
    let Reply::Edit(P::SaveSelfEmployedBenefits { value, .. }) = send(&mut s, request) else {
        panic!("expected the self-employed benefits page")
    };
    assert_eq!(
        value.businesses[0].retirement,
        SeRetirement::DefinedBenefit { contribution: 500_000 }
    );
}

#[test]
fn covered_at_work_comes_from_box_13_or_a_sep() {
    for (w2_covered, sep, expected) in
        [(true, false, true), (false, true, true), (false, false, false)]
    {
        let mut d = complete(FederalStatus::Single);
        if w2_covered {
            let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
            edit_wage(data).retirement_plan = true;
        }
        if sep {
            add(&mut d, income(business_with_sep(1_000_000, 100_000, 2500)));
        }
        let mut s = Interview::new(TEST_USER, d);
        crate::tests::support::client::admit_native_fixture_facts(&mut s);
        let source = open_documents(&mut s);
        accepted(&send(&mut s, source));
        let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::IraContributions);
        let Reply::Edit(P::SaveIraContributions { allowed_owners, .. }) = send(&mut s, request)
        else {
            panic!("expected the IRA contributions page")
        };
        // Submit a contribution; the client's covered answer, deliberately
        // wrong, is never trusted.
        let request = save_ira_people(
            &s.draft,
            allowed_owners,
            vec![IraContributor {
                owner: Owner::Taxpayer,
                traditional: 200_000,
                roth: 0,
                covered: !expected,
                form_8606_value: None,
                more: IraMore::default(),
            }],
        );
        let reply = send(&mut s, request);
        walk_on_to_review(&mut s, reply);
        assert!(matches!(
            send(&mut s, P::ReviewAccepted { user: TEST_USER }),
            Reply::Response(P::Review { .. })
        ));
        let iras = &s.draft.adjustments.as_ref().unwrap().iras;
        assert_eq!(iras.taxpayer.covered_at_work, expected, "w2={w2_covered} sep={sep}");
        // Reopening the page shows the same derived answer.
        let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::IraContributions);
        let Reply::Edit(P::SaveIraContributions { value, .. }) = send(&mut s, request) else {
            panic!("expected the IRA contributions page")
        };
        assert_eq!(value.people[0].covered, expected, "w2={w2_covered} sep={sep}");
    }
}

#[test]
fn nondeductible_above_traditional_is_rejected() {
    let d = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d);
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::IraContributions);
    let Reply::Edit(P::SaveIraContributions { allowed_owners, .. }) = send(&mut s, request) else {
        panic!("expected the IRA contributions page")
    };
    let request = save_ira_people(
        &s.draft,
        allowed_owners,
        vec![IraContributor {
            owner: Owner::Taxpayer,
            traditional: 100_000,
            roth: 0,
            covered: false,
            form_8606_value: None,
            more: IraMore { nondeductible: 200_000, ..Default::default() },
        }],
    );
    rejected_with(send(&mut s, request), "IRA_NONDEDUCTIBLE_ABOVE_TRADITIONAL");
}
fn saver_value(page: &P) -> &SaverCredit {
    let P::SaveSaver { value, .. } = page else { panic!("expected the saver page, got {page:?}") };
    value
}
fn save_saver(d: &FederalDraft, value: SaverCredit) -> P {
    P::SaveSaver {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_CREDITS,
        previous: PAGE_CREDITS,
        review: false,
        estimate: None,
        warnings: vec![],
        progress: vec![],
        value,
    }
}
// A single return with $`wages` of W-2 wages and box 12 deferrals.
fn with_deferrals(wages: i64, box12: Vec<W2CodeAmount>) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, wages, 0);
    if let FederalIncome::Wage { data } = &mut d.income[0].income {
        edit_wage(data).other_box12 = box12;
    }
    name_accepted_income_documents(&mut d);
    d
}

#[test]
fn saver_contributions_come_from_box_12() {
    // Form 8880 by hand: single, AGI $30,000 (the 10% band: above $26,250,
    // at most $40,250). Line 2: W-2 box 12 D $1,500 + AA $500 + $300 not on a
    // W-2 = 2,300; line 4: the 2025 distribution $200; line 6 = min(2,100,
    // 2,000) = 2,000; line 9 10%: credit $200, which the $1,420 tax on
    // $13,900 of taxable income covers. Box 12 D lowered to $1,000: line 2 =
    // 1,800, line 6 = 1,600, credit $160.
    let mut d = with_deferrals(
        3_000_000,
        vec![
            W2CodeAmount { code: W2Code::D, amount: 150_000 },
            W2CodeAmount { code: W2Code::AA, amount: 50_000 },
            // Code W is an HSA contribution, not a retirement deferral.
            W2CodeAmount { code: W2Code::W, amount: 40_000 },
        ],
    );
    // This nonretirement W-2 item is designated for the prior HSA year.
    health_w2_year(&mut d, HealthEmployerYear::PriorYear);
    let (mut s, hub) = at_the_credits_hub(d);
    let P::Hub { suggested, allowed_sections, .. } = &hub else { unreachable!() };
    assert_eq!(*suggested, TaxSection::Saver);
    // The previous and next shown sections: the adjustments hub
    // (box 12 deferrals make its IRA topic a derived one) and the review.
    assert_available_sections(
        allowed_sections,
        &[
            TaxSection::Adjustments,
            TaxSection::Checklist,
            TaxSection::Education,
            TaxSection::Saver,
            TaxSection::Marketplace,
            TaxSection::MoreCredits,
            TaxSection::Review,
        ]
        .map(|t| t as i32),
    );
    assert!(hub_rows(&hub).contains(&row_of(
        "Retirement savings credit",
        "Not started",
        TaxSection::Saver
    )));
    let page = s.handle(go_on(&hub)).unwrap();
    let Reply::Edit(page) = page else { panic!("expected the saver page, got {page:?}") };
    let value = saver_value(&page).clone();
    assert_eq!(
        value,
        SaverCredit {
            from_w2: vec![SaverContribution { owner: Owner::Taxpayer, amount: 200_000 }],
            distributions: vec![],
            more: SaverMore::default(),
        }
    );
    let entered = SaverCredit {
        distributions: vec![FederalSaverDistribution {
            owner: Owner::Taxpayer,
            year: SaverDistributionPeriod::PriorOne { joint_with_current_spouse: false },
            amount: 20_000,
        }],
        more: SaverMore { your_able: 0, spouse_able: 0, other_contributions: 30_000 },
        // The derived rows are ignored on save.
        from_w2: vec![],
    };
    let request = save_saver(&s.draft, entered.clone());
    let reply = send_past_less_common(&mut s, request);
    walk_on_to_review(&mut s, reply);
    assert!(matches!(
        send(&mut s, P::ReviewAccepted { user: TEST_USER }),
        Reply::Response(P::Review { .. })
    ));
    let hub = credits_hub(crate::tests::cross::interview::from_review(&mut s, TaxSection::Credits));
    let facts = s.draft.credits.as_ref().unwrap().saver.as_ref().unwrap().clone();
    assert_eq!(facts.taxpayer_contributions, 230_000);
    assert_eq!(facts.spouse_contributions, 0);
    assert_eq!(facts.distributions.len(), 1);
    assert!(hub_rows(&hub).contains(&row_of(
        "Retirement savings credit",
        "Recorded",
        TaxSection::Saver
    )));
    let P::Hub { suggested, .. } = &hub else { unreachable!() };
    // Continue goes to the next shown section, the last page before the
    // review (federal payments only with its box).
    assert_eq!(*suggested, TaxSection::OftenMissed);
    assert_eq!(value_of(&continued(&mut s), "saver_credit"), 200);
    // It reopens as entered, the workplace rows derived again.
    let page = open_credits_topic(&mut s, TaxSection::Saver);
    assert_eq!(saver_value(&page), &SaverCredit { from_w2: value.from_w2.clone(), ..entered });
    // Lowering box 12 D in the Document center re-derives the workplace part and
    // keeps the $300 not on a W-2.
    let mut wages = open_documents(&mut s);
    let P::SaveDocuments { documents, .. } = &mut wages else { unreachable!() };
    let crate::WageDocument::W2(w) = &mut documents.common.wages.documents[0].document else {
        panic!("expected the W-2")
    };
    for entry in w.box12.iter_mut() {
        if entry.code == W2Code::D {
            entry.amount = Some(100_000);
        }
    }
    let reply = send(&mut s, wages);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let facts = s.draft.credits.as_ref().unwrap().saver.as_ref().unwrap();
    assert_eq!(facts.taxpayer_contributions, 180_000);
    assert_eq!(value_of(&continued(&mut s), "saver_credit"), 160);
}

#[test]
fn saver_is_suggested_only_inside_its_bands() {
    // Notice 2025-67's 2026 single bands: 10% up to AGI $40,250, nothing
    // above. With box 12 D the topic is offered and suggested at $40,250 and
    // neither at $40,251. Without a contribution on the return it is still
    // offered inside the bands (the page asks for contributions no other
    // page records, such as an ABLE account), but not suggested.
    let deferral = vec![W2CodeAmount { code: W2Code::D, amount: 100_000 }];
    for (wages, box12, offered, hinted) in [
        (4_025_000, deferral.clone(), true, true),
        (4_025_100, deferral.clone(), false, false),
        (3_000_000, vec![], true, false),
    ] {
        let d = with_deferrals(wages, box12);
        assert_eq!(value(&supported(&d), "agi") * 100, wages);
        let (mut s, hub) = at_the_credits_hub(d);
        let P::Hub { suggested, allowed_sections, .. } = &hub else { unreachable!() };
        // Continue goes to the next shown section, the review.
        assert_eq!(
            *suggested,
            if hinted { TaxSection::Saver } else { TaxSection::OftenMissed },
            "{wages}"
        );
        assert_eq!(allowed_sections.contains(&PAGE_SAVER), offered, "{wages}");
        let rows = hub_rows(&hub);
        assert_eq!(rows.iter().any(|r| r.2 == TaxSection::Saver), offered, "{wages}");
        if !offered {
            let request = go_to(&s.draft, PAGE_CREDITS, TaxSection::Saver);
            error(send(&mut s, request));
            let request = save_saver(
                &s.draft.clone(),
                SaverCredit { from_w2: vec![], distributions: vec![], more: SaverMore::default() },
            );
            error(current_source_refusal(
                "secondary-admission-3114",
                &mut s,
                request,
                "PROBLEM_SECTION_UNAVAILABLE_YET",
                None,
                SourceScope::Fieldless,
            ));
            assert!(s.draft.credits.as_ref().unwrap().saver.is_none());
        }
    }
}

#[test]
fn ira_page_excess_fold_round_trips_to_the_excess_record() {
    // T2-42: the More fold's excess record is stored on the owner's
    // contribution record and reopens on the page unchanged.
    let d = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d);
    let source = open_documents(&mut s);
    accepted(&send(&mut s, source));
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::IraContributions);
    let Reply::Edit(P::SaveIraContributions { allowed_owners, .. }) = send(&mut s, request) else {
        panic!("expected the IRA contributions page")
    };
    // C18: the page's entry, whose confirmation is the branch itself.
    let entry = IraExcessEntry {
        traditional_value: Some(920_000),
        roth_value: 101_000,
        prior_traditional: 0,
        prior_roth: 0,
        returned_prior_excess: 0,
    };
    let excess = IraExcess {
        traditional_value: Some(920_000),
        roth_value: 101_000,
        prior_traditional: 0,
        prior_roth: 0,
        returned_prior_excess: 0,
    };
    let request = save_ira_people(
        &s.draft,
        allowed_owners,
        vec![IraContributor {
            owner: Owner::Taxpayer,
            traditional: 900_000,
            roth: 100_000,
            covered: false,
            form_8606_value: None,
            more: IraMore { excess: Some(entry.clone()), ..Default::default() },
        }],
    );
    let reply = send(&mut s, request);
    walk_on_to_review(&mut s, reply);
    assert!(matches!(
        send(&mut s, P::ReviewAccepted { user: TEST_USER }),
        Reply::Response(P::Review { .. })
    ));
    let iras = &s.draft.adjustments.as_ref().unwrap().iras;
    assert_eq!(iras.taxpayer.excess, Some(excess.clone()));
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::IraContributions);
    let Reply::Edit(P::SaveIraContributions { value, .. }) = send(&mut s, request) else {
        panic!("expected the IRA contributions page")
    };
    assert!(matches!(
        &value.people[0].more,
        IraMore { excess: Some(e), .. } if *e == entry
    ));
}

#[test]
fn ira_contributor_twice_is_rejected_on_save() {
    // The client twin checks uniqueness of the collected owners; the save checks it
    // again. Two taxpayer rows on a joint return.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let (_, allowed) = open_ira_page(&mut s);
    let before = s.draft.clone();
    let request = save_ira_people(
        &before,
        allowed,
        vec![ira_row(Owner::Taxpayer, 100_000, None), ira_row(Owner::Taxpayer, 200_000, None)],
    );
    rejected_with(send(&mut s, request), "IRA_CONTRIBUTOR_TWICE");
    assert_eq!(s.draft, before);
}

#[test]
fn ira_page_stores_its_rows_without_a_switch() {
    // Audit row 46: the IRA facts are stored as entered. A row with nothing
    // entered is no IRA contribution and reopens as no rows; a row with no
    // 2026 contribution keeps its 2025 excess record instead of dropping it.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let source = open_documents(&mut s);
    accepted(&send(&mut s, source));
    let (_, allowed) = open_ira_page(&mut s);
    let request =
        save_ira_people(&s.draft, allowed.clone(), vec![ira_row(Owner::Taxpayer, 0, None)]);
    let reply = send(&mut s, request);
    walk_on_to_review(&mut s, reply);
    assert!(matches!(
        send(&mut s, P::ReviewAccepted { user: TEST_USER }),
        Reply::Response(P::Review { .. })
    ));
    assert!(s.draft.adjustments.as_ref().unwrap().entered_iras().is_none());
    assert!(open_ira_page(&mut s).0.is_empty());
    let entry = IraExcessEntry {
        traditional_value: Some(920_000),
        roth_value: 0,
        prior_traditional: 50_000,
        prior_roth: 0,
        returned_prior_excess: 0,
    };
    let request =
        save_ira_people(&s.draft, allowed, vec![ira_row(Owner::Taxpayer, 0, Some(entry.clone()))]);
    let reply = send(&mut s, request);
    walk_on_to_review(&mut s, reply);
    assert!(matches!(
        send(&mut s, P::ReviewAccepted { user: TEST_USER }),
        Reply::Response(P::Review { .. })
    ));
    assert!(s.draft.adjustments.as_ref().unwrap().entered_iras().is_some());
    let (rows, _) = open_ira_page(&mut s);
    assert_eq!(rows[0].more.excess, Some(entry));
}

// B11 R2, R3: the IRA page's returned-excess rules and the contributions returned in 2027 list.
#[test]
fn ira_page_checks_the_returned_excess_and_keeps_the_contributions_returned_in_2027() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let (_, allowed) = open_ira_page(&mut s);
    let entry = |prior, returned| IraExcessEntry {
        traditional_value: Some(4_000_000),
        roth_value: 0,
        prior_traditional: prior,
        prior_roth: 0,
        returned_prior_excess: returned,
    };
    let try_save = |s: &mut Interview, entry: IraExcessEntry| {
        let request = save_ira_people(
            &s.draft,
            allowed.clone(),
            vec![ira_row(Owner::Taxpayer, 0, Some(entry))],
        );
        send(s, request)
    };
    // More paid back than was carried.
    rejected_with(try_save(&mut s, entry(700_000, 800_000)), "IRA_RETURNED_EXCESS_ABOVE_CARRIED");
    // More than the owner's 2026 traditional IRA withdrawals on the Retirement page.
    rejected_with(
        try_save(&mut s, entry(700_000, 700_000)),
        "IRA_RETURNED_EXCESS_ABOVE_DISTRIBUTIONS",
    );
    add(
        &mut s.draft,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            7_000,
            RetirementTreatment::Ordinary,
        ),
    );
    accepted(&try_save(&mut s, entry(700_000, 700_000)));
    // The list of contributions returned in 2027 round-trips through the page.
    let returned = crate::ReturnedContribution {
        kind: crate::IraKind::Roth,
        principal: 750_000,
        earnings: 40_000,
        returned_on: 20270310,
        late: None,
    };
    let mut row = ira_row(Owner::Taxpayer, 0, None);
    row.more.returned_in_following_year = vec![returned.clone()];
    let request = save_ira_people(&s.draft, allowed.clone(), vec![row]);
    accepted(&send(&mut s, request));
    let (rows, _) = open_ira_page(&mut s);
    assert_eq!(rows[0].more.returned_in_following_year, vec![returned.clone()]);
    // After April 15, 2027 without a route the server refuses, as the client does.
    let mut late = returned;
    late.returned_on = 20270601;
    let mut row = ira_row(Owner::Taxpayer, 0, None);
    row.more.returned_in_following_year = vec![late];
    let request = save_ira_people(&s.draft, allowed, vec![row]);
    rejected_with(send(&mut s, request), "IRA_RETURN_AFTER_DUE_DATE_NEEDS_ROUTE");
}

#[test]
fn ira_excess_takes_the_traditional_value_from_form_8606() {
    // $9,000 traditional against the $7,500 limit: line 16 = 1,500. The
    // Form 8606 year-end value is $400, so line 17 = 6% x min(1,500, 400)
    // = 24; after the Form 8606 changes to $1,500, 6% x 1,500 = 90.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    with_form_8606(&mut s, 40_000);
    // The page offers the owner's row with the Form 8606 value, read-only.
    let (rows, allowed) = open_ira_page(&mut s);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].form_8606_value, Some(40_000));
    assert_eq!(rows[0].traditional, 0);
    let mut row = ira_row(Owner::Taxpayer, 900_000, Some(excess_entry(None)));
    row.form_8606_value = Some(40_000);
    let request = save_ira_people(&s.draft, allowed.clone(), vec![row.clone()]);
    accepted(&send(&mut s, request));
    let stored = |s: &Interview| s.draft.adjustments.as_ref().unwrap().iras.taxpayer.excess.clone();
    assert_eq!(
        stored(&s),
        Some(IraExcess {
            traditional_value: Some(40_000),
            roth_value: 0,
            prior_traditional: 0,
            prior_roth: 0,
            returned_prior_excess: 0,
        })
    );
    let r = estimate(&s.draft);
    assert_eq!(value(&r, "ira_excess_taxpayer_line17_traditional_excess_tax"), 24);
    // Reopens with the value still at home on the Form 8606.
    assert_eq!(open_ira_page(&mut s).0, vec![row]);
    // Changing the Form 8606 carries its value into the excess record.
    with_form_8606(&mut s, 150_000);
    assert_eq!(stored(&s).unwrap().traditional_value, Some(150_000));
    assert_eq!(value(&estimate(&s.draft), "ira_excess_taxpayer_line17_traditional_excess_tax"), 90);
}

#[test]
fn ira_excess_traditional_value_is_entered_exactly_once() {
    // With a Form 8606: a typed value repeats it (IRA_EXCESS_TRADITIONAL_ON_8606).
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    with_form_8606(&mut s, 40_000);
    let (_, allowed) = open_ira_page(&mut s);
    let before = s.draft.clone();
    let request = save_ira_people(
        &before,
        allowed.clone(),
        vec![ira_row(Owner::Taxpayer, 900_000, Some(excess_entry(Some(40_000))))],
    );
    rejected_with(send(&mut s, request), "IRA_EXCESS_TRADITIONAL_ON_8606");
    assert_eq!(s.draft, before);
    // Without one: the value is required (IRA_EXCESS_TRADITIONAL_VALUE_NEEDED).
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let (rows, allowed) = open_ira_page(&mut s);
    assert!(rows.is_empty());
    let before = s.draft.clone();
    let request = save_ira_people(
        &before,
        allowed.clone(),
        vec![ira_row(Owner::Taxpayer, 900_000, Some(excess_entry(None)))],
    );
    rejected_with(send(&mut s, request), "IRA_EXCESS_TRADITIONAL_VALUE_NEEDED");
    assert_eq!(s.draft, before);
    // Zero is a value: accepted and stored as entered.
    let request = save_ira_people(
        &before,
        allowed,
        vec![ira_row(Owner::Taxpayer, 900_000, Some(excess_entry(Some(0))))],
    );
    accepted(&send(&mut s, request));
}

// T2-cy item 3. i5329 line 17: the value of the traditional IRAs on
// December 31, 2026 includes 2026 contributions made in 2027; Form 8606
// line 6 (IraBasis.year_end) does not, so the excess record adds the Form
// 8606's paid_next_year, derived from the IRA page's 2027 deposits.
#[test]
fn form_5329_line_17_value_adds_2026_contributions_made_in_2027() {
    // $9,000 traditional against the $7,500 limit: line 16 = 1,500. Form
    // 8606 year-end value $400 plus $1,000 deposited in 2027 for 2026:
    // line 17 = 6% x min(1,500, 1,400) = 84 (the year-end value alone
    // would give 6% x 400 = 24).
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    with_form_8606(&mut s, 40_000);
    let (_, allowed) = open_ira_page(&mut s);
    let mut row = ira_row(Owner::Taxpayer, 900_000, Some(excess_entry(None)));
    row.form_8606_value = Some(40_000);
    row.more.paid_next_year = Some(next_year_ira_deposits(100_000, 0, 20270301));
    let request = save_ira_people(&s.draft, allowed.clone(), vec![row.clone()]);
    accepted(&send(&mut s, request));
    let adjustments = s.draft.adjustments.as_ref().unwrap();
    assert_eq!(adjustments.ira_basis[0].paid_next_year, 100_000);
    assert_eq!(adjustments.iras.taxpayer.excess.as_ref().unwrap().traditional_value, Some(140_000));
    assert_eq!(value(&estimate(&s.draft), "ira_excess_taxpayer_line17_traditional_excess_tax"), 84);
    // Reopens with the Form 8606 value alone shown and nothing typed.
    assert_eq!(open_ira_page(&mut s).0, vec![row.clone()]);
    // Boundary: $500 + $1,000 = 1,500 equals line 16: 6% x 1,500 = 90.
    with_form_8606(&mut s, 50_000);
    assert_eq!(value(&estimate(&s.draft), "ira_excess_taxpayer_line17_traditional_excess_tax"), 90);
}

// Form 4972 on the 1099-R page (lump_sum.rs).
fn lump_tax(method: crate::LumpSumMethod, include_nua: bool) -> crate::LumpSumTax {
    crate::LumpSumTax::Form4972 {
        method,
        include_nua,
        event: crate::LumpSumEvent::Age,
        first_payment: 20260301,
        last_prior_payment: None,
        entire_balance: true,
        confirmed: true,
    }
}
fn inherited_tax(method: crate::LumpSumMethod) -> crate::InheritedLumpSumTax {
    crate::InheritedLumpSumTax::Form4972 {
        death_benefit: 0,
        born: 19330505,
        estate_tax: 100_000,
        method,
        include_nua: false,
        first_payment: 20260401,
        last_prior_payment: Some(20251201),
        entire_balance: true,
        confirmed: true,
    }
}
fn seven_a(tax: crate::LumpSumTax) -> R1099Answered {
    r1099(1_750_000, DistributionCode::NormalLumpSum { tax }, false)
}
fn four_a(tax: crate::InheritedLumpSumTax) -> R1099Answered {
    r1099(1_750_000, DistributionCode::DeathLumpSum { died: 20260201, tax }, false)
}

#[test]
fn lump_sum_codes_7a_and_4a_lead_to_their_own_branches() {
    // The branch (declaration order of DistributionCode): 7A without box 7b is 9, 4A is 10; with
    // box 7b checked an IRA cannot use Form 4972: 7A is Normal (0) and 4A Inherited (12); A alone
    // offers both Form 4972 branches (or both IRA branches with box 7b).
    for (printed, ira, branches) in [
        ("7A", false, vec![9]),
        ("4A", false, vec![10]),
        ("7A", true, vec![0]),
        ("4A", true, vec![12]),
        ("A", false, vec![9, 10]),
        ("A", true, vec![0, 12]),
        ("7", false, vec![0]),
        ("4", false, vec![3]),
    ] {
        assert_eq!(
            DistributionCodes::treatments(box7(printed), ira, false),
            branches,
            "{printed} {ira}"
        );
    }
    let normal = seven_a(lump_tax(crate::LumpSumMethod::TenYear, false));
    assert_eq!(normal.box7, box7("7A"));
    assert_eq!(DistributionCodes::treatments(normal.box7, normal.ira, normal.trump), vec![9]);
    let death = four_a(crate::InheritedLumpSumTax::WithOtherIncome);
    assert_eq!(death.box7, box7("4A"));
    assert_eq!(DistributionCodes::treatments(death.box7, death.ira, death.trump), vec![10]);
    // A 7 or 4 page branch cannot be saved against 7A or 4A.
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let wrong = printed_as(
        r1099(1_000_000, DistributionCode::Normal { what_happened: Disposition::Kept }, false),
        "7A",
    );
    let reply = {
        crate::tests::support::client::save_retirement_fixture(
            &mut session,
            vec![document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(wrong)),
                },
            )],
        )
    };
    rejected_with(reply, "R_BOX7_TREATMENT");
}

#[test]
fn lump_sum_page_answers_map_onto_the_stored_treatments() {
    use crate::LumpSumMethod::{Both, TenYear};
    // With your other income stores what 7 and 4 store.
    assert_eq!(
        retirement_treatment(
            DistributionCode::NormalLumpSum {
                tax: crate::LumpSumTax::WithOtherIncome { what_happened: Disposition::Kept }
            },
            false
        ),
        RetirementTreatment::Ordinary
    );
    assert_eq!(
        retirement_treatment(
            DistributionCode::NormalLumpSum {
                tax: crate::LumpSumTax::WithOtherIncome {
                    what_happened: Disposition::RolledOver {
                        amount: 1_000_000,
                        into: crate::RolloverDestination::Ira
                    }
                }
            },
            false
        ),
        RetirementTreatment::Rollover {
            destination: crate::RolloverDestination::Ira,
            amount: 1_000_000,
            early: false
        }
    );
    assert_eq!(
        retirement_treatment(
            DistributionCode::DeathLumpSum {
                died: 20260201,
                tax: crate::InheritedLumpSumTax::WithOtherIncome
            },
            false
        ),
        RetirementTreatment::Exception {
            second: None,
            distributed: 20261231,
            facts: crate::RetirementException::Death { died: 20260201 },
            early_simple: false
        }
    );
    // Separately on Form 4972 stores the election with the Other boxes; the two-decimal
    // percents become exact hundredths.
    let mut own = seven_a(lump_tax(Both, true));
    own.source_mut().form.taxable = Some(1_500_000);
    own.source_mut().form.box_3 = Some(100_000);
    own.source_mut().form.box_6 = Some(300_000);
    own.source_mut().form.box_8a = Some(200_000);
    own.source_mut().form.box_8b_percent = Some(33.33);
    own.source_mut().form.box_9a_percent = Some(40.0);
    let data = RetirementSource::R1099(own.into_source());
    let Some(RetirementTreatment::LumpSum { data: stored }) =
        &crate::FederalRetirement::from(&data).ordinary_distribution
    else {
        panic!("{:?}", crate::FederalRetirement::from(&data).ordinary_distribution)
    };
    assert_eq!(
        stored.participant,
        crate::Form4972Participant::Own { event: crate::LumpSumEvent::Age }
    );
    assert_eq!(
        (
            stored.method,
            crate::FederalRetirement::from(&data).box_3,
            crate::FederalRetirement::from(&data).box_6,
            stored.include_nua,
            crate::FederalRetirement::from(&data).box_8a,
            crate::FederalRetirement::from(&data).box_8b_percent,
            crate::FederalRetirement::from(&data).box_9a_percent
        ),
        (Both, 100_000, 300_000, true, 200_000, Some(33.33), Some(40.0))
    );
    let mut inherited = four_a(inherited_tax(TenYear));
    inherited.source_mut().form.box_8a = Some(50_000);
    let data = RetirementSource::R1099(inherited.into_source());
    let Some(RetirementTreatment::LumpSum { data: stored }) =
        &crate::FederalRetirement::from(&data).ordinary_distribution
    else {
        unreachable!()
    };
    assert_eq!(
        stored.participant,
        crate::Form4972Participant::Decedent {
            death_benefit: 0,
            born: 19330505,
            died: 20260201,
            estate_tax: 100_000
        }
    );
}

#[test]
fn lump_sum_page_rules_reject_on_both_sides() {
    use crate::LumpSumMethod::{Both, CapitalGain, TenYear};
    let original = complete(FederalStatus::Single);
    let reject = |r: R1099Answered, key: &str| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = {
            crate::tests::support::client::save_retirement_fixture(
                &mut session,
                vec![document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::R1099(retirement_document(r)),
                    },
                )],
            )
        };
        let Reply::Response(P::Problem { message, .. }) = reply else {
            panic!("expected a problem for {key}, got {reply:?}")
        };
        assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
        assert_eq!(session.draft, original);
    };
    let accept = |r: R1099Answered| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = {
            crate::tests::support::client::save_retirement_fixture(
                &mut session,
                vec![document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::R1099(retirement_document(r)),
                    },
                )],
            )
        };
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        session.draft
    };
    // Box 3 above box 2a; box 8b without box 8a.
    let mut r = seven_a(lump_tax(TenYear, false));
    r.source_mut().form.box_3 = Some(1_750_001);
    reject(r, "R_BOX3_ABOVE_TAXABLE");
    let mut r = seven_a(lump_tax(TenYear, false));
    r.source_mut().form.box_8b_percent = Some(50.0);
    reject(r, "R_BOX8B_WITHOUT_BOX8A");
    // The capital gain rate needs box 3; the NUA needs box 6.
    for method in [CapitalGain, Both] {
        reject(seven_a(lump_tax(method, false)), "LUMP_SUM_CAPITAL_GAIN_WITHOUT_BOX_3");
        let mut r = four_a(inherited_tax(method));
        r.source_mut().form.box_6 = Some(0);
        reject(r, "LUMP_SUM_CAPITAL_GAIN_WITHOUT_BOX_3");
    }
    reject(seven_a(lump_tax(TenYear, true)), "LUMP_SUM_NUA_WITHOUT_BOX_6");
    // No other special treatment alongside Form 4972 (the Simplified Method, box 10, a disaster
    // distribution and public safety premiums are refused by the same rule).
    let mut r = seven_a(lump_tax(TenYear, false));
    r.source_mut().disability =
        Some(DisabilityIncome::Qualifying { amount: 100_000, eligible: true });
    reject(r, "LUMP_SUM_OTHER_TREATMENTS");
    // The first payment is in 2026, the latest earlier payment before it, the birth before the death.
    let mut r = seven_a(lump_tax(TenYear, false));
    let Some(crate::DistributionCode::NormalLumpSum {
        tax: crate::LumpSumTax::Form4972 { first_payment, .. },
    }) = &mut r.source_mut().code
    else {
        unreachable!()
    };
    *first_payment = 20251231;
    reject(r, "LUMP_SUM_FIRST_PAYMENT_NOT_IN_2026");
    let mut r = four_a(inherited_tax(TenYear));
    let Some(crate::DistributionCode::DeathLumpSum {
        tax: crate::InheritedLumpSumTax::Form4972 { last_prior_payment, .. },
        ..
    }) = &mut r.source_mut().code
    else {
        unreachable!()
    };
    *last_prior_payment = Some(20260101);
    reject(r, "LUMP_SUM_PRIOR_PAYMENT_NOT_EARLIER");
    let mut r = four_a(inherited_tax(TenYear));
    let Some(crate::DistributionCode::DeathLumpSum {
        tax: crate::InheritedLumpSumTax::Form4972 { born, .. },
        ..
    }) = &mut r.source_mut().code
    else {
        unreachable!()
    };
    *born = 20260201;
    reject(r, "LUMP_SUM_BORN_AFTER_DEATH");
    // Box 9a is accepted with every treatment and stored only on an elected record.
    let mut r = seven_a(crate::LumpSumTax::WithOtherIncome { what_happened: Disposition::Kept });
    r.source_mut().form.box_9a_percent = Some(40.0);
    accept(r);
    let mut r = seven_a(lump_tax(TenYear, false));
    r.source_mut().form.box_9a_percent = Some(40.0);
    let saved = accept(r);
    assert!(matches!(
        &saved.income[1].income,
        FederalIncome::Retirement { data }
            if matches!(&crate::FederalRetirement::from(data).ordinary_distribution,
                Some(RetirementTreatment::LumpSum { .. })) && crate::FederalRetirement::from(data).box_9a_percent == Some(40.0)
    ));
    // A Roth IRA row still needs the new boxes blank.
    let mut roth = roth_item_for_tests();
    roth.source_mut().form.box_3 = Some(1);
    reject(roth, "R_ROTH_OTHER_BOXES");
}

#[test]
fn lump_sum_page_round_trips_through_the_interview() {
    use crate::LumpSumMethod::{Both, TenYear};
    let original = complete(FederalStatus::Single);
    let mut own = seven_a(lump_tax(Both, true));
    own.source_mut().form.taxable = Some(1_500_000);
    own.source_mut().form.box_3 = Some(100_000);
    own.source_mut().form.box_6 = Some(300_000);
    own.source_mut().form.box_8a = Some(200_000);
    own.source_mut().form.box_8b_percent = Some(33.33);
    own.source_mut().form.box_9a_percent = Some(40.0);
    let mut inherited = four_a(inherited_tax(TenYear));
    inherited.source_mut().form.box_8a = Some(50_000);
    let other_income = four_a(crate::InheritedLumpSumTax::WithOtherIncome);
    // Unelected amounts remain the same printed boxes when the source reopens.
    let mut unelected =
        seven_a(crate::LumpSumTax::WithOtherIncome { what_happened: Disposition::Kept });
    unelected.source_mut().form.box_3 = Some(100_000);
    unelected.source_mut().form.box_6 = Some(300_000);
    unelected.source_mut().form.box_8a = Some(200_000);
    unelected.source_mut().form.box_8b_percent = Some(25.0);
    // Each row carries the methods its codes allow, as the page builder writes them.
    let with_methods = |r: R1099Answered| r;
    let (own, inherited, other_income, unelected) = (
        with_methods(own),
        with_methods(inherited),
        with_methods(other_income),
        with_methods(unelected),
    );
    let sent = vec![
        document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(own)),
            },
        ),
        document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(inherited)),
            },
        ),
        document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(other_income)),
            },
        ),
        document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(unelected.clone())),
            },
        ),
    ];
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply =
        { crate::tests::support::client::save_retirement_fixture(&mut session, sent.clone()) };
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let mut session = restart(session.draft.clone());
    let documents = page_documents(&open_documents(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::Retirement { .. }))
        .collect::<Vec<_>>();
    let mut got: Vec<_> = documents.iter().map(|d| d.document.clone()).collect();
    let want: Vec<_> = sent.iter().map(|d| d.document.clone()).collect();
    let key = |d: &IncomeSource| alloc::format!("{d:?}");
    let mut want = expected_named_sources(&session.draft, Owner::Taxpayer, want);
    got.sort_by_key(key);
    want.sort_by_key(key);
    assert_eq!(got, want);
}

#[test]
fn lump_sum_import_reads_boxes_3_6_8a_and_8b() {
    // AI import: the Other boxes are import targets on the printed paper and the instruction names them.
    let model = crate::federal_model::HUB_SOURCE;
    let more = &model[model.find("pub struct Form1099RPaper {").unwrap()..];
    let more = &more[..more.find("\n}").unwrap()];
    for field in [
        "pub box_3: Option<i64>",
        "pub box_6: Option<i64>",
        "pub box_8a: Option<i64>",
        "pub box_8b_percent: Option<f64>",
        "pub box_9a_percent: Option<f64>",
    ] {
        let at = more.find(field).unwrap();
        let before = &more[..at];
        let attrs = &before[before.rfind("///").unwrap()..];
        assert!(
            attrs.contains("#[libertas_llm_doc_field]")
                || attrs.contains("#[libertas_llm_doc_field(blank = $null)]"),
            "{field}"
        );
    }
    // Box 8b follows box 9a's attribute set (U8): a two-decimal percent.
    let percent = "#[libertas_number(min = 0.01, max = 99.99, step = 0.01)]\n    #[libertas_physical_unit(\"percent\")]\n    pub box_";
    assert!(more.contains(&alloc::format!("{percent}8b_percent: Option<f64>")));
    assert!(more.contains(&alloc::format!("{percent}9a_percent: Option<f64>")));
    assert!(model.contains(
        "return the box 3, box 6, box 8a and box 10 amounts, or null when clearly blank; and box 8b's and box 9a's percentages as printed with up to two decimals, or null when blank"
    ));
}

// The 529-to-Roth record is box 4b: the IRA contributions page asks the
// lifetime use in 2024 and 2025, opens with a transfer present, and a return
// with the form suggests the IRA topic until it is visited.
#[test]
fn ira_page_asks_earlier_529_transfers_and_opens_with_a_box_4b_form() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_000_000, 0);
    set_qtp_roth(&mut d, Owner::Taxpayer, &[qtp_roth_rollover()]);
    assert!(
        crate::interview::derived_topics(&d, TaxSection::Adjustments)
            .contains(&TaxSection::IraContributions)
    );
    let mut s = Interview::new(TEST_USER, d);
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::IraContributions);
    let Reply::Edit(P::SaveIraContributions { allowed_owners, value, .. }) = send(&mut s, request)
    else {
        panic!("expected the IRA contributions page, not a refusal")
    };
    assert!(value.people.is_empty());
    let draft = s.draft.clone();
    let page = |prior| {
        save_ira_people(
            &draft,
            allowed_owners.clone(),
            vec![IraContributor {
                owner: Owner::Taxpayer,
                traditional: 0,
                roth: 100_000,
                covered: false,
                form_8606_value: None,
                more: IraMore { qtp_roth_prior: prior, ..IraMore::default() },
            }],
        )
    };
    accepted(&send(&mut s, page(2_000_000)));
    assert_eq!(
        s.draft.adjustments.as_ref().unwrap().iras.taxpayer.qtp_roth_lifetime_prior,
        2_000_000
    );
    // The transfer ($6,000) and the earlier $20,000 fit under $35,000.
    assert_eq!(
        crate::federal_rules::estimate(&s.draft).state,
        ResultState::EstimatedForSupportedScope
    );
    // Reopened, the page returns the stored amount.
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::IraContributions);
    let Reply::Edit(P::SaveIraContributions { value, .. }) = send(&mut s, request) else {
        panic!("expected the IRA contributions page")
    };
    assert_eq!(value.people[0].more.qtp_roth_prior, 2_000_000);
}

#[test]
fn qcd_gift_stores_its_date_and_amount_with_no_label() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Retirement,
            vec![document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::Qcd(QcdGift {
                        split_interest: false,
                        date: 20260301,
                        amount: 100_000,
                        inherited: None,
                    }),
                },
            )],
        );
        send(&mut session, request)
    });
    let charities = &session.draft.adjustments.as_ref().unwrap().qualified_charity;
    // A gift is named by its date and amount; no label text is stored.
    let payment = &charities[0].payments[0];
    assert_eq!((payment.date, payment.amount), (20260301, 100_000));
    // The stored gift survives a restart from storage unchanged.
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

// Derived Notice 2008-51 pp.2–3 + final TD8816 A-4/A-8/A-9;
// https://www.irs.gov/pub/irs-drop/n-08-51.pdf and
// https://www.irs.gov/pub/irs-regs/td8816.pdf . Neutral completion is the
// existing single wage return, own ordinary dated Roth payments, confirmed
// $800 opening regular basis, no incoming basis and full-year self-only HSA.
#[test]
fn roth_same_day_funding_order_saves_reopens_and_rejects_forged_range() {
    let mut session = restart(roth_earlier_funding());
    crate::tests::support::client::admit_native_fixture_facts(&mut session);
    let accounts = session.draft.adjustments.as_ref().unwrap().hsa.clone();
    let funding_id = session.draft.adjustments.as_ref().unwrap().hsa[0].funding[0].income;
    let ordinary_id = session.draft.income.iter().find(|entry| {
        matches!(&entry.income, FederalIncome::Roth { data } if RothWithdrawal::from(data).distributed == 20260215)
    }).unwrap().id;
    let mut page = open_topic(&mut session, TaxSection::Retirement);
    let P::SaveIncomes { followups, .. } = &mut page else { unreachable!() };
    for id in [funding_id, ordinary_id] {
        let row = followups
            .iter_mut()
            .find(|row| row.key == alloc::format!("income/{id}/treatment"))
            .unwrap();
        let crate::DocumentAnswer::RetirementTreatment {
            code:
                Some(crate::RetirementTreatmentView::RothIra {
                    roth: crate::RothIraCodeView::Early { distributed, .. },
                    sequence,
                    ..
                }),
        } = &mut row.answer
        else {
            unreachable!()
        };
        *distributed = 20260615;
        *sequence = Some(if id == funding_id { 2 } else { 1 });
    }
    accepted(&send(&mut session, page));
    assert_eq!(session.draft.adjustments.as_ref().unwrap().hsa, accounts);
    assert_eq!(return_line(&supported(&session.draft), "4b"), 10000);
    let saved = session.draft.clone();
    let mut session = restart(saved.clone());
    let mut page = open_topic(&mut session, TaxSection::Retirement);
    let P::SaveIncomes { followups, .. } = &mut page else { unreachable!() };
    for id in [funding_id, ordinary_id] {
        let row = followups
            .iter_mut()
            .find(|row| row.key == alloc::format!("income/{id}/treatment"))
            .unwrap();
        let crate::DocumentAnswer::RetirementTreatment {
            code: Some(crate::RetirementTreatmentView::RothIra { sequence, .. }),
        } = &mut row.answer
        else {
            unreachable!()
        };
        assert_eq!(*sequence, Some(if id == funding_id { 2 } else { 1 }));
        if id == ordinary_id {
            *sequence = Some(101);
        }
    }
    rejected_with(send(&mut session, page), "R_ROTH_SEQUENCE_INVALID");
    assert_eq!(session.draft, saved);
}

#[test]
fn roth_earlier_funding_public_save_reopen_and_source_change_boundaries() {
    let mut s = restart(complete(FederalStatus::Single));
    let mut rows: Vec<_> = [(30000, 20260215), (60000, 20260615), (20000, 20260915)]
        .into_iter()
        .map(|(gross, distributed)| IncomeDocumentItem {
            investment_caps: None,
            refund_management: None,
            refund_management_needed: None,
            allowed_community: Vec::new(),
            id: 0,
            owner: crate::DocumentOwner::Yours,
            preview: None,
            community: CommunityShare::NotNeeded,
            document: IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(r1099(
                    gross,
                    DistributionCode::RothIra {
                        sequence: None,
                        rolled_over: 0,
                        roth: RothIraCode::Early { distributed, exception: None, repaid: None },
                    },
                    false,
                ))),
            },
        })
        .collect();
    rows.push(IncomeDocumentItem {
        investment_caps: None,
        refund_management: None,
        refund_management_needed: None,
        allowed_community: Vec::new(),
        id: 0,
        owner: crate::DocumentOwner::Yours,
        preview: None,
        community: CommunityShare::NotNeeded,
        document: IncomeSource::Record {
            value: crate::IncomeRecord::RothHistory(RothIraHistory {
                first_year: 2010,
                amounts: Some(RothOrdering { regular: 80000, conversions: vec![] }),
            }),
        },
    });
    save_each_topic(&mut s, rows);
    let source = s
        .draft
        .income
        .iter()
        .find(|e| matches!(&e.income, FederalIncome::Roth { data } if RothWithdrawal::from(data).distributed == 20260615))
        .unwrap()
        .id;
    let mut page = open_hsa_page(&mut s);
    let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
    value.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0)];
    value.funding = vec![HsaFundingRow {
        ongoing_sep_simple: false,
        owner: Owner::Taxpayer,
        source,
        funded: 20260616,
        amount: 60000,
        basis: HsaFundingBasis::Roth { value: 70000, before_other_activity: false },
        coverage: HsaCoverage::SelfOnly,
        loss: None,
    }];
    accepted(&send(&mut s, page));
    let saved = s.draft.clone();
    let r = supported(&saved);
    assert_eq!(return_line(&r, "4b"), 10000);
    assert_eq!(super::value(&r, "early_distribution_tax"), 10);
    let mut restored = restart(saved.clone());
    let page = open_hsa_page(&mut restored);
    let P::SaveHsa { value, .. } = &page else { unreachable!() };
    assert_eq!(value.accounts[0].personal, 0);
    assert_eq!(value.funding[0].source, source);
    assert_eq!(
        value.funding[0].basis,
        HsaFundingBasis::Roth { value: 70000, before_other_activity: false }
    );
    assert_eq!(restored.draft, saved);
    assert_eq!(supported(&restored.draft), r);
    // The same accepted sources support annual regular contributions when
    // actual funding value covers opening800 + regular400 + transfer600.
    // Save the actual later payment as1100: ordinary1400 then taxes200.
    let mut incoming = restart(saved.clone());
    let mut retirement = open_documents(&mut incoming);
    let P::SaveDocuments { documents, .. } = &mut retirement else { unreachable!() };
    for row in &mut documents.common.retirement.documents {
        if let crate::RetirementDocument::R1099(data) = &mut row.document
            && data.gross == 20000
        {
            data.gross = 110000;
            data.taxable = Some(110000);
        }
    }
    accepted(&send(&mut incoming, retirement));
    let mut funding = open_hsa_page(&mut incoming);
    let P::SaveHsa { value, .. } = &mut funding else { unreachable!() };
    value.funding[0].basis = HsaFundingBasis::Roth { value: 180000, before_other_activity: false };
    accepted(&send(&mut incoming, funding));
    let (_, allowed) = open_ira_page(&mut incoming);
    let mut contribution = ira_row(Owner::Taxpayer, 0, None);
    contribution.roth = 40000;
    let save = save_ira_people(&incoming.draft, allowed, vec![contribution.clone()]);
    accepted(&send(&mut incoming, save));
    assert_eq!(return_line(&supported(&incoming.draft), "4b"), 20000);
    assert_eq!(super::value(&supported(&incoming.draft), "early_distribution_tax"), 20);
    assert_eq!(open_ira_page(&mut incoming).0, vec![contribution]);
    assert_eq!(restart(incoming.draft.clone()).draft, incoming.draft);
    // An actual contribution edit crosses the sufficient bound by one cent.
    // The raw link survives Save/restart; only its unverified exclusion leaves.
    let (mut rows, allowed) = open_ira_page(&mut incoming);
    rows[0].roth += 1;
    let save = save_ira_people(&incoming.draft, allowed, rows);
    accepted(&send(&mut incoming, save));
    assert!(noted(&supported(&incoming.draft), "WARN_HSA_FUNDING_LEFT_OUT"));
    let reopened = open_hsa_page(&mut incoming);
    let P::SaveHsa { value, .. } = reopened else { unreachable!() };
    assert_eq!(value.funding[0].source, source);
    assert_eq!(value.funding[0].amount, 60000);
    assert_eq!(restart(incoming.draft.clone()).draft, incoming.draft);
    // Editing the real source to combine ordinary and funding amounts retains
    // the accepted link, but leaves this unverified exclusion out with a notice.
    let mut income_page = open_documents(&mut restored);
    let P::SaveDocuments { documents, .. } = &mut income_page else { unreachable!() };
    let row = documents.common.retirement.documents.iter_mut().find(|e| e.id == source).unwrap();
    let crate::RetirementDocument::R1099(data) = &mut row.document else { unreachable!() };
    data.gross += 100;
    data.taxable = Some(data.taxable.unwrap_or(0) + 100);
    accepted(&send(&mut restored, income_page));
    let changed = supported(&restored.draft);
    assert!(noted(&changed, "WARN_HSA_FUNDING_LEFT_OUT"));
    assert_eq!(return_line(&changed, "4b"), 30100);
    let reopened = open_hsa_page(&mut restored);
    let P::SaveHsa { value, .. } = reopened else { unreachable!() };
    assert_eq!(value.funding[0].source, source);
    assert_eq!(value.funding[0].amount, 60000);
}

#[test]
fn inherited_roth_earlier_funding_public_save_reopen_and_source_correction() {
    let mut s = restart(complete(FederalStatus::Single));
    let mut rows: Vec<_> = [(30000, 20260215), (60000, 20260615), (20000, 20260915)]
        .into_iter()
        .map(|(gross, distributed)| IncomeDocumentItem {
            investment_caps: None,
            refund_management: None,
            refund_management_needed: None,
            allowed_community: Vec::new(),
            id: 0,
            owner: crate::DocumentOwner::Yours,
            preview: None,
            community: CommunityShare::NotNeeded,
            document: IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(r1099(
                    gross,
                    DistributionCode::RothIra {
                        sequence: None,
                        rolled_over: 0,
                        roth: RothIraCode::ExceptionApplies {
                            distributed,
                            reason: RothReason::Inherited { account: None },
                        },
                    },
                    false,
                ))),
            },
        })
        .collect();
    rows.push(IncomeDocumentItem {
        investment_caps: None,
        refund_management: None,
        refund_management_needed: None,
        allowed_community: Vec::new(),
        id: 0,
        owner: crate::DocumentOwner::Yours,
        preview: None,
        community: CommunityShare::NotNeeded,
        document: IncomeSource::Record {
            value: crate::IncomeRecord::InheritedIra(InheritedIraAccount {
                decedent: "Synthetic source owner".into(),
                died: 20251201,
                kind: InheritedIraKindFacts::Roth {
                    first_year: 2023,
                    basis: Some(InheritedRothBasis { regular: 80000, converted: 0 }),
                },
            }),
        },
    });
    save_each_topic(&mut s, rows);
    let source = s
        .draft
        .income
        .iter()
        .find(|e| matches!(&e.income, FederalIncome::Roth { data } if RothWithdrawal::from(data).distributed == 20260615))
        .unwrap()
        .id;
    let mut health = open_hsa_page(&mut s);
    let P::SaveHsa { value, .. } = &mut health else { unreachable!() };
    value.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0)];
    value.funding = vec![HsaFundingRow {
        ongoing_sep_simple: false,
        owner: Owner::Taxpayer,
        source,
        funded: 20260616,
        amount: 60000,
        basis: HsaFundingBasis::Roth { value: 70000, before_other_activity: false },
        coverage: HsaCoverage::SelfOnly,
        loss: None,
    }];
    accepted(&send(&mut s, health));
    let result = supported(&s.draft);
    assert!(!noted(&result, "WARN_HSA_FUNDING_LEFT_OUT"));
    assert_eq!(return_line(&result, "4b"), 10000);
    assert_eq!(super::value(&result, "early_distribution_tax"), 0);
    let saved = s.draft.clone();
    let mut contradicted = restart(saved.clone());
    let mut health = open_hsa_page(&mut contradicted);
    let P::SaveHsa { value, .. } = &mut health else { unreachable!() };
    value.funding[0].basis = HsaFundingBasis::Roth { value: 70000, before_other_activity: true };
    rejected_with(send(&mut contradicted, health), "HSA_FUNDING_ORDER_CONTRADICTED");
    assert_eq!(contradicted.draft, saved);
    let mut health = open_hsa_page(&mut contradicted);
    let P::SaveHsa { value, .. } = &mut health else { unreachable!() };
    assert_eq!(value.funding[0].source, source);
    value.funding[0].basis = HsaFundingBasis::Roth { value: 70000, before_other_activity: false };
    accepted(&send(&mut contradicted, health));
    assert_eq!(supported(&contradicted.draft), result);
    assert_eq!(restart(contradicted.draft.clone()).draft, contradicted.draft);
    let ordinary_id = saved
        .income
        .iter()
        .find(|e| matches!(&e.income, FederalIncome::Roth { data } if RothWithdrawal::from(data).distributed == 20260215))
        .unwrap()
        .id;
    let move_ordinary = |request: &mut P, date| {
        let P::SaveIncomes { followups, .. } = request else { unreachable!() };
        let crate::DocumentAnswer::RetirementTreatment {
            code:
                Some(crate::RetirementTreatmentView::RothIra {
                    roth: crate::RothIraCodeView::ExceptionApplies { distributed, .. },
                    ..
                }),
        } = &mut followups
            .iter_mut()
            .find(|row| row.key == alloc::format!("income/{ordinary_id}/treatment"))
            .unwrap()
            .answer
        else {
            unreachable!()
        };
        *distributed = date;
    };
    // Retirement answers use the same consistency check as Health-account edits.
    let mut retirement = open_topic(&mut contradicted, TaxSection::Retirement);
    move_ordinary(&mut retirement, 20260715);
    accepted(&send(&mut contradicted, retirement));
    let mut health = open_hsa_page(&mut contradicted);
    let P::SaveHsa { value, .. } = &mut health else { unreachable!() };
    value.funding[0].basis = HsaFundingBasis::Roth { value: 70000, before_other_activity: true };
    accepted(&send(&mut contradicted, health));
    let confirmed = contradicted.draft.clone();
    let mut retirement = open_topic(&mut contradicted, TaxSection::Retirement);
    move_ordinary(&mut retirement, 20260215);
    let path = retirement_followup_path(&retirement, ordinary_id, "code/roth/distributed");
    crate::tests::support::client::survey::survey_refusal(
        "health-context-09",
        &mut contradicted,
        retirement,
        "HSA_FUNDING_ORDER_CONTRADICTED",
        &path,
    );
    assert_eq!(contradicted.draft, confirmed);
    let mut health = open_hsa_page(&mut contradicted);
    let P::SaveHsa { value, .. } = &mut health else { unreachable!() };
    value.funding[0].basis = HsaFundingBasis::Roth { value: 70000, before_other_activity: false };
    accepted(&send(&mut contradicted, health));
    let mut retirement = open_topic(&mut contradicted, TaxSection::Retirement);
    move_ordinary(&mut retirement, 20260215);
    accepted(&send(&mut contradicted, retirement));
    assert_eq!(return_line(&supported(&contradicted.draft), "4b"), 10000);
    let mut restored = restart(saved.clone());
    let reopened = open_hsa_page(&mut restored);
    let P::SaveHsa { value, .. } = &reopened else { unreachable!() };
    assert_eq!(value.funding[0].source, source);
    assert_eq!(
        value.funding[0].basis,
        HsaFundingBasis::Roth { value: 70000, before_other_activity: false }
    );
    accepted(&send(&mut restored, reopened));
    assert_eq!(supported(&restored.draft), result);
    // Correcting the actual source to include an additional ordinary dollar
    // preserves its ID/link but leaves this ambiguous funding exclusion out.
    let mut retirement = open_documents(&mut restored);
    let P::SaveDocuments { documents, .. } = &mut retirement else { unreachable!() };
    let row = documents.common.retirement.documents.iter_mut().find(|e| e.id == source).unwrap();
    let crate::RetirementDocument::R1099(data) = &mut row.document else { unreachable!() };
    data.gross += 100;
    data.taxable = Some(data.taxable.unwrap_or(0) + 100);
    accepted(&send(&mut restored, retirement));
    let changed = supported(&restored.draft);
    assert!(noted(&changed, "WARN_HSA_FUNDING_LEFT_OUT"));
    assert_eq!(return_line(&changed, "4b"), 30100);
    let reopened = open_hsa_page(&mut restored);
    let P::SaveHsa { value, .. } = reopened else { unreachable!() };
    assert_eq!(value.funding[0].source, source);
    assert_eq!(value.funding[0].amount, 60000);
    assert_eq!(restart(restored.draft.clone()).draft, restored.draft);
}

#[test]
fn retirement_follow_up_calls_follow_printed_boxes_without_editing_the_return() {
    let original = complete(FederalStatus::Single);
    let session = Interview::new(TEST_USER, original.clone());
    // Form 1099-R instructions, boxes 7a/7b/7c/10; Pub. 575 disability,
    // public-safety insurance and recapture allocation; Pub. 590-B disaster relief.
    // 2026-10-22: align calls with treatment_issue's existing exclusions and
    // Roth-IRA Qualified-only disaster support. Code G/H premium calls retain
    // the owner's accepted ordinary-income and zero-funded-premium paths.
    for (printed, ira, trump, box_10, disaster, public_safety, disability, basis) in [
        ("7", false, false, 0, vec![1, 2, 3], vec![0], vec![1], vec![]),
        ("7", true, false, 0, vec![1, 2, 3], vec![], vec![], vec![]),
        ("3", false, false, 0, vec![1, 2, 3], vec![0], vec![1], vec![]),
        ("7D", false, false, 0, vec![], vec![], vec![1], vec![]),
        ("1B", false, false, 10_000, vec![1, 2, 3], vec![0], vec![], vec![0]),
        ("1B", false, false, 0, vec![1, 2, 3], vec![0], vec![], vec![]),
        ("J", false, false, 0, vec![1], vec![], vec![], vec![]),
        ("Q", false, false, 0, vec![1], vec![], vec![], vec![]),
        ("T", false, false, 0, vec![1], vec![], vec![], vec![]),
        ("4", false, false, 0, vec![1, 2, 3], vec![0], vec![], vec![]),
        ("4", true, false, 0, vec![], vec![], vec![], vec![]),
        ("4K", true, false, 0, vec![], vec![], vec![], vec![]),
        ("4A", true, false, 0, vec![], vec![], vec![], vec![]),
        ("A", true, false, 0, vec![1, 2, 3], vec![], vec![], vec![]),
        ("7Y", true, false, 0, vec![1, 2, 3], vec![], vec![], vec![]),
        ("5", true, false, 0, vec![1, 2, 3], vec![], vec![], vec![]),
        ("A", false, false, 0, vec![1, 2, 3], vec![0], vec![], vec![]),
        ("W", false, false, 0, vec![], vec![], vec![], vec![]),
        ("6W", false, false, 0, vec![], vec![], vec![], vec![]),
        ("6", false, false, 0, vec![], vec![], vec![], vec![]),
        ("8", false, false, 0, vec![], vec![], vec![], vec![]),
        ("N", false, false, 0, vec![], vec![], vec![], vec![]),
        ("R", false, false, 0, vec![], vec![], vec![], vec![]),
        ("P", false, false, 0, vec![], vec![], vec![], vec![]),
        ("9", false, false, 0, vec![], vec![], vec![], vec![]),
        ("E", false, false, 0, vec![], vec![], vec![], vec![]),
        ("U", false, false, 0, vec![], vec![], vec![], vec![]),
        ("G", false, false, 0, vec![], vec![0], vec![], vec![]),
        ("H", false, false, 0, vec![], vec![0], vec![], vec![]),
        ("4", false, true, 0, vec![], vec![], vec![], vec![]),
    ] {
        let (_, _, calls) =
            retirement_question_catalog(&session, box7(printed), ira, trump, box_10);
        assert_eq!(
            calls,
            crate::RCalls { disaster, public_safety, disability, box_10_basis: basis },
            "{printed}, IRA {ira}, Trump {trump}, box 10 {box_10}"
        );
    }
    assert_eq!(session.draft, original);
}

#[test]
fn retirement_follow_up_calls_rebuild_on_reopen_and_accepted_answers_stay_authoritative() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut pension = r1099(1_000_000, normal_kept(), false);
    pension.source_mut().disability =
        Some(DisabilityIncome::Qualifying { amount: 200_000, eligible: true });
    // Derived calls are ignored on Save. A withheld presentation row cannot
    // erase the accepted disability amount or change its tax treatment.
    accepted(&crate::tests::support::client::save_retirement_fixture(
        &mut session,
        vec![document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(pension)),
            },
        )],
    ));
    let accepted_return = session.draft.clone();
    let mut cold = restart(accepted_return.clone());
    let request = open_documents(&mut cold);
    let P::SaveDocuments { documents, .. } = &request else {
        panic!("expected the Document center")
    };
    let crate::RetirementDocument::R1099(reopened) =
        &documents.common.retirement.documents[0].document
    else {
        panic!("expected a pension document")
    };
    let (_, _, calls) = retirement_question_catalog(
        &cold,
        reopened.box7,
        reopened.ira,
        reopened.trump,
        reopened.box_10.unwrap_or(0),
    );
    assert_eq!(calls.disability, vec![1]);
    let FederalIncome::Retirement { data: RetirementSource::R1099(source) } =
        &cold.draft.income.last().unwrap().income
    else {
        unreachable!()
    };
    assert_eq!(
        source.disability,
        Some(DisabilityIncome::Qualifying { amount: 200_000, eligible: true })
    );
    accepted(&send(&mut cold, request));
    assert_eq!(cold.draft, accepted_return);
}

#[test]
fn printed_1099_r_blank_amounts_and_printed_zero_survive_save_and_reopen() {
    use crate::tests::support::builders::assert_retirement_paper_reopens;
    for amount in [None, Some(0)] {
        let paper = crate::Form1099RPaper {
            gross: 100_000,
            box7: DistributionCodes::CODE_7.0,
            taxable: amount,
            box_3: amount,
            withholding: amount,
            employee_contributions: amount,
            box_6: amount,
            excess_contribution_earnings: amount,
            box_8a: amount,
            box_10: amount,
            payer_telephone: if amount.is_some() {
                "800-555-0100".into()
            } else {
                alloc::string::String::new()
            },
            account_number: if amount.is_some() {
                "00001234".into()
            } else {
                alloc::string::String::new()
            },
            ..Default::default()
        };
        assert_eq!(crate::R1099::from(&paper).taxable, amount);
        assert_retirement_paper_reopens(crate::RetirementDocument::R1099(paper));
    }
}

#[test]
fn printed_rrb_1099_r_blank_amounts_and_printed_zero_survive_save_and_reopen() {
    use crate::tests::support::builders::assert_retirement_paper_reopens;
    for amount in [None, Some(0)] {
        let paper = crate::FormRrb1099RPaper {
            claim_number_and_payee_code: if amount.is_some() {
                "A 000012345".into()
            } else {
                alloc::string::String::new()
            },
            box_4: amount,
            gross: 0,
            box_5: amount,
            box_6: amount,
            box_8: amount,
            withholding: amount,
            medicare_premiums: amount,
            ..Default::default()
        };
        assert_eq!(crate::Rrb1099R::from(&paper).withholding, amount);
        assert_retirement_paper_reopens(crate::RetirementDocument::Rrb1099R(paper));
    }
}
