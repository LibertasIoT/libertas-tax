// Coverage: original section169,174/174A,178,59(e) and creative annual cohorts.
// Original elections and actual prior recovery stay source facts. Shared monthly
// history owns proration; the property owner owns mandatory disposition income.
use crate::kernel::cost_recovery::{Month, monthly_history::ActualMonthlySchedule};
use crate::kernel::lines::{
    LineValue, emit as emit_lines, line_cents, push_line, row as calculation_row,
};
use crate::kernel::ratio::fraction;
use crate::{AmortizationEnd, FederalResult, FormLine, TaxForm};
use alloc::{format, string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};
/// One Code-specific cost cohort linked to the existing accepted activity
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct OtherAmortizableCost {
    /// Cost or property described on Form 4562-B
    #[libertas_size(min = 1)]
    pub description: String,
    /// Documented qualifying business cost before statutory adjustments
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub cost: i64,
    /// Date costs were paid or incurred under the accounting method
    #[libertas_date_only]
    #[libertas_number(max = $crate::year_constants::TAX_YEAR_END_DATE)]
    pub incurred: u32,
    /// Original lawful amortization start date
    /// Supply this only when the original election itself establishes the start.
    /// Statutory midpoint/year starts and current expense need no extra date.
    /// [NullText]
    /// No separate original elected start date is supplied
    /// [SetText]
    /// The original elected start date was
    #[libertas_date_only]
    #[libertas_number(max = $crate::year_constants::TAX_YEAR_END_DATE)]
    #[libertas_default($null)]
    pub began: Option<u32>,
    /// Amortization allowed or allowable before the current year
    #[libertas_default(Unavailable)]
    pub prior_allowed: crate::SourceCents,
    /// Legal Code section and required election facts
    pub kind: OtherAmortizationKind,
    /// Disposition, retirement or termination of this cost cohort
    /// Enter historical events too. A current creative-property disposition can create ordinary income while its original amortization continues. Do not also include these proceeds in business receipts.
    /// [NullText]
    /// This cost cohort has not ended or been disposed of
    /// [SetText]
    /// This cost cohort ended through
    pub ended: Option<AmortizationEnd>,
}

/// Code-specific cohort; no arbitrary manual period for any expense
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum OtherAmortizationKind {
    /// Section 169 certified pollution-control facility
    Pollution { facts: PollutionControlCost },
    /// Section 174 or 174A research expenditures
    Research { facts: ResearchCost },
    /// Section 178 lease-acquisition cost
    Lease { facts: LeaseAcquisitionCost },
    /// Section 59(e) statutory preference write-off
    Preference { election: PreferenceWriteoff },
    /// Qualifying creative-property costs under Rev. Proc. 2004-36
    Creative { facts: CreativePropertyCost },
}

/// Section 169 certification, legal period and basis split
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PollutionControlCost {
    /// State certification under section 169(d)
    pub state_certified: bool,
    /// Federal certification under section 169(d)
    pub federal_certified: bool,
    /// New identifiable treatment facility satisfying statutory restrictions
    pub facility_qualifies: bool,
    /// Date associated plant first operated
    #[libertas_date_only]
    #[libertas_number(max = $crate::year_constants::TAX_YEAR_END_DATE)]
    pub plant_first_operation: u32,
    /// Qualifying post-April 11, 2005 atmospheric facility at primarily coal-fired property
    pub coal_atmospheric: bool,
    /// Useful life for statutory fifteen-year basis allocation
    #[libertas_number(min = 1)]
    pub useful_life_months: u32,
    /// Documented eligible statutory basis portion before bonus
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amortizable_allocation: i64,
    /// Allowed or allowable bonus on this facility
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub bonus: i64,
    /// Original elected amortization beginning
    pub start_choice: PollutionStart,
    /// Original written termination effective month, if any
    /// [NullText]
    /// The section 169 amortization election has not been terminated
    /// [SetText]
    /// The election termination took effect on
    #[libertas_date_only]
    #[libertas_number(max = $crate::year_constants::TAX_YEAR_END_DATE)]
    #[libertas_default($null)]
    pub terminated: Option<u32>,
}

/// Original section 169 election start
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum PollutionStart {
    /// Month after completion/acquisition
    NextMonth,
    /// First month of next taxable year
    NextTaxYear,
}

/// Annual research cohort; original dates determine law
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ResearchCost {
    /// Domestic or foreign research
    pub location: ResearchLocation,
    /// Original current-expense or amortization election
    pub treatment: ResearchTreatment,
    /// Actual property developed from this research, saved under Business equipment
    /// Save the actual property first, then select it here. Keep its basis, prior
    /// recovery and disposal facts on that property once. Mandatory section174
    /// expenditures are not part of its disposal basis and keep amortizing.
    /// [NullText]
    /// No separately recorded developed property is linked
    /// [SetText]
    /// The actual property is
    #[libertas_default($null)]
    pub property: Option<i64>,
}

/// Statutory research location
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum ResearchLocation {
    /// Domestic research under section174A, with its actual section280C adjustment
    Domestic { credit_adjustment: crate::SourceCents },
    /// Foreign research; no domestic research-credit adjustment applies
    Foreign,
}

/// Original lawful research treatment, with only its applicable source facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum ResearchTreatment {
    /// Domestic post-2024 current deduction
    CurrentExpense,
    /// Domestic elective capitalization from first benefits
    Capitalize { election: ResearchCapitalization },
    /// Section59(e) ten-year election
    TenYearElection,
    /// Mandatory foreign fifteen-year midpoint recovery
    ForeignMidYear,
    /// Original2022-2024 five-year domestic midpoint schedule
    OldDomesticMidYear { transition: DomesticResearchAcceleration },
    /// Original pre-2022 capitalization schedule
    Pre2022Capitalized { election: ResearchCapitalization },
}

