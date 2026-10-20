use super::*;
use crate::tests::cross::interview::{
    cash_farm, document, open_topic, page_documents, r1099, refusal, save_topic, topic_for, w2,
};
use crate::tests::support::client::roundtrip_native_draft;
use alloc::string::String;

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
            withholding: 1_234,
        },
        StateIncomeTax {
            state: TaxState::NJ,
            payer_state_id: "".into(),
            income: None,
            withholding: 0,
        },
    ]
}
fn withholding() -> StateWithholdingFields {
    StateWithholdingFields {
        payer: payer(),
        states: vec![StateTaxWithholding {
            state: TaxState::CA,
            payer_state_id: "0000123".into(),
            withholding: 1_234,
        }],
    }
}
fn localities() -> Vec<LocalIncomeTax> {
    vec![
        LocalIncomeTax {
            locality: "Example City".into(),
            state: None,
            income: None,
            withholding: 42,
        },
        LocalIncomeTax {
            locality: "Example District".into(),
            state: Some(TaxState::NY),
            income: Some(0),
            withholding: 0,
        },
    ]
}
fn state_income() -> StateIncomeFields {
    StateIncomeFields { payer: payer(), states: states() }
}
fn state_local() -> StateLocalFields {
    StateLocalFields { payer: payer(), states: states(), localities: localities() }
}
fn w2_source() -> W2StateFields {
    W2StateFields {
        employer: payer(),
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
        BusinessStateDocument { form: BusinessStateForm::Nec { data: state_income() } },
        BusinessStateDocument { form: BusinessStateForm::Misc { data: state_income() } },
        BusinessStateDocument { form: BusinessStateForm::K { data: withholding() } },
    ]
}
fn documents() -> Vec<IncomeDocument> {
    vec![
        IncomeDocument::W2(w2(100_000)),
        IncomeDocument::W2g(W2g {
            winnings: 100_000,
            withholding: 0,
            special: false,
            state_details: Default::default(),
        }),
        IncomeDocument::Int1099(native_int1099("".into(), 100_000)),
        IncomeDocument::Div1099(native_div1099("".into(), 100_000, 0)),
        IncomeDocument::Oid1099(native_oid1099("".into(), 100_000, 0)),
        IncomeDocument::Sale1099B(Sale1099B {
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
            adjustment: SaleAdjustment::None,
            special: crate::SaleSpecial::NoneOfThese,
            state_details: Default::default(),
        }),
        IncomeDocument::Contracts1099B(Contracts1099B {
            broker: "".into(),
            aggregate: -100_000,
            withholding: 0,
            special: false,
            state_details: Default::default(),
        }),
        IncomeDocument::Form1099G(Form1099G {
            amount: 100_000,
            withholding: 0,
            more: Form1099GMore::default(),
            state_details: Default::default(),
        }),
        IncomeDocument::Refund1099G(Refund1099G {
            amount: 100_000,
            year: 2025,
            withholding: 0,
            deducted: RefundDeduction::StandardDeduction,
            state_details: Default::default(),
        }),
        IncomeDocument::R1099(r1099(
            100_000,
            DistributionCode::Normal { what_happened: Disposition::Kept },
            false,
        )),
        IncomeDocument::Ssa1099(Ssa1099 { benefits: 100_000, withholding: 0, statement: None }),
        IncomeDocument::Form1099Misc(Form1099Misc {
            rents: 0,
            rents_use: None,
            royalties: 0,
            royalties_use: None,
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
            business: None,
            state_details: Default::default(),
        }),
        IncomeDocument::Form1099Nec(Form1099Nec {
            compensation: 100_000,
            withholding: 0,
            golden_parachute: 0,
            belongs: IncomeUse::NotForProfit,
            business: None,
            state_details: Default::default(),
        }),
        IncomeDocument::Form1099K(Form1099K {
            gross: 100_000,
            withholding: 0,
            parts: vec![KPart { amount: 100_000, belongs: KUse::NotIncome, refunded: 0 }],
            state_details: Default::default(),
        }),
        IncomeDocument::Farm(cash_farm(100_000, 0)),
    ]
}
fn source(document: &IncomeDocument) -> IncomeStateSource {
    match document {
        IncomeDocument::W2(_) => IncomeStateSource::Wage(w2_source()),
        IncomeDocument::W2g(_) => IncomeStateSource::Gambling(state_local()),
        IncomeDocument::Int1099(_) | IncomeDocument::Div1099(_) | IncomeDocument::Oid1099(_) => {
            IncomeStateSource::Investment(InvestmentStateFields {
                payer_federal_id: "00-1234567".into(),
                payer_address: "1 Example Street".into(),
                states: withholding().states,
                exempt_sources: vec![TaxExemptStateIncome {
                    state: TaxState::PR,
                    allocation: TaxExemptAllocation::Amount { amount: 0 },
                }],
                government_obligation_percentage: Some(27.43),
            })
        }
        IncomeDocument::Sale1099B(_)
        | IncomeDocument::Contracts1099B(_)
        | IncomeDocument::Form1099G(_) => IncomeStateSource::Withholding(withholding()),
        IncomeDocument::Refund1099G(_) => IncomeStateSource::Refund(RefundStateFields {
            refund_state: Some(TaxState::NY),
            refund_locality: "Example City".into(),
            taxes: withholding(),
        }),
        IncomeDocument::R1099(_) => {
            IncomeStateSource::Retirement(RetirementStateFields { taxes: state_local() })
        }
        IncomeDocument::Ssa1099(_) => {
            IncomeStateSource::SocialSecurity { statement: Some(SocialSecurityStatement::Railroad) }
        }
        IncomeDocument::Form1099Misc(_) => IncomeStateSource::Misc(state_income()),
        IncomeDocument::Form1099Nec(_) => IncomeStateSource::Business {
            documents: vec![BusinessStateDocument {
                form: BusinessStateForm::Nec { data: state_income() },
            }],
        },
        IncomeDocument::Form1099K(_) => IncomeStateSource::Business {
            documents: vec![BusinessStateDocument {
                form: BusinessStateForm::K { data: withholding() },
            }],
        },
        IncomeDocument::Farm(_) => IncomeStateSource::Business { documents: business_forms() },
        _ => unreachable!(),
    }
}
fn saved(document_value: IncomeDocument) -> Interview {
    let draft = empty_return(FederalStatus::Single);
    let topic = topic_for(&document_value);
    let mut session = Interview::new(TEST_USER, draft);
    let request = save_topic(&session.draft, topic, vec![document(0, document_value)]);
    let reply = send(&mut session, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    session
}

#[test]
fn every_state_source_survives_save_avro_storage_and_reopening_without_changing_federal_income() {
    for mut doc in documents() {
        let federal = crate::interview::document_income(&doc);
        let source = source(&doc);
        assert!(source.apply(&mut doc));
        assert_eq!(crate::interview::document_income(&doc), federal);
        let topic = topic_for(&doc);
        let session = saved(doc);
        assert_eq!(session.draft.income[0].state_source, Some(source.clone()));
        let draft = roundtrip_native_draft(session.draft);
        let mut restored = Interview::new(TEST_USER, draft);
        let page = open_topic(&mut restored, topic);
        let rows = page_documents(&page);
        assert_eq!(rows.len(), 1);
        assert_eq!(IncomeStateSource::from_document(&rows[0].document), Some(source));
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
    let mut session = saved(IncomeDocument::W2(form));
    let id = session.draft.income[0].id;
    for details in [
        W2StateFields {
            states: vec![StateIncomeTax {
                state: TaxState::CA,
                payer_state_id: "".into(),
                income: Some(0),
                withholding: 0,
            }],
            ..Default::default()
        },
        W2StateFields::default(),
    ] {
        let mut form = w2(100_000);
        form.state_details = details;
        let expected = IncomeStateSource::from_document(&IncomeDocument::W2(form.clone()));
        let request = save_topic(
            &session.draft,
            TaxSection::Wages,
            vec![document(id, IncomeDocument::W2(form))],
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
                withholding: 300_000,
            },
            StateIncomeTax {
                state: TaxState::NJ,
                payer_state_id: "".into(),
                income: None,
                withholding: 0,
            },
        ],
        localities: localities(),
        ..Default::default()
    };
    let session = saved(IncomeDocument::W2(form.clone()));
    let Some(IncomeStateSource::Wage(source)) = &session.draft.income[0].state_source else {
        panic!("expected W-2 source")
    };
    assert_eq!(source.states, form.state_details.states);
    assert_eq!(source.localities[0].income, None);
    assert_eq!(source.localities[1].income, Some(0));
}

#[test]
fn clearing_payer_text_survives_storage_without_changing_nullable_income() {
    let mut form = w2(100_000);
    form.state_details = w2_source();
    let mut session = saved(IncomeDocument::W2(form));
    let page = open_topic(&mut session, TaxSection::Wages);
    let mut row = page_documents(&page)[0].clone();
    let IncomeDocument::W2(form) = &mut row.document else { panic!("expected W-2") };
    form.state_details.employer = TaxPayerIdentity::default();
    for state in &mut form.state_details.states {
        state.payer_state_id.clear();
    }
    let expected = form.state_details.clone();
    let request = save_topic(&session.draft, TaxSection::Wages, vec![row]);
    let reply = send(&mut session, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    session.draft = FederalDraft::from_avro(&session.draft.to_avro()).unwrap();
    let page = open_topic(&mut session, TaxSection::Wages);
    let IncomeDocument::W2(form) = &page_documents(&page)[0].document else {
        panic!("expected W-2")
    };
    assert_eq!(form.state_details, expected);
    assert_eq!(form.state_details.states[1].income, None);
    assert_eq!(form.state_details.localities[1].income, Some(0));
}

#[test]
fn invalid_state_source_is_rejected_before_any_draft_change() {
    let mut form = w2(100_000);
    let mut session = saved(IncomeDocument::W2(form.clone()));
    let before = session.draft.clone();
    let invalid = [
        W2StateFields { states: [states(), states()].concat(), ..Default::default() },
        W2StateFields {
            localities: vec![LocalIncomeTax {
                locality: String::new(),
                state: None,
                income: None,
                withholding: 1,
            }],
            ..Default::default()
        },
        W2StateFields {
            states: vec![StateIncomeTax {
                state: TaxState::CA,
                payer_state_id: "".into(),
                income: None,
                withholding: -1,
            }],
            ..Default::default()
        },
        W2StateFields {
            tipped_occupation_codes: vec![TippedOccupationCode { code: 1000 }],
            ..Default::default()
        },
    ];
    for details in invalid {
        form.state_details = details;
        let request = save_topic(
            &session.draft,
            TaxSection::Wages,
            vec![document(before.income[0].id, IncomeDocument::W2(form.clone()))],
        );
        let reply = send(&mut session, request);
        assert!(refusal(reply).contains("Check the state and local information"));
        assert_eq!(session.draft, before);
    }
    // D-2026-10-15-13: the printed state payer ID has no arbitrary text ceiling.
    form.state_details = W2StateFields {
        states: vec![StateIncomeTax {
            state: TaxState::CA,
            payer_state_id: "x".repeat(41),
            income: None,
            withholding: 0,
        }],
        ..Default::default()
    };
    let expected = IncomeStateSource::from_document(&IncomeDocument::W2(form.clone()));
    let request = save_topic(
        &session.draft,
        TaxSection::Wages,
        vec![document(before.income[0].id, IncomeDocument::W2(form))],
    );
    let reply = send(&mut session, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    assert_eq!(session.draft.income[0].state_source, expected);
    assert_eq!(session.draft.income[0].income, before.income[0].income);
}

#[test]
fn state_source_cannot_be_applied_to_a_different_form_kind() {
    let mut form = IncomeDocument::W2(w2(100_000));
    assert!(!IncomeStateSource::Withholding(withholding()).apply(&mut form));
    assert_eq!(IncomeStateSource::from_document(&form), None);
}

#[test]
fn supplemental_percentage_precision_survives_storage_without_conversion() {
    let original = InvestmentStateFields {
        payer_federal_id: "".into(),
        payer_address: "".into(),
        states: vec![],
        exempt_sources: vec![TaxExemptStateIncome {
            state: TaxState::CA,
            allocation: TaxExemptAllocation::Percentage { value: 27.43125 },
        }],
        government_obligation_percentage: Some(3.00001),
    };
    let mut session = saved(IncomeDocument::Int1099(Int1099 {
        state_details: original.clone(),
        ..native_int1099("".into(), 100_000)
    }));
    session.draft = FederalDraft::from_avro(&session.draft.to_avro()).unwrap();
    let page = open_topic(&mut session, TaxSection::InterestDividends);
    let IncomeDocument::Int1099(form) = &page_documents(&page)[0].document else {
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
        let mut session = saved(IncomeDocument::Int1099(Int1099 {
            state_details: original.clone(),
            ..native_int1099("".into(), 100_000)
        }));
        session.draft = FederalDraft::from_avro(&session.draft.to_avro()).unwrap();
        let page = open_topic(&mut session, TaxSection::InterestDividends);
        let IncomeDocument::Int1099(form) = &page_documents(&page)[0].document else {
            panic!("expected interest form")
        };
        assert_eq!(form.state_details, original);
    }
}

#[test]
fn government_obligation_percentage_keeps_absence_distinct_from_reported_zero() {
    for percentage in [None, Some(0.0)] {
        let mut session = saved(IncomeDocument::Int1099(Int1099 {
            payer: "".into(),
            taxable: 100_000,
            more: IntMore::default(),
            after_death: None,
            state_details: InvestmentStateFields {
                government_obligation_percentage: percentage,
                ..Default::default()
            },

            nominee: None,
            adjustments: None,
        }));
        session.draft = FederalDraft::from_avro(&session.draft.to_avro()).unwrap();
        assert_eq!(session.draft.income[0].state_source.is_some(), percentage.is_some());
        let page = open_topic(&mut session, TaxSection::InterestDividends);
        let IncomeDocument::Int1099(form) = &page_documents(&page)[0].document else {
            panic!("expected interest form")
        };
        assert_eq!(form.state_details.government_obligation_percentage, percentage);
    }
}

#[test]
fn invalid_supplemental_percentages_are_rejected_before_any_draft_change() {
    for mut doc in documents().into_iter().filter(|doc| {
        matches!(
            doc,
            IncomeDocument::Int1099(_) | IncomeDocument::Div1099(_) | IncomeDocument::Oid1099(_)
        )
    }) {
        let mut session = saved(doc.clone());
        let before = session.draft.clone();
        for value in [-0.01, 100.01, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for government_percentage in [false, true] {
                let details = if government_percentage {
                    InvestmentStateFields {
                        government_obligation_percentage: Some(value),
                        ..Default::default()
                    }
                } else {
                    InvestmentStateFields {
                        exempt_sources: vec![TaxExemptStateIncome {
                            state: TaxState::CA,
                            allocation: TaxExemptAllocation::Percentage { value },
                        }],
                        ..Default::default()
                    }
                };
                assert!(IncomeStateSource::Investment(details).apply(&mut doc));
                let request = save_topic(
                    &session.draft,
                    TaxSection::InterestDividends,
                    vec![document(before.income[0].id, doc.clone())],
                );
                // NaN is unequal to itself; decode the request without send's
                // equality assertion so the server can reject the value.
                let request = P::from_avro(&request.to_avro()).unwrap();
                let reply = session.handle(request).unwrap();
                assert!(refusal(reply).contains("Check the state and local information"));
                assert_eq!(session.draft, before);
            }
        }
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
    let source = include_str!("../../federal/state_return.rs");
    let page = &source[source.find("pub struct RetirementStateFields {").unwrap()..];
    let page = &page[..page.find("\n}").unwrap()];
    assert!(page.contains("pub taxes: StateLocalFields,"));
    assert!(!page.contains("statement"));
    assert!(!source.contains("Railroad Retirement Board Form RRB-1099-R"));
}
