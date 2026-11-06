use super::*;
use crate::tests::farm::farm::schedule_f_lines;
use crate::tests::support::builders::native_income::document;
use crate::tests::support::client::income_pages::open_documents;

// Every printed box and topic answer of a stored Form 1099-G lands once in the
// internal unemployment model; the party and state blocks reach no federal figure.
#[test]
fn form_1099_g_converter_carries_every_box_into_the_internal_model() {
    let ids = GovernmentComponentIds { grant: 7, agriculture: 8, ..Default::default() };
    let repayment = UnemploymentRepayment { received_year: 2025, amount: 900 };
    let source = GovernmentPaymentSource {
        form: Form1099GPaper {
            unemployment_compensation: 100,
            state_or_local_income_tax_refunds: 800,
            box_2_tax_year: Some(2025),
            federal_income_tax_withheld: 200,
            rtaa_payments: Some(300),
            taxable_grants: Some(400),
            agriculture_payments: Some(500),
            market_gain: Some(600),
            family_leave_benefits: 700,
            ..Default::default()
        },
        component_ids: ids.clone(),
        grant_use: Some(GrantUse::You),
        family_leave_medical: true,
        repayments: vec![repayment.clone()],
        special: true,
        history: refund_history(),
        refund_state: None,
        refund_locality: Default::default(),
    };
    assert_eq!(
        FederalUnemployment::from(&source),
        FederalUnemployment {
            component_ids: ids,
            amount: 100,
            withholding: 200,
            repayments: vec![repayment],
            ordinary: false,
            taxable_grants: 400,
            agriculture_payment: 500,
            grant_use: Some(FederalGrantUse::You),
            rtaa: 300,
            market_gain: 600,
            family_leave: 700,
            family_leave_includes_medical: true,
        }
    );
    assert_eq!(
        FederalTaxRefund::from(&source),
        FederalTaxRefund { amount: 800, withholding: 0, history: refund_history() }
    );
}

#[test]
fn refund_box_8_suggests_a_visible_answer_only_after_paper_save() {
    use crate::tests::support::client::income_pages::{open_topic, save_documents};
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let original = session.draft.clone();
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::GovernmentPayment {
                refund_state: None,
                refund_locality: Default::default(),
                value: Form1099GPaper {
                    state_or_local_income_tax_refunds: 12_000,
                    box_2_tax_year: Some(2025),
                    federal_income_tax_withheld: 800,
                    trade_or_business_income: true,
                    ..Default::default()
                },
            },
        )],
    );
    assert_eq!(session.draft, original);
    accepted(&send(&mut session, request));
    let entry = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::GovernmentPayment { .. }))
        .unwrap();
    let id = entry.id;
    let FederalIncome::GovernmentPayment { data } = &entry.income else { unreachable!() };
    assert!(data.form.trade_or_business_income);
    assert_eq!(data.history, TaxRefundHistory::Unanswered);
    let before_topic = session.draft.clone();
    let result = estimate(&session.draft);
    assert_eq!(result.payment_withholding.as_ref().unwrap().forms_1099, 800);
    assert!(result.issues.iter().any(|issue| issue.display.key == "REFUND_DEDUCTION_NEEDED"));
    let P::SaveIncomes { followups, .. } = open_topic(&mut session, TaxSection::GovernmentBenefits)
    else {
        unreachable!()
    };
    assert_eq!(followups.len(), 1);
    assert_eq!(followups[0].key, alloc::format!("income/{id}/deduction"));
    assert_eq!(
        followups[0].answer,
        DocumentAnswer::RefundDeduction {
            value: Some(RefundDeduction::BusinessExpense { business: None }),
        }
    );
    assert_eq!(session.draft, before_topic);
    let mut unanswered = open_topic(&mut session, TaxSection::GovernmentBenefits);
    let P::SaveIncomes { followups: cleared, .. } = &mut unanswered else { unreachable!() };
    cleared[0].answer = DocumentAnswer::RefundDeduction { value: None };
    rejected_with(send(&mut session, unanswered), "REFUND_DEDUCTION_NEEDED");
    assert_eq!(session.draft, before_topic);
    let mut cold = restart(session.draft.clone());
    let P::SaveIncomes { followups: reopened, .. } =
        open_topic(&mut cold, TaxSection::GovernmentBenefits)
    else {
        unreachable!()
    };
    assert_eq!(reopened, followups);
}

