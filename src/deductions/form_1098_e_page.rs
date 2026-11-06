// form_1098_e items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

impl Interview {
    // The student loan interest page, opened with its complete value: the
    // stored Forms 1098-E with their box 2 amounts and the interest paid
    // with no form (M3-P8; T2-33x).
    pub(super) fn student_loan_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let claim =
            d.adjustments.as_ref().map_or(StudentLoanClaim::NONE, |a| a.student_loan.clone());
        let (forms, other_interest, employer_paid_interest) =
            (claim.forms, claim.student_loan_interest, claim.employer_paid_interest);
        Reply::Edit(P::SaveStudentLoan {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: ADJUSTMENTS_PAGE,
            previous: ADJUSTMENTS_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::StudentLoan),
            warnings: Vec::new(),
            progress: self.topic_progress(ADJUSTMENTS_PAGE, Some(TaxSection::StudentLoan)),
            document_facts: DocumentFacts { year: 2026 },
            value: StudentLoanInterest {
                forms: forms.iter().map(Into::into).collect(),
                other_interest,
                employer_paid_interest,
            },
        })
    }
    pub(super) fn student_loan_candidate(
        &self,
        value: &StudentLoanInterest,
    ) -> Result<FederalDraft, SaveFailure> {
        let mut forms =
            self.draft.adjustments.as_ref().map_or_else(Vec::new, |a| a.student_loan.forms.clone());
        Self::document_answer_ids(
            &forms.iter().map(|form| form.id).collect::<Vec<_>>(),
            &value.forms.iter().map(|form| form.id).collect::<Vec<_>>(),
            "forms",
        )?;
        for (index, row) in value.forms.iter().enumerate() {
            let form = forms.iter_mut().find(|form| form.id == row.id).unwrap();
            match (form.document.box_2, &row.additional_interest) {
                (false, crate::StudentLoanAdditionalInterest::NotNeeded) => {}
                (
                    true,
                    crate::StudentLoanAdditionalInterest::Checked { additional_interest: answer },
                ) if (0..=100000000000).contains(answer) => form.additional_interest = *answer,
                _ => {
                    return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                        .at_field("additional_interest")
                        .in_list("forms", index)
                        .in_field("value")
                        .into());
                }
            }
        }
        self.student_loan_records_candidate(
            &forms,
            value.other_interest,
            value.employer_paid_interest,
            true,
        )
    }
    pub(super) fn student_loan_sources_candidate(
        &self,
        forms: &[crate::StudentLoanDocument],
        other_interest: i64,
        employer_paid_interest: i64,
    ) -> Result<FederalDraft, SaveFailure> {
        // The accepted answer stays with its form; a new or unchecked form has none.
        let accepted =
            self.draft.adjustments.as_ref().map_or(&[][..], |a| &a.student_loan.forms[..]);
        let sources = forms
            .iter()
            .map(|document| {
                let mut source =
                    crate::StudentLoanSource::printed(document.id, document.form.clone());
                if document.form.box_2
                    && let Some(old) =
                        accepted.iter().find(|old| document.id != 0 && old.id == document.id)
                {
                    source.additional_interest = old.additional_interest;
                }
                source
            })
            .collect::<Vec<_>>();
        self.student_loan_records_candidate(&sources, other_interest, employer_paid_interest, false)
    }
    pub(super) fn student_loan_records_candidate(
        &self,
        forms: &[crate::StudentLoanSource],
        other_interest: i64,
        employer_paid_interest: i64,
        require_answers: bool,
    ) -> Result<FederalDraft, SaveFailure> {
        let d = &self.draft;
        if require_answers {
            // Employer-paid interest cannot exceed the topic's qualified interest.
            let interest = other_interest
                + forms.iter().map(|source| source.qualified_interest()).sum::<i64>();
            if employer_paid_interest > interest {
                return Err(ValidationIssue::new("STUDENT_LOAN_EMPLOYER_ABOVE_INTEREST")
                    .at_field("employer_paid_interest")
                    .in_field("value")
                    .into());
            }
        }
        let mut draft = d.clone();
        if let Some(adjustments) = draft.adjustments.as_mut() {
            adjustments.student_loan = StudentLoanClaim {
                student_loan_interest: other_interest,
                forms: forms.to_vec(),
                employer_paid_interest,
            };
        }
        // A walked topic saved is visited: the walk passes it (M3-P19).
        visit(&self.draft, &mut draft, TaxSection::Adjustments, TaxSection::StudentLoan);
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_student_loan(
        &self,
        value: &StudentLoanInterest,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != ADJUSTMENTS_PAGE || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.student_loan_candidate(value)
    }
}
