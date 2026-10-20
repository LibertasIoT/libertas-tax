use crate::validation::ValidationIssue;
// Indirect medical-account rollovers use the existing statement and contribution
// owners. Section223(f)(5) counts prior successful HSA-source receipts; section
//220(f)(5) separately counts Archer-source receipts. Current-year history is
// recomputed across accepted sources, never from editor order or a second ledger.
use crate::kernel::ratio::exact::{Exact, whole};
use crate::{FederalDraft, FederalIncome, MedicalSaAccount, MedicalSaPayment, Owner};
use alloc::{collections::BTreeMap, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Actual indirect-rollover source receipts
/// Keep one actual annual statement as one record. Separate source receipts and deposit dates only where they differ. Direct trustee transfers and IRA-to-HSA funding are different transactions and do not belong here.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MedicalRollover {
    /// Last successful prior-year rollover receipt from this source account type
    /// The actual most recent tax-free receipt before 2026, across all this owner's HSAs or, separately, Archer MSAs. Failed attempts, direct transfers and IRA-to-HSA funding do not count. Leave empty when no prior successful receipt can affect 2026. Use the same prior-year history on every statement for this owner/source type; the App derives 2026 history from actual entries.
    /// [NullText]
    /// No successful prior-year rollover affects 2026
    /// [SetText]
    /// The last relevant successful prior-year rollover receipt was on
    #[libertas_date_only]
    // Tax structure: retained prior-year history ends before the current return year.
    #[libertas_number(max = 20251231)]
    pub preceding: Option<u32>,
    /// Actual source receipts on this statement
    /// ----
    /// Receipt
    #[libertas_size(min = 1)]
    pub receipts: Vec<MedicalRolloverReceipt>,
}

/// One actual indirect-rollover source receipt
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MedicalRolloverReceipt {
    /// Actual distribution receipt date
    /// This is the source payment date for additional-tax exceptions. Do not combine separately dated payments into the last date on an annual form.
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
    pub received: u32,
    /// Actual destination deposits from this receipt
    /// Together with medical spending and other dated nonmedical uses, all deposits reconcile to this statement's gross. Separate day 60 from day 61; no second receipt total is entered.
    /// ----
    /// Deposit
    #[libertas_size(min = 1)]
    pub deposits: Vec<MedicalRolloverDeposit>,
}

/// Actual destination account type
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum MedicalRolloverDestination {
    /// Same owner's HSA
    Hsa,
    /// Same owner's Archer MSA
    Archer,
}

/// Actual deposit if rollover exclusion is unavailable
/// This records actual custodian treatment, not a request to invent a contribution designation. A failed cash transfer can use ordinary contribution room; it is not automatically all excess. Property is not an ordinary cash contribution.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MedicalDepositCharacter {
    /// Not established; leave only if rollover relief applies
    Unconfirmed,
    /// Actual ordinary cash contribution designation
    Cash {
        /// Custodian-accepted ordinary contribution year
        /// Use the actual designation, including 2025 or 2027 where applicable. Do not infer it from the source year or intended rollover label. Only 2026-designated cash enters current contribution limits.
        #[libertas_number(min = 1997)]
        year: u32,
        /// Actual applicable ordinary contribution deadline
        /// Normally April 15 of the following year. A later date needs actual applicable IRS relief; an income-tax filing extension alone does not extend ordinary contribution timing.
        #[libertas_date_only]
        deadline: u32,
    },
    /// Property, not cash
    /// Property can qualify for rollover relief, but a failed property deposit needs separate account-qualification treatment before a complete estimate can be made.
    Property,
}

/// One actual indirect-rollover deposit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MedicalRolloverDeposit {
    /// Actual deposit date
    #[libertas_date_only]
    pub deposited: u32,
    /// Actual deposited amount
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub amount: i64,
    /// Destination belonging to this same owner
    pub destination: MedicalRolloverDestination,
    /// Actual character if rollover exclusion fails
    #[libertas_default(Unconfirmed)]
    pub character: MedicalDepositCharacter,
}

