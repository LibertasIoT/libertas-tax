use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Vehicle leased for 30 days or more, actual expenses
/// Car, van, pickup or panel truck you lease and deduct by its lease payments and operating costs. A leased vehicle on the standard mileage rate is entered as a standard-mileage vehicle instead. Enter it once; a rental of under 30 days is line 20a rent.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct BusinessLeasedVehicle {
    /// First day of the lease term
    /// The App finds the inclusion-amount table by this year and counts the lease days in 2026 from it. For a vehicle you leased for personal use and later used in business, enter the first day of business use.
    #[libertas_date_only]
    pub began: u32,
    /// Whether the lease ends in 2026
    /// Choose continues after 2026 or enter the actual last day of the term in 2026.
    /// [NullText]
    /// The lease continues after 2026
    /// [SetText]
    /// The lease ends in 2026 on
    #[libertas_date_only]
    pub ends: Option<u32>,
    /// Fair market value on the first day of the lease
    /// The capitalized cost stated in the lease, or the value a willing buyer would pay. For a vehicle converted to business use, the value on the conversion date. No inclusion amount applies at $62,000 or less for a lease begun in 2024, 2025 or 2026.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub fair_market_value: i64,
    /// Lease payments for 2026
    /// The whole-vehicle payments for the year; spread an advance payment over the lease term and enter 2026's share. Not payments to buy the vehicle.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub lease_payments: i64,
    /// Total distance driven during 2026
    #[libertas_number(min = 1)]
    #[libertas_physical_unit("mile")]
    pub total_miles: u32,
    /// Business distance driven during 2026
    /// Exclude commuting, personal travel and distance claimed by another business. The business share of every amount below is this distance over the total.
    #[libertas_number(min = 1)]
    #[libertas_physical_unit("mile")]
    pub business_miles: u32,
    /// Commuting distance included in the annual total
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    #[libertas_physical_unit("mile")]
    pub commuting: u32,
    /// Fuel, insurance, repairs, registration and other operating costs for 2026
    /// Whole-vehicle amounts; not the lease payments, parking or tolls.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub operating_costs: i64,
    /// Business parking and tolls, excluding commuting
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub parking_tolls: i64,
    /// Gross vehicle weight 6,000 pounds or less
    /// Yes for a car or a truck or van rated at 6,000 pounds or less; the inclusion amount applies only to these. No for a heavier truck or van.
    #[libertas_default(true)]
    pub passenger_automobile: bool,
    /// Available for personal use during off-duty hours
    /// Schedule C Part IV line 45.
    #[libertas_default(true)]
    pub personal_use_available: bool,
    /// You or your spouse have another vehicle available for personal use
    /// Schedule C Part IV line 46.
    #[libertas_default(true)]
    pub other_vehicle: bool,
    /// Evidence supporting the deduction
    /// Schedule C Part IV lines 47a and 47b.
    #[libertas_default(Written)]
    pub evidence: VehicleEvidence,
}

