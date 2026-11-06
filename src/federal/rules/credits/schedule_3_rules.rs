// schedule_3 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct Schedule3CreditTotalsIn {
    pub(super) elderly: i64,
    pub(super) education_used: i64,
    pub(super) care_used: i64,
    pub(super) saver: i64,
    pub(super) adoption_used: i64,
    pub(super) minimum_credit: i64,
    pub(super) mortgage_credit_used: i64,
    pub(super) dc_homebuyer_credit: i64,
    pub(super) residential_energy_credit: i64,
    pub(super) personal_refueling_credit: i64,
    pub(super) general_business_credit: i64,
    pub(super) used_clean_vehicle_credit: i64,
    pub(super) new_clean_vehicle_credit: i64,
    pub(super) legacy_ev_credit: i64,
    pub(super) foreign_credit: i64,
    pub(super) bba_schedule_3_credit: i64,
    pub(super) bond_credit: i64,
    pub(super) child_nonrefund: i64,
}

pub(super) struct Schedule3CreditTotalsOut {
    pub(super) other_credits: i64,
    pub(super) credits: i64,
}

pub(super) fn schedule_3_credit_totals_phase(
    i: Schedule3CreditTotalsIn,
) -> Schedule3CreditTotalsOut {
    let Schedule3CreditTotalsIn {
        elderly,
        education_used,
        care_used,
        saver,
        adoption_used,
        minimum_credit,
        mortgage_credit_used,
        dc_homebuyer_credit,
        residential_energy_credit,
        personal_refueling_credit,
        general_business_credit,
        used_clean_vehicle_credit,
        new_clean_vehicle_credit,
        legacy_ev_credit,
        foreign_credit,
        bba_schedule_3_credit,
        bond_credit,
        child_nonrefund,
    } = i;
    // The foreign tax credit is figured on Form 1116 line 20 already decreased by a
    // negative Form 8978 line 14, so it no longer competes with Schedule 3 line 6l.
    let prior_other_credits = elderly
        + education_used
        + care_used
        + saver
        + adoption_used
        + minimum_credit
        + mortgage_credit_used
        + dc_homebuyer_credit
        + residential_energy_credit
        + personal_refueling_credit
        + general_business_credit
        + used_clean_vehicle_credit
        + new_clean_vehicle_credit
        + legacy_ev_credit;
    let other_credits = prior_other_credits + foreign_credit + bba_schedule_3_credit + bond_credit;
    let credits = child_nonrefund + other_credits;
    Schedule3CreditTotalsOut { other_credits, credits }
}
