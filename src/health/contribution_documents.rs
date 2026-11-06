pub use crate::health_contribution_source::{
    HealthContributionAllocation, HealthContributionKind, HealthStatementDeposit,
    HealthStatementFunding, HealthStatementW2Part,
};
// Coverage: Form5498-SA (2026) retains each original custodian statement and
// projects established contribution classifications into the existing8889/8853
// owners. Original principal, payroll, qualified IRA funding, failed rollovers
// and timely returns remain separate classes, counted once.
// https://www.irs.gov/pub/irs-prior/f5498sa--2026.pdf
// https://www.irs.gov/instructions/i1099sa
use crate::kernel::source_fact::{SourceCents, SourceFact};
use crate::validation::ValidationIssue;
use crate::{FederalDraft, Owner};
use alloc::{collections::BTreeSet, string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Form 5498-SA
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form5498Sa {
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Accepted association with a person on this return
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub assigned: crate::DocumentRecipient,
    /// Calendar year
    pub calendar_year: u32,
    /// VOID
    #[libertas_default(false)]
    pub void: bool,
    /// CORRECTED
    #[libertas_default(false)]
    pub corrected: bool,
    /// Trustee's name, TIN and printed address
    pub trustee: crate::TaxPayerIdentity,
    /// Trustee's telephone number
    #[libertas_default("")]
    pub trustee_phone: String,
    /// Participant's name, TIN and printed address
    pub participant: crate::TaxPayerIdentity,
    /// Trustee's room or suite number
    #[libertas_default("")]
    pub trustee_room: String,
    /// Trustee's city or town
    #[libertas_default("")]
    pub trustee_city: String,
    /// Trustee's state or province
    #[libertas_default("")]
    pub trustee_state: String,
    /// Trustee's country
    #[libertas_default("")]
    pub trustee_country: String,
    /// Trustee's ZIP or foreign code
    #[libertas_default("")]
    pub trustee_postal_code: String,
    /// Participant's apartment number
    #[libertas_default("")]
    pub participant_apartment: String,
    /// Participant's city or town
    #[libertas_default("")]
    pub participant_city: String,
    /// Participant's state or province
    #[libertas_default("")]
    pub participant_state: String,
    /// Participant's country
    #[libertas_default("")]
    pub participant_country: String,
    /// Participant's ZIP or foreign code
    #[libertas_default("")]
    pub participant_postal_code: String,
    /// Account number
    #[libertas_default("")]
    pub account_number: String,
    /// 1 — Personal Archer MSA contributions for this year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_1: i64,
    /// 2 — Total contributions made in the calendar year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_2: i64,
    /// 3 — Subsequent-year contributions for this calendar year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_3: i64,
    /// 4 — Rollover contributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_4: i64,
    /// 5 — Year-end fair market value
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_5: i64,
    /// 6 — Account type
    pub kind: HealthContributionKind,
}

/// Health savings account statements
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each actual Form5498-SA. This is not Form5498 or Form1099-SA. Separate custodians and accounts; count duplicate copies once."
)]
pub struct HealthContributionDocuments {
    /// Documents
    #[libertas_unordered]
    pub documents: Vec<crate::HealthStatementSource>,
}

