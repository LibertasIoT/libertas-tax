// Written-note Form8949 rows consume the original debt-basis and holding
// owners. The amounts are the same settled source cents as loan_gains;
// neither open-account ordinary gains nor another shareholder's debt enters.
use super::{ScorpK1, ScorpLoanKind};
use crate::scorp_basis::BasisResult;
use alloc::vec::Vec;

pub(crate) struct RepaymentRow {
    pub note: usize,
    pub long: bool,
    pub proceeds: i64,
    pub basis: i64,
}
impl ScorpK1 {
    pub(crate) fn loan_rows(
        &self,
        basis: &BasisResult,
    ) -> Result<Vec<RepaymentRow>, (&'static str, &'static str)> {
        // Reuse the financial owner's validation and actual holding-period
        // allocation; do not introduce another debt-basis calculation.
        let gains = self.loan_gains(basis)?;
        let mut rows = Vec::new();
        let mut projected = [0; 2];
        for (index, (source, computed)) in self.loans.iter().zip(&basis.loans).enumerate() {
            if source.kind != ScorpLoanKind::FormalNote || computed.capital_gain.get() == 0 {
                continue;
            }
            let gain = source.repayment_term.split(source, computed.capital_gain.get())?;
            let proceeds = source
                .repayment_term
                .split(source, computed.capital_gain.get() + computed.nontaxable_repayment.get())?;
            for term in 0..2 {
                if proceeds[term] == 0 {
                    continue;
                }
                projected[term] += gain[term];
                rows.push(RepaymentRow {
                    note: index + 1,
                    long: term == 1,
                    proceeds: proceeds[term],
                    basis: proceeds[term] - gain[term],
                });
            }
        }
        debug_assert_eq!(projected, [gains[1], gains[2]]);
        Ok(rows)
    }
}

impl ScorpK1 {
    pub(crate) fn loan_source_totals(&self) -> Option<[i64; 3]> {
        let opening =
            self.loans.iter().try_fold(0i64, |sum, loan| sum.checked_add(loan.principal))?;
        let repayments =
            self.loans.iter().try_fold(0i64, |sum, loan| sum.checked_add(loan.repayments))?;
        let result = self.basis_result().ok()?;
        let ending = result
            .loans
            .iter()
            .try_fold(0i64, |sum, loan| sum.checked_add(loan.principal.get()))?;
        Some([opening, ending, repayments])
    }
}
