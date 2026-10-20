// Section461(l) source character after the existing activity and QBI owners.
// Coverage: loss-only property sources and one nonpassive ordinary K-1 business
// with ordinary SALE amounts. Current qualified fractions and originally
// qualified released losses come from the existing basis/risk/passive and QBI
// owners. Gross losses, before sale profits, determine the deferred fraction.
// Sources: IRS Form 461 instructions (loss limits before section 461(l));
// Form 8995 instructions and 26 CFR 1.199A-3(b)(1)(iv) (original qualification).
// Tests: k1_property_ebl, k1_property_offsets, k1_sale_ordinary_ebl, including
// partial current qualification, history without a current statement, SSTB,
// multiple sale gains, short-held sale loss, basis limits and independent AMT.
// Multiple/nonuniform sources, property/ordinary loss mixtures and rental EBL
// still require their source allocation. Net QBI never proves gross-loss QBI.
use crate::business_rules::QbiActivity;
use crate::extended_rules::RentalResult;
use crate::kernel::carry::{Amount, Cents};
use crate::kernel::phase;
use crate::{FederalDraft, FederalIncome};
use alloc::vec::Vec;

// Reconstruct one ordinary source from its existing loss/QBI owners before
// combining SALE gains with it. Net business loss cannot establish the
// qualified fraction of the gross losses that section 461(l) defers.
fn ordinary_sales_deferral(
    draft: &FederalDraft,
    pool: &RentalResult,
    activities: &mut Vec<QbiActivity>,
    business_net: i64,
    excess: i64,
    eligibility: (i64, crate::FederalStatus),
) -> Option<i64> {
    let allocated = crate::community::entries(draft);
    let mut entries = allocated.iter().filter(|e| {
        matches!(e.income, FederalIncome::Partnership { .. } | FederalIncome::Scorp { .. })
    });
    let entry = entries.next()?;
    if entries.next().is_some() {
        return None;
    }
    let simple = match &entry.income {
        FederalIncome::Partnership { data: p } => {
            p.complete()
                && !p.publicly_traded
                && p.trade_kind == crate::ScorpTradeKind::Ordinary
                && p.material_for_losses() == crate::participation::Participation::Yes
                && p.regular.ordinary <= 0
                && p.regular.section1231.is_none()
                && p.regular.rentals == crate::K1RentalAmounts::default()
                && p.regular.carried == crate::K1LossCarryforwards::default()
                && p.regular.capital_carried == crate::PartnerCapitalCarry::default()
                && p.income.is_empty()
                && p.deductions.is_empty()
                && p.guaranteed_services == 0
                && p.guaranteed_capital == 0
                && p.reported_se == Some(0)
                && p.section179 == 0
                && p.section179_carryover == 0
                && p.pensions == 0
                && p.retirement == crate::PartnerRetirement::None
                && matches!(p.health_insurance, crate::BusinessHealthInsurance::None)
                && p.disposition == crate::K1Disposition::Kept
        }
        FederalIncome::Scorp { data: s } => {
            s.trade_kind == crate::ScorpTradeKind::Ordinary
                && s.nonpassive() == crate::participation::Participation::Yes
                && s.ordinary <= 0
                && s.section1231.is_none()
                && s.rentals == crate::K1RentalAmounts::default()
                && s.carried == crate::K1LossCarryforwards::default()
                && s.income.is_empty()
                && s.deductions.is_empty()
                && s.section179 == 0
                && s.section179_carryover == 0
                && s.disposition == crate::K1Disposition::Kept
        }
        _ => false,
    };
    if !simple {
        return None;
    }
    let facts = crate::k1_sales::facts(
        entry,
        false,
        pool.k1_ledgers.iter().find(|(id, _)| *id == entry.id).map(|(_, ledger)| ledger),
    )
    .ok()??;
    if facts.values.iter().any(|v| v.section1231 != 0) {
        return None;
    }
    let flows = crate::k1_sales::passive_flows(&facts, pool, entry.id)?;
    let (gain, sale_loss) = flows.sales.iter().fold((0, 0), |(gain, loss), (_, _, v)| {
        let amount = crate::kernel::ratio::dollars(*v);
        (gain + phase::floor(amount, 0), loss + phase::floor(-amount, 0))
    });
    let operating = pool.ordinary_pass_through_business();
    if operating > 0 || operating + gain - sale_loss != business_net {
        return None;
    }
    let loss = -operating + sale_loss;
    let operating_source =
        pool.k1_amounts.iter().find(|source| source.id == entry.id)?.source_net_cents?;
    let source_gross = flows
        .sales
        .iter()
        .map(|(_, _, cents)| phase::floor(-*cents, 0))
        .try_fold(phase::floor(-operating_source, 0), i64::checked_add)?;
    let sales = crate::k1_sale_qbi::current(entry, pool, eligibility.0, eligibility.1).ok()?;
    let mut expected = Vec::new();
    let (current_index, current, historical) = match &entry.income {
        FederalIncome::Partnership { data: p } => {
            let suspended = pool.partners.iter().find(|v| v.id == entry.id)?.suspended;
            if let Some(statement) = &p.qbi {
                let ordinary = statement
                    .calculate_combined(
                        p,
                        suspended,
                        0,
                        eligibility,
                        &Default::default(),
                        &Default::default(),
                    )
                    .ok()?;
                let combined = statement
                    .calculate_combined(p, suspended, 0, eligibility, &sales, &Default::default())
                    .ok()?;
                if combined.historical_cents.get() > 0 {
                    expected.push(historical_activity(combined.historical_cents));
                }
                let index = expected.len();
                expected.push(combined.current);
                (
                    Some(index),
                    ordinary.current_loss_cents.get() + sales.loss_cents.get(),
                    ordinary.historical_cents.get(),
                )
            } else {
                let historical = crate::partnership_qbi::historical_without_statement(p, suspended)
                    .ok()?
                    .map_or(0, |v| v.historical.get());
                if historical > 0 {
                    expected.push(historical_activity(
                        Amount::new(historical).expect("qualified historical release"),
                    ));
                }
                (None, 0, historical)
            }
        }
        FederalIncome::Scorp { data: s } => {
            let suspended = pool
                .scorp_qbi_ordinary_suspensions
                .iter()
                .chain(&pool.scorp_suspensions)
                .find(|(id, _)| *id == entry.id)
                .map(|(_, value)| *value)?;
            let character = crate::section1231::PropertyCharacter::Capital;
            if let Some(statement) = &s.qbi_statement {
                let prepared = statement.prepare(s, eligibility.0, eligibility.1).ok()?;
                let (ordinary, current_loss) =
                    statement.activities_with_events(&prepared, suspended, 0, None).ok()?;
                let current = current_loss.get() + sales.loss_cents.get();
                let historical = -ordinary
                    .iter()
                    .skip(1)
                    .map(|activity| activity.source_income_cents)
                    .collect::<Option<Vec<_>>>()?
                    .iter()
                    .sum::<i64>();
                expected = statement
                    .combined_with_sales(
                        s,
                        [suspended, crate::kernel::carry::Amount::default()],
                        character,
                        (Some(eligibility.0), eligibility.1),
                        0,
                        &sales,
                        0,
                    )
                    .ok()?;
                (Some(0), current, historical)
            } else {
                expected = s
                    .historical_qbi_activities(
                        [suspended, crate::kernel::carry::Amount::default()],
                        Some(character),
                    )
                    .ok()?;
                (
                    None,
                    0,
                    -expected
                        .iter()
                        .map(|activity| activity.source_income_cents)
                        .collect::<Option<Vec<_>>>()?
                        .iter()
                        .sum::<i64>(),
                )
            }
        }
        _ => return None,
    };
    // Source identity must survive the financial equality check and the
    // replacement below, just as it does in the central activity builder.
    for activity in &mut expected {
        activity.owner = Some(entry.owner);
    }
    if *activities != expected {
        return None;
    }
    let deferred = crate::scorp_qbi::defer_source_qbi(
        &mut expected,
        current_index,
        loss,
        excess,
        Amount::<Cents>::new(current + historical).ok()?,
        Amount::<Cents>::new(current).ok()?,
        Amount::<Cents>::new(source_gross).ok()?,
    )
    .ok()?;
    *activities = expected;
    Some(deferred)
}