/// Evidence supporting a vehicle deduction
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
)]
#[libertas_inline_choices]
pub enum VehicleEvidence {
    /// No evidence
    None,
    /// Evidence that is not written
    Oral,
    /// Written evidence, such as a mileage log
    #[default]
    Written,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct LeaseAmounts {
    pub payments: i64,
    pub inclusion: i64,
    pub operating: i64,
}
impl LeaseAmounts {
    pub(crate) fn total(self) -> i64 {
        self.net_payments() + self.operating
    }
    pub(crate) fn net_payments(self) -> i64 {
        crate::kernel::phase::floor(self.payments - self.inclusion, 0)
    }
}
impl BusinessLeasedVehicle {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if self.business_miles > self.total_miles {
            return Some(
                ValidationIssue::new("VEHICLE_BUSINESS_MILES_ABOVE_TOTAL")
                    .at_field("business_miles"),
            );
        }
        if self.business_miles.saturating_add(self.commuting) > self.total_miles {
            return Some(
                ValidationIssue::new("VEHICLE_BUSINESS_MILES_ABOVE_TOTAL").at_field("commuting"),
            );
        }
        if self.began < 20180101 {
            return Some(ValidationIssue::new("LEASE_BEFORE_2018").at_field("began"));
        }
        if !crate::dates::source_valid_date(self.began) || self.began > 20261231 {
            return Some(ValidationIssue::new("LEASE_ENDS_BEFORE_BEGAN").at_field("began"));
        }
        if self.ends.is_some_and(|d| {
            !crate::dates::source_valid_date(d)
                || !(20260101..=20261231).contains(&d)
                || d <= self.began
        }) {
            return Some(ValidationIssue::new("LEASE_ENDS_BEFORE_BEGAN").at_field("ends"));
        }
        if self.ends.is_some_and(|d| crate::dates::days_between(self.began, d) + 1 < 30) {
            return Some(ValidationIssue::new("LEASE_TERM_UNDER_30_DAYS").at_field("ends"));
        }
        None
    }
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn valid(&self) -> bool {
        self.rule().is_none()
            && (1..=2000000).contains(&self.total_miles)
            && (1..=self.total_miles).contains(&self.business_miles)
            && (1..=100000000000).contains(&self.fair_market_value)
            && [self.lease_payments, self.operating_costs, self.parking_tolls]
                .into_iter()
                .all(|a| (0..=100000000000).contains(&a))
    }
    pub(crate) fn amounts(&self) -> LeaseAmounts {
        if self.total_miles == 0 {
            return LeaseAmounts::default();
        }
        let fraction = crate::kernel::ratio::fraction;
        let payments = fraction(
            self.lease_payments,
            i64::from(self.business_miles),
            i64::from(self.total_miles),
        );
        let operating = fraction(
            self.operating_costs,
            i64::from(self.business_miles),
            i64::from(self.total_miles),
        ) + self.parking_tolls;
        let year = 2026u32.saturating_sub(self.began / 10000) + 1;
        let column =
            (if self.ends.is_some() { year.saturating_sub(1).max(1) } else { year }).min(5);
        let table = crate::year_constants::LEASE_INCLUSION_TABLES
            .iter()
            .find(|(y, _)| *y == self.began / 10000);
        let annual = if self.passenger_automobile {
            table
                .and_then(|(_, rows)| {
                    rows.iter().find(|(lo, hi, _)| {
                        self.fair_market_value > *lo && self.fair_market_value <= *hi
                    })
                })
                .map_or(0, |(_, _, values)| values[column.saturating_sub(1) as usize])
        } else {
            0
        };
        let days = i64::from(
            crate::dates::days_between(self.began.max(20260101), self.ends.unwrap_or(20261231)) + 1,
        )
        .max(0);
        // One exact rational allocation, then the form's whole-dollar inclusion.
        let numerator = i128::from(annual) * i128::from(days) * i128::from(self.business_miles);
        let denominator = 365 * i128::from(self.total_miles);
        let inclusion = crate::kernel::ratio::half_offset_i128(numerator, denominator) as i64 * 100;
        LeaseAmounts { payments, inclusion, operating }
    }
}

