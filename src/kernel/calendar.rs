// Gregorian calendar primitives shared by source validation and date arithmetic.

/// Whether the DateOnly encoding names an actual Gregorian calendar date.
pub(crate) fn valid_date(date: u32) -> bool {
    let year = date / 10000;
    let month = date / 100 % 100;
    let day = date % 100;
    // DateOnly uses a Gregorian year from 1 through the four-digit wire maximum.
    (1..=9999).contains(&year) && (1..=days_in_month(year, month)).contains(&day)
}

/// Whether `year` is a Gregorian leap year.
pub(crate) fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100))
}

/// The number of days in a month; 0 for a month outside 1 to 12.
pub(crate) fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}
