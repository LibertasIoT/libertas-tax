use crate::common::Owner;
use crate::dates::source_valid_date as valid_date;
use crate::filer_facts::FederalStatus;
use alloc::{string::String, vec::Vec};
use libertas_macros::*;

/// Form 8936: new and previously owned clean vehicles delivered in 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalCleanVehicleClaim {
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub current_territory_addbacks: i64,
    /// Vehicles delivered in 2026
    /// ----
    /// Vehicle
    pub vehicles: Vec<FederalCleanVehicle>,
}

/// 2025 Form 8936 MAGI comparison
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalCleanVehiclePrior {
    /// 2025 filing status
    pub status: FederalStatus,
    /// 2025 modified adjusted gross income
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub magi: i64,
}

/// One Schedule A (Form 8936) per vehicle
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FederalCleanVehicle {
    /// New clean vehicle
    New {
        /// Vehicle and purchase details
        details: FederalCleanVehicleDetails,
        /// Tentative credit from the accepted seller report, no more than $7,500
        #[libertas_money("USD", 2)]
        // Tax structure: The new-clean-vehicle credit and its dealer transfer cannot exceed $7,500.
        #[libertas_number(min = 0, max = 750000)]
        seller_credit: i64,
        /// Business or farm use of this new vehicle
        /// Select the same saved owned vehicle from Self-employment or Farming. Supply the distinct Form 8936 credit-use fraction from your use records.
        /// [NullText]
        /// This vehicle is used only personally
        /// [SetText]
        /// This saved vehicle is used in my business or farm
        business_use: Option<crate::clean_vehicle::CleanVehicleBusinessUse>,
    },
    /// Previously owned clean vehicle
    PreviouslyOwned {
        /// Vehicle and purchase details
        details: FederalCleanVehicleDetails,
        /// Vehicle model year; at least two years before the acquisition year
        // Tax structure: this date or year belongs to the stated current or retained-return period.
        #[libertas_number(min = 1900, max = 2023)]
        model_year: u32,
        /// Sales price on the accepted seller report, including required dealer fees
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        sales_price: i64,
        /// This is the first qualifying dealer transfer since August 16, 2022
        /// The dealer's time-of-sale report for this VIN states it.
        first_eligible_transfer: bool,
        /// The buyer has not claimed a previously owned clean vehicle credit in the three years before this purchase
        /// The buyer's 2023 through 2025 returns (Form 8936 Part III) show it.
        no_credit_in_three_years: bool,
    },
}

/// Facts shared by new and previously owned vehicle credits
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalCleanVehicleDetails {
    /// Seventeen-character VIN from the seller report
    #[libertas_pattern(r"[A-HJ-NPR-Za-hj-npr-z0-9]{17}")]
    pub vin: String,
    /// Buyer named in the accepted seller report
    pub buyer: Owner,
    /// Date of binding written contract and payment, including a nominal deposit
    pub acquired: u32,
    /// Date possession was taken in 2026
    pub placed_in_service: u32,
    /// The seller supplied an accepted time-of-sale report with this VIN and buyer
    /// The seller gives a copy of the IRS Energy Credits Online report at purchase.
    pub accepted_seller_report: bool,
    /// The vehicle meets the manufacturer, battery or fuel-cell, assembly, weight, price and other applicable IRS requirements
    /// The accepted seller report and the fueleconomy.gov qualified-vehicle list show it.
    pub qualified_vehicle: bool,
    /// The buyer owns the vehicle, uses it mainly in the United States, and acquired it for use or to lease to others, not for resale
    /// Only an individual may claim a previously owned vehicle. For a new vehicle, select any business or farm use separately. No gives no credit.
    pub owned_for_use: bool,
    /// What happened in the 30 days after possession
    /// Check the return or sale date in your records. A later sale alone does not require repayment of this credit.
    pub disposition: CleanVehicleDisposition,
    /// Credit amount transferred to the dealer at sale, if any
    /// Dealer transfers require a separate reconciliation, even if tax liability is zero.
    /// [NullText]
    /// No clean vehicle credit was transferred to the dealer
    /// [SetText]
    /// The clean vehicle credit transferred to the dealer was
    #[libertas_money("USD", 2)]
    // Tax structure: The new-clean-vehicle credit and its dealer transfer cannot exceed $7,500.
    #[libertas_number(min = 1, max = 750000)]
    pub dealer_transfer: Option<i64>,
}

