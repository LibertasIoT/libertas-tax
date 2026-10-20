mod direct_sources;
pub use direct_sources::{Section179DirectIncomeSource, section179_direct_income_sources};

// Coverage update: Form 4562 line 10 removes carryover restored to sold K-1
// property basis; active business income includes allowed activity sale
// amounts. This is an owner checkpoint, not completed R9 return support.
// Evidence: src/tests/passthrough/k1_sale_return.rs; i4562/i4797 (2025 final).
// Active non-PTP partners' documented services payments also supply their own
// line11 compensation, separately from entity income. Capital payments and
// owner-deduction coordination retain their named boundaries. Reg.1.179-2(c);
// src/tests/cost_recovery/unit17/partner_services.rs.
// Assigned PTP/PAL source seam: the sole return179 allocator exposes current
// and retained-income-carry components without a second allocation/limit.
// The checked transient supplier preserves source identity, tax-system basis,
// income carry and the named risk cycle. Existing whole-return PTP/PAL routing
// remains unsupported until its consumer is delivered. Evidence:
// src/tests/cost_recovery/passthrough.rs unit17_ptp179_*; src/tests/cost_recovery/interview.rs
// unit17_ptp179_actual_two_rows_complete_save_reopen_restart.
use crate::k1_loss::K1LossClass;
use crate::kernel::lines::line_cents;
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{fraction, positive};
use crate::{FederalBusiness, FederalDraft, FederalIncome, FederalResult};
use alloc::{collections::BTreeMap, format, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Section 179 facts for a separate return
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum Section179Spouse {
    /// Spouse's records not entered
    /// With a section 179 claim on this return, treated as No below: no section 179 deduction on this return, with a note.
    NotSupplied,
    /// Spouse's purchase records and allocation
    Records {
        /// Spouse's eligible 2026 section 179 business cost
        /// Include all qualifying property placed in service, even when the spouse elects zero deduction. Use the spouse's purchase and business-use records; exclude purchases already included on this return. This cost affects only the shared dollar-limit phaseout, not your income or MACRS convention. Enter zero only when the spouse had no eligible purchases.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        cost: i64,
        /// Allocation of the shared dollar limit after phaseout
        allocation: Section179Share,
        /// Complete separate-return purchase and election records
        /// Confirm both spouses' property totals and any elected percentages, with no amended separate-to-joint election or unresolved ownership/community-property allocation. Each spouse applies their own business-income limit. The spouse's 2026 Form 4562 Part I and the written allocation election show it. No takes no section 179 deduction on this return: your 2026 elections are depreciated normally and earlier carryforwards wait for 2027. A partnership or S corporation section 179 amount still needs these records.
        eligible: bool,
    },
}
/// Couple's section 179 dollar-limit allocation
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum Section179Share {
    /// Equal shares; no different allocation elected
    Equal,
    /// Each spouse elected a percentage
    /// Enter each actual elected percentage. If they do not total 100%, the required allocation is 50% each; the App does not normalize mismatched elections.
    Elected {
        /// Your elected share in hundredths of one percent
        /// Enter 9500 for 95.00%, 5000 for 50.00%, or 10000 for 100.00%.
        // Percentage encoding: 10,000 ten-thousandths is 100 percent.
        #[libertas_number(min = 0, max = 10000)]
        yours: u32,
        /// Spouse's elected share in hundredths of one percent
        // Percentage encoding: 10,000 ten-thousandths is 100 percent.
        #[libertas_number(min = 0, max = 10000)]
        spouse: u32,
    },
}
impl Section179Spouse {
    pub(crate) fn valid(self) -> bool {
        match self {
            Self::NotSupplied => true,
            Self::Records { cost, allocation, .. } => {
                (0..=100000000000).contains(&cost)
                    && match allocation {
                        Section179Share::Equal => true,
                        Section179Share::Elected { yours, spouse } => {
                            yours <= 10000 && spouse <= 10000
                        }
                    }
            }
        }
    }
    fn facts(self) -> (i64, i64, bool) {
        match self {
            // Records not entered or unconfirmed: no share of the dollar
            // limit is assumed.
            Self::NotSupplied => (0, 0, false),
            Self::Records { eligible: false, cost, .. } => (cost, 0, false),
            Self::Records { cost, allocation, eligible: true } => (
                cost,
                match allocation {
                    Section179Share::Elected { yours, spouse } if yours + spouse == 10000 => {
                        i64::from(yours)
                    }
                    _ => 5000,
                },
                true,
            ),
        }
    }
}
fn spouse(d: &FederalDraft) -> Option<Section179Spouse> {
    match d.setup.as_ref()?.status {
        crate::FilingChoice::Separate { section179, .. } => Some(section179),
        _ => None,
    }
}

// New elections first; then carryforward vintage, oldest first. Within one
// vintage, the stated proportional asset selection preserves the exact total.
fn allocate(amount: i64, items: &[(i64, usize, i64, u32)]) -> Vec<i64> {
    use crate::kernel::carry::native_consume_step;
    let mut result = alloc::vec![0; items.len()];
    let mut years: Vec<_> = items.iter().map(|i| i.3).collect();
    years.sort_unstable();
    years.dedup();
    let mut remaining = amount;
    for year in years {
        let indices: Vec<_> =
            items.iter().enumerate().filter(|(_, i)| i.3 == year).map(|(index, _)| index).collect();
        let weights: Vec<_> = indices.iter().map(|&index| items[index].2).collect();
        let allowed = native_consume_step(weights.iter().sum(), &mut remaining);
        for (index, value) in
            indices.into_iter().zip(crate::kernel::ratio::allocate_remaining(allowed, &weights))
        {
            result[index] = value;
        }
    }
    result
}

// A Schedule K-1 (1065) box 12 or (1120-S) box 11 amount is one Form 4562
// line 6 item of the current year (Reg. 1.179-3(e)); this index marks it in
// the shared allocation so it never lands in a business's asset map.
const PASS_THROUGH: usize = usize::MAX;

// One K-1 on the return, with the facts its box 12 / box 11 section 179 and
// the partner's line 11 business income need. Amounts are cents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PassThrough {
    pub id: i64,
    pub partnership: bool,
    // Box 12 (1065) or box 11 (1120-S).
    pub amount: i64,
    // The owner's 2025 Form 4562 line 13 carryover of this K-1's amounts
    // (question C13: PartnerK1 and ScorpK1 section179_carryover, T2-29p and
    // T2-31p), entering line 10; it does not reduce basis again.
    pub carryover: i64,
    // None preserves the native regular aggregate carry's unknown vintage.
    pub income_carry_vintages: Option<Vec<crate::Asset179Carryover>>,
    pub sold_carryover: i64,
    // Active business net, including its Form 4797 items after basis,
    // at-risk and release of its own prior passive loss; zero unless the owner materially participates
    // (Reg. 1.179-2(c)(2)(v), (c)(6)(ii)).
    pub income: i64,
    // Payments still lacking owner compensation coordination: capital, plus
    // services for PTPs. Established active non-PTP services are already in
    // owner income; the entity pool never includes section707(c) payments.
    pub guarantees: i64,
    // Remaining amount at risk after the activity's other items, before this
    // deduction (Form 6198 line 21 of the section 179 part); None when the
    // partner's opening at-risk history is unknown (question C9).
    pub risk_end: Option<i64>,
    pub material: bool,
    pub publicly_traded: bool,
    pub owner_deductions: bool,
}

// Reg.1.179-2(c)(2)(iv),(6): the entity excludes section707(c) payments;
// the approved R3 application includes established active services as the
// partner's own compensation. PTP payment coordination remains separate.
fn partner_service_compensation(data: &crate::PartnerK1) -> i64 {
    if !data.publicly_traded
        && data.participation.material() == crate::participation::Participation::Yes
    {
        data.guaranteed_services
    } else {
        0
    }
}

// Every K-1 whose activity records resolve. A K-1 whose basis, at-risk or
// passive records fail is already refused by name by the income block and
// the passive pool, so leaving it out here never hides an amount.
fn pass_throughs(d: &FederalDraft) -> Result<Vec<PassThrough>, Section179PreparedIssue> {
    let mut result = Vec::new();
    for e in &crate::community::entries(d) {
        match &e.income {
            FederalIncome::Partnership { data } => {
                let Ok(pool) = crate::community::partner_pool(d, e.id, data, &data.regular) else {
                    continue;
                };
                let material =
                    data.participation.material() == crate::participation::Participation::Yes;
                let services = partner_service_compensation(data);
                let income = if material {
                    pool.combined
                        .nonpassive
                        .checked_add(services)
                        .ok_or(Section179PreparedIssue::AmountOverflow(e.id))?
                } else {
                    0
                };
                let guarantees = if material {
                    data.guaranteed_services
                        .checked_add(data.guaranteed_capital)
                        .and_then(|total| total.checked_sub(services))
                        .ok_or(Section179PreparedIssue::AmountOverflow(e.id))?
                } else {
                    0
                };
                result.push(PassThrough {
                    id: e.id,
                    partnership: true,
                    amount: pool.activity.ledger.basis_allowed[K1LossClass::Section179].get(),
                    carryover: crate::kernel::carry::native_balance(
                        data.section179_carryover,
                        data.section179_carryover_sold,
                    ),
                    income_carry_vintages: None,
                    sold_carryover: data.section179_carryover_sold,
                    income,
                    guarantees,
                    risk_end: pool.activity.risk_end,
                    material,
                    publicly_traded: data.publicly_traded,
                    owner_deductions: !matches!(
                        data.health_insurance,
                        crate::BusinessHealthInsurance::None
                    ) || data.pensions != 0
                        || data.retirement != crate::PartnerRetirement::None,
                });
            }
            FederalIncome::Scorp { data } => {
                let (Ok(basis), Ok(activity)) =
                    (data.basis_result(), crate::community::scorp_pool(d, e.id, data, false))
                else {
                    continue;
                };
                let events = crate::community::scorp_events(d, e.id, false).ok().flatten();
                let Ok(risk) = data.risk.calculate_with_events(data, &basis, events.as_ref())
                else {
                    continue;
                };
                let material = data.participation.as_ref().is_some_and(|facts| {
                    facts.material() == crate::participation::Participation::Yes
                });
                result.push(PassThrough {
                    id: e.id,
                    partnership: false,
                    amount: risk.ledger.basis_allowed[K1LossClass::Section179].get(),
                    carryover: crate::kernel::carry::native_balance(
                        data.section179_carryover,
                        data.section179_carryover_sold,
                    ),
                    income_carry_vintages: None,
                    sold_carryover: data.section179_carryover_sold,
                    income: if material { activity.combined.nonpassive } else { 0 },
                    guarantees: 0,
                    risk_end: risk.year_end,
                    material,
                    publicly_traded: false,
                    owner_deductions: false,
                });
            }
            _ => {}
        }
    }
    Ok(result)
}