/// Actual elective research capitalization and any whole-cohort property conversion
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ResearchCapitalization {
    /// Original elected recovery period in months
    #[libertas_number(min = 60)]
    pub months: u32,
    /// First month in which benefits were realized
    /// [NullText]
    /// The first-benefit date is not established
    /// [SetText]
    /// Benefits were first realized on
    #[libertas_date_only]
    #[libertas_default($null)]
    pub first_benefits: Option<u32>,
    /// Actual remaining deferred research basis converted into developed property
    /// Under section1.174-4(a)(4), the remaining deferred amount becomes the
    /// developed property's unrecovered cost. Select the real property above,
    /// using its canonical in-service date and its actual basis inclusive of this
    /// transfer. This source supports a whole-cohort transfer, not an allocation
    /// among several properties or a presumed zero basis.
    /// [NullText]
    /// This deferred research cohort has not converted into developed property
    /// [SetText]
    /// The original basis transfer was
    #[libertas_default($null)]
    pub transfer: Option<crate::SourceCents>,
}

/// Actual2025 transition treatment; an acceleration carries its original balance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum DomesticResearchAcceleration {
    /// Retain the original schedule; no acceleration election
    None,
    /// All remaining domestic basis was deducted in2025
    All2025 { unamortized_2024: crate::SourceCents },
    /// Half in2025 and half in2026
    Half2025Half2026 { unamortized_2024: crate::SourceCents },
}

/// Section 178 term and renewal facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct LeaseAcquisitionCost {
    /// Remaining lease term at acquisition
    #[libertas_number(min = 1)]
    pub remaining_term_months: u32,
    /// Renewal/expected continuation months
    #[libertas_number(min = 0)]
    pub renewal_months: u32,
    /// Part of acquisition cost attributable to the original remaining term
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub cost_attributed_to_remaining_term: i64,
}

/// Section 59(e) fixed write-off period; enter research under Research costs and its ten-year election
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum PreferenceWriteoff {
    /// Section 173 circulation: thirty-six months
    Circulation36,
    /// Section 263(c) intangible drilling/development: sixty months
    Drilling60,
    /// Sections 616/617 mining: one hundred twenty months
    Mining120,
}

/// Source-defined three-year unproduced creative property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CreativePropertyCost {
    /// First capitalized transaction date
    #[libertas_date_only]
    #[libertas_number(max = $crate::year_constants::TAX_YEAR_END_DATE)]
    pub first_capitalized: u32,
    /// Property was not set for production within three years of the first capitalized transaction
    pub not_set_for_production_three_years: bool,
    /// This activity is a trade or business of film production
    pub film_production_trade: bool,
    /// Date properly written off for books under the applicable SOP00-2 method
    #[libertas_date_only]
    #[libertas_number(max = $crate::year_constants::TAX_YEAR_END_DATE)]
    pub book_writeoff: u32,
    /// Original safe-harbor method adopted on the return
    pub original_election: bool,
}

// Code-specific calendars and basis stay in the existing amortization owner;
// the shared kernels own the cap and proration arithmetic. All amounts are cents.
// https://www.irs.gov/instructions/i4562 (Amortization)
// https://www.irs.gov/pub/irs-drop/rp-04-36.pdf sections 4–5
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CostRecovery {
    pub basis: i64,
    pub started: u32,
    pub months: i64,
    pub current_expense: i64,
    pub amortization: i64,
    pub remaining: i64,
    pub ordinary_proceeds: Option<i64>,
}

fn month(date: u32) -> Month {
    Month { year: date / 10000, month: date / 100 % 100 }
}
fn calendar_month(date: u32) -> i64 {
    i64::from(date / 10000) * 12 + i64::from(date / 100 % 100) - 1
}
fn month_date(month: i64) -> u32 {
    ((month / 12) * 10000 + (month % 12 + 1) * 100 + 1) as u32
}
fn year_begin(date: u32) -> u32 {
    date / 10000 * 10000 + 101
}
fn midpoint(date: u32) -> u32 {
    date / 10000 * 10000 + 701
}
fn valid_cost_date(date: u32) -> bool {
    (19000101..=crate::year_constants::TAX_YEAR_END_DATE).contains(&date)
        && crate::federal_rules::valid_date(date)
}

impl OtherAmortizableCost {
    fn prior(&self) -> Option<i64> {
        self.prior_allowed.source_fact().ok()?.require(|| ()).ok().map(|v| v.unwrap_or(0))
    }
    pub(crate) fn valid(&self) -> bool {
        let money = |value| value >= 0;
        let prior = self.prior().unwrap_or(0);
        let no_prior = !matches!(&self.kind, OtherAmortizationKind::Research { facts }
            if matches!(facts.treatment, ResearchTreatment::CurrentExpense))
            || prior == 0;
        no_prior
            && !self.description.trim().is_empty()
            && self.prior_allowed.source_fact().is_ok()
            && money(self.cost)
            && (0..=self.cost).contains(&prior)
            && valid_cost_date(self.incurred)
            && self.began.is_none_or(valid_cost_date)
            && self.ended.as_ref().is_none_or(|end| {
                valid_cost_date(end.date)
                    && self.began.is_none_or(|began| end.date >= began)
                    && end.how.valid()
            })
            && match &self.kind {
                OtherAmortizationKind::Pollution { facts } => {
                    valid_cost_date(facts.plant_first_operation)
                        && facts.useful_life_months > 0
                        && (0..=self.cost).contains(&facts.amortizable_allocation)
                        && (0..=facts.amortizable_allocation).contains(&facts.bonus)
                        && facts.terminated.is_none_or(valid_cost_date)
                }
                OtherAmortizationKind::Research { facts } => {
                    let location = match &facts.location {
                        ResearchLocation::Domestic { credit_adjustment } => {
                            credit_adjustment.source_fact().is_ok()
                        }
                        ResearchLocation::Foreign => true,
                    };
                    location
                        && match &facts.treatment {
                            ResearchTreatment::Capitalize { election }
                            | ResearchTreatment::Pre2022Capitalized { election } => {
                                election.months >= 60
                                    && election.first_benefits.is_none_or(valid_cost_date)
                                    && election
                                        .transfer
                                        .as_ref()
                                        .is_none_or(|v| v.source_fact().is_ok())
                            }
                            ResearchTreatment::OldDomesticMidYear {
                                transition:
                                    DomesticResearchAcceleration::All2025 { unamortized_2024 }
                                    | DomesticResearchAcceleration::Half2025Half2026 { unamortized_2024 },
                            } => unamortized_2024.source_fact().is_ok(),
                            _ => true,
                        }
                }
                OtherAmortizationKind::Lease { facts } => {
                    facts.remaining_term_months > 0
                        && facts.remaining_term_months.checked_add(facts.renewal_months).is_some()
                        && (0..=self.cost).contains(&facts.cost_attributed_to_remaining_term)
                }
                OtherAmortizationKind::Preference { .. } => true,
                OtherAmortizationKind::Creative { facts } => {
                    valid_cost_date(facts.first_capitalized) && valid_cost_date(facts.book_writeoff)
                }
            }
    }

