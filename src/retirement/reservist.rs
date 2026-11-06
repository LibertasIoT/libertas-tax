use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::ratio::exact;
use crate::line_emission::emit as emit_lines;
pub use crate::reservist_sources::{ReservistPayment, ReservistRepayment};
use crate::{FederalResult, ReserveTerm};
use alloc::format;

impl ReservistRepayment {
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
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
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
