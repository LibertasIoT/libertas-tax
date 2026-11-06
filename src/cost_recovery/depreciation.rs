// Depreciation, listed-property recovery and return-wide Form 4562 coordination.
// https://www.irs.gov/publications/p946
pub use crate::asset_model::{
    BonusElection, BusinessAsset, EquipmentAmtHistory, EquipmentClass, EquipmentHistory,
    EquipmentMethod, EquipmentSchedule, ListedProperty, Section179, SuvException, VehicleKind,
};
use crate::kernel::phase;
pub use crate::vehicle_model::{BusinessLeasedVehicle, VehicleEvidence};
#[cfg(test)]
use alloc::string::String;

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
    pub(crate) fn amt_projection_or_self(&self) -> Self {
        self.amt_projection().unwrap_or_else(|| self.clone())
    }

    pub(crate) fn changed_nonlisted_use(&self) -> Option<(u32, u32)> {
        let earlier = self.earlier_business_percent?;
        match self.listed {
            ListedProperty::No { business_percent } => Some((business_percent, earlier)),
            ListedProperty::Vehicle {
                total_miles,
                business_miles,
                kind: VehicleKind::QualifiedNonpersonal,
                ..
            } => {
                // Validated mileage is at most2,000,000 and percent at most100;
                // both scaled operands fit u32. Preserve the exact miles.
                Some((business_miles * 100, total_miles * earlier))
            }
            _ => None,
        }
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
    // listed vehicle uses column (b), `recapture()`; a section 179 deduction on
    // property placed in service in 2026 and converted in 2026 has no
    // depreciation that would have been allowable.
    pub(crate) fn section179_recapture(&self) -> Option<ListedRecapture> {
        if self.is_listed_vehicle() || self.class == EquipmentClass::ThirtyNine {
            return None;
        }
        let converted = self.converted();
        match self.history {
            EquipmentHistory::New { section179, .. } => (converted && section179.elected() > 0)
                .then(|| ListedRecapture { allowed: section179.elected(), recomputed: 0 }),
            EquipmentHistory::Continuing {
                initial_section179, carryforward, mid_quarter, ..
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
                    recomputed: self.allowable_on(expensed, mid_quarter),
                })
            }
        }
    }
    // The depreciation that would have been allowable on `amount` (cents) from
    // the year placed in service through 2026 under the asset's original
    // method and convention: the published table rates, or the manual formula
    // (declining balance switching to straight line). For a conversion in 2026
    // the current year uses the kernel's disposition convention, including
    // the final recovery year's short-year treatment.
    fn allowable_on(&self, amount: i64, mid_quarter: bool) -> i64 {
        use crate::kernel::cost_recovery::gds;
        let Some(native) = self.gds_schedule(mid_quarter) else {
            unreachable!("section 179 recapture excludes nonresidential real property")
        };
        let mut remaining = amount;
        let mut total = 0;
        for year in native.placed.year..=2026 {
            let annual = gds::NativeSchedule {
                tax_year: year,
                table_basis_cents: native.table_basis_cents.map(|_| (amount, 0)),
                basis_after_raw_bonus_cents: remaining,
                continuing: year != native.placed.year,
                current_use_fraction: None,
                disposed: if year == 2026 && self.converted() { native.disposed } else { None },
                ..native
            }
            .amounts();
            remaining = crate::kernel::carry::native_balance(remaining, annual.full_year_cents);
            total += annual.current_macrs_cents;
        }
        total
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
                    section179.elected() == 0 || self.predominant()
                }
                EquipmentHistory::Continuing {
                    initial_section179, carryforward, schedule, ..
                } => {
                    carryforward == 0
                        && (initial_section179 == 0
                            || self.predominant()
                            || (!self.is_listed_vehicle()
                                && (self.converted() || self.earlier_business_percent.is_some())))
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
        if self.nonmacrs() {
            return self.valid()
                && self.nonmacrs_issue().is_none()
                && !self.is_vehicle()
                && matches!(self.listed, ListedProperty::No { business_percent: 100 })
                && self.available179() == 0;
        }
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

    pub(crate) fn qualifies(&self) -> bool {
        if self.nonmacrs() {
            return self.nonmacrs_valid() && self.nonmacrs_issue().is_none();
        }
        if self.macrs_source_fact_issue().is_some() {
            return false;
        }
        if self.source_ads_recovery().ok().flatten().is_some() {
            return self.valid()
                && self.acquired <= self.placed
                && match self.history {
                    EquipmentHistory::New { section179, .. } => {
                        self.placed / 10000 == 2026
                            && (matches!(section179, Section179::Ineligible)
                                || ((self.class != EquipmentClass::ThirtyNine
                                    || self.potential_real179())
                                    && (self.predominant() || self.converted())))
                    }
                    EquipmentHistory::Continuing { .. } => {
                        (19870101..20260101).contains(&self.placed)
                    }
                };
        }
        // Nonresidential real property is never section 179 or bonus
        // eligible, needs no AMT history (class.years() >= 15 already exempts
        // it below) and can never exhaust its 39-year period within the
        // supported 1999-2025 continuing-service window.
        let real_property = self.class == EquipmentClass::ThirtyNine && !self.potential_real179();
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
                            || self.macrs_source_issue().is_some()
                            || self.source_ads_recovery().ok().flatten().is_some()
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
    #[cfg(test)]
    pub(crate) fn qbi_property(&self) -> bool {
        self.qbi_property_in_context(crate::macrs_source::AdsInvocation::default())
    }

    pub(crate) fn qbi_property_in_context(
        &self,
        context: crate::macrs_source::AdsInvocation<'_>,
    ) -> bool {
        if self.nonmacrs() {
            return self.nonmacrs_qbi_property();
        }
        if self.disposal != crate::AssetDisposal::No {
            return false;
        }
        let (year, monthday) = (self.placed / 10000, self.placed % 10000);
        let ten_years = (year + 10) * 10000 + monthday;
        if ten_years >= 20261231 {
            return true;
        }
        if let Ok(Some(ads)) = self.source_ads_recovery_in_context(context) {
            let midpoint = if self.class == EquipmentClass::ThirtyNine && !ads.qip {
                2 * i64::from(self.placed / 100 % 100) - 1
            } else if matches!(self.history, EquipmentHistory::Continuing { mid_quarter: true, .. })
            {
                3 + 6 * i64::from((self.placed / 100 % 100).saturating_sub(1) / 3)
            } else {
                12
            };
            return crate::kernel::ratio::exact::whole(i64::from(year) * 24 + midpoint)
                + ads.period_half_months
                >= crate::kernel::ratio::exact::whole(2027 * 24);
        }
        if self.macrs_source_issue_in_context(context).is_some() {
            return false;
        }
        let recovery = (year + self.class.years() as u32 - 1) * 10000 + 1231;
        ten_years.max(recovery) >= 2026 * 10000 + 1231
    }

    // Only newly admitted documented reserve sources need this annual cap;
    // every pre-existing source retains its exact native allocator operand.
    pub(crate) fn annual179_capacity(&self) -> i64 {
        if self.documented_179_reserve() || crate::auto_recovery::amount(self, 2026).is_some() {
            phase::cap(self.available179(), [self.automobile_limit().unwrap_or(i64::MAX)])
        } else {
            self.available179()
        }
    }

    pub(crate) fn qualified179_cost_in_context(
        &self,
        context: crate::macrs_source::AdsInvocation<'_>,
    ) -> i64 {
        if self.source_ads_recovery_in_context(context).is_ok_and(|recovery| {
            recovery.is_some_and(|r| {
                r.reasons.iter().any(|reason| {
                    matches!(
                        reason,
                        crate::macrs_source::AdsReason::TaxExemptUse
                            | crate::macrs_source::AdsReason::ForeignUse
                    )
                })
            })
        }) {
            return 0;
        }
        if self.class == EquipmentClass::ThirtyNine && !self.real179_in_context(context) {
            0
        } else {
            self.qualified179_cost()
        }
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
        if let Some(route) = self.nonmacrs_route() {
            return route.section1250();
        }
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
        if self.nonmacrs() {
            return self.nonmacrs_straight_line_through(base, sold);
        }
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

impl BusinessAsset {
    pub(crate) fn amt_carry_choices(&self) -> alloc::vec::Vec<i32> {
        if matches!(self.history, EquipmentHistory::Continuing { initial_section179, .. } if initial_section179 > 0)
            && self.derived_amt_section179_carry().is_err()
        {
            alloc::vec![0]
        } else {
            alloc::vec::Vec::new()
        }
    }

    pub(crate) fn deduction_for(&self, mid_quarter: bool, amt: bool) -> i64 {
        if amt && let Some(asset) = self.amt_projection() {
            return asset.deduction(mid_quarter);
        }
        self.deduction(mid_quarter)
    }
    #[cfg(test)]
    pub(crate) fn amt_remaining_basis(&self, mid_quarter: bool) -> i64 {
        self.amt_projection().map_or_else(
            || self.opening_basis() - self.deduction(mid_quarter),
            |a| a.opening_basis() - a.deduction(mid_quarter),
        )
    }

    // A refund adjusts the same income-producing basis as its original
    // schedule. A fall in use is applied later by the shared schedule engine.
    pub(crate) fn historical_share_of(
        &self,
        amount: i64,
    ) -> Option<crate::kernel::ratio::exact::Exact> {
        if let Some(percent) = self.earlier_business_percent
            && !self.is_listed_vehicle()
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

    fn remaining_eighths(&self, mid_quarter: bool) -> i64 {
        let first = if mid_quarter {
            7 - 2 * i64::from((self.placed / 100 % 100).saturating_sub(1) / 3).min(3)
        } else {
            4
        };
        self.class.years() * 8 - first - (2026 - i64::from(self.placed / 10000) - 1) * 8
    }

    pub(crate) fn source_bonus_amount_in_context(
        &self,
        context: crate::macrs_source::AdsInvocation<'_>,
    ) -> i64 {
        phase::cap(self.raw_bonus_on_in_context(self.opening_basis(), context), [self.limit_room()])
    }

    // The source-qualified SL schedule supplies an uncapped current amount; the existing
    // passenger-auto and section179 reserve owners still limit that amount.
    pub(crate) fn source_macrs_amount_in_context(
        &self,
        mid_quarter: bool,
        context: crate::macrs_source::AdsInvocation<'_>,
    ) -> Option<i64> {
        if self.documented_179_reserve() || self.placed_and_disposed_in_current_year() {
            return Some(0);
        }
        if self.history_details.as_ref().is_some_and(|records| records.auto_history.is_some()) {
            let recovery = self.source_ads_recovery_in_context(context).ok()??;
            if recovery.period_half_months != crate::kernel::ratio::exact::whole(120) {
                return None;
            }
            return crate::auto_recovery::ads_amount(self, crate::year_constants::TAX_YEAR);
        }
        let amount = self.source_ads_amount_in_context(mid_quarter, context)?;
        Some(phase::cap(amount, [self.limit_room() - self.source_bonus_amount_in_context(context)]))
    }
    pub(crate) fn macrs_history_issue_in_context(
        &self,
        context: crate::macrs_source::AdsInvocation<'_>,
    ) -> Option<&'static str> {
        if self.disposal == crate::AssetDisposal::No
            && self.partial.is_none()
            && self.earlier_business_percent.is_none()
        {
            return None;
        }
        match self.source_ads_recovery_in_context(context) {
            Ok(None) => None,
            Ok(Some(recovery))
                if (recovery.qip || self.real179_in_context(context))
                    && self.partial.is_none()
                    && self.earlier_business_percent.is_none() =>
            {
                None
            }
            Ok(Some(_)) => Some("ASSET_ADS_ORIGINAL_HISTORY_UNAVAILABLE"),
            Err(key) => Some(key),
        }
    }
    pub(crate) fn macrs_amount(&self, mid_quarter: bool) -> i64 {
        if self.macrs_source_issue().is_some() {
            return 0;
        }
        if self.nonmacrs() {
            return self
                .nonmacrs_recovery(self.bonus_amount())
                .map_or(0, |recovery| recovery.amount);
        }
        if self.documented_179_reserve() {
            return 0;
        }
        if self.source_ads_recovery().ok().flatten().is_some() {
            return self
                .source_macrs_amount_in_context(
                    mid_quarter,
                    crate::macrs_source::AdsInvocation::default(),
                )
                .unwrap_or(0);
        }
        if let Some(amount) = crate::auto_recovery::amount(self, 2026) {
            return amount;
        }
        phase::cap(self.raw_macrs_amount(mid_quarter), [self.limit_room() - self.bonus_amount()])
    }

    pub(crate) fn needs_macrs(&self) -> bool {
        self.needs_macrs_in_context(crate::macrs_source::AdsInvocation::default())
    }
    pub(crate) fn needs_macrs_in_context(
        &self,
        context: crate::macrs_source::AdsInvocation<'_>,
    ) -> bool {
        if self.nonmacrs() {
            return false;
        }
        // Mid-month nonresidential real property never depends on the
        // return-wide half-year/mid-quarter convention or bonus depreciation.
        if self.class == EquipmentClass::ThirtyNine
            && self.qualified_improvement(context) != Ok(true)
        {
            return false;
        }
        self.placed / 10000 == 2026
            && (self.ads()
                || self
                    .source_ads_recovery_in_context(context)
                    .ok()
                    .flatten()
                    .is_some_and(|ads| ads.required())
                || !matches!(
                    self.history,
                    EquipmentHistory::New { bonus: BonusElection::Apply, .. }
                )
                || self.acquired < 20250120)
    }
    pub(crate) fn deduction(&self, mid_quarter: bool) -> i64 {
        self.bonus_amount() + self.macrs_amount(mid_quarter)
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
        if self.macrs_source_issue().is_some() {
            return 0;
        }
        if self.placed_and_disposed_in_current_year() {
            return 0;
        }
        if let Some(amount) = self.source_ads_amount(mid_quarter) {
            return amount;
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
        if self.source_ads_recovery().ok().flatten().is_some()
            || self.macrs_source_issue().is_some()
        {
            return None;
        }
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
        let full = self.changed_nonlisted_use().map_or(full, |(used, total)| {
            crate::kernel::ratio::fraction(full, i64::from(used), i64::from(total))
        });
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
        if self.macrs_source_issue().is_some() {
            return 0;
        }
        if let Some(amount) = self.source_ads_amount(mid_quarter) {
            return amount;
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
        if self.source_ads_recovery().ok().flatten().is_some() {
            return None;
        }
        let period = match self.class {
            EquipmentClass::Three => gds::Period::Three,
            EquipmentClass::Five => gds::Period::Five,
            EquipmentClass::Seven => gds::Period::Seven,
            EquipmentClass::Ten => gds::Period::Ten,
            EquipmentClass::Fifteen => gds::Period::Fifteen,
            EquipmentClass::Twenty => gds::Period::Twenty,
            EquipmentClass::ThirtyNine | EquipmentClass::NonMacrs { .. } => return None,
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
            current_use_fraction: self.changed_nonlisted_use(),
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
    assets.iter().any(|a| a.placed / 10000 == 2026 || a.is_listed_vehicle() || a.available179() > 0)
}
// The Form 4562 line of one asset's current bonus or MACRS amount: bonus on
// Part II line 14a; MACRS of an asset placed in service before 2026 on line
// 17, of a 2026 asset on its Part III line19 row by class. A listed vehicle
// is reported in PartV instead (vehicle_columns).
pub(crate) fn form_4562_line(asset: &BusinessAsset, suffix: &str) -> Option<&'static str> {
    form_4562_line_in_context(asset, suffix, crate::macrs_source::AdsInvocation::default())
}
pub(crate) fn form_4562_line_in_context(
    asset: &BusinessAsset,
    suffix: &str,
    context: crate::macrs_source::AdsInvocation<'_>,
) -> Option<&'static str> {
    if asset.is_listed_vehicle() {
        return None;
    }
    if asset.nonmacrs() {
        return match suffix {
            "bonus" => Some("14a"),
            "macrs" => Some(
                if matches!(asset.nonmacrs_route(), Some(crate::NonMacrsRecovery::Units { .. })) {
                    "15"
                } else {
                    "16"
                },
            ),
            _ => None,
        };
    }
    match suffix {
        "bonus" => Some("14a"),
        "macrs"
            if asset.placed / 10000 == 2026
                && asset
                    .source_ads_recovery_in_context(context)
                    .ok()
                    .flatten()
                    .is_some_and(|recovery| recovery.is_ads()) =>
        {
            Some("20")
        }
        "macrs" if asset.placed / 10000 < 2026 => Some("17"),
        "macrs" if asset.qualified_improvement(context) == Ok(true) => Some("19e"),
        "macrs" => Some(match asset.class {
            EquipmentClass::Three => "19a",
            EquipmentClass::Five => "19b",
            EquipmentClass::Seven => "19c",
            EquipmentClass::Ten => "19d",
            EquipmentClass::Fifteen => "19e",
            EquipmentClass::Twenty => "19f",
            EquipmentClass::ThirtyNine => "19j",
            EquipmentClass::NonMacrs { .. } => unreachable!("non-MACRS form lines dispatch above"),
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
                    crate::community::share(
                        setup,
                        entry.owner.owned(),
                        entry.community,
                        &entry.income,
                    )
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
        .chain(crate::employee_expenses::property_assets(d).map(|asset| (None, asset)))
}

pub(crate) fn mid_quarter(d: &crate::FederalDraft) -> bool {
    mid_quarter_with_macrs(d, None)
}
pub(crate) fn mid_quarter_with_macrs(
    d: &crate::FederalDraft,
    macrs: Option<&crate::macrs_source::MacrsElectionView>,
) -> bool {
    let fallback = macrs.is_none().then(|| crate::macrs_source::MacrsElectionView::new(d));
    let view = macrs.or(fallback.as_ref()).unwrap();
    // Pub946 chapter4: QIP follows HY/MQ; whole real property remains excluded.
    let (total, fourth) = assets(d)
        .filter(|asset| {
            asset.placed / 10000 == 2026
                && !asset.nonmacrs()
                && !asset.placed_and_disposed_in_current_year()
                && (asset.class != EquipmentClass::ThirtyNine
                    || asset.qualified_improvement(view.building_context(asset)) == Ok(true))
        })
        .fold((0_i64, 0_i64), |(total, fourth), asset| {
            let basis = asset.business_basis() - asset.elected179();
            (total + basis, fourth + if asset.placed / 100 % 100 >= 10 { basis } else { 0 })
        });
    // Exact comparison before any whole-dollar tax-line rounding. Bonus does
    // not reduce the basis used for the >40% test. Elected section179 does.
    i128::from(fourth) * 5 > i128::from(total) * 2
}

pub(crate) fn consistent_elections(d: &crate::FederalDraft) -> bool {
    election_conflict(d).is_none()
}
pub(crate) fn election_conflict(d: &crate::FederalDraft) -> Option<(AssetSource, AssetSource)> {
    let view = crate::macrs_source::MacrsElectionView::new(d);
    let qip = |asset: &BusinessAsset| {
        asset.qualified_improvement(view.building_context(asset)) == Ok(true)
    };
    let context = |source: AssetSource, asset: &BusinessAsset| {
        if let Some((id, _)) = source
            && let Some(entry) = d.income.iter().find(|entry| entry.id == id)
        {
            return view.context(entry.owner.owned(), Some(id), asset);
        }
        if let Some(claim) = d
            .adjustments
            .iter()
            .flat_map(|a| &a.employee_expenses)
            .find(|claim| claim.all_assets().iter().any(|actual| actual == asset))
        {
            return view.context(claim.owner, None, asset);
        }
        view.building_context(asset)
    };
    let software: alloc::vec::Vec<_> = assets_with_sources(d)
        .filter(|(_, asset)| {
            asset.placed / 10000 == 2026 && asset.nonmacrs() && asset.bonus_eligible()
        })
        .collect();
    for (index, (source, asset)) in software.iter().enumerate() {
        for (other_source, other) in &software[index + 1..] {
            if let (EquipmentHistory::New { bonus: a, .. }, EquipmentHistory::New { bonus: b, .. }) =
                (asset.history, other.history)
                && a != b
            {
                return Some((*source, *other_source));
            }
        }
    }

    // QIP shares its15-year bonus class; its statutory SL method is not an
    // election requiring another15-year asset to use SL. Whole real property
    // remains outside these bonus and method elections.
    assets_with_sources(d)
        .filter(|(_, asset)| {
            asset.placed / 10000 == 2026
                && !asset.ads()
                && (asset.class != EquipmentClass::ThirtyNine || qip(asset))
                && !asset.nonmacrs()
        })
        .find_map(|(source, asset)| {
            assets_with_sources(d)
                .filter(|(_, asset)| {
                    asset.placed / 10000 == 2026
                        && !asset.ads()
                        && (asset.class != EquipmentClass::ThirtyNine || qip(asset))
                        && !asset.nonmacrs()
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
                    let class = if qip(&asset) { EquipmentClass::Fifteen } else { asset.class };
                    let other_class =
                        if qip(&other) { EquipmentClass::Fifteen } else { other.class };
                    let consistent = class != other_class
                        || (match (asset.history, other.history) {
                            (
                                EquipmentHistory::New { bonus: a, .. },
                                EquipmentHistory::New { bonus: b, .. },
                            ) => {
                                !asset.bonus_eligible_in_context(context(source, &asset))
                                    || !other
                                        .bonus_eligible_in_context(context(other_source, &other))
                                    || a == b
                            }
                            _ => true,
                        } && (qip(&asset)
                            || qip(&other)
                            || asset.method == other.method
                            || equivalent_150));
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

// Community-property projection; accepted source facts are retained.

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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn old_acquisition() -> BusinessAsset {
        BusinessAsset {
            farm_rental_use: None,
            id: 0,
            macrs_source: None,
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
            printed: None,
            refund_management: None,
            state_source: Default::default(),
            id: 1,
            label: "Equipment rent".into(),
            owner: crate::DocumentOwner::Yours,
            income: crate::FederalIncome::NonbusinessProperty {
                kind: crate::NonbusinessPropertyKind::PersonalRental {
                    related: alloc::vec![],
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
    fn native_gds_new_disposed_skips_basis_and_use_arithmetic() {
        let mut asset = old_acquisition();
        // New/disposed returns before any use-ratio or history arithmetic.
        asset.history = EquipmentHistory::New {
            section179: Section179::Ineligible,
            bonus: BonusElection::Apply,
            tables: true,
        };
        asset.disposal =
            crate::AssetDisposal::Disposed { date: 20260115, how: crate::DisposalKind::Scrapped };
        assert_eq!(asset.raw_macrs_amount(false), 0);
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
