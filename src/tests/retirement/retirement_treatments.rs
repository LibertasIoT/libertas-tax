use crate::tests::support::assert::line_amount as cents;
use crate::tests::support::builders::survey::survey_uncatalogued_document as unanswered_document;
// Retirement treatments (unit B5, 2026-10-09): fixtures F1-F13 of the map, each a Pub. 575, 721 or
// 939 worked example or a boundary, saved through the 1099-R page and read back from the
// estimate. Amounts in the comments are dollars; the records carry cents. Unless stated: single
// filer born 1960-03-01 (age 66), one document, no other income, box 7b unchecked, every
// confirmation Yes.
use super::*;
use crate::tests::cross::interview::{
    accepted, document, open_documents, open_topic, page_documents as all_page_documents, r1099,
    save_documents, save_topic,
};
use crate::{
    AnnuityBalance, ChildAnnuity, ChildAnnuityPayment, LegacyAnnuity, PropertyRollover,
    RailroadRecipient, RailroadRecovery, RailroadRepayment, Rrb1099R, TaxableMethod as M,
};

fn page_documents(page: &P) -> Vec<IncomeDocumentItem> {
    all_page_documents(page)
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::Retirement { .. }))
        .collect()
}
fn observed(
    s: &mut Interview,
) -> (Vec<crate::IncomeRecordItem>, Vec<crate::RetirementDocumentRow>) {
    let P::SaveIncomes { documents: records, .. } = open_topic(s, TaxSection::Retirement) else {
        panic!("expected the retirement records")
    };
    let P::SaveDocuments { documents: sources, .. } = open_documents(s) else {
        panic!("expected the document center")
    };
    (records.documents, sources.common.retirement.documents)
}
fn assert_fixture(s: &mut Interview, fixture: &[IncomeDocumentItem]) {
    let (records, sources) = observed(s);
    // Each editor preserves its declared fixture order; identities never imply order.
    let want_records: Vec<_> = fixture
        .iter()
        .filter_map(|row| match &row.document {
            IncomeSource::Record { value } => {
                Some((row.owner.owned(), value.clone(), row.community, row.refund_management))
            }
            _ => None,
        })
        .collect();
    let want_sources: Vec<_> = fixture
        .iter()
        .filter_map(|row| {
            match expected_named_source(&s.draft, row.owner.owned(), row.document.clone()) {
                IncomeSource::Retirement { value } => {
                    Some((row.owner, value, row.community, row.refund_management))
                }
                _ => None,
            }
        })
        .collect();
    let shown_records: Vec<_> = records
        .into_iter()
        .map(|row| (row.owner, row.document, row.community, row.refund_management))
        .collect();
    let shown_sources: Vec<_> = sources
        .into_iter()
        .map(|row| (row.owner, row.document, row.community, row.refund_management))
        .collect();
    assert_eq!(shown_records, want_records);
    assert_eq!(shown_sources, want_sources);
}
fn filer(birth: u32) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = birth;
    d
}
// One 1099-R in dollars: box 1, box 2a, the printed box 7a and box 7b, the branch and the box 2b method.
fn form(
    printed: &str,
    ira: bool,
    gross: i64,
    taxable: i64,
    code: DistributionCode,
    method: M,
) -> R1099Answered {
    let mut r = printed_as(r1099(gross * 100, code, ira), printed);
    r.source_mut().form.taxable = Some(taxable * 100);
    r.source_mut().form.taxable_not_determined = !matches!(method, M::BoxTwoA);
    r.source_mut().taxable_method = Some(method);
    r
}
fn normal() -> DistributionCode {
    DistributionCode::Normal { what_happened: Disposition::Kept }
}
fn early() -> DistributionCode {
    DistributionCode::Early { what_happened: Disposition::Kept, exception: EarlyException::None }
}
// Saves payer rows through their source editor and returns the session.
fn save(d: &FederalDraft, rows: Vec<R1099Answered>) -> Interview {
    let mut s = Interview::new(TEST_USER, d.clone());
    let documents = rows
        .into_iter()
        .map(|r| {
            document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(r)),
                },
            )
        })
        .collect::<Vec<_>>();
    accepted(&{
        let request = save_documents(&mut s, documents.clone());
        send(&mut s, request)
    });
    crate::tests::support::client::income_pages::save_information_return_answers(
        &mut s, &documents,
    );
    s
}
// The key of the Problem a save of these rows is refused with.
fn refusal(d: &FederalDraft, rows: Vec<R1099Answered>) -> alloc::vec::Vec<u8> {
    let mut s = Interview::new(TEST_USER, d.clone());
    let documents = rows
        .into_iter()
        .map(|r| {
            document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(r)),
                },
            )
        })
        .collect::<Vec<_>>();
    let request = save_documents(&mut s, documents.clone());
    let before = s.draft.clone();
    let reply = send(&mut s, request);
    if let Reply::Response(P::Problem { message, .. }) = reply {
        assert_eq!(s.draft, before);
        return message;
    }
    let before = s.draft.clone();
    let request =
        crate::tests::support::client::income_pages::information_return_answers(&mut s, &documents);
    let Reply::Response(P::Problem { message, .. }) = send(&mut s, request) else {
        panic!("expected a refused topic save")
    };
    assert_eq!(s.draft, before);
    message
}
fn key(k: &str) -> alloc::vec::Vec<u8> {
    libertas::libertas_formatted_text(k, &[])
}
fn stored(s: &Interview) -> FederalRetirement {
    let FederalIncome::Retirement { data } = &s.draft.income[1].income else {
        panic!("a retirement record")
    };
    FederalRetirement::from(data)
}
// Saves, estimates, reopens the page and saves it unchanged: the records are the same.
fn round_trips(s: &Interview) {
    let mut again = restart(s.draft.clone());
    let request = open_documents(&mut again);
    accepted(&send(&mut again, request));
    let request = open_topic(&mut again, TaxSection::Retirement);
    if !all_page_documents(&request).is_empty() {
        accepted(&send(&mut again, request));
    }
    assert_eq!(again.draft, s.draft);
}

#[test]
fn validation_f08_cash_rollovers_are_bounded_by_the_distribution() {
    let d = filer(19600301);
    for branch in 0..4 {
        for amount in [9_999, 10_000, 10_001] {
            let disposition =
                Disposition::RolledOver { amount, into: crate::RolloverDestination::Ira };
            let code = match branch {
                0 => DistributionCode::Normal { what_happened: disposition },
                1 => DistributionCode::Early {
                    what_happened: disposition,
                    exception: EarlyException::None,
                },
                2 => DistributionCode::NormalLumpSum {
                    tax: LumpSumTax::WithOtherIncome { what_happened: disposition },
                },
                _ => DistributionCode::EarlySimple {
                    what_happened: SimpleDisposition::RolledOver { amount },
                },
            };
            let row = r1099(10_000, code, branch == 3);
            if amount > row.gross {
                assert_eq!(refusal(&d, vec![row]), key("R_ROLLOVER_ABOVE_GROSS"));
            } else {
                round_trips(&save(&d, vec![row]));
            }
        }
    }
}

#[test]
fn validation_f08_conversions_are_bounded_by_the_distribution() {
    let d = filer(19600301);
    for branch in 0..3 {
        for amount in [9_999, 10_000, 10_001] {
            let disposition = Disposition::Converted { amount };
            let code = match branch {
                0 => DistributionCode::Normal { what_happened: disposition },
                1 => DistributionCode::Early {
                    what_happened: disposition,
                    exception: EarlyException::None,
                },
                _ => DistributionCode::NormalLumpSum {
                    tax: LumpSumTax::WithOtherIncome { what_happened: disposition },
                },
            };
            let row = r1099(10_000, code, false);
            if amount > row.gross {
                assert_eq!(refusal(&d, vec![row]), key("R_CONVERSION_ABOVE_GROSS"));
            } else {
                round_trips(&save(&d, vec![row]));
            }
        }
    }
}

#[test]
fn validation_f09_emergency_repayments_fit_the_earlier_withdrawal() {
    let d = filer(19600301);
    for branch in 0..4 {
        for (amount, repaid, contributed) in
            [(0, 0, 0), (10_000, 10_000, 50_000), (10_000, 10_001, 0)]
        {
            let emergency = RetirementException::Emergency {
                balance: 1_000_000,
                history: EmergencyHistory { amount, repaid, contributed },
                eligible: true,
            };
            let exception = EarlyException::Applies {
                distributed: 20260601,
                facts: if branch == 3 {
                    RetirementException::Medical { expenses: 0 }
                } else {
                    emergency.clone()
                },
                second: (branch == 3).then_some(emergency.clone()),
                repaid: None,
            };
            let code = match branch {
                0 | 3 => DistributionCode::Early { what_happened: Disposition::Kept, exception },
                1 => DistributionCode::ProhibitedTransaction { exception },
                _ => DistributionCode::RothIra {
                    roth: RothIraCode::Early {
                        distributed: 20260601,
                        exception: Some(emergency),
                        repaid: None,
                    },
                    rolled_over: 0,
                    sequence: None,
                },
            };
            let row = r1099(10_000, code, true);
            if repaid > amount {
                assert_eq!(refusal(&d, vec![row]), key("EMERGENCY_REPAID_ABOVE_DISTRIBUTION"));
            } else {
                round_trips(&save(&d, vec![row]));
            }
        }
    }
}
fn general_annuity() -> GeneralAnnuity {
    GeneralAnnuity {
        frequency: AnnuityFrequency::Monthly,
        earlier_payments: 0,
        qualified_age: None,
        started: 20261001,
        cost: 2_205_000,
        increase: 0,
        periodic: 12_500,
        duration: AnnuityDuration::SingleLife { age: 61, guaranteed: 0, delay: 1 },
        received: 3,
        partial: 0,
        recovered: 0,
        eligible: true,
    }
}

