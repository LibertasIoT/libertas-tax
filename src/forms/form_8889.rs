// Form 8889 Part I and Form 5329 Part VII (D-2026-10-19-12, -14). The HSA engine stays exact
// in cents. Each form figures its derived lines from the printed whole-dollar lines, so the
// form foots, and the amounts that cross to Schedule 1 (line 13) and Schedule 2 (line 49)
// are those printed final lines.
use crate::kernel::ratio::{dollars, fraction, positive};

/// Source cents of the Form 8889 Part I lines the owner sets directly.
pub(crate) struct ContributionCents {
    pub personal: i64,
    pub before_archer: i64,
    pub archer: i64,
    pub allocated: i64,
    pub catchup: i64,
    pub employer: i64,
    pub funding: i64,
    /// Prior-year excess absorbed by this year's unused room; it adds to line 2 before the cap.
    pub prior_deducted: i64,
}

/// Printed whole-dollar lines 2 to 13.
pub(crate) struct Contribution {
    pub l2: i64,
    pub l3: i64,
    pub l4: i64,
    pub l5: i64,
    pub l6: i64,
    pub l7: i64,
    pub l8: i64,
    pub l9: i64,
    pub l10: i64,
    pub l11: i64,
    pub l12: i64,
    pub l13: i64,
}

pub(crate) fn contribution(c: &ContributionCents) -> Contribution {
    let (l2, l3, l4) = (dollars(c.personal), dollars(c.before_archer), dollars(c.archer));
    let (l6, l7) = (dollars(c.allocated), dollars(c.catchup));
    let (l9, l10) = (dollars(c.employer), dollars(c.funding));
    let l8 = l6 + l7;
    let l11 = l9 + l10;
    let l12 = positive(l8 - l11);
    Contribution {
        l2,
        l3,
        l4,
        l5: positive(l3 - l4),
        l6,
        l7,
        l8,
        l9,
        l10,
        l11,
        l12,
        l13: (l2 + dollars(c.prior_deducted)).min(l12),
    }
}

/// Printed whole-dollar Form 5329 Part VII lines 42 to 49.
pub(crate) struct Excess {
    pub l42: i64,
    pub l43: i64,
    pub l44: i64,
    pub l45: i64,
    pub l46: i64,
    pub l47: i64,
    pub l48: i64,
    pub l49: i64,
}

/// Cents are the engine's lines 42, 43, 44 and 47 and the year-end account value already
/// capped at the engine's line 48 (the value is optional and may exceed any amount).
pub(crate) fn excess(l42: i64, l43: i64, l44: i64, l47: i64, capped_value: i64) -> Excess {
    let (l42, l43, l44, l47) = (dollars(l42), dollars(l43), dollars(l44), dollars(l47));
    let l45 = l43 + l44;
    let l46 = positive(l42 - l45);
    let l48 = l46 + l47;
    Excess {
        l42,
        l43,
        l44,
        l45,
        l46,
        l47,
        l48,
        l49: fraction(l48.min(dollars(capped_value)), 6, 100),
    }
}
