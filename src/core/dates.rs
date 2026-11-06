// Calendar-date arithmetic on yyyymmdd values (the App's date encoding).
// Shared Gregorian validation and month rules belong to the kernel calendar.
// Callers validate dates before using arithmetic helpers.

pub(crate) use crate::kernel::calendar::{days_in_month, is_leap_year, valid_date};

pub(crate) fn source_valid_date(date: u32) -> bool {
    date / 10000 >= 1850 && valid_date(date)
}

pub(crate) fn valid_birth(date: u32) -> bool {
    date / 10000 <= 2026 && source_valid_date(date)
}
// A date a rental can first be available to rent on a 2026 return: through
// 2027, the year an exchange's replacement property can still be received
// (section 1031(a)(3)); not a birth date.
pub(crate) fn valid_service_date(date: u32) -> bool {
    date / 10000 <= 2027 && source_valid_date(date)
}

/// The last day of a month, as yyyymmdd.
pub(crate) fn last_day_of_month(year: u32, month: u32) -> u32 {
    year * 10000 + month * 100 + days_in_month(year, month)
}

/// The day number of a yyyymmdd date (day 1 is 0001-01-01), so that the
/// difference of two day numbers is the days between the dates.
pub(crate) fn ordinal(date: u32) -> i32 {
    let date = date as i32;
    let y = date / 10000;
    let m = date / 100 % 100;
    let d = date % 100;
    let before = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334][(m - 1) as usize];
    365 * (y - 1) + (y - 1) / 4 - (y - 1) / 100
        + (y - 1) / 400
        + before
        + d
        + i32::from(m > 2 && is_leap_year(y as u32))
}

/// The days from `from` to `to`; negative when `to` is earlier.
pub(crate) fn days_between(from: u32, to: u32) -> i32 {
    ordinal(to) - ordinal(from)
}

/// The date `days` calendar days after `date` (before it when negative), as yyyymmdd.
pub(crate) fn add_days(date: u32, days: i32) -> u32 {
    let target = ordinal(date) + days;
    let mut year = (date / 10000) as i32 + days / 365;
    while ordinal(year as u32 * 10000 + 101) > target {
        year -= 1;
    }
    while ordinal((year as u32 + 1) * 10000 + 101) <= target {
        year += 1;
    }
    let mut left = target - ordinal(year as u32 * 10000 + 101);
    let mut month = 1;
    while left >= days_in_month(year as u32, month) as i32 {
        left -= days_in_month(year as u32, month) as i32;
        month += 1;
    }
    year as u32 * 10000 + month * 100 + left as u32 + 1
}

/// The same month and day `years` later. February 29 has no such day in a
/// common year; its anniversary there is March 1, so "before the anniversary"
/// holds exactly through February 28.
pub(crate) fn add_years(date: u32, years: u32) -> u32 {
    let (year, month, day) = (date / 10000 + years, date / 100 % 100, date % 100);
    if day > days_in_month(year, month) {
        return (year * 100 + month + 1) * 100 + 1;
    }
    year * 10000 + month * 100 + day
}

/// Held more than one year: disposed after the first anniversary of acquisition.
pub(crate) fn held_more_than_one_year(acquired: u32, disposed: u32) -> bool {
    disposed > add_years(acquired, 1)
}

/// Shift a valid encoded Gregorian date forward by actual calendar days.
/// An invalid date or a result beyond year9999 has no representable value.
// Checked calendar shift for source deadlines; `add_days` keeps the
// unchecked signed shift the disaster rules use.
pub(crate) fn add_days_checked(date: u32, days: u32) -> Option<u32> {
    if !source_valid_date(date) {
        return None;
    }
    let target = ordinal(date).checked_add(i32::try_from(days).ok()?)?;
    if target > ordinal(99991231) {
        return None;
    }
    let (mut low, mut high) = (date / 10000, 9999);
    while low < high {
        let middle = low + (high - low).div_ceil(2);
        if ordinal(middle * 10000 + 101) <= target {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    let mut month = 1;
    while month < 12 && ordinal(low * 10000 + (month + 1) * 100 + 1) <= target {
        month += 1;
    }
    let first = low * 10000 + month * 100 + 1;
    Some(first + u32::try_from(target - ordinal(first)).ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leap_years_and_month_lengths() {
        assert!(is_leap_year(2024) && is_leap_year(2000) && !is_leap_year(1900));
        assert!(!is_leap_year(2026));
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2026, 4), 30);
        assert_eq!(days_in_month(2026, 12), 31);
        assert_eq!(days_in_month(2026, 13), 0);
        assert_eq!(last_day_of_month(2024, 2), 20240229);
        assert_eq!(last_day_of_month(2026, 6), 20260630);
    }

    #[test]
    fn day_numbers_and_differences() {
        assert_eq!(ordinal(10101), 1);
        assert_eq!(days_between(20260101, 20261231), 364);
        assert_eq!(days_between(20240101, 20250101), 366);
        assert_eq!(days_between(20260301, 20260228), -1);
        assert_eq!(days_between(20240228, 20240301), 2);
    }

    #[test]
    fn days_added_and_subtracted() {
        assert_eq!(add_days(20260120, 179), 20260718);
        assert_eq!(add_days(20260301, -180), 20250902);
        assert_eq!(add_days(20260320, 30), 20260419);
        assert_eq!(add_days(20261231, 1), 20270101);
        assert_eq!(add_days(20240228, 1), 20240229);
        assert_eq!(add_days(20260101, 0), 20260101);
        assert_eq!(add_days(20260415, 180), 20261012);
    }

    #[test]
    fn calendar_day_shifts_include_leap_days_and_appeal_deadlines() {
        assert_eq!(add_days_checked(20260228, 1), Some(20260301));
        assert_eq!(add_days_checked(20240228, 1), Some(20240229));
        assert_eq!(add_days_checked(20261231, 120), Some(20270430));
        assert_eq!(add_days_checked(20260431, 1), None);
        assert_eq!(add_days_checked(99991231, 1), None);
    }

    #[test]
    fn anniversaries() {
        assert_eq!(add_years(20240315, 2), 20260315);
        assert_eq!(add_years(20240229, 4), 20280229);
        // No February 29 in 2026: the anniversary is March 1.
        assert_eq!(add_years(20240229, 2), 20260301);
        assert_eq!(add_years(20231231, 1), 20241231);
    }
}
