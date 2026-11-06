// form_8962 items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// Form 8962 Part IV choices this return offers: a former spouse's share
// only off a joint return, a spouse's share only on a separate return.
// Form 8962 line A is asked only on a separate return, where it has no
// default: the filer answers No or Yes.
pub(super) fn allowed_separate(status: FederalStatus) -> Vec<i32> {
    if status == FederalStatus::Separate {
        alloc::vec![SeparateRelief::No as i32, SeparateRelief::Yes as i32]
    } else {
        alloc::vec![SeparateRelief::NotSeparate as i32]
    }
}
// The people a dependent return can belong to: the people you support, in
// the people page's order. A dependent return of a person who left the return
// goes with the person (the save's prune), so no stale person is offered.
pub(super) fn marketplace_people(d: &FederalDraft) -> Vec<crate::PersonChoice> {
    d.dependents()
        .iter()
        .map(|person| crate::PersonChoice {
            id: person.id,
            label: person.name(),
            valid_ssn: person.valid_ssn,
        })
        .collect()
}
// A policy whose covered people nobody has answered takes them from the printed Part II: each
// individual the return knows by name, and every month the printed dates cover whole. Anything the
// paper does not settle (an individual who is not on the return, a partial month, no dates) leaves
// the policy for the person to answer, so a covered person is never dropped silently.
pub(super) fn follow_marketplace_coverage(d: &mut FederalDraft) {
    let Some(crate::MarketplaceClaim::Yes { facts }) = d.credits.as_ref().map(|c| &c.marketplace)
    else {
        return;
    };
    let fills: Vec<_> = facts
        .forms
        .iter()
        .enumerate()
        .filter(|(_, policy)| policy.coverage.is_empty())
        .filter_map(|(index, policy)| Some((index, printed_coverage(d, policy)?)))
        .collect();
    if let Some(crate::MarketplaceClaim::Yes { facts }) =
        d.credits.as_mut().map(|credits| &mut credits.marketplace)
    {
        for (index, coverage) in fills {
            facts.forms[index].coverage = coverage;
        }
    }
}
fn printed_coverage(
    d: &FederalDraft,
    policy: &crate::MarketplacePolicyRecord,
) -> Option<Vec<crate::MarketplacePolicyCoverage>> {
    let amounts = crate::MarketplacePolicyRecordPage::from(&policy.form);
    if amounts.status == MarketplaceStatementStatus::Void
        || policy.form.covered_individuals.is_empty()
    {
        return None;
    }
    // Whole months only: coverage starting on a month's first day and ending on a month's last.
    let covered: Vec<(i64, u32, u32)> = policy
        .form
        .covered_individuals
        .iter()
        .map(|individual| {
            let id = crate::document_name_matcher::match_eligible_recipient(
                d,
                &individual.name,
                crate::document_name_matcher::RecipientDomain::Student,
            )?;
            let start = individual.coverage_start_date.or(policy.form.policy_start_date)?;
            let end = individual
                .coverage_termination_date
                .or(policy.form.policy_termination_date)
                .unwrap_or(20261231);
            (crate::federal_rules::valid_date(start)
                && crate::federal_rules::valid_date(end)
                && start % 100 == 1
                && !crate::federal_rules::valid_date(end + 1))
            .then_some((id, start, end))
        })
        .collect::<Option<_>>()?;
    let months: Vec<_> = (1..=12u32)
        .filter(|month| {
            crate::marketplace_sources::policy_month(&amounts.amounts, *month).is_some()
        })
        .map(|month| {
            let first = 20260000 + month * 100 + 1;
            let mut people: Vec<i64> = covered
                .iter()
                .filter(|(_, start, end)| *start <= first && *end >= first)
                .map(|(id, _, _)| *id)
                .collect();
            people.sort_unstable();
            people.dedup();
            (!people.is_empty()).then_some(crate::MarketplacePolicyCoverage {
                month,
                people,
                entry: Vec::new(),
            })
        })
        .collect::<Option<_>>()?;
    Some(months)
}
pub(super) struct MarketplaceCatalog {
    pub(super) people: Vec<PersonChoice>,
    pub(super) ptc_people: Vec<RecordChoice>,
    pub(super) ptc_families: Vec<RecordChoice>,
    pub(super) ptc_policies: Vec<RecordChoice>,
    pub(super) ptc_w2: Vec<RecordChoice>,
    pub(super) ptc_employees: Vec<RecordChoice>,
    pub(super) ptc_nonreturn_people: Vec<RecordChoice>,
    pub(super) ptc_claimed_dependents: Vec<RecordChoice>,
    pub(super) allowed_marriage: Vec<i32>,
}
pub(super) fn marketplace_catalog(d: &FederalDraft) -> MarketplaceCatalog {
    let facts = d.credits.as_ref().and_then(|c| crate::marketplace::facts(&c.marketplace));
    let family = crate::marketplace::family_people(d);
    let current = crate::federal_rules::people_students(d);
    // Each filer is shown by name, not by role, so a policy's covered people read as printed.
    let named = |id: i64| {
        let people = d.people.as_ref()?;
        let person = match id {
            -1 => Some(&people.taxpayer),
            -2 => people.spouse.as_ref(),
            _ => None,
        }?;
        Some(crate::person_names::person_name(&person.first_name, &person.last_name))
    };
    let mut ptc_people: Vec<_> = current
        .iter()
        .map(|p| RecordChoice { id: p.id, label: named(p.id).unwrap_or_else(|| p.label.clone()) })
        .collect();
    let ptc_employees = ptc_people
        .iter()
        .filter(|p| p.id == -1 || p.id == -2 && joint(d) || p.id > 0 && family.contains(&p.id))
        .cloned()
        .collect();
    let ptc_claimed_dependents =
        ptc_people.iter().filter(|p| p.id > 0 && family.contains(&p.id)).cloned().collect();
    let ptc_nonreturn_people =
        ptc_people.iter().filter(|p| !family.contains(&p.id)).cloned().collect();
    let choice = |id, label: alloc::string::String| RecordChoice {
        id,
        label: Message::new(
            "PTC_RECORD_CHOICE",
            alloc::vec![MessageArg::Text(label), MessageArg::Number(id)],
        )
        .encode(),
    };
    ptc_people.extend(facts.into_iter().flat_map(|f| &f.covered).map(|p| {
        choice(
            p.id,
            match &p.identity {
                MarketplacePersonIdentity::OtherFamily { label, .. }
                | MarketplacePersonIdentity::NoFamily { label } => label.clone(),
            },
        )
    }));
    let ptc_families = facts
        .into_iter()
        .flat_map(|f| &f.other_families)
        .map(|f| choice(f.id, f.label.clone()))
        .collect();
    let ptc_policies = facts
        .into_iter()
        .flat_map(|f| &f.forms)
        .filter(|p| {
            crate::MarketplacePolicyRecordPage::from(&p.form).status
                != MarketplaceStatementStatus::Void
        })
        .map(|p| choice(p.id, alloc::format!("{} {}", p.form.issuer, p.form.policy_number)))
        .collect();
    let w2 = crate::marketplace::w2_sources(d);
    let ptc_w2 = d
        .income
        .iter()
        .filter(|e| w2.iter().any(|(id, _, _)| *id == e.id))
        .map(source_choice)
        .collect();
    let mut allowed_marriage =
        alloc::vec![libertas_macros::variant_index!(MarketplaceMarriagePage::Regular) as i32];
    if joint(d) {
        allowed_marriage
            .push(libertas_macros::variant_index!(MarketplaceMarriagePage::Alternative) as i32);
    }
    MarketplaceCatalog {
        people: marketplace_people(d),
        ptc_people,
        ptc_families,
        ptc_policies,
        ptc_w2,
        ptc_employees,
        ptc_nonreturn_people,
        ptc_claimed_dependents,
        allowed_marriage,
    }
}
pub(super) fn marketplace_value(d: &FederalDraft) -> MarketplaceFactsPage {
    d.credits
        .as_ref()
        .and_then(|c| crate::marketplace::facts(&c.marketplace))
        .cloned()
        .map(Into::into)
        .unwrap_or_default()
}
// The hub and review row: "1 Form 1095-A · credit $1,234.00" or
// "· repay $567.00"; None without a claim.
pub(super) fn marketplace_status(d: &FederalDraft, result: &FederalResult) -> Option<Vec<u8>> {
    let facts = crate::marketplace::facts(&d.credits.as_ref()?.marketplace)?;
    let count = facts
        .forms
        .iter()
        .filter(|p| {
            crate::MarketplacePolicyRecordPage::from(&p.form).status
                != MarketplaceStatementStatus::Void
        })
        .count() as i64;
    let line = |id: &str| result.lines.iter().find(|l| l.id == id).map(|l| l.amount);
    let (key, amount) = match (line("premium_credit_refundable"), line("premium_credit_repayment"))
    {
        (_, Some(repay)) if repay > 0 => ("REVIEW_MARKETPLACE_REPAY", Some(repay)),
        (Some(credit), _) => ("REVIEW_MARKETPLACE_CREDIT", Some(credit)),
        _ => ("REVIEW_MARKETPLACE_COUNT", None),
    };
    let mut args = alloc::vec![LibertasMessageArgument::Plural(count)];
    if let Some(minor_units) = amount {
        args.push(LibertasMessageArgument::Currency {
            currency_code: "USD",
            scale: 2,
            minor_units,
        });
    }
    Some(libertas_formatted_text(key, &args))
}

