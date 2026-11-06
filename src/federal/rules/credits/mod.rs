use super::*;

// Schedule R: age qualification and taxable disability-income ceilings.
pub(crate) fn elderly_credit(
    status: FederalStatus,
    apart: bool,
    seniors: usize,
    disability: [i64; 2],
    agi: i64,
    exempt: i64,
) -> i64 {
    let disabled = disability.iter().filter(|v| **v > 0).count();
    if seniors + disabled == 0 || (status == FederalStatus::Separate && !apart) {
        return 0;
    }
    let (base, threshold) = match status {
        FederalStatus::Joint => (if seniors + disabled == 2 { 7500 } else { 5000 }, 10000),
        FederalStatus::Separate => (3750, 5000),
        _ => (5000, 7500),
    };
    let base = if disabled > 0 {
        phase::cap(base, [disability.iter().sum::<i64>() + if seniors > 0 { 5000 } else { 0 }])
    } else {
        base
    };
    fraction(phase::phase_out(base - exempt, fraction(positive(agi - threshold), 1, 2)), 15, 100)
}
