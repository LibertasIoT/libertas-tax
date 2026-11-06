// Record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::*;
use alloc::format;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::document;
use super::support::client::income_pages::{
    catalog_preview_document, open_documents, open_topic, page_documents, save_documents,
    save_topic,
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
    let mut int = Int1099 {
        more: IntMore {
            us_obligations: 25_000,
            special: InterestSpecial::NoneOfThese,
            ..Default::default()
        },
        ..native_int1099("Treasury Direct".into(), 10_000)
    };
    // "Amount on the return" (i1040gi): box 1 ($100.00) plus box 3 ($250.00).
    assert_eq!(
        crate::tests::support::client::income_pages::saved_document_amount(
            &mut session,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(int.clone()))
            }
        ),
        Some(35_000)
    );
    {
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(int.clone())),
                },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    };
    let FederalIncome::Interest { data } = &session.draft.income[1].income else { unreachable!() };
    assert_eq!(FederalInterest::from(data).us_obligations, 25_000);
    assert_eq!(FederalInterest::from(data).taxable, 10_000);
    // Reopening shows the same box 3 amount.
    let mut session = restart(session.draft);
    let documents = page_documents(&open_documents(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::InterestDividend { .. }))
        .collect::<Vec<_>>();
    int.more.calls = IntCalls { nominee: vec![0], ..Default::default() };
    assert_eq!(
        documents[0].document,
        expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(int))
            }
        )
    );
}

#[test]
fn oid_document_maps_onto_federal_oid() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut oid = Oid1099 {
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
    {
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(oid.clone())),
                },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    };
    let FederalIncome::Oid { data } = &session.draft.income[1].income else { unreachable!() };
    assert_eq!(
        FederalOid::from(data),
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
    let documents = page_documents(&open_documents(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::InterestDividend { .. }))
        .collect::<Vec<_>>();
    oid.more.calls = OidCalls { nominee: vec![0] };
    assert_eq!(
        documents[0].document,
        expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Oid1099(oid_document(oid))
            }
        )
    );
}

// Investment sales, unemployment/state refunds, and prizes,
// royalties and equipment rent.
#[test]
fn sale_maps_onto_federal_sale_with_its_box() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let sale = sale_document(SaleSource {
        form: Sale1099B {
            payer: Default::default(),
            recipient: Default::default(),
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
        },
        adjustment: SaleAdjustment::SellingExpenses { amount: 5_000 },
        special: crate::SaleSpecial::NoneOfThese,
        amt: SaleAmt::SameAsRegular,
    });
    // Proceeds $5,000 - basis $3,000 - selling expenses $50 = $1,950 gain.
    assert_eq!(
        crate::tests::support::client::income_pages::saved_document_amount(
            &mut session,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Sale1099B(sale.clone())
            }
        ),
        Some(195_000)
    );
    {
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Sale1099B(sale),
                },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    };
    let FederalIncome::Sale { data } = &session.draft.income[1].income else { unreachable!() };
    assert_eq!(
        FederalSale::from(data),
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
    let documents = page_documents(&open_documents(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::InvestmentSale { .. }))
        .collect::<Vec<_>>();
    assert_eq!(
        documents[0].document,
        expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Sale1099B(sale_document(SaleSource {
                    form: Sale1099B {
                        payer: Default::default(),
                        recipient: Default::default(),
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
                    },
                    adjustment: SaleAdjustment::SellingExpenses { amount: 5_000 },
                    special: crate::SaleSpecial::NoneOfThese,
                    amt: SaleAmt::SameAsRegular
                }))
            }
        )
    );
}

#[test]
fn wash_sale_above_the_loss_is_rejected() {
    let original = complete(FederalStatus::Single);
    // Proceeds $1,000, basis $4,000: a $3,000 loss.
    let sale_with_wash = |wash_sale: i64| {
        sale_document(SaleSource {
            form: Sale1099B {
                payer: Default::default(),
                recipient: Default::default(),
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
            },
            adjustment: SaleAdjustment::None,
            special: crate::SaleSpecial::NoneOfThese,
            amt: SaleAmt::SameAsRegular,
        })
    };
    // Boundary: a wash sale exactly equal to the loss is accepted.
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = {
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Sale1099B(sale_with_wash(300_000)),
                },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    };
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    // One cent more than the loss is refused by name, on save and preview.
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = {
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Sale1099B(sale_with_wash(300_001)),
                },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    };
    rejected_with(reply, "SALE_WASH_SALE_ABOVE_LOSS");
    assert_eq!(session.draft, original);
    assert!(!session.dirty);
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
            answers: Vec::new(),
            credits: vec![MoreCredit::ForeignTax(ordinary.clone().try_into().unwrap())],

            documents: Vec::new(),
        },
    );
    send(&mut session, request);
    let int = |foreign_tax| Int1099 {
        more: IntMore { foreign_tax, special: InterestSpecial::NoneOfThese, ..Default::default() },
        ..native_int1099("Synthetic Bank".into(), 50_000)
    };
    for foreign_tax in [0, 20_000] {
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(int(
                        foreign_tax,
                    ))),
                },
            )],
        );
        let reply = send(&mut session, request);
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        assert_eq!(
            session.draft.credits.as_ref().unwrap().foreign_tax,
            ordinary,
            "foreign tax {foreign_tax}"
        );
        // The box 6 amount is stored on the document itself: reopening shows it with the Form 1116
        // claim in force, where it used to read as zero.
        let mut reopened = restart(session.draft.clone());
        let page = open_documents(&mut reopened);
        let mut expected = int(foreign_tax);
        expected.more.calls = IntCalls { nominee: vec![0], ..Default::default() };
        assert_eq!(
            page_documents(&page)
                .into_iter()
                .find(|row| matches!(row.document, IncomeSource::InterestDividend { .. }))
                .unwrap()
                .document,
            expected_named_source(
                &session.draft,
                Owner::Taxpayer,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(expected))
                }
            ),
            "reopened with foreign tax {foreign_tax}"
        );
    }
}

