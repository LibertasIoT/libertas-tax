// Coverage: Form 4562 — Supported (audited 2026-09-25).
//   Lines: Part I §179 (dollar and business-income limits, carryforwards oldest-first, MFS
//   allocation); Part II bonus by acquisition date with election out; Part III GDS 3–20-year
//   classes from published tables, half-year/mid-quarter conventions, SL switch, AMT 150% refigure;
//   27.5-year residential mid-month; Part VI startup/organizational and §197 amortization. Refused
//   (doc guards): listed property other than vehicles (Part V; vehicles are calculated, see Unit
//   C19a below), ADS (except 40-year AMT for pre-1999 rentals),
//   nonresidential real property, self-constructed/related-party, conversions,
//   dispositions/recapture, short year.
//   Refused explicitly (units A6 and A19, from the 4562 map): dispositions (AssetDisposed)
//   and the 2026 draft classes 14b/14c/14d/19g/19h/19k (SpecialDepreciation), return-level
//   situations since M1-P3 (the asset answers disposed_this_year/special_class are removed).
//   Nonresidential property
//   is 19j on the 2026 draft; amortization moved to Form 4562-B.
//   Evidence: depreciation.rs, section179.rs, kernel/cost_recovery/macrs_tables.rs, startup.rs, amortization.rs,
//   extended_rules.rs; tests: section179_precedes_bonus_macrs_and_preserves_ubia,
//   section179_home_office_follows_two_income_limit_order,
//   rental_depreciation_midmonth_land_and_final_year (business.rs),
//   macrs_published_schedules_cover_all_methods_classes_and_conventions (general.rs),
//   continuing_macrs_matches_irs_declining_balance_switch_and_final_recovery (repayment.rs); Pub
//   946 tables, i4562, Pub 527
//   Unit C19a (2026-09-25): BusinessAsset.listed (No { business_percent } | Vehicle { total_miles, business_miles,
//   operating_costs, passenger_automobile }); business use from exact miles; use of 50% or
//   less takes ADS straight-line with no bonus; the section 280F caps (year_constants.rs PASSENGER_AUTOMOBILE_LIMITS_2025_CENTS/_2026_CENTS by
//   placed-in-service year, Rev. Proc. 2025-16 and 2026-15 Tables 1 and 2) apply to the business share across 179, bonus and MACRS;
//   Part V lines 25-27 columns (a), (c), (d), (e), (h), (i) emitted; a vehicle uses mileage or actual
//   expenses, never both. Refused by name: pre-2025 passenger automobiles, manual-formula and
//   post-recovery safe-harbor cases, heavy-SUV 179, gambling vehicles. Section B/C and lines 7,
//   21, 28, 29 not emitted. STORAGE_FORMAT 17. Tests: src/tests/cost_recovery/business.rs
//   vehicle_actual_expenses_apply_280f_limits_and_business_use, listed_property_below_50_percent_uses_straight_line,
//   vehicle_uses_mileage_or_actual_never_both, vehicle_280f_first_year_limit_caps_bonus_on_business_share,
//   vehicle_280f_limit_wins_over_macrs_without_bonus, listed_property_at_exactly_50_percent_uses_straight_line,
//   passenger_automobile_section179_above_first_year_limit_is_refused.
//   Line audit D10-D11 (unit F6, 2026-10-10): amortization is Form 4562-B (lines 1(c), 1(e), 1(f) for amortization
//   that begins in 2026, line 2 for earlier starts, line 3 the total; startup.rs, amortization.rs) and never calls for
//   Form 4562 (assets_need_form_4562: 2026 assets, section 179, vehicles). Listed-property recapture when a continuing
//   vehicle's business use falls to 50% or less (BusinessAsset::recapture, section 280F(b)(2), Form 4797 Part IV):
//   see the Schedule C block in federal_rules.rs. Tests: src/tests/cross/line_audit_c_se.rs d10_*, d11_*.
//   Exchanged basis (unit B2, 2026-10-09): a rental exchanged for other real property continues on
//   its entry. The exchanged basis (Form 8824 line 25a up to the building given up's adjusted
//   depreciable basis) keeps each building's remaining recovery period (the longer of the two
//   when the new building has one) from the new building's first rental day, Form 4562 line 17;
//   the excess basis is new property placed in service then, lines 19i/19j; a failed exchange
//   depreciates the property received at cost. Reg. 1.168(i)-6(b)-(d); the election out (i) is
//   not offered. Evidence: exchange.rs replacement_depreciation; src/tests/investment/exchange.rs.
//   Per-year 280F tables (2026-10-08): vehicle_280f_second_year_uses_the_placed_in_service_year_table,
//   vehicle_280f_first_year_limits_for_both_tables_in_2026; a 2025 vehicle uses Rev. Proc. 2025-16.
//   Vehicle loan interest (2026-10-09): ListedProperty::Vehicle.loan_interest (whole-vehicle annual
//   interest) is allocated by business miles to Schedule C line 16b / Schedule F line 21b in the
//   same business_<id>_vehicle_loan_interest row as the standard-mileage vehicles' share (U5), and
//   out of line 9 / 10; the total is unchanged. A Form 2106 employee vehicle must keep it at zero.
//   Test: src/tests/cost_recovery/business.rs schedule_c_actual_vehicle_loan_interest_is_line_16b_with_totals_unchanged.
//   Form 2106 vehicles (2026-10-09): employee_expenses.rs reuses BusinessAsset for lines 30-38 and
//   joins them to assets() for the mid-quarter and bonus-consistency tests (section 179 left out).
//   Unit C19b (2026-09-25): EquipmentClass::ThirtyNine (nonresidential real property, Pub 946 Table
//   A-7a monthly first-year rates shared with rentals, no 179/bonus, outside the mid-quarter test),
//   tagged Part III line 19j; residential rentals tagged 19i. STORAGE_FORMAT 18. Since the
//   acceptance fix of 2026-09-26 (interview.rs block), lines are tagged only when the activity
//   must file Form 4562 (assets_need_form_4562), earlier-year assets and buildings on line 17. Test: src/tests/cost_recovery/business.rs
//   nonresidential_property_depreciates_over_39_years_mid_month.
//   T2-35 (2026-09-26): BusinessAsset.disposal; a continuing asset disposed of in 2026 takes the
//   disposal-convention share of its full-year MACRS (raw_macrs_amount); see section1231.rs.
//   C-fixture (2026-09-25, src/tests/cost_recovery/business.rs): depreciation_return_matches_hand_computation (Parts I-IV
//   with section 179, a half-year 7-year asset and a 39-year building by hand),
//   mid_quarter_convention_applies_above_40_percent_in_the_fourth_quarter,
//   section179_business_income_limit_carries_the_excess_forward, special_depreciation_classes_stay_refused.
//   T2-29b (2026-09-26): Part I takes Schedule K-1 (1065) box 12 and (1120-S) box 11 as line 6
//   items (section179.rs pass_throughs; Reg. 1.179-2(b)(3), 1.179-3(e)); line 11 includes the
//   owner's materially participated K-1 box 1 net (Reg. 1.179-2(c)(2)(v)); line 13 carryover per
//   K-1. See partnership.rs and scorp.rs for the basis, at-risk, Schedule E, QBI and SE routing.
//   T2-31p (2026-09-27, question C13): line 10 takes ScorpK1.section179_carryover (the owner's
//   2025 line 13 carryover of that K-1) after the current elections, within the dollar and line
//   11 limits; it is not a line 8 election and does not reduce basis again; the allowed part joins
//   the K-1's Schedule E column (j) and QBI reduction, and line 13 carries the rest. Tests:
//   src/tests/cost_recovery/passthrough.rs shareholder_section179_carryover_joins_form_4562_line_10_by_hand,
//   shareholder_section179_carryover_equal_to_the_income_limit_is_allowed_in_full,
//   shareholder_section179_carryover_one_dollar_above_the_income_limit_carries_again.
//   Unit form-3468 (2026-10-09): BusinessAsset.investment_credit (Form 3468). A property that carries
//   an investment credit has the section 50(c) reduction the App computes in place of the entered
//   credit_reduction: all of the rehabilitation credit determined (20% of the cost of a 39-year
//   building row) or half of an energy credit (30%/6% plus bonuses of the cost less any section 179
//   deduction); business_basis() is the business share of the cost less that reduction, and every
//   depreciation base (table and manual MACRS, bonus, the 39-year rate, the AMT projection, the
//   section 179 and prior-depreciation limits) reads it. The entered field stays for credits the App
//   does not compute. A rental improvement's depreciation (extended_rules.rs depreciation_for_year)
//   subtracts the same reduction. Tests: src/tests/credits/investment_credit.rs fixtures B, D.
//   Validation audit F04 (2026-10-13): business use has one source per property:
//   ListedProperty::No owns the nonvehicle percentage; Vehicle owns exact positive
//   business/total miles, including use below 1%. No rounded percentage is stored
//   or used for vehicle basis, operating costs, depreciation, caps or eligibility.
//   Tests: f04_actual_vehicle_keeps_fraction_below_one_percent,
//   f04_actual_vehicle_rejects_zero_or_excess_miles_without_calculation (business.rs),
//   f04_actual_vehicle_saves_reopens_and_rejects_without_changing_answers (interview.rs).

use crate::kernel::phase;
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Purchased business or investment equipment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct BusinessAsset {
    /// Canonical property identity assigned on Save
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Original schedules and annual use needed for historical recovery
    /// [NullText]
    /// No changed-use or original annual schedule is supplied
    /// [SetText]
    /// The original schedule and annual records are
    #[libertas_default($null)]
    pub history_details: Option<crate::HistoricalAssetRecords>,
    /// Section 179 description (Form 4562 line 6)
    /// The description typed with a section 179 election; empty for every other asset, which a note names by its position.
    pub description: String,
    /// Original purchase cost, including capitalized acquisition and installation costs
    /// Do not also enter this amount in operating expenses, inventory, home-office costs or another asset record.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub cost: i64,
    /// Credit that reduces its cost
    /// Leave zero unless a tax credit was taken for this property. A clean-vehicle or fuel-cell vehicle credit (sections 30D(f)(1), 30B(h)(4) and 45W(d)(1)) and an investment credit (section 50(c)(1)) reduce its cost by the whole credit; an energy credit under section 48 reduces it by half the credit (section 50(c)(3)). Enter the reduction itself. It cannot be more than the cost.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub credit_reduction: i64,
    /// Acquisition date
    /// For a written binding purchase contract, use the acquisition date under that contract, not merely the delivery or payment date. Self-constructed property needs separate rules.
    #[libertas_date_only]
    pub acquired: u32,
    /// Original date ready and available for income-producing use
    #[libertas_date_only]
    pub placed: u32,
    /// Business use in earlier years, when it was more than 50%
    /// Only for an asset placed in service before 2026 whose business use this year is 50% or less. Enter the whole percentage it was used for business in the earlier years, the same in each. The App adds back the section 179 benefit (Form 4797 Part IV column (a)) or, for a vehicle, the depreciation allowed above straight-line (column (b)) as other income of the business, and raises the asset's basis by it. Leave empty if it was used 50% or less in every earlier year.
    /// [NullText]
    /// Business use did not fall from above 50% to 50% or less
    /// [SetText]
    /// Earlier business use above 50% was
    // Percentage encoding: business use cannot exceed 100 percent.
    #[libertas_number(min = 51, max = 100)]
    pub earlier_business_percent: Option<u32>,
    /// MACRS recovery class
    /// Use the IRS activity/asset classification, not an estimated useful life. Computers generally use 5 years and office furniture 7 years. Specialized equipment follows its assigned activity class. Nonresidential real property (a building or structural component used in this business) is 39-year straight-line with the mid-month convention; enter only the depreciable building cost, excluding land.
    pub class: EquipmentClass,
    /// Depreciation history
    pub history: EquipmentHistory,
    /// GDS method for basis remaining after section 179 and bonus depreciation
    /// For new assets, apply the same method to this recovery class across businesses and nonbusiness rentals. For continuing assets, enter the original GDS method from the depreciation record; the App applies the required switch to straight line. With 100% bonus there is no remaining basis after section 179. Ignored for nonresidential real property, which is always 39-year straight-line.
    pub method: EquipmentMethod,
    // The pages that write this record ask only for qualifying purchased
    // equipment; the scope the calculation supports is checked by qualifies().
    /// Business or investment use
    /// For a car, van, pickup or panel truck whose actual costs you deduct, enter its exact business and total miles. Other property uses its documented percentage.
    pub listed: ListedProperty,
    /// Sold or stopped using in 2026
    #[libertas_default(No)]
    pub disposal: crate::AssetDisposal,
    /// Investment credit (Form 3468)
    #[libertas_default(No)]
    pub investment_credit: crate::InvestmentCredit,
    /// Part of this building replaced or removed in 2026
    /// Only for a building (39-year): a roof, HVAC system or other structural component. Include partial sales and other mandatory dispositions. For replacement or removal, enter the component when electing separate partial-disposition treatment; otherwise choose no separately recognized disposition.
    /// [NullText]
    /// No building-component disposition is recognized separately in 2026
    /// [SetText]
    /// The separately recognized building-component disposition is
    pub partial: Option<crate::PartialDisposition>,
    /// Actual AMT unused elected cost; never inferred from the regular balance
    #[libertas_hidden]
    #[libertas_default($null)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amt_section179_income_carry: Option<i64>,
}

