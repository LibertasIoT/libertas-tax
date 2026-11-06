// Actual Save/Estimate witnesses for the remaining retirement key family.
// Invalid edits are made to an accepted payer source; correction restores that
// source and verifies its independent pension/withholding output and lifecycle.
use super::unit47_credit_source_validation::credit_estimate_documents_from_save;
use super::*;
use crate::tests::support::client::income_pages::{open_documents, save_documents};

fn gap_plan() -> R1099Answered {
    let mut row = r1099(1_000_000, normal_kept(), false);
    row.source_mut().form.taxable = Some(600_000);
    row.source_mut().form.withholding = Some(100_000);
    row
}

fn gap_row(code: DistributionCode, ira: bool) -> R1099Answered {
    let mut row = r1099(1_000_000, code, ira);
    row.source_mut().form.taxable = Some(600_000);
    row.source_mut().form.withholding = Some(100_000);
    row
}

fn gap_qualified_disaster() -> DisasterDistribution {
    DisasterDistribution::Qualified {
        fema: 4834,
        began: 20260601,
        declared: 20260602,
        distributed: 20260603,
        amount: 500_000,
        earlier: 0,
        spread: DisasterSpread::OverThreeYears,
        repaid: DisasterRepayment::No,
    }
}

fn gap_home_disaster(ended: u32, taxable_repaid: i64) -> DisasterDistribution {
    DisasterDistribution::HomePurchase {
        fema: 4834,
        began: 20260810,
        ended,
        declared: 20260811,
        distributed: 20260801,
        taxable_repaid,
    }
}