#[test]
fn explicit_refund_deduction_survives_box_8_correction_and_restart() {
    use crate::tests::support::client::income_pages::{open_topic, save_documents};
    for checked in [false, true] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::GovernmentPayment {
                    refund_state: None,
                    refund_locality: Default::default(),
                    value: Form1099GPaper {
                        state_or_local_income_tax_refunds: 12_000,
                        box_2_tax_year: Some(2025),
                        trade_or_business_income: checked,
                        ..Default::default()
                    },
                },
            )],
        );
        accepted(&send(&mut session, request));
        let mut topic = open_topic(&mut session, TaxSection::GovernmentBenefits);
        let P::SaveIncomes { followups, .. } = &mut topic else { unreachable!() };
        assert_eq!(followups.len(), 1);
        if !checked {
            assert_eq!(followups[0].answer, DocumentAnswer::RefundDeduction { value: None });
        }
        followups[0].answer =
            DocumentAnswer::RefundDeduction { value: Some(RefundDeduction::StandardDeduction) };
        let accepted_row = followups[0].clone();
        accepted(&send(&mut session, topic));
        let mut correction = open_documents(&mut session);
        let P::SaveDocuments { documents, .. } = &mut correction else { unreachable!() };
        let paper = &mut documents.common.government_payments.documents[0].document;
        paper.trade_or_business_income = !checked;
        paper.state_or_local_income_tax_refunds = 15_000;
        accepted(&send(&mut session, correction));
        let FederalIncome::GovernmentPayment { data } = &session
            .draft
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::GovernmentPayment { .. }))
            .unwrap()
            .income
        else {
            unreachable!()
        };
        assert_eq!(data.form.state_or_local_income_tax_refunds, 15_000);
        assert_eq!(data.form.trade_or_business_income, !checked);
        assert_eq!(data.history, TaxRefundHistory::NotDeducted);
        let P::SaveIncomes { followups, .. } =
            open_topic(&mut session, TaxSection::GovernmentBenefits)
        else {
            unreachable!()
        };
        assert_eq!(followups, vec![accepted_row.clone()]);
        let mut cold = restart(session.draft.clone());
        let P::SaveIncomes { followups, .. } =
            open_topic(&mut cold, TaxSection::GovernmentBenefits)
        else {
            unreachable!()
        };
        assert_eq!(followups, vec![accepted_row]);
    }
}

#[test]
fn refund_jurisdiction_survives_save_correction_reopen_and_restart() {
    use crate::tests::support::client::income_pages::{open_topic, save_documents};
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::GovernmentPayment {
                refund_state: Some(TaxState::CA),
                refund_locality: "Example California City".into(),
                value: Form1099GPaper {
                    payer: PrintedParty {
                        name: "Example California Refund Agency".into(),
                        address: "1 Example Street, Sacramento, CA".into(),
                        ..Default::default()
                    },
                    state_or_local_income_tax_refunds: 12_000,
                    box_2_tax_year: Some(2025),
                    federal_income_tax_withheld: 800,
                    ..Default::default()
                },
            },
        )],
    );
    accepted(&send(&mut session, request));
    let mut topic = open_topic(&mut session, TaxSection::GovernmentBenefits);
    let P::SaveIncomes { followups, .. } = &mut topic else { unreachable!() };
    assert_eq!(followups.len(), 1);
    followups[0].answer =
        DocumentAnswer::RefundDeduction { value: Some(RefundDeduction::StandardDeduction) };
    accepted(&send(&mut session, topic));

    for (state, locality) in [
        (Some(TaxState::CA), "Example California City"),
        (Some(TaxState::NY), "Example New York City"),
        (None, ""),
    ] {
        let mut correction = open_documents(&mut session);
        let P::SaveDocuments { documents, .. } = &mut correction else { unreachable!() };
        let row = &mut documents.common.government_payments.documents[0];
        row.document.payer.name = match state {
            Some(TaxState::CA) => "Example California Refund Agency",
            Some(TaxState::NY) => "Example New York Refund Agency",
            _ => "Example Refund Office",
        }
        .into();
        row.refund_state = state;
        row.refund_locality = locality.into();
        accepted(&send(&mut session, correction));
        let entry = session
            .draft
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::GovernmentPayment { .. }))
            .unwrap();
        let FederalIncome::GovernmentPayment { data } = &entry.income else { unreachable!() };
        let expected = RefundStateFields {
            refund_state: state,
            refund_locality: locality.into(),
            taxes: Default::default(),
        };
        assert_eq!(data.refund_state, state);
        assert_eq!(data.refund_locality, locality);
        assert_eq!(data.history, TaxRefundHistory::NotDeducted);
        assert_eq!(
            Refund1099G::with_place(&data.form, data.refund_state, &data.refund_locality)
                .state_details,
            expected
        );
        assert_eq!(
            entry.state_source,
            (state.is_some() || !locality.is_empty())
                .then_some(IncomeStateSource::Refund(expected))
        );
        let result = estimate(&session.draft);
        assert_eq!(result.payment_withholding.as_ref().unwrap().forms_1099, 800);
        session = restart(session.draft.clone());
        let P::SaveDocuments { documents, .. } = open_documents(&mut session) else {
            unreachable!()
        };
        let row = &documents.common.government_payments.documents[0];
        assert_eq!(row.refund_state, state);
        assert_eq!(row.refund_locality, locality);
        assert!(row.document.state_details.states.is_empty());
    }
}