    // A missing/contradictory legal source is an explicit result, never zero recovery.
    pub(crate) fn recovery(
        &self,
        year: i64,
        assets: &[crate::BusinessAsset],
    ) -> Result<CostRecovery, &'static str> {
        use crate::kernel::phase::cap;
        use crate::kernel::ratio::positive;
        use OtherAmortizationKind as Kind;
        if !self.valid() {
            return Err("AMORTIZATION_SOURCE_INVALID");
        }
        // Current expensing and an actual2025 accelerated closing balance do
        // not consume the original schedule's prior-amortization operand.
        let current_expense_source = matches!(&self.kind, Kind::Research { facts }
            if matches!(facts.treatment, ResearchTreatment::CurrentExpense));
        let accelerated_source = matches!(&self.kind, Kind::Research { facts }
            if matches!(facts.treatment, ResearchTreatment::OldDomesticMidYear { transition:
                DomesticResearchAcceleration::All2025 { .. } | DomesticResearchAcceleration::Half2025Half2026 { .. } }));
        let prior = if current_expense_source || accelerated_source {
            0
        } else {
            self.prior().ok_or("AMORTIZATION_HISTORY_UNAVAILABLE")?
        };
        let mut basis = self.cost;
        let mut current_expense = 0;
        let (started, months) = match &self.kind {
            Kind::Pollution { facts } => {
                if !facts.state_certified || !facts.federal_certified || !facts.facility_qualifies {
                    return Err("POLLUTION_CERTIFICATION_NEEDED");
                }
                // Section169(a)/(c): only the statutory useful-life portion is
                // amortizable; residual basis belongs to ordinary depreciation.
                let allocation_cap = fraction(
                    self.cost,
                    cap(180, [i64::from(facts.useful_life_months)]),
                    i64::from(facts.useful_life_months),
                );
                if facts.amortizable_allocation > allocation_cap {
                    return Err("POLLUTION_ALLOCATION_ABOVE_STATUTORY_BASIS");
                }
                basis = positive(facts.amortizable_allocation - facts.bonus);
                // Completion/acquisition is not necessarily the cost-incurred date.
                // The original lawful start is supplied from the election record.
                let start = self.began.ok_or("POLLUTION_ORIGINAL_ELECTION_START")?;
                if start % 100 != 1
                    || (matches!(facts.start_choice, PollutionStart::NextTaxYear)
                        && start % 10000 != 101)
                {
                    return Err("POLLUTION_ORIGINAL_ELECTION_START");
                }
                if (!facts.coal_atmospheric && facts.plant_first_operation > 19751231)
                    || (facts.coal_atmospheric && self.incurred <= 20050411)
                {
                    return Err("POLLUTION_ORIGINAL_FACILITY_DATE");
                }
                let period = if facts.coal_atmospheric && facts.plant_first_operation > 19751231 {
                    84
                } else if facts.coal_atmospheric {
                    return Err("POLLUTION_COAL_PLANT_DATE");
                } else {
                    60
                };
                (start, period)
            }
            Kind::Research { facts } => {
                let credit = match &facts.location {
                    ResearchLocation::Foreign => 0,
                    ResearchLocation::Domestic { credit_adjustment } => credit_adjustment
                        .source_fact()
                        .map_err(|_| "RESEARCH_CREDIT_ADJUSTMENT")?
                        .require(|| "RESEARCH_CREDIT_ADJUSTMENT")?
                        .unwrap_or(0),
                };
                basis = basis
                    .checked_sub(credit)
                    .filter(|v| *v >= 0)
                    .ok_or("RESEARCH_CREDIT_ADJUSTMENT")?;
                let original_year = self.incurred / 10000;
                match (&facts.location, &facts.treatment) {
                    (ResearchLocation::Domestic { .. }, ResearchTreatment::CurrentExpense)
                        if original_year >= 2025 =>
                    {
                        current_expense = if i64::from(original_year) == year { basis } else { 0 };
                        (self.incurred, 0)
                    }
                    (
                        ResearchLocation::Domestic { .. },
                        ResearchTreatment::Capitalize { election },
                    ) if original_year >= 2025 => (
                        election.first_benefits.ok_or("RESEARCH_FIRST_BENEFIT_DATE_NEEDED")?,
                        i64::from(election.months),
                    ),
                    (ResearchLocation::Domestic { .. }, ResearchTreatment::TenYearElection)
                        if !(2022..2025).contains(&original_year) =>
                    {
                        (year_begin(self.incurred), 120)
                    }
                    (ResearchLocation::Foreign, ResearchTreatment::ForeignMidYear)
                        if original_year >= 2022 =>
                    {
                        (midpoint(self.incurred), 180)
                    }
                    (
                        ResearchLocation::Domestic { .. },
                        ResearchTreatment::OldDomesticMidYear { transition },
                    ) => {
                        if !(2022..=2024).contains(&original_year) {
                            return Err("RESEARCH_TRANSITION_2025");
                        }
                        if !matches!(transition, DomesticResearchAcceleration::None) {
                            let source = match transition {
                                DomesticResearchAcceleration::All2025 { unamortized_2024 }
                                | DomesticResearchAcceleration::Half2025Half2026 {
                                    unamortized_2024,
                                } => unamortized_2024,
                                DomesticResearchAcceleration::None => unreachable!(),
                            };
                            let unamortized = source
                                .source_fact()
                                .map_err(|_| "RESEARCH_TRANSITION_2025")?
                                .require(|| "RESEARCH_TRANSITION_2025")?
                                .unwrap_or(0);
                            if unamortized > basis {
                                return Err("RESEARCH_TRANSITION_2025");
                            }
                            let half = fraction(unamortized, 1, 2);
                            let amount = match (transition, year) {
                                (DomesticResearchAcceleration::All2025 { .. }, 2025) => unamortized,
                                (DomesticResearchAcceleration::Half2025Half2026 { .. }, 2025) => {
                                    half
                                }
                                (DomesticResearchAcceleration::Half2025Half2026 { .. }, 2026) => {
                                    unamortized - half
                                }
                                _ => 0,
                            };
                            return Ok(CostRecovery {
                                basis: unamortized,
                                started: 20250101,
                                months: 0,
                                current_expense: 0,
                                amortization: amount,
                                remaining: if year < 2025 {
                                    unamortized
                                } else if year == 2025
                                    && matches!(
                                        transition,
                                        DomesticResearchAcceleration::Half2025Half2026 { .. }
                                    )
                                {
                                    unamortized - half
                                } else {
                                    0
                                },
                                ordinary_proceeds: Some(0),
                            });
                        }
                        (midpoint(self.incurred), 60)
                    }
                    (_, ResearchTreatment::Pre2022Capitalized { election })
                        if original_year < 2022 =>
                    {
                        (
                            election.first_benefits.ok_or("RESEARCH_FIRST_BENEFIT_DATE_NEEDED")?,
                            i64::from(election.months),
                        )
                    }
                    _ => return Err("RESEARCH_LOCATION_PERIOD"),
                }
            }
            Kind::Lease { facts } => {
                let renewals = if i128::from(facts.cost_attributed_to_remaining_term) * 4
                    >= i128::from(self.cost) * 3
                {
                    0
                } else {
                    facts.renewal_months
                };
                (
                    self.began.ok_or("LEASE_ORIGINAL_ELECTION_START")?,
                    i64::from(
                        facts
                            .remaining_term_months
                            .checked_add(renewals)
                            .ok_or("LEASE_TERM_OVERFLOW")?,
                    ),
                )
            }
            Kind::Preference { election } => {
                // Approved correction e1207ad: drilling alone starts in the
                // paid/incurred month, under section59(e)(1),(2)(C).
                match election {
                    PreferenceWriteoff::Circulation36 => (year_begin(self.incurred), 36),
                    PreferenceWriteoff::Drilling60 => {
                        (month_date(calendar_month(self.incurred)), 60)
                    }
                    PreferenceWriteoff::Mining120 => (year_begin(self.incurred), 120),
                }
            }
            Kind::Creative { facts } => {
                if !facts.not_set_for_production_three_years
                    || !facts.film_production_trade
                    || !facts.original_election
                    || facts.book_writeoff < crate::dates::add_years(facts.first_capitalized, 3)
                {
                    return Err("CREATIVE_SAFE_HARBOR_FACTS");
                }
                (midpoint(facts.book_writeoff), 180)
            }
        };
        if self.began.is_some_and(|began| began != started)
            || prior > basis
            || started < self.incurred
                && !matches!(
                    self.kind,
                    Kind::Preference { .. } | Kind::Research { .. } | Kind::Creative { .. }
                )
        {
            return Err("AMORTIZATION_CALENDAR_OR_BASIS");
        }
        if matches!(
            self.kind,
            Kind::Research {
                facts: ResearchCost {
                    treatment: ResearchTreatment::Capitalize { .. }
                        | ResearchTreatment::Pre2022Capitalized { .. },
                    ..
                }
            }
        ) && started < self.incurred
        {
            return Err("RESEARCH_FIRST_BENEFIT_DATE_NEEDED");
        }
        let continues_after_end = matches!(
            self.kind,
            Kind::Creative { .. }
                | Kind::Research {
                    facts: ResearchCost {
                        treatment: ResearchTreatment::ForeignMidYear
                            | ResearchTreatment::OldDomesticMidYear { .. },
                        ..
                    }
                }
        );
        let stop = if continues_after_end { None } else { self.ended.as_ref().map(|end| end.date) };
        let termination = match &self.kind {
            Kind::Pollution { facts } => facts.terminated,
            _ => None,
        };
        let transfer = match &self.kind {
            Kind::Research {
                facts:
                    ResearchCost {
                        treatment:
                            ResearchTreatment::Capitalize { election }
                            | ResearchTreatment::Pre2022Capitalized { election },
                        property,
                        ..
                    },
            } => election.transfer.as_ref().map(|source| (source, *property)),
            _ => None,
        };
        let transferred = match transfer {
            Some((source, property)) => {
                let property = crate::asset_sale::research_property(
                    assets,
                    property.ok_or("RESEARCH_TRANSFER_PROPERTY_NEEDED")?,
                )?;
                let amount = source
                    .source_fact()
                    .map_err(|_| "RESEARCH_TRANSFER_BASIS_NEEDED")?
                    .require(|| "RESEARCH_TRANSFER_BASIS_NEEDED")?
                    .unwrap_or(0);
                if property.placed < started || property.business_share_of(property.cost) < amount {
                    return Err("RESEARCH_TRANSFER_BASIS_NEEDED");
                }
                Some((property.placed, amount))
            }
            None => None,
        };
        let recovery = if months == 0 {
            crate::kernel::cost_recovery::monthly_history::ActualMonthlyAmounts {
                current_cents: 0,
                remaining_cents: 0,
            }
        } else {
            ActualMonthlySchedule {
                started: month(started),
                period_months: months as u32,
                tax_year: year as u32,
                basis_cents: basis,
                prior_recovery_cents: Some(prior),
                allowed_through: stop.map(month),
                allowed_before: match (termination, transferred.map(|(date, _)| date)) {
                    (Some(a), Some(b)) => Some(month(a.min(b))),
                    (Some(a), None) | (None, Some(a)) => Some(month(a)),
                    _ => None,
                },
            }
            .recover()
            .map_err(|_| "AMORTIZATION_CALENDAR_OR_BASIS")?
        };
        let amortization = recovery.current_cents;
        let remaining = if let Some((_, amount)) =
            transferred.filter(|(date, _)| i64::from(date / 10000) <= year)
        {
            // Reg1.174-4(a)(4): transfer the complete actual deferred balance.
            // Partial multi-property allocation needs its actual source split;
            // it cannot be derived from the developed property's total cost.
            if amount != recovery.remaining_cents {
                return Err("RESEARCH_TRANSFER_BASIS_NEEDED");
            }
            0
        } else {
            recovery.remaining_cents
        };
        let ordinary_proceeds = if matches!(self.kind, Kind::Creative { .. }) {
            self.ended.as_ref().filter(|end| i64::from(end.date / 10000) == year).map_or(
                Some(0),
                |end| match &end.how {
                    crate::DisposalKind::Sold {
                        proceeds,
                        selling_expenses,
                        payment: crate::SalePayment::InFull,
                        buyer: crate::Buyer::Unrelated,
                        livestock: crate::Livestock::No,
                    } => Some(positive(proceeds - selling_expenses)),
                    crate::DisposalKind::Scrapped | crate::DisposalKind::PersonalUse => Some(0),
                    _ => None,
                },
            )
        } else {
            Some(0)
        };
        Ok(CostRecovery {
            basis,
            started,
            months,
            current_expense,
            amortization,
            remaining,
            ordinary_proceeds,
        })
    }
}

