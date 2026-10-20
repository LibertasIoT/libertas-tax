// Coverage: prior Worksheet5-1 third-tier source totals and casualty character.
// Independent regular/AMT records feed the existing HOME tier owner. Unknown
// character never becomes depreciation. Sources: Pub527 Worksheet5-1; i4684;
// dispatcher2026-10-19: floor exact remaining capacity once at the cents boundary.
use crate::kernel::ratio::exact::{self, Exact};
use crate::kernel::source_fact::SourceFact;
use crate::validation::ValidationIssue;
use crate::{FederalRental, SourceCents};

pub(crate) const TOTAL: &str = "RENTAL_HOME_PRIOR_TOTAL_UNAVAILABLE";
pub(crate) const CHARACTER: &str = "RENTAL_HOME_PRIOR_CHARACTER_UNAVAILABLE";
pub(crate) const OVERFLOW: &str = "ACTIVITY_LOSS_AMOUNT_UNREPRESENTABLE";
pub(crate) const INVALID: &str = "RENTAL_HOME_PRIOR_CARRY_RECORDS";

pub(crate) fn known(source: &SourceCents) -> Option<i64> {
    match source.source_fact().ok()? {
        SourceFact::Reported(value) => Some(value),
        SourceFact::NotReported => Some(0),
        SourceFact::Unavailable => None,
    }
}

pub(crate) fn total(source: &FederalRental, amt: bool) -> Option<i64> {
    known(if amt {
        &source.amt_home_depreciation_carryover
    } else {
        &source.home_depreciation_carryover
    })
}

pub(crate) fn prior(
    source: &FederalRental,
    amt: bool,
) -> Result<crate::casualty::home::ThirdTierCarry, &'static str> {
    let total_cents = total(source, amt).ok_or(TOTAL)?;
    let casualty_cents = known(if amt {
        &source.amt_home_casualty_carryover
    } else {
        &source.home_casualty_carryover
    });
    if casualty_cents.is_some_and(|amount| amount > total_cents) {
        return Err(INVALID);
    }
    Ok(crate::casualty::home::ThirdTierCarry { total_cents, casualty_cents })
}

pub(crate) fn issue(source: &FederalRental) -> Option<ValidationIssue> {
    for (total, casualty, total_field, casualty_field) in [
        (
            &source.home_depreciation_carryover,
            &source.home_casualty_carryover,
            "home_depreciation_carryover",
            "home_casualty_carryover",
        ),
        (
            &source.amt_home_depreciation_carryover,
            &source.amt_home_casualty_carryover,
            "amt_home_depreciation_carryover",
            "amt_home_casualty_carryover",
        ),
    ] {
        for (value, field) in [(total, total_field), (casualty, casualty_field)] {
            if value.source_fact().is_err() {
                return Some(ValidationIssue::new(INVALID).at_field(field));
            }
        }
        if known(casualty).zip(known(total)).is_some_and(|(part, total)| part > total) {
            return Some(ValidationIssue::new(INVALID).at_field(casualty_field));
        }
    }
    None
}

pub(crate) fn tiers(
    source: &FederalRental,
    amt: bool,
    current_depreciation_cents: i64,
    capacity: &Exact,
) -> Result<crate::casualty::home::ThirdTierResult, &'static str> {
    // Capacity is a limit. Keep exact upstream arithmetic; one downward cent
    // projection at this native-owner boundary cannot exceed the available room.
    let capacity = crate::kernel::phase::floor(capacity.clone(), exact::whole(0));
    let capacity = i64::try_from(capacity.floor().to_integer()).map_err(|_| OVERFLOW)?;
    let prior = prior(source, amt)?;
    // Arithmetic safety: the cents owner's closing ledger must fit i64.
    let closing_bound = (i128::from(current_depreciation_cents) + i128::from(prior.total_cents)
        - i128::from(capacity))
    .max(0);
    i64::try_from(closing_bound).map_err(|_| OVERFLOW)?;
    crate::casualty::home::third_tier(0, 0, current_depreciation_cents, prior, capacity)
        .map_err(|_| INVALID)
}