// F1: Pub. 939's general rule example. Monthly from 2026-10-01, cost 22,050, 125 a month,
// single life aged 61 (multiple 23.3), three payments received.
//   expected return 125 x 12 x 23.3 = 34,950; ratio 22,050 / 34,950 = 63.1%
//   excluded 375 x 0.631 = 236.625 -> 236.63 (cents kept across installments)
//   taxable 375 - 236.63 = 138.37; the insurer's statement printed 7D: a nonqualified annuity,
//   so the 138.37 is net investment income (Form 8960) too.
#[test]
fn f1_general_rule_fixed_payments_from_a_7d_statement() {
    let d = filer(19600301);
    let row = form("7D", false, 375, 375, normal(), M::GeneralRule { data: general_annuity() });
    let s = save(&d, vec![row]);
    assert_eq!(stored(&s).kind, RetirementKind::Nonqualified);
    assert_eq!(stored(&s).box7, box7("7D"));
    let r = supported(&s.draft);
    assert_eq!(cents(&r, "annuity_2_expected_return"), 3_495_000);
    assert_eq!(cents(&r, "annuity_2_basis_recovery"), 23_663);
    assert_eq!(return_line(&r, "5a"), 37_500);
    assert_eq!(return_line(&r, "5b"), 13_800);
    assert_eq!(value(&r, "nonqualified_annuity_investment_income"), 138);
    round_trips(&s);
}

// F1b: the general rule for a qualified plan annuity that started before November 19, 1996, with
// the cost recovered: Pub. 939's survivor example (22.5%). Cost 7,938, 147 a month, single life
// aged 65 (multiple 20.0): expected return 147 x 12 x 20.0 = 35,280, ratio 22.5%. Box 1 =
// 147 x 12 + 228 (a cost-of-living increase) = 1,992. The exclusion 22.5% x 1,764 = 396.90 is
// limited to cost less recovered = 7,938 - 7,938 = 0 (a start after 1986), so 5b = 1,992.
#[test]
fn f1b_general_rule_stops_when_the_cost_is_recovered() {
    let d = filer(19600301);
    let annuity = GeneralAnnuity {
        started: 19960101,
        cost: 793_800,
        increase: 22_800,
        periodic: 14_700,
        duration: AnnuityDuration::SingleLife { age: 65, guaranteed: 0, delay: 1 },
        received: 12,
        recovered: 793_800,
        ..general_annuity()
    };
    let s =
        save(&d, vec![form("7", false, 1_992, 1_992, normal(), M::GeneralRule { data: annuity })]);
    assert_eq!(stored(&s).kind, RetirementKind::Pension);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5a"), 199_200);
    assert_eq!(return_line(&r, "5b"), 199_200);
    assert_eq!(cents(&r, "annuity_2_basis_recovery"), 0);
}

// F2: Pub. 939's variable annuity, new. Code 7D, box 1 920; cost 12,000, annual, one payment,
// age 65 (Table V 20.0, annual adjustment at 6 months 0): 12,000 / 20 = 600 excluded, 5b 320.
#[test]
fn f2_variable_annuity_new() {
    let d = filer(19600301);
    let s = save(
        &d,
        vec![form(
            "7D",
            false,
            920,
            920,
            normal(),
            M::VariablePayments { data: variable_annuity() },
        )],
    );
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5b"), 32_000);
    assert_eq!(cents(&r, "variable_annuity_2_annual_exclusion"), 60_000);
    round_trips(&s);
}

// F2b: the refigure of Pub. 939's third year. Started 2024-01-01, annual 600, 1,100 recovered in
// the earlier years, box 1 1,200. Shortfall 600 - 500 = 100; 100 / 18.4 = 5.43; new annual
// 605.43; 5b = 1,200 - 605.43 = 594.57.
#[test]
fn f2b_variable_annuity_refigures_a_shortfall() {
    let d = filer(19600301);
    let mut annuity = variable_annuity();
    annuity.started = 20240101;
    annuity.history = VariableHistory::Prior {
        annual: 60_000,
        recovered: 110_000,
        refigure: Some(VariableRefigure {
            years: 1,
            paid: 50_000,
            remaining: VariablePayments::Life { age: 67, delay: 6 },
        }),
    };
    let s = save(
        &d,
        vec![form("7D", false, 1_200, 1_200, normal(), M::VariablePayments { data: annuity })],
    );
    let r = supported(&s.draft);
    assert_eq!(cents(&r, "variable_annuity_2_annual_exclusion"), 60_543);
    assert_eq!(return_line(&r, "5b"), 59_500);
    assert_eq!(cents(&r, "variable_annuity_2_election"), 1_200_000);
}

// F3: a withdrawal from a contract with investment from before August 14, 1982 (code 1D, aged 54).
// Box 1 18,000; issued 1980-06-01; before = value 16,000, investment 10,000; after = value 7,000,
// investment 5,000. Order: 10,000 pre-1982 investment (free), 6,000 pre-1982 earnings, 2,000
// post-1982 earnings, 0 post-1982 investment: 5b = 8,000; the 10% tax leaves out the 6,000
// pre-1982 earnings (section 72(q)(2)(F)): 10% x 2,000 = 200.
#[test]
fn f3_pre_1982_withdrawal_orders_investment_first() {
    let d = filer(19720501);
    let legacy = LegacyAnnuity {
        issued: 19800601,
        before: AnnuityBalance { value: 1_600_000, investment: 1_000_000 },
        after: AnnuityBalance { value: 700_000, investment: 500_000 },
        confirmed: true,
    };
    let s = save(
        &d,
        vec![form("1D", false, 18_000, 18_000, early(), M::Pre1982Withdrawal { data: legacy })],
    );
    assert_eq!(stored(&s).kind, RetirementKind::Nonqualified);
    assert_eq!(stored(&s).ordinary_distribution, Some(RetirementTreatment::Early));
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5b"), 800_000);
    assert_eq!(value(&r, "early_distribution_tax"), 200);
}

