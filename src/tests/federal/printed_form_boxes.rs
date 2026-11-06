use super::*;
use crate::tests::cross::interview::{document, normal_kept, r1099};
use crate::tests::support::client::income_pages::{
    open_document_center, open_topic, save_documents,
};

#[test]
fn printed_contributions_do_not_reduce_an_already_determined_taxable_distribution_twice() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut form = r1099(1_000_000, normal_kept(), false).into_source().form;
    form.taxable = Some(850_000);
    form.employee_contributions = Some(150_000);
    form.total_distribution = true;
    form.fatca = true;
    let expected = form.form_boxes();
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::Retirement {
                value: RetirementDocument::R1099(crate::Form1099RPaper::from(&form)),
            },
        )],
    );
    accepted(&send(&mut session, request));
    accepted(&answer_only_retirement_treatment(&mut session, normal_kept()));
    assert_eq!(value(&supported(&session.draft), "agi"), 68_500);
    let mut session = restart(session.draft);
    let P::SaveDocuments { documents, .. } = open_document_center(&mut session) else {
        unreachable!()
    };
    let RetirementDocument::R1099(form) = &documents.common.retirement.documents[0].document else {
        unreachable!()
    };
    assert_eq!(R1099::from(form).form_boxes(), expected);
    assert_eq!(form.taxable, Some(850_000));
}

#[test]
fn social_printed_premiums_reach_medical_and_correct_after_save_and_restart() {
    let mut draft = complete(FederalStatus::Single);
    elect_itemized(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let form = Ssa1099 {
        benefits_paid: 2_000_000,
        paid_description: vec![SocialBenefitDescription {
            description: SocialBenefitDescriptionKind::MedicarePartB,
            amount: 240_000,
        }],
        benefits_repaid: 200_000,
        repaid_description: vec![],
        benefits: 1_800_000,
        withholding: 0,
        ..Default::default()
    };
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::Retirement { value: RetirementDocument::Ssa1099(ssa_document(form)) },
        )],
    );
    accepted(&send(&mut session, request));
    assert_eq!(value(&supported(&session.draft), "medical_expenses"), 2_400);
    let mut session = restart(session.draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let RetirementDocument::Ssa1099(form) = &mut documents.common.retirement.documents[0].document
    else {
        unreachable!()
    };
    assert_eq!(
        form.benefits_paid.unwrap_or(0) - form.benefits_repaid_to_ssa.unwrap_or(0),
        form.net_benefits
    );
    form.description_of_amount_in_box_3[0].amount = 250_000;
    accepted(&send(&mut session, request));
    let session = restart(session.draft);
    assert_eq!(value(&supported(&session.draft), "medical_expenses"), 2_500);
}

#[test]
fn printed_medicare_allocated_to_a_health_plan_enters_medical_once_even_when_se_is_disallowed() {
    let mut draft = complete(FederalStatus::Single);
    elect_itemized(&mut draft);
    add(&mut draft, income(health_business(10_000, 1_800, true)));
    let plan_id = draft.income.last().unwrap().id;
    let mut source = social_security_net(2_000_000, 0);
    let FederalIncome::SocialSecurity { data: SocialBenefitStatement::Ssa1099(data) } = &mut source
    else {
        unreachable!()
    };
    data.benefits_paid = Some(2_000_000);
    data.description_of_amount_in_box_3.push(SocialBenefitDescription {
        description: SocialBenefitDescriptionKind::MedicarePartB,
        amount: 240_000,
    });
    add(&mut draft, income(source));
    let source_id = draft.income.last().unwrap().id;
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_topic(&mut session, TaxSection::Retirement);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    let question = followups
        .iter_mut()
        .find(|row| row.key == alloc::format!("income/{source_id}/social_premiums"))
        .unwrap();
    assert_eq!(question.health_plans.iter().map(|plan| plan.id).collect::<Vec<_>>(), vec![plan_id]);
    question.answer = DocumentAnswer::SocialPremiums {
        value: Some(SocialPremiumUsePage::AlreadyInSelfEmployedPlan {
            income_id: plan_id,
            amount: 180_000,
        }),
    };
    accepted(&send(&mut session, request));
    let mut session = restart(session.draft);
    let result = supported(&session.draft);
    // Employer eligibility prevents the SE deduction. Its health owner returns
    // the included $1,800 to medical; the statement adds only the other $600.
    assert_eq!(value(&result, "medical_expenses"), 2_400);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.common.retirement.documents.clear();
    accepted(&send(&mut session, request));
    assert!(session.draft.adjustments.as_ref().unwrap().social_premiums.is_empty());
    assert_eq!(value(&supported(&session.draft), "medical_expenses"), 1_800);
}

