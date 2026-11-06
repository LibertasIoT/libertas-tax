// ira_contributions items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// "Covered by a retirement plan at work" (M3-P8): a W-2 box 13 retirement
// plan or a SEP contribution of this owner's own business or partnership.
pub(crate) fn ira_covered(d: &FederalDraft, owner: Owner) -> bool {
    d.income.iter().any(|e| {
        e.owner == crate::DocumentOwner::from(owner)
            && match &e.income {
                FederalIncome::Wage { data } => crate::FederalWage::from(data).retirement_plan,
                FederalIncome::Business { data } => {
                    !matches!(data.retirement, BusinessRetirement::None)
                }
                FederalIncome::Partnership { data } => {
                    data.pensions != 0 || data.retirement != PartnerRetirement::None
                }
                _ => false,
            }
    }) || d.adjustments.as_ref().is_some_and(|a| {
        a.pension_contributions.iter().any(|p| p.owner == owner && p.contributions > 0)
    })
}
// Box 12 elective-deferral codes (401(k)/403(b)/SEP/457(b)/SIMPLE): one of
// the adjustments hub's hints to suggest the IRA topic.
pub(super) fn ira_deferral_codes(d: &FederalDraft) -> bool {
    d.income.iter().any(|e| match &e.income {
        FederalIncome::Wage { data } => crate::FederalWage::from(data)
            .other_box12
            .iter()
            .any(|c| matches!(c.code, W2Code::D | W2Code::E | W2Code::F | W2Code::G | W2Code::S)),
        _ => false,
    })
}
// A 529-to-Roth transfer on a Form 1099-Q (box 4b): the earlier transfers the
// $35,000 lifetime ceiling counts are asked on the IRA topic.
pub(super) fn ira_qtp_transfer(d: &FederalDraft) -> bool {
    !crate::ira::QtpTransfers::of(&d.income).is_empty()
}
// A prior year's Form 8606: the other hint to suggest the IRA topic.
pub(super) fn ira_prior_return(d: &FederalDraft) -> bool {
    d.adjustments.as_ref().is_some_and(|a| !a.ira_basis.is_empty())
}
// The IRA page's rows for the stored FederalIras; empty when nothing is
// entered. None for a shape the page does not ask (a timely-returned
// contribution or a completed recharacterization).
pub(super) fn ira_contributor_rows(d: &FederalDraft) -> Option<Vec<IraContributor>> {
    let Some(iras) = d.adjustments.as_ref().and_then(|a| a.entered_iras()) else {
        // An owner with a Form 8606 gets a zero row carrying its year-end
        // value, so an excess record added there need not repeat it (C18);
        // a zero row saves as no contributions.
        let joint = d.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Joint);
        return Some(
            [Owner::Taxpayer, Owner::Spouse]
                .into_iter()
                .filter(|owner| *owner == Owner::Taxpayer || joint)
                .filter_map(|owner| {
                    form_8606_year_end(d, owner).map(|value| IraContributor {
                        owner,
                        traditional: 0,
                        roth: 0,
                        covered: ira_covered(d, owner),
                        form_8606_value: Some(value),
                        more: IraMore::default(),
                    })
                })
                .collect(),
        );
    };
    let mut people = alloc::vec![ira_contributor_item(Owner::Taxpayer, &iras.taxpayer, false, d)?];
    match &iras.spouse {
        IraSpouse::Joint { facts } => {
            people.push(ira_contributor_item(Owner::Spouse, facts, false, d)?);
        }
        // The spouse's coverage on a separate return folds into the
        // taxpayer's own row: this page has no spouse row to file it under.
        IraSpouse::SeparateTogether { covered_at_work } => {
            people[0] = ira_contributor_item(Owner::Taxpayer, &iras.taxpayer, *covered_at_work, d)?;
        }
        IraSpouse::NotApplicable => {}
    }
    Some(people)
}
// The owner's Form 8606 year-end traditional IRA value (IraBasis.year_end),
// when the Retirement page holds that owner's Form 8606.
pub(crate) fn form_8606_year_end(d: &FederalDraft, owner: Owner) -> Option<i64> {
    d.adjustments.as_ref()?.ira_basis.iter().find(|b| b.owner == owner).map(|b| b.year_end)
}
// Form 5329 line 17's traditional IRA value from the owner's Form 8606: the
// December 31, 2026 value (Form 8606 line 6) plus the 2026 contributions made
// in 2027 (i5329 line 17 includes them; line 6 does not), T2-cy.
pub(super) fn form_5329_traditional_value(
    d: &FederalDraft,
    owner: Owner,
) -> Option<crate::kernel::ratio::exact::Exact> {
    d.adjustments
        .as_ref()?
        .ira_basis
        .iter()
        .find(|b| b.owner == owner)
        .map(crate::IraBasis::form_5329_value_exact)
}
// Keeps a stored IRA excess record's traditional value equal to the
// owner's Form 8606 year-end value plus 2026 contributions made in 2027
// whenever the return changes: the facts have their home on the Form 8606
// and the IRA contributions page (C18). The Form 8606's paid_next_year is
// derived from that page's 2027 deposits, so it follows them first.
pub(super) fn follow_ira_year_end(d: &mut FederalDraft) {
    if let Some(adjustments) = d.adjustments.as_mut() {
        let deposits = [Owner::Taxpayer, Owner::Spouse]
            .map(|owner| ira_next_year_deposit(Some(&*adjustments), owner));
        for basis in &mut adjustments.ira_basis {
            basis.paid_next_year = deposits[usize::from(basis.owner == Owner::Spouse)];
        }
    }
    let values =
        [Owner::Taxpayer, Owner::Spouse].map(|owner| form_5329_traditional_value(d, owner));
    let Some(iras) = d.adjustments.as_mut().map(|a| &mut a.iras) else {
        return;
    };
    let spouse = match &mut iras.spouse {
        IraSpouse::Joint { facts } => Some(facts),
        _ => None,
    };
    for (facts, value) in [Some(&mut iras.taxpayer), spouse].into_iter().zip(values) {
        if let (Some(facts), Some(value)) = (facts, value)
            && let Some(excess) = facts.excess.as_mut()
        {
            excess.traditional_value = crate::kernel::ratio::exact::rounded(&value);
        }
    }
}
pub(super) fn ira_contributor_item(
    owner: Owner,
    facts: &FederalIraContribution,
    spouse_covered_if_separate: bool,
    d: &FederalDraft,
) -> Option<IraContributor> {
    let form_8606_value = form_8606_year_end(d, owner);
    let excess = match &facts.excess {
        Some(record) => Some(crate::IraExcessEntry::from_record(record, form_8606_value)?),
        None => None,
    };
    let more = IraMore {
        nondeductible: facts.nondeductible_election,
        paid_next_year: facts.next_year_deposits.clone(),
        spouse_covered_if_separate,
        excess,
        returned_in_following_year: facts.returned_next_year.clone(),
        qtp_roth_prior: facts.qtp_roth_lifetime_prior,
    };
    Some(IraContributor {
        owner,
        traditional: facts.traditional,
        roth: facts.roth,
        covered: ira_covered(d, owner),
        form_8606_value,
        more,
    })
}
// A contributor row onto its record; absent (no row submitted) is a
// complete zero contribution with the owner's derived coverage.
pub(super) fn ira_contribution_record(
    d: &FederalDraft,
    row: Option<&IraContributor>,
    owner: Owner,
) -> FederalIraContribution {
    let none = IraMore::default();
    let (traditional, roth, more) = match row {
        Some(p) => (p.traditional, p.roth, &p.more),
        None => (0, 0, &none),
    };
    let nondeductible_election = more.nondeductible;
    let next_year_deposits = more.paid_next_year.clone();
    // The save checked the traditional value's one home first.
    let excess = more.excess.as_ref().and_then(|e| e.record(form_5329_traditional_value(d, owner)));
    FederalIraContribution {
        traditional,
        roth,
        nondeductible_election,
        covered_at_work: ira_covered(d, owner),
        next_year_deposits,
        qtp_roth_lifetime_prior: more.qtp_roth_prior,
        returned_next_year: more.returned_in_following_year.clone(),
        excess,
    }
}

