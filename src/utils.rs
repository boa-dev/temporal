//! Utility date and time equations for Temporal

use alloc::format;
use alloc::string::String;

use crate::MS_PER_DAY;

// NOTE: Review the below for optimizations and add ALOT of tests.

// ==== Begin Date Equations ====

pub(crate) const MS_PER_HOUR: f64 = 3_600_000f64;
pub(crate) const MS_PER_MINUTE: f64 = 60_000f64;

/// `EpochDaysToEpochMS`
///
/// Functionally the same as Date's abstract operation `MakeDate`
pub(crate) fn epoch_days_to_epoch_ms(day: i32, time: f64) -> f64 {
    f64::from(day).mul_add(f64::from(MS_PER_DAY), time).floor()
}

/// 3.5.11 PadISOYear ( y )
///
/// returns a String representation of y suitable for inclusion in an ISO 8601 string
pub(crate) fn pad_iso_year(year: i32) -> String {
    if (0..9999).contains(&year) {
        return format!("{:04}", year);
    }
    let year_sign = if year > 0 { "+" } else { "-" };
    let year_string = format!("{:06}", year.abs());
    format!("{year_sign}{year_string}",)
}

/// `EpochTimeToDayNumber`
///
/// This equation is the equivalent to `ECMAScript`'s `Date(t)`
pub(crate) fn epoch_time_to_day_number(t: f64) -> i32 {
    (t / f64::from(MS_PER_DAY)).floor() as i32
}

#[cfg(feature = "tzdb")]
pub(crate) fn epoch_ms_to_ms_in_day(t: f64) -> u32 {
    (t % f64::from(MS_PER_DAY)) as u32
}

/// Mathematically determine the days in a year.
pub(crate) fn mathematical_days_in_year(y: i32) -> i32 {
    if y % 4 != 0 {
        365
    } else if y % 4 == 0 && y % 100 != 0 {
        366
    } else if y % 100 == 0 && y % 400 != 0 {
        365
    } else {
        // Assert that y is divisble by 400 to ensure we are returning the correct result.
        assert_eq!(y % 400, 0);
        366
    }
}

/// Returns the epoch day number for a given year.
pub(crate) fn epoch_days_for_year(y: i32) -> i32 {
    365 * (y - 1970) + (y - 1969).div_euclid(4) - (y - 1901).div_euclid(100)
        + (y - 1601).div_euclid(400)
}

pub(crate) fn epoch_time_for_year(y: i32) -> f64 {
    f64::from(MS_PER_DAY) * f64::from(epoch_days_for_year(y))
}

pub(crate) fn epoch_time_to_epoch_year(t: f64) -> i32 {
    // TODO: optimize lol.
    // There are other alogrithms out there worth looking into
    // for usage.
    // NOTE: The below loop should not have too much of an impact on
    // performance. Looping should only occur maximally at the date
    // limits of +/- 200_000 years.
    // Roughly calculate the largest possible year given the time t,
    // then check and refine the year.
    let day_count = epoch_time_to_day_number(t);
    let mut year = f64::from(day_count).div_euclid(365.2425) as i32 + 1970;
    loop {
        let time = epoch_time_for_year(year) as f64;
        if time <= t && epoch_time_for_year(year + 1) as f64 > t {
            break;
        } else if time < t {
            year += 1;
        } else {
            year -= 1;
        }
    }

    year
}

/// Returns either 1 (true) or 0 (false)
pub(crate) fn mathematical_in_leap_year(t: f64) -> i32 {
    mathematical_days_in_year(epoch_time_to_epoch_year(t)) - 365
}