fn gap_bad_row(case: &str) -> (R1099Answered, &'static str, &'static str) {
    use DistributionCode as Code;
    let mut row = gap_plan();
    let (key, path) = match case {
        "h-to-roth" => {
            row = printed_as(gap_row(Code::DirectRollover { to: RolloverTarget::Ira }, false), "H");
            row.source_mut().form.taxable = Some(0);
            ("R_BOX7_H_TO_ROTH", "code/to")
        }
        "drac-direct" => {
            row =
                printed_as(gap_row(Code::DirectRollover { to: RolloverTarget::Ira }, false), "BG");
            row.source_mut().form.taxable = Some(0);
            ("R_DRAC_DIRECT_ROLLOVER", "code/to")
        }
        "drac-indirect" | "drac-indirect-early" => {
            let what_happened =
                Disposition::RolledOver { amount: 500_000, into: RolloverDestination::Ira };
            row = printed_as(
                gap_row(
                    if case == "drac-indirect-early" {
                        Code::Early { what_happened, exception: EarlyException::None }
                    } else {
                        Code::Normal { what_happened }
                    },
                    false,
                ),
                if case == "drac-indirect-early" { "1B" } else { "7B" },
            );
            ("R_DRAC_ROLLOVER_TO_ROTH", "code/what_happened")
        }
        "employer-roth" => {
            row = gap_row(Code::EmployerRothContribution, true);
            ("R_EMPLOYER_ROTH_CONTRIBUTION", "code")
        }
        "excess-deferral-ira" => {
            row = gap_row(Code::Corrective { excess_deferral: 100_000 }, true);
            ("R_EXCESS_DEFERRAL_NOT_A_PLAN", "code/excess_deferral")
        }
        "exchange-boxes" => {
            row = gap_row(Code::Exchange, false);
            row.source_mut().form.taxable = Some(0);
            row.source_mut().disaster = Some(DisasterDistribution::NotDeclared);
            ("R_EXCHANGE_OTHER_BOXES", "disaster")
        }
        "inherited-boxes" => {
            row = gap_row(Code::Inherited { account: None }, true);
            row.source_mut().disaster = Some(DisasterDistribution::NotDeclared);
            ("R_INHERITED_OTHER_BOXES", "disaster")
        }
        "trump-death-boxes" => {
            row = gap_row(Code::TrumpDeath { estate: false, basis: Some(400_000) }, false);
            row.source_mut().form.trump = true;
            row.source_mut().disaster = Some(DisasterDistribution::NotDeclared);
            ("R_INHERITED_OTHER_BOXES", "disaster")
        }
        "trump-able-boxes" => {
            row = gap_row(Code::TrumpToAble { transferred: 20260704 }, false);
            row.source_mut().form.trump = true;
            row.source_mut().form.taxable = Some(0);
            row.source_mut().disaster = Some(DisasterDistribution::NotDeclared);
            ("R_INHERITED_OTHER_BOXES", "disaster")
        }
        "roth-disaster" => {
            row = gap_row(
                Code::RothIra { roth: RothIraCode::Qualified, rolled_over: 0, sequence: None },
                false,
            );
            row.source_mut().form.taxable = Some(0);
            row.source_mut().disaster = Some(gap_home_disaster(20260820, 0));
            ("R_ROTH_DISASTER_KIND", "disaster")
        }
        "roth-direct-from-ira" => {
            row = gap_row(Code::DirectRollover { to: RolloverTarget::Roth }, true);
            ("R_ROTH_ROLLOVER_FROM_A_PLAN", "code/to")
        }
        "roth-indirect-from-ira" | "roth-indirect-early-from-ira" => {
            let what_happened =
                Disposition::RolledOver { amount: 500_000, into: RolloverDestination::Roth };
            row = gap_row(
                if case == "roth-indirect-early-from-ira" {
                    Code::Early { what_happened, exception: EarlyException::None }
                } else {
                    Code::Normal { what_happened }
                },
                true,
            );
            ("R_ROTH_ROLLOVER_FROM_A_PLAN", "code/what_happened/into")
        }
        "special-code-boxes" | "not-income-boxes" | "no-additional-tax-boxes" => {
            row = gap_row(
                match case {
                    "not-income-boxes" => Code::NotIncome,
                    "no-additional-tax-boxes" => Code::NoAdditionalTax,
                    _ => Code::Corrective { excess_deferral: 0 },
                },
                false,
            );
            if case == "not-income-boxes" {
                row.source_mut().form.taxable = Some(0);
            }
            row.source_mut().disaster = Some(DisasterDistribution::NotDeclared);
            ("R_SPECIAL_CODE_OTHER_BOXES", "disaster")
        }
        "trump-death-needs-flag" => {
            row = gap_row(Code::TrumpDeath { estate: false, basis: Some(400_000) }, false);
            // Native constrained-treatment admission precedes the later
            // R_TRUMP_BRANCH_NEEDS_BOX_7C check. Preserve that first refusal.
            ("R_BOX7_TREATMENT", "code")
        }
        "trump-able-needs-flag" => {
            row = gap_row(Code::TrumpToAble { transferred: 20260704 }, false);
            row.source_mut().form.taxable = Some(0);
            ("R_BOX7_TREATMENT", "code")
        }
        "trump-codes" => {
            // The shared r1099 fixture derives printed codes from the branch;
            // Another code deliberately requires its own printed source.
            row.source_mut().code = Some(Code::Other);
            row.source_mut().form.trump = true;
            row = printed_as(row, "7");
            ("R_TRUMP_CODES", "box7")
        }
        "plan-withdrawal" => {
            // The payer did not settle box 2a; this fixture exercises its worksheet.
            row.source_mut().form.taxable_not_determined = true;
            row.source_mut().taxable_method = Some(TaxableMethod::PlanWithdrawal {
                data: PlanDistribution { value: 999_999, investment: 200_000, eligible: true },
            });
            ("R_WITHDRAWAL_ABOVE_BALANCE", "more/taxable_method/data/value")
        }
        "annuity-withdrawal" => {
            row = gap_row(Code::Nonqualified, false);
            // The payer did not settle box 2a; this fixture exercises its worksheet.
            row.source_mut().form.taxable_not_determined = true;
            row.source_mut().taxable_method = Some(TaxableMethod::AnnuityWithdrawal {
                data: AnnuityDistribution {
                    loan_pledge: false,
                    value: 999_999,
                    investment: 200_000,
                    related: vec![],
                    eligible: true,
                },
            });
            ("R_WITHDRAWAL_ABOVE_BALANCE", "more/taxable_method/data/value")
        }
        "csrs-recovery" => {
            row = gap_row(Code::Death { died: 20251231 }, false);
            // The payer did not settle box 2a; this fixture exercises its worksheet.
            row.source_mut().form.taxable_not_determined = true;
            row.source_mut().taxable_method = Some(TaxableMethod::CivilServiceLumpSum {
                data: SurvivorLumpSum { cost: 300_000, recovered: 300_001, eligible: true },
            });
            ("CSRS_RECOVERED_ABOVE_COST", "more/taxable_method/data/recovered")
        }
        "disaster-direct" => {
            row = gap_row(Code::DirectRollover { to: RolloverTarget::Ira }, false);
            row.source_mut().form.taxable = Some(0);
            row.source_mut().disaster = Some(gap_qualified_disaster());
            ("DISASTER_DIRECT_ROLLOVER", "code")
        }
        "disaster-ended" => {
            // The distribution is within the independent Part IV window,
            // isolating the reversed incident endpoints' own guard.
            row.source_mut().disaster = Some(gap_home_disaster(20260809, 0));
            ("DISASTER_ENDED_BEFORE_BEGAN", "disaster/ended")
        }
        "disaster-home-kept" => {
            row = gap_row(Code::DirectRollover { to: RolloverTarget::Ira }, false);
            row.source_mut().form.taxable = Some(0);
            row.source_mut().disaster = Some(gap_home_disaster(20260820, 0));
            ("DISASTER_HOME_KEPT", "code")
        }
        "disaster-home-repaid" => {
            row.source_mut().disaster = Some(gap_home_disaster(20260820, 600_001));
            ("DISASTER_HOME_REPAID_ABOVE_TAXABLE", "disaster/taxable_repaid")
        }
        "property-after-tax"
        | "property-early-after-tax"
        | "property-lump-after-tax"
        | "property-sold" => {
            let what_happened = Disposition::SoldAndRolledOver {
                data: PropertyRollover {
                    distributed: 20260601,
                    sold: if case == "property-sold" { 20260531 } else { 20260602 },
                    proceeds: 1_200_000,
                    confirmed: true,
                },
                amount: 500_000,
                into: RolloverDestination::Ira,
            };
            row = gap_row(
                match case {
                    "property-early-after-tax" => {
                        Code::Early { what_happened, exception: EarlyException::None }
                    }
                    "property-lump-after-tax" => {
                        Code::NormalLumpSum { tax: LumpSumTax::WithOtherIncome { what_happened } }
                    }
                    _ => Code::Normal { what_happened },
                },
                false,
            );
            if case == "property-sold" {
                row.source_mut().form.taxable = Some(row.gross);
                ("PROPERTY_SOLD_BEFORE_DISTRIBUTION", "code/what_happened/data/sold")
            } else {
                ("PROPERTY_ROLLOVER_AFTER_TAX", "taxable")
            }
        }
        _ => unreachable!(),
    };
    // Echo the sole owner's normal calculated choices; no stale or guessed
    // allowed-set values are needed for these server admission probes.
    (row, key, path)
}

