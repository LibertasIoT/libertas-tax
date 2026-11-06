pub use crate::prior_care_identity_source::{
    InjuredPriorCareClaimants, InjuredPriorCareIdentity, InjuredPriorCarePerson,
};
pub use crate::retained_loss_owners_model::{
    InjuredCapitalLossOwners, InjuredNolOwners, InjuredQbiLossOwners, InjuredRetainedOwners,
};
// Form 8379 ownership facts stay with the source whose joint calculation they
// explain. They do not change that calculation or create another income record.
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

pub(crate) fn retained_choices(d: &crate::FederalDraft) -> Vec<i32> {
    if requested(d) { alloc::vec![0, 1, 2] } else { Vec::new() }
}

pub(crate) fn niit_owners_valid(prior: &crate::PriorCapitalReturn) -> bool {
    let Some(owners) = &prior.injured_niit_owners else {
        return true;
    };
    let total = prior.niit_difference.unwrap_or(0);
    if !owners.valid(total) {
        return false;
    }
    let carry = crate::extended_rules::regular_capital_carry(prior).map(|v| v * 100);
    let available = match prior.owners {
        crate::PriorReturnOwners::Taxpayer => [carry.iter().sum(), 0],
        crate::PriorReturnOwners::Spouse => [0, carry.iter().sum()],
        crate::PriorReturnOwners::Joint => {
            if total == 0 {
                return true;
            }
            let Some(parts) = &prior.injured_owners else {
                return false;
            };
            if !parts.valid(carry) {
                return false;
            }
            parts.amounts().map(|parts| parts.iter().sum())
        }
    };
    owners.amounts(total).into_iter().zip(available).all(|(part, cap)| part <= cap)
}

/// Separate-return claimants of the prior-year care recipients
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredPriorCareClaimantsInput {
    /// The actual qualifying people and their claimants
    People {
        /// People whose 2025 care was first paid in 2026
        #[libertas_size(min = 1)]
        people: Vec<InjuredPriorCarePersonInput>,
    },
}

/// One actual prior-year qualifying person
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InjuredPriorCarePersonInput {
    /// The qualifying person
    pub person: InjuredPriorCareIdentityInput,
    /// Spouse who would claim this person on separate 2025 returns
    pub claimant: crate::Owner,
    /// Actual 2025 qualifying expenses for this person first paid in 2026
    /// From the care and payment records. Include only the expenses in this retained worksheet's paid-in-2026 total.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub expense: i64,
}

/// Identity of a prior-year qualifying person
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredPriorCareIdentityInput {
    /// A person already on this return
    OnReturn {
        /// Person
        #[libertas_enum_source("$.people")]
        person: i64,
    },
    /// A person not on this year's return
    Other {
        /// Name from the prior-year care records
        #[libertas_size(min = 1)]
        name: String,
    },
}

pub(crate) fn requested(d: &crate::FederalDraft) -> bool {
    d.setup.as_ref().is_some_and(|s| s.filing_status() == crate::FederalStatus::Joint)
        && (d.injured_spouse.is_some() || crate::interview::injured_chosen(d))
}

pub(crate) fn allowed(d: &crate::FederalDraft) -> Vec<i32> {
    if requested(d) { alloc::vec![0] } else { Vec::new() }
}

