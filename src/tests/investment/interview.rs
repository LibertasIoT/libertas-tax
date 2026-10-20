// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::*;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::document;
use super::support::client::income_pages::{
    catalog_preview_document, open_topic, page_documents, save_documents, save_topic,
};
use crate::tests::cross::interview::{
    assert_round_trip, business, continuing_page_asset, earlier_installment, exchange_fold,
    related_exchange_kind, rental_page, save_businesses, save_more_credits, save_rentals,
    stored_business, stored_rentals,
};

#[test]
fn us_obligations_interest_reopens_in_box_3() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let int = Int1099 {
        more: IntMore {
            us_obligations: 25_000,
            special: InterestSpecial::NoneOfThese,
            ..Default::default()
        },
        ..native_int1099("Treasury Direct".into(), 10_000)
    };
    // "Amount on the return" (i1040gi): box 1 ($100.00) plus box 3 ($250.00).
    let reply = send(&mut session, catalog_preview_document(IncomeDocument::Int1099(int.clone())));
    assert!(matches!(reply, Reply::Response(P::IncomePreview { preview: Some(35_000), .. })));
    send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::Int1099(int.clone()))]),
    );
    let FederalIncome::Interest { data } = &session.draft.income[1].income else { unreachable!() };
    assert_eq!(data.us_obligations, 25_000);
    assert_eq!(data.taxable, 10_000);
    // Reopening shows the same box 3 amount.
    let mut session = restart(session.draft);
    let documents =
        page_documents(&open_topic(&mut session, TaxSection::InterestDividends)).to_vec();
    assert_eq!(documents[0].document, IncomeDocument::Int1099(int));
}

#[test]
fn oid_document_maps_onto_federal_oid() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let oid = Oid1099 {
        more: OidMore {
            withholding: 1_000,
            market_discount: 2_000,
            acquisition_premium: 3_000,
            treasury_oid: -1_000,
            bond_premium: 500,
            exempt: 6_000,
            private_activity: 1_500,
            special: crate::OidSpecial::NoneOfThese,
            ..Default::default()
        },
        ..native_oid1099("Synthetic Broker".into(), 40_000, 5_000)
    };
    send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::Oid1099(oid.clone()))]),
    );
    let FederalIncome::Oid { data } = &session.draft.income[1].income else { unreachable!() };
    assert_eq!(
        *data,
        FederalOid {
            oid: 40_000,
            other_periodic_interest: 5_000,
            withholding: 1_000,
            market_discount: 2_000,
            acquisition_premium: 3_000,
            exempt: 6_000,
            bond_premium: 500,
            private_activity: 1_500,
            treasury_oid: -1_000,
            ..Default::default()
        }
    );
    // Reopening rebuilds the exact same document.
    let mut session = restart(session.draft);
    let documents =
        page_documents(&open_topic(&mut session, TaxSection::InterestDividends)).to_vec();
    assert_eq!(documents[0].document, IncomeDocument::Oid1099(oid));
}

// Unit M3-P3: investment sales, unemployment/state refunds, and prizes,
// royalties and equipment rent.
#[test]
fn sale_maps_onto_federal_sale_with_its_box() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let sale = Sale1099B {
        digital_asset: false,
        market_discount: 0,
        several_sales: false,
        state_details: Default::default(),
        description: "100 sh ABC Corp".into(),
        term: Some(GainTerm::Long),
        proceeds: 500_000,
        basis: Some(300_000),
        reporting: BasisReporting::NotReported,
        use_kind: SaleUse::Investment { wash_sale: 0 },
        collectible: false,
        withholding: 10_000,
        adjustment: SaleAdjustment::SellingExpenses { amount: 5_000 },
        special: crate::SaleSpecial::NoneOfThese,
    };
    // Proceeds $5,000 - basis $3,000 - selling expenses $50 = $1,950 gain.
    let reply =
        send(&mut session, catalog_preview_document(IncomeDocument::Sale1099B(sale.clone())));
    assert!(matches!(reply, Reply::Response(P::IncomePreview { preview: Some(195_000), .. })));
    send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::Sale1099B(sale))]),
    );
    let FederalIncome::Sale { data } = &session.draft.income[1].income else { unreachable!() };
    assert_eq!(
        *data,
        FederalSale {
            withholding: 10_000,
            adjustment: SaleAdjustment::SellingExpenses { amount: 5_000 },
            basis_reporting: BasisReporting::NotReported,
            ..native_sale(GainTerm::Long, 500_000, Some(300_000))
        }
    );
    // 2026-10-16, Source identity and association: document_entry retains the
    // accepted description as the entry label; sale_item restores that user text.
    let mut session = restart(session.draft);
    let documents = page_documents(&open_topic(&mut session, TaxSection::InvestmentSales)).to_vec();
    assert_eq!(
        documents[0].document,
        IncomeDocument::Sale1099B(Sale1099B {
            digital_asset: false,
            market_discount: 0,
            several_sales: false,
            state_details: Default::default(),
            description: "100 sh ABC Corp".into(),
            term: Some(GainTerm::Long),
            proceeds: 500_000,
            basis: Some(300_000),
            reporting: BasisReporting::NotReported,
            use_kind: SaleUse::Investment { wash_sale: 0 },
            collectible: false,
            withholding: 10_000,
            adjustment: SaleAdjustment::SellingExpenses { amount: 5_000 },
            special: crate::SaleSpecial::NoneOfThese,
        })
    );
}