#[test]
fn linked_printed_tips_and_overtime_correct_without_adding_receipts_twice_and_explicit_no_stays_no()
{
    let mut draft = complete(FederalStatus::Single);
    let mut activity = cash_business(0);
    let FederalIncome::Business { data } = &mut activity else { unreachable!() };
    data.tips = Some(FederalBusinessTips { amount: 0, eligible: true });
    data.overtime = Some(FederalBusinessOvertime { amount: 0, eligible: true });
    add(&mut draft, income(activity));
    let business_id = draft.income.last().unwrap().id;
    add(
        &mut draft,
        income(FederalIncome::Nec1099 {
            data: nec_source(FederalForm1099Nec {
                compensation: 1_000_000,
                cash_tips: 100_000,
                tip_occupation_codes: vec![TippedOccupationCode { code: 101 }],
                overtime: 50_000,
                golden_parachute: 0,
                withholding: 0,
                belongs: None,
            }),
        }),
    );
    let source_id = draft.income.last().unwrap().id;
    let names = fixture_document_names(&draft, -1);
    draft.income.last_mut().unwrap().printed = names;
    let FederalIncome::Business { data } =
        &mut draft.income.iter_mut().find(|entry| entry.id == business_id).unwrap().income
    else {
        unreachable!()
    };
    data.related = vec![source_id];
    let mut session = Interview::new(TEST_USER, draft);
    let first = supported(&session.draft);
    assert_eq!(value(&first, "tips_deduction"), 1_000);
    assert_eq!(value(&first, "overtime_deduction"), 500);
    assert_eq!(value(&first, &alloc::format!("business_{business_id}_net_profit")), 10_000);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let SelfEmploymentDocument::Form1099Nec(form) =
        &mut documents.common.self_employment.documents[0].document
    else {
        unreachable!()
    };
    form.cash_tips = Some(120_000);
    form.overtime = Some(60_000);
    accepted(&send(&mut session, request));
    let mut session = restart(session.draft);
    let corrected = supported(&session.draft);
    assert_eq!(value(&corrected, "tips_deduction"), 1_200);
    assert_eq!(value(&corrected, "overtime_deduction"), 600);
    assert_eq!(value(&corrected, &alloc::format!("business_{business_id}_net_profit")), 10_000);
    let mut request = open_topic(&mut session, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let IncomeRecord::Business(business) =
        &mut documents.documents.iter_mut().find(|row| row.id == business_id).unwrap().document
    else {
        unreachable!()
    };
    let BusinessTipsEntry::Printed { data: tips } = &mut business.more.tips else {
        panic!("linked tips retain their printed amount");
    };
    assert_eq!(tips.amount, BusinessTipsAmount::Printed { amount: 120_000 });
    tips.eligible = Some(false);
    let BusinessOvertimeEntry::Printed { data: overtime } = &mut business.more.overtime else {
        panic!("linked overtime retains its printed amount");
    };
    assert_eq!(overtime.amount, BusinessOvertimeAmount::Printed { amount: 60_000 });
    overtime.eligible = Some(false);
    accepted(&send(&mut session, request));
    let session = restart(session.draft);
    let refused = supported(&session.draft);
    assert_eq!(value(&refused, "tips_deduction"), 0);
    assert_eq!(value(&refused, "overtime_deduction"), 0);
}

#[test]
fn vehicle_refunds_share_the_prior_year_cap_and_leave_current_interest_intact() {
    let mut draft = complete(FederalStatus::Single);
    let prior = VehicleRecoveryYear {
        year: 2025,
        status: FederalStatus::Single,
        magi: 6_000_000,
        eligible_interest: 1_200_000,
        taxable_income: 5_000_000,
        ordinary: true,
    };
    let deductions = draft.deductions.as_mut().unwrap();
    let mut first = vehicle_loan("First lender", 200_000);
    first.id = 42;
    first.document.box_5 = Some(100_000);
    let mut second = vehicle_loan("Second lender", 200_000);
    second.id = 43;
    second.document.box_5 = Some(200_000);
    deductions.vehicle_loans = vec![first, second];
    deductions.vehicle_recoveries = [42, 43]
        .into_iter()
        .map(|id| VehicleInterestRecovery {
            id,
            origin: VehicleRefundOrigin::Deducted { prior: prior.clone() },
        })
        .collect();
    // $12,000 less $3,000 refunds changes the capped deduction from $10,000
    // to $9,000. Comparing each refund separately would miss this recovery.
    draft.next_id = 44;
    let result = supported(&draft);
    assert_eq!(value(&result, "vehicle_interest_recovery_2025"), 1_000);
    assert_eq!(value(&result, "vehicle_interest_deduction"), 4_000);
    assert_eq!(value(&result, "agi"), 61_000);
    for row in &mut draft.deductions.as_mut().unwrap().vehicle_recoveries {
        row.origin = VehicleRefundOrigin::NotDeducted;
    }
    let result = supported(&restart(draft).draft);
    assert_eq!(value(&result, "vehicle_interest_deduction"), 4_000);
    assert_eq!(value(&result, "agi"), 60_000);
}

#[test]
fn printed_prior_year_receipts_refigure_the_existing_election_and_prune_removed_years() {
    let mut draft = complete(FederalStatus::Single);
    let mut source = social_security_net(1_200_000, 0);
    let FederalIncome::SocialSecurity { data: SocialBenefitStatement::Ssa1099(data) } = &mut source
    else {
        unreachable!()
    };
    data.benefits_paid = Some(1_200_000);
    data.description_of_amount_in_box_3.push(SocialBenefitDescription {
        description: SocialBenefitDescriptionKind::PriorYear { year: 2025 },
        amount: 200_000,
    });
    add(&mut draft, income(source));
    let names = fixture_document_names(&draft, -1);
    draft.income.last_mut().unwrap().printed = names;
    draft.adjustments.as_mut().unwrap().social_election =
        SocialElection::Compare { years: vec![social_prior_year()], complete: true };
    let mut session = restart(draft);
    // The retained 2025 AGI is $23,000 with no prior benefits. The $2,000
    // receipt is untaxed for 2025, leaving $10,000 x 85% taxable in 2026.
    assert_eq!(return_line(&supported(&session.draft), "6b") / 100, 8_500);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let RetirementDocument::Ssa1099(form) = &mut documents.common.retirement.documents[0].document
    else {
        unreachable!()
    };
    form.description_of_amount_in_box_3[0].amount = 400_000;
    accepted(&send(&mut session, request));
    let mut session = restart(session.draft);
    // Correcting the allocation leaves $8,000 current benefits, without
    // asking the person to repeat either the receipt or the prior-return facts.
    assert_eq!(return_line(&supported(&session.draft), "6b") / 100, 6_800);
    let SocialElection::Compare { years, .. } =
        &session.draft.adjustments.as_ref().unwrap().social_election
    else {
        unreachable!()
    };
    assert_eq!(years[0].agi, 2_300_000);
    assert_eq!(years[0].received, 400_000);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let RetirementDocument::Ssa1099(form) = &mut documents.common.retirement.documents[0].document
    else {
        unreachable!()
    };
    form.description_of_amount_in_box_3.clear();
    accepted(&send(&mut session, request));
    let session = restart(session.draft);
    assert_eq!(
        session.draft.adjustments.as_ref().unwrap().social_election,
        SocialElection::Regular
    );
    assert_eq!(return_line(&supported(&session.draft), "6b") / 100, 10_200);
}

#[test]
fn railroad_box_twelve_reaches_medical_without_changing_the_pension_taxable_amount() {
    let mut draft = complete(FederalStatus::Single);
    elect_itemized(&mut draft);
    draft.people.as_mut().unwrap().taxpayer.birth_date = 19600101;
    let mut session = Interview::new(TEST_USER, draft);
    let form = Rrb1099R {
        issuer: Default::default(),
        claim_number_and_payee_code: alloc::string::String::new(),
        recipient: Default::default(),
        box_3: None,
        box_4: Some(1_000_000),
        box_5: Some(0),
        box_6: Some(0),
        gross: 1_000_000,
        box_8: Some(0),
        withholding: Some(0),
        medicare_premiums: Some(123_400),
    };
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::Retirement {
                value: RetirementDocument::Rrb1099R(crate::FormRrb1099RPaper::from(&form)),
            },
        )],
    );
    accepted(&send(&mut session, request));
    let mut request = open_topic(&mut session, TaxSection::Retirement);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    let question = followups
        .iter_mut()
        .find(|row| matches!(row.answer, DocumentAnswer::RailroadRecipient { .. }))
        .unwrap();
    question.answer = DocumentAnswer::RailroadRecipient { value: Some(RailroadRecipient::Spouse) };
    accepted(&send(&mut session, request));
    let mut session = restart(session.draft);
    let result = supported(&session.draft);
    assert_eq!(return_line(&result, "5b") / 100, 10_000);
    assert_eq!(value(&result, "medical_expenses"), 1_234);
    let P::SaveDocuments { documents, .. } = open_document_center(&mut session) else {
        unreachable!()
    };
    let RetirementDocument::Rrb1099R(form) = &documents.common.retirement.documents[0].document
    else {
        unreachable!()
    };
    assert_eq!(form.medicare_premiums, Some(123_400));
}