#[test]
fn one_mixed_government_paper_has_independent_unemployment_and_refund_answers() {
    use crate::tests::support::client::income_pages::{open_topic, save_documents};
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::GovernmentPayment {
                refund_state: None,
                refund_locality: Default::default(),
                value: mixed_government_paper(),
            },
        )],
    );
    accepted(&send(&mut session, request));
    let (entry, source) = accepted_government_source(&session);
    let id = entry.id;
    assert_eq!(entry.owner, DocumentOwner::Yours);
    assert!(source.repayments.is_empty());
    assert_eq!(source.history, TaxRefundHistory::Unanswered);
    let P::SaveIncomes { followups, .. } = open_topic(&mut session, TaxSection::GovernmentBenefits)
    else {
        unreachable!()
    };
    assert!(followups.iter().any(|row| row.key == alloc::format!("income/{id}/repayments")));
    assert!(followups.iter().any(|row| row.key == alloc::format!("income/{id}/deduction")));

    // Publication 525 limits the $3,000 refund to the $1,000 benefit from
    // itemizing ($16,750 less $15,750). The $300 repayment reduces only the
    // $2,000 unemployment payment; the form's $500 withholding is counted once.
    for (repayment, deduction, unemployment, taxable_refund, agi) in [
        (30_000, government_refund_deduction(), 1_700, 1_000, 62_700),
        (50_000, government_refund_deduction(), 1_500, 1_000, 62_500),
        (50_000, RefundDeduction::StandardDeduction, 1_500, 0, 61_500),
    ] {
        let request = government_answer_request(&mut session, Some(repayment), Some(deduction));
        accepted(&send(&mut session, request));
        let result = supported(&session.draft);
        assert_eq!(value(&result, &alloc::format!("unemployment_{id}_taxable")), unemployment);
        assert_eq!(value(&result, "taxable_state_tax_refund"), taxable_refund);
        assert_eq!(value(&result, "agi"), agi);
        assert_eq!(return_line(&result, "8"), (unemployment + taxable_refund) * 100);
        assert_eq!(result.payment_withholding.as_ref().unwrap().forms_1099, 50_000);
        assert_eq!(return_line(&result, "25a"), 650_000);
        assert_eq!(return_line(&result, "25b"), 50_000);
        assert_eq!(return_line(&result, "25d"), 700_000);
    }
    let P::SaveDocuments { documents, .. } = open_documents(&mut session) else { unreachable!() };
    assert_eq!(documents.common.government_payments.documents.len(), 1);
    assert_eq!(documents.common.government_payments.documents[0].id, id);
}

