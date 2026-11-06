// educator items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

impl Interview {
    // The educator expenses page, opened with its complete value: one row
    // per claimant, or an empty list (M3-P8).
    pub(super) fn educator_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        // A row per claimant with an amount; zero is no claim.
        let claim = d.adjustments.as_ref().map_or(EducatorClaim::NONE, |a| a.educators.clone());
        let people = [(Owner::Taxpayer, claim.taxpayer), (Owner::Spouse, claim.spouse)]
            .into_iter()
            .filter_map(|(owner, role)| role.map(|role| Educator { owner, role }))
            .collect();
        Reply::Edit(P::SaveEducator {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: ADJUSTMENTS_PAGE,
            previous: ADJUSTMENTS_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::Educator),
            warnings: Vec::new(),
            progress: self.topic_progress(ADJUSTMENTS_PAGE, Some(TaxSection::Educator)),
            allowed_owners: self.allowed_owners(),
            value: EducatorExpenses { people },
        })
    }
    // The educator expenses page's rows merged into the accepted return
    // (M3-P8): an empty page is No, matching the design's list-based
    // (not amount-based) confirmation for this topic.
    pub(super) fn educator_candidate(
        &self,
        value: &EducatorExpenses,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        let d = &self.draft;
        let owners = self.allowed_owners();
        if value.people.iter().any(|p| !owners.contains(&(p.owner as i32))) {
            return Err(crate::interview::status(
                "MESSAGE_YOUR_SPOUSE_S_EXPENSES_BELONG_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
            )
            .into());
        }
        if value.people.len() == 2 && value.people[0].owner == value.people[1].owner {
            return Err(ValidationIssue::new("EDUCATOR_TWICE")
                .at_field("owner")
                .in_list("people", 1)
                .in_field("value")
                .into());
        }
        let mut draft = d.clone();
        if let Some(adjustments) = draft.adjustments.as_mut() {
            let role =
                |owner| value.people.iter().find(|p| p.owner == owner).map(|p| p.role.clone());
            adjustments.educators =
                EducatorClaim { taxpayer: role(Owner::Taxpayer), spouse: role(Owner::Spouse) };
        }
        // A walked topic saved is visited: the walk passes it (M3-P19).
        visit(&self.draft, &mut draft, TaxSection::Adjustments, TaxSection::Educator);
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_educator(
        &self,
        value: &EducatorExpenses,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != ADJUSTMENTS_PAGE || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.educator_candidate(value)
    }
}