impl Interview {
    // The Form 1095-A page (T2-27p), opened with the stored claim
    // transposed onto its forms, or empty; the hidden lists carry the
    // choices this return offers and the people a dependent return can be.
    pub(super) fn marketplace_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let value = marketplace_value(d);
        let catalog = marketplace_catalog(d);
        let status = filing_status(d);
        Reply::Edit(P::SaveMarketplace {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: CREDITS_PAGE,
            previous: CREDITS_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::Marketplace),
            warnings: Vec::new(),
            progress: self.topic_progress(CREDITS_PAGE, Some(TaxSection::Marketplace)),
            people: catalog.people,
            ptc_people: catalog.ptc_people,
            ptc_families: catalog.ptc_families,
            ptc_policies: catalog.ptc_policies,
            ptc_w2: catalog.ptc_w2,
            ptc_employees: catalog.ptc_employees,
            ptc_nonreturn_people: catalog.ptc_nonreturn_people,
            ptc_claimed_dependents: catalog.ptc_claimed_dependents,
            allowed_marriage: catalog.allowed_marriage,
            allowed_separate: allowed_separate(status),
            document_facts: DocumentFacts { year: 2026 },
            value,
        })
    }
    // All own identities are assigned on the cloned candidate. Source links
    // are authorized from accepted IDs, never echoed picker catalogs.
    pub(super) fn marketplace_candidate(
        &self,
        value: &MarketplaceFactsPage,
    ) -> Result<FederalDraft, SaveFailure> {
        let accepted = self
            .draft
            .credits
            .as_ref()
            .and_then(|credits| crate::marketplace::facts(&credits.marketplace));
        let forms = accepted.map_or(&[][..], |facts| facts.forms.as_slice());
        if value.forms.len() != forms.len() {
            return Err(status("DOCUMENT_QUESTIONS_CHANGED").into());
        }
        let mut value = value.clone();
        let mut seen = alloc::collections::BTreeSet::new();
        for (index, row) in value.forms.iter_mut().enumerate() {
            let Some(source) =
                forms.iter().find(|source| source.id == row.id).filter(|_| seen.insert(row.id))
            else {
                return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                    .at_field("id")
                    .in_list("forms", index)
                    .in_field("value")
                    .into());
            };
            row.value = source.form.clone();
        }
        self.marketplace_candidate_rows(&value, true)
    }
    pub(super) fn marketplace_sources_candidate(
        &self,
        value: &MarketplaceFactsPage,
        forms: &[crate::Form1095ADocument],
    ) -> Result<FederalDraft, SaveFailure> {
        let accepted = self
            .draft
            .credits
            .as_ref()
            .and_then(|credits| crate::marketplace::facts(&credits.marketplace));
        let mut value = value.clone();
        value.forms = forms
            .iter()
            .map(|source| {
                let mut row: crate::MarketplaceDocumentView = accepted
                    .and_then(|facts| facts.forms.iter().find(|old| old.id == source.id))
                    .cloned()
                    .map_or_else(|| source.clone().into(), Into::into);
                row.value = source.value.clone();
                row.id = source.id;
                row
            })
            .collect();
        self.marketplace_candidate_rows(&value, false)
    }
    pub(super) fn marketplace_candidate_rows(
        &self,
        value: &MarketplaceFactsPage,
        require_answers: bool,
    ) -> Result<FederalDraft, SaveFailure> {
        let d = &self.draft;
        if let Some(issue) = crate::marketplace_sources::page_readonly_issue(value) {
            return Err(issue.in_field("value").into());
        }
        if !allowed_separate(filing_status(d)).contains(&(value.separate as i32))
            || !joint(d) && !matches!(value.marriage, MarketplaceMarriagePage::Regular)
        {
            return Err(status("MESSAGE_CHECK_THE_MONTHS_AND_AMOUNTS_OF_EACH_FORM_1095_A").into());
        }
        let mut draft = d.clone();
        let accepted = d
            .credits
            .as_ref()
            .and_then(|c| crate::marketplace::facts(&c.marketplace))
            .cloned()
            .unwrap_or_default();
        let mut facts: MarketplaceFacts = value.clone().into();
        for form in &mut facts.forms {
            if let Some(names) = &mut form.printed {
                names.retain_assignment_from(
                    accepted
                        .forms
                        .iter()
                        .find(|before| form.id != 0 && before.id == form.id)
                        .and_then(|before| before.printed.as_ref()),
                );
            }
        }
        if let Some(issue) = crate::marketplace_qsehra::entered_issue(&facts) {
            return Err(issue.in_field("value").into());
        }
        if let Some(issue) = crate::marketplace_sources::source_issue(&facts) {
            return Err(issue.in_field("value").into());
        }
        if let Some(issue) = crate::marketplace::family_sources_issue(&facts, d) {
            return Err(issue.in_field("value").into());
        }
        assign_record_ids(
            facts.forms.iter_mut().map(|r| &mut r.id),
            accepted.forms.iter().map(|r| r.id),
            &mut draft.next_id,
        )
        .map_err(|(key, _, index)| {
            ValidationIssue::new(key).at_field("id").in_list("forms", index).in_field("value")
        })?;
        assign_record_ids(
            facts.other_families.iter_mut().map(|r| &mut r.id),
            accepted.other_families.iter().map(|r| r.id),
            &mut draft.next_id,
        )
        .map_err(|(key, _, index)| {
            ValidationIssue::new(key)
                .at_field("id")
                .in_list("other_families", index)
                .in_field("value")
        })?;
        assign_record_ids(
            facts.covered.iter_mut().map(|r| &mut r.id),
            accepted.covered.iter().map(|r| r.id),
            &mut draft.next_id,
        )
        .map_err(|(key, _, index)| {
            ValidationIssue::new(key).at_field("id").in_list("covered", index).in_field("value")
        })?;
        assign_record_ids(
            facts.qsehra.iter_mut().map(|r| &mut r.id),
            accepted.qsehra.iter().map(|r| r.id),
            &mut draft.next_id,
        )
        .map_err(|(key, _, index)| {
            ValidationIssue::new(key).at_field("id").in_list("qsehra", index).in_field("value")
        })?;
        let (people, _) = crate::marketplace::current_people(d);
        if let Some(issue) = crate::marketplace_sources::authorized_issue(
            &facts,
            &accepted,
            &people,
            &crate::marketplace::w2_sources(d),
        ) {
            return Err(issue.in_field("value").into());
        }
        if require_answers {
            for (index, policy) in facts.forms.iter().enumerate() {
                if let Some(issue) = crate::marketplace::policy_answer_issue(&facts, d, policy) {
                    return Err(issue.in_list("forms", index).in_field("value").into());
                }
            }
        }
        let Some(credits) = draft.credits.as_mut() else {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        };
        credits.marketplace = if facts == MarketplaceFacts::default() {
            MarketplaceClaim::No
        } else {
            MarketplaceClaim::Yes { facts: alloc::boxed::Box::new(facts) }
        };
        finish_section(&mut draft, TaxSection::Marketplace);
        Ok(draft)
    }
}
