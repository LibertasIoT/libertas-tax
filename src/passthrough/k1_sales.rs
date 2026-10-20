// Coverage: Form 4797 K-1 section 179 sales — partial integration.
//   Per-property Part III, Part I lines 2/6, short-held Part II line 11,
//   shared recapture transfers and ordinary totals reuse normal K-1 limits.
//   Core and compound SALE returns reuse QBI, NIIT and independent AMT owners;
//   ordinary amounts enter Form461/Form172 as business amounts, while mixed
//   property/operating EBL acceptance remains due. Per-property current
//   losses share the business class's basis/risk allowance and passive cut;
//   current nonpassive losses bypass suspension of older passive balances.
//   The residual K-1 row retains original, historical and rental components.
//   Evidence: tests/passthrough/k1_sale_final.rs, tests/passthrough/k1_property_footing.rs,
//   tests/passthrough/k1_compound.rs and tests/passthrough/k1_property_ebl.rs;
//   i4797 and i8582 (2025 final), f4797 (2026 draft).
// Section 179 sale statements use the normal K-1 loss and passive owners.
// Reported worksheet/form amounts are whole dollars; source loss allocations
// remain actual cents through the existing basis, risk and passive owners. Input and
// worksheet arithmetic remain in k1_items; this module connects their normal
// loss-owner results to form rows and the QBI calculation.
use crate::k1_items::{K1ItemFlows, Section179SaleResult};
use crate::k1_loss::{K1LossClass::Section1231, LossLedger};
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::dollars;
use crate::{
    FederalDraft, FederalIncome, FederalIncomeEntry, FederalResult, FormLine, Message, TaxForm,
};
use alloc::{format, vec::Vec};

pub(crate) struct Facts {
    statements: Vec<crate::K1Section179Sale>,
    pub values: Vec<Section179SaleResult>,
    pub flows: K1ItemFlows,
    pub basis_flows: K1ItemFlows,
    property: Option<PropertySources>,
}

// Source attribution within an already computed business-property loss class.
// This does not refigure any owner limit or change its retained balances.
struct PropertySources {
    amounts: crate::k1_items::BusinessProperty,
    basis: i64,
    risk: i64,
    nonpassive: bool,
}

impl PropertySources {
    fn validate_net(
        &self,
        business: &crate::extended_rules::K1PropertyResult,
    ) -> Result<(), (&'static str, &'static str)> {
        if business.net_cents
            != self.amounts.gain - self.risk - self.amounts.passive_loss
                + business.suspended_cents.get()
        {
            return Err(PASSIVE);
        }
        Ok(())
    }

    fn new(
        original: Option<&crate::K1Section1231>,
        sales: &[Section179SaleResult],
        rentals: &crate::K1RentalAmounts,
        ledger: &LossLedger,
        nonpassive: bool,
    ) -> Result<Option<Self>, (&'static str, &'static str)> {
        let Some(amounts) = crate::k1_items::BusinessProperty::of(original, sales) else {
            return Ok(None);
        };
        // Rental property losses must pass these limits in full under the
        // existing rental-source rule; they never become business carries.
        let rental = rentals.property_allowed(ledger)?.iter().sum::<i64>();
        Ok(Some(Self {
            amounts,
            basis: ledger.basis_allowed[Section1231].get() - rental,
            risk: ledger.risk_allowed[Section1231].get() - rental,
            nonpassive,
        }))
    }

    fn allowed(amount: i64, weights: &[i64]) -> Result<Vec<i64>, (&'static str, &'static str)> {
        if weights.iter().any(|v| *v < 0) || !(0..=weights.iter().sum()).contains(&amount) {
            return Err(PASSIVE);
        }
        Ok(crate::kernel::ratio::allocate_remaining(amount, weights))
    }

    fn losses(
        &self,
        sales: &[Section179SaleResult],
        business: &crate::extended_rules::K1PropertyResult,
    ) -> Result<Vec<i64>, (&'static str, &'static str)> {
        let amounts = &self.amounts;
        let mut weights = Vec::with_capacity(sales.len() + 4);
        weights.push(
            amounts.loss - sales.iter().map(|v| phase::floor(-v.section1231, 0) * 100).sum::<i64>(),
        );
        weights.extend(sales.iter().map(|v| phase::floor(-v.section1231, 0) * 100));
        weights.push(amounts.basis_loss);
        let mut allowed = Self::allowed(self.basis, &weights)?;
        allowed.push(amounts.risk_loss);
        let mut allowed = Self::allowed(self.risk, &allowed)?;
        if self.nonpassive {
            if !(0..=amounts.passive_loss).contains(&business.suspended_cents.get()) {
                return Err(PASSIVE);
            }
        } else {
            allowed.push(amounts.passive_loss);
            // Form 8582 allocates the unallowed loss, then subtracts it.
            // Keep the shared allocator's source-cent residual order.
            let cuts = Self::allowed(business.suspended_cents.get(), &allowed)?;
            for (amount, cut) in allowed.iter_mut().zip(cuts) {
                *amount -= cut;
            }
        }
        self.validate_net(business)?;
        Ok(sales
            .iter()
            .enumerate()
            .map(
                |(index, sale)| {
                    if sale.section1231 >= 0 { sale.section1231 * 100 } else { -allowed[index + 1] }
                },
            )
            .collect())
    }
}