// Rev. Proc. 2004-36 section5.01 treats all qualifying creative costs
// beginning amortization in one year as one asset. Calculate and round that
// schedule once, then allocate its deduction back to the entered properties.
// Individual disposition proceeds retain their own source facts.
pub(crate) struct CostCohorts {
    pub parts: Vec<Result<CostRecovery, &'static str>>,
    forms: Vec<Option<CostRecovery>>,
}
pub(crate) fn cost_cohorts(
    costs: &[OtherAmortizableCost],
    year: i64,
    assets: &[crate::BusinessAsset],
) -> CostCohorts {
    use crate::kernel::ratio::{allocate_remaining, positive};
    let mut parts: Vec<_> = costs.iter().map(|cost| cost.recovery(year, assets)).collect();
    let mut forms: Vec<_> = parts.iter().map(|part| part.as_ref().ok().cloned()).collect();
    let mut visited = alloc::vec![false; costs.len()];
    for first in 0..costs.len() {
        if visited[first] || !matches!(costs[first].kind, OtherAmortizationKind::Creative { .. }) {
            continue;
        }
        let Ok(recovery) = &parts[first] else {
            continue;
        };
        let started = recovery.started;
        let indices: Vec<_> = (first..costs.len())
            .filter(|&index| {
                matches!(costs[index].kind, OtherAmortizationKind::Creative { .. })
                    && parts[index].as_ref().is_ok_and(|part| part.started == started)
            })
            .collect();
        let Some(basis) = crate::kernel::ratio::checked_sum(
            indices.iter().map(|&index| parts[index].as_ref().unwrap().basis),
        ) else {
            for &index in &indices {
                parts[index] = Err("AMORTIZATION_TOTAL_UNREPRESENTABLE");
                forms[index] = None;
            }
            continue;
        };
        let weights: Vec<_> = indices
            .iter()
            .map(|&index| parts[index].as_ref().unwrap().basis - costs[index].prior().unwrap())
            .collect();
        let Some(available) = crate::kernel::ratio::checked_sum(weights.iter().copied()) else {
            for &index in &indices {
                parts[index] = Err("AMORTIZATION_TOTAL_UNREPRESENTABLE");
                forms[index] = None;
            }
            continue;
        };
        let deduction = ActualMonthlySchedule {
            started: month(started),
            period_months: 180,
            tax_year: year as u32,
            basis_cents: basis,
            prior_recovery_cents: Some(basis - available),
            allowed_through: None,
            allowed_before: None,
        }
        .recover()
        .expect("validated creative annual schedule")
        .current_cents;
        let shares = allocate_remaining(deduction, &weights);
        let mut form = recovery.clone();
        form.basis = basis;
        form.amortization = deduction;
        form.remaining = available - deduction;
        for (&index, share) in indices.iter().zip(shares) {
            visited[index] = true;
            let part = parts[index].as_mut().unwrap();
            part.amortization = share;
            part.remaining = positive(part.basis - costs[index].prior().unwrap() - share);
            forms[index] = None;
        }
        forms[first] = Some(form);
    }
    CostCohorts { parts, forms }
}

