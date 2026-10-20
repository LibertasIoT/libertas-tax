// Transient source amounts from the existing capital calculation. This is
// attribution, not another Schedule D or a separately filed return.
// Final-estate carryovers retain their actual beneficiary and native annual
// short/long rounding. Their independent documented Form8960 excluded-loss
// difference retains each fiduciary statement's existing Dollar boundary.
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::{FederalDraft, Owner, PriorReturnOwners};

#[derive(Clone, Copy)]
pub(crate) struct CapitalOwners {
    // [first-rounded owner][owner]. The consumer chooses its actual claim
    // context; an absent claim uses taxpayer-first without fabricating one.
    pub by_first: [CapitalColumns; 2],
}
#[derive(Clone, Copy)]
pub(crate) struct CapitalColumns {
    pub net: [i64; 2],
    pub long: [i64; 2],
    pub rates: [[i64; 2]; 2],
}
impl CapitalOwners {
    #[cfg(test)]
    pub(crate) fn whole(net: [i64; 2], long: [i64; 2], rates: [[i64; 2]; 2]) -> Self {
        Self { by_first: [CapitalColumns { net, long, rates }; 2] }
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct CapitalSources {
    // Source cents, [first-rounded owner][owner][short/long].
    cents: [[[i64; 2]; 2]; 2],
    // Admitted collectible/section1202 and unrecaptured source cents.
    special: [[i64; 2]; 2],
    unavailable: bool,
}

impl CapitalSources {
    pub(crate) fn add(&mut self, owner: Option<Owner>, amounts: [i64; 2]) {
        if let Some(owner) = owner {
            for first in 0..2 {
                self.add_for_first(first, owner, amounts);
            }
        } else if amounts != [0; 2] {
            self.invalidate();
        }
    }
    fn add_for_first(&mut self, first: usize, owner: Owner, amounts: [i64; 2]) {
        let owner = usize::from(owner == Owner::Spouse);
        for (source, amount) in self.cents[first][owner].iter_mut().zip(amounts) {
            *source += amount;
        }
    }

    pub(crate) fn add_special(&mut self, owner: Option<Owner>, amounts: [i64; 2]) {
        if let Some(owner) = owner {
            let owner = usize::from(owner == Owner::Spouse);
            for (source, amount) in self.special[owner].iter_mut().zip(amounts) {
                *source += amount;
            }
        } else if amounts != [0; 2] {
            self.invalidate();
        }
    }

    pub(crate) fn invalidate(&mut self) {
        self.unavailable = true;
    }

    pub(crate) fn contracts(
        &mut self,
        d: &FederalDraft,
        form: &crate::section1256::Form6781,
        investment: &mut super::InvestmentSources,
    ) {
        let mut views = [form.source_net; 2];
        if form.carryback != 0 {
            let Some(screening) = &d.screening else {
                self.invalidate();
                return;
            };
            let Some(source) = &screening.section_1256_carryback_owners else {
                self.invalidate();
                return;
            };
            let total = screening.section_1256_carryback.unwrap_or(0);
            let Some(prior) = prior_owners(d) else {
                self.invalidate();
                return;
            };
            let absorbed = source.amounts(total);
            if !source.valid(total)
                || (0..2).any(|i| {
                    absorbed[i] > prior[i].iter().sum::<i64>() + positive(-form.source_net[i]) * 100
                })
            {
                self.invalidate();
                return;
            }
            for (first, owners) in views.iter_mut().enumerate() {
                let Some(carry) =
                    crate::kernel::ratio::project_rounded_pair(form.carryback, absorbed, first)
                else {
                    self.invalidate();
                    return;
                };
                for i in 0..2 {
                    owners[i] += carry[i];
                }
            }
        }
        if views.iter().any(|owners| owners.iter().sum::<i64>() != form.total) {
            self.invalidate();
            return;
        }
        // Retain both possible column-rounding contexts for the same native
        // 40/60 source character. No MFS carryback or loss-limit rerun.
        for (first, owners) in views.into_iter().enumerate() {
            let short = fraction(owners[first], 40, 100);
            let long = owners[first] - short;
            for i in 0..2 {
                let parts = if i == first {
                    [short * 100, long * 100]
                } else {
                    [(form.short - short) * 100, (form.long - long) * 100]
                };
                let owner = if i == 0 { Owner::Taxpayer } else { Owner::Spouse };
                self.add_for_first(first, owner, parts);
                investment.add_for_first(first, owner, [parts[0], parts[1], 0, 0, 0]);
            }
        }
    }

    // `net` is the existing joint ScheduleD short/long result before its
    // annual loss floor. Never run two MFS carryover or loss-limit engines.
    pub(crate) fn owners(
        &self,
        d: &FederalDraft,
        net: [i64; 2],
        special: [i64; 2],
    ) -> Option<CapitalOwners> {
        if self.unavailable {
            return None;
        }
        Some(CapitalOwners {
            by_first: [
                self.columns(net, special, native_prior_owners(d, 0)?, 0)?,
                self.columns(net, special, native_prior_owners(d, 1)?, 1)?,
            ],
        })
    }
    fn columns(
        &self,
        net: [i64; 2],
        special: [i64; 2],
        prior: [[i64; 2]; 2],
        first: usize,
    ) -> Option<CapitalColumns> {
        use crate::kernel::ratio::project_rounded_pair;
        let cents: [[i64; 2]; 2] = core::array::from_fn(|i| {
            core::array::from_fn(|term| self.cents[first][i][term] - prior[i][term])
        });
        let short = project_rounded_pair(net[0], [cents[0][0], cents[1][0]], first)?;
        let long = project_rounded_pair(net[1], [cents[0][1], cents[1][1]], first)?;
        let net = core::array::from_fn(|i| short[i] + long[i]);
        let rate28 =
            project_rounded_pair(special[0], [self.special[0][0], self.special[1][0]], first)?;
        let rate25 =
            project_rounded_pair(special[1], [self.special[0][1], self.special[1][1]], first)?;
        let prior_long = project_rounded_pair(
            dollars(prior[0][1] + prior[1][1]),
            [prior[0][1], prior[1][1]],
            first,
        )?;
        let rates = core::array::from_fn(|i| {
            let rates = crate::capital::CapitalAmounts {
                short: short[i],
                long: long[i],
                collectibles: rate28[i],
                unrecaptured: rate25[i],
            }
            .rate_pools(prior_long[i]);
            [rates.collectible_gain, rates.unrecaptured_gain]
        });
        Some(CapitalColumns { net, long, rates })
    }
}

fn prior_owners(d: &FederalDraft) -> Option<[[i64; 2]; 2]> {
    let mut owners = [[0; 2]; 2];
    for prior in &d.screening.as_ref()?.capital_history {
        let carry = crate::extended_rules::regular_capital_carry(prior).map(|v| v * 100);
        let parts = match prior.owners {
            PriorReturnOwners::Taxpayer => [carry, [0; 2]],
            PriorReturnOwners::Spouse => [[0; 2], carry],
            PriorReturnOwners::Joint if carry == [0; 2] => [[0; 2]; 2],
            PriorReturnOwners::Joint => {
                let parts = prior.injured_owners.as_ref()?;
                if !parts.valid(carry) {
                    return None;
                }
                parts.amounts()
            }
        };
        for owner in 0..2 {
            for term in 0..2 {
                owners[owner][term] += parts[owner][term];
            }
        }
    }
    Some(owners)
}

fn native_prior_owners(d: &FederalDraft, first: usize) -> Option<[[i64; 2]; 2]> {
    let mut owners = prior_owners(d)?;
    // The native ScheduleD owner rounds all succeeded estate carries once
    // per short/long category, independently of filed-return carryovers.
    let mut estate = [[0_i64; 2]; 2];
    for entry in d
        .income
        .iter()
        .filter(|entry| matches!(entry.income, crate::FederalIncome::EstateTrustPortfolio { .. }))
        .map(|entry| crate::community::entry(d, entry))
    {
        if let crate::FederalIncome::EstateTrustPortfolio { data } = &entry.income {
            let owner = usize::from(entry.owner == Owner::Spouse);
            for (term, amount) in
                [data.final_short_carry, data.final_long_carry].into_iter().enumerate()
            {
                estate[owner][term] = estate[owner][term].checked_add(amount)?;
            }
        }
    }
    for term in 0..2 {
        let cents = [estate[0][term], estate[1][term]];
        let joint = dollars(cents[0].checked_add(cents[1])?);
        let parts = crate::kernel::ratio::project_rounded_pair(joint, cents, first)?;
        for i in 0..2 {
            owners[i][term] = owners[i][term].checked_add(parts[i].checked_mul(100)?)?;
        }
    }
    Some(owners)
}

// Independent Form8960 source axis, in the same per-record whole dollars
// the joint owner uses. Regular capital-loss shares never identify it.
pub(super) fn niit_difference_owners(d: &FederalDraft) -> Option<[i64; 2]> {
    let mut amounts = [0; 2];
    for prior in &d.screening.as_ref()?.capital_history {
        let difference = match prior.niit_difference {
            Some(value) => value,
            None if crate::extended_rules::regular_capital_carry(prior) == [0; 2] => 0,
            None => return None,
        };
        let total = dollars(difference);
        let parts = if let Some(source) = &prior.injured_niit_owners {
            if !crate::injured_spouse_sources::niit_owners_valid(prior) {
                return None;
            }
            crate::kernel::ratio::project_native_pair(total, source.amounts(difference), 0)?
        } else {
            match prior.owners {
                PriorReturnOwners::Taxpayer => [total, 0],
                PriorReturnOwners::Spouse => [0, total],
                PriorReturnOwners::Joint if total == 0 => [0; 2],
                PriorReturnOwners::Joint => return None,
            }
        };
        for i in 0..2 {
            amounts[i] += parts[i];
        }
    }
    // Reg.1.642(h)-1 succeeded excluded-loss differences enter the native
    // net-gains worksheet at each fiduciary statement's Dollar boundary.
    // The beneficiary is the source owner; ordinary capital carries do not
    // establish this independent character or an unavailable statement.
    for entry in d
        .income
        .iter()
        .filter(|entry| matches!(entry.income, crate::FederalIncome::EstateTrustPortfolio { .. }))
        .map(|entry| crate::community::entry(d, entry))
    {
        if let crate::FederalIncome::EstateTrustPortfolio { data } = &entry.income {
            let carry = data.final_short_carry.checked_add(data.final_long_carry)?;
            let difference = data
                .final_character
                .as_ref()
                .and_then(|finals| finals.capital.niit_difference)
                .or((carry == 0).then_some(0))?;
            let owner = usize::from(entry.owner == Owner::Spouse);
            amounts[owner] = amounts[owner].checked_add(dollars(difference))?;
        }
    }
    Some(amounts)
}
