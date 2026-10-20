// Form 8379 ownership facts stay with the source whose joint calculation they
// explain. They do not change that calculation or create another income record.
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Actual owners of this retained source amount
/// Use the retained records. This answer never changes the joint amount or applies a proportional allocation.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredRetainedOwners {
    /// Taxpayer
    Taxpayer,
    /// Spouse
    Spouse,
    /// Both spouses, with these actual amounts
    Both {
        /// Taxpayer's actual amount from the retained records
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 1)]
        taxpayer: i64,
        /// Spouse's actual amount from the retained records
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 1)]
        spouse: i64,
    },
}

impl InjuredRetainedOwners {
    pub(crate) fn amounts(&self, total: i64) -> [i64; 2] {
        match *self {
            Self::Taxpayer => crate::kernel::attribution::exclusive(total, crate::Owner::Taxpayer),
            Self::Spouse => crate::kernel::attribution::exclusive(total, crate::Owner::Spouse),
            Self::Both { taxpayer, spouse } => [taxpayer, spouse],
        }
    }
    pub(crate) fn valid(&self, total: i64) -> bool {
        let amounts = self.amounts(total);
        (0..=100000000000).contains(&total)
            && amounts.iter().all(|n| (0..=100000000000).contains(n))
            && amounts[0] + amounts[1] == total
            && !matches!(self, Self::Both { taxpayer, spouse } if *taxpayer == 0 || *spouse == 0)
    }
}

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

/// Ownership of the surviving joint capital loss
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredCapitalLossOwners {
    /// Actual surviving amounts from each spouse's loss records
    Shares {
        /// Taxpayer's short-term loss carried to 2026
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        taxpayer_short: i64,
        /// Spouse's short-term loss carried to 2026
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        spouse_short: i64,
        /// Taxpayer's long-term loss carried to 2026
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        taxpayer_long: i64,
        /// Spouse's long-term loss carried to 2026
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        spouse_long: i64,
    },
}
impl InjuredCapitalLossOwners {
    pub(crate) fn amounts(&self) -> [[i64; 2]; 2] {
        match *self {
            Self::Shares { taxpayer_short, spouse_short, taxpayer_long, spouse_long } => {
                [[taxpayer_short, taxpayer_long], [spouse_short, spouse_long]]
            }
        }
    }
    pub(crate) fn valid(&self, carry: [i64; 2]) -> bool {
        let amounts = self.amounts();
        amounts.iter().flatten().all(|n| (0..=100000000000).contains(n))
            && [0, 1].iter().all(|i| amounts[0][*i] + amounts[1][*i] == carry[*i])
    }
}

/// Ownership of a surviving regular net operating loss
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredNolOwners {
    /// Actual surviving balances belonging to each spouse
    Shares {
        /// Taxpayer's unused regular loss from this year
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        taxpayer: i64,
        /// Spouse's unused regular loss from this year
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        spouse: i64,
    },
}
impl InjuredNolOwners {
    pub(crate) fn amounts(&self) -> [i64; 2] {
        match *self {
            Self::Shares { taxpayer, spouse } => [taxpayer, spouse],
        }
    }
    pub(crate) fn valid(&self, balance: i64) -> bool {
        let amounts = self.amounts();
        amounts.iter().all(|n| (0..=100000000000).contains(n)) && amounts[0] + amounts[1] == balance
    }
}

