// form_8889 items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
pub(super) fn hsa_accounts(d: &FederalDraft) -> &[FederalHsa] {
    d.adjustments.as_ref().map_or(&[], |a| &a.hsa)
}
pub(super) fn health_correction_reference_rule(d: &FederalDraft) -> Option<&'static str> {
    health_correction_income_issue(d, d, &[]).map(|issue| issue.key)
}
// A health relationship can be broken by editing its actual income source.
// The retained account field is outside an income request; identify only a
// changed source that is actually submitted. Deletions and unchanged retained
// payments (including one whose inherited group was removed) remain unlocated.
pub(super) fn health_income_source_issue(
    issue: ValidationIssue,
    source: Option<i64>,
    original: &FederalDraft,
    candidate: &FederalDraft,
    origins: &[(i64, usize, bool)],
) -> ValidationIssue {
    let unlocated = || ValidationIssue::new(issue.key);
    let Some(id) = source else {
        return unlocated();
    };
    let Some((_, index, _)) = origins.iter().find(|(source, _, _)| *source == id) else {
        return unlocated();
    };
    let Some(before) = original.income.iter().find(|entry| entry.id == id) else {
        return unlocated();
    };
    let Some(after) = candidate.income.iter().find(|entry| entry.id == id) else {
        return unlocated();
    };
    let field = if before.owner != after.owner && before.income == after.income {
        "owner"
    } else if before.income != after.income {
        // Source availability and aggregate deposit capacity govern the
        // submitted source record, not a guessed child of its active Union.
        "document"
    } else {
        return unlocated();
    };
    unlocated().at_field(field).in_list("documents", *index).in_field("documents")
}
pub(super) fn health_correction_income_issue(
    original: &FederalDraft,
    candidate: &FederalDraft,
    origins: &[(i64, usize, bool)],
) -> Option<ValidationIssue> {
    health_correction_source_issue(candidate).map(|(source, issue)| {
        health_income_source_issue(issue, source, original, candidate, origins)
    })
}
pub(super) fn health_correction_source_issue(
    candidate: &FederalDraft,
) -> Option<(Option<i64>, ValidationIssue)> {
    use crate::hsa_correction::CorrectionSource;
    if let Some(failure) = crate::hsa_funding::source_reference_issue(candidate) {
        return Some((failure.income_source, failure.issue));
    }
    if let Some((location, issue)) =
        crate::medical_rollover_return::correction_reference_issue(candidate)
    {
        let source = match location {
            CorrectionSource::Hsa { correction, .. } => correction.failed_deposit,
            CorrectionSource::Archer { correction, .. } => correction.failed_deposit,
            CorrectionSource::Document { id, .. } => Some(id),
        };
        return Some((source, issue));
    }
    let correction_source = |(location, issue)| {
        let source = match location {
            CorrectionSource::Hsa { correction, .. } => correction.document,
            CorrectionSource::Archer { correction, .. } => correction.document,
            CorrectionSource::Document { id, .. } => Some(id),
        };
        (source, issue)
    };
    crate::hsa_correction::reference_issue(hsa_accounts(candidate), &candidate.income)
        .or_else(|| {
            candidate.adjustments.as_ref().and_then(|adjustments| {
                crate::hsa_correction::archer_reference_issue(
                    &adjustments.archer,
                    &candidate.income,
                )
            })
        })
        .map(correction_source)
}
// Graph failures carry source facts, never a canonical-list position. Resolve
// those facts against this submitted page before composing its locator.
pub(super) fn hsa_correction_page_issue(
    page: &HsaPage,
    draft: &FederalDraft,
    failure: crate::hsa_correction::CorrectionReferenceIssue,
) -> ValidationIssue {
    use crate::hsa_correction::CorrectionSource;
    let (source, issue) = failure;
    match source {
        CorrectionSource::Hsa { owner, correction } => {
            for (i, account) in page.accounts.iter().enumerate() {
                if account.owner == owner
                    && let Some(j) =
                        account.corrections.iter().position(|c| c.record() == correction)
                {
                    return issue
                        .in_list("corrections", j)
                        .in_list("accounts", i)
                        .in_field("value");
                }
            }
            issue.at_field("accounts").in_field("value")
        }
        CorrectionSource::Archer { owner, correction } => {
            if let crate::ArcherContributionsPage::Records { owners, .. } = &page.archer {
                for (i, account) in owners.iter().enumerate() {
                    if account.owner == owner
                        && let Some(j) =
                            account.corrections.iter().position(|c| c.record() == correction)
                    {
                        return issue
                            .in_list("corrections", j)
                            .in_list("owners", i)
                            .in_field("archer")
                            .in_field("value");
                    }
                }
            }
            issue.at_field("archer").in_field("value")
        }
        CorrectionSource::Document { owner, id } => {
            // Missing details and aggregate gross/earnings belong to the
            // editable correction collection; source-document fields are not
            // fields of an account or a particular submitted correction.
            let hsa = draft.income.iter().any(|e| {
                e.id == id
                    && e.owner == crate::DocumentOwner::from(owner)
                    && matches!(&e.income, FederalIncome::MedicalDistribution { data }
                    if matches!(data.account(), crate::MedicalSaAccount::Hsa))
            });
            if hsa {
                if let Some(i) = page.accounts.iter().position(|a| a.owner == owner) {
                    return issue.at_field("corrections").in_list("accounts", i).in_field("value");
                }
                issue.at_field("accounts").in_field("value")
            } else {
                if let crate::ArcherContributionsPage::Records { owners, .. } = &page.archer
                    && let Some(i) = owners.iter().position(|a| a.owner == owner)
                {
                    return issue
                        .at_field("corrections")
                        .in_list("owners", i)
                        .in_field("archer")
                        .in_field("value");
                }
                issue.at_field("archer").in_field("value")
            }
        }
    }
}
pub(super) fn hsa_funding_page_issue(
    page: &HsaPage,
    failure: crate::hsa_funding::FundingReferenceIssue,
) -> ValidationIssue {
    use crate::hsa_funding::FundingIssueLocation;
    let issue = failure.issue;
    match failure.location {
        FundingIssueLocation::Transfer { owner, transfer } => {
            if let Some(i) =
                page.funding.iter().position(|f| f.owner == owner && f.record() == transfer)
            {
                issue.in_list("funding", i).in_field("value")
            } else {
                issue.at_field("funding").in_field("value")
            }
        }
        FundingIssueLocation::Account { owner } => {
            if let Some(i) = page.accounts.iter().position(|a| a.owner == owner) {
                issue.in_list("accounts", i).in_field("value")
            } else {
                issue.at_field("accounts").in_field("value")
            }
        }
        FundingIssueLocation::Selection { owner } => {
            if page.accounts.iter().any(|a| a.owner == owner) {
                issue.in_field("value")
            } else {
                issue.at_field("accounts").in_field("value")
            }
        }
    }
}
pub(super) fn hsa_limit_method(d: &FederalDraft) -> HsaAllocationMethod {
    let share = d.adjustments.as_ref().and_then(|a| a.hsa_family_share.as_ref());
    // Older records used the last-month rule automatically outside a
    // married family allocation. Reopen a material old December choice
    // explicitly so saving an unchanged record preserves its tax result.
    let legacy_december = share.is_none()
        && !(d.setup.as_ref().is_some_and(|s| s.marital_state.married())
            && hsa_accounts(d)
                .iter()
                .any(|h| (0..12).any(|m| h.coverage.month(m) == HsaCoverage::Family)))
        && hsa_accounts(d).iter().any(|h| {
            h.coverage.month(11) != HsaCoverage::None
                && (0..12).any(|m| h.coverage.month(m) != h.coverage.month(11))
        });
    share.map_or(
        if legacy_december { HsaAllocationMethod::December } else { HsaAllocationMethod::Monthly },
        |s| s.method,
    )
}
// An owner whose W-2 reports employer HSA contributions but who has no
// account on the HSA page.
pub(super) fn hsa_account_missing(d: &FederalDraft) -> bool {
    if crate::contribution_documents::pending(d) {
        return true;
    }
    if crate::health_employer::mandatory_rule(d, crate::health_employer::bindings(d)).is_some() {
        return true;
    }
    [Owner::Taxpayer, Owner::Spouse].into_iter().any(|owner| {
        crate::health_employer::source_totals(d, crate::health_employer::bindings(d), owner, crate::HealthEmployerKind::Hsa).is_ok_and(|(current, _)| current > 0) && !hsa_accounts(d).iter().any(|h| h.owner == owner)
    }) || crate::medical_rollover_return::deposit_choices(d,None).iter().any(|source| source.deposits.iter().any(|f| f.year == crate::HEALTH_TAX_YEAR && match f.destination {
        crate::MedicalRolloverDestination::Hsa => !hsa_accounts(d).iter().any(|h|h.owner == source.owner),
        crate::MedicalRolloverDestination::Archer => !d.adjustments.as_ref().is_some_and(|a|matches!(&a.archer,crate::ArcherContributions::Records{owners,..} if owners.iter().any(|v|v.owner == source.owner))),
    }))
}
// The existing contribution owners retain one derived current-year total,
// from accepted source-year allocations plus outside-source principal.
pub(super) fn derive_hsa_employer(d: &mut FederalDraft) {
    crate::health_employer::prune(d);
    let Some(a) = d.adjustments.as_ref() else {
        return;
    };
    let bindings = &a.health_employer_w2;
    let hsa: Vec<_> = a
        .hsa
        .iter()
        .map(|h| {
            crate::health_employer::effective_total(
                d,
                bindings,
                h.owner,
                crate::HealthEmployerKind::Hsa,
                h.employer_outside,
            )
        })
        .collect();
    let archer: Vec<_> = match &a.archer {
        crate::ArcherContributions::None => Vec::new(),
        crate::ArcherContributions::Records { owners, .. } => owners
            .iter()
            .map(|h| {
                crate::health_employer::effective_total(
                    d,
                    bindings,
                    h.owner,
                    crate::HealthEmployerKind::Archer,
                    h.employer_outside,
                )
            })
            .collect(),
    };
    if let Some(a) = d.adjustments.as_mut() {
        for (h, amount) in a.hsa.iter_mut().zip(hsa) {
            if let Ok(amount) = amount {
                h.employer = amount;
            }
        }
        if let crate::ArcherContributions::Records { owners, .. } = &mut a.archer {
            for (h, amount) in owners.iter_mut().zip(archer) {
                if let Ok(amount) = amount {
                    h.employer = amount;
                }
            }
        }
    }
}
// Both eligible spouses share the family limit when either has family
// coverage in that month. A self-only policy for the other spouse does not
// avoid Form8889 line6. Equal is the default; Agreed records an actual choice.
pub(super) fn allowed_family_split(
    accounts: &[HsaAccount],
    d: &FederalDraft,
    other: Option<&HsaEligibility>,
    method: HsaAllocationMethod,
) -> Vec<i32> {
    let married = d.setup.as_ref().is_some_and(|s| s.marital_state.married());
    let eligible = [
        d.people.as_ref().is_some_and(|p| !p.taxpayer.dependency.claimable()),
        d.people.as_ref().and_then(|p| p.spouse.as_ref()).is_none_or(|p| !p.dependency.claimable()),
    ];
    let coverage = |m: usize| {
        let mut values = [HsaCoverage::None; 2];
        for i in 0..2 {
            if !eligible[i] {
                continue;
            }
            if let Some(a) = accounts.iter().find(|a| usize::from(a.owner == Owner::Spouse) == i) {
                values[i] = match &a.coverage {
                    HsaCoverageYear::SelfOnly => HsaCoverage::SelfOnly,
                    HsaCoverageYear::Family => HsaCoverage::Family,
                    HsaCoverageYear::Changed { months } => months[m],
                };
            } else if accounts.len() == 1 {
                values[i] = other.map_or(HsaCoverage::None, |c| c.month(m));
            }
        }
        values
    };
    let shared_month = |m| {
        let values = coverage(m);
        values.contains(&HsaCoverage::Family) && values.iter().all(|c| *c != HsaCoverage::None)
    };
    let shared = married
        && !accounts.is_empty()
        && (accounts.len() == 1 || accounts[0].owner != accounts[1].owner)
        && if method == HsaAllocationMethod::December {
            shared_month(11)
        } else {
            (0..12).any(shared_month)
        };
    if shared {
        alloc::vec![
            libertas_macros::variant_index!(FamilySplit::Equal) as i32,
            libertas_macros::variant_index!(FamilySplit::Agreed) as i32,
        ]
    } else {
        alloc::vec![libertas_macros::variant_index!(FamilySplit::NotApplicable) as i32]
    }
}

