// se_benefits items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// The self-employed benefits page is offered while a business or a
// partnership K-1 exists (T2-43p: a row per Schedule K-1 (Form 1065)).
pub(super) fn has_benefit_rows(d: &FederalDraft) -> bool {
    d.income.iter().any(|e| {
        matches!(
            e.income,
            FederalIncome::Business { .. }
                | FederalIncome::Partnership { .. }
                | FederalIncome::Scorp { .. }
                | FederalIncome::PartnershipActivities { .. }
                | FederalIncome::ScorpActivities { .. }
        )
    })
}
// The self-employed benefits page's rows: one per business, built from
// FederalBusiness.health_insurance/.retirement (M3-P8). None for a business
// whose stored facts this page cannot represent (a shape only a direct
// fixture, not this page, can create).
pub(crate) fn business_benefit_rows(d: &FederalDraft) -> Option<Vec<BusinessBenefits>> {
    d.income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Business { data } => Some((e, data)),
            _ => None,
        })
        .map(|(e, data)| {
            Some(BusinessBenefits {
                id: e.id,
                name: income_issue_context(e).label().encode(),
                health: se_health_page(&data.health_insurance)?,
                retirement: se_retirement_page(&data.retirement)?,
            })
        })
        .collect()
}
// The self-employed benefits page's partnership rows (T2-43p): one per
// Schedule K-1 (Form 1065), built from PartnerK1; boxes 13 code M and R and
// box 19 code A are copied from the K-1 record. None for a stored shape the
// page cannot show (an unconfirmed SEP).
pub(crate) fn partner_benefit_rows(d: &FederalDraft) -> Option<Vec<PartnerBenefits>> {
    d.income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Partnership { data } => Some((e, data)),
            _ => None,
        })
        .map(|(e, data)| {
            let health = se_health_page(&data.health_insurance)?;
            Some(PartnerBenefits {
                id: e.id,
                name: income_issue_context(e).label().encode(),
                medical_insurance: data.health_guaranteed,
                pensions: data.pensions,
                cash_distributions: data.regular.basis.distributions,
                health,
                from_distributions: data.health_distributions,
                retirement: partner_plan_page(&data.retirement),
            })
        })
        .collect()
}
pub(crate) fn activity_partner_benefit_rows(
    d: &FederalDraft,
) -> Option<Vec<crate::K1ActivityPartnerBenefits>> {
    d.income
        .iter()
        .filter_map(|entry| match &entry.income {
            FederalIncome::PartnershipActivities { data } => Some((entry, data)),
            _ => None,
        })
        .map(|(entry, data)| {
            Some(crate::K1ActivityPartnerBenefits {
                id: entry.id,
                name: income_issue_context(entry).label().encode(),
                medical_insurance: data.person.as_ref().and_then(|v| v.health_guaranteed),
                pensions: data.person.as_ref().and_then(|v| v.pensions),
                cash_distributions: data.entity.regular.distributions,
                health: match data.health_insurance.as_ref() {
                    Some(value) => Some(se_health_page(value)?),
                    None => None,
                },
                from_distributions: data.person.as_ref().and_then(|v| v.health_distributions),
                retirement: data.retirement.as_ref().map(partner_plan_page),
            })
        })
        .collect()
}