// F4: the whole contract ended. (a) code 7D, box 1 30,000, investment 22,000 -> 8,000. (b) a loss:
// box 1 18,000 against 22,000 -> 0, no deduction. (c) a gift of the contract (the insurer
// reported the cash surrender value 50,000, box 2b checked), investment 35,000 -> 15,000.
// (d) the confirmation answered No takes the whole 30,000 with a note.
#[test]
fn f4_whole_contract_ended_surrender_loss_gift_and_unconfirmed() {
    let d = filer(19600301);
    let whole = |gross: i64, investment: i64, eligible: bool| {
        form(
            "7D",
            false,
            gross,
            gross,
            normal(),
            M::WholeContract {
                data: crate::ContractDischarge { investment: investment * 100, eligible },
            },
        )
    };
    let s = save(&d, vec![whole(30_000, 22_000, true)]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5a"), 3_000_000);
    assert_eq!(return_line(&r, "5b"), 800_000);
    round_trips(&s);
    let r = supported(&save(&d, vec![whole(18_000, 22_000, true)]).draft);
    assert_eq!(return_line(&r, "5b"), 0);
    let r = supported(&save(&d, vec![whole(50_000, 35_000, true)]).draft);
    assert_eq!(return_line(&r, "5b"), 1_500_000);
    let s = save(&d, vec![whole(30_000, 22_000, false)]);
    let before = stored(&s);
    assert!(before.form_boxes.taxable_not_determined);
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_RETIREMENT_BASIS_LEFT_OUT"));
    assert_eq!(return_line(&r, "5b"), 3_000_000);
    assert_eq!(stored(&s), before);
    round_trips(&s);
}

// F5: a withdrawal from an employer plan before annuity payments (Pub. 575, Ann Brown and Ryan).
// (a) code 7, box 1 50,000, account 100,000, investment 10,000: tax-free 50,000 x 10,000 /
// 100,000 = 5,000, 5b 45,000. (b) Ryan, a separate contract: box 1 5,000, value 12,500,
// investment 10,000: tax-free 4,000, 5b 1,000. (c) as (a) with code 1 (aged 50, no exception):
// 10% x 45,000 = 4,500.
#[test]
fn f5_plan_withdrawal_takes_the_investments_share_of_the_account() {
    let plan = |value: i64, investment: i64| M::PlanWithdrawal {
        data: crate::PlanDistribution {
            value: value * 100,
            investment: investment * 100,
            eligible: true,
        },
    };
    let d = filer(19600301);
    let s = save(&d, vec![form("7", false, 50_000, 50_000, normal(), plan(100_000, 10_000))]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5b"), 4_500_000);
    assert_eq!(stored(&s).kind, RetirementKind::Pension);
    round_trips(&s);
    let r = supported(
        &save(&d, vec![form("7", false, 5_000, 5_000, normal(), plan(12_500, 10_000))]).draft,
    );
    assert_eq!(return_line(&r, "5b"), 100_000);
    let d = filer(19760301);
    let r = supported(
        &save(&d, vec![form("1", false, 50_000, 50_000, early(), plan(100_000, 10_000))]).draft,
    );
    assert_eq!(return_line(&r, "5b"), 4_500_000);
    assert_eq!(value(&r, "early_distribution_tax"), 4_500);
}

// F6: a withdrawal from a commercial annuity before payments (Pub. 575 example). (a) code 7D,
// box 1 7,000, value 16,000, investment 10,000: min(7,000, 6,000) = 6,000 taxable. (b) a related
// contract {value 9,000, investment 8,000}: 25,000 - 18,000 = 7,000 -> all 7,000. (c) as (a) with
// code 1D (aged 54): 10% x 6,000 = 600.
#[test]
fn f6_annuity_withdrawal_earnings_first_with_a_related_contract() {
    let withdrawal = |related: Vec<AnnuityBalance>| M::AnnuityWithdrawal {
        data: crate::AnnuityDistribution {
            loan_pledge: false,
            value: 1_600_000,
            investment: 1_000_000,
            related,
            eligible: true,
        },
    };
    let d = filer(19600301);
    let s = save(&d, vec![form("7D", false, 7_000, 7_000, normal(), withdrawal(vec![]))]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5b"), 600_000);
    round_trips(&s);
    let related = vec![AnnuityBalance { value: 900_000, investment: 800_000 }];
    let r = supported(
        &save(&d, vec![form("7D", false, 7_000, 7_000, normal(), withdrawal(related))]).draft,
    );
    assert_eq!(return_line(&r, "5b"), 700_000);
    let d = filer(19720501);
    let r = supported(
        &save(&d, vec![form("1D", false, 7_000, 7_000, early(), withdrawal(vec![]))]).draft,
    );
    assert_eq!(return_line(&r, "5b"), 600_000);
    assert_eq!(value(&r, "early_distribution_tax"), 600);
}

// F7: a CSRS lump sum after a death (Pub. 721 Worksheet D). Code 4, died 2024-12-15, box 1
// 38,400, cost 45,000, 1,000 recovered earlier: 38,400 + 1,000 - 45,000 < 0 -> 5b 0. (b) the
// employee died before retiring: box 1 21,500, cost 20,000 -> the interest, 1,500.
#[test]
fn f7_civil_service_lump_sum_after_a_death() {
    let d = filer(19600301);
    let lump = |cost: i64, recovered: i64| M::CivilServiceLumpSum {
        data: crate::SurvivorLumpSum {
            cost: cost * 100,
            recovered: recovered * 100,
            eligible: true,
        },
    };
    let death = DistributionCode::Death { died: 20241215 };
    let s = save(&d, vec![form("4", false, 38_400, 38_400, death.clone(), lump(45_000, 1_000))]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5a"), 3_840_000);
    assert_eq!(return_line(&r, "5b"), 0);
    round_trips(&s);
    let r =
        supported(&save(&d, vec![form("4", false, 21_500, 21_500, death, lump(20_000, 0))]).draft);
    assert_eq!(return_line(&r, "5b"), 150_000);
}

// F8: a CSRS child survivor annuity (Pub. 721, surviving child only). The filer is the child, born
// 2011-01-15; code 4, box 1 6,000. Started 2026-01-01, term (2033 x 12 + 1) - (2026 x 12 + 1) = 84
// months, cost 36,000: 36,000 / 84 = 428.57 a month, 12 x 428.57 = 5,142.84 excluded, 5b 857.16.
// A child born a month later fails the term check against People.
#[test]
fn f8_child_survivor_annuity_recovers_cost_over_the_months_to_age_22() {
    let annuity = ChildAnnuity {
        ended: None,
        started: 20260101,
        term: 84,
        cost: 3_600_000,
        recovered: 0,
        deferred: 0,
        payments: (0..12).map(|_| ChildAnnuityPayment { amount: 50_000 }).collect(),
        confirmed: true,
    };
    let death = DistributionCode::Death { died: 20251220 };
    let d = filer(20110115);
    let s = save(
        &d,
        vec![form(
            "4",
            false,
            6_000,
            6_000,
            death.clone(),
            M::ChildAnnuity { data: annuity.clone() },
        )],
    );
    let r = estimate(&s.draft);
    assert_eq!(cents(&r, "child_annuity_2_income_exclusion"), 514_284);
    round_trips(&s);
    let d = filer(20110215);
    let s =
        save(&d, vec![form("4", false, 6_000, 6_000, death, M::ChildAnnuity { data: annuity })]);
    assert!(
        estimate(&s.draft).issues.iter().any(|i| i.display.key.contains("SINGLE_CHILD_CSRS_FERS")),
        "{:?}",
        estimate(&s.draft).issues
    );
}

fn simplified_cost(cost: i64) -> PensionBasis {
    PensionBasis {
        earlier_months: 0,
        changes: vec![],
        share: None,
        started: 20260101,
        cost: Some(cost * 100),
        months: 12,
        history: PensionHistory::New { duration: PensionDuration::Single { age: 65 } },
        eligible: true,
    }
}
// A Form 1040 line, zero when the return shows none.
fn line_or_zero(r: &FederalResult, reference: &str) -> i64 {
    let line = FormLine::new(TaxForm::Form1040, reference);
    ReturnRows::rows(r)
        .iter()
        .find(|row| row.form_line.as_ref() == Some(&line))
        .map_or(0, |row| row.amount)
}

// F10: distributed property sold and the proceeds rolled over (Pub. 575, Paul, Examples 3 and 4).
// Code 7, box 1 = box 2a 50,000 (nonemployer stock distributed 2026-09-04). (a) sold 2026-09-24
// for 60,000, 45,000 deposited into an IRA: kept 15,000; ordinary 50,000 x 15,000 / 60,000 =
// 12,500 (line 5b), short-term gain 15,000 - 12,500 = 2,500. (b) sold for 40,000, 25,000
// deposited: ordinary 50,000 x 15,000 / 40,000 = 18,750, short-term loss 15,000 - 18,750 = -3,750.
// (c) box 7b checked: refused.
#[test]
fn f10_property_sold_and_rolled_over() {
    let sold = |proceeds: i64, deposited: i64, ira: bool| {
        form(
            "7",
            ira,
            50_000,
            50_000,
            DistributionCode::Normal {
                what_happened: Disposition::SoldAndRolledOver {
                    data: PropertyRollover {
                        distributed: 20260904,
                        sold: 20260924,
                        proceeds: proceeds * 100,
                        confirmed: true,
                    },
                    amount: deposited * 100,
                    into: crate::RolloverDestination::Ira,
                },
            },
            M::BoxTwoA,
        )
    };
    let d = filer(19600301);
    let s = save(&d, vec![sold(60_000, 45_000, false)]);
    assert!(matches!(stored(&s).taxable, Some(RetirementTaxable::PropertyProceeds { .. })));
    assert_eq!(
        stored(&s).ordinary_distribution,
        Some(RetirementTreatment::Rollover {
            destination: crate::RolloverDestination::Ira,
            amount: 4_500_000,
            early: false
        })
    );
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5b"), 1_250_000);
    assert_eq!(cents(&r, "property_rollover_2_retained_capital_gain"), 250_000);
    round_trips(&s);
    // The sale may appreciate: proceeds, rather than box 1, bound this rollover.
    round_trips(&save(&d, vec![sold(60_000, 60_000, false)]));
    let r = supported(&save(&d, vec![sold(40_000, 25_000, false)]).draft);
    assert_eq!(return_line(&r, "5b"), 1_875_000);
    assert_eq!(cents(&r, "property_rollover_2_retained_capital_gain"), -375_000);
    assert_eq!(
        refusal(&d, vec![sold(60_000, 45_000, true)]),
        key("PROPERTY_ROLLOVER_NOT_FROM_A_PLAN")
    );
    assert_eq!(
        refusal(&d, vec![sold(60_000, 61_000, false)]),
        key("PROPERTY_ROLLOVER_ABOVE_PROCEEDS")
    );
}

// F11: a section 1035 exchange (code 6). Box 1 80,000, box 2a 0 -> lines 5a and 5b 0 (Pub. 575:
// "You need not report this on your tax return"); a box 2a above zero is refused.
#[test]
fn f11_code_6_exchange_shows_nothing_on_the_return() {
    let d = filer(19600301);
    let before = supported(&d);
    let s = save(&d, vec![form("6", false, 80_000, 0, DistributionCode::Exchange, M::BoxTwoA)]);
    assert_eq!(stored(&s).ordinary_distribution, Some(RetirementTreatment::Ordinary));
    assert_eq!(
        (stored(&s).taxable.clone(), stored(&s).box_2a),
        (Some(RetirementTaxable::Payer), 0)
    );
    let r = supported(&s.draft);
    assert_eq!(line_or_zero(&r, "5a"), 0);
    assert_eq!(line_or_zero(&r, "5b"), 0);
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    round_trips(&s);
    assert_eq!(
        refusal(&d, vec![form("6", false, 80_000, 1_000, DistributionCode::Exchange, M::BoxTwoA)]),
        key("R_BOX7_EXCHANGE_TAXABLE")
    );
}

// F12: employer stock with net unrealized appreciation, through the payer's box 2a. Code 7, box 1
// 100,000 (cash 40,000 + shares 60,000), box 6 45,000, box 2a 55,000 -> line 5a 100,000, line 5b
// 55,000; the 45,000 stays untaxed until the shares are sold.
#[test]
fn f12_nua_through_the_payers_box_2a() {
    let d = filer(19600301);
    let mut row = form("7", false, 100_000, 55_000, normal(), M::BoxTwoA);
    row.source_mut().form.box_6 = Some(4_500_000);
    let s = save(&d, vec![row]);
    assert_eq!(
        (stored(&s).taxable.clone(), stored(&s).box_2a),
        (Some(RetirementTaxable::Payer), 5_500_000)
    );
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5a"), 10_000_000);
    assert_eq!(return_line(&r, "5b"), 5_500_000);
}

// Instructions for Form 1099-R (2026), boxes 2a and 2b: use a settled payer
// amount rather than a second annuity worksheet; corrections rederive the choice.
#[test]
fn settled_box_2a_replaces_the_method_after_a_paper_correction() {
    let d = filer(19600301);
    let row = form(
        "7",
        false,
        100_000,
        55_000,
        normal(),
        M::Simplified { data: simplified_cost(20_000) },
    );
    let mut s = save(&d, vec![row]);
    let P::SaveIncomes { followups, .. } = open_topic(&mut s, TaxSection::Retirement) else {
        unreachable!()
    };
    assert!(
        followups
            .iter()
            .any(|row| matches!(row.answer, crate::DocumentAnswer::RetirementMethod { .. }))
    );
    // Pub. 575 Table 1: 260 payments at age 65; $20,000 / 260 =
    // $76.92 monthly exclusion, $923.04 for twelve months, taxable $99,077.
    assert_eq!(return_line(&supported(&s.draft), "5b"), 9_907_700);
    let mut documents = all_page_documents(&open_documents(&mut s));
    let paper = documents
        .iter_mut()
        .find_map(|row| match &mut row.document {
            IncomeSource::Retirement { value: RetirementDocument::R1099(paper) } => Some(paper),
            _ => None,
        })
        .unwrap();
    paper.taxable_not_determined = false;
    let request = save_documents(&mut s, documents);
    accepted(&send(&mut s, request));
    let P::SaveIncomes { followups, .. } = open_topic(&mut s, TaxSection::Retirement) else {
        unreachable!()
    };
    assert!(
        !followups
            .iter()
            .any(|row| matches!(row.answer, crate::DocumentAnswer::RetirementMethod { .. }))
    );
    assert_eq!(stored(&s).taxable, Some(RetirementTaxable::Payer));
    assert_eq!(return_line(&supported(&s.draft), "5b"), 5_500_000);
    round_trips(&s);
}

#[test]
fn settled_box_2a_only_narrows_non_ira_reported_amounts() {
    let mut row = form("7", false, 100_000, 55_000, normal(), M::BoxTwoA);
    assert_eq!(DistributionCodes::methods(&row), vec![0]);
    row.source_mut().taxable_method = Some(M::Simplified { data: simplified_cost(20_000) });
    assert_eq!(FederalRetirement::from(&row.into_source()).taxable, Some(RetirementTaxable::Payer));
    let mut row = form("7", false, 100_000, 55_000, normal(), M::BoxTwoA);
    row.source_mut().form.taxable = Some(0);
    assert_eq!(DistributionCodes::methods(&row), vec![0]);
    row.source_mut().form.taxable = None;
    assert_eq!(DistributionCodes::methods(&row), vec![0, 1, 2, 4, 5]);
    row.source_mut().form.taxable = Some(5_500_000);
    row.source_mut().form.taxable_not_determined = true;
    assert_eq!(DistributionCodes::methods(&row), vec![0, 1, 2, 4, 5]);
    row.source_mut().form.ira = true;
    assert_eq!(DistributionCodes::methods(&row), vec![0]);
    assert_eq!(row.source_mut().taxable_method, Some(M::BoxTwoA));
    // A legacy unanswered method cannot leave an invisible required question.
    row.source_mut().taxable_method = None;
    assert_eq!(row.effective_taxable_method(), Some(&M::BoxTwoA));
    row.source_mut().ira_basis = true;
    assert_eq!(FederalRetirement::from(&*row).taxable, Some(RetirementTaxable::IraBasis));
    let row = R1099Answered::new(row.into_source(), Vec::new());
    assert_eq!(row.taxable_method, Some(M::BoxTwoA));
}

// F13: code D printed with a numeric code leads to that code's branch and the kind comes from box
// 7a. 7D with box 2a 4,000 of box 1 12,000, method "Box 2a shows it": a nonqualified annuity, 5b
// 4,000 and net investment income 4,000 (today refused as "codes 7 and D printed together").
// 7D rolled over: refused.
#[test]
fn f13_code_d_with_a_numeric_code_routes_to_the_numeric_branch() {
    let d = filer(19600301);
    let s = save(&d, vec![form("7D", false, 12_000, 4_000, normal(), M::BoxTwoA)]);
    assert_eq!(stored(&s).kind, RetirementKind::Nonqualified);
    assert_eq!(stored(&s).box7, box7("7D"));
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5a"), 1_200_000);
    assert_eq!(return_line(&r, "5b"), 400_000);
    assert_eq!(value(&r, "nonqualified_annuity_investment_income"), 4_000);
    round_trips(&s);
    let rolled = DistributionCode::Normal {
        what_happened: Disposition::RolledOver {
            amount: 1_200_000,
            into: crate::RolloverDestination::Ira,
        },
    };
    assert_eq!(
        refusal(&d, vec![form("7D", false, 12_000, 4_000, rolled, M::BoxTwoA)]),
        key("R_NONQUALIFIED_KEPT")
    );
    // D with box 7b checked is not a nonqualified annuity.
    assert_eq!(
        refusal(&d, vec![form("7D", true, 12_000, 4_000, normal(), M::BoxTwoA)]),
        key("R_BOX7B_NOT_IRA")
    );
}

// 7D, 1D, 2D, 3D and 4D statements compute as the numeric code does, taxed as a nonqualified
// annuity; the branch reopens as printed.
#[test]
fn d_with_1_2_3_4_or_7_computes_and_reopens() {
    let d = filer(19720501);
    let disability = DistributionCode::Disability { began: 20250101, wages: DisabilityWages::None };
    let death = DistributionCode::Death { died: 20260301 };
    for (printed, code) in
        [("7D", normal()), ("1D", early()), ("2D", early()), ("3D", disability), ("4D", death)]
    {
        let s = save(&d, vec![form(printed, false, 12_000, 4_000, code, M::BoxTwoA)]);
        assert_eq!(stored(&s).kind, RetirementKind::Nonqualified, "{printed}");
        let r = supported(&s.draft);
        assert_eq!(return_line(&r, "5b"), 400_000, "{printed}");
        assert_eq!(value(&r, "nonqualified_annuity_investment_income"), 4_000, "{printed}");
        round_trips(&s);
    }
    // Code 1D without an exception adds the 10% tax on the taxable 4,000.
    let r = supported(&save(&d, vec![form("1D", false, 12_000, 4_000, early(), M::BoxTwoA)]).draft);
    assert_eq!(value(&r, "early_distribution_tax"), 400);
}

// Actual topic rows expose the methods their printed codes allow; a method outside the set is refused
// on the server (R_BOX2B_METHOD).
#[test]
fn the_box_2b_methods_follow_the_box_7a_codes() {
    let s = Interview::new(TEST_USER, filer(19600301));
    let pension = vec![0, 1, 2, 4, 5];
    let survivor = vec![0, 1, 2, 4, 5, 8, 9];
    let annuity = vec![0, 2, 3, 4, 6, 7];
    for (printed, ira, trump, methods) in [
        ("7", false, false, pension.clone()),
        ("1", false, false, pension.clone()),
        ("3", false, false, pension.clone()),
        ("7A", false, false, pension),
        ("4", false, false, survivor.clone()),
        ("4A", false, false, survivor),
        ("7D", false, false, annuity.clone()),
        ("1D", false, false, annuity.clone()),
        ("4D", false, false, annuity.clone()),
        ("D", false, false, annuity),
        ("7", true, false, vec![0]),
        ("4", true, false, vec![0]),
        ("G", false, false, vec![0]),
        ("6", false, false, vec![0]),
        ("Q", false, false, vec![0]),
        ("4", false, true, vec![0]),
    ] {
        let (_, allowed_methods, _) = retirement_question_catalog(&s, box7(printed), ira, trump, 0);
        assert_eq!(allowed_methods, methods, "{printed} {ira} {trump}");
    }
    // An IRA takes only box 2a; a nonqualified annuity has no Simplified Method.
    let d = filer(19600301);
    let simplified = M::Simplified { data: simplified_cost(20_000) };
    // The IRA's sole method has no question to submit; its native validator must still
    // refuse an incompatible supplied method rather than silently normalize it.
    assert_eq!(
        DistributionCodes::treatment_rule(&form(
            "7",
            true,
            1_000,
            1_000,
            normal(),
            simplified.clone()
        )),
        Some("R_BOX2B_METHOD")
    );
    assert_eq!(
        refusal(&d, vec![form("7D", false, 1_000, 1_000, normal(), simplified)]),
        key("R_BOX2B_METHOD")
    );
}

fn rrb(box_4: i64, box_5: i64, box_6: i64, recipient: RailroadRecipient) -> RailroadSource {
    RailroadSource {
        form: Rrb1099R {
            issuer: Default::default(),
            claim_number_and_payee_code: alloc::string::String::new(),
            recipient: Default::default(),
            medicare_premiums: Some(0),
            box_3: None,
            box_4: Some(box_4 * 100),
            box_5: Some(box_5 * 100),
            box_6: Some(box_6 * 100),
            gross: (box_4 + box_5 + box_6) * 100,
            box_8: Some(0),
            withholding: Some(200_000),
        },
        recipient_kind: Some(recipient),
        repayments: vec![],
    }
}
fn save_rrb(d: &FederalDraft, form: RailroadSource) -> Interview {
    let mut s = Interview::new(TEST_USER, d.clone());
    accepted(&{
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut s,
            vec![document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::Rrb1099R(railroad_document(form.clone())),
                },
            )],
        );
        send(&mut s, request)
    });
    crate::tests::support::client::income_pages::save_information_return_answers(
        &mut s,
        &[document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::Rrb1099R(railroad_document(form)),
            },
        )],
    );
    s
}
fn simplified_recovery() -> RailroadRecovery {
    RailroadRecovery::SimplifiedMethod { data: simplified_cost(20_000) }
}

