use crate::{ScorpK1, ScorpPassiveHistory, ScorpRisk};
use alloc::vec::Vec;
use libertas_macros::*;

/// AMT basis of a shareholder loan
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpAmtDebt {
    /// Loan number in the regular shareholder-loan list
    #[libertas_number(min = 1)]
    pub loan: u32,
    /// Opening AMT debt basis
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub basis: i64,
}

/// Separate AMT ordinary-business records
/// Keep AMT balances even when they currently equal regular-tax balances. Omission does not mean no AMT difference.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpAmt {
    /// Ordinary-business depreciation adjustment for AMT
    /// From K-1 box 15 code A and its activity statement. Add the signed adjustment to regular box 1. Include only this ordinary business's adjustment.
    #[libertas_money("USD", 2)]
    pub adjustment: i64,
    /// Separate AMT portfolio capital amounts and loss histories
    /// Required when the regular source has portfolio capital records, even if all AMT values are zero or equal regular tax. Use the AMT activity statement and retained AMT balances.
    /// [NullText]
    /// The regular source has no portfolio capital records
    /// [SetText]
    /// The separate AMT portfolio capital amounts and histories are
    pub capital: Option<crate::ScorpCapital>,
    /// Separate AMT section 1231 business-property records
    /// Required when the regular source has these records, even when AMT amounts are zero or equal regular tax. Use AMT gains and separate suspended-loss balances.
    /// [NullText]
    /// The regular source has no section 1231 business-property records
    /// [SetText]
    /// The separate AMT section 1231 records are
    pub section1231: Option<crate::K1Section1231>,
    /// Opening AMT stock basis
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub stock: i64,
    /// Prior ordinary losses suspended by AMT basis
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_loss: i64,
    /// Prior nondeductible carryforward for AMT
    /// Only under the continuing losses-first election; otherwise zero.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_nondeductible: i64,
    /// Opening AMT debt bases
    /// Supply one entry for every shareholder loan, including a zero basis. Actual principal, advances and repayments come from the same loan source records.
    /// ----
    /// AMT loan basis
    pub loans: Vec<ScorpAmtDebt>,
    /// AMT at-risk records
    pub risk: ScorpRisk,
    /// Prior AMT passive losses
    pub passive: ScorpPassiveHistory,
    /// AMT rental amounts and carryovers
    /// Boxes 2 and 3 after each activity's box 15 code A adjustment, with the AMT Form 8582 carryovers.
    #[libertas_foldable(initially_open = false)]
    pub rentals: crate::K1RentalAmounts,
    /// AMT losses carried because of your AMT basis or amount at risk
    /// Your AMT balances; never substitute the regular-tax ones.
    #[libertas_foldable(initially_open = false)]
    pub carried: crate::K1LossCarryforwards,
    /// Complete ordinary-business AMT differences
    /// Check the K-1 box 15 AMT items. No uses the regular-tax records for AMT instead, with a notice; other AMT differences are not supported. Confirm no AMT differences outside these depreciation, portfolio capital, section 1231, tax-exempt-interest and retained-balance records, including other K-1 preference items, property contributions, dispositions or additional expense adjustments. Entered noncapital nondeductible expenses are shared with regular tax; retained AMT at-risk expense balances are separate. Entered tax-exempt income increases basis; its separately entered specified private-activity interest is included in the return-level AMT calculation. Current cash transactions, separately entered portfolio interest/dividends and participation facts are shared with the regular records; no additional portfolio AMT differences are included in this supported path.
    pub complete: bool,
}

impl ScorpAmt {
    fn validation_record(&self, owner: &ScorpK1) -> Option<ScorpK1> {
        // Arithmetic safety: validation_record feeds the actual ScorpSourceCapacity AMT
        // chain; downstream basis/activity/k1_totals native arithmetic remains bounded.
        let money = |v| (0..=100_000_000_000).contains(&v);
        if !(-100_000_000_000..=100_000_000_000).contains(&owner.ordinary)
            || !(-100_000_000_000..=100_000_000_000).contains(&self.adjustment)
            || ![self.stock, self.prior_loss, self.prior_nondeductible].into_iter().all(money)
            || self.loans.len() != owner.loans.len()
            || self.loans.iter().enumerate().any(|(i, v)| {
                v.loan == 0
                    || v.loan as usize > owner.loans.len()
                    || !money(v.basis)
                    || self.loans[..i].iter().any(|p| p.loan == v.loan)
            })
        {
            return None;
        }
        let mut amt = owner.clone();
        amt.amt = None; // no recursive alternate-tax projection
        amt.property_qbi = crate::ScorpPropertyQbi::default();
        amt.loss_qbi = crate::ScorpLossQbi::default(); // QBI is the regular-tax deduction for AMT
        amt.qbi_statement = None;
        amt.ordinary = owner.ordinary.checked_add(self.adjustment)?;
        amt.capital = self.capital.clone();
        amt.section1231 = self.section1231.clone();
        // Retain the physical sources for structural validation. Their regular
        // amounts are not an AMT monetary projection.
        amt.opening = self.stock;
        amt.prior_loss = self.prior_loss;
        amt.prior_nondeductible = self.prior_nondeductible;
        amt.risk = self.risk.clone();
        amt.passive = self.passive.clone();
        amt.rentals = self.rentals.clone();
        amt.carried = self.carried.clone();
        for item in &self.loans {
            amt.loans[item.loan as usize - 1].basis = item.basis;
        }
        Some(amt)
    }

