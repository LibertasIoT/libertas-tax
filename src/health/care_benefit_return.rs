use crate::care_benefits::{
    CareBenefitPeriod, CareBenefitRecipient, FederalLtcInsured, FederalLtcPayment, LtcIllness,
    LtcPaidBasis, LtcPeriods, LtcSource,
};
use crate::kernel::phase;
use crate::kernel::ratio::exact::whole;
use crate::kernel::ratio::positive;
use crate::year_constants::LTC_PER_DIEM_LIMIT_CENTS;
use crate::{LtcContract, LtcPaymentOrigin, NonqualifiedCareTreatment};
use alloc::{string::String, vec::Vec};

impl CareBenefitPeriod {
    #[cfg(test)]
    pub(crate) fn amounts(&self) -> (i64, i64, i64, i64) {
        CalculatedCareBenefitPeriod::from(self).amounts()
    }
}

// Section C consumes accepted period facts and, for one period, the sum of
// linked form payments. These calculation values are never stored or decoded
// as another source period.
pub(crate) struct CalculatedCareBenefitPeriod {
    starts: u32,
    ends: u32,
    pub received: i64,
    services: i64,
    reimbursements: i64,
    recipient: CareBenefitRecipient,
    injured_payees: Option<crate::InjuredLtcPeriodPayees>,
}
impl From<&CareBenefitPeriod> for CalculatedCareBenefitPeriod {
    fn from(period: &CareBenefitPeriod) -> Self {
        Self {
            starts: period.starts,
            ends: period.ends,
            received: period.received,
            services: period.services,
            reimbursements: period.reimbursements,
            recipient: period.recipient.clone(),
            injured_payees: period.injured_payees.clone(),
        }
    }
}
impl CalculatedCareBenefitPeriod {
    // Aggregate limit, raw allocated line25 limit, exclusion used and taxable
    // payments. Required payee facts distinguish the raw limit from exclusion.
    pub(crate) fn amounts(&self) -> (i64, i64, i64, i64) {
        let days = i64::from(crate::dates::days_between(self.starts, self.ends) + 1);
        // Form 8853 Section C lines 21-25: the larger of the per diem limit for the
        // days and the care costs, less reimbursements.
        let limit = positive(
            phase::floor(days * LTC_PER_DIEM_LIMIT_CENTS, self.services) - self.reimbursements,
        );
        let (allocated, share) = match self.recipient {
            CareBenefitRecipient::Insured { other_payees } => {
                let share = phase::cap(self.received, [limit]);
                (if other_payees { share } else { limit }, share)
            }
            CareBenefitRecipient::Other { insured_received, others_received } => {
                let remaining = positive(limit - insured_received);
                let payments = whole(self.received) + whole(others_received);
                let allocated = if payments == whole(0) {
                    0
                } else {
                    crate::kernel::ratio::exact::rounded(
                        &(whole(remaining) * whole(self.received) / payments),
                    )
                    .expect("allocated LTC limit never exceeds its native remaining limit")
                };
                (allocated, phase::cap(self.received, [allocated]))
            }
        };
        (limit, allocated, share, positive(self.received - share))
    }
}

// Projection of the already calculated Section C taxable amount. These
// transient source amounts are not another benefit computation or ledger.
pub(crate) struct InjuredLtcPart {
    pub key: String,
    pub taxable: i64,
    pub receipts: Option<[i64; 2]>,
}
pub(crate) fn injured_period(
    id: i64,
    index: usize,
    period: &CalculatedCareBenefitPeriod,
    forms: &[(&crate::FederalIncomeEntry, i64)],
    single_period: bool,
    taxable: i64,
) -> InjuredLtcPart {
    let mut annual = [0; 2];
    for (entry, amount) in forms {
        annual[usize::from(entry.owner == crate::DocumentOwner::Spouses)] += amount;
    }
    let receipts = if let Some(payees) = &period.injured_payees {
        let mut amounts = [0; 2];
        let valid = payees.valid(period.received)
            && payees.payments().iter().all(|payment| {
                if let Some((entry, _)) = forms.iter().find(|(e, _)| e.id == payment.source) {
                    amounts[usize::from(entry.owner == crate::DocumentOwner::Spouses)] +=
                        payment.amount;
                    true
                } else {
                    false
                }
            });
        valid.then_some(amounts)
    } else if single_period {
        (annual.iter().sum::<i64>() == period.received).then_some(annual)
    } else {
        match annual {
            [a, 0] if a > 0 => Some([period.received, 0]),
            [0, a] if a > 0 => Some([0, period.received]),
            _ => (period.received == 0).then_some([0; 2]),
        }
    };
    InjuredLtcPart { key: alloc::format!("ltc_{}_period_{}", id, index + 1), taxable, receipts }
}

