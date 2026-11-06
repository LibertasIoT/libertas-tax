// form_1098 items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// Each document keeps its identity; transferred forms share only the first
// lender's year-wide ledger. Page projection preserves every stored fact.
// Native loan metadata projects through the exact existing Form1098 conversion.
// MortgageDetails/Points/refund-origin variants are transparent in field paths.
pub(super) fn mortgage_page_issue(issue: ValidationIssue) -> ValidationIssue {
    issue
        .map_field_prefix(&["outstanding_principal"], &["value", "box_2"])
        .map_field_prefix(&["incurred"], &["details"])
        .map_field_prefix(&["refund_of_interest", "origin"], &["refund_origin"])
}

pub(super) fn mortgage_answer_path(
    draft: &FederalDraft,
    id: i64,
    key: &'static str,
) -> Option<String> {
    let loans = &draft.deductions.as_ref()?.mortgage;
    let index = loans.iter().position(|loan| loan.id == id)?;
    if key == "DOCUMENT_MORTGAGE_REFUND_ORIGIN" {
        // Asked on the mortgage questions page, never on the loan's own row.
        return ValidationIssue::new(key)
            .at_field("answer")
            .in_key("questions", alloc::format!("mortgage/{id}").as_str())
            .field
            .map(|field| field.square());
    }
    let issue = loans[index].conversion_issue().filter(|issue| issue.key == key)?;
    mortgage_page_issue(issue)
        .in_key("loans", id)
        .in_field("value")
        .field
        .map(|field| field.square())
}

pub(super) fn mortgage_average_balance_needed(loans: &[MortgageLoan], separate: bool) -> bool {
    let loans: Vec<_> = loans
        .iter()
        .filter_map(|loan| {
            let MortgageLoanDetails::FirstLender(facts) = &loan.details else {
                return None;
            };
            (facts.home != MortgageHome::NotQualified
                && !matches!(facts.purpose, MortgageUse::Other))
            .then_some((loan, facts))
        })
        .collect();
    if !loans.iter().any(|(_, f)| f.average == MortgageAverageBalance::WithinLimit) {
        return false;
    }
    let all_earlier = loans.iter().all(|(l, f)| match &f.purpose {
        MortgageUse::Home => l.incurred <= crate::year_constants::MORTGAGE_EARLIER_LIMIT_LAST_DATE,
        MortgageUse::Partly(p) => {
            p.improvement_incurred <= crate::year_constants::MORTGAGE_EARLIER_LIMIT_LAST_DATE
        }
        MortgageUse::Refinance(r) => {
            r.earlier_incurred <= crate::year_constants::MORTGAGE_EARLIER_LIMIT_LAST_DATE
                && (l.incurred <= crate::year_constants::MORTGAGE_EARLIER_LIMIT_LAST_DATE
                    || r.earlier_term_ends
                        .is_some_and(|end| end >= crate::year_constants::TAX_YEAR_END_DATE))
                && (r.improvement == 0
                    || l.incurred <= crate::year_constants::MORTGAGE_EARLIER_LIMIT_LAST_DATE)
        }
        MortgageUse::Other => true,
    });
    let limit = match (all_earlier, separate) {
        (true, true) => crate::year_constants::MORTGAGE_DEBT_LIMIT_GRANDFATHERED_SEPARATE_CENTS,
        (true, false) => crate::year_constants::MORTGAGE_DEBT_LIMIT_GRANDFATHERED_CENTS,
        (false, true) => crate::year_constants::MORTGAGE_DEBT_LIMIT_SEPARATE_CENTS,
        (false, false) => crate::year_constants::MORTGAGE_DEBT_LIMIT_CENTS,
    };
    // Arithmetic safety: any Vec<i64> admitted on a 32/64-bit target has an
    // exactly representable i128 sum; compare the statutory debt limit in that unit.
    loans.iter().map(|(loan, _)| i128::from(loan.outstanding_principal)).sum::<i128>()
        > i128::from(limit)
}
pub(super) fn mortgage_available(d: &FederalDraft) -> bool {
    d.deductions.is_some()
}
pub(super) fn mortgage_parent(d: &FederalDraft) -> i32 {
    if itemizing(d) { ITEMIZED_PAGE } else { DEDUCTIONS_PAGE }
}
// A stored loan keeping the page's WithinLimit default after a later answer
// (a separate return halves the limit) put the box 2 balances above it: the
// mortgage page's own refusal, MORTGAGE_AVERAGE_NEEDED, is then a remaining
// item.
pub(super) fn mortgage_method_needed(d: &FederalDraft) -> bool {
    let separate = d.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Separate);
    d.deductions.as_ref().is_some_and(|facts| {
        mortgage_average_balance_needed(&facts.mortgage_loans().unwrap_or_default(), separate)
    })
}