#[test]
fn capital_loss_carryover_kind_reaches_schedule_d() {
    // The 2025 return capital_history: AGI $10,000, deductions $12,000,
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
        &mut s,
        TaxSection::InvestmentSales,
        vec![document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::CapitalLossCarryover(capital_history().into()),
            },
        )],
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
            ..document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::CapitalLossCarryover(capital_history().into())
                }
            )
        }]
    );
    let before = s.draft.clone();
    rejected_with(
        {
            let request = save_topic(
                &mut s,
                TaxSection::InvestmentSales,
                vec![
                    document(
                        0,
                        IncomeSource::Record {
                            value: crate::IncomeRecord::CapitalLossCarryover(
                                capital_history().into()
                            )
                        }
                    );
                    3
                ],
            );
            send(&mut s, request)
        },
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
            livestock: crate::SoldLivestock::No,
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
    assert_eq!(items[0].document, IncomeSource::Record { value: crate::IncomeRecord::Rental(r) });
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
        &mut s,
        TaxSection::InvestmentSales,
        vec![document(
            0,
            IncomeSource::Record { value: crate::IncomeRecord::InstallmentPayments(sale.page()) },
        )],
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
            ..document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::InstallmentPayments(sale.page())
                }
            )
        }]
    );
    let lookback =
        libertas_macros::variant_index!(crate::IncomeSourceKind::BusinessPropertyLosses) as i32;
    let P::SaveIncomes { allowed_documents, .. } = &reopened else { unreachable!() };
    assert!(!allowed_documents.contains(&lookback));
    let before = s.draft.screening.clone();
    let request =
        save_topic(&mut s, TaxSection::InvestmentSales, page_documents(&reopened).to_vec());
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
        &mut s,
        TaxSection::InvestmentSales,
        vec![document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::InstallmentPayments(building.page()),
            },
        )],
    );
    accepted(&send(&mut s, request));
    unsupported(&s.draft, "Section 1231 lookback");
    let P::SaveIncomes { allowed_documents, .. } = open_topic(&mut s, TaxSection::InvestmentSales)
    else {
        unreachable!()
    };
    assert!(allowed_documents.contains(&lookback));
    let request = save_topic(
        &mut s,
        TaxSection::InvestmentSales,
        vec![
            document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::InstallmentPayments(building.page()),
                },
            ),
            document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::BusinessPropertyLosses(
                        crate::BusinessPropertyLosses { regular: vec![], amt: vec![] },
                    ),
                },
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
        &mut s,
        TaxSection::InvestmentSales,
        vec![document(
            0,
            IncomeSource::Record { value: crate::IncomeRecord::InstallmentPayments(full.page()) },
        )],
    );
    accepted(&send(&mut s, request));
    let before = s.draft.clone();
    let over =
        earlier_installment(crate::PropertyCharacter::LongTermCapital, 5_000_000, 10_000_001);
    rejected_with(
        {
            let request = save_topic(
                &mut s,
                TaxSection::InvestmentSales,
                vec![document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::InstallmentPayments(over.page()),
                    },
                )],
            );
            send(&mut s, request)
        },
        "INSTALLMENT_PAID_ABOVE_CONTRACT",
    );

    let mut profit = earlier_installment(crate::PropertyCharacter::LongTermCapital, 0, 0);
    profit.gross_profit = 15_000_001;
    rejected_with(
        {
            let request = save_topic(
                &mut s,
                TaxSection::InvestmentSales,
                vec![document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::InstallmentPayments(profit.page()),
                    },
                )],
            );
            send(&mut s, request)
        },
        "INSTALLMENT_PROFIT_ABOVE_CONTRACT",
    );
    assert_eq!(s.draft, before);
}

// Accepted savings-bond statements retain their linked people for an explicit
// correction when household membership changes.
#[test]
fn savings_bond_rows_leave_with_their_person() {
    use crate::tests::investment::investment::{bond_record, bond_return, bond_student};
    let mut d = bond_return(FederalStatus::Single, 60_000, 3_000);
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
    // Removing a person does not silently rewrite the accepted statement.
    d.dependents.clear();
    let sources = d.income.clone();
    let pruned = crate::interview::prune(&mut d);
    assert!(pruned.removed.is_empty());
    assert_eq!(d.income, sources);
    let FederalIncome::SavingsBonds { data } = &d.income.last().unwrap().income else {
        unreachable!()
    };
    assert_eq!(data.answers.as_ref().unwrap().students.len(), 2);
    assert_eq!(data.answers.as_ref().unwrap().students[0].person, child);
    assert_eq!(data.answers.as_ref().unwrap().students[1].person, -1);
    // A missing linked person also leaves the source available for correction.
    let FederalIncome::SavingsBonds { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.answers.as_mut().unwrap().students[0].person = -2;
    let sources = d.income.clone();
    crate::interview::prune(&mut d);
    assert_eq!(d.income, sources);
}

#[test]
fn section_1256_carryback_round_trips_and_checks_its_rules() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let kind = |amount| IncomeSource::Record {
        value: crate::IncomeRecord::Section1256Carryback(crate::Section1256Carryback {
            amount,
            injured_owners: None,
        }),
    };
    let request = save_topic(&mut s, TaxSection::InvestmentSales, vec![document(0, kind(800_000))]);
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
    let request =
        save_topic(&mut s, TaxSection::InvestmentSales, page_documents(&reopened).to_vec());
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.screening, before.screening);
    // A second one is refused on the server with the client rule's key.
    rejected_with(
        {
            let request = save_topic(
                &mut s,
                TaxSection::InvestmentSales,
                vec![document(0, kind(800_000)), document(0, kind(100_000))],
            );
            send(&mut s, request)
        },
        "SECTION_1256_CARRYBACK_TWICE",
    );
    // Check the positive source range without constructing an inadmissible value.
    let model = [crate::federal_model::HUB_SOURCE, include_str!("../../investment/section1256.rs")]
        .concat();
    let amount = &model[model.find("pub struct Section1256Carryback").unwrap()..];
    assert!(amount.contains("#[libertas_number(min = 1, max = 100000000000)]"));
    // Removing the kind removes the election.
    let request = save_topic(&mut s, TaxSection::InvestmentSales, vec![]);
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.screening.as_ref().unwrap().section_1256_carryback, None);
}