/// Actual W-2 contribution included in this custodian statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthStatementW2PartPage {
    /// Accepted W-2 identity
    #[libertas_enum_source("$.value.employer_statements")]
    pub income: i64,
    /// Current-year designated principal included in this statement
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
}
/// Qualified IRA funding included in this statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthStatementFundingPage {
    /// Actual Form1099-R payment
    #[libertas_enum_source("$.ira_sources")]
    pub income: i64,
    /// Actual funding date already entered below
    #[libertas_date_only]
    pub funded: u32,
}
/// Failed-rollover cash included in this statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthStatementDepositPage {
    /// Original Form1099-SA statement
    #[libertas_enum_source("$.value.linked_deposits")]
    pub income: i64,
    /// Actual deposit date already entered on that source
    #[libertas_date_only]
    pub deposited: u32,
}
/// Actual contribution classifications not printed on Form5498-SA
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthContributionAllocationPage {
    /// Accepted Form5498-SA identity
    #[libertas_hidden]
    #[libertas_read_only]
    pub source: i64,
    /// Part of box2 designated for the preceding tax year
    pub calendar_prior_year: SourceCents,
    /// Employer and payroll-excluded current-year principal in box2
    pub employer_calendar: SourceCents,
    /// Employer and payroll-excluded current-year principal in box3
    pub employer_later: SourceCents,
    /// Personal principal from box3 already returned through the existing timely corrections
    /// This classifies existing correction principal by custodian statement; it is not another correction or deduction.
    pub later_correction_personal: SourceCents,
    /// Employer principal from box3 already returned through the existing timely corrections
    pub later_correction_employer: SourceCents,
    /// Current-year W-2 principal included in those employer portions
    /// Select actual accepted sources and their included parts. An empty list establishes that none of these employer deposits is already represented by an entered W-2.
    pub w2: Vec<HealthStatementW2PartPage>,
    /// Qualified IRA funding included in box2
    /// Choose the actual source and date already recorded on this page; an empty list establishes none. Amounts remain owned by those transfers.
    pub funding: Vec<HealthStatementFundingPage>,
    /// Failed-rollover ordinary deposits included in boxes2 or3
    /// Choose actual accepted deposits; an empty list establishes none. The existing rollover owner adds these deposits once.
    pub failed_deposits: Vec<HealthStatementDepositPage>,
}
macro_rules! statement_link_conversions {
    ($record:ident, $page:ident, [$($field:ident),+]) => {
        impl From<$record> for $page {
            fn from(value: $record) -> Self { Self { $($field: value.$field),+ } }
        }
        impl From<$page> for $record {
            fn from(value: $page) -> Self { Self { $($field: value.$field),+ } }
        }
    };
}
statement_link_conversions!(HealthStatementW2Part, HealthStatementW2PartPage, [income, amount]);
statement_link_conversions!(HealthStatementFunding, HealthStatementFundingPage, [income, funded]);
statement_link_conversions!(
    HealthStatementDeposit,
    HealthStatementDepositPage,
    [income, deposited]
);