#[test]
fn pure_government_papers_offer_only_their_own_answer_family() {
    use crate::tests::support::client::income_pages::{open_topic, save_documents};
    for refund_only in [false, true] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut form = mixed_government_paper();
        if refund_only {
            form.unemployment_compensation = 0;
        } else {
            form.state_or_local_income_tax_refunds = 0;
        }
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::GovernmentPayment {
                    refund_state: None,
                    refund_locality: Default::default(),
                    value: form,
                },
            )],
        );
        accepted(&send(&mut session, request));
        let P::SaveIncomes { followups, .. } =
            open_topic(&mut session, TaxSection::GovernmentBenefits)
        else {
            unreachable!()
        };
        assert_eq!(
            followups
                .iter()
                .any(|row| matches!(row.answer, DocumentAnswer::RefundDeduction { .. })),
            refund_only
        );
        assert_eq!(
            followups
                .iter()
                .any(|row| matches!(row.answer, DocumentAnswer::GovernmentRepayments { .. })),
            !refund_only
        );
        let request = government_answer_request(
            &mut session,
            (!refund_only).then_some(30_000),
            refund_only.then(government_refund_deduction),
        );
        accepted(&send(&mut session, request));
        let result = supported(&session.draft);
        assert_eq!(value(&result, "taxable_state_tax_refund"), if refund_only { 1_000 } else { 0 });
        assert_eq!(value(&result, "agi"), if refund_only { 61_000 } else { 61_700 });
        assert_eq!(return_line(&result, "25b"), 50_000);
        assert_eq!(supported(&restart(session.draft).draft), result);
    }
}

#[test]
fn mixed_government_paper_correction_and_removal_keep_one_native_source() {
    use crate::tests::support::client::income_pages::{open_topic, save_documents};
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::GovernmentPayment {
                refund_state: None,
                refund_locality: Default::default(),
                value: mixed_government_paper(),
            },
        )],
    );
    accepted(&send(&mut session, request));
    let request =
        government_answer_request(&mut session, Some(30_000), Some(government_refund_deduction()));
    accepted(&send(&mut session, request));
    let (entry, source) = accepted_government_source(&session);
    let (id, owner, component_ids) = (entry.id, entry.owner, source.component_ids.clone());
    assert!(component_ids.refund > 0);
    assert_ne!(component_ids.refund, id);
    let (repayments, history) = (source.repayments.clone(), source.history.clone());
    let mut correction = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut correction else { unreachable!() };
    let paper = &mut documents.common.government_payments.documents[0].document;
    paper.unemployment_compensation = 250_000;
    paper.state_or_local_income_tax_refunds = 50_000;
    paper.payer.address = "2 Example Street".into();
    accepted(&send(&mut session, correction));
    session = restart(session.draft.clone());
    let (entry, source) = accepted_government_source(&session);
    assert_eq!((entry.id, &entry.owner), (id, &owner));
    assert_eq!(source.component_ids, component_ids);
    assert_eq!(source.repayments, repayments);
    assert_eq!(source.history, history);
    let result = supported(&session.draft);
    assert_eq!(value(&result, &alloc::format!("unemployment_{id}_taxable")), 2_200);
    assert_eq!(value(&result, "taxable_state_tax_refund"), 500);
    assert_eq!(value(&result, "agi"), 62_700);
    assert_eq!(return_line(&result, "25b"), 50_000);
    let mut remove_refund = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut remove_refund else { unreachable!() };
    assert_eq!(documents.common.government_payments.documents.len(), 1);
    let row = &mut documents.common.government_payments.documents[0];
    assert_eq!((row.id, &row.owner), (id, &owner));
    assert_eq!(row.document.payer.address, "2 Example Street");
    row.document.state_or_local_income_tax_refunds = 0;
    accepted(&send(&mut session, remove_refund));
    let (_, source) = accepted_government_source(&session);
    assert_eq!(source.repayments, repayments);
    assert_eq!(source.history, history);
    let P::SaveIncomes { followups, .. } = open_topic(&mut session, TaxSection::GovernmentBenefits)
    else {
        unreachable!()
    };
    assert!(
        !followups.iter().any(|row| matches!(row.answer, DocumentAnswer::RefundDeduction { .. }))
    );
    assert_eq!(value(&supported(&session.draft), "agi"), 62_200);

    // Correcting away box 1 preserves the recorded repayment for the person
    // to reconcile; it must not silently discard an accepted answer.
    let mut remove_benefits = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut remove_benefits else { unreachable!() };
    let paper = &mut documents.common.government_payments.documents[0].document;
    paper.unemployment_compensation = 0;
    paper.state_or_local_income_tax_refunds = 50_000;
    accepted(&send(&mut session, remove_benefits));
    session = restart(session.draft);
    let (_, source) = accepted_government_source(&session);
    assert_eq!(source.repayments, repayments);
    assert_eq!(source.history, history);
    assert!(estimate(&session.draft).issues.iter().any(|issue| issue.display.key
        == "ISSUE_SAME_YEAR_UNEMPLOYMENT_REPAYMENTS_CANNOT_EXCEED_THIS_SOURCE_S_GROSS_BC0A30D5"));
    let mut topic = open_topic(&mut session, TaxSection::GovernmentBenefits);
    let P::SaveIncomes { followups, .. } = &mut topic else { unreachable!() };
    let repayment = followups
        .iter_mut()
        .find_map(|row| match &mut row.answer {
            DocumentAnswer::GovernmentRepayments { value } => Some(value),
            _ => None,
        })
        .unwrap();
    assert_eq!(*repayment, repayments);
    repayment.clear();
    accepted(&send(&mut session, topic));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "taxable_state_tax_refund"), 500);
    assert_eq!(value(&result, "agi"), 60_500);
    assert_eq!(return_line(&result, "25b"), 50_000);
    let mut removal = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut removal else { unreachable!() };
    documents.common.government_payments.documents.clear();
    accepted(&send(&mut session, removal));
    session = restart(session.draft);
    assert!(!session.draft.income.iter().any(|entry| entry.id == id));
    let P::SaveIncomes { followups, .. } = open_topic(&mut session, TaxSection::GovernmentBenefits)
    else {
        unreachable!()
    };
    assert!(!followups.iter().any(|row| row.key.starts_with(&alloc::format!("income/{id}/"))));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 60_000);
    assert_eq!(return_line(&result, "25b"), 0);
}