pub(crate) fn activity_scorp_benefit_rows(
    d: &FederalDraft,
) -> Option<Vec<crate::K1ActivityScorpBenefits>> {
    d.income
        .iter()
        .filter_map(|entry| match &entry.income {
            FederalIncome::ScorpActivities { data } => Some((entry, data)),
            _ => None,
        })
        .map(|(entry, data)| {
            Some(crate::K1ActivityScorpBenefits {
                activity: entry.id,
                name: income_issue_context(entry).label().encode(),
                health: match data.health_insurance.as_ref() {
                    Some(value) => Some(se_health_page(value)?),
                    None => None,
                },
            })
        })
        .collect()
}
pub(crate) fn scorp_benefit_rows(d: &FederalDraft) -> Option<Vec<HealthPlanPage>> {
    d.income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Scorp { data } => Some((e, data)),
            _ => None,
        })
        .map(|(e, data)| {
            Some(HealthPlanPage {
                activity: e.id,
                name: income_issue_context(e).label().encode(),
                health: se_health_page(&data.personal.health_insurance)?,
            })
        })
        .collect()
}
// The names of the self-employed benefits page's rows with something
// entered, businesses first (T2-43p: partnerships too).
pub(super) fn benefits_entered(d: &FederalDraft) -> Vec<Vec<u8>> {
    let businesses = business_benefit_rows(d).unwrap_or_default();
    let partnerships = partner_benefit_rows(d).unwrap_or_default();
    businesses
        .into_iter()
        .filter(|b| {
            !matches!(b.health, SeHealth::None) || !matches!(b.retirement, SeRetirement::None)
        })
        .map(|b| b.name)
        .chain(
            partnerships
                .into_iter()
                .filter(|p| {
                    !matches!(p.health, SeHealth::None)
                        || p.retirement != PartnerPlan::None
                        || p.from_distributions != 0
                })
                .map(|p| p.name),
        )
        .chain(
            scorp_benefit_rows(d)
                .unwrap_or_default()
                .into_iter()
                .filter(|p| !matches!(p.health, SeHealth::None))
                .map(|p| p.name),
        )
        .chain(
            activity_partner_benefit_rows(d)
                .unwrap_or_default()
                .into_iter()
                .filter(|v| {
                    v.health.as_ref().is_some_and(|h| !matches!(h, SeHealth::None))
                        || v.retirement.as_ref().is_some_and(|r| !matches!(r, PartnerPlan::None))
                        || v.from_distributions.is_some_and(|value| value != 0)
                })
                .map(|v| v.name),
        )
        .chain(
            activity_scorp_benefit_rows(d)
                .unwrap_or_default()
                .into_iter()
                .filter(|v| v.health.as_ref().is_some_and(|h| !matches!(h, SeHealth::None)))
                .map(|v| v.name),
        )
        .collect()
}
// The self-employed benefits topic is necessary while nothing is entered on
// it and a business exists or a partnership K-1 shows box 13 code M or R
// (T2-43p, question Q7: an investor's K-1 does not suggest an empty page).
pub(super) fn benefits_needed(d: &FederalDraft) -> bool {
    benefits_entered(d).is_empty()
        && d.income.iter().any(|e| match &e.income {
            FederalIncome::Business { .. } | FederalIncome::Scorp { .. } => true,
            FederalIncome::Partnership { data } => {
                data.health_guaranteed != 0 || data.pensions != 0
            }
            FederalIncome::PartnershipActivities { data } => {
                data.person.as_ref().is_some_and(|v| {
                    v.health_guaranteed.is_some_and(|value| value != 0)
                        || v.pensions.is_some_and(|value| value != 0)
                })
            }
            FederalIncome::ScorpActivities { .. } => true,
            _ => false,
        })
}
// Native admission for one partnership row uses the accepted K-1 source,
// never the row's read-only copies. Canonical plan validation precedes these
// payment/plan relationships, which retain their existing native messages.
pub(super) fn partner_benefit_issue(
    row: &PartnerBenefits,
    data: &PartnerK1,
) -> Option<ValidationIssue> {
    partner_benefit_source_issue(
        Some(&row.health),
        Some(data.health_guaranteed),
        Some(row.from_distributions),
        data.regular.basis.distributions,
        Some(data.pensions),
        Some(&row.retirement),
    )
    .map(|issue| {
        if issue.key == "PARTNER_PREMIUMS_BELOW_K1" {
            issue.map_field_prefix(&["health"], &["from_distributions"])
        } else {
            issue
        }
    })
}
pub(super) fn partner_plan_page(value: &PartnerRetirement) -> PartnerPlan {
    match value {
        PartnerRetirement::None => PartnerPlan::None,
        PartnerRetirement::Sep { rate } => PartnerPlan::Sep { rate: *rate },
        PartnerRetirement::Plan(BusinessRetirement::Simple {
            deferral,
            employer,
            employer_contribution,
        }) => PartnerPlan::Simple {
            deferral: *deferral,
            employer: *employer,
            employer_contribution: *employer_contribution,
        },
        PartnerRetirement::Plan(BusinessRetirement::Qualified {
            rate,
            contribution,
            deferral,
            catch_up,
            roth,
        }) => PartnerPlan::Qualified {
            rate: *rate,
            contribution: *contribution,
            deferral: *deferral,
            catch_up: *catch_up,
            roth: *roth,
        },
        PartnerRetirement::Plan(BusinessRetirement::DefinedBenefit { contribution }) => {
            PartnerPlan::DefinedBenefit { contribution: *contribution }
        }
        PartnerRetirement::Plan(_) | PartnerRetirement::Other => PartnerPlan::Other,
    }
}

