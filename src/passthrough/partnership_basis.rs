// Partner outside basis, before at-risk, passive, and return-level limits.
// IRS Partner's Instructions for Schedule K-1 (Form 1065), basis worksheet.
// https://www.irs.gov/instructions/i1065sk1
// Actual source cents. This is not Form 7203 stock/direct-shareholder-debt basis.
use crate::k1_loss::LossAmounts;
use crate::kernel::carry::{Amount, Cents};
use crate::kernel::phase;
use alloc::vec::Vec;

pub(crate) struct PartnerBasis<'a> {
    pub opening: Amount<Cents>,
    // Tax basis of acquisitions/contributions, after attached liabilities.
    // Not FMV, book capital, or compensation paid as guaranteed payments.
    pub contributions: Amount<Cents>,
    pub opening_liabilities: Amount<Cents>,
    pub ending_liabilities: Amount<Cents>,
    pub partnership_liabilities_assumed: Amount<Cents>,
    pub personal_liabilities_transferred: Amount<Cents>,
    // Positive basis-increasing items only, including tax-exempt income.
    // Never net losses here; guaranteed payments do not enter this subtotal.
    pub income: Amount<Cents>,
    pub cash_distributions: Amount<Cents>,
    // Ordinary nonliquidating property only; caller resolves sections 731(c),
    // 704(c), 737, 751, transaction-date ordering and special distributions.
    pub distributed_property_basis: Amount<Cents>,
    // Current plus retained worksheet carry, kept separate from loss buckets.
    pub nondeductible: Amount<Cents>,
    // The section 50(c)(5) reduction for an investment credit, not below zero
    // and never carried (Form 7203 line 8c).
    pub credit_reduction: Amount<Cents>,
    pub depletion: Amount<Cents>,
    // Combined current/prior basis-limited amounts by tax-character category.
    // Charitable FMV excess not subject to 704(d) stays outside these amounts.
    pub losses: &'a LossAmounts,
    // Together form ONE loss class. Basis allocated to that class first
    // absorbs currently deductible BIE, then EBIE (not currently deductible).
    pub business_interest: Amount<Cents>,
    pub excess_business_interest: Amount<Cents>,
    // Money a liquidation of the whole interest relieves: the item K1 share
    // of liabilities immediately before it, a deemed distribution (section
    // 752(b)) in addition to the change in allocated liabilities.
    pub deemed_money: Amount<Cents>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PartnerBasisResult {
    pub ending: Amount<Cents>,
    pub distribution_gain: Amount<Cents>,
    pub distributed_property_basis: Amount<Cents>,
    pub nondeductible_used: Amount<Cents>,
    pub nondeductible_carry: Amount<Cents>,
    pub depletion_used: Amount<Cents>,
    pub depletion_carry: Amount<Cents>,
    pub allowed: LossAmounts,
    pub suspended: LossAmounts,
    pub interest_allowed: Amount<Cents>,
    pub interest_basis_suspended: Amount<Cents>,
    pub excess_interest_basis_used: Amount<Cents>,
    pub excess_interest_basis_suspended: Amount<Cents>,
}

fn sum(values: impl IntoIterator<Item = i64>) -> Option<i64> {
    values.into_iter().try_fold(0i64, |a, b| {
        (b >= 0).then_some(())?;
        a.checked_add(b)
    })
}