fn gap_refuse(session: &mut Interview, request: P, case: &str, key: &str, path: &str) {
    let request = P::from_avro(&request.to_avro()).unwrap();
    crate::tests::support::client::survey::survey_refusal(case, session, request, key, path);
}

fn gap_plan_fixture() -> Interview {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![
            document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(r1099(
                        200_000,
                        normal_kept(),
                        false,
                    ))),
                },
            ),
            document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(gap_plan())),
                },
            ),
        ],
    );
    let desired = crate::tests::support::client::income_pages::page_documents(&request);
    accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
    crate::tests::support::client::income_pages::save_information_return_answers(
        &mut session,
        &desired,
    );
    let result = supported(&session.draft);
    assert_eq!(return_line(&result, "5a"), 1_200_000);
    assert_eq!(return_line(&result, "5b"), 800_000);
    assert_eq!(return_line(&result, "25b"), 100_000);
    assert_eq!(value(&result, "agi"), 68_000);
    session
}

#[test]
fn unit47_retirement_remaining_native_keys_preserve_actual_save_estimate_and_sources() {
    for case in [
        "h-to-roth",
        "drac-direct",
        "drac-indirect",
        "drac-indirect-early",
        "employer-roth",
        "excess-deferral-ira",
        "exchange-boxes",
        "inherited-boxes",
        "trump-death-boxes",
        "trump-able-boxes",
        "roth-disaster",
        "roth-direct-from-ira",
        "roth-indirect-from-ira",
        "roth-indirect-early-from-ira",
        "special-code-boxes",
        "not-income-boxes",
        "no-additional-tax-boxes",
        "trump-death-needs-flag",
        "trump-able-needs-flag",
        "trump-codes",
        "plan-withdrawal",
        "annuity-withdrawal",
        "csrs-recovery",
        "disaster-direct",
        "disaster-ended",
        "disaster-home-kept",
        "disaster-home-repaid",
        "property-after-tax",
        "property-early-after-tax",
        "property-lump-after-tax",
        "property-sold",
    ] {
        let session = gap_plan_fixture();
        gap_plan_case(case, session);
    }
}