#[test]
fn wash_sale_above_the_loss_is_rejected() {
    let original = complete(FederalStatus::Single);
    // Proceeds $1,000, basis $4,000: a $3,000 loss.
    let sale_with_wash = |wash_sale: i64| Sale1099B {
        digital_asset: false,
        market_discount: 0,
        several_sales: false,
        state_details: Default::default(),
        description: "".into(),
        term: Some(GainTerm::Short),
        proceeds: 100_000,
        basis: Some(400_000),
        reporting: BasisReporting::ReportedToIrs,
        use_kind: SaleUse::Investment { wash_sale },
        collectible: false,
        withholding: 0,
        adjustment: SaleAdjustment::None,
        special: crate::SaleSpecial::NoneOfThese,
    };
    // Boundary: a wash sale exactly equal to the loss is accepted.
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = send(
        &mut session,
        save_documents(
            &original,
            vec![document(0, IncomeDocument::Sale1099B(sale_with_wash(300_000)))],
        ),
    );
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    // One cent more than the loss is refused by name, on save and preview.
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = send(
        &mut session,
        save_documents(
            &original,
            vec![document(0, IncomeDocument::Sale1099B(sale_with_wash(300_001)))],
        ),
    );
    rejected_with(reply, "SALE_WASH_SALE_ABOVE_LOSS");
    assert_eq!(session.draft, original);
    assert!(!session.dirty);
    let reply = send(
        &mut session,
        catalog_preview_document(IncomeDocument::Sale1099B(sale_with_wash(300_001))),
    );
    rejected_with(reply, "SALE_WASH_SALE_ABOVE_LOSS");
}

#[test]
fn saving_interest_and_dividends_keeps_a_form_1116_claim() {
    // A Form 1116 claim chosen on More credits is not the payee-statement
    // election: saving the Interest and dividends topic (which owns only the
    // 1099 box 6/7 amounts) leaves it, its sources and its answers as they
    // were, whether the 1099s show foreign tax or none.
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let ordinary = ForeignTaxClaim::Ordinary {
        sources: vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Synthetic foreign fund".into(),
            category: ForeignTaxCategory::Passive {
                sales: vec![],
                withholding: ForeignWithholding::Below15,
                qualified: 0,
            },
            income: 100_000,
            tax: 10_000,
        }],
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::Unreviewed,
        amt_limitation: ForeignAmtLimitation::Normal,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment: ForeignRateAdjustment::Exception,
    };
    let request = save_more_credits(
        &session.draft,
        MoreCredits {
            credits: vec![MoreCredit::ForeignTax(ordinary.clone().try_into().unwrap())],

            credit_income_sources: Vec::new(),
        },
    );
    send(&mut session, request);
    let int = |foreign_tax| Int1099 {
        more: IntMore { foreign_tax, special: InterestSpecial::NoneOfThese, ..Default::default() },
        ..native_int1099("Synthetic Bank".into(), 50_000)
    };
    for foreign_tax in [0, 20_000] {
        let request = save_documents(
            &session.draft,
            vec![document(0, IncomeDocument::Int1099(int(foreign_tax)))],
        );
        let reply = send(&mut session, request);
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        assert_eq!(
            session.draft.credits.as_ref().unwrap().foreign_tax,
            ordinary,
            "foreign tax {foreign_tax}"
        );
        // The box 6 amount is stored on the document itself (row 191 of the
        // design-improvements log): reopening shows it with the Form 1116
        // claim in force, where it used to read as zero.
        let mut reopened = restart(session.draft.clone());
        let page = open_topic(&mut reopened, TaxSection::InterestDividends);
        assert_eq!(
            page_documents(&page)[0].document,
            IncomeDocument::Int1099(int(foreign_tax)),
            "reopened with foreign tax {foreign_tax}"
        );
    }
}