impl From<HealthContributionAllocation> for HealthContributionAllocationPage {
    fn from(value: HealthContributionAllocation) -> Self {
        Self {
            source: value.source,
            calendar_prior_year: value.calendar_prior_year,
            employer_calendar: value.employer_calendar,
            employer_later: value.employer_later,
            later_correction_personal: value.later_correction_personal,
            later_correction_employer: value.later_correction_employer,
            w2: value.w2.into_iter().map(Into::into).collect(),
            funding: value.funding.into_iter().map(Into::into).collect(),
            failed_deposits: value.failed_deposits.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<HealthContributionAllocationPage> for HealthContributionAllocation {
    fn from(value: HealthContributionAllocationPage) -> Self {
        Self {
            source: value.source,
            calendar_prior_year: value.calendar_prior_year,
            employer_calendar: value.employer_calendar,
            employer_later: value.employer_later,
            later_correction_personal: value.later_correction_personal,
            later_correction_employer: value.later_correction_employer,
            w2: value.w2.into_iter().map(Into::into).collect(),
            funding: value.funding.into_iter().map(Into::into).collect(),
            failed_deposits: value.failed_deposits.into_iter().map(Into::into).collect(),
        }
    }
}

/// Accepted statement and its contribution classifications
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthContributionSourceRow {
    /// Accepted source identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    #[libertas_read_only]
    pub id: i64,
    /// Original printed statement
    #[libertas_read_only]
    pub document: Form5498Sa,
    /// Actual contribution records
    pub allocation: HealthContributionAllocationPage,
}

impl Form5498Sa {
    pub(crate) fn owner(&self) -> Option<Owner> {
        self.assigned.tax_owner()
    }
    pub(crate) fn active(&self) -> bool {
        !self.void && self.calendar_year == crate::HEALTH_TAX_YEAR
    }
    fn has_contributions(&self) -> bool {
        self.box_1 != 0 || self.box_2 != 0 || self.box_3 != 0
    }
}
// One-way: every printed box either reaches the internal record or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&crate::HealthStatementSource> for Form5498Sa {
    fn from(source: &crate::HealthStatementSource) -> Self {
        let crate::HealthStatementSource { id, assigned, owner_source: _, joint_share: _, form } =
            source;
        let crate::Form5498SaPaper {
            void,
            corrected,
            trustee,
            trustee_room_suite,
            trustee_city_town,
            trustee_state_province,
            trustee_country,
            trustee_zip_foreign_code,
            trustee_telephone,
            participant,
            participant_apt_no,
            participant_city_town,
            participant_state_province,
            participant_country,
            participant_zip_foreign_code,
            account_number,
            calendar_year,
            archer_msa_contributions,
            total_contributions,
            subsequent_year_contributions,
            rollover_contributions,
            fair_market_value,
            account,
        } = form;
        Self {
            id: *id,
            assigned: *assigned,
            calendar_year: *calendar_year,
            void: *void,
            corrected: *corrected,
            trustee: trustee.clone(),
            trustee_phone: trustee_telephone.clone(),
            participant: participant.clone(),
            trustee_room: trustee_room_suite.clone(),
            trustee_city: trustee_city_town.clone(),
            trustee_state: trustee_state_province.clone(),
            trustee_country: trustee_country.clone(),
            trustee_postal_code: trustee_zip_foreign_code.clone(),
            participant_apartment: participant_apt_no.clone(),
            participant_city: participant_city_town.clone(),
            participant_state: participant_state_province.clone(),
            participant_country: participant_country.clone(),
            participant_postal_code: participant_zip_foreign_code.clone(),
            account_number: account_number.clone(),
            box_1: *archer_msa_contributions,
            box_2: *total_contributions,
            box_3: *subsequent_year_contributions,
            box_4: *rollover_contributions,
            box_5: *fair_market_value,
            kind: *account,
        }
    }
}
pub(crate) fn stored(d: &FederalDraft) -> &[crate::HealthStatementSource] {
    d.adjustments.as_ref().map_or(&[], |a| a.health_statements.as_slice())
}
pub(crate) fn sources(d: &FederalDraft) -> Vec<Form5498Sa> {
    d.adjustments
        .as_ref()
        .map_or_else(Vec::new, |a| a.health_statements.iter().map(Into::into).collect())
}
pub(crate) fn rows(d: &FederalDraft) -> Vec<HealthContributionSourceRow> {
    sources(d)
        .iter()
        .filter(|s| {
            s.active()
                && s.kind != HealthContributionKind::MedicareAdvantage
                && s.has_contributions()
        })
        .map(|s| HealthContributionSourceRow {
            id: s.id,
            document: s.clone(),
            allocation: d
                .adjustments
                .as_ref()
                .and_then(|a| a.health_contribution_allocations.iter().find(|v| v.source == s.id))
                .cloned()
                .unwrap_or_else(|| HealthContributionAllocation::new(s.id))
                .into(),
        })
        .collect()
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct StatementAmounts {
    pub personal: i64,
    pub employer_outside: i64,
    pub value: i64,
    pub later: i64,
    pub later_employer: i64,
}
#[derive(Default)]
pub(crate) struct Totals {
    pub hsa: [StatementAmounts; 2],
    pub archer: [StatementAmounts; 2],
}
fn add(target: &mut i64, amount: i64) -> Result<(), &'static str> {
    *target = target.checked_add(amount).ok_or("HEALTH_STATEMENT_TOTAL_INVALID")?;
    Ok(())
}
fn amount(value: &SourceCents, bound: i64) -> Result<i64, &'static str> {
    let amount = match value.source_fact()? {
        SourceFact::Reported(v) => v,
        SourceFact::NotReported => 0,
        SourceFact::Unavailable if bound == 0 => 0,
        SourceFact::Unavailable => return Err("HEALTH_STATEMENT_SPLIT_REQUIRED"),
    };
    if amount > bound { Err("HEALTH_STATEMENT_SPLIT_INVALID") } else { Ok(amount) }
}
pub(crate) fn project(d: &FederalDraft) -> Result<Totals, (i64, &'static str)> {
    if !sources(d).iter().any(|s| s.active() && s.kind != HealthContributionKind::MedicareAdvantage)
    {
        return Ok(Totals::default());
    }
    let mut result = Totals::default();
    let mut w2_used = alloc::collections::BTreeMap::<(i64, u8), i64>::new();
    let mut funding_used = BTreeSet::new();
    let mut deposits_used = BTreeSet::new();
    let mut corrections_used = [[0i64; 2]; 4];
    let has_contributions = sources(d).iter().any(|s| {
        s.active() && s.kind != HealthContributionKind::MedicareAdvantage && s.has_contributions()
    });
    let failed = if has_contributions {
        crate::medical_rollover_return::deposit_choices(d, None)
    } else {
        Vec::new()
    };
    let employer_sources = if has_contributions {
        crate::health_employer::page_rows(d, crate::health_employer::bindings(d))
            .map_err(|key| (0, key))?
    } else {
        Vec::new()
    };
    for statement in sources(d).iter().filter(|s| s.active()) {
        let id = statement.id;
        if statement.kind == HealthContributionKind::MedicareAdvantage {
            continue;
        }
        // Custodians may furnish an FMV-only statement with no contributions.
        // Its value participates only when an existing excess owner uses it;
        // the information alone never creates an eligibility questionnaire.
        if !statement.has_contributions() {
            if let Some(owner) = statement.owner() {
                let owner = usize::from(owner == Owner::Spouse);
                let target = if statement.kind == HealthContributionKind::Hsa {
                    &mut result.hsa[owner]
                } else {
                    &mut result.archer[owner]
                };
                add(&mut target.value, statement.box_5).map_err(|key| (id, key))?;
            }
            continue;
        }
        let mut resolve = || -> Result<StatementAmounts, &'static str> {
            let owner = statement.owner().ok_or("CHOOSE_WHOSE_DOCUMENT_THIS_IS")?;
            let a = d.adjustments.as_ref().ok_or("HEALTH_STATEMENT_ACCOUNT_REQUIRED")?;
            let hsa = statement.kind == HealthContributionKind::Hsa;
            let existing = if hsa {
                a.hsa.iter().any(|v| v.owner == owner)
            } else {
                matches!(&a.archer, crate::ArcherContributions::Records { owners, .. } if owners.iter().any(|v| v.owner == owner))
            };
            if !existing {
                return Err("HEALTH_STATEMENT_ACCOUNT_REQUIRED");
            }
            let allocation = a
                .health_contribution_allocations
                .iter()
                .find(|v| v.source == id)
                .cloned()
                .unwrap_or_else(|| HealthContributionAllocation::new(id));
            let prior = amount(&allocation.calendar_prior_year, statement.box_2)?;
            let employer_calendar = amount(&allocation.employer_calendar, statement.box_2 - prior)?;
            let employer_later = amount(&allocation.employer_later, statement.box_3)?;
            let employer = employer_calendar
                .checked_add(employer_later)
                .ok_or("HEALTH_STATEMENT_TOTAL_INVALID")?;
            let kind = if hsa {
                crate::HealthEmployerKind::Hsa
            } else {
                crate::HealthEmployerKind::Archer
            };
            let mut reported = 0;
            for part in &allocation.w2 {
                let employer_source = employer_sources
                    .iter()
                    .find(|v| v.document == part.income && v.owner == owner && v.kind == kind)
                    .ok_or("HEALTH_STATEMENT_W2_INVALID")?;
                let current = employer_source.allocation.current_part(employer_source.reported)?;
                crate::source_associations::income_source(d, part.income)
                    .map_err(|_| "HEALTH_STATEMENT_W2_INVALID")?;
                let used = w2_used.entry((part.income, u8::from(!hsa))).or_default();
                add(used, part.amount)?;
                if *used > current {
                    return Err("HEALTH_STATEMENT_W2_INVALID");
                }
                add(&mut reported, part.amount)?;
            }
            if reported > employer {
                return Err("HEALTH_STATEMENT_SPLIT_INVALID");
            }
            let mut qualified = 0;
            for link in &allocation.funding {
                if !hsa || !funding_used.insert((link.income, link.funded)) {
                    return Err("HEALTH_STATEMENT_FUNDING_INVALID");
                }
                let mut transfers = a
                    .hsa
                    .iter()
                    .filter(|h| h.owner == owner)
                    .flat_map(|h| &h.funding)
                    .filter(|f| f.income == link.income && f.funded == link.funded);
                let transfer = transfers.next().ok_or("HEALTH_STATEMENT_FUNDING_INVALID")?;
                if transfers.next().is_some() {
                    return Err("HEALTH_STATEMENT_FUNDING_INVALID");
                }
                crate::source_associations::income_source(d, link.income)
                    .map_err(|_| "HEALTH_STATEMENT_FUNDING_INVALID")?;
                add(&mut qualified, transfer.amount)?;
            }
            let mut linked_calendar = 0;
            let mut linked_later = 0;
            for link in &allocation.failed_deposits {
                if !deposits_used.insert((link.income, link.deposited, u8::from(!hsa))) {
                    return Err("HEALTH_STATEMENT_DEPOSIT_INVALID");
                }
                let source = failed
                    .iter()
                    .find(|v| v.id == link.income && v.owner == owner)
                    .ok_or("HEALTH_STATEMENT_DEPOSIT_INVALID")?;
                let destination = if hsa {
                    crate::MedicalRolloverDestination::Hsa
                } else {
                    crate::MedicalRolloverDestination::Archer
                };
                let deposit = source
                    .deposits
                    .iter()
                    .find(|v| {
                        v.deposited == link.deposited
                            && v.year == crate::HEALTH_TAX_YEAR
                            && v.destination == destination
                    })
                    .ok_or("HEALTH_STATEMENT_DEPOSIT_INVALID")?;
                match deposit.deposited / 10000 {
                    2026 => add(&mut linked_calendar, deposit.amount)?,
                    2027 => add(&mut linked_later, deposit.amount)?,
                    _ => return Err("HEALTH_STATEMENT_DEPOSIT_INVALID"),
                }
            }
            let calendar = statement
                .box_2
                .checked_sub(prior)
                .and_then(|v| v.checked_sub(employer_calendar))
                .and_then(|v| v.checked_sub(qualified))
                .and_then(|v| v.checked_sub(linked_calendar))
                .filter(|v| *v >= 0)
                .ok_or("HEALTH_STATEMENT_SPLIT_INVALID")?;
            let later_personal = statement
                .box_3
                .checked_sub(employer_later)
                .and_then(|v| v.checked_sub(linked_later))
                .filter(|v| *v >= 0)
                .ok_or("HEALTH_STATEMENT_SPLIT_INVALID")?;
            if !hsa {
                let gross_for_year = statement
                    .box_2
                    .checked_sub(prior)
                    .and_then(|v| v.checked_add(statement.box_3))
                    .ok_or("HEALTH_STATEMENT_TOTAL_INVALID")?;
                if statement.box_1.checked_add(employer).is_none_or(|total| total > gross_for_year)
                {
                    return Err("HEALTH_STATEMENT_SPLIT_INVALID");
                }
            }
            let personal = if hsa {
                calendar.checked_add(later_personal).ok_or("HEALTH_STATEMENT_TOTAL_INVALID")?
            } else {
                statement
                    .box_1
                    .checked_sub(linked_calendar)
                    .and_then(|v| v.checked_sub(linked_later))
                    .filter(|v| *v >= 0)
                    .ok_or("HEALTH_STATEMENT_SPLIT_INVALID")?
            };
            let owner_index = usize::from(owner == Owner::Spouse);
            let corrections: [i64; 2] = if hsa {
                let account = a.hsa.iter().find(|v| v.owner == owner).unwrap();
                let mut totals = [0, 0];
                for correction in &account.corrections {
                    if correction.failed_deposit.is_none()
                        && correction.year == crate::HEALTH_TAX_YEAR
                        && correction.contributed_on / 10000 == 2027
                    {
                        add(
                            &mut totals[usize::from(matches!(
                                correction.payer,
                                crate::HsaCorrectionPayer::Employer { .. }
                            ))],
                            correction.principal,
                        )?;
                    }
                }
                totals
            } else {
                let crate::ArcherContributions::Records { owners, .. } = &a.archer else {
                    unreachable!()
                };
                let account = owners.iter().find(|v| v.owner == owner).unwrap();
                let mut totals = [0, 0];
                for correction in &account.corrections {
                    if correction.failed_deposit.is_none()
                        && correction.year == crate::HEALTH_TAX_YEAR
                        && correction.contributed_on / 10000 == 2027
                    {
                        add(
                            &mut totals[usize::from(matches!(
                                correction.payer,
                                crate::ArcherCorrectionPayer::Employer { .. }
                            ))],
                            correction.principal,
                        )?;
                    }
                }
                totals
            };
            let returned_personal = amount(
                &allocation.later_correction_personal,
                if statement.box_3 == 0 { 0 } else { corrections[0] },
            )?;
            let returned_employer = amount(
                &allocation.later_correction_employer,
                if statement.box_3 == 0 { 0 } else { corrections[1] },
            )?;
            let used = &mut corrections_used[owner_index + if hsa { 0 } else { 2 }];
            for index in 0..2 {
                add(&mut used[index], [returned_personal, returned_employer][index])?;
                if used[index] > corrections[index] {
                    return Err("HEALTH_STATEMENT_SPLIT_INVALID");
                }
            }
            if returned_personal > later_personal || returned_employer > employer_later {
                return Err("HEALTH_STATEMENT_SPLIT_INVALID");
            }
            Ok(StatementAmounts {
                personal,
                employer_outside: employer - reported,
                value: statement.box_5,
                later: statement.box_3 - linked_later - returned_personal - returned_employer,
                later_employer: employer_later - returned_employer,
            })
        };
        let projected = resolve().map_err(|key| (id, key))?;
        let owner = usize::from(statement.owner() == Some(Owner::Spouse));
        let target = if statement.kind == HealthContributionKind::Hsa {
            &mut result.hsa[owner]
        } else {
            &mut result.archer[owner]
        };
        for (to, from) in [
            (&mut target.personal, projected.personal),
            (&mut target.employer_outside, projected.employer_outside),
            (&mut target.value, projected.value),
            (&mut target.later, projected.later),
            (&mut target.later_employer, projected.later_employer),
        ] {
            add(to, from).map_err(|key| (id, key))?;
        }
    }
    Ok(result)
}

