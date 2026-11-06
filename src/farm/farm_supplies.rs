// Cash-farm prepaid supplies. Schedule F expense ownership stays in FederalBusiness.
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::line_emission::emit as emit_lines;
use crate::line_emission::line_cents;
use crate::{BusinessActivity, FederalBusiness, FederalDraft, FederalIncome, FederalResult};
use alloc::{collections::BTreeMap, format, vec::Vec};
use libertas_macros::*;

/// Cash-farm prepaid supplies
/// Feed, seed, fertilizer and similar production supplies only. Enter current supplies already consumed and qualifying casualty-delayed supplies in ordinary business expenses. Do not repeat any costs entered here in expenses, inventory or another farm. Transfers, prior-year refunds and an accounting-method change other than one entered under Accounting-method changes need separate timing worksheets. A current refund attributable to an unused current payment reduces its retained cost; do not repeat the refunded amount as an expense. Enter supported production-poultry deferrals from 2025 below. Enter purchased resale poultry below; report its sales proceeds in farm receipts without repeating its cost there.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmSupplies {
    /// Payments in 2026 for supplies still unused at year-end
    pub advances: Vec<FarmSupplyAdvance>,
    /// Undeducted supply costs retained from earlier returns
    pub retained: Vec<FarmSupplyCarry>,
    /// Poultry purchased solely for resale
    /// Include current purchases and retained unsold lots. Report receipts separately with zero duplicate product cost; this worksheet owns recovery of the purchase cost.
    pub poultry: Vec<FarmPoultryResale>,
    /// Production-poultry purchase costs deferred from 2025
    /// Only excess prepaid costs from the 2025 payment year, supported by the earlier return and lot records. These costs release in 2026 without waiting for sale; do not enter them as ordinary consumed-supply or resale-poultry costs too.
    pub production_carry: Vec<FarmProductionPoultryCarry>,
    /// Production poultry bought and paid for in the current year
    /// Do not repeat these costs in ordinary expenses, inventory, depreciable assets or resale poultry. Retain the actual business-use start and useful life.
    pub production_current: Vec<crate::FarmProductionPoultryCurrent>,
    /// Exception to the half-of-other-expenses ceiling
    pub exception: FarmSupplyException,
}

/// Current unused supply purchase
/// A binding purchase rather than a refundable deposit (specific quantity and price terms with an obligation to accept delivery, without a right to refund or repurchase), made for a business benefit beyond reducing tax, such as securing supply or a price, and whose current deduction does not materially distort income. A deposit is deducted when the supplies are used; enter it then in ordinary expenses.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmSupplyAdvance {
    /// Paid cost of the identified unused supplies
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Actual purchase terms, payment and any refund for this unused portion
    pub purchase: crate::FarmSupplyPurchase,
}

/// Earlier undeducted supply portion
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmSupplyCarry {
    /// Undeducted cost entering 2026
    /// From the retained supply record and earlier return, not the original cost if part was already deducted. The entire identified portion must have the same 2026 consumption status.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub balance: i64,
    /// Separate undeducted AMT cost entering 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amt_balance: i64,
    /// This entire portion was used or consumed in 2026
    /// No keeps the cost for a later year.
    pub consumed: bool,
}

/// Identified poultry lot purchased solely for resale
/// Split portions disposed of at different times using source invoice costs and retained tax records. Do not repeat the whole lot or invent an allocation. Birds used for production, including laying hens or breeding birds later sold, need the production-poultry worksheet instead.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmPoultryResale {
    /// Purchase or lot description
    #[libertas_size(min = 1)]
    pub label: alloc::string::String,
    /// Established cost-recognition method
    /// Use the farm's adopted method, not whichever gives a larger current deduction. An accounting-method change other than one entered under Accounting-method changes needs its own worksheet and is not made by changing this answer.
    pub method: FarmPoultryMethod,
    /// Year the purchase cost was paid
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1900, max = 2026)]
    pub year: u32,
    /// Original paid cost attributable to this identified portion
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cost: i64,
    /// Cost already deducted through 2025
    /// Use retained tax records, including any deduction in the purchase year. Current-year purchases must have zero here. Never deduct a previously recovered cost again.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub deducted: i64,
    /// This entire identified portion was sold or otherwise disposed of in 2026
    /// No means still held at the end of the tax year, so its cost is not deducted. Casualty gains, insurance, refunds or transfers still require their separate source treatment.
    pub disposed: bool,
}

