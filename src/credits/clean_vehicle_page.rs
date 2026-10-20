// Editable identity pickers stay on page projections; accepted records retain IDs only.
use crate::{
    CleanVehicleActivity, CleanVehicleBusinessUse, FederalCleanVehicle, FederalCleanVehicleClaim,
    FederalCleanVehicleDetails, FederalCleanVehiclePrior, UseFraction,
};
use alloc::vec::Vec;
use libertas_macros::*;

/// Saved business or farm using a new clean vehicle
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CleanVehicleActivityPage {
    /// A business on Self-employment
    Business {
        /// Saved business
        #[libertas_enum_source("$.businesses")]
        business: i64,
    },
    /// A farm on Farming
    Farm {
        /// Saved farm
        #[libertas_enum_source("$.farms")]
        farm: i64,
    },
}
impl From<CleanVehicleActivity> for CleanVehicleActivityPage {
    fn from(value: CleanVehicleActivity) -> Self {
        match value {
            CleanVehicleActivity::Business { business } => Self::Business { business },
            CleanVehicleActivity::Farm { farm } => Self::Farm { farm },
        }
    }
}
impl From<CleanVehicleActivityPage> for CleanVehicleActivity {
    fn from(value: CleanVehicleActivityPage) -> Self {
        match value {
            CleanVehicleActivityPage::Business { business } => Self::Business { business },
            CleanVehicleActivityPage::Farm { farm } => Self::Farm { farm },
        }
    }
}

/// Credit-use fraction and saved owned vehicle
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CleanVehicleBusinessUsePage {
    /// Business or farm that owns this vehicle
    pub activity: CleanVehicleActivityPage,
    /// Same saved owned vehicle on that activity
    /// Select the same saved owned vehicle. Correct an actual-expense vehicle's credit basis reduction in its saved property record on Self-employment or Farming.
    #[libertas_enum_source("$.clean_vehicle_properties")]
    pub property: i64,
    /// Business/investment use fraction for Schedule A (Form 8936) line 10
    /// From your actual use records, enter business or income-producing miles over all miles, excluding commuting. For a personal-to-business conversion or its reverse, use the business-use period's fraction multiplied by its business-use months over 12. Employee personal use included in wages or reimbursed by employees, and a vehicle you own and lease to others, count as 100%. This credit-use fraction can differ from the annual depreciation-use fraction of the selected property; do not round it to a whole percent.
    pub credit_use: UseFraction,
}
impl From<CleanVehicleBusinessUse> for CleanVehicleBusinessUsePage {
    fn from(value: CleanVehicleBusinessUse) -> Self {
        Self {
            activity: value.activity.into(),
            property: value.property,
            credit_use: value.credit_use,
        }
    }
}
impl From<CleanVehicleBusinessUsePage> for CleanVehicleBusinessUse {
    fn from(value: CleanVehicleBusinessUsePage) -> Self {
        Self {
            activity: value.activity.into(),
            property: value.property,
            credit_use: value.credit_use,
        }
    }
}

/// One Schedule A (Form 8936) per vehicle
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FederalCleanVehiclePage {
    /// New clean vehicle
    New {
        /// Vehicle and purchase details
        details: FederalCleanVehicleDetails,
        /// Tentative credit from the accepted seller report, no more than $7,500
        #[libertas_money("USD", 2)]
        // Tax structure: a new clean vehicle's tentative credit cannot exceed $7,500.
        #[libertas_number(min = 0, max = 750000)]
        seller_credit: i64,
        /// Business or farm use of this new vehicle
        /// Select the same saved owned vehicle from Self-employment or Farming. Supply the distinct Form 8936 credit-use fraction from your use records.
        /// [NullText]
        /// This vehicle is used only personally
        /// [SetText]
        /// This saved vehicle is used in my business or farm
        business_use: Option<CleanVehicleBusinessUsePage>,
    },
    /// Previously owned clean vehicle
    PreviouslyOwned {
        /// Vehicle and purchase details
        details: FederalCleanVehicleDetails,
        /// Vehicle model year; at least two years before the acquisition year
        // Tax structure: an eligible vehicle acquired by September 2025 is no newer than 2023.
        #[libertas_number(min = 1900, max = 2023)]
        model_year: u32,
        /// Sales price on the accepted seller report, including required dealer fees
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        sales_price: i64,
        /// This is the first qualifying dealer transfer since August 16, 2022
        /// The dealer's time-of-sale report for this VIN states it.
        first_eligible_transfer: bool,
        /// The buyer has not claimed a previously owned clean vehicle credit in the three years before this purchase
        /// The buyer's 2023 through 2025 returns (Form 8936 Part III) show it.
        no_credit_in_three_years: bool,
    },
}
impl From<FederalCleanVehicle> for FederalCleanVehiclePage {
    fn from(value: FederalCleanVehicle) -> Self {
        match value {
            FederalCleanVehicle::New { details, seller_credit, business_use } => {
                Self::New { details, seller_credit, business_use: business_use.map(Into::into) }
            }
            FederalCleanVehicle::PreviouslyOwned {
                details,
                model_year,
                sales_price,
                first_eligible_transfer,
                no_credit_in_three_years,
            } => Self::PreviouslyOwned {
                details,
                model_year,
                sales_price,
                first_eligible_transfer,
                no_credit_in_three_years,
            },
        }
    }
}
impl From<FederalCleanVehiclePage> for FederalCleanVehicle {
    fn from(value: FederalCleanVehiclePage) -> Self {
        match value {
            FederalCleanVehiclePage::New { details, seller_credit, business_use } => {
                Self::New { details, seller_credit, business_use: business_use.map(Into::into) }
            }
            FederalCleanVehiclePage::PreviouslyOwned {
                details,
                model_year,
                sales_price,
                first_eligible_transfer,
                no_credit_in_three_years,
            } => Self::PreviouslyOwned {
                details,
                model_year,
                sales_price,
                first_eligible_transfer,
                no_credit_in_three_years,
            },
        }
    }
}

/// Form 8936: new and previously owned vehicles delivered in 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalCleanVehicleClaimPage {
    /// 2025 modified AGI and filing status, when the 2026 MAGI exceeds the applicable limit
    /// Include 2025 Form 1040 AGI plus excluded Puerto Rico income, Form 2555 lines 45 and 50, and Form 4563 line 15.
    /// [NullText]
    /// Use only 2026 modified AGI for the income-limit test
    /// [SetText]
    /// My 2025 modified AGI and filing status were
    #[libertas_default($null)]
    pub prior_year: Option<FederalCleanVehiclePrior>,
    /// 2026 excluded Puerto Rico income and Form 4563 line 15
    /// Form 2555 exclusions and housing deductions are already included from the income interview.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub current_territory_addbacks: i64,
    /// Vehicles delivered in 2026
    /// ----
    /// Vehicle
    pub vehicles: Vec<FederalCleanVehiclePage>,
}
impl From<FederalCleanVehicleClaim> for FederalCleanVehicleClaimPage {
    fn from(value: FederalCleanVehicleClaim) -> Self {
        Self {
            prior_year: value.prior_year,
            current_territory_addbacks: value.current_territory_addbacks,
            vehicles: value.vehicles.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<FederalCleanVehicleClaimPage> for FederalCleanVehicleClaim {
    fn from(value: FederalCleanVehicleClaimPage) -> Self {
        Self {
            prior_year: value.prior_year,
            current_territory_addbacks: value.current_territory_addbacks,
            vehicles: value.vehicles.into_iter().map(Into::into).collect(),
        }
    }
}
