// Investment-credit property declarations and original credit records.
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Credit from this property for one earlier year
/// From your Forms 3800 for that year and later years: the part of that year's credit from this property you used, and the part still carried to 2026.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CreditYear {
    /// Year the credit arose
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 2021, max = 2025)]
    pub year: u32,
    /// Part used on your 2025 or earlier returns
    /// Including any part carried back. Whole dollars.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub used: i64,
    /// Part carried forward to 2026
    /// Added to your general business credit carryforwards from this property; do not enter it again under More credits.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub carried: i64,
}

/// Rehabilitation credit for a certified historic building (Form 3468 Part VII)
/// The 20% credit for this row's qualified rehabilitation expenditures, allowed 4% a year for five years from the year the building was placed in service. The App reduces the building's depreciable basis by the full 20% and figures recapture (Form 4255) when the building is sold within five years.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct RehabilitationCredit {
    /// First day of your measuring period (line 1d)
    /// You choose a 24-month period (60 months for a rehabilitation done in phases under written plans completed before work began) that ends in the year the building was placed in service. The App shows its last day.
    #[libertas_date_only]
    pub period_start: u32,
    /// Rehabilitation done in phases under written architectural plans
    /// Yes uses a 60-month measuring period.
    #[libertas_default(false)]
    pub phased: bool,
    /// Adjusted basis of the building at the start of the period (line 1e)
    /// Your cost of the building and its structural components, less depreciation, on the period's first day (or the day you acquired it, if later). Exclude land.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub building_basis: i64,
    /// Qualified rehabilitation expenditures during the period (line 1f)
    /// All qualified rehabilitation expenditures for this building during the measuring period, including other phases. They must exceed both $5,000 and the adjusted basis above.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub period_expenditures: i64,
    /// This building's credit on your 2021–2025 returns
    /// One row for each year of the five-year credit period before 2026. Leave empty for a building placed in service in 2026.
    #[libertas_unordered]
    // Tax structure: the five prior credit years, 2021 through 2025.
    #[libertas_size(max = 5)]
    /// ----
    /// Year
    pub earlier: Vec<CreditYear>,
    /// Other credits you could have used instead (Form 4255 line 13)
    /// Needed only when the building was sold or stopped being used in your business in 2026 within five years: the result of Form 4255 Worksheet 2. Zero when you had no other unused general business credits in those years.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub freed: i64,
    /// Certified historic rehabilitation confirmed
    /// The building is listed in the National Register or certified as contributing to a registered historic district; the National Park Service approved the rehabilitation (Part 3 of NPS Form 10-168), or its approval is pending and you will file within 30 months of the first claim; this row holds only qualified rehabilitation expenditures — capitalized, depreciated straight line, not the cost of buying or enlarging the building and not for a part used by a tax-exempt or government user; you own the building (no lease pass-through election) and made no election to take expenditures before the building was placed in service; none of its cost is financed by nonqualified nonrecourse financing (section 49), and after it was placed in service that financing never increased. No leaves the 2026 credit out of this estimate.
    pub confirmed: bool,
}

/// Energy property
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EnergyProperty {
    /// Solar panels or other solar electric or heating equipment
    /// Not equipment that heats a swimming pool.
    Solar,
    /// Wind turbine
    /// With construction begun before 2025, a turbine of 100 kilowatts or less.
    Wind,
    /// Battery or other energy storage
    /// At least 5 kilowatt-hours of capacity.
    Storage,
    /// Geothermal heat pump
    /// Equipment using the ground or ground water to heat or cool a building of this business.
    GeothermalHeatPump,
}

/// Section of an earlier energy credit
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EnergySection {
    /// Energy credit (Form 3468 Part VI, section 48)
    Energy,
    /// Clean electricity investment credit (Form 3468 Part V, section 48E)
    CleanElectricity,
}