/// Ownership of retained qualified business income losses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredQbiLossOwners {
    /// Actual amounts belonging to each spouse
    Shares {
        /// Taxpayer's retained business QBI loss
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        taxpayer_business: i64,
        /// Spouse's retained business QBI loss
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        spouse_business: i64,
        /// Taxpayer's retained REIT/PTP QBI loss
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        taxpayer_reit_ptp: i64,
        /// Spouse's retained REIT/PTP QBI loss
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        spouse_reit_ptp: i64,
    },
}
impl InjuredQbiLossOwners {
    pub(crate) fn amounts(&self) -> [[i64; 2]; 2] {
        match *self {
            Self::Shares {
                taxpayer_business,
                spouse_business,
                taxpayer_reit_ptp,
                spouse_reit_ptp,
            } => [[taxpayer_business, taxpayer_reit_ptp], [spouse_business, spouse_reit_ptp]],
        }
    }
    pub(crate) fn valid(&self, losses: [i64; 2]) -> bool {
        let amounts = self.amounts();
        amounts.iter().flatten().all(|n| (0..=100000000000).contains(n))
            && [0, 1].iter().all(|i| amounts[0][*i] + amounts[1][*i] == losses[*i])
    }
}

/// Actual source payments in a long-term-care period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredLtcPeriodPayees {
    /// Payments identified by their existing 1099-LTC source
    Payments {
        /// Each source's actual periodic payment in these dates
        #[libertas_size(min = 1)]
        payments: Vec<InjuredLtcPeriodPayment>,
    },
}
impl InjuredLtcPeriodPayees {
    pub(crate) fn payments(&self) -> &[InjuredLtcPeriodPayment] {
        match self {
            Self::Payments { payments } => payments,
        }
    }
    pub(crate) fn valid(&self, received: i64) -> bool {
        self.issue(received).is_none()
    }
    pub(crate) fn issue(&self, received: i64) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let payments = self.payments();
        if payments.is_empty() {
            return Some(ValidationIssue::new("INJURED_LTC_PAYEE_TOTAL").at_field("payments"));
        }
        for (index, p) in payments.iter().enumerate() {
            if p.source <= 0 {
                return Some(
                    ValidationIssue::new("INJURED_LTC_PAYEE_TOTAL")
                        .at_field("source")
                        .in_list("payments", index),
                );
            }
            if p.amount <= 0 {
                return Some(
                    ValidationIssue::new("INJURED_LTC_PAYEE_TOTAL")
                        .at_field("amount")
                        .in_list("payments", index),
                );
            }
            if payments[..index].iter().any(|v| v.source == p.source) {
                return Some(
                    ValidationIssue::new("INJURED_LTC_SOURCE_TWICE")
                        .at_field("source")
                        .in_list("payments", index),
                );
            }
        }
        (crate::kernel::ratio::checked_sum(payments.iter().map(|p| p.amount)) != Some(received))
            .then_some(ValidationIssue::new("INJURED_LTC_PAYEE_TOTAL").at_field("payments"))
    }
}

/// A payment from an existing long-term-care statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InjuredLtcPeriodPayment {
    /// The statement whose policyholder received this payment
    pub source: i64,
    /// Actual periodic payment received in these dates
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub amount: i64,
}

/// Separate-return claimants of the prior-year care recipients
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredPriorCareClaimants {
    /// The actual qualifying people and their claimants
    People {
        /// People whose 2025 care was first paid in 2026
        #[libertas_size(min = 1)]
        people: Vec<InjuredPriorCarePerson>,
    },
}
impl InjuredPriorCareClaimants {
    pub(crate) fn people(&self) -> &[InjuredPriorCarePerson] {
        match self {
            Self::People { people } => people,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        let people = self.people();
        (1..=100).contains(&people.len()) && people.iter().enumerate().all(|(i,p)| {
            p.person.valid() && (1..=100000000000).contains(&p.expense) && !people[..i].iter().any(|v| matches!((&v.person, &p.person), (InjuredPriorCareIdentity::OnReturn { person: a }, InjuredPriorCareIdentity::OnReturn { person: b }) if a == b))
        })
    }
}

/// One actual prior-year qualifying person
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InjuredPriorCarePerson {
    /// The qualifying person
    pub person: InjuredPriorCareIdentity,
    /// Spouse who would claim this person on separate 2025 returns
    pub claimant: crate::Owner,
    /// Actual 2025 qualifying expenses for this person first paid in 2026
    /// From the care and payment records. Include only the expenses in this retained worksheet's paid-in-2026 total.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub expense: i64,
}

/// Identity of a prior-year qualifying person
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredPriorCareIdentity {
    /// A person already on this return
    OnReturn {
        /// Person
        person: i64,
    },
    /// A person not on this year's return
    Other {
        /// Name from the prior-year care records
        #[libertas_size(min = 1)]
        name: String,
    },
}
impl InjuredPriorCareIdentity {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::OnReturn { person } => *person != 0,
            Self::Other { name } => !name.trim().is_empty(),
        }
    }
}