#[test]
fn trump_printed_excess_earnings_feed_the_existing_additional_tax_and_follow_corrections() {
    use crate::tests::cross::interview::open_other_taxes;
    let mut draft = complete(FederalStatus::Single);
    draft.people.as_mut().unwrap().taxpayer.birth_date = 20100301;
    draft.screening.as_mut().unwrap().trump_accounts = vec![TrumpAccount {
        owner: Owner::Taxpayer,
        contributions: 600_000,
        exempt: 0,
        returns: vec![],
        confirmed: true,
    }];
    let mut session = Interview::new(TEST_USER, draft);
    let mut form = r1099(100_000, DistributionCode::Corrective { excess_deferral: 0 }, false)
        .into_source()
        .form;
    form.trump = true;
    form.taxable = Some(0);
    form.excess_contribution_earnings = Some(9_000);
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::Retirement {
                value: RetirementDocument::R1099(crate::Form1099RPaper::from(&form)),
            },
        )],
    );
    accepted(&send(&mut session, request));
    let mut request = open_other_taxes(&mut session);
    let P::SaveOtherTaxes { value: taxes, .. } = &mut request else { unreachable!() };
    assert_eq!(taxes.trump_returns.len(), 1);
    let row = &mut taxes.trump_returns[0];
    assert_eq!((row.principal, row.earnings), (100_000, 9_000));
    row.received = Some(20261201);
    row.deadline = Some(20270415);
    row.confirmed = true;
    accepted(&send(&mut session, request));
    let mut session = restart(session.draft);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "trump_account_taxpayer_line61_tax"), 0);
    assert_eq!(value(&result, "trump_account_taxpayer_line63_tax"), 90);
    assert_eq!(value(&result, "agi"), 60_000);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let RetirementDocument::R1099(form) = &mut documents.common.retirement.documents[0].document
    else {
        unreachable!()
    };
    form.excess_contribution_earnings = Some(12_000);
    accepted(&send(&mut session, request));
    let session = restart(session.draft);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "trump_account_taxpayer_line63_tax"), 120);
    let account = &session.draft.screening.as_ref().unwrap().trump_accounts[0];
    assert_eq!(account.returns[0].received, 20261201);
}

