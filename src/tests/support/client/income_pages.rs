use crate::tests::cross::interview::PAGE_INCOME;
use crate::tests::*;

pub(in crate::tests) fn topic_for(document: &IncomeDocument) -> TaxSection {
    match document {
        IncomeDocument::Clergy(_) => TaxSection::Wages,
        IncomeDocument::NonfarmOptionalMethod(_) => TaxSection::SelfEmployment,
        IncomeDocument::W2(_) | IncomeDocument::Form8919(_) => TaxSection::Wages,
        IncomeDocument::SellerMortgageInterest(_)
        | IncomeDocument::Int1099(_)
        | IncomeDocument::Div1099(_)
        | IncomeDocument::Oid1099(_)
        | IncomeDocument::Form8814(_)
        | IncomeDocument::ForeignAccountOrTrust(_)
        | IncomeDocument::TaxableBondPremium(_)
        | IncomeDocument::CapitalReturn1099Div(_)
        | IncomeDocument::Form2439(_)
        | IncomeDocument::SavingsBonds(_) => TaxSection::InterestDividends,
        IncomeDocument::Sale1099B(_)
        | IncomeDocument::OpportunityFund(_)
        | IncomeDocument::OpportunityFundDeferral(_)
        | IncomeDocument::CapitalLossCarryover(_)
        | IncomeDocument::Form3921(_)
        | IncomeDocument::IsoShareSale(_)
        | IncomeDocument::EmployeeShareSale(_)
        | IncomeDocument::EsppShareSale(_)
        | IncomeDocument::NuaShareSale(_)
        | IncomeDocument::QsbsSale(_)
        | IncomeDocument::SmallBusinessStockLoss(_)
        | IncomeDocument::WorthlessSecurity(_)
        | IncomeDocument::BadDebt(_)
        | IncomeDocument::MainHomeSale(_)
        | IncomeDocument::InstallmentPayments(_)
        | IncomeDocument::Contracts1099B(_)
        | IncomeDocument::Section1256Carryback(_)
        | IncomeDocument::RelatedExchange(_) => TaxSection::InvestmentSales,
        IncomeDocument::R1099(_)
        | IncomeDocument::Rrb1099R(_)
        | IncomeDocument::InheritedIra(_)
        | IncomeDocument::Ssa1099(_)
        | IncomeDocument::Form8606(_)
        | IncomeDocument::Qcd(_)
        | IncomeDocument::Form8915(_)
        | IncomeDocument::SocialLumpSums(_)
        | IncomeDocument::MissedRmd(_)
        | IncomeDocument::RothHistory(_) => TaxSection::Retirement,
        IncomeDocument::Business(_)
        | IncomeDocument::Form1099Nec(_)
        | IncomeDocument::Form1099K(_)
        | IncomeDocument::QbiLossCarryforward(_)
        | IncomeDocument::NetOperatingLoss(_)
        | IncomeDocument::BusinessPropertyLosses(_) => TaxSection::SelfEmployment,
        IncomeDocument::Farm(_)
        | IncomeDocument::FarmRental(_)
        | IncomeDocument::Patr1099(_)
        | IncomeDocument::FarmAveraging(_)
        | IncomeDocument::FarmLoss(_) => TaxSection::Farming,
        IncomeDocument::Rental(_) => TaxSection::Rentals,
        IncomeDocument::Form1099G(_) | IncomeDocument::Refund1099G(_) => {
            TaxSection::GovernmentBenefits
        }
        IncomeDocument::Form1099Misc(_)
        | IncomeDocument::OtherIncome(_)
        | IncomeDocument::Royalties(_)
        | IncomeDocument::EquipmentRental(_)
        | IncomeDocument::AlimonyReceived(_)
        | IncomeDocument::Q1099(_)
        | IncomeDocument::EducationAccountExpenses(_)
        | IncomeDocument::Sa1099(_)
        | IncomeDocument::Ltc1099(_)
        | IncomeDocument::LtcInsured(_)
        | IncomeDocument::Qa1099(_)
        | IncomeDocument::AbleContributions(_)
        | IncomeDocument::Scholarship(_)
        | IncomeDocument::JuryPay(_)
        | IncomeDocument::HobbySales(_)
        | IncomeDocument::WildfireRelief(_)
        | IncomeDocument::DisasterRelief(_)
        | IncomeDocument::CasualtyLivingInsurance(_)
        | IncomeDocument::DisasterMitigation(_) => TaxSection::OtherIncome,
        IncomeDocument::W2g(_) | IncomeDocument::GamblingLosses(_) => TaxSection::Gambling,
        IncomeDocument::K1Partnership(_)
        | IncomeDocument::K1PartnershipActivities(_)
        | IncomeDocument::K1ScorpActivities(_)
        | IncomeDocument::K1EstateTrust(_)
        | IncomeDocument::K1Scorp(_) => TaxSection::PassThrough,
        IncomeDocument::Cancel1099C(_)
        | IncomeDocument::Acquisition1099A(_)
        | IncomeDocument::HealthLoanRepayment(_) => TaxSection::CanceledDebt,
        IncomeDocument::Form2555(_) | IncomeDocument::ForeignPay(_) => TaxSection::WorkAbroad,
    }
}
// The document kinds an income topic's page offers, by variant name.
pub(in crate::tests) fn topic_kinds(topic: TaxSection) -> Vec<i32> {
    match topic {
        TaxSection::Wages => vec![
            libertas_macros::variant_index!(IncomeDocument::W2) as i32,
            libertas_macros::variant_index!(IncomeDocument::Form8919) as i32,
            libertas_macros::variant_index!(IncomeDocument::Clergy) as i32,
        ],
        TaxSection::InterestDividends => vec![
            libertas_macros::variant_index!(IncomeDocument::Int1099) as i32,
            libertas_macros::variant_index!(IncomeDocument::SellerMortgageInterest) as i32,
            libertas_macros::variant_index!(IncomeDocument::Div1099) as i32,
            libertas_macros::variant_index!(IncomeDocument::CapitalReturn1099Div) as i32,
            libertas_macros::variant_index!(IncomeDocument::Form2439) as i32,
            libertas_macros::variant_index!(IncomeDocument::Oid1099) as i32,
            libertas_macros::variant_index!(IncomeDocument::ForeignAccountOrTrust) as i32,
            libertas_macros::variant_index!(IncomeDocument::TaxableBondPremium) as i32,
            libertas_macros::variant_index!(IncomeDocument::SavingsBonds) as i32,
        ],
        TaxSection::InvestmentSales => vec![
            libertas_macros::variant_index!(IncomeDocument::Sale1099B) as i32,
            libertas_macros::variant_index!(IncomeDocument::CapitalLossCarryover) as i32,
            libertas_macros::variant_index!(IncomeDocument::Contracts1099B) as i32,
            libertas_macros::variant_index!(IncomeDocument::Section1256Carryback) as i32,
        ],
        TaxSection::Retirement => vec![
            libertas_macros::variant_index!(IncomeDocument::R1099) as i32,
            libertas_macros::variant_index!(IncomeDocument::Ssa1099) as i32,
            libertas_macros::variant_index!(IncomeDocument::Form8606) as i32,
            libertas_macros::variant_index!(IncomeDocument::Qcd) as i32,
            libertas_macros::variant_index!(IncomeDocument::Form8915) as i32,
            libertas_macros::variant_index!(IncomeDocument::SocialLumpSums) as i32,
            libertas_macros::variant_index!(IncomeDocument::MissedRmd) as i32,
        ],
        TaxSection::SelfEmployment => vec![
            libertas_macros::variant_index!(IncomeDocument::NonfarmOptionalMethod) as i32,
            libertas_macros::variant_index!(IncomeDocument::Business) as i32,
            libertas_macros::variant_index!(IncomeDocument::Form1099Nec) as i32,
            libertas_macros::variant_index!(IncomeDocument::Form1099K) as i32,
            libertas_macros::variant_index!(IncomeDocument::Patr1099) as i32,
            libertas_macros::variant_index!(IncomeDocument::QbiLossCarryforward) as i32,
            libertas_macros::variant_index!(IncomeDocument::NetOperatingLoss) as i32,
            libertas_macros::variant_index!(IncomeDocument::BusinessPropertyLosses) as i32,
        ],
        TaxSection::Farming => vec![
            libertas_macros::variant_index!(IncomeDocument::Farm) as i32,
            libertas_macros::variant_index!(IncomeDocument::FarmRental) as i32,
            libertas_macros::variant_index!(IncomeDocument::Patr1099) as i32,
            libertas_macros::variant_index!(IncomeDocument::FarmAveraging) as i32,
            libertas_macros::variant_index!(IncomeDocument::FarmLoss) as i32,
            libertas_macros::variant_index!(IncomeDocument::QbiLossCarryforward) as i32,
            libertas_macros::variant_index!(IncomeDocument::NetOperatingLoss) as i32,
            libertas_macros::variant_index!(IncomeDocument::BusinessPropertyLosses) as i32,
        ],
        TaxSection::Rentals => vec![
            libertas_macros::variant_index!(IncomeDocument::Royalties) as i32,
            libertas_macros::variant_index!(IncomeDocument::Rental) as i32,
            libertas_macros::variant_index!(IncomeDocument::BusinessPropertyLosses) as i32,
        ],
        TaxSection::GovernmentBenefits => vec![
            libertas_macros::variant_index!(IncomeDocument::Form1099G) as i32,
            libertas_macros::variant_index!(IncomeDocument::Refund1099G) as i32,
        ],
        TaxSection::OtherIncome => vec![
            libertas_macros::variant_index!(IncomeDocument::Form1099Misc) as i32,
            libertas_macros::variant_index!(IncomeDocument::OtherIncome) as i32,
            libertas_macros::variant_index!(IncomeDocument::Royalties) as i32,
            libertas_macros::variant_index!(IncomeDocument::EquipmentRental) as i32,
            libertas_macros::variant_index!(IncomeDocument::AlimonyReceived) as i32,
            libertas_macros::variant_index!(IncomeDocument::Q1099) as i32,
            libertas_macros::variant_index!(IncomeDocument::EducationAccountExpenses) as i32,
            libertas_macros::variant_index!(IncomeDocument::Sa1099) as i32,
            libertas_macros::variant_index!(IncomeDocument::Ltc1099) as i32,
            libertas_macros::variant_index!(IncomeDocument::LtcInsured) as i32,
            libertas_macros::variant_index!(IncomeDocument::Qa1099) as i32,
            libertas_macros::variant_index!(IncomeDocument::AbleContributions) as i32,
            libertas_macros::variant_index!(IncomeDocument::Scholarship) as i32,
            libertas_macros::variant_index!(IncomeDocument::JuryPay) as i32,
            libertas_macros::variant_index!(IncomeDocument::HobbySales) as i32,
            libertas_macros::variant_index!(IncomeDocument::WildfireRelief) as i32,
            libertas_macros::variant_index!(IncomeDocument::DisasterRelief) as i32,
            libertas_macros::variant_index!(IncomeDocument::CasualtyLivingInsurance) as i32,
            libertas_macros::variant_index!(IncomeDocument::DisasterMitigation) as i32,
        ],
        TaxSection::Gambling => vec![
            libertas_macros::variant_index!(IncomeDocument::W2g) as i32,
            libertas_macros::variant_index!(IncomeDocument::GamblingLosses) as i32,
        ],
        TaxSection::PassThrough => vec![
            libertas_macros::variant_index!(IncomeDocument::K1Partnership) as i32,
            libertas_macros::variant_index!(IncomeDocument::K1EstateTrust) as i32,
            libertas_macros::variant_index!(IncomeDocument::K1Scorp) as i32,
        ],
        TaxSection::CanceledDebt => vec![
            libertas_macros::variant_index!(IncomeDocument::Cancel1099C) as i32,
            libertas_macros::variant_index!(IncomeDocument::Acquisition1099A) as i32,
            libertas_macros::variant_index!(IncomeDocument::HealthLoanRepayment) as i32,
        ],
        TaxSection::WorkAbroad => vec![
            libertas_macros::variant_index!(IncomeDocument::Form2555) as i32,
            libertas_macros::variant_index!(IncomeDocument::ForeignPay) as i32,
        ],
        _ => vec![],
    }
}
pub(in crate::tests) fn save_topic(
    d: &FederalDraft,
    topic: TaxSection,
    documents: Vec<IncomeDocumentItem>,
) -> P {
    P::SaveIncomes {
        initial_rental_recovery_choices: Vec::new(),
        rental_energy_buildings: Vec::new(),
        empty_k1_activity_choices: crate::tests::support::builders::empty_k1_activity_choices(),
        qbi_sources: crate::qbi_aggregation::source_views(d, d)
            .into_iter()
            .map(|v| v.member)
            .collect(),
        initial_carbon_oxide_origins: vec![],
        initial_renewable_facility_origins: Vec::new(),
        allowed_continuing_risk: ScorpRiskStart::continuing_choices(),
        allowed_injured_source_facts: vec![],
        allowed_injured_retained_owners: vec![],
        ltc_payee_sources: vec![],

        wage_scorporations: Vec::new(),
        allowed_wage_scorporations: Vec::new(),
        initial_k1_investment_choices: alloc::vec![0, 1],
        research_businesses: crate::business_credit_page::research_businesses(d),
        retained_research_choices: crate::retained_research::available(d),
        k1_credit_lines: crate::business_credit_page::k1_credit_lines(),
        research_credit_lines: crate::business_credit_page::research_credit_lines(),
        renewable_credit_lines: crate::business_credit_page::renewable_credit_lines(),
        transferable_credit_lines: crate::business_credit_page::transferable_credit_lines(),
        mortgage_loans: Vec::new(),
        allowed_refund_management: vec![],
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_INCOME,
        previous: PAGE_INCOME,
        review: false,
        estimate: None,
        warnings: vec![],
        progress: None,
        topic,
        allowed_documents: topic_kinds(topic),
        allowed_owners: vec![Owner::Taxpayer as i32, Owner::Spouse as i32],
        allowed_business_source: vec![],
        allowed_farm_source: vec![],
        farms: vec![],
        cooperative_activities: Vec::new(),
        cooperative_properties: Vec::new(),
        businesses: vec![],
        rentals: vec![],
        royalty_properties: vec![],
        equipment_rentals: vec![],
        hobbies: vec![],
        ccc_lots: vec![],
        farm_codes: vec![],
        farm_limitations: vec![],
        farm_sources: vec![],
        basis_assets: vec![],
        farm_method_changes: vec![],
        inherited_iras: vec![],
        inherited_roth_iras: vec![],
        ltc_insured: vec![],
        annuity_sources: vec![],
        recovery_sources: vec![],
        allowed_partner_source: vec![],
        allowed_home_spouse: vec![],
        people: vec![],
        allowed_other_parent: vec![],
        children: vec![],
        document_facts: DocumentFacts { year: 2026 },
        documents: IncomeDocuments {
            qbi_aggregations: crate::qbi_aggregation::project_pages(
                d.screening.as_ref().map_or(&[], |v| v.qbi_aggregations.as_slice()),
                &crate::qbi_aggregation::source_views(d, d),
            ),
            receipt_context: crate::BusinessReceiptContext::default(),
            documents,
        },
        allowed_community: Vec::new(),

        foreign_countries: crate::foreign_accounts::countries(),

        interest_statements: Vec::new(),
        qbi_businesses: vec![],
    }
}
// One topic page's save: the topic of the first document, Wages when empty.
pub(in crate::tests) fn save_documents(d: &FederalDraft, documents: Vec<IncomeDocumentItem>) -> P {
    let topic = documents.first().map_or(TaxSection::Wages, |item| topic_for(&item.document));
    save_topic(d, topic, documents)
}
// Saves documents of several kinds the way the filer does: each topic's page
// in the hub's order.
pub(in crate::tests) fn save_each_topic(
    session: &mut Interview,
    documents: Vec<IncomeDocumentItem>,
) {
    for topic in [TaxSection::Wages, TaxSection::InterestDividends] {
        let mine: Vec<_> =
            documents.iter().filter(|item| topic_for(&item.document) == topic).cloned().collect();
        let request = save_topic(&session.draft.clone(), topic, mine);
        let reply = send(session, request);
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    }
}
// Opens an income topic's page from its hub, the way the client receives it.
pub(in crate::tests) fn open_topic(session: &mut Interview, topic: TaxSection) -> P {
    let request = P::Navigate {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        from: PAGE_INCOME,
        section: (topic).into(),
        navigation: NavigationAvailability::default(),
    };
    match send(session, request) {
        Reply::Edit(request @ P::SaveIncomes { .. }) => request,
        other => panic!("expected the {topic:?} page, got {other:?}"),
    }
}
// Opens the Wages page.
pub(in crate::tests) fn open_documents(session: &mut Interview) -> P {
    open_topic(session, TaxSection::Wages)
}
// Every income topic page's rows, in the hub's order.
pub(in crate::tests) fn all_documents(session: &mut Interview) -> Vec<IncomeDocumentItem> {
    [TaxSection::Wages, TaxSection::InterestDividends]
        .into_iter()
        .flat_map(|topic| page_documents(&open_topic(session, topic)).to_vec())
        .collect()
}
pub(in crate::tests) fn page_documents(page: &P) -> &[IncomeDocumentItem] {
    match page {
        P::SaveIncomes { documents, .. } => &documents.documents,
        other => panic!("expected the income documents page, got {other:?}"),
    }
}

