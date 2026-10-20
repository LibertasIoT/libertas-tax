// Source: IRS Pub917 (1994), printed p10 Table2, https://www.irs.gov/pub/irs-prior/p917--1994.pdf;
// Pub463 (1997), printed p18 Maximum Depreciation Limits, https://www.irs.gov/pub/irs-prior/p463--1997.pdf.
// Original MACRS service years only; columns are first, second, third, fourth and succeeding years.
// Amounts are cents. Pre-2003 trucks and vans share these ordinary passenger-auto limits.
const AUTO_CAR_LIMITS_1987_1996: [[i64; 4]; 10] = [
    [256000, 410000, 245000, 147500], // 1987
    [256000, 410000, 245000, 147500], // 1988
    [266000, 420000, 255000, 147500], // 1989
    [266000, 420000, 255000, 147500], // 1990
    [266000, 430000, 255000, 157500], // 1991
    [276000, 440000, 265000, 157500], // 1992
    [286000, 460000, 275000, 167500], // 1993
    [296000, 470000, 285000, 167500], // 1994
    [306000, 490000, 295000, 177500], // 1995
    [306000, 490000, 295000, 177500], // 1996
];

// Source: IRS Pub946 (2025), Maximum Depreciation Deduction; Pub463 (2025), chapter4.
// Original-service-year passenger-auto caps in cents, without first-year additional allowance.
// Columns are first, second, third, fourth and succeeding years. Pre-2003 trucks share car caps.
pub(crate) const AUTO_CAR_LIMITS_1997_2024: [[i64; 4]; 28] = [
    [316000, 500000, 305000, 177500],    // 1997
    [316000, 500000, 295000, 177500],    // 1998
    [306000, 500000, 295000, 177500],    // 1999
    [306000, 490000, 295000, 177500],    // 2000
    [306000, 490000, 295000, 177500],    // 2001
    [306000, 490000, 295000, 177500],    // 2002
    [306000, 490000, 295000, 177500],    // 2003
    [296000, 480000, 285000, 167500],    // 2004
    [296000, 470000, 285000, 167500],    // 2005
    [296000, 480000, 285000, 177500],    // 2006
    [306000, 490000, 285000, 177500],    // 2007
    [296000, 480000, 285000, 177500],    // 2008
    [296000, 480000, 285000, 177500],    // 2009
    [306000, 490000, 295000, 177500],    // 2010
    [306000, 490000, 295000, 177500],    // 2011
    [316000, 510000, 305000, 187500],    // 2012
    [316000, 510000, 305000, 187500],    // 2013
    [316000, 510000, 305000, 187500],    // 2014
    [316000, 510000, 305000, 187500],    // 2015
    [316000, 510000, 305000, 187500],    // 2016
    [316000, 510000, 305000, 187500],    // 2017
    [1000000, 1600000, 960000, 576000],  // 2018
    [1010000, 1610000, 970000, 576000],  // 2019
    [1010000, 1610000, 970000, 576000],  // 2020
    [1020000, 1640000, 980000, 586000],  // 2021
    [1120000, 1800000, 1080000, 646000], // 2022
    [1220000, 1950000, 1170000, 696000], // 2023
    [1240000, 1980000, 1190000, 716000], // 2024
];
// Source: IRS Pub463 (2025), Maximum Depreciation Deduction for Trucks and Vans Placed in Service Prior to2018.
pub(crate) const AUTO_TRUCK_LIMITS_2003_2017: [[i64; 4]; 15] = [
    [336000, 540000, 325000, 197500], // 2003
    [326000, 530000, 315000, 187500], // 2004
    [326000, 520000, 315000, 187500], // 2005
    [326000, 520000, 315000, 187500], // 2006
    [326000, 520000, 305000, 187500], // 2007
    [316000, 510000, 305000, 187500], // 2008
    [306000, 490000, 295000, 177500], // 2009
    [316000, 510000, 305000, 187500], // 2010
    [326000, 520000, 315000, 187500], // 2011
    [336000, 530000, 315000, 187500], // 2012
    [336000, 540000, 325000, 197500], // 2013
    [346000, 550000, 335000, 197500], // 2014
    [346000, 560000, 335000, 197500], // 2015
    [356000, 570000, 335000, 207500], // 2016
    [356000, 570000, 345000, 207500], // 2017
];
// Source: IRS Pub946 (2006), printed p64, Electric Vehicles; current Pub946 retains2003–2006.
// The separately indexed inflation adjustment is not three times the ordinary car's rounded cap.
pub(crate) const AUTO_ELECTRIC_LIMITS_1997_2006: [[i64; 4]; 10] = [
    [948000, 1510000, 905000, 542500], // 1997
    [938000, 1500000, 895000, 542500], // 1998
    [928000, 1490000, 895000, 532500], // 1999
    [928000, 1480000, 885000, 532500], // 2000
    [928000, 1480000, 885000, 532500], // 2001
    [918000, 1470000, 875000, 532500], // 2002
    [908000, 1460000, 875000, 522500], // 2003
    [888000, 1430000, 855000, 512500], // 2004
    [888000, 1420000, 845000, 512500], // 2005
    [898000, 1440000, 865000, 522500], // 2006
];

