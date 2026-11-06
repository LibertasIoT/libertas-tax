// Coverage: ordinary post-August1993 section197 sources, proved post-transition
// origins and statutory self-creation exceptions. Historical anti-churning,
// selected operating assets, transfer/decrease/lease and disposition groups
// remain explicit source boundaries. No confirmation substitutes for facts.
// Source: https://www.ecfr.gov/current/title-26/section-1.197-2
use crate::kernel::cost_recovery::{AmortizationRegime, Month, MonthlySchedule};
use crate::kernel::source_fact::SourceFact;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Later capitalized acquisition cost
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IntangibleCost {
    /// Additional allocated business acquisition cost
    /// Use purchase/allocation records. Exclude interest, original issue discount, amounts already included in original cost, deductible royalties, renewal costs and transfer or disposition adjustments. A renewal is a separate intangible. Include only amounts properly added to basis under your accounting method.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Date this cost became includible in basis
    /// Use the date supported by the acquisition obligation and accounting records, not automatically the cash-payment date. Unresolved contingent-debt interest allocation or timing needs separate review.
    #[libertas_date_only]
    pub date: u32,
}

/// Section 197 business intangible
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessIntangible {
    /// Asset or right acquired
    /// Examples include acquired goodwill, going-concern value, workforce, business records, customer/supplier relationships, a qualifying government license or a franchise. Eligibility depends on the acquisition and statutory exclusions, not the label alone.
    #[libertas_size(min = 1)]
    pub label: String,
    /// Original capitalized business cost allocated to this intangible
    /// Use the documented purchase-price allocation and original capitalized acquisition costs. Enter zero only when all cost became includible later and is entered below. Exclude personal basis. Do not deduct this cost again as an ordinary expense, start-up cost or equipment depreciation. Enter each eligible renewal separately.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cost: i64,
    /// Acquisition, qualifying creation or renewal date
    #[libertas_date_only]
    pub acquired: u32,
    /// Later capitalized acquisition costs
    /// Leave empty if none. Include all later costs through 2026 exactly once, including earlier-year additions. Each addition uses the original schedule's remaining months; an addition after its end is deductible immediately. These additions do not start another 15-year period.
    pub additions: Vec<IntangibleCost>,
    /// Date this trade or business began
    /// Amortization starts in the later of the acquisition month and the month this business began. An existing business may begin before it acquires this asset.
    #[libertas_date_only]
    pub business_started: u32,
    /// Canonical intangible identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Acquisition agreement and property facts
    pub source: IntangibleSource,
    /// Earlier deductions actually claimed
    pub prior_claimed: IntangiblePriorClaimed,
    /// Recorded basis, use or transfer events
    /// [NullText]
    /// Basis/use/transfer records unavailable
    /// [SetText]
    /// Recorded event kinds
    pub other_events: Option<Vec<IntangibleEventKind>>,
    /// Continued business use or its actual end
    pub use_end: IntangibleUseEnd,
}
impl BusinessIntangible {
    pub(crate) fn valid(&self) -> bool {
        !self.label.trim().is_empty()
            && self.cost >= 0
            && (self.cost > 0 || !self.additions.is_empty())
            && self.additions.iter().all(|a| {
                a.amount > 0 && crate::federal_rules::valid_birth(a.date) && a.date >= self.acquired
            })
            && crate::federal_rules::valid_birth(self.acquired)
            && crate::federal_rules::valid_birth(self.business_started)
            && self.source.valid(self.acquired)
            && self.prior_claimed.regular.source_fact().is_ok()
            && self.prior_claimed.amt.source_fact().is_ok()
            && match &self.use_end {
                IntangibleUseEnd::Ended { record } => {
                    crate::federal_rules::valid_date(record.date)
                        && record.date >= self.acquired
                        && record.date <= 20261231
                        && record.how.valid()
                }
                _ => true,
            }
            && crate::kernel::ratio::checked_sum(
                core::iter::once(self.cost).chain(self.additions.iter().map(|cost| cost.amount)),
            )
            .is_some()
            && self.schedule().is_some()
    }
    pub(crate) fn started(&self) -> u32 {
        self.acquired.max(self.business_started)
    }
    fn schedule(&self) -> Option<[i64; 3]> {
        let started = self.started();
        let month = Month { year: started / 10000, month: started / 100 % 100 };
        let original = MonthlySchedule::original_180(
            AmortizationRegime::Section197,
            month,
            2026,
            self.cost,
            0,
        );
        self.additions.iter().try_fold(
            [
                original.current_allowable_cents,
                original.prior_allowable_cents,
                original.statutory_remaining_cents,
            ],
            |total, cost| {
                let date = cost.date.max(started);
                let part = MonthlySchedule::section197_addition_180(
                    month,
                    Month { year: date / 10000, month: date / 100 % 100 },
                    2026,
                    cost.amount,
                );
                Some([
                    total[0].checked_add(part.current_allowable_cents)?,
                    total[1].checked_add(part.prior_allowable_cents)?,
                    total[2].checked_add(part.statutory_remaining_cents)?,
                ])
            },
        )
    }
    /// Actual proceeds or transfers can require gain/recapture even when the
    /// amortization benefit is omitted. Unknown records alone establish no event.
    /// Section 1245(a)-(b): realization can require recapture; a gift alone
    /// does not. https://www.irs.gov/publications/p544
    pub(crate) fn mandatory_tax_boundary(&self) -> Option<&'static str> {
        if let IntangibleUseEnd::Ended { record } = &self.use_end {
            let received = match &record.how {
                crate::DisposalKind::Sold { proceeds, selling_expenses, .. } => {
                    proceeds > selling_expenses
                }
                crate::DisposalKind::Condemned { award, expenses, .. } => award > expenses,
                crate::DisposalKind::Scrapped | crate::DisposalKind::PersonalUse => false,
            };
            if received {
                return Some("INTANGIBLE_DISPOSITION_GROUP");
            }
        }
        if self.other_events.as_ref().is_some_and(|events| {
            events.iter().any(|event| {
                matches!(
                    event,
                    IntangibleEventKind::EntityContribution
                        | IntangibleEventKind::EntityDistribution
                        | IntangibleEventKind::Exchange
                        | IntangibleEventKind::OtherTransfer
                )
            })
        }) {
            return Some("INTANGIBLE_RECORDED_EVENTS");
        }
        if matches!(
            self.source.obtained,
            IntangibleObtained::EntityContribution
                | IntangibleObtained::EntityDistribution
                | IntangibleObtained::Exchange
        ) {
            return Some("INTANGIBLE_TRANSFER_SOURCE");
        }
        None
    }
    pub(crate) fn boundary(&self) -> Option<&'static str> {
        if let Some(key) = self.mandatory_tax_boundary() {
            return Some(key);
        }
        if let Some(key) = self.source.limitation(self.acquired) {
            return Some(key);
        }
        if self.started() > 20261231 || self.additions.iter().any(|cost| cost.date > 20261231) {
            return Some("INTANGIBLE_SOURCE_DATES");
        }
        match self
            .other_events
            .as_ref()
            .map_or(SourceFact::Unavailable, SourceFact::Reported)
            .require(|| "INTANGIBLE_EVENTS_UNAVAILABLE")
        {
            Ok(Some(events)) if !events.is_empty() => return Some("INTANGIBLE_RECORDED_EVENTS"),
            Err(key) => return Some(key),
            _ => {}
        }
        match &self.use_end {
            IntangibleUseEnd::StillHeldAndUsed => {}
            // The original cumulative clock must exclude the disposal month.
            // Disposition recognition also needs retained-intangible grouping.
            IntangibleUseEnd::Ended { .. } => return Some("INTANGIBLE_DISPOSITION_GROUP"),
            IntangibleUseEnd::RecordsUnavailable => return Some("INTANGIBLE_USE_UNAVAILABLE"),
        }
        if self.started() / 10000 < 2026 {
            let expected = self.schedule()?[1];
            for amount in [&self.prior_claimed.regular, &self.prior_claimed.amt] {
                match amount.source_fact().ok()?.require(|| "INTANGIBLE_PRIOR_UNAVAILABLE") {
                    Ok(value) if value.unwrap_or(0) == expected => {}
                    Ok(_) => return Some("INTANGIBLE_PRIOR_CORRECTION"),
                    Err(key) => return Some(key),
                }
            }
        }
        None
    }
    pub(crate) fn deduction(&self) -> i64 {
        if self.boundary().is_some() {
            return 0;
        }
        self.schedule().expect("validated intangible schedule")[0]
    }
    pub(crate) fn remaining(&self) -> i64 {
        self.schedule().expect("validated intangible schedule")[2]
    }
}

