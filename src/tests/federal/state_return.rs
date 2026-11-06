use super::*;
use crate::tests::cross::interview::{
    cash_farm, document, open_topic, page_documents, r1099, refusal, save_topic, topic_for, w2,
};
use crate::tests::support::client::income_pages::{open_document_center, save_documents};
use crate::tests::support::client::roundtrip_native_draft;

fn payer() -> TaxPayerIdentity {
    TaxPayerIdentity {
        name: "Synthetic payer".into(),
        federal_id: "00-1234567".into(),
        address: "1 Example Street".into(),
    }
}
fn states() -> Vec<StateIncomeTax> {
    vec![
        StateIncomeTax {
            state: TaxState::NY,
            payer_state_id: "000-123".into(),
            income: Some(50_001),
            withholding: Some(1_234),
        },
        StateIncomeTax {
            state: TaxState::NJ,
            payer_state_id: "".into(),
            income: None,
            withholding: Some(0),
        },
    ]
}
fn withholding() -> StateWithholdingFields {
    StateWithholdingFields {
        states: vec![StateTaxWithholding {
            state: TaxState::CA,
            payer_state_id: "0000123".into(),
            withholding: Some(1_234),
        }],
    }
}
fn localities() -> Vec<LocalIncomeTax> {
    vec![
        LocalIncomeTax {
            locality: "Example City".into(),
            state: None,
            income: None,
            withholding: Some(42),
        },
        LocalIncomeTax {
            locality: "Example District".into(),
            state: Some(TaxState::NY),
            income: Some(0),
            withholding: Some(0),
        },
    ]
}
fn state_income() -> StateIncomeFields {
    StateIncomeFields { states: states() }
}
fn state_local() -> StateLocalFields {
    StateLocalFields { states: states(), localities: localities() }
}
fn w2_source() -> W2StateFields {
    W2StateFields {
        control_number: "".into(),
        nonqualified_plans: 12_345,
        third_party_sick_pay: true,
        states: states(),
        localities: localities(),
        box14a: vec![
            W2OtherItem { label: "CA SDI".into(), value: W2OtherValue::Money { amount: 4321 } },
            W2OtherItem {
                label: "Employer wording".into(),
                value: W2OtherValue::Text { text: "As printed".into() },
            },
        ],
        tipped_occupation_codes: vec![
            TippedOccupationCode { code: 0 },
            TippedOccupationCode { code: 101 },
        ],
    }
}
fn business_forms() -> Vec<BusinessStateDocument> {
    vec![
        BusinessStateDocument {
            payer: payer(),
            form: BusinessStateForm::Nec { data: state_income() },
        },
        BusinessStateDocument {
            payer: payer(),
            form: BusinessStateForm::Misc { data: state_income() },
        },
        BusinessStateDocument {
            payer: payer(),
            form: BusinessStateForm::K { data: withholding() },
        },
    ]
}
fn documents() -> Vec<IncomeSource> {
    vec![
        IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w2(100_000))) },
        IncomeSource::Gambling(w2g_document(W2g {
            payer: Default::default(),
            winner: Default::default(),
            winnings: 100_000,
            withholding: 0,
            special: false,
            state_details: Default::default(),
        })),
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(native_int1099(
                "".into(),
                100_000,
            ))),
        },
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Div1099(dividend_document(native_div1099(
                "".into(),
                100_000,
                0,
            ))),
        },
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Oid1099(oid_document(native_oid1099(
                "".into(),
                100_000,
                0,
            ))),
        },
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Sale1099B(Sale1099B {
                payer: Default::default(),
                recipient: Default::default(),
                digital_asset: false,
                market_discount: 0,
                several_sales: false,
                description: "".into(),
                term: Some(GainTerm::Long),
                proceeds: 100_000,
                basis: Some(50_000),
                reporting: BasisReporting::ReportedToIrs,
                use_kind: SaleUse::Investment { wash_sale: 0 },
                collectible: false,
                withholding: 0,
                state_details: Default::default(),
            }),
        },
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Contracts1099B(Contracts1099B {
                payer: crate::PrintedParty::from_name("".into()),
                recipient: Default::default(),
                aggregate: -100_000,
                withholding: 0,
                state_details: Default::default(),
            }),
        },
        IncomeSource::GovernmentPayment {
            refund_state: None,
            refund_locality: Default::default(),
            value: government_document(Form1099G { amount: 100_000, ..Default::default() }),
        },
        IncomeSource::GovernmentPayment {
            refund_state: None,
            refund_locality: Default::default(),
            value: refund_paper(Refund1099G {
                payer: Default::default(),
                recipient: Default::default(),
                amount: 100_000,
                year: 2025,
                withholding: 0,
                state_details: Default::default(),
            }),
        },
        IncomeSource::Retirement {
            value: crate::RetirementDocument::R1099(retirement_document(r1099(
                100_000,
                DistributionCode::Normal { what_happened: Disposition::Kept },
                false,
            ))),
        },
        IncomeSource::Retirement {
            value: crate::RetirementDocument::Ssa1099(ssa_document(ssa_1099_net(100_000, 0))),
        },
        IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::Form1099Misc(misc_document(Form1099Misc {
                payer: Default::default(),
                recipient: Default::default(),
                cash_tips: 0,
                tip_occupation_codes: Default::default(),
                overtime: 0,
                crop_insurance: 0,
                component_ids: Default::default(),
                rents: 0,
                royalties: 0,
                other: 0,
                other_use: None,
                withholding: 0,
                fishing: 0,
                medical: 0,
                substitute: 100_000,
                attorney: 0,
                client_funds: 0,
                fish_sales: 0,
                deferred: 0,
                state_details: Default::default(),
            })),
        },
        IncomeSource::SelfEmployment {
            value: crate::SelfEmploymentDocument::Form1099Nec(nec_document(Form1099Nec {
                payer: Default::default(),
                recipient: Default::default(),
                cash_tips: 0,
                tip_occupation_codes: Default::default(),
                overtime: 0,
                compensation: 100_000,
                withholding: 0,
                golden_parachute: 0,
                belongs: Some(IncomeUse::NotForProfit),
                state_details: Default::default(),
            })),
        },
        IncomeSource::SelfEmployment {
            value: crate::SelfEmploymentDocument::Form1099K(k_document(Form1099K {
                payer: Default::default(),
                recipient: Default::default(),
                cash_tips: 0,
                tip_occupation_codes: Default::default(),
                gross: 100_000,
                withholding: 0,
                parts: vec![KPart {
                    id: 0,
                    amount: 100_000,
                    belongs: Some(KUse::NotIncome),
                    refunded: 0,
                }],
                state_details: Default::default(),
            })),
        },
        IncomeSource::Record { value: crate::IncomeRecord::Farm(cash_farm(100_000, 0)) },
    ]
}
fn source(document: &IncomeSource) -> IncomeStateSource {
    match document {
        IncomeSource::Wage { value: crate::WageDocument::W2(_) } => {
            IncomeStateSource::Wage(w2_source())
        }
        IncomeSource::Gambling(_) => IncomeStateSource::Gambling(state_local()),
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Int1099(_) }
        | IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Div1099(_) }
        | IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Oid1099(_) } => {
            IncomeStateSource::Investment(InvestmentStateFields {
                states: withholding().states,
                exempt_sources: vec![TaxExemptStateIncome {
                    state: TaxState::PR,
                    allocation: TaxExemptAllocation::Amount { amount: 0 },
                }],
                government_obligation_percentage: Some(27.43),
            })
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Sale1099B(_) }
        | IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Contracts1099B(_),
        } => IncomeStateSource::Withholding(withholding()),
        IncomeSource::GovernmentPayment { value, .. }
            if value.state_or_local_income_tax_refunds != 0 =>
        {
            IncomeStateSource::Refund(RefundStateFields {
                refund_state: Some(TaxState::NY),
                refund_locality: "Example City".into(),
                taxes: withholding(),
            })
        }
        IncomeSource::GovernmentPayment { .. } => IncomeStateSource::Withholding(withholding()),
        IncomeSource::Retirement { value: crate::RetirementDocument::R1099(_) } => {
            IncomeStateSource::Retirement(RetirementStateFields { taxes: state_local() })
        }
        IncomeSource::Retirement { value: crate::RetirementDocument::Ssa1099(_) } => {
            IncomeStateSource::SocialSecurity {
                statement: Some(SocialSecurityStatement::SocialSecurity),
            }
        }
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Form1099Misc(_) } => {
            IncomeStateSource::Misc(state_income())
        }
        IncomeSource::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099Nec(_) } => {
            IncomeStateSource::Business {
                documents: vec![BusinessStateDocument {
                    payer: payer(),
                    form: BusinessStateForm::Nec { data: state_income() },
                }],
            }
        }
        IncomeSource::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099K(_) } => {
            IncomeStateSource::Business {
                documents: vec![BusinessStateDocument {
                    payer: payer(),
                    form: BusinessStateForm::K { data: withholding() },
                }],
            }
        }
        IncomeSource::Record { value: crate::IncomeRecord::Farm(_) } => {
            IncomeStateSource::Business { documents: business_forms() }
        }
        _ => unreachable!(),
    }
}
fn saved(document_value: IncomeSource) -> Interview {
    let draft = empty_return(FederalStatus::Single);
    let topic = topic_for(&document_value);
    let mut session = Interview::new(TEST_USER, draft);
    let request = if matches!(document_value, IncomeSource::Record { .. }) {
        save_topic(&mut session, topic, vec![document(0, document_value)])
    } else {
        save_documents(&mut session, vec![document(0, document_value)])
    };
    let reply = send(&mut session, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    session
}

#[test]
fn every_state_source_survives_save_avro_storage_and_reopening_without_changing_federal_income() {
    for mut doc in documents() {
        let federal = crate::interview::document_income(&doc);
        let source = source(&doc);
        // 2026-10-26: typed SSA/RRB papers and FederalSocial.form_boxes retain
        // statement origin without creating state facts or a second source.
        let expected = match &source {
            IncomeStateSource::SocialSecurity { .. } => None,
            _ => Some(source.clone()),
        };
        assert!(source.apply(&mut doc), "state source {source:?} not applied to {doc:?}");
        if !matches!(
            doc,
            IncomeSource::Record { .. }
                | IncomeSource::Retirement { value: crate::RetirementDocument::Ssa1099(_) }
        ) {
            let mut names = doc.printed_names().unwrap_or_default();
            names.issuer = Some(payer());
            doc.set_printed_names(&names);
        }
        // Compare the calculation owners: party blocks and state records stay
        // on the paper without changing any federal input.
        let assert_figures =
            |actual: &FederalIncome, expected: &FederalIncome| match (actual, expected) {
                (FederalIncome::Wage { data: actual }, FederalIncome::Wage { data: expected }) => {
                    assert_eq!(
                        crate::FederalWage::from(actual),
                        crate::FederalWage::from(expected)
                    );
                }
                (
                    FederalIncome::Gambling { data: actual },
                    FederalIncome::Gambling { data: expected },
                ) => {
                    assert_eq!(
                        crate::FederalGambling::from(actual),
                        crate::FederalGambling::from(expected)
                    );
                }
                (
                    FederalIncome::Interest { data: actual },
                    FederalIncome::Interest { data: expected },
                ) => {
                    assert_eq!(
                        crate::FederalInterest::from(actual),
                        crate::FederalInterest::from(expected)
                    );
                }
                (
                    FederalIncome::Dividend { data: actual },
                    FederalIncome::Dividend { data: expected },
                ) => {
                    assert_eq!(
                        crate::FederalDividend::from(actual),
                        crate::FederalDividend::from(expected)
                    );
                }
                (FederalIncome::Oid { data: actual }, FederalIncome::Oid { data: expected }) => {
                    assert_eq!(crate::FederalOid::from(actual), crate::FederalOid::from(expected));
                }
                (FederalIncome::Sale { data: actual }, FederalIncome::Sale { data: expected }) => {
                    assert_eq!(
                        crate::FederalSale::from(actual),
                        crate::FederalSale::from(expected)
                    );
                }
                (
                    FederalIncome::Section1256 { data: actual },
                    FederalIncome::Section1256 { data: expected },
                ) => {
                    assert_eq!(
                        crate::FederalSection1256::from(actual),
                        crate::FederalSection1256::from(expected)
                    );
                }
                (
                    FederalIncome::GovernmentPayment { data: actual },
                    FederalIncome::GovernmentPayment { data: expected },
                ) => {
                    assert_eq!(
                        crate::FederalUnemployment::from(actual),
                        crate::FederalUnemployment::from(expected)
                    );
                    assert_eq!(
                        crate::FederalTaxRefund::from(actual),
                        crate::FederalTaxRefund::from(expected)
                    );
                }
                (
                    FederalIncome::Retirement { data: actual },
                    FederalIncome::Retirement { data: expected },
                ) => {
                    assert_eq!(
                        crate::FederalRetirement::from(actual),
                        crate::FederalRetirement::from(expected)
                    );
                }
                (
                    FederalIncome::SocialSecurity { data: actual },
                    FederalIncome::SocialSecurity { data: expected },
                ) => {
                    assert_eq!(
                        crate::FederalSocial::from(actual),
                        crate::FederalSocial::from(expected)
                    );
                }
                (
                    FederalIncome::Misc1099 { data: actual },
                    FederalIncome::Misc1099 { data: expected },
                ) => {
                    assert_eq!(
                        crate::FederalForm1099Misc::from(actual),
                        crate::FederalForm1099Misc::from(expected)
                    );
                }
                (
                    FederalIncome::Nec1099 { data: actual },
                    FederalIncome::Nec1099 { data: expected },
                ) => {
                    assert_eq!(
                        crate::FederalForm1099Nec::from(actual),
                        crate::FederalForm1099Nec::from(expected)
                    );
                }
                (
                    FederalIncome::K1099 { data: actual },
                    FederalIncome::K1099 { data: expected },
                ) => {
                    assert_eq!(
                        crate::FederalForm1099K::from(actual),
                        crate::FederalForm1099K::from(expected)
                    );
                }
                _ => assert_eq!(actual, expected),
            };
        assert_figures(&crate::interview::document_income(&doc), &federal);
        let topic = topic_for(&doc);
        let source_editor = !matches!(doc, IncomeSource::Record { .. });
        let session = saved(doc);
        assert_eq!(session.draft.income[0].state_source, expected);
        if matches!(source, IncomeStateSource::SocialSecurity { .. }) {
            assert_figures(&session.draft.income[0].income, &federal);
        }
        let draft = roundtrip_native_draft(session.draft);
        if matches!(source, IncomeStateSource::SocialSecurity { .. }) {
            assert_figures(&draft.income[0].income, &federal);
        }
        let mut restored = Interview::new(TEST_USER, draft);
        let page = if source_editor {
            open_document_center(&mut restored)
        } else {
            open_topic(&mut restored, topic)
        };
        let rows = page_documents(&page);
        assert_eq!(rows.len(), 1);
        assert_eq!(IncomeStateSource::from_document(&rows[0].document), expected);
        if matches!(source, IncomeStateSource::SocialSecurity { .. }) {
            assert_figures(&crate::interview::document_income(&rows[0].document), &federal);
        }
    }
}

#[test]
fn absent_state_facts_do_not_make_existing_forms_incomplete_or_create_source_records() {
    for doc in documents() {
        assert_eq!(IncomeStateSource::from_document(&doc), None);
        assert_eq!(crate::state_return::document_rule(&doc), None);
        let session = saved(doc);
        assert_eq!(session.draft.income[0].state_source, None);
    }
}

#[test]
fn replacing_and_removing_state_facts_replaces_the_saved_source() {
    let mut form = w2(100_000);
    form.state_details = w2_source();
    form.employer = payer();
    let mut session =
        saved(IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) });
    let id = session.draft.income[0].id;
    for details in [
        W2StateFields {
            states: vec![StateIncomeTax {
                state: TaxState::CA,
                payer_state_id: "".into(),
                income: Some(0),
                withholding: Some(0),
            }],
            ..Default::default()
        },
        W2StateFields::default(),
    ] {
        let mut form = w2(100_000);
        form.state_details = details;
        let expected = IncomeStateSource::from_document(&IncomeSource::Wage {
            value: crate::WageDocument::W2(wage_document(form.clone())),
        });
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut session,
            vec![document(
                id,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) },
            )],
        );
        let reply = send(&mut session, request);
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        assert_eq!(session.draft.income[0].id, id);
        assert_eq!(session.draft.income[0].state_source, expected);
    }
}