// Source: Pub946 (2025/2006), automobile tables and their acquisition-date footnotes;
// Rev.Proc.2025-16 and2026-15. `additional_first_year_limit` means the specific
// section280F additional cap applies, not merely that some other bonus provision applies.
// A missing source period is returned as None; callers must retain its named limitation.
pub(crate) fn original_auto_limit_cents(
    acquired: u32,
    placed: u32,
    tax_year: u32,
    kind: &crate::HistoricalAutoKind,
    additional_first_year_limit: bool,
) -> Option<i64> {
    let year = placed / 10000;
    let elapsed = tax_year.checked_sub(year)?;
    let column = elapsed.min(3) as usize;
    let electric = matches!(kind, crate::HistoricalAutoKind::ElectricCar);
    if electric && !(19970806..=20061231).contains(&placed) {
        return None;
    }
    let limits = if electric {
        *AUTO_ELECTRIC_LIMITS_1997_2006.get(year.checked_sub(1997)? as usize)?
    } else if matches!(kind, crate::HistoricalAutoKind::TruckVan) && (2003..=2017).contains(&year) {
        AUTO_TRUCK_LIMITS_2003_2017[(year - 2003) as usize]
    } else if year == 2025 || year == 2026 {
        let current = if year == 2025 {
            crate::year_constants::PASSENGER_AUTOMOBILE_LIMITS_2025_CENTS
        } else {
            crate::year_constants::PASSENGER_AUTOMOBILE_LIMITS_2026_CENTS
        };
        [current[1], current[2], current[3], current[4]]
    } else if (1987..=1996).contains(&year) {
        AUTO_CAR_LIMITS_1987_1996[(year - 1987) as usize]
    } else {
        *AUTO_CAR_LIMITS_1997_2024.get(year.checked_sub(1997)? as usize)?
    };
    if elapsed != 0 || !additional_first_year_limit {
        return Some(limits[column]);
    }
    let additional = if year >= 2020 && acquired <= 20170927 {
        0
    } else if year == 2018 && acquired <= 20170927 {
        640_000
    } else if year == 2019 && acquired <= 20170927 {
        480_000
    } else if year >= 2008 {
        800_000
    } else if year == 2004 || (year == 2003 && placed >= 20030506 && acquired >= 20030506) {
        if electric { 2_295_000 } else { 765_000 }
    } else if (2001..=2003).contains(&year) && acquired >= 20010911 {
        if electric { 1_380_000 } else { 460_000 }
    } else {
        0
    };
    Some(limits[0] + additional)
}
