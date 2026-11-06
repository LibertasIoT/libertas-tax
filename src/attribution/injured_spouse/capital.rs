// Transient source amounts from the existing capital calculation. This is
// attribution, not another Schedule D or a separately filed return.
// Final-estate carryovers retain their actual beneficiary and native annual
// short/long rounding. Their independent documented Form8960 excluded-loss
// difference retains each fiduciary statement's existing Dollar boundary.
use crate::kernel::ratio::{
    dollars,
    exact::{self, Exact},
    fraction,
};
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

#[derive(Clone, Default)]
pub(crate) struct CapitalSources {
    // Source cents, [first-rounded owner][owner][short/long].
    cents: [[[Exact; 2]; 2]; 2],
    // Admitted collectible/section1202 and unrecaptured source cents.
    special: [[Exact; 2]; 2],
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
            *source += exact::whole(amount);
        }
    }

    // Accepted ownership weights retain fractional cents until the native
    // annual short/long boundaries select the first rounded owner.
    pub(crate) fn add_weighted(&mut self, weights: Option<([i64; 2], i64)>, amounts: [i64; 2]) {
        let Some((weights, denominator)) = weights else {
            if amounts != [0; 2] {
                self.invalidate();
            }
            return;
        };
        for (owner, weight) in weights.into_iter().enumerate() {
            let Some(weight) = exact::ratio(weight, denominator) else {
                self.invalidate();
                return;
            };
            for first in 0..2 {
                for (source, amount) in self.cents[first][owner].iter_mut().zip(amounts) {
                    *source += exact::whole(amount) * &weight;
                }
            }
        }
    }

    pub(crate) fn add_special(&mut self, owner: Option<Owner>, amounts: [i64; 2]) {
        let weights =
            owner.map(|owner| (if owner == Owner::Taxpayer { [1, 0] } else { [0, 1] }, 1));
        self.add_special_weighted(weights, amounts);
    }

    pub(crate) fn add_special_weighted(
        &mut self,
        weights: Option<([i64; 2], i64)>,
        amounts: [i64; 2],
    ) {
        let Some((weights, denominator)) = weights else {
            if amounts != [0; 2] {
                self.invalidate();
            }
            return;
        };
        for (owner, weight) in weights.into_iter().enumerate() {
            let Some(weight) = exact::ratio(weight, denominator) else {
                self.invalidate();
                return;
            };
            for (source, amount) in self.special[owner].iter_mut().zip(amounts) {
                *source += exact::whole(amount) * &weight;
            }
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
                    i128::from(absorbed[i])
                        > prior[i].iter().map(|value| i128::from(*value)).sum::<i128>()
                            + (-i128::from(form.source_net[i])).max(0) * 100
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
                    let Some(amount) = owners[i].checked_add(carry[i]) else {
                        self.invalidate();
                        return;
                    };
                    owners[i] = amount;
                }
            }
        }
        if views.iter().any(|owners| {
            owners.iter().map(|value| i128::from(*value)).sum::<i128>() != i128::from(form.total)
        }) {
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
                    [i128::from(short) * 100, i128::from(long) * 100]
                } else {
                    [
                        (i128::from(form.short) - i128::from(short)) * 100,
                        (i128::from(form.long) - i128::from(long)) * 100,
                    ]
                };
                let [Ok(short), Ok(long)] = parts.map(i64::try_from) else {
                    self.invalidate();
                    return;
                };
                let parts = [short, long];
                let owner = if i == 0 { Owner::Taxpayer } else { Owner::Spouse };
                self.add_for_first(first, owner, parts);
                investment.add_for_first(first, owner, [parts[0], parts[1], 0, 0, 0]);
            }
        }
    }

    // Form 8960's section 6013(g)/(h) default method uses one resident's
    // actual sources and retained losses, then the existing MFS loss floor.
    // The caller supplies only a changed per-person fund exclusion, in exact
    // cents, before this owner's annual short/long rounding boundary.
    pub(crate) fn separate_income(
        &self,
        d: &FederalDraft,
        owner: usize,
        long_adjustment: Exact,
    ) -> Option<crate::capital::CapitalResult> {
        if self.unavailable {
            return None;
        }
        let prior = native_prior_owners(d, owner)?;
        let short =
            exact::dollars(&(&self.cents[owner][owner][0] - exact::whole(prior[owner][0])))?;
        let long = exact::dollars(
            &(&self.cents[owner][owner][1] - exact::whole(prior[owner][1]) + long_adjustment),
        )?;
        Some(
            crate::capital::CapitalAmounts { short, long, ..Default::default() }
                .calculate([0; 2], true, false),
        )
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
        let cents: [[Exact; 2]; 2] = core::array::from_fn(|owner| {
            core::array::from_fn(|term| {
                &self.cents[first][owner][term] - exact::whole(prior[owner][term])
            })
        });
        prior[0][1].checked_add(prior[1][1])?;
        let mut columns = [[0; 2]; 4];
        for (column, (joint, sources)) in columns.iter_mut().zip([
            (net[0], [cents[0][0].clone(), cents[1][0].clone()]),
            (net[1], [cents[0][1].clone(), cents[1][1].clone()]),
            (special[0], [self.special[0][0].clone(), self.special[1][0].clone()]),
            (special[1], [self.special[0][1].clone(), self.special[1][1].clone()]),
        ]) {
            let total = &sources[0] + &sources[1];
            *column = exact::project_rounded_pair(joint, &total, sources, first)?;
        }
        let [short, long, rate28, rate25] = columns;
        let net = core::array::from_fn(|i| short[i] + long[i]);
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
            let data = &crate::FederalEstateTrustPortfolioK1::from(data);
            let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
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
pub(crate) fn niit_difference_owners(d: &FederalDraft) -> Option<[i64; 2]> {
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
            let data = &crate::FederalEstateTrustPortfolioK1::from(data);
            let carry = data.final_short_carry.checked_add(data.final_long_carry)?;
            let difference = data
                .final_character
                .as_ref()
                .and_then(|finals| finals.capital.niit_difference)
                .or((carry == 0).then_some(0))?;
            let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
            amounts[owner] = amounts[owner].checked_add(dollars(difference))?;
        }
    }
    Some(amounts)
}
