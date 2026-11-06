// form_5695 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct Form5695In<'a> {
    pub(super) homeowner_credits: &'a crate::forms::schedule_8812::HomeownerCredits,
}

pub(super) fn form_5695_phase(pass: &Pass, r: &mut FederalResult, i: Form5695In<'_>) -> i64 {
    let &Pass { c, .. } = pass;
    let Form5695In { homeowner_credits } = i;
    let energy_carry = c.residential_energy_carry.map_or(0, dollars);
    // The latest published Form 5695 credit-limit worksheet lists the DC
    // carryforward among credits taken before the residential energy credit.
    let residential_energy_credit = homeowner_credits.energy;
    line(
        r,
        "residential_energy_credit",
        residential_energy_credit,
        &["credits.residential_energy_carry", "income_tax", "credits"],
        "https://www.irs.gov/pub/irs-dft/f5695--dft.pdf",
        msg!("EXPLAIN_RESIDENTIAL_ENERGY_CREDIT"),
    );
    line(
        r,
        "residential_energy_carryforward_2027",
        crate::kernel::carry::closing_values::<crate::kernel::carry::Dollars>(
            energy_carry,
            residential_energy_credit,
        )
        .expect("energy credit use is capped by its balance")
        .get(),
        &["credits.residential_energy_carry", "residential_energy_credit"],
        "https://www.irs.gov/pub/irs-dft/f5695--dft.pdf",
        msg!("EXPLAIN_RESIDENTIAL_ENERGY_CARRYFORWARD_N"),
    );
    if c.residential_energy_carry.is_some() {
        r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "The 2026 Form 5695 carryforward uses the latest published residential-energy credit-limit worksheet. Reconcile its ordering with the final 2026 instructions before filing.", crate::Message::new("NOTE_THE_2026_FORM_5695_CARRYFORWARD_USES_THE_LATEST_PUBLISHED_RESIDE_62320D62", alloc::vec![])));
    }
    residential_energy_credit
}