/// Adopted resale-poultry cost method
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FarmPoultryMethod {
    /// Deduct eligible purchases when paid, subject to the prepaid ceiling
    /// Qualifying raising-and-resale purchase costs under the adopted cash method, consistently expensed when paid without distorting income. The App does not switch methods.
    Paid,
    /// Recover purchase costs when the poultry is sold or otherwise disposed of
    /// Keep unsold purchase costs until disposition. A prepaid-supply ceiling exception does not change this adopted method or accelerate those costs.
    Sold,
}

/// Retained 2025 production-poultry purchase record
/// The remaining cost is solely a 2025 excess-prepaid production-poultry deferral: poultry purchased for use, or use and later resale, in 2025 under the consistently adopted eligible cash payment method, deferred only by the prepaid-supply ceiling, with identical retained regular/AMT treatment and no transfer, refund, capitalization election or accounting-method change other than one entered under Accounting-method changes. Not old unclaimed depreciation, resale-only inventory or a balance from an earlier payment year.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmProductionPoultryCarry {
    /// Purchase or lot description
    #[libertas_size(min = 1)]
    pub label: alloc::string::String,
    /// Cost paid in 2025 for this identified lot
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cost: i64,
    /// Cost deducted on the 2025 return
    /// Include all deductions already allowed for the same cost. The App computes the remaining cost; never repeat the original purchase as a new 2026 payment.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub deducted: i64,
}

/// Farm-related taxpayer status
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FarmSupplyRelation {
    /// Your main home is on a farm
    Home,
    /// Your principal business is farming
    Business,
    /// A qualifying family member meets either condition
    /// Spouse, sibling or half sibling, parent, grandparent, child or grandchild.
    Family,
}

/// Retained annual farm expense records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmSupplyYear {
    /// Tax year
    // Tax structure: this retained source belongs to a year before the 2026 return year.
    #[libertas_number(min = 2023, max = 2025)]
    pub year: u32,
    /// Prepaid farm supplies for that year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prepaid: i64,
    /// Other deductible farming expenses from that year's records
    /// Include depreciation and amortization; exclude prepaid supplies. Include all of the taxpayer's farms, not just this activity.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub other: i64,
}