/// Vehicle deducted with actual expenses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ListedProperty {
    /// Not a vehicle
    No {
        /// Business or investment use percentage
        /// Personal use is not deductible. Nonbusiness rentals require 100% investment use. For continuing property whose use fell to 50% or less this year, enter this year's percentage here and the earlier percentage above.
        // Percentage encoding: business use cannot exceed 100 percent.
        #[libertas_number(min = 1, max = 100)]
        business_percent: u32,
    },
    /// Vehicle using actual expenses
    Vehicle {
        /// All miles driven during 2026
        /// Include business, commuting and personal miles from the odometer records.
        #[libertas_number(min = 1)]
        total_miles: u32,
        /// Business miles driven during 2026
        /// Exclude commuting, personal travel and miles claimed by another business. Every business-use allocation uses these miles divided by all miles, without rounding that fraction to a percentage.
        #[libertas_number(min = 1)]
        business_miles: u32,
        /// Fuel, insurance, repairs, registration and other operating costs for 2026
        /// Enter the whole-vehicle amount. The App deducts the business-mile share on the car and truck line. Do not enter loan interest, business parking and tolls, lease payments or depreciation here.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        operating_costs: i64,
        /// Annual vehicle-loan interest paid
        /// Enter the whole vehicle amount. The App allocates its business-mile share to the loan-interest line; do not deduct that share elsewhere. Enter zero when there is no loan.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        loan_interest: i64,
        /// Passenger automobile subject to the annual depreciation limits
        /// Yes for a car or a truck or van rated at 6,000 pounds gross vehicle weight or less. No only for a qualified nonpersonal-use vehicle or a heavier truck or van; section 179 for those vehicles is not calculated.
        passenger_automobile: bool,
    },
}

/// Form 4797 Part IV lines 35 and 36 of a vehicle whose business use fell to 50% or less.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ListedRecapture {
    /// Depreciation, with section 179, allowed for the earlier years (line 35).
    pub allowed: i64,
    /// Straight-line depreciation those years would have allowed (line 36).
    pub recomputed: i64,
}
impl ListedRecapture {
    /// The excess depreciation included in gross income (line 37).
    pub(crate) fn excess(&self) -> i64 {
        crate::kernel::recapture::deduction_excess(self.allowed, self.recomputed)
    }
}

// Section 280F(a)(1) passenger-automobile limits: see year_constants.rs
// PASSENGER_AUTOMOBILE_LIMITS_2025_CENTS / _2026_CENTS.

// Shared published building schedule; native callers retain their existing stages.
pub(crate) use crate::kernel::cost_recovery::historical::{
    nonresidential_closing, nonresidential_rate,
};

impl BusinessAsset {
    fn vehicle(&self) -> Option<(i64, i64, i64, bool)> {
        match self.listed {
            ListedProperty::No { .. } => None,
            ListedProperty::Vehicle {
                total_miles,
                business_miles,
                operating_costs,
                passenger_automobile,
                ..
            } => Some((
                i64::from(total_miles),
                i64::from(business_miles),
                operating_costs,
                passenger_automobile,
            )),
        }
    }
    pub(crate) fn business_share_of(&self, amount: i64) -> i64 {
        self.business_share(amount)
    }
    pub(crate) fn amt_projection_or_self(&self) -> Self {
        self.amt_projection().unwrap_or_else(|| self.clone())
    }
    pub(crate) fn is_vehicle(&self) -> bool {
        self.vehicle().is_some()
    }
    // Share of an amount attributable to business use: exact business miles
    // over all miles for a vehicle, the entered percentage otherwise.
    fn business_share(&self, amount: i64) -> i64 {
        match self.listed {
            ListedProperty::Vehicle { total_miles, business_miles, .. } => {
                crate::kernel::ratio::fraction(
                    amount,
                    i64::from(business_miles),
                    i64::from(total_miles),
                )
            }
            ListedProperty::No { business_percent } => {
                crate::kernel::ratio::fraction(amount, i64::from(business_percent), 100)
            }
        }
    }
    // More than 50% qualified business use (Pub. 946 chapter 5); a vehicle's
    // test uses exact miles.
    pub(crate) fn predominant(&self) -> bool {
        match self.listed {
            ListedProperty::Vehicle { total_miles, business_miles, .. } => {
                u64::from(business_miles) * 2 > u64::from(total_miles)
            }
            ListedProperty::No { business_percent } => business_percent > 50,
        }
    }
    // Listed property used 50% or less must use ADS straight line (5 years for
    // automobiles and light trucks), with no section 179 and no bonus.
    pub(crate) fn ads(&self) -> bool {
        self.is_vehicle() && !self.predominant()
    }
    fn effective_method(&self) -> EquipmentMethod {
        if self.ads() { EquipmentMethod::StraightLine } else { self.method }
    }
    // Business-mile share of the vehicle's operating costs (Schedule C line 9).
    pub(crate) fn vehicle_operating(&self) -> i64 {
        self.vehicle().map_or(0, |(_, _, operating, ..)| self.business_share(operating))
    }
    // The business-mile share of the vehicle-loan interest (Schedule C line
    // 16b, Schedule F line 21b): the allocation the standard-mileage vehicles
    // make with `BusinessMileage::loan_interest_share`.
    pub(crate) fn vehicle_loan_interest(&self) -> i64 {
        match self.listed {
            ListedProperty::No { .. } => 0,
            ListedProperty::Vehicle { loan_interest, .. } => self.business_share(loan_interest),
        }
    }
    // Recapture of excess depreciation when business use of listed property falls to
    // 50% or less (section 280F(b)(2); Form 4797 Part IV lines 35-37, column (b)).
    // The depreciation allowed through 2025 (the entered prior depreciation, with any
    // section 179 and bonus) is compared with the straight-line ADS depreciation that
    // the earlier years would have allowed at the earlier business use (5-year
    // property under the original convention, the section 280F caps of a passenger
    // automobile applied to each year). None when the vehicle is not a continuing one
    // used more than 50% earlier and 50% or less now.
    pub(crate) fn recapture(&self) -> Option<ListedRecapture> {
        let percent = i64::from(self.earlier_business_percent?);
        let EquipmentHistory::Continuing { prior_depreciation, mid_quarter, .. } = self.history
        else {
            return None;
        };
        if !self.ads() {
            return None;
        }
        let (_, _, _, passenger) = self.vehicle()?;
        let basis = crate::kernel::ratio::fraction(self.cost - self.credit_reduction, percent, 100);
        // Straight line, 5-year property: Tables A-8 to A-12 (method 2).
        let convention = if mid_quarter {
            1 + (self.placed / 100 % 100).saturating_sub(1).min(11) as usize / 3
        } else {
            0
        };
        let rates = crate::kernel::cost_recovery::macrs_tables::RATES[2 * 30 + convention * 6 + 1];
        let first_year = self.placed / 10000;
        let table = match first_year {
            2025 => &crate::year_constants::PASSENGER_AUTOMOBILE_LIMITS_2025_CENTS,
            _ => &crate::year_constants::PASSENGER_AUTOMOBILE_LIMITS_2026_CENTS,
        };
        let recomputed = (first_year..2026)
            .map(|year| {
                let index = (year - first_year) as usize;
                let ads = crate::kernel::ratio::fraction(
                    basis,
                    rates.get(index).copied().unwrap_or(0),
                    100_000,
                );
                if passenger {
                    // No bonus under ADS, so the first year uses the limit without it.
                    let limit = table[(index + 1).min(4)];
                    phase::cap(ads, [crate::kernel::ratio::fraction(limit, percent, 100)])
                } else {
                    ads
                }
            })
            .sum();
        Some(ListedRecapture { allowed: prior_depreciation, recomputed })
    }
    // Whether the asset was kept for personal use in 2026 (Reg. 1.168(i)-4(c)).
    fn converted(&self) -> bool {
        matches!(
            self.disposal,
            crate::AssetDisposal::Disposed { how: crate::DisposalKind::PersonalUse, .. }
        )
    }
    // Recapture of a section 179 deduction when the property stops being used
    // predominantly in business before the end of its recovery period (section
    // 179(d)(10), Reg. 1.179-1(e); Form 4797 Part IV lines 35-37, column (a)):
    // either it is kept for personal use, or its business use fell to 50% or
    // less (the earlier use, above 50%, is entered). Line 35 is the deduction
    // taken (a carryover not yet deducted is no longer available and is no
    // amount expensed, Reg. 1.179-3(f)(2)); line 36 is the depreciation that
    // would have been allowable on that amount from the year placed in service
    // through and including 2026 (to the date, for a conversion); line 37, the
    // difference, is other income of the business and raises the basis. A
    // vehicle uses column (b), `recapture()`; a section 179 deduction on
    // property placed in service in 2026 and converted in 2026 has no
    // depreciation that would have been allowable.
    pub(crate) fn section179_recapture(&self) -> Option<ListedRecapture> {
        if self.is_vehicle() || self.class == EquipmentClass::ThirtyNine {
            return None;
        }
        let converted = self.converted();
        match self.history {
            EquipmentHistory::New { section179, .. } => (converted && section179.elected() > 0)
                .then(|| ListedRecapture { allowed: section179.elected(), recomputed: 0 }),
            EquipmentHistory::Continuing {
                initial_section179,
                carryforward,
                mid_quarter,
                schedule,
                ..
            } => {
                let expensed = initial_section179 - carryforward;
                if expensed <= 0
                    || self.remaining_eighths(mid_quarter) <= 0
                    || !(converted
                        || (self.earlier_business_percent.is_some() && !self.predominant()))
                {
                    return None;
                }
                Some(ListedRecapture {
                    allowed: expensed,
                    recomputed: self.allowable_on(expensed, mid_quarter, schedule),
                })
            }
        }
    }
    // The depreciation that would have been allowable on `amount` (cents) from
    // the year placed in service through 2026 under the asset's original
    // method and convention: the published table rates, or the manual formula
    // (declining balance switching to straight line). For a conversion in 2026
    // the last year is the convention's share to the date.
    fn allowable_on(&self, amount: i64, mid_quarter: bool, schedule: EquipmentSchedule) -> i64 {
        let first = self.placed / 10000;
        let share = |value: i64| match self.disposal {
            crate::AssetDisposal::Disposed { date, how: crate::DisposalKind::PersonalUse } => {
                crate::kernel::ratio::fraction(
                    value,
                    crate::asset_sale::disposal_half_months(
                        date,
                        if mid_quarter {
                            crate::asset_sale::Convention::MidQuarter
                        } else {
                            crate::asset_sale::Convention::HalfYear
                        },
                    ),
                    24,
                )
            }
            _ => value,
        };
        match schedule {
            EquipmentSchedule::Tables { .. } => {
                let rates = self.table_rates(mid_quarter);
                (first..=2026)
                    .map(|year| {
                        let rate = rates.get((year - first) as usize).copied().unwrap_or(0);
                        let value = crate::kernel::ratio::fraction(amount, rate, 100_000);
                        if year == 2026 { share(value) } else { value }
                    })
                    .sum()
            }
            EquipmentSchedule::Manual => {
                let years = self.class.years();
                let factor = match self.effective_method() {
                    EquipmentMethod::Regular if years <= 10 => 200,
                    EquipmentMethod::Regular | EquipmentMethod::Declining150 => 150,
                    _ => 100,
                };
                let eighths = if mid_quarter {
                    7 - 2 * i64::from((self.placed / 100 % 100).saturating_sub(1) / 3).min(3)
                } else {
                    4
                };
                let mut remaining = amount;
                let mut total = 0;
                for year in first..=2026 {
                    let value = if year == first {
                        crate::kernel::ratio::fraction(amount, factor * eighths, 800 * years)
                    } else {
                        let left = years * 8 - eighths - i64::from(year - first - 1) * 8;
                        if left <= 0 {
                            0
                        } else {
                            let straight =
                                crate::kernel::ratio::fraction(remaining, 8, left.max(8));
                            if self.effective_method() == EquipmentMethod::StraightLine {
                                straight
                            } else {
                                phase::cap(
                                    phase::floor(
                                        straight,
                                        crate::kernel::ratio::fraction(
                                            remaining,
                                            factor,
                                            100 * years,
                                        ),
                                    ),
                                    [remaining],
                                )
                            }
                        }
                    };
                    let value = phase::cap(value, [remaining]);
                    remaining = crate::kernel::carry::native_balance(remaining, value);
                    total += if year == 2026 { share(value) } else { value };
                }
                total
            }
        }
    }
    // The whole-vehicle loan interest as entered, for the rules that forbid it.
    pub(crate) fn vehicle_loan_interest_entered(&self) -> i64 {
        match self.listed {
            ListedProperty::No { .. } => 0,
            ListedProperty::Vehicle { loan_interest, .. } => loan_interest,
        }
    }
    // Business-use percentage in hundredths of a percent, for Part V column (c).
    pub(crate) fn business_use_hundredths(&self) -> i64 {
        self.business_share(10000)
    }
    // The section 280F cap on section 179, bonus and MACRS together, applied to
    // the business-use share; None when no cap applies.
    pub(crate) fn automobile_limit(&self) -> Option<i64> {
        let (_, _, _, passenger) = self.vehicle()?;
        if !passenger {
            return None;
        }
        if let Some(history) = self.history_details.as_ref().and_then(|h| h.auto_history.as_ref()) {
            return crate::year_constants::original_auto_limit_cents(
                self.acquired,
                self.placed,
                2026,
                &history.car_or_truck,
                self.raw_bonus_amount() > 0,
            )
            .map(|amount| self.business_share(amount));
        }
        let year = 2026_u32.saturating_sub(self.placed / 10000);
        let index = match year {
            0 if self.raw_bonus_amount() > 0 => 0,
            0 => 1,
            1 => 2,
            2 => 3,
            _ => 4,
        };
        let table = match self.placed / 10000 {
            2025 => &crate::year_constants::PASSENGER_AUTOMOBILE_LIMITS_2025_CENTS,
            _ => &crate::year_constants::PASSENGER_AUTOMOBILE_LIMITS_2026_CENTS,
        };
        Some(self.business_share(table[index]))
    }
    // Room under the section 280F cap for bonus and MACRS after the elected
    // section 179 cost, which the cap includes.
    pub(crate) fn limit_room(&self) -> i64 {
        self.automobile_limit().map_or(i64::MAX, |cap| phase::floor(cap - self.elected179(), 0))
    }
    pub(crate) fn vehicle_valid(&self) -> bool {
        match self.listed {
            ListedProperty::No { business_percent } => (1..=100).contains(&business_percent),
            ListedProperty::Vehicle {
                total_miles,
                business_miles,
                operating_costs,
                loan_interest,
                ..
            } => {
                (1..=2000000).contains(&total_miles)
                    && (1..=total_miles).contains(&business_miles)
                    && (0..=100000000000).contains(&operating_costs)
                    && (0..=100000000000).contains(&loan_interest)
            }
        }
    }
    // The vehicle situations this App calculates; others are refused by name.
    pub(crate) fn vehicle_supported(&self) -> bool {
        let Some((_, _, _, passenger)) = self.vehicle() else {
            return true;
        };
        if crate::auto_recovery::amount(self, 2026).is_some() || self.documented_179_reserve() {
            return true;
        }
        self.class == EquipmentClass::Five
            && match self.history {
                EquipmentHistory::New { section179, .. } => {
                    section179.elected() == 0 || (passenger && self.predominant())
                }
                EquipmentHistory::Continuing {
                    initial_section179, carryforward, schedule, ..
                } => {
                    carryforward == 0
                        && (initial_section179 == 0 || self.predominant())
                        && (!passenger
                            || (self.placed >= 20250101
                                && schedule == EquipmentSchedule::Tables { initial_bonus: 0 }))
                }
            }
    }
}

