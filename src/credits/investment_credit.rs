// Coverage: Form 3468 — Supported (audited 2026-10-09 against the 2026 DRAFT f3468 created 7/27/26 and
//   the 2026 draft i3468 dated Sep 15, 2026; the final form and instructions must be reread). The
//   credit is a record on the property it belongs to: Asset.investment_credit on a Schedule C or F
//   asset row (BusinessAsset.investment_credit), RentalBuilding.rehabilitation on a rental
//   improvement row, and the K-1 list K1InvestmentProperty. Component availability for property
//   placed in service in 2026: Part VII rehabilitation credit (section 47) available, 20% of the
//   qualified rehabilitation expenditures allowed 4% a year for five years from the year the
//   certified historic building was placed in service (the pre-1936 10% credit ended with P.L.
//   115-97 section 13402); Part VI energy credit (section 48) only for construction begun before
//   2025 (solar, small wind, energy storage) and for geothermal heat pumps (construction before
//   2035), other energy property gets 0% for construction begun on or after June 16, 2025 (P.L.
//   119-21 section 70513(e)); Part V clean electricity investment credit (section 48E) for solar,
//   wind and storage begun after 2024, with a prohibited-foreign-entity (FEOC) confirmation for
//   construction begun after 2025. Excluded by name: Parts II to IV (sections 48A, 48B, 48C, 48D:
//   allocation-based projects and semiconductor facilities), cooperative lines (K-1 codes P to U),
//   the low-income communities bonus (Part I lines 11 to 11g), financing reductions (Part V lines
//   6a to 6e, Part VI lines 28a to 28e), interconnection property, expansions of existing
//   facilities, lessee elections, fuel cells, microturbines, combined heat and power, waste energy,
//   biogas, microgrids, section 45 facility elections, clean hydrogen, elective payment and
//   transfer. Lines: Part I 4 (construction date) and 7 to 10 (rates, one question), Part VII 1d to
//   1g, 1j and 3, Part V 1a to 1g, 2 to 5, 7, 8, 11 (storage 3a to 3g, 4), Part VI Sections B, F,
//   H, I and lines 27 to 32. The credit is a Form 3800 Part III line 4k (rehabilitation, specified),
//   4a (energy, specified) or 1v (clean electricity, not specified) amount, passive for a rental
//   improvement or a K-1 without material participation. The section 50(c) basis reduction (100%
//   of the rehabilitation credit determined, 50% of an energy credit) lowers the property's
//   depreciable basis (BusinessAsset::business_basis, RentalBuilding depreciation) and the K-1
//   owner's basis. The substantial-rehabilitation measuring period (24 or 60 months) must end in the
//   placed-in-service year: the client rule REHABILITATION_PERIOD_NOT_IN_SERVICE_YEAR on the asset
//   and the rental improvement states it as year(add_months(start, months) - 1) == year(placed), and
//   RehabilitationCredit::rule repeats it on save (the year_constants test pins the 24 and 60).
//   Evidence: src/tests/credits/investment_credit.rs fixtures A to E (K-1 rehabilitation credit limited by Form 3800,
//   direct rehabilitation credit with its basis reduction, year-three recapture with the Form 4797
//   additional depreciation, farm solar under section 48E and its section 48 twin, rental
//   rehabilitation credit with the Form 8582-CR special allowance), the K-1 import mapping, the
//   carryback year; 26 U.S.C. 46-50, 469(i); Reg. 1.47-7; i3468 (2026 draft), i4255, i3800, i8582cr.
// Coverage: Form 4255 — Supported (audited 2026-10-09 against the 2026 DRAFT f4255 Rev. December
//   2026 created 7/17/26 and its draft instructions dated Aug 26, 2026; the finals must be reread).
//   Recapture of the investment credit from the property's own disposal record
//   (BusinessAsset.disposal, FederalRental.sale): Part I rows 1g, 1j, 1m columns (a), (e), (f), (g),
//   (h), (i), (j), (r); Part II Sections A, B (B1, B2, B4 to B6) and D (D10 to D16, 19, 20), the
//   100/80/60/40/20% table by whole years from the first day of the month the property was placed
//   in service (year_constants.rs), Worksheet 2 freed credits entered per property, the property's
//   own carryforward cut by the same percentage (Form 3800 Part IV), the basis increase of section
//   50(c)(2) (100% rehabilitation, 50% energy) and the tax to Schedule 2 line 13a ("3468"), outside
//   the Form 3800 limits (section 50(a)(7)(C)). The net section 50(c) reduction counts as
//   depreciation on Form 4797 line 24 and as section 1250 additional depreciation (line 28a, 100%
//   ordinary) for buildings. Section C (increases in nonqualified nonrecourse financing), Part III
//   (hydrogen emissions tier) and Part IV (prevailing wage alteration or repair) are excluded: the
//   continuing records' confirmations name them, and a property placed before 2026 whose later
//   change is not confirmed is refused by name. K-1 recapture codes (box 20 code H, box 17 code G)
//   stay refused. The native direct-property tax retains its actual owner after the existing
//   community settlement, in whole dollars, for shared Form8379/Form3800 liability sources.
//   src/tests/credits/investment_credit.rs unit13_native_* and unit13_direct_recapture_* cover original fixture F,
//   both owners/claim selections, and actual Business/Rental Save, correction and restart.
//   This tax source does not establish final property-income attribution. Evidence: as Form 3468.

pub use crate::investment_credit_model::{
    CreditYear, EnergyProperty, EnergySection, InvestmentCredit, RehabilitationCredit,
};
pub(crate) use crate::investment_credit_source::energy_percent;
#[cfg(test)]
use crate::investment_credit_source::period_end;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::year_constants::{
    ENERGY_BASIS_REDUCTION_PERCENT, ENERGY_CONSTRUCTION_CUTOFF_DATE, INVESTMENT_RECAPTURE_PERCENT,
    REHABILITATION_BASIS_REDUCTION_PERCENT, REHABILITATION_CREDIT_PERCENT,
    REHABILITATION_PERIOD_YEARS, REHABILITATION_RATABLE_PERCENT,
};
use crate::{BusinessAsset, BusinessCredit};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Rates from the entity's Form 3468 Part I
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EnergyRate {
    /// 30% rate: line 7a or 8b checked
    #[libertas_llm_doc_field]
    pub increased_rate: bool,
    /// Domestic content bonus: line 9a or 9b checked
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub domestic_content: bool,
    /// Energy community bonus: line 10a or 10b checked
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub energy_community: bool,
}

/// Credit you calculated for this K-1 property on your own Form 3468
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OwnInvestmentCredit {
    /// Gross credit for this property, before Form 3800 limits
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Property basis and deductible expenses already reflect this credit
    /// Confirm the reductions required by your completed Form3468. The App separately applies this year's outside-basis reduction once; do not subtract that reduction from opening basis. No leaves this credit out of the estimate.
    pub reduced: bool,
}