/// Vehicle using the standard-mileage method
/// Car, van, pickup or panel truck; no five-or-more-vehicle simultaneous fleet. For owned vehicles, standard mileage was chosen in the first business-use year and no disqualifying accelerated depreciation, bonus, section 179 or special allowance was claimed. Leased vehicles use standard mileage throughout the lease and renewals. Maintain mileage/business-purpose records. No actual operating costs, lease payments or depreciation claimed again; no sale, trade-in or recapture this year. Enter this vehicle only once for this business and allocate any other business's use separately.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessMileage {
    /// Canonical vehicle identity assigned on Save
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Date placed in service for business (Schedule C Part IV line 43)
    #[libertas_date_only]
    #[libertas_default($null)]
    pub placed: Option<u32>,
    /// Available for personal use during off-duty hours (line 45)
    #[libertas_default(true)]
    pub personal_use_available: bool,
    /// Another vehicle available for personal use (line 46)
    #[libertas_default(true)]
    pub other_vehicle: bool,
    /// Evidence supporting the vehicle deduction (lines 47a and 47b)
    #[libertas_default(Written)]
    pub evidence: crate::VehicleEvidence,

    /// Business distance from January 1 through June 30
    /// Exclude commuting, personal travel and distance claimed by another business.
    #[libertas_number(min = 0)]
    #[libertas_physical_unit("mile")]
    pub first_half: u32,
    /// Business distance from July 1 through December 31
    #[libertas_number(min = 0)]
    #[libertas_physical_unit("mile")]
    pub second_half: u32,
    /// Total distance driven during 2026
    /// Include business, commuting and personal driving; used to allocate interest and vehicle tax.
    #[libertas_number(min = 0)]
    #[libertas_physical_unit("mile")]
    pub total: u32,
    /// Commuting distance included in the annual total
    #[libertas_number(min = 0)]
    #[libertas_physical_unit("mile")]
    pub commuting: u32,
    /// Business parking and tolls, excluding commuting
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub parking_tolls: i64,
    /// Annual vehicle-loan interest paid
    /// Enter the whole vehicle amount. The App allocates its business share; do not deduct that share elsewhere. Enter zero for a leased vehicle.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub interest: i64,
    /// Annual value-based personal property tax on the vehicle
    /// Enter the whole vehicle amount. Do not include registration/license fees or deduct the calculated business share elsewhere.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub property_tax: i64,
    /// You own this vehicle
    /// Turn off for a leased vehicle. Only owned vehicles receive a depreciation basis-reduction amount.
    pub owned: bool,
}
impl BusinessMileage {
    pub(crate) fn miles(&self) -> i64 {
        i64::from(self.first_half) + i64::from(self.second_half)
    }
    pub(crate) fn mileage_cost_twice(&self) -> i64 {
        i64::from(self.first_half) * crate::year_constants::MILEAGE_RATE_2026_JAN_JUN_HALF_CENTS
            + i64::from(self.second_half)
                * crate::year_constants::MILEAGE_RATE_2026_JUL_DEC_HALF_CENTS
    }
    fn additional_cost(&self) -> i64 {
        self.parking_tolls
            + if self.total == 0 {
                0
            } else {
                crate::kernel::ratio::fraction(
                    self.interest + self.property_tax,
                    self.miles(),
                    i64::from(self.total),
                )
            }
    }
    // The business-use share of the vehicle-loan interest (Schedule C line 16b,
    // Schedule F line 21b in the 2026 drafts): annual interest times business
    // miles over total miles, the allocation `additional_cost` already makes.
    fn loan_interest_share(&self) -> i64 {
        if self.total == 0 {
            0
        } else {
            crate::kernel::ratio::fraction(self.interest, self.miles(), i64::from(self.total))
        }
    }
    // The business-use share of the vehicle's personal property tax, which a
    // standard-mileage vehicle reports on Schedule C line 23 (2026 i1040sc line 9,
    // "Standard mileage rate"), not on line 9.
    fn property_tax_share(&self) -> i64 {
        if self.total == 0 {
            0
        } else {
            crate::kernel::ratio::fraction(self.property_tax, self.miles(), i64::from(self.total))
        }
    }
    pub(crate) fn expense(&self) -> i64 {
        crate::kernel::ratio::fraction(self.mileage_cost_twice(), 1, 2) + self.additional_cost()
    }
    // The vehicles' business-use property tax, which prints on line 23.
    pub(crate) fn property_tax(vehicles: &[Self]) -> i64 {
        vehicles.iter().map(Self::property_tax_share).sum()
    }
    // The vehicles' business-use loan interest, which prints on its own line.
    pub(crate) fn loan_interest(vehicles: &[Self]) -> i64 {
        vehicles.iter().map(Self::loan_interest_share).sum()
    }
    pub(crate) fn total(vehicles: &[Self]) -> i64 {
        crate::kernel::ratio::fraction(vehicles.iter().map(Self::mileage_cost_twice).sum(), 1, 2)
            + vehicles.iter().map(Self::additional_cost).sum::<i64>()
    }
    pub(crate) fn basis_reduction(&self) -> i64 {
        if self.owned {
            self.miles() * crate::year_constants::MILEAGE_DEPRECIATION_COMPONENT_2026_CENTS_PER_MILE
        } else {
            0
        }
    }
    pub(crate) fn valid(&self) -> bool {
        self.first_half <= 1000000
            && self.second_half <= 1000000
            && self.total <= 2000000
            && self.commuting <= 2000000
            && self.miles() + i64::from(self.commuting) <= i64::from(self.total)
            && [self.parking_tolls, self.interest, self.property_tax]
                .iter()
                .all(|v| (0..=100000000000).contains(v))
            && (self.owned || self.interest == 0)
            && (self.miles() > 0 || self.parking_tolls == 0)
    }
}