#[test]
fn capital_loss_carryover_kind_reaches_schedule_d() {
    // The 2025 return (capital_history): AGI $10,000, deductions $12,000,
    // Schedule D line 7 -$5,000, line 15 -$10,000, line 21 $3,000 deducted.
    // 2025 Capital Loss Carryover Worksheet: line 1 taxable income -$2,000;
    // line 2 $3,000; line 3 $1,000; line 4 $1,000; line 5 $5,000; line 7
    // $1,000; line 8 short-term carryover $4,000; line 9 $10,000; line 11
    // $0; line 13 long-term carryover $10,000. 2026 Schedule D lines 6 and
    // 14: -$4,000 and -$10,000; line 16 -$14,000; line 21 and Form 1040
    // line 7a: -$3,000 (single). 2026 taxable income $60,000 - $3,000 -
    // $16,100 = $40,900 is positive, so the whole $3,000 is used,
    // short-term first: $1,000 short-term and $10,000 long-term carry to
    // 2027.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        vec![document(0, IncomeDocument::CapitalLossCarryover(capital_history().into()))],
    );
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.screening.as_ref().unwrap().capital_history, vec![capital_history()]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "7a"), -300_000);
    assert_eq!(value(&r, "capital_short_carryforward"), 1_000);
    assert_eq!(value(&r, "capital_long_carryforward"), 10_000);
    // It reopens as the kind; three returns are refused by name.
    assert_eq!(
        page_documents(&open_topic(&mut s, TaxSection::InvestmentSales)),
        &[IncomeDocumentItem {
            refund_management_needed: Some(false),
            ..document(0, IncomeDocument::CapitalLossCarryover(capital_history().into()))
        }]
    );
    let before = s.draft.clone();
    rejected_with(
        send(
            &mut s,
            save_topic(
                &before,
                TaxSection::InvestmentSales,
                vec![
                    document(0, IncomeDocument::CapitalLossCarryover(capital_history().into()));
                    3
                ],
            ),
        ),
        "CAPITAL_LOSS_CARRYOVER_LIMIT",
    );
    assert_eq!(s.draft, before);
}

// ---- T3-6252 installment sales on the pages ----

fn page_installment(
    date: u32,
    proceeds: i64,
    assumed_debt: i64,
    payments: i64,
) -> crate::AssetDisposal {
    crate::AssetDisposal::Disposed {
        date,
        how: crate::DisposalKind::Sold {
            livestock: crate::Livestock::No,
            proceeds,
            selling_expenses: 0,
            payment: crate::SalePayment::Installment {
                assumed_debt,
                payments,
                interest: crate::NoteInterest::Adequate,
                special: false,
            },
            buyer: crate::Buyer::Unrelated,
        },
    }
}

#[test]
fn installment_asset_fold_round_trips_and_rejects_payments_above_the_price() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut b = business(15_000_000);
    // Boundary: loans taken over plus principal equal to the price.
    b.assets =
        vec![continuing_page_asset(page_installment(20260715, 2_000_000, 1_500_000, 500_000))];
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let (_, data) = stored_business(&session);
    assert_eq!(data.assets[0].disposal, page_installment(20260715, 2_000_000, 1_500_000, 500_000));
    assert_round_trip(&mut session, &[b]);
    // One cent more is rejected by the key both sides share.
    let before = session.draft.clone();
    let mut over = business(15_000_000);
    over.assets =
        vec![continuing_page_asset(page_installment(20260715, 2_000_000, 1_500_000, 500_001))];
    rejected_with(save_businesses(&mut session, vec![over]), "INSTALLMENT_PAID_ABOVE_PRICE");
    assert_eq!(session.draft, before);
}

