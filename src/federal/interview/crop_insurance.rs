use super::*;
use crate::farm_insurance_paper::{
    self as crop, AcceptedCropClassification, AcceptedCropTiming, CropPaperFacts, CropPaperTiming,
};

impl Interview {
    fn crop_insurance_page(&self, activity: i64) -> Reply {
        let Some(source) = crate::farm::sources(&self.draft).find(|s| s.entry.id == activity)
        else {
            return problem_key("PROBLEM_RELATED_FORM_UNAVAILABLE");
        };
        let claims = crate::related_forms::effective_claimants(&self.draft);
        let cash = matches!(source.farm.accounting, crate::FarmAccounting::Cash { .. });
        let defer =
            cash && matches!(source.farm.insurance_papers.timing, Some(CropPaperTiming::Defer));
        let papers = crop::papers(&self.draft, activity, &claims);
        let mut allowed_actions =
            alloc::vec![libertas_macros::variant_index!(P::CloseCropInsuranceFacts) as i32];
        if cash && (!papers.is_empty() || source.farm.insurance.current_total() > 0) {
            allowed_actions
                .insert(0, libertas_macros::variant_index!(P::SaveCropInsuranceTiming) as i32);
        }
        if defer && !papers.is_empty() {
            allowed_actions.push(libertas_macros::variant_index!(P::OpenCropPaperFacts) as i32);
        }
        if defer && !crop::crop_names(&self.draft, activity, &claims).is_empty() {
            allowed_actions.push(libertas_macros::variant_index!(P::OpenCropPractice) as i32);
        }
        Reply::Response(P::CropInsuranceFactsPage {
            cookie: self.draft.cookie.clone(),
            revision: self.draft.revision,
            activity,
            label: income_issue_context(source.entry).label().encode(),
            allowed_actions,
            accepted: source
                .farm
                .insurance_papers
                .timing
                .clone()
                .map(|timing| AcceptedCropTiming { timing }),
            papers: papers
                .into_iter()
                .map(|p| crop::CropPaperLink { source: p.id, summary: p.summary })
                .collect(),
            crops: if defer {
                crop::crop_names(&self.draft, activity, &claims)
                    .into_iter()
                    .map(|crop| crop::CropPracticeLink { crop })
                    .collect()
            } else {
                Vec::new()
            },
        })
    }
    fn save_crop_facts(
        &mut self,
        activity: i64,
        change: impl FnOnce(&mut crate::FarmIncome),
    ) -> Reply {
        let mut draft = self.draft.clone();
        let Some(farm) =
            draft.income.iter_mut().find(|e| e.id == activity).and_then(crop::farm_mut)
        else {
            return problem_key("PROBLEM_RELATED_FORM_UNAVAILABLE");
        };
        change(farm);
        match self.commit(draft) {
            Ok(_) => self.crop_insurance_page(activity),
            Err(message) => problem(message),
        }
    }
    pub(super) fn handle_crop_insurance(&mut self, request: &P) -> Option<Reply> {
        let claims = crate::related_forms::effective_claimants(&self.draft);
        let activity = match request {
            P::OpenCropInsuranceFacts { activity, .. }
            | P::SaveCropInsuranceTiming { activity, .. }
            | P::CloseCropInsuranceFacts { activity, .. }
            | P::OpenCropPaperFacts { activity, .. }
            | P::SaveCropPaperFacts { activity, .. }
            | P::OpenCropPractice { activity, .. }
            | P::SaveCropPractice { activity, .. } => *activity,
            _ => return None,
        };
        let Some(source) = crate::farm::sources(&self.draft).find(|s| s.entry.id == activity)
        else {
            return Some(problem_key("PROBLEM_RELATED_FORM_UNAVAILABLE"));
        };
        let cash = matches!(source.farm.accounting, crate::FarmAccounting::Cash { .. });
        let defer = matches!(source.farm.insurance_papers.timing, Some(CropPaperTiming::Defer));
        Some(match request {
            P::OpenCropInsuranceFacts { .. } => self.crop_insurance_page(activity),
            P::CloseCropInsuranceFacts { .. } => self.topic_page(TaxSection::Farming, false),
            P::SaveCropInsuranceTiming { timing, .. } => {
                if !cash
                    || (crop::papers(&self.draft, activity, &claims).is_empty()
                        && source.farm.insurance.current_total() == 0)
                {
                    problem_key("PROBLEM_SECTION_UNAVAILABLE_HERE")
                } else {
                    let timing = timing.clone();
                    self.save_crop_facts(activity, move |farm| {
                        farm.insurance_papers.timing = Some(timing)
                    })
                }
            }
            P::OpenCropPaperFacts { source: component, .. } if cash && defer => {
                if let Some(paper) = crop::papers(&self.draft, activity, &claims)
                    .into_iter()
                    .find(|p| p.id == *component)
                {
                    let accepted = source
                        .farm
                        .insurance_papers
                        .papers
                        .iter()
                        .find(|p| p.source == *component)
                        .map(|p| AcceptedCropClassification {
                            classification: p.classification.clone(),
                        });
                    Reply::Response(P::BeginCropPaperFacts {
                        cookie: self.draft.cookie.clone(),
                        revision: self.draft.revision,
                        activity,
                        source: *component,
                        summary: paper.summary,
                        accepted,
                    })
                } else {
                    problem_key("PROBLEM_RELATED_FORM_UNAVAILABLE")
                }
            }
            P::SaveCropPaperFacts { source: component, classification, .. } if cash && defer => {
                if let Some(paper) = crop::papers(&self.draft, activity, &claims)
                    .into_iter()
                    .find(|p| p.id == *component)
                {
                    if classification.payments(paper.summary.amount.unwrap_or(0)).is_none() {
                        refused(
                            ValidationIssue::new("CROP_PAPER_ALLOCATION_INVALID")
                                .at_field("classification"),
                        )
                    } else {
                        let facts = CropPaperFacts {
                            source: *component,
                            classification: classification.clone(),
                        };
                        self.save_crop_facts(activity, move |farm| {
                            farm.insurance_papers.papers.retain(|p| p.source != facts.source);
                            farm.insurance_papers.papers.push(facts);
                        })
                    }
                } else {
                    problem_key("PROBLEM_RELATED_FORM_UNAVAILABLE")
                }
            }
            P::OpenCropPractice { crop: label, .. }
                if cash
                    && defer
                    && crop::crop_names(&self.draft, activity, &claims).contains(label) =>
            {
                Reply::Response(P::BeginCropPractice {
                    cookie: self.draft.cookie.clone(),
                    revision: self.draft.revision,
                    activity,
                    crop: label.clone(),
                    accepted: source
                        .farm
                        .insurance_papers
                        .practice
                        .iter()
                        .find(|p| &p.crop == label)
                        .cloned(),
                })
            }
            P::SaveCropPractice { crop: label, normal_total, normal_following, .. }
                if cash
                    && defer
                    && crop::crop_names(&self.draft, activity, &claims).contains(label) =>
            {
                if normal_following > normal_total {
                    refused(
                        ValidationIssue::new("CROP_PRACTICE_ABOVE_TOTAL")
                            .at_field("normal_following"),
                    )
                } else {
                    let practice = crate::FarmCropPractice {
                        crop: label.clone(),
                        normal_total: *normal_total,
                        normal_following: *normal_following,
                    };
                    self.save_crop_facts(activity, move |farm| {
                        farm.insurance_papers.practice.retain(|p| p.crop != practice.crop);
                        farm.insurance_papers.practice.push(practice);
                    })
                }
            }
            _ => problem_key("PROBLEM_SECTION_UNAVAILABLE_HERE"),
        })
    }
}
