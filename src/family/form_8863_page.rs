// form_8863 items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// The people an education credit can be for: you, your spouse on a joint
// return and the people you support, the medical page's catalog without ages.
pub(super) fn education_people(d: &FederalDraft) -> Vec<PersonChoice> {
    medical_people(d).into_iter().map(|(p, _)| p).collect()
}
pub(super) fn student_credit_value(record: &EducationSource) -> StudentCreditDocument {
    StudentCreditDocument { id: record.id, student_id: record.student, form: record.form.clone() }
}
// A filer or a person you support who is a student: the credits hub suggests
// the education topic.
pub(super) fn student_on_return(d: &FederalDraft) -> bool {
    d.people.as_ref().is_some_and(|p| {
        p.taxpayer.full_time_student || p.spouse.as_ref().is_some_and(|s| s.full_time_student)
    }) || d.dependents().iter().any(|v| v.student())
}

// The student key keeps a locator attached to its annual answer after row reordering.
pub(super) fn education_answer_path(
    draft: &FederalDraft,
    student: i64,
    key: &'static str,
) -> Option<String> {
    let credits = draft.credits.as_ref()?;
    let index = credits.students.iter().position(|row| row.student == student)?;
    let issue = match key {
        "EDUCATION_METHOD_NEEDED" => ValidationIssue::new(key).at_field("method"),
        "EDUCATION_AOTC_HISTORY_NEEDED" => {
            ValidationIssue::new(key).at_field("american_opportunity_used_up").in_field("more")
        }
        _ if credits.education_insurance_refund_issue(&credits.students[index]) == Some(key) => {
            ValidationIssue::new(key).at_field("insurance_refund_current_expenses").in_field("more")
        }
        _ => return None,
    };
    issue.in_key("students", student).in_field("value").field.map(|field| field.square())
}