#[test]
fn installment_rental_fold_round_trips_and_rejects_payments_above_the_price() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let sale = |payments| crate::RentalSale::Sold {
        date: 20260815,
        price: 40_000_000,
        land_price: 8_000_000,
        selling_expenses: 0,
        payment: crate::SalePayment::Installment {
            assumed_debt: 0,
            payments,
            interest: crate::NoteInterest::Adequate,
            special: false,
        },
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
    let mut r = rental_page(1_200_000);
    r.sale = sale(10_000_000);
    accepted(&save_rentals(&mut session, vec![r.clone()]));
    let (_, _, data) = stored_rentals(&session).remove(0);
    assert_eq!(data.sale, sale(10_000_000));
    let mut again = restart(session.draft.clone());
    let items = page_documents(&open_topic(&mut again, TaxSection::Rentals)).to_vec();
    assert_eq!(items[0].document, IncomeDocument::Rental(r));
    let before = session.draft.clone();
    let mut r = rental_page(1_200_000);
    r.sale = sale(40_000_001);
    rejected_with(save_rentals(&mut session, vec![r]), "INSTALLMENT_PAID_ABOVE_PRICE");
    assert_eq!(session.draft, before);
}

#[test]
fn installment_payments_kind_reaches_schedule_d_and_round_trips() {
    // $60,000 W-2. Gross profit $60,000 of a $150,000 contract price (40%),
    // $50,000 received before 2026 and $25,000 in 2026: $10,000 long-term
    // gain; AGI $70,000.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let sale = earlier_installment(crate::PropertyCharacter::LongTermCapital, 5_000_000, 2_500_000);
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        vec![document(0, IncomeDocument::InstallmentPayments(sale.page()))],
    );
    accepted(&send(&mut s, request));
    assert_eq!(
        s.draft.screening.as_ref().unwrap().installment_payments,
        vec![crate::EarlierInstallment { owner: Owner::Taxpayer, sale: sale.clone() }]
    );
    let r = supported(&s.draft);
    assert_eq!(value(&r, "installment_1_gain"), 10_000);
    assert_eq!(value(&r, "agi"), 70_000);
    // It reopens as the kind; the section 1231 lookback kind is not offered
    // for investment property; saving unchanged keeps the record.
    let reopened = open_topic(&mut s, TaxSection::InvestmentSales);
    assert_eq!(
        page_documents(&reopened),
        &[IncomeDocumentItem {
            refund_management_needed: Some(false),
            ..document(0, IncomeDocument::InstallmentPayments(sale.page()))
        }]
    );
    let lookback = libertas_macros::variant_index!(IncomeDocument::BusinessPropertyLosses) as i32;
    let P::SaveIncomes { allowed_documents, .. } = &reopened else { unreachable!() };
    assert!(!allowed_documents.contains(&lookback));
    let before = s.draft.screening.clone();
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        page_documents(&reopened).to_vec(),
    );
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.screening, before);
    // A business-property sale needs the lookback: the page then offers it
    // and writes it for the whole return.
    let building = earlier_installment(
        crate::PropertyCharacter::BusinessProperty {
            used: crate::PropertyUse::Business,
            unrecaptured: 0,
        },
        0,
        2_500_000,
    );
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        vec![document(0, IncomeDocument::InstallmentPayments(building.page()))],
    );
    accepted(&send(&mut s, request));
    unsupported(&s.draft, "Section 1231 lookback");
    let P::SaveIncomes { allowed_documents, .. } = open_topic(&mut s, TaxSection::InvestmentSales)
    else {
        unreachable!()
    };
    assert!(allowed_documents.contains(&lookback));
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        vec![
            document(0, IncomeDocument::InstallmentPayments(building.page())),
            document(
                0,
                IncomeDocument::BusinessPropertyLosses(crate::BusinessPropertyLosses {
                    regular: vec![],
                    amt: vec![],
                }),
            ),
        ],
    );
    accepted(&send(&mut s, request));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "form4797_line8"), 10_000);
}

