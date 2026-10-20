// Runtime orchestration only. Pub974(2025), printed52–54 Step6, and
// RevProc2014-41 section5.01 select Step4 credit with Step5 deduction when
// BOTH changes are strictly below100c. No accepted facts or tax formulas here.
use crate::kernel::ratio::exact::{Exact, whole};
use alloc::{collections::BTreeMap, vec::Vec};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Decision {
    Continue,
    Accept,
    Simplified,
}

#[derive(Default)]
pub(crate) struct Iteration {
    previous: Option<(i64, Exact)>,
    seen: Vec<(i64, BTreeMap<i64, Exact>)>,
}

impl Iteration {
    pub(crate) fn observe(&mut self, credit: i64, next: &BTreeMap<i64, Exact>) -> Decision {
        let deduction: Exact = next.values().sum();
        if let Some((previous_credit, previous_deduction)) = &self.previous {
            let change = &deduction - previous_deduction;
            if (credit - previous_credit).abs() < 100 && change > whole(-100) && change < whole(100)
            {
                return Decision::Accept;
            }
        }
        // A finite computation bound is not a tax acceptance rule. Failed
        // convergence starts the complete Simplified method from its Step1;
        // no last iterate is selected. Track source vectors, not list order.
        if self.seen.len() >= 128
            || self.seen.iter().any(|(c, amounts)| *c == credit && amounts == next)
        {
            return Decision::Simplified;
        }
        self.previous = Some((credit, deduction));
        self.seen.push((credit, next.clone()));
        Decision::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amount(value: i64) -> BTreeMap<i64, Exact> {
        [(19, whole(value))].into_iter().collect()
    }

    #[test]
    fn iterative_accepts_only_both_strict_cent_changes() {
        for (deduction_change, credit_change, expected) in [
            (99, 99, Decision::Accept),
            (99, 100, Decision::Continue),
            (100, 99, Decision::Continue),
            (100, 0, Decision::Continue),
            (-100, 0, Decision::Continue),
            (-99, -99, Decision::Accept),
        ] {
            let mut run = Iteration::default();
            assert_eq!(run.observe(150_000, &amount(100_049)), Decision::Continue);
            assert_eq!(
                run.observe(150_000 + credit_change, &amount(100_049 + deduction_change)),
                expected
            );
        }
        let mut run = Iteration::default();
        run.observe(150_000, &amount(100_049));
        // Rounding the two deductions to dollars first would see1000→1001,
        // although the actual changes1c/99c satisfy the source stopping rule.
        assert_eq!(run.observe(150_099, &amount(100_050)), Decision::Accept);
    }

    #[test]
    fn iterative_cycle_and_budget_require_a_fresh_simplified_pair() {
        let mut cycle = Iteration::default();
        assert_eq!(cycle.observe(100_000, &amount(200_000)), Decision::Continue);
        assert_eq!(cycle.observe(110_000, &amount(190_000)), Decision::Continue);
        assert_eq!(cycle.observe(100_000, &amount(200_000)), Decision::Simplified);
        let mut increasing = Iteration::default();
        for index in 0..128 {
            assert_eq!(increasing.observe(index * 100, &amount(index * 100)), Decision::Continue);
        }
        assert_eq!(increasing.observe(12_800, &amount(12_800)), Decision::Simplified);
    }
}
