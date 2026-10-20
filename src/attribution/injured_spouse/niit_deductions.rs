// Attribution of the same final native Form8960 itemized deduction. This
// owner does not calculate an expense, recognize income or rerun a tax cap.
use crate::kernel::attribution::{
    allowed_bounds, allowed_capacities, nonnegative_capacities, unique_allowed,
};
use crate::*;

// Same-call native source cents, heap-backed because Totals participates in
// the large return/fixedpoint frame. The old scalar owners remain unchanged.
#[derive(Default)]
pub(crate) struct NiitDeductionSourceCents {
    pub(crate) estate_ird: [i64; 2],
    pub(crate) k1_interest: [i64; 2],
}

pub(crate) struct NiitItemizedDeductions {
    pub(crate) joint_dollars: i64,
    // Whether the final native allowed amount is emitted/established.
    // Its provisional scalar is not evidence of zero when withheld.
    pub(crate) allowed_available: bool,
    eligible_capacities: Option<[i64; 2]>,
    complement_capacities: Option<[i64; 2]>,
    pub(crate) owners_dollars: Option<[i64; 2]>,
    // Actual current paid capacities for a verified pure ScheduleA portfolio
    // interest pool. A joint limit can leave a choice even with known payers.
    pub(crate) pure_interest_capacities: Option<[i64; 2]>,
    pub(crate) same_itemized_capacities: Option<[i64; 2]>,
    pub(crate) whole_itemized_capacities: Option<[i64; 2]>,
    pub(crate) whole_itemized_dollars: i64,
}

fn portfolio_current_owners(
    draft: &FederalDraft,
    k1_cents: [i64; 2],
    first: usize,
) -> Option<[i64; 2]> {
    let mut current = crate::kernel::attribution::ExpenseSourceCents::known(k1_cents);
    let mut retained = crate::kernel::attribution::ExpenseSourceCents::known([0; 2]);
    if let Some(borrowing) = draft.deductions.as_ref()?.investment_interest.as_ref() {
        for loan in borrowing.loans.iter().filter(|loan| loan.source == 0) {
            current.add(loan.paid, loan.paid_owners);
            retained.add(loan.carry, loan.carry_owners);
        }
    }
    // Form4952 rounds paid and retained carry as separate native categories.
    // Originating entity and destination never stand in for either person.
    let current = current.annual_capacities(first)?;
    let retained = retained.annual_capacities(first)?;
    Some([0, 1].map(|i| current[i] + retained[i]))
}

pub(crate) struct NativeDeductionOperands {
    pub(crate) itemizing: bool,
    // Same-invocation ScheduleA composition, not equality of scalar totals.
    pub(crate) portfolio_only_itemized: bool,
    pub(crate) same_itemized_pool: bool,
    pub(crate) whole_itemized_pool: bool,
    pub(crate) whole_itemized_dollars: i64,
    pub(crate) whole_income_tax_dollars: Option<i64>,
    pub(crate) allowed_dollars: i64,
    pub(crate) allowed_available: bool,
    // Native separate annual Dollar categories: property tax, portfolio
    // interest, connected income tax, foreign investment tax, estate IRD.
    pub(crate) before_dollars: [i64; 5],
    pub(crate) income_tax_available: bool,
    pub(crate) foreign_cents: Option<[i64; 2]>,
    pub(crate) foreign_capacities: Option<crate::kernel::attribution::ExpenseSourceCents>,
    pub(crate) estate_ird_cents: [i64; 2],
    pub(crate) current_k1_interest_cents: [i64; 2],
}