#[test]
fn installment_payments_kind_rules_reject_on_both_sides() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    // Boundary: $50,000 + $100,000 equals the $150,000 contract price.
    let full =
        earlier_installment(crate::PropertyCharacter::LongTermCapital, 5_000_000, 10_000_000);
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        vec![document(0, IncomeDocument::InstallmentPayments(full.page()))],
    );
    accepted(&send(&mut s, request));
    let before = s.draft.clone();
    let over =
        earlier_installment(crate::PropertyCharacter::LongTermCapital, 5_000_000, 10_000_001);
    rejected_with(
        send(
            &mut s,
            save_topic(
                &before,
                TaxSection::InvestmentSales,
                vec![document(0, IncomeDocument::InstallmentPayments(over.page()))],
            ),
        ),
        "INSTALLMENT_PAID_ABOVE_CONTRACT",
    );

    let over = earlier_installment(crate::PropertyCharacter::LongTermCapital, i64::MAX, i64::MAX);
    rejected_with(
        send(
            &mut s,
            save_topic(
                &before,
                TaxSection::InvestmentSales,
                vec![document(0, IncomeDocument::InstallmentPayments(over.page()))],
            ),
        ),
        "INSTALLMENT_PAID_ABOVE_CONTRACT",
    );
    assert_eq!(
        crate::SalePayment::Installment {
            assumed_debt: i64::MAX,
            payments: i64::MAX,
            interest: crate::NoteInterest::Adequate,
            special: false,
        }
        .issue(100),
        Some(
            crate::validation::ValidationIssue::new("INSTALLMENT_PAID_ABOVE_PRICE")
                .at_field("payments")
        )
    );
    let mut profit = earlier_installment(crate::PropertyCharacter::LongTermCapital, 0, 0);
    profit.gross_profit = 15_000_001;
    rejected_with(
        send(
            &mut s,
            save_topic(
                &before,
                TaxSection::InvestmentSales,
                vec![document(0, IncomeDocument::InstallmentPayments(profit.page()))],
            ),
        ),
        "INSTALLMENT_PROFIT_ABOVE_CONTRACT",
    );
    assert_eq!(s.draft, before);
}

// A saved savings-bond row follows its person: a person who leaves the return
// takes the row; a record left with nobody goes, and its page row says so.
#[test]
fn savings_bond_rows_leave_with_their_person() {
    use crate::tests::investment::investment::{bond_record, bond_return, bond_student};
    let mut d = bond_return(FederalStatus::Single, 60_000, 3_000);
    d.checklist.income |= IncomeTopics::WAGES.0 | IncomeTopics::INTEREST_DIVIDENDS.0;
    let child = d.dependents[0].id;
    add(
        &mut d,
        bond_record(
            Owner::Taxpayer,
            12_000,
            3_000,
            20100601,
            vec![bond_student(child, 5_000, 0), bond_student(-1, 4_000, 0)],
        ),
    );
    let mut kept = d.clone();
    let first = crate::interview::prune(&mut kept);
    assert!(first.removed.is_empty(), "{:?}", first.removed);
    // The dependent leaves the return: that row goes, the filer's stays.
    d.dependents.clear();
    let pruned = crate::interview::prune(&mut d);
    assert!(!pruned.removed.is_empty());
    let FederalIncome::SavingsBonds { data } = &d.income.last().unwrap().income else {
        unreachable!()
    };
    assert_eq!(data.students.len(), 1);
    assert_eq!(data.students[0].person, -1);
    // A record whose only person is gone is no record.
    let FederalIncome::SavingsBonds { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.students[0].person = -2;
    crate::interview::prune(&mut d);
    assert!(d.income.iter().all(|e| !matches!(e.income, FederalIncome::SavingsBonds { .. })));
}

#[test]
fn section_1256_carryback_round_trips_and_checks_its_rules() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let kind = |amount| {
        IncomeDocument::Section1256Carryback(crate::Section1256Carryback {
            amount,
            injured_owners: None,
        })
    };
    let request =
        save_topic(&s.draft.clone(), TaxSection::InvestmentSales, vec![document(0, kind(800_000))]);
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.screening.as_ref().unwrap().section_1256_carryback, Some(800_000));
    // It is a data answer, not an income entry, and reopens as the kind.
    assert!(s.draft.income.iter().all(|e| !matches!(e.income, FederalIncome::Section1256 { .. })));
    let reopened = open_topic(&mut s, TaxSection::InvestmentSales);
    assert_eq!(
        page_documents(&reopened).iter().map(|item| item.document.clone()).collect::<Vec<_>>(),
        vec![kind(800_000)]
    );
    let before = s.draft.clone();
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        page_documents(&reopened).to_vec(),
    );
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.screening, before.screening);
    // A second one is refused on the server with the client rule's key.
    rejected_with(
        send(
            &mut s,
            save_topic(
                &before,
                TaxSection::InvestmentSales,
                vec![document(0, kind(800_000)), document(0, kind(100_000))],
            ),
        ),
        "SECTION_1256_CARRYBACK_TWICE",
    );
    // Zero is below the client range (min 1) and refused on the server too.
    let model = [
        include_str!("../../federal/federal_model.rs"),
        include_str!("../../investment/section1256.rs"),
    ]
    .concat();
    let amount = &model[model.find("pub struct Section1256Carryback").unwrap()..];
    assert!(amount.contains("#[libertas_number(min = 1)]"));
    assert!(matches!(
        send(&mut s, save_topic(&before, TaxSection::InvestmentSales, vec![document(0, kind(0))])),
        Reply::Response(P::Problem { .. })
    ));
    assert_eq!(s.draft.screening, before.screening);
    // Removing the kind removes the election.
    let request = save_topic(&s.draft.clone(), TaxSection::InvestmentSales, vec![]);
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.screening.as_ref().unwrap().section_1256_carryback, None);
}