pub(crate) fn partner_plan_record(value: &PartnerPlan) -> PartnerRetirement {
    match value {
        PartnerPlan::None => PartnerRetirement::None,
        PartnerPlan::Sep { rate } => PartnerRetirement::Sep { rate: *rate },
        PartnerPlan::Simple { deferral, employer, employer_contribution } => {
            PartnerRetirement::Plan(BusinessRetirement::Simple {
                deferral: *deferral,
                employer: *employer,
                employer_contribution: *employer_contribution,
            })
        }
        PartnerPlan::Qualified { rate, contribution, deferral, catch_up, roth } => {
            PartnerRetirement::Plan(BusinessRetirement::Qualified {
                rate: *rate,
                contribution: *contribution,
                deferral: *deferral,
                catch_up: *catch_up,
                roth: *roth,
            })
        }
        PartnerPlan::DefinedBenefit { contribution } => {
            PartnerRetirement::Plan(BusinessRetirement::DefinedBenefit {
                contribution: *contribution,
            })
        }
        PartnerPlan::Other => PartnerRetirement::Other,
    }
}

pub(super) fn partner_benefit_source_issue(
    health: Option<&SeHealth>,
    medical: Option<i64>,
    payment: Option<i64>,
    distributions: i64,
    pensions: Option<i64>,
    retirement: Option<&PartnerPlan>,
) -> Option<crate::validation::ValidationIssue> {
    if let Some(health) = health {
        let premiums = match health {
            SeHealth::None => Some(0),
            SeHealth::Plan { months, long_term_care, .. } => crate::kernel::ratio::checked_sum(
                months
                    .iter()
                    .map(|m| m.premiums)
                    .chain(long_term_care.iter().flat_map(|p| &p.months).map(|m| m.premiums)),
            ),
        };
        let Some(premiums) = premiums else {
            return Some(
                crate::validation::ValidationIssue::new("PROBLEM_INVALID_HEALTH_PLAN_SOURCE")
                    .at_field("health"),
            );
        };
        // Both sources are nonnegative. One actually known part exceeding the
        // complete premium total proves a contradiction independently of the
        // other unknown part; it is never used as a zero for a deduction.
        let known_parts = match (medical, payment) {
            (Some(a), Some(b)) => {
                // The medical part is an accepted nonnegative source, and the
                // canonical plan owner bounds every premium before this call.
                // Positive overflow therefore proves this same mathematical
                // sum exceeds premiums. Negative submitted payments still reach
                // the existing later intrinsic source guard in its old order.
                let Some(total) = crate::kernel::ratio::checked_sum([a, b]) else {
                    return Some(
                        crate::validation::ValidationIssue::new("PARTNER_PREMIUMS_BELOW_K1")
                            .at_field("health"),
                    );
                };
                Some(total)
            }
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => None,
        };
        if known_parts.is_some_and(|v| v > premiums) {
            return Some(
                crate::validation::ValidationIssue::new("PARTNER_PREMIUMS_BELOW_K1")
                    .at_field("health"),
            );
        }
    }
    if payment.is_some_and(|v| v > distributions) {
        return Some(
            crate::validation::ValidationIssue::new("PARTNER_DISTRIBUTION_PREMIUMS_ABOVE_19A")
                .at_field("from_distributions"),
        );
    }
    match (pensions, retirement) {
        (Some(value), Some(PartnerPlan::None)) if value != 0 => Some(
            crate::validation::ValidationIssue::new("PARTNER_BOX_13R_NEEDS_PLAN")
                .at_field("retirement"),
        ),
        (Some(0), Some(plan)) if *plan != PartnerPlan::None => Some(
            crate::validation::ValidationIssue::new("PARTNER_PLAN_WITHOUT_BOX_13R")
                .at_field("retirement"),
        ),
        _ => None,
    }
}