fn historical_activity(loss: Amount<Cents>) -> QbiActivity {
    QbiActivity {
        source_wages_cents: Some(0),
        source_property_cents: Some(0),
        source_income_cents: Some(-loss.get()),
        owner: None,
        income: -crate::kernel::ratio::dollars(loss.get()),
        wages: 0,
        property: 0,
        active: false,
        service: Some(false),
        group: None,
    }
}

fn current_ordinary_sales_only(draft: &FederalDraft, pool: &RentalResult) -> bool {
    let allocated = crate::community::entries(draft);
    let mut sources = allocated.iter().filter(|entry| {
        matches!(entry.income, FederalIncome::Partnership { .. } | FederalIncome::Scorp { .. })
    });
    let Some(entry) = sources.next() else {
        return false;
    };
    if sources.next().is_some() {
        return false;
    }
    let simple = match &entry.income {
        FederalIncome::Partnership { data } => {
            data.regular.ordinary < 0
                && data
                    .qbi
                    .as_ref()
                    .is_some_and(|q| !q.service && q.income == data.regular.ordinary)
                && data.regular.section1231.is_none()
                && data.regular.basis.prior_loss == 0
                && data.regular.risk.prior_loss == 0
                && data.regular.passive_loss == 0
                && data.income.is_empty()
                && data.deductions.is_empty()
                && data.disposition == crate::K1Disposition::Kept
                && data.guaranteed_services == 0
                && data.guaranteed_capital == 0
                && data.section179 == 0
                && data.section179_carryover == 0
                && data.pensions == 0
                && matches!(data.health_insurance, crate::BusinessHealthInsurance::None)
                && data.regular.rentals == crate::K1RentalAmounts::default()
        }
        FederalIncome::Scorp { data } => {
            data.ordinary < 0
                && data.qbi_statement.as_ref().is_some_and(|q| !q.service && q.income == 0)
                && data.loss_qbi.current == -data.ordinary
                && data.section1231.is_none()
                && data.prior_loss == 0
                && data.risk.prior_loss == 0
                && data.passive.loss == 0
                && data.income.is_empty()
                && data.deductions.is_empty()
                && data.disposition == crate::K1Disposition::Kept
                && data.section179 == 0
                && data.section179_carryover == 0
                && data.rentals == crate::K1RentalAmounts::default()
        }
        _ => false,
    };
    simple
        && crate::k1_sales::facts(
            entry,
            false,
            pool.k1_ledgers.iter().find(|(id, _)| *id == entry.id).map(|(_, ledger)| ledger),
        )
        .is_ok_and(|facts| {
            facts.is_some_and(|facts| facts.values.iter().all(|value| value.section1231 == 0))
        })
}