impl BusinessAsset {
    // The asset as a note names it: the typed description, else its position
    // in the record's list (F3, 2026-10-09): "Asset n" among the property and
    // "Vehicle n" among the actual-expense vehicles, counted after the
    // `mileage` standard-mileage vehicles, the order the page numbers them.
    pub(crate) fn display_arg(
        assets: &[BusinessAsset],
        index: usize,
        mileage: usize,
    ) -> crate::MessageArg {
        let asset = &assets[index];
        if !asset.description.trim().is_empty() {
            return crate::MessageArg::Text(asset.description.clone());
        }
        let vehicle = asset.is_vehicle();
        let before = assets[..index].iter().filter(|other| other.is_vehicle() == vehicle).count();
        crate::MessageArg::Nested(alloc::boxed::Box::new(crate::Message::new(
            if vehicle { "LABEL_VEHICLE_N" } else { "LABEL_ASSET_N" },
            alloc::vec![crate::MessageArg::Number(
                (before + if vehicle { mileage } else { 0 }) as i64 + 1
            )],
        )))
    }
    #[cfg(test)]
    pub(crate) fn display_text(assets: &[BusinessAsset], index: usize, mileage: usize) -> String {
        crate::Message::new("TEXT", alloc::vec![Self::display_arg(assets, index, mileage)])
            .english()
    }
    pub(crate) fn qualifies_nonbusiness_rental(&self) -> bool {
        self.qualifies()
            && !self.is_vehicle()
            && matches!(self.listed, ListedProperty::No { business_percent: 100 })
            && match self.history {
                EquipmentHistory::New { section179, .. } => {
                    matches!(section179, Section179::Ineligible)
                }
                EquipmentHistory::Continuing { initial_section179, carryforward, .. } => {
                    initial_section179 == 0 && carryforward == 0
                }
            }
    }
    pub(crate) fn valid(&self) -> bool {
        self.id >= 0
            && (1..=100000000000).contains(&self.cost)
            && (0..=self.cost).contains(&self.credit_reduction)
            // Validate the exact use fraction before any basis calculations.
            && self.vehicle_valid()
            && self.investment_credit.valid()
            && self.investment_shape_ok()
            && self.amt_section179_carry_fact_valid()
            && (self.history_details.as_ref().is_none_or(|h| h.auto_history.is_none())
                || self.documented_179_reserve()
                || crate::auto_recovery::amount(self, 2026).is_some())
            && crate::federal_rules::valid_birth(self.acquired)
            && crate::federal_rules::valid_birth(self.placed)
            && self.disposal.valid()
            // The earlier use matters only for a continuing asset whose use
            // fell to 50% or less this year.
            && self.earlier_business_percent.is_none_or(|percent| {
                (51..=100).contains(&percent)
                    && matches!(self.history, EquipmentHistory::Continuing { .. })
                    && !self.predominant()
            })
            && self.partial.as_ref().is_none_or(|part| {
                self.class == EquipmentClass::ThirtyNine
                    && (20260101..=20261231).contains(&part.date)
                    && crate::federal_rules::valid_date(part.date)
                    && part.date > self.placed
                    && (1..self.cost).contains(&part.cost_of_part)
                    && (0..=100000000000).contains(&part.proceeds)
            })
            && match self.history {
                EquipmentHistory::New { section179, .. } => {
                    (0..=self.business_basis()).contains(&section179.elected())
                }
                EquipmentHistory::Continuing {
                    prior_depreciation,
                    initial_section179,
                    carryforward,
                    schedule,
                    ..
                } => {
                    self.amt_projection().is_none_or(|a| a.valid())
                        && (0..=self.history_basis()).contains(&prior_depreciation)
                        && (0..=self.history_basis() - prior_depreciation).contains(&carryforward)
                        && (carryforward..=prior_depreciation + carryforward)
                            .contains(&initial_section179)
                        && match schedule {
                            EquipmentSchedule::Manual => true,
                            EquipmentSchedule::Tables { initial_bonus } => {
                                (0..=prior_depreciation + carryforward - initial_section179)
                                    .contains(&initial_bonus)
                            }
                        }
                }
            }
    }
    pub(crate) fn qualifies(&self) -> bool {
        // Nonresidential real property is never section 179 or bonus
        // eligible, needs no AMT history (class.years() >= 15 already exempts
        // it below) and can never exhaust its 39-year period within the
        // supported 1999-2025 continuing-service window.
        let real_property = self.class == EquipmentClass::ThirtyNine;
        if crate::auto_recovery::amount(self, 2026).is_some() {
            return self.vehicle_valid();
        }
        if self.documented_179_reserve() {
            return self.vehicle_valid()
                && self.class == EquipmentClass::Five
                && self.disposal == crate::AssetDisposal::No
                && self.partial.is_none();
        }
        self.amt_projection().is_none_or(|a| a.qualifies())
            && self.acquired <= self.placed
            && match self.history {
                EquipmentHistory::New { section179, .. } => {
                    self.placed / 10000 == 2026
                        && (matches!(section179, Section179::Ineligible)
                            || ((self.predominant() || self.converted()) && !real_property))
                        && self.partial.is_none()
                }
                EquipmentHistory::Continuing {
                    initial_section179,
                    carryforward,
                    mid_quarter,
                    amt,
                    ..
                } => {
                    (19990101..20260101).contains(&self.placed)
                        // Post-1998 GDS 150% and straight line need no AMT
                        // refigure. The 200% path still requires the supported
                        // post-2015 bonus-eligible acquisition history.
                        && (self.acquired >= 20170928
                            || self.method != EquipmentMethod::Regular
                            || self.class.years() >= 15
                            || matches!(amt, EquipmentAmtHistory::Separate { .. }))
                        && match amt {
                            EquipmentAmtHistory::Same => true,
                            EquipmentAmtHistory::Separate { .. } => self.acquired < 20170928
                                && self.method == EquipmentMethod::Regular
                                && self.class.years() <= 10,
                        }
                        && (real_property
                            || self.remaining_eighths(mid_quarter) > 0
                            || self.opening_basis() == 0)
                        && (initial_section179 == 0
                            || (!real_property
                                && (self.predominant()
                                    || self.converted()
                                    || self.earlier_business_percent.is_some())))
                        && (carryforward == 0 || (!real_property && self.placed >= 20180101))
                        // A partial disposition is of a kept building; a building
                        // sold or converted in the same year is a whole disposition.
                        && (self.partial.is_none() || self.disposal == crate::AssetDisposal::No)
                }
            }
    }
    // Qualified property for the QBI unadjusted-basis limit (section 199A(b)(6),
    // 2025 i8995a, "Determining your UBIA"): held at the close of 2026 with a
    // depreciable period that has not ended before then. The period ends on the
    // later of ten years after the property was placed in service and the last
    // day of the last full calendar year of its recovery period (December 31
    // of the placed-in-service year plus the class years less one). A period
    // that ends on December 31, 2026 has not ended before the close of 2026.
    pub(crate) fn qbi_property(&self) -> bool {
        if self.disposal != crate::AssetDisposal::No {
            return false;
        }
        let (year, monthday) = (self.placed / 10000, self.placed % 10000);
        let ten_years = (year + 10) * 10000 + monthday;
        let recovery = (year + self.class.years() as u32 - 1) * 10000 + 1231;
        ten_years.max(recovery) >= 2026 * 10000 + 1231
    }
    pub(crate) fn elected179(&self) -> i64 {
        match self.history {
            EquipmentHistory::New { section179, .. } => section179.elected(),
            _ => 0,
        }
    }
    // Only newly admitted documented reserve sources need this annual cap;
    // every pre-existing source retains its exact native allocator operand.
    pub(crate) fn annual179_capacity(&self) -> i64 {
        if self.documented_179_reserve() {
            phase::cap(self.available179(), [self.automobile_limit().unwrap_or(i64::MAX)])
        } else {
            self.available179()
        }
    }
    pub(crate) fn available179(&self) -> i64 {
        match self.history {
            // The unused carryforward of an asset disposed of in 2026 is added
            // back to its basis and is not available as a deduction to the
            // transferor (Reg. 1.179-3(f)(1)); one kept for personal use is
            // no longer available for expensing (Reg. 1.179-3(f)(2)).
            EquipmentHistory::Continuing { .. } if self.disposal != crate::AssetDisposal::No => 0,
            EquipmentHistory::Continuing { carryforward, .. } => carryforward,
            _ => self.elected179(),
        }
    }
    pub(crate) fn documented_179_reserve(&self) -> bool {
        // This predicate is also used before native source admission. Establish
        // safe basis operands without recursing through valid().
        if !self.vehicle_valid()
            || self.cost < 0
            || !(0..=self.cost).contains(&self.credit_reduction)
            || !self.investment_shape_ok()
            || self.earlier_business_percent.is_some()
        {
            return false;
        }
        let EquipmentHistory::Continuing {
            initial_section179,
            carryforward,
            prior_depreciation,
            ..
        } = self.history
        else {
            return false;
        };
        let Some(h) = &self.history_details else {
            return false;
        };
        let Some(auto) = &h.auto_history else {
            return false;
        };
        let Some((total, business, _, true)) = self.vehicle() else {
            return false;
        };
        if initial_section179 < 0
            || prior_depreciation < 0
            || carryforward <= 0
            || prior_depreciation.checked_add(carryforward) != Some(self.history_basis())
        {
            return false;
        }
        let Some(original_cap) = crate::year_constants::original_auto_limit_cents(
            self.acquired,
            self.placed,
            self.placed / 10000,
            &auto.car_or_truck,
            false,
        ) else {
            return false;
        };
        let same_use = |fraction: crate::UseFraction| {
            fraction.total > 0
                && fraction.used <= fraction.total
                && i64::from(fraction.used) * total == business * i64::from(fraction.total)
        };
        self.placed < 20260101
            && self.acquired <= self.placed
            && self.automobile_limit().is_some()
            && self.predominant()
            && carryforward > 0
            && self.opening_basis() == 0
            && h.original_business_basis == self.history_basis()
            && h.original_section179_elected == initial_section179
            && initial_section179 == h.original_business_basis
            && h.original_bonus == 0
            && initial_section179 <= self.business_share(original_cap)
            && auto.first_year_capped_deduction >= 0
            && auto.first_year_capped_deduction
                == h.years
                    .iter()
                    .find(|year| year.year == self.placed / 10000)
                    .map_or(0, |year| year.section179_allowed)
            && auto.post_recovery_basis == 0
            && auto.first_year_bonus != crate::OriginalBonusTreatment::HundredPercent
            && auto.safe_harbor_adopted_year.is_none()
            && same_use(h.original_income_use)
            && same_use(h.original_qualified_use)
            && h.years.len() == (2026 - self.placed / 10000) as usize
            && h.years.iter().all(|year| {
                (self.placed / 10000..2026).contains(&year.year)
                    && h.years
                        .iter()
                        .filter(|other| other.year == year.year)
                        .count()
                        == 1
                    && same_use(year.income_use)
                    && same_use(year.qualified_use)
                    && year.section179_allowed >= 0
                    && year.section179_allowed <= initial_section179
                    && year.bonus == 0
                    && year.regular == 0
                    && year.amt == 0
                    && year.basis_increase == 0
                    && year.basis_decrease == 0
                    && year.recaptured == 0
            })
            && h.years
                .iter()
                .try_fold(0i64, |sum, row| sum.checked_add(row.section179_allowed))
                == Some(prior_depreciation)
            && h.carryovers
                .iter()
                .all(|carry| carry.year == self.placed / 10000 && carry.amount >= 0)
            // This source is one asset's original election, not a pooled history.
            && h.carryovers.len() == 1
            && h.carryovers[0].amount == carryforward
    }
    pub(crate) fn qualified179_cost(&self) -> i64 {
        if matches!(
            self.history,
            EquipmentHistory::New { section179: Section179::Eligible { .. }, .. }
        ) {
            self.business_basis()
        } else {
            0
        }
    }
    // The depreciable basis: the business share of the cost less the credit
    // reduction. A property carrying an investment credit (Form 3468) has the
    // section 50(c) reduction the App computes (100% of the rehabilitation
    // credit, 50% of an energy credit), which replaces the entered amount;
    // any other credit's reduction is the entered one.
    pub(crate) fn business_basis(&self) -> i64 {
        self.business_basis_on_cost(self.cost)
    }
    pub(crate) fn business_basis_on_cost(&self, cost: i64) -> i64 {
        if self.investment_credit.is_some() {
            return self.credit_cost_on(cost) - self.investment_basis_reduction_on(cost);
        }
        self.business_share(cost - self.credit_reduction)
    }
    // The entered credit reduction's business share (cents): a clean-vehicle
    // credit or an investment credit the App does not compute lowers the
    // depreciable basis, yet Form 4797 keeps the cost on line 23 and counts the
    // reduction as depreciation on line 24 (i4797 line 22, Step 1). Zero when
    // the App computes the investment credit's reduction itself.
    pub(crate) fn entered_credit_reduction(&self) -> i64 {
        if self.investment_credit.is_some() {
            return 0;
        }
        self.business_share(self.cost) - self.business_share(self.cost - self.credit_reduction)
    }
    // Section 1250 property (Pub 225 chapter 9): a building or its structural
    // components, the 15-year land improvements and the 20-year farm buildings.
    // A sale recaptures only its additional depreciation (Form 4797 line 28),
    // not all depreciation as section 1245 property does (line 27).
    pub(crate) fn section1250(&self) -> bool {
        matches!(
            self.class,
            EquipmentClass::ThirtyNine | EquipmentClass::Fifteen | EquipmentClass::Twenty
        )
    }
    // The section 179 deduction taken in the asset's first year (cents), which
    // a sale recaptures as section 1245 property even for section 1250 property.
    pub(crate) fn first_year_179(&self) -> i64 {
        match self.history {
            EquipmentHistory::Continuing { initial_section179, .. } => initial_section179,
            EquipmentHistory::New { section179, .. } => section179.elected(),
        }
    }
    // Straight-line depreciation (cents) of a continuing 15- or 20-year asset
    // on `base` from its service year through the sale on `sold`, by the
    // convention of the original schedule: the first year, the full years in
    // between and the disposal-year share, in half-months of the recovery
    // period (Pub 946 chapter 4), never more than the base. Form 4797 line 28a
    // subtracts it from the depreciation allowed.
    pub(crate) fn straight_line_through(&self, base: i64, sold: u32) -> i64 {
        let EquipmentHistory::Continuing { mid_quarter, .. } = self.history else {
            return 0;
        };
        let quarter = i64::from((self.placed / 100 % 100).clamp(1, 12) - 1) / 3;
        let (first, convention) = if mid_quarter {
            (21 - 6 * quarter, crate::asset_sale::Convention::MidQuarter)
        } else {
            (12, crate::asset_sale::Convention::HalfYear)
        };
        let life = self.class.years() * 24;
        let served = first
            + 24 * (2025 - i64::from(self.placed / 10000))
            + crate::asset_sale::disposal_half_months(sold, convention);
        crate::kernel::ratio::fraction(base, served.min(life), life)
    }
}

