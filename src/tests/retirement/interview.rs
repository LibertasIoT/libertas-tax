// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::*;
use crate::interview::document_income;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::{document, owned};
use super::support::client::income_pages::{
    open_documents, open_topic, page_documents, save_documents, save_topic,
};
use crate::tests::cross::interview::{
    PAGE_ADJUSTMENTS, PAGE_CREDITS, PAGE_REVIEW, assert_available_sections,
    at_the_adjustments_hub_with_business, at_the_credits_hub, continued, credits_hub, early_kept,
    excess_entry, go_on, go_to, hsa_account, hub_rows, ira_row, normal_kept, open_adjustments_hub,
    open_credits_topic, open_hsa_page, open_ira_page, other_1099, r1099, roth_item_for_tests,
    row_of, se_health_test_year, send_past_less_common, to_the_hub, value_of, with_form_8606,
};
const PAGE_SAVER: i32 = TaxSection::Saver as i32;
pub(in crate::tests) fn retirement_treatment(
    code: DistributionCode,
    ira: bool,
) -> RetirementTreatment {
    let FederalIncome::Retirement { data } =
        document_income(&IncomeDocument::R1099(r1099(1_000_000, code, ira)))
    else {
        unreachable!()
    };
    data.ordinary_distribution
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
    direct.taxable = 0;
    let FederalIncome::Retirement { data } = document_income(&IncomeDocument::R1099(direct)) else {
        unreachable!()
    };
    assert_eq!(
        data.ordinary_distribution,
        RetirementTreatment::Rollover {
            destination: crate::RolloverDestination::EmployerPlan,
            amount: 1_000_000,
            early: false,
        }
    );
    // S, kept.
    assert_eq!(
        retirement_treatment(
            DistributionCode::EarlySimple { what_happened: SimpleDisposition::Kept },
            true
        ),
        RetirementTreatment::EarlySimple
    );
    // D, a nonqualified annuity.
    let FederalIncome::Retirement { data } = document_income(&IncomeDocument::R1099(r1099(
        1_000_000,
        DistributionCode::Nonqualified,
        false,
    ))) else {
        unreachable!()
    };
    assert_eq!(data.kind, RetirementKind::Nonqualified);
    assert_eq!(data.ordinary_distribution, RetirementTreatment::Ordinary);
}

#[test]
fn form_8606_switches_that_owners_iras_to_the_basis_worksheet() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let ira_r1099 =
        r1099(1_000_000, DistributionCode::Normal { what_happened: Disposition::Kept }, true);
    let mut ira_r1099 = R1099 { taxable: 800_000, ..ira_r1099 };
    send(
        &mut session,
        save_topic(
            &original,
            TaxSection::Retirement,
            vec![document(0, IncomeDocument::R1099(ira_r1099.clone()))],
        ),
    );
    let entry_id = session.draft.income[1].id;
    let FederalIncome::Retirement { data } = &session.draft.income[1].income else {
        unreachable!()
    };
    // Without a Form 8606, the IRA distribution uses the payer's box 2a.
    assert_eq!(data.taxable, RetirementTaxable::Payer { amount: 800_000 });

    // Adding this owner's Form 8606 switches the same IRA distribution to
    // the basis worksheet, never asking the box 2a question again.
    let draft = session.draft.clone();
    send(
        &mut session,
        save_topic(
            &draft,
            TaxSection::Retirement,
            vec![
                document(entry_id, IncomeDocument::R1099(ira_r1099.clone())),
                document(
                    0,
                    IncomeDocument::Form8606(Form8606 {
                        adjustment: 0,
                        reservist_repayments: vec![],
                        basis: 500_000,
                        year_end: 900_000,
                    }),
                ),
            ],
        ),
    );
    let FederalIncome::Retirement { data } = &session.draft.income[1].income else {
        unreachable!()
    };
    assert_eq!(data.taxable, RetirementTaxable::IraBasis);
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

    // Removing the Form 8606 turns the IRA document back to box 2a.
    ira_r1099.taxable = 750_000;
    let draft = session.draft.clone();
    send(
        &mut session,
        save_topic(
            &draft,
            TaxSection::Retirement,
            vec![document(entry_id, IncomeDocument::R1099(ira_r1099))],
        ),
    );
    let FederalIncome::Retirement { data } = &session.draft.income[1].income else {
        unreachable!()
    };
    assert_eq!(data.taxable, RetirementTaxable::Payer { amount: 750_000 });
    assert!(session.draft.adjustments.as_ref().unwrap().ira_basis.is_empty());
}