/// The source tax system is explicit even while the legacy return allocator
/// supports only its existing regular inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section179IncomeSystem {
    Regular,
    Alternative,
}
/// Actual source-cent class amounts supplied by the owning activity stage.
/// A current net may be negative. Released prior losses remain separate and
/// nonnegative; Form4562 income carry is not a released passive loss.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Section179IncomeClass {
    pub current_net_cents: i64,
    pub prior_allowed_loss_cents: i64,
}
/// Complete replacement of one accepted K-1 parent's business-income
/// contribution before section179. The caller owns legal stage selection;
/// this input neither chooses PAL ordering nor infers component association.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Section179BusinessIncomeSource {
    pub source: i64,
    pub system: Section179IncomeSystem,
    pub ordinary: Section179IncomeClass,
    pub section1231: Section179IncomeClass,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section179IncomeSourceKind {
    Partnership,
    Scorp,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section179IncomeK1Allocation {
    pub kind: Section179IncomeSourceKind,
    pub components: Vec<Ptp179ComponentAllocation>,
    pub allowed_cents: i64,
    /// Form4562 income-limit carry stays outside the PAL loss classes.
    pub business_income_carry_cents: i64,
}
/// Results of the sole return allocator, not another source-level limit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section179IncomeAllocation {
    pub system: Section179IncomeSystem,
    pub business_income_limit_cents: i64,
    pub total_allowed_cents: i64,
    pub k1: BTreeMap<i64, Section179IncomeK1Allocation>,
}

// The box 12 / box 11 amount of one K-1 allowed on Form 4562 line 12 after
// the dollar and business-income limits, in cents (Schedule E line 28
// column (j)); zero for any other entry.
pub(crate) fn pass_through_allowed(d: &FederalDraft, id: i64) -> i64 {
    calculate(d).pass_through.get(&id).copied().unwrap_or(0)
}

fn income_allocation(
    allocated: &Deduction,
    system: Section179IncomeSystem,
) -> Section179IncomeAllocation {
    let k1 = allocated
        .pass_throughs
        .iter()
        .map(|source| {
            let allowed = allocated.pass_through.get(&source.id).copied().unwrap_or(0);
            (
                source.id,
                Section179IncomeK1Allocation {
                    kind: if source.partnership {
                        Section179IncomeSourceKind::Partnership
                    } else {
                        Section179IncomeSourceKind::Scorp
                    },
                    components: allocated
                        .pass_through_components
                        .get(&source.id)
                        .cloned()
                        .unwrap_or_default(),
                    allowed_cents: allowed,
                    business_income_carry_cents: crate::kernel::carry::native_balance(
                        source.amount + source.carryover,
                        allowed,
                    ),
                },
            )
        })
        .collect();
    Section179IncomeAllocation {
        system,
        business_income_limit_cents: allocated.income,
        total_allowed_cents: allocated.allowed,
        k1,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Section179DirectBusinessIncomeSource {
    pub source: i64,
    pub system: Section179IncomeSystem,
    pub operating_net_cents: Option<i64>,
    /// Legacy slot for the whole recognized business-property gain, including
    /// ordinary character; the source owner represents native annual Dollars here.
    pub section1231_net_cents: Option<i64>,
}
/// Actual other income-limited deduction for the selected tax system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Section179OtherLimitedDeduction {
    pub system: Section179IncomeSystem,
    pub amount_cents: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Section179PreparedIssue {
    MissingReturnContext,
    UnknownSource(i64),
    DuplicateSource(i64),
    MissingSource(i64),
    WrongSourceKind(i64),
    WrongSystem(i64),
    InvalidSource(i64, &'static str),
    K1Availability(i64, K1Section179BasisIssue),
    AssetIdentityUnavailable(i64),
    DuplicateAsset(i64),
    Association(crate::source_associations::SourceAssociationIssue),
    AlternativeAssetCarryUnavailable(i64, usize),
    SourceIncomeUnavailable(i64),
    SourceBoundaryUnavailable(i64, &'static str),
    NegativePriorLoss(i64),
    AmountOverflow(i64),
    AggregateAmountOverflow,
    NativeAmountOverflow,
    WageIncomeUnavailable,
    RiskUnavailable(i64),
    RiskOrderingCycle(i64),
    OtherDeductionUnavailable,
    OtherDeductionWrongSystem,
    NegativeOtherDeduction,
    OwnerIssues(Vec<crate::FederalIssue>),
}
/// Selected-system Form4562 income-limit results and bound PTP source inputs.
/// Direct-asset keys are actual accepted (income ID, asset ID) pairs. The same
/// native allocation remains bound to its report; no PAL ordering is selected.
#[derive(Clone, Debug, PartialEq)]
pub struct Section179PreparedAllocation {
    pub income: Section179IncomeAllocation,
    pub assets: BTreeMap<(i64, i64), i64>,
    // Bound to this exact selected allocation, never reconstructed by the reader.
    ptp: BTreeMap<i64, Result<Ptp179PalInput, Ptp179SourceIssue>>,
    native: Deduction,
    direct_operating:
        BTreeMap<i64, Result<direct_sources::DirectOperatingOperands, Section179PreparedIssue>>,
    owner_issues: Vec<crate::FederalIssue>,
    // Already-built selected projection, bound to the allocation/report.
    report_context: FederalDraft,
    pub(crate) home_context: crate::home_office::HomeContext,
    pub(crate) final_ordinary: BTreeMap<i64, crate::business_risk::FinalOrdinarySource>,
}

/// Borrowed office-profit operands from this invocation's selected projection.
/// The hypothetical allowance enters the same checked recipe as final reads.
pub(crate) struct Section179PreparedDirectOperands<'a> {
    draft: &'a FederalDraft,
    operating:
        &'a BTreeMap<i64, Result<direct_sources::DirectOperatingOperands, Section179PreparedIssue>>,
}
impl Section179PreparedDirectOperands<'_> {
    pub(crate) fn home_income(
        &self,
        source: i64,
        allowed: i64,
    ) -> Result<crate::home_office::HomeIncome, Section179PreparedIssue> {
        let operands = self
            .operating
            .get(&source)
            .ok_or(Section179PreparedIssue::MissingSource(source))?
            .as_ref()
            .map_err(Clone::clone)?;
        let entry = self
            .draft
            .income
            .iter()
            .find(|entry| entry.id == source)
            .ok_or(Section179PreparedIssue::UnknownSource(source))?;
        let FederalIncome::Business { data } = &entry.income else {
            return Err(Section179PreparedIssue::WrongSourceKind(source));
        };
        crate::home_office::HomeIncome::from_source(
            self.draft,
            source,
            data,
            operands
                .cost_cents
                .checked_add(allowed)
                .ok_or(Section179PreparedIssue::AmountOverflow(source))?,
        )
    }
}

/// Report view for the exact PTP inputs validated and applied by a consumer.
/// This view does not perform or certify that consumer's passive-loss work.
#[derive(Debug)]
pub struct Section179PtpReportContext<'a> {
    allocation: &'a Section179PreparedAllocation,
    consumed: &'a [&'a Ptp179PalInput],
}
impl Section179PtpReportContext<'_> {
    /// Emit the bound native result once, retaining every other owner boundary.
    pub fn report(&self, result: &mut FederalResult) {
        report_allocated_with_consumed(
            &self.allocation.report_context,
            result,
            &self.allocation.native,
            |id| self.consumed.iter().any(|source| source.k1 == id),
        );
    }
}

impl Section179PreparedAllocation {
    pub(crate) fn native_allowed(&self, activity: i64) -> i64 {
        self.native.pass_through.get(&activity).copied().unwrap_or(0)
    }
    fn direct_business(&self, source: i64) -> Result<&FederalBusiness, Section179PreparedIssue> {
        let entry = self
            .report_context
            .income
            .iter()
            .find(|entry| entry.id == source)
            .ok_or(Section179PreparedIssue::UnknownSource(source))?;
        let FederalIncome::Business { data } = &entry.income else {
            return Err(Section179PreparedIssue::WrongSourceKind(source));
        };
        if !self.owner_issues.is_empty() {
            return Err(Section179PreparedIssue::OwnerIssues(self.owner_issues.clone()));
        }
        Ok(data)
    }

    fn direct_operands(
        &self,
        source: i64,
    ) -> Result<&direct_sources::DirectOperatingOperands, Section179PreparedIssue> {
        self.direct_business(source)?;
        self.direct_operating
            .get(&source)
            .ok_or(Section179PreparedIssue::MissingSource(source))?
            .as_ref()
            .map_err(Clone::clone)
    }

    /// Final physical Schedule C source after its current at-risk stage.
    /// No community-income allocation is applied to these source cents.
    pub(crate) fn final_ordinary_profit(&self, source: i64) -> Option<i64> {
        self.final_ordinary.get(&source).map(|source| source.profit_cents)
    }

