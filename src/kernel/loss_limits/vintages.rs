use super::LossLimitError;
use crate::kernel::carry::{self, Amount, Entry};
use crate::kernel::{phase, ratio, source_fact::SourceFact};
use alloc::vec::Vec;

// Source adapters retain unique actual-year validation, pre-2018 eligibility
// and oldest-first order. Opaque keys never supply or infer a tax year.
// Original qualified percentage survives partial releases at every provision.
pub(crate) struct LossVintage<Key, Unit> {
    pub key: Key,
    pub original: Amount<Unit>,
    pub original_qualified: Amount<Unit>,
    pub remaining: Amount<Unit>,
    pub remaining_qualified: Amount<Unit>,
}

pub(crate) struct VintageInput<'a, Key, Unit> {
    pub vintages: &'a [LossVintage<Key, Unit>],
    pub amount: Amount<Unit>,
    pub year: u32,
    pub order: &'a [usize],
}

pub(crate) struct VintageRelease<'a, Key, Unit> {
    pub amount: Amount<Unit>,
    pub qualified: Amount<Unit>,
    pub remaining: Vec<LossVintage<&'a Key, Unit>>,
}

pub(crate) fn release_vintages<Key, Unit>(
    input: SourceFact<VintageInput<'_, Key, Unit>>,
) -> Result<SourceFact<VintageRelease<'_, Key, Unit>>, LossLimitError> {
    let input = match input {
        SourceFact::Reported(value) => value,
        SourceFact::NotReported => return Ok(SourceFact::NotReported),
        SourceFact::Unavailable => return Ok(SourceFact::Unavailable),
    };
    if input.vintages.iter().any(|v| {
        v.original.get() == 0
            || v.original_qualified.get() > v.original.get()
            || v.remaining.get() > v.original.get()
            || v.remaining_qualified.get() > v.remaining.get()
            || v.remaining_qualified.get() > v.original_qualified.get()
            || v.remaining.get() - v.remaining_qualified.get()
                > v.original.get() - v.original_qualified.get()
    }) {
        return Err(LossLimitError::InvalidAmount);
    }
    let total = input
        .vintages
        .iter()
        .try_fold(0i64, |sum, v| sum.checked_add(v.remaining.get()))
        .ok_or(LossLimitError::Overflow)?;
    if input.amount.get() > total {
        return Err(LossLimitError::Footing);
    }
    let entries: Vec<_> = input
        .vintages
        .iter()
        .map(|v| {
            Ok(Entry {
                key: &v.key,
                amount: Amount::<Unit>::new(v.remaining.get())
                    .map_err(|_| LossLimitError::InvalidAmount)?,
                last_use_year: None,
            })
        })
        .collect::<Result<_, LossLimitError>>()?;
    let settlements = carry::consume(&entries, input.amount, input.year, input.order)
        .map_err(|_| LossLimitError::Footing)?;
    let mut released = 0i64;
    let mut qualified = 0i64;
    let mut remaining = Vec::new();
    for (v, settled) in input.vintages.iter().zip(settlements) {
        let used = settled.used.get();
        let part = phase::floor(
            phase::cap(
                ratio::fraction(used, v.original_qualified.get(), v.original.get()),
                [v.remaining_qualified.get()],
            ),
            ratio::positive(used - (v.remaining.get() - v.remaining_qualified.get())),
        );
        released = released.checked_add(used).ok_or(LossLimitError::Overflow)?;
        qualified = qualified.checked_add(part).ok_or(LossLimitError::Overflow)?;
        remaining.push(LossVintage {
            key: &v.key,
            original: Amount::new(v.original.get()).map_err(|_| LossLimitError::InvalidAmount)?,
            original_qualified: Amount::new(v.original_qualified.get())
                .map_err(|_| LossLimitError::InvalidAmount)?,
            remaining: settled.closing,
            remaining_qualified: carry::closing_values::<Unit>(v.remaining_qualified.get(), part)
                .map_err(|_| LossLimitError::Footing)?,
        });
    }
    Ok(SourceFact::Reported(VintageRelease {
        amount: Amount::new(released).map_err(|_| LossLimitError::InvalidAmount)?,
        qualified: Amount::new(qualified).map_err(|_| LossLimitError::InvalidAmount)?,
        remaining,
    }))
}

pub(crate) struct NextLimit<Key, Unit> {
    pub allowed_qualified: Amount<Unit>,
    pub held: Option<LossVintage<Key, Unit>>,
}

impl<Key, Unit> VintageRelease<'_, Key, Unit> {
    // A release caught by the next provision becomes THAT provision's current
    // vintage. The caller supplies its actual current-year key, never an origin
    // key inferred here. This does not alter the preceding provision's balances.
    pub(crate) fn after_next_limit<NextKey>(
        &self,
        allowed: Amount<Unit>,
        next_key: NextKey,
    ) -> Result<NextLimit<NextKey, Unit>, LossLimitError> {
        if self.qualified.get() > self.amount.get() {
            return Err(LossLimitError::Footing);
        }
        let held = carry::closing_values::<Unit>(self.amount.get(), allowed.get())
            .map_err(|_| LossLimitError::Footing)?;
        if held.get() == 0 {
            return Ok(NextLimit {
                allowed_qualified: Amount::new(self.qualified.get())
                    .map_err(|_| LossLimitError::InvalidAmount)?,
                held: None,
            });
        }
        let qualified = ratio::fraction(held.get(), self.qualified.get(), self.amount.get());
        Ok(NextLimit {
            allowed_qualified: carry::closing_values::<Unit>(self.qualified.get(), qualified)
                .map_err(|_| LossLimitError::Footing)?,
            held: Some(LossVintage {
                key: next_key,
                original: Amount::new(held.get()).map_err(|_| LossLimitError::InvalidAmount)?,
                original_qualified: Amount::new(qualified)
                    .map_err(|_| LossLimitError::InvalidAmount)?,
                remaining: held,
                remaining_qualified: Amount::new(qualified)
                    .map_err(|_| LossLimitError::InvalidAmount)?,
            }),
        })
    }
}

#[cfg(test)]
#[path = "vintages_tests.rs"]
mod tests;