/// Equipment recovery class under GDS
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EquipmentClass {
    /// 3-year property
    Three,
    /// 5-year property
    Five,
    /// 7-year property
    Seven,
    /// 10-year property
    Ten,
    /// 15-year property
    Fifteen,
    /// 20-year property
    Twenty,
    /// 39-year nonresidential real property
    ThirtyNine,
}
impl EquipmentClass {
    fn years(self) -> i64 {
        match self {
            Self::Three => 3,
            Self::Five => 5,
            Self::Seven => 7,
            Self::Ten => 10,
            Self::Fifteen => 15,
            Self::Twenty => 20,
            Self::ThirtyNine => 39,
        }
    }
}

/// Section 179 property classification and election
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum Section179 {
    /// Property does not qualify for section 179
    Ineligible,
    /// Qualifying purchased equipment used predominantly in an active business
    /// Confirm eligible tangible section 179 property, more than 50% business use, eligible purchase and domestic active-business use. All eligible purchases and active-business income are included; unentered working-capital interest, section 1231 gains/losses, entity allocations or amended separate-to-joint elections need their own treatment. No excluded land improvement, special noncorporate lease, tax-exempt/foreign use, like-kind basis or other exception. Include all eligible purchases even when electing zero: they count toward the annual phaseout.
    Eligible {
        /// Business cost elected for section 179
        /// Enter your chosen amount, including zero. It cannot exceed business cost or the shared annual dollar limit. The App applies the business-income limit and uses current elections first, then prior carryforwards oldest year first, with proportional asset allocation within each year. Undeducted elected cost remains a separate carryforward, never bonus or MACRS basis. Retain the election and asset allocation with Form 4562.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        elected: i64,
    },
}
impl Section179 {
    pub(crate) fn elected(self) -> i64 {
        match self {
            Self::Ineligible => 0,
            Self::Eligible { elected } => elected,
        }
    }
}

