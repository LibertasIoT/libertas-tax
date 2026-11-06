// charity items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// Shared source facts are edited and retained directly; tax-specific appraisal
// placement and validation have one owner in charity.rs.
pub(super) fn gift_issue(
    value: &GiftCarryovers,
    documents: &[crate::Form1098CDocument],
) -> Option<ValidationIssue> {
    crate::charity::page_issue(
        &value.gifts.iter().map(crate::PropertyGiftPage::stored).collect::<Vec<_>>(),
        &value.vehicles.iter().filter_map(crate::VehicleGiftPage::stored).collect::<Vec<_>>(),
        documents,
        &value.appraisals,
        &value.carryovers.iter().cloned().map(Into::into).collect::<Vec<_>>(),
        value.basis_election,
    )
}
pub(super) fn appraisal_catalog(values: &[crate::Appraisal]) -> Vec<crate::RecordChoice> {
    values
        .iter()
        .map(|a| crate::RecordChoice {
            id: a.id,
            label: crate::Message::new(
                "GIFT_APPRAISAL_CHOICE",
                alloc::vec![
                    crate::MessageArg::Text(a.appraiser.clone()),
                    crate::MessageArg::DateOnly(i64::from(a.report_date))
                ],
            )
            .encode(),
        })
        .collect()
}
// The deductions page's repayment answers: the comparison only while the
// income entries record more than $3,000 of earlier-year repayments.
pub(super) fn allowed_repayment(d: &FederalDraft) -> Vec<i32> {
    let mut allowed = alloc::vec![libertas_macros::variant_index!(RepaymentClaim::None) as i32];
    if crate::claim_repayment::repayment_offered(d) {
        allowed.push(libertas_macros::variant_index!(RepaymentClaim::Claim) as i32);
    }
    allowed
}
pub(super) fn repayment_index(claim: &RepaymentClaim) -> i32 {
    (match claim {
        RepaymentClaim::None => libertas_macros::variant_index!(RepaymentClaim::None),
        RepaymentClaim::Claim { .. } => libertas_macros::variant_index!(RepaymentClaim::Claim),
    }) as i32
}
// What the gifts of property and carryovers page holds (the prune's
// itemizing line): every list it shows.
pub(crate) fn gift_page_entered(v: &FederalDeductions) -> bool {
    !v.charity_carryovers.is_empty() || property_gifts_entered(v)
}
// The gifts of property on that page: the charity box's share of it.
pub(super) fn property_gifts_entered(v: &FederalDeductions) -> bool {
    !v.property_gifts.is_empty()
        || !v.vehicle_gift_documents.is_empty()
        || !v.vehicle_gifts.is_empty()
        || !v.appraisals.is_empty()
        || v.basis_election
        || !v.volunteer_expenses.is_empty()
        || !v.hosted_students.is_empty()
        || !v.whaling_captains.is_empty()
}
pub(super) fn clear_property_gifts(v: &mut FederalDeductions) {
    v.property_gifts.clear();
    v.vehicle_gifts.clear();
    v.appraisals.clear();
    v.basis_election = false;
    v.volunteer_expenses.clear();
    v.hosted_students.clear();
    v.whaling_captains.clear();
}
// Any gift to charity, cash or property: the gifts box's entries, so either
// one answers its plan.
pub(super) fn gifts_entered(d: &FederalDraft) -> bool {
    d.deductions.as_ref().is_some_and(|v| {
        v.cash_charity != 0
            || v.fund_cash_charity != 0
            || v.other_cash_charity != 0
            || property_gifts_entered(v)
    })
}

