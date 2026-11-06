use super::*;

// Schedule 1-A Part V, 2025–2028. Eligibility and MAGI sources belong to the caller.
pub(crate) fn senior_deduction(status: FederalStatus, eligible: i64, magi: i64) -> i64 {
    if status == FederalStatus::Separate {
        return 0;
    }
    eligible
        * phase::phase_out(
            6000,
            fraction(
                positive(magi - if status == FederalStatus::Joint { 150000 } else { 75000 }),
                6,
                100,
            ),
        )
}
