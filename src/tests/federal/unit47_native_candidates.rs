use super::*;
use crate::tests::support::client::survey::survey_native_refusal as refuse;

fn calculation(request: &P) -> P {
    match request {
        P::SaveDeductions { value, allowed_routes, allowed_repayment, home_sites, .. } => {
            P::EstimateDeductions {
                user: TEST_USER,
                value: value.clone(),
                allowed_routes: allowed_routes.clone(),
                allowed_repayment: allowed_repayment.clone(),
                home_sites: home_sites.clone(),
            }
        }
        P::SaveOtherItemized {
            value,
            allowed_owners,
            interest_sources,
            investment_k1_sources,
            investment_origin_choices,
            ..
        } => P::EstimateOtherItemized {
            user: TEST_USER,
            value: value.clone(),
            allowed_owners: allowed_owners.clone(),
            interest_sources: interest_sources.clone(),
            investment_k1_sources: investment_k1_sources.clone(),
            investment_origin_choices: investment_origin_choices.clone(),
        },
        P::SavePayments {
            value,
            allowed_prior_returns,
            allowed_farmer,
            allowed_farmer_next_year,
            allowed_refund_management,
            ..
        } => P::EstimatePayments {
            user: TEST_USER,
            value: value.clone().into(),
            allowed_prior_returns: allowed_prior_returns.clone(),
            allowed_farmer: allowed_farmer.clone(),
            allowed_farmer_next_year: allowed_farmer_next_year.clone(),
            allowed_refund_management: allowed_refund_management.clone(),
        },
        _ => panic!("unexpected candidate"),
    }
}

fn open(session: &mut Interview, topic: TaxSection) -> P {
    let Reply::Edit(request) = from_review(session, topic) else {
        panic!("expected {topic:?} editor")
    };
    request
}
fn case(
    case: &str,
    session: &mut Interview,
    bad: P,
    corrected: P,
    topic: TaxSection,
    key: &str,
    path: &str,
) {
    refuse(case, session, bad.clone(), key, path);
    refuse(&alloc::format!("{case}-estimate"), session, calculation(&bad), key, path);
    let before = session.draft.clone();
    let dirty = session.dirty;
    let reply = send(session, calculation(&corrected));
    assert!(
        if matches!(corrected, P::SavePayments { .. }) {
            matches!(reply, Reply::Response(P::PaymentsEstimate { .. }))
        } else {
            matches!(reply, Reply::Response(P::RunningEstimate { .. }))
        },
        "{case}: corrected estimate"
    );
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, P::from_avro(&corrected.to_avro()).unwrap()));
    let saved = session.draft.clone();
    let request = open(session, topic);
    accepted(&send(session, request));
    assert_eq!(session.draft, saved, "{case}: reopened inputs changed");
    let mut restored = restart(saved.clone());
    let request = open(&mut restored, topic);
    accepted(&send(&mut restored, request));
    assert_eq!(restored.draft, saved, "{case}: cold reopened inputs changed");
}

#[test]
fn unit47_candidate_casualty_fields_preserve_event_and_property_order() {
    for i in 0..4 {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut corrected = open(&mut session, TaxSection::Deductions);
        let P::SaveDeductions { value, .. } = &mut corrected else { unreachable!() };
        value.casualties = vec![
            personal_disaster("Earlier event", 10_000).try_into().unwrap(),
            personal_disaster("Edited event", 10_000).try_into().unwrap(),
        ];
        let extra = value.casualties[1].properties[0].clone();
        value.casualties[1].properties.push(extra);
        if i == 3 {
            // 2026-10-17: labels are descriptions. Save two same-named
            // events first, then reject an actual repeated accepted identity.
            value.casualties[1].label = value.casualties[0].label.clone();
            accepted(&send(&mut session, corrected));
            corrected = open(&mut session, TaxSection::Deductions);
        }
        let mut bad = corrected.clone();
        let P::SaveDeductions { value, .. } = &mut bad else { unreachable!() };
        let (key, path) = match i {
            0 => {
                value.casualties[1].properties[1].after = 10_001;
                ("CASUALTY_AFTER_ABOVE_BEFORE", "[value/casualties/1/properties/1/after]")
            }
            1 => {
                value.casualties[1].properties[1].received = 1;
                (
                    "CASUALTY_RECEIVED_ABOVE_REIMBURSEMENT",
                    "[value/casualties/1/properties/1/received]",
                )
            }
            2 => {
                value.casualties[1].zip = "1234".into();
                ("CASUALTY_ZIP", "[value/casualties/1/zip]")
            }
            3 => {
                value.casualties[1].id = value.casualties[0].id;
                (
                    "ISSUE_INTERVIEW_A_RECORD_IDENTITY_IS_REPEATED_IN_EE6DC425",
                    "[value/casualties/1/id]",
                )
            }
            _ => unreachable!(),
        };
        case(
            &alloc::format!("candidate-casualty-{i}"),
            &mut session,
            bad,
            corrected,
            TaxSection::Deductions,
            key,
            path,
        );
    }
}