// F9: the RRB-1099-R. Box 4 24,000, box 5 1,200, box 6 2,400, box 7 27,600, box 9 2,000; the
// employee's own annuity with the Simplified Method: started 2026-01-01, aged 65 (Table 1 after
// November 18, 1996, 61-65: 260), cost (box 3) 20,000, 12 months. Monthly exclusion 20,000 / 260 =
// 76.92; 12 x 76.92 = 923.04, no more than box 4. Line 5a 27,600, line 5b 27,600 - 923.04 =
// 26,676.96; withholding 2,000. (b) a spouse annuity (owner 62): 27,600 taxable. (c) a survivor
// (died 2025-11-02) stores the death exception and the same arithmetic. (d) box 8 3,500 by year.
#[test]
fn f9_rrb_1099_r_own_spouse_and_survivor_annuities() {
    let d = filer(19600301);
    let own =
        rrb(24_000, 1_200, 2_400, RailroadRecipient::Employee { recovery: simplified_recovery() });
    let own_form = own.clone();
    let s = save_rrb(&d, own);
    let FederalIncome::Retirement { data } = &s.draft.income[1].income else {
        panic!("a retirement record")
    };
    assert_eq!(crate::FederalRetirement::from(data).kind, RetirementKind::Pension);
    assert_eq!(crate::FederalRetirement::from(data).box7, DistributionCodes::CODE_7.0);
    assert_eq!(
        crate::FederalRetirement::from(data).ordinary_distribution,
        Some(RetirementTreatment::Ordinary)
    );
    assert!(matches!(
        &crate::FederalRetirement::from(data).taxable,
        Some(RetirementTaxable::Railroad { contributory: 2_400_000, basis: Some(_), .. })
    ));
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5a"), 2_760_000);
    assert_eq!(return_line(&r, "5b"), 2_667_700);
    assert_eq!(cents(&r, "pension_2_income_exclusion"), 92_304);
    assert_eq!(return_line(&r, "25b"), 200_000);
    round_trips(&s);
    // The page reopens as the RRB-1099-R, exactly as entered.
    let reopened = |s: &Interview| {
        let mut again = restart(s.draft.clone());
        let page = open_documents(&mut again);
        page_documents(&page)[0].document.clone()
    };
    assert_eq!(
        reopened(&s),
        expected_named_source(
            &s.draft,
            Owner::Taxpayer,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::Rrb1099R(railroad_document(own_form))
            }
        )
    );
    // (b) A spouse annuity recovers nothing.
    let d62 = filer(19640301);
    let spouse = rrb(24_000, 1_200, 2_400, RailroadRecipient::Spouse);
    let spouse_form = spouse.clone();
    let s = save_rrb(&d62, spouse);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5b"), 2_760_000);
    // It reopens as a spouse annuity with boxes 5 and 6 as entered (box 6 nonzero).
    assert_eq!(
        reopened(&s),
        expected_named_source(
            &s.draft,
            Owner::Taxpayer,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::Rrb1099R(railroad_document(spouse_form))
            }
        )
    );
    // (c) A survivor annuity stores the death exception.
    let survivor = rrb(
        24_000,
        1_200,
        2_400,
        RailroadRecipient::Survivor {
            died: 20251102,
            distributed: 20261231,
            recovery: simplified_recovery(),
        },
    );
    let survivor_form = survivor.clone();
    let s = save_rrb(&d, survivor);
    assert_eq!(
        reopened(&s),
        expected_named_source(
            &s.draft,
            Owner::Taxpayer,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::Rrb1099R(railroad_document(survivor_form))
            }
        )
    );
    let FederalIncome::Retirement { data } = &s.draft.income[1].income else {
        panic!("a retirement record")
    };
    assert_eq!(
        crate::FederalRetirement::from(data).ordinary_distribution,
        Some(RetirementTreatment::Exception {
            second: None,
            distributed: 20261231,
            facts: RetirementException::Death { died: 20251102 },
            early_simple: false
        })
    );
    assert_eq!(return_line(&supported(&s.draft), "5b"), 2_667_700);
    round_trips(&s);
    // (d) Box 8 by the year the benefits were paid reaches the repayment comparison.
    let mut own =
        rrb(24_000, 1_200, 2_400, RailroadRecipient::Employee { recovery: RailroadRecovery::None });
    own.form.box_8 = Some(350_000);
    own.repayments =
        vec![RailroadRepayment { received_year: 2024, amount: 350_000, contributory: true }];
    let s = save_rrb(&d, own);
    assert_eq!(crate::claim_repayment::recorded_repayments(&s.draft), 350_000);
    assert!(crate::claim_repayment::repayment_offered(&s.draft));
    round_trips(&s);
}