#[test]
fn investment_sales_offers_the_contracts_kinds_and_search_names_them() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let page = open_topic(&mut s, TaxSection::InvestmentSales);
    let P::SaveIncomes { allowed_documents, .. } = &page else { unreachable!() };
    for kind in [
        libertas_macros::variant_index!(IncomeDocument::Contracts1099B) as i32,
        libertas_macros::variant_index!(IncomeDocument::Section1256Carryback) as i32,
    ] {
        assert!(allowed_documents.contains(&kind), "{kind}");
    }
    let protocol = include_str!("../../core/protocol.rs");
    assert!(protocol.contains("Form 6781 Part I, 60/40 treatment"));
    assert!(protocol.contains("straddles (Form 6781 Part II)"));
    let strings = APP_STRINGS.iter().find(|(key, _)| *key == "TAX_NAVIGATION_PROMPT").unwrap().1;
    assert!(strings.contains(
        "Futures, section 1256 contracts, index options and Form 6781 Part I belong to Investment sales; straddles belong to Less common income."
    ));
}

#[test]
fn exchange_fold_maps_and_its_rules_reject_on_both_sides() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut r = rental_page(1_200_000);
    r.sale = crate::RentalSale::Exchanged(exchange_fold());
    accepted(&save_rentals(&mut session, vec![r.clone()]));
    let (_, _, data) = stored_rentals(&session).remove(0);
    assert_eq!(data.sale, r.sale);
    let mut again = restart(session.draft.clone());
    let items = page_documents(&open_topic(&mut again, TaxSection::Rentals)).to_vec();
    assert_eq!(items[0].document, IncomeDocument::Rental(r));
    let before = session.draft.clone();
    // Each page rule, on both sides: the client rule is in the schema and the
    // server twin rejects the save.
    let mut changed = |change: fn(&mut crate::RentalExchange), key: &str| {
        let mut x = exchange_fold();
        change(&mut x);
        let mut r = rental_page(1_200_000);
        r.sale = crate::RentalSale::Exchanged(x);
        rejected_with(save_rentals(&mut session, vec![r]), key);
    };
    changed(|x| x.identified = 20260911, "EXCHANGE_IDENTIFIED_AFTER_RECEIVED");
    changed(|x| x.replacement.land = 52_000_001, "EXCHANGE_LAND_ABOVE_VALUE");
    changed(|x| x.replacement.land = 52_000_000, "EXCHANGE_RENTAL_WITHOUT_BUILDING");
    changed(
        |x| {
            x.replacement.used = crate::ReplacementUse::Rental {
                kind: RentalPropertyType::SingleFamily,
                in_service: 20260909,
            }
        },
        "EXCHANGE_RENTED_BEFORE_RECEIVED",
    );
    changed(|x| x.land_value = 90_000_000, "EXCHANGE_LAND_VALUE_ABOVE_TOTAL");
    // After April 15, 2027 the extension question is required; on the day it
    // is not.
    changed(
        |x| {
            x.transferred = 20261120;
            x.identified = 20261220;
            x.received = 20270420;
            x.replacement.used = crate::ReplacementUse::Rental {
                kind: RentalPropertyType::SingleFamily,
                in_service: 20270501,
            };
        },
        "EXCHANGE_SAY_WHETHER_YOU_EXTEND",
    );
    // An exchange before the building was first available to rent (the
    // fixture building's March 1, 2021).
    let mut r = rental_page(1_200_000);
    r.building.in_service = 20260901;
    r.sale = crate::RentalSale::Exchanged(exchange_fold());
    rejected_with(save_rentals(&mut session, vec![r]), "RENTAL_EXCHANGED_BEFORE_IN_SERVICE");
    assert_eq!(session.draft, before);
}

