// form_8615 items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

impl Interview {
    // The parent's return page (u5-losses): the stored answers, or the
    // defaults the first time. Back returns to the other taxes page, or the
    // section before it when that section is not shown.
    pub(super) fn parent_tax_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let value = d.people.as_ref().and_then(|p| p.parent_tax.as_ref()).map_or_else(
            || ParentReturn {
                foreign_excluded: None,
                status: FederalStatus::Joint,
                taxable: 0,
                gains: None,
                others: Vec::new(),
                special: ParentSituation::Nothing,
            },
            ParentReturn::from_record,
        );
        let previous = if d.finished_sections.contains(&TaxSection::OtherTaxes)
            && !shown_sections(d).contains(&TaxSection::OtherTaxes)
        {
            self.previous_shown(TaxSection::OtherTaxes)
        } else {
            OTHER_TAXES_PAGE
        };
        Reply::Edit(P::SaveParentTax {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: PARENT_TAX_PAGE,
            previous,
            review,
            estimate: self.estimate_now(TaxSection::ParentTax),
            warnings: Vec::new(),
            progress: self.progress(PARENT_TAX_PAGE),
            value,
        })
    }
    // The parent's return page (u5-losses) onto FederalPeople.parent_tax,
    // with the server twins of its client rules and the stored ranges.
    pub(super) fn parent_tax_candidate(
        &self,
        value: &ParentReturn,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        // Unrelated generic admission errors remain explicitly unlocated.
        let status =
            |key: &str| crate::validation::SaveFailure::from(crate::interview::status(key));
        if !parent_tax_in_walk(&self.draft) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET"));
        }
        if let Some(issue) = value.issue() {
            return Err(issue.in_field("value").into());
        }
        let record = value.record();
        if !record.valid() {
            return Err(status(
                "MESSAGE_YOUR_PARENT_S_RETURN_ENTER_THE_AMOUNTS_WITHIN_THE_SUPPORTED_RANGE_EACH_BROTH",
            ));
        }
        let mut draft = self.draft.clone();
        let Some(people) = draft.people.as_mut() else {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET"));
        };
        people.parent_tax = Some(record);
        Ok(draft)
    }
}
// Form 8615 applies (u5-losses): its person tests (federal_rules::young_8615)
// and line 1 over $2,700 as the estimate states it; while the estimate
// cannot state line 1 the page stays in the walk.
pub(crate) fn parent_tax_in_walk(d: &FederalDraft) -> bool {
    d.people.as_ref().is_some_and(|p| crate::federal_rules::young_8615(joint(d), p))
        && crate::federal_rules::form_8615_applies(&estimate(d)) != Some(false)
}

impl Interview {
    pub(super) fn save_parent_tax(
        &self,
        value: &ParentReturn,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != PARENT_TAX_PAGE || !parent_tax_in_walk(&self.draft) || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.parent_tax_candidate(value)
    }
}
