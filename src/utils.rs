//! Utility date and time equations for Temporal

use alloc::format;
use alloc::string::String;
use date_equations::gregorian;

use crate::MS_PER_DAY;

// NOTE: Review the below for optimizations and add ALOT of tests.

// ==== Begin Date Equations ====

pub(crate) const MS_PER_HOUR: i64 = 3_600_000;
pub(crate) const MS_PER_MINUTE: i64 = 60_000;

/// `EpochDaysToEpochMS`
///
/// Functionally the same as Date's abstract operation `MakeDate`
pub(crate) fn epoch_days_to_epoch_ms(day: i32, time: i64) -> i64 {
    let sign = if day.is_negative() { -1 } else { 1 };
    (day as i64 * MS_PER_DAY as i64) + (time * sign)
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
pub(crate) fn epoch_time_to_day_number(t: i64) -> i32 {
    i64_div_floor(t, MS_PER_DAY as i64) as i32
}

// NOTE: this is pulled in from num_integer
// TODO: Use i64::div_floor when stable.
fn i64_div_floor(lhs: i64, rhs: i64) -> i64 {
    let (div, rem) = (lhs / rhs, lhs % rhs);
    if (rem > 0 && rhs < 0) || (rem < 0 && rhs > 0) {
        div - 1
    } else {
        div
    }
}

#[cfg(feature = "tzdb")]
pub(crate) fn epoch_ms_to_ms_in_day(t: i64) -> u32 {
    (t % i64::from(MS_PER_DAY)) as u32
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

pub(crate) fn epoch_time_to_epoch_year(t: i64) -> i32 {
    let epoch_days = epoch_ms_to_epoch_days(t);
    let (rata_die, shift_constant) = gregorian::rata_die_for_epoch_days(epoch_days, 680);
    gregorian::year(rata_die, shift_constant)
}

/// Returns either 1 (true) or 0 (false)
pub(crate) fn mathematical_in_leap_year(t: i64) -> i32 {
    mathematical_days_in_year(epoch_time_to_epoch_year(t)) - 365
}

/// Returns the epoch day number for a given year.
pub(crate) fn epoch_days_for_year(y: i32) -> i32 {
    365 * (y - 1970) + (y - 1969).div_euclid(4) - (y - 1901).div_euclid(100)
        + (y - 1601).div_euclid(400)
}

// TODO: test limits
pub(crate) fn epoch_time_for_year(y: i32) -> i64 {
    i64::from(MS_PER_DAY) * i64::from(epoch_days_for_year(y))
}

pub(crate) const fn epoch_ms_to_epoch_days(ms: i64) -> i32 {
    (ms / MS_PER_DAY as i64) as i32
}

pub(crate) const fn ymd_from_epoch_milliseconds(epoch_milliseconds: i64) -> (i32, u8, u8) {
    let epoch_days = epoch_ms_to_epoch_days(epoch_milliseconds);
    gregorian::ymd_from_epoch_days(epoch_days, 680)
}

// Returns the time for a month in a given year plus date(t) = 1.
pub(crate) fn epoch_time_for_month_given_year(m: u8, y: i32) -> i64 {
    let leap_day = mathematical_days_in_year(y) - 365;

    // Includes day. i.e. end of month + 1
    let days = month_to_day(m, leap_day as u16);

    i64::from(MS_PER_DAY) * i64::from(days)
}

fn month_to_day(m: u8, leap_day: u16) -> u16 {
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

#[cfg(feature = "tzdb")]
pub(crate) fn epoch_ms_to_month_in_year(t: i64) -> u8 {
    let epoch_days = epoch_ms_to_epoch_days(t);
    let (rata_die, _) = gregorian::rata_die_for_epoch_days(epoch_days, 680);
    gregorian::month(rata_die)
}

#[cfg(feature = "tzdb")]
pub(crate) fn epoch_time_to_day_in_year(t: i64) -> i32 {
    epoch_time_to_day_number(t) - (epoch_days_for_year(epoch_time_to_epoch_year(t)))
}

#[cfg(feature = "tzdb")]
pub(crate) fn epoch_seconds_to_day_of_week(t: i64) -> u16 {
    (((t / 86_400) + 4) % 7) as u16
}

#[cfg(feature = "tzdb")]
pub(crate) fn epoch_seconds_to_day_of_month(t: i64) -> u16 {
    let leap_day = mathematical_in_leap_year(t);
    epoch_time_to_day_in_year(t * 1_000) as u16
        - month_to_day(epoch_ms_to_month_in_year(t * 1_000) - 1, leap_day as u16)
}

// Trait implementations

// EpochTimeTOWeekDay -> REMOVED

// ==== End Date Equations ====

// ==== Begin Calendar Equations ====

// NOTE: below was the iso methods in temporal::calendar -> Need to be reassessed.

/// 12.2.31 `ISODaysInMonth ( year, month )`
pub(crate) fn iso_days_in_month(year: i32, month: i32) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 28 + mathematical_in_leap_year(epoch_time_for_year(year)) as u8,
        _ => unreachable!("ISODaysInMonth panicking is an implementation error."),
    }
}

// The below calendar abstract equations/utilities were removed for being unused.
// 12.2.32 `ToISOWeekOfYear ( year, month, day )`
// 12.2.33 `ISOMonthCode ( month )`
// 12.2.39 `ToISODayOfYear ( year, month, day )`
// 12.2.40 `ToISODayOfWeek ( year, month, day )`

// ==== End Calendar Equations ====