impl FederalLtcInsured {
    // The periods Section C is figured over, with the linked forms' line 20 payments
    // (cents) as the one period's payments; None when several periods' own payments
    // do not total them.
    pub(crate) fn rows(&self, line_20: i64) -> Option<Vec<CalculatedCareBenefitPeriod>> {
        match &self.periods {
            LtcPeriods::One { starts, ends, services, reimbursements, recipient } => {
                Some(alloc::vec![CalculatedCareBenefitPeriod {
                    starts: *starts,
                    ends: *ends,
                    received: line_20,
                    services: *services,
                    reimbursements: *reimbursements,
                    recipient: recipient.clone(),
                    injured_payees: None,
                }])
            }
            LtcPeriods::Several { periods, .. } => {
                (crate::kernel::ratio::checked_sum(periods.iter().map(|p| p.received))
                    == Some(line_20))
                .then(|| periods.iter().map(Into::into).collect())
            }
        }
    }
}

// Input paths in these Page projections have the same field names as their
// accepted records. Union payloads are transparent; no variant segment is added.
// Unresolved character remains an accepted source that withholds the estimate.
// Only the existing independently-entered-source refusal belongs in preflight;
// the whole-return reference owner still checks shared annuity gross totals.
pub(crate) fn document_issue(
    d: &crate::FederalDraft,
    owner: crate::Owner,
    document: &crate::IncomeSource,
) -> Option<crate::validation::ValidationIssue> {
    match document {
        crate::IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(page) } => {
            let record = FederalLtcPayment::from(&LtcSource::printed(page.clone()));
            record.issue().or_else(|| {
                record
                    .character_issue(d, owner)
                    .filter(|issue| issue.key == "LTC1099_SOURCE_UNAVAILABLE")
            })
        }
        crate::IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(page) } => {
            page.record().issue()
        }
        _ => None,
    }
}

