#[allow(unused_imports)]
use super::*;

// The refund's federal deduction history. `SeveralYears` (a placeholder the
// page cannot yet detail) is stored as a structurally valid but deliberately
// incomplete IncomeTaxMultiplePriorYears record (allocation_complete false),
// so the estimate leaves the refund out with a notice (unknowns::supported)
// instead of the page rejecting the save.
pub(super) fn refund_history(year: i32, deducted: &RefundDeduction) -> TaxRefundHistory {
    match deducted {
        RefundDeduction::StandardDeduction => TaxRefundHistory::NotDeducted,
        RefundDeduction::SalesTax => TaxRefundHistory::SalesTax,
        RefundDeduction::IncomeTax {
            income_paid,
            salt_paid,
            salt_deducted,
            itemized,
            standard,
            sales_available,
            property_refunded,
        } => TaxRefundHistory::IncomeTax {
            deduction_year: year,
            income_paid: *income_paid,
            sales_available: *sales_available,
            salt_paid: *salt_paid,
            salt_deducted: *salt_deducted,
            itemized: *itemized,
            standard: *standard,
            overall_limit: None,
            property_refunded: *property_refunded,
            ordinary: true,
        },
        RefundDeduction::BusinessExpense { business } => {
            TaxRefundHistory::BusinessExpense { business: *business }
        }
        RefundDeduction::SeveralYears => TaxRefundHistory::IncomeTaxMultiplePriorYears {
            years: alloc::vec![
                TaxRefundYearPayment {
                    payment_year: 2018,
                    payment_for_refunded_return: 1,
                    income_paid: 1,
                    sales_available: 0,
                    salt_paid: 1,
                    salt_deducted: 1,
                    itemized: 1,
                    standard: 0,
                    overall_limit: None,
                    ordinary: false,
                },
                TaxRefundYearPayment {
                    payment_year: 2019,
                    payment_for_refunded_return: 1,
                    income_paid: 1,
                    sales_available: 0,
                    salt_paid: 1,
                    salt_deducted: 1,
                    itemized: 1,
                    standard: 0,
                    overall_limit: None,
                    ordinary: false,
                },
            ],
            allocation_complete: false,
        },
    }
}
// The reverse of refund_history: only the ordinary single-year deduction
// round-trips onto the topic row; NotDeducted/SalesTax/SeveralYears each
// collapse onto their own branch. A record from IncomeTaxPriorYearsAndCurrent
// reopens as SeveralYears because the topic cannot detail its allocation.
pub(super) fn refund_deduction(history: &TaxRefundHistory) -> Option<RefundDeduction> {
    Some(match history {
        TaxRefundHistory::Unanswered => return None,
        TaxRefundHistory::NotDeducted => RefundDeduction::StandardDeduction,
        TaxRefundHistory::SalesTax => RefundDeduction::SalesTax,
        TaxRefundHistory::BusinessExpense { business } => {
            RefundDeduction::BusinessExpense { business: *business }
        }
        TaxRefundHistory::IncomeTax {
            income_paid,
            sales_available,
            salt_paid,
            salt_deducted,
            itemized,
            standard,
            property_refunded,
            ..
        } => RefundDeduction::IncomeTax {
            income_paid: *income_paid,
            sales_available: *sales_available,
            salt_paid: *salt_paid,
            salt_deducted: *salt_deducted,
            itemized: *itemized,
            standard: *standard,
            property_refunded: *property_refunded,
        },
        TaxRefundHistory::IncomeTaxMultiplePriorYears { .. }
        | TaxRefundHistory::IncomeTaxPriorYearsAndCurrent { .. } => RefundDeduction::SeveralYears,
    })
}

// The topic question is a view of the accepted deduction history. An unchanged
// answer keeps the detailed year allocations and prior-return facts it cannot
// display. Editing ordinary amounts does not establish different prior limits
// or confirm a treatment that the accepted facts left unconfirmed.
pub(super) fn apply_refund_deduction(
    source: &mut GovernmentPaymentSource,
    deducted: &RefundDeduction,
) {
    if refund_deduction(&source.history).as_ref() == Some(deducted) {
        return;
    }
    let mut history = refund_history(source.form.refund_year(), deducted);
    if let (
        TaxRefundHistory::IncomeTax { overall_limit, ordinary, .. },
        TaxRefundHistory::IncomeTax {
            overall_limit: accepted_limit,
            ordinary: accepted_ordinary,
            ..
        },
    ) = (&mut history, &mut source.history)
    {
        *overall_limit = accepted_limit.take();
        *ordinary = *accepted_ordinary;
    }
    source.history = history;
}

// Box 8 supplies a visible topic suggestion after the paper has been saved.
// An accepted deduction answer always wins, including StandardDeduction.
// https://www.irs.gov/instructions/i1099g (Box 8)
pub(super) fn refund_topic_deduction(source: &GovernmentPaymentSource) -> Option<RefundDeduction> {
    if matches!(source.history, TaxRefundHistory::Unanswered)
        && source.form.trade_or_business_income
    {
        Some(RefundDeduction::BusinessExpense { business: None })
    } else {
        refund_deduction(&source.history)
    }
}

// The three amounts of the income-tax deduction must agree with each other.
pub(super) fn refund_deduction_issue(deducted: &RefundDeduction) -> Option<ValidationIssue> {
    let RefundDeduction::IncomeTax {
        income_paid, salt_paid, salt_deducted, property_refunded, ..
    } = deducted
    else {
        return None;
    };
    if salt_deducted > salt_paid {
        Some(ValidationIssue::new("REFUND_SALT_DEDUCTED_ABOVE_PAID").at_field("salt_deducted"))
    } else if income_paid > salt_paid {
        Some(ValidationIssue::new("REFUND_INCOME_TAX_ABOVE_SALT").at_field("income_paid"))
    } else if *property_refunded > salt_paid - income_paid {
        Some(
            ValidationIssue::new("REFUND_PROPERTY_ABOVE_OTHER_TAXES").at_field("property_refunded"),
        )
    } else {
        None
    }
}
