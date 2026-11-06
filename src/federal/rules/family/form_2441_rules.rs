// form_2441 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(crate) fn care_earned_input(t: &Totals, earned: [i64; 2]) -> [i64; 2] {
    core::array::from_fn(|i| {
        earned[i] + dollars(t.combat_pay[i] + t.medicaid_excluded[i] + t.medicaid_in_box1[i])
    })
}

pub(super) struct CareBenefitsIn<'a> {
    pub(super) care_current: &'a crate::care::CareDeductions,
    pub(super) earned: [i64; 2],
    pub(super) compensation: [i64; 2],
    pub(super) optional_eic: [i64; 2],
}

pub(super) struct CareBenefitsOut<'a> {
    pub(super) form_2441_context: crate::forms::form_2441::Form2441Context<'a>,
    pub(super) form_2441: crate::forms::form_2441::Form2441,
    pub(super) benefit_wages: i64,
    pub(super) earned: [i64; 2],
    pub(super) compensation: [i64; 2],
}

pub(super) fn care_benefits_phase<'d>(
    pass: &Pass<'d>,
    r: &mut FederalResult,
    t: &Totals,
    care_next: &mut Option<crate::care::CareDeductions>,
    input: CareBenefitsIn<'_>,
) -> core::ops::ControlFlow<(), CareBenefitsOut<'d>> {
    let &Pass { d, s, c, j, family, care_original, .. } = pass;
    let CareBenefitsIn { care_current, mut earned, mut compensation, optional_eic } = input;
    let form_2441_context = crate::forms::form_2441::Form2441Context {
        draft: d,
        setup: s,
        credits: c,
        joint: j,
        care_people: family.care_people,
        care_expenses: family.care_expenses,
        earned: care_earned_input(t, earned),
        optional_eic,
        deduction_owners: care_current.owners(care_original),
    };
    let Some(form_2441) = crate::forms::form_2441::compute(&form_2441_context, r) else {
        crate::issue_localized!(
            r,
            &crate::IssueContext::Section(crate::TaxSection::Care),
            "Child and dependent care",
            "CARE_SOURCE_AMOUNTS_UNAVAILABLE",
            "The combined dependent-care source amounts exceed the supported arithmetic range.",
        );
        r.state = ResultState::Unsupported;
        r.lines.clear();
        r.return_lines.clear();
        return core::ops::ControlFlow::Break(());
    };
    let next_care = crate::care::CareDeductions::calculate(&c.care, form_2441.deducted);
    if next_care != *care_current {
        *care_next = Some(next_care);
        return core::ops::ControlFlow::Break(());
    }
    let benefit_wages = form_2441.taxable.iter().sum::<i64>();
    for i in 0..2 {
        earned[i] += form_2441.taxable[i];
        compensation[i] += form_2441.taxable[i];
    }
    core::ops::ControlFlow::Continue(CareBenefitsOut {
        form_2441_context,
        form_2441,
        benefit_wages,
        earned,
        compensation,
    })
}
