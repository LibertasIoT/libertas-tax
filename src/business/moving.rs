use crate::kernel::ratio::{dollars, positive};
pub use crate::moving_model::{MilitaryMove, MoveReason, MoveTrip, MovingClaim};
use crate::{FederalDraft, FederalResult, IssueContext, Owner, TaxSection, W2Code};

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