#[test]
fn rrb_1099_r_rules_reject_on_the_server_and_import_reads_boxes_4_to_9() {
    let d = filer(19600301);
    let reject = |mut f: RailroadSource, k: &str| {
        let mut s = Interview::new(TEST_USER, d.clone());
        if k != "RRB_BOX7_NOT_SUM" {
            let mut corrected = f.clone();
            if corrected.repayments.is_empty() {
                corrected.repayments.push(RailroadRepayment {
                    received_year: 2024,
                    amount: corrected.box_8.unwrap_or(0),
                    contributory: true,
                });
            } else {
                corrected.form.box_8 =
                    Some(corrected.repayments.iter().map(|row| row.amount).sum());
            }
            crate::tests::support::client::railroad_answer_case(k, &mut s, None, f, corrected, k);
            return;
        }
        let reply = {
            let request = crate::tests::support::client::income_pages::save_documents(
                &mut s,
                vec![document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::Rrb1099R({
                            f.form.withholding = Some(f.withholding.unwrap_or(0).min(f.gross));
                            railroad_document(f)
                        }),
                    },
                )],
            );
            send(&mut s, request)
        };
        let Reply::Response(P::Problem { message, .. }) = reply else {
            panic!("expected a problem for {k}")
        };
        assert_eq!(message, key(k));
    };
    let employee = RailroadRecipient::Employee { recovery: RailroadRecovery::None };
    let mut sum = rrb(24_000, 1_200, 2_400, employee.clone());
    sum.form.gross += 100;
    reject(sum, "RRB_BOX7_NOT_SUM");
    let mut with_rows = rrb(24_000, 1_200, 2_400, employee.clone());
    with_rows.repayments =
        vec![RailroadRepayment { received_year: 2024, amount: 100, contributory: true }];
    reject(with_rows, "RRB_REPAYMENTS_WITHOUT_BOX_8");
    let mut unsplit = rrb(24_000, 1_200, 2_400, employee.clone());
    unsplit.form.box_8 = Some(100);
    reject(unsplit, "RRB_BOX_8_NOT_SPLIT");
    let mut mismatch = rrb(24_000, 1_200, 2_400, employee);
    mismatch.form.box_8 = Some(300);
    mismatch.repayments =
        vec![RailroadRepayment { received_year: 2024, amount: 200, contributory: true }];
    reject(mismatch, "RRB_REPAYMENTS_NOT_BOX_8");
    // Import: printed parties and boxes 3-9 are document-sourced; annuity kind and years are not, and
    // boxes 10 and 11 are an error.
    let model = crate::federal_model::HUB_SOURCE;
    let page = &model[model.find("pub struct FormRrb1099RPaper {").unwrap()..];
    let page = &page[..page.find("\n}").unwrap()];
    for field in [
        "pub issuer",
        "pub recipient:",
        "pub box_3",
        "pub box_4",
        "pub box_5",
        "pub box_6",
        "pub gross",
        "pub box_8",
        "pub withholding",
    ] {
        let at = page.find(field).unwrap();
        let attrs = &page[..at];
        assert!(
            attrs[attrs.rfind("///").unwrap()..].contains("#[libertas_llm_doc_field"),
            "{field}"
        );
    }
    let answers = &model[model.find("pub struct RailroadSource {").unwrap()..];
    let answers = &answers[..answers.find("\n}").unwrap()];
    for field in ["pub recipient_kind", "pub repayments"] {
        assert!(!page.contains(field), "{field} is a separate topic answer");
        let at = answers.find(field).unwrap();
        let attrs = &answers[..at];
        assert!(
            !attrs[attrs.rfind("///").unwrap()..].contains("libertas_llm_doc_field"),
            "{field}"
        );
    }
    assert!(
        model.contains("Report an entry in box 10 (rate of tax) or box 11 (country) as an error")
    );
}

// ---- Form 1099-R box 7c: Trump accounts (section 530A(d)(6) and (d)(4)) ----

fn trump_row(printed: &str, gross: i64, taxable: i64, code: DistributionCode) -> R1099Answered {
    let mut r = form(printed, false, gross, taxable, code, M::BoxTwoA);
    r.source_mut().form.trump = true;
    printed_as(r, printed)
}