#[test]
fn investment_sales_offers_the_contracts_kinds_and_search_names_them() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let page = open_topic(&mut s, TaxSection::InvestmentSales);
    let P::SaveIncomes { allowed_documents, .. } = &page else { unreachable!() };
    assert!(allowed_documents.contains(
        &(libertas_macros::variant_index!(crate::IncomeSourceKind::Section1256Carryback) as i32)
    ));
    assert!(!allowed_documents.contains(
        &(libertas_macros::variant_index!(crate::IncomeSourceKind::Contracts1099B) as i32)
    ));
    let model = include_str!("../../federal/income_editor.rs");
    let source = &model[model.find("pub enum InvestmentSaleDocument {").unwrap()..];
    assert!(source.contains("Contracts1099B(Contracts1099B)"));
    let protocol = include_str!("../../core/shared/navigation.rs");
    assert!(protocol.contains("Form 6781 Part I, 60/40 treatment"));
    assert!(protocol.contains("straddles (Form 6781 Part II)"));
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
    assert_eq!(items[0].document, IncomeSource::Record { value: crate::IncomeRecord::Rental(r) });
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
        &mut s,
        TaxSection::InvestmentSales,
        vec![document(
            0,
            IncomeSource::Record { value: crate::IncomeRecord::RelatedExchange(exchange.clone()) },
        )],
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
            ..document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::RelatedExchange(exchange.clone())
                }
            )
        }]
    );
    let kind = libertas_macros::variant_index!(crate::IncomeSourceKind::RelatedExchange) as i32;
    let P::SaveIncomes { allowed_documents, .. } = &reopened else { unreachable!() };
    assert!(allowed_documents.contains(&kind));
    let before = s.draft.screening.clone();
    let request =
        save_topic(&mut s, TaxSection::InvestmentSales, page_documents(&reopened).to_vec());
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.screening, before);
    // Saving the page without it removes the record.
    let request = save_topic(&mut s, TaxSection::InvestmentSales, vec![]);
    accepted(&send(&mut s, request));
    assert!(s.draft.screening.as_ref().unwrap().related_exchanges.is_empty());
    // A business-property record needs the section 1231 lookback, which the
    // page then offers.
    let lookback =
        libertas_macros::variant_index!(crate::IncomeSourceKind::BusinessPropertyLosses) as i32;
    let mut business = related_exchange_kind(20260430);
    business.kind = crate::PropertyCharacter::BusinessProperty {
        used: crate::PropertyUse::Business,
        unrecaptured: 0,
    };
    let request = save_topic(
        &mut s,
        TaxSection::InvestmentSales,
        vec![document(
            0,
            IncomeSource::Record { value: crate::IncomeRecord::RelatedExchange(business) },
        )],
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
    // A spouse's record only on a joint return.
    let mut spouse = document(
        0,
        IncomeSource::Record {
            value: crate::IncomeRecord::RelatedExchange(related_exchange_kind(20260430)),
        },
    );
    spouse.owner = crate::DocumentOwner::Spouses;
    let request = save_topic(&mut s, TaxSection::InvestmentSales, vec![spouse]);
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
    // Entered exchanges have no arbitrary 20-row limit.
    let request = save_topic(
        &mut s,
        TaxSection::InvestmentSales,
        (0..21)
            .map(|index| {
                let mut exchange = related_exchange_kind(20260430);
                exchange.property = alloc::format!("Synthetic property {index}");
                document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::RelatedExchange(exchange) },
                )
            })
            .collect(),
    );
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.screening.as_ref().unwrap().related_exchanges.len(), 21);
    assert_eq!(value(&supported(&s.draft), "agi"), 1_950_000);
}

#[test]
fn retained_installment_livestock_classification_saves_reopens_and_restarts() {
    let mut session = Interview::new(TEST_USER, farmer_gross_return(60_000, 31_500));
    for classification in [None, Some(crate::Livestock::No), Some(crate::Livestock::Other)] {
        let mut sale = farmer_gross_earlier_property().sale;
        sale.livestock = classification;
        let page = sale.page();
        let request = save_topic(
            &mut session,
            TaxSection::InvestmentSales,
            vec![
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::InstallmentPayments(page.clone()),
                    },
                ),
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::BusinessPropertyLosses(
                            crate::BusinessPropertyLosses { regular: vec![], amt: vec![] },
                        ),
                    },
                ),
            ],
        );
        accepted(&if matches!(request, P::SaveDocuments { .. }) {
            crate::tests::support::client::income_pages::save_source_fixture_request(
                &mut session,
                request,
            )
        } else {
            send(&mut session, request)
        });
        let stored = &session.draft.screening.as_ref().unwrap().installment_payments[0].sale;
        assert_eq!(stored, &sale);
        session = restart(session.draft);
        let reopened = open_topic(&mut session, TaxSection::InvestmentSales);
        assert!(page_documents(&reopened).iter().any(|item| {
            item.document
                == IncomeSource::Record {
                    value: crate::IncomeRecord::InstallmentPayments(page.clone()),
                }
        }));
    }
}

#[test]
fn retained_installment_rejects_livestock_with_building_gain_before_save() {
    let mut session = Interview::new(TEST_USER, farmer_gross_return(60_000, 31_500));
    let mut sale = farmer_gross_earlier_property().sale;
    sale.livestock = Some(crate::Livestock::Other);
    let crate::PropertyCharacter::BusinessProperty { unrecaptured, .. } = &mut sale.kind else {
        unreachable!()
    };
    *unrecaptured = 1;
    assert!(!sale.valid());
    let before = session.draft.clone();
    let request = save_topic(
        &mut session,
        TaxSection::InvestmentSales,
        vec![document(
            0,
            IncomeSource::Record { value: crate::IncomeRecord::InstallmentPayments(sale.page()) },
        )],
    );
    let dirty = session.dirty;
    let saved = send(&mut session, request);
    let Reply::Response(P::Problem { field, .. }) = &saved else { unreachable!() };
    assert_eq!(field.as_deref(), Some("[documents/documents/0/document/InstallmentPayments/kind]"));
    rejected_with(saved, "INSTALLMENT_LIVESTOCK_CHARACTER");
    let preview = send(
        &mut session,
        catalog_preview_document(IncomeSource::Record {
            value: crate::IncomeRecord::InstallmentPayments(sale.page()),
        }),
    );
    let Reply::Response(P::Problem { field, .. }) = &preview else { unreachable!() };
    assert_eq!(field.as_deref(), Some("[document/InstallmentPayments/kind]"));
    rejected_with(preview, "INSTALLMENT_LIVESTOCK_CHARACTER");
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
}