impl FederalLtcPayment {
    pub(crate) fn topic_issue(
        &self,
        d: &crate::FederalDraft,
        owner: crate::Owner,
        require_answers: bool,
    ) -> Option<crate::validation::ValidationIssue> {
        self.issue().or_else(|| self.checkbox_issue()).or_else(|| {
            self.character_issue(d, owner).filter(|issue| {
                issue.key == "LTC1099_SOURCE_UNAVAILABLE"
                    || (require_answers
                        && match issue.key {
                            "ISSUE_LTC_SOURCE_CHARACTER_UNRESOLVED" => {
                                self.benefit_origin.is_none()
                            }
                            "ISSUE_LTC_LIFE_CHARACTER_UNRESOLVED" => {
                                self.accelerated_origin.is_none()
                            }
                            "ISSUE_LTC_NONQUALIFIED_CHARACTER_UNRESOLVED" => {
                                matches!(self.effective_contract(), LtcContract::Unknown)
                            }
                            _ => false,
                        })
            })
        })
    }
    fn checkbox_issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.benefits > 0 && self.box4_qualified && !self.effective_contract().qualified()).then(
            || {
                crate::validation::ValidationIssue::new("LTC1099_CONTRACT_CHECK_CONFLICT")
                    .at_field("contract")
            },
        )
    }
    pub(crate) fn character_issue(
        &self,
        d: &crate::FederalDraft,
        owner: crate::Owner,
    ) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if let Some(issue) = self.issue().or_else(|| self.checkbox_issue()) {
            return Some(issue);
        }
        if self.accelerated > 0 && self.illness.is_none() {
            return Some(ValidationIssue::new("LTC1099_ILLNESS_UNANSWERED").at_field("illness"));
        }
        if self.benefits > 0 && !matches!(self.benefit_origin, Some(LtcPaymentOrigin::Insurance)) {
            return Some(
                ValidationIssue::new("ISSUE_LTC_SOURCE_CHARACTER_UNRESOLVED")
                    .at_field("benefit_origin"),
            );
        }
        if self.accelerated > 0
            && !self.accelerated_origin.as_ref().is_some_and(LtcPaymentOrigin::life_eligible)
        {
            return Some(
                ValidationIssue::new("ISSUE_LTC_LIFE_CHARACTER_UNRESOLVED")
                    .at_field("accelerated_origin"),
            );
        }
        if self.benefits > 0 {
            match self.effective_contract() {
                LtcContract::Qualified { .. } if self.effective_contract().qualified() => {}
                LtcContract::Nonqualified {
                    treatment: NonqualifiedCareTreatment::AccidentHealth { .. },
                } if self.accident_health().is_some() => {}
                LtcContract::Nonqualified {
                    treatment: NonqualifiedCareTreatment::LifeAnnuity { source },
                } => {
                    if !annuity_source(d, *source, owner, self.benefits) {
                        return Some(
                            ValidationIssue::new("LTC1099_SOURCE_UNAVAILABLE")
                                .at_field("source")
                                .in_field("treatment")
                                .in_field("contract"),
                        );
                    }
                }
                _ => {
                    return Some(
                        ValidationIssue::new("ISSUE_LTC_NONQUALIFIED_CHARACTER_UNRESOLVED")
                            .at_field("contract"),
                    );
                }
            }
        }
        if let LtcContract::Nonqualified {
            treatment: NonqualifiedCareTreatment::AccidentHealth { recovery: Some(id), .. },
        } = self.effective_contract()
            && !recovery_source(d, *id, owner, self.benefits)
        {
            return Some(
                ValidationIssue::new("LTC1099_SOURCE_UNAVAILABLE")
                    .at_field("recovery")
                    .in_field("treatment")
                    .in_field("contract"),
            );
        }
        None
    }
    pub(crate) fn character_rule(
        &self,
        d: &crate::FederalDraft,
        owner: crate::Owner,
    ) -> Option<&'static str> {
        self.character_issue(d, owner).map(|issue| issue.key)
    }
    // Section C line 20 payments of this form (cents): the per diem box 1 and the
    // chronically ill box 2 of a qualified contract. A reimbursed amount and a
    // terminally ill accelerated death benefit are excluded without Section C.
    pub(crate) fn periodic(&self) -> i64 {
        if self.paid == LtcPaidBasis::Reimbursed {
            return 0;
        }
        let benefits = if self.effective_contract().qualified()
            && matches!(self.benefit_origin, Some(LtcPaymentOrigin::Insurance))
        {
            self.benefits
        } else {
            0
        };
        let accelerated = if self.illness != Some(LtcIllness::Terminal)
            && self.accelerated_origin.as_ref().is_some_and(LtcPaymentOrigin::life_eligible)
        {
            self.accelerated
        } else {
            0
        };
        benefits + accelerated
    }
    pub(crate) fn excluded(&self) -> i64 {
        let benefits = if self.effective_contract().qualified()
            && self.paid == LtcPaidBasis::Reimbursed
            && matches!(self.benefit_origin, Some(LtcPaymentOrigin::Insurance))
        {
            self.benefits
        } else {
            0
        };
        let accelerated = if (self.illness == Some(LtcIllness::Terminal)
            || self.paid == LtcPaidBasis::Reimbursed)
            && self.accelerated_origin.as_ref().is_some_and(LtcPaymentOrigin::life_eligible)
        {
            self.accelerated
        } else {
            0
        };
        benefits + accelerated
    }
    pub(crate) fn preview(&self) -> Option<i64> {
        // Linked annuity/recovery character and the shared per-diem limit need
        // the coordinated return. Only a completely known outright exclusion
        // has a standalone row preview.
        (self.rule().is_none()
            && self.periodic() == 0
            && self.excluded() == self.benefits + self.accelerated)
            .then_some(self.interest)
    }
}