impl NiitItemizedDeductions {
    #[inline(never)]
    pub(crate) fn from_native(draft: &FederalDraft, input: NativeDeductionOperands) -> Self {
        let NativeDeductionOperands {
            itemizing,
            portfolio_only_itemized,
            same_itemized_pool,
            whole_itemized_pool,
            whole_itemized_dollars,
            whole_income_tax_dollars,
            allowed_dollars: joint_dollars,
            allowed_available,
            before_dollars,
            income_tax_available,
            foreign_cents,
            foreign_capacities,
            estate_ird_cents,
            current_k1_interest_cents,
        } = input;
        let first = draft
            .injured_spouse
            .as_ref()
            .map_or(0, |claim| usize::from(claim.injured == Owner::Spouse));
        let current_interest = portfolio_current_owners(draft, current_k1_interest_cents, first);
        let pure_interest_capacities = current_interest.filter(|capacities| {
            allowed_available
                && itemizing
                && portfolio_only_itemized
                && income_tax_available
                && before_dollars == [0, joint_dollars, 0, 0, 0]
                && allowed_bounds(joint_dollars, *capacities).is_some()
        });
        // This is an allowed nonnegative expense, not signed income. The
        // actual native zero target uniquely fixes both owners at zero even
        // when a positive paid source has an unidentified individual payer.
        let project_capacities = |connected_dollars: i64| {
            let deductions = draft.deductions.as_ref()?;
            let property = nonnegative_capacities(
                before_dollars[0],
                deductions.investment_property_tax_owners(),
            )?;
            let interest = if before_dollars[1] == 0 {
                [0; 2]
            } else {
                allowed_capacities(before_dollars[1], current_interest?)?
            };
            let connected =
                nonnegative_capacities(connected_dollars, deductions.income_tax_owners())?;
            let foreign = if before_dollars[3] == 0 {
                [0; 2]
            } else if let Some(capacities) = &foreign_capacities {
                allowed_capacities(before_dollars[3], capacities.annual_capacities(first)?)?
            } else {
                crate::kernel::ratio::project_rounded_pair(
                    before_dollars[3],
                    foreign_cents?,
                    first,
                )?
            };
            let estate = crate::kernel::ratio::project_rounded_pair(
                before_dollars[4],
                estate_ird_cents,
                first,
            )?;
            Some([0, 1].map(|i| property[i] + interest[i] + connected[i] + foreign[i] + estate[i]))
        };
        let before_capacities = (allowed_available && itemizing && income_tax_available)
            .then(|| project_capacities(before_dollars[2]))
            .flatten();
        let whole_itemized_capacities = if itemizing && whole_itemized_pool {
            whole_income_tax_dollars
                .and_then(project_capacities)
                .filter(|capacities| allowed_bounds(whole_itemized_dollars, *capacities).is_some())
        } else {
            None
        };
        // Only this complete native inventory establishes D and its disjoint
        // complement E. A cap/equality of unrelated totals cannot do so.
        let complement_capacities = if allowed_available && whole_itemized_pool {
            whole_income_tax_dollars.and_then(|whole_income| {
                let complement = whole_income - before_dollars[2];
                (complement >= 0
                    && before_dollars.iter().sum::<i64>() == joint_dollars
                    && joint_dollars + complement == whole_itemized_dollars)
                    .then(|| {
                        nonnegative_capacities(
                            complement,
                            draft.deductions.as_ref()?.income_tax_owners(),
                        )
                    })
                    .flatten()
            })
        } else {
            None
        };
        let same_itemized_capacities = before_capacities.filter(|capacities| {
            same_itemized_pool && allowed_bounds(joint_dollars, *capacities).is_some()
        });
        let owners_dollars = if !allowed_available {
            None
        } else if joint_dollars == 0 {
            Some([0; 2])
        } else {
            before_capacities.and_then(|capacities| unique_allowed(joint_dollars, capacities))
        };
        Self {
            joint_dollars,
            allowed_available,
            eligible_capacities: before_capacities,
            complement_capacities,
            owners_dollars,
            pure_interest_capacities,
            same_itemized_capacities,
            whole_itemized_capacities,
            whole_itemized_dollars,
        }
    }

    // Intersect the eligible component D with the accepted whole-pool R5
    // amount Y and actual noneligible complement E. All values are final
    // native annual Dollars; this does not allocate or rerun either tax cap.
    pub(crate) fn subset_bounds(
        &self,
        accepted_itemized: [i64; 2],
        owner: usize,
    ) -> Option<(i64, i64)> {
        if !self.allowed_available
            || accepted_itemized.iter().any(|amount| *amount < 0)
            || accepted_itemized.iter().sum::<i64>() != self.whole_itemized_dollars
        {
            return None;
        }
        crate::kernel::attribution::subset_bounds(
            self.joint_dollars,
            self.whole_itemized_dollars,
            self.eligible_capacities?,
            self.complement_capacities?,
            accepted_itemized,
            owner,
        )
    }

    pub(crate) fn itemized_bounds(&self, target: i64, owner: usize) -> Option<(i64, i64)> {
        if target == self.whole_itemized_dollars
            && let Some(capacities) = self.whole_itemized_capacities
        {
            let (minimum, maximum) = allowed_bounds(target, capacities)?;
            return Some(if owner == 0 {
                (minimum, maximum)
            } else {
                (target - maximum, target - minimum)
            });
        }
        (target == self.joint_dollars).then(|| self.pure_interest_bounds(owner)).flatten()
    }

    pub(crate) fn pure_interest_bounds(&self, owner: usize) -> Option<(i64, i64)> {
        let capacities = self.same_itemized_capacities.or(self.pure_interest_capacities)?;
        let (minimum, maximum) = allowed_bounds(self.joint_dollars, capacities)?;
        Some(if owner == 0 {
            (minimum, maximum)
        } else {
            (self.joint_dollars - maximum, self.joint_dollars - minimum)
        })
    }
}