pub(super) fn family_split_index(split: &FamilySplit) -> i32 {
    (match split {
        FamilySplit::NotApplicable => libertas_macros::variant_index!(FamilySplit::NotApplicable),
        FamilySplit::Equal => libertas_macros::variant_index!(FamilySplit::Equal),
        FamilySplit::Agreed { .. } => libertas_macros::variant_index!(FamilySplit::Agreed),
    }) as i32
}
// An account onto its record, with the IRA-to-HSA transfers of its owner (the
// page lists the transfers once, each naming its owner).
pub(super) fn hsa_record(
    account: &HsaAccount,
    employer: i64,
    funding: &[crate::HsaFundingRow],
) -> FederalHsa {
    FederalHsa {
        owner: account.owner,
        coverage: match &account.coverage {
            HsaCoverageYear::SelfOnly => {
                HsaEligibility::FullYear { coverage: HsaCoverage::SelfOnly }
            }
            HsaCoverageYear::Family => HsaEligibility::FullYear { coverage: HsaCoverage::Family },
            HsaCoverageYear::Changed { months } => {
                HsaEligibility::Monthly { months: months.ordered().to_vec() }
            }
        },
        personal: account.personal,
        personal_outside: account.personal,
        employer,
        employer_outside: account.employer,
        late_excess: account.late_excess.clone(),
        prior_employer_excess: account.prior_employer_excess.clone(),
        corrections: account.corrections.iter().map(crate::HsaCorrectionPage::record).collect(),
        excess_accounting: account.excess_accounting.clone(),
        last_month_loss: account.last_month_loss.clone(),
        last_month_history: account.last_month_history.clone(),
        funding: funding
            .iter()
            .filter(|row| row.owner == account.owner)
            .map(crate::HsaFundingRow::record)
            .collect(),
        funding_history: account.funding_history.clone(),
        uses: account.uses.clone(),
    }
}
// The page's account for a stored record.
pub(crate) fn hsa_item(record: &FederalHsa, employer: i64) -> HsaAccount {
    let coverage = match &record.coverage {
        HsaEligibility::FullYear { coverage: HsaCoverage::SelfOnly } => HsaCoverageYear::SelfOnly,
        HsaEligibility::FullYear { coverage: HsaCoverage::Family } => HsaCoverageYear::Family,
        HsaEligibility::FullYear { coverage: HsaCoverage::None } => {
            HsaCoverageYear::Changed { months: HsaCoverageMonths::from_ordered(&[]) }
        }
        HsaEligibility::Monthly { months } => {
            HsaCoverageYear::Changed { months: HsaCoverageMonths::from_ordered(months) }
        }
    };
    HsaAccount {
        owner: record.owner,
        coverage,
        personal: record.personal,
        employer,
        corrections: record.corrections.iter().map(crate::HsaCorrection::page).collect(),
        late_excess: record.late_excess.clone(),
        prior_employer_excess: record.prior_employer_excess.clone(),
        excess_accounting: record.excess_accounting.clone(),
        last_month_loss: record.last_month_loss.clone(),
        last_month_history: record.last_month_history.clone(),
        funding_history: record.funding_history.clone(),
        uses: record.uses.clone(),
    }
}
// An income record as a choice: its document and whose it is.
pub(super) fn source_choice(entry: &FederalIncomeEntry) -> crate::RecordChoice {
    crate::RecordChoice {
        id: entry.id,
        label: crate::Message::new(
            "SOURCE_DOCUMENT_CHOICE",
            alloc::vec![
                crate::MessageArg::Nested(alloc::boxed::Box::new(
                    income_issue_context(entry).label()
                )),
                crate::MessageArg::Resource(
                    if entry.owner == crate::DocumentOwner::Spouses {
                        "PERSON_SPOUSE"
                    } else {
                        "PERSON_YOU"
                    }
                    .into()
                ),
            ],
        )
        .encode(),
    }
}
// The split onto hsa_family_share: the monthly method, one part each when
// equal.
pub(super) fn family_share_record(
    split: FamilySplit,
    method: HsaAllocationMethod,
    other_coverage: Option<HsaEligibility>,
    accounts: usize,
) -> Option<HsaFamilyShare> {
    if accounts == 0 {
        return None;
    }
    let (taxpayer_parts, spouse_parts) = match split {
        FamilySplit::NotApplicable | FamilySplit::Equal => (1, 1),
        FamilySplit::Agreed { you, spouse } => (you, spouse),
    };
    Some(HsaFamilyShare { method, taxpayer_parts, spouse_parts, other_coverage })
}
pub(super) fn family_split_item(share: Option<&HsaFamilyShare>) -> FamilySplit {
    match share.map(|v| (v.taxpayer_parts, v.spouse_parts)) {
        None => FamilySplit::NotApplicable,
        Some((1, 1)) => FamilySplit::Equal,
        Some((you, spouse)) => FamilySplit::Agreed { you, spouse },
    }
}