/// Acquisition agreement and property facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IntangibleSource {
    /// What asset or right?
    pub kind: IntangibleKind,
    /// How acquired?
    pub obtained: IntangibleObtained,
    /// What was acquired in the transaction?
    pub context: IntangibleAcquisitionContext,
    /// When did this intangible first exist?
    /// Identify the underlying intangible, not merely a new renewal agreement.
    /// [NullText]
    /// Origin records unavailable
    /// [SetText]
    /// Origin evidence
    pub first_existed: Option<IntangibleOriginEvidence>,
    /// Payment terms
    pub consideration: IntangibleConsideration,
    /// Lease facts for this actual right
    pub lease_status: IntangibleLeaseStatus,
}

/// What asset or right?
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum IntangibleKind {
    /// Goodwill
    Goodwill,
    /// Going-concern value
    GoingConcern,
    /// Workforce in place
    Workforce,
    /// Business information base
    InformationBase,
    /// Customer relationship
    CustomerRelationship,
    /// Supplier relationship
    SupplierRelationship,
    /// Government license, permit or right
    GovernmentRight,
    /// Franchise
    Franchise,
    /// Trademark
    Trademark,
    /// Trade name
    TradeName,
    /// Covenant not to compete
    Noncompete,
    /// Patent, copyright or know-how
    PatentCopyrightKnowhow,
    /// Computer software
    Software,
    /// Film, book or recording
    FilmBookRecording,
    /// Mortgage servicing right
    MortgageServicing,
    /// Financial or entity interest
    FinancialInterest,
    /// Land right
    LandRight,
    /// Existing tangible-property lease
    ExistingTangibleLease,
    /// Right of fixed duration or amount
    FixedTermRight,
    /// Another asset or right
    Other { description: String },
}

