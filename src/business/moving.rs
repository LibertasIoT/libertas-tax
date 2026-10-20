// Coverage: Form 3903 moving expenses, Schedule 1 line 14 and Form 1040 line 1h
//   (2026 drafts: Form 3903 created 2026-04-13, its instructions dated 2026-08-27, Schedule 1
//   created 2026-04-24; 26 U.S.C. 217(g), (h)(1), (k)(1)-(2) and 132(g) as amended by P.L. 119-21
//   section 70113, effective for tax years beginning after 2025). Status: Supported. For 2026 only
//   two movers deduct: an Armed Forces member on active duty moving under a military order
//   incident to a permanent change of station, and an intelligence-community employee or new
//   appointee relocating on a change in assignment. No 217(c) distance or time test applies to
//   them. Asked on More adjustments as the "Moving expenses" kind (MovingItem, one row per move,
//   owner inside allowed_owners), stored as FederalAdjustments.moving (MovingClaim). Lines:
//   certification box = MilitaryMove.reason (Other is left out under the answers-and-unknowns
//   decision with WARN_MOVING_LEFT_OUT); line 1 = goods; line 2 = MoveTrip.paid plus miles at the
//   standard rate by arrival date (20.5 cents through 2026-06-30, 23.5 cents from 2026-07-01,
//   year_constants.rs, half a cent rounded up); line 3 = lines 1 + 2; line 4 = the owner's W-2
//   box 12 code P (W2Code::P, never entered on the move: MilitaryMove.reimbursed is shown
//   read-only and re-derived by reconcile); line 5 "Yes" = line 3 less line 4 to Schedule 1
//   line 14 (moving_expense_deduction), line 5 "No" = the excess of line 4 over line 3 to Form 1040
//   line 1h (moving_excess_reimbursement, earned income). Lines 3 to 5 are figured once per owner
//   across that owner's qualifying moves against the owner's one code P total (inventor decision
//   of 2026-10-08; the W-2 gives one total per employer, not per move). A code P with no
//   qualifying move for its owner is a blocking remaining item (REVIEW_MOVING_EXPENSES_NEEDED,
//   the HSA code W convention). Excluded: the Schedule 1 line 14 storage-only checkbox (a printed
//   document matter; the amount path is the same) and the Form 2555 allocation (Form 2555 is
//   refused by name). Dislocation, temporary lodging and move-in housing allowances are never in
//   code P and never entered. The 2026 Form 1040 instructions draft (2026-02-25) still says
//   Armed Forces only; the Code and the later Form 3903 and Schedule 1 drafts govern, and the
//   step 4 recheck covers 3903, its instructions and the rates. Tests: src/tests/business/moving.rs
//   (fixtures A to E, import mapping, rules and their server twins, Search text).
use crate::federal_rules::valid_date;
use crate::kernel::ratio::{dollars, positive};
use crate::{FederalDraft, FederalResult, IssueContext, Owner, TaxSection, W2Code};
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
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub paid: i64,
    /// Miles you drove your own car at the standard moving rate
    /// 20.5 cents a mile through June 30, 2026 and 23.5 cents a mile after that; the day you arrived sets the rate. Zero when you entered actual gas and oil above.
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
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

// An owner's W-2 box 12 code P total in cents (Form 3903 line 4).
pub(crate) fn code_p(d: &FederalDraft, owner: Owner) -> i64 {
    crate::interview::box12_total(d, owner, W2Code::P)
}

// An owner with a code P and no qualifying move to compare it with.
pub(crate) fn claim_missing(d: &FederalDraft) -> bool {
    [Owner::Taxpayer, Owner::Spouse].into_iter().any(|owner| {
        code_p(d, owner) > 0
            && !d
                .adjustments
                .as_ref()
                .is_some_and(|a| a.moving.iter().any(|m| m.owner == owner && m.record.qualifies()))
    })
}

// The stored moves carry their owner's code P: a changed W-2 re-derives it.
pub(crate) fn derive_reimbursed(d: &mut FederalDraft) {
    let reimbursed: alloc::vec::Vec<i64> =
        d.adjustments.iter().flat_map(|a| &a.moving).map(|m| code_p(d, m.owner)).collect();
    if let Some(adjustments) = d.adjustments.as_mut() {
        for (m, amount) in adjustments.moving.iter_mut().zip(reimbursed) {
            m.record.reimbursed = amount;
        }
    }
}

// Form 3903 lines 3 to 5 for each owner, in cents: the deduction (Schedule 1
// line 14) and the reimbursement above the expenses (Form 1040 line 1h).
pub(crate) struct Moving {
    pub(crate) deduction: [i64; 2],
    pub(crate) excess: [i64; 2],
}

pub(crate) fn figure(d: &FederalDraft, r: &mut FederalResult, joint: bool) -> Moving {
    for claim in d.adjustments.iter().flat_map(|a| &a.moving) {
        crate::require_localized!(
            r,
            claim.owner == Owner::Taxpayer || joint,
            &IssueContext::Section(TaxSection::MoreAdjustments),
            "Moving expenses",
            "ISSUE_SPOUSE_MOVE_REQUIRES_A_JOINT_RETURN",
            "A spouse's moving expenses require a joint return.",
        );
    }
    amounts(d)
}

// The same figure without the joint-return check: Form 8379 line 14 reads each
// spouse's own deduction from it.
pub(crate) fn amounts(d: &FederalDraft) -> Moving {
    let mut line3 = [0; 2];
    let mut qualifying = [false; 2];
    for claim in d.adjustments.iter().flat_map(|a| &a.moving) {
        let owner = usize::from(claim.owner == Owner::Spouse);
        if claim.record.qualifies() {
            line3[owner] += claim.record.line3_cents();
            qualifying[owner] = true;
        }
    }
    let mut moving = Moving { deduction: [0; 2], excess: [0; 2] };
    for (i, owner) in [Owner::Taxpayer, Owner::Spouse].into_iter().enumerate() {
        let line4 = code_p(d, owner);
        // An owner with no qualifying move keeps the blocking remaining item
        // (claim_missing) rather than any assumed income.
        if qualifying[i] {
            moving.deduction[i] = positive(line3[i] - line4);
            moving.excess[i] = positive(line4 - line3[i]);
        }
    }
    moving
}

// The whole-dollar amounts of a figure.
impl Moving {
    pub(crate) fn deduction_dollars(&self) -> i64 {
        dollars(self.deduction.iter().sum())
    }
    pub(crate) fn excess_dollars(&self) -> i64 {
        dollars(self.excess.iter().sum())
    }
}