/// Investment credit property on the K-1 (Form 3468)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum K1InvestmentProperty {
    /// Rehabilitation expenditures, rental real estate — K-1 (1065) box 15 code E or K-1 (1120-S) box 13 code E
    RehabilitationRental {
        /// Your share of qualified rehabilitation expenditures
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        expenditures: i64,
        /// Year the building was placed in service
        /// From the entity's statement. The credit is 4% of the expenditures in each of the five years starting that year.
        #[libertas_llm_doc_field(allow_empty = false)]
        // Tax-year source window: only the stated current or prior tax years apply.
        #[libertas_number(min = 2022, max = 2026)]
        placed_year: u32,
        /// Accepted property identity, assigned only by normal Save
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_default(0)]
        #[libertas_number(min = 0)]
        id: i64,
        /// Own Form 3468 credit
        /// Leave automatic when the App can calculate this property. Enter only this property's gross credit from your completed Form3468, before Form3800 limits.
        /// [NullText]
        /// Calculate the credit from this property's facts
        /// [SetText]
        /// My own Form 3468 credit is
        #[libertas_default($null)]
        #[libertas_hidden]
        own_form: Option<OwnInvestmentCredit>,
    },
    /// Rehabilitation expenditures, other activities — K-1 (1065) box 20 code D or K-1 (1120-S) box 17 code C
    RehabilitationOther {
        /// Your share of qualified rehabilitation expenditures
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        expenditures: i64,
        /// Year the building was placed in service
        #[libertas_llm_doc_field(allow_empty = false)]
        // Tax-year source window: only the stated current or prior tax years apply.
        #[libertas_number(min = 2022, max = 2026)]
        placed_year: u32,
        /// Accepted property identity, assigned only by normal Save
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_default(0)]
        #[libertas_number(min = 0)]
        id: i64,
        /// Own Form 3468 credit
        /// Leave automatic when the App can calculate this property. Enter only this property's gross credit from your completed Form3468, before Form3800 limits.
        /// [NullText]
        /// Calculate the credit from this property's facts
        /// [SetText]
        /// My own Form 3468 credit is
        #[libertas_default($null)]
        #[libertas_hidden]
        own_form: Option<OwnInvestmentCredit>,
    },
    /// Basis of energy property placed in service in 2026 — K-1 (1065) box 20 code E or K-1 (1120-S) box 17 code D
    Energy {
        /// Your share of the basis
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        basis: i64,
        /// Rates from the entity's Form 3468 Part I
        #[libertas_llm_doc_field]
        rate: EnergyRate,
        /// Accepted property identity, assigned only by normal Save
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_default(0)]
        #[libertas_number(min = 0)]
        id: i64,
        /// Own Form 3468 credit
        /// Leave automatic when the App can calculate this property. Enter only this property's gross credit from your completed Form3468, before Form3800 limits.
        /// [NullText]
        /// Calculate the credit from this property's facts
        /// [SetText]
        /// My own Form 3468 credit is
        #[libertas_default($null)]
        #[libertas_hidden]
        own_form: Option<OwnInvestmentCredit>,
    },
    /// Clean electricity investment property placed in service in 2026 — K-1 (1065) box 20 code AV or K-1 (1120-S) box 17 code AV
    CleanElectricity {
        /// Your share of the basis
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        basis: i64,
        /// Energy storage technology
        /// Form 3468 Part V Section B instead of Section A.
        #[libertas_llm_doc_field]
        #[libertas_default(false)]
        storage: bool,
        /// Rates from the entity's Form 3468 Part I
        #[libertas_llm_doc_field]
        rate: EnergyRate,
        /// Accepted property identity, assigned only by normal Save
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_default(0)]
        #[libertas_number(min = 0)]
        id: i64,
        /// Own Form 3468 credit
        /// Leave automatic when the App can calculate this property. Enter only this property's gross credit from your completed Form3468, before Form3800 limits.
        /// [NullText]
        /// Calculate the credit from this property's facts
        /// [SetText]
        /// My own Form 3468 credit is
        #[libertas_default($null)]
        #[libertas_hidden]
        own_form: Option<OwnInvestmentCredit>,
    },
}

// Whole years from the first day of the month `placed` to `ceased` (Reg.
// 1.47-1(c); i4255 line 10 and line 12 "don't enter partial years").
pub(crate) fn full_years(placed: u32, ceased: u32) -> u32 {
    let (placed_year, placed_month) = (placed / 10000, placed / 100 % 100);
    let (ceased_year, ceased_month) = (ceased / 10000, ceased / 100 % 100);
    (ceased_year.saturating_sub(placed_year)).saturating_sub(u32::from(ceased_month < placed_month))
}

// The percent of the credit recaptured after `years` full years (Form 4255
// line 15).
pub(crate) fn recapture_percent(years: u32) -> i64 {
    INVESTMENT_RECAPTURE_PERCENT[(years as usize).min(INVESTMENT_RECAPTURE_PERCENT.len() - 1)]
}

// The first day of the five-year rehabilitation period and its last year.
fn in_period(placed: u32, year: u32) -> bool {
    let first = placed / 10000;
    year >= first && year < first + REHABILITATION_PERIOD_YEARS
}

// ---- Form 4255 Section D ------------------------------------------------------------------------

/// Form 4255 Section D for one property that was disposed of, or ceased to be
/// investment credit property, in 2026. Whole dollars except the basis
/// increase, in cents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Recapture {
    pub energy: bool,
    // Line D12: whole years.
    pub years: u32,
    // Line D15.
    pub percent: i64,
    // Line B6: the credit used on earlier returns.
    pub claimed: i64,
    // Line D13: other credits that would have been allowed.
    pub freed: i64,
    // Lines D14, D16 and 20: the recapture added to the tax.
    pub tax: i64,
    // Column (i): the part of the property's carryforward the recapture cuts.
    pub carryover: i64,
    // Section 50(c)(2): the basis increase, cents.
    pub basis_increase: i64,
}

pub(crate) fn recapture(
    placed: u32,
    ceased: u32,
    used: i64,
    carried: i64,
    freed: i64,
    energy: bool,
) -> Recapture {
    let years = full_years(placed, ceased);
    let percent = recapture_percent(years);
    let reduction = if energy {
        ENERGY_BASIS_REDUCTION_PERCENT
    } else {
        REHABILITATION_BASIS_REDUCTION_PERCENT
    };
    let amounts =
        crate::kernel::recapture::investment_credit(used, carried, freed, percent, reduction);
    Recapture {
        energy,
        years,
        percent,
        claimed: used,
        freed,
        tax: amounts.tax_dollars,
        carryover: amounts.carryover_reduction_dollars,
        basis_increase: amounts.basis_increase_cents,
    }
}

// ---- rehabilitation credit ---------------------------------------------------------------------

impl RehabilitationCredit {
    // Credit used and carried on earlier returns, whole dollars.
    pub(crate) fn used(&self) -> i64 {
        self.earlier.iter().map(|row| row.used).sum()
    }

