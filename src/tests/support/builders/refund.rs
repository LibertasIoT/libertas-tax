use crate::tests::*;

pub(in crate::tests) fn refund_history() -> TaxRefundHistory {
    TaxRefundHistory::IncomeTax {
        deduction_year: 2025,
        income_paid: 1000000,
        sales_available: 500000,
        salt_paid: 1000000,
        salt_deducted: 1000000,
        itemized: 1675000,
        standard: 1575000,
        overall_limit: None,
        property_refunded: 0,
        ordinary: true,
    }
}

pub(in crate::tests) fn two_year_refund_history() -> TaxRefundHistory {
    TaxRefundHistory::IncomeTaxMultiplePriorYears {
        years: vec![
            TaxRefundYearPayment {
                payment_year: 2024,
                payment_for_refunded_return: 750_000,
                income_paid: 1_000_000,
                sales_available: 0,
                salt_paid: 1_000_000,
                salt_deducted: 1_000_000,
                itemized: 2_000_000,
                standard: 1_460_000,
                overall_limit: None,
                ordinary: true,
            },
            TaxRefundYearPayment {
                payment_year: 2025,
                payment_for_refunded_return: 250_000,
                income_paid: 250_000,
                sales_available: 0,
                salt_paid: 250_000,
                salt_deducted: 250_000,
                itemized: 1_625_000,
                standard: 1_575_000,
                overall_limit: None,
                ordinary: true,
            },
        ],
        allocation_complete: true,
    }
}

pub(in crate::tests) fn three_year_refund_history() -> TaxRefundHistory {
    TaxRefundHistory::IncomeTaxMultiplePriorYears {
        years: vec![
            TaxRefundYearPayment {
                payment_year: 2023,
                payment_for_refunded_return: 500_000,
                income_paid: 1_000_000,
                sales_available: 0,
                salt_paid: 1_000_000,
                salt_deducted: 1_000_000,
                itemized: 2_000_000,
                standard: 1_385_000,
                overall_limit: None,
                ordinary: true,
            },
            TaxRefundYearPayment {
                payment_year: 2024,
                payment_for_refunded_return: 300_000,
                income_paid: 1_000_000,
                sales_available: 0,
                salt_paid: 1_200_000,
                salt_deducted: 1_000_000,
                itemized: 2_000_000,
                standard: 1_460_000,
                overall_limit: None,
                ordinary: true,
            },
            TaxRefundYearPayment {
                payment_year: 2025,
                payment_for_refunded_return: 200_000,
                income_paid: 200_000,
                sales_available: 0,
                salt_paid: 200_000,
                salt_deducted: 200_000,
                itemized: 1_585_000,
                standard: 1_575_000,
                overall_limit: None,
                ordinary: true,
            },
        ],
        allocation_complete: true,
    }
}

pub(in crate::tests) fn prior_and_current_refund_history(
    gross_current_paid: i64,
) -> TaxRefundHistory {
    TaxRefundHistory::IncomeTaxPriorYearsAndCurrent {
        prior_years: vec![TaxRefundYearPayment {
            payment_year: 2025,
            payment_for_refunded_return: 300_000,
            income_paid: 1_000_000,
            sales_available: 0,
            salt_paid: 1_000_000,
            salt_deducted: 1_000_000,
            itemized: 2_000_000,
            standard: 1_575_000,
            overall_limit: None,
            ordinary: true,
        }],
        current_payment_for_refunded_return: 100_000,
        gross_current_income_tax_paid: gross_current_paid,
        allocation_complete: true,
    }
}

