pub use crate::social_sources::{SocialPremiumAllocation, SocialPremiumUse};
use crate::{BusinessHealthInsurance, FederalDraft, FederalIncome, FederalIncomeEntry};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Prevent the same Medicare premium from entering two expense sources
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum SocialPremiumUsePage {
    /// None included in a self-employed health plan
    /// The statement premiums enter itemized medical expenses automatically; do not type them again.
    Medical,
    /// Some or all already included in a self-employed health plan
    AlreadyInSelfEmployedPlan {
        /// Business whose health coverage already includes these premiums
        #[libertas_enum_source("^<DocumentQuestion>.health_plans")]
        income_id: i64,
        /// Part of the printed premiums already included in that plan's monthly premiums
        /// This is an allocation of an existing expense, not an additional premium.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}

impl From<SocialPremiumUse> for SocialPremiumUsePage {
    fn from(value: SocialPremiumUse) -> Self {
        match value {
            SocialPremiumUse::Medical => Self::Medical,
            SocialPremiumUse::AlreadyInSelfEmployedPlan { income_id, amount } => {
                Self::AlreadyInSelfEmployedPlan { income_id, amount }
            }
        }
    }
}

impl From<SocialPremiumUsePage> for SocialPremiumUse {
    fn from(value: SocialPremiumUsePage) -> Self {
        match value {
            SocialPremiumUsePage::Medical => Self::Medical,
            SocialPremiumUsePage::AlreadyInSelfEmployedPlan { income_id, amount } => {
                Self::AlreadyInSelfEmployedPlan { income_id, amount }
            }
        }
    }
}

pub(crate) fn printed_premiums(entry: &FederalIncomeEntry) -> i64 {
    match &entry.income {
        FederalIncome::SocialSecurity { data } => crate::FederalSocial::from(data)
            .form_boxes
            .paid_description
            .iter()
            .filter(|line| {
                matches!(
                    line.description,
                    crate::SocialBenefitDescriptionKind::MedicareTotal
                        | crate::SocialBenefitDescriptionKind::MedicarePartB
                        | crate::SocialBenefitDescriptionKind::MedicarePartC
                        | crate::SocialBenefitDescriptionKind::MedicarePartD
                )
            })
            .map(|line| line.amount)
            .sum(),
        FederalIncome::Retirement { data: source } => {
            let data = &crate::FederalRetirement::from(source);
            data.form_boxes.medicare_premiums
        }
        _ => 0,
    }
}

fn plan_premiums(entry: &FederalIncomeEntry) -> Option<i64> {
    match crate::health::source_health_plan(&entry.income)? {
        BusinessHealthInsurance::Plan { months, .. } => {
            Some(months.iter().map(|month| month.premiums).sum())
        }
        BusinessHealthInsurance::None => None,
    }
}

// A spouse's premiums may be included in the other spouse's family coverage.
pub(crate) fn plans(draft: &FederalDraft) -> impl Iterator<Item = &FederalIncomeEntry> {
    draft.income.iter().filter(|entry| plan_premiums(entry).is_some_and(|amount| amount > 0))
}

pub(crate) fn allocation(draft: &FederalDraft, id: i64) -> Option<&SocialPremiumUse> {
    draft
        .adjustments
        .as_ref()?
        .social_premiums
        .iter()
        .find(|row| row.document_id == id)
        .map(|row| &row.usage)
}

pub(crate) fn reimbursement(draft: &FederalDraft, id: i64) -> Option<i64> {
    draft
        .adjustments
        .as_ref()?
        .social_premiums
        .iter()
        .find(|row| row.document_id == id)
        .and_then(|row| row.reimbursed)
}

pub(crate) fn issue(
    draft: &FederalDraft,
    required: bool,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    let rows = draft.adjustments.as_ref().map_or(&[][..], |a| a.social_premiums.as_slice());
    let spending = crate::medical_account_return::qualified_hsa_spending(draft);
    let reimbursed_total: i128 = rows.iter().filter_map(|row| row.reimbursed).map(i128::from).sum();
    for source in draft.income.iter().filter(|entry| printed_premiums(entry) > 0) {
        let failure =
            |key| Some((source.id, crate::validation::ValidationIssue::new(key).at_field("value")));
        if spending > 0 {
            let Some(reimbursed) = reimbursement(draft, source.id) else {
                if required {
                    return failure("SOCIAL_PREMIUM_REIMBURSEMENT_NEEDED");
                }
                continue;
            };
            let in_plan = match allocation(draft, source.id) {
                Some(SocialPremiumUse::AlreadyInSelfEmployedPlan { amount, .. }) => *amount,
                _ => 0,
            };
            if i128::from(reimbursed) + i128::from(in_plan) > i128::from(printed_premiums(source))
                || reimbursed_total > spending
            {
                return failure("SOCIAL_PREMIUM_ALLOCATION_CONFLICT");
            }
        }
        match allocation(draft, source.id) {
            None if required && plans(draft).next().is_some() => {
                return failure("SOCIAL_PREMIUM_USE_NEEDED");
            }
            Some(SocialPremiumUse::AlreadyInSelfEmployedPlan { income_id, amount }) => {
                let plan = plans(draft).find(|entry| entry.id == *income_id);
                let allocated: i64 = rows
                    .iter()
                    .filter_map(|row| match row.usage {
                        SocialPremiumUse::AlreadyInSelfEmployedPlan {
                            income_id: found,
                            amount,
                        } if found == *income_id => Some(amount),
                        _ => None,
                    })
                    .sum();
                if *amount > printed_premiums(source)
                    || plan.and_then(plan_premiums).is_none_or(|total| allocated > total)
                {
                    return failure("SOCIAL_PREMIUM_ALLOCATION_CONFLICT");
                }
            }
            _ => {}
        }
    }
    None
}

// Pub. 502: the health owner already returns unused self-employed premiums to
// Schedule A. Subtract the included expense here, not the allowed deduction.
pub(crate) fn medical(draft: &FederalDraft) -> i64 {
    draft
        .income
        .iter()
        .map(|source| {
            let included = match allocation(draft, source.id) {
                Some(SocialPremiumUse::AlreadyInSelfEmployedPlan { amount, .. }) => *amount,
                _ => 0,
            };
            let reimbursed = if crate::medical_account_return::qualified_hsa_spending(draft) > 0 {
                reimbursement(draft, source.id).unwrap_or(0)
            } else {
                0
            };
            printed_premiums(source) - included - reimbursed
        })
        .sum()
}

pub(crate) fn reconcile(draft: &mut FederalDraft) {
    let retained: Vec<_> = draft
        .adjustments
        .as_ref()
        .into_iter()
        .flat_map(|a| &a.social_premiums)
        .filter(|row| {
            draft.income.iter().find(|entry| entry.id == row.document_id).is_some_and(|source| {
                printed_premiums(source) > 0
                    && match row.usage {
                        SocialPremiumUse::Medical => {
                            plans(draft).next().is_some()
                                || crate::medical_account_return::qualified_hsa_spending(draft) > 0
                        }
                        SocialPremiumUse::AlreadyInSelfEmployedPlan { income_id, .. } => {
                            plans(draft).any(|entry| entry.id == income_id)
                        }
                    }
            })
        })
        .cloned()
        .map(|mut row| {
            if crate::medical_account_return::qualified_hsa_spending(draft) == 0 {
                row.reimbursed = None;
            }
            row
        })
        .collect();
    if let Some(adjustments) = draft.adjustments.as_mut() {
        adjustments.social_premiums = retained;
    }
}

// The SSA-1099's description identifies the payment year. Prior-return AGI,
// filing status and taxable-benefit facts remain the existing election owner's
// independent inputs (Publication 915, lump-sum election Worksheets 2–4).
pub(crate) fn printed_lump_sums(
    draft: &crate::FederalDraft,
) -> alloc::collections::BTreeMap<u32, i64> {
    let mut amounts = alloc::collections::BTreeMap::new();
    for entry in &draft.income {
        let crate::FederalIncome::SocialSecurity { data } = &entry.income else {
            continue;
        };
        for line in &crate::FederalSocial::from(data).form_boxes.paid_description {
            if let crate::SocialBenefitDescriptionKind::PriorYear { year } = line.description {
                *amounts.entry(year).or_insert(0) += line.amount;
            }
        }
    }
    amounts
}

pub(crate) fn follow_printed_lump_sums(
    before: &crate::FederalDraft,
    draft: &mut crate::FederalDraft,
) {
    let old = printed_lump_sums(before);
    let current = printed_lump_sums(draft);
    let Some(adjustments) = &mut draft.adjustments else {
        return;
    };
    let crate::SocialElection::Compare { years, .. } = &mut adjustments.social_election else {
        return;
    };
    years.retain(|year| !old.contains_key(&year.year) || current.contains_key(&year.year));
    for year in years.iter_mut() {
        if let Some(amount) = current.get(&year.year) {
            year.received = *amount;
        }
    }
    if years.is_empty() {
        adjustments.social_election = crate::SocialElection::Regular;
    }
}

pub(crate) fn unallocated_railroad_years(draft: &crate::FederalDraft) -> Option<i64> {
    draft.income.iter().find_map(|entry| match &entry.income {
        crate::FederalIncome::SocialSecurity {
            data: crate::SocialBenefitStatement::Rrb1099(form),
        } if form.earlier_years > 0 && form.earlier_year_details.is_empty() => Some(entry.id),
        _ => None,
    })
}