impl Interview {
    // The education credits page, opened with its complete value: one row
    // per stored student, or an empty list (M3-P11).
    pub(super) fn education_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let people = education_people(d);
        Reply::Edit(P::SaveEducation {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: CREDITS_PAGE,
            previous: CREDITS_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::Education),
            warnings: Vec::new(),
            progress: self.topic_progress(CREDITS_PAGE, Some(TaxSection::Education)),
            document_facts: DocumentFacts { year: 2026 },
            people: people.clone(),
            followups: self.education_followups(),
            questions: Vec::new(),
            value: crate::EducationDocumentView {
                documents: d.credits.as_ref().map_or_else(Vec::new, |credits| {
                    credits.education.iter().map(StudentCreditView::from).collect()
                }),
                students: d.credits.as_ref().map_or_else(Vec::new, |c| {
                    c.students.iter().map(|row| c.education_student_page(row)).collect()
                }),
                people,
            },
        })
    }
    // The other taxes section's page (u5-credits), opened with its complete
    // value: one list item per record kind it owns and its refusal kinds, or
    // an empty list.
    pub(super) fn coverdell_corrections(&self) -> Vec<RecordChoice> {
        self.draft
            .income
            .iter()
            .filter(|e| crate::coverdell::correction_source(e))
            .map(|entry| RecordChoice {
                id: entry.id,
                label: crate::Message::new(
                    "COVERDELL_SOURCE_LABEL",
                    alloc::vec![crate::MessageArg::Text(entry.label.clone())],
                )
                .encode(),
            })
            .collect()
    }
    // The printed statements keep their accepted identities; ownership is
    // assigned by the shared matcher after the complete Document Center batch.
    pub(super) fn education_sources_candidate(
        &self,
        value: &EducationCredits,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        let people = education_people(&self.draft);
        if value.people != people.to_vec() {
            return Err(crate::interview::status(
                "MESSAGE_THE_PEOPLE_ON_THIS_RETURN_CHANGED_REOPEN_THIS_PAGE_BEFORE_SAVING",
            )
            .into());
        }
        let mut draft = self.draft.clone();
        let Some(credits) = draft.credits.as_mut() else {
            return Err(crate::interview::status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        };
        let accepted = core::mem::take(&mut credits.education);
        credits.education = value
            .students
            .iter()
            .map(|document| {
                // Existing assignment stays with its accepted identity until the
                // shared printed-name matcher runs after the complete batch.
                let mut source = accepted
                    .iter()
                    .find(|old| document.id != 0 && old.id == document.id)
                    .cloned()
                    .unwrap_or_else(|| EducationSource::printed(document.clone()));
                source.form = document.form.clone();
                source
            })
            .collect();
        credits.reconcile_education_students();
        Ok(draft)
    }
    pub(super) fn education_candidate(
        &self,
        value: &crate::EducationDocumentView,
    ) -> Result<FederalDraft, SaveFailure> {
        self.education_candidate_from(self.draft.clone(), value)
    }

    fn education_candidate_from(
        &self,
        mut draft: FederalDraft,
        value: &crate::EducationDocumentView,
    ) -> Result<FederalDraft, SaveFailure> {
        let people = education_people(&self.draft);
        let previous = self
            .draft
            .credits
            .as_ref()
            .ok_or_else(|| SaveFailure::from(status("PROBLEM_SECTION_UNAVAILABLE_YET")))?;
        let credits = draft
            .credits
            .as_mut()
            .ok_or_else(|| SaveFailure::from(status("PROBLEM_SECTION_UNAVAILABLE_YET")))?;
        if value.students.len() != previous.students.len() {
            return Err(status(
                "MESSAGE_THE_PEOPLE_ON_THIS_RETURN_CHANGED_REOPEN_THIS_PAGE_BEFORE_SAVING",
            )
            .into());
        }
        for (index, row) in value.students.iter().enumerate() {
            if !people.iter().any(|person| person.id == row.student)
                || !previous.students.iter().any(|old| old.student == row.student)
                || value.students[..index].iter().any(|old| old.student == row.student)
            {
                return Err(ValidationIssue::new(
                    "MESSAGE_A_STUDENT_IS_NOT_ON_THIS_RETURN_CHOOSE_THE_STUDENT_AGAIN",
                )
                .at_field("student")
                .in_list("students", index)
                .in_field("value")
                .into());
            }
            // Removing a person's last added statement removes their annual row.
            if !credits.students.iter().any(|old| old.student == row.student) {
                continue;
            }
            if row.method.is_none() {
                return Err(ValidationIssue::new("EDUCATION_METHOD_NEEDED")
                    .at_field("method")
                    .in_list("students", index)
                    .in_field("value")
                    .into());
            }
            if row.more.american_opportunity_used_up.is_none() {
                return Err(ValidationIssue::new("EDUCATION_AOTC_HISTORY_NEEDED")
                    .at_field("american_opportunity_used_up")
                    .in_field("more")
                    .in_list("students", index)
                    .in_field("value")
                    .into());
            }
            let annual = EducationStudent::from(row);
            if let Some(key) = credits.education_insurance_refund_issue(&annual)
                && key == "EDUCATION_INSURANCE_REFUND_ABOVE_BOX_10"
            {
                return Err(ValidationIssue::new(key)
                    .at_field("insurance_refund_current_expenses")
                    .in_field("more")
                    .in_list("students", index)
                    .in_field("value")
                    .into());
            }
            if let ScholarshipInclusion::Amount { included } = row.more.scholarship_in_income {
                let scholarships = credits
                    .education
                    .iter()
                    .filter(|source| source.student == row.student)
                    .try_fold(0_i64, |sum, source| sum.checked_add(source.form.box_5));
                if scholarships.is_none_or(|amount| included > amount) {
                    return Err(ValidationIssue::new("SCHOLARSHIP_INCLUDED_ABOVE_BOX_5")
                        .at_field("included")
                        .in_field("scholarship_in_income")
                        .in_field("more")
                        .in_list("students", index)
                        .in_field("value")
                        .into());
                }
            }
        }
        // Membership and accepted order belong to the retained student list.
        for index in 0..credits.students.len() {
            let student = credits.students[index].student;
            let no_insurance = credits.education_insurance_refund_total(student) == Some(0);
            let old = &mut credits.students[index];
            // A newly assigned student has no annual answers on the submitted
            // page. Its new unanswered row is completed on the next visit.
            let Some(supplied) = value.students.iter().find(|row| row.student == student) else {
                continue;
            };
            old.method = supplied.method;
            old.more = crate::StudentMore::from(&supplied.more);
            if no_insurance {
                old.more.insurance_refund_current_expenses = crate::SourceCents::NotReported;
            }
        }
        finish_section(&mut draft, TaxSection::Education);
        Ok(draft)
    }
}