    // Gross native election/carry availability precedes the income cap.
    pub(crate) fn available_section179(&self) -> i64 {
        self.native.available
    }

    // Absence means this bound source has no current ordinary at-risk stage.
    // Unknown sources fail preparation before an allocation reaches a consumer.
    pub(crate) fn ordinary_risk_adjustment(&self, source: i64) -> i64 {
        self.final_ordinary.get(&source).map_or(0, |source| source.adjustment_cents)
    }

    /// Selected operating expense cents, including this allocation's allowed179.
    /// The actual source ID resolves only in this allocation's prepared view.
    pub fn operating(&self, source: i64) -> Result<i64, Section179PreparedIssue> {
        self.direct_operands(source)?
            .cost_cents
            .checked_add(self.allowed(source)?)
            .ok_or(Section179PreparedIssue::AmountOverflow(source))
    }

    /// This business's actual Form4562 line12 cents, without allocator replay.
    pub fn allowed(&self, source: i64) -> Result<i64, Section179PreparedIssue> {
        self.direct_business(source)?;
        Ok(self.native.businesses.get(&source).copied().unwrap_or(0))
    }

    /// Selected expense cents through the existing office/gambling owner.
    pub fn expenses(&self, source: i64) -> Result<i64, Section179PreparedIssue> {
        Ok(self
            .direct_business(source)?
            .expense_with_office(self.operating(source)?, self.office(source)?))
    }

    /// The existing business-receipt stage less selected operating costs.
    /// Routed receipts and other income adjustments keep their existing owners.
    pub fn tentative_profit(&self, source: i64) -> Result<i64, Section179PreparedIssue> {
        self.direct_operands(source)?.tentative_profit(source, self.allowed(source)?)
    }

    pub(crate) fn home_income(
        &self,
        source: i64,
    ) -> Result<crate::home_office::HomeIncome, Section179PreparedIssue> {
        crate::home_office::HomeIncome::from_source(
            &self.report_context,
            source,
            self.direct_business(source)?,
            self.operating(source)?,
        )
    }

    /// Existing home-office deduction cents using this same selected view.
    pub fn office(&self, source: i64) -> Result<i64, Section179PreparedIssue> {
        self.direct_business(source)?;
        let profits = self
            .report_context
            .income
            .iter()
            .filter(|entry| matches!(entry.income, FederalIncome::Business { .. }))
            .map(|entry| self.home_income(entry.id).map(|income| (entry.id, income)))
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        crate::home_office::deduction_with_context(
            &self.report_context,
            source,
            &profits,
            &self.home_context,
        )
    }

    /// Bind inputs that the caller has validated and applied to its PTP result.
    /// Every reference must come from this exact allocation's source getter;
    /// another allocation, system, or cloned source cannot establish consumption.
    pub fn ptp_report_context<'a>(
        &'a self,
        consumed: &'a [&'a Ptp179PalInput],
    ) -> Result<Section179PtpReportContext<'a>, Ptp179SourceIssue> {
        for source in consumed {
            if !core::ptr::eq(self.ptp_section179_pal_input(source.k1)?, *source) {
                return Err(Ptp179SourceIssue::DifferentAllocation);
            }
        }
        Ok(Section179PtpReportContext { allocation: self, consumed })
    }

    /// Read the selected PTP source after this invocation's availability and
    /// before179 risk checks. No allocator, basis or risk owner runs here.
    pub fn ptp_section179_pal_input(&self, k1: i64) -> Result<&Ptp179PalInput, Ptp179SourceIssue> {
        let source = match self.ptp.get(&k1) {
            Some(source) => source,
            None if self.income.k1.contains_key(&k1) => return Err(Ptp179SourceIssue::NotPtp),
            None => return Err(Ptp179SourceIssue::UnknownSource),
        };
        if !self.owner_issues.is_empty() {
            return Err(Ptp179SourceIssue::OwnerIssues(self.owner_issues.clone()));
        }
        source.as_ref().map_err(Clone::clone)
    }
}

// Bind the existing native no-election path without demanding an unrelated
// activity's before179 income certification. Every actual expense source stays
// in the same selected view. K-1 sources use their owner's actual selected
// basis allowance and retained income carry before any income source is needed.
pub(crate) fn no_section179_context(
    draft: &FederalDraft,
    system: Section179IncomeSystem,
    home: &crate::home_office::HomeContext,
) -> Result<Option<Section179PreparedAllocation>, Section179PreparedIssue> {
    for entry in &draft.income {
        let relevant = match &entry.income {
            FederalIncome::Partnership { data } => {
                data.section179 != 0
                    || data.section179_carryover != 0
                    || data
                        .amt_section179_income_carry
                        .as_ref()
                        .is_some_and(|carry| carry.gross != 0)
            }
            FederalIncome::Scorp { data } => {
                data.section179 != 0
                    || data.section179_carryover != 0
                    || data
                        .amt_section179_income_carry
                        .as_ref()
                        .is_some_and(|carry| carry.gross != 0)
            }
            _ => false,
        };
        if !relevant {
            continue;
        }
        let source = k1_section179_availability_source(
            draft,
            entry.id,
            system == Section179IncomeSystem::Alternative,
        )
        .map_err(|issue| Section179PreparedIssue::K1Availability(entry.id, issue))?;
        if source.current.basis_allowed_cents != 0
            || crate::kernel::carry::native_balance(
                source.gross_income_carry_cents,
                source.sold_income_carry_cents,
            ) != 0
        {
            return Ok(None);
        }
    }
    let mut selected = draft.clone();
    if system == Section179IncomeSystem::Alternative {
        for entry in &mut selected.income {
            if let FederalIncome::Business { data } = &mut entry.income {
                for (index, asset) in data.assets.iter_mut().enumerate() {
                    asset.amt_section179_carry().map_err(|()| {
                        Section179PreparedIssue::AlternativeAssetCarryUnavailable(entry.id, index)
                    })?;
                    *asset = asset.amt_projection_or_self();
                }
            }
        }
    }
    let collected = collect_section179_items::<true>(&selected, pass_throughs(&selected)?);
    if let Some(issue) = &collected.result.source_issue {
        return Err(issue.clone());
    }
    if collected.result.prepared_income_overflow {
        return Err(Section179PreparedIssue::AggregateAmountOverflow);
    }
    if collected.result.available != 0 {
        return Ok(None);
    }
    let native = calculate_with_home(&selected, home)?;
    let income = income_allocation(&native, system);
    // The native available==0 branch does not need line11 income. Its report
    // returns before line11, and no certified before179 income is substituted.
    let direct_operating =
        direct_sources::selected_operating_operands(draft, &selected, system, Some(home));
    Ok(Some(Section179PreparedAllocation {
        income,
        assets: BTreeMap::new(),
        ptp: BTreeMap::new(),
        native,
        direct_operating,
        owner_issues: Vec::new(),
        report_context: selected,
        home_context: home.clone(),
        final_ordinary: BTreeMap::new(),
    }))
}

// Shared selected projection for the native allocator and its before179 input
// owner. Keep validation and actual AMT carry availability in their native order.
fn prepare_direct_business<'a>(
    entry: &'a mut crate::FederalIncomeEntry,
    alternative: bool,
    associations: &crate::source_associations::SourceAssociations<'_>,
    asset_ids: &mut alloc::collections::BTreeSet<i64>,
) -> Result<&'a mut FederalBusiness, Section179PreparedIssue> {
    use Section179PreparedIssue as E;
    let id = entry.id;
    crate::federal_rules::validate_income(entry).map_err(|(key, _)| E::InvalidSource(id, key))?;
    let FederalIncome::Business { data } = &mut entry.income else {
        return Err(E::WrongSourceKind(id));
    };
    for asset in &data.assets {
        if asset.id <= 0 {
            return Err(E::AssetIdentityUnavailable(id));
        }
        if !asset_ids.insert(asset.id) {
            return Err(E::DuplicateAsset(asset.id));
        }
    }
    for (index, asset) in data.assets.iter_mut().enumerate() {
        associations.asset(id, asset.id).map_err(E::Association)?;
        if alternative {
            asset
                .amt_section179_carry()
                .map_err(|()| E::AlternativeAssetCarryUnavailable(id, index))?;
            // Null means the actual selected method/history/carry agree,
            // only after the independently sourced carry check above.
            *asset = asset.amt_projection_or_self();
        }
    }
    Ok(data)
}

/// Allocate actual selected-system sources with each K-1's native before179
/// risk result. The hypothetical deduction callback borrows this same projection.
#[cfg(test)]
pub(crate) fn section179_with_native_prepared_income_and_operating_context(
    draft: &FederalDraft,
    system: Section179IncomeSystem,
    businesses: &[Section179DirectBusinessIncomeSource],
    k1s: &[Section179BusinessIncomeSource],
    other_limited_deduction: impl FnOnce(
        &FederalDraft,
        &BTreeMap<i64, i64>,
        &Section179PreparedDirectOperands<'_>,
    ) -> Option<Section179OtherLimitedDeduction>,
) -> Result<Section179PreparedAllocation, Section179PreparedIssue> {
    section179_with_pass_through_context(
        draft,
        system,
        businesses,
        k1s,
        None,
        &[],
        other_limited_deduction,
    )
}