/// How acquired?
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum IntangibleObtained {
    Purchased,
    Created,
    ContractForExistingRight,
    /// New renewal agreement, entered as its own intangible
    Renewal {
        /// Recorded predecessor
        /// [NullText]
        /// Predecessor not recorded here
        /// [SetText]
        /// Recorded predecessor
        previous_intangible_id: Option<i64>,
    },
    Gift,
    Inheritance,
    EntityContribution,
    EntityDistribution,
    Exchange,
}

/// What was acquired in the transaction?
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum IntangibleAcquisitionContext {
    /// A separate right
    SeparateRight,
    /// All assets of an operating business
    EntireOperatingBusinessAssets,
    /// Selected assets of an operating business
    SelectedOperatingAssets {
        /// Assets acquired
        acquired: String,
        /// Assets retained by the seller
        retained_by_seller: String,
    },
    /// An ownership interest in the business
    BusinessOwnershipInterest,
}

/// Records establishing when the underlying intangible first existed
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum IntangibleOriginEvidence {
    CreatedOn {
        #[libertas_date_only]
        date: u32,
    },
    CreatedAfter {
        #[libertas_date_only]
        date: u32,
    },
    ExistedBy {
        #[libertas_date_only]
        date: u32,
    },
}

/// Payment terms
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum IntangibleConsideration {
    FixedCash,
    FixedDebt,
    ContingentDebt,
    OtherContingent,
    RecurringUsePayments,
    NoPurchasePayment,
}

/// Actual lease facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum IntangibleLeaseStatus {
    /// This intangible was not leased
    NotLeased,
    /// This intangible was leased
    Leased,
    /// Lease records unavailable
    RecordsUnavailable,
}

/// Actual continued use or its end
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum IntangibleUseEnd {
    /// Still held and used in this business
    StillHeldAndUsed,
    /// Disposed of or stopped business use
    Ended { record: crate::AmortizationEnd },
    /// Disposition/use records unavailable
    RecordsUnavailable,
}