#[test]
fn undetermined_designated_roth_uses_the_printed_clock_and_then_the_existing_basis_owner() {
    let mut draft = complete(FederalStatus::Single);
    draft.people.as_mut().unwrap().taxpayer.birth_date = 19600101;
    let mut session = Interview::new(TEST_USER, draft);
    let mut form = r1099(1_000_000, normal_kept(), false).into_source().form;
    form.box7 |= DistributionCodes::CODE_B.0;
    form.taxable = Some(0);
    form.taxable_not_determined = true;
    form.first_roth_year = 2021;
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::Retirement {
                value: RetirementDocument::R1099(crate::Form1099RPaper::from(&form)),
            },
        )],
    );
    accepted(&send(&mut session, request));
    accepted(&answer_only_retirement_treatment(&mut session, normal_kept()));
    assert_eq!(return_line(&supported(&session.draft), "5b") / 100, 0);
    let mut session = restart(session.draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let RetirementDocument::R1099(form) = &mut documents.common.retirement.documents[0].document
    else {
        unreachable!()
    };
    form.first_roth_year = Some(2024);
    accepted(&send(&mut session, request));
    let mut request = open_topic(&mut session, TaxSection::Retirement);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    let method = followups
        .iter_mut()
        .find(|row| matches!(row.answer, DocumentAnswer::RetirementMethod { .. }))
        .unwrap();
    method.answer = DocumentAnswer::RetirementMethod {
        method: Some(TaxableMethod::PlanWithdrawal {
            data: PlanDistribution { value: 2_000_000, investment: 1_000_000, eligible: true },
        }),
    };
    accepted(&send(&mut session, request));
    let session = restart(session.draft);
    // A nonqualified $10,000 payment from a $20,000 account with $10,000
    // unrecovered investment is half taxable, using the existing plan owner.
    assert_eq!(return_line(&supported(&session.draft), "5b") / 100, 5_000);
}