// The account's value at the child's death less its basis is the acquirer's income: code 4 with
// box 7c, box 1 1,500 (value), box 2a 500 (value 1,500 - basis 1,000), 10 withheld.
#[test]
fn trump_account_death_is_the_acquirers_income() {
    let d = filer(19600301);
    let before = supported(&d);
    let mut row = trump_row(
        "4",
        1_500,
        500,
        DistributionCode::TrumpDeath { estate: false, basis: Some(100_000) },
    );
    row.source_mut().form.withholding = Some(1_000);
    let s = save(&d, vec![row.clone()]);
    let FederalIncome::TrumpDeath { data } = &s.draft.income[1].income else {
        panic!("a Trump account death")
    };
    assert_eq!(
        TrumpDeathAcquisition::from(data),
        TrumpDeathAcquisition {
            form_boxes: Default::default(),
            box_2a: crate::kernel::ratio::positive((150_000) - (100_000)),
            box_3: 0,
            box_6: 0,
            box_8a: 0,
            box_8b_percent: None,
            box_9a_percent: None,
            box_9b: None,
            box_10: 0,
            box7: crate::DistributionCodes::CODE_4.0,
            ira: false,
            trump: true,
            beneficiary_estate: false,
            value: 150_000,
            basis: Some(100_000),
            withholding: 1_000,
        }
    );
    let r = supported(&s.draft);
    assert_eq!(value(&r, "trump_death_taxable_income"), 500);
    // Schedule 1 line 8z shows it (and line 9 totals it) so line 10 adds up.
    assert_eq!(schedule_1_line(&r, "8z"), 500);
    assert_eq!(schedule_1_line(&r, "9"), 500);
    assert_eq!(value(&r, "agi"), value(&before, "agi") + 500);
    assert_eq!(r.payments, Some(before.payments.unwrap() + 1_000));
    // The page reopens as the 1099-R it was.
    let mut again = restart(s.draft.clone());
    let page = open_documents(&mut again);
    assert_eq!(
        page_documents(&page)[0].document,
        expected_named_source(
            &s.draft,
            Owner::Taxpayer,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(row))
            }
        )
    );
    // The estate case is the child's final return: a living adult is refused by name.
    let estate = trump_row(
        "4",
        1_500,
        500,
        DistributionCode::TrumpDeath { estate: true, basis: Some(100_000) },
    );
    let s = save(&d, vec![estate]);
    let r = estimate(&s.draft);
    assert_eq!(r.state, ResultState::Unsupported);
}

#[test]
fn trump_account_transfer_to_the_able_account_is_not_income() {
    let d = filer(20090704);
    let baseline = supported(&d);
    let row = trump_row("G", 2_000, 0, DistributionCode::TrumpToAble { transferred: 20260801 });
    let s = save(&d, vec![row.clone()]);
    assert!(matches!(
        &s.draft.income[1].income,
        FederalIncome::TrumpAbleRollover { data }
            if TrumpAbleRollover::from(data).amount == 200_000 && TrumpAbleRollover::from(data).transferred == 20260801
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "trump_able_rollover_2_gross"), 2_000);
    assert_eq!(value(&r, "agi"), value(&baseline, "agi"));
    let mut again = restart(s.draft.clone());
    let page = open_documents(&mut again);
    assert_eq!(
        page_documents(&page)[0].document,
        expected_named_source(
            &s.draft,
            Owner::Taxpayer,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(row))
            }
        )
    );
    // Rules: box 2a is zero, the branches need box 7c, and box 7c is not box 7b.
    let taxable = trump_row("G", 2_000, 1, DistributionCode::TrumpToAble { transferred: 20260801 });
    assert_eq!(refusal(&d, vec![taxable]), key("R_TRUMP_ABLE_TAXABLE"));
    let mut no_7c = trump_row(
        "4",
        1_500,
        500,
        DistributionCode::TrumpDeath { estate: false, basis: Some(100_000) },
    );
    no_7c.source_mut().form.trump = false;
    assert_eq!(refusal(&d, vec![no_7c]), key("R_BOX7_TREATMENT"));
    let mut ira = trump_row(
        "4",
        1_500,
        500,
        DistributionCode::TrumpDeath { estate: false, basis: Some(100_000) },
    );
    ira.source_mut().form.ira = true;
    ira.source_mut().code = Some(DistributionCode::Other);
    assert_eq!(refusal(&d, vec![ira]), key("R_TRUMP_AND_IRA"));
}

// ---- Form 1099-R code 4 with box 7b: the Inherited IRA record ----

fn traditional(decedent: &str, prior: i64, year_end: i64) -> InheritedIraAccount {
    InheritedIraAccount {
        decedent: decedent.into(),
        died: 20251201,
        kind: InheritedIraKindFacts::Traditional {
            basis: Some(InheritedIraBasis { prior: prior * 100, year_end: year_end * 100 }),
            estate_tax: None,
        },
    }
}
fn roth_record(
    decedent: &str,
    first_year: u32,
    regular: i64,
    converted: i64,
) -> InheritedIraAccount {
    InheritedIraAccount {
        decedent: decedent.into(),
        died: 20251201,
        kind: InheritedIraKindFacts::Roth {
            first_year,
            basis: Some(InheritedRothBasis { regular: regular * 100, converted: converted * 100 }),
        },
    }
}
fn inherited_row(account: Option<i64>, gross: i64) -> R1099Answered {
    let mut r = printed_as(r1099(gross * 100, DistributionCode::Inherited { account }, true), "4");
    r.source_mut().form.withholding = Some(12_300);
    r
}
fn inherited_roth_row(account: Option<i64>, gross: i64) -> R1099Answered {
    printed_as(
        r1099(
            gross * 100,
            DistributionCode::RothIra {
                sequence: None,
                rolled_over: 0,
                roth: RothIraCode::ExceptionApplies {
                    distributed: 20260601,
                    reason: RothReason::Inherited { account },
                },
            },
            false,
        ),
        "T",
    )
}
fn record_request(s: &mut Interview, rows: Vec<IncomeDocumentItem>) -> P {
    let mut request = open_topic(s, TaxSection::Retirement);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    for row in rows {
        let mut row = crate::IncomeRecordItem::from_income(row).unwrap();
        let old = documents.documents.iter_mut().find(|old| {
            if row.id != 0 {
                old.id == row.id
            } else {
                old.owner == row.owner
                    && matches!(
                        &row.document,
                        crate::IncomeRecord::Form8606(_) | crate::IncomeRecord::RothHistory(_)
                    )
                    && core::mem::discriminant(&old.document)
                        == core::mem::discriminant(&row.document)
            }
        });
        if let Some(old) = old {
            row.id = old.id;
            *old = row;
        } else {
            documents.documents.push(row);
        }
    }
    request
}
fn save_items(d: &FederalDraft, items: Vec<IncomeDocumentItem>) -> Interview {
    let mut s = Interview::new(TEST_USER, d.clone());
    let (records, sources): (Vec<_>, Vec<_>) =
        items.into_iter().partition(|row| matches!(row.document, IncomeSource::Record { .. }));
    let (prior, later): (Vec<_>, Vec<_>) = records.into_iter().partition(|row| {
        !matches!(row.document, IncomeSource::Record { value: crate::IncomeRecord::Qcd(_) })
    });
    if !prior.is_empty() {
        let request = record_request(&mut s, prior);
        accepted(&send(&mut s, request));
    }
    if !sources.is_empty() {
        let request = save_documents(&mut s, sources.clone());
        accepted(&send(&mut s, request));
        crate::tests::support::client::income_pages::save_information_return_answers(
            &mut s, &sources,
        );
    }
    if !later.is_empty() {
        let request = record_request(&mut s, later);
        accepted(&send(&mut s, request));
    }
    s
}
fn record_item(owner: Owner, id: i64, record: InheritedIraAccount) -> IncomeDocumentItem {
    unanswered_document(
        owner,
        id,
        IncomeSource::Record { value: crate::IncomeRecord::InheritedIra(record) },
    )
}
fn row_item(owner: Owner, row: R1099Answered) -> IncomeDocumentItem {
    unanswered_document(
        owner,
        0,
        IncomeSource::Retirement {
            value: crate::RetirementDocument::R1099(retirement_document(row)),
        },
    )
}
fn catalog(page: &P) -> &[RecordChoice] {
    let P::SaveDocuments { inherited_iras, .. } = page else { panic!("the income page") };
    inherited_iras
}
fn roth_catalog(page: &P) -> &[RecordChoice] {
    let P::SaveDocuments { inherited_roth_iras, .. } = page else { panic!("the income page") };
    inherited_roth_iras
}

// One record for the person inherited from and the 1099-R (code 4, box 7b) that chooses it: Pub.
// 590-B's allocation. Distribution 10,000, allocated basis 5,000, December 31 value 40,000: basis
// recovered 10,000 x 5,000 / (10,000 + 40,000) = 1,000, remaining 4,000; line 4a 10,000, line 4b
// 9,000; no 10% tax (section 72(t)(2)(A)(ii)); the 1099-R's box 4 withholding counts.
#[test]
fn inherited_ira_record_and_its_1099_r_on_one_page() {
    let d = filer(19600301);
    let before = supported(&d);
    let sent = vec![
        record_item(Owner::Taxpayer, 0, traditional("Pat Sample", 5_000, 40_000)),
        row_item(Owner::Taxpayer, inherited_row(None, 10_000)),
    ];
    let mut s = save_items(&d, sent.clone());
    let FederalIncome::Retirement { data } = &s.draft.income[2].income else {
        panic!("a retirement record")
    };
    assert_eq!(
        (
            crate::FederalRetirement::from(data).taxable.clone(),
            crate::FederalRetirement::from(data).box_2a
        ),
        (Some(RetirementTaxable::Inherited { account: None }), 1_000_000)
    );
    assert_eq!(crate::FederalRetirement::from(data).kind, RetirementKind::Inherited);
    assert_eq!(s.draft.income[1].label, "Pat Sample");
    let r = supported(&s.draft);
    assert_eq!(value(&r, "inherited_ira_2_basis_recovered"), 1_000);
    assert_eq!(value(&r, "inherited_ira_2_basis_remaining"), 4_000);
    assert_eq!(return_line(&r, "4a"), 1_000_000);
    assert_eq!(return_line(&r, "4b"), 900_000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(r.payments.unwrap() - before.payments.unwrap(), 12_300);
    round_trips(&s);
    // Each editor reopens with the same accepted identities and payloads.
    let saved = observed(&mut s);
    let mut again = restart(s.draft.clone());
    let page = open_documents(&mut again);
    assert_eq!(observed(&mut again), saved);
    assert_fixture(&mut again, &sent);
    let choices = catalog(&page);
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].id, 2);
    let label = status_text(&choices[0].label);
    assert!(label.contains("Pat Sample") && label.contains("traditional IRA"), "{label}");
    assert!(label.contains("You"), "{label}");
}

