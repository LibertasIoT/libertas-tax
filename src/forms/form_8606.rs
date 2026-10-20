// Form 8606 Part III (D-2026-10-19-12, -14). The Roth owner stays exact in cents. The form
// figures lines 21, 23, 25a and 25c from the printed whole-dollar lines 19, 20, 22, 24 and 25b
// (and the Line 25c Worksheet repayments), so it foots; the taxable amount that crosses to the
// 1040 and Form 5329 is the printed line 25c.
use crate::kernel::ratio::{dollars, positive};

pub(crate) struct PartThree {
    pub l19: i64,
    pub l20: i64,
    pub l21: i64,
    pub l22: i64,
    pub l23: i64,
    pub l24: i64,
    pub l25a: i64,
    pub l25b: i64,
    pub l25c: i64,
}

/// Inputs are the owner's source cents; `repaid` is what the Line 25c Worksheet took off.
pub(crate) fn part_three(
    l19: i64,
    l20: i64,
    l22: i64,
    l24: i64,
    l25b: i64,
    repaid: i64,
) -> PartThree {
    let (l19, l20, l22, l24) = (dollars(l19), dollars(l20), dollars(l22), dollars(l24));
    let (l25b, repaid) = (dollars(l25b), dollars(repaid));
    let l21 = l19 - l20;
    let l23 = positive(l21 - l22);
    let l25a = positive(l23 - l24);
    PartThree { l19, l20, l21, l22, l23, l24, l25a, l25b, l25c: positive(l25a - l25b - repaid) }
}

/// Move the cent amounts that cross to the 1040 so they add to the printed line, taking the
/// difference from the first amounts that can bear it. The difference is under a few dollars.
pub(crate) fn fit_total(values: &mut [i64], target: i64) {
    let mut delta = target - values.iter().sum::<i64>();
    if delta > 0 {
        if let Some(first) = values.iter_mut().find(|v| **v > 0) {
            *first += delta;
        } else if let Some(first) = values.first_mut() {
            *first += delta;
        }
        return;
    }
    for value in values.iter_mut() {
        let take = (-delta).min(*value);
        *value -= take;
        delta += take;
    }
}