/// Equipment depreciation history
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EquipmentHistory {
    /// First placed in service in 2026
    New {
        /// Section 179 classification and elected business cost
        section179: Section179,
        /// Bonus depreciation election for this recovery class
        /// Apply the same election to bonus-eligible 2026 property in this class across your return. The acquisition date determines eligibility and percentage; equipment acquired before September 28, 2017 receives no bonus. Retain any election out with the timely return.
        bonus: BonusElection,
        /// Use IRS percentage tables
        /// Yes preserves the published table percentages, including their rounding. No uses the manual MACRS formula. Keep the chosen approach with this asset's depreciation records for later years.
        tables: bool,
    },
    /// Placed in service before 2026
    Continuing {
        /// Original section 179 elected cost
        /// Copy the elected business cost from the original asset schedule, or zero. It equals amounts already included in Prior depreciation plus any unused section 179 carryforward below. No recapture or change in business use. Do not enter a current-year election or deduct this historical amount again.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        initial_section179: i64,
        /// Unused section 179 carryforward attributable to this asset through 2025
        /// Use the prior Form 4562 and asset allocation records. This is not a QBI/NOL carryover or remaining MACRS basis. Enter zero when fully deducted. The asset remains owned in the same active business; no disposition, allocation change, recapture or unresolved suspended-loss limitation. The original suspended deduction must be wholly attributable to this qualified business for QBI when released. Pre-2018 amounts, mixed qualified/nonqualified allocations or changed specified-service history need separate QBI treatment.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        carryforward: i64,
        /// Prior depreciation allowed or allowable through 2025
        /// Copy the cumulative income-producing amount from the asset's depreciation records, including previously allowed amounts of the original section 179 election and first-year bonus. This reduces original income-producing basis; do not enter the remaining basis or deduct this amount again. If earlier returns claimed a different amount, enter the allowable amount here and the difference as a section 481(a) adjustment entered under Accounting-method changes (Form 3115, DCN 7).
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        prior_depreciation: i64,
        /// AMT depreciation records
        amt: EquipmentAmtHistory,
        /// Original convention was mid-quarter
        /// Use the convention on the original depreciation schedule. No means half-year. The App uses the original service quarter, not the 2026 purchase mix.
        mid_quarter: bool,
        /// Original computation approach
        /// Keep the approach used on the prior depreciation schedule. Table percentages apply to the original depreciable income-producing basis after original section 179 and first-year bonus, not the current remaining basis.
        schedule: EquipmentSchedule,
    },
}

/// Alternative minimum tax asset history
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EquipmentAmtHistory {
    /// Same depreciation and basis as regular tax
    Same,
    /// Separate post-1998 150% declining-balance history
    /// For non-section-1250 equipment using regular 200% declining balance that requires AMT refiguring. Use the same recovery period and convention. Not property covered by the post-2015 bonus-eligible exception; no ADS, changed use, missing deductions or other basis differences.
    Separate {
        /// Cumulative AMT depreciation allowed or allowable through 2025
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        prior_depreciation: i64,
        /// Original AMT computation approach
        schedule: EquipmentSchedule,
    },
}

/// Original MACRS computation approach
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EquipmentSchedule {
    /// Manual MACRS calculation
    Manual,
    /// IRS percentage tables
    Tables {
        /// Original first-year bonus deduction
        /// Enter the actual income-producing bonus amount from the original depreciation schedule, or zero if elected out. It is already included in Prior depreciation; do not subtract it twice. Enter the actual bonus after the original section 179 election.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        initial_bonus: i64,
    },
}

/// Bonus depreciation election
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum BonusElection {
    /// Apply the available bonus allowance
    /// Ordinary qualifying 2026 equipment uses 100% if acquired after January 19, 2025, or 20% if acquired after September 27, 2017 and before January 20, 2025. MACRS recovers any remaining basis.
    Apply,
    /// Elect out for this recovery class
    ElectOut,
}

/// Equipment depreciation choice
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EquipmentMethod {
    /// Ordinary GDS declining balance
    /// 200% declining balance for 3-, 5-, 7- and 10-year classes; 150% for 15- and 20-year classes.
    Regular,
    /// 150% declining balance
    Declining150,
    /// Straight line over GDS recovery period
    StraightLine,
}
impl BusinessAsset {
    fn bonus_eligible(&self) -> bool {
        self.acquired >= 20170928
    }

    pub(crate) fn amt_carry_choices(&self) -> alloc::vec::Vec<i32> {
        if matches!(self.history, EquipmentHistory::Continuing { initial_section179, .. } if initial_section179 > 0)
            && self.derived_amt_section179_carry().is_err()
        {
            alloc::vec![0]
        } else {
            alloc::vec::Vec::new()
        }
    }

    pub(crate) fn amt_section179_carry(&self) -> Result<i64, ()> {
        let derived = self.derived_amt_section179_carry();
        match (derived, self.amt_section179_income_carry) {
            (Ok(actual), Some(reported)) if actual != reported => Err(()),
            (Ok(actual), _) => Ok(actual),
            (Err(()), Some(reported)) if self.amt_section179_carry_fact_valid() => Ok(reported),
            _ => Err(()),
        }
    }

    fn amt_section179_carry_fact_valid(&self) -> bool {
        let Some(reported) = self.amt_section179_income_carry else {
            return true;
        };
        let EquipmentHistory::Continuing { initial_section179, .. } = self.history else {
            return false;
        };
        if initial_section179 <= 0 || !(0..=initial_section179).contains(&reported) {
            return false;
        }
        match self.derived_amt_section179_carry() {
            Ok(actual) => actual == reported,
            Err(()) => {
                // A complete unchanged-use separate schedule must reconcile;
                // a manual balance cannot override contradictory actual rows.
                let complete = self.history_details.as_ref().is_some_and(|records| {
                    let year = self.placed / 10000;
                    // Match the post-1998 GDS/AMT source window in qualifies().
                    (1999..2026).contains(&year)
                        && records.years.len() == (2026 - year) as usize
                        && (year..2026).all(|year| {
                            records.years.iter().filter(|row| row.year == year).count() == 1
                        })
                        && records.auto_history.is_none()
                        && self.earlier_business_percent.is_none()
                        && records.years.iter().all(|row| {
                            row.basis_increase == 0
                                && row.basis_decrease == 0
                                && row.recaptured == 0
                        })
                });
                !complete
                    || matches!(
                        self.history,
                        EquipmentHistory::Continuing { amt: EquipmentAmtHistory::Same, .. }
                    )
            }
        }
    }

    fn derived_amt_section179_carry(&self) -> Result<i64, ()> {
        // This is also read while an ordinary editor's source facts change.
        // Reject an invalid fraction before evaluating its business basis.
        if !self.vehicle_valid()
            || self.cost < 0
            || !(0..=self.cost).contains(&self.credit_reduction)
        {
            return Err(());
        }
        let EquipmentHistory::Continuing {
            initial_section179,
            carryforward,
            prior_depreciation,
            amt,
            ..
        } = self.history
        else {
            return Ok(0);
        };
        if initial_section179 < 0
            || !(0..=initial_section179).contains(&carryforward)
            || prior_depreciation < 0
        {
            return Err(());
        }
        if initial_section179 == 0 {
            return Ok(0);
        }
        let EquipmentAmtHistory::Separate { prior_depreciation: alternative_prior, .. } = amt
        else {
            // The same MACRS method and other depreciation do not identify
            // independently income-limited AMT section179 allowances.
            return Err(());
        };
        if alternative_prior < 0 {
            return Err(());
        }
        let records = self.history_details.as_ref().ok_or(())?;
        let first_year = self.placed / 10000;
        // Match the post-1998 GDS/AMT source window in qualifies().
        if !(1999..2026).contains(&first_year)
            || records.years.len() != (2026 - first_year) as usize
            || records.original_section179_elected != initial_section179
            || records.original_business_basis != self.history_basis()
            || self.earlier_business_percent.is_some()
            || records.auto_history.is_some()
        {
            return Err(());
        }
        let (used, total) =
            self.vehicle().map(|(total, used, _, _)| (used, total)).unwrap_or_else(|| {
                match self.listed {
                    ListedProperty::No { business_percent } => (i64::from(business_percent), 100),
                    _ => unreachable!(),
                }
            });
        let same_use = |fraction: crate::UseFraction| {
            fraction.total > 0
                && fraction.used <= fraction.total
                && i64::from(fraction.used) * total == used * i64::from(fraction.total)
        };
        if !same_use(records.original_income_use) || !same_use(records.original_qualified_use) {
            return Err(());
        }
        let (mut regular_179, mut bonus, mut regular_other, mut alternative_other) =
            (0i64, 0i64, 0i64, 0i64);
        for year in first_year..2026 {
            let mut rows = records.years.iter().filter(|row| row.year == year);
            let row = rows.next().ok_or(())?;
            if rows.next().is_some()
                || !same_use(row.income_use)
                || !same_use(row.qualified_use)
                || [row.section179_allowed, row.bonus, row.regular, row.amt]
                    .iter()
                    .any(|value| *value < 0)
                || row.basis_increase != 0
                || row.basis_decrease != 0
                || row.recaptured != 0
            {
                return Err(());
            }
            regular_179 = regular_179.checked_add(row.section179_allowed).ok_or(())?;
            bonus = bonus.checked_add(row.bonus).ok_or(())?;
            regular_other = regular_other.checked_add(row.regular).ok_or(())?;
            alternative_other = alternative_other.checked_add(row.amt).ok_or(())?;
        }
        if regular_179.checked_add(carryforward).ok_or(())? != initial_section179
            || regular_179.checked_add(bonus).and_then(|v| v.checked_add(regular_other)).ok_or(())?
                != prior_depreciation
            || bonus != records.original_bonus
        {
            return Err(());
        }
        let alternative_179 = alternative_prior
            .checked_sub(bonus)
            .and_then(|v| v.checked_sub(alternative_other))
            .ok_or(())?;
        if !(0..=initial_section179).contains(&alternative_179) {
            return Err(());
        }
        Ok(crate::kernel::carry::native_balance(initial_section179, alternative_179))
    }

