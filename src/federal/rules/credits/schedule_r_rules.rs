// schedule_r items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct ScheduleRIn<'a> {
    pub(super) t: &'a Totals,
    pub(super) agi: i64,
    pub(super) social: i64,
}

pub(super) fn schedule_r_phase(pass: &Pass, r: &mut FederalResult, i: ScheduleRIn<'_>) -> i64 {
    let &Pass { s, p, c, .. } = pass;
    let ScheduleRIn { t, agi, social } = i;
    let age_qualified =
        core::iter::once(&p.taxpayer).chain(p.spouse.iter()).filter(|v| v.senior()).count();
    let mut disability = [0; 2];
    for (owner, person) in core::iter::once(&p.taxpayer).chain(p.spouse.iter()).enumerate() {
        crate::require_localized!(
            r,
            // Only a Schedule R claim needs the People page to say disabled; a
            // pension before minimum retirement age is wages on line 1h and
            // asks nothing of that page (2026-10-09).
            t.disability[owner] == 0 || person.disabled,
            &crate::IssueContext::owner(
                if owner == 0 { crate::TaxSection::People } else { crate::TaxSection::Spouse },
                if owner == 0 { Owner::Taxpayer } else { Owner::Spouse },
            )
            .alternatives(&[
                if owner == 0 { crate::TaxSection::People } else { crate::TaxSection::Spouse },
                crate::TaxSection::Retirement,
                crate::TaxSection::Wages,
            ]),
            "Disability retirement",
            "ISSUE_TAXABLE_DISABILITY_RETIREMENT_INCOME_REQUIRES_CONSISTENT_PERMANENT_7B8EE958",
            "Taxable disability retirement income requires consistent permanent-and-total-disability facts in People.",
        );
        if !person.senior() {
            disability[owner] = dollars(t.disability[owner]);
        }
    }
    crate::extended_rules::elderly_credit(
        s.filing_status(),
        s.lived_apart_all_year(),
        age_qualified,
        disability,
        agi,
        positive(dollars(t.social) - social)
            + dollars(c.exempt_pensions.iter().map(|v| v.amount).sum()),
    )
}