pub(crate) fn document_facts_issue(
    document: &crate::IncomeSource,
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    let issue = || ValidationIssue::new("INJURED_SOURCE_FACTS_NOT_REQUESTED");
    match document {
        crate::IncomeSource::Record { value: crate::IncomeRecord::CapitalLossCarryover(v) } => {
            if v.injured_owners.is_some() {
                Some(issue().at_field("injured_owners"))
            } else {
                v.injured_niit_owners.is_some().then(|| issue().at_field("injured_niit_owners"))
            }
        }
        crate::IncomeSource::Record { value: crate::IncomeRecord::Section1256Carryback(v) } => {
            v.injured_owners.is_some().then(|| issue().at_field("injured_owners"))
        }
        crate::IncomeSource::Record { value: crate::IncomeRecord::QbiLossCarryforward(v) } => {
            v.injured_owners.is_some().then(|| issue().at_field("injured_owners"))
        }
        crate::IncomeSource::Record { value: crate::IncomeRecord::NetOperatingLoss(v) } => v
            .regular
            .iter()
            .position(|y| y.injured_owners.is_some())
            .map(|index| issue().at_field("injured_owners").in_list("regular", index)),
        crate::IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(v) } => {
            if let crate::care_benefits::LtcPeriodsInput::Several { periods, .. } = &v.periods {
                periods.iter().position(|p| p.injured_payees.is_some()).map(|index| {
                    issue().at_field("injured_payees").in_list("periods", index).in_field("periods")
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(crate) fn clear_document(document: &mut crate::IncomeSource) {
    match document {
        crate::IncomeSource::Record { value: crate::IncomeRecord::CapitalLossCarryover(v) } => {
            v.injured_owners = None;
            v.injured_niit_owners = None;
        }
        crate::IncomeSource::Record { value: crate::IncomeRecord::Section1256Carryback(v) } => {
            v.injured_owners = None
        }
        crate::IncomeSource::Record { value: crate::IncomeRecord::QbiLossCarryforward(v) } => {
            v.injured_owners = None
        }
        crate::IncomeSource::Record { value: crate::IncomeRecord::NetOperatingLoss(v) } => {
            for year in &mut v.regular {
                year.injured_owners = None;
            }
        }
        crate::IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(v) } => {
            if let crate::care_benefits::LtcPeriodsInput::Several { periods, .. } = &mut v.periods {
                for period in periods {
                    period.injured_payees = None;
                }
            }
        }
        _ => {}
    }
}

// Ordinary checklist pruning removes answers to a claim that is no longer
// requested. No source amount, period, person or ordinary calculation changes.
pub(crate) fn clear_unused(d: &mut crate::FederalDraft) {
    if requested(d) {
        return;
    }
    if let Some(screening) = &mut d.screening {
        for prior in &mut screening.capital_history {
            prior.injured_owners = None;
            prior.injured_niit_owners = None;
        }
        screening.section_1256_carryback_owners = None;
        screening.qbi_history.injured_owners = None;
        if let crate::NolHistory::Records { years, .. } = &mut screening.nol {
            for year in years {
                year.injured_owners = None;
            }
        }
    }
    for entry in &mut d.income {
        if let crate::FederalIncome::LtcInsured { data } = &mut entry.income
            && let crate::LtcPeriods::Several { periods, .. } = &mut data.periods
        {
            for period in periods {
                period.injured_payees = None;
            }
        }
    }
    if let Some(prior) = d.credits.as_mut().and_then(|c| c.care.prior_year.as_mut()) {
        prior.injured_claimants = None;
    }
}

pub(crate) fn reference_issue(
    d: &crate::FederalDraft,
) -> Option<(Option<i64>, crate::validation::ValidationIssue)> {
    use crate::validation::ValidationIssue;
    for entry in &d.income {
        let crate::FederalIncome::LtcInsured { data } = &entry.income else {
            continue;
        };
        let crate::LtcPeriods::Several { periods, .. } = &data.periods else {
            continue;
        };
        let sources = crate::care_benefit_return::payments(d, entry.id);
        for (period_index, period) in periods.iter().enumerate() {
            let Some(payees) = &period.injured_payees else {
                continue;
            };
            if !payees.valid(period.received) {
                return Some((
                    Some(entry.id),
                    ValidationIssue::new("INJURED_LTC_PAYEE_TOTAL")
                        .at_field("injured_payees")
                        .in_list("periods", period_index)
                        .in_field("periods"),
                ));
            }
            if let Some(index) = payees
                .payments()
                .iter()
                .position(|p| !sources.iter().any(|(source, _)| source.id == p.source))
            {
                return Some((
                    Some(entry.id),
                    ValidationIssue::new("INJURED_LTC_SOURCE_UNAVAILABLE")
                        .at_field("source")
                        .in_list("payments", index)
                        .in_field("injured_payees")
                        .in_list("periods", period_index)
                        .in_field("periods"),
                ));
            }
        }
        let complete = periods.iter().all(|p| p.received == 0 || p.injured_payees.is_some());
        for (source, annual) in &sources {
            let entered = crate::kernel::ratio::checked_sum(
                periods
                    .iter()
                    .filter_map(|p| p.injured_payees.as_ref())
                    .flat_map(crate::InjuredLtcPeriodPayees::payments)
                    .filter(|p| p.source == source.id)
                    .map(|p| p.amount),
            );
            if entered.is_none_or(|entered| entered > *annual || complete && entered != *annual) {
                return Some((
                    Some(entry.id),
                    ValidationIssue::new("INJURED_LTC_SOURCE_TOTAL")
                        .at_field("periods")
                        .in_field("periods"),
                ));
            }
        }
    }
    if let Some(claimants) = d
        .credits
        .as_ref()
        .and_then(|c| c.care.prior_year.as_ref())
        .and_then(|p| p.injured_claimants.as_ref())
    {
        if !claimants.valid() {
            return Some((None, ValidationIssue::new("INJURED_PRIOR_CARE_PERSON_INVALID")));
        }
        if claimants.people().iter().any(|p| match &p.person {
            InjuredPriorCareIdentity::OnReturn { person: -1 } => d.people.is_none(),
            InjuredPriorCareIdentity::OnReturn { person: -2 } => {
                d.people.as_ref().is_none_or(|p| p.spouse.is_none())
            }
            InjuredPriorCareIdentity::OnReturn { person } => {
                !d.dependents().iter().any(|p| p.id == *person)
            }
            InjuredPriorCareIdentity::Other { .. } => false,
        }) {
            return Some((None, ValidationIssue::new("INJURED_PRIOR_CARE_PERSON_INVALID")));
        }
    }
    None
}

pub(crate) fn reference_rule(d: &crate::FederalDraft) -> Option<&'static str> {
    reference_issue(d).map(|(_, issue)| issue.key)
}

record_page_conversions!(
    InjuredPriorCareClaimants,
    InjuredPriorCareClaimantsInput,
    [],
    |value, Target, Source| match value {
        Source::People { people } =>
            Target::People { people: people.into_iter().map(Into::into).collect() },
    }
);
record_page_conversions!(
    InjuredPriorCarePerson,
    InjuredPriorCarePersonInput,
    [],
    |value, Target| Target {
        claimant: value.claimant,
        expense: value.expense,
        person: value.person.into(),
    }
);
record_page_conversions!(
    InjuredPriorCareIdentity,
    InjuredPriorCareIdentityInput,
    [],
    |value, Target, Source| match value {
        Source::OnReturn { person } => Target::OnReturn { person },
        Source::Other { name } => Target::Other { name },
    }
);
