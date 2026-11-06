// form_1099_oid items of src/tests/support/builders/native_income.rs, split out by tools/split-hubs.
use super::super::paper_documents::remember;
#[allow(unused_imports)]
use super::*;

pub(in crate::tests) fn native_oid1099(
    payer: alloc::string::String,
    oid: i64,
    other_periodic_interest: i64,
) -> Oid1099 {
    Oid1099 {
        payer: crate::PrintedParty::from_name(payer),
        recipient: Default::default(),
        oid,
        other_periodic_interest,
        more: OidMore::default(),
        state_details: Default::default(),
        nominee: None,
        adjustments: None,
    }
}

pub(in crate::tests) fn oid_source(data: FederalOid) -> Form1099OidSource {
    let FederalOid {
        tax_exempt_bond_premium,
        nominee,
        adjustments,
        investment_expenses,
        oid,
        other_periodic_interest,
        early_penalty,
        withholding,
        market_discount,
        acquisition_premium,
        exempt,
        bond_premium,
        private_activity,
        special,
        treasury_oid,
        printed_facts,
    } = data;
    let OidPrintedFacts {
        corrected,
        payer_telephone,
        description,
        fatca_filing_requirement,
        account_number,
    } = printed_facts;
    Form1099OidSource {
        form: Form1099OidPaper {
            calendar_year: Default::default(),
            corrected,
            payer: Default::default(),
            payer_telephone,
            recipient: Default::default(),
            original_issue_discount_for_the_year: oid,
            other_periodic_interest,
            early_withdrawal_penalty: early_penalty,
            federal_income_tax_withheld: withholding,
            market_discount,
            acquisition_premium,
            description,
            original_issue_discount_on_us_treasury_obligations: treasury_oid,
            investment_expenses,
            bond_premium,
            tax_exempt_oid: exempt,
            fatca_filing_requirement,
            account_number,
            state_details: Default::default(),
        },
        calls: Default::default(),
        nominee: nominee.map(Into::into),
        adjustments,
        tax_exempt_bond_premium,
        private_activity,
        special: Some(special),
    }
}

pub(in crate::tests) fn oid_document(source: impl OidPaperFixture) -> Form1099OidPaper {
    if let Some(form) = source.existing_paper() {
        return form;
    }
    let source = source.oid_source();
    let form = source.form.clone();
    remember(
        IncomeSource::InterestDividend { value: InterestDividendDocument::Oid1099(form.clone()) },
        vec![
            DocumentAnswer::OidTreatment { value: source.special },
            DocumentAnswer::OidNominee { value: source.nominee },
            DocumentAnswer::InterestAdjustment { value: source.adjustments },
            DocumentAnswer::OidPremium { value: source.tax_exempt_bond_premium },
            DocumentAnswer::OidPrivateActivity { value: source.private_activity },
        ],
    );
    form
}

pub(in crate::tests) trait OidPaperFixture {
    fn existing_paper(&self) -> Option<Form1099OidPaper> {
        None
    }
    fn oid_source(self) -> Form1099OidSource;
}

impl OidPaperFixture for Form1099OidSource {
    fn oid_source(self) -> Form1099OidSource {
        self
    }
}

impl OidPaperFixture for Oid1099 {
    fn oid_source(self) -> Form1099OidSource {
        let mut source = oid_source(FederalOid::from(&self));
        source.form.payer = self.payer;
        source.form.recipient = self.recipient;
        source.form.state_details = self.state_details;
        source.calls = self.more.calls;
        source
    }
}

impl OidPaperFixture for Form1099OidPaper {
    fn existing_paper(&self) -> Option<Form1099OidPaper> {
        Some(self.clone())
    }
    fn oid_source(self) -> Form1099OidSource {
        unreachable!("paper passes through unchanged")
    }
}