pub(crate) fn report_other_costs(
    costs: &[OtherAmortizableCost],
    assets: &[crate::BusinessAsset],
    result: &mut FederalResult,
    id: i64,
    #[cfg(test)] section: &str,
) {
    let cohorts = cost_cohorts(costs, 2026, assets);
    for (index, cost) in costs.iter().enumerate() {
        cost.report(
            result,
            id,
            index,
            &cohorts.parts[index],
            cohorts.forms[index].as_ref(),
            #[cfg(test)]
            section,
        );
    }
}

impl OtherAmortizableCost {
    pub(crate) fn needs_amt_source(&self, material_participation: bool) -> bool {
        // IRS i6251 line2r and Pollution Control Facilities:
        // https://www.irs.gov/instructions/i6251 . Never substitute regular
        // section169 recovery/basis for ADS or post1998 straight-line AMT.
        match &self.kind {
            OtherAmortizationKind::Pollution { .. } => true,
            OtherAmortizationKind::Research { facts } if !material_participation => matches!(
                facts.treatment,
                ResearchTreatment::CurrentExpense
                    | ResearchTreatment::ForeignMidYear
                    | ResearchTreatment::OldDomesticMidYear { .. }
            ),
            _ => false,
        }
    }

    pub(crate) fn report(
        &self,
        result: &mut FederalResult,
        id: i64,
        index: usize,
        recovery: &Result<CostRecovery, &'static str>,
        form: Option<&CostRecovery>,
        #[cfg(test)] section: &str,
    ) {
        let context = crate::IssueContext::income_id(id).item("data.other_amortization", index);
        let Ok(recovery) = recovery else {
            crate::federal_rules::require_localized(
                result,
                false,
                &context,
                #[cfg(test)]
                section,
                "ISSUE_AMORTIZATION_DISPOSITION_SOURCE_NEEDED",
                #[cfg(test)]
                "Resolve this disposed cost cohort's original method and basis before its mandatory disposition income can be calculated.",
            );
            return;
        };
        crate::federal_rules::require_localized(
            result,
            recovery.ordinary_proceeds.is_some(),
            &context,
            #[cfg(test)]
            section,
            "ISSUE_CREATIVE_TRANSFER_OWNER_NEEDED",
            #[cfg(test)]
            "This creative-property transfer needs its section5.04 transfer or whole-business owner before the required gain is calculated.",
        );
        let current = recovery.started / 10000 == 2026 && recovery.months > 0;
        let code = match &self.kind {
            OtherAmortizationKind::Pollution { .. } => "169",
            OtherAmortizationKind::Research { facts }
                if facts.treatment == ResearchTreatment::TenYearElection =>
            {
                "59(e)"
            }
            OtherAmortizationKind::Research { facts }
                if matches!(facts.location, ResearchLocation::Foreign) =>
            {
                "174"
            }
            OtherAmortizationKind::Research { .. } => "174A",
            OtherAmortizationKind::Lease { .. } => "178",
            OtherAmortizationKind::Preference { .. } => "59(e)",
            OtherAmortizationKind::Creative { .. } => "Rev. Proc. 2004-36",
        };
        let source = if matches!(self.kind, OtherAmortizationKind::Creative { .. }) {
            "https://www.irs.gov/pub/irs-drop/rp-04-36.pdf"
        } else {
            "https://www.irs.gov/instructions/i4562"
        };
        let prefix = format!("business_{id}_amortization_{index}");
        let input = ["income.business.other_amortization"];
        if let Some(form) = form.filter(|form| form.months > 0) {
            let rows = [
                (
                    "1(a)",
                    "description",
                    LineValue::Text,
                    crate::Message::new(
                        "TEXT",
                        alloc::vec![crate::MessageArg::Text(self.description.clone())],
                    ),
                ),
                (
                    "1(b)",
                    "started",
                    LineValue::Text,
                    crate::Message::new(
                        "AMORTIZATION_DATE_VALUE",
                        alloc::vec![crate::MessageArg::DateOnly(i64::from(form.started))],
                    ),
                ),
                (
                    "1(c)",
                    "basis",
                    LineValue::MoneyCents(form.basis),
                    msg!("EXPLAIN_OTHER_AMORTIZATION_BASIS"),
                ),
                (
                    "1(d)",
                    "code",
                    LineValue::Text,
                    crate::Message::new("TEXT", alloc::vec![crate::MessageArg::Text(code.into())]),
                ),
                (
                    "1(e)",
                    "period",
                    LineValue::CountWhole(form.months),
                    msg!("EXPLAIN_OTHER_AMORTIZATION_PERIOD"),
                ),
                (
                    "1(f)",
                    "deduction",
                    LineValue::MoneyCents(form.amortization),
                    msg!("EXPLAIN_OTHER_AMORTIZATION_DEDUCTION"),
                ),
            ];
            emit_lines(result, &mut (), rows, |(line, suffix, value, explanation), _, _| {
                Some(calculation_row(
                    current.then(|| FormLine::new(TaxForm::Form4562B, line)),
                    &format!("{prefix}_{suffix}"),
                    value,
                    &input,
                    source,
                    explanation,
                ))
            });
        }
        line_cents(
            result,
            &format!("{prefix}_current_expense"),
            recovery.current_expense,
            &input,
            source,
            msg!("EXPLAIN_OTHER_AMORTIZATION_CURRENT_EXPENSE"),
        );
        line_cents(
            result,
            &format!("{prefix}_remaining"),
            recovery.remaining,
            &input,
            source,
            msg!("EXPLAIN_OTHER_AMORTIZATION_REMAINING"),
        );
        if let Some(proceeds) = recovery.ordinary_proceeds.filter(|amount| *amount != 0) {
            push_line(
                result,
                Some(FormLine::new(TaxForm::Form4797, "11")),
                &format!("{prefix}_ordinary"),
                proceeds,
                &input,
                source,
                msg!("EXPLAIN_CREATIVE_PROPERTY_ORDINARY"),
            );
        }
    }
}