    fn amt_projection(&self) -> Option<Self> {
        if matches!(self.history, EquipmentHistory::New { .. }) {
            if !self.bonus_eligible()
                && self.method == EquipmentMethod::Regular
                && self.class.years() <= 10
            {
                // A 2026 first-use asset has no prior depreciation to enter.
                // Section 179 reduces both bases equally, and an acquisition
                // before September 28, 2017 cannot receive bonus in 2026.
                let mut asset = self.clone();
                asset.method = EquipmentMethod::Declining150;
                return Some(asset);
            }
            return None;
        }
        let EquipmentHistory::Continuing {
            prior_depreciation,
            carryforward: regular_carry,
            schedule,
            amt,
            ..
        } = self.history
        else {
            return None;
        };
        let actual_carry = self.amt_section179_carry().ok();
        let (alternative_prior, alternative_schedule, separate) = match amt {
            EquipmentAmtHistory::Separate { prior_depreciation, schedule } => {
                (prior_depreciation, schedule, true)
            }
            EquipmentAmtHistory::Same => {
                let carry = actual_carry?;
                if carry == regular_carry {
                    return None;
                }
                // The other MACRS allowances match. Replace only the actual
                // prior179 portion, preserving the original elected basis.
                (
                    prior_depreciation.checked_add(regular_carry)?.checked_sub(carry)?,
                    schedule,
                    false,
                )
            }
        };
        let mut asset = self.clone();
        asset.amt_section179_income_carry = None;
        if separate {
            asset.method = EquipmentMethod::Declining150;
        }
        if let EquipmentHistory::Continuing {
            prior_depreciation: prior,
            schedule: approach,
            carryforward,
            amt,
            ..
        } = &mut asset.history
        {
            *prior = alternative_prior;
            *approach = alternative_schedule;
            if let Some(actual) = actual_carry {
                *carryforward = actual;
            }
            *amt = EquipmentAmtHistory::Same;
        }
        Some(asset)
    }
    pub(crate) fn deduction_for(&self, mid_quarter: bool, amt: bool) -> i64 {
        if amt && let Some(asset) = self.amt_projection() {
            return asset.deduction(mid_quarter);
        }
        self.deduction(mid_quarter)
    }
    pub(crate) fn amt_remaining_basis(&self, mid_quarter: bool) -> i64 {
        self.amt_projection().map_or_else(
            || self.opening_basis() - self.deduction(mid_quarter),
            |a| a.opening_basis() - a.deduction(mid_quarter),
        )
    }
    pub(crate) fn table_rates(&self, mid_quarter: bool) -> &'static [i64] {
        let method = match self.effective_method() {
            EquipmentMethod::Regular => 0,
            EquipmentMethod::Declining150 => 1,
            EquipmentMethod::StraightLine => 2,
        };
        let convention = if mid_quarter {
            1 + (self.placed / 100 % 100).saturating_sub(1).min(11) as usize / 3
        } else {
            0
        };
        let class = match self.class {
            EquipmentClass::Three => 0,
            EquipmentClass::Five => 1,
            EquipmentClass::Seven => 2,
            EquipmentClass::Ten => 3,
            EquipmentClass::Fifteen => 4,
            EquipmentClass::Twenty => 5,
            EquipmentClass::ThirtyNine => {
                unreachable!(
                    "nonresidential real property uses nonresidential_rate, not table_rates"
                )
            }
        };
        crate::kernel::cost_recovery::macrs_tables::RATES[method * 30 + convention * 6 + class]
    }
    // The basis the earlier years' depreciation was figured on: the business
    // share at the earlier business use for a property (not a vehicle) whose use
    // fell to 50% or less this year, else this year's business basis.
    pub(crate) fn history_basis(&self) -> i64 {
        match self.earlier_business_percent {
            Some(percent) if !self.is_vehicle() && !self.investment_credit.is_some() => {
                crate::kernel::ratio::fraction(
                    self.cost - self.credit_reduction,
                    i64::from(percent),
                    100,
                )
            }
            _ => self.business_basis(),
        }
    }
    // A refund adjusts the same income-producing basis as its original
    // schedule. A fall in use is applied later by the shared schedule engine.
    pub(crate) fn historical_share_of(
        &self,
        amount: i64,
    ) -> Option<crate::kernel::ratio::exact::Exact> {
        if let Some(percent) = self.earlier_business_percent
            && !self.is_vehicle()
            && !self.investment_credit.is_some()
        {
            return Some(
                crate::kernel::ratio::exact::whole(amount)
                    * crate::kernel::ratio::exact::ratio(i64::from(percent), 100)?,
            );
        }
        self.current_share_of(amount)
    }
    pub(crate) fn current_share_of(
        &self,
        amount: i64,
    ) -> Option<crate::kernel::ratio::exact::Exact> {
        use crate::kernel::ratio::exact;
        let (numerator, denominator) = match &self.listed {
            ListedProperty::Vehicle { total_miles, business_miles, .. } => {
                (i64::from(*business_miles), i64::from(*total_miles))
            }
            ListedProperty::No { business_percent } => (i64::from(*business_percent), 100),
        };
        Some(exact::whole(amount) * exact::ratio(numerator, denominator)?)
    }
    pub(crate) fn opening_basis(&self) -> i64 {
        self.history_basis()
            - match self.history {
                EquipmentHistory::New { section179, .. } => section179.elected(),
                EquipmentHistory::Continuing { prior_depreciation, carryforward, .. } => {
                    prior_depreciation + carryforward
                }
            }
    }
    fn remaining_eighths(&self, mid_quarter: bool) -> i64 {
        let first = if mid_quarter {
            7 - 2 * i64::from((self.placed / 100 % 100).saturating_sub(1) / 3).min(3)
        } else {
            4
        };
        self.class.years() * 8 - first - (2026 - i64::from(self.placed / 10000) - 1) * 8
    }
    // Bonus and MACRS after the section 280F cap, bonus first.
    pub(crate) fn bonus_amount(&self) -> i64 {
        phase::cap(self.raw_bonus_amount(), [self.limit_room()])
    }
    pub(crate) fn macrs_amount(&self, mid_quarter: bool) -> i64 {
        if self.documented_179_reserve() {
            return 0;
        }
        if let Some(amount) = crate::auto_recovery::amount(self, 2026) {
            return amount;
        }
        phase::cap(self.raw_macrs_amount(mid_quarter), [self.limit_room() - self.bonus_amount()])
    }
    // The tentative amounts the section 280F cap limits; later-year basis
    // follows these, not the capped deduction.
    fn raw_bonus_amount(&self) -> i64 {
        self.raw_bonus_on(self.opening_basis())
    }
    pub(crate) fn raw_bonus_on(&self, opening_basis: i64) -> i64 {
        if !self.bonus_eligible()
            || self.placed_and_disposed_in_current_year()
            || self.ads()
            // Nonresidential real property is not section 168(k) bonus
            // eligible (other than qualified improvement property, out of
            // scope here).
            || self.class == EquipmentClass::ThirtyNine
            || !matches!(
                self.history,
                EquipmentHistory::New {
                    bonus: BonusElection::Apply,
                    ..
                }
            )
        {
            return 0;
        }
        crate::kernel::ratio::fraction(
            opening_basis,
            if self.acquired >= 20250120 { 100 } else { 20 },
            100,
        )
    }
    pub(crate) fn needs_macrs(&self) -> bool {
        // Mid-month nonresidential real property never depends on the
        // return-wide half-year/mid-quarter convention or bonus depreciation.
        if self.class == EquipmentClass::ThirtyNine {
            return false;
        }
        self.placed / 10000 == 2026
            && (self.ads()
                || !matches!(
                    self.history,
                    EquipmentHistory::New { bonus: BonusElection::Apply, .. }
                )
                || self.acquired < 20250120)
    }
    pub(crate) fn deduction(&self, mid_quarter: bool) -> i64 {
        self.bonus_amount() + self.macrs_amount(mid_quarter)
    }
    // Property placed in service and disposed of in 2026 (sold, scrapped or
    // converted to personal use) takes no MACRS or bonus depreciation: no
    // depreciation deduction is allowed for it (Reg. 1.168(d)-1(b)(3)(ii);
    // Pub 946 ch. 4; i4562 "Special depreciation allowance"). The section 179
    // election stays (Reg. 1.179-1(e)(3)); a disposition recaptures it as
    // section 1245 gain.
    fn placed_and_disposed_in_current_year(&self) -> bool {
        matches!(self.history, EquipmentHistory::New { .. }) && self.disposal_date().is_some()
    }
    // The year-of-disposition amount (Pub 946 ch. 4): the full year's
    // deduction times the convention's share of the year up to the date
    // sold or stopped using. Under the half-year convention the last
    // recovery year is already a half year and is taken whole. Under the
    // mid-quarter convention the last recovery year is a short year of 24
    // less the first year's half-months (Pub 946 table A-5 to A-8 footnote;
    // "the unrecovered basis"): the disposal quarter's midpoint takes that
    // share of it, and a disposal at or after the end of the short year takes
    // the whole unrecovered basis. A 39-year building with a partial
    // disposition depreciates on its cost less the part from that month on.
    fn raw_macrs_amount(&self, mid_quarter: bool) -> i64 {
        if self.placed_and_disposed_in_current_year() {
            return 0;
        }
        if let Some(schedule) = self.gds_schedule(mid_quarter) {
            return schedule.amounts().current_macrs_cents;
        }
        self.apply_building_year(self.full_year_macrs(mid_quarter))
    }
    pub(crate) fn purchase_building_amount(&self, reduction: i64, tax_year: u32) -> Option<i64> {
        Some(self.apply_building_year(self.purchase_building_full_year(reduction, tax_year)?))
    }
    pub(crate) fn purchase_building_full_year(&self, reduction: i64, tax_year: u32) -> Option<i64> {
        use crate::kernel::cost_recovery::historical::{
            self, DatedBasisReduction, RentalBuildingRecovery,
        };
        if self.class != EquipmentClass::ThirtyNine {
            return None;
        }
        if self.placed_and_disposed_in_current_year() {
            return Some(0);
        }
        let (basis, adjustment) = match self.history {
            EquipmentHistory::New { .. } => (self.history_basis().checked_sub(reduction)?, None),
            EquipmentHistory::Continuing { prior_depreciation, .. } => (
                self.history_basis(),
                Some(DatedBasisReduction {
                    event_date: tax_year.checked_mul(10000)?.checked_add(101)?,
                    adjusted_basis_cents: self
                        .history_basis()
                        .checked_sub(prior_depreciation)?
                        .checked_sub(reduction)?,
                    depreciation_through_event_cents: prior_depreciation,
                    prior_depreciation_cents: prior_depreciation,
                }),
            ),
        };
        let full = historical::rental_building_amount(
            RentalBuildingRecovery::Commercial39,
            basis,
            self.placed,
            tax_year,
            24,
            adjustment,
        )
        .ok()?
        .rounded_cents();
        Some(full)
    }
    fn apply_building_year(&self, full: i64) -> i64 {
        let full = match (self.earlier_business_percent, &self.listed) {
            (Some(percent), ListedProperty::No { business_percent }) => {
                crate::kernel::ratio::fraction(
                    full,
                    i64::from(*business_percent),
                    i64::from(percent),
                )
            }
            _ => full,
        };
        if let (Some(date), EquipmentHistory::Continuing { .. }) =
            (self.disposal_date(), self.history)
        {
            return crate::kernel::ratio::fraction(
                full,
                crate::asset_sale::disposal_half_months(
                    date,
                    crate::asset_sale::Convention::MidMonth,
                ),
                24,
            );
        }
        self.partial_year(full)
    }
    // A 39-year building's year when part of it was disposed of: its mid-month
    // share before the part leaves, and the cost less the part's share after.
    fn partial_year(&self, full: i64) -> i64 {
        let Some(part) = self.partial.as_ref().filter(|_| self.class == EquipmentClass::ThirtyNine)
        else {
            return full;
        };
        let before = crate::kernel::ratio::fraction(
            full,
            crate::asset_sale::disposal_half_months(
                part.date,
                crate::asset_sale::Convention::MidMonth,
            ),
            24,
        );
        before
            + crate::kernel::ratio::fraction(
                full - before,
                self.cost - part.cost_of_part,
                self.cost,
            )
    }
    // A 39-year building's whole-year amount before any part of it is
    // disposed of (mid-month needs no convention choice).
    pub(crate) fn whole_year_macrs_before_partial(&self) -> i64 {
        self.full_year_macrs(false)
    }
    fn full_year_macrs(&self, mid_quarter: bool) -> i64 {
        if let Some(schedule) = self.gds_schedule(mid_quarter) {
            return schedule.full_year();
        }
        let basis = self.opening_basis() - self.raw_bonus_amount();
        let years_since = 2026_u32.saturating_sub(self.placed / 10000);
        let month = self.placed / 100 % 100;
        match nonresidential_rate(month, years_since) {
            Some(rate) => phase::cap(
                crate::kernel::ratio::fraction(self.business_basis(), rate, 100000),
                [basis],
            ),
            None => phase::cap(nonresidential_closing(self.business_basis(), month), [basis]),
        }
    }
    // Borrow the native source preparation for checked contextual calculations.
    // The caller selects the actual regular/AMT asset before entering here;
    // this adapter never rewrites the accepted cost or recovery history.
    pub(crate) fn checked_gds_schedule(
        &self,
        mid_quarter: bool,
        tax_year: u32,
    ) -> Option<crate::kernel::cost_recovery::gds::Schedule> {
        use crate::kernel::cost_recovery::gds;
        let native = self.gds_schedule(mid_quarter)?;
        let (approach, history) = match self.history {
            EquipmentHistory::New { section179, tables, .. } => (
                if tables { gds::Approach::Tables } else { gds::Approach::Manual },
                gds::History::New { elected179_cents: section179.elected() },
            ),
            EquipmentHistory::Continuing {
                prior_depreciation,
                carryforward,
                initial_section179,
                schedule,
                ..
            } => (
                match schedule {
                    EquipmentSchedule::Tables { .. } => gds::Approach::Tables,
                    EquipmentSchedule::Manual => gds::Approach::Manual,
                },
                gds::History::Continuing {
                    prior_depreciation_cents: prior_depreciation,
                    carryforward179_cents: carryforward,
                    initial179_cents: initial_section179,
                    initial_bonus_cents: match schedule {
                        EquipmentSchedule::Tables { initial_bonus } => initial_bonus,
                        EquipmentSchedule::Manual => 0,
                    },
                },
            ),
        };
        Some(gds::Schedule {
            placed: native.placed,
            tax_year,
            period: native.period,
            method: native.method,
            convention: native.convention,
            approach,
            history_basis_cents: self.history_basis(),
            history,
            raw_bonus_cents: self.raw_bonus_amount(),
            current_use_fraction: native.current_use_fraction,
            disposed: native.disposed,
            continuation: None,
        })
    }
    // One source boundary for ordinary equipment, including selected AMT assets.
    // Keep preparation native: previews can contain histories, dates and use
    // fractions outside the checked SourceFact schedule's admitted domain.
    fn gds_schedule(
        &self,
        mid_quarter: bool,
    ) -> Option<crate::kernel::cost_recovery::gds::NativeSchedule> {
        use crate::kernel::cost_recovery::{Month, gds};
        let period = match self.class {
            EquipmentClass::Three => gds::Period::Three,
            EquipmentClass::Five => gds::Period::Five,
            EquipmentClass::Seven => gds::Period::Seven,
            EquipmentClass::Ten => gds::Period::Ten,
            EquipmentClass::Fifteen => gds::Period::Fifteen,
            EquipmentClass::Twenty => gds::Period::Twenty,
            EquipmentClass::ThirtyNine => return None,
        };
        let basis = self.opening_basis() - self.raw_bonus_amount();
        let (table_basis_cents, convention, continuing) = match self.history {
            EquipmentHistory::New { tables, .. } => (
                if tables {
                    let deductions = self.elected179() + self.raw_bonus_amount();
                    Some((self.history_basis(), deductions))
                } else {
                    None
                },
                mid_quarter,
                false,
            ),
            EquipmentHistory::Continuing { schedule, initial_section179, mid_quarter, .. } => (
                match schedule {
                    EquipmentSchedule::Tables { initial_bonus } => {
                        let deductions = initial_bonus + initial_section179;
                        Some((self.history_basis(), deductions))
                    }
                    EquipmentSchedule::Manual => None,
                },
                mid_quarter,
                true,
            ),
        };
        Some(gds::NativeSchedule {
            placed: Month { year: self.placed / 10000, month: self.placed / 100 % 100 },
            tax_year: 2026,
            period,
            method: match self.effective_method() {
                EquipmentMethod::Regular => gds::Method::Regular,
                EquipmentMethod::Declining150 => gds::Method::Declining150,
                EquipmentMethod::StraightLine => gds::Method::StraightLine,
            },
            convention: if convention {
                gds::Convention::MidQuarter
            } else {
                gds::Convention::HalfYear
            },
            table_basis_cents,
            basis_after_raw_bonus_cents: basis,
            continuing,
            current_use_fraction: match (self.earlier_business_percent, &self.listed) {
                (Some(percent), ListedProperty::No { business_percent }) => {
                    Some((*business_percent, percent))
                }
                _ => None,
            },
            disposed: self
                .disposal_date()
                .map(|date| Month { year: date / 10000, month: date / 100 % 100 }),
        })
    }
}

