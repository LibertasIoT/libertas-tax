use crate::kernel::ratio::exact::{Exact, whole};
use crate::medical_rollover::{
    Contributions, FailedDeposit, Failure, HealthFailedDepositChoice, HealthFailedDepositFact,
    MedicalDepositCharacter, MedicalRolloverDestination, Projection,
};
use crate::validation::ValidationIssue;
use crate::{FederalDraft, FederalIncome, MedicalSaAccount, MedicalSaPayment, Owner};
use alloc::{collections::BTreeMap, vec::Vec};

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
        // Only an answered payment can carry a rollover; a printed excess return cannot.
        let Some(MedicalSaPayment::Own { rollover: Some(rollover), .. }) = &data.payment else {
            continue;
        };
        let record = crate::FederalMedicalDistribution::from(data);
        if !record.valid() || record.rule().is_some() {
            continue;
        }
        let Some(account) = data.calculation_account() else { continue };
        let kind = usize::from(!matches!(account, MedicalSaAccount::Hsa));
        let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
        let key = (owner, kind);
        if let Some(existing) = prior.insert(key, rollover.preceding)
            && existing != rollover.preceding
        {
            return Err((entry.id, "MEDICAL_ROLLOVER_HISTORY_CONFLICT"));
        }
        let person = d.people.as_ref().and_then(|p| {
            if entry.owner == crate::DocumentOwner::Yours {
                Some(&p.taxpayer)
            } else {
                p.spouse.as_ref()
            }
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
            receipts.push((receipt.received, key, entry.id, entry.owner, account, receipt));
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
                    owner: owner.owned(),
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
        let Some(MedicalSaPayment::Own { nonmedical, rollover: Some(rollover), .. }) =
            &mut data.payment
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
            && let Some(MedicalSaPayment::Own { rollover, .. }) = &mut data.payment
        {
            *rollover = None;
        }
    }
    Ok(Prepared { draft: Some(working), contributions })
}

// Independent owner/source-kind histories keep an unrelated incomplete group
// from invalidating known links in another group. Each selected group still
// includes every source that could change its successful-rollover history.
fn source_group(d: &FederalDraft, id: i64, owner: Owner) -> Option<usize> {
    d.income.iter().find_map(|e| {
        if e.id != id || e.owner != crate::DocumentOwner::from(owner) {
            return None;
        }
        let FederalIncome::MedicalDistribution { data } = &e.income else {
            return None;
        };
        let data = &crate::FederalMedicalDistribution::from(data);
        if !matches!(
            data.effective_payment(),
            Some(MedicalSaPayment::Own { rollover: Some(_), .. })
        ) {
            return None;
        }
        match data.calculation_account()? {
            MedicalSaAccount::Hsa => Some(0),
            MedicalSaAccount::Archer => Some(1),
            _ => None,
        }
    })
}
fn group_projection(d: &FederalDraft, owner: Owner, kind: usize) -> Result<Projection, Failure> {
    let mut group = d.clone();
    group.income.retain(|e| {
        e.owner == crate::DocumentOwner::from(owner) && source_group(d, e.id, owner) == Some(kind)
    });
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
        // A retained correction waits for the linked distribution's actual
        // records to reconcile after a printed-source edit. A missing source
        // or an ownership change still fails the reference checks below.
        if d.income.iter().any(|entry| {
            entry.id == c.source
                && entry.owner == crate::DocumentOwner::from(c.owner)
                && matches!(&entry.income, FederalIncome::MedicalDistribution { data }
                if crate::FederalMedicalDistribution::from(data).source_issue().is_none()
                    && crate::FederalMedicalDistribution::from(data).answer_issue().is_some())
        }) {
            continue;
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