#[test]
fn current_installment_land_native_save_correct_reopen_and_restart() {
    use crate::tests::support::builders::current_installment_land;
    use crate::tests::support::client::current_installment_session;
    let sale = current_installment_land();
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = session.draft.clone();
    let preview = send(
        &mut session,
        catalog_preview_document(IncomeSource::Record {
            value: crate::IncomeRecord::InstallmentSale(sale.clone()),
        }),
    );
    assert!(matches!(preview, Reply::Response(P::IncomePreview { preview: Some(1_400_000), .. })));
    assert_eq!(session.draft, before);
    let mut session = current_installment_session(sale.clone());
    let accepted_id = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, crate::FederalIncome::InstallmentSale { .. }))
        .unwrap()
        .id;
    assert!(accepted_id > 0);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "capital_gain_11"), 14_000);
    assert_eq!(value(&result, "agi"), 74_000);
    assert_eq!(
        crate::tests::support::assert::displayed_line_amount(
            &result,
            &format!("installment_sale_{accepted_id}_profit"),
        ),
        7_000_000,
    );
    let mut page = open_topic(&mut session, TaxSection::InvestmentSales);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    let row = documents
        .documents
        .iter_mut()
        .find(|row| matches!(row.document, crate::IncomeRecord::InstallmentSale(_)))
        .unwrap();
    assert_eq!(row.id, accepted_id);
    let crate::IncomeRecord::InstallmentSale(edited) = &mut row.document else { panic!() };
    assert_eq!(*edited, sale);
    edited.payments = 6_000_000;
    let before_running = session.draft.clone();
    let running = crate::tests::support::client::estimate_documents_from_save(&page);
    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(&mut session, running),
        Reply::Response(P::IncomeRunningEstimate { .. })
    ));
    assert_eq!(session.draft, before_running);
    accepted(&send(&mut session, roundtrip_native_request(page)));
    assert_eq!(value(&supported(&session.draft), "capital_gain_11"), 21_000);
    let expected = session.draft.income.clone();
    let mut session = restart(session.draft);
    let reopened = open_topic(&mut session, TaxSection::InvestmentSales);
    accepted(&send(&mut session, roundtrip_native_request(reopened)));
    assert_eq!(session.draft.income, expected);
}

#[test]
fn current_installment_elect_out_uses_full_sale_owner() {
    use crate::tests::support::builders::current_installment_land;
    use crate::tests::support::client::current_installment_session;
    let mut sale = current_installment_land();
    sale.method = crate::InstallmentMethod::ElectOut;
    let session = current_installment_session(sale.clone());
    let result = supported(&session.draft);
    assert_eq!(value(&result, "capital_gain_10"), 70_000);
    assert!(
        !result
            .lines
            .iter()
            .any(|line| line.form_line.as_ref().is_some_and(|line| line.form == TaxForm::Form6252))
    );
}

#[test]
fn current_installment_personal_loss_is_not_deductible() {
    use crate::tests::support::{
        builders::current_installment_land, client::current_installment_session,
    };
    let mut sale = current_installment_land();
    sale.use_kind = crate::InstallmentUse::Personal;
    sale.basis = 23_000_000;
    let session = current_installment_session(sale);
    assert_eq!(value(&supported(&session.draft), "agi"), 60_000);
}

#[test]
fn current_installment_debt_above_basis_is_received_once() {
    use crate::tests::support::{
        builders::current_installment_land, client::current_installment_session,
    };
    let mut sale = current_installment_land();
    sale.basis = 8_000_000;
    sale.assumed_debt = 12_000_000;
    sale.payments = 2_000_000;
    let session = current_installment_session(sale);
    let result = supported(&session.draft);
    let id = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, crate::FederalIncome::InstallmentSale { .. }))
        .unwrap()
        .id;
    assert_eq!(value(&result, &format!("installment_sale_{id}_excess_debt")), 30_000);
    assert_eq!(value(&result, &format!("installment_sale_{id}_contract_price")), 110_000);
    assert_eq!(value(&result, "capital_gain_11"), 50_000);
}

#[test]
fn current_installment_anniversary_is_short_term() {
    use crate::tests::support::{
        builders::current_installment_land, client::current_installment_session,
    };
    let mut sale = current_installment_land();
    sale.acquired = 20250301;
    let result = supported(&current_installment_session(sale.clone()).draft);
    assert_eq!(value(&result, "capital_gain_4"), 14_000);
}

#[test]
fn current_installment_related_loss_is_disallowed() {
    use crate::tests::support::{
        builders::current_installment_land, client::current_installment_session,
    };
    let mut sale = current_installment_land();
    sale.basis = 23_000_000;
    sale.buyer = crate::Buyer::Relative;
    let result = supported(&current_installment_session(sale).draft);
    assert_eq!(value(&result, "agi"), 60_000);
    assert!(
        result
            .lines
            .iter()
            .any(|line| line.form_line.as_ref().is_some_and(|form| form.form == TaxForm::Form8949))
    );
}

#[test]
fn current_installment_native_invalid_payment_rejects_without_changing_accepted_source() {
    use crate::tests::support::client::current_income_refusal;
    use crate::tests::support::{
        builders::current_installment_land, client::current_installment_session,
    };
    let mut session = current_installment_session(current_installment_land());
    for case in [0, 2, 11, 13] {
        let before = session.draft.clone();
        let mut page = open_topic(&mut session, TaxSection::InvestmentSales);
        let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
        let crate::IncomeRecord::InstallmentSale(sale) = &mut documents.documents[0].document
        else {
            panic!()
        };
        let (key, field) = match case {
            0 => {
                sale.payments = sale.price + 1;
                ("INSTALLMENT_PAID_ABOVE_PRICE", "payments")
            }
            2 => {
                sale.acquired = 20270101;
                ("INSTALLMENT_SOLD_BEFORE_ACQUIRED", "acquired")
            }
            11 => {
                sale.second_disposition =
                    crate::SecondDisposition::Resold { on: sale.sold - 1, amount_realized: 0 };
                ("INSTALLMENT_RESALE_BEFORE_SALE", "second_disposition/Resold/on")
            }
            _ => {
                documents.documents[0].owner = Owner::Spouse;
                ("INSTALLMENT_OWNER_INVALID", "owner")
            }
        };
        if case != 13 {
            let preview = catalog_preview_document(IncomeSource::Record {
                value: documents.documents[0].document.clone(),
            });
            let reply = send(&mut session, preview);
            let Reply::Response(P::Problem { field: actual, .. }) = &reply else { panic!() };
            assert_eq!(
                actual.as_deref(),
                Some(format!("[document/InstallmentSale/{field}]").as_str())
            );
            rejected_with(reply, key);
            assert_eq!(session.draft, before);
        }
        let path = if case == 13 {
            "[documents/documents/0/owner]".into()
        } else {
            format!("[documents/documents/0/document/InstallmentSale/{field}]")
        };
        current_income_refusal(
            &format!("installment-boundary-{case}"),
            &mut session,
            page,
            key,
            &path,
        );
        assert_eq!(session.draft, before);
    }
}