    pub(crate) fn carried(&self) -> i64 {
        self.earlier.iter().map(|row| row.carried).sum()
    }

    // The 2026 ratable share (4% of the expenditures) of a building placed in
    // service on `placed`, less the part a disposition in 2026 recaptured
    // (Reg. 1.47-7(e)(3) Example 3), whole dollars. Zero outside the
    // five-year period or when the rehabilitation is not confirmed.
    pub(crate) fn current_year_share(&self, qre: i64, placed: u32, percent: i64) -> i64 {
        if !self.confirmed || !in_period(placed, 2026) {
            return 0;
        }
        dollars(fraction(qre, REHABILITATION_RATABLE_PERCENT * (100 - percent), 10_000))
    }

    // Form 4255 for a disposal on `ceased`.
    pub(crate) fn recapture(&self, placed: u32, ceased: u32) -> Recapture {
        recapture(
            placed,
            ceased,
            dollars(self.used()),
            dollars(self.carried()),
            dollars(self.freed),
            false,
        )
    }
}

impl EnergyRate {
    pub(crate) fn percent(&self) -> i64 {
        energy_percent(self.increased_rate, self.domestic_content, self.energy_community)
    }
}

// The section of a property whose construction began on `began`: a geothermal
// heat pump and construction before 2025 are the energy credit (Part VI),
// later construction the clean electricity investment credit (Part V).
pub(crate) fn energy_section(kind: EnergyProperty, began: u32) -> BusinessCredit {
    if kind == EnergyProperty::GeothermalHeatPump || began < ENERGY_CONSTRUCTION_CUTOFF_DATE {
        BusinessCredit::Energy
    } else {
        BusinessCredit::CleanElectricity
    }
}

// ---- the credit on a business or farm asset ------------------------------------------------------

impl BusinessAsset {
    // The cost the credit rests on: the business share of the cost, cents
    // (the qualified rehabilitation expenditures of a building row).
    pub(crate) fn credit_cost(&self) -> i64 {
        self.credit_cost_on(self.cost)
    }

    // The Form 3800 credit the property's 2026 credit belongs to, or the
    // credit an earlier-year history belongs to.
    pub(crate) fn investment_section(&self) -> Option<BusinessCredit> {
        match &self.investment_credit {
            InvestmentCredit::No => None,
            InvestmentCredit::Rehabilitation(_) => Some(BusinessCredit::Rehabilitation),
            InvestmentCredit::NewEnergy { kind, construction_began, .. } => {
                Some(energy_section(*kind, *construction_began))
            }
            InvestmentCredit::EarlierEnergy { section, .. } => Some(match section {
                EnergySection::Energy => BusinessCredit::Energy,
                EnergySection::CleanElectricity => BusinessCredit::CleanElectricity,
            }),
        }
    }

    // The 2026 recapture of the property, when it was disposed of or ceased to
    // be investment credit property in 2026.
    pub(crate) fn investment_recapture(&self) -> Option<Recapture> {
        let ceased = self.disposal_date()?;
        match &self.investment_credit {
            InvestmentCredit::Rehabilitation(claim) => {
                Some(claim.recapture(self.placed, ceased)).filter(|r| r.percent > 0)
            }
            InvestmentCredit::EarlierEnergy { used, carried, freed, .. } => Some(recapture(
                self.placed,
                ceased,
                dollars(*used),
                dollars(*carried),
                dollars(*freed),
                true,
            ))
            .filter(|r| r.percent > 0),
            _ => None,
        }
    }

    // The recapture percent of a disposal in 2026 (zero without one).
    fn current_year_recapture_percent(&self) -> i64 {
        self.investment_recapture().map_or(0, |r| r.percent)
    }

    // The 2026 Form 3800 credit of the property, whole dollars.
    #[cfg(test)]
    pub(crate) fn current_year_investment_credit(&self) -> i64 {
        self.current_year_investment_credit_on(self.cost)
    }
    pub(crate) fn current_year_investment_credit_on(&self, cost: i64) -> i64 {
        match &self.investment_credit {
            InvestmentCredit::Rehabilitation(claim) => claim.current_year_share(
                self.credit_cost_on(cost),
                self.placed,
                self.current_year_recapture_percent(),
            ),
            InvestmentCredit::NewEnergy { confirmed, .. } if *confirmed => {
                self.energy_credit_determined_on(cost)
            }
            _ => 0,
        }
    }

    // The section 50(c) reduction of the property's basis, cents: the whole
    // rehabilitation credit determined, half of an energy credit.
    pub(crate) fn investment_basis_reduction(&self) -> i64 {
        self.investment_basis_reduction_on(self.cost)
    }

    // The basis increase a recapture in 2026 brings back, cents.
    pub(crate) fn investment_basis_increase(&self) -> i64 {
        self.investment_recapture().map_or(0, |r| r.basis_increase)
    }

    // The straight-line depreciation of a 39-year building on its cost before
    // the section 50(c) reduction, from the year placed in service through the
    // disposal in 2026 (the mid-month convention), cents: Form 4797 line 28a
    // subtracts it from the depreciation allowed (i4797 line 26a).
    pub(crate) fn straight_line_on_cost(&self, disposed: u32) -> i64 {
        use crate::kernel::cost_recovery::historical::{
            RentalBuildingRecovery, rental_building_amount,
        };
        let basis = self.credit_cost();
        let half_months = 2 * (disposed / 100 % 100).clamp(1, 12) - 1;
        (self.placed / 10000..=2026)
            .map(|year| {
                let window = if year == 2026 { half_months } else { 24 };
                rental_building_amount(
                    RentalBuildingRecovery::Commercial39,
                    basis,
                    self.placed,
                    year,
                    window,
                    None,
                )
                .map_or(0, |amount| amount.rounded_cents())
            })
            .sum()
    }
}

// ---- Form 3468 and Form 4255 rows ----------------------------------------------------------------

const F3468: &str = "https://www.irs.gov/pub/irs-dft/i3468--dft.pdf";
const F4255: &str = "https://www.irs.gov/pub/irs-dft/i4255--dft.pdf";

// The printed line of the credit base and of the credit for a section. Part
// VII holds the rehabilitation credit; Part V the clean electricity
// investment credit (Section A facilities, Section B storage); Part VI the
// energy credit (Section B solar, F small wind, H geothermal heat pumps, I
// storage).
fn printed_lines(
    section: BusinessCredit,
    kind: Option<EnergyProperty>,
) -> (&'static str, &'static str) {
    match (section, kind) {
        (BusinessCredit::Rehabilitation, _) => ("1g", "3"),
        (BusinessCredit::CleanElectricity, Some(EnergyProperty::Storage)) => ("3a", "11"),
        (BusinessCredit::CleanElectricity, _) => ("1a", "11"),
        (_, Some(EnergyProperty::Wind)) => ("11d", "32"),
        (_, Some(EnergyProperty::GeothermalHeatPump)) => ("15a", "32"),
        (_, Some(EnergyProperty::Storage)) => ("17a", "32"),
        _ => ("3a", "32"),
    }
}

