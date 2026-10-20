use crate::tests::*;

// Ordinary paid foreign-credit sources with the shared neutral ancillary facts.
pub(in crate::tests) fn survey_ordinary_foreign_credit(
    sources: Vec<ForeignOrdinaryTax>,
) -> ForeignTaxClaim {
    ForeignTaxClaim::Ordinary {
        sources,
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::Unreviewed,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        amt_limitation: ForeignAmtLimitation::Normal,
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment: ForeignRateAdjustment::Exception,
    }
}

// Reuse the canonical cash-business facts when a fixture varies only its explicit fields.
pub(in crate::tests) fn survey_cash_business_data() -> FederalBusiness {
    let FederalIncome::Business { data } = cash_business(0) else { unreachable!() };
    data
}

// Native rows without output catalogs, used by the medical-document source probes.
pub(in crate::tests) fn survey_uncatalogued_document(
    owner: Owner,
    id: i64,
    document: IncomeDocument,
) -> IncomeDocumentItem {
    IncomeDocumentItem {
        refund_management: None,
        refund_management_needed: None,
        allowed_community: Vec::new(),
        investment_caps: None,
        id,
        owner,
        document,
        preview: None,
        community: CommunityShare::NotNeeded,
    }
}
