// form_1098_e items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct EligibleLoanIn<'a> {
    pub(super) education: &'a crate::education_account::EducationTotals,
}

pub(super) fn eligible_loan_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    i: EligibleLoanIn<'_>,
) -> i64 {
    let &Pass { a, allocated_entries, .. } = pass;
    let EligibleLoanIn { education } = i;
    let health_loan_contexts: Vec<_> = allocated_entries
        .iter()
        .filter_map(|entry| match &entry.income {
            FederalIncome::HealthLoanAssistance { data } if data.interest > 0 => Some(
                crate::interview::income_issue_context(entry)
                    .corrective(crate::TaxSection::StudentLoan),
            ),
            _ => None,
        })
        .collect();
    for context in &health_loan_contexts {
        crate::require_localized!(
            r,
            t.health_loan_interest <= a.student_loan_interest(),
            context,
            "Health-service loan assistance",
            "ISSUE_PROGRAM_PAID_INTEREST_OVERLAP_CANNOT_EXCEED_THE_TOTAL_INTEREST_ENTE_F86D91E8",
            "Program-paid interest overlap cannot exceed the total interest entered in Adjustments. Reconcile the servicer and program entries without double subtraction.",
        );
    }
    // Section 221(e)(1): interest excluded under section 127(c)(1)(B) is
    // not deductible either; it is removed before the cap and phase-out.
    let employer_loan_interest = a.student_loan_employer_paid();
    for context in health_loan_contexts.iter().chain(
        (health_loan_contexts.is_empty())
            .then_some(crate::IssueContext::Section(crate::TaxSection::StudentLoan))
            .as_ref(),
    ) {
        crate::require_localized!(
            r,
            employer_loan_interest == 0
                || t.health_loan_interest + employer_loan_interest <= a.student_loan_interest(),
            context,
            "Student loan interest",
            "ISSUE_HEALTH_SERVICE_PROGRAM_PAID_INTEREST_AND_INTEREST_YOUR_EMPLOYER_PAI_CDAF5E31",
            "Health-service program-paid interest and interest your employer paid under an educational assistance program together cannot exceed the student loan interest entered. Reconcile the entries so no payment is subtracted twice.",
        );
    }
    // IRC 221(e)(1): interest paid with tax-free 529 earnings is not deducted.
    let eligible_loan_interest = phase::floor(
        a.student_loan_interest()
            - t.health_loan_interest
            - employer_loan_interest
            - education.loan_excluded,
        0,
    );
    if education.loan_excluded > 0 {
        line_cents(
            r,
            "education_account_loan_interest_removed",
            education.loan_excluded,
            &["income.education_distribution", "adjustments.student_loan"],
            "https://www.law.cornell.edu/uscode/text/26/221",
            msg!("EXPLAIN_EDUCATION_ACCOUNT_LOAN_INTEREST_REMOVED"),
        );
    }
    line_cents(
        r,
        "health_loan_interest_removed",
        t.health_loan_interest,
        &["income.health_loan_assistance", "adjustments.student_loan"],
        "https://www.irs.gov/publications/p970",
        msg!("EXPLAIN_HEALTH_LOAN_INTEREST_REMOVED"),
    );
    line_cents(
        r,
        "employer_loan_interest_removed",
        employer_loan_interest,
        &["adjustments.student_loan"],
        "https://www.irs.gov/publications/p970",
        msg!("EXPLAIN_EMPLOYER_LOAN_INTEREST_REMOVED"),
    );
    eligible_loan_interest
}
