use super::*;
use crate::{W2AnswerRow, WagesDocumentQuestion, WagesRecordItem, WagesRecords};

impl Interview {
    pub(super) fn wages_answers_page(
        &self,
        review: bool,
        supplied: Option<&FederalResult>,
    ) -> Reply {
        let owned;
        let result = match supplied {
            Some(result) => result,
            None => {
                owned = self.income_result();
                &owned
            }
        };
        let Reply::Edit(P::SaveIncomes {
            user,
            cookie,
            revision,
            previous,
            review,
            estimate,
            warnings,
            progress,
            businesses,
            wage_scorporations,
            allowed_wage_scorporations,
            allowed_owners,
            allowed_community,
            allowed_refund_management,
            documents,
            ..
        }) = self.topic_page_with_result(TaxSection::Wages, review, Some(result))
        else {
            return problem_key("PROBLEM_UNDISPLAYABLE_ANSWER");
        };
        let Some(documents) = documents
            .documents
            .into_iter()
            .map(WagesRecordItem::from_income)
            .collect::<Option<Vec<_>>>()
        else {
            return problem_key("PROBLEM_UNDISPLAYABLE_ANSWER");
        };
        let mut wages = Vec::new();
        for (id, row) in
            Self::document_followup_rows(&self.draft, Some(TaxSection::Wages), Some(result))
        {
            let crate::DocumentAnswer::Wage { answers } = row.answer else { continue };
            // A W-2 without accepted printed parties is still named by its kind.
            let Some(entry) = self.draft.income.iter().find(|e| e.id == id) else {
                return problem_key("PROBLEM_UNDISPLAYABLE_ANSWER");
            };
            let document = document_followups::followup_document_name(&self.draft, entry);
            wages.push(W2AnswerRow { key: row.key, document, answers });
        }
        Reply::Edit(P::SaveWagesAnswers {
            user,
            cookie,
            revision,
            page: WAGES_PAGE,
            previous,
            review,
            estimate,
            warnings,
            progress,
            businesses,
            wage_scorporations,
            allowed_wage_scorporations,
            allowed_owners,
            allowed_community,
            allowed_refund_management,
            wages,
            followups: self.wages_followups(),
            // Regrouped for the person at the protocol boundary (followup_page).
            questions: Vec::new(),
            situations: crate::DocumentSituations::NoneOfThese,
            allowed_situations: Vec::new(),
            situation_answers: Vec::new(),
            situation_documents: Vec::new(),
            disaster_choices: Vec::new(),
            people: Vec::new(),
            inherited_iras: Vec::new(),
            inherited_roth_iras: Vec::new(),
            records: WagesRecords { documents },
        })
    }

    pub(super) fn wages_answers_candidate(
        &self,
        wages: Vec<W2AnswerRow>,
        followups: Vec<WagesDocumentQuestion>,
        records: WagesRecords,
        require_answers: bool,
        origins: Option<&mut Vec<(i64, usize, bool)>>,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        let result = self.income_result();
        let mut expected =
            Self::document_followup_rows(&self.draft, Some(TaxSection::Wages), Some(&result));
        let w2_count = expected
            .iter()
            .filter(|(_, row)| matches!(row.answer, crate::DocumentAnswer::Wage { .. }))
            .count();
        let changed = || ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED");
        if wages.len() != w2_count {
            return Err(changed().at_field("wages").into());
        }
        if followups.len() != expected.len() - w2_count {
            return Err(changed().at_field("followups").into());
        }
        let mut followup_keys = alloc::collections::BTreeSet::new();
        for row in &followups {
            if !followup_keys.insert(row.key.as_str()) {
                return Err(changed().at_field("key").in_key("followups", row.key.as_str()).into());
            }
        }
        let mut seen = alloc::collections::BTreeSet::new();
        let mut targets = Vec::new();
        let mut is_w2 = Vec::new();
        for (_, row) in &mut expected {
            let w2 = matches!(row.answer, crate::DocumentAnswer::Wage { .. });
            let target = if w2 {
                let Some(supplied) = wages.iter().find(|supplied| supplied.key == row.key) else {
                    return Err(changed().at_field("wages").into());
                };
                if !seen.insert(supplied.key.clone()) {
                    return Err(changed()
                        .at_field("key")
                        .in_key("wages", supplied.key.as_str())
                        .into());
                }
                row.answer = crate::DocumentAnswer::Wage { answers: supplied.answers.clone() };
                changed().at_record().in_key("wages", supplied.key.as_str()).field
            } else {
                let Some(supplied) = followups.iter().find(|supplied| supplied.key == row.key)
                else {
                    return Err(changed().at_field("followups").into());
                };
                *row = supplied.clone().into_canonical(row.clone()).ok_or_else(|| {
                    changed().at_field("answer").in_key("followups", supplied.key.as_str())
                })?;
                changed().at_record().in_key("followups", supplied.key.as_str()).field
            };
            targets.push(target);
            is_w2.push(w2);
        }
        let keys: Vec<_> = expected.iter().map(|(_, row)| row.key.clone()).collect();
        let records: Vec<_> =
            records.documents.into_iter().map(WagesRecordItem::into_income).collect();
        let candidate = self
            .followup_candidate_indexed(
                TaxSection::Wages,
                expected.into_iter().map(|(_, row)| row).collect(),
                require_answers,
                &records,
            )
            .map_err(|failure| {
                failure
                    .map_list_item_field_prefix(&["followups"], &["answer"], &[], &is_w2)
                    .map_submitted_items(&["followups"], &targets)
            })?;
        let Reply::Edit(P::SaveIncomes { documents, .. }) =
            self.topic_page(TaxSection::Wages, false)
        else {
            return Err(changed().at_field("records").into());
        };
        candidate
            .documents_candidate_inner_with_origins(
                TaxSection::Wages,
                IncomeEdits {
                    documents: records.into_iter().map(IncomeRecordItem::into_income).collect(),
                    qbi_aggregations: documents.qbi_aggregations,
                },
                !require_answers,
                origins,
                Some(&keys),
            )
            .map_err(|failure| {
                failure.map_field_prefix(&["documents", "documents"], &["records", "documents"])
            })
    }
}