/// Actual cumulative deductions before this return year, including additions
/// Use earlier returns; do not enter a newly calculated statutory schedule.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IntangiblePriorClaimed {
    /// Regular-tax deductions actually claimed
    pub regular: crate::SourceCents,
    /// AMT deductions actually claimed
    pub amt: crate::SourceCents,
}

/// Recorded basis, use or transfer events
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum IntangibleEventKind {
    PurchasePriceReduction,
    OtherBasisDecrease,
    IncomeUseChanged,
    Gift,
    EntityContribution,
    EntityDistribution,
    Exchange,
    OtherTransfer,
}

impl IntangibleSource {
    pub(super) fn valid(&self, acquired: u32) -> bool {
        !matches!(&self.kind, IntangibleKind::Other { description } if description.trim().is_empty())
            && !matches!(&self.context, IntangibleAcquisitionContext::SelectedOperatingAssets { acquired, retained_by_seller }
                if acquired.trim().is_empty() || retained_by_seller.trim().is_empty())
            && !matches!(&self.obtained, IntangibleObtained::Renewal { previous_intangible_id: Some(id) } if *id <= 0)
            && self.first_existed.as_ref().is_none_or(|origin| {
                let date = match origin {
                    IntangibleOriginEvidence::CreatedOn { date }
                    | IntangibleOriginEvidence::CreatedAfter { date }
                    | IntangibleOriginEvidence::ExistedBy { date } => *date,
                };
                crate::federal_rules::valid_date(date)
                    && date <= acquired
                    && (!matches!(origin, IntangibleOriginEvidence::CreatedAfter { .. })
                        || date < acquired)
            })
    }
    pub(crate) fn needs_origin(&self) -> bool {
        // New statutory rights created by the taxpayer have no transition-period
        // holder. A renewal is not evidence that the underlying right is new.
        Self::origin_needed(&self.kind, &self.obtained, &self.context)
    }
    fn origin_needed(
        kind: &IntangibleKind,
        obtained: &IntangibleObtained,
        context: &IntangibleAcquisitionContext,
    ) -> bool {
        !matches!(obtained, IntangibleObtained::Created)
            && Self::classification(kind, obtained, context).is_none()
    }
    fn origin(&self) -> SourceFact<&IntangibleOriginEvidence> {
        self.first_existed.as_ref().map_or(SourceFact::Unavailable, SourceFact::Reported)
    }
    fn classification(
        kind: &IntangibleKind,
        obtained: &IntangibleObtained,
        context: &IntangibleAcquisitionContext,
    ) -> Option<&'static str> {
        use IntangibleAcquisitionContext as Context;
        use IntangibleKind as Kind;
        use IntangibleObtained as Obtained;
        // Reg1.197-2(c) preserves the actual non-section197 owners.
        let alternative = match kind {
            Kind::Software => Some("INTANGIBLE_SOFTWARE_SOURCE"),
            Kind::MortgageServicing => Some("INTANGIBLE_MORTGAGE_SOURCE"),
            Kind::FixedTermRight if !matches!(context, Context::SeparateRight) => {
                Some("INTANGIBLE_FIXED_RIGHT_SOURCE")
            }
            Kind::FilmBookRecording if matches!(context, Context::SeparateRight) => {
                Some("INTANGIBLE_CREATIVE_SOURCE")
            }
            Kind::FinancialInterest | Kind::LandRight | Kind::ExistingTangibleLease => {
                Some("INTANGIBLE_OTHER_OWNER")
            }
            Kind::Other { .. } => Some("INTANGIBLE_CLASSIFICATION"),
            Kind::GovernmentRight | Kind::PatentCopyrightKnowhow | Kind::FixedTermRight
                if matches!(context, Context::SeparateRight) =>
            {
                Some("INTANGIBLE_FINITE_SOURCE")
            }
            Kind::Noncompete
                if !matches!(
                    context,
                    Context::EntireOperatingBusinessAssets | Context::BusinessOwnershipInterest
                ) =>
            {
                Some("INTANGIBLE_NONCOMPETE_TRANSACTION")
            }
            _ => None,
        };
        if alternative.is_some() {
            return alternative;
        }
        if matches!(
            obtained,
            Obtained::Gift
                | Obtained::Inheritance
                | Obtained::EntityContribution
                | Obtained::EntityDistribution
                | Obtained::Exchange
        ) {
            return Some("INTANGIBLE_TRANSFER_SOURCE");
        }
        if matches!(context, Context::SelectedOperatingAssets { .. }) {
            return Some("INTANGIBLE_SELECTED_ASSETS");
        }
        if matches!(context, Context::BusinessOwnershipInterest)
            && !matches!(kind, Kind::Noncompete)
        {
            return Some("INTANGIBLE_OWNERSHIP_INTEREST");
        }
        // Reg1.197-2(d)(2)(iii): governmental rights, covenants, franchises,
        // trademarks/trade names and creation in a business acquisition.
        if matches!(obtained, Obtained::Created)
            && !matches!(
                kind,
                Kind::GovernmentRight
                    | Kind::Noncompete
                    | Kind::Franchise
                    | Kind::Trademark
                    | Kind::TradeName
            )
            && !matches!(context, Context::EntireOperatingBusinessAssets)
        {
            return Some("INTANGIBLE_SELF_CREATED");
        }
        None
    }
    pub(super) fn limitation(&self, acquired: u32) -> Option<&'static str> {
        use IntangibleObtained as Obtained;
        if let Some(key) = Self::classification(&self.kind, &self.obtained, &self.context) {
            return Some(key);
        }
        if acquired <= 19930810 {
            return Some("INTANGIBLE_HISTORIC_SOURCE");
        }
        if self.needs_origin() {
            let proved_new = match self.origin().require(|| ()).ok().flatten() {
                Some(IntangibleOriginEvidence::CreatedOn { date }) => *date > 19930810,
                Some(IntangibleOriginEvidence::CreatedAfter { date }) => *date >= 19930810,
                _ => false,
            };
            if !proved_new {
                return Some("INTANGIBLE_HISTORIC_SOURCE");
            }
        }
        if !(matches!(
            self.consideration,
            IntangibleConsideration::FixedCash | IntangibleConsideration::FixedDebt
        ) || (matches!(self.obtained, Obtained::Created)
            && matches!(self.consideration, IntangibleConsideration::NoPurchasePayment)))
        {
            return Some("INTANGIBLE_PAYMENT_SOURCE");
        }
        match self.lease_status {
            IntangibleLeaseStatus::NotLeased => None,
            IntangibleLeaseStatus::Leased => Some("INTANGIBLE_LEASE_SOURCE"),
            IntangibleLeaseStatus::RecordsUnavailable => Some("INTANGIBLE_LEASE_UNAVAILABLE"),
        }
    }
}

