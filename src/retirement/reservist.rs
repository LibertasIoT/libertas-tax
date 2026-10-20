use crate::federal_rules::valid_date;
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::ratio::exact::{self, Exact};
use crate::{FederalResult, ReserveTerm};
use alloc::{format, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

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
    #[libertas_number(min = 1)]
    pub gross: i64,
    /// Repayments of this distribution completed before 2026
    /// Their basis is already in your prior Form 8606 amount.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub prior: i64,
    /// 2026 repayments
    /// Deposits the IRA trustee accepted as qualified reservist repayments (Form 5498 box 14 code QR). Not a regular contribution, a Roth deposit or a repayment of another distribution.
    /// ----
    /// Repayment accepted by the IRA trustee
    #[libertas_size(min = 1)]
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
    #[libertas_number(min = 0)]
    pub amount: i64,
}
impl ReservistRepayment {
    // The client rules of the record, checked again on save.
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
        [self.ordered, self.released, self.distributed]
            .into_iter()
            .all(crate::federal_rules::valid_birth)
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
    pub(crate) fn report(&self, r: &mut FederalResult, owner: &str, index: usize) {
        let deadline = self.repayment_deadline();
        let amount = self.amount_exact();
        let valid = self.ordered > 20010911
            && self.ordered <= self.distributed
            && self.distributed <= self.released
            && match self.term {
                ReserveTerm::Indefinite => true,
                ReserveTerm::Days { days } => days >= 180,
            }
            && self.payments.iter().all(|p| p.date > self.released && p.date <= deadline)
            && exact::whole(self.prior) + &amount <= exact::whole(self.gross);
        crate::require_localized!(
            r,
            valid,
            &crate::IssueContext::Review.manual(),
            "Reservist IRA repayment",
            "ISSUE_CONFIRM_A_QUALIFIED_DISTRIBUTION_DURING_ELIGIBLE_RESERVE_DUTY_AN_OR_0A51B128",
            "Confirm a qualified distribution during eligible reserve duty, an order for at least 180 days or indefinitely, and actual traditional IRA repayments after release and within two years. All repayments together cannot exceed that original distribution.",
        );
        // An invalid distribution retains its diagnostic, not a fabricated basis row.
        if !valid {
            return;
        }
        // The original-distribution ceiling proves both actual cent rows fit.
        let amount = exact::rounded(&amount).expect("repayment bounded by original gross");
        emit_lines(
            r,
            &mut (),
            [
                ("basis", amount, msg!("EXPLAIN_RESERVIST_N_N_N_BASIS")),
                (
                    "unrepaid",
                    self.gross - self.prior - amount,
                    msg!("EXPLAIN_RESERVIST_N_N_N_UNREPAID"),
                ),
            ],
            |(suffix, value, note), _, _| {
                Some(calculation_row(
                    None,
                    &format!("reservist_{owner}_{index}_{suffix}"),
                    LineValue::MoneyCents(value),
                    &["adjustments.ira_basis.reservist_repayments"],
                    "https://www.irs.gov/publications/p590a",
                    note,
                ))
            },
        );
    }
}
