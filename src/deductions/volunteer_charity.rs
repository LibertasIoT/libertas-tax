// Coverage: see charity.rs (Unit15; original T3-deduct, 2026-09-28): volunteer costs are asked on the itemized
//   hub's gifts of property and carryovers page.
// Direct 2026 volunteer-service expenses for qualified public charities.
// https://www.irs.gov/publications/p526
use crate::federal_rules::valid_date;
use crate::validation::ValidationIssue;
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Choose one method for the same volunteer car travel
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum VolunteerCarCost {
    /// Statutory 2026 charitable mileage rate of 14 cents per mile
    Mileage {
        /// Distance driven solely to provide volunteer service
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("mile")]
        miles: u32,
    },
    /// Direct gas and oil cost instead of statutory mileage
    ActualFuel {
        /// Unreimbursed gas and oil directly attributable to this travel
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        gas_oil: i64,
    },
}
impl VolunteerCarCost {
    fn valid(&self) -> bool {
        match self {
            // Even u32::MAX miles at 14 cents fits i64 and remains below the
            // already admitted ActualFuel source maximum used by these consumers.
            Self::Mileage { miles } => *miles >= 1,
            Self::ActualFuel { gas_oil } => (1..=100_000_000_000).contains(gas_oil),
        }
    }
    fn cents(&self) -> i64 {
        match self {
            Self::Mileage { miles } => i64::from(*miles) * 14,
            Self::ActualFuel { gas_oil } => *gas_oil,
        }
    }
}

/// Direct expenses incurred solely to serve an eligible public charity
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum VolunteerExpenseKind {
    /// Travel in the volunteer's car, using one operating-cost method
    Car {
        /// Mileage or actual gas and oil, never both for the same travel
        car_cost: VolunteerCarCost,
        /// Unreimbursed parking and tolls in addition to either car-cost method
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        parking_tolls: i64,
    },
    /// Necessary away-from-home travel with substantial volunteer duties
    AwayTravel {
        /// Air, rail, bus, taxi or other non-car transportation costs
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        transportation: i64,
        /// Necessary lodging while away from home
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        lodging: i64,
        /// Necessary meals while away from home
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        meals: i64,
    },
    /// Required volunteer uniform unsuitable for ordinary wear
    Uniform {
        /// Purchase cost of the mandatory uniform
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        purchase: i64,
        /// Cleaning and upkeep cost of that uniform
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        upkeep: i64,
    },
    /// Unreimbursed foster-child care primarily benefiting the placing organization
    FosterCare {
        /// Direct out-of-pocket costs to feed, clothe and care for the placed child
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        amount: i64,
    },
    /// Direct cost for charity-selected underprivileged youths to attend an outing
    YouthOuting {
        /// Reasonable unreimbursed admission or meal costs for selected youths, excluding the volunteer's own costs
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        amount: i64,
    },
    /// Required expenses in a church-established permanent diaconate program
    DeaconProgram {
        /// Required vestments for a candidate or ordained deacon
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        vestments: i64,
        /// Required program books
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        books: i64,
        /// Required transportation to provide the church service
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        transportation: i64,
    },
}
impl VolunteerExpenseKind {
    fn valid(&self) -> bool {
        let money = |amount: &i64| (0..=100_000_000_000).contains(amount);
        match self {
            Self::Car { car_cost, parking_tolls } => car_cost.valid() && money(parking_tolls),
            Self::AwayTravel { transportation, lodging, meals } => {
                money(transportation)
                    && money(lodging)
                    && money(meals)
                    && transportation + lodging + meals > 0
            }
            Self::Uniform { purchase, upkeep } => {
                money(purchase) && money(upkeep) && purchase + upkeep > 0
            }
            Self::FosterCare { amount } | Self::YouthOuting { amount } => {
                (1..=100_000_000_000).contains(amount)
            }
            Self::DeaconProgram { vestments, books, transportation } => {
                money(vestments)
                    && money(books)
                    && money(transportation)
                    && vestments + books + transportation > 0
            }
        }
    }
    fn cents(&self) -> i64 {
        match self {
            Self::Car { car_cost, parking_tolls } => car_cost.cents() + parking_tolls,
            Self::AwayTravel { transportation, lodging, meals } => transportation + lodging + meals,
            Self::Uniform { purchase, upkeep } => purchase + upkeep,
            Self::FosterCare { amount } | Self::YouthOuting { amount } => *amount,
            Self::DeaconProgram { vestments, books, transportation } => {
                vestments + books + transportation
            }
        }
    }
}

/// One 2026 volunteer-service expense for an eligible public charity or government unit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct VolunteerExpense {
    /// Date expense was paid for volunteer service in 2026
    #[libertas_date_only]
    pub date: u32,
    /// Eligible U.S. public charity or government unit served
    #[libertas_size(min = 1)]
    pub charity: String,
    /// Services provided for the charity
    #[libertas_size(min = 1)]
    pub service: String,
    /// Type of directly paid service expense
    pub kind: VolunteerExpenseKind,
    /// Service, eligibility and contemporaneous written records confirmed
    /// Check your charity acknowledgment, service and expense records to answer. No leaves this expense undeducted. Confirm the expense arose solely from unpaid services for an eligible public charity or government unit, was neither reimbursed nor claimed elsewhere, and was not personal or family expense. Car travel needs reliable mileage or actual-cost records, and excludes depreciation, repairs, insurance and registration. Away travel needs substantial duties without significant personal pleasure, vacation or recreation; no spouse or child costs. Uniforms must be required and unsuitable for ordinary wear. Foster care needs a child selected by the qualified organization, no profit motive or actual profit, and expenses incurred primarily to benefit that organization rather than an adoption or family purpose. Youth outings need a charity whose purpose is reducing juvenile delinquency, its selection of the youths, and only the youths' reasonable costs. Permanent diaconate needs a program established by a church and costs required for the candidate's or ordained deacon's service. An expense of $250 or more considered separately also needs a timely charity acknowledgment describing the services and any benefits.
    pub documented: bool,
}
impl VolunteerExpense {
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        let issue = |field| ValidationIssue::new(VALIDATION_KEY).at_field(field);
        if !(20260101..=20261231).contains(&self.date) || !valid_date(self.date) {
            return Some(issue("date"));
        }
        if self.charity.trim().is_empty() {
            return Some(issue("charity"));
        }
        if self.service.trim().is_empty() {
            return Some(issue("service"));
        }
        if !self.kind.valid() {
            return Some(issue("kind"));
        }
        None
    }
    pub(crate) fn valid(&self) -> bool {
        self.issue().is_none()
    }
    pub(crate) fn cents(&self) -> i64 {
        self.kind.cents()
    }
}

pub(crate) const VALIDATION_KEY: &str =
    "MESSAGE_CHECK_EACH_VOLUNTEER_COST_A_RETURN_YEAR_DATE_THE_CHARITY_AND_THE_SERVICE_UP_TO_160";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_width_volunteer_mileage_retains_exact_cents() {
        // Full-width mileage retains exact cents without an arbitrary maximum.
        let full = VolunteerCarCost::Mileage { miles: u32::MAX };
        assert!(full.valid());
        assert_eq!(full.cents(), 60_129_542_130);
    }
}