// Form 8606 items (already known to be at most one per owner) onto the
// stored IRA basis worksheet. `paid_next_year` is derived from the IRA
// contributions page's 2027 deposits (Adjustments), never asked here.
pub(super) fn ira_basis_records(
    items: &[IncomeDocumentItem],
    adjustments: Option<&FederalAdjustments>,
    repayments: Vec<(Owner, crate::IraRepayment)>,
) -> Vec<crate::IraBasis> {
    items
        .iter()
        .filter_map(|item| match &item.document {
            IncomeSource::Record { value: crate::IncomeRecord::Form8606(f) } => {
                Some(crate::IraBasis {
                    owner: item.owner.owned(),
                    prior: f.basis,
                    adjustment: f.adjustment,
                    year_end: f.year_end,
                    paid_next_year: ira_next_year_deposit(adjustments, item.owner.owned()),
                    reservist_repayments: f.reservist_repayments.clone(),
                    // The 1099-R rows' repayments of their special distributions.
                    special_repayments: repayments
                        .iter()
                        .filter(|(owner, _)| *owner == item.owner.owned())
                        .map(|(_, repayment)| repayment.clone())
                        .collect(),
                })
            }
            _ => None,
        })
        .collect()
}
pub(super) fn ira_next_year_deposit(adjustments: Option<&FederalAdjustments>, owner: Owner) -> i64 {
    let Some(iras) = adjustments.and_then(|a| a.entered_iras()) else {
        return 0;
    };
    let contribution = match owner {
        Owner::Taxpayer => Some(&iras.taxpayer),
        Owner::Spouse => iras.spouse.as_ref(),
    };
    contribution.and_then(|c| c.next_year_deposits.as_ref()).map_or(0, |n| n.traditional)
}