impl MedicalRolloverDeposit {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if let MedicalDepositCharacter::Cash { year, deadline } = self.character {
            if !(u64::from(year)..=u64::from(year) + 1).contains(&u64::from(self.deposited / 10000))
            {
                return Some(
                    ValidationIssue::new("MEDICAL_DEPOSIT_YEAR_INVALID").at_field("deposited"),
                );
            }
            if deadline / 10000 < year {
                return Some(
                    ValidationIssue::new("MEDICAL_DEPOSIT_YEAR_INVALID")
                        .at_field("deadline")
                        .in_field("character"),
                );
            }
            if self.deposited > deadline {
                return Some(
                    ValidationIssue::new("MEDICAL_DEPOSIT_AFTER_DEADLINE").at_field("deposited"),
                );
            }
        }
        None
    }
    fn valid(&self) -> bool {
        crate::federal_rules::valid_date(self.deposited)
            && self.amount > 0
            && match self.character {
                MedicalDepositCharacter::Cash { year, deadline } => {
                    (1997..=2099).contains(&year)
                        && (year..=year + 1).contains(&(self.deposited / 10000))
                        && crate::federal_rules::valid_date(deadline)
                        && deadline / 10000 >= year
                        && self.deposited <= deadline
                }
                _ => true,
            }
    }
    fn timely(&self, received: u32, source: &MedicalSaAccount) -> bool {
        crate::dates::days_between(received, self.deposited) <= 60
            && (matches!(source, MedicalSaAccount::Archer | MedicalSaAccount::MedicareFromSpouse)
                || (matches!(source, MedicalSaAccount::Hsa)
                    && self.destination == MedicalRolloverDestination::Hsa))
    }
}
impl MedicalRollover {
    pub(crate) fn amount(&self) -> Exact {
        self.receipts.iter().flat_map(|p| &p.deposits).map(|d| whole(d.amount)).sum()
    }
    pub(crate) fn valid(&self) -> bool {
        self.preceding
            .is_none_or(|d| crate::federal_rules::valid_birth(d) && d < crate::TAX_YEAR_START_DATE)
            && !self.receipts.is_empty()
            && self.receipts.iter().all(|p| {
                crate::federal_rules::valid_date(p.received)
                    && (crate::TAX_YEAR_START_DATE..=crate::TAX_YEAR_END_DATE).contains(&p.received)
                    && !p.deposits.is_empty()
                    && p.deposits.iter().all(|d| d.valid() && d.deposited >= p.received)
            })
    }
    // Local source checking only. The coordinated estimate first resolves actual
    // current-year history and normalizes excluded/failed parts on a working copy.
    pub(crate) fn eligible(&self, source: &MedicalSaAccount) -> bool {
        self.receipts.iter().all(|p| {
            self.preceding.is_none_or(|last| p.received >= last + 10000)
                && p.deposits.iter().all(|d| d.timely(p.received, source))
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct FailedDeposit {
    pub source: i64,
    pub owner: Owner,
    pub received: u32,
    pub deposit: MedicalRolloverDeposit,
}
#[derive(Default)]
pub(crate) struct Projection {
    pub failed: Vec<FailedDeposit>,
}
// The pair is actual accepted source identity plus a named required source issue.
pub(crate) type Failure = (i64, &'static str);

// Actual failed cash deposits are a calculation input to the existing annual
// contribution owner, not new manual answers on its persisted record.
#[derive(Clone, Default)]
pub(crate) struct Contribution {
    pub personal: Exact,
    pub later: Exact,
}

#[derive(Clone, Default)]
pub(crate) struct Contributions {
    pub hsa: [Contribution; 2],
    pub archer: [Contribution; 2],
}

pub(crate) struct Prepared {
    pub draft: Option<alloc::boxed::Box<FederalDraft>>,
    pub contributions: alloc::boxed::Box<Contributions>,
}

pub(crate) fn classify(d: &FederalDraft) -> Result<Projection, Failure> {
    let mut prior = BTreeMap::new();
    let mut receipts = Vec::new();
    for entry in &d.income {
        let FederalIncome::MedicalDistribution { data } = &entry.income else {
            continue;
        };
        let MedicalSaPayment::Own { rollover: Some(rollover), .. } = &data.payment else {
            continue;
        };
        if !data.valid() || data.rule().is_some() {
            continue;
        }
        let kind = usize::from(!matches!(data.account, MedicalSaAccount::Hsa));
        let owner = usize::from(entry.owner == Owner::Spouse);
        let key = (owner, kind);
        if let Some(existing) = prior.insert(key, rollover.preceding)
            && existing != rollover.preceding
        {
            return Err((entry.id, "MEDICAL_ROLLOVER_HISTORY_CONFLICT"));
        }
        let person = d.people.as_ref().and_then(|p| {
            if entry.owner == Owner::Taxpayer { Some(&p.taxpayer) } else { p.spouse.as_ref() }
        });
        if person.and_then(|p| p.death_date()).is_some_and(|date| {
            rollover
                .receipts
                .iter()
                .any(|r| r.received > date || r.deposits.iter().any(|p| p.deposited > date))
        }) {
            return Err((entry.id, "ISSUE_ARCHER_MSA_POST_DEATH_ACTIVITY"));
        }
        for receipt in &rollover.receipts {
            receipts.push((receipt.received, key, entry.id, entry.owner, &data.account, receipt));
        }
    }
    receipts.sort_by_key(|v| (v.0, v.1));
    let mut successful = prior;
    let mut projection = Projection::default();
    for (index, &(date, key, source, owner, account, receipt)) in receipts.iter().enumerate() {
        let available = successful[&key].is_none_or(|last| date >= last + 10000);
        let has_timely = receipt.deposits.iter().any(|p| p.timely(date, account));
        if available
            && has_timely
            && receipts[index + 1..].iter().any(|v| {
                v.0 == date && v.1 == key && v.5.deposits.iter().any(|p| p.timely(date, v.4))
            })
        {
            // No arbitrary list/ID priority for same-day competing exemptions.
            return Err((source, "MEDICAL_ROLLOVER_SAME_DAY_PRIORITY_UNRESOLVED"));
        }
        if available && has_timely {
            successful.insert(key, Some(date));
        }
        for deposit in &receipt.deposits {
            if !available || !deposit.timely(date, account) {
                projection.failed.push(FailedDeposit {
                    source,
                    owner,
                    received: date,
                    deposit: deposit.clone(),
                });
            }
        }
    }
    Ok(projection)
}

// Normalize only a calculation copy. The accepted statement and manual annual
// contribution inputs remain unchanged; every subsequent owner sees this one
// composed source projection at the distribution collector; validation retains
// the original correction graph.
pub(crate) fn prepare(d: &FederalDraft) -> Result<Prepared, Failure> {
    if let Some(key) = correction_reference_rule(d) {
        return Err((deposit_corrections(d).first().map_or(0, |c| c.source), key));
    }
    let projection = classify(d)?;
    if projection.failed.is_empty() {
        return Ok(Prepared { draft: None, contributions: alloc::boxed::Box::default() });
    }
    // Accepted raw records keep strict bounds on the separate manual partition.
    if d.adjustments.as_ref().is_some_and(|a| a.hsa.iter().any(|h| !h.valid()) || !a.archer.valid())
    {
        return Err((0, "HSA_RECORD_INVALID"));
    }
    let mut working = alloc::boxed::Box::new(d.clone());
    let mut contributions = alloc::boxed::Box::<Contributions>::default();
    for part in &projection.failed {
        let MedicalDepositCharacter::Cash { year, .. } = part.deposit.character else {
            return Err((
                part.source,
                if matches!(part.deposit.character, MedicalDepositCharacter::Property) {
                    "MEDICAL_FAILED_PROPERTY_DEPOSIT_UNRESOLVED"
                } else {
                    "ISSUE_ARCHER_MSA_TREATMENT_UNCONFIRMED"
                },
            ));
        };
        if year < 2026 {
            return Err((part.source, "MEDICAL_FAILED_PRIOR_YEAR_DEPOSIT_UNRESOLVED"));
        }
        if year != 2026 {
            continue;
        }
        let Some(adjustments) = &working.adjustments else {
            return Err((part.source, "MEDICAL_FAILED_DEPOSIT_ACCOUNT_REQUIRED"));
        };
        let owner = usize::from(part.owner == Owner::Spouse);
        let contribution = match part.deposit.destination {
            MedicalRolloverDestination::Hsa => {
                let Some(_) = adjustments.hsa.iter().find(|a| a.owner == part.owner) else {
                    return Err((part.source, "MEDICAL_FAILED_DEPOSIT_ACCOUNT_REQUIRED"));
                };
                &mut contributions.hsa[owner]
            }
            MedicalRolloverDestination::Archer => {
                let crate::ArcherContributions::Records { owners, .. } = &adjustments.archer else {
                    return Err((part.source, "MEDICAL_FAILED_DEPOSIT_ACCOUNT_REQUIRED"));
                };
                let Some(_) = owners.iter().find(|a| a.owner == part.owner) else {
                    return Err((part.source, "MEDICAL_FAILED_DEPOSIT_ACCOUNT_REQUIRED"));
                };
                &mut contributions.archer[owner]
            }
        };
        contribution.personal += whole(part.deposit.amount);
        if part.deposit.deposited / 10000 == 2027 {
            contribution.later += whole(part.deposit.amount);
        }
    }
    if let Some(a) = &working.adjustments {
        for h in &a.hsa {
            for c in &h.corrections {
                if c.failed_deposit.is_some() && c.year == 2026 && c.contributed_on / 10000 == 2027
                {
                    contributions.hsa[usize::from(h.owner == Owner::Spouse)].later -=
                        whole(c.principal);
                }
            }
        }
        if let crate::ArcherContributions::Records { owners, .. } = &a.archer {
            for h in owners {
                for c in &h.corrections {
                    if c.failed_deposit.is_some()
                        && c.year == 2026
                        && c.contributed_on / 10000 == 2027
                    {
                        contributions.archer[usize::from(h.owner == Owner::Spouse)].later -=
                            whole(c.principal);
                    }
                }
            }
        }
    }
    for entry in &mut working.income {
        let FederalIncome::MedicalDistribution { data } = &mut entry.income else {
            continue;
        };
        let MedicalSaPayment::Own { nonmedical, rollover: Some(rollover), .. } = &mut data.payment
        else {
            continue;
        };
        for receipt in &mut rollover.receipts {
            receipt.deposits.retain(|deposit| {
                let failed = projection.failed.iter().any(|part| {
                    part.source == entry.id
                        && part.received == receipt.received
                        && part.deposit == *deposit
                });
                if failed {
                    nonmedical.push(crate::MedicalAccountWithdrawal {
                        date: receipt.received,
                        amount: deposit.amount,
                    });
                }
                !failed
            });
        }
        rollover.receipts.retain(|r| !r.deposits.is_empty());
        if rollover.receipts.is_empty()
            && let MedicalSaPayment::Own { rollover, .. } = &mut data.payment
        {
            *rollover = None;
        }
    }
    Ok(Prepared { draft: Some(working), contributions })
}

/// Original failed-rollover source available for a corrective return
/// These actual source facts are supplied read-only. Other manual contribution totals exclude these deposits.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthFailedDepositChoice {
    /// Original accepted statement identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    pub id: i64,
    /// Actual original statement
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Actual contribution owner
    #[libertas_read_only]
    pub owner: Owner,
    /// Actual failed cash deposits
    #[libertas_read_only]
    pub deposits: Vec<HealthFailedDepositFact>,
}
/// Actual failed cash contribution facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthFailedDepositFact {
    /// Actual deposit date
    #[libertas_date_only]
    #[libertas_read_only]
    pub deposited: u32,
    /// Actual ordinary contribution year
    #[libertas_read_only]
    pub year: u32,
    /// Actual destination
    #[libertas_read_only]
    pub destination: MedicalRolloverDestination,
    /// Original deposited principal before corrective returns
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    pub amount: i64,
}

// Independent owner/source-kind histories keep an unrelated incomplete group
// from invalidating known links in another group. Each selected group still
// includes every source that could change its successful-rollover history.
fn source_group(d: &FederalDraft, id: i64, owner: Owner) -> Option<usize> {
    d.income.iter().find_map(|e| {
        if e.id != id || e.owner != owner {
            return None;
        }
        let FederalIncome::MedicalDistribution { data } = &e.income else {
            return None;
        };
        if !matches!(data.payment, MedicalSaPayment::Own { rollover: Some(_), .. }) {
            return None;
        }
        match data.account {
            MedicalSaAccount::Hsa => Some(0),
            MedicalSaAccount::Archer | MedicalSaAccount::MedicareFromSpouse => Some(1),
            _ => None,
        }
    })
}
fn group_projection(d: &FederalDraft, owner: Owner, kind: usize) -> Result<Projection, Failure> {
    let mut group = d.clone();
    group.income.retain(|e| e.owner == owner && source_group(d, e.id, owner) == Some(kind));
    classify(&group)
}

#[derive(Clone)]
struct DepositCorrection {
    location: crate::hsa_correction::CorrectionSource,
    source: i64,
    owner: Owner,
    destination: MedicalRolloverDestination,
    deposited: u32,
    year: u32,
    principal: i64,
    personal: bool,
}
fn deposit_corrections(d: &FederalDraft) -> Vec<DepositCorrection> {
    let mut result = Vec::new();
    if let Some(a) = &d.adjustments {
        for h in &a.hsa {
            for c in &h.corrections {
                if let Some(source) = c.failed_deposit {
                    result.push(DepositCorrection {
                        location: crate::hsa_correction::CorrectionSource::Hsa {
                            owner: h.owner,
                            correction: c.clone(),
                        },
                        source,
                        owner: h.owner,
                        destination: MedicalRolloverDestination::Hsa,
                        deposited: c.contributed_on,
                        year: c.year,
                        principal: c.principal,
                        personal: matches!(c.payer, crate::HsaCorrectionPayer::Personal),
                    });
                }
            }
        }
        if let crate::ArcherContributions::Records { owners, .. } = &a.archer {
            for h in owners {
                for c in &h.corrections {
                    if let Some(source) = c.failed_deposit {
                        result.push(DepositCorrection {
                            location: crate::hsa_correction::CorrectionSource::Archer {
                                owner: h.owner,
                                correction: c.clone(),
                            },
                            source,
                            owner: h.owner,
                            destination: MedicalRolloverDestination::Archer,
                            deposited: c.contributed_on,
                            year: c.year,
                            principal: c.principal,
                            personal: matches!(c.payer, crate::ArcherCorrectionPayer::Personal),
                        });
                    }
                }
            }
        }
    }
    result
}
pub(crate) fn correction_reference_issue(
    d: &FederalDraft,
) -> Option<crate::hsa_correction::CorrectionReferenceIssue> {
    let links = deposit_corrections(d);
    for c in &links {
        if !c.personal {
            return Some((
                c.location.clone(),
                ValidationIssue::new("HEALTH_FAILED_DEPOSIT_PERSONAL_REQUIRED").at_field("payer"),
            ));
        }
        let Some(kind) = source_group(d, c.source, c.owner) else {
            return Some((
                c.location.clone(),
                ValidationIssue::new("HEALTH_FAILED_DEPOSIT_UNAVAILABLE")
                    .at_field("failed_deposit"),
            ));
        };
        let Ok(p) = group_projection(d, c.owner, kind) else {
            return Some((
                c.location.clone(),
                ValidationIssue::new("HEALTH_FAILED_DEPOSIT_UNAVAILABLE")
                    .at_field("failed_deposit"),
            ));
        };
        let gross: Exact = p.failed.iter().filter(|f| f.source == c.source && f.owner == c.owner && f.deposit.destination == c.destination && f.deposit.deposited == c.deposited && matches!(f.deposit.character,MedicalDepositCharacter::Cash {year,..} if year == c.year)).map(|f| whole(f.deposit.amount)).sum();
        if gross == whole(0) {
            return Some((
                c.location.clone(),
                ValidationIssue::new("HEALTH_FAILED_DEPOSIT_UNAVAILABLE")
                    .at_field("failed_deposit"),
            ));
        }
        let returned: Exact = links
            .iter()
            .filter(|v| {
                v.source == c.source
                    && v.owner == c.owner
                    && v.destination == c.destination
                    && v.deposited == c.deposited
                    && v.year == c.year
            })
            .map(|v| whole(v.principal))
            .sum();
        if returned > gross {
            return Some((
                c.location.clone(),
                ValidationIssue::new("HEALTH_FAILED_DEPOSIT_RETURN_ABOVE_DEPOSIT")
                    .at_field("principal"),
            ));
        }
    }
    None
}

pub(crate) fn correction_reference_rule(d: &FederalDraft) -> Option<&'static str> {
    correction_reference_issue(d).map(|(_, issue)| issue.key)
}

pub(crate) fn deposit_choices(
    d: &FederalDraft,
    destination: Option<MedicalRolloverDestination>,
) -> Vec<HealthFailedDepositChoice> {
    let mut choices = Vec::new();
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for kind in 0..2 {
            let Ok(p) = group_projection(d, owner, kind) else {
                continue;
            };
            for f in p.failed {
                let MedicalDepositCharacter::Cash { year, .. } = f.deposit.character else {
                    continue;
                };
                if destination.is_some_and(|v| v != f.deposit.destination) {
                    continue;
                }
                if !choices.iter().any(|v: &HealthFailedDepositChoice| v.id == f.source) {
                    let Some(entry) = d.income.iter().find(|e| e.id == f.source) else {
                        continue;
                    };
                    choices.push(HealthFailedDepositChoice {
                        id: f.source,
                        owner,
                        label: crate::Message::new(
                            "HEALTH_FAILED_DEPOSIT_CHOICE",
                            alloc::vec![crate::MessageArg::Text(entry.label.clone())],
                        )
                        .encode(),
                        deposits: Vec::new(),
                    });
                }
                let choice = choices.iter_mut().find(|v| v.id == f.source).unwrap();
                if let Some(fact) = choice.deposits.iter_mut().find(|v| {
                    v.deposited == f.deposit.deposited
                        && v.year == year
                        && v.destination == f.deposit.destination
                }) {
                    fact.amount += f.deposit.amount;
                } else {
                    choice.deposits.push(HealthFailedDepositFact {
                        deposited: f.deposit.deposited,
                        year,
                        destination: f.deposit.destination,
                        amount: f.deposit.amount,
                    });
                }
            }
        }
    }
    choices
}