// Native preview catalogs used by the ordinary income interview producers.
pub(in crate::tests) fn catalog_preview_document(document: IncomeDocument) -> P {
    let mut request = super::survey::survey_preview_document(document);
    let P::PreviewDocument {
        k1_credit_lines,
        research_credit_lines,
        renewable_credit_lines,
        transferable_credit_lines,
        foreign_countries,
        ..
    } = &mut request
    else {
        unreachable!()
    };
    *k1_credit_lines = crate::business_credit_page::k1_credit_lines();
    *research_credit_lines = crate::business_credit_page::research_credit_lines();
    *renewable_credit_lines = crate::business_credit_page::renewable_credit_lines();
    *transferable_credit_lines = crate::business_credit_page::transferable_credit_lines();
    *foreign_countries = crate::foreign_accounts::countries();
    request
}

// Static native catalogs for pure calculation tests. Editor-derived dynamic
// catalogs continue through estimate_documents_from_save.
pub(in crate::tests) fn catalog_estimate_documents(
    topic: TaxSection,
    documents: crate::IncomeDocumentsEstimateInput,
) -> P {
    P::EstimateDocuments {
        initial_rental_recovery_choices: Vec::new(),
        rental_energy_buildings: Vec::new(),
        empty_k1_activity_choices: crate::tests::support::builders::empty_k1_activity_choices(),
        allowed_continuing_risk: ScorpRiskStart::continuing_choices(),
        allowed_injured_source_facts: vec![],
        allowed_injured_retained_owners: vec![],
        ltc_payee_sources: vec![],
        wage_scorporations: Vec::new(),
        allowed_wage_scorporations: Vec::new(),
        initial_k1_investment_choices: alloc::vec![0, 1],
        research_businesses: Vec::new(),
        retained_research_choices: Vec::new(),
        k1_credit_lines: crate::business_credit_page::k1_credit_lines(),
        research_credit_lines: crate::business_credit_page::research_credit_lines(),
        renewable_credit_lines: crate::business_credit_page::renewable_credit_lines(),
        transferable_credit_lines: crate::business_credit_page::transferable_credit_lines(),
        mortgage_loans: Vec::new(),
        allowed_refund_management: vec![],
        user: TEST_USER,
        topic,
        allowed_documents: topic_kinds(topic),
        allowed_owners: vec![Owner::Taxpayer as i32],
        allowed_business_source: vec![],
        allowed_farm_source: vec![],
        farms: vec![],
        cooperative_activities: Vec::new(),
        cooperative_properties: Vec::new(),
        businesses: vec![],
        rentals: vec![],
        royalty_properties: vec![],
        equipment_rentals: vec![],
        hobbies: vec![],
        ccc_lots: vec![],
        farm_codes: vec![],
        farm_limitations: vec![],
        farm_sources: vec![],
        basis_assets: vec![],
        farm_method_changes: vec![],
        inherited_iras: vec![],
        inherited_roth_iras: vec![],
        ltc_insured: vec![],
        annuity_sources: vec![],
        recovery_sources: vec![],
        allowed_partner_source: vec![],
        allowed_home_spouse: vec![],
        people: vec![],
        allowed_other_parent: vec![],
        children: vec![],
        documents,
        allowed_community: Vec::new(),
        foreign_countries: crate::foreign_accounts::countries(),
        interest_statements: Vec::new(),
        qbi_businesses: vec![],
    }
}