#[test]
fn government_topic_saves_preserve_full_accepted_refund_history() {
    use crate::tests::support::client::income_pages::open_topic;
    for (history, taxable_refund) in retained_government_history_cases() {
        let mut session = restart(retained_government_history_draft(history.clone()));
        let id = accepted_government_source(&session).0.id;
        let baseline = supported(&session.draft);
        // The $400 refund is fully taxable under the Pease and two-year
        // worksheets. The prior/current allocation taxes only its $300 prior
        // share. A history outside the ordinary worksheet stays outside the estimate.
        assert_eq!(value(&baseline, "taxable_state_tax_refund"), taxable_refund);
        assert_eq!(value(&baseline, &alloc::format!("unemployment_{id}_taxable")), 1_700);
        assert_eq!(value(&baseline, "agi"), 61_700 + taxable_refund);
        assert_eq!(return_line(&baseline, "25b"), 50_000);
        let unchanged = open_topic(&mut session, TaxSection::GovernmentBenefits);
        accepted(&send(&mut session, unchanged));
        session = restart(session.draft);
        assert_eq!(accepted_government_source(&session).1.history, history);
        let mut saved = supported(&session.draft);
        assert_eq!(saved.revision, baseline.revision + 1);
        saved.revision = baseline.revision;
        assert_eq!(saved, baseline);

        let request = government_answer_request(&mut session, Some(50_000), None);
        accepted(&send(&mut session, request));
        session = restart(session.draft);
        assert_eq!(accepted_government_source(&session).1.history, history);
        let repaid = supported(&session.draft);
        assert_eq!(value(&repaid, &alloc::format!("unemployment_{id}_taxable")), 1_500);
        assert_eq!(value(&repaid, "taxable_state_tax_refund"), taxable_refund);
        assert_eq!(value(&repaid, "agi"), 61_500 + taxable_refund);
        assert_eq!(return_line(&repaid, "25b"), 50_000);

        let mut expected = history.clone();
        if let TaxRefundHistory::IncomeTax { sales_available, ordinary, .. } = &mut expected {
            // This visible amount can change without replacing the accepted
            // Pease worksheet or changing its recorded ordinary treatment.
            *sales_available += 10_000;
            let ordinary = *ordinary;
            let mut request = open_topic(&mut session, TaxSection::GovernmentBenefits);
            let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
            let sales = followups
                .iter_mut()
                .find_map(|row| match &mut row.answer {
                    DocumentAnswer::RefundDeduction {
                        value: Some(RefundDeduction::IncomeTax { sales_available, .. }),
                    } => Some(sales_available),
                    _ => None,
                })
                .unwrap();
            *sales += 10_000;
            accepted(&send(&mut session, request));
            session = restart(session.draft);
            assert_eq!(accepted_government_source(&session).1.history, expected);
            let edited = supported(&session.draft);
            assert_eq!(value(&edited, "taxable_state_tax_refund"), taxable_refund);
            assert_eq!(value(&edited, "agi"), 61_500 + taxable_refund);
            assert_eq!(noted(&edited, "WARN_TAX_REFUND_LEFT_OUT"), !ordinary);
        }
    }
}