#[test]
fn state_amounts_are_independent_of_federal_and_blank_income_is_distinct_from_zero() {
    let mut form = w2(100_000);
    form.state_details = W2StateFields {
        states: vec![
            StateIncomeTax {
                state: TaxState::NY,
                payer_state_id: "".into(),
                income: Some(200_000),
                withholding: Some(300_000),
            },
            StateIncomeTax {
                state: TaxState::NJ,
                payer_state_id: "".into(),
                income: None,
                withholding: Some(0),
            },
        ],
        localities: localities(),
        ..Default::default()
    };
    let session =
        saved(IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form.clone())) });
    let Some(IncomeStateSource::Wage(source)) = &session.draft.income[0].state_source else {
        panic!("expected W-2 source")
    };
    assert_eq!(source.states, form.state_details.states);
    assert_eq!(source.localities[0].income, None);
    assert_eq!(source.localities[1].income, Some(0));
    for withheld in [None, Some(0)] {
        form.state_details.states[1].withholding = withheld;
        form.state_details.localities[1].withholding = withheld;
        let session = saved(IncomeSource::Wage {
            value: crate::WageDocument::W2(wage_document(form.clone())),
        });
        let mut cold = restart(session.draft.clone());
        let request = open_document_center(&mut cold);
        let row = page_documents(&request).into_iter().next().unwrap();
        let IncomeSource::Wage { value: crate::WageDocument::W2(paper) } = row.document else {
            unreachable!()
        };
        assert_eq!(paper.states[1].withholding, withheld);
        assert_eq!(paper.localities[1].withholding, withheld);
        let reply = send(&mut cold, request);
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        assert_eq!(cold.draft, session.draft);
    }
}