    pub(crate) fn valid(&self, owner: &ScorpK1) -> bool {
        self.validation_record(owner)
            .is_some_and(|v| v.valid_with_sale_carry(false, ScorpSourceCapacity::amt()))
            && crate::K1Section179Sale::views(&owner.section179_sales, true).map_or(true, |sales| {
                crate::K1Section179Sale::carryover_rule(
                    &sales,
                    owner.section179_carryover,
                    owner.section179_carryover_sold,
                )
                .is_none()
            })
    }

    pub(crate) fn source(
        &self,
        owner: &ScorpK1,
    ) -> Result<ScorpSource<'static>, (&'static str, &'static str)> {
        if owner.capital.is_some() != self.capital.is_some() {
            return Err((
                "ISSUE_SCORP_AMT_SUPPLY_EXPLICIT_REGULAR_AND_AMT_CAPITAL_AFCA6E93",
                crate::fixture_english!(
                    "Supply explicit regular and AMT capital records for the same K-1; missing capital history is not a zero amount."
                ),
            ));
        }
        if owner.section1231.is_some() != self.section1231.is_some() {
            return Err((
                "ISSUE_SCORP_AMT_SUPPLY_EXPLICIT_REGULAR_AND_AMT_SECTION_D67066FF",
                crate::fixture_english!(
                    "Supply explicit regular and AMT section 1231 records for the same K-1; missing history is not a zero amount."
                ),
            ));
        }
        let sales = crate::K1Section179Sale::views(&owner.section179_sales, true)?;
        self.validation_record(owner)
            .and_then(|mut value| {
                value.section179_sales = sales;
                let source = ScorpSource {
                    record: ScorpSourceRecord::Owned(alloc::boxed::Box::new(value)),
                    capacity: ScorpSourceCapacity::amt(),
                };
                source.valid().then_some(source)
            })
            .ok_or((
                "ISSUE_SCORP_AMT_THE_SEPARATE_AMT_STOCK__DEBT_AND_C841361B",
                crate::fixture_english!(
                    "The separate AMT stock, debt and loss records are inconsistent."
                ),
            ))
    }
}

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(ScorpAmtDebt { amounts: [basis], gains: [], facts: [loan] });

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(ScorpAmt {
    amounts: [
        adjustment,
        capital,
        section1231,
        stock,
        prior_loss,
        prior_nondeductible,
        loans,
        risk,
        passive,
        rentals,
        carried
    ],
    gains: [],
    facts: [complete],
});

// Arithmetic safety: this token bounds the actual ordinary-income producer
// through basis, risk, property/QBI and k1_totals native receivers. Widening
// requires their complete source-to-output chain, not only the kernel stage.
// Only this native producer can create an AMT-capacity token. The token is
// transient: no editable or stored flag can widen the regular source cap.
#[derive(Clone, Copy)]
pub(crate) struct ScorpSourceCapacity(i64);
impl ScorpSourceCapacity {
    pub(crate) const fn regular() -> Self {
        Self(100_000_000_000)
    }
    const fn amt() -> Self {
        Self(2 * 100_000_000_000)
    }
    pub(crate) const fn limit(self) -> i64 {
        self.0
    }
}

enum ScorpSourceRecord<'a> {
    Borrowed(&'a ScorpK1),
    Owned(alloc::boxed::Box<ScorpK1>),
}