/// Prepaid-supply ceiling exception
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FarmSupplyException {
    /// No exception claimed
    None,
    /// Farm-related taxpayer with three years of retained expense records
    History {
        relation: FarmSupplyRelation,
        /// Exactly one record for each of 2023, 2024 and 2025
        // Tax structure: the prepaid-supplies exception uses the three preceding tax years.
        #[libertas_size(min = 3, max = 3)]
        years: Vec<FarmSupplyYear>,
    },
    /// Farm-related taxpayer affected by extraordinary circumstances
    /// The excess prepayment directly results from a business-operation change caused by extraordinary circumstances, confirmed from your records.
    Extraordinary { relation: FarmSupplyRelation },
}
impl FarmSupplyException {
    fn applies(&self) -> bool {
        match self {
            Self::None => false,
            Self::History { years, .. } => {
                2 * years.iter().map(|y| y.prepaid).sum::<i64>()
                    < years.iter().map(|y| y.other).sum::<i64>()
            }
            Self::Extraordinary { .. } => true,
        }
    }
}
impl FarmSupplies {
    pub(crate) fn valid(&self) -> bool {
        let money = |v| (0..=100000000000).contains(&v);
        self.advances.len() <= 400
            && self.retained.len() <= 400
            && self.poultry.len() <= 400
            && self.production_current.len() <= 400
            && self.production_current.iter().all(crate::FarmProductionPoultryCurrent::valid)
            && self.production_carry.len() <= 400
            && self.production_carry.iter().all(|p| {
                !p.label.trim().is_empty() && money(p.cost) && (0..=p.cost).contains(&p.deducted)
            })
            && self.poultry.iter().all(|p| {
                !p.label.trim().is_empty()
                    && (1900..=2026).contains(&p.year)
                    && money(p.cost)
                    && (0..=p.cost).contains(&p.deducted)
                    && (p.year < 2026 || p.deducted == 0)
                    && (!matches!(p.method, FarmPoultryMethod::Sold) || p.deducted == 0)
            })
            && self.advances.iter().all(|v| money(v.amount) && v.purchase.valid(v.amount))
            && self.retained.iter().all(|v| money(v.balance) && money(v.amt_balance))
            && match &self.exception {
                FarmSupplyException::History { years, .. } => {
                    years.len() == 3
                        && (2023..=2025)
                            .all(|year| years.iter().filter(|v| v.year == year).count() == 1)
                        && years.iter().all(|v| money(v.prepaid) && money(v.other))
                }
                _ => true,
            }
    }
    fn production_current_parts(&self) -> [i64; 2] {
        self.production_current.iter().filter_map(crate::FarmProductionPoultryCurrent::timing).fold(
            [0, 0],
            |mut total, parts| {
                total[0] += parts[0];
                total[1] += parts[1];
                total
            },
        )
    }
    fn production_release(&self) -> i64 {
        self.production_carry
            .iter()
            .map(|p| crate::kernel::carry::native_balance(p.cost, p.deducted))
            .sum()
    }
    // Current unsold cost, current disposed cost, prior released cost, paid, opening.
    fn poultry_amounts(&self) -> [i64; 5] {
        let mut amounts = [0; 5];
        for p in &self.poultry {
            if p.year == 2026 {
                amounts[3] += p.cost;
                if p.disposed {
                    amounts[1] += p.cost;
                } else if p.method == FarmPoultryMethod::Paid {
                    amounts[0] += p.cost;
                }
            } else {
                amounts[4] += crate::kernel::carry::native_balance(p.cost, p.deducted);
                if p.disposed {
                    amounts[2] += crate::kernel::carry::native_balance(p.cost, p.deducted);
                }
            }
        }
        amounts
    }
    fn costs(&self) -> SupplyCosts {
        self.costs_for(false)
    }
    fn costs_for(&self, amt: bool) -> SupplyCosts {
        let poultry = self.poultry_amounts();
        let production = self.production_current_parts();
        SupplyCosts {
            eligible: production[1]
                + poultry[0]
                + self.advances.iter().map(|v| v.eligible()).sum::<i64>(),
            released: self.production_release()
                + poultry[2]
                + self
                    .retained
                    .iter()
                    .filter(|v| v.consumed)
                    .map(|v| if amt { v.amt_balance } else { v.balance })
                    .sum::<i64>(),
            immediate: production[0] + poultry[1],
            pool: self.production_current.iter().map(|record| record.cost).sum::<i64>()
                + self.production_release()
                + poultry[3]
                + poultry[4]
                + self.advances.iter().map(|v| v.amount - v.purchase.refunded).sum::<i64>()
                + self
                    .retained
                    .iter()
                    .map(|v| if amt { v.amt_balance } else { v.balance })
                    .sum::<i64>(),
        }
    }
    // Eligible new prepayments, released prior costs, current deduction, ending balance.
    pub(crate) fn amounts(&self, other: i64, _business: &FederalBusiness) -> [i64; 4] {
        let costs = self.costs();
        let allowed = if self.exception.applies() {
            costs.eligible
        } else {
            // This source-only operand precedes invocation-bound home use.
            // Completed return readers replace it with selected_amounts_with_home.
            limited_prepaid(costs.eligible, |_| other + costs.fixed())
        };
        costs.amounts(allowed)
    }

