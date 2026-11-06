use crate::dates::{source_valid_date as valid_date, valid_birth};
use crate::kernel::ratio::exact::{self, Exact};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Duration specified by the reserve call-up orders
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ReserveTerm {
    /// Indefinite period
    Indefinite,
    /// Specified number of days
    Days {
        /// Time ordered, not time already served
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("day")]
        days: u32,
    },
}

/// Original distribution and its 2026 repayments
/// A distribution taken while on active duty of at least 180 days (or indefinite) after September 11, 2001, repaid to a traditional IRA within two years after the active-duty period. Repayment adds nondeductible basis; it does not reverse the original income or use the contribution limit.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ReservistRepayment {
    /// Date ordered or called to active duty
    #[libertas_date_only]
    // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
    #[libertas_number(min = 20010912, max = 20261231)]
    pub ordered: u32,
    /// Ordered service duration
    pub term: ReserveTerm,
    /// Last day of this active-duty period
    /// Repayment is allowed through the second anniversary of this date. A February 29 release has a February 28 deadline in the second year.
    #[libertas_date_only]
    // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
    #[libertas_number(min = 20010912, max = 20261231)]
    pub released: u32,
    /// Original distribution date
    #[libertas_date_only]
    // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
    #[libertas_number(min = 20010912, max = 20261231)]
    pub distributed: u32,
    /// Original qualified reservist distribution
    /// From an IRA or your elective deferrals under a 401(k), 403(b) or similar plan, as the distribution records show it.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub gross: i64,
    /// Repayments of this distribution completed before 2026
    /// Their basis is already in your prior Form 8606 amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub prior: i64,
    /// 2026 repayments
    /// Deposits the IRA trustee accepted as qualified reservist repayments (Form 5498 box 14 code QR). Not a regular contribution, a Roth deposit or a repayment of another distribution.
    #[libertas_size(min = 1)]
    /// ----
    /// Repayment accepted by the IRA trustee
    pub payments: Vec<ReservistPayment>,
}

/// Qualified reservist repayment payment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ReservistPayment {
    /// Actual deposit date in 2026
    #[libertas_date_only]
    pub date: u32,
    /// Amount deposited
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
}
impl ReservistRepayment {
    // Relationships checked when accepting the source record.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        let issue = |key, field| crate::validation::ValidationIssue::new(key).at_field(field);
        if !(self.ordered <= self.distributed && self.distributed <= self.released) {
            Some(issue("RESERVIST_DISTRIBUTION_OUTSIDE_DUTY", "distributed"))
        } else if let Some((index, _)) = self
            .payments
            .iter()
            .enumerate()
            .find(|(_, p)| !(p.date > self.released && p.date <= self.repayment_deadline()))
        {
            Some(issue("RESERVIST_REPAYMENT_LATE", "date").in_list("payments", index))
        } else if exact::whole(self.prior) + self.amount_exact() > exact::whole(self.gross) {
            Some(issue("RESERVIST_REPAYMENT_ABOVE_GROSS", "prior"))
        } else {
            None
        }
    }
    pub(crate) fn valid(&self) -> bool {
        [self.ordered, self.released, self.distributed].into_iter().all(valid_birth)
            && self.gross >= 0
            && self.prior >= 0
            && match self.term {
                ReserveTerm::Indefinite => true,
                ReserveTerm::Days { days } => (1..=36500).contains(&days),
            }
            && !self.payments.is_empty()
            && self
                .payments
                .iter()
                .all(|p| valid_date(p.date) && p.date / 10000 == 2026 && p.amount >= 0)
    }
    pub(crate) fn amount_exact(&self) -> Exact {
        self.payments.iter().map(|p| exact::whole(p.amount)).sum()
    }
    pub(crate) fn repayment_deadline(&self) -> u32 {
        let mut deadline = self.released + 20000;
        // February 29's two-year anniversary falls on February 28.
        while !valid_date(deadline) {
            deadline -= 1;
        }
        deadline
    }
}