impl Facts {
    pub(crate) fn settle_property(
        &self,
        pool: &crate::extended_rules::RentalResult,
        id: i64,
    ) -> Result<Vec<i64>, (&'static str, &'static str)> {
        let Some(property) = &self.property else {
            return Ok(alloc::vec![0; self.values.len()]);
        };
        let business = pool
            .k1_section1231
            .iter()
            .find(|value| value.id == id)
            .map(crate::extended_rules::K1PropertyResult::business)
            .ok_or(PASSIVE)?;
        property.losses(&self.values, &business)
    }

    fn property_amounts(
        &self,
        pool: &crate::extended_rules::RentalResult,
        id: i64,
    ) -> Result<Vec<i64>, (&'static str, &'static str)> {
        if self.property.is_none() {
            return Ok(alloc::vec![0; self.values.len()]);
        }
        let value = pool.k1_section1231.iter().find(|value| value.id == id).ok_or(PASSIVE)?;
        self.property
            .as_ref()
            .expect("property source checked above")
            .validate_net(&value.business())?;
        if value.section179_sales_cents.len() != self.values.len()
            || value.reported_net(&value.section179_sales_cents) != Some(value.net)
        {
            return Err(PASSIVE);
        }
        Ok(value.section179_sales_cents.clone())
    }
}

pub(crate) fn facts(
    entry: &FederalIncomeEntry,
    amt: bool,
    ledger: Option<&LossLedger>,
) -> Result<Option<Facts>, (&'static str, &'static str)> {
    Ok(match &entry.income {
        FederalIncome::Partnership { data } => {
            let source = if amt {
                let Some(source) = &data.alternative else {
                    // Missing parent records cannot establish a property's AMT absence.
                    crate::K1Section179Sale::views(&data.section179_sales, true)?;
                    return Ok(None);
                };
                source
            } else {
                &data.regular
            };
            let statements = crate::K1Section179Sale::views(&data.section179_sales, amt)?;
            if statements.is_empty() {
                return Ok(None);
            }
            let values = source.sale_results(&data.items_for_record(source)?)?;
            let ledger = ledger.ok_or(PASSIVE)?;
            let items = source.source_k1_items(&data.items_for_record(source)?)?;
            Some(Facts {
                property: PropertySources::new(
                    source.section1231.as_ref(),
                    &values,
                    &source.rentals,
                    ledger,
                    data.material_for_losses() == crate::participation::Participation::Yes
                        || data.disposition.ended(),
                )?,
                statements,
                values,
                flows: items.flows(ledger),
                basis_flows: items.flows(&crate::k1_loss::LossLedger {
                    risk_allowed: ledger.basis_allowed,
                    ..*ledger
                }),
            })
        }
        FederalIncome::Scorp { data } => {
            let source: crate::scorp_amt::ScorpSource<'_> = if amt {
                let Some(source) = &data.amt else {
                    crate::K1Section179Sale::views(&data.section179_sales, true)?;
                    return Ok(None);
                };
                source.source(data)?
            } else {
                data.into()
            };
            let statements = crate::K1Section179Sale::views(&source.section179_sales, false)?;
            if statements.is_empty() {
                return Ok(None);
            }
            let values = source.sale_results()?;
            let ledger = ledger.ok_or(PASSIVE)?;
            let items = source.k1_items();
            Some(Facts {
                property: PropertySources::new(
                    source.section1231.as_ref(),
                    &values,
                    &source.rentals,
                    ledger,
                    source.nonpassive() == crate::participation::Participation::Yes
                        || source.disposition.ended(),
                )?,
                statements,
                values,
                flows: items.flows(ledger),
                basis_flows: items.flows(&crate::k1_loss::LossLedger {
                    risk_allowed: ledger.basis_allowed,
                    ..*ledger
                }),
            })
        }
        _ => None,
    })
}

pub(crate) fn passive_flows(
    facts: &Facts,
    pool: &crate::extended_rules::RentalResult,
    id: i64,
) -> Option<K1ItemFlows> {
    let cut = pool
        .k1_cuts
        .ordinary_by_id
        .iter()
        .filter(|(key, _)| *key == id)
        .map(|(_, cut)| cut.get())
        .sum();
    facts.flows.after_passive(cut)
}

pub(crate) const PASSIVE: (&str, &str) = (
    "ISSUE_K1_SALE_PASSIVE_ALLOCATION",
    crate::fixture_english!(
        "Review this K-1's sale losses and owner loss worksheets; the allowed amounts do not reconcile."
    ),
);