// The catalog round trip: a 1099-R, an inherited Roth withdrawal and a QCD gift choose their
// record by its identity, keep the choice across a reopen, and each group uses its own basis.
//   Pat: basis 5,000, value 40,000, distributions 10,000 -> basis recovered 1,000 (4b 9,000).
//   Lee: basis 10,000, value 0, distributions 2,000 -> all 2,000 recovered (nothing taxable).
//   Kim's Roth (first year 2022): 4,000 with regular basis 1,000 and conversions 2,500 -> 500.
#[test]
fn inherited_choices_round_trip_through_the_catalog_and_use_their_own_basis() {
    let mut d = filer(19500101);
    let first = save_items(
        &d,
        vec![
            record_item(Owner::Taxpayer, 0, traditional("Pat Sample", 5_000, 40_000)),
            record_item(Owner::Taxpayer, 0, traditional("Lee Sample", 10_000, 0)),
            record_item(Owner::Taxpayer, 0, roth_record("Kim Sample", 2022, 1_000, 2_500)),
        ],
    );
    d = first.draft.clone();
    let mut s = Interview::new(TEST_USER, d.clone());
    let page = open_documents(&mut s);
    // Each field is offered only the records of its own kind (F3, 2026-10-09):
    // the traditional ones for a code 4 payment or a QCD gift, the Roth one for
    // an inherited Roth withdrawal.
    let ids: Vec<i64> = catalog(&page).iter().map(|c| c.id).collect();
    assert_eq!(ids, vec![2, 3]);
    let roth_ids: Vec<i64> = roth_catalog(&page).iter().map(|c| c.id).collect();
    assert_eq!(roth_ids, vec![4]);
    let roth_label = status_text(&roth_catalog(&page)[0].label);
    assert!(roth_label.contains("Roth IRA"), "{roth_label}");
    let items = vec![
        record_item(Owner::Taxpayer, 2, traditional("Pat Sample", 5_000, 40_000)),
        record_item(Owner::Taxpayer, 3, traditional("Lee Sample", 10_000, 0)),
        record_item(Owner::Taxpayer, 4, roth_record("Kim Sample", 2022, 1_000, 2_500)),
        row_item(Owner::Taxpayer, inherited_row(Some(2), 10_000)),
        row_item(Owner::Taxpayer, inherited_row(Some(3), 2_000)),
        row_item(Owner::Taxpayer, inherited_roth_row(Some(4), 4_000)),
        unanswered_document(
            Owner::Taxpayer,
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::Qcd(QcdGift {
                    split_interest: false,
                    date: 20260601,
                    amount: 500_000,
                    inherited: Some(2),
                }),
            },
        ),
    ];
    let mut s = save_items(&d, items.clone());
    let r = supported(&s.draft);
    assert_eq!(value(&r, "inherited_ira_3_basis_remaining"), 8_000);
    assert_eq!(value(&r, "inherited_roth_4_regular_recovered"), 1_000);
    assert_eq!(value(&r, "inherited_roth_4_conversion_recovered"), 2_500);
    assert_eq!(value(&r, "qcd_exclusion"), 5_000);
    assert_eq!(
        s.draft.adjustments.as_ref().unwrap().qualified_charity[0].payments[0].inherited,
        Some(2)
    );
    let saved = observed(&mut s);
    let mut again = restart(s.draft.clone());
    assert_eq!(observed(&mut again), saved);
    assert_fixture(&mut again, &items);
}

#[test]
fn an_inherited_payment_without_a_record_is_taxed_in_full_with_a_note() {
    let d = filer(19600301);
    let s = save_items(&d, vec![row_item(Owner::Taxpayer, inherited_row(None, 10_000))]);
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_INHERITED_IRA_RECORD_MISSING"));
    assert_eq!(return_line(&r, "4a"), 1_000_000);
    assert_eq!(return_line(&r, "4b"), 1_000_000);
    // Two records and none chosen: the same.
    let s = save_items(
        &d,
        vec![
            record_item(Owner::Taxpayer, 0, traditional("Pat Sample", 5_000, 40_000)),
            record_item(Owner::Taxpayer, 0, traditional("Lee Sample", 10_000, 0)),
            row_item(Owner::Taxpayer, inherited_row(None, 10_000)),
        ],
    );
    assert!(noted(&supported(&s.draft), "WARN_INHERITED_IRA_RECORD_MISSING"));
}

#[test]
fn an_inherited_payment_naming_another_filers_or_the_other_kinds_record_is_refused() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19600301;
    let first = save_items(
        &d,
        vec![
            record_item(Owner::Spouse, 0, traditional("Pat Sample", 5_000, 40_000)),
            record_item(Owner::Taxpayer, 0, roth_record("Kim Sample", 2022, 1_000, 2_500)),
        ],
    );
    let d = first.draft;
    let items = |row: R1099Answered| {
        vec![
            record_item(Owner::Spouse, 2, traditional("Pat Sample", 5_000, 40_000)),
            record_item(Owner::Taxpayer, 3, roth_record("Kim Sample", 2022, 1_000, 2_500)),
            row_item(Owner::Taxpayer, row),
        ]
    };
    let issue = |row: R1099Answered, k: &str| {
        let s = save_items(&d, items(row));
        let r = estimate(&s.draft);
        assert_eq!(r.state, ResultState::Unsupported, "{k}");
        assert!(r.issues.iter().any(|i| i.display.key == k), "{k}: {:?}", r.issues);
    };
    // The taxpayer's payment names the spouse's record.
    issue(inherited_row(Some(2), 10_000), "ISSUE_INHERITED_IRA_OTHER_OWNER");
    // A traditional payment names the Roth record, and a Roth withdrawal the traditional one.
    issue(inherited_row(Some(3), 10_000), "ISSUE_INHERITED_IRA_KIND_MISMATCH");
    issue(inherited_roth_row(Some(2), 4_000), "ISSUE_INHERITED_IRA_OTHER_OWNER");
}

#[test]
fn an_inherited_roth_without_basis_taxes_earnings_unless_the_owners_five_years_are_over() {
    let d = filer(19600301);
    let record = |first_year: u32| {
        let mut r = roth_record("Kim Sample", first_year, 0, 0);
        if let InheritedIraKindFacts::Roth { basis, .. } = &mut r.kind {
            *basis = None;
        }
        r
    };
    let page = |first_year| {
        vec![
            record_item(Owner::Taxpayer, 0, record(first_year)),
            row_item(Owner::Taxpayer, inherited_roth_row(None, 4_000)),
        ]
    };
    let r = supported(&save_items(&d, page(2022)).draft);
    assert!(noted(&r, "WARN_INHERITED_ROTH_BASIS_ASSUMED_NONE"));
    assert_eq!(return_line(&r, "4a"), 400_000);
    assert_eq!(return_line(&r, "4b"), 400_000);
    let r = supported(&save_items(&d, page(2021)).draft);
    assert!(!noted(&r, "WARN_INHERITED_ROTH_BASIS_ASSUMED_NONE"));
    assert_eq!(return_line(&r, "4b"), 0);
}

// A Roth account's first year is not after the year of death: the page rule, repeated on save with
// its own key. The year of death itself is allowed (died 2025-12-01).
#[test]
fn an_inherited_roth_first_year_after_the_year_of_death_is_refused_with_its_key() {
    let d = filer(19600301);
    for (first_year, allowed) in [(2025, true), (2026, false)] {
        let mut s = Interview::new(TEST_USER, d.clone());
        let page = save_topic(
            &mut s,
            TaxSection::Retirement,
            vec![record_item(Owner::Taxpayer, 0, roth_record("Kim Sample", first_year, 0, 0))],
        );
        let reply = send(&mut s, page);
        if allowed {
            accepted(&reply);
        } else {
            let Reply::Response(P::Problem { message, .. }) = reply else {
                panic!("expected a refused save")
            };
            assert_eq!(message, key("INHERITED_ROTH_FIRST_YEAR_AFTER_DEATH"));
            assert_eq!(s.draft, d);
        }
    }
}

// The filer's own Form 8606 switches the filer's own IRA distributions to the basis worksheet and
// never an inherited one: the inherited 1099-R keeps its choice and its record's basis.
#[test]
fn a_form_8606_does_not_take_over_an_inherited_ira_payment() {
    let d = filer(19600301);
    let s = save_items(
        &d,
        vec![
            record_item(Owner::Taxpayer, 0, traditional("Pat Sample", 5_000, 40_000)),
            row_item(Owner::Taxpayer, inherited_row(None, 10_000)),
            unanswered_document(
                Owner::Taxpayer,
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::Form8606(Form8606 {
                        adjustment: 0,
                        reservist_repayments: vec![],
                        basis: 500_000,
                        year_end: 900_000,
                    }),
                },
            ),
        ],
    );
    let FederalIncome::Retirement { data } = &s.draft.income[2].income else {
        panic!("a retirement record")
    };
    assert_eq!(
        (
            crate::FederalRetirement::from(data).taxable.clone(),
            crate::FederalRetirement::from(data).box_2a
        ),
        (Some(RetirementTaxable::Inherited { account: None }), 1_000_000)
    );
    let r = supported(&s.draft);
    assert_eq!(value(&r, "inherited_ira_2_basis_recovered"), 1_000);
    assert_eq!(return_line(&r, "4b"), 900_000);
}