#[test]
fn current_installment_community_source_is_retained_and_gain_is_shared_once() {
    use crate::tests::support::{
        builders::{community_w2_draft, current_installment_land},
        client::current_installment_session_for,
    };
    let draft = community_w2_draft(4_000_000, 8_000_000, 0, 0);
    let session = current_installment_session_for(
        draft,
        current_installment_land(),
        Owner::Spouse,
        CommunityShare::Community,
    );
    let record = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, crate::FederalIncome::InstallmentSale { .. }))
        .unwrap();
    assert_eq!(record.community, CommunityShare::Community);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "capital_gain_11"), 7_000);
    assert_eq!(value(&result, "agi"), 67_000);
}

#[test]
fn current_installment_collectible_keeps_28_percent_character() {
    use crate::tests::support::{
        builders::current_installment_land, client::current_installment_session,
    };
    let mut sale = current_installment_land();
    sale.collectible = true;
    let session = current_installment_session(sale);
    assert_eq!(value(&supported(&session.draft), "rate_28_gain"), 14_000);
}

#[test]
fn current_installment_actual_interest_is_separate_from_principal() {
    use crate::tests::support::{
        builders::current_installment_land, client::current_installment_session,
    };
    let mut session = current_installment_session(current_installment_land());
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(native_int1099(
                    "Synthetic land buyer".into(),
                    20_000,
                ))),
            },
        )],
    );
    accepted(&send(&mut session, roundtrip_native_request(request)));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "capital_gain_11"), 14_000);
    assert_eq!(value(&result, "agi"), 74_200);
}

#[test]
fn current_installment_investment_gain_enters_niit_once() {
    use crate::tests::support::{
        builders::{current_installment_land, current_installment_niit_draft},
        client::current_installment_session_for,
    };
    let session = current_installment_session_for(
        current_installment_niit_draft(),
        current_installment_land(),
        Owner::Taxpayer,
        CommunityShare::NotNeeded,
    );
    assert_eq!(value(&supported(&session.draft), "net_investment_income_tax"), 532);
}

#[test]
fn current_installment_personal_gain_keeps_existing_niit_property_treatment() {
    use crate::tests::support::{
        builders::{current_installment_land, current_installment_niit_draft},
        client::current_installment_session_for,
    };
    let mut sale = current_installment_land();
    sale.use_kind = crate::InstallmentUse::Personal;
    let session = current_installment_session_for(
        current_installment_niit_draft(),
        sale,
        Owner::Taxpayer,
        CommunityShare::NotNeeded,
    );
    // 2026-10-18 primary-source correction to this new fixture: i8960 lines5a–5d
    // include taxable personal-property gain in NII; Form4952 has a separate investment pool.
    // https://www.irs.gov/instructions/i8960
    assert_eq!(value(&supported(&session.draft), "net_investment_income_tax"), 532);
}

#[test]
fn current_installment_personal_gain_attribution_uses_existing_capital_owner() {
    use crate::tests::support::{
        builders::{current_installment_joint_niit_draft, current_installment_land},
        client::current_installment_session_for,
    };
    let mut sale = current_installment_land();
    sale.use_kind = crate::InstallmentUse::Personal;
    let session = current_installment_session_for(
        current_installment_joint_niit_draft(),
        sale,
        Owner::Spouse,
        CommunityShare::NotNeeded,
    );
    let result = supported(&session.draft);
    assert_eq!(value(&result, "net_investment_income_tax"), 532);
    assert_eq!(
        ["a", "b", "c"].map(|column| value(&result, &format!("form_8379_line_18_{column}"))),
        [532, 0, 532]
    );
}

#[test]
fn current_installment_personal_collectible_keeps_28_percent_character_and_native_fact() {
    use crate::tests::support::{
        builders::current_installment_land, client::current_installment_session,
    };
    let mut sale = current_installment_land();
    sale.use_kind = crate::InstallmentUse::Personal;
    sale.collectible = true;
    let session = current_installment_session(sale.clone());
    assert_eq!(value(&supported(&session.draft), "rate_28_gain"), 14_000);
    let mut session = restart(session.draft);
    let page = open_topic(&mut session, TaxSection::InvestmentSales);
    let IncomeSource::Record { value: crate::IncomeRecord::InstallmentSale(reopened) } =
        &page_documents(&page)[0].document
    else {
        panic!()
    };
    assert_eq!(*reopened, sale);
}

#[test]
fn current_installment_fully_paid_sale_uses_full_sale_owner_without_form6252() {
    use crate::tests::support::{
        builders::current_installment_land, client::current_installment_session,
    };
    let mut sale = current_installment_land();
    sale.payments = sale.price;
    let session = current_installment_session(sale);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "capital_gain_10"), 70_000);
    assert!(
        !result
            .lines
            .iter()
            .any(|line| line.form_line.as_ref().is_some_and(|line| line.form == TaxForm::Form6252))
    );
}