#[inline(never)]
fn gap_plan_case(case: &str, mut session: Interview) {
    let baseline = session.draft.clone();
    let mut request = open_documents(&mut session);
    let correction = request.clone();
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    assert_eq!(documents.common.retirement.documents.len(), 2);
    assert!(documents.common.retirement.documents.iter().all(|item| item.id > 0));
    let source_ids: Vec<_> =
        documents.common.retirement.documents.iter().map(|item| item.id).collect();
    assert_ne!(source_ids[0], source_ids[1]);
    let (bad, key, local_path) = gap_bad_row(case);
    documents.common.retirement.documents[1].document =
        crate::RetirementDocument::R1099(retirement_document(bad.clone()));
    let path = alloc::format!("[documents/common/retirement/documents/1/document/{local_path}]");
    let mut candidate = restart(session.draft.clone());
    let source_reply = send(&mut candidate, request.clone());
    if matches!(source_reply, Reply::Response(P::Problem { .. })) {
        let calculation = credit_estimate_documents_from_save(&request);
        gap_refuse(
            &mut session,
            request,
            &alloc::format!("retirement-gap-{case}-save"),
            key,
            &path,
        );
        gap_refuse(
            &mut session,
            calculation,
            &alloc::format!("retirement-gap-{case}-estimate"),
            key,
            &path,
        );
    } else {
        let desired = vec![document(
            source_ids[1],
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(bad.clone())),
            },
        )];
        let request = crate::tests::support::client::income_pages::information_return_answers(
            &mut candidate,
            &desired,
        );
        let P::SaveIncomes { followups, .. } = &request else { unreachable!() };
        let field = match local_path.split('/').next().unwrap() {
            "disaster" => "disaster",
            "public_safety" => "public_safety",
            "more" => "method",
            _ => "treatment",
        };
        let field =
            if key.starts_with("DISASTER_") && bad.disaster.is_some() { "disaster" } else { field };
        let question = alloc::format!("income/{}/{}", source_ids[1], field);
        if followups.iter().any(|row| row.key == question) {
            let path = retirement_followup_path(&request, source_ids[1], local_path);
            let calculation = credit_estimate_documents_from_save(&request);
            gap_refuse(
                &mut candidate,
                request,
                &alloc::format!("retirement-gap-{case}-save"),
                key,
                &path,
            );
            gap_refuse(
                &mut candidate,
                calculation,
                &alloc::format!("retirement-gap-{case}-estimate"),
                key,
                &path,
            );
        } else {
            assert_eq!(field, "disaster", "{case}: an applicable question must remain editable");
            let page = open_documents(&mut candidate);
            let P::SaveDocuments { documents, .. } = page else { unreachable!() };
            let crate::RetirementDocument::R1099(retained) =
                &documents.common.retirement.documents[1].document
            else {
                unreachable!()
            };
            assert!(
                crate::RCalls::for_boxes(
                    retained.box7,
                    retained.ira,
                    retained.trump,
                    retained.box_10.unwrap_or(0)
                )
                .disaster
                .is_empty(),
                "{case}: the printed class excludes this question"
            );
            assert!(
                matches!(&candidate.draft.income.iter().find(|entry| entry.id == source_ids[1]).unwrap().income, FederalIncome::Retirement { data: RetirementSource::R1099(source) } if source.disaster.is_none()),
                "{case}: the hidden source echo cannot create an answer"
            );
        }
    }
    let dirty = session.dirty;
    accepted(&crate::tests::support::client::income_pages::candidate_save_review(
        &mut session,
        P::from_avro(&credit_estimate_documents_from_save(&correction).to_avro()).unwrap(),
    ));
    assert_eq!(session.draft, baseline);
    assert_eq!(session.dirty, dirty);
    accepted(&send(&mut session, P::from_avro(&correction.to_avro()).unwrap()));
    assert_eq!(session.draft, baseline);
    let result = supported(&session.draft);
    assert_eq!(return_line(&result, "5a"), 1_200_000);
    assert_eq!(return_line(&result, "5b"), 800_000);
    assert_eq!(return_line(&result, "25b"), 100_000);
    assert_eq!(value(&result, "agi"), 68_000);
    let reopened = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &reopened else { unreachable!() };
    assert_eq!(
        documents.common.retirement.documents.iter().map(|item| item.id).collect::<Vec<_>>(),
        source_ids
    );
    assert_eq!(
        IncomeSource::Retirement {
            value: documents.common.retirement.documents[1].document.clone()
        },
        expected_named_sources(
            &session.draft,
            Owner::Taxpayer,
            vec![IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(gap_plan()))
            }]
        )[0]
    );
    accepted(&send(&mut session, P::from_avro(&reopened.to_avro()).unwrap()));
    assert_eq!(session.draft, baseline);
    let mut restored = restart(baseline.clone());
    let reopened = open_documents(&mut restored);
    accepted(&send(&mut restored, P::from_avro(&reopened.to_avro()).unwrap()));
    assert_eq!(restored.draft, baseline);
    assert_eq!(supported(&restored.draft), result);
}