// The Form 4255 Part I row of a section.
fn recapture_row(section: BusinessCredit) -> &'static str {
    match section {
        BusinessCredit::Rehabilitation => "1m",
        BusinessCredit::CleanElectricity => "1g",
        _ => "1j",
    }
}

// Form 3468 rows of one property: the credit base and the 2026 credit, whole
// dollars, and the section 50(c) reduction of its basis.
pub(crate) struct PropertyRows<'a> {
    pub prefix: &'a str,
    pub input: &'a str,
    pub section: BusinessCredit,
    pub kind: Option<EnergyProperty>,
    // Cents: the qualified rehabilitation expenditures after a recapture, or
    // the credit base of energy property.
    pub basis: i64,
    // Whole dollars.
    pub credit: i64,
    pub credit_explanation: Option<crate::Message>,
    // Cents: the gross section 50(c) reduction of the basis.
    pub reduction: i64,
    // The rehabilitation record of lines 1e and 1f, and the year of the
    // five-year credit period the 2026 credit is.
    pub record: Option<&'a RehabilitationCredit>,
    pub period_year: u32,
    pub percent: Option<i64>,
    pub recapture: Option<Recapture>,
}

impl PropertyRows<'_> {
    pub(crate) fn report(&self, r: &mut crate::FederalResult) {
        use crate::line_emission::{form_count, form_line, line_cents};
        use crate::{FormLine, TaxForm};
        let prefix = self.prefix;
        let input = [self.input];
        let (basis_line, credit_line) = printed_lines(self.section, self.kind);
        if let Some(claim) = self.record {
            form_line(
                r,
                FormLine::new(TaxForm::Form3468, "1e"),
                &alloc::format!("{prefix}_form3468_building_basis"),
                dollars(claim.building_basis),
                &input,
                F3468,
                msg!("EXPLAIN_FORM3468_BUILDING_BASIS"),
            );
            form_line(
                r,
                FormLine::new(TaxForm::Form3468, "1f"),
                &alloc::format!("{prefix}_form3468_period_expenditures"),
                dollars(claim.period_expenditures),
                &input,
                F3468,
                msg!("EXPLAIN_FORM3468_PERIOD_EXPENDITURES"),
            );
        }
        form_line(
            r,
            FormLine::new(TaxForm::Form3468, basis_line),
            &alloc::format!("{prefix}_form3468_basis"),
            dollars(self.basis),
            &input,
            F3468,
            if self.section == BusinessCredit::Rehabilitation {
                msg!("EXPLAIN_FORM3468_QUALIFIED_EXPENDITURES")
            } else {
                msg!("EXPLAIN_FORM3468_CREDIT_BASE")
            },
        );
        form_line(
            r,
            FormLine::new(TaxForm::Form3468, credit_line),
            &alloc::format!("{prefix}_form3468_credit"),
            self.credit,
            &input,
            F3468,
            self.credit_explanation.clone().unwrap_or_else(|| match (self.section, self.percent) {
                (BusinessCredit::Rehabilitation, _) => crate::Message::new(
                    "EXPLAIN_FORM3468_REHABILITATION_CREDIT",
                    alloc::vec![crate::MessageArg::Number(i64::from(self.period_year))],
                ),
                (_, Some(percent)) => crate::Message::new(
                    "EXPLAIN_FORM3468_ENERGY_CREDIT",
                    alloc::vec![crate::MessageArg::Number(percent)],
                ),
                (_, None) => msg!("EXPLAIN_FORM3468_EARLIER_ENERGY_CREDIT"),
            }),
        );
        line_cents(
            r,
            &alloc::format!("{prefix}_basis_reduction"),
            self.reduction,
            &input,
            F3468,
            msg!("EXPLAIN_INVESTMENT_CREDIT_BASIS_REDUCTION"),
        );
        let Some(recapture) = self.recapture else {
            return;
        };
        let row = recapture_row(self.section);
        for (id, printed, amount, count, explanation) in [
            ("claimed", "B6".into(), recapture.claimed, false, msg!("EXPLAIN_FORM4255_CLAIMED")),
            (
                "years",
                "D12".into(),
                i64::from(recapture.years),
                true,
                msg!("EXPLAIN_FORM4255_YEARS"),
            ),
            ("freed", "D13".into(), recapture.freed, false, msg!("EXPLAIN_FORM4255_FREED")),
            ("percent", "D15".into(), recapture.percent, true, msg!("EXPLAIN_FORM4255_PERCENT")),
            ("recapture", "D16".into(), recapture.tax, false, msg!("EXPLAIN_FORM4255_RECAPTURE")),
            (
                "carryover",
                alloc::format!("{row}(i)"),
                recapture.carryover,
                false,
                msg!("EXPLAIN_FORM4255_CARRYOVER"),
            ),
            ("tax", alloc::format!("{row}(r)"), recapture.tax, false, msg!("EXPLAIN_FORM4255_TAX")),
        ] {
            let printed: alloc::string::String = printed;
            let form_line_ref = FormLine::new(TaxForm::Form4255, &printed);
            let id = alloc::format!("{prefix}_form4255_{id}");
            if count {
                form_count(r, form_line_ref, &id, amount, &input, F4255, explanation);
            } else {
                form_line(r, form_line_ref, &id, amount, &input, F4255, explanation);
            }
        }
    }
}

impl BusinessAsset {
    // The Form 3468 and Form 4255 rows of the property's investment credit.
    pub(crate) fn report_investment_credit(
        &self,
        id: i64,
        index: usize,
        cost: i64,
        r: &mut crate::FederalResult,
    ) {
        let Some(section) = self.investment_section() else {
            return;
        };
        let prefix = alloc::format!("business_{id}_asset_{index}");
        let recapture = self.investment_recapture();
        let (kind, percent, record, basis, credit) = match &self.investment_credit {
            InvestmentCredit::Rehabilitation(claim) => (
                None,
                None,
                Some(claim),
                fraction(
                    self.credit_cost_on(cost),
                    100 - self.current_year_recapture_percent(),
                    100,
                ),
                self.current_year_investment_credit_on(cost),
            ),
            InvestmentCredit::NewEnergy {
                kind,
                increased_rate,
                domestic_content,
                energy_community,
                ..
            } => (
                Some(*kind),
                Some(energy_percent(*increased_rate, *domestic_content, *energy_community)),
                None,
                positive(self.credit_cost_on(cost) - self.elected179()),
                self.current_year_investment_credit_on(cost),
            ),
            _ => (None, None, None, 0, 0),
        };
        // A property whose credit was taken before 2026 has no 2026 rows but
        // its basis reduction and any recapture.
        let current = credit > 0;
        if !current && recapture.is_none() {
            return;
        }
        PropertyRows {
            prefix: &prefix,
            input: "income.business.assets.investment_credit",
            section,
            kind,
            basis,
            credit,
            credit_explanation: None,
            reduction: self.investment_basis_reduction_on(cost),
            record: record.filter(|_| current),
            period_year: 2026 + 1 - self.placed / 10000,
            percent,
            recapture,
        }
        .report(r);
    }
}