// Form 4562 "Who Must File" for one business: depreciation of property
// placed in service in 2026, a section 179 deduction or carryover, or a
// vehicle or other listed property (2026 i1040sc line 13). Amortization has its
// own Form 4562-B (the 2026 Form 4562 has no Part VI) and never calls for Form
// 4562. Without any of these the depreciation of earlier-year assets goes on
// Schedule C line 13 or Schedule F line 14 alone.
pub(crate) fn assets_need_form_4562(assets: &[BusinessAsset]) -> bool {
    assets.iter().any(|a| a.placed / 10000 == 2026 || a.is_vehicle() || a.available179() > 0)
}
// The Form 4562 line of one asset's current bonus or MACRS amount: bonus on
// Part II line 14a; MACRS of an asset placed in service before 2026 on line
// 17, of a 2026 asset on its Part III line 19 row by class. A vehicle is
// reported in Part V instead (vehicle_columns).
pub(crate) fn form_4562_line(asset: &BusinessAsset, suffix: &str) -> Option<&'static str> {
    if asset.is_vehicle() {
        return None;
    }
    match suffix {
        "bonus" => Some("14a"),
        "macrs" if asset.placed / 10000 < 2026 => Some("17"),
        "macrs" => Some(match asset.class {
            EquipmentClass::Three => "19a",
            EquipmentClass::Five => "19b",
            EquipmentClass::Seven => "19c",
            EquipmentClass::Ten => "19d",
            EquipmentClass::Fifteen => "19e",
            EquipmentClass::Twenty => "19f",
            EquipmentClass::ThirtyNine => "19j",
        }),
        _ => None,
    }
}

// Every depreciable asset of the return: the businesses', the farm rentals',
// the personal-rental ones and the vehicles of Form 2106 (section 179 left out
// of those and of the farm rentals').
pub(crate) fn assets(
    d: &crate::FederalDraft,
) -> impl Iterator<Item = alloc::borrow::Cow<'_, BusinessAsset>> {
    assets_with_sources(d).map(|(_, asset)| asset)
}
type AssetSource = Option<(i64, usize)>;

fn assets_with_sources(
    d: &crate::FederalDraft,
) -> impl Iterator<Item = (AssetSource, alloc::borrow::Cow<'_, BusinessAsset>)> {
    d.income
        .iter()
        .flat_map(|entry| {
            let assets = match &entry.income {
                crate::FederalIncome::Business { data } => data.assets.as_slice(),
                crate::FederalIncome::FarmRental { data } => data.assets.as_slice(),
                crate::FederalIncome::Rental { data } => data.assets.as_slice(),
                crate::FederalIncome::NonbusinessProperty {
                    kind: crate::NonbusinessPropertyKind::PersonalRental { assets, .. },
                    ..
                } => assets.as_slice(),
                _ => &[],
            };
            let share = d
                .setup
                .as_ref()
                .filter(|_| crate::community::self_divided(&entry.income))
                .and_then(|setup| {
                    crate::community::share(setup, entry.owner, entry.community, &entry.income)
                });
            assets.iter().enumerate().map(move |(index, asset)| {
                (
                    Some((entry.id, index)),
                    match share {
                        Some(share) if !share.whole() => alloc::borrow::Cow::Owned(
                            crate::community::Divide::divided(asset, share),
                        ),
                        _ => alloc::borrow::Cow::Borrowed(asset),
                    },
                )
            })
        })
        .chain(crate::employee_expenses::vehicle_assets(d).map(|asset| (None, asset)))
}

pub(crate) fn mid_quarter(d: &crate::FederalDraft) -> bool {
    // Pub 946 ch. 4: residential rental and nonresidential real property are
    // not taken into account for the mid-quarter >40% test.
    let (total, fourth) = assets(d)
        .filter(|asset| {
            asset.placed / 10000 == 2026
                && asset.class != EquipmentClass::ThirtyNine
                // Pub 946 ch. 4: property placed in service and disposed of in
                // the same year is not taken into account.
                && !asset.placed_and_disposed_in_current_year()
        })
        .fold((0_i64, 0_i64), |(total, fourth), asset| {
            let basis = asset.business_basis() - asset.elected179();
            (total + basis, fourth + if asset.placed / 100 % 100 >= 10 { basis } else { 0 })
        });
    // Exact comparison before any whole-dollar tax-line rounding. Bonus does
    // not reduce the basis used for the >40% test. Elected section 179 does.
    i128::from(fourth) * 5 > i128::from(total) * 2
}

pub(crate) fn consistent_elections(d: &crate::FederalDraft) -> bool {
    election_conflict(d).is_none()
}
pub(crate) fn election_conflict(d: &crate::FederalDraft) -> Option<(AssetSource, AssetSource)> {
    // ADS listed property is outside the class-wide GDS and bonus elections;
    // nonresidential real property has no method or bonus choice to keep
    // consistent (always 39-year straight-line, never bonus eligible).
    assets_with_sources(d)
        .filter(|(_, asset)| {
            asset.placed / 10000 == 2026
                && !asset.ads()
                && asset.class != EquipmentClass::ThirtyNine
        })
        .find_map(|(source, asset)| {
            assets_with_sources(d)
                .filter(|(_, asset)| {
                    asset.placed / 10000 == 2026
                        && !asset.ads()
                        && asset.class != EquipmentClass::ThirtyNine
                })
                .find_map(|(other_source, other)| {
                    let equivalent_150 = asset.class.years() >= 15
                        && matches!(
                            asset.method,
                            EquipmentMethod::Regular | EquipmentMethod::Declining150
                        )
                        && matches!(
                            other.method,
                            EquipmentMethod::Regular | EquipmentMethod::Declining150
                        );
                    let consistent = asset.class != other.class
                        || (match (asset.history, other.history) {
                            (
                                EquipmentHistory::New { bonus: a, .. },
                                EquipmentHistory::New { bonus: b, .. },
                            ) => !asset.bonus_eligible() || !other.bonus_eligible() || a == b,
                            _ => true,
                        } && (asset.method == other.method || equivalent_150));
                    (!consistent).then_some((source, other_source))
                })
        })
}