pub(crate) fn epoch_time_to_month_in_year(t: f64) -> u8 {
    const DAYS: [i32; 11] = [30, 58, 89, 119, 150, 180, 211, 242, 272, 303, 333];
    const LEAP_DAYS: [i32; 11] = [30, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

    let in_leap_year = mathematical_in_leap_year(t) == 1;
    let day = epoch_time_to_day_in_year(t);

    let result = if in_leap_year {
        LEAP_DAYS.binary_search(&day)
    } else {
        DAYS.binary_search(&day)
    };

    match result {
        Ok(i) | Err(i) => i as u8,
    }
}

// Returns the time for a month in a given year plus date(t) = 1.
pub(crate) fn epoch_time_for_month_given_year(m: i32, y: i32) -> f64 {
    let leap_day = mathematical_days_in_year(y) - 365;

    // Includes day. i.e. end of month + 1
    let days = month_to_day(m, leap_day as u16);

    f64::from(MS_PER_DAY) * f64::from(days)
}

fn month_to_day(m: i32, leap_day: u16) -> u16 {
    match m {
        0 => 0,
        1 => 31,
        2 => 59 + leap_day,
        3 => 90 + leap_day,
        4 => 120 + leap_day,
        5 => 151 + leap_day,
        6 => 181 + leap_day,
        7 => 212 + leap_day,
        8 => 243 + leap_day,
        9 => 273 + leap_day,
        10 => 304 + leap_day,
        11 => 334 + leap_day,
        _ => unreachable!(),
    }
}

pub(crate) fn epoch_time_to_date(t: f64) -> u8 {
    const OFFSETS: [i16; 12] = [
        1, -30, -58, -89, -119, -150, -180, -211, -242, -272, -303, -333,
    ];
    let day_in_year = epoch_time_to_day_in_year(t);
    let in_leap_year = mathematical_in_leap_year(t);
    let month = epoch_time_to_month_in_year(t);

    // Cast from i32 to usize should be safe as the return must be 0-11
    let mut date = day_in_year + i32::from(OFFSETS[month as usize]);

    if month >= 2 {
        date -= in_leap_year;
    }

    // This return of date should be < 31.
    date as u8
}

// TODO: Update to u16 (1..=365 + leap_day)
pub(crate) fn epoch_time_to_day_in_year(t: f64) -> i32 {
    epoch_time_to_day_number(t) - (epoch_days_for_year(epoch_time_to_epoch_year(t)))
}

#[cfg(feature = "tzdb")]
pub(crate) fn epoch_seconds_to_day_of_week(t: f64) -> u16 {
    (((t / 86_400.0).floor() + 4.0) % 7.0) as u16
}

#[cfg(feature = "tzdb")]
pub(crate) fn epoch_seconds_to_day_of_month(t: f64) -> u16 {
    let leap_day = mathematical_in_leap_year(t);
    epoch_time_to_day_in_year(t * 1_000.0) as u16
        - month_to_day(
            epoch_time_to_month_in_year(t * 1_000.0) as i32,
            leap_day as u16,
        )
}

// Trait implementations

// EpochTimeTOWeekDay -> REMOVED

// ==== End Date Equations ====

// ==== Begin Calendar Equations ====

// NOTE: below was the iso methods in temporal::calendar -> Need to be reassessed.

/// 12.2.31 `ISODaysInMonth ( year, month )`
pub(crate) fn iso_days_in_month(year: i32, month: i32) -> i32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 28 + mathematical_in_leap_year(epoch_time_for_year(year)),
        _ => unreachable!("ISODaysInMonth panicking is an implementation error."),
    }
}

// The below calendar abstract equations/utilities were removed for being unused.
// 12.2.32 `ToISOWeekOfYear ( year, month, day )`
// 12.2.33 `ISOMonthCode ( month )`
// 12.2.39 `ToISODayOfYear ( year, month, day )`
// 12.2.40 `ToISODayOfWeek ( year, month, day )`

// ==== End Calendar Equations ====

// ==== Tests =====