fn gap_reservist(distributed: u32) -> ReservistRepayment {
    ReservistRepayment {
        ordered: 20220101,
        term: ReserveTerm::Indefinite,
        released: 20240101,
        distributed,
        gross: 1_200_000,
        prior: 0,
        payments: vec![
            ReservistPayment { date: 20260101, amount: 100_000 },
            ReservistPayment { date: 20260101, amount: 200_000 },
        ],
    }
}

fn gap_reservist_fixture() -> Interview {
    // Same genuine two-distribution/two-payment anniversary fixture as the
    // existing core reservist controls; total basis $6,000 is not a deduction.
    let form = Form8606 {
        basis: 0,
        adjustment: 0,
        year_end: 5_000_000,
        reservist_repayments: vec![gap_reservist(20230101), gap_reservist(20230102)],
    };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_documents(
        &mut session,
        vec![
            document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(r1099(
                        200_000,
                        normal_kept(),
                        false,
                    ))),
                },
            ),
            document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(r1099(
                        1_000_000,
                        normal_kept(),
                        true,
                    ))),
                },
            ),
        ],
    );
    let desired = crate::tests::support::client::income_pages::page_documents(&request);
    accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
    crate::tests::support::client::income_pages::save_information_return_answers(
        &mut session,
        &desired,
    );
    let request = save_topic(
        &mut session,
        TaxSection::Retirement,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Form8606(form) })],
    );
    accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "reservist_taxpayer_2_basis"), 3_000);
    assert_eq!(value(&result, "reservist_taxpayer_2_unrepaid"), 9_000);
    assert_eq!(return_line(&result, "4b"), 900_000);
    assert_eq!(return_line(&result, "5b"), 200_000);
    assert_eq!(value(&result, "agi"), 71_000);
    session
}

#[test]
fn unit47_reservist_missing_native_keys_locate_second_original_distribution() {
    for duplicate in [true, false] {
        let session = gap_reservist_fixture();
        gap_reservist_case(duplicate, session);
    }
}

#[inline(never)]
fn gap_reservist_case(duplicate: bool, mut session: Interview) {
    let baseline = session.draft.clone();
    let mut request = open_topic(&mut session, TaxSection::Retirement);
    let correction = request.clone();
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let form_index = documents
        .documents
        .iter()
        .position(|item| matches!(item.document, crate::IncomeRecord::Form8606(_)))
        .unwrap();
    assert_eq!(form_index, 0);
    let crate::IncomeRecord::Form8606(form) = &mut documents.documents[form_index].document else {
        unreachable!()
    };
    form.reservist_repayments[1].distributed = if duplicate { 20230101 } else { 20240102 };
    let key = if duplicate {
        "RESERVIST_DISTRIBUTION_TWICE"
    } else {
        "RESERVIST_DISTRIBUTION_OUTSIDE_DUTY"
    };
    let case = if duplicate { "duplicate" } else { "outside-duty" };
    let path = "[documents/documents/0/document/reservist_repayments/1/distributed]";
    let calculation = credit_estimate_documents_from_save(&request);
    gap_refuse(
        &mut session,
        request,
        &alloc::format!("retirement-gap-reservist-{case}-save"),
        key,
        path,
    );
    gap_refuse(
        &mut session,
        calculation,
        &alloc::format!("retirement-gap-reservist-{case}-estimate"),
        key,
        path,
    );
    let dirty = session.dirty;
    accepted(&crate::tests::support::client::income_pages::candidate_save_review(
        &mut session,
        P::from_avro(&credit_estimate_documents_from_save(&correction).to_avro()).unwrap(),
    ));
    assert_eq!(session.draft, baseline);
    assert_eq!(session.dirty, dirty);
    accepted(&send(&mut session, P::from_avro(&correction.to_avro()).unwrap()));
    assert_eq!(session.draft, baseline);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "reservist_taxpayer_2_basis"), 3_000);
    assert_eq!(value(&result, "reservist_taxpayer_2_unrepaid"), 9_000);
    assert_eq!(return_line(&result, "4b"), 900_000);
    assert_eq!(value(&result, "agi"), 71_000);
    let reopened = open_topic(&mut session, TaxSection::Retirement);
    accepted(&send(&mut session, P::from_avro(&reopened.to_avro()).unwrap()));
    assert_eq!(session.draft, baseline);
    let mut restored = restart(baseline.clone());
    let reopened = open_topic(&mut restored, TaxSection::Retirement);
    accepted(&send(&mut restored, P::from_avro(&reopened.to_avro()).unwrap()));
    assert_eq!(restored.draft, baseline);
    assert_eq!(supported(&restored.draft), result);
}
