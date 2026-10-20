// Attribution of the actual account owners' final components. Native account
// calculations own eligibility, expense coordination, rates and rounding.
use super::*;
use crate::kernel::ratio::exact::{self, Exact};

// Exact owner arrays stay off the nested Save/estimate stack.
pub(crate) type AccountSources = alloc::boxed::Box<AccountSourceState>;

#[derive(Clone, Default)]
pub(crate) struct AccountSourceState {
    // Actual registered accounts leave the generic Rest ledger before summing.
    // Source IDs remain tied to their owner; corrections invent no income ID.
    sources: alloc::collections::BTreeMap<i64, (Owner, Exact)>,
    income: [[Exact; 2]; 2], // [first-rounded owner][owner], Dollars
    income_joint: Exact,
    tax: [[Exact; 2]; 2],
    deduction: [[Exact; 2]; 2],
    income_unavailable: bool,
    tax_unavailable: bool,
    deduction_unavailable: bool,
}

impl AccountSourceState {
    pub(crate) fn source(&mut self, id: i64, owner: Owner, cents: Exact) {
        if let Some((previous_owner, previous)) = self.sources.get_mut(&id) {
            if *previous_owner != owner {
                self.income_unavailable = true;
            }
            *previous += cents;
        } else {
            self.sources.insert(id, (owner, cents));
        }
    }
    pub(super) fn contains(&self, id: i64) -> bool {
        self.sources.contains_key(&id)
    }
    pub(crate) fn income(&mut self, joint: i64, cents: [Exact; 2]) {
        self.income_joint += exact::whole(joint);
        let total = cents.iter().sum::<Exact>();
        for first in 0..2 {
            if let Some(parts) = exact::project_rounded_pair(joint, &total, cents.clone(), first) {
                for (sum, value) in self.income[first].iter_mut().zip(parts) {
                    *sum += exact::whole(value);
                }
            } else {
                self.income_unavailable = true;
            }
        }
    }
    pub(crate) fn fixed_income(&mut self, dollars: [i64; 2]) {
        self.income_joint += dollars.iter().map(|v| exact::whole(*v)).sum::<Exact>();
        for view in &mut self.income {
            for (total, value) in view.iter_mut().zip(dollars) {
                *total += exact::whole(value);
            }
        }
    }
    pub(crate) fn fixed_tax(&mut self, dollars: [i64; 2]) {
        for view in &mut self.tax {
            for (total, value) in view.iter_mut().zip(dollars) {
                *total += exact::whole(value);
            }
        }
    }
    pub(crate) fn msa_deduction(&mut self, joint: i64, cents: [Exact; 2]) {
        let total = cents.iter().sum::<Exact>();
        for first in 0..2 {
            if let Some(parts) = exact::project_rounded_pair(joint, &total, cents.clone(), first) {
                self.deduction[first] = parts.map(exact::whole);
            } else {
                self.deduction_unavailable = true;
            }
        }
    }
    pub(super) fn msa_deduction_owners(&self, first: usize) -> Option<[i64; 2]> {
        if self.deduction_unavailable {
            return None;
        }
        Some([
            exact::rounded(&self.deduction[first][0])?,
            exact::rounded(&self.deduction[first][1])?,
        ])
    }
    pub(crate) fn archer_tax(&mut self, joint: i64, penalized: [Exact; 2]) {
        // Native Archer tax is20% of its jointly rounded penalty base.
        // The existing1/500 dollar projection retains exact penalized Cents.
        let cents = penalized.map(|v| v * exact::ratio(1, 5).expect("fixed tax rate"));
        let total = cents.iter().sum::<Exact>();
        for first in 0..2 {
            if let Some(parts) = exact::project_rounded_pair(joint, &total, cents.clone(), first) {
                for (sum, value) in self.tax[first].iter_mut().zip(parts) {
                    *sum += exact::whole(value);
                }
            } else {
                self.tax_unavailable = true;
            }
        }
    }
    pub(super) fn income_owners(&self, first: usize) -> Option<[i64; 2]> {
        if self.income_unavailable {
            return None;
        }
        Some([exact::rounded(&self.income[first][0])?, exact::rounded(&self.income[first][1])?])
    }
    pub(super) fn income_joint(&self) -> Option<i64> {
        exact::rounded(&self.income_joint)
    }
    pub(super) fn tax_owners(&self, first: usize) -> Option<[i64; 2]> {
        if self.tax_unavailable {
            return None;
        }
        Some([exact::rounded(&self.tax[first][0])?, exact::rounded(&self.tax[first][1])?])
    }
}