impl PartnerBasis<'_> {
    pub(crate) fn calculate(&self) -> Option<PartnerBasisResult> {
        // Validate source components before differences can conceal a negative.
        for amount in [
            self.opening.get(),
            self.contributions.get(),
            self.opening_liabilities.get(),
            self.ending_liabilities.get(),
            self.partnership_liabilities_assumed.get(),
            self.personal_liabilities_transferred.get(),
            self.income.get(),
            self.cash_distributions.get(),
            self.distributed_property_basis.get(),
            self.nondeductible.get(),
            self.credit_reduction.get(),
            self.depletion.get(),
            self.business_interest.get(),
            self.excess_business_interest.get(),
            self.deemed_money.get(),
        ] {
            if amount < 0 {
                return None;
            }
        }
        let liabilities = self
            .ending_liabilities
            .get()
            .checked_sub(self.opening_liabilities.get())?
            .checked_add(self.partnership_liabilities_assumed.get())?;
        let before_distributions = sum([
            self.opening.get(),
            self.contributions.get(),
            self.income.get(),
            phase::floor(liabilities, 0),
        ])?;
        let money = sum([
            self.cash_distributions.get(),
            phase::floor(-liabilities, 0),
            self.personal_liabilities_transferred.get(),
            self.deemed_money.get(),
        ])?;
        let distribution_gain = phase::floor(money - before_distributions, 0);
        let after_money = phase::floor(before_distributions - money, 0);
        let property_basis = phase::cap(self.distributed_property_basis.get(), [after_money]);
        let after_distributions = after_money - property_basis;
        let after_distributions =
            after_distributions - phase::cap(self.credit_reduction.get(), [after_distributions]);
        let expenses = [self.nondeductible.get(), self.depletion.get()];
        let expense_total = sum(expenses)?;
        let expense_used = phase::cap(expense_total, [after_distributions]);
        let used = crate::kernel::ratio::allocate_remaining(expense_used, &expenses);
        let available = after_distributions - expense_used;
        let interest = sum([self.business_interest.get(), self.excess_business_interest.get()])?;
        let mut classes = self.losses.as_slice().to_vec();
        classes.push(Amount::new(interest).expect("nonnegative interest"));
        let total = sum(classes.iter().map(Amount::get))?;
        let deductible = phase::cap(available, [total]);
        let weights: Vec<_> = classes.iter().map(Amount::get).collect();
        let mut allowed = crate::kernel::ratio::allocate_remaining(deductible, &weights);
        let interest_basis = allowed.pop()?;
        let interest_allowed = phase::cap(interest_basis, [self.business_interest.get()]);
        let excess_basis = interest_basis - interest_allowed;
        let allowed: Vec<_> = allowed
            .into_iter()
            .map(|v| Amount::new(v).expect("nonnegative allocated class"))
            .collect();
        let allowed = LossAmounts::from_slice(&allowed);
        let suspended = self.losses.minus(&allowed);
        use crate::kernel::carry::closing_values;
        Some(PartnerBasisResult {
            ending: closing_values::<Cents>(available, deductible).ok()?,
            distribution_gain: Amount::<Cents>::new(distribution_gain)
                .expect("bounded basis result"),
            distributed_property_basis: Amount::<Cents>::new(property_basis)
                .expect("bounded basis result"),
            nondeductible_used: Amount::<Cents>::new(used[0]).expect("bounded basis result"),
            nondeductible_carry: closing_values::<Cents>(self.nondeductible.get(), used[0]).ok()?,
            depletion_used: Amount::<Cents>::new(used[1]).expect("bounded basis result"),
            depletion_carry: closing_values::<Cents>(self.depletion.get(), used[1]).ok()?,
            allowed,
            suspended,
            interest_allowed: Amount::<Cents>::new(interest_allowed).expect("bounded basis result"),
            interest_basis_suspended: closing_values::<Cents>(
                self.business_interest.get(),
                interest_allowed,
            )
            .ok()?,
            excess_interest_basis_used: Amount::<Cents>::new(excess_basis)
                .expect("bounded basis result"),
            excess_interest_basis_suspended: closing_values::<Cents>(
                self.excess_business_interest.get(),
                excess_basis,
            )
            .ok()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::k1_loss::K1LossClass;

    fn cents(value: i64) -> Amount<Cents> {
        Amount::new(value).expect("nonnegative source cents")
    }

    fn dollars_amount(value: i64) -> Amount<Cents> {
        cents(value * 100)
    }

    fn first_two(amounts: &LossAmounts) -> [i64; 2] {
        [
            crate::kernel::ratio::dollars(amounts[K1LossClass::Ordinary].get()),
            crate::kernel::ratio::dollars(amounts[K1LossClass::RentalRealEstate].get()),
        ]
    }

    fn source(losses: &LossAmounts) -> PartnerBasis<'_> {
        PartnerBasis {
            opening: dollars_amount(0),
            contributions: dollars_amount(0),
            opening_liabilities: dollars_amount(0),
            ending_liabilities: dollars_amount(0),
            partnership_liabilities_assumed: dollars_amount(0),
            personal_liabilities_transferred: dollars_amount(0),
            income: dollars_amount(0),
            cash_distributions: dollars_amount(0),
            distributed_property_basis: dollars_amount(0),
            nondeductible: dollars_amount(0),
            depletion: dollars_amount(0),
            losses,
            business_interest: dollars_amount(0),
            excess_business_interest: dollars_amount(0),
            credit_reduction: dollars_amount(0),
            deemed_money: dollars_amount(0),
        }
    }

    #[test]
    fn liability_basis_is_distinct_from_direct_shareholder_debt() {
        // Pub. 541: $20,000 contribution plus $30,000 allocated nonrecourse debt.
        let losses = LossAmounts::default();
        let mut s = source(&losses);
        s.contributions = dollars_amount(20_000);
        s.ending_liabilities = dollars_amount(30_000);
        assert_eq!(crate::kernel::ratio::dollars(s.calculate().unwrap().ending.get()), 50_000);
        // All $60,000 recourse debt assigned to this partner instead.
        s.ending_liabilities = dollars_amount(60_000);
        assert_eq!(crate::kernel::ratio::dollars(s.calculate().unwrap().ending.get()), 80_000);
        // This establishes outside basis only, not amount at risk.
    }

    #[test]
    fn cash_and_debt_relief_precede_property_and_loss_allocations() {
        let losses = LossAmounts::of(&[
            (K1LossClass::Ordinary, dollars_amount(6_000)),
            (K1LossClass::RentalRealEstate, dollars_amount(3_000)),
        ]);
        let mut s = source(&losses);
        s.opening = dollars_amount(15_000);
        s.opening_liabilities = dollars_amount(8_000);
        s.ending_liabilities = dollars_amount(2_000);
        s.income = dollars_amount(4_000);
        s.cash_distributions = dollars_amount(4_000);
        s.distributed_property_basis = dollars_amount(3_000);
        s.nondeductible = dollars_amount(600);
        let r = s.calculate().unwrap();
        assert_eq!(first_two(&r.allowed), [3_600, 1_800]);
        assert_eq!(first_two(&r.suspended), [2_400, 1_200]);
        assert_eq!(crate::kernel::ratio::dollars(r.ending.get()), 0);
        assert_eq!(crate::kernel::ratio::dollars(r.distribution_gain.get()), 0);
        s.personal_liabilities_transferred = dollars_amount(10_000);
        let r = s.calculate().unwrap();
        assert_eq!(crate::kernel::ratio::dollars(r.distribution_gain.get()), 1_000);
        assert_eq!(crate::kernel::ratio::dollars(r.distributed_property_basis.get()), 0);
        assert_eq!(crate::kernel::ratio::dollars(r.nondeductible_carry.get()), 600);
        assert_eq!(first_two(&r.suspended), [6_000, 3_000]);
    }

    #[test]
    fn noncash_distribution_is_limited_without_creating_cash_gain() {
        let losses = LossAmounts::default();
        let mut s = source(&losses);
        s.opening = dollars_amount(1_000);
        s.distributed_property_basis = dollars_amount(5_000);
        let r = s.calculate().unwrap();
        assert_eq!(crate::kernel::ratio::dollars(r.distributed_property_basis.get()), 1_000);
        assert_eq!(crate::kernel::ratio::dollars(r.distribution_gain.get()), 0);
        assert_eq!(crate::kernel::ratio::dollars(r.ending.get()), 0);
    }

    #[test]
    fn expense_priority_and_single_interest_class_preserve_carryovers() {
        let losses = LossAmounts::of(&[(K1LossClass::Ordinary, dollars_amount(1_000))]);
        let mut s = source(&losses);
        s.opening = dollars_amount(2_500);
        s.nondeductible = dollars_amount(300);
        s.depletion = dollars_amount(200);
        s.business_interest = dollars_amount(500);
        s.excess_business_interest = dollars_amount(1_500);
        let r = s.calculate().unwrap();
        assert_eq!(crate::kernel::ratio::dollars(r.allowed[K1LossClass::Ordinary].get()), 667);
        assert_eq!(crate::kernel::ratio::dollars(r.interest_allowed.get()), 500);
        assert_eq!(crate::kernel::ratio::dollars(r.excess_interest_basis_used.get()), 833);
        assert_eq!(crate::kernel::ratio::dollars(r.excess_interest_basis_suspended.get()), 667);
        assert_eq!(crate::kernel::ratio::dollars(r.ending.get()), 0);
        s.opening = dollars_amount(250);
        let r = s.calculate().unwrap();
        assert_eq!(
            (
                crate::kernel::ratio::dollars(r.nondeductible_used.get()),
                crate::kernel::ratio::dollars(r.depletion_used.get())
            ),
            (150, 100)
        );
        assert_eq!(
            (
                crate::kernel::ratio::dollars(r.nondeductible_carry.get()),
                crate::kernel::ratio::dollars(r.depletion_carry.get())
            ),
            (150, 100)
        );
        assert_eq!(crate::kernel::ratio::dollars(r.allowed[K1LossClass::Ordinary].get()), 0);
        assert_eq!(crate::kernel::ratio::dollars(r.interest_basis_suspended.get()), 500);
        assert_eq!(crate::kernel::ratio::dollars(r.excess_interest_basis_suspended.get()), 1_500);
    }

    #[test]
    fn liability_assumption_nets_once_and_invalid_amounts_fail() {
        let losses = LossAmounts::default();
        let mut s = source(&losses);
        s.opening_liabilities = dollars_amount(10_000);
        s.ending_liabilities = dollars_amount(6_000);
        s.partnership_liabilities_assumed = dollars_amount(7_000);
        assert_eq!(crate::kernel::ratio::dollars(s.calculate().unwrap().ending.get()), 3_000);
        s.contributions = cents(i64::MAX);
        assert!(s.calculate().is_none());
    }
    #[test]
    fn source_cent_charity_limits_and_carries_conserve_each_boundary() {
        for current in [49, 50, 149] {
            let losses = LossAmounts::of(&[(K1LossClass::Charity, cents(current))]);
            let mut ample = source(&losses);
            ample.opening = cents(1_000);
            let allowed = ample.calculate().unwrap();
            assert_eq!(allowed.allowed[K1LossClass::Charity].get(), current);
            assert_eq!(allowed.suspended[K1LossClass::Charity].get(), 0);
            let held = source(&losses).calculate().unwrap();
            assert_eq!(held.allowed[K1LossClass::Charity].get(), 0);
            assert_eq!(held.suspended[K1LossClass::Charity].get(), current);
        }
        for (loss, opening, allowed, held) in [(149, 50, 50, 99), (249, 100, 100, 149)] {
            let losses = LossAmounts::of(&[(K1LossClass::Charity, cents(loss))]);
            let mut record = source(&losses);
            record.opening = cents(opening);
            let result = record.calculate().unwrap();
            assert_eq!(result.allowed[K1LossClass::Charity].get(), allowed);
            assert_eq!(result.suspended[K1LossClass::Charity].get(), held);
            assert_eq!(allowed + held, loss);
        }
        let current = LossAmounts::of(&[(K1LossClass::Charity, cents(49))]);
        let prior = LossAmounts::of(&[(K1LossClass::Charity, cents(50))]);
        let together = current.plus(&prior);
        let mut record = source(&together);
        record.opening = cents(100);
        let result = record.calculate().unwrap();
        assert_eq!(result.allowed[K1LossClass::Charity].get(), 99);
        assert_eq!(result.ending.get(), 1);
        let mut total_allowed = 0;
        for _ in 0..2 {
            let losses = LossAmounts::of(&[(K1LossClass::Charity, cents(249))]);
            let mut record = source(&losses);
            record.opening = cents(100);
            total_allowed += record.calculate().unwrap().allowed[K1LossClass::Charity].get();
        }
        assert_eq!(total_allowed, 200);
        assert_eq!(crate::kernel::ratio::dollars(total_allowed), 2);
    }
}
