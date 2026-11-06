use crate::tests::*;

fn student_loan_paper_native(form: Form1098E) -> Form1098EPaper {
    let Form1098E { lender, borrower, interest, box_2 } = form;
    Form1098EPaper {
        lender,
        borrower,
        box_1: interest,
        box_2,
        calendar_year: 2026,
        ..Default::default()
    }
}

paper_adapter!(
    student_loan_paper,
    student_loan_paper_native,
    StudentLoanBoxFixture,
    Form1098E,
    Form1098EPaper
);

pub(in crate::tests) fn loan_claim(
    forms: Vec<crate::StudentLoanSource>,
    no_form: i64,
) -> StudentLoanClaim {
    StudentLoanClaim { student_loan_interest: no_form, forms, employer_paid_interest: 0 }
}

pub(in crate::tests) fn box_2(interest: i64, additional_interest: i64) -> crate::StudentLoanSource {
    crate::StudentLoanSource {
        additional_interest,
        // No printed borrower: the loan is yours, as your section added it.
        assigned: crate::DocumentRecipient::Taxpayer,
        owner_source: Some(crate::OwnerSource::Added),
        ..crate::StudentLoanSource::printed(
            0,
            student_loan_paper(Form1098E { interest, box_2: true, ..Default::default() }),
        )
    }
}

pub(in crate::tests) fn student_documents(
    forms: &[crate::StudentLoanSource],
) -> Vec<crate::StudentLoanDocument> {
    forms
        .iter()
        .map(|source| crate::StudentLoanDocument { id: source.id, form: source.document.clone() })
        .collect()
}

pub(in crate::tests) fn employer_paid(
    mut claim: StudentLoanClaim,
    amount: i64,
) -> StudentLoanClaim {
    claim.employer_paid_interest = amount;
    claim
}

pub(in crate::tests) fn box_1(interest: i64) -> crate::StudentLoanSource {
    crate::StudentLoanSource {
        assigned: crate::DocumentRecipient::Taxpayer,
        owner_source: Some(crate::OwnerSource::Added),
        ..crate::StudentLoanSource::printed(
            0,
            student_loan_paper(Form1098E { interest, ..Default::default() }),
        )
    }
}