#[test]
fn current_installment_related_resale_caps_and_reports_additional_income_once() {
    use crate::tests::support::{
        builders::current_installment_land, client::current_installment_session,
    };
    let mut sale = current_installment_land();
    sale.buyer = crate::Buyer::Relative;
    sale.second_disposition =
        crate::SecondDisposition::Resold { on: 20260901, amount_realized: 30_000_000 };
    let mut session = current_installment_session(sale.clone());
    let id = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, crate::FederalIncome::InstallmentSale { .. }))
        .unwrap()
        .id;
    let result = supported(&session.draft);
    assert_eq!(value(&result, "capital_gain_11"), 70_000);
    for (suffix, expected) in [
        ("gain", 14_000),
        ("resale_realized", 300_000),
        ("resale_contract_price", 200_000),
        ("lesser", 200_000),
        ("payments_to_date", 40_000),
        ("accelerated", 160_000),
        ("resale_income", 56_000),
        ("resale_ordinary", 0),
        ("resale_gain", 56_000),
    ] {
        assert_eq!(value(&result, &format!("installment_sale_{id}_{suffix}")), expected);
    }
    let before = session.draft.clone();
    let preview = send(
        &mut session,
        catalog_preview_document(IncomeSource::Record {
            value: crate::IncomeRecord::InstallmentSale(sale),
        }),
    );
    assert!(matches!(preview, Reply::Response(P::IncomePreview { preview: Some(7_000_000), .. })));
    assert_eq!(session.draft, before);
    let mut session = restart(session.draft);
    let reopened = open_topic(&mut session, TaxSection::InvestmentSales);
    accepted(&send(&mut session, roundtrip_native_request(reopened)));
    assert_eq!(value(&supported(&session.draft), "capital_gain_11"), 70_000);

    // Current-year reported Form 6252 lines share the annual owner's dollar
    // stages: a $1/$3 ratio and 49 cents actual principal, then a resale on
    // either side of $1.50. Same-day acquisition and sale remains short-term.
    for (amount_realized, reported, income) in [(149, 1, 0), (150, 2, 1), (151, 2, 1)] {
        let mut cents = current_installment_land();
        cents.acquired = cents.sold;
        cents.price = 300;
        cents.basis = 200;
        cents.selling_expenses = 0;
        cents.payments = 49;
        cents.buyer = crate::Buyer::Relative;
        cents.second_disposition =
            crate::SecondDisposition::Resold { on: 20260901, amount_realized };
        let session = current_installment_session(cents);
        let id = session
            .draft
            .income
            .iter()
            .find(|entry| matches!(entry.income, crate::FederalIncome::InstallmentSale { .. }))
            .unwrap()
            .id;
        let result = supported(&session.draft);
        assert_eq!(value(&result, "capital_gain_4"), income);
        for (suffix, expected) in [
            ("resale_realized", reported),
            ("resale_contract_price", 3),
            ("lesser", reported),
            ("payments_to_date", 0),
            ("accelerated", reported),
            ("resale_income", income),
        ] {
            assert_eq!(value(&result, &format!("installment_sale_{id}_{suffix}")), expected);
        }
    }
}

#[test]
fn current_installment_eligible_pledge_enters_line21_once() {
    use crate::tests::support::{
        builders::current_installment_land, client::current_installment_session,
    };
    let mut sale = current_installment_land();
    sale.pledged = 4_000_000;
    let session = current_installment_session(sale);
    let id = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, crate::FederalIncome::InstallmentSale { .. }))
        .unwrap()
        .id;
    let result = supported(&session.draft);
    assert_eq!(value(&result, "capital_gain_11"), 28_000);
    assert_eq!(value(&result, &format!("installment_sale_{id}_payments")), 80_000);
    assert_eq!(value(&result, &format!("installment_sale_{id}_pledged")), 40_000);

    // 2026-10-18, Publication 537, Pledge Rule: cap the proceeds treated as
    // payment, not the actual loan proceeds. Personal notes and electing out
    // retain their existing treatment even when actual proceeds exceed unpaid principal.
    for (use_kind, method, line, expected) in [
        (
            crate::InstallmentUse::Investment,
            crate::InstallmentMethod::Installment,
            "capital_gain_11",
            70_000,
        ),
        (
            crate::InstallmentUse::Personal,
            crate::InstallmentMethod::Installment,
            "capital_gain_11",
            14_000,
        ),
        (
            crate::InstallmentUse::Investment,
            crate::InstallmentMethod::ElectOut,
            "capital_gain_10",
            70_000,
        ),
    ] {
        let mut capped = current_installment_land();
        capped.pledged = 30_000_000;
        capped.use_kind = use_kind;
        capped.method = method;
        let session = current_installment_session(capped);
        assert_eq!(value(&supported(&session.draft), line), expected);
    }
}

#[test]
fn current_installment_excess_debt_pledge_and_related_resale_share_paid_principal() {
    use crate::tests::support::{
        builders::current_installment_land, client::current_installment_session,
    };
    let mut sale = current_installment_land();
    sale.basis = 8_000_000;
    sale.assumed_debt = 12_000_000;
    sale.payments = 2_000_000;
    sale.pledged = 1_000_000;
    sale.buyer = crate::Buyer::ControlledEntity;
    sale.second_disposition =
        crate::SecondDisposition::Resold { on: 20260901, amount_realized: 9_000_000 };
    let session = current_installment_session(sale);
    let id = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, crate::FederalIncome::InstallmentSale { .. }))
        .unwrap()
        .id;
    let result = supported(&session.draft);
    assert_eq!(value(&result, "capital_gain_11"), 90_000);
    for (suffix, expected) in [
        ("deemed_payment", 30_000),
        ("payments", 30_000),
        ("gain", 60_000),
        ("payments_to_date", 60_000),
        ("accelerated", 30_000),
        ("resale_gain", 30_000),
    ] {
        assert_eq!(value(&result, &format!("installment_sale_{id}_{suffix}")), expected);
    }
}

#[test]
fn current_installment_personal_note_is_excepted_from_pledge_rule() {
    use crate::tests::support::{
        builders::current_installment_land, client::current_installment_session,
    };
    let mut sale = current_installment_land();
    sale.use_kind = crate::InstallmentUse::Personal;
    sale.pledged = 4_000_000;
    let session = current_installment_session(sale);
    assert_eq!(value(&supported(&session.draft), "capital_gain_11"), 14_000);
}

#[test]
fn current_installment_pledge_threshold_is_strictly_above_150000() {
    use crate::tests::support::{
        builders::current_installment_land, client::current_installment_session,
    };
    let mut sale = current_installment_land();
    sale.price = 15_000_000;
    sale.basis = 10_000_000;
    sale.selling_expenses = 0;
    sale.payments = 3_000_000;
    sale.pledged = 3_000_000;
    let session = current_installment_session(sale);
    assert_eq!(value(&supported(&session.draft), "capital_gain_11"), 10_000);
}

#[test]
fn current_installment_excepted_related_disposition_does_not_accelerate() {
    use crate::tests::support::{
        builders::current_installment_land, client::current_installment_session,
    };
    let mut sale = current_installment_land();
    sale.buyer = crate::Buyer::Relative;
    sale.second_disposition =
        crate::SecondDisposition::Excepted { condition: crate::ResaleException::AfterDeath };
    let session = current_installment_session(sale);
    assert_eq!(value(&supported(&session.draft), "capital_gain_11"), 14_000);
}