/// Actual source payments in a long-term-care period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredLtcPeriodPayeesInput {
    /// Payments identified by their existing 1099-LTC source
    Payments {
        /// Each source's actual periodic payment in these dates
        #[libertas_size(min = 1)]
        payments: Vec<InjuredLtcPeriodPaymentInput>,
    },
}

/// A payment from an existing long-term-care statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InjuredLtcPeriodPaymentInput {
    /// The statement whose policyholder received this payment
    #[libertas_enum_source("$.ltc_payee_sources")]
    pub source: i64,
    /// Actual periodic payment received in these dates
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub amount: i64,
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
    #[libertas_number(min = 1)]
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
    document: &crate::IncomeDocument,
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    let issue = || ValidationIssue::new("INJURED_SOURCE_FACTS_NOT_REQUESTED");
    match document {
        crate::IncomeDocument::CapitalLossCarryover(v) => {
            if v.injured_owners.is_some() {
                Some(issue().at_field("injured_owners"))
            } else {
                v.injured_niit_owners.is_some().then(|| issue().at_field("injured_niit_owners"))
            }
        }
        crate::IncomeDocument::Section1256Carryback(v) => {
            v.injured_owners.is_some().then(|| issue().at_field("injured_owners"))
        }
        crate::IncomeDocument::QbiLossCarryforward(v) => {
            v.injured_owners.is_some().then(|| issue().at_field("injured_owners"))
        }
        crate::IncomeDocument::NetOperatingLoss(v) => v
            .regular
            .iter()
            .position(|y| y.injured_owners.is_some())
            .map(|index| issue().at_field("injured_owners").in_list("regular", index)),
        crate::IncomeDocument::LtcInsured(v) => {
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

pub(crate) fn clear_document(document: &mut crate::IncomeDocument) {
    match document {
        crate::IncomeDocument::CapitalLossCarryover(v) => {
            v.injured_owners = None;
            v.injured_niit_owners = None;
        }
        crate::IncomeDocument::Section1256Carryback(v) => v.injured_owners = None,
        crate::IncomeDocument::QbiLossCarryforward(v) => v.injured_owners = None,
        crate::IncomeDocument::NetOperatingLoss(v) => {
            for year in &mut v.regular {
                year.injured_owners = None;
            }
        }
        crate::IncomeDocument::LtcInsured(v) => {
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
        let sources = crate::care_benefits::payments(d, entry.id);
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
                    .flat_map(InjuredLtcPeriodPayees::payments)
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

impl From<InjuredLtcPeriodPayeesInput> for InjuredLtcPeriodPayees {
    fn from(value: InjuredLtcPeriodPayeesInput) -> Self {
        match value {
            InjuredLtcPeriodPayeesInput::Payments { payments } => Self::Payments {
                payments: payments
                    .into_iter()
                    .map(|p| InjuredLtcPeriodPayment { source: p.source, amount: p.amount })
                    .collect(),
            },
        }
    }
}
impl From<InjuredLtcPeriodPayees> for InjuredLtcPeriodPayeesInput {
    fn from(value: InjuredLtcPeriodPayees) -> Self {
        match value {
            InjuredLtcPeriodPayees::Payments { payments } => Self::Payments {
                payments: payments
                    .into_iter()
                    .map(|p| InjuredLtcPeriodPaymentInput { source: p.source, amount: p.amount })
                    .collect(),
            },
        }
    }
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
