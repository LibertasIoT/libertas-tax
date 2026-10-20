use crate::tests::*;

pub(in crate::tests) fn mortgage_certificate_credit(
    rate: u32,
    ownership: u32,
    interest: i64,
) -> FederalMortgageCredit {
    FederalMortgageCredit {
        current: Some(FederalMortgageCertificate {
            issuer: "Synthetic housing agency".into(),
            number: "MCC-TEST".into(),
            issue_date: 20200115,
            interest: MortgageInterestSource::Certified { amount: interest },
            rate_hundredths_percent: rate,
            ownership_basis_points: ownership,
            eligible: true,
            history: MortgageCertificateHistory::Original,
        }),
        carry_three_years_prior: 0,
        carry_two_years_prior: 0,
        carry_prior_year: 0,
        loan: None,
    }
}

pub(in crate::tests) fn set_mortgage_subsidy(
    d: &mut FederalDraft,
    disposition: MortgageSubsidyDisposition,
) {
    d.credits.as_mut().unwrap().mortgage_subsidies =
        vec![MortgageSubsidyEntry { label: "Synthetic home".into(), disposition }];
}

pub(in crate::tests) fn mortgage_disposition(
    d: &mut FederalDraft,
) -> &mut MortgageSubsidyDisposition {
    &mut d.credits.as_mut().unwrap().mortgage_subsidies[0].disposition
}

pub(in crate::tests) fn mortgage_recapture_facts(
    d: &mut FederalDraft,
) -> &mut MortgageSubsidyFacts {
    match mortgage_disposition(d) {
        MortgageSubsidyDisposition::Ordinary { facts, .. } => facts,
        _ => panic!(),
    }
}

pub(in crate::tests) fn mortgage_recapture_sale_id(d: &mut FederalDraft) -> &mut i64 {
    match mortgage_disposition(d) {
        MortgageSubsidyDisposition::Ordinary {
            transfer: MortgageSubsidyTransfer::Sale { home_sale_income_id, .. },
            ..
        } => home_sale_income_id,
        _ => panic!(),
    }
}

pub(in crate::tests) fn mortgage_recapture_qsml(
    d: &mut FederalDraft,
) -> &mut MortgageSubsidyQsmlGainShare {
    match mortgage_disposition(d) {
        MortgageSubsidyDisposition::Ordinary {
            transfer: MortgageSubsidyTransfer::Sale { qsml_gain_share: Some(share), .. },
            ..
        } => share,
        _ => panic!(),
    }
}

pub(in crate::tests) fn ordinary_subsidy_facts() -> MortgageSubsidyFacts {
    MortgageSubsidyFacts {
        original_closed_on: 20200115,
        disposed_on: 20260701,
        notice_reference: "Synthetic issuer notice 2020-01".into(),
        family_size_at_disposition: 1,
        adjusted_qualifying_income: 5_000_000,
        highest_subsidized_principal: 10_000_000,
        holding_percentage: 60,
        payoff: MortgageSubsidyPayoff::AtDispositionOrReissued,
        confirmed: true,
    }
}

// Preserve the former ItemizedExpenses helper's automatic-comparison setup.
pub(in crate::tests) fn fixture_mortgage_interest(deductions: &mut FederalDeductions, amount: i64) {
    itemized(deductions);
    deductions.set_mortgage_interest(amount);
}

pub(in crate::tests) fn identified_mortgages(mut loans: Vec<MortgageLoan>) -> Vec<MortgageLoan> {
    for (index, loan) in loans.iter_mut().enumerate() {
        loan.id = index as i64 + 1;
    }
    loans
}

pub(in crate::tests) fn mortgage_ledger_mut(loan: &mut MortgageLoan) -> &mut MortgageLedger {
    let MortgageLoanDetails::FirstLender(ledger) = &mut loan.details else {
        panic!("expected original loan fixture")
    };
    ledger
}