#[test]
fn clearing_payer_text_survives_storage_without_changing_nullable_income() {
    let mut form = w2(100_000);
    form.state_details = w2_source();
    form.employer = payer();
    let mut session =
        saved(IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) });
    let page = open_document_center(&mut session);
    let mut row = page_documents(&page)[0].clone();
    let IncomeSource::Wage { value: crate::WageDocument::W2(form) } = &mut row.document else {
        panic!("expected W-2")
    };
    form.employer = TaxPayerIdentity::default();
    for state in &mut form.states {
        state.payer_state_id.clear();
    }
    let expected = form.state_details();
    let request = save_documents(&mut session, vec![row]);
    let reply = send(&mut session, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    session.draft = FederalDraft::from_avro(&session.draft.to_avro()).unwrap();
    let page = open_document_center(&mut session);
    let IncomeSource::Wage { value: crate::WageDocument::W2(form) } =
        &page_documents(&page)[0].document
    else {
        panic!("expected W-2")
    };
    assert_eq!(form.state_details(), expected);
    assert_eq!(form.states[1].income, None);
    assert_eq!(form.localities[1].income, Some(0));
}

#[test]
fn invalid_state_source_is_rejected_before_any_draft_change() {
    let mut form = w2(100_000);
    let mut session =
        saved(IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form.clone())) });
    let before = session.draft.clone();
    {
        let details = W2StateFields { states: [states(), states()].concat(), ..Default::default() };
        form.state_details = details;
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut session,
            vec![document(
                before.income[0].id,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form.clone())) },
            )],
        );
        let reply = send(&mut session, request);
        assert!(refusal(reply).contains("Check the state and local information"));
        assert_eq!(session.draft, before);
    }
    // The printed state payer ID retains its full text.
    form.state_details = W2StateFields {
        states: vec![StateIncomeTax {
            state: TaxState::CA,
            payer_state_id: "x".repeat(41),
            income: None,
            withholding: Some(0),
        }],
        ..Default::default()
    };
    let expected = IncomeStateSource::from_document(&IncomeSource::Wage {
        value: crate::WageDocument::W2(wage_document(form.clone())),
    });
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![document(
            before.income[0].id,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) },
        )],
    );
    let reply = send(&mut session, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    assert_eq!(session.draft.income[0].state_source, expected);
    // The stored paper keeps its printed state blocks; the rest of the form is unchanged.
    let without_state = |income: &FederalIncome| {
        let mut income = income.clone();
        if let FederalIncome::Wage { data: crate::WageSource::W2(w2) } = &mut income {
            w2.form.states.clear();
            w2.form.localities.clear();
        }
        income
    };
    assert_eq!(
        without_state(&session.draft.income[0].income),
        without_state(&before.income[0].income)
    );
}

