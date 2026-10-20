// IRS EIC Worksheet A and exact published childless columns, 2023–2025.
// No current-year eligibility is borrowed into an earlier return.
use crate::historical_eic_tables::{EIC_2023, EIC_2024, EIC_2025};
use crate::kernel::phase;

pub(crate) fn cutoff(year: u32, joint: bool) -> Option<i64> {
    Some(match (year, joint) {
        (2023, false) => 17640,
        (2023, true) => 24210,
        (2024, false) => 18591,
        (2024, true) => 25511,
        (2025, false) => 19104,
        (2025, true) => 26214,
        _ => return None,
    })
}

pub(crate) fn table(year: u32, income: i64, joint: bool) -> Option<i64> {
    let cutoff = cutoff(year, joint)?;
    if income <= 0 || income >= cutoff {
        return Some(0);
    }
    let rows = match year {
        2023 => EIC_2023,
        2024 => EIC_2024,
        2025 => EIC_2025,
        _ => return None,
    };
    let row = rows.get((crate::kernel::ratio::trunc_i64(income, 50)) as usize)?;
    Some(i64::from(if joint { row.1 } else { row.0 }))
}

pub(crate) fn amount(year: u32, earned: i64, agi: i64, joint: bool) -> Option<i64> {
    let earned_credit = table(year, earned, joint)?;
    let start = match (year, joint) {
        (2023, false) => 9800,
        (2023, true) => 16370,
        (2024, false) => 10330,
        (2024, true) => 17250,
        (2025, false) => 10620,
        (2025, true) => 17730,
        _ => return None,
    };
    // Worksheet A does not substitute a lower phase-in AGI for earned income.
    Some(if agi >= start {
        phase::cap(earned_credit, [table(year, agi, joint)?])
    } else {
        earned_credit
    })
}