pub(in crate::tests) fn multiple_prior_and_current_refund_history(
    count: usize,
) -> TaxRefundHistory {
    let prior_years = (0..count)
        .map(|index| {
            let year = 2026 - count as i32 + index as i32;
            let standard = match year {
                2019 => 1_220_000,
                2020 => 1_240_000,
                2021 => 1_255_000,
                2022 => 1_295_000,
                2023 => 1_385_000,
                2024 => 1_460_000,
                2025 => 1_575_000,
                _ => unreachable!(),
            };
            TaxRefundYearPayment {
                payment_year: year,
                payment_for_refunded_return: 100_000,
                income_paid: if year == 2024 { 1_000_000 } else { 100_000 },
                sales_available: 0,
                salt_paid: if year == 2024 { 1_200_000 } else { 100_000 },
                salt_deducted: if year == 2024 { 1_000_000 } else { 100_000 },
                itemized: if year == 2025 { 1_580_000 } else { 2_000_000 },
                standard,
                overall_limit: None,
                ordinary: true,
            }
        })
        .collect();
    TaxRefundHistory::IncomeTaxPriorYearsAndCurrent {
        prior_years,
        current_payment_for_refunded_return: 100_000,
        gross_current_income_tax_paid: 2_000_000,
        allocation_complete: true,
    }
}

pub(in crate::tests) fn multiple_prior_and_current_refund_draft(count: usize) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    itemized(d.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
        paid: 1_990_000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    add(
        &mut d,
        income(FederalIncome::GovernmentPayment {
            data: tax_refund_source(FederalTaxRefund {
                amount: (count as i64 + 1) * 10_000,
                withholding: 0,
                history: multiple_prior_and_current_refund_history(count),
            }),
        }),
    );
    d
}

pub(in crate::tests) fn current_year_refund_draft(
    gross_current_paid: i64,
    net_current_paid: i64,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    itemized(d.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
        paid: net_current_paid,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    add(
        &mut d,
        income(FederalIncome::GovernmentPayment {
            data: tax_refund_source(FederalTaxRefund {
                amount: 40_000,
                withholding: 0,
                history: prior_and_current_refund_history(gross_current_paid),
            }),
        }),
    );
    d
}

pub(in crate::tests) fn historical_refund_payment(year: i32) -> TaxRefundYearPayment {
    let standard = match year {
        2010 => 570_000,
        2011 => 580_000,
        2012 => 595_000,
        2013 => 610_000,
        2014 => 620_000,
        2015 | 2016 => 630_000,
        2017 => 635_000,
        2018 => 1_200_000,
        2019 => 1_220_000,
        2020 => 1_240_000,
        2021 => 1_255_000,
        2022 => 1_295_000,
        2023 => 1_385_000,
        2024 => 1_460_000,
        2025 => 1_575_000,
        _ => unreachable!(),
    };
    TaxRefundYearPayment {
        payment_year: year,
        payment_for_refunded_return: 100_000,
        income_paid: 1_000_000,
        sales_available: 0,
        salt_paid: if year == 2024 { 1_200_000 } else { 1_000_000 },
        salt_deducted: 1_000_000,
        itemized: if year == 2025 { 1_580_000 } else { 2_000_000 },
        standard,
        overall_limit: None,
        ordinary: true,
    }
}

pub(in crate::tests) fn historical_pease_payment(
    year: i32,
    eighty_percent_branch: bool,
) -> TaxRefundYearPayment {
    let threshold = match year {
        2013 => 25_000_000,
        2014 => 25_420_000,
        2015 => 25_825_000,
        2016 => 25_940_000,
        2017 => 26_150_000,
        _ => unreachable!(),
    };
    let mut payment = historical_refund_payment(year);
    payment.itemized = if eighty_percent_branch { 1_200_000 } else { 1_970_000 };
    payment.overall_limit = Some(PeaseWorksheet {
        agi: if eighty_percent_branch { 60_000_000 } else { threshold + 1_000_000 },
        before_limit: 2_000_000,
        protected: 1_000_000,
        filing_status: PeaseFilingStatus::Single,
    });
    payment
}

// Retained Schedule A facts for the pre-TCJA refund scenarios. All amounts
// remain cents, and the optional Pease worksheet is supplied by the case.
pub(in crate::tests) fn historical_refund_history(
    deduction_year: i32,
    itemized: i64,
    standard: i64,
    overall_limit: Option<PeaseWorksheet>,
) -> TaxRefundHistory {
    TaxRefundHistory::IncomeTax {
        deduction_year,
        income_paid: 1_000_000,
        sales_available: 500_000,
        salt_paid: 1_000_000,
        salt_deducted: 1_000_000,
        itemized,
        standard,
        overall_limit,
        property_refunded: 0,
        ordinary: true,
    }
}