/// Acquisition agreement and property facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IntangibleSourcePage {
    /// What asset or right?
    #[libertas_calculation_request("IntangibleSourceOptions")]
    pub kind: IntangibleKind,
    /// How acquired?
    #[libertas_calculation_request("IntangibleSourceOptions")]
    pub obtained: IntangibleObtainedPage,
    /// What was acquired in the transaction?
    #[libertas_calculation_request("IntangibleSourceOptions")]
    pub context: IntangibleAcquisitionContext,
    /// When did this intangible first exist?
    /// Identify the underlying intangible, not merely a new renewal agreement.
    /// [NullText]
    /// Origin records unavailable
    /// [SetText]
    /// Origin evidence
    #[libertas_constrained_by("^<BusinessIntangiblePage>.origin_choices")]
    pub first_existed: Option<IntangibleOriginEvidence>,
    /// Payment terms
    pub consideration: IntangibleConsideration,
    /// Lease facts for this actual right
    pub lease_status: IntangibleLeaseStatus,
}

impl From<IntangibleSource> for IntangibleSourcePage {
    fn from(source: IntangibleSource) -> Self {
        Self {
            kind: source.kind,
            obtained: source.obtained.into(),
            context: source.context,
            first_existed: source.first_existed,
            consideration: source.consideration,
            lease_status: source.lease_status,
        }
    }
}
impl From<IntangibleSourcePage> for IntangibleSource {
    fn from(source: IntangibleSourcePage) -> Self {
        Self {
            kind: source.kind,
            obtained: source.obtained.into(),
            context: source.context,
            first_existed: source.first_existed,
            consideration: source.consideration,
            lease_status: source.lease_status,
        }
    }
}