impl Interview {
    // The IRA contributions page, opened with its complete value: the saved
    // contributors, or an empty list (M3-P8).
    pub(super) fn ira_contributions_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let Some(people) = ira_contributor_rows(d) else {
            return problem_key("PROBLEM_UNDISPLAYABLE_IRA_RECORD");
        };
        Reply::Edit(P::SaveIraContributions {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: ADJUSTMENTS_PAGE,
            previous: ADJUSTMENTS_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::IraContributions),
            warnings: Vec::new(),
            progress: self.topic_progress(ADJUSTMENTS_PAGE, Some(TaxSection::IraContributions)),
            allowed_owners: self.allowed_owners(),
            value: IraContributions { people },
        })
    }
    // The IRA contributions page's rows merged into the accepted return
    // (M3-P8): every row's facts are stored as entered (a record with nothing
    // entered is no IRA contribution), and the
    // spouse variant follows the return's own filing status and living
    // arrangement (FederalSetup::ira_spouse_index), never a client choice.
    pub(super) fn ira_contributions_candidate(
        &self,
        value: &IraContributions,
    ) -> Result<FederalDraft, SaveFailure> {
        let d = &self.draft;
        if let Some(issue) = crate::ira::contributions_issue(value, d, &self.allowed_owners()) {
            return Err(issue.in_field("value").into());
        }
        let mut draft = d.clone();
        if let Some(adjustments) = draft.adjustments.as_mut() {
            adjustments.iras = {
                let taxpayer_row = value.people.iter().find(|p| p.owner == Owner::Taxpayer);
                let taxpayer = ira_contribution_record(d, taxpayer_row, Owner::Taxpayer);
                let spouse =
                    match d.setup.as_ref().map(|s| (s.filing_status(), s.lived_apart_all_year())) {
                        Some((FederalStatus::Joint, _)) => {
                            let spouse_row = value.people.iter().find(|p| p.owner == Owner::Spouse);
                            IraSpouse::Joint {
                                facts: ira_contribution_record(d, spouse_row, Owner::Spouse),
                            }
                        }
                        Some((FederalStatus::Separate, false)) => IraSpouse::SeparateTogether {
                            covered_at_work: taxpayer_row
                                .is_some_and(|p| p.more.spouse_covered_if_separate),
                        },
                        _ => IraSpouse::NotApplicable,
                    };
                FederalIras { taxpayer, spouse }
            };
        }
        // A walked topic saved is visited: the walk passes it (M3-P19).
        visit(&self.draft, &mut draft, TaxSection::Adjustments, TaxSection::IraContributions);
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_ira_contributions(
        &self,
        value: &IraContributions,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != ADJUSTMENTS_PAGE || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.ira_contributions_candidate(value)
    }
}
