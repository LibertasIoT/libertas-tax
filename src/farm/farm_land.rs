// Coverage: direct whole-parcel and actual allocated partial-parcel farm-land
// sales reuse Form 4797, section 1252, Form 6252 and Form 1062. Sold and retained
// opening regular/AMT basis and prior section 175 deductions share one parcel.
// Unassociated active section 126, current conservation and cooperative
// adjustments remain mandatory source boundaries, never guessed allocations.
// Source: Publication 225, Sale of a Farm; instructions for Forms 4797 and 6252.

use crate::dates::source_valid_date as valid_date;
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use alloc::string::String;
use libertas_macros::*;

/// Direct sale of an accepted farm land parcel
/// Opening basis, acquisition and recapture history come from the selected parcel once.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmLandSale {
    /// Property use throughout your ownership
    /// Use the lease and property-use records for this sold property.
    /// [NullText]
    /// Property-use records are unavailable
    /// [SetText]
    /// The property-use records show
    pub farm_rental_use: Option<crate::FarmRentalPropertyUse>,

    /// Accepted sale identity; assigned on normal Save
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    /// Accepted parcel from this farm
    #[libertas_number(min = 1)]
    pub basis_property: i64,
    /// Sale label
    #[libertas_size(min = 1)]
    pub label: String,
    /// Actual date sold
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
    pub sold: u32,
    /// Full consideration, including buyer note and assumed liabilities
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub proceeds: i64,
    /// Actual selling expenses
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub selling: i64,
    /// Elect section 1062 tax-payment installments with qualifying use, buyer and recorded covenant
    pub deferral: bool,
    /// Actual transaction character
    pub situation: FarmLandSituation,
    /// How consideration was received
    pub payment: crate::SalePayment,
}

/// Actual farm land transaction; special source owners retain unresolved mandatory gain
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmLandSituation {
    /// Ordinary whole-parcel direct sale with purchased or estate fair-market-value basis
    Direct,
    /// Only an identified part of this parcel was sold
    /// Use the actual allocation records for the named sold tract, not the whole parcel's basis or an assumed acreage fraction.
    PartialParcel {
        /// Recorded opening basis and prior deductions attributable to the sold tract
        allocation: FarmLandAllocation,
    },
    /// Related-party disposition; actual loss and recapture rules are required
    RelatedParty,
    /// Exchange or involuntary conversion
    ExchangeOrConversion,
    /// Gift, carryover, special-use valuation, conservation-easement or income-in-respect-of-decedent basis
    SpecialBasis,
    /// Other unresolved ownership or transaction facts
    Other,
}

