// Form 8959 Part V (D-2026-10-19-12). Lines 19 and 20 combine source amounts (W-2 box 6 and
// uncollected tax, box 5 wages) and round that total once. Lines 21 and 22 are computed from
// those printed whole-dollar lines, so the form foots: line 21 = 1.45% of line 20 and
// line 22 = line 19 - line 21, never below zero.
use crate::kernel::ratio::{dollars, fraction, positive};

pub(crate) struct OrdinaryWithholdingLines {
    pub(crate) withheld: i64,
    pub(crate) wages: i64,
    pub(crate) regular: i64,
    pub(crate) additional: i64,
}

/// Printed whole-dollar lines 19 to 22 from the source cents of box 6 and box 5.
pub(crate) fn ordinary_withholding_lines(
    withheld_cents: i64,
    wages_cents: i64,
) -> OrdinaryWithholdingLines {
    let withheld = dollars(withheld_cents);
    let wages = dollars(wages_cents);
    let regular = dollars(fraction(wages * 100, 145, 10000));
    OrdinaryWithholdingLines { withheld, wages, regular, additional: positive(withheld - regular) }
}
