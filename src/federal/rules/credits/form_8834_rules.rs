// form_8834 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct Form8834In<'a> {
    pub(super) clean_vehicle: &'a crate::clean_vehicle::PersonalCredits,
    #[cfg(test)]
    pub(super) regular_liability: i64,
    pub(super) regular_creditable_tax: i64,
    pub(super) preceding_homeowner_credits: i64,
    pub(super) dc_homebuyer_credit: i64,
    pub(super) residential_energy_credit: i64,
    pub(super) child_nonrefund: i64,
    pub(super) tentative: i64,
    pub(super) amt_credit: i64,
}

pub(super) fn form_8834_phase(pass: &Pass, r: &mut FederalResult, i: Form8834In<'_>) -> i64 {
    let &Pass { c, .. } = pass;
    let Form8834In {
        clean_vehicle,
        #[cfg(test)]
        regular_liability,
        regular_creditable_tax,
        preceding_homeowner_credits,
        dc_homebuyer_credit,
        residential_energy_credit,
        child_nonrefund,
        tentative,
        amt_credit,
    } = i;
    let legacy_ev_potential = c.legacy_ev_passive_allowed.map_or(0, dollars);
    crate::require_localized!(
        r,
        legacy_ev_potential == 0 || (clean_vehicle.new == 0 && clean_vehicle.used == 0),
        &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
        "Legacy EV and clean vehicle credits",
        "ISSUE_THE_PUBLISHED_FORM_8834_LINE_3B_AND_FORM_8936_PREDECESSOR_LISTS_DO_6E3815EB",
        "The published Form 8834 line-3b and Form 8936 predecessor lists do not resolve their combined ordering. Review a return claiming both credits before using either tax limit.",
    );
    // Form 8834 lines 2–7 use regular tax plus Schedule 2 Part I additions,
    // then the listed earlier credits and Form 6251 tentative minimum tax.
    // General business, prior-minimum-tax, refueling and bond credits are
    // excluded from the line-3b predecessor list.
    #[cfg(test)]
    record_credit_base("Form 8834 limit", regular_liability);
    let legacy_ev_credit = phase::cap(
        legacy_ev_potential,
        [positive(
            regular_creditable_tax
                - preceding_homeowner_credits
                - dc_homebuyer_credit
                - residential_energy_credit
                - child_nonrefund
                - positive(tentative - amt_credit),
        )],
    );
    line(
        r,
        "legacy_ev_passive_credit",
        legacy_ev_credit,
        &["credits.legacy_ev_passive_allowed", "income_tax", "alternative_minimum_tax"],
        "https://www.irs.gov/pub/irs-pdf/f8834.pdf",
        msg!("EXPLAIN_LEGACY_EV_PASSIVE_CREDIT"),
    );
    line(
        r,
        "legacy_ev_passive_credit_unusable",
        legacy_ev_potential - legacy_ev_credit,
        &["credits.legacy_ev_passive_allowed", "legacy_ev_passive_credit"],
        "https://www.irs.gov/pub/irs-pdf/f8834.pdf",
        msg!("EXPLAIN_LEGACY_EV_PASSIVE_CREDIT_UNUSABLE"),
    );
    legacy_ev_credit
}
