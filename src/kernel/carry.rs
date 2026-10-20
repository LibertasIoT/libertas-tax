use alloc::vec::Vec;
use core::marker::PhantomData;

pub(crate) enum Cents {}
pub(crate) enum Dollars {}

pub(crate) struct Amount<Unit> {
    value: i64,
    unit: PhantomData<Unit>,
}
impl<Unit> Amount<Unit> {
    pub(crate) fn new(value: i64) -> Result<Self, LedgerError> {
        if value < 0 {
            return Err(LedgerError::InvalidAmount);
        }
        Ok(Self { value, unit: PhantomData })
    }
    pub(crate) fn get(&self) -> i64 {
        self.value
    }
}
// Manual impls: derives would require the Unit marker itself to implement each trait.
impl<Unit> Clone for Amount<Unit> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<Unit> Copy for Amount<Unit> {}
impl<Unit> Default for Amount<Unit> {
    fn default() -> Self {
        Self { value: 0, unit: PhantomData }
    }
}
impl<Unit> PartialEq for Amount<Unit> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl<Unit> Eq for Amount<Unit> {}
impl<Unit> core::fmt::Debug for Amount<Unit> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.value.fmt(f)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LedgerError {
    InvalidAmount,
    InvalidOrder,
}

pub(crate) struct Settlement<Unit> {
    pub(crate) used: Amount<Unit>,
    pub(crate) expired: Amount<Unit>,
    pub(crate) closing: Amount<Unit>,
}

// The caller establishes eligibility, absorption and the inclusive expiry year.
// Different source classes and units never share a settlement implicitly.
pub(crate) fn settle<Unit>(
    opening: Amount<Unit>,
    used: Amount<Unit>,
    expires_after_use: bool,
) -> Result<Settlement<Unit>, LedgerError> {
    if used.value > opening.value {
        return Err(LedgerError::InvalidAmount);
    }
    let left = native_balance(opening.value, used.value);
    Ok(Settlement {
        used,
        expired: Amount::new(if expires_after_use { left } else { 0 })?,
        closing: Amount::new(if expires_after_use { 0 } else { left })?,
    })
}

pub(crate) struct Entry<Key, Unit> {
    pub(crate) key: Key,
    pub(crate) amount: Amount<Unit>,
    pub(crate) last_use_year: Option<u32>,
}

// Explicit order comes from the source owner, including same-year credit
// priority. Results retain input indexing. No source pools are combined here.
pub(crate) fn consume<Key, Unit>(
    entries: &[Entry<Key, Unit>],
    capacity: Amount<Unit>,
    year: u32,
    order: &[usize],
) -> Result<Vec<Settlement<Unit>>, LedgerError> {
    if order.len() != entries.len() {
        return Err(LedgerError::InvalidOrder);
    }
    let mut seen = alloc::vec![false; entries.len()];
    for &index in order {
        let Some(seen) = seen.get_mut(index) else {
            return Err(LedgerError::InvalidOrder);
        };
        if *seen {
            return Err(LedgerError::InvalidOrder);
        }
        *seen = true;
    }
    let mut result: Vec<_> = entries
        .iter()
        .map(|e| {
            settle(
                Amount::new(e.amount.value)?,
                Amount::new(0)?,
                e.last_use_year.is_some_and(|last| last <= year),
            )
        })
        .collect::<Result<_, _>>()?;
    let mut remaining = capacity;
    for &index in order {
        let entry = &entries[index];
        result[index] = if entry.last_use_year.is_some_and(|last| last < year) {
            settle(Amount::new(entry.amount.value)?, Amount::new(0)?, true)?
        } else {
            consume_step(
                Amount::new(entry.amount.value)?,
                &mut remaining,
                entry.last_use_year.is_some_and(|last| last <= year),
            )?
        };
    }
    Ok(result)
}

// Preserve callers that project a cohort and allocate its use before reading
// the next cohort. Grouping and source projection remain with the tax owner.
pub(crate) fn consume_step<Unit>(
    opening: Amount<Unit>,
    remaining_capacity: &mut Amount<Unit>,
    expires_after_use: bool,
) -> Result<Settlement<Unit>, LedgerError> {
    let used = native_consume_step(opening.value, &mut remaining_capacity.value);
    settle(opening, Amount::new(used)?, expires_after_use)
}

// Preserve a legacy native-width cohort sum and its literal min/subtract
// sequence, even when a release-mode sum wraps. Unlike Amount settlement,
// this adapter makes no nonnegative-input claim; the source owns its units.
pub(crate) fn native_consume_step(opening: i64, remaining_capacity: &mut i64) -> i64 {
    let used = (*remaining_capacity).min(opening);
    *remaining_capacity = native_balance(*remaining_capacity, used);
    used
}

// Signed worksheet accounts retain their original native-width subtraction.
// Eligibility, source units and any explicit zero floor belong to the caller.
pub(crate) fn native_balance(opening: i64, debit: i64) -> i64 {
    opening - debit
}

pub(crate) fn closing_values<Unit>(opening: i64, used: i64) -> Result<Amount<Unit>, LedgerError> {
    Ok(settle_values::<Unit>(opening, used, false)?.closing)
}

// Legacy worksheet balances can have signed source adjustments or a debit
// exceeding the preliminary opening. Preserve the native subtraction before
// the explicitly requested zero floor, including its overflow behavior.
pub(crate) fn native_remaining<Unit>(opening: i64, debit: i64) -> Amount<Unit> {
    Amount { value: super::ratio::positive(native_balance(opening, debit)), unit: PhantomData }
}

