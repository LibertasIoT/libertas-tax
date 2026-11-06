// Actual Form4952 collector inputs, separate from the ScheduleD source pool.
use super::*;
use crate::kernel::ratio::exact::{self, Exact};

pub(crate) struct InvestmentElectionSources {
    pub elected: [i64; 2], // actual joint capital/qualified election dollars
    pub capital: Option<[[i64; 2]; 2]>, // [first-rounded owner][owner] capacities, dollars; not a partition
}

#[derive(Clone, Default)]
pub(crate) struct InvestmentSources {
    // Source cents [first-rounded owner][owner]: short, long, shareholder loan, investment K1 ordinary,
    // and the actual section1244 ordinary loss. These native components
    // round independently in the existing joint calculation.
    cents: [[[Exact; 5]; 2]; 2],
    unavailable: bool,
}

impl InvestmentSources {
    pub(crate) fn add(&mut self, owner: Option<Owner>, amounts: [i64; 5]) {
        if let Some(owner) = owner {
            for first in 0..2 {
                self.add_for_first(first, owner, amounts);
            }
        } else if amounts != [0; 5] {
            self.unavailable = true;
        }
    }
    pub(crate) fn add_for_first(&mut self, first: usize, owner: Owner, amounts: [i64; 5]) {
        for (value, amount) in self.cents[first][owner_index(owner)].iter_mut().zip(amounts) {
            *value += exact::whole(amount);
        }
    }

    pub(crate) fn add_weighted(&mut self, weights: Option<([i64; 2], i64)>, amounts: [i64; 5]) {
        let Some((weights, denominator)) = weights else {
            if amounts != [0; 5] {
                self.unavailable = true;
            }
            return;
        };
        for (owner, weight) in weights.into_iter().enumerate() {
            let Some(weight) = exact::ratio(weight, denominator) else {
                self.unavailable = true;
                return;
            };
            for first in 0..2 {
                for (value, amount) in self.cents[first][owner].iter_mut().zip(amounts) {
                    *value += exact::whole(amount) * &weight;
                }
            }
        }
    }

    pub(crate) fn owners(
        &self,
        d: &FederalDraft,
        joint: [i64; 5],
        carry: [i64; 2],
        net_capital: i64,
    ) -> Option<[[i64; 2]; 2]> {
        if self.unavailable {
            return None;
        }
        let prior = prior_owners(d)?;
        Some([
            self.capacities(joint, carry, prior, net_capital, 0)?,
            self.capacities(joint, carry, prior, net_capital, 1)?,
        ])
    }
    fn capacities(
        &self,
        joint: [i64; 5],
        carry: [i64; 2],
        prior: [[i64; 2]; 2],
        net_capital: i64,
        first: usize,
    ) -> Option<[i64; 2]> {
        use crate::kernel::ratio::project_rounded_pair;
        // Reconcile all five independently rounded native source components.
        let mut sources = [[0; 2]; 5];
        for (k, amount) in joint.into_iter().enumerate() {
            let cents = [self.cents[first][0][k].clone(), self.cents[first][1][k].clone()];
            let total = &cents[0] + &cents[1];
            sources[k] = exact::project_rounded_pair(amount, &total, cents, first)?;
        }
        for (term, amount) in carry.into_iter().enumerate() {
            project_rounded_pair(amount, [prior[0][term], prior[1][term]], first)?;
        }
        let source = sources[1];
        let prior = project_rounded_pair(carry[1], [prior[0][1], prior[1][1]], first)?;
        let long: [i64; 2] = core::array::from_fn(|i| source[i] - prior[i]);
        // Capacities may overlap; the native joint pool and E-other..own
        // bounds conserve the single election without two source loss limits.
        if net_capital < 0 || net_capital > positive(long.iter().sum()) {
            return None;
        }
        let capacity: [i64; 2] =
            core::array::from_fn(|i| phase::cap(positive(long[i]), [net_capital]));
        (capacity.iter().sum::<i64>() >= net_capital).then_some(capacity)
    }
}

fn prior_owners(d: &FederalDraft) -> Option<[[i64; 2]; 2]> {
    let mut owners = [[0; 2]; 2];
    for prior in &d.screening.as_ref()?.capital_history {
        let regular = crate::extended_rules::regular_capital_carry(prior).map(|v| v * 100);
        let investment = match crate::extended_rules::investment_capital_parts(prior) {
            Some(parts) => parts[0].map(|v| v * 100),
            None if regular == [0; 2] => [0; 2],
            None => return None,
        };
        let parts = match prior.owners {
            crate::PriorReturnOwners::Taxpayer => [investment, [0; 2]],
            crate::PriorReturnOwners::Spouse => [[0; 2], investment],
            crate::PriorReturnOwners::Joint if investment == [0; 2] => [[0; 2]; 2],
            crate::PriorReturnOwners::Joint => {
                let parts = prior.injured_owners.as_ref()?;
                if !parts.valid(regular) {
                    return None;
                }
                let regular_parts = parts.amounts();
                let mut result = [[0; 2]; 2];
                for term in 0..2 {
                    if investment[term] == regular[term] {
                        for i in 0..2 {
                            result[i][term] = regular_parts[i][term];
                        }
                    } else if investment[term] != 0 {
                        let owner = (0..2).find(|&i| {
                            regular_parts[i][term] > 0 && regular_parts[1 - i][term] == 0
                        })?;
                        result[owner][term] = investment[term];
                    }
                }
                result
            }
        };
        for i in 0..2 {
            for term in 0..2 {
                owners[i][term] += parts[i][term];
            }
        }
    }
    Some(owners)
}