pub(super) fn activity_partner_benefit_record_issue(
    data: &crate::PartnerActivityK1,
) -> Option<crate::validation::ValidationIssue> {
    let health = match data.health_insurance.as_ref() {
        Some(source) => Some(se_health_page(source)?),
        None => None,
    };
    let retirement = data.retirement.as_ref().map(partner_plan_page);
    partner_benefit_source_issue(
        health.as_ref(),
        data.person.as_ref().and_then(|v| v.health_guaranteed),
        data.person.as_ref().and_then(|v| v.health_distributions),
        data.entity.regular.distributions,
        data.person.as_ref().and_then(|v| v.pensions),
        retirement.as_ref(),
    )
}
// Every accepted premium plan reopens with all its source facts.
pub(super) fn se_health_page(value: &BusinessHealthInsurance) -> Option<SeHealth> {
    Some(match value {
        BusinessHealthInsurance::None => SeHealth::None,
        BusinessHealthInsurance::Plan { months, long_term_care, establishment } => SeHealth::Plan {
            establishment: establishment.clone().into(),
            months: months.clone(),
            long_term_care: long_term_care.iter().cloned().map(Into::into).collect(),
        },
    })
}
pub(crate) fn se_health_insurance(value: &SeHealth) -> BusinessHealthInsurance {
    match value {
        SeHealth::None => BusinessHealthInsurance::None,
        SeHealth::Plan { months, long_term_care, establishment } => BusinessHealthInsurance::Plan {
            establishment: establishment.clone().into(),
            months: months.clone(),
            long_term_care: long_term_care.iter().cloned().map(Into::into).collect(),
        },
    }
}
// The page's retirement choice for a stored plan.
pub(super) fn se_retirement_page(value: &BusinessRetirement) -> Option<SeRetirement> {
    Some(match value {
        BusinessRetirement::None => SeRetirement::None,
        BusinessRetirement::Sep { contribution, rate } => {
            SeRetirement::Sep { contribution: *contribution, rate: *rate }
        }
        BusinessRetirement::Simple { deferral, employer, employer_contribution } => {
            SeRetirement::Simple {
                deferral: *deferral,
                employer: *employer,
                employer_contribution: *employer_contribution,
            }
        }
        BusinessRetirement::Qualified { rate, contribution, deferral, catch_up, roth } => {
            SeRetirement::Qualified {
                rate: *rate,
                contribution: *contribution,
                deferral: *deferral,
                catch_up: *catch_up,
                roth: *roth,
            }
        }
        BusinessRetirement::DefinedBenefit { contribution } => {
            SeRetirement::DefinedBenefit { contribution: *contribution }
        }
    })
}
pub(super) fn se_retirement(value: &SeRetirement) -> BusinessRetirement {
    match value {
        SeRetirement::None => BusinessRetirement::None,
        SeRetirement::Sep { contribution, rate } => {
            BusinessRetirement::Sep { contribution: *contribution, rate: *rate }
        }
        SeRetirement::Simple { deferral, employer, employer_contribution } => {
            BusinessRetirement::Simple {
                deferral: *deferral,
                employer: *employer,
                employer_contribution: *employer_contribution,
            }
        }
        SeRetirement::Qualified { rate, contribution, deferral, catch_up, roth } => {
            BusinessRetirement::Qualified {
                rate: *rate,
                contribution: *contribution,
                deferral: *deferral,
                catch_up: *catch_up,
                roth: *roth,
            }
        }
        SeRetirement::DefinedBenefit { contribution } => {
            BusinessRetirement::DefinedBenefit { contribution: *contribution }
        }
    }
}