// Difference in deductible asset costs; positive means higher AMT income.
pub(crate) fn business_amt_adjustment(b: &crate::FederalBusiness, mid_quarter: bool) -> i64 {
    b.assets
        .iter()
        .map(|asset| asset.deduction(mid_quarter) - asset.deduction_for(mid_quarter, true))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn old_acquisition() -> BusinessAsset {
        BusinessAsset {
            id: 0,
            history_details: None,
            amt_section179_income_carry: None,
            description: String::new(),
            cost: 1_000_000,
            credit_reduction: 0,
            acquired: 20170927,
            placed: 20260115,
            class: EquipmentClass::Five,
            history: EquipmentHistory::New {
                section179: Section179::Ineligible,
                bonus: BonusElection::Apply,
                tables: false,
            },
            method: EquipmentMethod::Declining150,
            listed: ListedProperty::No { business_percent: 100 },
            disposal: crate::AssetDisposal::No,
            investment_credit: crate::InvestmentCredit::No,
            earlier_business_percent: None,
            partial: None,
        }
    }

    fn rental_draft(assets: alloc::vec::Vec<BusinessAsset>) -> crate::FederalDraft {
        let mut draft = crate::FederalDraft::empty("depreciation-example".into());
        draft.income.push(crate::FederalIncomeEntry {
            refund_management: None,
            state_source: Default::default(),
            id: 1,
            label: "Equipment rent".into(),
            owner: crate::Owner::Taxpayer,
            income: crate::FederalIncome::NonbusinessProperty {
                kind: crate::NonbusinessPropertyKind::PersonalRental {
                    basis: 2_000_000,
                    depreciable_basis: 2_000_000,
                    assets,
                    history: crate::PersonalRentalHistory::NONE,
                },
                receipts: 0,
                expenses: 0,
                ordinary: true,
            },
            community: crate::CommunityShare::NotNeeded,
        });
        draft
    }

    #[test]
    fn native_gds_preview_keeps_unvalidated_calendar() {
        let mut asset = old_acquisition();
        // Future service year and a zero service month still use the native
        // saturating year/quarter, rather than becoming unavailable.
        asset.placed = 20270015;
        assert_eq!(asset.raw_macrs_amount(true), 262_500);
    }

    #[test]
    fn native_gds_preview_keeps_increasing_use() {
        let mut asset = old_acquisition();
        // An increasing-use editor value is not the checked falling-use domain.
        asset.earlier_business_percent = Some(50);
        assert_eq!(asset.raw_macrs_amount(false), 150_000);
    }

    #[test]
    fn native_gds_final_table_year_skips_original_basis_subtraction() {
        let mut asset = old_acquisition();
        // A final table year returns remaining basis before subtracting original
        // deductions; that unused subtraction would overflow for this input.
        asset.cost = i64::MIN;
        asset.placed = 20210115;
        asset.history = EquipmentHistory::Continuing {
            initial_section179: 0,
            carryforward: 0,
            prior_depreciation: 0,
            amt: EquipmentAmtHistory::Same,
            mid_quarter: false,
            schedule: EquipmentSchedule::Tables { initial_bonus: 1 },
        };
        assert_eq!(asset.raw_macrs_amount(false), i64::MIN);
    }

    #[test]
    fn native_gds_new_disposed_skips_basis_and_use_arithmetic() {
        let mut asset = old_acquisition();
        // New/disposed returns before any use-ratio or history arithmetic.
        asset.history = EquipmentHistory::New {
            section179: Section179::Ineligible,
            bonus: BonusElection::Apply,
            tables: true,
        };
        asset.earlier_business_percent = Some(0);
        asset.disposal =
            crate::AssetDisposal::Disposed { date: 20260115, how: crate::DisposalKind::Scrapped };
        assert_eq!(asset.raw_macrs_amount(false), 0);
    }

    #[test]
    #[should_panic(expected = "divide by zero")]
    fn native_gds_preview_preserves_zero_use_divisor_failure() {
        let mut asset = old_acquisition();
        asset.earlier_business_percent = Some(0);
        asset.raw_macrs_amount(false);
    }

    #[test]
    fn a_credit_taken_on_the_asset_reduces_its_depreciable_basis() {
        // Sections 30D(f)(1), 45W(d)(1) and 50(c)(1): the basis is reduced by
        // the credit. Cost $10,000.00, credit $2,500.00: basis $7,500.00. The
        // first year of 5-year 150% declining balance, half-year, is 15%:
        // 7,500.00 x 15% = 1,125.00 (1,500.00 without the credit); the
        // straight-line first year is 10%: 750.00. Opening basis 7,500.00.
        let mut asset = old_acquisition();
        asset.credit_reduction = 250_000;
        assert!(asset.valid());
        assert_eq!(asset.business_basis(), 750_000);
        assert_eq!(asset.deduction(false), 112_500);
        asset.method = EquipmentMethod::StraightLine;
        assert_eq!(asset.deduction(false), 75_000);
        // A credit cannot be more than the cost; the whole cost leaves no basis.
        asset.credit_reduction = 1_000_001;
        assert!(!asset.valid());
        asset.credit_reduction = 1_000_000;
        assert!(asset.valid());
        assert_eq!(asset.deduction(false), 0);
    }

    #[test]
    fn older_purchase_first_placed_in_service_in_2026_uses_gds_without_bonus() {
        // Pub. 946 starts depreciation at first income-producing service;
        // Rev. Proc. 2024-13 denies bonus for a pre-2017 acquisition placed
        // after 2019; Form 6251 does not refigure post-1998 150%/SL property.
        let mut asset = old_acquisition();
        assert!(asset.valid());
        assert!(asset.qualifies());
        assert!(asset.qualifies_nonbusiness_rental());
        assert_eq!(asset.bonus_amount(), 0);
        assert_eq!(asset.deduction(false), 150_000); // $10,000 × 150% / 5 × 1/2
        assert_eq!(asset.deduction_for(false, true), 150_000);

        asset.method = EquipmentMethod::StraightLine;
        assert!(asset.qualifies());
        assert_eq!(asset.deduction(false), 100_000);

        asset.method = EquipmentMethod::Regular;
        assert!(asset.qualifies()); // AMT uses 150% on the same first-year basis.
        assert_eq!(asset.deduction(false), 200_000);
        assert_eq!(asset.deduction_for(false, true), 150_000);
        assert_eq!(asset.amt_remaining_basis(false), 850_000);
        asset.class = EquipmentClass::Fifteen;
        assert!(asset.qualifies()); // Regular 15-year GDS is 150%.
        assert_eq!(asset.deduction(false), 50_000);

        asset.acquired = 20170928;
        assert!(asset.bonus_eligible());
        assert_eq!(asset.bonus_amount(), 200_000); // 2026 phase-down rate.
    }

    #[test]
    fn ineligible_older_purchase_does_not_choose_bonus_for_eligible_classmates() {
        let mut old = old_acquisition();
        old.placed = 20261015;
        let mut recent = old_acquisition();
        recent.acquired = 20260115;
        recent.placed = 20260120;
        recent.history = EquipmentHistory::New {
            section179: Section179::Ineligible,
            bonus: BonusElection::ElectOut,
            tables: false,
        };
        let draft = rental_draft(vec![old.clone(), recent.clone()]);
        assert!(consistent_elections(&draft));
        assert!(mid_quarter(&draft)); // Both new assets count in the >40% test.

        old.acquired = 20170928;
        let draft = rental_draft(vec![old, recent]);
        assert!(!consistent_elections(&draft));
    }
}

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
    #[libertas_number(min = 1)]
    pub fair_market_value: i64,
    /// Lease payments for 2026
    /// The whole-vehicle payments for the year; spread an advance payment over the lease term and enter 2026's share. Not payments to buy the vehicle.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub lease_payments: i64,
    /// All miles driven during 2026
    #[libertas_number(min = 1)]
    pub total_miles: u32,
    /// Business miles driven during 2026
    /// Exclude commuting, personal travel and miles claimed by another business. The business share of every amount below is these miles over all miles.
    #[libertas_number(min = 1)]
    pub business_miles: u32,
    /// Commuting miles included in the annual total
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub commuting: u32,
    /// Fuel, insurance, repairs, registration and other operating costs for 2026
    /// Whole-vehicle amounts; not the lease payments, parking or tolls.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub operating_costs: i64,
    /// Business parking and tolls, excluding commuting
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
        if !crate::federal_rules::valid_date(self.began) || self.began > 20261231 {
            return Some(ValidationIssue::new("LEASE_ENDS_BEFORE_BEGAN").at_field("began"));
        }
        if self.ends.is_some_and(|d| {
            !crate::federal_rules::valid_date(d)
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

// Community-property projection; accepted source facts are retained.

crate::community::divide_record!(BusinessAsset {
    amounts: [
        cost,
        credit_reduction,
        history,
        listed,
        investment_credit,
        history_details,
        amt_section179_income_carry
    ],
    gains: [disposal, partial],
    facts: [id, description, acquired, placed, earlier_business_percent, class, method],
});
crate::community::divide_enum!(EquipmentHistory {
    records: [
        New { amounts: [section179], facts: [bonus, tables] },
        Continuing {
            amounts: [initial_section179, carryforward, prior_depreciation, amt, schedule],
            facts: [mid_quarter]
        },
    ],
    units: [],
    tuples: [],
});
crate::community::divide_enum!(EquipmentAmtHistory {
    records: [Separate { amounts: [prior_depreciation, schedule], facts: [] }],
    units: [Same],
    tuples: [],
});
crate::community::divide_enum!(EquipmentSchedule {
    records: [Tables { amounts: [initial_bonus], facts: [] }],
    units: [Manual],
    tuples: [],
});
crate::community::divide_enum!(ListedProperty {
    records: [
        No { amounts: [], facts: [business_percent] },
        Vehicle {
            amounts: [operating_costs, loan_interest],
            facts: [total_miles, business_miles, passenger_automobile]
        }
    ],
    units: [],
    tuples: [],
});

// Community-property projections preserve accepted source identity and facts.
crate::community::divide_enum!(Section179 {
    records: [Eligible { amounts: [elected], facts: [] }],
    units: [Ineligible],
    tuples: [],
});

// Canonical child identity projection for normal Save. An asset's identity
// survives reorder/edit and cannot be rebound to a different accepted activity.
pub(crate) fn income_asset_ids_mut(income: &mut crate::FederalIncome) -> alloc::vec::Vec<&mut i64> {
    let assets = match income {
        crate::FederalIncome::Business { data } => &mut data.assets,
        crate::FederalIncome::FarmRental { data } => &mut data.assets,
        crate::FederalIncome::Rental { data } => &mut data.assets,
        crate::FederalIncome::NonbusinessProperty {
            kind: crate::NonbusinessPropertyKind::PersonalRental { assets, .. },
            ..
        } => assets,
        _ => return alloc::vec::Vec::new(),
    };
    assets.iter_mut().map(|asset| &mut asset.id).collect()
}
pub(crate) fn income_asset_ids(income: &crate::FederalIncome) -> alloc::vec::Vec<i64> {
    let assets = match income {
        crate::FederalIncome::Business { data } => &data.assets,
        crate::FederalIncome::FarmRental { data } => &data.assets,
        crate::FederalIncome::Rental { data } => &data.assets,
        crate::FederalIncome::NonbusinessProperty {
            kind: crate::NonbusinessPropertyKind::PersonalRental { assets, .. },
            ..
        } => assets,
        _ => return alloc::vec::Vec::new(),
    };
    assets.iter().map(|asset| asset.id).collect()
}