#[test]
fn form_8606_twice_for_the_same_owner_is_rejected() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let form =
        Form8606 { adjustment: 0, reservist_repayments: vec![], basis: 100_000, year_end: 200_000 };
    let reply = send(
        &mut session,
        save_topic(
            &original,
            TaxSection::Retirement,
            vec![
                document(0, IncomeDocument::Form8606(form.clone())),
                document(0, IncomeDocument::Form8606(form)),
            ],
        ),
    );
    rejected_with(reply, "FORM_8606_TWICE");
    assert_eq!(session.draft, original);
}

#[test]
fn qcd_gifts_group_by_owner() {
    let original = complete(FederalStatus::Joint);
    let mut session = Interview::new(TEST_USER, original.clone());
    send(
        &mut session,
        save_topic(
            &original,
            TaxSection::Retirement,
            vec![
                owned(
                    Owner::Taxpayer,
                    0,
                    IncomeDocument::Qcd(QcdGift {
                        split_interest: false,
                        date: 20260301,
                        amount: 100_000,
                        inherited: None,
                    }),
                ),
                owned(
                    Owner::Taxpayer,
                    0,
                    IncomeDocument::Qcd(QcdGift {
                        split_interest: false,
                        date: 20260601,
                        amount: 50_000,
                        inherited: None,
                    }),
                ),
                owned(
                    Owner::Spouse,
                    0,
                    IncomeDocument::Qcd(QcdGift {
                        split_interest: false,
                        date: 20260815,
                        amount: 75_000,
                        inherited: None,
                    }),
                ),
            ],
        ),
    );
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
    let reject = |r: R1099, key: &str| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = send(
            &mut session,
            save_documents(&original, vec![document(0, IncomeDocument::R1099(r))]),
        );
        let Reply::Response(P::Problem { message, .. }) = reply else {
            panic!("expected a problem for {key}, got {reply:?}")
        };
        assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
        assert_eq!(session.draft, original);
    };
    // Box 2a cannot exceed box 1.
    let mut over_taxable =
        r1099(1_000_000, DistributionCode::Normal { what_happened: Disposition::Kept }, false);
    over_taxable.taxable = 1_000_001;
    reject(over_taxable, "R_TAXABLE_ABOVE_GROSS");
    // Box 4 cannot exceed box 1.
    let mut over_withheld =
        r1099(1_000_000, DistributionCode::Normal { what_happened: Disposition::Kept }, false);
    over_withheld.withholding = 1_000_001;
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
        with_box_9a.more = RMore { box_9a_percent: Some(percent), ..RMore::default() };
        let mut session = Interview::new(TEST_USER, original.clone());
        accepted(&send(
            &mut session,
            save_documents(&original, vec![document(0, IncomeDocument::R1099(with_box_9a))]),
        ));
    }
    // Box 10 is for a designated Roth account distribution (code B), and no more than box 1.
    let mut with_box_10 =
        r1099(1_000_000, DistributionCode::Normal { what_happened: Disposition::Kept }, false);
    with_box_10.more = RMore { box_10: 1, ..RMore::default() };
    reject(with_box_10.clone(), "R_BOX10_WITHOUT_B");
    let mut designated = printed_as(with_box_10, "7B");
    let mut session = Interview::new(TEST_USER, original.clone());
    accepted(&send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::R1099(designated.clone()))]),
    ));
    designated.more.box_10 = 1_000_001;
    reject(designated, "R_BOX10_ABOVE_GROSS");
}