#[test]
fn state_source_cannot_be_applied_to_a_different_form_kind() {
    let mut form =
        IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w2(100_000))) };
    assert!(!IncomeStateSource::Withholding(withholding()).apply(&mut form));
    assert_eq!(IncomeStateSource::from_document(&form), None);
}

#[test]
fn supplemental_percentage_precision_survives_storage_without_conversion() {
    let original = InvestmentStateFields {
        states: vec![],
        exempt_sources: vec![TaxExemptStateIncome {
            state: TaxState::CA,
            allocation: TaxExemptAllocation::Percentage { value: 27.43125 },
        }],
        government_obligation_percentage: Some(3.00001),
    };
    let mut session = saved(IncomeSource::InterestDividend {
        value: crate::InterestDividendDocument::Int1099(interest_document(Int1099 {
            state_details: original.clone(),
            ..native_int1099("".into(), 100_000)
        })),
    });
    session.draft = FederalDraft::from_avro(&session.draft.to_avro()).unwrap();
    let page = open_document_center(&mut session);
    let IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Int1099(form) } =
        &page_documents(&page)[0].document
    else {
        panic!("expected interest form")
    };
    assert_eq!(form.state_details, original);
}

#[test]
fn supplemental_percentages_accept_zero_fractional_and_full_allocations() {
    for value in [0.0, 0.25, 27.43125, 100.0] {
        let original = InvestmentStateFields {
            exempt_sources: vec![TaxExemptStateIncome {
                state: TaxState::CA,
                allocation: TaxExemptAllocation::Percentage { value },
            }],
            government_obligation_percentage: Some(value),
            ..Default::default()
        };
        let mut session = saved(IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(Int1099 {
                state_details: original.clone(),
                ..native_int1099("".into(), 100_000)
            })),
        });
        session.draft = FederalDraft::from_avro(&session.draft.to_avro()).unwrap();
        let page = open_document_center(&mut session);
        let IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(form),
        } = &page_documents(&page)[0].document
        else {
            panic!("expected interest form")
        };
        assert_eq!(form.state_details, original);
    }
}