// Only available schedules contribute a deduction; report_other_costs retains
// the mandatory unavailable-source issue instead of claiming a complete return.
pub(crate) fn totals(
    costs: &[OtherAmortizableCost],
    assets: &[crate::BusinessAsset],
) -> Option<[i64; 3]> {
    let mut totals = [0_i64; 3];
    for part in cost_cohorts(costs, 2026, assets).parts.into_iter().filter_map(Result::ok) {
        totals[0] = totals[0].checked_add(part.current_expense)?;
        let index = if part.started / 10000 < 2026 { 1 } else { 2 };
        totals[index] = totals[index].checked_add(part.amortization)?;
    }
    Some(totals)
}
pub(crate) fn deduction(
    costs: &[OtherAmortizableCost],
    assets: &[crate::BusinessAsset],
) -> Option<i64> {
    crate::kernel::ratio::checked_sum(totals(costs, assets)?)
}

pub(crate) fn property_sales(
    business: &crate::FederalBusiness,
) -> Result<Vec<(usize, crate::asset_sale::AmortizedSale)>, &'static str> {
    let mut sales = Vec::new();
    for (index, (cost, recovery)) in business
        .other_amortization
        .iter()
        .zip(cost_cohorts(&business.other_amortization, 2026, &business.assets).parts)
        .enumerate()
    {
        let Some(end) = cost.ended.as_ref().filter(|end| end.date / 10000 == 2026) else {
            continue;
        };
        let recovery = recovery?;
        if matches!(cost.kind, OtherAmortizationKind::Creative { .. }) {
            let proceeds =
                recovery.ordinary_proceeds.ok_or("ISSUE_CREATIVE_TRANSFER_OWNER_NEEDED")?;
            if proceeds != 0 {
                sales.push((
                    index,
                    crate::asset_sale::AmortizedSale {
                        value: crate::farm_land::LandGain {
                            gain: crate::kernel::ratio::dollars(proceeds),
                            percent: 0,
                            recapture: 0,
                            ordinary: crate::kernel::ratio::dollars(proceeds),
                            section1231: 0,
                            unrecaptured: 0,
                        },
                        realized: crate::kernel::ratio::dollars(proceeds),
                        recognized_gain: crate::kernel::ratio::dollars(proceeds),
                        cost: 0,
                        recovery: 0,
                        postponed: 0,
                        disallowed: 0,
                        part_two: true,
                        installment: None,
                        section1254: false,
                    },
                ));
            }
            continue;
        }
        // Mandatory section174 SRE continues its schedule and supplies no
        // disposed-property basis. Elective research similarly needs the actual
        // developed-property owner; remaining expenditures are never that basis.
        if matches!(cost.kind, OtherAmortizationKind::Research { .. }) {
            if !matches!(end.how, crate::DisposalKind::PersonalUse) {
                return Err("ISSUE_AMORTIZATION_DISPOSITION_SOURCE_NEEDED");
            }
            continue;
        }
        // Residual pollution-facility basis has its own depreciation owner.
        // Without that history the cohort cannot claim a complete sale result.
        let (basis, extra_recovery) = match &cost.kind {
            OtherAmortizationKind::Pollution { facts } => {
                if facts.amortizable_allocation != cost.cost {
                    return Err("ISSUE_AMORTIZATION_DISPOSITION_SOURCE_NEEDED");
                }
                (cost.cost, facts.bonus)
            }
            _ => (recovery.basis, 0),
        };
        let recovered = cost
            .prior()
            .and_then(|prior| prior.checked_add(recovery.amortization))
            .and_then(|v| v.checked_add(extra_recovery))
            .ok_or("ISSUE_AMORTIZATION_DISPOSITION_SOURCE_NEEDED")?;
        let sale =
            crate::asset_sale::amortized_property_sale(crate::asset_sale::AmortizedProperty {
                acquired: cost.incurred,
                end,
                cost_cents: basis,
                recovery_cents: recovered,
                section1254: matches!(
                    cost.kind,
                    OtherAmortizationKind::Preference {
                        election: PreferenceWriteoff::Drilling60 | PreferenceWriteoff::Mining120
                    }
                ),
            })?;
        if let Some(sale) = sale {
            sales.push((index, sale));
        }
    }
    Ok(sales)
}

