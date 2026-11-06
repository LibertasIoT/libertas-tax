use alloc::string::String;
use libertas_macros::*;

/// Personal-use Form 8911 Schedule A refueling property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalPersonalRefueling {
    /// Equipment or charging-port label
    pub label: String,
    /// Date the unit became ready and available for use
    #[libertas_date_only]
    pub placed_in_service: u32,
    /// Qualified cost after rebates and other basis reductions, including only this item's allocated share of directly traceable installation costs
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cost: i64,
    /// Eleven-digit 2020 census-tract GEOID for the installation address
    #[libertas_pattern(r"[0-9]{11}")]
    pub census_geoid: String,
    /// The GEOID is on the IRS eligible-tract list
    /// Look the GEOID up in the IRS eligible census tract list (Notice 2024-20 appendices) or the Argonne 30C map.
    pub eligible_tract: bool,
    /// This qualifying unit was new to you, installed at your U.S. main home, and met applicable safety requirements
    /// The purchase and installation invoice, and any permit or inspection, show it.
    pub qualified_property: bool,
    /// The unit had no business or investment use in 2026
    /// Mixed and business use requires the separate Form 3800 credit and basis calculation, so No leaves this unit's credit out of this estimate.
    pub personal_only: bool,
}

impl FederalPersonalRefueling {
    pub(crate) fn valid(&self) -> bool {
        !(self.label.trim().is_empty()
            || !crate::dates::source_valid_date(self.placed_in_service)
            || !(0..=100000000000).contains(&self.cost)
            // Identifier format: a Census tract GEOID has exactly eleven digits.
            || self.census_geoid.len() != 11
            || !self.census_geoid.bytes().all(|digit| digit.is_ascii_digit()))
    }
}