// Married with exactly one account whose owner contributes: the other filer's
// actual eligibility is a required answer (the same test as the estimate's
// HSA_OTHER_OWNER_ELIGIBILITY_REQUIRED). `employers` is each account's
// effective W-2 amount.
pub(super) fn hsa_other_needed(d: &FederalDraft, value: &HsaPage, employers: &[i64]) -> bool {
    let people = d.people.as_ref();
    let claimable = |owner| match owner {
        Owner::Taxpayer => people.is_some_and(|p| p.taxpayer.dependency.claimable()),
        _ => people.and_then(|p| p.spouse.as_ref()).is_some_and(|p| p.dependency.claimable()),
    };
    value.other_coverage.is_none()
        && d.setup.as_ref().is_some_and(|s| s.marital_state.married())
        && value.accounts.len() == 1
        && value.accounts.iter().zip(employers).any(|(a, employer)| {
            let record = hsa_record(a, *employer, &value.funding);
            !claimable(a.owner)
                && (a.personal + employer > 0 || !value.funding.is_empty())
                && (0..12).any(|m| record.month(m) != HsaCoverage::None)
        })
}

impl Interview {
    // The saved HSA page still needs the other filer's eligibility.
    pub(super) fn hsa_other_open(&self) -> Option<Owner> {
        let d = &self.draft;
        let Reply::Edit(P::SaveHsa { value, .. }) = self.hsa_page(false) else { return None };
        let employers: Vec<i64> = value
            .accounts
            .iter()
            .map(|a| {
                crate::health_employer::effective_total(
                    d,
                    crate::health_employer::bindings(d),
                    a.owner,
                    crate::HealthEmployerKind::Hsa,
                    a.employer,
                )
                .unwrap_or(0)
            })
            .collect();
        hsa_other_needed(d, &value, &employers).then(|| value.accounts[0].owner)
    }
}