pub(crate) fn property_total(
    business: &crate::FederalBusiness,
) -> Result<Option<crate::farm_land::LandGain>, &'static str> {
    let mut total = None;
    for sale in all_property_sales(business)? {
        let value = total.get_or_insert(crate::farm_land::LandGain::zero());
        *value = crate::asset_sale::checked_property_sum(*value, sale.value)
            .ok_or("AMORTIZATION_TOTAL_UNREPRESENTABLE")?;
    }
    Ok(total)
}

pub(crate) fn timber_sale(
    cost: &crate::BusinessTimberCost,
) -> Result<Option<crate::asset_sale::AmortizedSale>, &'static str> {
    let Some(end) = &cost.ended else { return Ok(None) };
    if end.date / 10000 != crate::year_constants::TAX_YEAR {
        return Ok(None);
    }
    // IRC1245(b)(7): the original expense retains a distinct post-window
    // character boundary. Do not presume that it has the same recapture class.
    if cost.facts.current_expense_elected > 0
        && end.date >= crate::dates::add_years(cost.incurred, 10)
    {
        return Err("ISSUE_TIMBER_DISPOSITION_NEEDED");
    }
    let recovered = cost.recovery(2026).ok_or("ISSUE_TIMBER_ORIGINAL_HISTORY_NEEDED")?;
    let prior = cost
        .prior_allowed
        .source_fact()
        .map_err(|_| "ISSUE_TIMBER_ORIGINAL_HISTORY_NEEDED")?
        .require(|| "ISSUE_TIMBER_ORIGINAL_HISTORY_NEEDED")?
        .unwrap_or(0);
    let cost_cents = cost.cost
        - if cost.facts.reimbursement_taxable { 0 } else { cost.facts.government_reimbursement };
    let recovery_cents = prior
        .checked_add(cost.facts.current_expense_elected)
        .and_then(|v| v.checked_add(recovered.amortization_cents))
        .ok_or("ISSUE_TIMBER_ORIGINAL_HISTORY_NEEDED")?;
    crate::asset_sale::amortized_property_sale(crate::asset_sale::AmortizedProperty {
        acquired: cost.incurred,
        end,
        cost_cents,
        recovery_cents,
        section1254: false,
    })
}

fn all_property_sales(
    business: &crate::FederalBusiness,
) -> Result<Vec<crate::asset_sale::AmortizedSale>, &'static str> {
    let mut sales: Vec<_> = property_sales(business)?.into_iter().map(|(_, sale)| sale).collect();
    for cost in &business.timber {
        if let Some(sale) = timber_sale(cost)? {
            sales.push(sale);
        }
    }
    Ok(sales)
}
pub(crate) fn property_gross(business: &crate::FederalBusiness) -> Result<i64, &'static str> {
    crate::kernel::ratio::checked_sum(
        all_property_sales(business)?.into_iter().map(|sale| sale.value.gain.max(0)),
    )
    .ok_or("ISSUE_AMORTIZATION_DISPOSITION_SOURCE_NEEDED")
}

