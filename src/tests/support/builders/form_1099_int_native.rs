// form_1099_int items of src/tests/support/builders/native_income.rs, split out by tools/split-hubs.
use super::super::paper_documents::remember;
#[allow(unused_imports)]
use super::*;
// Client document values use native cents and remain distinct from FederalIncome fixtures.
pub(in crate::tests) fn native_int1099(payer: alloc::string::String, taxable: i64) -> Int1099 {
    Int1099 {
        payer: crate::PrintedParty::from_name(payer),
        recipient: Default::default(),
        taxable,
        more: IntMore::default(),
        state_details: Default::default(),
        after_death: None,
        nominee: None,
        adjustments: None,
    }
}

pub(in crate::tests) fn private_loan_interest(residence: bool) -> IncomeSource {
    IncomeSource::Record {
        value: IncomeRecord::PrivateLoanInterest(PrivateLoanInterest {
            borrower: "Synthetic borrower".into(),
            interest: 140_000,
            seller_mortgage: residence.then(|| PrivateLoanResidence {
                buyer_tin: "012345678".into(),
                buyer_address: "100 Example St, Example, CA 90001".into(),
            }),
        }),
    }
}

pub(in crate::tests) fn interest_source(data: FederalInterest) -> InterestSource {
    if data.private_loan || data.seller_mortgage.is_some() {
        return InterestSource::Record(data);
    }
    let FederalInterest {
        private_loan: _,
        seller_mortgage: _,
        nominee,
        adjustments,
        investment_expenses,
        market_discount,
        foreign_country,
        taxable,
        us_obligations,
        early_penalty,
        exempt,
        private_activity,
        withholding,
        special,
        bond_premium,
        treasury_bond_premium,
        tax_exempt_bond_premium,
        accrued_interest,
        foreign_tax,
        after_death,
        printed_facts,
    } = data;
    let InterestPrintedFacts {
        corrected,
        payer_telephone,
        payer_rtn,
        fatca_filing_requirement,
        account_number,
        tax_exempt_and_tax_credit_bond_cusip_no,
    } = printed_facts;
    InterestSource::Int1099(Form1099IntSource {
        form: Form1099IntPaper {
            calendar_year: Default::default(),
            corrected,
            payer: Default::default(),
            payer_telephone,
            payer_rtn,
            recipient: Default::default(),
            interest_income: taxable,
            early_withdrawal_penalty: early_penalty,
            us_savings_bond_and_treasury_interest: us_obligations,
            federal_income_tax_withheld: withholding,
            investment_expenses,
            foreign_tax_paid: foreign_tax,
            foreign_country_or_us_territory: foreign_country,
            tax_exempt_interest: exempt,
            specified_private_activity_bond_interest: private_activity,
            market_discount,
            bond_premium,
            bond_premium_on_treasury_obligations: treasury_bond_premium,
            bond_premium_on_tax_exempt_bond: tax_exempt_bond_premium,
            fatca_filing_requirement,
            account_number,
            tax_exempt_and_tax_credit_bond_cusip_no,
            state_details: Default::default(),
        },
        calls: Default::default(),
        nominee: nominee.map(Into::into),
        adjustments,
        after_death: after_death.map(Into::into),
        accrued_interest,
        special,
    })
}

pub(in crate::tests) fn interest_document(source: impl InterestPaperFixture) -> Form1099IntPaper {
    if let Some(form) = source.existing_paper() {
        return form;
    }
    let source = source.interest_source();
    let form = source.form;
    remember(
        IncomeSource::InterestDividend { value: InterestDividendDocument::Int1099(form.clone()) },
        vec![
            DocumentAnswer::InterestNominee { value: source.nominee },
            DocumentAnswer::InterestAfterDeath { value: source.after_death },
            DocumentAnswer::InterestAdjustment { value: source.adjustments },
            DocumentAnswer::InterestAccrued { value: source.accrued_interest },
            DocumentAnswer::InterestTreatment { value: source.special },
        ],
    );
    form
}

pub(in crate::tests) trait InterestPaperFixture {
    fn existing_paper(&self) -> Option<Form1099IntPaper> {
        None
    }
    fn interest_source(self) -> Form1099IntSource;
}

impl InterestPaperFixture for Form1099IntSource {
    fn interest_source(self) -> Form1099IntSource {
        self
    }
}

impl InterestPaperFixture for Int1099 {
    fn interest_source(self) -> Form1099IntSource {
        let InterestSource::Int1099(mut source) = interest_source(FederalInterest::from(&self))
        else {
            unreachable!("printed interest fixture")
        };
        source.form.payer = self.payer;
        source.form.recipient = self.recipient;
        source.form.state_details = self.state_details;
        source.calls = self.more.calls;
        source
    }
}

impl InterestPaperFixture for Form1099IntPaper {
    fn existing_paper(&self) -> Option<Form1099IntPaper> {
        Some(self.clone())
    }
    fn interest_source(self) -> Form1099IntSource {
        unreachable!("paper passes through unchanged")
    }
}