#[test]
fn printed_ccc_market_gain_matches_multiple_farms_and_each_recipient_separately() {
    let mut template = ccc_return(
        CccOrigin::Current,
        CccMethod::Loan,
        CccDisposition::Redeemed { year: 2026, paid: 285_000, sale: None },
        0,
    );
    ccc_mut(&mut template).lots[0].principal = 300_000;
    ccc_mut(&mut template).withholding = 0;
    let mut draft = empty_return(FederalStatus::Joint);
    for owner in
        [crate::DocumentOwner::Yours, crate::DocumentOwner::Yours, crate::DocumentOwner::Spouses]
    {
        let mut farm = template.income[0].clone();
        farm.owner = owner;
        let FederalIncome::Business { data } = &mut farm.income else { unreachable!() };
        let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
        data.ccc.as_mut().unwrap().lots[0].id = 0;
        add(&mut draft, farm);
    }
    for (owner, market_gain) in [
        (crate::DocumentOwner::Yours, 10_000),
        (crate::DocumentOwner::Yours, 20_000),
        (crate::DocumentOwner::Spouses, 15_000),
    ] {
        let mut paper = government_unemployment(0, &[], 0);
        paper.owner = owner;
        let FederalIncome::GovernmentPayment { data } = &mut paper.income else { unreachable!() };
        data.form.market_gain = Some(market_gain);
        add(&mut draft, paper);
    }
    let report = supported(&draft);
    assert_eq!(return_line(&report, "8"), 45_000);
    assert_eq!(
        schedule_f_lines(&report)
            .iter()
            .filter(|(line, _)| line == "4b")
            .map(|(_, cents)| cents)
            .sum::<i64>(),
        45_000
    );
    let issue =
        "ISSUE_FORM_1099_G_BOX_9_MARKET_GAIN_CANNOT_BE_RECONCILED_TO_THIS_RETURN_S_165FF0EF";
    assert!(!report.issues.iter().any(|row| row.display.key == issue));
    let FederalIncome::GovernmentPayment { data } = &mut draft.income.last_mut().unwrap().income
    else {
        unreachable!()
    };
    data.form.market_gain = Some(14_000);
    assert!(estimate(&draft).issues.iter().any(|row| row.display.key == issue));
}

// A state refund's paying agency prints its address. The Document Center save fills an
// unset refund state from it once; the recorded state then stays through later saves and
// paper corrections, and an address without a state leaves it unknown.
#[test]
fn document_center_save_fills_the_refund_state_once_from_the_paying_agencys_address() {
    use crate::tests::support::client::income_pages::save_documents;
    let paper = |address: &str| Form1099GPaper {
        payer: PrintedParty {
            name: "Example State Tax Agency".into(),
            address: address.into(),
            ..Default::default()
        },
        state_or_local_income_tax_refunds: 12_000,
        box_2_tax_year: Some(2025),
        ..Default::default()
    };
    let stored = |session: &Interview| {
        let entry = session
            .draft
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::GovernmentPayment { .. }))
            .unwrap();
        let FederalIncome::GovernmentPayment { data } = &entry.income else { unreachable!() };
        data.refund_state
    };
    for (address, expected) in [
        ("1 Example Plaza\nExample City\nCA US 95812", Some(TaxState::CA)),
        ("1 Example Plaza, Example City, california 95812-1234", Some(TaxState::CA)),
        ("1 Example Plaza, Example City", None),
    ] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::GovernmentPayment {
                    value: paper(address),
                    refund_state: None,
                    refund_locality: Default::default(),
                },
            )],
        );
        accepted(&send(&mut session, request));
        assert_eq!(stored(&session), expected, "{address}");
    }
    // A recorded state is kept when the paper is corrected later.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::GovernmentPayment {
                value: paper("1 Example Plaza, Example City, CA 95812"),
                refund_state: None,
                refund_locality: Default::default(),
            },
        )],
    );
    accepted(&send(&mut session, request));
    let mut correction = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut correction else { unreachable!() };
    let row = &mut documents.common.government_payments.documents[0];
    assert_eq!(row.refund_state, Some(TaxState::CA));
    row.document.payer.address = "1 Example Plaza, Example City, NY 10001".into();
    accepted(&send(&mut session, correction));
    assert_eq!(stored(&session), Some(TaxState::CA));
    assert_eq!(stored(&restart(session.draft.clone())), Some(TaxState::CA));
}