// ---- the credit on a rental improvement --------------------------------------------------------

impl crate::RentalBuilding {
    // The qualified rehabilitation expenditures of an improvement row: its
    // cost (the land is zero), cents.
    pub(crate) fn credit_cost(&self) -> i64 {
        positive(self.cost - self.land)
    }

    // The section 50(c) reduction of the improvement's depreciable basis,
    // cents: the whole credit determined.
    pub(crate) fn rehabilitation_reduction(&self) -> i64 {
        if self.rehabilitation.is_some() {
            fraction(
                RehabilitationCredit::determined(self.credit_cost()),
                REHABILITATION_BASIS_REDUCTION_PERCENT,
                100,
            )
        } else {
            self.credit_reduction
        }
    }

    // Form 4255 when the whole property is sold on `sold`.
    pub(crate) fn rehabilitation_recapture(&self, sold: Option<u32>) -> Option<Recapture> {
        let (claim, ceased) = (self.rehabilitation.as_ref()?, sold?);
        Some(claim.recapture(self.in_service, ceased)).filter(|r| r.percent > 0)
    }

    // The 2026 Form 3800 credit of the improvement, whole dollars.
    pub(crate) fn current_year_rehabilitation_credit(&self, sold: Option<u32>) -> i64 {
        let percent = self.rehabilitation_recapture(sold).map_or(0, |r| r.percent);
        self.rehabilitation.as_ref().map_or(0, |claim| {
            claim.current_year_share(self.credit_cost(), self.in_service, percent)
        })
    }

    // The basis increase a sale in 2026 brings back, cents.
    pub(crate) fn rehabilitation_basis_increase(&self, sold: Option<u32>) -> i64 {
        self.rehabilitation_recapture(sold).map_or(0, |r| r.basis_increase)
    }

    // The net section 50(c) reduction counted as depreciation allowed on Form
    // 4797 line 24, cents: the reduction less the basis a recapture restores.
    pub(crate) fn rehabilitation_net_reduction(&self, sold: Option<u32>) -> i64 {
        self.rehabilitation_reduction() - self.rehabilitation_basis_increase(sold)
    }
}

// ---- the claims of a return --------------------------------------------------------------------

// A property's earlier-year credit still carried to 2026: it joins the Form
// 3800 carryforwards itself (the person enters it once, on the property), and
// a recapture cuts it by the percentage of Form 4255 column (i).
pub(crate) struct Carried {
    pub credit: BusinessCredit,
    pub year: u32,
    // A rental improvement's carryforward is a Form 8582-CR Worksheet 2
    // prior-year unallowed amount while the rental is passive.
    pub passive: Option<crate::PassiveWorksheet>,
    pub amount: i64,
    pub cut: i64,
}

// What the return's business, farm and rental investment credits contribute:
// the 2026 Form 3800 entries, the earlier-year credit carried to 2026, and the
// recapture tax of Schedule 2 line 13a, whole dollars.
#[derive(Default)]
pub(crate) struct Claims {
    pub entries: Vec<crate::general_business_credit::CreditEntry>,
    pub carried: Vec<Carried>,
    pub recapture_tax: i64,
    // Final native Form4255 tax, whole dollars, retained at the same admitted
    // property calculation for its actual settled source owner. No rerounding.
    pub recapture_by_owner: [i64; 2],
}

impl Claims {
    // The Form 3800 Part IV carryforward entries the properties carry to 2026,
    // after the recapture cut.
    pub(crate) fn carryforward_entries(&self) -> Vec<crate::general_business_credit::CreditEntry> {
        self.carried
            .iter()
            .map(|claim| crate::general_business_credit::CreditEntry {
                source_owner: None,
                conditional_origin: None,
                transfer: 0,
                elective: 0,
                activity: None,
                filer: crate::CarryFiler::SameFilers,
                credit: claim.credit,
                year: claim.year,
                amount: claim.amount - claim.cut,
                passive: claim.passive,
            })
            .collect()
    }
}

#[cfg(test)]
const ACTIVE_ACTIVITY: &str = "Form 3468 for business property needs an active U.S. business or farm in which you materially participate.";

// The investment credits of the business and farm assets and the rental
// improvements: refusals, Form 3468 and Form 4255 rows and the Form 3800
// entries of 2026. Schedule K-1 items are collected where the K-1s are.
#[cfg(test)]
pub(crate) fn claims(d: &crate::FederalDraft, r: &mut crate::FederalResult) -> Claims {
    let source_claims = crate::related_forms::effective_claimants(d);
    claims_with_claims(crate::related_forms::RelatedFormContext::original(d, &source_claims), r)
}