impl Interview {
    // The charity carryovers page, opened with its complete value. The
    // balances live on FederalDeductions directly, not the route's
    // ItemizedExpenses, so this only needs itemizing to be open, not `facts`.
    pub(super) fn gift_carryovers_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        if itemized_facts(d).is_none() {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
        }
        // itemized_facts above already confirmed d.deductions is Some.
        let ded = d.deductions.as_ref().unwrap();
        let appraisal_choices = appraisal_catalog(&ded.appraisals);
        let value = GiftCarryovers {
            gifts: ded.property_gifts.iter().cloned().map(Into::into).collect(),
            vehicles: ded
                .vehicle_gift_documents
                .iter()
                .map(|document| {
                    let mut gift =
                        ded.vehicle_gifts.iter().find(|gift| gift.id == document.id).cloned();
                    let appraisal = gift.as_mut().and_then(|gift| gift.appraisal.take());
                    crate::VehicleGiftPage { document: document.clone(), gift, appraisal }
                })
                .collect(),
            appraisals: ded.appraisals.clone(),
            appraisal_choices: appraisal_choices.clone(),
            carryovers: ded
                .charity_carryovers
                .iter()
                .cloned()
                .map(|record| {
                    let mut page = crate::CharityCarryoverPage::from(record);
                    page.allowed_research_parts = crate::retained_research::choices(
                        page.balance,
                        &crate::retained_research::available(d),
                    );
                    page.allowed_source_appreciation =
                        crate::retained_charity::appreciation_choices(
                            page.class,
                            page.balance,
                            page.appreciation,
                            ded.basis_election,
                            &crate::retained_research::available(d),
                        );
                    page
                })
                .collect(),
            basis_election: ded.basis_election,
            volunteer: ded.volunteer_expenses.clone(),
            less_common: less_common_rows(ded),
        };
        Reply::Edit(P::SaveGiftCarryovers {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: ITEMIZED_PAGE,
            previous: ITEMIZED_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::GiftCarryovers),
            warnings: Vec::new(),
            progress: self.topic_progress(ITEMIZED_PAGE, Some(TaxSection::GiftCarryovers)),
            allowed_owners: self.allowed_owners(),
            appraisal_choices,
            document_facts: DocumentFacts { year: 2026 },
            retained_charity_k1s: crate::investment_interest::k1_sources(d),
            retained_charity_origins: crate::retained_charity::origin_choices(d),
            retained_research_choices: crate::retained_research::available(d),
            value,
        })
    }
    // Charity carryovers (Pub 526; M3-P10). Stored on FederalDeductions
    // directly, not the route's ItemizedExpenses, so this replaces
    // prior_cash_charity itself rather than going through itemized_candidate.
    pub(super) fn gift_carryovers_candidate(
        &self,
        value: &GiftCarryovers,
    ) -> Result<FederalDraft, SaveFailure> {
        if itemized_facts(&self.draft).is_none() {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        for (index, record) in value.carryovers.iter().enumerate() {
            let stored: crate::CharityCarryover = record.clone().into();
            if let Some(issue) = crate::retained_charity::parts_issue(&stored, None) {
                return Err(issue.in_list("carryovers", index).in_field("value").into());
            }
        }
        let accepted = self.draft.deductions.as_ref().unwrap();
        for (index, page) in value.vehicles.iter().enumerate() {
            if !accepted
                .vehicle_gift_documents
                .iter()
                .any(|document| document.id == page.document.id)
            {
                return Err(ValidationIssue::new("VEHICLE_GIFT_DOCUMENT_NEEDED")
                    .at_field("id")
                    .in_field("document")
                    .in_list("vehicles", index)
                    .in_field("value")
                    .into());
            }
            if value.vehicles[..index]
                .iter()
                .any(|previous| previous.document.id == page.document.id)
            {
                return Err(ValidationIssue::new("VEHICLE_GIFT_DOCUMENT_TWICE")
                    .at_field("id")
                    .in_field("document")
                    .in_list("vehicles", index)
                    .in_field("value")
                    .into());
            }
        }
        if let Some(issue) = gift_issue(value, &accepted.vehicle_gift_documents) {
            let paths: Vec<_> = value
                .vehicles
                .iter()
                .enumerate()
                .filter(|(_, page)| page.gift.is_some())
                .map(|(index, _)| {
                    ValidationIssue::new("").at_list_item("vehicles", index).in_field("value").field
                })
                .collect();
            return Err(SaveFailure::from(issue.in_field("value"))
                .map_submitted_items(&["value", "vehicles"], &paths));
        }
        if let Some(issue) = less_common_issue(value, &self.allowed_owners()) {
            return Err(issue.in_field("value").into());
        }
        let mut draft = self.draft.clone();
        let mut appraisals = value.appraisals.clone();
        assign_record_ids(
            appraisals.iter_mut().map(|a| &mut a.id),
            accepted.appraisals.iter().map(|a| a.id),
            &mut draft.next_id,
        )
        .map_err(|(key, _, index)| {
            ValidationIssue::new(key).at_field("id").in_list("appraisals", index).in_field("value")
        })?;
        let deductions = draft.deductions.as_mut().unwrap();
        deductions.property_gifts =
            value.gifts.iter().map(crate::PropertyGiftPage::stored).collect();
        deductions.vehicle_gifts =
            value.vehicles.iter().filter_map(crate::VehicleGiftPage::stored).collect();
        deductions.appraisals = appraisals;
        deductions.charity_carryovers = value.carryovers.iter().cloned().map(Into::into).collect();
        deductions.basis_election = value.basis_election;
        deductions.volunteer_expenses = value.volunteer.clone();
        let gifts = less_common_gifts(&value.less_common);
        deductions.hosted_students = gifts.hosted_students;
        deductions.whaling_captains = gifts.whaling_captains;
        // This candidate replaced every carryover in submitted order above.
        if let Some((index, issue)) = crate::retained_charity::issue(&draft) {
            return Err(issue.in_list("carryovers", index).in_field("value").into());
        }
        finish_section(&mut draft, TaxSection::GiftCarryovers);
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_gift_carryovers(
        &self,
        value: &GiftCarryovers,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != ITEMIZED_PAGE || !self.reached(page) || !itemizing(&self.draft) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.gift_carryovers_candidate(value)
    }
}
