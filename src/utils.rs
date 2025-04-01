#![allow(
    unused,
    reason = "prefer to have unused methods instead of having to gate everything behind features"
)]

//! Utility date and time equations for Temporal
// NOTE: Potentially add more tests.

use crate::MS_PER_DAY;
pub(crate) const MS_PER_HOUR: i64 = 3_600_000;
pub(crate) const MS_PER_MINUTE: i64 = 60_000;

mod neri_schneider;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Epoch {
    millis: i64,
}

impl Epoch {
    /// Creates a new epoch.
    pub(crate) fn new(millis: i64) -> Self {
        Self { millis }
    }

    /// Creates a new epoch from a given epoch second.
    pub(crate) fn from_seconds(secs: i64) -> Self {
        Self {
            millis: secs * 1000,
        }
    }

    /// Creates a new epoch from a year.
    pub(crate) fn from_year(year: i32) -> Self {
        Self::from_days(epoch_days_for_year(year))
    }

    /// `EpochDaysToEpochMS`
    /// Creates a new epoch from a given epoch day.
    ///
    /// Functionally the same as Date's abstract operation `MakeDate`
    pub(crate) fn from_days(days: i32) -> Self {
        Self::new(i64::from(days) * i64::from(MS_PER_DAY))
    }

    /// Creates a new epoch from a year and a day of year (1-based).
    pub(crate) fn from_year_and_day_of_year(year: i32, day: u16) -> Self {
        Self::from_days(epoch_days_for_year(year) + i32::from(day) - 1)
    }

    /// Creates a new epoch from a gregorian date
    pub(crate) fn from_gregorian_date(year: i32, month: u8, day: u8) -> Self {
        Self::from_days(neri_schneider::epoch_days_from_gregorian_date(
            year, month, day,
        ))
    }

    /// Creates a new epoch from a POSIX date.
    pub(crate) fn from_posix_date(year: i32, month: u8, week: u8, day: u8) -> Self {
        let leap_year = in_leap_year(year);
        let days_in_month = days_in_month(month, in_leap_year(year)) - 1;
        let days_to_year = epoch_days_for_year(year);

        let days_to_month =
            days_to_year + i32::from(day_of_year_until_start_of_month(month, leap_year));

        // Month starts in the day...
        let day_offset = self::day_of_week(days_to_month);

        // EXAMPLE:
        //
        // 0   1   2   3   4   5   6
        // sun mon tue wed thu fri sat
        // -   -   -   0   1   2   3
        // 4   5   6   7   8   9   10
        // 11  12  13  14  15  16  17
        // 18  19  20  21  22  23  24
        // 25  26  27  28  29  30  -
        //
        // The day_offset = 3, since the month starts on a wednesday.
        //
        // We're looking for the second friday of the month. Thus, since the month started before
        // a friday, we need to start counting from week 0:
        //
        // day_of_month = (week - u16::from(day_offset <= day)) * 7 + day - day_offset = (2 - 1) * 7 + 5 - 3 = 9
        //
        // This works if the month started on a day before the day we want (day_offset <= day). However, if that's not the
        // case, we need to start counting on week 1. For example, calculate the day of the month for the third monday
        // of the month:
        //
        // day_of_month = (week - u16::from(day_offset <= day)) * 7 + day - day_offset = (3 - 0) * 7 + 1 - 3 = 19
        let mut day_of_month = (week - u8::from(day_offset <= day)) * 7 + day - day_offset;

        // If we're on week 5, we need to clamp to the last valid day.
        if day_of_month > days_in_month {
            day_of_month -= 7
        }

        Self::from_days(days_to_month + i32::from(day_of_month))
    }

    /// Gets the total elapsed milliseconds of this epoch.
    pub(crate) fn millis(self) -> i64 {
        self.millis
    }

    /// Gets the total elapsed seconds of this epoch.
    pub(crate) fn seconds(self) -> i64 {
        self.millis / 1000
    }

    /// `EpochTimeToDayNumber`
    /// Gets the total elapsed days of this epoch.
    ///
    /// This equation is the equivalent to `ECMAScript`'s `Date(t)`
    pub(crate) fn days(self) -> i32 {
        self.millis.div_euclid(i64::from(MS_PER_DAY)) as i32
    }

    /// Gets the total elapsed years of this epoch.
    pub(crate) fn year(self) -> i32 {
        let (rata_die, shift_constant) = neri_schneider::rata_die_for_epoch_days(self.days());
        neri_schneider::year(rata_die, shift_constant)
    }

    /// Returns the year, month and day of the month for a given millisecond epoch.
    pub(crate) fn ymd(self) -> (i32, u8, u8) {
        neri_schneider::ymd_from_epoch_days(self.days())
    }

    /// Returns the total elapsed milliseconds since the last start of day.
    pub(crate) fn millis_since_start_of_day(self) -> u32 {
        (self.millis.rem_euclid(i64::from(MS_PER_DAY))) as u32
    }

    /// Returns `true` if the epoch is within a leap year.
    pub(crate) fn in_leap_year(self) -> bool {
        in_leap_year(self.year())
    }

    /// Returns the month of the year of this epoch (1-based).
    pub(crate) fn month_in_year(self) -> u8 {
        let epoch_days = self.days();
        let (rata_die, _) = neri_schneider::rata_die_for_epoch_days(epoch_days);
        neri_schneider::month(rata_die)
    }

    /// 12.2.31 `ISODaysInMonth ( year, month )`
    ///
    /// Returns the number of days of the current month (1-based).
    pub(crate) fn days_in_month(self) -> u8 {
        days_in_month(self.month_in_year(), self.in_leap_year())
    }
}

/// Returns the elapsed days until the start of the current month (0-based).
fn day_of_year_until_start_of_month(month: u8, leap_year: bool) -> u16 {
    let leap_day = u16::from(leap_year);
    match month {
        1 => 0,
        2 => 31,
        3 => 59 + leap_day,
        4 => 90 + leap_day,
        5 => 120 + leap_day,
        6 => 151 + leap_day,
        7 => 181 + leap_day,
        8 => 212 + leap_day,
        9 => 243 + leap_day,
        10 => 273 + leap_day,
        11 => 304 + leap_day,
        12 => 334 + leap_day,
        _ => unreachable!(),
    }
}

/// Returns the day of the week of the given epoch day.
pub(crate) fn day_of_week(day: i32) -> u8 {
    (day + 4).rem_euclid(7) as u8
}

/// Returns `true` if the year is a leap year.
fn in_leap_year(year: i32) -> bool {
    days_in_year(year) > 365
}

/// Returns the number of days the given year has.
fn days_in_year(year: i32) -> u16 {
    if year % 4 != 0 {
        365
    } else if year % 4 == 0 && year % 100 != 0 {
        366
    } else if year % 100 == 0 && year % 400 != 0 {
        365
    } else {
        // Assert that y is divisble by 400 to ensure we are returning the correct result.
        assert_eq!(year % 400, 0);
        366
    }
}

/// 12.2.31 `ISODaysInMonth ( year, month )`
///
/// Returns the number of days the current month has (1-based).
fn days_in_month(month: u8, leap_year: bool) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 28 + u8::from(leap_year),
        _ => unreachable!("ISODaysInMonth panicking is an implementation error."),
    }
}

/// Returns the number of days since the epoch for a given year.
fn epoch_days_for_year(year: i32) -> i32 {
    365 * (year - 1970) + (year - 1969).div_euclid(4) - (year - 1901).div_euclid(100)
        + (year - 1601).div_euclid(400)
}
