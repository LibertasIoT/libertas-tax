#[allow(unused_imports)]
use super::*;
pub(super) fn education_expenses_record(
    v: &EducationAccountExpenses,
) -> FederalEducationAccountExpenses {
    FederalEducationAccountExpenses {
        students: v
            .students
            .iter()
            .map(|s| FederalEducationAccountStudent {
                person: s.person,
                college: s.college.clone(),
                other_college: s.other_college,
                k12: s.k12,
                k12_coverdell: s.k12_coverdell,
                apprenticeship: s.apprenticeship,
                loan_payments: s.loan_payments,
                loan_payments_before: s.loan_payments_before,
                academy_costs: s.academy_costs,
                used_elsewhere: s.used_elsewhere,
            })
            .collect(),
    }
}
pub(super) fn education_expenses_item(
    data: &FederalEducationAccountExpenses,
) -> EducationAccountExpenses {
    EducationAccountExpenses {
        students: data
            .students
            .iter()
            .map(|s| EducationAccountStudent {
                person: s.person,
                college: s.college.clone(),
                other_college: s.other_college,
                k12: s.k12,
                k12_coverdell: s.k12_coverdell,
                apprenticeship: s.apprenticeship,
                loan_payments: s.loan_payments,
                loan_payments_before: s.loan_payments_before,
                academy_costs: s.academy_costs,
                used_elsewhere: s.used_elsewhere,
            })
            .collect(),
    }
}
// The existing Form 1099-Q box relationships and source choices, in native order.
pub(super) fn q1099_input_issue(v: &crate::Form1099QPaper) -> Option<ValidationIssue> {
    use crate::{Form1099QDistributionCode as Code, Form1099QDistributionFrom as From};
    let qtp = matches!(v.distribution_from, From::PrivateQtp | From::StateQtp);
    let boxes = (qtp || v.fair_market_value.is_none() || v.earnings != 0 || v.basis != 0)
        .then_some((v.earnings, v.basis));
    if boxes.is_some_and(|(earnings, basis)| {
        i128::from(earnings) + i128::from(basis) != i128::from(v.gross_distribution)
    }) {
        return Some(
            ValidationIssue::new("Q1099_BOXES_DO_NOT_ADD_UP").at_field("gross_distribution"),
        );
    }
    if v.trustee_to_trustee {
        return None;
    }
    if v.qtp_to_roth_ira {
        if !qtp {
            return Some(
                ValidationIssue::new("Q1099_ROTH_ROLLOVER_NOT_529").at_field("distribution_from"),
            );
        }
        if boxes.is_some_and(|(earnings, _)| earnings < 0) {
            return Some(ValidationIssue::new("Q1099_ROTH_ROLLOVER_LOSS").at_field("earnings"));
        }
        return None;
    }
    match &v.distribution_code {
        Some(Code::ExcessThisYear | Code::ExcessLastYear | Code::ProhibitedTransaction) if qtp => {
            Some(ValidationIssue::new("Q1099_CODE_NOT_COVERDELL").at_field("distribution_code"))
        }
        Some(Code::ExcessThisYear | Code::ExcessLastYear) if boxes.is_none() => {
            Some(ValidationIssue::new("Q1099_EXCESS_NEEDS_BOXES").at_field("fair_market_value"))
        }
        _ => None,
    }
}
