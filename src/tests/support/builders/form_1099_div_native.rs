// form_1099_div items of src/tests/support/builders/native_income.rs, split out by tools/split-hubs.
use super::super::paper_documents::remember;
#[allow(unused_imports)]
use super::*;

pub(in crate::tests) fn native_div1099(
    payer: alloc::string::String,
    ordinary: i64,
    qualified: i64,
) -> Div1099 {
    Div1099 {
        payer: crate::PrintedParty::from_name(payer),
        ordinary,
        qualified,
        ..Default::default()
    }
}

pub(in crate::tests) fn div1099_source(form: Div1099) -> DividendSource {
    let mut source = dividend_source(FederalDividend {
        ordinary: form.ordinary,
        qualified: form.qualified,
        nondividend_distributions: form.nondividend_distributions,
        cash_liquidation: form.cash_liquidation,
        noncash_liquidation: form.noncash_liquidation,
        section_897_ordinary: form.section_897_ordinary,
        section_897_capital: form.section_897_capital,
        foreign_country: form.foreign_country.clone(),
        capital_distributions: form.capital_distributions,
        unrecaptured_distributions: form.unrecaptured_distributions,
        collectible_distributions: form.collectible_distributions,
        section_1202_gain: form.section_1202_gain,
        withholding: form.withholding,
        section199a: if form.section199a == 0 {
            ReitDividends::None
        } else {
            ReitDividends::Reported { amount: form.section199a, eligible: None }
        },
        foreign_tax: form.foreign_tax,
        exempt_dividends: form.exempt_dividends,
        private_activity_dividends: form.private_activity_dividends,
        ..Default::default()
    });
    source.form.payer = form.payer;
    source.form.recipient = form.recipient;
    source.form.state_details = form.state_details;
    source
}

// Calculation fixtures keep their native cents; accepted entries retain the paper and answers.
pub(in crate::tests) fn dividend_source(data: FederalDividend) -> DividendSource {
    let FederalDividend {
        nondividend_distributions,
        cash_liquidation,
        noncash_liquidation,
        capital_returns,
        nominee,
        section_897_ordinary,
        section_897_capital,
        foreign_country,
        ordinary,
        qualified,
        section199a,
        capital_distributions,
        collectible_distributions,
        unrecaptured_distributions,
        section_1202_gain,
        section_1202_exclusion,
        withholding,
        special,
        exempt_dividends,
        private_activity_dividends,
        foreign_tax,
        after_death,
    } = data;
    let (section199a, reit_eligible) = match section199a {
        ReitDividends::None => (0, None),
        ReitDividends::Reported { amount, eligible } => (amount, eligible.map(Into::into)),
    };
    DividendSource {
        form: Form1099DivPaper {
            nondividend_distributions: Some(nondividend_distributions),
            cash_liquidation: Some(cash_liquidation),
            noncash_liquidation: Some(noncash_liquidation),
            section_897_ordinary: Some(section_897_ordinary),
            section_897_capital: Some(section_897_capital),
            foreign_country,
            ordinary: Some(ordinary),
            qualified: Some(qualified),
            section199a: Some(section199a),
            capital_distributions: Some(capital_distributions),
            collectible_distributions: Some(collectible_distributions),
            unrecaptured_distributions: Some(unrecaptured_distributions),
            section_1202_gain: Some(section_1202_gain),
            withholding: Some(withholding),
            exempt_dividends: Some(exempt_dividends),
            private_activity_dividends: Some(private_activity_dividends),
            foreign_tax: Some(foreign_tax),
            ..Default::default()
        },
        calls: Default::default(),
        capital_returns,
        nominee: nominee.map(Into::into),
        reit_eligible,
        after_death: after_death.map(Into::into),
        section_1202_exclusion,
        special,
    }
}

pub(in crate::tests) fn dividend_document(source: impl DividendPaperFixture) -> Form1099DivPaper {
    use crate::DocumentAnswer as A;
    if let Some(form) = source.existing_paper() {
        return form;
    }
    let source = source.dividend_source();
    let form = source.form;
    remember(
        IncomeSource::InterestDividend { value: InterestDividendDocument::Div1099(form.clone()) },
        vec![
            A::DividendNominee { value: source.nominee },
            A::DividendAfterDeath { value: source.after_death },
            A::DividendEligibility { value: source.reit_eligible },
            A::DividendFundExclusion { value: source.section_1202_exclusion },
            A::DividendTreatment { value: source.special },
            A::DividendCapitalReturns { value: source.capital_returns },
        ],
    );
    form
}

pub(in crate::tests) trait DividendPaperFixture {
    fn existing_paper(&self) -> Option<Form1099DivPaper> {
        None
    }
    fn dividend_source(self) -> DividendSource;
}

impl DividendPaperFixture for DividendSource {
    fn dividend_source(self) -> DividendSource {
        self
    }
}

impl DividendPaperFixture for Div1099 {
    fn dividend_source(self) -> DividendSource {
        div1099_source(self)
    }
}

impl DividendPaperFixture for Form1099DivPaper {
    fn existing_paper(&self) -> Option<Form1099DivPaper> {
        Some(self.clone())
    }
    fn dividend_source(self) -> DividendSource {
        unreachable!("paper passes through unchanged")
    }
}

// One printed DIV carries its capital lot answer through the ordinary topic.
pub(in crate::tests) fn capital_return_document(
    payer: crate::PrintedParty,
    recipient: crate::PrintedParty,
    lot: crate::CapitalReturn1099Div,
) -> Form1099DivPaper {
    let noncash_liquidation =
        crate::kernel::ratio::checked_sum(lot.property.iter().map(|property| property.value))
            .expect("capital-return property total fits exact cents");
    let (nondividend_distributions, cash_liquidation) = match lot.kind {
        crate::CapitalDistributionKind::Nondividend => (lot.amount, 0),
        crate::CapitalDistributionKind::Liquidation
        | crate::CapitalDistributionKind::InterimLiquidation
        | crate::CapitalDistributionKind::FinalLiquidation => (0, lot.amount),
    };
    let form = Form1099DivPaper {
        payer,
        recipient,
        nondividend_distributions: Some(nondividend_distributions),
        cash_liquidation: Some(cash_liquidation),
        noncash_liquidation: Some(noncash_liquidation),
        ..Default::default()
    };
    remember(
        IncomeSource::InterestDividend { value: InterestDividendDocument::Div1099(form.clone()) },
        vec![crate::DocumentAnswer::DividendCapitalReturns { value: vec![lot] }],
    );
    form
}