// Returns the ordinary sale amount already included in the normal K-1 total,
// after the same passive cut that adjusts that total. It is subtracted from
// the old code-R aggregate so the sale cannot be printed on two source lines.
pub(crate) fn report(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    amt: bool,
    result: &mut FederalResult,
) -> i64 {
    if !pool.inputs_complete {
        return 0;
    }
    let mut ordinary = 0;
    for entry in &crate::community::entries(draft) {
        let source = facts(
            entry,
            amt,
            pool.k1_ledgers.iter().find(|(id, _)| *id == entry.id).map(|(_, ledger)| ledger),
        )
        .and_then(|facts| {
            let Some(facts) = facts else { return Ok(None) };
            let flows = passive_flows(&facts, pool, entry.id).ok_or(PASSIVE)?;
            let property = facts.property_amounts(pool, entry.id)?;
            Ok(Some((facts, flows, property)))
        });
        let (facts, flows, property) = match source {
            Ok(Some(source)) => source,
            Ok(None) => continue,
            Err((key, _message)) => {
                crate::issue_localized!(
                    result,
                    &crate::IssueContext::income_id(entry.id),
                    "K-1 section 179 sale",
                    key,
                    _message,
                );
                continue;
            }
        };
        ordinary += flows.sales.iter().map(|(_, _, amount)| dollars(*amount)).sum::<i64>();
        for (index, (statement, value)) in facts.statements.iter().zip(&facts.values).enumerate() {
            let index = index + 1;
            let allowed = flows
                .sales
                .iter()
                .find(|(number, _, _)| *number == index)
                .map_or(0, |(_, _, amount)| dollars(*amount));
            let long_gain = value.held_long && value.gain > 0;
            let prefix =
                format!("{}k1_{}_section179_sale_{index}", if amt { "amt_" } else { "" }, entry.id);
            emit_lines(
                result,
                &mut (),
                [
                    (
                        "price",
                        long_gain.then_some("22"),
                        dollars(statement.regular.as_ref().expect("selected monetary view").price),
                    ),
                    (
                        "cost",
                        long_gain.then_some("23"),
                        dollars(statement.regular.as_ref().expect("selected monetary view").cost),
                    ),
                    ("depreciation", long_gain.then_some("24"), value.depreciation),
                    ("basis", long_gain.then_some("25"), value.line4),
                    ("gain", long_gain.then_some("26"), value.gain),
                    ("depreciation1245", long_gain.then_some("27a"), value.depreciation),
                    ("recapture", long_gain.then_some("27b"), value.recapture),
                    ("partthree", long_gain.then_some("34"), phase::floor(value.section1231, 0)),
                    (
                        "section1231",
                        if long_gain { Some("6") } else { value.held_long.then_some("2") },
                        dollars(property[index - 1]),
                    ),
                    ("ordinary", (!value.held_long).then_some("11"), allowed),
                ],
                |(suffix, line, amount), _, _| {
                    Some(calculation_row(
                        if amt {
                            None
                        } else {
                            line.map(|line| FormLine::new(TaxForm::Form4797, line))
                        },
                        &format!("{prefix}_{suffix}"),
                        LineValue::MoneyCents(amount * 100),
                        &[if matches!(entry.income, FederalIncome::Partnership { .. }) {
                            "income.partnership"
                        } else {
                            "income.scorp"
                        }],
                        "https://www.irs.gov/instructions/i4797",
                        Message::new("EXPLAIN_K1_SECTION179_SALE", alloc::vec![]),
                    ))
                },
            );
        }
    }
    ordinary
}

// Form 4797 Part III has one transfer through line 33 to Part II line 14.
// Include the existing direct-asset and rental recapture rows too.
pub(crate) fn report_recapture(result: &mut FederalResult) {
    let recap = result
        .lines
        .iter()
        .filter(|row| {
            row.form_line.as_ref().is_some_and(|form| {
                form.form == TaxForm::Form4797
                    && matches!(form.line.as_str(), "27b" | "28g" | "29b" | "30b" | "31b" | "32b")
            })
        })
        .map(|row| row.amount)
        .sum::<i64>();
    emit_lines(
        result,
        &mut (),
        [("33", "form4797_line33"), ("14", "form4797_line14")],
        |(line, id), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Form4797, line)),
                id,
                LineValue::MoneyCents(recap),
                &["income.business", "income.rental", "income.partnership", "income.scorp"],
                "https://www.irs.gov/instructions/i4797",
                Message::new("EXPLAIN_FORM4797_RECAPTURE_TOTAL", alloc::vec![]),
            ))
        },
    );
}

// All supported ordinary sources in this return flow to Schedule 1 line 4.
// The separate income-producing casualty-loss transfer requires its own
// unsupported Form 4684 source; it is not inferred from business losses.
pub(crate) fn report_ordinary_total(result: &mut FederalResult, ordinary: i64) {
    emit_lines(
        result,
        &mut (),
        [("19", "form4797_line19"), ("20b", "form4797_line20b")],
        |(line, id), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Form4797, line)),
                id,
                LineValue::MoneyCents(ordinary * 100),
                &[
                    "income.business",
                    "income.rental",
                    "income.partnership",
                    "income.scorp",
                    "income.sales",
                ],
                "https://www.irs.gov/instructions/i4797",
                Message::new("EXPLAIN_FORM4797_ORDINARY_TOTAL", alloc::vec![]),
            ))
        },
    );
}