impl Interview {
    // The HSA page, opened with its complete value: the saved accounts with
    // their outside-source principal and actual W-2 year sources, or an empty
    // list; the family split's allowed set as HsaOptions would compute it.
    pub(super) fn hsa_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let employer_sources =
            match crate::health_employer::page_rows(d, crate::health_employer::bindings(d)) {
                Ok(rows) => rows,
                Err(key) => return problem_key(key),
            };
        let accounts: Vec<HsaAccount> = hsa_accounts(d)
            .iter()
            .map(|h| {
                let mut row = hsa_item(h, h.employer_outside);
                row.personal = crate::contribution_documents::personal_input(
                    d,
                    h.owner,
                    crate::HealthContributionKind::Hsa,
                    h.personal,
                    h.personal_outside,
                );
                row
            })
            .collect();
        let funding =
            hsa_accounts(d).iter().flat_map(|h| h.funding.iter().map(|f| f.row(h.owner))).collect();
        let share = d.adjustments.as_ref().and_then(|a| a.hsa_family_share.as_ref());
        let other_coverage = share.and_then(|s| s.other_coverage.clone());
        let family_method = hsa_limit_method(d);
        let mut family_split = family_split_item(share);
        let allowed_family_split =
            allowed_family_split(&accounts, d, other_coverage.as_ref(), family_method);
        if !allowed_family_split.contains(&family_split_index(&family_split)) {
            family_split =
                if allowed_family_split.contains(&family_split_index(&FamilySplit::Equal)) {
                    FamilySplit::Equal
                } else {
                    FamilySplit::NotApplicable
                };
        }
        Reply::Edit(P::SaveHsa {
            followups: self.hsa_followups(),
            questions: Vec::new(),
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: ADJUSTMENTS_PAGE,
            previous: ADJUSTMENTS_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::Hsa),
            warnings: Vec::new(),
            progress: self.topic_progress(ADJUSTMENTS_PAGE, Some(TaxSection::Hsa)),
            allowed_owners: self.allowed_owners(),
            other_coverage_allowed: d.setup.as_ref().is_some_and(|s| s.marital_state.married()),
            allowed_family_split,
            hsa_failed_deposits: crate::medical_rollover_return::deposit_choices(
                d,
                Some(crate::MedicalRolloverDestination::Hsa),
            ),
            archer_failed_deposits: crate::medical_rollover_return::deposit_choices(
                d,
                Some(crate::MedicalRolloverDestination::Archer),
            ),
            hsa_return_documents: self.hsa_return_documents(),
            archer_return_documents: self.archer_return_documents(),
            ira_sources: self.hsa_ira_sources(),
            archer_income: self.archer_income_sources(),
            value: HsaPage {
                employer_statements: {
                    let mut ids = alloc::collections::BTreeSet::new();
                    employer_sources
                        .iter()
                        .filter(|row| ids.insert(row.document))
                        .map(|row| crate::RecordChoice {
                            id: row.document,
                            label: row.label.clone(),
                        })
                        .collect()
                },
                contribution_sources: crate::contribution_documents::rows(d),
                employer_sources,
                linked_deposits: crate::medical_rollover_return::deposit_choices(d, None),
                other_coverage: other_coverage.as_ref().map(HsaEligibilityPage::from_record),
                family_method,
                accounts,
                family_split,
                funding,
                archer: d.adjustments.as_ref().map_or(crate::ArcherContributionsPage::None, |a| {
                    let mut page = a.archer.page();
                    if let (
                        crate::ArcherContributionsPage::Records { owners: rows, .. },
                        crate::ArcherContributions::Records { owners, .. },
                    ) = (&mut page, &a.archer)
                    {
                        for (row, account) in rows.iter_mut().zip(owners) {
                            row.personal = crate::contribution_documents::personal_input(
                                d,
                                account.owner,
                                crate::HealthContributionKind::Archer,
                                account.personal,
                                account.personal_outside,
                            );
                        }
                    }
                    page
                }),
            },
        })
    }
    // Actual accepted corrective sources, keyed by their income identity.
    // The page catalog is descriptive; candidate validation reads stored sources.
    pub(super) fn hsa_return_documents(&self) -> Vec<crate::HealthReturnDocumentChoice> {
        self.correction_return_documents(true)
    }
    pub(super) fn archer_return_documents(&self) -> Vec<crate::HealthReturnDocumentChoice> {
        self.correction_return_documents(false)
    }
    pub(super) fn correction_return_documents(
        &self,
        hsa: bool,
    ) -> Vec<crate::HealthReturnDocumentChoice> {
        self.draft
            .income
            .iter()
            .filter_map(|entry| {
                let FederalIncome::MedicalDistribution { data } = &entry.income else {
                    return None;
                };
                let data = &crate::FederalMedicalDistribution::from(data);
                if !matches!(
                    (&data.account, hsa),
                    (crate::MedicalSaAccount::Hsa, true) | (crate::MedicalSaAccount::Archer, false)
                ) {
                    return None;
                }
                let Some(crate::MedicalSaPayment::ExcessReturned) = data.effective_payment() else {
                    return None;
                };
                Some(crate::HealthReturnDocumentChoice {
                    id: entry.id,
                    owner: entry.owner.owned(),
                    gross: data.gross,
                    earnings: data.box2_earnings,
                    label: Message::new(
                        "HEALTH_RETURN_DOCUMENT_CHOICE",
                        alloc::vec![income_display_arg(entry), MessageArg::usd_cents(data.gross)],
                    )
                    .encode(),
                })
            })
            .collect()
    }
    // IRA distributions a qualified IRA-to-HSA transfer can name, keyed by
    // the existing income identity: ordinary, Roth or inherited IRA.
    pub(super) fn hsa_ira_sources(&self) -> Vec<crate::HsaFundingSourceChoice> {
        self.draft
            .income
            .iter()
            .filter_map(|entry| {
                let source = crate::retirement_return::ira_funding_source(&self.draft, entry.id)?;
                let kind = match &entry.income {
                    FederalIncome::Roth { data: source } => {
                        let data = &crate::RothWithdrawal::from(source);
                        crate::HsaFundingSourceKind::Roth { distributed: data.distributed }
                    }
                    FederalIncome::Retirement { .. } => crate::HsaFundingSourceKind::Traditional,
                    _ => return None,
                };
                Some(crate::HsaFundingSourceChoice {
                    id: entry.id,
                    label: source_choice(entry).label,
                    owner: entry.owner.owned(),
                    kind,
                    inherited_died: source.inherited.map(|(_, group)| group.died),
                    owner_died: self
                        .draft
                        .people
                        .as_ref()
                        .and_then(|people| {
                            if entry.owner == crate::DocumentOwner::Yours {
                                Some(&people.taxpayer)
                            } else {
                                people.spouse.as_ref()
                            }
                        })
                        .and_then(|person| person.death_date()),
                })
            })
            .collect()
    }

    // The List host of each Archer row's compensation: every element names an
    // entry the page offers, and no entry is named twice. A stale element is
    // refused, never dropped or remapped.
    pub(super) fn archer_income_issue(
        &self,
        archer: &crate::ArcherContributionsPage,
    ) -> Option<ValidationIssue> {
        let crate::ArcherContributionsPage::Records { owners, .. } = archer else {
            return None;
        };
        let offered = self.archer_income_sources();
        owners.iter().enumerate().find_map(|(index, row)| {
            if let Some(i) = row.income.iter().position(|id| !offered.iter().any(|c| c.id == *id)) {
                Some(
                    ValidationIssue::new("ARCHER_INCOME_NOT_OFFERED")
                        .at_list_item("income", i)
                        .in_list("owners", index),
                )
            } else if let Some((i, _)) =
                row.income.iter().enumerate().find(|(i, id)| row.income[..*i].contains(id))
            {
                Some(
                    ValidationIssue::new("ARCHER_INCOME_TWICE")
                        .at_list_item("income", i)
                        .in_list("owners", index),
                )
            } else {
                None
            }
        })
    }
    // The earned compensation an Archer MSA owner's plan can cover, keyed by
    // income identity: W-2s, Forms 8919 and the Schedule C and F businesses.
    pub(super) fn archer_income_sources(&self) -> Vec<crate::RecordChoice> {
        self.draft
            .income
            .iter()
            .filter(|entry| {
                matches!(
                    entry.income,
                    FederalIncome::Wage { .. }
                        | FederalIncome::EmployeeWages { .. }
                        | FederalIncome::Business { .. }
                )
            })
            .map(source_choice)
            .collect()
    }
    // The HSA page's accounts and split merged into the accepted return.
    pub(super) fn hsa_candidate(&self, value: &HsaPage) -> Result<FederalDraft, SaveFailure> {
        let d = &self.draft;
        let owners = self.allowed_owners();
        let bindings = crate::health_employer::accept_page_issue(
            d,
            crate::health_employer::bindings(d),
            &value.employer_sources,
        )
        .map_err(|issue| SaveFailure::from(issue.in_field("value")))?;
        let allocations = crate::contribution_documents::accept(d, &value.contribution_sources)
            .map_err(|issue| SaveFailure::from(issue.in_field("value")))?;
        let mut source_draft = d.clone();
        if let Some(adjustments) = &mut source_draft.adjustments {
            adjustments.health_employer_w2 = bindings.clone();
            adjustments.health_contribution_allocations = allocations;
            adjustments.hsa =
                value.accounts.iter().map(|a| hsa_record(a, 0, &value.funding)).collect();
            adjustments.archer = value.archer.record();
        }
        let statement_totals = match crate::contribution_documents::project(&source_draft) {
            Ok(totals) => totals,
            Err((
                _,
                "HEALTH_STATEMENT_SPLIT_REQUIRED"
                | "HEALTH_STATEMENT_ACCOUNT_REQUIRED"
                | "CHOOSE_WHOSE_DOCUMENT_THIS_IS"
                | "HEALTH_EMPLOYER_YEAR_REQUIRED",
            )) => Default::default(),
            Err((id, key)) => {
                let issue = ValidationIssue::new(key).at_field("allocation");
                let issue =
                    value.contribution_sources.iter().position(|row| row.id == id).map_or_else(
                        || issue.clone().at_field("contribution_sources"),
                        |index| issue.clone().in_list("contribution_sources", index),
                    );
                return Err(issue.in_field("value").into());
            }
        };
        let original_value = value;
        let mut projected_value = value.clone();
        for account in &mut projected_value.accounts {
            let source = &statement_totals.hsa[usize::from(account.owner == Owner::Spouse)];
            account.personal = account.personal.checked_add(source.personal).ok_or_else(|| {
                SaveFailure::from(
                    ValidationIssue::new("HEALTH_STATEMENT_TOTAL_INVALID")
                        .at_field("accounts")
                        .in_field("value"),
                )
            })?;
        }
        if let crate::ArcherContributionsPage::Records { owners, .. } = &mut projected_value.archer
        {
            for account in owners {
                let source = &statement_totals.archer[usize::from(account.owner == Owner::Spouse)];
                account.personal =
                    account.personal.checked_add(source.personal).ok_or_else(|| {
                        SaveFailure::from(
                            ValidationIssue::new("HEALTH_STATEMENT_TOTAL_INVALID")
                                .at_field("archer")
                                .in_field("value"),
                        )
                    })?;
            }
        }
        let value = &projected_value;
        let employer = |owner, kind, outside, location: ValidationIssue| {
            crate::health_employer::effective_total(&source_draft, &bindings, owner, kind, outside)
                .map_err(|key| SaveFailure { message: status(key), field: location.field })
        };
        let hsa_employer: Vec<i64> = value
            .accounts
            .iter()
            .enumerate()
            .map(|(index, account)| {
                employer(
                    account.owner,
                    crate::HealthEmployerKind::Hsa,
                    account.employer,
                    ValidationIssue::new("HEALTH_EMPLOYER_TOTAL_INVALID")
                        .at_list_item("accounts", index)
                        .in_field("value"),
                )
            })
            .collect::<Result<_, _>>()?;
        let mut archer_inputs = value.archer.clone();
        if let crate::ArcherContributionsPage::Records { owners, .. } = &mut archer_inputs {
            for (index, owner) in owners.iter_mut().enumerate() {
                owner.employer = employer(
                    owner.owner,
                    crate::HealthEmployerKind::Archer,
                    owner.employer,
                    ValidationIssue::new("HEALTH_EMPLOYER_TOTAL_INVALID")
                        .at_list_item("owners", index)
                        .in_field("archer")
                        .in_field("value"),
                )?;
            }
        }
        if let Some(index) =
            value.accounts.iter().position(|account| !owners.contains(&(account.owner as i32)))
        {
            return Err(ValidationIssue::new(
                "MESSAGE_YOUR_SPOUSE_S_ACCOUNT_BELONGS_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
            )
            .at_field("owner")
            .in_list("accounts", index)
            .in_field("value")
            .into());
        }
        // Tax structure: one annual HSA aggregate per actual filer, regardless
        // of the number of submitted rows; allowed owners were checked above.
        if let Some(index) = value.accounts.iter().enumerate().position(|(index, account)| {
            value.accounts[..index].iter().any(|prior| prior.owner == account.owner)
        }) {
            let issue = ValidationIssue::new("HSA_ACCOUNT_TWICE")
                .at_field("owner")
                .in_list("accounts", index);
            return Err(issue.in_field("value").into());
        }
        if value.family_method == HsaAllocationMethod::December && value.accounts.is_empty() {
            return Err(ValidationIssue::new("HSA_LIMIT_METHOD_NEEDS_ACCOUNT")
                .at_field("family_method")
                .in_field("value")
                .into());
        }
        // Tax structure: the other spouse's eligibility applies only when one
        // filer has an HSA aggregate; monthly requests carry all twelve months.
        // An echo of the stored answer that the accounts no longer call for is
        // pruned; a new answer that does not apply is refused below.
        let echoed =
            d.adjustments.as_ref().and_then(|a| a.hsa_family_share.as_ref()).and_then(|share| {
                share.other_coverage.as_ref().map(HsaEligibilityPage::from_record)
            });
        let married = d.setup.as_ref().is_some_and(|s| s.marital_state.married());
        let mut value = value.clone();
        if value.other_coverage.is_some()
            && value.other_coverage == echoed
            && (value.accounts.len() != 1 || !married)
        {
            value.other_coverage = None;
        }
        let value = &value;
        let other_coverage = value.other_coverage.as_ref().map(HsaEligibilityPage::record);
        if (value.other_coverage.is_some() && value.accounts.len() != 1)
            || (value.other_coverage.is_some()
                && !d.setup.as_ref().is_some_and(|s| s.marital_state.married()))
        {
            return Err(ValidationIssue::new("HSA_OTHER_OWNER_CONTEXT_INVALID")
                .at_field("other_coverage")
                .in_field("value")
                .into());
        }
        // The allowed set HsaOptions computes, checked again on save.
        if !allowed_family_split(&value.accounts, d, other_coverage.as_ref(), value.family_method)
            .contains(&family_split_index(&value.family_split))
        {
            return Err(ValidationIssue::new("MESSAGE_SPLIT_THE_FAMILY_CONTRIBUTION_LIMIT_ONLY_WHEN_BOTH_OF_YOU_HAVE_AN_ACCOUNT_WI").at_field("family_split").in_field("value").into());
        }
        if matches!(value.family_split, FamilySplit::Agreed { you: 0, spouse: 0 }) {
            return Err(ValidationIssue::new("HSA_FAMILY_PARTS_ZERO")
                .at_field("family_split")
                .in_field("value")
                .into());
        }
        // The transfers: each names an owner with an account on this page, and the
        // rules of each row and account are checked again with the client's keys.
        if let Some(i) =
            value.funding.iter().position(|f| !value.accounts.iter().any(|a| a.owner == f.owner))
        {
            return Err(ValidationIssue::new("HSA_FUNDING_NEEDS_AN_ACCOUNT")
                .at_field("owner")
                .in_list("funding", i)
                .in_field("value")
                .into());
        }
        if let Some(issue) = value
            .accounts
            .iter()
            .zip(&hsa_employer)
            .enumerate()
            .find_map(|(i, (a, amount))| a.issue(*amount).map(|issue| issue.in_list("accounts", i)))
            .or_else(|| {
                value
                    .funding
                    .iter()
                    .enumerate()
                    .find_map(|(i, f)| f.issue().map(|issue| issue.in_list("funding", i)))
            })
            .or_else(|| archer_inputs.issue().map(|issue| issue.in_field("archer")))
            .or_else(|| {
                self.archer_income_issue(&value.archer).map(|issue| issue.in_field("archer"))
            })
        {
            return Err(issue.in_field("value").into());
        }
        let mut draft = source_draft.clone();
        let records: Vec<FederalHsa> = value
            .accounts
            .iter()
            .zip(&hsa_employer)
            .map(|(a, amount)| {
                let mut record = hsa_record(a, *amount, &value.funding);
                record.personal_outside = original_value
                    .accounts
                    .iter()
                    .find(|row| row.owner == a.owner)
                    .unwrap()
                    .personal;
                record
            })
            .collect();
        if let Some(issue) = records.iter().enumerate().find_map(|(index, record)| {
            record.record_issue().map(|issue| {
                issue
                    .map_field_prefix(&["employer_outside"], &["employer"])
                    .in_list("accounts", index)
                    .in_field("value")
            })
        }) {
            return Err(issue.into());
        }
        if let Some(failure) = crate::hsa_correction::source_issue(&records, &draft.income) {
            return Err(hsa_correction_page_issue(value, &draft, failure).into());
        }
        let mut archer = value.archer.record();
        if let crate::ArcherContributions::Records { owners, .. } = &mut archer {
            for (index, owner) in owners.iter_mut().enumerate() {
                if let crate::ArcherContributionsPage::Records { owners: outside, .. } =
                    &original_value.archer
                {
                    owner.personal_outside =
                        outside.iter().find(|row| row.owner == owner.owner).unwrap().personal;
                }
                owner.employer = employer(
                    owner.owner,
                    crate::HealthEmployerKind::Archer,
                    owner.employer_outside,
                    ValidationIssue::new("HEALTH_EMPLOYER_TOTAL_INVALID")
                        .at_list_item("owners", index)
                        .in_field("archer")
                        .in_field("value"),
                )?;
            }
        }
        if let Some(failure) = crate::hsa_correction::archer_source_issue(&archer, &draft.income) {
            return Err(hsa_correction_page_issue(value, &draft, failure).into());
        }
        if !archer.valid() {
            return Err(libertas_formatted_text("ARCHER_RECORD_INVALID", &[]).into());
        }
        if let Some(adjustments) = draft.adjustments.as_mut() {
            adjustments.health_employer_w2 = bindings;
            adjustments.hsa = records;
            adjustments.archer = archer;
            // Preserve an unchanged accepted implicit method. An explicit
            // new method or account/eligibility edit stores its full context;
            // merely opening and saving never changes the return revision.
            adjustments.hsa_family_share = if d
                .adjustments
                .as_ref()
                .is_some_and(|a| a.hsa_family_share.is_none() && a.hsa == adjustments.hsa)
                && value.other_coverage.is_none()
                && value.family_method == hsa_limit_method(d)
                && !matches!(value.family_split, FamilySplit::Agreed { .. })
            {
                None
            } else {
                family_share_record(
                    value.family_split,
                    value.family_method,
                    other_coverage,
                    value.accounts.len(),
                )
            };
        }
        if let Some(key) =
            crate::health_employer::reference_rule(&draft, crate::health_employer::bindings(&draft))
        {
            return Err(status(key).into());
        }
        // A current-year employer amount needs its owner's account record on
        // this page; it is never left for the Review.
        if let Some(kind) = crate::health_employer::account_missing(
            &draft,
            crate::health_employer::bindings(&draft),
        ) {
            let issue = ValidationIssue::new("HEALTH_EMPLOYER_ACCOUNT_REQUIRED");
            return Err(match kind {
                crate::HealthEmployerKind::Hsa => issue.at_field("accounts").in_field("value"),
                crate::HealthEmployerKind::Archer => issue.at_field("archer").in_field("value"),
            }
            .into());
        }
        if let Some(failure) = crate::hsa_funding::source_reference_issue(&draft) {
            return Err(hsa_funding_page_issue(value, failure).into());
        }
        if let Some(failure) = crate::medical_rollover_return::correction_reference_issue(&draft)
            .or_else(|| crate::hsa_correction::reference_issue(hsa_accounts(&draft), &draft.income))
            .or_else(|| {
                draft.adjustments.as_ref().and_then(|a| {
                    crate::hsa_correction::archer_reference_issue(&a.archer, &draft.income)
                })
            })
        {
            return Err(hsa_correction_page_issue(value, &draft, failure).into());
        }
        // A walked topic saved is visited: the walk passes it (M3-P19).
        visit(&self.draft, &mut draft, TaxSection::Adjustments, TaxSection::Hsa);
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_hsa(&self, value: &HsaPage, page: i32) -> Result<FederalDraft, SaveFailure> {
        if page != ADJUSTMENTS_PAGE || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.hsa_candidate(value)
    }
}
