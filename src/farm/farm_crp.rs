use crate::FederalResult;
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::line_emission::emit as emit_lines;

use libertas_macros::*;

/// Conservation Reserve Program payment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CrpPayment {
    /// Taxable annual or incentive payment
    /// Enter each payment once, outside other farm receipts. Use 2026 income under the farm's accounting method. Section 126 cost-sharing exclusions and nonbusiness landowner treatment require separate worksheets; do not enter those here. These payments are not compensation for personal services in a mixed-income classification.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Receiving Social Security retirement or disability benefits when this payment was received
    /// Use the recipient's actual benefit status on the payment date, not age or merely receiving benefits later in the year. Confirm the retirement/disability condition specifically rather than assuming any government benefit qualifies. Enter any reportable Social Security benefits separately. The SSA-1099 or the award letter shows the benefit start date. No keeps the payment in self-employment earnings.
    pub benefits: bool,
    /// Federal income tax withheld in 2026
    /// Actual current-year withholding, not withholding already claimed or entered elsewhere.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
}
impl CrpPayment {
    pub(crate) fn valid(&self) -> bool {
        (0..=100000000000).contains(&self.amount) && (0..=100000000000).contains(&self.withholding)
    }
    pub(crate) fn se_exclusion(&self) -> i64 {
        if self.benefits { self.amount } else { 0 }
    }
}
pub(crate) fn report(payments: &[CrpPayment], id: i64, r: &mut FederalResult) {
    if payments.is_empty() {
        return;
    }
    emit_lines(
        r,
        &mut (),
        [
            (
                "income",
                payments.iter().map(|p| p.amount).sum(),
                msg!("EXPLAIN_FARM_N_CRP_N_INCOME"),
            ),
            (
                "se_exclusion",
                payments.iter().map(CrpPayment::se_exclusion).sum(),
                msg!("EXPLAIN_FARM_N_CRP_N_SE_EXCLUSION"),
            ),
            (
                "withholding",
                payments.iter().map(|p| p.withholding).sum(),
                msg!("EXPLAIN_FARM_N_CRP_N_WITHHOLDING"),
            ),
        ],
        |(name, amount, note), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("farm_{id}_crp_{name}"),
                LineValue::MoneyCents(amount),
                &["income.business.activity.farming.crp"],
                "https://www.irs.gov/instructions/i1040sse",
                note,
            ))
        },
    );
}

// Community-property projections preserve accepted source identity and facts.
crate::community::divide_record!(CrpPayment {
    amounts: [amount, withholding],
    gains: [],
    facts: [benefits],
});