/// How acquired?
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum IntangibleObtainedPage {
    Purchased,
    Created,
    ContractForExistingRight,
    /// New renewal agreement, entered as its own intangible
    Renewal {
        /// Recorded predecessor
        /// [NullText]
        /// Predecessor not recorded here
        /// [SetText]
        /// Recorded predecessor
        #[libertas_enum_source("^<BusinessDetails>.intangible_predecessor_choices")]
        previous_intangible_id: Option<i64>,
    },
    Gift,
    Inheritance,
    EntityContribution,
    EntityDistribution,
    Exchange,
}

impl From<IntangibleObtained> for IntangibleObtainedPage {
    fn from(value: IntangibleObtained) -> Self {
        match value {
            IntangibleObtained::Purchased => Self::Purchased,
            IntangibleObtained::Created => Self::Created,
            IntangibleObtained::ContractForExistingRight => Self::ContractForExistingRight,
            IntangibleObtained::Renewal { previous_intangible_id } => {
                Self::Renewal { previous_intangible_id }
            }
            IntangibleObtained::Gift => Self::Gift,
            IntangibleObtained::Inheritance => Self::Inheritance,
            IntangibleObtained::EntityContribution => Self::EntityContribution,
            IntangibleObtained::EntityDistribution => Self::EntityDistribution,
            IntangibleObtained::Exchange => Self::Exchange,
        }
    }
}
impl From<IntangibleObtainedPage> for IntangibleObtained {
    fn from(value: IntangibleObtainedPage) -> Self {
        match value {
            IntangibleObtainedPage::Purchased => Self::Purchased,
            IntangibleObtainedPage::Created => Self::Created,
            IntangibleObtainedPage::ContractForExistingRight => Self::ContractForExistingRight,
            IntangibleObtainedPage::Renewal { previous_intangible_id } => {
                Self::Renewal { previous_intangible_id }
            }
            IntangibleObtainedPage::Gift => Self::Gift,
            IntangibleObtainedPage::Inheritance => Self::Inheritance,
            IntangibleObtainedPage::EntityContribution => Self::EntityContribution,
            IntangibleObtainedPage::EntityDistribution => Self::EntityDistribution,
            IntangibleObtainedPage::Exchange => Self::Exchange,
        }
    }
}

/// Earlier deductions actually claimed, including additions
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IntangiblePriorClaimedPage {
    /// Regular-tax deductions actually claimed
    /// [NullText]
    /// Regular-tax deduction records unavailable
    #[libertas_constrained_by("^<BusinessIntangiblePage>.prior_choices")]
    pub regular: Option<crate::SourceCents>,
    /// AMT deductions actually claimed
    /// [NullText]
    /// AMT deduction records unavailable
    #[libertas_constrained_by("^<BusinessIntangiblePage>.prior_choices")]
    pub amt: Option<crate::SourceCents>,
}
impl IntangiblePriorClaimedPage {
    fn from_record(record: IntangiblePriorClaimed, needed: bool) -> Self {
        let project = |value| {
            if needed && !matches!(value, crate::SourceCents::Unavailable) {
                Some(value)
            } else {
                None
            }
        };
        Self { regular: project(record.regular), amt: project(record.amt) }
    }
    fn record(&self) -> IntangiblePriorClaimed {
        IntangiblePriorClaimed {
            regular: self.regular.clone().unwrap_or(crate::SourceCents::Unavailable),
            amt: self.amt.clone().unwrap_or(crate::SourceCents::Unavailable),
        }
    }
}

