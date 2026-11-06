use super::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::client::income_pages::open_document_center;

#[test]
fn broker_b_paper_real_matcher_answers_restart_and_existing_financial_owners() {
    let original = complete(FederalStatus::Single);
    let control = supported(&broker_manual_control(&original));
    let mut paper = broker_b_paper(&original);
    paper.box_8_closed_contracts = Some(40_000);
    paper.box_9_prior_open_contracts = Some(10_000);
    paper.box_10_open_contracts = Some(20_000);
    paper.box_11_aggregate = Some(50_000);
    paper.box_13_barter = Some(0);
    let mut session = Interview::new(TEST_USER, original);
    let id = admit_broker_paper(&mut session, BrokerPaper::Form1099B(paper.clone()));
    assert_eq!(broker_source(&session.draft, id).paper, BrokerPaper::Form1099B(paper.clone()));
    assert_eq!(broker_source(&session.draft, id).personal, BrokerPersonalRecords::default());
    assert!(estimate(&session.draft).tax.is_none(), "personal asset use is still unanswered");
    ordinary_broker_answers(&mut session, id, None);
    let result = supported(&session.draft);
    // $3,000 capital-sale gain plus the actual $500 contracts aggregate.
    // Form 6781 contributes $200 short and $300 long; withholding is $123.
    assert_eq!(value(&result, "capital_gain"), 3_500);
    assert_eq!(value(&result, "form_6781_short_term"), 200);
    assert_eq!(value(&result, "form_6781_long_term"), 300);
    for key in ["agi", "capital_gain", "form_6781_total"] {
        assert_eq!(value(&result, key), value(&control, key), "{key}");
    }
    // The tax return's line 25d owns total withholding.
    assert_eq!(return_line(&result, "25d"), 662_300);
    assert_eq!(return_line(&result, "25d"), return_line(&control, "25d"));
    assert_eq!(
        (result.tax, result.payments, result.refundable_credits, result.refund, result.balance),
        (
            control.tax,
            control.payments,
            control.refundable_credits,
            control.refund,
            control.balance
        )
    );
    let projected = crate::income_records::project(TEST_USER, 1, &session.draft).unwrap();
    let stored = projected
        .iter()
        .filter_map(|(_, record)| match record {
            TaxAppData::IncomeDocument { entry, .. }
                if matches!(entry.income, FederalIncome::Broker { .. }) =>
            {
                Some(entry)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(stored.len(), 1, "one durable document, not duplicated component records");
    assert!(stored[0].printed.is_none());
    assert!(stored[0].state_source.is_none());
    assert_eq!(
        stored[0].income,
        session.draft.income.iter().find(|row| row.id == id).unwrap().income
    );
    let saved = session.draft.clone();
    let mut cold = restart(saved.clone());
    assert_eq!(cold.draft, saved);
    let mut page = open_document_center(&mut cold);
    assert_eq!(*broker_b_editor(&mut page, id), paper);
    accepted(&send(&mut cold, page));
    assert_eq!(broker_source(&cold.draft, id), broker_source(&saved, id));
    assert_eq!(supported(&cold.draft).tax, result.tax);
}

#[test]
fn broker_da_quantity_correction_preserves_every_financial_figure() {
    let original = complete(FederalStatus::Single);
    let paper = broker_da_paper(&original);
    let mut session = Interview::new(TEST_USER, original);
    let id = admit_broker_paper(&mut session, BrokerPaper::Form1099DA(paper.clone()));
    ordinary_broker_answers(&mut session, id, None);
    let before = supported(&session.draft);
    assert_eq!(value(&before, "capital_gain"), 3_000);
    let personal = broker_source(&session.draft, id).personal.clone();
    let mut page = open_document_center(&mut session);
    let raw = broker_da_editor(&mut page, id);
    assert_eq!(*raw, paper);
    // Quantities retain their native fractions; money comes only from boxes 1f/1g.
    raw.box_1c_units = Some(0.987_654_321);
    raw.box_12a_transferred_units = Some(0.123_456_789);
    accepted(&send(&mut session, page));
    let mut after = supported(&session.draft);
    after.revision = before.revision;
    assert_eq!(after, before, "quantity changes cannot alter any financial result");
    assert_eq!(broker_source(&session.draft, id).personal, personal);
    let saved = session.draft.clone();
    let mut cold = restart(saved.clone());
    let mut page = open_document_center(&mut cold);
    let raw = broker_da_editor(&mut page, id);
    assert_eq!(raw.box_1c_units, Some(0.987_654_321));
    assert_eq!(raw.box_12a_transferred_units, Some(0.123_456_789));
    assert_eq!(raw.box_12b_transfer_date, None);
    assert_eq!(raw.recipient, paper.recipient);
    assert_eq!(raw.filer, paper.filer);
    assert_eq!(raw.account_number, "00001234-A");
    assert_eq!(raw.box_1a_asset_code, "000001");
    assert_eq!(raw.box_1d_acquired, Some(BrokerAcquisitionDate::Various));
    assert_eq!(raw.box_11b_transactions, Some(2));
    accepted(&send(&mut cold, page));
    assert_eq!(cold.draft, saved);
}

#[test]
fn broker_printed_corrections_retain_independent_basis_amt_and_source_boundaries() {
    let original = complete(FederalStatus::Single);
    let paper = broker_b_paper(&original);
    let mut session = Interview::new(TEST_USER, original);
    let id = admit_broker_paper(&mut session, BrokerPaper::Form1099B(paper));
    let request = broker_answers_page(&mut session, id, |answer| {
        if let DocumentAnswer::BrokerSaleRecords { value } = answer {
            value.use_kind = Some(SaleUse::Investment { wash_sale: 0 });
            value.basis = Some(BrokerBasisAnswer { amount: Some(175_000) });
            value.adjustment = crate::broker_document::BrokerSaleAdjustment::BasisCorrection;
            value.amt = SaleAmt::Different {
                record: CapitalSaleAmt { term: None, basis: 200_000, wash_sale: 0 },
            };
        }
    });
    accepted(&send(&mut session, request));
    let personal = broker_source(&session.draft, id).personal.clone();
    let before = supported(&session.draft);
    assert_eq!(value(&before, "capital_gain"), 2_750);
    assert_eq!(value(&before, &alloc::format!("form_8949_{id}_1_basis")), 1_500);
    assert_eq!(value(&before, &alloc::format!("form_8949_{id}_1_adjustment")), -250);
    let mut page = open_document_center(&mut session);
    let raw = broker_b_editor(&mut page, id);
    raw.corrected = true;
    raw.box_1e_basis = Some(175_000);
    accepted(&send(&mut session, page));
    assert_eq!(broker_source(&session.draft, id).personal, personal);
    let corrected = supported(&session.draft);
    assert_eq!(corrected.tax, before.tax);
    assert_eq!(value(&corrected, &alloc::format!("form_8949_{id}_1_basis")), 1_750);
    assert_eq!(value(&corrected, &alloc::format!("form_8949_{id}_1_adjustment")), 0);
    for (flag, key) in [
        (0, "BROKER_ORDINARY_NOT_CALCULATED"),
        (1, "BROKER_LOSS_STATEMENT_NEEDED"),
        (2, "BROKER_QOF_RECORDS_NEEDED"),
    ] {
        let mut page = open_document_center(&mut session);
        let form = &mut broker_b_editor(&mut page, id);
        form.box_2_ordinary = flag == 0;
        form.box_7_loss_not_allowed = flag == 1;
        form.box_3_qof = flag == 2;
        accepted(&send(&mut session, page));
        let result = estimate(&session.draft);
        assert!(result.tax.is_none(), "{key} must not become a completed zero");
        assert!(result.issues.iter().any(|issue| issue.display.key == key), "{key}");
        assert_eq!(broker_source(&session.draft, id).personal, personal);
    }
    let mut page = open_document_center(&mut session);
    broker_b_editor(&mut page, id).box_3_qof = false;
    accepted(&send(&mut session, page));
    assert_eq!(supported(&session.draft).tax, before.tax);
    let cold = restart(session.draft.clone());
    assert_eq!(broker_source(&cold.draft, id).personal, personal);
    let mut page = open_document_center(&mut session);
    broker_b_editor(&mut page, id).box_1e_basis = None;
    accepted(&send(&mut session, page));
    let result = estimate(&session.draft);
    assert!(result.tax.is_none());
    assert!(
        result
            .issues
            .iter()
            .any(|issue| { issue.display.key == "BROKER_CORRECTION_PRINTED_BASIS_NEEDED" })
    );
    assert_eq!(broker_source(&session.draft, id).personal, personal);
}

#[test]
fn broker_blank_basis_requires_an_explicit_conservative_answer() {
    let original = complete(FederalStatus::Single);
    let mut paper = broker_b_paper(&original);
    paper.box_1e_basis = None;
    paper.box_12_basis_reported = false;
    paper.form_8949_checkbox = Some(Form1099B8949Checkbox::E);
    let mut session = Interview::new(TEST_USER, original);
    let id = admit_broker_paper(&mut session, BrokerPaper::Form1099B(paper));
    let page = broker_answers_page(&mut session, id, |answer| {
        if let DocumentAnswer::BrokerSaleRecords { value } = answer {
            assert!(value.basis.is_none(), "blank paper does not establish a zero-basis choice");
        }
    });
    drop(page);
    assert!(broker_source(&session.draft, id).personal.sale.basis.is_none());
    assert!(estimate(&session.draft).tax.is_none());
    ordinary_broker_answers(&mut session, id, Some(None));
    assert_eq!(
        broker_source(&session.draft, id).personal.sale.basis,
        Some(BrokerBasisAnswer { amount: None })
    );
    let result = supported(&session.draft);
    assert_eq!(value(&result, "capital_gain"), 4_500);
    let cold = restart(session.draft.clone());
    assert_eq!(broker_source(&cold.draft, id), broker_source(&session.draft, id));
}

#[test]
fn broker_contracts_require_actual_box11_instead_of_manufacturing_the_aggregate() {
    let original = complete(FederalStatus::Single);
    let mut paper = broker_b_paper(&original);
    paper.box_1d_proceeds = None;
    paper.box_8_closed_contracts = Some(40_000);
    paper.box_9_prior_open_contracts = Some(10_000);
    paper.box_10_open_contracts = Some(20_000);
    paper.box_11_aggregate = None;
    let mut session = Interview::new(TEST_USER, original);
    let id = admit_broker_paper(&mut session, BrokerPaper::Form1099B(paper));
    let result = estimate(&session.draft);
    assert!(result.tax.is_none());
    assert!(
        result.issues.iter().any(|issue| issue.display.key == "BROKER_CONTRACTS_AGGREGATE_NEEDED")
    );
    let mut page = open_document_center(&mut session);
    let raw = broker_b_editor(&mut page, id);
    assert!(raw.box_11_aggregate.is_none());
    raw.box_11_aggregate = Some(50_000);
    accepted(&send(&mut session, page));
    ordinary_broker_answers(&mut session, id, None);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "capital_gain"), 500);
    assert_eq!(value(&result, "form_6781_short_term"), 200);
    assert_eq!(value(&result, "form_6781_long_term"), 300);
    let cold = restart(session.draft.clone());
    assert_eq!(broker_source(&cold.draft, id), broker_source(&session.draft, id));
}

#[test]
fn broker_barter_and_creator_receipts_keep_their_explicit_noncapital_boundaries() {
    let original = complete(FederalStatus::Single);
    let mut barter = broker_b_paper(&original);
    barter.box_1d_proceeds = None;
    barter.box_13_barter = Some(12_000);
    let mut creator = broker_da_paper(&original);
    creator.box_1f_proceeds = None;
    creator.box_11c_creator_proceeds = Some(34_000);
    for (paper, key) in [
        (BrokerPaper::Form1099B(barter), "BROKER_BARTER_RECEIPTS_NOT_CALCULATED"),
        (BrokerPaper::Form1099DA(creator), "BROKER_CREATOR_RECEIPTS_NOT_CALCULATED"),
    ] {
        let mut session = Interview::new(TEST_USER, original.clone());
        let id = admit_broker_paper(&mut session, paper.clone());
        let result = estimate(&session.draft);
        assert!(result.tax.is_none());
        assert!(!result.financial_source_scope_complete);
        assert!(result.issues.iter().any(|issue| issue.display.key == key), "{key}");
        assert_eq!(broker_source(&session.draft, id).paper, paper);
        let cold = restart(session.draft.clone());
        assert_eq!(broker_source(&cold.draft, id), broker_source(&session.draft, id));
    }
}

#[test]
fn broker_printed_quantity_correction_preserves_personal_basis() {
    let original = complete(FederalStatus::Single);
    let paper = broker_b_paper(&original);
    let mut session = Interview::new(TEST_USER, original);
    let id = admit_broker_paper(&mut session, BrokerPaper::Form1099B(paper));
    ordinary_broker_answers(&mut session, id, Some(Some(175_000)));
    let personal = broker_source(&session.draft, id).personal.clone();
    let before = supported(&session.draft);
    let mut page = open_document_center(&mut session);
    let raw = broker_b_editor(&mut page, id);
    raw.box_1a_quantity = Some(0.25);
    accepted(&send(&mut session, page));
    assert_eq!(broker_source(&session.draft, id).personal, personal);
    assert_eq!(supported(&session.draft).tax, before.tax);
}

#[test]
fn broker_printed_recipient_correction_runs_the_real_matcher_and_keeps_personal_records() {
    let original = complete(FederalStatus::Joint);
    let spouse = fixture_document_names(&original, -2).unwrap().recipient.name;
    let paper = broker_b_paper(&original);
    let mut session = Interview::new(TEST_USER, original);
    let id = admit_broker_paper(&mut session, BrokerPaper::Form1099B(paper));
    ordinary_broker_answers(&mut session, id, Some(Some(175_000)));
    let personal = broker_source(&session.draft, id).personal.clone();
    let before = supported(&session.draft);
    let mut page = open_document_center(&mut session);
    broker_b_editor(&mut page, id).recipient.name = spouse.clone();
    accepted(&send(&mut session, page));
    let entry = session.draft.income.iter().find(|entry| entry.id == id).unwrap();
    assert_eq!(entry.owner, crate::DocumentOwner::Spouses);
    let source = broker_source(&session.draft, id);
    assert_eq!(source.assigned, DocumentRecipient::Spouse);
    assert_eq!(source.personal, personal);
    assert_eq!(source.printed_names().recipient.name, spouse);
    let result = supported(&session.draft);
    assert_eq!(result.tax, before.tax);
    assert_eq!(result.payments, before.payments);
    let cold = restart(session.draft.clone());
    assert_eq!(broker_source(&cold.draft, id), broker_source(&session.draft, id));
}

#[test]
fn broker_community_sale_contracts_and_withholding_share_without_dividing_the_paper() {
    let original = broker_community_return();
    let mut paper = broker_b_paper(&original);
    paper.box_11_aggregate = Some(50_000);
    paper.box_4_withholding = Some(12_000);
    let mut session = Interview::new(TEST_USER, original);
    let id = admit_broker_paper(&mut session, BrokerPaper::Form1099B(paper.clone()));
    let request = broker_answers_page(&mut session, id, |answer| match answer {
        DocumentAnswer::Community { value } => *value = CommunityShare::Community,
        DocumentAnswer::BrokerSaleRecords { value } => {
            value.use_kind = Some(SaleUse::Investment { wash_sale: 0 });
        }
        DocumentAnswer::BrokerContractsTreatment { value } => *value = Some(false),
        _ => {}
    });
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "capital_gain"), 1_750);
    assert_eq!(value(&result, "form_6781_short_term"), 100);
    assert_eq!(value(&result, "form_6781_long_term"), 150);
    assert_eq!(return_line(&result, "25b"), 6_000);
    assert_eq!(broker_source(&session.draft, id).paper, BrokerPaper::Form1099B(paper));
    let cold = restart(session.draft.clone());
    assert_eq!(broker_source(&cold.draft, id), broker_source(&session.draft, id));
    assert_eq!(supported(&cold.draft).tax, result.tax);
}

