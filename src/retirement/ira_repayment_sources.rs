use crate::dates::source_valid_date as valid_date;
use crate::kernel::ratio::exact::{self, Exact};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Repayment of a special 2026 traditional IRA distribution
/// For qualified birth/adoption, emergency, domestic-abuse or terminal-illness distributions only. Keep the original gross distribution in Income and select the owner-level IRA basis worksheet. Repayments reduce taxable income under the separate Form 8606 worksheet; they are not regular contributions or ordinary 60-day rollovers.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IraRepayment {
    /// Original 2026 retirement Income entry ID
    /// Must belong to this IRA owner and use one of the four supported exceptions. Use one record for all repayments of this entry. Prior-year distributions and employer-plan, Roth or inherited sources need their separate treatment.
    #[libertas_number(min = 1)]
    pub income: i64,
    /// Actual deposits accepted as repayment
    /// Include later-year deposits only after they have actually occurred; these may require amending the original return. Never enter planned deposits. Amounts cannot exceed the original qualified gross amount.
    #[libertas_size(min = 1)]
    /// ----
    /// Deposit to this owner's eligible traditional IRA
    pub payments: Vec<IraRepaymentDeposit>,
    /// Eligible original distribution and accepted traditional IRA repayments confirmed
    /// Check the receiving trustee’s repayment acceptance records and original distribution statement.
    /// The receiving trustee accepted these as repayments of the linked distribution within its three-year period. No refunded or duplicated repayment, regular contribution, Roth conversion, disaster distribution or other repayment treatment. Include 2026 deposits in the actual December 31 value; do not adjust that value yourself. No credits none of these deposits as repayments, so the original distribution stays taxable.
    pub confirmed: bool,
}

/// Special IRA repayment deposit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IraRepaymentDeposit {
    /// Actual date deposited
    #[libertas_date_only]
    pub date: u32,
    /// Amount deposited
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
}
impl IraRepayment {
    pub(crate) fn valid(&self) -> bool {
        self.income > 0
            && !self.payments.is_empty()
            && self.payments.iter().all(|p| {
                valid_date(p.date) && (2026..=2029).contains(&(p.date / 10000)) && p.amount >= 0
            })
    }
    pub(crate) fn amount_exact(&self) -> Exact {
        deposit_amount(&self.payments)
    }
}
pub(crate) fn deposit_amount(payments: &[IraRepaymentDeposit]) -> Exact {
    payments.iter().map(|p| exact::whole(p.amount)).sum()
}