impl Interview {
    // The self-employed benefits page, opened with its complete value: one
    // row per business (M3-P8), built from its stored health insurance and
    // retirement plan.
    pub(super) fn self_employed_benefits_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let (
            Some(businesses),
            Some(partnerships),
            Some(scorporations),
            Some(activity_partnerships),
            Some(activity_scorporations),
        ) = (
            business_benefit_rows(d),
            partner_benefit_rows(d),
            scorp_benefit_rows(d),
            activity_partner_benefit_rows(d),
            activity_scorp_benefit_rows(d),
        )
        else {
            return problem_key("PROBLEM_UNDISPLAYABLE_BUSINESS_BENEFIT_RECORD");
        };
        let coordination =
            d.adjustments.as_ref().map(|a| a.health_marketplace.clone()).unwrap_or_default();
        let activities = crate::health_marketplace::activities(d);
        Reply::Edit(P::SaveSelfEmployedBenefits {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: ADJUSTMENTS_PAGE,
            previous: ADJUSTMENTS_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::SelfEmployedBenefits),
            warnings: Vec::new(),
            progress: Some(
                self.topic_progress(ADJUSTMENTS_PAGE, Some(TaxSection::SelfEmployedBenefits)),
            ),
            ptc_policies: marketplace_catalog(d).ptc_policies,
            health_people: crate::federal_rules::people_students(d),
            health_wages: d
                .income
                .iter()
                .filter(|e| matches!(e.income, FederalIncome::Wage { .. }))
                .map(source_choice)
                .collect(),
            health_activities: d
                .income
                .iter()
                .filter(|e| activities.contains(&e.id))
                .map(|e| RecordChoice { id: e.id, label: income_issue_context(e).label().encode() })
                .collect(),
            value: SelfEmployedBenefits {
                ptc_method: coordination.method,
                policies: coordination.policies.into_iter().map(Into::into).collect(),
                benchmarks: coordination.benchmarks.into_iter().map(Into::into).collect(),
                businesses,
                partnerships,
                scorporations,
                activity_partnerships,
                activity_scorporations,
            },
        })
    }
    // The self-employed benefits page's rows merged into the accepted
    // return: only the matching business's health insurance and retirement
    // plan change (M3-P8). Membership is read-only, so a mismatch (a
    // business added or removed since the page was built) is server-only;
    // the revision guard already refuses a save from a stale build.
    pub(super) fn self_employed_benefits_candidate(
        &self,
        value: &SelfEmployedBenefits,
    ) -> Result<FederalDraft, SaveFailure> {
        let d = &self.draft;
        let (
            Some(current),
            Some(partners),
            Some(corporations),
            Some(activity_partners),
            Some(activity_corporations),
        ) = (
            business_benefit_rows(d),
            partner_benefit_rows(d),
            scorp_benefit_rows(d),
            activity_partner_benefit_rows(d),
            activity_scorp_benefit_rows(d),
        )
        else {
            return Err(
                crate::interview::status("PROBLEM_UNDISPLAYABLE_BUSINESS_BENEFIT_RECORD").into()
            );
        };
        if value.businesses.len() != current.len()
            || value.businesses.iter().zip(&current).any(|(v, c)| v.id != c.id)
            || value.partnerships.len() != partners.len()
            || value.partnerships.iter().zip(&partners).any(|(v, c)| v.id != c.id)
            || value.scorporations.len() != corporations.len()
            || value.scorporations.iter().zip(&corporations).any(|(v, c)| v.activity != c.activity)
        {
            return Err(crate::interview::status(
                "MESSAGE_THIS_RETURN_S_BUSINESSES_CHANGED_REOPEN_THE_EDITOR_BEFORE_TRYING_AGAIN",
            )
            .into());
        }
        for (index, row) in value.businesses.iter().enumerate() {
            if let Some(issue) = se_retirement(&row.retirement).issue() {
                return Err(issue
                    .in_field("retirement")
                    .in_list("businesses", index)
                    .in_field("value")
                    .into());
            }
            if let Some(issue) = se_health_insurance(&row.health).issue() {
                return Err(issue
                    .in_field("health")
                    .in_list("businesses", index)
                    .in_field("value")
                    .into());
            }
        }
        for (index, row) in value.partnerships.iter().enumerate() {
            if let Some(issue) = se_health_insurance(&row.health).issue() {
                return Err(issue
                    .in_field("health")
                    .in_list("partnerships", index)
                    .in_field("value")
                    .into());
            }
        }
        for (index, row) in value.scorporations.iter().enumerate() {
            if let Some(issue) = se_health_insurance(&row.health).issue() {
                return Err(issue
                    .in_field("health")
                    .in_list("scorporations", index)
                    .in_field("value")
                    .into());
            }
        }
        for (field, id_field, submitted, accepted) in [
            (
                "activity_partnerships",
                "id",
                value.activity_partnerships.iter().map(|v| v.id).collect::<Vec<_>>(),
                activity_partners.iter().map(|v| v.id).collect::<Vec<_>>(),
            ),
            (
                "activity_scorporations",
                "activity",
                value.activity_scorporations.iter().map(|v| v.activity).collect::<Vec<_>>(),
                activity_corporations.iter().map(|v| v.activity).collect::<Vec<_>>(),
            ),
        ] {
            let key =
                "MESSAGE_THIS_RETURN_S_BUSINESSES_CHANGED_REOPEN_THE_EDITOR_BEFORE_TRYING_AGAIN";
            if submitted.len() != accepted.len() {
                return Err(crate::validation::ValidationIssue::new(key)
                    .at_field(field)
                    .in_field("value")
                    .into());
            }
            if let Some(index) = submitted.iter().zip(&accepted).position(|(v, c)| v != c) {
                return Err(crate::validation::ValidationIssue::new(key)
                    .at_field(id_field)
                    .in_list(field, index)
                    .in_field("value")
                    .into());
            }
        }
        let mut draft = d.clone();
        for entry in &mut draft.income {
            let FederalIncome::Business { data } = &mut entry.income else {
                continue;
            };
            let Some(row) = value.businesses.iter().find(|v| v.id == entry.id) else {
                continue;
            };
            data.health_insurance = se_health_insurance(&row.health);
            data.retirement = se_retirement(&row.retirement);
        }
        // Partnership rows (T2-43p): the K-1 boxes stay the K-1 topic's.
        for entry in &mut draft.income {
            let FederalIncome::Partnership { data } = &mut entry.income else {
                continue;
            };
            let Some((index, row)) =
                value.partnerships.iter().enumerate().find(|(_, v)| v.id == entry.id)
            else {
                continue;
            };
            if let Some(issue) = partner_benefit_issue(row, data) {
                return Err(issue.in_list("partnerships", index).in_field("value").into());
            }
            data.health_insurance = se_health_insurance(&row.health);
            data.health_distributions = row.from_distributions;
            data.retirement = partner_plan_record(&row.retirement);
        }
        for entry in &mut draft.income {
            if let FederalIncome::Scorp { data } = &mut entry.income
                && let Some(row) = value.scorporations.iter().find(|v| v.activity == entry.id)
            {
                data.personal.health_insurance = se_health_insurance(&row.health);
            }
        }
        for entry in &mut draft.income {
            match &mut entry.income {
                FederalIncome::PartnershipActivities { data } => {
                    let (index, row) = value
                        .activity_partnerships
                        .iter()
                        .enumerate()
                        .find(|(_, v)| v.id == entry.id)
                        .expect("accepted readonly source membership");
                    // Only this real writer changes the one accepted payment fact.
                    if row.from_distributions.is_some() || data.person.is_some() {
                        data.person
                            .get_or_insert_with(crate::PartnerPersonSource::default)
                            .health_distributions = row.from_distributions;
                    }
                    data.health_insurance = row.health.as_ref().map(se_health_insurance);
                    data.retirement = row.retirement.as_ref().map(partner_plan_record);
                    if let Some(issue) =
                        data.health_insurance.as_ref().and_then(|plan| plan.issue())
                    {
                        return Err(issue
                            .in_field("health")
                            .in_list("activity_partnerships", index)
                            .in_field("value")
                            .into());
                    }
                    // The benefit relationship belongs to this question. A K-1
                    // source correction may precede its matching plan correction.
                    if let Some(issue) = activity_partner_benefit_record_issue(data) {
                        return Err(issue
                            .in_list("activity_partnerships", index)
                            .in_field("value")
                            .into());
                    }
                    if let Some(issue) = data.benefits_issue() {
                        return Err(issue
                            .map_field_prefix(
                                &["person", "health_distributions"],
                                &["from_distributions"],
                            )
                            .in_list("activity_partnerships", index)
                            .in_field("value")
                            .into());
                    }
                }
                FederalIncome::ScorpActivities { data } => {
                    let (index, row) = value
                        .activity_scorporations
                        .iter()
                        .enumerate()
                        .find(|(_, v)| v.activity == entry.id)
                        .expect("accepted readonly source membership");
                    data.health_insurance = row.health.as_ref().map(se_health_insurance);
                    if let Some(issue) =
                        data.health_insurance.as_ref().and_then(|plan| plan.issue())
                    {
                        return Err(issue
                            .in_field("health")
                            .in_list("activity_scorporations", index)
                            .in_field("value")
                            .into());
                    }
                }
                _ => {}
            }
        }
        let coordination = HealthMarketplace {
            method: value.ptc_method,
            policies: value.policies.clone().into_iter().map(Into::into).collect(),
            benchmarks: value.benchmarks.clone().into_iter().map(Into::into).collect(),
        };
        if let Some(issue) = coordination.issue(&draft) {
            return Err(issue.in_field("value").into());
        }
        draft.adjustments.as_mut().expect("adjustments page").health_marketplace = coordination;
        if let Some((source, issue)) = crate::health::plans_issue(&draft) {
            let source = source.and_then(|id| {
                value
                    .businesses
                    .iter()
                    .position(|row| row.id == id)
                    .map(|i| ("businesses", i))
                    .or_else(|| {
                        value
                            .partnerships
                            .iter()
                            .position(|row| row.id == id)
                            .map(|i| ("partnerships", i))
                    })
                    .or_else(|| {
                        value
                            .scorporations
                            .iter()
                            .position(|row| row.activity == id)
                            .map(|i| ("scorporations", i))
                    })
                    .or_else(|| {
                        value
                            .activity_partnerships
                            .iter()
                            .position(|row| row.id == id)
                            .map(|i| ("activity_partnerships", i))
                    })
                    .or_else(|| {
                        value
                            .activity_scorporations
                            .iter()
                            .position(|row| row.activity == id)
                            .map(|i| ("activity_scorporations", i))
                    })
            });
            return Err(if let Some((list, index)) = source {
                issue.in_field("health").in_list(list, index).in_field("value")
            } else {
                ValidationIssue::new(issue.key)
            }
            .into());
        }
        // A walked topic saved is visited: the walk passes it (M3-P19).
        visit(&self.draft, &mut draft, TaxSection::Adjustments, TaxSection::SelfEmployedBenefits);
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_self_employed_benefits(
        &self,
        value: &SelfEmployedBenefits,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != ADJUSTMENTS_PAGE || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.self_employed_benefits_candidate(value)
    }
}