/// Business intangible source editor
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("IntangibleSourceOptions")]
pub struct BusinessIntangiblePage {
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Asset or right described in the agreement
    #[libertas_size(min = 1)]
    pub label: String,
    /// Original capitalized business cost, including acquisition-date corrections
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cost: i64,
    /// Acquisition, creation or renewal date
    #[libertas_date_only]
    #[libertas_calculation_request("IntangibleSourceOptions")]
    pub acquired: u32,
    /// Date this business began
    #[libertas_date_only]
    #[libertas_calculation_request("IntangibleSourceOptions")]
    pub business_started: u32,
    /// Agreement and property facts
    pub source: IntangibleSourcePage,
    /// Earlier deductions actually claimed, including additions
    pub prior_claimed: IntangiblePriorClaimedPage,
    /// Recorded basis, use or transfer events
    /// [NullText]
    /// Basis/use/transfer records unavailable
    /// [SetText]
    /// Recorded event kinds
    pub other_events: Option<Vec<IntangibleEventKind>>,
    /// Actual continued business use or end
    pub use_end: IntangibleUseEnd,
    /// Later capitalized acquisition costs, included once on their basis dates
    /// A renewal is a new intangible, never an addition. Do not duplicate original cost.
    pub additions: Vec<IntangibleCost>,
    /// Applicable origin evidence choices
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_intangible_choices")]
    pub origin_choices: Vec<i32>,
    /// Applicable earlier-deduction question
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_intangible_choices")]
    pub prior_choices: Vec<i32>,
}
impl BusinessIntangiblePage {
    pub(crate) fn choices(
        kind: &IntangibleKind,
        obtained: &IntangibleObtained,
        context: &IntangibleAcquisitionContext,
        acquired: u32,
        business_started: u32,
    ) -> (Vec<i32>, Vec<i32>) {
        (
            if IntangibleSource::origin_needed(kind, obtained, context) {
                alloc::vec![0, 1, 2]
            } else {
                Vec::new()
            },
            if acquired.max(business_started) / 10000 < 2026
                && acquired > 19930810
                && IntangibleSource::classification(kind, obtained, context).is_none()
            {
                // SourceCents discriminants: documented absence and reported amount.
                // Nullable null represents unavailable without a duplicate Union choice.
                alloc::vec![1, 2]
            } else {
                Vec::new()
            },
        )
    }
    pub(crate) fn record(&self, accepted: Option<&BusinessIntangible>) -> BusinessIntangible {
        let obtained: IntangibleObtained = self.source.obtained.clone().into();
        let (origin, prior) = Self::choices(
            &self.source.kind,
            &obtained,
            &self.source.context,
            self.acquired,
            self.business_started,
        );
        let mut source: IntangibleSource = self.source.clone().into();
        if origin.is_empty() {
            source.first_existed = accepted.and_then(|record| record.source.first_existed.clone());
        }
        let prior_claimed = if prior.is_empty() {
            accepted.map(|record| record.prior_claimed.clone())
        } else {
            Some(self.prior_claimed.record())
        }
        .unwrap_or(IntangiblePriorClaimed {
            regular: crate::SourceCents::Unavailable,
            amt: crate::SourceCents::Unavailable,
        });
        BusinessIntangible {
            id: self.id,
            label: self.label.clone(),
            cost: self.cost,
            acquired: self.acquired,
            business_started: self.business_started,
            source,
            prior_claimed,
            other_events: self.other_events.clone(),
            use_end: self.use_end.clone(),
            additions: self.additions.clone(),
        }
    }
}
impl From<BusinessIntangible> for BusinessIntangiblePage {
    fn from(record: BusinessIntangible) -> Self {
        let (origin_choices, prior_choices) = Self::choices(
            &record.source.kind,
            &record.source.obtained,
            &record.source.context,
            record.acquired,
            record.business_started,
        );
        let mut source: IntangibleSourcePage = record.source.into();
        if origin_choices.is_empty() {
            source.first_existed = None;
        }
        Self {
            id: record.id,
            label: record.label,
            cost: record.cost,
            acquired: record.acquired,
            business_started: record.business_started,
            source,
            prior_claimed: IntangiblePriorClaimedPage::from_record(
                record.prior_claimed,
                !prior_choices.is_empty(),
            ),
            other_events: record.other_events,
            use_end: record.use_end,
            additions: record.additions,
            origin_choices,
            prior_choices,
        }
    }
}
