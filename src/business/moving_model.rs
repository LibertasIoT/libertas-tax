use crate::Owner;
use crate::dates::source_valid_date as valid_date;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Move under military orders or an intelligence-community reassignment
/// One for each move. The government reimbursement is read from the W-2 box 12 code P of the person who moved; enter that W-2 on Wages.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MilitaryMove {
    /// Why you moved
    /// Since 2018 only these two moves are deductible (section 217(k)).
    pub reason: MoveReason,
    /// Moving and storing household goods and personal effects (Form 3903 line 1)
    /// Packing, crating, hauling, in-transit storage and insurance for up to 30 days in a row after the goods left your old home. For a move to a foreign country, also storing the goods while the new post stays your main job location and moving them to and from that storage. Not goods bought on the way, and not moving or storage the government provided.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub goods: i64,
    /// Travel from your old home to your new home (Form 3903 line 2)
    /// Leave empty when you paid nothing for the trip, or when you claim only storage fees for an earlier move to a foreign post.
    /// [NullText]
    /// No paid travel claimed for this move
    /// [SetText]
    /// Paid travel from the old home to the new home is
    #[libertas_default($null)]
    pub trip: Option<MoveTrip>,
    /// Government reimbursement not included in your wages (W-2 box 12 code P)
    /// Read from the W-2s of the person who moved; change it on Wages. With several moves by one person, the total is compared once with all of that person's moves. Allowances for dislocation, temporary lodging and move-in housing are never included.
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub reimbursed: i64,
}

/// Why you moved
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum MoveReason {
    /// Armed Forces: on active duty, under a military order, for a permanent change of station
    /// Includes your spouse or dependents moving under your order, the move to your first post, between permanent posts, or home within one year after active duty ends.
    ArmedForcesOrders,
    /// Intelligence community: a change in assignment that required relocation
    /// You are an employee or new appointee of an element of the intelligence community (section 3 of the National Security Act of 1947), not a member of the Armed Forces.
    IntelligenceAssignment,
    /// Another reason
    /// No other move is deductible after 2017; this move is left out of the estimate with a notice.
    Other,
}

/// Travel from your old home to your new home
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MoveTrip {
    /// Lodging, airfare, parking, tolls, and actual gas and oil
    /// One trip per household member, including the day you arrive. No meals, side trips, house-hunting trips or lavish lodging. Enter gas and oil here only when you kept a record of each expense; then claim no miles below for that car.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub paid: i64,
    /// Distance you drove your own car at the standard moving rate
    /// 20.5 cents a mile through June 30, 2026 and 23.5 cents a mile after that; the day you arrived sets the rate. Zero when you entered actual gas and oil above.
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    #[libertas_physical_unit("mile")]
    pub miles: u32,
    /// Date you arrived at your new home
    #[libertas_date_only]
    pub arrived: u32,
}

/// Moving expenses of one filer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MovingClaim {
    /// Filer who moved
    pub owner: Owner,
    /// The move
    pub record: MilitaryMove,
}

impl MoveTrip {
    // Standard-rate car expense in cents, half a cent rounded up.
    pub(crate) fn mileage_cents(&self) -> i64 {
        let half_cents = if self.arrived < crate::year_constants::MOVING_MILEAGE_RATE_CHANGE_DATE {
            crate::year_constants::MOVING_MILEAGE_RATE_2026_THROUGH_JUN_HALF_CENTS
        } else {
            crate::year_constants::MOVING_MILEAGE_RATE_2026_FROM_JUL_HALF_CENTS
        };
        crate::kernel::ratio::half_offset_i64(i64::from(self.miles) * half_cents, 2)
    }
}
impl MilitaryMove {
    // Form 3903 line 3 in cents.
    pub(crate) fn line3_cents(&self) -> i64 {
        self.goods + self.trip.as_ref().map_or(0, |t| t.paid + t.mileage_cents())
    }
    pub(crate) fn qualifies(&self) -> bool {
        !matches!(self.reason, MoveReason::Other)
    }
    // The server twins of the client rules.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if self.goods <= 0 && self.trip.is_none() {
            Some(
                crate::validation::ValidationIssue::new("MOVING_ENTER_AN_EXPENSE")
                    .at_field("goods"),
            )
        } else if self.trip.as_ref().is_some_and(|t| !(20260101..=20261231).contains(&t.arrived)) {
            Some(
                crate::validation::ValidationIssue::new("MOVING_ARRIVAL_OUTSIDE_CURRENT_YEAR")
                    .at_field("arrived")
                    .in_field("trip"),
            )
        } else {
            None
        }
    }
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    // The declared ranges, a real calendar date and the rules.
    pub(crate) fn valid(&self) -> bool {
        (0..=100000000000).contains(&self.goods)
            && (0..=100000000000).contains(&self.reimbursed)
            && self.rule().is_none()
            && self.trip.as_ref().is_none_or(|t| {
                (0..=100000000000).contains(&t.paid) && t.miles <= 20000 && valid_date(t.arrived)
            })
    }
}