#[test]
fn box_7a_rules_reject_on_both_sides_at_their_boundaries() {
    let original = complete(FederalStatus::Single);
    let reject = |r: R1099, key: &str| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = send(
            &mut session,
            save_documents(&original, vec![document(0, IncomeDocument::R1099(r))]),
        );
        let Reply::Response(P::Problem { message, .. }) = reply else {
            panic!("expected a problem for {key}, got {reply:?}")
        };
        assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
        assert_eq!(session.draft, original);
    };
    let accept = |r: R1099| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = send(
            &mut session,
            save_documents(&original, vec![document(0, IncomeDocument::R1099(r))]),
        );
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
    recharacterized.taxable = 0;
    accept(recharacterized);
    let mut annuity = r1099(1_000_000, DistributionCode::Nonqualified, false);
    accept(annuity.clone());
    annuity.ira = true;
    reject(annuity, "R_BOX7B_NOT_IRA");
    reject(other_1099(1_000_000, "K", false), "R_BOX7B_NEEDED");
    accept(printed_as(r1099(1_000_000, normal_kept(), true), "K"));
    let mut simple = r1099(
        1_000_000,
        DistributionCode::EarlySimple { what_happened: SimpleDisposition::Kept },
        true,
    );
    accept(simple.clone());
    simple.ira = false;
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
        DistributionCode::EarlySimple { what_happened: SimpleDisposition::Kept },
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
        DistributionCode::TrumpDeath { estate: false },
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
    // Box 7c: a Trump account death (4) and the transfer to the child's ABLE account (G); any
    // other code, a returned excess (8) included, and box 7b with it, is Another code.
    for (printed, ira, branch) in
        [("4", false, 13), ("G", false, 14), ("8", false, 8), ("7", false, 8), ("4", true, 8)]
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
    // Box7Options is the row's calculation: its one-entry reply is the
    // singleton the client selects (Normal 0, DirectRollover 4, RothIra 7,
    // Other 8).
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
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
        let reply = send(
            &mut session,
            P::Box7Options { user: TEST_USER, box7: box7(printed), ira, trump: false },
        );
        let Reply::Response(P::Box7Treatments { allowed_treatments, .. }) = reply else {
            panic!("expected the treatments for {printed}, got {reply:?}")
        };
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
    assert_eq!(normal.allowed_treatments, vec![0]);
    assert_eq!(rollover.allowed_treatments, vec![4]);
    assert_eq!(other.allowed_treatments, vec![8]);
    let reply = send(
        &mut session,
        save_documents(
            &original,
            vec![
                document(0, IncomeDocument::R1099(normal)),
                document(0, IncomeDocument::R1099(rollover)),
                document(0, IncomeDocument::R1099(other)),
            ],
        ),
    );
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let mut session = restart(session.draft);
    let documents = page_documents(&open_topic(&mut session, TaxSection::Retirement)).to_vec();
    let mut sets: Vec<(u32, Vec<i32>)> = documents
        .iter()
        .map(|d| {
            let IncomeDocument::R1099(r) = &d.document else { unreachable!() };
            assert!(DistributionCodes::leads_to(r.box7, r.ira, r.trump, &r.code));
            (r.box7, r.allowed_treatments.clone())
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
        let reply = send(
            &mut session,
            save_documents(&original, vec![document(0, IncomeDocument::R1099(r.clone()))]),
        );
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        let FederalIncome::Retirement { data } = &session.draft.income[1].income else {
            unreachable!()
        };
        assert_eq!(data.box7, box7(printed));
        assert_eq!(data.ordinary_distribution, RetirementTreatment::Unsupported);
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
        let documents = page_documents(&open_topic(&mut session, TaxSection::Retirement)).to_vec();
        assert_eq!(documents[0].document, IncomeDocument::R1099(r));
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
        document(0, IncomeDocument::R1099(early("1"))),
        document(0, IncomeDocument::R1099(early("2"))),
        document(
            0,
            IncomeDocument::R1099(r1099(
                1_000_000,
                DistributionCode::Disability { began: 20250101, wages: DisabilityWages::None },
                false,
            )),
        ),
        document(
            0,
            IncomeDocument::R1099(r1099(
                1_000_000,
                DistributionCode::Normal {
                    what_happened: Disposition::RolledOver {
                        amount: 1_000_000,
                        into: crate::RolloverDestination::Ira,
                    },
                },
                true,
            )),
        ),
        document(
            0,
            IncomeDocument::R1099(r1099(
                1_000_000,
                DistributionCode::DirectRollover { to: RolloverTarget::Ira },
                false,
            )),
        ),
    ];
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = send(&mut session, save_documents(&original, sent.clone()));
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let saved = session.draft.clone();
    let mut session = restart(saved.clone());
    let documents = page_documents(&open_topic(&mut session, TaxSection::Retirement)).to_vec();
    let mut got: Vec<_> = documents.iter().map(|d| d.document.clone()).collect();
    let mut want: Vec<_> = sent.iter().map(|d| d.document.clone()).collect();
    let key = |d: &IncomeDocument| alloc::format!("{d:?}");
    got.sort_by_key(key);
    want.sort_by_key(key);
    assert_eq!(got, want);
}