    pub(crate) fn report_with_claims(
        &self,
        d: &FederalDraft,
        b: &FederalBusiness,
        id: i64,
        r: &mut FederalResult,
        home: Option<&crate::home_office::HomeContext>,
        source_claims: &alloc::collections::BTreeMap<i64, i64>,
    ) {
        // Each current purchase keeps its own source facts and retained balance.
        crate::require_localized!(
            r,
            self.poultry_method_known(),
            &crate::IssueContext::income_id(id),
            "Poultry cost method",
            "ISSUE_CONFIRM_ONE_CONSISTENTLY_ADOPTED_ELIGIBLE_POULTRY_COST_METHOD_AN_UN_25613C7F",
            "Confirm one consistently adopted eligible poultry-cost method. An uncertain, income-distorting or changed method needs its own accounting worksheet; neither the supply exception nor this editor elects a new method.",
        );
        let mq = crate::depreciation::mid_quarter(d);
        // The local expense stage cannot yet coordinate cross-farm allocations or
        // mutually income-limited deductions. Never publish its provisional sum there.
        let farms = d.income.iter().filter(|e| matches!(&e.income, FederalIncome::Business { data } if matches!(data.activity, BusinessActivity::Farming { .. }))).count();
        let shared = selected_amounts_with_home_with_claims(d, false, home, source_claims)
            .ok()
            .and_then(|values| values.get(&id).copied());
        let distinct_amt = crate::depreciation::business_amt_adjustment(b, mq) != 0
            || crate::farm_conservation::business_amt_adjustment_with_claims(d, id, source_claims)
                != 0
            || crate::farm_production::business_amt_adjustment(d, id) != 0
            || self.retained.iter().any(|cost| cost.balance != cost.amt_balance);
        crate::require_localized!(
            r,
            !distinct_amt
                || selected_amounts_with_home_with_claims(d, true, home, source_claims)
                    .ok()
                    .and_then(|values| values.get(&id).copied())
                    .is_some(),
            &crate::IssueContext::income_id(id),
            "Farm AMT prepaid source",
            "ISSUE_FARM_AMT_PREPAID_SOURCE",
            "Distinct AMT farm expenses or retained costs need their final prepaid allowance and any dependent home-office limit. Known receipts remain; an unresolved dependent owner does not establish completed tax.",
        );
        let coupled = (farms != 1 && shared.is_none())
            || d.income.iter().any(|e| matches!(&e.income, FederalIncome::Business { data } if data.assets.iter().any(|a| a.available179() > 0)))
            || selected_amounts_with_home_with_claims(d, true, home, source_claims).ok().and_then(|values| values.get(&id).copied()).is_none();
        crate::require_localized!(
            r,
            matches!(&b.activity, BusinessActivity::Farming { data } if matches!(data.accounting, crate::FarmAccounting::Cash { .. })),
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "Farm prepaid accounting",
            "ISSUE_THIS_PREPAID_SUPPLY_WORKSHEET_USES_CASH_METHOD_TIMING_ACCRUAL_SUPPL_B035D15E",
            "This prepaid-supply worksheet uses cash-method timing. Accrual supply expenses follow their separate recognition records.",
        );
        crate::require_localized!(
            r,
            !coupled || self.exception.applies() || self.costs().eligible == 0,
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "Farm prepaid expense coordination",
            "ISSUE_THE_PREPAID_CEILING_STILL_NEEDS_COORDINATION_WHEN_SEVERAL_FARMS_COM_960C7361",
            "The prepaid ceiling needs its dependent expense worksheet when separate-return owners or accounting methods differ, section 179 or separate AMT depreciation applies, or source records are incomplete.",
        );
        crate::require_localized!(
            r,
            !crate::home_office::uses_activity(d, id)
                || matches!(&b.activity, BusinessActivity::Farming { data } if data.land_sales.is_empty()),
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "Farm prepaid home-office disposition",
            "ISSUE_A_FARM_LAND_DISPOSITION_WITH_HOME_OFFICE_USE_REQUIRES_GAIN_LOSS_ATT_317F08BC",
            "A farm-land disposition with home-office use requires gain/loss attribution to the home's business-income limit before coordinating the supply ceiling. Operating receipts alone do not establish that limit.",
        );
        crate::require_localized!(
            r,
            self.current_timing_known(),
            &crate::IssueContext::income_id(id),
            "Current production poultry timing",
            "ISSUE_FARM_CURRENT_POULTRY_TIMING_SOURCE",
            "Current production poultry needs its actual cash-payment method, business-use start and lesser-of-12-months/useful-life allocation. Partial-month or earlier starts and depreciation/method changes need their retained source worksheet; no current expense or prepaid percentage is guessed.",
        );
        let amounts = shared.unwrap_or_else(|| {
            let other = b.operating_cost_before_supplies(mq)
                + crate::farm_conservation::expense_adjustment_with_claims(d, id, b, source_claims)
                + crate::section179::calculate_with_claims(d, source_claims)
                    .businesses
                    .get(&id)
                    .copied()
                    .unwrap_or(0);
            self.amounts(other, b)
        });
        let poultry = self.poultry_amounts();
        if let Some(amt) = selected_amounts_with_home_with_claims(d, true, home, source_claims)
            .ok()
            .and_then(|values| values.get(&id).copied())
            && amt != amounts
        {
            emit_lines(
                r,
                &mut (),
                [
                    ("amt_deduction", amt[2], "EXPLAIN_FARM_SUPPLIES_AMT_DEDUCTION"),
                    ("amt_remaining", amt[3], "EXPLAIN_FARM_SUPPLIES_AMT_REMAINING"),
                ],
                |(name, value, key), _, _| {
                    Some(calculation_row(
                        None,
                        &format!("farm_{id}_supplies_{name}"),
                        LineValue::MoneyCents(value),
                        &["income.business.activity.farming.supplies"],
                        "https://www.irs.gov/publications/p225",
                        crate::Message::new(key, alloc::vec![]),
                    ))
                },
            );
        }
        // The deduction is a Schedule F Part II amount (lines 32a-32f).
        if amounts[2] != 0 {
            line_cents(
                r,
                &format!("farm_{id}_expense_line32_prepaid_supplies"),
                amounts[2],
                &["income.business.activity.farming.supplies"],
                "https://www.irs.gov/publications/p225",
                msg!("EXPLAIN_FARM_N_EXPENSE_LINE32_PREPAID_SUPPLIES"),
            );
        }
        for (name, amount, note) in [
            (
                "production_poultry_current_portion",
                self.production_current_parts()[0],
                msg!("EXPLAIN_FARM_PRODUCTION_POULTRY_CURRENT_PORTION"),
            ),
            (
                "production_poultry_prepaid_portion",
                self.production_current_parts()[1],
                msg!("EXPLAIN_FARM_PRODUCTION_POULTRY_PREPAID_PORTION"),
            ),
            (
                "production_poultry_released",
                self.production_release(),
                msg!("EXPLAIN_FARM_N_SUPPLIES_N_PRODUCTION_POULTRY_RELEASED"),
            ),
            (
                "poultry_disposed_cost",
                poultry[1] + poultry[2],
                msg!("EXPLAIN_FARM_N_SUPPLIES_N_POULTRY_DISPOSED_COST"),
            ),
            ("eligible", amounts[0], msg!("EXPLAIN_FARM_N_SUPPLIES_N_ELIGIBLE")),
            ("released", amounts[1], msg!("EXPLAIN_FARM_N_SUPPLIES_N_RELEASED")),
            ("deduction", amounts[2], msg!("EXPLAIN_FARM_N_SUPPLIES_N_DEDUCTION")),
            ("remaining", amounts[3], msg!("EXPLAIN_FARM_N_SUPPLIES_N_REMAINING")),
        ] {
            if ((name == "production_poultry_current_portion"
                || name == "production_poultry_prepaid_portion")
                && self.production_current.is_empty())
                || (name == "production_poultry_released" && self.production_carry.is_empty())
                || (name == "poultry_disposed_cost" && self.poultry.is_empty())
            {
                continue;
            }
            line_cents(
                r,
                &format!("farm_{id}_supplies_{name}"),
                amount,
                &["income.business.activity.farming.supplies"],
                "https://www.irs.gov/publications/p225",
                note,
            );
        }
    }
}