pub(crate) fn pending(d: &FederalDraft) -> bool {
    project(d).is_err()
}
pub(crate) fn has_statements(d: &FederalDraft, owner: Owner, kind: HealthContributionKind) -> bool {
    sources(d)
        .iter()
        .any(|s| s.active() && s.kind == kind && s.owner() == Some(owner) && s.has_contributions())
}

pub(crate) fn personal_input(
    d: &FederalDraft,
    owner: Owner,
    kind: HealthContributionKind,
    personal: i64,
    outside: i64,
) -> i64 {
    if has_statements(d, owner, kind) { outside } else { personal }
}

// The original manual amount remains authoritative until a custodian source
// participates. First import retains that amount as the outside partition;
// later corrections and removal keep that same independent partition.
pub(crate) fn reconcile(previous: &FederalDraft, d: &mut FederalDraft) {
    if let Some(a) = &mut d.adjustments {
        a.health_contribution_allocations.retain(|v| {
            a.health_statements.iter().any(|s| s.id == v.source && Form5498Sa::from(s).active())
        });
    }
    let current_hsa = [Owner::Taxpayer, Owner::Spouse]
        .map(|owner| has_statements(d, owner, HealthContributionKind::Hsa));
    let current_archer = [Owner::Taxpayer, Owner::Spouse]
        .map(|owner| has_statements(d, owner, HealthContributionKind::Archer));
    let previous_hsa = [Owner::Taxpayer, Owner::Spouse]
        .map(|owner| has_statements(previous, owner, HealthContributionKind::Hsa));
    let previous_archer = [Owner::Taxpayer, Owner::Spouse]
        .map(|owner| has_statements(previous, owner, HealthContributionKind::Archer));
    let totals = project(d).ok();
    if let Some(a) = &mut d.adjustments {
        for h in &mut a.hsa {
            let owner = usize::from(h.owner == Owner::Spouse);
            if !previous_hsa[owner] && !current_hsa[owner] {
                h.personal_outside = h.personal;
                continue;
            }
            if !previous_hsa[owner] && current_hsa[owner] {
                h.personal_outside = previous
                    .adjustments
                    .as_ref()
                    .and_then(|a| a.hsa.iter().find(|v| v.owner == h.owner))
                    .map_or(h.personal_outside, |v| v.personal);
            }
            let source = totals.as_ref().map_or(0, |t| t.hsa[owner].personal);
            if let Some(total) = h.personal_outside.checked_add(source) {
                h.personal = total;
            }
        }
        if let crate::ArcherContributions::Records { owners, .. } = &mut a.archer {
            for h in owners {
                let owner = usize::from(h.owner == Owner::Spouse);
                if !previous_archer[owner] && !current_archer[owner] {
                    h.personal_outside = h.personal;
                    continue;
                }
                if !previous_archer[owner] && current_archer[owner] {
                    h.personal_outside = previous
                        .adjustments
                        .as_ref()
                        .and_then(|a| {
                            let crate::ArcherContributions::Records { owners, .. } = &a.archer
                            else {
                                return None;
                            };
                            owners.iter().find(|v| v.owner == h.owner)
                        })
                        .map_or(h.personal_outside, |v| v.personal);
                }
                let source = totals.as_ref().map_or(0, |t| t.archer[owner].personal);
                if let Some(total) = h.personal_outside.checked_add(source) {
                    h.personal = total;
                }
            }
        }
    }
}
pub(crate) fn accept(
    d: &FederalDraft,
    rows: &[HealthContributionSourceRow],
) -> Result<Vec<HealthContributionAllocation>, ValidationIssue> {
    let expected = self::rows(d);
    if expected.len() != rows.len() {
        return Err(ValidationIssue::new("HEALTH_STATEMENT_SOURCES_CHANGED")
            .at_field("contribution_sources"));
    }
    let mut seen = BTreeSet::new();
    rows.iter()
        .enumerate()
        .map(|(index, row)| {
            if !expected.iter().any(|v| v.id == row.id) || !seen.insert(row.id) {
                return Err(ValidationIssue::new("HEALTH_STATEMENT_SOURCES_CHANGED")
                    .at_field("id")
                    .in_list("contribution_sources", index));
            }
            let mut allocation: HealthContributionAllocation = row.allocation.clone().into();
            allocation.source = row.id;
            Ok(allocation)
        })
        .collect()
}

pub(crate) fn derived_rule(d: &FederalDraft) -> Option<&'static str> {
    let a = d.adjustments.as_ref()?;
    let totals = project(d).ok()?;
    for h in &a.hsa {
        if !has_statements(d, h.owner, HealthContributionKind::Hsa) {
            continue;
        }
        let source = totals.hsa[usize::from(h.owner == Owner::Spouse)].personal;
        if h.personal_outside.checked_add(source) != Some(h.personal) {
            return Some("HEALTH_STATEMENT_TOTAL_INVALID");
        }
    }
    if let crate::ArcherContributions::Records { owners, .. } = &a.archer {
        for h in owners {
            if !has_statements(d, h.owner, HealthContributionKind::Archer) {
                continue;
            }
            let source = totals.archer[usize::from(h.owner == Owner::Spouse)].personal;
            if h.personal_outside.checked_add(source) != Some(h.personal) {
                return Some("HEALTH_STATEMENT_TOTAL_INVALID");
            }
        }
    }
    None
}