// Which periods record a 1099-LTC names: an explicit choice is the record's income
// identity, an empty one uses the only record (the owner's, or on a joint return
// either filer's).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Link {
    Record(i64),
    // No record to use: none entered, or several and none chosen.
    Missing,
    // The chosen record belongs to the other filer on a separate return.
    OtherOwner,
}
pub(crate) fn resolve(d: &crate::FederalDraft, owner: crate::Owner, insured: Option<i64>) -> Link {
    use crate::FederalIncome;
    let joint = d.setup.as_ref().is_some_and(|s| crate::federal_rules::joint(s.filing_status()));
    match insured {
        Some(id) => match d.income.iter().find(|e| e.id == id) {
            Some(e) if matches!(e.income, FederalIncome::LtcInsured { .. }) => {
                if e.owner != crate::DocumentOwner::from(owner) && !joint {
                    Link::OtherOwner
                } else {
                    Link::Record(id)
                }
            }
            _ => Link::Missing,
        },
        None => {
            let mut records = d.income.iter().filter(|e| {
                (joint || e.owner == crate::DocumentOwner::from(owner))
                    && matches!(e.income, FederalIncome::LtcInsured { .. })
            });
            match (records.next(), records.next()) {
                (Some(e), None) => Link::Record(e.id),
                _ => Link::Missing,
            }
        }
    }
}
// The link of a 1099-LTC that has a per diem payment to place.
pub(crate) fn link_of(d: &crate::FederalDraft, e: &crate::FederalIncomeEntry) -> Option<Link> {
    match &e.income {
        crate::FederalIncome::LtcBenefits { data } if data.record().periodic() > 0 => {
            Some(resolve(d, e.owner.owned(), data.insured))
        }
        _ => None,
    }
}
// The forms that name a record, with their line 20 payments.
pub(crate) fn payments(d: &crate::FederalDraft, id: i64) -> Vec<(&crate::FederalIncomeEntry, i64)> {
    d.income
        .iter()
        .filter(|e| link_of(d, e) == Some(Link::Record(id)))
        .filter_map(|e| match &e.income {
            crate::FederalIncome::LtcBenefits { data } => Some((e, data.record().periodic())),
            _ => None,
        })
        .collect()
}

// These are references to existing independently entered sources. No tax or
// gross is copied into another accepted income record.
pub(crate) fn annuity_source(
    d: &crate::FederalDraft,
    id: i64,
    owner: crate::Owner,
    amount: i64,
) -> bool {
    d.income.iter().any(|e| {
        e.id == id
            && e.owner == crate::DocumentOwner::from(owner)
            && e.income
                .retirement()
                .is_some_and(|data| data.annuity_benefit_source() && data.gross >= amount)
    })
}
pub(crate) fn recovery_source(
    d: &crate::FederalDraft,
    id: i64,
    owner: crate::Owner,
    _amount: i64,
) -> bool {
    // The existing recovery may combine this receipt with other same-year
    // reimbursements; its total is not bounded by one LTC statement.

    d.income.iter().any(|e|e.id==id && e.owner==crate::DocumentOwner::from(owner) && matches!(&e.income,crate::FederalIncome::OtherIncome { data } if matches!(data.kind,crate::OtherIncomeKind::Recovery { .. }) && data.amount>0))
}
pub(crate) fn reference_issue(
    d: &crate::FederalDraft,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    reference_input_issue(d, None)
}
// Printed corrections retain links; removing or reassigning an actual source
// still requires correcting its link through the owning topic.
pub(crate) fn source_reference_issue(
    before: &crate::FederalDraft,
    after: &crate::FederalDraft,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    reference_input_issue(after, Some(before))
}
fn reference_input_issue(
    d: &crate::FederalDraft,
    before: Option<&crate::FederalDraft>,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    for entry in &d.income {
        let crate::FederalIncome::LtcBenefits { data } = &entry.income else {
            continue;
        };
        let linked = match data.record().effective_contract() {
            LtcContract::Nonqualified {
                treatment: NonqualifiedCareTreatment::LifeAnnuity { source },
            } => Some(*source),
            LtcContract::Nonqualified {
                treatment: NonqualifiedCareTreatment::AccidentHealth { recovery, .. },
            } => *recovery,
            _ => None,
        };
        if before.is_some_and(|before| {
            linked
                .and_then(|id| {
                    before
                        .income
                        .iter()
                        .find(|entry| entry.id == id)
                        .zip(d.income.iter().find(|entry| entry.id == id))
                })
                .is_some_and(|(old, new)| {
                    old.owner == new.owner
                        && matches!(
                            (&old.income, &new.income),
                            (
                                crate::FederalIncome::Retirement { .. },
                                crate::FederalIncome::Retirement { .. }
                            ) | (
                                crate::FederalIncome::OtherIncome { .. },
                                crate::FederalIncome::OtherIncome { .. }
                            )
                        )
                })
        }) {
            continue;
        }
        match data.record().effective_contract() {
            LtcContract::Nonqualified {
                treatment: NonqualifiedCareTreatment::LifeAnnuity { source },
            } => {
                let gross=d.income.iter().filter_map(|e| match &e.income {
                    crate::FederalIncome::LtcBenefits { data } if matches!(data.record().effective_contract(),LtcContract::Nonqualified { treatment:NonqualifiedCareTreatment::LifeAnnuity { source:other } } if source==other) => Some(data.form.benefits.unwrap_or(0)),
                    _=>None,
                });
                let gross = crate::kernel::ratio::checked_sum(gross);
                if !gross
                    .is_some_and(|gross| annuity_source(d, *source, entry.owner.owned(), gross))
                {
                    return Some((
                        entry.id,
                        crate::validation::ValidationIssue::new("LTC1099_SOURCE_UNAVAILABLE")
                            .at_field("source")
                            .in_field("treatment")
                            .in_field("contract"),
                    ));
                }
            }
            LtcContract::Nonqualified {
                treatment: NonqualifiedCareTreatment::AccidentHealth { recovery: Some(id), .. },
            } if !recovery_source(d, *id, entry.owner.owned(), data.form.benefits.unwrap_or(0)) => {
                return Some((
                    entry.id,
                    crate::validation::ValidationIssue::new("LTC1099_SOURCE_UNAVAILABLE")
                        .at_field("recovery")
                        .in_field("treatment")
                        .in_field("contract"),
                ));
            }
            _ => {}
        }
    }
    native_receipts_issue(d)
}

