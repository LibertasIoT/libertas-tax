// form_2106 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct EmployeeExpensesIn<'a> {
    pub(super) selected: Option<&'a [crate::section179_return::Selected; 2]>,
}

pub(super) struct EmployeeExpensesOut {
    pub(super) employee: crate::employee_expenses::Calculation,
    pub(super) employee_recapture: [i64; 2],
    pub(super) employee_ordinary: i64,
}

pub(super) fn employee_expenses_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &mut Totals,
    i: EmployeeExpensesIn<'_>,
) -> EmployeeExpensesOut {
    let &Pass { d, context, .. } = pass;
    let EmployeeExpensesIn { selected } = i;
    // Form 2106: the lines, the line 8 excess (wage income) and the vehicles'
    // AMT depreciation difference join the totals before the income is read.
    let employee = crate::employee_expenses::calculate_with_section179(
        d,
        r,
        selected,
        Some(context.macrs_elections),
    );
    let employee_recapture = employee.recapture_by_owner();
    t.other_8z += employee_recapture.iter().sum::<i64>();
    for (wages, excess) in t.wages.iter_mut().zip(employee.excess()) {
        *wages += excess;
    }
    t.business_depreciation_adjustment += employee.amt_adjustment(d);
    employee_personal_parts(d, t, r, &employee);
    let employee_ordinary = employee.ordinary_deduction();
    EmployeeExpensesOut { employee, employee_recapture, employee_ordinary }
}
