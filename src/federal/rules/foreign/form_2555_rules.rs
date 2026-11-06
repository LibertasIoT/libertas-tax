// form_2555 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct ForeignValidationOut {
    pub(super) foreign: crate::extended_rules::ForeignResult,
    pub(super) foreign_business: bool,
    pub(super) foreign_wages: i64,
}

pub(super) fn foreign_validation_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
) -> ForeignValidationOut {
    let &Pass { d, allocated_entries, ded, a, c, .. } = pass;
    if let Some(key) = crate::foreign_housing::source_pay_rule(d) {
        crate::issue_localized!(
            r,
            &crate::IssueContext::Section(crate::TaxSection::WorkAbroad)
                .alternatives(&[crate::TaxSection::Wages, crate::TaxSection::WorkAbroad]),
            "Foreign employee pay",
            key,
            "Direct prior-year pay and awards cannot exceed this person’s entered W-2 foreign pay and foreign-employer pay.",
        );
    }
    let foreign = crate::extended_rules::foreign(d, r);
    // The K-1 adjustment, the Form 4952 election and the Form 2555 exclusion
    // that together need these worksheets are each answered on their own page.
    let estate_amt = crate::IssueContext::Section(crate::TaxSection::PassThrough).alternatives(&[
        crate::TaxSection::PassThrough,
        crate::TaxSection::OtherItemized,
        crate::TaxSection::WorkAbroad,
    ]);
    crate::require_localized!(
        r,
        !allocated_entries.iter().any(|entry| {
            matches!(&entry.income, FederalIncome::EstateTrustPortfolio { data } if { let data = &crate::FederalEstateTrustPortfolioK1::from(data); data.amt_box12b != 0
                && (foreign.election() || (ded.investment_interest().is_some()
                    && (data.investment_income.is_none() || crate::estate_trust::investment_adjustment(data).is_none()))) })
        })
            || (ded.investment_interest().is_none() && !foreign.election()),
        &estate_amt,"Estate/trust AMT qualified dividends",
        "ISSUE_A_K_1_BOX_12_CODE_B_QUALIFIED_DIVIDEND_ADJUSTMENT_WITH_A_FORM_4952_6794A37F","A K-1 box 12 code B qualified-dividend adjustment with a Form 4952 investment-interest election or Form 2555 exclusion needs coordinated AMT preferential-rate worksheets.",
    );
    crate::require_localized!(
        r,
        !allocated_entries.iter().any(|entry| {
            matches!(&entry.income, FederalIncome::EstateTrustPortfolio { data } if { let data = &crate::FederalEstateTrustPortfolioK1::from(data); data.amt_box12c != 0
                && (foreign.election() || (ded.investment_interest().is_some()
                    && (data.investment_income.is_none() || crate::estate_trust::investment_adjustment(data).is_none()))) })
        })
            || (ded.investment_interest().is_none() && !foreign.election()),
        &estate_amt,"Estate/trust AMT short-term gain",
        "ISSUE_A_K_1_BOX_12_CODE_C_SHORT_TERM_GAIN_ADJUSTMENT_WITH_FORM_4952_INVES_9532A78C","A K-1 box 12 code C short-term gain adjustment with Form 4952 investment interest or Form 2555 exclusion needs coordinated AMT rate worksheets.",
    );
    crate::require_localized!(
        r,
        !allocated_entries.iter().any(|entry| {
            matches!(&entry.income, FederalIncome::EstateTrustPortfolio { data } if { let data = &crate::FederalEstateTrustPortfolioK1::from(data); data.amt_box12d != 0
                && (foreign.election() || (ded.investment_interest().is_some()
                    && (data.investment_income.is_none() || crate::estate_trust::investment_adjustment(data).is_none()))) })
        }) || (ded.investment_interest().is_none() && !foreign.election()),
        &estate_amt,"Estate/trust AMT long-term gain",
        "ISSUE_A_K_1_BOX_12_CODE_D_LONG_TERM_GAIN_ADJUSTMENT_WITH_FORM_4952_INVEST_E7A6C7A9","A K-1 box 12 code D long-term gain adjustment with Form 4952 investment interest or Form 2555 exclusion needs coordinated AMT rate worksheets.",
    );
    crate::require_localized!(
        r,
        !allocated_entries.iter().any(|entry| {
            matches!(&entry.income, FederalIncome::EstateTrustPortfolio { data } if { let data = &crate::FederalEstateTrustPortfolioK1::from(data); data.amt_box12e != 0
                && (foreign.election() || (ded.investment_interest().is_some()
                    && (data.investment_income.is_none() || crate::estate_trust::investment_adjustment(data).is_none()))) })
        }) || (ded.investment_interest().is_none() && !foreign.election()),
        &estate_amt,"Estate/trust AMT unrecaptured section 1250 gain",
        "ISSUE_A_K_1_BOX_12_CODE_E_SECTION_1250_ADJUSTMENT_WITH_FORM_4952_INVESTME_29856F5A","A K-1 box 12 code E section 1250 adjustment with Form 4952 investment interest or Form 2555 exclusion needs coordinated AMT special-rate worksheets.",
    );
    let foreign_business = allocated_entries.iter().any(|e| matches!(&e.income,
        FederalIncome::Business { data } if matches!(data.source, crate::BusinessSource::ForeignServices { .. })));
    // Form 2555 line 44: a Form 2106 deduction of a person who claims Form 2555
    // would be allocable to the excluded pay, which the lines do not figure.
    for (index, owner) in [Owner::Taxpayer, Owner::Spouse].into_iter().enumerate() {
        crate::require_localized!(
            r,
            !foreign.elected[index] || !a.employee_expenses.iter().any(|f| f.owner == owner),
            &crate::IssueContext::owner(crate::TaxSection::WorkAbroad, owner).alternatives(&[
                crate::TaxSection::WorkAbroad,
                crate::TaxSection::EmployeeExpenses,
            ]),
            "Foreign earned income exclusion",
            "ISSUE_FORM_2555_LINE_44_EMPLOYEE_EXPENSES",
            "Form 2555 line 44: job expenses on Form 2106 are allocable to excluded foreign earned income in part. Not calculated; remove the Form 2106 or the Form 2555.",
        );
    }
    let foreign_wages = foreign.wages.iter().sum::<i64>();
    if foreign_business
        || foreign_wages != 0
        || foreign.election()
        || !matches!(c.foreign_tax, crate::ForeignTaxClaim::No)
    {
        r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "Review any applicable foreign-account and information-reporting obligations separately; this estimate prepares no filing forms.", crate::Message::new("NOTE_REVIEW_ANY_APPLICABLE_FOREIGN_ACCOUNT_AND_INFORMATION_REPORTING_BB6EA0B4", alloc::vec![])));
    }
    ForeignValidationOut { foreign, foreign_business, foreign_wages }
}