#[derive(Default)]
struct SupplyCosts {
    eligible: i64,
    released: i64,
    immediate: i64,
    pool: i64,
}
impl SupplyCosts {
    fn fixed(&self) -> i64 {
        self.released + self.immediate
    }
    fn amounts(&self, allowed: i64) -> [i64; 4] {
        [self.eligible, self.released, allowed + self.fixed(), self.pool - allowed - self.fixed()]
    }
}

// The same decreasing-cap test serves one farm and an unambiguous shared limit.
fn limited_prepaid(eligible: i64, other_expenses: impl Fn(i64) -> i64) -> i64 {
    match limited_prepaid_checked(eligible, |candidate| {
        Ok::<_, core::convert::Infallible>(other_expenses(candidate))
    }) {
        Ok(amount) => amount,
        Err(never) => match never {},
    }
}
fn limited_prepaid_checked<E>(
    eligible: i64,
    other_expenses: impl Fn(i64) -> Result<i64, E>,
) -> Result<i64, E> {
    let mut low = 0;
    let mut high = eligible;
    while low < high {
        let candidate = low + crate::kernel::ratio::half_offset_i64(high - low, 2);
        if candidate <= phase::floor(other_expenses(candidate)?, 0) / 2 {
            low = candidate;
        } else {
            high = candidate - 1;
        }
    }
    Ok(low)
}