#[test]
fn ssa_1099_reaches_line_6() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    send(
        &mut session,
        save_documents(
            &original,
            vec![document(
                0,
                IncomeDocument::Ssa1099(Ssa1099 {
                    statement: Default::default(),
                    benefits: 1_800_000,
                    withholding: 40_000,
                }),
            )],
        ),
    );
    let FederalIncome::SocialSecurity { data } = &session.draft.income[1].income else {
        unreachable!()
    };
    assert_eq!(
        *data,
        FederalSocial { benefits: 1_800_000, withholding: 40_000, special_treatment: false }
    );
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
    let documents = page_documents(&open_topic(&mut session, TaxSection::Retirement)).to_vec();
    assert_eq!(
        documents[0].document,
        IncomeDocument::Ssa1099(Ssa1099 {
            statement: Default::default(),
            benefits: 1_800_000,
            withholding: 40_000,
        })
    );
}
// A minimal Schedule C business with only an owner-only SEP, for the IRA
// "covered by a plan at work" derivation.
fn business_with_sep(receipts: i64, contribution: i64, rate: u32) -> FederalIncome {
    FederalIncome::Business {
        data: FederalBusiness {
            activity: BusinessActivity::Ordinary {
                returns: 0,
                receipts,
                accounting: BusinessAccounting::Cash,
            },
            retirement: BusinessRetirement::Sep { contribution, rate },
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
        estimate: Some(vec![]),
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
    let Reply::Response(_) = to_the_hub(&mut s, reply) else {
        panic!("expected the adjustments hub")
    };
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
            data.retirement_plan = true;
        }
        if sep {
            add(&mut d, income(business_with_sep(1_000_000, 100_000, 2500)));
        }
        let mut s = Interview::new(TEST_USER, d);
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
        assert!(matches!(send(&mut s, request), Reply::Response(P::Review { .. })));
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
        estimate: vec![],
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
        data.other_box12 = box12;
    }
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
    // M3-P19: the previous and next shown sections: the adjustments hub
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
    let hub = credits_hub(send_past_less_common(&mut s, request));
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
    // M3-P19: Continue goes to the next shown section, other taxes.
    assert_eq!(*suggested, TaxSection::OtherTaxes);
    assert_eq!(value_of(&continued(&mut s), "saver_credit"), 200);
    // It reopens as entered, the workplace rows derived again.
    let page = open_credits_topic(&mut s, TaxSection::Saver);
    assert_eq!(saver_value(&page), &SaverCredit { from_w2: value.from_w2.clone(), ..entered });
    // Lowering box 12 D on the Wages page re-derives the workplace part and
    // keeps the $300 not on a W-2.
    let mut wages = open_documents(&mut s);
    let P::SaveIncomes { documents, .. } = &mut wages else { unreachable!() };
    let IncomeDocument::W2(w) = &mut documents.documents[0].document else {
        panic!("expected the W-2")
    };
    for entry in w.more.box12.iter_mut() {
        if entry.code == W2Code::D {
            entry.amount = 100_000;
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
        // M3-P19: Continue goes to the next shown section, the review.
        assert_eq!(
            *suggested,
            if hinted { TaxSection::Saver } else { TaxSection::OtherTaxes },
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
    assert!(matches!(send(&mut s, request), Reply::Response(P::Review { .. })));
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
    let (_, allowed) = open_ira_page(&mut s);
    let request =
        save_ira_people(&s.draft, allowed.clone(), vec![ira_row(Owner::Taxpayer, 0, None)]);
    assert!(matches!(send(&mut s, request), Reply::Response(P::Review { .. })));
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
    assert!(matches!(send(&mut s, request), Reply::Response(P::Review { .. })));
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
    // D-2026-10-15-13: a duplicate native projection cannot cap actual
    // Form8606 sources. MAX Cents plus the real $1,000 next-year contribution
    // is exact and known; line17 still applies 6% to its actual $1,500 cap.
    let ordinary = estimate(&s.draft);
    with_form_8606(&mut s, i64::MAX);
    assert!(s.draft.valid_stored());
    assert_eq!(
        s.draft
            .adjustments
            .as_ref()
            .unwrap()
            .iras
            .taxpayer
            .excess
            .as_ref()
            .unwrap()
            .traditional_value,
        None
    );
    assert_eq!(value(&estimate(&s.draft), "ira_excess_taxpayer_line17_traditional_excess_tax"), 90);
    assert_eq!(estimate(&s.draft).tax, ordinary.tax);
    let saved = s.draft.clone();
    let mut again = restart(saved.clone());
    let (rows, allowed) = open_ira_page(&mut again);
    assert_eq!(rows[0].form_8606_value, Some(i64::MAX));
    assert_eq!(rows[0].more.excess.as_ref().unwrap().traditional_value, None);
    let request = save_ira_people(&again.draft, allowed, rows);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft, saved);
    let mut missing = saved.clone();
    missing.adjustments.as_mut().unwrap().ira_basis.clear();
    assert!(!missing.valid_stored(), "None without its actual source is unavailable");
    let mut dishonest = saved;
    dishonest
        .adjustments
        .as_mut()
        .unwrap()
        .iras
        .taxpayer
        .excess
        .as_mut()
        .unwrap()
        .traditional_value = Some(0);
    // 2026-10-16: app-contract.md, Validation: derived fields are recomputed
    // on page construction and ignored on Save. A stale duplicate cannot
    // replace the known Form 8606 operands, even when they exceed native cents.
    assert!(dishonest.valid_stored());
    assert_eq!(
        value(&estimate(&dishonest), "ira_excess_taxpayer_line17_traditional_excess_tax"),
        90
    );
    assert_eq!(estimate(&dishonest).tax, ordinary.tax);
    let mut stale = restart(dishonest);
    let (rows, allowed) = open_ira_page(&mut stale);
    assert_eq!(rows[0].more.excess.as_ref().unwrap().traditional_value, None);
    let request = save_ira_people(&stale.draft, allowed, rows);
    accepted(&send(&mut stale, request));
    let mut canonical = again.draft.clone();
    canonical.revision += 1; // Replacing the stale stored projection is one accepted write.
    assert_eq!(stale.draft, canonical);
}

// Form 4972 on the 1099-R page (lump_sum.rs; unit B2).
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
fn seven_a(tax: crate::LumpSumTax) -> R1099 {
    r1099(1_750_000, DistributionCode::NormalLumpSum { tax }, false)
}
fn four_a(tax: crate::InheritedLumpSumTax) -> R1099 {
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
    assert_eq!(normal.allowed_treatments, vec![9]);
    let death = four_a(crate::InheritedLumpSumTax::WithOtherIncome);
    assert_eq!(death.box7, box7("4A"));
    assert_eq!(death.allowed_treatments, vec![10]);
    // A 7 or 4 page branch cannot be saved against 7A or 4A.
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let wrong = printed_as(
        r1099(1_000_000, DistributionCode::Normal { what_happened: Disposition::Kept }, false),
        "7A",
    );
    let reply = send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::R1099(wrong))]),
    );
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
    own.taxable = 1_500_000;
    own.more = RMore {
        box_3: 100_000,
        box_6: 300_000,
        box_8a: 200_000,
        box_8b_percent: Some(33.33),
        box_9a_percent: Some(40.0),
        ..RMore::default()
    };
    let FederalIncome::Retirement { data } = document_income(&IncomeDocument::R1099(own.clone()))
    else {
        unreachable!()
    };
    let RetirementTreatment::LumpSum { data: stored } = &data.ordinary_distribution else {
        panic!("{:?}", data.ordinary_distribution)
    };
    assert_eq!(
        stored.participant,
        crate::Form4972Participant::Own { event: crate::LumpSumEvent::Age }
    );
    assert_eq!(
        (
            stored.method,
            stored.capital_gain,
            stored.nua,
            stored.include_nua,
            stored.annuity_value,
            stored.annuity_percent,
            stored.percent
        ),
        (Both, 100_000, 300_000, true, 200_000, Some(3_333), Some(4_000))
    );
    let mut inherited = four_a(inherited_tax(TenYear));
    inherited.more = RMore { box_8a: 50_000, ..RMore::default() };
    let FederalIncome::Retirement { data } =
        document_income(&IncomeDocument::R1099(inherited.clone()))
    else {
        unreachable!()
    };
    let RetirementTreatment::LumpSum { data: stored } = &data.ordinary_distribution else {
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
    let reject = |r: R1099, key: &str| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = send(
            &mut session,
            save_documents(&original, vec![document(0, IncomeDocument::R1099(r))]),
        );
        let Reply::Response(P::Problem { message, .. }) = reply else {
            panic!("expected a problem for {key}, got {reply:?}")
        };
        assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
        assert_eq!(session.draft, original);
    };
    let accept = |r: R1099| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = send(
            &mut session,
            save_documents(&original, vec![document(0, IncomeDocument::R1099(r))]),
        );
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        session.draft
    };
    // Box 3 above box 2a; box 8b without box 8a.
    let mut r = seven_a(lump_tax(TenYear, false));
    r.more.box_3 = 1_750_001;
    reject(r, "R_BOX3_ABOVE_TAXABLE");
    let mut r = seven_a(lump_tax(TenYear, false));
    r.more.box_8b_percent = Some(50.0);
    reject(r, "R_BOX8B_WITHOUT_BOX8A");
    // The capital gain rate needs box 3; the NUA needs box 6.
    for method in [CapitalGain, Both] {
        reject(seven_a(lump_tax(method, false)), "LUMP_SUM_CAPITAL_GAIN_WITHOUT_BOX_3");
        let mut r = four_a(inherited_tax(method));
        r.more.box_6 = 0;
        reject(r, "LUMP_SUM_CAPITAL_GAIN_WITHOUT_BOX_3");
    }
    reject(seven_a(lump_tax(TenYear, true)), "LUMP_SUM_NUA_WITHOUT_BOX_6");
    // No other special treatment alongside Form 4972 (the Simplified Method, box 10, a disaster
    // distribution and public safety premiums are refused by the same rule).
    let mut r = seven_a(lump_tax(TenYear, false));
    r.more.disability = DisabilityIncome::Qualifying { amount: 100_000, eligible: true };
    reject(r, "LUMP_SUM_OTHER_TREATMENTS");
    // The first payment is in 2026, the latest earlier payment before it, the birth before the death.
    let mut r = seven_a(lump_tax(TenYear, false));
    let crate::DistributionCode::NormalLumpSum {
        tax: crate::LumpSumTax::Form4972 { first_payment, .. },
    } = &mut r.code
    else {
        unreachable!()
    };
    *first_payment = 20251231;
    reject(r, "LUMP_SUM_FIRST_PAYMENT_NOT_IN_2026");
    let mut r = four_a(inherited_tax(TenYear));
    let crate::DistributionCode::DeathLumpSum {
        tax: crate::InheritedLumpSumTax::Form4972 { last_prior_payment, .. },
        ..
    } = &mut r.code
    else {
        unreachable!()
    };
    *last_prior_payment = Some(20260101);
    reject(r, "LUMP_SUM_PRIOR_PAYMENT_NOT_EARLIER");
    let mut r = four_a(inherited_tax(TenYear));
    let crate::DistributionCode::DeathLumpSum {
        tax: crate::InheritedLumpSumTax::Form4972 { born, .. },
        ..
    } = &mut r.code
    else {
        unreachable!()
    };
    *born = 20260201;
    reject(r, "LUMP_SUM_BORN_AFTER_DEATH");
    // Box 9a is accepted with every treatment and stored only on an elected record.
    let mut r = seven_a(crate::LumpSumTax::WithOtherIncome { what_happened: Disposition::Kept });
    r.more.box_9a_percent = Some(40.0);
    accept(r);
    let mut r = seven_a(lump_tax(TenYear, false));
    r.more.box_9a_percent = Some(40.0);
    let saved = accept(r);
    assert!(matches!(
        &saved.income[1].income,
        FederalIncome::Retirement { data }
            if matches!(&data.ordinary_distribution,
                RetirementTreatment::LumpSum { data } if data.percent == Some(4_000))
    ));
    // A Roth IRA row still needs the new boxes blank.
    let mut roth = roth_item_for_tests();
    roth.more.box_3 = 1;
    reject(roth, "R_ROTH_OTHER_BOXES");
}