#[test]
fn undetermined_ira_with_blank_two_a_uses_gross_until_the_existing_basis_worksheet_applies() {
    let mut draft = complete(FederalStatus::Single);
    draft.people.as_mut().unwrap().taxpayer.birth_date = 19600101;
    let mut session = Interview::new(TEST_USER, draft);
    let mut form = r1099(1_000_000, normal_kept(), true).into_source().form;
    form.taxable = Some(0);
    form.taxable_not_determined = true;
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::Retirement {
                value: RetirementDocument::R1099(crate::Form1099RPaper::from(&form)),
            },
        )],
    );
    accepted(&send(&mut session, request));
    accepted(&answer_only_retirement_treatment(&mut session, normal_kept()));
    let session = restart(session.draft);
    let result = supported(&session.draft);
    assert_eq!(return_line(&result, "4b") / 100, 10_000);
    let source = session
        .draft
        .income
        .iter()
        .find_map(|entry| match &entry.income {
            FederalIncome::Retirement { data } => Some(data),
            _ => None,
        })
        .unwrap();
    let source = FederalRetirement::from(source);
    assert_eq!(source.box_2a, 0);
    assert!(source.form_boxes.taxable_not_determined);
}

#[test]
fn medicare_reimbursed_from_an_hsa_is_excluded_once_and_source_removal_rederives_medical() {
    let mut draft = complete(FederalStatus::Single);
    elect_itemized(&mut draft);
    draft.people.as_mut().unwrap().taxpayer.birth_date = 19600101;
    add_hsa_distribution(&mut draft, Owner::Taxpayer, 60_000, vec![], None, None);
    let mut source = social_security_net(2_000_000, 0);
    let FederalIncome::SocialSecurity { data: SocialBenefitStatement::Ssa1099(data) } = &mut source
    else {
        unreachable!()
    };
    data.description_of_amount_in_box_3.push(SocialBenefitDescription {
        description: SocialBenefitDescriptionKind::MedicarePartB,
        amount: 240_000,
    });
    add(&mut draft, income(source));
    let names = fixture_document_names(&draft, -1);
    draft.income.last_mut().unwrap().printed = names;
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_topic(&mut session, TaxSection::Retirement);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    let question = followups
        .iter_mut()
        .find(|row| matches!(row.answer, DocumentAnswer::SocialReimbursedPremiums { .. }))
        .unwrap();
    question.answer = DocumentAnswer::SocialReimbursedPremiums { reimbursed: Some(60_000) };
    accepted(&send(&mut session, request));
    let mut session = restart(session.draft);
    assert_eq!(value(&supported(&session.draft), "medical_expenses"), 1_800);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.other.other_income.documents.clear();
    accepted(&send(&mut session, request));
    let session = restart(session.draft);
    assert!(session.draft.adjustments.as_ref().unwrap().social_premiums.is_empty());
    assert_eq!(value(&supported(&session.draft), "medical_expenses"), 2_400);
}