// Section 464's limit belongs to the taxpayer. A binding pool is allocated
// proportionally to eligible purchases under the approved bookkeeping policy.
// Accepted IDs fix exact-cent remainders; unimplemented dependent deductions
// remain explicit rather than changing the denominator.
// This stage reads base costs only; never call section179::expenses here.
#[cfg(test)]
pub(crate) fn shared_amounts(d: &FederalDraft, id: i64) -> Option<[i64; 4]> {
    let source_claims = crate::related_forms::effective_claimants(d);
    shared_amounts_with_claims(d, id, &source_claims)
}

pub(crate) fn shared_amounts_with_claims(
    d: &FederalDraft,
    id: i64,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<[i64; 4]> {
    shared_amounts_for_with_claims(d, id, false, source_claims)
}
type SupplyAmounts = BTreeMap<i64, [i64; 4]>;
type SupplyIssue = (i64, &'static str);

#[cfg(test)]
pub(crate) fn shared_amounts_for(d: &FederalDraft, id: i64, amt: bool) -> Option<[i64; 4]> {
    let source_claims = crate::related_forms::effective_claimants(d);
    shared_amounts_for_with_claims(d, id, amt, &source_claims)
}

pub(crate) fn shared_amounts_for_with_claims(
    d: &FederalDraft,
    id: i64,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<[i64; 4]> {
    shared_source_scope_with_claims(d, amt, None, source_claims).0?.get(&id).copied()
}

pub(crate) fn selected_amounts_with_home_with_claims(
    d: &FederalDraft,
    amt: bool,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<SupplyAmounts, SupplyIssue> {
    let (amounts, issue) = shared_source_scope_with_claims(d, amt, home, source_claims);
    match (amounts, issue) {
        (_, Some(issue)) => Err(issue),
        (Some(amounts), None) => Ok(amounts),
        (None, None) => Ok(BTreeMap::new()),
    }
}

impl FarmSupplies {
    fn poultry_method_known(&self) -> bool {
        self.poultry.first().is_none_or(|first| {
            self.poultry.iter().all(|p| {
                matches!(p.method, FarmPoultryMethod::Sold)
                    == matches!(first.method, FarmPoultryMethod::Sold)
            })
        })
    }
    fn current_timing_known(&self) -> bool {
        self.production_current.iter().all(|record| record.timing().is_some())
    }
}

// One native pool retains its old amounts separately from source availability.
// Legacy callers keep their old None/local-fallback behavior unchanged.

fn shared_source_scope_with_claims(
    d: &FederalDraft,
    amt: bool,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> (Option<SupplyAmounts>, Option<SupplyIssue>) {
    let mut farms: Vec<_> = d
        .income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Business { data: b } => match &b.activity {
                BusinessActivity::Farming { data: f } => Some((e, b, f)),
                _ => None,
            },
            _ => None,
        })
        .collect();
    if farms.is_empty() || farms.iter().all(|(_, _, f)| f.supplies.is_none()) {
        return (None, None);
    }
    let source_id = farms.iter().find(|(_, _, farm)| farm.supplies.is_some()).unwrap().0.id;
    let method_issue = farms.iter().find_map(|(entry, _, farm)| {
        farm.supplies.as_ref().filter(|supplies| !supplies.poultry_method_known())
            .map(|_| (entry.id, "ISSUE_CONFIRM_ONE_CONSISTENTLY_ADOPTED_ELIGIBLE_POULTRY_COST_METHOD_AN_UN_25613C7F"))
    });
    let timing_issue = farms.iter().find_map(|(entry, _, farm)| {
        farm.supplies
            .as_ref()
            .filter(|supplies| !supplies.current_timing_known())
            .map(|_| (entry.id, "ISSUE_FARM_CURRENT_POULTRY_TIMING_SOURCE"))
    });
    farms.sort_by_key(|(e, _, _)| e.id);
    let owner = farms[0].0.owner;
    let joint = d.setup.as_ref().is_some_and(|s| s.filing_status() == crate::FederalStatus::Joint);
    let mq = crate::depreciation::mid_quarter(d);
    let invalid = farms.iter().find_map(|(e, b, f)| {
        let key = if amt
            && (crate::depreciation::business_amt_adjustment(b, mq) != 0
                || crate::farm_conservation::business_amt_adjustment_with_claims(
                    d,
                    e.id,
                    source_claims,
                ) != 0
                || crate::farm_production::business_amt_adjustment(d, e.id) != 0
                || f.supplies
                    .as_ref()
                    .is_some_and(|s| s.retained.iter().any(|c| c.balance != c.amt_balance)))
            && crate::home_office::uses_activity(d, e.id)
        {
            "ISSUE_FARM_AMT_PREPAID_SOURCE"
        } else if !matches!(f.accounting, crate::FarmAccounting::Cash { .. }) {
            "ISSUE_THIS_PREPAID_SUPPLY_WORKSHEET_USES_CASH_METHOD_TIMING_ACCRUAL_SUPPL_B035D15E"
        } else if !joint && e.owner != owner {
            "ISSUE_THE_PREPAID_CEILING_STILL_NEEDS_COORDINATION_WHEN_SEVERAL_FARMS_COM_960C7361"
        } else if !f.land_sales.is_empty() && crate::home_office::uses_activity(d, e.id) {
            "ISSUE_A_FARM_LAND_DISPOSITION_WITH_HOME_OFFICE_USE_REQUIRES_GAIN_LOSS_ATT_317F08BC"
        } else {
            return None;
        };
        Some((e.id, key))
    });
    let ceiling_depends_on_expenses = farms.iter().any(|(_, _, farm)| {
        farm.supplies.as_ref().is_some_and(|supplies| {
            !supplies.exception.applies() && supplies.costs_for(amt).eligible > 0
        })
    });
    let invalid = invalid.or_else(|| (ceiling_depends_on_expenses && d.income.iter().any(|entry|
        matches!(&entry.income, FederalIncome::Business { data } if data.assets.iter().any(|asset| asset.available179() > 0))))
        .then_some((source_id, "ISSUE_THE_PREPAID_CEILING_STILL_NEEDS_COORDINATION_WHEN_SEVERAL_FARMS_COM_960C7361")));
    if let Some(issue) = invalid {
        return (None, method_issue.or(Some(issue)));
    }
    let costs: Vec<_> = farms
        .iter()
        .map(|(_, _, f)| {
            f.supplies
                .as_ref()
                .map_or_else(SupplyCosts::default, |supplies| supplies.costs_for(amt))
        })
        .collect();
    let base: Vec<_> = farms
        .iter()
        .zip(&costs)
        .map(|((e, b, _), c)| {
            b.operating_cost_before_supplies(mq)
                + crate::farm_conservation::expense_adjustment_with_claims(
                    d,
                    e.id,
                    b,
                    source_claims,
                )
                - if amt {
                    crate::depreciation::business_amt_adjustment(b, mq)
                        + crate::farm_conservation::business_amt_adjustment_with_claims(
                            d,
                            e.id,
                            source_claims,
                        )
                        + crate::farm_production::business_amt_adjustment(d, e.id)
                } else {
                    0
                }
                + c.fixed()
        })
        .collect();
    let active: Vec<_> =
        costs.iter().enumerate().filter(|(_, c)| c.eligible > 0).map(|(i, _)| i).collect();
    let exempt =
        active.first().is_some_and(|&i| farms[i].2.supplies.as_ref().unwrap().exception.applies());
    if active.iter().any(|&i| farms[i].2.supplies.as_ref().unwrap().exception.applies() != exempt) {
        return (None, method_issue.or(Some((source_id,
            "ISSUE_THE_PREPAID_CEILING_STILL_NEEDS_COORDINATION_WHEN_SEVERAL_FARMS_COM_960C7361"))));
    }
    let Some(total) = crate::kernel::ratio::checked_sum(costs.iter().map(|c| c.eligible)) else {
        return (None, Some((source_id, "HOME_OUTPUT_UNAVAILABLE")));
    };
    let weights: Vec<_> = costs.iter().map(|c| c.eligible).collect();
    let other_expenses = |candidate| -> Result<i64, SupplyIssue> {
        let allocated = crate::kernel::ratio::allocate_remaining(candidate, &weights);
        let mut inputs: BTreeMap<_, _> = d
            .income
            .iter()
            .filter_map(|entry| {
                let FederalIncome::Business { data } = &entry.income else {
                    return None;
                };
                Some(
                    crate::home_office::HomeIncome::from_source_with_claims(
                        d,
                        entry.id,
                        data,
                        data.operating_cost_with_adjustment(
                            mq,
                            crate::farm_conservation::expense_adjustment_with_claims(
                                d,
                                entry.id,
                                data,
                                source_claims,
                            ),
                        ),
                        source_claims,
                    )
                    .map(|income| (entry.id, income))
                    .map_err(|_| (entry.id, "HOME_OUTPUT_UNAVAILABLE")),
                )
            })
            .collect::<Result<_, _>>()?;
        for (((entry, business, _), &base), share) in farms.iter().zip(&base).zip(&allocated) {
            inputs.insert(
                entry.id,
                crate::home_office::HomeIncome::from_source_with_claims(
                    d,
                    entry.id,
                    business,
                    base.checked_add(*share).ok_or((entry.id, "HOME_OUTPUT_UNAVAILABLE"))?,
                    source_claims,
                )
                .map_err(|_| (entry.id, "HOME_OUTPUT_UNAVAILABLE"))?,
            );
        }
        let mut total = 0i64;
        for ((entry, _, _), &base) in farms.iter().zip(&base) {
            let office = if crate::home_office::uses_activity(d, entry.id) {
                let context = home.ok_or((entry.id, "HOME_CONTEXT_REQUIRED"))?;
                crate::home_office::deduction_with_context_with_claims(
                    context.source_context(d, source_claims),
                    entry.id,
                    &inputs,
                    context,
                )
                .map_err(|_| (entry.id, "HOME_FARM_PREPAID_SOURCE_UNAVAILABLE"))?
            } else {
                0
            };
            total = total
                .checked_add(base)
                .and_then(|value| value.checked_add(office))
                .ok_or((entry.id, "HOME_OUTPUT_UNAVAILABLE"))?;
        }
        Ok(total)
    };
    let full_other = if exempt {
        0
    } else {
        match other_expenses(total) {
            Ok(amount) => amount,
            Err(issue) => return (None, Some(issue)),
        }
    };
    if exempt || total <= phase::floor(full_other, 0) / 2 {
        let values = farms
            .iter()
            .zip(&costs)
            .map(|((entry, _, _), cost)| (entry.id, cost.amounts(cost.eligible)))
            .collect();
        return (Some(values), method_issue.or(timing_issue));
    }
    let allowed = match limited_prepaid_checked(total, other_expenses) {
        Ok(amount) => amount,
        Err(issue) => return (None, Some(issue)),
    };
    let allocated = crate::kernel::ratio::allocate_remaining(allowed, &weights);
    let values = farms
        .iter()
        .zip(&costs)
        .zip(allocated)
        .map(|(((entry, _, _), cost), allowed)| (entry.id, cost.amounts(allowed)))
        .collect();
    (Some(values), method_issue.or(timing_issue))
}

impl FarmSupplyAdvance {
    fn eligible(&self) -> i64 {
        if self.purchase.eligible() { self.amount - self.purchase.refunded } else { 0 }
    }
}

// The ordinary business collector has deducted the regular supply allowance.
// Feed only its separate AMT difference into the existing AMT business owner.

pub(crate) fn business_amt_adjustment_with_claims(
    d: &FederalDraft,
    id: i64,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> i64 {
    match (
        shared_amounts_for_with_claims(d, id, false, source_claims),
        shared_amounts_for_with_claims(d, id, true, source_claims),
    ) {
        (Some(regular), Some(amt)) => regular[2] - amt[2],
        _ => 0,
    }
}

// Community-property projections preserve accepted source identity and facts.
crate::community::divide_record!(FarmSupplies {
    amounts: [advances, retained, poultry, production_carry, production_current, exception],
    gains: [],
    facts: [],
});

crate::community::divide_enum!(FarmSupplyException {
    records: [
        History { amounts: [relation], facts: [years] },
        Extraordinary { amounts: [relation], facts: [] }
    ],
    units: [None],
    tuples: [],
});

crate::community::divide_enum!(FarmSupplyRelation {
    records: [],
    units: [Home, Business, Family],
    tuples: [],
});

crate::community::divide_enum!(FarmPoultryMethod { records: [], units: [Paid, Sold], tuples: [] });

crate::community::divide_record!(FarmProductionPoultryCarry {
    amounts: [cost, deducted],
    gains: [],
    facts: [label],
});

crate::community::divide_record!(FarmPoultryResale {
    amounts: [method, cost, deducted],
    gains: [],
    facts: [label, year, disposed],
});

crate::community::divide_record!(FarmSupplyCarry {
    amounts: [balance, amt_balance],
    gains: [],
    facts: [consumed],
});

crate::community::divide_record!(FarmSupplyAdvance {
    amounts: [amount, purchase],
    gains: [],
    facts: [],
});