/// Investment credit (Form 3468)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[allow(clippy::large_enum_variant)]
pub enum InvestmentCredit {
    /// None
    No,
    /// Rehabilitation credit for a certified historic building (Form 3468 Part VII)
    /// Only on a 39-year building row that holds just the qualified rehabilitation expenditures; enter the purchase of the building and any enlargement as other rows.
    Rehabilitation(RehabilitationCredit),
    /// Energy credit for property placed in service in 2026 (Form 3468 Part V or VI)
    /// Solar, wind, energy storage or geothermal heat pump equipment of this business. The App reduces its depreciable basis by half of the credit.
    NewEnergy {
        /// Kind of property
        kind: EnergyProperty,
        /// Date construction began (line 4)
        /// When physical work of a significant nature began, or when you first paid or incurred 5% of the cost. Before 2025 it is the energy credit (section 48); later, the clean electricity investment credit (section 48E), except a geothermal heat pump.
        #[libertas_date_only]
        construction_began: u32,
        /// Qualifies for the 30% rate (line 7a or 8b)
        /// Yes when the installation's maximum net output is under 1 megawatt AC (storage: under 1 megawatt of capacity), construction began before January 29, 2023, or the prevailing wage and apprenticeship requirements were met without penalty payments. No uses 6%.
        increased_rate: bool,
        /// Meets the domestic content requirement (line 9)
        /// You have the manufacturer's certification that the steel, iron and manufactured products meet the section 48 or 48E domestic content percentage.
        #[libertas_default(false)]
        domestic_content: bool,
        /// Located in an energy community (line 10)
        /// A brownfield site, or a census tract the IRS lists for the year (statistical area with fossil-fuel employment, or a closed coal mine or plant).
        #[libertas_default(false)]
        energy_community: bool,
        /// Energy credit eligibility confirmed
        /// A new installation you own (not an expansion of an existing one or leased to you), used in the United States in this business, meeting the IRS performance standards; no low-income communities allocation or interconnection property is claimed; no tax-exempt bonds, subsidized energy financing or nonqualified nonrecourse financing (section 49); for construction begun before 2025, placed in service within four years of it; for clean electricity property begun after 2025, no prohibited foreign entity provided material assistance; no elective payment or transfer of the credit. No leaves this credit out of this estimate.
        confirmed: bool,
    },
    /// Energy credit claimed for an earlier year
    /// Property placed in service before 2026 whose basis an energy or clean electricity investment credit reduced. The App keeps the reduced depreciable basis and figures recapture (Form 4255) when it is sold within five years.
    EarlierEnergy {
        /// Credit
        section: EnergySection,
        /// Credit determined for this property
        /// From the Form 3468 for the year it was placed in service, before any limit. Whole dollars.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 100, max = 100000000000)]
        credit: i64,
        /// Part used on your 2025 or earlier returns
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        used: i64,
        /// Part carried forward to 2026
        /// Added to your general business credit carryforwards from this property; do not enter it again under More credits.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        carried: i64,
        /// Other credits you could have used instead (Form 4255 line 13)
        /// Needed only when the property was sold or stopped being used in your business in 2026 within five years. Zero when you had no other unused general business credits in those years.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        freed: i64,
        /// No later change confirmed
        /// Since it was placed in service, nonqualified nonrecourse financing for it never increased and, if the 30% rate rested on prevailing wages, every alteration or repair paid them. No: the recapture of those changes is not calculated, and this return is refused.
        unchanged: bool,
    },
}

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(RehabilitationCredit {
    amounts: [building_basis, period_expenditures, earlier, freed],
    gains: [],
    facts: [period_start, phased, confirmed],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(CreditYear { amounts: [used, carried], gains: [], facts: [year] });

// Community-property projections preserve accepted source identity and facts.
crate::community::divide_enum!(InvestmentCredit {
    records: [
        NewEnergy {
            amounts: [kind],
            facts: [
                construction_began,
                increased_rate,
                domestic_content,
                energy_community,
                confirmed
            ]
        },
        EarlierEnergy { amounts: [section, credit, used, carried, freed], facts: [unchanged] }
    ],
    units: [No],
    tuples: [Rehabilitation],
});

crate::community::divide_enum!(EnergySection {
    records: [],
    units: [Energy, CleanElectricity],
    tuples: [],
});

crate::community::divide_enum!(EnergyProperty {
    records: [],
    units: [Solar, Wind, Storage, GeothermalHeatPump],
    tuples: [],
});
