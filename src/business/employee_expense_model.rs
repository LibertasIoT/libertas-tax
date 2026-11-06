use crate::Owner;
use crate::asset_model::BusinessAsset;
use crate::vehicle_model::BusinessMileage;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Who you were in this job
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum EmployeeCapacity {
    /// Armed Forces reservist
    /// Army, Navy, Marine Corps, Air Force or Coast Guard Reserve, Army or Air National Guard, or the Public Health Service Reserve Corps. Enter only expenses of duty travel more than 100 miles from home with an overnight stay: lodging within the federal lodging rate and meals within the federal M&IE rate for the locality (GSA.gov/perdiem), transportation, parking, ferries and tolls, and vehicles at the standard mileage rate. Other reserve expenses are not deductible.
    Reservist,
    /// Qualified performing artist
    /// You performed services in the performing arts as an employee for at least two employers, at least two of them paying you $200 or more, your performing-arts expenses exceed 10% of your performing-arts wages, and your adjusted gross income before these expenses is $16,000 or less (a joint return uses your combined income). If you are married you must file jointly unless you lived apart from your spouse all year. The App checks these from your W-2s and this return; enter only performing-arts expenses.
    PerformingArtist,
    /// Fee-basis state or local government official
    /// You were employed by a state or a political subdivision of a state and paid in whole or in part on a fee basis. Enter only expenses of that office.
    FeeBasisOfficial,
}

/// Form 2106 record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployeeExpenseClaim {
    /// Physical buildings containing separately recorded improvements
    /// Enter each building once; its accepted identity is offered in the improvement picker.
    pub improvement_buildings: Vec<crate::ImprovementBuildingFacts>,
    /// DOT hours-of-service meals
    #[libertas_default(false)]
    pub dot_hours_of_service: bool,
    /// Leased actual-expense vehicles
    pub leased_vehicles: Vec<crate::BusinessLeasedVehicle>,
    /// Whose form
    pub owner: Owner,
    /// Occupation
    #[libertas_size(min = 1)]
    pub occupation: String,
    /// Capacity
    pub capacity: EmployeeCapacity,
    /// Chosen W-2 income record identities
    #[libertas_size(min = 1)]
    pub employers: Vec<i64>,
    /// Line 2 local transportation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub local_transportation: i64,
    /// Line 3 travel
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub travel: i64,
    /// Line 4 other
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub other: i64,
    /// Line 5 meals
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub meals: i64,
    /// Line 7 reimbursements other than code L
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub reimbursements: i64,
    /// Standard-mileage vehicles
    pub mileage: Vec<BusinessMileage>,
    /// Actual-expense vehicles
    pub vehicle_assets: Vec<BusinessAsset>,
    /// Equipment used in this job, with its original recovery sources
    pub equipment_assets: Vec<BusinessAsset>,
}

impl EmployeeExpenseClaim {
    pub(crate) fn valid(&self) -> bool {
        self.admission_issue().is_none()
    }

    pub(crate) fn admission_issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if let Some(issue) =
            crate::building_model::building_facts_issue(&self.improvement_buildings)
        {
            return Some(issue);
        }
        let key = "MESSAGE_CHECK_THE_FORM_2106_AMOUNTS_AND_VEHICLES";
        let amounts =
            [self.local_transportation, self.travel, self.other, self.meals, self.reimbursements];
        let records_valid = !self.occupation.trim().is_empty()
            && (1..=10).contains(&self.employers.len())
            && self
                .employers
                .iter()
                .enumerate()
                .all(|(i, id)| !self.employers[..i].contains(id))
            && amounts.iter().all(|v| (0..=100000000000).contains(v))
            && self.mileage.len() + self.vehicle_assets.len() + self.leased_vehicles.len() <= 10
            && self.leased_vehicles.iter().all(crate::BusinessLeasedVehicle::valid)
            // Car-loan interest and personal property tax are not an employee's
            // deductions (Pub. 463); a reservist uses only the standard mileage rate.
            && self
                .mileage
                .iter()
                .all(|m| m.valid() && m.interest == 0 && m.property_tax == 0)
            && self.equipment_assets.iter().all(|a| !a.is_vehicle() && a.valid())
            && self.vehicle_assets.iter().all(|a| {
                a.is_vehicle()
                    && a.valid()
                    && a.vehicle_loan_interest_entered() == 0
            });
        if !records_valid {
            return Some(ValidationIssue::new(key).at_record());
        }
        if self.capacity == EmployeeCapacity::Reservist
            && !(self.vehicle_assets.is_empty() && self.leased_vehicles.is_empty())
        {
            return Some(ValidationIssue::new(key).at_field(if self.vehicle_assets.is_empty() {
                "leased_vehicles"
            } else {
                "vehicle_assets"
            }));
        }
        None
    }
}
