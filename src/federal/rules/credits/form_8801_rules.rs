// form_8801 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct Form8801In {
    pub(super) regular_liability: i64,
    pub(super) foreign_credit: i64,
    pub(super) other_nonrefundable_before_business: i64,
    pub(super) general_business_credit: i64,
    pub(super) tentative: i64,
    pub(super) amt_credit: i64,
}

pub(super) fn form_8801_phase(pass: &Pass, r: &mut FederalResult, i: Form8801In) -> i64 {
    let &Pass { s, c, .. } = pass;
    let Form8801In {
        regular_liability,
        foreign_credit,
        other_nonrefundable_before_business,
        general_business_credit,
        tentative,
        amt_credit,
    } = i;
    #[cfg(test)]
    record_credit_base(
        "Form 8801 line 22",
        (regular_liability
            - foreign_credit
            - other_nonrefundable_before_business
            - general_business_credit)
            + foreign_credit
            + other_nonrefundable_before_business
            + general_business_credit,
    );
    let minimum_credit_use = crate::minimum_tax::calculate(
        c.minimum_tax.as_ref(),
        s.filing_status(),
        positive(
            regular_liability
                - foreign_credit
                - other_nonrefundable_before_business
                - general_business_credit,
        ),
        positive(tentative - amt_credit),
        r,
    );
    minimum_credit_use.map_or(0, |credit| credit.used)
}