// Schedule B 2026 draft line 1: private-loan interest is taxable without a
// 1099-INT; seller financing of the buyer's residence also retains buyer facts.
#[test]
fn private_loan_interest_uses_schedule_b_and_reopens_as_a_record() {
    use super::support::builders::native_income::private_loan_interest;
    for residence in [false, true] {
        let source = private_loan_interest(residence);
        let mut s = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
        crate::tests::support::client::income_pages::save_each_topic(
            &mut s,
            vec![
                document(
                    0,
                    IncomeSource::InterestDividend {
                        value: InterestDividendDocument::Int1099(interest_document(
                            native_int1099("Synthetic bank".into(), 20_000),
                        )),
                    },
                ),
                document(0, source.clone()),
            ],
        );
        let result = supported(&s.draft);
        assert_eq!(value(&result, "schedule_b_taxable_interest"), 1_600);
        assert_eq!(return_line(&result, "2b"), 160_000);
        let entry = s
            .draft
            .income
            .iter()
            .find(|e| {
                matches!(&e.income,
            FederalIncome::Interest { data } if FederalInterest::from(data).private_loan)
            })
            .unwrap();
        let FederalIncome::Interest { data } = &entry.income else { unreachable!() };
        assert_eq!(FederalInterest::from(data).withholding, 0);
        assert_eq!(FederalInterest::from(data).seller_mortgage.is_some(), residence);
        assert!(entry.state_source.is_none());
        let mut s = restart(crate::tests::support::client::roundtrip_native_draft(s.draft.clone()));
        let page = open_topic(&mut s, TaxSection::InterestDividends);
        assert_eq!(page_documents(&page).len(), 1);
        assert_eq!(page_documents(&page)[0].document, source);
        let saved_id = page_documents(&page)[0].id;
        assert!(matches!(
            &page,
            P::SaveIncomes { source_documents: Some(crate::TopicDocumentViews::InterestDividends { documents }), .. }
                if documents.len() == 1
        ));
        assert_eq!(supported(&s.draft), result);
        accepted(&if matches!(page, P::SaveDocuments { .. }) {
            crate::tests::support::client::income_pages::save_source_fixture_request(&mut s, page)
        } else {
            send(&mut s, page)
        });
        let center = open_documents(&mut s);
        let rows = page_documents(&center);
        assert_eq!(rows.len(), 1);
        assert!(matches!(rows[0].document, IncomeSource::InterestDividend { .. }));
        accepted(&send(&mut s, center));
        let page = open_topic(&mut s, TaxSection::InterestDividends);
        assert_eq!(page_documents(&page)[0].id, saved_id);
        assert_eq!(page_documents(&page)[0].document, source);
        assert_eq!(supported(&s.draft).return_lines, result.return_lines);
    }
}

// Read the choices on the actual keyed topic page for an accepted native source.
fn investment_topic_questions(
    draft: &FederalDraft,
    source: IncomeSource,
) -> Vec<crate::DocumentQuestion> {
    let mut draft = draft.clone();
    let mut entry = income(crate::interview::document_income(&source));
    entry.state_source = crate::IncomeStateSource::from_document(&source);
    // The filer's topic answers recorded with the fixture reach the accepted source.
    for answer in crate::tests::support::builders::fixture_paper_answers(&source) {
        if let crate::DocumentAnswer::InterestAdjustment { value } = answer {
            match &mut entry.income {
                FederalIncome::Interest { data: crate::InterestSource::Int1099(data) } => {
                    data.adjustments = value
                }
                FederalIncome::Oid { data } => data.adjustments = value,
                _ => {}
            }
        }
    }
    add(&mut draft, entry);
    let mut session = Interview::new(TEST_USER, draft);
    let before = session.draft.clone();
    let P::SaveIncomes { followups, .. } = open_topic(&mut session, TaxSection::InterestDividends)
    else {
        panic!("expected the interest and dividends topic")
    };
    assert_eq!(session.draft, before);
    followups
}
fn question_choices(
    rows: &[crate::DocumentQuestion],
    matches: fn(&crate::DocumentAnswer) -> bool,
) -> Vec<i32> {
    rows.iter()
        .find(|row| matches(&row.answer))
        .map_or_else(Vec::new, |row| row.allowed_values.clone())
}

#[test]
fn interest_calls_follow_reported_components_and_keep_replacement_records_available() {
    let draft = complete(FederalStatus::Single);
    for (taxable, exempt, premium, adjustments, expected) in [
        (0, 0, 0, None, Vec::new()),
        (1, 0, 0, None, vec![0]),
        (0, 1, 0, None, vec![0]),
        (0, 0, 1, None, vec![0]),
        (0, 0, 0, Some(InterestAdjustments::default()), vec![0]),
        (0, 0, 0, None, Vec::new()),
    ] {
        let mut form = native_int1099("Synthetic lender".into(), taxable);
        form.more.exempt = exempt;
        form.more.bond_premium = premium;
        form.adjustments = adjustments;
        let rows = investment_topic_questions(
            &draft,
            IncomeSource::InterestDividend {
                value: InterestDividendDocument::Int1099(interest_document(form)),
            },
        );
        assert_eq!(
            question_choices(&rows, |answer| matches!(
                answer,
                crate::DocumentAnswer::InterestNominee { .. }
            )),
            expected
        );
        assert!(
            question_choices(&rows, |answer| matches!(
                answer,
                crate::DocumentAnswer::InterestAfterDeath { .. }
            ))
            .is_empty()
        );
    }
}