// TODO(nekevss): Add way more to the below.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_to_month() {
        let oct_2023 = 1_696_459_917_000_f64;
        let mar_1_2020 = 1_583_020_800_000_f64;
        let feb_29_2020 = 1_582_934_400_000_f64;
        let mar_1_2021 = 1_614_556_800_000_f64;

        assert_eq!(epoch_time_to_month_in_year(oct_2023), 9);
        assert_eq!(epoch_time_to_month_in_year(mar_1_2020), 2);
        assert_eq!(mathematical_in_leap_year(mar_1_2020), 1);
        assert_eq!(epoch_time_to_month_in_year(feb_29_2020), 1);
        assert_eq!(mathematical_in_leap_year(feb_29_2020), 1);
        assert_eq!(epoch_time_to_month_in_year(mar_1_2021), 2);
        assert_eq!(mathematical_in_leap_year(mar_1_2021), 0);
    }

    #[test]
    fn time_for_month_and_year() {
        // NOTE: Month is 0-11

        // Test standard year.
        let standard_year_t = epoch_time_for_year(2015);
        assert_eq!(
            epoch_time_to_date(standard_year_t as f64 + epoch_time_for_month_given_year(0, 2015)),
            1,
            "January is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t as f64 + epoch_time_for_month_given_year(1, 2015)),
            1,
            "February is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t as f64 + epoch_time_for_month_given_year(2, 2015)),
            1,
            "March is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t as f64 + epoch_time_for_month_given_year(3, 2015)),
            1,
            "April is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t as f64 + epoch_time_for_month_given_year(4, 2015)),
            1,
            "May is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t as f64 + epoch_time_for_month_given_year(5, 2015)),
            1,
            "June is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t as f64 + epoch_time_for_month_given_year(6, 2015)),
            1,
            "July is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t as f64 + epoch_time_for_month_given_year(7, 2015)),
            1,
            "August is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t as f64 + epoch_time_for_month_given_year(8, 2015)),
            1,
            "September is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t as f64 + epoch_time_for_month_given_year(9, 2015)),
            1,
            "October is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t as f64 + epoch_time_for_month_given_year(10, 2015)),
            1,
            "November is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t as f64 + epoch_time_for_month_given_year(11, 2015)),
            1,
            "December is unaligned."
        );

        // Test leap Year
        let leap_year_t = epoch_time_for_year(2020);
        assert_eq!(
            epoch_time_to_date(leap_year_t as f64 + epoch_time_for_month_given_year(0, 2020)),
            1,
            "January is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t as f64 + epoch_time_for_month_given_year(1, 2020)),
            1,
            "February is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t as f64 + epoch_time_for_month_given_year(2, 2020)),
            1,
            "March is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t as f64 + epoch_time_for_month_given_year(3, 2020)),
            1,
            "April is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t as f64 + epoch_time_for_month_given_year(4, 2020)),
            1,
            "May is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t as f64 + epoch_time_for_month_given_year(5, 2020)),
            1,
            "June is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t as f64 + epoch_time_for_month_given_year(6, 2020)),
            1,
            "July is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t as f64 + epoch_time_for_month_given_year(7, 2020)),
            1,
            "August is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t as f64 + epoch_time_for_month_given_year(8, 2020)),
            1,
            "September is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t as f64 + epoch_time_for_month_given_year(9, 2020)),
            1,
            "October is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t as f64 + epoch_time_for_month_given_year(10, 2020)),
            1,
            "November is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t as f64 + epoch_time_for_month_given_year(11, 2020)),
            1,
            "December is unaligned."
        );
    }

    #[test]
    fn epoch_time_to_epoch_year_tests() {
        let t = -8_640_000_000_000_000.0;
        let day_number = epoch_time_to_day_number(t);
        assert_eq!(day_number, -100_000_000);

        // Potentially 108 depending on euclidean or not. tbd
        let day_number = epoch_days_for_year(-271_821);
        assert_eq!(day_number, -100_000_109);

        let result = epoch_time_to_epoch_year(t);
        assert_eq!(result, -271_821);
    }
}