#[test]
fn broker_entry_corrections_update_paper_without_stored_printed_overrides() {
    for digital in [false, true] {
        let draft = complete(FederalStatus::Single);
        let paper = if digital {
            let mut paper = broker_da_paper(&draft);
            paper.box_1f_proceeds = Some(100_000);
            BrokerPaper::Form1099DA(paper)
        } else {
            let mut paper = broker_b_paper(&draft);
            paper.box_1d_proceeds = Some(100_000);
            BrokerPaper::Form1099B(paper)
        };
        let mut session = Interview::new(TEST_USER, draft);
        let id = admit_broker_paper(&mut session, paper);
        let page = broker_answers_page(&mut session, id, |answer| {
            if let DocumentAnswer::BrokerSaleRecords { value } = answer {
                value.use_kind = Some(SaleUse::Investment { wash_sale: 2_500 });
                value.basis = Some(BrokerBasisAnswer { amount: Some(175_000) });
                value.market_discount = 10_000;
            }
        });
        accepted(&send(&mut session, page));
        let source = broker_source(&session.draft, id);
        assert!(source.personal.sale.basis.is_none(), "a paper-entry correction is not code B");
        let amounts = match &source.paper {
            BrokerPaper::Form1099B(form) => {
                (form.box_1e_basis, form.box_1g_wash_sale, form.box_1f_market_discount)
            }
            BrokerPaper::Form1099DA(form) => {
                (form.box_1g_basis, form.box_1i_wash_sale, form.box_1h_market_discount)
            }
        };
        assert_eq!(amounts, (Some(175_000), Some(2_500), Some(10_000)));
        let saved = source.clone();
        session = restart(session.draft);
        assert_eq!(broker_source(&session.draft, id), &saved);
        let mut page = open_document_center(&mut session);
        if digital {
            let form = broker_da_editor(&mut page, id);
            form.box_1g_basis = Some(200_000);
            form.box_1i_wash_sale = Some(0);
            form.box_1h_market_discount = Some(0);
        } else {
            let form = broker_b_editor(&mut page, id);
            form.box_1e_basis = Some(200_000);
            form.box_1g_wash_sale = Some(0);
            form.box_1f_market_discount = Some(0);
        }
        accepted(&send(&mut session, page));
        let source = broker_source(&session.draft, id);
        assert_eq!(source.personal, saved.personal);
        let sale = source.projection().sale.unwrap();
        assert_eq!(sale.basis, Some(200_000));
        assert_eq!(sale.market_discount, 0);
        assert!(matches!(sale.use_kind, SaleUse::Investment { wash_sale: 0 }));
        supported(&session.draft);
    }
}