// The shared graph supplies global identity, kind and the actual containing
// business; no label/index/cost becomes a property association.
pub(crate) fn source_issue(
    draft: &crate::FederalDraft,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    let sources = crate::source_associations::SourceAssociations::new(draft);
    for entry in &draft.income {
        let crate::FederalIncome::Business { data } = &entry.income else { continue };
        for (index, cost) in data.other_amortization.iter().enumerate() {
            let OtherAmortizationKind::Research { facts } = &cost.kind else { continue };
            let Some(id) = facts.property else { continue };
            if cost.ended.is_some() || sources.asset(entry.id, id).is_err() {
                return Some((
                    entry.id,
                    crate::validation::ValidationIssue::new("RESEARCH_PROPERTY_SOURCE_INVALID")
                        .at_field("property")
                        .in_field("facts")
                        .in_field("kind")
                        .in_list("other_amortization", index)
                        .in_field("more"),
                ));
            }
        }
    }
    None
}
pub(crate) fn property_choices(business: &crate::FederalBusiness) -> Vec<crate::RecordChoice> {
    business
        .assets
        .iter()
        .enumerate()
        .filter(|(_, asset)| {
            crate::asset_sale::research_property(&business.assets, asset.id).is_ok()
        })
        .map(|(index, asset)| crate::RecordChoice {
            id: asset.id,
            label: crate::Message::new(
                "RESEARCH_PROPERTY_CHOICE",
                alloc::vec![crate::BusinessAsset::display_arg(
                    &business.assets,
                    index,
                    business.vehicles.len()
                )],
            )
            .encode(),
        })
        .collect()
}

/// One Code-specific cost cohort linked to the existing accepted activity
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct OtherAmortizableCostPage {
    /// Cost or property described on Form 4562-B
    #[libertas_size(min = 1)]
    pub description: String,
    /// Documented qualifying business cost before statutory adjustments
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub cost: i64,
    /// Date costs were paid or incurred under the accounting method
    #[libertas_date_only]
    #[libertas_number(max = $crate::year_constants::TAX_YEAR_END_DATE)]
    pub incurred: u32,
    /// Original lawful amortization start date
    /// Supply this only when the original election itself establishes the start.
    /// Statutory midpoint/year starts and current expense need no extra date.
    /// [NullText]
    /// No separate original elected start date is supplied
    /// [SetText]
    /// The original elected start date was
    #[libertas_date_only]
    #[libertas_number(max = $crate::year_constants::TAX_YEAR_END_DATE)]
    #[libertas_default($null)]
    pub began: Option<u32>,
    /// Amortization allowed or allowable before the current year
    #[libertas_default(Unavailable)]
    pub prior_allowed: crate::SourceCents,
    /// Legal Code section and required election facts
    pub kind: OtherAmortizationKindPage,
    /// Disposition, retirement or termination of this cost cohort
    /// Enter historical events too. A current creative-property disposition can create ordinary income while its original amortization continues. Do not also include these proceeds in business receipts.
    /// [NullText]
    /// This cost cohort has not ended or been disposed of
    /// [SetText]
    /// This cost cohort ended through
    pub ended: Option<AmortizationEnd>,
}

/// Code-specific cohort; no arbitrary manual period for any expense
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum OtherAmortizationKindPage {
    /// Section 169 certified pollution-control facility
    Pollution { facts: PollutionControlCost },
    /// Section 174 or 174A research expenditures
    Research { facts: ResearchCostPage },
    /// Section 178 lease-acquisition cost
    Lease { facts: LeaseAcquisitionCost },
    /// Section 59(e) statutory preference write-off
    Preference { election: PreferenceWriteoff },
    /// Qualifying creative-property costs under Rev. Proc. 2004-36
    Creative { facts: CreativePropertyCost },
}

/// Annual research cohort; original dates determine law
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ResearchCostPage {
    /// Domestic or foreign research
    pub location: ResearchLocation,
    /// Original current-expense or amortization election
    pub treatment: ResearchTreatment,
    /// Actual property developed from this research, saved under Business equipment
    /// Save the actual property first, then select it here. Keep its basis, prior
    /// recovery and disposal facts on that property once. Mandatory section174
    /// expenditures are not part of its disposal basis and keep amortizing.
    /// [NullText]
    /// No separately recorded developed property is linked
    /// [SetText]
    /// The actual property is
    #[libertas_default($null)]
    #[libertas_enum_source("^<BusinessDetails>.research_property_choices")]
    pub property: Option<i64>,
}

impl From<OtherAmortizableCost> for OtherAmortizableCostPage {
    fn from(v: OtherAmortizableCost) -> Self {
        Self {
            description: v.description,
            cost: v.cost,
            incurred: v.incurred,
            began: v.began,
            prior_allowed: v.prior_allowed,
            kind: v.kind.into(),
            ended: v.ended,
        }
    }
}

impl From<OtherAmortizableCostPage> for OtherAmortizableCost {
    fn from(v: OtherAmortizableCostPage) -> Self {
        Self {
            description: v.description,
            cost: v.cost,
            incurred: v.incurred,
            began: v.began,
            prior_allowed: v.prior_allowed,
            kind: v.kind.into(),
            ended: v.ended,
        }
    }
}

impl From<OtherAmortizationKind> for OtherAmortizationKindPage {
    fn from(v: OtherAmortizationKind) -> Self {
        match v {
            OtherAmortizationKind::Pollution { facts } => Self::Pollution { facts },
            OtherAmortizationKind::Research { facts } => Self::Research { facts: facts.into() },
            OtherAmortizationKind::Lease { facts } => Self::Lease { facts },
            OtherAmortizationKind::Preference { election } => Self::Preference { election },
            OtherAmortizationKind::Creative { facts } => Self::Creative { facts },
        }
    }
}

impl From<OtherAmortizationKindPage> for OtherAmortizationKind {
    fn from(v: OtherAmortizationKindPage) -> Self {
        match v {
            OtherAmortizationKindPage::Pollution { facts } => Self::Pollution { facts },
            OtherAmortizationKindPage::Research { facts } => Self::Research { facts: facts.into() },
            OtherAmortizationKindPage::Lease { facts } => Self::Lease { facts },
            OtherAmortizationKindPage::Preference { election } => Self::Preference { election },
            OtherAmortizationKindPage::Creative { facts } => Self::Creative { facts },
        }
    }
}

impl From<ResearchCost> for ResearchCostPage {
    fn from(v: ResearchCost) -> Self {
        Self { location: v.location, treatment: v.treatment, property: v.property }
    }
}

impl From<ResearchCostPage> for ResearchCost {
    fn from(v: ResearchCostPage) -> Self {
        Self { location: v.location, treatment: v.treatment, property: v.property }
    }
}
