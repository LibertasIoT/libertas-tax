// Form 5329 Part IX (D-2026-10-19-12, -14). The required-distribution owner stays exact in
// cents; the form prints lines 52 and 53 and the waiver in whole dollars and figures the
// shortfall that is taxed from those printed lines, so the form foots.
use crate::kernel::ratio::{dollars, positive};

/// The shortfall taxed at one rate: printed line 52 less printed line 53 less the printed waiver.
pub(crate) fn rmd_shortfall(line_52: i64, line_53: i64, waived: i64) -> i64 {
    positive(dollars(line_52) - dollars(line_53) - dollars(waived))
}