#[test]
fn lump_sum_page_round_trips_through_the_interview() {
    use crate::LumpSumMethod::{Both, TenYear};
    let original = complete(FederalStatus::Single);
    let mut own = seven_a(lump_tax(Both, true));
    own.taxable = 1_500_000;
    own.more = RMore {
        box_3: 100_000,
        box_6: 300_000,
        box_8a: 200_000,
        box_8b_percent: Some(33.33),
        box_9a_percent: Some(40.0),
        ..RMore::default()
    };
    let mut inherited = four_a(inherited_tax(TenYear));
    inherited.more.box_8a = 50_000;
    let other_income = four_a(crate::InheritedLumpSumTax::WithOtherIncome);
    // Boxes 3, 6, 8a and 8b on a record that is not elected are accepted and not stored: the
    // page reopens them at zero (the box 2a precedent under the Simplified Method).
    let mut unelected =
        seven_a(crate::LumpSumTax::WithOtherIncome { what_happened: Disposition::Kept });
    unelected.more = RMore {
        box_3: 100_000,
        box_6: 300_000,
        box_8a: 200_000,
        box_8b_percent: Some(25.0),
        ..RMore::default()
    };
    // Each row carries the methods its codes allow, as the page builder writes them.
    let with_methods = |mut r: R1099| {
        r.allowed_methods = DistributionCodes::methods(r.box7, r.ira, r.trump);
        r
    };
    let (own, inherited, other_income, mut unelected) = (
        with_methods(own),
        with_methods(inherited),
        with_methods(other_income),
        with_methods(unelected),
    );
    let sent = vec![
        document(0, IncomeDocument::R1099(own)),
        document(0, IncomeDocument::R1099(inherited)),
        document(0, IncomeDocument::R1099(other_income)),
        document(0, IncomeDocument::R1099(unelected.clone())),
    ];
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = send(&mut session, save_documents(&original, sent.clone()));
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let mut session = restart(session.draft.clone());
    let documents = page_documents(&open_topic(&mut session, TaxSection::Retirement)).to_vec();
    let mut got: Vec<_> = documents.iter().map(|d| d.document.clone()).collect();
    let mut want: Vec<_> = sent.iter().map(|d| d.document.clone()).collect();
    let key = |d: &IncomeDocument| alloc::format!("{d:?}");
    // The unelected record reopens with its Other boxes at zero.
    unelected.more = RMore::default();
    want.retain(|d| !matches!(d, IncomeDocument::R1099(r) if r.more.box_8b_percent == Some(25.0)));
    want.push(IncomeDocument::R1099(unelected));
    got.sort_by_key(key);
    want.sort_by_key(key);
    assert_eq!(got, want);
}

