// form_8962 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) type MarketplaceSnapshot = OwnerSnapshot<crate::marketplace::MarketplaceOutcome>;

#[derive(Clone)]
pub(super) struct HealthRentalSnapshot {
    pub(super) calculation: OwnerSnapshot<crate::extended_rules::RentalResult>,
    pub(super) magi_input: i64,
}

#[derive(Clone, Copy)]
pub(super) enum HealthSpecialKind {
    PassiveRentals,
    CoveredIra,
    SavingsBonds,
    StudentInterest,
}

// Pub. 974 (2025), pp. 56–57: a listed deduction is calculated only after
// the preceding health/PTC method has selected its deduction. Retain that
// existing owner's result while repeating the method for the next item.
// These are calculation snapshots, never another accepted fact record.
#[derive(Default)]
pub(super) struct HealthSpecials {
    pub(super) initialized: bool,
    pub(super) queue: Vec<HealthSpecialKind>,
    pub(super) completed: usize,
    pub(super) rental: Option<HealthRentalSnapshot>,
    pub(super) rental_trial: Option<HealthRentalSnapshot>,
    pub(super) ira: Option<crate::ira::IraResult>,
    pub(super) bonds: Option<OwnerSnapshot<i64>>,
    pub(super) student: Option<i64>,
    pub(super) ira_trial: Option<crate::ira::IraResult>,
    pub(super) bonds_trial: Option<OwnerSnapshot<i64>>,
    pub(super) student_trial: i64,
}

impl HealthSpecials {
    pub(super) fn initialize(
        &mut self,
        passive: bool,
        covered_ira: bool,
        bonds: bool,
        student: bool,
    ) {
        if self.initialized {
            return;
        }
        self.initialized = true;
        for (listed, kind) in [
            (passive, HealthSpecialKind::PassiveRentals),
            (covered_ira, HealthSpecialKind::CoveredIra),
            (bonds, HealthSpecialKind::SavingsBonds),
            (student, HealthSpecialKind::StudentInterest),
        ] {
            if listed {
                self.queue.push(kind);
            }
        }
    }

    pub(super) fn addback(&self, passive: i64, ira: i64, bonds: i64, student: i64) -> i64 {
        self.queue
            .iter()
            .map(|kind| match kind {
                HealthSpecialKind::PassiveRentals if self.rental.is_none() => passive,
                HealthSpecialKind::CoveredIra if self.ira.is_none() => ira,
                HealthSpecialKind::SavingsBonds if self.bonds.is_none() => bonds,
                HealthSpecialKind::StudentInterest if self.student.is_none() => student,
                _ => 0,
            })
            .sum()
    }

    pub(super) fn advance(&mut self) -> bool {
        let Some(kind) = self.queue.get(self.completed) else {
            return false;
        };
        match kind {
            HealthSpecialKind::PassiveRentals => {
                self.rental = self.rental_trial.take();
                assert!(self.rental.is_some(), "completed passive owner");
            }
            HealthSpecialKind::CoveredIra => {
                self.ira = self.ira_trial.take();
                assert!(self.ira.is_some(), "completed IRA owner");
            }
            HealthSpecialKind::SavingsBonds => {
                self.bonds = self.bonds_trial.take();
                assert!(self.bonds.is_some(), "completed savings-bond owner");
            }
            HealthSpecialKind::StudentInterest => self.student = Some(self.student_trial),
        }
        self.completed += 1;
        true
    }
}

pub(super) struct HealthInsuranceIn<'a> {
    pub(super) half: [i64; 2],
    pub(super) partner_half_attribution: Option<&'a alloc::collections::BTreeMap<i64, i64>>,
    pub(super) business_half_se_cents: &'a alloc::collections::BTreeMap<i64, i64>,
    pub(super) optional_weights: &'a alloc::collections::BTreeMap<i64, i64>,
    pub(super) optional_weights_cents: alloc::collections::BTreeMap<i64, i64>,
    pub(super) sep: &'a crate::sep::SepResult,
    pub(super) partners: &'a [(i64, crate::partnership::PartnerSeResult)],
    pub(super) iterative: bool,
    pub(super) specified: Option<&'a alloc::collections::BTreeMap<i64, Exact>>,
}

pub(super) struct HealthInsuranceOut {
    pub(super) health: crate::health::HealthResult,
    pub(super) health_earnings: alloc::collections::BTreeMap<i64, i64>,
}

pub(super) fn health_insurance_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    i: HealthInsuranceIn<'_>,
) -> core::ops::ControlFlow<(), HealthInsuranceOut> {
    let &Pass { d, a, basis, selected_regular, office_context, context, .. } = pass;
    let source_claims = context.source_claims;
    let HealthInsuranceIn {
        half,
        partner_half_attribution,
        business_half_se_cents,
        optional_weights,
        optional_weights_cents,
        sep,
        partners,
        iterative,
        specified,
    } = i;
    let mut health_earnings: alloc::collections::BTreeMap<_, _> =
        partners.iter().map(|(id, source)| (*id, positive(source.earnings))).collect();
    health_earnings.extend(optional_weights.iter().map(|(id, amount)| (*id, *amount)));
    let mut health_earnings_cents: alloc::collections::BTreeMap<_, _> =
        partners.iter().map(|(id, source)| (*id, positive(source.earnings_cents))).collect();
    health_earnings_cents.extend(optional_weights_cents);
    let health_uses_cents = iterative && !a.health_marketplace.policies.is_empty();
    let health = match crate::health::calculate_with_money_with_claims(
        d,
        half,
        if health_uses_cents {
            (!partners.is_empty()).then_some(business_half_se_cents)
        } else {
            partner_half_attribution
        },
        if health_uses_cents { &health_earnings_cents } else { &health_earnings },
        sep,
        &basis.public_safety,
        specified,
        if health_uses_cents {
            crate::marketplace::CalculationMoney::Cents
        } else {
            crate::marketplace::CalculationMoney::Dollars
        },
        selected_regular,
        r,
        Some(office_context),
        source_claims,
    ) {
        Ok(value) => value,
        Err(issue) => {
            issue.report(r);
            return core::ops::ControlFlow::Break(());
        }
    };
    core::ops::ControlFlow::Continue(HealthInsuranceOut { health, health_earnings })
}