#[test]
fn government_obligation_percentage_keeps_absence_distinct_from_reported_zero() {
    for percentage in [None, Some(0.0)] {
        let mut session = saved(IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(Int1099 {
                payer: crate::PrintedParty::from_name("".into()),
                recipient: Default::default(),
                taxable: 100_000,
                more: IntMore::default(),
                after_death: None,
                state_details: InvestmentStateFields {
                    government_obligation_percentage: percentage,
                    ..Default::default()
                },
                nominee: None,
                adjustments: None,
            })),
        });
        session.draft = FederalDraft::from_avro(&session.draft.to_avro()).unwrap();
        assert_eq!(session.draft.income[0].state_source.is_some(), percentage.is_some());
        let page = open_document_center(&mut session);
        let IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(form),
        } = &page_documents(&page)[0].document
        else {
            panic!("expected interest form")
        };
        assert_eq!(form.state_details.government_obligation_percentage, percentage);
    }
}

#[test]
fn sponsoring_state_survives_529_transfer_storage_without_changing_federal_eligibility() {
    let mut original = qtp_roth_rollover();
    let eligibility = original.eligible();
    let amount = original.amount();
    original.rollover.plan_state = Some(TaxState::NY);
    let restored = QtpRothRollover::from_avro(&original.rollover.to_avro()).unwrap();
    assert_eq!(restored.plan_state, Some(TaxState::NY));
    assert_eq!(restored.valid(amount), eligibility);
}

#[test]
fn the_1099_r_page_asks_no_railroad_statement_choice() {
    // F3 (2026-10-09): the R1099 import is for Form 1099-R only, and the
    // choice changed nothing, so the page's state fields hold the payer and
    // state boxes alone.
    let source = include_str!("../../core/shared/printed_state.rs");
    let page = &source[source.find("pub struct RetirementStateFields {").unwrap()..];
    let page = &page[..page.find("\n}").unwrap()];
    assert!(page.contains("pub taxes: StateLocalFields,"));
    assert!(!page.contains("statement"));
    assert!(!source.contains("Railroad Retirement Board Form RRB-1099-R"));
}