#[test]
fn dividend_calls_keep_box5_and_box2c_answers_independent() {
    let draft = complete(FederalStatus::Single);
    for (reit, qsbs, expected_reit, expected_qsbs) in [
        (0, 0, Vec::new(), Vec::new()),
        (1, 0, vec![0, 1], Vec::new()),
        (0, 1, Vec::new(), vec![0, 1, 2, 3]),
        (1, 1, vec![0, 1], vec![0, 1, 2, 3]),
        (0, 0, Vec::new(), Vec::new()),
    ] {
        let mut form = native_div1099("Synthetic fund".into(), reit, 0);
        form.section199a = reit;
        form.section_1202_gain = qsbs;
        form.capital_distributions = qsbs;
        let rows = investment_topic_questions(
            &draft,
            IncomeSource::InterestDividend {
                value: InterestDividendDocument::Div1099(dividend_document(form)),
            },
        );
        assert_eq!(
            question_choices(&rows, |answer| matches!(
                answer,
                crate::DocumentAnswer::DividendEligibility { .. }
            )),
            expected_reit
        );
        assert_eq!(
            question_choices(&rows, |answer| matches!(
                answer,
                crate::DocumentAnswer::DividendFundExclusion { .. }
            )),
            expected_qsbs
        );
        assert_eq!(
            question_choices(&rows, |answer| matches!(
                answer,
                crate::DocumentAnswer::DividendNominee { .. }
            )),
            if reit != 0 || qsbs != 0 { vec![0] } else { Vec::new() }
        );
        assert!(
            question_choices(&rows, |answer| matches!(
                answer,
                crate::DocumentAnswer::DividendAfterDeath { .. }
            ))
            .is_empty()
        );
    }
}

#[test]
fn oid_calls_include_negative_treasury_oid_and_replacement_records() {
    let draft = complete(FederalStatus::Single);
    for (treasury, periodic, adjustments, expected) in [
        (0, 0, None, Vec::new()),
        (-1, 0, None, vec![0]),
        (0, 1, None, vec![0]),
        (0, 0, Some(InterestAdjustments::default()), vec![0]),
        (0, 0, None, Vec::new()),
    ] {
        let mut form = native_oid1099("Synthetic lender".into(), 0, periodic);
        form.more.treasury_oid = treasury;
        form.other_periodic_interest = periodic;
        form.adjustments = adjustments;
        let rows = investment_topic_questions(
            &draft,
            IncomeSource::InterestDividend {
                value: InterestDividendDocument::Oid1099(oid_document(form)),
            },
        );
        assert_eq!(
            question_choices(&rows, |answer| matches!(
                answer,
                crate::DocumentAnswer::OidNominee { .. }
            )),
            expected
        );
    }
}

#[test]
fn investment_calls_are_rebuilt_after_save_and_do_not_change_figures() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut dividend = div1099_source(native_div1099("Synthetic fund".into(), 100_000, 0));
    dividend.form.section199a = Some(20_000);
    dividend.reit_eligible = Some(true.into());
    dividend.nominee = Some(DividendNominee { ordinary: 10_000, ..Default::default() }.into());
    // A submitted derived cookie cannot supply or withhold a personal answer.
    dividend.calls = DivCalls::default();
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::InterestDividend {
                value: InterestDividendDocument::Div1099(dividend_document(dividend.clone())),
            },
        )],
    );
    accepted(&if matches!(request, P::SaveDocuments { .. }) {
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    } else {
        send(&mut session, request)
    });
    let expected = estimate(&session.draft);
    let mut session = restart(session.draft);
    let page = open_documents(&mut session);
    let documents = page_documents(&page);
    let IncomeSource::InterestDividend { value: InterestDividendDocument::Div1099(_) } = &documents
        .iter()
        .find(|row| matches!(row.document, IncomeSource::InterestDividend { .. }))
        .unwrap()
        .document
    else {
        panic!("dividend source")
    };
    let FederalIncome::Dividend { data: reopened } = &session.draft.income.last().unwrap().income
    else {
        panic!("dividend source")
    };
    assert_eq!(reopened.nominee, dividend.nominee);
    assert_eq!(reopened.reit_eligible, dividend.reit_eligible);
    assert_eq!(reopened.after_death, dividend.after_death);
    let mut topic_session = Interview::new(TEST_USER, session.draft.clone());
    let P::SaveIncomes { followups, .. } =
        open_topic(&mut topic_session, TaxSection::InterestDividends)
    else {
        panic!("dividend topic")
    };
    assert_eq!(
        question_choices(&followups, |answer| matches!(
            answer,
            crate::DocumentAnswer::DividendEligibility { .. }
        )),
        vec![0, 1]
    );
    assert!(
        question_choices(&followups, |answer| matches!(
            answer,
            crate::DocumentAnswer::DividendFundExclusion { .. }
        ))
        .is_empty()
    );
    accepted(&if matches!(page, P::SaveDocuments { .. }) {
        crate::tests::support::client::income_pages::save_source_fixture_request(&mut session, page)
    } else {
        send(&mut session, page)
    });
    assert_eq!(estimate(&session.draft), expected);
}

#[test]
fn investment_after_death_calls_require_a_final_return_and_reported_components() {
    let mut draft = complete(FederalStatus::Single);
    draft.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { date: 20260630, ordinary: true, bonds: None, estate_medical: 0 });
    for amount in [0, 1, 0] {
        let interest = investment_topic_questions(
            &draft,
            IncomeSource::InterestDividend {
                value: InterestDividendDocument::Int1099(interest_document(native_int1099(
                    "Synthetic lender".into(),
                    amount,
                ))),
            },
        );
        let dividends = investment_topic_questions(
            &draft,
            IncomeSource::InterestDividend {
                value: InterestDividendDocument::Div1099(dividend_document(native_div1099(
                    "Synthetic fund".into(),
                    amount,
                    0,
                ))),
            },
        );
        let expected = if amount > 0 { vec![0] } else { Vec::new() };
        assert_eq!(
            question_choices(&interest, |answer| matches!(
                answer,
                crate::DocumentAnswer::InterestAfterDeath { .. }
            )),
            expected
        );
        assert_eq!(
            question_choices(&dividends, |answer| matches!(
                answer,
                crate::DocumentAnswer::DividendAfterDeath { .. }
            )),
            expected
        );
    }
}

#[test]
fn dividend_answer_choices_keep_both_existing_reit_tax_paths() {
    for eligible in [false, true] {
        let original = complete(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, original.clone());
        let mut dividend = div1099_source(native_div1099("Synthetic fund".into(), 100_000, 0));
        dividend.form.section199a = Some(20_000);
        dividend.reit_eligible = Some(eligible.into());
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::InterestDividend {
                    value: InterestDividendDocument::Div1099(dividend_document(dividend)),
                },
            )],
        );
        accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        ));
        let FederalIncome::Dividend { data } = &session.draft.income.last().unwrap().income else {
            panic!("dividend tax record")
        };
        assert_eq!(
            FederalDividend::from(data).section199a,
            ReitDividends::Reported { amount: 20_000, eligible: Some(eligible) }
        );
    }
}