#[test]
fn lump_sum_import_reads_boxes_3_6_8a_and_8b() {
    // AI import: the Other boxes are import targets and the instruction names them.
    let model = include_str!("../../federal/federal_model.rs");
    let more = &model[model.find("pub struct RMore {").unwrap()..];
    let more = &more[..more.find("\n}").unwrap()];
    for field in [
        "pub box_3: i64",
        "pub box_6: i64",
        "pub box_8a: i64",
        "pub box_8b_percent: Option<f64>",
        "pub box_9a_percent: Option<f64>",
    ] {
        let at = more.find(field).unwrap();
        let before = &more[..at];
        let attrs = &before[before.rfind("///").unwrap()..];
        assert!(attrs.contains("#[libertas_llm_doc_field]"), "{field}");
    }
    // Box 8b follows box 9a's attribute set (U8): a two-decimal percent.
    let percent = "#[libertas_number(min = 0.01, max = 99.99, step = 0.01)]\n    #[libertas_physical_unit(\"percent\")]\n    pub box_";
    assert!(more.contains(&alloc::format!("{percent}8b_percent: Option<f64>")));
    assert!(more.contains(&alloc::format!("{percent}9a_percent: Option<f64>")));
    assert!(model.contains(
        "return the box 3, box 6, box 8a and box 10 amounts, or zero when clearly blank; and box 8b's and box 9a's percentages as printed with up to two decimals, or null when blank"
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
    accepted(&send(
        &mut session,
        save_topic(
            &original,
            TaxSection::Retirement,
            vec![document(
                0,
                IncomeDocument::Qcd(QcdGift {
                    split_interest: false,
                    date: 20260301,
                    amount: 100_000,
                    inherited: None,
                }),
            )],
        ),
    ));
    let charities = &session.draft.adjustments.as_ref().unwrap().qualified_charity;
    // A gift is named by its date and amount; no label text is stored (F3,
    // 2026-10-09).
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
    let mut page = open_topic(&mut session, TaxSection::Retirement);
    let P::SaveIncomes { documents, .. } = &mut page else { unreachable!() };
    for item in &mut documents.documents {
        if [2, 3].contains(&item.id) {
            let IncomeDocument::R1099(source) = &mut item.document else { unreachable!() };
            let DistributionCode::RothIra {
                roth: RothIraCode::Early { distributed, .. },
                sequence,
                ..
            } = &mut source.code
            else {
                unreachable!()
            };
            *distributed = 20260615;
            *sequence = Some(if item.id == 2 { 2 } else { 1 });
        }
    }
    accepted(&send(&mut session, page));
    assert_eq!(return_line(&supported(&session.draft), "4b"), 10000);
    let saved = session.draft.clone();
    let mut session = restart(saved.clone());
    let mut page = open_topic(&mut session, TaxSection::Retirement);
    let P::SaveIncomes { documents, .. } = &mut page else { unreachable!() };
    for item in &mut documents.documents {
        if [2, 3].contains(&item.id) {
            let IncomeDocument::R1099(source) = &mut item.document else { unreachable!() };
            let DistributionCode::RothIra { sequence, .. } = &mut source.code else {
                unreachable!()
            };
            assert_eq!(*sequence, Some(if item.id == 2 { 2 } else { 1 }));
            if item.id == 3 {
                *sequence = Some(101);
            }
        }
    }
    rejected_with(send(&mut session, page), "R_ROTH_SEQUENCE_INVALID");
    assert_eq!(session.draft, saved);
}

#[test]
fn roth_earlier_funding_public_save_reopen_and_source_change_boundaries() {
    let mut s = restart(complete(FederalStatus::Single));
    let mut income_page = open_topic(&mut s, TaxSection::Retirement);
    let P::SaveIncomes { documents, .. } = &mut income_page else { unreachable!() };
    documents.documents = [(30000, 20260215), (60000, 20260615), (20000, 20260915)]
        .into_iter()
        .map(|(gross, distributed)| IncomeDocumentItem {
            investment_caps: None,
            refund_management: None,
            refund_management_needed: None,
            allowed_community: Vec::new(),
            id: 0,
            owner: Owner::Taxpayer,
            preview: None,
            community: CommunityShare::NotNeeded,
            document: IncomeDocument::R1099(r1099(
                gross,
                DistributionCode::RothIra {
                    sequence: None,
                    rolled_over: 0,
                    roth: RothIraCode::Early { distributed, exception: None, repaid: None },
                },
                false,
            )),
        })
        .collect();
    documents.documents.push(IncomeDocumentItem {
        investment_caps: None,
        refund_management: None,
        refund_management_needed: None,
        allowed_community: Vec::new(),
        id: 0,
        owner: Owner::Taxpayer,
        preview: None,
        community: CommunityShare::NotNeeded,
        document: IncomeDocument::RothHistory(RothIraHistory {
            first_year: 2010,
            amounts: Some(RothOrdering { regular: 80000, conversions: vec![] }),
        }),
    });
    accepted(&send(&mut s, income_page));
    let source = s
        .draft
        .income
        .iter()
        .find(|e| matches!(&e.income, FederalIncome::Roth { data } if data.distributed == 20260615))
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
    let mut retirement = open_topic(&mut incoming, TaxSection::Retirement);
    let P::SaveIncomes { documents, .. } = &mut retirement else { unreachable!() };
    for row in &mut documents.documents {
        if let IncomeDocument::R1099(data) = &mut row.document
            && data.gross == 20000
        {
            data.gross = 110000;
            data.taxable = 110000;
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
    let mut income_page = open_topic(&mut restored, TaxSection::Retirement);
    let P::SaveIncomes { documents, .. } = &mut income_page else { unreachable!() };
    let row = documents.documents.iter_mut().find(|e| e.id == source).unwrap();
    let IncomeDocument::R1099(data) = &mut row.document else { unreachable!() };
    data.gross += 100;
    data.taxable += 100;
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
    let mut retirement = open_topic(&mut s, TaxSection::Retirement);
    let P::SaveIncomes { documents, .. } = &mut retirement else { unreachable!() };
    documents.documents = [(30000, 20260215), (60000, 20260615), (20000, 20260915)]
        .into_iter()
        .map(|(gross, distributed)| IncomeDocumentItem {
            investment_caps: None,
            refund_management: None,
            refund_management_needed: None,
            allowed_community: Vec::new(),
            id: 0,
            owner: Owner::Taxpayer,
            preview: None,
            community: CommunityShare::NotNeeded,
            document: IncomeDocument::R1099(r1099(
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
            )),
        })
        .collect();
    documents.documents.push(IncomeDocumentItem {
        investment_caps: None,
        refund_management: None,
        refund_management_needed: None,
        allowed_community: Vec::new(),
        id: 0,
        owner: Owner::Taxpayer,
        preview: None,
        community: CommunityShare::NotNeeded,
        document: IncomeDocument::InheritedIra(InheritedIraAccount {
            decedent: "Synthetic source owner".into(),
            died: 20251201,
            kind: InheritedIraKindFacts::Roth {
                first_year: 2023,
                basis: Some(InheritedRothBasis { regular: 80000, converted: 0 }),
            },
        }),
    });
    accepted(&send(&mut s, retirement));
    let source = s
        .draft
        .income
        .iter()
        .find(|e| matches!(&e.income, FederalIncome::Roth { data } if data.distributed == 20260615))
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
        .find(|e| matches!(&e.income, FederalIncome::Roth { data } if data.distributed == 20260215))
        .unwrap()
        .id;
    let move_ordinary = |request: &mut P, date| {
        let P::SaveIncomes { documents, .. } = request else { unreachable!() };
        let row = documents.documents.iter_mut().find(|row| row.id == ordinary_id).unwrap();
        let IncomeDocument::R1099(data) = &mut row.document else { unreachable!() };
        let DistributionCode::RothIra {
            roth: RothIraCode::ExceptionApplies { distributed, .. },
            ..
        } = &mut data.code
        else {
            unreachable!()
        };
        *distributed = date;
    };
    // Source edits use the same raw consistency check as Health-account edits.
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
    rejected_with(
        current_source_refusal(
            "health-context-09",
            &mut contradicted,
            retirement,
            "HSA_FUNDING_ORDER_CONTRADICTED",
            Some("[documents/documents/0/document]"),
            SourceScope::Health,
        ),
        "HSA_FUNDING_ORDER_CONTRADICTED",
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
    let mut retirement = open_topic(&mut restored, TaxSection::Retirement);
    let P::SaveIncomes { documents, .. } = &mut retirement else { unreachable!() };
    let row = documents.documents.iter_mut().find(|e| e.id == source).unwrap();
    let IncomeDocument::R1099(data) = &mut row.document else { unreachable!() };
    data.gross += 100;
    data.taxable += 100;
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
