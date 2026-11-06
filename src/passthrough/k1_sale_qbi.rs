// Coverage: current ordinary K-1 section 179 sale QBI — owner integration.
//   Gains join the existing business before its one SSTB/wage/UBIA decision.
//   Current loss qualification is fixed before the same source's basis,
//   at-risk and passive limits; suspended qualified portions stay separate.
//   Section 1231 and prior-year OtherLoss qualification remain with their
//   existing owners; no separate qualification or retained history is created.
//   Evidence: tests/passthrough/k1_sale_qbi.rs; i8995 (2025 final), 26 CFR 1.199A-3.
use crate::k1_items::{K1ItemFlows, Section179SaleResult};
use crate::kernel::carry::{Amount, Cents};
use crate::scorp_basis::{LossRelease, LossVintage, release_losses};
use crate::{FederalIncome, FederalIncomeEntry, FederalResult, FederalStatus, Message, MessageArg};
use alloc::{vec, vec::Vec};

#[derive(Default)]
pub(crate) struct CurrentSales {
    // Gains have not yet passed current SSTB eligibility. The business owner
    // combines them with its other gains and applies that fraction once.
    pub gain_cents: i64,
    // Qualified current loss already allowed by every activity limit. Never
    // apply current SSTB eligibility to this amount a second time.
    pub loss_cents: Amount<Cents>,
    pub carries: Vec<(usize, [LossRelease; 3])>,
}

fn stage(
    current: Amount<Cents>,
    qualified: Amount<Cents>,
    allowed: Amount<Cents>,
) -> Result<LossRelease, (&'static str, &'static str)> {
    let years = if current.get() == 0 {
        Vec::new()
    } else {
        vec![LossVintage {
            year: 2026,
            original: current,
            original_qbi: qualified,
            remaining: current,
            remaining_qbi: qualified,
        }]
    };
    release_losses(&years, allowed).map_err(|_| crate::k1_sales::PASSIVE)
}

fn ordinary(flows: &K1ItemFlows, index: usize) -> i64 {
    flows.sales.iter().find(|(number, _, _)| *number == index).map_or(0, |(_, _, amount)| *amount)
}

pub(crate) fn has_loss(
    sales: Result<Vec<Section179SaleResult>, (&'static str, &'static str)>,
) -> bool {
    sales.is_ok_and(|values| values.iter().any(|value| value.gain < 0))
}

pub(crate) fn current(
    entry: &FederalIncomeEntry,
    pool: &crate::extended_rules::RentalResult,
    taxable: i64,
    status: FederalStatus,
) -> Result<CurrentSales, (&'static str, &'static str)> {
    let service = match &entry.income {
        FederalIncome::Partnership { data } => {
            let Some(statement) = &data.qbi else {
                return Ok(CurrentSales::default());
            };
            if !statement.valid(data) {
                return Err((
                    "ISSUE_PARTNERSHIP_QBI_COMPLETE_AND_RECONCILE_THE_PARTNERSHIP_S_ORDINARY_82FE9FDB",
                    crate::fixture_english!(
                        "Complete and reconcile the partnership's ordinary section 199A statement and partner adjustments."
                    ),
                ));
            }
            Some(statement.service)
        }
        FederalIncome::Scorp { data } => {
            let data = &crate::ScorpK1::from(data);
            let Some(statement) = &data.qbi_statement else {
                return Ok(CurrentSales::default());
            };
            if data.qbi_left_out().is_some() {
                return Ok(CurrentSales::default());
            }
            if !statement.valid(data) {
                return Err((
                    "ISSUE_SCORP_QBI_COMPLETE_THE_ORDINARY_BUSINESS_QBI_STATEMENT_AND_7B111E6E",
                    crate::fixture_english!(
                        "Complete the ordinary-business QBI statement and resolve additional shareholder adjustments."
                    ),
                ));
            }
            Some(statement.service)
        }
        _ => return Ok(CurrentSales::default()),
    };
    let Some(facts) = crate::k1_sales::facts(
        entry,
        false,
        pool.k1_ledgers.iter().find(|(id, _)| *id == entry.id).map(|(_, ledger)| ledger),
    )?
    else {
        return Ok(CurrentSales::default());
    };
    if !pool.inputs_complete {
        return Err(crate::k1_sales::PASSIVE);
    }
    let passive =
        crate::k1_sales::passive_flows(&facts, pool, entry.id).ok_or(crate::k1_sales::PASSIVE)?;
    let mut result = CurrentSales::default();
    let mut loss_cents = 0;
    for (index, value) in facts.values.iter().enumerate() {
        let index = index + 1;
        if value.ordinary >= 0 {
            result.gain_cents += value.ordinary * 100;
            continue;
        }
        let current = -value.ordinary * 100;
        let (applicable, range) = crate::business_rules::qbi_eligibility_factor(
            -current, service, false, taxable, status,
        )?;
        let qualified = crate::kernel::ratio::fraction(current, applicable, range);
        let basis = stage(
            Amount::new(current).expect("sale loss is nonnegative"),
            Amount::new(qualified).expect("qualified sale loss is nonnegative"),
            Amount::new(-ordinary(&facts.basis_flows, index))
                .expect("allowed sale loss is nonnegative"),
        )?;
        let risk = stage(
            basis.amount,
            basis.qbi,
            Amount::new(-ordinary(&facts.flows, index)).expect("allowed sale loss is nonnegative"),
        )?;
        let passive = stage(
            risk.amount,
            risk.qbi,
            Amount::new(-ordinary(&passive, index)).expect("allowed sale loss is nonnegative"),
        )?;
        loss_cents += passive.qbi.get();
        result.carries.push((index, [basis, risk, passive]));
    }
    result.loss_cents = Amount::new(loss_cents).expect("nonnegative qualified sale loss");
    Ok(result)
}

impl CurrentSales {
    pub(crate) fn report(&self, id: i64, result: &mut FederalResult) {
        for (index, stages) in &self.carries {
            for (stage, key) in [
                "NOTE_K1_SALE_QBI_BASIS_CARRY",
                "NOTE_K1_SALE_QBI_RISK_CARRY",
                "NOTE_K1_SALE_QBI_PASSIVE_CARRY",
            ]
            .into_iter()
            .enumerate()
            {
                for vintage in
                    stages[stage].remaining.iter().filter(|year| year.remaining.get() > 0)
                {
                    crate::federal_rules::note_once_scoped(
                        result,
                        Message::new(
                            key,
                            vec![
                                MessageArg::Number(id),
                                MessageArg::Number(*index as i64),
                                MessageArg::usd_cents(vintage.remaining.get()),
                                MessageArg::usd_cents(vintage.remaining_qbi.get()),
                                MessageArg::usd_cents(vintage.original.get()),
                                MessageArg::usd_cents(vintage.original_qbi.get()),
                            ],
                        ),
                        crate::IssueContext::income_id(id),
                    );
                }
            }
        }
    }
}
