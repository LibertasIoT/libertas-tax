// injured_spouse items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

impl Interview {
    // The injured spouse allocation page (T3-8379): the stored answers, or the
    // defaults the first time; the joint payments and the estimated share are
    // derived from the accepted return. Back returns to the last shown
    // section.
    pub(super) fn injured_spouse_page(&self, review: bool) -> Reply {
        self.injured_spouse_page_for(review, &self.draft)
    }
    pub(super) fn injured_spouse_page_for(&self, review: bool, d: &FederalDraft) -> Reply {
        let mut stored = d
            .injured_spouse
            .clone()
            .unwrap_or_else(crate::injured_spouse::InjuredSpouseAllocation::defaults);
        stored.claimants.retain(|c| d.dependents.iter().any(|p| p.id == c.person));
        for person in &d.dependents {
            if !stored.claimants.iter().any(|c| c.person == person.id) {
                stored.claimants.push(crate::InjuredClaimantFacts {
                    person: person.id,
                    claimant: Owner::Taxpayer,
                });
            }
        }
        let mut calculating = d.clone();
        calculating.injured_spouse = Some(stored.clone());
        let result = estimate(&calculating);
        Reply::Response(P::BeginInjuredSpouse {
            followups: Self::injured_spouse_followups_for(d, &result),
            questions: Vec::new(),
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: INJURED_SPOUSE_PAGE,
            previous: self.previous_shown(TaxSection::InjuredSpouse),
            review,
            estimate: self.estimate_now(TaxSection::InjuredSpouse).map(Into::into),
            warnings: Vec::new(),
            progress: self.progress(INJURED_SPOUSE_PAGE),
            value: stored.construction(d, &result),
        })
    }
    // The injured spouse page (T3-8379) onto FederalDraft.injured_spouse; the
    // derived joint payments and share are ignored, and the agreed part is
    // checked against the return's own payments (the client rule's twin).
    pub(super) fn injured_spouse_candidate(
        &self,
        value: &crate::InjuredSpouse,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        // Unrelated generic admission errors remain explicitly unlocated.
        let status =
            |key: &str| crate::validation::SaveFailure::from(crate::interview::status(key));
        if !injured_chosen(&self.draft) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET"));
        }
        if let Some(issue) = crate::injured_spouse::issue(
            value,
            &crate::injured_spouse::joint_estimated(&self.draft),
        ) {
            return Err(issue.in_field("value").into());
        }
        let mut draft = self.draft.clone();
        draft.injured_spouse =
            Some(crate::injured_spouse::InjuredSpouseAllocation::from_page(value));
        let result = estimate(&draft);
        if let Some(issue) = crate::injured_spouse::allocation_issue(value, &draft, &result) {
            return Err(issue.in_field("value").into());
        }
        Ok(draft)
    }
}
