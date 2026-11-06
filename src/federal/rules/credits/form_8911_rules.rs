// form_8911 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// Form 8911 personal-use refueling property credit, limited by the regular-tax
// capacity left after the earlier personal credits.
pub(super) struct Form8911In {
    #[cfg(test)]
    pub(super) regular_liability: i64,
    pub(super) regular_creditable_tax: i64,
    pub(super) preceding_homeowner_credits: i64,
    pub(super) dc_homebuyer_credit: i64,
    pub(super) residential_energy_credit: i64,
    pub(super) child_nonrefund: i64,
    pub(super) legacy_ev_credit: i64,
    pub(super) tentative: i64,
    pub(super) amt_credit: i64,
}

pub(super) fn form_8911_phase(pass: &Pass, r: &mut FederalResult, i: Form8911In) -> i64 {
    let &Pass { c, .. } = pass;
    let Form8911In {
        #[cfg(test)]
        regular_liability,
        regular_creditable_tax,
        preceding_homeowner_credits,
        dc_homebuyer_credit,
        residential_energy_credit,
        child_nonrefund,
        legacy_ev_credit,
        tentative,
        amt_credit,
    } = i;
    let personal_refueling_potential = c.personal_refueling.iter().enumerate().fold(0, |sum, (index, property)| {
        crate::require_localized!(
            r,
            (20260101..=20260630).contains(&property.placed_in_service),
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),"Refueling property credit",
            "ISSUE_PERSONAL_USE_REFUELING_PROPERTY_MUST_BE_PLACED_IN_SERVICE_DURING_20_B3682C42","Personal-use refueling property must be placed in service during 2026 and no later than June 30.",
        );
        // An ineligible tract or unit takes no credit; business or
        // investment use needs Form 3800 and is left out.
        if !property.personal_only {
            note_once_scoped(
                r,
                Message::new(
                    "WARN_REFUELING_PROPERTY_LEFT_OUT",
                    alloc::vec![MessageArg::Text(property.label.clone())],
                ), crate::IssueContext::Section(crate::TaxSection::MoreCredits).item("personal_refueling", index));
        }
        if !(property.eligible_tract && property.qualified_property && property.personal_only) {
            return sum;
        }
        sum + phase::cap(fraction(dollars(property.cost), 30, 100), [1000])
    });
    // Form 8911 lines 5–10: all earlier personal credits and Form 6251 line 9
    // reduce the regular-tax capacity. An unused personal credit cannot carry.
    #[cfg(test)]
    record_credit_base("Form 8911 limit", regular_liability);
    let personal_refueling_credit = phase::cap(
        personal_refueling_potential,
        [positive(
            regular_creditable_tax
                - preceding_homeowner_credits
                - dc_homebuyer_credit
                - residential_energy_credit
                - child_nonrefund
                - legacy_ev_credit
                - positive(tentative - amt_credit),
        )],
    );
    line(
        r,
        "personal_refueling_credit",
        personal_refueling_credit,
        &["credits.personal_refueling", "income_tax", "alternative_minimum_tax"],
        "https://www.irs.gov/instructions/i8911",
        msg!("EXPLAIN_PERSONAL_REFUELING_CREDIT"),
    );
    line(
        r,
        "personal_refueling_credit_unusable",
        personal_refueling_potential - personal_refueling_credit,
        &["credits.personal_refueling", "personal_refueling_credit"],
        "https://www.irs.gov/instructions/i8911",
        msg!("EXPLAIN_PERSONAL_REFUELING_CREDIT_UNUSABLE"),
    );
    personal_refueling_credit
}