/// What happened to the vehicle in the 30 days after possession
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum CleanVehicleDisposition {
    /// Kept for at least 30 days
    Kept,
    /// Returned to the seller within 30 days
    /// No credit. A dealer transfer is cancelled; the IRS recovers its payment from the dealer, not you.
    ReturnedWithin30Days,
    /// Resold within 30 days
    /// No credit. Repay any credit transferred to the dealer with your tax on Schedule 2.
    ResoldWithin30Days,
}

/// Saved business or farm using a new clean vehicle
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CleanVehicleActivity {
    /// A business on Self-employment
    Business {
        /// Saved business
        business: i64,
    },
    /// A farm on Farming
    Farm {
        /// Saved farm
        farm: i64,
    },
}

/// Credit-use fraction and the saved owned vehicle supplying basis and activity identity
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CleanVehicleBusinessUse {
    /// Business or farm that owns this vehicle
    pub activity: CleanVehicleActivity,
    /// Same saved owned vehicle on that activity
    /// Select the same saved owned vehicle. Correct an actual-expense vehicle's credit basis reduction in its saved property record on Self-employment or Farming.
    pub property: i64,
    /// Business/investment use fraction for Schedule A (Form 8936) line 10
    /// From your actual use records, enter business or income-producing miles over all miles, excluding commuting. For a personal-to-business conversion or its reverse, use the business-use period's fraction multiplied by its business-use months over 12. Employee personal use included in wages or reimbursed by employees, and a vehicle you own and lease to others, count as 100%. This credit-use fraction can differ from the annual depreciation-use fraction of the selected property; do not round it to a whole percent.
    pub credit_use: crate::UseFraction,
}
impl CleanVehicleBusinessUse {
    pub(crate) fn fraction_valid(&self) -> bool {
        self.credit_use.total > 0 && self.credit_use.used <= self.credit_use.total
    }
}

impl FederalCleanVehicle {
    pub(crate) fn details(&self) -> &FederalCleanVehicleDetails {
        match self {
            Self::New { details, .. } | Self::PreviouslyOwned { details, .. } => details,
        }
    }
}

pub(crate) fn valid(claim: &FederalCleanVehicleClaim) -> bool {
    if !(0..=MAX_CENTS).contains(&claim.current_territory_addbacks)
        || claim.prior_year.as_ref().is_some_and(|prior| !(0..=MAX_CENTS).contains(&prior.magi))
    {
        return false;
    }
    for (index, vehicle) in claim.vehicles.iter().enumerate() {
        let details = vehicle.details();
        if details.vin.len() != 17
            || !details.vin.bytes().all(|byte| {
                byte.is_ascii_digit()
                    || (byte.is_ascii_alphabetic()
                        && !matches!(byte.to_ascii_uppercase(), b'I' | b'O' | b'Q'))
            })
            || !valid_date(details.acquired)
            || !valid_date(details.placed_in_service)
            || details.dealer_transfer.is_some_and(|amount| !(1..=750_000).contains(&amount))
            || claim.vehicles[..index]
                .iter()
                .any(|earlier| earlier.details().vin.eq_ignore_ascii_case(&details.vin))
        {
            return false;
        }
        match vehicle {
            FederalCleanVehicle::New { seller_credit, business_use, .. } => {
                if !(0..=750_000).contains(seller_credit)
                    || business_use.as_ref().is_some_and(|use_| !use_.fraction_valid())
                {
                    return false;
                }
            }
            FederalCleanVehicle::PreviouslyOwned { model_year, sales_price, .. } => {
                if !(1900..=2023).contains(model_year) || !(0..=MAX_CENTS).contains(sales_price) {
                    return false;
                }
            }
        }
    }
    true
}

const MAX_CENTS: i64 = 100_000_000_000;
