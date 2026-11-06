//! Source-only acceptance of Form 1099-K split answers.
//!
//! The ordinary linked composer submits these complete answers independently
//! of the containing business editor. Only accepted component identities are
//! returned to that editor; its unfinished values never enter this candidate.
use super::*;

impl Interview {
    pub(super) fn payment_parts_candidate(
        &self,
        key: &str,
        value: Vec<crate::KPart>,
    ) -> Result<(FederalDraft, i64), SaveFailure> {
        let unavailable = || SaveFailure::from(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED"));
        // The accepted question catalog owns eligibility and source identity.
        // Never parse a caller-supplied numeric identity into authorization.
        let source =
            Self::document_followup_rows(&self.draft, Some(TaxSection::SelfEmployment), None)
                .into_iter()
                .find_map(|(id, question)| {
                    (question.key == key
                        && matches!(question.answer, crate::DocumentAnswer::PaymentParts(..)))
                    .then_some(id)
                })
                .ok_or_else(unavailable)?;
        let mut candidate = self.draft.clone();
        let entry =
            candidate.income.iter_mut().find(|entry| entry.id == source).ok_or_else(unavailable)?;
        if entry.owner.tax_owner().is_none() {
            return Err(unavailable());
        }
        let FederalIncome::K1099 { data } = &mut entry.income else {
            return Err(unavailable());
        };
        data.parts = value;
        related_forms::prepare_component_ids(
            core::slice::from_mut(entry),
            &self.draft.income,
            &mut candidate.next_id,
        )
        .map_err(|(_, issue)| SaveFailure::from(issue.map_field_prefix(&["parts"], &["value"])))?;
        let FederalIncome::K1099 { data } = &entry.income else { unreachable!() };
        if let Some(issue) =
            crate::information_returns::k_input_issue(&crate::Form1099K::from(data), true)
        {
            return Err(issue.map_field_prefix(&["parts"], &["value"]).into());
        }
        Ok((candidate, source))
    }

    pub(super) fn save_payment_parts(&mut self, key: &str, value: Vec<crate::KPart>) -> Reply {
        let (candidate, source) = match self.payment_parts_candidate(key, value) {
            Ok(candidate) => candidate,
            Err(failure) => return refused(failure),
        };
        // Normal commit reconciliation preserves this printed source's answers.
        // Capture the already allocated parts before accepting the transaction,
        // so every possible refusal occurs before any state change.
        let FederalIncome::K1099 { data } = &candidate
            .income
            .iter()
            .find(|entry| entry.id == source)
            .expect("validated accepted source")
            .income
        else {
            unreachable!("validated Form 1099-K source")
        };
        let value = data.parts.clone();
        if let Err((issue, _)) = self.commit(candidate) {
            return problem_key(issue);
        }
        Reply::Response(P::PaymentPartsSaved { value, revision: self.draft.revision })
    }
}