pub(crate) fn qualified_deferral(
    draft: &FederalDraft,
    pool: &RentalResult,
    activities: &mut Vec<QbiActivity>,
    business_net: i64,
    excess: i64,
    property_only: bool,
    eligibility: (i64, crate::FederalStatus),
) -> Option<i64> {
    if !pool.inputs_complete
        || business_net >= 0
        || excess <= 0
        || excess > -business_net
        || pool.k1_section1231.iter().any(|v| !v.rentals.is_empty())
        || draft.income.iter().any(|entry| {
            !matches!(
                entry.income,
                FederalIncome::Partnership { .. }
                    | FederalIncome::Scorp { .. }
                    | FederalIncome::Wage { .. }
                    | FederalIncome::Interest { .. }
                    | FederalIncome::Dividend { .. }
                    | FederalIncome::Sale { .. }
            )
        })
        || !draft.income.iter().any(|entry| {
            matches!(entry.income, FederalIncome::Partnership { .. } | FederalIncome::Scorp { .. })
        })
    {
        return None;
    }
    if !property_only
        && let Some(deferred) =
            ordinary_sales_deferral(draft, pool, activities, business_net, excess, eligibility)
    {
        return Some(deferred);
    }
    if activities.iter().any(|v| v.income > 0 || v.service != Some(false)) {
        return None;
    }
    let loss = -business_net;
    let qualified = activities.iter().try_fold(0i64, |sum, v| sum.checked_sub(v.income))?;
    if property_only
        && draft.income.iter().any(|entry| {
            let codes = match &entry.income {
                FederalIncome::Partnership { data } => {
                    crate::k1_items::K1Items::partner(&data.income, &data.deductions)
                }
                FederalIncome::Scorp { data } => {
                    crate::k1_items::K1Items::scorp(&data.income, &data.deductions)
                }
                _ => return false,
            };
            use crate::k1_items::K1ItemKind::*;
            [DebtCancellation, OrdinaryActivity, NonportfolioShort, NonportfolioLong]
                .iter()
                .any(|kind| codes.has(*kind))
        })
    {
        return None;
    }
    // With one loss character and no income offsets, the existing shared
    // proportional rule retains each already resolved qualified fraction.
    // One current ordinary source and its SALE items share full qualification.
    // Prove that from the source, not equality between aggregate net amounts.
    if !property_only && (qualified != loss || !current_ordinary_sales_only(draft, pool)) {
        return None;
    }
    if activities.is_empty() {
        return (qualified == 0).then_some(0);
    }
    let mut candidate = activities.clone();
    let deferred = crate::scorp_qbi::defer_loss_only_qbi(&mut candidate, loss, excess).ok()?;
    *activities = candidate;
    Some(deferred)
}