// Entity property facts remain with the entity; these are receiving activity cents.
pub(crate) struct Section179PassThrough {
    pub activity: i64,
    pub amount_cents: i64,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn section179_with_pass_through_context(
    draft: &FederalDraft,
    system: Section179IncomeSystem,
    businesses: &[Section179DirectBusinessIncomeSource],
    k1s: &[Section179BusinessIncomeSource],
    home: Option<&crate::home_office::HomeContext>,
    pass_through: &[Section179PassThrough],
    other_limited_deduction: impl FnOnce(
        &FederalDraft,
        &BTreeMap<i64, i64>,
        &Section179PreparedDirectOperands<'_>,
    ) -> Option<Section179OtherLimitedDeduction>,
) -> Result<Section179PreparedAllocation, Section179PreparedIssue> {
    use Section179PreparedIssue as E;
    if !draft.home_sites.is_empty() && home.is_none() {
        return Err(E::SourceBoundaryUnavailable(0, "HOME_CONTEXT_REQUIRED"));
    }
    if draft.setup.is_none() {
        return Err(E::MissingReturnContext);
    }
    let associations = crate::source_associations::SourceAssociations::new(draft);
    let alternative = system == Section179IncomeSystem::Alternative;
    let mut prepared_draft = draft.clone();
    let mut seen = alloc::collections::BTreeSet::new();
    let mut asset_ids = alloc::collections::BTreeSet::new();
    let mut contributions = BTreeMap::new();
    for source in businesses {
        let id = source.source;
        if !seen.insert(id) {
            return Err(E::DuplicateSource(id));
        }
        if source.system != system {
            return Err(E::WrongSystem(id));
        }
        associations.income(id).map_err(|_| E::UnknownSource(id))?;
        let entry = prepared_draft
            .income
            .iter_mut()
            .find(|entry| entry.id == id)
            .filter(|_| id > 0)
            .ok_or(E::UnknownSource(id))?;
        prepare_direct_business(entry, alternative, &associations, &mut asset_ids)?;
        let operating = source.operating_net_cents.ok_or(E::SourceIncomeUnavailable(id))?;
        let property = source.section1231_net_cents.ok_or(E::SourceIncomeUnavailable(id))?;
        contributions.insert(id, operating.checked_add(property).ok_or(E::AmountOverflow(id))?);
    }
    let mut pass_throughs = Vec::new();
    let mut risk_remaining = BTreeMap::new();
    let mut ptp_sources = BTreeMap::new();
    for source in k1s {
        let id = source.source;
        if !seen.insert(id) {
            return Err(E::DuplicateSource(id));
        }
        if source.system != system {
            return Err(E::WrongSystem(id));
        }
        if matches!(
            draft.income.iter().find(|entry| entry.id == id).map(|entry| &entry.income),
            Some(
                FederalIncome::PartnershipActivities { .. } | FederalIncome::ScorpActivities { .. }
            )
        ) {
            let net = [source.ordinary, source.section1231].into_iter().try_fold(
                0i64,
                |net, class| {
                    if class.prior_allowed_loss_cents < 0 {
                        return Err(E::NegativePriorLoss(id));
                    }
                    net.checked_add(class.current_net_cents)
                        .and_then(|net| net.checked_sub(class.prior_allowed_loss_cents))
                        .ok_or(E::AmountOverflow(id))
                },
            )?;
            contributions.insert(id, net);
            continue;
        }
        let availability = k1_section179_availability_source(draft, id, alternative)
            .map_err(|issue| E::K1Availability(id, issue))?;
        let remaining = availability.risk.remaining_cents;
        if remaining.is_some_and(|amount| amount < 0)
            || (remaining.is_none()
                && (availability.current.basis_allowed_cents > 0
                    || availability.gross_income_carry_cents
                        > availability.sold_income_carry_cents))
        {
            return Err(E::RiskUnavailable(id));
        }
        if let Some(remaining) = remaining {
            risk_remaining.insert(id, remaining);
        }
        let mut net = 0i64;
        for class in [source.ordinary, source.section1231] {
            if class.prior_allowed_loss_cents < 0 {
                return Err(E::NegativePriorLoss(id));
            }
            net = net
                .checked_add(class.current_net_cents)
                .and_then(|net| net.checked_sub(class.prior_allowed_loss_cents))
                .ok_or(E::AmountOverflow(id))?;
        }
        let mut income = net;
        let original = draft.income.iter().find(|entry| entry.id == id).unwrap();
        let entry = crate::community::entry(draft, original);
        let (material, publicly_traded, owner_deductions, guarantees) = match &entry.income {
            FederalIncome::Partnership { data } => {
                let material =
                    data.participation.material() == crate::participation::Participation::Yes;
                // Entity ordinary/property classes exclude personal payments.
                // Add actual service cents once after their selected-system
                // net; retain capital and PTP payments in the boundary guard.
                let services = partner_service_compensation(data);
                income = income.checked_add(services).ok_or(E::AmountOverflow(id))?;
                let guarantees = if material {
                    data.guaranteed_services
                        .checked_add(data.guaranteed_capital)
                        .and_then(|total| total.checked_sub(services))
                        .ok_or(E::AmountOverflow(id))?
                } else {
                    0
                };
                (
                    material,
                    data.publicly_traded,
                    !matches!(data.health_insurance, crate::BusinessHealthInsurance::None)
                        || data.pensions != 0
                        || data.retirement != crate::PartnerRetirement::None,
                    guarantees,
                )
            }
            FederalIncome::Scorp { data } => (
                data.participation
                    .as_ref()
                    .is_some_and(|p| p.material() == crate::participation::Participation::Yes),
                false,
                false,
                0,
            ),
            _ => unreachable!(),
        };
        contributions.insert(id, income);
        if let FederalIncome::Partnership { data } = &entry.income
            && data.publicly_traded
        {
            // Prepared activity income intentionally excludes guaranteed
            // payments; retain the actual original source payments here.
            ptp_sources.insert(
                id,
                (material, income, data.guaranteed_services, data.guaranteed_capital, remaining),
            );
        }
        pass_throughs.push(PassThrough {
            id,
            partnership: availability.current.partnership,
            amount: availability.current.basis_allowed_cents,
            carryover: crate::kernel::carry::native_balance(
                availability.gross_income_carry_cents,
                availability.sold_income_carry_cents,
            ),
            income_carry_vintages: availability.income_carry_vintages,
            sold_carryover: availability.sold_income_carry_cents,
            income,
            guarantees,
            risk_end: remaining,
            material,
            publicly_traded,
            owner_deductions,
        });
    }
    let mut accepted_ids = alloc::collections::BTreeSet::new();
    let mut income = 0i64;
    for entry in &draft.income {
        if associations.income(entry.id).is_err() || !accepted_ids.insert(entry.id) {
            return Err(E::DuplicateSource(entry.id));
        }
        let amount = match &entry.income {
            FederalIncome::Business { .. }
            | FederalIncome::Partnership { .. }
            | FederalIncome::Scorp { .. }
            | FederalIncome::PartnershipActivities { .. }
            | FederalIncome::ScorpActivities { .. } => {
                *contributions.get(&entry.id).ok_or(E::MissingSource(entry.id))?
            }
            FederalIncome::Wage { data } => data
                .total_income()
                .map(|value| {
                    value
                        - if data.statutory_employee && data.statutory_business.is_some() {
                            data.wages
                        } else {
                            0
                        }
                        - data.disability.amount()
                })
                .map_err(|_| E::WageIncomeUnavailable)?,
            FederalIncome::EmployeeWages { data } => {
                data.income().map_err(|_| E::WageIncomeUnavailable)?
            }
            _ => 0,
        };
        income = income.checked_add(amount).ok_or(E::AggregateAmountOverflow)?;
    }
    let Collected179 { mut result, mut items, mut allocatable } =
        collect_section179_items::<true>(&prepared_draft, pass_throughs);
    if let Some(issue) = &result.source_issue {
        return Err(issue.clone());
    }
    let mut native_ids = alloc::collections::BTreeSet::new();
    for source in pass_through {
        let id = source.activity;
        let actual = draft.income.iter().any(|entry| {
            let activities = match &entry.income {
                FederalIncome::PartnershipActivities { data } => &data.activities,
                FederalIncome::ScorpActivities { data } => &data.activities,
                _ => return false,
            };
            contributions.contains_key(&entry.id)
                && activities.source().is_some_and(|source| {
                    source.components.iter().any(|component| component.id == id)
                })
        });
        if !actual || id <= 0 {
            return Err(E::UnknownSource(id));
        }
        if !native_ids.insert(id) {
            return Err(E::DuplicateSource(id));
        }
        if source.amount_cents < 0 {
            return Err(E::InvalidSource(id, "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE"));
        }
        result.elected =
            result.elected.checked_add(source.amount_cents).ok_or(E::NativeAmountOverflow)?;
        result.available =
            result.available.checked_add(source.amount_cents).ok_or(E::NativeAmountOverflow)?;
        allocatable =
            allocatable.checked_add(source.amount_cents).ok_or(E::NativeAmountOverflow)?;
        items.push((id, PASS_THROUGH, source.amount_cents, 0));
    }
    if result.prepared_income_overflow {
        return Err(E::AggregateAmountOverflow);
    }
    let direct_operating =
        direct_sources::selected_operating_operands(draft, &prepared_draft, system, home);
    if result.available == 0 {
        result.income_known = true;
    } else {
        result.income_known = true;
        let mut other_issue = None;
        result = allocate_income_stage::<true>(result, &items, allocatable, income, |allocation| {
            let operands = Section179PreparedDirectOperands {
                draft: &prepared_draft,
                operating: &direct_operating,
            };
            let source = other_limited_deduction(&prepared_draft, allocation, &operands)?;
            if source.system != system {
                other_issue = Some(E::OtherDeductionWrongSystem);
                None
            } else if source.amount_cents < 0 {
                other_issue = Some(E::NegativeOtherDeduction);
                None
            } else {
                Some(source.amount_cents)
            }
        });
        if let Some(issue) = other_issue {
            return Err(issue);
        }
        if result.prepared_income_overflow {
            return Err(E::AggregateAmountOverflow);
        }
        if !result.income_known {
            return Err(E::OtherDeductionUnavailable);
        }
    }
    for (id, remaining) in risk_remaining {
        if result.pass_through.get(&id).copied().unwrap_or(0) > remaining {
            return Err(E::RiskOrderingCycle(id));
        }
    }
    // Preserve the existing owner-policy checks without recalculating the
    // allocation. Only the two notices belonging to the receiving PAL owner
    // are excluded from the source getter; report() still emits them.
    let mut checked = FederalResult::empty(prepared_draft.revision);
    report_allocated(&prepared_draft, &mut checked, &result);
    checked.issues.retain(|issue| {
        !matches!(
            issue.display.key.as_str(),
            "ISSUE_SECTION179_PUBLICLY_TRADED" | "ISSUE_SECTION179_PASSIVE_ACTIVITY"
        )
    });
    let income = income_allocation(&result, system);
    let ptp = ptp_sources
        .into_iter()
        .map(
            |(
                id,
                (
                    active,
                    owner_income_cents,
                    guaranteed_services_cents,
                    guaranteed_capital_cents,
                    remaining,
                ),
            )| {
                let allocated = income.k1.get(&id).expect("collected accepted K1 source");
                let value =
                    remaining.ok_or(Ptp179SourceIssue::RiskUnavailable).map(|_| Ptp179PalInput {
                        k1: id,
                        alternative,
                        active,
                        owner_income_cents,
                        guaranteed_services_cents,
                        guaranteed_capital_cents,
                        components: allocated.components.clone(),
                        allowed_cents: allocated.allowed_cents,
                        business_income_carry_cents: allocated.business_income_carry_cents,
                    });
                (id, value)
            },
        )
        .collect();
    let assets = result
        .assets
        .iter()
        .map(|(&(id, index), &allowed)| {
            let FederalIncome::Business { data } = &prepared_draft
                .income
                .iter()
                .find(|entry| entry.id == id)
                .expect("collected accepted direct source")
                .income
            else {
                unreachable!()
            };
            ((id, data.assets[index].id), allowed)
        })
        .collect();
    Ok(Section179PreparedAllocation {
        income,
        assets,
        ptp,
        native: result,
        direct_operating,
        owner_issues: checked.issues,
        report_context: prepared_draft,
        home_context: home.cloned().unwrap_or_default(),
        final_ordinary: BTreeMap::new(),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct K1Section179AvailabilitySource {
    pub current: K1Section179BasisSource,
    /// Actual native before179 risk amount; None remains unavailable.
    /// Cents representation of the native annual-Dollar source boundary.
    pub risk: K1Section179RiskInput,
    pub gross_income_carry_cents: i64,
    pub sold_income_carry_cents: i64,
    pub income_carry_vintages: Option<Vec<crate::Asset179Carryover>>,
}

/// Use this checked boundary for AMT availability. The older activity
/// projection is not an actual AMT income-carry source when its history is null.
pub fn k1_section179_availability_source(
    draft: &FederalDraft,
    k1: i64,
    alternative: bool,
) -> Result<K1Section179AvailabilitySource, K1Section179BasisIssue> {
    use K1Section179BasisIssue as E;
    if draft.setup.is_none() {
        return Err(E::MissingReturnContext);
    }
    let original =
        crate::source_associations::income_source(draft, k1).map_err(|issue| match issue {
            crate::source_associations::SourceAssociationIssue::AmbiguousSource(_) => {
                E::InvalidSource("K1_SOURCE_ID_TWICE")
            }
            _ => E::UnknownSource,
        })?;
    if !matches!(original.income, FederalIncome::Partnership { .. } | FederalIncome::Scorp { .. }) {
        return Err(E::NotK1);
    }
    crate::federal_rules::validate_income(original).map_err(|(key, _)| E::InvalidSource(key))?;
    let entry = crate::community::entry(draft, original);
    let (gross, sold, amt_history) = match &entry.income {
        FederalIncome::Partnership { data } => (
            data.section179_carryover,
            data.section179_carryover_sold,
            data.amt_section179_income_carry.as_ref(),
        ),
        FederalIncome::Scorp { data } => (
            data.section179_carryover,
            data.section179_carryover_sold,
            data.amt_section179_income_carry.as_ref(),
        ),
        _ => return Err(E::NotK1),
    };
    let (gross, sold, vintages) = if alternative {
        let history = amt_history.ok_or(E::AlternativeIncomeCarryUnavailable)?;
        if let Some(key) = history.rule() {
            return Err(E::InvalidSource(key));
        }
        (history.gross, history.sold, Some(history.vintages.clone()))
    } else {
        (gross, sold, None)
    };
    let source = k1_section179_current_source(draft, k1, alternative)?;
    Ok(K1Section179AvailabilitySource {
        current: source.basis,
        risk: source.risk,
        gross_income_carry_cents: gross,
        sold_income_carry_cents: sold,
        income_carry_vintages: vintages,
    })
}

/// Source component retained by the sole return-wide section179 allocator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptp179Component {
    Current,
    /// None means the existing source retains only an aggregate income carry.
    /// Never infer a vintage from the allocator's provisional year bucket.
    PriorIncomeCarry {
        original_year: Option<u32>,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ptp179ComponentAllocation {
    pub component: Ptp179Component,
    pub available_cents: i64,
    pub allowed_cents: i64,
}
/// The risk owner's actual before-section179 result, with its source identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct K1Section179RiskInput {
    pub k1: i64,
    pub alternative: bool,
    pub remaining_cents: Option<i64>,
}
/// Only allowed_cents enters the PTP's current PAL section179 class. Income
/// carry stays with Form4562; prior PTP-PAL losses stay with the PAL owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ptp179PalInput {
    pub k1: i64,
    pub alternative: bool,
    pub active: bool,
    pub owner_income_cents: i64,
    pub guaranteed_services_cents: i64,
    pub guaranteed_capital_cents: i64,
    pub components: Vec<Ptp179ComponentAllocation>,
    pub allowed_cents: i64,
    pub business_income_carry_cents: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ptp179SourceIssue {
    UnknownSource,
    NotPtp,
    /// The consumed input is not borrowed from this exact selected allocation.
    DifferentAllocation,
    OwnerIssues(Vec<crate::FederalIssue>),
    RiskUnavailable,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Deduction {
    pub income_known: bool,
    prepared_income_overflow: bool,
    source_issue: Option<Section179PreparedIssue>,
    pub cost: i64,
    pub spouse_cost: i64,
    pub share: i64,
    pub shared_limit: i64,
    pub elected: i64,
    pub available: i64,
    pub limit: i64,
    pub income: i64,
    pub allowed: i64,
    pub assets: BTreeMap<(i64, usize), i64>,
    pub businesses: BTreeMap<i64, i64>,
    // K-1 shares allowed on line 12, by entry ID.
    pub pass_through: BTreeMap<i64, i64>,
    pub pass_through_components: BTreeMap<i64, Vec<Ptp179ComponentAllocation>>,
    pub pass_throughs: Vec<PassThrough>,
    // Guaranteed payments of materially participating partners, kept out of
    // line 11 (see PassThrough::guarantees).
    pub guarantees: i64,
}

pub(crate) fn calculate(d: &FederalDraft) -> Deduction {
    match pass_throughs(d) {
        Ok(sources) => calculate_with_pass_throughs::<false>(d, sources, None, &mut None),
        Err(_) => Deduction {
            income_known: false,
            prepared_income_overflow: true,
            ..Deduction::default()
        },
    }
}

pub(crate) fn calculate_with_home(
    d: &FederalDraft,
    home: &crate::home_office::HomeContext,
) -> Result<Deduction, Section179PreparedIssue> {
    let mut issue = None;
    let result =
        calculate_with_pass_throughs::<false>(d, pass_throughs(d)?, Some(home), &mut issue);
    if let Some(issue) = &result.source_issue {
        return Err(issue.clone());
    }
    if let Some(issue) = issue {
        return Err(issue);
    }
    if !result.income_known {
        return Err(Section179PreparedIssue::SourceIncomeUnavailable(0));
    }
    Ok(result)
}

// Only the new transient input path checks aggregation of externally prepared
// amounts. The legacy producer retains its original native arithmetic. Both
// paths use this same owner, limits, ordering and allocation below.
type Election179Item = (i64, usize, i64, u32);

struct Collected179 {
    result: Deduction,
    items: Vec<Election179Item>,
    allocatable: i64,
}

// The source collector retains the native tuple/index and aggregate carry
// buckets. Selected-system preparation reuses it without changing allocation.
fn collect_section179_items<const PREPARED: bool>(
    d: &FederalDraft,
    sources: Vec<PassThrough>,
) -> Collected179 {
    let mut result = Deduction::default();
    let mut items = Vec::new();
    let mut allocatable = 0i64;
    result.pass_throughs = sources;
    macro_rules! add {
        ($target:expr, $value:expr) => {
            if PREPARED {
                let Some(sum) = $target.checked_add($value) else {
                    result.prepared_income_overflow = true;
                    return Collected179 { result, items, allocatable };
                };
                $target = sum;
            } else {
                $target += $value;
            }
        };
    }
    // Reg. 1.179-2(b)(3)(i): each K-1 share joins the partner's own elections
    // on lines 6 and 8, but the entity's property never counts toward the
    // partner's line 2 phaseout.
    for k1 in &result.pass_throughs {
        if k1.amount > 0 {
            add!(result.elected, k1.amount);
            items.push((k1.id, PASS_THROUGH, k1.amount, 0));
        }
        // Line 10: a carryover is not a new election (line 8), and follows
        // the current elections in the allocation.
        if let Some(vintages) = &k1.income_carry_vintages {
            for vintage in vintages {
                items.push((k1.id, PASS_THROUGH, vintage.amount, vintage.year));
            }
        } else if k1.carryover > 0 {
            items.push((k1.id, PASS_THROUGH, k1.carryover, 2025));
        }
        add!(result.guarantees, k1.guarantees);
    }
    let projected_entries = crate::community::entries(d);
    for e in &projected_entries {
        if let FederalIncome::Business { data } = &e.income {
            for (index, a) in data.assets.iter().enumerate() {
                let cost = match crate::cooperative_basis::qualified179_cost(d, e.id, a) {
                    Ok(cost) => cost,
                    Err(issue) => {
                        result.source_issue = Some(issue);
                        return Collected179 { result, items, allocatable };
                    }
                };
                add!(result.cost, cost);
                add!(result.elected, a.elected179());
                if a.available179() > 0 {
                    items.push((
                        e.id,
                        index,
                        a.available179(),
                        if a.placed / 10000 == 2026 { 0 } else { a.placed / 10000 },
                    ));
                }
            }
        }
    }
    if PREPARED {
        for item in &items {
            add!(result.available, item.2);
        }
    } else {
        result.available = items.iter().map(|i| i.2).sum();
    }
    // Form 4562 retains gross unused elected cost. A documented passenger-auto
    // reserve releases only its original schedule's annual section 280F capacity.
    // Read that capacity from the same projected source that supplied the item.
    for (id, index, amount, _) in &mut items {
        if *index != PASS_THROUGH
            && let Some(entry) = projected_entries.iter().find(|entry| entry.id == *id)
            && let FederalIncome::Business { data } = &entry.income
        {
            *amount = phase::cap(*amount, [data.assets[*index].annual179_capacity()]);
        }
    }
    if PREPARED {
        for item in &items {
            add!(allocatable, item.2);
        }
    } else {
        allocatable = items.iter().map(|item| item.2).sum();
    }
    drop(projected_entries);
    let (spouse_cost, share, _) = spouse(d).map_or((0, 10000, true), Section179Spouse::facts);
    result.spouse_cost = spouse_cost;
    result.share = share;
    let mut combined_cost = result.cost;
    add!(combined_cost, spouse_cost);
    result.shared_limit = phase::phase_out(
        crate::year_constants::SECTION_179_DOLLAR_LIMIT_2026_CENTS,
        positive(combined_cost - crate::year_constants::SECTION_179_PHASEOUT_THRESHOLD_2026_CENTS),
    );
    result.limit = fraction(result.shared_limit, share, 10000);
    Collected179 { result, items, allocatable }
}

fn calculate_with_pass_throughs<const PREPARED: bool>(
    d: &FederalDraft,
    sources: Vec<PassThrough>,
    home: Option<&crate::home_office::HomeContext>,
    home_issue: &mut Option<Section179PreparedIssue>,
) -> Deduction {
    let Collected179 { mut result, items, allocatable } =
        collect_section179_items::<PREPARED>(d, sources);
    if result.source_issue.is_some() {
        return result;
    }
    if result.available == 0 {
        result.income_known = true;
        return result;
    }
    let mq = crate::depreciation::mid_quarter(d);
    // Before section 179 and the income-limited home-office deduction.
    let mut source_issue = None;
    let mut income = d.income.iter().map(|e| match &e.income {
        FederalIncome::Wage { data } => data.total_income().map(|value| {
            // Linked statutory box 1 already supplies this Schedule C's
            // receipts below; count that job's net active income once.
            value
                - if data.statutory_employee && data.statutory_business.is_some() {
                    data.wages
                } else {
                    0
                }
                - data.disability.amount()
        }),
        FederalIncome::EmployeeWages { data } => data.income(),
        FederalIncome::Business { data } => {
            let source = (|| {
                let receipts = crate::federal_rules::farm_receipts(d, e.id, data)?;
                let cost = crate::cooperative_basis::operating_cost(d, e.id, data, mq, false)?
                    .checked_add(crate::farm_conservation::expense_adjustment(d, e.id, data))
                    .ok_or(Section179PreparedIssue::AmountOverflow(e.id))?;
                receipts
                    .checked_sub(data.operating_cost_with_supplies(cost))
                    .ok_or(Section179PreparedIssue::AmountOverflow(e.id))
            })();
            match source {
                Ok(amount) => Ok(amount),
                Err(issue) => {
                    source_issue = Some(issue);
                    Err("section179 business source unavailable")
                }
            }
        }
        // The partner's or shareholder's allocable share of the entity's
        // active-business taxable income (Reg. 1.179-2(c)(2)(v), (c)(3)).
        FederalIncome::Partnership { .. } | FederalIncome::Scorp { .. } => {
            Ok(result.pass_throughs.iter().find(|k1| k1.id == e.id).map_or(0, |k1| k1.income))
        }
        _ => Ok(0),
    });
    const INPUT_OVERFLOW: &str = "prepared section179 income overflow";
    let income: Result<i64, &str> = if PREPARED {
        income.try_fold(0i64, |total, value| total.checked_add(value?).ok_or(INPUT_OVERFLOW))
    } else {
        income.sum()
    };
    if let Some(issue) = source_issue {
        result.source_issue = Some(issue);
        return result;
    }
    // The main income pass reports unresolved wage facts and withholds totals.
    let Ok(income) = income else {
        result.prepared_income_overflow = income == Err(INPUT_OVERFLOW);
        return result;
    };
    result.income_known = true;
    // Pub. 946 includes active-business section 1231 gains/losses in this
    // limit, regardless of their later ordinary/capital characterization.
    // Disposition proceeds remain outside operating receipts and SE income.
    let property_income = crate::asset_sale::direct_sources(d, false)
        .iter()
        // A farm rental takes no section 179, and its gain is not
        // active-business income.
        .filter(|(id, _)| {
            d.income
                .iter()
                .any(|e| e.id == *id && matches!(e.income, FederalIncome::Business { .. }))
        })
        .map(|(_, v)| v.gain * 100)
        .sum::<i64>();
    let income = if PREPARED {
        let Some(value) = income.checked_add(property_income) else {
            result.prepared_income_overflow = true;
            return result;
        };
        value
    } else {
        income + property_income
    };
    allocate_income_stage::<PREPARED>(
        result,
        &items,
        allocatable,
        income,
        |hypothetical_by_business| {
            if d.home_sites.is_empty() {
                return Some(0);
            }
            let amount = (|| {
                let home = home.ok_or(Section179PreparedIssue::SourceBoundaryUnavailable(
                    0,
                    "HOME_CONTEXT_REQUIRED",
                ))?;
                let inputs = d
                    .income
                    .iter()
                    .filter_map(|entry| {
                        let FederalIncome::Business { data } = &entry.income else {
                            return None;
                        };
                        let conservation =
                            crate::farm_conservation::expense_adjustment(d, entry.id, data);
                        Some(
                            crate::home_office::HomeIncome::from_source(
                                d,
                                entry.id,
                                data,
                                data.operating_cost_with_adjustment(
                                    mq,
                                    conservation
                                        + hypothetical_by_business
                                            .get(&entry.id)
                                            .copied()
                                            .unwrap_or(0),
                                ),
                            )
                            .map(|income| (entry.id, income)),
                        )
                    })
                    .collect::<Result<BTreeMap<_, _>, _>>()?;
                let mut total = 0i64;
                for source in inputs.keys() {
                    let deduction =
                        crate::home_office::deduction_with_context(d, *source, &inputs, home)?;
                    total = total
                        .checked_add(deduction)
                        .ok_or(Section179PreparedIssue::AggregateAmountOverflow)?;
                }
                Ok::<_, Section179PreparedIssue>(total)
            })();
            match amount {
                Ok(amount) => Some(amount),
                Err(issue) => {
                    *home_issue = Some(issue);
                    None
                }
            }
        },
    )
}

// Both paths retain the native hypothetical179 → other deduction → final179
// ordering; source preparation owns the selected before179 income operands.
fn allocate_income_stage<const PREPARED: bool>(
    mut result: Deduction,
    items: &[Election179Item],
    allocatable: i64,
    income: i64,
    other_limited_deduction: impl FnOnce(&BTreeMap<i64, i64>) -> Option<i64>,
) -> Deduction {
    let hypothetical =
        allocate(phase::cap(phase::cap(allocatable, [result.limit]), [positive(income)]), items);
    let mut hypothetical_by_business = BTreeMap::<i64, i64>::new();
    for ((id, index, _, _), allowed) in items.iter().zip(hypothetical) {
        if *index != PASS_THROUGH {
            *hypothetical_by_business.entry(*id).or_default() += allowed;
        }
    }
    // Pub. 946's two-taxable-income-limit ordering: hypothetical section 179,
    // hypothetical other deduction, actual section 179, actual other deduction.
    let Some(office) = other_limited_deduction(&hypothetical_by_business) else {
        result.income_known = false;
        return result;
    };
    result.income = positive(if PREPARED {
        let Some(value) = income.checked_sub(office) else {
            result.prepared_income_overflow = true;
            return result;
        };
        value
    } else {
        income - office
    });
    result.allowed = phase::cap(phase::cap(allocatable, [result.limit]), [result.income]);
    for ((id, index, available, year), allowed) in items.iter().zip(allocate(result.allowed, items))
    {
        if *index == PASS_THROUGH {
            *result.pass_through.entry(*id).or_default() += allowed;
            result.pass_through_components.entry(*id).or_default().push(
                Ptp179ComponentAllocation {
                    component: if *year == 0 {
                        Ptp179Component::Current
                    } else {
                        // Main retains one aggregate Form4562 carry, not its
                        // original vintages.2025 is an allocator bucket only.
                        Ptp179Component::PriorIncomeCarry {
                            original_year: result
                                .pass_throughs
                                .iter()
                                .find(|source| source.id == *id)
                                .and_then(|source| source.income_carry_vintages.as_ref())
                                .map(|_| *year),
                        }
                    },
                    available_cents: *available,
                    allowed_cents: allowed,
                },
            );
        } else {
            result.assets.insert((*id, *index), allowed);
            *result.businesses.entry(*id).or_default() += allowed;
        }
    }
    result
}

pub(crate) fn operating(d: &FederalDraft, id: i64, business: &FederalBusiness) -> i64 {
    let mq = crate::depreciation::mid_quarter(d);
    let additional = calculate(d).businesses.get(&id).copied().unwrap_or(0)
        + crate::farm_conservation::expense_adjustment(d, id, business);
    match crate::farm_supplies::shared_amounts(d, id) {
        Some(amounts) => business.operating_cost_before_supplies(mq) + additional + amounts[2],
        None => business.operating_cost_with_adjustment(mq, additional),
    }
}
// The section 179 deduction allowed to one business after the dollar and
// income limits (its share of Form 4562 line 12).
pub(crate) fn allowed(d: &FederalDraft, id: i64) -> i64 {
    calculate(d).businesses.get(&id).copied().unwrap_or(0)
}
// Schedule C line 29 before the home office (Form 8829 line 8 starts here).

#[cfg(test)]
pub(crate) fn report(d: &FederalDraft, r: &mut FederalResult) {
    report_allocated(d, r, &calculate(d));
}

pub(crate) fn report_with_home(
    d: &FederalDraft,
    result: &mut FederalResult,
    home: &crate::home_office::HomeContext,
) -> Result<(), Section179PreparedIssue> {
    let allocation = calculate_with_home(d, home)?;
    report_allocated(d, result, &allocation);
    Ok(())
}

fn report_allocated(d: &FederalDraft, r: &mut FederalResult, result: &Deduction) {
    report_allocated_with_consumed(d, r, result, |_| false);
}

fn report_allocated_with_consumed(
    d: &FederalDraft,
    r: &mut FederalResult,
    result: &Deduction,
    consumed: impl Fn(i64) -> bool,
) {
    if let Some(issue) = &result.source_issue {
        crate::section179_return::Issue::Prepared(issue.clone()).report(r);
        return;
    }
    if result.available != 0 || result.pass_throughs.iter().any(|k1| k1.sold_carryover != 0) {
        // Line 10 includes old direct-asset carryovers and the K-1 balances
        // remaining after the sold-property part was restored to basis.
        let direct = d
            .income
            .iter()
            .filter_map(|entry| match &entry.income {
                FederalIncome::Business { data } => Some(data),
                _ => None,
            })
            .flat_map(|data| &data.assets)
            .map(|asset| asset.available179() - asset.elected179())
            .sum::<i64>();
        crate::kernel::lines::push_line(
            r,
            Some(crate::FormLine::new(crate::TaxForm::Form4562, "10")),
            "section179_prior_carryover_total",
            direct + result.pass_throughs.iter().map(|k1| k1.carryover).sum::<i64>(),
            &["income.business.assets", "income.partnership", "income.scorp"],
            "https://www.irs.gov/instructions/i4797",
            crate::Message::new("EXPLAIN_SECTION179_PRIOR_CARRYOVER_TOTAL", alloc::vec![]),
        );
        for k1 in &result.pass_throughs {
            if k1.amount + k1.carryover + k1.sold_carryover == 0 {
                continue;
            }
            line_cents(
                r,
                &format!(
                    "{}_{}_section179_prior_carryover",
                    if k1.partnership { "partnership" } else { "scorp" },
                    k1.id
                ),
                k1.carryover,
                &[if k1.partnership {
                    "income.partnership.section179"
                } else {
                    "income.scorp.section179"
                }],
                "https://www.irs.gov/instructions/i4562",
                msg!("EXPLAIN_N_N_N_SECTION179_PRIOR_CARRYOVER"),
            );
        }
    }
    if result.available == 0 {
        return;
    }
    // Records answered No take a zero share of the dollar limit: 2026
    // elections become ordinary depreciation (unknowns::supported) and
    // carryforwards wait for 2027. A K-1 amount above that limit is lost while
    // still reducing basis, which this App does not calculate.
    let unconfirmed = matches!(
        spouse(d),
        Some(Section179Spouse::NotSupplied | Section179Spouse::Records { eligible: false, .. })
    );
    let k1_current = result.pass_throughs.iter().any(|source| source.amount > 0)
        || result.pass_through_components.values().flatten().any(|component| {
            component.component == Ptp179Component::Current && component.available_cents > 0
        });
    crate::require_localized!(
        r,
        spouse(d).is_none_or(|s| s.facts().2) || (unconfirmed && !k1_current),
        &crate::IssueContext::Business,
        "Section 179",
        "ISSUE_MARRIED_SEPARATE_SECTION_179_NEEDS_THE_SPOUSE_S_ELIGIBLE_PURCHASES_C6523EBC",
        "A Schedule K-1 section 179 amount on a married-separate return needs the spouse's eligible purchases and agreed dollar-limit allocation; without them the amount is lost while still reducing basis, which is not calculated.",
    );
    crate::require_localized!(
        r,
        result.elected <= result.limit || unconfirmed,
        &crate::IssueContext::Business,
        "Section 179",
        "ISSUE_THE_ASSET_ELECTIONS_AND_SCHEDULE_K_1_SECTION_179_AMOUNTS_EXCEED_THE_CE43F95A",
        "The asset elections and Schedule K-1 section 179 amounts exceed the 2026 shared dollar limit after the eligible-purchase phaseout (Form 4562 line 5). Reduce your own elected costs; the excess cannot become an income-limit carryforward, and a K-1 share above the limit is lost while still reducing basis (Regulations sections 1.179-2(b)(3) and 1.179-3(a)), which this App does not calculate.",
    );
    let income_limited = result.allowed < phase::cap(result.available, [result.limit]);
    // Reg. 1.179-2(c)(2)(iv) keeps section 707(c) payments out of the
    // partnership's taxable income; whether the partner's own guaranteed
    // payments count as compensation is decided only when it matters.
    crate::require_localized!(
        r,
        result.guarantees == 0 || !income_limited,
        &crate::IssueContext::Business,
        "Section 179",
        "ISSUE_GUARANTEED_PAYMENTS_SCHEDULE_K_1_1065_BOXES_4A_AND_4B_WHERE_THE_FOR_3C50E6B6",
        "Guaranteed payments (Schedule K-1 (1065) boxes 4a and 4b) where the Form 4562 line 11 business income limit binds need an established treatment as active-business compensation, which this App does not calculate.",
    );
    for k1 in &result.pass_throughs {
        let (form, _section, input) = if k1.partnership {
            (
                "Schedule K-1 (1065) box 12",
                "Partnership section 179",
                "income.partnership.section179",
            )
        } else {
            ("Schedule K-1 (1120-S) box 11", "S-corporation section 179", "income.scorp.section179")
        };
        #[cfg(test)]
        let section = _section;

        crate::federal_rules::require_message(
            r,
            !k1.owner_deductions || (k1.amount + k1.carryover == 0 && !income_limited),
            &crate::IssueContext::income_id(k1.id).manual(),
            #[cfg(test)]
            section,
            #[cfg(test)]
            format!(
                "{form} section 179 or an income-limited section 179 combined with partner health-insurance or retirement deductions still needs verified income-limit coordination."
            ),
            crate::Message::new(
                "ISSUE_SECTION179_OWNER_DEDUCTIONS",
                alloc::vec![crate::MessageArg::Text(form.into())],
            ),
        );
        if k1.amount + k1.carryover == 0 {
            continue;
        }
        let allowed = result.pass_through.get(&k1.id).copied().unwrap_or(0);
        crate::federal_rules::require_message(
            r,
            k1.material || consumed(k1.id),
            &crate::IssueContext::Section(crate::TaxSection::PassThrough),
            #[cfg(test)]
            section,
            #[cfg(test)]
            format!(
                "{form} section 179 from an activity without established material participation is a passive activity deduction allocated on Form 8582 from Form 4562 line 12, which this App does not calculate."
            ),
            crate::Message::new(
                "ISSUE_SECTION179_PASSIVE_ACTIVITY",
                alloc::vec![crate::MessageArg::Text(form.into())],
            ),
        );
        crate::federal_rules::require_message(
            r,
            !k1.publicly_traded || consumed(k1.id),
            &crate::IssueContext::Section(crate::TaxSection::PassThrough),
            #[cfg(test)]
            section,
            #[cfg(test)]
            format!(
                "{form} section 179 of a publicly traded partnership follows the PTP rules for Form 4562 line 12, which this App does not calculate."
            ),
            crate::Message::new(
                "ISSUE_SECTION179_PUBLICLY_TRADED",
                alloc::vec![crate::MessageArg::Text(form.into())],
            ),
        );
        // Form 6198: the deduction allowed this year shares the activity's
        // remaining amount at risk. Form 4562 line 11 counts the box 1 amount
        // after the at-risk limit, while a pro rata Form 6198 would need the
        // line 12 share, so the two depend on each other and the instructions
        // give no order: above the remaining amount the K-1 is refused, and
        // the at-risk stage never produces a section 179 carry. The 2027
        // carryover is not a current deduction.
        crate::federal_rules::require_message(
            r,
            allowed == 0 || k1.risk_end.is_some(),
            &crate::IssueContext::Section(crate::TaxSection::PassThrough),
            #[cfg(test)]
            section,
            #[cfg(test)]
            format!(
                "{form} section 179 is tested against your remaining amount at risk (Form 6198), which needs your 2025 Form 6198 figures (Your at-risk history on the K-1)."
            ),
            crate::Message::new(
                "ISSUE_SECTION179_RISK_HISTORY",
                alloc::vec![crate::MessageArg::Text(form.into())],
            ),
        );
        crate::federal_rules::require_message(
            r,
            k1.risk_end.is_none_or(|end| allowed <= end),
            &crate::IssueContext::Section(crate::TaxSection::PassThrough),
            #[cfg(test)]
            section,
            #[cfg(test)]
            format!(
                "{form} section 179 above the activity's remaining amount at risk (Form 6198 line 21) depends on the Form 4562 business income limit, which itself depends on the at-risk limit; this App does not calculate that case."
            ),
            crate::Message::new(
                "ISSUE_SECTION179_RISK_CARRYFORWARD",
                alloc::vec![crate::MessageArg::Text(form.into())],
            ),
        );
        emit_lines(
            r,
            &mut (),
            [
                ("section179", allowed, msg!("EXPLAIN_N_N_N_SECTION179")),
                (
                    "section179_carryforward",
                    crate::kernel::carry::native_balance(k1.amount + k1.carryover, allowed),
                    msg!("EXPLAIN_N_N_N_SECTION179_CARRYFORWARD"),
                ),
            ],
            |(name, amount, note), _, _| {
                Some(calculation_row(
                    None,
                    &format!(
                        "{}_{}_{name}",
                        if k1.partnership { "partnership" } else { "scorp" },
                        k1.id
                    ),
                    LineValue::MoneyCents(amount),
                    &[input],
                    "https://www.irs.gov/instructions/i4562",
                    note,
                ))
            },
        );
    }
    if result.allowed < result.available {
        crate::require_localized!(
            r,
            d.credits.as_ref().is_none_or(|c| {
                !crate::adoption::has_benefits(&c.adoptions) && !c.care.has_benefits(d)
            }),
            &crate::IssueContext::Business,
            "Section 179",
            "ISSUE_INCOME_LIMITED_SECTION_179_WITH_ADOPTION_OR_CARE_BENEFITS_NEEDS_THE_58B6F42A",
            "Income-limited section 179 with adoption or care benefits needs the coordinated employee-compensation worksheet.",
        );
    }
    for e in &d.income {
        match &e.income {
            FederalIncome::Business { data } => {
                crate::require_localized!(
                    r,
                    data.material_participation
                        && matches!(
                            data.activity,
                            crate::BusinessActivity::Ordinary { .. }
                                | crate::BusinessActivity::Farming { .. }
                        ),
                    &crate::IssueContext::Business,
                    "Section 179",
                    "ISSUE_SECTION_179_INCOME_COORDINATION_CURRENTLY_REQUIRES_ORDINARY_ACTIVE_F5FE33AB",
                    "Section 179 income coordination currently requires ordinary active businesses; passive activity and professional wagering need separate treatment.",
                );
                crate::require_localized!(
                    r,
                    matches!(data.health_insurance, crate::BusinessHealthInsurance::None)
                        && matches!(data.retirement, crate::BusinessRetirement::None),
                    &crate::interview::income_issue_context(e).manual(),
                    "Section 179",
                    "ISSUE_SECTION_179_COMBINED_WITH_OWNER_HEALTH_INSURANCE_OR_RETIREMENT_DEDU_0C0CC3A2",
                    "Section 179 combined with owner health-insurance or retirement deductions still needs verified income-limit coordination.",
                );
                for (index, asset) in data.assets.iter().enumerate() {
                    let available = asset.available179();
                    if available == 0 {
                        continue;
                    }
                    let allowed = result.assets.get(&(e.id, index)).copied().unwrap_or(0);
                    if let Some(cap) = asset.automobile_limit() {
                        // Section 280F counts section 179 inside the passenger
                        // automobile's first-year limit on its business-use share.
                        crate::require_localized!(
                            r,
                            asset.elected179() <= cap
                                && (asset.documented_179_reserve() || allowed == available),
                            &crate::IssueContext::Business,
                            "Section 179",
                            "ISSUE_A_PASSENGER_AUTOMOBILE_S_SECTION_179_ELECTION_CANNOT_EXCEED_ITS_FIR_581FCDA1",
                            "A passenger automobile's section 179 election cannot exceed its first-year depreciation limit on the business-use share, and an income-limited section 179 carryforward on a passenger automobile is not calculated (Form 4562 line 26(i)).",
                        );
                    }
                    emit_lines(
                        r,
                        &mut (),
                        [
                            (
                                "section179",
                                allowed,
                                msg!("EXPLAIN_BUSINESS_N_ASSET_N_N_SECTION179"),
                            ),
                            (
                                "section179_carryforward",
                                available - allowed,
                                msg!("EXPLAIN_BUSINESS_N_ASSET_N_N_SECTION179_CARRYFORWARD"),
                            ),
                        ],
                        |(name, amount, note), _, _| {
                            Some(calculation_row(
                                None,
                                &format!("business_{}_asset_{}_{}", e.id, index, name),
                                LineValue::MoneyCents(amount),
                                &["income.business.assets"],
                                "https://www.irs.gov/instructions/i4562",
                                note,
                            ))
                        },
                    );
                }
            }
            FederalIncome::Rental { .. } | FederalIncome::ForeignEmployment { .. } => {
                crate::require_localized!(
                    r,
                    false,
                    &crate::IssueContext::Business,
                    "Section 179",
                    "ISSUE_RENTAL_FOREIGN_OR_OTHER_PROPERTY_INCOME_WITH_SECTION_179_NEEDS_ACTI_F72A67AD",
                    "Rental, foreign or other property income with section 179 needs active-business income classification and coordination.",
                )
            }
            _ => {}
        }
    }
    emit_lines(
        r,
        &mut (),
        [
            ("section179_eligible_cost", result.cost, msg!("EXPLAIN_SECTION179_ELIGIBLE_COST")),
            (
                "section179_spouse_eligible_cost",
                result.spouse_cost,
                msg!("EXPLAIN_SECTION179_SPOUSE_ELIGIBLE_COST"),
            ),
            (
                "section179_shared_dollar_limit",
                result.shared_limit,
                msg!("EXPLAIN_SECTION179_SHARED_DOLLAR_LIMIT"),
            ),
            ("section179_dollar_limit", result.limit, msg!("EXPLAIN_SECTION179_DOLLAR_LIMIT")),
            (
                "section179_business_income_limit",
                result.income,
                msg!("EXPLAIN_SECTION179_BUSINESS_INCOME_LIMIT"),
            ),
            ("section179_deduction", result.allowed, msg!("EXPLAIN_SECTION179_DEDUCTION")),
            (
                "section179_carryforward",
                result.available - result.allowed,
                msg!("EXPLAIN_SECTION179_CARRYFORWARD"),
            ),
        ],
        |(name, amount, note), _, _| {
            Some(calculation_row(
                None,
                name,
                LineValue::MoneyCents(amount),
                &["income.business.assets"],
                "https://www.irs.gov/publications/p946",
                note,
            ))
        },
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct K1Section179BasisSource {
    pub k1: i64,
    pub alternative: bool,
    pub partnership: bool,
    pub elected_cents: i64,
    pub basis_allowed_cents: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum K1Section179BasisIssue {
    MissingReturnContext,
    UnknownSource,
    NotK1,
    AlternativeHistoryUnavailable,
    AlternativeIncomeCarryUnavailable,
    InvalidSource(&'static str),
    SourceLedgerUnavailable(&'static str),
    /// The native annual-Dollar result cannot be represented as Cents.
    AmountOverflow,
}

// Pair current basis allowance and remaining risk from one actual native
// selected-system invocation. Section179 is not a class of that risk stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct K1Section179CurrentSource {
    pub basis: K1Section179BasisSource,
    pub risk: K1Section179RiskInput,
}
// Actual current selected-system basis and before179 risk from one native
// owning invocation. AMT income-limit carry is a separate source fact.
pub fn k1_section179_current_source(
    draft: &FederalDraft,
    k1: i64,
    alternative: bool,
) -> Result<K1Section179CurrentSource, K1Section179BasisIssue> {
    use K1Section179BasisIssue as E;
    if draft.setup.is_none() {
        return Err(E::MissingReturnContext);
    }
    let original =
        crate::source_associations::income_source(draft, k1).map_err(|issue| match issue {
            crate::source_associations::SourceAssociationIssue::AmbiguousSource(_) => {
                E::InvalidSource("K1_SOURCE_ID_TWICE")
            }
            _ => E::UnknownSource,
        })?;
    if !matches!(original.income, FederalIncome::Partnership { .. } | FederalIncome::Scorp { .. }) {
        return Err(E::NotK1);
    }
    crate::federal_rules::validate_income(original).map_err(|(key, _)| E::InvalidSource(key))?;
    let entry = crate::community::entry(draft, original);
    let (partnership, elected, ledger, remaining) = match &entry.income {
        FederalIncome::Partnership { data } => {
            let record = if alternative {
                data.alternative.as_ref().ok_or(E::AlternativeHistoryUnavailable)?
            } else {
                &data.regular
            };
            let risk = crate::community::partner_risk(draft, k1, data, record)
                .map_err(|(key, _)| E::SourceLedgerUnavailable(key))?;
            (true, data.section179, risk.ledger, risk.risk_end)
        }
        FederalIncome::Scorp { data } => {
            let source = if alternative {
                let record = data
                    .amt
                    .as_ref()
                    .filter(|record| record.complete)
                    .ok_or(E::AlternativeHistoryUnavailable)?;
                record.source(data).map_err(|(key, _)| E::SourceLedgerUnavailable(key))?
            } else {
                data.into()
            };
            let risk = crate::community::scorp_risk(draft, k1, &source, alternative)
                .map_err(|(key, _)| E::SourceLedgerUnavailable(key))?;
            (false, source.section179, risk.ledger, risk.year_end)
        }
        _ => unreachable!(),
    };
    Ok(K1Section179CurrentSource {
        basis: K1Section179BasisSource {
            k1,
            alternative,
            partnership,
            elected_cents: elected,
            basis_allowed_cents: ledger.basis_allowed[K1LossClass::Section179].get(),
        },
        risk: K1Section179RiskInput {
            k1,
            alternative,
            // The existing activity risk owner now retains actual source cents.
            remaining_cents: remaining,
        },
    })
}

pub(crate) use direct_sources::direct_income_sources_with_home;