#[test]
fn related_exchange_kind_reaches_schedule_d_and_round_trips() {
    // $60,000 W-2. Land held as an investment, exchanged with a related person
    // in 2024 and sold by them 2026-04-30: the deferred $90,000 is long-term
    // gain on Schedule D line 11; AGI $150,000.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let exchange = related_exchange_kind(20260430);
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        vec![document(0, IncomeDocument::RelatedExchange(exchange.clone()))],
    );
    accepted(&send(&mut s, request));
    assert_eq!(
        s.draft.screening.as_ref().unwrap().related_exchanges,
        vec![crate::EarlierRelatedExchange { owner: Owner::Taxpayer, exchange: exchange.clone() }]
    );
    let r = supported(&s.draft);
    assert_eq!(value(&r, "related_exchange_1_gain"), 90_000);
    assert_eq!(value(&r, "agi"), 150_000);
    let reopened = open_topic(&mut s, TaxSection::InvestmentSales);
    assert_eq!(
        page_documents(&reopened),
        &[IncomeDocumentItem {
            refund_management_needed: Some(false),
            ..document(0, IncomeDocument::RelatedExchange(exchange.clone()))
        }]
    );
    let kind = libertas_macros::variant_index!(IncomeDocument::RelatedExchange) as i32;
    let P::SaveIncomes { allowed_documents, .. } = &reopened else { unreachable!() };
    assert!(allowed_documents.contains(&kind));
    let before = s.draft.screening.clone();
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        page_documents(&reopened).to_vec(),
    );
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.screening, before);
    // Saving the page without it removes the record.
    let request = save_topic(&s.draft.clone(), TaxSection::InvestmentSales, vec![]);
    accepted(&send(&mut s, request));
    assert!(s.draft.screening.as_ref().unwrap().related_exchanges.is_empty());
    // A business-property record needs the section 1231 lookback, which the
    // page then offers.
    let lookback = libertas_macros::variant_index!(IncomeDocument::BusinessPropertyLosses) as i32;
    let mut business = related_exchange_kind(20260430);
    business.kind = crate::PropertyCharacter::BusinessProperty {
        used: crate::PropertyUse::Business,
        unrecaptured: 0,
    };
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        vec![document(0, IncomeDocument::RelatedExchange(business))],
    );
    accepted(&send(&mut s, request));
    unsupported(&s.draft, "Section 1231 lookback");
    let P::SaveIncomes { allowed_documents, .. } = open_topic(&mut s, TaxSection::InvestmentSales)
    else {
        unreachable!()
    };
    assert!(allowed_documents.contains(&lookback));
}

#[test]
fn related_exchange_kind_rules_reject_on_both_sides() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = s.draft.clone();
    // The client rule (unreachable inside the date ranges) and its server twin.
    let mut early = related_exchange_kind(20260101);
    early.last_transfer = 20260101;
    assert_eq!(early.rule(), Some("RELATED_EXCHANGE_DISPOSED_BEFORE_TRANSFER"));
    // A spouse's record only on a joint return.
    let mut spouse = document(0, IncomeDocument::RelatedExchange(related_exchange_kind(20260430)));
    spouse.owner = Owner::Spouse;
    let request = save_topic(&before, TaxSection::InvestmentSales, vec![spouse]);
    rejected_with(
        current_income_refusal(
            "income-source-2924",
            &mut s,
            request,
            "MESSAGE_RELATED_EXCHANGE_YOUR_SPOUSE_S_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
            "[documents/documents/0/owner]",
        ),
        "MESSAGE_RELATED_EXCHANGE_YOUR_SPOUSE_S_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
    );
    assert_eq!(s.draft, before);
    // D-2026-10-15-13: entered exchanges have no arbitrary 20-row limit.
    let request = save_topic(
        &before,
        TaxSection::InvestmentSales,
        (0..21)
            .map(|index| {
                let mut exchange = related_exchange_kind(20260430);
                exchange.property = alloc::format!("Synthetic property {index}");
                document(0, IncomeDocument::RelatedExchange(exchange))
            })
            .collect(),
    );
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.screening.as_ref().unwrap().related_exchanges.len(), 21);
    assert_eq!(value(&supported(&s.draft), "agi"), 1_950_000);
}