#[test]
fn unit47_candidate_investment_duplicate_keeps_generic_message_and_actual_label() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut deductions = open(&mut session, TaxSection::Deductions);
    let P::SaveDeductions { value, .. } = &mut deductions else { unreachable!() };
    value.route = DeductionRoute::Itemize;
    accepted(&send(&mut session, deductions));
    let mut corrected = open(&mut session, TaxSection::OtherItemized);
    let P::SaveOtherItemized { value, .. } = &mut corrected else { unreachable!() };
    let loan = |label: &str| InvestmentLoanPage {
        paid_owners: None,
        carry_owners: None,
        label: label.into(),
        source: 0,
        originating_k1: None,
        allowed_origin_choices: vec![],
        paid: 10_000,
        carry: 0,
        amt_carry: 0,
        private_activity: 0,
    };
    value.investment_interest = Some(InvestmentBorrowingPage {
        loans: vec![loan("Earlier borrowing"), loan("Edited borrowing")],
        election: InvestmentElection::None,
        income_use: InvestmentInterestUse::Unresolved,
        limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
    });
    let mut bad = corrected.clone();
    let P::SaveOtherItemized { value, .. } = &mut bad else { unreachable!() };
    let loans = &mut value.investment_interest.as_mut().unwrap().loans;
    loans[1].label = loans[0].label.clone();
    case(
        "candidate-investment-loan-label",
        &mut session,
        bad,
        corrected,
        TaxSection::OtherItemized,
        "INVESTMENT_BORROWING_RECORDS",
        "[value/investment_interest/loans/1/label]",
    );
}

#[test]
fn unit47_candidate_payment_property_preserves_amount_management_priority() {
    for i in 0..3 {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut corrected = open(&mut session, TaxSection::Payments);
        let P::SavePayments { value, .. } = &mut corrected else { unreachable!() };
        let property = RefundSourceProperty::Partly {
            community: 100,
            separate_owner: Owner::Taxpayer,
            management: RefundManagement::NotApplicable,
        };
        value.estimated = vec![
            InjuredDatedPayment {
                date: 20260415,
                amount: 100,
                payer: PaymentPayer::Taxpayer,
                refund_property: None,
            },
            InjuredDatedPayment {
                date: 20260615,
                amount: 100,
                payer: PaymentPayer::Taxpayer,
                refund_property: Some(property.clone()),
            },
        ];
        value.extension = 100;
        value.extension_payer = Some(PaymentPayer::Taxpayer);
        value.extension_refund_property = Some(property);
        let mut bad = corrected.clone();
        let P::SavePayments { value, .. } = &mut bad else { unreachable!() };
        let path = if i == 1 {
            let Some(RefundSourceProperty::Partly { community, .. }) =
                &mut value.extension_refund_property
            else {
                unreachable!()
            };
            *community = 101;
            "[value/extension_refund_property/community]"
        } else {
            let Some(RefundSourceProperty::Partly { community, management, .. }) =
                &mut value.estimated[1].refund_property
            else {
                unreachable!()
            };
            *management = RefundManagement::Spouse;
            if i == 0 {
                *community = 101;
                "[value/estimated/1/refund_property/community]"
            } else {
                "[value/estimated/1/refund_property/management]"
            }
        };
        case(
            &alloc::format!("candidate-payment-property-{i}"),
            &mut session,
            bad,
            corrected,
            TaxSection::Payments,
            "INJURED_SPOUSE_COMMUNITY_ABOVE_PAYMENT",
            path,
        );
    }
}

#[test]
fn unit47_candidate_repayment_nested_minimum_tax_keeps_original_generic_message() {
    let mut session = Interview::new(TEST_USER, repayment_minimum_chain());
    let mut corrected = open(&mut session, TaxSection::Deductions);
    let P::SaveDeductions { value, .. } = &mut corrected else { unreachable!() };
    let RepaymentClaim::Claim { years, .. } = &mut value.repayment else { unreachable!() };
    years[1].minimum_tax = Some(repayment_minimum_source(2024));
    years[1].minimum_tax.as_mut().unwrap().foreign_credit =
        MinimumTaxForeignCredit::PriorStatementElection { credit: 30_000 };
    let mut bad = corrected.clone();
    let P::SaveDeductions { value, .. } = &mut bad else { unreachable!() };
    let RepaymentClaim::Claim { years, .. } = &mut value.repayment else { unreachable!() };
    years[1].minimum_tax.as_mut().unwrap().foreign_credit =
        MinimumTaxForeignCredit::PriorStatementElection { credit: 30_001 };
    case(
        "candidate-repayment-minimum-tax",
        &mut session,
        bad,
        corrected,
        TaxSection::Deductions,
        "MESSAGE_CHECK_THE_ORIGINAL_YEAR_RECORDS_OF_YOUR_REPAYMENT_COMPARISON_ONE_RECORD_PER",
        "[value/repayment/years/1/minimum_tax/foreign_credit/credit]",
    );
}