#[test]
fn special_retirement_kinds_keep_corrected_paper_boxes_and_personal_elections() {
    use crate::tests::support::client::income_pages::{
        information_return_answers, save_information_return_answers,
    };
    let rows = [
        form(
            "J",
            false,
            1_500,
            1_500,
            DistributionCode::RothIra {
                roth: crate::RothIraCode::Early {
                    distributed: 20260601,
                    exception: None,
                    repaid: None,
                },
                rolled_over: 0,
                sequence: None,
            },
            M::BoxTwoA,
        ),
        trump_row(
            "4",
            1_500,
            500,
            DistributionCode::TrumpDeath { estate: false, basis: Some(100_000) },
        ),
        trump_row("G", 1_500, 0, DistributionCode::TrumpToAble { transferred: 20260801 }),
    ];
    for original in rows {
        let mut session = save(&filer(19600301), vec![original.clone()]);
        let entry = session
            .draft
            .income
            .iter()
            .find(|entry| {
                matches!(
                    entry.income,
                    FederalIncome::Roth { .. }
                        | FederalIncome::TrumpDeath { .. }
                        | FederalIncome::TrumpAbleRollover { .. }
                )
            })
            .unwrap()
            .clone();
        let mut corrected = original.clone();
        corrected.source_mut().form.box_6 = Some(200);
        corrected.source_mut().form.box_8a = Some(300);
        corrected.source_mut().form.box_8b_percent = Some(25.0);
        corrected.source_mut().form.box_9a_percent = Some(50.0);
        corrected.source_mut().form.box_9b = Some(400);
        corrected.source_mut().form.box_10 = Some(500);
        corrected.source_mut().form.withholding = Some(600);
        let row = |source| {
            vec![document(
                entry.id,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(source)),
                },
            )]
        };
        let request = save_documents(&mut session, row(corrected.clone()));
        accepted(&send(&mut session, request));
        assert!(session.draft.valid_stored());
        session = restart(session.draft);
        let page = open_documents(&mut session);
        let actual = all_page_documents(&page).into_iter().find(|row| row.id == entry.id).unwrap();
        let IncomeSource::Retirement { value: crate::RetirementDocument::R1099(actual) } =
            actual.document
        else {
            unreachable!()
        };
        assert_eq!(
            (
                actual.gross,
                actual.taxable,
                actual.withholding,
                actual.box7,
                actual.ira,
                actual.trump
            ),
            (
                corrected.gross,
                corrected.taxable,
                Some(600),
                corrected.box7,
                corrected.ira,
                corrected.trump
            )
        );
        assert_eq!(
            (
                actual.box_6,
                actual.box_8a,
                actual.box_8b_percent,
                actual.box_9a_percent,
                actual.box_9b,
                actual.box_10
            ),
            (Some(200), Some(300), Some(25.0), Some(50.0), Some(400), Some(500))
        );
        let accepted_code =
            match &session.draft.income.iter().find(|row| row.id == entry.id).unwrap().income {
                FederalIncome::Roth { data }
                | FederalIncome::TrumpDeath { data }
                | FederalIncome::TrumpAbleRollover { data } => &data.code,
                _ => unreachable!(),
            };
        assert_eq!(*accepted_code, original.code);
        let Reply::Response(P::Review { remaining_items, .. }) =
            send(&mut session, P::ReviewAccepted { user: TEST_USER })
        else {
            unreachable!()
        };
        let request = information_return_answers(&mut session, &row(corrected));
        if matches!(original.code, Some(DistributionCode::RothIra { .. })) {
            assert!(remaining_items.iter().any(|row| row.section == TaxSection::Retirement));
            let before = session.draft.clone();
            let Reply::Response(P::Problem { message, .. }) = send(&mut session, request) else {
                unreachable!()
            };
            assert_eq!(message, key("R_ROTH_OTHER_BOXES"));
            assert_eq!(session.draft, before);
        } else {
            // The original Trump owner permits these unused printed boxes.
            accepted(&send(&mut session, request));
        }
        let request = save_documents(&mut session, row(original.clone()));
        accepted(&send(&mut session, request));
        save_information_return_answers(&mut session, &row(original));
        assert_eq!(session.draft.income.iter().find(|row| row.id == entry.id).unwrap(), &entry);
    }
}

#[test]
fn trump_death_missing_actual_basis_round_trips_and_requires_topic_answer() {
    use crate::tests::support::client::income_pages::{
        information_return_answers, save_information_return_answers,
    };
    let original = trump_row(
        "4",
        1_500,
        500,
        DistributionCode::TrumpDeath { estate: false, basis: Some(100_000) },
    );
    let mut session = save(&filer(19600301), vec![original.clone()]);
    let entry = session
        .draft
        .income
        .iter_mut()
        .find(|entry| matches!(entry.income, FederalIncome::TrumpDeath { .. }))
        .unwrap();
    let id = entry.id;
    let FederalIncome::TrumpDeath { data } = &mut entry.income else { unreachable!() };
    let Some(DistributionCode::TrumpDeath { basis, .. }) = &mut data.code else { unreachable!() };
    *basis = None;
    assert_eq!(TrumpDeathAcquisition::from(&*data).taxable(), None);
    assert_eq!(TrumpDeathAcquisition::from(&*data).box_2a, 50_000);
    session = restart(session.draft);
    let request = open_documents(&mut session);
    accepted(&send(&mut session, request));
    let entry = session.draft.income.iter().find(|entry| entry.id == id).unwrap();
    assert!(
        matches!(&entry.income, FederalIncome::TrumpDeath { data } if TrumpDeathAcquisition::from(data).basis.is_none() && TrumpDeathAcquisition::from(data).box_2a == 50_000)
    );
    let Reply::Response(P::Review { remaining_items, .. }) =
        send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        unreachable!()
    };
    assert!(remaining_items.iter().any(|row| row.section == TaxSection::Retirement));
    let mut missing = original.clone();
    missing.source_mut().code = Some(DistributionCode::TrumpDeath { estate: false, basis: None });
    let rows = |source| {
        vec![document(
            id,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(source)),
            },
        )]
    };
    let request = information_return_answers(&mut session, &rows(missing));
    let before = session.draft.clone();
    let Reply::Response(P::Problem { message, .. }) = send(&mut session, request) else {
        unreachable!()
    };
    assert_eq!(message, key("R_TRUMP_BASIS_NEEDED"));
    assert_eq!(session.draft, before);
    save_information_return_answers(&mut session, &rows(original));
    assert_eq!(value(&supported(&session.draft), "trump_death_taxable_income"), 500);
}

// G-34 and G-35 (Smith round 3): an own employee annuity with the Simplified Method started
// 2026-06-01, 7 months, one recipient, aged 65, box 3 $8,400. The empty cost takes box 3 and the age
// comes from the owner's date of birth: the page raises nothing, the figures equal the entered ones.
fn rrb_own_basis(cost: Option<i64>, duration: PensionDuration) -> RailroadSource {
    let mut own = rrb(
        2_100,
        0,
        0,
        RailroadRecipient::Employee {
            recovery: RailroadRecovery::SimplifiedMethod {
                data: PensionBasis {
                    started: 20260601,
                    cost,
                    months: 7,
                    history: PensionHistory::New { duration },
                    ..simplified_cost(0)
                },
            },
        },
    );
    own.form.box_3 = Some(840_000);
    own
}
#[test]
fn rrb_simplified_method_takes_box_3_cost_and_the_owner_age() {
    let d = filer(19600801);
    let entered = save_rrb(&d, rrb_own_basis(Some(840_000), PensionDuration::Single { age: 65 }));
    let derived = save_rrb(&d, rrb_own_basis(None, PensionDuration::Own { age: None }));
    let FederalIncome::Retirement { data } = &derived.draft.income[1].income else {
        panic!("a retirement record")
    };
    let crate::RetirementSource::Rrb1099R(source) = data else { panic!("an RRB-1099-R") };
    let Some(RailroadRecipient::Employee {
        recovery: RailroadRecovery::SimplifiedMethod { data: basis },
    }) = &source.recipient_kind
    else {
        panic!("the Simplified Method")
    };
    // The cost stays empty (it follows box 3); the age is the owner's on 2026-06-01.
    assert_eq!(basis.cost, None);
    assert_eq!(
        basis.history,
        PensionHistory::New { duration: PensionDuration::Own { age: Some(65) } }
    );
    let (entered, derived) = (supported(&entered.draft), supported(&derived.draft));
    assert_eq!(return_line(&derived, "5b"), return_line(&entered, "5b"));
    assert!(return_line(&derived, "5b") < 210_000);
}

#[test]
fn pension_age_follows_the_birth_date_and_the_start() {
    let mut basis = simplified_cost(8_400);
    basis.started = 20260601;
    basis.history = PensionHistory::New { duration: PensionDuration::Own { age: None } };
    // Born 1960-08-01: 65 on 2026-06-01, 66 after the birthday; a start before birth refuses.
    basis.follow_birth(Some(19600801));
    assert_eq!(
        basis.history,
        PensionHistory::New { duration: PensionDuration::Own { age: Some(65) } }
    );
    basis.started = 20260901;
    basis.follow_birth(Some(19600801));
    assert_eq!(
        basis.history,
        PensionHistory::New { duration: PensionDuration::Own { age: Some(66) } }
    );
    assert!(!basis.lacks_birth(Some(19600801)));
    assert!(basis.lacks_birth(None));
    assert!(basis.lacks_birth(Some(20270101)));
    // An entered age is the person's own and is never replaced.
    basis.history = PensionHistory::New { duration: PensionDuration::Single { age: 15 } };
    basis.follow_birth(Some(19600801));
    assert_eq!(
        basis.history,
        PensionHistory::New { duration: PensionDuration::Single { age: 15 } }
    );
    assert!(!basis.lacks_birth(None));
}