pub(crate) fn claims_with_claims(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    r: &mut crate::FederalResult,
) -> Claims {
    let d = form_context.draft;
    let source_claims = form_context.claims;
    let direct_participation = form_context.participation();
    use crate::{FederalIncome, PassiveWorksheet};
    let mut out = Claims::default();
    for entry in &crate::community::entries(d) {
        match &entry.income {
            FederalIncome::Business { data } => {
                let context = crate::interview::income_issue_context(entry);
                for (index, asset) in data.assets.iter().enumerate() {
                    let Some(section) = asset.investment_section() else {
                        continue;
                    };
                    let item = context.item("business_assets", index);
                    let active = matches!(
                        data.activity,
                        crate::BusinessActivity::Ordinary { .. }
                            | crate::BusinessActivity::Farming { .. }
                    ) && data.source == crate::BusinessSource::UnitedStates
                        && (direct_participation.get(entry.id)
                            == crate::participation::Participation::Yes)
                        && asset.valid();
                    crate::require_localized!(
                        r,
                        active,
                        &item.clone().manual(),
                        "Investment credit",
                        "ISSUE_FORM_3468_NEEDS_AN_ACTIVE_U_S_SCHEDULE_C_OR_F_ACTIVITY",
                        ACTIVE_ACTIVITY,
                    );
                    let recapture = asset.investment_recapture();
                    crate::require_localized!(
                        r,
                        asset.history_confirmed() || recapture.is_none(),
                        &item.clone().manual(),
                        "Investment credit",
                        "ISSUE_INVESTMENT_CREDIT_RECAPTURE_NEEDS_A_CONFIRMED_HISTORY",
                        RECAPTURE_HISTORY,
                    );
                    crate::require_localized!(
                        r,
                        !matches!(
                            asset.investment_credit,
                            InvestmentCredit::EarlierEnergy { unchanged: false, .. }
                        ),
                        &item.clone().manual(),
                        "Investment credit",
                        "ISSUE_INVESTMENT_CREDIT_RECAPTURE_NEEDS_A_CONFIRMED_HISTORY",
                        RECAPTURE_HISTORY,
                    );
                    if !active {
                        continue;
                    }
                    let cost = match crate::cooperative_basis::current_cost_with_claims(
                        d,
                        entry.id,
                        asset,
                        source_claims,
                    ) {
                        Ok(cost) => cost,
                        Err(issue) => {
                            crate::section179_return::Issue::Prepared(issue).report(r);
                            continue;
                        }
                    };
                    asset.report_investment_credit(entry.id, index, cost, r);
                    let credit = asset.current_year_investment_credit_on(cost);
                    if credit > 0 {
                        out.entries.push(crate::general_business_credit::CreditEntry {
                            source_owner: Some(entry.owner.owned()),
                            conditional_origin: None,
                            transfer: 0,
                            elective: 0,
                            activity: None,
                            filer: crate::CarryFiler::SameFilers,
                            credit: section,
                            year: 2026,
                            amount: credit,
                            passive: None,
                        });
                    }
                    let recapture_tax = recapture.map_or(0, |v| v.tax);
                    out.recapture_tax += recapture_tax;
                    out.recapture_by_owner
                        [usize::from(entry.owner == crate::DocumentOwner::Spouses)] +=
                        recapture_tax;
                    for (year, amount, cut) in asset.carried_history() {
                        out.carried.push(Carried {
                            credit: section,
                            year,
                            passive: None,
                            amount,
                            cut: cut.map_or(0, |percent| fraction(amount, percent, 100)),
                        });
                    }
                }
            }
            FederalIncome::Rental { data } => {
                let context = crate::interview::income_issue_context(entry);
                let sold = match &data.sale {
                    crate::RentalSale::Sold { date, .. } => Some(*date),
                    _ => None,
                };
                for (index, building) in data.buildings.iter().enumerate() {
                    let Some(claim) = &building.rehabilitation else {
                        continue;
                    };
                    let item = context.item("rental_buildings", index);
                    let in_scope = claim.valid()
                        && building.land == 0
                        && building.in_service >= 20180101
                        && claim.rule(building.in_service, building.credit_cost()).is_none()
                        && matches!(data.personal_use, crate::RentalPersonalUse::No)
                        && !matches!(data.sale, crate::RentalSale::Exchanged(_));
                    crate::require_localized!(
                        r,
                        in_scope,
                        &item.clone().manual(),
                        "Rehabilitation credit",
                        "ISSUE_RENTAL_REHABILITATION_CREDIT_SCOPE",
                        RENTAL_SCOPE,
                    );
                    let recapture = building.rehabilitation_recapture(sold);
                    crate::require_localized!(
                        r,
                        claim.confirmed || recapture.is_none() || building.in_service >= 20260101,
                        &item.clone().manual(),
                        "Investment credit",
                        "ISSUE_INVESTMENT_CREDIT_RECAPTURE_NEEDS_A_CONFIRMED_HISTORY",
                        RECAPTURE_HISTORY,
                    );
                    if !in_scope {
                        continue;
                    }
                    let credit = building.current_year_rehabilitation_credit(sold);
                    let percent = recapture.map_or(0, |v| v.percent);
                    let prefix = alloc::format!("rental_{}_building_{index}", entry.id);
                    if credit > 0 || recapture.is_some() {
                        PropertyRows {
                            prefix: &prefix,
                            input: "income.rental.buildings.rehabilitation",
                            section: BusinessCredit::Rehabilitation,
                            kind: None,
                            basis: fraction(building.credit_cost(), 100 - percent, 100),
                            credit,
                            credit_explanation: None,
                            reduction: building.rehabilitation_reduction(),
                            record: (credit > 0).then_some(claim),
                            period_year: 2026 + 1 - building.in_service / 10000,
                            percent: None,
                            recapture,
                        }
                        .report(r);
                    }
                    if credit > 0 {
                        out.entries.push(crate::general_business_credit::CreditEntry {
                            source_owner: Some(entry.owner.owned()),
                            conditional_origin: None,
                            transfer: 0,
                            elective: 0,
                            activity: None,
                            filer: crate::CarryFiler::SameFilers,
                            credit: BusinessCredit::Rehabilitation,
                            year: 2026,
                            amount: credit,
                            passive: (!(data.participation.professional()
                                && d.professional(entry.owner.owned()).is_some()))
                            .then_some(PassiveWorksheet::Rehabilitation),
                        });
                    }
                    let recapture_tax = recapture.map_or(0, |v| v.tax);
                    out.recapture_tax += recapture_tax;
                    out.recapture_by_owner
                        [usize::from(entry.owner == crate::DocumentOwner::Spouses)] +=
                        recapture_tax;
                    for row in &claim.earlier {
                        if row.carried > 0 {
                            out.carried.push(Carried {
                                credit: BusinessCredit::Rehabilitation,
                                year: row.year,
                                passive: (!(data.participation.professional()
                                    && d.professional(entry.owner.owned()).is_some()))
                                .then_some(PassiveWorksheet::Rehabilitation),
                                amount: dollars(row.carried),
                                cut: fraction(dollars(row.carried), percent, 100),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }
    out
}

#[cfg(test)]
const RECAPTURE_HISTORY: &str = "Investment credit property sold or taken out of use within five years needs a confirmed credit history; recapture from increased nonrecourse financing or a later prevailing-wage failure (Form 4255 Parts II Section C and IV) is not calculated.";
#[cfg(test)]
const RENTAL_SCOPE: &str = "A rental improvement's rehabilitation credit needs a rental with no personal use, no like-kind exchange in 2026, no land in the row and a measuring period that ends in the year it was available to rent.";

impl BusinessAsset {
    // Whether the credit history is confirmed: a continuing rehabilitation
    // record answered No cannot support a recapture.
    pub(crate) fn history_confirmed(&self) -> bool {
        match &self.investment_credit {
            InvestmentCredit::Rehabilitation(claim) => {
                claim.confirmed || self.placed / 10000 >= 2026
            }
            _ => true,
        }
    }

    // The earlier-year credit still carried to 2026, by the year it arose
    // (an energy credit arises in the year the property was placed in
    // service), whole dollars, with the recapture percent that cuts it.
    pub(crate) fn carried_history(&self) -> Vec<(u32, i64, Option<i64>)> {
        let percent = self.investment_recapture().map(|r| r.percent);
        match &self.investment_credit {
            InvestmentCredit::Rehabilitation(claim) => claim
                .earlier
                .iter()
                .filter(|row| row.carried > 0)
                .map(|row| (row.year, dollars(row.carried), percent))
                .collect(),
            InvestmentCredit::EarlierEnergy { carried, .. } if *carried > 0 => {
                alloc::vec![(self.placed / 10000, dollars(*carried), percent)]
            }
            _ => Vec::new(),
        }
    }
}

// ---- the K-1 list ------------------------------------------------------------------------------

impl K1InvestmentProperty {
    pub(crate) fn id(&self) -> i64 {
        match self {
            Self::RehabilitationRental { id, .. }
            | Self::RehabilitationOther { id, .. }
            | Self::Energy { id, .. }
            | Self::CleanElectricity { id, .. } => *id,
        }
    }
    pub(crate) fn id_mut(&mut self) -> &mut i64 {
        match self {
            Self::RehabilitationRental { id, .. }
            | Self::RehabilitationOther { id, .. }
            | Self::Energy { id, .. }
            | Self::CleanElectricity { id, .. } => id,
        }
    }
    pub(crate) fn own_form_mut(&mut self) -> &mut Option<OwnInvestmentCredit> {
        match self {
            Self::RehabilitationRental { own_form, .. }
            | Self::RehabilitationOther { own_form, .. }
            | Self::Energy { own_form, .. }
            | Self::CleanElectricity { own_form, .. } => own_form,
        }
    }
    pub(crate) fn own_form(&self) -> Option<&OwnInvestmentCredit> {
        match self {
            Self::RehabilitationRental { own_form, .. }
            | Self::RehabilitationOther { own_form, .. }
            | Self::Energy { own_form, .. }
            | Self::CleanElectricity { own_form, .. } => own_form.as_ref(),
        }
    }
    pub(crate) fn valid(&self) -> bool {
        if !(0..=100_000_000_000).contains(&self.id())
            || self.own_form().is_some_and(|v| !(1..=100_000_000_000).contains(&v.amount))
        {
            return false;
        }
        match self {
            Self::RehabilitationRental { expenditures, placed_year, .. }
            | Self::RehabilitationOther { expenditures, placed_year, .. } => {
                (1..=100_000_000_000).contains(expenditures) && (2022..=2026).contains(placed_year)
            }
            Self::Energy { basis, .. } | Self::CleanElectricity { basis, .. } => {
                (1..=100_000_000_000).contains(basis)
            }
        }
    }

    pub(crate) fn valid_list(items: &[Self]) -> bool {
        items.len() <= 10 && items.iter().all(Self::valid)
    }

    // The Form 3800 credit the item belongs to.
    pub(crate) fn section(&self) -> BusinessCredit {
        match self {
            Self::RehabilitationRental { .. } | Self::RehabilitationOther { .. } => {
                BusinessCredit::Rehabilitation
            }
            Self::Energy { .. } => BusinessCredit::Energy,
            Self::CleanElectricity { .. } => BusinessCredit::CleanElectricity,
        }
    }

    // A rehabilitation credit of the partnership's rental real estate
    // activity is a Worksheet 2 passive credit (section 469(i)(6)(B)).
    pub(crate) fn rental_real_estate(&self) -> bool {
        matches!(self, Self::RehabilitationRental { .. })
    }

    // The item's 2026 credit, whole dollars: the ratable 4% of rehabilitation
    // expenditures within the five-year period, the energy credit determined
    // on the basis.
    pub(crate) fn current_year_credit(&self) -> i64 {
        if let Some(own) = self.own_form() {
            return if own.reduced { dollars(own.amount) } else { 0 };
        }
        match self {
            Self::RehabilitationRental { expenditures, placed_year, .. }
            | Self::RehabilitationOther { expenditures, placed_year, .. } => {
                if in_period(*placed_year * 10000, 2026) {
                    dollars(fraction(*expenditures, REHABILITATION_RATABLE_PERCENT, 100))
                } else {
                    0
                }
            }
            Self::Energy { basis, rate, .. } | Self::CleanElectricity { basis, rate, .. } => {
                dollars(fraction(*basis, rate.percent(), 100))
            }
        }
    }

    // The section 50(c)(5) reduction of the owner's basis in the interest,
    // cents, in the year the property was placed in service: all of the
    // rehabilitation credit determined, half of the energy credit.
    pub(crate) fn basis_decrease(&self) -> i64 {
        if let Some(own) = self.own_form() {
            if !own.reduced {
                return 0;
            }
            let credit_cents = dollars(own.amount) * 100;
            return match self {
                Self::RehabilitationRental { placed_year: 2026, .. }
                | Self::RehabilitationOther { placed_year: 2026, .. } => fraction(
                    fraction(
                        credit_cents,
                        REHABILITATION_CREDIT_PERCENT,
                        REHABILITATION_RATABLE_PERCENT,
                    ),
                    REHABILITATION_BASIS_REDUCTION_PERCENT,
                    100,
                ),
                Self::Energy { .. } | Self::CleanElectricity { .. } => {
                    fraction(credit_cents, ENERGY_BASIS_REDUCTION_PERCENT, 100)
                }
                _ => 0,
            };
        }
        match self {
            Self::RehabilitationRental { expenditures, placed_year, .. }
            | Self::RehabilitationOther { expenditures, placed_year, .. }
                if *placed_year == 2026 =>
            {
                fraction(
                    RehabilitationCredit::determined(*expenditures),
                    REHABILITATION_BASIS_REDUCTION_PERCENT,
                    100,
                )
            }
            Self::Energy { .. } | Self::CleanElectricity { .. } => {
                fraction(self.current_year_credit() * 100, ENERGY_BASIS_REDUCTION_PERCENT, 100)
            }
            _ => 0,
        }
    }
}

// The Form 3468 rows of one K-1 item under `prefix`.
pub(crate) fn report_k1_item(
    r: &mut crate::FederalResult,
    prefix: &str,
    item: &K1InvestmentProperty,
) {
    let (kind, percent, basis) = match item {
        K1InvestmentProperty::RehabilitationRental { expenditures, .. }
        | K1InvestmentProperty::RehabilitationOther { expenditures, .. } => {
            (None, None, *expenditures)
        }
        K1InvestmentProperty::Energy { basis, rate, .. } => (None, Some(rate.percent()), *basis),
        K1InvestmentProperty::CleanElectricity { basis, storage, rate, .. } => {
            (storage.then_some(EnergyProperty::Storage), Some(rate.percent()), *basis)
        }
    };
    let placed_year = match item {
        K1InvestmentProperty::RehabilitationRental { placed_year, .. }
        | K1InvestmentProperty::RehabilitationOther { placed_year, .. } => *placed_year,
        _ => 2026,
    };
    PropertyRows {
        prefix,
        input: "income.pass_through.investment_property",
        section: item.section(),
        kind,
        basis,
        credit: item.current_year_credit(),
        credit_explanation: item.own_form().map(|_| msg!("EXPLAIN_OWN_FORM3468_CREDIT")),
        reduction: item.basis_decrease(),
        record: None,
        period_year: 2026 + 1 - placed_year,
        percent,
        recapture: None,
    }
    .report(r);
}

// Traverse the existing parent-owned property list; there is no second ledger.
fn k1_properties(
    income: &crate::FederalIncome,
) -> alloc::vec::Vec<alloc::borrow::Cow<'_, K1InvestmentProperty>> {
    match income {
        crate::FederalIncome::Partnership { data } => {
            data.investment_property.iter().map(alloc::borrow::Cow::Borrowed).collect()
        }
        crate::FederalIncome::Scorp { data } => crate::ScorpK1::from(data)
            .investment_property
            .into_iter()
            .map(alloc::borrow::Cow::Owned)
            .collect(),
        crate::FederalIncome::PartnershipActivities { data } => {
            data.entity.investment_property.iter().map(alloc::borrow::Cow::Borrowed).collect()
        }
        crate::FederalIncome::ScorpActivities { data } => {
            data.entity.investment_property.iter().map(alloc::borrow::Cow::Borrowed).collect()
        }
        crate::FederalIncome::EstateTrustPortfolio { data } => data
            .statements
            .investment_property
            .iter()
            .map(|v| alloc::borrow::Cow::Borrowed(&v.property))
            .collect(),
        _ => alloc::vec::Vec::new(),
    }
}
pub(crate) fn k1_property_ids(income: &crate::FederalIncome) -> alloc::vec::Vec<i64> {
    k1_properties(income).into_iter().map(|property| property.id()).collect()
}
pub(crate) fn k1_property_ids_mut(income: &mut crate::FederalIncome) -> alloc::vec::Vec<&mut i64> {
    match income {
        crate::FederalIncome::Partnership { data } => {
            data.investment_property.iter_mut().map(K1InvestmentProperty::id_mut).collect()
        }
        crate::FederalIncome::Scorp { data } => data
            .statements
            .investment_property
            .iter_mut()
            .map(K1InvestmentProperty::id_mut)
            .collect(),
        crate::FederalIncome::PartnershipActivities { data } => {
            data.entity.investment_property.iter_mut().map(K1InvestmentProperty::id_mut).collect()
        }
        crate::FederalIncome::ScorpActivities { data } => {
            data.entity.investment_property.iter_mut().map(K1InvestmentProperty::id_mut).collect()
        }
        crate::FederalIncome::EstateTrustPortfolio { data } => {
            data.statements.investment_property.iter_mut().map(|v| v.property.id_mut()).collect()
        }
        _ => alloc::vec::Vec::new(),
    }
}
// Native property source output. The separately owned component relationship
// must supply its actual activity identity; this view never infers one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct K1PropertyCreditSource {
    pub property_id: i64,
    pub basis_decrease_cents: i64,
}
pub(crate) fn k1_property_credit_sources(
    income: &crate::FederalIncome,
) -> Option<alloc::vec::Vec<K1PropertyCreditSource>> {
    property_credit_sources(k1_properties(income).iter().map(alloc::borrow::Cow::as_ref))
}
pub(crate) fn property_credit_sources<'a>(
    properties: impl IntoIterator<Item = &'a K1InvestmentProperty>,
) -> Option<alloc::vec::Vec<K1PropertyCreditSource>> {
    let mut seen = alloc::collections::BTreeSet::new();
    properties
        .into_iter()
        .map(|property| {
            let property_id = property.id();
            (property_id > 0 && seen.insert(property_id)).then(|| K1PropertyCreditSource {
                property_id,
                basis_decrease_cents: property.basis_decrease(),
            })
        })
        .collect()
}

// The total basis decrease of a K-1's investment credit list, cents.
pub(crate) fn k1_basis_decrease(items: &[K1InvestmentProperty]) -> i64 {
    items.iter().map(K1InvestmentProperty::basis_decrease).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measuring_period_ends_a_day_before_the_anniversary() {
        assert_eq!(period_end(20240501, 24), 20260430);
        assert_eq!(period_end(20240601, 24), 20260531);
        assert_eq!(period_end(20231201, 24), 20251130);
        assert_eq!(period_end(20240229, 24), 20260227);
        assert_eq!(period_end(20230115, 60), 20280114);
    }

    #[test]
    fn rehabilitation_history_refuses_an_unrepresentable_total_before_source_validation() {
        let claim = RehabilitationCredit {
            period_start: 20231201,
            phased: false,
            building_basis: 0,
            period_expenditures: 1_000_000,
            earlier: alloc::vec![CreditYear { year: 2025, used: i64::MAX, carried: 1 }],
            freed: 0,
            confirmed: true,
        };
        // Save calls this relationship check before the native scalar admission check.
        let issue = claim.issue(20251201, 1_000_000).unwrap();
        assert_eq!(issue.key, "INVESTMENT_CREDIT_HISTORY_ABOVE_CREDIT");
        assert_eq!(issue.field.unwrap().square(), "[earlier/0/carried]");
    }

    #[test]
    fn full_years_count_from_the_first_of_the_placed_month() {
        assert_eq!(full_years(20240310, 20260630), 2);
        assert_eq!(full_years(20240310, 20260228), 1);
        assert_eq!(full_years(20240310, 20260301), 2);
        assert_eq!(full_years(20210310, 20260630), 5);
        assert_eq!(recapture_percent(2), 60);
        assert_eq!(recapture_percent(5), 0);
        assert_eq!(recapture_percent(9), 0);
    }

    #[test]
    fn recapture_is_the_percent_of_the_credit_used_less_the_freed_credits() {
        // $16,000 used, 60%: $9,600, and the same amount returns to basis.
        let value = recapture(20240310, 20260630, 16_000, 0, 0, false);
        assert_eq!((value.percent, value.tax, value.carryover), (60, 9_600, 0));
        assert_eq!(value.basis_increase, 960_000);
        // Worksheet 2 of $1,000 leaves $15,000 recaptured at 60%.
        let value = recapture(20240310, 20260630, 16_000, 0, 1_000, false);
        assert_eq!(value.tax, 9_000);
        // Energy property: half of the recapture and of the cut carryforward
        // returns to basis.
        let value = recapture(20240310, 20260630, 16_000, 3_000, 0, true);
        assert_eq!((value.tax, value.carryover), (9_600, 1_800));
        assert_eq!(value.basis_increase, (9_600 + 1_800) * 100 / 2);
    }

    #[test]
    fn energy_percent_uses_the_bonus_that_follows_the_rate() {
        assert_eq!(energy_percent(true, false, true), 40);
        assert_eq!(energy_percent(false, false, false), 6);
        assert_eq!(energy_percent(false, true, true), 10);
        assert_eq!(energy_percent(true, true, true), 50);
    }
}

// Allocate amounts; legal character, dates and retained facts are unchanged.
crate::community::divide_enum!(K1InvestmentProperty {
    records: [
        RehabilitationRental { amounts: [expenditures, own_form], facts: [placed_year, id] },
        RehabilitationOther { amounts: [expenditures, own_form], facts: [placed_year, id] },
        Energy { amounts: [basis, own_form], facts: [rate, id] },
        CleanElectricity { amounts: [basis, own_form], facts: [storage, rate, id] },
    ],
    units: [],
    tuples: [],
});

crate::community::divide_record!(OwnInvestmentCredit {
    amounts: [amount],
    gains: [],
    facts: [reduced],
});