// Source owners project statutory priority from their retained origin/category
// key. Stable sorting preserves same-priority source order without copying tax
// period or credit-rank policy into the ledger.
pub(crate) fn consume_by_key<Key, Unit, Priority: Ord>(
    entries: &[Entry<Key, Unit>],
    capacity: Amount<Unit>,
    year: u32,
    mut priority: impl FnMut(&Key) -> Priority,
) -> Result<Vec<Settlement<Unit>>, LedgerError> {
    let mut order: Vec<_> = (0..entries.len()).collect();
    order.sort_by_key(|&index| priority(&entries[index].key));
    consume(entries, capacity, year, &order)
}

// Convenience adapter for already-projected primitive values; units remain
// explicit at the call site and invalid values are never clamped.
pub(crate) fn settle_values<Unit>(
    opening: i64,
    used: i64,
    expires: bool,
) -> Result<Settlement<Unit>, LedgerError> {
    settle(Amount::new(opening)?, Amount::new(used)?, expires)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settlement_conserves_opening_across_use_and_expiry() {
        for opening in 0..=64 {
            for used in 0..=opening {
                for expires in [false, true] {
                    let s = settle_values::<Cents>(opening, used, expires).unwrap();
                    assert_eq!(s.used.get() + s.expired.get() + s.closing.get(), opening);
                    assert_eq!(s.used.get(), used);
                    assert_eq!(s.closing.get() == 0, expires || used == opening);
                }
            }
        }
    }

    #[test]
    fn settlement_rejects_negative_and_excess_use() {
        for (opening, used) in [(-1, 0), (0, -1), (7, 8)] {
            assert!(settle_values::<Dollars>(opening, used, false).is_err());
        }
    }

    fn entries() -> [Entry<&'static str, Dollars>; 3] {
        [("expired", 5, Some(2025)), ("last-year", 7, Some(2026)), ("ongoing", 11, None)].map(
            |(key, amount, last_use_year)| Entry {
                key,
                amount: Amount::new(amount).unwrap(),
                last_use_year,
            },
        )
    }

    #[test]
    fn explicit_priority_preserves_indices_and_uses_inclusive_expiry() {
        let entries = entries();
        let settlements = consume(&entries, Amount::new(15).unwrap(), 2026, &[2, 1, 0]).unwrap();
        let values: Vec<_> = entries
            .iter()
            .zip(settlements)
            .map(|(e, s)| (e.key, s.used.get(), s.expired.get(), s.closing.get()))
            .collect();
        assert_eq!(
            values,
            alloc::vec![("expired", 0, 5, 0), ("last-year", 4, 3, 0), ("ongoing", 11, 0, 0)]
        );
    }

    #[test]
    fn order_must_address_every_entry_exactly_once() {
        for order in [&[0, 1][..], &[0, 0, 2], &[0, 1, 3]] {
            assert!(consume(&entries(), Amount::new(1).unwrap(), 2026, order).is_err());
        }
    }

    #[test]
    fn consumption_does_not_sum_unneeded_openings() {
        let entries = [0, 1].map(|key| Entry {
            key,
            amount: Amount::<Cents>::new(i64::MAX).unwrap(),
            last_use_year: None,
        });
        let settlements = consume(&entries, Amount::new(i64::MAX).unwrap(), 2026, &[1, 0]).unwrap();
        assert_eq!(settlements[0].closing.get(), i64::MAX);
        assert_eq!(settlements[1].used.get(), i64::MAX);
    }
}

#[cfg(test)]
mod ordering_tests {
    use super::*;

    #[test]
    fn native_consumption_keeps_signed_cohorts_and_capacity() {
        for (opening, capacity, used, closing) in [(8, 5, 5, 0), (-3, 5, -3, 8), (5, -3, -3, 0)] {
            let mut remaining = capacity;
            assert_eq!(native_consume_step(opening, &mut remaining), used);
            assert_eq!(remaining, closing);
        }
    }
    #[test]
    fn legacy_balances_preserve_signed_adjustments_and_explicit_zero_floor() {
        for (opening, debit, expected) in [(5, 8, 0), (5, -3, 8), (-5, -8, 3), (-5, 0, 0)] {
            assert_eq!(native_remaining::<Dollars>(opening, debit).get(), expected);
        }
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "overflow")]
    fn legacy_native_subtraction_keeps_overflow() {
        let _ = native_remaining::<Dollars>(i64::MIN, 1);
    }

    #[test]
    fn equal_priority_keeps_source_order_and_original_result_indices() {
        let entries = [(2025, 8), (2024, 10), (2025, 3)].map(|(key, value)| Entry {
            key,
            amount: Amount::<Dollars>::new(value).unwrap(),
            last_use_year: None,
        });
        let result =
            consume_by_key(&entries, Amount::new(15).unwrap(), 2026, |year| *year).unwrap();
        assert_eq!(result.iter().map(|s| s.used.get()).collect::<Vec<_>>(), alloc::vec![5, 10, 0]);
    }
    #[test]
    fn sequential_cohorts_share_capacity_and_settle_after_use() {
        let mut capacity = Amount::<Cents>::new(7).unwrap();
        let first = consume_step(Amount::new(10).unwrap(), &mut capacity, true).unwrap();
        assert_eq!((first.used.get(), first.expired.get(), first.closing.get()), (7, 3, 0));
        let second = consume_step(Amount::new(5).unwrap(), &mut capacity, false).unwrap();
        assert_eq!((second.used.get(), second.expired.get(), second.closing.get()), (0, 0, 5));
        assert_eq!(capacity.get(), 0);
    }
}