// Native receiver admission shared by candidate references and federal collection.
// The actual existing periodic, clinical-period and accident/health owners compute
// the amounts; this adapter only checks their required native aggregate carriers.
pub(crate) fn native_receipts_issue(
    d: &crate::FederalDraft,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    let invalid = |id| {
        (
            id,
            crate::validation::ValidationIssue::new("MEDICAL_REQUIRED_OUTPUT_UNAVAILABLE")
                .at_record(),
        )
    };
    for entry in &d.income {
        if let crate::FederalIncome::LtcBenefits { data } = &entry.income
            && let Some(issue) = data.record().intrinsic_issue()
        {
            return Some((entry.id, issue));
        }
    }
    let mut taxable = alloc::vec::Vec::new();
    for entry in &d.income {
        match &entry.income {
            crate::FederalIncome::LtcBenefits { data } => {
                if data.record().character_rule(d, entry.owner.owned()).is_some() {
                    continue;
                }
                if let Some((amount, _)) = data.record().accident_health() {
                    taxable.push(amount);
                }
                if link_of(d, entry) == Some(Link::Missing) {
                    taxable.push(data.record().periodic());
                }
            }
            crate::FederalIncome::LtcInsured { data } => {
                let forms = payments(d, entry.id);
                let received =
                    crate::kernel::ratio::checked_sum(forms.iter().map(|(_, amount)| *amount));
                let Some(received) = received else {
                    return Some(invalid(entry.id));
                };
                if data.valid()
                    && let Some(periods) = data.rows(received)
                {
                    taxable.extend(periods.iter().map(|period| period.amounts().3));
                }
            }
            _ => {}
        }
    }
    let id = d
        .income
        .iter()
        .rev()
        .find(|e| {
            matches!(
                e.income,
                crate::FederalIncome::LtcBenefits { .. } | crate::FederalIncome::LtcInsured { .. }
            )
        })?
        .id;
    crate::kernel::ratio::checked_sum(taxable).is_none().then(|| invalid(id))
}

pub(crate) fn reference_rule(d: &crate::FederalDraft) -> Option<&'static str> {
    reference_issue(d).map(|(_, issue)| issue.key)
}