impl Interview {
    pub(super) fn mortgage_choices(&self) -> Vec<RecordChoice> {
        estate_followups::mortgage_choices(&self.draft)
            .into_iter()
            .map(|row| RecordChoice { id: row.id, label: row.label })
            .collect()
    }
    pub(super) fn mortgage_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let Some(deductions) = d.deductions.as_ref() else {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
        };
        let parent = mortgage_parent(d);
        let loans: Vec<crate::MortgageDocumentView> =
            deductions.mortgage.iter().map(crate::MortgageDocumentView::from).collect();
        // A loan whose facts this page has not answered yet: the page opens by
        // WorkflowRequest (BeginMortgages) with the answered loans as saved
        // and the new ones unanswered beyond what their paper settles.
        if deductions.mortgage.iter().any(|source| !source.answered) {
            let (answered, new): (Vec<_>, Vec<_>) = loans.into_iter().partition(|row| {
                deductions.mortgage.iter().any(|source| source.id == row.id && source.answered)
            });
            return Reply::Response(P::BeginMortgages {
                cookie: d.cookie.clone(),
                revision: d.revision,
                page: parent,
                previous: parent,
                review,
                estimate: self.estimate_now(TaxSection::Mortgage).map(Into::into),
                progress: self.topic_progress(parent, Some(TaxSection::Mortgage)),
                document_facts: DocumentFacts { year: 2026 },
                mortgage_loans: self.mortgage_choices(),
                rental_mortgage_loans: crate::extended_rules::rental_mortgage_roots(d),
                value: crate::MortgagesSeed {
                    loans: answered.into_iter().map(Into::into).collect(),
                    new_loans: new.into_iter().map(Into::into).collect(),
                },
            });
        }
        Reply::Edit(P::SaveMortgages {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: parent,
            previous: parent,
            review,
            estimate: self.estimate_now(TaxSection::Mortgage),
            warnings: Vec::new(),
            document_questions: self.mortgage_questions_link(),
            progress: self.topic_progress(parent, Some(TaxSection::Mortgage)),
            document_facts: DocumentFacts { year: 2026 },
            mortgage_loans: self.mortgage_choices(),

            rental_mortgage_loans: crate::extended_rules::rental_mortgage_roots(d),
            value: Mortgages { loans, new_loans: Vec::new() },
        })
    }
    pub(super) fn mortgage_sources_candidate(
        &self,
        sources: &[Mortgage1098Source],
    ) -> Result<FederalDraft, SaveFailure> {
        self.mortgage_loans_candidate(sources, false)
    }
    pub(super) fn mortgage_loans_candidate(
        &self,
        sources: &[Mortgage1098Source],
        require_answers: bool,
    ) -> Result<FederalDraft, SaveFailure> {
        let Some(accepted) = self.draft.deductions.as_ref() else {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        };
        let choices = self.mortgage_choices();
        for (index, source) in sources.iter().enumerate() {
            let loan = MortgageLoan::try_from(source);
            if require_answers
                && matches!(&loan, Err(crate::Mortgage1098ConversionError::PaidInterestAboveBox1))
                && let Some(issue) = source.conversion_issue()
            {
                return Err(issue.in_list("loans", index).in_field("value").into());
            }
            if require_answers && source.form.box_4 > 0 && source.refund_origin.is_none() {
                return Err(ValidationIssue::new("MORTGAGE_REFUND_ORIGIN_NEEDED")
                    .at_field("refund_origin")
                    .in_list("loans", index)
                    .in_field("value")
                    .into());
            }
            if let MortgageDetails::EarlierLender { loan: target } = source.details
                && !choices.iter().any(|c| c.id == target)
            {
                return Err(ValidationIssue::new("MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED")
                    .at_field("loan")
                    .in_field("details")
                    .in_list("loans", index)
                    .in_field("value")
                    .into());
            }
            if let Ok(loan) = loan
                && let Some(issue) = loan.issue()
            {
                return Err(mortgage_page_issue(issue)
                    .in_list("loans", index)
                    .in_field("value")
                    .into());
            }
        }
        let separate =
            self.draft.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Separate);
        let mut draft = self.draft.clone();
        let mut sources = sources.to_vec();
        assign_record_ids(
            sources.iter_mut().map(|source| &mut source.id),
            accepted.mortgage.iter().map(|source| source.id),
            &mut draft.next_id,
        )
        .map_err(|(key, _, index)| {
            ValidationIssue::new(key).at_field("id").in_list("loans", index).in_field("value")
        })?;
        let records: Vec<MortgageLoan> =
            sources.iter().filter_map(|source| MortgageLoan::try_from(source).ok()).collect();
        if require_answers && mortgage_average_balance_needed(&records, separate) {
            return Err(ValidationIssue::new("MORTGAGE_AVERAGE_NEEDED")
                .at_field("loans")
                .in_field("value")
                .into());
        }
        if let Some((index, issue)) = crate::mortgage_limit::graph_issue(&records) {
            return Err(mortgage_page_issue(issue)
                .in_list("loans", index)
                .in_field("value")
                .into());
        }
        if records.iter().any(|r| !r.valid()) {
            return Err(status("MORTGAGE_LOAN_ID_INVALID").into());
        }
        // A rental's accepted source link cannot be erased by replacing its lender catalog.
        if crate::extended_rules::rental_mortgage_roots(&self.draft).iter().any(|id| {
            self.draft.income.iter().any(|entry| {
                matches!(&entry.income,
                FederalIncome::Rental { data } if data.related.contains(id))
            }) && !records.iter().any(|loan| {
                loan.id == *id && matches!(loan.details, crate::MortgageLoanDetails::FirstLender(_))
            })
        }) {
            return Err(ValidationIssue::new("MORTGAGE_RENTAL_ROOT_NEEDED")
                .at_field("loans")
                .in_field("value")
                .into());
        }
        draft.deductions.as_mut().unwrap().mortgage = sources;
        if let Some((_, issue)) = crate::extended_rules::rental_mortgage_issue(&draft) {
            return Err(issue.at_field("loans").in_field("value").into());
        }
        finish_section(&mut draft, TaxSection::Mortgage);
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_mortgages(
        &self,
        value: &Mortgages,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != mortgage_parent(&self.draft)
            || !self.reached(MORTGAGE_PAGE)
            || !mortgage_available(&self.draft)
        {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.mortgages_candidate(value)
    }
}