/// Actual opening/acquisition amounts allocated to the tract named by this sale
/// Publication 225, Sale of a Farm: the part sold uses its properly allocated basis.
/// Retain the allocation records; no acreage, value or recapture allocation is inferred.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmLandAllocation {
    /// Regular opening/acquisition basis of the sold tract, excluding current-year adjustments
    #[libertas_default(Unavailable)]
    pub regular: crate::SourceCents,
    /// AMT opening/acquisition basis of that same tract, excluding current-year adjustments
    #[libertas_default(Unavailable)]
    pub amt: crate::SourceCents,
    /// Prior section 175 deductions attributable to that tract
    #[libertas_default(Unavailable)]
    pub conservation: crate::SourceCents,
    /// Prior section 175 deductions attributable to that tract for AMT
    #[libertas_default(Unavailable)]
    pub amt_conservation: crate::SourceCents,
}
impl FarmLandSituation {
    pub(crate) fn direct(&self) -> bool {
        matches!(self, Self::Direct | Self::PartialParcel { .. })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedFarmLandSale {
    pub cost_sharing_recapture: i64,
    pub label: String,
    pub acquisition: FarmLandAcquisition,
    pub acquired: u32,
    pub sold: u32,
    pub proceeds: i64,
    pub selling: i64,
    pub basis: i64,
    pub conservation: i64,
    pub amt: FarmLandAmt,
    pub deferral: bool,
    pub eligible: bool,
    pub payment: crate::SalePayment,
}

/// Land acquisition
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FarmLandAcquisition {
    /// Purchased
    Purchased,
    /// Inherited with estate fair-market-value basis
    /// Use the reconciled section 1014 basis from estate records, including later basis adjustments. The initial basis must be solely date-of-death or elected alternate-valuation fair market value. Special-use valuation, carryover basis and income in respect of a decedent require other source calculations.
    Inherited,
}

/// AMT land-sale source records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FarmLandAmt {
    /// Same as for regular tax
    Same,
    /// Different retained AMT records
    Different {
        /// Retained AMT parcel basis
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis: i64,
        /// Prior conservation deductions allowed for AMT
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        conservation: i64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LandGain {
    pub gain: i64,
    pub percent: i64,
    pub recapture: i64,
    pub ordinary: i64,
    pub section1231: i64,
    // Business-asset unrecaptured section 1250 component; zero for land.
    pub unrecaptured: i64,
}
impl LandGain {
    pub(crate) fn zero() -> Self {
        Self { gain: 0, percent: 0, recapture: 0, ordinary: 0, section1231: 0, unrecaptured: 0 }
    }
}
impl ResolvedFarmLandSale {
    pub(crate) fn valid(&self) -> bool {
        self.eligible
            && !self.label.is_empty()
            && valid_date(self.acquired)
            && valid_date(self.sold)
            && self.acquired <= self.sold
            && (crate::year_constants::TAX_YEAR_START_DATE
                ..=crate::year_constants::TAX_YEAR_END_DATE)
                .contains(&self.sold)
            && match self.amt {
                FarmLandAmt::Same => true,
                FarmLandAmt::Different { basis, conservation } => {
                    [basis, conservation].iter().all(|v| (0..=100000000000).contains(v))
                }
            }
            && [self.proceeds, self.selling, self.basis, self.conservation]
                .iter()
                .all(|v| (0..=100000000000).contains(v))
            && self.payment.valid(self.proceeds)
    }
    // The year-of-sale Form 6252 of an installment sale at a gain of land
    // held more than a year: section 1252 recapture is not taxed ahead of
    // the payments (it is not section 453(i) income) but taken first from
    // each year's installment income (line 25).
    pub(crate) fn installment_for(
        &self,
        amt: bool,
    ) -> Option<crate::kernel::installment::Schedule> {
        let (basis, _) = self.records(amt);
        let long = self.acquisition == FarmLandAcquisition::Inherited
            || crate::dates::held_more_than_one_year(self.acquired, self.sold);
        if !long {
            return None;
        }
        let (recapture, cost_sharing) = self.ordinary_pools_for(amt);
        self.payment.form(
            dollars(self.proceeds),
            dollars(basis),
            0,
            dollars(self.selling),
            0,
            recapture + cost_sharing,
        )
    }
    fn records(&self, amt: bool) -> (i64, i64) {
        match (&self.amt, amt) {
            (FarmLandAmt::Different { basis, conservation }, true) => (*basis, *conservation),
            _ => (self.basis, self.conservation),
        }
    }
    pub(crate) fn ordinary_pools_for(&self, amt: bool) -> (i64, i64) {
        let (basis, conservation) = self.records(amt);
        let gain = positive(dollars(self.proceeds) - dollars(self.selling) - dollars(basis));
        let section1252 =
            phase::cap(gain, [fraction(dollars(conservation), self.recapture_percent(), 100)]);
        (
            section1252,
            phase::cap(positive(gain - section1252), [dollars(self.cost_sharing_recapture)]),
        )
    }
    // Form 4797 line 29: full recapture through five years, then
    // 80/60/40/20%; no recapture during the tenth or later holding year.
    fn recapture_percent(&self) -> i64 {
        if self.sold <= crate::dates::add_years(self.acquired, 5) {
            100
        } else if self.sold <= crate::dates::add_years(self.acquired, 6) {
            80
        } else if self.sold <= crate::dates::add_years(self.acquired, 7) {
            60
        } else if self.sold <= crate::dates::add_years(self.acquired, 8) {
            40
        } else if self.sold <= crate::dates::add_years(self.acquired, 9) {
            20
        } else {
            0
        }
    }
    pub(crate) fn calculate(&self) -> LandGain {
        self.calculate_for(false)
    }
    pub(crate) fn calculate_for(&self, amt: bool) -> LandGain {
        let (basis, _) = self.records(amt);
        let gain = dollars(self.proceeds) - dollars(self.selling) - dollars(basis);
        if self.acquisition == FarmLandAcquisition::Purchased
            && !crate::dates::held_more_than_one_year(self.acquired, self.sold)
        {
            return LandGain {
                gain,
                percent: 0,
                recapture: 0,
                ordinary: gain,
                section1231: 0,
                unrecaptured: 0,
            };
        }
        let percent = self.recapture_percent();
        let (recapture, cost_sharing) = self.ordinary_pools_for(amt);
        if let Some(form) = self.installment_for(amt) {
            // 2026's part: line 25 is the recapture taken first from line 24.
            return LandGain {
                gain: form.income,
                percent,
                recapture: phase::cap(form.ordinary, [recapture]),
                ordinary: form.ordinary,
                section1231: form.gain,
                unrecaptured: 0,
            };
        }
        let ordinary = recapture + cost_sharing;
        LandGain {
            gain,
            percent,
            recapture,
            ordinary,
            section1231: gain - ordinary,
            unrecaptured: 0,
        }
    }
}

impl FarmLandSale {
    pub(crate) fn valid(&self) -> bool {
        self.id >= 0
            && self.id <= 100_000_000_000
            && self.basis_property > 0
            && self.basis_property <= 100_000_000_000
            && !self.label.trim().is_empty()
            && valid_date(self.sold)
            && (crate::year_constants::TAX_YEAR_START_DATE
                ..=crate::year_constants::TAX_YEAR_END_DATE)
                .contains(&self.sold)
            && [self.proceeds, self.selling].into_iter().all(|v| (0..=100_000_000_000).contains(&v))
            && self.payment.valid(self.proceeds)
    }
}

// Community-property projections preserve accepted source identity and facts.
crate::community::divide_record!(FarmLandSale {
    amounts: [proceeds, selling, payment],
    gains: [],
    facts: [id, basis_property, label, sold, deferral, situation, farm_rental_use],
});

crate::community::divide_enum!(FarmLandAcquisition {
    records: [],
    units: [Purchased, Inherited],
    tuples: [],
});