// The source owns admission context through every downstream financial body.
// Boxed ownership keeps this view small on regular/default-stack paths.
pub(crate) struct ScorpSource<'a> {
    record: ScorpSourceRecord<'a>,
    capacity: ScorpSourceCapacity,
}
impl core::ops::Deref for ScorpSource<'_> {
    type Target = ScorpK1;
    fn deref(&self) -> &Self::Target {
        match &self.record {
            ScorpSourceRecord::Borrowed(value) => value,
            ScorpSourceRecord::Owned(value) => value,
        }
    }
}
impl<'a> From<&'a ScorpK1> for ScorpSource<'a> {
    fn from(value: &'a ScorpK1) -> Self {
        Self {
            record: ScorpSourceRecord::Borrowed(value),
            capacity: ScorpSourceCapacity::regular(),
        }
    }
}
impl<'a> From<&'a ScorpSource<'_>> for ScorpSource<'a> {
    fn from(value: &'a ScorpSource<'_>) -> Self {
        Self { record: ScorpSourceRecord::Borrowed(value), capacity: value.capacity }
    }
}
impl From<ScorpK1> for ScorpSource<'static> {
    fn from(value: ScorpK1) -> Self {
        Self {
            record: ScorpSourceRecord::Owned(alloc::boxed::Box::new(value)),
            capacity: ScorpSourceCapacity::regular(),
        }
    }
}
impl ScorpSource<'_> {
    pub(crate) fn report_section1231(
        &self,
        basis: &crate::scorp_basis::BasisResult,
        id: i64,
        amt: bool,
        result: &mut crate::FederalResult,
    ) {
        self.record().report_section1231_in(basis, id, amt, result, self.capacity)
    }
    pub(crate) fn risk_result(
        &self,
        basis: &crate::scorp_basis::BasisResult,
    ) -> Result<crate::scorp_risk::ScorpRiskResult, crate::scorp_risk::ScorpRiskError> {
        self.risk.calculate_in(self, basis, None, self.capacity)
    }
    pub(crate) fn capital_result_with_events(
        &self,
        basis: &crate::scorp_basis::BasisResult,
        id: i64,
        amt: bool,
        result: &mut crate::FederalResult,
        events: Option<&crate::scorp::ScorpEvents>,
    ) -> Result<[i64; 4], (&'static str, &'static str)> {
        self.record().capital_result_in(basis, id, amt, result, events, self.capacity)
    }
    pub(crate) fn investment_interest_income(&self) -> Result<i64, (&'static str, &'static str)> {
        self.record().investment_interest_income_in(self.capacity)
    }
    pub(crate) fn valid(&self) -> bool {
        self.record().valid_with_sale_carry(true, self.capacity)
    }
    fn record(&self) -> &ScorpK1 {
        self
    }
    pub(crate) fn basis_result(
        &self,
    ) -> Result<crate::scorp_basis::BasisResult, (&'static str, &'static str)> {
        self.record().basis_result_in(self.capacity)
    }
    pub(crate) fn activity_result(
        &self,
    ) -> Result<crate::scorp::ScorpActivity, (&'static str, &'static str)> {
        self.record().activity_result_in(self.capacity)
    }
    pub(crate) fn pool_activity_result(
        &self,
    ) -> Result<crate::scorp::ScorpActivity, (&'static str, &'static str)> {
        self.pool_activity_with_events(None)
    }
    pub(crate) fn pool_activity_with_events(
        &self,
        events: Option<&crate::scorp::ScorpEvents>,
    ) -> Result<crate::scorp::ScorpActivity, (&'static str, &'static str)> {
        self.record().pool_activity_in(events, self.capacity)
    }
    pub(crate) fn ledger(
        &self,
    ) -> Result<crate::k1_loss::LossLedger, (&'static str, &'static str)> {
        self.ledger_with_events(None)
    }
    pub(crate) fn risk_with_events(
        &self,
        events: Option<&crate::scorp::ScorpEvents>,
    ) -> Result<crate::scorp_risk::ScorpRiskResult, (&'static str, &'static str)> {
        self.record().risk_in(events, self.capacity)
    }
    pub(crate) fn ledger_with_events(
        &self,
        events: Option<&crate::scorp::ScorpEvents>,
    ) -> Result<crate::k1_loss::LossLedger, (&'static str, &'static str)> {
        self.record().ledger_in(events, self.capacity)
    }
    pub(crate) fn outcome(&self) -> (crate::k1_items::K1ItemFlows, crate::k1_items::Disposal) {
        self.outcome_with_events(None)
    }
    pub(crate) fn outcome_with_events(
        &self,
        events: Option<&crate::scorp::ScorpEvents>,
    ) -> (crate::k1_items::K1ItemFlows, crate::k1_items::Disposal) {
        self.record().outcome_in(events, self.capacity)
    }
}
