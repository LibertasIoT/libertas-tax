// form_8859 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct Form8859In<'a> {
    pub(super) homeowner_credits: &'a crate::forms::schedule_8812::HomeownerCredits,
}

pub(super) fn form_8859_phase(pass: &Pass, r: &mut FederalResult, i: Form8859In<'_>) -> i64 {
    let &Pass { c, .. } = pass;
    let Form8859In { homeowner_credits } = i;
    let dc_homebuyer_carry = c.dc_homebuyer_carry.map_or(0, dollars);
    // Form 8859 line 2 subtracts the foreign credit and the listed personal
    // credits from Form 1040 line 18. Form 8801 then uses the remaining regular
    // liability after allowable credits, including this carryforward.
    let dc_homebuyer_credit = homeowner_credits.dc;
    line(
        r,
        "dc_homebuyer_credit",
        dc_homebuyer_credit,
        &["credits.dc_homebuyer_carry", "income_tax", "credits"],
        "https://www.irs.gov/pub/irs-dft/f8859--dft.pdf",
        msg!("EXPLAIN_DC_HOMEBUYER_CREDIT"),
    );
    line(
        r,
        "dc_homebuyer_carryforward_2027",
        crate::kernel::carry::closing_values::<crate::kernel::carry::Dollars>(
            positive(dc_homebuyer_carry),
            dc_homebuyer_credit,
        )
        .expect("DC credit use is capped by its balance")
        .get(),
        &["credits.dc_homebuyer_carry", "dc_homebuyer_credit"],
        "https://www.irs.gov/pub/irs-dft/f8859--dft.pdf",
        msg!("EXPLAIN_DC_HOMEBUYER_CARRYFORWARD_N"),
    );
    dc_homebuyer_credit
}
