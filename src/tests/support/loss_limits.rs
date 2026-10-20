// Test fixtures project owned held vectors from the actual shared owners.
// Keep existing independent expected figures at each source-owner test.
pub(crate) use crate::kernel::loss_limits::PassiveActivity;
use crate::kernel::{
    carry::{Amount, Dollars},
    loss_limits::{self, CharacterInput, PoolInput},
    source_fact::SourceFact,
};

pub(crate) fn passive_suspensions<Key>(
    activities: &[PassiveActivity<Key>],
    income: i64,
    allowance: i64,
) -> Option<alloc::vec::Vec<i64>> {
    passive_suspensions_in_units::<Dollars>(activities, income, allowance)
}

pub(crate) fn passive_suspensions_in_units<Unit>(
    activities: &[PassiveActivity<impl Sized>],
    income: i64,
    allowance: i64,
) -> Option<alloc::vec::Vec<i64>> {
    let SourceFact::Reported(pool) = loss_limits::passive_pool(SourceFact::Reported(PoolInput {
        activities,
        other_passive_income: Amount::<Unit>::new(income).ok()?,
        allowance: Amount::new(allowance).ok()?,
    }))
    .ok()?
    else {
        unreachable!("reported test pool remains reported")
    };
    Some(pool.suspended().to_vec())
}

pub(crate) fn passive_character_suspensions(
    nets: &[i64],
    suspended: i64,
) -> Option<alloc::vec::Vec<i64>> {
    passive_character_suspensions_in_units::<Dollars>(nets, suspended)
}

pub(crate) fn passive_character_suspensions_in_units<Unit>(
    nets: &[i64],
    suspended: i64,
) -> Option<alloc::vec::Vec<i64>> {
    let nets = nets.iter().copied().enumerate().collect::<alloc::vec::Vec<_>>();
    let SourceFact::Reported(parts) =
        loss_limits::passive_characters(SourceFact::Reported(CharacterInput {
            nets: &nets,
            suspended: Amount::<Unit>::new(suspended).ok()?,
        }))
        .ok()?
    else {
        unreachable!("reported test characters remain reported")
    };
    Some(parts.iter().map(|part| part.held()).collect())
}
